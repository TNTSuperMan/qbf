use crate::ssa::{inline::inline_ssa_history, r#loop::{detect_ssa_loop, try_2step_loop}, parse::build_ssa_from_ir, to_ir::{SSAOpIR, resolve_eval_order, ssa_to_ir}};

use crate::error::SyntaxError;

#[derive(Clone, PartialEq, Debug)]
pub struct IR {
    pub pointer: isize,
    pub opcode: IROp,
}

#[derive(Clone, PartialEq, Debug)]
pub enum IROp {
    Breakpoint,

    Add(u8),
    Set(u8),

    Shift(isize),
    MulAndSetZero(Box<[(isize, u8)]>),
    MovesAndSetZero(Box<[(isize, bool /* is_positive */)]>),

    In,
    Out,

    LoopStart(usize), // end
    LoopEnd(usize), // start
    LoopEndWithOffset(usize, isize), // start, diff

    StartSSA,
    PushSSA(SSAOpIR),
    AssignSSA(u8), // 連番

    End,
}

pub fn parse_to_ir(code: &str) -> Result<Vec<IR>, SyntaxError> {
    let mut insts: Vec<IR> = vec![];
    let mut loop_stack: Vec<usize> = vec![];
    let mut pointer: isize = 0;

    let mut is_flat: bool = true; // LoopEnd時に確定します

    macro_rules! push_inst {
        ($opcode:expr) => {
            insts.push(IR {
                pointer,
                opcode: $opcode,
            })
        };
    }

    for char in code.chars() {
        match char {
            '!' => {
                if cfg!(feature = "debug") {
                    push_inst!(IROp::End);
                }
            }
            '#' => {
                if cfg!(feature = "debug") {
                    push_inst!(IROp::Breakpoint);
                }
            }
            '+' => {
                if let Some(IR { pointer: last_ptr, opcode }) = insts.last_mut() {
                    if *last_ptr == pointer {
                        if let IROp::Add(val) = opcode {
                            *val = val.wrapping_add(1);
                            continue;
                        } else if let IROp::Set(val) = opcode {
                            *val = val.wrapping_add(1);
                            continue;
                        }
                    }
                }
                push_inst!(IROp::Add(1));
            }
            '-' => {
                if let Some(IR { pointer: last_ptr, opcode }) = insts.last_mut() {
                    if *last_ptr == pointer {
                        if let IROp::Add(val) = opcode {
                            *val = val.wrapping_sub(1);
                            continue;
                        } else if let IROp::Set(val) = opcode {
                            *val = val.wrapping_sub(1);
                            continue;
                        }
                    }
                }
                push_inst!(IROp::Add(255));
            }
            '>' => {
                pointer += 1;
            }
            '<' => {
                pointer -= 1;
            }
            '.' => {
                push_inst!(IROp::Out);
            }
            ',' => {
                push_inst!(IROp::In);
            }
            '[' => {
                loop_stack.push(insts.len());
                is_flat = true;
                push_inst!(IROp::LoopStart(usize::MAX));
            }
            ']' => {
                let start = loop_stack.pop().ok_or_else(|| SyntaxError::UnmatchedClosingBracket)?;
                let start_ptr = insts[start].pointer;
                let end = insts.len();
                let end_ptr = pointer;
                let is_ptr_stable = start_ptr == end_ptr;
                let children = &insts[(start+1)..end];

                if !is_ptr_stable {
                    is_flat = false;
                    pointer = start_ptr;
                    if children.len() == 0 {
                        insts.truncate(start);
                        push_inst!(IROp::Shift(end_ptr - start_ptr));
                        continue;
                    }
                } else if is_flat {
                    if children == [IR { opcode: IROp::Add(255), pointer }] {
                        insts.truncate(start);
                        push_inst!(IROp::Set(0));
                        continue;
                    }

                    let mut dests_res: Result<Vec<(isize, u8)>, ()> = children.iter().map(|dest| {
                        if let IR { pointer, opcode: IROp::Add(val) } = dest {
                            Ok((*pointer, *val))
                        } else {
                            Err(())
                        }
                    }).collect();
                    if let Ok(dests) = dests_res.as_mut() {
                        if let Some(decrement_pos) = dests.iter().position(|&dest| dest == (pointer, 255)) {
                            dests.remove(decrement_pos);
                            if dests.iter().all(|&(ptr, _)| ptr != pointer) {
                                insts.truncate(start);

                                if dests.iter().all(|&(_, val)| val == 1 || val == 255) {
                                    let moves = dests.iter().map(
                                        |&(ptr, val)| {
                                            if val == 1 {
                                                (ptr, true)
                                            } else {
                                                (ptr, false)
                                            }
                                        }
                                    ).collect::<Vec<(isize, bool)>>();

                                    push_inst!(IROp::MovesAndSetZero(moves.into_boxed_slice()));
                                    continue;
                                }

                                push_inst!(IROp::MulAndSetZero(dests.clone().into_boxed_slice()));
                                continue;
                            }
                        }
                    }

                    let ssa = build_ssa_from_ir(children);
                    if cfg!(feature = "debug") && ssa.is_some() {
                        let r_ssa = ssa.unwrap();
                        let inlined = inline_ssa_history(&r_ssa, false);
                        if let Some((loop_el, loop_ssa)) = detect_ssa_loop(&inlined) {
                            let fst = inline_ssa_history(&loop_ssa, true);
                            let fst_order = resolve_eval_order(&fst);
                            println!("{start} LOOP [{loop_el}] {fst:?}\n{fst_order:?}");

                            let sec = try_2step_loop(&loop_ssa).unwrap();
                            let sec_order = resolve_eval_order(&sec.0);
                            println!("{sec:?}\n{sec_order:?}\n");
                        } else {
                            let order = resolve_eval_order(&inlined);
                            println!("{start} notloop {inlined:?}\n{order:?}\n");
                        }
                    }

                    is_flat = false;
                }

                insts[start].opcode = IROp::LoopStart(end);
                if is_ptr_stable {
                    insts.push(IR { pointer: end_ptr, opcode: IROp::LoopEnd(start) });
                } else {
                    insts.push(IR { pointer: end_ptr, opcode: IROp::LoopEndWithOffset(start, end_ptr - start_ptr) });
                }
            }
            _ => {}
        }
    }

    insts.push(IR { pointer, opcode: IROp::End });

    if loop_stack.len() != 0 {
        return Err(SyntaxError::UnmatchedOpeningBracket);
    }

    Ok(insts)
}

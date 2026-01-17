#[derive(Clone, PartialEq)]
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
    MoveAdd(isize),

    In,
    Out,

    LoopStart(usize), // end
    LoopEnd(usize), // start
    LoopEndWithOffset(usize, isize), // start, diff

    End,
}

pub fn parse_to_ir(code: &str) -> Result<Vec<IR>, String> {
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
            '#' => { push_inst!(IROp::Breakpoint) }
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
                let start = loop_stack.pop().ok_or_else(|| "Syntax Error: Unmatched closing bracket")?;
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
                    is_flat = false;

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

                                if dests.len() == 1 {
                                    if dests[0].1 == 1 {
                                        push_inst!(IROp::MoveAdd(dests[0].0));
                                        continue;
                                    }
                                }

                                push_inst!(IROp::MulAndSetZero(dests.clone().into_boxed_slice()));
                                continue;
                            }
                        }
                    }
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
        return Err(String::from("Syntax Error: Unmatched opening bracket"))
    }

    Ok(insts)
}

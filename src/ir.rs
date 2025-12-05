#[derive(Clone)]
pub struct IR {
    pub pointer: isize,
    pub opcode: IROp,
}

#[derive(Clone)]
pub enum IROp {
    Breakpoint,

    Add(u8),
    Set(u8),

    Shift(isize),
    MulAndSetZero(Vec<(isize, u8)>),

    In,
    Out,

    LoopStart(usize), // end
    LoopEnd(usize), // start
    LoopEndWithOffset(usize, isize), // start, diff

    End,
}

pub fn parse_to_ir(code: &str) -> Vec<IR> {
    let mut insts: Vec<IR> = Vec::new();
    let mut loop_stack: Vec<usize> = Vec::new();
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
                let start = loop_stack.pop().unwrap();
                let start_ptr = insts[start].pointer;
                let end = insts.len();
                let end_ptr = pointer;
                let is_ptr_stable = start_ptr == pointer;

                if end - start == 2 {
                    if let IROp::Add(255) = insts.last().unwrap().opcode {
                        insts.truncate(insts.len() - 2);
                        push_inst!(IROp::Set(0));
                        continue;
                    }
                }
                if !is_ptr_stable {
                    is_flat = false;
                    pointer = start_ptr;
                    if end - start == 1 {
                        insts.truncate(start);
                        push_inst!(IROp::Shift(end_ptr - start_ptr));
                        continue;
                    }
                } else if is_flat {
                    is_flat = false;
                    let children = &insts[(start+1)..end];

                    let mut dests_res: Result<Vec<(isize, u8)>, ()> = children.iter().map(|dest| {
                        if let IR { pointer, opcode: IROp::Add(val) } = dest {
                            Ok((*pointer, *val))
                        } else {
                            Err(())
                        }
                    }).collect();
                    if let Ok(dests) = dests_res.as_mut() {
                        if let Some(decrement_pos) = dests.iter().position(|&dest| dest == (pointer, 255)) {
                            if !dests.iter().all(|&(ptr, _)| ptr == pointer) {
                                dests.remove(decrement_pos);
                                insts.truncate(start);

                                push_inst!(IROp::MulAndSetZero(dests.to_vec()));
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

    insts
}

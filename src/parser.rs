#[derive(Clone)]
pub struct Instruction {
    pub pointer: isize,
    pub opcode: InstOp,
}

#[derive(Clone)]
pub enum InstOp {
    Breakpoint,

    Add(u8),
    Set(u8),

    Shift(isize),
    MulAndSetZero(Vec<(isize, u8)>),
    MulAndSetZeroTo(isize, Vec<(isize, u8)>),

    In,
    Out,

    LoopStart(usize), // end
    LoopEnd(usize), // start
    LoopEndWithOffset(usize, isize), // start, diff

    End,
}

pub fn parse(code: &str) -> Vec<Instruction> {
    let mut insts: Vec<Instruction> = Vec::new();
    let mut loop_stack: Vec<usize> = Vec::new();
    let mut pointer: isize = 0;

    let mut is_flat: bool = true; // LoopEnd時に確定します

    macro_rules! push_inst {
        ($opcode:expr) => {
            insts.push(Instruction {
                pointer,
                opcode: $opcode,
            })
        };
    }

    for char in code.chars() {
        match char {
            '#' => { push_inst!(InstOp::Breakpoint) }
            '+' => {
                if let Some(Instruction { pointer: last_ptr, opcode }) = insts.last_mut() {
                    if *last_ptr == pointer {
                        if let InstOp::Add(val) = opcode {
                            *val = val.wrapping_add(1);
                            continue;
                        } else if let InstOp::Set(val) = opcode {
                            *val = val.wrapping_add(1);
                            continue;
                        }
                    }
                }
                push_inst!(InstOp::Add(1));
            }
            '-' => {
                if let Some(Instruction { pointer: last_ptr, opcode }) = insts.last_mut() {
                    if *last_ptr == pointer {
                        if let InstOp::Add(val) = opcode {
                            *val = val.wrapping_sub(1);
                            continue;
                        } else if let InstOp::Set(val) = opcode {
                            *val = val.wrapping_sub(1);
                            continue;
                        }
                    }
                }
                push_inst!(InstOp::Add(255));
            }
            '>' => {
                pointer += 1;
            }
            '<' => {
                pointer -= 1;
            }
            '.' => {
                push_inst!(InstOp::Out);
            }
            ',' => {
                push_inst!(InstOp::In);
            }
            '[' => {
                loop_stack.push(insts.len());
                is_flat = true;
                push_inst!(InstOp::LoopStart(usize::MAX));
            }
            ']' => {
                let start = loop_stack.pop().unwrap();
                let start_ptr = insts[start].pointer;
                let end = insts.len();
                let end_ptr = pointer;
                let is_ptr_stable = start_ptr == pointer;

                if end - start == 2 {
                    if let InstOp::Add(255) = insts.last().unwrap().opcode {
                        insts.truncate(insts.len() - 2);
                        push_inst!(InstOp::Set(0));
                        continue;
                    }
                }
                if !is_ptr_stable {
                    pointer = start_ptr;
                    if end - start == 1 {
                        insts.truncate(start);
                        push_inst!(InstOp::Shift(end_ptr - start_ptr));
                        continue;
                    }
                } else if is_flat {
                    is_flat = false;
                    let mut dests_res: Result<Vec<(isize, u8)>, ()> = insts[(start+1)..end].iter().map(|dest| {
                        if let Instruction { pointer, opcode: InstOp::Add(val) } = dest {
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

                                if let Instruction { pointer: source, opcode: InstOp::MulAndSetZero(last_dests) } = insts.last().unwrap().clone() {
                                    if let Some(to_ldests_at) = last_dests.iter().position(|&dest| dest == (pointer, 1)) {
                                        if let Some(from_dests_at) = dests.iter().position(|&dest| dest == (source, 1)) {
                                            dests.remove(from_dests_at);
                                            for (i, dest) in last_dests.iter().enumerate() {
                                                if i != to_ldests_at {
                                                    dests.push(*dest);
                                                }
                                            }
                                            
                                            insts.truncate(start - 1);
                                            push_inst!(InstOp::MulAndSetZeroTo(source, dests.to_vec()));
                                            continue;
                                        }
                                    }
                                }

                                push_inst!(InstOp::MulAndSetZero(dests.to_vec()));
                                continue;
                            }
                        }
                    }
                }

                insts[start].opcode = InstOp::LoopStart(end);
                if is_ptr_stable {
                    insts.push(Instruction { pointer: end_ptr, opcode: InstOp::LoopEnd(start) });
                } else {
                    insts.push(Instruction { pointer: end_ptr, opcode: InstOp::LoopEndWithOffset(start, end_ptr - start_ptr) });
                }
            }
            _ => {}
        }
    }

    insts.push(Instruction { pointer, opcode: InstOp::End });

    insts
}

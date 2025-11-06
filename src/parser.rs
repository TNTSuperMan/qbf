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

    In,
    Out,

    LoopStart(usize), // end
    LoopEnd(usize), // start
    LoopEndWithOffset(usize, isize), // start, diff
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
                    let mut dests = insts[(start+1)..end].to_vec();
                    if let Some(decrement_pos) = dests.iter().position(|dest| if let Instruction { pointer: dest_ptr, opcode: InstOp::Add(255) } = dest { *dest_ptr == pointer } else { false }) {
                        dests.remove(decrement_pos);
                        if dests.iter().all(|inst| if let InstOp::Add(_) = inst.opcode { true } else { false }) {
                            insts.truncate(start);
                            push_inst!(InstOp::MulAndSetZero(dests.iter().map(|dest| {
                                if let Instruction { pointer, opcode: InstOp::Add(val) } = dest {
                                    (*pointer, *val)
                                } else {
                                    unreachable!("InstOp::Addしかないことを上のiter().all()で検証したはず");
                                }
                            }).collect()));
                            continue;
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

    insts
}

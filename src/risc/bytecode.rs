use std::fmt::Debug;

use crate::{ir::{IR, IROp}};

#[derive(Clone)]
pub struct Bytecode {
    pub opcode: OpCode,
    pub delta: i16,
    pub val: u8,
    pub addr: u32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum OpCode {
    Breakpoint,

    Add,
    Set,

    Shift,

    MulStart,
    Mul,

    SingleMoveAdd,
    SingleMoveSub,

    MoveStart,
    MoveAdd,
    MoveSub,

    In,
    Out,

    JmpIfZero, // LoopStart
    JmpIfNotZero, // LoopEnd

    End,
}

impl Debug for Bytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op = match self.opcode {
            OpCode::Breakpoint => format!("brk"),

            OpCode::Add => format!("add {}", self.val),
            OpCode::Set => format!("set {}", self.val),

            OpCode::Shift => format!("shift {}", self.addr as i32),

            OpCode::MulStart => format!("mulstart or jmp {}", self.addr),
            OpCode::Mul => format!("mul {}", self.val),

            OpCode::SingleMoveAdd => format!("smadd {}", self.addr as i32),
            OpCode::SingleMoveSub => format!("smsub {}", self.addr as i32),

            OpCode::MoveStart => format!("mvstart or jmp {}", self.addr as i32),
            OpCode::MoveAdd => format!("madd"),
            OpCode::MoveSub => format!("msub"),

            OpCode::In => format!("in"),
            OpCode::Out => format!("out"),

            OpCode::JmpIfZero => format!("jpz {}", self.addr),
            OpCode::JmpIfNotZero => format!("jpnz {}", self.addr),

            OpCode::End => format!("end"),
        };

        f.write_str(&format!("{} {}", self.delta, op))
    }
}

pub fn ir_to_bytecodes(ir_nodes: &[IR]) -> Result<Vec<Bytecode>, String> {
    let mut bytecodes: Vec<Bytecode> = vec![];
    let mut loop_stack: Vec<usize> = vec![];

    let mut i = 0usize;
    let mut last_ptr = 0isize;

    loop {
        match ir_nodes.get(i) {
            None => {
                // Finalize?
                return Ok(bytecodes);
            }
            Some(node) => {
                let delta = i16::try_from(node.pointer.wrapping_sub(last_ptr)).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                last_ptr = node.pointer;
                match &node.opcode {
                    IROp::Breakpoint => {
                        bytecodes.push(Bytecode {
                            opcode: OpCode::Breakpoint,
                            delta,
                            val: 0,
                            addr: 0,
                        });
                    }

                    IROp::Add(val) => {
                        bytecodes.push(Bytecode {
                            opcode: OpCode::Add,
                            delta,
                            val: *val,
                            addr: 0,
                        });
                    }
                    IROp::Set(val) => {
                        bytecodes.push(Bytecode {
                            opcode: OpCode::Set,
                            delta,
                            val: *val,
                            addr: 0,
                        });
                    }

                    IROp::Shift(step) => {
                        bytecodes.push(Bytecode {
                            opcode: OpCode::Shift,
                            delta,
                            val: 0,
                            addr: *step as i32 as u32,
                        });
                    }
                    IROp::MulAndSetZero(dests) => {
                        let skip_pc = (bytecodes.len() + dests.len() + 1) as u32;

                        bytecodes.push(Bytecode {
                            opcode: OpCode::MulStart,
                            delta,
                            val: 0,
                            addr: skip_pc,
                        });

                        for (dest_ptr, dest_val) in dests {
                            bytecodes.push(Bytecode {
                                opcode: OpCode::Mul,
                                delta: i16::try_from(dest_ptr.wrapping_sub(last_ptr)).map_err(|_| "Optimization Error: Pointer Delta Overflow")?,
                                val: *dest_val,
                                addr: 0,
                            });
                        }
                    }
                    IROp::MovesAndSetZero(dests) => {
                        let skip_pc = (bytecodes.len() + dests.len() + 1) as u32;

                        bytecodes.push(Bytecode {
                            opcode: OpCode::MoveStart,
                            delta,
                            val: 0,
                            addr: skip_pc,
                        });

                        for (dest_ptr, is_pos) in dests {
                            if *is_pos {
                                bytecodes.push(Bytecode {
                                    opcode: OpCode::MoveAdd,
                                    delta: i16::try_from(dest_ptr.wrapping_sub(last_ptr)).map_err(|_| "Optimization Error: Pointer Delta Overflow")?,
                                    val: 0,
                                    addr: 0,
                                });
                            } else {
                                bytecodes.push(Bytecode {
                                    opcode: OpCode::MoveSub,
                                    delta: i16::try_from(dest_ptr.wrapping_sub(last_ptr)).map_err(|_| "Optimization Error: Pointer Delta Overflow")?,
                                    val: 0,
                                    addr: 0,
                                });
                            }
                        }
                    }
                    IROp::MoveAdd(dest) => {
                        bytecodes.push(Bytecode {
                            opcode: OpCode::SingleMoveAdd,
                            delta,
                            val: 0,
                            addr: i32::try_from(dest - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")? as u32,
                        });
                    }
                    IROp::MoveSub(dest) => {
                        bytecodes.push(Bytecode {
                            opcode: OpCode::SingleMoveSub,
                            delta,
                            val: 0,
                            addr: i32::try_from(dest - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")? as u32,
                        });
                    }

                    IROp::In => {
                        bytecodes.push(Bytecode {
                            opcode: OpCode::In,
                            delta,
                            val: 0,
                            addr: 0,
                        });
                    }
                    IROp::Out => {
                        bytecodes.push(Bytecode {
                            opcode: OpCode::Out,
                            delta,
                            val: 0,
                            addr: 0,
                        });
                    }

                    IROp::LoopStart(_end) => {
                        loop_stack.push(bytecodes.len());
                        bytecodes.push(Bytecode {
                            opcode: OpCode::JmpIfZero,
                            delta,
                            val: 0,
                            addr: u32::MAX,
                        });
                    }
                    IROp::LoopEnd(_start) => {
                        let start = loop_stack.pop().unwrap();
                        let end = bytecodes.len();
                        bytecodes[start].addr = (end + 1) as u32;
                        bytecodes.push(Bytecode {
                            opcode: OpCode::JmpIfNotZero,
                            delta,
                            val: 0,
                            addr: (start + 1) as u32,
                        });
                    }
                    IROp::LoopEndWithOffset(_start, offset) => {
                        let start = loop_stack.pop().unwrap();
                        let end = bytecodes.len();
                        last_ptr -= offset;
                        bytecodes[start].addr = (end + 1) as u32;
                        bytecodes.push(Bytecode {
                            opcode: OpCode::JmpIfNotZero,
                            delta,
                            val: 0,
                            addr: (start + 1) as u32,
                        });
                    }

                    IROp::End => {
                        bytecodes.push(Bytecode {
                            opcode: OpCode::End,
                            delta,
                            val: 0,
                            addr: 0
                        });
                    }
                }
            }
        }
        i += 1;
    }
}

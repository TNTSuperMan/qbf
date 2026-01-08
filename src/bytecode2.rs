use crate::ir::{IR, IROp};

#[derive(Clone, Debug)]
pub struct Bytecode2 {
    pub opcode: OpCode2,
    pub delta: i16,
    pub val: u8,
    pub addr: u32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum OpCode2 {
    Breakpoint,

    SingleAdd,
    SingleSet,
    AddAdd,
    AddSet,
    SetAdd,
    SetSet,

    Shift,
    ShiftAdd,
    ShiftSet,

    MulStart,
    Mul,
    MulLast,

    In,
    Out,

    JmpIfZero, // LoopStart
    JmpIfNotZero, // LoopEnd

    End,
}

pub fn ir_to_bytecodes2(ir_nodes: &[IR]) -> Result<Vec<Bytecode2>, String> {
    let mut bytecodes: Vec<Bytecode2> = vec![];
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
                        bytecodes.push(Bytecode2 {
                            opcode: OpCode2::Breakpoint,
                            delta,
                            val: 0,
                            addr: 0,
                        });
                    }

                    IROp::Add(val1) => {
                        match ir_nodes[i + 1] {
                            IR { opcode: IROp::Add(val2), pointer: ptr2 } => {
                                let delta2 = i16::try_from(ptr2 - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                                last_ptr = ptr2;
                                bytecodes.push(Bytecode2 {
                                    opcode: OpCode2::AddAdd,
                                    delta,
                                    val: *val1,
                                    addr: (delta2 as u16 as u32) | ((val2 as u32) << 16),
                                });
                                i += 2;
                                continue;
                            }
                            IR { opcode: IROp::Set(val2), pointer: ptr2 } => {
                                let delta2 = i16::try_from(ptr2 - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                                last_ptr = ptr2;
                                bytecodes.push(Bytecode2 {
                                    opcode: OpCode2::AddSet,
                                    delta,
                                    val: *val1,
                                    addr: (delta2 as u16 as u32) | ((val2 as u32) << 16),
                                });
                                i += 2;
                                continue;
                            }
                            _ => {
                                bytecodes.push(Bytecode2 {
                                    opcode: OpCode2::SingleAdd,
                                    delta,
                                    val: *val1,
                                    addr: 0,
                                });
                            }
                        }
                    }
                    IROp::Set(val1) => {
                        match ir_nodes[i + 1] {
                            IR { opcode: IROp::Add(val2), pointer: ptr2 } => {
                                let delta2 = i16::try_from(ptr2 - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                                last_ptr = ptr2;
                                bytecodes.push(Bytecode2 {
                                    opcode: OpCode2::SetAdd,
                                    delta,
                                    val: *val1,
                                    addr: (delta2 as u16 as u32) | ((val2 as u32) << 16),
                                });
                                i += 2;
                                continue;
                            }
                            IR { opcode: IROp::Set(val2), pointer: ptr2 } => {
                                let delta2 = i16::try_from(ptr2 - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                                last_ptr = ptr2;
                                bytecodes.push(Bytecode2 {
                                    opcode: OpCode2::SetSet,
                                    delta,
                                    val: *val1,
                                    addr: (delta2 as u16 as u32) | ((val2 as u32) << 16),
                                });
                                i += 2;
                                continue;
                            }
                            _ => {
                                bytecodes.push(Bytecode2 {
                                    opcode: OpCode2::SingleSet,
                                    delta,
                                    val: *val1,
                                    addr: 0,
                                });
                            }
                        }
                    }

                    IROp::Shift(step) => {
                        if let Ok(step_i8) = i8::try_from(*step) {
                            match ir_nodes[i + 1] {
                                IR { opcode: IROp::Add(val), pointer: ptr } => {
                                    let delta2 = i16::try_from(ptr - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                                    last_ptr = ptr;
                                    bytecodes.push(Bytecode2 {
                                        opcode: OpCode2::ShiftAdd,
                                        delta,
                                        val: step_i8 as u8,
                                        addr: (delta2 as u16 as u32) | ((val as u32) << 16),
                                    });
                                    i += 2;
                                    continue;
                                }
                                IR { opcode: IROp::Set(val), pointer: ptr } => {
                                    let delta2 = i16::try_from(ptr - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                                    last_ptr = ptr;
                                    bytecodes.push(Bytecode2 {
                                        opcode: OpCode2::ShiftSet,
                                        delta,
                                        val: step_i8 as u8,
                                        addr: (delta2 as u16 as u32) | ((val as u32) << 16),
                                    });
                                    i += 2;
                                    continue;
                                }
                                _ => { /* 下のフローで処理 */ }
                            }
                        }
                        bytecodes.push(Bytecode2 {
                            opcode: OpCode2::Shift,
                            delta,
                            val: 0,
                            addr: *step as i32 as u32,
                        });
                    }
                    IROp::MulAndSetZero(dests) => {
                        let skip_pc = (bytecodes.len() + dests.len() + 1) as u32;

                        bytecodes.push(Bytecode2 {
                            opcode: OpCode2::MulStart,
                            delta,
                            val: 0,
                            addr: skip_pc,
                        });

                        let ((l_ptr, l_val), rest) = dests.split_last().unwrap(); // SAFETY: dests要素は1つ以上存在するはず

                        for (dest_ptr, dest_val) in rest {
                            let delta = i16::try_from(dest_ptr.wrapping_sub(last_ptr)).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                            last_ptr = *dest_ptr;
                            bytecodes.push(Bytecode2 {
                                opcode: OpCode2::Mul,
                                delta,
                                val: *dest_val,
                                addr: 0,
                            });
                        }

                        bytecodes.push(Bytecode2 {
                            opcode: OpCode2::MulLast,
                            delta: i16::try_from(l_ptr.wrapping_sub(last_ptr)).map_err(|_| "Optimization Error: Pointer Delta Overflow")?,
                            val: *l_val,
                            addr: (node.pointer - l_ptr) as i32 as u32,
                        });

                        last_ptr = node.pointer;
                    }

                    IROp::In => {
                        bytecodes.push(Bytecode2 {
                            opcode: OpCode2::In,
                            delta,
                            val: 0,
                            addr: 0,
                        });
                    }
                    IROp::Out => {
                        bytecodes.push(Bytecode2 {
                            opcode: OpCode2::Out,
                            delta,
                            val: 0,
                            addr: 0,
                        });
                    }

                    IROp::LoopStart(_end) => {
                        loop_stack.push(bytecodes.len());
                        bytecodes.push(Bytecode2 {
                            opcode: OpCode2::JmpIfZero,
                            delta,
                            val: 0,
                            addr: u32::MAX,
                        });
                    }
                    IROp::LoopEnd(_start) => {
                        let start = loop_stack.pop().unwrap();
                        let end = bytecodes.len();
                        bytecodes[start].addr = (end + 1) as u32;
                        bytecodes.push(Bytecode2 {
                            opcode: OpCode2::JmpIfNotZero,
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
                        bytecodes.push(Bytecode2 {
                            opcode: OpCode2::JmpIfNotZero,
                            delta,
                            val: 0,
                            addr: (start + 1) as u32,
                        });
                    }

                    IROp::End => {
                        bytecodes.push(Bytecode2 {
                            opcode: OpCode2::End,
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

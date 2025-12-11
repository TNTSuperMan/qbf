use crate::ir::{IR, IROp};

#[derive(Clone)]
pub struct Bytecode {
    pub opcode: OpCode /* u8 */,
    pub val: u8,
    pub ptr: isize,
    pub ptr2: isize,
    pub addr: usize,

    _padding1: u32,
    _padding2: u16,
}

#[derive(Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    Breakpoint,

    Add,
    Set,

    Shift, // ptr: shift source, ptr3: shift size
    MulStart, // addr: skip addr
    Mul,
    AddFromMemory,

    SingleMul, // ptr: to, ptr2: from, val
    SingleAddFM, // ptr: to, ptr2: from

    In,
    Out,

    LoopStart, // ptr: condition, addr: addr(+1)
    LoopEnd,   // ptr: condition, addr: addr(+1)
    LoopEndWithOffset, // ptr: condition, ptr2: offset, addr: addr(+1)

    End,
}

pub fn ir_to_bytecodes(ir: Vec<IR>) -> Vec<Bytecode> {
    let mut bytecodes: Vec<Bytecode> = vec![];
    let mut loop_stack: Vec<usize> = vec![];

    for ir in ir {
        match ir.opcode {
            IROp::Breakpoint => {
                bytecodes.push(Bytecode {
                    opcode: OpCode::Breakpoint,
                    val: 0,
                    ptr: ir.pointer,
                    ptr2: 0,
                    addr: 0,
                    _padding1: 0,
                    _padding2: 0,
                });
            }
            IROp::Add(val) => {
                bytecodes.push(Bytecode {
                    opcode: OpCode::Add,
                    val,
                    ptr: ir.pointer,
                    ptr2: 0,
                    addr: 0,
                    _padding1: 0,
                    _padding2: 0,
                });
            }
            IROp::Set(val) => {
                bytecodes.push(Bytecode {
                    opcode: OpCode::Set,
                    val,
                    ptr: ir.pointer,
                    ptr2: 0,
                    addr: 0,
                    _padding1: 0,
                    _padding2: 0,
                });
            }
            
            IROp::Shift(s) => {
                bytecodes.push(Bytecode {
                    opcode: OpCode::Shift,
                    val: 0,
                    ptr: ir.pointer,
                    ptr2: s,
                    addr: 0,
                    _padding1: 0,
                    _padding2: 0,
                });
            }
            IROp::MulAndSetZero(dests) => {
                if dests.len() == 1 {
                    let (dest_ptr, dest_val) = dests[0]; // SAFETY: 要素が一つあることを確認済み
                    match dest_val {
                        0 => {
                            /* 下の通常のバイトコード生成にスキップ */
                        }
                        1 => {
                            bytecodes.push(Bytecode {
                                opcode: OpCode::SingleAddFM,
                                val: 0,
                                ptr: dest_ptr,
                                ptr2: ir.pointer,
                                addr: 0,
                                _padding1: 0,
                                _padding2: 0,
                            });
                            continue;
                        }
                        _ => {
                            bytecodes.push(Bytecode {
                                opcode: OpCode::SingleMul,
                                val: dest_val,
                                ptr: dest_ptr,
                                ptr2: ir.pointer,
                                addr: 0,
                                _padding1: 0,
                                _padding2: 0,
                            });
                            continue;
                        }
                    }
                }
                bytecodes.push(Bytecode {
                    opcode: OpCode::MulStart,
                    val: 0,
                    ptr: ir.pointer,
                    ptr2: ir.pointer,
                    addr: bytecodes.len() + dests.len() + 1,
                    _padding1: 0,
                    _padding2: 0,
                });
                for (ptr, val) in dests {
                    match val {
                        0 => {
                            /* 何もする必要は無い */
                        }
                        1 => {
                            bytecodes.push(Bytecode {
                                opcode: OpCode::AddFromMemory,
                                val: 0,
                                ptr,
                                ptr2: 0,
                                addr: 0,
                                _padding1: 0,
                                _padding2: 0,
                            });
                        }
                        _ => {
                            bytecodes.push(Bytecode {
                                opcode: OpCode::Mul,
                                val,
                                ptr,
                                ptr2: 0,
                                addr: 0,
                                _padding1: 0,
                                _padding2: 0,
                            });
                        }
                    }
                }
            }

            IROp::In => {
                bytecodes.push(Bytecode {
                    opcode: OpCode::In,
                    val: 0,
                    ptr: ir.pointer,
                    ptr2: 0,
                    addr: 0,
                    _padding1: 0,
                    _padding2: 0,
                });
            }
            IROp::Out => {
                bytecodes.push(Bytecode {
                    opcode: OpCode::Out,
                    val: 0,
                    ptr: ir.pointer,
                    ptr2: 0,
                    addr: 0,
                    _padding1: 0,
                    _padding2: 0,
                });
            }

            IROp::LoopStart(_end) => {
                loop_stack.push(bytecodes.len());
                bytecodes.push(Bytecode {
                    opcode: OpCode::LoopStart,
                    val: 0,
                    ptr: ir.pointer,
                    ptr2: 0,
                    addr: usize::MAX,
                    _padding1: 0,
                    _padding2: 0,
                });
            }
            IROp::LoopEnd(_start) => {
                let start = loop_stack.pop().unwrap();
                let end = bytecodes.len();
                bytecodes[start].addr = end + 1;
                bytecodes.push(Bytecode {
                    opcode: OpCode::LoopEnd,
                    val: 0,
                    ptr: ir.pointer,
                    ptr2: 0,
                    addr: start + 1,
                    _padding1: 0,
                    _padding2: 0,
                });
            }
            IROp::LoopEndWithOffset(_start, offset) => {
                let start = loop_stack.pop().unwrap();
                let end = bytecodes.len();
                bytecodes[start].addr = end + 1;
                bytecodes.push(Bytecode {
                    opcode: OpCode::LoopEndWithOffset,
                    val: 0,
                    ptr: ir.pointer,
                    ptr2: offset,
                    addr: start + 1,
                    _padding1: 0,
                    _padding2: 0,
                });
            }

            IROp::End => {
                bytecodes.push(Bytecode {
                    opcode: OpCode::End,
                    val: 0,
                    ptr: ir.pointer,
                    ptr2: 0,
                    addr: 0,
                    _padding1: 0,
                    _padding2: 0,
                });
            }
        }
    }

    bytecodes
}


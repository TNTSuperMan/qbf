use std::fmt::Debug;

use crate::{ir::{IR, IROp}, range::{MemoryRange, RangeInfo}};

// メモ: jz ゼロ時ジャンプ jnz 非ゼロ時ジャンプ

#[derive(Clone, Copy, Debug)]
pub enum NewBytecode {
    Breakpoint { delta: i16 },

    SingleAdd { delta: i16, val: u8 },
    SingleSet { delta: i16, val: u8 },
    AddAdd { delta1: i16, val1: u8, delta2: i16, val2: u8 },
    AddSet { delta1: i16, val1: u8, delta2: i16, val2: u8 },
    SetAdd { delta1: i16, val1: u8, delta2: i16, val2: u8 },
    SetSet { delta1: i16, val1: u8, delta2: i16, val2: u8 },

    BothRangeCheck { positive: u16, negative: u16 },
    Shift  { delta: i16, step: i16 },
    ShiftP { delta: i16, step: i16, range: u16 },
    ShiftN { delta: i16, step: i16, range: u16 },
    ShiftAdd  { delta1: i16, step: i8, delta2: i8, val: u8 },
    ShiftAddP { delta1: i16, step: i8, delta2: i8, val: u8, range: u16 },
    ShiftAddN { delta1: i16, step: i8, delta2: i8, val: u8, range: u16 },
    ShiftSet  { delta1: i16, step: i8, delta2: i8, val: u8 },
    ShiftSetP { delta1: i16, step: i8, delta2: i8, val: u8, range: u16 },
    ShiftSetN { delta1: i16, step: i8, delta2: i8, val: u8, range: u16 },

    MulStart { delta: i16, jz: u32 },
    Mul { delta: i16, val: u8 },

    SingleMoveAdd { delta: i16, to: i16 },
    SingleMoveSub { delta: i16, to: i16 },

    DoubleMoveAddAdd { delta: i16, to1: i16, to2: i16 },
    DoubleMoveAddSub { delta: i16, to1: i16, to2: i16 },
    DoubleMoveSubAdd { delta: i16, to1: i16, to2: i16 },
    DoubleMoveSubSub { delta: i16, to1: i16, to2: i16 },

    MoveStart { delta: i16, jz: u32 },
    MoveAdd { delta: i16 },
    MoveSub { delta: i16 },

    In { delta: i16 },
    Out { delta: i16 },

    JmpIfZero { delta: i16, addr: u32 },
    JmpIfNotZero { delta: i16, addr: u32 },
    PositiveRangeCheckJNZ { delta: i16, addr_subrel: u16, range: u16 },
    NegativeRangeCheckJNZ { delta: i16, addr_subrel: u16, range: u16 },
    BothRangeCheckJNZ { delta: i8, addr_subrel: u16, positive: u16, negative: u16 },

    End { delta: i16 },
}

pub fn ir_to_bytecodes(ir_nodes: &[IR], range_info: &RangeInfo) -> Result<Vec<NewBytecode>, String> {
    let mut bytecodes: Vec<NewBytecode> = vec![];
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
                        bytecodes.push(NewBytecode::Breakpoint { delta });
                    }

                    IROp::Add(val1) => {
                        match ir_nodes[i + 1] {
                            IR { opcode: IROp::Add(val2), pointer: ptr2 } => {
                                let delta2 = i16::try_from(ptr2 - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                                last_ptr = ptr2;
                                bytecodes.push(NewBytecode::AddAdd { delta1: delta, val1: *val1, delta2, val2 });
                                i += 2;
                                continue;
                            }
                            IR { opcode: IROp::Set(val2), pointer: ptr2 } => {
                                let delta2 = i16::try_from(ptr2 - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                                last_ptr = ptr2;
                                bytecodes.push(NewBytecode::AddSet { delta1: delta, val1: *val1, delta2, val2 });
                                i += 2;
                                continue;
                            }
                            _ => {
                                bytecodes.push(NewBytecode::SingleAdd { delta, val: *val1 });
                            }
                        }
                    }
                    IROp::Set(val1) => {
                        match ir_nodes[i + 1] {
                            IR { opcode: IROp::Add(val2), pointer: ptr2 } => {
                                let delta2 = i16::try_from(ptr2 - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                                last_ptr = ptr2;
                                bytecodes.push(NewBytecode::SetAdd { delta1: delta, val1: *val1, delta2, val2 });
                                i += 2;
                                continue;
                            }
                            IR { opcode: IROp::Set(val2), pointer: ptr2 } => {
                                let delta2 = i16::try_from(ptr2 - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                                last_ptr = ptr2;
                                bytecodes.push(NewBytecode::SetSet { delta1: delta, val1: *val1, delta2, val2 });
                                i += 2;
                                continue;
                            }
                            _ => {
                                bytecodes.push(NewBytecode::SingleSet { delta, val: *val1 });
                            }
                        }
                    }

                    IROp::Shift(step) => {
                        let range = range_info.map.get(&i).unwrap();
                        if let MemoryRange::Both { positive, negative } = range {
                            bytecodes.push(NewBytecode::Shift { delta, step: *step as i16 });
                            bytecodes.push(NewBytecode::BothRangeCheck { positive: *positive, negative: *negative });
                            i += 1;
                            continue;
                        }
                        if let Ok(step_i8) = i8::try_from(*step) {
                            match ir_nodes[i + 1] {
                                IR { opcode: IROp::Add(val), pointer: ptr } => {
                                    let delta2 = i8::try_from(ptr - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                                    last_ptr = ptr;
                                    match range {
                                        MemoryRange::None => bytecodes.push(NewBytecode::ShiftAdd { delta1: delta, step: step_i8, delta2, val }),
                                        MemoryRange::Positive(r) => bytecodes.push(NewBytecode::ShiftAddP { delta1: delta, step: step_i8, delta2, val, range: *r }),
                                        MemoryRange::Negative(r) => bytecodes.push(NewBytecode::ShiftAddN { delta1: delta, step: step_i8, delta2, val, range: *r }),
                                        MemoryRange::Both { positive: _, negative: _ } => { unreachable!(); /* 上でMemoryRange::Bothは処理済みのはず */ }
                                    }
                                    i += 2;
                                    continue;
                                }
                                IR { opcode: IROp::Set(val), pointer: ptr } => {
                                    let delta2 = i8::try_from(ptr - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                                    last_ptr = ptr;
                                    match range {
                                        MemoryRange::None => bytecodes.push(NewBytecode::ShiftSet { delta1: delta, step: step_i8, delta2, val }),
                                        MemoryRange::Positive(r) => bytecodes.push(NewBytecode::ShiftSetP { delta1: delta, step: step_i8, delta2, val, range: *r }),
                                        MemoryRange::Negative(r) => bytecodes.push(NewBytecode::ShiftSetN { delta1: delta, step: step_i8, delta2, val, range: *r }),
                                        MemoryRange::Both { positive: _, negative: _ } => { unreachable!(); /* 上でMemoryRange::Bothは処理済みのはず */ }
                                    }
                                    i += 2;
                                    continue;
                                }
                                _ => { /* 下のフローで処理 */ }
                            }
                        }
                        match range {
                            MemoryRange::None => bytecodes.push(NewBytecode::Shift { delta, step: *step as i16 }),
                            MemoryRange::Positive(r) => bytecodes.push(NewBytecode::ShiftP { delta, step: *step as i16, range: *r }),
                            MemoryRange::Negative(r) => bytecodes.push(NewBytecode::ShiftN { delta, step: *step as i16, range: *r }),
                            MemoryRange::Both { positive: _, negative: _ } => { unreachable!(); /* 上でMemoryRange::Bothは処理済みのはず */ }
                        };
                    }
                    IROp::MulAndSetZero(dests) => {
                        let skip_pc = (bytecodes.len() + dests.len() + 1) as u32;

                        bytecodes.push(NewBytecode::MulStart { delta, jz: skip_pc });

                        for (dest_ptr, dest_val) in dests {
                            bytecodes.push(NewBytecode::Mul { delta: i16::try_from(dest_ptr.wrapping_sub(last_ptr)).map_err(|_| "Optimization Error: Pointer Delta Overflow")?, val: *dest_val });
                        }
                    }
                    IROp::MovesAndSetZero(dests) => {
                        if let [(p1, f1), (p2, f2)] = dests.iter().as_slice() {
                            let delta1 = i16::try_from(p1.wrapping_sub(last_ptr)).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                            let delta2 = i16::try_from(p2.wrapping_sub(last_ptr)).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;

                            match (*f1, *f2) {
                                (true, true) =>   bytecodes.push(NewBytecode::DoubleMoveAddAdd { delta, to1: delta1, to2: delta2 }),
                                (true, false) =>  bytecodes.push(NewBytecode::DoubleMoveAddSub { delta, to1: delta1, to2: delta2 }),
                                (false, true) =>  bytecodes.push(NewBytecode::DoubleMoveSubAdd { delta, to1: delta1, to2: delta2 }),
                                (false, false) => bytecodes.push(NewBytecode::DoubleMoveSubSub { delta, to1: delta1, to2: delta2 }),
                            };
                        } else {
                            let skip_pc = (bytecodes.len() + dests.len() + 1) as u32;

                            bytecodes.push(NewBytecode::MoveStart { delta, jz: skip_pc });

                            for (dest_ptr, is_pos) in dests {
                                if *is_pos {
                                    bytecodes.push(NewBytecode::MoveAdd {
                                        delta: i16::try_from(dest_ptr.wrapping_sub(last_ptr)).map_err(|_| "Optimization Error: Pointer Delta Overflow")?,
                                    });
                                } else {
                                    bytecodes.push(NewBytecode::MoveSub {
                                        delta: i16::try_from(dest_ptr.wrapping_sub(last_ptr)).map_err(|_| "Optimization Error: Pointer Delta Overflow")?,
                                    });
                                }
                            }
                        }
                    }
                    IROp::MoveAdd(dest) => {
                        bytecodes.push(NewBytecode::SingleMoveAdd {
                            delta,
                            to: i16::try_from(dest - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?,
                        });
                    }
                    IROp::MoveSub(dest) => {
                        bytecodes.push(NewBytecode::SingleMoveSub {
                            delta,
                            to: i16::try_from(dest - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?,
                        });
                    }

                    IROp::In => {
                        bytecodes.push(NewBytecode::In { delta });
                    }
                    IROp::Out => {
                        bytecodes.push(NewBytecode::Out { delta });
                    }

                    IROp::LoopStart(_end) => {
                        loop_stack.push(bytecodes.len());
                        bytecodes.push(NewBytecode::JmpIfZero { delta, addr: u32::MAX });
                    }
                    IROp::LoopEnd(_start) => {
                        let start = loop_stack.pop().unwrap();
                        let end = bytecodes.len();
                        if let NewBytecode::JmpIfZero { addr, .. } = &mut bytecodes[start] {
                            *addr = (end + 1) as u32;
                        } else {
                            return Err("InternalError: Corresponding JmpIfZero is not hit".to_owned());
                        }
                        bytecodes.push(NewBytecode::JmpIfNotZero { delta, addr: (start + 1) as u32 });
                    }
                    IROp::LoopEndWithOffset(_start, offset) => {
                        let range = range_info.map.get(&i).unwrap();
                        let start = loop_stack.pop().unwrap();
                        let end = bytecodes.len();
                        last_ptr -= offset;
                        if let NewBytecode::JmpIfZero { addr, .. } = &mut bytecodes[start] {
                            *addr = (end + 1) as u32;
                        } else {
                            return Err("InternalError: Corresponding JmpIfZero is not hit".to_owned());
                        }
                        let subrel = end - start + 1;
                        match range {
                            MemoryRange::None => bytecodes.push(NewBytecode::JmpIfNotZero { delta, addr: (start + 1) as u32 }),
                            MemoryRange::Positive(r) => bytecodes.push(NewBytecode::PositiveRangeCheckJNZ { delta, addr_subrel: subrel as u16, range: *r }),
                            MemoryRange::Negative(r) => bytecodes.push(NewBytecode::NegativeRangeCheckJNZ { delta, addr_subrel: subrel as u16, range: *r }),
                            MemoryRange::Both { positive, negative } => bytecodes.push(NewBytecode::BothRangeCheckJNZ { delta: i8::try_from(delta).map_err(|_| "OptimizationError: delta Overflow")?, addr_subrel: subrel as u16, positive: *positive, negative: *negative }),
                        }
                    }

                    IROp::End => {
                        bytecodes.push(NewBytecode::End { delta });
                    }
                }
            }
        }
        i += 1;
    }
}

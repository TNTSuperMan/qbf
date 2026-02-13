use std::{fmt::Debug, ops::{Range, RangeFrom, RangeTo}};

use crate::{ir::{IR, IROp}, range::{MidRange, RangeInfo}};

// メモ: jz ゼロ時ジャンプ jnz 非ゼロ時ジャンプ

#[derive(Clone, Debug)]
pub enum Bytecode {
    Breakpoint { delta: i16 },

    SingleAdd { delta: i16, val: u8 },
    SingleSet { delta: i16, val: u8 },
    AddAdd { delta1: i16, val1: u8, delta2: i16, val2: u8 },
    AddSet { delta1: i16, val1: u8, delta2: i16, val2: u8 },
    SetAdd { delta1: i16, val1: u8, delta2: i16, val2: u8 },
    SetSet { delta1: i16, val1: u8, delta2: i16, val2: u8 },

    BothRangeCheck { range: Range<u16> },
    Shift  { delta: i16, step: i16 },
    ShiftN { delta: i16, step: i16, range: RangeFrom<u16> },
    ShiftP { delta: i16, step: i16, range: RangeTo<u16> },
    ShiftAdd  { delta1: i16, step: i8, delta2: i8, val: u8 },
    ShiftAddN { delta1: i16, step: i8, delta2: i8, val: u8, range: RangeFrom<u16> },
    ShiftAddP { delta1: i16, step: i8, delta2: i8, val: u8, range: RangeTo<u16> },
    ShiftSet  { delta1: i16, step: i8, delta2: i8, val: u8 },
    ShiftSetN { delta1: i16, step: i8, delta2: i8, val: u8, range: RangeFrom<u16> },
    ShiftSetP { delta1: i16, step: i8, delta2: i8, val: u8, range: RangeTo<u16> },

    MulStart { delta: i16, jz_abs: u32 },
    Mul { delta: i16, val: u8 },

    SingleMoveAdd { delta: i16, to: i16 },
    SingleMoveSub { delta: i16, to: i16 },

    DoubleMoveAddAdd { delta: i16, to1: i16, to2: i16 },
    DoubleMoveAddSub { delta: i16, to1: i16, to2: i16 },
    DoubleMoveSubAdd { delta: i16, to1: i16, to2: i16 },
    DoubleMoveSubSub { delta: i16, to1: i16, to2: i16 },

    MoveStart { delta: i16, jz_abs: u32 },
    MoveAdd { delta: i16 },
    MoveSub { delta: i16 },

    In { delta: i16 },
    Out { delta: i16 },

    JmpIfZero { delta: i16, addr_abs: u32 },
    JmpIfNotZero { delta: i16, addr_abs: u32 },
    NegativeRangeCheckJNZ { delta: i16, addr_back: u16, range: RangeFrom<u16> },
    PositiveRangeCheckJNZ { delta: i16, addr_back: u16, range: RangeTo<u16> },
    BothRangeCheckJNZ { delta: i8, addr_back: u16, range: Range<u16> },

    End { delta: i16 },
}

pub fn ir_to_bytecodes(ir_nodes: &[IR], range_info: &RangeInfo) -> Result<Vec<Bytecode>, String> {
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
                        bytecodes.push(Bytecode::Breakpoint { delta });
                    }

                    IROp::Add(val1) => {
                        match ir_nodes[i + 1] {
                            IR { opcode: IROp::Add(val2), pointer: ptr2 } => {
                                let delta2 = i16::try_from(ptr2 - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                                last_ptr = ptr2;
                                bytecodes.push(Bytecode::AddAdd { delta1: delta, val1: *val1, delta2, val2 });
                                i += 2;
                                continue;
                            }
                            IR { opcode: IROp::Set(val2), pointer: ptr2 } => {
                                let delta2 = i16::try_from(ptr2 - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                                last_ptr = ptr2;
                                bytecodes.push(Bytecode::AddSet { delta1: delta, val1: *val1, delta2, val2 });
                                i += 2;
                                continue;
                            }
                            _ => {
                                bytecodes.push(Bytecode::SingleAdd { delta, val: *val1 });
                            }
                        }
                    }
                    IROp::Set(val1) => {
                        match ir_nodes[i + 1] {
                            IR { opcode: IROp::Add(val2), pointer: ptr2 } => {
                                let delta2 = i16::try_from(ptr2 - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                                last_ptr = ptr2;
                                bytecodes.push(Bytecode::SetAdd { delta1: delta, val1: *val1, delta2, val2 });
                                i += 2;
                                continue;
                            }
                            IR { opcode: IROp::Set(val2), pointer: ptr2 } => {
                                let delta2 = i16::try_from(ptr2 - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                                last_ptr = ptr2;
                                bytecodes.push(Bytecode::SetSet { delta1: delta, val1: *val1, delta2, val2 });
                                i += 2;
                                continue;
                            }
                            _ => {
                                bytecodes.push(Bytecode::SingleSet { delta, val: *val1 });
                            }
                        }
                    }

                    IROp::Shift(step) => {
                        let mid_range = range_info.map.get(&i).unwrap();
                        if let MidRange::Both(range) = mid_range {
                            bytecodes.push(Bytecode::Shift { delta, step: *step as i16 });
                            bytecodes.push(Bytecode::BothRangeCheck { range: range.clone() });
                            i += 1;
                            continue;
                        }
                        if let Ok(step_i8) = i8::try_from(*step) {
                            match ir_nodes[i + 1] {
                                IR { opcode: IROp::Add(val), pointer: ptr } => {
                                    let delta2 = i8::try_from(ptr - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                                    last_ptr = ptr;
                                    match mid_range {
                                        MidRange::None => bytecodes.push(Bytecode::ShiftAdd { delta1: delta, step: step_i8, delta2, val }),
                                        MidRange::Positive(range) => bytecodes.push(Bytecode::ShiftAddP { delta1: delta, step: step_i8, delta2, val, range: *range }),
                                        MidRange::Negative(range) => bytecodes.push(Bytecode::ShiftAddN { delta1: delta, step: step_i8, delta2, val, range: range.clone() }),
                                        MidRange::Both { .. } => { unreachable!(); /* 上でMemoryRange::Bothは処理済みのはず */ }
                                    }
                                    i += 2;
                                    continue;
                                }
                                IR { opcode: IROp::Set(val), pointer: ptr } => {
                                    let delta2 = i8::try_from(ptr - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                                    last_ptr = ptr;
                                    match mid_range {
                                        MidRange::None => bytecodes.push(Bytecode::ShiftSet { delta1: delta, step: step_i8, delta2, val }),
                                        MidRange::Positive(range) => bytecodes.push(Bytecode::ShiftSetP { delta1: delta, step: step_i8, delta2, val, range: *range }),
                                        MidRange::Negative(range) => bytecodes.push(Bytecode::ShiftSetN { delta1: delta, step: step_i8, delta2, val, range: range.clone() }),
                                        MidRange::Both { .. } => { unreachable!(); /* 上でMemoryRange::Bothは処理済みのはず */ }
                                    }
                                    i += 2;
                                    continue;
                                }
                                _ => { /* 下のフローで処理 */ }
                            }
                        }
                        match mid_range {
                            MidRange::None => bytecodes.push(Bytecode::Shift { delta, step: *step as i16 }),
                            MidRange::Positive(range) => bytecodes.push(Bytecode::ShiftP { delta, step: *step as i16, range: *range }),
                            MidRange::Negative(range) => bytecodes.push(Bytecode::ShiftN { delta, step: *step as i16, range: range.clone() }),
                            MidRange::Both { .. } => { unreachable!(); /* 上でMemoryRange::Bothは処理済みのはず */ }
                        };
                    }
                    IROp::MulAndSetZero(dests) => {
                        let skip_pc = (bytecodes.len() + dests.len() + 1) as u32;

                        bytecodes.push(Bytecode::MulStart { delta, jz_abs: skip_pc });

                        for (dest_ptr, dest_val) in dests {
                            bytecodes.push(Bytecode::Mul { delta: i16::try_from(dest_ptr.wrapping_sub(last_ptr)).map_err(|_| "Optimization Error: Pointer Delta Overflow")?, val: *dest_val });
                        }
                    }
                    IROp::MovesAndSetZero(dests) => {
                        if let [(p1, f1), (p2, f2)] = dests.iter().as_slice() {
                            let delta1 = i16::try_from(p1.wrapping_sub(last_ptr)).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;
                            let delta2 = i16::try_from(p2.wrapping_sub(last_ptr)).map_err(|_| "Optimization Error: Pointer Delta Overflow")?;

                            match (*f1, *f2) {
                                (true, true) =>   bytecodes.push(Bytecode::DoubleMoveAddAdd { delta, to1: delta1, to2: delta2 }),
                                (true, false) =>  bytecodes.push(Bytecode::DoubleMoveAddSub { delta, to1: delta1, to2: delta2 }),
                                (false, true) =>  bytecodes.push(Bytecode::DoubleMoveSubAdd { delta, to1: delta1, to2: delta2 }),
                                (false, false) => bytecodes.push(Bytecode::DoubleMoveSubSub { delta, to1: delta1, to2: delta2 }),
                            };
                        } else {
                            let skip_pc = (bytecodes.len() + dests.len() + 1) as u32;

                            bytecodes.push(Bytecode::MoveStart { delta, jz_abs: skip_pc });

                            for (dest_ptr, is_pos) in dests {
                                if *is_pos {
                                    bytecodes.push(Bytecode::MoveAdd {
                                        delta: i16::try_from(dest_ptr.wrapping_sub(last_ptr)).map_err(|_| "Optimization Error: Pointer Delta Overflow")?,
                                    });
                                } else {
                                    bytecodes.push(Bytecode::MoveSub {
                                        delta: i16::try_from(dest_ptr.wrapping_sub(last_ptr)).map_err(|_| "Optimization Error: Pointer Delta Overflow")?,
                                    });
                                }
                            }
                        }
                    }
                    IROp::MoveAdd(dest) => {
                        bytecodes.push(Bytecode::SingleMoveAdd {
                            delta,
                            to: i16::try_from(dest - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?,
                        });
                    }
                    IROp::MoveSub(dest) => {
                        bytecodes.push(Bytecode::SingleMoveSub {
                            delta,
                            to: i16::try_from(dest - last_ptr).map_err(|_| "Optimization Error: Pointer Delta Overflow")?,
                        });
                    }

                    IROp::In => {
                        bytecodes.push(Bytecode::In { delta });
                    }
                    IROp::Out => {
                        bytecodes.push(Bytecode::Out { delta });
                    }

                    IROp::LoopStart(_end) => {
                        loop_stack.push(bytecodes.len());
                        bytecodes.push(Bytecode::JmpIfZero { delta, addr_abs: u32::MAX });
                    }
                    IROp::LoopEnd(_start) => {
                        let start = loop_stack.pop().unwrap();
                        let end = bytecodes.len();
                        if let Bytecode::JmpIfZero { addr_abs: addr, .. } = &mut bytecodes[start] {
                            *addr = (end + 1) as u32;
                        } else {
                            return Err("InternalError: Corresponding JmpIfZero is not hit".to_owned());
                        }
                        bytecodes.push(Bytecode::JmpIfNotZero { delta, addr_abs: (start + 1) as u32 });
                    }
                    IROp::LoopEndWithOffset(_start, offset) => {
                        let range = range_info.map.get(&i).unwrap();
                        let start = loop_stack.pop().unwrap();
                        let end = bytecodes.len();
                        last_ptr -= offset;
                        if let Bytecode::JmpIfZero { addr_abs: addr, .. } = &mut bytecodes[start] {
                            *addr = (end + 1) as u32;
                        } else {
                            return Err("InternalError: Corresponding JmpIfZero is not hit".to_owned());
                        }
                        let subrel = end - start - 1;
                        match range {
                            MidRange::None => bytecodes.push(Bytecode::JmpIfNotZero { delta, addr_abs: (start + 1) as u32 }),
                            MidRange::Positive(range) => bytecodes.push(Bytecode::PositiveRangeCheckJNZ { delta, addr_back: subrel as u16, range: *range }),
                            MidRange::Negative(range) => bytecodes.push(Bytecode::NegativeRangeCheckJNZ { delta, addr_back: subrel as u16, range: range.clone() }),
                            MidRange::Both(range) => bytecodes.push(Bytecode::BothRangeCheckJNZ { delta: i8::try_from(delta).map_err(|_| "OptimizationError: delta Overflow")?, addr_back: subrel as u16, range: range.clone() }),
                        }
                    }

                    IROp::End => {
                        bytecodes.push(Bytecode::End { delta });
                    }
                }
            }
        }
        i += 1;
    }
}

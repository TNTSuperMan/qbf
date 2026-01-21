use std::collections::HashMap;

use crate::ir::{IR, IROp};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sign {
    Positive,
    Negative,
}
impl Sign {
    pub fn isize_to_sign(num: isize) -> Sign {
        if num >= 0 {
            Sign::Positive
        } else {
            Sign::Negative
        }
    }
}

struct InternalRangeInfo {
    map: HashMap<usize, (Sign, isize, isize, isize)>, // pointer, pos, neg
    curr_positive: isize,
    curr_negative: isize,
}
impl InternalRangeInfo {
    pub fn new() -> InternalRangeInfo {
        InternalRangeInfo {
            map: HashMap::new(),
            curr_positive: isize::MIN,
            curr_negative: isize::MAX,
        }
    }
    pub fn subscribe(&mut self, pointer: isize) {
        if self.curr_negative > pointer {
            self.curr_negative = pointer;
        }
        if self.curr_positive < pointer {
            self.curr_positive = pointer;
        }
    }
    pub fn insert(&mut self, ir_at: usize, sign: Sign, pointer: isize) {
        self.map.insert(ir_at, (sign, pointer, self.curr_positive, self.curr_negative));
        self.curr_positive = pointer;
        self.curr_negative = pointer;
    }
    pub fn apply_loop(&mut self, ir_at: usize) {
        let ri = self.map.get_mut(&ir_at).unwrap();
        if ri.2 < self.curr_positive {
            ri.2 = self.curr_positive;
        }
        if ri.3 > self.curr_negative {
            ri.3 = self.curr_negative;
        }
    }
}

pub struct RangeInfo {
    pub map: HashMap<usize, RangeData>,
    pub do_opt_first: bool,
}

#[derive(Debug)]
pub enum RangeData {
    Positive(u16),
    Negative(u16),
    Both(u16, u16), // pos, neg
}
impl RangeInfo {
    fn from(internal_ri: &InternalRangeInfo) -> Result<RangeInfo, String> {
        let map_arr: Result<Vec<(usize, RangeData)>, String> = internal_ri.map.iter().map(|(&ir_at, &(sign, ptr, pos, neg))| {
            match sign {
                Sign::Positive => {
                    let pos16 = i16::try_from(ptr - pos).map_err(|_| "OptimizationError: Pointer Range Overflow")?;
                    if neg >= ptr {
                        Ok((ir_at, RangeData::Positive(pos16.wrapping_sub(1) as u16)))
                    } else {
                        let pos16n = i16::try_from(ptr - neg).map_err(|_| "OptimizationError: Pointer Range Overflow")?;
                        Err(format!("Err+ neg:{} ptr:{} pos:{} pos16:{}, 16n:{}", neg, ptr, pos, pos16 as u16, pos16n as u16))
                    }
                }
                Sign::Negative => {
                    let pos16 = i16::try_from(ptr - neg).map_err(|_| "OptimizationError: Pointer Range Overflow")?;
                    if pos <= ptr {
                        Ok((ir_at, RangeData::Negative(pos16 as u16)))
                    } else {
                        let pos16n = i16::try_from(ptr - pos).map_err(|_| "OptimizationError: Pointer Range Overflow")?;
                        Err(format!("Err- neg:{} ptr:{} pos:{} pos16:{}, 16n:{}", neg, ptr, pos, pos16 as u16, pos16n.wrapping_sub(1) as u16))
                    }
                }
            }
        }).collect();
        Ok(RangeInfo {
            map: HashMap::from_iter(map_arr?),
            do_opt_first: !(internal_ri.curr_negative < 0) && !(internal_ri.curr_positive >= 65536),
        })
    }
}

pub fn generate_range_info(ir_nodes: &[IR]) -> Result<RangeInfo, String> {
    let mut internal_ri = InternalRangeInfo::new();

    for (i, IR { pointer, opcode }) in ir_nodes.iter().enumerate().rev() {
        internal_ri.subscribe(*pointer);
        match opcode {
            IROp::Shift(step) => {
                internal_ri.insert(i, Sign::isize_to_sign(*step), *pointer);
            }
            IROp::MulAndSetZero(dests) => {
                for (ptr, _val) in dests {
                    internal_ri.subscribe(*ptr);
                }
            }
            IROp::MovesAndSetZero(dests) => {
                for (ptr, _val) in dests {
                    internal_ri.subscribe(*ptr);
                }
            }
            IROp::MoveAdd(dest) => {
                internal_ri.subscribe(*dest);
            }
            IROp::MoveSub(dest) => {
                internal_ri.subscribe(*dest);
            }
            IROp::LoopStart(end) => {
                if let IROp::LoopEndWithOffset(_, _) = ir_nodes[*end].opcode {
                    internal_ri.apply_loop(*end);
                }
            }
            IROp::LoopEndWithOffset(_start, offset) => {
                internal_ri.insert(i, Sign::isize_to_sign(*offset), *pointer);
            }
            _ => {}
        }
    }

    Ok(RangeInfo::from(&internal_ri)?)
}

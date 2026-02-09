use std::{cmp::{max, min}, collections::HashMap};

use crate::ir::{IR, IROp};

pub fn positive_is_out_of_range(range: u16, pointer: usize) -> bool {
    pointer >= (range as usize)
}
pub fn negative_is_out_of_range(range: u16, pointer: usize) -> bool {
    pointer < (range as usize)
}

#[derive(Debug)]
struct RSMapElement {
    pointer: isize,
    positive: isize,
    negative: isize,
}
#[derive(Debug)]
struct InternalRangeState {
    map: HashMap<usize, RSMapElement>,
    curr_positive: isize,
    curr_negative: isize,
}
impl InternalRangeState {
    pub fn new() -> InternalRangeState {
        InternalRangeState {
            map: HashMap::new(),
            curr_positive: isize::MIN,
            curr_negative: isize::MAX,
        }
    }
    pub fn subscribe(&mut self, pointer: isize) {
        self.curr_positive = max(self.curr_positive, pointer);
        self.curr_negative = min(self.curr_negative, pointer);
    }
    pub fn insert(&mut self, ir_at: usize, pointer: isize) {
        self.map.insert(ir_at, RSMapElement {
            pointer,
            positive: self.curr_positive,
            negative: self.curr_negative
        });
        self.curr_positive = pointer;
        self.curr_negative = pointer;
    }
    pub fn apply_loop(&mut self, ir_at: usize, pointer: isize) {
        let ri = self.map.get_mut(&ir_at).unwrap();
        ri.pointer = pointer;
        ri.positive = max(ri.positive, self.curr_positive);
        ri.negative = min(ri.negative, self.curr_negative);
    }
}

pub enum MemoryRange {
    None,
    Positive(u16), // deopt when ptr >= X
    Negative(u16), // deopt when ptr < X
    Both { positive: u16, negative: u16 },
}
pub struct RangeInfo {
    pub map: HashMap<usize, MemoryRange>,
    pub do_opt_first: bool,
}
impl RangeInfo {
    fn from(internal_ri: &InternalRangeState) -> Result<RangeInfo, String> {
        let map_arr: Result<Vec<(usize, MemoryRange)>, String> = internal_ri.map.iter().map(|(&ir_at, &RSMapElement { pointer, positive, negative })| {
            let posr_raw = 65536 - (positive - pointer);
            let negr_raw = -(negative - pointer);

            match (pointer == positive, pointer == negative) {
                (false, false) => {
                    let posr_val = u16::try_from(posr_raw).map_err(|_| "OptimizationError: Pointer Range Overflow")?;
                    let negr_val = u16::try_from(negr_raw).map_err(|_| "OptimizationError: Pointer Range Overflow")?;
                    Ok((ir_at, MemoryRange::Both { positive: posr_val, negative: negr_val }))
                }
                (false, true) => {
                    let posr_val = u16::try_from(posr_raw).map_err(|_| "OptimizationError: Pointer Range Overflow")?;
                    Ok((ir_at, MemoryRange::Positive(posr_val)))
                }
                (true, false) => {
                    let negr_val = u16::try_from(negr_raw).map_err(|_| "OptimizationError: Pointer Range Overflow")?;
                    Ok((ir_at, MemoryRange::Negative(negr_val)))
                }
                (true, true) => {
                    Ok((ir_at, MemoryRange::None))
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
    let mut internal_ri = InternalRangeState::new();

    for (i, IR { pointer, opcode }) in ir_nodes.iter().enumerate().rev() {
        internal_ri.subscribe(*pointer);
        match opcode {
            IROp::Shift(_step) => {
                internal_ri.insert(i, *pointer);
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
                    internal_ri.apply_loop(*end, *pointer);
                }
            }
            IROp::LoopEndWithOffset(_start, _offset) => {
                internal_ri.insert(i, *pointer);
            }
            _ => {}
        }
    }

    Ok(RangeInfo::from(&internal_ri)?)
}

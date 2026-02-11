use std::{cmp::{max, min}, collections::HashMap};

use crate::ir::{IR, IROp};

#[inline(always)]
pub fn positive_is_out_of_range(range: u16, pointer: usize) -> bool {
    pointer >= (range as usize)
}
#[inline(always)]
pub fn negative_is_out_of_range(range: u16, pointer: usize) -> bool {
    pointer < (range as usize)
}

#[derive(Debug, Clone, Copy)]
struct Range {
    positive: isize,
    negative: isize,
}
impl Range {
    pub fn subscribe(&mut self, pointer: isize) {
        self.positive = max(self.positive, pointer);
        self.negative = min(self.negative, pointer);
    }
    pub fn subscribe_from_range(&mut self, range: Range) {
        self.positive = max(self.positive, range.positive);
        self.negative = min(self.negative, range.negative);
    }
}
#[derive(Debug)]
struct RSMapElement {
    pointer: isize,
    range: Range,
}
#[derive(Debug)]
struct InternalRangeState {
    map: HashMap<usize, RSMapElement>,
    scope_stack: Vec<Range>,
    curr: Range,
}
impl InternalRangeState {
    pub fn new() -> InternalRangeState {
        InternalRangeState {
            map: HashMap::new(),
            scope_stack: vec![],
            curr: Range { positive: isize::MIN, negative: isize::MAX },
        }
    }
    pub fn subscribe(&mut self, pointer: isize) {
        self.curr.subscribe(pointer);
    }
    pub fn insert(&mut self, ir_at: usize, pointer: isize) {
        self.map.insert(ir_at, RSMapElement {
            pointer,
            range: self.curr,
        });
        self.curr = Range { positive: pointer, negative: pointer };
    }
    pub fn push_loopend(&mut self) {
        self.scope_stack.push(self.curr);
    }
    pub fn pop_loopstart(&mut self, ir_at: usize, pointer: isize) {
        let mut scope = self.scope_stack.pop().unwrap();
        scope.subscribe(pointer);
        self.map.insert(ir_at, RSMapElement {
            pointer,
            range: scope,
        });
    }
    pub fn apply_loop(&mut self, ir_at: usize, pointer: isize) {
        let ri = self.map.get_mut(&ir_at).unwrap();
        ri.pointer = pointer;
        ri.range.subscribe_from_range(self.curr);
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
        let map_arr: Result<Vec<(usize, MemoryRange)>, String> = internal_ri.map.iter().map(|(&ir_at, &RSMapElement { pointer, range: Range { positive, negative } })| {
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
            do_opt_first: !(internal_ri.curr.negative < 0) && !(internal_ri.curr.positive >= 65536),
        })
    }
}

pub fn generate_range_info(ir_nodes: &[IR]) -> Result<RangeInfo, String> {
    let mut internal_ri = InternalRangeState::new();

    for (i, IR { pointer, opcode }) in ir_nodes.iter().enumerate().rev() {
        if let IROp::LoopEndWithOffset(..) = opcode {
            internal_ri.push_loopend();
        }
        if let IROp::LoopEnd(..) = opcode {
            internal_ri.push_loopend();
        }
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
                internal_ri.pop_loopstart(i, *pointer);
                if let IROp::LoopEndWithOffset(..) = ir_nodes[*end].opcode {
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

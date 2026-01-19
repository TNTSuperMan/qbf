use std::collections::HashMap;

use crate::ir::{IR, IROp};

#[derive(Debug)]
pub struct RangeInfo {
    pub pointer: isize,
    pub kind: RangeInfoKind,
    pub val: isize,
}
impl RangeInfo {
    #[inline]
    pub fn positive(pointer: isize, r: (isize, isize)) -> RangeInfo {
        RangeInfo { pointer, val: r.1, kind: RangeInfoKind::Positive }
    }
    #[inline]
    pub fn negative(pointer: isize, r: (isize, isize)) -> RangeInfo {
        RangeInfo { pointer, val: r.0, kind: RangeInfoKind::Negative }
    }
}
#[derive(Debug)]
pub enum RangeInfoKind {
    Positive,
    Negative,
}

pub fn calculate_range_data(ir_nodes: &[IR]) -> (HashMap<usize, RangeInfo>, (isize, isize)) {
    let mut map: HashMap<usize, RangeInfo> = HashMap::new();
    let mut curr_range = (isize::MAX, isize::MIN);

    macro_rules! subscribe {
        ($p: expr) => {
            if curr_range.0 > $p {
                curr_range.0 = $p;
            }
            if curr_range.1 < $p {
                curr_range.1 = $p;
            }
        };
    }

    for (i, IR { pointer, opcode }) in ir_nodes.iter().enumerate().rev() {
        match opcode {
            IROp::LoopStart(end) => {
                if let Some(ri) = map.get_mut(end) {
                    match &ri.kind {
                        RangeInfoKind::Positive => {
                            if ri.val < curr_range.1 {
                                ri.val = curr_range.1;
                            }
                        }
                        RangeInfoKind::Negative => {
                            if ri.val > curr_range.0 {
                                ri.val = curr_range.0;
                            }
                        }
                    }
                }
            }
            IROp::LoopEndWithOffset(_start, offset) => {
                if 0 <= *offset {
                    map.insert(i, RangeInfo::positive(*pointer, curr_range));
                } else {
                    map.insert(i, RangeInfo::negative(*pointer, curr_range));
                }
                curr_range = (*pointer, *pointer);
            }
            IROp::Shift(step) => {
                if 0 <= *step {
                    map.insert(i, RangeInfo::positive(*pointer, curr_range));
                } else {
                    map.insert(i, RangeInfo::negative(*pointer, curr_range));
                }
                curr_range = (*pointer, *pointer);
            }
            IROp::MulAndSetZero(dests) => {
                for (ptr, _val) in dests {
                    subscribe!(*ptr);
                }
            }
            IROp::MovesAndSetZero(dests) => {
                for (ptr, _val) in dests {
                    subscribe!(*ptr);
                }
            }
            _ => {}
        }
        subscribe!(*pointer);
    }

    (map, curr_range)
}

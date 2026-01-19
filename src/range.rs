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
    map: HashMap<usize, (Sign, isize, isize)>,
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
        self.map.insert(ir_at, (sign, pointer, match sign {
            Sign::Positive => self.curr_positive,
            Sign::Negative => self.curr_negative,
        }));
        self.curr_positive = pointer;
        self.curr_negative = pointer;
    }
    pub fn apply_loop(&mut self, ir_at: usize) {
        let ri = self.map.get_mut(&ir_at).unwrap();
        match &ri.0 {
            Sign::Positive => {
                if ri.2 < self.curr_positive {
                    ri.2 = self.curr_positive;
                }
            }
            Sign::Negative => {
                if ri.2 > self.curr_negative {
                    ri.2 = self.curr_negative;
                }
            }
        }
    }
}

pub struct RangeInfo {
    pub map: HashMap<usize, (Sign, isize)>,
    pub do_opt_first: bool,
}
impl RangeInfo {
    fn from(internal_ri: &InternalRangeInfo) -> RangeInfo {
        RangeInfo {
            map: HashMap::from_iter(
                internal_ri.map.iter().map(|(&ir_at, &(sign, ptr, r))| (
                    ir_at, (sign, ptr - r)
                ))
            ),
            do_opt_first: !(internal_ri.curr_negative < 0) && !(internal_ri.curr_positive >= 65536),
        }
    }
}

pub fn generate_range_info(ir_nodes: &[IR]) -> RangeInfo {
    let mut internal_ri = InternalRangeInfo::new();

    for (i, IR { pointer, opcode }) in ir_nodes.iter().enumerate().rev() {
        match opcode {
            IROp::LoopStart(end) => {
                if let IROp::LoopEndWithOffset(_, _) = ir_nodes[*end].opcode {
                    internal_ri.apply_loop(*end);
                }
            }
            IROp::LoopEndWithOffset(_start, offset) => {
                internal_ri.insert(i, Sign::isize_to_sign(*offset), *pointer);
            }
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
            _ => {}
        }
        internal_ri.subscribe(*pointer);
    }

    RangeInfo::from(&internal_ri)
}

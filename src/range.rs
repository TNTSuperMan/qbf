use std::{collections::HashMap, num::TryFromIntError, ops::{Range, RangeFrom, RangeInclusive, RangeTo}};

use thiserror::Error;

use crate::{TAPE_LENGTH, ir::{IR, IROp}};

#[derive(Error, Debug)]
pub enum RangeError {
    #[error("Start range overflow")]
    StartOverflow(TryFromIntError, isize),
    
    #[error("End range overflow")]
    EndOverflow(TryFromIntError, isize),
}

pub fn extend_ri_pointer(range: &RangeInclusive<isize>, pointer: isize) -> RangeInclusive<isize> {
    return (*range.start()).min(pointer)..=(*range.end()).max(pointer);
}
pub fn extend_ri_range(range1: &RangeInclusive<isize>, range2: &RangeInclusive<isize>) -> RangeInclusive<isize> {
    return (*range1.start()).min(*range2.start())..=(*range1.end()).max(*range2.end());
}

#[derive(Debug)]
struct RSMapElement {
    pointer: isize,
    range: RangeInclusive<isize>,
}
#[derive(Debug)]
struct InternalRangeState {
    map: HashMap<usize, RSMapElement>,
    scope_stack: Vec<RangeInclusive<isize>>,
    curr: RangeInclusive<isize>,
}
impl InternalRangeState {
    pub fn new() -> InternalRangeState {
        InternalRangeState {
            map: HashMap::new(),
            scope_stack: vec![],
            curr: isize::MAX..=isize::MIN,
        }
    }
    pub fn subscribe(&mut self, range: &RangeInclusive<isize>) {
        self.curr = extend_ri_range(&self.curr, range);
    }
    pub fn insert(&mut self, ir_at: usize, pointer: isize) {
        self.map.insert(ir_at, RSMapElement {
            pointer,
            range: self.curr.clone(),
        });
        self.curr = pointer..=pointer;
    }
    pub fn push_loopend(&mut self) {
        self.scope_stack.push(self.curr.clone());
    }
    pub fn pop_loopstart(&mut self) {
        self.curr = extend_ri_range(&self.curr, &self.scope_stack.pop().unwrap());
    }
    pub fn apply_loop(&mut self, ir_at: usize, pointer: isize) {
        let ri = self.map.get_mut(&ir_at).unwrap();
        ri.pointer = pointer;
        ri.range = extend_ri_range(&ri.range, &self.curr);
    }
}

pub enum MidRange {
    None,
    Negative(RangeFrom<u16>), // deopt when ptr < X
    Positive(RangeTo<u16>), // deopt when ptr >= X
    Both(Range<u16>),
}
pub struct RangeInfo {
    pub map: HashMap<usize, MidRange>,
    pub do_opt_first: bool,
}
impl RangeInfo {
    fn from(internal_ri: &InternalRangeState) -> Result<RangeInfo, RangeError> {
        let map_arr: Result<Vec<(usize, MidRange)>, RangeError> = internal_ri.map.iter().map(|(&ir_at, &RSMapElement { pointer, range: ref range_raw })| {
            let range = (-(range_raw.start() - pointer))..((TAPE_LENGTH as isize) - (range_raw.end() - pointer));

            match (range.start == 0, range.end == (TAPE_LENGTH as isize)) {
                (false, false) => {
                    let start: u16 = range.start.try_into().map_err(|e| RangeError::StartOverflow(e, range.start))?;
                    let end: u16 = range.end.try_into().map_err(|e| RangeError::EndOverflow(e, range.end))?;
                    Ok((ir_at, MidRange::Both(start..end)))
                }
                (false, true) => {
                    let start: u16 = range.start.try_into().map_err(|e| RangeError::StartOverflow(e, range.start))?;
                    Ok((ir_at, MidRange::Negative(start..)))
                }
                (true, false) => {
                    let end: u16 = range.end.try_into().map_err(|e| RangeError::EndOverflow(e, range.end))?;
                    Ok((ir_at, MidRange::Positive(..end)))
                }
                (true, true) => {
                    Ok((ir_at, MidRange::None))
                }
            }
        }).collect();
        Ok(RangeInfo {
            map: HashMap::from_iter(map_arr?),
            do_opt_first: !(*internal_ri.curr.start() < 0) && !(*internal_ri.curr.end() >= (TAPE_LENGTH as isize)),
        })
    }
}

pub fn generate_range_info(ir_nodes: &[IR]) -> Result<RangeInfo, RangeError> {
    let mut internal_ri = InternalRangeState::new();

    for (i, op) in ir_nodes.iter().enumerate().rev() {
        let IR { pointer, opcode } = op;
        if let IROp::LoopEndWithOffset(..) = opcode {
            internal_ri.push_loopend();
        }
        if let IROp::LoopEnd(..) = opcode {
            internal_ri.push_loopend();
        }
        internal_ri.subscribe(&op.get_range());
        match opcode {
            IROp::Shift(_step) => {
                internal_ri.insert(i, *pointer);
            }
            IROp::LoopStart(end) => {
                internal_ri.pop_loopstart();
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

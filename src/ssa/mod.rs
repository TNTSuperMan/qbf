use std::{collections::HashMap, fmt::Debug};

pub mod parse;
pub mod inline;
pub mod to_ir;

#[derive(Clone)]
pub struct PointerSSAHistory(HashMap<isize, Vec<SSAOp>>);
impl PointerSSAHistory {
    pub fn new() -> PointerSSAHistory {
        PointerSSAHistory(HashMap::new())
    }
    pub fn get_op(&self, version: PointerVersion) -> Option<SSAOp> {
        match self.0.get(&version.ptr) {
            Some(history) => {
                match history.get(version.version) {
                    Some(op) => Some(*op),
                    None => None,
                }
            },
            None => None,
        }
    }
    pub fn get_history(&self, ptr: isize) -> Option<&Vec<SSAOp>> {
        match self.0.get(&ptr) {
            Some(history) => Some(history),
            None => None,
        }
    }
    pub fn get_history_mut(&mut self, ptr: isize) -> &mut Vec<SSAOp> {
        self.0.entry(ptr).or_insert_with(|| vec![SSAOp::raw(ptr)])
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, isize, Vec<SSAOp>> {
        self.0.iter()
    }
    pub fn insert(&mut self, ptr: isize, history: Vec<SSAOp>) {
        self.0.insert(ptr, history);
    }
}
impl Debug for PointerSSAHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct PointerVersion {
    ptr: isize,
    version: usize,
}

impl Debug for PointerVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("[{}]#{}", self.ptr, self.version))?;
        Ok(())
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub enum SSAOp {
    raw(isize),
    set_c(u8),
    set_p(PointerVersion),
    add_pc(PointerVersion, u8),
    add_pp(PointerVersion, PointerVersion),
    mul_pc(PointerVersion, u8),

    mul_add(PointerVersion, PointerVersion, u8),
}

impl Debug for SSAOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SSAOp::raw(ptr) => f.write_str(&format!("raw [{}]", ptr)),
            SSAOp::set_c(val) => f.write_str(&format!("{}", val)),
            SSAOp::set_p(version) => f.write_str(&format!("set_p {:?}", version)),
            SSAOp::add_pc(ptr, val) => f.write_str(&format!("add_pc {:?} + {}", ptr, val)),
            SSAOp::add_pp(to, dest) => f.write_str(&format!("add_pp {:?} + {:?}", to, dest)),
            SSAOp::mul_pc(dest, val) => f.write_str(&format!("mul_pc {:?} * {}", dest, val)),

            SSAOp::mul_add(from, dest, val) => f.write_str(&format!("mul_add {:?} + {:?} * {}", from, dest, val)),
        }?;
        Ok(())
    }
}

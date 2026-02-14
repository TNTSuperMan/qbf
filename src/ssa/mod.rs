use std::{collections::HashMap, fmt::Debug};

pub mod parse;
pub mod inline;
pub mod to_ir;
pub mod r#loop;

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
        self.0.entry(ptr).or_insert_with(|| vec![SSAOp::Value(SSAValue::Raw(ptr))])
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SSAValue {
    Const(u8),
    Version(PointerVersion),
    Raw(isize),
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SSAOp {
    Value(SSAValue),
    Add(SSAValue, SSAValue),
    Sub(SSAValue, SSAValue),
    Mul(SSAValue, SSAValue),
}

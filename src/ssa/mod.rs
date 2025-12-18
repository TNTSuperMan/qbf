use std::{collections::HashMap, fmt::Debug};

pub mod parse;

type PointerSSAHistory = HashMap<isize, Vec<PointerOperation>>;

#[derive(Clone, Copy)]
pub struct PointerVersion {
    ptr: isize,
    version: usize,
}

impl Debug for PointerVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("PointerVersion([{}]#{})", self.ptr, self.version))?;
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub enum PointerOperation {
    UntrackedValue(isize),
    AssignConstant(u8),
    AddConstant(u8),
    AssignFromPointer(PointerVersion),
    AddFromPointer(PointerVersion),
    AddMultipliedValue(u8, PointerVersion),
}

impl Debug for PointerOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PointerOperation::UntrackedValue(ptr) => f.write_str(&format!("UntrackedValue([{}])", ptr)),
            PointerOperation::AssignConstant(val) => f.write_str(&format!("AssignConstant({})", val)),
            PointerOperation::AddConstant(val) => f.write_str(&format!("AddConstant({})", val)),
            PointerOperation::AssignFromPointer(version) => f.write_str(&format!("AssignFromPointer({:?})", version)),
            PointerOperation::AddFromPointer(version) => f.write_str(&format!("AddFromPointer({:?})", version)),
            PointerOperation::AddMultipliedValue(val, version) => f.write_str(&format!("AddMultipliedValue({},{:?})", val, version)),
        }?;
        Ok(())
    }
}

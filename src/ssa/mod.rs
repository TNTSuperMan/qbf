use std::{collections::HashMap, fmt::Debug};

pub mod parse;
mod inline;

type PointerSSAHistory = HashMap<isize, Vec<PointerOperation>>;

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
pub enum PointerOperation {
    UntrackedValue(isize),
    AssignConstant(u8),
    AddConstant(PointerVersion, u8),
    AssignFromPointer(PointerVersion),
    AddFromPointer(PointerVersion, PointerVersion),
    AddMultipliedValue(PointerVersion, PointerVersion, u8),
}

impl Debug for PointerOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PointerOperation::UntrackedValue(ptr) => f.write_str(&format!("raw [{}]", ptr)),
            PointerOperation::AssignConstant(val) => f.write_str(&format!("const {}", val)),
            PointerOperation::AddConstant(ptr, val) => f.write_str(&format!("$ac {:?} + {}", ptr, val)),
            PointerOperation::AssignFromPointer(version) => f.write_str(&format!("$ld {:?}", version)),
            PointerOperation::AddFromPointer(to, dest) => f.write_str(&format!("$afp {:?} + {:?}", to, dest)),
            PointerOperation::AddMultipliedValue(to, dest, val) => f.write_str(&format!("$amv {:?} + {:?} * {}", to, dest, val)),
        }?;
        Ok(())
    }
}

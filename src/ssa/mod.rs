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
    raw(isize),
    set_c(u8),
    set_p(PointerVersion),
    add_pc(PointerVersion, u8),
    add_pp(PointerVersion, PointerVersion),
    mul_pc(PointerVersion, u8),
    mul_pp(PointerVersion, PointerVersion),

    mul_add(PointerVersion, PointerVersion, u8),
}

impl Debug for PointerOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PointerOperation::raw(ptr) => f.write_str(&format!("raw [{}]", ptr)),
            PointerOperation::set_c(val) => f.write_str(&format!("const {}", val)),
            PointerOperation::set_p(version) => f.write_str(&format!("{:?}", version)),
            PointerOperation::add_pc(ptr, val) => f.write_str(&format!("{:?} + {}", ptr, val)),
            PointerOperation::add_pp(to, dest) => f.write_str(&format!("{:?} + {:?}", to, dest)),
            PointerOperation::mul_pc(dest, val) => f.write_str(&format!("{:?} * {}", dest, val)),
            PointerOperation::mul_pp(dest, val) => f.write_str(&format!("{:?} * {:?}", dest, val)),

            PointerOperation::mul_add(from, dest, val) => f.write_str(&format!("{:?} + {:?} * {}", from, dest, val)),
        }?;
        Ok(())
    }
}

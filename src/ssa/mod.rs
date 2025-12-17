use std::{collections::HashMap, fmt::Debug};

pub mod parse;

type SSAData = HashMap<isize, Vec<SSAVariant>>;

#[derive(Clone, Copy)]
pub struct SSA {
    ptr: isize,
    index: usize,
}

impl Debug for SSA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("SSA([{}]#{})", self.ptr, self.index))?;
        Ok(())
    }
}

pub enum SSAVariant {
    Raw(isize),
    SetConst(u8),
    AddConst(u8),
    SetFrom(SSA),
    AddFrom(SSA),
    AddMul(u8, SSA),
}

impl Debug for SSAVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SSAVariant::Raw(ptr) => f.write_str(&format!("Raw([{}])", ptr)),
            SSAVariant::SetConst(val) => f.write_str(&format!("SetConst({})", val)),
            SSAVariant::AddConst(val) => f.write_str(&format!("AddConst({})", val)),
            SSAVariant::SetFrom(ssa) => f.write_str(&format!("SetFrom({:?})", ssa)),
            SSAVariant::AddFrom(ssa) => f.write_str(&format!("AddFrom({:?})", ssa)),
            SSAVariant::AddMul(val, ssa) => f.write_str(&format!("AddMul({},{:?})", val, ssa)),
        }?;
        Ok(())
    }
}

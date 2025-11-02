use crate::vm::BFVM;

pub enum TargetPointer {
    Current,
    Absolute(usize),
    Relative(isize),
}

impl TargetPointer {
    pub fn use_point(&self, vm: &mut BFVM) -> usize {
        match self {
            TargetPointer::Current => (),
            TargetPointer::Absolute(p) => vm.pointer = *p,
            TargetPointer::Relative(p) => vm.pointer = ((vm.pointer as isize) + p) as usize,
        };
        vm.pointer
    }
}

pub struct LoopOptimizationInfo {
    pub is_flat: Option<bool>,
    pub pointer_assumption: Option<usize>,
}
impl LoopOptimizationInfo {
    pub fn new() -> LoopOptimizationInfo {
        LoopOptimizationInfo {
            is_flat: None,
            pointer_assumption: None,
        }
    }
}

pub enum Instruction {
    Add (TargetPointer, u8),
    Set (TargetPointer, u8),

    MulAndSetZero(
        TargetPointer,
        Vec<(TargetPointer, u8)>,
    ),

    To(TargetPointer),

    In(TargetPointer),
    Out(TargetPointer),

    LoopStart(usize),
    LoopEnd(usize, LoopOptimizationInfo),
}

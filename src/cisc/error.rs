use std::{io, num::TryFromIntError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum OptimizationError {
    #[error("OptimizationError: Pointer delta overflow")]
    Delta(TryFromIntError),

    #[error("OptimizationError: Shift step overflow")]
    ShiftStep(TryFromIntError),

    #[error("OptimizationError: Program address overflow")]
    ProgramAbs(TryFromIntError),

    #[error("OptimizationError: Program relative address overflow")]
    ProgramRel(TryFromIntError),
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("RuntimeError: Out of bound memory get, address: {0}")]
    OOBGet(usize),
    
    #[error("RuntimeError: Out of bound memory set, address: {0}")]
    OOBSet(usize),
    
    #[error("RuntimeError: Out of bound memory add, address: {0}")]
    OOBAdd(usize),
    
    #[error("RuntimeError: Out of bound memory sub, address: {0}")]
    OOBSub(usize),

    #[error("RuntimeError: {0}")]
    IOError(#[from] io::Error)
}

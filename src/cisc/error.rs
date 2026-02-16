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
    #[error("RuntimeError: Out of bounds while getting cell {0}")]
    OOBGet(usize),
    
    #[error("RuntimeError: Out of bounds while setting {1} to cell {0}")]
    OOBSet(usize, u8),
    
    #[error("RuntimeError: Out of bounds while adding {1} to cell {0}")]
    OOBAdd(usize, u8),
    
    #[error("RuntimeError: Out of bounds while subtracting {1} to cell {0}")]
    OOBSub(usize, u8),

    #[error("RuntimeError: {0}")]
    IOError(#[from] io::Error)
}

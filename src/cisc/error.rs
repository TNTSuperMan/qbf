use std::{io, num::TryFromIntError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum OptimizationError {
    #[error("Pointer delta overflow")]
    Delta(#[source] TryFromIntError),

    #[error("Shift step overflow")]
    ShiftStep(#[source] TryFromIntError),

    #[error("Program address overflow")]
    ProgramAbs(#[source] TryFromIntError),

    #[error("Program relative address overflow")]
    ProgramRel(#[source] TryFromIntError),
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Out of bounds while getting cell {0}")]
    OOBGet(usize),
    
    #[error("Out of bounds while setting {1} to cell {0}")]
    OOBSet(usize, u8),
    
    #[error("Out of bounds while adding {1} to cell {0}")]
    OOBAdd(usize, u8),
    
    #[error("Out of bounds while subtracting {1} to cell {0}")]
    OOBSub(usize, u8),

    #[error("{0}")]
    IOError(#[from] io::Error)
}

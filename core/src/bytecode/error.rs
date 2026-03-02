use std::num::TryFromIntError;

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

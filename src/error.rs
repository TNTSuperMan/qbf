use thiserror::Error;

use crate::cisc::error::{OptimizationError, RuntimeError};

#[derive(Error, Debug)]
pub enum SyntaxError {
    #[error("Unmatched opening bracket")]
    UnmatchedOpeningBracket,
    #[error("Unmatched closing bracket")]
    UnmatchedClosingBracket,
}

#[derive(Error, Debug)]
pub enum BrainrotError {
    #[error("SyntaxError: {0}")]
    SyntaxError(#[from] SyntaxError),

    #[error("OptimizationError: {0}")]
    OptimizationError(#[from] OptimizationError),

    #[error("RuntimeError: {err}")]
    RuntimeError{
        err: RuntimeError,
        pc: usize,
        pointer: usize,
    },
}

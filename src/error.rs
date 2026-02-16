use std::io;

use thiserror::Error;

use crate::{cisc::error::{OptimizationError, RuntimeError}, range::RangeError};

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

    #[error("RangeError: {0}")]
    RangeError(#[from] RangeError),

    #[error("IOError: {0}")]
    IOError(#[from] io::Error),
}

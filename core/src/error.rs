use std::io;

use thiserror::Error;

use crate::{bytecode::error::OptimizationError, ir::error::{SyntaxError, RangeError}};

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
    IOError(#[from] io::Error),

    #[error("Timeouted")]
    TimeoutError,
}

#[derive(Error, Debug)]
pub enum BrainrotError {
    #[error("SyntaxError: {0}")]
    SyntaxError(#[from] SyntaxError),

    #[error("OptimizationError: {0}")]
    OptimizationError(#[from] OptimizationError),

    #[error("RuntimeError: {err}")]
    RuntimeError{
        #[source]
        err: RuntimeError,
        pc: usize,
        pointer: usize,
    },

    #[error("RangeError: {0}")]
    RangeError(#[from] RangeError),

    #[error("IOError: {0}")]
    IOError(#[from] io::Error),

    #[error("FeatureError: {0}")]
    FetureError(String),
}

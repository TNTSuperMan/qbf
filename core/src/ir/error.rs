use std::num::TryFromIntError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyntaxError {
    #[error("Unmatched opening bracket")]
    UnmatchedOpeningBracket,
    #[error("Unmatched closing bracket")]
    UnmatchedClosingBracket,
}

#[derive(Error, Debug)]
pub enum RangeError {
    #[error("Start range overflow")]
    StartOverflow(TryFromIntError, isize),
    
    #[error("End range overflow")]
    EndOverflow(TryFromIntError, isize),
}

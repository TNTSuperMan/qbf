use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyntaxError {
    #[error("SyntaxError: Unmatched opening bracket")]
    UnmatchedOpeningBracket,
    #[error("SyntaxError: Unmatched closing bracket")]
    UnmatchedClosingBracket,
}

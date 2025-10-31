pub enum Token {
    Add(u8),
    Left(usize),
    Right(usize),
    In,
    Out,
    Loop(Vec<Token>),
}

pub fn tokenize(code: &str) -> Vec<Token> {
    Vec::new()
}

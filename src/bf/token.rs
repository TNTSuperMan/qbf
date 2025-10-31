pub enum Token {
    Add(u8),
    RelativeTo(isize),
    In,
    Out,
    Loop(Vec<Token>),
}

pub fn tokenize(code: &str) -> Vec<Token> {
    Vec::new()
}

use std::fmt::Display;

pub enum Token {
    Add(u8),
    RelativeTo(isize),
    In,
    Out,
    LoopStart(usize),
    LoopEnd(usize),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Add(count) => f.write_str(&format!("Add({})", count)),
            Token::RelativeTo(count) => f.write_str(&format!("RelativeTo({})", count)),
            Token::In => f.write_str("In"),
            Token::Out => f.write_str("Out"),
            Token::LoopStart(end) => f.write_str(&format!("LoopStart({})", end)),
            Token::LoopEnd(start) => f.write_str(&format!("LoopEnd({})", start)),
        }
    }
}

pub fn tokenize(code: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut loop_stack: Vec<usize> = Vec::new();

    for char in code.chars() {
        match char {
            '+' => {
                if let Some(Token::Add(count)) = tokens.last_mut() {
                    *count = count.wrapping_add(1);
                } else {
                    tokens.push(Token::Add(1));
                }
            }
            '-' => {
                if let Some(Token::Add(count)) = tokens.last_mut() {
                    *count = count.wrapping_sub(1);
                } else {
                    tokens.push(Token::Add(255)); // 255u8 = -1i8
                }
            }
            '>' => {
                if let Some(Token::RelativeTo(to)) = tokens.last_mut() {
                    *to += 1;
                } else {
                    tokens.push(Token::RelativeTo(1));
                }
            }
            '<' => {
                if let Some(Token::RelativeTo(to)) = tokens.last_mut() {
                    *to -= 1;
                } else {
                    tokens.push(Token::RelativeTo(-1));
                }
            }
            '.' => {
                tokens.push(Token::Out);
            }
            ',' => {
                tokens.push(Token::In);
            }
            '[' => {
                loop_stack.push(tokens.len()); // ループ先頭のASTポインタになるよ
                tokens.push(Token::LoopStart(usize::MAX));
            }
            ']' => {
                let start = loop_stack.pop().unwrap();
                let end = tokens.len(); // 上のコメントと同じ感じ
                tokens.push(Token::LoopEnd(start));
                tokens[start] = Token::LoopStart(end);
            }
            _ => {}
        }
    }

    tokens
}

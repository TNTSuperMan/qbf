pub enum Token {
    Add(u8),
    RelativeTo(isize),
    In,
    Out,
    Loop(Vec<Token>),
}

pub fn tokenize(code: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();

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
            
            _ => {}
        }
    }

    tokens
}

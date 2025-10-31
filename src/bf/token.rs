pub enum Token {
    Add(u8),
    RelativeTo(isize),
    In,
    Out,
    LoopStart(usize),
    LoopEnd(usize),
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
                let end = loop_stack.len(); // 上のコメントと同じ感じ
                tokens.push(Token::LoopEnd(start));
                tokens[start] = Token::LoopStart(end);
            }
            _ => {}
        }
    }

    tokens
}

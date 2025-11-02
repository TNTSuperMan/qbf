use crate::inst::{Instruction, TargetPointer};

pub fn parse(code: &str) -> Vec<Instruction> {
    let mut tokens: Vec<Instruction> = Vec::new();
    let mut loop_stack: Vec<usize> = Vec::new();

    for char in code.chars() {
        match char {
            '+' => {
                if let Some(Instruction::Add(_, count)) = tokens.last_mut() {
                    *count = count.wrapping_add(1);
                } else {
                    tokens.push(Instruction::Add(
                        TargetPointer::Current,
                        1
                    ));
                }
            }
            '-' => {
                if let Some(Instruction::Add(_, count)) = tokens.last_mut() {
                    *count = count.wrapping_sub(1);
                } else {
                    tokens.push(Instruction::Add(
                        TargetPointer::Current,
                        255 // 255u8 = -1i8
                    ));
                }
            }
            '>' => {
                if let Some(Instruction::To(TargetPointer::Relative(to))) = tokens.last_mut() {
                    *to += 1;
                } else {
                    tokens.push(Instruction::To(TargetPointer::Relative(1)));
                }
            }
            '<' => {
                if let Some(Instruction::To(TargetPointer::Relative(to))) = tokens.last_mut() {
                    *to -= 1;
                } else {
                    tokens.push(Instruction::To(TargetPointer::Relative(-1)));
                }
            }
            '.' => {
                tokens.push(Instruction::Out(TargetPointer::Current));
            }
            ',' => {
                tokens.push(Instruction::In(TargetPointer::Current));
            }
            '[' => {
                loop_stack.push(tokens.len()); // ループ先頭のASTポインタになるよ
                tokens.push(Instruction::LoopStart(usize::MAX));
            }
            ']' => {
                let start = loop_stack.pop().unwrap();
                let end = tokens.len(); // 上のコメントと同じ感じ
                tokens.push(Instruction::LoopEnd(start));
                tokens[start] = Instruction::LoopStart(end);
            }
            _ => {}
        }
    }

    tokens
}

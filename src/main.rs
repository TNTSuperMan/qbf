use std::fs;

use crate::{instruction::ast_to_instructions, interpret::run, parser::parse, trace::instructions_to_string, vm::BFVM};

mod instruction;
mod interpret;
mod parser;
mod trace;
mod vm;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() == 2 {
        match std::fs::read_to_string(args[1].clone()) {
            Err(e) => {
                eprintln!("Error: {}", e);
            }
            Ok(code) => {
                let ast = parse(&code);
                let (instructions, hints) = ast_to_instructions(ast);
                fs::write("./box/instructions", instructions_to_string(instructions.clone())).expect("failed to write");
                let mut vm = BFVM {
                    pc: 0,
                    pointer: 0,
                    memory: vec![0u8; 30000],
                    input: Box::new(|| { 0 }),
                    output: Box::new(|val| { print!("{}", val as char)}),
                };
                run(&mut vm, instructions, hints);
                println!();
            }
        }
    } else {
        println!("usage: {} <file>", args[0]);
    }
}

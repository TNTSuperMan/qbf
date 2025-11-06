use std::fs;

use crate::{interpret::run, parser::parse, trace::instructions_to_string, vm::BFVM};

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
                let insts = parse(&code);
                fs::write("./box/instructions", instructions_to_string(insts.clone())).expect("failed to write");
                let mut vm = BFVM {
                    pc: 0,
                    memory: vec![0u8; 30000],
                    input: Box::new(|| { 0 }),
                    output: Box::new(|val| { print!("{}", val as char)}),
                };
                run(&mut vm, insts);
                println!();
            }
        }
    } else {
        println!("usage: {} <file>", args[0]);
    }
}

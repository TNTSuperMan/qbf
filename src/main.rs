use crate::{interpret::run, parser::parse, vm::BFVM};

mod inst;
mod interpret;
mod parser;
mod vm;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() == 2 {
        match std::fs::read_to_string(args[1].clone()) {
            Err(e) => {
                eprintln!("Error: {}", e);
            }
            Ok(code) => {
                let instrs = parse(&code);
                let mut vm = BFVM {
                    pc: 0,
                    pointer: 0,
                    memory: vec![0u8; 30000],
                    input: Box::new(|| { 0 }),
                    output: Box::new(|val| { print!("{}", val as char)}),
                };
                run(&mut vm, instrs);
                println!();
            }
        }
    } else {
        println!("usage: {} <file>", args[0]);
    }
}

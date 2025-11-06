use std::fs;

use crate::{trace::instructions_to_string, vm::BFVM};

mod interpret;
mod io;
mod jit;
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
                let mut vm = BFVM::new(
                    &code,
                    65536,
                );
                fs::write("./box/instructions", instructions_to_string(vm.insts.clone())).expect("failed to write");
                vm.run();
            }
        }
    } else {
        println!("usage: {} <file>", args[0]);
    }
}

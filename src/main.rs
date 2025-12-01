use crate::{bytecode::ir_to_bytecodes, interpret::run, ir::parse_to_ir, trace::OperationCountMap};
use clap::Parser;

mod interpret;
mod ir;
mod bytecode;
mod trace;

#[derive(Parser, Debug)]
#[command(name = "qbf")]
struct Args {
    #[arg(short, long, default_value_t = 65536)]
    memory_size: usize,
    
    #[arg(value_name = "FILE")]
    file: String,
}

fn main() {
    let args = Args::parse();
    
    match std::fs::read_to_string(args.file) {
        Err(e) => {
            eprintln!("Error: {}", e);
        }
        Ok(code) => {
            let ir = parse_to_ir(&code);
            let bytecodes = ir_to_bytecodes(ir);
            let mut v = OperationCountMap::new(bytecodes.len());
            let result = run(bytecodes.clone(), args.memory_size, &mut v);
            if let Err(err) = result.clone() {
                eprintln!("{}", err);
            }
            #[cfg(feature = "debug")] {
                use crate::trace::instructions_to_string;
                use std::fs;
                if let Ok(mem) = result {
                    fs::write("./box/memory", mem).expect("failed to write");
                }
                fs::write("./box/bytecodes", instructions_to_string(bytecodes, v)).expect("failed to write");
            }
        }
    }
}

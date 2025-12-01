use crate::{bytecode::ir_to_bytecodes, interpret::run, ir::parse_to_ir, trace::OperationCountMap};

mod interpret;
mod ir;
mod bytecode;
mod trace;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() == 2 {
        match std::fs::read_to_string(args[1].clone()) {
            Err(e) => {
                eprintln!("Error: {}", e);
            }
            Ok(code) => {
                let ir = parse_to_ir(&code);
                let bytecodes = ir_to_bytecodes(ir);
                let mut v = OperationCountMap::new(bytecodes.len());
                if let Err(err) = run(bytecodes.clone(), 65536, &mut v) {
                    eprintln!("{}", err);
                }
                #[cfg(feature = "debug")] {
                    use crate::trace::instructions_to_string;
                    use std::fs;
                    fs::write("./box/bytecodes", instructions_to_string(bytecodes, v)).expect("failed to write");
                }
            }
        }
    } else {
        println!("usage: {} <file>", args[0]);
    }
}

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
                let result = run(bytecodes.clone(), 65536, &mut v);
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
    } else {
        println!("usage: {} <file>", args[0]);
    }
}

use crate::{interpret::run, parser::parse, trace::OperationCountMap};

mod interpret;
mod parser;
mod trace;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() == 2 {
        match std::fs::read_to_string(args[1].clone()) {
            Err(e) => {
                eprintln!("Error: {}", e);
            }
            Ok(code) => {
                let insts = parse(&code);
                let mut v = OperationCountMap::new(insts.len());
                run(insts.clone(), 65536, &mut v);
                #[cfg(feature = "debug")] {
                    use crate::trace::instructions_to_string;
                    use std::fs;
                    fs::write("./box/instructions", instructions_to_string(insts, v)).expect("failed to write");
                }
            }
        }
    } else {
        println!("usage: {} <file>", args[0]);
    }
}

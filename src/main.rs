use std::time::Instant;

use crate::{bytecode::ir_to_bytecodes, interpret::run, ir::parse_to_ir, trace::OperationCountMap};
use clap::Parser;

mod interpret;
mod ir;
mod bytecode;
mod trace;

#[derive(Parser, Debug)]
#[command(name = "qbf")]
struct Args {
    #[arg(value_name = "FILE")]
    file: String,

    #[arg(short, long, default_value_t = 65536)]
    memory_size: usize,
    
    #[arg(short, long)]
    benchmark_count: Option<usize>,
}

fn main() {
    let args = Args::parse();
    
    match std::fs::read_to_string(args.file) {
        Err(e) => {
            eprintln!("Error: {}", e);
        }
        Ok(code) => {
            if let Some(count) = args.benchmark_count {
                let mut times: Vec<f64> = vec![];
                for _ in 0..count {
                    let start = Instant::now();

                    let ir = parse_to_ir(&code);
                    let bytecodes = ir_to_bytecodes(ir);
                    let mut v = OperationCountMap::new(bytecodes.len());
                    let result = run(bytecodes.clone(), args.memory_size, &mut v);
                    if let Err(err) = result.clone() {
                        eprintln!("{}", err);
                        return;
                    }

                    times.push(start.elapsed().as_secs_f64());
                }

                let mean = times.iter().sum::<f64>() / times.len() as f64;
                println!("Mean time(sec): {}", mean);
            } else {
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
}

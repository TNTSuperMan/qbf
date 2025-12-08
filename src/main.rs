use std::time::Instant;

use crate::{bytecode::ir_to_bytecodes, interpret::run, ir::parse_to_ir, memory::StaticMemory, trace::OperationCountMap};
use clap::Parser;

mod memory;
mod ir;
mod bytecode;
mod interpret;
mod trace;

#[derive(Parser, Debug)]
#[command(name = "qbf")]
struct Args {
    #[arg(value_name = "FILE")]
    file: String,

    // #[arg(short, long, default_value_t = 65536)]
    // memory_size: usize,
    
    #[arg(short, long)]
    benchmark_count: Option<usize>,
}

fn main() {
    let args = Args::parse();
    
    match std::fs::read_to_string(args.file) {
        Err(e) => {
            eprintln!("File Error: {}", e);
        }
        Ok(code) => {
            if let Some(count) = args.benchmark_count {
                match parse_to_ir(&code) {
                    Ok(_ir) => {},
                    Err(msg) => {
                        eprintln!("{}", msg);
                        eprintln!("Run without --benchmark-count for more details");
                        return;
                    }
                };
                let mut times: Vec<f64> = vec![];
                for _ in 0..count {
                    let start = Instant::now();

                    let ir = parse_to_ir(&code).unwrap(); // SAFETY: 最初のparse_to_irで事前に検証済みのため安全
                    let bytecodes = ir_to_bytecodes(ir);

                    let mut ocm = OperationCountMap::new(bytecodes.len());
                    let mut memory = StaticMemory::new();
                    
                    let result = run(bytecodes.clone(), &mut memory, &mut ocm);
                    if let Err(err) = result.clone() {
                        eprintln!("{}", err);
                        return;
                    }

                    times.push(start.elapsed().as_secs_f64());
                }

                let mean = times.iter().sum::<f64>() / times.len() as f64;
                println!("Mean time(sec): {}", mean);
            } else {
                let ir = match parse_to_ir(&code) {
                    Ok(ir) => ir,
                    Err(msg) => {
                        // TODO: 詳細にエラーを出す仕組みにする
                        eprintln!("{}", msg);
                        return;
                    }
                };
                let bytecodes = ir_to_bytecodes(ir);

                let mut ocm = OperationCountMap::new(bytecodes.len());
                let mut memory = StaticMemory::new();

                let result = run(bytecodes.clone(), &mut memory, &mut ocm);
                if let Err(err) = result.clone() {
                    eprintln!("{}", err);
                }

                #[cfg(feature = "debug")] {
                    use crate::trace::instructions_to_string;
                    use std::fs;
                    fs::write("./box/memory", *memory.0).expect("failed to write");
                    fs::write("./box/bytecodes", instructions_to_string(bytecodes, ocm)).expect("failed to write");
                }
            }
        }
    }
}

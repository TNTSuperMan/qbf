use std::time::Instant;

use crate::{bytecode::ir_to_bytecodes, bytecode2::ir_to_bytecodes2, interpret::run, interpret2::run2, ir::parse_to_ir, memory::Memory, trace::OperationCountMap};
use clap::Parser;

mod memory;
mod ir;
mod bytecode;
mod bytecode2;
mod interpret;
mod interpret2;
mod trace;
mod ssa;

#[derive(Parser, Debug)]
#[command(name = "qbf")]
struct Args {
    #[arg(value_name = "FILE")]
    file: String,
    
    #[arg(short, long)]
    benchmark_count: Option<usize>,

    #[arg(short, long)]
    use_next_bytecode: bool,
}

fn main() {
    let args = Args::parse();
    
    match std::fs::read_to_string(args.file) {
        Err(e) => {
            eprintln!("File Error: {}", e);
        }
        Ok(code) => {
            if let Some(count) = args.benchmark_count {
                if args.use_next_bytecode {
                    eprintln!("Error: --use-next-bytecode not implemented with --benchmark-count")
                }
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
                    let bytecodes = ir_to_bytecodes(&ir);

                    let mut ocm = OperationCountMap::new(bytecodes.len());
                    let mut memory = Memory::new();
                    
                    let result = run(&bytecodes, &mut memory, &mut ocm);
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

                let mut memory = Memory::new();

                if args.use_next_bytecode {
                    let bytecodes = match ir_to_bytecodes2(&ir) {
                        Ok(b) => b,
                        Err(msg) => {
                            eprintln!("{}", msg);
                            return;
                        }
                    };
                    if let Err(msg) = run2(&bytecodes, &mut memory) {
                        eprintln!("{}", msg);
                    }
                    #[cfg(feature = "debug")] {
                        use std::fs;
                        fs::write("./box/bytecodes", format!("{:?}", bytecodes)).expect("failed to write");
                    }
                } else {
                    let bytecodes = ir_to_bytecodes(&ir);
                    let mut ocm = OperationCountMap::new(bytecodes.len());
                    if let Err(msg) = run(&bytecodes, &mut memory, &mut ocm) {
                        eprintln!("{}", msg);
                    }
                    #[cfg(feature = "debug")] {
                        use std::fs;
                        use crate::trace::instructions_to_string;
                        fs::write("./box/bytecodes", instructions_to_string(bytecodes, ocm)).expect("failed to write");
                    }
                }

                #[cfg(feature = "debug")] {
                    use crate::{ssa::{PointerSSAHistory, inline::inline_ssa_history, parse::build_ssa_from_ir, to_ir::resolve_eval_order}};
                    use std::fs;
                    fs::write("./box/memory", *memory.0).expect("failed to write");
                    let noend_ir = &ir[0..ir.len()-1];
                    let raw = build_ssa_from_ir(&noend_ir).unwrap_or_else(|| PointerSSAHistory::new());
                    let one_round = inline_ssa_history(&raw);
                    let two_round = inline_ssa_history(&one_round);
                    fs::write("./box/ssa", format!("{:?}", raw)).expect("failed to write");
                    fs::write("./box/ssa_opt1", format!("{:?}", one_round)).expect("failed to write");
                    fs::write("./box/ssa_opt2", format!("{:?}", two_round)).expect("failed to write");
                    fs::write("./box/eval_order", format!("{:?}", resolve_eval_order(&two_round))).expect("failed to write");

                }
            }
        }
    }
}

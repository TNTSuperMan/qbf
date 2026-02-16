use std::fs;

use crate::{cisc::run_cisc, ir::parse_to_ir, range::generate_range_info};
use clap::Parser;

mod error;
mod ir;
mod cisc;
mod trace;
mod range;

#[derive(Parser, Debug)]
#[command(name = "brainrot")]
struct Args {
    #[arg(value_name = "FILE")]
    file: String,

    #[arg(short, long)]
    flush: bool,

    #[arg(short, long)]
    out_dump: bool,
}

fn main() {
    let args = Args::parse();
    
    match fs::read_to_string(args.file) {
        Err(e) => {
            eprintln!("File Error: {}", e);
        }
        Ok(code) => {
            let ir = match parse_to_ir(&code) {
                Ok(ir) => ir,
                Err(msg) => {
                    // TODO: 詳細にエラーを出す仕組みにする
                    eprintln!("{}", msg);
                    return;
                }
            };
            let range_info = match generate_range_info(&ir) {
                Ok(ri) => ri,
                Err(msg) => {
                    // TODO: 詳細にエラーを出す仕組みにする
                    eprintln!("{}", msg);
                    return;
                }
            };
            if cfg!(feature = "debug") && args.out_dump {
                fs::write("./box/ir", crate::trace::generate_ir_trace(&ir, &range_info)).expect("failed to write");
            }

            if let Err(msg) = run_cisc(&ir, &range_info, args.flush, args.out_dump) {
                eprintln!("{}", msg);
            }

            if cfg!(feature = "debug") {
                // use crate::ssa::{PointerSSAHistory, inline::inline_ssa_history, parse::build_ssa_from_ir, to_ir::resolve_eval_order};
                /* let noend_ir = &ir[0..ir.len()-1];
                let raw = build_ssa_from_ir(&noend_ir).unwrap_or_else(|| PointerSSAHistory::new());
                let one_round = inline_ssa_history(&raw);
                let two_round = inline_ssa_history(&one_round);
                fs::write("./box/ssa", format!("{:?}", raw)).expect("failed to write");
                fs::write("./box/ssa_opt1", format!("{:?}", one_round)).expect("failed to write");
                fs::write("./box/ssa_opt2", format!("{:?}", two_round)).expect("failed to write");
                fs::write("./box/eval_order", format!("{:?}", resolve_eval_order(&two_round))).expect("failed to write"); */
            }
        }
    }
}

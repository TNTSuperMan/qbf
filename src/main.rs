use std::fs;

use crate::{cisc::run_cisc, ir::parse_to_ir, range::generate_range_info};
use anyhow::Result;
use clap::Parser;

mod memory;
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

fn main() -> Result<()> {
    let args = Args::parse();

    let code = fs::read_to_string(args.file)?;
    
    let ir = parse_to_ir(&code)?;
    let range_info = generate_range_info(&ir)?;
    if cfg!(feature = "debug") && args.out_dump {
        fs::write("./box/ir", crate::trace::generate_ir_trace(&ir, &range_info))?;
    }

    run_cisc(&ir, &range_info, args.flush, args.out_dump)?;

    if cfg!(feature = "debug") {
        // use crate::ssa::{PointerSSAHistory, inline::inline_ssa_history, parse::build_ssa_from_ir, to_ir::resolve_eval_order};
        /* let noend_ir = &ir[0..ir.len()-1];
        let raw = build_ssa_from_ir(&noend_ir).unwrap_or_else(|| PointerSSAHistory::new());
        let one_round = inline_ssa_history(&raw);
        let two_round = inline_ssa_history(&one_round);
        fs::write("./box/ssa", format!("{:?}", raw))?;
        fs::write("./box/ssa_opt1", format!("{:?}", one_round))?;
        fs::write("./box/ssa_opt2", format!("{:?}", two_round))?;
        fs::write("./box/eval_order", format!("{:?}", resolve_eval_order(&two_round)))?; */
    }

    Ok(())
}

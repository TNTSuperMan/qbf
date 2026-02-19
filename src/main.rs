use std::{fs, process::ExitCode};

const TAPE_LENGTH: usize = 65536;

use crate::{cisc::run_cisc, error::BrainrotError, ir::parse_to_ir, range::generate_range_info};
use clap::Parser;

mod error;
mod ir;
mod ssa;
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

fn resulty_main(args: Args) -> Result<(), BrainrotError> {
    let code = fs::read_to_string(args.file)?;
    let ir = parse_to_ir(&code)?;
    let range_info = generate_range_info(&ir)?;
    
    if cfg!(feature = "debug") && args.out_dump {
        fs::write("./box/ir", crate::trace::generate_ir_trace(&ir, &range_info))?;
    }

    Ok(run_cisc(&ir, &range_info, args.flush, args.out_dump)?)
}

fn main() -> ExitCode {
    let args = Args::parse();
    
    match resulty_main(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            if cfg!(feature = "debug") {
                eprintln!("{err:?}");
            } else {
                eprintln!("{err}");
            }
            ExitCode::FAILURE
        }
    }
}

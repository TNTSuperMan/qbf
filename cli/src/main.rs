use core::{Brainrot, BrainrotInit, error::{BrainrotError, RuntimeError}};
use std::{fs, io::{Read, Write, stdin, stdout}, process::ExitCode};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "brainrot")]
struct Args {
    #[arg(value_name = "FILE")]
    file: String,

    #[arg(short, long)]
    flush: bool,

    #[arg(short, long)]
    dump: Option<String>,
}

fn resulty_main(args: Args) -> Result<(), BrainrotError> {
    let code = fs::read_to_string(args.file)?;
    
    let mut stdin = stdin().lock();
    let mut stdout = stdout().lock();
    let mut stdin_buf = [0u8; 1];

    let mut vm = Brainrot::new(&code, BrainrotInit {
        input: || {
            match stdin.read_exact(&mut stdin_buf) {
                Ok(_) => stdin_buf[0],
                Err(_) => 0,
            }
        },
        output: |v| {
            let _ = stdout.write_all(&[v]);
            if args.flush {
                let _ = stdout.flush();
            }
        },
        io_break: false,
        timeout_step: None,
    })?;
    vm.step()?;

    if let Some(dump) = args.dump {
        fs::write(&dump, vm.generate_trace())?;
    }

    Ok(())
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
            match err {
                BrainrotError::RuntimeError { err: RuntimeError::TimeoutError, .. } => ExitCode::from(3),
                | BrainrotError::RuntimeError { err: RuntimeError::OOBGet(..), .. }
                | BrainrotError::RuntimeError { err: RuntimeError::OOBSet(..), .. }
                | BrainrotError::RuntimeError { err: RuntimeError::OOBAdd(..), .. }
                | BrainrotError::RuntimeError { err: RuntimeError::OOBSub(..), .. } => ExitCode::from(2),
                _ => ExitCode::FAILURE,
            }
        }
    }
}

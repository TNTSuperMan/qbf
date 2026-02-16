use crate::{cisc::{bytecode::ir_to_bytecodes, internal::{InterpreterResult, Tier}, interpret_deopt::run_deopt, interpret_opt::run_opt, tape::{Tape, UnsafeTape}, trace::write_trace, program::{Program, UnsafeProgram}}, error::BrainrotError, ir::IR, range::RangeInfo};

mod tape;
pub mod error;
mod bytecode;
mod interpret_deopt;
mod interpret_opt;
mod trace;
mod program;
mod internal;

pub fn run_cisc(ir_nodes: &[IR], range_info: &RangeInfo, flush: bool, out_dump: bool) -> Result<(), BrainrotError> {
    let insts = ir_to_bytecodes(ir_nodes, range_info)?;
    let mut tape = Tape::new();
    let mut program = Program::new(&insts, flush);
    let mut tier = if range_info.do_opt_first {
        Tier::Opt
    } else {
        Tier::Deopt
    };
    #[cfg(feature = "trace")]
    println!("[TRACE] first: {:?}", tier);

    loop {
        let result = match tier {
            Tier::Deopt => run_deopt(&mut tape, &mut program),
            Tier::Opt => unsafe {
                run_opt(&mut UnsafeTape::new(&mut tape), &mut UnsafeProgram::new(&mut program))
            },
        };
        match result {
            Ok(InterpreterResult::End) => {
                if cfg!(feature = "debug") && out_dump {
                    write_trace(&program, &insts);
                }
                return Ok(());
            }
            Ok(InterpreterResult::ToggleTier(t)) => {
                #[cfg(feature = "trace")]
                println!("[TRACE] tier switch to {:?}", t);
                tier = t;
            }
            Err(err) => {
                let pc = program.pc();
                if cfg!(feature = "debug") {
                    if out_dump {
                        write_trace(&program, &insts);
                    }
                    println!("PC: {}({:?}), ptr: {}", pc, program.inst(), tape.data_pointer);
                }
                return Err(BrainrotError::RuntimeError {
                    err,
                    pc,
                    pointer: tape.data_pointer,
                });
            }
        }
    }
}

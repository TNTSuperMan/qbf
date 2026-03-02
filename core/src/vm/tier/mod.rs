use crate::{error::BrainrotError, vm::{program::{Program, UnsafeProgram}, tape::{Tape, UnsafeTape}, tier::{deopt::run_deopt, internal::{InterpreterResult, Tier}, opt::run_opt}}};

pub mod internal;
mod deopt;
mod opt;

pub enum BrainrotResult {
    End, IoBreak,
}

pub fn run<I: FnMut() -> u8, O: FnMut(u8) -> ()>(tier: &mut Tier, tape: &mut Tape, program: &mut Program<I, O>) -> Result<BrainrotResult, BrainrotError> {
    loop {
        let result = match tier {
            Tier::Deopt => run_deopt(tape, program),
            Tier::Opt => unsafe {
                run_opt(&mut UnsafeTape::new(tape), &mut UnsafeProgram::new(program))
            },
        };
        match result {
            Ok(InterpreterResult::End) => {
                return Ok(BrainrotResult::End);
            }
            Ok(InterpreterResult::IoBreak) => {
                return Ok(BrainrotResult::IoBreak)
            }
            Ok(InterpreterResult::ToggleTier(t)) => {
                *tier = t;
            }
            Err(err) => {
                return Err(BrainrotError::RuntimeError {
                    err,
                    pc: program.pc(),
                    pointer: tape.data_pointer,
                })
            }
        }
    }
}

use crate::{bytecode::bytecode::Bytecode, error::BrainrotError, vm::{program::Program, tape::Tape, tier::{BrainrotResult, internal::Tier, run}}};

pub mod program;
pub mod tape;
pub mod tier;

pub fn run_cisc<I: FnMut() -> u8, O: FnMut(u8) -> ()>(insts: Box<[Bytecode]>, timeout: Option<usize>, input: I, output: O) -> Result<BrainrotResult, BrainrotError> {
    let mut tape = Tape::new();
    let mut program = Program::new(insts, timeout, input, output, false);
    let mut tier = Tier::Deopt;
    if cfg!(feature = "trace") {
        println!("[TRACE] first: {:?}", tier);
    }

    run(&mut tier, &mut tape, &mut program)
}

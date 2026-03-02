use crate::{bytecode::bytecode::ir_to_bytecodes, error::BrainrotError, ir::{ir::{IR, parse_to_ir}, range::{RangeInfo, generate_range_info}}, trace::{generate_bytecode_trace, generate_ir_trace}, vm::{program::Program, tape::Tape, tier::{BrainrotResult, internal::Tier, run}}};

pub struct BrainrotInit<I, O>
where I: FnMut() -> u8,
      O: FnMut(u8) -> (),
{
    pub input: I,
    pub output: O,
    pub io_break: bool,
    pub timeout_step: Option<usize>,
}

pub struct Brainrot<I, O>
where I: FnMut() -> u8,
      O: FnMut(u8) -> (),
{
    ir: Vec<IR>, range: RangeInfo,

    tier: Tier,
    tape: Tape,
    program: Program<I, O>,
}

impl<I, O> Brainrot<I, O>
where I: FnMut() -> u8,
      O: FnMut(u8) -> (),
{
    pub fn new(code: &str, init: BrainrotInit<I, O>) -> Result<Brainrot<I, O>, BrainrotError> {
        let ir = parse_to_ir(code)?;
        let range = generate_range_info(&ir)?;
        let bytecode = ir_to_bytecodes(&ir, &range)?;

        let tier = if range.do_opt_first { Tier::Opt } else { Tier::Deopt };

        Ok(Brainrot {
            ir, range,

            tier,
            tape: Tape::new(),
            program: Program::new(bytecode.into_boxed_slice(), init.timeout_step, init.input, init.output, init.io_break),
        })
    }
    pub fn step(&mut self) -> Result<BrainrotResult, BrainrotError> {
        run(&mut self.tier, &mut self.tape, &mut self.program)
    }
    pub fn get_tape(&self, pointer: usize) -> Option<&u8> {
        self.tape.buffer.get(pointer)
    }
    pub fn get_tape_mut(&mut self, pointer: usize) -> Option<&mut u8> {
        self.tape.buffer.get_mut(pointer)
    }
    pub fn set_timeout(&mut self, value: Option<usize>) {
        self.program.step_remains = value;
    }
    pub fn generate_trace(&self) -> String {
        let mut trace = String::new();

        trace += "IR:\n";
        trace += &generate_ir_trace(&self.ir, &self.range);
        trace += "\nBytecode:\n";
        trace += &generate_bytecode_trace(&self.program);

        return trace;
    }
}

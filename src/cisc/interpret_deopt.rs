use std::io::{Read, Write, stdin, stdout};

use crate::cisc::{bytecode::Bytecode, error::RuntimeError, internal::{InterpreterResult, Tier}, tape::Tape, program::Program};

pub fn run_deopt(tape: &mut Tape, program: &mut Program) -> Result<InterpreterResult, RuntimeError> {
    let mut stdout = stdout().lock();
    let mut stdin = stdin().lock();
    let mut stdin_buf: [u8; 1] = [0];
    let mut mul_val: u8 = 0;
    
    loop {
        if cfg!(feature = "debug") {
            let pc = program.pc();
            program.ocm.deopt[pc] += 1;
        }

        if cfg!(feature = "trace") {
            println!("[TRACE] tier: Deopt ptr: {}, executing {}", tape.data_pointer, program.pc());
        }
        
        match program.inst() {
            Bytecode::Breakpoint { delta } => {
                tape.step(*delta as isize);
                eprintln!("PC: {}, PTR: {}", program.pc(), tape.data_pointer);
            }

            Bytecode::SingleAdd { delta, val } => {
                tape.step(*delta as isize);
                tape.add(*val)?;
            }
            Bytecode::SingleSet { delta, val } => {
                tape.step(*delta as isize);
                tape.set(*val)?;
            }
            Bytecode::AddAdd { delta1, val1, delta2, val2 } => {
                tape.step(*delta1 as isize);
                tape.add(*val1)?;
                tape.step(*delta2 as isize);
                tape.add(*val2)?;
            }
            Bytecode::AddSet { delta1, val1, delta2, val2 } => {
                tape.step(*delta1 as isize);
                tape.add(*val1)?;
                tape.step(*delta2 as isize);
                tape.set(*val2)?;
            }
            Bytecode::SetAdd { delta1, val1, delta2, val2 } => {
                tape.step(*delta1 as isize);
                tape.set(*val1)?;
                tape.step(*delta2 as isize);
                tape.add(*val2)?;
            }
            Bytecode::SetSet { delta1, val1, delta2, val2 } => {
                tape.step(*delta1 as isize);
                tape.set(*val1)?;
                tape.step(*delta2 as isize);
                tape.set(*val2)?;
            }

            Bytecode::BothRangeCheck { range } => {
                if range.contains(&(tape.data_pointer as u16)) {
                    program.step();
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
            }
            Bytecode::Shift { delta, step } => {
                tape.step(*delta as isize);
                while tape.get()? != 0 {
                    tape.step(*step as isize);
                }
            }
            Bytecode::ShiftP { delta, step, range } => {
                tape.step(*delta as isize);
                while tape.get()? != 0 {
                    tape.step(*step as isize);
                }
                if range.contains(&(tape.data_pointer as u16)) {
                    program.step();
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
            }
            Bytecode::ShiftN { delta, step, range } => {
                tape.step(*delta as isize);
                while tape.get()? != 0 {
                    tape.step(*step as isize);
                }
                if range.contains(&(tape.data_pointer as u16)) {
                    program.step();
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
            }
            Bytecode::ShiftAdd { delta1, step, delta2, val } => {
                tape.step(*delta1 as isize);
                while tape.get()? != 0 {
                    tape.step(*step as isize);
                }
                tape.step(*delta2 as isize);
                tape.add(*val)?;
            }
            Bytecode::ShiftAddP { delta1, step, delta2, val, range } => {
                tape.step(*delta1 as isize);
                while tape.get()? != 0 {
                    tape.step(*step as isize);
                }
                if range.contains(&(tape.data_pointer as u16)) {
                    tape.step(*delta2 as isize);
                    tape.add(*val)?;
                    program.step();
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                tape.step(*delta2 as isize);
                tape.add(*val)?;
            }
            Bytecode::ShiftAddN { delta1, step, delta2, val, range } => {
                tape.step(*delta1 as isize);
                while tape.get()? != 0 {
                    tape.step(*step as isize);
                }
                if range.contains(&(tape.data_pointer as u16)) {
                    tape.step(*delta2 as isize);
                    tape.add(*val)?;
                    program.step();
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                tape.step(*delta2 as isize);
                tape.add(*val)?;
            }
            Bytecode::ShiftSet { delta1, step, delta2, val } => {
                tape.step(*delta1 as isize);
                while tape.get()? != 0 {
                    tape.step(*step as isize);
                }
                tape.step(*delta2 as isize);
                tape.set(*val)?;
            }
            Bytecode::ShiftSetP { delta1, step, delta2, val, range } => {
                tape.step(*delta1 as isize);
                while tape.get()? != 0 {
                    tape.step(*step as isize);
                }
                if range.contains(&(tape.data_pointer as u16)) {
                    tape.step(*delta2 as isize);
                    tape.set(*val)?;
                    program.step();
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                tape.step(*delta2 as isize);
                tape.set(*val)?;
            }
            Bytecode::ShiftSetN { delta1, step, delta2, val, range } => {
                tape.step(*delta1 as isize);
                while tape.get()? != 0 {
                    tape.step(*step as isize);
                }
                if range.contains(&(tape.data_pointer as u16)) {
                    tape.step(*delta2 as isize);
                    tape.set(*val)?;
                    program.step();
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                tape.step(*delta2 as isize);
                tape.set(*val)?;
            }

            Bytecode::MulStart { delta, jz_abs } => {
                tape.step(*delta as isize);
                let val = tape.get()?;
                if val == 0 {
                    program.jump_abs(*jz_abs as usize);
                    continue;
                } else {
                    mul_val = val;
                    tape.set(0)?;
                }
            }
            Bytecode::Mul { delta, val } => {
                tape.add_with_offset(*delta as isize, mul_val.wrapping_mul(*val))?;
            }

            Bytecode::SingleMoveAdd { delta, to } => {
                tape.step(*delta as isize);
                let v = tape.get()?;
                if v != 0 {
                    tape.set(0)?;
                    tape.add_with_offset(*to as isize, v)?;
                }
            }
            Bytecode::SingleMoveSub { delta, to } => {
                tape.step(*delta as isize);
                let v = tape.get()?;
                if v != 0 {
                    tape.set(0)?;
                    tape.sub_with_offset(*to as isize, v)?;
                }
            }

            Bytecode::DoubleMoveAddAdd { delta, to1, to2 } => {
                tape.step(*delta as isize);
                let v = tape.get()?;
                if v != 0 {
                    tape.add_with_offset(*to1 as isize, v)?;
                    tape.add_with_offset(*to2 as isize, v)?;
                    tape.set(0)?;
                }
            }
            Bytecode::DoubleMoveAddSub { delta, to1, to2 } => {
                tape.step(*delta as isize);
                let v = tape.get()?;
                if v != 0 {
                    tape.add_with_offset(*to1 as isize, v)?;
                    tape.sub_with_offset(*to2 as isize, v)?;
                    tape.set(0)?;
                }
            }
            Bytecode::DoubleMoveSubAdd { delta, to1, to2 } => {
                tape.step(*delta as isize);
                let v = tape.get()?;
                if v != 0 {
                    tape.sub_with_offset(*to1 as isize, v)?;
                    tape.add_with_offset(*to2 as isize, v)?;
                    tape.set(0)?;
                }
            }
            Bytecode::DoubleMoveSubSub { delta, to1, to2 } => {
                tape.step(*delta as isize);
                let v = tape.get()?;
                if v != 0 {
                    tape.sub_with_offset(*to1 as isize, v)?;
                    tape.sub_with_offset(*to2 as isize, v)?;
                    tape.set(0)?;
                }
            }

            Bytecode::MoveStart { delta, jz_abs } => {
                tape.step(*delta as isize);
                let val = tape.get()?;
                if val == 0 {
                    program.jump_abs(*jz_abs as usize);
                    continue;
                } else {
                    mul_val = val;
                    tape.set(0)?;
                }
            }
            Bytecode::MoveAdd { delta } => {
                tape.add_with_offset(*delta as isize, mul_val)?;
            }
            Bytecode::MoveSub { delta } => {
                tape.sub_with_offset(*delta as isize, mul_val)?;
            }

            Bytecode::In { delta } => {
                tape.step(*delta as isize);
                match stdin.read_exact(&mut stdin_buf) {
                    Ok(_) => tape.set(stdin_buf[0])?,
                    Err(_) => tape.set(0)?,
                }
            }
            Bytecode::Out { delta } => {
                tape.step(*delta as isize);
                stdout.write(&[tape.get()?])?;
                if program.flush {
                    stdout.flush()?;
                }
            }

            Bytecode::JmpIfZero { delta, addr_abs } => {
                tape.step(*delta as isize);
                if tape.get()? == 0 {
                    program.jump_abs(*addr_abs as usize);
                    continue;
                }
            }
            Bytecode::JmpIfNotZero { delta, addr_abs } => {
                tape.step(*delta as isize);
                if tape.get()? != 0 {
                    program.jump_abs((*addr_abs) as usize);
                    continue;
                }
            }
            Bytecode::PositiveRangeCheckJNZ { delta, addr_back, range } => {
                tape.step(*delta as isize);
                if range.contains(&(tape.data_pointer as u16)) {
                    if tape.get()? != 0 {
                        program.jump_back(*addr_back as usize);
                    } else {
                        program.step();
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                if tape.get()? != 0 {
                    program.jump_back(*addr_back as usize);
                    continue;
                }
            }
            Bytecode::NegativeRangeCheckJNZ { delta, addr_back, range } => {
                tape.step(*delta as isize);
                if range.contains(&(tape.data_pointer as u16)) {
                    if tape.get()? != 0 {
                        program.jump_back(*addr_back as usize);
                    } else {
                        program.step();
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                if tape.get()? != 0 {
                    program.jump_back(*addr_back as usize);
                    continue;
                }
            }
            Bytecode::BothRangeCheckJNZ { delta, addr_back, range } => {
                tape.step(*delta as isize);
                if range.contains(&(tape.data_pointer as u16)) {
                    if tape.get()? != 0 {
                        program.jump_back(*addr_back as usize);
                    } else {
                        program.step();
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                if tape.get()? != 0 {
                    program.jump_back(*addr_back as usize);
                    continue;
                }
            }

            Bytecode::End { delta } => {
                tape.step(*delta as isize);
                return Ok(InterpreterResult::End);
            }
        }
        program.step();
    }
}

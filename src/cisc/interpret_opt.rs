use std::io::{Read, Write, stdin, stdout};

use crate::cisc::{bytecode::Bytecode, error::RuntimeError, internal::{InterpreterResult, Tier}, tape::UnsafeTape, program::UnsafeProgram};

pub unsafe fn run_opt(tape: &mut UnsafeTape, program: &mut UnsafeProgram) -> Result<InterpreterResult, RuntimeError> {
    let mut stdout = stdout().lock();
    let mut stdin = stdin().lock();
    let mut stdin_buf: [u8; 1] = [0];
    let mut mul_val: u8 = 0;
    
    loop {
        if cfg!(feature = "debug") {
            let pc = program.pc();
            program.inner.ocm.opt[pc] += 1;

            program.check_timeout()?;
        }

        if cfg!(feature = "trace") {
            println!("[TRACE] tier: Opt ptr: {}, val: {}, executing {}", tape.get_ptr(), tape.get(), program.pc());
        }
        
        match program.inst() {
            Bytecode::Breakpoint { delta } => {
                tape.step_ptr((*delta) as isize);
                eprintln!("PC: {}, PTR: {}", program.pc(), tape.get_ptr());
            }

            Bytecode::SingleAdd { delta, val } => {
                tape.step_ptr((*delta) as isize);
                tape.add(*val);
            }
            Bytecode::SingleSet { delta, val } => {
                tape.step_ptr((*delta) as isize);
                tape.set(*val);
            }
            Bytecode::AddAdd { delta1, val1, delta2, val2 } => {
                tape.step_ptr((*delta1) as isize);
                tape.add(*val1);
                tape.step_ptr((*delta2) as isize);
                tape.add(*val2);
            }
            Bytecode::AddSet { delta1, val1, delta2, val2 } => {
                tape.step_ptr((*delta1) as isize);
                tape.add(*val1);
                tape.step_ptr((*delta2) as isize);
                tape.set(*val2);
            }
            Bytecode::SetAdd { delta1, val1, delta2, val2 } => {
                tape.step_ptr((*delta1) as isize);
                tape.set(*val1);
                tape.step_ptr((*delta2) as isize);
                tape.add(*val2);
            }
            Bytecode::SetSet { delta1, val1, delta2, val2 } => {
                tape.step_ptr((*delta1) as isize);
                tape.set(*val1);
                tape.step_ptr((*delta2) as isize);
                tape.set(*val2);
            }

            Bytecode::BothRangeCheck { range } => {
                if !range.contains(&(tape.get_ptr() as u16)) {
                    program.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
            }
            Bytecode::Shift { delta, step } => {
                tape.step_ptr((*delta) as isize);
                while tape.get_safe(tape.get_ptr())? != 0 {
                    tape.step_ptr((*step) as isize);
                }
            }
            Bytecode::ShiftP { delta, step, range } => {
                tape.step_ptr((*delta) as isize);
                while tape.get_safe(tape.get_ptr())? != 0 {
                    tape.step_ptr((*step) as isize);
                }
                if !range.contains(&(tape.get_ptr() as u16)) {
                    program.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
            }
            Bytecode::ShiftN { delta, step, range } => {
                tape.step_ptr((*delta) as isize);
                while tape.get_safe(tape.get_ptr())? != 0 {
                    tape.step_ptr((*step) as isize);
                }
                if !range.contains(&(tape.get_ptr() as u16)) {
                    program.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
            }
            Bytecode::ShiftAdd { delta1, step, delta2, val } => {
                tape.step_ptr((*delta1) as isize);
                while tape.get_safe(tape.get_ptr())? != 0 {
                    tape.step_ptr((*step) as isize);
                }
                tape.step_ptr((*delta2) as isize);
                tape.add(*val);
            }
            Bytecode::ShiftAddP { delta1, step, delta2, val, range } => {
                tape.step_ptr((*delta1) as isize);
                while tape.get_safe(tape.get_ptr())? != 0 {
                    tape.step_ptr((*step) as isize);
                }
                if !range.contains(&(tape.get_ptr() as u16)) {
                    tape.step_ptr((*delta2) as isize);
                    tape.add_safe(tape.get_ptr(), *val)?;
                    program.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                tape.step_ptr((*delta2) as isize);
                tape.add(*val);
            }
            Bytecode::ShiftAddN { delta1, step, delta2, val, range } => {
                tape.step_ptr((*delta1) as isize);
                while tape.get_safe(tape.get_ptr())? != 0 {
                    tape.step_ptr((*step) as isize);
                }
                if !range.contains(&(tape.get_ptr() as u16)) {
                    tape.step_ptr((*delta2) as isize);
                    tape.add_safe(tape.get_ptr(), *val)?;
                    program.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                tape.step_ptr((*delta2) as isize);
                tape.add(*val);
            }
            Bytecode::ShiftSet { delta1, step, delta2, val } => {
                tape.step_ptr((*delta1) as isize);
                while tape.get_safe(tape.get_ptr())? != 0 {
                    tape.step_ptr((*step) as isize);
                }
                tape.step_ptr((*delta2) as isize);
                tape.set(*val);
            }
            Bytecode::ShiftSetP { delta1, step, delta2, val, range } => {
                tape.step_ptr((*delta1) as isize);
                while tape.get_safe(tape.get_ptr())? != 0 {
                    tape.step_ptr((*step) as isize);
                }
                if !range.contains(&(tape.get_ptr() as u16)) {
                    tape.step_ptr((*delta2) as isize);
                    tape.set_safe(tape.get_ptr(), *val)?;
                    program.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                tape.step_ptr((*delta2) as isize);
                tape.set(*val);
            }
            Bytecode::ShiftSetN { delta1, step, delta2, val, range } => {
                tape.step_ptr((*delta1) as isize);
                while tape.get_safe(tape.get_ptr())? != 0 {
                    tape.step_ptr((*step) as isize);
                }
                if !range.contains(&(tape.get_ptr() as u16)) {
                    tape.step_ptr((*delta2) as isize);
                    tape.set_safe(tape.get_ptr(), *val)?;
                    program.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                tape.step_ptr((*delta2) as isize);
                tape.set(*val);
            }

            Bytecode::MulStart { delta, jz_abs } => {
                tape.step_ptr((*delta) as isize);
                let val = tape.get();
                if val == 0 {
                    program.jump_abs(*jz_abs);
                    continue;
                } else {
                    mul_val = val;
                    tape.set(0);
                }
            }
            Bytecode::Mul { delta, val } => {
                tape.add_with_offset((*delta) as isize, mul_val.wrapping_mul(*val));
            }

            Bytecode::SingleMoveAdd { delta, to } => {
                tape.step_ptr((*delta) as isize);
                tape.add_with_offset((*to) as isize, tape.get());
                tape.set(0);
            }
            Bytecode::SingleMoveSub { delta, to } => {
                tape.step_ptr((*delta) as isize);
                tape.sub_with_offset((*to) as isize, tape.get());
                tape.set(0);
            }

            Bytecode::DoubleMoveAddAdd { delta, to1, to2 } => {
                tape.step_ptr((*delta) as isize);
                let v = tape.get();
                tape.add_with_offset((*to1) as isize, v);
                tape.add_with_offset((*to2) as isize, v);
                tape.set(0);
            }
            Bytecode::DoubleMoveAddSub { delta, to1, to2 } => {
                tape.step_ptr((*delta) as isize);
                let v = tape.get();
                tape.add_with_offset((*to1) as isize, v);
                tape.sub_with_offset((*to2) as isize, v);
                tape.set(0);
            }
            Bytecode::DoubleMoveSubAdd { delta, to1, to2 } => {
                tape.step_ptr((*delta) as isize);
                let v = tape.get();
                tape.sub_with_offset((*to1) as isize, v);
                tape.add_with_offset((*to2) as isize, v);
                tape.set(0);
            }
            Bytecode::DoubleMoveSubSub { delta, to1, to2 } => {
                tape.step_ptr((*delta) as isize);
                let v = tape.get();
                tape.sub_with_offset((*to1) as isize, v);
                tape.sub_with_offset((*to2) as isize, v);
                tape.set(0);
            }

            Bytecode::MoveStart { delta, jz_abs } => {
                tape.step_ptr((*delta) as isize);
                let val = tape.get();
                if val == 0 {
                    program.jump_abs(*jz_abs);
                    continue;
                } else {
                    mul_val = val;
                    tape.set(0);
                }
            }
            Bytecode::MoveAdd { delta } => {
                tape.add_with_offset((*delta) as isize, mul_val);
            }
            Bytecode::MoveSub { delta } => {
                tape.sub_with_offset((*delta) as isize, mul_val);
            }

            Bytecode::In { delta } => {
                tape.step_ptr((*delta) as isize);
                match stdin.read_exact(&mut stdin_buf) {
                    Ok(_) => tape.set(stdin_buf[0]),
                    Err(_) => tape.set(0),
                }
            }
            Bytecode::Out { delta } => {
                tape.step_ptr((*delta) as isize);
                stdout.write(&[tape.get()])?;
                if program.inner.flush {
                    stdout.flush()?;
                }
            }

            Bytecode::JmpIfZero { delta, addr_abs } => {
                tape.step_ptr((*delta) as isize);
                if tape.get() == 0 {
                    program.jump_abs(*addr_abs);
                    continue;
                }
            }
            Bytecode::JmpIfNotZero { delta, addr_abs } => {
                tape.step_ptr((*delta) as isize);
                if tape.get() != 0 {
                    program.jump_abs(*addr_abs);
                    continue;
                }
            }
            Bytecode::PositiveRangeCheckJNZ { delta, addr_back, range } => {
                tape.step_ptr((*delta) as isize);
                if !range.contains(&(tape.get_ptr() as u16)) {
                    if tape.get_safe(tape.get_ptr())? != 0 {
                        program.jump_back(*addr_back);
                    } else {
                        program.jump_one();
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                if tape.get() != 0 {
                    program.jump_back(*addr_back);
                    continue;
                }
            }
            Bytecode::NegativeRangeCheckJNZ { delta, addr_back, range } => {
                tape.step_ptr((*delta) as isize);
                if !range.contains(&(tape.get_ptr() as u16)) {
                    if tape.get_safe(tape.get_ptr())? != 0 {
                        program.jump_back(*addr_back);
                    } else {
                        program.jump_one();
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                if tape.get() != 0 {
                    program.jump_back(*addr_back);
                    continue;
                }
            }
            Bytecode::BothRangeCheckJNZ { delta, addr_back, range } => {
                tape.step_ptr((*delta) as isize);
                let ptr = tape.get_ptr();
                if !range.contains(&(ptr as u16)) {
                    if tape.get_safe(ptr)? != 0 {
                        program.jump_back(*addr_back);
                    } else {
                        program.jump_one();
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                if tape.get() != 0 {
                    program.jump_back(*addr_back);
                    continue;
                }
            }

            Bytecode::End { delta } => {
                tape.step_ptr((*delta) as isize);
                return Ok(InterpreterResult::End);
            }
        }
        program.jump_one();
    }
}

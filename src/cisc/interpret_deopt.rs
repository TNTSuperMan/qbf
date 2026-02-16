use std::io::{Read, Write, stdin, stdout};

use crate::cisc::{bytecode::Bytecode, error::RuntimeError, internal::{InterpreterResult, Tier}, vm::VM};

pub fn run_deopt(vm: &mut VM, insts: &[Bytecode]) -> Result<InterpreterResult, RuntimeError> {
    let mut stdout = stdout().lock();
    let mut stdin = stdin().lock();
    let mut stdin_buf: [u8; 1] = [0];
    let mut mul_val: u8 = 0;
    
    loop {
        if cfg!(feature = "debug") {
            vm.ocm.deopt[vm.pc] += 1;
        }

        let bytecode = &insts[vm.pc];
        
        if cfg!(feature = "trace") {
            println!("[TRACE] tier: Deopt ptr: {}, executing {}", vm.pointer, vm.pc);
        }
        
        match bytecode {
            Bytecode::Breakpoint { delta } => {
                vm.step_ptr((*delta) as isize);
                eprintln!("PC: {}, PTR: {}", vm.pc, vm.pointer);
            }

            Bytecode::SingleAdd { delta, val } => {
                vm.step_ptr((*delta) as isize);
                vm.memory.add(vm.pointer, *val)?;
            }
            Bytecode::SingleSet { delta, val } => {
                vm.step_ptr((*delta) as isize);
                vm.memory.set(vm.pointer, *val)?;
            }
            Bytecode::AddAdd { delta1, val1, delta2, val2 } => {
                vm.step_ptr((*delta1) as isize);
                vm.memory.add(vm.pointer, *val1)?;
                vm.step_ptr((*delta2) as isize);
                vm.memory.add(vm.pointer, *val2)?;
            }
            Bytecode::AddSet { delta1, val1, delta2, val2 } => {
                vm.step_ptr((*delta1) as isize);
                vm.memory.add(vm.pointer, *val1)?;
                vm.step_ptr((*delta2) as isize);
                vm.memory.set(vm.pointer, *val2)?;
            }
            Bytecode::SetAdd { delta1, val1, delta2, val2 } => {
                vm.step_ptr((*delta1) as isize);
                vm.memory.set(vm.pointer, *val1)?;
                vm.step_ptr((*delta2) as isize);
                vm.memory.add(vm.pointer, *val2)?;
            }
            Bytecode::SetSet { delta1, val1, delta2, val2 } => {
                vm.step_ptr((*delta1) as isize);
                vm.memory.set(vm.pointer, *val1)?;
                vm.step_ptr((*delta2) as isize);
                vm.memory.set(vm.pointer, *val2)?;
            }

            Bytecode::BothRangeCheck { range } => {
                if range.contains(&(vm.pointer as u16)) {
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
            }
            Bytecode::Shift { delta, step } => {
                vm.step_ptr((*delta) as isize);
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr((*step) as isize);
                }
            }
            Bytecode::ShiftP { delta, step, range } => {
                vm.step_ptr((*delta) as isize);
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr((*step) as isize);
                }
                if range.contains(&(vm.pointer as u16)) {
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
            }
            Bytecode::ShiftN { delta, step, range } => {
                vm.step_ptr((*delta) as isize);
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr((*step) as isize);
                }
                if range.contains(&(vm.pointer as u16)) {
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
            }
            Bytecode::ShiftAdd { delta1, step, delta2, val } => {
                vm.step_ptr((*delta1) as isize);
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr((*step) as isize);
                }
                vm.step_ptr((*delta2) as isize);
                vm.memory.add(vm.pointer, *val)?;
            }
            Bytecode::ShiftAddP { delta1, step, delta2, val, range } => {
                vm.step_ptr((*delta1) as isize);
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr((*step) as isize);
                }
                if range.contains(&(vm.pointer as u16)) {
                    vm.step_ptr((*delta2) as isize);
                    vm.memory.add(vm.pointer, *val)?;
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                vm.step_ptr((*delta2) as isize);
                vm.memory.add(vm.pointer, *val)?;
            }
            Bytecode::ShiftAddN { delta1, step, delta2, val, range } => {
                vm.step_ptr((*delta1) as isize);
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr((*step) as isize);
                }
                if range.contains(&(vm.pointer as u16)) {
                    vm.step_ptr((*delta2) as isize);
                    vm.memory.add(vm.pointer, *val)?;
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                vm.step_ptr((*delta2) as isize);
                vm.memory.add(vm.pointer, *val)?;
            }
            Bytecode::ShiftSet { delta1, step, delta2, val } => {
                vm.step_ptr((*delta1) as isize);
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr((*step) as isize);
                }
                vm.step_ptr((*delta2) as isize);
                vm.memory.set(vm.pointer, *val)?;
            }
            Bytecode::ShiftSetP { delta1, step, delta2, val, range } => {
                vm.step_ptr((*delta1) as isize);
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr((*step) as isize);
                }
                if range.contains(&(vm.pointer as u16)) {
                    vm.step_ptr((*delta2) as isize);
                    vm.memory.set(vm.pointer, *val)?;
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                vm.step_ptr((*delta2) as isize);
                vm.memory.set(vm.pointer, *val)?;
            }
            Bytecode::ShiftSetN { delta1, step, delta2, val, range } => {
                vm.step_ptr((*delta1) as isize);
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr((*step) as isize);
                }
                if range.contains(&(vm.pointer as u16)) {
                    vm.step_ptr((*delta2) as isize);
                    vm.memory.set(vm.pointer, *val)?;
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                vm.step_ptr((*delta2) as isize);
                vm.memory.set(vm.pointer, *val)?;
            }

            Bytecode::MulStart { delta, jz_abs } => {
                vm.step_ptr((*delta) as isize);
                let val = vm.memory.get(vm.pointer)?;
                if val == 0 {
                    vm.pc = (*jz_abs) as usize;
                    continue;
                } else {
                    mul_val = val;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }
            Bytecode::Mul { delta, val } => {
                vm.memory.add(vm.pointer.wrapping_add_signed((*delta) as isize), mul_val.wrapping_mul(*val))?;
            }

            Bytecode::SingleMoveAdd { delta, to } => {
                vm.step_ptr((*delta) as isize);
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    vm.memory.set(vm.pointer, 0)?;
                    vm.memory.add(vm.pointer.wrapping_add_signed((*to) as isize), v)?;
                }
            }
            Bytecode::SingleMoveSub { delta, to } => {
                vm.step_ptr((*delta) as isize);
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    vm.memory.set(vm.pointer, 0)?;
                    vm.memory.sub(vm.pointer.wrapping_add_signed((*to) as isize), v)?;
                }
            }

            Bytecode::DoubleMoveAddAdd { delta, to1, to2 } => {
                vm.step_ptr((*delta) as isize);
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    vm.memory.add(vm.pointer.wrapping_add_signed((*to1) as isize), v)?;
                    vm.memory.add(vm.pointer.wrapping_add_signed((*to2) as isize), v)?;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }
            Bytecode::DoubleMoveAddSub { delta, to1, to2 } => {
                vm.step_ptr((*delta) as isize);
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    vm.memory.add(vm.pointer.wrapping_add_signed((*to1) as isize), v)?;
                    vm.memory.sub(vm.pointer.wrapping_add_signed((*to2) as isize), v)?;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }
            Bytecode::DoubleMoveSubAdd { delta, to1, to2 } => {
                vm.step_ptr((*delta) as isize);
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    vm.memory.sub(vm.pointer.wrapping_add_signed((*to1) as isize), v)?;
                    vm.memory.add(vm.pointer.wrapping_add_signed((*to2) as isize), v)?;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }
            Bytecode::DoubleMoveSubSub { delta, to1, to2 } => {
                vm.step_ptr((*delta) as isize);
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    vm.memory.sub(vm.pointer.wrapping_add_signed((*to1) as isize), v)?;
                    vm.memory.sub(vm.pointer.wrapping_add_signed((*to2) as isize), v)?;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }

            Bytecode::MoveStart { delta, jz_abs } => {
                vm.step_ptr((*delta) as isize);
                let val = vm.memory.get(vm.pointer)?;
                if val == 0 {
                    vm.pc = (*jz_abs) as usize;
                    continue;
                } else {
                    mul_val = val;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }
            Bytecode::MoveAdd { delta } => {
                vm.memory.add(vm.pointer.wrapping_add_signed((*delta) as isize), mul_val)?;
            }
            Bytecode::MoveSub { delta } => {
                vm.memory.sub(vm.pointer.wrapping_add_signed((*delta) as isize), mul_val)?;
            }

            Bytecode::In { delta } => {
                vm.step_ptr((*delta) as isize);
                match stdin.read_exact(&mut stdin_buf) {
                    Ok(_) => vm.memory.set(vm.pointer, stdin_buf[0])?,
                    Err(_) => vm.memory.set(vm.pointer, 0)?,
                }
            }
            Bytecode::Out { delta } => {
                vm.step_ptr((*delta) as isize);
                stdout.write(&[vm.memory.get(vm.pointer)?])?;
                if vm.flush {
                    stdout.flush()?;
                }
            }

            Bytecode::JmpIfZero { delta, addr_abs } => {
                vm.step_ptr((*delta) as isize);
                if vm.memory.get(vm.pointer)? == 0 {
                    vm.pc = (*addr_abs) as usize;
                    continue;
                }
            }
            Bytecode::JmpIfNotZero { delta, addr_abs } => {
                vm.step_ptr((*delta) as isize);
                if vm.memory.get(vm.pointer)? != 0 {
                    vm.pc = (*addr_abs) as usize;
                    continue;
                }
            }
            Bytecode::PositiveRangeCheckJNZ { delta, addr_back, range } => {
                vm.step_ptr((*delta) as isize);
                if range.contains(&(vm.pointer as u16)) {
                    if vm.memory.get(vm.pointer)? != 0 {
                        vm.pc -= (*addr_back) as usize;
                    } else {
                        vm.pc += 1;
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                if vm.memory.get(vm.pointer)? != 0 {
                    vm.pc -= (*addr_back) as usize;
                    continue;
                }
            }
            Bytecode::NegativeRangeCheckJNZ { delta, addr_back, range } => {
                vm.step_ptr((*delta) as isize);
                if range.contains(&(vm.pointer as u16)) {
                    if vm.memory.get(vm.pointer)? != 0 {
                        vm.pc -= (*addr_back) as usize;
                    } else {
                        vm.pc += 1;
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                if vm.memory.get(vm.pointer)? != 0 {
                    vm.pc -= (*addr_back) as usize;
                    continue;
                }
            }
            Bytecode::BothRangeCheckJNZ { delta, addr_back, range } => {
                vm.step_ptr((*delta) as isize);
                if range.contains(&(vm.pointer as u16)) {
                    if vm.memory.get(vm.pointer)? != 0 {
                        vm.pc -= (*addr_back) as usize;
                    } else {
                        vm.pc += 1;
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                if vm.memory.get(vm.pointer)? != 0 {
                    vm.pc -= (*addr_back) as usize;
                    continue;
                }
            }

            Bytecode::End { delta } => {
                vm.step_ptr((*delta) as isize);
                return Ok(InterpreterResult::End);
            }
        }
        vm.pc += 1;
    }
}

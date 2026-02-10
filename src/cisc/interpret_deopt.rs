use std::io::{Read, Write, stdin, stdout};

use crate::{cisc::{bytecode::NewBytecode, internal::{InterpreterResult, Tier}, vm::VM}, range::{negative_is_out_of_range, positive_is_out_of_range}};

pub fn run_deopt(vm: &mut VM) -> Result<InterpreterResult, String> {
    let mut stdout = stdout().lock();
    let mut stdin = stdin().lock();
    let mut stdin_buf: [u8; 1] = [0];
    let mut mul_val: u8 = 0;
    
    loop {
        #[cfg(feature = "debug")] {
            vm.ocm.0[vm.pc] += 1;
        }

        let bytecode = vm.insts[vm.pc];
        
        #[cfg(feature = "trace")]
        println!("[TRACE] tier: Deopt ptr: {}, executing {}", vm.pointer, vm.pc);
        
        match bytecode {
            NewBytecode::Breakpoint { delta } => {
                vm.step_ptr(delta as isize);
                eprintln!("PC: {}, PTR: {}", vm.pc, vm.pointer);
            }

            NewBytecode::SingleAdd { delta, val } => {
                vm.step_ptr(delta as isize);
                vm.memory.add(vm.pointer, val)?;
            }
            NewBytecode::SingleSet { delta, val } => {
                vm.step_ptr(delta as isize);
                vm.memory.set(vm.pointer, val)?;
            }
            NewBytecode::AddAdd { delta1, val1, delta2, val2 } => {
                vm.step_ptr(delta1 as isize);
                vm.memory.add(vm.pointer, val1)?;
                vm.step_ptr(delta2 as isize);
                vm.memory.add(vm.pointer, val2)?;
            }
            NewBytecode::AddSet { delta1, val1, delta2, val2 } => {
                vm.step_ptr(delta1 as isize);
                vm.memory.add(vm.pointer, val1)?;
                vm.step_ptr(delta2 as isize);
                vm.memory.set(vm.pointer, val2)?;
            }
            NewBytecode::SetAdd { delta1, val1, delta2, val2 } => {
                vm.step_ptr(delta1 as isize);
                vm.memory.set(vm.pointer, val1)?;
                vm.step_ptr(delta2 as isize);
                vm.memory.add(vm.pointer, val2)?;
            }
            NewBytecode::SetSet { delta1, val1, delta2, val2 } => {
                vm.step_ptr(delta1 as isize);
                vm.memory.set(vm.pointer, val1)?;
                vm.step_ptr(delta2 as isize);
                vm.memory.set(vm.pointer, val2)?;
            }

            NewBytecode::BothRangeCheck { positive, negative } => {
                if !positive_is_out_of_range(positive, vm.pointer) && !negative_is_out_of_range(negative, vm.pointer) {
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
            }
            NewBytecode::Shift { delta, step } => {
                vm.step_ptr(delta as isize);
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr(step as isize);
                }
            }
            NewBytecode::ShiftP { delta, step, range } => {
                vm.step_ptr(delta as isize);
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr(step as isize);
                }
                if !positive_is_out_of_range(range, vm.pointer) {
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
            }
            NewBytecode::ShiftN { delta, step, range } => {
                vm.step_ptr(delta as isize);
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr(step as isize);
                }
                if !negative_is_out_of_range(range, vm.pointer) {
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
            }
            NewBytecode::ShiftAdd { delta1, step, delta2, val } => {
                vm.step_ptr(delta1 as isize);
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr(step as isize);
                }
                vm.step_ptr(delta2 as isize);
                vm.memory.add(vm.pointer, val)?;
            }
            NewBytecode::ShiftAddP { delta1, step, delta2, val, range } => {
                vm.step_ptr(delta1 as isize);
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr(step as isize);
                }
                if !positive_is_out_of_range(range, vm.pointer) {
                    vm.step_ptr(delta2 as isize);
                    vm.memory.add(vm.pointer, val)?;
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                vm.step_ptr(delta2 as isize);
                vm.memory.add(vm.pointer, val)?;
            }
            NewBytecode::ShiftAddN { delta1, step, delta2, val, range } => {
                vm.step_ptr(delta1 as isize);
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr(step as isize);
                }
                if !negative_is_out_of_range(range, vm.pointer) {
                    vm.step_ptr(delta2 as isize);
                    vm.memory.add(vm.pointer, val)?;
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                vm.step_ptr(delta2 as isize);
                vm.memory.add(vm.pointer, val)?;
            }
            NewBytecode::ShiftSet { delta1, step, delta2, val } => {
                vm.step_ptr(delta1 as isize);
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr(step as isize);
                }
                vm.step_ptr(delta2 as isize);
                vm.memory.set(vm.pointer, val)?;
            }
            NewBytecode::ShiftSetP { delta1, step, delta2, val, range } => {
                vm.step_ptr(delta1 as isize);
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr(step as isize);
                }
                if !positive_is_out_of_range(range, vm.pointer) {
                    vm.step_ptr(delta2 as isize);
                    vm.memory.set(vm.pointer, val)?;
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                vm.step_ptr(delta2 as isize);
                vm.memory.set(vm.pointer, val)?;
            }
            NewBytecode::ShiftSetN { delta1, step, delta2, val, range } => {
                vm.step_ptr(delta1 as isize);
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr(step as isize);
                }
                if !negative_is_out_of_range(range, vm.pointer) {
                    vm.step_ptr(delta2 as isize);
                    vm.memory.set(vm.pointer, val)?;
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                vm.step_ptr(delta2 as isize);
                vm.memory.set(vm.pointer, val)?;
            }

            NewBytecode::MulStart { delta, jz } => {
                vm.step_ptr(delta as isize);
                let val = vm.memory.get(vm.pointer)?;
                if val == 0 {
                    vm.pc = jz as usize;
                    continue;
                } else {
                    mul_val = val;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }
            NewBytecode::Mul { delta, val } => {
                vm.memory.add(vm.pointer.wrapping_add_signed(delta as isize), mul_val.wrapping_mul(val))?;
            }

            NewBytecode::SingleMoveAdd { delta, to } => {
                vm.step_ptr(delta as isize);
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    vm.memory.set(vm.pointer, 0)?;
                    vm.memory.add(vm.pointer.wrapping_add_signed(to as isize), v)?;
                }
            }
            NewBytecode::SingleMoveSub { delta, to } => {
                vm.step_ptr(delta as isize);
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    vm.memory.set(vm.pointer, 0)?;
                    vm.memory.sub(vm.pointer.wrapping_add_signed(to as isize), v)?;
                }
            }

            NewBytecode::DoubleMoveAddAdd { delta, to1, to2 } => {
                vm.step_ptr(delta as isize);
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    vm.memory.add(vm.pointer.wrapping_add_signed(to1 as isize), v)?;
                    vm.memory.add(vm.pointer.wrapping_add_signed(to2 as isize), v)?;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }
            NewBytecode::DoubleMoveAddSub { delta, to1, to2 } => {
                vm.step_ptr(delta as isize);
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    vm.memory.add(vm.pointer.wrapping_add_signed(to1 as isize), v)?;
                    vm.memory.sub(vm.pointer.wrapping_add_signed(to2 as isize), v)?;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }
            NewBytecode::DoubleMoveSubAdd { delta, to1, to2 } => {
                vm.step_ptr(delta as isize);
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    vm.memory.sub(vm.pointer.wrapping_add_signed(to1 as isize), v)?;
                    vm.memory.add(vm.pointer.wrapping_add_signed(to2 as isize), v)?;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }
            NewBytecode::DoubleMoveSubSub { delta, to1, to2 } => {
                vm.step_ptr(delta as isize);
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    vm.memory.sub(vm.pointer.wrapping_add_signed(to1 as isize), v)?;
                    vm.memory.sub(vm.pointer.wrapping_add_signed(to2 as isize), v)?;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }

            NewBytecode::MoveStart { delta, jz } => {
                vm.step_ptr(delta as isize);
                let val = vm.memory.get(vm.pointer)?;
                if val == 0 {
                    vm.pc = jz as usize;
                    continue;
                } else {
                    mul_val = val;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }
            NewBytecode::MoveAdd { delta } => {
                vm.memory.add(vm.pointer.wrapping_add_signed(delta as isize), mul_val)?;
            }
            NewBytecode::MoveSub { delta } => {
                vm.memory.sub(vm.pointer.wrapping_add_signed(delta as isize), mul_val)?;
            }

            NewBytecode::In { delta } => {
                vm.step_ptr(delta as isize);
                match stdin.read_exact(&mut stdin_buf) {
                    Ok(_) => vm.memory.set(vm.pointer, stdin_buf[0])?,
                    Err(_) => vm.memory.set(vm.pointer, 0)?,
                }
            }
            NewBytecode::Out { delta } => {
                vm.step_ptr(delta as isize);
                stdout.write(&[vm.memory.get(vm.pointer)?]).map_err(|_| "Runtime Error: Failed to print")?;
            }

            NewBytecode::JmpIfZero { delta, addr } => {
                vm.step_ptr(delta as isize);
                if vm.memory.get(vm.pointer)? == 0 {
                    vm.pc += addr as usize;
                    continue;
                }
            }
            NewBytecode::JmpIfNotZero { delta, addr } => {
                vm.step_ptr(delta as isize);
                if vm.memory.get(vm.pointer)? != 0 {
                    vm.pc -= addr as usize;
                    continue;
                }
            }
            NewBytecode::PositiveRangeCheckJNZ { delta, addr, range } => {
                vm.step_ptr(delta as isize);
                if !positive_is_out_of_range(range, vm.pointer) {
                    if vm.memory.get(vm.pointer)? != 0 {
                        vm.pc -= addr as usize;
                    } else {
                        vm.pc += 1;
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                if vm.memory.get(vm.pointer)? != 0 {
                    vm.pc -= addr as usize;
                    continue;
                }
            }
            NewBytecode::NegativeRangeCheckJNZ { delta, addr, range } => {
                vm.step_ptr(delta as isize);
                if !negative_is_out_of_range(range, vm.pointer) {
                    if vm.memory.get(vm.pointer)? != 0 {
                        vm.pc -= addr as usize;
                    } else {
                        vm.pc += 1;
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                if vm.memory.get(vm.pointer)? != 0 {
                    vm.pc -= addr as usize;
                    continue;
                }
            }
            NewBytecode::BothRangeCheckJNZ { delta, addr: addr_subrel, positive, negative } => {
                vm.step_ptr(delta as isize);
                if !positive_is_out_of_range(positive, vm.pointer) && !negative_is_out_of_range(negative, vm.pointer) {
                    if vm.memory.get(vm.pointer)? != 0 {
                        vm.pc -= addr_subrel as usize;
                    } else {
                        vm.pc += 1;
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                if vm.memory.get(vm.pointer)? != 0 {
                    vm.pc -= addr_subrel as usize;
                    continue;
                }
            }

            NewBytecode::End { delta } => {
                vm.step_ptr(delta as isize);
                return Ok(InterpreterResult::End);
            }
        }
        vm.pc += 1;
    }
}

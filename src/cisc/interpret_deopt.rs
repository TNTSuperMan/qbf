use std::io::{Read, Write, stdin, stdout};

use crate::cisc::{bytecode::OpCode, internal::{InterpreterResult, Tier, negative_out_of_range, positive_out_of_range, u32_to_delta_and_two_val, u32_to_delta_and_val, u32_to_two_delta}, vm::VM};

pub fn run_deopt(vm: &mut VM) -> Result<InterpreterResult, String> {
    let mut stdout = stdout().lock();
    let mut stdin = stdin().lock();
    let mut stdin_buf: [u8; 1] = [0];
    let mut mul_val: u8 = 0;
    
    loop {
        #[cfg(feature = "debug")] {
            vm.ocm.0[vm.pc] += 1;
        }

        let bytecode = &vm.insts[vm.pc];
        
        match bytecode.opcode {
            OpCode::Breakpoint => {
                vm.pointer += bytecode.delta as isize as usize;
                eprintln!("PC: {}, PTR: {}", vm.pc, vm.pointer);
            }

            OpCode::SingleAdd => {
                vm.pointer += bytecode.delta as isize as usize;
                vm.memory.add(vm.pointer, bytecode.val)?;
            }
            OpCode::SingleSet => {
                vm.pointer += bytecode.delta as isize as usize;
                vm.memory.set(vm.pointer, bytecode.val)?;
            }
            OpCode::AddAdd => {
                vm.pointer += bytecode.delta as isize as usize;
                vm.memory.add(vm.pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                vm.pointer += delta as isize as usize;
                vm.memory.add(vm.pointer, val)?;
            }
            OpCode::AddSet => {
                vm.pointer += bytecode.delta as isize as usize;
                vm.memory.add(vm.pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                vm.pointer += delta as isize as usize;
                vm.memory.set(vm.pointer, val)?;
            }
            OpCode::SetAdd => {
                vm.pointer += bytecode.delta as isize as usize;
                vm.memory.set(vm.pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                vm.pointer += delta as isize as usize;
                vm.memory.add(vm.pointer, val)?;
            }
            OpCode::SetSet => {
                vm.pointer += bytecode.delta as isize as usize;
                vm.memory.set(vm.pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                vm.pointer += delta as isize as usize;
                vm.memory.set(vm.pointer, val)?;
            }

            OpCode::ShiftP => {
                vm.pointer += bytecode.delta as isize as usize;
                let step = bytecode.addr as i32 as isize as usize;
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.pointer += step;
                }
                if positive_out_of_range(bytecode.val, vm.pointer) {
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
            }
            OpCode::ShiftN => {
                vm.pointer += bytecode.delta as isize as usize;
                let step = bytecode.addr as i32 as isize as usize;
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.pointer += step;
                }
                if negative_out_of_range(bytecode.val, vm.pointer) {
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
            }
            OpCode::ShiftAddP => {
                vm.pointer += bytecode.delta as isize as usize;
                let step = bytecode.val as i8 as isize as usize;
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.pointer += step;
                }
                let (delta, val, val2) = u32_to_delta_and_two_val(bytecode.addr);
                if !positive_out_of_range(val2, vm.pointer) {
                    vm.pointer += delta as isize as usize;
                    vm.memory.add(vm.pointer, val)?;
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                vm.pointer += delta as isize as usize;
                vm.memory.add(vm.pointer, val)?;
            }
            OpCode::ShiftAddN => {
                vm.pointer += bytecode.delta as isize as usize;
                let step = bytecode.val as i8 as isize as usize;
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.pointer += step;
                }
                let (delta, val, val2) = u32_to_delta_and_two_val(bytecode.addr);
                if !negative_out_of_range(val2, vm.pointer) {
                    vm.pointer += delta as isize as usize;
                    vm.memory.add(vm.pointer, val)?;
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                vm.pointer += delta as isize as usize;
                vm.memory.add(vm.pointer, val)?;
            }
            OpCode::ShiftSetP => {
                vm.pointer += bytecode.delta as isize as usize;
                let step = bytecode.val as i8 as isize as usize;
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.pointer += step;
                }
                let (delta, val, val2) = u32_to_delta_and_two_val(bytecode.addr);
                if !positive_out_of_range(val2, vm.pointer) {
                    vm.pointer += delta as isize as usize;
                    vm.memory.set(vm.pointer, val)?;
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                vm.pointer += delta as isize as usize;
                vm.memory.set(vm.pointer, val)?;
            }
            OpCode::ShiftSetN => {
                vm.pointer += bytecode.delta as isize as usize;
                let step = bytecode.val as i8 as isize as usize;
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.pointer += step;
                }
                let (delta, val, val2) = u32_to_delta_and_two_val(bytecode.addr);
                if !negative_out_of_range(val2, vm.pointer) {
                    vm.pointer += delta as isize as usize;
                    vm.memory.set(vm.pointer, val)?;
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
                vm.pointer += delta as isize as usize;
                vm.memory.set(vm.pointer, val)?;
            }

            OpCode::MulStart => {
                vm.pointer += bytecode.delta as isize as usize;
                let val = vm.memory.get(vm.pointer)?;
                if val == 0 {
                    vm.pc = bytecode.addr as usize;
                    continue;
                } else {
                    mul_val = val;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }
            OpCode::Mul => {
                vm.memory.add(vm.pointer + bytecode.delta as isize as usize, mul_val.wrapping_mul(bytecode.val))?;
            }

            OpCode::SingleMoveAdd => {
                vm.pointer += bytecode.delta as isize as usize;
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    vm.memory.set(vm.pointer, 0)?;
                    vm.memory.add(vm.pointer + (bytecode.addr as i32 as isize as usize), v)?;
                }
            }
            OpCode::SingleMoveSub => {
                vm.pointer += bytecode.delta as isize as usize;
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    vm.memory.set(vm.pointer, 0)?;
                    vm.memory.sub(vm.pointer + (bytecode.addr as i32 as isize as usize), v)?;
                }
            }

            OpCode::DoubleMoveAddAdd => {
                vm.pointer += bytecode.delta as isize as usize;
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    let (d1, d2) = u32_to_two_delta(bytecode.addr);
                    vm.memory.add(vm.pointer + d1 as isize as usize, v)?;
                    vm.memory.add(vm.pointer + d2 as isize as usize, v)?;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }
            OpCode::DoubleMoveAddSub => {
                vm.pointer += bytecode.delta as isize as usize;
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    let (d1, d2) = u32_to_two_delta(bytecode.addr);
                    vm.memory.add(vm.pointer + d1 as isize as usize, v)?;
                    vm.memory.sub(vm.pointer + d2 as isize as usize, v)?;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }
            OpCode::DoubleMoveSubAdd => {
                vm.pointer += bytecode.delta as isize as usize;
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    let (d1, d2) = u32_to_two_delta(bytecode.addr);
                    vm.memory.sub(vm.pointer + d1 as isize as usize, v)?;
                    vm.memory.add(vm.pointer + d2 as isize as usize, v)?;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }
            OpCode::DoubleMoveSubSub => {
                vm.pointer += bytecode.delta as isize as usize;
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    let (d1, d2) = u32_to_two_delta(bytecode.addr);
                    vm.memory.sub(vm.pointer + d1 as isize as usize, v)?;
                    vm.memory.sub(vm.pointer + d2 as isize as usize, v)?;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }

            OpCode::MoveStart => {
                vm.pointer += bytecode.delta as isize as usize;
                let val = vm.memory.get(vm.pointer)?;
                if val == 0 {
                    vm.pc = bytecode.addr as usize;
                    continue;
                } else {
                    mul_val = val;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }
            OpCode::MoveAdd => {
                vm.memory.add(vm.pointer + bytecode.delta as isize as usize, mul_val)?;
            }
            OpCode::MoveSub => {
                vm.memory.sub(vm.pointer + bytecode.delta as isize as usize, mul_val)?;
            }

            OpCode::In => {
                vm.pointer += bytecode.delta as isize as usize;
                match stdin.read_exact(&mut stdin_buf) {
                    Ok(_) => vm.memory.set(vm.pointer, stdin_buf[0])?,
                    Err(_) => vm.memory.set(vm.pointer, 0)?,
                }
            }
            OpCode::Out => {
                vm.pointer += bytecode.delta as isize as usize;
                stdout.write(&[vm.memory.get(vm.pointer)?]).map_err(|_| "Runtime Error: Failed to print")?;
            }

            OpCode::JmpIfZero => {
                vm.pointer += bytecode.delta as isize as usize;
                if vm.memory.get(vm.pointer)? == 0 {
                    vm.pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode::JmpIfNotZero => {
                vm.pointer += bytecode.delta as isize as usize;
                if vm.memory.get(vm.pointer)? != 0 {
                    vm.pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode::PositiveRangeCheckJNZ => {
                vm.pointer += bytecode.delta as isize as usize;
                if vm.memory.get(vm.pointer)? != 0 {
                    vm.pc = bytecode.addr as usize;
                    continue;
                }
                if !positive_out_of_range(bytecode.val, vm.pointer) {
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
            }
            OpCode::NegativeRangeCheckJNZ => {
                vm.pointer += bytecode.delta as isize as usize;
                if vm.memory.get(vm.pointer)? != 0 {
                    vm.pc = bytecode.addr as usize;
                    continue;
                }
                if !negative_out_of_range(bytecode.val, vm.pointer) {
                    return Ok(InterpreterResult::ToggleTier(Tier::Opt));
                }
            }

            OpCode::End => {
                return Ok(InterpreterResult::End);
            }
        }
        vm.pc += 1;
    }
}

use std::io::{Read, Write, stdin, stdout};

use crate::cisc::{bytecode::OpCode, internal::{InterpreterResult, Tier, negative_out_of_range, positive_out_of_range, u32_to_delta_and_two_val, u32_to_delta_and_val, u32_to_two_delta}, vm::VM};

pub unsafe fn run_opt(vm: &mut VM) -> Result<InterpreterResult, String> {
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
        println!("[TRACE] tier: Opt ptr: {}, executing {}({:?})", vm.pointer, vm.pc, bytecode.opcode);
        
        match bytecode.opcode {
            OpCode::Breakpoint => {
                vm.step_ptr(bytecode.delta as isize);
                eprintln!("PC: {}, PTR: {}", vm.pc, vm.pointer);
            }

            OpCode::SingleAdd => {
                vm.step_ptr(bytecode.delta as isize);
                vm.memory.add_unchecked(vm.pointer, bytecode.val);
            }
            OpCode::SingleSet => {
                vm.step_ptr(bytecode.delta as isize);
                vm.memory.set_unchecked(vm.pointer, bytecode.val);
            }
            OpCode::AddAdd => {
                vm.step_ptr(bytecode.delta as isize);
                vm.memory.add_unchecked(vm.pointer, bytecode.val);
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                vm.step_ptr(delta as isize);
                vm.memory.add_unchecked(vm.pointer, val);
            }
            OpCode::AddSet => {
                vm.step_ptr(bytecode.delta as isize);
                vm.memory.add_unchecked(vm.pointer, bytecode.val);
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                vm.step_ptr(delta as isize);
                vm.memory.set_unchecked(vm.pointer, val);
            }
            OpCode::SetAdd => {
                vm.step_ptr(bytecode.delta as isize);
                vm.memory.set_unchecked(vm.pointer, bytecode.val);
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                vm.step_ptr(delta as isize);
                vm.memory.add_unchecked(vm.pointer, val);
            }
            OpCode::SetSet => {
                vm.step_ptr(bytecode.delta as isize);
                vm.memory.set_unchecked(vm.pointer, bytecode.val);
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                vm.step_ptr(delta as isize);
                vm.memory.set_unchecked(vm.pointer, val);
            }

            OpCode::ShiftP => {
                vm.step_ptr(bytecode.delta as isize);
                let step = bytecode.addr as i32 as isize;
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr(step);
                }
                if positive_out_of_range(bytecode.val, vm.pointer) {
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
            }
            OpCode::ShiftN => {
                vm.step_ptr(bytecode.delta as isize);
                let step = bytecode.addr as i32 as isize;
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr(step);
                }
                if negative_out_of_range(bytecode.val, vm.pointer) {
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
            }
            OpCode::ShiftAddP => {
                vm.step_ptr(bytecode.delta as isize);
                let step = bytecode.val as i8 as isize;
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr(step);
                }
                let (delta, val, val2) = u32_to_delta_and_two_val(bytecode.addr);
                if positive_out_of_range(val2, vm.pointer) {
                    vm.step_ptr(delta as isize);
                    vm.memory.add(vm.pointer, val)?;
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                vm.step_ptr(delta as isize);
                vm.memory.add_unchecked(vm.pointer, val);
            }
            OpCode::ShiftAddN => {
                vm.step_ptr(bytecode.delta as isize);
                let step = bytecode.val as i8 as isize;
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr(step);
                }
                let (delta, val, val2) = u32_to_delta_and_two_val(bytecode.addr);
                if !negative_out_of_range(val2, vm.pointer) {
                    vm.step_ptr(delta as isize);
                    vm.memory.add(vm.pointer, val)?;
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                vm.step_ptr(delta as isize);
                vm.memory.add_unchecked(vm.pointer, val);
            }
            OpCode::ShiftSetP => {
                vm.step_ptr(bytecode.delta as isize);
                let step = bytecode.val as i8 as isize;
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr(step);
                }
                let (delta, val, val2) = u32_to_delta_and_two_val(bytecode.addr);
                if !positive_out_of_range(val2, vm.pointer) {
                    vm.step_ptr(delta as isize);
                    vm.memory.set(vm.pointer, val)?;
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                vm.step_ptr(delta as isize);
                vm.memory.set_unchecked(vm.pointer, val);
            }
            OpCode::ShiftSetN => {
                vm.step_ptr(bytecode.delta as isize);
                let step = bytecode.val as i8 as isize;
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.step_ptr(step);
                }
                let (delta, val, val2) = u32_to_delta_and_two_val(bytecode.addr);
                if negative_out_of_range(val2, vm.pointer) {
                    vm.step_ptr(delta as isize);
                    vm.memory.set(vm.pointer, val)?;
                    vm.pc += 1;
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                vm.step_ptr(delta as isize);
                vm.memory.set_unchecked(vm.pointer, val);
            }

            OpCode::MulStart => {
                vm.step_ptr(bytecode.delta as isize);
                let val = vm.memory.get_unchecked(vm.pointer);
                if val == 0 {
                    vm.pc = bytecode.addr as usize;
                    continue;
                } else {
                    mul_val = val;
                    vm.memory.set_unchecked(vm.pointer, 0);
                }
            }
            OpCode::Mul => {
                vm.memory.add_unchecked(vm.pointer.wrapping_add_signed(bytecode.delta as isize), mul_val.wrapping_mul(bytecode.val));
            }

            OpCode::SingleMoveAdd => {
                vm.step_ptr(bytecode.delta as isize);
                vm.memory.add_unchecked(vm.pointer.wrapping_add_signed(bytecode.addr as i32 as isize), vm.memory.get_unchecked(vm.pointer));
                vm.memory.set_unchecked(vm.pointer, 0);
            }
            OpCode::SingleMoveSub => {
                vm.step_ptr(bytecode.delta as isize);
                vm.memory.sub_unchecked(vm.pointer.wrapping_add_signed(bytecode.addr as i32 as isize), vm.memory.get_unchecked(vm.pointer));
                vm.memory.set_unchecked(vm.pointer, 0);
            }

            OpCode::DoubleMoveAddAdd => {
                vm.step_ptr(bytecode.delta as isize);
                let v = vm.memory.get_unchecked(vm.pointer);
                let (d1, d2) = u32_to_two_delta(bytecode.addr);
                vm.memory.add_unchecked(vm.pointer.wrapping_add_signed(d1 as isize), v);
                vm.memory.add_unchecked(vm.pointer.wrapping_add_signed(d2 as isize), v);
                vm.memory.set_unchecked(vm.pointer, 0);
            }
            OpCode::DoubleMoveAddSub => {
                vm.step_ptr(bytecode.delta as isize);
                let v = vm.memory.get_unchecked(vm.pointer);
                let (d1, d2) = u32_to_two_delta(bytecode.addr);
                vm.memory.add_unchecked(vm.pointer.wrapping_add_signed(d1 as isize), v);
                vm.memory.sub_unchecked(vm.pointer.wrapping_add_signed(d2 as isize), v);
                vm.memory.set_unchecked(vm.pointer, 0);
            }
            OpCode::DoubleMoveSubAdd => {
                vm.step_ptr(bytecode.delta as isize);
                let v = vm.memory.get_unchecked(vm.pointer);
                let (d1, d2) = u32_to_two_delta(bytecode.addr);
                vm.memory.sub_unchecked(vm.pointer.wrapping_add_signed(d1 as isize), v);
                vm.memory.add_unchecked(vm.pointer.wrapping_add_signed(d2 as isize), v);
                vm.memory.set_unchecked(vm.pointer, 0);
            }
            OpCode::DoubleMoveSubSub => {
                vm.step_ptr(bytecode.delta as isize);
                let v = vm.memory.get_unchecked(vm.pointer);
                let (d1, d2) = u32_to_two_delta(bytecode.addr);
                vm.memory.sub_unchecked(vm.pointer.wrapping_add_signed(d1 as isize), v);
                vm.memory.sub_unchecked(vm.pointer.wrapping_add_signed(d2 as isize), v);
                vm.memory.set_unchecked(vm.pointer, 0);
            }

            OpCode::MoveStart => {
                vm.step_ptr(bytecode.delta as isize);
                let val = vm.memory.get_unchecked(vm.pointer);
                if val == 0 {
                    vm.pc = bytecode.addr as usize;
                    continue;
                } else {
                    mul_val = val;
                    vm.memory.set_unchecked(vm.pointer, 0);
                }
            }
            OpCode::MoveAdd => {
                vm.memory.add_unchecked(vm.pointer.wrapping_add_signed(bytecode.delta as isize), mul_val);
            }
            OpCode::MoveSub => {
                vm.memory.sub_unchecked(vm.pointer.wrapping_add_signed(bytecode.delta as isize), mul_val);
            }

            OpCode::In => {
                vm.step_ptr(bytecode.delta as isize);
                match stdin.read_exact(&mut stdin_buf) {
                    Ok(_) => vm.memory.set_unchecked(vm.pointer, stdin_buf[0]),
                    Err(_) => vm.memory.set_unchecked(vm.pointer, 0),
                }
            }
            OpCode::Out => {
                vm.step_ptr(bytecode.delta as isize);
                stdout.write(&[vm.memory.get_unchecked(vm.pointer)]).map_err(|_| "Runtime Error: Failed to print")?;
            }

            OpCode::JmpIfZero => {
                vm.step_ptr(bytecode.delta as isize);
                if vm.memory.get_unchecked(vm.pointer) == 0 {
                    vm.pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode::JmpIfNotZero => {
                vm.step_ptr(bytecode.delta as isize);
                if vm.memory.get_unchecked(vm.pointer) != 0 {
                    vm.pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode::PositiveRangeCheckJNZ => {
                vm.step_ptr(bytecode.delta as isize);
                if positive_out_of_range(bytecode.val, vm.pointer) {
                    if vm.memory.get(vm.pointer)? != 0 {
                        vm.pc = bytecode.addr as usize;
                    } else {
                        vm.pc += 1;
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                if vm.memory.get_unchecked(vm.pointer) != 0 {
                    vm.pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode::NegativeRangeCheckJNZ => {
                vm.step_ptr(bytecode.delta as isize);
                if negative_out_of_range(bytecode.val, vm.pointer) {
                    if vm.memory.get(vm.pointer)? != 0 {
                        vm.pc = bytecode.addr as usize;
                    } else {
                        vm.pc += 1;
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                if vm.memory.get_unchecked(vm.pointer) != 0 {
                    vm.pc = bytecode.addr as usize;
                    continue;
                }
            }

            OpCode::End => {
                return Ok(InterpreterResult::End);
            }
        }
        vm.pc += 1;
    }
}

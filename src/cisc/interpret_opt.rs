use std::io::{Read, Write, stdin, stdout};

use crate::cisc::{bytecode::NewBytecode, internal::{InterpreterResult, Tier}, vm::UnsafeVM};
use crate::range::{negative_is_out_of_range, positive_is_out_of_range};

pub unsafe fn run_opt(vm: &mut UnsafeVM) -> Result<InterpreterResult, String> {
    let mut stdout = stdout().lock();
    let mut stdin = stdin().lock();
    let mut stdin_buf: [u8; 1] = [0];
    let mut mul_val: u8 = 0;
    
    loop {
        #[cfg(feature = "debug")] {
            let pc = vm.get_pc();
            vm.inner.ocm.opt[pc] += 1;
        }

        #[cfg(feature = "trace")]
        println!("[TRACE] tier: Opt ptr: {}, executing {}", vm.get_ptr(), vm.get_pc());
        
        match vm.get_op() {
            NewBytecode::Breakpoint { delta } => {
                vm.step_ptr(delta as isize);
                eprintln!("PC: {}, PTR: {}", vm.get_pc(), vm.get_ptr());
            }

            NewBytecode::SingleAdd { delta, val } => {
                vm.step_ptr(delta as isize);
                vm.add(val);
            }
            NewBytecode::SingleSet { delta, val } => {
                vm.step_ptr(delta as isize);
                vm.set(val);
            }
            NewBytecode::AddAdd { delta1, val1, delta2, val2 } => {
                vm.step_ptr(delta1 as isize);
                vm.add(val1);
                vm.step_ptr(delta2 as isize);
                vm.add(val2);
            }
            NewBytecode::AddSet { delta1, val1, delta2, val2 } => {
                vm.step_ptr(delta1 as isize);
                vm.add(val1);
                vm.step_ptr(delta2 as isize);
                vm.set(val2);
            }
            NewBytecode::SetAdd { delta1, val1, delta2, val2 } => {
                vm.step_ptr(delta1 as isize);
                vm.set(val1);
                vm.step_ptr(delta2 as isize);
                vm.add(val2);
            }
            NewBytecode::SetSet { delta1, val1, delta2, val2 } => {
                vm.step_ptr(delta1 as isize);
                vm.set(val1);
                vm.step_ptr(delta2 as isize);
                vm.set(val2);
            }

            NewBytecode::BothRangeCheck { positive, negative } => {
                let ptr = vm.get_ptr();
                if positive_is_out_of_range(positive, ptr) || negative_is_out_of_range(negative, ptr) {
                    vm.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
            }
            NewBytecode::Shift { delta, step } => {
                vm.step_ptr(delta as isize);
                while vm.inner.memory.get(vm.get_ptr())? != 0 {
                    vm.step_ptr(step as isize);
                }
            }
            NewBytecode::ShiftP { delta, step, range } => {
                vm.step_ptr(delta as isize);
                while vm.inner.memory.get(vm.get_ptr())? != 0 {
                    vm.step_ptr(step as isize);
                }
                if positive_is_out_of_range(range, vm.get_ptr()) {
                    vm.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
            }
            NewBytecode::ShiftN { delta, step, range } => {
                vm.step_ptr(delta as isize);
                while vm.inner.memory.get(vm.get_ptr())? != 0 {
                    vm.step_ptr(step as isize);
                }
                if negative_is_out_of_range(range, vm.get_ptr()) {
                    vm.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
            }
            NewBytecode::ShiftAdd { delta1, step, delta2, val } => {
                vm.step_ptr(delta1 as isize);
                while vm.inner.memory.get(vm.get_ptr())? != 0 {
                    vm.step_ptr(step as isize);
                }
                vm.step_ptr(delta2 as isize);
                vm.add(val);
            }
            NewBytecode::ShiftAddP { delta1, step, delta2, val, range } => {
                vm.step_ptr(delta1 as isize);
                while vm.inner.memory.get(vm.get_ptr())? != 0 {
                    vm.step_ptr(step as isize);
                }
                if positive_is_out_of_range(range, vm.get_ptr()) {
                    vm.step_ptr(delta2 as isize);
                    vm.inner.memory.add(vm.get_ptr(), val)?;
                    vm.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                vm.step_ptr(delta2 as isize);
                vm.add(val);
            }
            NewBytecode::ShiftAddN { delta1, step, delta2, val, range } => {
                vm.step_ptr(delta1 as isize);
                while vm.inner.memory.get(vm.get_ptr())? != 0 {
                    vm.step_ptr(step as isize);
                }
                if negative_is_out_of_range(range, vm.get_ptr()) {
                    vm.step_ptr(delta2 as isize);
                    vm.inner.memory.add(vm.get_ptr(), val)?;
                    vm.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                vm.step_ptr(delta2 as isize);
                vm.add(val);
            }
            NewBytecode::ShiftSet { delta1, step, delta2, val } => {
                vm.step_ptr(delta1 as isize);
                while vm.inner.memory.get(vm.get_ptr())? != 0 {
                    vm.step_ptr(step as isize);
                }
                vm.step_ptr(delta2 as isize);
                vm.set(val);
            }
            NewBytecode::ShiftSetP { delta1, step, delta2, val, range } => {
                vm.step_ptr(delta1 as isize);
                while vm.inner.memory.get(vm.get_ptr())? != 0 {
                    vm.step_ptr(step as isize);
                }
                if positive_is_out_of_range(range, vm.get_ptr()) {
                    vm.step_ptr(delta2 as isize);
                    vm.inner.memory.set(vm.get_ptr(), val)?;
                    vm.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                vm.step_ptr(delta2 as isize);
                vm.set(val);
            }
            NewBytecode::ShiftSetN { delta1, step, delta2, val, range } => {
                vm.step_ptr(delta1 as isize);
                while vm.inner.memory.get(vm.get_ptr())? != 0 {
                    vm.step_ptr(step as isize);
                }
                if negative_is_out_of_range(range, vm.get_ptr()) {
                    vm.step_ptr(delta2 as isize);
                    vm.inner.memory.set(vm.get_ptr(), val)?;
                    vm.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                vm.step_ptr(delta2 as isize);
                vm.set(val);
            }

            NewBytecode::MulStart { delta, jz_abs } => {
                vm.step_ptr(delta as isize);
                let val = vm.get();
                if val == 0 {
                    vm.jump_abs(jz_abs);
                    continue;
                } else {
                    mul_val = val;
                    vm.set(0);
                }
            }
            NewBytecode::Mul { delta, val } => {
                vm.add_with_offset(delta as isize, mul_val.wrapping_mul(val));
            }

            NewBytecode::SingleMoveAdd { delta, to } => {
                vm.step_ptr(delta as isize);
                vm.add_with_offset(to as isize, vm.get());
                vm.set(0);
            }
            NewBytecode::SingleMoveSub { delta, to } => {
                vm.step_ptr(delta as isize);
                vm.sub_with_offset(to as isize, vm.get());
                vm.set(0);
            }

            NewBytecode::DoubleMoveAddAdd { delta, to1, to2 } => {
                vm.step_ptr(delta as isize);
                let v = vm.get();
                vm.add_with_offset(to1 as isize, v);
                vm.add_with_offset(to2 as isize, v);
                vm.set(0);
            }
            NewBytecode::DoubleMoveAddSub { delta, to1, to2 } => {
                vm.step_ptr(delta as isize);
                let v = vm.get();
                vm.add_with_offset(to1 as isize, v);
                vm.sub_with_offset(to2 as isize, v);
                vm.set(0);
            }
            NewBytecode::DoubleMoveSubAdd { delta, to1, to2 } => {
                vm.step_ptr(delta as isize);
                let v = vm.get();
                vm.sub_with_offset(to1 as isize, v);
                vm.add_with_offset(to2 as isize, v);
                vm.set(0);
            }
            NewBytecode::DoubleMoveSubSub { delta, to1, to2 } => {
                vm.step_ptr(delta as isize);
                let v = vm.get();
                vm.sub_with_offset(to1 as isize, v);
                vm.sub_with_offset(to2 as isize, v);
                vm.set(0);
            }

            NewBytecode::MoveStart { delta, jz_abs } => {
                vm.step_ptr(delta as isize);
                let val = vm.get();
                if val == 0 {
                    vm.jump_abs(jz_abs);
                    continue;
                } else {
                    mul_val = val;
                    vm.set(0);
                }
            }
            NewBytecode::MoveAdd { delta } => {
                vm.add_with_offset(delta as isize, mul_val);
            }
            NewBytecode::MoveSub { delta } => {
                vm.sub_with_offset(delta as isize, mul_val);
            }

            NewBytecode::In { delta } => {
                vm.step_ptr(delta as isize);
                match stdin.read_exact(&mut stdin_buf) {
                    Ok(_) => vm.set(stdin_buf[0]),
                    Err(_) => vm.set(0),
                }
            }
            NewBytecode::Out { delta } => {
                vm.step_ptr(delta as isize);
                stdout.write(&[vm.get()]).map_err(|_| "Runtime Error: Failed to print")?;
            }

            NewBytecode::JmpIfZero { delta, addr_abs } => {
                vm.step_ptr(delta as isize);
                if vm.get() == 0 {
                    vm.jump_abs(addr_abs);
                    continue;
                }
            }
            NewBytecode::JmpIfNotZero { delta, addr_abs } => {
                vm.step_ptr(delta as isize);
                if vm.get() != 0 {
                    vm.jump_abs(addr_abs);
                    continue;
                }
            }
            NewBytecode::PositiveRangeCheckJNZ { delta, addr_back, range } => {
                vm.step_ptr(delta as isize);
                if positive_is_out_of_range(range, vm.get_ptr()) {
                    if vm.inner.memory.get(vm.get_ptr())? != 0 {
                        vm.jump_back(addr_back);
                    } else {
                        vm.jump_one();
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                if vm.get() != 0 {
                    vm.jump_back(addr_back);
                    continue;
                }
            }
            NewBytecode::NegativeRangeCheckJNZ { delta, addr_back, range } => {
                vm.step_ptr(delta as isize);
                if negative_is_out_of_range(range, vm.get_ptr()) {
                    if vm.inner.memory.get(vm.get_ptr())? != 0 {
                        vm.jump_back(addr_back);
                    } else {
                        vm.jump_one();
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                if vm.get() != 0 {
                    vm.jump_back(addr_back);
                    continue;
                }
            }
            NewBytecode::BothRangeCheckJNZ { delta, addr_back, positive, negative } => {
                vm.step_ptr(delta as isize);
                let ptr = vm.get_ptr();
                if positive_is_out_of_range(positive, ptr) || negative_is_out_of_range(negative, ptr) {
                    if vm.inner.memory.get(vm.get_ptr())? != 0 {
                        vm.jump_back(addr_back);
                    } else {
                        vm.jump_one();
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                if vm.get() != 0 {
                    vm.jump_back(addr_back);
                    continue;
                }
            }

            NewBytecode::End { delta } => {
                vm.step_ptr(delta as isize);
                return Ok(InterpreterResult::End);
            }
        }
        vm.jump_one();
    }
}

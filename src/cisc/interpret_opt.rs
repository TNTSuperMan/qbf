use std::io::{Read, Write, stdin, stdout};

use crate::cisc::{bytecode::Bytecode, internal::{InterpreterResult, Tier}, vm::UnsafeVM};

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
            Bytecode::Breakpoint { delta } => {
                vm.step_ptr((*delta) as isize);
                eprintln!("PC: {}, PTR: {}", vm.get_pc(), vm.get_ptr());
            }

            Bytecode::SingleAdd { delta, val } => {
                vm.step_ptr((*delta) as isize);
                vm.add(*val);
            }
            Bytecode::SingleSet { delta, val } => {
                vm.step_ptr((*delta) as isize);
                vm.set(*val);
            }
            Bytecode::AddAdd { delta1, val1, delta2, val2 } => {
                vm.step_ptr((*delta1) as isize);
                vm.add(*val1);
                vm.step_ptr((*delta2) as isize);
                vm.add(*val2);
            }
            Bytecode::AddSet { delta1, val1, delta2, val2 } => {
                vm.step_ptr((*delta1) as isize);
                vm.add(*val1);
                vm.step_ptr((*delta2) as isize);
                vm.set(*val2);
            }
            Bytecode::SetAdd { delta1, val1, delta2, val2 } => {
                vm.step_ptr((*delta1) as isize);
                vm.set(*val1);
                vm.step_ptr((*delta2) as isize);
                vm.add(*val2);
            }
            Bytecode::SetSet { delta1, val1, delta2, val2 } => {
                vm.step_ptr((*delta1) as isize);
                vm.set(*val1);
                vm.step_ptr((*delta2) as isize);
                vm.set(*val2);
            }

            Bytecode::BothRangeCheck { range } => {
                if !range.contains(&(vm.get_ptr() as u16)) {
                    vm.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
            }
            Bytecode::Shift { delta, step } => {
                vm.step_ptr((*delta) as isize);
                while vm.inner.memory.get(vm.get_ptr())? != 0 {
                    vm.step_ptr((*step) as isize);
                }
            }
            Bytecode::ShiftP { delta, step, range } => {
                vm.step_ptr((*delta) as isize);
                while vm.inner.memory.get(vm.get_ptr())? != 0 {
                    vm.step_ptr((*step) as isize);
                }
                if !range.contains(&(vm.get_ptr() as u16)) {
                    vm.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
            }
            Bytecode::ShiftN { delta, step, range } => {
                vm.step_ptr((*delta) as isize);
                while vm.inner.memory.get(vm.get_ptr())? != 0 {
                    vm.step_ptr((*step) as isize);
                }
                if !range.contains(&(vm.get_ptr() as u16)) {
                    vm.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
            }
            Bytecode::ShiftAdd { delta1, step, delta2, val } => {
                vm.step_ptr((*delta1) as isize);
                while vm.inner.memory.get(vm.get_ptr())? != 0 {
                    vm.step_ptr((*step) as isize);
                }
                vm.step_ptr((*delta2) as isize);
                vm.add(*val);
            }
            Bytecode::ShiftAddP { delta1, step, delta2, val, range } => {
                vm.step_ptr((*delta1) as isize);
                while vm.inner.memory.get(vm.get_ptr())? != 0 {
                    vm.step_ptr((*step) as isize);
                }
                if !range.contains(&(vm.get_ptr() as u16)) {
                    vm.step_ptr((*delta2) as isize);
                    vm.inner.memory.add(vm.get_ptr(), *val)?;
                    vm.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                vm.step_ptr((*delta2) as isize);
                vm.add(*val);
            }
            Bytecode::ShiftAddN { delta1, step, delta2, val, range } => {
                vm.step_ptr((*delta1) as isize);
                while vm.inner.memory.get(vm.get_ptr())? != 0 {
                    vm.step_ptr((*step) as isize);
                }
                if !range.contains(&(vm.get_ptr() as u16)) {
                    vm.step_ptr((*delta2) as isize);
                    vm.inner.memory.add(vm.get_ptr(), *val)?;
                    vm.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                vm.step_ptr((*delta2) as isize);
                vm.add(*val);
            }
            Bytecode::ShiftSet { delta1, step, delta2, val } => {
                vm.step_ptr((*delta1) as isize);
                while vm.inner.memory.get(vm.get_ptr())? != 0 {
                    vm.step_ptr((*step) as isize);
                }
                vm.step_ptr((*delta2) as isize);
                vm.set(*val);
            }
            Bytecode::ShiftSetP { delta1, step, delta2, val, range } => {
                vm.step_ptr((*delta1) as isize);
                while vm.inner.memory.get(vm.get_ptr())? != 0 {
                    vm.step_ptr((*step) as isize);
                }
                if !range.contains(&(vm.get_ptr() as u16)) {
                    vm.step_ptr((*delta2) as isize);
                    vm.inner.memory.set(vm.get_ptr(), *val)?;
                    vm.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                vm.step_ptr((*delta2) as isize);
                vm.set(*val);
            }
            Bytecode::ShiftSetN { delta1, step, delta2, val, range } => {
                vm.step_ptr((*delta1) as isize);
                while vm.inner.memory.get(vm.get_ptr())? != 0 {
                    vm.step_ptr((*step) as isize);
                }
                if !range.contains(&(vm.get_ptr() as u16)) {
                    vm.step_ptr((*delta2) as isize);
                    vm.inner.memory.set(vm.get_ptr(), *val)?;
                    vm.jump_one();
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                vm.step_ptr((*delta2) as isize);
                vm.set(*val);
            }

            Bytecode::MulStart { delta, jz_abs } => {
                vm.step_ptr((*delta) as isize);
                let val = vm.get();
                if val == 0 {
                    vm.jump_abs(*jz_abs);
                    continue;
                } else {
                    mul_val = val;
                    vm.set(0);
                }
            }
            Bytecode::Mul { delta, val } => {
                vm.add_with_offset((*delta) as isize, mul_val.wrapping_mul(*val));
            }

            Bytecode::SingleMoveAdd { delta, to } => {
                vm.step_ptr((*delta) as isize);
                vm.add_with_offset((*to) as isize, vm.get());
                vm.set(0);
            }
            Bytecode::SingleMoveSub { delta, to } => {
                vm.step_ptr((*delta) as isize);
                vm.sub_with_offset((*to) as isize, vm.get());
                vm.set(0);
            }

            Bytecode::DoubleMoveAddAdd { delta, to1, to2 } => {
                vm.step_ptr((*delta) as isize);
                let v = vm.get();
                vm.add_with_offset((*to1) as isize, v);
                vm.add_with_offset((*to2) as isize, v);
                vm.set(0);
            }
            Bytecode::DoubleMoveAddSub { delta, to1, to2 } => {
                vm.step_ptr((*delta) as isize);
                let v = vm.get();
                vm.add_with_offset((*to1) as isize, v);
                vm.sub_with_offset((*to2) as isize, v);
                vm.set(0);
            }
            Bytecode::DoubleMoveSubAdd { delta, to1, to2 } => {
                vm.step_ptr((*delta) as isize);
                let v = vm.get();
                vm.sub_with_offset((*to1) as isize, v);
                vm.add_with_offset((*to2) as isize, v);
                vm.set(0);
            }
            Bytecode::DoubleMoveSubSub { delta, to1, to2 } => {
                vm.step_ptr((*delta) as isize);
                let v = vm.get();
                vm.sub_with_offset((*to1) as isize, v);
                vm.sub_with_offset((*to2) as isize, v);
                vm.set(0);
            }

            Bytecode::MoveStart { delta, jz_abs } => {
                vm.step_ptr((*delta) as isize);
                let val = vm.get();
                if val == 0 {
                    vm.jump_abs(*jz_abs);
                    continue;
                } else {
                    mul_val = val;
                    vm.set(0);
                }
            }
            Bytecode::MoveAdd { delta } => {
                vm.add_with_offset((*delta) as isize, mul_val);
            }
            Bytecode::MoveSub { delta } => {
                vm.sub_with_offset((*delta) as isize, mul_val);
            }

            Bytecode::In { delta } => {
                vm.step_ptr((*delta) as isize);
                match stdin.read_exact(&mut stdin_buf) {
                    Ok(_) => vm.set(stdin_buf[0]),
                    Err(_) => vm.set(0),
                }
            }
            Bytecode::Out { delta } => {
                vm.step_ptr((*delta) as isize);
                stdout.write(&[vm.get()]).map_err(|_| "Runtime Error: Failed to print")?;
                if vm.inner.flush {
                    stdout.flush().map_err(|_| "Runtime Error: Failed to flush")?;
                }
            }

            Bytecode::JmpIfZero { delta, addr_abs } => {
                vm.step_ptr((*delta) as isize);
                if vm.get() == 0 {
                    vm.jump_abs(*addr_abs);
                    continue;
                }
            }
            Bytecode::JmpIfNotZero { delta, addr_abs } => {
                vm.step_ptr((*delta) as isize);
                if vm.get() != 0 {
                    vm.jump_abs(*addr_abs);
                    continue;
                }
            }
            Bytecode::PositiveRangeCheckJNZ { delta, addr_back, range } => {
                vm.step_ptr((*delta) as isize);
                if !range.contains(&(vm.get_ptr() as u16)) {
                    if vm.inner.memory.get(vm.get_ptr())? != 0 {
                        vm.jump_back(*addr_back);
                    } else {
                        vm.jump_one();
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                if vm.get() != 0 {
                    vm.jump_back(*addr_back);
                    continue;
                }
            }
            Bytecode::NegativeRangeCheckJNZ { delta, addr_back, range } => {
                vm.step_ptr((*delta) as isize);
                if !range.contains(&(vm.get_ptr() as u16)) {
                    if vm.inner.memory.get(vm.get_ptr())? != 0 {
                        vm.jump_back(*addr_back);
                    } else {
                        vm.jump_one();
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                if vm.get() != 0 {
                    vm.jump_back(*addr_back);
                    continue;
                }
            }
            Bytecode::BothRangeCheckJNZ { delta, addr_back, range } => {
                vm.step_ptr((*delta) as isize);
                let ptr = vm.get_ptr();
                if !range.contains(&(ptr as u16)) {
                    if vm.inner.memory.get(ptr)? != 0 {
                        vm.jump_back(*addr_back);
                    } else {
                        vm.jump_one();
                    }
                    return Ok(InterpreterResult::ToggleTier(Tier::Deopt));
                }
                if vm.get() != 0 {
                    vm.jump_back(*addr_back);
                    continue;
                }
            }

            Bytecode::End { delta } => {
                vm.step_ptr((*delta) as isize);
                return Ok(InterpreterResult::End);
            }
        }
        vm.jump_one();
    }
}

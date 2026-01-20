use std::io::{Read, Write, stdin, stdout};

use crate::cisc::{bytecode::OpCode, vm::VM};

#[inline(always)]
pub fn u32_to_delta_and_val(val: u32) -> (i16, u8) {
    (
        (val & 0xFFFF) as u16 as i16,
        (val >> 16) as u8,
    )
}

#[inline(always)]
pub fn u32_to_two_delta(val: u32) -> (i16, i16) {
    (
        (val & 0xFFFF) as u16 as i16,
        (val >> 16) as u16 as i16,
    )
}

pub fn run(vm: &mut VM) -> Result<(), String> {
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
                vm.pointer += bytecode.delta as isize;
                eprintln!("PC: {}, PTR: {}", vm.pc, vm.pointer);
            }

            OpCode::SingleAdd => {
                vm.pointer += bytecode.delta as isize;
                vm.memory.add(vm.pointer, bytecode.val)?;
            }
            OpCode::SingleSet => {
                vm.pointer += bytecode.delta as isize;
                vm.memory.set(vm.pointer, bytecode.val)?;
            }
            OpCode::AddAdd => {
                vm.pointer += bytecode.delta as isize;
                vm.memory.add(vm.pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                vm.pointer += delta as isize;
                vm.memory.add(vm.pointer, val)?;
            }
            OpCode::AddSet => {
                vm.pointer += bytecode.delta as isize;
                vm.memory.add(vm.pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                vm.pointer += delta as isize;
                vm.memory.set(vm.pointer, val)?;
            }
            OpCode::SetAdd => {
                vm.pointer += bytecode.delta as isize;
                vm.memory.set(vm.pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                vm.pointer += delta as isize;
                vm.memory.add(vm.pointer, val)?;
            }
            OpCode::SetSet => {
                vm.pointer += bytecode.delta as isize;
                vm.memory.set(vm.pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                vm.pointer += delta as isize;
                vm.memory.set(vm.pointer, val)?;
            }

            OpCode::Shift => {
                vm.pointer += bytecode.delta as isize;
                let step = bytecode.addr as i32 as isize;
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.pointer += step;
                }
            }
            OpCode::ShiftAdd => {
                vm.pointer += bytecode.delta as isize;
                let step = bytecode.val as i8 as isize;
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.pointer += step;
                }
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                vm.pointer += delta as isize;
                vm.memory.add(vm.pointer, val)?;
            }
            OpCode::ShiftSet => {
                vm.pointer += bytecode.delta as isize;
                let step = bytecode.val as i8 as isize;
                while vm.memory.get(vm.pointer)? != 0 {
                    vm.pointer += step;
                }
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                vm.pointer += delta as isize;
                vm.memory.set(vm.pointer, val)?;
            }

            OpCode::MulStart => {
                vm.pointer += bytecode.delta as isize;
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
                vm.memory.add(vm.pointer + bytecode.delta as isize, mul_val.wrapping_mul(bytecode.val))?;
            }

            OpCode::SingleMoveAdd => {
                vm.pointer += bytecode.delta as isize;
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    vm.memory.set(vm.pointer, 0)?;
                    vm.memory.add(vm.pointer + (bytecode.addr as i32 as isize), v)?;
                }
            }
            OpCode::SingleMoveSub => {
                vm.pointer += bytecode.delta as isize;
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    vm.memory.set(vm.pointer, 0)?;
                    vm.memory.sub(vm.pointer + (bytecode.addr as i32 as isize), v)?;
                }
            }

            OpCode::DoubleMoveAddAdd => {
                vm.pointer += bytecode.delta as isize;
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    let (d1, d2) = u32_to_two_delta(bytecode.addr);
                    vm.memory.add(vm.pointer + d1 as isize, v)?;
                    vm.memory.add(vm.pointer + d2 as isize, v)?;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }
            OpCode::DoubleMoveAddSub => {
                vm.pointer += bytecode.delta as isize;
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    let (d1, d2) = u32_to_two_delta(bytecode.addr);
                    vm.memory.add(vm.pointer + d1 as isize, v)?;
                    vm.memory.sub(vm.pointer + d2 as isize, v)?;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }
            OpCode::DoubleMoveSubAdd => {
                vm.pointer += bytecode.delta as isize;
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    let (d1, d2) = u32_to_two_delta(bytecode.addr);
                    vm.memory.sub(vm.pointer + d1 as isize, v)?;
                    vm.memory.add(vm.pointer + d2 as isize, v)?;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }
            OpCode::DoubleMoveSubSub => {
                vm.pointer += bytecode.delta as isize;
                let v = vm.memory.get(vm.pointer)?;
                if v != 0 {
                    let (d1, d2) = u32_to_two_delta(bytecode.addr);
                    vm.memory.sub(vm.pointer + d1 as isize, v)?;
                    vm.memory.sub(vm.pointer + d2 as isize, v)?;
                    vm.memory.set(vm.pointer, 0)?;
                }
            }

            OpCode::MoveStart => {
                vm.pointer += bytecode.delta as isize;
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
                vm.memory.add(vm.pointer + bytecode.delta as isize, mul_val)?;
            }
            OpCode::MoveSub => {
                vm.memory.sub(vm.pointer + bytecode.delta as isize, mul_val)?;
            }

            OpCode::In => {
                vm.pointer += bytecode.delta as isize;
                match stdin.read_exact(&mut stdin_buf) {
                    Ok(_) => vm.memory.set(vm.pointer, stdin_buf[0])?,
                    Err(_) => vm.memory.set(vm.pointer, 0)?,
                }
            }
            OpCode::Out => {
                vm.pointer += bytecode.delta as isize;
                stdout.write(&[vm.memory.get(vm.pointer)?]).map_err(|_| "Runtime Error: Failed to print")?;
            }

            OpCode::JmpIfZero => {
                vm.pointer += bytecode.delta as isize;
                if vm.memory.get(vm.pointer)? == 0 {
                    vm.pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode::JmpIfNotZero => {
                vm.pointer += bytecode.delta as isize;
                if vm.memory.get(vm.pointer)? != 0 {
                    vm.pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode::PositiveRangeCheckJNZ => {
                vm.pointer += bytecode.delta as isize;
                if (bytecode.val as i8 as i16 as u16 as isize) <= vm.pointer {
                    // TODO: deopt
                }
                if vm.memory.get(vm.pointer)? != 0 {
                    vm.pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode::NegativeRangeCheckJNZ => {
                vm.pointer += bytecode.delta as isize;
                if (bytecode.val as i8 as i16 as u16 as isize) > vm.pointer {
                    // TODO: deopt
                }
                if vm.memory.get(vm.pointer)? != 0 {
                    vm.pc = bytecode.addr as usize;
                    continue;
                }
            }

            OpCode::End => {
                return Ok(());
            }
        }
        vm.pc += 1;
    }
}

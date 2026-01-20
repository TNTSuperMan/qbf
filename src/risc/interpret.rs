use std::io::{Read, Write, stdin, stdout};

use crate::{risc::bytecode::{Bytecode, OpCode}, memory::Memory, trace::OperationCountMap};

pub fn run(insts: &[Bytecode], memory: &mut Memory, ocm: &mut OperationCountMap) -> Result<(), String> {
    let mut stdout = stdout().lock();
    let mut stdin = stdin().lock();
    let mut stdin_buf: [u8; 1] = [0];
    let mut pc: usize = 0;
    let mut pointer: usize = 0;
    let mut mul_val: u8 = 0;
    
    loop {
        #[cfg(feature = "debug")] {
            ocm.0[pc] += 1;
        }

        let bytecode = &insts[pc];
        
        match bytecode.opcode {
            OpCode::Breakpoint => {
                pointer += bytecode.delta as isize as usize;
                eprintln!("PC: {}, PTR: {}", pc, pointer);
            }

            OpCode::Add => {
                pointer += bytecode.delta as isize as usize;
                memory.add(pointer, bytecode.val)?;
            }
            OpCode::Set => {
                pointer += bytecode.delta as isize as usize;
                memory.set(pointer, bytecode.val)?;
            }

            OpCode::Shift => {
                pointer += bytecode.delta as isize as usize;
                let step = bytecode.addr as i32 as isize as usize;
                while memory.get(pointer)? != 0 {
                    pointer += step;
                }
            }

            OpCode::MulStart => {
                pointer += bytecode.delta as isize as usize;
                let val = memory.get(pointer)?;
                if val == 0 {
                    pc = bytecode.addr as usize;
                    continue;
                } else {
                    mul_val = val;
                    memory.set(pointer, 0)?;
                }
            }
            OpCode::Mul => {
                memory.add(pointer + bytecode.delta as isize as usize, mul_val.wrapping_mul(bytecode.val))?;
            }

            OpCode::SingleMoveAdd => {
                pointer += bytecode.delta as isize as usize;
                let v = memory.get(pointer)?;
                if v != 0 {
                    memory.set(pointer, 0)?;
                    memory.add(pointer + (bytecode.addr as i32 as isize as usize), v)?;
                }
            }
            OpCode::SingleMoveSub => {
                pointer += bytecode.delta as isize as usize;
                let v = memory.get(pointer)?;
                if v != 0 {
                    memory.set(pointer, 0)?;
                    memory.sub(pointer + (bytecode.addr as i32 as isize as usize), v)?;
                }
            }

            OpCode::MoveStart => {
                pointer += bytecode.delta as isize as usize;
                let val = memory.get(pointer)?;
                if val == 0 {
                    pc = bytecode.addr as usize;
                    continue;
                } else {
                    mul_val = val;
                    memory.set(pointer, 0)?;
                }
            }
            OpCode::MoveAdd => {
                memory.add(pointer + bytecode.delta as isize as usize, mul_val)?;
            }
            OpCode::MoveSub => {
                memory.sub(pointer + bytecode.delta as isize as usize, mul_val)?;
            }

            OpCode::In => {
                pointer += bytecode.delta as isize as usize;
                match stdin.read_exact(&mut stdin_buf) {
                    Ok(_) => memory.set(pointer, stdin_buf[0])?,
                    Err(_) => memory.set(pointer, 0)?,
                }
            }
            OpCode::Out => {
                pointer += bytecode.delta as isize as usize;
                stdout.write(&[memory.get(pointer)?]).map_err(|_| "Runtime Error: Failed to print")?;
            }

            OpCode::JmpIfZero => {
                pointer += bytecode.delta as isize as usize;
                if memory.get(pointer)? == 0 {
                    pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode::JmpIfNotZero => {
                pointer += bytecode.delta as isize as usize;
                if memory.get(pointer)? != 0 {
                    pc = bytecode.addr as usize;
                    continue;
                }
            }

            OpCode::End => {
                return Ok(());
            }
        }
        pc += 1;
    }
}

use std::{io::{Write, stdout}};

use crate::{parser::{InstOp, Instruction}, trace::OperationCountMap};

pub fn run(insts: Vec<Instruction>, size: usize, map: &mut OperationCountMap) -> Result<(), String> {
    let mut stdout = stdout().lock();
    let mut pc: usize = 0;
    let mut offset: isize = 0;
    let mut memory: Vec<u8> = vec![0; size];

    macro_rules! get {
        ($index: expr) => {
            if let Some(value) = memory.get($index as usize) {
                Ok(*value)
            } else {
                Err(format!("Out of range memory read. Address: {}, PC: {}", $index, pc))
            }
        };
    }
    macro_rules! set {
        ($index: expr, $value: expr) => {
            if let Some(mem) = memory.get_mut($index as usize) {
                *mem = $value;
                Ok(())
            } else {
                Err(format!("Out of range memory write. Address: {}, PC: {}", $index, pc))
            }
        };
    }

    loop {
        #[cfg(feature = "debug")] {
            map.0[pc] += 1;
        }
        let Instruction { opcode, pointer } = &insts[pc];
        let ptr = offset.wrapping_add(*pointer) as isize;
        match opcode {
            InstOp::Breakpoint => {
                // 標準出力と分けるだけ、エラーじゃない
                eprintln!("PC: {}, PTR: {}, ", pc, ptr);
            }

            InstOp::Add(val) => {
                let add_res = get!(ptr)?.wrapping_add(*val);
                set!(ptr, add_res)?;
            }
            InstOp::Set(val) => {
                set!(ptr, *val)?;
            }

            InstOp::Shift(diff) => {
                while get!(offset.wrapping_add(*pointer))? != 0 {
                    offset = offset.wrapping_add(*diff);
                }
            }
            InstOp::MulAndSetZero(dests) => {
                let source_val = get!(ptr)?;
                if source_val != 0 {
                    for (dest_p, m) in dests {
                        let dest_ptr = offset.wrapping_add(*dest_p);
                        let dest_val = get!(dest_ptr)?.wrapping_add(source_val.wrapping_mul(*m));
                        set!(dest_ptr, dest_val)?;
                    }
                    set!(ptr, 0)?;
                }
            }
            InstOp::MulAndSetZeroTo(source, dests) => {
                let source_val = memory[offset.wrapping_add(*source) as usize].wrapping_add(get!(ptr)?);
                if source_val != 0 {
                    for (dest_p, m) in dests {
                        let dest_ptr = offset.wrapping_add(*dest_p) as usize;
                        memory[dest_ptr] = memory[dest_ptr].wrapping_add(source_val.wrapping_mul(*m));
                    }
                    set!(ptr, 0)?;
                }
            }

            InstOp::In => {
                set!(ptr, 0)?; // TODO
            }
            InstOp::Out => {
                stdout.write(&[get!(ptr)?]).unwrap();
            }

            InstOp::LoopStart(end) => {
                if get!(ptr)? == 0 {
                    pc = *end;
                }
            }
            InstOp::LoopEnd(start) => {
                if get!(ptr)? != 0 {
                    pc = *start;
                }
            }
            InstOp::LoopEndWithOffset(start, off) => {
                if get!(ptr)? != 0 {
                    pc = *start;
                }
                offset = offset.wrapping_add(*off);
            }
            InstOp::End => {
                return Ok(());
            }
        }
        pc = pc.wrapping_add(1);
    }
}

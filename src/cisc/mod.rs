use crate::{cisc::{bytecode::ir_to_bytecodes, internal::{InterpreterResult, Tier}, interpret_deopt::run_deopt, interpret_opt::run_opt, trace::write_trace, vm::{UnsafeInsts, UnsafeVM, VM}}, ir::IR, range::RangeInfo};

mod memory;
mod error;
mod bytecode;
mod interpret_deopt;
mod interpret_opt;
mod trace;
mod vm;
mod internal;

pub fn run_cisc(ir_nodes: &[IR], range_info: &RangeInfo, flush: bool, out_dump: bool) -> Result<(), String> {
    let insts = ir_to_bytecodes(ir_nodes, range_info)?;
    let mut vm = VM::new(insts.len(), flush)?;
    let mut tier = if range_info.do_opt_first {
        Tier::Opt
    } else {
        Tier::Deopt
    };
    #[cfg(feature = "trace")]
    println!("[TRACE] first: {:?}", tier);

    loop {
        let result = match tier {
            Tier::Deopt => run_deopt(&mut vm, &insts),
            Tier::Opt => unsafe {
                let mut insts = UnsafeInsts::new(&insts, vm.pc);
                let run_res = {
                    let mut unsafe_vm = UnsafeVM::new(&mut vm);
                    run_opt(&mut unsafe_vm, &mut insts)
                };
                vm.pc = insts.get_pc();
                run_res
            },
        };
        match result {
            Ok(InterpreterResult::End) => {
                if cfg!(feature = "debug") && out_dump {
                    write_trace(&vm, &insts);
                }
                return Ok(());
            }
            Ok(InterpreterResult::ToggleTier(t)) => {
                #[cfg(feature = "trace")]
                println!("[TRACE] tier switch to {:?}", t);
                tier = t;
            }
            Err(msg) => {
                if cfg!(feature = "debug") {
                    if out_dump {
                        write_trace(&vm, &insts);
                    }
                    println!("PC: {}({:?}), ptr: {}", vm.pc, insts[vm.pc], vm.pointer);
                }
                return Err(msg);
            }
        }
    }
}

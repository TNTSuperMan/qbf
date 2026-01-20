use crate::{cisc::{internal::{InterpreterResult, Tier}, interpret_deopt::run_deopt, interpret_opt::run_opt, trace::write_trace, vm::VM}, ir::IR, range::RangeInfo};

mod bytecode;
mod interpret_deopt;
mod interpret_opt;
mod trace;
mod vm;
mod internal;

pub fn run_cisc(ir_nodes: &[IR], range_info: &RangeInfo) -> Result<(), String> {
    let mut vm = VM::new(ir_nodes, range_info)?;
    let mut tier = if range_info.do_opt_first {
        Tier::Opt
    } else {
        Tier::Deopt
    };
    #[cfg(feature = "trace")]
    println!("[TRACE] first: {:?}", tier);

    loop {
        let result = match tier {
            Tier::Deopt => run_deopt(&mut vm),
            Tier::Opt => unsafe { run_opt(&mut vm) },
        };
        match result {
            Ok(InterpreterResult::End) => {
                write_trace(&vm);
                return Ok(());
            }
            Ok(InterpreterResult::ToggleTier(t)) => {
                #[cfg(feature = "trace")]
                println!("[TRACE] tier switch to {:?}", t);
                tier = t;
            }
            Err(msg) => {
                write_trace(&vm);
                #[cfg(feature = "debug")] {
                    println!("PC: {}({:?}), ptr: {}", vm.pc, vm.insts[vm.pc], vm.pointer);
                }
                return Err(msg);
            }
        }
    }
}

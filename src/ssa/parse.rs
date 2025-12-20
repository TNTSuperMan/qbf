use crate::{ir::{IR, IROp}, ssa::{SSAOp, PointerSSAHistory, PointerVersion}};

pub fn build_ssa_from_ir(ir_arr: &[IR]) -> Option<PointerSSAHistory> {
    let mut ssa_history: PointerSSAHistory = PointerSSAHistory::new();

    for ir in ir_arr {
        match &ir.opcode {
            IROp::Add(val) => {
                let current_history = ssa_history.get_history_mut(ir.pointer);
                current_history.push(SSAOp::add_pc(PointerVersion {
                    ptr: ir.pointer,
                    version: current_history.len() - 1,
                }, *val));
            }
            IROp::Set(val) => {
                let current_history = ssa_history.get_history_mut(ir.pointer);
                current_history.push(SSAOp::set_c(*val));
            }
            IROp::MulAndSetZero(dests) => {
                let source_version = PointerVersion {
                    ptr: ir.pointer,
                    version: match ssa_history.get_history(ir.pointer) {
                        Some(history) => history.len() - 1,
                        None => 0,
                    },
                };
                for (dest_ptr, dest_val) in dests {
                    let dest_history = ssa_history.get_history_mut(*dest_ptr);
                    dest_history.push(SSAOp::mul_add(PointerVersion {
                        ptr: *dest_ptr,
                        version: dest_history.len() - 1,
                    }, source_version, *dest_val));
                }
                let source_history = ssa_history.get_history_mut(ir.pointer);
                source_history.push(SSAOp::set_c(0));
            }
            _ => {
                return None;
            }
        }
    }

    Some(ssa_history)
}

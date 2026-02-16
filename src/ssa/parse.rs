use crate::{ir::{IR, IROp}, ssa::structs::{PointerSSAHistory, PointerVersion, SSAOp, SSAValue}};

pub fn build_ssa_from_ir(ir_nodes: &[IR]) -> Option<PointerSSAHistory> {
    let mut ssa_history: PointerSSAHistory = PointerSSAHistory::new();

    for ir in ir_nodes {
        match &ir.opcode {
            IROp::Add(val) => {
                let current_history = ssa_history.get_history_mut(ir.pointer);
                current_history.push(SSAOp::Add(SSAValue::Version(PointerVersion {
                    ptr: ir.pointer,
                    version: current_history.len() - 1,
                }), SSAValue::Const(*val)));
            }
            IROp::Set(val) => {
                let current_history = ssa_history.get_history_mut(ir.pointer);
                current_history.push(SSAOp::Value(SSAValue::Const(*val)));
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
                    if *dest_val == 1 {
                        dest_history.push(SSAOp::Add(SSAValue::Version(PointerVersion {
                            ptr: *dest_ptr,
                            version: dest_history.len() - 1,
                        }), SSAValue::Version(source_version)));
                    } else {
                        dest_history.push(SSAOp::Mul(SSAValue::Version(source_version), SSAValue::Const(*dest_val)));
                        dest_history.push(SSAOp::Add(SSAValue::Version(PointerVersion {
                            ptr: *dest_ptr,
                            version: dest_history.len() - 2,
                        }), SSAValue::Version(PointerVersion {
                            ptr: *dest_ptr,
                            version: dest_history.len() - 1,
                        })));
                    }
                }
                let source_history = ssa_history.get_history_mut(ir.pointer);
                source_history.push(SSAOp::Value(SSAValue::Const(0)));
            }
            IROp::MovesAndSetZero(dests) => {
                let source_version = PointerVersion {
                    ptr: ir.pointer,
                    version: match ssa_history.get_history(ir.pointer) {
                        Some(history) => history.len() - 1,
                        None => 0,
                    },
                };
                for (dest_ptr, is_pos) in dests {
                    let dest_history = ssa_history.get_history_mut(*dest_ptr);
                    if *is_pos {
                        dest_history.push(SSAOp::Add(SSAValue::Version(PointerVersion {
                            ptr: *dest_ptr,
                            version: dest_history.len() - 1,
                        }), SSAValue::Version(source_version)));
                    } else {
                        dest_history.push(SSAOp::Sub(SSAValue::Version(PointerVersion {
                            ptr: *dest_ptr,
                            version: dest_history.len() - 1,
                        }), SSAValue::Version(source_version)));
                    }
                }
                let source_history = ssa_history.get_history_mut(ir.pointer);
                source_history.push(SSAOp::Value(SSAValue::Const(0)));
            }
            _ => {
                return None;
            }
        }
    }

    Some(ssa_history)
}

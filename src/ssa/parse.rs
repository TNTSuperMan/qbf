use std::collections::HashMap;

use crate::{ir::{IR, IROp}, ssa::{PointerOperation, PointerSSAHistory, PointerVersion}};

pub fn build_ssa_from_ir(ir_arr: &[IR]) -> Option<PointerSSAHistory> {
    let mut ssa_history: PointerSSAHistory = HashMap::new();

    macro_rules! get_pointer_history {
        ($ptr: expr) => {
            ssa_history.entry($ptr).or_insert_with(|| vec![PointerOperation::UntrackedValue($ptr)])
        };
    }

    for ir in ir_arr {
        match &ir.opcode {
            IROp::Add(val) => {
                let current_history = get_pointer_history!(ir.pointer);
                current_history.push(PointerOperation::AddConstant(PointerVersion {
                    ptr: ir.pointer,
                    version: current_history.len() - 1,
                }, *val));
            }
            IROp::Set(val) => {
                let current_history = get_pointer_history!(ir.pointer);
                current_history.push(PointerOperation::AssignConstant(*val));
            }
            IROp::MulAndSetZero(dests) => {
                let source_version = PointerVersion {
                    ptr: ir.pointer,
                    version: match ssa_history.get(&ir.pointer) {
                        Some(history) => history.len() - 1,
                        None => 0,
                    },
                };
                for (dest_ptr, dest_val) in dests {
                    let dest_history = get_pointer_history!(*dest_ptr);
                    dest_history.push(PointerOperation::AddMultipliedValue(PointerVersion {
                        ptr: *dest_ptr,
                        version: dest_history.len() - 1,
                    }, source_version, *dest_val));
                }
                let source_history = get_pointer_history!(ir.pointer);
                source_history.push(PointerOperation::AssignConstant(0));
            }
            _ => {
                return None;
            }
        }
    }

    Some(ssa_history)
}

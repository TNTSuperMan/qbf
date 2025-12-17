use std::collections::HashMap;

use crate::{ir::{IR, IROp}, ssa::{SSA, SSAData, SSAVariant}};

pub fn ir_to_ssa(ir_arr: &[IR]) -> Option<SSAData> {
    let mut ssa_data: SSAData = HashMap::new();

    macro_rules! get {
        ($ptr: expr) => {
            ssa_data.entry($ptr).or_insert_with(|| vec![SSAVariant::Raw($ptr)])
        };
    }

    for ir in ir_arr {
        match &ir.opcode {
            IROp::Add(val) => {
                let curr_ssa = get!(ir.pointer);
                curr_ssa.push(SSAVariant::AddConst(*val));
            }
            IROp::Set(val) => {
                let curr_ssa = get!(ir.pointer);
                curr_ssa.push(SSAVariant::SetConst(*val));
            }
            IROp::MulAndSetZero(dests) => {
                let src_ssa = SSA {
                    ptr: ir.pointer,
                    index: match ssa_data.get(&ir.pointer) {
                        Some(ssa) => ssa.len() - 1,
                        None => 0,
                    },
                };
                for (dest_ptr, dest_val) in dests {
                    let dest_ssa = get!(*dest_ptr);
                    dest_ssa.push(SSAVariant::AddMul(*dest_val, src_ssa));
                }
                let src_ssa = get!(ir.pointer);
                src_ssa.push(SSAVariant::SetConst(0));
            }
            _ => {
                return None;
            }
        }
    }

    Some(ssa_data)
}

use crate::ssa::{PointerSSAHistory, PointerVersion, SSAOp, SSAValue};

pub fn detect_ssa_loop(history: &PointerSSAHistory) -> Option<(isize, PointerSSAHistory)> {
    let loop_el_opt = history.iter().find(|(&ptr, h)| {
        let root1 = SSAValue::Version(PointerVersion { ptr, version: 0 });
        let root2 = SSAValue::Raw(ptr);
        let last = *h.last().unwrap();
        last == SSAOp::Add(root1, SSAValue::Const(255))
        || last == SSAOp::Add(SSAValue::Const(255), root1)
        || last == SSAOp::Add(root2, SSAValue::Const(255))
        || last == SSAOp::Add(SSAValue::Const(255), root2)
        || last == SSAOp::Sub(root1, SSAValue::Const(1))
        || last == SSAOp::Sub(root2, SSAValue::Const(1))
    });

    match loop_el_opt {
        None => None,
        Some((&loop_ptr, _)) => {
            let mut ret_history = history.clone();
            ret_history.0.remove(&loop_ptr);
            for (_, h) in ret_history.iter() {
                macro_rules! is_eq_ptr {
                    ($val: expr) => {
                        if let SSAValue::Version(ver) = $val {
                            ver.ptr == loop_ptr
                        } else {
                            false
                        }
                    };
                }
                let use_loop_el = h.iter().any(|op| match op {
                    SSAOp::Value(val) => is_eq_ptr!(*val),
                    SSAOp::Add(v1, v2) => is_eq_ptr!(v1) || is_eq_ptr!(v2),
                    SSAOp::Sub(v1, v2) => is_eq_ptr!(v1) || is_eq_ptr!(v2),
                    SSAOp::Mul(v1, v2) => is_eq_ptr!(v1) || is_eq_ptr!(v2),
                });
                if use_loop_el {
                    return None;
                }
            }
            Some((loop_ptr, ret_history))
        },
    }
}

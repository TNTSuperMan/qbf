use crate::ssa::{PointerSSAHistory, PointerVersion, SSAOp, SSAValue};

pub fn detect_ssa_loop(history: &PointerSSAHistory) -> Option<(isize, PointerSSAHistory)> {
    return None;
    let loop_el_opt = history.iter().find(|(&ptr, h)| {
        h.len() == 2
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
                };
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

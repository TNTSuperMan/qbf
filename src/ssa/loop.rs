use crate::ssa::{PointerSSAHistory, PointerVersion, SSAOp};

pub fn detect_ssa_loop(history: &PointerSSAHistory) -> Option<(isize, PointerSSAHistory)> {
    let loop_el_opt = history.iter().find(|(&ptr, h)| {
        h.len() == 2 && h[1] == SSAOp::add_pc(PointerVersion { ptr, version: 0 }, 255)
    });

    match loop_el_opt {
        None => None,
        Some((&loop_ptr, _)) => {
            let mut ret_history = history.clone();
            ret_history.0.remove(&loop_ptr);
            for (_, h) in ret_history.iter() {
                let use_loop_el = h.iter().any(|op| match op {
                    SSAOp::raw(ptr) => *ptr == loop_ptr,
                    SSAOp::set_c(..) => false,
                    SSAOp::add_pc(PointerVersion { ptr, .. }, ..) => *ptr == loop_ptr,
                    SSAOp::sub_pc(PointerVersion { ptr, .. }, ..) => *ptr == loop_ptr,
                    SSAOp::sub_cp(_,  PointerVersion { ptr, .. }) => *ptr == loop_ptr,
                    SSAOp::mul_pc(PointerVersion { ptr, .. }, ..) => *ptr == loop_ptr,
                    SSAOp::add_pp(PointerVersion { ptr: ptr1, .. }, PointerVersion { ptr: ptr2, .. }) => *ptr1 == loop_ptr || *ptr2 == loop_ptr,
                    SSAOp::sub_pp(PointerVersion { ptr: ptr1, .. }, PointerVersion { ptr: ptr2, .. }) => *ptr1 == loop_ptr || *ptr2 == loop_ptr,

                    SSAOp::mul_add(PointerVersion { ptr: ptr1, .. }, PointerVersion { ptr: ptr2, .. }, ..) => *ptr1 == loop_ptr || *ptr2 == loop_ptr,
                });
                if use_loop_el {
                    return None;
                }
            }
            Some((loop_ptr, ret_history))
        },
    }
}

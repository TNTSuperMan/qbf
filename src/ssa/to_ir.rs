use std::collections::HashSet;

use crate::{ir::IR, ssa::{PointerSSAHistory, PointerVersion, SSAOp}};

enum Schedule {
    Visit(PointerVersion),
    PushOrder(PointerVersion),
}

pub fn resolve_eval_order(history: &PointerSSAHistory) -> Vec<PointerVersion> {
    let mut eval_order: Vec<PointerVersion> = vec![];
    let mut visited: HashSet<PointerVersion> = HashSet::new();
    let mut schedule: Vec<Schedule> = history.iter().map(|(ptr,h)| {
        Schedule::Visit(PointerVersion { ptr: *ptr, version: h.len() - 1 })
    }).collect();

    loop {
        match schedule.pop() {
            None => {
                return eval_order;
            }
            Some(Schedule::Visit(ver)) => {
                if !visited.insert(ver) {
                    continue;
                }
                schedule.push(Schedule::PushOrder(ver));
                match history.get_op(ver).unwrap() {
                    SSAOp::raw(_ptr) => (),
                    SSAOp::set_c(_val) => (),
                    SSAOp::set_p(ver) => schedule.push(Schedule::Visit(ver)),
                    SSAOp::add_pc(ver, _val) => schedule.push(Schedule::Visit(ver)),
                    SSAOp::add_pp(ver, ver2) => { schedule.push(Schedule::Visit(ver)); schedule.push(Schedule::Visit(ver2)); },
                    SSAOp::mul_pc(ver, _val) => schedule.push(Schedule::Visit(ver)),

                    SSAOp::mul_add(from, dest, _val) => { schedule.push(Schedule::Visit(from)); schedule.push(Schedule::Visit(dest)); },
                }
            }
            Some(Schedule::PushOrder(ver)) => {
                eval_order.push(ver);
            }
        }
    }
}

pub fn ssa_to_ir(history: &PointerSSAHistory) -> Vec<IR> {
    let order = resolve_eval_order(history);
    unimplemented!();
}

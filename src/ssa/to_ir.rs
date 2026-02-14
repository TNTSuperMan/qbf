use std::collections::HashSet;

use crate::{ir::IR, ssa::{PointerSSAHistory, PointerVersion, SSAOp, SSAValue}};

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

    macro_rules! schedule_visit {
        ($v: expr) => {
            if let SSAValue::Version(ver) = $v {
                schedule.push(Schedule::Visit(ver))
            }
        };
    }

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
                    SSAOp::Value(v) => schedule_visit!(v),
                    SSAOp::Add(v1, v2) => { schedule_visit!(v1); schedule_visit!(v2); },
                    SSAOp::Sub(v1, v2) => { schedule_visit!(v1); schedule_visit!(v2); },
                    SSAOp::Mul(v1, v2) => { schedule_visit!(v1); schedule_visit!(v2); },
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

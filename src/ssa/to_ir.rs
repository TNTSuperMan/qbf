use std::collections::HashSet;

use crate::{ir::{IR, IROp}, ssa::{PointerSSAHistory, PointerVersion, SSAOp, SSAValue}};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SSAValueIR {
    Const(u8),
    Get(u8), // 連番
    Raw(isize),
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SSAOpIR {
    Value(SSAValueIR),
    Add(SSAValueIR, SSAValueIR),
    Sub(SSAValueIR, SSAValueIR),
    Mul(SSAValueIR, SSAValueIR),
}


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

fn ssa_val_to_ir(val: &SSAValue, order: &[PointerVersion]) -> SSAValueIR {
    match val {
        SSAValue::Const(val) => SSAValueIR::Const(*val),
        SSAValue::Version(ver) => SSAValueIR::Get(order.iter().position(|v| *v == *ver).unwrap() as u8),
        SSAValue::Raw(ptr) => SSAValueIR::Raw(*ptr),
    }
}

fn ssa_op_to_ir(op: &SSAOp, order: &[PointerVersion]) -> SSAOpIR {
    match op {
        SSAOp::Value(val) => SSAOpIR::Value(ssa_val_to_ir(val, order)),
        SSAOp::Add(v1, v2) => SSAOpIR::Add(
            ssa_val_to_ir(v1, order),
            ssa_val_to_ir(v2, order),
        ),
        SSAOp::Sub(v1, v2) => SSAOpIR::Sub(
            ssa_val_to_ir(v1, order),
            ssa_val_to_ir(v2, order),
        ),
        SSAOp::Mul(v1, v2) => SSAOpIR::Mul(
            ssa_val_to_ir(v1, order),
            ssa_val_to_ir(v2, order),
        ),
    }
}

pub fn ssa_to_ir(history: &PointerSSAHistory) -> Vec<IR> {
    let order = resolve_eval_order(history);
    let mut ir: Vec<IR> = vec![IR { pointer: order[0].ptr, opcode: IROp::StartSSA }];

    for ver in order.iter() {
        let op = history.get_op(*ver).unwrap();
        ir.push(IR { pointer: ver.ptr, opcode: IROp::PushSSA(ssa_op_to_ir(&op, &order)) });
    }

    for (&pointer, h) in history.iter() {
        let last_idx = h.len() - 1;
        let i = order.iter().position(|v| *v == PointerVersion { ptr: pointer, version: last_idx }).unwrap();
        ir.push(IR { pointer, opcode: IROp::AssignSSA(i as u8) });
    }

    ir
}

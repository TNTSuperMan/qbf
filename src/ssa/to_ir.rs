use std::collections::HashSet;

use crate::{ir::{IR, IROp}, ssa::structs::{PointerSSAHistory, PointerVersion, SSAOp, SSAValue}};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SSAOpIR { // R-Reference C-Const M-Memory
    Const(u8),
    Memory(isize),

    AddRR(u8, u8),
    AddRC(u8, u8),
    AddMR(isize, u8),
    AddMC(isize, u8),
    AddMM(isize, isize),

    SubRR(u8, u8),
    SubRC(u8, u8),
    SubRM(u8, isize),
    SubCR(u8, u8),
    SubCM(u8, isize),
    SubMR(isize, u8),
    SubMC(isize, u8),
    SubMM(isize, isize),
    
    MulRR(u8, u8),
    MulRC(u8, u8),
    MulMR(isize, u8),
    MulMC(isize, u8),
    MulMM(isize, isize),
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

fn ssa_op_to_ir(op: &SSAOp, order: &[PointerVersion]) -> SSAOpIR {
    macro_rules! refid {
        ($ver: expr) => {
            order.iter().position(|v| *v == $ver).unwrap() as u8
        };
    }
    match op {
        SSAOp::Value(SSAValue::Const(c)) => SSAOpIR::Const(*c),
        SSAOp::Value(SSAValue::Version(_)) => panic!("SSAValue::Version must to be inlined"),
        SSAOp::Value(SSAValue::Raw(m)) => SSAOpIR::Memory(*m),

        SSAOp::Add(v1, v2) => match (v1, v2) {
            (SSAValue::Const(c1), SSAValue::Const(c2)) => SSAOpIR::Const(c1.wrapping_add(*c2)),
            (SSAValue::Const(c1), SSAValue::Version(v2)) => SSAOpIR::AddRC(refid!(*v2), *c1),
            (SSAValue::Const(c1), SSAValue::Raw(m2)) => SSAOpIR::AddMC(*m2, *c1),
            (SSAValue::Version(v1), SSAValue::Const(c2)) => SSAOpIR::AddRC(refid!(*v1), *c2),
            (SSAValue::Version(v1), SSAValue::Version(v2)) => SSAOpIR::AddRR(refid!(*v1), refid!(*v2)),
            (SSAValue::Version(v1), SSAValue::Raw(m2)) => SSAOpIR::AddMR(*m2, refid!(*v1)),
            (SSAValue::Raw(m1), SSAValue::Const(c2)) => SSAOpIR::AddMC(*m1, *c2),
            (SSAValue::Raw(m1), SSAValue::Version(v2)) => SSAOpIR::AddMR(*m1, refid!(*v2)),
            (SSAValue::Raw(m1), SSAValue::Raw(m2)) => SSAOpIR::AddMM(*m1, *m2),
        },

        SSAOp::Sub(v1, v2) => match (v1, v2) {
            (SSAValue::Const(c1), SSAValue::Const(c2)) => SSAOpIR::Const(c1.wrapping_sub(*c2)),
            (SSAValue::Const(c1), SSAValue::Version(v2)) => SSAOpIR::SubCR(refid!(*v2), *c1),
            (SSAValue::Const(c1), SSAValue::Raw(m2)) => SSAOpIR::SubCM(*c1, *m2),

            (SSAValue::Version(v1), SSAValue::Const(c2)) => SSAOpIR::SubRC(refid!(*v1), *c2),
            (SSAValue::Version(v1), SSAValue::Version(v2)) => SSAOpIR::SubRR(refid!(*v1), refid!(*v2)),
            (SSAValue::Version(v1), SSAValue::Raw(m2)) => SSAOpIR::SubRM(refid!(*v1), *m2),

            (SSAValue::Raw(m1), SSAValue::Const(c2)) => SSAOpIR::SubMC(*m1, *c2),
            (SSAValue::Raw(m1), SSAValue::Version(v2)) => SSAOpIR::SubMR(*m1, refid!(*v2)),
            (SSAValue::Raw(m1), SSAValue::Raw(m2)) => SSAOpIR::SubMM(*m1, *m2),
        },

        SSAOp::Mul(v1, v2) => match (v1, v2) {
            (SSAValue::Const(c1), SSAValue::Const(c2)) => SSAOpIR::Const(c1.wrapping_mul(*c2)),
            (SSAValue::Const(c1), SSAValue::Version(v2)) => SSAOpIR::MulRC(refid!(*v2), *c1),
            (SSAValue::Const(c1), SSAValue::Raw(m2)) => SSAOpIR::MulMC(*m2, *c1),
            (SSAValue::Version(v1), SSAValue::Const(c2)) => SSAOpIR::MulRC(refid!(*v1), *c2),
            (SSAValue::Version(v1), SSAValue::Version(v2)) => SSAOpIR::MulRR(refid!(*v1), refid!(*v2)),
            (SSAValue::Version(v1), SSAValue::Raw(m2)) => SSAOpIR::MulMR(*m2, refid!(*v1)),
            (SSAValue::Raw(m1), SSAValue::Const(c2)) => SSAOpIR::MulMC(*m1, *c2),
            (SSAValue::Raw(m1), SSAValue::Version(v2)) => SSAOpIR::MulMR(*m1, refid!(*v2)),
            (SSAValue::Raw(m1), SSAValue::Raw(m2)) => SSAOpIR::MulMM(*m1, *m2),
        },
    }
}

/*
pub fn ssa_to_ir(history: &PointerSSAHistory) -> Vec<IR> {
    let order = resolve_eval_order(history);
    let mut ir: Vec<IR> = vec![];

    for (i, ver) in order.iter().enumerate() {
        let op = history.get_op(*ver).unwrap();
        ir.push(IR { pointer: ver.ptr, opcode: IROp::SetSSA(i as u8, ssa_op_to_ir(&op, &order)), source_range: None });
    }

    for (&pointer, h) in history.iter() {
        let last_idx = h.len() - 1;
        let i = order.iter().position(|v| *v == PointerVersion { ptr: pointer, version: last_idx }).unwrap();
        ir.push(IR { pointer, opcode: IROp::AssignSSA(i as u8), source_range: None });
    }

    ir
}
*/

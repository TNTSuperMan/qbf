use crate::ssa::{PointerSSAHistory, PointerVersion, SSAOp, SSAValue};

impl PointerSSAHistory {
    fn get_val(&self, ver: PointerVersion, inline_raw: bool) -> SSAValue {
        match self.get_op(ver).unwrap() {
            SSAOp::Value(SSAValue::Raw(i)) => {
                if inline_raw {
                    SSAValue::Raw(i)
                } else {
                    SSAValue::Version(ver)
                }
            }
            SSAOp::Value(v) => v,
            _ => SSAValue::Version(ver),
        }
    }
}

pub fn inline_ssa_history(history_map: &PointerSSAHistory, inline_raw: bool) -> PointerSSAHistory {
    let mut inlined_history_map: PointerSSAHistory = PointerSSAHistory::new();
    for (ptr, history) in history_map.iter() {
        let mut inlined_history: Vec<SSAOp> = vec![];
        for h in history {
            let op: Option<SSAOp> = match h {
                SSAOp::Value(val) => {
                    match val {
                        SSAValue::Const(..) => None,
                        SSAValue::Version(ver) => Some(history_map.get_op(*ver).unwrap()),
                        SSAValue::Raw(..) => None,
                    }
                },

                SSAOp::Add(val1, val2) => match (val1, val2) {
                    (SSAValue::Const(v1), SSAValue::Const(v2)) => Some(SSAOp::Value(SSAValue::Const(v1.wrapping_add(*v2)))),
                    (SSAValue::Const(v1), SSAValue::Version(v2)) => {
                        if *v1 == 0 {
                            Some(SSAOp::Value(history_map.get_val(*v2, inline_raw)))
                        } else {
                            Some(SSAOp::Add(SSAValue::Const(*v1), history_map.get_val(*v2, inline_raw)))
                        }
                    },
                    (SSAValue::Const(v1), SSAValue::Raw(v2)) => {
                        if *v1 == 0 {
                            Some(SSAOp::Value(SSAValue::Raw(*v2)))
                        } else {
                            None
                        }
                    },
                    (SSAValue::Version(v1), SSAValue::Const(v2)) => {
                        if *v2 == 0 {
                            Some(SSAOp::Value(history_map.get_val(*v1, inline_raw)))
                        } else {
                            Some(SSAOp::Add(history_map.get_val(*v1, inline_raw), SSAValue::Const(*v2)))
                        }
                    },
                    (SSAValue::Version(v1), SSAValue::Version(v2)) => Some(SSAOp::Add(history_map.get_val(*v1, inline_raw), history_map.get_val(*v2, inline_raw))),
                    (SSAValue::Version(v1), SSAValue::Raw(v2)) => Some(SSAOp::Add(history_map.get_val(*v1, inline_raw), SSAValue::Raw(*v2))),
                    (SSAValue::Raw(v1), SSAValue::Const(v2)) => {
                        if *v2 == 0 {
                            Some(SSAOp::Value(SSAValue::Raw(*v1)))
                        } else {
                            None
                        }
                    },
                    (SSAValue::Raw(v1), SSAValue::Version(v2)) => Some(SSAOp::Add(SSAValue::Raw(*v1), history_map.get_val(*v2, inline_raw))),
                    (SSAValue::Raw(_v1), SSAValue::Raw(_v2)) => None,
                },

                SSAOp::Sub(val1, val2) => match (val1, val2) {
                    (SSAValue::Const(v1), SSAValue::Const(v2)) => Some(SSAOp::Value(SSAValue::Const(v1.wrapping_sub(*v2)))),
                    (SSAValue::Const(v1), SSAValue::Version(v2)) => Some(SSAOp::Sub(SSAValue::Const(*v1), history_map.get_val(*v2, inline_raw))),
                    (SSAValue::Const(_v1), SSAValue::Raw(_v2)) => None,
                    (SSAValue::Version(v1), SSAValue::Const(v2)) => {
                        if *v2 == 0 {
                            Some(SSAOp::Value(history_map.get_val(*v1, inline_raw)))
                        } else {
                            Some(SSAOp::Sub(history_map.get_val(*v1, inline_raw), SSAValue::Const(*v2)))
                        }
                    },
                    (SSAValue::Version(v1), SSAValue::Version(v2)) => Some(SSAOp::Sub(history_map.get_val(*v1, inline_raw), history_map.get_val(*v2, inline_raw))),
                    (SSAValue::Version(v1), SSAValue::Raw(v2)) => Some(SSAOp::Sub(history_map.get_val(*v1, inline_raw), SSAValue::Raw(*v2))),
                    (SSAValue::Raw(v1), SSAValue::Const(v2)) => {
                        if *v2 == 0 {
                            Some(SSAOp::Value(SSAValue::Raw(*v1)))
                        } else {
                            None
                        }
                    },
                    (SSAValue::Raw(v1), SSAValue::Version(v2)) => Some(SSAOp::Sub(SSAValue::Raw(*v1), history_map.get_val(*v2, inline_raw))),
                    (SSAValue::Raw(_v1), SSAValue::Raw(_v2)) => None,
                },

                SSAOp::Mul(val1, val2) => match (val1, val2) {
                    (SSAValue::Const(v1), SSAValue::Const(v2)) => Some(SSAOp::Value(SSAValue::Const(v1.wrapping_mul(*v2)))),
                    (SSAValue::Const(v1), SSAValue::Version(v2)) => {
                        if *v1 == 1 {
                            Some(SSAOp::Value(history_map.get_val(*v2, inline_raw)))
                        } else {
                            Some(SSAOp::Mul(SSAValue::Const(*v1), history_map.get_val(*v2, inline_raw)))
                        }
                    },
                    (SSAValue::Const(v1), SSAValue::Raw(v2)) => {
                        if *v1 == 1 {
                            Some(SSAOp::Value(SSAValue::Raw(*v2)))
                        } else {
                            None
                        }
                    },
                    (SSAValue::Version(v1), SSAValue::Const(v2)) => {
                        if *v2 == 0 {
                            Some(SSAOp::Value(history_map.get_val(*v1, inline_raw)))
                        } else {
                            Some(SSAOp::Mul(history_map.get_val(*v1, inline_raw), SSAValue::Const(*v2)))
                        }
                    },
                    (SSAValue::Version(v1), SSAValue::Version(v2)) => Some(SSAOp::Mul(history_map.get_val(*v1, inline_raw), history_map.get_val(*v2, inline_raw))),
                    (SSAValue::Version(v1), SSAValue::Raw(v2)) => Some(SSAOp::Mul(history_map.get_val(*v1, inline_raw), SSAValue::Raw(*v2))),
                    (SSAValue::Raw(v1), SSAValue::Const(v2)) => {
                        if *v2 == 1 {
                            Some(SSAOp::Value(SSAValue::Raw(*v1)))
                        } else {
                            None
                        }
                    },
                    (SSAValue::Raw(v1), SSAValue::Version(v2)) => Some(SSAOp::Mul(SSAValue::Raw(*v1), history_map.get_val(*v2, inline_raw))),
                    (SSAValue::Raw(_v1), SSAValue::Raw(_v2)) => None,
                },
            };
            inlined_history.push(op.unwrap_or(*h));
        }
        inlined_history_map.insert(*ptr, inlined_history);
    }

    inlined_history_map
}

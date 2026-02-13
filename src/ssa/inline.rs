use crate::ssa::{PointerSSAHistory, PointerVersion, SSAOp};

enum SimpleSSAOp {
    Const(u8),
    Version(PointerVersion),
}
impl PointerSSAHistory {
    fn get_simple_op(&self, ver: PointerVersion) -> SimpleSSAOp {
        match self.get_op(ver).unwrap() {
            SSAOp::set_c(val) => SimpleSSAOp::Const(val),
            SSAOp::set_p(v) => self.get_simple_op(v),
            SSAOp::mul_pc(_, 0) => SimpleSSAOp::Const(0),
            SSAOp::mul_pc(v, 1) => self.get_simple_op(v),
            _ => SimpleSSAOp::Version(ver)
        }
    }
}

pub fn inline_ssa_history(history_map: &PointerSSAHistory) -> PointerSSAHistory {
    let mut inlined_history_map: PointerSSAHistory = PointerSSAHistory::new();
    for (ptr, history) in history_map.iter() {
        let mut inlined_history: Vec<SSAOp> = vec![];
        for h in history {
            match h {
                SSAOp::raw(_raw_ptr) => {}
                SSAOp::set_c(_val) => {}
                SSAOp::set_p(_ver) => {}
                SSAOp::add_pc(ver, val) => {
                    if *val == 0 {
                        inlined_history.push(SSAOp::set_p(*ver));
                        continue;
                    }
                    match history_map.get_simple_op(*ver) {
                        SimpleSSAOp::Const(val2) => {
                            inlined_history.push(SSAOp::set_c(val.wrapping_add(val2)));
                            continue;
                        }
                        SimpleSSAOp::Version(ver2) => {
                            inlined_history.push(SSAOp::add_pc(ver2, *val));
                            continue;
                        }
                    }
                }
                SSAOp::sub_pc(ver, val) => {
                    if *val == 0 {
                        inlined_history.push(SSAOp::set_p(*ver));
                        continue;
                    }
                    match history_map.get_simple_op(*ver) {
                        SimpleSSAOp::Const(val2) => {
                            inlined_history.push(SSAOp::set_c(val.wrapping_sub(val2)));
                            continue;
                        }
                        SimpleSSAOp::Version(ver2) => {
                            inlined_history.push(SSAOp::sub_pc(ver2, *val));
                            continue;
                        }
                    }
                }
                SSAOp::add_pp(ver_left, ver_right) => {
                    let left = history_map.get_simple_op(*ver_left);
                    let right = history_map.get_simple_op(*ver_right);

                    match (left, right) {
                        (SimpleSSAOp::Const(lv), SimpleSSAOp::Const(rv)) => {
                            inlined_history.push(SSAOp::set_c(lv.wrapping_add(rv)));
                            continue;
                        }
                        
                        (SimpleSSAOp::Version(ver_l), SimpleSSAOp::Const(0)) => {
                            inlined_history.push(SSAOp::set_p(ver_l));
                            continue;
                        }
                        (SimpleSSAOp::Version(ver_l), SimpleSSAOp::Const(rv)) => {
                            inlined_history.push(SSAOp::add_pc(ver_l, rv));
                            continue;
                        }
                        
                        
                        (SimpleSSAOp::Const(0), SimpleSSAOp::Version(ver_r)) => {
                            inlined_history.push(SSAOp::set_p(ver_r));
                            continue;
                        }
                        (SimpleSSAOp::Const(lv), SimpleSSAOp::Version(ver_r)) => {
                            inlined_history.push(SSAOp::add_pc(ver_r, lv));
                            continue;
                        }
                        _ => {}
                    }
                }
                SSAOp::sub_pp(ver_left, ver_right) => {
                    let left = history_map.get_simple_op(*ver_left);
                    let right = history_map.get_simple_op(*ver_right);

                    match (left, right) {
                        (SimpleSSAOp::Const(lv), SimpleSSAOp::Const(rv)) => {
                            inlined_history.push(SSAOp::set_c(lv.wrapping_sub(rv)));
                            continue;
                        }
                        
                        (SimpleSSAOp::Version(ver_l), SimpleSSAOp::Const(0)) => {
                            inlined_history.push(SSAOp::set_p(ver_l));
                            continue;
                        }
                        (SimpleSSAOp::Version(ver_l), SimpleSSAOp::Const(rv)) => {
                            inlined_history.push(SSAOp::sub_pc(ver_l, rv));
                            continue;
                        }
                        
                        (SimpleSSAOp::Const(lv), SimpleSSAOp::Version(ver_r)) => {
                            inlined_history.push(SSAOp::sub_cp(lv, ver_r));
                            continue;
                        }
                        _ => {}
                    }
                }
                SSAOp::mul_add(from, dest, val) => {
                    match history_map.get_simple_op(*from) {
                        SimpleSSAOp::Const(0) => {
                            match *val {
                                0 => {
                                    inlined_history.push(SSAOp::set_c(0));
                                }
                                1 => {
                                    inlined_history.push(SSAOp::set_p(*dest));
                                }
                                _ => {
                                    inlined_history.push(SSAOp::mul_pc(*dest, *val));
                                }
                            }
                            continue;
                        }
                        SimpleSSAOp::Const(_from) => {}
                        SimpleSSAOp::Version(from) => {
                            match *val {
                                0 => {
                                    inlined_history.push(SSAOp::set_p(from));
                                    continue;
                                }
                                1 => {
                                    inlined_history.push(SSAOp::add_pp(from, *dest));
                                    continue;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
            inlined_history.push(*h);
        }
        inlined_history_map.insert(*ptr, inlined_history);
    }

    inlined_history_map
}

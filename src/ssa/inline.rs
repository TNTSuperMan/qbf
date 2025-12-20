use crate::ssa::{SSAOp, PointerSSAHistory};

pub fn inline_ssa_history(history_map: PointerSSAHistory) -> PointerSSAHistory {
    let mut inlined_history_map: PointerSSAHistory = PointerSSAHistory::new();
    for (ptr, history) in history_map.iter() {
        let mut inlined_history: Vec<SSAOp> = Vec::new();
        for h in history {
            match h {
                SSAOp::raw(_raw_ptr) => {}
                SSAOp::set_c(_val) => {}
                SSAOp::set_p(ver) => {
                    inlined_history.push(history_map.get_op(*ver).unwrap());
                    continue;
                }
                SSAOp::add_pc(ver, val) => {
                    if *val == 0 {
                        inlined_history.push(SSAOp::set_p(*ver));
                        continue;
                    }
                    match history_map.get_op(*ver).unwrap() {
                        SSAOp::set_c(val2) => {
                            inlined_history.push(SSAOp::set_c(val.wrapping_add(val2)));
                            continue;
                        }
                        SSAOp::set_p(ver2) => {
                            inlined_history.push(SSAOp::add_pc(ver2, *val));
                            continue;
                        }
                        SSAOp::mul_pc(ver2, val2) => {
                            match val2 {
                                0 => {
                                    inlined_history.push(SSAOp::set_c(*val));
                                    continue;
                                }
                                1 => {
                                    inlined_history.push(SSAOp::add_pc(ver2, *val));
                                    continue;
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                SSAOp::add_pp(ver_left, ver_right) => {
                    let left = history_map.get_op(*ver_left).unwrap();
                    let right = history_map.get_op(*ver_right).unwrap();
                    println!("{:?} + {:?}", left, right);

                    match (left, right) {
                        (SSAOp::set_c(lv), SSAOp::set_c(rv)) => {
                            inlined_history.push(SSAOp::set_c(lv.wrapping_add(rv)));
                            continue;
                        }
                        
                        (SSAOp::set_p(ver_l), SSAOp::set_c(rv)) => {
                            if rv == 0 {
                                inlined_history.push(SSAOp::set_p(ver_l));
                            } else {
                                inlined_history.push(SSAOp::add_pc(ver_l, rv));
                            }
                            continue;
                        }
                        (SSAOp::set_c(lv), SSAOp::set_p(ver_r)) => {
                            if lv == 0 {
                                inlined_history.push(SSAOp::set_p(ver_r));
                            } else {
                                inlined_history.push(SSAOp::add_pc(ver_r, lv));
                            }
                            continue;
                        }

                        (SSAOp::set_c(lv), right) => {
                            if lv == 0 {
                                inlined_history.push(right);
                            } else {
                                inlined_history.push(SSAOp::add_pc(*ver_right, lv));
                            }
                            continue;
                        }
                        (left, SSAOp::set_c(rv)) => {
                            if rv == 0 {
                                inlined_history.push(left);
                            } else {
                                inlined_history.push(SSAOp::add_pc(*ver_left, rv));
                            }
                            continue;
                        }
                        _ => {}
                    }
                }
                SSAOp::mul_add(from, dest, val) => {
                    match *val {
                        0 => {
                            inlined_history.push(SSAOp::set_p(*from));
                            continue;
                        }
                        1 => {
                            inlined_history.push(SSAOp::add_pp(*from, *dest));
                            continue;
                        }
                        _ => {}
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

pub struct OperationCountMap {
    pub deopt: Vec<usize>,
    pub opt: Vec<usize>,
}
impl OperationCountMap {
    pub fn new(len: usize) -> OperationCountMap {
        OperationCountMap {
            deopt: vec![0usize; len],
            opt: vec![0usize; len],
        }
    }
}

use std::ops::RangeInclusive;

use crate::{bytecode::bytecode::Bytecode, ir::{ir::{IR, IROp}, range::{MidRange, RangeInfo}}, vm::program::Program};

fn range_to_string(range: &Option<RangeInclusive<usize>>) -> String {
    match range {
        None => "-".to_owned(),
        Some(r) => {
            let mut str = format!("{}~{}", r.start(), r.end());
            str += &" ".repeat(std::cmp::max(11 - str.len(), 0) as usize);
            str += &"|";
            str
        }
    }
}

pub fn generate_ir_trace(ir_nodes: &[IR], range: &RangeInfo) -> String {
    let mut str = String::new();
    let mut lv: usize = 0;

    if range.do_opt_first {
        str += "opt\n";
    } else {
        str += "deopt\n";
    }

    for (i, ir) in ir_nodes.iter().enumerate() {
        if let IROp::LoopEnd(..) = ir.opcode {
            lv -= 1;
        }
        if let IROp::LoopEndWithOffset(..) = ir.opcode {
            lv -= 1;
        }
        if let Some(ri) = range.map.get(&i) {
            str += &format!("{} {}{} {:?} (deopt condition: {})\n", range_to_string(&ir.source_range), "    ".repeat(lv), ir.pointer, ir.opcode, match ri {
                MidRange::None => format!("false"),
                MidRange::Negative(r) => format!("ptr < {}", r.start),
                MidRange::Positive(r) => format!("ptr >= {}", r.end),
                MidRange::Both(r) => format!("ptr < {} || ptr >= {}", r.start, r.end),
            });
        } else {
            str += &format!("{} {}{} {:?}\n", range_to_string(&ir.source_range), "    ".repeat(lv), ir.pointer, ir.opcode);
        }

        if let IROp::LoopStart(..) = ir.opcode {
            lv += 1;
        }
    }

    str
}


pub fn generate_bytecode_trace<I: FnMut() -> u8, O: FnMut(u8) -> ()>(program: &Program<I, O>) -> String {
    let mut str = String::new();
    let mut lv: usize = 0;

    for (i, b) in program.insts().iter().enumerate() {
        match b {
            Bytecode::JmpIfNotZero { .. } => lv -= 1,
            Bytecode::PositiveRangeCheckJNZ { .. } => lv -= 1,
            Bytecode::NegativeRangeCheckJNZ { .. } => lv -= 1,
            Bytecode::BothRangeCheckJNZ { .. } => lv -= 1,
            _ => {}
        }
        str += &format!("{}\t{}\t{}{:?}\n", (program.ocm.deopt[i].wrapping_add(1) as f64).log2().floor(), (program.ocm.opt[i].wrapping_add(1) as f64).log2().floor(), "    ".repeat(lv), b);
        match b {
            Bytecode::JmpIfZero { .. } => lv += 1,
            _ => {}
        }
    }
    str += &format!("step count(deopt/opt): {}/{}", program.ocm.deopt.iter().fold(0, |acc, e| acc + e), program.ocm.opt.iter().fold(0, |acc, e| acc + e));

    str
}

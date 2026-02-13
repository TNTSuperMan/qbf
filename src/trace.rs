#[cfg(feature = "debug")]
pub struct OperationCountMap {
    pub deopt: Vec<usize>,
    pub opt: Vec<usize>,
}
#[cfg(feature = "debug")]
impl OperationCountMap {
    pub fn new(len: usize) -> OperationCountMap {
        OperationCountMap {
            deopt: vec![0usize; len],
            opt: vec![0usize; len],
        }
    }
}

#[cfg(not(feature = "debug"))]
pub struct OperationCountMap;
#[cfg(not(feature = "debug"))]
impl OperationCountMap {
    pub fn new(_len: usize) -> OperationCountMap {
        OperationCountMap
    }
}

#[cfg(feature = "debug")]
use crate::{ir::IR, range::RangeInfo};

#[cfg(feature = "debug")]
pub fn generate_ir_trace(ir_nodes: &[IR], range: &RangeInfo) -> String {
    let mut str = String::new();
    let mut lv: usize = 0;

    if range.do_opt_first {
        str += "opt\n";
    } else {
        str += "deopt\n";
    }

    for (i, ir) in ir_nodes.iter().enumerate() {
        use crate::ir::IROp;

        if let IROp::LoopEnd(_) = ir.opcode {
            lv -= 1;
        }
        if let IROp::LoopEndWithOffset(_, _) = ir.opcode {
            lv -= 1;
        }
        if let Some(ri) = range.map.get(&i) {
            use crate::range::MidRange;

            str += &format!("{}{} {:?} (deopt condition: {})\n", "    ".repeat(lv), ir.pointer, ir.opcode, match ri {
                MidRange::None => format!("false"),
                MidRange::Negative(r) => format!("ptr < {}", r.start),
                MidRange::Positive(r) => format!("ptr >= {}", r.end),
                MidRange::Both(r) => format!("ptr < {} || ptr >= {}", r.start, r.end),
            });
        } else {
            str += &format!("{}{} {:?}\n", "    ".repeat(lv), ir.pointer, ir.opcode);
        }

        if let IROp::LoopStart(_) = ir.opcode {
            lv += 1;
        }
    }

    str
}

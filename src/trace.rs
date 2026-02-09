#[cfg(feature = "debug")]
pub struct OperationCountMap (pub Vec<usize>);
#[cfg(feature = "debug")]
impl OperationCountMap {
    pub fn new(len: usize) -> OperationCountMap {
        OperationCountMap(vec![0usize; len])
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
            use crate::range::MemoryRange;

            str += &format!("{}{} {:?} (deopt condition: {})\n", "    ".repeat(lv), ir.pointer, ir.opcode, match ri {
                MemoryRange::None => format!("false"),
                MemoryRange::Positive(x) => format!("ptr >= {x}"),
                MemoryRange::Negative(x) => format!("ptr < {x}"),
                MemoryRange::Both { positive, negative } => format!("ptr >= {positive} || ptr < {negative}"),
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

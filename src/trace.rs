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
use std::collections::HashMap;

#[cfg(feature = "debug")]
use crate::{ir::IR, range::RangeInfo};

#[cfg(feature = "debug")]
pub fn generate_ir_trace(ir_nodes: &[IR], range: &HashMap<usize, RangeInfo>) -> String {
    let mut str = String::new();
    let mut lv: usize = 0;

    for (i, ir) in ir_nodes.iter().enumerate() {
        use crate::ir::IROp;

        if let IROp::LoopEnd(_) = ir.opcode {
            lv -= 1;
        }
        if let IROp::LoopEndWithOffset(_, _) = ir.opcode {
            lv -= 1;
        }
        if let Some(ri) = range.get(&i) {
            use crate::range::RangeInfoKind;

            str += &format!("{}{} {:?} ({})\n", "    ".repeat(lv), ir.pointer, ir.opcode, match ri.kind {
                RangeInfoKind::Positive => format!("ptr < {}", (ri.pointer - ri.val) as i16 as u16),
                RangeInfoKind::Negative => format!("ptr >= {}", (ri.pointer - ri.val) as i16 as u16),
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

use crate::{bytecode::{Bytecode, OpCode}};

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

fn indent(level: usize) -> String {
    vec![""; level+1].join("    ")
}

pub fn instructions_to_string(bytecodes: &[Bytecode], m: &OperationCountMap) -> String {
    let mut strings: Vec<String> = vec![];
    let mut lv: usize = 0;
    for (i, b) in bytecodes.iter().enumerate() {
        if OpCode::LoopEnd == b.opcode {
            lv -= 1;
        }
        if OpCode::LoopEndWithOffset == b.opcode {
            lv -= 1;
        }
        let ind = indent(lv);
        if OpCode::LoopStart == b.opcode {
            lv += 1;
        }
        let t = match b.opcode {
            OpCode::Breakpoint => format!("@BREAKPOINT at {}", b.ptr),
            OpCode::Add => format!("[{}] += {}", b.ptr, b.val),
            OpCode::Set => format!("[{}] = {}", b.ptr, b.val),
            OpCode::Shift => format!("Shift {} from {}", b.ptr2, b.ptr),
            OpCode::MulStart => format!("MulStart [{}]", b.ptr),
            OpCode::Mul => format!("[{}] += m * {}", b.ptr, b.val),
            OpCode::AddFromMemory => format!("[{}] += m", b.ptr),
            OpCode::SingleMul => format!("[{}] += [{}] * {}", b.ptr, b.ptr2, b.val),
            OpCode::SingleAddFM => format!("[{}] += [{}]", b.ptr, b.ptr2),
            OpCode::In => format!("[{}] = In()", b.ptr),
            OpCode::Out => format!("Out [{}]", b.ptr),
            OpCode::LoopStart => format!("loop [{}] {{", b.ptr),
            OpCode::LoopEnd => format!("}} stable"),
            OpCode::LoopEndWithOffset => format!("}} offset({})", b.ptr2),
            OpCode::End => format!("End"),
        };
        #[cfg(feature = "debug")] {
            strings.push(format!("{}\t{}{}", ((m.0[i] as f64).ln() as usize), ind, t));
        }
        #[cfg(not(feature = "debug"))] {
            strings.push(format!("\t{}{}", ind, t));
        }
    }
    strings.join("\n")
}

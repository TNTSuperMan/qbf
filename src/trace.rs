use crate::parser::{InstOp, Instruction};

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

fn dests_to_string(dests: Vec<(isize, u8)>) -> String {
    let mut strs: Vec<String> = Vec::new();
    for (ptr, val) in dests {
        strs.push(format!("[{}]**{}", ptr, val))
    }
    strs.join(", ")
}

fn indent(level: usize) -> String {
    vec![""; level+1].join("    ")
}

pub fn instructions_to_string(instructions: Vec<Instruction>, m: OperationCountMap) -> String {
    let mut strings: Vec<String> = Vec::new();
    let mut lv: usize = 0;
    for (i, Instruction { opcode, pointer }) in instructions.iter().enumerate() {
        if let InstOp::LoopEnd(_) = opcode {
            lv -= 1;
        }
        if let InstOp::LoopEndWithOffset(_, _) = opcode {
            lv -= 1;
        }
        let ind = indent(lv);
        let t = match opcode {
            InstOp::Breakpoint => format!("@BREAKPOINT! at {}", pointer),
            InstOp::Add(val) => format!("[{}] += {}", pointer, val),
            InstOp::Set(val) => format!("[{}] = {}", pointer, val),
            InstOp::Shift(diff) => format!("Shift({})", diff),
            InstOp::Mul(source, val) => format!("[{}] += [{}] * {}", pointer, source, val),
            InstOp::MulAndSetZero(dests) => format!("MulAndSetZero [{}] => {}", pointer,dests_to_string(dests.clone())),
            InstOp::MulAndSetZeroTo(source, dests) => format!("MulAndSetZeroTo [{}] = 0, [{}] => {}", pointer,source,dests_to_string(dests.clone())),
            InstOp::Out => format!("Out {}", pointer),
            InstOp::In => format!("In {}", pointer),
            InstOp::LoopStart(start) => { lv += 1; format!("loop {{ -> {} [{}]", start, pointer) },
            InstOp::LoopEnd(end) => format!("}} <- {} [{}] STABLE", end, pointer),
            InstOp::LoopEndWithOffset(end, off) => format!("}} <- {} [{}] unstable({})", end, pointer, off) ,
            InstOp::End => format!("{}End", indent(lv)),
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

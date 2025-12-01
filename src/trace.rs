use crate::ir::{IROp, IR};

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

pub fn instructions_to_string(instructions: Vec<IR>, m: OperationCountMap) -> String {
    let mut strings: Vec<String> = Vec::new();
    let mut lv: usize = 0;
    for (i, IR { opcode, pointer }) in instructions.iter().enumerate() {
        if let IROp::LoopEnd(_) = opcode {
            lv -= 1;
        }
        if let IROp::LoopEndWithOffset(_, _) = opcode {
            lv -= 1;
        }
        let ind = indent(lv);
        let t = match opcode {
            IROp::Breakpoint => format!("@BREAKPOINT! at {}", pointer),
            IROp::Add(val) => format!("[{}] += {}", pointer, val),
            IROp::Set(val) => format!("[{}] = {}", pointer, val),
            IROp::Shift(diff) => format!("Shift({})", diff),
            IROp::MulAndSetZero(dests) => format!("MulAndSetZero [{}] => {}", pointer,dests_to_string(dests.clone())),
            IROp::MulAndSetZeroTo(source, dests) => format!("MulAndSetZeroTo [{}] = 0, [{}] => {}", pointer,source,dests_to_string(dests.clone())),
            IROp::Out => format!("Out {}", pointer),
            IROp::In => format!("In {}", pointer),
            IROp::LoopStart(start) => { lv += 1; format!("loop {{ -> {} [{}]", start, pointer) },
            IROp::LoopEnd(end) => format!("}} <- {} [{}] STABLE", end, pointer),
            IROp::LoopEndWithOffset(end, off) => format!("}} <- {} [{}] unstable({})", end, pointer, off) ,
            IROp::End => format!("{}End", indent(lv)),
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

use crate::parser::{InstOp, Instruction};

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

pub fn instructions_to_string(instructions: Vec<Instruction>) -> String {
    let mut strings: Vec<String> = Vec::new();
    let mut lv: usize = 0;
    for Instruction { opcode, pointer } in instructions {
        strings.push(match opcode {
            InstOp::Breakpoint => format!("{}@BREAKPOINT! at {}", indent(lv), pointer),
            InstOp::Add(val) => format!("{}[{}] += {}", indent(lv), pointer, val),
            InstOp::Set(val) => format!("{}[{}] = {}", indent(lv), pointer, val),
            InstOp::Shift(diff) => format!("{}Shift({})", indent(lv), diff),
            InstOp::MulAndSetZero(dests) => format!("{}MulAndSetZero [{}] => {}", indent(lv),pointer,dests_to_string(dests)),
            InstOp::MulAndSetZeroTo(source, dests) => format!("{}MulAndSetZeroTo [{}] = 0, [{}] => {}", indent(lv),pointer,source,dests_to_string(dests)),
            InstOp::Out => format!("{}Out {}", indent(lv), pointer),
            InstOp::In => format!("{}In {}", indent(lv), pointer),
            InstOp::LoopStart(start) => { let i = indent(lv); lv += 1; format!("{}loop {{ -> {} [{}]", i, start, pointer) },
            InstOp::LoopEnd(end) => { lv -= 1; format!("{}}} <- {} [{}] STABLE", indent(lv), end, pointer) },
            InstOp::LoopEndWithOffset(end, off) => { lv -= 1; format!("{}}} <- {} [{}] unstable({})", indent(lv), end, pointer, off) },
        });
    }
    strings.join("\n")
}

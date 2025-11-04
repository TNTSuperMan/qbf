use crate::instruction::Instruction;

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
    for inst in instructions {
        strings.push(match inst {
            Instruction::Breakpoint => format!("{}@BREAKPOINT!", indent(lv)),
            Instruction::Add(ptr, val) => format!("{}[{}] += {}", indent(lv), ptr, val),
            Instruction::Set(ptr, val) => format!("{}[{}] = {}", indent(lv), ptr, val),
            Instruction::MulAndSetZero(src, dests) => format!("{}MulAndSetZero [{}] => {}", indent(lv),src,dests_to_string(dests)),
            Instruction::Out(ptr) => format!("{}Out {}", indent(lv), ptr),
            Instruction::In(ptr) => format!("{}In {}", indent(lv), ptr),
            Instruction::LoopStart(addr, ptr, is_stable) => { let i = indent(lv); lv += 1; format!("{}loop {{ -> {} [{}] {}", i, addr, ptr, if is_stable { "STABLE" } else { "unstable" }) },
            Instruction::LoopEnd(addr, ptr, is_stable) => { lv -= 1; format!("{}}} <- {} [{}] {}", indent(lv), addr, ptr, if is_stable { "STABLE" } else { "unstable" }) },
        });
    }
    strings.join("\n")
}

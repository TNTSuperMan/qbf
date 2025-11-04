use crate::instruction::Instruction;

pub fn instructions_to_string(instructions: Vec<Instruction>) -> String {
    let mut strings: Vec<String> = Vec::new();
    for inst in instructions {
        strings.push(match inst {
            Instruction::Add(ptr, val) => format!("Add(to: {}, val: {})", ptr, val),
            Instruction::Set(ptr, val) => format!("Set(to: {}, val: {})", ptr, val),
            Instruction::MulAndSetZero(src, _dests) => format!("MulAndSetZero(src: {})",src),
            Instruction::Out(ptr) => format!("Out(to: {})", ptr),
            Instruction::In(ptr) => format!("In(to: {})", ptr),
            Instruction::LoopStart(addr, ptr, is_stable) => format!("LoopStart(addr: {}, ptr{}, {})", addr, ptr, if is_stable { "stable" } else { "unstable" }),
            Instruction::LoopEnd(addr, ptr, is_stable) => format!("LoopEnd(addr: {}, ptr: {}, {})", addr, ptr, if is_stable { "stable" } else { "unstable" }),
        });
    }
    strings.join("\n")
}

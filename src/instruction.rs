use crate::parser::BFNode;

#[derive(Clone)]
pub enum Instruction {
    Breakpoint,

    Add (isize, u8),
    Set (isize, u8),

    MulAndSetZero (
        isize,
        Vec<(isize, u8)>,
    ),

    In(isize),
    Out(isize),

    LoopStart(usize, isize, bool), // cond, is_ptr_stable
    LoopEnd(usize, isize, bool),
}

pub struct Hints {
}

pub fn ast_to_instructions(ast: Vec<BFNode>) -> (Vec<Instruction>, Hints) {
    let mut instructions: Vec<Instruction> = Vec::new();
    let mut loop_stack: Vec<(
        usize, // address
        isize, // pointer
    )> = Vec::new();
    let hints = Hints {
    };
    let mut pointer: isize = 0;

    let mut masz_dests: Option<Vec<(isize, u8)>> = None; // masz: MulAndSetZero

    for node in ast.iter() {
        match *node {
            BFNode::Breakpoint => { instructions.push(Instruction::Breakpoint) }
            BFNode::Add(val) => {
                if let Some(dests) = masz_dests.as_mut() {
                    dests.push((pointer, val));
                }
                instructions.push(Instruction::Add(pointer, val));
            }
            BFNode::Set(val) => {
                if masz_dests.is_some() { masz_dests = None }
                instructions.push(Instruction::Set(pointer, val));
            }
            BFNode::To(to) => {
                pointer += to;
            }
            BFNode::Out => {
                if masz_dests.is_some() { masz_dests = None }
                instructions.push(Instruction::Out(pointer));
            }
            BFNode::In => {
                if masz_dests.is_some() { masz_dests = None }
                instructions.push(Instruction::In(pointer));
            }
            BFNode::LoopStart(end) => {
                loop_stack.push((instructions.len(), pointer));
                instructions.push(Instruction::LoopStart(usize::MAX, pointer, false));
                if let BFNode::LoopEnd(_, true) = ast[end] {
                    masz_dests = Some(Vec::new());
                }
            }
            BFNode::LoopEnd(_start, _is_flat) => {
                let (start, start_ptr) = loop_stack.pop().unwrap();
                let end = instructions.len();

                let is_ptr_stable = start_ptr == pointer;

                if let Some(ref dests_raw) = masz_dests {
                    let mut dests = dests_raw.clone();
                    let decr_pos = dests.iter().position(|&dest| dest == (pointer, 255));
                    if is_ptr_stable && decr_pos.is_some() {
                        dests.remove(decr_pos.unwrap());
                        if !dests.iter().any(|&(ptr, _)| ptr == pointer) {
                            instructions.truncate(start);
                            instructions.push(Instruction::MulAndSetZero(pointer, dests.to_vec()));
                            continue;
                        }
                    }
                }

                instructions[start] = Instruction::LoopStart(end, start_ptr, is_ptr_stable);
                instructions.push(Instruction::LoopEnd(start, pointer, is_ptr_stable));

                if !is_ptr_stable {
                    pointer = start_ptr;
                }
            }
        }
    }

    (instructions, hints)
}

use crate::{tokenizer::Token, vm::BFVM};

pub fn run(vm: &mut BFVM, tokens: Vec<Token>) {
    let len = tokens.len();
    while vm.pc < len {
        match tokens[vm.pc] {
            Token::Add(count) => vm.memory[vm.pointer] = vm.memory[vm.pointer].wrapping_add(count),
            Token::RelativeTo(to) => vm.pointer = ((vm.pointer as isize) + to) as usize,
            Token::In => vm.memory[vm.pointer] = vm.input.as_ref()(),
            Token::Out => vm.output.as_ref()(vm.memory[vm.pointer]),
            Token::LoopStart(end) => {
                if vm.memory[vm.pointer] == 0 {
                    vm.pc = end;
                }
            },
            Token::LoopEnd(start) => {
                if vm.memory[vm.pointer] != 0 {
                    vm.pc = start;
                }
            }
        }
        vm.pc += 1;
    }
}

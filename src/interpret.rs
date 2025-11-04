use crate::{instruction::{Hints, Instruction}, vm::BFVM};

pub fn run(vm: &mut BFVM, instrs: Vec<Instruction>, _hints: Hints) {
    let mut loop_ptr_stack: Vec<isize> = Vec::new();
    let mut offset: isize = 0;
    let len = instrs.len();

    while vm.pc < len {
        println!("PC: {} offset: {}", vm.pc, offset);
        match &instrs[vm.pc] {
            Instruction::Breakpoint => {
                // 標準出力と分けるだけ、エラーじゃない
                eprint!("PC: {}, ", vm.pc);
                for i in 0..50 {
                    eprint!("{:02x} ", vm.memory[i]);
                }
                eprintln!();
            }

            Instruction::Add(p, val) => {
                let ptr = (*p + offset) as usize;
                vm.memory[ptr] = vm.memory[ptr].wrapping_add(*val);
            }
            Instruction::Set(p, val) => {
                let ptr = (*p + offset) as usize;
                vm.memory[ptr] = *val;
            }

            Instruction::MulAndSetZero(source, dests) => {
                let source_ptr = (*source + offset) as usize;
                let source_val = vm.memory[source_ptr];
                for (dest_p, m) in dests {
                    let dest_ptr = (*dest_p + offset) as usize;
                    vm.pointer = dest_ptr;
                    vm.memory[dest_ptr] = vm.memory[dest_ptr].wrapping_add(source_val * m);
                }
                vm.memory[source_ptr] = 0;
            }

            Instruction::In(p) => {
                let ptr = (*p + offset) as usize;
                vm.memory[ptr] = vm.input.as_ref()();
            }
            Instruction::Out(p) => {
                let ptr = (*p + offset) as usize;
                vm.output.as_ref()(vm.memory[ptr]);
            }

            Instruction::LoopStart(end, cond, is_ptr_stable) => {
                if vm.memory[(*cond + offset) as usize] == 0 {
                    vm.pc = *end;
                } else if !is_ptr_stable {
                    loop_ptr_stack.push(*cond);
                }
            }
            Instruction::LoopEnd(start, cond, is_ptr_stable) => {
                if !is_ptr_stable {
                    let start_ptr = loop_ptr_stack.pop().unwrap();
                    offset += *cond - start_ptr;
                }
                vm.pc = *start;
                continue; // LoopStartに処理を飛ばすため、PCインクリメントを回避
            }
        }
        vm.pc += 1;
    }
}

use crate::{instruction::{Hints, Instruction}, vm::BFVM};

pub fn run(vm: &mut BFVM, instrs: Vec<Instruction>, _hints: Hints) {
    let mut offset: isize = 0;
    let len = instrs.len();

    while vm.pc < len {
        match &instrs[vm.pc] {
            Instruction::Breakpoint(ptr) => {
                // 標準出力と分けるだけ、エラーじゃない
                eprintln!("PC: {}, PTR: {}, ", vm.pc, offset + ptr);
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
                if source_val != 0 {
                    for (dest_p, m) in dests {
                        let dest_ptr = (*dest_p + offset) as usize;
                        vm.memory[dest_ptr] = vm.memory[dest_ptr].wrapping_add(source_val * m);
                    }
                    vm.memory[source_ptr] = 0;
                }
            }

            Instruction::In(p) => {
                let ptr = (*p + offset) as usize;
                vm.memory[ptr] = vm.input.as_ref()();
            }
            Instruction::Out(p) => {
                let ptr = (*p + offset) as usize;
                vm.output.as_ref()(vm.memory[ptr]);
            }

            Instruction::LoopStart(end, cond, _is_ptr_stable) => {
                if vm.memory[(*cond + offset) as usize] == 0 {
                    vm.pc = *end;
                }
            }
            Instruction::LoopEnd(start, cond, is_ptr_stable) => {
                if !is_ptr_stable {
                    if let Instruction::LoopStart(_end, start_cond, _is_ptr_stable) = instrs[*start] {
                        offset += *cond - start_cond;
                    } else {
                        unreachable!("対になってるループ先頭がLoopStart以外な訳がないよね");
                    }
                }
                vm.pc = *start;
                continue; // LoopStartに処理を飛ばすため、PCインクリメントを回避
            }
        }
        vm.pc += 1;
    }
}

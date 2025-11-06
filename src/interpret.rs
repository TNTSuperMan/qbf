use crate::{parser::{InstOp, Instruction}, vm::BFVM};

pub fn step(vm: &mut BFVM) {
    let Instruction { opcode, pointer: ptr } = &vm.insts[vm.pc];
    match opcode {
        InstOp::Breakpoint => {
            // 標準出力と分けるだけ、エラーじゃない
            eprintln!("PC: {}, PTR: {}, ", vm.pc, vm.offset + ptr);
        }

        InstOp::Add(val) => {
            let ptr = (ptr + vm.offset) as usize;
            vm.memory[ptr] = vm.memory[ptr].wrapping_add(*val);
        }
        InstOp::Set(val) => {
            let ptr = (ptr + vm.offset) as usize;
            vm.memory[ptr] = *val;
        }

        InstOp::MulAndSetZero(dests) => {
            let source_ptr = (ptr + vm.offset) as usize;
            let source_val = vm.memory[source_ptr];
            if source_val != 0 {
                for (dest_p, m) in dests {
                    let dest_ptr = (*dest_p + vm.offset) as usize;
                    vm.memory[dest_ptr] = vm.memory[dest_ptr].wrapping_add(source_val.wrapping_mul(*m));
                }
                vm.memory[source_ptr] = 0;
            }
        }

        InstOp::In => {
            let ptr = (ptr + vm.offset) as usize;
            vm.memory[ptr] = vm.input.as_ref()();
        }
        InstOp::Out => {
            let ptr = (ptr + vm.offset) as usize;
            vm.output.as_ref()(vm.memory[ptr]);
        }

        InstOp::LoopStart(end) => {
            if vm.memory[(ptr + vm.offset) as usize] == 0 {
                vm.pc = *end;
            }
        }
        InstOp::LoopEnd(start) => {
            if vm.memory[(ptr + vm.offset) as usize] != 0 {
                vm.pc = *start;
            }
        }
        InstOp::LoopEndWithOffset(start, off) => {
            if vm.memory[(ptr + vm.offset) as usize] != 0 {
                vm.pc = *start;
            }
            vm.offset += off;
        }
    }
    vm.pc += 1;
}

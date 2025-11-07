use crate::{io::{input, output}, parser::{InstOp, Instruction}, vm::BFVM};

pub fn step(vm: &mut BFVM) {
    let Instruction { opcode, pointer } = &vm.insts[vm.pc];
    let ptr = (pointer + vm.offset) as usize;
    match opcode {
        InstOp::Breakpoint => {
            // 標準出力と分けるだけ、エラーじゃない
            eprintln!("PC: {}, PTR: {}, ", vm.pc, ptr);
        }

        InstOp::Add(val) => {
            vm.memory[ptr] = vm.memory[ptr].wrapping_add(*val);
        }
        InstOp::Set(val) => {
            vm.memory[ptr] = *val;
        }

        InstOp::Shift(diff) => {
            while vm.memory[(pointer + vm.offset) as usize] != 0 {
                vm.offset += diff;
            }
        }
        InstOp::MulAndSetZero(dests) => {
            let source_val = vm.memory[ptr];
            if source_val != 0 {
                for (dest_p, m) in dests {
                    let dest_ptr = (*dest_p + vm.offset) as usize;
                    vm.memory[dest_ptr] = vm.memory[dest_ptr].wrapping_add(source_val.wrapping_mul(*m));
                }
                vm.memory[ptr] = 0;
            }
        }
        InstOp::MulAndSetZeroTo(source, dests) => {
            let source_val = vm.memory[(source + vm.offset) as usize].wrapping_add(vm.memory[ptr]);
            if source_val != 0 {
                for (dest_p, m) in dests {
                    let dest_ptr = (*dest_p + vm.offset) as usize;
                    vm.memory[dest_ptr] = vm.memory[dest_ptr].wrapping_add(source_val.wrapping_mul(*m));
                }
                vm.memory[ptr] = 0;
            }
        }

        InstOp::In => {
            vm.memory[ptr] = input();
        }
        InstOp::Out => {
            output(vm.memory[ptr]);
        }

        InstOp::LoopStart(end) => {
            if vm.memory[ptr] == 0 {
                vm.pc = *end;
            }
        }
        InstOp::LoopEnd(start) => {
            if vm.memory[ptr] != 0 {
                vm.pc = *start;
            }
        }
        InstOp::LoopEndWithOffset(start, off) => {
            if vm.memory[ptr] != 0 {
                vm.pc = *start;
            }
            vm.offset += off;
        }
    }
    vm.pc += 1;
}

use crate::{inst::Instruction, vm::BFVM};

pub fn run(vm: &mut BFVM, instrs: Vec<Instruction>) {
    let len = instrs.len();
    while vm.pc < len {
        match &instrs[vm.pc] {
            Instruction::Nop => {}

            Instruction::Add(p, val) => {
                let point = p.use_point(vm);
                vm.memory[point] = vm.memory[point].wrapping_add(*val);
            }
            Instruction::Set(p, val) => {
                let point = p.use_point(vm);
                vm.memory[point] = *val;
            }

            Instruction::MulAndSetZero(source, dests) => {
                let source_ptr = source.use_point(vm);
                let source_val = vm.memory[source_ptr];
                for (dest_p, m) in dests {
                    let dest_point = dest_p.use_point(vm);
                    vm.pointer = dest_point;
                    vm.memory[dest_point] = vm.memory[dest_point].wrapping_add(source_val * m);
                }
                vm.memory[source_ptr] = 0;
            }

            Instruction::To(p) => {
                p.use_point(vm);
            }

            Instruction::In(p) => {
                let point = p.use_point(vm);
                vm.memory[point] = vm.input.as_ref()();
            }
            Instruction::Out(p) => {
                let point = p.use_point(vm);
                vm.output.as_ref()(vm.memory[point]);
            }

            Instruction::LoopStart(jump) => {
                if vm.memory[vm.pointer] == 0 {
                    vm.pc = *jump;
                }
            }
            Instruction::LoopEnd(jump, opt) => {
                if let Some(assumpt) = opt.pointer_assumption {
                    if assumpt != vm.pointer {
                        todo!("deopt & continue");
                    }
                }
                if vm.memory[vm.pointer] != 0 {
                    vm.pc = *jump;
                }
            }
        }
        vm.pc += 1;
    }
}

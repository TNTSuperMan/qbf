use crate::{ir::IR, ssa::{inline::inline_ssa_history, r#loop::{detect_ssa_loop, try_2step_loop}, parse::build_ssa_from_ir}};

pub mod structs;
pub mod parse;
pub mod inline;
pub mod to_ir;
pub mod r#loop;

pub fn optimize_loop(children: &[IR], start_pc: usize, loop_pointer: isize) -> Result<Vec<IR>, ()> {
    let children_ssa = build_ssa_from_ir(children).ok_or(())?;

    let inlined_children_ssa = inline_ssa_history(&children_ssa, false);

    if let Some((loop_ptr, loop_ssa)) = detect_ssa_loop(&inlined_children_ssa) {
        if loop_ptr != loop_pointer {
            return Err(());
        }

        if let Some((second_loop_ssa, const_cells)) = try_2step_loop(&loop_ssa) {
            todo!();
        } else {
            todo!();
        }
    }
    todo!();

    Err(())
}

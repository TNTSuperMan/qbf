const TAPE_LENGTH: usize = 65536;

pub mod error;
mod ir;
mod bytecode;
mod vm;
mod trace;

mod brainrot;

pub use crate::brainrot::{Brainrot, BrainrotInit};

pub mod advance {
    pub use crate::ir::*;
    pub use crate::bytecode::*;
    pub use crate::vm::*;
    pub use crate::trace::*;
}

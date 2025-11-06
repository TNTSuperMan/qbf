pub struct BFVM {
    pub pc: usize,
    pub memory: Vec<u8>,
    pub output: Box<dyn Fn(u8)>,
    pub input: Box<dyn Fn() -> u8>,
}

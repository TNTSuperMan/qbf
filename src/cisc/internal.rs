#[derive(Debug)]
pub enum Tier {
    Deopt,
    Opt,
}

pub enum InterpreterResult {
    End,
    ToggleTier(Tier),
}

#[inline(always)]
pub fn positive_out_of_range(r: u8, pointer: usize) -> bool {
    (r as i8 as i16 as u16 as isize) <= pointer as isize
}

#[inline(always)]
pub fn negative_out_of_range(r: u8, pointer: usize) -> bool {
    (r as i8 as i16 as u16 as isize) > pointer as isize
}

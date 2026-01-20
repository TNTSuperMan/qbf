pub enum Tier {
    Deopt,
    Opt,
}

pub enum InterpreterResult {
    End,
    ToggleTier(Tier),
}

#[inline(always)]
pub fn u32_to_delta_and_val(val: u32) -> (i16, u8) {
    (
        (val & 0xFFFF) as u16 as i16,
        (val >> 16) as u8,
    )
}

#[inline(always)]
pub fn u32_to_delta_and_two_val(val: u32) -> (i16, u8, u8) {
    (
        (val & 0xFFFF) as u16 as i16,
        ((val >> 16) & 0xFF) as u8,
        (val >> 24)  as u8,
    )
}

#[inline(always)]
pub fn u32_to_two_delta(val: u32) -> (i16, i16) {
    (
        (val & 0xFFFF) as u16 as i16,
        (val >> 16) as u16 as i16,
    )
}

#[inline(always)]
pub fn positive_out_of_range(r: u8, pointer: usize) -> bool {
    (r as i8 as i16 as u16 as isize) <= pointer as isize
}

#[inline(always)]
pub fn negative_out_of_range(r: u8, pointer: usize) -> bool {
    (r as i8 as i16 as u16 as isize) > pointer as isize
}

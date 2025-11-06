pub extern "C" fn output(val: u8) {
    print!("{}", val as char);
}

pub extern "C" fn input() -> u8 {
    0 // TODO
}

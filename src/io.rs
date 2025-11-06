pub extern "C" fn output(val: u8) {
    print!("{}", val);
}

pub extern "C" fn input() -> u8 {
    0 // TODO
}

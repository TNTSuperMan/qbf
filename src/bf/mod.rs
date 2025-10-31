use crate::bf::token::tokenize;

mod token;

pub fn run(code: &str) {
    let _token = tokenize(code);
}

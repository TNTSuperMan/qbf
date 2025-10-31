use crate::tokenizer::tokenize;

mod tokenizer;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() == 2 {
        match std::fs::read_to_string(args[1].clone()) {
            Err(e) => {
                eprintln!("Error: {}", e);
            }
            Ok(code) => {
                let token = tokenize(&code);
            }
        }
    } else {
        println!("usage: {} <file>", args[0]);
    }
}

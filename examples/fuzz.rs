use std::{collections::HashMap, fs, io::{Read, Write}, ops::Sub, process::{Command, Stdio}, time::Instant};
use clap::Parser;

#[derive(Parser, Debug)] #[command(name = "tool")] struct Args {
    #[arg(value_name = "FILE")]
    file: String,
    
    #[arg(short, long)]
    debug: bool,
}
fn main() {
    let args = Args::parse();
    if args.debug {
        let f = fs::read_to_string(&args.file).unwrap();
        println!("{:?}", exec_lite_bf(&f, usize::MAX));
    } else {
        let start = Instant::now();
        let res = run_brainrot(&args.file, "+++++[->+++++<]>.", 500000).unwrap();
        let end = Instant::now();
        let f = exec_lite_bf("+++++[->+++++<]>.", 500000).unwrap();
        res.eq(&f);
        println!("{:?}", res);
        println!("{:?}", f);
        println!("{:?}", end.sub(start));
    }
}


#[derive(Debug)] enum LiteBfErr { OutOfRange, SyntaxError, MaxStep }
fn exec_lite_bf(code: &str, max_step: usize) -> Result<Vec<u8>, LiteBfErr> {
    let jumptable = {
        let mut jumptable: HashMap<usize, usize> = HashMap::new();
        let mut stack: Vec<usize> = vec![];
        for (i, c) in code.chars().enumerate() {
            if c == '[' {
                stack.push(i);
            } else if c == ']' {
                let start = stack.pop().ok_or_else(|| LiteBfErr::SyntaxError)?;
                jumptable.insert(start, i+1);
                jumptable.insert(i, start+1);
            }
        }
        jumptable
    };
    let mut memory = [0u8; 65536];
    let mut stdout: Vec<u8> = vec![];
    let mut step = 0usize;
    let mut dp = 0usize;
    let mut pc = 0usize;
    let code_chars: Vec<char> = code.chars().collect();

    loop {
        step += 1;
        if step > max_step {
            return Err(LiteBfErr::MaxStep);
        }
        if pc >= code.len() {
            return Ok(stdout);
        }
        match code_chars[pc] {
            '+' => {
                let cell = memory.get_mut(dp).ok_or_else(|| LiteBfErr::OutOfRange)?;
                *cell = cell.wrapping_add(1);
            }
            '-' => {
                let cell = memory.get_mut(dp).ok_or_else(|| LiteBfErr::OutOfRange)?;
                *cell = cell.wrapping_sub(1);
            }
            '<' => dp = dp.wrapping_sub(1),
            '>' => dp = dp.wrapping_add(1),
            '[' => {
                if *memory.get(dp).ok_or_else(|| LiteBfErr::OutOfRange)? == 0 {
                    pc = *jumptable.get(&pc).unwrap();
                    continue;
                }
            }
            ']' => {
                if *memory.get(dp).ok_or_else(|| LiteBfErr::OutOfRange)? != 0 {
                    pc = *jumptable.get(&pc).unwrap();
                    continue;
                }
            }
            '.' => {
                stdout.push(*memory.get(dp).ok_or_else(|| LiteBfErr::OutOfRange)?);
            }
            _ => {}
        }
        pc += 1;
    }
}


#[derive(Debug)] enum BrainrotErr { OutOfRange, Timeout, UnknownError(Vec<u8>), Panic(Vec<u8>) }
fn run_brainrot(runtime: &str, code: &str, timeout: usize) -> Result<Vec<u8>, BrainrotErr> { // 500000
    let mut process = Command::new(runtime)
        .arg("/dev/stdin")
        .arg(format!("--timeout={timeout}"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(stdin) = process.stdin.as_mut() {
        stdin.write(code.as_bytes()).unwrap();
    }

    let exit = process.wait().unwrap();

    match exit.code().unwrap() {
        0 => {
            let arr: Result<Vec<u8>, std::io::Error> = process.stdout.unwrap().bytes().collect();
            Ok(arr.unwrap())
        }
        1 => {
            Err(BrainrotErr::OutOfRange)
        }
        2 => {
            Err(BrainrotErr::Timeout)
        }
        101 => {
            let arr: Result<Vec<u8>, std::io::Error> = process.stderr.unwrap().bytes().collect();
            Err(BrainrotErr::Panic(arr.unwrap()))
        }
        _ => {
            let arr: Result<Vec<u8>, std::io::Error> = process.stderr.unwrap().bytes().collect();
            Err(BrainrotErr::UnknownError(arr.unwrap()))
        }
    }
}

mod lexer;

use std::{env, fs, process};
use std::path::Path;
use std::time::Instant;
use lexer::Lexer;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Not enough arguments supplied.\nUsage: litc file.lit");
        process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    if input_path.extension().and_then(|e| e.to_str()) != Some("lit") {
        eprintln!("Error: file does not have .lit extension");
        process::exit(1);
    }

    let src = fs::read_to_string(input_path).unwrap_or_else(|e| {
        eprintln!("Cannot read {:?}: {}", input_path, e);
        process::exit(1);
    });

    let now = Instant::now();

    let tokens = Lexer::new(&src).tokenize();

    //tokens.iter().for_each(|token| { println!("{:?}", token.kind) });
    println!("Took: {:?}", now.elapsed());
}

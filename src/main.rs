mod lexer;
mod ast;
mod parser;
mod codegen;
mod analyzer;
mod utils;

use lexer::Lexer;
use parser::Parser;
use analyzer::analyze;
//use ast::*;

use std::env;
use std::fs;
use std::process;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        generate_error!("Not enough arguments supplied.\nUsage: litc file.lit");
    }

    let mut arg_pos = 1;

    if args[arg_pos] == "-h" {
        println!("Usage: litc [options] files...\n");
        println!("[OPTIONS]:\n");
        let padding = " ".repeat(16);

        println!("-h{}Get help using litc", padding);
        println!("-S{}Check for semantic error", padding);

        process::exit(0);
    }

    let mut is_just_checking = false;
    if args[arg_pos] == "-S" {
        arg_pos += 1;
        is_just_checking = true;
    }

    let input_path = Path::new(&args[arg_pos]);
    if input_path.extension().and_then(|e| e.to_str()) != Some("lit") {
        generate_error!("File does not have '.lit' extension");
    }

    let src = fs::read_to_string(input_path).unwrap_or_else(|e| {
        generate_error!("Cannot read {:?}, cause by:\n{}", input_path, e);
    });

    let now = Instant::now();

    let tokens = Lexer::new(&src).tokenize();
    //println!("\x1B[1;33m[Tokens]\x1B[0m");
    //tokens.iter().for_each(|token| { println!("{}", token) });

    let program = Parser::new(tokens).parse();
    //println!("\x1B[1;33m[AST]\x1B[0m");
    //print_ast(&program);
    
    analyze(&program);

    let ir = codegen::generate(&program);
    //println!("\x1B[1;33m[IR]\x1B[0m:\n{}", ir);

    println!("Took: {:?}", now.elapsed());

    if is_just_checking {
        println!("\x1b[1;32m[Analysis complete]\x1b[0m: No semantic errors found.");
        process::exit(0);
    }

    let ir_path = input_path.with_extension("ll");
    fs::write(&ir_path, &ir).unwrap_or_else(|e| {
        generate_error!("Cannot write {:?}, cause by:\n{}", ir_path, e);
    });

    let exe_path = input_path.with_extension("exe");
    let clang_status = Command::new("clang")
        .args([
            "--target=x86_64-pc-windows-gnu",
            "-Wno-override-module",
            ir_path.to_str().unwrap(),
            "-o",
            exe_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run clang");

    if !clang_status.success() {
        generate_error!("Clang compilation failed");
    }

    let exit_code = Command::new(&exe_path)
        .status()
        .expect(format!("Failed to run: {:?}", exe_path.to_str().unwrap()).as_str())
        .code()
        .unwrap_or(1);


    println!("Process finished with code: {}", exit_code);
}

/*fn print_expr(expr_arena: &ExprArena, expr_id: ExprId, indent: usize) {
    let padding = " ".repeat(indent);

    match expr_arena.get(expr_id) {
        Expr::Unt(u) => println!("{}Unt({}),", padding, u),
        Expr::Int(i) => println!("{}Int({}),", padding, i),
        Expr::Float(f) => println!("{}Float({}),", padding, f),
        Expr::Bool(b) => println!("{}Bool({}),", padding, b),
        Expr::Str(s) => println!("{}Str(\"{}\"),", padding, s),
        Expr::Var(name) => println!("{}${},", padding, name),
        Expr::Binary { left, op, right } => {
            println!("{}Binary {{", padding);
            print_expr(expr_arena, *left, indent + 2);
            println!("{}  {},", padding, op);
            print_expr(expr_arena, *right, indent + 2);
            println!("{}}}", padding);
        }
    }
}

fn print_ast(program: &Program) {
    println!("Program:");
    for func in &program.funcs {
        println!("  FuncDef: {}()", func.name);
        for stmt in &func.body {
            match stmt {
                Stmt::VarDecl(v) => {
                    let ty = format!("{:?}", v._type).to_lowercase();
                    print!("    VarDecl:  {} {} = ", ty, v.name);
                    print_expr(&program.expr_arena, v.value, 0);
                }
                Stmt::Println(arg) => {
                    print!("    Println:  ");
                    print_expr(&program.expr_arena, *arg, 0);
                }
            }
        }
    }
}*/

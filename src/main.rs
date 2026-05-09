mod lexer;
mod ast;
mod parser;
mod codegen;
mod analyzer;
mod utils;

use lexer::Lexer;
use parser::Parser;
use crate::analyzer::analyze;
use ast::*;

use std::env;
use std::fs;
use std::process;
use std::path::Path;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        generate_error!("Not enough arguments supplied.\nUsage: litc file.lit");
    }

    let input_path = Path::new(&args[1]);
    if input_path.extension().and_then(|e| e.to_str()) != Some("lit") {
        generate_error!("Error: file does not have .lit extension");
    }

    let src = fs::read_to_string(input_path).unwrap_or_else(|e| {
        generate_error!("Cannot read {:?}: {}", input_path, e);
    });

    let now = Instant::now();

    let tokens = Lexer::new(&src).tokenize();
    //println!("[Tokens]");
    //tokens.iter().for_each(|token| { println!("{}", token) });

    let program = Parser::new(tokens).parse();
    println!("[AST]");
    print_ast(&program);
    
    analyze(&program);

    //let ir = codegen::generate(&program);
    //println!("[IR]\n{}", ir);

    println!("Took: {:?}", now.elapsed());

    /*let ir_path = input_path.with_extension("ll");
    fs::write(&ir_path, &ir).unwrap_or_else(|e| {
        generate_error!("Cannot write {:?}: {}", ir_path, e);
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

    let run_status = Command::new(&exe_path)
        .status()
        .expect(format!("Failed to run: {:?}", exe_path.to_str().unwrap()).as_str())
        .code()
        .unwrap_or(1);

    println!("Process finished with code: {}", run_status);*/
}

fn print_expr(expr_arena: &ExprArena, expr_id: ExprId, indent: usize) {
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
        println!("  FuncDef: \"{}\"", func.name);
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
}

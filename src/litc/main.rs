mod analyzer;
mod ast;
mod codegen;
mod lexer;
mod parser;
mod utils;

use std::path::PathBuf;
use std::time::Instant;
use std::env;
use std::fs;
use std::process;

use ast::*;
use lexer::Lexer;
use parser::Parser;
use analyzer::analyze;
use parser::desugar;

const VERSION: &str = "v0.1.0";
const HELP_TEXT: &str = "litc - Lit language compiler\n\
\n\
\x1B[1mUsage:\x1B[0m\n  \
  litc [options] <file.lit>\n\
\n\
\x1B[1m[Options]\x1B[0m\n  \
  -h, --help       Show this help message\n  \
  -v, --version    Show version\n  \
  -T               Mark time of compilation\n  \
  -S, --check      Check for semantic errors only (no output)\n  \
  -o <file>        Set output file path (default: <input>.lit)\
";

struct Options {
    input: PathBuf,
    output: PathBuf,
    mark_time: bool,
    check_only: bool,
    print_ast: bool,
    print_tokens: bool,
}

impl Options {
    fn parse(args: &[String]) -> Options {
        if args.is_empty() {
            println!("Guide for litc: ");
            Options::help();
            process::exit(0);
        }

        let mut input: Option<PathBuf> = None;
        let mut output: Option<PathBuf> = None;
        let mut mark_time = false;
        let mut check_only = false;
        let mut print_ast = false;
        let mut print_tokens = false;

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-h" | "--help" => {
                    Options::help();
                    process::exit(0);
                }

                "-v" | "--version" => {
                    Options::version();
                    process::exit(0);
                }

                "-T" => {
                    mark_time = true;
                }

                "-S" | "--check" => {
                    check_only = true;
                }

                "-TOK" => {
                    print_tokens = true;
                }

                "-AST" => {
                    print_ast = true;
                }

                "-o" => {
                    i += 1;
                    if i >= args.len() {
                        generate_error!("Expected output path after `-o`");
                    }

                    output = Some(PathBuf::from(&args[i]));
                }

                arg if arg.starts_with("-") => {
                    generate_error!("Unexpected option: `{}`", arg);
                }

                path => {
                    if input.is_some() {
                        generate_error!("Multiple input files are not supported yet. sorry!");
                    }

                    input = Some(PathBuf::from(path));
                }
            }
            i += 1;
        }

        let input = input.unwrap_or_else(|| {
            generate_error!("Missing input file\nUsage: litc [options] <file.lit>");
        });

        let output = output.unwrap_or_else(|| input.with_extension("ll"));

        Options { input, output, mark_time, check_only, print_ast, print_tokens }
    }

    fn version() {
        println!("litc - {}", VERSION);
    }

    fn help() {
        println!("{}", HELP_TEXT);
    }
}

fn main() {
    let argv: Vec<String> = env::args().skip(1).collect();
    let options = Options::parse(&argv);

    if options.input.extension().and_then(|e| e.to_str()) != Some("lit") {
        generate_error!("Input file must have `.lit` extension");
    }

    let src = fs::read_to_string(&options.input).unwrap_or_else(|e| {
        generate_error!("Cannot read `{}` due to: {}", options.input.display(), e);
    });

    let now = Instant::now();
    let tokens = Lexer::new(&src).tokenize();

    if options.print_tokens {
        tokens.iter().for_each(|t| println!("{}", t));
    }

    let mut program = Parser::new(tokens).parse();

    // TODO: See Desugar through to the end
    desugar(&mut program);

    if options.print_ast {
        print_ast(&program);
    }

    analyze(&program);

    if options.check_only {
        if options.mark_time {
            println!("Took: {:?}", now.elapsed());
        }

        println!("\x1B[1;32m[Analysis complete]\x1B[0m: No Semantic errors found");
        return;
    }

    let ir = codegen::generate(program);

    if options.mark_time {
        println!("Took: {:?}", now.elapsed());
    }

    fs::write(&options.output, &ir).unwrap_or_else(|e| {
        generate_error!("Cannot write into `{}` due to: {}", options.output.display(), e);
    });
}

fn print_expr(expr_arena: &ExprArena, expr_id: ExprId, indent: usize) {
    let padding = " ".repeat(indent);
    let expr_node = expr_arena.get(expr_id);

    use Lit::*;
    match &expr_node.expr {
        Expr::Lit(Unt(u)) => println!("{}Unt({}),", padding, u),
        Expr::Lit(Int(i)) => println!("{}Int({}),", padding, i),
        Expr::Lit(Float(f)) => println!("{}Float({}),", padding, f),
        Expr::Lit(Bool(b)) => println!("{}Bool({}),", padding, b),
        Expr::Lit(Str(s)) => println!("{}Str(\"{}\"),", padding, s),
        Expr::Var(name) => println!("{}${},", padding, name),
        Expr::Binary { op, left, right } => {
            println!("{}Binary {{", padding);
            print_expr(expr_arena, *left, indent + 2);
            println!("{}  {},", padding, op);
            print_expr(expr_arena, *right, indent + 2);
            println!("{}}}", padding);
        }
        Expr::Unary { op, expr } => {
            println!("{}Unary {{", padding);
            println!("{}  {},", padding, op);
            print_expr(expr_arena, *expr, indent + 2);
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
                    print_expr(&program.expr_arena, v.expr_id, 0);
                }
                Stmt::Println(arg) => {
                    print!("    Println:  ");
                    print_expr(&program.expr_arena, *arg, 0);
                }
            }
        }
    }
}
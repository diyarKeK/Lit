mod utils;
mod toml;

use std::env;
use std::fs;
use std::process;
use std::path::Path;
use std::process::Command;

const VERSION: &str = "v0.1.0";
const HELP_TEXT: &str = "lit - Lit package manager\n\
\n\
\x1B[1mUsage:\x1B[0m\n  \
  lit <command> [options]\n\
\n\
\x1B[1mCommands:\x1B[0m\n  \
  new [name]   Create a new project\n  \
  build        Compile project\n  \
  run          Compile and run project\n  \
  check        Only checks project for semantic errors \n\
\n\
\x1B[1m[Options]\x1B[0m\n  \
  -h, --help             Show this help message\n  \
  -v, --version          Show version\n  \
  --litc-args \"args\"   Adds compiler arguments if you build or run\
";

enum Cmd {
    New { name: String },
    Build,
    Run,
    Check,
}

struct Options {
    cmd: Cmd,
    litc_args: Vec<String>,
}

impl Options {
    fn parse(args: &[String]) -> Options {
        if args.is_empty() {
            println!("Guide for lit: ");
            Options::help();
            process::exit(0);
        }

        let mut cmd: Option<Cmd> = None;
        let mut litc_args = Vec::new();

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

                "new" => {
                    i += 1;
                    if i >= args.len() {
                        generate_error!("Expected project name after command `new`\nUsage: lit new <name>");
                    }

                    cmd = Some(Cmd::New { name: args[i].clone() })
                }

                "build" => cmd = Some(Cmd::Build),
                "run" => cmd = Some(Cmd::Run),
                "check" => cmd = Some(Cmd::Check),

                "--litc-args" => {
                    i += 1;
                    if i >= args.len() {
                        generate_error!("Expected compiler arguments after option `--litc-args`\nUsage: lit ... --litc-args \"args\"");
                    }

                    let mut cur = String::new();
                    for c in args[i].chars() {
                        if c == ' ' {
                            litc_args.push(cur);
                            cur = String::new();
                            continue;
                        }
                        cur.push(c);
                    }
                    litc_args.push(cur);
                }

                other => {
                    println!("Unknown option: `{}`. Use: lit -h to get help", other);
                    process::exit(0);
                },
            }

            i += 1;
        }

        Options { cmd: cmd.unwrap(), litc_args }
    }

    fn new(name: &str) {
        if Path::new(name).exists() {
            generate_error!("Directory `{}` already exists!", name);
        }

        fs::create_dir_all(format!("{}/src", name)).unwrap_or_else(|e| {
            generate_error!("Cannot create project `src` directory due to: {}", e);
        });

        fs::write(
            format!("{}/build.toml", name),
            format!("[project]\nname = \"{}\"\n", name),
        ).unwrap();

        fs::create_dir_all(format!("{}/out/bin", name)).unwrap_or_else(|e| {
            generate_error!("Cannot create `out/bin` directory due to: {}", e);
        });

        fs::create_dir_all(format!("{}/out/ir", name)).unwrap_or_else(|e| {
            generate_error!("Cannot create `out/ir` directory due to: {}", e);
        });

        fs::write(
            format!("{}/src/main.lit", name),
            format!("fun main() {{\n    \n}}"),
        ).unwrap();

        let s = format!(
            "\x1B[1;32mComplete\x1B[0m Created project `{name}`:\n\
             {name}/\n  \
               out/bin/\n  \
               out/ir/\n  \
               src/\n    \
                 main.lit\n  \
               build.toml",
            name = name
        );
        println!("{}", s);
    }

    fn build(litc_args: Vec<String>, run_after: bool) {
        let toml = fs::read_to_string("build.toml").unwrap_or_else(|e| {
            generate_error!("Cannot read `build.toml` due to: {}", e);
        });
        let project_name = Options::parse_toml_name(&toml);

        let src_path = "src/main.lit";
        let ll_path = &format!("out/ir/{}.ll", project_name);
        let exe_path = &format!("out/bin/{}.exe", project_name);

        println!("\x1B[1;32mCompiling\x1B[0m `{}`...", src_path);
        let litc = Command::new("litc")
            .args(litc_args)
            .args([src_path, "-o", ll_path])
            .status()
            .unwrap_or_else(|_| generate_error!("Cannot run `litc`. Is it installed?"));

        if !litc.success() { generate_error!(""); }

        println!("\x1B[1;32mLinking\x1B[0m   `{}`...", ll_path);
        let clang = Command::new("clang")
            .args([
                "--target=x86_64-pc-windows-gnu",
                "-Wno-override-module",
                ll_path,
                "-o",
                exe_path,
            ])
            .status()
            .unwrap_or_else(|_| generate_error!("Failed to run `clang`. Is it installed?"));

        if !clang.success() { generate_error!("Clang compilation failed"); }

        println!("\x1B[1;32mDone.\x1B[0m      Built in: `{}`;", exe_path);

        if run_after {
            println!("\x1B[1;32mRunning\x1B[0m   `{}`...\n", exe_path);
            let code = Command::new(format!("./{}", exe_path))
                .status()
                .unwrap()
                .code()
                .unwrap_or(1);

            println!("\nProcess finished with code: {}", code);
        }
    }

    fn parse_toml_name(toml: &str) -> String {
        for line in toml.lines() {
            let line = line.trim();

            if line.starts_with("name") {
                if let Some(val) = line.split('=').nth(1) {
                    return val.trim().trim_matches('"').to_string();
                }
            }
        }

        generate_error!("Cannot find attribute: `name` in build.toml")
    }

    fn check() {
        let src_path = "src/main.lit";

        println!("\x1B[1;32mChecking\x1B[0m `{}`...", src_path);
        Command::new("litc")
            .args(["-S", src_path])
            .status()
            .unwrap_or_else(|_| generate_error!("Cannot run `litc`. Is it installed?"));
    }

    fn version() {
        println!("Lit package manager - {}", VERSION);
    }

    fn help() {
        println!("{}", HELP_TEXT);
    }
}

fn main() {
    let argv: Vec<String> = env::args().skip(1).collect();
    let options = Options::parse(&argv);

    match options.cmd {
        Cmd::New { name } => {
            Options::new(&name);
        }
        Cmd::Build => Options::build(options.litc_args, false),
        Cmd::Run => Options::build(options.litc_args, true),
        Cmd::Check => Options::check(),
    }
}
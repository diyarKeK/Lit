mod utils;

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
  build        Compile the current project\n  \
  run          Compile and run the current project\n  \
  help         Print this help message\n  \
\n\
\x1B[1m[Options]\x1B[0m\n  \
  -h, --help       Show this help message\n  \
  -v, --version    Show version\
";

enum Cmd {
    New { name: String },
    Build,
    Run,
    Version,
    Help,
}

impl Cmd {
    fn parse(args: &[String]) -> Cmd {
        if args.is_empty() {
            return Cmd::Help;
        }

        match args[0].as_str() {
            "new" => {
                if args.len() < 2 {
                    generate_error!("Expected project name\nUsage: lit new <name>");
                }
                Cmd::New { name: args[1].clone() }
            }

            "build" => Cmd::Build,
            "run" => Cmd::Run,

            "-v" | "--version" => Cmd::Version,
            "-h" | "--help" | "help" => Cmd::Help,

            other => {
                println!("Unknown command: `{}`. To know: ", other);
                Cmd::Help
            },
        }
    }

    fn cmd_new(name: &str) {
        if Path::new(name).exists() {
            generate_error!("Directory `{}` already exists!", name);
        }

        fs::create_dir_all(format!("{}/src", name)).unwrap_or_else(|e| {
            generate_error!("Cannot create project `src` directory due to: {}", e);
        });

        fs::write(
            format!("{}/build.toml", name),
            format!("[project]\nname = `{}`\nversion = `0.1.0`", name),
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
            "\x1B[1;32m[Complete]\x1B[0m Created project `{name}`:\n\
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

    fn cmd_build(run_after: bool) {
        let toml = fs::read_to_string("build.toml").unwrap_or_else(|e| {
            generate_error!("Cannot read `build.toml` due to: {}", e);
        });
        let project_name = Cmd::parse_toml_name(&toml);

        let src_path = "src/main.lit";
        let ll_path = format!("out/ir/{}.ll", project_name);
        let exe_path = format!("out/bin/{}.exe", project_name);

        println!("[1/2] Compiling `{}`...", src_path);
        let litc = Command::new("litc")
            .args([src_path, "-o", &ll_path])
            .status()
            .unwrap_or_else(|_| generate_error!("Cannot run `litc`. Is it installed?"));

        if !litc.success() { generate_error!(""); }

        println!("[2/2] Linking `{}`...", ll_path);
        let clang = Command::new("clang")
            .args([
                "--target=x86_64-pc-windows-gnu",
                "-Wno-override-module",
                &ll_path,
                "-o",
                &exe_path,
            ])
            .status()
            .unwrap_or_else(|_| generate_error!("Failed to run `clang`. Is it installed?"));

        if !clang.success() { generate_error!("Clang compilation failed"); }

        println!("\x1B[1;32m[Build success]\x1B[0m Done. Built in: `{}`;", exe_path);

        if run_after {
            println!("[run] Running `{}`...\n", exe_path);
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

    fn cmd_version() {
        println!("Lit package manager - {}", VERSION);
    }

    fn cmd_help() {
        println!("{}", HELP_TEXT);
    }
}

fn main() {
    let argv: Vec<String> = env::args().skip(1).collect();
    let cmd = Cmd::parse(&argv);

    match cmd {
        Cmd::New { name } => Cmd::cmd_new(&name),
        Cmd::Build => Cmd::cmd_build(false),
        Cmd::Run => Cmd::cmd_build(true),
        Cmd::Version => Cmd::cmd_version(),
        Cmd::Help => Cmd::cmd_help(),
    }
}
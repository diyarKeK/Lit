use std::collections::HashSet;
use std::process;

use crate::ast::*;

pub fn analyze(program: &Program) {
    for func in &program.funcs {
        let mut declared: HashSet<String> = HashSet::new();

        for stmt in &func.body {
            match stmt {
                Stmt::VarDecl(v) => {
                    if !declared.insert(v.name.clone()) {
                        eprintln!(
                            "Semantic error in '{}': variable '{}' is already declared",
                            func.name, v.name
                        );
                        process::exit(1);
                    }
                }

                Stmt::Println(PrintlnArg::Var(name)) => {
                    if !declared.contains(name) {
                        eprintln!(
                            "Semantic error in '{}': variable '{}' is not declared",
                            func.name, name
                        );
                        process::exit(1);
                    }
                }

                _ => {}
            }
        }
    }
}
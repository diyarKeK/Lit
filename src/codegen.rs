/*use std::collections::HashMap;
use std::process;

use crate::ast::*;
use crate::generate_error;

pub fn generate(program: &Program) -> String {
    let mut out = String::new();

    out.push_str("; Lit compiler v1 - generated LLVM IR\n\n");
    out.push_str("declare i32 @puts(i8* nocapture)\n");
    out.push_str("declare i32 @printf(i8*, ...)\n\n");

    out.push_str("@bool.true  = private unnamed_addr constant [5 x i8] c\"true\\00\"\n");
    out.push_str("@bool.false = private unnamed_addr constant [6 x i8] c\"false\\00\"\n\n");

    for func in &program.funcs {
        emit_func(&mut out, &func);
    }

    out
}

fn emit_func(out: &mut String, func: &FuncDef) {
    let ctx = FuncCtx::build(func);

    for (i, s) in ctx.str_consts.iter().enumerate() {
        let b = s.len() + 1;
        out.push_str(&format!(
           "@str.{fn_name}.{i} = private unnamed_addr constant [{b} x i8] c\"{esc}\\00\"\n",
            fn_name = func.name, i = i, b = b, esc = escape_llvm(s),
        ));
    }

    for (i, fmt) in ctx.num_fmts.iter().enumerate() {
        let b = fmt.len() + 1;
        out.push_str(&format!(
            "@fmt.{fn_name}.{i} = private unnamed_addr constant [{b} x i8] c\"{esc}\\00\"\n",
            fn_name = func.name, i = i, b = b, esc = escape_llvm(fmt),
        ));
    }
    out.push('\n');

    out.push_str(&format!("define i32 @{}() {{\n", func.name));
    out.push_str("entry:\n");

    let mut state = EmitState::new();

    for stmt in &func.body {
        match stmt {
            Stmt::VarDecl(v) => emit_vardecl(out, v, &func.name, &mut state),
            Stmt::Println(arg) => emit_println(out, arg, &func.name, &ctx, &mut state),
        }
    }

    out.push_str("  ret i32 0\n");
    out.push_str("}\n\n");
}

fn emit_vardecl(out: &mut String, v: &VarDecl, fn_name: &str, state: &mut EmitState) {
    match &v.value {
        Value::Unt(u) => {
            out.push_str(&format!("  %{name} = alloca i64\n", name = v.name));
            out.push_str(&format!(
                "  store i64 {u}, i64* %{name}\n",
                u = *u as i64, name = v.name,
            ));
        }

        Value::Int(i) => {
            out.push_str(&format!("  %{name} = alloca i64\n", name = v.name));
            out.push_str(&format!(
                "  store i64 {i}, i64* %{name}\n",
                i = i, name = v.name,
            ));
        }

        Value::Float(f) => {
            out.push_str(&format!("  %{name} = alloca double\n", name = v.name));
            out.push_str(&format!(
                "  store double {f:.6e}, double* %{name}\n",
                f = f, name = v.name,
            ));
        }

        Value::Bool(b) => {
            let val = if *b { 1 } else { 0 };
            out.push_str(&format!("  %{name} = alloca i1\n", name = v.name));
            out.push_str(&format!(
                "  store i1 {val}, i1* %{name}\n",
                val = val, name = v.name,
            ));
        }

        Value::Str(s) => {
            let b = s.len() + 1;
            let si = state.str_idx;
            state.str_idx += 1;
            let tmp = state.next_reg();

            out.push_str(&format!("  %{name} = alloca i8*\n", name = v.name));
            out.push_str(&format!(
                "  %r{tmp} = getelementptr inbounds [{b} x i8], [{b} x i8]* @str.{fn_name}.{si}, i32 0, i32 0\n",
                tmp = tmp, b = b, fn_name = fn_name, si = si,
            ));
            out.push_str(&format!(
                "  store i8* %r{tmp}, i8** %{name}\n",
                tmp = tmp, name = v.name,
            ));
        }
    }
}

fn emit_println(out: &mut String, arg: &PrintlnArg, fn_name: &str, ctx: &FuncCtx, state: &mut EmitState) {
    match arg {
        PrintlnArg::StringLit(s) => {
            let b = s.len() + 1;
            let si = state.str_idx;
            state.str_idx += 1;
            let ptr = state.next_reg();

            out.push_str(&format!(
                "  %r{ptr} = getelementptr inbounds [{b} x i8], [{b} x i8]* @str.{fn_name}.{si}, i32 0, i32 0\n",
                ptr = ptr, b = b, fn_name = fn_name, si = si,
            ));
            out.push_str(&format!(
                "  call i32 @puts(i8* %r{ptr})\n", ptr = ptr,
            ));
        }

        PrintlnArg::Var(name) => {
            let _type = ctx.var_types.get(name)
                .unwrap_or_else(|| generate_error!("Undefined variable: {}", name));

            match _type {
                Type::Unt | Type::Int | Type::Float => {
                    let fi = state.fmt_idx;
                    state.fmt_idx += 1;
                    let fmt = &ctx.num_fmts[fi];
                    let fb = fmt.len() + 1;
                    let rf = state.next_reg();
                    let rv = state.next_reg();

                    out.push_str(&format!(
                        "  %r{rf} = getelementptr inbounds [{fb} x i8], [{fb} x i8]* @fmt.{fn_name}.{fi}, i32 0, i32 0\n",
                        rf = rf, fb = fb, fn_name = fn_name, fi = fi,
                    ));

                    match _type {
                        Type::Unt | Type::Int => {
                            out.push_str(&format!(
                                "  %r{rv} = load i64, i64* %{name}\n",
                                rv = rv, name = name,
                            ));
                            out.push_str(&format!(
                                "  call i32 (i8*, ...) @printf(i8* %r{rf}, i64 %r{rv})\n",
                                rf = rf, rv = rv,
                            ));
                        }
                        Type::Float => {
                            out.push_str(&format!(
                                "  %r{rv} = load double, double* %{name}\n",
                                rv = rv, name = name,
                            ));
                            out.push_str(&format!(
                                "  call i32 (i8*, ...) @printf(i8* %r{rf}, double %r{rv})\n",
                                rf = rf, rv = rv,
                            ));
                        }
                        _ => unreachable!(),
                    }
                }

                Type::Bool => {
                    let rb = state.next_reg();
                    let rt = state.next_reg();
                    let rf = state.next_reg();
                    let rs = state.next_reg();

                    out.push_str(&format!(
                        "  %r{rb} = load i1, i1* %{name}\n",
                        rb = rb, name = name,
                    ));
                    out.push_str(&format!(
                       "  %r{rt} = getelementptr inbounds [5 x i8], [5 x i8]* @bool.true, i32 0, i32 0\n",
                        rt = rt,
                    ));
                    out.push_str(&format!(
                        "  %r{rf} = getelementptr inbounds [6 x i8], [6 x i8]* @bool.false, i32 0, i32 0\n",
                        rf = rf,
                    ));

                    out.push_str(&format!(
                        "  %r{rs} = select i1 %r{rb}, i8* %r{rt}, i8* %r{rf}\n",
                        rs = rs, rb = rb, rt = rt, rf = rf,
                    ));
                    out.push_str(&format!(
                        "  call i32 @puts(i8* %r{rs})\n",
                        rs = rs,
                    ));
                }

                Type::Str => {
                    let rv = state.next_reg();

                    out.push_str(&format!(
                        "  %r{rv} = load i8*, i8** %{name}\n",
                        rv = rv, name = name,
                    ));
                    out.push_str(&format!(
                        "  call i32 @puts(i8* %r{rv})\n",
                        rv = rv,
                    ));
                }
            }
        }
    }
}

fn escape_llvm(s: &str) -> String {
    s.chars().flat_map(|c| {
        let b = c as u32;
        if b == b'"' as u32 || b == b'\\' as u32 || b < 0x20 || b > 0x7e {
            format!("\\{:02X}", b).chars().collect::<Vec<_>>()
        } else {
            vec![c]
        }
    }).collect()
}

struct FuncCtx {
    str_consts: Vec<String>,
    num_fmts: Vec<String>,
    var_types: HashMap<String, Type>,
}

impl FuncCtx {
    fn build(func: &FuncDef) -> FuncCtx {
        let mut str_consts = Vec::new();
        let mut num_fmts = Vec::new();
        let mut var_types = HashMap::new();

        for stmt in &func.body {
            match stmt {
                Stmt::VarDecl(v) => {
                    var_types.insert(v.name.clone(), v._type.clone());

                    if let Value::Str(s) = &v.value {
                        str_consts.push(s.clone());
                    }
                }
                Stmt::Println(PrintlnArg::StringLit(s)) => {
                    str_consts.push(s.clone());
                }
                Stmt::Println(PrintlnArg::Var(_)) => {}
            }
        }

        for stmt in &func.body {
            if let Stmt::Println(PrintlnArg::Var(name)) = stmt {
                let _type = var_types.get(name)
                    .unwrap_or_else(|| generate_error!("Undefined variable: {}", name));

                let fmt = match _type {
                    Type::Unt => Some("%llu\n"),
                    Type::Int => Some("%lld\n"),
                    Type::Float => Some("%g\n"),
                    Type::Bool => None,
                    Type::Str => None,
                };

                if let Some(f) = fmt {
                    num_fmts.push(f.to_string());
                }
            }
        }

        FuncCtx { str_consts, num_fmts, var_types }
    }
}

struct EmitState {
    reg: usize,
    str_idx: usize,
    fmt_idx: usize,
}

impl EmitState {
    fn new() -> EmitState {
        EmitState { reg: 0, str_idx: 0, fmt_idx: 0 }
    }

    fn next_reg(&mut self) -> usize {
        let r = self.reg;
        self.reg += 1;
        r
    }
}*/


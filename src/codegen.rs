use std::collections::HashMap;

use crate::ast::*;

pub fn generate(program: &Program) -> String {
    let mut out = String::new();

    out.push_str("; Lit compiler v1 - generated LLVM IR\n\n");
    out.push_str("declare i32 @puts(i8* nocapture)\n");
    out.push_str("declare i32 @printf(i8*, ...)\n\n");

    out.push_str("@bool.true  = private unnamed_addr constant [5 x i8] c\"true\\00\"\n");
    out.push_str("@bool.false = private unnamed_addr constant [6 x i8] c\"false\\00\"\n\n");

    for func in &program.funcs {
        emit_func(&mut out, func, &program.expr_arena);
    }

    out
}

fn emit_func(out: &mut String, func: &FuncDef, expr_arena: &ExprArena) {
    let ctx = FuncCtx::build(func, expr_arena);

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
            Stmt::VarDecl(v) => emit_vardecl(out, v, expr_arena, &func.name, &ctx, &mut state),
            Stmt::Println(arg) => emit_println(out, expr_arena, *arg, &func.name, &ctx, &mut state),
        }
    }

    out.push_str("  ret i32 0\n");
    out.push_str("}\n\n");
}

fn emit_vardecl(
    out: &mut String,
    v: &VarDecl,
    arena: &ExprArena,
    fn_name: &str,
    ctx: &FuncCtx,
    state: &mut EmitState,
) {
    let llvm_type = infer_llvm_type(arena, v.value, &ctx.var_types);
    let alloca_type = llvm_type.get_alloca_type();

    out.push_str(&format!("  %{name} = alloca {_type}\n", name = v.name, _type = alloca_type));

    let val = emit_expr(out, arena, v.value, fn_name, ctx, state).0;

    out.push_str(&format!(
        "  store {_type} {val}, {_type}* %{name}\n",
        _type = alloca_type, val = val, name = v.name,
    ));
}

fn emit_println(
    out: &mut String,
    arena: &ExprArena,
    expr_id: ExprId,
    fn_name: &str,
    ctx: &FuncCtx,
    state: &mut EmitState
) {
    let _type = infer_llvm_type(arena, expr_id, &ctx.var_types);
    let val = emit_expr(out, arena, expr_id, fn_name, ctx, state).0;

    match _type {
        LlvmType::I64Unsigned | LlvmType::I64Signed | LlvmType::Double => {
            let fi = state.fmt_idx;
            state.fmt_idx += 1;
            let fmt = &ctx.num_fmts[fi];
            let fb = fmt.len() + 1;
            let rf = state.next_reg();
            let llvm_type = _type.get_alloca_type();

            out.push_str(&format!(
                "  %r{rf} = getelementptr inbounds [{fb} x i8], [{fb} x i8]* @fmt.{fn_name}.{fi}, i32 0, i32 0\n",
                rf = rf, fb = fb, fn_name = fn_name, fi = fi,
            ));
            out.push_str(&format!(
                "  call i32 (i8*, ...) @printf(i8* %r{rf}, {_type} {val})\n",
                rf = rf, _type = llvm_type, val = val
            ));
        }

        LlvmType::I1 => {
            let rt = state.next_reg();
            let rf = state.next_reg();
            let rs = state.next_reg();

            out.push_str(&format!(
                "  %r{rt} = getelementptr inbounds [5 x i8], [5 x i8]* @bool.true, i32 0, i32 0\n",
                rt = rt,
            ));
            out.push_str(&format!(
                "  %r{rf} = getelementptr inbounds [6 x i8], [6 x i8]* @bool.false, i32 0, i32 0\n",
                rf = rf,
            ));
            out.push_str(&format!(
                "  %r{rs} = select i1 {val}, i8* %r{rt}, i8* %r{rf}\n",
                rs = rs, val = val, rt = rt, rf = rf,
            ));
            out.push_str(&format!("  call i32 @puts(i8* %r{rs})\n", rs = rs));
        }

        LlvmType::I8Ptr => {
            out.push_str(&format!("  call i32 @puts(i8* {val})\n", val = val));
        }
    }
}

fn emit_expr(
    out: &mut String,
    arena: &ExprArena,
    id: ExprId,
    fn_name: &str,
    ctx: &FuncCtx,
    state: &mut EmitState
) -> (String, LlvmType) {
    match arena.get(id) {
        Expr::Unt(u) => (format!("{}", *u as i64), LlvmType::I64Unsigned),
        Expr::Int(i) => (format!("{}", i), LlvmType::I64Signed),
        Expr::Float(f) => (format!("{:.6e}", f), LlvmType::Double),
        Expr::Bool(b) => (format!("{}", *b as i32), LlvmType::I1),
        Expr::Str(s) => {
            let b = s.len() + 1;
            let si = state.str_idx;
            state.str_idx += 1;
            let reg = state.next_reg();

            out.push_str(&format!(
                "  %r{reg} = getelementptr inbounds [{b} x i8], [{b} x i8]* @str.{fn_name}.{si}, i32 0, i32 0\n",
                reg = reg, b = b, fn_name = fn_name, si = si,
            ));
            (format!("%r{}", reg), LlvmType::I8Ptr)
        }

        Expr::Var(name) => {
            let _type = infer_llvm_type(arena, id, &ctx.var_types);
            let llvm_type = _type.get_alloca_type();
            let reg = state.next_reg();

            out.push_str(&format!(
                "  %r{reg} = load {_type}, {_type}* %{name}\n",
                reg = reg, _type = llvm_type, name = name,
            ));
            (format!("%r{}", reg), _type)
        }

        Expr::Binary { left, op, right } => {
            let (l_value, l_type) = emit_expr(out, arena, *left, fn_name, ctx, state);
            let (r_value, _) = emit_expr(out, arena, *right, fn_name, ctx, state);

            let instr = llvm_instr_for_operator_by_type(op, &l_type);
            let llvm_type = l_type.get_alloca_type();
            let reg = state.next_reg();

            out.push_str(&format!(
                "  %r{reg} = {instr} {_type} {l_value}, {r_value}\n",
                reg = reg, instr = instr, _type = llvm_type, l_value = l_value, r_value = r_value,
            ));
            (format!("%r{}", reg), l_type)
        }
    }
}

fn llvm_instr_for_operator_by_type(op: &Operand, llvm_type: &LlvmType) -> &'static str {
    match (op, llvm_type) {
        (Operand::Plus, LlvmType::I64Unsigned | LlvmType::I64Signed) => "add",
        (Operand::Minus, LlvmType::I64Unsigned | LlvmType::I64Signed) => "sub",
        (Operand::Mul, LlvmType::I64Unsigned | LlvmType::I64Signed) => "mul",

        (Operand::Div, LlvmType::I64Unsigned) => "udiv",
        (Operand::Div, LlvmType::I64Signed) => "sdiv",
        (Operand::Rem, LlvmType::I64Unsigned) => "urem",
        (Operand::Rem, LlvmType::I64Signed) => "srem",

        (Operand::Plus, LlvmType::Double) => "fadd",
        (Operand::Minus, LlvmType::Double) => "fsub",
        (Operand::Mul, LlvmType::Double) => "fmul",
        (Operand::Div, LlvmType::Double) => "fdiv",
        (Operand::Rem, LlvmType::Double) => "frem",

        _ => unreachable!(),
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

#[derive(Debug, Clone, PartialEq)]
enum LlvmType {
    I64Unsigned,
    I64Signed,
    Double,
    I1,
    I8Ptr
}

impl LlvmType {
    fn get_alloca_type(&self) -> &'static str {
        match self {
            LlvmType::I64Unsigned => "i64",
            LlvmType::I64Signed => "i64",
            LlvmType::Double => "double",
            LlvmType::I1 => "i1",
            LlvmType::I8Ptr => "i8*",
        }
    }
}

fn infer_llvm_type(arena: &ExprArena, id: ExprId, var_types: &HashMap<String, Type>) -> LlvmType {
    match arena.get(id) {
        Expr::Unt(_) => LlvmType::I64Unsigned,
        Expr::Int(_) => LlvmType::I64Signed,
        Expr::Float(_) => LlvmType::Double,
        Expr::Bool(_) => LlvmType::I1,
        Expr::Str(_) => LlvmType::I8Ptr,

        Expr::Var(name) => {
            match var_types.get(name).unwrap() {
                Type::Unt => LlvmType::I64Unsigned,
                Type::Int => LlvmType::I64Signed,
                Type::Float => LlvmType::Double,
                Type::Bool => LlvmType::I1,
                Type::Str => LlvmType::I8Ptr,
            }
        }

        Expr::Binary { left, ..} => infer_llvm_type(arena, *left, var_types),
    }
}

struct FuncCtx {
    str_consts: Vec<String>,
    num_fmts: Vec<String>,
    var_types: HashMap<String, Type>,
}

impl FuncCtx {
    fn build(func: &FuncDef, arena: &ExprArena) -> FuncCtx {
        let mut str_consts: Vec<String> = Vec::new();
        let mut num_fmts = Vec::new();
        let mut var_types = HashMap::new();

        for stmt in &func.body {
            match stmt {
                Stmt::VarDecl(v) => {
                    var_types.insert(v.name.clone(), v._type.clone());

                    if let Expr::Str(s) = arena.get(v.value) {
                         str_consts.push(s.clone());
                    }
                }
                Stmt::Println(id) => {
                    if let Expr::Str(s) = arena.get(*id) {
                        str_consts.push(s.clone());
                    }
                }
            }
        }

        for stmt in &func.body {
            if let Stmt::Println(expr_id) = stmt {
                let _type = infer_llvm_type(arena, *expr_id, &var_types);

                let fmt = match _type {
                    LlvmType::I64Unsigned => Some("%llu\n"),
                    LlvmType::I64Signed => Some("%lld\n"),
                    LlvmType::Double => Some("%g\n"),
                    _ => None,
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
}


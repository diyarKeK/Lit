use std::collections::HashMap;

use super::LlvmType;
use super::FuncCtx;
use super::EmitState;
use crate::ast::*;

pub fn generate(program: Program) -> String {
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

    for (i, s) in ctx.get_string_consts().iter().enumerate() {
        let b = s.len() + 1;
        out.push_str(&format!(
           "@str.{fn_name}.{i} = private unnamed_addr constant [{b} x i8] c\"{esc}\\00\"\n",
            fn_name = func.name, i = i, b = b, esc = escape_llvm(s),
        ));
    }

    for (i, fmt) in ctx.get_num_fmts().iter().enumerate() {
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
    let llvm_type = infer_llvm_type(arena, v.expr_id, ctx.get_var_types());
    let alloca_type = llvm_type.get_alloca_type();

    out.push_str(&format!("  %{name} = alloca {_type}\n", name = v.name, _type = alloca_type));

    let val = emit_expr(out, arena, v.expr_id, fn_name, ctx, state).0;

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
    let _type = infer_llvm_type(arena, expr_id, ctx.get_var_types());
    let val = emit_expr(out, arena, expr_id, fn_name, ctx, state).0;

    match _type {
        LlvmType::I64Unsigned | LlvmType::I64Signed | LlvmType::Double => {
            let fi = state.next_fmt_idx();
            let fmt = &ctx.get_num_fmts()[fi];
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

#[allow(clippy::needless_return)]
fn emit_expr(
    out: &mut String,
    arena: &ExprArena,
    id: ExprId,
    fn_name: &str,
    ctx: &FuncCtx,
    state: &mut EmitState
) -> (String, LlvmType) {
    let expr_node = arena.get(id);
    let expr = &expr_node.expr;

    use Lit::*;
    match expr {
        Expr::Lit(Unt(u)) => (format!("{}", *u as i64), LlvmType::I64Unsigned),
        Expr::Lit(Int(i)) => (format!("{}", i), LlvmType::I64Signed),
        Expr::Lit(Float(f)) => (format!("{:.6e}", f), LlvmType::Double),
        Expr::Lit(Bool(b)) => (format!("{}", *b as i32), LlvmType::I1),
        Expr::Lit(Str(s)) => {
            let b = s.len() + 1;
            let si = state.next_str_idx();
            let reg = state.next_reg();

            out.push_str(&format!(
                "  %r{reg} = getelementptr inbounds [{b} x i8], [{b} x i8]* @str.{fn_name}.{si}, i32 0, i32 0\n",
                reg = reg, b = b, fn_name = fn_name, si = si,
            ));
            (format!("%r{}", reg), LlvmType::I8Ptr)
        }

        Expr::Var(name) => {
            let _type = infer_llvm_type(arena, id, ctx.get_var_types());
            let llvm_type = _type.get_alloca_type();
            let reg = state.next_reg();

            out.push_str(&format!(
                "  %r{reg} = load {_type}, {_type}* %{name}\n",
                reg = reg, _type = llvm_type, name = name,
            ));
            (format!("%r{}", reg), _type)
        }

        Expr::Binary { op, left, right } => {
            let (l_value, l_type) = emit_expr(out, arena, *left, fn_name, ctx, state);
            let (r_value, _) = emit_expr(out, arena, *right, fn_name, ctx, state);

            let instr = llvm_instr_for_operator_by_type(op, &l_type);
            let llvm_type = l_type.get_alloca_type();
            let reg = state.next_reg();

            out.push_str(&format!(
                "  %r{reg} = {instr} {_type} {l_value}, {r_value}\n",
                reg = reg, instr = instr, _type = llvm_type, l_value = l_value, r_value = r_value,
            ));

            return (
                format!("%r{}", reg),
                if op.is_comparison() || op.is_arranging() {
                    LlvmType::I1
                } else {
                    l_type
                }
            );
        }

        Expr::Unary { op, expr } => {
            let (value, _type) = emit_expr(out, arena, *expr, fn_name, ctx, state);

            let (instr, literal) = llvm_instr_and_literal_for_unary_operator_by_type(op, &_type);
            let llvm_type = _type.get_alloca_type();
            let reg = state.next_reg();

            out.push_str(&format!(
                "  %r{reg} = {instr} {_type} {lit}, {value}\n",
                reg = reg, instr = instr, _type = llvm_type, lit = literal, value = value,
            ));

            (format!("%r{}", reg), _type)
        }
    }
}

fn llvm_instr_for_operator_by_type(op: &BinaryOp, llvm_type: &LlvmType) -> &'static str {
    match (op, llvm_type) {
        (BinaryOp::Add, LlvmType::I64Unsigned | LlvmType::I64Signed) => "add",
        (BinaryOp::Sub, LlvmType::I64Unsigned | LlvmType::I64Signed) => "sub",
        (BinaryOp::Mul, LlvmType::I64Unsigned | LlvmType::I64Signed) => "mul",

        (BinaryOp::Div, LlvmType::I64Unsigned) => "udiv",
        (BinaryOp::Div, LlvmType::I64Signed) => "sdiv",
        (BinaryOp::Mod, LlvmType::I64Unsigned) => "urem",
        (BinaryOp::Mod, LlvmType::I64Signed) => "srem",

        (BinaryOp::Add, LlvmType::Double) => "fadd",
        (BinaryOp::Sub, LlvmType::Double) => "fsub",
        (BinaryOp::Mul, LlvmType::Double) => "fmul",
        (BinaryOp::Div, LlvmType::Double) => "fdiv",
        (BinaryOp::Mod, LlvmType::Double) => "frem",

        (BinaryOp::EqEq, LlvmType::I64Unsigned | LlvmType::I64Signed | LlvmType::I1) => "icmp eq",
        (BinaryOp::NotEq, LlvmType::I64Unsigned | LlvmType::I64Signed | LlvmType::I1) => "icmp ne",

        (BinaryOp::Gt, LlvmType::I64Unsigned) => "icmp ugt",
        (BinaryOp::Lt, LlvmType::I64Unsigned) => "icmp ult",
        (BinaryOp::GtEq, LlvmType::I64Unsigned) => "icmp uge",
        (BinaryOp::LtEq, LlvmType::I64Unsigned) => "icmp ule",

        (BinaryOp::Gt, LlvmType::I64Signed) => "icmp sgt",
        (BinaryOp::Lt, LlvmType::I64Signed) => "icmp slt",
        (BinaryOp::GtEq, LlvmType::I64Signed) => "icmp sge",
        (BinaryOp::LtEq, LlvmType::I64Signed) => "icmp sle",

        (BinaryOp::EqEq, LlvmType::Double) => "fcmp oeq",
        (BinaryOp::NotEq, LlvmType::Double) => "fcmp ue",
        (BinaryOp::Gt, LlvmType::Double) => "fcmp ogt",
        (BinaryOp::Lt, LlvmType::Double) => "fcmp olt",
        (BinaryOp::GtEq, LlvmType::Double) => "fcmp oge",
        (BinaryOp::LtEq, LlvmType::Double) => "fcmp ole",

        (BinaryOp::And, LlvmType::I64Unsigned | LlvmType::I64Signed | LlvmType::I1) => "and",
        (BinaryOp::Or, LlvmType::I64Unsigned | LlvmType::I64Signed | LlvmType::I1) => "or",
        (BinaryOp::Xor, LlvmType::I64Unsigned | LlvmType::I64Signed | LlvmType::I1) => "xor",

        (BinaryOp::LShift, LlvmType::I64Unsigned | LlvmType::I64Signed) => "shl",
        (BinaryOp::RShift, LlvmType::I64Unsigned) => "lshr",
        (BinaryOp::RShift, LlvmType::I64Signed) => "ashr",

        _ => unreachable!(),
    }
}

fn llvm_instr_and_literal_for_unary_operator_by_type(op: &UnaryOp, llvm_type: &LlvmType) -> (&'static str, &'static str) {
    match (op, llvm_type) {
        (UnaryOp::Minus, LlvmType::I64Unsigned | LlvmType::I64Signed) => ("sub", "0"),
        (UnaryOp::Minus, LlvmType::Double) => ("fsub", "0.0"),

        (UnaryOp::Not, LlvmType::I64Unsigned | LlvmType::I64Signed) => ("xor", "-1"),
        (UnaryOp::Not, LlvmType::I1) => ("xor", "1"),

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

pub fn infer_llvm_type(arena: &ExprArena, id: ExprId, var_types: &HashMap<String, Type>) -> LlvmType {
    let expr_node = arena.get(id);
    let expr = &expr_node.expr;

    match expr {
        Expr::Lit(Lit::Unt(_)) => LlvmType::I64Unsigned,
        Expr::Lit(Lit::Int(_)) => LlvmType::I64Signed,
        Expr::Lit(Lit::Float(_)) => LlvmType::Double,
        Expr::Lit(Lit::Bool(_)) => LlvmType::I1,
        Expr::Lit(Lit::Str(_)) => LlvmType::I8Ptr,

        Expr::Var(name) => {
            match var_types.get(name).unwrap() {
                Type::Unt => LlvmType::I64Unsigned,
                Type::Int => LlvmType::I64Signed,
                Type::Float => LlvmType::Double,
                Type::Bool => LlvmType::I1,
                Type::Str => LlvmType::I8Ptr,
            }
        }

        Expr::Binary { op, left, .. } => {
            if op.is_comparison() || op.is_arranging() {
                LlvmType::I1
            } else {
                infer_llvm_type(arena, *left, var_types)
            }
        },

        Expr::Unary { expr, .. } => infer_llvm_type(arena, *expr, var_types),
    }
}
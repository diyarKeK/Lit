use std::collections::HashMap;

use super::utils;
use super::LlvmType;
use super::FuncCtx;
use super::EmitState;
use crate::ast::*;

pub fn generate(program: Program) -> String {
    let mut out = String::new();

    out.push_str("\
        ; Lit compiler v1 - generated LLVM IR\n\n\
        declare i32 @putchar(i32)\n\
        declare i32 @puts(i8* nocapture)\n\
        declare i32 @printf(i8*, ...)\n\
        declare void @exit(i32)\n\n\
        @unreachable_msg = private unnamed_addr constant [29 x i8] c\"Entered to unreachable code\\0A\\00\"\n\
        @fmt.u64 = private unnamed_addr constant [6 x i8] c\"%llu\\0A\\00\"\n\
        @fmt.i64 = private unnamed_addr constant [6 x i8] c\"%lld\\0A\\00\"\n\
        @fmt.f64 = private unnamed_addr constant [4 x i8] c\"%g\\0A\\00\"\n\
        @bool.true = private unnamed_addr constant [5 x i8] c\"true\\00\"\n\
        @bool.false = private unnamed_addr constant [6 x i8] c\"false\\00\"\n\n\
    ");

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
            fn_name = func.name, i = i, b = b, esc = utils::escape_llvm(s),
        ));
    }
    out.push('\n');

    out.push_str(&format!("define i32 @{}() {{\n", func.name));

    let mut state = EmitState::new();

    for stmt in func.body.stmts() {
        match stmt {
            Stmt::VarDecl(v) => emit_vardecl(out, v, expr_arena, &func.name, &ctx, &mut state),
            Stmt::Println(arg) => emit_println(out, expr_arena, *arg, &func.name, &ctx, &mut state),
            Stmt::Unreachable => emit_unreachable(out, &mut state),
        }
    }

    out.push_str("  \
            ret i32 0\n\
        }\n\
    ");
}

fn emit_unreachable(out: &mut String, state: &mut EmitState) {
    let reg = state.next_reg();
    out.push_str(&format!("  \
            %r{reg} = getelementptr inbounds [29 x i8], [29 x i8]* @unreachable_msg, i32 0, i32 0\n  \
            call i32 @puts(i8* %r{reg})\n  \
            call void @exit(i32 1)\n  \
            unreachable\n\
        ",
        reg = reg,
    ));
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

    let val = emit_expr(out, arena, v.expr_id, fn_name, ctx, state).0;

    out.push_str(&format!("  \
           %{name} = alloca {_type}\n  \
           store {_type} {val}, {_type}* %{name}\n\
        ",
        name = v.name, _type = alloca_type, val = val,
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
        LlvmType::I64Unsigned => {
            let reg = state.next_reg();

            out.push_str(&format!("  \
                   %r{reg} = getelementptr inbounds [6 x i8], [6 x i8]* @fmt.u64, i32 0, i32 0\n  \
                   call i32 (i8*, ...) @printf(i8* %r{reg}, i64 {val})\n\
                ",
                reg = reg, val = val
            ));
        }

        LlvmType::I64Signed => {
            let reg = state.next_reg();

            out.push_str(&format!("  \
                   %r{reg} = getelementptr inbounds [6 x i8], [6 x i8]* @fmt.i64, i32 0, i32 0\n  \
                   call i32 (i8*, ...) @printf(i8* %r{reg}, i64 {val})\n\
                ",
                reg = reg, val = val
            ));
        }

        LlvmType::Double => {
            let reg = state.next_reg();

            out.push_str(&format!("  \
                   %r{reg} = getelementptr inbounds [4 x i8], [4 x i8]* @fmt.f64, i32 0, i32 0\n  \
                   call i32 (i8*, ...) @printf(i8* %r{reg}, double {val})\n\
                ",
                reg = reg, val = val
            ));
        }

        LlvmType::I1 => {
            let rt = state.next_reg();
            let rf = state.next_reg();
            let rs = state.next_reg();

            out.push_str(&format!("  \
                   %r{rt} = getelementptr inbounds [5 x i8], [5 x i8]* @bool.true, i32 0, i32 0\n  \
                   %r{rf} = getelementptr inbounds [6 x i8], [6 x i8]* @bool.false, i32 0, i32 0\n  \
                   %r{rs} = select i1 {val}, i8* %r{rt}, i8* %r{rf}\n  \
                   call i32 @puts(i8* %r{rs})\n\
                ",
                                  rt = rt, rf = rf, rs = rs, val = val
            ));
        }

        LlvmType::Char => {
            let rs = state.next_reg();

            out.push_str(&format!("  \
                   %r{rs} = zext i8 {val} to i32\n  \
                   call i32 @putchar(i32 %r{rs})\n  \
                   call i32 @putchar(i32 10)\n\
                ",
                rs = rs, val = val,
            ));
        }

        LlvmType::I8Ptr => {
            out.push_str(&format!(
                "  call i32 @puts(i8* {val})\n",
                val = val
            ));
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
    let expr_node = arena.get(id);
    let expr = &expr_node.expr;

    use Lit::*;
    match expr {
        Expr::Lit(Unt(u)) =>  ((*u as i64).to_string(), LlvmType::I64Unsigned),
        Expr::Lit(Int(i)) => (i.to_string(), LlvmType::I64Signed),
        Expr::Lit(Float(f)) => (format!("{:.6e}", f), LlvmType::Double),
        Expr::Lit(Bool(b)) => ((*b as i32).to_string(), LlvmType::I1),
        Expr::Lit(Char(c)) => ((*c as i32).to_string(), LlvmType::Char),
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

        Expr::Binary (op, left, right) => {
            let (l_value, l_type) = emit_expr(out, arena, *left, fn_name, ctx, state);
            let (r_value, _) = emit_expr(out, arena, *right, fn_name, ctx, state);

            let instr = llvm_instr_for_operator_by_type(op, &l_type);
            let llvm_type = l_type.get_alloca_type();
            let reg = state.next_reg();

            out.push_str(&format!(
                "  %r{reg} = {instr} {_type} {l_value}, {r_value}\n",
                reg = reg, instr = instr, _type = llvm_type, l_value = l_value, r_value = r_value,
            ));

            let final_type = if op.is_comparison() || op.is_arranging() {
                LlvmType::I1
            } else {
                l_type
            };

            (format!("%r{}", reg), final_type)
        }

        Expr::Unary (op, expr) => {
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

        Expr::Cast (to, expr) => {
            let (value, from_type) = emit_expr(out, arena, *expr, fn_name, ctx, state);

            let to_type = LlvmType::from(to);

            let instr = llvm_instr_for_cast(&from_type, &to_type);

            if instr.eq("") {
                return (value, to_type)
            }

            let from_llvm_type = from_type.get_alloca_type();
            let to_llvm_type = to_type.get_alloca_type();
            let reg = state.next_reg();

            out.push_str(&format!(
                "  %r{reg} = {instr} {from} {value} to {to}\n",
                reg = reg, instr = instr, from = from_llvm_type, value = value, to = to_llvm_type,
            ));

            (format!("%r{}", reg), to_type)
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

        (BinaryOp::EqEq, LlvmType::I64Unsigned | LlvmType::I64Signed | LlvmType::I1 | LlvmType::Char) => "icmp eq",
        (BinaryOp::NotEq, LlvmType::I64Unsigned | LlvmType::I64Signed | LlvmType::I1 | LlvmType::Char) => "icmp ne",

        (BinaryOp::Gt, LlvmType::I64Unsigned) => "icmp ugt",
        (BinaryOp::Lt, LlvmType::I64Unsigned) => "icmp ult",
        (BinaryOp::GtEq, LlvmType::I64Unsigned) => "icmp uge",
        (BinaryOp::LtEq, LlvmType::I64Unsigned) => "icmp ule",

        (BinaryOp::Gt, LlvmType::I64Signed) => "icmp sgt",
        (BinaryOp::Lt, LlvmType::I64Signed) => "icmp slt",
        (BinaryOp::GtEq, LlvmType::I64Signed) => "icmp sge",
        (BinaryOp::LtEq, LlvmType::I64Signed) => "icmp sle",

        (BinaryOp::EqEq, LlvmType::Double) => "fcmp oeq",
        (BinaryOp::NotEq, LlvmType::Double) => "fcmp une",
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
        (UnaryOp::Minus, LlvmType::I64Signed) => ("sub", "0"),
        (UnaryOp::Minus, LlvmType::Double) => ("fsub", "0.0"),

        (UnaryOp::Not, LlvmType::I64Unsigned | LlvmType::I64Signed) => ("xor", "-1"),
        (UnaryOp::Not, LlvmType::I1) => ("xor", "1"),

        _ => unreachable!(),
    }
}

fn llvm_instr_for_cast(from: &LlvmType, to: &LlvmType) -> &'static str {
    match (from, to) {
        (a, b) if *a == *b => "",

        (LlvmType::I64Unsigned, LlvmType::I64Signed) |
        (LlvmType::I64Signed, LlvmType::I64Unsigned)
        => "",

        (LlvmType::I64Unsigned, LlvmType::Double) => "uitofp",
        (LlvmType::I64Signed, LlvmType::Double) => "sitofp",

        (LlvmType::Double, LlvmType::I64Unsigned) => "fptoui",
        (LlvmType::Double, LlvmType::I64Signed) => "fptosi",

        (LlvmType::I64Unsigned | LlvmType::I64Signed, LlvmType::Char) => "trunc",
        (LlvmType::Char, LlvmType::I64Signed) => "sext",
        (LlvmType::Char, LlvmType::I64Unsigned) => "zext",

        _ => unreachable!(),
    }
}

pub fn infer_llvm_type(arena: &ExprArena, id: ExprId, var_types: &HashMap<String, Type>) -> LlvmType {
    let expr_node = arena.get(id);
    let expr = &expr_node.expr;

    match expr {
        Expr::Lit(Lit::Unt(_)) => LlvmType::I64Unsigned,
        Expr::Lit(Lit::Int(_)) => LlvmType::I64Signed,
        Expr::Lit(Lit::Float(_)) => LlvmType::Double,
        Expr::Lit(Lit::Bool(_)) => LlvmType::I1,
        Expr::Lit(Lit::Char(_)) => LlvmType::Char,
        Expr::Lit(Lit::Str(_)) => LlvmType::I8Ptr,

        Expr::Var(name) => {
            LlvmType::from(var_types.get(name).unwrap())
        }

        Expr::Binary (op, left, _) => {
            if op.is_comparison() || op.is_arranging() {
                LlvmType::I1
            } else {
                infer_llvm_type(arena, *left, var_types)
            }
        },

        Expr::Unary (_, expr) => infer_llvm_type(arena, *expr, var_types),

        Expr::Cast (to, _) => LlvmType::from(to),
    }
}
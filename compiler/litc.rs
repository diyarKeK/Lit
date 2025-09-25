use std::collections::HashMap;
use std::env;
use std::fs;
use std::fmt;
use std::error;

#[derive(Debug)]
struct Error {
    msg: String,
}

impl Error {
    fn new(msg: impl Into<String>) -> Self {
        Self { msg: msg.into() }
    }
}

impl error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Fun,

    Unt,
    Int,
    Float,
    Bool,
    Str,

    Ident(String),

    Assign,
    Number(i64),

    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Semi,

    LiteralStr(String),

    Print,
}

fn tokenize(src: &str) -> Result<Vec<Token>, Error> {
    let mut tokens = Vec::new();
    let mut it = src.chars().peekable();

    while let Some(&ch) = it.peek() {

        match ch {
            ch if ch.is_whitespace() => { it.next(); },
            '(' => { tokens.push(Token::LParen); it.next(); },
            ')' => { tokens.push(Token::RParen); it.next(); },
            '{' => { tokens.push(Token::LBrace); it.next(); },
            '}' => { tokens.push(Token::RBrace); it.next(); },
            ',' => { tokens.push(Token::Comma); it.next(); },
            ':' => { tokens.push(Token::Colon); it.next(); },
            ';' => { tokens.push(Token::Semi); it.next(); },
            '=' => { tokens.push(Token::Assign); it.next(); },
            '"' => {
                it.next();
                let mut s = String::new();

                while let Some(&c) = it.peek() {
                    if c == '"' {
                        it.next();
                        break;
                    }
                    s.push(c);
                    it.next();
                }

                tokens.push(Token::LiteralStr(s));
            },

            c if c.is_ascii_digit() => {
                let mut n = String::new();

                while let Some(&d) = it.peek() {
                    if d.is_ascii_digit() {
                        n.push(d);
                        it.next();
                    } else {
                        break;
                    }
                }

                let v: i64 = n.parse().map_err(|_| Error::new("Not a number"))?;
                tokens.push(Token::Number(v));
            },

            c if c.is_ascii_alphabetic() || c == '_' => {
                let mut ident = String::new();

                while let Some(&d) = it.peek() {
                    if d.is_ascii_alphanumeric() || d == '_' {
                        ident.push(d);
                        it.next();
                    } else {
                        break;
                    }
                }

                match ident.as_str() {
                    "fun" => tokens.push(Token::Fun),
                    "unt" => tokens.push(Token::Unt),
                    "int" => tokens.push(Token::Int),
                    "float" => tokens.push(Token::Float),
                    "bool" => tokens.push(Token::Bool),
                    "str" => tokens.push(Token::Str),
                    "print" => tokens.push(Token::Print),

                    _ => tokens.push(Token::Ident(ident)),
                }
            }

            _ => return Err(Error::new(format!("Unexpected character '{}'", ch))),
        }
    }

    Ok(tokens)
}

#[derive(Debug, Clone)]
enum Expr {
    Number(i64),
    StrLiteral(String),
    Var(String),
}

#[derive(Debug, Clone)]
enum Stmt {
    VarDecl { name: String, dtype: String, val: Expr },
    Print(Expr),
}

#[derive(Debug, Clone)]
struct Function {
    name: String,
    body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
struct Program {
    pub functions: Vec<Function>,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn eat(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned();
        self.pos += 1;

        token
    }

    fn parse_program(&mut self) -> Result<Program, Error> {

        let mut functions = Vec::new();

        while self.peek().is_some() {
            functions.push(self.parse_function()?);
        }

        Ok(Program { functions })
    }

    fn parse_function(&mut self) -> Result<Function, Error> {
        match self.eat() {
            Some(Token::Fun) => {},
            _ => return Err(Error::new("Expected function!")),
        }

        let name = match self.eat() {
            Some(Token::Ident(name)) => name,
            _ => return Err(Error::new("Expected function name after 'fun'!")),
        };

        match self.eat() { Some(Token::LParen) => {}, _ => return Err(Error::new("Expected '(' after function name!")) }
        match self.eat() { Some(Token::RParen) => {}, _ => return Err(Error::new("Expected ')' after function name!")) }
        match self.eat() { Some(Token::LBrace) => {}, _ => return Err(Error::new("Expected '{' after function name!")) }

        let mut body = Vec::new();

        while let Some(token) = self.peek() {
            match token {
                Token::RBrace => {
                    self.eat();
                    break;
                },
                Token::Unt | Token::Int | Token::Float | Token::Bool | Token::Str => {
                    body.push(self.parse_var_declaration()?);
                },

                Token::Print => {
                    body.push(self.parse_print()?);
                },

                _ => return Err(Error::new(format!("Unexpected token in function: {}!", name))),
            }
        }

        Ok(Function { name, body })
    }

    fn parse_var_declaration(&mut self) -> Result<Stmt, Error> {
        let dtype = match self.eat() {
            Some(Token::Unt) => "unt".to_string(),
            Some(Token::Int) => "int".to_string(),
            Some(Token::Float) => "float".to_string(),
            Some(Token::Bool) => "bool".to_string(),
            Some(Token::Str) => "str".to_string(),

            _ => return Err(Error::new("Unknown variable type!")),
        };

        let name = match self.eat() {
            Some(Token::Ident(name)) => name,
            _ => return Err(Error::new("Expected variable name!")),
        };

        match self.eat() {
            Some(Token::Assign) => {},
            _ => return Err(Error::new("Expected '=' after variable name!")),
        }

        let val = match self.eat() {
            Some(Token::Number(n)) => Expr::Number(n),
            Some(Token::LiteralStr(s)) => Expr::StrLiteral(s),
            Some(Token::Ident(i)) => Expr::Var(i),

            _ => return Err(Error::new("Expected variable value!")),
        };



        Ok(Stmt::VarDecl { dtype, name, val })
    }

    fn parse_print(&mut self) -> Result<Stmt, Error> {
        self.eat();

        match self.eat() {
            Some(Token::LParen) => {},
            _ => return Err(Error::new("Expected '(' after 'print' function!")),
        }

        let expr = match self.eat() {
            Some(Token::LiteralStr(s)) => Expr::StrLiteral(s),
            Some(Token::Ident(i)) => Expr::Var(i),
            _ => return Err(Error::new("Expected string for print function!")),
        };

        match self.eat() {
            Some(Token::RParen) => {},
            _ => return Err(Error::new("Expected ')' after 'print' function!")),
        }

        Ok(Stmt::Print(expr))
    }
}

fn check_program(program: &Program) -> Result<(), Error> {

    let mut found_main_func = false;

    for f in &program.functions {

        if f.name == "main" {
            found_main_func = true;
        }
        check_function(f)?;
    }

    if !found_main_func {
        return Err(Error::new("No main function found!"));
    }

    Ok(())
}

fn check_function(f: &Function) -> Result<(), Error> {

    let _ = f.name == "main";

    Ok(())
}

fn generate(program: &Program) -> Result<String, Error> {
    let mut out = String::new();

    for f in &program.functions {
        out.push_str(&format!("label {}:\n", f.name));
        let mut symbol_idx: HashMap<String, String> = HashMap::new();

        for stmt in &f.body {
            match stmt {

                Stmt::VarDecl { name, dtype, val } => {
                    match val {
                        Expr::Number(n) => {
                            out.push_str(&format!("    push_const int {}\n", n));
                        }
                        Expr::StrLiteral(s) => {
                            let esc = s.replace('"', "\\\"");
                            out.push_str(&format!("    push_const str \"{}\"\n", esc));
                        }
                        Expr::Var(i) => {
                            out.push_str(&format!("    load_var {}\n", i));
                        }
                    }

                    out.push_str(&format!("    store_var {}\n", name));
                    symbol_idx.insert(name.clone(), dtype.clone());
                }

                Stmt::Print(expr) => {
                    match expr {
                        Expr::Number(n) => {
                            out.push_str(&format!("    push_const int {}\n", n));
                            out.push_str("    print int\n");
                        }
                        Expr::StrLiteral(s) => {
                            let esc = s.replace('"', "\\\"");
                            out.push_str(&format!("    push_const str \"{}\"\n", esc));
                            out.push_str("    print str\n");
                        }
                        Expr::Var(i) => {
                            out.push_str(&format!("    load_var {}\n", i));

                            if let Some(dtype) = symbol_idx.get(i) {
                                match dtype.as_str() {
                                    "unt" => out.push_str("    print unt\n"),
                                    "int" => out.push_str("    print int\n"),
                                    "float" => out.push_str("    print float\n"),
                                    "str" => out.push_str("    print ref\n"),

                                    _ => return Err(Error::new(format!("Unknown variable type: {}", dtype))),
                                }
                            }
                        }
                    }
                }
            }
        }

        if f.name == "main" {
            out.push_str("    halt 0\n");
        } else {
            out.push_str("    ret\n");
        }
    }

    Ok(out)
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(Error::new("Usage: litc <file.lit>"));
    }

    let path = &args[1];

    if !path.ends_with(".lit") {
        return Err(Error::new("Not .lit file provided!"));
    }

    let src = fs::read_to_string(path).unwrap();
    let tokens = tokenize(&src)?;
    tokens.iter().for_each(|token| {
        println!("{:?}", token);
    });
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;

    check_program(&program)?;

    let bytecode = generate(&program);

    let path = path.replace(".lit", ".lbc");

    fs::write(&path, bytecode?)
        .expect("Unable to write bytecode file");

    println!("Completed successfully in {}!", &path);

    Ok(())
}
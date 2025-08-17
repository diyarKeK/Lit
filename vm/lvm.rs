use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, Write};

#[derive(Debug, Clone)]
enum Value {
    Int(i32),
    Float(f32),
    Bool(bool),
    Str(String),
}

struct LVM {
    code: Vec<String>,
    frame_stack: Vec<HashMap<String, Value>>,
    ip: usize,
    labels: HashMap<String, usize>,
    path: String,
    stack: Vec<Value>,
}

impl LVM {
    fn new(code: Vec<String>, path: String) -> Self {
        Self {
            code,
            frame_stack: vec![HashMap::new()],
            ip: 0,
            labels: HashMap::new(),
            path,
            stack: Vec::new(),
        }
    }

    fn parse_line(line: &str) -> Vec<String> {
        line.split_whitespace().map(
            |str| str.to_string()
        ).collect()
    }

    fn collect_labels(&mut self) {
        for (idx, line) in self.code.iter().enumerate() {
            let parts = Self::parse_line(line);

            if parts.is_empty() {
                continue
            };

            let op = parts[0].to_lowercase();

            if op.eq_ignore_ascii_case("label") && parts.len() > 1 {
                self.labels.insert(parts[1].clone(), idx);
            }
        }
    }

    fn run(&mut self) {
        self.collect_labels();

        if let Some(start) = self.labels.get("main") {
            self.ip = start + 1;
        } else {
            panic!("No 'main' label found!");
        }

        while self.ip < self.code.len() {
            self.execute();
        }
    }

    fn execute(&mut self) {
        let raw_line = self.code[self.ip].trim().to_string();
        self.ip += 1;

        if raw_line.is_empty() || raw_line.starts_with(";") || raw_line.starts_with('#') {
            return;
        }

        let parts = Self::parse_line(&raw_line);
        let op = parts[0].to_lowercase();

        match op.as_str() {
            "push_const" => {
                if parts.len() < 3 {
                    panic!("At {}:{}:\n    {}\nUse:\n    push_const <type> <value>", self.path, self.ip, raw_line);
                }

                let dtype = parts[1].to_lowercase();

                let raw_val = &parts[2];
                let value = match dtype.as_str() {
                    "int" => Value::Int(raw_val.parse().unwrap()),
                    "float" => Value::Float(raw_val.parse().unwrap()),
                    "bool" => Value::Bool(raw_val == "true"),
                    "str" => Value::Str(raw_val.trim_matches('"').to_string()),
                    _ => panic!("Unknown type: {}, at {}:{}:\n    {}", dtype, self.path, self.ip, raw_line)
                };

                self.stack.push(value);
            }

            "store_var" => {
                if parts.len() < 2 {
                    panic!("At {}:{}:\n    {}\nUse:\n    store_var <name>", self.path, self.ip, raw_line)
                }
                if self.stack.is_empty() {
                    panic!("No elements in stack, at {}:{}:\n    {}", self.path, self.ip, raw_line);
                }

                let name = parts[1].clone();
                let val = self.stack.pop().unwrap();
                self.frame_stack.last_mut().unwrap().insert(name, val);
            }

            "load_var" => {
                if parts.len() < 2 {
                    panic!("At {}:{}:\n    {}\nUse:\n    load_var <name>", self.path, self.ip, raw_line);
                }

                let name = &parts[1];

                if !self.frame_stack.last().unwrap().contains_key(name) {
                    panic!("Undefined variable {}, at {}:{}:\n    {}", name, self.path, self.ip, raw_line);
                }

                let val = self.frame_stack.last().unwrap().get(name).unwrap();

                self.stack.push(val.clone());
            }

            "print" => {
                if let Some(val) = self.stack.pop() {
                    match val {
                        Value::Int(i) => println!("{}", i),
                        Value::Float(f) => println!("{}", f),
                        Value::Bool(b) => println!("{}", b),
                        Value::Str(s) => println!("{}", s),
                    }
                } else {
                    println!("Undefined value");
                }
            }

            "input" => {
                if parts.len() < 2 {
                    panic!("At {}:{}:\n    {}\nUse:\n    input <type> (prompt)", self.path, self.ip, raw_line);
                }

                let dtype = parts[1].to_lowercase();
                let prompt = if parts.len() > 2 {
                    parts[2..].join(" ")
                } else {
                    "".to_string()
                };

                print!("{}", prompt);
                io::stdout().flush().unwrap();

                let mut buf = String::new();
                io::stdin().read_line(&mut buf).unwrap();

                let buf = buf.trim();

                let val = match dtype.as_str() {
                    "int" => Value::Int(buf.parse().unwrap()),
                    "float" => Value::Float(buf.parse().unwrap()),
                    "bool" => Value::Bool(buf == "true"),
                    _ => Value::Str(buf.to_string()),
                };

                self.stack.push(val);
            }

            "label" => {}

            "halt" => {
                let code = if parts.len() > 1 {
                    parts[1].parse::<i32>().unwrap_or(0)
                } else {
                    0
                };

                println!("Exiting...");
                std::process::exit(code);
            }

            _ => {
                panic!("Unknown instruction: {}", raw_line);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: lvm <path/to/.lbc>");
        return;
    }

    let path = &args[1];

    let code = fs::read_to_string(path).expect("Failed to read file");

    let lines: Vec<String> = code.lines().map(|line| line.to_string()).collect();

    let mut lvm = LVM::new(lines, path.clone());
    lvm.run();
}
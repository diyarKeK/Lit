use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::process;
use std::thread;
use std::time::{Duration, Instant};

struct Instruction {
    op: u32,
    args: Vec<String>,
    line_idx: usize,
    raw: String,
}

#[derive(Debug, Clone)]
enum HeapValue {
    Str(String),
}

impl fmt::Display for HeapValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HeapValue::Str(s) => write!(f, "{}", s),
        }
    }
}

struct LVM {
    call_stack: Vec<usize>,
    frame_stack: Vec<HashMap<String, u64>>,
    heap: HashMap<u64, HeapValue>,
    instructions: Vec<Instruction>,
    ip: usize,
    labels: HashMap<String, usize>,
    next_heap_id: u64,
    path: String,
    stack: Vec<u64>,
}

impl LVM {
    fn new(path: String) -> Self {
        LVM {
            call_stack: Vec::new(),
            frame_stack: vec![HashMap::new()],
            heap: HashMap::new(),
            instructions: Vec::new(),
            ip: 0,
            labels: HashMap::new(),
            next_heap_id: 1,
            path,
            stack: Vec::new(),
        }
    }

    fn opcode_hash(s: &str) -> u32 {
        let mut h: u32 = 0x811c9dc5;
        for b in s.as_bytes() {
            h ^= *b as u32;
            h = h.wrapping_mul(0x01000193);
        }
        h
    }

    fn current_mut_frame(&mut self) -> &mut HashMap<String, u64> {
        self.frame_stack.last_mut().unwrap()
    }

    fn alloc_heap(&mut self, val: HeapValue) -> u64 {
        let id = self.next_heap_id;
        self.next_heap_id += 1;
        self.heap.insert(id, val);
        id
    }

    fn push_u64(&mut self, val: u64) {
        self.stack.push(val);
    }

    fn push_i64(&mut self, val: i64) {
        self.stack.push(val as u64);
    }

    fn push_f64(&mut self, val: f64) {
        let bits = val.to_bits();
        self.push_u64(bits);
    }

    fn push_ref(&mut self, val: u64) {
        self.stack.push(val);
    }

    fn pop_slot(&mut self) -> u64 {
        match self.stack.pop() {
            Some(v) => v,
            None => panic!("No elements in stack!\nAt {}:{}", self.path, self.ip)
        }
    }

    fn parse_and_load(&mut self, source: &str) -> Result<(), String> {
        let mut instructions: Vec<Instruction> = vec![];

        for (i, raw_line) in source.lines().enumerate() {
            let mut line = raw_line.trim();

            if line.is_empty() {
                continue;
            }

            if let Some(idx) = line.find('#') {
                line = &line[..idx].trim();
            }

            if line.is_empty() {
                continue;
            }

            let mut parts: Vec<String> = vec![];
            let mut cur = String::new();
            let mut in_quote = false;
            let mut chars = line.chars().peekable();

            while let Some(c) = chars.next() {
                if c == '"' {
                    in_quote = !in_quote;
                    cur.push(c);
                    while in_quote {
                        if let Some(nch) = chars.next() {
                            cur.push(nch);
                            if nch == '"' {
                                in_quote = false; break;
                            }
                        } else {
                            panic!("Unterminated quote at {}:{}:\n    {}", self.path, i, raw_line)
                        }
                    }
                } else if c.is_whitespace() && !in_quote {
                    if !cur.is_empty() {
                        parts.push(cur.clone());
                        cur.clear();
                    }
                } else {
                    cur.push(c);
                }
            }

            if !cur.is_empty() {
                parts.push(cur);
            }
            if parts.is_empty() {
                continue;
            }

            let opname = parts[0].to_lowercase().clone();
            let args = if parts.len() > 1 {
                parts[1..].to_vec()
            } else {
                vec![]
            };

            let instr = Instruction {
                op: LVM::opcode_hash(&opname),
                args,
                raw: raw_line.to_string(),
                line_idx: i,
            };
            instructions.push(instr);
        }

        self.collect_labels(&instructions);

        self.instructions = instructions;

        Ok(())
    }

    fn collect_labels(&mut self, instructions: &Vec<Instruction>) {
        for (idx, instr) in instructions.iter().enumerate() {
/* LABEL */ if instr.op == 4137097213 {
                if instr.args.len() != 1 {
                    panic!("At {}:{}:\n    {}\nlabel expects 1 argument;\nUsage: label <name>", self.path, idx, instr.raw)
                }
                let name = instr.args[0].clone();
                if self.labels.contains_key(&name) {
                    panic!("label \"{}\" already defined, at {}:{}:\n    {}", name, self.path, idx, instr.raw)
                }
                self.labels.insert(name, idx);
            }
        }
    }

    fn run(&mut self, now: Instant) {
        self.ip = self.labels.get("main")
            .unwrap_or_else(|| panic!("No main label found in {}", self.path))
            .clone() + 1;

        while self.ip < self.instructions.len() {
            self.execute(now);
        }
    }

    fn execute(&mut self, now: Instant) {
        let instr = &self.instructions[self.ip];
        let args = instr.args.clone();
        let line_idx = instr.line_idx.clone();
        let raw = instr.raw.clone();

        self.ip += 1;

        match instr.op {
/* push_const */3362566317 => {
                if args.len() != 2 {
                    panic!("At {}:{}:\n    {}\npush_const expects 2 arguments\nUsage: push_const <type> <val>", self.path, line_idx, raw);
                }

                let dtype = args[0].as_str();
                let val = args[1..].join(" ");

                match dtype {
                    "unt" => {
                        let val: u64 = val.parse::<u64>().unwrap();
                        self.push_u64(val);
                    }

                    "int" => {
                        let val: i64 = val.parse::<i64>().unwrap();
                        self.push_i64(val);
                    }

                    "float" => {
                        let val: f64 = val.parse::<f64>().unwrap();
                        self.push_f64(val);
                    }

                    "str" => {
                        let s = if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
                            val[1..(val.len() - 1)].to_string()
                        } else {
                            val.clone()
                        };

                        let id = self.alloc_heap(HeapValue::Str(s));
                        self.push_ref(id);
                    }

                    "lambda" => {
                        let lambda_pos = self.labels.get(&val)
                            .unwrap_or_else(|| panic!("Label: {} is not found, at {}:{}:\n    {}", val, self.path, line_idx, raw)).clone();
                        self.push_u64(lambda_pos as u64);
                    }

                    _ => {
                        panic!("Unknown type {}, at {}:{}:\n    {}", dtype, self.path, line_idx, raw);
                    },
                }
            },

/* u_inc */ 3504395983 => {
                let a = self.pop_slot();
                self.push_u64(a + 1);
            },

/* u_dec */ 4196425563 => {
                let a = self.pop_slot();
                self.push_u64(a - 1);
            }

/* i_inc */ 3066074899 => {
                let a = self.pop_slot() as i64;
                self.push_i64(a + 1);
            },

/* i_dec */ 2261244279 => {
                let a = self.pop_slot() as i64;
                self.push_i64(a - 1);
            }

/* f_inc */ 3479561274 => {
                let a = f64::from_bits(self.pop_slot());
                self.push_f64(a + 1.0);
            }

/* f_dec */ 2117118482 => {
                let a = f64::from_bits(self.pop_slot());
                self.push_f64(a - 1.0);
            },

/* u_add */ 814136636 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.push_u64(a + b);
            },

/* u_sub */ 874937213 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.push_u64(a - b);
            },

/* u_mul */ 629139689 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.push_u64(a * b);
            },

/* u_div */ 3708006304 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.push_u64(a / b);
            }

/* u_mod */ 163603499 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.push_u64(a % b);
            }

/* i_add */ 1620772024 => {
                let b = self.pop_slot() as i64;
                let a = self.pop_slot() as i64;
                self.push_i64(a + b);
            },

/* i_sub */ 660410561 => {
                let b = self.pop_slot() as i64;
                let a = self.pop_slot() as i64;
                self.push_i64(a - b);
            }

/* i_mul */ 2048868125 => {
                let b = self.pop_slot() as i64;
                let a = self.pop_slot() as i64;
                self.push_i64(a * b);
            }

/* i_div */ 2176767804 => {
                let b = self.pop_slot() as i64;
                let a = self.pop_slot() as i64;
                self.push_i64(a / b);
            }

/* i_mod */ 2383434767 => {
                let b = self.pop_slot() as i64;
                let a = self.pop_slot() as i64;
                self.push_i64(a % b);
            }

/* f_add */ 1471602089 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.push_f64(f64::from_bits(a) + f64::from_bits(b));
            }

/* f_sub */ 2796889488 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.push_f64(f64::from_bits(a) - f64::from_bits(b));
            }

/* f_mul */ 1429630668 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.push_f64(f64::from_bits(a) * f64::from_bits(b));
            }

/* f_div */ 2335815909 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.push_f64(f64::from_bits(a) / f64::from_bits(b));
            }

/* f_mod */ 1358974598 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.push_f64(f64::from_bits(a) % f64::from_bits(b));
            }

/* store_var */533751560 => {
                if args.len() != 1 {
                    panic!("At {}:{}:\n    {}\nstore_var requires 1 argument;\nUsage: store_var <name>", self.path, line_idx, raw);
                }

                let name = args[0].clone();
                let val = self.pop_slot();

                self.current_mut_frame().insert(name, val);
            }

/* load_var */3095714799 => {
                if args.len() != 1 {
                    panic!("At {}:{}:\n    {}\nload_var requires 1 argument;\nUsage: load_var <name>", self.path, line_idx, raw);
                }

                let name = args[0].clone();

                let val = {
                    let frame = self.frame_stack.last().unwrap();
                    frame.get(&name).unwrap_or_else(|| panic!("Undefined variable {}, at {}:{}:\n    {}", name, self.path, line_idx, raw)).clone()
                };
                self.stack.push(val);
            }

/* print */ 372738696 => {
                if self.stack.len() != 0 {
                    let val = self.pop_slot();
                    match args[0].as_str() {
                        "unt" => println!("{}", val),
                        "int" => println!("{}", val as i64),
                        "float" => println!("{}", f64::from_bits(val)),
                        "str" => {
                            if let Some(HeapValue::Str(s)) = self.heap.get(&val) {
                                println!("{}", s);
                            } else {
                                panic!("Invalid heap reference: {}, at {}:{}:\n    {}", val, self.path, line_idx, raw);
                            }
                        }
                        _ => panic!("Unknown type: {}, at {}:{}:\n    {}", args[0], self.path, line_idx, raw)
                    }
                } else {
                    println!()
                }
            }

/* input */ 4191711099 => {
                if args.len() != 1 {
                    panic!("At {}:{}:\n    {}\ninput requires 1 argument;\nUsage: input <required_type>", self.path, line_idx, raw);
                }

                let required_type = &args[0];

                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read!");

                let input = input.trim();

                match required_type.as_str() {
                    "unt" => {
                        let val = input.parse::<u64>().unwrap();
                        self.push_u64(val);
                    }
                    "int" => {
                        let val = input.parse::<i64>().unwrap();
                        self.push_i64(val);
                    },
                    "float" => {
                        let val = input.parse::<f64>().unwrap();
                        self.push_f64(val);
                    },
                    "str" => {
                        let id = self.alloc_heap(HeapValue::Str(input.to_string()));
                        self.push_ref(id);
                    },
                    _ => panic!("Unknown type: {}, at {}:{}:\n    {}", required_type, self.path, line_idx, raw)
                }
            }

/* call */  3018949801 => {
                if args.len() != 1 {
                    panic!("At {}:{}:\n    {}\ncall requires 1 argument;\nUsage: call <label>", self.path, line_idx, raw);
                }

                let label = args[0].clone();
                if let Some(lab) = self.labels.get(&label) {
                    self.call_stack.push(self.ip);
                    self.frame_stack.push(HashMap::new());

                    self.ip = lab.clone() + 1;
                } else {
                    panic!("Label: {} is not found, at {}:{}:\n    {}", label, self.path, line_idx, raw);
                }
            }

/* call_dynamic */4082794239 => {
                let lambdas_pos = self.pop_slot() as usize;

                self.call_stack.push(self.ip);
                self.frame_stack.push(HashMap::new());

                self.ip = lambdas_pos + 1;
            }

/* ret */   821323692 => {
                let label = self.call_stack.pop()
                    .unwrap_or_else(|| panic!("ret is used without calling, at {}:{}:\n    {}", self.path, line_idx, raw));
                self.frame_stack.pop();

                self.ip = label;
            }

/* sleep */ 2313861896 => {
                let time = self.pop_slot();
                thread::sleep(Duration::from_millis(time));
            }

/* u_eq */  3848242203 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.stack.push(if a == b { 1 } else { 0 });
            },

/* u_neq */ 1377440367 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.stack.push(if a != b { 1 } else { 0 });
            },

/* u_lt */  3729666037 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.stack.push(if a < b { 1 } else { 0 });
            },

/* u_gt*/   3965979726 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.stack.push(if a > b { 1 } else { 0 });
            },

/* u_lte */ 2232737712 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.stack.push(if a <= b { 1 } else { 0 });
            },

/* u_gte */ 1283401649 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.stack.push(if a >= b { 1 } else { 0 });
            },

/* i_eq */  3360026535 => {
                let b = self.pop_slot() as i64;
                let a = self.pop_slot() as i64;
                self.stack.push(if a == b { 1 } else { 0 });
            }

/* i_neq */ 3473496443 => {
                let b = self.pop_slot() as i64;
                let a = self.pop_slot() as i64;
                self.stack.push(if a != b { 1 } else { 0 });
            }

/* i_lt */  3109892441 => {
                let b = self.pop_slot() as i64;
                let a = self.pop_slot() as i64;
                self.stack.push(if a < b { 1 } else { 0 });
            }

/* i_gt */  3075204370 => {
                let b = self.pop_slot() as i64;
                let a = self.pop_slot() as i64;
                self.stack.push(if a > b { 1 } else { 0 });
            }

/* i_lte */ 162824564 => {
                let b = self.pop_slot() as i64;
                let a = self.pop_slot() as i64;
                self.stack.push(if a <= b { 1 } else { 0 });
            }

/* i_gte */ 58341973 => {
                let b = self.pop_slot() as i64;
                let a = self.pop_slot() as i64;
                self.stack.push(if a >= b { 1 } else { 0 });
            }

/* f_eq */  3248363596 => {
                let b = f64::from_bits(self.pop_slot());
                let a = f64::from_bits(self.pop_slot());
                self.stack.push(if a == b { 1 } else { 0 });
            }

/* f_neq */ 371414550 => {
                let b = f64::from_bits(self.pop_slot());
                let a = f64::from_bits(self.pop_slot());
                self.stack.push(if a != b { 1 } else { 0 });
            }

/* f_lt */  3501160714 => {
                let b = f64::from_bits(self.pop_slot());
                let a = f64::from_bits(self.pop_slot());
                self.stack.push(if a < b { 1 } else { 0 });
            }

/* f_gt */  3533288929 => {
                let b = f64::from_bits(self.pop_slot());
                let a = f64::from_bits(self.pop_slot());
                self.stack.push(if a > b { 1 } else { 0 });
            }

/* f_lte */ 4080806333 => {
                let b = f64::from_bits(self.pop_slot());
                let a = f64::from_bits(self.pop_slot());
                self.stack.push(if a <= b { 1 } else { 0 });
            }

/* f_gte */ 200851148 => {
                let b = f64::from_bits(self.pop_slot());
                let a = f64::from_bits(self.pop_slot());
                self.stack.push(if a >= b { 1 } else { 0 });
            }

/* and */   254395046 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.stack.push(if a == 1 && b == 1 { 1 } else { 0 })
            }

/* or */    1563699588 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.stack.push(if a == 1 || b == 1 { 1 } else { 0 })
            }

/* not */   699505802 => {
                let a = self.pop_slot();
                self.stack.push(if a != 1 { 1 } else { 0 });
            }

/* jump */  2805947405 => {
                if args.len() != 1 {
                    panic!("At {}:{}:\n    {}\njump requires 1 argument;\nUsage: jump <label>", self.path, line_idx, raw);
                }

                let label_name = &args[0];
                self.ip = self.labels.get(label_name)
                    .unwrap_or_else(|| panic!("Label: {} is not found, at {}:{}:\n    {}", label_name, self.path, line_idx, raw))
                    .clone() + 1;
            }

/* jump_if_false */3471442001 => {
                if args.len() != 1 {
                    panic!("At {}:{}:\n    {}\njump_if_false requires 1 argument;\nUsage: jump_if_false <label>", self.path, line_idx, raw);
                }

                let cond = self.pop_slot();

                if cond != 1 {
                    let label_name = &args[0];
                    self.ip = self.labels.get(label_name)
                        .unwrap_or_else(|| panic!("Label: {} is not found, at {}:{}:\n    {}", label_name, self.path, line_idx, raw))
                        .clone() + 1;
                }
            }

/* jump_if_true */4207249494 => {
                if args.len() != 1 {
                    panic!("At {}:{}:\n    {}\njump_if_true requires 1 argument;\nUsage: jump_if_true <label>", self.path, line_idx, raw);
                }

                let cond = self.pop_slot();
                if cond == 1 {
                    let label_name = &args[0];
                    self.ip = self.labels.get(label_name)
                        .unwrap_or_else(|| panic!("Label: {} is not found, at {}:{}:\n    {}", label_name, self.path, line_idx, raw))
                        .clone() + 1;
                }
            }

/* label */ 4137097213 => {
                return;
            }

/* halt */  3904824570 => {
                let code = if args.len() == 1 {
                    args[0].parse::<i32>().unwrap().clone()
                } else {
                    0
                };

                println!("[Stack]\n{:?}", self.stack);
                println!("[Frame-Stack]\n{:?}", self.frame_stack);
                println!("[Heap]\n{:?}", self.heap);
                println!("Took: {:?}", now.elapsed());

                process::exit(code);
            }

            _ => panic!("Unknown instruction, at {}:{}:\n    {}", self.path, self.ip, instr.raw),
        }
    }
}

fn main() {
    let mut args = env::args();
    let _prog = args.next();
    let path = match args.next() {
        Some(p) => p,
        None => panic!("No path provided"),
    };

    let data = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => panic!("Unable to read file: {};\nError: {}", path, e),
    };

    let start = Instant::now();

    let mut lvm = LVM::new(path);
    lvm.parse_and_load(&data).unwrap();
    lvm.run(start);
}
#[allow(non_snake_case)]
pub mod Loader {
    use std::collections::HashMap;
    use crate::{ClassInfo, Instruction};
    use crate::utils::Utils;
    
    pub fn parse(path: &str, source: String) -> Vec<Instruction> {
        let mut instructions: Vec<Instruction> = vec![];

        let mut i = 0;

        for raw_line in source.lines() {
            let mut line = raw_line.trim();

            if line.is_empty() {
                continue;
            }

            if let Some(idx) = line.find('#') {
                line = &line[..idx].trim();
            } else if let Some(idx) = line.find("//") {
                line = &line[..idx].trim();
            } else if let Some(idx) = line.find(';') {
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
                                in_quote = false;
                                break;
                            }
                        } else {
                            panic!("Unterminated quote at {}:{}:\n    {}", path, i, raw_line)
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

            let opcode = Utils::opcode_hash(parts[0].to_lowercase().as_str());
            let args = if parts.len() > 1 {
                parts[1..].to_vec()
            } else {
                vec![]
            };

            let instr = Instruction {
                op: opcode,
                args,
                raw: raw_line.to_string(),
                line_idx: i,
            };
            instructions.push(instr);

            i += 1;
        }

        instructions
    }

    pub fn collect_labels(path: &str, instructions: &Vec<Instruction>) -> HashMap<String, usize> {
        let mut labels = HashMap::new();

        for instr in instructions.iter() {
            if instr.op == 4137097213 {
                /* LABEL */

                if instr.args.len() != 1 {
                    panic!("At {}:{}:\n    {}\nlabel expects 1 argument;\nUsage: label <name>", path, instr.line_idx, instr.raw)
                }

                let mut name = instr.args[0].clone();

                if name.ends_with(':') {
                    name.pop();
                }

                if labels.contains_key(&name) {
                    panic!("Label: \"{}\" already defined, at {}:{}:\n    {}", name, path, instr.line_idx, instr.raw)
                }

                labels.insert(name, instr.line_idx);
            }
        }

        labels
    }

    pub fn collect_classes(path: &str, instructions: &Vec<Instruction>) -> HashMap<u64, ClassInfo> {
        let mut classes: HashMap<u64, ClassInfo> = HashMap::new();

        for instr in instructions.iter() {
            if instr.op == 2462236192 {
                /* STRUCT */

                if instr.args.len() < 1 {
                    panic!("At {}:{}:\n    {}\nclass expects as minimum 1 argument;\nUsage: struct <name>: [field1], [field2], ...", path, instr.line_idx, instr.raw)
                }

                let mut name = instr.args[0].clone();
                if name.ends_with(':') {
                    name.pop();
                }

                let class_hash = Utils::class_hash(&name);

                if classes.contains_key(&class_hash) {
                    panic!("Class: \"{}\" already defined, at {}:{}:\n    {}", name, path, instr.line_idx, instr.raw)
                }

                let info = ClassInfo {
                    class: name,
                    fields: instr.args[1..].to_vec().clone(),
                };

                classes.insert(class_hash, info);
            }
        }

        classes
    }
}
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::process;
use std::thread;
use std::time::{Duration, Instant};
use std::alloc::{alloc, dealloc, Layout, handle_alloc_error};
use std::slice;
use std::ptr;

#[derive(Debug, Clone)]
struct Instruction {
    op: u32,
    args: Vec<String>,
    line_idx: usize,
    raw: String,
}

#[derive(Debug, Clone)]
struct ClassInfo {
    class: String,
    fields: Vec<String>,
}

#[derive(Debug, Clone)]
struct HeapEntry {
    ptr: *mut u8,
    size: usize,
    kind: u8,
}

#[derive(Debug, Clone)]
struct LVM {
    call_stack: Vec<usize>,
    classes: HashMap<u64, ClassInfo>,
    class_positions: HashMap<String, usize>,
    frame_stack: Vec<HashMap<String, u64>>,
    heap: HashMap<u64, HeapEntry>,
    instructions: Vec<Instruction>,
    ip: usize,
    labels: HashMap<String, usize>,
    next_heap_id: u64,
    path: String,
    stack: Vec<u64>,
}

impl LVM {
    fn new(path: String) -> LVM {
        LVM {
            call_stack: Vec::new(),
            classes: HashMap::new(),
            class_positions: HashMap::new(),
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

    fn class_hash(s: &str) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in s.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x100000001b32);
        }
        h
    }

    fn alloc_heap_bytes(&mut self, size: usize, kind: u8) -> u64 {
        let layout = Layout::from_size_align(size, 8).unwrap();
        unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                handle_alloc_error(layout);
            }


            let id = self.next_heap_id;
            self.next_heap_id += 1;

            self.heap.insert(
                id,
                HeapEntry {
                    ptr,
                    size,
                    kind,
                }
            );

            id
        }
    }

    fn free_heap_bytes(&mut self, id: u64) {
        if let Some(entry) = self.heap.remove(&id) {

            let layout = Layout::from_size_align(entry.size, 8).unwrap();
            unsafe {
                dealloc(entry.ptr, layout);
            }
        } else {
            panic!("free_heap_bytes:\n    Id of object: {} is not found in heap", id);
        }
    }

    fn write_u64_at(&mut self, id: u64, offset_slots: usize, value: u64) {
        let entry = self.heap.get(&id)
            .unwrap_or_else(|| panic!("write_u64_at:\n    Id: {} not found in heap", id));
        let byte_offset = offset_slots.checked_mul(8).unwrap();

        if byte_offset + 8 > entry.size {
            panic!("write_u64_at:\n    out of bounds write (id: {}, offset: {}, entry.size: {}, byte_offset: {})", id, offset_slots, entry.size, byte_offset);
        }

        unsafe {
            let dest = entry.ptr.add(byte_offset) as *mut u64;
            ptr::write_unaligned(dest, value);
        }
    }

    fn read_u64_at(&self, id: u64, offset_slots: usize) -> u64 {
        let entry = self.heap.get(&id)
            .unwrap_or_else(|| panic!("read_u64_at:\n    Id: {} not found in heap", id));
        let byte_offset = offset_slots.checked_mul(8).unwrap();

        if byte_offset + 8 > entry.size {
            panic!("read_u64_at:\n    out of bounds read (id: {}, offset: {})", id, offset_slots);
        }

        unsafe {
            let src = entry.ptr.add(byte_offset) as *const u64;
            ptr::read_unaligned(src)
        }
    }

    fn alloc_object(&mut self, class_hash: u64, field_count: usize) -> u64 {
        let size = 16 + field_count * 8;

        let id = self.alloc_heap_bytes(size, 4);

        self.write_u64_at(id, 0, class_hash);
        self.write_u64_at(id, 1, field_count as u64);

        id
    }

    fn alloc_array(&mut self, len: usize) -> u64 {
        if len > 2147483647 {
            let info = &self.instructions[self.ip - 1];
            panic!("Array length too big: {}!\nAt {}:{}:\n    {}", len, self.path, info.line_idx, info.raw);
        }

        let size = 8 + len * 8;

        let id = self.alloc_heap_bytes(size, 3);
        self.write_u64_at(id, 0, len as u64);

        for i in 0..len {
            self.write_u64_at(id, i + 1, 0);
        }

        id
    }

    fn alloc_str(&mut self, s: &str) -> u64 {
        let bytes = s.as_bytes();
        let len = bytes.len();
        let mut total = 8 + len;

        if total % 8 != 0 {
            total += 8 - (total % 8);
        }

        let id = self.alloc_heap_bytes(total, 2);
        self.write_u64_at(id, 0, len as u64);

        unsafe {
            let entry = self.heap.get(&id).unwrap();
            let dest = entry.ptr.add(8);
            ptr::copy_nonoverlapping(bytes.as_ptr(), dest, len);

            let pad = total - 8 - len;

            if pad > 0 {
                ptr::write_bytes(dest.add(len), 0, pad);
            }
        }

        id
    }

    fn alloc_num(&mut self, v: u64) -> u64 {
        let id = self.alloc_heap_bytes(8, 1);
        self.write_u64_at(id, 0, v);
        id
    }

    fn read_string(&self, id: u64) -> String {
        let entry = self.heap.get(&id)
            .unwrap_or_else(|| panic!("read_string:\n    Id: {} not found in heap", id));

        match entry.kind {
            2 => {
                let len = self.read_u64_at(id, 0) as usize;

                unsafe {
                    let src = entry.ptr.add(8);
                    let slice = slice::from_raw_parts(src, len);
                    String::from_utf8_lossy(slice).into_owned()
                }
            },
            _ => panic!("read_string:\n    Id: {} is not String", id),
        }
    }

    fn current_mut_frame(&mut self) -> &mut HashMap<String, u64> {
        self.frame_stack.last_mut().unwrap()
    }

    fn get_label(&mut self, label: &String) -> usize {
        self.labels.get(label)
            .unwrap_or_else(|| panic!("Label: {} is not found, at {}:{}:\n    {}", label, self.path, self.ip, self.instructions[self.ip].raw))
            .clone()
    }

    fn push_u64(&mut self, val: u64) {
        self.stack.push(val);
    }

    fn push_i64(&mut self, val: i64) {
        self.stack.push(val as u64);
    }

    fn push_f64(&mut self, val: f64) {
        self.stack.push(val.to_bits());
    }

    fn push_ref(&mut self, val: u64) {
        self.stack.push(val);
    }

    fn pop_slot(&mut self) -> u64 {
        match self.stack.pop() {
            Some(v) => v,
            None => panic!("No elements in stack!\nAt {}:{}:\n    {}", self.path, self.ip, self.instructions[self.ip].raw),
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

            let opcode = LVM::opcode_hash(parts[0].to_lowercase().as_str());
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
        }

        self.collect_labels_and_classes(&instructions);

        self.instructions = instructions;

        Ok(())
    }

    fn collect_labels_and_classes(&mut self, instructions: &Vec<Instruction>) {
        for (idx, instr) in instructions.iter().enumerate() {
/* LABEL */ if instr.op == 4137097213 {

                if instr.args.len() != 1 {
                    panic!("At {}:{}:\n    {}\nlabel expects 1 argument;\nUsage: label <name>", self.path, idx, instr.raw)
                }

                let mut name = instr.args[0].clone();

                if name.ends_with(':') {
                    name.pop();
                }

                if self.labels.contains_key(&name) {
                    panic!("Label: \"{}\" already defined, at {}:{}:\n    {}", name, self.path, idx, instr.raw)
                }

                self.labels.insert(name, idx);
/* STRUCT */ } else if instr.op == 2462236192 {

                if instr.args.len() < 1 {
                    panic!("At {}:{}:\n    {}\nclass expects as minimum 1 argument;\nUsage: struct <name>: [field1], [field2], ...", self.path, idx, instr.raw)
                }

                let mut name = instr.args[0].clone();
                name.pop();

                if self.class_positions.contains_key(&name) {
                    panic!("Class: \"{}\" already defined, at {}:{}:\n    {}", name, self.path, idx, instr.raw)
                }

                self.class_positions.insert(name, idx);
            }
        }
    }

    fn load_class_if_needed(&mut self, class_name: String) {
        let class_hash = LVM::class_hash(&class_name);

        if self.classes.contains_key(&class_hash) {
            return;
        }

        let start_idx = match self.class_positions.get(&class_name) {
            Some(idx) => *idx,
            None => panic!("Class: {} is not found, at {}:{}", class_name, self.path, self.ip)
        };

        let mut args = self.instructions[start_idx].args.clone();
        args.remove(0);

        let info = ClassInfo {
            class: class_name.clone(),
            fields: args,
        };

        self.classes.insert(class_hash, info);
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
                let hash_type = LVM::opcode_hash(dtype);
                let val = args[1..].join(" ");

                match hash_type {
            /* unt */1255446122 => {
                        let val: u64 = val.parse::<u64>().unwrap();
                        self.push_u64(val);
                    }

            /* int */2515107422 => {
                        let val: i64 = val.parse::<i64>().unwrap();
                        self.push_i64(val);
                    }

            /* float */2797886853 => {
                        let val: f64 = val.parse::<f64>().unwrap();
                        self.push_f64(val);
                    }

            /* str */3259748752 => {
                        let s = if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
                            val[1..(val.len() - 1)].to_string()
                        } else {
                            val.clone()
                        };

                        let id = self.alloc_str(&s);
                        self.push_ref(id);
                    }

            /* lambda */2131063274 => {
                        let lambda_pos = self.get_label(&val);
                        self.push_u64(lambda_pos as u64);
                    },

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

/* str_add */2122984124 => {
                let b = self.pop_slot();
                let a = self.pop_slot();

                let entry_b = self.heap.get(&b)
                    .unwrap_or_else(|| panic!("Cannot found string ref: {} in heap, at {}:{}:\n    {}", b, self.path, line_idx, raw)).clone();
                let entry_a = self.heap.get(&a)
                    .unwrap_or_else(|| panic!("Cannot found string ref: {} in heap, at {}:{}:\n    {}", a, self.path, line_idx, raw)).clone();


                let len_b = self.read_u64_at(b, 0) as usize;
                let len_a = self.read_u64_at(a, 0) as usize;

                let total = len_a + len_b;

                let id = self.alloc_heap_bytes(8 + total, 2);
                self.write_u64_at(id, 0, total as u64);

                unsafe {
                    let entry = self.heap.get(&id).unwrap();
                    let dest = entry.ptr.add(8);

                    let src_a = entry_a.ptr.add(8);
                    ptr::copy_nonoverlapping(src_a, dest, len_a);

                    let src_b = entry_b.ptr.add(8);
                    ptr::copy_nonoverlapping(src_b, dest.add(len_a), len_b);
                }

                self.push_ref(id);
            }

/* str_set */2349001035 => {
                let s = self.pop_slot();
                let entry_s = self.heap.get(&s)
                    .unwrap_or_else(|| panic!("Cannot found string ref: {} in heap, at {}:{}:\n    {}", s, self.path, line_idx, raw)).clone();

                let idx = self.pop_slot() as usize;

                let sym = {
                    let raw_v = self.pop_slot();

                    if raw_v > u8::MAX as u64 {
                        panic!("Num: {} is more than 8 Bit max value, at {}:{}:\n    {}", raw, self.path, line_idx, raw);
                    }

                    raw_v as u8
                };

                match entry_s.kind {
                    2 => {
                        let len_s = self.read_u64_at(s, 0) as usize;

                        if idx >= len_s {
                            panic!("Index out of bounds:\n    index={}, length={}\n at {}:{}:\n    {}", idx, len_s, self.path, line_idx, raw);
                        }

                        unsafe {
                            let dest = entry_s.ptr.add(8);
                            ptr::write(dest.add(idx), sym);
                        }
                    }

                    _ => panic!("Expected string for str_set, but got {}, at {}:{}:\n    {}", entry_s.kind.to_string(), self.path, line_idx, raw),
                }
            }

/* str_get */2539503519 => {
                let s = self.pop_slot();
                let entry_s = self.heap.get(&s)
                    .unwrap_or_else(|| panic!("Cannot found string ref: {} in heap, at {}:{}:\n    {}", s, self.path, line_idx, raw)).clone();

                let idx = self.pop_slot() as usize;

                match entry_s.kind {
                    2 => {
                        let len_s = self.read_u64_at(s, 0) as usize;

                        if idx >= len_s {
                            panic!("Index out of bounds:\n    index={}, length={}\n at {}:{}:\n    {}", idx, len_s, self.path, line_idx, raw);
                        }

                        unsafe {
                            let src = entry_s.ptr.add(8);
                            self.push_u64(*src.add(idx) as u64)
                        }
                    }

                    _ => panic!("Expected string for str_set, but got {}, at {}:{}:\n    {}", entry_s.kind.to_string(), self.path, line_idx, raw),
                }
            }

/* str_len */3689876820 => {
                let s = self.pop_slot();

                let entry = self.heap.get(&s)
                    .unwrap_or_else(|| panic!("Cannot found string ref: {} in heap, at {}:{}:\n    {}", s, self.path, line_idx, raw));

                match entry.kind {
                    2 => {
                        let len = self.read_u64_at(s, 0);
                        self.push_u64(len);
                    }

                    _ => panic!("Expected string for str_len, but got {}, at {}:{}:\n    {}", entry.kind.to_string(), self.path, line_idx, raw),
                }
            }

/* str_bytes */3717867548 => {
                let s = self.pop_slot();

                let entry = self.heap.get(&s)
                    .unwrap_or_else(|| panic!("Cannot found string ref: {} in heap, at {}:{}:\n    {}", s, self.path, line_idx, raw)).clone();

                match entry.kind {
                    2 => {
                        let len = self.read_u64_at(s, 0) as usize;

                        let bytes_id = self.alloc_array(len);

                        unsafe {
                            let src = entry.ptr.add(8);

                            for i in 0..len {
                                let b = *src.add(i);
                                self.write_u64_at(bytes_id, i + 1, b as u64);
                            }
                        }

                        self.push_ref(bytes_id);
                    }

                    _ => panic!("Expected string for str_bytes, but got {}, at {}:{}:\n    {}", entry.kind.to_string(), self.path, line_idx, raw),
                }
            }

/* str_eq */1149816987 => {
                let b = self.pop_slot();
                let a = self.pop_slot();

                let entry_b = self.heap.get(&b)
                    .unwrap_or_else(|| panic!("Cannot found string ref: {} in heap, at {}:{}:\n    {}", b, self.path, line_idx, raw));
                let entry_a = self.heap.get(&a)
                    .unwrap_or_else(|| panic!("Cannot found string ref: {} in heap, at {}:{}:\n    {}", a, self.path, line_idx, raw));

                let len_b = self.read_u64_at(a, 0) as usize;
                let len_a = self.read_u64_at(a, 0) as usize;

                if len_a != len_b {
                    self.push_u64(0);
                } else {
                    let mut equal = true;

                    unsafe {
                        let src_a = entry_a.ptr.add(8);
                        let src_b = entry_b.ptr.add(8);

                        for i in 0..len_a {
                            if *src_a.add(i) != *src_b.add(i) {
                                equal = false;
                                break;
                            }
                        }
                    }

                    self.push_u64(if equal { 1 } else { 0 });
                }
            }

/* str_neq */2686287855 => {
                let b = self.pop_slot();
                let a = self.pop_slot();

                let entry_b = self.heap.get(&b)
                    .unwrap_or_else(|| panic!("Cannot found string ref: {} in heap, at {}:{}:\n    {}", b, self.path, line_idx, raw));
                let entry_a = self.heap.get(&a)
                    .unwrap_or_else(|| panic!("Cannot found string ref: {} in heap, at {}:{}:\n    {}", a, self.path, line_idx, raw));

                let len_b = self.read_u64_at(a, 0) as usize;
                let len_a = self.read_u64_at(a, 0) as usize;

                if len_a != len_b {
                    self.push_u64(1);
                } else {
                    let mut not_equal = false;

                    unsafe {
                        let src_a = entry_a.ptr.add(8);
                        let src_b = entry_b.ptr.add(8);

                        for i in 0..len_a {
                            if *src_a.add(i) != *src_b.add(i) {
                                not_equal = true;
                                break;
                            }
                        }
                    }

                    self.push_u64(if not_equal { 1 } else { 0 });
                }
            }

/* str_upper */2975764495 => {
                let s = self.pop_slot();
                let entry = self.heap.get(&s)
                    .unwrap_or_else(|| panic!("Cannot found string ref: {} in heap, at {}:{}:\n    {}", s, self.path, line_idx, raw)).clone();

                let len = self.read_u64_at(s, 0) as usize;

                let id = self.alloc_heap_bytes(8 + len, 2);
                self.write_u64_at(id, 0, len as u64);

                unsafe {
                    let src = entry.ptr.add(8);

                    let dest_entry = self.heap.get(&id).unwrap();
                    let dest = dest_entry.ptr.add(8);

                    for i in 0..len {
                        let ch = *src.add(i);

                        *dest.add(i) = if ch >= b'a' && ch <= b'z' {
                            ch - 32
                        } else {
                            ch
                        };
                    }
                }

                self.push_ref(id);
            }

/* str_lower */871400802 => {
                let s = self.pop_slot();
                let entry = self.heap.get(&s)
                    .unwrap_or_else(|| panic!("Cannot found string ref: {} in heap, at {}:{}:\n    {}", s, self.path, line_idx, raw)).clone();

                let len = self.read_u64_at(s, 0) as usize;

                let id = self.alloc_heap_bytes(8 + len, 2);
                self.write_u64_at(id, 0, len as u64);

                unsafe {
                    let src = entry.ptr.add(8);

                    let dest_entry = self.heap.get(&id).unwrap();
                    let dest = dest_entry.ptr.add(8);

                    for i in 0..len {
                        let ch = *src.add(i);

                        *dest.add(i) = if ch >= b'A' && ch <= b'Z' {
                            ch + 32
                        } else {
                            ch
                        };
                    }
                }

                self.push_ref(id);
            }

/* cast */  2854572110 => {
                if args.len() != 2 {
                    panic!("At {}:{}:\n    {}\ncast requires 1 argument;\nUsage: cast <from_type> <to_type>", self.path, line_idx, raw);
                }
                let val = self.pop_slot();
                let from = args.get(0).map(|s| s.as_str()).unwrap_or("");
                let to   = args.get(1).map(|s| s.as_str()).unwrap_or("");

                let hashed_from = LVM::opcode_hash(from);
                let hashed_to = LVM::opcode_hash(to);

                let casted = match (hashed_from, hashed_to) {
    /* unt - int */ (1255446122, 2515107422)   => val as i64 as u64,
    /* unt - float */(1255446122, 2797886853) => (val as f64).to_bits(),

    /* int - unt */ (2515107422, 1255446122)   => val as i64 as u64,
    /* int - float */(2515107422, 2797886853) => (val as i64 as f64).to_bits(),

    /* float - int */(2797886853, 2515107422)   => (f64::from_bits(val).trunc() as i64) as u64,
    /* float - unt */(2797886853, 1255446122)   => f64::from_bits(val).trunc() as u64,

                    _ => panic!("Unsupported cast: {} -> {}, at {}:{}:\n    {}",
                                from, to, self.path, line_idx, raw),
                };

                self.push_u64(casted);
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
                    frame.get(&name).unwrap_or_else(|| panic!("Undefined variable: {}, at {}:{}:\n    {}", name, self.path, line_idx, raw)).clone()
                };

                self.push_u64(val);
            }

/* free */  2578706139 => {

                if args.len() != 1 {
                    panic!("At {}:{}:\n    {}\nfree requires 1 argument;\nUsage: free <name>", self.path, line_idx, raw);
                }

                let name = args[0].clone();

                if let Some(obj_ref) = self.current_mut_frame().remove(&name) {
                    self.free_heap_bytes(obj_ref);
                } else {
                    panic!("Undefined variable: {}, at {}:{}:\n    {}", name, self.path, line_idx, raw);
                }
            }

/* clone */ 730356610 => {
                let reference = self.pop_slot();

                let kind = {
                    let entry = self.heap.get(&reference)
                        .unwrap_or_else(|| panic!("Cannot found ref: {} in heap, at {}:{}:\n    {}", reference, self.path, line_idx, raw)).clone();

                    entry.kind.clone()
                };

                match kind {

                    1 => {
                        let num = self.read_u64_at(reference, 0);
                        let new_id = self.alloc_num(num);
                        self.push_ref(new_id);
                    }

                    2 => {
                        let s = self.read_string(reference);
                        let new_id = self.alloc_str(&s);
                        self.push_ref(new_id);
                    }

                    3 => {
                        let len = self.read_u64_at(reference, 0) as usize;
                        let new_id = self.alloc_array(len);

                        for i in 0..len {
                            let v = self.read_u64_at(reference, i);
                            self.write_u64_at(new_id, i, v);
                        }
                        self.push_ref(new_id);
                    }

                    4 => {
                        let class_hash = self.read_u64_at(reference, 0);
                        let field_count = self.read_u64_at(reference, 1) as usize;

                        let new_id = self.alloc_object(class_hash, field_count);

                        for i in 0..field_count {
                            let v = self.read_u64_at(reference, i);
                            self.write_u64_at(new_id, i, v);
                        }

                        self.push_ref(new_id);
                    }

                    _ => {}
                }
            },

/* print */ 372738696 => {
                if self.stack.len() != 0 {

                    if args.len() != 1 {
                        panic!("At {}:{}:\n    {}\nprint requires 1 argument due to stack is not empty;\nUsage: print <type>", self.path, line_idx, raw);
                    }

                    let val = self.pop_slot();
                    let hashed_type = LVM::opcode_hash(&args[0]);

                    match hashed_type {
                /* unt */1255446122 => println!("{}", val),
                /* int */2515107422 => println!("{}", val as i64),
                /* float */2797886853 => println!("{}", f64::from_bits(val)),
                /* char */2823553821 => println!("{}", val as u8 as char),
                /* ref */1123320834 => {
                            let entry = self.heap.get(&val)
                                .unwrap_or_else(|| panic!("Cannot found object reference: {} in heap, at {}:{}:\n    {}", val, self.path, line_idx, raw));

                            match entry.kind {

                                1 => {
                                    let v = self.read_u64_at(val, 0);
                                    println!("{}", v);
                                },

                                2 => {
                                    let s = self.read_string(val);
                                    println!("{}", s);
                                },

                                3 => {
                                    let mut items = Vec::new();
                                    let len = self.read_u64_at(val, 0) as usize;

                                    for i in 1..len+1 {
                                        let v = self.read_u64_at(val, i);
                                        items.push(format!("{}", v));
                                    }

                                    println!("[{}]", items.join(", "))
                                },

                                4 => {

                                    let class_hash = self.read_u64_at(val, 0);
                                    let field_count = self.read_u64_at(val, 1) as usize;

                                    print!("{} {{ ", self.classes.get(&class_hash).unwrap().class);

                                    let mut parts = Vec::new();

                                    for i in 2..field_count+2 {
                                        let v = self.read_u64_at(val, i);
                                        parts.push(format!("f{}: {}", i, v));
                                    }


                                    print!("{}", parts.join(", "));
                                    println!(" }}");
                                },

                                _ => {}
                            }
                        },
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
                let required_type_hash = LVM::opcode_hash(required_type);

                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read!");

                let input = input.trim();

                match required_type_hash {
            /* unt */1255446122 => {
                        let val = input.parse::<u64>().unwrap();
                        self.push_u64(val);
                    },
            /* int */2515107422 => {
                        let val = input.parse::<i64>().unwrap();
                        self.push_i64(val);
                    },
            /* float */2797886853 => {
                        let val = input.parse::<f64>().unwrap();
                        self.push_f64(val);
                    },
            /* str */3259748752 => {
                        let id = self.alloc_str(input);
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
                self.call_stack.push(self.ip);
                self.frame_stack.push(HashMap::new());

                self.ip = self.get_label(&label) + 1;
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
            },

/* new */   681154065 => {
                if args.len() != 2 {
                    panic!("At {}:{}:\n    {}\nnew requires 2 arguments;\nUsage: new <class> <init_label>", self.path, line_idx, raw);
                }

                let class_name = args[0].clone();
                let class_hash = LVM::class_hash(&class_name);
                let init_label = args[1].clone();

                self.load_class_if_needed(class_name.clone());

                let class_info = self.classes.get(&class_hash).unwrap();
                let field_count = class_info.fields.len();

                let obj_id = self.alloc_object(class_hash, field_count);

                for i in 0..field_count {
                    self.write_u64_at(obj_id, i + 2, 0);
                }

                self.push_u64(obj_id);
                self.call_stack.push(self.ip);
                self.frame_stack.push(HashMap::new());
                self.ip = self.get_label(&init_label) + 1;
            },

/* set_field */2059520392 => {
                if args.len() != 1 {
                    panic!("At {}:{}:\n    {}\nset_field requires 1 argument;\nUsage: set_field <name>", self.path, line_idx, raw);
                }
                
                let field_name = args[0].clone();
                let obj_ref = self.pop_slot();
                let val = self.pop_slot();

                let entry = self.heap.get(&obj_ref)
                    .unwrap_or_else(|| panic!("Cannot found object reference: {} in heap, at {}:{}:\n    {}", obj_ref, self.path, line_idx, raw));

                match &entry.kind {
                    4 => {
                        let class_hash = self.read_u64_at(obj_ref, 0);

                        let class_info = self.classes.get(&class_hash).unwrap();
                        let idx = class_info.fields.iter()
                            .position(|n| n == &field_name)
                            .unwrap_or_else(|| panic!("field: {} is not found in Class: {}, at {}:{}:\n    {}", field_name, class_info.class, self.path, line_idx, raw));

                        self.write_u64_at(obj_ref, idx + 2, val);
                    },

                    _ => panic!("Not a Object: {}, at {}:{}:\n    {}", obj_ref, self.path, line_idx, raw),
                }
            },

/* load_field */1285198278 => {
                if args.len() != 1 {
                    panic!("At {}:{}:\n    {}\nload_field requires 1 argument;\nUsage: load_field <name>", self.path, line_idx, raw);
                }

                let field_name = args[0].clone();
                let obj_ref = self.pop_slot();

                let entry = self.heap.get(&obj_ref)
                    .unwrap_or_else(|| panic!("Cannot found object reference: {} in heap, at {}:{}:\n    {}", obj_ref, self.path, line_idx, raw));

                match &entry.kind {
                    4 => {
                        let class_hash = self.read_u64_at(obj_ref, 0);

                        let class_info = self.classes.get(&class_hash).unwrap();
                        let idx = class_info.fields.iter()
                            .position(|n| n == &field_name)
                            .unwrap_or_else(|| panic!("field: {} is not found in Class: {}, at {}:{}:\n    {}", field_name, class_info.class, self.path, line_idx, raw));

                        let v = self.read_u64_at(obj_ref, idx + 2);
                        self.push_u64(v);
                    },

                    _ => panic!("Not a Object: {}, at {}:{}:\n    {}", obj_ref, self.path, line_idx, raw)
                }
            },

/* sleep */ 2313861896 => {
                let time = self.pop_slot();
                thread::sleep(Duration::from_millis(time));
            },

/* new_array */3719752907 => {
                let len = self.pop_slot() as usize;

                let id = self.alloc_array(len);

                self.push_ref(id);
            }

/* array_set */1287122249 => {
                let arr_ref = self.pop_slot();
                let idx = self.pop_slot() as usize;
                let val = self.pop_slot();

                let entry = self.heap.get(&arr_ref)
                    .unwrap_or_else(|| panic!("Cannot found array reference: {} in heap, at {}:{}:\n    {}", arr_ref, self.path, line_idx, raw));

                match &entry.kind {
                    3 => {
                        let len = self.read_u64_at(arr_ref, 0) as usize;

                        if idx >= len {
                            panic!("Index out of bounds:\n    index={}, length={}\n at {}:{}:\n    {}", idx, len, self.path, line_idx, raw)
                        }

                        self.write_u64_at(arr_ref, idx + 1, val);
                    },

                    _ => panic!("Not an Array: {}, at {}:{}:\n    {}", arr_ref, self.path, line_idx, raw)
                }
            },

/* array_get */3467232181 => {
                let arr_ref = self.pop_slot();
                let idx = self.pop_slot() as usize;

                let entry = self.heap.get(&arr_ref)
                    .unwrap_or_else(|| panic!("Cannot found array reference: {} in heap, at {}:{}:\n    {}", arr_ref, self.path, line_idx, raw));

                match &entry.kind {
                    3 => {
                        let len = self.read_u64_at(arr_ref, 0) as usize;

                        if idx >= len {
                            panic!("Index out of bounds:\n    index={}, length={}\n at {}:{}:\n    {}", idx, len, self.path, line_idx, raw)
                        }

                        let val = self.read_u64_at(arr_ref, idx + 1);
                        self.push_u64(val);
                    },

                    _ => panic!("Not an Array: {}, at {}:{}:\n    {}", arr_ref, self.path, line_idx, raw)

                }
            }

/* array_len */3246697146 => {
                let arr_ref = self.pop_slot();

                let entry = self.heap.get(&arr_ref)
                    .unwrap_or_else(|| panic!("Cannot found array reference: {} in heap, at {}:{}:\n    {}", arr_ref, self.path, line_idx, raw));

                match &entry.kind {
                    3 => {
                        let len = self.read_u64_at(arr_ref, 0);
                        self.push_u64(len);
                    },

                    _ => panic!("Not an Array: {}, at {}:{}:\n    {}", arr_ref, self.path, line_idx, raw)

                }
            }

/* u_eq */  3848242203 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.push_u64(if a == b { 1 } else { 0 });
            },

/* u_neq */ 1377440367 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.push_u64(if a != b { 1 } else { 0 });
            },

/* u_lt */  3729666037 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.push_u64(if a < b { 1 } else { 0 });
            },

/* u_gt*/   3965979726 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.push_u64(if a > b { 1 } else { 0 });
            },

/* u_lte */ 2232737712 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.push_u64(if a <= b { 1 } else { 0 });
            },

/* u_gte */ 1283401649 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.push_u64(if a >= b { 1 } else { 0 });
            },

/* i_eq */  3360026535 => {
                let b = self.pop_slot() as i64;
                let a = self.pop_slot() as i64;
                self.push_u64(if a == b { 1 } else { 0 });
            }

/* i_neq */ 3473496443 => {
                let b = self.pop_slot() as i64;
                let a = self.pop_slot() as i64;
                self.push_u64(if a != b { 1 } else { 0 });
            }

/* i_lt */  3109892441 => {
                let b = self.pop_slot() as i64;
                let a = self.pop_slot() as i64;
                self.push_u64(if a < b { 1 } else { 0 });
            }

/* i_gt */  3075204370 => {
                let b = self.pop_slot() as i64;
                let a = self.pop_slot() as i64;
                self.push_u64(if a > b { 1 } else { 0 });
            }

/* i_lte */ 162824564 => {
                let b = self.pop_slot() as i64;
                let a = self.pop_slot() as i64;
                self.push_u64(if a <= b { 1 } else { 0 });
            }

/* i_gte */ 58341973 => {
                let b = self.pop_slot() as i64;
                let a = self.pop_slot() as i64;
                self.push_u64(if a >= b { 1 } else { 0 });
            }

/* f_eq */  3248363596 => {
                let b = f64::from_bits(self.pop_slot());
                let a = f64::from_bits(self.pop_slot());
                self.push_u64(if a == b { 1 } else { 0 });
            }

/* f_neq */ 371414550 => {
                let b = f64::from_bits(self.pop_slot());
                let a = f64::from_bits(self.pop_slot());
                self.push_u64(if a != b { 1 } else { 0 });
            }

/* f_lt */  3501160714 => {
                let b = f64::from_bits(self.pop_slot());
                let a = f64::from_bits(self.pop_slot());
                self.push_u64(if a < b { 1 } else { 0 });
            }

/* f_gt */  3533288929 => {
                let b = f64::from_bits(self.pop_slot());
                let a = f64::from_bits(self.pop_slot());
                self.push_u64(if a > b { 1 } else { 0 });
            }

/* f_lte */ 4080806333 => {
                let b = f64::from_bits(self.pop_slot());
                let a = f64::from_bits(self.pop_slot());
                self.push_u64(if a <= b { 1 } else { 0 });
            }

/* f_gte */ 200851148 => {
                let b = f64::from_bits(self.pop_slot());
                let a = f64::from_bits(self.pop_slot());
                self.push_u64(if a >= b { 1 } else { 0 });
            }

/* and */   254395046 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.push_u64(if a == 1 && b == 1 { 1 } else { 0 })
            }

/* or */    1563699588 => {
                let b = self.pop_slot();
                let a = self.pop_slot();
                self.push_u64(if a == 1 || b == 1 { 1 } else { 0 })
            }

/* not */   699505802 => {
                let a = self.pop_slot();
                self.push_u64(if a != 1 { 1 } else { 0 });
            }

/* jump */  2805947405 => {
                if args.len() != 1 {
                    panic!("At {}:{}:\n    {}\njump requires 1 argument;\nUsage: jump <label>", self.path, line_idx, raw);
                }

                let label_name = args[0].clone();
                self.ip = self.get_label(&label_name) + 1;
            }

/* jump_if_false */3471442001 => {
                if args.len() != 1 {
                    panic!("At {}:{}:\n    {}\njump_if_false requires 1 argument;\nUsage: jump_if_false <label>", self.path, line_idx, raw);
                }

                let cond = self.pop_slot();

                if cond != 1 {
                    let label_name = args[0].clone();

                    self.ip = self.get_label(&label_name) + 1;
                }
            }

/* jump_if_true */4207249494 => {
                if args.len() != 1 {
                    panic!("At {}:{}:\n    {}\njump_if_true requires 1 argument;\nUsage: jump_if_true <label>", self.path, line_idx, raw);
                }

                let cond = self.pop_slot();
                if cond == 1 {
                    let label_name = args[0].clone();

                    self.ip = self.get_label(&label_name) + 1;
                }
            }

/* jump_if_null */238760827 => {
                if args.len() != 1 {
                    panic!("At {}:{}:\n    {}\njump_if_null requires 1 argument;\nUsage: jump_if_null <label>", self.path, line_idx, raw);
                }

                let val = self.pop_slot();
                if val == 0 {
                    let label_name = args[0].clone();

                    self.ip = self.get_label(&label_name) + 1;
                }
            },

/* label */ 4137097213 => {
                return;
            }

/* halt */  3904824570 => {
                let code = if args.len() == 1 {
                    args[0].parse::<i32>().unwrap().clone()
                } else if !self.stack.is_empty() {
                    self.pop_slot() as i32
                } else {
                    0
                };

                println!("[Stack]\n{:?}", self.stack);
                println!("[Frame-Stack]\n{:?}", self.frame_stack);
                println!("[Heap]\n{:?}", self.heap);
                println!("[Classes]\n{:?}", self.classes);
                println!("Took: {:?}", now.elapsed());

                process::exit(code);
            }

            _ => panic!("Unknown instruction, at {}:{}:\n    {}", self.path, self.ip, instr.raw),
        }
    }
}

fn print_op_hash(op: &str) {
    if op == "" {
        return;
    }

    let hash = LVM::opcode_hash(op);
    println!("{}: {}", op, hash);
}

fn print_size_of<T>(class: &str) {
    if class == "" {
        return;
    }

    let size_of = size_of::<T>();
    println!("Size of {}: {}", class, size_of);
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

    print_op_hash("");
    print_size_of::<i32>("");
    
    let start = Instant::now();
    
    let mut lvm = LVM::new(path);
    lvm.parse_and_load(&data).unwrap();
    lvm.run(start);
}
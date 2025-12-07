#[allow(non_snake_case)]
pub mod Utils {
    pub fn opcode_hash(s: &str) -> u32 {
        let mut h: u32 = 0x811c9dc5;
        for b in s.as_bytes() {
            h ^= *b as u32;
            h = h.wrapping_mul(0x01000193);
        }
        h
    }

    pub fn class_hash(s: &str) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in s.as_bytes() {
            h ^= *b as u64;
            h = h.wrapping_mul(0x100000001b32);
        }
        h
    }
    
    pub fn print_op_hash(op: &str) {
        if op == "" {
            return;
        }

        let hash = opcode_hash(op);
        println!("{}: {}", op, hash);
    }

    pub fn print_size_of<T>(class: &str) {
        if class == "" {
            return;
        }

        let size_of = size_of::<T>();
        println!("Size of {}: {}", class, size_of);
    }
}
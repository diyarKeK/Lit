pub fn escape_llvm(s: &str) -> String {
    s.chars().flat_map(|c| {
        let b = c as u32;
        if b == b'"' as u32 || b == b'\\' as u32 || b < 0x20 || b > 0x7e {
            format!("\\{:02X}", b).chars().collect::<Vec<_>>()
        } else {
            vec![c]
        }
    }).collect()
}
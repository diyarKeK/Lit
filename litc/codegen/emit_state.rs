pub struct EmitState {
    reg: usize,
    str_idx: usize,
}

impl EmitState {
    #[inline]
    pub fn new() -> EmitState {
        EmitState { reg: 0, str_idx: 0 }
    }

    pub fn next_reg(&mut self) -> usize {
        let r = self.reg;
        self.reg += 1;
        r
    }

    pub fn next_str_idx(&mut self) -> usize {
        let i = self.str_idx;
        self.str_idx += 1;
        i
    }
}
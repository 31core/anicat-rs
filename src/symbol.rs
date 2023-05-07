#[derive(Default)]
pub struct Symbols {
    internal_syms: Vec<(usize, u64)>,
    internal_refs: Vec<(usize, u64)>,
    external_syms: Vec<(String, u64)>,
    external_refs: Vec<(String, u64)>,
}

impl Symbols {
    pub fn new() -> Self {
        Symbols {
            internal_syms: Vec::new(),
            internal_refs: Vec::new(),
            external_syms: Vec::new(),
            external_refs: Vec::new(),
        }
    }
    /// Add a symbol
    pub fn add_external_sym(&mut self, symbol: &str, addr: u64) -> Result<(), String> {
        if self.lookup(symbol).is_some() {
            return Err(format!("'{}' has already defined", symbol));
        }
        self.external_syms.push((symbol.to_string(), addr));
        Ok(())
    }
    /// Allocate an internal symbol
    pub fn alloc_internal_sym(&mut self, addr: u64) -> usize {
        let id = self.internal_syms.len();
        self.internal_syms.push((id, addr));
        id
    }
    /// Add a reference
    pub fn external_ref(&mut self, symbol: &str, addr: u64) -> Result<(), String> {
        if self.lookup(symbol).is_none() {
            return Err(format!("'{}' not defined", symbol));
        }
        self.external_refs.push((symbol.to_string(), addr));
        Ok(())
    }
    /// Add an internal reference
    pub fn internal_ref(&mut self, symbol: usize, addr: u64) {
        self.internal_refs.push((symbol, addr));
    }
    /// Add an internal reference
    pub fn modify_internal_sym(&mut self, id: usize, addr: u64) {
        self.internal_syms[id].1 = addr;
    }
    pub fn link(&self, byte_code: &mut [u8]) {
        for sym in &self.external_refs {
            let addr = self.lookup(&sym.0).unwrap();
            for i in 0..8 {
                byte_code[sym.1 as usize + i] = addr.to_be_bytes()[i];
            }
        }

        for ref_i in &self.internal_refs {
            let addr = self.internal_syms[ref_i.0].1;
            for i in 0..8 {
                byte_code[ref_i.1 as usize + i] = addr.to_be_bytes()[i];
            }
        }
    }
    /// lookup external symbol
    pub fn lookup(&self, name: &str) -> Option<u64> {
        for i in &self.external_syms {
            if i.0 == name {
                return Some(i.1);
            }
        }
        None
    }
}

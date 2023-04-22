#[derive(Clone, Debug, Default)]
pub struct VRAM {
    pub area: Vec<u8>,
    pub size: u64,
}

impl VRAM {
    pub fn new(size: u64) -> Self {
        VRAM {
            area: vec![0; size as usize],
            size,
        }
    }
    /// load to VRAM
    pub fn load(&mut self, addr: u64, size: u64, data: &[u8]) {
        if addr + size > self.size {
            panic!("VRAM overflow");
        }
        for (i, byte) in data.iter().enumerate() {
            self.area[addr as usize + i] = *byte;
        }
    }
    /// dump from VRAM
    pub fn dump(&self, addr: u64, size: u64) -> &[u8] {
        if addr + size > self.size {
            panic!("VRAM overflow");
        }
        &self.area[addr as usize..(addr + size) as usize]
    }
}

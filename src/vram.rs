#[derive(Clone, Debug)]
pub struct VRAM {
    pub area: Vec<u8>,
}

impl VRAM {
    pub fn new(size: usize) -> Self {
        VRAM {
            area: vec![0; size],
        }
    }
    /// load to VRAM
    pub fn load(&mut self, addr: u64, size: u64, data: &[u8]) {
        for i in 0..size as usize {
            self.area[addr as usize + i] = data[i];
        }
    }
    /// dump from VRAM
    pub fn dump(&self, addr: u64, size: u64) -> &[u8] {
        &self.area[addr as usize..(addr + size) as usize]
    }
}

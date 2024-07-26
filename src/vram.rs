#[derive(Clone, Debug, Default)]
pub struct Vram {
    pub area: Vec<u8>,
    pub size: u64,
}

impl Vram {
    pub fn new(size: u64) -> Self {
        Self {
            area: vec![0; size as usize],
            size,
        }
    }
    /** load to VRAM */
    pub fn load(&mut self, addr: u64, data: &[u8]) {
        if addr + data.len() as u64 > self.size {
            panic!("VRAM overflow");
        }
        self.area[addr as usize..addr as usize + data.len()].copy_from_slice(data);
    }
    /** dump from VRAM */
    pub fn dump(&self, addr: u64, size: u64) -> &[u8] {
        if addr + size > self.size {
            panic!("VRAM overflow");
        }
        &self.area[addr as usize..(addr + size) as usize]
    }
}

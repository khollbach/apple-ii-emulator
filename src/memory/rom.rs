/// $d000..=$ffff
pub struct Rom {
    /// $d000..$f800
    applesoft: &'static [u8; 0x2800],
    /// $f800..=$ffff
    f8: &'static [u8; 0x800],
}

impl Rom {
    pub fn new() -> Self {
        Self {
            applesoft: include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/rom/Applesoft")),
            f8: include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/rom/Unenh_IIe_F8ROM")),
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0xcfff => panic!("address out of bounds for ROM: ${addr:04x}"),
            0xd000..=0xf7ff => self.applesoft[addr as usize - 0xd000],
            0xf800..=0xffff => self.f8[addr as usize - 0xf800],
        }
    }
}

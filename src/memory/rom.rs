/// $d000..=$ffff
pub struct Rom {
    /// $d000..$f800
    applesoft: &'static [u8; 0x2800],
    /// $f800..=$ffff
    f8: &'static [u8; 0x800],
}

// TODO: remove this hack once bank-switching is impl'd
static mut EXTRA_RAM: [u8; 256] = [0u8; 256];

impl Rom {
    pub fn new() -> Self {
        let this = Self {
            applesoft: include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/rom/Applesoft")),
            f8: include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/rom/Unenh_IIe_F8ROM")),
        };
        unsafe {
            for i in 0..256 {
                EXTRA_RAM[i] = this.f8[0xff00 - 0xf800 + i];
            }
        }
        this
    }

    pub fn read(&self, addr: u16) -> u8 {
        if addr >= 0xff00 {
            unsafe {
                return EXTRA_RAM[addr as u8 as usize];
            }
        }

        match addr {
            0x0000..=0xcfff => panic!("address out of bounds for ROM: ${addr:04x}"),
            0xd000..=0xf7ff => self.applesoft[addr as usize - 0xd000],
            0xf800..=0xffff => self.f8[addr as usize - 0xf800],
        }
    }

    pub fn write(&self, addr: u16, value: u8) {
        // Otherwise, tron program crashes.
        // (Maybe it's trying to use bank-switched RAM?) Yeah, seems like.
        if addr >= 0xff00 {
            // eprintln!("{addr:04x} {value:02x}");
            unsafe {
                EXTRA_RAM[addr as u8 as usize] = value;
            }
            return;
        }

        panic!("cannot set value of ROM. addr ${addr:04x} value ${value:02x}");
    }
}

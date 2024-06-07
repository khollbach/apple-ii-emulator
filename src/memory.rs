mod load_program;

/// Interface between the CPU and the memory address space (including RAM, ROM,
/// and I/O).
pub trait Memory {
    fn get(&mut self, addr: u16) -> u8;

    fn set(&mut self, addr: u16, value: u8);

    fn get_word(&mut self, addr: u16) -> u16 {
        let lo = self.get(addr);
        let hi = self.get(addr.checked_add(1).unwrap());
        u16::from_le_bytes([lo, hi])
    }
}

#[derive(Clone)]
pub struct Mem {
    ram: Vec<u8>,
}

impl Memory for Mem {
    fn get(&mut self, addr: u16) -> u8 {
        self.trigger_soft_switches(addr);
        self.ram[addr as usize]
    }

    fn set(&mut self, addr: u16, value: u8) {
        self.trigger_soft_switches(addr);
        self.ram[addr as usize] = value;
    }
}

impl Mem {
    pub fn new(program: &[u8], load_addr: u16) -> Self {
        load_program::load_program(program, load_addr)
    }

    pub fn gr_page1(&self) -> &[u8] {
        &self.ram[0x400..0x800]
    }

    pub fn hgr_page1(&self) -> &[u8] {
        &self.ram[0x2000..0x4000]
    }

    fn trigger_soft_switches(&mut self, addr: u16) {
        if addr == 0xc010 {
            // Clear keyboard strobe.
            self.ram[0xc000] &= 0x7f;
        }
    }

    /// temporary hack: run RESET routine, but skip disk loading code.
    /// This sets up required state for keyboard input routines, etc.
    /// Once we figure out how interrupts work, we can re-visit this.
    pub fn set_softev(&mut self, start_addr: u16) -> u16 {
        assert_eq!(self.ram[0x03f2..][..3], [0, 0, 0]);

        let [lo, hi] = start_addr.to_le_bytes();
        self.ram[0x03f2] = lo;
        self.ram[0x03f3] = hi;
        self.ram[0x03f4] = 0xa5 ^ self.ram[0x03f3]; // magic number to indicate "warm start"

        let pc = 0xfa62; // RESET
        pc
    }
}

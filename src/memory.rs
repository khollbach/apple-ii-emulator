mod io;
mod rom;

use std::mem;

use io::{Io, SoftSwitch};
use rom::Rom;

use crate::display::{color::Color, gr, hgr, text};

/// Everything in the memory address space (including RAM, ROM, and I/O).
pub struct AddressSpace {
    /// $0000..$c000
    main_ram: Box<[u8; 0xc000]>,
    /// $c000..$d000
    io: Io,
    /// $d000..=$ffff
    rom: Rom,
}

impl AddressSpace {
    pub fn new(program: &[u8], load_addr: u16) -> Self {
        let mut main_ram = Box::new([0u8; 0xc000]);
        main_ram[load_addr as usize..][..program.len()].copy_from_slice(program);

        Self {
            main_ram,
            io: Io::new(),
            rom: Rom::new(),
        }
    }

    /// temporary hack: run RESET routine, but skip disk loading code.
    /// This sets up required state for keyboard input routines, etc.
    /// Once we figure out how interrupts work, we can re-visit this.
    pub fn set_softev(&mut self, start_addr: u16) -> u16 {
        assert_eq!(self.main_ram[0x03f2..][..3], [0, 0, 0]);

        let [lo, hi] = start_addr.to_le_bytes();
        self.main_ram[0x03f2] = lo;
        self.main_ram[0x03f3] = hi;
        self.main_ram[0x03f4] = 0xa5 ^ self.main_ram[0x03f3]; // magic number to indicate "warm start"

        let pc = 0xfa62; // RESET
        pc
    }

    pub fn get(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0xbfff => self.main_ram[addr as usize],
            0xc000..=0xcfff => self.io.read(addr),
            0xd000..=0xffff => self.rom.read(addr),
        }
    }

    pub fn set(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0xbfff => self.main_ram[addr as usize] = value,
            0xc000..=0xcfff => self.io.write(addr, value),
            0xd000..=0xffff => self.rom.write(addr, value),
        }
    }

    pub fn display(&self) -> Vec<Vec<Color>> {
        let page = match (
            self.io.soft_switch(SoftSwitch::Hires),
            self.io.soft_switch(SoftSwitch::Page2),
        ) {
            (false, false) => &self.main_ram[0x400..0x800],
            (false, true) => &self.main_ram[0x800..0xc00],
            (true, false) => &self.main_ram[0x2000..0x4000],
            (true, true) => &self.main_ram[0x4000..0x6000],
        };

        if self.io.soft_switch(SoftSwitch::Text) {
            return text::dots(page);
        }

        let mut dots = if self.io.soft_switch(SoftSwitch::Hires) {
            hgr::dots_bw(page); // swap these if you want B&W display
            hgr::dots_color(page)
        } else {
            gr::dots(page)
        };

        if self.io.soft_switch(SoftSwitch::Mixed) {
            let mut text_dots = text::dots(page);
            for y in 20 * text::CELL_H..24 * text::CELL_H {
                dots[y] = mem::take(&mut text_dots[y]);
            }
        }

        dots
    }

    pub fn key_down(&mut self, ascii_code: u8) {
        self.io.key_down(ascii_code);
    }

    pub fn all_keys_up(&mut self) {
        self.io.all_keys_up();
    }
}

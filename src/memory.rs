mod io;
mod rom;

use std::{
    io::{self as std_io, Read},
    mem,
};

use anyhow::{Context, Result};
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

    /// Language card RAM:
    /// $d000..=$ffff
    lc_ram: Box<[u8; 0x4000]>,
    /// Language card RAM, bank 2:
    /// $d000..$e000
    lc_bank_2: Box<[u8; 0x1000]>,
}

impl AddressSpace {
    pub fn new(program: &[u8], load_addr: u16) -> Self {
        let mut main_ram = Box::new([0u8; 0xc000]);
        main_ram[load_addr as usize..][..program.len()].copy_from_slice(program);

        Self {
            main_ram,
            io: Io::new(),
            rom: Rom::new(),
            lc_ram: Box::new([0u8; 0x4000]),
            lc_bank_2: Box::new([0u8; 0x1000]),
        }
    }

    /// The memory image file format is that of llvm-mos. From their docs:
    ///
    /// The image file is a collection of blocks. Each block consists of a
    /// 16-bit starting address, then a 16-bit block size, then that many bytes
    /// of contents. Both the address and size are stored little-endian.
    pub fn from_memory_image(mut image: &[u8]) -> Result<(Self, u16)> {
        let mut main_ram = Box::new([0u8; 0xc000]);
        let mut start_addr = None;

        // todo: it would be nice to check that no blocks overlap
        // * see my old code for loading programs, for ideas

        while !image.is_empty() {
            let mut word = [0u8; 2];
            image.read_exact(&mut word).context("eof during header")?;
            let offset = u16::from_le_bytes(word) as usize;
            image.read_exact(&mut word).context("eof during header")?;
            let length = u16::from_le_bytes(word) as usize;

            eprintln!("${:04x} ${:04x}", offset, length);

            if (offset, length) == (0xfffa, 6) {
                // special case: setting start_addr (via reset vector)
                let mut buf = [0u8; 6];
                image.read_exact(&mut buf)?;
                assert_eq!(buf[..2], [0, 0]);
                start_addr = Some(u16::from_le_bytes(buf[2..4].try_into().unwrap()));
                assert_eq!(buf[4..], [0, 0]);
            } else {
                image.read_exact(&mut main_ram[offset..][..length])?; // can panic
            }
        }

        // todo: hack: init stack pointer, since llvm-mos seems not to do it correctly
        main_ram[0] = 0x00;
        main_ram[1] = 0xc0; // $0000..$c000 <- RAM     (above that, it's IO / ROM / etc)

        Ok((
            Self {
                main_ram,
                io: Io::new(),
                rom: Rom::new(),
                lc_ram: Box::new([0u8; 0x4000]),
                lc_bank_2: Box::new([0u8; 0x1000]),
            },
            start_addr.unwrap(),
        ))
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

    // TODO: use bank select soft switches to decide which RAM/ROM to read/write

    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0xbfff => self.main_ram[addr as usize],
            0xc000..=0xcfff => self.io.read(addr),
            0xd000..=0xffff => self.rom.read(addr),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
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

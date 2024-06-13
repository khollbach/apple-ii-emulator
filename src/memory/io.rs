//
// Soft switch locations taken from Apple //e TRM, Page 29.
//

// TODO: is there a better way to refactor the soft-switch code?

// const _ALTCHAR_OFF: u16 = 0xc00e;
// const _ALTCHAR_ON: u16 = 0xc00f;
// const _READ_ALTCHAR: u16 = 0xc01e;

// const _80COL_OFF: u16 = 0xc00c;
// const _80COL_ON: u16 = 0xc00d;
// const _READ_80COL: u16 = 0xc01f;

const PAGE2_OFF: u16 = 0xc054;
const PAGE2_ON: u16 = 0xc055;
const PAGE2_READ: u16 = 0xc01c;

const TEXT_OFF: u16 = 0xc050;
const TEXT_ON: u16 = 0xc051;
const TEXT_READ: u16 = 0xc01a;

const MIXED_OFF: u16 = 0xc052;
const MIXED_ON: u16 = 0xc053;
const MIXED_READ: u16 = 0xc01b;

const HIRES_OFF: u16 = 0xc056;
const HIRES_ON: u16 = 0xc057;
const HIRES_READ: u16 = 0xc01d;

/// $c000..$d000
pub struct Io {
    /// $c100..$c400
    c100_rom: Box<[u8; 0x300]>,
    /// $c800..=$cffe
    c800_rom: Box<[u8; 0x800 - 1]>,

    /// $c000
    most_recent_key: u8,
    /// $c000 hibit
    strobe_bit: bool,
    /// $c010 hibit
    any_key_down: bool,

    // todo: refactor soft switches somehow.
    pub page2: bool,
    pub text: bool,
    pub mixed: bool,
    pub hires: bool,
}

impl Io {
    pub fn new() -> Self {
        let rom = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/rom/Unenh_IIe_80col"));
        Self {
            c100_rom: Box::new(rom[..0x300].try_into().unwrap()),
            c800_rom: Box::new(rom[0x300..].try_into().unwrap()),

            most_recent_key: 0u8,
            strobe_bit: false,
            any_key_down: false,

            page2: false,
            text: false,
            mixed: false,
            hires: false,
        }
    }

    pub fn key_down(&mut self, ascii_code: u8) {
        assert!(ascii_code < 0x80);
        self.most_recent_key = ascii_code;
        self.strobe_bit = true;
        self.any_key_down = true;
    }

    pub fn all_keys_up(&mut self) {
        self.any_key_down = false;
    }

    pub fn get(&mut self, addr: u16) -> u8 {
        match addr {
            0xc000 => {
                let mut byte = self.most_recent_key;
                if self.strobe_bit {
                    byte |= 0x80;
                }
                byte
            }
            0xc010 => {
                self.strobe_bit = false;
                if self.any_key_down {
                    0x80
                } else {
                    0
                }
            }

            0xc100..=0xc3ff => self.c100_rom[addr as usize - 0xc100],
            0xc800..=0xcffe => self.c800_rom[addr as usize - 0xc800],

            PAGE2_OFF => {
                self.page2 = false;
                0
            }
            PAGE2_ON => {
                self.page2 = true;
                0
            }
            PAGE2_READ => {
                if self.page2 {
                    0x80
                } else {
                    0
                }
            }

            TEXT_OFF => {
                self.text = false;
                0
            }
            TEXT_ON => {
                self.text = true;
                0
            }
            TEXT_READ => {
                if self.text {
                    0x80
                } else {
                    0
                }
            }

            MIXED_OFF => {
                self.mixed = false;
                0
            }
            MIXED_ON => {
                self.mixed = true;
                0
            }
            MIXED_READ => {
                if self.mixed {
                    0x80
                } else {
                    0
                }
            }

            HIRES_OFF => {
                self.hires = false;
                0
            }
            HIRES_ON => {
                self.hires = true;
                0
            }
            HIRES_READ => {
                if self.hires {
                    0x80
                } else {
                    0
                }
            }

            // Hacks to make the ROM code not crash:
            // todo: maybe some of these are soft switches?
            0xc015 | 0xc018 | 0xc01f | 0xc058 | 0xc05a | 0xc05d | 0xc05f | 0xc030 | 0xc062
            | 0xc061 | 0xcfff | 0xc081 | 0xc080 | 0xc082 => 0,

            // see bank select switches, Table 4-6, page 82 of TRM
            _ => panic!("${addr:04x}"),
        }
    }

    pub fn set(&mut self, addr: u16, value: u8) {
        match addr {
            0xc010 => self.strobe_bit = false,

            // todo: how to reduce code dup w/ get ?
            TEXT_OFF => self.text = false,
            TEXT_ON => self.text = true,
            MIXED_OFF => self.mixed = false,
            MIXED_ON => self.mixed = true,
            HIRES_OFF => self.hires = false,
            HIRES_ON => self.hires = true,

            // Hacks to make the ROM code not crash:
            // todo: maybe some of these are soft switches?
            0xc007 | 0xc001 | 0xc055 | 0xc054 | 0xc000 | 0xc006 => (),

            _ => panic!("${addr:04x} ${value:02x}"),
        }
    }
}

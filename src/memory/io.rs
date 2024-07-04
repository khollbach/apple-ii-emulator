mod soft_switches;

pub use soft_switches::SoftSwitch;
use soft_switches::SoftSwitches;

/// $c000..$d000
pub struct Io {
    /// $c100..$c400
    c100_rom: Box<[u8; 0x300]>,
    /// $c400..$c800
    self_test_rom: Box<[u8; 0x400]>,
    /// $c800..=$cffe
    c800_rom: Box<[u8; 0x800 - 1]>,

    /// $c000 (without hibit)
    most_recent_key: u8,
    /// $c000 hibit
    strobe_bit: bool,
    /// $c010 hibit
    any_key_down: bool,

    /// $c000..$c100
    switches: SoftSwitches,
}

impl Io {
    pub fn new() -> Self {
        let rom = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/rom/Unenh_IIe_80col"));
        let self_test = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/rom/unenh_self_test"));

        Self {
            c100_rom: Box::new(rom[..0x300].try_into().unwrap()),
            self_test_rom: Box::new(*self_test),
            c800_rom: Box::new(rom[0x300..].try_into().unwrap()),

            most_recent_key: 0u8,
            strobe_bit: false,
            any_key_down: false,

            switches: SoftSwitches::new(),
        }
    }

    pub fn soft_switch(&self, switch: SoftSwitch) -> bool {
        self.switches.is_set(switch)
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

    pub fn read(&mut self, addr: u16) -> u8 {
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

            // Hacks to make these programs not crash.
            // (todo: presumably these are soft switches?)
            // * tron
            0xc015 | 0xc058 | 0xc05a | 0xc05d | 0xc062 | 0xc061 | 0xc030 => 0,

            // todo: bank select
            0xc081 | 0xc080 | 0xc082 => 0,

            0xc000..=0xc0ff => self.switches.read(addr),

            0xcfff => 0, // todo: what's this byte supposed to be?

            0xc100..=0xc3ff => self.c100_rom[addr as usize - 0xc100],
            0xc400..=0xc7ff => self.self_test_rom[addr as usize - 0xc400],
            0xc800..=0xcffe => self.c800_rom[addr as usize - 0xc800],

            _ => panic!("${addr:04x}"),
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xc010 => self.strobe_bit = false,

            // Hacks to make the tron program not crash:
            0xc007 | 0xc006 => (),

            0xc000..=0xc0ff => self.switches.write(addr),

            _ => panic!("${addr:04x} ${value:02x}"),
        }
    }
}

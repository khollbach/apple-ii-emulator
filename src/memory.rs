#[derive(Clone)]
pub struct Memory {
    pub ram: Vec<u8>,
}

impl Memory {
    pub fn load_program(prog: &[u8], load_addr: u16) -> Self {
        struct Slice<'a> {
            offset: usize,
            bytes: &'a [u8],
        }

        let mut slices = vec![];
        slices.push(Slice {
            bytes: prog,
            offset: load_addr.into(),
        });

        let rom_f8 = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/rom/Unenh_IIe_F8ROM"));
        debug_assert_eq!(rom_f8.len(), 0x800);
        slices.push(Slice {
            bytes: rom_f8,
            offset: 0xf800,
        });

        let rom_applesoft = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/rom/Applesoft"));
        debug_assert_eq!(rom_applesoft.len(), 0x2800);
        slices.push(Slice {
            bytes: rom_applesoft,
            offset: 0xd000,
        });

        let rom_80col = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/rom/Unenh_IIe_80col"));
        debug_assert_eq!(rom_80col.len(), 0x300 + 0x800 - 1);
        slices.push(Slice {
            bytes: &rom_80col[..0x300],
            offset: 0xc100,
        });
        slices.push(Slice {
            bytes: &rom_80col[0x300..],
            offset: 0xc800,
        });

        // Check the slices don't overlap.
        slices.sort_by_key(|s| s.offset);
        for w in slices.windows(2) {
            let [s1, s2] = w else { unreachable!() };
            assert!(s1.offset + s1.bytes.len() <= s2.offset);
        }

        let mut ram = vec![0; 2_usize.pow(16)];
        for s in slices {
            ram[s.offset..][..s.bytes.len()].copy_from_slice(s.bytes);
        }
        Self { ram }
    }

    pub fn get(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    pub fn set(&mut self, addr: u16, value: u8) {
        self.ram[addr as usize] = value;
    }
}

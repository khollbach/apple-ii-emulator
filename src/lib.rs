mod flags;
mod instr;
mod opcode;
mod operand;

use instr::{Instr, Mode};
use operand::Operand;

pub const MEM_LEN: usize = 2_usize.pow(16);

pub struct Cpu {
    pc: u16,
    sp: u8,
    flags: u8,

    a: u8,
    x: u8,
    y: u8,

    ram: Vec<u8>,
}

impl Cpu {
    pub fn new(ram: Vec<u8>) -> Self {
        assert_eq!(ram.len(), MEM_LEN);
        Self {
            pc: 0,
            sp: u8::MAX,
            flags: 0u8,
            a: 0,
            x: 0,
            y: 0,
            ram,
        }
    }

    pub fn run(mut self) -> Vec<u8> {
        loop {
            self.step();
        }
    }

    /// If the CPU would "halt" gracefully, this exits and returns the contents
    /// of ram. This can be useful for debugging purposes.
    pub fn run_until_halt(mut self) -> Vec<u8> {
        while !self.would_halt() {
            self.step();
        }
        self.ram
    }

    /// Hack for testing: detect a "halt" instruction.
    fn would_halt(&self) -> bool {
        let [lo, hi] = self.pc.to_le_bytes();
        let jmp_absolute = 0x4c;
        let halt = [jmp_absolute, lo, hi];
        &self.ram[self.pc as usize..][..3] == &halt
    }

    fn step(&mut self) {
        let (instr, mode) = opcode::decode(self.ram[self.pc as usize]);

        let loc = Operand::from_mode(self, mode);

        let curr_pc = self.pc; // Addr of currently executing instr.
        self.pc = self.pc.checked_add(mode.instr_len()).unwrap();

        match instr {
            Instr::Brk => panic!("brk at {:#04x}", curr_pc),
            Instr::Nop => (),

            Instr::Tax => self.x = self.nz(self.a),
            Instr::Txa => self.a = self.nz(self.x),
            Instr::Tay => self.y = self.nz(self.a),
            Instr::Tya => self.a = self.nz(self.y),
            Instr::Dex => self.x = self.nz(self.x.wrapping_sub(1)),
            Instr::Inx => self.x = self.nz(self.x.wrapping_add(1)),
            Instr::Dey => self.y = self.nz(self.y.wrapping_sub(1)),
            Instr::Iny => self.y = self.nz(self.y.wrapping_add(1)),

            Instr::Txs => self.sp = self.x,
            Instr::Tsx => self.x = self.nz(self.sp),
            Instr::Pha => self.push(self.a),
            Instr::Pla => {
                let v = self.pop();
                self.a = self.nz(v);
            }
            Instr::Php => self.push(self.flags),
            Instr::Plp => {
                let v = self.pop();
                self.flags = self.nz(v);
            }

            Instr::Clc => flags::clear(&mut self.flags, flags::CARRY),
            Instr::Sec => flags::set(&mut self.flags, flags::CARRY),
            Instr::Cli => flags::clear(&mut self.flags, flags::INTERRUPT),
            Instr::Sei => flags::set(&mut self.flags, flags::INTERRUPT),
            Instr::Clv => flags::clear(&mut self.flags, flags::OVERFLOW),
            Instr::Cld => flags::clear(&mut self.flags, flags::DECIMAL),
            Instr::Sed => flags::set(&mut self.flags, flags::DECIMAL),

            Instr::Adc => self.a = self.nz(self.a.wrapping_add(loc.get(self))),
            Instr::Sbc => self.a = self.nz(self.a.wrapping_sub(loc.get(self))),
            Instr::And => self.a &= self.nz(loc.get(self)),
            Instr::Ora => self.a |= self.nz(loc.get(self)),
            Instr::Eor => self.a ^= self.nz(loc.get(self)),

            Instr::Lda => self.a = self.nz(loc.get(self)),
            Instr::Ldx => self.x = self.nz(loc.get(self)),
            Instr::Ldy => self.y = self.nz(loc.get(self)),

            Instr::Sta => loc.set(self, self.a),
            Instr::Stx => loc.set(self, self.x),
            Instr::Sty => loc.set(self, self.y),

            Instr::Asl => {
                let v = self.nz(loc.get(self) << 1);
                loc.set(self, v);
            }
            Instr::Lsr => {
                let v = self.nz(loc.get(self) >> 1);
                loc.set(self, v);
            }
            Instr::Rol => {
                let mut v = loc.get(self);
                v <<= 1;
                v |= flags::is_set(self.flags, flags::CARRY) as u8;
                self.nz(v);
                loc.set(self, v);
            }
            Instr::Ror => {
                let mut v = loc.get(self);
                v >>= 1;
                if flags::is_set(self.flags, flags::CARRY) {
                    v |= 0x80;
                }
                self.nz(v);
                loc.set(self, v);
            }

            Instr::Inc => {
                let v = self.nz(loc.get(self).wrapping_add(1));
                loc.set(self, v);
            }
            Instr::Dec => {
                let v = self.nz(loc.get(self).wrapping_sub(1));
                loc.set(self, v);
            }
            Instr::Bit => todo!(),

            Instr::Cmp => todo!(),
            Instr::Cpx => todo!(),
            Instr::Cpy => todo!(),

            Instr::Bpl => self.branch(loc.addr(), flags::NEGATIVE, false),
            Instr::Bmi => self.branch(loc.addr(), flags::NEGATIVE, true),
            Instr::Bvc => self.branch(loc.addr(), flags::OVERFLOW, false),
            Instr::Bvs => self.branch(loc.addr(), flags::OVERFLOW, true),
            Instr::Bcc => self.branch(loc.addr(), flags::CARRY, false),
            Instr::Bcs => self.branch(loc.addr(), flags::CARRY, true),
            Instr::Bne => self.branch(loc.addr(), flags::ZERO, false),
            Instr::Beq => self.branch(loc.addr(), flags::ZERO, true),

            Instr::Jmp => self.pc = loc.addr(),

            Instr::Jsr => {
                let return_addr_minus_one = curr_pc.checked_add(2).unwrap();
                self.push2(return_addr_minus_one);

                self.pc = loc.addr();
            }

            Instr::Rts => self.pc = self.pop2().checked_add(1).unwrap(),

            Instr::Rti => {
                self.flags = self.pop();

                // Note that unlike RTS, there is no off-by-one here.
                self.pc = self.pop2();
            }
        }
    }

    fn get_byte(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    fn set_byte(&mut self, addr: u16, value: u8) {
        self.ram[addr as usize] = value;
    }

    fn get_word(&self, addr: u16) -> u16 {
        let lo = self.get_byte(addr);
        let hi = self.get_byte(addr.checked_add(1).unwrap());
        let word = u16::from_le_bytes([lo, hi]);
        word
    }

    /// Push to the stack.
    fn push(&mut self, value: u8) {
        let addr = 0x0100 + self.sp as u16;
        self.ram[addr as usize] = value;
        self.sp = self.sp.wrapping_sub(1);
    }

    /// Pop from the stack.
    fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_sub(1);
        let addr = 0x0100 + self.sp as u16;
        self.ram[addr as usize]
    }

    /// Push a word to the stack.
    fn push2(&mut self, value: u16) {
        let [lo, hi] = u16::to_le_bytes(value);

        // The stack grows down, so this stores the bytes in little-endian
        // order in RAM.
        self.push(hi);
        self.push(lo);
    }

    /// Pop a word from the stack.
    fn pop2(&mut self) -> u16 {
        let lo = self.pop();
        let hi = self.pop();
        u16::from_le_bytes([lo, hi])
    }

    /// Update negative and zero flags, based on the value.
    ///
    /// Return the value, for convenience.
    fn nz(&mut self, value: u8) -> u8 {
        flags::set_to(&mut self.flags, flags::ZERO, value == 0);
        flags::set_to(&mut self.flags, flags::NEGATIVE, (value as i8) < 0);
        value
    }

    fn branch(&mut self, addr: u16, flag: u8, when: bool) {
        let is_set = flags::is_set(self.flags, flag);
        if is_set == when {
            self.pc = addr;
        }
    }
}

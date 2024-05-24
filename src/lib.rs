mod flags;
mod instr;
mod opcode;
mod operand;

use std::{fmt, io};

use instr::{Instr, Mode};
use operand::Operand;

pub const MEM_LEN: usize = 2_usize.pow(16);

pub struct Cpu {
    pub pc: u16,
    pub sp: u8,
    pub flags: u8,

    pub a: u8,
    pub x: u8,
    pub y: u8,

    pub ram: Vec<u8>,
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

    pub fn run(mut self, start_addr: u16) -> Vec<u8> {
        self.pc = start_addr;
        loop {
            self.step();
        }
    }

    /// If the CPU would "halt" gracefully, this will return instead of looping.
    /// This can be useful for debugging.
    pub fn run_until_halt(mut self, start_addr: u16) -> Self {
        let mut enable_debugger = false;

        self.pc = start_addr;
        for i in 0.. {
            if self.would_halt() {
                eprintln!("would halt");
                enable_debugger = true;
            }

            // Detect long-running loops that aren't a simple "halt instruction".
            if i != 0 && i % 100_000_000 == 0 {
                eprintln!("after {}M instructions,", i / 1_000_000);
                eprintln!("{self:?}");
                eprintln!();
            }

            // // """breakpoint"""
            // if self.pc == 0x1bb1 {
            //     enable_debugger = true;
            // }

            // todo: hacky "single-step debugger", for testing
            if enable_debugger {
                eprintln!("{:?}", self);
                let (instr, mode) = opcode::decode(self.ram[self.pc as usize]);
                let instr_bytes = &self.ram[self.pc as usize..][..mode.instr_len() as usize];
                eprintln!("next instr: {:02x?} {:?} {:?}", instr_bytes, instr, mode);

                loop {
                    let line: String = io::stdin().lines().next().unwrap().unwrap();
                    let cmd = line.trim();
                    if cmd.is_empty() {
                        break;
                    }

                    if cmd.contains('.') {
                        eprintln!("not yet implemented: range of bytes");
                        continue;
                    }

                    let is_valid = cmd.chars().all(|c| c.is_digit(16)) && cmd.len() <= 4;
                    if !is_valid {
                        continue;
                    }

                    let padding = "0".repeat(4_usize.saturating_sub(cmd.len()));
                    let cmd = format!("{}{}", padding, cmd);
                    let addr = hex::decode(cmd).unwrap();
                    assert_eq!(addr.len(), 2);
                    let addr = u16::from_be_bytes([addr[0], addr[1]]);
                    eprintln!("ram[${:04x}]: ${:02x}", addr, self.get_byte(addr));
                }
            }

            // todo: hack: detect jumping to `start` label.
            let (_, mode) = opcode::decode(self.ram[self.pc as usize]);
            let instr_bytes = &self.ram[self.pc as usize..][..mode.instr_len() as usize];
            if instr_bytes == [0x4c, 0x00, 0x04] {
                eprintln!("would jump to start");
                break;
            }

            self.step();
        }
        self
    }

    /// Hack for testing: detect a "halt" instruction.
    fn would_halt(&self) -> bool {
        self.would_halt_jmp() || self.would_halt_branch()
    }

    fn would_halt_jmp(&self) -> bool {
        let [lo, hi] = self.pc.to_le_bytes();
        let jmp_absolute = 0x4c;
        let halt = [jmp_absolute, lo, hi];
        &self.ram[self.pc as usize..][..3] == &halt
    }

    fn would_halt_branch(&self) -> bool {
        let (instr, mode) = opcode::decode(self.ram[self.pc as usize]);
        let is_branch = mode == Mode::Relative;
        let in_place = self.get_byte(self.pc.checked_add(1).unwrap()) as i8 == -2;
        is_branch && in_place && would_branch(instr, self.flags)
    }

    fn step(&mut self) {
        let (instr, mode) = opcode::decode(self.ram[self.pc as usize]);

        let loc = Operand::from_mode(self, mode);

        let curr_pc = self.pc; // Addr of currently executing instr.
        self.pc = self.pc.checked_add(mode.instr_len()).unwrap();

        match instr {
            Instr::Brk => panic!("brk at 0x{:04x}", curr_pc),
            Instr::Nop => (),

            Instr::Tax => self.x = self.nz(self.a),
            Instr::Txa => self.a = self.nz(self.x),
            Instr::Tay => self.y = self.nz(self.a),
            Instr::Tya => self.a = self.nz(self.y),

            Instr::Txs => self.sp = self.x,
            Instr::Tsx => self.x = self.nz(self.sp),
            Instr::Pha => self.push(self.a),
            Instr::Pla => {
                let v = self.pop();
                self.a = self.nz(v);
            }
            Instr::Php => {
                let mut f = self.flags;
                flags::set(&mut f, flags::BREAK);
                flags::set(&mut f, flags::RESERVED);
                self.push(f);
            }
            Instr::Plp => {
                let v = self.pop();
                self.flags = self.nz(v);
            }

            Instr::Lda => self.a = self.nz(loc.get(self)),
            Instr::Ldx => self.x = self.nz(loc.get(self)),
            Instr::Ldy => self.y = self.nz(loc.get(self)),
            Instr::Sta => loc.set(self, self.a),
            Instr::Stx => loc.set(self, self.x),
            Instr::Sty => loc.set(self, self.y),

            Instr::Inx => self.x = self.nz(self.x.wrapping_add(1)),
            Instr::Dex => self.x = self.nz(self.x.wrapping_sub(1)),
            Instr::Iny => self.y = self.nz(self.y.wrapping_add(1)),
            Instr::Dey => self.y = self.nz(self.y.wrapping_sub(1)),
            Instr::Inc => {
                let v = self.nz(loc.get(self).wrapping_add(1));
                loc.set(self, v);
            }
            Instr::Dec => {
                let v = self.nz(loc.get(self).wrapping_sub(1));
                loc.set(self, v);
            }

            Instr::Clc => flags::clear(&mut self.flags, flags::CARRY),
            Instr::Sec => flags::set(&mut self.flags, flags::CARRY),
            Instr::Cli => flags::clear(&mut self.flags, flags::INTERRUPT),
            Instr::Sei => flags::set(&mut self.flags, flags::INTERRUPT),
            Instr::Clv => flags::clear(&mut self.flags, flags::OVERFLOW),
            Instr::Cld => flags::clear(&mut self.flags, flags::DECIMAL),
            Instr::Sed => flags::set(&mut self.flags, flags::DECIMAL),

            Instr::And => {
                self.a &= loc.get(self);
                self.nz(self.a);
            }
            Instr::Ora => {
                self.a |= loc.get(self);
                self.nz(self.a);
            }
            Instr::Eor => {
                self.a ^= loc.get(self);
                self.nz(self.a);
            }

            Instr::Adc => {
                let v = loc.get(self);
                self.a = self.adc(self.a, v);
            }
            Instr::Sbc => {
                let v = loc.get(self);
                self.a = self.sbc(self.a, v, true);
            }

            Instr::Cmp => {
                flags::set(&mut self.flags, flags::CARRY);
                let _ = self.sbc(self.a, loc.get(self), false);
            }
            Instr::Cpx => {
                flags::set(&mut self.flags, flags::CARRY);
                let _ = self.sbc(self.x, loc.get(self), false);
            }
            Instr::Cpy => {
                flags::set(&mut self.flags, flags::CARRY);
                let _ = self.sbc(self.y, loc.get(self), false);
            }

            Instr::Asl => {
                let (v, c) = loc.get(self).overflowing_shl(1);
                loc.set(self, v);
                self.nz(v);
                flags::set_to(&mut self.flags, flags::CARRY, c);
            }
            Instr::Lsr => {
                let (v, c) = loc.get(self).overflowing_shr(1);
                loc.set(self, v);
                self.nz(v);
                flags::set_to(&mut self.flags, flags::CARRY, c);
            }
            Instr::Rol => {
                let (mut v, c) = loc.get(self).overflowing_shl(1);
                v |= flags::is_set(self.flags, flags::CARRY) as u8;
                loc.set(self, v);
                self.nz(v);
                flags::set_to(&mut self.flags, flags::CARRY, c);
            }
            Instr::Ror => {
                let (mut v, c) = loc.get(self).overflowing_shr(1);
                if flags::is_set(self.flags, flags::CARRY) {
                    v |= 0x80;
                }
                loc.set(self, v);
                self.nz(v);
                flags::set_to(&mut self.flags, flags::CARRY, c);
            }

            Instr::Bit => {
                let v = loc.get(self);
                flags::set_to(&mut self.flags, flags::NEGATIVE, v & 0x80 != 0); // todo: introduce a bitset API?
                flags::set_to(&mut self.flags, flags::OVERFLOW, v & 0x40 != 0);
                flags::set_to(&mut self.flags, flags::ZERO, (v & self.a) == 0);
            }

            b @ (Instr::Bpl
            | Instr::Bmi
            | Instr::Bvc
            | Instr::Bvs
            | Instr::Bcc
            | Instr::Bcs
            | Instr::Bne
            | Instr::Beq) => {
                if would_branch(b, self.flags) {
                    self.pc = loc.addr();
                }
            }

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
        self.sp = self.sp.wrapping_add(1);
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

    #[must_use]
    fn adc(&mut self, a: u8, v: u8) -> u8 {
        let mut sum = 0u16;
        sum += a as u16;
        sum += v as u16;
        if flags::is_set(self.flags, flags::CARRY) {
            sum += 1;
        }

        let carry = sum >= 0x0100;
        let sum = sum as u8;

        let overflow = {
            let same_sign = a & 0x80 == v & 0x80;
            let flipped = sum & 0x80 != a & 0x80;
            same_sign && flipped
        };

        flags::set_to(&mut self.flags, flags::CARRY, carry);
        flags::set_to(&mut self.flags, flags::OVERFLOW, overflow);
        self.nz(sum)
    }

    // todo: this needs a major refactor. (Maybe we make it a function w/o side
    // effects, that returns a result alongside various flags?)
    #[must_use]
    fn sbc(&mut self, a: u8, v: u8, affects_overflow_flag: bool) -> u8 {
        let neg_v = (v as i8).wrapping_mul(-1) as u8;

        let mut sum: u16 = 0;
        sum += a as u16;
        sum += neg_v as u16;
        if !flags::is_set(self.flags, flags::CARRY) {
            sum = sum.wrapping_sub(1);
        }

        // todo: re-write this somehow so there's no need for the special case sum=0.
        let carry = sum >= 0x0100 || sum == 0;
        let sum = sum as u8;

        let overflow = {
            let same_sign = a & 0x80 == neg_v & 0x80;
            let flipped = sum & 0x80 != a & 0x80;
            same_sign && flipped
        };

        flags::set_to(&mut self.flags, flags::CARRY, carry);
        if affects_overflow_flag {
            flags::set_to(&mut self.flags, flags::OVERFLOW, overflow);
        }
        self.nz(sum)
    }
}

fn would_branch(branch: Instr, cpu_flags: u8) -> bool {
    let (flag, when) = match branch {
        Instr::Bpl => (flags::NEGATIVE, false),
        Instr::Bmi => (flags::NEGATIVE, true),
        Instr::Bvc => (flags::OVERFLOW, false),
        Instr::Bvs => (flags::OVERFLOW, true),
        Instr::Bcc => (flags::CARRY, false),
        Instr::Bcs => (flags::CARRY, true),
        Instr::Bne => (flags::ZERO, false),
        Instr::Beq => (flags::ZERO, true),
        _ => panic!("not a branch: {branch:?}"),
    };
    flags::is_set(cpu_flags, flag) == when
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "pc: ${:04x}", self.pc)?;
        writeln!(f, "sp: ${:02x}", self.sp)?;
        writeln!(f, "flags: {:08b}", self.flags)?;
        writeln!(f, "       NV-BDIZC")?;
        writeln!(f, "a: ${:02x}", self.a)?;
        writeln!(f, "x: ${:02x}", self.x)?;
        write!(f, "y: ${:02x}", self.y)?;
        Ok(())
    }
}

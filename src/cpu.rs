mod arith;
mod debugger;
mod flags;
mod instr;
mod opcode;
mod operand;

use std::fmt;

pub use debugger::Debugger;
use flags::{Flag, Flags};
use instr::{Instr, Mode};
use operand::Operand;

use crate::memory::Memory;

#[derive(Clone)]
pub struct Cpu {
    pc: u16,
    sp: u8,
    flags: Flags,

    a: u8,
    x: u8,
    y: u8,
}

impl Cpu {
    pub fn new(start_addr: u16) -> Self {
        Self {
            pc: start_addr,
            sp: u8::MAX,
            flags: Flags { bits: 0 },

            a: 0,
            x: 0,
            y: 0,
        }
    }

    pub fn step(&mut self, mem: &mut impl Memory) {
        let (instr, mode) = opcode::decode(mem.get(self.pc));
        let loc = Operand::new(self, mem, mode);

        let mut pc_set = false;
        match instr {
            Instr::Brk => panic!("brk at 0x{:04x}", self.pc),
            Instr::Nop => (),

            Instr::Tax => self.x = self.flags.nz(self.a),
            Instr::Txa => self.a = self.flags.nz(self.x),
            Instr::Tay => self.y = self.flags.nz(self.a),
            Instr::Tya => self.a = self.flags.nz(self.y),
            Instr::Txs => self.sp = self.x,
            Instr::Tsx => self.x = self.flags.nz(self.sp),

            Instr::Pha => self.push(mem, self.a),
            Instr::Pla => {
                let v = self.pop(mem);
                self.a = self.flags.nz(v);
            }
            Instr::Php => {
                let mut f = self.flags.clone();
                f.set(Flag::Break);
                f.set(Flag::Reserved);
                self.push(mem, f.bits);
            }
            Instr::Plp => self.flags.bits = self.pop(mem),

            Instr::Lda => self.a = self.flags.nz(loc.get(self, mem)),
            Instr::Ldx => self.x = self.flags.nz(loc.get(self, mem)),
            Instr::Ldy => self.y = self.flags.nz(loc.get(self, mem)),
            Instr::Sta => loc.set(self, mem, self.a),
            Instr::Stx => loc.set(self, mem, self.x),
            Instr::Sty => loc.set(self, mem, self.y),

            Instr::Inx => self.x = self.flags.nz(self.x.wrapping_add(1)),
            Instr::Dex => self.x = self.flags.nz(self.x.wrapping_sub(1)),
            Instr::Iny => self.y = self.flags.nz(self.y.wrapping_add(1)),
            Instr::Dey => self.y = self.flags.nz(self.y.wrapping_sub(1)),
            Instr::Inc => {
                let v = self.flags.nz(loc.get(self, mem).wrapping_add(1));
                loc.set(self, mem, v);
            }
            Instr::Dec => {
                let v = self.flags.nz(loc.get(self, mem).wrapping_sub(1));
                loc.set(self, mem, v);
            }

            Instr::Clc => self.flags.clear(Flag::Carry),
            Instr::Sec => self.flags.set(Flag::Carry),
            Instr::Cli => self.flags.clear(Flag::Interrupt),
            Instr::Sei => self.flags.set(Flag::Interrupt),
            Instr::Clv => self.flags.clear(Flag::Overflow),
            Instr::Cld => self.flags.clear(Flag::Decimal),
            Instr::Sed => self.flags.set(Flag::Decimal),

            Instr::And => self.a = self.flags.nz(self.a & loc.get(self, mem)),
            Instr::Ora => self.a = self.flags.nz(self.a | loc.get(self, mem)),
            Instr::Eor => self.a = self.flags.nz(self.a ^ loc.get(self, mem)),

            Instr::Adc => self.adc(self.a, loc.get(self, mem)),
            Instr::Sbc => self.adc(self.a, !loc.get(self, mem)),
            Instr::Cmp => self.cmp(self.a, loc.get(self, mem)),
            Instr::Cpx => self.cmp(self.x, loc.get(self, mem)),
            Instr::Cpy => self.cmp(self.y, loc.get(self, mem)),

            Instr::Asl => {
                self.flags.clear(Flag::Carry);
                let v = self.rol(loc.get(self, mem));
                loc.set(self, mem, v);
            }
            Instr::Lsr => {
                self.flags.clear(Flag::Carry);
                let v = self.ror(loc.get(self, mem));
                loc.set(self, mem, v);
            }
            Instr::Rol => {
                let v = self.rol(loc.get(self, mem));
                loc.set(self, mem, v);
            }
            Instr::Ror => {
                let v = self.ror(loc.get(self, mem));
                loc.set(self, mem, v);
            }

            Instr::Bit => {
                let v = loc.get(self, mem);
                self.flags.assign(Flag::Negative, v & 0x80 != 0);
                self.flags.assign(Flag::Overflow, v & 0x40 != 0);
                self.flags.assign(Flag::Zero, (v & self.a) == 0);
            }

            b @ (Instr::Bpl
            | Instr::Bmi
            | Instr::Bvc
            | Instr::Bvs
            | Instr::Bcc
            | Instr::Bcs
            | Instr::Bne
            | Instr::Beq) => {
                if self.would_branch(b) {
                    self.pc = loc.addr();
                    pc_set = true;
                }
            }

            Instr::Jmp => {
                self.pc = loc.addr();
                pc_set = true;
            }
            Instr::Jsr => {
                let return_addr_minus_one = self.pc.checked_add(2).unwrap();
                self.push2(mem, return_addr_minus_one);
                self.pc = loc.addr();
                pc_set = true;
            }
            Instr::Rts => {
                self.pc = self.pop2(mem).checked_add(1).unwrap();
                pc_set = true;
            }
            Instr::Rti => {
                self.flags.bits = self.pop(mem);
                // Note that unlike RTS, there is no off-by-one here.
                self.pc = self.pop2(mem);
                pc_set = true;
            }
        }

        if !pc_set {
            self.pc = self.pc.checked_add(mode.instr_len()).unwrap();
        }
    }

    fn adc(&mut self, arg1: u8, arg2: u8) {
        let ret = arith::add(arg1, arg2, self.flags.is_set(Flag::Carry));
        self.a = self.flags.nz(ret.sum);
        self.flags.assign(Flag::Carry, ret.carry);
        self.flags.assign(Flag::Overflow, ret.overflow);
    }

    fn cmp(&mut self, arg1: u8, arg2: u8) {
        let ret = arith::add(arg1, !arg2, true);
        self.flags.nz(ret.sum);
        self.flags.assign(Flag::Carry, ret.carry);
    }

    fn rol(&mut self, arg: u8) -> u8 {
        let (mut out, c) = arith::overflowing_shl(arg, 1);
        if self.flags.is_set(Flag::Carry) {
            out |= 1;
        }
        self.flags.nz(out);
        self.flags.assign(Flag::Carry, c);
        out
    }

    fn ror(&mut self, arg: u8) -> u8 {
        let (mut out, c) = arith::overflowing_shr(arg, 1);
        if self.flags.is_set(Flag::Carry) {
            out |= 0x80;
        }
        self.flags.nz(out);
        self.flags.assign(Flag::Carry, c);
        out
    }

    fn would_branch(&self, branch: Instr) -> bool {
        let (flag, when) = match branch {
            Instr::Bpl => (Flag::Negative, false),
            Instr::Bmi => (Flag::Negative, true),
            Instr::Bvc => (Flag::Overflow, false),
            Instr::Bvs => (Flag::Overflow, true),
            Instr::Bcc => (Flag::Carry, false),
            Instr::Bcs => (Flag::Carry, true),
            Instr::Bne => (Flag::Zero, false),
            Instr::Beq => (Flag::Zero, true),
            _ => panic!("not a branch: {branch:?}"),
        };
        self.flags.is_set(flag) == when
    }
}

/// Stack operations.
impl Cpu {
    fn push(&mut self, mem: &mut impl Memory, value: u8) {
        mem.set(0x0100 + self.sp as u16, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pop(&mut self, mem: &mut impl Memory) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        mem.get(0x0100 + self.sp as u16)
    }

    fn push2(&mut self, mem: &mut impl Memory, word: u16) {
        let [lo, hi] = u16::to_le_bytes(word);

        // The stack grows down, so this stores the bytes in little-endian
        // order in RAM.
        self.push(mem, hi);
        self.push(mem, lo);
    }

    fn pop2(&mut self, mem: &mut impl Memory) -> u16 {
        let lo = self.pop(mem);
        let hi = self.pop(mem);
        u16::from_le_bytes([lo, hi])
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "pc: ${:04x}", self.pc)?;
        writeln!(f, "sp: ${:02x}", self.sp)?;
        writeln!(f, "flags: {:08b}", self.flags.bits)?;
        writeln!(f, "       NV-BDIZC")?;
        writeln!(f, "a: ${:02x}", self.a)?;
        writeln!(f, "x: ${:02x}", self.x)?;
        write!(f, "y: ${:02x}", self.y)?;
        Ok(())
    }
}

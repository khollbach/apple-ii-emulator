mod debugging;
mod flags;
mod instr;
mod opcode;
mod operand;

use flags::{Flag, Flags};
use instr::{Instr, Mode};
use operand::Operand;

pub const MEM_LEN: usize = 2_usize.pow(16);

pub struct Cpu {
    pub pc: u16,
    pub sp: u8,
    pub flags: Flags,

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
            flags: Flags { bits: 0 },
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

    fn step(&mut self) {
        let (instr, mode) = opcode::decode(self.get_byte(self.pc));

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
                let mut f = self.flags.clone();
                f.set(Flag::Break);
                f.set(Flag::Reserved);
                self.push(f.bits);
            }
            Instr::Plp => self.flags.bits = self.pop(),

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

            Instr::Clc => self.flags.clear(Flag::Carry),
            Instr::Sec => self.flags.set(Flag::Carry),
            Instr::Cli => self.flags.clear(Flag::Interrupt),
            Instr::Sei => self.flags.set(Flag::Interrupt),
            Instr::Clv => self.flags.clear(Flag::Overflow),
            Instr::Cld => self.flags.clear(Flag::Decimal),
            Instr::Sed => self.flags.set(Flag::Decimal),

            Instr::And => self.a = self.nz(self.a & loc.get(self)),
            Instr::Ora => self.a = self.nz(self.a | loc.get(self)),
            Instr::Eor => self.a = self.nz(self.a ^ loc.get(self)),

            Instr::Adc => self.adc(self.a, loc.get(self)),
            Instr::Sbc => self.adc(self.a, !loc.get(self)),
            Instr::Cmp => self.cmp(self.a, loc.get(self)),
            Instr::Cpx => self.cmp(self.x, loc.get(self)),
            Instr::Cpy => self.cmp(self.y, loc.get(self)),

            Instr::Asl => {
                let (v, c) = overflowing_shl(loc.get(self), 1);
                loc.set(self, v);
                self.nz(v);
                self.flags.assign(Flag::Carry, c);
            }
            Instr::Lsr => {
                let (v, c) = overflowing_shr(loc.get(self), 1);
                loc.set(self, v);
                self.nz(v);
                self.flags.assign(Flag::Carry, c);
            }
            Instr::Rol => {
                let (mut v, c) = overflowing_shl(loc.get(self), 1);
                if self.flags.is_set(Flag::Carry) {
                    v |= 1;
                }
                loc.set(self, v);
                self.nz(v);
                self.flags.assign(Flag::Carry, c);
            }
            Instr::Ror => {
                let (mut v, c) = overflowing_shr(loc.get(self), 1);
                if self.flags.is_set(Flag::Carry) {
                    v |= 0x80;
                }
                loc.set(self, v);
                self.nz(v);
                self.flags.assign(Flag::Carry, c);
            }

            Instr::Bit => {
                let v = loc.get(self);
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
                self.flags.bits = self.pop();
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
        self.set_byte(addr, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    /// Pop from the stack.
    fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let addr = 0x0100 + self.sp as u16;
        self.get_byte(addr)
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
        self.flags.assign(Flag::Zero, value == 0);
        self.flags.assign(Flag::Negative, (value as i8) < 0);
        value
    }

    fn adc(&mut self, arg1: u8, arg2: u8) {
        let ret = addition(arg1, arg2, self.flags.is_set(Flag::Carry));
        self.a = self.nz(ret.sum);
        self.flags.assign(Flag::Carry, ret.carry);
        self.flags.assign(Flag::Overflow, ret.overflow);
    }

    fn cmp(&mut self, arg1: u8, arg2: u8) {
        let ret = addition(arg1, !arg2, true);
        self.nz(ret.sum);
        self.flags.assign(Flag::Carry, ret.carry);
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

/// Perform addition, detecting signed integer overflows.
fn addition(x: u8, y: u8, carry_in: bool) -> Addition {
    let (sum, carry_out) = add_with_carry(x, y, carry_in);

    let overflow = {
        let same_sign = x & 0x80 == y & 0x80;
        let flipped = sum & 0x80 != x & 0x80;
        same_sign && flipped
    };

    Addition {
        sum,
        carry: carry_out,
        overflow,
    }
}

struct Addition {
    sum: u8,
    carry: bool,
    overflow: bool,
}

fn add_with_carry(x: u8, y: u8, carry_in: bool) -> (u8, bool) {
    let (s1, c1) = x.overflowing_add(y);
    if !carry_in {
        return (s1, c1);
    }

    let (s2, c2) = s1.overflowing_add(1);
    debug_assert!(!(c1 && c2));
    (s2, c1 || c2)
}

/// The standard library `overflowing_shl` behaviour isn't what I expected, so
/// we write our own that does what we want.
///
/// The (weird) behaviour of std::u8::overflowing_shl is:
/// ```
/// assert_eq!(0x80_u8.overflowing_shl(1), (0, false)); // ???
/// assert_eq!(0x55_u8.overflowing_shl(0x81), (0xaa, true)); // I mean, sure?
/// ```
fn overflowing_shl(x: u8, shift_amount: u8) -> (u8, bool) {
    assert!(shift_amount < 8);
    let out = x << shift_amount;
    let overflow = out.count_ones() != x.count_ones();
    (out, overflow)
}

/// Returns true if any of the bits got shifted off the end.
///
/// See also `overflowing_shl`.
fn overflowing_shr(x: u8, shift_amount: u8) -> (u8, bool) {
    assert!(shift_amount < 8);
    let out = x >> shift_amount;
    let overflow = out.count_ones() != x.count_ones();
    (out, overflow)
}

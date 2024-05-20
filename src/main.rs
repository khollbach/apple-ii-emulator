mod flags;
mod opcode;

const MEM_LEN: usize = 2_usize.pow(16);

fn main() {
    let mut prog = vec![0; MEM_LEN];
    prog[0] = 0xea; // nop
    prog[1] = 0x00; // brk
    Cpu::new(prog).run();
}

struct Cpu {
    pc: u16,
    sp: u8,
    flags: u8,

    a: u8,
    x: u8,
    y: u8,

    ram: Vec<u8>,
}

impl Cpu {
    fn new(ram: Vec<u8>) -> Self {
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

    fn run(mut self) {
        loop {
            self.step();
        }
    }

    fn step(&mut self) {
        let (opcode, mode) = opcode::decode(self.ram[self.pc as usize]);

        let mode_argument = match mode.instr_len() {
            1 => 0u16,
            2 => self.byte(self.pc.checked_add(1).unwrap()) as u16,
            3 => self.word(self.pc.checked_add(1).unwrap()),
            _ => unreachable!(),
        };

        let operand16 = match mode {
            Mode::Accumulator => self.a as u16,
            Mode::Implied => 0,

            Mode::Immediate => self.word(self.pc.checked_add(1).unwrap()),
            Mode::Absolute => {
                let addr = self.word(self.pc.checked_add(1).unwrap());
                self.word(addr)
            }
            Mode::Indirect => {
                let pointer = self.word(self.pc.checked_add(1).unwrap());
                let addr = self.word(pointer);
                self.word(addr)
            }

            Mode::AbsoluteX => todo!(),
            Mode::AbsoluteY => todo!(),

            Mode::XIndirect => todo!(),
            Mode::IndirectY => todo!(),

            Mode::Relative => 0, // ?

            // todo: check the details here; and maybe clean it up
            Mode::ZeroPage => {
                let lo = self.ram[self.pc.checked_add(1).unwrap() as usize];
                let hi = 0;
                let addr = u16::from_le_bytes([lo, hi]);
                self.word(addr)
            }

            Mode::ZeroPageX => todo!(),
            Mode::ZeroPageY => todo!(),
        };
        let operand8 = operand16 as u8;

        let curr_pc = self.pc; // Addr of currently executing instr.
        self.pc = self.pc.checked_add(mode.instr_len().into()).unwrap();

        match opcode {
            Instr::Brk => panic!("brk at {:#04x}", curr_pc),
            Instr::Nop => (),

            // todo: update flags accordingly, for all operations

            Instr::Tax => self.x = self.a,
            Instr::Txa => self.a = self.x,
            Instr::Tay => self.y = self.a,
            Instr::Tya => self.a = self.y,
            Instr::Dex => self.x = self.x.wrapping_sub(1),
            Instr::Inx => self.x = self.x.wrapping_add(1),
            Instr::Dey => self.y = self.y.wrapping_sub(1),
            Instr::Iny => self.y = self.y.wrapping_add(1),

            Instr::Txs => self.sp = self.x,
            Instr::Tsx => self.x = self.sp,
            Instr::Pha => self.push(self.a),
            Instr::Pla => self.a = self.pop(),
            Instr::Php => self.push(self.flags),
            Instr::Plp => self.flags = self.pop(),

            Instr::Clc => flags::clear(&mut self.flags, flags::CARRY),
            Instr::Sec => flags::set(&mut self.flags, flags::CARRY),
            Instr::Cli => flags::clear(&mut self.flags, flags::INTERRUPT),
            Instr::Sei => flags::set(&mut self.flags, flags::INTERRUPT),
            Instr::Clv => flags::clear(&mut self.flags, flags::OVERFLOW),
            Instr::Cld => flags::clear(&mut self.flags, flags::DECIMAL),
            Instr::Sed => flags::set(&mut self.flags, flags::DECIMAL),

            Instr::Adc => self.a = self.a.wrapping_add(operand8),
            Instr::And => self.a &= operand8,
            Instr::Sbc => self.a = self.a.wrapping_sub(operand8),
            Instr::Ora => self.a |= operand8,
            Instr::Eor => self.a ^= operand8,

            Instr::Lda => self.a = operand8,
            Instr::Ldx => self.x = operand8,
            Instr::Ldy => self.y = operand8,

            Instr::Sta => todo!(),
            Instr::Stx => todo!(),
            Instr::Sty => todo!(),

            Instr::Rol => todo!(),
            Instr::Ror => todo!(),
            Instr::Asl => todo!(),
            Instr::Lsr => todo!(),

            Instr::Inc => todo!(),
            Instr::Dec => todo!(),
            Instr::Bit => todo!(),

            Instr::Cmp => todo!(),
            Instr::Cpx => todo!(),
            Instr::Cpy => todo!(),

            Instr::Bpl => self.branch(flags::NEGATIVE, false),
            Instr::Bmi => self.branch(flags::NEGATIVE, true),
            Instr::Bvc => self.branch(flags::OVERFLOW, false),
            Instr::Bvs => self.branch(flags::OVERFLOW, true),
            Instr::Bcc => self.branch(flags::CARRY, false),
            Instr::Bcs => self.branch(flags::CARRY, true),
            Instr::Bne => self.branch(flags::ZERO, false),
            Instr::Beq => self.branch(flags::ZERO, true),

            Instr::Jmp => {
                let addr = self.word(curr_pc.checked_add(1).unwrap());
                match mode {
                    Mode::Absolute => self.pc = addr,
                    Mode::Indirect => self.pc = self.word(addr),
                    _ => unreachable!(),
                }
            }

            Instr::Jsr => {
                let addr = self.word(curr_pc.checked_add(1).unwrap());
                self.pc = addr;

                let return_addr_minus_one = curr_pc.checked_add(2).unwrap();
                self.push2(return_addr_minus_one);
            }

            Instr::Rts => self.pc = self.pop2().checked_add(1).unwrap(),

            Instr::Rti => {
                self.flags = self.pop();

                // Note that unlike RTS, there is no off-by-one here.
                self.pc = self.pop2();
            }
        }
    }

    fn byte(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    fn word(&self, addr: u16) -> u16 {
        let lo = self.byte(addr);
        let hi = self.byte(addr.checked_add(1).unwrap());
        let word = u16::from_le_bytes([lo, hi]);
        word
    }

    fn push(&mut self, value: u8) {
        let addr = 0x0100 + self.sp as u16;
        self.ram[addr as usize] = value;
        self.sp = self.sp.wrapping_sub(1);
    }

    fn pop(&mut self) -> u8 {
        self.sp = self.sp.wrapping_sub(1);
        let addr = 0x0100 + self.sp as u16;
        self.ram[addr as usize]
    }

    fn push2(&mut self, value: u16) {
        let [lo, hi] = u16::to_le_bytes(value);

        // The stack grows down, so this stores the bytes in little-endian
        // order in RAM.
        self.push(hi);
        self.push(lo);
    }

    fn pop2(&mut self) -> u16 {
        let lo = self.pop();
        let hi = self.pop();
        u16::from_le_bytes([lo, hi])
    }

    fn branch(&mut self, flag: u8, branch_if: bool) {
        let is_set = flags::is_set(self.flags, flag);
        if is_set == branch_if {
            let value: u8 = self.ram[self.pc.checked_add(1).expect("overflow") as usize];
            let offset = value as i8 as i16;
            self.pc = self.pc.wrapping_add(offset as u16);
        }
    }
}

impl Mode {
    fn instr_len(self) -> u8 {
        match self {
            Mode::Implied => 1,
            Mode::Accumulator => 1,

            Mode::Relative => 2,
            Mode::ZeroPage => 2,
            Mode::ZeroPageX => 2,
            Mode::ZeroPageY => 2,

            Mode::Immediate => 3,
            Mode::Absolute => 3,
            Mode::AbsoluteX => 3,
            Mode::AbsoluteY => 3,
            Mode::Indirect => 3,
            Mode::XIndirect => 3,
            Mode::IndirectY => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Accumulator,

    Absolute,
    AbsoluteX,
    AbsoluteY,

    Immediate,

    Implied,

    Indirect,
    XIndirect,
    IndirectY,

    Relative,

    ZeroPage,
    ZeroPageX,
    ZeroPageY,
}

enum Instr {
    Adc,
    And,
    Asl,
    Bcc,
    Bcs,
    Beq,
    Bit,
    Bmi,
    Bne,
    Bpl,
    Brk,
    Bvc,
    Bvs,
    Clc,
    Cld,
    Cli,
    Clv,
    Cmp,
    Cpx,
    Cpy,
    Dec,
    Dex,
    Dey,
    Eor,
    Inc,
    Inx,
    Iny,
    Jmp,
    Jsr,
    Lda,
    Ldx,
    Ldy,
    Lsr,
    Nop,
    Ora,
    Pha,
    Php,
    Pla,
    Plp,
    Rol,
    Ror,
    Rti,
    Rts,
    Sbc,
    Sec,
    Sed,
    Sei,
    Sta,
    Stx,
    Sty,
    Tax,
    Tay,
    Tsx,
    Txa,
    Txs,
    Tya,
}

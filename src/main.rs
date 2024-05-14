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

        // match mode {
        //     Mode::Accumulator => (),
        //     Mode::Absolute => (),
        //     Mode::AbsoluteX => (),
        //     Mode::AbsoluteY => (),
        //     Mode::Immediate => (),
        //     Mode::Implied => (),
        //     Mode::Indirect => (),
        //     Mode::XIndirect => (),
        //     Mode::IndirectY => (),
        //     Mode::Relative => (),
        //     Mode::ZeroPage => (),
        //     Mode::ZeroPageX => (),
        //     Mode::ZeroPageY => (),
        // }

        match opcode {
            Instr::Brk => panic!("brk at {:#04x}", self.pc),
            Instr::Nop => (),

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

            Instr::Bpl => self.branch(flags::NEGATIVE, false),
            Instr::Bmi => self.branch(flags::NEGATIVE, true),
            Instr::Bvc => self.branch(flags::OVERFLOW, false),
            Instr::Bvs => self.branch(flags::OVERFLOW, true),
            Instr::Bcc => self.branch(flags::CARRY, false),
            Instr::Bcs => self.branch(flags::CARRY, true),
            Instr::Bne => self.branch(flags::ZERO, false),
            Instr::Beq => self.branch(flags::ZERO, true),

            Instr::Jmp => {
                let addr = self.word(self.pc.checked_add(1).unwrap());
                match mode {
                    Mode::Absolute => self.pc = addr,
                    Mode::Indirect => self.pc = self.word(addr),
                    _ => unreachable!(),
                }
            }

            _ => todo!(),
        }

        // todo: adjust the right amount, based on the instr
        // match mode {
        //     Mode::Accumulator => (),
        //     Mode::Absolute => (),
        //     Mode::AbsoluteX => (),
        //     Mode::AbsoluteY => (),
        //     Mode::Immediate => (),
        //     Mode::Implied => (),
        //     Mode::Indirect => (),
        //     Mode::XIndirect => (),
        //     Mode::IndirectY => (),
        //     Mode::Relative => (),
        //     Mode::ZeroPage => (),
        //     Mode::ZeroPageX => (),
        //     Mode::ZeroPageY => (),
        // }
        if matches!(mode, Mode::Relative) {
            self.pc += 2;
        } else {
            self.pc += 1;
        }
    }

    fn word(&self, addr: u16) -> u16 {
        let lo = self.ram[addr as usize];
        let hi = self.ram[addr.checked_add(1).unwrap() as usize];
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

    fn branch(&mut self, flag: u8, branch_if: bool) {
        let is_set = flags::is_set(self.flags, flag);
        if is_set == branch_if {
            let value: u8 = self.ram[self.pc.checked_add(1).expect("overflow") as usize];
            let offset = value as i8 as i16;
            self.pc = self.pc.wrapping_add(offset as u16);
        }
    }
}

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

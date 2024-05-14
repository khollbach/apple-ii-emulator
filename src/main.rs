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
        match opcode {
            Instr::Brk => panic!("brk at {:#04x}", self.pc),
            Instr::Nop => (),

            Instr::Tax => self.x = self.a,
            Instr::Txa => self.a = self.x,
            Instr::Dex => self.x = self.x.wrapping_sub(1),
            Instr::Inx => self.x = self.x.wrapping_add(1),
            Instr::Tay => self.y = self.a,
            Instr::Tya => self.a = self.y,
            Instr::Dey => self.y = self.y.wrapping_sub(1),
            Instr::Iny => self.y = self.y.wrapping_add(1),

            Instr::Txs => self.sp = self.x,
            Instr::Tsx => self.x = self.sp,
            Instr::Pha => {
                let addr = 0x0100 + self.sp as u16;
                self.ram[addr as usize] = self.a;
                self.sp = self.sp.wrapping_sub(1);
            }
            Instr::Pla => {
                self.sp = self.sp.wrapping_sub(1);
                let addr = 0x0100 + self.sp as u16;
                self.a = self.ram[addr as usize];
            }
            Instr::Php => {
                let addr = 0x0100 + self.sp as u16;
                self.ram[addr as usize] = self.flags;
                self.sp = self.sp.wrapping_sub(1);
            }
            Instr::Plp => {
                self.sp = self.sp.wrapping_sub(1);
                let addr = 0x0100 + self.sp as u16;
                self.flags = self.ram[addr as usize];
            }

            Instr::Clc => self.flags &= !flags::CARRY,
            Instr::Sec => self.flags |= flags::CARRY,
            Instr::Cli => self.flags &= !flags::INTERRUPT,
            Instr::Sei => self.flags |= flags::INTERRUPT,
            Instr::Clv => self.flags &= !flags::OVERFLOW,
            Instr::Cld => self.flags &= !flags::DECIMAL,
            Instr::Sed => self.flags |= flags::DECIMAL,

            // todo: how to handle operands...
            // ...maybe we lump this in with "immediate" mode?
            Instr::Bpl => {
                if self.flags & flags::NEGATIVE == 0 {
                    let offset: i8 = 0; // todo
                                        // todo: confirm we're allowed to wrap below $0000, and above $ffff
                    self.pc = (self.pc as i16).wrapping_add(offset as i16) as u16;
                }
            }
            Instr::Bmi => todo!(),
            Instr::Bvc => todo!(),
            Instr::Bvs => todo!(),
            Instr::Bcc => todo!(),
            Instr::Bcs => todo!(),
            Instr::Bne => todo!(),
            Instr::Beq => todo!(),

            _ => todo!(),
        }

        // todo: adjust the right amount, based on the instr
        self.pc += 1;
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

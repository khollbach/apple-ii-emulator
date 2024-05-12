const MEM_LEN: usize = 2_usize.pow(16);

mod flags {
    pub const CARRY: u8 = 1 << 0;
    pub const ZERO: u8 = 1 << 1;
    pub const INTERRUPT: u8 = 1 << 2;
    pub const DECIMAL: u8 = 1 << 3;

    pub const BREAK: u8 = 1 << 4;

    pub const OVERFLOW: u8 = 1 << 6;
    pub const NEGATIVE: u8 = 1 << 7;
}

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
        match instruction(opcode) {
            Opcode::Brk => panic!("brk at {:#04x}", self.pc),
            Opcode::Nop => (),

            Opcode::Tax => self.x = self.a,
            Opcode::Txa => self.a = self.x,
            Opcode::Dex => self.x = self.x.wrapping_sub(1),
            Opcode::Inx => self.x = self.x.wrapping_add(1),
            Opcode::Tay => self.y = self.a,
            Opcode::Tya => self.a = self.y,
            Opcode::Dey => self.y = self.y.wrapping_sub(1),
            Opcode::Iny => self.y = self.y.wrapping_add(1),

            Opcode::Txs => self.sp = self.x,
            Opcode::Tsx => self.x = self.sp,
            Opcode::Pha => {
                let addr = 0x0100 + self.sp as u16;
                self.ram[addr as usize] = self.a;
                self.sp = self.sp.wrapping_sub(1);
            }
            Opcode::Pla => {
                self.sp = self.sp.wrapping_sub(1);
                let addr = 0x0100 + self.sp as u16;
                self.a = self.ram[addr as usize];
            }
            Opcode::Php => {
                let addr = 0x0100 + self.sp as u16;
                self.ram[addr as usize] = self.flags;
                self.sp = self.sp.wrapping_sub(1);
            }
            Opcode::Plp => {
                self.sp = self.sp.wrapping_sub(1);
                let addr = 0x0100 + self.sp as u16;
                self.flags = self.ram[addr as usize];
            }

            Opcode::Clc => self.flags &= !flags::CARRY,
            Opcode::Sec => self.flags |= flags::CARRY,
            Opcode::Cli => self.flags &= !flags::INTERRUPT,
            Opcode::Sei => self.flags |= flags::INTERRUPT,
            Opcode::Clv => self.flags &= !flags::OVERFLOW,
            Opcode::Cld => self.flags &= !flags::DECIMAL,
            Opcode::Sed => self.flags |= flags::DECIMAL,

            // todo: how to handle operands...
            // ...maybe we lump this in with "immediate" mode?
            Opcode::Bpl => {
                if self.flags & flags::NEGATIVE == 0 {
                    let offset: i8 = 0; // todo
                                        // todo: confirm we're allowed to wrap below $0000, and above $ffff
                    self.pc = (self.pc as i16).wrapping_add(offset as i16) as u16;
                }
            }
            Opcode::Bmi => todo!(),
            Opcode::Bvc => todo!(),
            Opcode::Bvs => todo!(),
            Opcode::Bcc => todo!(),
            Opcode::Bcs => todo!(),
            Opcode::Bne => todo!(),
            Opcode::Beq => todo!(),
        }

        // todo: adjust the right amount, based on the instr
        self.pc += 1;
    }

    fn decode(&mut self) -> (Opcode, Mode) {
        match self.ram[self.pc as usize] {
            0x00 => (Opcode::Brk, Mode::Implied),
            0xea => (Opcode::Nop, Mode::Implied),

            0xaa => (Opcode::Tax, Mode::Implied),
            0x8a => (Opcode::Txa, Mode::Implied),
            0xca => (Opcode::Dex, Mode::Implied),
            0xe8 => (Opcode::Inx, Mode::Implied),
            0xa8 => (Opcode::Tay, Mode::Implied),
            0x98 => (Opcode::Tya, Mode::Implied),
            0x88 => (Opcode::Dey, Mode::Implied),
            0xc8 => (Opcode::Iny, Mode::Implied),

            0x9a => (Opcode::Txs, Mode::Implied),
            0xba => (Opcode::Tsx, Mode::Implied),
            0x48 => (Opcode::Pha, Mode::Implied),
            0x68 => (Opcode::Pla, Mode::Implied),
            0x08 => (Opcode::Php, Mode::Implied),
            0x28 => (Opcode::Plp, Mode::Implied),

            0x18 => (Opcode::Clc, Mode::Implied),
            0x38 => (Opcode::Sec, Mode::Implied),
            0x58 => (Opcode::Cli, Mode::Implied),
            0x78 => (Opcode::Sei, Mode::Implied),
            0xb8 => (Opcode::Clv, Mode::Implied),
            0xd8 => (Opcode::Cld, Mode::Implied),
            0xf8 => (Opcode::Sed, Mode::Implied),

            0x10 => (Opcode::Bpl, Mode::Immediate),
            0x30 => (Opcode::Bmi, Mode::Immediate),
            0x50 => (Opcode::Bvc, Mode::Immediate),
            0x70 => (Opcode::Bvs, Mode::Immediate),
            0x90 => (Opcode::Bcc, Mode::Immediate),
            0xb0 => (Opcode::Bcs, Mode::Immediate),
            0xd0 => (Opcode::Bne, Mode::Immediate),
            0xf0 => (Opcode::Beq, Mode::Immediate),

            // TODO: left off here 2024-05-12
            // Ok, let's take a quick step back. Now that I understand "decoding"
            // a little better, it feels like it'd be good to automate this process,
            // so I'm not just hand-typing out ~256 different rows of information
            // into a Rust `match` statement.
            //
            // Maybe we can get a table of the relevant decoding information, and
            // write a script to process it and output the Rust code that I want?
            //
            // Or perhaps it would be simpler to read the table and parse it at
            // runtime? But this may add some complexity to the API, because of
            // the possibility of errors. Perhaps there'a middle-ground of running
            // the parsing code at compile-time (would this have to be a proc macro?
            // something else?).
            //
            // I think the compile-time generated match statement appeals to me
            // aesthetically, and I'll learn some new stuff if I do it. So let's
            // go with that.

            // 0x69 => (Opcode::Adc)

// Immediate     ADC #$44      $69  2   2
// Zero Page     ADC $44       $65  2   3
// Zero Page,X   ADC $44,X     $75  2   4
// Absolute      ADC $4400     $6D  3   4
// Absolute,X    ADC $4400,X   $7D  3   4+
// Absolute,Y    ADC $4400,Y   $79  3   4+
// Indirect,X    ADC ($44,X)   $61  2   6
// Indirect,Y    ADC ($44),Y   $71  2   5+

            _ => unimplemented!(),
        }
    }
}

enum Mode {
    Absolute,
    AbsoluteX,
    AbsoluteY,

    Immediate,

    // Includes "accumulator" addressing mode.
    Implied,

    Indirect,
    XIndirect,
    IndirectY,

    ZeroPage,
    ZeroPageX,
    ZeroPageY,
}

enum Opcode {
    Brk,
    Nop,

    //
    // Register
    //
    Tax,
    Txa,
    Dex,
    Inx,
    Tay,
    Tya,
    Dey,
    Iny,

    //
    // Stack
    //
    Txs,
    Tsx,
    Pha,
    Pla,
    Php,
    Plp,

    //
    // Flag
    //
    Clc,
    Sec,
    Cli,
    Sei,
    Clv,
    Cld,
    Sed,

    //
    // Branch
    //
    Bpl,
    Bmi,
    Bvc,
    Bvs,
    Bcc,
    Bcs,
    Bne,
    Beq,

    Adc,
}

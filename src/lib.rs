mod flags;
mod instr;
mod opcode;
mod operand;

use instr::{Instr, Mode};

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
        let (opcode, mode) = opcode::decode(self.ram[self.pc as usize]);

        let operand = self.operand(mode);

        let curr_pc = self.pc; // Addr of currently executing instr.
        self.pc = self.pc.checked_add(mode.instr_len()).unwrap();

        match opcode {
            Instr::Brk => panic!("brk at {:#04x}", curr_pc),
            Instr::Nop => (),

            //
            // todo: update flags accordingly, for all operations
            //
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

            Instr::Adc => self.a = self.a.wrapping_add(self.get_operand_value(operand)),
            Instr::And => self.a &= self.get_operand_value(operand),
            Instr::Sbc => self.a = self.a.wrapping_sub(self.get_operand_value(operand)),
            Instr::Ora => self.a |= self.get_operand_value(operand),
            Instr::Eor => self.a ^= self.get_operand_value(operand),

            Instr::Lda => self.a = self.get_operand_value(operand),
            Instr::Ldx => self.x = self.get_operand_value(operand),
            Instr::Ldy => self.y = self.get_operand_value(operand),

            Instr::Sta => self.set_operand_value(operand, self.a),
            Instr::Stx => self.set_operand_value(operand, self.x),
            Instr::Sty => self.set_operand_value(operand, self.y),

            Instr::Asl => {
                let value = self.get_operand_value(operand);
                self.set_operand_value(operand, value << 1);
            }
            Instr::Lsr => {
                let value = self.get_operand_value(operand);
                self.set_operand_value(operand, value >> 1);
            }
            Instr::Rol => {
                let mut value = self.get_operand_value(operand);
                let new_bit = flags::is_set(self.flags, flags::CARRY) as u8;
                value <<= 1;
                value |= new_bit;
                self.set_operand_value(operand, value);
            }
            Instr::Ror => {
                let mut value = self.get_operand_value(operand);
                let new_bit = flags::is_set(self.flags, flags::CARRY) as u8;
                value >>= 1;
                value |= new_bit;
                self.set_operand_value(operand, value);
            }

            Instr::Inc => {
                let value = self.get_operand_value(operand);
                self.set_operand_value(operand, value.wrapping_add(1));
            }
            Instr::Dec => {
                let value = self.get_operand_value(operand);
                self.set_operand_value(operand, value.wrapping_sub(1));
            }
            Instr::Bit => todo!(),

            Instr::Cmp => todo!(),
            Instr::Cpx => todo!(),
            Instr::Cpy => todo!(),

            Instr::Bpl => self.branch(operand.addr(), flags::NEGATIVE, false),
            Instr::Bmi => self.branch(operand.addr(), flags::NEGATIVE, true),
            Instr::Bvc => self.branch(operand.addr(), flags::OVERFLOW, false),
            Instr::Bvs => self.branch(operand.addr(), flags::OVERFLOW, true),
            Instr::Bcc => self.branch(operand.addr(), flags::CARRY, false),
            Instr::Bcs => self.branch(operand.addr(), flags::CARRY, true),
            Instr::Bne => self.branch(operand.addr(), flags::ZERO, false),
            Instr::Beq => self.branch(operand.addr(), flags::ZERO, true),

            Instr::Jmp => self.pc = operand.addr(),

            Instr::Jsr => {
                let return_addr_minus_one = curr_pc.checked_add(2).unwrap();
                self.push2(return_addr_minus_one);

                self.pc = operand.addr();
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

    fn branch(&mut self, addr: u16, flag: u8, when: bool) {
        let is_set = flags::is_set(self.flags, flag);
        if is_set == when {
            self.pc = addr;
        }
    }
}

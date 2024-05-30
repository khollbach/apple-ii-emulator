#[derive(Debug, Clone, Copy)]
pub enum Instr {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
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

impl Mode {
    /// How many bytes long is this instruction, *not* including the opcode.
    pub fn arg_len(self) -> u16 {
        match self {
            Mode::Implied => 0,
            Mode::Accumulator => 0,

            Mode::Immediate => 1,
            Mode::Relative => 1,

            Mode::ZeroPage => 1,
            Mode::ZeroPageX => 1,
            Mode::ZeroPageY => 1,

            // Note: these are not the same as Indirect.
            Mode::XIndirect => 1,
            Mode::IndirectY => 1,

            Mode::Absolute => 2,
            Mode::AbsoluteX => 2,
            Mode::AbsoluteY => 2,

            Mode::Indirect => 2,
        }
    }

    pub fn instr_len(self) -> u16 {
        1 + self.arg_len()
    }
}

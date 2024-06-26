use anyhow::{bail, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    Implied,
    Accumulator,

    Immediate,
    Relative,

    ZeroPage,
    ZeroPageX,
    ZeroPageY,

    XIndirect,
    IndirectY,

    Absolute,
    AbsoluteX,
    AbsoluteY,

    Indirect,
}

impl Mode {
    pub fn instr_len(self) -> u16 {
        let arg_len = match self {
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
        };

        1 + arg_len
    }
}

pub fn decode(opcode: u8) -> Result<(Instr, Mode)> {
    // This code was generated by a script, from the data here:
    // https://www.masswerk.at/6502/6502_instruction_set.html
    let ret = match opcode {
        0x00 => (Instr::Brk, Mode::Implied),
        0x01 => (Instr::Ora, Mode::XIndirect),
        0x05 => (Instr::Ora, Mode::ZeroPage),
        0x06 => (Instr::Asl, Mode::ZeroPage),
        0x08 => (Instr::Php, Mode::Implied),
        0x09 => (Instr::Ora, Mode::Immediate),
        0x0a => (Instr::Asl, Mode::Accumulator),
        0x0d => (Instr::Ora, Mode::Absolute),
        0x0e => (Instr::Asl, Mode::Absolute),
        0x10 => (Instr::Bpl, Mode::Relative),
        0x11 => (Instr::Ora, Mode::IndirectY),
        0x15 => (Instr::Ora, Mode::ZeroPageX),
        0x16 => (Instr::Asl, Mode::ZeroPageX),
        0x18 => (Instr::Clc, Mode::Implied),
        0x19 => (Instr::Ora, Mode::AbsoluteY),
        0x1d => (Instr::Ora, Mode::AbsoluteX),
        0x1e => (Instr::Asl, Mode::AbsoluteX),
        0x20 => (Instr::Jsr, Mode::Absolute),
        0x21 => (Instr::And, Mode::XIndirect),
        0x24 => (Instr::Bit, Mode::ZeroPage),
        0x25 => (Instr::And, Mode::ZeroPage),
        0x26 => (Instr::Rol, Mode::ZeroPage),
        0x28 => (Instr::Plp, Mode::Implied),
        0x29 => (Instr::And, Mode::Immediate),
        0x2a => (Instr::Rol, Mode::Accumulator),
        0x2c => (Instr::Bit, Mode::Absolute),
        0x2d => (Instr::And, Mode::Absolute),
        0x2e => (Instr::Rol, Mode::Absolute),
        0x30 => (Instr::Bmi, Mode::Relative),
        0x31 => (Instr::And, Mode::IndirectY),
        0x35 => (Instr::And, Mode::ZeroPageX),
        0x36 => (Instr::Rol, Mode::ZeroPageX),
        0x38 => (Instr::Sec, Mode::Implied),
        0x39 => (Instr::And, Mode::AbsoluteY),
        0x3d => (Instr::And, Mode::AbsoluteX),
        0x3e => (Instr::Rol, Mode::AbsoluteX),
        0x40 => (Instr::Rti, Mode::Implied),
        0x41 => (Instr::Eor, Mode::XIndirect),
        0x45 => (Instr::Eor, Mode::ZeroPage),
        0x46 => (Instr::Lsr, Mode::ZeroPage),
        0x48 => (Instr::Pha, Mode::Implied),
        0x49 => (Instr::Eor, Mode::Immediate),
        0x4a => (Instr::Lsr, Mode::Accumulator),
        0x4c => (Instr::Jmp, Mode::Absolute),
        0x4d => (Instr::Eor, Mode::Absolute),
        0x4e => (Instr::Lsr, Mode::Absolute),
        0x50 => (Instr::Bvc, Mode::Relative),
        0x51 => (Instr::Eor, Mode::IndirectY),
        0x55 => (Instr::Eor, Mode::ZeroPageX),
        0x56 => (Instr::Lsr, Mode::ZeroPageX),
        0x58 => (Instr::Cli, Mode::Implied),
        0x59 => (Instr::Eor, Mode::AbsoluteY),
        0x5d => (Instr::Eor, Mode::AbsoluteX),
        0x5e => (Instr::Lsr, Mode::AbsoluteX),
        0x60 => (Instr::Rts, Mode::Implied),
        0x61 => (Instr::Adc, Mode::XIndirect),
        0x65 => (Instr::Adc, Mode::ZeroPage),
        0x66 => (Instr::Ror, Mode::ZeroPage),
        0x68 => (Instr::Pla, Mode::Implied),
        0x69 => (Instr::Adc, Mode::Immediate),
        0x6a => (Instr::Ror, Mode::Accumulator),
        0x6c => (Instr::Jmp, Mode::Indirect),
        0x6d => (Instr::Adc, Mode::Absolute),
        0x6e => (Instr::Ror, Mode::Absolute),
        0x70 => (Instr::Bvs, Mode::Relative),
        0x71 => (Instr::Adc, Mode::IndirectY),
        0x75 => (Instr::Adc, Mode::ZeroPageX),
        0x76 => (Instr::Ror, Mode::ZeroPageX),
        0x78 => (Instr::Sei, Mode::Implied),
        0x79 => (Instr::Adc, Mode::AbsoluteY),
        0x7d => (Instr::Adc, Mode::AbsoluteX),
        0x7e => (Instr::Ror, Mode::AbsoluteX),
        0x81 => (Instr::Sta, Mode::XIndirect),
        0x84 => (Instr::Sty, Mode::ZeroPage),
        0x85 => (Instr::Sta, Mode::ZeroPage),
        0x86 => (Instr::Stx, Mode::ZeroPage),
        0x88 => (Instr::Dey, Mode::Implied),
        0x8a => (Instr::Txa, Mode::Implied),
        0x8c => (Instr::Sty, Mode::Absolute),
        0x8d => (Instr::Sta, Mode::Absolute),
        0x8e => (Instr::Stx, Mode::Absolute),
        0x90 => (Instr::Bcc, Mode::Relative),
        0x91 => (Instr::Sta, Mode::IndirectY),
        0x94 => (Instr::Sty, Mode::ZeroPageX),
        0x95 => (Instr::Sta, Mode::ZeroPageX),
        0x96 => (Instr::Stx, Mode::ZeroPageY),
        0x98 => (Instr::Tya, Mode::Implied),
        0x99 => (Instr::Sta, Mode::AbsoluteY),
        0x9a => (Instr::Txs, Mode::Implied),
        0x9d => (Instr::Sta, Mode::AbsoluteX),
        0xa0 => (Instr::Ldy, Mode::Immediate),
        0xa1 => (Instr::Lda, Mode::XIndirect),
        0xa2 => (Instr::Ldx, Mode::Immediate),
        0xa4 => (Instr::Ldy, Mode::ZeroPage),
        0xa5 => (Instr::Lda, Mode::ZeroPage),
        0xa6 => (Instr::Ldx, Mode::ZeroPage),
        0xa8 => (Instr::Tay, Mode::Implied),
        0xa9 => (Instr::Lda, Mode::Immediate),
        0xaa => (Instr::Tax, Mode::Implied),
        0xac => (Instr::Ldy, Mode::Absolute),
        0xad => (Instr::Lda, Mode::Absolute),
        0xae => (Instr::Ldx, Mode::Absolute),
        0xb0 => (Instr::Bcs, Mode::Relative),
        0xb1 => (Instr::Lda, Mode::IndirectY),
        0xb4 => (Instr::Ldy, Mode::ZeroPageX),
        0xb5 => (Instr::Lda, Mode::ZeroPageX),
        0xb6 => (Instr::Ldx, Mode::ZeroPageY),
        0xb8 => (Instr::Clv, Mode::Implied),
        0xb9 => (Instr::Lda, Mode::AbsoluteY),
        0xba => (Instr::Tsx, Mode::Implied),
        0xbc => (Instr::Ldy, Mode::AbsoluteX),
        0xbd => (Instr::Lda, Mode::AbsoluteX),
        0xbe => (Instr::Ldx, Mode::AbsoluteY),
        0xc0 => (Instr::Cpy, Mode::Immediate),
        0xc1 => (Instr::Cmp, Mode::XIndirect),
        0xc4 => (Instr::Cpy, Mode::ZeroPage),
        0xc5 => (Instr::Cmp, Mode::ZeroPage),
        0xc6 => (Instr::Dec, Mode::ZeroPage),
        0xc8 => (Instr::Iny, Mode::Implied),
        0xc9 => (Instr::Cmp, Mode::Immediate),
        0xca => (Instr::Dex, Mode::Implied),
        0xcc => (Instr::Cpy, Mode::Absolute),
        0xcd => (Instr::Cmp, Mode::Absolute),
        0xce => (Instr::Dec, Mode::Absolute),
        0xd0 => (Instr::Bne, Mode::Relative),
        0xd1 => (Instr::Cmp, Mode::IndirectY),
        0xd5 => (Instr::Cmp, Mode::ZeroPageX),
        0xd6 => (Instr::Dec, Mode::ZeroPageX),
        0xd8 => (Instr::Cld, Mode::Implied),
        0xd9 => (Instr::Cmp, Mode::AbsoluteY),
        0xdd => (Instr::Cmp, Mode::AbsoluteX),
        0xde => (Instr::Dec, Mode::AbsoluteX),
        0xe0 => (Instr::Cpx, Mode::Immediate),
        0xe1 => (Instr::Sbc, Mode::XIndirect),
        0xe4 => (Instr::Cpx, Mode::ZeroPage),
        0xe5 => (Instr::Sbc, Mode::ZeroPage),
        0xe6 => (Instr::Inc, Mode::ZeroPage),
        0xe8 => (Instr::Inx, Mode::Implied),
        0xe9 => (Instr::Sbc, Mode::Immediate),
        0xea => (Instr::Nop, Mode::Implied),
        0xec => (Instr::Cpx, Mode::Absolute),
        0xed => (Instr::Sbc, Mode::Absolute),
        0xee => (Instr::Inc, Mode::Absolute),
        0xf0 => (Instr::Beq, Mode::Relative),
        0xf1 => (Instr::Sbc, Mode::IndirectY),
        0xf5 => (Instr::Sbc, Mode::ZeroPageX),
        0xf6 => (Instr::Inc, Mode::ZeroPageX),
        0xf8 => (Instr::Sed, Mode::Implied),
        0xf9 => (Instr::Sbc, Mode::AbsoluteY),
        0xfd => (Instr::Sbc, Mode::AbsoluteX),
        0xfe => (Instr::Inc, Mode::AbsoluteX),
        _ => bail!("invalid opcode: 0x{opcode:02x}"),
    };
    Ok(ret)
}

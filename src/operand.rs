use crate::{instr::Mode, Cpu};

#[derive(Debug, Clone, Copy)]
pub enum Operand {
    Memory { addr: u16 },
    Literal { value: u8 },
    Accumulator,
    None,
}

impl Operand {
    pub fn from_mode(cpu: &Cpu, mode: Mode) -> Self {
        match mode {
            Mode::Implied => Self::None,
            Mode::Accumulator => Self::Accumulator,

            Mode::Immediate => {
                let value = cpu.get_byte(cpu.pc.checked_add(1).unwrap());
                Self::Literal { value }
            }

            Mode::Relative => {
                let value = cpu.get_byte(cpu.pc.checked_add(1).unwrap());
                let offset = value as i8 as i16;
                // todo: detect under/overflow and panic
                let addr = cpu.pc.wrapping_add(offset as u16);
                Self::Memory { addr }
            }

            Mode::ZeroPage => {
                let lo = cpu.get_byte(cpu.pc.checked_add(1).unwrap());
                let addr = u16::from_le_bytes([lo, 0]);
                Self::Memory { addr }
            }
            Mode::ZeroPageX => todo!(),
            Mode::ZeroPageY => todo!(),

            Mode::Absolute => {
                let addr = cpu.get_word(cpu.pc.checked_add(1).unwrap());
                Self::Memory { addr }
            }
            Mode::AbsoluteX => todo!(),
            Mode::AbsoluteY => todo!(),

            Mode::Indirect => {
                let pointer = cpu.get_word(cpu.pc.checked_add(1).unwrap());
                let addr = cpu.get_word(pointer);
                Self::Memory { addr }
            }
            Mode::XIndirect => todo!(),
            Mode::IndirectY => todo!(),
        }
    }

    pub fn get(self, cpu: &Cpu) -> u8 {
        match self {
            Self::Memory { addr } => cpu.get_byte(addr),
            Self::Literal { value } => value,
            Self::Accumulator => cpu.a,
            Self::None => panic!("operand is none; cannot get its value"),
        }
    }

    pub fn set(self, cpu: &mut Cpu, value: u8) {
        match self {
            Self::Memory { addr } => cpu.set_byte(addr, value),
            Self::Literal { .. } => panic!("cannot mutate literal value {self:?}"),
            Self::Accumulator => cpu.a = value,
            Self::None => panic!("operand is none; cannot set its value"),
        }
    }

    pub fn addr(self) -> u16 {
        match self {
            Self::Memory { addr } => addr,
            _ => panic!("operand doesn't have a memory address: {self:?}"),
        }
    }
}

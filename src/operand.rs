use crate::{instr::Mode, Cpu};

#[derive(Debug, Clone, Copy)]
pub(super) enum Operand {
    Memory { addr: u16 },
    Literal { value: u8 },
    Accumulator,
    None,
}

impl Cpu {
    pub(super) fn operand(&self, mode: Mode) -> Operand {
        match mode {
            Mode::Implied => Operand::None,
            Mode::Accumulator => Operand::Accumulator,

            Mode::Immediate => {
                let value = self.get_byte(self.pc.checked_add(1).unwrap());
                Operand::Literal { value }
            }

            Mode::Relative => {
                let value = self.get_byte(self.pc.checked_add(1).unwrap());
                let offset = value as i8 as i16;
                // todo: detect under/overflow and panic
                let addr = self.pc.wrapping_add(offset as u16);
                Operand::Memory { addr }
            }

            Mode::ZeroPage => {
                let lo = self.get_byte(self.pc.checked_add(1).unwrap());
                let addr = u16::from_le_bytes([lo, 0]);
                Operand::Memory { addr }
            }
            Mode::ZeroPageX => todo!(),
            Mode::ZeroPageY => todo!(),

            Mode::Absolute => {
                let addr = self.get_word(self.pc.checked_add(1).unwrap());
                Operand::Memory { addr }
            }
            Mode::AbsoluteX => todo!(),
            Mode::AbsoluteY => todo!(),

            Mode::Indirect => {
                let pointer = self.get_word(self.pc.checked_add(1).unwrap());
                let addr = self.get_word(pointer);
                Operand::Memory { addr }
            }
            Mode::XIndirect => todo!(),
            Mode::IndirectY => todo!(),
        }
    }

    pub(super) fn get_operand_value(&mut self, operand: Operand) -> u8 {
        match operand {
            Operand::Memory { addr } => self.get_byte(addr),
            Operand::Literal { value } => value,
            Operand::Accumulator => self.a,
            Operand::None => panic!("operand is none; cannot get its value"),
        }
    }

    pub(super) fn set_operand_value(&mut self, operand: Operand, value: u8) {
        match operand {
            Operand::Memory { addr } => self.set_byte(addr, value),
            Operand::Literal { .. } => panic!("cannot mutate literal value {operand:?}"),
            Operand::Accumulator => self.a = value,
            Operand::None => panic!("operand is none; cannot set its value"),
        }
    }
}

impl Operand {
    pub(super) fn addr(self) -> u16 {
        match self {
            Operand::Memory { addr } => addr,
            _ => panic!("operand doesn't have a memory address: {self:?}"),
        }
    }
}

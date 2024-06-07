use crate::cpu::{instr::Mode, Cpu};

#[derive(Debug, Clone, Copy)]
pub enum Operand {
    Memory { addr: u16 },
    Literal { value: u8 },
    Accumulator,
    None,
}

impl Operand {
    pub fn from_mode(cpu: &Cpu, mode: Mode) -> Self {
        let arg: u16 = match mode.arg_len() {
            0 => 0,
            1 => cpu.mem.get(cpu.pc.checked_add(1).unwrap()).into(),
            2 => cpu.get_word(cpu.pc.checked_add(1).unwrap()),
            _ => unreachable!(),
        };

        match mode {
            Mode::Implied => Self::None,
            Mode::Accumulator => Self::Accumulator,
            Mode::Immediate => Self::Literal { value: arg as u8 },

            Mode::Relative => {
                // Note: branch offset is relative to the *next* instruction,
                // not the current one.
                let base = cpu.pc.checked_add(2).unwrap();
                let offset = arg as u8 as i8;
                Self::Memory {
                    addr: checked_offset(base, offset).unwrap(),
                }
            }

            Mode::ZeroPage => Self::Memory { addr: arg },
            Mode::ZeroPageX => Self::Memory {
                addr: (arg as u8).wrapping_add(cpu.x).into(),
            },
            Mode::ZeroPageY => Self::Memory {
                addr: (arg as u8).wrapping_add(cpu.y).into(),
            },

            Mode::Absolute => Self::Memory { addr: arg },
            Mode::AbsoluteX => Self::Memory {
                addr: arg.checked_add(cpu.x as u16).unwrap(),
            },
            Mode::AbsoluteY => Self::Memory {
                addr: arg.checked_add(cpu.y as u16).unwrap(),
            },

            Mode::Indirect => Self::Memory {
                addr: cpu.get_word(arg),
            },
            Mode::XIndirect => Self::Memory {
                addr: cpu.get_word((arg as u8).wrapping_add(cpu.x) as u16),
            },
            Mode::IndirectY => Self::Memory {
                addr: cpu.get_word(arg).checked_add(cpu.y as u16).unwrap(),
            },
        }
    }

    pub fn get(self, cpu: &Cpu) -> u8 {
        match self {
            Self::Memory { addr } => cpu.mem.get(addr),
            Self::Literal { value } => value,
            Self::Accumulator => cpu.a,
            Self::None => panic!("operand is none; cannot get its value"),
        }
    }

    pub fn set(self, cpu: &mut Cpu, value: u8) {
        match self {
            Self::Memory { addr } => cpu.mem.set(addr, value),
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

fn checked_offset(addr: u16, offset: i8) -> Option<u16> {
    if offset >= 0 {
        return addr.checked_add(offset as u16);
    }

    let abs_offset = (offset as i16).abs() as u16;
    addr.checked_sub(abs_offset)
}

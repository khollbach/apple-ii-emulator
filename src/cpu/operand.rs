use std::fmt;

use crate::{
    cpu::{instr::Mode, Cpu},
    memory::AddressSpace,
};

/// This abstracts away the details of addressing modes.
#[derive(Clone, Copy)]
pub enum Operand {
    Memory { addr: u16 },
    Literal { value: u8 },
    Accumulator,
    None,
}

impl Operand {
    pub fn new(cpu: &Cpu, mem: &mut AddressSpace, mode: Mode) -> Self {
        let arg_len = mode.instr_len() - 1;
        let arg: u16 = match arg_len {
            0 => 0,
            1 => mem.read(cpu.pc.checked_add(1).unwrap()).into(),
            2 => read_word(mem, cpu.pc.checked_add(1).unwrap()),
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
                addr: read_word(mem, arg),
            },
            Mode::XIndirect => Self::Memory {
                addr: read_word(mem, (arg as u8).wrapping_add(cpu.x) as u16),
            },
            Mode::IndirectY => Self::Memory {
                addr: read_word(mem, arg).checked_add(cpu.y as u16).unwrap(),
            },
        }
    }

    pub fn get(self, cpu: &Cpu, mem: &mut AddressSpace) -> u8 {
        match self {
            Self::Memory { addr } => mem.read(addr),
            Self::Literal { value } => value,
            Self::Accumulator => cpu.a,
            Self::None => panic!("operand is none; cannot get its value"),
        }
    }

    pub fn set(self, cpu: &mut Cpu, mem: &mut AddressSpace, value: u8) {
        match self {
            Self::Memory { addr } => mem.write(addr, value),
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

fn read_word(mem: &mut AddressSpace, addr: u16) -> u16 {
    let lo = mem.read(addr);
    let hi = mem.read(addr.checked_add(1).unwrap());
    u16::from_le_bytes([lo, hi])
}

fn checked_offset(addr: u16, offset: i8) -> Option<u16> {
    if offset >= 0 {
        return addr.checked_add(offset as u16);
    }

    let abs_offset = (offset as i16).abs() as u16;
    addr.checked_sub(abs_offset)
}

impl fmt::Debug for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Memory { addr } => f
                .debug_struct("Memory")
                .field("addr", &AddrDbg(addr))
                .finish(),
            Self::Literal { value } => f
                .debug_struct("Literal")
                .field("value", &ValueDbg(value))
                .finish(),
            Self::Accumulator => write!(f, "Accumulator"),
            Self::None => write!(f, "None"),
        }
    }
}

struct AddrDbg(u16);

impl fmt::Debug for AddrDbg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:04x}", self.0)
    }
}

struct ValueDbg(u8);

impl fmt::Debug for ValueDbg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:02x}", self.0)
    }
}

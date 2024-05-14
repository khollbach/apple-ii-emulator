pub const CARRY: u8 = 1 << 0;
pub const ZERO: u8 = 1 << 1;
pub const INTERRUPT: u8 = 1 << 2;
pub const DECIMAL: u8 = 1 << 3;

pub const BREAK: u8 = 1 << 4;
const _RESERVED: u8 = 1 << 5;
pub const OVERFLOW: u8 = 1 << 6;
pub const NEGATIVE: u8 = 1 << 7;

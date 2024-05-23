pub const CARRY: u8 = 1 << 0;
pub const ZERO: u8 = 1 << 1;
pub const INTERRUPT: u8 = 1 << 2;
pub const DECIMAL: u8 = 1 << 3;

pub const BREAK: u8 = 1 << 4;
pub const RESERVED: u8 = 1 << 5;
pub const OVERFLOW: u8 = 1 << 6;
pub const NEGATIVE: u8 = 1 << 7;

pub fn set(flags: &mut u8, flag: u8) {
    assert_eq!(flag.count_ones(), 1);
    *flags |= flag;
}

pub fn clear(flags: &mut u8, flag: u8) {
    assert_eq!(flag.count_ones(), 1);
    *flags &= !flag;
}

pub fn set_to(flags: &mut u8, flag: u8, setting: bool) {
    if setting {
        set(flags, flag)
    } else {
        clear(flags, flag)
    }
}

pub fn is_set(flags: u8, flag: u8) -> bool {
    assert_eq!(flag.count_ones(), 1);
    flags & flag != 0
}

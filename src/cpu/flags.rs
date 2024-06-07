#[repr(u8)]
pub enum Flag {
    Carry = 1 << 0,
    Zero = 1 << 1,
    Interrupt = 1 << 2,
    Decimal = 1 << 3,

    Break = 1 << 4,
    Reserved = 1 << 5,
    Overflow = 1 << 6,
    Negative = 1 << 7,
}

#[derive(Clone)]
pub struct Flags {
    pub bits: u8,
}

impl Flags {
    pub fn set(&mut self, flag: Flag) {
        self.bits |= flag as u8;
    }

    pub fn clear(&mut self, flag: Flag) {
        self.bits &= !(flag as u8);
    }

    pub fn assign(&mut self, flag: Flag, setting: bool) {
        if setting {
            self.set(flag)
        } else {
            self.clear(flag)
        }
    }

    pub fn is_set(&self, flag: Flag) -> bool {
        self.bits & (flag as u8) != 0
    }

    /// Update negative and zero flags, based on the value.
    ///
    /// Return the value, for convenience.
    pub fn nz(&mut self, value: u8) -> u8 {
        self.assign(Flag::Zero, value == 0);
        self.assign(Flag::Negative, (value as i8) < 0);
        value
    }
}

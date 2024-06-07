//! Misc arithmetic helper functions for CPU operations.

/// Perform addition, detecting signed integer overflows.
pub fn add(x: u8, y: u8, carry_in: bool) -> Add {
    let (sum, carry_out) = add_with_carry(x, y, carry_in);

    let overflow = {
        let same_sign = x & 0x80 == y & 0x80;
        let flipped = sum & 0x80 != x & 0x80;
        same_sign && flipped
    };

    Add {
        sum,
        carry: carry_out,
        overflow,
    }
}

pub struct Add {
    pub sum: u8,
    pub carry: bool,
    pub overflow: bool,
}

fn add_with_carry(x: u8, y: u8, carry_in: bool) -> (u8, bool) {
    let (s1, c1) = x.overflowing_add(y);
    if !carry_in {
        return (s1, c1);
    }

    let (s2, c2) = s1.overflowing_add(1);
    debug_assert!(!(c1 && c2));
    (s2, c1 || c2)
}

/// The standard library `overflowing_shl` behaviour isn't what I expected, so
/// we write our own that does what we want.
///
/// The (weird) behaviour of std::u8::overflowing_shl is:
/// ```
/// assert_eq!(0x80_u8.overflowing_shl(1), (0, false)); // ???
/// assert_eq!(0x55_u8.overflowing_shl(0x81), (0xaa, true)); // I mean, sure?
/// ```
pub fn overflowing_shl(x: u8, shift_amount: u8) -> (u8, bool) {
    assert!(shift_amount < 8);
    let out = x << shift_amount;
    let overflow = out.count_ones() != x.count_ones();
    (out, overflow)
}

/// Returns true if any of the bits got shifted off the end.
///
/// See also `overflowing_shl`.
pub fn overflowing_shr(x: u8, shift_amount: u8) -> (u8, bool) {
    assert!(shift_amount < 8);
    let out = x >> shift_amount;
    let overflow = out.count_ones() != x.count_ones();
    (out, overflow)
}

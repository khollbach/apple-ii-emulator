use anyhow::{ensure, Context, Result};
use itertools::Itertools;

/// Optional $ or 0x prefix.
pub fn decode_u16(mut s: &str) -> Result<u16> {
    if let Some(stripped) = s.strip_prefix("$") {
        s = stripped;
    } else if let Some(stripped) = s.strip_prefix("0x") {
        s = stripped;
    }

    ensure!(!s.is_empty());
    ensure!(s.len() <= 4);

    let left_pad = "0".repeat(4_usize.saturating_sub(s.len()));
    let digits = format!("{left_pad}{s}");
    let hi = hex_to_byte(&digits[0..2])?;
    let lo = hex_to_byte(&digits[2..4])?;
    Ok(u16::from_be_bytes([hi, lo]))
}

fn hex_to_byte(s: &str) -> Result<u8> {
    ensure!(s.len() == 2);
    let (hi, lo) = s.chars().collect_tuple().unwrap();
    let hi = hex_to_nibble(hi)?;
    let lo = hex_to_nibble(lo)?;
    let byte = hi << 4 | lo;
    Ok(byte)
}

fn hex_to_nibble(c: char) -> Result<u8> {
    let n: u32 = c
        .to_digit(16)
        .with_context(|| format!("not a hex digit: {c:?}"))?;
    Ok(n as u8)
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case("0000", 0x0)]
    #[test_case("1234", 0x1234)]
    #[test_case("0x1234", 0x1234)]
    #[test_case("$1234", 0x1234; "dollar sign")]
    #[test_case("abcd", 0xabcd)]
    #[test_case("ffff", 0xffff)]
    #[test_case("aBcD", 0xabcd; "mixed case")]
    #[test_case("abc", 0x0abc)]
    #[test_case("9", 0x9)]
    #[test_case("$1", 0x1)]
    fn ok(s: &str, expected: u16) {
        let actual = decode_u16(s).unwrap();
        assert_eq!(expected, actual);
    }

    #[test_case("")]
    #[test_case("0X1234")]
    #[test_case("12345")]
    #[test_case("123456")]
    #[test_case("123g")]
    #[test_case("123G"; "capital g")]
    #[test_case("12 34"; "space inside")]
    #[test_case(" 1234"; "space before")]
    #[test_case("1234 "; "space after")]
    #[test_case(" "; "all whitespace")]
    fn err(s: &str) {
        let result = decode_u16(s);
        assert!(result.is_err());
    }
}

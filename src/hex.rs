use anyhow::{ensure, Result};

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
    let bytes = hex::decode(format!("{left_pad}{s}"))?;
    debug_assert_eq!(bytes.len(), 2);
    Ok(u16::from_be_bytes(bytes.try_into().unwrap()))
}

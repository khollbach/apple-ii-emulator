use std::mem;

use super::{CELL_H, CELL_W};

const SPRITES_RAW: &[u8; CELL_W * CELL_H * 256] = include_bytes!("spritesheet.in");

pub const SPRITES: &[[[bool; CELL_W]; CELL_H]; 256] = unsafe { mem::transmute(SPRITES_RAW) };

#[cfg(test)]
mod tests {
    use super::SPRITES_RAW;

    #[test]
    fn spritesheet_all_bool() {
        SPRITES_RAW.iter().all(|byte| [0, 1].contains(byte));
    }
}

use super::color::Color;
use crate::display::hgr;

pub const W: usize = 40;
pub const H: usize = 48;

pub const BLOCK_W: usize = 7;
pub const BLOCK_H: usize = 4;

pub fn dots(page: &[u8]) -> Vec<Vec<Color>> {
    let blocks = color_grid(page);

    let mut out = vec![vec![Color::Black; hgr::W]; hgr::H];
    for y in 0..hgr::H {
        for x in 0..hgr::W {
            out[y][x] = blocks[y / BLOCK_H][x / BLOCK_W];
        }
    }
    out
}

fn color_grid(page: &[u8]) -> Vec<Vec<Color>> {
    let bytes = unscramble_bytes(page);

    let mut out = vec![vec![Color::Black; W]; H];
    for y in 0..H / 2 {
        for x in 0..W {
            let b = bytes[y][x];
            let top = b & 0xf;
            let bot = b >> 4;
            out[y * 2][x] = Color::from_nibble(top);
            out[y * 2 + 1][x] = Color::from_nibble(bot);
        }
    }
    out
}

pub(super) fn unscramble_bytes(page: &[u8]) -> Vec<Vec<u8>> {
    assert_eq!(page.len(), 0x400); // 1 KiB

    let mut out = vec![vec![0u8; W]; H / 2];
    for i in 0..8 {
        for j in 0..3 {
            let row = &page[i * 0x80 + j * W..][..W];
            out[j * 8 + i].copy_from_slice(row);
        }
    }
    out
}

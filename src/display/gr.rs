use super::color::Color;
use crate::display::hgr;

pub const W: usize = 40;
pub const H: usize = 24; // todo: make this 48 and fix up details elsewhere accordingly

pub const BLOCK_W: usize = 7;
pub const BLOCK_H2: usize = 8; // Note: "top-half" & "bottom-half" are 4 dots each.
                               // ^todo make this 4, and just 'H'

pub fn ram_to_dots(mem: &[u8]) -> Vec<Vec<Color>> {
    assert_eq!(mem.len(), 2_usize.pow(16)); // 64 KiB

    let page1 = &mem[0x400..0x800];
    let grid = color_grid(page1);
    let mut out = vec![vec![Color::Black; hgr::W]; hgr::H];
    for y in 0..hgr::H {
        for x in 0..hgr::W {
            out[y][x] = grid[y / BLOCK_H2][x / BLOCK_W];
        }
    }
    out
}

fn color_grid(page: &[u8]) -> Vec<Vec<Color>> {
    let bytes = unscramble(page);

    let mut out = vec![vec![Color::Black; W]; H];
    for y in 0..H {
        for x in 0..W {
            // todo: top-half + bot-half
            out[y][x] = Color::from_nibble(bytes[y][x] & 0xf);
        }
    }
    out
}

fn unscramble(page: &[u8]) -> Vec<Vec<u8>> {
    assert_eq!(page.len(), 0x400); // 1 KiB

    let mut out = vec![vec![0u8; W]; H];
    for i in 0..8 {
        for j in 0..3 {
            let row = &page[i * 0x80 + j * W..][..W];
            out[j * 8 + i].copy_from_slice(row);
        }
    }
    out
}

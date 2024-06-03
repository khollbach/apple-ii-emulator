pub mod color;

use color::Color;

pub const HGR_W: usize = 280;
pub const HGR_H: usize = 192;

pub const GR_W: usize = 40;
pub const GR_H: usize = 24; // todo: make this 48 and fix up details elsewhere accordingly

pub const GR_BLOCK_W: usize = 7;
pub const GR_BLOCK_H2: usize = 8; // Note: "top-half" & "bottom-half" are 4 dots each.
                                  // ^todo make this 4, and just 'H'

pub fn gr_to_280x192(mem: &[u8]) -> Vec<Vec<Color>> {
    assert_eq!(mem.len(), 2_usize.pow(16)); // 64 KiB

    let page1 = &mem[0x400..0x800];
    let grid = gr_grid(page1);
    let mut out = vec![vec![Color::Black; HGR_W]; HGR_H];
    for y in 0..HGR_H {
        for x in 0..HGR_W {
            out[y][x] = grid[y / GR_BLOCK_H2][x / GR_BLOCK_W];
        }
    }
    out
}

fn gr_grid(page: &[u8]) -> Vec<Vec<Color>> {
    let bytes = unscramble(page);

    let mut out = vec![vec![Color::Black; GR_W]; GR_H];
    for y in 0..GR_H {
        for x in 0..GR_W {
            // todo: top-half + bot-half
            out[y][x] = Color::from_nibble(bytes[y][x] & 0xf);
        }
    }
    out
}

fn unscramble(page: &[u8]) -> Vec<Vec<u8>> {
    assert_eq!(page.len(), 0x400); // 1 KiB

    let mut out = vec![vec![0u8; GR_W]; GR_H];

    for i in 0..8 {
        for j in 0..3 {
            let row = &page[i * 0x80 + j * GR_W..][..GR_W];
            out[j * 8 + i].copy_from_slice(row);
        }
    }

    out
}

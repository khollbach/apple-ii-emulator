mod color_math;

use std::{f64::consts::PI, mem};

pub const HGR_W: usize = 280;
pub const HGR_H: usize = 192;

pub const GR_W: usize = 40;
pub const GR_H: usize = 24; // todo: make this 48 and fix up details elsewhere accordingly

pub const GR_BLOCK_W: usize = 7;
pub const GR_BLOCK_H2: usize = 8; // Note: "top-half" & "bottom-half" are 4 dots each.
                                  // ^todo make this 4, and just 'H'

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0x0,
    Magenta = 0x1,
    DarkBlue = 0x2,
    Purple = 0x3,

    DarkGreen = 0x4,
    Grey1 = 0x5,
    MediumBlue = 0x6,
    LightBlue = 0x7,

    Brown = 0x8,
    Orange = 0x9,
    Grey2 = 0xa,
    Pink = 0xb,

    Green = 0xc,
    Yellow = 0xd,
    Aqua = 0xe,
    White = 0xf,
}

impl Color {
    fn from_nibble(b: u8) -> Self {
        assert!(b < 0x10);
        unsafe { mem::transmute(b) }
    }

    pub fn rgb(self) -> [u8; 3] {
        let yuv = self.yuv();
        color_math::yuv_to_rgb(yuv)
    }

    fn yuv(self) -> [f64; 3] {
        let x = (2_f64).sqrt() / PI;

        match self {
            Color::Black => [0., 0., 0.],
            Color::Magenta => [0.25, 0., x],
            Color::DarkBlue => [0.25, x, 0.],
            Color::Purple => [0.5, x, x],

            Color::DarkGreen => [0.25, 0., -x],
            Color::Grey1 => [0.5, 0., 0.],
            Color::MediumBlue => [0.5, x, -x],
            Color::LightBlue => [0.75, x, 0.],

            Color::Brown => [0.25, -x, 0.],
            Color::Orange => [0.5, -x, x],
            Color::Grey2 => [0.5, 0., 0.],
            Color::Pink => [0.75, 0., x],

            Color::Green => [0.5, -x, -x],
            Color::Yellow => [0.75, -x, 0.],
            Color::Aqua => [0.75, 0., -x],
            Color::White => [1., 0., 0.],
        }
    }
}

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

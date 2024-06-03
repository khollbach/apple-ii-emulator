mod spritesheet;

use core::fmt;

use spritesheet::SPRITES;

use super::{color::Color, hgr};
use crate::display::gr::unscramble_bytes;

pub const W: usize = 40;
pub const H: usize = 24;

pub const CELL_W: usize = 7;
pub const CELL_H: usize = 8;

pub fn ram_to_text(mem: &[u8]) -> Vec<Vec<Glyph>> {
    assert_eq!(mem.len(), 2_usize.pow(16)); // 64 KiB

    let page1 = &mem[0x400..0x800];
    let bytes = unscramble_bytes(page1);

    let mut out = vec![vec![Glyph::Cursor; W]; H];
    for y in 0..H {
        for x in 0..W {
            out[y][x] = Glyph::from_byte(bytes[y][x]);
        }
    }
    out
}

pub fn ram_to_dots(mem: &[u8]) -> Vec<Vec<Color>> {
    let screen = ram_to_text(mem);

    let mut out = vec![vec![Color::Black; hgr::W]; hgr::H];
    for y in 0..H {
        for x in 0..W {
            draw(&mut out, x * CELL_W, y * CELL_H, screen[y][x]);
        }
    }
    out
}

fn draw(dots: &mut Vec<Vec<Color>>, x: usize, y: usize, glyph: Glyph) {
    let sprite = glyph.dots();
    for dy in 0..CELL_H {
        for dx in 0..CELL_W {
            let color = if sprite[dy][dx] {
                Color::White
            } else {
                Color::Black
            };
            dots[y + dy][x + dx] = color;
        }
    }
}

// (can add inverse & blinking text at some point)
#[derive(Debug, Clone, Copy)]
pub enum Glyph {
    /// 0x20..=0x7e
    PrintableAscii(u8),

    /// Checkerboard block, often used as a cursor when typing text.
    Cursor,
}

impl fmt::Display for Glyph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = match *self {
            Glyph::PrintableAscii(b) => b as char,
            Glyph::Cursor => '�',
        };
        write!(f, "{}", c)
    }
}

impl Glyph {
    // this isn't quite right; but it's close enough for now.
    fn from_byte(mut b: u8) -> Self {
        // clear them hibits (not technically correct, but we can come back to this)
        if b >= 0x80 {
            b -= 0x80;
        }

        match b {
            0x00..=0x1f => Self::Cursor, // todo: these should show up as capital letters

            // "normal" range
            0x20..=0x7e => Self::PrintableAscii(b),
            0x7f => Self::Cursor,

            0x80..=0xff => unreachable!(),
        }
    }

    fn to_byte(self) -> u8 {
        match self {
            Glyph::PrintableAscii(b) => b | 0x80, // todo: handle hibit details correctly
            Glyph::Cursor => 0xff,
        }
    }

    fn dots(self) -> [[bool; CELL_W]; CELL_H] {
        let idx = self.to_byte() as usize;
        SPRITES[idx]
    }
}

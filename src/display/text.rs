use crate::display::gr::unscramble_bytes;

pub const W: usize = 40;
pub const H: usize = 24;

pub const CELL_W: usize = 7;
pub const CELL_H: usize = 8;

pub fn ram_to_text(mem: &[u8]) -> Vec<Vec<char>> {
    assert_eq!(mem.len(), 2_usize.pow(16)); // 64 KiB

    let page1 = &mem[0x400..0x800];
    let bytes = unscramble_bytes(page1);

    let mut out = vec![vec![' '; W]; H];
    for y in 0..H {
        for x in 0..W {
            out[y][x] = byte_to_char(bytes[y][x]);
        }
    }
    out
}

// this isn't quite right; but it's close enough for now.
fn byte_to_char(b: u8) -> char {
    text_byte_to_ascii(b) as char
}

fn text_byte_to_ascii(mut b: u8) -> u8 {
    // clear them hibits (not technically correct, but we can come back to this)
    if b >= 0x80 {
        b -= 0x80;
    }

    match b {
        0x00..=0x1f => b'~', // todo: these should actually be capital letters

        // "normal" range
        0x20..=0x7e => b,
        0x7f => b'~', // todo: special "block" glyph

        0x80..=0xff => unreachable!(),
    }
}

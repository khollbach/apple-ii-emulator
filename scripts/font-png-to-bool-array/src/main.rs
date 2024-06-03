use std::{
    collections::HashMap,
    io::{self, Write},
    mem,
};

use anyhow::Result;
use image::{io::Reader as ImageReader, DynamicImage, GenericImageView, ImageBuffer, Rgb, Rgba};

const GLYPH_W: usize = 7;
const GLYPH_H: usize = 8;

fn main() -> Result<()> {
    // let img = ImageReader::open("Apple2e.png")?.decode()?;
    // dbg!(img.width(), img.height());
    // let scale = 2;
    // let n_rows = img.height() / scale / GLYPH_H;
    // let n_cols = img.width() / scale / GLYPH_W;
    // dbg!(n_rows, n_cols);
    // assert_eq!(n_rows * scale * GLYPH_H, img.height());
    // assert_eq!(n_cols * scale * GLYPH_W, img.width());

    // let img = unscale(img);
    // img.save("scaled-down.png")?;
    // panic!();

    let img = ImageReader::open("scaled-down.png")?.decode()?;
    dbg!(img.width(), img.height());
    let n_rows = img.height() / GLYPH_H as u32;
    let n_cols = img.width() / GLYPH_W as u32;
    dbg!(n_rows, n_cols);
    assert_eq!(n_rows * GLYPH_H as u32, img.height());
    assert_eq!(n_cols * GLYPH_W as u32, img.width());

    // build a hashtable of   '#' => (0, 2)   [e.g.]

    let layout = [
        br###"!"#$%&'()*+,-./0123456789:;<=>?@"###,
        br###"ABCDEFGHIJKLMNOPQRSTUVWXYZ[\]^_`"###,
        br###"abcdefghijklmonpqrstuvwxyz{|}~  "###,
        //                                  ^the block is 2nd from last
    ];
    assert_eq!(layout.len(), n_rows as usize);
    assert_eq!(layout[0].len(), n_cols as usize);

    let mut map = HashMap::new();
    for (y, row) in layout.iter().enumerate() {
        for (x, &byte) in row.iter().enumerate() {
            let x = x as u32;
            let y = y as u32;
            if (x, y) == (n_cols - 1, n_rows - 1) {
                continue;
            }

            let exists = map.insert(byte, (x, y));
            assert!(exists.is_none());
        }
    }

    // dbg!(map);

    let table = [
        // Inverse.
        br###"@ABCDEFGHIJKLMNO"###,
        br###"PQRSTUVWXYZ[\]^_"###,
        br###" !"#$%&'()*+,-./"###,
        br###"0123456789:;<=>?"###,
        // Blinking.
        br###"@ABCDEFGHIJKLMNO"###,
        br###"PQRSTUVWXYZ[\]^_"###,
        br###" !"#$%&'()*+,-./"###,
        br###"0123456789:;<=>?"###,
        //
        br###"@ABCDEFGHIJKLMNO"###,
        br###"PQRSTUVWXYZ[\]^_"###,
        br###" !"#$%&'()*+,-./"###,
        br###"0123456789:;<=>?"###,
        //
        br###"@ABCDEFGHIJKLMNO"###,
        br###"PQRSTUVWXYZ[\]^_"###,
        br###"`abcdefghijklmno"###,
        br###"pqrstuvwxyz{|}~ "###,
    ];

    for i in 0..16 {
        for j in 0..16 {
            let (x, y) = if (i, j) == (15, 15) {
                // special case the BLOCK
                (n_cols - 2, n_rows - 1)
            } else {
                let ascii = table[i][j];
                map[&ascii]
            };

            let bool_array = get_glyph(&img, x * GLYPH_W as u32, y * GLYPH_H as u32);
            dbg_bool_array(bool_array);
            let raw_bytes: [u8; GLYPH_W * GLYPH_H] = unsafe { mem::transmute(bool_array) };
            io::stdout().write_all(&raw_bytes)?;
        }
    }

    Ok(())
}

fn dbg_bool_array(bool_array: [[bool; 7]; 8]) {
    eprintln!();
    for y in 0..8 {
        for x in 0..7 {
            let c = if bool_array[y][x] { '#' } else { '.' };
            eprint!("{}", c);
        }
        eprintln!();
    }
}

fn get_glyph(img: &DynamicImage, x: u32, y: u32) -> [[bool; GLYPH_W]; GLYPH_H] {
    let mut out = [[false; GLYPH_W]; GLYPH_H];
    for dy in 0..GLYPH_H as u32 {
        for dx in 0..GLYPH_W as u32 {
            let pix = img.get_pixel(x + dx, y + dy);
            let bit = pix != Rgba::from([0, 0, 0, 255]);
            out[dy as usize][dx as usize] = bit;
        }
    }
    out
}

// ---

fn stats(img: DynamicImage) {
    let mut vals = vec![std::collections::HashSet::new(); 4];
    for y in 0..img.height() {
        for x in 0..img.width() {
            let p = img.get_pixel(x, y);
            for i in 0..4 {
                vals[i].insert(p.0[i]);
            }
        }
    }
    dbg!(vals);
}

#[allow(dead_code)]
fn unscale(img: DynamicImage) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    stats(img.clone());

    let mut out = ImageBuffer::new(img.width() / 2, img.height() / 2);
    for y in 0..out.height() {
        for x in 0..out.width() {
            out[(x, y)] = if img.get_pixel(x * 2, y * 2) != Rgba::from([0, 0, 0, 255]) {
                Rgb::from([255, 255, 255])
            } else {
                Rgb::from([0, 0, 0])
            };
        }
    }
    out
}

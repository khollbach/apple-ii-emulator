use super::color::Color;
use crate::display::gr;

pub const W: usize = 280;
pub const H: usize = 192;

/// Black & white display.
pub fn ram_to_dots_bw(mem: &[u8]) -> Vec<Vec<Color>> {
    ram_to_dots(mem)
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|dot| if dot { Color::White } else { Color::Black })
                .collect()
        })
        .collect()
}

pub fn ram_to_dots_color(mem: &[u8]) -> Vec<Vec<Color>> {
    let dots = ram_to_dots(mem);
    dots.into_iter()
        .map(|row| {
            row.chunks(2)
                .flat_map(|pair| color_mapping(pair.try_into().unwrap()))
                .collect()
        })
        .collect()
}

// todo: handle flag bit
fn color_mapping(dots: [bool; 2]) -> [Color; 2] {
    let c = match dots {
        [false, false] => Color::Black,
        [true, true] => Color::White,
        [true, false] => Color::Purple,
        [false, true] => Color::Green,
    };
    [c, c]
}

fn ram_to_dots(mem: &[u8]) -> Vec<Vec<bool>> {
    assert_eq!(mem.len(), 2_usize.pow(16)); // 64 KiB
    let page1 = &mem[0x2000..0x4000];

    let sheets = split_sheets(page1);
    let sheets = sheets.map(gr::unscramble_bytes);
    let sheets = sheets.map(sheet_to_dots);
    weave_sheets(sheets)
}

fn split_sheets(page: &[u8]) -> [&[u8]; 8] {
    assert_eq!(page.len(), 0x2000);
    let mut out = [Default::default(); 8];
    for i in 0..8 {
        out[i] = &page[i * 0x400..][..0x400];
    }
    out
}

fn sheet_to_dots(sheet: Vec<Vec<u8>>) -> Vec<Vec<bool>> {
    sheet.into_iter().map(line_of_dots).collect()
}

fn line_of_dots(row: Vec<u8>) -> Vec<bool> {
    row.into_iter().flat_map(byte_to_dots).collect()
}

fn byte_to_dots(byte: u8) -> [bool; 7] {
    let mut out = [false; 7];
    for i in 0..7 {
        let is_set = byte & 1 << i != 0;
        out[i] = is_set;
    }
    out
}

fn weave_sheets(sheets: [Vec<Vec<bool>>; 8]) -> Vec<Vec<bool>> {
    // Each sheet has the same number of rows.
    let num_rows = sheets[0].len();
    debug_assert!(sheets.iter().all(|sheet| sheet.len() == num_rows));

    let mut sheets = sheets.map(IntoIterator::into_iter);

    let mut out = vec![];
    for _ in 0..num_rows {
        for i in 0..8 {
            let row = sheets[i].next().unwrap();
            out.push(row);
        }
    }

    // All done!
    debug_assert!(sheets.iter_mut().all(|sheet| sheet.next().is_none()));

    out
}

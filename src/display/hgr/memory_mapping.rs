use crate::display::gr;

// Note: we could make this a wrapper around a u8, if it ends up mattering for
// performance.
#[derive(Debug, Clone, Copy)]
pub struct Byte {
    /// Selects the "color palette".
    pub flag_bit: bool,
    /// Left-to-right.
    pub bits: [bool; 7],
}

pub fn unscramble(mem: &[u8]) -> Vec<Vec<Byte>> {
    assert_eq!(mem.len(), 2_usize.pow(16)); // 64 KiB
    let page1 = &mem[0x2000..0x4000];

    let sheets = split_sheets(page1);
    let sheets = sheets.map(gr::unscramble_bytes);
    let sheets = sheets.map(sheet_u8_to_byte);
    weave_sheets(sheets)
}

fn split_sheets(page: &[u8]) -> [&[u8]; 8] {
    assert_eq!(page.len(), 0x2000);
    let mut out = [[].as_slice(); 8];
    for i in 0..8 {
        out[i] = &page[i * 0x400..][..0x400];
    }
    out
}

fn sheet_u8_to_byte(sheet: Vec<Vec<u8>>) -> Vec<Vec<Byte>> {
    sheet
        .into_iter()
        .map(|row| row.into_iter().map(Byte::new).collect())
        .collect()
}

impl Byte {
    fn new(byte: u8) -> Self {
        let mut bits = [false; 7];
        for i in 0..7 {
            let is_set = byte & 1 << i != 0;
            bits[i] = is_set;
        }
        let flag_bit = byte & 0x80 != 0;
        Self { flag_bit, bits }
    }
}

fn weave_sheets(sheets: [Vec<Vec<Byte>>; 8]) -> Vec<Vec<Byte>> {
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

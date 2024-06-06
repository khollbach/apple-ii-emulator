use crate::display::gr;

pub fn ram_to_dots(mem: &[u8]) -> Vec<Vec<bool>> {
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

pub const GR_W: usize = 0x28; // 40
pub const GR_H: usize = 0x18; // 24

pub fn gr(mem: &[u8]) -> Vec<Vec<bool>> {
    assert_eq!(mem.len(), 2_usize.pow(16)); // 64 KiB

    let page1 = &mem[0x400..0x800];
    let bytes = unscramble(page1);

    // B&W, for now. todo: colors
    let mut out = vec![vec![false; GR_W]; GR_H];
    for y in 0..GR_H {
        for x in 0..GR_W {
            out[y][x] = bytes[y][x] != 0;
        }
    }
    out
}

fn unscramble(gr_mem: &[u8]) -> Vec<Vec<u8>> {
    assert_eq!(gr_mem.len(), 0x400); // 1 KiB

    let mut out = vec![vec![0u8; GR_W]; GR_H];

    for i in 0..8 {
        for j in 0..3 {
            let row = &gr_mem[i * 0x80 + j * GR_W..][..GR_W];
            out[j * 8 + i].copy_from_slice(row);
        }
    }

    out
}

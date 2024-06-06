mod memory_mapping;

use itertools::Itertools;
use memory_mapping::Byte;

use super::color::Color;

pub const W: usize = 280;
pub const H: usize = 192;

/// Black & white display.
pub fn ram_to_dots_bw(mem: &[u8]) -> Vec<Vec<Color>> {
    memory_mapping::unscramble(mem)
        .into_iter()
        .map(|row| row.into_iter().flat_map(|byte| byte.bits.map(bw)).collect())
        .collect()
}

fn bw(bit: bool) -> Color {
    if bit {
        Color::White
    } else {
        Color::Black
    }
}

pub fn ram_to_dots_color(mem: &[u8]) -> Vec<Vec<Color>> {
    memory_mapping::unscramble(mem)
        .into_iter()
        .map(|row| {
            row.chunks(2)
                .flat_map(|word| color_dots(word.try_into().unwrap()))
                .collect()
        })
        .collect()
}

fn color_dots(word: [Byte; 2]) -> [Color; 14] {
    let [lo, hi] = word;

    // We consider each adjacent pair of dots as a single colored blob. This
    // isn't technically correct, but it's a good-enough simplification. (And
    // it's similar to the way colors are described in the //e Technical
    // Reference Manual, for example.)

    let first_3_blobs = lo
        .bits
        .chunks_exact(2)
        .map(|dots| (dots.try_into().unwrap(), lo.flag_bit));

    let middle_blob = {
        let bits = [lo.bits[6], hi.bits[0]];
        let flag = match bits {
            [true, false] => lo.flag_bit,
            [false, true] => hi.flag_bit,
            _ => lo.flag_bit,
        };
        (bits, flag)
    };

    let last_3_blobs = hi.bits[1..]
        .chunks_exact(2)
        .map(|dots| (dots.try_into().unwrap(), hi.flag_bit));

    let all_blobs = first_3_blobs.chain([middle_blob]).chain(last_3_blobs);

    all_blobs
        .flat_map(color_mapping)
        .collect_vec()
        .try_into()
        .unwrap()
}

fn color_mapping((bits, flag): ([bool; 2], bool)) -> [Color; 2] {
    let c = match (bits, flag) {
        ([false, false], _) => Color::Black,
        ([true, true], _) => Color::White,
        ([true, false], false) => Color::Purple,
        ([false, true], false) => Color::Green,
        ([true, false], true) => Color::MediumBlue,
        ([false, true], true) => Color::Orange,
    };
    [c, c]
}

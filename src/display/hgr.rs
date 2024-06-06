mod memory_mapping;

use super::color::Color;

pub const W: usize = 280;
pub const H: usize = 192;

/// Black & white display.
pub fn ram_to_dots_bw(mem: &[u8]) -> Vec<Vec<Color>> {
    memory_mapping::ram_to_dots(mem)
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|dot| if dot { Color::White } else { Color::Black })
                .collect()
        })
        .collect()
}

pub fn ram_to_dots_color(mem: &[u8]) -> Vec<Vec<Color>> {
    memory_mapping::ram_to_dots(mem)
        .into_iter()
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

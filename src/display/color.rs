mod math;

use std::{f64::consts::PI, mem};

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
    pub fn from_nibble(b: u8) -> Self {
        assert!(b < 0x10);
        unsafe { mem::transmute(b) }
    }

    pub fn rgb(self) -> [u8; 3] {
        let yuv = self.yuv();
        math::yuv_to_rgb(yuv)
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

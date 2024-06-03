//! The implementation details are taken from this wikipedia article:
//! https://en.wikipedia.org/wiki/Y%E2%80%B2UV
//!
//! I didn't actually read & understand the article. In particular, it sounds
//! like there are different possible meanings of "YUV". Hopefully what we've
//! got here is appropriate (or at least good enough) for our purposes.

type Vec3 = [f64; 3];
type Mat3x3 = [Vec3; 3];

pub fn yuv_to_rgb(yuv: Vec3) -> [u8; 3] {
    let rgb = mat_mul(YUV_TO_RGB, yuv);
    rgb.map(f64_to_u8)
}

/// Maps a number in the range [0, 1] to the range `0..=255`.
fn f64_to_u8(x: f64) -> u8 {
    assert!(!x.is_nan());
    let scaled = x.clamp(0., 1.) * 255.;
    debug_assert!(0. <= scaled && scaled <= 255.);
    scaled.round() as u8
}

const _RGB_TO_YUV: Mat3x3 = [
    [0.299, 0.587, 0.114],
    [-0.14713, -0.28886, 0.436],
    [0.615, -0.51499, -0.10001],
];

const YUV_TO_RGB: Mat3x3 = [
    [1., 0., 1.13983],
    [1., -0.39465, -0.58060],
    [1., 2.03211, 0.],
];

fn mat_mul(m: Mat3x3, x: Vec3) -> Vec3 {
    let mut out = [0.; 3];
    for i in 0..3 {
        out[i] = dot(m[i], x);
    }
    out
}

fn dot(x: Vec3, y: Vec3) -> f64 {
    let mut out = 0.;
    for i in 0..3 {
        out += x[i] * y[i];
    }
    out
}

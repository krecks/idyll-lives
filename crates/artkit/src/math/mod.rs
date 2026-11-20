//! Numbers as the family scripts had them. A Python float is an `f64` and its `math` functions are the ones
//! here (computed in software, the same on every machine); a `mathutils.Vector` is a `V3`, single precision
//! with `mathutils`' own rounding (a scalar is cast down before it multiplies, a dot product sums in double),
//! as are `Quat` and the matrices. A build repeats the scripts' arithmetic bit for bit where that decides where
//! a vertex or a perch ends up.

pub(crate) use std::ops::{
    Add, AddAssign, Div, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

mod euler;
mod invert;
mod matrix;
mod mul;
mod num;
mod object;
mod quat;
mod rotations;
mod scalars;
mod sines;
mod vector;

pub use euler::*;
pub use invert::*;
pub use matrix::*;
pub(crate) use mul::*;
pub use num::*;
pub use object::*;
pub use quat::*;
pub(crate) use rotations::*;
pub use scalars::*;
pub(crate) use sines::*;
pub use vector::*;

#[cfg(test)]
mod tests {
    use super::asinf;

    #[test]
    fn the_arc_sine_is_the_c_librarys_where_it_isnt_the_nearest_single() {
        // (number, its arc sine) as bits, from the library: twelve the nearest single isn't, then the ends
        let pairs: [(u32, u32); 17] = [
            (0x3ed2_bd22, 0x3ed9_31ce),
            (0x3f19_1eec, 0x3f24_2347),
            (0xbf0c_511d, 0xbf14_81d5),
            (0x3f07_91bc, 0x3f0e_df06),
            (0xbf00_1edd, 0xbf06_2e37),
            (0x3f43_6e01, 0x3f5e_59b6),
            (0x3f62_a526, 0x3f8b_29a7),
            (0x3f3d_93df, 0x3f55_7880),
            (0x3f1e_bde4, 0x3f2b_39d3),
            (0x3f24_8edd, 0x3f32_ba3e),
            (0x3f32_6871, 0x3f45_6521),
            (0x3f70_fc51, 0x3f9d_0187),
            (0x3f80_0000, 0x3fc9_0fda),
            (0xbf80_0000, 0xbfc9_0fda),
            (0x3f00_0000, 0x3f06_0a92),
            (0x3100_0000, 0x3100_0000),
            (0x0000_0000, 0x0000_0000),
        ];
        for (x, want) in pairs {
            assert_eq!(asinf(f32::from_bits(x)).to_bits(), want, "asinf of {x:08x}");
        }
    }
}

#[cfg(test)]
mod sines_tests {
    use super::{cosf_alone, sincosf, sinf_alone};

    #[test]
    fn sines_and_cosines_are_the_c_librarys_where_they_arent_the_nearest_singles() {
        // (angle, sine, cosine) as bits, from the library's routine for the two at once: six where one of them isn't
        // the nearest single, then the ends
        let both: [(u32, u32, u32); 18] = [
            (0x3e55_95a1, 0x3e54_0a08, 0x3f7a_7397),
            (0xbf3c_b831, 0xbf2c_15c0, 0x3f3d_8874),
            (0x4050_3362, 0xbde3_f7ba, 0xbf7e_68b9),
            (0x41d9_7ed4, 0x3f62_ab2a, 0xbeed_f81f),
            (0xc208_9987, 0xbeca_f4af, 0xbf6b_0735),
            (0x42ef_216c, 0x3e3c_1dad, 0x3f7b_a4a6),
            (0x0000_0000, 0x0000_0000, 0x3f80_0000),
            (0x8000_0000, 0x8000_0000, 0x3f80_0000),
            (0x3727_c5ac, 0x3727_c5ac, 0x3f80_0000),
            (0xb951_b717, 0xb951_b717, 0x3f80_0000),
            (0x3980_0000, 0x3980_0000, 0x3f7f_ffff),
            (0x3f00_0000, 0x3ef5_7744, 0x3f60_a940),
            (0x3fc9_0fdb, 0x3f80_0000, 0xb33b_bd2e),
            (0x4049_0fdb, 0xb3bb_bd2e, 0xbf80_0000),
            (0xc096_cbe4, 0x3f80_0000, 0x324c_de2e),
            (0x40c9_0fdb, 0x343b_bd2e, 0x3f80_0000),
            (0x42c8_0000, 0xbf01_a12e, 0x3f5c_c0ee),
            (0x42ef_ffff, 0x3f14_a287, 0x3f50_6e74),
        ];
        for (x, sin, cos) in both {
            let (s, c) = sincosf(f32::from_bits(x));
            assert_eq!((s.to_bits(), c.to_bits()), (sin, cos), "of {x:08x}");
        }
        // and from its routines for one alone
        let alone: [(u32, u32, u32); 16] = [
            (0x3e55_95a1, 0x3e54_0a08, 0x3f7a_7397),
            (0xbf3c_b831, 0xbf2c_15c0, 0x3f3d_8874),
            (0x4050_3362, 0xbde3_f7ba, 0xbf7e_68ba),
            (0x41d9_7ed4, 0x3f62_ab2a, 0xbeed_f81f),
            (0xc208_9987, 0xbeca_f4af, 0xbf6b_0735),
            (0x42ef_216c, 0x3e3c_1dad, 0x3f7b_a4a6),
            (0x0000_0000, 0x0000_0000, 0x3f80_0000),
            (0x8000_0000, 0x8000_0000, 0x3f80_0000),
            (0x3727_c5ac, 0x3727_c5ac, 0x3f80_0000),
            (0x3980_0000, 0x3980_0000, 0x3f80_0000),
            (0x3f00_0000, 0x3ef5_7744, 0x3f60_a940),
            (0x3f49_0fdb, 0x3f35_04f3, 0x3f35_04f3),
            (0x3fc9_0fdb, 0x3f80_0000, 0xb33b_bd2e),
            (0x4049_0fdb, 0xb3bb_bd2e, 0xbf80_0000),
            (0xc096_cbe4, 0x3f80_0000, 0x324c_de2e),
            (0x42ef_ffff, 0x3f14_a287, 0x3f50_6e74),
        ];
        for (x, sin, cos) in alone {
            let x = f32::from_bits(x);
            assert_eq!(
                (sinf_alone(x).to_bits(), cosf_alone(x).to_bits()),
                (sin, cos),
                "of {x:?} alone"
            );
        }
    }
}

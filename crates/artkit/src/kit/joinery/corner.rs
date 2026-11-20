//! A point of a path to bend, and the shapes one may be.

use crate::math::{Num, P3, V3};

/// A point of a path to bend (`fillet`): a tuple of three numbers or a vector, or a tuple of two for a path drawn
/// flat (an outline, a lathe's profile). The bent path's points are tuples of as many floats.
pub trait Corner: Copy {
    type Out;
    /// As a `mathutils.Vector` (one of two with no third part: its sums come out the same).
    fn vector(self) -> V3;
    /// `tuple(vector)`.
    fn tuple(v: V3) -> Self::Out;
}
impl<A: Num, B: Num, C: Num> Corner for (A, B, C) {
    type Out = (f64, f64, f64);
    fn vector(self) -> V3 {
        self.into()
    }
    fn tuple(v: V3) -> Self::Out {
        (v.x as f64, v.y as f64, v.z as f64)
    }
}
impl<A: Num, B: Num> Corner for (A, B) {
    type Out = (f64, f64);
    fn vector(self) -> V3 {
        (self.0, self.1, 0.0).into()
    }
    fn tuple(v: V3) -> Self::Out {
        (v.x as f64, v.y as f64)
    }
}
impl Corner for V3 {
    type Out = (f64, f64, f64);
    fn vector(self) -> V3 {
        self
    }
    fn tuple(v: V3) -> Self::Out {
        (v.x as f64, v.y as f64, v.z as f64)
    }
}
impl Corner for P3 {
    type Out = (f64, f64, f64);
    fn vector(self) -> V3 {
        self.into()
    }
    fn tuple(v: V3) -> Self::Out {
        (v.x as f64, v.y as f64, v.z as f64)
    }
}

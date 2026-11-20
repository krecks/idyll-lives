//! The numbers a point or a vector may be made of.
//!
//! Split from `crates/artkit/src/math.rs` (see `mod.rs`).

/// A number a script wrote, whole or not: `v3(0, 0.5, 1)`.
pub trait Num: Copy {
    fn f(self) -> f64;
}
impl Num for f64 {
    fn f(self) -> f64 {
        self
    }
}
impl Num for f32 {
    fn f(self) -> f64 {
        self as f64
    }
}
impl Num for i32 {
    fn f(self) -> f64 {
        self as f64
    }
}
impl Num for i64 {
    fn f(self) -> f64 {
        self as f64
    }
}
impl Num for usize {
    fn f(self) -> f64 {
        self as f64
    }
}

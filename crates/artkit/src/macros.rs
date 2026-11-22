//! The builders called as the family scripts called them: a function's options are a struct with defaults, and a
//! macro a function sets the ones named, so `sweep!("trunk", &pts, 0.3, material = P.bark, verts = low(11, 5))`
//! reads as the Python did, and a misspelt option is a field the struct hasn't.

/// `shapes::cube` with its options by name, as the family scripts wrote them: `cube!(name, size, key = value, ...)`.
#[macro_export]
macro_rules! cube {
    ($name:expr, $size:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($name, $size);
        #[allow(unused_mut)]
        let mut o = $crate::shapes::Cube { bevel: $crate::shapes::AUTO_BEVEL, ..Default::default() };
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::shapes::cube(args.0, args.1, o)
    }};
}

/// `shapes::sphere` with its options by name, as the family scripts wrote them: `sphere!(name, radius, key = value, ...)`.
#[macro_export]
macro_rules! sphere {
    ($name:expr, $radius:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($name, $radius);
        #[allow(unused_mut)]
        let mut o = $crate::shapes::Sphere::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::shapes::sphere(args.0, args.1, o)
    }};
}

/// `shapes::cylinder` with its options by name, as the family scripts wrote them: `cylinder!(name, radius, depth, key = value, ...)`.
#[macro_export]
macro_rules! cylinder {
    ($name:expr, $radius:expr, $depth:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($name, $radius, $depth);
        #[allow(unused_mut)]
        let mut o = $crate::shapes::Round::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::shapes::cylinder(args.0, args.1, args.2, o)
    }};
}

/// `shapes::cone` with its options by name, as the family scripts wrote them: `cone!(name, r1, r2, depth, key = value, ...)`.
#[macro_export]
macro_rules! cone {
    ($name:expr, $r1:expr, $r2:expr, $depth:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($name, $r1, $r2, $depth);
        #[allow(unused_mut)]
        let mut o = $crate::shapes::Round::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::shapes::cone(args.0, args.1, args.2, args.3, o)
    }};
}

/// `shapes::rod` with its options by name, as the family scripts wrote them: `rod!(name, a, b, radius, key = value, ...)`.
#[macro_export]
macro_rules! rod {
    ($name:expr, $a:expr, $b:expr, $radius:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($name, $a, $b, $radius);
        #[allow(unused_mut)]
        let mut o = $crate::shapes::Rod::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::shapes::rod(args.0, args.1, args.2, args.3, o)
    }};
}

/// `shapes::revolve` with its options by name, as the family scripts wrote them: `revolve!(name, profile, key = value, ...)`.
#[macro_export]
macro_rules! revolve {
    ($name:expr, $profile:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($name, $profile);
        #[allow(unused_mut)]
        let mut o = $crate::shapes::Revolve::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::shapes::revolve(args.0, args.1, o)
    }};
}

/// `shapes::boulder` with its options by name, as the family scripts wrote them: `boulder!(name, radius, key = value, ...)`.
#[macro_export]
macro_rules! boulder {
    ($name:expr, $radius:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($name, $radius);
        #[allow(unused_mut)]
        let mut o = $crate::shapes::Boulder::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::shapes::boulder(args.0, args.1, o)
    }};
}

/// `forms::sweep` with its options by name, as the family scripts wrote them: `sweep!(name, points, radius, key = value, ...)`.
#[macro_export]
macro_rules! sweep {
    ($name:expr, $points:expr, $radius:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($name, $points, $radius);
        #[allow(unused_mut)]
        let mut o = $crate::forms::Sweep::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::forms::sweep(args.0, args.1, args.2, o)
    }};
}

/// `forms::ring` with its options by name, as the family scripts wrote them: `ring!(rx, ry, key = value, ...)`.
#[macro_export]
macro_rules! ring {
    ($rx:expr, $ry:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($rx, $ry);
        #[allow(unused_mut)]
        let mut o = $crate::forms::Ring::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::forms::ring(args.0, args.1, o)
    }};
}

/// `forms::loft` with its options by name, as the family scripts wrote them: `loft!(name, sections, key = value, ...)`.
#[macro_export]
macro_rules! loft {
    ($name:expr, $sections:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($name, $sections);
        #[allow(unused_mut)]
        let mut o = $crate::forms::Loft::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::forms::loft(args.0, args.1, o)
    }};
}

/// `forms::jitter` with its options by name, as the family scripts wrote them: `jitter!(obj, amount, key = value, ...)`.
#[macro_export]
macro_rules! jitter {
    ($obj:expr, $amount:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($obj, $amount);
        #[allow(unused_mut)]
        let mut o = $crate::forms::Jitter::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::forms::jitter(args.0, args.1, o)
    }};
}

/// `perch::find` with its options by name, as the family scripts wrote them: `find!(n, aims, key = value, ...)`.
#[macro_export]
macro_rules! find {
    ($n:expr, $aims:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($n, $aims);
        #[allow(unused_mut)]
        let mut o = $crate::perch::Find::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::perch::find(args.0, args.1, o)
    }};
}

/// `forms::object_of` with its options by name, as the packs' scripts called their `_obj`: `object_of!(name, verts, faces, mats, face_mats = .., sharp = 50, bevel = 0.004)`.
#[macro_export]
macro_rules! object_of {
    ($name:expr, $verts:expr, $faces:expr, $mats:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($name, $verts, $faces, $mats);
        #[allow(unused_mut)]
        let mut o = $crate::forms::ObjectOf::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::forms::object_of(args.0, args.1, args.2, args.3, o)
    }};
}

/// A value given for an option: itself, or what a script would have written for it (a whole number for a count
/// or a length, a tuple for a point, a bare value where there may be none).
pub trait Arg<T> {
    fn arg(self) -> T;
}

impl<T> Arg<T> for T {
    fn arg(self) -> T {
        self
    }
}
impl<T> Arg<Option<T>> for T {
    fn arg(self) -> Option<T> {
        Some(self)
    }
}
impl Arg<usize> for i32 {
    fn arg(self) -> usize {
        usize::try_from(self).expect("a count isn't negative")
    }
}
impl Arg<Option<usize>> for i32 {
    fn arg(self) -> Option<usize> {
        Some(Arg::<usize>::arg(self))
    }
}
impl Arg<f64> for i32 {
    fn arg(self) -> f64 {
        self as f64
    }
}
impl Arg<Option<f64>> for i32 {
    fn arg(self) -> Option<f64> {
        Some(self as f64)
    }
}
impl<A: crate::math::Num, B: crate::math::Num, C: crate::math::Num> Arg<crate::math::P3>
    for (A, B, C)
{
    fn arg(self) -> crate::math::P3 {
        self.into()
    }
}
impl Arg<crate::math::P3> for crate::math::V3 {
    fn arg(self) -> crate::math::P3 {
        self.into()
    }
}
impl Arg<crate::math::P3> for [f64; 3] {
    fn arg(self) -> crate::math::P3 {
        self.into()
    }
}
impl Arg<crate::forms::Scale> for f64 {
    fn arg(self) -> crate::forms::Scale {
        crate::forms::Scale::One(self)
    }
}
impl Arg<crate::forms::Scale> for i32 {
    fn arg(self) -> crate::forms::Scale {
        crate::forms::Scale::One(self as f64)
    }
}
impl<const N: usize> Arg<crate::forms::Scale> for [f64; N] {
    fn arg(self) -> crate::forms::Scale {
        crate::forms::Scale::List(self.to_vec())
    }
}
impl Arg<crate::forms::Scale> for Vec<f64> {
    fn arg(self) -> crate::forms::Scale {
        crate::forms::Scale::List(self)
    }
}
impl Arg<crate::forms::Scale> for &[f64] {
    fn arg(self) -> crate::forms::Scale {
        crate::forms::Scale::List(self.to_vec())
    }
}
impl Arg<crate::forms::Profile> for Vec<(f64, f64)> {
    fn arg(self) -> crate::forms::Profile {
        crate::forms::Profile(Some(self))
    }
}
impl Arg<crate::forms::Profile> for &Vec<(f64, f64)> {
    fn arg(self) -> crate::forms::Profile {
        crate::forms::Profile(Some(self.clone()))
    }
}
impl Arg<crate::forms::Profile> for &[(f64, f64)] {
    fn arg(self) -> crate::forms::Profile {
        crate::forms::Profile(Some(self.to_vec()))
    }
}
impl<const N: usize> Arg<crate::forms::Profile> for [(f64, f64); N] {
    fn arg(self) -> crate::forms::Profile {
        crate::forms::Profile(Some(self.to_vec()))
    }
}
impl<T: Clone> Arg<Vec<T>> for &[T] {
    fn arg(self) -> Vec<T> {
        self.to_vec()
    }
}
impl<T: Clone> Arg<Vec<T>> for &Vec<T> {
    fn arg(self) -> Vec<T> {
        self.clone()
    }
}
impl<T: Clone> Arg<Option<Vec<T>>> for &[T] {
    fn arg(self) -> Option<Vec<T>> {
        Some(self.to_vec())
    }
}
impl<T: Clone> Arg<Option<Vec<T>>> for &Vec<T> {
    fn arg(self) -> Option<Vec<T>> {
        Some(self.clone())
    }
}

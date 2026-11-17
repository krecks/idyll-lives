//! The botany's builders called as the family scripts called them, their options by name (as the kernel's own:
//! `crate::macros`).

/// `botany::_mesh` with its options by name, as the family scripts wrote them: `_mesh!(name, verts, faces, material, key = value, ...)`.
#[macro_export]
macro_rules! _mesh {
    ($name:expr, $verts:expr, $faces:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::botany::MeshOf::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::botany::_mesh($name, $verts, $faces, $material, o)
    }};
}

/// `botany::leaf` with its options by name, as the family scripts wrote them: `leaf!(name, base, yaw, tilt, length, width, material, key = value, ...)`.
#[macro_export]
macro_rules! leaf {
    ($name:expr, $base:expr, $yaw:expr, $tilt:expr, $length:expr, $width:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::botany::Leaf::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::botany::leaf($name, $base, $yaw, $tilt, $length, $width, $material, o)
    }};
}

/// `botany::tube` with its options by name, as the family scripts wrote them: `tube!(name, points, r0, r1, material, key = value, ...)`.
#[macro_export]
macro_rules! tube {
    ($name:expr, $points:expr, $r0:expr, $r1:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::botany::Tube::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::botany::tube($name, $points, $r0, $r1, $material, o)
    }};
}

/// `botany::sprigs` with its options by name, as the family scripts wrote them: `sprigs!(name, palette, centre, spread, key = value, ...)`.
#[macro_export]
macro_rules! sprigs {
    ($name:expr, $palette:expr, $centre:expr, $spread:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::botany::Sprigs::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::botany::sprigs($name, $palette, $centre, $spread, o)
    }};
}

/// `botany::_with_part` with its options by name, as the family scripts wrote them: `_with_part!(name, still, build, origin, clip, key = value, ...)`.
#[macro_export]
macro_rules! _with_part {
    ($name:expr, $still:expr, $build:expr, $origin:expr, $clip:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::botany::WithPart::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::botany::_with_part($name, $still, $build, $origin, $clip, o)
    }};
}

/// `botany::pot` with its options by name, as the family scripts wrote them: `pot!(palette, r_top, h, key = value, ...)`.
#[macro_export]
macro_rules! pot {
    ($palette:expr, $r_top:expr, $h:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::botany::Pot::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::botany::pot($palette, $r_top, $h, o)
    }};
}

/// `botany::_put` with its options by name, as the family scripts wrote them: `_put!(obj, at, key = value, ...)`.
#[macro_export]
macro_rules! _put {
    ($obj:expr, $at:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::botany::Put::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::botany::_put($obj, $at, o)
    }};
}

/// `botany::cupped` with its options by name, as the family scripts wrote them: `cupped!(name, profile, at, material, key = value, ...)`.
#[macro_export]
macro_rules! cupped {
    ($name:expr, $profile:expr, $at:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::botany::Cupped::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::botany::cupped($name, $profile, $at, $material, o)
    }};
}

/// `botany::lump` with its options by name, as the family scripts wrote them: `lump!(name, r, at, material, key = value, ...)`.
#[macro_export]
macro_rules! lump {
    ($name:expr, $r:expr, $at:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::botany::Lump::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::botany::lump($name, $r, $at, $material, o)
    }};
}

/// `botany::leaves` with its options by name, as the family scripts wrote them: `leaves!(name, r, at, material, key = value, ...)`.
#[macro_export]
macro_rules! leaves {
    ($name:expr, $r:expr, $at:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::botany::Leaves::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::botany::leaves($name, $r, $at, $material, o)
    }};
}

/// `Cards::over` with its options by name, as the family scripts wrote them: `over!(cards, centre, half, top, n, size, material, crown, shade, key = value, ...)`.
#[macro_export]
macro_rules! over {
    ($cards:expr, $centre:expr, $half:expr, $top:expr, $n:expr, $size:expr, $material:expr, $crown:expr, $shade:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::botany::Over::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $cards.over($centre, $half, $top, $n, $size, $material, $crown, $shade, o)
    }};
}

/// `botany::clad` with its options by name, as the family scripts wrote them: `clad!(palette, still, seed, more = |cards: &mut Cards| ..., tiles = ...)`.
#[macro_export]
macro_rules! clad {
    (@set $o:ident, more, $v:expr) => {
        $o.more = Some(Box::new($v));
    };
    (@set $o:ident, $k:ident, $v:expr) => {
        $o.$k = $crate::Arg::arg($v);
    };
    ($palette:expr, $still:expr, $seed:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        #[allow(unused_mut)]
        let mut o = $crate::botany::Clad::default();
        $($crate::clad!(@set o, $k, $v);)*
        $crate::botany::clad($palette, $still, $seed, o)
    }};
}

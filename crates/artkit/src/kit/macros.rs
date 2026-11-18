//! The kit's builders called as the family scripts called them (`crate::macros`): `kit::pull!(F, "pull", 0, 0.45,
//! 0.02, 0.12, M.brass, r = 0.004)` for the script's `G.kit.pull(F, 'pull', 0, 0.45, 0.02, 0.12, M.brass, r=0.004)`.
//! They are exported at the crate's root under `kit_` names (the plants have a `tube!` of their own) and
//! named as the scripts named them in `kit`.

/// `kit::tube` with its options by name, as the family scripts wrote them: `kit::tube!(frame, name, points, r, material, key = value, ...)`.
#[macro_export]
macro_rules! kit_tube {
    ($frame:expr, $name:expr, $points:expr, $r:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $points, $r, $material);
        #[allow(unused_mut)]
        let mut o = $crate::kit::Tube::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::kit::tube(args.0, args.1, args.2, $crate::forms::num(args.3), args.4, o)
    }};
}

/// `kit::cabriole` with its options by name, as the family scripts wrote them: `kit::cabriole!(frame, name, x, z, h, material, sx, sz, key = value, ...)`.
#[macro_export]
macro_rules! kit_cabriole {
    ($frame:expr, $name:expr, $x:expr, $z:expr, $h:expr, $material:expr, $sx:expr, $sz:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $x, $z, $h, $material, $sx, $sz);
        #[allow(unused_mut)]
        let mut o = $crate::kit::Cabriole::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::kit::cabriole(args.0, args.1, args.2, args.3, $crate::forms::num(args.4), args.5, $crate::forms::num(args.6), $crate::forms::num(args.7), o)
    }};
}

/// `kit::fillet` as the family scripts wrote it: `kit::fillet!(points, radius)`, `kit::fillet!(points, radius, 2)` or
/// `kit::fillet!(points, radius, steps = 2)` (3 steps round a corner where none are given).
#[macro_export]
macro_rules! kit_fillet {
    ($points:expr, $radius:expr $(,)?) => {
        $crate::kit::fillet($points, $crate::forms::num($radius), 3)
    };
    ($points:expr, $radius:expr, steps = $steps:expr $(,)?) => {
        $crate::kit::fillet($points, $crate::forms::num($radius), $steps)
    };
    ($points:expr, $radius:expr, $steps:expr $(,)?) => {
        $crate::kit::fillet($points, $crate::forms::num($radius), $steps)
    };
}

/// `kit::pull` with its options by name, as the family scripts wrote them: `kit::pull!(frame, name, x, y, face, length, material, key = value, ...)`.
#[macro_export]
macro_rules! kit_pull {
    ($frame:expr, $name:expr, $x:expr, $y:expr, $face:expr, $length:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $x, $y, $face, $length, $material);
        #[allow(unused_mut)]
        let mut o = $crate::kit::Pull::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::kit::pull(args.0, args.1, $crate::forms::num(args.2), $crate::forms::num(args.3), $crate::forms::num(args.4), $crate::forms::num(args.5), args.6, o)
    }};
}

/// `kit::knob` with its options by name, as the family scripts wrote them: `kit::knob!(frame, name, x, y, face, r, material, key = value, ...)`.
#[macro_export]
macro_rules! kit_knob {
    ($frame:expr, $name:expr, $x:expr, $y:expr, $face:expr, $r:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $x, $y, $face, $r, $material);
        #[allow(unused_mut)]
        let mut o = $crate::kit::Knob::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::kit::knob(args.0, args.1, args.2, args.3, args.4, $crate::forms::num(args.5), args.6, o)
    }};
}

/// `kit::shaker` with its options by name, as the family scripts wrote them: `kit::shaker!(frame, name, w, h, at, t, material, key = value, ...)`.
#[macro_export]
macro_rules! kit_shaker {
    ($frame:expr, $name:expr, $w:expr, $h:expr, $at:expr, $t:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $w, $h, $at, $t, $material);
        #[allow(unused_mut)]
        let mut o = $crate::kit::Shaker::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::kit::shaker(args.0, args.1, $crate::forms::num(args.2), $crate::forms::num(args.3), args.4, $crate::forms::num(args.5), args.6, o)
    }};
}

/// `kit::moulding` with its options by name, as the family scripts wrote them: `kit::moulding!(frame, name, outline, profile, material, key = value, ...)`;
/// `plane = Facing::Top` for the script's `plane='top'`.
#[macro_export]
macro_rules! kit_moulding {
    ($frame:expr, $name:expr, $outline:expr, $profile:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $outline, $profile, $material);
        #[allow(unused_mut)]
        let mut o = $crate::kit::Moulding::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::kit::moulding(args.0, args.1, args.2, args.3, args.4, o)
    }};
}

/// `kit::songbird` with its options by name, as the family scripts wrote them:
/// `kit::songbird!(frame, name, x, y, z, body, breast, beak, eye, key = value, ...)`, or without `eye` for a bird
/// that has none.
#[macro_export]
macro_rules! kit_songbird {
    ($frame:expr, $name:expr, $x:expr, $y:expr, $z:expr, $body:expr, $breast:expr, $beak:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $x, $y, $z, $body, $breast, $beak);
        #[allow(unused_mut)]
        let mut o = $crate::kit::Songbird::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::kit::songbird(args.0, args.1, $crate::forms::num(args.2), $crate::forms::num(args.3), $crate::forms::num(args.4), args.5, args.6, args.7, None, o)
    }};
    ($frame:expr, $name:expr, $x:expr, $y:expr, $z:expr, $body:expr, $breast:expr, $beak:expr, $eye:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = (&$frame, $name, $x, $y, $z, $body, $breast, $beak, $eye);
        #[allow(unused_mut)]
        let mut o = $crate::kit::Songbird::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::kit::songbird(args.0, args.1, $crate::forms::num(args.2), $crate::forms::num(args.3), $crate::forms::num(args.4), args.5, args.6, args.7, $crate::Arg::arg(args.8), o)
    }};
}

/// `kit::scroll` with its options by name, as the family scripts wrote them: `kit::scroll!(frame, name, start, centre, material, key = value, ...)`.
#[macro_export]
macro_rules! kit_scroll {
    ($frame:expr, $name:expr, $start:expr, $centre:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $start, $centre, $material);
        #[allow(unused_mut)]
        let mut o = $crate::kit::Scroll::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::kit::scroll(args.0, args.1, args.2, args.3, args.4, o)
    }};
}

/// `kit::bench_end` with its options by name, as the family scripts wrote them: `kit::bench_end!(frame, name, x, seat, front, back, top, material, key = value, ...)`.
#[macro_export]
macro_rules! kit_bench_end {
    ($frame:expr, $name:expr, $x:expr, $seat:expr, $front:expr, $back:expr, $top:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $x, $seat, $front, $back, $top, $material);
        #[allow(unused_mut)]
        let mut o = $crate::kit::BenchEnd::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::kit::bench_end(args.0, args.1, args.2, $crate::forms::num(args.3), $crate::forms::num(args.4), $crate::forms::num(args.5), args.6, args.7, o)
    }};
}

/// `kit::taxi_word` with its options by name, as the family scripts wrote them: `kit::taxi_word!(frame, name, centre, h, z, facing, material, key = value, ...)`.
#[macro_export]
macro_rules! kit_taxi_word {
    ($frame:expr, $name:expr, $centre:expr, $h:expr, $z:expr, $facing:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $centre, $h, $z, $facing, $material);
        #[allow(unused_mut)]
        let mut o = $crate::kit::TaxiWord::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::kit::taxi_word(args.0, args.1, args.2, $crate::forms::num(args.3), $crate::forms::num(args.4), args.5, args.6, o)
    }};
}

/// `kit::chequer` with its options by name, as the family scripts wrote them: `kit::chequer!(frame, name, x0, x1, y, z, size, facing, material, key = value, ...)`.
#[macro_export]
macro_rules! kit_chequer {
    ($frame:expr, $name:expr, $x0:expr, $x1:expr, $y:expr, $z:expr, $size:expr, $facing:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = (&$frame, $name, $x0, $x1, $y, $z, $size, $facing, $material);
        #[allow(unused_mut)]
        let mut o = $crate::kit::Chequer::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::kit::chequer(args.0, args.1, $crate::forms::num(args.2), $crate::forms::num(args.3), $crate::forms::num(args.4), $crate::forms::num(args.5), $crate::forms::num(args.6), args.7, args.8, o)
    }};
}

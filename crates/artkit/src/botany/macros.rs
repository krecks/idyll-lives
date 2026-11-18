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

// The macros below work out what a script gave by place before what it gave by name, as Python does: a random
// number drawn for a name comes after one drawn for a place (but for the random numbers' own source, `rng`, which
// is handed on as it is).

/// `botany::cut_leaf` with its options by name, as the family scripts wrote them: `cut_leaf!(name, base, yaw, tilt, length, edge, material, key = value, ...)`.
#[macro_export]
macro_rules! cut_leaf {
    ($name:expr, $base:expr, $yaw:expr, $tilt:expr, $length:expr, $edge:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($name, $base, $yaw, $tilt, $length, $edge, $material) {
            (name, base, yaw, tilt, length, edge, material) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::CutLeaf::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::cut_leaf(name, base, yaw, tilt, length, edge, material, o)
            }
        }
    };
}

/// `botany::outline` with its options by name, as the family scripts wrote them: `outline!(shape, width, key = value, ...)`.
#[macro_export]
macro_rules! outline {
    ($shape:expr, $width:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($shape, $width) {
            (shape, width) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::OutlineOf::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::outline(shape, width, o)
            }
        }
    };
}

/// `botany::pinnate` with its options by name, as the family scripts wrote them: `pinnate!(pairs, width, shape, key = value, ...)`
/// (`shape` left out: "lance").
#[macro_export]
macro_rules! pinnate {
    ($pairs:expr, $width:expr $(, $k:ident = $v:expr)* $(,)?) => {
        $crate::pinnate!($pairs, $width, "lance" $(, $k = $v)*)
    };
    ($pairs:expr, $width:expr, $shape:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($pairs, $width, $shape) {
            (pairs, width, shape) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::Pinnate::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::pinnate(pairs, width, shape, o)
            }
        }
    };
}

/// `botany::slits` with its options by name, as the family scripts wrote them: `slits!(shape, width, n, key = value, ...)`.
#[macro_export]
macro_rules! slits {
    ($shape:expr, $width:expr, $n:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($shape, $width, $n) {
            (shape, width, n) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::Slits::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::slits(shape, width, n, o)
            }
        }
    };
}

/// `botany::lobed` with its options by name, as the family scripts wrote them: `lobed!(lobes, key = value, ...)`.
#[macro_export]
macro_rules! lobed {
    ($lobes:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($lobes,) {
            (lobes,) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::Lobed::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::lobed(lobes, o)
            }
        }
    };
}

/// `botany::rose_leaf` with its options by name, as the family scripts wrote them: `rose_leaf!(name, base, yaw, tilt, size, material, key = value, ...)`.
#[macro_export]
macro_rules! rose_leaf {
    ($name:expr, $base:expr, $yaw:expr, $tilt:expr, $size:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($name, $base, $yaw, $tilt, $size, $material) {
            (name, base, yaw, tilt, size, material) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::RoseLeaf::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::rose_leaf(name, base, yaw, tilt, size, material, o)
            }
        }
    };
}

/// `botany::arc` with its options by name, as the family scripts wrote them: `arc!(a, b, lift, key = value, ...)`.
#[macro_export]
macro_rules! arc {
    ($a:expr, $b:expr, $lift:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($a, $b, $lift) {
            (a, b, lift) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::ArcOf::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::arc(a, b, lift, o)
            }
        }
    };
}

/// `botany::twig` with its options by name, as the family scripts wrote them: `twig!(name, points, r0, r1, material, key = value, ...)`.
#[macro_export]
macro_rules! twig {
    ($name:expr, $points:expr, $r0:expr, $r1:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($name, $points, $r0, $r1, $material) {
            (name, points, r0, r1, material) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::Twig::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::twig(name, points, r0, r1, material, o)
            }
        }
    };
}

/// `botany::stem` with its options by name, as the family scripts wrote them: `stem!(name, a, b, material, key = value, ...)`.
#[macro_export]
macro_rules! stem {
    ($name:expr, $a:expr, $b:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($name, $a, $b, $material) {
            (name, a, b, material) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::Stem::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::stem(name, a, b, material, o)
            }
        }
    };
}

/// `botany::blade` with its options by name, as the family scripts wrote them: `blade!(name, base, yaw, tilt, length, width, material, key = value, ...)`.
#[macro_export]
macro_rules! blade {
    ($name:expr, $base:expr, $yaw:expr, $tilt:expr, $length:expr, $width:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($name, $base, $yaw, $tilt, $length, $width, $material) {
            (name, base, yaw, tilt, length, width, material) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::Blade::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::blade(name, base, yaw, tilt, length, width, material, o)
            }
        }
    };
}

/// `botany::stalk` with its options by name, as the family scripts wrote them: `stalk!(name, a, b, material, key = value, ...)`.
#[macro_export]
macro_rules! stalk {
    ($name:expr, $a:expr, $b:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($name, $a, $b, $material) {
            (name, a, b, material) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::Stalk::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::stalk(name, a, b, material, o)
            }
        }
    };
}

/// `botany::ribbed` with its options by name, as the family scripts wrote them: `ribbed!(r, ribs, key = value, ...)`.
#[macro_export]
macro_rules! ribbed {
    ($r:expr, $ribs:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($r, $ribs) {
            (r, ribs) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::Ribbed::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::ribbed(r, ribs, o)
            }
        }
    };
}

/// `botany::rays` with its options by name, as the family scripts wrote them: `rays!(name, at, r, n, material, key = value, ...)`.
#[macro_export]
macro_rules! rays {
    ($name:expr, $at:expr, $r:expr, $n:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($name, $at, $r, $n, $material) {
            (name, at, r, n, material) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::Rays::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::rays(name, at, r, n, material, o)
            }
        }
    };
}

/// `botany::dome` with its options by name, as the family scripts wrote them: `dome!(name, at, r, h, material, key = value, ...)`.
#[macro_export]
macro_rules! dome {
    ($name:expr, $at:expr, $r:expr, $h:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($name, $at, $r, $h, $material) {
            (name, at, r, h, material) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::Dome::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::dome(name, at, r, h, material, o)
            }
        }
    };
}

/// `botany::star` with its options by name, as the family scripts wrote them: `star!(name, at, r, n, material, key = value, ...)`.
#[macro_export]
macro_rules! star {
    ($name:expr, $at:expr, $r:expr, $n:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($name, $at, $r, $n, $material) {
            (name, at, r, n, material) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::Star::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::star(name, at, r, n, material, o)
            }
        }
    };
}

/// `botany::tulip_head` with its options by name, as the family scripts wrote them: `tulip_head!(name, at, r, h, material, key = value, ...)`.
#[macro_export]
macro_rules! tulip_head {
    ($name:expr, $at:expr, $r:expr, $h:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($name, $at, $r, $h, $material) {
            (name, at, r, h, material) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::TulipHead::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::tulip_head(name, at, r, h, material, o)
            }
        }
    };
}

/// `botany::lupin_spike` with its options by name, as the family scripts wrote them: `lupin_spike!(name, at, h, r, material, key = value, ...)`.
#[macro_export]
macro_rules! lupin_spike {
    ($name:expr, $at:expr, $h:expr, $r:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($name, $at, $h, $r, $material) {
            (name, at, h, r, material) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::LupinSpike::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::lupin_spike(name, at, h, r, material, o)
            }
        }
    };
}

/// `botany::florets` with its options by name, as the family scripts wrote them: `florets!(name, r, at, material, key = value, ...)`.
#[macro_export]
macro_rules! florets {
    ($name:expr, $r:expr, $at:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($name, $r, $at, $material) {
            (name, r, at, material) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::Florets::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::florets(name, r, at, material, o)
            }
        }
    };
}

/// `botany::rose_head` with its options by name, as the family scripts wrote them: `rose_head!(name, at, r, material, key = value, ...)`.
#[macro_export]
macro_rules! rose_head {
    ($name:expr, $at:expr, $r:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($name, $at, $r, $material) {
            (name, at, r, material) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::RoseHead::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::rose_head(name, at, r, material, o)
            }
        }
    };
}

/// `botany::vessel` with its options by name, as the family scripts wrote them: `vessel!(palette, r_top, h, key = value, ...)`.
#[macro_export]
macro_rules! vessel {
    ($palette:expr, $r_top:expr, $h:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($palette, $r_top, $h) {
            (palette, r_top, h) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::Vessel::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::vessel(palette, r_top, h, o)
            }
        }
    };
}

/// `botany::soil_top` with its options by name, as the family scripts wrote them: `soil_top!(palette, r, z, key = value, ...)`,
/// the palette `None` where the script gave none (and a `material`).
#[macro_export]
macro_rules! soil_top {
    (None, $r:expr, $z:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($r, $z) {
            (r, z) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::SoilTop::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::soil_top(None, r, z, o)
            }
        }
    };
    ($palette:expr, $r:expr, $z:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($palette, $r, $z) {
            (palette, r, z) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::SoilTop::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::soil_top(Some(palette), r, z, o)
            }
        }
    };
}

/// `botany::leafy_shrub` with its options by name, as the family scripts wrote them: `leafy_shrub!(palette, seed, w, h, cols, wood, key = value, ..., blooms = |masses, rng| ...)`.
#[macro_export]
macro_rules! leafy_shrub {
    (@set $o:ident, blooms, $v:expr) => {
        $o.blooms = Some(Box::new($v));
    };
    (@set $o:ident, $k:ident, $v:expr) => {
        $o.$k = $crate::Arg::arg($v);
    };
    ($palette:expr, $seed:expr, $w:expr, $h:expr, $cols:expr, $wood:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($palette, $seed, $w, $h, $cols, $wood) {
            (palette, seed, w, h, cols, wood) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::LeafyShrub::default();
                $($crate::leafy_shrub!(@set o, $k, $v);)*
                $crate::botany::leafy_shrub(palette, seed, w, h, cols, wood, o)
            }
        }
    };
}

/// `botany::on_masses` with its options by name, as the family scripts wrote them: `on_masses!(masses, rng, n, key = value, ...)`.
#[macro_export]
macro_rules! on_masses {
    ($masses:expr, $rng:expr, $n:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($masses, $n) {
            (masses, n) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::OnMasses::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::on_masses(masses, $rng, n, o)
            }
        }
    };
}

/// `botany::dots` with its options by name, as the family scripts wrote them: `dots!(rng, n, centre, radii, size, materials, name, key = value, ...)`.
#[macro_export]
macro_rules! dots {
    ($rng:expr, $n:expr, $centre:expr, $radii:expr, $size:expr, $materials:expr, $name:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($n, $centre, $radii, $size, $materials, $name) {
            (n, centre, radii, size, materials, name) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::Dots::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::dots($rng, n, centre, radii, size, materials, name, o)
            }
        }
    };
}

/// `botany::perches` with its options by name, as the family scripts wrote them: `perches!(n, seed, centres, key = value, ...)`.
#[macro_export]
macro_rules! perches {
    ($n:expr, $seed:expr, $centres:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($n, $seed, $centres) {
            (n, seed, centres) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::Perches::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::perches(n, seed, centres, o)
            }
        }
    };
}

/// `botany::hedge_block` with its options by name, as the family scripts wrote them: `hedge_block!(palette, length, h, depth, key = value, ...)`.
#[macro_export]
macro_rules! hedge_block {
    ($palette:expr, $length:expr, $h:expr, $depth:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($palette, $length, $h, $depth) {
            (palette, length, h, depth) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::HedgeBlock::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::hedge_block(palette, length, h, depth, o)
            }
        }
    };
}

/// `botany::spiral` with its options by name, as the family scripts wrote them: `spiral!(n, radius, rng, key = value, ...)`.
#[macro_export]
macro_rules! spiral {
    ($n:expr, $radius:expr, $rng:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($n, $radius) {
            (n, radius) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::Spiral::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::spiral(n, radius, $rng, o)
            }
        }
    };
}

/// `botany::mound` with its options by name, as the family scripts wrote them: `mound!(palette, key = value, ...)`.
#[macro_export]
macro_rules! mound {
    ($palette:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($palette,) {
            (palette,) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::Mound::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::mound(palette, o)
            }
        }
    };
}

/// `botany::rosettes` with its options by name, as the family scripts wrote them: `rosettes!(palette, rng, n, radius, z, cols, key = value, ...)`.
#[macro_export]
macro_rules! rosettes {
    ($palette:expr, $rng:expr, $n:expr, $radius:expr, $z:expr, $cols:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($palette, $n, $radius, $z, $cols) {
            (palette, n, radius, z, cols) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::Rosettes::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::rosettes(palette, $rng, n, radius, z, cols, o)
            }
        }
    };
}

/// `botany::_crop` with its options by name, as the family scripts wrote them: `_crop!(still, origin, leaves, crop, key = value, ...)`.
#[macro_export]
macro_rules! _crop {
    ($still:expr, $origin:expr, $leaves:expr, $crop:expr $(, $k:ident = $v:expr)* $(,)?) => {
        match ($still, $origin, $leaves, $crop) {
            (still, origin, leaves, crop) => {
                #[allow(unused_mut)]
                let mut o = $crate::botany::Crop::default();
                $(o.$k = $crate::Arg::arg($v);)*
                $crate::botany::_crop(still, origin, leaves, crop, o)
            }
        }
    };
}

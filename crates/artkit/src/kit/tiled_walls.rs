//! A shower's tiled corner: two walls of square tiles on grout.

use crate::fbox;
use crate::materials::Mat;
use crate::place::GameFrame;
use crate::py::round;
use crate::shapes::options;

options! {
    /// `tiled_walls`' options: each tile `thick` (proud of the grout), its edges softened by `bevel`.
    TiledWalls { thick: f64 = 0.006, bevel: f64 = 0.0 }
}

/// The back wall (game x0..x1 at the back z0) and the left wall (at x0, z0..z1), from `y0` up to `y1`: a grout
/// backing `t` thick with square tiles about `size` across, `gap` between them, as many as fit whole.
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn tiled_walls(
    frame: &GameFrame,
    (x0, x1): (f64, f64),
    (z0, z1): (f64, f64),
    (y0, y1): (f64, f64),
    t: f64,
    size: f64,
    gap: f64,
    grout: Mat,
    tile: Mat,
    o: TiledWalls,
) {
    let f = frame;
    fbox!(
        f,
        "grout_back",
        x1 - x0,
        y1 - y0,
        t,
        ((x0 + x1) / 2.0, (y0 + y1) / 2.0, z0 + t / 2.0),
        grout
    );
    fbox!(
        f,
        "grout_left",
        t,
        y1 - y0,
        z1 - z0 - t,
        (x0 + t / 2.0, (y0 + y1) / 2.0, (z0 + t + z1) / 2.0),
        grout
    );
    let rows = 1.max(round((y1 - y0) / size));
    let th = (y1 - y0) / rows as f64;
    let cols = 1.max(round((x1 - x0 - t) / size));
    let tw = (x1 - x0 - t) / cols as f64;
    let deep = 1.max(round((z1 - z0 - t) / size));
    let td = (z1 - z0 - t) / deep as f64;
    let k = o.thick;
    for j in 0..rows {
        let y = y0 + th * (j as f64 + 0.5);
        for i in 0..cols {
            fbox!(
                f,
                format!("tile_b{i}_{j}"),
                tw - gap,
                th - gap,
                k,
                (x0 + t + tw * (i as f64 + 0.5), y, z0 + t + k / 2.0),
                tile,
                bevel = o.bevel,
                bevel_segments = 1
            );
        }
        for i in 0..deep {
            fbox!(
                f,
                format!("tile_l{i}_{j}"),
                k,
                th - gap,
                td - gap,
                (x0 + t + k / 2.0, y, z0 + t + td * (i as f64 + 0.5)),
                tile,
                bevel = o.bevel,
                bevel_segments = 1
            );
        }
    }
}

/// `kit::tiled_walls` with its options by name: `kit::tiled_walls!(frame, (x0, x1), (z0, z1), (y0, y1), t, size, gap, grout, tile, thick = 0.008, bevel = 0.002)`.
#[macro_export]
macro_rules! kit_tiled_walls {
    ($frame:expr, $xs:expr, $zs:expr, $ys:expr, $t:expr, $size:expr, $gap:expr, $grout:expr, $tile:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = (&$frame, $xs, $zs, $ys, $t, $size, $gap, $grout, $tile);
        #[allow(unused_mut)]
        let mut o = $crate::kit::TiledWalls::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::kit::tiled_walls(args.0, args.1, args.2, args.3, $crate::forms::num(args.4), $crate::forms::num(args.5), $crate::forms::num(args.6), args.7, args.8, o)
    }};
}

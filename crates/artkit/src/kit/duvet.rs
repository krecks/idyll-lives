//! A bed's duvet whose top third turns down when someone gets in.

use crate::materials::Mat;
use crate::math::Axis;
use crate::parts::{hinged, join, new_parts};
use crate::place::GameFrame;
use crate::scene::Obj;
use crate::shapes::options;
use crate::{soft, squash, swing};

options! {
    /// `duvet`'s options: `height`, its top above the mattress top; `over`, how far it reaches past the bedding's
    /// half width and its foot; `puff`, `round`; `rumple` (how deep its folds are) and `folds` (how far apart);
    /// `seed`; `flat`; `runner` (from, to, in game depth): where a runner lies across it and it is smooth.
    Duvet { height: f64 = 0.05, over: f64 = 0.028, puff: f64 = 0.012, round: f64 = 0.06, rumple: f64 = 0.03, folds: f64 = 0.26, seed: f64 = 0.0, flat: Option<usize> = None, runner: Option<(f64, f64)> = None }
}

/// The duvet's lower two thirds over a mattress whose top is at `top`, `half` its half width: from the fold
/// `z_fold` to the foot `z_foot`, draped `drape` down the sides and the foot end, puffed on top and rumpled, as a
/// duvet shaken out lies: soft folds across it, its sides hanging unevenly, its corners drawn in. The folds only go
/// down and in, so it stays under the game's quilt and inside its box.
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn duvet(
    frame: &GameFrame,
    top: f64,
    half: f64,
    z_fold: f64,
    z_foot: f64,
    drape: f64,
    material: Mat,
    o: Duvet,
) -> Obj {
    let (over, puff) = (o.over, o.puff);
    let (y0, y1) = (top - drape, top + o.height);
    let mid = (z_fold + z_foot + over) / 2.0;
    // (the soft box's own y runs to the back: a game depth z is its y = mid - z)
    let calm = o.runner.map(|r| (mid - r.1, mid - r.0));
    soft!(
        frame,
        "duvet",
        2.0 * (half + over),
        y1 - y0 - 2.0 * puff,
        z_foot + over - z_fold,
        (0, (y0 + y1) / 2.0, mid),
        material,
        round = o.round,
        puff = puff,
        flat = o.flat,
        pinch = 0.0,
        rumple = o.rumple,
        rumple_size = o.folds,
        seed = o.seed + half,
        calm = calm,
        resting = true
    )
}

options! {
    /// `duvet_flap`'s options: `height` (the duvet's, as `Duvet`), `thick` (the flap's thickness at rest), its
    /// `flat`; the sheet turned down over its head edge `band` deep, its middle `band_at` from the head, `band_round`.
    DuvetFlap { height: f64 = 0.05, thick: f64 = 0.045, flat: Option<usize> = None, band: f64 = 0.13, band_at: f64 = 0.055, band_round: f64 = 0.022 }
}

/// The duvet's top third, from `z_top` to the fold `z_fold`, flat on the mattress with the `sheet` turned down over
/// its head edge; one part hinged across its fold on the duvet's top, so turning it 180 degrees (`turn_down`) lays
/// it over the rest.
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn duvet_flap(
    frame: &GameFrame,
    top: f64,
    half: f64,
    z_top: f64,
    z_fold: f64,
    material: Mat,
    sheet: Mat,
    o: DuvetFlap,
) -> Obj {
    let parts = new_parts(|| {
        let y = top + o.height - o.thick / 2.0;
        soft!(
            frame,
            "flap",
            2.0 * (half + 0.012),
            o.thick - 0.01,
            z_fold - z_top,
            (0, y, (z_top + z_fold) / 2.0),
            material,
            round = 0.018,
            puff = 0.005,
            flat = o.flat
        );
        soft!(
            frame,
            "flap_sheet",
            2.0 * (half + 0.016),
            o.thick + 0.008,
            o.band,
            (0, y, z_top + o.band_at),
            sheet,
            round = o.band_round
        );
    });
    // A hair above the duvet's top, so the flap folded over doesn't lie in its plane.
    hinged(
        join(&parts, "duvet_flap", None),
        frame.at(0, top + o.height + 0.003, z_fold),
    )
}

options! {
    /// `turn_down`'s options: `seconds`; `folded`, how much of its thickness the flap keeps folded over; `roll`, how
    /// much of its length it is gathered to half way over.
    TurnDown { seconds: f64 = 1.0, folded: f64 = 0.4, roll: f64 = 0.2 }
}

/// Keys `clip` on a `duvet_flap`: it turns 180 degrees over the rest of the duvet and, as it goes, flattens to
/// `folded` of its thickness (about its hinge, so it stays on the duvet) and narrows to keep 8 cm inside `edge`, the
/// game's quilt's edge from the bed's middle, so the quilt over a sleeper hides it. `half`: the bedding's half
/// width. Half way over it is gathered to `roll` of its length along the bed (it springs out again as it lies
/// down), so it rolls over its fold as cloth does, a low bulge, never a board standing up.
pub fn turn_down(flap: Obj, clip: &str, half: f64, edge: f64, o: TurnDown) {
    let narrow = 1.0_f64.min((edge - 0.08) / (half + 0.016));
    swing!(flap, clip, 180.0, seconds = o.seconds, axis = Axis::X);
    squash!(flap, clip, (narrow, 1, o.folded), seconds = o.seconds);
    squash!(flap, clip, (1, o.roll, 1), seconds = o.seconds, back = true);
}

/// `kit::duvet` with its options by name: `kit::duvet!(frame, top, half, z_fold, z_foot, drape, material, seed = 1)`.
#[macro_export]
macro_rules! kit_duvet {
    ($frame:expr, $top:expr, $half:expr, $z_fold:expr, $z_foot:expr, $drape:expr, $material:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = (&$frame, $top, $half, $z_fold, $z_foot, $drape, $material);
        #[allow(unused_mut)]
        let mut o = $crate::kit::Duvet::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::kit::duvet(args.0, $crate::forms::num(args.1), $crate::forms::num(args.2), $crate::forms::num(args.3), $crate::forms::num(args.4), $crate::forms::num(args.5), args.6, o)
    }};
}

/// `kit::duvet_flap` with its options by name: `kit::duvet_flap!(frame, top, half, z_top, z_fold, material, sheet, flat = 3)`.
#[macro_export]
macro_rules! kit_duvet_flap {
    ($frame:expr, $top:expr, $half:expr, $z_top:expr, $z_fold:expr, $material:expr, $sheet:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = (&$frame, $top, $half, $z_top, $z_fold, $material, $sheet);
        #[allow(unused_mut)]
        let mut o = $crate::kit::DuvetFlap::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::kit::duvet_flap(args.0, $crate::forms::num(args.1), $crate::forms::num(args.2), $crate::forms::num(args.3), $crate::forms::num(args.4), args.5, args.6, o)
    }};
}

/// `kit::turn_down` with its options by name: `kit::turn_down!(flap, clip, half, edge, seconds = 1)`.
#[macro_export]
macro_rules! kit_turn_down {
    ($flap:expr, $clip:expr, $half:expr, $edge:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        let args = ($flap, $clip, $half, $edge);
        #[allow(unused_mut)]
        let mut o = $crate::kit::TurnDown::default();
        $(o.$k = $crate::Arg::arg($v);)*
        $crate::kit::turn_down(args.0, args.1, $crate::forms::num(args.2), $crate::forms::num(args.3), o)
    }};
}

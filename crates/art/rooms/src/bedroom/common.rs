//! What the beds are built from: the palette, the frame, the bedding (mattresses, pillows, the duvet and its
//! flap that turns down), headboard channels, posts, boards and legs.

use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, PI, cos, radians, sin, sqrt};
use artkit::palette;
use artkit::parts::{Settle, hinged, join, measure, new_parts, settle_with};
use artkit::place::{Coord, GameFrame};
use artkit::scene::{self, Obj, meshes};
use artkit::{
    fbox, fcone, fcyl, fextrude, fsphere, fsweep, jitter, loft, ring, soft, squash, swing,
};

palette! { M:
    // shared
    mattress = ("#F7F7F5", fabric),
    sheet = ("#FFFFFF", fabric),       // the sheet turned down over the duvet's top edge
    pillow = ("#F6F4EF", fabric),
    legs_teak = ("#B08E66", wood),
    // the double bed (`bed`) and the king: upholstered in linen
    frame_linen = ("#D9D4CA", fabric),
    headboard_linen = ("#D3CDC1", fabric),
    bedding_cream = ("#F1F0EC", fabric),
    throw_sage = ("#9DAF8C", fabric),  // the runner across the foot
    // the single, the bunk and the oak double: oak
    frame_oak = ("#CDB08A", wood),
    headboard_oak = ("#CDB08A", wood),
    headboard_teak = ("#B08E66", wood),  // the oak double's slats
    bedding_sage = ("#9DAF8C", fabric),
    bedding_teal = ("#4E7A7F", fabric),
    bedding_blue = ("#9DB7C9", fabric),
    // the futon: pale ash
    frame_ash = ("#E3CFA8", wood),
    headboard_ash = ("#E3CFA8", wood),
    bedding_futon = ("#F4F2EE", fabric),
    quilt_oat = ("#E7E0D4", fabric),   // folded on the seat
    // the upholstered queen
    frame_stone = ("#B9B2A6", fabric),
    headboard_stone = ("#C4BDB1", fabric),
    bedding_oat = ("#E9DCC9", fabric),
    // the four-poster: dark walnut, a linen canopy
    frame_walnut = ("#4A3426", wood),
    headboard_walnut = ("#4A3426", wood),
    bedding_rust = ("#B5653F", fabric),
    canopy = ("#F2EBDD", fabric),
    // the daybed: painted white
    frame_white = ("#F1F0EC", satin),
    back_white = ("#F1F0EC", satin),
    cushion_rose = ("#E4B8AE", fabric),
}

pub const F: GameFrame = GameFrame::new(0.0);

pub const DUVET: f64 = 0.05; // the duvet's top above the mattress top
pub const FLAP: f64 = 0.045; // the flap's thickness at rest
// The game's quilt over a sleeper lies 10 cm above the mattress off the body, times the sleeper's size (8.5 cm
// for the smallest), and rounds down to 6 cm over its outer 8 cm. Folded, the flap must stay under it, so the
// clip also flattens it to this much of its thickness (its top, the sheet's, 7.5 cm above the mattress) and,
// where the quilt's edge would cross it, narrows it to 8 cm inside that edge (`turn_down`).
pub const FOLDED: f64 = 0.4;
/// The quilt's edge from the bed's middle: a one-tile bed's, a two-tile bed's outer one.
fn quilt_edge(tiles: usize) -> f64 {
    match tiles {
        1 => 0.46,
        _ => 1.0,
    }
}
pub const SECONDS: f64 = 1.0; // the duvet turns down this fast, as the resident gets in
pub const WALL: f64 = -0.93; // the wall's face in a 2-deep footprint: the head end's back stands on it

/// Stands the bed on the floor where it was built (its mattress centred, its head end on the wall's face) and
/// says the centre of its bounds, the Model's `centre`.
pub fn settle_bed() {
    let (lo, hi) = measure(&meshes());
    scene::log(format!(
        "[bedroom] centre (game x, depth) {:.4}, {:.4}",
        -(lo[0] + hi[0]) / 2.0,
        -(lo[1] + hi[1]) / 2.0
    ));
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
}

/// A pillow on a mattress top `top`, `h` thick in the middle: plump, thinning to its seams, its corners pinched
/// in (not `pinch`: its sides drawn in between them instead, for one whose corners are the bed's outermost
/// points, the futon's standing on its back), its head end raised a little and a little crumpled.
#[allow(clippy::too_many_arguments)]
pub fn pillow(
    name: &str,
    w: f64,
    h: f64,
    d: f64,
    x: impl Coord,
    top: f64,
    z: f64,
    tilt: f64,
    material: Option<Mat>,
    seed: f64,
    pinch: bool,
) -> Obj {
    let p = soft!(
        F,
        name,
        w,
        h * 0.34,
        d,
        (x, top + h / 2.0 - 0.01, z),
        material.unwrap_or(M.pillow),
        rot = (tilt, 0.0, 0.0),
        round = (h * 0.17).min(0.03),
        puff = h * 0.33,
        pinch = if pinch { h * 0.26 } else { 0.0 },
        waist = h * (if pinch { 0.04 } else { 0.12 }),
        resting = true
    );
    jitter!(p, 0.003, size = 0.12, seed = seed + x.f() * 7.0, up = 0.4)
}

/// A plain pillow: tilted 8 degrees, pinched, seed 0.
pub fn pillow_at(name: &str, w: f64, h: f64, d: f64, x: impl Coord, top: f64, z: f64) -> Obj {
    pillow(name, w, h, d, x, top, z, 8.0, None, 0.0, true)
}

/// `mattress`'s options.
pub struct Mattress {
    pub x: f64,
    pub material: Option<Mat>,
    pub puff: f64,
    pub piping: f64,
    pub round: f64,
    pub flat: usize,
}

impl Default for Mattress {
    fn default() -> Mattress {
        Mattress {
            x: 0.0,
            material: None,
            puff: 0.006,
            piping: 0.007,
            round: 0.035,
            flat: 3,
        }
    }
}

/// A mattress `w` x `d`, `h` deep, its top (the middle of it, a hair puffed) at `top`, where sleepers lie:
/// rounded, with a piped seam round its top and bottom.
#[allow(clippy::too_many_arguments)]
pub fn mattress(name: &str, w: f64, h: f64, d: f64, top: f64, z: impl Coord, o: Mattress) -> Obj {
    let Mattress {
        x,
        material,
        puff,
        piping,
        round,
        flat,
    } = o;
    // (firm, its edges taped: no pinched corners)
    soft!(
        F,
        name,
        w,
        h - 2.0 * puff,
        d,
        (x, top - h / 2.0, z),
        material.unwrap_or(M.mattress),
        round = round,
        puff = puff,
        flat = flat,
        piping = piping,
        pinch = 0.0,
        resting = true
    )
}

pub const RUMPLE: f64 = 0.03; // how deep the duvet's folds are
pub const FOLDS: f64 = 0.26; // and how far apart

/// The duvet's lower two thirds: from the fold to the foot, draped down the sides and the foot end, puffed on top
/// (its middle at `DUVET`) and rumpled, as a duvet shaken out lies: soft folds across it, its sides hanging
/// unevenly, its corners drawn in. The folds only go down and in, so it stays under the game's quilt and inside its
/// box. `runner` (from, to, in game depth): where a runner lies across it and it is smooth.
#[allow(clippy::too_many_arguments)]
fn duvet(
    top: f64,
    half: f64,
    z_fold: f64,
    z_foot: f64,
    drape: f64,
    material: Mat,
    seed: f64,
    flat: Option<usize>,
    runner: Option<(f64, f64)>,
) -> Obj {
    let (over, puff) = (0.028, 0.012);
    let (y0, y1) = (top - drape, top + DUVET);
    let mid = (z_fold + z_foot + over) / 2.0;
    // (the soft box's own y runs to the back: a game depth z is its y = mid - z)
    let calm = runner.map(|r| (mid - r.1, mid - r.0));
    soft!(
        F,
        "duvet",
        2.0 * (half + over),
        y1 - y0 - 2.0 * puff,
        z_foot + over - z_fold,
        (0, (y0 + y1) / 2.0, mid),
        material,
        round = 0.06,
        puff = puff,
        flat = flat,
        pinch = 0.0,
        rumple = RUMPLE,
        rumple_size = FOLDS,
        seed = seed + half,
        calm = calm,
        resting = true
    )
}

/// The duvet's top third, flat on the mattress with the sheet turned down over its head edge; one part hinged
/// across its fold on the duvet's top, so turning it 180 degrees lays it over the rest.
pub fn flap(top: f64, half: f64, z_top: f64, z_fold: f64, material: Mat) -> Obj {
    let parts = new_parts(|| {
        let y = top + DUVET - FLAP / 2.0;
        soft!(
            F,
            "flap",
            2.0 * (half + 0.012),
            FLAP - 0.01,
            z_fold - z_top,
            (0, y, (z_top + z_fold) / 2.0),
            material,
            round = 0.018,
            puff = 0.005
        );
        soft!(
            F,
            "flap_sheet",
            2.0 * (half + 0.016),
            FLAP + 0.008,
            0.13,
            (0, y, z_top + 0.055),
            M.sheet,
            round = 0.022
        );
    });
    // A hair above the duvet's top, so the flap folded over doesn't lie in its plane.
    hinged(
        join(&parts, "duvet_flap", None),
        F.at(0, top + DUVET + 0.003, z_fold),
    )
}

/// `bedding`'s options.
#[derive(Default)]
pub struct Bedding {
    pub fold: Option<f64>,
    pub seed: f64,
    pub flat: Option<usize>,
    pub runner: Option<(f64, f64)>,
}

/// A duvet whose top third turns down: the still lower part (left in the scene, joined with the bed); returns
/// where it folds.
pub fn bedding(
    top: f64,
    half: f64,
    z_top: f64,
    z_foot: f64,
    drape: f64,
    material: Mat,
    o: Bedding,
) -> f64 {
    let z_fold = o.fold.unwrap_or(z_top + (z_foot - z_top) / 3.0);
    duvet(
        top, half, z_fold, z_foot, drape, material, o.seed, o.flat, o.runner,
    );
    z_fold
}

/// A runner laid across the foot of the bed over the duvet, `depth` wide, hanging `hang` down each side to game
/// x = ±`x`: one cloth swept over the bed, its top `y_top`.
pub fn runner(name: &str, x: f64, y_top: f64, z: f64, depth: f64, hang: f64, material: Mat) -> Obj {
    let (t, rr) = (0.012, 0.035);
    let yc = y_top - t / 2.0;
    let mut pts = vec![(-x, yc - hang, z)];
    pts.extend((0..91).step_by(15).map(|a| {
        let a = radians(a as f64);
        (-x + rr - rr * cos(a), yc - rr + rr * sin(a), z)
    }));
    pts.extend([0.5, 0.0, -0.5].map(|k| (-x * k, yc, z)));
    pts.extend((0..91).step_by(15).map(|a| {
        let a = radians(a as f64);
        (x - rr + rr * sin(a), yc - rr + rr * cos(a), z)
    }));
    pts.push((x, yc - hang, z));
    let r = fsweep!(
        F,
        name,
        &pts,
        0,
        material,
        profile = rounded_rect(t, depth, 0.005, 1),
        up = (1, 0, 0)
    );
    jitter!(r, 0.003, size = 0.1, seed = z, up = 0.2)
}

/// An upholstered channel on a headboard: a padded column `w` wide from height y0 to y1, its back at game depth
/// `z` and its front `t` in front of it, plump across and tucked in at its seams; `dome` rounds its top to a half
/// circle, else its top corners are softly rounded.
#[allow(clippy::too_many_arguments)]
pub fn channel(
    name: &str,
    x: f64,
    w: f64,
    y0: f64,
    y1: f64,
    z: f64,
    t: f64,
    material: Mat,
    dome: bool,
) -> Obj {
    let (verts, rows) = (16, 4);
    let (bx, by) = (-x, -(z + t / 2.0));
    let hw = w / 2.0;
    let r = if dome { hw } else { 0.06_f64.min(hw * 0.5) };
    let ring = |rx: f64, ry: f64, at_z: f64| {
        ring!(rx, ry, z = at_z, verts = verts, square = 3.0, at = (bx, by))
    };
    let mut secs: Vec<Vec<P3>> = vec![ring(hw * 0.92, t / 2.0 * 0.5, y0)];
    for k in 0..=rows {
        secs.push(ring(
            hw,
            t / 2.0,
            y0 + 0.015 + (y1 - r - y0 - 0.015) * k as f64 / rows as f64,
        ));
    }
    for a in [25.0, 50.0, 70.0, 84.0] {
        let (c, s) = (cos(radians(a)), sin(radians(a)));
        let rx = if dome { hw * c } else { hw - r + r * c };
        secs.push(ring(
            rx.max(0.004),
            t / 2.0 * (0.55 + 0.45 * c),
            y1 - r + r * s,
        ));
    }
    secs.push(ring(if dome { 0.004 } else { hw - r }, t / 2.0 * 0.4, y1));
    loft!(name, &secs, material = material)
}

/// Keys `clip`: the flap turns 180 degrees over the rest of the duvet and, as it goes, flattens to `FOLDED` of its
/// thickness (about its hinge, so it stays on the duvet) and narrows to keep 8 cm inside the edge of the game's
/// quilt (`quilt_edge`), so the quilt over a sleeper hides it. `half`: the bedding's half width.
pub fn turn_down(flap: Obj, clip: &str, half: f64, tiles: usize) {
    let narrow = 1.0_f64.min((quilt_edge(tiles) - 0.08) / (half + 0.016));
    swing!(flap, clip, 180.0, seconds = SECONDS, axis = Axis::X);
    squash!(flap, clip, (narrow, 1, FOLDED), seconds = SECONDS);
    squash!(flap, clip, (1, ROLL, 1), seconds = SECONDS, back = true);
}

// Half way over, the flap is gathered to this much of its length along the bed (it springs out again as it lies
// down), so a duvet turned back rolls over its fold as cloth does, a low bulge, never a board standing up.
const ROLL: f64 = 0.2;

/// Joins what's in the scene as the still bed, builds the flap, settles, and keys `turn_down`.
pub fn finish(name: &str, flap_args: (f64, f64, f64, f64, Mat), tiles: usize) -> Vec<Obj> {
    let still = meshes();
    join(&still, name, Some(P3::ZERO));
    let (top, half, z_top, z_fold, material) = flap_args;
    let f = flap(top, half, z_top, z_fold, material);
    settle_bed();
    turn_down(f, "turn_down", half, tiles);
    meshes()
}

/// Round legs tapering to their feet.
pub fn legs_round(xs: &[f64], zs: &[f64], h: f64, material: Mat, r: f64) {
    for &x in xs {
        for &z in zs {
            fcone!(
                F,
                format!("leg{}{}", fr(x), fr(z)),
                r * 0.75,
                r,
                h,
                (x, h / 2.0, z),
                material,
                verts = 12
            );
        }
    }
}

/// Square legs tapering to their feet, `w` across at the top.
pub fn legs_square(xs: &[f64], zs: &[f64], h: f64, material: Mat, w: f64) {
    let r = w / sqrt(2.0);
    for &x in xs {
        for &z in zs {
            fcone!(
                F,
                format!("leg{}{}", fr(x), fr(z)),
                r * 0.72,
                r,
                h,
                (x, h / 2.0, z),
                material,
                verts = 4,
                rot = (0, 0, 45),
                smooth = false
            );
        }
    }
}

/// A float as Python writes it in a name.
pub fn fr(x: f64) -> String {
    artkit::json::float_repr(x)
}

/// What a post's top is.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cap {
    Ball,
    Pyramid,
    Flat,
}

/// A square post from y0 up to `h`, `w` across: its shaft, a collar under the top and a cap: a ball, a pyramid or a
/// flat moulded block.
#[allow(clippy::too_many_arguments)]
pub fn post(name: &str, x: f64, z: f64, h: f64, w: f64, material: Mat, cap: Cap, y0: f64) {
    let ch = match cap {
        Cap::Ball => w * 0.8,
        Cap::Pyramid => w * 0.55,
        Cap::Flat => w * 0.3,
    };
    let shaft = h - ch - 0.018;
    fbox!(
        F,
        format!("{name}_shaft"),
        w,
        shaft - y0,
        w,
        (x, (y0 + shaft) / 2.0, z),
        material,
        bevel = 0.006,
        bevel_segments = 1
    );
    fbox!(
        F,
        format!("{name}_collar"),
        w + 0.008,
        0.018,
        w + 0.008,
        (x, shaft + 0.009, z),
        material,
        bevel = 0.004,
        bevel_segments = 1
    );
    match cap {
        Cap::Ball => {
            fcyl!(
                F,
                format!("{name}_neck"),
                w * 0.28,
                ch * 0.3,
                (x, shaft + 0.018 + ch * 0.15, z),
                material,
                verts = 10
            );
            fsphere!(
                F,
                format!("{name}_ball"),
                ch * 0.38,
                (x, h - ch * 0.38, z),
                material,
                segments = 12,
                rings = 7
            );
        }
        Cap::Pyramid => {
            fcone!(
                F,
                format!("{name}_cap"),
                (w + 0.008) / sqrt(2.0),
                0.004,
                ch,
                (x, h - ch / 2.0, z),
                material,
                verts = 4,
                rot = (0, 0, 45),
                smooth = false
            );
        }
        Cap::Flat => {
            fbox!(
                F,
                format!("{name}_cap"),
                w + 0.004,
                ch,
                w + 0.004,
                (x, h - ch / 2.0, z),
                material,
                bevel = 0.008,
                bevel_segments = 1
            );
        }
    }
}

/// `board`'s options.
pub struct Board {
    pub arch: f64,
    pub holes: Vec<(f64, f64, f64, f64)>,
    pub x: f64,
    pub bevel: f64,
    pub n: usize,
}

impl Default for Board {
    fn default() -> Board {
        Board {
            arch: 0.0,
            holes: Vec::new(),
            x: 0.0,
            bevel: 0.006,
            n: 14,
        }
    }
}

/// A board `w` wide from y0 up to y1 at game depth `z`, `t` thick, its top arched up `arch` in the middle (its ends
/// at y1 - arch); `holes`: (x, y, w, h) rounded slots cut through it.
#[allow(clippy::too_many_arguments)]
pub fn board(name: &str, w: f64, y0: f64, y1: f64, z: f64, t: f64, material: Mat, o: Board) -> Obj {
    let Board {
        arch,
        holes,
        x,
        bevel,
        n,
    } = o;
    let top = (0..=n).map(|i| {
        (
            w / 2.0 - w * i as f64 / n as f64,
            y1 - arch + arch * cos(PI * (0.5 - i as f64 / n as f64)),
        )
    });
    let mut outline = vec![(-w / 2.0, y0), (w / 2.0, y0)];
    outline.extend(top);
    let loops: Vec<Vec<(f64, f64)>> = holes
        .iter()
        .map(|&(hx, hy, hw, hh)| {
            rounded_rect(hw, hh, hw.min(hh) * 0.5, 3)
                .iter()
                .map(|&(u, v)| (hx + u, hy + v))
                .collect()
        })
        .collect();
    fextrude!(
        F,
        name,
        &outline,
        t,
        (x, 0, z),
        material,
        holes = loops,
        bevel = bevel,
        bevel_segments = 1
    )
}

/// Tufting buttons across a mattress or cushion top at height `y`, a hair proud of it.
pub fn tufts(name: &str, xs: &[f64], zs: &[f64], y: f64, material: Mat) {
    let r = 0.014;
    for (i, &x) in xs.iter().enumerate() {
        for (k, &z) in zs.iter().enumerate() {
            fsphere!(
                F,
                format!("{name}{i}_{k}"),
                r,
                (x, y, z),
                material,
                scale = (1, 1, 0.4),
                segments = 8,
                rings = 5
            );
        }
    }
}

//! What the cheerful pack is built from: its palette, flat shapes from outlines, arch outlines, bands, bulbs,
//! pleated curtains and tumblers.

use artkit::bevel::{EdgeOrder, bevel_edges};
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{P3, PI, V3, cos, sin};
use artkit::parts::numbered;
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{fcyl, fsphere, fsweep, kit, object_of, palette, revolve, torus};

palette! { M:
    // the jukebox
    cherry = ("#D7263D", gloss), // the cabinet; its main part
    amber = ("#F6A609", gloss, glow("#FFB733")), // the arch band and the bubble tubes: lit at night
    sunny = ("#FFE29A", gloss, glow("#FFE7A8")), // the inner arch band, lit
    window = ("#5B3A8C", gloss), // the record window's back
    grille = ("#E9D8B0", matte), // the speaker cloth (plain: the spinning label is the fourth draw it would cost)
    cards = ("#FFF2C8", gloss), // the song cards
    vinyl = ("#2B2433", gloss),
    label = ("#F6C744", gloss),
    coral = ("#F28D8D", gloss),
    mint = ("#9ED8C6", gloss), // the buttons; the mocktail cart's shelves (its main part)
    sky = ("#6EC1E4", gloss),
    chrome = ("#D0D3D6", chrome),
    brass = ("#C9A54A", metal),
    // the karaoke machine
    speaker = ("#8A6BC0", gloss), // the speaker cabinet; its main part
    speaker_trim = ("#54456E", satin),
    cone = ("#3B2F4E", satin),
    steel = ("#C9CCD0", metal),
    dark = ("#2B2433", satin), // the screen's case, the mic, the tablet
    lyrics = ("#3B2A6B", gloss, glow("#8E7BFF"), screen()), // the lyrics screen: lit while it's sung to
    white = ("#F4F4F2", gloss),
    stand = ("#3A3D41", satin), // stands and poles
    lens = ("#FFF4B0", gloss),
    stage = ("#6A3FA0", fabric), // the round stage rug
    star = ("#F6C744", satin),
    // the mocktail cart
    canopy = ("#F28D8D", fabric),
    canopy_white = ("#F4F4F2", fabric),
    tyre = ("#F2F2F0", satin),
    porcelain = ("#F6F3EA", gloss),
    juice = ("#FF9A3C", gloss),
    ice = ("#E6F7FF", gloss),
    tumbler = ("#E6F0F2", gloss), // the glasses' rims, opaque (no glass draw)
    pineapple = ("#E8B33C", satin),
    leaf = ("#4F9A3E", satin),
    lemon = ("#F2E05C", gloss),
    lime = ("#8CC63F", gloss),
    orange = ("#F28C28", gloss),
    drink_pink = ("#F07AA0", gloss),
    drink_sun = ("#F6C744", gloss),
    drink_teal = ("#7FD4C1", gloss),
    fridge = ("#F2F2F0", satin),
    // the hammock
    wood = ("#C9905A", wood), // the stand
    spreader = ("#B07A4A", satin), // the spreader bars
    rope = ("#EDE4D3", matte),
    stripe_red = ("#E4572E", fabric),
    stripe_orange = ("#F28C28", fabric),
    stripe_yellow = ("#F6C744", fabric),
    stripe_green = ("#8CC63F", fabric),
    stripe_blue = ("#3FA7D6", fabric),
    stripe_violet = ("#7B5EA7", fabric),
    pillow = ("#F4F4F2", satin),
    // the sunrise lamp
    housing = ("#F4F1EA", gloss), // the disc's case and the base; its main part
    lamp_metal = ("#E9DCC7", metal), // the ring and the stem
    dawn = ("#FFCF9E", satin, glow("#FFB070")), // the sky behind the sun: the lamp glows at night
    hills = ("#F28C5A", satin),
    hills_far = ("#F6A97A", satin),
    sun = ("#FFE08A", satin), // plain: a glow on it shows through the hills while it is set
    yoga = ("#F29E8E", matte),
    yoga_roll = ("#E9876F", matte),
    // the party booth
    booth = ("#F28D8D", fabric), // the upholstery; its main part
    piping = ("#F6C744", satin),
    panel = ("#7A5FB8", satin),
    curtain = ("#6EC1E4", fabric),
    tablet_face = ("#E84F9A", gloss),
    popcorn = ("#FFF3C4", matte),
    bulb_a = ("#F6C744", gloss),
    bulb_b = ("#6EC1E4", gloss),
    bulb_c = ("#F28D8D", gloss),
    bulb_d = ("#8CC63F", gloss),
    bulb_e = ("#B8A1E3", gloss),
    bulb_f = ("#F28C28", gloss),
    // added with the detail pass
    cloud = ("#FFF1E2", satin), // the sunrise scene's clouds
    cork = ("#C9A27A", matte), // the yoga block
}

/// The bulbs' colours, in turn.
pub const BULBS: [Mat; 6] = [M.bulb_a, M.bulb_b, M.bulb_c, M.bulb_d, M.bulb_e, M.bulb_f];
/// The hammock's stripes, in turn.
pub const STRIPES: [Mat; 6] = [
    M.stripe_red,
    M.stripe_orange,
    M.stripe_yellow,
    M.stripe_green,
    M.stripe_blue,
    M.stripe_violet,
];
/// The wall's face in a 1-deep footprint, game depth.
pub use artkit::kit::WALL;
/// In a 2-deep one.
pub const WALL2: f64 = -0.93;
/// The mocktail cart's top shelf's top.
pub const TOP: f64 = 0.815;

/// Eases a swept part's flat-capped ends by 4 mm (the rule's pixel at the play camera): only the cap rims turn
/// 60° or more, so only they are rounded.
pub fn ease_ends(obj: Obj) -> Obj {
    bevel_edges(obj, 0.004, 1, 60.0, &EdgeOrder::Corners)
}

// --- meshes from points ------------------------------------------------------------------------------------------

/// A flat shape: the outline `pts` [(x, y)] (game, in order round it) between depths `z0` and `z1`.
pub fn slab(
    f: &GameFrame,
    name: &str,
    pts: &[(f64, f64)],
    z0: f64,
    z1: f64,
    mat: Mat,
    bevel: f64,
) -> Obj {
    let n = pts.len() as u32;
    let mut verts: Vec<P3> = pts.iter().map(|&(x, y)| f.at(x, y, z1)).collect();
    verts.extend(pts.iter().map(|&(x, y)| f.at(x, y, z0)));
    let mut faces = vec![(0..n).collect::<Vec<u32>>(), (n..2 * n).rev().collect()];
    faces.extend((0..n).map(|i| vec![i, (i + 1) % n, n + (i + 1) % n, n + i]));
    object_of!(name, &verts, &faces, &[mat], bevel = bevel)
}

/// An arch-topped outline: straight sides `hw` either side of x = 0 from `yb` up to `yc`, a half circle over.
pub fn tomb(hw: f64, yb: f64, yc: f64, n: usize) -> Vec<(f64, f64)> {
    let mut pts = vec![(-hw, yb), (hw, yb)];
    pts.extend((0..=n).map(|k| {
        (
            hw * cos(PI * k as f64 / n as f64),
            yc + hw * sin(PI * k as f64 / n as f64),
        )
    }));
    pts
}

/// Turns a part built along Blender +Z (a revolve) to point along `direction` (Blender).
pub fn aim(obj: Obj, direction: V3) -> Obj {
    obj.edit(|o| o.quat = Some(direction.to_track_quat_zy()));
    obj
}

/// A band swept round a rounded block's front and sides (not its back, on the wall): the block `w` wide, from
/// depth `z_back` to `front`, its upright corners rounded by `r`, at height `y`. `profile` is the band's
/// cross-section (u up, v out).
#[allow(clippy::too_many_arguments)]
pub fn around(
    f: &GameFrame,
    name: &str,
    w: f64,
    _d: f64,
    r: f64,
    y: f64,
    z_back: f64,
    front: f64,
    profile: Vec<(f64, f64)>,
    mat: Mat,
    corner: usize,
) -> Obj {
    let zc = (z_back + front) / 2.0;
    let lp = rounded_rect(w, front - z_back, r, corner);
    let pts: Vec<(f64, f64, f64)> = lp.iter().map(|&(x, z)| (x, y, zc + z)).collect();
    // from the back of the right side round the front to the back of the left side
    let key = |p: &(f64, f64, f64)| (p.2 < z_back + 0.02 && p.0 > 0.0, -p.2);
    let mut k = 0;
    for i in 1..pts.len() {
        if key(&pts[i]) > key(&pts[k]) {
            k = i;
        }
    }
    let pts: Vec<(f64, f64, f64)> = [&pts[k..], &pts[..k]]
        .concat()
        .into_iter()
        .filter(|p| p.2 > z_back + 0.015)
        .collect();
    let mut all = vec![(pts[0].0, y, z_back + 0.005)];
    all.extend(pts.iter().copied());
    all.push((pts[pts.len() - 1].0, y, z_back + 0.005));
    fsweep!(f, name, &all, 0, mat, profile = profile, up = (0, 0, 1))
}

pub fn bulb(f: &GameFrame, x: f64, y: f64, z: f64, k: usize, r: f64) {
    fsphere!(
        f,
        numbered("bulb"),
        r,
        (x, y, z),
        BULBS[k % BULBS.len()],
        segments = 6,
        rings = 3 // (a bead 3 cm across: 7 px at the play camera)
    );
}

/// A pleated curtain hanging from `y1` (its heading) down to `y0` (its hem), along `z0`..`z1` in the plane at
/// `x`, `per_fold` points across each fold (enough that no crest turns 60°, the gate's hard edge). Its folds are
/// shallow under the heading tape, deepen as it falls and drift a little; a turned-up hem thickens its foot (to
/// 11.5 mm, as the heading: thinner than a hard edge's 12 mm faces).
#[allow(clippy::too_many_arguments)]
pub fn pleats(
    f: &GameFrame,
    name: &str,
    x: f64,
    y0: f64,
    y1: f64,
    z0: f64,
    z1: f64,
    folds: i32,
    amp: f64,
    mat: Mat,
    t: f64,
    per_fold: usize,
) -> Obj {
    let h = y1 - y0;
    let rows = [
        (y0, 1.0, 1.15, 0.0),
        (y0 + 0.012, 1.0, 1.15, 0.02),
        (y0 + 0.04, 1.0, 1.0, 0.05),
        (y0 + h * 0.4, 0.92, 1.0, 0.25),
        (y1 - 0.07, 0.72, 1.0, 0.4),
        (y1 - 0.06, 0.4, 1.15, 0.42),
        (y1, 0.36, 1.15, 0.42),
    ];
    let (verts, faces) = kit::pleats!(f, &rows, z0, z1, x, folds, amp, t, per_fold = per_fold, along_x = false);
    // its side hems: a cord down each end, where the cloth's end is (the folds end where they start)
    for z in [z0, z1] {
        let down: Vec<(f64, f64, f64)> = rows
            .iter()
            .map(|&(y, af, _, phase)| (x + amp * af * sin(phase), y, z))
            .collect();
        kit::tube!(f, numbered("side_hem"), &down, 0.0065, mat, verts = 6, round = 1);
    }
    object_of!(name, &verts, &faces, &[mat], sharp = 70)
}

/// A tumbler full of `drink` (opaque: no glass draw): a thick clear foot, the drink, a clear rim, a bendy straw
/// and (`slice_`, a material) a citrus wheel on the rim.
#[allow(clippy::too_many_arguments)]
pub fn tumbler(
    f: &GameFrame,
    x: f64,
    z: f64,
    drink: Mat,
    y: Option<f64>,
    h: f64,
    r: f64,
    straw: Option<Mat>,
    slice_: Option<Mat>,
) {
    let y = y.unwrap_or(TOP);
    fcyl!(
        f,
        numbered("tumbler_foot"),
        r * 0.85,
        0.012,
        (x, y + 0.006, z),
        M.tumbler,
        verts = 12
    );
    revolve!(
        numbered("tumbler"),
        &[
            (0.0, 0.0),
            (r * 0.86, 0.0),
            (r * 0.98, h - 0.012),
            (0.0, h - 0.012)
        ],
        at = f.at(x, y + 0.012, z),
        material = drink,
        segments = 12
    );
    torus!(
        numbered("rim"),
        r * 0.98,
        0.004,
        at = f.at(x, y + h, z),
        material = M.tumbler,
        major_segments = 12,
        minor_segments = 3
    );
    kit::tube!(
        f,
        numbered("straw"),
        &kit::fillet!(
            &[
                (x + 0.008, y + 0.03, z),
                (x + 0.018, y + h + 0.04, z - 0.006),
                (x + 0.045, y + h + 0.06, z - 0.01),
            ],
            0.01,
            steps = 2
        ),
        0.0035,
        straw.unwrap_or(M.coral),
        verts = 5,
        round = 1
    );
    if let Some(s) = slice_ {
        fcyl!(
            f,
            numbered("slice"),
            0.024,
            0.006,
            (x - r * 0.9, y + h - 0.004, z),
            s,
            rot = (0, 0, 90),
            verts = 12
        );
    }
}

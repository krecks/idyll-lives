//! What the cheerful pack is built from: its palette, meshes from points, arch outlines, bands, bulbs, pleated
//! curtains, tumblers, and the kit's parts it uses.

use artkit::forms::{Scale, rounded_rect};
use artkit::materials::Mat;
use artkit::math::{P3, PI, V3, cos, sin};
use artkit::mesh::Mesh;
use artkit::place::{GameFrame, Spot};
use artkit::scene::{self, Obj};
use artkit::{fcyl, fsphere, fsweep, palette, revolve, torus};
use std::cell::Cell;

palette! { M:
    // the jukebox
    cherry = ("#D7263D", gloss), // the cabinet; its main part
    amber = ("#F6A609", gloss, glow("#FFB733")), // the arch band and the bubble tubes: lit at night
    sunny = ("#FFE29A", gloss, glow("#FFE7A8")), // the inner arch band, lit
    window = ("#5B3A8C", gloss), // the record window's back
    grille = ("#E9D8B0", fabric), // the speaker cloth
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
pub const WALL: f64 = -0.43;
/// In a 2-deep one.
pub const WALL2: f64 = -0.93;
/// The mocktail cart's top shelf's top.
pub const TOP: f64 = 0.815;

thread_local! {
    static N: Cell<u32> = const { Cell::new(0) };
}

/// `stem` numbered: each part its own name.
pub fn name(stem: &str) -> String {
    N.with(|n| {
        n.set(n.get() + 1);
        format!("{stem}{}", n.get())
    })
}

// --- meshes from points ------------------------------------------------------------------------------------------

/// A mesh object from Blender-space `verts` and `faces`, normals made outward, smooth below `sharp` degrees.
#[allow(clippy::too_many_arguments)]
pub fn obj(
    name: &str,
    verts: &[P3],
    faces: &[Vec<u32>],
    mats: &[Mat],
    face_mats: Option<&[u16]>,
    sharp: f64,
    bevel: f64,
) -> Obj {
    let mut mesh = Mesh::from_pydata(verts.iter().map(|&p| V3::from(p)).collect(), faces);
    for &m in mats {
        mesh.slot(m);
    }
    if let Some(face_mats) = face_mats {
        for (face, &i) in mesh.faces.iter_mut().zip(face_mats) {
            face.mat = i;
        }
    }
    mesh.recalc_normals();
    let o = scene::add(name, mesh);
    o.edit(|p| p.mesh.shade_smooth(sharp));
    if bevel != 0.0 {
        artkit::bevel::bevel_edges(o, bevel, 2, 40.0, &artkit::bevel::EdgeOrder::Corners); // (a mesh made of points and faces)
    }
    o
}

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
    obj(name, &verts, &faces, &[mat], None, 35.0, bevel)
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

/// A polygon's middle, as Blender gives it: the mean of its corners.
fn centre(mesh: &Mesh, f: usize) -> V3 {
    let c = mesh.corners(f);
    let mut s = V3::ZERO;
    for &v in c {
        s += mesh.verts[v as usize];
    }
    let n = c.len() as f32;
    V3 {
        x: s.x / n,
        y: s.y / n,
        z: s.z / n,
    }
}

/// Gives the faces of `obj` whose middle (its own axes, Blender) passes `test` another material: a band, a
/// surround, a label, with no more triangles.
pub fn paint(obj: Obj, material: Mat, test: impl Fn(V3) -> bool) -> Obj {
    obj.edit(|p| {
        let k = p.mesh.slot(material);
        for f in 0..p.mesh.faces.len() {
            if test(centre(&p.mesh, f)) {
                p.mesh.faces[f].mat = k;
            }
        }
    });
    obj
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
        name("bulb"),
        r,
        (x, y, z),
        BULBS[k % BULBS.len()],
        segments = 6,
        rings = 4
    );
}

/// A pleated curtain hanging from `y1` (its heading) down to `y0` (its hem), along `z0`..`z1` in the plane at
/// `x` (or along x, `z0`..`z1` being x, in the plane at depth `x`: `along_x`). Its folds are shallow under the
/// heading tape, deepen as it falls and drift a little; a turned-up hem thickens its foot.
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
    along_x: bool,
) -> Obj {
    let n = folds * 4;
    let h = y1 - y0;
    let rows = [
        (y0, 1.0, 1.5, 0.0),
        (y0 + 0.012, 1.0, 1.5, 0.02),
        (y0 + 0.04, 1.0, 1.0, 0.05),
        (y0 + h * 0.4, 0.92, 1.0, 0.25),
        (y1 - 0.07, 0.72, 1.0, 0.4),
        (y1 - 0.06, 0.4, 1.3, 0.42),
        (y1, 0.36, 1.3, 0.42),
    ];
    let (mut verts, mut m) = (Vec::new(), 0);
    for &(y, af, tf, phase) in &rows {
        let front: Vec<(f64, f64)> = (0..=n)
            .map(|k| {
                (
                    z0 + (z1 - z0) * k as f64 / n as f64,
                    amp * af * sin(2.0 * PI * folds as f64 * k as f64 / n as f64 + phase)
                        + t * tf / 2.0,
                )
            })
            .collect();
        let mut ring = front.clone();
        ring.extend(front.iter().rev().map(|&(u, o)| (u, o - t * tf)));
        m = ring.len();
        verts.extend(ring.iter().map(|&(u, o)| {
            if along_x {
                f.at(u, y, x + o)
            } else {
                f.at(x + o, y, u)
            }
        }));
    }
    let m = m as u32;
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for r in 0..rows.len() as u32 - 1 {
        let (a, b) = (r * m, (r + 1) * m);
        faces.extend((0..m).map(|i| vec![a + i, a + (i + 1) % m, b + (i + 1) % m, b + i]));
    }
    let n = n as u32;
    for base in [0, (rows.len() as u32 - 1) * m] {
        // the top and bottom edges across the cloth's thickness
        faces.extend(
            (0..n).map(|k| vec![base + k, base + k + 1, base + m - 2 - k, base + m - 1 - k]),
        );
    }
    obj(name, &verts, &faces, &[mat], None, 70.0, 0.0)
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
        name("tumbler_foot"),
        r * 0.85,
        0.012,
        (x, y + 0.006, z),
        M.tumbler,
        verts = 12
    );
    revolve!(
        name("tumbler"),
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
        name("rim"),
        r * 0.98,
        0.004,
        at = f.at(x, y + h, z),
        material = M.tumbler,
        major_segments = 12,
        minor_segments = 3
    );
    tube(
        f,
        &name("straw"),
        &fillet(
            &[
                (x + 0.008, y + 0.03, z),
                (x + 0.018, y + h + 0.04, z - 0.006),
                (x + 0.045, y + h + 0.06, z - 0.01),
            ],
            0.01,
            2,
        ),
        0.0035,
        straw.unwrap_or(M.coral),
        Some(5),
        1,
        Scale::One(1.0),
    );
    if let Some(s) = slice_ {
        fcyl!(
            f,
            name("slice"),
            0.024,
            0.006,
            (x - r * 0.9, y + h - 0.004, z),
            s,
            rot = (0, 0, 90),
            verts = 12
        );
    }
}

// --- the kit's parts ------------------------------------------------------------------------------------------

/// The kit's tube (`artkit::kit::tube`), as the script called it.
#[allow(clippy::too_many_arguments)]
pub fn tube<P: Spot>(
    f: &GameFrame,
    name: &str,
    points: &[P],
    r: f64,
    material: Mat,
    verts: Option<usize>,
    round: usize,
    scale: Scale,
) -> Obj {
    artkit::kit::tube(
        f,
        name,
        points,
        r,
        material,
        artkit::kit::Tube {
            verts,
            round,
            scale,
        },
    )
}

/// The kit's fillet (`artkit::kit::fillet`): the path through `points` with each inner corner rounded.
pub fn fillet(points: &[(f64, f64, f64)], radius: f64, steps: usize) -> Vec<(f64, f64, f64)> {
    artkit::kit::fillet(points, radius, steps)
}

/// The kit's bar pull (`artkit::kit::pull`), as the script called it.
#[allow(clippy::too_many_arguments)]
pub fn pull(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    face: f64,
    length: f64,
    material: Mat,
    r: f64,
    out: f64,
    upright: bool,
    verts: usize,
) -> Obj {
    artkit::kit::pull(
        f,
        name,
        x,
        y,
        face,
        length,
        material,
        artkit::kit::Pull {
            r,
            out,
            upright,
            verts,
        },
    )
}

/// The kit's moulding (`artkit::kit::moulding`), as the script called it: on the front at depth `at`, or on top
/// at height `at` (`front` false).
#[allow(clippy::too_many_arguments)]
pub fn moulding(
    f: &GameFrame,
    name: &str,
    outline: &[(f64, f64)],
    profile: &[(f64, f64)],
    material: Mat,
    at: f64,
    front: bool,
    angle: f64,
) -> Obj {
    let plane = if front {
        artkit::place::Facing::Front
    } else {
        artkit::place::Facing::Top
    };
    artkit::kit::moulding(
        f,
        name,
        outline,
        profile,
        material,
        artkit::kit::Moulding { at, plane, angle },
    )
}

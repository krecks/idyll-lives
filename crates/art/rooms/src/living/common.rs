//! What the living room's things are built from: stuffed cushions and upholstered panels, legs, throws, the seat
//! that sinks in, turned discs, bent tubes, table tops, knobs and books.

use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{P3, PI, V3, cos, exp, pow, sin};
use artkit::mesh::Mesh;
use artkit::motion::{Squash, squash};
use artkit::parts::{join, new_parts, settle};
use artkit::place::{Coord, Facing};
use artkit::scene::{self, Obj, meshes};
use artkit::soft::{Puff, Sag, SoftBox};
use artkit::{fbox, fextrude, fsweep, loft, ring, soft};

pub const SINK: f64 = 0.02;
const SEAT_SAG: f64 = 0.014; // how far a seat cushion's top sinks towards its front edge, where legs go over it
const SEAT_OVERHANG: f64 = 0.005; // how far its top stands over its boxing

/// `cushion`'s options: its turn, its puff, its piping (none: `None`), its steps across and round (by default
/// the soft box's own), and `keep_top` for a seat cushion.
#[derive(Clone, Copy)]
pub struct Cushion {
    pub rot: P3,
    pub puff: f64,
    pub piping: Option<f64>,
    pub flat: Option<usize>,
    pub rs: Option<usize>,
    pub keep_top: bool,
}

impl Default for Cushion {
    fn default() -> Cushion {
        Cushion {
            rot: P3::ZERO,
            puff: 0.018,
            piping: Some(0.006),
            flat: None,
            rs: None,
            keep_top: false,
        }
    }
}

/// A stuffed cushion: its edges rounded, its two largest faces puffed out by `puff`, its corners pinched in, a
/// piped seam round each (`piping` None: none). `keep_top`: a seat cushion, its puffed middle where the old box's
/// top was (the seat height), its edges `puff` lower, its top sagging towards its front edge and standing over its
/// boxing.
pub fn cushion(
    name: &str,
    w: f64,
    h: f64,
    d: f64,
    at: (f64, f64, f64),
    mat: Mat,
    o: Cushion,
) -> Obj {
    let (mut h, mut at) = (h, at);
    let (mut sag, mut overhang) = (Sag::default(), 0.0);
    if o.keep_top {
        (h, at) = (h - o.puff, (at.0, at.1 - o.puff / 2.0, at.2));
        (sag, overhang) = (Sag::Front(SEAT_SAG), SEAT_OVERHANG);
    }
    // (it lies on the deck or leans on the back: `resting`)
    soft!(
        F,
        name,
        w,
        h,
        d,
        at,
        mat,
        rot = o.rot,
        round = 0.045_f64.min(0.4 * w.min(h).min(d)),
        puff = o.puff,
        piping = o.piping,
        flat = o.flat,
        round_segments = o.rs,
        resting = true,
        sag = sag,
        overhang = overhang
    )
}

/// An upholstered panel (a base, an arm, a back): firm, its edges rounded, a welt round its thin side's seams.
/// `o`: more of the soft box's (a buttoned back's `tufts`).
#[allow(clippy::too_many_arguments)]
pub fn panel(
    name: &str,
    w: f64,
    h: f64,
    d: f64,
    at: (f64, f64, f64),
    mat: Mat,
    round: f64,
    puff: f64,
    piping: Option<f64>,
    mut o: SoftBox,
) -> Obj {
    o.round = round;
    o.puff = if puff != 0.0 {
        Puff::Faces(puff)
    } else {
        Puff::None
    };
    o.piping = piping;
    F.soft(name, w, h, d, at, mat, o)
}

/// `panel` with nothing more of the soft box's.
pub fn plain(round: f64) -> SoftBox {
    SoftBox {
        round,
        ..SoftBox::default()
    }
}

/// A turned leg tapering from the frame at height `h` to its foot on the floor, a collar under the frame,
/// splayed by (dx, dz) at the foot (game).
#[allow(clippy::too_many_arguments)]
pub fn leg(
    name: &str,
    x: f64,
    z: f64,
    h: f64,
    mat: Mat,
    r: f64,
    splay: (f64, f64),
    verts: usize,
) -> Obj {
    let (sx, sz) = splay;
    fsweep!(
        F,
        name,
        &[
            (x, h, z),
            (x, h - 0.003, z), // its top eased
            (x, h - 0.012, z),
            (x + sx * 0.12, h - 0.02, z + sz * 0.12),
            (x + sx, 0.006, z + sz),
            (x + sx, 0.0, z + sz)
        ],
        r,
        mat,
        verts = verts,
        scale = [0.86, 1.0, 1.0, 0.86, 0.66, 0.6]
    )
}

/// The seat cushions `cushions()` makes (returning their top), joined into the part `seat`, with the clip that
/// sinks them: (the seat, how thick it is).
pub fn seat(cushions: impl FnOnce() -> f64, bottom: f64, mid: impl Coord) -> (Obj, f64) {
    let mut top = 0.0;
    let parts = new_parts(|| top = cushions());
    let seat = join(&parts, "seat", Some(F.at(0, bottom, mid)));
    (seat, top - bottom)
}

pub fn sink(seat_and_thickness: (Obj, f64)) {
    let (seat, thick) = seat_and_thickness;
    squash(
        seat,
        "sink_in",
        (1.0, 1.0, (thick - SINK) / thick),
        Squash {
            seconds: 0.6,
            ..Squash::default()
        },
    );
}

/// A sofa's leg, splayed a little towards its corner (by default).
pub fn taper_leg(
    name: &str,
    x: f64,
    z: f64,
    h: f64,
    mat: Mat,
    r: f64,
    splay: Option<(f64, f64)>,
) -> Obj {
    leg(
        name,
        x,
        z,
        h,
        mat,
        r,
        splay.unwrap_or((0.008_f64.copysign(x), 0.008_f64.copysign(z))),
        10,
    )
}

/// A throw pillow leaning back: plump in the middle, thin at its seams, its corners pinched into ears.
#[allow(clippy::too_many_arguments)]
pub fn pillow(name: &str, x: f64, y: f64, z: f64, mat: Mat, turn: i32, w: f64, h: f64) {
    soft!(
        F,
        name,
        w,
        h,
        0.07,
        (x, y, z),
        mat,
        rot = (-13, 0, -turn),
        round = 0.03,
        puff = 0.055,
        pinch = 0.008,
        waist = 0.022,
        resting = true
    );
}

/// Settled on its Model's `centre`: built in the old part's numbers, its back moved onto the wall face; then the
/// seat's clip.
pub fn settled(seat: (Obj, f64)) -> Vec<Obj> {
    settle(&meshes());
    sink(seat);
    Vec::new()
}

/// Presses dimples into `obj`'s top (Blender, its mesh at the origin): each vertex higher than `above` moves down
/// by `depth`, falling off over `radius` from each (Blender x, y) point: the tufts under a buttoned top.
pub fn press(obj: Obj, points: &[(f64, f64)], depth: f64, radius: f64, above: f64) -> Obj {
    obj.edit(|o| {
        for v in &mut o.mesh.verts {
            if v.z as f64 <= above {
                continue;
            }
            let (x, y) = (v.x as f64, v.y as f64);
            let d =
                artkit::py::sum(points.iter().map(|&(px, py)| {
                    exp(-(pow(x - px, 2.0) + pow(y - py, 2.0)) / pow(radius, 2.0))
                }));
            v.z = (v.z as f64 - depth * 1.0_f64.min(d)) as f32;
        }
    });
    obj
}

/// Rings for `loft` from rows of (height, radius) or (height, rx, ry[, centre y]), bottom first (Blender).
pub fn round_rings(rows: &[&[f64]], verts: usize, square: f64, at: (f64, f64)) -> Vec<Vec<P3>> {
    rows.iter()
        .map(|row| {
            let (z, rx) = (row[0], row[1]);
            let ry = if row.len() > 2 { row[2] } else { rx };
            let cy = if row.len() > 3 { row[3] } else { 0.0 };
            ring!(
                rx,
                ry,
                z = z,
                verts = verts,
                square = square,
                at = (at.0, at.1 + cy)
            )
        })
        .collect()
}

/// A round thing turned about the upright through game (x, z): rows of (height, radius), bottom first; a radius
/// near 0 at either end closes it there (a top's middle).
pub fn disc(name: &str, rows: &[(f64, f64)], mat: Mat, x: f64, z: f64, verts: usize) -> Obj {
    let rows: Vec<[f64; 2]> = rows.iter().map(|&(a, b)| [a, b]).collect();
    let rows: Vec<&[f64]> = rows.iter().map(|r| r.as_slice()).collect();
    loft!(
        name,
        &round_rings(&rows, verts, 2.0, (-x, -z)),
        material = mat
    )
}

/// A table top `w` × `d`, its corners rounded by `r`, `t` thick, its top at `y`; its edges eased (`bevel`).
#[allow(clippy::too_many_arguments)]
pub fn top(
    name: &str,
    w: f64,
    d: f64,
    r: f64,
    t: f64,
    y: f64,
    mat: Mat,
    z: f64,
    x: f64,
    bevel: f64,
) -> Obj {
    fextrude!(
        F,
        name,
        &rounded_rect(w, d, r, 3),
        t,
        (x, y - t / 2.0, z),
        mat,
        plane = Facing::Top,
        bevel = bevel,
        bevel_segments = 2
    )
}

/// A turned knob standing out of a drawer front at depth `z`: a stem swelling into a mushroom head.
pub fn knob(name: &str, x: f64, y: f64, z: f64, mat: Mat) -> Obj {
    let r = 0.014;
    let at = F.at(x, y, z);
    let mut secs = vec![ring!(r * 0.4, r * 0.4, z = 0.0, verts = 10)];
    for (d, k) in [
        (0.004, 0.45),
        (0.009, 0.6),
        (0.014, 1.0),
        (0.02, 0.95),
        (0.024, 0.55),
    ] {
        secs.push(ring!(r * k, r * k, z = d, verts = 10));
    }
    loft!(name, &secs, material = mat, at = at, rot = (90, 0, 0))
}

/// A flat-shaded mesh from game points and faces, its normals made to face out.
pub fn mesh(name: &str, verts: &[(f64, f64, f64)], faces: &[Vec<u32>], material: Mat) -> Obj {
    let vs: Vec<V3> = verts
        .iter()
        .map(|&(x, y, z)| F.at(x, y, z).into())
        .collect();
    let mut m = Mesh::from_pydata(vs, faces);
    m.set_material(Some(material));
    m.recalc_normals();
    scene::add(name, m)
}

/// The faces of a prism of two loops of `k` points: the first loop's, the second's (reversed), the sides.
pub fn prism_faces(k: usize) -> Vec<Vec<u32>> {
    let k32 = k as u32;
    let mut faces = vec![
        (0..k32).collect::<Vec<u32>>(),
        (k32..2 * k32).rev().collect(),
    ];
    for i in 0..k32 {
        faces.push(vec![i, (i + 1) % k32, k32 + (i + 1) % k32, k32 + i]);
    }
    faces
}

/// A hardback seen end on, as an outline (across, out) about the crown of its spine (0, 0), counter-clockwise: two
/// boards `b` thick running `depth` back from the crown, and the spine between them, rounded `bulge` proud of where
/// the boards' edges end and `sp` thick (the page block stands behind it), in `n` steps round.
pub fn hardback_end(t: f64, depth: f64, bulge: f64, b: f64, sp: f64, n: usize) -> Vec<(f64, f64)> {
    let h = t / 2.0;
    let mut pts = vec![
        (-h, -depth),
        (-h + b, -depth),
        (-h + b, -bulge - sp),
        (h - b, -bulge - sp),
        (h - b, -depth),
        (h, -depth),
        (h, -bulge),
    ];
    for k in 1..n {
        let a = PI * k as f64 / n as f64;
        pts.push((h * cos(a), -bulge + bulge * sin(a)));
    }
    pts.push((-h, -bulge));
    pts
}

/// The band round a hardback's spine (`hardback_end`'s crown), standing `e` proud of it and `w` deep into it.
pub fn spine_band(t: f64, bulge: f64, e: f64, w: f64, n: usize) -> Vec<(f64, f64)> {
    let h = t / 2.0;
    let arc = |r: f64, rb: f64, k: usize| {
        let a = PI * k as f64 / n as f64;
        (r * cos(a), -bulge + rb * sin(a))
    };
    let mut pts: Vec<(f64, f64)> = (0..=n).map(|k| arc(h + e, bulge + e, k)).collect();
    pts.extend((0..=n).rev().map(|k| arc(h - w, bulge - w, k)));
    pts
}

/// How a hardback is made: its boards' thickness, its spine's bulge for a book `t` thick, the spine's thickness
/// behind the bulge, how far the page block stands in from the boards' edges, and the steps round the spine.
pub const BOARD: f64 = 0.0022;
pub const SPINE: f64 = 0.002;
pub const PAGES_IN: f64 = 0.0025;
pub const SPINE_STEPS: usize = 4;
pub fn bulge(t: f64) -> f64 {
    (t * 0.2).clamp(0.003, 0.0075)
}

/// A hardback standing on `y`, `t` thick and `h` high, its spine's crown at `front` and `d` deep: boards and a
/// rounded spine in the cover's colour, the page block in pale paper set in from the boards' edges (its top a little
/// under theirs), and a gilt band round some spines.
#[allow(clippy::too_many_arguments)]
pub fn hardback(x: f64, y: f64, front: f64, t: f64, h: f64, d: f64, cover: Mat, band: bool) {
    let bu = bulge(t);
    let end = hardback_end(t, d, bu, BOARD, SPINE, SPINE_STEPS);
    fextrude!(
        F,
        "book",
        &end,
        h,
        (x, y + h / 2.0, front),
        cover,
        plane = Facing::Top,
        smooth = true
    );
    let pd = d - bu - SPINE - PAGES_IN;
    fbox!(
        F,
        "pages",
        t - 2.0 * BOARD - 0.0006,
        h - 2.0 * PAGES_IN,
        pd,
        (x, y + h / 2.0, front - bu - SPINE - pd / 2.0),
        M.pages
    );
    if band {
        fextrude!(
            F,
            "band",
            &spine_band(t, bu, 0.0006, 0.0012, SPINE_STEPS),
            0.012,
            (x, y + h * 0.82, front),
            M.gilt,
            plane = Facing::Top,
            smooth = true
        );
    }
}

/// One book standing on `y`, `t` thick, its spine at `front`: the cover, the page block just showing on top, and
/// a gilt band across some spines.
#[allow(clippy::too_many_arguments)]
pub fn book(x: f64, y: f64, front: f64, t: f64, h: f64, d: f64, cover: Mat, band: bool) {
    fbox!(F, "book", t, h, d, (x, y + h / 2.0, front - d / 2.0), cover);
    fbox!(
        F,
        "pages",
        t - 0.006,
        0.004,
        d - 0.012,
        (x, y + h, front - d / 2.0 - 0.004),
        M.pages
    );
    if band {
        fbox!(
            F,
            "band",
            t - 0.004,
            0.014,
            0.002,
            (x, y + h * 0.82, front + 0.0008),
            M.gilt
        );
    }
}

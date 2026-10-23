//! The head: its cage (part of the body's, see cage.rs) and the head interface (`head_frame`):
//! where the skull, face, eyes, mouth, brows and hairline of a family at a life stage are. The body
//! builds the head from the cage here, the face puts its features on the frame and the hair fits
//! over it, so each can be rebuilt on its own.
//!
//! The head cage is a grid of rows (`ROWS`, from the jaw up) and columns (`COLS`, degrees round the
//! head, 0 = the face, 90 = the resident's right) under a crown patch: 8 rings of 12 vertices round
//! the whole head, and over the face a finer grid of rows and columns that run only part of the way
//! (each ends on a whole one, so every face is a rectangle of grid cells: a quad, or a pentagon or
//! hexagon where a line ends on its side): columns beside the nose's middle (7 degrees) and at the
//! eyes' outer corners (40), rows at the mouth, at the nose's base, under the eyes and over them.
//! Its vertices lie where rays from the head's middle leave a smooth head (`head_shape`: a soft union
//! of a cranium, cheeks-and-jaw, a muzzle and a chin ellipsoid, rounder on small children); an ear is
//! pulled out of each side, and the features are offsets on the grid's vertices (`features`): the
//! nose (bridge, tip, wings and base), the upper lip and philtrum and a soft brow. body.rs draws the
//! smooth head between the grid's vertices too (`onto_head`) and carves the eye sockets into every
//! point of it (`sockets`): round each eyeball a shallow hollow its front stands out of, the lids over
//! it. There is no mouth hole: the lips and the mouth plate sit on the face. The LOD keeps the 8 x 12
//! grid (cage.rs `lod_faces`).
//!
//! `HeadFrame::sdf` and `surface_z` are measured on the real (subdivided) head and neck without
//! the ears, so the face features and the hair sit on what is drawn; `ears` bound the ears.
//! `centre` / `radii` are a loose ellipsoid round the skull for code that wants a simple one.

use std::collections::BTreeMap;
use std::sync::Arc;

use super::cage::{Cage, Tags, Weights};
use super::kdtree::KdTree;
use super::np::{PI, clip, linspace_open, mean, median, radians, sign};
use super::rig::{b, joints, proportions};
use super::{Cache, V3, key, v3};

/// Rows from the jaw up: elevation (degrees) of the ray from the head's middle (`MIDDLE`) at the
/// front and at the back, and how far round the head the row runs (degrees either side of the face;
/// 180: all round). The smooth head is a soft union of ellipsoids (`Shapes`): the cranium, the
/// cheeks and jaw, the muzzle, the chin; the rows' vertices are where the rays leave it.
const ROWS: [(f64, f64, f64); 12] = [
    (-52.0, -35.0, 180.0),
    (-42.0, -20.0, 180.0),
    (-27.0, -9.0, 28.0),
    (-20.0, -2.0, 180.0),
    (-14.0, 4.0, 28.0),
    (-8.0, 10.0, 180.0),
    (-1.0, 16.0, 50.0),
    (6.0, 22.0, 180.0),
    (14.0, 29.0, 50.0),
    (24.0, 36.0, 180.0),
    (46.0, 52.0, 180.0),
    (68.0, 70.0, 180.0),
];
/// The rows by name: under the chin, the chin, the mouth, the upper lip, the nose's base, its tip,
/// under the eyes, the eyes' middle, over the eyes, the brow (then the forehead and the top).
const JAW: usize = 0;
const MOUTH: usize = 2;
const LIP: usize = 3;
const BASE: usize = 4;
const TIP: usize = 5;
const UNDER: usize = 6;
const EYE: usize = 7;
const LID: usize = 8;
const BROW: usize = 9;
const TOP: usize = 11;
/// Columns round the head (degrees round its vertical axis; 0 = the face, + = the resident's right,
/// +x), in order, and the rows each runs through (first, last).
const COLS: [(f64, usize, usize); 16] = [
    (0.0, JAW, TOP),
    (7.0, MOUTH, LID),
    (14.0, JAW, TOP),
    (28.0, JAW, TOP),
    (40.0, TIP, BROW),
    (50.0, JAW, TOP),
    (92.0, JAW, TOP),
    (122.0, JAW, TOP),
    (180.0, JAW, TOP),
    (-122.0, JAW, TOP),
    (-92.0, JAW, TOP),
    (-50.0, JAW, TOP),
    (-40.0, TIP, BROW),
    (-28.0, JAW, TOP),
    (-14.0, JAW, TOP),
    (-7.0, MOUTH, LID),
];
/// The ear leaves through the face between the columns at 92 and 122 degrees (and their mirrors), from
/// the nose tip's row to the eyes'.
const EAR: (f64, f64) = (92.0, 122.0);
/// (height above the chin, z), fractions of hh.
const MIDDLE: (f64, f64) = (0.40, 0.0);
/// The eye sockets (`sockets`), eye radii: a hollow behind the eyeball, `SOCKET[0]` in front of the eye's
/// middle above and below it (where the lids cover it) and `SOCKET[1]` beside it (the eye's corners), a
/// bowl rising out from `SOCKET[2]` from the eye's axis (`SOCKET_WIDE` times that towards the nose and
/// towards the ear) as `c d^2 + SOCKET[5] d^4`, c from `SOCKET[3]` above and below to `SOCKET[4]` beside;
/// the face goes softly down into it (a smooth min over `SOCKET_SOFT`). The low-detail body's coarse face
/// has one vertex in each socket: it lies `SOCKET_LOD` in front of the eye's middle.
const SOCKET: [f64; 6] = [0.7, 0.3, 0.5, 1.0, 2.0, 0.5];
const SOCKET_WIDE: [f64; 2] = [1.0, 1.2];
const SOCKET_SOFT: f64 = 0.2;
const SOCKET_LOD: f64 = 0.4;
/// Under the lids the socket goes on down behind the eyeball (`sockets`): `SLOT[0]` eye radii behind its
/// surface, rising `SLOT[1]` per eye radius past its edge.
const SLOT: [f64; 2] = [0.06, 10.0];
/// The age folds' half-width (fractions of hh; `HeadFrame::fold`).
const FOLD_WIDTH: f64 = 0.02;

/// Per family: (centre, radii) of the cranium, the cheeks and jaw, the muzzle (the front of the face
/// from the nose down, so it stands up rather than sloping back to the chin), the chin (fractions of
/// hh from the chin and the Head joint's z), and the blend width.
struct Shapes {
    cranium: (V3, V3),
    cheeks: (V3, V3),
    muzzle: (V3, V3),
    chin: (V3, V3),
    blend: f64,
}

fn shapes(family: &str) -> Shapes {
    if family == "male" {
        Shapes {
            cranium: (v3(0.0, 0.56, -0.02), v3(0.375, 0.44, 0.445)),
            cheeks: (v3(0.0, 0.32, 0.07), v3(0.335, 0.31, 0.33)),
            muzzle: (v3(0.0, 0.2, 0.15), v3(0.19, 0.15, 0.225)),
            chin: (v3(0.0, 0.12, 0.26), v3(0.15, 0.11, 0.12)),
            blend: 0.14,
        }
    } else {
        Shapes {
            cranium: (v3(0.0, 0.56, -0.02), v3(0.37, 0.44, 0.44)),
            cheeks: (v3(0.0, 0.32, 0.07), v3(0.325, 0.30, 0.325)),
            muzzle: (v3(0.0, 0.2, 0.15), v3(0.18, 0.15, 0.22)),
            chin: (v3(0.0, 0.13, 0.25), v3(0.13, 0.10, 0.11)),
            blend: 0.14,
        }
    }
}

/// How far each feature's cage vertex moves (fractions of hh; the surface follows by about half).
fn nose(family: &str) -> f64 {
    if family == "male" { 0.085 } else { 0.07 }
}

fn brow(family: &str) -> f64 {
    if family == "male" { 0.025 } else { 0.014 }
}

/// The nose: (row, column's degrees, then the offset out to the side, up and forward), in `nose`
/// units, each side the same. The bridge rises from between the eyes to a round tip over the nose's
/// base; its wings flare beside the tip and the base.
const NOSE: [(usize, f64, [f64; 3]); 10] = [
    (LID, 0.0, [0.0, 0.0, 0.12]),
    (EYE, 0.0, [0.0, 0.0, 0.35]),
    (EYE, 7.0, [0.0, 0.0, -0.05]),
    (UNDER, 0.0, [0.0, 0.0, 0.8]),
    (UNDER, 7.0, [0.0, 0.0, 0.08]),
    (TIP, 0.0, [0.0, 0.0, 1.6]),
    (TIP, 7.0, [0.08, -0.04, 0.4]),
    (TIP, 14.0, [0.0, 0.0, 0.06]),
    (BASE, 0.0, [0.0, 0.0, 0.2]),
    (BASE, 7.0, [0.1, 0.0, 0.25]),
];

/// 0 for adult heads (1 / 6.5 of the height), 1 for babies (1 / 3.4).
pub fn youth(stage_hh: f64) -> f64 {
    clip((stage_hh - 1.0 / 6.5) / (1.0 / 3.4 - 1.0 / 6.5), 0.0, 1.0)
}

/// Approximate signed distance to an ellipsoid (exact on its surface).
fn ellipsoid_sd(p: V3, c: V3, r: V3) -> f64 {
    let q = (p - c).component_div(&r);
    let k0 = q.norm();
    let k1 = q.component_div(&r).norm();
    k0 * (k0 - 1.0) / k1.max(1e-9)
}

/// A smooth minimum of a and b (blended over k).
pub fn smin(a: f64, b: f64, k: f64) -> f64 {
    let h = clip(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    b + (a - b) * h - k * h * (1.0 - h)
}

/// The smooth head (no features) as a signed distance function of points in hh units (origin: the
/// chin's height and the Head joint's z).
pub fn head_shape(family: &str, stage: &str) -> impl Fn(V3) -> f64 {
    let pr = proportions(family, stage);
    let yg = youth(pr.hh);
    let s = shapes(family);
    let (cc, mut cr) = s.cranium;
    let (mut kc, mut kr) = s.cheeks;
    let (mut mc, mr) = s.muzzle;
    let (mut nc, mut nr) = s.chin;
    // Small children: a bigger, rounder cranium, chubby cheeks low on a short face, a flatter muzzle
    // and a small chin.
    cr += v3(0.035, 0.0, 0.02) * yg;
    kc += v3(0.0, -0.03, -0.01) * yg;
    kr += v3(0.04, 0.0, 0.0) * yg;
    mc += v3(0.0, -0.02, -0.03) * yg;
    nc += v3(0.0, 0.0, -0.02) * yg;
    nr *= 1.0 - 0.3 * yg;
    if stage == "elder" {
        kr += v3(0.008, 0.0, 0.0);
    }
    let bl = s.blend;
    move |p: V3| {
        let d = smin(ellipsoid_sd(p, cc, cr), ellipsoid_sd(p, kc, kr), bl);
        let d = smin(d, ellipsoid_sd(p, mc, mr), 0.08);
        smin(d, ellipsoid_sd(p, nc, nr), 0.09)
    }
}

/// Where a ray from inside (o) along unit direction d leaves the shape f (bisection).
fn cast(f: &impl Fn(V3) -> f64, o: V3, d: V3) -> V3 {
    let (mut lo, mut hi) = (0.0, 1.2);
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        if f(o + d * mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    o + d * lo
}

/// The eye sockets carved into the drawn head (body.rs, on every point of the surface rather than on
/// the cage, so they are as fine as the surface is; `lod`: on the low-detail body's): round each eye
/// the face goes softly down into a shallow hollow (`SOCKET`) the eye's front stands out of, deepest
/// at its corners, and on down behind the eyeball (`SLOT`), so the eye shows between the lids right into
/// their corners. The lids cover all of it (face.rs: they reach out to the face round it), and the skin
/// is shaded and its shade baked as it is without it (body.rs, build.rs).
pub fn sockets(family: &str, stage: &str, lod: bool) -> impl Fn(V3) -> V3 {
    let pr = proportions(family, stage);
    let j = joints(family, stage);
    let r = (0.075 + 0.02 * youth(pr.hh)) * pr.hh * pr.h;
    let eyes = [j[b("eye_l")], j[b("eye_r")]];
    let [over, side, r0, cv, ch, c4] = SOCKET;
    move |p: V3| {
        let mut z = p.z;
        for e in eyes {
            let q = (p - e) / r;
            if q.z <= 0.0 {
                continue;
            }
            // (0 above and below the eye, 1 beside it.)
            let across = q.x * q.x / (q.x * q.x + q.y * q.y + 0.1);
            let deep = if lod { SOCKET_LOD } else { over + (side - over) * across };
            let wide = SOCKET_WIDE[usize::from(q.x * e.x > 0.0)];
            let d = ((q.x / wide).hypot(q.y) - r0).max(0.0);
            let mut floor = deep + (cv + (ch - cv) * across) * d * d + c4 * d.powi(4);
            if !lod {
                let rho = q.x.hypot(q.y);
                floor = floor.min((1.0 - rho * rho).max(0.0).sqrt() - SLOT[0] + SLOT[1] * (rho - 1.0).max(0.0));
            }
            z = smin(z, e.z + r * floor, SOCKET_SOFT * r);
        }
        v3(p.x, p.y, z)
    }
}

/// Where the ray from the head's middle through a point (model space, metres) leaves the smooth head
/// (`head_shape`): what body.rs fits the drawn head to between the cage's vertices.
pub fn onto_head(family: &str, stage: &str) -> impl Fn(V3) -> V3 {
    let pr = proportions(family, stage);
    let hh = pr.hh * pr.h;
    let origin = v3(0.0, pr.h - hh, 0.0);
    let f = head_shape(family, stage);
    let o = v3(0.0, MIDDLE.0, MIDDLE.1);
    move |p: V3| cast(&f, o, ((p - origin) / hh - o).normalize()) * hh + origin
}

/// Whether row r and column c meet at a vertex (both run there).
fn at(r: usize, c: usize) -> bool {
    let (deg, r0, r1) = COLS[c];
    deg.abs() <= ROWS[r].2 && (r0..=r1).contains(&r)
}

/// Whether row r runs from column c to the next one.
fn row_runs(r: usize, c: usize) -> bool {
    let n = (c + 1) % COLS.len();
    COLS[c].0.abs() <= ROWS[r].2 && COLS[n].0.abs() <= ROWS[r].2
}

/// Whether column c runs from row r to the next one.
fn col_runs(c: usize, r: usize) -> bool {
    let (_, r0, r1) = COLS[c];
    r0 <= r && r < r1
}

/// The coarse grid (the LOD's): the rows and columns that run all round.
fn coarse_row(r: usize) -> bool {
    ROWS[r].2 >= 180.0
}

fn coarse_col(c: usize) -> bool {
    let (_, r0, r1) = COLS[c];
    r0 == 0 && r1 == ROWS.len() - 1
}

fn col_of(deg: f64) -> usize {
    COLS.iter().position(|c| c.0 == deg).expect("column")
}

/// The faces of the grid's cells (row r to r + 1, column c to c + 1), the ears' openings left out:
/// cells not parted by a row or column that runs between them make one face, a rectangle of cells
/// whose corners (and the vertices along its sides, where a line ends on it) are its corners.
/// `coarse`: only the lines that run all round (the LOD's grid).
fn grid_faces(ids: &[Vec<Option<usize>>], coarse: bool) -> Vec<Vec<usize>> {
    let (nr, nc) = (ROWS.len() - 1, COLS.len());
    let keep_r = |r: usize| !coarse || coarse_row(r);
    let keep_c = |c: usize| !coarse || coarse_col(c);
    let vert = |r: usize, c: usize| if keep_r(r) && keep_c(c) { ids[r][c] } else { None };
    // Union-find over the cells (cell k = r * nc + c).
    let mut parent: Vec<usize> = (0..nr * nc).collect();
    fn root(p: &mut [usize], mut k: usize) -> usize {
        while p[k] != k {
            p[k] = p[p[k]];
            k = p[k];
        }
        k
    }
    for r in 0..nr {
        for c in 0..nc {
            let k = r * nc + c;
            // The next column's cell, unless that column runs between them.
            let cn = (c + 1) % nc;
            if !(keep_c(cn) && col_runs(cn, r)) {
                let (a, bb) = (root(&mut parent, k), root(&mut parent, r * nc + cn));
                parent[a.max(bb)] = a.min(bb);
            }
            // The next row's cell, unless that row runs between them.
            if r + 1 < nr && !(keep_r(r + 1) && row_runs(r + 1, c)) {
                let (a, bb) = (root(&mut parent, k), root(&mut parent, (r + 1) * nc + c));
                parent[a.max(bb)] = a.min(bb);
            }
        }
    }
    let ears = [EAR, (-EAR.1, -EAR.0)].map(|(a, _)| col_of(a));
    let mut groups: BTreeMap<usize, Vec<(usize, usize)>> = BTreeMap::new();
    for r in 0..nr {
        for c in 0..nc {
            let g = root(&mut parent, r * nc + c);
            groups.entry(g).or_default().push((r, c));
        }
    }
    let mut out = Vec::new();
    for cells in groups.values() {
        let r0 = cells.iter().map(|q| q.0).min().unwrap();
        let r1 = cells.iter().map(|q| q.0).max().unwrap() + 1;
        // Columns: the cells' first, counting round from it (no face crosses the face's middle or
        // the back of the head, where whole columns run).
        let cs: Vec<usize> = cells.iter().map(|q| q.1).collect();
        let c0 = *cs.iter().min().unwrap();
        let c1 = cs.iter().max().unwrap() + 1;
        assert_eq!(cells.len(), (r1 - r0) * (c1 - c0), "head cage: a face that is not a rectangle of cells");
        if (r0..r1).contains(&TIP) && ears.contains(&c0) {
            continue;
        }
        assert!([(r0, c0), (r0, c1), (r1, c0), (r1, c1)].iter().all(|&(r, c)| vert(r, c % nc).is_some()), "head cage: a face without its corners");
        let c1w = c1 % nc;
        let mut f = Vec::new();
        f.extend((c0..c1).filter_map(|c| vert(r0, c)));
        f.extend((r0..r1).filter_map(|r| vert(r, c1w)));
        f.extend((c0 + 1..=c1).rev().filter_map(|c| vert(r1, c % nc)));
        f.extend((r0 + 1..=r1).rev().filter_map(|r| vert(r, c0)));
        out.push(f);
    }
    out
}

/// Offsets of the grid's vertices: the nose, the philtrum and the brow (see the module docs), metres.
fn features(family: &str, stage: &str) -> Vec<Vec<V3>> {
    let pr = proportions(family, stage);
    let hh = pr.hh * pr.h;
    let yg = youth(pr.hh);
    let mut dd = vec![vec![V3::zeros(); COLS.len()]; ROWS.len()];
    let z = v3(0.0, 0.0, 1.0);
    let young = 1.0 - 0.55 * yg;
    let n = nose(family) * young * hh * (if stage == "elder" { 1.1 } else { 1.0 });
    for (r, deg, [ox, oy, oz]) in NOSE {
        for sx in if deg == 0.0 { vec![1.0] } else { vec![1.0, -1.0] } {
            dd[r][col_of(sx * deg)] += v3(sx * ox, oy, oz) * n;
        }
    }
    // The upper lip rounded forward over the teeth (so it faces ahead under the nose), with the
    // philtrum, a soft groove, down its middle.
    for (deg, k) in [(0.0, 0.08), (7.0, 0.16), (-7.0, 0.16), (14.0, 0.1), (-14.0, 0.1)] {
        dd[LIP][col_of(deg)] += z * k * n;
    }
    let bw = brow(family) * (1.0 - 0.6 * yg) * hh;
    for deg in [14.0, 28.0, 40.0, 50.0] {
        for sx in [1.0, -1.0] {
            dd[BROW][col_of(sx * deg)] += z * bw;
        }
    }
    dd[BROW][0] += z * 0.5 * bw;
    dd
}

/// Adds the head to `cage`; returns its jaw ring (12 ids, the ring's columns in order) for the neck.
pub fn head_cage(cage: &mut Cage, family: &str, stage: &str) -> Vec<usize> {
    let pr = proportions(family, stage);
    let h = pr.h;
    let hh = pr.hh * h;
    let chin = h - hh;
    let yg = youth(pr.hh);
    let f = head_shape(family, stage);
    let o = v3(0.0, MIDDLE.0, MIDDLE.1);
    let w: Weights = vec![("Head", 1.0)];
    let origin = v3(0.0, chin, 0.0);
    // The smooth head at every row and column.
    let grid: Vec<Vec<V3>> = ROWS
        .iter()
        .map(|&(ef, eb, _)| {
            COLS.iter()
                .map(|&(deg, _, _)| {
                    let th = radians(deg);
                    let el = radians(ef + (eb - ef) * (1.0 - th.cos()) / 2.0);
                    let d = v3(el.cos() * th.sin(), el.sin(), el.cos() * th.cos());
                    cast(&f, o, d) * hh + origin
                })
                .collect()
        })
        .collect();
    let exists: Vec<Vec<bool>> = (0..ROWS.len()).map(|r| (0..COLS.len()).map(|c| at(r, c)).collect()).collect();
    let mut ids: Vec<Vec<Option<usize>>> = vec![vec![None; COLS.len()]; ROWS.len()];
    let dd = features(family, stage);
    for r in 0..ROWS.len() {
        for c in 0..COLS.len() {
            if !exists[r][c] {
                continue;
            }
            // Blush on the cheeks under the eyes and the nose's tip (tone is the base stage's for
            // every stage).
            let blush = if r == TIP {
                match COLS[c].0.abs() {
                    0.0 => 0.15,
                    28.0 => 0.32 + 0.2 * yg,
                    50.0 => 0.2 + 0.15 * yg,
                    _ => 0.0,
                }
            } else {
                0.0
            };
            ids[r][c] = Some(cage.add(grid[r][c], w.clone(), "head", Some(dd[r][c]), Tags { blush, ..Default::default() }));
        }
    }
    for fc in grid_faces(&ids, false) {
        cage.fine_face(&fc);
    }
    for fc in grid_faces(&ids, true) {
        cage.lod_face(&fc);
    }
    let ring = |r: usize| -> Vec<usize> { (0..COLS.len()).filter(|&c| coarse_col(c)).map(|c| ids[r][c].expect("ring")).collect() };
    let id = |r: usize, deg: f64| ids[r][col_of(deg)].expect("vertex");
    // Ears: from the face left open between columns 92 and 122 (and -122 .. -92), from the nose tip's
    // row to the eyes'. The smooth head has a low dome there (the skull the hair fits over); the ear
    // is the offset.
    for (c0, c1) in [EAR, (-EAR.1, -EAR.0)] {
        let lp = [id(TIP, c0), id(TIP, c1), id(EYE, c1), id(EYE, c0)];
        let p: Vec<V3> = lp.iter().map(|&i| cage.p[i]).collect();
        let m = super::np::mean_rows(&p);
        let sx = sign(m[0]);
        let out = v3(sx, 0.0, 0.0);
        let (mut base, mut offs) = (Vec::new(), Vec::new());
        for (i, &q0) in p.iter().enumerate() {
            let top = i >= 2;
            let front = if sx > 0.0 { i == 0 || i == 3 } else { i == 1 || i == 2 };
            let bpt = m + (q0 - m) * 0.6 + out * 0.012 * hh;
            let mut q = m + (q0 - m).component_mul(&v3(0.6, 1.0, 0.55));
            q += out * (0.04 + 0.015 * yg) * hh * (if front { 0.7 } else { 1.0 });
            q[1] += (if top { 0.085 } else { -0.06 }) * hh * (1.0 - 0.3 * yg);
            q[2] -= 0.015 * hh;
            base.push(bpt);
            offs.push(q - bpt);
        }
        let tags = Tags { blush: 0.35, ..Default::default() };
        let e: Vec<usize> = (0..4).map(|i| cage.add(base[i], w.clone(), "ear", Some(offs[i]), tags)).collect();
        cage.band(&lp, &e, &[]);
        cage.cap(&e);
    }
    // Crown: a 2 x 4 patch over the top row (right edge columns 14..122, left edge -14..-122), its
    // middle line on the head's top.
    let top = ring(TOP);
    let mut mid = Vec::new();
    for (el, az) in [(80.0, 0.0), (90.0, 0.0), (80.0, PI)] {
        let e = radians(el);
        let d = v3(f64::cos(e) * f64::sin(az), e.sin(), e.cos() * f64::cos(az));
        mid.push(cage.add(cast(&f, o, d) * hh + origin, w.clone(), "head", None, Tags::default()));
    }
    let right = [top[1], top[2], top[3], top[4], top[5]];
    let left = [top[11], top[10], top[9], top[8], top[7]];
    let centre = [top[0], mid[0], mid[1], mid[2], top[6]];
    for jj in 0..4 {
        cage.face(&[left[jj], centre[jj], centre[jj + 1], left[jj + 1]]);
        cage.face(&[centre[jj], right[jj], right[jj + 1], centre[jj + 1]]);
    }
    ring(JAW)
}

/// The head of a family at a life stage (model space at rest, metres).
pub struct HeadFrame {
    pub family: String,
    pub stage: String,
    /// Head height (chin to crown), chin height and the crown.
    pub hh: f64,
    pub chin_y: f64,
    pub top_y: f64,
    /// A loose ellipsoid round the skull: centre and semi-axes (x, y, z). It encloses the cranium
    /// with a margin (more front to back, clearing the brow and chin); fit close things to `sdf`.
    pub centre: V3,
    pub radii: V3,
    /// Face plane: z of the face surface at eye level (round the eye sockets).
    pub face_z: f64,
    /// Eyeball centres (left at -x, then right) and radius.
    pub eyes: [V3; 2],
    pub eye_r: f64,
    /// Iris and pupil radius (m, on the eye's front).
    pub iris_r: f64,
    pub pupil_r: f64,
    /// Mouth: centre of the lip line on the surface, and its width (corner to corner).
    pub mouth: V3,
    pub mouth_w: f64,
    /// Brows: per side (left, right), inner and outer end points on the surface.
    pub brows: [[V3; 2]; 2],
    /// Hairline: a closed loop of points on the skull (front, temples, behind the ears, nape).
    pub hairline: Vec<V3>,
    pub chin: V3,
    /// Ears (left, right): centre and half extents (x, y, z) of a box round each. They stand off
    /// the skull and are not part of `sdf` / `surface_z`; hair should go round or over them.
    pub ears: [[V3; 2]; 2],
    /// The real head and neck surface (dense samples, their normals, and its triangles).
    pub samples: Vec<V3>,
    pub normals: Vec<V3>,
    pub tris: Vec<[V3; 3]>,
    /// Its triangles as they would be without the eye sockets (the face the lids lie on, face.rs).
    pub bare: Vec<[V3; 3]>,
    /// The samples' KD-tree, their typical spacing and lowest height (head.py's module caches).
    tree: KdTree,
    spacing: f64,
    floor_y: f64,
}

impl HeadFrame {
    /// Signed distance (m; < 0 inside) of a point to the head and neck as drawn: the nearest
    /// samples' tangent-plane distances blended (moving least squares, Gaussian weights about
    /// the nearest one), so the field is continuous wherever it is evaluated (the ears are not
    /// part of the skull, see `ears`).
    pub fn sdf(&self, p: V3) -> f64 {
        let nb = self.tree.query(&p, 16);
        let sigma = 1.5 * self.spacing;
        let d0 = nb[0].0;
        let mut ws = [0.0; 16];
        let mut wv = [0.0; 16];
        for (j, &(d, i)) in nb.iter().enumerate() {
            let q = p - self.samples[i];
            let n = self.normals[i];
            let side = q.x * n.x + q.y * n.y + q.z * n.z;
            let w = (-(d * d - d0 * d0) / (sigma * sigma)).exp();
            ws[j] = w;
            wv[j] = w * side;
        }
        let mut s = super::np::pairwise_sum(&wv[..nb.len()]) / super::np::pairwise_sum(&ws[..nb.len()]);
        // Farther off, the distance to the nearest sample, with that sign (blended, so still smooth).
        let t = clip((d0 - sigma) / (2.0 * sigma), 0.0, 1.0) * clip(s.abs() / sigma, 0.0, 1.0);
        let t = t * t * (3.0 - 2.0 * t);
        s += t * (sign(s) * d0 - s);
        // The samples stop at the base of the neck: below that is outside.
        s.max(self.floor_y - p.y)
    }

    /// z of the head's front surface at (x, y) (the skull centre's z where (x, y) misses it).
    pub fn surface_z(&self, x: f64, y: f64) -> f64 {
        front_z(&self.tris, x, y, self.centre[2])
    }

    /// The age folds (0..1) at a point of the face: a soft line from beside each nose wing down past
    /// the mouth's corner (nasolabial), on the front of the face.
    pub fn fold(&self, p: V3) -> f64 {
        let q = (p.x.abs() / self.hh, (p.y - self.chin_y) / self.hh);
        let (a, b) = ((0.08, 0.31), (0.135, 0.15));
        let ab = (b.0 - a.0, b.1 - a.1);
        let t = clip(((q.0 - a.0) * ab.0 + (q.1 - a.1) * ab.1) / (ab.0 * ab.0 + ab.1 * ab.1), 0.0, 1.0);
        let d = (q.0 - a.0 - t * ab.0).hypot(q.1 - a.1 - t * ab.1);
        let front = clip((p.z - self.mouth.z + 0.1 * self.hh) / (0.05 * self.hh), 0.0, 1.0);
        (-(d / FOLD_WIDTH).powi(2)).exp() * front
    }

    /// Scalp weight (0..1) of a point: 1 well above the hairline, 0 on the face and neck.
    pub fn scalp(&self, p: V3) -> f64 {
        clip((p.y - hairline_y(self, p)) / (0.06 * self.hh) + 0.5, 0.0, 1.0)
    }
}

/// Height of the hairline above a point, by its direction around the head's axis.
pub fn hairline_y(h: &HeadFrame, p: V3) -> f64 {
    // 0 = front, pi = back.
    let a = (p.x - h.centre[0]).atan2(p.z - h.centre[2]);
    let t = a.abs() / PI;
    let (front, side, back) = (h.chin_y + 0.74 * h.hh, h.chin_y + 0.56 * h.hh, h.chin_y + 0.18 * h.hh);
    if t < 0.5 { front + (side - front) * (t / 0.5).powf(1.5) } else { side + (back - side) * ((t - 0.5) / 0.5) }
}

/// The largest z where the vertical line (x, y) crosses triangles; `miss` where none does.
pub fn front_z(tris: &[[V3; 3]], x: f64, y: f64, miss: f64) -> f64 {
    let mut out = f64::NEG_INFINITY;
    for [a, b, c] in tris {
        let lo = (a.x.min(b.x).min(c.x), a.y.min(b.y).min(c.y));
        let hi = (a.x.max(b.x).max(c.x), a.y.max(b.y).max(c.y));
        if !(lo.0 <= x && x <= hi.0 && lo.1 <= y && y <= hi.1) {
            continue;
        }
        let v0 = (b.x - a.x, b.y - a.y);
        let v1 = (c.x - a.x, c.y - a.y);
        let den = v0.0 * v1.1 - v1.0 * v0.1;
        if den.abs() <= 1e-14 {
            continue;
        }
        let v2 = (x - a.x, y - a.y);
        let u = (v2.0 * v1.1 - v1.0 * v2.1) / den;
        let v = (v0.0 * v2.1 - v2.0 * v0.1) / den;
        if u >= -1e-9 && v >= -1e-9 && u + v <= 1.0 + 1e-9 {
            let z = a.z + u * (b.z - a.z) + v * (c.z - a.z);
            out = out.max(z);
        }
    }
    if out.is_finite() { out } else { miss }
}

static HEADS: Cache<(String, String), HeadFrame> = Cache::new();

/// The head of `family` at life stage `stage` (see `HeadFrame`).
pub fn head_frame(family: &str, stage: &str) -> Arc<HeadFrame> {
    HEADS.get(key(family, stage), || make_head_frame(family, stage))
}

fn make_head_frame(family: &str, stage: &str) -> HeadFrame {
    // The head is drawn as part of the body.
    let srf = super::body::surface(family, stage);
    let pr = proportions(family, stage);
    let j = joints(family, stage);
    let (hgt, hh) = (pr.h, pr.hh * pr.h);
    let chin_y = hgt - hh;
    let yg = youth(pr.hh);
    let eyes = [j[b("eye_l")], j[b("eye_r")]];
    let eye_r = (0.075 + 0.02 * yg) * hh;
    // The cranium's ellipsoid: round the head above the ears (ears and face excluded).
    let cran = &srf.cranium;
    let zmin = cran.iter().map(|p| p.z).fold(f64::INFINITY, f64::min);
    let zmax = cran.iter().map(|p| p.z).fold(f64::NEG_INFINITY, f64::max);
    let xmax = cran.iter().map(|p| p.x.abs()).fold(f64::NEG_INFINITY, f64::max);
    let centre = v3(0.0, chin_y + 0.56 * hh, (zmin + zmax) / 2.0);
    let radii = v3(xmax, hgt - centre[1], (zmax - zmin) / 2.0);
    let k = cran.iter().map(|p| (p - centre).component_div(&radii).norm()).fold(f64::NEG_INFINITY, f64::max);
    let radii = (radii * k.max(1.0)).component_mul(&v3(1.05, 1.03, 1.12));
    let samples = srf.head_samples.clone();
    let tree = KdTree::new(&samples);
    // Typical distance between neighbouring surface samples.
    let second: Vec<f64> = samples.iter().map(|p| tree.query(p, 2)[1].0).collect();
    let spacing = median(&second);
    let floor_y = samples.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
    let mut h = HeadFrame {
        family: family.to_string(),
        stage: stage.to_string(),
        hh,
        chin_y,
        top_y: hgt,
        centre,
        radii,
        face_z: 0.0,
        eyes,
        eye_r,
        iris_r: eye_r * 0.5,
        pupil_r: eye_r * 0.2,
        mouth: V3::zeros(),
        mouth_w: 0.2 * hh,
        brows: [[V3::zeros(); 2]; 2],
        hairline: Vec::new(),
        chin: V3::zeros(),
        ears: srf.ears,
        samples,
        normals: srf.head_normals.clone(),
        tris: srf.head_tris.clone(),
        bare: srf.bare_tris.clone(),
        tree,
        spacing,
        floor_y,
    };
    let a = linspace_open(0.0, 2.0 * PI, 8);
    let rim: Vec<f64> = a.iter().map(|&a| {
        let p = eyes[1] + v3(a.cos(), a.sin(), 0.0) * 1.45 * eye_r;
        h.surface_z(p.x, p.y)
    }).collect();
    h.face_z = mean(&rim);
    let my = chin_y + 0.2 * hh;
    h.mouth = v3(0.0, my, h.surface_z(0.0, my));
    let by = chin_y + 0.6 * hh;
    for (k, sx) in [-1.0, 1.0].into_iter().enumerate() {
        let inner = v3(sx * 0.07 * hh, by, 0.0);
        let outer = v3(sx * 0.3 * hh, by - 0.02 * hh, 0.0);
        for (e, mut pt) in [inner, outer].into_iter().enumerate() {
            pt[2] = h.surface_z(pt[0], pt[1]);
            h.brows[k][e] = pt;
        }
    }
    // Hairline loop: on the surface, at the hairline's height in each direction round the head.
    let a = linspace_open(-PI, PI, 48);
    let mut pts = Vec::new();
    for &a in &a {
        let d = v3(a.sin(), 0.0, a.cos());
        let probe = centre + d.component_mul(&radii);
        let y = hairline_y(&h, probe);
        pts.push(ray(&h, v3(centre[0], y, centre[2]), d));
    }
    h.hairline = pts;
    h.chin = ray(&h, v3(0.0, chin_y + 0.03 * hh, centre[2]), v3(0.0, -0.45, 1.0) / 0.45f64.hypot(1.0));
    h
}

/// Where a ray from inside the head leaves its surface (bisection on the sdf).
fn ray(h: &HeadFrame, o: V3, d: V3) -> V3 {
    let (mut lo, mut hi) = (0.0, 0.5 * h.top_y);
    for _ in 0..40 {
        let mid = (lo + hi) / 2.0;
        if h.sdf(o + d * mid) < 0.0 {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    o + d * lo
}

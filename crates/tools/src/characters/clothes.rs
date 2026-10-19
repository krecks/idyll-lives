//! The residents' clothes. They are painted on the body by its shader from the body's fields and
//! each garment's cut (`garments` in rig.json, see docs/characters-format.md); shells are added
//! only where the silhouette needs them:
//!
//! - `top.jacket`, `top.suit`: the body's own surface over the region the cut covers (torso and
//!   sleeves), cut exactly along the fields' iso-lines, lifted 8-10 mm (more towards the hem,
//!   which flares a little), open in a V at the front (the lapels), with a rim turned back to the
//!   skin;
//! - `top.polo`, `top.shirt`: a turned-down collar round the neckline;
//! - `bottom.skirt`: an A-line loft from the waistband to above the knees, swinging with the thighs;
//! - `shoes.*`: a sole under each foot (darker), and a cuff round the shin at the top of `boots`.
//!
//! Contract: `garment_cuts(family, P)`: every id of `contract::garments`, in order, with its cut;
//! `build_clothes(family, stage, cuts)`: `top.jacket`, `lod.top.jacket`, ... (pos, nor, idx (CCW
//! out), dense, fields (hem distance, flag, category, 0), tone).
//!
//! Cut keys (metres on the base body's fields): top `sleeve` (sleeve parameter), `hem`, `crew` /
//! `scoop`, `over` (its hem is below the waistband: painted over the bottom); bottom `waist`,
//! `leg`, `skirt`; shoes `shoe` and `colour` (sRGB hex). Each shell is cut and laid out on the base
//! body, so it has the same vertices at every life stage (stages are deltas).

use std::sync::Arc;

use serde::Serialize;

use super::body::{fields_of, region, surface, topology, tris};
use super::contract::garments;
use super::garments::{ARM_M, BOTTOM, FLAG_SKIRT, FLAG_SOLE, Landmarks, SHOES, TOP};
use super::head::head_frame;
use super::json::OMap;
use super::kdtree::KdTree;
use super::meshops::{Src, boundary_edges, clip_by_field, smooth_normals};
use super::np::{PI, clip, linspace, linspace_open, mean, percentile, py_mod, py_round, radians, sign, smoothstep, unique};
use super::rig::{BASE_STAGE, NB, b, proportions};
use super::{Cache, Dense, Mesh, V3, v3};

fn shoe_colour(name: &str) -> &'static str {
    match name {
        "sneakers" => "#EDEBE6",
        "trainers" => "#3D4B66",
        "boots" => "#5B3B28",
        _ => "#221C19",
    }
}

/// Shells: lift off the skin (m).
fn jacket_lift(gid: &str) -> f64 {
    if gid == "top.jacket" { 0.009 } else { 0.008 }
}
/// The jacket shells' sleeves end (sleeve parameter) on the upper arm; the rest is painted.
const SHELL_SLEEVE: f64 = 0.42;
const SKIRT_SEGS: usize = 24;
const SKIRT_ROWS: usize = 7;

/// A garment's cut (the keys a kind has, in rig.json's order).
#[derive(Clone, Debug, Default, Serialize)]
pub struct Cut {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleeve: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hem: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crew: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scoop: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub over: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waist: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skirt: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shoe: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colour: Option<&'static str>,
}

pub type Cuts = OMap<Cut>;

pub fn garment_cuts(family: &str, p: &[V3; NB]) -> Cuts {
    let lm = Landmarks::new(p, family);
    let thigh = lm.hip - lm.knee;
    let shin = lm.knee - lm.ankle;
    let hem = lm.hem;
    let top = |sleeve: f64, hem: f64, crew: Option<f64>, scoop: Option<f64>, over: bool| Cut { sleeve: Some(sleeve), hem: Some(hem), crew, scoop, over: over.then_some(true), ..Cut::default() };
    let bottom = |waist: f64, leg: f64, skirt: bool| Cut { waist: Some(waist), leg: Some(leg), skirt: skirt.then_some(true), ..Cut::default() };
    let mut out = Cuts::new();
    for &gid in garments(family) {
        let (kind, name) = gid.split_once('.').unwrap();
        let c = match (kind, name) {
            ("top", "tee") => top(0.3, hem, Some(0.0), None, false),
            ("top", "vneck") => top(0.3, hem, None, Some(0.0), false),
            ("top", "polo") => top(0.36, hem - 0.02, Some(-0.006), None, false),
            ("top", "long") => top(0.965, hem, Some(0.0), None, false),
            ("top", "shirt") => top(0.965, hem - 0.07, Some(-0.012), None, true),
            ("top", "jacket") => top(0.975, lm.hip + 0.025, Some(0.0), None, true),
            ("top", "suit") => top(0.975, lm.hip + 0.015, Some(-0.012), None, true),
            ("top", "tank") => top(-0.62, hem, None, Some(0.0), false),
            ("top", "blouse") => top(0.5, hem, None, Some(0.0), false),
            ("bottom", "trousers") => bottom(lm.waist, lm.ankle + 0.03, false),
            ("bottom", "shorts") => bottom(lm.waist, lm.knee + 0.45 * thigh, false),
            ("bottom", "suit") => bottom(lm.waist + 0.01, lm.ankle + 0.02, false),
            ("bottom", "capri") => bottom(lm.waist, lm.knee - 0.45 * shin, false),
            ("bottom", "skirt") => bottom(lm.waist, lm.knee + 0.05, true),
            ("shoes", n) => {
                let shoe = match n {
                    "sneakers" => lm.ankle + 0.03,
                    "trainers" => lm.ankle + 0.035,
                    "boots" => lm.ankle + 0.13,
                    _ => lm.ankle + 0.012,
                };
                Cut { shoe: Some(shoe), colour: Some(shoe_colour(n)), ..Cut::default() }
            }
            _ => panic!("no garment {gid}"),
        };
        // Floats rounded to 4 places (Python's round).
        let r = |v: Option<f64>| v.map(|v| py_round(v, 4));
        let c = Cut { sleeve: r(c.sleeve), hem: r(c.hem), crew: r(c.crew), scoop: r(c.scoop), waist: r(c.waist), leg: r(c.leg), shoe: r(c.shoe), ..c };
        out.insert(gid, c);
    }
    out
}

/// Signed coverage (m, > 0 covered) of a top's cut over body fields (as the shader paints it).
pub fn top_coverage(f: &[f64; 4], cut: &Cut) -> f64 {
    let mut g = ((cut.sleeve.unwrap() - f[0]) * ARM_M).min(f[1] - cut.hem.unwrap());
    if let Some(c) = cut.crew {
        g = g.min(f[2] - c);
    }
    if let Some(s) = cut.scoop {
        g = g.min(f[3] - s);
    }
    g
}

type Fields = (Vec<[f64; 4]>, Vec<[f64; 4]>);
static BASE_FIELDS: Cache<String, Fields> = Cache::new();

/// The base body's fields and its LOD's.
fn base_fields(family: &str) -> Arc<Fields> {
    BASE_FIELDS.get(family.to_string(), || {
        let s = surface(family, BASE_STAGE);
        let h = head_frame(family, BASE_STAGE);
        let f = fields_of(family, &s.pos, &s.tags.arm, &s.region, &s.p, &h);
        let fl = fields_of(family, &s.lod_pos, &s.lod_tags.arm, &s.lod_region, &s.p, &h);
        (f, fl)
    })
}

/// Values lerped along each new vertex's source edge.
fn lerp_src<T: Copy + std::ops::Mul<f64, Output = T> + std::ops::Add<Output = T>>(src: &Src, arr: &[T]) -> Vec<T> {
    let (a, bv, t) = src;
    (0..a.len()).map(|i| arr[a[i]] * (1.0 - t[i]) + arr[bv[i]] * t[i]).collect()
}

fn shell_dense(src: &Src, dense: &[Dense]) -> Vec<Dense> {
    let (a, bv, t) = src;
    (0..a.len())
        .map(|i| {
            let mut d = [0.0; NB];
            for k in 0..NB {
                d[k] = dense[a[i]][k] * (1.0 - t[i]) + dense[bv[i]][k] * t[i];
            }
            d
        })
        .collect()
}

/// The jacket's piece of the base body: clipped faces and their source edges (stage independent).
struct Layout {
    src: Src,
    idx: Vec<usize>,
    hem_y: Vec<f64>,
    sleeve: Vec<f64>,
    cut: Cut,
}

static LAYOUTS: Cache<(String, String, bool), Layout> = Cache::new();

fn jacket_layout(family: &str, gid: &str, lod: bool) -> Arc<Layout> {
    LAYOUTS.get((family.to_string(), gid.to_string(), lod), || {
        let s = surface(family, BASE_STAGE);
        let fl = base_fields(family);
        let cuts = garment_cuts(family, &s.p);
        let cut = cuts.get(gid).unwrap().clone();
        let (pos, idx, ff): (&[V3], Vec<usize>, &[[f64; 4]]) = if lod { (&s.lod_pos, topology().lod_tris.concat(), &fl.1) } else { (&s.pos, tris(family).to_vec(), &fl.0) };
        // The lapels: open in a V at the front, from the neckline down to the button.
        let p = &s.p;
        let zn = p[b("neck_01")].z;
        let y_button = p[b("spine_02")].y + if gid == "top.suit" { 0.02 } else { 0.0 };
        let y_top = p[b("neck_01")].y - 0.01;
        let open_w = radians(46.0);
        let g: Vec<f64> = pos
            .iter()
            .zip(ff)
            .map(|(q, f)| {
                let g = top_coverage(f, &cut);
                let th = q.x.atan2(q.z - zn);
                let vline = y_button + (y_top - y_button) * clip(th.abs() / open_w, 0.0, 1.0).powf(1.2);
                // The shell's sleeves stop on the upper arm, sinking into the painted sleeve.
                g.min(vline - q.y).min((SHELL_SLEEVE - f[0]) * ARM_M)
            })
            .collect();
        let (cidx, src) = clip_by_field(&idx, &g);
        let f1: Vec<f64> = ff.iter().map(|f| f[1]).collect();
        let f0: Vec<f64> = ff.iter().map(|f| f[0]).collect();
        let hem_y = lerp_src(&src, &f1);
        let sleeve = lerp_src(&src, &f0);
        Layout { src, idx: cidx, hem_y, sleeve, cut }
    })
}

/// A band turned back from each open edge towards the skin (the cloth's thickness); edges with
/// both ends in `skip` get none.
fn rim(pos: &[V3], nor: &[V3], idx: &[usize], depth: f64, skip: &[bool]) -> (Vec<V3>, Vec<usize>, Vec<usize>) {
    let be: Vec<[usize; 2]> = boundary_edges(idx).into_iter().filter(|e| !(skip[e[0]] && skip[e[1]])).collect();
    let verts = unique(&be.concat());
    let mut newi = vec![usize::MAX; pos.len()];
    for (i, &v) in verts.iter().enumerate() {
        newi[v] = pos.len() + i;
    }
    let mut p = pos.to_vec();
    p.extend(verts.iter().map(|&v| pos[v] - nor[v] * depth));
    let mut out = idx.to_vec();
    for [a, bb] in be {
        out.extend([bb, a, newi[a], bb, newi[a], newi[bb]]);
    }
    (p, out, verts)
}

fn jacket(family: &str, stage: &str, gid: &str, lod: bool) -> Mesh {
    let lay = jacket_layout(family, gid, lod);
    let s = surface(family, stage);
    let (bpos, bidx, dense): (&[V3], Vec<usize>, &[Dense]) = if lod { (&s.lod_pos, topology().lod_tris.concat(), &s.lod_dense) } else { (&s.pos, tris(family).to_vec(), &s.dense) };
    let bnor = smooth_normals(bpos, &bidx);
    let pos = lerp_src(&lay.src, bpos);
    let nor: Vec<V3> = lerp_src(&lay.src, &bnor).iter().map(|n| n / n.norm()).collect();
    let cut = &lay.cut;
    let h = proportions(family, stage).h;
    let k = h / 1.7;
    // Lift: thicker over the hips (the flare of the hem) and a little looser on the sleeves.
    let hem = cut.hem.unwrap();
    let mut pos: Vec<V3> = (0..pos.len())
        .map(|i| {
            let flare = smoothstep(hem + 0.14, hem, lay.hem_y[i]);
            let taper = smoothstep(SHELL_SLEEVE, SHELL_SLEEVE - 0.24, lay.sleeve[i]);
            let lift = (jacket_lift(gid) + 0.012 * flare) * k * taper;
            pos[i] + nor[i] * lift
        })
        .collect();
    let cuts = garment_cuts(family, &surface(family, BASE_STAGE).p);
    if cuts.contains("bottom.skirt") {
        // The hem lies over a skirt (it is a shell too).
        pos = over_skirt(&pos, family, stage, &cuts, 0.005 * k);
    }
    let n0 = pos.len();
    let skip: Vec<bool> = lay.sleeve.iter().map(|&v| v > SHELL_SLEEVE - 0.06).collect();
    let (pos, idx, verts) = rim(&pos, &nor, &lay.idx, 0.6 * jacket_lift(gid) * k, &skip);
    let nor_all = smooth_normals(&pos, &idx);
    let mut d = shell_dense(&lay.src, dense);
    let extra: Vec<Dense> = verts.iter().map(|&v| d[v]).collect();
    d.extend(extra);
    let n = pos.len();
    let fields = (0..n).map(|i| [if i < n0 { 0.05 } else { 0.0 }, 0.0, TOP, 0.0]).collect();
    Mesh { pos, nor: nor_all, idx, dense: d, fields, tone: vec![[0.0; 4]; n], ..Mesh::default() }
}

/// Triangles turned to face away from a vertical axis through `centre` (x, z) where most don't.
fn outward(pos: &[V3], idx: Vec<usize>, centre: (f64, f64)) -> Vec<usize> {
    let mut wrong = 0usize;
    let nt = idx.len() / 3;
    for t in idx.chunks(3) {
        let n = (pos[t[1]] - pos[t[0]]).cross(&(pos[t[2]] - pos[t[0]]));
        let c = (pos[t[0]] + pos[t[1]] + pos[t[2]]) / 3.0;
        let radial = v3(c.x - centre.0, 0.0, c.z - centre.1);
        if n.x * radial.x + n.y * radial.y + n.z * radial.z < 0.0 {
            wrong += 1;
        }
    }
    if wrong as f64 / nt as f64 > 0.5 { flip(&idx) } else { idx }
}

fn flip(idx: &[usize]) -> Vec<usize> {
    idx.chunks(3).flat_map(|t| [t[0], t[2], t[1]]).collect()
}

/// A closed mesh's triangles, turned round if they enclose a negative volume.
fn closed_outward(pos: &[V3], idx: Vec<usize>) -> Vec<usize> {
    let v: Vec<f64> = idx.chunks(3).map(|t| pos[t[0]].dot(&pos[t[1]].cross(&pos[t[2]]))).collect();
    if super::np::pairwise_sum(&v) < 0.0 { flip(&idx) } else { idx }
}

/// Points where the mesh's edges (of triangles whose corners are all in `keep`) cross height y.
fn section(pos: &[V3], tris: &[usize], y: f64, keep: &[bool]) -> Vec<V3> {
    let t: Vec<&[usize]> = tris.chunks(3).filter(|t| t.iter().all(|&v| keep[v])).collect();
    let mut out = Vec::new();
    for (i, j) in [(0, 1), (1, 2), (2, 0)] {
        for tt in &t {
            let (a, bb) = (pos[tt[i]], pos[tt[j]]);
            if (a.y - y) * (bb.y - y) < 0.0 {
                let f = (y - a.y) / (bb.y - a.y);
                out.push(a + (bb - a) * f);
            }
        }
    }
    out
}

/// Skin weights from the nearest body vertices.
fn follow(pos: &[V3], s: &super::body::Surface) -> Vec<Dense> {
    let tree = KdTree::new(&s.pos);
    pos.iter()
        .map(|p| {
            let nb = tree.query(p, 4);
            let w: Vec<f64> = nb.iter().map(|&(d, _)| 1.0 / d.max(1e-4)).collect();
            let ws = w.iter().fold(0.0, |a, v| a + v);
            let mut d = [0.0; NB];
            for (&(_, i), wi) in nb.iter().zip(&w) {
                let wi = wi / ws;
                for (dk, sk) in d.iter_mut().zip(&s.dense[i]) {
                    *dk += sk * wi;
                }
            }
            d
        })
        .collect()
}

/// Quads of a grid of points (rows along the strip's length).
fn strip(rows: &[Vec<V3>], closed: bool) -> (Vec<V3>, Vec<usize>) {
    let (r, c) = (rows.len(), rows[0].len());
    let mut idx = Vec::new();
    let cols = if closed { c } else { c - 1 };
    for i in 0..r - 1 {
        for j in 0..cols {
            let (a, bb) = (i * c + j, i * c + (j + 1) % c);
            idx.extend([a, bb, bb + c, a, bb + c, a + c]);
        }
    }
    (rows.iter().flatten().copied().collect(), idx)
}

/// A turned-down collar on the neckline (open at the front).
fn collar(family: &str, stage: &str, gid: &str, cuts: &Cuts) -> Mesh {
    let s = surface(family, stage);
    let sb = surface(family, BASE_STAGE);
    let p = &s.p;
    let cut = cuts.get(gid).unwrap();
    let fl = base_fields(family);
    let f = &fl.0;
    // Points on the neckline of the base body (crew field = cut), carried to the stage by the
    // nearest base vertices.
    let zn = sb.p[b("neck_01")].z;
    let th: Vec<f64> = linspace(28.0, 332.0, 15).iter().map(|&d| radians(d)).collect();
    let (rh, re) = (region("head"), region("ear"));
    let crew = cut.crew.unwrap();
    let cand: Vec<usize> = (0..sb.pos.len()).filter(|&i| (f[i][2] - crew).abs() < 0.012 && sb.region[i][rh] + sb.region[i][re] < 0.5 && sb.tags.arm[i] < 0.3).collect();
    let ang: Vec<f64> = cand.iter().map(|&i| py_mod(sb.pos[i].x.atan2(sb.pos[i].z - zn), 2.0 * PI)).collect();
    let picks: Vec<[usize; 3]> = th
        .iter()
        .map(|&t| {
            let da: Vec<f64> = ang.iter().map(|&a| (py_mod(a - t + PI, 2.0 * PI) - PI).abs()).collect();
            let o = super::np::argsort(&da);
            [cand[o[0]], cand[o[1]], cand[o[2]]]
        })
        .collect();
    let base_pts: Vec<V3> = picks.iter().map(|pk| (s.pos[pk[0]] + s.pos[pk[1]] + s.pos[pk[2]]) / 3.0).collect();
    let sn = smooth_normals(&s.pos, &surface_tris(family));
    let nor: Vec<V3> = picks.iter().map(|pk| (sn[pk[0]] + sn[pk[1]] + sn[pk[2]]) / 3.0).map(|n| n / n.norm()).collect();
    let up = v3(0.0, 1.0, 0.0);
    let h = proportions(family, stage).h;
    let k = h / 1.7;
    let shirt = gid == "top.shirt";
    let stand = (if shirt { 0.022 } else { 0.018 }) * k;
    let fall = (if shirt { 0.034 } else { 0.028 }) * k;
    let (mut r0, mut r1, mut r2) = (Vec::new(), Vec::new(), Vec::new());
    for i in 0..th.len() {
        let front = clip(th[i].cos(), 0.0, 1.0);
        let a = base_pts[i] + nor[i] * 0.003 * k;
        let bb = a + up * stand + nor[i] * 0.004 * k;
        let c = bb + nor[i] * 0.012 * k - up * (fall * (1.0 + 0.5 * (front * front))) + nor[i] * (0.01 * front) * k;
        r0.push(a);
        r1.push(bb);
        r2.push(c);
    }
    let (pos, idx) = strip(&[r0, r1, r2], false);
    let idx = outward(&pos, idx, (0.0, p[b("neck_01")].z));
    let mut dense = follow(&pos, &s);
    // The collar's back is on the midline, where the nearest body vertices come in mirror pairs:
    // its weights are made exactly symmetric (each bone and its mirror get the mean), so rounding
    // noise can't favour a side. Of a clavicle pair tied for the last of the four joints, `top4`
    // then keeps one by its fixed order, as on the body's own midline.
    let mirror = super::rig::mirror_bones();
    for (q, d) in pos.iter().zip(dense.iter_mut()) {
        if q.x.abs() < 1e-9 {
            let m = *d;
            for (bi, v) in d.iter_mut().enumerate() {
                *v = (m[bi] + m[mirror[bi]]) / 2.0;
            }
        }
    }
    let nor = smooth_normals(&pos, &idx);
    let n = pos.len();
    let nt = th.len();
    let fields = (0..n).map(|i| [[0.03, 0.0, 0.0][i / nt], 0.0, TOP, 0.0]).collect();
    Mesh { pos, idx, nor, dense, fields, tone: vec![[0.0; 4]; n], ..Mesh::default() }
}

fn surface_tris(family: &str) -> Arc<Vec<usize>> {
    tris(family)
}

/// The skirt's rows (heights), axis z, angles and radii (rows x segs).
struct SkirtProfile {
    ys: Vec<f64>,
    cz: f64,
    ang: Vec<f64>,
    r: Vec<Vec<f64>>,
}

static SKIRTS: Cache<(String, String, u64, usize, usize), SkirtProfile> = Cache::new();

/// The body's sections (torso and legs, sampled between the rows) plus ease, snug over the hips
/// and flaring below.
fn skirt_profile(family: &str, stage: &str, waist: f64, segs: usize, rows: usize) -> Arc<SkirtProfile> {
    SKIRTS.get((family.to_string(), stage.to_string(), waist.to_bits(), segs, rows), || {
        let s = surface(family, stage);
        let sb = surface(family, BASE_STAGE);
        let p = &s.p;
        let pb = &sb.p;
        // Heights on the base body, carried to the stage proportionally between the hips and knees.
        let (yb0, yb1) = (waist + 0.004, pb[b("calf_l")].y + 0.07);
        let stage_y = |yb: f64| {
            let t = (yb - pb[b("calf_l")].y) / (pb[b("thigh_l")].y - pb[b("calf_l")].y);
            p[b("calf_l")].y + t * (p[b("thigh_l")].y - p[b("calf_l")].y)
        };
        let ys: Vec<f64> = (0..rows).map(|i| stage_y(yb0 + (yb1 - yb0) * (i as f64 / (rows - 1) as f64))).collect();
        let cz = p[b("pelvis")].z;
        let ang = linspace_open(0.0, 2.0 * PI, segs);
        let (rt, rl) = (region("torso"), region("leg"));
        let keep: Vec<bool> = (0..s.pos.len()).map(|i| s.tags.arm[i] < 0.1 && s.region[i][rt] + s.region[i][rl] > 0.5).collect();
        let tri = surface_tris(family);
        let step = (ys[0] - ys[rows - 1]) / (rows - 1) as f64;
        let mut r = vec![vec![0.0; segs]; rows];
        let width = PI / segs as f64 * 1.6;
        for (i, &y) in ys.iter().enumerate() {
            let mut offs = vec![-0.5, -0.25, 0.0, 0.25, 0.5];
            if i == 0 {
                offs.push(0.75);
            }
            let pts: Vec<V3> = offs.iter().flat_map(|&f| section(&s.pos, &tri, y + f * step, &keep)).collect();
            let va: Vec<f64> = pts.iter().map(|q| py_mod(q.x.atan2(q.z - cz), 2.0 * PI)).collect();
            let vr: Vec<f64> = pts.iter().map(|q| q.x.hypot(q.z - cz)).collect();
            for (j, &a) in ang.iter().enumerate() {
                let m = va.iter().zip(&vr).filter(|&(&v, _)| (py_mod(v - a + PI, 2.0 * PI) - PI).abs() < width).map(|(_, &r)| r).fold(f64::NEG_INFINITY, f64::max);
                r[i][j] = if m.is_finite() { m } else if i > 0 { r[i - 1][j] } else { 0.1 };
            }
        }
        let r: Vec<Vec<f64>> = r.iter().map(|row| (0..segs).map(|j| row[j].max((row[(j + segs - 1) % segs] + 2.0 * row[j] + row[(j + 1) % segs]) / 4.0)).collect()).collect();
        let k = proportions(family, stage).h / 1.7;
        let t = linspace(0.0, 1.0, rows);
        let top: Vec<f64> = r[0].iter().map(|v| v + 0.006 * k).collect();
        let hem: Vec<f64> = (0..segs).map(|j| r.iter().map(|row| row[j]).fold(f64::NEG_INFINITY, f64::max) + 0.05 * k).collect();
        let r = (0..rows).map(|i| (0..segs).map(|j| (r[i][j] + 0.006 * k).max(top[j] + (hem[j] - top[j]) * clip((t[i] - 0.2) / 0.8, 0.0, 1.0).powf(0.9))).collect()).collect();
        SkirtProfile { ys, cz, ang, r }
    })
}

/// Points pushed out from the pelvis' axis to lie `lift` outside the skirt where it is.
fn over_skirt(pos: &[V3], family: &str, stage: &str, cuts: &Cuts, lift: f64) -> Vec<V3> {
    let sp = skirt_profile(family, stage, cuts.get("bottom.skirt").unwrap().waist.unwrap(), SKIRT_SEGS, SKIRT_ROWS);
    let (ys, cz, na) = (&sp.ys, sp.cz, sp.ang.len());
    let ny = ys.len();
    pos.iter()
        .map(|q| {
            let a = py_mod(q.x.atan2(q.z - cz), 2.0 * PI);
            let r = q.x.hypot(q.z - cz);
            let j = a / (2.0 * PI) * na as f64;
            let j0 = (j.floor() as usize) % na;
            let f = j - j.floor();
            let yi = clip((ys[0] - q.y) / (ys[0] - ys[ny - 1]) * (ny - 1) as f64, 0.0, (ny - 1) as f64);
            let i0 = (yi.floor() as usize).min(ny - 2);
            let g = yi - i0 as f64;
            let rr = |ii: usize, jj: usize| sp.r[ii % ny][jj % na];
            let rv = 0.0 + (1.0 - g) * (1.0 - f) * rr(i0, j0) + (1.0 - g) * f * rr(i0, j0 + 1) + g * (1.0 - f) * rr(i0 + 1, j0) + g * f * rr(i0 + 1, j0 + 1);
            let inside = q.y <= ys[0] + 0.03 && q.y >= ys[ny - 1];
            let need = if inside { (rv + lift - r).max(0.0) } else { 0.0 };
            q + v3(a.sin(), 0.0, a.cos()) * need
        })
        .collect()
}

/// An A-line skirt from the waistband to above the knees.
fn skirt_shell(family: &str, stage: &str, cuts: &Cuts, segs: usize, rows: usize) -> Mesh {
    let cut = cuts.get("bottom.skirt").unwrap();
    let sp = skirt_profile(family, stage, cut.waist.unwrap(), segs, rows);
    let (ys, cz, ang) = (&sp.ys, sp.cz, &sp.ang);
    let k = proportions(family, stage).h / 1.7;
    let (yb0, yb1) = (ys[0], ys[rows - 1]);
    let g: Vec<Vec<V3>> = (0..rows).map(|i| (0..segs).map(|j| v3(ang[j].sin() * sp.r[i][j], ys[i], cz + ang[j].cos() * sp.r[i][j])).collect()).collect();
    // Hem: a band turned up inside the bottom edge (the cloth's thickness).
    let hem_in: Vec<V3> = (0..segs).map(|j| g[rows - 1][j] - v3(ang[j].sin(), 0.0, ang[j].cos()) * 0.004 * k + v3(0.0, 0.012 * k, 0.0)).collect();
    let (mut pos, mut idx) = strip(&g, true);
    let (hp, hi) = strip(&[g[rows - 1].clone(), hem_in], true);
    pos.extend(hp);
    idx.extend(hi.iter().map(|&i| i + rows * segs));
    let n = pos.len();
    let tl = linspace(0.0, 1.0, rows);
    let t: Vec<f64> = (0..n).map(|i| if i < rows * segs { tl[i / segs] } else { 1.0 }).collect();
    let side: Vec<f64> = (0..n).map(|i| ang[i % segs].sin()).collect();
    let (tr, tlf, pv) = (b("thigh_r"), b("thigh_l"), b("pelvis"));
    let dense = (0..n)
        .map(|i| {
            let tw = smoothstep(0.1, 1.0, t[i]) * 0.9;
            let sr = clip(0.5 + side[i] * 0.9, 0.0, 1.0);
            let mut d = [0.0; NB];
            d[tr] = tw * sr;
            d[tlf] = tw * (1.0 - sr);
            d[pv] = 1.0 - tw;
            d
        })
        .collect();
    let fields = (0..n).map(|i| [(1.0 - t[i]) * (yb0 - yb1), FLAG_SKIRT, BOTTOM, 0.0]).collect();
    let body_n = (rows - 1) * segs * 6;
    let tri = outward(&pos[..rows * segs], idx[..body_n].to_vec(), (0.0, cz));
    let tri = if tri != idx[..body_n] { flip(&idx) } else { idx };
    let nor = smooth_normals(&pos, &tri);
    Mesh { pos, idx: tri, nor, dense, fields, tone: vec![[0.0; 4]; n], ..Mesh::default() }
}

/// A closed prism: the outline (x, z) from y0 to y1, capped (counter-clockwise outside).
fn prism(outline: &[(f64, f64)], y0: f64, y1: f64, centre: (f64, f64)) -> (Vec<V3>, Vec<usize>) {
    let k = outline.len();
    let mut pos: Vec<V3> = outline.iter().map(|&(x, z)| v3(x, y0, z)).collect();
    pos.extend(outline.iter().map(|&(x, z)| v3(x, y1, z)));
    pos.push(v3(centre.0, y0, centre.1));
    pos.push(v3(centre.0, y1, centre.1));
    let mut idx = Vec::new();
    for j in 0..k {
        let (a, bb) = (j, (j + 1) % k);
        idx.extend([a, bb, bb + k, a, bb + k, a + k]);
        idx.extend([2 * k, bb, a]);
        idx.extend([2 * k + 1, a + k, bb + k]);
    }
    (pos, idx)
}

/// A sole under each foot (a rounded slab a little wider than the foot), and boots' cuffs.
fn soles(family: &str, stage: &str, gid: &str, cuts: &Cuts, lod: bool) -> Mesh {
    let s = surface(family, stage);
    let p = &s.p;
    let h = proportions(family, stage).h;
    let k = h / 1.7;
    let thick = match gid {
        "shoes.sneakers" => 0.016,
        "shoes.trainers" => 0.018,
        "shoes.boots" => 0.02,
        _ => 0.011,
    } * k;
    let (mut pos_all, mut idx_all, mut flag_all) = (Vec::new(), Vec::new(), Vec::new());
    let rf = region("foot");
    let segs = if lod { 10 } else { 16 };
    for side in ["l", "r"] {
        let sx = if side == "l" { -1.0 } else { 1.0 };
        let pts: Vec<V3> = s.pos.iter().zip(&s.region).filter(|(q, r)| r[rf] > 0.5 && sign(q.x) == sx).map(|(q, _)| *q).collect();
        let pc = percentile(&pts.iter().map(|q| q.y).collect::<Vec<_>>(), 40.0);
        // Below the 40th percentile by more than a nanometre: the foot's rings are mirror-symmetric,
        // so heights come in equal pairs, and the percentile can fall on one; a plain `< pc` took
        // or dropped one of the pair by rounding noise (the sole's width moved by 2 mm).
        let low: Vec<V3> = pts.iter().filter(|q| q.y < pc - 1e-9).copied().collect();
        let cx = mean(&low.iter().map(|q| q.x).collect::<Vec<_>>());
        let z0 = low.iter().map(|q| q.z).fold(f64::INFINITY, f64::min);
        let z1 = low.iter().map(|q| q.z).fold(f64::NEG_INFINITY, f64::max);
        let w = (low.iter().map(|q| q.x).fold(f64::NEG_INFINITY, f64::max) - low.iter().map(|q| q.x).fold(f64::INFINITY, f64::min)) / 2.0;
        let a = linspace_open(0.0, 2.0 * PI, segs);
        let l = (z1 - z0) / 2.0 + 0.005 * k;
        let cz = (z0 + z1) / 2.0;
        // A rounded rectangle (superellipse), a little narrower at the heel.
        let e = 2.0 / 3.2;
        let outline: Vec<(f64, f64)> = a
            .iter()
            .map(|&a| {
                let (sa, ca) = (a.sin(), a.cos());
                let ox = sign(sa) * sa.abs().powf(e) * (w + 0.005 * k) * (1.0 - 0.1 * (if ca < 0.0 { 1.0 } else { 0.0 }) * ca.abs());
                let oz = sign(ca) * ca.abs().powf(e) * l;
                (cx + ox, cz + oz)
            })
            .collect();
        let (pp, idx) = prism(&outline, 0.0, thick, (cx, cz));
        let idx = closed_outward(&pp, idx);
        let n = pos_all.len();
        idx_all.extend(idx.iter().map(|&i| i + n));
        flag_all.extend(vec![FLAG_SOLE; pp.len()]);
        pos_all.extend(pp);
        if gid == "shoes.boots" {
            // The cuff: a short tube round the shin at the boot's top, a little proud of the leg.
            let pb = surface(family, BASE_STAGE).p;
            let top_b = cuts.get(gid).unwrap().shoe.unwrap();
            let (fb, cb) = (b(&format!("foot_{side}")), b(&format!("calf_{side}")));
            let t = (top_b - pb[fb].y) / (pb[cb].y - pb[fb].y);
            let yc = p[fb].y + t * (p[cb].y - p[fb].y);
            let rl = region("leg");
            let keep: Vec<bool> = s.pos.iter().zip(&s.region).map(|(q, r)| r[rl] > 0.3 && sign(q.x) == sx).collect();
            let lp = section(&s.pos, &surface_tris(family), yc, &keep);
            let lx: Vec<f64> = lp.iter().map(|q| q.x).collect();
            let lz: Vec<f64> = lp.iter().map(|q| q.z).collect();
            let c = v3(mean(&lx), yc, mean(&lz));
            let span = |v: &[f64]| (v.iter().copied().fold(f64::NEG_INFINITY, f64::max) - v.iter().copied().fold(f64::INFINITY, f64::min)) / 2.0;
            let rx = span(&lx) + 0.006 * k;
            let rz = span(&lz) + 0.006 * k;
            let cs = if lod { 8 } else { 12 };
            let bb = linspace_open(0.0, 2.0 * PI, cs);
            let ring = |y: f64, grow: f64| -> Vec<V3> { bb.iter().map(|&b| v3(c[0] + b.sin() * (rx + grow), y, c[2] + b.cos() * (rz + grow))).collect() };
            let rows = [ring(yc - 0.022 * k, -0.004 * k), ring(yc - 0.016 * k, 0.0), ring(yc + 0.002 * k, 0.0), ring(yc + 0.004 * k, -0.005 * k)];
            let (pp, idx) = strip(&rows, true);
            let idx = outward(&pp, idx, (c[0], c[2]));
            let n = pos_all.len();
            idx_all.extend(idx.iter().map(|&i| i + n));
            flag_all.extend(vec![0.0; pp.len()]);
            pos_all.extend(pp);
        }
    }
    let dense = follow(&pos_all, &s);
    let nor = smooth_normals(&pos_all, &idx_all);
    let n = pos_all.len();
    let fields = (0..n).map(|i| [0.0, flag_all[i], SHOES, 0.0]).collect();
    Mesh { pos: pos_all, idx: idx_all, nor, dense, fields, tone: vec![[0.0; 4]; n], ..Mesh::default() }
}

/// Every shell of `family` at `stage`, each with a low-detail copy (`lod.<id>`).
pub fn build_clothes(family: &str, stage: &str, cuts: &Cuts) -> Vec<(String, Mesh)> {
    let mut out = Vec::new();
    for gid in ["top.jacket", "top.suit"] {
        if cuts.contains(gid) {
            out.push((gid.to_string(), jacket(family, stage, gid, false)));
            out.push((format!("lod.{gid}"), jacket(family, stage, gid, true)));
        }
    }
    for gid in ["top.polo", "top.shirt"] {
        if cuts.contains(gid) {
            let m = collar(family, stage, gid, cuts);
            out.push((gid.to_string(), m.clone()));
            out.push((format!("lod.{gid}"), m));
        }
    }
    if cuts.contains("bottom.skirt") {
        out.push(("bottom.skirt".into(), skirt_shell(family, stage, cuts, SKIRT_SEGS, SKIRT_ROWS)));
        out.push(("lod.bottom.skirt".into(), skirt_shell(family, stage, cuts, 12, 4)));
    }
    for gid in cuts.keys().filter(|g| g.starts_with("shoes.")) {
        out.push((gid.to_string(), soles(family, stage, gid, cuts, false)));
        out.push((format!("lod.{gid}"), soles(family, stage, gid, cuts, true)));
    }
    out
}

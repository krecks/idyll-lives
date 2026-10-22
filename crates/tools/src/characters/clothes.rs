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
//! - `shoes.*`: an upper (the foot's skin under the collar, lifted off it: a toe box, a heel counter,
//!   a collar with its lining turned in to the skin, laces) on a sole that stands out round it.
//!
//! Seated, the jacket and the skirt lie on the lap rather than keep the flare they hang with: their
//! tone carries a seated corrective the runtime adds as the thighs turn forward (`seated_tone`).
//!
//! Contract: `garment_cuts(family, P)`: every id of `contract::garments`, in order, with its cut;
//! `build_clothes(family, stage, cuts)`: `top.jacket`, `lod.top.jacket`, ... (pos, nor, idx (CCW
//! out), dense, fields (hem distance, flag, category, 0), tone (the seated corrective, xyz)).
//!
//! Cut keys (metres on the base body's fields): top `sleeve` (sleeve parameter), `hem`, `crew` /
//! `scoop`, `over` (its hem is below the waistband: painted over the bottom); bottom `waist`,
//! `leg`, `skirt`; shoes `shoe` and `colour` (sRGB hex). Each shell is cut and laid out on the base
//! body, so it has the same vertices at every life stage (stages are deltas).

use std::sync::Arc;

use serde::Serialize;

use super::body::{NR, fields_of, region, seated_pose, surface, topology, tris};
use super::contract::garments;
use super::garments::{ARM_M, BOTTOM, FLAG_MIDSOLE, FLAG_POLISHED, FLAG_SKIRT, FLAG_SOLE, Landmarks, SHOES, TOP};
use super::head::head_frame;
use super::json::OMap;
use super::kdtree::KdTree;
use super::meshops::{Src, boundary_edges, clip_by_field, smooth_normals};
use super::np::{PI, clip, linspace, linspace_open, py_mod, py_round, radians, sign, smoothstep, unique};
use super::rig::{BASE_STAGE, NB, b, proportions};
use super::{Cache, Dense, M3, Mesh, V3, v3};

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
            // Painted up to the sock's top: under the shell, and the sock in its collar's opening.
            ("shoes", n) => Cut { shoe: Some(lm.ankle + shoe_style(gid).sock), colour: Some(shoe_colour(n)), ..Cut::default() },
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
    let cuts = garment_cuts(family, &surface(family, BASE_STAGE).p);
    let skip: Vec<bool> = lay.sleeve.iter().map(|&v| v > SHELL_SLEEVE - 0.06).collect();
    // `flared`: as it hangs standing; else as it lies seated (no flare, over the skirt as that lies).
    let shape = |flared: bool| {
        let mut pos: Vec<V3> = (0..pos.len())
            .map(|i| {
                let flare = if flared { smoothstep(hem + 0.14, hem, lay.hem_y[i]) } else { 0.0 };
                let taper = smoothstep(SHELL_SLEEVE, SHELL_SLEEVE - 0.24, lay.sleeve[i]);
                let lift = (jacket_lift(gid) + 0.012 * flare) * k * taper;
                pos[i] + nor[i] * lift
            })
            .collect();
        if cuts.contains("bottom.skirt") {
            // The hem lies over a skirt (it is a shell too).
            pos = over_skirt(&pos, family, stage, &cuts, 0.005 * k, !flared);
        }
        let n0 = pos.len();
        let (pos, idx, verts) = rim(&pos, &nor, &lay.idx, 0.6 * jacket_lift(gid) * k, &skip);
        (pos, idx, verts, n0)
    };
    let (pos, idx, verts, n0) = shape(true);
    let nor_all = smooth_normals(&pos, &idx);
    let mut d = shell_dense(&lay.src, dense);
    let extra: Vec<Dense> = verts.iter().map(|&v| d[v]).collect();
    d.extend(extra);
    // The hem distance (m, on the base body, so every stage shares it): from each vertex to the nearest open
    // edge (hem, lapels, neck; not a sleeve's end, which sinks into the painted sleeve), for the shader's band.
    let base = surface(family, BASE_STAGE);
    let bp = lerp_src(&lay.src, if lod { &base.lod_pos } else { &base.pos });
    let open: Vec<[usize; 2]> = boundary_edges(&lay.idx).into_iter().filter(|e| !(skip[e[0]] && skip[e[1]])).collect();
    let to_edge = |q: &V3, [a, c]: [usize; 2]| {
        let e = bp[c] - bp[a];
        (q - (bp[a] + e * clip((q - bp[a]).dot(&e) / e.norm_squared().max(1e-12), 0.0, 1.0))).norm()
    };
    let hem_d: Vec<f64> = bp.iter().map(|q| open.iter().map(|&e| to_edge(q, e)).fold(0.05, f64::min)).collect();
    let n = pos.len();
    let fields = (0..n).map(|i| [if i < n0 { hem_d[i] } else { 0.0 }, 0.0, TOP, 0.0]).collect();
    let clear = if cuts.contains("bottom.skirt") { JACKET_CLEAR } else { SKIRT_CLEAR };
    let tone = seated_tone(family, stage, &pos, &d, &shape(false).0, clear * k);
    Mesh { pos, nor: nor_all, idx, dense: d, fields, tone, ..Mesh::default() }
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
    let fields = vec![[0.05, 0.0, TOP, 0.0]; n];
    Mesh { pos, idx, nor, dense, fields, tone: vec![[0.0; 4]; n], ..Mesh::default() }
}

fn surface_tris(family: &str) -> Arc<Vec<usize>> {
    tris(family)
}

/// The skirt's rows (heights), axis z, angles and radii (rows x segs); `hug`: the radii without the
/// flare (the body's sections with their hollows bridged, plus the ease), where it lies seated.
struct SkirtProfile {
    ys: Vec<f64>,
    cz: f64,
    ang: Vec<f64>,
    r: Vec<Vec<f64>>,
    hug: Vec<Vec<f64>>,
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
        let flared: Vec<Vec<f64>> = (0..rows).map(|i| (0..segs).map(|j| (r[i][j] + 0.006 * k).max(top[j] + (hem[j] - top[j]) * clip((t[i] - 0.2) / 0.8, 0.0, 1.0).powf(0.9))).collect()).collect();
        let hug = (0..rows).map(|i| bridged(&r[i], 2.0 * PI / segs as f64).iter().zip(&flared[i]).map(|(h, f)| (h + 0.006 * k).min(*f)).collect()).collect();
        SkirtProfile { ys, cz, ang, r: flared, hug }
    })
}

/// A ring of radii (`step` rad apart, round the whole ring) with its hollows bridged: each no less
/// than where the chord between its neighbours crosses its ray (the cloth spans the gap between the legs).
fn bridged(r: &[f64], step: f64) -> Vec<f64> {
    let n = r.len();
    let c = step.cos();
    let mut out = r.to_vec();
    for _ in 0..n {
        let mut moved = false;
        for j in 0..n {
            let (a, bb) = (out[(j + n - 1) % n], out[(j + 1) % n]);
            let chord = 2.0 * a * bb * c / (a + bb).max(1e-9);
            if chord > out[j] + 1e-9 {
                out[j] = chord;
                moved = true;
            }
        }
        if !moved {
            break;
        }
    }
    out
}

/// Points pushed out from the pelvis' axis to lie `lift` outside the skirt where it is (as it lies
/// seated with `hug`).
fn over_skirt(pos: &[V3], family: &str, stage: &str, cuts: &Cuts, lift: f64, hug: bool) -> Vec<V3> {
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
            let rr = |ii: usize, jj: usize| if hug { sp.hug[ii % ny][jj % na] } else { sp.r[ii % ny][jj % na] };
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
    let shape = |r: &[Vec<f64>]| {
        let g: Vec<Vec<V3>> = (0..rows).map(|i| (0..segs).map(|j| v3(ang[j].sin() * r[i][j], ys[i], cz + ang[j].cos() * r[i][j])).collect()).collect();
        // Hem: a band turned up inside the bottom edge (the cloth's thickness).
        let hem_in: Vec<V3> = (0..segs).map(|j| g[rows - 1][j] - v3(ang[j].sin(), 0.0, ang[j].cos()) * 0.004 * k + v3(0.0, 0.012 * k, 0.0)).collect();
        let (mut pos, mut idx) = strip(&g, true);
        let (hp, hi) = strip(&[g[rows - 1].clone(), hem_in], true);
        pos.extend(hp);
        idx.extend(hi.iter().map(|&i| i + rows * segs));
        (pos, idx)
    };
    let (pos, idx) = shape(&sp.r);
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
        .collect::<Vec<Dense>>();
    let fields = (0..n).map(|i| [(1.0 - t[i]) * (yb0 - yb1), FLAG_SKIRT, BOTTOM, 0.0]).collect();
    let body_n = (rows - 1) * segs * 6;
    let tri = outward(&pos[..rows * segs], idx[..body_n].to_vec(), (0.0, cz));
    let tri = if tri != idx[..body_n] { flip(&idx) } else { idx };
    let nor = smooth_normals(&pos, &tri);
    // Seated, it lies on the thighs: without its flare, and clear of the skin.
    let tone = seated_tone(family, stage, &pos, &dense, &shape(&sp.hug).0, SKIRT_CLEAR * k);
    Mesh { pos, idx: tri, nor, dense, fields, tone, ..Mesh::default() }
}

/// The thighs' turn forward from the hips (degrees) the seated corrective is made for: the `sit`
/// clip's (clips/seated.ts), which the runtime weighs it by (characters.ts `SIT_FLEX`).
const SIT_FLEX: f64 = 76.0;
/// Seated, how far (m) a shell keeps off the skin at least: the skirt, and a jacket (over it).
const SKIRT_CLEAR: f64 = 0.005;
const JACKET_CLEAR: f64 = 0.008;

/// A body seated (`seated_pose`): its skin skinned so, the skin's normals, a tree of its vertices,
/// and the seat's height (the lowest skin under the hips, as `seat_drop` has it).
struct Seated {
    pose: Vec<(M3, V3)>,
    pos: Vec<V3>,
    nor: Vec<V3>,
    tree: KdTree,
    seat: f64,
}

static SEATED: Cache<(String, String), Seated> = Cache::new();

fn seated(family: &str, stage: &str) -> Arc<Seated> {
    SEATED.get((family.to_string(), stage.to_string()), || {
        let s = surface(family, stage);
        let pose = seated_pose(&s.p, SIT_FLEX);
        let pos: Vec<V3> = s.pos.iter().zip(&s.dense).map(|(q, d)| skinned(&pose, *q, d).0).collect();
        let nor = smooth_normals(&pos, &tris(family));
        let tree = KdTree::new(&pos);
        let hips = ["pelvis", "thigh_l", "thigh_r", "spine_01"].map(b);
        let seat = pos.iter().zip(&s.dense).filter(|(_, d)| hips.iter().map(|&i| d[i]).sum::<f64>() > 0.6).map(|(q, _)| q.y).fold(f64::INFINITY, f64::min);
        Seated { pose, pos, nor, tree, seat }
    })
}

/// `q` with weights `d` skinned by `pose`, and the blend of the bones' rotations there.
fn skinned(pose: &[(M3, V3)], q: V3, d: &Dense) -> (V3, M3) {
    let mut p = V3::zeros();
    let mut a = M3::zeros();
    for (bi, (r, t)) in pose.iter().enumerate() {
        if d[bi] != 0.0 {
            p += (r * q + t) * d[bi];
            a += r * d[bi];
        }
    }
    (p, a)
}

/// A shell's seated corrective (its `tone` xyz, m in its rest space; w 0): what moves each vertex from
/// where the seated pose skins it to where `hug` (its shape as it lies seated, the same vertices) is
/// skinned, pushed out to `clear` off the seated skin where it would sink in. The runtime adds it as
/// far as the thighs are turned forward (`sitW`, characters.ts), so seated the skirt lies on the lap
/// and the jacket's hem on it, rather than standing out round the seat as they hang standing.
fn seated_tone(family: &str, stage: &str, pos: &[V3], dense: &[Dense], hug: &[V3], clear: f64) -> Vec<[f64; 4]> {
    let st = seated(family, stage);
    pos.iter()
        .zip(hug)
        .zip(dense)
        .map(|((q, h), d)| {
            let (now, a) = skinned(&st.pose, *q, d);
            let (mut to, _) = skinned(&st.pose, *h, d);
            // Out of the skin, and no lower than the seat (the cloth sat on lies on it).
            let j = st.tree.nearest(&to);
            let off = (to - st.pos[j]).dot(&st.nor[j]);
            if off < clear {
                to += st.nor[j] * (clear - off);
            }
            to.y = to.y.max(st.seat + clear);
            let delta = a.try_inverse().map_or(V3::zeros(), |inv| inv * (to - now));
            // (Nothing to move: exactly zero, so the shells that don't move keep a zero tone.)
            if delta.norm() < 1e-5 { [0.0; 4] } else { [delta.x, delta.y, delta.z, 0.0] }
        })
        .collect()
}

/// A shoe type's shape (m, on a body 1.7 m tall and scaled with the body; heights over the ankle
/// joint are on the base body, as the cuts are).
struct ShoeStyle {
    /// The upper's lift off the skin, more on top of the toe box, round the heel counter, in a padded
    /// collar and at a boot's top.
    lift: f64,
    toe: f64,
    heel: f64,
    pad: f64,
    shaft: f64,
    /// How far the toe is drawn out forward: a dress shoe's long toe.
    point: f64,
    /// The collar's height over the ankle joint at the back, the sides and the front (the tongue, or
    /// a dress shoe's low throat, which ends at `throat` along the foot: the vamp covers the toes).
    collar: [f64; 3],
    throat: f64,
    /// The sock's top over the ankle joint: the painted cut (`shoe`), above the collar's sides.
    sock: f64,
    /// The sole's thickness under the ball and under the heel, where the heel's thickness gives way to
    /// the ball's (along the foot, 0 heel .. 1 toe), how far its edge stands out from the upper, the
    /// arch's lift off the floor between the heel and the ball, and the light midsole's share of its edge.
    sole: f64,
    heel_h: f64,
    block: (f64, f64),
    welt: f64,
    arch: f64,
    midsole: f64,
    laces: bool,
    /// Polished leather (glossier).
    polish: bool,
}

fn shoe_style(gid: &str) -> ShoeStyle {
    match gid {
        // Chunky: a padded collar, a round toe box, a thick white cup sole on a dark outsole, laces.
        "shoes.sneakers" => ShoeStyle {
            lift: 0.005,
            toe: 0.003,
            heel: 0.003,
            pad: 0.002,
            shaft: 0.0,
            point: 0.0,
            collar: [0.028, 0.018, 0.04],
            throat: 1.0,
            sock: 0.035,
            sole: 0.02,
            heel_h: 0.022,
            block: (0.15, 0.6),
            welt: 0.004,
            arch: 0.0,
            midsole: 0.5,
            laces: true,
            polish: false,
        },
        // Running shoes: chunkier still, a high heel counter, a thick white midsole rising to the heel.
        "shoes.trainers" => ShoeStyle {
            lift: 0.006,
            toe: 0.004,
            heel: 0.004,
            pad: 0.003,
            shaft: 0.0,
            point: 0.0,
            collar: [0.034, 0.024, 0.046],
            throat: 1.0,
            sock: 0.04,
            sole: 0.022,
            heel_h: 0.034,
            block: (0.1, 0.7),
            welt: 0.005,
            arch: 0.0,
            midsole: 0.8,
            laces: true,
            polish: false,
        },
        // Up the ankle, a little loose at the top, laced; a dark sole with a heel.
        "shoes.boots" => ShoeStyle {
            lift: 0.005,
            toe: 0.006,
            heel: 0.003,
            pad: 0.0,
            shaft: 0.004,
            point: 0.0,
            collar: [0.13, 0.125, 0.125],
            throat: 1.0,
            sock: 0.12,
            sole: 0.02,
            heel_h: 0.028,
            block: (0.26, 0.31),
            welt: 0.005,
            arch: 0.005,
            midsole: 0.0,
            laces: true,
            polish: false,
        },
        // Low and sleek: close to the foot, a throat down the instep, a long toe, polished, a thin sole
        // and a heel.
        _ => ShoeStyle {
            lift: 0.003,
            toe: 0.001,
            heel: 0.002,
            pad: 0.0,
            shaft: 0.0,
            point: 0.014,
            collar: [-0.02, -0.03, -0.022],
            throat: 0.45,
            sock: 0.032,
            sole: 0.011,
            heel_h: 0.024,
            block: (0.26, 0.31),
            welt: 0.003,
            arch: 0.006,
            midsole: 0.0,
            laces: false,
            polish: true,
        },
    }
}

impl ShoeStyle {
    /// The sole's top (m on a 1.7 m body) at `u` along the foot (0 heel .. 1 toe).
    fn top(&self, u: f64) -> f64 {
        self.sole + (self.heel_h - self.sole) * (1.0 - smoothstep(self.block.0, self.block.1, u))
    }

    /// The collar's height over the ankle joint at a point `dx` to the side of it, `dz` ahead and `u`
    /// along the foot.
    fn collar_at(&self, dx: f64, dz: f64, u: f64) -> f64 {
        let c = dz / dx.hypot(dz).max(1e-9);
        let [back, side, front] = self.collar;
        side + (front - side) * c.max(0.0) + (back - side) * (-c).max(0.0) + 0.05 * smoothstep(self.throat - 0.03, self.throat + 0.03, u)
    }
}

/// The upper below the sole's top (m on a 1.7 m body): cut there, inside the sole.
const UPPER_IN_SOLE: f64 = 0.005;

static FINER: Cache<(String, String, bool), Vec<V3>> = Cache::new();

/// The next level of a body's surface, on its limit (level 2 under the body, level 1 under the
/// low-detail body): there, the point a cut edge's middle lies on.
fn finer(family: &str, stage: &str, lod: bool) -> Arc<Vec<V3>> {
    FINER.get((family.to_string(), stage.to_string(), lod), || {
        let s = surface(family, stage);
        if lod { s.pos.clone() } else { topology().fine.apply(&s.ctrl) }
    })
}

/// Per edge of a surface's triangles (its faces' own edges, and the diagonals they are cut along), the
/// next level's vertex at its middle: an edge's point, or the face's (subdiv.rs: the `edges`' points
/// come after the `n` vertices, then the faces').
fn edge_mids(faces: &[Vec<usize>], edges: &[(usize, usize)], n: usize) -> std::collections::BTreeMap<(usize, usize), usize> {
    let mut out: std::collections::BTreeMap<(usize, usize), usize> = edges.iter().enumerate().map(|(k, &e)| (e, n + k)).collect();
    for (f, face) in faces.iter().enumerate() {
        for (i, &a) in face.iter().enumerate() {
            for &bv in face.iter().skip(i + 2) {
                out.entry((a.min(bv), a.max(bv))).or_insert(n + edges.len() + f);
            }
        }
    }
    out
}

/// A point on a body's triangles: up to three of its vertices (a triangle's, an edge's or one) and
/// weights summing to 1.
#[derive(Clone, Copy)]
struct Bary {
    v: [usize; 3],
    w: [f64; 3],
}

impl Bary {
    fn at(v: usize) -> Self {
        Bary { v: [v; 3], w: [1.0, 0.0, 0.0] }
    }

    /// (1 - t) a + t b (both on one of the body's triangles).
    fn lerp(a: &Bary, b: &Bary, t: f64) -> Self {
        let (mut v, mut w, mut n) = ([0; 3], [0.0; 3], 0);
        for (x, f) in [(a, 1.0 - t), (b, t)] {
            for k in 0..3 {
                let wk = x.w[k] * f;
                if wk == 0.0 {
                    continue;
                }
                match v[..n].iter().position(|&u| u == x.v[k]) {
                    Some(j) => w[j] += wk,
                    None => {
                        assert!(n < 3, "a shoe's point off the body's triangles");
                        (v[n], w[n]) = (x.v[k], wk);
                        n += 1;
                    }
                }
            }
        }
        for k in n..3 {
            v[k] = v[0];
        }
        Bary { v, w }
    }

    /// Linear in the body's per-vertex values.
    fn mix<T: Copy + std::ops::Mul<f64, Output = T> + std::ops::Add<Output = T>>(&self, arr: &[T]) -> T {
        arr[self.v[0]] * self.w[0] + arr[self.v[1]] * self.w[1] + arr[self.v[2]] * self.w[2]
    }

    /// On the curved skin: the quadratic patch through its triangle's corners and the finer surface
    /// at its edges' middles (`mid`: per pair (0, 1), (0, 2), (1, 2) of its vertices).
    fn curved(&self, pos: &[V3], fine: &[V3], mid: &[usize; 3]) -> V3 {
        let l = self.w;
        let mut p = V3::zeros();
        for i in 0..3 {
            if l[i] != 0.0 {
                p += pos[self.v[i]] * (l[i] * (2.0 * l[i] - 1.0));
            }
        }
        for (k, (i, j)) in [(0, 1), (0, 2), (1, 2)].into_iter().enumerate() {
            if l[i] != 0.0 && l[j] != 0.0 {
                p += fine[mid[k]] * (4.0 * l[i] * l[j]);
            }
        }
        p
    }
}

/// A shoe's upper laid out on the base body: the skin under the collar and above the sole's inside,
/// its edges the collar crosses split (twice, on the full body; so its edge is smooth however coarse the
/// skin is there), cut exactly along both (stage independent), with what each vertex needs to be lifted.
struct UpperLayout {
    pts: Vec<Bary>,
    /// Per point: the finer surface's vertices at its triangle's edges' middles (`finer`), so it lies on
    /// the curved skin, not on the body's flat triangles (whose sag made the collar jagged).
    mid: Vec<[usize; 3]>,
    idx: Vec<usize>,
    /// Per point: the lift along the normal and the toe's offset (m on a 1.7 m body), the distance from
    /// the collar (m, the hem distance), across the instep (m; 1 elsewhere), and whether it lies on the
    /// cut inside the sole (no lining there).
    lift: Vec<f64>,
    point: Vec<V3>,
    hem: Vec<f64>,
    across: Vec<f64>,
    low: Vec<bool>,
}

static UPPERS: Cache<(String, String, bool), UpperLayout> = Cache::new();

/// Triangles split at the middles of the edges `split` says (in 2, 3 or 4, so no T-junctions).
fn refine(pts: &mut Vec<Bary>, idx: &[usize], split: impl Fn(usize, usize) -> bool) -> Vec<usize> {
    let key = |a: usize, b: usize| (a.min(b), a.max(b));
    let mut cut: std::collections::BTreeMap<(usize, usize), usize> = std::collections::BTreeMap::new();
    for t in idx.chunks(3) {
        for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
            if split(a, b) && !cut.contains_key(&key(a, b)) {
                pts.push(Bary::lerp(&pts[a], &pts[b], 0.5));
                cut.insert(key(a, b), pts.len() - 1);
            }
        }
    }
    let mut out = Vec::with_capacity(idx.len() * 2);
    for t in idx.chunks(3) {
        let m: Vec<Option<usize>> = [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])].iter().map(|&(a, b)| cut.get(&key(a, b)).copied()).collect();
        // Turned so the split edges come first (edge k joins corner k and k + 1; winding kept).
        let r = match m.iter().filter(|e| e.is_some()).count() {
            0 => {
                out.extend_from_slice(t);
                continue;
            }
            1 => m.iter().position(|e| e.is_some()).unwrap(),
            2 => m.iter().position(|e| e.is_none()).map(|k| (k + 1) % 3).unwrap(),
            _ => 0,
        };
        let (a, b, c) = (t[r], t[(r + 1) % 3], t[(r + 2) % 3]);
        let (ab, bc, ca) = (m[r], m[(r + 1) % 3], m[(r + 2) % 3]);
        match (ab, bc, ca) {
            (Some(ab), None, None) => out.extend([a, ab, c, ab, b, c]),
            (Some(ab), Some(bc), None) => out.extend([ab, b, bc, a, ab, bc, a, bc, c]),
            (Some(ab), Some(bc), Some(ca)) => out.extend([a, ab, ca, ab, b, bc, ca, bc, c, ab, bc, ca]),
            _ => unreachable!(),
        }
    }
    out
}

fn upper_layout(family: &str, gid: &str, lod: bool) -> Arc<UpperLayout> {
    UPPERS.get((family.to_string(), gid.to_string(), lod), || {
        let s = surface(family, BASE_STAGE);
        let fl = base_fields(family);
        let st = shoe_style(gid);
        let kb = proportions(family, BASE_STAGE).h / 1.7;
        let (pos, nor): (&[V3], &[V3]) = if lod { (&s.lod_pos, &s.lod_nor) } else { (&s.pos, &s.nor) };
        let (ff, reg): (&[[f64; 4]], &[[f64; NR]]) = if lod { (&fl.1, &s.lod_region) } else { (&fl.0, &s.region) };
        let idx = if lod { topology().lod_tris.concat() } else { tris(family).to_vec() };
        let topo = topology();
        let mids = if lod {
            edge_mids(&topo.faces, &topo.s.edges, topo.n)
        } else {
            edge_mids(&topo.s.quads.iter().map(|q| q.to_vec()).collect::<Vec<_>>(), &topo.s2.edges, topo.s.m.rows)
        };
        let mid_of = |p: &Bary| [(0, 1), (0, 2), (1, 2)].map(|(i, j)| if p.w[i] != 0.0 && p.w[j] != 0.0 { mids[&(p.v[i].min(p.v[j]), p.v[i].max(p.v[j]))] } else { usize::MAX });
        let fine = finer(family, BASE_STAGE, lod);
        let height: Vec<f64> = ff.iter().map(|f| f[1]).collect();
        let rf = region("foot");
        let pointed = if st.point > 0.0 { 1.0 } else { 0.0 };
        // Per side: the ankle joint, and the foot's length (z at the heel and at the toe).
        let feet = ["l", "r"].map(|side| {
            let sx = if side == "l" { -1.0 } else { 1.0 };
            let zs: Vec<f64> = (0..pos.len()).filter(|&i| reg[i][rf] > 0.5 && sign(pos[i].x) == sx).map(|i| pos[i].z).collect();
            (sx, s.p[b(&format!("foot_{side}"))], zs.iter().copied().fold(f64::INFINITY, f64::min), zs.iter().copied().fold(f64::NEG_INFINITY, f64::max))
        });
        // A point's lift, toe offset, distance below the collar, across the instep and clip field.
        let attrs = |p: &Bary| -> (f64, V3, f64, f64, f64) {
            let q = p.curved(pos, &fine, &mid_of(p));
            let Some(&(_, ja, z0, z1)) = feet.iter().find(|f| f.0 == sign(q.x)) else { return (0.0, V3::zeros(), 0.0, 1.0, -1.0) };
            let n = p.mix(nor).normalize();
            let fy = p.mix(&height);
            let u = (q.z - z0) / (z1 - z0);
            let (dx, dz) = ((q.x - ja.x).abs(), q.z - ja.z);
            let up = n.y.max(0.0);
            let collar = ja.y + st.collar_at(dx, dz, u);
            let below = collar - fy;
            let toe = smoothstep(0.6, 0.88, u);
            let tip = smoothstep(0.65, 1.0, u).powi(2);
            // Toe box (on top), heel counter (at the back, low), a padded collar, a boot's loose top.
            let lift = st.lift * (1.0 - 0.6 * tip * up * pointed)
                + st.toe * toe * (0.3 + 0.7 * up)
                + st.heel * (1.0 - smoothstep(0.0, 0.25, u)) * (1.0 - up)
                + st.pad * (1.0 - smoothstep(0.0, 0.015, below))
                + st.shaft * smoothstep(ja.y + 0.03, collar, fy);
            // A long toe: drawn forward (not narrowed: the toes would come through).
            let point = v3(0.0, 0.0, st.point * tip);
            // Across the instep: m off the foot's middle (exact between vertices: the foot's middle is
            // a line of them), more than 1 off the front; the laces lie where this is small, down from
            // the collar.
            let across = if st.laces { dx + if dz > 0.5 * dx.hypot(dz) { 0.0 } else { 1.0 } } else { 1.0 };
            // Under the collar, above the sole's inside (lifted as at the base), off the sole (the
            // foot's flat underside).
            let y = q.y + n.y * lift * kb;
            let g = below.min(y - (st.top(u) - UPPER_IN_SOLE) * kb).min(n.y + 0.9);
            (lift, point, below, across, g)
        };
        let mut pts: Vec<Bary> = (0..pos.len()).map(Bary::at).collect();
        let mut tri = idx;
        for _ in 0..if lod { 0 } else { 1 } {
            let a: Vec<(f64, f64)> = pts.iter().map(&attrs).map(|a| (a.2, a.4)).collect();
            // The edges the collar crosses (on the shoe: the clip field over 0 at an end).
            tri = refine(&mut pts, &tri, |i, j| (a[i].1 > -0.01 || a[j].1 > -0.01) && (a[i].0 > 0.0) != (a[j].0 > 0.0));
        }
        let all: Vec<(f64, V3, f64, f64, f64)> = pts.iter().map(attrs).collect();
        let g: Vec<f64> = all.iter().map(|a| a.4).collect();
        let (cidx, src) = clip_by_field(&tri, &g);
        let pts: Vec<Bary> = (0..src.0.len()).map(|i| Bary::lerp(&pts[src.0[i]], &pts[src.1[i]], src.2[i])).collect();
        let lift = lerp_src(&src, &all.iter().map(|a| a.0).collect::<Vec<_>>());
        let point = lerp_src(&src, &all.iter().map(|a| a.1).collect::<Vec<_>>());
        let hem = lerp_src(&src, &all.iter().map(|a| a.2).collect::<Vec<_>>());
        let across = lerp_src(&src, &all.iter().map(|a| a.3).collect::<Vec<_>>());
        let mid: Vec<[usize; 3]> = pts.iter().map(mid_of).collect();
        // The hem distance: how far from the collar's edge (the nearest point of it), so it runs down
        // the instep as far as along the sides. Under the laces, how far down them from the collar's
        // front (straight down the middle of the instep: linear, so the laces' bars stay straight
        // across the big triangles there).
        let at: Vec<V3> = pts.iter().zip(&mid).map(|(p, m)| p.curved(pos, &fine, m)).collect();
        let edge: Vec<V3> = (0..at.len()).filter(|&i| src.0[i] != src.1[i] && hem[i].abs() < 1e-4).map(|i| at[i]).collect();
        let mut hem: Vec<f64> = at.iter().zip(&hem).map(|(q, &h)| if h <= 0.0 { 0.0 } else { edge.iter().map(|e| (q - e).norm()).fold(f64::INFINITY, f64::min) }).collect();
        for sx in [-1.0, 1.0] {
            let laced: Vec<usize> = (0..at.len()).filter(|&i| sign(at[i].x) == sx && across[i] < 1.0).collect();
            let middle = |i: &&usize| across[**i].abs() < 0.012;
            let Some(&c) = laced.iter().filter(middle).filter(|&&i| hem[i] == 0.0).min_by(|&&i, &&j| across[i].abs().total_cmp(&across[j].abs())) else { continue };
            let far = *laced.iter().filter(middle).min_by(|&&i, &&j| ((at[i] - at[c]).norm() - 0.06).abs().total_cmp(&((at[j] - at[c]).norm() - 0.06).abs())).unwrap();
            let down = (at[far] - at[c]).normalize();
            for &i in &laced {
                hem[i] = (at[i] - at[c]).dot(&down).max(0.0);
            }
        }
        // The cut inside the sole: new vertices on an edge, not at the collar.
        let low = (0..src.0.len()).map(|i| src.0[i] != src.1[i] && hem[i] > 0.01).collect();
        UpperLayout { pts, mid, idx: cidx, lift, point, hem, across, low }
    })
}

/// A shoe's upper at `stage`: the layout's skin lifted off it (with the body's own weights, so it
/// moves as the foot does), and a lining turned in from the collar to the skin.
fn upper(family: &str, stage: &str, gid: &str, lod: bool) -> Mesh {
    let lay = upper_layout(family, gid, lod);
    let st = shoe_style(gid);
    let s = surface(family, stage);
    let k = proportions(family, stage).h / 1.7;
    // (The body's normals: from its finer limit surface, so the shoe shades as smoothly as the skin.)
    let (bpos, bnor, dense): (&[V3], &[V3], &[Dense]) = if lod { (&s.lod_pos, &s.lod_nor, &s.lod_dense) } else { (&s.pos, &s.nor, &s.dense) };
    let fine = finer(family, stage, lod);
    let nor: Vec<V3> = lay.pts.iter().map(|p| p.mix(bnor).normalize()).collect();
    let pos: Vec<V3> = (0..lay.pts.len()).map(|i| lay.pts[i].curved(bpos, &fine, &lay.mid[i]) + nor[i] * lay.lift[i] * k + lay.point[i] * k).collect();
    let mut d: Vec<Dense> = lay.pts.iter().map(|p| std::array::from_fn(|k| (0..3).map(|j| dense[p.v[j]][k] * p.w[j]).sum())).collect();
    let n0 = pos.len();
    let flag = if st.polish { FLAG_POLISHED } else { 0.0 };
    let fields = (0..n0).map(|i| [lay.hem[i], flag, SHOES, lay.across[i]]);
    let mut out = Mesh { pos, nor, idx: lay.idx.clone(), fields: fields.collect(), ..Mesh::default() };
    if !lod {
        // The lining: a band from the collar's edge in to the skin (its hem distance negative: drawn dark).
        let depth = 0.85 * (st.lift + st.pad + st.shaft) * k;
        let be: Vec<[usize; 2]> = boundary_edges(&out.idx).into_iter().filter(|e| !(lay.low[e[0]] && lay.low[e[1]])).collect();
        let verts = unique(&be.concat());
        let mut inner = vec![usize::MAX; n0];
        for &v in &verts {
            inner[v] = out.pos.len();
            let (p, nv) = (out.pos[v], out.nor[v]);
            out.pos.push(p - nv * depth);
            out.fields.push([-depth, flag, SHOES, 1.0]);
            d.push(d[v]);
        }
        for [a, bv] in be {
            out.idx.extend([bv, a, inner[a], bv, inner[a], inner[bv]]);
        }
        out.nor.extend_from_slice(&smooth_normals(&out.pos, &out.idx)[n0..]);
    }
    let n = out.pos.len();
    Mesh { dense: d, tone: vec![[0.0; 4]; n], ..out }
}

/// The sole under a foot's upper: a slab round the upper's foot (its widest, seen from above, from the
/// full upper, so a low-detail sole matches) standing out by the welt, from the floor (the arch lifted
/// off it) to the sole's top; its edge a light midsole over a dark outsole, or dark.
fn sole(family: &str, stage: &str, gid: &str, up: &Mesh, side: f64, segs: usize) -> (Vec<V3>, Vec<usize>, Vec<f64>) {
    let st = shoe_style(gid);
    let k = proportions(family, stage).h / 1.7;
    // (Not its lining.)
    let keep: Vec<bool> = up.pos.iter().zip(&up.fields).map(|(q, f)| sign(q.x) == side && f[0] >= 0.0).collect();
    let pts: Vec<V3> = up.pos.iter().zip(&keep).filter(|(_, k)| **k).map(|(q, _)| *q).collect();
    let span = |f: fn(&V3) -> f64| (pts.iter().map(f).fold(f64::INFINITY, f64::min), pts.iter().map(f).fold(f64::NEG_INFINITY, f64::max));
    let ((x0, x1), (z0, z1)) = (span(|q| q.x), span(|q| q.z));
    let (cx, cz) = ((x0 + x1) / 2.0, (z0 + z1) / 2.0);
    // The upper's outline low down (its vertices and its sections at a few heights), seen from above,
    // in the foot's box scaled to a unit square (so the directions spread evenly round its length).
    let ys = linspace(0.0, (st.heel_h + 0.03) * k, 7);
    let mut low: Vec<V3> = pts.iter().filter(|q| q.y < ys[6]).copied().collect();
    for &y in &ys[1..] {
        low.extend(section(&up.pos, &up.idx, y, &keep));
    }
    let (hx, hz) = ((x1 - x0) / 2.0, (z1 - z0) / 2.0);
    let ang = linspace_open(0.0, 2.0 * PI, segs);
    let width = PI / segs as f64 * 0.75;
    let mut r: Vec<f64> = ang
        .iter()
        .map(|&a| {
            low.iter()
                .map(|q| ((q.x - cx) / hx, (q.z - cz) / hz))
                .filter(|(x, z)| (py_mod(x.atan2(*z) - a + PI, 2.0 * PI) - PI).abs() < width)
                .map(|(x, z)| x.hypot(z))
                .fold(f64::NAN, f64::max)
        })
        .collect();
    // (A direction with no vertex near it takes the nearest that has one.)
    for j in 0..segs {
        if r[j].is_nan() {
            r[j] = (1..segs).flat_map(|o| [r[(j + o) % segs], r[(j + segs - o) % segs]]).find(|v| !v.is_nan()).unwrap();
        }
    }
    let u = |z: f64| clip((z - z0) / (z1 - z0), 0.0, 1.0);
    let arch = |u: f64| st.arch * k * smoothstep(st.block.1 - 0.02, st.block.1 + 0.06, u) * (1.0 - smoothstep(0.5, 0.62, u));
    // The outline, and its outward normals (seen from above).
    let edge: Vec<(f64, f64)> = (0..segs).map(|j| (cx + ang[j].sin() * r[j] * hx, cz + ang[j].cos() * r[j] * hz)).collect();
    let out: Vec<(f64, f64)> = (0..segs)
        .map(|j| {
            let (a, c) = (edge[(j + segs - 1) % segs], edge[(j + 1) % segs]);
            let (tx, tz) = (c.0 - a.0, c.1 - a.1);
            let l = tx.hypot(tz);
            (-tz / l, tx / l)
        })
        .collect();
    let ring = |grow: f64, y: &dyn Fn(f64) -> f64| -> Vec<V3> {
        (0..segs)
            .map(|j| {
                let (x, z) = (edge[j].0 + out[j].0 * grow, edge[j].1 + out[j].1 * grow);
                v3(x, y(u(z)), z)
            })
            .collect()
    };
    let welt = st.welt * k;
    let top = |u: f64| st.top(u) * k;
    let split = |u: f64| top(u) * (1.0 - st.midsole);
    // Rows down its edge: a light midsole and a dark outsole meet at a seam (a row twice, one colour
    // above it, the other below); else dark all the way down.
    let (rows, mut flags) = if st.midsole > 0.0 {
        let rows = vec![ring(welt, &top), ring(welt + 0.001 * k, &split), ring(welt + 0.001 * k, &split), ring(welt - 0.0015 * k, &arch)];
        (rows, [FLAG_MIDSOLE, FLAG_MIDSOLE, FLAG_SOLE, FLAG_SOLE].iter().flat_map(|&f| vec![f; segs]).collect::<Vec<_>>())
    } else {
        (vec![ring(welt, &top), ring(welt - 0.0015 * k, &arch)], vec![FLAG_SOLE; 2 * segs])
    };
    let (mut pos, mut idx) = strip(&rows, true);
    if st.midsole > 0.0 {
        idx.drain(segs * 6..segs * 12);
    }
    let (ct, cb, last) = (pos.len(), pos.len() + 1, (rows.len() - 1) * segs);
    pos.push(v3(cx, top(0.5), cz));
    pos.push(v3(cx, arch(0.5), cz));
    for j in 0..segs {
        let j1 = (j + 1) % segs;
        idx.extend([ct, j1, j, cb, last + j, last + j1]);
    }
    flags.extend([flags[0], FLAG_SOLE]);
    let idx = closed_outward(&pos, idx);
    (pos, idx, flags)
}

/// A shoe: the upper and a sole under each foot.
fn shoe(family: &str, stage: &str, gid: &str, lod: bool) -> Mesh {
    let s = surface(family, stage);
    let up = upper(family, stage, gid, lod);
    let full = if lod { upper(family, stage, gid, false) } else { up.clone() };
    let (mut pos, mut flags) = (Vec::new(), Vec::new());
    let mut idx = Vec::new();
    for side in [-1.0, 1.0] {
        let (p, i, f) = sole(family, stage, gid, &full, side, if lod { 10 } else { 18 });
        let n = pos.len();
        idx.extend(i.iter().map(|&v| v + n));
        pos.extend(p);
        flags.extend(f);
    }
    let dense = follow(&pos, &s);
    let nor = smooth_normals(&pos, &idx);
    let fields = flags.iter().map(|&f| [0.05, f, SHOES, 1.0]).collect();
    let n = pos.len();
    let sole = Mesh { pos, nor, idx, dense, fields, tone: vec![[0.0; 4]; n], ..Mesh::default() };
    super::merge(&[&up, &sole])
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
        out.push((gid.to_string(), shoe(family, stage, gid, false)));
        out.push((format!("lod.{gid}"), shoe(family, stage, gid, true)));
    }
    out
}

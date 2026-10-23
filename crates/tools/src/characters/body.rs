//! The residents' bodies: a control cage of superellipse rings (cage.rs) fitted so its limit
//! surface passes through the ring points, subdivided once (subdiv.rs, about 5k triangles) for
//! `body`; the cage at its limit (about 1.1k triangles) is `lod.body`. Contract:
//!
//! - `build_body(family, stage)`: parts `body` and `lod.body` (pos, nor, idx (CCW out), dense,
//!   arm), meta (`seatDrop`, `eyeY`, `grip`) and the joints;
//! - `body_fields_and_tone(family, mesh, P, head)`: fields (n, 4) and tone (n, 4), base stage only.
//!
//! Proportions come from rig.rs (`proportions`: height, head size, hips, shoulders, limb lengths
//! and girth, waist and belly) plus the shapes below, per family. Every stage of both families
//! builds the same cage, so the topology is fixed and stages are deltas.
//!
//! Skin weights are set per ring (a ring's bone, blended 50/50 or so with the neighbour across a
//! joint; the shoulders blend spine_03, clavicle and upperarm, the hips pelvis and thigh, the neck
//! spine_03, neck_01 and Head) and subdivide with the positions, so they fall off smoothly.
//!
//! Fields (docs/characters-format.md): x = sleeve parameter (-1 torso, 0 shoulder .. 1 wrist, over
//! 1 on the hand: the arm's share of a vertex blends from the torso's -1 at the armhole), y = height,
//! z / w = the crew and scoop (male: V) necklines, metres below a smooth curve round the neck
//! (positive: covered); on the head -1 - scalp weight. Tone: x = blush (cheeks, nose, ears), z =
//! the face's age folds (`HeadFrame::fold`); the shade in the creases is baked into w afterwards
//! (ao.rs).
//!
//! The head's cage is finer over the face (head.rs) than the LOD's: the LOD keeps the coarse grid
//! (`Cage::lod_faces`), so `lod.body` has the face's extra cage vertices unused.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use nalgebra::{DMatrix, DVector, Dyn, LU};

use super::cage::{Cage, Per, RingTags, Tags, Weights, se, turn, unit};
use super::head::{HeadFrame, head_cage, youth};
use super::kdtree::KdTree;
use super::meshops::smooth_normals;
use super::np::{clip, lstsq, mean, mean_rows, radians, sign, unique};
use super::rig::{BASE_STAGE, NB, b, joints, proportions};
use super::sparse::Csr;
use super::subdiv::{Subdivision, check_closed, limit_matrix, orient, polygon_triangles, subdivision, triangles};
use super::{Cache, Dense, M3, Mesh, V3, key, v3};

const SPINE: [&str; 4] = ["pelvis", "spine_01", "spine_02", "spine_03"];
pub const REGIONS: [&str; 8] = ["torso", "arm", "hand", "leg", "foot", "neck", "head", "ear"];
pub const NR: usize = REGIONS.len();
/// The arm's root loop on the chest rings (k: torso ring index): right side, then mirrored.
fn arm_k(s: &str) -> (usize, usize, usize) {
    if s == "r" { (2, 3, 4) } else { (10, 9, 8) }
}
/// The thighs' first rings keep this far (x thigh radius) from the midline, either side of the crotch.
const CROTCH_GAP: f64 = 0.1;

pub fn region(name: &str) -> usize {
    REGIONS.iter().position(|&r| r == name).expect("region")
}

/// Weighted sum of bone -> weight lists: mix(&[(0.5, a), (0.5, b)]).
fn mix(pairs: &[(f64, &Weights)]) -> Weights {
    let mut out: Weights = Vec::new();
    for &(k, w) in pairs {
        for &(n, v) in w {
            match out.iter_mut().find(|e| e.0 == n) {
                Some(e) => e.1 += k * v,
                None => out.push((n, 0.0 + k * v)),
            }
        }
    }
    out
}

fn one(name: &'static str) -> Weights {
    vec![(name, 1.0)]
}

/// Leaks a formatted bone name (the names are a fixed small set; cage weights hold `&'static str`).
fn bone(name: String) -> &'static str {
    super::rig::BONES[b(&name)]
}

/// Bone weights along the spine by height (hat functions between the segments' middles).
fn spine_weights(p: &[V3; NB]) -> impl Fn(f64) -> Weights {
    let ys: Vec<f64> = SPINE.iter().chain(["neck_01"].iter()).map(|n| p[b(n)].y).collect();
    let mids: Vec<f64> = (0..4).map(|i| (ys[i] + ys[i + 1]) / 2.0).collect();
    move |y: f64| {
        if y <= mids[0] {
            return one("pelvis");
        }
        if y >= mids[3] {
            return one("spine_03");
        }
        let i = mids.partition_point(|&m| m < y) - 1;
        let t = (y - mids[i]) / (mids[i + 1] - mids[i]);
        vec![(SPINE[i], 1.0 - t), (SPINE[i + 1], t)]
    }
}

/// The numbers the cage is built from (metres), per family and stage.
struct Shape {
    h: f64,
    female: bool,
    yg: f64,
    r_th: f64,
    r_a: f64,
    r_n: f64,
    waist: f64,
    belly: f64,
}

fn shape(family: &str, stage: &str) -> Shape {
    let pr = proportions(family, stage);
    let (h, g) = (pr.h, pr.girth);
    let female = family == "female";
    let yg = youth(pr.hh);
    Shape {
        h,
        female,
        yg,
        r_th: (if female { 0.058 } else { 0.056 }) * h * g * (1.0 + 0.08 * yg),
        r_a: (if female { 0.029 } else { 0.033 }) * h * g,
        r_n: (if female { 0.031 } else { 0.035 }) * h * g.powf(0.5),
        waist: pr.waist,
        belly: pr.belly * h,
    }
}

/// Ring points round centre c: angle 0 towards +V, 90 towards +U; depth bf in front (+V), bb behind.
#[allow(clippy::too_many_arguments)]
fn ring_pts(c: V3, u: V3, v: V3, a: f64, bf: f64, bb: f64, ang: &[f64], p: f64) -> Vec<V3> {
    ang.iter()
        .map(|&t| {
            let (s, co) = (t.sin(), t.cos());
            c + u * (a * se(s, p)) + v * ((if co >= 0.0 { bf } else { bb }) * se(co, p))
        })
        .collect()
}

/// The control cage (targets on the designed surface) of `family` at `stage`, and its joints.
pub fn body_cage(family: &str, stage: &str) -> (Cage, [V3; NB]) {
    let pr = proportions(family, stage);
    let p = joints(family, stage);
    let pj = |n: &str| p[b(n)];
    let s = shape(family, stage);
    let (h, yg, female) = (s.h, s.yg, s.female);
    let (r_th, r_a, r_n) = (s.r_th, s.r_a, s.r_n);
    let bel = s.belly;
    let sw = spine_weights(&p);
    let mut c = Cage::new();
    let (x, y, z) = (v3(1.0, 0.0, 0.0), v3(0.0, 1.0, 0.0), v3(0.0, 0.0, 1.0));
    let y_hip = pj("thigh_l").y;
    let (y_s1, y_s2, y_s3) = (pj("spine_01").y, pj("spine_02").y, pj("spine_03").y);
    let y_neck = pj("neck_01").y;
    let hip_x = pj("thigh_r").x;
    let sh = pj("upperarm_r");

    // Arm root loops (right, left): the 2 x 2 patch on the chest rings T5-T7 the arm leaves through.
    let mut roots: HashMap<&str, Vec<V3>> = HashMap::new();
    for (sd, sx) in [("r", 1.0), ("l", -1.0)] {
        let jsh = pj(&format!("upperarm_{sd}"));
        let o = v3(sx, 0.0, 0.0);
        // (chi: 135 180 225 | 270 | 315 0 45 | 90): bottom front .. round the loop.
        let spec = [(-0.75, -1.5, 0.75), (-0.82, -1.95, 0.0), (-0.75, -1.5, -0.72), (-0.6, -0.75, -1.0), (-0.4, 0.05, -0.78), (-0.2, 0.25, 0.0), (-0.4, 0.05, 0.8), (-0.6, -0.75, 1.02)];
        roots.insert(sd, spec.iter().map(|&(dx, dy, dz)| jsh + (o * dx + y * dy + z * dz) * r_a).collect());
    }

    // Torso rings T1 (hips) .. T7 (shoulders); (y, half width, front depth, back depth, z,
    // squareness). T7 is all neck loop and arm loops (below).
    let hw = hip_x + r_th * 0.85;
    let f = |a: f64, m: f64| if female { a } else { m };
    let rows = [
        (y_hip + 0.012 * h, hw, 0.064 * h + bel * 0.3, f(0.084, 0.078) * h, -0.006 * h, 2.3),
        (y_hip + 0.55 * (y_s1 - y_hip), f(0.106, 0.095) * h * (1.0 + 0.1 * yg), 0.062 * h + bel * 0.7, f(0.075, 0.07) * h, 0.0, 2.3),
        ((y_s1 + y_s2) / 2.0, f(0.068, 0.08) * h * s.waist * (1.0 + 0.25 * yg), 0.062 * h + bel, 0.056 * h, 0.003 * h, 2.3),
        (y_s2 + 0.45 * (y_s3 - y_s2), f(0.085, 0.098) * h * (1.0 + 0.12 * yg), 0.068 * h + bel * 0.6, 0.060 * h, 0.002 * h, 2.2),
        (sh[1] - 1.95 * r_a, f(0.092, 0.108) * h, f(0.07, 0.074) * h + bel * 0.2, 0.064 * h, 0.0, 2.2),
        (sh[1] - 0.75 * r_a, f(0.096, 0.112) * h, 0.07 * h, 0.064 * h, -0.002 * h, 2.2),
        (sh[1] + 0.25 * r_a, f(0.09, 0.1) * h, 0.05 * h, 0.055 * h, -0.006 * h, 2.2),
    ];
    let th12: Vec<f64> = (0..12).map(|i| radians(i as f64 * 30.0)).collect();
    let mut t: Vec<Vec<V3>> = rows.iter().map(|&(yy, a, bf, bb, zc, pw)| ring_pts(v3(0.0, yy, zc), x, z, a, bf, bb, &th12, pw)).collect();
    // Bust (female, gentle), shoulder blades, buttocks, belly.
    if female {
        for k in [1, 11] {
            t[4][k][2] += 0.016 * h * (1.0 - yg);
            t[4][k][1] -= 0.008 * h;
            t[3][k][2] += 0.005 * h * (1.0 - yg);
        }
    }
    for k in [5, 7] {
        t[5][k][2] -= 0.006 * h;
        t[0][k][2] -= f(0.008, 0.004) * h;
        t[1][k][2] -= 0.004 * h;
    }
    t[2][0][2] += bel * 0.4;
    // The arm loops replace the chest rings' sides.
    for sd in ["l", "r"] {
        let (k0, k1, k2) = arm_k(sd);
        let r = &roots[sd];
        t[4][k0] = r[0];
        t[4][k1] = r[1];
        t[4][k2] = r[2];
        t[5][k2] = r[3];
        t[6][k2] = r[4];
        t[6][k1] = r[5];
        t[6][k0] = r[6];
        t[5][k0] = r[7];
    }
    // The shoulder ring's front and back meet the neck: they are the neck loop's.
    let zn = pj("neck_01").z;
    let neck_loop = [
        (0, (0.0, y_neck - 0.016 * h, zn + r_n * 1.3)),
        (1, (r_n * 0.95, y_neck - 0.008 * h, zn + r_n * 0.95)),
        (5, (r_n * 0.95, y_neck + 0.006 * h, zn - r_n * 0.95)),
        (6, (0.0, y_neck + 0.002 * h, zn - r_n * 1.3)),
    ];
    for (k, (xx, yy, zz)) in neck_loop {
        t[6][k] = v3(xx, yy, zz);
        if k == 1 || k == 5 {
            t[6][12 - k] = v3(-xx, yy, zz);
        }
    }

    let tw = |r: usize, k: usize| -> Weights {
        let yy = t[r][k][1];
        let w = sw(yy);
        let xx = t[r][k][0];
        let side = if xx > 0.0 { "r" } else { "l" };
        let thigh = one(bone(format!("thigh_{side}")));
        if r == 0 {
            return if xx.abs() > 0.3 * hw { mix(&[(0.62, &w), (0.38, &thigh)]) } else { mix(&[(0.76, &w), (0.12, &one("thigh_l")), (0.12, &one("thigh_r"))]) };
        }
        if r == 1 {
            return if xx.abs() > 0.3 * hw { mix(&[(0.9, &w), (0.1, &thigh)]) } else { w };
        }
        let clav = bone(format!("clavicle_{side}"));
        if r == 6 {
            if k == 0 || k == 6 {
                return vec![("spine_03", 0.6), ("neck_01", 0.4)];
            }
            if [1, 5, 7, 11].contains(&k) {
                return vec![("spine_03", 0.5), ("neck_01", 0.25), (clav, 0.25)];
            }
        }
        if r >= 4 && [1, 5, 7, 11].contains(&k) {
            return mix(&[(0.85, &w), (0.15, &one(clav))]);
        }
        w
    };

    let arm_w = |chi_index: usize, side: &str| -> Weights {
        let (cl, ua) = (bone(format!("clavicle_{side}")), bone(format!("upperarm_{side}")));
        // Root loop entries: 0-2 bottom (armpit), 3 back, 4-6 top, 7 front.
        if chi_index <= 2 {
            return vec![("spine_03", 0.5), (cl, 0.15), (ua, 0.35)];
        }
        if chi_index == 3 || chi_index == 7 {
            return vec![("spine_03", 0.35), (cl, 0.3), (ua, 0.35)];
        }
        vec![("spine_03", 0.1), (cl, 0.5), (ua, 0.4)]
    };

    let mut root_index: HashMap<(usize, usize), (usize, &str)> = HashMap::new();
    for sd in ["l", "r"] {
        let (k0, k1, k2) = arm_k(sd);
        for (i, rk) in [(4, k0), (4, k1), (4, k2), (5, k2), (6, k2), (6, k1), (6, k0), (5, k0)].into_iter().enumerate() {
            root_index.insert(rk, (i, sd));
        }
    }
    let mut tid: Vec<Vec<usize>> = Vec::new();
    for (r, row) in t.iter().enumerate() {
        let mut ids = Vec::new();
        for (k, &q) in row.iter().enumerate() {
            if r == 5 && (k == 3 || k == 9) {
                ids.push(usize::MAX);
                continue;
            }
            if let Some(&(i, sd)) = root_index.get(&(r, k)) {
                ids.push(c.add(q, arm_w(i, sd), "torso", None, Tags { arm: 0.5, blush: 0.0 }));
            } else {
                ids.push(c.add(q, tw(r, k), "torso", None, Tags::default()));
            }
        }
        tid.push(ids);
    }
    let crotch = c.add(
        v3(0.0, y_hip - (0.05 - 0.012 * yg) * h, -0.006 * h),
        vec![("pelvis", 0.4), ("thigh_l", 0.3), ("thigh_r", 0.3)],
        "torso",
        None,
        Tags::default(),
    );
    // Shoulder cap: the neck's sides.
    let mut cap: HashMap<&str, usize> = HashMap::new();
    for (sd, sx) in [("r", 1.0), ("l", -1.0)] {
        let id = c.add(
            v3(sx * r_n * 1.3, y_neck + 0.012 * h, zn - 0.1 * r_n),
            vec![("spine_03", 0.35), ("neck_01", 0.3), (bone(format!("clavicle_{sd}")), 0.35)],
            "torso",
            None,
            Tags::default(),
        );
        cap.insert(sd, id);
    }
    for r in 0..rows.len() - 1 {
        for k in 0..12 {
            let j = (k + 1) % 12;
            if (r == 4 || r == 5) && [2, 3, 8, 9].contains(&k) {
                continue;
            }
            c.face(&[tid[r][k], tid[r][j], tid[r + 1][j], tid[r + 1][k]]);
        }
    }
    let t7 = tid[6].clone();
    c.face(&[t7[1], t7[2], t7[3], cap["r"]]);
    c.face(&[cap["r"], t7[3], t7[4], t7[5]]);
    c.face(&[t7[11], t7[10], t7[9], cap["l"]]);
    c.face(&[cap["l"], t7[9], t7[8], t7[7]]);

    // Neck: the shoulder cap's loop up to a ring under the jaw, then the head.
    let n0 = vec![t7[0], t7[1], cap["r"], t7[5], t7[6], t7[7], cap["l"], t7[11]];
    c.neck_loop = n0.clone();
    let pr_head = pr.hh * h;
    let chin = h - pr_head;
    let y_n = chin - 0.01 * h + 0.005 * h * yg;
    let psi: Vec<f64> = (0..8).map(|i| radians(i as f64 * 45.0)).collect();
    let mut nh = ring_pts(v3(0.0, y_n, zn - 0.2 * r_n), x, z, r_n, r_n * 0.95, r_n * 1.0, &psi, 2.0);
    for k in [3, 4, 5] {
        // The nape is higher than the throat.
        nh[k][1] += 0.012 * h;
    }
    let nw: Weights = vec![("neck_01", 0.55), ("Head", 0.45)];
    let nh_id = c.ring(&nh, Per::All(&nw), "neck", None, RingTags::default());
    c.band(&n0, &nh_id, &[]);
    let jaw = head_cage(&mut c, family, stage);
    c.join(&nh_id, &jaw, &[0, 2, 4, 5, 6, 7, 8, 10]);

    // Arms and hands.
    for sd in ["r", "l"] {
        let (k0, k1, k2) = arm_k(sd);
        let lp = vec![tid[4][k0], tid[4][k1], tid[4][k2], tid[5][k2], tid[6][k2], tid[6][k1], tid[6][k0], tid[5][k0]];
        let chi: Vec<f64> = [135.0, 180.0, 225.0, 270.0, 315.0, 0.0, 45.0, 90.0].map(radians).to_vec();
        let nm = |n: &str| bone(format!("{n}_{sd}"));
        let (jsh, jel, jwr) = (pj(nm("upperarm")), pj(nm("lowerarm")), pj(nm("hand")));
        let d = unit(jwr - jsh);
        let u = unit(y - d * y.dot(&d));
        let upper = (jel - jsh).norm();
        let lower = (jwr - jel).norm();
        let rings: [(V3, f64, f64, Weights); 4] = [
            (jsh + d * (0.32 * upper), 1.0, 0.98, vec![(nm("upperarm"), 0.9), (nm("clavicle"), 0.1)]),
            (jel, 0.84, 0.84, vec![(nm("upperarm"), 0.5), (nm("lowerarm"), 0.5)]),
            (jel + d * (0.45 * lower), 0.80, 0.86, vec![(nm("lowerarm"), 1.0)]),
            (jwr - d * (0.02 * lower), 0.5, 0.68, vec![(nm("lowerarm"), 0.45), (nm("hand"), 0.55)]),
        ];
        let mut prev = lp;
        for (cc, ru, rf, w) in &rings {
            let pts: Vec<V3> = chi.iter().map(|&a| cc + u * (r_a * ru * se(a.cos(), 2.2)) + z * (r_a * rf * se(a.sin(), 2.2))).collect();
            let cur = c.ring(&pts, Per::All(w), "arm", None, RingTags { arm: Per::All(1.0), ..Default::default() });
            c.band(&prev, &cur, &[]);
            prev = cur;
        }
        let wrist = prev;
        // Palm: two 10-rings (4 across from the back of the hand (-z) to the thumb side, 1 deep),
        // the ball of the thumb swelling on the palm side, then the knuckle ring: the same 5
        // columns plus a vertex at each side.
        let k1j = pj(nm("fingers_01"));
        let lpm = (k1j - jwr).norm();
        // (A small child's hand is a little wider, its fingers plumper and shorter.)
        let (wp, tp) = ((0.80 + 0.12 * yg) * lpm, 0.30 * lpm);
        let zs: Vec<f64> = [-0.5, -0.25, 0.0, 0.25, 0.5].iter().zip([0.9, 1.0, 1.0, 1.0, 0.92]).map(|(&a, m)| a * wp * m).collect();
        let order: [(usize, f64); 10] = [(3, -1.0), (2, -1.0), (1, -1.0), (0, -1.0), (0, 1.0), (1, 1.0), (2, 1.0), (3, 1.0), (4, 1.0), (4, -1.0)];
        // (The thumb's ball: palm-side offsets (towards the palm, forward) of columns 3 and 4.)
        let palm_ring = |tt: f64, scale: f64, ball: f64| -> Vec<V3> {
            let cc = jwr + d * (tt * lpm) - u * 0.06 * tp;
            order
                .iter()
                .map(|&(k, v)| {
                    let swell = if v < 0.0 && k >= 3 { ball * [0.0, 0.0, 0.0, 0.6, 1.0][k] * lpm } else { 0.0 };
                    cc + z * (zs[k] * scale + 0.5 * swell) + u * ((tp / 2.0) * v * (if k == 0 || k == 4 { 0.85 } else { 1.0 }) - swell)
                })
                .collect()
        };
        let arm1 = || RingTags { arm: Per::All(1.0), ..Default::default() };
        let hand_w = one(nm("hand"));
        let p1 = c.ring(&palm_ring(0.32, 0.92, 0.05), Per::All(&hand_w), "hand", None, arm1());
        c.join(&wrist, &p1, &[0, 1, 2, 3, 5, 6, 7, 8]);
        let p2 = c.ring(&palm_ring(0.64, 1.0, 0.04), Per::All(&hand_w), "hand", None, arm1());
        c.band(&p1, &p2, &[8, 9]);
        // The knuckle ring: the back over the knuckles (about half hand, half fingers_01), the
        // palm's pads a little further out (more the fingers'), so the knuckles bend there;
        // between the fingers the back dips a little.
        let kn: Vec<V3> = {
            let cc = jwr - u * 0.02 * lpm;
            let (top, pad) = (0.11 * lpm, 0.15 * lpm);
            let mut out: Vec<V3> = order
                .iter()
                .map(|&(k, v)| {
                    let edge = if k == 0 || k == 4 { 0.85 } else { 1.0 };
                    if v > 0.0 {
                        cc + d * (1.0 * lpm) + z * (zs[k] * 0.98) + u * (top * edge - if k == 0 || k == 4 { 0.0 } else { 0.012 * lpm })
                    } else {
                        cc + d * (1.08 * lpm) + z * (zs[k] * 0.98) - u * (pad * edge)
                    }
                })
                .collect();
            // Order: palm side 3 .. 0, the pinky's side, back 0 .. 4, the index's side, palm side 4.
            out.insert(4, cc + d * (1.04 * lpm) + z * (zs[0] * 0.98 - 0.02 * lpm) - u * (0.01 * lpm));
            out.insert(10, cc + d * (1.04 * lpm) + z * (zs[4] * 0.98 + 0.02 * lpm) - u * (0.01 * lpm));
            out
        };
        let wk = |f: f64| -> Weights { vec![(nm("hand"), 1.0 - f), (nm("fingers_01"), f)] };
        let wks: Vec<Weights> = (0..12).map(|i| wk(if (5..10).contains(&i) { 0.45 } else if i == 4 || i == 10 { 0.6 } else { 0.7 })).collect();
        let p3 = c.ring(&kn, Per::Each(&wks.iter().collect::<Vec<_>>()), "hand", None, arm1());
        c.join(&p2, &p3, &[0, 1, 2, 3, 5, 6, 7, 8, 9, 11]);
        // Fingers, pinky (-z) to index: each leaves the knuckle ring between two of its columns,
        // a 6-ring (back two, the side towards the thumb, palm two, the other side) whose sides
        // between two fingers are one vertex in the crotch (towards the palm, past the knuckles),
        // so neighbours share the edges down into it and stand apart beyond it. Straight from
        // the knuckle to a ring just short of the middle joint (fingers_01), and a ring before
        // the tip (fingers_02) turned a little about that joint towards the palm (the rest pose's
        // curl), narrowing; a 6-sided cap. The middle joint is fingers_02's (rig.rs), so a fist
        // bends at the knuckle and there. (`ki`: the knuckle ring's index of column k, back or palm;
        // its sides are 4 and 10.)
        let ki = |k: usize, v: f64| {
            let i = order.iter().position(|&(a, b)| a == k && b == v).unwrap();
            i + usize::from(i >= 4) + usize::from(i >= 9)
        };
        let k2j = pj(nm("fingers_02"));
        let (pivot_d, pivot_u) = ((k2j - jwr).dot(&d), (k2j - jwr).dot(&u));
        let chub = 1.0 + 0.15 * yg;
        let wc = wk(0.9);
        let crotch: Vec<usize> = (1..4)
            .map(|k| c.add(jwr + d * (1.2 * lpm) + z * (zs[k] * 0.98) - u * (0.045 * lpm), wc.clone(), "hand", None, Tags { arm: 1.0, blush: 0.0 }))
            .collect();
        let w01 = one(nm("fingers_01"));
        let w02 = one(nm("fingers_02"));
        for fi in 0..4 {
            let side = |k: isize| if k < 0 { p3[4] } else if k > 2 { p3[10] } else { crotch[k as usize] };
            let loop_f = vec![p3[ki(fi, 1.0)], p3[ki(fi + 1, 1.0)], side(fi as isize), p3[ki(fi + 1, -1.0)], p3[ki(fi, -1.0)], side(fi as isize - 1)];
            // (Half width, half depth; distal length past the joint, in palm lengths. The palm side is fuller.)
            let (hw, hd) = ([0.088, 0.098, 0.1, 0.098][fi] * chub, [0.095, 0.105, 0.107, 0.105][fi] * chub);
            let tip = [0.12, 0.16, 0.18, 0.16][fi] * (1.0 - 0.3 * yg);
            let zf = (zs[fi] + zs[fi + 1]) / 2.0 * 0.98;
            let spread = (fi as f64 - 1.5) * 0.05;
            // (A 6-ring round cc facing ax: half width, half depth on the back and on the palm side.)
            let hex = |cc: V3, ax: V3, up: V3, hw: f64, back: f64, palm: f64| -> Vec<V3> {
                let sd = unit(up.cross(&ax)) * sign(up.cross(&ax).dot(&z));
                [(-0.5, 0.87), (0.5, 0.87), (1.0, 0.0), (0.5, -0.87), (-0.5, -0.87), (-1.0, 0.0)]
                    .iter()
                    .map(|&(a, b): &(f64, f64)| cc + sd * (a * hw * lpm) + up * (b * if b > 0.0 { back } else { palm } * lpm))
                    .collect()
            };
            let pivot = jwr + d * pivot_d + u * pivot_u + z * (zf + spread * (pivot_d - lpm));
            let a = hex(pivot - d * (0.04 * lpm), d, u, hw * 1.1, hd * 1.05, hd * 1.5);
            let ra = c.ring(&a, Per::All(&w01), "hand", None, arm1());
            c.band(&loop_f, &ra, &[]);
            let rho: f64 = 0.2;
            let (ad, au) = (d * rho.cos() - u * rho.sin(), u * rho.cos() + d * rho.sin());
            let b = hex(pivot + ad * (tip * lpm) + z * (spread * tip * lpm), ad, au, hw * 0.95, hd * 0.85, hd * 1.15);
            let rb = c.ring(&b, Per::All(&w02), "hand", None, arm1());
            c.band(&ra, &rb, &[]);
            c.cap(&rb);
        }
        // Thumb: from the palm between the two rings, round its thumb side and the palm side next
        // to it (the ball of the thumb), a 6-ring at its knuckle (thumb_02) and one before the
        // tip, narrowing like a finger; a 6-sided cap. Each ring is the base loop turned from its
        // own plane to facing along the thumb (wrist / tip of the hand, back / palm keep their
        // sides), so the bands don't twist.
        let tl = vec![p1[8], p1[9], p1[0], p2[0], p2[9], p2[8]];
        let base: Vec<V3> = tl.iter().map(|&i| c.p[i]).collect();
        let m = mean_rows(&base);
        let up = unit((base[0] - base[2]) - d * (base[0] - base[2]).dot(&d));
        let out = unit(d.cross(&up));
        let out = out * sign(out.dot(&(z - u)));
        let t2 = pj(nm("thumb_02"));
        let tip = unit(d * 0.85 - u * 0.45 + z * 0.22);
        let mut prev_t = tl;
        let thumb: [(V3, V3, f64, f64, Weights); 2] = [
            (t2, unit(unit(t2 - m) + tip), 0.15, 0.135, vec![(nm("thumb_01"), 0.5), (nm("thumb_02"), 0.5)]),
            (t2 + tip * (0.42 * lpm), tip, 0.11, 0.1, vec![(nm("thumb_02"), 1.0)]),
        ];
        for (cc, ax, thw, tht, w) in &thumb {
            let r = turn(out, *ax);
            let (rd, ru) = (r * d, r * up);
            let pts: Vec<V3> = [(-0.5, 0.87), (-1.0, 0.0), (-0.5, -0.87), (0.5, -0.87), (1.0, 0.0), (0.5, 0.87)].iter().map(|&(a, b)| cc + (rd * (a * thw) + ru * (b * tht)) * lpm).collect();
            let cur = c.ring(&pts, Per::All(w), "hand", None, arm1());
            c.band(&prev_t, &cur, &[]);
            prev_t = cur;
        }
        c.cap(&prev_t);
    }

    // Legs and feet.
    for sd in ["r", "l"] {
        let (lp, psi): (Vec<usize>, Vec<f64>) = if sd == "r" {
            let mut l = tid[0][0..7].to_vec();
            l.push(crotch);
            (l, [-45.0, 0.0, 45.0, 90.0, 135.0, 180.0, -135.0, -90.0].map(radians).to_vec())
        } else {
            let mut l = tid[0][6..12].to_vec();
            l.extend([tid[0][0], crotch]);
            (l, [135.0, 180.0, -135.0, -90.0, -45.0, 0.0, 45.0, 90.0].map(radians).to_vec())
        };
        let nm = |n: &str| bone(format!("{n}_{sd}"));
        let (jt, jk, ja) = (pj(nm("thigh")), pj(nm("calf")), pj(nm("foot")));
        let rings: [(V3, f64, f64, f64, Weights); 4] = [
            (jt + (jk - jt) * 0.24, 1.0, 0.98, 1.06, vec![(nm("thigh"), 0.92), ("pelvis", 0.08)]),
            (jt + (jk - jt) * 0.86, 0.68, 0.70, 0.64, vec![(nm("thigh"), 0.75), (nm("calf"), 0.25)]),
            (jk + (ja - jk) * 0.22, 0.64, 0.64, 0.74, vec![(nm("calf"), 0.85), (nm("thigh"), 0.15)]),
            (ja + (jk - ja) * 0.1, 0.42, 0.42, 0.44, vec![(nm("calf"), 0.5), (nm("foot"), 0.5)]),
        ];
        let mut prev = lp;
        let sx = if sd == "r" { 1.0 } else { -1.0 };
        for (k, (cc, a, bf, bb, w)) in rings.iter().enumerate() {
            let mut pts = ring_pts(*cc, x, z, r_th * a, r_th * bf, r_th * bb, &psi, 2.1);
            if k == 0 {
                // The thigh's top stops short of the midline (a baby's plump thighs would cross
                // it): the strip between it and the crotch would fold over when the legs spread.
                for q in &mut pts {
                    q[0] = sx * (sx * q[0]).max(CROTCH_GAP * r_th);
                }
            }
            let cur = c.ring(&pts, Per::All(w), "leg", None, RingTags::default());
            c.band(&prev, &cur, &[]);
            prev = cur;
        }
        // The foot: rings turning from the ankle to point forward, flat underneath, a round toe.
        let jb = pj(nm("ball"));
        let fl = jb[2] - ja[2];
        let fw = 0.33 * fl * (if female { 0.94 } else { 1.0 });
        let fx = ja[0];
        let feet: [(V3, f64, f64, f64, f64, Weights); 2] = [
            (v3(fx, 0.5 * ja[1], ja[2] - 0.02 * fl), 45.0, fw * 0.92, 0.38 * fl, 0.38 * fl, one(nm("foot"))),
            (v3(fx, 0.42 * ja[1], jb[2] - 0.2 * fl), 86.0, fw * 1.0, 0.29 * fl, 0.30 * fl, vec![(nm("ball"), 0.5), (nm("foot"), 0.5)]),
        ];
        for (cc, al, a, bt, bb, w) in &feet {
            let al = radians(*al);
            let vv = v3(0.0, al.sin(), al.cos());
            let mut pts = ring_pts(*cc, x, vv, *a, *bt, *bb, &psi, 2.6);
            for q in &mut pts {
                q[1] = q[1].max(0.004 * h);
            }
            let cur = c.ring(&pts, Per::All(w), "foot", None, RingTags::default());
            c.band(&prev, &cur, &[]);
            prev = cur;
        }
        let tip = c.add(v3(fx, 0.4 * ja[1], jb[2] + 0.04 * fl), one(nm("ball")), "foot", None, Tags::default());
        for i in [1, 3, 5, 7] {
            c.face(&[prev[i], prev[(i + 1) % 8], prev[(i + 2) % 8], tip]);
        }
    }
    (c, p)
}

/// The cage's faces (oriented), its subdivision and limit operators (shared by every body).
pub struct Topology {
    pub faces: Vec<Vec<usize>>,
    pub n: usize,
    pub s: Subdivision,
    /// Cage -> level 1 on the limit surface.
    pub body: Csr,
    /// Cage -> its own limit points, and its LU (factored once).
    pub a: Csr,
    lu: LU<f64, Dyn, Dyn>,
    pub lod_tris: Vec<[usize; 3]>,
    /// A second level, only to sample the head densely (head_frame).
    pub s2: Subdivision,
    pub fine: Csr,
    pub fine_m: Csr,
}

impl Topology {
    fn new(cage: &Cage) -> Self {
        let pos = &cage.p;
        let n = pos.len();
        let faces = orient(&cage.faces, pos);
        if let Err(e) = check_closed(&faces, Some(n)) {
            panic!("{e}");
        }
        // The LOD leaves the face's finer cage out (its vertices stay, unused).
        let lod_faces = orient(&cage.lod_faces, pos);
        if let Err(e) = check_closed(&lod_faces, None) {
            panic!("LOD: {e}");
        }
        let s = subdivision(&faces, n);
        let n1 = s.m.rows;
        let l1 = limit_matrix(&s.quads, n1);
        let body = l1.matmul(&s.m);
        let a = body.head(n);
        let lu = a.to_dense().lu();
        let lod_tris = polygon_triangles(&lod_faces);
        let q1: Vec<Vec<usize>> = s.quads.iter().map(|q| q.to_vec()).collect();
        let s2 = subdivision(&q1, n1);
        let l2 = limit_matrix(&s2.quads, s2.m.rows);
        let fine = l2.matmul(&s2.m).matmul(&s.m);
        let fine_m = s2.m.matmul(&s.m);
        Topology { faces, n, s, body, a, lu, lod_tris, s2, fine, fine_m }
    }

    /// Cage positions whose limit points are `t` (splu's solve, here a dense LU).
    fn solve(&self, t: &[V3]) -> Vec<V3> {
        let mut out = vec![V3::zeros(); t.len()];
        for k in 0..3 {
            let rhs = DVector::from_iterator(t.len(), t.iter().map(|p| p[k]));
            let x = self.lu.solve(&rhs).expect("singular cage");
            for (o, v) in out.iter_mut().zip(x.iter()) {
                o[k] = *v;
            }
        }
        out
    }
}

pub fn topology() -> &'static Topology {
    static TOPO: OnceLock<Topology> = OnceLock::new();
    TOPO.get_or_init(|| Topology::new(&body_cage("male", BASE_STAGE).0))
}

/// Tags per vertex, as separate arrays.
#[derive(Clone)]
pub struct TagArrays {
    pub arm: Vec<f64>,
    pub blush: Vec<f64>,
}

impl TagArrays {
    fn of(t: &[Tags]) -> Self {
        TagArrays { arm: t.iter().map(|t| t.arm).collect(), blush: t.iter().map(|t| t.blush).collect() }
    }

    fn subdivided(&self, m: &Csr) -> Self {
        TagArrays { arm: m.apply1(&self.arm), blush: m.apply1(&self.blush) }
    }
}

/// The body of a family at a stage: cage, level-1 mesh, LOD and the dense head samples.
pub struct Surface {
    pub p: [V3; NB],
    pub cage: Cage,
    pub ctrl: Vec<V3>,
    pub lod_pos: Vec<V3>,
    pub lod_dense: Vec<Dense>,
    pub lod_tags: TagArrays,
    pub lod_region: Vec<[f64; NR]>,
    pub lod_nor: Vec<V3>,
    pub pos: Vec<V3>,
    pub dense: Vec<Dense>,
    pub tags: TagArrays,
    pub region: Vec<[f64; NR]>,
    pub nor: Vec<V3>,
    pub ears: [[V3; 2]; 2],
    pub head_samples: Vec<V3>,
    pub head_normals: Vec<V3>,
    pub head_tris: Vec<[V3; 3]>,
    pub cranium: Vec<V3>,
}

static SURFACES: Cache<(String, String), Surface> = Cache::new();

/// The body of `family` at `stage` (cached).
pub fn surface(family: &str, stage: &str) -> Arc<Surface> {
    SURFACES.get(key(family, stage), || make_surface(family, stage))
}

fn make_surface(family: &str, stage: &str) -> Surface {
    let topo = topology();
    let (cage, p) = body_cage(family, stage);
    let arr = cage.arrays();
    assert_eq!(arr.t.len(), topo.n);
    // The smooth shape is interpolated exactly; features are offsets on the fitted cage.
    let ctrl: Vec<V3> = topo.solve(&arr.t).iter().zip(&arr.d).map(|(c, d)| c + d).collect();
    let s = &topo.s;
    let pos = topo.body.apply(&ctrl);
    let region: Vec<[f64; NR]> = cage
        .region
        .iter()
        .map(|r| {
            let mut v = [0.0; NR];
            v[super::body::region(r)] = 1.0;
            v
        })
        .collect();
    let lod_pos = topo.a.apply(&ctrl);
    let tags = TagArrays::of(&arr.tags);
    let dense1 = s.m.apply_n(&arr.dense);
    let tags1 = tags.subdivided(&s.m);
    let region1 = s.m.apply_n(&region);
    // Head and neck, densely, without the ears (for the head interface's sdf and surface_z).
    let fpos = topo.fine.apply(&ctrl);
    // Normals from the finer (level-2) limit surface: smooth shading on the coarse level 1 (whose
    // vertices are the first of level 2's).
    let ftri: Vec<usize> = triangles(&topo.s2.quads, &fpos).concat();
    let fnor = smooth_normals(&fpos, &ftri);
    let nor = fnor[..pos.len()].to_vec();
    let lod_nor = fnor[..topo.n].to_vec();
    let freg = topo.fine_m.apply_n(&region);
    let (ri_head, ri_ear, ri_neck) = (super::body::region("head"), super::body::region("ear"), super::body::region("neck"));
    let mut ears = [[V3::zeros(); 2]; 2];
    for (e, sx) in [-1.0, 1.0].into_iter().enumerate() {
        let q: Vec<&V3> = fpos.iter().zip(&freg).filter(|(p, r)| r[ri_ear] > 0.3 && sign(p.x) == sx).map(|(p, _)| p).collect();
        let mx = q.iter().fold(V3::repeat(f64::NEG_INFINITY), |a, p| a.sup(p));
        let mn = q.iter().fold(V3::repeat(f64::INFINITY), |a, p| a.inf(p));
        ears[e] = [(mx + mn) / 2.0, (mx - mn) / 2.0];
    }
    let flat_ctrl: Vec<V3> = ctrl.iter().zip(&arr.d).zip(&cage.region).map(|((c, d), r)| if *r == "ear" { c - d } else { c - d * 0.0 }).collect();
    let fpos = topo.fine.apply(&flat_ctrl);
    let hmask: Vec<bool> = freg.iter().map(|r| (r[ri_head] + r[ri_ear]) + r[ri_neck] > 0.5).collect();
    let q = &topo.s2.quads;
    let qm: Vec<[usize; 4]> = q.iter().filter(|q| q.iter().all(|&v| hmask[v])).copied().collect();
    let tri = triangles(&qm, &fpos);
    let nor2 = smooth_normals(&fpos, &triangles(q, &fpos).concat());
    let used = unique(&tri.concat());
    // Samples: the vertices and the triangles' centres (about 4 mm apart on an adult head).
    let mut head_samples: Vec<V3> = used.iter().map(|&i| fpos[i]).collect();
    let mut head_normals: Vec<V3> = used.iter().map(|&i| nor2[i]).collect();
    for t in &tri {
        let cn = nor2[t[0]] + nor2[t[1]] + nor2[t[2]];
        head_normals.push(cn / cn.norm());
        head_samples.push((fpos[t[0]] + fpos[t[1]] + fpos[t[2]]) / 3.0);
    }
    let head_tris = tri.iter().map(|t| [fpos[t[0]], fpos[t[1]], fpos[t[2]]]).collect();
    let pr = proportions(family, stage);
    let hh = pr.hh * pr.h;
    let cranium = fpos.iter().zip(&freg).filter(|(p, r)| r[ri_head] > 0.99 && p.y > pr.h - hh + 0.6 * hh).map(|(p, _)| *p).collect();
    Surface {
        p,
        cage,
        ctrl,
        lod_pos,
        lod_dense: arr.dense,
        lod_tags: tags,
        lod_region: region,
        lod_nor,
        pos,
        dense: dense1,
        tags: tags1,
        region: region1,
        nor,
        ears,
        head_samples,
        head_normals,
        head_tris,
        cranium,
    }
}

static TRIS: Cache<String, Vec<usize>> = Cache::new();

/// Level-1 triangles: quads split along their shorter diagonal on the family's base body.
pub fn tris(family: &str) -> Arc<Vec<usize>> {
    TRIS.get(family.to_string(), || triangles(&topology().s.quads, &surface(family, BASE_STAGE).pos).concat())
}

/// The seat drop, eye height and grip of a body.
#[derive(Clone, Copy)]
pub struct Meta {
    pub seat_drop: f64,
    pub eye_y: f64,
    pub grip: [f64; 3],
}

pub struct Body {
    pub body: Mesh,
    pub lod: Mesh,
    pub meta: Meta,
    pub joints: [V3; NB],
}

/// The body of `family` at `stage` (and its low-detail copy); see the module docs.
pub fn build_body(family: &str, stage: &str) -> Body {
    let s = surface(family, stage);
    let topo = topology();
    let body = Mesh { pos: s.pos.clone(), idx: tris(family).to_vec(), dense: s.dense.clone(), arm: s.tags.arm.clone(), nor: s.nor.clone(), ..Mesh::default() };
    let lod = Mesh { pos: s.lod_pos.clone(), idx: topo.lod_tris.concat(), dense: s.lod_dense.clone(), arm: s.lod_tags.arm.clone(), nor: s.lod_nor.clone(), ..Mesh::default() };
    let meta = Meta { seat_drop: seat_drop(&body, &s.p), eye_y: s.p[b("eye_l")].y, grip: grip(family, stage) };
    Body { body, lod, meta, joints: s.p }
}

fn rx(a: f64) -> M3 {
    let (c, s) = (a.cos(), a.sin());
    M3::new(1.0, 0.0, 0.0, 0.0, c, -s, 0.0, s, c)
}

/// Seated (thighs turned `flex` degrees forward about the hips, a quarter turn for `seat_drop`; shins
/// straight down from the knees; every other bone at rest): each bone's skinning transform
/// `x -> R x + t` (seat_drop; the shells' seated corrective, clothes.rs).
pub fn seated_pose(p: &[V3; NB], flex: f64) -> Vec<(M3, V3)> {
    let up = rx(radians(-flex));
    let down = rx(radians(flex));
    let mut mats: Vec<(M3, V3)> = vec![(M3::identity(), V3::zeros()); NB];
    for sd in ["l", "r"] {
        let (jt, jk) = (p[b(&format!("thigh_{sd}"))], p[b(&format!("calf_{sd}"))]);
        mats[b(&format!("thigh_{sd}"))] = (up, jt - up * jt);
        let knee = up * jk + (jt - up * jt);
        let r = up * down;
        for bn in ["calf", "foot", "ball"] {
            mats[b(&format!("{bn}_{sd}"))] = (r, knee - r * jk);
        }
    }
    mats
}

/// Seated (`seated_pose`): the pelvis joint's height above the lowest skin under the hips.
fn seat_drop(body: &Mesh, p: &[V3; NB]) -> f64 {
    let mats = seated_pose(p, 90.0);
    let n = body.pos.len();
    let mut out = vec![V3::zeros(); n];
    for (bi, (r, t)) in mats.iter().enumerate() {
        if !body.dense.iter().any(|d| d[bi] != 0.0) {
            continue;
        }
        for (o, (q, d)) in out.iter_mut().zip(body.pos.iter().zip(&body.dense)) {
            *o += (super::prims::rot(r, *q) + t) * d[bi];
        }
    }
    let hips = ["pelvis", "thigh_l", "thigh_r", "spine_01"].map(b);
    let low = out
        .iter()
        .zip(&body.dense)
        .filter(|(_, d)| hips.iter().fold(0.0, |s, &i| s + d[i]) > 0.6)
        .map(|(o, _)| o.y)
        .fold(f64::INFINITY, f64::min);
    p[b("pelvis")].y - low
}

/// The left palm's centre (just off the skin) relative to the wrist.
fn grip(family: &str, stage: &str) -> [f64; 3] {
    let s = surface(family, stage);
    let (wr, k1) = (s.p[b("hand_l")], s.p[b("fingers_01_l")]);
    let d = unit(k1 - wr);
    let u = unit(v3(0.0, 1.0, 0.0) - d * d[1]);
    let lp = (k1 - wr).norm();
    let g = d * 0.62 * lp - u * 0.28 * lp;
    [g.x, g.y, g.z]
}

// Fields and tone -----------------------------------------------------------------------------

/// A smooth even function of the angle round the neck through the samples (least squares).
fn fourier(theta: &[f64], y: &[f64], n: usize) -> impl Fn(f64) -> f64 {
    let a = DMatrix::from_fn(theta.len(), n, |i, k| (k as f64 * theta[i]).cos());
    let coef = lstsq(&a, &DVector::from_column_slice(y));
    move |t: f64| (0..n).fold(0.0, |s, k| s + (k as f64 * t).cos() * coef[k])
}

/// Body fields (n, 4) of positions on the base body (see the module docs).
pub fn fields_of(family: &str, pos: &[V3], arm: &[f64], region: &[[f64; NR]], p: &[V3; NB], head: &HeadFrame) -> Vec<[f64; 4]> {
    let female = family == "female";
    let sh = p[b("upperarm_r")];
    let wr = p[b("hand_r")];
    let axis = wr - sh;
    let l = axis.norm();
    // Necklines: cones round the neck (distance from its axis plus depth below a curve through the
    // shoulder cap's neck loop), so the edge crosses the sloping shoulders steeply and stays clean
    // on coarse triangles (a level cut would graze them).
    let s = surface(family, BASE_STAGE);
    let lp: Vec<V3> = s.cage.neck_loop.iter().map(|&i| s.cage.p[i]).collect();
    let zn = p[b("neck_01")].z;
    let rho_l: Vec<f64> = lp.iter().map(|q| q.x.hypot(q.z - zn)).collect();
    let th_l: Vec<f64> = lp.iter().map(|q| q.x.atan2(q.z - zn)).collect();
    let rho0 = mean(&rho_l);
    let ys: Vec<f64> = lp.iter().zip(&rho_l).map(|(q, r)| q.y - (r - rho0)).collect();
    let thetas: Vec<f64> = th_l.iter().copied().chain(th_l.iter().map(|t| -t)).collect();
    let yy: Vec<f64> = ys.iter().chain(ys.iter()).copied().collect();
    let refc = fourier(&thetas, &yy, 3);
    let (ri_head, ri_ear) = (region_ix("head"), region_ix("ear"));
    pos.iter()
        .enumerate()
        .map(|(i, q)| {
            let (x, y, z) = (q.x, q.y, q.z);
            let pa = v3(x.abs(), y, z);
            let t = (pa - sh).dot(&axis) / (l * l);
            let sleeve = arm[i] * t - (1.0 - arm[i]);
            let rho = x.hypot(z - zn);
            let th = x.atan2(z - zn);
            let y0 = refc(th);
            let crew = (rho - rho0) + (y0 - 0.008) - y;
            let front = clip(th.cos(), 0.0, 1.0);
            let scoop = if female {
                (rho - 1.2 * rho0) + (y0 - 0.012 - 0.07 * (front * front)) - y
            } else {
                (rho - rho0) + (y0 - 0.01 - 0.085 * clip(1.0 - th.abs() / 0.85, 0.0, 1.0)) - y
            };
            let mut f = [sleeve, y, crew, scoop];
            let hd = region[i][ri_head] + region[i][ri_ear];
            if hd > 0.5 {
                let scalp = head.scalp(*q) * (if region[i][ri_ear] < 0.5 { 1.0 } else { 0.0 });
                f[2] = -1.0 - scalp;
                f[3] = -1.0 - scalp;
            }
            f
        })
        .collect()
}

fn region_ix(name: &str) -> usize {
    region(name)
}

/// Clothes fields and tone of the (base stage) body or its LOD; the face's pieces after the skin.
pub fn body_fields_and_tone(family: &str, mesh: &Mesh, p: &[V3; NB], head: &HeadFrame) -> (Vec<[f64; 4]>, Vec<[f64; 4]>) {
    let s = surface(family, BASE_STAGE);
    let n = mesh.pos.len();
    let lod = n < s.pos.len();
    let (pos, tags, region) = if lod { (&s.lod_pos, &s.lod_tags, &s.lod_region) } else { (&s.pos, &s.tags, &s.region) };
    let k = pos.len();
    let mut f = fields_of(family, pos, &tags.arm, region, p, head);
    // Blush (x) and, on the face, the age folds (z; the shader deepens them with age).
    let ri_head = region_ix("head");
    let mut tone: Vec<[f64; 4]> = (0..k).map(|i| [clip(tags.blush[i], 0.0, 1.0), 0.0, if region[i][ri_head] > 0.5 { head.fold(pos[i]) } else { 0.0 }, 0.0]).collect();
    if n > k {
        // The face's pieces (lids, lips) inside the body: head fields; the nearest skin's tone.
        let extra = &mesh.pos[k..];
        let mut onehead = [0.0; NR];
        onehead[region_ix("head")] = 1.0;
        f.extend(fields_of(family, extra, &vec![0.0; n - k], &vec![onehead; n - k], p, head));
        let tree = KdTree::new(pos);
        for q in extra {
            let t = tone[tree.nearest(q)];
            tone.push(t);
        }
    }
    (f, tone)
}

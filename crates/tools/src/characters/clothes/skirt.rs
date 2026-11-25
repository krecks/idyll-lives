//! Skirts: their profile, their shell, and how they sit when seated.

use super::*;

/// The skirt's rows (heights), axis z, angles and radii (rows x segs); `hug`: the radii without the
/// flare (the body's sections with their hollows bridged, plus the ease), where it lies seated.
pub(super) struct SkirtProfile {
    pub(super) ys: Vec<f64>,
    pub(super) cz: f64,
    pub(super) ang: Vec<f64>,
    pub(super) r: Vec<Vec<f64>>,
    pub(super) hug: Vec<Vec<f64>>,
}

pub(super) static SKIRTS: Cache<(String, String, u64, usize, usize), SkirtProfile> = Cache::new();

/// The body's sections (torso and legs, sampled between the rows) plus ease, snug over the hips
/// and flaring below.
pub(super) fn skirt_profile(
    family: &str,
    stage: &str,
    waist: f64,
    segs: usize,
    rows: usize,
) -> Arc<SkirtProfile> {
    SKIRTS.get(
        (
            family.to_string(),
            stage.to_string(),
            waist.to_bits(),
            segs,
            rows,
        ),
        || {
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
            let ys: Vec<f64> = (0..rows)
                .map(|i| stage_y(yb0 + (yb1 - yb0) * (i as f64 / (rows - 1) as f64)))
                .collect();
            let cz = p[b("pelvis")].z;
            let ang = linspace_open(0.0, 2.0 * PI, segs);
            let (rt, rl) = (region("torso"), region("leg"));
            let keep: Vec<bool> = (0..s.pos.len())
                .map(|i| s.tags.arm[i] < 0.1 && s.region[i][rt] + s.region[i][rl] > 0.5)
                .collect();
            let tri = surface_tris(family);
            let step = (ys[0] - ys[rows - 1]) / (rows - 1) as f64;
            let mut r = vec![vec![0.0; segs]; rows];
            let width = PI / segs as f64 * 1.6;
            for (i, &y) in ys.iter().enumerate() {
                let mut offs = vec![-0.5, -0.25, 0.0, 0.25, 0.5];
                if i == 0 {
                    offs.push(0.75);
                }
                let pts: Vec<V3> = offs
                    .iter()
                    .flat_map(|&f| section(&s.pos, &tri, y + f * step, &keep))
                    .collect();
                let va: Vec<f64> = pts
                    .iter()
                    .map(|q| py_mod(q.x.atan2(q.z - cz), 2.0 * PI))
                    .collect();
                let vr: Vec<f64> = pts.iter().map(|q| q.x.hypot(q.z - cz)).collect();
                for (j, &a) in ang.iter().enumerate() {
                    let m = va
                        .iter()
                        .zip(&vr)
                        .filter(|&(&v, _)| (py_mod(v - a + PI, 2.0 * PI) - PI).abs() < width)
                        .map(|(_, &r)| r)
                        .fold(f64::NEG_INFINITY, f64::max);
                    r[i][j] = if m.is_finite() {
                        m
                    } else if i > 0 {
                        r[i - 1][j]
                    } else {
                        0.1
                    };
                }
            }
            let r: Vec<Vec<f64>> = r
                .iter()
                .map(|row| {
                    (0..segs)
                        .map(|j| {
                            row[j].max(
                                (row[(j + segs - 1) % segs] + 2.0 * row[j] + row[(j + 1) % segs])
                                    / 4.0,
                            )
                        })
                        .collect()
                })
                .collect();
            let k = proportions(family, stage).h / 1.7;
            let t = linspace(0.0, 1.0, rows);
            let top: Vec<f64> = r[0].iter().map(|v| v + 0.006 * k).collect();
            let hem: Vec<f64> = (0..segs)
                .map(|j| r.iter().map(|row| row[j]).fold(f64::NEG_INFINITY, f64::max) + 0.05 * k)
                .collect();
            let flared: Vec<Vec<f64>> = (0..rows)
                .map(|i| {
                    (0..segs)
                        .map(|j| {
                            (r[i][j] + 0.006 * k).max(
                                top[j]
                                    + (hem[j] - top[j])
                                        * clip((t[i] - 0.2) / 0.8, 0.0, 1.0).powf(0.9),
                            )
                        })
                        .collect()
                })
                .collect();
            let hug = (0..rows)
                .map(|i| {
                    bridged(&r[i], 2.0 * PI / segs as f64)
                        .iter()
                        .zip(&flared[i])
                        .map(|(h, f)| (h + 0.006 * k).min(*f))
                        .collect()
                })
                .collect();
            SkirtProfile {
                ys,
                cz,
                ang,
                r: flared,
                hug,
            }
        },
    )
}

/// A ring of radii (`step` rad apart, round the whole ring) with its hollows bridged: each no less
/// than where the chord between its neighbours crosses its ray (the cloth spans the gap between the legs).
pub(super) fn bridged(r: &[f64], step: f64) -> Vec<f64> {
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
pub(super) fn over_skirt(
    pos: &[V3],
    family: &str,
    stage: &str,
    cuts: &Cuts,
    lift: f64,
    hug: bool,
) -> Vec<V3> {
    let sp = skirt_profile(
        family,
        stage,
        cuts.get("bottom.skirt").unwrap().waist.unwrap(),
        SKIRT_SEGS,
        SKIRT_ROWS,
    );
    let (ys, cz, na) = (&sp.ys, sp.cz, sp.ang.len());
    let ny = ys.len();
    pos.iter()
        .map(|q| {
            let a = py_mod(q.x.atan2(q.z - cz), 2.0 * PI);
            let r = q.x.hypot(q.z - cz);
            let j = a / (2.0 * PI) * na as f64;
            let j0 = (j.floor() as usize) % na;
            let f = j - j.floor();
            let yi = clip(
                (ys[0] - q.y) / (ys[0] - ys[ny - 1]) * (ny - 1) as f64,
                0.0,
                (ny - 1) as f64,
            );
            let i0 = (yi.floor() as usize).min(ny - 2);
            let g = yi - i0 as f64;
            let rr = |ii: usize, jj: usize| {
                if hug {
                    sp.hug[ii % ny][jj % na]
                } else {
                    sp.r[ii % ny][jj % na]
                }
            };
            let rv = 0.0
                + (1.0 - g) * (1.0 - f) * rr(i0, j0)
                + (1.0 - g) * f * rr(i0, j0 + 1)
                + g * (1.0 - f) * rr(i0 + 1, j0)
                + g * f * rr(i0 + 1, j0 + 1);
            let inside = q.y <= ys[0] + 0.03 && q.y >= ys[ny - 1];
            let need = if inside {
                (rv + lift - r).max(0.0)
            } else {
                0.0
            };
            q + v3(a.sin(), 0.0, a.cos()) * need
        })
        .collect()
}

/// An A-line skirt from the waistband to above the knees.
pub(super) fn skirt_shell(
    family: &str,
    stage: &str,
    cuts: &Cuts,
    segs: usize,
    rows: usize,
) -> Mesh {
    let cut = cuts.get("bottom.skirt").unwrap();
    let sp = skirt_profile(family, stage, cut.waist.unwrap(), segs, rows);
    let (ys, cz, ang) = (&sp.ys, sp.cz, &sp.ang);
    let k = proportions(family, stage).h / 1.7;
    let (yb0, yb1) = (ys[0], ys[rows - 1]);
    let shape = |r: &[Vec<f64>]| {
        let g: Vec<Vec<V3>> = (0..rows)
            .map(|i| {
                (0..segs)
                    .map(|j| v3(ang[j].sin() * r[i][j], ys[i], cz + ang[j].cos() * r[i][j]))
                    .collect()
            })
            .collect();
        // Hem: a band turned up inside the bottom edge (the cloth's thickness).
        let hem_in: Vec<V3> = (0..segs)
            .map(|j| {
                g[rows - 1][j] - v3(ang[j].sin(), 0.0, ang[j].cos()) * 0.004 * k
                    + v3(0.0, 0.012 * k, 0.0)
            })
            .collect();
        let (mut pos, mut idx) = grid(&g, true);
        let (hp, hi) = grid(&[g[rows - 1].clone(), hem_in], true);
        pos.extend(hp);
        idx.extend(hi.iter().map(|&i| i + rows * segs));
        (pos, idx)
    };
    let (pos, idx) = shape(&sp.r);
    let n = pos.len();
    let tl = linspace(0.0, 1.0, rows);
    let t: Vec<f64> = (0..n)
        .map(|i| if i < rows * segs { tl[i / segs] } else { 1.0 })
        .collect();
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
    let fields = (0..n)
        .map(|i| [(1.0 - t[i]) * (yb0 - yb1), FLAG_SKIRT, BOTTOM, 0.0])
        .collect();
    let body_n = (rows - 1) * segs * 6;
    let tri = outward(&pos[..rows * segs], idx[..body_n].to_vec(), (0.0, cz));
    let tri = if tri != idx[..body_n] {
        flip(&idx)
    } else {
        idx
    };
    let nor = smooth_normals(&pos, &tri);
    // Seated, it lies on the thighs: without its flare, and clear of the skin.
    let tone = seated_tone(
        family,
        stage,
        &pos,
        &dense,
        &shape(&sp.hug).0,
        SKIRT_CLEAR * k,
    );
    Mesh {
        pos,
        idx: tri,
        nor,
        dense,
        fields,
        tone,
        ..Mesh::default()
    }
}

/// The thighs' turn forward from the hips (degrees) the seated corrective is made for: the `sit`
/// clip's (clips/seated.ts), which the runtime weighs it by (characters.ts `SIT_FLEX`).
pub(super) const SIT_FLEX: f64 = 76.0;
/// Seated, how far (m) a shell keeps off the skin at least: the skirt, and a jacket (over it).
pub(super) const SKIRT_CLEAR: f64 = 0.005;
pub(super) const JACKET_CLEAR: f64 = 0.008;

/// A body seated (`seated_pose`): its skin skinned so, the skin's normals, a tree of its vertices,
/// and the seat's height (the lowest skin under the hips, as `seat_drop` has it).
pub(super) struct Seated {
    pub(super) pose: Vec<(M3, V3)>,
    pub(super) pos: Vec<V3>,
    pub(super) nor: Vec<V3>,
    pub(super) tree: KdTree,
    pub(super) seat: f64,
}

pub(super) static SEATED: Cache<(String, String), Seated> = Cache::new();

pub(super) fn seated(family: &str, stage: &str) -> Arc<Seated> {
    SEATED.get((family.to_string(), stage.to_string()), || {
        let s = surface(family, stage);
        let pose = seated_pose(&s.p, SIT_FLEX);
        let pos: Vec<V3> = s
            .pos
            .iter()
            .zip(&s.dense)
            .map(|(q, d)| skinned(&pose, *q, d).0)
            .collect();
        let nor = smooth_normals(&pos, &tris(family));
        let tree = KdTree::new(&pos);
        let hips = ["pelvis", "thigh_l", "thigh_r", "spine_01"].map(b);
        let seat = pos
            .iter()
            .zip(&s.dense)
            .filter(|(_, d)| hips.iter().map(|&i| d[i]).sum::<f64>() > 0.6)
            .map(|(q, _)| q.y)
            .fold(f64::INFINITY, f64::min);
        Seated {
            pose,
            pos,
            nor,
            tree,
            seat,
        }
    })
}

/// `q` with weights `d` skinned by `pose`, and the blend of the bones' rotations there.
pub(super) fn skinned(pose: &[(M3, V3)], q: V3, d: &Dense) -> (V3, M3) {
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
pub(super) fn seated_tone(
    family: &str,
    stage: &str,
    pos: &[V3],
    dense: &[Dense],
    hug: &[V3],
    clear: f64,
) -> Vec<[f64; 4]> {
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
            if delta.norm() < 1e-5 {
                [0.0; 4]
            } else {
                [delta.x, delta.y, delta.z, 0.0]
            }
        })
        .collect()
}

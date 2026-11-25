//! The fields a body is shaded with, and its tone.

use super::*;

/// Body fields (n, 4) of positions on the base body (see the module docs).
pub fn fields_of(
    family: &str,
    pos: &[V3],
    arm: &[f64],
    region: &[[f64; NR]],
    p: &[V3; NB],
    head: &HeadFrame,
) -> Vec<[f64; 4]> {
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
    let ys: Vec<f64> = lp
        .iter()
        .zip(&rho_l)
        .map(|(q, r)| q.y - (r - rho0))
        .collect();
    let thetas: Vec<f64> = th_l
        .iter()
        .copied()
        .chain(th_l.iter().map(|t| -t))
        .collect();
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

/// The face's age folds (`HeadFrame::fold`) per vertex of the skin (the triangles of `idx` among `pos`),
/// as its triangles can show them: each vertex's the mean over its triangles weighted by how much of the
/// colour there it carries (its barycentric weight). A line a few millimetres wide sampled at the
/// vertices alone, which lie farther apart, came out in blotches.
fn folds(pos: &[V3], idx: &[usize], head: &HeadFrame) -> Vec<f64> {
    const N: usize = 6;
    let (mut num, mut den) = (vec![0.0; pos.len()], vec![0.0; pos.len()]);
    for t in idx.chunks(3).filter(|t| t.iter().all(|&i| i < pos.len())) {
        let [a, b, c] = [pos[t[0]], pos[t[1]], pos[t[2]]];
        let area = (b - a).cross(&(c - a)).norm();
        // The centres of the triangle's N x N equal parts.
        for i in 0..N {
            for j in 0..N - i {
                for (di, dj, ok) in [(1.0, 1.0, true), (2.0, 2.0, i + j + 2 <= N)] {
                    if !ok {
                        continue;
                    }
                    let (u, v) = (
                        (i as f64 + di / 3.0) / N as f64,
                        (j as f64 + dj / 3.0) / N as f64,
                    );
                    let w = [1.0 - u - v, u, v];
                    let f = head.fold(a * w[0] + b * w[1] + c * w[2]);
                    for (&vi, wk) in t.iter().zip(w) {
                        num[vi] += wk * f * area;
                        den[vi] += wk * area;
                    }
                }
            }
        }
    }
    num.iter()
        .zip(&den)
        .map(|(n, d)| if *d > 0.0 { n / d } else { 0.0 })
        .collect()
}

/// The nostrils' shade on the nose's underside (`body_fields_and_tone`): from where beside the nose's
/// middle it begins, where it is full and where it ends (|x|, of hh), how far above the mouth and below
/// the eyes the nose's underside is looked for (of hh), and how dark (tone.y).
const NOSTRIL: (f64, f64, f64, f64) = (0.006, 0.026, 0.044, 0.03);
const NOSTRIL_SHADE: f64 = 0.6;

/// Which of a body's three meshes: the low-detail one, the body, or the close-up body (`hd.body`).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Detail {
    Lod,
    Body,
    CloseUp,
}

/// Clothes fields and tone of the (base stage) body at `detail`, worked out at that mesh's own vertices;
/// the face's pieces and the ears after the skin.
pub fn body_fields_and_tone(
    family: &str,
    detail: Detail,
    mesh: &Mesh,
    p: &[V3; NB],
    head: &HeadFrame,
) -> (Vec<[f64; 4]>, Vec<[f64; 4]>) {
    let s = surface(family, BASE_STAGE);
    let n = mesh.pos.len();
    let lod = detail == Detail::Lod;
    let (pos, tags, region, nor) = match detail {
        Detail::Lod => (&s.lod_pos, &s.lod_tags, &s.lod_region, &s.lod_nor),
        Detail::Body => (&s.pos, &s.tags, &s.region, &s.nor),
        Detail::CloseUp => (&s.fine_pos, &s.fine_tags, &s.fine_region, &s.fine_nor),
    };
    let k = pos.len();
    let mut f = fields_of(family, pos, &tags.arm, region, p, head);
    // Blush (x) and, on the face, the age folds (z; the shader deepens them with age).
    let ri_head = region_ix("head");
    let fold = folds(pos, &mesh.idx, head);
    let mut tone: Vec<[f64; 4]> = (0..k)
        .map(|i| {
            [
                clip(tags.blush[i], 0.0, 1.0),
                0.0,
                if region[i][ri_head] > 0.5 {
                    fold[i]
                } else {
                    0.0
                },
                0.0,
            ]
        })
        .collect();
    // The nostrils (y: the shader darkens the skin by it): the nose's underside, either side of its
    // middle. Shade, not shape: the cage has one face there.
    if !lod {
        let (lo, hi) = (
            head.mouth.y + NOSTRIL.3 * head.hh,
            head.eyes[0].y - NOSTRIL.3 * head.hh,
        );
        for i in 0..k {
            let (q, nr) = (pos[i], nor[i]);
            if region[i][ri_head] < 0.5 || q.z < head.face_z || !(lo..hi).contains(&q.y) {
                continue;
            }
            let x = q.x.abs() / head.hh;
            let beside =
                smoothstep(NOSTRIL.0, NOSTRIL.1, x) * (1.0 - smoothstep(NOSTRIL.1, NOSTRIL.2, x));
            tone[i][1] = NOSTRIL_SHADE * beside * smoothstep(0.3, 0.7, -nr.y);
        }
    }
    if n > k {
        // The pieces after the skin (the lids and lips, then the ears, which are the last: the low-detail
        // body's, or the whole ones): head fields (the ears no scalp); their tone
        // is set after the bake (build.rs `build_family`), from the skin they lie on.
        let extra = &mesh.pos[k..];
        let ears = 2 * if lod {
            super::ear::LOD_VERTS
        } else {
            super::ear::VERTS
        };
        let one = |name: &str| {
            let mut v = [0.0; NR];
            v[region_ix(name)] = 1.0;
            v
        };
        let mut regions = vec![one("head"); n - k - ears];
        regions.resize(n - k, one("ear"));
        f.extend(fields_of(
            family,
            extra,
            &vec![0.0; n - k],
            &regions,
            p,
            head,
        ));
        tone.resize(n, [0.0; 4]);
    }
    (f, tone)
}

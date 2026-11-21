//! The sole under an upper, and the shoe that is both.

use super::*;

/// The sole under a foot's upper: a slab round the upper's foot (its widest, seen from above, from the
/// full upper, so a low-detail sole matches) standing out by the welt, from the floor (the arch lifted
/// off it) to the sole's top; its edge a light midsole over a dark outsole, or dark.
fn sole(
    family: &str,
    stage: &str,
    gid: &str,
    up: &Mesh,
    side: f64,
    segs: usize,
) -> (Vec<V3>, Vec<usize>, Vec<f64>) {
    let st = shoe_style(gid);
    let k = proportions(family, stage).h / 1.7;
    // (Not its lining.)
    let keep: Vec<bool> = up
        .pos
        .iter()
        .zip(&up.fields)
        .map(|(q, f)| sign(q.x) == side && f[0] >= 0.0)
        .collect();
    let pts: Vec<V3> = up
        .pos
        .iter()
        .zip(&keep)
        .filter(|(_, k)| **k)
        .map(|(q, _)| *q)
        .collect();
    let span = |f: fn(&V3) -> f64| {
        (
            pts.iter().map(f).fold(f64::INFINITY, f64::min),
            pts.iter().map(f).fold(f64::NEG_INFINITY, f64::max),
        )
    };
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
            r[j] = (1..segs)
                .flat_map(|o| [r[(j + o) % segs], r[(j + segs - o) % segs]])
                .find(|v| !v.is_nan())
                .unwrap();
        }
    }
    let u = |z: f64| clip((z - z0) / (z1 - z0), 0.0, 1.0);
    let arch = |u: f64| {
        st.arch
            * k
            * smoothstep(st.block.1 - 0.02, st.block.1 + 0.06, u)
            * (1.0 - smoothstep(0.5, 0.62, u))
    };
    // The outline, and its outward normals (seen from above).
    let edge: Vec<(f64, f64)> = (0..segs)
        .map(|j| (cx + ang[j].sin() * r[j] * hx, cz + ang[j].cos() * r[j] * hz))
        .collect();
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
    // The toe spring: the sole's whole edge lifts towards the tip (its top and its underside alike, so
    // its band keeps its thickness), which leaves the shoe's silhouette curved off the floor at the
    // toe rather than a slab lying flat. The upper's cut rises with it (`upper_layout`), so it stays
    // inside the sole however thin the sole is.
    let spring = |u: f64| st.spring_at(u) * k;
    // Rows down its edge: a welt line (dark, the step under the upper), a light midsole, then the dark
    // outsole; the midsole and the outsole meet at a seam (a row twice, one colour above it, the other
    // below); else dark all the way down.
    let (rows, mut flags) = if st.midsole > 0.0 {
        let line = 0.004 * k;
        let rows = vec![
            ring(welt, &|u| top(u) + spring(u)),
            ring(welt, &|u| top(u) + spring(u) - line),
            ring(welt, &|u| top(u) + spring(u) - line),
            ring(welt + 0.001 * k, &|u| split(u) + spring(u)),
            ring(welt + 0.001 * k, &|u| split(u) + spring(u)),
            ring(welt - 0.0015 * k, &|u| arch(u) + spring(u)),
        ];
        (
            rows,
            [
                FLAG_SOLE,
                FLAG_SOLE,
                FLAG_MIDSOLE,
                FLAG_MIDSOLE,
                FLAG_SOLE,
                FLAG_SOLE,
            ]
            .iter()
            .flat_map(|&f| vec![f; segs])
            .collect::<Vec<_>>(),
        )
    } else {
        (
            vec![
                ring(welt, &|u| top(u) + spring(u)),
                ring(welt - 0.0015 * k, &|u| arch(u) + spring(u)),
            ],
            vec![FLAG_SOLE; 2 * segs],
        )
    };
    let (mut pos, mut idx) = strip(&rows, true);
    if st.midsole > 0.0 {
        // The two rows twice (the welt's and the midsole's seam, and the midsole's and the outsole's):
        // no triangles between them, so each band keeps one flag from end to end.
        idx.drain(segs * 18..segs * 24);
        idx.drain(segs * 6..segs * 12);
    }
    let (ct, cb, last) = (pos.len(), pos.len() + 1, (rows.len() - 1) * segs);
    pos.push(v3(cx, top(0.5) + spring(0.5), cz));
    pos.push(v3(cx, arch(0.5) + spring(0.5), cz));
    for j in 0..segs {
        let j1 = (j + 1) % segs;
        idx.extend([ct, j1, j, cb, last + j, last + j1]);
    }
    flags.extend([flags[0], FLAG_SOLE]);
    let idx = closed_outward(&pos, idx);
    (pos, idx, flags)
}

/// A shoe: the upper and a sole under each foot (cut round `full`, the full-detail upper).
pub(super) fn shoe(family: &str, stage: &str, gid: &str, full: &Mesh, lod: bool) -> Mesh {
    let s = surface(family, stage);
    let lod_up = lod.then(|| upper(family, stage, gid, true));
    let up = lod_up.as_ref().unwrap_or(full);
    let (mut pos, mut flags) = (Vec::new(), Vec::new());
    let mut idx = Vec::new();
    for side in [-1.0, 1.0] {
        let (p, i, f) = sole(family, stage, gid, full, side, if lod { 10 } else { 18 });
        let n = pos.len();
        idx.extend(i.iter().map(|&v| v + n));
        pos.extend(p);
        flags.extend(f);
    }
    let dense = follow(&pos, &s);
    let nor = smooth_normals(&pos, &idx);
    let fields = flags.iter().map(|&f| [0.05, f, SHOES, 1.0]).collect();
    let n = pos.len();
    let sole = Mesh {
        pos,
        nor,
        idx,
        dense,
        fields,
        tone: vec![[0.0; 4]; n],
        ..Mesh::default()
    };
    super::merge(&[up, &sole])
}

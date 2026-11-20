//! A shoe's upper: the body triangles refined into a layout and shelled.
//!
//! Split from `characters/clothes.rs` (see `mod.rs`).

use super::*;

/// A point on a body's triangles: up to three of its vertices (a triangle's, an edge's or one) and
/// weights summing to 1.
#[derive(Clone, Copy)]
pub(crate) struct Bary {
    pub(crate) v: [usize; 3],
    pub(crate) w: [f64; 3],
}

impl Bary {
    fn at(v: usize) -> Self {
        Bary {
            v: [v; 3],
            w: [1.0, 0.0, 0.0],
        }
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
    fn mix<T: Copy + std::ops::Mul<f64, Output = T> + std::ops::Add<Output = T>>(
        &self,
        arr: &[T],
    ) -> T {
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
pub(crate) struct UpperLayout {
    pub(crate) pts: Vec<Bary>,
    /// Per point: the finer surface's vertices at its triangle's edges' middles (`finer`), so it lies on
    /// the curved skin, not on the body's flat triangles (whose sag made the collar jagged).
    pub(crate) mid: Vec<[usize; 3]>,
    pub(crate) idx: Vec<usize>,
    /// Per point: the lift along the normal and the toe's offset (m on a 1.7 m body), the distance from
    /// the collar (m, the hem distance), across the instep (m; 1 elsewhere), and whether it lies on the
    /// cut inside the sole (no lining there).
    pub(crate) lift: Vec<f64>,
    pub(crate) point: Vec<V3>,
    pub(crate) hem: Vec<f64>,
    pub(crate) across: Vec<f64>,
    pub(crate) low: Vec<bool>,
}

pub(crate) static UPPERS: Cache<(String, String, bool), UpperLayout> = Cache::new();

/// Triangles split at the middles of the edges `split` says (in 2, 3 or 4, so no T-junctions).
fn refine(pts: &mut Vec<Bary>, idx: &[usize], split: impl Fn(usize, usize) -> bool) -> Vec<usize> {
    let key = |a: usize, b: usize| (a.min(b), a.max(b));
    let mut cut: std::collections::BTreeMap<(usize, usize), usize> =
        std::collections::BTreeMap::new();
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
        let m: Vec<Option<usize>> = [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])]
            .iter()
            .map(|&(a, b)| cut.get(&key(a, b)).copied())
            .collect();
        // Turned so the split edges come first (edge k joins corner k and k + 1; winding kept).
        let r = match m.iter().filter(|e| e.is_some()).count() {
            0 => {
                out.extend_from_slice(t);
                continue;
            }
            1 => m.iter().position(|e| e.is_some()).unwrap(),
            2 => m
                .iter()
                .position(|e| e.is_none())
                .map(|k| (k + 1) % 3)
                .unwrap(),
            _ => 0,
        };
        let (a, b, c) = (t[r], t[(r + 1) % 3], t[(r + 2) % 3]);
        let (ab, bc, ca) = (m[r], m[(r + 1) % 3], m[(r + 2) % 3]);
        match (ab, bc, ca) {
            (Some(ab), None, None) => out.extend([a, ab, c, ab, b, c]),
            (Some(ab), Some(bc), None) => out.extend([ab, b, bc, a, ab, bc, a, bc, c]),
            (Some(ab), Some(bc), Some(ca)) => {
                out.extend([a, ab, ca, ab, b, bc, ca, bc, c, ab, bc, ca])
            }
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
        let (pos, nor): (&[V3], &[V3]) = if lod {
            (&s.lod_pos, &s.lod_nor)
        } else {
            (&s.pos, &s.nor)
        };
        let (ff, reg): (&[[f64; 4]], &[[f64; NR]]) = if lod {
            (&fl.1, &s.lod_region)
        } else {
            (&fl.0, &s.region)
        };
        let idx = if lod {
            topology().lod_tris.concat()
        } else {
            tris(family).to_vec()
        };
        let topo = topology();
        let mids = if lod {
            edge_mids(&topo.faces, &topo.s.edges, topo.n)
        } else {
            edge_mids(
                &topo.s.quads.iter().map(|q| q.to_vec()).collect::<Vec<_>>(),
                &topo.s2.edges,
                topo.s.m.rows,
            )
        };
        let mid_of = |p: &Bary| {
            [(0, 1), (0, 2), (1, 2)].map(|(i, j)| {
                if p.w[i] != 0.0 && p.w[j] != 0.0 {
                    mids[&(p.v[i].min(p.v[j]), p.v[i].max(p.v[j]))]
                } else {
                    usize::MAX
                }
            })
        };
        let fine = finer(family, BASE_STAGE, lod);
        let height: Vec<f64> = ff.iter().map(|f| f[1]).collect();
        let rf = region("foot");
        let pointed = if st.point > 0.0 { 1.0 } else { 0.0 };
        // Per side: the ankle joint, and the foot's length (z at the heel and at the toe).
        let feet = ["l", "r"].map(|side| {
            let sx = if side == "l" { -1.0 } else { 1.0 };
            let zs: Vec<f64> = (0..pos.len())
                .filter(|&i| reg[i][rf] > 0.5 && sign(pos[i].x) == sx)
                .map(|i| pos[i].z)
                .collect();
            (
                sx,
                s.p[b(&format!("foot_{side}"))],
                zs.iter().copied().fold(f64::INFINITY, f64::min),
                zs.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            )
        });
        // A point's lift, toe offset, distance below the collar, across the instep and clip field.
        let attrs = |p: &Bary| -> (f64, V3, f64, f64, f64) {
            let q = p.curved(pos, &fine, &mid_of(p));
            let Some(&(_, ja, z0, z1)) = feet.iter().find(|f| f.0 == sign(q.x)) else {
                return (0.0, V3::zeros(), 0.0, 1.0, -1.0);
            };
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
            let across = if st.laces {
                dx + if dz > 0.5 * dx.hypot(dz) { 0.0 } else { 1.0 }
            } else {
                1.0
            };
            // Under the collar, above the sole's inside (lifted as at the base, and with the toe spring,
            // which can be taller than a thin sole: cut where the flat sole's top was, the upper would
            // come out under a sprung toe), off the sole (the foot's flat underside).
            let y = q.y + n.y * lift * kb;
            let g = below
                .min(y - (st.top(u) + st.spring_at(u) - UPPER_IN_SOLE) * kb)
                .min(n.y + 0.9);
            (lift, point, below, across, g)
        };
        let mut pts: Vec<Bary> = (0..pos.len()).map(Bary::at).collect();
        let mut tri = idx;
        if !lod {
            let a: Vec<(f64, f64)> = pts.iter().map(&attrs).map(|a| (a.2, a.4)).collect();
            // The edges the collar crosses (on the shoe: the clip field over 0 at an end).
            tri = refine(&mut pts, &tri, |i, j| {
                (a[i].1 > -0.01 || a[j].1 > -0.01) && (a[i].0 > 0.0) != (a[j].0 > 0.0)
            });
        }
        let all: Vec<(f64, V3, f64, f64, f64)> = pts.iter().map(attrs).collect();
        let g: Vec<f64> = all.iter().map(|a| a.4).collect();
        let (cidx, src) = clip_by_field(&tri, &g);
        let pts: Vec<Bary> = (0..src.0.len())
            .map(|i| Bary::lerp(&pts[src.0[i]], &pts[src.1[i]], src.2[i]))
            .collect();
        let lift = lerp_src(&src, &all.iter().map(|a| a.0).collect::<Vec<_>>());
        let point = lerp_src(&src, &all.iter().map(|a| a.1).collect::<Vec<_>>());
        let hem = lerp_src(&src, &all.iter().map(|a| a.2).collect::<Vec<_>>());
        let across = lerp_src(&src, &all.iter().map(|a| a.3).collect::<Vec<_>>());
        let mid: Vec<[usize; 3]> = pts.iter().map(mid_of).collect();
        // The hem distance: how far from the collar's edge (the nearest point of it), so it runs down
        // the instep as far as along the sides. Under the laces, how far down them from the collar's
        // front (straight down the middle of the instep: linear, so the laces' bars stay straight
        // across the big triangles there).
        let at: Vec<V3> = pts
            .iter()
            .zip(&mid)
            .map(|(p, m)| p.curved(pos, &fine, m))
            .collect();
        let edge: Vec<V3> = (0..at.len())
            .filter(|&i| src.0[i] != src.1[i] && hem[i].abs() < 1e-4)
            .map(|i| at[i])
            .collect();
        let edge = KdTree::new(&edge);
        let mut hem: Vec<f64> = at
            .iter()
            .zip(&hem)
            .map(|(q, &h)| {
                if h <= 0.0 {
                    0.0
                } else {
                    edge.query(q, 1).first().map_or(f64::INFINITY, |e| e.0)
                }
            })
            .collect();
        for sx in [-1.0, 1.0] {
            let laced: Vec<usize> = (0..at.len())
                .filter(|&i| sign(at[i].x) == sx && across[i] < 1.0)
                .collect();
            let middle = |i: &&usize| across[**i].abs() < 0.012;
            let Some(&c) = laced
                .iter()
                .filter(middle)
                .filter(|&&i| hem[i] == 0.0)
                .min_by(|&&i, &&j| across[i].abs().total_cmp(&across[j].abs()))
            else {
                continue;
            };
            let far = *laced
                .iter()
                .filter(middle)
                .min_by(|&&i, &&j| {
                    ((at[i] - at[c]).norm() - 0.06)
                        .abs()
                        .total_cmp(&((at[j] - at[c]).norm() - 0.06).abs())
                })
                .unwrap();
            let down = (at[far] - at[c]).normalize();
            for &i in &laced {
                hem[i] = (at[i] - at[c]).dot(&down).max(0.0);
            }
        }
        // The cut inside the sole: new vertices on an edge, not at the collar.
        let low = (0..src.0.len())
            .map(|i| src.0[i] != src.1[i] && hem[i] > 0.01)
            .collect();
        UpperLayout {
            pts,
            mid,
            idx: cidx,
            lift,
            point,
            hem,
            across,
            low,
        }
    })
}

/// A shoe's upper at `stage`: the layout's skin lifted off it (with the body's own weights, so it
/// moves as the foot does), and a lining turned in from the collar to the skin.
pub(crate) fn upper(family: &str, stage: &str, gid: &str, lod: bool) -> Mesh {
    let lay = upper_layout(family, gid, lod);
    let st = shoe_style(gid);
    let s = surface(family, stage);
    let k = proportions(family, stage).h / 1.7;
    // (The body's normals: from its finer limit surface, so the shoe shades as smoothly as the skin.)
    let (bpos, bnor, dense): (&[V3], &[V3], &[Dense]) = if lod {
        (&s.lod_pos, &s.lod_nor, &s.lod_dense)
    } else {
        (&s.pos, &s.nor, &s.dense)
    };
    let fine = finer(family, stage, lod);
    let nor: Vec<V3> = lay.pts.iter().map(|p| p.mix(bnor).normalize()).collect();
    let pos: Vec<V3> = (0..lay.pts.len())
        .map(|i| {
            lay.pts[i].curved(bpos, &fine, &lay.mid[i])
                + nor[i] * lay.lift[i] * k
                + lay.point[i] * k
        })
        .collect();
    let mut d: Vec<Dense> = lay
        .pts
        .iter()
        .map(|p| std::array::from_fn(|k| (0..3).map(|j| dense[p.v[j]][k] * p.w[j]).sum()))
        .collect();
    let n0 = pos.len();
    let flag = if st.polish { FLAG_POLISHED } else { 0.0 };
    let fields = (0..n0).map(|i| [lay.hem[i], flag, SHOES, lay.across[i]]);
    let mut out = Mesh {
        pos,
        nor,
        idx: lay.idx.clone(),
        fields: fields.collect(),
        ..Mesh::default()
    };
    if !lod {
        // The lining: a band from the collar's edge in to the skin (its hem distance negative: drawn dark).
        let depth = 0.85 * (st.lift + st.pad + st.shaft) * k;
        let be: Vec<[usize; 2]> = boundary_edges(&out.idx)
            .into_iter()
            .filter(|e| !(lay.low[e[0]] && lay.low[e[1]]))
            .collect();
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
        out.nor
            .extend_from_slice(&smooth_normals(&out.pos, &out.idx)[n0..]);
    }
    let n = out.pos.len();
    Mesh {
        dense: d,
        tone: vec![[0.0; 4]; n],
        ..out
    }
}

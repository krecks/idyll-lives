//! The cap: the surface the hair grows from, its strips and its grid.

use super::*;

/// A component of a style: faces, how far each vertex follows the body rather than the head
/// (`f`, -1: by height, see `finish`), clump bands (`z`) and shade (`y`); the wisps past its
/// edges; and how loose it hangs (`sway`: 0 on the head, as most are; see `finish`).
#[derive(Clone, Default)]
pub struct Comp {
    pub(crate) pos: Vec<V3>,
    pub(crate) idx: Vec<usize>,
    pub(crate) f: Vec<f64>,
    pub(crate) z: Vec<f64>,
    pub(crate) y: Vec<f64>,
    pub(crate) wisps: Vec<Strip>,
    pub(crate) sway: f64,
}

/// Hair that hangs moves after the head (the runtime's lag vector times tone.x, `sway.ts`): a
/// component's looseness is its `sway` times how far down it a vertex is. Along a tail or a braid
/// (its `f`, 0 at the head) as the power `TAIL`; on a curtain by the height below `FROM` head
/// heights over the chin (about the ears: above, it lies on the head), all of it `OVER` head
/// heights further down (the shoulder blades), as the power `FALL` between.
pub(crate) const SWAY_TAIL: f64 = 1.3;
pub(crate) const SWAY_FROM: f64 = 0.4;
pub(crate) const SWAY_OVER: f64 = 1.2;
pub(crate) const SWAY_FALL: f64 = 1.5;

/// A soft edge: a row of quads growing from an edge of a component (or, where it has none, from a
/// row of its surface: `shingles`), each from two of its vertices next to each other out to where
/// their wisps end (`wisps`). The vertices are the solid hair's own, so the strip starts exactly on
/// it, and takes their shade, normals and skin.
#[derive(Clone)]
pub(crate) struct Strip {
    /// The edge's vertices (of the component), in order.
    pub(crate) at: Vec<usize>,
    /// Where each one's wisp ends, from the vertex (m).
    pub(crate) by: Vec<V3>,
    /// The edge is a ring: the last vertex's quad goes on to the first.
    pub(crate) closed: bool,
    /// How loose the strands are (fields.z): 0 combed, lying side by side; 1 a curl's, crossing and
    /// broken along their length.
    pub(crate) loose: f64,
    /// The share of each quad's length, from its root, over which the strands are still one (fields.w):
    /// solid hair, as over a hem's lip, which the wisps hang down in front of.
    pub(crate) rooted: Vec<f64>,
}

/// Maximal runs of `show` round a ring of columns as strips (`by(k)`: column k's wisp); the whole
/// ring, a closed one.
fn rim_strips(show: &[bool], by: &dyn Fn(usize) -> V3) -> Vec<Strip> {
    let n = show.len();
    let strip = |at: Vec<usize>, closed: bool| Strip {
        by: at.iter().map(|&k| by(k)).collect(),
        rooted: vec![0.0; at.len()],
        at,
        closed,
        loose: 0.0,
    };
    let Some(start) = show.iter().position(|&s| !s) else {
        return vec![strip((0..n).collect(), true)];
    };
    let (mut out, mut run) = (Vec::new(), Vec::new());
    for o in 1..=n {
        let k = (start + o) % n;
        if show[k] {
            run.push(k);
        } else if run.len() > 1 {
            out.push(strip(std::mem::take(&mut run), false));
        } else {
            run.clear();
        }
    }
    out
}

/// A lock's tip in wisps: a brush from one of a loft's rings (`segs` points each; the ring `back`
/// rings before its last) on past its end by `past` (m), drawn in to `taper` of that ring's breadth.
/// `span`: over the lock's outer face only, that many points either side of the ring's first (a flat
/// lock lying on the head has no other side to show); without, all round the ring, a sleeve (a
/// tail's end).
pub(crate) fn brush(
    pos: &[V3],
    segs: usize,
    back: usize,
    past: f64,
    taper: f64,
    span: Option<usize>,
) -> Strip {
    let rings = pos.len() / segs;
    let ring = |r: usize| &pos[r * segs..(r + 1) * segs];
    let (root, end) = (rings - 1 - back, mean_rows(ring(rings - 1)));
    let c = mean_rows(ring(root));
    let axis = (end - c) / (end - c).norm().max(1e-9);
    let of: Vec<usize> = match span {
        Some(n) => (0..=2 * n).map(|q| (q + segs - n) % segs).collect(),
        None => (0..segs).collect(),
    };
    Strip {
        by: of
            .iter()
            .map(|&q| end + axis * past + (ring(root)[q] - c) * taper - ring(root)[q])
            .collect(),
        rooted: vec![0.0; of.len()],
        at: of.iter().map(|q| root * segs + q).collect(),
        closed: span.is_none(),
        loose: 0.0,
    }
}

/// A component with faces turned to point along `out` (if given).
pub(crate) fn comp(
    pos: Vec<V3>,
    idx: Vec<usize>,
    out: Option<Vec<V3>>,
    z: Option<Vec<f64>>,
    y: Option<Vec<f64>>,
    follow: Option<Vec<f64>>,
) -> Comp {
    let n = pos.len();
    let idx = match out {
        Some(o) => orient(&pos, idx, &o),
        None => idx,
    };
    Comp {
        f: follow.unwrap_or_else(|| vec![-1.0; n]),
        z: z.unwrap_or_else(|| vec![0.5; n]),
        y: y.unwrap_or_else(|| vec![0.0; n]),
        pos,
        idx,
        wisps: Vec::new(),
        sway: 0.0,
    }
}

type Field<'a> = &'a dyn Fn(f64, f64) -> f64;

/// How much denser than its layout the hair is built (`dense_*`, `LOCK`): the same style, more
/// columns and rows, so its locks round off instead of reading as plates and its hem is a curve
/// rather than a polyline. The styles' numbers are the shapes (a cap's columns, 4 to a lock; a
/// curtain's or a fringe's locks; a parting's width in the layout's columns); this is only the
/// resolution they are cut at: the locks keep their count and width, each on more columns.
const DETAIL: f64 = 1.5;
/// Columns a lock is wide in a style's layout.
pub(crate) const LAYOUT_LOCK: usize = 4;
/// Columns a lock is wide as built: `LAYOUT_LOCK` at `DETAIL` (even, so a lock has a middle column).
pub(crate) const LOCK: usize = 6;
const _: () = assert!(
    LOCK as f64 == LAYOUT_LOCK as f64 * DETAIL,
    "LOCK is LAYOUT_LOCK at DETAIL"
);
/// A cap's `n` layout columns (a multiple of `LAYOUT_LOCK`) at `DETAIL`: the same locks, `LOCK`
/// columns each.
pub(crate) fn dense_cols(n: usize) -> usize {
    (n / LAYOUT_LOCK).max(1) * LOCK
}
/// `n` rows at `DETAIL`.
pub(crate) fn dense_rows(n: usize) -> usize {
    ((n as f64 * DETAIL).round() as usize).max(2)
}
/// How far into a lock column j of a sheet or cap is, its locks `w` columns wide: 0 on the parting
/// between two locks (columns w k), 1 in a lock's middle (w k + w/2).
fn lock_at(j: f64, w: usize) -> f64 {
    (PI * j / w as f64).sin().abs().powf(0.6)
}

/// `lock_at` across `cols` columns starting half a lock in (whole locks at the edges).
pub(crate) fn locks(cols: usize, w: usize) -> Vec<f64> {
    (0..cols).map(|j| lock_at((j + w / 2) as f64, w)).collect()
}

/// A fixed 0..1 for lock k of a component (seed), so lengths and tips vary from lock to lock.
pub(crate) fn jitter(k: usize, seed: f64) -> f64 {
    let v = ((k as f64 + 1.0) * 12.9898 + seed * 78.233).sin() * 43758.5453;
    v - v.floor()
}

/// The skull cap's shape (see `cap`).
pub(crate) struct Cap<'a> {
    /// Thickness (m) by azimuth and u (0 at the crown .. 1 at the hairline).
    pub(crate) thick: Field<'a>,
    pub(crate) rows: usize,
    /// Columns round the head in the layout, a multiple of `LAYOUT_LOCK` (a lock is that many wide;
    /// built at `DETAIL`, `dense_cols`).
    pub(crate) cols: usize,
    /// The hairline's polar angle by azimuth (else the scalp's).
    pub(crate) limit: Option<&'a dyn Fn(f64) -> f64>,
    /// Thickness at the rim (m).
    pub(crate) edge: f64,
    /// Locks combed down from the crown: the grooves between them this deep (a share of the
    /// thickness), and their tips this far past the hairline (rad of polar angle, by azimuth).
    pub(crate) ridge: f64,
    pub(crate) tips: Option<&'a dyn Fn(f64) -> f64>,
    /// A parting: its azimuth (the columns turn so a groove lies on it) and the u it starts at.
    pub(crate) part: Option<(f64, f64)>,
    /// Clump bands by azimuth and u (else the locks': light in their middles).
    pub(crate) bands: Option<Field<'a>>,
    /// How it meets the ears.
    pub(crate) ears: Meet,
    /// Wisps past the rim, on over the skin: their length (m) by azimuth (0: none there, where other
    /// hair covers the rim); none round an ear. Only on a cap whose rim goes round the ears
    /// (`Meet::Round`).
    pub(crate) wisps: Option<&'a dyn Fn(f64) -> f64>,
}

/// How a cap meets the ears: `Round(n)`, its rim goes round each in an arc, with n more columns there;
/// `Over`, it stands off them (the rim at the hairline) and closes under itself above them.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Meet {
    Round(usize),
    Over,
}

/// The azimuths (from the front, rad) between which a cap's columns may move to go round an ear.
const EAR_SPAN: [f64; 2] = [57.0 * PI / 180.0, 153.0 * PI / 180.0];

/// How far round from the front azimuth a is (rad, 0..pi).
pub(crate) fn round(a: f64) -> f64 {
    (py_mod(a + PI, TAU) - PI).abs()
}

/// The point `len` (m) from `p` over the head's skin, setting out along `dir`: walked in steps, each
/// along the skin and back to `p`'s distance off it (clear of it at the least; the head's field
/// knows the neck too, which a direction from the head's centre does not reach).
fn over_skin(h: &HeadFrame, p: V3, dir: V3, len: f64) -> V3 {
    let off = h.sdf(p).max(CLEAR + 0.0008);
    let steps = 4;
    let (mut q, mut d) = (p, dir);
    for _ in 0..steps {
        let g = grad(h, q, 1e-3);
        d -= g * d.dot(&g);
        d /= d.norm().max(1e-9);
        q += d * (len / steps as f64);
        q += grad(h, q, 1e-3) * (off - h.sdf(q));
    }
    q
}

/// The skull cap: rows from the crown down to the hairline, pushed out by the thickness, tapering to
/// `edge` at the rim; in locks, with a parting. Round an ear (`Meet::Round`) the rim follows an arc
/// over it (`Oval::rim`) on more columns, and the rows above bend round it too; over them
/// (`Meet::Over`) the cap stands off the ear and closes under itself in a hem above it.
pub(crate) fn cap(sc: &Scalp, mut c: Cap) -> Comp {
    let h = sc.h;
    // A layout column (the parting's width is in them), then the columns as built.
    let layout_step = TAU / c.cols as f64;
    (c.cols, c.rows) = (dense_cols(c.cols), dense_rows(c.rows));
    let step = TAU / c.cols as f64;
    let lock = LOCK as f64 * step;
    let turn = c
        .part
        .map_or(0.0, |(ap, _)| ap - (((ap + PI) / lock).round() * lock - PI));
    let base: Vec<f64> = linspace_open(-PI, PI, c.cols)
        .iter()
        .map(|&a| a + turn)
        .collect();
    let hl = |a: f64| match c.limit {
        Some(l) => l(a),
        None => sc.hairline(a),
    };
    // The columns: where each is in the locks (column j of the even layout, fractional between the
    // ones added round an ear), its azimuth up on the head, its rim (azimuth, polar angle) and whether
    // it goes round an ear.
    let (mut ell, mut a, mut rim, mut moved) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let extra = match c.ears {
        Meet::Round(n) => n,
        Meet::Over => 0,
    };
    let span = |a: f64| (EAR_SPAN[0]..=EAR_SPAN[1]).contains(&round(a));
    let mut j = 0;
    while j < c.cols {
        if extra == 0 || !span(base[j]) {
            ell.push(j as f64);
            a.push(base[j]);
            rim.push((base[j], hl(base[j])));
            moved.push(false);
            j += 1;
            continue;
        }
        let j0 = j;
        while j < c.cols && span(base[j]) {
            j += 1;
        }
        assert!(
            j0 > 0 && j < c.cols,
            "a cap's columns round an ear wrap past the back"
        );
        // Between the columns either side of the ear, the edge round it evenly.
        let n = j - j0 + extra;
        let (a0, a1) = (base[j0 - 1], base[j]);
        let pts = match sc.oval((a0 + a1) / 2.0) {
            Some(o) => o.rim(&hl, a0, a1, n),
            None => (1..=n)
                .map(|k| a0 + (a1 - a0) * k as f64 / (n + 1) as f64)
                .map(|x| (x, hl(x)))
                .collect(),
        };
        for (k, q) in pts.into_iter().enumerate() {
            let f = (k + 1) as f64 / (n + 1) as f64;
            ell.push((j0 - 1) as f64 + f * (j - j0 + 1) as f64);
            a.push(a0 + f * (a1 - a0));
            rim.push(q);
            moved.push(true);
        }
    }
    let cols = ell.len();
    let lk: Vec<f64> = ell.iter().map(|&l| lock_at(l, LOCK)).collect();
    // Tips past the hairline, none where it goes round an ear.
    let lim: Vec<f64> = (0..cols)
        .map(|k| {
            let (ra, rp) = rim[k];
            let near = match sc.oval(ra) {
                Some(o) if extra > 0 => smoothstep(1.25, 1.7, o.rho(ra, rp)),
                _ => 1.0,
            };
            rp + c.tips.map_or(0.0, |t| t(ra) * lk[k].powf(1.5)) * near
        })
        .collect();
    // Each row's (azimuth, polar angle) by column: the columns round an ear turn onto their rims'
    // azimuths down from the crown, the rows there eased (each point the mean of its neighbours) so
    // they bend round the ear evenly.
    let us = linspace(1.0, 1.0 / c.rows as f64, c.rows);
    let ease = 0.45;
    let mut node: Vec<Vec<(f64, f64)>> = us
        .iter()
        .map(|&u| {
            (0..cols)
                .map(|k| {
                    (
                        a[k] + (rim[k].0 - a[k]) * smoothstep(ease, 1.0, u),
                        u * lim[k],
                    )
                })
                .collect()
        })
        .collect();
    for _ in 0..60 {
        let was = node.clone();
        for i in 1..c.rows - 1 {
            if us[i] <= ease {
                break;
            }
            for k in (0..cols).filter(|&k| moved[k]) {
                let (l, r) = (was[i][k - 1], was[i][k + 1]);
                let (d, t) = (was[i - 1][k], was[i + 1][k]);
                node[i][k] = ((l.0 + r.0 + d.0 + t.0) / 4.0, (l.1 + r.1 + d.1 + t.1) / 4.0);
            }
        }
    }
    // How near a point is to the parting (1 on it), from where it starts down to the hairline.
    let parting = |a: f64, u: f64| {
        c.part.map_or(0.0, |(ap, u0)| {
            let d = py_mod(a - ap + PI, TAU) - PI;
            (-(d / (0.6 * layout_step)).powi(2)).exp() * smoothstep(u0, u0 + 0.35, u)
        })
    };
    // Over the ears: at least as far out as they reach, and a little more over them.
    let over = |a: f64, phi: f64, more: f64| match sc.oval(a) {
        Some(o) if c.ears == Meet::Over => {
            (o.reach + EAR_CLEAR + CLEAR + more) * smoothstep(1.8, 1.0, o.rho(a, phi))
        }
        _ => 0.0,
    };
    let mut p = Vec::new();
    let (mut z, mut y) = (Vec::new(), Vec::new());
    let mut rim_t = Vec::new();
    for (i, &u) in us.iter().enumerate() {
        let mut ring = Vec::new();
        for k in 0..cols {
            let (ak, phi) = node[i][k];
            // The locks part from the crown in a whorl, the grooves between them deepening down to the hairline.
            // None over the ears (the cap is thinnest there).
            let ear = (-((round(ak) - PI / 2.0) / 0.5).powi(2)).exp() * smoothstep(0.5, 0.8, u);
            let groove = c.ridge * (1.0 - lk[k]) * smoothstep(0.25, 0.8, u) * (1.0 - ear);
            let pt = parting(ak, u);
            let t = (c.thick)(ak, u) * (1.0 - groove) * (1.0 - 0.8 * pt);
            let t = (t + (c.edge - t) * smoothstep(0.82, 1.0, u))
                .max(c.edge)
                .max(over(ak, phi, 0.003));
            if i == 0 {
                rim_t.push(t);
            }
            // A cut's edge comes down to the skin (the rows above are rounded out over it).
            let (q, n) = sc.rounded(
                ak + WHORL * (1.0 - u).powi(2),
                phi,
                if c.ears == Meet::Over {
                    ROUND
                } else {
                    ROUND * smoothstep(1.0, 0.75, u)
                },
            );
            ring.push(q + n * t);
            z.push(match c.bands {
                Some(bf) => bf(ak, u),
                None => {
                    0.5 + (0.6 * lk[k] * (0.7 + 0.3 * jitter((ell[k] / LOCK as f64) as usize, 3.0))
                        - 0.3
                        + 0.1 * (u * 9.0 + ell[k]).sin())
                        * smoothstep(0.1, 0.6, u)
                }
            });
            y.push(0.25 * smoothstep(0.85, 1.0, u) + 0.5 * groove + 0.6 * pt);
        }
        p.push(ring);
    }
    // Wisps past the rim: on over the skin from each of its points, the way its column comes down to
    // it, shorter towards an ear.
    let strips = match c.wisps {
        Some(wl) if c.ears != Meet::Over => {
            let show: Vec<bool> = a.iter().map(|&ak| wl(ak) > 0.0).collect();
            rim_strips(&show, &|k| {
                let (ra, rp) = node[0][k];
                let near = sc
                    .oval(ra)
                    .map_or(1.0, |o| smoothstep(1.2, 1.7, o.rho(ra, rp)));
                over_skin(h, p[0][k], p[0][k] - p[1][k], wl(a[k]) * near) - p[0][k]
            })
        }
        _ => Vec::new(),
    };
    if c.ears == Meet::Over {
        // Under the rim, up and in to the skin (so no scalp shows under it); over an ear, up and out of its
        // oval (out past the ear on the way).
        let d = 0.03;
        let (mut hem, mut hz) = (vec![Vec::new(); 3], Vec::new());
        for k in 0..cols {
            let (ak, pr) = node[0][k];
            let tr = rim_t[k];
            let (qa, qp) = sc.oval(ak).map_or((ak, pr), |o| o.up(ak, pr));
            let need = over(ak, pr, 0.0);
            let (skin, n) = sc.rounded(qa, qp - 1.6 * d, 0.0);
            hem[2].push(sc.point(ak, pr - 0.3 * d, (0.8 * tr).max(need).max(CLEAR)));
            hem[1].push(sc.point(qa, qp - d, (0.45 * tr).max(need).max(CLEAR)));
            hem[0].push(skin + n * CLEAR);
            hz.push(z[k]);
        }
        p.splice(0..0, hem);
        z.splice(
            0..0,
            hz.iter().chain(&hz).chain(&hz).copied().collect::<Vec<_>>(),
        );
        y.splice(
            0..0,
            [1.0, 0.9, 0.5]
                .iter()
                .flat_map(|&v| std::iter::repeat_n(v, cols))
                .collect::<Vec<_>>(),
        );
    }
    let top = sc.point(0.0, 0.0, (c.thick)(0.0, 0.0));
    let (pos, idx, _) = rings_mesh(&p, None, Some(top));
    z.push(0.5);
    y.push(0.0);
    let out = pos.iter().map(|q| q - h.centre).collect();
    Comp {
        wisps: strips,
        ..comp(pos, idx, Some(out), Some(z), Some(y), None)
    }
}

/// Unit normals of a grid of points (rows, cols), turned to point along `out(p)`.
pub(crate) fn grid_normals(p: &[Vec<V3>], out: &dyn Fn(V3) -> V3) -> Vec<Vec<V3>> {
    let (r, c) = (p.len(), p[0].len());
    (0..r)
        .map(|i| {
            (0..c)
                .map(|j| {
                    let down = p[(i + 1).min(r - 1)][j] - p[i.saturating_sub(1)][j];
                    let across = p[i][(j + 1).min(c - 1)] - p[i][j.saturating_sub(1)];
                    let n = across.cross(&down);
                    let n = n / n.norm().max(1e-12);
                    if n.dot(&out(p[i][j])) < 0.0 { -n } else { n }
                })
                .collect()
        })
        .collect()
}

/// A row's section round a sheet with a thickness: the outer face (`c + n h k`) across, a rounded
/// edge at the last column (`s`: across the sheet), the inner face back, a rounded edge at the
/// first; k = 0 closes it to the middle. Each point with its direction out and its shade.
fn section(c: &[V3], n: &[V3], s: &[V3], h: &[f64], k: f64) -> Vec<(V3, V3, f64)> {
    let m = c.len();
    let mut out = Vec::with_capacity(2 * m + 4);
    out.extend((0..m).map(|j| (c[j] + n[j] * h[j] * k, n[j], 0.0)));
    for th in [PI / 3.0, 2.0 * PI / 3.0] {
        let d = n[m - 1] * th.cos() + s[m - 1] * (1.1 * th.sin());
        out.push((c[m - 1] + d * h[m - 1] * k, d, 0.3));
    }
    out.extend((0..m).rev().map(|j| (c[j] - n[j] * h[j] * k, -n[j], 0.55)));
    for th in [2.0 * PI / 3.0, PI / 3.0] {
        let d = n[0] * th.cos() - s[0] * (1.1 * th.sin());
        out.push((c[0] + d * h[0] * k, d, 0.3));
    }
    out
}

/// How far under a slab's last row its hem closes, of its half thickness there.
const HEM_DROP: f64 = 0.9;

/// A sheet of hair with a thickness, from its middle surface (rows from the top to the hem, columns
/// across), the normals out from the head and the half thickness: every row a closed section with
/// rounded edges, and under the last a rounded hem, curled in by `curl` (a share of the thickness).
/// Clump bands per column. Wisps hang `wisp` (m) below the hem: from the outer face's last row, down
/// in front of the hem's lip (as one to its lowest line) and on, turned in half as far as it is.
pub(crate) fn slab(
    mid: &[Vec<V3>],
    nrm: &[Vec<V3>],
    half: &[Vec<f64>],
    curl: f64,
    bands: &[f64],
    wisp: f64,
) -> Comp {
    let (rows, cols) = (mid.len(), mid[0].len());
    let unit = |v: V3| v / v.norm().max(1e-12);
    let across = |q: &[V3]| -> Vec<V3> {
        (0..cols)
            .map(|j| unit(q[(j + 1).min(cols - 1)] - q[j.saturating_sub(1)]))
            .collect()
    };
    let mut rings = Vec::new();
    let (mut outs, mut y) = (Vec::new(), Vec::new());
    let mut add = |sec: Vec<(V3, V3, f64)>, shade: Option<f64>| {
        rings.push(sec.iter().map(|e| e.0).collect::<Vec<_>>());
        outs.extend(sec.iter().map(|e| e.1));
        y.extend(sec.iter().map(|e| shade.unwrap_or(e.2)));
    };
    for i in 0..rows {
        add(
            section(&mid[i], &nrm[i], &across(&mid[i]), &half[i], 1.0),
            None,
        );
    }
    let l = rows - 1;
    let s = across(&mid[l]);
    for (kd, kn, shade) in [(0.55, 0.75, None), (HEM_DROP, 0.0, Some(0.2))] {
        let c: Vec<V3> = (0..cols)
            .map(|j| {
                mid[l][j] + (unit(mid[l][j] - mid[l - 1][j]) - nrm[l][j] * curl) * (half[l][j] * kd)
            })
            .collect();
        add(section(&c, &nrm[l], &s, &half[l], kn), shade);
    }
    let (pos, idx, _) = rings_mesh(&rings, None, None);
    // The column each point of a section belongs to.
    let k = 2 * cols + 4;
    let col = |v: usize| match v % k {
        q if q < cols => q,
        q if q < cols + 2 => cols - 1,
        q if q < 2 * cols + 2 => 2 * cols + 1 - q,
        _ => 0,
    };
    let z = (0..pos.len()).map(|v| bands[col(v)]).collect();
    // (The hem's lowest line is `HEM_DROP` of the half thickness under the last row.)
    let hem = Strip {
        at: (l * k..l * k + cols).collect(),
        by: (0..cols)
            .map(|j| {
                unit(unit(mid[l][j] - mid[l - 1][j]) - nrm[l][j] * (0.5 * curl))
                    * (HEM_DROP * half[l][j] + wisp)
            })
            .collect(),
        closed: false,
        loose: 0.0,
        rooted: (0..cols)
            .map(|j| HEM_DROP * half[l][j] / (HEM_DROP * half[l][j] + wisp))
            .collect(),
    };
    Comp {
        wisps: if wisp > 0.0 { vec![hem] } else { Vec::new() },
        ..comp(pos, idx, Some(outs), Some(z), Some(y), None)
    }
}

/// Clump bands across a sheet's columns (whole locks at its edges): light in the locks' middles,
/// varying from lock to lock.
pub(crate) fn lock_bands(cols: usize, seed: f64) -> Vec<f64> {
    locks(cols, LOCK)
        .iter()
        .enumerate()
        .map(|(j, l)| {
            clip(
                0.15 + 0.7 * l * (0.75 + 0.25 * jitter((j + LOCK / 2) / LOCK, seed)),
                0.0,
                1.0,
            )
        })
        .collect()
}

/// How far out hair hangs at azimuth a, by height: the scalp's section there (a share of the skull
/// ellipsoid's radii), and below its widest straight down from it.
pub(crate) fn hang(sc: &Scalp, a: f64) -> impl Fn(f64) -> f64 {
    let h = sc.h;
    // Depths below the centre (rising) and scales.
    let (mut ds, mut ss) = (Vec::new(), Vec::new());
    let (mut deepest, mut widest) = (f64::NEG_INFINITY, 0.0f64);
    for phi in linspace(radians(10.0), radians(170.0), 33) {
        let p = sc.surface(a, phi).0 - h.centre;
        deepest = deepest.max(-p.y);
        widest = widest.max(((p.x / h.radii[0]).powi(2) + (p.z / h.radii[2]).powi(2)).sqrt());
        ds.push(deepest);
        ss.push(widest);
    }
    let cy = h.centre[1];
    move |y: f64| interp(cy - y, &ds, &ss)
}

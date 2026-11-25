use super::*;

// --- the organic generators -------------------------------------------------------------------------

/// The lawn (`grass_lush`, 2 m a tile): tufts of short blades leaning every way over dark thatch,
/// the upper blades lighter, the gaps in shade, the tufts a little uneven in vigour.
pub(super) fn lawn(n: &Noise, r: &mut Rng, size: usize) -> Organic {
    let px_m = size as f64 / 2.0;
    let k = size as f64 / 1024.0;
    let mut c = Canvas::new(size, size);
    c.fill(|u, v| {
        let t = n.fbm(u, v, 64, 3);
        (0.04 * t, 0.42 + 0.20 * t, 0.95)
    });
    // Dead blades lying in the thatch: pale, low.
    for _ in 0..(5000.0 * k * k) as usize {
        let b = Blade {
            dir: r.f() * TAU,
            len: r.range(0.02, 0.05) * px_m,
            half: r.range(1.0, 1.6) * k,
            bend: r.range(-0.15, 0.15),
            z0: 0.05,
            z1: 0.10,
            tone: r.range(0.60, 0.90),
            rough: 0.9,
        };
        let (x, y) = (r.f() * size as f64, r.f() * size as f64);
        c.sprite(x, y, b.len + 2.0, |dx, dy| b.frag(dx, dy));
    }
    // Tufts: a handful of blades from a common crown, leaning outwards; one to a jittered cell
    // (about 1.5 cm), so the sward is even with no bare patches.
    let cells = (130.0 * k) as i64;
    let cell = size as f64 / cells as f64;
    for j in 0..cells {
        for i in 0..cells {
            let x = (i as f64 + 0.5 + r.gauss() * 0.35) * cell;
            let y = (j as f64 + 0.5 + r.gauss() * 0.35) * cell;
            let (u, v) = (x / size as f64, y / size as f64);
            let patch = n.fbm(u, v, 24, 3) - 0.5;
            let lean = r.f() * TAU;
            let m = 4 + (r.f() * 5.0) as usize;
            let crown = r.range(0.20, 0.35);
            let vigour = r.range(0.88, 1.10) * (1.0 + 0.4 * patch);
            for _ in 0..m {
                let dir = lean + r.gauss() * 1.0;
                let len = r.range(0.016, 0.040) * px_m * vigour;
                let b = Blade {
                    dir,
                    len,
                    half: r.range(1.3, 2.1) * k,
                    bend: r.gauss() * 0.15,
                    z0: crown,
                    z1: crown + r.range(0.25, 0.70) * vigour,
                    tone: r.range(0.74, 1.0) * vigour,
                    rough: r.range(0.62, 0.80),
                };
                let (ox, oy) = (r.gauss() * 1.5 * k, r.gauss() * 1.5 * k);
                c.sprite(x + ox, y + oy, len + 3.0, |dx, dy| b.frag(dx, dy));
            }
        }
    }
    finish_layered(c, 0.30, &[(2, 1.2), (6, 0.6)], 0.40)
}

/// Turns a sprite canvas into maps: the tone darkened with depth (light reaches less far down a
/// sward or a crown; `depth_dark` is the share lost at the bottom), AO from depth and cavities.
pub(super) fn finish_layered(
    c: Canvas,
    depth_dark: f64,
    cav: &[(usize, f64)],
    ao_floor: f64,
) -> Organic {
    let (w, h) = (c.w, c.h);
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for &z in &c.z {
        lo = lo.min(z);
        hi = hi.max(z);
    }
    let occ = cavity(&c.z, w, h, cav);
    let mut tone = vec![0.0; w * h];
    let mut ao = vec![0.0; w * h];
    for i in 0..w * h {
        let d = ((c.z[i] - lo) / (hi - lo).max(1e-9)).clamp(0.0, 1.0);
        let lit = (1.0 - depth_dark) + depth_dark * d.powf(0.8);
        tone[i] = (c.tone[i] * lit * (1.0 - 0.6 * occ[i].min(1.0))).clamp(0.0, 1.0);
        ao[i] = (ao_floor + (1.0 - ao_floor) * d.powf(0.6) - 0.5 * occ[i]).clamp(0.05, 1.0);
    }
    Organic {
        height: HeightMap { w, h, v: c.z },
        albedo: grey_img(&tone, w, h),
        ao,
        rough: c.rough,
    }
}

/// Field earth (`grass`, 2 m a tile, tinted brown by the game for soil and furrows): crumbly soil
/// cracked into irregular clods, strewn with litter (flakes of dead leaf and husk, bits of straw)
/// and grit, the hollows in shade.
pub(super) fn earth(n: &Noise, r: &mut Rng, size: usize) -> Organic {
    let k = size as f64 / 1024.0;
    let mut c = Canvas::new(size, size);
    c.fill(|u, v| {
        let (wu, wv) = n.warp(u, v, 0.03, 12);
        // Clods: cells of a warped Voronoi, cracked apart, each heaped unevenly.
        let w = worley(n, wu, wv, 36, 36, 0.95, 3);
        let (d1, d2, (ci, cj)) = (w.f1, w.f2, w.cell);
        let open = smoothstep(0.45, 0.65, n.fbm(u, v, 6, 3));
        let crack = 1.0 - (1.0 - smoothstep(0.0, 0.12, d2 - d1)) * open;
        let heap = 0.5 + 0.5 * n.rand(ci, cj, 5);
        let clod = crack * (0.6 + 0.4 * (1.0 - d1).max(0.0)) * heap;
        // Crumbs: turbulent noise, cut into bits.
        let t1 = (n.fbm(wu, wv, 96, 3) - 0.5).abs() * 2.0;
        let t2 = (n.fbm(u, v, 220, 2) - 0.5).abs() * 2.0;
        let crumbs = smoothstep(0.05, 0.35, t1) * 0.6 + smoothstep(0.05, 0.4, t2) * 0.4;
        let lumpy = n.fbm(u, v, 10, 4);
        let z = 0.22 * clod + 0.30 * crumbs + 0.30 * lumpy;
        let tone = 0.56 + 0.12 * (crumbs - 0.5) + 0.14 * (n.fbm(u, v, 20, 3) - 0.5) + 0.06 * heap
            - 0.10 * (1.0 - crack);
        (z, tone, 0.90 + 0.08 * crumbs)
    });
    // Litter: small flat flakes, some pale (husk, dry leaf), some dark (rotting).
    for _ in 0..(26000.0 * k * k) as usize {
        let (x, y) = (r.f() * size as f64, r.f() * size as f64);
        let (ang, len) = (r.f() * PI, r.range(2.0, 7.0) * k);
        let wid = len * r.range(0.3, 0.8);
        let tone = if r.f() < 0.7 {
            r.range(0.70, 1.0)
        } else {
            r.range(0.25, 0.45)
        };
        let z0 = r.range(0.45, 0.75);
        let (tx, ty) = (r.gauss() * 0.04, r.gauss() * 0.04);
        let rag = r.f() * 10.0;
        c.sprite(x, y, len + 2.0, |dx, dy| {
            let (ca, sa) = (ang.cos(), ang.sin());
            let (lx, ly) = (dx * ca + dy * sa, -dx * sa + dy * ca);
            // A ragged outline.
            let edge = 1.0 + 0.25 * ((lx * 1.7 + rag).sin() * (ly * 2.3 - rag).cos());
            let d = ((lx / len).powi(2) + (ly / wid).powi(2)).sqrt() / edge;
            let a = ((1.0 - d) * wid + 0.5).clamp(0.0, 1.0);
            Some(Frag {
                z: z0 + tx * lx + ty * ly,
                tone: tone * (0.9 + 0.1 * (1.0 - d)),
                rough: 0.85,
                a,
            })
        });
    }
    // Straw and dead stalks lying on the soil.
    for _ in 0..(700.0 * k * k) as usize {
        let b = Blade {
            dir: r.f() * TAU,
            len: r.range(12.0, 40.0) * k,
            half: r.range(1.0, 1.7) * k,
            bend: r.gauss() * 0.12,
            z0: r.range(0.55, 0.80),
            z1: r.range(0.55, 0.80),
            tone: r.range(0.85, 1.05),
            rough: 0.75,
        };
        let (x, y) = (r.f() * size as f64, r.f() * size as f64);
        c.sprite(x, y, b.len + 2.0, |dx, dy| b.frag(dx, dy));
    }
    // Grit: small pale stones.
    for _ in 0..(900.0 * k * k) as usize {
        let rad = r.range(1.0, 3.0) * k;
        let (x, y) = (r.f() * size as f64, r.f() * size as f64);
        let tone = r.range(0.75, 1.0);
        c.sprite(x, y, rad + 1.0, |dx, dy| {
            let d = (dx * dx + dy * dy).sqrt() / rad;
            let a = ((1.0 - d) * rad + 0.5).clamp(0.0, 1.0);
            Some(Frag {
                z: 0.55 + 0.25 * (1.0 - d * d).max(0.0).sqrt(),
                tone,
                rough: 0.6,
                a,
            })
        });
    }
    let occ = cavity(&c.z, size, size, &[(1, 2.0), (3, 2.0), (8, 1.0)]);
    let tone: Vec<f64> = c
        .tone
        .iter()
        .zip(&occ)
        .map(|(&t, &o)| t * (1.0 - 0.75 * o.min(1.0)))
        .collect();
    let ao = occ
        .iter()
        .map(|&o| (1.0 - 0.9 * o).clamp(0.2, 1.0))
        .collect();
    Organic {
        height: HeightMap {
            w: size,
            h: size,
            v: c.z,
        },
        albedo: grey_img(&tone, size, size),
        ao,
        rough: c.rough,
    }
}

/// Tree foliage (1.25 m a tile, on the crowns): layers of small overlapping leaves at every angle,
/// each tilted its own way, the inner ones darker, the gaps in shade.
pub(super) fn foliage(n: &Noise, r: &mut Rng, size: usize) -> Organic {
    let px_m = size as f64 / 1.25;
    let mut c = Canvas::new(size, size);
    c.fill(|u, v| (0.0, 0.50 + 0.10 * n.fbm(u, v, 16, 3), 0.9));
    let k = size as f64 / 512.0;
    // Leaves in sprays: a few to a twig, pointing away from it.
    for _ in 0..(7000.0 * k * k) as usize {
        let (x, y) = (r.f() * size as f64, r.f() * size as f64);
        let depth = r.f();
        let out = r.f() * TAU;
        let m = 3 + (r.f() * 4.0) as usize;
        let tone = r.range(0.70, 1.0);
        for _ in 0..m {
            let dir = out + r.gauss() * 0.8;
            let len = r.range(0.020, 0.034) * px_m;
            let leaf = Leaf {
                dir,
                len,
                wid: len * r.range(0.32, 0.46),
                zc: depth + r.range(0.0, 0.12),
                tilt: (r.gauss() * 0.020 / k, r.gauss() * 0.030 / k),
                fold: r.range(0.004, 0.016) / k,
                tone: tone * r.range(0.85, 1.12),
            };
            let off = len * r.range(0.35, 0.6);
            let (cx, cy) = (x + dir.cos() * off, y + dir.sin() * off);
            c.sprite(cx, cy, len * 0.55 + 2.0, |dx, dy| leaf.frag(dx, dy));
        }
    }
    finish_layered(c, 0.35, &[(2, 1.6), (5, 0.8)], 0.35)
}

/// The plates of a bark: cells of a Voronoi over a wrapping grid of `cols` x `rows` cells, nearest
/// by the grid's own measure (so the cells are as long as its rows are tall), and how far `(u, v)`
/// lies from its plate's border in cm (a tile being 1 m), with the plate's index.
pub(super) fn plates(
    n: &Noise,
    u: f64,
    v: f64,
    cols: i64,
    rows: i64,
    salt: u32,
) -> (f64, i64, i64) {
    let (sx, sy) = (100.0 / cols as f64, 100.0 / rows as f64);
    let (x, y) = (u * cols as f64, v * rows as f64);
    let (cx, cy) = (x.floor() as i64, y.floor() as i64);
    let mut seeds = [(0.0, 0.0); 25];
    let (mut near, mut best, mut id) = (0, f64::INFINITY, (0, 0));
    for (k, seed) in seeds.iter_mut().enumerate() {
        let (i, j) = (cx + k as i64 % 5 - 2, cy + k as i64 / 5 - 2);
        let (wi, wj) = (i.rem_euclid(cols), j.rem_euclid(rows));
        *seed = (
            i as f64 + n.rand(wi, wj, salt),
            j as f64 + n.rand(wi, wj, salt + 1),
        );
        let d = (x - seed.0).powi(2) + (y - seed.1).powi(2);
        if d < best {
            (near, best, id) = (k, d, (wi, wj));
        }
    }
    // The border with each neighbour is the two seeds' bisector in the grid; in cm its normal is
    // squeezed by the scale. The nearest of them is the plate's border.
    let (ax, ay) = seeds[near];
    let mut edge = f64::INFINITY;
    for (k, &(bx, by)) in seeds.iter().enumerate() {
        if k == near {
            continue;
        }
        let (nx, ny) = ((bx - ax) / sx, (by - ay) / sy);
        let (mx, my) = ((x - (ax + bx) * 0.5) * sx, (y - (ay + by) * 0.5) * sy);
        edge = edge.min(-(mx * nx + my * ny) / (nx * nx + ny * ny).sqrt());
    }
    (edge, id.0, id.1)
}

/// Bark (1 m a tile, on the trunks, rings round the trunk along u): thick plates a few cm wide and a
/// hand or two long, braided between deep, dark fissures that wander, pinch and meet, the plates
/// broken across here and there and crazed finely on top; a few knots the plates part around. The
/// plates' tops are grey and weathered with a fibrous grain, the fissures' walls warm brown; moss
/// streaks down the trunk in places, lichen spots the crowns.
pub(super) fn bark(n: &Noise, r: &mut Rng, size: usize) -> Organic {
    let px = 1.0 / size as f64;
    let wrap = |d: f64| d - d.round();
    // Knots: (u, v, half width, half height), in tile units.
    let knots: Vec<[f64; 4]> = (0..3)
        .map(|_| {
            let w = r.range(0.014, 0.024);
            [r.f(), r.f(), w, w * r.range(1.2, 1.7)]
        })
        .collect();
    let (nw, nf, nc, nk) = (n.layer(1), n.layer(2), n.layer(3), n.layer(4));
    let mut z = vec![0.0; size * size];
    let mut rough = vec![0.0; size * size];
    let mut cols = vec![[0.0f64; 3]; size * size];
    for y in 0..size {
        let v0 = (y as f64 + 0.5) * px;
        for x in 0..size {
            let u0 = (x as f64 + 0.5) * px;
            // The nearest knot, in its own radii. The plates part round a knot: close to one the
            // network is drawn from a smaller patch round its centre, so a plate swells to wrap it.
            let (mut kr, mut slit, mut pu, mut pv) = (f64::INFINITY, 0.0, 0.0, 0.0);
            for k in &knots {
                let (dx, dy) = (wrap(u0 - k[0]), wrap(v0 - k[1]));
                let rr = ((dx / k[2]).powi(2) + (dy / k[3]).powi(2)).sqrt();
                if rr < kr {
                    kr = rr;
                    slit = ((dx / k[2] * 3.0).powi(2) + (dy / k[3]).powi(2)).sqrt();
                }
                let pull = 0.65 * (-(rr / 2.6).powi(2)).exp();
                (pu, pv) = (pu + dx * pull, pv + dy * pull);
            }
            let (u, v) = ((u0 - pu).rem_euclid(1.0), (v0 - pv).rem_euclid(1.0));
            // Bent at a hand's scale, ragged at a finger's.
            let (wu, wv) = nw.warp(u, v, 0.12, 4);
            let (wu, wv) = nw.layer(1).warp(wu, wv, 0.045, 12);
            let (wu, wv) = nw.layer(2).warp(wu, wv, 0.014, 28);
            let (ru, rv) = nw.layer(3).warp(wu, wv, 0.006, 64);
            // Plates: cells of a Voronoi stretched along the trunk, their borders the fissures; a
            // finer network splits them again, here deep, there barely.
            let (edge, ci, cj) = plates(n, ru, rv, 17, 3, 11);
            let (su, sv) = nc.warp(ru, rv, 0.025, 24);
            let (edge2, _, _) = plates(&nc, su, sv, 30, 7, 31);
            // A plate's own tone, fading out at its border (where a fissure may close up).
            let tone = lerp(0.5, n.rand(ci, cj, 13), smoothstep(0.0, 1.5, edge));
            let fw =
                0.3 + 1.2 * smoothstep(0.25, 0.75, nf.fbm2(wu, wv, 6, 3, 3)).powf(1.5) + 0.2 * tone;
            let jag = (nf.fbm2(u, v, 90, 60, 3) - 0.5) / 0.15;
            // From the fissures' lips (cm), negative inside them.
            let e = edge - fw + 0.35 * jag;
            let deep2 = smoothstep(0.35, 0.65, nc.fbm2(u, v, 6, 3, 3));
            let e2 = edge2 - (0.12 + 0.35 * deep2) + 0.2 * jag;
            let open = smoothstep(-0.2, 0.25, e);
            let open2 = 1.0 - (1.0 - smoothstep(-0.15, 0.2, e2)) * (0.35 + 0.65 * deep2);
            let crown = smoothstep(0.0, 2.0, e) * smoothstep(-0.1, 1.2, e2).max(1.0 - deep2);
            // The plates' tops flake: short splits along the grain, scaly bits, a fibrous grain.
            let split = nf.fbm2(wu, wv, 64, 9, 3) - 0.5;
            let flake = (1.0 - smoothstep(0.0, 0.03, split.abs()))
                * smoothstep(0.5, 0.62, nf.fbm2(u, v, 22, 8, 2));
            let scale = (nc.fbm(wu, wv, 40, 3) - 0.5).abs();
            let fibre = nf.fbm2(u, v, 150, 12, 3) - 0.5;
            let grit = nf.fbm(u, v, 128, 2) - 0.5;
            let lump = nc.fbm2(u, v, 18, 5, 3) - 0.5;
            let mut h = 0.12
                + open * open2 * (0.30 + (0.26 + 0.14 * tone) * crown)
                + 0.04 * open * (1.0 - open2)
                - 0.10 * smoothstep(0.0, 1.0, -e)
                - 0.10 * flake * crown
                + 0.10 * smoothstep(0.05, 0.25, scale) * crown
                + 0.05 * fibre * open
                + 0.03 * grit
                + 0.08 * lump;
            // Colour: grey on the weathered tops, warm brown down the walls, near black deep in.
            let drift = nk.fbm2(u, v, 3, 2, 3);
            let warm = nk.fbm2(u + 0.3, v, 5, 2, 3);
            let mut top = mix(
                rgb(122, 112, 96),
                rgb(158, 154, 141),
                smoothstep(0.35, 0.65, drift),
            );
            top = mix(top, rgb(120, 94, 68), 0.45 * smoothstep(0.55, 0.70, warm));
            top = mix(
                top,
                rgb(160, 156, 144),
                0.35 * crown * smoothstep(0.45, 0.65, drift),
            );
            top = mul(top, 0.86 + 0.24 * tone);
            let wall = mix(rgb(22, 17, 13), rgb(84, 64, 46), smoothstep(-fw, 0.2, e));
            let wall2 = mix(rgb(40, 31, 24), rgb(90, 72, 54), smoothstep(-0.4, 0.15, e2));
            let mut col = mix(wall2, top, open2 * (0.6 + 0.4 * crown));
            col = mix(wall, col, open);
            col = mix(col, mul(top, 0.45), 0.7 * flake * crown);
            col = mul(
                col,
                0.94 + 0.35 * fibre * open + 0.2 * grit + 0.3 * lump
                    - 0.25 * smoothstep(0.12, 0.0, scale) * crown,
            );
            // Knots: an almond scar sunk in, healed over in rings, the bark swelling round it.
            let inner = 1.0 - smoothstep(0.55, 0.85, kr + 0.15 * jag);
            let lip = (-((kr - 0.95) / 0.3).powi(2)).exp();
            if inner > 0.0 || lip > 0.01 {
                let eye = 1.0 - smoothstep(0.3, 0.75, slit + 0.25 * jag.abs());
                let curl = (kr * 16.0 + 1.5 * jag).sin() * 0.5 + 0.5;
                let kh = 0.40 - 0.22 * eye + 0.04 * curl * (1.0 - eye) + 0.04 * grit;
                h = lerp(h, kh, inner) + 0.12 * lip * open;
                let kc = mix(mul(top, 0.8), rgb(84, 66, 50), 0.3 + 0.3 * curl);
                let kc = mix(kc, rgb(28, 20, 15), eye);
                col = mix(col, mul(kc, 0.9 + 0.4 * grit + 0.3 * fibre), inner);
            }
            // Moss: streaks running down the trunk, thickest beside the fissures.
            let mp = smoothstep(0.50, 0.68, nk.fbm2(u0, v0, 4, 1, 4));
            let moss = mp * (0.35 + 0.65 * (1.0 - crown)) * (0.6 + 0.8 * fibre).clamp(0.0, 1.0);
            col = mix(col, mul(rgb(80, 92, 42), 0.8 + 0.5 * grit), moss * 0.7);
            // Lichen: pale grey-green spots on the crowns.
            let lichen = smoothstep(0.62, 0.74, nk.fbm(u0, v0, 9, 4)) * crown;
            col = mix(col, rgb(160, 164, 146), lichen * 0.45);
            let i = y * size + x;
            z[i] = h;
            cols[i] = col;
            rough[i] =
                0.82 + 0.12 * (1.0 - open * open2) - 0.06 * lichen + 0.06 * moss + 0.04 * grit;
        }
    }
    let occ = cavity(&z, size, size, &[(2, 1.5), (6, 1.0)]);
    let mut img = vec![0u8; size * size * 3];
    let mut ao = vec![0.0; size * size];
    for i in 0..size * size {
        let o = occ[i].min(1.0);
        let c = mul(cols[i], 1.14 * (1.0 - 0.3 * o));
        img[i * 3] = to_u8(c[0]);
        img[i * 3 + 1] = to_u8(c[1]);
        img[i * 3 + 2] = to_u8(c[2]);
        ao[i] =
            ((1.0 - 0.8 * occ[i]) * (0.40 + 0.60 * smoothstep(0.08, 0.55, z[i]))).clamp(0.1, 1.0);
    }
    Organic {
        height: HeightMap {
            w: size,
            h: size,
            v: z,
        },
        albedo: Img {
            w: size,
            h: size,
            ch: 3,
            px: img,
        },
        ao,
        rough,
    }
}

/// Frieze carpet (1 m a tile): twisted yarn ends, each tuft's strands splayed about a lean that
/// drifts across the carpet (the pile's shading), the tips catching the light.
pub(super) fn carpet(n: &Noise, r: &mut Rng, size: usize) -> Organic {
    let k = size as f64 / 512.0;
    let mut c = Canvas::new(size, size);
    c.fill(|u, v| (0.0, 0.50 + 0.1 * n.fbm(u, v, 64, 2), 0.95));
    // Tufts on a jittered grid (one each ~6 mm).
    let cells = (110.0 * k) as i64;
    for j in 0..cells {
        for i in 0..cells {
            let (x, y) = (
                (i as f64 + 0.5 + r.gauss() * 0.35) * size as f64 / cells as f64,
                (j as f64 + 0.5 + r.gauss() * 0.35) * size as f64 / cells as f64,
            );
            let (u, v) = (x / size as f64, y / size as f64);
            let lean = n.fbm(u, v, 3, 3) * TAU * 2.0 + r.gauss() * 0.35;
            let tip = r.range(0.75, 1.0);
            let tone = r.range(0.80, 1.0);
            for _ in 0..6 {
                let dir = lean + r.gauss() * 0.5;
                let len = r.range(3.5, 8.0) * k;
                let b = Blade {
                    dir,
                    len,
                    half: r.range(0.7, 1.1) * k,
                    bend: r.gauss() * 0.30,
                    z0: tip - 0.40,
                    z1: tip * r.range(0.85, 1.0),
                    tone: tone * r.range(0.88, 1.08),
                    rough: 0.95,
                };
                let (ox, oy) = (r.gauss() * 1.5 * k, r.gauss() * 1.5 * k);
                c.sprite(x + ox, y + oy, len + 2.0, |dx, dy| b.frag(dx, dy));
            }
        }
    }
    finish_layered(c, 0.30, &[(1, 1.0), (3, 0.6)], 0.45)
}

/// Linen (0.4 m a tile): a plain weave of slubby threads, each a little uneven in tone and not quite
/// straight, the slubs (short thick stretches) standing out paler, a fuzz of loose fibre over all.
pub(super) fn linen(n: &Noise, size: usize) -> Organic {
    let threads = 150i64;
    let tn = threads as f64;
    let mut z = vec![0.0; size * size];
    let mut tone = vec![0.0; size * size];
    let mut rough = vec![0.0; size * size];
    for y in 0..size {
        let v = (y as f64 + 0.5) / size as f64;
        for x in 0..size {
            let u = (x as f64 + 0.5) / size as f64;
            // Warp threads run along v (at integer x), weft along u; both drift a little.
            // Spacing uneven too (the yarn is), so the weave is no perfect grid.
            let xs =
                u * tn + 0.25 * line_noise(n, v, 3, 0.0, 2) + 0.6 * line_noise(n, u, 37, 7.0, 1);
            let ys =
                v * tn + 0.25 * line_noise(n, u, 3, 1.0, 2) + 0.6 * line_noise(n, v, 37, 9.0, 1);
            let (i, j) = (xs.floor() as i64, ys.floor() as i64);
            let (wi, wj) = (i.rem_euclid(threads), j.rem_euclid(threads));
            // A thread's cross-section where it is `off` from its middle, and how slubby it is here.
            let thread = |idx: i64, along: f64, off: f64, salt: f64| {
                let s = line_noise(n, along, 20, idx as f64 + salt, 2);
                let slub = smoothstep(0.35, 0.9, s);
                let thin = smoothstep(-0.4, -0.9, s);
                let thick = 0.42 * (1.0 + 0.40 * slub - 0.25 * thin);
                let d = (off / thick).abs();
                let streak = line_noise(n, along, 6, idx as f64 + salt + 250.0, 2);
                (
                    (1.0 - d * d).max(0.0).sqrt(),
                    slub - 0.5 * thin + 0.6 * streak,
                )
            };
            let (pw, sw) = thread(wi, v, xs - i as f64 - 0.5, 0.0);
            let (pf, sf) = thread(wj, u, ys - j as f64 - 0.5, 500.0);
            let sign_i = if wi % 2 == 0 { 1.0 } else { -1.0 };
            let sign_j = if wj % 2 == 0 { 1.0 } else { -1.0 };
            let warp_h = pw * (0.62 + 0.38 * sign_i * (PI * ys).sin()) * (1.0 + 0.2 * sw);
            let weft_h = pf * (0.62 - 0.38 * sign_j * (PI * xs).sin()) * (1.0 + 0.2 * sf);
            let fuzz = n.fbm(u, v, 256, 2) - 0.5;
            let i_ = y * size + x;
            let (h, t) = if warp_h >= weft_h {
                (
                    warp_h,
                    0.62 + 0.04 * (n.rand(wi, 1, 11) - 0.5) + 0.10 * sw + 0.05 * pw,
                )
            } else {
                (
                    weft_h,
                    0.60 + 0.04 * (n.rand(wj, 2, 11) - 0.5) + 0.10 * sf + 0.05 * pf,
                )
            };
            z[i_] = h + 0.06 * fuzz;
            tone[i_] = t + 0.05 * fuzz + 0.03 * (n.fbm(u, v, 12, 3) - 0.5);
            rough[i_] = 0.78 + 0.14 * (1.0 - h.clamp(0.0, 1.0));
        }
    }
    let occ = cavity(&z, size, size, &[(1, 1.0), (3, 0.5)]);
    let ao = occ
        .iter()
        .map(|&o| (1.0 - 0.6 * o).clamp(0.5, 1.0))
        .collect();
    let tone: Vec<f64> = tone
        .iter()
        .zip(&occ)
        .map(|(&t, &o)| t * (1.0 - 0.12 * o.min(1.0)))
        .collect();
    Organic {
        height: HeightMap {
            w: size,
            h: size,
            v: z,
        },
        albedo: grey_img(&tone, size, size),
        ao,
        rough,
    }
}

/// Wallpaper (2 m a tile; the game draws its stripes over it): ingrain paper, fine fibres and the
/// odd embedded chip under a coat of paint that has dried a little unevenly.
pub(super) fn wallpaper(n: &Noise, r: &mut Rng, size: usize) -> Organic {
    let k = size as f64 / 1024.0;
    let mut c = Canvas::new(size, size);
    c.fill(|u, v| {
        let (wu, wv) = n.warp(u, v, 0.04, 8);
        let mottle = n.fbm(wu, wv, 6, 5) - 0.5;
        let paper = n.fbm(u, v, 192, 3) - 0.5;
        let cloud = n.fbm(u, v, 24, 3) - 0.5;
        (
            0.30 + 0.08 * paper,
            0.70 + 0.10 * mottle + 0.04 * cloud + 0.05 * paper,
            0.82 + 0.08 * mottle,
        )
    });
    // Ingrain chips: small flat flakes under the paint.
    for _ in 0..(26000.0 * k * k) as usize {
        let (x, y) = (r.f() * size as f64, r.f() * size as f64);
        let (ang, len, wid) = (r.f() * PI, r.range(1.5, 4.0) * k, r.range(0.8, 1.8) * k);
        let lift = r.range(0.10, 0.30);
        c.sprite(x, y, len + 2.0, |dx, dy| {
            let (ca, sa) = (ang.cos(), ang.sin());
            let (lx, ly) = (dx * ca + dy * sa, -dx * sa + dy * ca);
            let d = ((lx / len).powi(2) + (ly / wid).powi(2)).sqrt();
            let a = ((1.0 - d) * wid + 0.5).clamp(0.0, 1.0);
            Some(Frag {
                z: 0.30 + lift * (1.0 - d * d).max(0.0).sqrt(),
                tone: 0.70,
                rough: 0.78,
                a,
            })
        });
    }
    let occ = cavity(&c.z, size, size, &[(1, 1.5), (3, 0.8)]);
    let ao = occ
        .iter()
        .map(|&o| (1.0 - 0.6 * o).clamp(0.5, 1.0))
        .collect();
    let tone: Vec<f64> = c
        .tone
        .iter()
        .zip(&occ)
        .map(|(&t, &o)| t * (1.0 - 0.15 * o.min(1.0)))
        .collect();
    Organic {
        height: HeightMap {
            w: size,
            h: size,
            v: c.z,
        },
        albedo: grey_img(&tone, size, size),
        ao,
        rough: c.rough,
    }
}

/// Painted wood (1 m a tile, on trims): boards' grain showing through satin paint, brush marks
/// along it, paint cracked along the odd grain line, a few knots bleeding through, dents and dings.
pub(super) fn painted_wood(n: &Noise, r: &mut Rng, size: usize) -> Organic {
    let k = size as f64 / 512.0;
    let mut c = Canvas::new(size, size);
    c.fill(|u, v| {
        // Grain along u: fine, slightly wavy lines, the latewood a little raised.
        let (wu, wv) = n.warp(u, v, 0.03, 2);
        let phase = wv * 46.0 + 2.5 * n.fbm2(wu, wv, 2, 3, 4);
        let late = smoothstep(0.55, 0.95, 0.5 + 0.5 * (phase * TAU).sin());
        let brush = n.fbm2(u, v, 3, 140, 3) - 0.5;
        let fine = n.fbm(u, v, 128, 2) - 0.5;
        // Paint checked along a few grain lines, in broken stretches.
        let line = phase.floor() as i64;
        let along = phase - line as f64 - 0.5;
        let check = (1.0 - smoothstep(0.02, 0.06, along.abs()))
            * smoothstep(
                0.55,
                0.65,
                line_noise(n, u, 3, line.rem_euclid(46) as f64, 2) * 0.5 + 0.5,
            )
            * if n.rand(line.rem_euclid(46), 0, 77) < 0.10 {
                1.0
            } else {
                0.0
            };
        let grime = n.fbm(u, v, 8, 4) - 0.5 + 0.5 * (n.fbm(u, v, 40, 3) - 0.5);
        (
            0.5 + 0.08 * late + 0.10 * brush + 0.03 * fine - 0.25 * check,
            0.72 - 0.012 * late + 0.015 * brush + 0.06 * grime - 0.12 * check,
            0.38 + 0.10 * brush.abs() + 0.05 * late + 0.06 * grime,
        )
    });
    // Knots under the paint: a faint raised ring and a stain.
    for _ in 0..(3.0 * k * k).ceil() as usize {
        let (x, y) = (r.f() * size as f64, r.f() * size as f64);
        let (rx, ry) = (r.range(8.0, 16.0) * k, r.range(4.0, 7.0) * k);
        for yy in (y - ry * 2.0) as i64..=(y + ry * 2.0) as i64 {
            for xx in (x - rx * 2.0) as i64..=(x + rx * 2.0) as i64 {
                let d = (((xx as f64 + 0.5 - x) / rx).powi(2)
                    + ((yy as f64 + 0.5 - y) / ry).powi(2))
                .sqrt();
                let i = yy.rem_euclid(size as i64) as usize * size
                    + xx.rem_euclid(size as i64) as usize;
                let ring = (-((d - 1.0) / 0.25).powi(2)).exp();
                c.z[i] += 0.08 * ring;
                c.tone[i] -= 0.05 * (1.0 - smoothstep(0.4, 1.4, d)) * (0.6 + 0.4 * (d * 9.0).cos());
            }
        }
    }
    // Dents and dings: shallow dimples, the deeper ones with dirt in.
    for _ in 0..(60.0 * k * k) as usize {
        let (x, y) = (r.f() * size as f64, r.f() * size as f64);
        let rad = r.range(1.0, 3.5) * k;
        let depth = r.range(0.08, 0.30);
        c.add(x, y, rad, |d| -depth * (1.0 - d * d));
        let i = (y as usize % size) * size + (x as usize % size);
        c.tone[i] -= depth * 0.3;
    }
    let occ = cavity(&c.z, size, size, &[(1, 1.0), (4, 0.5)]);
    let ao = occ
        .iter()
        .map(|&o| (1.0 - 0.5 * o).clamp(0.6, 1.0))
        .collect();
    let tone: Vec<f64> = c
        .tone
        .iter()
        .zip(&occ)
        .map(|(&t, &o)| t * (1.0 - 0.3 * o.min(1.0)))
        .collect();
    Organic {
        height: HeightMap {
            w: size,
            h: size,
            v: c.z,
        },
        albedo: grey_img(&tone, size, size),
        ao,
        rough: c.rough,
    }
}

/// Oak (0.83 m a tile, on furniture and panelling): flat-sawn boards glued side by side, the growth
/// rings cut into cathedral arches and straight stripes, the earlywood's open pores in dark dashes.
pub(super) fn oak(n: &Noise, size: usize) -> Organic {
    // Board edges across u (the tile holds four boards of uneven width).
    let mut edges = [0.0f64; 5];
    let mut acc = 0.0;
    for (b, e) in edges.iter_mut().enumerate().skip(1) {
        acc += 0.8 + 0.4 * n.rand(b as i64, 0, 61);
        *e = acc;
    }
    for e in &mut edges {
        *e /= acc;
    }
    let mut z = vec![0.0; size * size];
    let mut tone = vec![0.0; size * size];
    let mut rough = vec![0.0; size * size];
    for y in 0..size {
        let v = (y as f64 + 0.5) / size as f64;
        for x in 0..size {
            let u = (x as f64 + 0.5) / size as f64;
            let b = (0..4).find(|&b| u < edges[b + 1]).unwrap_or(3);
            let (l, rr) = (edges[b], edges[b + 1]);
            let bi = b as i64;
            // Across the board, in metres from where the pith runs (each board its own cut).
            let width = (rr - l) * 0.83;
            let off = (n.rand(bi, 1, 62) - 0.5) * 0.6;
            let across = ((u - l) / (rr - l) - 0.5 + off) * width;
            // How far out from the pith the cut lies: further at the board's edges (the rings curve
            // away), varying along the board as the log tapers and bends (the arches), and wandering
            // (the log was never a perfect cone), more along the grain than across.
            let curve = 0.06 + 0.10 * n.rand(bi, 2, 63);
            let lean = 0.015 * line_noise(n, v, 1, 10.0 + b as f64, 3);
            let wob = 0.004 * line_noise(n, v, 4, 20.0 + b as f64, 3)
                + 0.07 * (n.fbm2(u, v, 12, 1, 4) - 0.5)
                + 0.006 * (n.fbm2(u, v, 32, 4, 3) - 0.5);
            let rad = 0.2 + across * across / (2.0 * curve) + lean + wob;
            // Growth rings ~6 mm apart (as wide as they can be and still show), their widths uneven.
            let ring_w = 0.005 * (1.0 + 0.35 * line_noise(n, rad * 6.0, 1, 30.0 + b as f64, 2));
            let phase = rad / ring_w;
            let f = phase - phase.floor();
            // Each ring: a dark, porous band of earlywood, then dense latewood paling outwards.
            let early = smoothstep(0.0, 0.06, f) * (1.0 - smoothstep(0.20, 0.38, f));
            let line = 1.0 - smoothstep(0.0, 0.05, f.min(1.0 - f));
            let pores = smoothstep(0.50, 0.70, n.fbm2(u, v, 160, 12, 2)) * early;
            let fleck = n.fbm2(u, v, 64, 4, 3) - 0.5;
            let streak = n.fbm2(u, v, 14, 1, 3) - 0.5;
            let board = (n.rand(bi, 3, 64) - 0.5) * 0.10;
            let t = 0.62 + board - 0.13 * early - 0.06 * line - 0.12 * pores
                + 0.04 * f
                + 0.05 * fleck
                + 0.16 * streak;
            let i = y * size + x;
            // A glue line between boards: a hair darker.
            let joint = (u - l).min(rr - u) * size as f64;
            let seam = 1.0 - smoothstep(0.0, 1.2, joint);
            tone[i] = t * (1.0 - 0.25 * seam);
            z[i] = 0.5 - 0.20 * early - 0.5 * pores + 0.05 * fleck - 0.6 * seam;
            rough[i] = 0.50 + 0.16 * early + 0.12 * pores + 0.05 * fleck;
        }
    }
    let ao = z
        .iter()
        .map(|&h| (0.85 + 0.3 * h).clamp(0.7, 1.0))
        .collect();
    Organic {
        height: HeightMap {
            w: size,
            h: size,
            v: z,
        },
        albedo: grey_img(&tone, size, size),
        ao,
        rough,
    }
}

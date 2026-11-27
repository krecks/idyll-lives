use super::*;

/// Honed travertine floor tiles, 33 cm, thin grout.
pub(super) fn floor_tile(n: &Noise, size: usize) -> Surface {
    let t = TILE_FLOOR_TILE;
    let k = FLOOR_TILES;
    let s = t / k as f64;
    let nc = n.layer(1);
    let np = n.layer(2);
    paint(size, size, t, &[(0.003, 2.0), (0.02, 0.4)], |u, v| {
        let (ix, iy) = ((u * k as f64).floor() as i64, (v * k as f64).floor() as i64);
        let (wi, wj) = (ix.rem_euclid(k), iy.rem_euclid(k));
        let r = |q: u32| n.rand(wi, wj, 60 + q);
        let x = (u * k as f64 - ix as f64) * s;
        let y = (v * k as f64 - iy as f64) * s;
        let g = FLOOR_TILE_HALF_JOINT;
        let d = rounded((x - g).min(s - g - x), (y - g).min(s - g - y), 0.0015);
        let face = smoothstep(0.0, 0.0015, d);
        // Bedding: each tile its own vein direction and clouds.
        let ang = (r(0) - 0.5) * 0.6
            + if r(1) < 0.5 {
                0.0
            } else {
                std::f64::consts::FRAC_PI_2
            };
        let (ca, sa) = (ang.cos(), ang.sin());
        let (lx, ly) = (x * ca + y * sa + r(2) * 7.0, -x * sa + y * ca + r(3) * 7.0);
        let warp = fbm_free(&nc, lx * 4.0, ly * 4.0, 3) * 0.04;
        let bands = fbm_free(&nc.layer(1), lx * 2.0, (ly + warp) * 30.0, 4);
        let clouds = fbm_free(&nc.layer(2), lx * 5.0, ly * 5.0, 4);
        let base = mix(rgb(230, 222, 214), rgb(214, 204, 194), r(4));
        let mut c = mul(
            base,
            1.0 + 0.06 * bands + 0.10 * clouds + (r(5) - 0.5) * 0.10,
        );
        // Grey cloudy shading, stronger on some tiles.
        let shade = smoothstep(0.0, 0.25, clouds + (r(9) - 0.5) * 0.25);
        c = mix(c, rgb(176, 170, 164), shade * 0.35);
        let vein = 1.0 - smoothstep(0.0, 0.012, (bands - 0.08).abs());
        c = mix(c, rgb(196, 184, 170), vein * 0.12);
        // Travertine holes: few, irregular, elongated along the bedding.
        let pore_n = fbm_free(&np, lx * 22.0, (ly + warp) * 70.0, 4) + clouds * 0.2;
        let pore = smoothstep(0.2, 0.26, pore_n);
        // Rust-red blotches and specks in clusters on some tiles.
        let cluster = smoothstep(0.04, 0.18, fbm_free(&np.layer(5), lx * 6.0, ly * 6.0, 3))
            * if r(8) < 0.6 { 1.0 } else { 0.0 };
        let blot = smoothstep(0.05, 0.16, fbm_free(&np.layer(7), lx * 40.0, ly * 40.0, 3));
        let (sp, st) = grains(&np.layer(6), u, v, 300, 0.45, 0.5, 53);
        let rust = mix(rgb(156, 96, 76), rgb(120, 80, 66), st);
        c = mix(c, rust, (sp * 0.8 + blot * 0.6).min(1.0) * cluster);
        let halo = smoothstep(0.12, 0.22, pore_n);
        c = mix(c, rgb(186, 150, 124), halo * 0.3);
        c = mix(c, rgb(120, 104, 90), pore * 0.8);
        let (dot, _) = grains(&np.layer(8), u, v, 240, 0.025, 0.3, 59);
        c = mix(c, rgb(130, 112, 100), dot * 0.6);
        c = mul(
            c,
            1.0 + 0.05 * fbm(&np.layer(9), u, v, 450, 450, 1)
                + 0.03 * fbm(&np.layer(10), u, v, 120, 120, 2),
        );
        let lippage = (x / s - 0.5) * (r(6) - 0.5) * 0.06 + (y / s - 0.5) * (r(7) - 0.5) * 0.06;
        let micro = fbm(&np.layer(2), u, v, 400, 400, 2) * 0.03;
        let h = lerp(
            0.3 + fbm(&np.layer(3), u, v, 300, 300, 2) * 0.2,
            1.0 + lippage + micro - 0.25 * pore,
            face,
        );
        let grout = mul(
            rgb(150, 144, 134),
            1.0 + 0.3 * fbm(&np.layer(4), u, v, 200, 200, 2),
        );
        let col = mix(grout, c, face);
        let rough = lerp(
            0.9,
            0.48 + (r(10) - 0.5) * 0.12 + 0.12 * clouds + 0.4 * pore,
            face,
        );
        Px {
            h,
            c: col,
            r: rough,
        }
    })
}

/// Hexagonal concrete pavers (20 cm), sand-filled joints with dirt.
pub(super) fn hexpaving(n: &Noise, size: usize) -> Surface {
    let t = TILE_HEX;
    // Pointy-top hexes: 7 across, 8 rows (rows must be even to wrap; 1% stretch).
    let (cols, rows) = (HEX_ACROSS, HEX_ROWS);
    let (cw, rh) = (t / cols as f64, t / rows as f64);
    let nd = n.layer(1);
    let ng = n.layer(2);
    paint(size, size, t, &[(0.005, 0.8), (0.03, 0.3)], |u, v| {
        let (x, y) = (u * t, v * t);
        // Nearest lattice centre.
        let cj = (y / rh).floor() as i64;
        let (mut best, mut cell, mut lp) = (f64::INFINITY, (0, 0), (0.0, 0.0));
        for dj in -1..=1 {
            let j = cj + dj;
            let off = if j.rem_euclid(2) == 0 { 0.0 } else { 0.5 };
            let ci = (x / cw - off).floor() as i64;
            for di in -1..=1 {
                let i = ci + di;
                let (px, py) = ((i as f64 + 0.5 + off) * cw, (j as f64 + 0.5) * rh);
                let d = (x - px).powi(2) + (y - py).powi(2);
                if d < best {
                    best = d;
                    cell = (i.rem_euclid(cols), j.rem_euclid(rows));
                    lp = (x - px, y - py);
                }
            }
        }
        let r = |q: u32| n.rand(cell.0, cell.1, 70 + q);
        // The paver: a little rotated and shifted in its bed.
        let ang = (r(0) - 0.5) * 0.05;
        let (ca, sa) = (ang.cos(), ang.sin());
        let (px, py) = (lp.0 - (r(1) - 0.5) * 0.003, lp.1 - (r(2) - 0.5) * 0.003);
        let (px, py) = (px * ca + py * sa, -px * sa + py * ca);
        let hexd = px
            .abs()
            .max((px * 0.5 + py * 0.866_025_4).abs())
            .max((px * 0.5 - py * 0.866_025_4).abs());
        let inr = cw * 0.5 - HEX_HALF_JOINT;
        let wear = fbm(&ng, u, v, 60, 60, 3) * 0.004;
        let d = inr - hexd + wear.min(0.001);
        let face = smoothstep(0.0, 0.002, d);
        let chamfer = smoothstep(0.0, 0.007, d);
        let crown = 1.0 - (px * px + py * py) / (inr * inr) * 0.04;
        // Concrete: sand and small aggregate.
        let sand = fbm(&ng.layer(1), u, v, 360, 360, 2);
        let (agg, at) = grains(&ng.layer(2), u, v, 260, 0.45, 0.45, 23);
        let ph = 0.6 + 0.4 * chamfer + 0.05 * crown + 0.12 * sand - 0.08 * agg * (at - 0.3);
        let jh = 0.1 + fbm(&ng.layer(3), u, v, 200, 200, 3) * 0.3;
        let h = lerp(jh, ph, face);

        let base = mix(rgb(178, 164, 140), rgb(194, 180, 156), r(3));
        let grit = fbm(&ng.layer(5), u, v, 700, 700, 1);
        let mut c = mul(base, 1.0 + 0.35 * sand + 0.2 * grit + (r(4) - 0.5) * 0.08);
        c = mix(
            c,
            if at < 0.5 {
                rgb(132, 122, 106)
            } else {
                rgb(210, 200, 182)
            },
            agg * 0.2,
        );
        // Dirt creeping in from the joints, and broad stains across the path.
        let dirt =
            (1.0 - smoothstep(0.0, 0.025, d)) * smoothstep(-0.15, 0.12, fbm(&nd, u, v, 30, 30, 3));
        c = mix(c, rgb(104, 94, 78), dirt * 0.5);
        let blotch = smoothstep(0.05, 0.2, fbm(&nd.layer(4), u, v, 20, 20, 4));
        c = mix(c, mul(c, 0.82), blotch * 0.6);
        let stain = fbm(&nd.layer(1), u, v, 4, 4, 4);
        c = mul(c, 1.0 + 0.16 * stain);
        let moss = smoothstep(0.08, 0.22, fbm(&nd.layer(2), u, v, 16, 16, 3));
        let joint = mix(
            mul(
                rgb(86, 78, 66),
                1.0 + 0.6 * fbm(&nd.layer(3), u, v, 220, 220, 2),
            ),
            rgb(70, 80, 50),
            moss * 0.6,
        );
        let col = mix(joint, c, face);
        let rough = lerp(1.0, 0.88 + 0.08 * sand, face);
        Px {
            h,
            c: col,
            r: rough,
        }
    })
}

/// Concrete sidewalk slabs (grey detail): 1.4 m square in running bond, pepper-speckled aggregate.
pub(super) fn sidewalk(n: &Noise, size: usize) -> Surface {
    let t = TILE_SIDEWALK;
    let (cols, rows) = SIDEWALK_SLABS;
    let (sw, sh) = (t / cols as f64, t / rows as f64);
    let ng = n.layer(1);
    let nd = n.layer(2);
    paint(size, size, t, &[(0.008, 0.6), (0.04, 0.2)], |u, v| {
        let row = (v * rows as f64).floor() as i64;
        let off = if row.rem_euclid(2) == 0 { 0.0 } else { 0.5 };
        let xx = u * cols as f64 + off;
        let col = xx.floor() as i64;
        let (wi, wj) = (col.rem_euclid(cols), row.rem_euclid(rows));
        let r = |q: u32| n.rand(wi, wj, 120 + q);
        let x = (xx - col as f64) * sw;
        let y = (v * rows as f64 - row as f64) * sh;
        let g = 0.0022;
        let chip = fbm(&ng, u, v, 40, 40, 3) * 0.006;
        let d = rounded((x - g).min(sw - g - x), (y - g).min(sh - g - y), 0.004) + chip.min(0.0);
        let face = smoothstep(0.0, 0.004, d);
        let (pep, pt) = grains(&ng.layer(1), u, v, 200, 0.18, 0.45, 37);
        let sand = fbm(&ng.layer(2), u, v, 256, 256, 1);
        let tilt = (x / sw - 0.5) * (r(0) - 0.5) * 0.15 + (y / sh - 0.5) * (r(1) - 0.5) * 0.15;
        let h = lerp(
            0.2,
            1.0 + tilt + 0.15 * sand - 0.15 * pep * (1.0 - pt),
            face,
        );
        let mut l = 0.84 + (r(2) - 0.5) * 0.05 + 0.08 * sand;
        l = lerp(l, if pt < 0.7 { 0.5 } else { 0.97 }, pep * 0.45);
        l *= 1.0
            + 0.12 * white(&ng, u, v, 256, 61)
            + 0.08 * white(&ng, u, v, 512, 62)
            + 0.08 * fbm(&ng.layer(4), u, v, 128, 128, 2);
        let stain = fbm(&nd, u, v, 5, 5, 4);
        l *= 1.0 + 0.10 * stain;
        let blot = smoothstep(0.1, 0.25, fbm(&nd.layer(1), u, v, 10, 10, 3));
        l *= 1.0 - 0.08 * blot;
        l -= 0.08 * (1.0 - smoothstep(0.0, 0.015, d));
        l = lerp(0.6, l, face);
        Px {
            h,
            c: grey(l),
            r: 0.88 + 0.06 * sand,
        }
    })
}

/// Worn asphalt, 5 m: aggregate in dark binder, broad patches, a few hairline cracks running every
/// which way (some sealed), none of them standing out so the tile's repeat doesn't show.
pub(super) fn asphalt(n: &Noise, size: usize) -> Surface {
    let t = TILE_ROAD;
    let k = size as f64 / 1024.0;
    let (cd, ch) = cracks(
        &n.layer(9),
        size,
        size,
        5,
        size as f64 * 0.14,
        k * 1.0,
        k * 4.0,
    );
    let ng = n.layer(1);
    let nd = n.layer(2);
    let w = size;
    paint(size, size, t, &[(0.012, 1.5), (0.05, 0.4)], move |u, v| {
        let i = ((v * w as f64) as usize).min(w - 1) * w + ((u * w as f64) as usize).min(w - 1);
        let (crack, halo) = (cd[i], ch[i]);
        // Aggregate: stones ~6-12 mm (1-3 px), some light.
        let (st, tone) = grains(&ng, u, v, 520, 0.7, 0.7, 41);
        let (big, bt) = grains(&ng.layer(1), u, v, 240, 0.25, 0.55, 43);
        let binder = fbm(&ng.layer(2), u, v, 400, 400, 2);
        let wear = fbm(&nd, u, v, 3, 3, 4) + 0.5 * fbm(&nd.layer(1), u, v, 12, 12, 3);
        // Worn surface: aggregate exposed and lighter where the binder wore off.
        let exposed = sat(0.5 + wear * 2.0);
        let mut c = mul(rgb(70, 70, 68), 1.0 + 0.4 * binder);
        let stone = mix(rgb(96, 94, 90), rgb(170, 166, 158), tone.powf(1.5));
        c = mix(c, stone, st * (0.45 + 0.4 * exposed));
        let bstone = mix(rgb(80, 78, 74), rgb(150, 146, 138), bt);
        c = mix(c, bstone, big * 0.6);
        let (white, _) = grains(&ng.layer(3), u, v, 330, 0.08, 0.5, 45);
        c = mix(c, rgb(196, 194, 186), white * 0.7);
        c = mul(c, 1.0 + 0.18 * wear);
        // Sealed cracks (dark, smoother tar band) and open cracks.
        let seal =
            smoothstep(0.3, 0.8, halo) * smoothstep(-0.05, 0.1, fbm(&nd.layer(2), u, v, 3, 3, 2));
        c = mix(c, rgb(44, 44, 43), seal * 0.4);
        c = mix(c, rgb(34, 34, 33), crack * 0.55);
        let oil = smoothstep(0.12, 0.28, fbm(&nd.layer(3), u, v, 6, 6, 4));
        c = mul(c, 1.0 - 0.12 * oil);
        let h = 0.5 + 0.35 * st * (0.5 + tone * 0.5) + 0.3 * big + 0.1 * binder
            - 0.6 * crack
            - 0.05 * seal;
        let rough =
            (0.86 - 0.1 * st * tone - 0.25 * seal - 0.1 * oil + 0.1 * crack).clamp(0.3, 1.0);
        Px { h, c, r: rough }
    })
}

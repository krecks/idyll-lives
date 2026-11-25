use super::*;

/// Common brick in running bond: 16 courses and 5 bricks a metre, 9 mm recessed mortar.
pub(super) fn brick(n: &Noise, size: usize) -> Surface {
    let (rows, cols) = (BRICK_COURSES, BRICK_ALONG);
    let t = TILE_BRICK;
    let (bw, bh) = (t / cols as f64, t / rows as f64);
    let palette = [
        rgb(138, 72, 56),
        rgb(124, 64, 52),
        rgb(110, 58, 50),
        rgb(98, 62, 56),
        rgb(134, 88, 70),
        rgb(118, 74, 60),
        rgb(80, 50, 46),
        rgb(146, 96, 76),
    ];
    let nw = n.layer(1);
    let nc = n.layer(2);
    let ng = n.layer(3);
    let nm = n.layer(4);
    paint(size, size, t, &[(0.004, 1.2), (0.02, 0.5)], |u, v| {
        // Slightly wavy courses and perpends.
        let wu = u + fbm(&nw, u, v, 6, 6, 3) * 0.0016 / t;
        let wv = v + fbm(&nw.layer(9), u, v, 6, 6, 3) * 0.0012 / t;
        let row = (wv * rows as f64).floor() as i64;
        let off = if row.rem_euclid(2) == 0 { 0.0 } else { 0.5 }
            + (n.rand(row.rem_euclid(rows), 0, 5) - 0.5) * 0.06;
        let xx = wu * cols as f64 + off;
        let col = xx.floor() as i64;
        let (wi, wj) = (col.rem_euclid(cols), row.rem_euclid(rows));
        let r = |k: u32| n.rand(wi, wj, 20 + k);
        let x = (xx - col as f64) * bw;
        let y = (wv * rows as f64 - row as f64) * bh;
        let joint = BRICK_HALF_JOINT;
        let (l, rr) = (joint + (r(0) - 0.5) * 0.002, joint + (r(1) - 0.5) * 0.002);
        let (tp, bt) = (joint + (r(2) - 0.5) * 0.0015, joint + (r(3) - 0.5) * 0.0015);
        let dx = (x - l).min(bw - rr - x);
        let dy = (y - tp).min(bh - bt - y);
        let chip = fbm(&nc, u, v, 90, 90, 3) * 0.004 + fbm(&nc.layer(2), u, v, 12, 12, 2) * 0.004
            - smoothstep(0.08, 0.22, fbm(&nc.layer(1), u, v, 24, 24, 3)) * 0.006;
        let d = rounded(dx, dy, 0.005) + chip.min(0.001);
        let face = smoothstep(-0.0004, 0.0035, d);
        // Brick face: a slight random tilt and bulge, sandy texture, a few pits.
        let tilt = (x / bw - 0.5) * (r(4) - 0.5) * 0.25 + (y / bh - 0.5) * (r(5) - 0.5) * 0.25;
        let tex = fbm(&ng, u, v, 160, 160, 4) * 0.35 + fbm(&ng.layer(1), u, v, 40, 40, 3) * 0.2;
        let (pit, _) = grains(&ng.layer(2), u, v, 220, 0.05, 0.45, 7);
        let bh_ = 1.0 + tilt + tex - 0.12 * pit;
        // Mortar: recessed, sandy.
        let sand = fbm(&nm, u, v, 300, 300, 2) * 0.35 + fbm(&nm.layer(1), u, v, 60, 60, 3) * 0.2;
        let mh = 0.25 + sand;
        let h = lerp(mh, bh_, face);

        let base = palette[(r(6) * palette.len() as f64) as usize % palette.len()];
        let base = mul(base, 0.86 + 0.26 * r(7));
        // Kiln flashing: one end or face darker.
        let flash =
            sat(0.5 + (if r(8) < 0.5 { x / bw } else { 1.0 - x / bw } - 0.5) * 1.6 * (r(9) - 0.3));
        let mottle = fbm(&nc.layer(3), u, v, 30, 30, 4);
        let streak = fbm(&nc.layer(8), u, v, 12, 90, 3);
        let speck = fbm(&nc.layer(4), u, v, 200, 200, 2);
        let mut c = mul(
            base,
            (1.0 + 0.40 * mottle + 0.15 * streak + 0.18 * speck) * (0.92 + 0.12 * flash),
        );
        // Soot and lime bloom in soft patches.
        let soot = smoothstep(
            0.05,
            0.22,
            fbm(&nc.layer(6), u, v, 20, 20, 4) + (r(10) - 0.5) * 0.2,
        );
        c = mix(c, mul(c, 0.62), soot * 0.6);
        let (dark, _) = grains(&nc.layer(5), u, v, 140, 0.05, 0.4, 11);
        c = mix(c, rgb(52, 34, 30), dark * 0.4);
        c = mix(c, mul(c, 0.8), pit * 0.4);
        // Worn arrises read a little darker; fine sand in the fired skin.
        c = mul(
            c,
            (0.9 + 0.1 * smoothstep(0.0, 0.006, d))
                * (1.0 + 0.12 * fbm(&ng.layer(4), u, v, 420, 420, 1)),
        );
        let lum = 0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2];
        c = mul(mix(c, [lum; 3], 0.22), 0.95);
        // Mortar smeared onto the arris here and there.
        let smear = (1.0 - smoothstep(0.0, 0.004, d))
            * smoothstep(0.05, 0.2, fbm(&nm.layer(2), u, v, 50, 50, 3));
        c = mix(c, rgb(170, 162, 150), smear * 0.6);
        // Soot and weathering across the wall (gentle, so the repeat does not show).
        let grime = fbm(&nc.layer(7), u, v, 2, 3, 4);
        c = mul(c, 1.0 + 0.10 * grime);
        let mortar = mul(
            rgb(124, 121, 115),
            1.0 + 0.5 * fbm(&nm.layer(3), u, v, 260, 260, 2)
                + 0.2 * fbm(&nm.layer(4), u, v, 20, 20, 3),
        );
        let (mgrain, mt) = grains(&nm.layer(5), u, v, 300, 0.5, 0.5, 17);
        let mortar = mix(
            mortar,
            mul(rgb(160, 156, 148), 0.6 + 0.8 * mt),
            mgrain * 0.7,
        );
        let col = mix(mortar, c, face);
        let rough = lerp(0.88, 0.76 + 0.1 * speck + 0.08 * pit, face);
        Px {
            h,
            c: col,
            r: rough,
        }
    })
}

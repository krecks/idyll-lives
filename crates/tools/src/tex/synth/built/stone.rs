use super::*;

/// Coursed rubble stone: thin split stones in uneven courses, dark recessed joints.
pub(super) fn stone(n: &Noise, size: usize) -> Surface {
    let t = TILE_STONE;
    // Course heights, summing to the tile.
    const NC: usize = 25;
    let mut ys = [0.0f64; NC + 1];
    let mut acc = 0.0;
    for k in 0..NC {
        acc += 0.5
            + n.rand(k as i64, 0, 40)
                * (if n.rand(k as i64, 1, 40) < 0.2 {
                    1.4
                } else {
                    0.8
                });
        ys[k + 1] = acc;
    }
    for y in &mut ys {
        *y /= acc;
    }
    let palette = [
        rgb(170, 148, 116),
        rgb(150, 130, 104),
        rgb(128, 110, 88),
        rgb(182, 160, 124),
        rgb(140, 128, 112),
        rgb(158, 128, 94),
        rgb(112, 96, 80),
        rgb(166, 150, 128),
        rgb(146, 122, 96),
    ];
    let nw = n.layer(1);
    let ns = n.layer(2);
    let nc = n.layer(3);
    paint(
        size,
        size,
        t,
        &[(0.006, 1.6), (0.03, 0.7)],
        false,
        move |u, v| {
            let wu = u + fbm(&nw, u, v, 10, 10, 3) * 0.012 / t;
            let wv = v + fbm(&nw.layer(1), u, v, 10, 10, 3) * 0.008 / t;
            let wvv = wv.rem_euclid(1.0);
            let mut k = 0;
            while k + 1 < NC && wvv >= ys[k + 1] {
                k += 1;
            }
            let (y0, y1) = (ys[k], ys[k + 1]);
            let ch = (y1 - y0) * t;
            let y = (wvv - y0) * t;
            let ki = k as i64;
            // Stones along the course.
            let phase = n.rand(ki, 2, 41);
            // Perpends lean a little, the same way along a course.
            let slant = (n.rand(ki, 1, 41) - 0.5) * 0.8;
            let uu = (wu + phase + (y / ch - 0.5) * ch / t * slant).rem_euclid(1.0);
            let mut xs = [0.0f64; 12];
            let mut nx = 0;
            let mut a = 0.0;
            while nx < 11 {
                a += 0.3 + n.rand(ki, nx as i64 + 3, 42) * 0.9;
                nx += 1;
                xs[nx] = a;
                if a > 6.0 {
                    break;
                }
            }
            for x in xs.iter_mut().take(nx + 1) {
                *x /= a;
            }
            let mut s = 0;
            while s + 1 < nx && uu >= xs[s + 1] {
                s += 1;
            }
            let si = s as i64;
            let r = |q: u32| n.rand(ki, si, 50 + q);
            let sw = (xs[s + 1] - xs[s]) * t;
            let x = (uu - xs[s]) * t;
            // Some stones fall short of the course (mortar or a thin filler shows), some are split in two.
            let short = if r(14) < 0.35 { r(15) * 0.35 * ch } else { 0.0 };
            let (top_in, bot_in) = if r(16) < 0.5 {
                (short, 0.0)
            } else {
                (0.0, short)
            };
            let split = r(17) < 0.25 && ch > 0.05;
            let sy = 0.35 + 0.3 * r(18);
            let (y, ch, sub) = if split {
                if y < ch * sy {
                    (y, ch * sy, 0)
                } else {
                    (y - ch * sy, ch * (1.0 - sy), 1)
                }
            } else {
                (y, ch, 0)
            };
            let r = |q: u32| n.rand(ki, si * 2 + sub, 50 + q);
            let gap = 0.0015 + 0.003 * r(0);
            let (gl, gr) = (gap * (0.5 + r(1)), gap * (0.5 + r(2)));
            let (gt, gb) = (
                gap * (0.4 + 0.8 * r(3)) + if sub == 0 { top_in } else { 0.0 },
                gap * (0.4 + 0.8 * r(4)) + if split && sub == 0 { 0.0 } else { bot_in },
            );
            let dx = (x - gl).min(sw - gr - x);
            let dy = (y - gt).min(ch - gb - y);
            let rc = 0.003 + 0.008 * r(5);
            // Broken outlines: fine roughness plus larger bites out of the edges.
            let bite = smoothstep(0.15, 0.3, fbm(&ns.layer(5), u, v, 40, 40, 3)) * 0.008;
            let angular = fbm(&ns, u, v, 70, 70, 3).abs() - 0.08;
            let edge_noise = angular * 0.02 + fbm(&ns.layer(1), u, v, 20, 20, 2) * 0.006 - bite;
            let d = rounded(dx, dy, rc.min(ch * 0.45)) + edge_noise;
            let face = smoothstep(0.0, 0.003, d);
            let round = smoothstep(0.0, 0.008, d);
            // Split face: fine bedding striations, ledges, a random tilt and proud stones.
            let (lx, ly) = (x + r(6) * 10.0, y + r(7) * 10.0);
            let strata = fbm_free(&ns.layer(2), lx * 4.0, ly * 28.0, 4);
            let ledges = (strata * 6.0).floor() / 6.0 * 0.5 + strata * 0.5;
            let scratch = fbm_free(&ns.layer(6), lx * 6.0, ly * 160.0, 3);
            let bumps = fbm(&ns.layer(3), u, v, 50, 50, 4) * 0.3
                + fbm(&ns.layer(4), u, v, 200, 200, 2) * 0.1;
            let tilt = (x / sw - 0.5) * (r(8) - 0.5) * 0.4 + (y / ch - 0.5) * (r(9) - 0.5) * 0.4;
            let proud = r(10) * 0.35;
            let sh = 0.6 + 0.4 * round + ledges * 0.6 + 0.18 * scratch + bumps + tilt + proud;
            // Joints: earthy mortar, raked back, rough.
            let mortar_n = fbm(&nc, u, v, 150, 150, 2);
            let gh = 0.25 + mortar_n * 0.3 + 0.1 * fbm(&nc.layer(7), u, v, 30, 30, 3);
            let h = lerp(gh, sh, face);

            let base = palette[(r(11) * palette.len() as f64) as usize % palette.len()];
            let mut c = mul(base, 0.9 + 0.18 * r(12));
            let band = fbm_free(&nc.layer(1), lx * 3.0, ly * 40.0, 3);
            c = mul(c, 1.0 + 0.35 * band + 0.3 * scratch);
            // Iron staining and a second mineral hue.
            let rust = smoothstep(
                0.04,
                0.2,
                fbm(&nc.layer(2), u, v, 14, 14, 4) + (r(13) - 0.5) * 0.3,
            );
            c = mix(c, rgb(150, 96, 56), rust * 0.35);
            let greyish = smoothstep(
                0.05,
                0.2,
                fbm(&nc.layer(3), u, v, 9, 9, 3) + (r(19) - 0.5) * 0.2,
            );
            c = mix(c, rgb(128, 132, 124), greyish * 0.4);
            let fine = fbm(&nc.layer(4), u, v, 300, 300, 2);
            c = mul(c, 1.0 + 0.35 * fine);
            // Fresh, lighter broken edges; lichen; dark weathering in hollows.
            c = mix(
                c,
                mul(c, 1.12),
                (1.0 - smoothstep(0.0, 0.008, d)) * face * 0.6,
            );
            let (lich, lt) = grains(&nc.layer(5), u, v, 90, 0.05, 0.6, 19);
            c = mix(
                c,
                if lt < 0.5 {
                    rgb(196, 192, 170)
                } else {
                    rgb(70, 72, 58)
                },
                lich * 0.3,
            );
            c = mul(c, 1.0 + (bumps.min(0.0)) * 0.6);
            let gapc = mul(
                rgb(150, 122, 88),
                0.75 + 0.6 * mortar_n + 0.2 * fbm(&nc.layer(6), u, v, 20, 20, 3),
            );
            let gapc = mix(gapc, rgb(64, 52, 40), smoothstep(-0.004, 0.0, d) * 0.8);
            let col = mix(gapc, c, face);
            let rough = lerp(0.9, 0.68 + 0.1 * fine + 0.08 * scratch, face);
            Px {
                h,
                c: col,
                r: rough,
            }
        },
    )
}

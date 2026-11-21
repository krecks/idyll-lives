use super::*;

/// Pine floorboards, 21 cm wide and calm (`boards`), running along v.
pub(super) fn floor_wood(n: &Noise, size: usize, look: &Boards) -> Surface {
    let t = TILE_FLOOR_WOOD;
    let rows = look.rows;
    let bw = t / rows as f64;
    let px_m = t / size as f64;
    let ng = n.layer(1);
    paint(
        size,
        size,
        t,
        &[(0.002, 1.5), (0.02, 0.3)],
        false,
        |u0, v0| {
            let (u, v) = (v0, u0);
            let b = board_at(n, u, v, rows, 2, 0.3, t, 80);
            let lg = log_for(n, b.row, b.seg, bw, b.len, 0.25, (0.004, 0.007));
            let (late, fibre, knot) = grain_at(&ng, &lg, b.x, b.y, px_m);
            let side = b.y.min(bw - b.y);
            let joint = smoothstep(0.0003, 0.0011, side) * smoothstep(0.0003, 0.0011, b.end);
            let bevel = smoothstep(0.0, 0.003, side.min(b.end));
            let cup = -((b.y / bw - 0.5) * 2.0).powi(2) * 0.04;
            let h = lerp(
                0.0,
                0.8 + 0.2 * bevel + cup + 0.03 * late + 0.02 * fibre - 0.04 * knot,
                joint,
            );
            let tone = lg.tone;
            let early = mix(look.early.0, look.early.1, tone);
            let latec = mix(look.late.0, look.late.1, tone);
            let mut c = mix(early, latec, late * 0.75 * look.figure);
            let bright = look.bright.0 + look.bright.1 * n.rand(b.row, b.seg, 88);
            c = mul(
                c,
                bright
                    * (1.0
                        + 0.16 * look.figure * fibre
                        + 0.16
                            * look.figure
                            * fbm_free(&ng.layer(2), b.x * 2.0 + lg.salt, b.y * 20.0, 3)),
            );
            c = mix(c, rgb(156, 94, 46), knot * 0.85 * look.knots);
            let kring = knot * (1.0 - knot) * 4.0;
            c = mix(c, rgb(84, 48, 26), kring * 0.8 * look.knots);
            let col = mix(rgb(70, 48, 30), c, joint);
            let rough = lerp(0.8, 0.58 + 0.06 * late + 0.06 * fibre, joint);
            Px {
                h,
                c: col,
                r: rough,
            }
        },
    )
}

/// Oak-print laminate: 23 cm planks running along u, micro-bevelled joints.
pub(super) fn laminate(n: &Noise, size: usize) -> Surface {
    let t = TILE_LAMINATE;
    let rows = LAMINATE_ROWS;
    let bw = t / rows as f64;
    let px_m = t / size as f64;
    let ng = n.layer(1);
    paint(size, size, t, &[(0.002, 2.0)], false, |u, v| {
        let b = board_at(n, u, v, rows, LAMINATE_JOINTS, 0.5, t, 90);
        let mut lg = log_for(n, b.row, b.seg, bw, b.len, 0.12, (0.004, 0.008));
        lg.z0 += 0.03;
        lg.slope *= 0.6;
        let (late, fibre, knot) = grain_at(&ng, &lg, b.x, b.y, px_m);
        // Oak's figure: long irregular streaks of light and dark along the board.
        let sx = b.x + lg.salt;
        let streak = fbm_free(
            &ng.layer(4),
            sx * 2.5 + fbm_free(&ng.layer(5), sx * 4.0, b.y * 12.0, 2) * 0.6,
            b.y * 45.0,
            5,
        );
        let flame = smoothstep(-0.05, 0.2, streak);
        // Oak pores: short dark dashes along the grain.
        let pore = smoothstep(
            0.17,
            0.26,
            fbm_free(&ng.layer(3), b.x * 60.0 + lg.salt, b.y * 1400.0, 2)
                + late * 0.08
                + streak * 0.2,
        );
        let side = b.y.min(bw - b.y);
        let edge = side.min(b.end);
        let joint = smoothstep(0.0, 0.0012, edge);
        let h = lerp(0.2, 1.0 - 0.05 * pore + 0.01 * fibre, joint);
        let tone = lg.tone;
        let early = mix(rgb(176, 148, 116), rgb(164, 136, 106), tone);
        let latec = mix(rgb(132, 102, 74), rgb(120, 92, 66), tone);
        let mut c = mix(early, latec, late * 0.45 + flame * 0.45);
        c = mul(
            c,
            1.0 + 0.12 * fibre + 0.08 * fbm_free(&ng.layer(2), b.x * 1.5 + lg.salt, b.y * 12.0, 3),
        );
        c = mix(c, rgb(104, 80, 58), pore * 0.3);
        c = mix(c, rgb(110, 80, 56), knot * 0.7);
        let col = mix(rgb(78, 60, 44), c, smoothstep(0.0003, 0.0014, edge));
        let rough = lerp(0.6, 0.36 + 0.12 * pore, joint);
        Px {
            h,
            c: col,
            r: rough,
        }
    })
}

/// Painted lap siding (grey detail): 12 boards of 21 cm across 2.5 m, each thicker at its butt (the
/// top of its course in the image, the bottom on the wall), with a few butt joints.
pub(super) fn siding(n: &Noise, size: usize) -> Surface {
    let t = TILE_SIDING;
    let rows = SIDING_ROWS;
    let bh = t / rows as f64;
    let px_m = t / size as f64;
    let ng = n.layer(1);
    paint(size, size, t, &[(0.004, 2.4), (0.03, 0.6)], true, |u, v| {
        // Half a board down, so the tile edge falls mid-board rather than on a butt.
        let b = board_at(n, u, v + 0.5 / rows as f64, rows, 1, 0.5, t, 95);
        let mut lg = log_for(n, b.row, b.seg, bh, b.len, 0.3, (0.005, 0.009));
        lg.z0 += 0.04;
        lg.slope *= 0.6;
        let (late, fibre, knot) = grain_at(&ng, &lg, b.x, b.y, px_m);
        let fy = b.y / bh;
        // Lap profile: thick butt at fy = 0, thin top under the next board at fy = 1.
        let butt = smoothstep(0.0, 0.006, b.y);
        let lap = 1.0 - 0.45 * fy;
        let end = smoothstep(0.0, 0.0018f64.max(px_m), b.end);
        let h = lerp(0.5 * lap, lap * (0.25 + 0.75 * butt), end) + 0.01 * late + 0.012 * fibre;
        // Paint over wood: grain shows faintly, dirt gathers under the butts.
        let streak = fbm_free(&ng.layer(4), b.x * 2.5 + lg.salt, b.y * 45.0, 4);
        let mut l = 0.80
            + 0.06 * late
            + 0.18 * fibre
            + 0.22 * streak
            + 0.05 * fbm(&ng.layer(2), u, v, 6, 24, 3);
        l -= 0.06 * knot + 0.25 * knot * (1.0 - knot);
        l -= 0.2 * (1.0 - smoothstep(0.0, 0.008, bh - b.y));
        l -= 0.25 * (1.0 - end);
        let streak = fbm(&ng.layer(3), u, v, 20, 3, 3);
        l *= 1.0 + 0.08 * streak;
        let rough = 0.72 + 0.1 * fibre + 0.1 * (1.0 - end);
        Px {
            h,
            c: grey(l),
            r: rough,
        }
    })
}

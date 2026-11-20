use super::*;

/// Shingles / shakes (grey detail): 11 courses on 1.5 m, random widths, ragged butts.
pub(super) fn roof(n: &Noise, size: usize) -> Surface {
    let t = TILE_ROOF;
    let rows = ROOF_COURSES;
    let ch = t / rows as f64;
    let ng = n.layer(1);
    let nb = n.layer(2);
    paint(size, size, t, &[(0.004, 4.0), (0.03, 2.0)], true, |u, v| {
        // Ragged butt line (top of each course in the image); side joints wander a little.
        let jag =
            fbm(&nb, u, v, 160, 4, 3) * 0.008 / t + fbm(&nb.layer(1), u, v, 30, 3, 2) * 0.006 / t;
        let vv = v - jag;
        let uu = u + fbm(&nb.layer(2), u, v, 6, 40, 3) * 0.006 / t;
        let b = board_at(n, uu, vv, rows, 11, 0.05, t, 85);
        let r = |q: u32| n.rand(b.row, b.seg, 100 + q);
        let fy = b.y / ch;
        let gap = 0.002 + 0.004 * r(0);
        let side = smoothstep(gap * 0.5, gap, b.end);
        let butt = smoothstep(0.0, 0.004, b.y);
        // The thick, frayed butt: a raised lip along the course's edge.
        let lipw = 0.006 + 0.004 * (fbm(&nb.layer(3), u, v, 90, 6, 2) + 0.5);
        let lip = 1.0 - smoothstep(lipw * 0.5, lipw, b.y);
        let fray = fbm(&nb.layer(4), u, v, 300, 60, 2);
        let thick = 0.6 + 0.4 * (1.0 - fy);
        let warp = (b.x / b.len - 0.5) * (r(1) - 0.5) * 0.3 + (r(2) - 0.5) * 0.2;
        // Riven/sawn face: strokes at a slant that differs per shingle.
        let ang = (r(6) - 0.5) * 1.2;
        let (ca, sa) = (ang.cos(), ang.sin());
        let (rx, ry) = (b.x * ca + b.y * sa + r(3) * 9.0, -b.x * sa + b.y * ca);
        let grain = fbm_free(&ng, rx * 10.0, ry * 260.0, 4)
            + 0.5 * fbm_free(&ng.layer(4), b.x * 400.0 + r(3) * 99.0, b.y * 20.0, 2);
        let split = smoothstep(
            0.2,
            0.3,
            fbm_free(&ng.layer(1), b.x * 90.0 + r(4) * 50.0, b.y * 4.0, 2),
        );
        let h = side
            * (thick * (0.3 + 0.7 * butt) + warp + 0.14 * grain - 0.15 * split
                + lip * (0.12 + 0.2 * fray) * butt);
        let mut l = 0.62 + (r(5) - 0.5) * 0.14 + 0.14 * grain;
        l *= 1.0 + 0.10 * fbm(&ng.layer(2), u, v, 8, 8, 3);
        // Weathering: the frayed butt lighter, the shadowed tail darker, dark gaps.
        l += 0.06 * lip * fray.max(0.0) * 4.0;
        l -= 0.05 * smoothstep(0.75, 1.0, fy);
        l -= 0.2 * split;
        l = lerp(0.18, l, side);
        let rough = 0.56 + 0.1 * grain.abs() + 0.08 * (1.0 - side);
        Px {
            h,
            c: grey(l),
            r: rough,
        }
    })
}

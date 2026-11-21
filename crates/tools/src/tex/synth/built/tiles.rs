use super::*;
use std::f64::consts::{FRAC_1_SQRT_2, SQRT_2};

// --- shared by the floor coverings ----------------------------------------------------------------

/// The two nearest feature points of a periodic jittered grid (\`nx\` x \`ny\` cells over the tile), in cell
/// units, and the nearest one's cell. \`f2 - f1\` is about twice the distance to the cell's border, so a
/// cell shrunk by it is an angular chip (terrazzo, cork granules).
pub(super) struct Voronoi {
    pub f1: f64,
    pub f2: f64,
    pub cell: (i64, i64),
}

pub(super) fn voronoi(n: &Noise, u: f64, v: f64, nx: i64, ny: i64, salt: u32) -> Voronoi {
    let x = u.rem_euclid(1.0) * nx as f64;
    let y = v.rem_euclid(1.0) * ny as f64;
    let (cx, cy) = (x.floor() as i64, y.floor() as i64);
    let (mut f1, mut f2, mut cell) = (f64::INFINITY, f64::INFINITY, (0, 0));
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (i, j) = (cx + dx, cy + dy);
            let (wi, wj) = (i.rem_euclid(nx), j.rem_euclid(ny));
            let px = i as f64 + 0.1 + 0.8 * n.rand(wi, wj, salt);
            let py = j as f64 + 0.1 + 0.8 * n.rand(wi, wj, salt + 1);
            let d = ((x - px).powi(2) + (y - py).powi(2)).sqrt();
            if d < f1 {
                f2 = f1;
                f1 = d;
                cell = (wi, wj);
            } else if d < f2 {
                f2 = d;
            }
        }
    }
    Voronoi { f1, f2, cell }
}

/// One block of a herringbone laid at 45° to the tile's edges: its key (unique on the tile, for its
/// random draws), whether it runs along one diagonal or the other, and the position in it (m along, m
/// across, from its corner).
pub(super) struct Block {
    pub key: (i64, i64),
    pub turned: bool,
    pub along: f64,
    pub across: f64,
}

/// The herringbone of \`len\` x \`wid\` blocks (joints included) at (x, y) m on a tile of \`t\` m. Turned
/// 45° back it is the herringbone of upright and lying blocks that repeats by (wid, wid) and (len, -len);
/// those are (0, wid √2) and (len √2, 0) on the tile, so the tile holds a whole number of both.
pub(super) fn herringbone(x: f64, y: f64, len: f64, wid: f64, t: f64) -> Block {
    let (qx, qy) = ((x + y) * FRAC_1_SQRT_2, (y - x) * FRAC_1_SQRT_2);
    let pm = (t / (wid * SQRT_2)).round() as i64;
    let pn = (t / (len * SQRT_2)).round() as i64;
    let n0 = ((qx - qy) / (2.0 * len)).floor() as i64;
    let key = |m: f64, k: i64, nn: i64| ((m as i64).rem_euclid(pm) * 2 + k, nn.rem_euclid(pn));
    // The block whose length holds the point (on a joint, where rounding can leave it in neither, the
    // nearer).
    let mut best = (
        f64::INFINITY,
        Block {
            key: (0, 0),
            turned: false,
            along: 0.0,
            across: 0.0,
        },
    );
    let outside = |a: f64| (-a).max(a - len).max(0.0);
    for nn in n0 - 2..=n0 + 2 {
        let nl = nn as f64 * len;
        // The lying block [0, len] x [0, wid], moved by m (wid, wid) + nn (len, -len).
        let m = ((qy + nl) / wid).floor();
        let bx = qx - m * wid - nl;
        if outside(bx) < best.0 {
            best = (
                outside(bx),
                Block {
                    key: key(m, 0, nn),
                    turned: false,
                    along: bx,
                    across: qy - m * wid + nl,
                },
            );
        }
        // The upright one [0, wid] x [wid, wid + len] beside it.
        let m = ((qx - nl) / wid).floor();
        let by = qy - m * wid + nl - wid;
        if outside(by) < best.0 {
            best = (
                outside(by),
                Block {
                    key: key(m, 1, nn),
                    turned: true,
                    along: by,
                    across: qx - m * wid - nl,
                },
            );
        }
    }
    best.1
}

/// A square grid of \`k\` x \`k\` pieces on the tile: the piece and the position in it (m), for pieces of \`s\` m.
fn grid(u: f64, v: f64, k: i64, s: f64) -> ((i64, i64), f64, f64) {
    let (ix, iy) = ((u * k as f64).floor() as i64, (v * k as f64).floor() as i64);
    (
        (ix.rem_euclid(k), iy.rem_euclid(k)),
        (u * k as f64 - ix as f64) * s,
        (v * k as f64 - iy as f64) * s,
    )
}

/// The distance (m) from \`x\` to the nearest of the lines every \`every\` m from 0.
pub(super) fn to_line(x: f64, every: f64) -> f64 {
    let r = x.rem_euclid(every);
    r.min(every - r)
}

/// A ridge 1 on the line where \`f\` crosses 0, falling to 0 at \`width\` from it.
pub(super) fn ridge(f: f64, width: f64) -> f64 {
    1.0 - smoothstep(0.0, width, f.abs())
}

// --- stone and ceramic ----------------------------------------------------------------------------

/// Black and white marble chequerboard, 30 cm squares, 2 mm grout: Carrara-like white with soft grey
/// veins and near-black (Nero Marquina) with fine white ones, honed and sealed.
pub(super) fn checker(n: &Noise, size: usize) -> Surface {
    let t = TILE_CHECKER;
    let k = CHECKER_SQUARES;
    let s = t / k as f64;
    let nv = n.layer(1);
    let ng = n.layer(2);
    paint(
        size,
        size,
        t,
        &[(0.002, 2.0), (0.02, 0.3)],
        false,
        |u, v| {
            let ((wi, wj), x, y) = grid(u, v, k, s);
            let r = |q: u32| n.rand(wi, wj, 140 + q);
            let black = (wi + wj).rem_euclid(2) == 1;
            let g = CHECKER_HALF_JOINT;
            let d = rounded((x - g).min(s - g - x), (y - g).min(s - g - y), 0.001);
            let face = smoothstep(0.0, 0.0008, d);
            let ease = smoothstep(0.0, 0.0025, d);
            // Each square cut from its own block: its own vein direction.
            let ang = r(0) * std::f64::consts::PI;
            let (ca, sa) = (ang.cos(), ang.sin());
            let (lx, ly) = (x * ca + y * sa + r(1) * 9.0, -x * sa + y * ca + r(2) * 9.0);
            let warp = fbm_free(&nv, lx * 3.0, ly * 3.0, 4) * 0.35;
            let main = fbm_free(&nv.layer(1), lx * 2.2 + warp, ly * 6.0 + warp, 5);
            let fine = fbm_free(&nv.layer(2), lx * 7.0 + warp * 2.0, ly * 9.0, 4);
            let clouds = fbm_free(&nv.layer(3), lx * 4.0, ly * 4.0, 4);
            let grain = fbm(&ng, u, v, 500, 500, 1);
            let mut c;
            if black {
                c = mul(
                    rgb(46, 45, 46),
                    1.0 + 0.10 * clouds + (r(3) - 0.5) * 0.06 + 0.04 * grain,
                );
                let vein = ridge(main, 0.006) * smoothstep(-0.05, 0.12, clouds + r(4) * 0.2 - 0.05);
                let hair = ridge(fine, 0.004) * smoothstep(0.0, 0.15, clouds + 0.05);
                c = mix(c, rgb(176, 172, 166), vein * 0.75);
                c = mix(c, rgb(120, 118, 116), hair * 0.35);
            } else {
                c = mul(
                    rgb(228, 227, 223),
                    1.0 + 0.025 * clouds + (r(3) - 0.5) * 0.03 + 0.012 * grain,
                );
                let drift = smoothstep(0.0, 0.3, clouds + (r(4) - 0.5) * 0.3);
                c = mix(c, rgb(204, 204, 204), drift * 0.45);
                let vein = ridge(main, 0.016);
                let hair = ridge(fine, 0.005) * smoothstep(-0.1, 0.1, clouds);
                c = mix(c, rgb(150, 152, 158), vein * 0.55);
                c = mix(c, rgb(128, 130, 136), hair * 0.4);
            }
            let grout = mul(
                rgb(150, 148, 144),
                1.0 + 0.2 * fbm(&ng.layer(1), u, v, 300, 300, 2),
            );
            let lip = (x / s - 0.5) * (r(5) - 0.5) * 0.04 + (y / s - 0.5) * (r(6) - 0.5) * 0.04;
            let h = lerp(0.25, 0.9 + 0.1 * ease + lip + 0.004 * grain, face);
            let rough = lerp(0.85, 0.24 + 0.05 * clouds + (r(7) - 0.5) * 0.04, face);
            Px {
                h,
                c: mix(grout, c, face),
                r: rough,
            }
        },
    )
}

/// White glazed hexagon mosaic, 58 mm across the flats, 2.5 mm light grey grout, for a covering to tint.
pub(super) fn hex_mosaic(n: &Noise, size: usize) -> Surface {
    let t = TILE_HEX_MOSAIC;
    let (cols, rows) = HEX_MOSAIC;
    let (cw, rh) = (t / cols as f64, t / rows as f64);
    let ng = n.layer(1);
    paint(
        size,
        size,
        t,
        &[(0.0015, 2.0), (0.012, 0.3)],
        false,
        |u, v| {
            let (x, y) = (u * t, v * t);
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
            let r = |q: u32| n.rand(cell.0, cell.1, 150 + q);
            // Each piece a little off its place on the sheet.
            let (px, py) = (lp.0 - (r(0) - 0.5) * 0.0008, lp.1 - (r(1) - 0.5) * 0.0008);
            let hexd = px
                .abs()
                .max((px * 0.5 + py * 0.866_025_4).abs())
                .max((px * 0.5 - py * 0.866_025_4).abs());
            let d = cw * 0.5 - HEX_MOSAIC_HALF_JOINT - hexd;
            let face = smoothstep(0.0, 0.0006, d);
            // A glazed piece's cushioned edge.
            let cushion = smoothstep(0.0, 0.004, d);
            let glaze = fbm(&ng, u, v, 60, 60, 2);
            let h = lerp(0.2, 0.7 + 0.3 * cushion.sqrt() + 0.01 * glaze, face);
            let tone = 1.0 + (r(2) - 0.5) * 0.045 + 0.012 * glaze;
            let mut c = mul(rgb(230, 230, 228), tone);
            // Glaze pools a touch darker at the rim.
            c = mul(c, 1.0 - 0.05 * (1.0 - cushion));
            let grout = mul(
                rgb(190, 190, 188),
                1.0 + 0.08 * fbm(&ng.layer(2), u, v, 120, 120, 2),
            );
            let dirt = smoothstep(0.0, 0.2, fbm(&ng.layer(3), u, v, 12, 12, 3));
            let grout = mul(grout, 1.0 - 0.10 * dirt);
            let rough = lerp(0.88, 0.17 + 0.06 * (1.0 - cushion), face);
            Px {
                h,
                c: mix(grout, c, face),
                r: rough,
            }
        },
    )
}

/// Cream terrazzo: crushed marble (white, greys, black, terracotta, sage) in a cream matrix, ground and
/// polished, poured in 1 m bays between brass divider strips.
pub(super) fn terrazzo(n: &Noise, size: usize) -> Surface {
    let t = TILE_TERRAZZO;
    let nm = n.layer(1);
    let palette = [
        (rgb(226, 223, 216), 0.24),
        (rgb(200, 196, 188), 0.16),
        (rgb(152, 150, 146), 0.13),
        (rgb(108, 106, 104), 0.09),
        (rgb(54, 52, 52), 0.08),
        (rgb(180, 104, 76), 0.12),
        (rgb(148, 160, 130), 0.08),
        (rgb(214, 190, 152), 0.10),
    ];
    let pick = |x: f64| {
        let mut acc = 0.0;
        for &(c, w) in &palette {
            acc += w;
            if x < acc {
                return c;
            }
        }
        palette[0].0
    };
    paint(
        size,
        size,
        t,
        &[(0.001, 1.0), (0.01, 0.2)],
        false,
        |u, v| {
            let (x, y) = (u * t, v * t);
            // Each bay its own pour: a shade apart.
            let bay = n.rand(
                (x / TERRAZZO_BAY).floor() as i64,
                (y / TERRAZZO_BAY).floor() as i64,
                230,
            );
            let cloud = fbm(&nm.layer(1), u, v, 6, 6, 4);
            let dust = fbm(&nm, u, v, 80, 80, 2);
            let mut c = mul(
                rgb(228, 216, 194),
                1.0 + 0.03 * cloud + 0.03 * dust + (bay - 0.5) * 0.04,
            );
            let mut chip_h = 0.0;
            // Chips at three sizes (6 to 30 mm), the larger over the smaller: Voronoi cells shrunk from their borders.
            let sizes = [(240i64, 0.22, 0.22), (120, 0.28, 0.16), (60, 0.16, 0.12)];
            for (k, &(cells, density, gap)) in sizes.iter().enumerate() {
                let vo = voronoi(
                    &nm.layer(10 + k as u64),
                    u,
                    v,
                    cells,
                    cells,
                    30 + k as u32 * 5,
                );
                let (i, j) = vo.cell;
                let rr = |q: u32| nm.rand(i, j, 200 + k as u32 * 10 + q);
                if rr(0) > density {
                    continue;
                }
                let edge = vo.f2 - vo.f1 - gap * (0.6 + 0.8 * rr(1));
                let cover = smoothstep(0.0, 0.06, edge);
                if cover <= 0.0 {
                    continue;
                }
                let mut cc = pick(rr(2));
                // A chip's own veins and clouds.
                let figure = if k == 0 {
                    0.0
                } else {
                    fbm_free(&nm.layer(20), x * 40.0 + rr(4) * 90.0, y * 40.0, 2)
                };
                cc = mul(cc, 1.0 + (rr(3) - 0.5) * 0.10 + 0.10 * figure);
                c = mix(c, cc, cover);
                chip_h = cover;
            }
            // The brass strip (no metal: the arm map has none, so it reads as brass-coloured polish).
            let ds = to_line(x, TERRAZZO_BAY).min(to_line(y, TERRAZZO_BAY));
            let strip = 1.0
                - smoothstep(
                    TERRAZZO_HALF_STRIP - 0.0004,
                    TERRAZZO_HALF_STRIP + 0.0004,
                    ds,
                );
            let brass = mul(
                rgb(192, 154, 86),
                1.0 + 0.06 * fbm(&nm.layer(3), u, v, 120, 120, 2),
            );
            c = mix(c, brass, strip);
            let rim = 1.0 - smoothstep(0.0, 0.0012, (ds - TERRAZZO_HALF_STRIP).abs());
            c = mix(c, mul(c, 0.8), rim * 0.5);
            // Ground flat: chips and matrix one plane, the odd pit where a grain fell out.
            let (pit, _) = grains(&nm.layer(4), u, v, 500, 0.05, 0.35, 81);
            c = mix(c, rgb(150, 138, 118), pit * 0.5);
            let wear = fbm(&nm.layer(5), u, v, 8, 8, 3);
            let h = 0.6 + 0.01 * chip_h - 0.4 * pit + 0.05 * strip - 0.15 * rim;
            let rough = (0.25 + 0.05 * wear + 0.4 * pit + 0.1 * rim - 0.06 * strip).clamp(0.1, 1.0);
            Px { h, c, r: rough }
        },
    )
}

/// White marble slabs, 60 cm with hairline joints, polished: soft grey veins that run on from slab to slab
/// as one block's would, mostly on the diagonal, with fainter cross veins and grey clouds.
pub(super) fn marble(n: &Noise, size: usize) -> Surface {
    let t = TILE_MARBLE;
    let k = MARBLE_SLABS;
    let s = t / k as f64;
    let nv = n.layer(1);
    paint(
        size,
        size,
        t,
        &[(0.0015, 2.0), (0.015, 0.2)],
        false,
        |u, v| {
            // Half a slab over, so the tile's edge falls mid-slab rather than on a joint.
            let half = 0.5 / k as f64;
            let ((wi, wj), x, y) = grid(u + half, v + half, k, s);
            let r = |q: u32| n.rand(wi, wj, 160 + q);
            let g = MARBLE_HALF_JOINT;
            let d = (x - g).min(s - g - x).min((y - g).min(s - g - y));
            let face = smoothstep(0.0, 0.0006, d);
            let ease = smoothstep(0.0, 0.002, d);
            // The veins: level lines of fields that change slowly along the diagonal (u + v) and fast across it
            // (both whole multiples of the tile, so they wrap), meandering by a warp.
            let (wu, wv) = nv.warp(u, v, 0.16, 2);
            let (wu, wv) = nv.layer(1).warp(wu, wv, 0.008, 4);
            let diag = |a: f64, b: f64| ((a + b).rem_euclid(1.0), (b - a).rem_euclid(1.0));
            let (p, q) = diag(wu, wv);
            let main = nv.layer(2).fbm2(p, q, 1, 3, 3) - 0.5
                + 0.08 * (nv.layer(7).fbm2(p, q, 4, 12, 2) - 0.5);
            let (p2, q2) = (
                (2.0 * wu - wv).rem_euclid(1.0),
                (wu + wv * 3.0).rem_euclid(1.0),
            );
            let cross = nv.layer(3).fbm2(p2, q2, 2, 2, 3) - 0.5;
            let fine = nv.layer(4).fbm2(p, q, 2, 10, 2) - 0.5;
            let clouds = nv.layer(5).fbm2(wu, wv, 4, 4, 5) - 0.5;
            let strength = smoothstep(-0.1, 0.12, nv.layer(6).fbm2(p, q, 3, 2, 3) - 0.5);
            let mut c = mul(
                rgb(233, 232, 229),
                1.0 + 0.02 * clouds + (r(0) - 0.5) * 0.02,
            );
            // Grey drifts beside the veins, and in clouds.
            let drift = (1.0 - smoothstep(0.0, 0.05, main.abs())) * (0.5 + 0.5 * strength);
            c = mix(c, rgb(208, 209, 212), drift * 0.55);
            c = mix(c, rgb(214, 215, 218), smoothstep(0.02, 0.2, clouds) * 0.5);
            let vein = ridge(main, 0.006) * (0.35 + 0.65 * strength);
            c = mix(c, rgb(140, 144, 152), vein * 0.75);
            let cvein = ridge(cross, 0.004) * smoothstep(0.0, 0.15, clouds + 0.08);
            c = mix(c, rgb(168, 171, 178), cvein * 0.3);
            let hair = ridge(fine, 0.008) * strength * smoothstep(0.0, 0.2, clouds + 0.1);
            c = mix(c, rgb(190, 192, 196), hair * 0.35);
            let joint = mul(c, 0.86);
            let lip = (x / s - 0.5) * (r(1) - 0.5) * 0.02 + (y / s - 0.5) * (r(2) - 0.5) * 0.02;
            let h = lerp(0.3, 0.85 + 0.15 * ease + lip, face);
            let rough = lerp(
                0.6,
                0.11 + 0.02 * clouds + 0.02 * vein + 0.02 * (r(3) - 0.5),
                face,
            );
            Px {
                h,
                c: mix(joint, c, face),
                r: rough,
            }
        },
    )
}

/// Brick floor pavers, 190 x 90 mm with 10 mm mortar joints, in a 45° herringbone: warm reds from orange
/// to plum, worn soft at the arrises, each a little proud or sunk.
pub(super) fn pavers(n: &Noise, size: usize) -> Surface {
    let t = TILE_PAVERS;
    let (len, wid) = PAVER;
    let ng = n.layer(1);
    let nd = n.layer(2);
    paint(
        size,
        size,
        t,
        &[(0.004, 1.6), (0.03, 0.35)],
        false,
        |u, v| {
            let b = herringbone(u * t, v * t, len, wid, t);
            let (i, j) = b.key;
            let r = |q: u32| n.rand(i, j, 170 + q);
            let g = PAVER_HALF_JOINT;
            let chip = fbm(&ng, u, v, 90, 90, 3) * 0.006;
            let d = rounded(
                (b.along - g).min(len - g - b.along),
                (b.across - g).min(wid - g - b.across),
                0.004,
            ) + chip.min(0.0);
            let face = smoothstep(0.0, 0.0015, d);
            let arris = smoothstep(0.0, 0.008, d);
            let (fx, fy) = (b.along / len - 0.5, b.across / wid - 0.5);
            let tilt = fx * (r(0) - 0.5) * 0.12 + fy * (r(1) - 0.5) * 0.10 + (r(2) - 0.5) * 0.10;
            let sand = fbm(&ng.layer(1), u, v, 160, 160, 3);
            let (grit, gt) = grains(&ng.layer(2), u, v, 300, 0.12, 0.45, 91);
            let h = lerp(
                0.1 + 0.25 * fbm(&ng.layer(3), u, v, 120, 120, 3),
                0.75 + 0.25 * arris.sqrt() + tilt + 0.04 * sand - 0.03 * grit * gt,
                face,
            );
            // A brick's own colour: most warm red, some orange, some dark and plum (the kiln's hot side).
            let base = match r(3) {
                x if x < 0.10 => rgb(176, 96, 66),
                x if x < 0.22 => rgb(116, 64, 54),
                x if x < 0.28 => rgb(136, 92, 78),
                _ => mix(rgb(156, 76, 56), rgb(140, 68, 52), r(4)),
            };
            let flash = fbm_free(&nd, b.along * 9.0 + r(5) * 40.0, b.across * 9.0, 3);
            let mut c = mul(base, 1.0 + (r(6) - 0.5) * 0.10 + 0.14 * sand + 0.22 * flash);
            c = mix(
                c,
                if gt < 0.5 {
                    rgb(92, 50, 40)
                } else {
                    rgb(196, 150, 116)
                },
                grit * 0.3,
            );
            // Worn paler where it is walked on, mortar dust in the low parts.
            let wear = smoothstep(-0.05, 0.2, fbm(&nd.layer(1), u, v, 6, 6, 3));
            c = mix(c, rgb(176, 128, 104), wear * 0.18 * arris);
            c = mix(c, rgb(150, 120, 100), (1.0 - arris) * 0.3);
            let mortar = mul(
                rgb(156, 148, 134),
                1.0 + 0.2 * fbm(&nd.layer(2), u, v, 120, 120, 2),
            );
            let mortar = mix(
                mortar,
                rgb(120, 110, 98),
                smoothstep(0.0, 0.2, fbm(&nd.layer(3), u, v, 20, 20, 3)) * 0.4,
            );
            let rough = lerp(0.95, 0.78 + 0.08 * sand - 0.08 * wear, face);
            Px {
                h,
                c: mix(mortar, c, face),
                r: rough,
            }
        },
    )
}

/// Victorian geometric hall tiles: unglazed pieces, one colour through, on a 17 cm grid. In each square a
/// four-pointed star (a square on point cut on its diagonals into four points round a 54 mm square), the
/// stars red and black by turns on a buff ground of squares on point; the red stars' centres black, the black
/// ones' blue-grey. The pattern repeats every 34 cm.
pub(super) fn encaustic(n: &Noise, size: usize) -> Surface {
    let t = TILE_ENCAUSTIC;
    let c0 = ENCAUSTIC_CELL;
    let k = (t / c0).round() as i64;
    let ng = n.layer(1);
    let nd = n.layer(2);
    let red = rgb(158, 74, 52);
    let black = rgb(50, 47, 46);
    let buff = rgb(214, 196, 160);
    let slate = rgb(108, 116, 120);
    let half = c0 * 0.5;
    let sq = ENCAUSTIC_CENTRE;
    paint(
        size,
        size,
        t,
        &[(0.0015, 2.0), (0.012, 0.3)],
        false,
        |u, v| {
            let (cx, cy) = ((u * k as f64).floor() as i64, (v * k as f64).floor() as i64);
            let lx = (u * k as f64 - cx as f64 - 0.5) * c0;
            let ly = (v * k as f64 - cy as f64 - 0.5) * c0;
            let (ax, ay) = (lx.abs(), ly.abs());
            let d1 = ax + ay;
            let star_red = (cx + cy).rem_euclid(2) == 0;
            let (wx, wy) = (cx.rem_euclid(k), cy.rem_euclid(k));
            // Which piece, its colour, and the distance to its edge (m).
            let (piece, col, edge) = if d1 >= half {
                // The ground round the grid's corner: four small squares on point about a dot.
                let (kx, ky) = (
                    (cx + (lx > 0.0) as i64).rem_euclid(k),
                    (cy + (ly > 0.0) as i64).rem_euclid(k),
                );
                let (gx, gy) = (lx - half.copysign(lx), ly - half.copysign(ly));
                let to_dot = gx.abs().max(gy.abs()) - ENCAUSTIC_DOT;
                if to_dot < 0.0 {
                    ((kx, ky * 8 + 6), slate, -to_dot)
                } else {
                    let q = 2 * ((gx + gy) > 0.0) as i64 + ((gx - gy) > 0.0) as i64;
                    let e = ((d1 - half) * FRAC_1_SQRT_2)
                        .min((gx + gy).abs() * FRAC_1_SQRT_2)
                        .min((gx - gy).abs() * FRAC_1_SQRT_2)
                        .min(to_dot);
                    ((kx * 4 + q, ky * 8 + 7), buff, e)
                }
            } else if ax.max(ay) < sq {
                (
                    (wx, wy * 8),
                    if star_red { black } else { red },
                    sq - ax.max(ay),
                )
            } else {
                // A point: the star's quarter beyond the centre square, between its diagonals.
                let q = if ax > ay {
                    if lx > 0.0 { 1 } else { 2 }
                } else if ly > 0.0 {
                    3
                } else {
                    4
                };
                let e = ((half - d1) * FRAC_1_SQRT_2)
                    .min((ax - ay).abs() * FRAC_1_SQRT_2)
                    .min(ax.max(ay) - sq);
                ((wx, wy * 8 + q), if star_red { red } else { black }, e)
            };
            let r = |q: u32| n.rand(piece.0, piece.1, 180 + q);
            let g = ENCAUSTIC_HALF_JOINT;
            let wob = fbm(&ng, u, v, 160, 160, 2) * 0.0004;
            let d = edge - g + wob;
            let face = smoothstep(0.0, 0.0006, d);
            let ease = smoothstep(0.0, 0.002, d);
            let clay = fbm(&ng.layer(1), u, v, 200, 200, 2);
            let mottle = fbm(&ng.layer(2), u, v, 40, 40, 3);
            // Each piece fired a little differently: its own shade, sometimes a cloud across it.
            let cloud = fbm_free(&ng.layer(4), lx * 30.0 + r(3) * 50.0, ly * 30.0, 3);
            let mut c = mul(
                col,
                1.0 + (r(0) - 0.5) * 0.10 + 0.05 * clay + 0.08 * mottle + 0.12 * cloud,
            );
            // Specks of grog in the clay.
            let (sp, st) = grains(&ng.layer(3), u, v, 400, 0.05, 0.4, 97);
            c = mix(
                c,
                if st < 0.5 {
                    mul(col, 0.75)
                } else {
                    mix(col, buff, 0.4)
                },
                sp * 0.3,
            );
            // A century of feet: a pale haze worn into the dark pieces, grime at the edges.
            let wear = smoothstep(-0.1, 0.25, fbm(&nd, u, v, 5, 5, 4));
            c = mix(
                c,
                mix(c, rgb(150, 136, 120), 0.3),
                wear * 0.5 * (1.0 - luma01(col)),
            );
            c = mix(c, mul(c, 0.84), (1.0 - ease) * 0.6);
            let grout = mul(
                rgb(128, 120, 108),
                1.0 + 0.2 * fbm(&nd.layer(1), u, v, 150, 150, 2),
            );
            let lip = (r(1) - 0.5) * 0.08;
            let h = lerp(0.2, 0.85 + 0.15 * ease + lip, face);
            let rough = lerp(
                0.9,
                0.5 + 0.04 * clay + 0.08 * wear + (r(2) - 0.5) * 0.06,
                face,
            );
            Px {
                h,
                c: mix(grout, c, face),
                r: rough,
            }
        },
    )
}

fn luma01(c: [f64; 3]) -> f64 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every point of the plane lies in exactly one block of the herringbone, and the tile wraps onto
    /// the same blocks.
    #[test]
    fn the_herringbone_tiles() {
        let (len, wid, t) = PAVER_TILE_TEST;
        for i in 1..4000 {
            let (x, y) = (
                (i as f64 * 0.618_033_9).fract() * t,
                (i as f64 * 0.414_213_5).fract() * t,
            );
            let a = herringbone(x, y, len, wid, t);
            assert!((0.0..len).contains(&a.along) && (0.0..wid).contains(&a.across));
            for (dx, dy) in [(t, 0.0), (0.0, t), (-t, 2.0 * t)] {
                let b = herringbone(x + dx, y + dy, len, wid, t);
                assert_eq!(a.key, b.key, "({x}, {y})");
                assert!((a.along - b.along).abs() < 1e-9 && (a.across - b.across).abs() < 1e-9);
            }
        }
    }

    const PAVER_TILE_TEST: (f64, f64, f64) = (0.2, 0.1, 5.0 * 0.2 * SQRT_2);
}

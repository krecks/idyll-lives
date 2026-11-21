use super::tiles::{herringbone, ridge, to_line, voronoi};
use super::*;

/// Oak parquet, 71 x 354 mm blocks in a 45° herringbone, lacquered: select oak, straight-grained, no knots,
/// light and near the oak's own colour so a covering's tint (natural, honey, smoked) gives the tone.
pub(super) fn parquet(n: &Noise, size: usize) -> Surface {
    let t = TILE_PARQUET;
    let (len, wid) = PARQUET_BLOCK;
    let px_m = t / size as f64;
    let ng = n.layer(1);
    paint(
        size,
        size,
        t,
        &[(0.0015, 1.5), (0.02, 0.2)],
        false,
        |u, v| {
            let b = herringbone(u * t, v * t, len, wid, t);
            let (i, j) = b.key;
            let r = |q: u32| n.rand(i, j, 190 + q);
            // Cut well away from the pith: the rings run along the block as near-straight stripes.
            let mut lg = log_for(n, i, j, wid, len, 0.0, (0.002, 0.004));
            lg.z0 += 0.05 + 0.1 * r(4);
            lg.slope *= 0.4;
            let (late, fibre, _) = grain_at(&ng, &lg, b.along, b.across, px_m);
            let sx = b.along + lg.salt;
            let streak = fbm_free(
                &ng.layer(4),
                sx * 3.0 + fbm_free(&ng.layer(5), sx * 4.0, b.across * 12.0, 2) * 0.6,
                b.across * 40.0,
                4,
            );
            let flame = smoothstep(-0.05, 0.2, streak);
            // Oak's pores, short dark dashes, and on a quartered block its pale rays.
            let pore = smoothstep(
                0.17,
                0.26,
                fbm_free(&ng.layer(3), b.along * 60.0 + lg.salt, b.across * 1400.0, 2)
                    + late * 0.08,
            );
            let ray = if r(5) < 0.3 {
                smoothstep(
                    0.14,
                    0.24,
                    fbm_free(&ng.layer(6), b.along * 22.0 + lg.salt, b.across * 140.0, 2),
                )
            } else {
                0.0
            };
            let edge = b.along.min(len - b.along).min(b.across).min(wid - b.across);
            let joint = smoothstep(0.0002, 0.0009, edge);
            let tone = r(0);
            let early = mix(rgb(212, 184, 146), rgb(194, 164, 126), tone);
            let latec = mix(rgb(176, 144, 108), rgb(160, 128, 94), tone);
            let mut c = mix(early, latec, late * 0.5 + flame * 0.3);
            // Each block its own shade, and the lacquer catches the two directions a shade apart.
            let sheen = if b.turned { 1.015 } else { 0.985 };
            c = mul(
                c,
                (0.93 + 0.12 * r(1))
                    * sheen
                    * (1.0
                        + 0.08 * fibre
                        + 0.05
                            * fbm_free(&ng.layer(2), b.along * 1.5 + lg.salt, b.across * 12.0, 3)),
            );
            c = mix(c, rgb(140, 110, 80), pore * 0.35);
            c = mix(c, rgb(224, 202, 166), ray * 0.4);
            let col = mix(rgb(110, 86, 62), c, smoothstep(0.0001, 0.0011, edge));
            // Sanded flat after laying, so the blocks are one plane; the odd one a hair proud.
            let lip = (r(2) - 0.5) * 0.05;
            let h = lerp(0.3, 0.9 + lip - 0.04 * pore + 0.01 * fibre, joint);
            let rough = lerp(0.7, 0.40 + 0.08 * pore + 0.04 * (r(3) - 0.5), joint);
            Px {
                h,
                c: col,
                r: rough,
            }
        },
    )
}

/// Cork tiles, 30 cm and butted tight: granules of 2 to 8 mm in their binder, each tile its own shade, sealed
/// satin. Warm tan; a covering's tint lightens or darkens it.
pub(super) fn cork(n: &Noise, size: usize) -> Surface {
    let t = TILE_CORK;
    let k = CORK_TILES;
    let s = t / k as f64;
    let ng = n.layer(1);
    paint(
        size,
        size,
        t,
        &[(0.001, 1.5), (0.006, 0.4)],
        false,
        |u, v| {
            let (ix, iy) = ((u * k as f64).floor() as i64, (v * k as f64).floor() as i64);
            let (wi, wj) = (ix.rem_euclid(k), iy.rem_euclid(k));
            let r = |q: u32| n.rand(wi, wj, 200 + q);
            let x = (u * k as f64 - ix as f64) * s;
            let y = (v * k as f64 - iy as f64) * s;
            let g = CORK_HALF_JOINT;
            let d = (x - g).min(s - g - x).min((y - g).min(s - g - y));
            let face = smoothstep(0.0, 0.0006, d);
            // Granules at two sizes: each cell a granule of its own tone, the gaps between them darker binder.
            let fine = voronoi(&ng, u, v, 300, 300, 11);
            let big = voronoi(&ng.layer(1), u, v, 120, 120, 13);
            let gr = |c: (i64, i64), q: u32, salt: u32| ng.rand(c.0, c.1, salt + q);
            let use_big = gr(big.cell, 0, 40) < 0.35 && big.f2 - big.f1 > 0.10;
            let (cell, gap, salt) = if use_big {
                (big.cell, big.f2 - big.f1, 40)
            } else {
                (fine.cell, fine.f2 - fine.f1, 60)
            };
            let gt = gr(cell, 1, salt);
            let binder = 1.0 - smoothstep(0.02, 0.12, gap);
            let shade = 1.0 + (r(0) - 0.5) * 0.04;
            // Granules from pale tan to brown, the odd one dark (the bark's back), each mottled within.
            let mut c = mix(rgb(214, 174, 126), rgb(150, 104, 66), gt.powf(1.4));
            if gr(cell, 2, salt) < 0.025 {
                c = rgb(104, 68, 44);
            }
            let inner = fbm_free(
                &ng.layer(4),
                u * 300.0 + cell.0 as f64,
                v * 300.0 + cell.1 as f64,
                2,
            );
            c = mul(
                c,
                shade * (1.0 + 0.12 * inner + 0.05 * fbm(&ng.layer(2), u, v, 24, 24, 3)),
            );
            c = mix(c, rgb(112, 76, 50), binder * 0.45);
            // Pores in the granules.
            let (pore, _) = grains(&ng.layer(3), u, v, 600, 0.08, 0.4, 101);
            c = mix(c, rgb(130, 90, 60), pore * 0.3);
            let joint = rgb(120, 86, 60);
            // The butt joint shows as a fine dark line, the cut edges a touch crushed.
            let near = 1.0 - smoothstep(0.0, 0.0025, d);
            let c = mix(c, mul(c, 0.8), near * 0.6);
            let h = lerp(
                0.3,
                0.85 - 0.12 * binder - 0.15 * pore + 0.05 * (gt - 0.5) + 0.04 * (r(1) - 0.5),
                face,
            );
            let rough = lerp(0.8, 0.42 + 0.12 * binder + 0.2 * pore, face);
            Px {
                h,
                c: mix(joint, c, face),
                r: rough,
            }
        },
    )
}

/// Sheet linoleum, softly marbled along the roll, near white for a covering to tint; one welded seam
/// across the 2 m tile, the roll's width.
pub(super) fn lino(n: &Noise, size: usize) -> Surface {
    let t = TILE_LINO;
    let nm = n.layer(1);
    paint(size, size, t, &[(0.001, 1.0)], false, |u, v| {
        // Marbling: streaks drawn out along the roll (v), warped.
        let wx = nm.fbm2(u, v, 6, 2, 4) - 0.5;
        let wy = nm.layer(1).fbm2(u, v, 3, 1, 3) - 0.5;
        let (mu, mv) = (
            (u + wx * 0.05).rem_euclid(1.0),
            (v + wy * 0.3).rem_euclid(1.0),
        );
        let streak = nm.layer(2).fbm2(mu, mv, 14, 2, 5) - 0.5;
        let fine = nm.layer(3).fbm2(mu, mv, 60, 4, 4) - 0.5;
        let mottle = nm.layer(4).fbm2(u, v, 40, 40, 3) - 0.5;
        let (fleck, ft) = grains(&nm.layer(5), u, v, 500, 0.25, 0.4, 121);
        let mut l = 0.86 + 0.05 * streak + 0.02 * fine + 0.02 * mottle + 0.03 * fleck * (ft - 0.5);
        l -= 0.05 * ridge(streak, 0.03);
        l += 0.03 * ridge(fine, 0.02);
        // The seam, hot-welded: a line a touch darker, barely there.
        let seam = 1.0 - smoothstep(0.0003, 0.0009, to_line(u * t, t));
        l -= 0.04 * seam;
        let emboss = fbm(&nm.layer(6), u, v, 160, 160, 2);
        let h = 0.6 + 0.08 * emboss + 0.02 * streak - 0.3 * seam;
        Px {
            h,
            c: grey(l.min(0.92)),
            r: 0.46 + 0.04 * emboss + 0.03 * streak + 0.1 * seam,
        }
    })
}

/// Polished grey concrete: ground to a sheen, its fine aggregate showing as flecks, the power trowel's
/// clouds, and saw-cut control joints every 1.2 m.
pub(super) fn concrete(n: &Noise, size: usize) -> Surface {
    let t = TILE_CONCRETE;
    let nm = n.layer(1);
    paint(
        size,
        size,
        t,
        &[(0.002, 1.8), (0.02, 0.2)],
        false,
        |u, v| {
            let (x, y) = (u * t, v * t);
            let paste = fbm(&nm, u, v, 120, 120, 3);
            let clouds =
                fbm(&nm.layer(1), u, v, 3, 3, 5) + 0.6 * fbm(&nm.layer(9), u, v, 12, 12, 3);
            // Trowel swirls: a warped field's level lines, faint arcs.
            let (wu, wv) = nm.layer(2).warp(u, v, 0.4, 3);
            let swirl = nm.layer(3).fbm2(wu, wv, 10, 10, 3) - 0.5;
            let trowel =
                ridge(swirl, 0.04) * smoothstep(-0.1, 0.15, fbm(&nm.layer(4), u, v, 6, 6, 3));
            let mut c = mul(rgb(150, 148, 144), 1.0 + 0.26 * clouds + 0.06 * paste);
            c = mul(c, 1.0 - 0.08 * trowel);
            // Aggregate cut by the grinding: grey, dark and buff stones, 1 to 6 mm.
            let (agg, at) = grains(&nm.layer(5), u, v, 700, 0.22, 0.5, 111);
            let (big, bt) = grains(&nm.layer(6), u, v, 300, 0.12, 0.55, 113);
            let stone = |tone: f64| match tone {
                x if x < 0.35 => rgb(96, 94, 92),
                x if x < 0.7 => rgb(184, 182, 176),
                x if x < 0.85 => rgb(170, 152, 128),
                _ => rgb(70, 68, 68),
            };
            c = mix(c, stone(at), agg * 0.45);
            c = mix(c, stone(bt), big * 0.6);
            let (pin, _) = grains(&nm.layer(7), u, v, 900, 0.04, 0.3, 117);
            c = mix(c, rgb(74, 72, 70), pin * 0.7);
            // Saw cuts, 3 mm, a little spalled.
            let ragged = fbm(&nm.layer(8), u, v, 300, 300, 2) * 0.0008;
            let dc = to_line(x, CONCRETE_JOINTS).min(to_line(y, CONCRETE_JOINTS))
                - CONCRETE_HALF_CUT
                + ragged.min(0.0);
            let cut = 1.0 - smoothstep(0.0, 0.0006, dc);
            let lip = 1.0 - smoothstep(0.0, 0.004, dc);
            c = mix(c, mul(c, 0.85), lip * 0.5);
            c = mix(c, rgb(58, 57, 55), cut);
            let h = 0.8 + 0.01 * paste + 0.02 * clouds - 0.3 * pin - 0.08 * lip - 0.6 * cut;
            let rough = (0.32 + 0.06 * clouds - 0.05 * trowel
                + 0.06 * paste
                + 0.4 * pin
                + 0.55 * cut
                + 0.1 * lip)
                .clamp(0.1, 1.0);
            Px { h, c, r: rough }
        },
    )
}

/// Sisal matting in a basketweave: blocks of four twisted strands (25 mm), over and under by turns, natural
/// straw with darker fibres. A covering's tint shades it.
pub(super) fn sisal(n: &Noise, size: usize) -> Surface {
    let t = TILE_SISAL;
    let k = SISAL_BLOCKS;
    let s = t / k as f64;
    let strands = SISAL_STRANDS;
    let sw = s / strands as f64;
    let nf = n.layer(1);
    paint(
        size,
        size,
        t,
        &[(0.002, 1.2), (0.012, 0.3)],
        false,
        |u, v| {
            let (ix, iy) = ((u * k as f64).floor() as i64, (v * k as f64).floor() as i64);
            let x = (u * k as f64 - ix as f64) * s;
            let y = (v * k as f64 - iy as f64) * s;
            let across_x = (ix + iy).rem_euclid(2) == 0;
            // In this block the strands run along x or along y; a strand is one length of yarn through the row.
            let (along, across, line) = if across_x {
                (x + ix as f64 * s, y, (iy.rem_euclid(k), 0))
            } else {
                (y + iy as f64 * s, x, (ix.rem_euclid(k), 1))
            };
            let fa = if across_x { x / s } else { y / s };
            let si = ((across / sw).floor() as i64).clamp(0, strands - 1);
            let a = (across - si as f64 * sw) / sw;
            let strand = (line.0 * strands + si, line.1);
            let r = |q: u32| n.rand(strand.0, strand.1, 220 + q);
            // A strand's round section, and its dip where it passes under the next block.
            let round = (std::f64::consts::PI * a).sin();
            let dip = smoothstep(0.0, 0.2, fa) * smoothstep(0.0, 0.2, 1.0 - fa);
            // The twist: fibres lie on a helix, light and dark bands on the slant.
            let twist = ((along / 0.006 + a * 0.8 + r(0) * 7.0) * std::f64::consts::TAU).sin();
            // Along the yarn its colour wanders, as a plant's fibres do.
            let wander = fbm_free(
                &nf.layer(2),
                along * 6.0 + r(1) * 50.0,
                strand.0 as f64 * 0.37 + strand.1 as f64 * 11.0,
                3,
            );
            let h = (0.3 + 0.7 * dip) * (0.4 + 0.6 * round) + 0.03 * twist * round;
            let base = mix(rgb(228, 206, 162), rgb(210, 184, 136), r(2));
            // The blocks catch the light a shade apart by their strands' direction.
            let facing = if across_x { 1.03 } else { 0.97 };
            let mut c = mul(
                base,
                facing * (1.0 + 0.05 * twist + 0.22 * wander + (r(3) - 0.5) * 0.06),
            );
            // Darker fibres here and there, and the shade in the weave's gaps.
            let dark = smoothstep(
                0.2,
                0.3,
                fbm_free(
                    &nf.layer(1),
                    along * 30.0 + r(5) * 30.0,
                    a * 2.0 + strand.0 as f64,
                    2,
                ),
            );
            c = mix(c, rgb(150, 124, 84), dark * 0.45);
            c = mul(c, 0.68 + 0.32 * (round * dip).sqrt());
            let rough = 0.86 + 0.06 * (1.0 - dip);
            Px { h, c, r: rough }
        },
    )
}

//! The garden paths' and drives' own surfaces (`material.path.*`, `game/town.ts` `pathLooks`): broom-finished
//! concrete, riven flagstones and pea gravel. The concrete has no joints of its own: the path's geometry draws them
//! square to its run (`render/shared/paving.ts`), where a texture laid by world position could not.

use super::tiles::voronoi;
use super::*;

/// Broom-finished concrete, poured: the broom's fine ridges, sand and the odd stone in the paste, weathered
/// darker in clouds, a few hairline cracks.
pub(super) fn walk_concrete(n: &Noise, size: usize) -> Surface {
    let t = TILE_WALK;
    let k = size as f64 / 1024.0;
    let (cd, ch) = cracks(
        &n.layer(9),
        size,
        size,
        2,
        size as f64 * 0.1,
        k * 0.7,
        k * 2.5,
    );
    let nm = n.layer(1);
    let w = size;
    paint(size, size, t, &[(0.003, 1.2), (0.03, 0.3)], move |u, v| {
        let i = ((v * w as f64) as usize).min(w - 1) * w + ((u * w as f64) as usize).min(w - 1);
        let (crack, halo) = (cd[i], ch[i]);
        let clouds = fbm(&nm, u, v, 3, 3, 5) + 0.5 * fbm(&nm.layer(1), u, v, 10, 10, 3);
        // The broom: streaks along u, a few millimetres apart, wavering.
        let broom = fbm(
            &nm.layer(3),
            u,
            v + fbm(&nm.layer(2), u, v, 4, 4, 2) * 0.004,
            6,
            640,
            2,
        );
        let paste = fbm(&nm.layer(4), u, v, 300, 300, 2);
        let mut c = mul(
            rgb(156, 152, 144),
            1.0 + 0.22 * clouds + 0.06 * paste + 0.06 * broom,
        );
        // Sand and fine stones the broom brought up: grey, buff and dark.
        let (sand, st) = grains(&nm.layer(5), u, v, 560, 0.3, 0.45, 131);
        let stone = |tone: f64| match tone {
            x if x < 0.4 => rgb(132, 128, 122),
            x if x < 0.75 => rgb(196, 190, 178),
            x if x < 0.9 => rgb(170, 154, 130),
            _ => rgb(96, 92, 88),
        };
        c = mix(c, stone(st), sand * 0.35);
        // Weathering: grime in broad patches, a pale bloom here and there, lichen dots.
        let grime = smoothstep(0.02, 0.22, fbm(&nm.layer(6), u, v, 5, 5, 4));
        c = mix(c, mul(c, 0.84), grime * 0.55);
        let bloom = smoothstep(0.1, 0.25, fbm(&nm.layer(7), u, v, 7, 7, 3));
        c = mix(c, rgb(176, 174, 168), bloom * 0.2);
        let (lichen, lt) = grains(&nm.layer(8), u, v, 90, 0.06, 0.35, 137);
        c = mix(
            c,
            if lt < 0.6 {
                rgb(176, 178, 160)
            } else {
                rgb(150, 146, 120)
            },
            lichen * 0.45,
        );
        c = mix(c, rgb(120, 118, 112), halo * 0.15);
        c = mix(c, rgb(96, 94, 90), crack * 0.4);
        let h = 0.6 + 0.08 * broom + 0.03 * paste + 0.05 * sand - 0.5 * crack;
        let rough = (0.86 + 0.05 * clouds - 0.04 * bloom + 0.1 * crack).clamp(0.6, 1.0);
        Px { h, c, r: rough }
    })
}

/// Riven flagstones, random (about 40 to 60 cm), set a centimetre or two apart in a dirty joint with moss in
/// it: sandstone buff, grey, warm and blue-grey stones, each split along its bedding.
pub(super) fn flagstone(n: &Noise, size: usize) -> Surface {
    let t = TILE_FLAGSTONE;
    let k = FLAGSTONE_CELLS;
    let cell = t / k as f64;
    let nw = n.layer(1);
    let ns = n.layer(2);
    let nj = n.layer(3);
    // (Yorkstone's buffs and greys: the brighter colours of a mixed batch read as painted.)
    let palette = [
        rgb(174, 164, 142),
        rgb(158, 154, 144),
        rgb(168, 152, 128),
        rgb(146, 146, 140),
        rgb(162, 150, 136),
        rgb(150, 142, 124),
    ];
    paint(size, size, t, &[(0.006, 1.4), (0.04, 0.5)], move |u, v| {
        // A little warp, so the stones' sides are not ruled.
        let wu = u + fbm(&nw, u, v, 12, 12, 3) * 0.02 / t;
        let wv = v + fbm(&nw.layer(1), u, v, 12, 12, 3) * 0.02 / t;
        let vor = voronoi(n, wu, wv, k, k, 151);
        let (ci, cj) = vor.cell;
        let r = |q: u32| n.rand(ci, cj, 160 + q);
        // How far in from its side (m), less the joint, broken at the arrises.
        let inside = (vor.f2 - vor.f1) * 0.5 * cell;
        let joint = 0.006 + 0.008 * r(0);
        let chips = fbm(&ns.layer(4), u, v, 60, 60, 3).abs() * 0.008;
        let d = inside - joint - chips;
        let face = smoothstep(0.0, 0.002, d);
        let arris = smoothstep(0.0, 0.012, d);
        // The stone: its bedding (a direction of its own), riven ledges, clouds of colour.
        let ang = r(1) * std::f64::consts::PI;
        let (ca, sa) = (ang.cos(), ang.sin());
        // (In the stone's own metres: the tile's, from u and v, would jump at its wrap inside a stone.)
        let (x, y) = (
            vor.off.0 * cell + r(2) * 20.0,
            vor.off.1 * cell + r(3) * 20.0,
        );
        let (lx, ly) = (x * ca + y * sa, -x * sa + y * ca);
        let bed = fbm_free(&ns, lx * 3.0, ly * 18.0, 4);
        let riven = fbm_free(&ns.layer(1), lx * 6.0, ly * 6.0, 4);
        let ledge = (bed * 5.0).floor() / 5.0;
        let base = palette[(r(4) * palette.len() as f64) as usize % palette.len()];
        let clouds = fbm(&ns.layer(5), u, v, 6, 6, 4);
        let mut c = mul(
            base,
            1.0 + 0.22 * bed + 0.2 * riven + 0.2 * clouds + (r(5) - 0.5) * 0.12,
        );
        let iron = smoothstep(0.1, 0.3, fbm_free(&ns.layer(2), lx * 2.0, ly * 2.0, 3));
        c = mix(c, rgb(150, 120, 92), iron * 0.25 * r(6));
        // Weathered darker and greener in patches, as stone that lies out in the rain.
        let weather = smoothstep(0.0, 0.25, fbm(&ns.layer(6), u, v, 9, 9, 4));
        c = mix(c, mul(mix(c, rgb(110, 116, 96), 0.3), 0.8), weather * 0.5);
        let (grit, _) = grains(&ns.layer(3), u, v, 500, 0.3, 0.4, 167);
        c = mul(c, 1.0 - 0.06 * grit);
        // Worn paler where feet go over it, dirt and green at its rim.
        c = mix(c, mul(c, 0.8), (1.0 - arris) * 0.6);
        // The joint: earth and grit, moss in some of it.
        let moss = smoothstep(-0.05, 0.15, fbm(&nj, u, v, 14, 14, 3));
        let earth = mul(
            rgb(92, 82, 68),
            1.0 + 0.5 * fbm(&nj.layer(1), u, v, 200, 200, 2),
        );
        let gap = mix(earth, rgb(78, 94, 50), moss * 0.7);
        let col = mix(gap, c, face);
        let tilt = (lx - r(7) * 20.0) * (r(8) - 0.5) * 0.02;
        let h = lerp(
            0.15 + 0.1 * moss,
            0.75 + 0.08 * ledge + 0.05 * riven + tilt + 0.06 * arris,
            face,
        );
        let rough = lerp(1.0, 0.78 + 0.08 * riven + 0.06 * grit, face);
        Px {
            h,
            c: col,
            r: rough,
        }
    })
}

/// Pea gravel, raked: rounded stones of 6 to 14 mm, buff, grey, white and rust, two layers over each other with
/// dark gaps between.
pub(super) fn gravel(n: &Noise, size: usize) -> Surface {
    let t = TILE_GRAVEL;
    let ng = n.layer(1);
    // (Close tones: a gravel of whites and darks side by side glitters as the camera moves.)
    let palette = [
        rgb(184, 172, 150),
        rgb(168, 154, 132),
        rgb(152, 142, 126),
        rgb(192, 186, 172),
        rgb(160, 132, 106),
        rgb(140, 136, 128),
        rgb(176, 160, 134),
    ];
    paint(size, size, t, &[(0.004, 1.6), (0.02, 0.4)], move |u, v| {
        // The stones of each layer: a dome over each cell's point (radius a share of the cell), the higher wins.
        let broad = fbm(&ng.layer(7), u, v, 4, 4, 4);
        let mut best = (0.0f64, [0.0; 3], 0.0f64);
        for (layer, cells, lift) in [(0u32, GRAVEL_CELLS, 0.0), (1, GRAVEL_CELLS + 19, 0.25)] {
            let w = worley(
                &ng.layer(u64::from(layer)),
                u,
                v,
                cells,
                cells,
                0.9,
                171 + layer * 10,
            );
            let (ci, cj) = w.cell;
            let r = |q: u32| n.rand(ci, cj, 180 + layer * 10 + q);
            let rad = 0.42 + 0.2 * r(0);
            if w.f1 >= rad {
                continue;
            }
            let dome = (1.0 - (w.f1 / rad).powi(2)).sqrt();
            let h = lift + dome * (0.6 + 0.3 * r(1));
            if h > best.0 {
                let base = palette[(r(2) * palette.len() as f64) as usize % palette.len()];
                let speck = fbm(&ng.layer(5), u, v, 700, 700, 1);
                best = (
                    h,
                    mul(
                        base,
                        0.9 + 0.14 * r(3) + 0.1 * dome + 0.06 * speck + 0.12 * broad,
                    ),
                    r(4),
                );
            }
        }
        let (h, c, gloss) = best;
        // Between the stones: the dark of the gaps, a little dust.
        let dust = fbm(&ng.layer(6), u, v, 8, 8, 3);
        let gap = mul(rgb(108, 98, 84), 1.0 + 0.3 * dust);
        let on = smoothstep(0.0, 0.15, h);
        let col = mix(gap, mul(c, 1.0 + 0.08 * dust), on);
        Px {
            h,
            c: col,
            r: lerp(1.0, 0.72 + 0.12 * gloss, on),
        }
    })
}

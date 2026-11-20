//! The leaves: their shapes, veins and colours.

use super::*;

// ---- the leaves -------------------------------------------------------------------------------------------------

/// The dense wall's share of its tile, across and up.
pub(super) const DENSE: f64 = 0.8;
/// A cluster keeps within this of its tile's middle, of half the tile: no leaf crosses into the next tile, nor
/// under a card's edge.
pub(super) const REACH: f64 = 0.925;
/// The covered texels' mean luminance (linear) in every cluster's tile, and every texel's in a dense one: the first
/// atlas's, which the trees' card and core tones were matched to their far versions with.
pub(super) const CLUSTER_LIGHT: f32 = 0.44;
pub(super) const DENSE_LIGHT: f32 = 0.335;

/// A matte leaf colour: nearly grey (`value`), a little yellow (`warm` > 0) or blue-green (< 0).
pub(super) fn tone(value: f64, warm: f64) -> Tone {
    [
        value * (1.0 + 0.10 * warm),
        value,
        value * (1.0 - 0.16 * warm - 0.04),
    ]
    .map(|c| c.min(1.0) as f32)
}

/// A leaf's colour `depth` (0..1) down in its cluster: three depths, the top the lightest, five tones each.
pub(super) fn leaf_tone(rng: &mut Random, depth: f64) -> Tone {
    const TONES: [(f64, f64); 5] = [
        (0.8, 0.0),
        (0.7, 0.35),
        (0.74, -0.3),
        (0.86, 0.5),
        (0.66, -0.6),
    ];
    let k = [1.0, 0.76, 0.56][2.min((depth * 3.0) as usize)];
    let (value, warm) = TONES[rng.randrange(TONES.len() as i64) as usize];
    tone(value * k, warm)
}

/// How a blade is held: its tilt up (degrees), how it bends down along its length, how its halves lift into a V,
/// its turn about its midrib by the tip (degrees either way), and its gloss.
#[derive(Clone, Copy)]
pub(super) struct Hold {
    tilt: (f64, f64),
    droop: (f64, f64),
    fold: (f64, f64),
    twist: f64,
    gloss: f32,
}

#[allow(clippy::too_many_arguments)]
/// A blade cut from `edge` (its right half, `cut_leaf`'s), heading `heading` (radians) from `at`.
pub(super) fn blade(
    sketch: &mut Sketch,
    rng: &mut Random,
    edge: &[(f64, f64)],
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    tone: Tone,
    hold: Hold,
) {
    let tilt = rng.uniform(hold.tilt.0, hold.tilt.1);
    let droop = rng.uniform(hold.droop.0, hold.droop.1);
    let fold = rng.uniform(hold.fold.0, hold.fold.1);
    let twist = rng.uniform(-hold.twist, hold.twist);
    let o = CutLeaf {
        droop,
        fold,
        thick: 0.004_f64.min(length * 0.03),
        ribs: 5,
        twist,
        left: None,
    };
    sketch.add(
        cut_leaf(
            "leaf",
            at,
            90.0 - degrees(heading),
            tilt,
            length,
            edge,
            BLANK,
            o,
        ),
        tone,
        hold.gloss,
    );
}

/// A species' leaf: (sketch, random numbers, where, heading, length, how deep in its cluster 0..1).
pub(super) type Leaf = fn(&mut Sketch, &mut Random, (f64, f64, f64), f64, f64, f64);

/// An English oak's: lobed, four or five rounded lobes a side, widest above the middle.
pub(super) fn oak_leaf(
    sketch: &mut Sketch,
    rng: &mut Random,
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    depth: f64,
) {
    let width = length * rng.uniform(0.56, 0.66);
    let lobes = sinuate(
        rng.uniform(4.0, 5.0),
        Sinuate {
            depth: rng.uniform(0.4, 0.52),
        },
    );
    let edge = outline(
        lobes,
        width,
        OutlineOf {
            n: 30,
            ..OutlineOf::default()
        },
    );
    let tone = leaf_tone(rng, depth);
    let hold = Hold {
        tilt: (-6.0, 22.0),
        droop: (0.05, 0.3),
        fold: (0.15, 0.35),
        twist: 25.0,
        gloss: 0.0,
    };
    blade(sketch, rng, &edge, at, heading, length, tone, hold);
}

/// A Norway maple's: five pointed lobes with a tooth to each side, as wide as long, held flat.
pub(super) fn maple_leaf(
    sketch: &mut Sketch,
    rng: &mut Random,
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    depth: f64,
) {
    let o = Palmate {
        cut: rng.uniform(0.36, 0.44),
        bulge: 0.075,
        teeth: 1,
        spread: rng.uniform(98.0, 108.0),
        fall: 0.84,
    };
    let edge = palmate(5, length, o);
    let tone = leaf_tone(rng, depth);
    let hold = Hold {
        tilt: (-4.0, 14.0),
        droop: (0.05, 0.22),
        fold: (0.06, 0.18),
        twist: 12.0,
        gloss: 0.0,
    };
    blade(sketch, rng, &edge, at, heading, length, tone, hold);
}

/// A silver birch's: small, triangular with a long point, toothed.
pub(super) fn birch_leaf(
    sketch: &mut Sketch,
    rng: &mut Random,
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    depth: f64,
) {
    let width = length * rng.uniform(0.72, 0.84);
    let edge = outline(
        "deltoid",
        width,
        OutlineOf {
            n: 18,
            teeth: 0.2,
            ..OutlineOf::default()
        },
    );
    let tone = leaf_tone(rng, depth);
    let hold = Hold {
        tilt: (-10.0, 25.0),
        droop: (0.05, 0.3),
        fold: (0.2, 0.45),
        twist: 35.0,
        gloss: 0.0,
    };
    blade(sketch, rng, &edge, at, heading, length, tone, hold);
}

/// An apple's: ovate, finely toothed, its halves folded up.
pub(super) fn apple_leaf(
    sketch: &mut Sketch,
    rng: &mut Random,
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    depth: f64,
) {
    let width = length * rng.uniform(0.58, 0.68);
    let edge = outline(
        "ovate",
        width,
        OutlineOf {
            n: 20,
            teeth: 0.1,
            ..OutlineOf::default()
        },
    );
    let tone = leaf_tone(rng, depth);
    let hold = Hold {
        tilt: (-6.0, 24.0),
        droop: (0.1, 0.35),
        fold: (0.3, 0.55),
        twist: 25.0,
        gloss: 0.0,
    };
    blade(sketch, rng, &edge, at, heading, length, tone, hold);
}

/// A Japanese maple's: seven narrow pointed lobes cut nearly to the stalk.
pub(super) fn japanese_maple_leaf(
    sketch: &mut Sketch,
    rng: &mut Random,
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    depth: f64,
) {
    let o = Palmate {
        cut: rng.uniform(0.7, 0.78),
        bulge: 0.055,
        teeth: 0,
        spread: rng.uniform(118.0, 130.0),
        fall: 0.87,
    };
    let edge = palmate(7, length, o);
    let tone = leaf_tone(rng, depth);
    let hold = Hold {
        tilt: (-4.0, 12.0),
        droop: (0.05, 0.25),
        fold: (0.04, 0.14),
        twist: 10.0,
        gloss: 0.0,
    };
    blade(sketch, rng, &edge, at, heading, length, tone, hold);
}

/// A saucer magnolia's: large, widest above the middle, a wedge to the stalk.
pub(super) fn magnolia_leaf(
    sketch: &mut Sketch,
    rng: &mut Random,
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    depth: f64,
) {
    let width = length * rng.uniform(0.48, 0.58);
    let wave = rng.uniform(0.03, 0.07);
    let waves = rng.choice(&[3.0, 5.0]);
    let edge = outline(
        "obovate",
        width,
        OutlineOf {
            n: 14,
            wave,
            waves,
            teeth: 0.0,
        },
    );
    let tone = leaf_tone(rng, depth);
    let hold = Hold {
        tilt: (-6.0, 20.0),
        droop: (0.1, 0.35),
        fold: (0.2, 0.4),
        twist: 20.0,
        gloss: 0.12,
    };
    blade(sketch, rng, &edge, at, heading, length, tone, hold);
}

/// A weeping willow's: long and narrow.
pub(super) fn willow_leaf(
    sketch: &mut Sketch,
    rng: &mut Random,
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    depth: f64,
) {
    // (a little broader than a willow's, a fifth of its length for a real one's eighth: narrower, a leaf is under a
    // pixel wide at the play camera)
    let width = length * rng.uniform(0.2, 0.25);
    let edge = outline(
        "lance",
        width,
        OutlineOf {
            n: 10,
            ..OutlineOf::default()
        },
    );
    let tone = leaf_tone(rng, depth);
    let hold = Hold {
        tilt: (-8.0, 20.0),
        droop: (0.1, 0.5),
        fold: (0.2, 0.4),
        twist: 40.0,
        gloss: 0.0,
    };
    blade(sketch, rng, &edge, at, heading, length, tone, hold);
}

/// A lemon's: elliptic, pointed, glossy.
pub(super) fn lemon_leaf(
    sketch: &mut Sketch,
    rng: &mut Random,
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    depth: f64,
) {
    let width = length * rng.uniform(0.42, 0.5);
    let edge = outline(
        "elliptic",
        width,
        OutlineOf {
            n: 14,
            wave: 0.03,
            waves: 3.0,
            teeth: 0.0,
        },
    );
    let tone = leaf_tone(rng, depth);
    let hold = Hold {
        tilt: (-6.0, 24.0),
        droop: (0.05, 0.25),
        fold: (0.25, 0.45),
        twist: 22.0,
        gloss: 0.45,
    };
    blade(sketch, rng, &edge, at, heading, length, tone, hold);
}

/// A flowering cherry's young leaf among its blossom: ovate with a long point, folded, bronze. (Its colour is the
/// bronze over the petals' pink: the card's pink tints the whole tile, the petals pale in it.)
pub(super) fn cherry_leaf(
    sketch: &mut Sketch,
    rng: &mut Random,
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    depth: f64,
) {
    let width = length * rng.uniform(0.42, 0.5);
    let edge = outline(
        "ovate",
        width,
        OutlineOf {
            n: 20,
            teeth: 0.1,
            ..OutlineOf::default()
        },
    );
    let k = (0.9 - 0.3 * depth) as f32 * rng.uniform(0.85, 1.0) as f32;
    let hold = Hold {
        tilt: (0.0, 28.0),
        droop: (0.15, 0.45),
        fold: (0.35, 0.6),
        twist: 25.0,
        gloss: 0.15,
    };
    blade(
        sketch,
        rng,
        &edge,
        at,
        heading,
        length,
        [0.42 * k, 0.5 * k, 0.13 * k],
        hold,
    );
}

/// A double cherry blossom `size` across at `at`: three rings of notched petals, the outer ones lying open, the
/// inner standing, paler outside and a deeper pink at its heart.
pub(super) fn flower(
    sketch: &mut Sketch,
    rng: &mut Random,
    at: (f64, f64, f64),
    size: f64,
    depth: f64,
) {
    let turn = rng.uniform(0.0, TAU);
    // (one flower paler, the next deeper: a mass of them is not one pattern)
    let k = (1.0 - 0.2 * depth) as f32 * rng.uniform(0.86, 1.0) as f32;
    for (ring, (petals, long, tilt, pink)) in [
        (9, 0.5, (8.0, 26.0), 0.0),
        (7, 0.4, (34.0, 54.0), 0.5),
        (5, 0.28, (58.0, 78.0), 1.0),
    ]
    .into_iter()
    .enumerate()
    {
        for i in 0..petals {
            let heading = turn
                + TAU * (i as f64 + 0.5 * ring as f64 + rng.uniform(-0.2, 0.2)) / petals as f64;
            let length = size * long * rng.uniform(0.9, 1.1);
            let wave = rng.uniform(0.1, 0.2);
            let edge = outline(
                "petal",
                length * 0.95,
                OutlineOf {
                    n: 8,
                    wave,
                    waves: 3.0,
                    teeth: 0.0,
                },
            );
            let pale = rng.uniform(0.9, 0.98) as f32 * k;
            let tone = [pale, pale * (1.0 - 0.14 * pink), pale * (1.0 - 0.08 * pink)];
            let hold = Hold {
                tilt,
                droop: (0.0, 0.2),
                fold: (0.1, 0.3),
                twist: 20.0,
                gloss: 0.0,
            };
            blade(
                sketch,
                rng,
                &edge,
                (at.0, at.1, at.2 + 0.004 * ring as f64),
                heading,
                length,
                tone,
                hold,
            );
        }
    }
}

/// The longest a leaf from `p` heading `heading` may be (up to `length`) and stay within `reach` (across, up)
/// of the tile's middle, its tip and both its shoulders (`wide` of its length across).
pub(super) fn fits(
    p: (f64, f64),
    heading: f64,
    mut length: f64,
    wide: f64,
    reach: (f64, f64),
) -> f64 {
    let (dx, dy) = (cos(heading), sin(heading));
    for _ in 0..14 {
        let inside = [
            (1.0, 0.0),
            (0.55, wide / 2.0),
            (0.55, -wide / 2.0),
            (0.0, wide / 3.0),
            (0.0, -wide / 3.0),
        ]
        .iter()
        .all(|(along, side)| {
            let (x, y) = (
                p.0 + (dx * along - dy * side) * length,
                p.1 + (dy * along + dx * side) * length,
            );
            x.abs() < reach.0 && y.abs() < reach.1
        });
        if inside {
            return length;
        }
        length *= 0.88;
    }
    0.0
}

/// How a species' leaves stand in a cluster: on twigs fanning from the tile's middle.
#[derive(Clone, Copy)]
pub(super) struct Spray {
    pub(super) leaf: Leaf,
    /// The tile, metres (the species' cards are about as wide).
    pub(super) across: f64,
    /// A leaf's length, metres, and its width of that (for keeping it in the tile).
    pub(super) size: (f64, f64),
    pub(super) wide: f64,
    /// Twigs, how far the first is turned, and each one's length, of the reach.
    pub(super) twigs: usize,
    pub(super) turn: f64,
    pub(super) long: (f64, f64),
    /// A twig's radius at its foot and its tip, metres, and its colour.
    pub(super) wood: (f64, f64, Tone),
    /// Leaves along each twig (from `from` of the way out), in pairs when `opposite`, standing `lean` radians
    /// off the twig at its foot and at its tip; `tip` more fanned round its end; `fill` lying under them all.
    pub(super) per: usize,
    pub(super) from: f64,
    pub(super) opposite: bool,
    pub(super) lean: (f64, f64),
    pub(super) tip: usize,
    pub(super) fill: usize,
    /// How far out the leaves lying under lie, of the reach.
    pub(super) fill_out: f64,
    /// Leaves laid over the twigs' foot last.
    pub(super) top: usize,
    /// How wide the twigs fan (radians: all round, or to one side as a hanging spray's) and how far back from the
    /// tile's middle their foot is, of the reach.
    pub(super) fan: f64,
    pub(super) foot: f64,
}

/// A cluster in a tile: the leaves lying under it first, then each twig's, the later laid over the earlier.
pub(super) fn spray(sketch: &mut Sketch, rng: &mut Random, s: Spray) {
    let reach = s.across / 2.0 * REACH;
    let total = s.fill + s.top + s.twigs * (s.per * if s.opposite { 2 } else { 1 } + s.tip);
    let mut n = 0;
    let mut add =
        |sketch: &mut Sketch, rng: &mut Random, p: (f64, f64), heading: f64, size: f64| {
            let size = fits(p, heading, size, s.wide, (reach, reach));
            if size >= s.size.0 * 0.5 {
                let at = (p.0, p.1, 0.012 + 0.22 * s.across * n as f64 / total as f64);
                (s.leaf)(
                    sketch,
                    rng,
                    at,
                    heading,
                    size,
                    1.0 - n as f64 / total as f64,
                );
            }
            n += 1;
        };
    for _ in 0..s.fill {
        let a = rng.uniform(0.0, TAU);
        let r = reach * rng.uniform(0.0, s.fill_out);
        let heading = a + rng.uniform(-1.2, 1.2);
        let size = rng.uniform(s.size.0, s.size.1);
        add(sketch, rng, (r * cos(a), r * sin(a)), heading, size);
    }
    // the twigs' foot: the tile's middle, or back from it for a spray that fans to one side
    let (fx, fy) = (-s.foot * reach * cos(s.turn), -s.foot * reach * sin(s.turn));
    for t in 0..s.twigs {
        let a = if s.fan >= TAU {
            s.turn + TAU * (t as f64 + rng.uniform(-0.25, 0.25)) / s.twigs as f64
        } else {
            s.turn + s.fan * ((t as f64 + rng.uniform(0.25, 0.75)) / s.twigs as f64 - 0.5)
        };
        // (as long as it may be and end in the tile)
        let mut length = reach * rng.uniform(s.long.0, s.long.1);
        while (fx + length * cos(a)).abs() > 0.94 * reach
            || (fy + length * sin(a)).abs() > 0.94 * reach
        {
            length *= 0.94;
        }
        let bow = rng.uniform(-0.3, 0.3);
        let pts = [
            v3(
                fx + 0.02 * s.across * cos(a),
                fy + 0.02 * s.across * sin(a),
                0.0,
            ),
            v3(
                fx + length * 0.5 * cos(a + bow * 0.5),
                fy + length * 0.5 * sin(a + bow * 0.5),
                0.012 * s.across,
            ),
            v3(fx + length * cos(a), fy + length * sin(a), 0.02 * s.across),
        ];
        sketch.add(
            twig(
                "twig",
                &pts,
                s.wood.0,
                s.wood.1,
                BLANK,
                Twig {
                    verts: 4,
                    ..Twig::default()
                },
            ),
            s.wood.2,
            0.0,
        );
        let along = |f: f64| {
            if f < 0.5 {
                pts[0].lerp(pts[1], f * 2.0)
            } else {
                pts[1].lerp(pts[2], f * 2.0 - 1.0)
            }
        };
        for i in 0..s.per {
            let f = s.from + (1.0 - s.from) * (i as f64 + 0.6) / s.per as f64;
            let p = along(f);
            let lean = s.lean.0 + (s.lean.1 - s.lean.0) * f;
            let sides: &[f64] = if s.opposite {
                &[1.0, -1.0]
            } else if i % 2 != 0 {
                &[-1.0]
            } else {
                &[1.0]
            };
            for side in sides {
                let heading = a + side * lean * rng.uniform(0.75, 1.25) + rng.uniform(-0.2, 0.2);
                let size = rng.uniform(s.size.0, s.size.1);
                add(sketch, rng, (p.x as f64, p.y as f64), heading, size);
            }
        }
        for i in 0..s.tip {
            let p = along(1.0);
            let heading = a
                + (i as f64 - (s.tip as f64 - 1.0) / 2.0) * TAU * 0.62 / s.tip as f64
                + rng.uniform(-0.2, 0.2);
            let size = rng.uniform(s.size.0, s.size.1);
            add(sketch, rng, (p.x as f64, p.y as f64), heading, size);
        }
    }
    for _ in 0..s.top {
        let a = rng.uniform(0.0, TAU);
        let r = reach * rng.uniform(0.0, 0.22);
        let heading = a + rng.uniform(-0.6, 0.6);
        let size = rng.uniform(s.size.0, s.size.1);
        add(
            sketch,
            rng,
            (fx + r * cos(a), fy + r * sin(a)),
            heading,
            size,
        );
    }
}

/// A wall of a species' leaves over `across` metres round the tile's middle (and a little past it), at every angle,
/// a few deep.
pub(super) fn wall(
    sketch: &mut Sketch,
    rng: &mut Random,
    leaf: Leaf,
    across: f64,
    size: (f64, f64),
    wide: f64,
) {
    let half = across / 2.0 + 0.4 * size.1;
    let mean = (size.0 + size.1) / 2.0;
    let leaves = (2.6 * (2.0 * half) * (2.0 * half) / (mean * mean * wide * 0.62)) as usize;
    for i in 0..leaves {
        let at = (
            rng.uniform(-half, half),
            rng.uniform(-half, half),
            0.3 * across * i as f64 / leaves as f64,
        );
        let heading = rng.uniform(0.0, TAU);
        let length = rng.uniform(size.0, size.1);
        leaf(
            sketch,
            rng,
            at,
            heading,
            length,
            1.0 - i as f64 / leaves as f64,
        );
    }
}

/// A cluster of cherry blossom: bunches of flowers on short stalks along twigs fanning from the tile's middle,
/// a couple of bronze young leaves beside each bunch.
pub(super) fn blossom(sketch: &mut Sketch, rng: &mut Random, across: f64, twigs: usize, turn: f64) {
    let reach = across / 2.0 * REACH;
    let size = (0.06, 0.092);
    let wood = tone(0.32, 1.4);
    let mut bunches: Vec<(f64, f64, f64)> = vec![(0.0, 0.0, rng.uniform(0.0, TAU))];
    for t in 0..twigs {
        let a = turn + TAU * (t as f64 + rng.uniform(-0.2, 0.2)) / twigs as f64;
        let length = reach * rng.uniform(0.62, 0.8);
        let bow = rng.uniform(-0.25, 0.25);
        let pts = [
            v3(0.0, 0.0, 0.0),
            v3(
                length * 0.5 * cos(a + bow),
                length * 0.5 * sin(a + bow),
                0.008,
            ),
            v3(length * cos(a), length * sin(a), 0.012),
        ];
        sketch.add(
            twig(
                "twig",
                &pts,
                0.007,
                0.004,
                BLANK,
                Twig {
                    verts: 4,
                    ..Twig::default()
                },
            ),
            wood,
            0.0,
        );
        for f in [0.38, 0.7, 1.0] {
            let p = if f < 1.0 {
                pts[0].lerp(pts[2], f) + (pts[1] - pts[0].lerp(pts[2], 0.5)) * 0.8
            } else {
                pts[2]
            };
            bunches.push((p.x as f64, p.y as f64, a));
        }
        // (and one between this twig and the next, on a stalk too short to show)
        let between = a + PI / twigs as f64;
        let r = reach * rng.uniform(0.5, 0.68);
        bunches.push((r * cos(between), r * sin(between), between));
    }
    let total = bunches.len();
    // the leaves first, under the flowers: two to a bunch, pointing out past it
    for (i, &(x, y, a)) in bunches.iter().enumerate() {
        for side in [-1.0, 0.0, 1.0] {
            let heading = a + side * rng.uniform(0.5, 1.3) + rng.uniform(-0.2, 0.2);
            let length = fits((x, y), heading, rng.uniform(0.1, 0.13), 0.5, (reach, reach));
            if length > 0.06 {
                let depth = 0.6 * rng.random();
                cherry_leaf(
                    sketch,
                    rng,
                    (x, y, 0.01 + 0.002 * i as f64),
                    heading,
                    length,
                    depth,
                );
            }
        }
    }
    for (i, &(x, y, _)) in bunches.iter().enumerate() {
        let flowers = rng.randint(5, 7);
        let off = rng.uniform(0.0, TAU);
        for j in 0..flowers {
            let a = off + TAU * (j as f64 + rng.uniform(-0.2, 0.2)) / flowers as f64;
            let r = rng.uniform(0.3, 0.8) * size.1;
            let across = rng.uniform(size.0, size.1);
            let (fx, fy) = (x + r * cos(a), y + r * sin(a));
            if fx.abs() + across / 2.0 < reach && fy.abs() + across / 2.0 < reach {
                let depth = 1.0 - (i * 6 + j as usize) as f64 / (total * 6) as f64;
                flower(
                    sketch,
                    rng,
                    (
                        fx,
                        fy,
                        0.03 + 0.012 * (i * 6 + j as usize) as f64 / total as f64,
                    ),
                    across,
                    depth * 0.8,
                );
            }
        }
    }
}

/// A wall of cherry blossom with a few young leaves in it.
pub(super) fn blossom_wall(sketch: &mut Sketch, rng: &mut Random, across: f64) {
    let half = across / 2.0 + 0.04;
    let flowers = (2.2 * (2.0 * half) * (2.0 * half) / (0.078 * 0.078 * 0.8)) as usize;
    for i in 0..flowers {
        let at = (
            rng.uniform(-half, half),
            rng.uniform(-half, half),
            0.3 * across * i as f64 / flowers as f64,
        );
        let depth = 1.0 - i as f64 / flowers as f64;
        if i % 6 == 0 {
            let heading = rng.uniform(0.0, TAU);
            let length = rng.uniform(0.1, 0.13);
            cherry_leaf(sketch, rng, at, heading, length, depth);
        } else {
            let size = rng.uniform(0.07, 0.085);
            flower(sketch, rng, at, size, depth);
        }
    }
}

/// A weeping willow's hanging shoots as its strands' cards show them, `wide` by `tall` metres (the tile's top is
/// up): slender shoots from the top edge to the bottom, swaying a little, narrow leaves down both sides of each,
/// pointing down and out.
pub(super) fn strands(sketch: &mut Sketch, rng: &mut Random, wide: f64, tall: f64, shoots: usize) {
    let reach = (wide / 2.0 * REACH, tall / 2.0 * REACH);
    let wood = tone(0.7, 1.6);
    for s in 0..shoots {
        let x0 = wide * 0.7 * ((s as f64 + 0.5) / shoots as f64 - 0.5) + rng.uniform(-0.01, 0.01);
        let sway = rng.uniform(0.012, 0.028) * if s % 2 != 0 { -1.0 } else { 1.0 };
        let phase = rng.uniform(0.0, TAU);
        let x = |f: f64| x0 + sway * sin(phase + f * 4.2);
        // (a shoot ends short of the tile's top and bottom: the cards down a strand lap over one another)
        let pts: Vec<V3> = (0..=6)
            .map(|i| {
                let f = i as f64 / 6.0;
                v3(x(f), reach.1 * (1.0 - 2.0 * f), 0.02 + 0.01 * s as f64)
            })
            .collect();
        sketch.add(
            twig(
                "shoot",
                &pts,
                0.0035,
                0.002,
                BLANK,
                Twig {
                    verts: 4,
                    ..Twig::default()
                },
            ),
            wood,
            0.0,
        );
        let leaves = (tall / 0.03) as usize;
        for i in 0..leaves {
            let f = (i as f64 + rng.uniform(0.2, 0.8)) / leaves as f64;
            let side = if i % 2 != 0 { -1.0 } else { 1.0 };
            let p = (x(f), reach.1 * (1.0 - 2.0 * f));
            let heading = -PI / 2.0 + side * rng.uniform(0.2, 0.6);
            let length = fits(p, heading, rng.uniform(0.085, 0.12), 0.2, reach);
            if length > 0.05 {
                let depth = 1.0 - (s * leaves + i) as f64 / (shoots * leaves) as f64;
                willow_leaf(
                    sketch,
                    rng,
                    (p.0, p.1, 0.02 + 0.01 * s as f64 + 0.004),
                    heading,
                    length,
                    depth * 0.8,
                );
            }
        }
    }
}

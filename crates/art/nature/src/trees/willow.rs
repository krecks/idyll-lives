//! Weeping willow: a thick, rough trunk on root buttresses, five limbs rising out of it and arching over, a domed
//! crown of leaf masses along their arches, and curtains of leaves hanging from them nearly to the ground.

use artkit::botany::Tiles;
use artkit::botany::skeleton::{Buttress, ClumpAims, Limb, LimbPoints, Stem};
use artkit::botany::species::{Aims, Perches, Roots, Seeds, Species, Stage, Tree, Trunk, grow};
use artkit::botany::{Cards, card_of, stand_in};
use artkit::far::{is_low, low};
use artkit::materials::Mat;
use artkit::math::{TAU, V3, cos, pow, sin, v3};
use artkit::perch::Find;
use artkit::py;
use artkit::scene::Obj;
use artkit::{_mesh, bough, mass};

pub fn species() -> Species {
    Species {
        name: "willow",
        seeds: Seeds {
            build: 18,
            perches: 118,
            cards: Some(218),
        },
        leaves: &["#6A8A36", "#7A9A3E", "#8AAA48", "#9CBA54", "#AEC862"],
        tiles: Tiles::of("willow"),
        trunk: Stage::Spec(Trunk {
            pts: vec![
                (0.0, 0.0, 0.0),
                (0.0, 0.0, 0.3),
                (0.06, 0.03, 1.2),
                (0.02, -0.04, 2.1),
                (0.05, 0.0, 2.7),
            ],
            r: 0.31,
            scale: vec![1.4, 1.05, 0.97, 0.9, 0.8],
            seed: 10.0,
            o: Stem {
                wobble: 0.045,
                ..Stem::default()
            },
        }),
        roots: Stage::Spec(Roots {
            n: 6,
            r: 0.43,
            h: 0.6,
            reach: 0.75,
            o: Buttress::default(),
        }),
        crown: Stage::Own(crown),
        // On the crown's top, from above (lower, the lines would meet the curtains), and on the limbs under it,
        // inside the curtain.
        perches: Stage::Spec(Perches {
            n: 20,
            aims: Aims::Clumps {
                o: ClumpAims {
                    per: 4,
                    up: (0.55, 1.2),
                },
                more: Vec::new(),
            },
            limbs: Some(LimbPoints {
                along: [0.3, 0.45, 0.6],
            }),
            o: Find {
                limbs_n: Some(4),
                ..Find::default()
            },
        }),
        ..Species::default()
    }
}

pub fn willow() -> Vec<Obj> {
    grow(&species())
}

/// A willow's hanging strand: a thin leafy ribbon from `top` (Blender metres), out `reach` towards `heading`
/// (radians) and down `drop`, falling straight by its foot, its face outwards, narrowing to a third of `width`: a
/// thin wedge across (three points a row, its ridge out, smooth), so it shades as a soft strip.
fn streamer(
    name: &str,
    top: (f64, f64, f64),
    heading: f64,
    reach: f64,
    drop: f64,
    width: f64,
    material: Mat,
) -> Obj {
    let rows = low(5, 3);
    let out = v3(cos(heading), sin(heading), 0);
    let side = v3(-out.y, out.x, 0);
    let (mut verts, mut faces) = (Vec::new(), Vec::new());
    for i in 0..=rows {
        let t = i as f64 / rows as f64;
        let c =
            V3::from(top) + out * reach * (1.0 - pow(1.0 - t, 3.0)) - v3(0, 0, drop * pow(t, 1.15));
        let w = width * (1.0 - 0.66 * t) / 2.0;
        verts.extend([c - side * w, c + out * 0.03_f64.min(w * 0.5), c + side * w]);
    }
    for i in 0..rows {
        let (a, b) = (3 * i, 3 * (i + 1));
        for j in 0..3 {
            faces.push(vec![a + j, a + (j + 1) % 3, b + (j + 1) % 3, b + j]);
        }
    }
    faces.extend([vec![0, 2, 1], vec![3 * rows, 3 * rows + 1, 3 * rows + 2]]);
    _mesh!(name, verts, faces, material)
}

// A strand of the willow's curtain as leaf cards: one every `STRAND_STEP` metres down it, each `STRAND_LAP` of
// that tall (they overlap) and `STRAND_WIDE` of the ribbon's width across.
const STRAND_STEP: f64 = 0.5;
const STRAND_LAP: f64 = 1.5;
const STRAND_WIDE: f64 = 1.5;

/// A strand as the crown stage leaves it for the cards: `streamer`'s line, and its cards' colour.
type Strand = ((f64, f64, f64), f64, f64, f64, f64, Mat);

/// A willow's hanging strand (`streamer`'s line) as upright leaf cards down it (`Cards::hang`), each turned a
/// little off the next so the curtain shows from every side, the clusters drawn long as hanging leaves; their
/// normals out from the trunk and up, darker towards the ground. (They stay in the ribbons' box: `Cards`.)
fn strand(cards: &mut Cards, (top, heading, reach, drop, width, material): Strand) {
    let out = v3(cos(heading), sin(heading), 0);
    let n = 2.max(py::round(drop / STRAND_STEP));
    for i in 0..n {
        let t = (i as f64 + 0.5) / n as f64;
        let c =
            V3::from(top) + out * reach * (1.0 - pow(1.0 - t, 3.0)) - v3(0, 0, drop * pow(t, 1.15));
        let turn = heading + cards.rng.uniform(-0.9, 0.9);
        let tall = drop / n as f64 * STRAND_LAP;
        let facing = (cos(turn), sin(turn), cards.rng.uniform(0.0, 0.3));
        let shade = 0.62 + 0.3 * (1.0 - t) + cards.rng.uniform(-0.08, 0.08);
        cards.hang(
            c,
            facing,
            width * STRAND_WIDE,
            tall,
            material,
            |q: V3| v3(q.x, q.y, 0).normalized() + v3(0, 0, 0.5),
            shade,
        );
    }
}

/// The limbs, the crown's leaf masses along their arches and the curtains hanging from them.
fn crown(tree: &mut Tree) {
    let (p, rng, cols) = (&tree.palette, &mut tree.rng, &tree.cols);
    let full = !is_low();
    // the limbs: up and out of the trunk's top, then over in an arch and down at their ends
    let mut made: Vec<Limb> = Vec::new();
    let mut masses: Vec<(V3, f64, usize)> = Vec::new();
    let off = rng.random() * TAU;
    for j in 0..5 {
        let a = off + TAU * j as f64 / 5.0 + rng.uniform(-0.2, 0.2);
        let (reach, z0, top) = (
            rng.uniform(1.85, 2.05),
            rng.uniform(2.3, 2.6),
            rng.uniform(4.8, 5.1),
        );
        let (c, s_) = (cos(a), sin(a));
        let line = [
            v3(0.1 * c, 0.1 * s_, z0),
            v3(0.45 * reach * c, 0.45 * reach * s_, z0 + (top - z0) * 0.62),
            v3(0.72 * reach * c, 0.72 * reach * s_, top),
            v3(reach * c, reach * s_, top - 0.35),
        ];
        bough!(format!("limb{j}"), &line, 0.13, 0.04, p, verts = 6, res = 2);
        made.push((line[..3].to_vec(), 0.13, 0.07));
        // a bough from the arch's crest over to each side, arching down too
        for (k, turn) in [-0.55, 0.55].into_iter().enumerate() {
            let b = a + turn;
            let end = v3(
                reach * 0.92 * cos(b),
                reach * 0.92 * sin(b),
                top - 0.4 + rng.uniform(-0.1, 0.1),
            );
            let crest = line[1] + (line[2] - line[1]) * 0.5;
            let mid = (crest + end) / 2 + v3(0, 0, 0.3);
            if full {
                bough!(
                    format!("bough{j}_{k}"),
                    &[crest, mid, end],
                    0.045,
                    0.015,
                    p,
                    verts = 4,
                    res = 2
                );
            }
            masses.push((mid, b, 1));
        }
        masses.push((line[2], a, 2));
    }
    masses.push((v3(0.05, 0.0, 5.0), off, 3));
    // the crown: leaf masses along the arches' crests and over the middle, drooping
    let light = |z: f64| (z - 4.2) / 1.6;
    for &(at, a, k) in &masses {
        mass!(
            "crown",
            rng,
            at + v3(0, 0, 0.12),
            a,
            cols,
            &light,
            &mut tree.clumps,
            k = k,
            size = if k < 3 { (0.62, 0.74) } else { (0.85, 0.95) },
            ring = if k < 3 { 0.42 } else { 0.6 },
            drop = (-0.2, 0.0),
            squash = (0.6, 0.72),
            noise = 0.3
        );
    }
    // Hanging curtains: long, thin leafy streamers all round, out of the crown's edge and straight down to about
    // half a metre off the lawn, in two staggered rows with gaps between them, so whoever sits by the trunk shows
    // through.
    let n = 60;
    let mut strands: Vec<Strand> = Vec::new();
    for i in 0..n {
        let a = i as f64 / n as f64 * TAU + rng.uniform(-0.04, 0.04);
        let outer = i % 2 == 0;
        let rr = if outer {
            rng.uniform(2.0, 2.25)
        } else {
            rng.uniform(1.6, 1.85)
        };
        let top = if outer {
            rng.uniform(4.5, 5.1)
        } else {
            rng.uniform(4.1, 4.6)
        };
        let foot = if outer {
            rng.uniform(0.45, 0.8)
        } else {
            rng.uniform(0.9, 1.5)
        };
        let (heading, reach, width) = (
            a + rng.uniform(-0.15, 0.15),
            rng.uniform(0.25, 0.45),
            rng.uniform(0.17, 0.23),
        );
        let col = cols[rng.randint(1, 4) as usize];
        let from = (rr * cos(a), rr * sin(a), top);
        if full || outer {
            // far: the outer row, wider
            let ribbon = streamer(
                &format!("strand{i}"),
                from,
                heading,
                reach,
                top - foot,
                width * low(1.0, 1.5),
                col,
            );
            if full {
                // leaf cards down the ribbon, once the crown is clad
                stand_in(ribbon);
                strands.push((
                    from,
                    heading,
                    reach,
                    top - foot,
                    width,
                    p.get(card_of(col.name)),
                ));
            }
        }
    }
    tree.limbs = made;
    tree.more = Some(Box::new(move |cards: &mut Cards| {
        for s in strands {
            strand(cards, s);
        }
    }));
}

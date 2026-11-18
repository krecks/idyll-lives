//! Oak: a stout trunk flared on root buttresses, branching low into five wide limbs that fork into boughs, each
//! bough ending in a mass of leaf clumps, lighter on top and darker below: a broad, broken crown whose limbs show
//! through the gaps between its masses. The first of the broadleaves, grown by hand: only its trunk is the shared
//! skeleton's.

use artkit::botany::Tiles;
use artkit::botany::skeleton::{ClumpAims, Stem};
use artkit::botany::species::{Aims, Perches, Seeds, Species, Stage, Tree, Trunk, grow};
use artkit::far::is_low;
use artkit::math::{PI, TAU, V3, cos, sin, v3};
use artkit::perch::Find;
use artkit::py;
use artkit::scene::Obj;
use artkit::{bend, bough, leaves};

pub fn species() -> Species {
    Species {
        name: "oak",
        seeds: Seeds {
            build: 11,
            perches: 111,
            cards: Some(211),
        },
        leaves: &["#558238", "#64923E", "#76A246", "#8AB250", "#9EC25C"],
        tiles: Tiles::of("oak"),
        // (the bark texture is dark: under white it is as light as it gets, a little under an oak's bark)
        colours: &[("bark", "#FFFFFF")],
        // the trunk: a slight lean and wave, flared at the foot, its base flat on the ground
        trunk: Stage::Spec(Trunk {
            pts: vec![
                (0.0, 0.0, 0.0),
                (0.0, 0.0, 0.3),
                (0.05, -0.03, 1.1),
                (-0.02, 0.04, 1.9),
                (0.03, 0.0, 2.7),
            ],
            r: 0.3,
            scale: vec![1.5, 1.08, 0.95, 0.88, 0.72],
            seed: 4.0,
            o: Stem {
                verts: 11,
                wobble: 0.035,
                ..Stem::default()
            },
        }),
        roots: Stage::Own(roots),
        crown: Stage::Own(crown),
        hearts: Stage::Own(heart),
        perches: Stage::Spec(Perches {
            n: 24,
            aims: Aims::Clumps {
                o: ClumpAims::default(),
                more: Vec::new(),
            },
            limbs: None,
            o: Find::default(),
        }),
        ..Species::default()
    }
}

pub fn oak() -> Vec<Obj> {
    grow(&species())
}

/// Root buttresses running out from the foot, ending just above the ground.
fn roots(tree: &mut Tree) {
    let (p, rng) = (&tree.palette, &mut tree.rng);
    for i in 0..5 {
        let a = TAU * (i as f64 + rng.uniform(-0.2, 0.2)) / 5.0;
        let reach = rng.uniform(0.5, 0.7);
        let root = [
            (0.12 * cos(a), 0.12 * sin(a), 0.55),
            (0.34 * cos(a), 0.34 * sin(a), 0.2),
            (reach * cos(a), reach * sin(a), 0.085),
        ];
        bough!(format!("root{i}"), &root, 0.17, 0.07, p, verts = 5, res = 1);
    }
}

/// What the limbs leave for the boughs and the heart: the turn the crown starts at, each limb's (heading, end,
/// reach) and the leader's end.
struct Limbs {
    off: f64,
    ends: Vec<(f64, V3, f64)>,
    leader: V3,
}

/// A fork of boughs: (its boughs' names start so, where it forks, its heading, how many boughs, their reach and
/// their rise, each a range).
type Fork = (String, V3, f64, usize, (f64, f64), (f64, f64));

fn crown(tree: &mut Tree) {
    let made = limbs(tree);
    masses(tree, &made);
    tree.kept = Some(Box::new(made));
}

/// Five limbs from the top of the trunk, wide and only a little up; a leader carries on up the middle.
fn limbs(tree: &mut Tree) -> Limbs {
    let (p, rng) = (&tree.palette, &mut tree.rng);
    let off = rng.random() * TAU + PI / 2.0; // (turned: its broad side across)
    let mut ends = Vec::new();
    for i in 0..5 {
        let a = off + TAU * i as f64 / 5.0 + rng.uniform(-0.25, 0.25);
        let z0 = rng.uniform(1.95, 2.6);
        let (out, up) = (rng.uniform(1.15, 1.45), rng.uniform(3.6, 4.4));
        let end = v3(out * cos(a), out * sin(a), up);
        let line = bend!((0.02, 0.0, z0), end, rng, lift = 0.15);
        bough!(format!("limb{i}"), &line, 0.16, 0.075, p, verts = 5);
        ends.push((a, end, out));
    }
    let leader = v3(0.1, -0.05, 4.9);
    let line = bend!((0.03, 0.0, 2.6), leader, rng, lift = 0.0, wander = 0.3);
    bough!("leader", &line, 0.17, 0.07, p, verts = 5);
    Limbs { off, ends, leader }
}

/// Each limb forks into boughs, out and up; the leader into boughs fanning over the top. A mass of clumps at each
/// bough's tip.
fn masses(tree: &mut Tree, limbs: &Limbs) {
    let (p, rng, cols) = (&tree.palette, &mut tree.rng, &tree.cols);
    let full = !is_low();
    let mut forks: Vec<Fork> = limbs
        .ends
        .iter()
        .enumerate()
        .map(|(i, &(a, end, _))| {
            let count = if i % 2 != 0 { 2 } else { 3 };
            (format!("b{i}_"), end, a, count, (1.65, 1.95), (0.75, 1.95))
        })
        .collect();
    forks.push((
        "top_".to_string(),
        limbs.leader,
        limbs.off,
        4,
        (0.5, 1.2),
        (1.55, 2.2),
    ));
    let shades = cols.len() as i64;
    let mut n = 0;
    for (prefix, base, a0, count, reach, rise) in forks {
        for j in 0..count {
            let a = if prefix != "top_" {
                a0 + (j as f64 - (count - 1) as f64 / 2.0) * 0.75 + rng.uniform(-0.2, 0.2)
            } else {
                a0 + TAU * j as f64 / count as f64 + rng.uniform(-0.3, 0.3)
            };
            let out = rng.uniform(reach.0, reach.1);
            let tip = v3(
                out * cos(a),
                out * sin(a),
                base.z as f64 + rng.uniform(rise.0, rise.1),
            );
            let mid = base
                + (tip - base) * 0.5
                + v3(rng.uniform(-0.15, 0.15), rng.uniform(-0.15, 0.15), 0.2);
            if full {
                bough!(
                    format!("{prefix}{j}"),
                    &[base, mid, tip],
                    0.07,
                    0.025,
                    p,
                    verts = 5,
                    res = 2
                );
            }
            // the mass: a clump on the tip and two round it, a little below and out
            for m in 0..3 {
                let at = if m == 0 {
                    tip
                } else {
                    let b = a + (m as f64 - 1.5) * 1.6 + rng.uniform(-0.4, 0.4);
                    tip + v3(0.42 * cos(b), 0.42 * sin(b), rng.uniform(-0.35, 0.05))
                };
                let mut size = rng.uniform(0.5, 0.64) * if m == 0 { 1.0 } else { 0.86 };
                let (sx, sy, sq) = (
                    rng.uniform(0.9, 1.15),
                    rng.uniform(0.9, 1.15),
                    rng.uniform(0.75, 0.88),
                );
                let light = 0.0_f64.max(1.0_f64.min((at.z as f64 - 3.4) / 3.4));
                let shade = py::round(light * (shades - 1) as f64 + rng.uniform(-0.6, 0.6));
                let col = cols[0.max((shades - 1).min(shade)) as usize];
                let seed = n as f64 * 1.7 + rng.random();
                let mut half = (size * sx, size * sy, size * sq);
                tree.clumps.push((
                    (at.x as f64, at.y as f64, at.z as f64),
                    Some(half),
                    2.0 * light - 1.0,
                    true,
                ));
                if !full {
                    // far: one bigger clump stands for the mass
                    if m != 0 {
                        n += 1;
                        continue;
                    }
                    size *= 1.4;
                    half = (size * sx, size * sy, size * sq);
                }
                leaves!(
                    format!("clump{n}"),
                    size,
                    (at.x as f64, at.y as f64, at.z as f64 - half.2),
                    col,
                    scale = (sx, sy, sq),
                    noise = 0.34,
                    seed = seed,
                    segments = 6,
                    rings = 5,
                    group = (prefix.as_str(), j)
                );
                n += 1;
            }
        }
    }
}

/// The crown's heart, dark, round the leader: it closes the middle, the gaps between the masses stay.
fn heart(tree: &mut Tree) {
    let off = tree.kept::<Limbs>().off;
    let (rng, cols) = (&mut tree.rng, &tree.cols);
    for i in 0..4 {
        let a = off + TAU * (i as f64 + 0.5) / 4.0 + rng.uniform(-0.3, 0.3);
        let (size, seed) = (rng.uniform(0.75, 0.9), rng.random() * 9.0);
        let at = (0.75 * cos(a), 0.75 * sin(a), rng.uniform(4.0, 4.7));
        let col = cols[rng.randrange(2) as usize];
        leaves!(
            format!("heart{i}"),
            size,
            at,
            col,
            scale = (1, 1, 0.85),
            noise = 0.3,
            seed = seed,
            segments = 6,
            rings = 5
        );
    }
}

//! Japanese maple: a short trunk branching at knee height into six slender stems rising in long S curves, a vase of
//! flat red clouds in layers at their boughs' tips: a narrow first layer inside its tile at chest height, the broad
//! tiers over head height (from 2.1 m), so residents walk past and work at it from the next tile; a few stray sprigs
//! on top, which shaping takes off.

use artkit::botany::Tiles;
use artkit::botany::skeleton::{Buttress, ClumpAims, Stem};
use artkit::botany::species::{Aims, Perches, Roots, Seeds, Species, Stage, Tree, Trunk, grow};
use artkit::botany::{_with_part, WithPart};
use artkit::far::is_low;
use artkit::math::{PI, TAU, V3, cos, sin, v3};
use artkit::perch::Find;
use artkit::py;
use artkit::scene::Obj;
use artkit::{bough, mass, sprigs};

pub fn species() -> Species {
    Species {
        name: "japaneseMaple",
        seeds: Seeds {
            build: 16,
            perches: 116,
            cards: Some(216),
        },
        leaves: &["#6E1C1C", "#8A2424", "#A2302A", "#B83F2F", "#CC5636"],
        tiles: Tiles::of("japaneseMaple"),
        colours: &[("bark", "#FFFFFF"), ("leaf_light", "#B83F2F")],
        // a short trunk splitting at knee height into six slender stems, each rising in a long S out to the
        // vase's rim
        trunk: Stage::Spec(Trunk {
            pts: vec![(0.0, 0.0, 0.0), (0.0, 0.0, 0.25), (0.02, 0.0, 0.55)],
            r: 0.1,
            scale: vec![1.5, 1.1, 1.0],
            seed: 8.0,
            o: Stem {
                verts: 8,
                wobble: 0.012,
                ..Stem::default()
            },
        }),
        roots: Stage::Spec(Roots {
            n: 3,
            r: 0.14,
            h: 0.22,
            reach: 0.26,
            o: Buttress::default(),
        }),
        crown: Stage::Own(crown),
        // (before the sprigs)
        perches: Stage::Spec(Perches {
            n: 18,
            aims: Aims::Clumps {
                o: ClumpAims::default(),
                more: Vec::new(),
            },
            limbs: None,
            o: Find {
                gap: 0.35,
                ..Find::default()
            },
        }),
        fruit: Some(trimmed),
        ..Species::default()
    }
}

pub fn japanese_maple() -> Vec<Obj> {
    grow(&species())
}

/// Six slender stems, and the layers of flat red clouds at their boughs' tips.
fn crown(tree: &mut Tree) {
    let (p, rng, cols) = (&tree.palette, &mut tree.rng, &tree.cols);
    let full = !is_low();
    let mut forks: Vec<(V3, f64, V3)> = Vec::new();
    for j in 0..6 {
        let a = TAU * (j as f64 + 0.5) / 6.0 + rng.uniform(-0.25, 0.25);
        let (r, z) = (rng.uniform(0.5, 0.72), rng.uniform(2.2, 2.5));
        let (c, s_) = (cos(a), sin(a));
        let w = rng.uniform(-0.12, 0.12); // the S: out, back in a little, out again
        let stem = [
            (0.03 * c, 0.03 * s_, 0.45),
            ((0.2 + w) * c - w * s_, (0.2 + w) * s_ + w * c, 1.0),
            (
                (r * 0.55 - w) * c + w * s_,
                (r * 0.55 - w) * s_ - w * c,
                1.65,
            ),
            (r * c, r * s_, z),
        ];
        bough!(
            format!("stem{j}"),
            &stem,
            0.055,
            0.03,
            p,
            verts = 5,
            res = 2
        );
        forks.push((V3::from(stem[3]), a, V3::from(stem[2])));
    }
    // the layers: flat red clouds at the boughs' tips, broad at 2.7 m, narrower over it, the narrow first layer at
    // chest height on short shoots inside the tile
    let light = |z: f64| (z - 1.3) / 2.6;
    let clumps = &mut tree.clumps;
    for (k, (z, reach, n)) in [(2.7, 1.3, 7), (3.2, 0.95, 5), (3.6, 0.45, 3)]
        .into_iter()
        .enumerate()
    {
        let off = rng.random() * TAU;
        for i in 0..n {
            let a = off + TAU * i as f64 / n as f64 + rng.uniform(-0.2, 0.2);
            let x = reach * rng.uniform(0.9, 1.08) * cos(a);
            let y = reach * rng.uniform(0.9, 1.08) * sin(a);
            let tip = v3(x, y, z + rng.uniform(-0.08, 0.08));
            let turn = |q: &(V3, f64, V3)| (py::fmod(a - q.1 + PI, TAU) - PI).abs();
            let mut nearest = 0;
            for q in 1..forks.len() {
                if turn(&forks[q]) < turn(&forks[nearest]) {
                    nearest = q;
                }
            }
            let f = forks[nearest].0;
            let mid = f + (tip - f) * 0.5 + v3(0, 0, 0.12 + rng.uniform(-0.04, 0.04));
            if full {
                bough!(
                    format!("bough{k}_{i}"),
                    &[f, mid, tip],
                    0.028,
                    0.012,
                    p,
                    verts = 4,
                    res = 2
                );
            }
            mass!(
                "cloud",
                rng,
                tip,
                a,
                cols,
                &light,
                clumps,
                k = 3,
                size = if k < 2 { (0.4, 0.5) } else { (0.36, 0.44) },
                ring = 0.34,
                drop = (-0.12, 0.02),
                squash = (0.48, 0.58),
                noise = 0.3
            );
        }
    }
    for &(_, a, low_pt) in forks.iter().step_by(2) {
        let tip = v3(
            low_pt.x as f64 * 0.7,
            low_pt.y as f64 * 0.7,
            low_pt.z as f64 - 0.08,
        );
        mass!(
            "first",
            rng,
            tip,
            a,
            &cols[1..],
            &light,
            clumps,
            k = 2,
            size = (0.26, 0.32),
            ring = 0.15,
            drop = (-0.08, 0.0),
            squash = (0.6, 0.7),
            noise = 0.3
        );
    }
}

/// Clad, with the stray sprigs on top a part of their own, which shaping takes off.
fn trimmed(tree: &mut Tree, still: Obj) -> Vec<Obj> {
    let at = (0.0, 0.0, 3.87);
    let still = tree.clad(still);
    let p = &tree.palette;
    _with_part(
        "sprigs",
        still,
        || sprigs!("sprig", p, at, 0.35, n = 6, length = 0.22),
        at,
        "trim",
        WithPart::default(),
    )
}

//! Flower beds and crops (families/plants.py's): a soil mound and what is spread over it, a raised bed of boards,
//! and a crop's growth from seedlings to ripe.

use crate::botany::cut::{CutLeaf, OutlineOf, Pinnate, cut_leaf, outline, pinnate};
use crate::botany::lump::{Lump, lump};
use crate::botany::shrubs::fit_to;
use crate::far::{is_low, low};
use crate::materials::{Mat, Palette};
use crate::math::{P3, PI, TAU, cos, degrees, sin, sqrt};
use crate::motion::{Factors, Stages, stages};
use crate::noise::noise_at;
use crate::parts::{join, join_all, measure, new_parts, settle};
use crate::rand::Random;
use crate::scene::{Obj, meshes};
use crate::shapes::{AUTO_BEVEL, Cube, Revolve, cube, options, revolve};

options! {
    /// `spiral`'s options.
    Spiral { jitter: f64 = 0.25 }
}

/// `n` points spread evenly over a disc (a sunflower's spiral), jittered: (x, y, degrees round).
pub fn spiral(n: usize, radius: f64, rng: &mut Random, o: Spiral) -> Vec<(f64, f64, f64)> {
    let ga = PI * (3.0 - sqrt(5.0));
    let off = rng.random() * TAU;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let rr = radius * sqrt((i as f64 + 0.5) / n as f64);
        let a = off + i as f64 * ga + rng.uniform(-o.jitter, o.jitter);
        out.push((rr * sin(a), rr * cos(a), degrees(a)));
    }
    out
}

options! {
    /// `mound`'s options.
    Mound { r: f64 = 0.42, h: f64 = 0.07 }
}

/// A bed's low soil mound (the palette `p`'s `bed`, the part the player colours), a little uneven on top, its edge
/// lifted where the soil was banked up: returns its top.
pub fn mound(p: &Palette, o: Mound) -> f64 {
    let (r, h) = (o.r, o.h);
    let obj = revolve(
        "bed",
        &[
            (0.0, 0.0),
            (r, 0.0),
            (r * 0.94, h * 0.4),
            (r * 0.65, h * 0.85),
            (0.0, h),
        ],
        Revolve {
            material: Some(p.get("bed")),
            segments: Some(low(18, 9)),
            ..Revolve::default()
        },
    );
    obj.edit(|part| {
        // the same on a far version: it keeps its floor
        for v in &mut part.mesh.verts {
            if v.z as f64 > 0.004 {
                v.z = (v.z as f64 + h * 0.18 * noise_at(v.x as f64 * 7.0, v.y as f64 * 7.0, 0.5))
                    as f32;
            }
        }
    });
    h
}

/// Low leafy clumps between the flowers.
pub fn clumps(rng: &mut Random, n: usize, radius: f64, size: f64, h: f64, z: f64, cols: &[Mat]) {
    for (i, (x, y, _)) in spiral(n, radius, rng, Spiral::default())
        .into_iter()
        .enumerate()
    {
        let s = rng.uniform(0.8, 1.2);
        lump(
            format!("clump{i}"),
            size / 2.0 * s,
            (x, y, z - h * 0.6),
            cols[i % cols.len()],
            Lump {
                scale: P3 {
                    x: 1.0,
                    y: 1.0,
                    z: h / size,
                },
                seed: i as f64 * 1.3 + rng.random(),
                segments: 8,
                rings: 5,
                ..Lump::default()
            },
        );
    }
}

options! {
    /// `rosettes`' options.
    Rosettes { leaves: usize = 5, length: f64 = 0.12, width: f64 = 0.06, shape: &'static str = "lance", tilt: (f64, f64) = (20.0, 45.0), droop: f64 = 0.3, name: &'static str = "rosette", pinnae: usize = 0 }
}

/// `n` low rosettes of `leaves` leaves each (or cut into `pinnae` leaflet pairs) spread over a bed: the flowers'
/// own leaves, where the old beds had lumps.
pub fn rosettes(rng: &mut Random, n: usize, radius: f64, z: f64, cols: &[Mat], o: Rosettes) {
    let name = o.name;
    for (i, (x, y, a)) in spiral(n, radius, rng, Spiral::default())
        .into_iter()
        .enumerate()
    {
        let s = rng.uniform(0.8, 1.2);
        for j in 0..o.leaves {
            let yaw = a + (j * 360) as f64 / o.leaves as f64 + rng.uniform(-15.0, 15.0);
            let t = rng.uniform(o.tilt.0, o.tilt.1);
            let edge = if o.pinnae != 0 {
                pinnate(
                    o.pinnae,
                    o.width * s,
                    o.shape,
                    Pinnate {
                        lean: 0.6,
                        fill: 0.7,
                        ..Pinnate::default()
                    },
                )
            } else {
                outline(
                    o.shape,
                    o.width * s,
                    OutlineOf {
                        n: 3,
                        ..OutlineOf::default()
                    },
                )
            };
            cut_leaf(
                format!("{name}{i}_{j}"),
                (x, y, z),
                yaw,
                t,
                o.length * s,
                &edge,
                cols[(i + j) % cols.len()],
                CutLeaf {
                    droop: o.droop,
                    fold: 0.25,
                    ribs: 2,
                    thick: 0.004,
                    ..CutLeaf::default()
                },
            );
        }
    }
}

/// A crop's scale at each stage of its growth clip, times its size at rest (`motion::stages`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Growth {
    pub picked: Factors,
    pub growing: Factors,
    pub planted: Factors,
}

// The growth stages (the growth clip, `motion::stages`): ripe at rest; picked, the crop is gone and the plants
// stay; growing, the plants are a bit smaller; planted, they are seedlings. The plants are spread over a bed, so
// they only get lower and sink into the soil: each shows where it stands, and the game needn't know where that is.
// The crop shrinks to nothing about the same origin, on the soil.

/// The plants' stages.
pub const PLANTS: Growth = Growth {
    picked: Factors([1.0, 1.0, 1.0]),
    growing: Factors([1.0, 1.0, 0.8]),
    planted: Factors([1.0, 1.0, 0.5]),
};
/// How far the plants go into the soil when picked, growing and planted: shares of their height.
pub const SINK: (f64, f64, f64) = (0.0, 0.1, 0.32);

/// A bed whose crop is the plants themselves (lettuces, cabbages, carrots): picked, they are gone under the soil.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Harvested {
    pub picked: Factors,
    pub sink: f64,
}
pub const HARVESTED: Harvested = Harvested {
    picked: Factors([1.0, 1.0, 0.5]),
    sink: 0.6,
};
/// The crop's stages: gone but when ripe.
pub const CROP: Growth = Growth {
    picked: Factors([0.001, 0.001, 0.001]),
    growing: Factors([0.001, 0.001, 0.001]),
    planted: Factors([0.001, 0.001, 0.001]),
};

options! {
    /// `_crop`'s options.
    Crop { harvested: bool = false, sink: (f64, f64, f64) = SINK, size: Option<(f64, f64, f64)> = None }
}

/// The still part, and the leaves and crop `leaves()` and `crop()` make as the growth clip's two parts.
/// `harvested`: picking takes the plants too (a vegetable patch), not only their crop (tomatoes); `sink`: how far
/// the plants go into the soil at each stage, shares of their height (low plants on a bed sink less: `SINK` would
/// bury them); `size`: the whole fitted to it first (`fit_to`).
pub fn _crop(
    still: Obj,
    origin: impl Into<P3>,
    leaves: impl FnOnce(),
    crop: impl FnOnce(),
    o: Crop,
) -> Vec<Obj> {
    if is_low() {
        // a far version: all one still part, ripe
        leaves();
        crop();
        if let Some(size) = o.size {
            fit_to(&meshes(), size.0, size.1, size.2);
        }
        return settle(&[join_all(&still.name())]);
    }
    let lp = new_parts(leaves);
    let cp = new_parts(crop);
    let k = match o.size {
        Some(size) => fit_to(&meshes(), size.0, size.1, size.2),
        None => [1.0, 1.0, 1.0],
    };
    let origin: P3 = origin.into();
    let origin = P3 {
        x: origin.x * k[0],
        y: origin.y * k[1],
        z: origin.z * k[2],
    };
    let plants = join(&lp, "plants", Some(origin));
    let fruit = join(&cp, "crop", Some(origin));
    settle(&[still, plants, fruit]);
    let height = measure(&[plants]).1[2] - plants.location().z as f64; // the plants' top over their origin
    let mut sink = [o.sink.0 * height, o.sink.1 * height, o.sink.2 * height];
    let mut grown = PLANTS;
    if o.harvested {
        grown.picked = HARVESTED.picked;
        sink[0] = HARVESTED.sink * height;
    }
    stages(
        plants,
        "grow",
        grown.picked,
        grown.growing,
        grown.planted,
        Stages {
            sink: Some((sink[0], sink[1], sink[2])),
            ..Stages::default()
        },
    );
    stages(
        fruit,
        "grow",
        CROP.picked,
        CROP.growing,
        CROP.planted,
        Stages::default(),
    );
    vec![still, plants, fruit]
}

/// A raised bed of boards (the palette `p`'s `body`, the part the player colours), two to a side, their arrises
/// chamfered (`AUTO_BEVEL`), with corner posts, full of soil mounded into rows: returns the soil's height.
pub fn raised_bed(p: &Palette, w: f64, d: f64, h: f64) -> f64 {
    let body = p.get("body");
    for s in [-1, 1] {
        for k in 0..2 {
            cube(
                format!("board{s}_{k}"),
                (w - 0.08, 0.04, h / 2.0 - 0.004),
                Cube {
                    at: P3 {
                        x: 0.0,
                        y: s as f64 * (d / 2.0 - 0.02),
                        z: h / 4.0 + k as f64 * h / 2.0,
                    },
                    material: Some(body),
                    bevel: AUTO_BEVEL,
                    ..Cube::default()
                },
            );
            cube(
                format!("board_end{s}_{k}"),
                (0.04, d - 0.08, h / 2.0 - 0.004),
                Cube {
                    at: P3 {
                        x: s as f64 * (w / 2.0 - 0.02),
                        y: 0.0,
                        z: h / 4.0 + k as f64 * h / 2.0,
                    },
                    material: Some(body),
                    bevel: AUTO_BEVEL,
                    ..Cube::default()
                },
            );
        }
    }
    for x in [-1, 1] {
        for y in [-1, 1] {
            cube(
                format!("post{x}{y}"),
                (0.05, 0.05, h + 0.02),
                Cube {
                    at: P3 {
                        x: x as f64 * (w / 2.0 - 0.025),
                        y: y as f64 * (d / 2.0 - 0.025),
                        z: (h + 0.02) / 2.0,
                    },
                    material: Some(body),
                    bevel: low(0.005, 0.0),
                    bevel_segments: 1,
                    ..Cube::default()
                },
            );
        }
    }
    cube(
        "soil",
        (w - 0.08, d - 0.08, 0.03),
        Cube {
            at: P3 {
                x: 0.0,
                y: 0.0,
                z: h - 0.015,
            },
            material: Some(p.get("soil")),
            ..Cube::default()
        },
    );
    h
}

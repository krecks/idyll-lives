//! The lawn's wildflowers and its drifts: daisies, buttercups, violets and fallen petals.

use super::M;
use super::common::{_mesh, SKY, blade, stem};
use crate::plants::common::options;
use artkit::far::{is_low, low};
use artkit::materials::Mat;
use artkit::math::{Axis, M4, TAU, V3, atan2, cos, degrees, pow, radians, sin, sqrt, v3};
use artkit::parts::{Settle, join_all, settle, settle_with};
use artkit::rand::Random;
use artkit::scene::Obj;

options! {
    /// A flower head's shape (`flower`, `_head`).
    Head { n: usize = 5, notch: f64 = 0.8, cup: f64 = 0.0, disc: f64 = 0.32, dome: f64 = 0.011 }
}

/// A flower head of `n` petals at `at` (a Blender point, the top of its stem) turned up `tilt` degrees towards
/// `yaw`: petals round a raised eye on top (its own colour, `disc` of the radius across, domed `dome` high), a
/// shallow cone underneath, one closed shell of 6n triangles. `notch`: how far out the notches between the petals
/// are (0.8: broad petals; 0.4: a daisy's separate rays); `cup`: how far the petals' tips rise (a buttercup's
/// bowl). A far version keeps the petal tips (at most eight) and leaves out the eye: 2 a tip.
#[allow(clippy::too_many_arguments)]
pub fn flower(
    name: &str,
    at: V3,
    radius: f64,
    tilt: f64,
    yaw: f64,
    petals: Mat,
    eye: Mat,
    o: Head,
) -> Obj {
    let Head {
        n,
        notch,
        cup,
        disc,
        dome,
    } = o;
    let full = low(true, false);
    let tips = if full { n } else { n.min(8) };
    let mut rim = Vec::new();
    for i in 0..tips * if full { 2 } else { 1 } {
        let a = TAU * i as f64 / if full { tips * 2 } else { tips } as f64;
        let tip = i % 2 == 0 || !full;
        let r = if tip { radius } else { radius * notch };
        rim.push(v3(
            r * cos(a),
            r * sin(a),
            if tip { 0.004 + cup } else { cup * 0.6 },
        ));
    }
    let turn = M4::rotation(radians(yaw), Axis::Z) * M4::rotation(radians(tilt), Axis::Y);
    let place = M4::translation(at) * turn;
    let (verts, mut faces, mut colours): (Vec<V3>, Vec<Vec<u32>>, Vec<u16>);
    let n = n as u32;
    if full {
        let inner: Vec<V3> = (0..n)
            .map(|i| {
                v3(
                    radius * disc * cos(TAU * i as f64 / n as f64),
                    radius * disc * sin(TAU * i as f64 / n as f64),
                    0.005 + cup * 0.3,
                )
            })
            .collect();
        let apex = v3(0, 0, dome + cup * 0.3);
        let bottom = v3(0, 0, -0.008);
        let rims = rim.len() as u32;
        verts = [rim, inner, vec![apex, bottom]].concat();
        let (r0, i0, a0, b0) = (0, rims, rims + n, rims + n + 1);
        (faces, colours) = (Vec::new(), Vec::new());
        for i in 0..n {
            let j = (i + 1) % n;
            // the petals: each eye edge to its two rim segments (tip, notch, next tip)
            faces.extend([
                vec![i0 + i, r0 + 2 * i, r0 + 2 * i + 1],
                vec![i0 + i, r0 + 2 * i + 1, i0 + j],
                vec![i0 + j, r0 + 2 * i + 1, r0 + (2 * i + 2) % (2 * n)],
            ]);
            colours.extend([0, 0, 0]);
            faces.push(vec![a0, i0 + i, i0 + j]);
            colours.push(1);
        }
        for i in 0..2 * n {
            faces.push(vec![b0, r0 + (i + 1) % (2 * n), r0 + i]);
            colours.push(0);
        }
    } else {
        let (top, bottom) = (v3(0, 0, 0.006 + cup * 0.5), v3(0, 0, -0.008));
        let tips = tips as u32;
        verts = [rim, vec![top, bottom]].concat();
        let (t, b) = (tips, tips + 1);
        (faces, colours) = (Vec::new(), Vec::new());
        for i in 0..tips {
            faces.extend([vec![t, i, (i + 1) % tips], vec![b, (i + 1) % tips, i]]);
            colours.extend([0, 0]);
        }
    }
    let placed: Vec<V3> = verts.iter().map(|&v| place * v).collect();
    let obj = _mesh(name, placed, &faces, petals, true, 0.0, true);
    if full {
        paint(obj, eye, &colours);
    }
    obj
}

/// Puts the faces numbered 1 in `colours` on `second`, the part's other material.
fn paint(obj: Obj, second: Mat, colours: &[u16]) {
    obj.edit(|o| {
        let slot = o.mesh.slot(second);
        for (face, &c) in o.mesh.faces.iter_mut().zip(colours) {
            face.mat = if c == 0 { 0 } else { slot };
        }
    });
}

/// A closed flower bud on its stem's top `at` (Blender): green sepals cupping it from below, the white of its
/// petals showing at the top, five sides round, `h` long, leaning `tilt` degrees towards `yaw`: 15 triangles.
#[allow(clippy::too_many_arguments)]
pub fn bud(
    name: &str,
    at: V3,
    r: f64,
    h: f64,
    tilt: f64,
    yaw: f64,
    sepals: Mat,
    petals: Mat,
) -> Obj {
    let turn = M4::rotation(radians(yaw), Axis::Z) * M4::rotation(radians(tilt), Axis::Y);
    let place = M4::translation(at) * turn;
    let ring: Vec<V3> = (0..5)
        .map(|i| {
            v3(
                r * cos(TAU * i as f64 / 5.0),
                r * sin(TAU * i as f64 / 5.0),
                h * 0.45,
            )
        })
        .collect();
    let neck: Vec<V3> = (0..5)
        .map(|i| {
            v3(
                r * 0.55 * cos(TAU * (i as f64 + 0.5) / 5.0),
                r * 0.55 * sin(TAU * (i as f64 + 0.5) / 5.0),
                h * 0.8,
            )
        })
        .collect();
    let verts = [ring, neck, vec![v3(0, 0, h), v3(0, 0, -0.002)]].concat();
    let (t, b) = (10, 11);
    let (mut faces, mut colours): (Vec<Vec<u32>>, Vec<u16>) = (Vec::new(), Vec::new());
    for i in 0..5 {
        let j = (i + 1) % 5;
        faces.extend([vec![b, j, i], vec![i, j, 5 + i]]);
        colours.extend([0, 0]);
        faces.extend([vec![j, 5 + j, 5 + i], vec![5 + i, 5 + j, t]]);
        colours.extend([0, 1]);
    }
    let placed: Vec<V3> = verts.iter().map(|&v| place * v).collect();
    let obj = _mesh(name, placed, &faces, sepals, true, 0.0, true);
    paint(obj, petals, &colours);
    obj
}

/// Wildflowers: five low leaves, each folded along its midrib, round three stems with an ox-eye daisy, a
/// buttercup and a cranesbill where the old part's three heads stand, and a daisy's bud on a fourth, every part on
/// `foliage`, in one draw call.
pub fn flowers() -> Vec<Obj> {
    // Where the old part's three flowers stand: (game x, depth), height, petals, eye, and the head's shape: an
    // ox-eye daisy's thirteen white rays round a broad yellow disc, a buttercup's five glossy yellow petals cupped
    // up, a cranesbill's five broad violet ones, flat, round a pale eye.
    let heads = [
        (
            (0.03, 0.02),
            0.26,
            M.petal_white,
            M.eye,
            Head {
                n: 13,
                notch: 0.42,
                disc: 0.36,
                dome: 0.009,
                ..Head::default()
            },
        ),
        (
            (-0.04, 0.03),
            0.22,
            M.petal_yellow,
            M.eye_orange,
            Head {
                n: 5,
                notch: 0.82,
                cup: 0.008,
                disc: 0.28,
                ..Head::default()
            },
        ),
        (
            (0.0, -0.05),
            0.19,
            M.petal_violet,
            M.eye_pale,
            Head {
                n: 5,
                notch: 0.86,
                disc: 0.26,
                dome: 0.007,
                ..Head::default()
            },
        ),
    ];
    let mut rng = Random::new(42);
    for i in 0..5 {
        let yaw = i as f64 * 72.0 + rng.uniform(-20.0, 20.0);
        let (lean, bend, length, width) = (
            rng.uniform(20.0, 28.0),
            rng.uniform(16.0, 24.0),
            rng.uniform(0.12, 0.15),
            rng.uniform(0.034, 0.042),
        );
        blade(
            format!("leaf{i}"),
            (cos(radians(yaw)) * 0.006, sin(radians(yaw)) * 0.006),
            yaw,
            lean,
            bend,
            length,
            width,
            M.leaf,
            0.5,
            true,
            0.22,
        );
    }
    for (i, ((x, z), h, petals, eye, shape)) in heads.into_iter().enumerate() {
        let top = v3(-x, -z, h);
        stem(
            format!("stem{i}"),
            v3(-x * 0.3, -z * 0.3, 0),
            top,
            0.0045,
            M.stem,
            0.012,
        );
        let tilt = rng.uniform(8.0, 22.0);
        let yaw = degrees(atan2(-z, -x)) + rng.uniform(-30.0, 30.0);
        flower(
            &format!("flower{i}"),
            top,
            0.037,
            tilt,
            yaw,
            petals,
            eye,
            shape,
        );
    }
    // the bud, nodding on its own stem (left out far off; its numbers drawn either way)
    let (tilt, yaw) = (rng.uniform(25.0, 35.0), rng.uniform(-30.0, 30.0));
    if low(true, false) {
        let top = v3(-0.045, 0.035, 0.15);
        stem("stem_bud", v3(-0.012, 0.01, 0), top, 0.0035, M.stem, 0.008);
        bud(
            "bud",
            top,
            0.008,
            0.022,
            tilt,
            180.0 + yaw,
            M.leaf,
            M.petal_white,
        );
    }
    settle(&[join_all("flowers")])
}

// ---- the lawn's drifts -----------------------------------------------------------------------------------------

/// A patch of one kind of lawn flower, as daisies stand in a lawn: `heads` small heads (their `heights` over
/// the ground, from and to: at the tips of a mown lawn's blades) scattered over a disc `spread` across, none on
/// another, each tilted its own way a little. No leaves: they are down among the grass. The game stands patches by
/// the hundred where the lawn's flowers gather (render/shared/nature.ts `lawns`), so it is light: a head under 3
/// triangles a petal (`_head`) and no stem, which the blades round it would hide (a sliver of one drew as a dark
/// mark beside each head); a far version's heads a fan each.
#[allow(clippy::too_many_arguments)]
fn _drift(
    name: &str,
    seed: i128,
    petals: Mat,
    eye: Mat,
    heads: usize,
    spread: f64,
    radius: f64,
    heights: (f64, f64),
    shape: Head,
) -> Vec<Obj> {
    let mut rng = Random::new(seed);
    let mut spots = Vec::new();
    // spread evenly (a sunflower's spiral), then nudged
    for i in 0..heads {
        let a = i as f64 * 2.39996 + rng.uniform(-0.5, 0.5);
        let r = spread / 2.0 * sqrt((i as f64 + rng.uniform(0.3, 0.9)) / heads as f64);
        spots.push((r * cos(a), r * sin(a)));
    }
    for (i, (x, y)) in spots.into_iter().enumerate() {
        let h = rng.uniform(heights.0, heights.1);
        let lean = (rng.uniform(-0.012, 0.012), rng.uniform(-0.012, 0.012));
        let top = v3(x + lean.0, y + lean.1, h);
        let (r, tilt, yaw) = (
            radius * rng.uniform(0.85, 1.15),
            rng.uniform(3.0, 12.0),
            rng.uniform(0.0, 360.0),
        );
        _head(
            &format!("{name}_head{i}"),
            top,
            r,
            tilt,
            yaw,
            petals,
            eye,
            shape.clone(),
        );
    }
    settle_with(
        &[join_all(name)],
        Settle {
            centre: true,
            floor: false,
        },
    )
}

/// A lawn flower's head, as `flower` shapes one, for looking down on, by the thousand: its top only (the lawn is
/// under it), the petals one fan from its middle, the eye a flat polygon laid on them, and every face lit as the
/// sky sees it (`SKY`), so its petals are one light colour whichever way they lean: shaded each by its own slope,
/// half of a head 4 px across drew dark. 3n - 2 triangles; n in a far version (no eye, no notches).
#[allow(clippy::too_many_arguments)]
fn _head(
    name: &str,
    at: V3,
    radius: f64,
    tilt: f64,
    yaw: f64,
    petals: Mat,
    eye: Mat,
    o: Head,
) -> Obj {
    let Head {
        n,
        notch,
        cup,
        disc,
        dome,
    } = o;
    let full = low(true, false);
    let turn = M4::rotation(radians(yaw), Axis::Z) * M4::rotation(radians(tilt), Axis::Y);
    let place = M4::translation(at) * turn;
    let (verts, faces, colours): (Vec<V3>, Vec<Vec<u32>>, Vec<u16>);
    if full {
        let rim: Vec<V3> = (0..2 * n)
            .map(|i| {
                let r = if i % 2 == 0 { radius } else { radius * notch };
                v3(
                    r * cos(TAU * i as f64 / (2 * n) as f64),
                    r * sin(TAU * i as f64 / (2 * n) as f64),
                    if i % 2 == 0 { 0.004 + cup } else { cup * 0.6 },
                )
            })
            .collect();
        let middle = v3(0, 0, cup * 0.3);
        let top: Vec<V3> = (0..n)
            .map(|i| {
                v3(
                    radius * disc * cos(TAU * (i as f64 + 0.5) / n as f64),
                    radius * disc * sin(TAU * (i as f64 + 0.5) / n as f64),
                    dome * 0.6 + cup * 0.3,
                )
            })
            .collect();
        verts = [rim, vec![middle], top].concat();
        let n = n as u32;
        let (c, e) = (2 * n, 2 * n + 1);
        let mut all: Vec<Vec<u32>> = (0..2 * n).map(|i| vec![c, i, (i + 1) % (2 * n)]).collect();
        all.extend((1..n - 1).map(|i| vec![e, e + i, e + i + 1]));
        faces = all;
        colours = [vec![0; 2 * n as usize], vec![1; n as usize - 2]].concat();
    } else {
        let rim: Vec<V3> = (0..n)
            .map(|i| {
                v3(
                    radius * 0.9 * cos(TAU * i as f64 / n as f64),
                    radius * 0.9 * sin(TAU * i as f64 / n as f64),
                    0.004 + cup,
                )
            })
            .collect();
        // (its middle where the model's is: the gate measures the lowest point)
        verts = [rim, vec![v3(0, 0, cup * 0.3)]].concat();
        let n = n as u32;
        faces = (0..n).map(|i| vec![n, i, (i + 1) % n]).collect();
        colours = vec![0; n as usize];
    }
    let placed: Vec<V3> = verts.iter().map(|&v| place * v).collect();
    let obj = _mesh(name, placed, &faces, petals, false, SKY, false);
    if full {
        paint(obj, eye, &colours);
    }
    obj
}

/// A patch of lawn daisies: white rays round a yellow eye.
pub fn daisies() -> Vec<Obj> {
    _drift(
        "daisies",
        7,
        M.petal_white,
        M.eye,
        6,
        0.66,
        0.043,
        (0.045, 0.08),
        Head {
            n: low(7, 6),
            notch: 0.66,
            disc: 0.36,
            dome: 0.009,
            ..Head::default()
        },
    )
}

/// A patch of creeping buttercups: five glossy yellow petals, cupped.
pub fn buttercups() -> Vec<Obj> {
    _drift(
        "buttercups",
        11,
        M.petal_yellow,
        M.eye_orange,
        5,
        0.6,
        0.037,
        (0.05, 0.09),
        Head {
            n: 5,
            notch: 0.84,
            cup: 0.007,
            disc: 0.26,
            ..Head::default()
        },
    )
}

/// A patch of low violet flowers (a selfheal's or a speedwell's blue-violet), a pale eye in each.
pub fn violets() -> Vec<Obj> {
    _drift(
        "violets",
        19,
        M.petal_violet,
        M.eye_pale,
        6,
        0.54,
        0.034,
        (0.04, 0.075),
        Head {
            n: 5,
            notch: 0.86,
            disc: 0.24,
            dome: 0.006,
            ..Head::default()
        },
    )
}

/// Fallen petals under a tree in blossom (render/shared/nature.ts `natureDecor`): pale pink flakes lying on the
/// lawn's tips over a disc 5 m across (at a tree's scale 1), most of them under the crown's middle, each a flat
/// rhombus tipped a little its own way, lit as the lawn is, larger than life as the game's flowers are (a petal
/// under 6 cm did not show from the play camera): 2 triangles; a far version's half as many, 1 each.
pub fn petals() -> Vec<Obj> {
    let mut rng = Random::new(23);
    let (mut verts, mut faces): (Vec<V3>, Vec<Vec<u32>>) = (Vec::new(), Vec::new());
    for i in 0..340 {
        let (a, r) = (rng.uniform(0.0, TAU), 2.5 * pow(rng.random(), 0.62));
        let (long, wide) = (rng.uniform(0.11, 0.15), rng.uniform(0.075, 0.1));
        let place = M4::translation(v3(r * cos(a), r * sin(a), rng.uniform(0.07, 0.1)));
        let spin = M4::rotation(rng.uniform(0.0, TAU), Axis::Z);
        let tip = M4::rotation(radians(rng.uniform(-14.0, 14.0)), Axis::Y);
        let turn = place * spin * tip;
        if is_low() && i % 2 != 0 {
            continue;
        }
        let k = verts.len() as u32;
        verts.extend(
            [
                v3(long / 2.0, 0, 0),
                v3(0, wide / 2.0, 0),
                v3(-long / 2.0, 0, 0),
                v3(0, -wide / 2.0, 0),
            ]
            .map(|v| turn * v),
        );
        faces.push(vec![k, k + 1, k + 2]);
        if !is_low() {
            faces.push(vec![k, k + 2, k + 3]);
        }
    }
    _mesh("petals", verts, &faces, M.petal_pink, false, SKY, false);
    settle_with(
        &[join_all("petals")],
        Settle {
            centre: true,
            floor: false,
        },
    )
}

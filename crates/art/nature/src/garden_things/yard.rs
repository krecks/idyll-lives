//! The dog house, the sundial and the stepping stones.

use super::common::{_own, _prism, F, M};
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{TAU, V3, atan2, cos, cosd, degrees, pow, sin, sind, v3};
use artkit::noise::noise;
use artkit::parts::{Settle, join_all, numbered, settle_all, settle_with};
use artkit::place::Facing;
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{boulder, fbox, fcyl, fextrude, fsphere, fsweep, revolve, rod, sweep, torus};

// --- the dog house (1x1): a red kennel with a shingled roof, a name over the arched door and a water bowl -----
/// A letter of the dog's name in strokes, `size` tall, its middle at game (x, y), on the face at depth z. Game x
/// runs to the viewer's left seen from the front, so the strokes and the letters' order are mirrored to read
/// right.
fn _letter(ch: char, x: f64, y: f64, z: f64, size: f64, material: Mat) {
    let (s, t) = (size, 0.008);
    type Stroke = ((f64, f64), (f64, f64));
    let strokes: &[Stroke] = match ch {
        'R' => &[
            ((-0.5, -0.5), (-0.5, 0.5)),
            ((-0.5, 0.5), (0.3, 0.5)),
            ((0.3, 0.5), (0.4, 0.25)),
            ((0.4, 0.25), (0.3, 0.0)),
            ((0.3, 0.0), (-0.5, 0.0)),
            ((-0.1, 0.0), (0.4, -0.5)),
        ],
        'E' => &[
            ((-0.4, -0.5), (-0.4, 0.5)),
            ((-0.4, 0.5), (0.4, 0.5)),
            ((-0.4, 0.0), (0.25, 0.0)),
            ((-0.4, -0.5), (0.4, -0.5)),
        ],
        'X' => &[((-0.45, -0.5), (0.45, 0.5)), ((-0.45, 0.5), (0.45, -0.5))],
        _ => panic!("no strokes for {ch:?}"),
    };
    for &((ax, ay), (bx, by)) in strokes {
        let a: V3 = F.at(x - ax * s * 0.8, y + ay * s, z).into();
        let b: V3 = F.at(x - bx * s * 0.8, y + by * s, z).into();
        rod!(
            numbered("stroke"),
            a,
            b,
            t / 2.0,
            material = material,
            verts = 4
        );
    }
}

#[allow(non_snake_case)]
pub fn dogHouse() -> Vec<Obj> {
    let m = _own("dogHouse");
    let (zc, d) = (-0.03, 0.84); // the old part's body: 0.8 wide, centred 3-4 cm back
    let front = zc + d / 2.0;
    let slope = degrees(atan2(0.28, 0.4));
    let (cs, sn) = (cosd(slope), sind(slope));
    let mut rng = Random::new(7);
    fbox!(
        F,
        "base",
        0.84,
        0.04,
        0.9,
        (0, 0.02, zc),
        m.plinth,
        bevel = 0.008,
        bevel_segments = 1
    );
    _prism(
        "house",
        &[
            (-0.4, 0.04),
            (0.4, 0.04),
            (0.4, 0.62),
            (0.0, 0.9),
            (-0.4, 0.62),
        ],
        zc - d / 2.0,
        front,
        m.body,
    );
    // lapped boards down the sides and the back: each a strip proud of the wall, its lower edge out a little
    for k in 0..7 {
        let y = 0.08 + k as f64 * 0.08;
        for i in [-1, 1] {
            fbox!(
                F,
                format!("side_board{i}{k}"),
                0.008,
                0.082,
                d - 0.04,
                (i as f64 * 0.403, y, zc),
                m.body,
                rot = (0, i * 3, 0)
            );
        }
        fbox!(
            F,
            format!("back_board{k}"),
            0.76,
            0.082,
            0.008,
            (0, y, zc - d / 2.0 - 0.003),
            m.body,
            rot = (3, 0, 0)
        );
    }
    // the front: a gable board with the arched doorway through it, its boards' laps either side of the door
    let gable = [
        (-0.4, 0.04),
        (0.4, 0.04),
        (0.4, 0.62),
        (0.0, 0.9),
        (-0.4, 0.62),
    ];
    let mut arch = vec![(0.15, 0.04), (0.15, 0.3)];
    arch.extend((15..180).step_by(15).map(|a| {
        let a = a as f64;
        (0.15 * cosd(a), 0.3 + 0.15 * sind(a))
    }));
    arch.extend([(-0.15, 0.3), (-0.15, 0.04)]);
    let mut hole = vec![(0.15, 0.045)];
    hole.extend_from_slice(&arch[1..arch.len() - 1]);
    hole.push((-0.15, 0.045));
    fextrude!(
        F,
        "front",
        &gable,
        0.012,
        (0, 0, front + 0.006),
        m.body,
        holes = [hole]
    );
    for k in 1..9 {
        let y = 0.04 + k as f64 * 0.08;
        let half = if y <= 0.62 {
            0.4
        } else {
            0.4 * (0.9 - y) / 0.28
        };
        if half < 0.05 {
            continue;
        }
        let mut spans = if y > 0.46 {
            vec![(-half, half)]
        } else {
            vec![(-half, -0.17), (0.17, half)]
        };
        if 0.3 < y && y <= 0.46 {
            let gap = pow(pow(0.15, 2.0) - pow(y - 0.3, 2.0), 0.5) + 0.02;
            spans = vec![(-half, -gap), (gap, half)];
        }
        for (i, (a, b)) in spans.into_iter().enumerate() {
            fbox!(
                F,
                format!("lap{k}{i}"),
                b - a,
                0.006,
                0.004,
                ((a + b) / 2.0, y, front + 0.013),
                m.body
            );
        }
    }
    // the dark inside, seen through the doorway, and a trim round the arch
    fbox!(F, "door", 0.3, 0.42, 0.01, (0, 0.25, front - 0.002), m.dark);
    let trim: Vec<(f64, f64, f64)> = arch.iter().map(|&(x, y)| (x, y, front + 0.016)).collect();
    fsweep!(
        F,
        "door_trim",
        &trim,
        0.0,
        m.plate,
        profile = rounded_rect(0.012, 0.036, 0.004, 1),
        up = (0, -1, 0)
    );
    fbox!(
        F,
        "sill",
        0.36,
        0.025,
        0.05,
        (0, 0.05, front + 0.015),
        m.plate,
        bevel = 0.006,
        bevel_segments = 1
    );
    // corner boards
    for i in [-1, 1] {
        let s = i as f64;
        for j in [-1, 1] {
            fbox!(
                F,
                format!("corner{i}{j}"),
                0.04,
                0.58,
                0.04,
                (s * 0.395, 0.33, zc + j as f64 * (d / 2.0 - 0.005)),
                m.plate,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        // the bargeboards up the front gable's edges
        sweep!(
            format!("barge{i}"),
            &[
                F.at(s * 0.47, 0.885 - 0.47 * 0.7, front + 0.035),
                F.at(0, 0.885, front + 0.035)
            ],
            0.0,
            material = m.plate,
            profile = [(-0.01, -0.03), (0.01, -0.03), (0.01, 0.03), (-0.01, 0.03)],
            up = (0, -1, 0)
        );
    }
    // the roof: an underlay board a slope, four courses of shingles lapped over it, staggered, and a ridge cap
    let down = 0.47 / cs; // down each slope, from the ridge to the eave
    for i in [-1, 1] {
        let s = i as f64;
        // The game (x, height) `t` down the slope from the ridge, `out` off the walls' line.
        let on = |t: f64, out: f64| (s * (t * cs + out * sn), 0.9 - t * sn + out * cs);
        let (x, y) = on(down / 2.0, 0.015);
        fbox!(
            F,
            format!("underlay{i}"),
            down,
            0.03,
            0.94,
            (x, y, zc),
            m.roof,
            rot = (0.0, -s * slope, 0.0)
        );
        for k in 0..4 {
            let t0 = down - k as f64 * 0.135;
            let (x, y) = on(t0 - 0.085, 0.038 + 0.009 * k as f64);
            let (width, off) = (0.94 / 6.0, if k % 2 != 0 { 0.94 / 12.0 } else { 0.0 });
            let mut edges: Vec<f64> = (-1..8)
                .map(|i| (-0.47_f64).max(0.47_f64.min(-0.47 + off + i as f64 * width)))
                .collect();
            edges.sort_by(f64::total_cmp);
            edges.dedup();
            for (n, w) in edges.windows(2).enumerate() {
                let (a, b) = (w[0], w[1]);
                if b - a < 0.02 {
                    continue;
                }
                let tilt = rng.uniform(-1.2, 1.2);
                // (a cedar shingle's butt, 11 mm: too thin to show an edge, and no chamfer needed)
                fbox!(
                    F,
                    format!("shingle{i}{k}{n}"),
                    0.17,
                    0.011,
                    b - a - 0.006,
                    (x, y, zc + (a + b) / 2.0),
                    m.roof,
                    rot = (tilt, -s * (slope + 2.0), 0.0)
                );
            }
        }
    }
    fsweep!(
        F,
        "ridge",
        &[(0, 0.948, zc - 0.47), (0, 0.948, zc + 0.47)],
        0.0,
        m.roof,
        profile = rounded_rect(0.05, 0.09, 0.022, 2),
        up = (0, 0, 1)
    );
    // the name plate in the gable
    fbox!(
        F,
        "plate",
        0.26,
        0.075,
        0.015,
        (0, 0.6, front + 0.016),
        m.plate,
        bevel = 0.006,
        bevel_segments = 1
    );
    for (i, ch) in "REX".chars().enumerate() {
        _letter(
            ch,
            (1.0 - i as f64) * 0.06,
            0.6,
            front + 0.026,
            0.042,
            m.dark,
        );
    }
    // the water bowl by the door, and a chew bone on the plinth
    revolve!(
        "bowl",
        &[
            (0.0, 0.0),
            (0.055, 0.0),
            (0.065, 0.04),
            // (its lip rolled: no turn of 60°)
            (0.063, 0.044),
            (0.0595, 0.0452),
            (0.0565, 0.0435),
            (0.05, 0.012),
            (0.0, 0.012)
        ],
        at = F.at(0.29, 0, 0.43),
        material = m.bowl,
        segments = 18
    );
    fcyl!(
        F,
        "bowl_water",
        0.052,
        0.004,
        (0.29, 0.034, 0.43),
        m.bowl_water,
        verts = 18
    );
    fsweep!(
        F,
        "bone",
        &[(-0.3, 0.05, 0.4), (-0.22, 0.05, 0.43)],
        0.009,
        m.plate,
        verts = 6
    );
    for (k, (bx, bz)) in [(-0.3, 0.4), (-0.22, 0.43)].into_iter().enumerate() {
        for j in [-1, 1] {
            let e = j as f64;
            fsphere!(
                F,
                format!("bone_knob{k}{j}"),
                0.013,
                (bx + e * 0.006, 0.052, bz - e * 0.012),
                m.plate,
                segments = 8,
                rings = 5
            );
        }
    }
    // the ridge cap's sawn ends eased
    artkit::bevel::ease(artkit::scene::obj("ridge"), artkit::shapes::SHARP, 1);
    join_all("dogHouse");
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    meshes()
}

// --- the sundial (1x1): a copper dial with its gnomon on a stone baluster ---------------------------------------
pub fn sundial() -> Vec<Obj> {
    let m = _own("sundial");
    // two square steps, their top edges chamfered, a few patches of lichen on them
    fbox!(
        F,
        "step",
        0.36,
        0.045,
        0.36,
        (0, 0.0225, 0),
        m.body,
        bevel = 0.008,
        bevel_segments = 1
    );
    fbox!(
        F,
        "step2",
        0.3,
        0.03,
        0.3,
        (0, 0.06, 0),
        m.body,
        bevel = 0.006,
        bevel_segments = 1
    );
    for (i, (x, z, r)) in [
        (0.13, 0.12, 0.02),
        (-0.11, 0.15, 0.014),
        (0.12, -0.14, 0.017),
    ]
    .into_iter()
    .enumerate()
    {
        fcyl!(
            F,
            format!("lichen{i}"),
            r,
            0.002,
            (x, 0.045, z),
            m.lichen,
            verts = 7,
            scale = (1.3, 1, 1)
        );
    }
    // the baluster: a torus and a scotia at its foot, a swelling vase, a ringed neck and a spreading capital
    revolve!(
        "pedestal",
        &[
            (0.0, 0.075),
            (0.125, 0.075),
            (0.125, 0.095),
            (0.135, 0.105),
            (0.13, 0.12),
            (0.11, 0.128),
            (0.095, 0.14),
            (0.1, 0.155),
            (0.09, 0.17),
            (0.085, 0.22),
            (0.098, 0.32),
            (0.1, 0.4),
            (0.088, 0.5),
            (0.068, 0.6),
            (0.06, 0.64),
            (0.072, 0.652),
            (0.072, 0.668),
            (0.06, 0.678),
            (0.066, 0.7),
            (0.09, 0.73),
            (0.125, 0.765),
            (0.14, 0.78),
            (0.14, 0.79),
            (0.0, 0.79)
        ],
        at = F.at(0, 0, 0),
        material = m.body,
        segments = 20
    );
    // a square cap stone, chamfered, the dial on it
    fbox!(
        F,
        "cap",
        0.32,
        0.05,
        0.32,
        (0, 0.815, 0),
        m.body,
        bevel = 0.012,
        bevel_segments = 1
    );
    fbox!(
        F,
        "cap_top",
        0.29,
        0.012,
        0.29,
        (0, 0.846, 0),
        m.body,
        bevel = 0.004,
        bevel_segments = 1
    );
    fcyl!(
        F,
        "dial",
        0.15,
        0.012,
        (0, 0.856, 0),
        m.dial,
        verts = 28,
        bevel = 0.002,
        bevel_segments = 1
    );
    torus!(
        "dial_rim",
        0.146,
        0.006,
        at = F.at(0, 0.862, 0),
        material = m.dial,
        major_segments = 28,
        minor_segments = 4
    );
    torus!(
        "chapter_ring",
        0.088,
        0.0025,
        at = F.at(0, 0.862, 0),
        material = m.dial_mark,
        major_segments = 24,
        minor_segments = 3
    );
    // the hour marks, longer at the quarters, a dot at each half hour
    for a in (0..360).step_by(30) {
        let r = if a % 90 != 0 { 0.115 } else { 0.11 };
        let deg = a as f64;
        fbox!(
            F,
            format!("mark{a}"),
            0.008,
            0.003,
            if a % 90 != 0 { 0.03 } else { 0.04 },
            (r * sind(deg), 0.8635, r * cosd(deg)),
            m.dial_mark,
            rot = (0, 0, -a)
        );
        fcyl!(
            F,
            format!("dot{a}"),
            0.004,
            0.003,
            (0.118 * sind(deg + 15.0), 0.8635, 0.118 * cosd(deg + 15.0)),
            m.dial_mark,
            verts = 5
        );
    }
    // a compass rose in the middle: an eight-pointed star
    let rose: Vec<(f64, f64)> = (0..16)
        .map(|k| {
            let r = if k % 4 == 0 {
                0.06
            } else if k % 2 == 0 {
                0.035
            } else {
                0.012
            };
            (r * cosd(22.5 * k as f64), r * sind(22.5 * k as f64))
        })
        .collect();
    fextrude!(
        F,
        "rose",
        &rose,
        0.002,
        (0, 0.8625, 0),
        m.dial_mark,
        plane = Facing::Top
    );
    fcyl!(
        F,
        "boss",
        0.014,
        0.008,
        (0, 0.866, 0),
        m.dial_mark,
        verts = 10,
        bevel = 0.002,
        bevel_segments = 1
    );
    // the gnomon: a plate along the noon line, its edge at the latitude's angle, pierced
    let hole: Vec<(f64, f64)> = (0..360)
        .step_by(36)
        .map(|a| {
            (
                -0.055 + 0.018 * cosd(a as f64),
                0.03 + 0.018 * sind(a as f64),
            )
        })
        .collect();
    fextrude!(
        F,
        "gnomon",
        &[(0.11, 0.0), (-0.1, 0.0), (-0.1, 0.1), (-0.085, 0.1)],
        0.008,
        (0, 0.862, 0),
        m.dial,
        plane = Facing::Side,
        holes = [hole]
    );
    join_all("sundial");
    settle_all();
    meshes()
}

// --- the stepping stones (1x3, rug layer): four flat stones set in the lawn -----------------------------------
const STONES: [(f64, f64, f64, i32); 4] = [
    (0.05, -1.05, 0.52, 10),
    (-0.08, -0.35, 0.48, 40),
    (0.06, 0.35, 0.54, 70),
    (-0.04, 1.05, 0.5, 15),
];
const STONE_H: f64 = 0.04;

#[allow(non_snake_case)]
pub fn steppingStones() -> Vec<Obj> {
    let m = _own("steppingStones");
    let mut rng = Random::new(31);
    for (i, (x, z, d, r)) in STONES.into_iter().enumerate() {
        let n = i as f64;
        let s = boulder!(
            format!("stone{i}"),
            d / 2.0,
            at = F.at(x, 0, z),
            material = m.body,
            rot = (0, 0, r),
            seed = n * 3.1 + 0.4,
            noise = 0.24,
            subdivisions = 3
        );
        // flatten it into a slab: a worn, faintly uneven top with rounded edges, its bottom on the ground
        s.edit(|o| {
            let top = o.mesh.verts.iter().map(|v| v.z).fold(f32::MIN, f32::max) as f64;
            for v in &mut o.mesh.verts {
                let k = v.z as f64 / top;
                let edge = 1.0_f64.min(k * 2.2);
                let relief = noise(v3(v.x as f64 * 9.0 + n, v.y as f64 * 9.0 - n, 0.5));
                v.z = ((STONE_H - 0.0045) * pow(edge, 0.6)
                    + (0.0025 + 0.002 * relief) * pow(edge, 4.0)) as f32;
                v.y = (v.y as f64 * 0.86) as f32;
            }
        });
    }
    // moss creeping in at the stones' edges, a scatter of gravel between them
    for (i, (x, z, d, _)) in STONES.into_iter().enumerate() {
        for k in 0..3 {
            let a = rng.uniform(0.0, TAU);
            let rr = d / 2.0 * rng.uniform(0.75, 0.9);
            let mx = (-0.2_f64).max(0.2_f64.min(x + rr * cos(a)));
            let (radius, turn) = (rng.uniform(0.035, 0.045), rng.uniform(0.0, 180.0));
            boulder!(
                format!("moss{i}{k}"),
                radius,
                at = F.at(mx, 0.026, z + 0.86 * rr * sin(a)),
                material = m.moss,
                scale = (1.5, 1.0, 0.12),
                rot = (0.0, 0.0, turn),
                seed = (i * 5 + k) as f64,
                noise = 0.4,
                subdivisions = 1
            );
        }
    }
    for i in 0..14 {
        let (x, z) = (rng.uniform(-0.25, 0.25), rng.uniform(-1.2, 1.2));
        if STONES
            .iter()
            .any(|&(sx, sz, d, _)| (x - sx).abs() < d * 0.45 && (z - sz).abs() < d * 0.4)
        {
            continue;
        }
        boulder!(
            format!("gravel{i}"),
            rng.uniform(0.014, 0.022),
            at = F.at(x, 0, z),
            material = M().pebble,
            scale = (1.3, 1.0, 0.6),
            seed = i as f64 * 1.9,
            noise = 0.35,
            subdivisions = 1
        );
    }
    join_all("steppingStones");
    settle_all();
    meshes()
}

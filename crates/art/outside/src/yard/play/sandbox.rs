use super::super::common::{F, M, blob, ground, prism, rod};
use artkit::far::{is_low, low};
use artkit::kit;
use artkit::math::{Axis, P3, cosd, sind};
use artkit::motion::{Grow, grow};
use artkit::parts::numbered;
use artkit::parts::{join, join_all, new_parts};
use artkit::place::Facing;
use artkit::place::N;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcone, fcyl, fextrude, fsweep, loft, revolve, ring, settle};

// --- the sandpit (2x2): a box of boards round the sand, two corner seats, a bucket and spade; a castle grows ---
pub(super) const SAND: f64 = 0.18; // the sand's top
pub(super) const CASTLE: (f64, f64) = (-0.32, 0.18); // where the castle is built (x, depth)

/// A star's outline: `n` points out to r1, in to r0 between them.
pub(super) fn star(r0: f64, r1: f64, n: usize, turn: f64) -> Vec<(f64, f64)> {
    (0..2 * n)
        .map(|k| {
            let r = if k % 2 == 0 { r1 } else { r0 };
            let a = turn + (180 * k) as f64 / n as f64;
            (r * cosd(a), r * sind(a))
        })
        .collect()
}

pub fn sandbox() -> Vec<Obj> {
    // two boards high each side, lapped at the corners; a mitred capping frame round the top to sit on
    for y in [0.065, 0.195] {
        for z in [-0.92, 0.92] {
            fbox!(
                F,
                numbered("board"),
                1.92,
                0.125,
                0.08,
                (0, y, z),
                M.frame,
                bevel = low(0.01, 0.0),
                bevel_segments = 1
            );
        }
        for x in [-0.92, 0.92] {
            fbox!(
                F,
                numbered("board"),
                0.08,
                0.125,
                1.76,
                (x, y, 0),
                M.frame,
                bevel = low(0.01, 0.0),
                bevel_segments = 1
            );
        }
    }
    kit::moulding!(
        F,
        "cap",
        &[(0.99, 0.99), (-0.99, 0.99), (-0.99, -0.99), (0.99, -0.99)],
        &[
            (0.0, 0.0),
            (0.0, 0.024),
            (0.006, 0.03),
            (0.124, 0.03),
            (0.13, 0.024),
            (0.13, 0.0),
        ],
        M.frame,
        at = 0.26,
        plane = Facing::Top
    );
    for sx in [-1.0, 1.0] {
        // corner seats at the back: triangles across the corners
        let tri = [
            (sx * 0.86, -0.86),
            (sx * 0.86 - sx * 0.36, -0.86),
            (sx * 0.86, -0.86 + 0.36),
        ];
        fextrude!(
            F,
            numbered("seat"),
            &tri,
            0.03,
            (0, 0.3, 0),
            M.frame,
            plane = Facing::Top,
            bevel = low(0.008, 0.0),
            bevel_segments = 1
        );
    }
    ground(
        "sand",
        1.76,
        1.76,
        SAND - 0.012,
        M.sand,
        low((12, 12), (3, 3)),
        0.014,
        2.0,
        (0.0, 0.0),
        0.12,
    );
    blob(
        0.42,
        SAND - 0.03,
        -0.38,
        0.2,
        M.sand,
        3.0,
        (1.2, 1.0, 0.4),
        0.3,
    ); // a heap
    blob(
        -0.55,
        SAND - 0.02,
        -0.5,
        0.12,
        M.sand,
        5.0,
        (1.0, 1.0, 0.35),
        0.3,
    );
    // the bucket, full of sand, its handle down; the spade lying beside it
    let (bx, bz) = (0.3, 0.32);
    let p = F.at(bx, 0, bz);
    let (px, pz) = (p.x, p.y);
    let rings = [
        (0.0, 0.075),
        (0.15, 0.093),
        (0.15, 0.104),
        (0.17, 0.104),
        (0.17, 0.097),
        (0.16, 0.096),
    ];
    let secs: Vec<Vec<P3>> = rings
        .iter()
        .map(|&(h, r)| ring!(r, r, z = SAND + h, verts = low(16, 8), at = (px, pz)))
        .collect();
    loft!("bucket", &secs, material = M.bucket);
    fcyl!(
        F,
        "bucket_sand",
        0.096,
        0.01,
        (bx, SAND + 0.163, bz),
        M.sand,
        verts = low(16, 8)
    );
    revolve!(
        "bucket_mound",
        &[(0.0, 0.0), (0.09, 0.0), (0.06, 0.008), (0.0, 0.011)],
        at = F.at(bx, SAND + 0.165, bz),
        material = M.sand,
        segments = low(10, 5)
    );
    if !is_low() {
        let arc: Vec<(f64, f64, f64)> = (0..181)
            .step_by(30)
            .map(|a| {
                let a = a as f64;
                (
                    bx + 0.1 * cosd(a),
                    SAND + 0.165 - 0.06 * sind(a) * 0.3,
                    bz + 0.1 * sind(a) * 0.95,
                )
            })
            .collect();
        fsweep!(F, "bucket_handle", &arc, 0.004, M.spade, verts = 4);
    }
    // the spade: a T grip, its shaft and a blade with a rounded tip, lying on the sand
    let th = 18;
    let d = (sind(th as f64), cosd(th as f64));
    let g0 = (0.5, 0.0);
    let g1 = (g0.0 + 0.25 * d.0, g0.1 + 0.25 * d.1);
    let mut blade = vec![(-0.055, 0.0), (0.055, 0.0), (0.055, 0.08)];
    blade.extend(
        (30..151)
            .step_by(30)
            .map(|a| (0.055 * cosd(a as f64), 0.08 + 0.05 * sind(a as f64))),
    );
    blade.push((-0.055, 0.08));
    fextrude!(
        F,
        "spade_blade",
        &blade,
        0.008,
        (g1.0, SAND + 0.006, g1.1),
        M.spade,
        plane = Facing::Top,
        rot = (0, 0, -th)
    );
    fbox!(
        F,
        "spade_handle",
        0.025,
        0.02,
        0.25,
        ((g0.0 + g1.0) / 2.0, SAND + 0.012, (g0.1 + g1.1) / 2.0),
        M.spade,
        rot = (0, 0, -th),
        bevel = 0.006,
        bevel_segments = 1
    );
    fbox!(
        F,
        "spade_grip",
        0.09,
        0.022,
        0.025,
        (g0.0, SAND + 0.012, g0.1),
        M.spade,
        rot = (0, 0, -th),
        bevel = 0.008,
        bevel_segments = 1
    );
    // a star mould turned out, and the mould lying on its side
    fextrude!(
        F,
        "mould_turned",
        &star(0.03, 0.055, 5, 90.0),
        0.045,
        (-0.62, SAND + 0.0225, 0.55),
        M.sand_damp,
        plane = Facing::Top,
        bevel = low(0.006, 0.0),
        bevel_segments = 1
    );
    fextrude!(
        F,
        "mould",
        &star(0.032, 0.058, 5, 90.0),
        0.05,
        (-0.4, SAND + 0.025, 0.62),
        M.bucket,
        plane = Facing::Top,
        rot = (0, 0, 15),
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
    if is_low() {
        return settle!(&[join_all("sandbox")], centre = false);
    }
    join(&meshes(), "sandbox", Some(P3::from((0, 0, 0))));
    // The castle: a keep and four turrets turned out of the bucket, walls between them, a flag. Its origin is 2 cm
    // down in the sand, so flattened at rest it is out of sight; `castle` raises it.
    let (cx, cz) = CASTLE;
    let castle = new_parts(|| {
        fcone!(
            F,
            "keep",
            0.1,
            0.075,
            0.17,
            (cx, SAND + 0.085, cz),
            M.sand_damp,
            verts = 16
        );
        for k in 0..8 {
            let a = 45 * k;
            fbox!(
                F,
                numbered("merlon"),
                0.03,
                0.03,
                0.025,
                (
                    cx + 0.062 * cosd(a as f64),
                    SAND + 0.183,
                    cz + 0.062 * sind(a as f64)
                ),
                M.sand_damp,
                rot = (0, 0, a)
            );
        }
        fcyl!(
            F,
            "keep_top",
            0.06,
            0.012,
            (cx, SAND + 0.172, cz),
            M.sand_damp,
            verts = 12
        );
        for k in 0..3 {
            // the bucket's ridges, pressed into the keep
            fcyl!(
                F,
                numbered("keep_ring"),
                0.1 - 0.025 * (k as f64 + 0.5) / 3.0 * 1.0 + 0.003,
                0.008,
                (cx, SAND + 0.03 + 0.05 * k as f64, cz),
                M.sand_damp,
                verts = 16
            );
        }
        fbox!(
            F,
            "keep_door",
            0.04,
            0.05,
            0.02,
            (cx, SAND + 0.025, cz + 0.092),
            M.sand,
            bevel = 0.006,
            bevel_segments = 1
        );
        for sx in [-1.0, 1.0] {
            for sz in [-1.0, 1.0] {
                let (x, z) = (cx + sx * 0.15, cz + sz * 0.15);
                fcone!(
                    F,
                    numbered("turret"),
                    0.055,
                    0.042,
                    0.11,
                    (x, SAND + 0.055, z),
                    M.sand_damp,
                    verts = 12
                );
                fcone!(
                    F,
                    numbered("turret_top"),
                    0.046,
                    0.0,
                    0.06,
                    (x, SAND + 0.14, z),
                    M.sand_damp,
                    verts = 12
                );
            }
        }
        for s in [-1.0, 1.0] {
            fbox!(
                F,
                numbered("wall"),
                0.24,
                0.06,
                0.03,
                (cx, SAND + 0.03, cz + s * 0.15),
                M.sand_damp,
                bevel = 0.006
            );
            fbox!(
                F,
                numbered("wall"),
                0.03,
                0.06,
                0.24,
                (cx + s * 0.15, SAND + 0.03, cz),
                M.sand_damp,
                bevel = 0.006
            );
        }
        fbox!(
            F,
            "gate",
            0.07,
            0.045,
            0.01,
            (cx, SAND + 0.023, cz + 0.166),
            M.sand,
            bevel = 0.003
        );
        rod(
            "flag_pole",
            (cx, SAND + 0.17, cz),
            (cx, SAND + 0.31, cz),
            0.004,
            M.haft,
            5,
        );
        prism(
            "flag",
            &[
                (N::F(cx), SAND + 0.3),
                (N::F(cx), SAND + 0.25),
                (N::F(cx + 0.08), SAND + 0.275),
            ],
            cz - 0.002,
            cz + 0.002,
            M.flag,
        );
    });
    let castle = join(&castle, "castle", Some(F.at(cx, SAND - 0.02, cz)));
    castle.edit(|o| o.scale.z = 0.001);
    let objs = settle!(centre = false);
    grow(
        castle,
        "castle",
        Grow {
            seconds: 2.5,
            axis: Axis::Z,
        },
    );
    objs
}

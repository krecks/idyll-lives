//! KD2: the train table.

use super::common::{P, arc, place, still, walls};
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::json::float_repr as r;
use artkit::materials::Mat;
use artkit::math::{Axis, TAU, cos, radians, sin};
use artkit::motion::{Spin, spin};
use artkit::parts::{join, new_parts};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcone, fcyl, fextrude, fsphere, fsweep, revolve, torus};

const RING_X: f64 = 0.26;
const RING_Z: f64 = 0.02;
const RING_R: f64 = 0.28;
/// The table's length: a train table is 0.9-1.3 m (review M-kids-02: it was 1.8).
const TRAIN_W: f64 = 1.24;
const TABLE_TOP: f64 = 0.44;

/// A piece of the train made by `build(name, y)` along its own depth axis at the middle (x across, its front to
/// -depth, standing on the track's top `y`), then turned onto the ring at `a` degrees round it.
fn train_piece(name: &str, a: f64, build: &dyn Fn(&str, f64)) -> Obj {
    let parts = new_parts(|| build(name, TABLE_TOP + 0.012));
    let obj = join(&parts, name, Some(F.at(0, TABLE_TOP, 0)));
    let ar = radians(a);
    obj.set_location(F.at(
        RING_X + RING_R * cos(ar),
        TABLE_TOP,
        RING_Z + RING_R * sin(ar),
    ));
    obj.edit(|o| o.rot.z = ar as f32);
    obj
}

fn train_wheels(name: &str, y: f64, zs: &[f64]) {
    for (k, &z) in zs.iter().enumerate() {
        for s in [-1, 1] {
            fcyl!(
                F,
                format!("{name}_wheel{k}{s}"),
                0.013,
                0.007,
                (s as f64 * 0.027, y + 0.013, z),
                M.dark,
                rot = (0, 90, 0),
                verts = 10
            );
        }
    }
}

/// A wooden toy engine: a boiler with a chimney and a dome, a cab with a roof, three wheels a side.
fn engine(name: &str, y: f64) {
    fbox!(
        F,
        format!("{name}_chassis"),
        0.05,
        0.012,
        0.12,
        (0, y + 0.016, 0),
        M.train,
        bevel = 0.003,
        bevel_segments = 1
    );
    fcyl!(
        F,
        format!("{name}_boiler"),
        0.022,
        0.075,
        (0, y + 0.042, -0.02),
        M.train,
        rot = (90, 0, 0),
        verts = 12
    );
    fcyl!(
        F,
        format!("{name}_smokebox"),
        0.0235,
        0.01,
        (0, y + 0.042, -0.058),
        M.dark,
        rot = (90, 0, 0),
        verts = 12
    );
    fcone!(
        F,
        format!("{name}_chimney"),
        0.009,
        0.013,
        0.03,
        (0, y + 0.074, -0.042),
        M.dark,
        verts = 10
    );
    fsphere!(
        F,
        format!("{name}_dome"),
        0.011,
        (0, y + 0.064, -0.012),
        M.toy_yellow,
        segments = 8,
        rings = 5
    );
    fbox!(
        F,
        format!("{name}_cab"),
        0.054,
        0.052,
        0.042,
        (0, y + 0.048, 0.037),
        M.train,
        bevel = 0.004,
        bevel_segments = 1
    );
    fbox!(
        F,
        format!("{name}_roof"),
        0.064,
        0.008,
        0.054,
        (0, y + 0.078, 0.037),
        M.dark,
        bevel = 0.003,
        bevel_segments = 1
    );
    for s in [-1, 1] {
        fbox!(
            F,
            format!("{name}_window{s}"),
            0.004,
            0.018,
            0.022,
            (s as f64 * 0.027, y + 0.058, 0.037),
            M.water
        );
    }
    fbox!(
        F,
        format!("{name}_buffer"),
        0.044,
        0.01,
        0.008,
        (0, y + 0.018, -0.063),
        M.dark,
        bevel = 0.002,
        bevel_segments = 1
    );
    train_wheels(name, y, &[-0.038, 0.0, 0.038]);
}

fn wagon(colour: Mat, cargo: fn(&str, f64)) -> impl Fn(&str, f64) {
    move |name: &str, y: f64| {
        fbox!(
            F,
            format!("{name}_chassis"),
            0.05,
            0.012,
            0.09,
            (0, y + 0.016, 0),
            M.toy_yellow,
            bevel = 0.003,
            bevel_segments = 1
        );
        walls(
            &format!("{name}_tub"),
            0.054,
            0.08,
            y + 0.022,
            y + 0.052,
            0,
            0.006,
            colour,
            0.008,
            0.0,
        );
        fbox!(
            F,
            format!("{name}_tub_floor"),
            0.046,
            0.006,
            0.072,
            (0, y + 0.025, 0),
            colour
        );
        cargo(name, y);
        train_wheels(name, y, &[-0.028, 0.028]);
    }
}

/// A toy tree: a trunk and two cones of leaves, one over the other.
fn tree(name: &str, x: f64, z: f64, h: f64, y: f64) {
    fcyl!(
        F,
        format!("{name}_trunk"),
        0.012,
        0.04,
        (x, y + 0.02, z),
        M.trunk,
        verts = 8
    );
    fcone!(
        F,
        format!("{name}_low"),
        0.055,
        0.012,
        h * 0.55,
        (x, y + 0.035 + h * 0.275, z),
        M.tree,
        verts = 10
    );
    fcone!(
        F,
        format!("{name}_high"),
        0.042,
        0.0,
        h * 0.5,
        (x, y + 0.035 + h * 0.75, z),
        M.tree,
        verts = 10
    );
}

/// An arch's outline (x, height) standing on the ground: `ro` out and `ho` up to where its top rounds over, an
/// opening `ri` and `hi` likewise through it.
fn archway(ro: f64, ho: f64, ri: f64, hi: f64, n: usize) -> Vec<(f64, f64)> {
    let mut out = vec![(ro, 0.0), (ro, ho)];
    out.extend_from_slice(&arc(ro, 0.0, 180.0, n, 0.0, ho)[1..]);
    out.push((-ri, 0.0));
    out.extend(arc(ri, 180.0, 0.0, n, 0.0, hi));
    out.push((ri, 0.0));
    out
}

pub fn train_table() -> Vec<Obj> {
    // the table: round tapered legs, an apron, the board with a raised lip round it, a shelf below with two bins
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            let (x, z) = (sx as f64 * (TRAIN_W / 2.0 - 0.08), sz as f64 * 0.34);
            fsweep!(
                F,
                format!("leg{sx}{sz}"),
                &[(x, 0.36, z), (x, 0.0, z)],
                0.03,
                M.train_table,
                verts = 12,
                scale = [1.0, 0.78]
            );
        }
    }
    fbox!(
        F,
        "frame",
        TRAIN_W - 0.02,
        0.06,
        0.82,
        (0, 0.38, 0),
        M.train_table,
        bevel = 0.012,
        bevel_segments = 2
    );
    fbox!(
        F,
        "grass",
        TRAIN_W - 0.04,
        0.03,
        0.8,
        (0, 0.425, 0),
        M.grass,
        bevel = 0.006,
        bevel_segments = 1
    );
    let lip = rounded_rect(TRAIN_W - 0.02, 0.82, 0.03, 3);
    let path: Vec<P> = lip.iter().map(|&(u, v)| (u, 0.432, v)).collect();
    fsweep!(
        F,
        "lip",
        &path,
        0,
        M.train_table,
        profile = rounded_rect(0.036, 0.02, 0.008, 1),
        up = (0, 0, 1),
        closed = true
    );
    fbox!(
        F,
        "shelf",
        TRAIN_W - 0.16,
        0.02,
        0.62,
        (0, 0.1, 0),
        M.train_table,
        bevel = 0.005,
        bevel_segments = 1
    );
    for (k, (x, m)) in [(-0.25, M.toy_red), (0.12, M.toy_blue)]
        .into_iter()
        .enumerate()
    {
        walls(
            &format!("bin{k}"),
            0.3,
            0.26,
            0.11,
            0.22,
            0.02,
            0.008,
            m,
            0.02,
            x,
        );
        fbox!(
            F,
            format!("bin_floor{k}"),
            0.29,
            0.006,
            0.25,
            (x, 0.113, 0.02),
            m
        );
    }
    fbox!(
        F,
        "bin_block0",
        0.05,
        0.05,
        0.05,
        (-0.3, 0.185, 0.05),
        M.toy_yellow,
        rot = (0, 0, 20),
        bevel = 0.006,
        bevel_segments = 1
    );
    fsphere!(
        F,
        "bin_ball",
        0.035,
        (0.08, 0.19, 0.0),
        M.ball_yellow,
        segments = 10,
        rings = 6
    );
    // the ring of wooden track, with two grooves
    revolve!(
        "track",
        &[
            (RING_R - 0.04, 0.0),
            (RING_R + 0.04, 0.0),
            (RING_R + 0.04, 0.012),
            (RING_R - 0.04, 0.012),
            (RING_R - 0.04, 0.0)
        ],
        at = F.at(RING_X, TABLE_TOP - 0.002, RING_Z),
        material = M.track,
        segments = 40,
        smooth = false
    );
    for rr in [RING_R - 0.016, RING_R + 0.016] {
        torus!(
            format!("groove{}", r(rr)),
            rr,
            0.003,
            at = F.at(RING_X, TABLE_TOP + 0.0105, RING_Z),
            material = M.track_groove,
            major_segments = 40,
            minor_segments = 4
        );
    }
    // a tunnel the track runs through on the right, grass over its top, a stone portal at each end
    let (tx, tz) = (RING_X + RING_R, RING_Z);
    fextrude!(
        F,
        "tunnel",
        &archway(0.072, 0.09, 0.045, 0.075, 8),
        0.12,
        (tx, TABLE_TOP, tz),
        M.grass,
        bevel = 0.008,
        bevel_segments = 1
    );
    for s in [-1, 1] {
        fextrude!(
            F,
            format!("portal{s}"),
            &archway(0.06, 0.08, 0.045, 0.075, 6),
            0.01,
            (tx, TABLE_TOP, tz + s as f64 * 0.062),
            M.spot,
            bevel = 0.002,
            bevel_segments = 1
        );
    }
    // the station on the left of the track: a platform, a building with a pitched roof, a signal
    fbox!(
        F,
        "platform",
        0.05,
        0.02,
        0.26,
        (-0.09, TABLE_TOP + 0.01, RING_Z),
        M.spot,
        bevel = 0.004,
        bevel_segments = 1
    );
    fbox!(
        F,
        "station",
        0.1,
        0.09,
        0.2,
        (-0.16, TABLE_TOP + 0.045, RING_Z),
        M.toy_yellow,
        bevel = 0.005,
        bevel_segments = 1
    );
    fextrude!(
        F,
        "station_roof",
        &[(-0.065, 0.0), (0.065, 0.0), (0.0, 0.05)],
        0.22,
        (-0.16, TABLE_TOP + 0.09, RING_Z),
        M.toy_red,
        bevel = 0.004,
        bevel_segments = 1
    );
    fbox!(
        F,
        "station_door",
        0.004,
        0.06,
        0.04,
        (-0.108, TABLE_TOP + 0.03, RING_Z),
        M.house_door
    );
    for s in [-1, 1] {
        fbox!(
            F,
            format!("station_window{s}"),
            0.004,
            0.03,
            0.035,
            (-0.108, TABLE_TOP + 0.055, RING_Z + s as f64 * 0.06),
            M.water
        );
    }
    fcyl!(
        F,
        "signal_post",
        0.005,
        0.14,
        (-0.09, TABLE_TOP + 0.07, RING_Z + 0.17),
        M.dark,
        verts = 6
    );
    fbox!(
        F,
        "signal",
        0.016,
        0.04,
        0.014,
        (-0.09, TABLE_TOP + 0.13, RING_Z + 0.17),
        M.dark,
        bevel = 0.003,
        bevel_segments = 1
    );
    fsphere!(
        F,
        "signal_light",
        0.006,
        (-0.09, TABLE_TOP + 0.14, RING_Z + 0.178),
        M.toy_red,
        segments = 6,
        rings = 4
    );
    // the town: a road with its line, a house, a pond with stones round it, trees
    fbox!(
        F,
        "road",
        0.08,
        0.004,
        0.8,
        (-0.3, TABLE_TOP + 0.002, 0),
        M.track_groove
    );
    for k in 0..5 {
        fbox!(
            F,
            format!("road_line{k}"),
            0.008,
            0.002,
            0.07,
            (-0.3, TABLE_TOP + 0.005, -0.32 + 0.16 * k as f64),
            M.paper
        );
    }
    fbox!(
        F,
        "house",
        0.11,
        0.08,
        0.1,
        (-0.5, TABLE_TOP + 0.04, -0.25),
        M.house,
        bevel = 0.004,
        bevel_segments = 1
    );
    fextrude!(
        F,
        "house_roof",
        &[(-0.065, 0.0), (0.065, 0.0), (0.0, 0.05)],
        0.12,
        (-0.5, TABLE_TOP + 0.08, -0.25),
        M.roof,
        bevel = 0.003,
        bevel_segments = 1
    );
    fbox!(
        F,
        "house_door",
        0.03,
        0.045,
        0.004,
        (-0.5, TABLE_TOP + 0.0225, -0.199),
        M.house_door
    );
    fcyl!(
        F,
        "pond",
        0.09,
        0.006,
        (-0.49, TABLE_TOP + 0.003, 0.2),
        M.water,
        verts = 20
    );
    for k in 0..7 {
        let a = TAU * k as f64 / 7.0;
        fsphere!(
            F,
            format!("stone{k}"),
            0.014,
            (
                -0.49 + 0.103 * cos(a),
                TABLE_TOP + 0.004,
                0.2 + 0.103 * sin(a)
            ),
            M.spot,
            scale = (1.2, 1, 0.6),
            segments = 6,
            rings = 4
        );
    }
    for (k, (x, z, h)) in [
        (-0.55, -0.02, 0.2),
        (-0.17, -0.3, 0.16),
        (-0.17, 0.3, 0.18),
        (0.55, -0.33, 0.16),
        (0.56, 0.33, 0.14),
    ]
    .into_iter()
    .enumerate()
    {
        tree(&format!("tree{k}"), x, z, h, TABLE_TOP);
    }
    still("body", None);
    // the train: an engine and two wagons, turning round the ring's middle
    let parts = new_parts(|| {
        train_piece("engine", 90.0, &engine);
        train_piece(
            "wagon0",
            (90 + 28) as f64,
            &wagon(M.toy_blue, |n, y| {
                fsphere!(
                    F,
                    format!("{n}_ball"),
                    0.022,
                    (0, y + 0.065, 0),
                    M.ball_yellow,
                    segments = 10,
                    rings = 6
                );
            }),
        );
        train_piece(
            "wagon1",
            (90 + 52) as f64,
            &wagon(M.toy_green, |n, y| {
                fcyl!(
                    F,
                    format!("{n}_log"),
                    0.014,
                    0.08,
                    (0, y + 0.06, 0),
                    M.trunk,
                    rot = (90, 0, 0),
                    verts = 8
                );
            }),
        );
    });
    let train = join(&parts, "train", Some(F.at(RING_X, TABLE_TOP, RING_Z)));
    place(0.0);
    spin(
        train,
        "train",
        Spin {
            turns: -1.0,
            seconds: 6.0,
            axis: Axis::Z,
        },
    );
    meshes()
}

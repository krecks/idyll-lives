//! Bathroom storage and mirrors: the mirror cabinet, the round mirror and the tall cabinet.
//!
//! Built where the game places them. The two wall things hang at their real height, their backs on the wall face
//! (`settle` with `floor` false, `on_wall` on the Model). Mirror glass is a flat piece of the `mirror` material
//! (`gloss`, `front`): the game reflects the room in it where the preset has live mirrors (High) and draws it
//! pale gloss elsewhere, as the old parts' mirrors. The cabinets' doors are parts of their own, hinged on their
//! left edge. The mirror cabinet's swings open (`door_open`, the mirror with it) while a resident brushes their
//! teeth; the tall cabinet's upper door onto its towels while one takes a fresh towel.

use super::M;
use super::common::{LATHE, Lathe, WALL, bends, bottle, lathe, pull, shaker, wall_rose};
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{TAU, cos, sin};
use artkit::motion::Ease;
use artkit::parts::{hinged, join, join_all, new_parts};
use artkit::place::{Coord, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fsphere, fsweep, settle, soft, swing};

/// An open-fronted box: back, sides, top and bottom, `w` x `h` x `d`, its back at game depth `z0`.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn carcass(
    f: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    d: f64,
    x: impl Coord,
    y0: f64,
    z0: f64,
    material: Mat,
    t: f64,
) {
    let zc = z0 + d / 2.0;
    fbox!(
        f,
        format!("{name}_back"),
        w,
        h,
        t,
        (x, y0 + h / 2.0, z0 + t / 2.0),
        material
    );
    for s_ in [-1, 1] {
        fbox!(
            f,
            format!("{name}_side{s_}"),
            t,
            h,
            d,
            (x.f() + s_ as f64 * (w - t) / 2.0, y0 + h / 2.0, zc),
            material,
            bevel = 0.003
        );
    }
    for (k, y) in [y0 + t / 2.0, y0 + h - t / 2.0].into_iter().enumerate() {
        fbox!(
            f,
            format!("{name}_end{k}"),
            w,
            t,
            d,
            (x, y, zc),
            material,
            bevel = 0.003
        );
    }
}

/// A cabinet's hinges inside its side at game x, at heights `ys`: a plate on the side and an arm towards the
/// door at depth `z`.
fn hinge_cups(f: &GameFrame, name: &str, x: f64, ys: [f64; 2], z: f64, material: Mat) {
    for (k, y) in ys.into_iter().enumerate() {
        fbox!(
            f,
            format!("{name}{k}"),
            0.012,
            0.045,
            0.05,
            (x, y, z - 0.03),
            material,
            bevel = 0.003,
            bevel_segments = 1
        );
    }
}

/// A tumbler standing at height `y`: a slightly flared cup with a rim, open at the top.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn cup(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, r: f64, h: f64, material: Mat) {
    lathe(
        f,
        name,
        x,
        z,
        &[
            (y, r * 0.86, r * 0.86),
            (y + 0.004, r * 0.92, r * 0.92),
            (y + h, r, r),
            (y + h, r - 0.004, r - 0.004),
            (y + 0.012, r * 0.86 - 0.004, r * 0.86 - 0.004),
        ],
        material,
        Lathe { verts: 14, ..LATHE },
    );
}

/// Toothbrushes leaning out of a cup whose floor is at height `y`.
fn brushes(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, colours: &[Mat]) {
    for (k, &m) in colours.iter().enumerate() {
        let s = if k % 2 == 1 { 1.0 } else { -1.0 };
        kit::tube!(
            f,
            format!("{name}{k}"),
            &[
                (x + s * 0.004, y + 0.01, z),
                (x + s * 0.02, y + 0.14, z + 0.008 * s),
            ],
            0.0045,
            m,
            verts = 6,
            round = 1
        );
        fbox!(
            f,
            format!("{name}_head{k}"),
            0.008,
            0.024,
            0.012,
            (x + s * 0.0215, y + 0.155, z + 0.008 * s),
            M.paper,
            bevel = 0.002,
            bevel_segments = 1
        );
    }
}

pub fn mirror_cabinet() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (w, h, d, y0, z0) = (0.56, 0.72, 0.16, 1.2, WALL);
    let front = z0 + d;
    let still = new_parts(|| {
        carcass(&f, "cabinet", w, h, d, 0, y0, z0, M.cab, 0.016);
        hinge_cups(
            &f,
            "hinge",
            -(w / 2.0 - 0.022),
            [y0 + 0.1, y0 + h - 0.1],
            front,
            M.chrome,
        );
        // two shelves with a lip along the front, a cup of toothbrushes, bottles, a jar and a tube
        for (k, y) in [1.43, 1.66].into_iter().enumerate() {
            fbox!(
                f,
                format!("shelf{k}"),
                w - 0.04,
                0.008,
                d - 0.03,
                (0, y, z0 + d / 2.0),
                M.porcelain,
                bevel = 0.002
            );
            fbox!(
                f,
                format!("shelf_lip{k}"),
                w - 0.04,
                0.02,
                0.006,
                (0, y + 0.006, z0 + d - 0.018),
                M.porcelain,
                bevel = 0.002,
                bevel_segments = 1
            );
        }
        cup(&f, "cup", -0.15, y0 + 0.016, -0.36, 0.03, 0.09, M.teal);
        brushes(
            &f,
            "brush",
            -0.15,
            y0 + 0.028,
            -0.36,
            &[M.porcelain, M.sage],
        );
        bottle(
            &f,
            "bottle0",
            (0.12, 1.434, -0.36),
            0.022,
            0.12,
            M.bottle,
            M.teal,
            12,
            true,
        );
        bottle(
            &f,
            "bottle1",
            (0.18, 1.434, -0.35),
            0.02,
            0.09,
            M.sage,
            M.bottle,
            12,
            false,
        );
        lathe(
            &f,
            "jar",
            -0.1,
            -0.36,
            &[
                (1.664, 0.028, 0.028),
                (1.668, 0.03, 0.03),
                (1.71, 0.03, 0.03),
                (1.712, 0.026, 0.026),
            ],
            M.porcelain,
            Lathe { verts: 14, ..LATHE },
        );
        fcyl!(
            f,
            "jar_lid",
            0.031,
            0.014,
            (-0.1, 1.718, -0.36),
            M.chrome,
            verts = 14,
            bevel = 0.003,
            bevel_segments = 1
        );
        fsweep!(
            f,
            "paste",
            &[(0.05, y0 + 0.03, -0.36), (0.14, y0 + 0.03, -0.355)],
            0.0,
            M.paper,
            profile = rounded_rect(0.03, 0.012, 0.005, 1),
            up = (0, 0, 1),
            scale = [1.0, 0.5]
        );
        fcyl!(
            f,
            "paste_cap",
            0.008,
            0.016,
            (0.148, y0 + 0.03, -0.355),
            M.teal,
            rot = (0, 90, 0),
            verts = 8
        );
        // a light bar over the top on an arm from the wall
        kit::tube!(
            f,
            "lamp_arm",
            &bends(
                &[
                    (0.0, 1.935, z0 + 0.004),
                    (0.0, 1.935, -0.36),
                    (0.0, 1.95, -0.345),
                ],
                0.015,
                3,
            ),
            0.007,
            M.chrome,
            verts = 8,
            round = 1
        );
        wall_rose(&f, "lamp_rose", 0, 1.935, z0, 0.02, M.chrome, 0.008, 12);
        soft!(
            f,
            "lamp",
            0.46,
            0.04,
            0.06,
            (0, 1.95, -0.33),
            M.cab,
            round = 0.015,
            round_segments = 2
        );
        soft!(
            f,
            "lamp_strip",
            0.42,
            0.01,
            0.03,
            (0, 1.927, -0.32),
            M.bulb,
            round = 0.004,
            round_segments = 1
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let parts = new_parts(|| {
        soft!(
            f,
            "door",
            w - 0.02,
            h - 0.02,
            0.02,
            (0, y0 + h / 2.0, front + 0.01),
            M.cab_door,
            round = 0.008,
            round_segments = 2
        );
        fbox!(
            f,
            "door_mirror",
            0.48,
            0.64,
            0.004,
            (0, y0 + h / 2.0, front + 0.0235),
            M.mirror
        );
        pull(
            &f,
            "door_pull",
            0.25,
            y0 + h / 2.0,
            front + 0.02,
            0.1,
            M.chrome,
            true,
            0.005,
            0.016,
        );
        for (k, y) in [y0 + 0.1, y0 + h - 0.1].into_iter().enumerate() {
            fbox!(
                f,
                format!("door_hinge{k}"),
                0.02,
                0.045,
                0.012,
                (-(w / 2.0 - 0.04), y, front - 0.006),
                M.chrome,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
    });
    let door = hinged(
        join(&parts, "door", None),
        f.at(-(w - 0.02) / 2.0, y0 + h / 2.0, front),
    );
    settle!(centre = false, floor = false);
    swing!(door, "door_open", 100, seconds = 0.8, ease = Ease::BackOut);
    meshes()
}

pub fn mirror_round() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (y, z, r) = (1.55, -0.405, 0.3);
    let still = new_parts(|| {
        // the frame: a moulded section, flat at the back and rounded to the front, swept round the circle
        let moulding = vec![
            (-0.02, -0.02),
            (-0.02, 0.02),
            (0.0, 0.02),
            (0.012, 0.016),
            (0.019, 0.007),
            (0.02, 0.0),
            (0.019, -0.007),
            (0.012, -0.016),
            (0.0, -0.02),
        ];
        let circle: Vec<(f64, f64, f64)> = (0..48)
            .map(|i| {
                (
                    r * cos(TAU * i as f64 / 48.0),
                    y + r * sin(TAU * i as f64 / 48.0),
                    z,
                )
            })
            .collect();
        fsweep!(
            f,
            "frame",
            &circle,
            0.0,
            M.frame_brass,
            profile = moulding,
            closed = true,
            up = (0, -1, 0),
            angle = 60
        );
        fcyl!(
            f,
            "backing",
            r,
            0.01,
            (0, y, z - 0.012),
            M.frame_brass,
            rot = (90, 0, 0),
            verts = 48
        );
        // a leather strap riveted to the frame's top, up over a brass hook on the wall
        let hook = (0, 1.98, WALL + 0.02);
        fsweep!(
            f,
            "strap",
            &[
                (-0.075, 1.83, z + 0.012),
                (-0.06, 1.87, z + 0.01),
                (-0.025, 1.94, WALL + 0.032),
                (0.0, 1.99, WALL + 0.036),
                (0.025, 1.94, WALL + 0.032),
                (0.06, 1.87, z + 0.01),
                (0.075, 1.83, z + 0.012)
            ],
            0.0,
            M.leather,
            profile = rounded_rect(0.004, 0.026, 0.0015, 1),
            up = (0, -1, 0),
            resolution = 2
        );
        for s_ in [-1, 1] {
            fsphere!(
                f,
                format!("rivet{s_}"),
                0.005,
                (s_ as f64 * 0.072, 1.838, z + 0.014),
                M.frame_brass,
                segments = 8,
                rings = 4
            );
        }
        wall_rose(
            &f,
            "hook_plate",
            hook.0,
            hook.1,
            WALL,
            0.022,
            M.frame_brass,
            0.01,
            14,
        );
        kit::tube!(
            f,
            "hook",
            &bends(
                &[
                    (0.0, hook.1, WALL + 0.008),
                    (0.0, hook.1, WALL + 0.04),
                    (0.0, hook.1 + 0.025, WALL + 0.04),
                ],
                0.012,
                3,
            ),
            0.006,
            M.frame_brass,
            verts = 8,
            round = 1
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let glass = new_parts(|| {
        fcyl!(
            f,
            "mirror",
            r - 0.012,
            0.004,
            (0, y, z - 0.004),
            M.mirror,
            rot = (90, 0, 0),
            verts = 48
        );
    });
    join(&glass, "mirror", None);
    join_all("mirrorRound");
    settle!(centre = false, floor = false);
    meshes()
}

pub fn bath_cabinet_tall() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (w, d, z0) = (0.42, 0.34, WALL);
    let front = z0 + d;
    let niche = (0.61, 0.86); // the open shelf between the doors
    let still = new_parts(|| {
        fbox!(
            f,
            "plinth",
            0.4,
            0.06,
            0.3,
            (0, 0.03, -0.27),
            M.dark,
            bevel = 0.004
        );
        carcass(&f, "cabinet", w, 1.76, d, 0, 0.06, z0, M.cab, 0.016);
        fbox!(
            f,
            "niche_floor",
            w - 0.03,
            0.022,
            d - 0.02,
            (0, niche.0 + 0.011, z0 + d / 2.0),
            M.cab,
            bevel = 0.003
        );
        fbox!(
            f,
            "niche_top",
            w - 0.03,
            0.022,
            d - 0.02,
            (0, niche.1 - 0.011, z0 + d / 2.0),
            M.cab,
            bevel = 0.003
        );
        // two towels rolled in the niche
        for (k, (x, m)) in [(-0.09, M.towel_blue), (0.09, M.towel_cream)]
            .into_iter()
            .enumerate()
        {
            kit::tube!(
                f,
                format!("towel{k}"),
                &[(x, niche.0 + 0.07, -0.4), (x, niche.0 + 0.07, -0.13)],
                0.048,
                m,
                verts = 12,
                round = 1
            );
            fcyl!(
                f,
                format!("towel_end{k}"),
                0.03,
                0.004,
                (x, niche.0 + 0.07, -0.128),
                m,
                rot = (90, 0, 0),
                verts = 10
            );
        }
        // behind the upper door: a stack of folded towels, a shelf with two more and bottles on those
        for (k, m) in [M.towel_cream, M.towel_blue, M.towel_cream]
            .into_iter()
            .enumerate()
        {
            soft!(
                f,
                format!("stack{k}"),
                0.3,
                0.072,
                0.24,
                (0, niche.1 + 0.047 + 0.075 * k as f64, -0.26),
                m,
                round = 0.03,
                puff = (0, 0.006, 0),
                round_segments = 2,
                flat = 2
            );
        }
        fbox!(
            f,
            "shelf",
            w - 0.03,
            0.018,
            d - 0.03,
            (0, 1.36, z0 + d / 2.0 - 0.005),
            M.cab,
            bevel = 0.003
        );
        for (k, m) in [M.towel_blue, M.towel_cream].into_iter().enumerate() {
            soft!(
                f,
                format!("stack_top{k}"),
                0.28,
                0.068,
                0.22,
                (0, 1.404 + 0.07 * k as f64, -0.27),
                m,
                round = 0.028,
                puff = (0, 0.006, 0),
                round_segments = 2,
                flat = 2
            );
        }
        bottle(
            &f,
            "bottle0",
            (-0.08, 1.509, -0.3),
            0.025,
            0.14,
            M.bottle,
            M.teal,
            12,
            true,
        );
        bottle(
            &f,
            "bottle1",
            (0.04, 1.509, -0.31),
            0.022,
            0.11,
            M.sage,
            M.bottle,
            12,
            false,
        );
        hinge_cups(
            &f,
            "hinge",
            -(w / 2.0 - 0.024),
            [niche.1 + 0.12, 1.69],
            front,
            M.chrome,
        );
        // the lower door (still), shaker-panelled as the upper one
        shaker(
            &f,
            "door_low",
            0.4,
            niche.0 - 0.08,
            0,
            (0.07 + niche.0) / 2.0,
            front,
            M.cab_door,
            0.02,
            0.055,
        );
        pull(
            &f,
            "pull_low",
            0.15,
            niche.0 - 0.1,
            front + 0.02,
            0.1,
            M.chrome,
            true,
            0.005,
            0.015,
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let top = 1.81;
    let parts = new_parts(|| {
        shaker(
            &f,
            "door",
            0.4,
            top - niche.1 - 0.01,
            0,
            (niche.1 + top) / 2.0,
            front,
            M.cab_door,
            0.02,
            0.055,
        );
        pull(
            &f,
            "pull",
            0.15,
            niche.1 + 0.1,
            front + 0.02,
            0.12,
            M.chrome,
            true,
            0.005,
            0.015,
        );
    });
    let door = hinged(
        join(&parts, "door", None),
        f.at(-0.2, (niche.1 + top) / 2.0, front),
    );
    settle!(centre = false);
    swing!(door, "door_open", 100, seconds = 0.8, ease = Ease::BackOut);
    meshes()
}

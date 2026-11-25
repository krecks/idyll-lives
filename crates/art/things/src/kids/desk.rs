//! KD3: the kids' desk (and its chair), table and chair.

use super::common::{P, heart, place, still, walls};
use super::{F, M};
use artkit::forms::rounded_rect;
use artkit::json::float_repr as r;
use artkit::kit;
use artkit::kit::fillet;
use artkit::materials::Mat;
use artkit::math::{PI, TAU, cos, sin};
use artkit::parts::{join, join_all, new_parts};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, fsphere, fsweep, revolve, slide};

/// A turned knob on a front at game depth `face` that faces the back (-depth): kit.knob turned round.
fn back_knob(
    name: &str,
    x: impl artkit::place::Coord,
    y: f64,
    face: f64,
    r: f64,
    material: Mat,
    segments: usize,
) -> Obj {
    let prof = [
        (0.0, 0.0),
        (r * 0.75, 0.0),
        (r * 0.7, r * 0.15),
        (r * 0.36, r * 0.35),
        (r * 0.42, r * 0.75),
        (r * 0.95, r * 1.05),
        (r, r * 1.35),
        (r * 0.8, r * 1.7),
        (r * 0.4, r * 1.88),
        (0.0, r * 1.92),
    ];
    revolve!(
        name,
        &prof,
        at = F.at(x, y, face),
        material = material,
        segments = segments,
        rot = (-90, 0, 0)
    )
}

/// A crayon from `a` to its sharpened tip at `b` (game points), a paper wrapper round its middle.
fn crayon(name: &str, a: P, b: P, material: Mat, r: f64) {
    let k = 0.82;
    let mid = (
        a.0 + (b.0 - a.0) * k,
        a.1 + (b.1 - a.1) * k,
        a.2 + (b.2 - a.2) * k,
    );
    fsweep!(
        F,
        name,
        &[a, mid, b],
        r,
        material,
        verts = 6,
        scale = [1.0, 1.0, 0.25]
    );
}

pub fn kids_desk() -> Vec<Obj> {
    // the desk: round legs tapering to the floor, side stretchers and a foot bar, a rounded wooden top with an
    // upstand along its far edge
    for x in [-0.34, 0.34] {
        for z in [0.02, 0.38] {
            fsweep!(
                F,
                format!("leg{}{}", r(x), r(z)),
                &[(x, 0.545, z), (x, 0.0, z)],
                0.021,
                M.kdesk,
                verts = 10,
                scale = [1.0, 0.8]
            );
        }
        fsweep!(
            F,
            format!("stretcher{}", r(x)),
            &[(x, 0.12, 0.02), (x, 0.12, 0.38)],
            0.009,
            M.kdesk,
            verts = 8
        );
    }
    fsweep!(
        F,
        "foot_bar",
        &[(-0.34, 0.1, 0.02), (0.34, 0.1, 0.02)],
        0.011,
        M.kdesk,
        verts = 8
    );
    fextrude!(
        F,
        "top",
        &rounded_rect(0.76, 0.44, 0.03, 3),
        0.03,
        (0, 0.56, 0.2),
        M.kdesk_top,
        plane = Facing::Top,
        bevel = 0.006,
        bevel_segments = 2
    );
    fbox!(
        F,
        "apron",
        0.64,
        0.06,
        0.38,
        (0, 0.515, 0.2),
        M.kdesk,
        bevel = 0.006,
        bevel_segments = 1
    );
    fbox!(
        F,
        "upstand",
        0.72,
        0.045,
        0.016,
        (0, 0.5975, 0.404),
        M.kdesk,
        bevel = 0.006,
        bevel_segments = 2
    );
    // on it: a sheet of paper with a drawing, a pot of crayons
    fbox!(
        F,
        "paper",
        0.3,
        0.004,
        0.22,
        (-0.12, 0.577, 0.2),
        M.paper,
        rot = (0, 0, 6)
    );
    fbox!(
        F,
        "drawing",
        0.2,
        0.002,
        0.14,
        (-0.12, 0.5795, 0.2),
        M.chalk_blue,
        rot = (0, 0, 6)
    );
    let sun: Vec<(f64, f64)> = (0..10)
        .map(|i| {
            (
                0.022 * cos(TAU * i as f64 / 10.0),
                0.022 * sin(TAU * i as f64 / 10.0),
            )
        })
        .collect();
    fextrude!(
        F,
        "drawing_sun",
        &sun,
        0.002,
        (-0.06, 0.581, 0.15),
        M.paint_yellow,
        plane = Facing::Top
    );
    let mut hill = vec![(-0.1, 0.0), (0.1, 0.0)];
    hill.extend((0..7).map(|i| {
        (
            0.1 - 0.2 * i as f64 / 6.0,
            -0.035 * sin(PI * i as f64 / 6.0),
        )
    }));
    fextrude!(
        F,
        "drawing_hill",
        &hill,
        0.002,
        (-0.12, 0.581, 0.265),
        M.paint_green,
        plane = Facing::Top,
        rot = (0, 0, 6)
    );
    kit::turned!(
        F,
        "cup",
        0.24,
        0.575,
        0.3,
        &[
            (0.0, 0.0),
            (0.03, 0.0),
            (0.036, 0.075),
            (0.033, 0.077),
            (0.028, 0.012),
            (0.0, 0.012),
        ],
        M.toy_yellow,
        segments = 14
    );
    for (k, (dx, dz, m)) in [
        (-0.012, -0.006, M.toy_red),
        (0.01, 0.004, M.toy_blue),
        (0.0, 0.012, M.toy_green),
    ]
    .into_iter()
    .enumerate()
    {
        crayon(
            &format!("crayon{k}"),
            (0.24 + dx, 0.59, 0.3 + dz),
            (0.24 + dx * 2.5, 0.69, 0.3 + dz * 2.0),
            m,
            0.005,
        );
    }
    // the chair, joined to the desk: its seat where the game sits a child (SEATS.kidsDesk), a curved back on two posts
    fextrude!(
        F,
        "seat",
        &rounded_rect(0.4, 0.38, 0.05, 3),
        0.03,
        (0, 0.35, -0.19),
        M.kdesk_chair,
        plane = Facing::Top,
        bevel = 0.008,
        bevel_segments = 2
    );
    fbox!(
        F,
        "seat_rail",
        0.34,
        0.04,
        0.32,
        (0, 0.315, -0.2),
        M.kdesk_chair,
        bevel = 0.006,
        bevel_segments = 1
    );
    for s in [-1, 1] {
        let sf = s as f64;
        fsweep!(
            F,
            format!("front_leg{s}"),
            &[(sf * 0.17, 0.32, -0.04), (sf * 0.18, 0.0, -0.025)],
            0.0175,
            M.kdesk_chair,
            verts = 8,
            scale = [1.0, 0.85]
        );
        fsweep!(
            F,
            format!("back_post{s}"),
            &[(sf * 0.18, 0.0, -0.38), (sf * 0.18, 0.66, -0.38)],
            0.0175,
            M.kdesk_chair,
            verts = 8
        );
        fsphere!(
            F,
            format!("post_cap{s}"),
            0.019,
            (sf * 0.18, 0.662, -0.38),
            M.kdesk_chair,
            segments = 8,
            rings = 5
        );
        fsweep!(
            F,
            format!("rung{s}"),
            &[(sf * 0.178, 0.12, -0.03), (sf * 0.18, 0.12, -0.38)],
            0.009,
            M.kdesk_chair,
            verts = 6
        );
    }
    for (name, y, h) in [("backrest", 0.61, 0.1), ("back_rail", 0.44, 0.04)] {
        fsweep!(
            F,
            name,
            &[(-0.19, y, -0.374), (0.0, y, -0.385), (0.19, y, -0.374)],
            0,
            M.kdesk_chair,
            profile = rounded_rect(h, 0.02, 0.008, 2),
            up = (0, 0, 1),
            resolution = 3
        );
    }
    still("body", None);
    // the pencil drawer under the top, pulled out towards the child
    let parts = new_parts(|| {
        fbox!(
            F,
            "drawer_front",
            0.4,
            0.05,
            0.015,
            (0, 0.512, 0.0025),
            M.kdesk,
            bevel = 0.005,
            bevel_segments = 2
        );
        fbox!(F, "drawer_tray", 0.38, 0.04, 0.3, (0, 0.512, 0.15), M.kdesk);
        back_knob("drawer_knob", 0, 0.512, -0.005, 0.01, M.handle, 8);
        for (k, m) in [M.toy_red, M.toy_blue, M.toy_green, M.toy_yellow]
            .into_iter()
            .enumerate()
        {
            let kf = k as f64;
            crayon(
                &format!("pencil{k}"),
                (-0.12 + 0.07 * kf, 0.536, 0.03),
                (-0.1 + 0.07 * kf, 0.536, 0.2),
                m,
                0.005,
            );
        }
        fbox!(
            F,
            "eraser",
            0.04,
            0.012,
            0.025,
            (0.15, 0.538, 0.12),
            M.chalk_pink,
            bevel = 0.003,
            bevel_segments = 1
        );
    });
    let drawer = join(&parts, "drawer", None);
    place(0.0);
    slide!(drawer, "drawer_open", (0, 0.15, 0), seconds = 0.6);
    meshes()
}

pub fn kids_table() -> Vec<Obj> {
    // a rounded top with a band in the frame's colour round its edge, a rounded apron, turned legs on bun feet
    fextrude!(
        F,
        "top",
        &rounded_rect(0.632, 0.632, 0.08, 4),
        0.03,
        (0, 0.485, 0),
        M.ktable_top,
        plane = Facing::Top,
        bevel = 0.008,
        bevel_segments = 2
    );
    let band = rounded_rect(0.632, 0.632, 0.08, 4);
    let path: Vec<P> = band.iter().map(|&(u, v)| (u, 0.481, v)).collect();
    fsweep!(
        F,
        "band",
        &path,
        0,
        M.ktable,
        profile = rounded_rect(0.016, 0.008, 0.003, 1),
        up = (0, 0, 1),
        closed = true
    );
    walls(
        "apron", 0.52, 0.52, 0.415, 0.47, 0, 0.02, M.ktable, 0.05, 0.0,
    );
    let prof = [
        (0.0, 0.0),
        (0.02, 0.0),
        (0.03, 0.012),
        (0.03, 0.026),
        (0.021, 0.042),
        (0.018, 0.1),
        (0.016, 0.26),
        (0.019, 0.34),
        (0.026, 0.37),
        (0.022, 0.39),
        (0.025, 0.47),
        (0.0, 0.47),
    ];
    for sx in [-1, 1] {
        for sz in [-1, 1] {
            kit::turned!(
                F,
                format!("leg{sx}{sz}"),
                sx as f64 * 0.235,
                0.0,
                sz as f64 * 0.235,
                &prof,
                M.ktable,
                segments = 10
            );
        }
    }
    join_all("kidsTable");
    place(0.0);
    meshes()
}

pub fn kids_chair() -> Vec<Obj> {
    // a rounded seat, legs splayed a little, the back legs running up into posts behind a back with a heart cut
    // through it, rungs along the sides and across the front
    fextrude!(
        F,
        "seat",
        &rounded_rect(0.32, 0.3, 0.05, 3),
        0.03,
        (0, 0.29, 0.05),
        M.kchair_seat,
        plane = Facing::Top,
        bevel = 0.008,
        bevel_segments = 2
    );
    for sx in [-1.0, 1.0] {
        fsweep!(
            F,
            format!("leg{sx}"),
            &[(sx * 0.13, 0.28, 0.165), (sx * 0.135, 0.0, 0.172)],
            0.0175,
            M.kchair,
            verts = 8,
            scale = [1.0, 0.85]
        );
        fsweep!(
            F,
            format!("post{sx}"),
            &fillet(
                &[
                    (sx * 0.135, 0.0, -0.075),
                    (sx * 0.13, 0.28, -0.07),
                    (sx * 0.13, 0.5, -0.1)
                ],
                0.06,
                3
            ),
            0.016,
            M.kchair,
            verts = 8
        );
        fsweep!(
            F,
            format!("rung{sx}"),
            &[(sx * 0.133, 0.1, -0.072), (sx * 0.133, 0.1, 0.168)],
            0.009,
            M.kchair,
            verts = 6
        );
    }
    fsweep!(
        F,
        "rung_front",
        &[(-0.133, 0.14, 0.168), (0.133, 0.14, 0.168)],
        0.008,
        M.kchair,
        verts = 6
    );
    fextrude!(
        F,
        "back",
        &rounded_rect(0.32, 0.18, 0.06, 3),
        0.028,
        (0, 0.48, -0.1),
        M.kchair,
        rot = (-6, 0, 0),
        holes = vec![heart(0.0, 0.005, 0.04, 20)],
        bevel = 0.008,
        bevel_segments = 2
    );
    join_all("kidsChair");
    place(0.0);
    meshes()
}

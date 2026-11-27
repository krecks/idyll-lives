use super::super::common::{F, M};
use artkit::json::float_repr;
use artkit::kit;
use artkit::math::{PI, cos, sin};
use artkit::parts::join_all;
use artkit::place::Facing;
use artkit::place::N;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, fsphere, settle};

// A weatherproof table at the ITTF's size (2.74 x 1.525 m, its top 0.76 m up) on 3 x 2 tiles, the players at its two
// ends (content `places`): a green top on two concrete legs, a steel net 15.25 cm high reaching 15.25 cm past each
// side, and a ball. Still. The bats are in the players' hands (held `tool.bat`; review I-community-06: they lay on
// the top while it was played). (It was 1.9 x 0.94 m, its top at 0.78.)
pub(super) const PONG: (f64, f64, f64) = (2.74, 0.76, 1.525); // the top's length, height and width

pub fn ping_pong_table() -> Vec<Obj> {
    let (length, top, width) = PONG;
    let (hl, hw) = (length / 2.0, width / 2.0);
    // the top: a slab with a band round its edge, a steel frame under it
    fbox!(
        F,
        "top",
        length,
        0.03,
        width,
        (0, top - 0.015, 0),
        M.table_top,
        bevel = 0.006,
        bevel_segments = 1
    );
    kit::moulding!(
        F,
        "top_band",
        &[(hl, -hw), (hl, hw), (-hl, hw), (-hl, -hw)],
        &[
            (0.0, -0.035),
            (0.0, -0.002),
            (0.006, -0.002),
            (0.006, -0.035),
        ],
        M.steel,
        at = top,
        plane = Facing::Top
    );
    for z in [-hw + 0.1, hw - 0.1] {
        fbox!(
            F,
            format!("apron{}", float_repr(z)),
            length - 0.3,
            0.06,
            0.04,
            (0, top - 0.06, z),
            M.steel,
            bevel = 0.004,
            bevel_segments = 1
        );
    }
    // two concrete legs: a tapered pier with an arch through it, on a broad foot
    for x in [-0.85, 0.85] {
        let leg = [
            (-0.5, 0.04),
            (0.5, 0.04),
            (0.42, top - 0.03),
            (-0.42, top - 0.03),
        ];
        let arch: Vec<(f64, f64)> = (0..11)
            .map(|k| {
                (
                    0.18 * cos(PI * k as f64 / 10.0),
                    0.04 + 0.32 * sin(PI * k as f64 / 10.0),
                )
            })
            .collect();
        let mut hole = vec![(0.18, 0.12)];
        hole.extend(
            arch[1..arch.len() - 1]
                .iter()
                .map(|&(px, py)| (px, py + 0.08)),
        );
        hole.push((-0.18, 0.12));
        fextrude!(
            F,
            format!("leg{}", float_repr(x)),
            &leg,
            0.12,
            (x, 0, 0),
            M.concrete,
            plane = Facing::Side,
            holes = [hole],
            bevel = 0.01,
            bevel_segments = 1
        );
        fbox!(
            F,
            format!("foot{}", float_repr(x)),
            0.22,
            0.04,
            1.12,
            (x, 0.02, 0),
            M.concrete,
            bevel = 0.01,
            bevel_segments = 1
        );
        fbox!(
            F,
            format!("cap{}", float_repr(x)),
            0.16,
            0.02,
            0.88,
            (x, top - 0.04, 0),
            M.steel,
            bevel = 0.004,
            bevel_segments = 1
        );
    }
    let y = top + 0.001;
    for (w, d, x, z) in [
        (length, 0.02, N::I(0), N::F(hw - 0.01)),
        (length, 0.02, N::I(0), N::F(-hw + 0.01)),
        (0.02, width, N::F(hl - 0.01), N::I(0)),
        (0.02, width, N::F(-hl + 0.01), N::I(0)),
        (length, 0.003, N::I(0), N::I(0)),
    ] {
        fbox!(F, format!("line{x}{z}"), w, 0.003, d, (x, y, z), M.lines);
    }
    // the steel net: a perforated plate (rows of holes as a darker band) between two posts clamped to the top
    let net = width + 2.0 * 0.1525;
    fbox!(
        F,
        "net",
        0.012,
        0.13,
        net - 0.04,
        (0, top + 0.07, 0),
        M.steel
    );
    for k in 0..3 {
        fbox!(
            F,
            format!("net_slot{k}"),
            0.014,
            0.012,
            net - 0.08,
            (0, top + 0.03 + k as f64 * 0.035, 0),
            M.rubber
        );
    }
    fbox!(
        F,
        "net_band",
        0.026,
        0.02,
        net,
        (0, top + 0.1425, 0),
        M.lines,
        bevel = 0.004,
        bevel_segments = 1
    );
    for z in [-net / 2.0 + 0.015, net / 2.0 - 0.015] {
        fbox!(
            F,
            format!("net_post{}", float_repr(z)),
            0.04,
            0.17,
            0.03,
            (0, top + 0.08, z),
            M.steel,
            bevel = 0.005,
            bevel_segments = 1
        );
        fbox!(
            F,
            format!("net_clamp{}", float_repr(z)),
            0.06,
            0.05,
            0.05,
            (0, top - 0.015, (hw - 0.005).copysign(z)),
            M.steel,
            bevel = 0.005,
            bevel_segments = 1
        );
        fbox!(
            F,
            format!("net_arm{}", float_repr(z)),
            0.03,
            0.02,
            z.abs() - hw + 0.03,
            (0, top + 0.005, ((z.abs() + hw) / 2.0).copysign(z)),
            M.steel
        );
    }
    // the ball
    fsphere!(
        F,
        "ball",
        0.02,
        (0.5, top + 0.02, -0.15),
        M.ping,
        segments = 10,
        rings = 6
    );
    join_all("pingPongTable");
    settle!(centre = false);
    meshes()
}

//! The teacher's desk, the lectern and the bar.

use super::common::{F, M, book, open_book, rod, tube};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::math::{Axis, V3, atan2, cos, degrees, radians, sin};
use artkit::parts::{join, new_parts, settle_all};
use artkit::place::Facing;
use artkit::scene::{Obj, RotationMode, meshes};
use artkit::{
    fbox, fcyl, fextrude, fsphere, fsweep, kit_knob, kit_moulding, kit_shaker, revolve, sphere,
};

// --- the teacher's desk: 2x1, oak, a pedestal of drawers to the teacher's side ----------------------------------
pub fn teacher_desk() -> Vec<Obj> {
    let t = 0.76;
    let parts = new_parts(|| {
        fbox!(
            F,
            "top",
            1.5,
            0.03,
            0.7,
            (0, t - 0.015, 0),
            M.oak,
            bevel = 0.006
        );
        // the pedestal on the right of the class's view, its drawers to the teacher at the back
        fbox!(
            F,
            "pedestal",
            0.42,
            t - 0.04,
            0.64,
            (0.5, (t - 0.04) / 2.0 + 0.01, 0),
            M.oak,
            bevel = 0.004
        );
        fbox!(
            F,
            "pedestal_plinth",
            0.4,
            0.04,
            0.62,
            (0.5, 0.02, 0.0),
            M.plinth
        );
        for i in 0..3 {
            let y = 0.17 + i as f64 * 0.2;
            kit_shaker!(
                F,
                format!("drawer{i}"),
                0.38,
                0.18,
                (0.5, y, -0.326),
                0.014,
                M.oak,
                rail = 0.04
            );
            kit_knob!(F, format!("knob{i}"), 0.5, y, -0.333, 0.014, M.brass)
                .set_rotation_euler_axis(0, radians(-90.0));
        }
        for s in [-1, 1] {
            fbox!(
                F,
                format!("leg{s}"),
                0.05,
                t - 0.03,
                0.05,
                (-0.7, (t - 0.03) / 2.0, s as f64 * 0.3),
                M.oak,
                bevel = 0.004
            );
        }
        fbox!(
            F,
            "apron_side",
            0.03,
            0.1,
            0.56,
            (-0.7, t - 0.08, 0.0),
            M.oak
        );
        fbox!(
            F,
            "apron_back",
            0.98,
            0.08,
            0.03,
            (-0.22, t - 0.07, -0.31),
            M.oak
        );
        // the modesty panel to the class
        kit_shaker!(
            F,
            "modesty",
            0.98,
            0.5,
            (-0.215, t - 0.28, 0.31),
            0.02,
            M.oak,
            rail = 0.06,
            raised = true
        );
        // the register, exercise books, an apple, a pot of pencils and a globe
        book(
            "register",
            0.24,
            0.025,
            0.32,
            (0.05, t + 0.0125, -0.08),
            M.leaflet3,
            (0.0, 0.0, -6.0),
        );
        for k in 0..5 {
            let kf = k as f64;
            book(
                &format!("exercise{k}"),
                0.21,
                0.008,
                0.29,
                (0.45 + 0.005 * (k % 2) as f64, t + 0.004 + kf * 0.0085, 0.1),
                [M.leaflet1, M.leaflet4, M.leaflet2, M.leaflet1, M.leaflet3][k],
                (0.0, 0.0, (k as i32 * 4 - 6) as f64),
            );
        }
        fsphere!(
            F,
            "apple",
            0.04,
            (0.25, t + 0.037, 0.16),
            M.leaflet4,
            scale = (1, 1, 0.92),
            segments = 14,
            rings = 10
        );
        artkit::bevel::eased(fcyl!(
            F,
            "apple_stalk",
            0.003,
            0.025,
            (0.25, t + 0.083, 0.16),
            M.soil,
            verts = 5
        ));
        artkit::bevel::eased(revolve!(
            "pencil_pot",
            &[
                (0.0, 0.0),
                (0.035, 0.0),
                (0.035, 0.1),
                (0.031, 0.1),
                (0.031, 0.006),
                (0.0, 0.006)
            ],
            at = F.at(0.62, t, -0.2),
            material = M.pot,
            segments = 14
        ));
        for k in 0..4 {
            let a = k as f64 * 1.6;
            rod(
                &format!("pencil{k}"),
                (0.62 + 0.01 * cos(a), t + 0.02, -0.2 + 0.01 * sin(a)),
                (0.62 + 0.035 * cos(a), t + 0.17, -0.2 + 0.035 * sin(a)),
                0.004,
                [M.leaflet2, M.leaflet4, M.leaflet1, M.leaflet3][k],
                6,
            );
        }
        let (gx, gz) = (-0.5, -0.12);
        // (box UVs on the turned base: round about its axis, the grain stretched over its slopes near the middle)
        let base = artkit::bevel::eased(revolve!(
            "globe_base",
            &[
                (0.0, 0.0),
                (0.08, 0.0),
                (0.08, 0.015),
                (0.05, 0.03),
                (0.02, 0.04),
                (0.015, 0.09),
                (0.0, 0.09)
            ],
            at = F.at(gx, t, gz),
            material = M.walnut,
            segments = 16
        ));
        base.edit(|p| p.uv = artkit::mesh::UvWay::Box);
        let arc: Vec<(f64, f64, f64)> = (150..371)
            .step_by(20)
            .map(|d| radians(d as f64))
            .map(|a| (gx + 0.13 * sin(a), t + 0.23 + 0.13 * cos(a), gz))
            .collect();
        artkit::bevel::eased(fsweep!(
            F,
            "globe_arc",
            &arc,
            0,
            M.brass,
            profile = rounded_rect(0.006, 0.012, 0.0, 4),
            up = (0, 1, 0)
        ));
        fsphere!(
            F,
            "globe",
            0.12,
            (gx, t + 0.23, gz),
            M.leaflet1,
            segments = 20,
            rings = 14
        );
        // its lands: flattened patches lying on the sphere
        let centre = V3::from(F.at(gx, t + 0.23, gz));
        for (k, (dx, dy, dz, r)) in [
            (0.5, 0.5, 0.7, 0.06),
            (-0.7, -0.1, 0.7, 0.05),
            (0.2, -0.6, -0.75, 0.07),
            (-0.6, 0.6, -0.5, 0.045),
        ]
        .into_iter()
        .enumerate()
        {
            let n = V3::from(F.at(dx, dy, dz)).normalized();
            let land = sphere!(
                format!("land{k}"),
                r,
                at = centre + n * 0.116,
                material = M.leaflet3,
                scale = (1, 0.8, 0.1),
                segments = 10,
                rings = 6
            );
            land.set_rotation_mode(RotationMode::Quaternion);
            land.set_rotation_quaternion(n.to_track_quat(Axis::Z, Axis::Y));
        }
    });
    join(&parts, "desk", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

// --- the lectern: 1x1, a carved oak stand, the book sloping to the reader at the back ---------------------------
pub fn lectern() -> Vec<Obj> {
    let parts = new_parts(|| {
        fbox!(
            F,
            "base",
            0.56,
            0.06,
            0.56,
            (0, 0.03, 0),
            M.pew_oak,
            bevel = 0.012
        );
        fbox!(
            F,
            "base_step",
            0.44,
            0.05,
            0.44,
            (0, 0.085, 0),
            M.pew_oak,
            bevel = 0.01
        );
        artkit::bevel::eased(revolve!(
            "column",
            &[
                (0.0, 0.11),
                (0.09, 0.11),
                (0.09, 0.14),
                (0.06, 0.16),
                (0.05, 0.2),
                (0.045, 0.45),
                (0.07, 0.48),
                (0.07, 0.52),
                (0.045, 0.55),
                (0.04, 0.85),
                (0.06, 0.88),
                (0.075, 0.92),
                (0.0, 0.92)
            ],
            material = M.pew_oak,
            segments = 16
        ));
        // the desk: a sloping board on a box, its front to the church carved with a cross
        fextrude!(
            F,
            "desk_side_l",
            &[(-0.24, 0.98), (0.24, 1.2), (0.24, 0.92), (-0.24, 0.92)],
            0.03,
            (0.3, 0, 0),
            M.pew_oak,
            plane = Facing::Side,
            bevel = 0.004
        );
        fextrude!(
            F,
            "desk_side_r",
            &[(-0.24, 0.98), (0.24, 1.2), (0.24, 0.92), (-0.24, 0.92)],
            0.03,
            (-0.3, 0, 0),
            M.pew_oak,
            plane = Facing::Side,
            bevel = 0.004
        );
        fbox!(
            F,
            "desk_front",
            0.57,
            0.28,
            0.03,
            (0, 1.06, 0.225),
            M.pew_oak,
            bevel = 0.004
        );
        fbox!(
            F,
            "desk_back",
            0.57,
            0.06,
            0.03,
            (0, 0.95, -0.225),
            M.pew_oak,
            bevel = 0.004
        );
        fbox!(F, "desk_floor", 0.57, 0.02, 0.45, (0, 0.93, 0), M.pew_oak);
        let slope = degrees(atan2(0.22, 0.48));
        fbox!(
            F,
            "desk_top",
            0.66,
            0.025,
            0.54,
            (0, 1.1, 0.0),
            M.pew_oak,
            bevel = 0.006,
            rot = (-slope, 0, 0)
        );
        fbox!(
            F,
            "desk_lip",
            0.62,
            0.05,
            0.025,
            (0, 1.0, -0.255),
            M.pew_oak,
            bevel = 0.006
        );
        artkit::bevel::eased(kit_moulding!(
            F,
            "front_frame",
            &[(-0.25, 0.94), (0.25, 0.94), (0.25, 1.17), (-0.25, 1.17)],
            &[
                (0.0, 0.0),
                (0.0, 0.012),
                (0.006, 0.014),
                (0.02, 0.014),
                (0.02, 0.0)
            ],
            M.pew_oak,
            at = 0.24
        ));
        fbox!(
            F,
            "cross_v",
            0.035,
            0.18,
            0.012,
            (0, 1.055, 0.246),
            M.brass,
            bevel = 0.003
        );
        fbox!(
            F,
            "cross_h",
            0.12,
            0.035,
            0.012,
            (0, 1.09, 0.246),
            M.brass,
            bevel = 0.003
        );
        // the big book, open on the slope, its ribbon over the lower edge
        let b = open_book("bible", 0.46, 0.34, (0, 0, 0), M.book, 0.0, 0.035);
        b.set_rotation_euler_axis(0, radians(-slope));
        b.set_location(F.at(0, 1.1 + 0.014 / cos(radians(slope)), 0.0));
        fbox!(F, "ribbon", 0.016, 0.16, 0.003, (0.02, 1.0, -0.27), M.fob);
    });
    join(&parts, "lectern", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

// --- the bar: 2x1, panelled mahogany, three pumps, a brass foot rail ---------------------------------------------
pub fn bar_counter() -> Vec<Obj> {
    let top = 1.05;
    let parts = new_parts(|| {
        fbox!(
            F,
            "carcass",
            1.9,
            top - 0.06,
            0.33,
            (0, (top - 0.06) / 2.0 + 0.01, 0.065),
            M.mahogany,
            bevel = 0.006
        );
        fbox!(F, "plinth", 1.88, 0.08, 0.58, (0, 0.04, -0.07), M.plinth);
        // the barman's side: shelves under the top, open to the back, glasses and bottles on them
        for x in [-0.935, 0.0, 0.935] {
            fbox!(
                F,
                format!("back_end{}", float_repr(x)),
                0.03,
                top - 0.06,
                0.3,
                (x, (top - 0.06) / 2.0 + 0.01, -0.245),
                M.mahogany,
                bevel = 0.004
            );
        }
        for y in [0.095, 0.5] {
            fbox!(
                F,
                format!("back_shelf{}", float_repr(y)),
                1.84,
                0.03,
                0.29,
                (0, y, -0.245),
                M.mahogany,
                bevel = 0.003
            );
        }
        for k in 0..10 {
            let x = -0.83 + k as f64 * 0.18 + (if k >= 5 { 0.04 } else { 0.0 });
            revolve!(
                format!("glass{k}"),
                &[
                    (0.0, 0.0),
                    (0.03, 0.0),
                    (0.033, 0.01),
                    (0.038, 0.13),
                    (0.035, 0.13),
                    (0.03, 0.012),
                    (0.0, 0.012)
                ],
                at = F.at(x, 0.515, -0.25),
                material = M.acrylic,
                segments = 12
            );
        }
        for k in 0..6 {
            let x = -0.78 + k as f64 * 0.3;
            artkit::bevel::eased(revolve!(
                format!("bottle{k}"),
                &[
                    (0.0, 0.0),
                    (0.035, 0.0),
                    (0.035, 0.18),
                    (0.014, 0.23),
                    (0.012, 0.28),
                    (0.0, 0.28)
                ],
                at = F.at(x, 0.11, -0.25),
                material = [M.leaf, M.beer, M.mag1][k % 3],
                segments = 12
            ));
        }
        for (i, x) in [-0.62, 0.0, 0.62].into_iter().enumerate() {
            kit_shaker!(
                F,
                format!("panel{i}"),
                0.6,
                0.78,
                (x, 0.53, 0.241),
                0.022,
                M.mahogany,
                rail = 0.07,
                raised = true
            );
        }
        // the top: a thick slab with a rounded nose to the drinkers, a bead under it
        artkit::bevel::eased(fextrude!(
            F,
            "top",
            &[
                (-0.4, 0.0),
                (0.33, 0.0),
                (0.355, 0.006),
                (0.37, 0.022),
                (0.37, 0.034),
                (0.355, 0.05),
                (0.33, 0.056),
                (-0.4, 0.056)
            ],
            1.96,
            (0, top - 0.056, 0),
            M.mahogany,
            plane = Facing::Side
        ));
        artkit::bevel::eased(fsweep!(
            F,
            "cove",
            &[(-0.95, top - 0.068, 0.236), (0.95, top - 0.068, 0.236)],
            0,
            M.mahogany,
            profile = [
                (0.012, -0.01),
                (0.012, 0.012),
                (-0.012, 0.012),
                (-0.012, 0.004),
                (0.0, 0.0),
                (0.006, -0.01)
            ]
        ));
        // the brass foot rail on three brackets
        rod(
            "rail",
            (-0.92, 0.2, 0.4),
            (0.92, 0.2, 0.4),
            0.024,
            M.brass,
            14,
        );
        for x in [-0.92, 0.92] {
            fsphere!(
                F,
                format!("rail_end{}", float_repr(x)),
                0.03,
                (x, 0.2, 0.4),
                M.brass,
                segments = 12,
                rings = 8
            );
        }
        for x in [-0.7, 0.0, 0.7] {
            tube(
                &format!("bracket{}", float_repr(x)),
                &[(x, 0.32, 0.25), (x, 0.32, 0.36), (x, 0.2, 0.4)],
                0.012,
                M.brass,
                0.03,
                8,
            );
        }
        // the pumps over a drip tray at the back of the top
        fbox!(
            F,
            "drip_tray",
            0.7,
            0.02,
            0.12,
            (0, top + 0.01, -0.1),
            M.bell,
            bevel = 0.004
        );
        for (i, (x, clip)) in [(-0.25, M.clip1), (0.0, M.clip2), (0.25, M.clip3)]
            .into_iter()
            .enumerate()
        {
            artkit::bevel::eased(revolve!(
                format!("pump{i}"),
                &[
                    (0.0, 0.0),
                    (0.045, 0.0),
                    (0.045, 0.01),
                    (0.035, 0.02),
                    (0.03, 0.1),
                    (0.04, 0.12),
                    (0.02, 0.13),
                    (0.0, 0.13)
                ],
                at = F.at(x, top, -0.2),
                material = M.bell,
                segments = 16
            ));
            artkit::bevel::eased(fsweep!(
                F,
                format!("spout{i}"),
                &[
                    (x, top + 0.09, -0.17),
                    (x, top + 0.12, -0.12),
                    (x, top + 0.08, -0.09)
                ],
                0.008,
                M.bell,
                resolution = 3,
                verts = 8
            ));
            artkit::bevel::eased(revolve!(
                format!("handle{i}"),
                &[
                    (0.0, 0.0),
                    (0.016, 0.0),
                    (0.02, 0.05),
                    (0.024, 0.2),
                    (0.02, 0.26),
                    (0.012, 0.28),
                    (0.0, 0.28)
                ],
                at = F.at(x, top + 0.13, -0.2),
                material = M.handle,
                segments = 12
            ));
            fbox!(
                F,
                format!("clip{i}"),
                0.065,
                0.085,
                0.01,
                (x, top + 0.31, -0.18),
                clip,
                bevel = 0.004
            );
        }
        // a runner along the front with two pints on it, beer mats
        fbox!(
            F,
            "runner",
            1.2,
            0.006,
            0.12,
            (0, top + 0.003, 0.2),
            M.towel
        );
        for (k, x) in [-0.45, 0.5].into_iter().enumerate() {
            artkit::bevel::eased(revolve!(
                format!("pint{k}"),
                &[
                    (0.0, 0.0),
                    (0.03, 0.0),
                    (0.033, 0.01),
                    (0.038, 0.11),
                    (0.041, 0.13),
                    (0.0, 0.13)
                ],
                at = F.at(x, top + 0.006, 0.2),
                material = M.beer,
                segments = 16
            ));
            artkit::bevel::eased(revolve!(
                format!("head{k}"),
                &[
                    (0.0, 0.0),
                    (0.041, 0.0),
                    (0.041, 0.018),
                    (0.03, 0.022),
                    (0.0, 0.022)
                ],
                at = F.at(x, top + 0.136, 0.2),
                material = M.froth,
                segments = 16
            ));
        }
        for (k, x) in [-0.8, 0.82].into_iter().enumerate() {
            fbox!(
                F,
                format!("mat{k}"),
                0.095,
                0.004,
                0.095,
                (x, top + 0.002, 0.18),
                if k != 0 { M.clip2 } else { M.clip1 }
            );
        }
    });
    join(&parts, "bar", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

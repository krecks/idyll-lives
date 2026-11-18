//! The founder's statue: a bronze woman in a long coat on a granite plinth, pointing.

use super::common::{N, far_keeping_feet, lobed, own};
use artkit::math::P3;
use artkit::parts::{join_all, settle_all};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, kit_moulding, loft, revolve, ring};

pub fn town_statue() -> Vec<Obj> {
    let m = own("townStatue");
    let f = GameFrame::new(0.0);
    let bronze = m.body;
    let y = 1.42; // the plinth's top
    {
        // the plinth: two steps, a moulded base, the die with a sunk panel on each face, a moulded cornice
        fbox!(
            f,
            "step",
            1.8,
            0.1,
            1.8,
            (0, 0.05, 0),
            m.granite,
            bevel = 0.015
        );
        fbox!(
            f,
            "step2",
            1.56,
            0.1,
            1.56,
            (0, 0.15, 0),
            m.granite,
            bevel = 0.015
        );
        fbox!(
            f,
            "die",
            1.2,
            1.1,
            1.2,
            (0, 0.75, 0),
            m.granite,
            bevel = 0.02
        );
        let sq = [(0.6, 0.6), (-0.6, 0.6), (-0.6, -0.6), (0.6, -0.6)];
        kit_moulding!(
            f,
            "base_moulding",
            &sq,
            &[
                (-0.07, 0.0),
                (-0.07, 0.04),
                (-0.04, 0.06),
                (-0.02, 0.1),
                (0.02, 0.1),
                (0.02, 0.0)
            ],
            m.granite,
            at = 0.2,
            plane = Facing::Top
        );
        kit_moulding!(
            f,
            "cornice",
            &sq,
            &[
                (0.02, -0.04),
                (-0.02, -0.04),
                (-0.04, 0.0),
                (-0.08, 0.03),
                (-0.08, 0.11),
                (0.02, 0.11)
            ],
            m.granite,
            at = 1.31,
            plane = Facing::Top
        );
        for s in [-1, 1] {
            let sf = s as f64;
            for axis in ['x', 'z'] {
                if axis == 'z' {
                    fbox!(
                        f,
                        format!("panel_z{s}"),
                        0.9,
                        0.7,
                        0.02,
                        (0, 0.78, sf * 0.6),
                        m.granite,
                        bevel = 0.01,
                        bevel_segments = 1
                    );
                } else {
                    fbox!(
                        f,
                        format!("panel_x{s}"),
                        0.02,
                        0.7,
                        0.9,
                        (sf * 0.6, 0.78, 0),
                        m.granite,
                        bevel = 0.01,
                        bevel_segments = 1
                    );
                }
            }
        }
        // the plaque: brass with a bevelled edge, three lines of raised lettering
        fbox!(
            f,
            "plaque",
            0.5,
            0.28,
            0.02,
            (0, 0.85, 0.615),
            m.brass,
            bevel = 0.008,
            bevel_segments = 2
        );
        for (k, w) in [0.36, 0.28, 0.2].into_iter().enumerate() {
            fbox!(
                f,
                format!("plaque_text{k}"),
                w,
                if k == 0 { 0.03 } else { 0.018 },
                0.006,
                (0, 0.92 - 0.065 * k as f64, 0.626),
                m.iron
            );
        }
        // the figure on a rough bronze base: shoes, a long coat falling in folds, the body, a collar and lapels
        fbox!(
            f,
            "base",
            0.7,
            0.05,
            0.6,
            (0, y + 0.025, 0),
            bronze,
            bevel = 0.015
        );
        fbox!(
            f,
            "shoe_left",
            0.12,
            0.08,
            0.26,
            (-0.1, y + 0.09, 0.06),
            bronze,
            bevel = 0.03
        );
        fbox!(
            f,
            "shoe_right",
            0.12,
            0.08,
            0.26,
            (0.12, y + 0.09, 0.02),
            bronze,
            bevel = 0.03
        );
        let P3 { x: bx, y: by_, .. } = f.at(0, 0, 0);
        let coat = [
            (0.32, 0.26, 0.08),
            (0.31, 0.25, 0.2),
            (0.285, 0.23, 0.42),
            (0.255, 0.2, 0.68),
            (0.225, 0.175, 0.9),
            (0.205, 0.16, 1.05),
            (0.195, 0.15, 1.15),
        ];
        let mut rings = Vec::new();
        for (rx, rz, h) in coat {
            let fold = 0.11 * (1.0 - h / 1.15); // the folds deepest at the hem, gone at the waist
            let ring = lobed(1.0, y + h, 9, fold, 3);
            rings.push(
                ring.iter()
                    .map(|p| P3 {
                        x: bx + p.x * rx,
                        y: by_ + p.y * rz,
                        z: p.z,
                    })
                    .collect::<Vec<P3>>(),
            );
        }
        loft!("coat", &rings, material = bronze);
        let body = [
            (0.19, 0.145, 1.1),
            (0.205, 0.15, 1.3),
            (0.235, 0.16, 1.46),
            (0.22, 0.15, 1.54),
            (0.16, 0.12, 1.6),
            (0.07, 0.07, 1.63),
        ];
        let secs: Vec<Vec<P3>> = body
            .iter()
            .map(|&(rx, rz, h)| ring!(rx, rz, z = y + h, verts = 18, square = 2.4, at = (bx, by_)))
            .collect();
        loft!("body", &secs, material = bronze);
        fextrude!(
            f,
            "lapel_l",
            &[(0.0, -0.18), (0.07, 0.0), (0.06, 0.12), (0.0, 0.02)],
            0.03,
            (-0.01, y + 1.42, 0.13),
            bronze,
            bevel = 0.006,
            bevel_segments = 1,
            rot = (-12, 0, 0)
        );
        fextrude!(
            f,
            "lapel_r",
            &[(0.0, -0.18), (0.0, 0.02), (-0.06, 0.12), (-0.07, 0.0)],
            0.03,
            (0.01, y + 1.42, 0.13),
            bronze,
            bevel = 0.006,
            bevel_segments = 1,
            rot = (-12, 0, 0)
        );
        revolve!(
            "collar",
            &[
                (0.0, 1.6),
                (0.09, 1.6),
                (0.085, 1.66),
                (0.06, 1.68),
                (0.0, 1.68)
            ],
            at = f.at(0, y, 0),
            material = bronze,
            segments = 12
        );
        for (k, h) in [1.18, 1.3, 1.42].into_iter().enumerate() {
            // the coat's buttons
            fsphere!(
                f,
                format!("button{k}"),
                0.016,
                (0, y + h, 0.152 + 0.012 * k as f64),
                bronze,
                segments = 6,
                rings = 4
            );
        }
        fcyl!(f, "neck", 0.055, 0.12, (0, y + 1.66, 0), bronze, verts = 10);
        fsphere!(
            f,
            "head",
            0.12,
            (0, y + 1.82, 0.01),
            bronze,
            segments = 16,
            rings = 10,
            scale = (0.95, 1.05, 1.15)
        );
        fsphere!(
            f,
            "hair",
            0.125,
            (0, y + 1.85, -0.02),
            bronze,
            segments = 16,
            rings = 8,
            scale = (1.0, 1.0, 0.95)
        );
        fsphere!(
            f,
            "bun",
            0.07,
            (0, y + 1.9, -0.13),
            bronze,
            segments = 10,
            rings = 6
        );
        fsphere!(
            f,
            "nose",
            0.02,
            (0, y + 1.81, 0.13),
            bronze,
            segments = 6,
            rings = 4
        );
        fsphere!(
            f,
            "chin",
            0.05,
            (0, y + 1.74, 0.07),
            bronze,
            segments = 8,
            rings = 5,
            scale = (1.1, 0.8, 0.9)
        );
        // the right arm points up and away, one sleeve from the shoulder to a turned-back cuff; the left holds a book
        // to her side
        fsweep!(
            f,
            "arm_right",
            &[
                (0.2, y + 1.52, N::I(0)),
                (0.31, y + 1.57, N::F(0.08)),
                (0.4, y + 1.62, N::F(0.16)),
                (0.5, y + 1.71, N::F(0.23)),
                (0.59, y + 1.79, N::F(0.29))
            ],
            0.056,
            bronze,
            verts = 10,
            resolution = 2,
            scale = [1.0, 0.92, 0.85, 0.8, 0.85]
        );
        fsphere!(
            f,
            "hand_right",
            0.042,
            (0.63, y + 1.83, 0.32),
            bronze,
            segments = 10,
            rings = 6,
            scale = (1, 0.8, 1.2)
        );
        fsweep!(
            f,
            "finger",
            &[(0.63, y + 1.83, 0.32), (0.69, y + 1.87, 0.37)],
            0.013,
            bronze,
            verts = 6
        );
        fsweep!(
            f,
            "arm_left",
            &[
                (-0.21, y + 1.52, N::I(0)),
                (-0.25, y + 1.36, N::F(0.02)),
                (-0.27, y + 1.2, N::F(0.04)),
                (-0.24, y + 1.16, N::F(0.12)),
                (-0.21, y + 1.14, N::F(0.18))
            ],
            0.056,
            bronze,
            verts = 10,
            resolution = 2,
            scale = [1.0, 0.9, 0.85, 0.82, 0.85]
        );
        fsphere!(
            f,
            "hand_left",
            0.04,
            (-0.2, y + 1.14, 0.2),
            bronze,
            segments = 8,
            rings = 5
        );
        fbox!(
            f,
            "book",
            0.06,
            0.26,
            0.2,
            (-0.25, y + 1.12, 0.16),
            bronze,
            bevel = 0.01
        );
        fbox!(
            f,
            "book_pages",
            0.064,
            0.24,
            0.18,
            (-0.25, y + 1.12, 0.165),
            bronze
        );
        fsweep!(
            f,
            "cuff_right",
            &[(0.57, y + 1.775, 0.278), (0.605, y + 1.805, 0.302)],
            0.05,
            bronze,
            verts = 10
        );
        fsweep!(
            f,
            "cuff_left",
            &[(-0.215, y + 1.145, 0.16), (-0.205, y + 1.14, 0.19)],
            0.05,
            bronze,
            verts = 10
        );
    }
    join_all("townStatue");
    settle_all();
    meshes()
}

pub fn town_statue_far() -> Vec<Obj> {
    far_keeping_feet(town_statue, "townStatue")
}

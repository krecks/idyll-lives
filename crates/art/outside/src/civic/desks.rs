//! The front desks (2 x 1): a high ledge to the front over a low worktop at the back. The reception, hotel and duty
//! desks and the teller counter share one carcass.

use super::common::{F, M, bell, book, keyboard, monitor, open_book, phone, plant, rod, tube};
use artkit::materials::Mat;
use artkit::math::P3;
use artkit::parts::{join, new_parts, settle_all};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, kit_chequer, kit_shaker, loft, revolve, ring};

pub const LEDGE: f64 = 1.08; // the customer's ledge
pub const WORK: f64 = 0.75; // the worktop behind it

/// The carcass the reception, hotel and duty desks share: ends stepping from the ledge down to the worktop, a
/// front panel, a dark kick, an inner panel facing the staff, the ledge and the worktop, a drawer pedestal.
pub fn desk_carcass(body: Mat, ledge: Mat, band: bool) {
    for s in [-1, 1] {
        fextrude!(
            F,
            format!("end{s}"),
            &[
                (-0.4, 0.0),
                (0.4, 0.0),
                (0.4, LEDGE - 0.02),
                (0.06, LEDGE - 0.02),
                (0.06, WORK - 0.02),
                (-0.4, WORK - 0.02)
            ],
            0.04,
            (s as f64 * 0.91, 0, 0),
            body,
            plane = Facing::Side,
            bevel = 0.004
        );
    }
    fbox!(
        F,
        "front",
        1.78,
        LEDGE - 0.1,
        0.03,
        (0, 0.06 + (LEDGE - 0.1) / 2.0 + 0.0, 0.375),
        body,
        bevel = 0.004
    );
    fbox!(F, "kick", 1.78, 0.06, 0.03, (0, 0.03, 0.355), M.plinth);
    fbox!(
        F,
        "inner",
        1.78,
        LEDGE - WORK - 0.01,
        0.02,
        (0, (LEDGE + WORK) / 2.0 - 0.015, 0.07),
        body
    );
    fbox!(
        F,
        "worktop",
        1.78,
        0.03,
        0.47,
        (0, WORK - 0.015, -0.165),
        if band { ledge } else { body },
        bevel = 0.004
    );
    fbox!(
        F,
        "pedestal",
        0.44,
        WORK - 0.05,
        0.46,
        (0.66, (WORK - 0.03) / 2.0 + 0.01, -0.16),
        body,
        bevel = 0.004
    );
    for i in 0..3 {
        let y = 0.13 + i as f64 * 0.22;
        fbox!(
            F,
            format!("drawer{i}"),
            0.41,
            0.2,
            0.012,
            (0.66, y, -0.396),
            body,
            bevel = 0.003
        );
        fbox!(
            F,
            format!("drawer_pull{i}"),
            0.14,
            0.012,
            0.014,
            (0.66, y + 0.06, -0.408),
            M.frame,
            bevel = 0.004
        );
    }
}

pub fn reception_desk() -> Vec<Obj> {
    let parts = new_parts(|| {
        desk_carcass(M.desk, M.oak, false);
        fbox!(
            F,
            "ledge",
            1.92,
            0.035,
            0.42,
            (0, LEDGE + 0.0175, 0.225),
            M.oak,
            bevel = 0.008
        );
        fbox!(
            F,
            "band",
            1.8,
            0.16,
            0.015,
            (0, 0.8, 0.398),
            M.oak,
            bevel = 0.003
        );
        for i in 0..13 {
            // oak fins under the band
            fbox!(
                F,
                format!("fin{i}"),
                0.03,
                0.56,
                0.02,
                (-0.84 + i as f64 * 0.14, 0.4, 0.4),
                M.oak,
                bevel = 0.004
            );
        }
        let top = WORK;
        monitor("monitor", -0.15, top, -0.08, 0.5, 0.3, -1);
        keyboard("keyboard", -0.2, top, -0.27, 0.42);
        phone("phone", 0.5, top, -0.18, 180.0);
        revolve!(
            "pens",
            &[
                (0.0, 0.0),
                (0.035, 0.0),
                (0.035, 0.1),
                (0.03, 0.1),
                (0.03, 0.006),
                (0.0, 0.006)
            ],
            at = F.at(-0.7, top, -0.15),
            material = M.monitor,
            segments = 12
        );
        for k in 0..3 {
            let kf = k as f64;
            rod(
                &format!("pen{k}"),
                (-0.7 + 0.012 * (kf - 1.0), top + 0.03, -0.15 + 0.01 * kf),
                (
                    -0.7 + 0.03 * (kf - 1.0),
                    top + 0.16,
                    -0.15 + 0.02 * (kf - 1.0),
                ),
                0.005,
                [M.leaflet1, M.leaflet4, M.monitor][k],
                6,
            );
        }
        // on the ledge: a bell, a stand of leaflets, a plant
        let y = LEDGE + 0.035;
        bell("bell", 0.15, y, 0.3);
        let (lx, lz) = (-0.5, 0.22);
        fbox!(
            F,
            "stand_base",
            0.36,
            0.012,
            0.2,
            (lx, y + 0.006, lz),
            M.acrylic
        );
        for row in 0..3 {
            let rf = row as f64;
            let (z, h) = (lz + 0.06 - rf * 0.06, 0.07 + rf * 0.06);
            fbox!(
                F,
                format!("stand_pocket{row}"),
                0.36,
                h,
                0.006,
                (lx, y + h / 2.0, z + 0.02),
                M.acrylic,
                rot = (-8, 0, 0)
            );
            for c in 0..3 {
                let lh = h + 0.06;
                fbox!(
                    F,
                    format!("leaflet{row}{c}"),
                    0.1,
                    lh,
                    0.006,
                    (lx - 0.12 + c as f64 * 0.12, y + lh / 2.0 + 0.01, z + 0.005),
                    [M.leaflet1, M.leaflet2, M.leaflet3, M.leaflet4][(row * 3 + c) % 4],
                    rot = (-8, 0, 0)
                );
            }
        }
        plant("plant", 0.68, y, 0.24, 0.07);
    });
    join(&parts, "desk", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

pub fn hotel_reception() -> Vec<Obj> {
    let parts = new_parts(|| {
        desk_carcass(M.walnut, M.marble, false);
        // its front: three raised walnut panels over a brass kick strip
        for (i, x) in [-0.595, 0.0, 0.595].into_iter().enumerate() {
            kit_shaker!(
                F,
                format!("panel{i}"),
                0.57,
                0.84,
                (x, 0.56, 0.398),
                0.022,
                M.walnut,
                rail = 0.07,
                raised = true
            );
        }
        fbox!(F, "kick_strip", 1.8, 0.05, 0.006, (0, 0.06, 0.393), M.brass);
        // the marble ledge, its nose rounded, on a walnut moulding
        artkit::bevel::eased(fextrude!(
            F,
            "ledge",
            &[
                (0.03, 0.0),
                (0.42, 0.0),
                (0.436, 0.006),
                (0.443, 0.018),
                (0.436, 0.03),
                (0.42, 0.036),
                (0.03, 0.036)
            ],
            1.94,
            (0, LEDGE, 0),
            M.marble,
            plane = Facing::Side
        ));
        fbox!(
            F,
            "ledge_bead",
            1.86,
            0.03,
            0.03,
            (0, LEDGE - 0.015, 0.398),
            M.walnut,
            bevel = 0.008
        );
        let y = LEDGE + 0.036;
        open_book("guest_book", 0.42, 0.3, (-0.2, y, 0.25), M.book, 0.0, 0.02);
        rod(
            "pen",
            (0.06, y + 0.006, 0.18),
            (0.2, y + 0.006, 0.24),
            0.005,
            M.brass,
            6,
        );
        bell("bell", 0.3, y, 0.3);
        // a banker's lamp: a brass base and stem, the green glass shade
        artkit::bevel::eased(revolve!(
            "lamp_base",
            &[
                (0.0, 0.0),
                (0.08, 0.0),
                (0.08, 0.012),
                (0.06, 0.03),
                (0.0, 0.032)
            ],
            at = F.at(0.68, y, 0.2),
            material = M.brass,
            segments = 20
        ));
        artkit::bevel::eased(fcyl!(
            F,
            "lamp_stem",
            0.01,
            0.26,
            (0.68, y + 0.16, 0.15),
            M.brass,
            verts = 8
        ));
        fbox!(
            F,
            "lamp_arm",
            0.02,
            0.02,
            0.1,
            (0.68, y + 0.29, 0.2),
            M.brass
        );
        let P3 {
            x: bx,
            y: by,
            z: bz,
        } = F.at(0.68, y + 0.3, 0.24);
        let mut sections: Vec<Vec<P3>> = [0.0, 0.03]
            .iter()
            .map(|&h| {
                ring!(0.2, 0.09, z = h, verts = 16, square = 2.6)
                    .iter()
                    .map(|p| P3 {
                        x: bx + p.x,
                        y: by + p.y * 0.55,
                        z: bz + p.z,
                    })
                    .collect()
            })
            .collect();
        sections.push(
            ring!(0.18, 0.04, z = 0.065, verts = 16, square = 2.6)
                .iter()
                .map(|p| P3 {
                    x: bx + p.x,
                    y: by + p.y * 0.25,
                    z: bz + p.z + 0.0,
                })
                .collect(),
        );
        loft!("lamp_shade", &sections, material = M.shade);
        // the room keys on hooks, a board standing at the back of the worktop for the guests to see
        let kx = -0.6;
        fbox!(
            F,
            "key_board",
            0.56,
            0.66,
            0.03,
            (kx, WORK + 0.33, -0.385),
            M.walnut,
            bevel = 0.006
        );
        fbox!(
            F,
            "key_cap",
            0.6,
            0.03,
            0.06,
            (kx, WORK + 0.675, -0.375),
            M.walnut,
            bevel = 0.008
        );
        for r in 0..3 {
            for c in 0..4 {
                let (hx, hy) = (kx - 0.195 + c as f64 * 0.13, WORK + 0.56 - r as f64 * 0.19);
                artkit::bevel::eased(fcyl!(
                    F,
                    format!("hook{r}{c}"),
                    0.006,
                    0.04,
                    (hx, hy, -0.35),
                    M.brass,
                    rot = (90, 0, 0),
                    verts = 6
                ));
                if (r * 4 + c) % 5 != 3 {
                    // a few rooms out
                    fbox!(
                        F,
                        format!("key{r}{c}"),
                        0.012,
                        0.06,
                        0.004,
                        (hx, hy - 0.035, -0.345),
                        M.brass
                    );
                    artkit::bevel::eased(revolve!(
                        format!("fob{r}{c}"),
                        &[
                            (0.0, 0.0),
                            (0.016, 0.0),
                            (0.02, 0.03),
                            (0.016, 0.07),
                            (0.0, 0.075)
                        ],
                        at = F.at(hx, hy - 0.14, -0.345),
                        material = M.fob,
                        segments = 8
                    ));
                }
            }
        }
        phone("phone", 0.2, WORK, -0.2, 180.0);
        book(
            "ledger",
            0.3,
            0.035,
            0.22,
            (-0.15, WORK + 0.0175, -0.2),
            M.book,
            (0.0, 0.0, 8.0),
        );
    });
    join(&parts, "desk", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

pub fn duty_desk() -> Vec<Obj> {
    let parts = new_parts(|| {
        desk_carcass(M.counter, M.laminate, true);
        fbox!(
            F,
            "ledge",
            1.92,
            0.03,
            0.42,
            (0, LEDGE + 0.015, 0.225),
            M.laminate,
            bevel = 0.008
        );
        // a chequered band along the front
        fbox!(F, "band", 1.78, 0.1, 0.008, (0, 0.82, 0.394), M.label);
        kit_chequer!(
            F,
            "chequer",
            -0.88,
            0.88,
            0.87,
            0.398,
            0.05,
            1,
            M.police,
            rows = 2
        );
        // the security screen on the ledge: glass in an aluminium frame, the hatch under its middle pane
        let y = LEDGE + 0.03;
        let h = 0.8;
        let zs = 0.1;
        fbox!(
            F,
            "screen_rail",
            1.92,
            0.04,
            0.05,
            (0, y + h, zs),
            M.frame,
            bevel = 0.006
        );
        for (i, x) in [-0.94, -0.3, 0.3, 0.94].into_iter().enumerate() {
            fbox!(
                F,
                format!("mullion{i}"),
                0.04,
                h,
                0.05,
                (x, y + h / 2.0, zs),
                M.frame,
                bevel = 0.005
            );
        }
        for s in [-1, 1] {
            fbox!(
                F,
                format!("pane{s}"),
                0.6,
                h - 0.02,
                0.008,
                (s as f64 * 0.62, y + h / 2.0, zs),
                M.glass
            );
        }
        fbox!(
            F,
            "pane_middle",
            0.56,
            h - 0.2,
            0.008,
            (0, y + 0.19 + (h - 0.2) / 2.0, zs),
            M.glass
        );
        fbox!(
            F,
            "hatch_rail",
            0.56,
            0.03,
            0.05,
            (0, y + 0.175, zs),
            M.frame,
            bevel = 0.005
        );
        artkit::bevel::eased(revolve!(
            "grille",
            &[(0.0, 0.0), (0.06, 0.0), (0.06, 0.008), (0.0, 0.008)],
            at = F.at(0, y + 0.42, zs + 0.004),
            material = M.frame,
            segments = 16,
            rot = (90, 0, 0)
        ));
        open_book("log_book", 0.36, 0.26, (0.0, y, 0.29), M.counter, 0.0, 0.02);
        rod(
            "pen",
            (0.24, y + 0.006, 0.25),
            (0.36, y + 0.006, 0.3),
            0.005,
            M.monitor,
            6,
        );
        tube(
            "pen_chain",
            &[
                (0.36, y + 0.006, 0.3),
                (0.42, y + 0.003, 0.33),
                (0.45, y + 0.003, 0.18),
            ],
            0.0025,
            M.frame,
            0.02,
            4,
        );
        monitor("monitor", -0.2, WORK, -0.1, 0.5, 0.3, -1);
        keyboard("keyboard", -0.2, WORK, -0.28, 0.42);
        // a radio on its charger
        fbox!(
            F,
            "charger",
            0.1,
            0.04,
            0.12,
            (0.25, WORK + 0.02, -0.2),
            M.radio,
            bevel = 0.006
        );
        fbox!(
            F,
            "radio",
            0.06,
            0.13,
            0.035,
            (0.25, WORK + 0.1, -0.2),
            M.radio,
            bevel = 0.01
        );
        artkit::bevel::eased(fcyl!(
            F,
            "aerial",
            0.006,
            0.08,
            (0.265, WORK + 0.2, -0.2),
            M.radio,
            verts = 6
        ));
        fbox!(
            F,
            "radio_screen",
            0.04,
            0.03,
            0.004,
            (0.25, WORK + 0.13, -0.218),
            M.screen
        );
    });
    join(&parts, "desk", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

// --- the teller counter: 2x1, oak under glass, the teller behind ---------------------------------------------
pub fn teller_counter() -> Vec<Obj> {
    let parts = new_parts(|| {
        desk_carcass(M.teller, M.laminate, true);
        for (i, x) in [-0.595, 0.0, 0.595].into_iter().enumerate() {
            kit_shaker!(
                F,
                format!("panel{i}"),
                0.57,
                0.84,
                (x, 0.56, 0.398),
                0.022,
                M.teller,
                rail = 0.07,
                raised = true
            );
        }
        fbox!(
            F,
            "ledge",
            1.92,
            0.035,
            0.42,
            (0, LEDGE + 0.0175, 0.225),
            M.teller,
            bevel = 0.008
        );
        let y = LEDGE + 0.035;
        let h = 0.85;
        let zs = 0.08;
        // the glass from the ledge nearly to the ceiling's height, oak rails top and bottom, a grille to speak
        // through, a deal tray under it for the notes
        fbox!(
            F,
            "screen_sill",
            1.92,
            0.05,
            0.08,
            (0, y + 0.025, zs),
            M.teller,
            bevel = 0.008
        );
        fbox!(
            F,
            "screen_rail",
            1.92,
            0.06,
            0.08,
            (0, y + h, zs),
            M.teller,
            bevel = 0.01
        );
        for (i, x) in [-0.94, 0.0, 0.94].into_iter().enumerate() {
            fbox!(
                F,
                format!("post{i}"),
                0.04,
                h - 0.05,
                0.06,
                (x, y + h / 2.0 + 0.02, zs),
                M.teller,
                bevel = 0.006
            );
        }
        for s in [-1, 1] {
            let sf = s as f64;
            fbox!(
                F,
                format!("pane{s}"),
                0.9,
                h - 0.08,
                0.008,
                (sf * 0.47, y + 0.05 + (h - 0.08) / 2.0, zs),
                M.glass
            );
            revolve!(
                format!("grille{s}"),
                &[
                    (0.0, 0.0),
                    // its back rim on the glass cut across 4 mm
                    (0.061, 0.0),
                    (0.065, 0.004),
                    (0.065, 0.006),
                    (0.055, 0.01),
                    (0.0, 0.01)
                ],
                at = F.at(sf * 0.47, y + 0.42, zs + 0.004),
                material = M.frame,
                segments = 18,
                rot = (90, 0, 0)
            );
            for k in 0..4 {
                fbox!(
                    F,
                    format!("grille{s}_slot{k}"),
                    0.07,
                    0.006,
                    0.002,
                    (sf * 0.47, y + 0.39 + k as f64 * 0.02, zs + 0.015),
                    M.slot
                );
            }
            // the deal tray, sunk through the ledge under the glass
            fbox!(
                F,
                format!("tray{s}"),
                0.32,
                0.012,
                0.26,
                (sf * 0.47, y + 0.004, zs),
                M.frame,
                bevel = 0.004
            );
            fbox!(
                F,
                format!("tray{s}_dish"),
                0.28,
                0.004,
                0.22,
                (sf * 0.47, y + 0.011, zs),
                M.slot
            );
            // on the teller's side: notes and coins in a tray, a stamp
            fbox!(
                F,
                format!("cash{s}"),
                0.36,
                0.05,
                0.28,
                (sf * 0.5, WORK + 0.025, -0.25),
                M.monitor,
                bevel = 0.004
            );
            for k in 0..4 {
                let kf = k as f64;
                fbox!(
                    F,
                    format!("notes{s}{k}"),
                    0.06,
                    0.012,
                    0.13,
                    (sf * 0.5 - 0.12 + kf * 0.08, WORK + 0.054, -0.3),
                    M.note
                );
                artkit::bevel::eased(fcyl!(
                    F,
                    format!("coins{s}{k}"),
                    0.014,
                    0.02,
                    (sf * 0.5 - 0.12 + kf * 0.08, WORK + 0.06, -0.17),
                    M.coin,
                    verts = 8
                ));
            }
        }
        // in front of the cash trays (their front at -0.11), behind the inner panel (0.06)
        monitor("monitor", 0.5, WORK, -0.03, 0.4, 0.26, -1);
        keyboard("keyboard", -0.35, WORK, -0.03, 0.36);
        // a pen on a chain for the customers
        rod(
            "pen",
            (0.0, y + 0.006, 0.3),
            (0.14, y + 0.006, 0.36),
            0.005,
            M.monitor,
            6,
        );
        tube(
            "pen_chain",
            &[
                (0.0, y + 0.006, 0.3),
                (-0.05, y + 0.003, 0.25),
                (-0.02, y + 0.003, 0.14),
            ],
            0.0025,
            M.frame,
            0.02,
            4,
        );
    });
    join(&parts, "counter", Some((0, 0, 0).into()));
    settle_all();
    meshes()
}

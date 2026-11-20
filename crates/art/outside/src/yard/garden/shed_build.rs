//! The small shed as parts are given them.

use super::*;

pub fn shed_small() -> Vec<Obj> {
    shed_walls();
    shed_roof();
    if !is_low() {
        shed_inside();
    }
    // the window, right of the door: a moulded white frame mitred round four dark panes and their bars; a sill
    let (wx, wy) = (0.42, 1.12);
    let profile: &[(f64, f64)] = if !is_low() {
        &[
            (0.0, 0.0),
            (0.0, 0.03),
            (0.012, 0.04),
            (0.045, 0.04),
            (0.055, 0.03),
            (0.055, 0.0),
        ]
    } else {
        &[(0.0, 0.0), (0.0, 0.04), (0.055, 0.04), (0.055, 0.0)]
    };
    kit::moulding!(
        F,
        "window_frame",
        &[
            (wx + 0.28, wy - 0.23),
            (wx + 0.28, wy + 0.23),
            (wx - 0.28, wy + 0.23),
            (wx - 0.28, wy - 0.23),
        ],
        profile,
        M.trim,
        at = FRONT_Z - 0.002
    );
    fbox!(
        F,
        "window_pane",
        0.46,
        0.36,
        0.02,
        (wx, wy, FRONT_Z + 0.002),
        M.pane
    );
    fbox!(
        F,
        "window_bar_v",
        0.022,
        0.36,
        0.02,
        (wx, wy, FRONT_Z + 0.014),
        M.trim
    );
    fbox!(
        F,
        "window_bar_h",
        0.46,
        0.022,
        0.02,
        (wx, wy, FRONT_Z + 0.014),
        M.trim
    );
    fextrude!(
        F,
        "window_sill",
        &[(-0.33, 0.0), (0.33, 0.0), (0.33, 0.035), (-0.33, 0.035)],
        0.08,
        (wx, wy - 0.262, FRONT_Z + 0.04),
        M.trim,
        bevel = low(0.008, 0.0),
        bevel_segments = 1
    );
    // the window box on brackets, in flower
    let b = F.at(wx, 0, FRONT_Z + 0.095);
    let (bx, by) = (b.x, b.y);
    let bx_ = [(-0.3, -0.085), (0.3, -0.085), (0.3, 0.085), (-0.3, 0.085)];
    let secs: Vec<Vec<P3>> = [(-0.43, 0.9), (-0.29, 1.0), (-0.285, 0.96)]
        .iter()
        .map(|&(dy, k)| {
            bx_.iter()
                .map(|&(u, v)| P3 {
                    x: bx - u * k,
                    y: by - v * k,
                    z: wy + dy,
                })
                .collect()
        })
        .collect();
    loft!("window_box", &secs, material = M.box_wood);
    for s in [-1.0, 1.0] {
        fextrude!(
            F,
            numbered("bracket"),
            &[(0.0, 0.0), (0.0, 0.14), (0.15, 0.14), (0.15, 0.12)],
            0.03,
            (wx + s * 0.22, wy - 0.57, FRONT_Z),
            M.box_wood,
            plane = Facing::Side,
            bevel = low(0.004, 0.0),
            bevel_segments = 1
        );
    }
    let flowers = [M.flower_pink, M.flower_yellow, M.flower_red, M.flower_lilac];
    for i in 0..4 {
        let x = wx - 0.21 + i as f64 * 0.14;
        blob(
            x,
            wy - 0.3,
            FRONT_Z + 0.1,
            0.07,
            M.green,
            (30 + i) as f64,
            (1.0, 1.0, 0.8),
            0.35,
        );
        for k in 0..3 {
            fsphere!(
                F,
                numbered("flower"),
                0.025,
                (
                    x + (k - 1) as f64 * 0.035,
                    wy - 0.2 + 0.01 * (k % 2) as f64,
                    FRONT_Z + 0.09 + 0.03 * (k - 1) as f64
                ),
                flowers[((i + k) % 4) as usize],
                segments = low(8, 4),
                rings = low(5, 3)
            );
        }
    }
    if !is_low() {
        join(&meshes(), "shed", Some(P3::from((0, 0, 0))));
    }
    // The door: ledged and braced boards, black T-hinges on the left, a ring latch on the right; it swings out.
    let door = new_parts(|| {
        let (y0, x0) = (0.09, DOOR_X - DOOR_W / 2.0);
        let dz = FRONT_Z + 0.015;
        for k in 0..5 {
            let w = (DOOR_W - 0.02) / 5.0;
            fbox!(
                F,
                numbered("door_board"),
                w - 0.006,
                DOOR_H - 0.02,
                0.03,
                (
                    x0 + 0.01 + w * (k as f64 + 0.5),
                    y0 + (DOOR_H - 0.02) / 2.0,
                    dz
                ),
                M.shed_door,
                bevel = low(0.004, 0.0),
                bevel_segments = 1
            );
        }
        for y in [0.3, 1.27] {
            fbox!(
                F,
                numbered("ledge"),
                DOOR_W - 0.08,
                0.1,
                0.02,
                (DOOR_X, y, dz + 0.024),
                M.shed_door,
                bevel = low(0.005, 0.0),
                bevel_segments = 1
            );
            if is_low() {
                fbox!(
                    F,
                    numbered("hinge"),
                    0.3,
                    0.03,
                    0.008,
                    (x0 + 0.15, y, dz + 0.038),
                    M.iron
                );
            } else {
                t_hinge(&numbered("hinge"), x0 + 0.02, y, dz + 0.0365, 0.32, M.iron);
            }
        }
        board(
            "brace",
            (x0 + 0.08, 0.36, dz + 0.024),
            (x0 + DOOR_W - 0.08, 1.21, dz + 0.024),
            0.09,
            0.02,
            M.shed_door,
            (0, 0, 1),
            low(0.005, 0.0),
        );
        let lx = x0 + DOOR_W - 0.08;
        fextrude!(
            F,
            "latch_plate",
            &rounded_rect(0.06, 0.15, 0.012, low(2, 1)),
            0.006,
            (lx, 0.86, dz + 0.018),
            M.iron
        );
        if !is_low() {
            torus!(
                "latch_ring",
                0.03,
                0.005,
                at = F.at(lx, 0.83, dz + 0.026),
                material = M.iron,
                rot = (90, 0, 0),
                major_segments = 12,
                minor_segments = 4
            );
            fcyl!(
                F,
                "latch_boss",
                0.012,
                0.012,
                (lx, 0.86, dz + 0.026),
                M.iron,
                rot = (90, 0, 0),
                verts = 8
            );
            fbox!(
                F,
                "latch_bar",
                0.12,
                0.012,
                0.012,
                (lx + 0.05, 0.9, dz + 0.024),
                M.iron,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
    });
    if is_low() {
        return settle!(&[join_all("shedSmall")], centre = false);
    }
    let door = hinged(
        join(&door, "door", None),
        F.at(DOOR_X - DOOR_W / 2.0, 0.8, FRONT_Z + 0.015),
    );
    let objs = settle!(centre = false);
    swing(
        door,
        "door_open",
        100.0,
        Swing {
            seconds: 1.0,
            ease: Ease::BackOut,
            ..Swing::default()
        },
    );
    objs
}

//! The barn's doors and windows: the big sliding doors and the hayloft at the front, the small door and loft door
//! at the back, and the four-pane windows down each side.

use super::barn::BW;
use super::common::{F, M, frame, prism, pts, strut};
use artkit::json::float_repr;
use artkit::place::Coord;
use artkit::place::N;
use artkit::{cube, cylinder, fbox, kit_knob, kit_pull, torus};

/// The big doors (front, s=1) or the small one (back, s=-1) on the gable face at depth `z`.
pub(super) fn barn_doors(z: f64, s: i32, detail: bool) {
    if s > 0 {
        let (x0, x1, top) = (-1.3, 1.3, 2.6);
        prism(
            "door_dark",
            &pts(&[(x0, 0.3), (x1, 0.3), (x1, top), (x0, top)]),
            z,
            z + 0.02,
            M.shadow,
            'z',
        );
        let gap = 0.02;
        for (side, (a, b)) in [("l", (x0, -gap)), ("r", (gap, x1))] {
            prism(
                &format!("leaf_{side}"),
                &pts(&[(a, 0.3), (b, 0.3), (b, top), (a, top)]),
                z,
                z + 0.06,
                M.barn,
                'z',
            );
            frame(
                &format!("leaf_{side}_frame"),
                a + 0.06,
                0.36,
                b - 0.06,
                top - 0.06,
                z + 0.06,
                0.12,
                0.04,
                M.barn_trim,
                'z',
            );
            strut(
                &format!("leaf_{side}_x1"),
                (a + 0.1, 0.4),
                (b - 0.1, top - 0.1),
                0.1,
                z + 0.06,
                z + 0.09,
                M.barn_trim,
                'z',
            );
            strut(
                &format!("leaf_{side}_x2"),
                (b - 0.1, 0.4),
                (a + 0.1, top - 0.1),
                0.1,
                z + 0.06,
                z + 0.09,
                M.barn_trim,
                'z',
            );
            if detail {
                let hx = if side == "l" { -0.16 } else { 0.16 };
                kit_pull!(
                    F,
                    format!("handle_{side}"),
                    hx,
                    1.35,
                    z + 0.09,
                    0.34,
                    M.iron,
                    r = 0.014,
                    out = 0.06,
                    upright = true,
                    verts = 6
                );
                fbox!(
                    F,
                    format!("guide_{side}"),
                    0.12,
                    0.06,
                    0.05,
                    ((a + b) / 2.0, 0.33, z + 0.085),
                    M.iron,
                    bevel = 0.008,
                    bevel_segments = 1
                );
            }
        }
        // the rail the leaves hang from, and its hangers
        cube!(
            "rail",
            (2.0 * x1 + 1.6, 0.08, 0.1),
            at = F.at(0, top + 0.12, z + 0.06),
            material = M.iron
        );
        if detail {
            for x in [-1.0, -0.3, 0.3, 1.0] {
                cylinder!(
                    format!("hanger{}", float_repr(x)),
                    0.05,
                    0.05,
                    at = F.at(x, top + 0.12, z + 0.13),
                    material = M.iron,
                    rot = (90, 0, 0),
                    verts = 10
                );
            }
        }
        // the hayloft door and the hay beam above it
        let (hx, hy0, hy1) = (0.5, 3.2, 4.1);
        prism(
            "loft",
            &pts(&[(-hx, hy0), (hx, hy0), (hx, hy1), (-hx, hy1)]),
            z,
            z + 0.05,
            M.barn,
            'z',
        );
        frame(
            "loft_frame",
            -hx,
            hy0,
            hx,
            hy1,
            z + 0.05,
            0.12,
            0.04,
            M.barn_trim,
            'z',
        );
        strut(
            "loft_x1",
            (-hx + 0.06, hy0 + 0.06),
            (hx - 0.06, hy1 - 0.06),
            0.09,
            z + 0.05,
            z + 0.08,
            M.barn_trim,
            'z',
        );
        strut(
            "loft_x2",
            (hx - 0.06, hy0 + 0.06),
            (-hx + 0.06, hy1 - 0.06),
            0.09,
            z + 0.05,
            z + 0.08,
            M.barn_trim,
            'z',
        );
        if detail {
            for y in [hy0 + 0.2, hy1 - 0.2] {
                // its strap hinges, on the left
                fbox!(
                    F,
                    format!("loft_hinge{}", float_repr(y)),
                    0.36,
                    0.045,
                    0.015,
                    (-hx + 0.2, y, z + 0.1),
                    M.iron,
                    bevel = 0.004,
                    bevel_segments = 1
                );
                cylinder!(
                    format!("loft_pin{}", float_repr(y)),
                    0.018,
                    0.07,
                    at = F.at(-hx - 0.02, y, z + 0.1),
                    material = M.iron,
                    verts = 6
                );
            }
            kit_knob!(
                F,
                "loft_knob",
                hx - 0.14,
                (hy0 + hy1) / 2.0,
                z + 0.09,
                0.025,
                M.iron,
                segments = 8
            );
            cube!(
                "hay_beam",
                (0.14, 0.24, 0.14),
                at = F.at(0, 4.4, z + 0.11),
                material = M.barn_trim
            );
            torus!(
                "hay_pulley",
                0.06,
                0.016,
                at = F.at(0, 4.27, z + 0.17),
                material = M.iron,
                rot = (0, 90, 0),
                major_segments = 12,
                minor_segments = 6
            );
        }
    } else {
        let z0 = z - 0.06;
        prism(
            "back_door",
            &pts(&[(0.6, 0.3), (1.6, 0.3), (1.6, 2.2), (0.6, 2.2)]),
            z0,
            z,
            M.barn,
            'z',
        );
        frame(
            "back_door_frame",
            0.6,
            0.3,
            1.6,
            2.2,
            z0 - 0.04,
            0.1,
            0.04,
            M.barn_trim,
            'z',
        );
        strut(
            "back_door_x",
            (0.66, 0.36),
            (1.54, 2.14),
            0.09,
            z0 - 0.04,
            z0,
            M.barn_trim,
            'z',
        );
        prism(
            "back_loft",
            &pts(&[(-0.4, 3.3), (0.4, 3.3), (0.4, 4.0), (-0.4, 4.0)]),
            z0,
            z,
            M.barn,
            'z',
        );
        frame(
            "back_loft_frame",
            -0.4,
            3.3,
            0.4,
            4.0,
            z0 - 0.04,
            0.1,
            0.04,
            M.barn_trim,
            'z',
        );
    }
}

/// A four-pane window on a side wall (axis 'x': at=(depth, height), the wall at x s * BW).
pub(super) fn barn_window(name: &str, at: (N, f64), axis: char, s: i32) {
    let (u, v) = at;
    let uf = u.f();
    let (w, h) = (0.38, 0.38);
    let s = s as f64;
    let x = s * BW;
    prism(
        &format!("{name}_pane"),
        &pts(&[
            (uf - w, v - h),
            (uf + w, v - h),
            (uf + w, v + h),
            (uf - w, v + h),
        ]),
        x,
        x + s * 0.02,
        M.shadow,
        axis,
    );
    frame(
        &format!("{name}_frame"),
        uf - w,
        v - h,
        uf + w,
        v + h,
        x,
        0.1,
        s * 0.05,
        M.barn_trim,
        axis,
    );
    prism(
        &format!("{name}_mv"),
        &pts(&[
            (uf - 0.025, v - h),
            (uf + 0.025, v - h),
            (uf + 0.025, v + h),
            (uf - 0.025, v + h),
        ]),
        x,
        x + s * 0.04,
        M.barn_trim,
        axis,
    );
    prism(
        &format!("{name}_mh"),
        &pts(&[
            (uf - w, v - 0.025),
            (uf + w, v - 0.025),
            (uf + w, v + 0.025),
            (uf - w, v + 0.025),
        ]),
        x,
        x + s * 0.04,
        M.barn_trim,
        axis,
    );
    // a sloped sill under it and a drip cap over it
    fbox!(
        F,
        format!("{name}_sill"),
        0.11,
        0.045,
        2.0 * w + 0.22,
        (x + s * 0.05, v - h - 0.075, u),
        M.barn_trim,
        bevel = 0.012,
        bevel_segments = 1
    );
    fbox!(
        F,
        format!("{name}_cap"),
        0.08,
        0.07,
        2.0 * w + 0.26,
        (x + s * 0.04, v + h + 0.085, u),
        M.barn_trim,
        bevel = 0.012,
        bevel_segments = 1
    );
}

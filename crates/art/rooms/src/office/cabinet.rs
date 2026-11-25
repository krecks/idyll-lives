//! O3: the storage: the filing cabinet (the boards are boards.rs).

use super::{F, M, WALL};
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{TAU, cos, sin};
use artkit::parts::{join, new_parts};
use artkit::place::Facing;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, kit, revolve, settle, slide};

// --- storage and boards ----------------------------------------------------------------------------------------
/// A steel drawer front, pressed with a panel line, a bar handle and a label card in its chrome holder.
pub(super) fn cabinet_front(name: &str, y: f64, front: f64, label: bool) {
    kit::shaker!(
        F,
        name,
        0.42,
        0.3,
        (0, y, front + 0.01),
        0.02,
        M.steel,
        rail = 0.028,
        recess = 0.003
    );
    kit::pull!(
        F,
        format!("{name}_handle"),
        0,
        y + 0.06,
        front + 0.02,
        0.15,
        M.chrome,
        r = 0.006,
        out = 0.012
    );
    if label {
        let holder: Vec<(f64, f64, f64)> = rounded_rect(0.1, 0.04, 0.004, 1)
            .iter()
            .map(|&(u, v)| (u, y + 0.11 + v, front + 0.021))
            .collect();
        fsweep!(
            F,
            format!("{name}_holder"),
            &holder,
            0.0022,
            M.chrome,
            verts = 4,
            closed = true
        );
        fbox!(
            F,
            format!("{name}_label"),
            0.09,
            0.032,
            0.002,
            (0, y + 0.11, front + 0.0205),
            M.label
        );
        fbox!(
            F,
            format!("{name}_writing"),
            0.06,
            0.004,
            0.0004,
            (0, y + 0.112, front + 0.0217),
            M.pen
        );
    }
}

/// `n` long leaves arching out of a pot at (x, y, z): each a flat blade swept up and over, narrowing to its tip.
#[allow(clippy::too_many_arguments)]
pub(super) fn leaves(
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    n: i32,
    length: f64,
    material: Mat,
    seed: f64,
) {
    for k in 0..n {
        let k_ = f64::from(k);
        let a = TAU * k_ / f64::from(n) + 0.4 * sin(k_ * 2.3 + seed);
        let (c, s) = (cos(a), sin(a));
        let l = length * (0.75 + 0.25 * artkit::math::pow(sin(k_ * 1.7 + seed), 2.0));
        let lift = 0.75 + 0.2 * cos(k_ * 1.3);
        let pts = [
            (x + c * 0.01, y, z + s * 0.01),
            (x + c * l * 0.25, y + l * lift * 0.75, z + s * l * 0.25),
            (x + c * l * 0.55, y + l * lift, z + s * l * 0.55),
            (x + c * l * 0.85, y + l * lift * 0.82, z + s * l * 0.85),
        ];
        fsweep!(
            F,
            format!("{name}{k}"),
            &pts,
            0.0,
            material,
            profile = [(0.0, -0.016), (0.003, 0.0), (0.0, 0.016), (-0.0015, 0.0)],
            up = (0, 0, 1),
            angle = 180.0, // a leaf: thin, its margins shade round
            resolution = 2,
            scale = [0.4, 1.0, 0.8, 0.15]
        );
    }
}

pub fn filing_cabinet() -> Vec<Obj> {
    let (z0, d) = (WALL, 0.6);
    let front = z0 + d;
    let still = new_parts(|| {
        fbox!(
            F,
            "cabinet",
            0.46,
            0.975,
            d,
            (0, 0.5125, z0 + d / 2.0),
            M.steel,
            bevel = 0.01,
            bevel_segments = 2
        );
        fbox!(
            F,
            "kick",
            0.42,
            0.025,
            d - 0.04,
            (0, 0.0125, z0 + d / 2.0 - 0.02),
            M.reveal,
            bevel = 0.004,
            bevel_segments = 1
        );
        fbox!(
            F,
            "top_lip",
            0.46,
            0.008,
            d,
            (0, 0.996, z0 + d / 2.0),
            M.steel,
            bevel = 0.003,
            bevel_segments = 1
        );
        // the drawers' dark gaps, then the two still fronts (the top one moves)
        fbox!(
            F,
            "gaps",
            0.43,
            0.94,
            0.002,
            (0, 0.5, front + 0.0005),
            M.reveal
        );
        for (i, y) in [0.175, 0.5].into_iter().enumerate() {
            cabinet_front(&format!("front{i}"), y, front, true);
        }
        fcyl!(
            F,
            "lock",
            0.012,
            0.01,
            (0.17, 0.975, front + 0.005),
            M.chrome,
            rot = (90, 0, 0),
            verts = 12
        );
        fbox!(
            F,
            "lock_key",
            0.003,
            0.012,
            0.002,
            (0.17, 0.975, front + 0.0102),
            M.reveal
        );
        // a letter tray of papers and a potted plant on top
        let y0 = 1.0;
        let side = [
            (-0.15, 0.0),
            (0.15, 0.0),
            (0.15, 0.03),
            (0.06, 0.04),
            (-0.15, 0.06),
        ];
        for s in [-1, 1] {
            fextrude!(
                F,
                format!("tray_side{s}"),
                &side,
                0.006,
                (-0.1 + f64::from(s) * 0.117, y0, -0.2),
                M.plastic,
                plane = Facing::Side
            );
        }
        fbox!(
            F,
            "tray_bottom",
            0.24,
            0.006,
            0.3,
            (-0.1, y0 + 0.003, -0.2),
            M.plastic
        );
        fbox!(
            F,
            "tray_back",
            0.24,
            0.06,
            0.006,
            (-0.1, y0 + 0.03, -0.347),
            M.plastic
        );
        for k in 0..3 {
            fbox!(
                F,
                format!("papers{k}"),
                0.21,
                0.004,
                0.28,
                (
                    -0.1 + 0.004 * f64::from(k),
                    y0 + 0.008 + 0.005 * f64::from(k),
                    -0.2 - 0.003 * f64::from(k)
                ),
                M.paper,
                rot = (0, 0, 2 * k - 2)
            );
        }
        revolve!(
            "pot",
            &[
                (0.0, 0.0),
                (0.05, 0.0),
                (0.06, 0.09),
                (0.068, 0.095),
                (0.068, 0.11),
                (0.06, 0.11),
                (0.058, 0.1),
                (0.0, 0.1)
            ],
            at = F.at(0.12, y0, -0.2),
            material = M.pot,
            segments = 18
        );
        fcyl!(
            F,
            "pot_soil",
            0.059,
            0.004,
            (0.12, y0 + 0.1, -0.2),
            M.soil,
            verts = 18
        );
        leaves("leaf", 0.12, y0 + 0.1, -0.2, 9, 0.12, M.leaf, 1.0);
    });
    join(&still, "filingCabinet", Some((0, 0, 0).into()));
    // the top drawer with its hanging files behind it, sliding out while a resident files papers
    let parts = new_parts(|| {
        cabinet_front("drawer", 0.825, front, true);
        fbox!(
            F,
            "drawer_bottom",
            0.38,
            0.015,
            0.5,
            (0, 0.7, front - 0.25),
            M.steel,
            bevel = 0.003
        );
        for s in [-1, 1] {
            fbox!(
                F,
                format!("drawer_side{s}"),
                0.015,
                0.22,
                0.5,
                (f64::from(s) * 0.19, 0.8, front - 0.25),
                M.steel,
                bevel = 0.003
            );
            fbox!(
                F,
                format!("drawer_rail{s}"),
                0.006,
                0.008,
                0.5,
                (f64::from(s) * 0.175, 0.905, front - 0.25),
                M.chrome
            );
        }
        fbox!(
            F,
            "drawer_back",
            0.38,
            0.22,
            0.015,
            (0, 0.8, front - 0.5),
            M.steel,
            bevel = 0.003
        );
        let tabs = [M.note, M.note4, M.note3, M.note, M.note2, M.note4];
        for k in 0..6 {
            let zf = front - 0.06 - f64::from(k) * 0.07;
            fbox!(
                F,
                format!("file{k}"),
                0.34,
                0.19,
                0.008,
                (0, 0.81, zf),
                M.folder,
                bevel = 0.002,
                bevel_segments = 1
            );
            fbox!(
                F,
                format!("file_hook{k}"),
                0.36,
                0.006,
                0.006,
                (0, 0.905, zf),
                M.folder
            );
            fbox!(
                F,
                format!("file_tab{k}"),
                0.05,
                0.022,
                0.009,
                (-0.12 + 0.05 * f64::from(k % 5), 0.915, zf),
                tabs[k as usize]
            );
            fbox!(
                F,
                format!("file_papers{k}"),
                0.3,
                0.17,
                0.004,
                (0.005, 0.8, zf + 0.006),
                M.paper
            );
        }
    });
    let drawer = join(&parts, "drawer", Some(F.at(0, 0.825, front)));
    settle!(centre = false);
    slide!(drawer, "drawer_open", (0, -0.3, 0), seconds = 1.2);
    meshes()
}

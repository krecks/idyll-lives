//! The scarecrow: a crooked post and cross-bar in a plaid shirt and patched trousers, straw at its ends, a sack
//! head under a floppy hat, swaying in the wind.

use super::common::{F, M};
use artkit::forms::rounded_rect;
use artkit::math::{Axis, P3, TAU, atan2, cos, cosd, pow, radians, sin, sind, sqrt};
use artkit::parts::{join, settle_all};
use artkit::py;
use artkit::scene::{Obj, meshes};
use artkit::{
    cube, cylinder, fbox, fcone, fsphere, fsweep, jitter, loft, revolve, ring, rock, soft, sphere,
    sweep, torus,
};

// --- the scarecrow ---------------------------------------------------------------------------------------
const POST_Z: f64 = -0.05;

/// A fan of straw ends poking out at `at` (a Blender point) along `direction` (Blender (x, y, z)), each a thin
/// tapering stalk, bent a little.
fn straw_tuft(name: &str, at: P3, direction: (f64, f64, f64), n: usize, length: f64, spread: f64) {
    let (dx, dy, dz) = direction;
    let k = sqrt(dx * dx + dy * dy + dz * dz);
    let (dx, dy, dz) = (dx / k, dy / k, dz / k);
    for i in 0..n {
        let fi = i as f64;
        let t = (fi - (n - 1) as f64 / 2.0) / 1.max(n - 1) as f64;
        let a = radians(spread * t * 2.0);
        // turn the direction about the vertical a little, and droop alternate ones
        let ex = dx * cos(a) - dy * sin(a);
        let ey = dx * sin(a) + dy * cos(a);
        let ez = dz - if i % 2 != 0 { 0.25 } else { 0.05 };
        let ln = length * (0.85 + 0.3 * py::fmod(fi * 0.618, 1.0));
        let mid = (
            at.x + ex * ln * 0.5,
            at.y + ey * ln * 0.5,
            at.z + ez * ln * 0.5 + 0.006,
        );
        let end = (at.x + ex * ln, at.y + ey * ln, at.z + (ez - 0.12) * ln);
        sweep!(
            format!("{name}{i}"),
            &[(at.x, at.y, at.z), mid, end],
            0.008,
            material = M.straw,
            verts = 3,
            scale = [1.0, 0.8, 0.3],
            smooth = false
        );
    }
}

/// The shirt's body, a rounded section at each height: (height, half its width, half its depth).
const TORSO: [(f64, f64, f64); 6] = [
    (0.97, 0.205, 0.125),
    (1.1, 0.2, 0.12),
    (1.3, 0.205, 0.118),
    (1.47, 0.215, 0.115),
    (1.54, 0.18, 0.1),
    (1.585, 0.085, 0.07),
];
const SQ: f64 = 3.0;

/// How far in front of the post the shirt's front is at (x, height y): on its rounded section there.
fn torso_depth(x: f64, y: f64) -> f64 {
    for (&(y0, rx0, rz0), &(y1, rx1, rz1)) in TORSO.iter().zip(&TORSO[1..]) {
        if y <= y1 {
            let k = 0.0f64.max((y - y0) / (y1 - y0));
            let (rx, rz) = (rx0 + (rx1 - rx0) * k, rz0 + (rz1 - rz0) * k);
            return rz * pow(0.0f64.max(1.0 - pow((x / rx).abs(), SQ)), 1.0 / SQ);
        }
    }
    0.0
}

pub fn scarecrow() -> Vec<Obj> {
    // the post, rough and a little crooked, and the cross-bar inside the sleeves
    let post = sweep!(
        "post",
        &[
            F.at(0, 0.0, POST_Z),
            F.at(0.006, 0.9, POST_Z + 0.004),
            F.at(-0.004, 1.9, POST_Z)
        ],
        0.04,
        material = M.post,
        verts = 8,
        scale = [1.05, 1.0, 0.92]
    );
    jitter!(post, 0.006, size = 0.2, seed = 1.5, up = 0.0);
    fsweep!(
        F,
        "crossbar",
        &[(-0.43, 1.45, POST_Z), (0.43, 1.45, POST_Z + 0.01)],
        0.0,
        M.post,
        profile = rounded_rect(0.06, 0.055, 0.012, 1),
        up = (0, 0, 1)
    );
    // the shirt: a body rounded in section, sleeves bagging round the bar, collar, cuffs, its plaid lines and buttons
    let sections: Vec<Vec<P3>> = TORSO
        .iter()
        .map(|&(y, rx, rz)| {
            ring!(rx, rz, z = 0, verts = 16, square = SQ)
                .iter()
                .map(|p| F.at(p.x, y, p.y))
                .collect()
        })
        .collect();
    loft!("shirt", &sections, material = M.shirt);
    for s in [-1, 1] {
        let sf = s as f64;
        sweep!(
            format!("sleeve{s}"),
            &[
                F.at(sf * 0.14, 1.47, 0),
                F.at(sf * 0.24, 1.455, 0.005),
                F.at(sf * 0.33, 1.435, -0.003),
                F.at(sf * 0.4, 1.42, 0)
            ],
            0.075,
            material = M.shirt,
            verts = 10,
            resolution = 2,
            scale = [1.0, 0.92, 1.02, 0.95]
        );
        cylinder!(
            format!("cuff{s}"),
            0.08,
            0.04,
            at = F.at(sf * 0.4, 1.42, 0),
            material = M.shirt_check,
            rot = (0, 90 + s * 12, 0),
            verts = 10
        );
        straw_tuft(
            &format!("hand{s}"),
            F.at(sf * 0.42, 1.42, 0),
            (-sf, 0.0, -0.1),
            5,
            0.07,
            18.0,
        );
        for (i, x) in [0.22, 0.3].into_iter().enumerate() {
            torus!(
                format!("check_sleeve{s}{i}"),
                0.077,
                0.008,
                at = F.at(sf * x, 1.455 - 0.15 * (x - 0.16), 0),
                material = M.shirt_check,
                rot = (0, 90 + s * 12, 0),
                major_segments = 10,
                minor_segments = 4
            );
        }
    }
    for (i, x) in [-0.12, 0.0, 0.12].into_iter().enumerate() {
        // the plaid, on the shirt's front
        let line: Vec<(f64, f64, f64)> = [1.0, 1.15, 1.3, 1.45]
            .iter()
            .map(|&y| (x, y, torso_depth(x, y) + 0.004))
            .collect();
        fsweep!(
            F,
            format!("check_v{i}"),
            &line,
            0.009,
            M.shirt_check,
            verts = 4,
            resolution = 2
        );
    }
    for (i, y) in [1.08, 1.22, 1.36].into_iter().enumerate() {
        let line: Vec<(f64, f64, f64)> = [-0.19, -0.1, 0.0, 0.1, 0.19]
            .iter()
            .map(|&x| (x, y, torso_depth(x, y) + 0.004))
            .collect();
        fsweep!(
            F,
            format!("check_h{i}"),
            &line,
            0.009,
            M.shirt_check,
            verts = 4,
            resolution = 2
        );
    }
    for (i, y) in [1.12, 1.25, 1.38, 1.5].into_iter().enumerate() {
        fsphere!(
            F,
            format!("button{i}"),
            0.014,
            (0.0, y, torso_depth(0.0, y) + 0.008),
            M.stitch,
            segments = 6,
            rings = 4
        );
    }
    fbox!(
        F,
        "pocket",
        0.09,
        0.09,
        0.012,
        (-0.1, 1.38, torso_depth(-0.1, 1.38) + 0.002),
        M.shirt_check,
        bevel = 0.004,
        bevel_segments = 1
    );
    soft!(
        F,
        "collar",
        0.26,
        0.05,
        0.2,
        (0, 1.58, 0),
        M.shirt_check,
        round = 0.02,
        round_segments = 1
    );
    // trousers hanging below, a patch on one knee, straw at the ankles, a rope belt tied in front
    soft!(
        F,
        "seat",
        0.38,
        0.16,
        0.22,
        (0, 0.95, 0),
        M.trousers,
        round = 0.05,
        round_segments = 2
    );
    for s in [-1, 1] {
        let sf = s as f64;
        sweep!(
            format!("leg{s}"),
            &[
                F.at(sf * 0.09, 0.97, 0),
                F.at(sf * 0.105, 0.72, 0.012),
                F.at(sf * 0.12, 0.49, 0.022)
            ],
            0.072,
            material = M.trousers,
            verts = 10,
            resolution = 2,
            scale = [1.0, 0.92, 1.05]
        );
        fsweep!(
            F,
            format!("hem{s}"),
            &[(sf * 0.12, 0.505, 0.022), (sf * 0.12, 0.485, 0.022)],
            0.079,
            M.trousers,
            verts = 10
        );
        straw_tuft(
            &format!("ankle{s}"),
            F.at(sf * 0.12, 0.47, 0.02),
            (sf * 0.3, 0.0, -1.0),
            5,
            0.09,
            18.0,
        );
    }
    soft!(
        F,
        "patch",
        0.09,
        0.1,
        0.016,
        (0.105, 0.7, 0.084),
        M.patch,
        round = 0.006,
        round_segments = 1
    );
    for k in 0..4 {
        // its big stitches
        fbox!(
            F,
            format!("patch_stitch{k}"),
            0.005,
            0.02,
            0.006,
            (0.06 + 0.03 * k as f64, 0.755, 0.093),
            M.stitch
        );
    }
    let belt: Vec<(f64, f64, f64)> = (0..16)
        .map(|k| {
            (
                0.22 * cos(TAU * k as f64 / 16.0),
                0.995,
                0.135 * sin(TAU * k as f64 / 16.0),
            )
        })
        .collect();
    fsweep!(F, "belt", &belt, 0.014, M.twine, verts = 5, closed = true);
    fsphere!(
        F,
        "belt_knot",
        0.026,
        (0.06, 0.99, 0.135),
        M.twine,
        segments = 8,
        rings = 5
    );
    for (k, dx) in [-0.01, 0.02].into_iter().enumerate() {
        fsweep!(
            F,
            format!("belt_end{k}"),
            &[(0.06 + dx, 0.98, 0.14), (0.065 + dx * 2.0, 0.9, 0.145)],
            0.009,
            M.twine,
            verts = 4
        );
    }
    straw_tuft(
        "waist",
        F.at(0, 0.98, 0.1),
        (0.0, -1.0, -0.4),
        4,
        0.08,
        18.0,
    );
    // the sack head, gathered and tied at the neck, with a stitched face: crossed eyes, a nose, a stitched smile
    sphere!(
        "head",
        0.17,
        at = F.at(0, 1.78, 0),
        material = M.sack,
        scale = (0.95, 0.9, 1.06),
        segments = 14,
        rings = 9
    );
    fcone!(
        F,
        "gather",
        0.1,
        0.05,
        0.08,
        (0, 1.64, 0),
        M.sack,
        verts = 10
    );
    torus!(
        "neck_tie",
        0.06,
        0.015,
        at = F.at(0, 1.635, 0),
        material = M.twine,
        major_segments = 10,
        minor_segments = 4
    );
    straw_tuft(
        "neck",
        F.at(0, 1.62, 0.04),
        (0.0, -1.0, -0.5),
        4,
        0.07,
        18.0,
    );
    for s in [-1, 1] {
        for k in [-1, 1] {
            cube!(
                format!("eye{s}{k}"),
                (0.05, 0.008, 0.012),
                at = F.at(s as f64 * 0.06, 1.82, 0.152),
                material = M.stitch,
                rot = (0, 45 * k, 0)
            );
        }
    }
    fcone!(
        F,
        "nose",
        0.025,
        0.0,
        0.07,
        (0, 1.76, 0.18),
        M.straw,
        verts = 6,
        rot = (90, 0, 0)
    );
    let smile: Vec<(f64, f64, f64)> = (-55..56)
        .step_by(22)
        .map(|a| (0.085 * sind(a as f64), 1.725 - 0.03 * cosd(a as f64), 0.0))
        .collect();
    let smile: Vec<(f64, f64, f64)> = smile
        .iter()
        .map(|&(x, y, _)| {
            (
                x,
                y,
                0.153 * sqrt(0.0f64.max(1.0 - pow(x / 0.16, 2.0) - pow((y - 1.78) / 0.18, 2.0)))
                    + 0.003,
            )
        })
        .collect();
    fsweep!(F, "smile", &smile, 0.005, M.stitch, verts = 4);
    for (i, &(x, y, z)) in smile[1..smile.len() - 1].iter().enumerate() {
        fbox!(
            F,
            format!("smile_stitch{i}"),
            0.005,
            0.03,
            0.008,
            (x, y, z),
            M.stitch
        );
    }
    // the floppy hat: a dented crown, a brim drooping in waves, a band; straw sticking out under it
    let brim = revolve!(
        "brim",
        &[
            (0.14, 0.0),
            (0.2, -0.008),
            (0.27, -0.022),
            (0.275, -0.012),
            (0.2, 0.004),
            (0.14, 0.012)
        ],
        at = F.at(0, 1.92, 0),
        material = M.hat,
        segments = 24,
        rot = (4, -3, 0)
    );
    brim.edit(|o| {
        for v in o.mesh.verts.iter_mut() {
            let r = py::hypot(v.x as f64, v.y as f64);
            if r > 0.16 {
                v.z = (v.z as f64
                    - 0.035
                        * ((r - 0.16) / 0.11)
                        * (0.5 + 0.5 * sin(3.0 * atan2(v.y as f64, v.x as f64) + 0.7)))
                    as f32;
            }
        }
    });
    revolve!(
        "crown",
        &[
            (0.0, 0.17),
            (0.06, 0.155),
            (0.1, 0.18),
            (0.12, 0.165),
            (0.145, 0.01),
            (0.148, -0.005),
            (0.0, -0.005)
        ],
        at = F.at(0, 1.93, 0),
        material = M.hat,
        segments = 14,
        rot = (4, -6, 0)
    );
    revolve!(
        "hatband",
        &[(0.146, 0.008), (0.149, 0.01), (0.146, 0.05), (0.141, 0.052)],
        at = F.at(0, 1.93, 0),
        material = M.shirt_check,
        segments = 14,
        rot = (4, -6, 0)
    );
    for s in [-1, 1] {
        let sf = s as f64;
        straw_tuft(
            &format!("hair{s}"),
            F.at(sf * 0.15, 1.86, 0),
            (-sf, 0.0, -0.6),
            4,
            0.08,
            18.0,
        );
    }
    let scare = join(&meshes(), "scarecrow", Some(F.at(0, 0, POST_Z)));
    settle_all();
    // a slow sway in the wind about the foot of its post, side to side
    rock!(scare, "sway", 2.5, seconds = 3.0, axis = Axis::Y);
    meshes()
}

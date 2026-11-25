//! The snoozecloud adjustable bed and the daydream nap pod.

use super::common::{_sphere_band, M};
use artkit::forms::smooth_outline;
use artkit::math::{Axis, P3, cos, pow, radians, sin, sqrt};
use artkit::parts::{hinged, join, new_parts, numbered};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{
    fbox, fcyl, fextrude, fsphere, fsweep, jitter, kit, revolve, settle, soft, swing, torus,
};

// --- the snoozecloud adjustable bed (2x2): two sleepers side by side, mattress top 0.49, head at the back ----
const HINGE_Z: f64 = -0.42; // the head section's front edge (depth), where it rises from
const MAT_TOP: f64 = 0.49;

/// A cloud's outline `w` across and `h` high above `base`: a row of round bumps (centre x, height, radius) along its
/// top, flat along its foot.
fn _cloud(w: f64, _h: f64, bumps: &[(f64, f64, f64)], base: f64) -> Vec<(f64, f64)> {
    let mut top = Vec::new();
    for k in 0..49 {
        let x = -w / 2.0 + w * k as f64 / 48.0;
        let mut ys: Vec<f64> = bumps
            .iter()
            .filter(|&&(cx, _, r)| (x - cx).abs() < r)
            .map(|&(cx, cy, r)| cy + sqrt(r * r - pow(x - cx, 2.0)))
            .collect();
        ys.push(base + 0.02);
        // (Python's max: the first of the greatest)
        let mut y = ys[0];
        for &v in &ys[1..] {
            if v > y {
                y = v;
            }
        }
        top.push((x, y));
    }
    let mut out = vec![(-w / 2.0, base)];
    out.extend(top);
    out.push((w / 2.0, base));
    out
}

pub fn snoozecloud_bed() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        // the base: one puffy upholstered block, piped round its top, on four turned birch feet
        soft!(
            f,
            "base",
            1.86,
            0.26,
            1.86,
            (0, 0.15, 0),
            M.body,
            round = 0.1,
            puff = (0.0, 0.0, 0.0),
            piping = 0.008,
            flat = 1
        );
        for sx in [-1, 1] {
            for sz in [-1, 1] {
                kit::turned!(
                    f,
                    numbered("foot"),
                    sx as f64 * 0.8,
                    0.0,
                    sz as f64 * 0.8,
                    &[
                        (0.0, 0.0),
                        (0.045, 0.0),
                        (0.05, 0.012),
                        (0.06, 0.035),
                        (0.062, 0.05),
                        (0.0, 0.05),
                    ],
                    M.birch,
                    segments = 12
                );
            }
        }
        // the cloud headboard: a big cloud cut from one outline, a smaller one in front of it in the other white
        let bumps = [
            (-0.66, 0.62, 0.24),
            (-0.36, 0.78, 0.3),
            (0.0, 0.86, 0.32),
            (0.36, 0.78, 0.3),
            (0.66, 0.62, 0.24),
            (-0.5, 0.98, 0.2),
            (0.48, 0.98, 0.22),
            (0.0, 1.08, 0.21),
        ];
        let out = smooth_outline(&_cloud(1.82, 1.29, &bumps, 0.3), 1, true);
        fextrude!(
            f,
            "cloud_back",
            &out,
            0.12,
            (0, 0, -0.86),
            M.body,
            bevel = 0.045,
            bevel_segments = 3,
            smooth = true,
            angle = 60
        );
        let front = [
            (-0.42, 0.62, 0.2),
            (-0.12, 0.76, 0.24),
            (0.22, 0.72, 0.22),
            (0.5, 0.6, 0.18),
        ];
        let out2 = smooth_outline(&_cloud(1.2, 0.0, &front, 0.5), 1, true);
        fextrude!(
            f,
            "cloud_front",
            &out2,
            0.05,
            (0, 0, -0.776),
            M.cloud,
            bevel = 0.022,
            bevel_segments = 2,
            smooth = true,
            angle = 60
        );
        // the foot of the mattress, quilted, a throw folded over its end, a remote on the side
        let d = 0.93 - HINGE_Z;
        soft!(
            f,
            "mattress_foot",
            1.74,
            0.2 - 0.016,
            d - 0.01,
            (0, MAT_TOP - 0.1 - 0.008, HINGE_Z + d / 2.0 + 0.005),
            M.upholstery,
            round = 0.05,
            puff = 0.016,
            piping = 0.006,
            flat = 3,
            resting = true
        );
        let throw = soft!(
            f,
            "throw",
            1.78,
            0.036,
            0.42,
            (0, MAT_TOP + 0.018, 0.66),
            M.throw,
            round = 0.016,
            puff = 0.006,
            flat = 2
        );
        jitter!(throw, 0.004, size = 0.15, seed = 2.0, up = 0.6);
        fsweep!(
            f,
            "throw_fold",
            &[(-0.89, MAT_TOP + 0.02, 0.87), (0.89, MAT_TOP + 0.02, 0.87)],
            0.02,
            M.throw,
            verts = 10
        );
        fbox!(
            f,
            "remote",
            0.06,
            0.02,
            0.14,
            (0.78, MAT_TOP + 0.016, 0.3),
            M.porcelain,
            bevel = 0.008,
            bevel_segments = 2
        );
        for k in 0..3 {
            fcyl!(
                f,
                numbered("remote_button"),
                0.008,
                0.004,
                (0.78, MAT_TOP + 0.027, 0.26 + 0.025 * k as f64),
                M.screen,
                verts = 8
            );
        }
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The head section of the mattress and its pillows, hinged on its front edge: `head_up` raises it 30°.
    let head = new_parts(|| {
        let hd = HINGE_Z + 0.77;
        soft!(
            f,
            "mattress_head",
            1.74,
            0.2 - 0.016,
            hd,
            (0, MAT_TOP - 0.1 - 0.008, HINGE_Z - hd / 2.0),
            M.upholstery,
            round = 0.05,
            puff = 0.016,
            piping = 0.006,
            flat = 3,
            resting = true
        );
        for s in [-1, 1] {
            let sf = s as f64;
            soft!(
                f,
                numbered("pillow"),
                0.68,
                0.14,
                0.36,
                (sf * 0.42, MAT_TOP + 0.06, -0.6),
                M.pillow,
                round = 0.06,
                puff = 0.03,
                flat = 3,
                resting = true
            );
            soft!(
                f,
                numbered("cushion"),
                0.22,
                0.26,
                0.1,
                (sf * 0.77, MAT_TOP + 0.15, -0.68),
                if s < 0 { M.blush } else { M.lilac },
                rot = (18, 0, 0),
                round = 0.045,
                puff = 0.025,
                flat = 2
            );
        }
    });
    let head = hinged(
        join(&head, "head_section", None),
        f.at(0, MAT_TOP - 0.2, HINGE_Z),
    );
    let objs = settle!(centre = false);
    swing!(head, "head_up", 30.0, seconds = 1.2, axis = Axis::X);
    objs
}

// --- the daydream nap pod (1x1): an egg chair open to the front, seat top 0.45, a hood that swings over -----
const POD: (f64, f64, f64) = (0.0, 0.82, 0.09); // the egg's centre (x, height, depth)
const POD_R: f64 = 0.44; // its radius across
const POD_H: f64 = 0.66; // and its half height (a grown-up sitter's crown reaches about 1.4)
const HOOD: (f64, f64, f64) = (0.0, 1.05, 0.09); // the hood's centre: a sphere about the egg's upper half, clear of a grown-up's head
const HOOD_R: f64 = 0.5; // 2026-10-21: 0.48 about 0.95, the hood closing onto the head (A-packs-04)

pub fn nap_pod() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let (cx, cy, cz) = POD;
    let still = new_parts(|| {
        // the egg: an ellipsoid open at the front, revolved about the depth axis and stretched upwards, its lip rounded
        let mut prof = Vec::new();
        for k in 0..19 {
            let t = radians(100.0 * k as f64 / 18.0);
            prof.push((POD_R * sin(t), -POD_R * cos(t)));
        }
        let ri = POD_R - 0.035;
        prof.push((
            (POD_R - 0.0175) * sin(radians(102.0)),
            -(POD_R - 0.0175) * cos(radians(102.0)),
        ));
        for k in (0..19).rev() {
            let t = radians(100.0 * k as f64 / 18.0);
            prof.push((ri * sin(t), -ri * cos(t)));
        }
        revolve!(
            "egg",
            &prof,
            at = f.at(cx, cy, cz),
            material = M.shell,
            segments = 36,
            rot = (90, 0, 0),
            scale = (1.0, POD_H / POD_R, 1.0)
        );
        // its stem and swivel base
        kit::turned!(
            f,
            "stem",
            0,
            0.035,
            cz,
            &[
                (0.0, 0.0),
                (0.07, 0.0),
                (0.06, 0.04),
                (0.045, 0.2),
                (0.06, 0.24),
                (0.0, 0.24),
            ],
            M.chrome,
            segments = 20
        );
        kit::turned!(
            f,
            "base_disc",
            0,
            0.0,
            cz,
            &[
                (0.0, 0.0),
                (0.27, 0.0),
                (0.28, 0.01),
                (0.27, 0.03),
                (0.2, 0.04),
                (0.0, 0.04),
            ],
            M.chrome,
            segments = 36
        );
        // the cushions: a round seat (top 0.45), a tufted back cushion in the egg's curve, a little pillow
        kit::lathe!(
            f,
            "seat",
            0,
            0.35,
            0.1,
            &[
                (0.27, 0.0),
                (0.29, 0.015),
                (0.3, 0.05),
                (0.295, 0.08),
                (0.27, 0.096),
                (0.18, 0.1),
                (0.0, 0.1),
            ],
            M.seat,
            verts = 32
        );
        torus!(
            "seat_piping",
            0.293,
            0.006,
            at = f.at(0, 0.43, 0.1),
            material = M.seat,
            major_segments = 32,
            minor_segments = 4
        );
        fsphere!(
            f,
            "back_cushion",
            0.3,
            (0, 0.84, -0.21),
            M.seat,
            scale = (1, 0.3, 1.3),
            segments = 20,
            rings = 12
        );
        for (u, v) in [
            (0.0, 0.18),
            (-0.12, 0.05),
            (0.12, 0.05),
            (0.0, -0.08),
            (-0.12, -0.2),
            (0.12, -0.2),
            (0.0, 0.32),
        ] {
            let left = 1.0 - pow(u / 0.3, 2.0) - pow(v / 0.39, 2.0);
            let depth = 0.09 * sqrt(if left > 0.0 { left } else { 0.0 });
            fsphere!(
                f,
                numbered("button"),
                0.012,
                (u, 0.84 + v, -0.21 + depth - 0.004),
                M.hood,
                scale = (1, 0.5, 1),
                segments = 6,
                rings = 4
            );
        }
        soft!(
            f,
            "pillow",
            0.26,
            0.2,
            0.12,
            (0.17, 0.56, -0.01),
            M.blush,
            rot = (-20, 0, 20),
            round = 0.05,
            puff = 0.035,
            waist = 0.012,
            pinch = 0.006,
            resting = true
        );
        // a shaggy rug in front, its pile jittered
        let rug = fcyl!(
            f,
            "rug",
            0.27,
            0.016,
            (0, 0.008, 0.33),
            M.shag,
            verts = 32,
            scale = (1, 0.55, 1),
            bevel = 0.006,
            bevel_segments = 1
        );
        jitter!(rug, 0.003, size = 0.03, seed = 1.0, up = 0.6);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The hood: a band of a sphere about the egg's upper half, folded back over the egg's top and turning about that
    // sphere's centre, piped along its front edge; `hood` swings it 60° forward, half over the opening. Its back edge
    // stops 6° below level: lower, the hood's box swings its corner into the wall (the gate's `opened`).
    let hood = new_parts(|| {
        _sphere_band(
            "hood",
            f,
            HOOD,
            HOOD_R,
            HOOD_R + 0.02,
            (-62.0, 62.0),
            (-96.0, -20.0),
            M.hood,
            18,
            12,
        );
        let (hx, hy, hz) = HOOD;
        let r = HOOD_R + 0.01;
        let mut edge = Vec::new();
        for i in 0..13 {
            let a = radians(-62.0 + 124.0 * i as f64 / 12.0);
            let p = radians(-20.0);
            edge.push((
                hx + r * sin(a),
                hy + r * cos(a) * cos(p),
                hz + r * cos(a) * sin(p),
            ));
        }
        fsweep!(f, "hood_piping", &edge, 0.014, M.seat, verts = 8);
        for k in 1..4 {
            // the ribs' seams across the hood
            let p = radians((-20 - 22 * k) as f64);
            let seam: Vec<(f64, f64, f64)> = (-60..61)
                .step_by(12)
                .map(|a| {
                    let a = radians(a as f64);
                    (
                        hx + (r + 0.011) * sin(a),
                        hy + (r + 0.011) * cos(a) * cos(p),
                        hz + (r + 0.011) * cos(a) * sin(p),
                    )
                })
                .collect();
            fsweep!(f, numbered("hood_seam"), &seam, 0.004, M.seat, verts = 4);
        }
    });
    let hood = hinged(join(&hood, "hood", None), f.at(HOOD.0, HOOD.1, HOOD.2));
    let objs = settle!(centre = false); // built in place: the seat under the sitter, the hood over their head
    swing!(hood, "hood", 60.0, seconds = 1.0, axis = Axis::X);
    objs
}

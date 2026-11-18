//! The builders' yard's timber stack and cement mixer, and the town's flagpole.

use super::common::{Axle, F, own_palette, tyre};
use artkit::far::{is_low, low};
use artkit::math::{Axis, TAU, cos, degrees, pow, radians, sin};
use artkit::parts::{Settle, join, join_all, new_parts, settle_all, settle_with};
use artkit::py;
use artkit::scene::{Obj, meshes};
use artkit::{
    fbox, fcone, fcyl, fsphere, fsweep, kit_fillet, kit_tube, revolve, rock, soft, sphere, torus,
};

own_palette! { TIMBER:
    body = ("#3C6E9E", fabric),       // the tarpaulin
    timber = ("#C9A56E", wood),       // sawn planks and posts
    bearer = ("#8A6A48", wood),       // the weathered bearers and stickers
    rope = ("#D9C28A", matte),
}

own_palette! { MIXER:
    body = ("#E8742A", gloss),        // the orange drum and motor cover
    crust = ("#A39F96", matte),       // dried cement
    frame = ("#3A3D41", metal),       // the stand
}

own_palette! { FLAG:
    body = ("#F1F0EC", gloss),        // the white pole
    flag = ("#2E5FA8", fabric),       // the flag's field
    flag2 = ("#F2C14E", fabric),      // its gold band
    flag3 = ("#F7F5EE", fabric),      // its white canton
    concrete = ("#A9A59C", matte), rope = ("#E6E2D6", matte),
}

// --- the timber stack: 2x1 outdoors, planks and posts on bearers under a roped tarpaulin, an old tyre on it ---------
pub const TS_L: f64 = 1.82; // the planks' length, across

pub fn timber_stack() -> Vec<Obj> {
    let m = &TIMBER;
    new_parts(|| {
        for (k, x) in [-0.7, 0.0, 0.7].into_iter().enumerate() {
            fbox!(
                F,
                format!("bearer{k}"),
                0.1,
                0.08,
                0.86,
                (x, 0.04, 0),
                m.bearer,
                bevel = low(0.006, 0.0),
                bevel_segments = 1
            );
        }
        let mut y = 0.08;
        // layers of planks, stickers across between them; posts on top
        for lv in 0..4_i32 {
            for k in 0..5_i32 {
                let z = -0.34 + k as f64 * 0.17;
                fbox!(
                    F,
                    format!("plank{lv}{k}"),
                    TS_L - 0.02 * ((lv + k) % 3) as f64,
                    0.032,
                    0.15,
                    (0.01 * ((lv * 3 + k) % 5 - 2) as f64, y + 0.016, z),
                    m.timber,
                    bevel = low(0.003, 0.0),
                    bevel_segments = 1
                );
            }
            y += 0.032;
            for (j, x) in [-0.7, 0.0, 0.7].into_iter().enumerate() {
                fbox!(
                    F,
                    format!("sticker{lv}{j}"),
                    0.04,
                    0.02,
                    0.84,
                    (x, y + 0.01, 0),
                    m.bearer
                );
            }
            y += 0.02;
        }
        for k in 0..4_i32 {
            fbox!(
                F,
                format!("post{k}"),
                TS_L - 0.04 * (k % 2) as f64,
                0.1,
                0.1,
                (
                    0.015 * (k % 3 - 1) as f64,
                    y + 0.05,
                    -0.27 + k as f64 * 0.18
                ),
                m.timber,
                bevel = low(0.005, 0.0),
                bevel_segments = 1
            );
        }
        y += 0.1;
        let top = y;
        // the tarpaulin over the middle of it, hanging down its front and back, the ends of the timber showing
        let drape = 0.26;
        soft!(
            F,
            "tarp",
            1.5,
            drape + 0.03,
            0.9,
            (0, top - drape / 2.0 + 0.03, 0),
            m.body,
            round = 0.04,
            puff = 0.006,
            pinch = 0.0,
            rumple = low(0.02, 0.0),
            rumple_size = 0.3,
            seed = 3.0,
            resting = true
        );
        // two ropes over it, down to the bearers
        for (k, x) in [-0.5, 0.5].into_iter().enumerate() {
            let pts = [
                (x, 0.06, 0.44),
                (x + 0.02, top - drape + 0.02, 0.458),
                (x + 0.02, top + 0.03, 0.43),
                (x, top + 0.035, 0.0),
                (x - 0.02, top + 0.03, -0.43),
                (x - 0.02, top - drape + 0.02, -0.458),
                (x, 0.06, -0.44),
            ];
            kit_tube!(
                F,
                format!("rope{k}"),
                &kit_fillet!(&pts, 0.03, steps = low(2, 1)),
                0.008,
                m.rope,
                verts = low(6, 4),
                round = 1
            );
        }
        // an old tyre on top, keeping the tarp down
        tyre(
            "old_tyre",
            0.32,
            top + 0.045 + 0.08,
            -0.1,
            0.3,
            0.17,
            m.rubber,
            None,
            Some(low(24, 10)),
            Axle::Y,
        );
    });
    join_all(if is_low() {
        "timberStack_far"
    } else {
        "timberStack"
    });
    settle_all();
    meshes()
}

// --- the cement mixer: 1x1 outdoors, a little orange mixer, its drum crusted grey -------------------------------
pub const CM_TILT: f64 = 55.0; // the drum's axis, degrees from upright, its mouth up to the front

pub fn cement_mixer() -> Vec<Obj> {
    let m = &MIXER;
    let (dy, dz) = (0.86, 0.0); // the drum's middle
    let drum = new_parts(|| {
        // the drum: a cone up from its gear end to the belly, a shoulder in to the mouth, the inside dark and crusted
        let n = low(32, 12);
        revolve!(
            "drum",
            &[
                (0.0, -0.32),
                (0.13, -0.32),
                (0.24, -0.24),
                (0.3, -0.1),
                (0.305, -0.02),
                (0.3, 0.04),
                (0.24, 0.16),
                (0.16, 0.26),
                (0.15, 0.3),
                (0.13, 0.3),
                (0.13, 0.26),
                (0.22, 0.12),
                (0.0, 0.06)
            ],
            at = (0, 0, 0),
            material = m.body,
            segments = n
        );
        // the cement dried on its inside: a thin shell just within it
        revolve!(
            "drum_crust",
            &[
                (0.0, 0.064),
                (0.214, 0.122),
                (0.126, 0.258),
                (0.126, 0.296),
                (0.116, 0.296),
                (0.116, 0.255),
                (0.2, 0.13),
                (0.0, 0.075)
            ],
            at = (0, 0, 0),
            material = m.crust,
            segments = n
        );
        torus!(
            "lip_crust",
            0.142,
            0.016,
            at = (0, 0, 0.3),
            material = m.crust,
            major_segments = n,
            minor_segments = low(6, 4)
        );
        torus!(
            "gear",
            0.302,
            0.018,
            at = (0, 0, -0.06),
            material = m.iron,
            major_segments = n,
            minor_segments = low(6, 4)
        );
        if !is_low() {
            for k in 0..8 {
                // crusts and dribbles down the front of the drum from its mouth
                let a = TAU * k as f64 / 8.0 + 0.4 * (k % 3) as f64;
                let t = 0.15 + 0.2 * py::fmod(k as f64 * 0.618, 1.0); // how far down the cone from the mouth
                let (r, h) = (
                    0.16 + (0.3 - 0.16) * t * 1.6,
                    0.26 - (0.26 - 0.04) * t * 1.6,
                );
                sphere!(
                    format!("crust{k}"),
                    0.035 + 0.012 * (k % 3) as f64,
                    at = (r * cos(a), r * sin(a), h),
                    material = m.crust,
                    scale = (1.6, 0.9, 0.22),
                    segments = 8,
                    rings = 5,
                    rot = (0.0, 63.4, degrees(a))
                );
            }
        }
    });
    let drum = join(&drum, "drum", None);
    drum.set_rotation_euler_axis(0, radians(CM_TILT));
    drum.set_location(F.at(0, dy, dz));
    new_parts(|| {
        // the yoke the drum turns in, on a post, the gearbox under the drum, the tilting wheel on its right
        fbox!(
            F,
            "gearbox",
            0.24,
            0.16,
            0.22,
            (0, dy - 0.2, dz - 0.14),
            m.frame,
            bevel = low(0.02, 0.0),
            bevel_segments = 1,
            rot = (CM_TILT, 0, 0)
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fbox!(
                F,
                format!("yoke{s}"),
                0.04,
                0.38,
                0.06,
                (sf * 0.33, dy - 0.17, dz - 0.1),
                m.frame,
                bevel = low(0.006, 0.0),
                bevel_segments = 1
            );
            fcyl!(
                F,
                format!("pivot{s}"),
                0.035,
                0.05,
                (sf * 0.33, dy - 0.08, dz - 0.1),
                m.steel,
                rot = (0, 90, 0),
                verts = low(12, 6)
            );
        }
        fbox!(
            F,
            "yoke_base",
            0.7,
            0.05,
            0.08,
            (0, dy - 0.36, dz - 0.1),
            m.frame,
            bevel = low(0.006, 0.0),
            bevel_segments = 1
        );
        fcyl!(
            F,
            "post",
            0.04,
            0.3,
            (0, 0.38, -0.1),
            m.frame,
            verts = low(12, 6)
        );
        torus!(
            "handwheel",
            0.12,
            0.012,
            at = F.at(-0.38, dy - 0.08, dz - 0.1),
            material = m.frame,
            rot = (0, 90, 0),
            major_segments = low(16, 8),
            minor_segments = 4
        );
        for k in 0..low(3, 0) {
            let a = TAU * k as f64 / 3.0;
            fbox!(
                F,
                format!("spoke{k}"),
                0.012,
                0.24,
                0.012,
                (-0.38, dy - 0.08, dz - 0.1),
                m.frame,
                rot = (degrees(a), 0, 0)
            );
        }
        fcyl!(
            F,
            "wheel_knob",
            0.016,
            0.07,
            (-0.41, dy - 0.08 + 0.12, dz - 0.1),
            m.plastic,
            rot = (0, 90, 0),
            verts = 6
        );
        // the motor under its cover, behind the post, its vents
        fbox!(
            F,
            "motor",
            0.34,
            0.24,
            0.26,
            (0, 0.42, -0.3),
            m.body,
            bevel = low(0.03, 0.0),
            bevel_segments = low(2, 1)
        );
        if !is_low() {
            for k in 0..4 {
                fbox!(
                    F,
                    format!("vent{k}"),
                    0.004,
                    0.012,
                    0.16,
                    (-0.172, 0.36 + k as f64 * 0.03, -0.3),
                    m.dark
                );
            }
            fbox!(
                F,
                "switch",
                0.06,
                0.08,
                0.04,
                (0.12, 0.47, -0.16),
                m.plastic
            );
            fbox!(
                F,
                "switch_red",
                0.03,
                0.02,
                0.012,
                (0.12, 0.49, -0.135),
                m.red_lens
            );
            kit_tube!(
                F,
                "lead",
                &[
                    (0.12, 0.43, -0.16),
                    (0.2, 0.3, -0.25),
                    (0.25, 0.1, -0.35),
                    (0.3, 0.012, -0.4)
                ],
                0.007,
                m.plastic,
                verts = 4,
                round = 2
            );
        }
        // the stand: a tube frame, two wheels at the back, two legs at the front, a handle to wheel it by
        for s in [-1, 1] {
            let sf = s as f64;
            kit_tube!(
                F,
                format!("frame{s}"),
                &[
                    (sf * 0.22, 0.03, 0.4),
                    (sf * 0.22, 0.25, 0.1),
                    (sf * 0.22, 0.25, -0.38)
                ],
                0.018,
                m.frame,
                round = low(2, 1)
            );
            fbox!(
                F,
                format!("foot{s}"),
                0.07,
                0.03,
                0.09,
                (sf * 0.22, 0.015, 0.41),
                m.rubber
            );
            fbox!(
                F,
                format!("axle_bracket{s}"),
                0.03,
                0.12,
                0.05,
                (sf * 0.22, 0.2, -0.36),
                m.frame
            );
            tyre(
                &format!("wheel{s}"),
                sf * 0.29,
                0.15,
                -0.36,
                0.15,
                0.07,
                m.rubber,
                None,
                Some(low(20, 10)),
                Axle::X,
            );
            fcyl!(
                F,
                format!("hub{s}"),
                0.095,
                0.06,
                (sf * 0.29, 0.15, -0.36),
                m.yellow,
                rot = (0, 90, 0),
                verts = low(14, 6)
            );
        }
        fcyl!(
            F,
            "axle",
            0.016,
            0.62,
            (0, 0.15, -0.36),
            m.steel,
            rot = (0, 90, 0),
            verts = 6
        );
        fbox!(F, "cross", 0.44, 0.04, 0.04, (0, 0.25, -0.1), m.frame);
        kit_tube!(
            F,
            "tow",
            &[
                (0.22, 0.25, -0.38),
                (0.15, 0.45, -0.46),
                (-0.15, 0.45, -0.46),
                (-0.22, 0.25, -0.38)
            ],
            0.016,
            m.frame,
            round = low(2, 1)
        );
    });
    join_all(if is_low() {
        "cementMixer_far"
    } else {
        "cementMixer"
    });
    settle_all();
    meshes()
}

// --- the flagpole: 1x1 outdoors, a white pole with the town's flag at the top ---------------------------------
pub const FP_H: f64 = 6.0; // the pole's top, its truck and ball above it
pub const FP_FLAG: (f64, f64) = (1.5, 0.9); // the flag: its fly (length) and hoist (height)

/// The flag's middle line from the pole out to its fly: rippling in the depth, falling a little.
pub fn flag_path(n: usize) -> Vec<(f64, f64, f64)> {
    let l = FP_FLAG.0;
    (0..n + 1)
        .map(|i| {
            let (i, n) = (i as f64, n as f64);
            (
                0.05 + l * i / n,
                FP_H - 0.12 - FP_FLAG.1 / 2.0 - 0.04 * pow(i / n, 2.0),
                0.07 * sin(TAU * 1.3 * i / n) * pow(i / n, 0.7),
            )
        })
        .collect()
}

pub fn flag_pole() -> Vec<Obj> {
    let m = &FLAG;
    let still = new_parts(|| {
        // the base: a concrete pad, a steel flange bolted to it, the pole tapering up to its truck and gilt ball
        fbox!(
            F,
            "pad",
            0.6,
            0.06,
            0.6,
            (0, 0.03, 0),
            m.concrete,
            bevel = low(0.01, 0.0),
            bevel_segments = 1
        );
        revolve!(
            "flange",
            &[
                (0.0, 0.06),
                (0.12, 0.06),
                (0.12, 0.075),
                (0.07, 0.09),
                (0.07, 0.2),
                (0.0, 0.2)
            ],
            at = F.at(0, 0, 0),
            material = m.body,
            segments = low(24, 8)
        );
        if !is_low() {
            for k in 0..4 {
                let a = TAU * (k as f64 + 0.5) / 4.0;
                fcyl!(
                    F,
                    format!("bolt{k}"),
                    0.012,
                    0.024,
                    (0.095 * cos(a), 0.08, 0.095 * sin(a)),
                    m.steel,
                    verts = 6
                );
            }
        }
        fcone!(
            F,
            "pole",
            0.055,
            0.03,
            FP_H - 0.2,
            (0, 0.2 + (FP_H - 0.2) / 2.0, 0),
            m.body,
            verts = low(16, 8)
        );
        fcyl!(
            F,
            "truck",
            0.045,
            0.06,
            (0, FP_H + 0.03, 0),
            m.body,
            verts = low(16, 8)
        );
        fsphere!(
            F,
            "ball",
            0.06,
            (0, FP_H + 0.11, 0),
            m.brass,
            segments = low(16, 8),
            rings = low(10, 5)
        );
        // the halyard: down from the truck's pulley to a cleat at hand height, its two strands
        fbox!(
            F,
            "cleat",
            0.025,
            0.14,
            0.03,
            (0.0, 1.3, -0.065),
            m.steel,
            bevel = low(0.004, 0.0),
            bevel_segments = 1
        );
        for (k, x) in [-0.012, 0.012].into_iter().enumerate() {
            fcyl!(
                F,
                format!("halyard{k}"),
                0.0035,
                FP_H - 1.3,
                (x, 1.3 + (FP_H - 1.3) / 2.0, -0.06),
                m.rope,
                verts = 4
            );
        }
        for (j, y) in [FP_H - 0.12, FP_H - 0.12 - FP_FLAG.1]
            .into_iter()
            .enumerate()
        {
            // the flag's clips on the halyard
            fbox!(
                F,
                format!("clip{j}"),
                0.04,
                0.03,
                0.02,
                (0.03, y, -0.0),
                m.steel
            );
        }
    });
    join(
        &still,
        if is_low() { "flagPole_far" } else { "flagPole" },
        Some((0, 0, 0).into()),
    );
    let flag = new_parts(|| {
        let n = low(24, 8);
        let path = flag_path(n);
        let (hh, t) = (FP_FLAG.1 / 2.0, 0.006);
        fsweep!(
            F,
            "flag",
            &path,
            0,
            m.flag,
            profile = [
                (-hh, -t / 2.0),
                (hh, -t / 2.0),
                (hh, t / 2.0),
                (-hh, t / 2.0)
            ],
            up = (0, 0, 1)
        );
        fsweep!(
            F,
            "flag_band",
            &path,
            0,
            m.flag2,
            profile = [
                (-0.09, -t / 2.0 - 0.002),
                (0.09, -t / 2.0 - 0.002),
                (0.09, t / 2.0 + 0.002),
                (-0.09, t / 2.0 + 0.002)
            ],
            up = (0, 0, 1)
        );
        let k = n * 4 / 10;
        fsweep!(
            F,
            "flag_canton",
            &path[..k + 1],
            0,
            m.flag3,
            profile = [
                (0.13, -t / 2.0 - 0.003),
                (hh - 0.05, -t / 2.0 - 0.003),
                (hh - 0.05, t / 2.0 + 0.003),
                (0.13, t / 2.0 + 0.003)
            ],
            up = (0, 0, 1)
        );
    });
    let on_its_pad = Settle {
        centre: false,
        floor: true,
    };
    if is_low() {
        join_all("flagPole_far");
        settle_with(&meshes(), on_its_pad);
        return meshes();
    }
    let flag_part = join(&flag, "flag", Some(F.at(0, FP_H - 0.6, 0)));
    settle_with(&meshes(), on_its_pad);
    rock!(flag_part, "flutter", 7, seconds = 3.4, axis = Axis::Z);
    meshes()
}

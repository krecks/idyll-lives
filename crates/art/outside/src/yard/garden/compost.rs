//! The compost bin.

use super::*;

// --- the compost bin (1x1): a slatted box on corner posts, the heap inside; the lid lifts ---------------------
const CB: (f64, f64, f64) = (0.8, 0.76, -0.08); // width, depth, middle (depth)
const CB_TOP: f64 = 0.72;

pub fn compost_bin() -> Vec<Obj> {
    let (w, d, cz) = CB;
    let (z0, z1) = (cz - d / 2.0, cz + d / 2.0);
    for x in [-w / 2.0 + 0.03, w / 2.0 - 0.03] {
        for z in [z0 + 0.03, z1 - 0.03] {
            crate::boxed(fbox!(
                F,
                numbered("post"),
                0.06,
                CB_TOP - 0.012,
                0.06,
                (x, (CB_TOP - 0.012) / 2.0, z),
                M.bin,
                bevel = low(0.008, 0.0),
                bevel_segments = 1
            ));
            if !is_low() {
                // each post's top cut to a shallow point
                cone!(
                    numbered("post_cap"),
                    0.042,
                    0.0,
                    0.012,
                    at = F.at(x, CB_TOP - 0.006, z),
                    material = M.bin,
                    verts = 4,
                    rot = (0, 0, 45)
                );
            }
        }
    }
    for (k, y) in [0.08, 0.25, 0.42, 0.59].into_iter().enumerate() {
        fbox!(
            F,
            numbered("slat_back"),
            w - 0.04,
            0.12,
            0.025,
            (0, y + 0.02, z0 + 0.01),
            M.bin,
            bevel = low(0.006, 0.0),
            bevel_segments = 1
        );
        for s in [-1.0, 1.0] {
            fbox!(
                F,
                numbered("slat_side"),
                0.025,
                0.12,
                d - 0.12,
                (s * (w / 2.0 - 0.01), y + 0.02, cz),
                M.bin,
                bevel = low(0.006, 0.0),
                bevel_segments = 1
            );
        }
        if k < 3 {
            // the front boards: the top one lifted out
            fbox!(
                F,
                numbered("slat_front"),
                w - 0.12,
                0.12,
                0.025,
                (0, y + 0.02, z1 - 0.01),
                M.bin,
                bevel = low(0.006, 0.0),
                bevel_segments = 1
            );
        }
    }
    if !is_low() {
        // galvanised nails in the slats' ends
        for y in [0.08, 0.25, 0.42, 0.59] {
            for s in [-1.0, 1.0] {
                for z in [z0 + 0.03, z1 - 0.03] {
                    fcyl!(
                        F,
                        numbered("nail"),
                        0.005,
                        0.004,
                        (s * (w / 2.0 + 0.001), y + 0.02, z),
                        M.can,
                        rot = (0, 90, 0),
                        verts = 5
                    );
                }
            }
        }
    }
    for s in [-1.0, 1.0] {
        // the front boards' guide battens
        fbox!(
            F,
            numbered("guide"),
            0.03,
            CB_TOP - 0.06,
            0.03,
            (s * (w / 2.0 - 0.075), (CB_TOP - 0.06) / 2.0, z1 - 0.035),
            M.bin,
            bevel = low(0.004, 0.0),
            bevel_segments = 1
        );
    }
    // the heap: a mound of compost, peelings and greens on top, a few lumps
    blob(0.0, 0.02, cz, 0.36, M.compost, 2.0, (1.0, 1.0, 0.82), 0.25);
    for (k, (x, z, m)) in [
        (0.12, cz + 0.06, M.greens),
        (-0.15, cz - 0.08, M.peel),
        (0.02, cz - 0.14, M.greens),
        (-0.05, cz + 0.12, M.peel),
        (0.18, cz - 0.1, M.compost),
        (-0.2, cz + 0.08, M.greens),
    ]
    .into_iter()
    .enumerate()
    {
        blob(
            x,
            0.56 - 0.02 * (k / 4) as f64,
            z,
            0.05,
            m,
            (9 + k) as f64,
            (1.4, 1.0, 0.4),
            0.5,
        );
    }
    if !is_low() {
        for (k, (x, z)) in [(0.06, cz - 0.02), (-0.1, cz + 0.02), (0.12, cz + 0.14)]
            .into_iter()
            .enumerate()
        {
            fcone!(
                F,
                numbered("shell"),
                0.025,
                0.018,
                0.014,
                (x, 0.585, z),
                M.shell,
                verts = 6,
                rot = ((10 * k) as i32, 0, 0)
            );
        }
    }
    if !is_low() {
        join(&meshes(), "bin", Some(P3::from((0, 0, 0))));
    }
    // The lid: four boards on two battens, a wooden grip, hinged along the back.
    let lid = new_parts(|| {
        for k in 0..4 {
            let bw = (d + 0.04) / 4.0;
            fbox!(
                F,
                numbered("lid_board"),
                w + 0.04,
                0.025,
                bw - 0.008,
                (0, CB_TOP + 0.0125, z0 - 0.02 + bw * (k as f64 + 0.5)),
                M.bin,
                bevel = low(0.006, 0.0),
                bevel_segments = 1
            );
        }
        for s in [-1.0, 1.0] {
            fbox!(
                F,
                numbered("batten"),
                0.05,
                0.03,
                d - 0.04,
                (s * (w / 2.0 - 0.1), CB_TOP - 0.012, cz),
                M.bin,
                bevel = low(0.004, 0.0),
                bevel_segments = 1
            );
        }
        fextrude!(
            F,
            "grip",
            &[
                (-0.1, 0.0),
                (0.1, 0.0),
                (0.085, 0.03),
                (0.06, 0.035),
                (-0.06, 0.035),
                (-0.085, 0.03)
            ],
            0.04,
            (0, CB_TOP + 0.025, z1 - 0.03),
            M.bin,
            bevel = low(0.006, 0.0),
            bevel_segments = 1
        );
    });
    if is_low() {
        return settle!(&[join_all("compostBin")], centre = false);
    }
    let lid = hinged(join(&lid, "lid", None), F.at(0, CB_TOP, z0 - 0.02));
    let objs = settle!(centre = false);
    swing(
        lid,
        "lid_open",
        -75.0,
        Swing {
            seconds: 0.8,
            axis: Axis::X,
            ..Swing::default()
        },
    );
    objs
}

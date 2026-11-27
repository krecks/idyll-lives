use super::super::common::{F, M, end_round, mesh_of, plank, side_slab};
use artkit::json::float_repr;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, V3, cosd, pow};
use artkit::motion::{Rock, rock};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::N;
use artkit::scene::{Obj, meshes};
use artkit::{fcyl, fsweep, revolve, settle, soft, torus};

// --- the hammock: a striped sling between spreader bars on a wooden stand; it sways while lain in -------------
pub(super) const HOOK_Y: f64 = 1.1; // the hooks, height: the sling sways about the line through them

/// A sagging sling `width` wide from depth -half to +half, `low` in the middle and `high` at the ends, `t` thick,
/// in bands of alternating materials; its sides curl up by `curl` where it sags (flat at the spreader bars), so
/// the middle line keeps its heights.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub(super) fn sling(
    name: &str,
    width: f64,
    half: f64,
    low: f64,
    high: f64,
    t: f64,
    materials: &[Mat],
    bands: usize,
    cols: usize,
    curl: f64,
) -> Obj {
    let n = bands * 3;
    let xs: Vec<f64> = (0..=cols)
        .map(|i| -width / 2.0 + width * i as f64 / cols as f64)
        .collect();
    let (mut verts, mut faces, mut mats) = (Vec::new(), Vec::new(), Vec::new());
    let y_at = |x: f64, z: f64| {
        low + (high - low) * pow(z / half, 2.0)
            + curl * pow(2.0 * x / width, 2.0) * (1.0 - pow(z / half, 2.0))
    };
    for i in 0..=n {
        let z = -half + 2.0 * half * i as f64 / n as f64;
        for &x in &xs {
            let y = y_at(x, z);
            verts.push(V3::from(F.at(x, y, z)));
            verts.push(V3::from(F.at(x, y - t, z)));
        }
    }
    let row = 2 * (cols + 1) as u32;
    for i in 0..n as u32 {
        let m = ((i / 3) as usize % materials.len()) as u16;
        let (a, b) = (i * row, (i + 1) * row);
        for c in 0..cols as u32 {
            let (p, q) = (2 * c, 2 * (c + 1));
            for f in [
                vec![a + p, b + p, b + q, a + q],
                vec![a + p + 1, a + q + 1, b + q + 1, b + p + 1],
            ] {
                faces.push(f);
                mats.push(m);
            }
        }
        for p in [0, 2 * cols as u32] {
            // the two side edges
            faces.push(vec![a + p, a + p + 1, b + p + 1, b + p]);
            mats.push(m);
        }
    }
    for a in [0, n as u32 * row] {
        // the two ends
        for c in 0..cols as u32 {
            let (p, q) = (2 * c, 2 * (c + 1));
            faces.push(vec![a + p, a + q, a + q + 1, a + p + 1]);
            mats.push(0);
        }
    }
    mesh_of(name, verts, &faces, materials, &mats, true)
}

// A stand hammock on its 1 x 3 tiles: a 2.8 m stand, a 1.96 m bed between the spreader bars, long enough to lie in
// flat (`beds.hammock` `sling`: `low`, `end`, `half`, and the pillow at `head`).
pub(super) const SLING_LOW: f64 = 0.52;
pub(super) const SLING_END: f64 = 0.84;
pub(super) const SLING_HALF: f64 = 0.98;
pub(super) const SLING_HEAD: f64 = 0.78;
pub(super) const HAMMOCK_HOOK: f64 = 1.245; // the hooks on the posts' inner faces, either side of the middle

pub fn hammock() -> Vec<Obj> {
    let stand = new_parts(|| {
        side_slab(
            "base",
            0,
            &[(-1.4, 0.0), (1.4, 0.0), (1.37, 0.06), (-1.37, 0.06)],
            0.1,
            M.hammock_frame,
            0.006,
        );
        for s in [-1, 1] {
            let sf = s as f64;
            // the feet across, rounded at their ends; the posts leaning in, cut level at the foot; braces
            plank(
                &format!("foot{s}"),
                (-0.5, 0.02, sf * 1.36),
                (0.5, 0.02, sf * 1.36),
                0.04,
                M.hammock_frame,
                &end_round(0.08, 1.0, Some(0.03), 2, true),
                (0, 1, 0),
                0.006,
            );
            let hw = 0.04 / cosd(2.9);
            side_slab(
                &format!("post{s}"),
                0,
                &[
                    (sf * 1.36 - hw, 0.0),
                    (sf * 1.36 + hw, 0.0),
                    (sf * 1.3 + hw, 1.2),
                    (sf * 1.3 - hw, 1.2),
                ],
                0.08,
                M.hammock_frame,
                0.006,
            );
            kit::board(
                &F,
                &format!("brace{s}"),
                (0, 0.06, sf * 1.0),
                (0, 0.5, sf * 1.34),
                0.05,
                0.05,
                M.hammock_frame,
                (1, 0, 0),
                0.008,
            );
            fcyl!(
                F,
                format!("cap{s}"),
                0.045,
                0.02,
                (0, 1.19, sf * 1.3),
                M.hammock_frame,
                verts = 10,
                bevel = 0.006,
                bevel_segments = 1
            );
            // the hook: an eye screwed into the post and a ring hanging from it
            torus!(
                format!("hook{s}"),
                0.018,
                0.005,
                at = F.at(0, HOOK_Y, sf * HAMMOCK_HOOK),
                material = M.hammock_frame,
                rot = (0, 90, 0),
                major_segments = 10,
                minor_segments = 4
            );
        }
    });
    join(&stand, "stand", Some(P3::from((0, 0, 0))));
    let bed = new_parts(|| {
        sling(
            "sling",
            0.74,
            SLING_HALF,
            SLING_LOW,
            SLING_END,
            0.02,
            &[M.sling, M.sling_stripe],
            10,
            4,
            0.05,
        );
        let bar = SLING_HALF + 0.01;
        for s in [-1, 1] {
            let sf = s as f64;
            revolve!(
                format!("spreader{s}"),
                &[
                    (0.0, -0.4),
                    (0.012, -0.4),
                    (0.016, -0.39),
                    (0.016, 0.39),
                    (0.012, 0.4),
                    (0.0, 0.4)
                ],
                at = F.at(0, SLING_END + 0.005, sf * bar),
                material = M.hammock_frame,
                segments = 8,
                rot = (0, 90, 0)
            );
            // the cords fanning from the bar to a ring under the hook, and the rope up to it
            let knot = (N::I(0), HOOK_Y - 0.09, sf * (HAMMOCK_HOOK - 0.06));
            for x in [-0.36, -0.18, 0.0, 0.18, 0.36] {
                fsweep!(
                    F,
                    format!("cord{s}{}", float_repr(x)),
                    &[(N::F(x), SLING_END + 0.01, sf * bar), knot],
                    0.0035,
                    M.sling_stripe,
                    verts = 4
                );
            }
            torus!(
                format!("ring{s}"),
                0.02,
                0.006,
                at = F.at(knot.0, knot.1, knot.2),
                material = M.sling_stripe,
                rot = (90, 0, 0),
                major_segments = 10,
                minor_segments = 4
            );
            fsweep!(
                F,
                format!("rope{s}"),
                &[knot, (N::I(0), HOOK_Y, sf * HAMMOCK_HOOK)],
                0.006,
                M.sling_stripe,
                verts = 5
            );
            // the sling's edges whipped round the bar
            let hem: Vec<(f64, f64, f64)> = [-0.37, 0.37]
                .iter()
                .map(|&x| (x, SLING_END - 0.002, sf * (bar - 0.012)))
                .collect();
            fsweep!(F, format!("hem{s}"), &hem, 0.012, M.sling, verts = 6);
        }
        let head = SLING_LOW + (SLING_END - SLING_LOW) * pow(SLING_HEAD / SLING_HALF, 2.0);
        soft!(
            F,
            "pillow",
            0.4,
            0.08,
            0.26,
            (0, head + 0.03, -SLING_HEAD),
            M.pillow,
            rot = (12, 0, 0),
            round = 0.035,
            puff = 0.02,
            waist = 0.012,
            pinch = 0.006,
            resting = true
        );
    });
    let bed = hinged(join(&bed, "sling", None), F.at(0, HOOK_Y, 0));
    settle!(centre = false);
    rock(
        bed,
        "sway",
        8.0,
        Rock {
            seconds: 2.0,
            axis: Axis::Y,
        },
    );
    meshes()
}

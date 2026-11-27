use artkit::botany::{fit_to, palette, polar};
use artkit::far::low;
use artkit::math::{M4, V3, cos, radians, sin, v3};
use artkit::parts::{join_all, settle};
use artkit::scene::{Obj, meshes};
use artkit::{arc, cone, cut_leaf, outline, twig, vessel};

/// Potted plant: a terracotta pot with a rolled rim on its saucer, and a lush arrowhead plant, three rings of
/// heart-shaped leaves on arching stalks round a few young ones still rolled up in the middle.
pub fn plant() -> Vec<Obj> {
    let p = super::super::plain_leaves(palette(&[
        ("body", "#C46A43"),
        // (plain, so as dark as the foliage texture drew them)
        ("leaf", "#37632B"),
        ("leaf_dark", "#2E5626"),
        ("leaf_light", "#4C7A3A"),
        ("stem", "#5E8044"),
    ]));
    let ys = vessel!(
        &p,
        0.19,
        0.32,
        r_bottom = 0.14,
        lip = 0.016,
        foot = 0.0,
        saucer = 0.035,
        segments = 18
    );
    let cols = [p.get("leaf"), p.get("leaf_dark"), p.get("leaf_light")];
    let mut k = 0;
    // count, heading offset, stalk length, stalk tilt, leaf length, width, tilt, droop
    for (n, off, sl, st, ll, lw, lt, dr) in [
        (8, 0, 0.17, 30.0, 0.22, 0.17, 0.0, 0.32),
        (7, 22, 0.28, 56.0, 0.23, 0.17, 22.0, 0.24),
        (5, 10, 0.37, 74.0, 0.22, 0.16, 44.0, 0.12),
    ] {
        for i in 0..n {
            let yaw = off as f64 + (i * 360) as f64 / n as f64 + ((k * 37) % 17) as f64;
            let a = V3::from(polar(0.03, yaw, ys));
            let tip = a + V3::from(polar(sl * cos(radians(st)), yaw, sl * sin(radians(st))));
            twig!(
                format!("stalk{k}"),
                &arc!(a, tip, 0.03, steps = 2),
                0.006,
                0.004,
                p.get("stem"),
                verts = 3,
                res = 1
            );
            cut_leaf!(
                format!("leaf{k}"),
                tip,
                yaw,
                lt,
                ll,
                &outline!("heart", lw, n = 6),
                cols[k % 3],
                droop = dr,
                fold = 0.22,
                ribs = 4
            );
            k += 1;
        }
    }
    // young leaves, still rolled, in the middle
    for i in 0..low(3, 0) {
        let yaw = (i * 120 + 40) as f64;
        let young = cone!(
            format!("young{i}"),
            0.012,
            0.002,
            0.14,
            at = (0, 0, 0),
            material = p.get("leaf_light"),
            verts = 5
        );
        let d = V3::from(polar(sin(0.35), yaw, cos(0.35)));
        young.edit(|o| {
            o.mesh.transform(
                &(M4::translation(v3(0, 0, ys + 0.01) + d * 0.08)
                    * d.to_track_quat_zy().to_matrix().to_4x4()),
            )
        });
    }
    fit_to(&meshes(), 0.772, 0.825, 0.791);
    settle(&[join_all("plant")])
}

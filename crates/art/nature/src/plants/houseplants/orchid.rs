use artkit::botany::{_flower, fit_to, palette, polar};
use artkit::far::low;
use artkit::math::{V3, v3};
use artkit::parts::{join_all, settle};
use artkit::scene::{Obj, meshes};
use artkit::{cut_leaf, cylinder, outline, revolve, soil_top, sphere, twig, vessel};

/// Orchid: a small white pot of bark on a turned wooden pedestal, three broad, fleshy leaves, silvery aerial
/// roots, and a staked, arching spike of moth-orchid flowers and buds.
pub fn orchid() -> Vec<Obj> {
    let p = super::super::plain_leaves(palette(&[
        ("body", "#F2F0EC"),
        ("leaf", "#2E5A30"),
        ("stem", "#4E6A3A"),
        ("wood", "#5A4030"),
        ("bloom", "#F6F0F4"),
        ("bloom_accent", "#C2367A"),
        ("soil", "#6A4A30"),
        ("flesh", "#AAB8A0"),
    ]));
    revolve!(
        "pedestal",
        &[
            (0.0, 0.0),
            (0.15, 0.0),
            (0.15, 0.03),
            (0.13, 0.05),
            (0.105, 0.08),
            (0.095, 0.32),
            (0.1, 0.52),
            (0.12, 0.56),
            (0.13, 0.585),
            (0.16, 0.6),
            // (its top's arris eased: two turns of 45°)
            (0.16, 0.626),
            (0.156, 0.63),
            (0.0, 0.63)
        ],
        material = p.get("wood"),
        segments = low(16, 8)
    );
    let ys = vessel!(
        &p,
        0.085,
        0.16,
        z0 = 0.63,
        r_bottom = 0.07,
        lip = 0.008,
        segments = 16,
        soil = false
    );
    soil_top!(
        &p,
        0.077,
        ys,
        segments = low(12, 6),
        mound = 0.012,
        seed = 4.0
    );
    for (i, yaw) in [10.0, 150.0, 260.0].into_iter().enumerate() {
        cut_leaf!(
            format!("leaf{i}"),
            (0, 0, ys + 0.01),
            yaw,
            18.0,
            0.22,
            &outline!("oval", 0.095, n = 6),
            p.get("leaf"),
            droop = 0.35,
            fold = 0.22,
            ribs = 5,
            thick = 0.012
        );
    }
    // aerial roots, over the rim
    for i in 0..low(3, 0) {
        let a = (60 + i * 110) as f64;
        let top = V3::from(polar(0.03, a, ys + 0.015));
        twig!(
            format!("root{i}"),
            &[
                top,
                top + V3::from(polar(0.06, a, 0.015)),
                top + V3::from(polar(0.1, a, -0.03))
            ],
            0.005,
            0.004,
            p.get("flesh"),
            verts = 3
        );
    }
    cylinder!(
        "stake",
        0.003,
        0.39,
        at = (-0.03, 0.01, ys + 0.195),
        material = p.get("wood"),
        verts = low(5, 3)
    );
    let spike: Vec<V3> = [
        (0.0, 0.0, ys),
        (-0.03, 0.01, ys + 0.18),
        // (a moth orchid in flower stands 0.46-0.61 m)
        (-0.06, 0.0, ys + 0.37),
        (-0.16, -0.01, ys + 0.46),
        (-0.27, -0.02, ys + 0.43),
    ]
    .into_iter()
    .map(V3::from)
    .collect();
    twig!(
        "spike",
        &spike,
        0.005,
        0.003,
        p.get("stem"),
        verts = 4,
        res = 2
    );
    for i in 0..8 {
        let t = 0.42 + i as f64 * 0.075;
        let k = (spike.len() - 2).min((t * (spike.len() - 1) as f64) as usize);
        let f = t * (spike.len() - 1) as f64 - k as f64;
        let at = spike[k].lerp(spike[k + 1], f) + v3(0, -0.03, -0.03);
        if i < 6 {
            _flower(
                &format!("flower{i}"),
                at,
                (180 + 12 * (i % 3 - 1)) as f64,
                if i < 5 { 0.05 } else { 0.04 },
                &p,
            );
        } else {
            // buds at the end
            sphere!(
                format!("bud{i}"),
                0.012,
                at = at + v3(0, 0.02, 0.025),
                material = p.get("bloom"),
                segments = low(6, 4),
                rings = low(4, 3),
                scale = (1, 1, 1.3)
            );
        }
    }
    fit_to(&meshes(), 0.431, 1.228, 0.381);
    settle(&[join_all("orchid")])
}

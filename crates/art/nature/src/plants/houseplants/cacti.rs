use artkit::botany::{fit_to, palette, polar};
use artkit::far::{is_low, low};
use artkit::forms::by;
use artkit::kit::fillet;
use artkit::math::{P3, V3, v3};
use artkit::parts::{join_all, new_parts, settle};
use artkit::scene::{Obj, meshes};
use artkit::{loft, rays, ribbed, soil_top, sphere, sweep, vessel};

/// Cactus trio: three little terracotta pots with rolled rims, topped with grit: a ribbed column cactus with two
/// arms and a pink flower, a ribbed barrel with a crown of yellow flowers, and a prickly pear's pads.
pub fn cacti() -> Vec<Obj> {
    let p = palette(&[
        ("body", "#C0683F"),
        ("flesh", "#4E8A4A"),
        ("flesh_light", "#6A9E5A"),
        ("bloom", "#E8508A"),
        ("bloom_accent", "#F2C14E"),
        ("sand", "#B8A88A"),
    ]);
    // game (x, depth) -> Blender (-x, -depth)
    let pots = [
        ((0.17, -0.05), 0.105, 0.18),
        ((-0.14, 0.08), 0.115, 0.2),
        ((-0.12, -0.17), 0.08, 0.13),
    ];
    let mut tops = Vec::new();
    for (i, ((x, y), r, h)) in pots.into_iter().enumerate() {
        let mut top = 0.0;
        let parts = new_parts(|| {
            top = vessel!(
                &p,
                r,
                h,
                r_bottom = r * 0.75,
                lip = 0.01,
                segments = 10,
                soil = false
            );
            soil_top!(
                &p,
                r - 0.01,
                top,
                segments = low(8, 6),
                mound = 0.006,
                seed = i as f64,
                material = p.get("sand")
            );
        });
        for o in parts {
            o.edit(|part| {
                part.loc.x = (part.loc.x as f64 + x) as f32;
                part.loc.y = (part.loc.y as f64 + y) as f32;
            });
        }
        tops.push(((x, y), top));
    }
    // the column, two arms and a pink flower
    let ((x, y), z) = tops[0];
    let n = low(6, 4);
    let secs: Vec<Vec<P3>> = [
        (0.0, 0.042),
        (0.12, 0.045),
        (0.26, 0.044),
        (0.32, 0.038),
        (0.35, 0.024),
        (0.36, 0.008),
    ]
    .iter()
    .map(|&(hz, rr)| {
        ribbed!(rr, n)
            .iter()
            .map(|&(u, v)| P3::from((x + u, y + v, z - 0.01 + hz)))
            .collect()
    })
    .collect();
    loft!("column", &secs, material = p.get("flesh"));
    for (name, pts, r) in [
        (
            "armL",
            [
                (x + 0.03, y, z + 0.12),
                (x + 0.1, y, z + 0.13),
                (x + 0.11, y, z + 0.2),
                (x + 0.11, y, z + 0.25),
            ],
            0.024,
        ),
        (
            "armR",
            [
                (x - 0.03, y, z + 0.18),
                (x - 0.09, y, z + 0.19),
                (x - 0.095, y, z + 0.25),
                (x - 0.095, y, z + 0.28),
            ],
            0.02,
        ),
    ] {
        let path = if !is_low() {
            fillet(&pts, 0.03, 2)
        } else {
            pts.to_vec()
        };
        sweep!(
            name,
            &path,
            0.02,
            profile = ribbed!(r, n),
            material = p.get("flesh"),
            scale = by(|w| 1.0 - 0.35 * 0.0_f64.max((w - 0.75) / 0.25))
        );
    }
    rays!(
        "flower",
        (x, y, z + 0.345),
        0.026,
        8,
        p.get("bloom"),
        inner = 0.3,
        cup = 0.6,
        width = 0.9,
        centre = 0.004
    );
    // the barrel, ribbed, with a crown of yellow flowers
    let ((x, y), z) = tops[1];
    let m = low(10, 6);
    let secs: Vec<Vec<P3>> = [
        (0.0, 0.06),
        (0.03, 0.09),
        (0.08, 0.1),
        (0.13, 0.085),
        (0.16, 0.05),
        (0.17, 0.02),
    ]
    .iter()
    .map(|&(hz, rr)| {
        // (six points a rib: rounded ribs, no turn of 60° at their ridges)
        ribbed!(rr, m, depth = 0.14, verts = m * 6)
            .iter()
            .map(|&(u, v)| P3::from((x + u, y + v, z + hz)))
            .collect()
    })
    .collect();
    loft!("barrel", &secs, material = p.get("flesh_light"));
    for i in 0..low(3, 1) {
        let a = (i * 120) as f64;
        rays!(
            format!("crown{i}"),
            v3(x, y, z + 0.165) + V3::from(polar(0.025, a, 0.0)),
            0.018,
            6,
            p.get("bloom_accent"),
            yaw = a,
            tilt = 25,
            inner = 0.3,
            cup = 0.5
        );
    }
    // the prickly pear's pads
    let ((x, y), z) = tops[2];
    for (i, (dx, dz, w, h, r)) in [
        (0.0, 0.06, 0.11, 0.14, 8),
        (0.05, 0.16, 0.08, 0.1, -28),
        (-0.045, 0.14, 0.07, 0.09, 30),
    ]
    .into_iter()
    .enumerate()
    {
        sphere!(
            format!("pad{i}"),
            0.5,
            at = (x + dx, y, z + dz),
            material = p.get("flesh_light"),
            segments = low(8, 6),
            rings = low(5, 4),
            scale = (w, 0.032, h),
            rot = (0, r, 0)
        );
    }
    fit_to(&meshes(), 0.568, 0.521, 0.465);
    settle(&[join_all("cacti")])
}

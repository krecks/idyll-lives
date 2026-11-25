//! OU1's tables and the rest: the round mosaic table, the teak table, the patio chair, the outdoor bar and the
//! parasol, whose canopy stirs in the wind.

use super::common::{F, M, bent, bottle, cup, finish, rod, side_slab, slab, tapered_leg};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{Axis, P3, PI, TAU, V3, cos, cosd, pow, sin, sind};
use artkit::mesh::Mesh;
use artkit::motion::{Rock, rock};
use artkit::parts::{hinged, join, new_parts};
use artkit::scene::{self, Obj, meshes};
use artkit::{
    cylinder, fbox, fcyl, fextrude, fsweep, loft, revolve, rod as rod_, settle, soft, sphere, sweep,
};

// --- the round patio table: a mosaic top on an iron pedestal ---------------------------------------------------

/// A round mosaic at height `y`: rings of tiles between `radii` (the first a disc in the middle), `counts` tiles
/// round each ring, each tile set `raised` proud with a sloping edge `grout` wide, so a V of grout shows between
/// them (its shading baked dark); `pattern(ring, k)` gives each tile's material. Open underneath: it lies on a top.
fn mosaic(
    name: &str,
    y: f64,
    radii: &[f64],
    counts: &[usize],
    pattern: impl Fn(usize, usize) -> Mat,
    grout: f64,
    raised: f64,
) -> Obj {
    let mut mesh = Mesh::default();
    for (ring, ((&r0, &r1), &n)) in radii.iter().zip(&radii[1..]).zip(counts).enumerate() {
        let off = PI / n as f64 * (ring % 2) as f64;
        for k in 0..n {
            let (a0, a1) = (
                TAU * k as f64 / n as f64 + off,
                TAU * (k + 1) as f64 / n as f64 + off,
            );
            let base = mesh.verts.len() as u32;
            let vs: Vec<V3> = if r0 == 0.0 {
                let mut vs: Vec<V3> = (0..3)
                    .map(|i| {
                        let a = a0 + (a1 - a0) * i as f64 / 2.0;
                        V3::from(F.at(r1 * cos(a), y, r1 * sin(a)))
                    })
                    .collect();
                vs.push(V3::from(F.at(0, y, 0)));
                vs
            } else {
                [(r0, a0), (r1, a0), (r1, a1), (r0, a1)]
                    .iter()
                    .map(|&(r, a)| V3::from(F.at(r * cos(a), y, r * sin(a))))
                    .collect()
            };
            let corners: Vec<u32> = (0..vs.len() as u32).map(|i| base + i).collect();
            mesh.verts.extend(vs);
            let m = mesh.slot(pattern(ring, k));
            mesh.push_face(&corners, m, false);
        }
    }
    mesh.remove_doubles(1e-6);
    for f in 0..mesh.faces.len() {
        if mesh.face_normal(f).z < 0.0 {
            let face = mesh.faces[f];
            mesh.loops[face.start as usize + 1..(face.start + face.len) as usize].reverse();
        }
    }
    let obj = scene::add(name, mesh);
    artkit::edit::inset_individual(
        obj,
        artkit::edit::InsetIndividual {
            thickness: grout,
            depth: raised,
            use_even_offset: true,
            ..Default::default()
        },
    )
}

pub fn patio_table_round() -> Vec<Obj> {
    let pattern = |ring: usize, k: usize| -> Mat {
        if ring == 0 {
            return M.tile;
        }
        if ring == 1 {
            return if k % 2 == 1 { M.tile_light } else { M.tile };
        }
        if ring == 3 {
            return if k.is_multiple_of(3) {
                M.tile
            } else {
                M.mosaic
            };
        }
        if ring == 5 {
            return if k % 2 == 1 { M.tile_light } else { M.mosaic };
        }
        M.mosaic
    };
    // the top: a slab under the mosaic, the tiles, an iron band round its edge
    artkit::bevel::eased(revolve!(
        "top",
        &[
            (0.0, 0.676),
            (0.37, 0.676),
            (0.392, 0.69),
            (0.392, 0.719),
            (0.0, 0.719)
        ],
        at = F.at(0, 0, 0),
        material = M.mosaic,
        segments = 24,
        smooth = false
    ));
    mosaic(
        "tiles",
        0.72,
        &[0.0, 0.07, 0.13, 0.19, 0.25, 0.31, 0.375],
        &[8, 12, 16, 20, 24, 28],
        pattern,
        0.003,
        0.003,
    );
    let band: Vec<(f64, f64, f64)> = (0..28)
        .map(|i| {
            (
                0.402 * cos(TAU * i as f64 / 28.0),
                0.702,
                0.402 * sin(TAU * i as f64 / 28.0),
            )
        })
        .collect();
    fsweep!(
        F,
        "rim",
        &band,
        0,
        M.iron,
        profile = rounded_rect(0.046, 0.012, 0.005, 1),
        up = (0, 0, 1),
        closed = true
    );
    // the pedestal: a hub under the top, a turned column with collars, a domed foot on a plinth
    fcyl!(
        F,
        "hub",
        0.1,
        0.03,
        (0, 0.661, 0),
        M.iron,
        verts = 16,
        bevel = 0.008,
        bevel_segments = 1
    );
    revolve!(
        "column",
        &[
            (0.0, 0.07),
            (0.05, 0.07),
            (0.05, 0.085),
            (0.034, 0.1),
            (0.03, 0.16),
            (0.042, 0.172),
            (0.042, 0.184),
            (0.028, 0.196),
            (0.024, 0.6),
            (0.036, 0.612),
            (0.036, 0.63),
            (0.05, 0.646),
            (0.0, 0.646)
        ],
        at = F.at(0, 0, 0),
        material = M.iron,
        segments = 12
    );
    revolve!(
        "foot",
        &[
            (0.0, 0.0),
            (0.25, 0.0),
            (0.25, 0.012),
            (0.235, 0.022),
            (0.17, 0.04),
            (0.09, 0.062),
            (0.05, 0.075),
            (0.0, 0.075)
        ],
        at = F.at(0, 0, 0),
        material = M.iron,
        segments = 20
    );
    finish("patioTableRound", false)
}

// --- the teak patio table: a planked top framed by breadboard ends, on arched aprons and tapered legs ----------
pub fn patio_table4() -> Vec<Obj> {
    let (zc, y) = (-0.04, 0.74);
    for (k, z) in [-0.36, -0.2, -0.04, 0.12, 0.28].into_iter().enumerate() {
        fbox!(
            F,
            format!("top{k}"),
            1.5,
            0.03,
            0.143,
            (0, y - 0.015, z),
            M.teak,
            bevel = 0.007
        );
    }
    for s in [-1, 1] {
        // the breadboard ends across the planks
        let sf = s as f64;
        fbox!(
            F,
            format!("end{s}"),
            0.05,
            0.03,
            0.785,
            (sf * 0.775, y - 0.015, zc),
            M.teak,
            bevel = 0.007
        );
        for z in [-0.3, -0.04, 0.22] {
            // the pegs through them
            fcyl!(
                F,
                format!("peg{s}{}", float_repr(z)),
                0.007,
                0.003,
                (sf * 0.775, y + 0.0005, z),
                M.teak_dark,
                verts = 6
            );
        }
    }
    // the aprons, their lower edges arched between the legs
    for s in [-1, 1] {
        let sf = s as f64;
        let mut arch = vec![(0.69, 0.03), (-0.69, 0.03)];
        arch.extend((0..13).map(|i| {
            (
                -0.69 + 1.38 * i as f64 / 12.0,
                -0.03 + 0.018 * sin(PI * i as f64 / 12.0),
            )
        }));
        fextrude!(
            F,
            format!("apron{s}"),
            &arch,
            0.026,
            (0, 0.68, zc + sf * 0.33),
            M.teak_dark,
            bevel = 0.004,
            bevel_segments = 1
        );
        let mut arch = vec![(-0.3, 0.03), (0.3, 0.03)];
        arch.extend((0..9).map(|i| {
            (
                0.3 - 0.6 * i as f64 / 8.0,
                -0.03 + 0.014 * sin(PI * i as f64 / 8.0),
            )
        }));
        let side: Vec<(f64, f64)> = arch.iter().map(|&(u, v)| (zc + u, v + 0.68)).collect();
        side_slab(
            &format!("apron_end{s}"),
            sf * 0.7,
            &side,
            0.026,
            M.teak_dark,
            0.004,
        );
        for e in [-1, 1] {
            tapered_leg(
                &format!("leg{s}{e}"),
                sf * 0.72,
                zc + e as f64 * 0.33,
                0.0,
                y - 0.03,
                0.044,
                0.06,
                M.teak_dark,
                16,
            );
        }
    }
    // an H stretcher: a foot rail at each end, a rail between their middles
    fbox!(
        F,
        "stretcher",
        1.4,
        0.04,
        0.035,
        (0, 0.15, zc),
        M.teak_dark,
        bevel = 0.008
    );
    for s in [-1, 1] {
        fbox!(
            F,
            format!("foot_rail{s}"),
            0.035,
            0.045,
            0.62,
            (s as f64 * 0.72, 0.14, zc),
            M.teak_dark,
            bevel = 0.008
        );
    }
    finish("patioTable4", false)
}

// --- the patio chair: a bent-iron frame, slatted back and a seat cushion ---------------------------------------
pub fn patio_chair() -> Vec<Obj> {
    let (sz, r) = (0.05, 0.0125);
    // the seat: a pan on a tube frame, a piped cushion tied on, its top at 0.48 (a dining chair's 0.42-0.48)
    let ring: Vec<(f64, f64, f64)> = rounded_rect(0.44, 0.4, 0.04, 2)
        .iter()
        .map(|&(u, v)| (u, 0.412, sz + v))
        .collect();
    fsweep!(
        F,
        "seat_frame",
        &ring,
        0.011,
        M.patio_green,
        verts = 6,
        closed = true
    );
    fbox!(
        F,
        "seat",
        0.42,
        0.012,
        0.38,
        (0, 0.42, sz),
        M.patio_green,
        bevel = 0.004,
        bevel_segments = 1
    );
    soft!(
        F,
        "cushion",
        0.42,
        0.05,
        0.38,
        (0, 0.446, sz + 0.01),
        M.cushion,
        round = 0.02,
        puff = 0.008,
        flat = 2,
        piping = 0.004,
        round_segments = 1
    );
    // the back: one bent tube from foot to foot over the top; each front leg runs up into an arm and back into it
    bent(
        "back_frame",
        &[
            (-0.2, 0.0, sz - 0.22),
            (-0.19, 0.915, sz - 0.2),
            (0.19, 0.915, sz - 0.2),
            (0.2, 0.0, sz - 0.22),
        ],
        r,
        M.patio_green,
        0.07,
        8,
        3,
        1.0,
    );
    for s in [-1, 1] {
        let x = s as f64 * 0.2;
        bent(
            &format!("arm{s}"),
            &[
                (x, 0.0, sz + 0.204),
                (x, 0.66, sz + 0.17),
                (x * 0.985, 0.672, sz - 0.205),
            ],
            r,
            M.patio_green,
            0.05,
            8,
            3,
            1.0,
        );
        for z in [sz + 0.204, sz - 0.22] {
            // glides under the feet
            fcyl!(
                F,
                format!("glide{s}{}", float_repr(z)),
                0.0125,
                0.008,
                (x, 0.004, z),
                M.patio_green,
                verts = 8
            );
        }
    }
    // three back slats, bowed back
    for (k, y) in [0.6, 0.72, 0.84].into_iter().enumerate() {
        let z = sz - 0.207 - (y - 0.6) * 0.02;
        let bow: Vec<(f64, f64, f64)> = [-0.19, -0.1, 0.0, 0.1, 0.19]
            .iter()
            .map(|&x| (x, y, z - 0.012 * (1.0 - pow(x / 0.19, 2.0))))
            .collect();
        fsweep!(
            F,
            format!("back{k}"),
            &bow,
            0,
            M.patio_green,
            profile = rounded_rect(0.035, 0.01, 0.004, 1),
            up = (0, 0, 1),
            resolution = 2
        );
    }
    rod(
        "stretcher",
        (-0.2, 0.15, sz - 0.21),
        (0.2, 0.15, sz - 0.21),
        0.009,
        M.patio_green,
        8,
    );
    finish("patioChair", false)
}

// --- the outdoor bar: a slatted teak counter with a back shelf and a foot rail ---------------------------------
pub fn outdoor_bar() -> Vec<Obj> {
    let (zb, zf) = (-0.37, 0.13); // the body's back and front
    slab(
        "counter",
        2.0,
        0.88,
        1.06,
        0.06,
        M.bar_top,
        0.025,
        (0.0, -0.04),
    );
    // the carcass: end panels, a front behind the slats, a middle back with open shelves behind it
    for s in [-1, 1] {
        let sf = s as f64;
        fbox!(
            F,
            format!("end{s}"),
            0.03,
            0.94,
            zf - zb,
            (sf * 0.985, 0.53, (zb + zf) / 2.0),
            M.teak,
            bevel = 0.006,
            bevel_segments = 1
        );
        for k in 0..4 {
            // the end's slats
            fbox!(
                F,
                format!("end_slat{s}{k}"),
                0.02,
                0.86,
                0.1,
                (sf * 0.99, 0.5, zb + 0.07 + 0.12 * k as f64),
                M.teak,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
    }
    fbox!(F, "front", 1.94, 0.94, 0.02, (0, 0.53, zf - 0.01), M.teak);
    fbox!(F, "back", 1.94, 0.94, 0.02, (0, 0.53, -0.12), M.teak_dark);
    for y in [0.07, 0.48] {
        // the shelves in the back, and their divider
        fbox!(
            F,
            format!("shelf{}", float_repr(y)),
            1.94,
            0.025,
            0.23,
            (0, y, -0.245),
            M.teak,
            bevel = 0.004,
            bevel_segments = 1
        );
    }
    fbox!(F, "divider", 0.025, 0.86, 0.23, (0, 0.5, -0.245), M.teak);
    fbox!(
        F,
        "plinth",
        1.92,
        0.06,
        0.06,
        (0, 0.03, zf + 0.0),
        M.teak_dark
    );
    for k in 0..18 {
        // the front's slats, a gap between each
        fbox!(
            F,
            format!("slat{k}"),
            0.096,
            0.9,
            0.02,
            (-0.935 + 0.11 * k as f64, 0.51, zf + 0.012),
            M.teak,
            bevel = 0.004,
            bevel_segments = 1
        );
    }
    fbox!(
        F,
        "apron",
        2.0,
        0.05,
        0.03,
        (0, 0.975, zf + 0.025),
        M.teak_dark,
        bevel = 0.005,
        bevel_segments = 1
    );
    // the speed rail under the counter's back edge, bottles and glasses on the shelves
    fbox!(
        F,
        "rail_shelf",
        2.0,
        0.04,
        0.26,
        (0, 0.92, -0.36),
        M.bar_top,
        bevel = 0.006,
        bevel_segments = 1
    );
    let kinds = [
        (M.bottle_green, 0.034, 0.3),
        (M.bottle_amber, 0.03, 0.26),
        (M.bottle, 0.032, 0.28),
        (M.bottle_clear, 0.03, 0.3),
    ];
    for (k, x) in [-0.86, -0.78, -0.7, 0.2, 0.3].into_iter().enumerate() {
        let (mat, r, h) = kinds[k % 4];
        let odd = k % 2 == 1;
        bottle(
            &format!("bottle{k}"),
            x,
            if odd { 0.4925 } else { 0.0825 },
            -0.24 + 0.03 * (k % 2) as f64,
            r,
            h * (if odd { 0.8 } else { 1.0 }),
            mat,
            Some(M.steel),
            8,
        );
    }
    for (k, x) in [-0.4, -0.3, 0.65, 0.75].into_iter().enumerate() {
        cup(
            &format!("glass{k}"),
            x,
            0.4925,
            -0.25,
            0.032,
            0.11,
            M.bottle_clear,
            false,
            0.0,
            8,
        );
    }
    // the foot rail: one bent tube out on two brackets, a third in the middle, flanges on the slats
    bent(
        "foot_rail",
        &[
            (-0.94, 0.2, zf + 0.02),
            (-0.94, 0.2, 0.26),
            (0.94, 0.2, 0.26),
            (0.94, 0.2, zf + 0.02),
        ],
        0.02,
        M.steel,
        0.06,
        12,
        3,
        1.0,
    );
    rod(
        "bracket",
        (0, 0.2, zf + 0.02),
        (0, 0.2, 0.25),
        0.013,
        M.steel,
        10,
    );
    for x in [-0.94, 0.0, 0.94] {
        cylinder!(
            format!("flange{}", float_repr(x)),
            0.035,
            0.008,
            at = F.at(x, 0.2, zf + 0.026),
            material = M.steel,
            rot = (90, 0, 0),
            verts = 12
        );
    }
    finish("outdoorBar", true)
}

// --- the parasol: an octagonal canopy on ribs, on a pole in a weighted base ------------------------------------
const CANOPY_Y: f64 = 1.97; // the canopy's rim, height
const CANOPY_R: f64 = 1.19; // the rib tips' radius, the apex over the rim
const CANOPY_H: f64 = 0.39;

/// A Blender point on the canopy `t` of the way from its apex to its rim, `j` of `n` steps across the panel between
/// the ribs at angles a and b (degrees); the cloth sags between the ribs; `drop` lowers it, `out` pushes it out.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn canopy_point(
    a: f64,
    b: f64,
    j: usize,
    n: usize,
    t: f64,
    drop: f64,
    out: f64,
) -> (f64, f64, f64) {
    let r = 0.04 + (CANOPY_R - 0.04) * t + out;
    let pa = V3::from((r * cosd(a), r * sind(a), 0));
    let pb = V3::from((r * cosd(b), r * sind(b), 0));
    let f = j as f64 / n as f64;
    let mut p = pa + (pb - pa) * f;
    let sag = sin(PI * f);
    p *= 1.0 - 0.035 * sag * t;
    let z = CANOPY_Y + CANOPY_H * (1.0 - pow(t, 1.3)) - 0.045 * sag * pow(t, 0.8) - drop;
    (p.x as f64, p.y as f64, z)
}

fn ring_of(t: f64, n: usize, drop: f64, out: f64) -> Vec<P3> {
    (0..8)
        .flat_map(|k| (0..n).map(move |j| (k, j)))
        .map(|(k, j)| {
            P3::from(canopy_point(
                22.5 + 45.0 * k as f64,
                22.5 + 45.0 * (k + 1) as f64,
                j,
                n,
                t,
                drop,
                out,
            ))
        })
        .collect()
}

pub fn parasol() -> Vec<Obj> {
    let still = new_parts(|| {
        revolve!(
            "base",
            &[
                (0.0, 0.0),
                (0.2, 0.0),
                (0.2, 0.025),
                (0.185, 0.045),
                (0.12, 0.06),
                (0.06, 0.07),
                (0.0, 0.07)
            ],
            at = F.at(0, 0, 0),
            material = M.base,
            segments = 20
        );
        revolve!(
            "base_boss",
            &[
                (0.0, 0.07),
                (0.045, 0.07),
                (0.04, 0.12),
                (0.03, 0.135),
                (0.0, 0.135)
            ],
            at = F.at(0, 0, 0),
            material = M.base,
            segments = 12
        );
        rod(
            "base_screw",
            (0.03, 0.105, 0),
            (0.09, 0.105, 0),
            0.007,
            M.base,
            6,
        );
        fcyl!(
            F,
            "base_knob",
            0.016,
            0.03,
            (0.095, 0.105, 0),
            M.base,
            rot = (0, 0, 90),
            verts = 8
        );
        fcyl!(F, "pole", 0.02, 1.62, (0, 0.93, 0), M.pole, verts = 12);
        // the crank's housing on the pole, its handle turning to the front
        fbox!(
            F,
            "crank",
            0.06,
            0.1,
            0.06,
            (0, 1.3, 0),
            M.base,
            bevel = 0.012,
            bevel_segments = 2
        );
        bent(
            "crank_arm",
            &[(0.0, 1.3, 0.03), (0.0, 1.3, 0.05), (0.07, 1.27, 0.05)],
            0.006,
            M.base,
            0.012,
            6,
            3,
            1.0,
        );
        fcyl!(
            F,
            "crank_knob",
            0.01,
            0.035,
            (0.07, 1.27, 0.065),
            M.base,
            rot = (90, 0, 0),
            verts = 8
        );
    });
    join(&still, "stand", Some(P3::from((0, 0, 0))));
    let top = new_parts(|| {
        fcyl!(F, "pole_top", 0.02, 0.7, (0, 2.06, 0), M.pole, verts = 12);
        fcyl!(
            F,
            "runner",
            0.03,
            0.07,
            (0, 1.78, 0),
            M.pole,
            verts = 12,
            bevel = 0.006,
            bevel_segments = 1
        );
        // the canopy: eight panels, sagging between their ribs, in one piece with its thickness; the scalloped valance
        let ts = [0.0, 0.15, 0.35, 0.55, 0.75, 0.9, 1.0];
        let mut sections: Vec<Vec<P3>> = ts.iter().map(|&t| ring_of(t, 4, 0.0, 0.0)).collect();
        sections.extend(ts.iter().rev().map(|&t| ring_of(t, 4, 0.01, 0.0)));
        loft!("canopy", &sections, material = M.canopy, angle = 30);
        let mut val: Vec<Vec<P3>> = Vec::new();
        for (drop, out) in [
            (Some(0.004), 0.003),
            (None, 0.003),
            (None, -0.004),
            (Some(0.004), -0.004),
        ] {
            let mut lp = Vec::new();
            for k in 0..8 {
                for j in 0..6 {
                    let f = j as f64 / 6.0;
                    let d = drop.unwrap_or_else(|| 0.075 + 0.045 * pow(sin(PI * f), 0.7));
                    lp.push(P3::from(canopy_point(
                        22.5 + 45.0 * k as f64,
                        22.5 + 45.0 * (k + 1) as f64,
                        j,
                        6,
                        1.0,
                        d,
                        out,
                    )));
                }
            }
            val.push(lp);
        }
        val.push(val[0].clone());
        loft!(
            "valance",
            &val,
            material = M.valance,
            caps = false,
            angle = 50
        );
        // the ribs under it, each with a strut from the runner; a hub and a finial over the apex
        for k in 0..8 {
            let a = 22.5 + 45.0 * k as f64;
            let rib: Vec<P3> = [0.04, 0.3, 0.6, 0.85, 0.99]
                .iter()
                .map(|&t| P3::from(canopy_point(a, a, 0, 1, t, 0.016, 0.0)))
                .collect();
            sweep!(
                format!("rib{k}"),
                &rib,
                0.006,
                material = M.pole,
                verts = 5,
                resolution = 2
            );
            let mid = canopy_point(a, a, 0, 1, 0.48, 0.02, 0.0);
            rod_!(
                format!("strut{k}"),
                (0.03 * cosd(a), 0.03 * sind(a), 1.8),
                mid,
                0.005,
                material = M.pole,
                verts = 4
            );
            sphere!(
                format!("tip{k}"),
                0.011,
                at = canopy_point(a, a, 0, 1, 1.0, 0.012, 0.004),
                material = M.pole,
                segments = 6,
                rings = 4
            );
        }
        fcyl!(F, "hub", 0.035, 0.05, (0, 2.33, 0), M.pole, verts = 10);
        revolve!(
            "finial",
            &[
                (0.0, 0.0),
                (0.04, 0.0),
                (0.042, 0.012),
                (0.02, 0.03),
                (0.026, 0.055),
                (0.018, 0.075),
                (0.0, 0.08)
            ],
            at = F.at(0, 2.355, 0),
            material = M.pole,
            segments = 10
        );
    });
    let canopy = hinged(join(&top, "canopy", None), F.at(0, 1.3, 0));
    settle!(centre = false);
    // 3°: at 1.2° the rim moved a pixel or two from the game camera, too little to notice
    rock(
        canopy,
        "stir",
        3.0,
        Rock {
            seconds: 2.4,
            axis: Axis::X,
        },
    );
    meshes()
}

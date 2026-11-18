//! What the dressing things are built from: the palette, the frame, mouldings swept round an outline, mirror glass
//! gridded for its shading.

use artkit::forms;
use artkit::materials::{Mat, Mirror};
use artkit::math::{P3, PI, V3, cos, radians, sin};
use artkit::mesh::Mesh;
use artkit::palette;
use artkit::place::{Coord, GameFrame};
use artkit::py;
use artkit::scene::Obj;
use artkit::shapes::Revolve;
use artkit::{fbox, revolve};

palette! { M:
    // the cheval mirror
    frame = ("#A8754A", wood),          // posts, feet, swing frame and crest (the colour part)
    backing = ("#7A5234", wood),        // the board behind the glass
    brass = ("#B8925A", metal),         // pivot knobs and axles
    mirror = ("#DCE6EA", gloss, mirror(Mirror::Front)),     // glass that reflects to the front (+z)
    // the dressing table
    table = ("#CDB08A", wood),          // top, drawer cases, drawers (the main colour part)
    table_frame = ("#B8925A", metal),   // legs, the mirror's ring and stand, the drawer knobs
    pouf = ("#E4B8AE", satin),          // the pouf (plain, not fabric: a draw call saved for the drawer)
    inside = ("#3A332C", satin),        // the dark behind an open drawer
    perfume = ("#E8B4C0", gloss),
    lotion = ("#E8E6E1", gloss),
    gilt = ("#C9A54A", metal),          // bottle caps
    case = ("#2A2C30", satin),          // the jewellery box
    // the wall mirror
    wall_frame = ("#2B2C2E", metal),    // frame and ledge (the colour part)
    cup = ("#9DC6CF", gloss),
    brush = ("#F2F1EE", gloss),
    bristle = ("#DDEFF2", matte),       // the toothbrush's bristles
    paste = ("#E9E6EF", gloss),         // a tube of toothpaste
    paste_cap = ("#C65D52", gloss),
    // the standing mirror
    stand = ("#C9A06A", metal),         // the brass frame and stand (the colour part)
    stand_back = ("#3A332C", satin),
}

pub const F: GameFrame = GameFrame::new(0.0);

// --- shapes ---------------------------------------------------------------------------------------------

/// A rounded rectangle `w` x `h` (corner radius `r`) as [(u, v, nu, nv)]: points round it, anticlockwise, with
/// their outward normals.
pub fn rounded(w: f64, h: f64, r: f64, seg: usize) -> Vec<(f64, f64, f64, f64)> {
    let mut pts = Vec::new();
    for (k, (sx, sy)) in [(1.0, 1.0), (-1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)]
        .into_iter()
        .enumerate()
    {
        let (cu, cv) = (sx * (w / 2.0 - r), sy * (h / 2.0 - r));
        for i in 0..=seg {
            let a = radians((90 * k) as f64 + (90 * i) as f64 / seg as f64);
            pts.push((cu + r * cos(a), cv + r * sin(a), cos(a), sin(a)));
        }
    }
    pts
}

/// An ellipse of half-axes `a` (across) and `b` (up) as [(u, v, nu, nv)].
pub fn ellipse(a: f64, b: f64, n: usize) -> Vec<(f64, f64, f64, f64)> {
    (0..n)
        .map(|k| {
            let t = 2.0 * PI * k as f64 / n as f64;
            let (nu, nv) = (b * cos(t), a * sin(t));
            let length = py::hypot(nu, nv);
            (a * cos(t), b * sin(t), nu / length, nv / length)
        })
        .collect()
}

/// A frame's cross-section [(o, d)]: `w` wide along the outline's normal (o: - in, + out) and `d` deep (+ to the
/// front): a flat back, a rounded outer edge, a cove and a bead, and a lip falling to the glass, standing well in
/// front of it (the glass is set in behind).
pub fn moulding(w: f64, d: f64) -> Vec<(f64, f64)> {
    let (hw, hd) = (w / 2.0, d / 2.0);
    vec![
        (-hw, -hd),
        (hw, -hd),
        (hw, hd * 0.2),
        (hw * 0.85, hd * 0.75),
        (hw * 0.5, hd),
        (hw * 0.15, hd * 0.92),
        (-hw * 0.05, hd * 0.8),
        (-hw * 0.3, hd * 0.98),
        (-hw * 0.6, hd * 0.92),
        (-hw * 0.85, hd * 0.82),
        (-hw, hd * 0.75),
    ]
}

/// A frame swept round `outline` (in the mirror's plane, about game `centre`), its cross-section `section` [(o, d)]:
/// o along the outline's normals, d along game z (`facing` -1: its front to -z).
pub fn frame_loop(
    name: &str,
    outline: &[(f64, f64, f64, f64)],
    section: &[(f64, f64)],
    centre: (impl Coord, f64, f64),
    material: Mat,
    facing: f64,
) -> Obj {
    let (cx, cy, cz) = (centre.0.f(), centre.1, centre.2);
    let mut verts: Vec<V3> = Vec::new();
    for &(u, v, nu, nv) in outline {
        for &(o, d) in section {
            verts.push(
                F.at(cx + u + nu * o, cy + v + nv * o, cz + facing * d)
                    .into(),
            );
        }
    }
    let (n, m) = (outline.len(), section.len());
    let mut faces: Vec<Vec<u32>> = Vec::new();
    for i in 0..n {
        for j in 0..m {
            faces.push(vec![
                (i * m + j) as u32,
                (((i + 1) % n) * m + j) as u32,
                (((i + 1) % n) * m + (j + 1) % m) as u32,
                (i * m + (j + 1) % m) as u32,
            ]);
        }
    }
    forms::object(name, verts, &faces, Some(material), true, 50.0, false)
}

/// Mirror glass: a flat box `w` x `h`, `t` thick at game `at`, its faces gridded, the grid's outer lines `inset` in
/// from its edges, so the shading baked at its vertices darkens only that rim, set into the frame, not the whole
/// pane (with only its corners, buried in the frame, it baked grey all over).
pub fn pane(name: &str, w: f64, h: f64, t: f64, at: (impl Coord, f64, f64), material: Mat) -> Obj {
    pane_with(name, w, h, t, at, material, 5, 0.012)
}

/// `pane` with its grid's `cuts` and `inset` given.
#[allow(clippy::too_many_arguments)]
pub fn pane_with(
    name: &str,
    w: f64,
    h: f64,
    t: f64,
    at: (impl Coord, f64, f64),
    material: Mat,
    cuts: usize,
    inset: f64,
) -> Obj {
    let obj = fbox!(F, name, w, h, t, at, material);
    // the box's long edges (longer than twice its thickness: across and up) cut `cuts` times, its faces' cuts
    // joined: a grid on its front and back, strips round its sides
    let mesh = subdivided_pane(w, h, t, cuts);
    let remap = |c: f64, half: f64| {
        // the first grid line in from each edge moves out to `inset`, the lines between spread evenly
        let mut k = c.abs() / half;
        let step = 1.0 / (cuts + 1) as f64;
        let a = 1.0 - inset / half; // where the first line in from the edge goes, as a fraction of `half`
        if k <= 1.0 - 2.0 * step + 1e-9 {
            k = k * a / (1.0 - 2.0 * step);
        } else {
            k = a + (k - (1.0 - 2.0 * step)) / (2.0 * step) * (1.0 - a);
        }
        (k * half).copysign(c)
    };
    let mut mesh = mesh;
    for v in &mut mesh.verts {
        v.x = remap(v.x as f64, w / 2.0) as f32;
        v.z = remap(v.z as f64, h / 2.0) as f32;
    }
    mesh.set_material(Some(material));
    mesh.recalc_normals();
    mesh.shade_flat();
    obj.edit(|o| o.mesh = mesh);
    obj
}

// KIT: bmesh.ops.subdivide_edges (cuts, use_grid_fill) on a box's edges longer than twice its thickness
/// A box `w` (x) by `t` (y) by `h` (z) about its middle, its edges along x and z each cut `cuts` times evenly: its
/// faces at ±y gridded (grid fill), the four round its sides cut into strips.
fn subdivided_pane(w: f64, h: f64, t: f64, cuts: usize) -> Mesh {
    let n = cuts + 1;
    let along = |i: usize, size: f64| -size / 2.0 + size * i as f64 / n as f64;
    let mut verts: Vec<V3> = Vec::new();
    let mut index = std::collections::BTreeMap::new();
    let mut at = |i: usize, j: usize, k: usize, verts: &mut Vec<V3>| -> u32 {
        *index.entry((i, j, k)).or_insert_with(|| {
            verts.push(V3::from(P3 {
                x: along(i, w),
                y: if j == 0 { -t / 2.0 } else { t / 2.0 },
                z: along(k, h),
            }));
            (verts.len() - 1) as u32
        })
    };
    let mut faces: Vec<Vec<u32>> = Vec::new();
    // the front (-y) and back (+y): a grid
    for j in [0, 1] {
        for i in 0..n {
            for k in 0..n {
                faces.push(vec![
                    at(i, j, k, &mut verts),
                    at(i + 1, j, k, &mut verts),
                    at(i + 1, j, k + 1, &mut verts),
                    at(i, j, k + 1, &mut verts),
                ]);
            }
        }
    }
    // the sides at ±x: strips up z; the top and bottom at ±z: strips across x
    for i in [0, n] {
        for k in 0..n {
            faces.push(vec![
                at(i, 0, k, &mut verts),
                at(i, 1, k, &mut verts),
                at(i, 1, k + 1, &mut verts),
                at(i, 0, k + 1, &mut verts),
            ]);
        }
    }
    for k in [0, n] {
        for i in 0..n {
            faces.push(vec![
                at(i, 0, k, &mut verts),
                at(i + 1, 0, k, &mut verts),
                at(i + 1, 1, k, &mut verts),
                at(i, 1, k, &mut verts),
            ]);
        }
    }
    Mesh::from_pydata(verts, &faces)
}

/// A flat oval `w` x `h`, `t` thick along game z, upright at game `at` (mirror glass, a back plate), its faces in
/// rings, so the shading baked at its vertices darkens it only at its rim.
pub fn oval(
    name: &str,
    w: f64,
    h: f64,
    t: f64,
    at: (impl Coord, f64, f64),
    material: Mat,
    rings: &[f64],
) -> Obj {
    let r = w / 2.0;
    let mut prof = vec![(0.0, -t / 2.0), (r, -t / 2.0), (r, t / 2.0)];
    prof.extend(rings.iter().map(|&k| (r * k, t / 2.0)));
    prof.push((0.0, t / 2.0));
    revolve!(
        name,
        &prof,
        at = F.at(at.0, at.1, at.2),
        material = material,
        segments = 48,
        scale = (1.0, h / w, 1.0),
        rot = (90, 0, 0)
    )
}

/// A turned post at game (x, z): `profile` is [(radius, height)], bottom first.
pub fn turned(
    name: &str,
    x: impl Coord,
    z: impl Coord,
    profile: &[(f64, f64)],
    material: Mat,
    segments: usize,
) -> Obj {
    forms_revolve(name, profile, F.at(x, 0, z), material, segments)
}

fn forms_revolve(
    name: &str,
    profile: &[(f64, f64)],
    at: P3,
    material: Mat,
    segments: usize,
) -> Obj {
    artkit::shapes::revolve(
        name,
        profile,
        Revolve {
            at,
            material: Some(material),
            segments: Some(segments),
            ..Revolve::default()
        },
    )
}

/// A turned knob on the side facing game x's sign `s`, its rose at game (x, y, z).
pub fn side_knob(name: &str, x: f64, y: f64, z: impl Coord, r: f64, s: i32, material: Mat) -> Obj {
    let k = super::kit::knob!(F, name, x, y, z, r, material);
    k.set_rotation_euler((0.0, radians((-90 * s) as f64), 0.0))
}

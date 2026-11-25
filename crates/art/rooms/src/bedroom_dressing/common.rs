//! What the dressing things are built from: the palette, the frame, mouldings swept round an outline, mirror glass
//! gridded for its shading.

use artkit::materials::{Mat, Mirror};
use artkit::math::{PI, V3, cos, radians, sin};
use artkit::palette;
use artkit::place::{Coord, GameFrame};
use artkit::py;
use artkit::scene::Obj;
use artkit::{edit, forms, subdivide_edges};
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
/// front of it (the glass is set in behind). Its square corners are eased 3 mm.
pub fn moulding(w: f64, d: f64) -> Vec<(f64, f64)> {
    let (hw, hd) = (w / 2.0, d / 2.0);
    let section = vec![
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
    ];
    // its back's and the lip's square corners eased (the gate's `edges`)
    crate::common::eased_loop(&section, 0.003)
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
    let pts = obj.read(|o| o.mesh.verts.clone());
    let long: Vec<(u32, u32)> = edit::edges(obj)
        .into_iter()
        .filter(|&(a, b)| (pts[b as usize] - pts[a as usize]).length() > t * 2.0)
        .collect();
    subdivide_edges!(obj, &long, cuts = cuts, use_grid_fill = true);
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
    obj.edit(|o| {
        for v in &mut o.mesh.verts {
            v.x = remap(v.x as f64, w / 2.0) as f32;
            v.z = remap(v.z as f64, h / 2.0) as f32;
        }
    });
    obj
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

/// A turned knob on the side facing game x's sign `s`, its rose at game (x, y, z).
pub fn side_knob(name: &str, x: f64, y: f64, z: impl Coord, r: f64, s: i32, material: Mat) -> Obj {
    let k = artkit::kit::knob!(F, name, x, y, z, r, material);
    k.set_rotation_euler((0.0, radians((-90 * s) as f64), 0.0))
}

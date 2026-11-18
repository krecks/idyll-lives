//! What the fridges and the sinks are built from: parts every model here shares (each in the model's own
//! GameFrame), and the glossy round shading of a bevelled shell.

use super::M;
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::V3;
use artkit::mesh::{Mesh, edge};
use artkit::place::{Coord, Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fcyl, fextrude, fsphere, fsweep, loft, ring, soft};
use std::collections::BTreeMap;

/// Python's `min` of two: the first unless the second is strictly below it.
pub fn py_min(a: f64, b: f64) -> f64 {
    if b < a { b } else { a }
}

/// Python's `max` of two: the first unless the second is strictly above it.
pub fn py_max(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

// KIT: kit.tube
#[allow(clippy::too_many_arguments)] // (as the kit has it)
/// A round tube of radius `r` through game points, its bends rounded (`round` steps between each pair of points; 1
/// keeps them sharp), its ends capped. `verts` round: by default 8, and more for one over 13 mm in radius, as many
/// as keep its outline within a millimetre of the circle (`forms.sweep`).
pub fn tube<P: artkit::place::Spot>(
    frame: &GameFrame,
    name: &str,
    points: &[P],
    r: f64,
    material: Mat,
    verts: Option<usize>,
    round: usize,
    scale: f64,
) -> Obj {
    let o = artkit::forms::Sweep {
        verts,
        resolution: round,
        scale: artkit::forms::Scale::One(scale),
        least: 8,
        ..artkit::forms::Sweep::default()
    };
    frame.sweep(name, points, r, material, o)
}

// --- parts the K1 fridges and the sinks share (each in the model's own GameFrame) ---------------------------

/// A bar handle in one piece from face point `a` (x, height) to `b`: out from the face at game depth `face` on
/// rounded stand-offs, along, and back in, its front `out` proud; a rounded `w` × `t` section (`r` None: a round bar
/// of radius w / 2).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn pull(
    f: &GameFrame,
    name: impl AsRef<str>,
    a: (f64, f64),
    b: (f64, f64),
    face: f64,
    material: Option<Mat>,
    out: f64,
    w: f64,
    t: f64,
    r: Option<f64>,
    verts: usize,
) -> Obj {
    let ((x0, y0), (x1, y1)) = (a, b);
    let n = artkit::py::hypot(x1 - x0, y1 - y0);
    let (ux, uy) = ((x1 - x0) / n, (y1 - y0) / n);
    let (e, o) = (
        py_min(0.012, n / 4.0),
        face + out - (if r.is_none() { w } else { t }) / 2.0,
    );
    let pts = [
        (x0, y0, face - 0.004),
        (x0, y0, o - e),
        (x0 + ux * e, y0 + uy * e, o),
        (x1 - ux * e, y1 - uy * e, o),
        (x1, y1, o - e),
        (x1, y1, face - 0.004),
    ];
    let material = material.unwrap_or(M.chrome);
    match r {
        None => fsweep!(
            f,
            name,
            &pts,
            w / 2.0,
            material,
            up = (uy, 0, ux),
            resolution = 2,
            angle = 60.0,
            verts = verts
        ),
        Some(r) => fsweep!(
            f,
            name,
            &pts,
            0.0,
            material,
            up = (uy, 0, ux),
            resolution = 2,
            angle = 60.0,
            profile = rounded_rect(w, t, r, 1)
        ),
    }
}

/// `pull` with the script's defaults but for the ones named.
pub struct Pull {
    pub material: Option<Mat>,
    pub out: f64,
    pub w: f64,
    pub t: f64,
    pub r: Option<f64>,
    pub verts: usize,
}

impl Default for Pull {
    fn default() -> Pull {
        Pull {
            material: None,
            out: 0.034,
            w: 0.016,
            t: 0.02,
            r: Some(0.007),
            verts: 8,
        }
    }
}

/// A cabinet door or drawer front: a frame (stiles and rails, one slab with a hole) round a panel set 6 mm back.
/// `at` is the middle of its front face's slab (game x, height, depth).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn shaker<X: Coord, Y: Coord>(
    f: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    at: (X, Y, f64),
    material: Mat,
    t: f64,
    rail: f64,
    r: f64,
) {
    let (x, y, z) = at;
    let hole = rounded_rect(w - 2.0 * rail, h - 2.0 * rail, 0.003, 1);
    fextrude!(
        f,
        format!("{name}_frame"),
        &rounded_rect(w, h, r, 1),
        t,
        at,
        material,
        holes = [hole],
        bevel = 0.003,
        bevel_segments = 1
    );
    // the panel a hair inside the hole, lightly puffed: its grid gives the baked shading points away from the frame's
    // edge, so only its border darkens (a flat box's four corners, hidden in the frame, darkened all of it)
    soft!(
        f,
        format!("{name}_panel"),
        w - 2.0 * rail - 0.002,
        h - 2.0 * rail - 0.002,
        t * 0.6,
        (x, y, z + t * 0.2 - 0.006),
        material,
        puff = 0.002,
        round = 0.003,
        round_segments = 1,
        flat = 2
    );
}

/// A door's rubber seal: a cord round the back of a w × h door whose middle is at (x, y), at depth `z`.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn gasket(
    f: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    y: f64,
    z: f64,
    r: f64,
    corner: f64,
    x: f64,
) -> Obj {
    let lp = rounded_rect(w, h, corner, 2);
    let pts: Vec<(f64, f64, f64)> = lp.iter().map(|&(u, v)| (x + u, y + v, z)).collect();
    fsweep!(f, name, &pts, r, M.gasket, verts = 5, closed = true)
}

/// A kick plate with a row of vent slots, over a dark recess, at height `y` (its middle) and depth `z`.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn kick(
    f: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    y: f64,
    z: f64,
    material: Option<Mat>,
    slots: usize,
) -> Obj {
    artkit::fbox!(
        f,
        format!("{name}_recess"),
        w - 0.02,
        h,
        0.01,
        (0, y, z - 0.009),
        M.dark
    );
    let slot = rounded_rect(
        py_min(0.07, w / slots as f64 - 0.03),
        py_min(0.012, h * 0.3),
        0.005,
        1,
    );
    let step = (w - 0.06) / slots as f64;
    let holes: Vec<Vec<(f64, f64)>> = (0..slots)
        .map(|i| {
            slot.iter()
                .map(|&(u, v)| (u + (i as f64 - (slots - 1) as f64 / 2.0) * step, v))
                .collect()
        })
        .collect();
    fextrude!(
        f,
        name,
        &rounded_rect(w, h, 0.006, 1),
        0.008,
        (0, y, z),
        material.unwrap_or(M.seal),
        holes = holes
    )
}

/// A gable-top carton standing on game height `y`: its side outline (a house with a fin) extruded across.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn carton(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
    h: f64,
    material: Mat,
    turn: i32,
) -> Obj {
    let (d, g) = (w, w * 0.42);
    let outline = [
        (-d / 2.0, 0.0),
        (d / 2.0, 0.0),
        (d / 2.0, h),
        (0.003, h + g),
        (0.003, h + g + 0.014),
        (-0.003, h + g + 0.014),
        (-0.003, h + g),
        (-d / 2.0, h),
    ];
    fextrude!(
        f,
        name,
        &outline,
        w,
        (x, y, z),
        material,
        plane = Facing::Side,
        rot = (0, 0, turn),
        bevel = 0.002,
        bevel_segments = 1
    )
}

/// An apple sitting on game height `y`: a squat ball with a dimple's stalk.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn apple(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, r: f64, material: Option<Mat>) {
    fsphere!(
        f,
        name,
        r,
        (x, y + r * 0.92, z),
        material.unwrap_or(M.produce),
        segments = 10,
        rings = 7,
        scale = (1, 1, 0.92)
    );
    fcyl!(
        f,
        format!("{name}_stalk"),
        0.0028,
        0.016,
        (x + 0.002, y + r * 1.78 + 0.006, z),
        M.board,
        verts = 5,
        smooth = false
    );
}

/// A jar on game height `y`: a rounded body, a shoulder and a screw lid.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn jar(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    material: Mat,
    lid: Mat,
    verts: usize,
) {
    let b = f.at(x, y, z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    let rings = [
        (0.0, 0.86),
        (0.04, 1.0),
        (0.8, 1.0),
        (0.9, 0.84),
        (0.92, 0.8),
    ];
    let sections: Vec<_> = rings
        .iter()
        .map(|&(t, k)| ring!(r * k, r * k, z = bz + h * t, verts = verts, at = (bx, by)))
        .collect();
    loft!(name, &sections, material = material);
    fcyl!(
        f,
        format!("{name}_lid"),
        r * 0.84,
        h * 0.12,
        (x, y + h * 0.97, z),
        lid,
        verts = verts,
        bevel = 0.002,
        bevel_segments = 1
    );
}

/// Glossy round shading for a bevelled shell: smooth over the bevels, flat on the broad faces. Its bevel is made
/// (a cut is made before, by the caller), it is shaded smooth with the edges turning more than 40° kept sharp as
/// well as those that were, and each smooth run of faces round a point takes the mean of their normals weighed by
/// their areas (Blender's weighted-normal modifier, by face area, keeping the sharp edges).
pub fn soft(obj: Obj) -> Obj {
    artkit::bevel::apply(obj);
    obj.edit(|o| {
        o.mesh.shade_smooth_by_angle(40.0);
        let normals = weighted_normals(&o.mesh);
        o.mesh.normals = Some(normals);
    });
    obj
}

/// A face's area, as Blender works it out for the weighted normals (half its Newell sum's length).
fn face_area(mesh: &Mesh, f: usize) -> f32 {
    let c = mesh.corners(f);
    let mut n = [0.0f32; 3];
    for i in 0..c.len() {
        let (a, b) = (
            mesh.verts[c[i] as usize],
            mesh.verts[c[(i + 1) % c.len()] as usize],
        );
        n[0] += a.y * b.z - a.z * b.y;
        n[1] += a.z * b.x - a.x * b.z;
        n[2] += a.x * b.y - a.y * b.x;
    }
    (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt() * 0.5
}

/// A normal a corner by Blender's weighted-normal modifier (face area, weight 50, `keep_sharp`): the corners round a
/// point that the shading joins (a fan: faces meeting across edges that are smooth, not sharp, and shared by two
/// faces running them opposite ways) share the sum of their faces' normals times their areas, largest face first,
/// made a unit vector. A fan whose sum is nothing keeps the normal it had.
pub fn weighted_normals(mesh: &Mesh) -> Vec<V3> {
    let nf = mesh.faces.len();
    let normals: Vec<V3> = (0..nf).map(|f| mesh.face_normal(f)).collect();
    let areas: Vec<f32> = (0..nf).map(|f| face_area(mesh, f)).collect();
    // each edge's faces, and whether each runs it from its lower vertex
    let mut uses: BTreeMap<(u32, u32), Vec<(usize, bool)>> = BTreeMap::new();
    for f in 0..nf {
        let c = mesh.corners(f);
        for i in 0..c.len() {
            let (a, b) = (c[i], c[(i + 1) % c.len()]);
            uses.entry(edge(a, b)).or_default().push((f, a < b));
        }
    }
    let smooth = |e: (u32, u32)| -> bool {
        let u = &uses[&e];
        u.len() == 2
            && u[0].1 != u[1].1
            && !mesh.sharp.contains(&e)
            && mesh.faces[u[0].0].smooth
            && mesh.faces[u[1].0].smooth
    };
    // the fans: each corner's root among the corners at its vertex, joined across smooth edges
    let n = mesh.loops.len();
    let mut root: Vec<usize> = (0..n).collect();
    fn find(root: &mut [usize], i: usize) -> usize {
        let mut i = i;
        while root[i] != i {
            root[i] = root[root[i]];
            i = root[i];
        }
        i
    }
    // the corners at each vertex by the vertex at the other end of each of their two edges there
    let mut by_side: BTreeMap<(u32, u32), Vec<usize>> = BTreeMap::new();
    for f in 0..nf {
        let face = mesh.faces[f];
        let c = mesh.corners(f);
        for i in 0..c.len() {
            let corner = face.start as usize + i;
            let (prev, next) = (c[(i + c.len() - 1) % c.len()], c[(i + 1) % c.len()]);
            by_side.entry((c[i], prev)).or_default().push(corner);
            by_side.entry((c[i], next)).or_default().push(corner);
        }
    }
    for (&(v, w), corners) in &by_side {
        if corners.len() == 2 && smooth(edge(v, w)) {
            let (a, b) = (find(&mut root, corners[0]), find(&mut root, corners[1]));
            if a != b {
                root[a] = b;
            }
        }
    }
    // the faces largest first (a stable sort: of two alike, the first)
    let mut order: Vec<usize> = (0..nf).collect();
    order.sort_by(|&a, &b| areas[b].partial_cmp(&areas[a]).unwrap());
    let mut sums = vec![V3::ZERO; n];
    for &f in &order {
        let face = mesh.faces[f];
        for corner in face.start as usize..(face.start + face.len) as usize {
            let r = find(&mut root, corner);
            sums[r] += normals[f] * areas[f];
        }
    }
    let had = artkit::normals::corner_normals(mesh);
    (0..n)
        .map(|corner| {
            let s = sums[find(&mut root, corner)];
            let d = s.x * s.x + s.y * s.y + s.z * s.z;
            if d > 1.0e-35 {
                let l = d.sqrt();
                V3::new(s.x / l, s.y / l, s.z / l)
            } else {
                had[corner]
            }
        })
        .collect()
}

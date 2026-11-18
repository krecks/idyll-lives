//! What the example pack is built from: its palette, the wall, the kit's parts it uses, and the oven's mouth.

use artkit::bevel::EdgeOrder;
use artkit::forms::{Sweep, rounded_rect};
use artkit::materials::Mat;
use artkit::math::{V3, pow, tan};
use artkit::mesh::{Mesh, edge};
use artkit::place::{GameFrame, Spot};
use artkit::scene::Obj;
use artkit::shapes::shade;
use artkit::{fbox, fextrude, palette};

palette! { M:
    body = ("#C3C7CB", metal),       // the stove's stainless steel; its main part
    dark = ("#2F3236", satin),       // the hob top, kick plate, control strip and the oven's enamel inside
    grate = ("#1E1F22", satin),      // cast-iron grates and burner heads
    cap = ("#4A4D52", metal),        // burner caps
    window = ("#1D2228", gloss),     // the oven windows (opaque, glossy black)
    chrome = ("#D0D3D6", chrome),    // handles, knobs' skirts, the utensil rail, the tub's jets
    knob = ("#2B2C2E", metal),       // black metal knobs
    tile = ("#ECEAE5", gloss),       // the splashback's metro tiles
    grout = ("#B9B4AA", matte),
    pot = ("#B9BDC1", metal),        // the stock pot
    copper = ("#B87333", metal),     // the hanging pan and ladle
    pan = ("#2F3236", satin),
    egg = ("#E9C46A", satin),
    crust = ("#B8763C", satin),      // the loaf baking in the oven
    flame = ("#3E7CC2", gloss),      // the gas flame (gloss, never emissive)
    shell = ("#F4F4F2", gloss),      // the tub's acrylic shell; its main part
    wood = ("#A8754A", wood),        // the tub's slatted cedar surround
    wood_dark = ("#7A5234", wood),   // the surround's posts and foot rail
    plinth = ("#3A3D41", satin),
    water = ("#6FC3CF", gloss),
    foam = ("#F2FAFA", satin),       // the bubbles
    pillow = ("#E9E6DE", satin),     // headrests
    towel = ("#FFFFFF", satin),
    towel2 = ("#9DC6CF", satin),
    candle = ("#F3EAD8", satin),
    leaf = ("#5E8F4E", satin),
    terracotta = ("#B5653F", satin),
}

/// The wall's face in a 1-deep footprint, game depth.
pub const WALL: f64 = -0.43;

// KIT: kit.tube
/// A round tube of radius `r` through game points, its bends rounded (`round` steps between each pair of
/// points; 1 keeps them sharp), its ends capped. `verts` round: by default 8, and more for one over 13 mm in
/// radius, as many as keep its outline within a millimetre of the circle (`forms::sweep`).
pub fn tube<P: Spot>(
    f: &GameFrame,
    name: &str,
    points: &[P],
    r: f64,
    material: Mat,
    verts: Option<usize>,
    round: usize,
) -> Obj {
    f.sweep(
        name,
        points,
        r,
        material,
        Sweep {
            verts,
            resolution: round,
            least: 8,
            ..Sweep::default()
        },
    )
}

// KIT: kit.fillet
/// The path through `points` (any axes) with each inner corner rounded to about `radius` (`steps` points round
/// it, on a curve that never overshoots, however uneven the spacing: Catmull-Rom loops on a short leg beside a long
/// one). For handles, bent rails and draped cloth swept with `resolution` 1.
// (3.1415 is the script's own limit for a straight corner, not pi.)
#[allow(clippy::approx_constant)]
pub fn fillet(points: &[(f64, f64, f64)], radius: f64, steps: usize) -> Vec<(f64, f64, f64)> {
    let pts: Vec<V3> = points.iter().map(|&p| V3::from(p)).collect();
    let mut out = vec![pts[0]];
    for w in pts.windows(3) {
        let (a, p, c) = (w[0], w[1], w[2]);
        let (d1, d2) = (a - p, c - p);
        if d1.length() < 1e-9 || d2.length() < 1e-9 || d1.angle(d2) > 3.1415 {
            out.push(p);
            continue;
        }
        let t = (radius / 1e-6_f64.max(tan(d1.angle(d2) / 2.0)))
            .min(d1.length() * 0.5)
            .min(d2.length() * 0.5);
        let (p1, p2) = (p + d1.normalized() * t, p + d2.normalized() * t);
        for i in 0..=steps {
            let k = i as f64 / steps as f64;
            out.push(p1 * pow(1.0 - k, 2.0) + p * 2 * k * (1.0 - k) + p2 * k * k);
        }
    }
    out.push(pts[pts.len() - 1]);
    out.iter()
        .map(|p| (p.x as f64, p.y as f64, p.z as f64))
        .collect()
}

// KIT: kit.pull
/// A bar pull in one piece: out from a front at game depth `face` on two round stand-offs `length` apart
/// (across it, or `upright`), its bends rounded: a drawer's or a door's handle.
#[allow(clippy::too_many_arguments)]
pub fn pull(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    face: f64,
    length: f64,
    material: Mat,
    r: f64,
    out: f64,
    upright: bool,
    verts: usize,
) -> Obj {
    let (h, b) = (length / 2.0, 0.012_f64.min(out * 0.45));
    let ends = [
        (-h, face - 0.003),
        (-h, face + out),
        (h, face + out),
        (h, face - 0.003),
    ];
    let pts: Vec<(f64, f64, f64)> = ends
        .iter()
        .map(|&(u, z)| {
            if upright {
                (x, y + u, z)
            } else {
                (x + u, y, z)
            }
        })
        .collect();
    f.sweep(
        name,
        &fillet(&pts, b, 3),
        r,
        material,
        Sweep {
            verts: Some(verts),
            ..Sweep::default()
        },
    )
}

// KIT: kit.shaker
/// A frame-and-panel front `w` x `h`, `t` thick, centred at game `at`, facing the front (a cabinet's door, a
/// drawer's front): a frame `rail` wide round a panel set back `recess`, flat; `panel`: the panel's own
/// material. Returns (frame, panel). (The kit's raised panel isn't used here.)
#[allow(clippy::too_many_arguments)]
pub fn shaker(
    f: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    at: (f64, f64, f64),
    t: f64,
    material: Mat,
    rail: f64,
    panel: Option<Mat>,
    recess: f64,
) -> (Obj, Obj) {
    let hole = rounded_rect(w - 2.0 * rail, h - 2.0 * rail, 0.003, 1);
    let fr = fextrude!(
        f,
        name,
        &rounded_rect(w, h, 0.003, 1),
        t,
        at,
        material,
        holes = vec![hole],
        bevel = 0.004_f64.min(t * 0.2),
        bevel_segments = 1
    );
    // (a hair inside the opening, not into the frame: a vertex buried in it would bake black)
    let (pw, ph) = (w - 2.0 * rail - 0.002, h - 2.0 * rail - 0.002);
    let pn = fbox!(
        f,
        format!("{name}_panel"),
        pw,
        ph,
        t - 2.0 * recess,
        at,
        panel.unwrap_or(material),
        bevel = 0.002,
        bevel_segments = 1
    );
    (fr, pn)
}

// KIT: kit.moulding
/// A moulding run round a closed `outline` with mitred corners: a picture or mirror frame, a panel's bead, a
/// plinth's or a cornice's band. `outline` is game (x, height) points drawn at depth `at` (`front`), or (x, depth)
/// points at height `at`; `profile` is the moulding's closed cross-section [(inset, out), ...]: `inset` metres in
/// from the outline, `out` metres forward of `at` (front) or up from it (top). The corners stay square however
/// wide the profile (a sweep pinches them).
#[allow(clippy::too_many_arguments)]
pub fn moulding(
    f: &GameFrame,
    name: &str,
    outline: &[(f64, f64)],
    profile: &[(f64, f64)],
    material: Mat,
    at: f64,
    front: bool,
    angle: f64,
) -> Obj {
    let n = outline.len();
    let area =
        artkit::py::sum((0..n).map(|i| {
            outline[i].0 * outline[(i + 1) % n].1 - outline[(i + 1) % n].0 * outline[i].1
        }));
    let turn = if area > 0.0 { 1.0 } else { -1.0 }; // inward is to the left of an anticlockwise outline
    let inward = |a: (f64, f64), b: (f64, f64)| -> (f64, f64) {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let mut length = pow(dx * dx + dy * dy, 0.5);
        if length == 0.0 {
            length = 1.0;
        }
        (-dy / length * turn, dx / length * turn)
    };
    let (mut vs, mut faces) = (Vec::new(), Vec::new());
    let m = profile.len();
    for i in 0..n {
        let p = outline[i];
        let (n0, n1) = (
            inward(outline[(i + n - 1) % n], p),
            inward(p, outline[(i + 1) % n]),
        );
        let k = 1.0 + n0.0 * n1.0 + n0.1 * n1.1;
        let (mx, my) = ((n0.0 + n1.0) / k, (n0.1 + n1.1) / k);
        for &(inset, out) in profile {
            let (u, v) = (p.0 + inset * mx, p.1 + inset * my);
            vs.push(V3::from(if front {
                f.at(u, v, at + out)
            } else {
                f.at(u, at + out, v)
            }));
        }
    }
    for i in 0..n {
        let (a, b) = (i * m, ((i + 1) % n) * m);
        for j in 0..m {
            let j2 = (j + 1) % m;
            faces.push(vec![
                (a + j) as u32,
                (a + j2) as u32,
                (b + j2) as u32,
                (b + j) as u32,
            ]);
        }
    }
    artkit::forms::object(name, vs, &faces, Some(material), true, angle, false)
}

// KIT: joinery.recessed_box (G.cut of a box out of a box)
/// What the script's boolean (`G.cut`, Blender's exact solver) makes of a box `obj` with a box `cutter` run
/// into it through its front (Blender's -y face) and not out the back or the sides: the front cut round the mouth
/// (an eight-cornered face and a quad, as Blender splits a face with a hole), the pocket's four walls and back
/// flat-shaded as the cutter's faces were, its points, faces and edges in the order Blender's solver leaves them.
/// The box's bevel, asked for before the cut, is still to come, as the modifier's was. Both unturned and unscaled;
/// the cutter goes.
pub fn cut_mouth(obj: Obj, cutter: Obj) -> Obj {
    let (loc, half) = obj.read(|o| {
        let v = &o.mesh.verts;
        let hi = v.iter().fold(V3::ZERO, |a, p| {
            V3::new(a.x.max(p.x), a.y.max(p.y), a.z.max(p.z))
        });
        (o.loc, hi)
    });
    let (cl, ch) = cutter.read(|o| {
        let v = &o.mesh.verts;
        let hi = v.iter().fold(V3::ZERO, |a, p| {
            V3::new(a.x.max(p.x), a.y.max(p.y), a.z.max(p.z))
        });
        (o.loc, hi)
    });
    cutter.remove();
    // the cutter's faces in the box's own frame (as Blender's solver works in, single precision)
    let (x0, x1) = (cl.x - ch.x - loc.x, cl.x + ch.x - loc.x);
    let back = cl.y + ch.y - loc.y;
    let (z0, z1) = (cl.z - ch.z - loc.z, cl.z + ch.z - loc.z);
    let (hx, hy, hz) = (half.x, half.y, half.z);
    let verts = vec![
        V3::new(-hx, -hy, -hz),
        V3::new(-hx, -hy, hz),
        V3::new(-hx, hy, -hz),
        V3::new(x0, back, z0),
        V3::new(-hx, hy, hz),
        V3::new(x0, back, z1),
        V3::new(hx, -hy, -hz),
        V3::new(hx, -hy, hz),
        V3::new(hx, hy, -hz),
        V3::new(x1, back, z0),
        V3::new(hx, hy, hz),
        V3::new(x1, back, z1),
        V3::new(x0, -hy, z1),
        V3::new(x1, -hy, z0),
        V3::new(x1, -hy, z1),
        V3::new(x0, -hy, z0),
    ];
    let faces: Vec<Vec<u32>> = vec![
        vec![0, 1, 4, 2],
        vec![2, 4, 10, 8],
        vec![8, 10, 7, 6],
        vec![13, 6, 7, 14],
        vec![14, 7, 1, 0, 6, 13, 15, 12],
        vec![2, 8, 6, 0],
        vec![10, 4, 1, 7],
        vec![3, 5, 12, 15],
        vec![3, 9, 11, 5],
        vec![11, 9, 13, 14],
        vec![13, 9, 3, 15],
        vec![12, 5, 11, 14],
    ];
    let edges: Vec<(u32, u32)> = vec![
        (0, 2),
        (0, 1),
        (1, 4),
        (2, 4),
        (2, 8),
        (4, 10),
        (8, 10),
        (6, 8),
        (7, 10),
        (6, 7),
        (13, 14),
        (6, 13),
        (7, 14),
        (12, 14),
        (1, 7),
        (0, 6),
        (13, 15),
        (12, 15),
        (3, 15),
        (3, 5),
        (5, 12),
        (3, 9),
        (9, 11),
        (5, 11),
        (11, 14),
        (9, 13),
    ];
    obj.edit(|o| {
        let mut mesh = Mesh::from_pydata(verts, &faces);
        for m in o.mesh.mats.clone() {
            mesh.slot(m);
        }
        o.mesh = mesh;
    });
    shade(obj, true, artkit::shapes::SHARP);
    obj.edit(|o| {
        // the box's own twelve edges stay sharp, the cut's are left smooth; the pocket's faces are flat
        o.mesh.sharp = [
            (0, 2),
            (0, 1),
            (1, 4),
            (2, 4),
            (2, 8),
            (4, 10),
            (8, 10),
            (6, 8),
            (7, 10),
            (6, 7),
            (1, 7),
            (0, 6),
        ]
        .iter()
        .map(|&(a, b)| edge(a, b))
        .collect();
        for face in &mut o.mesh.faces[7..12] {
            face.smooth = false;
        }
        if let Some(p) = o.bevel.as_mut() {
            p.order = EdgeOrder::Listed(edges);
        }
    });
    obj
}

#[cfg(test)]
mod tests {
    use super::*;
    use artkit::place::GameFrame;

    /// The stove's carcass as Blender 5.2's exact boolean left it (its points in the carcass's frame), before the
    /// bevel: captured from the script's two boxes and `G.cut`.
    const BLENDER: [[f32; 3]; 16] = [
        [-0.8, -0.29, -0.41],
        [-0.8, -0.29, 0.41],
        [-0.8, 0.29, -0.41],
        [0.09, 0.06, -0.28],
        [-0.8, 0.29, 0.41],
        [0.09, 0.06, 0.14],
        [0.8, -0.29, -0.41],
        [0.8, -0.29, 0.41],
        [0.8, 0.29, -0.41],
        [0.71, 0.06, -0.28],
        [0.8, 0.29, 0.41],
        [0.71, 0.06, 0.14],
        [0.09, -0.29, 0.14],
        [0.71, -0.29, -0.28],
        [0.71, -0.29, 0.14],
        [0.09, -0.29, -0.28],
    ];

    #[test]
    fn the_oven_mouth_is_cut_as_blender_cuts_it() {
        artkit::scene::reset();
        let f = GameFrame::new(0.0);
        let (front, wall) = (0.15, WALL);
        let carcass = fbox!(
            f,
            "carcass",
            1.6,
            0.82,
            front - wall,
            (0, 0.49, (front + wall) / 2.0),
            M.body,
            bevel = 0.01
        );
        let cutter = fbox!(
            f,
            "oven_cut",
            0.62,
            0.42,
            0.36,
            (-0.4, 0.42, front - 0.17),
            M.body
        );
        let obj = cut_mouth(carcass, cutter);
        assert!(!cutter.is_alive());
        obj.read(|o| {
            assert_eq!(o.mesh.verts.len(), 16);
            for (v, b) in o.mesh.verts.iter().zip(BLENDER) {
                for i in 0..3 {
                    assert!((v[i] - b[i]).abs() < 1e-6, "{v:?} against {b:?}");
                }
            }
            assert_eq!(o.mesh.faces.len(), 12);
            assert_eq!(o.mesh.faces.iter().filter(|f| !f.smooth).count(), 5);
            assert_eq!(o.mesh.sharp.len(), 12);
            assert!(
                o.bevel
                    .as_ref()
                    .is_some_and(|p| p.width == 0.01 && matches!(p.order, EdgeOrder::Listed(_)))
            );
        });
    }
}

//! What the example pack is built from: its palette, the wall, and the oven's mouth.

use artkit::bevel::EdgeOrder;
use artkit::math::V3;
use artkit::mesh::{Mesh, edge};
use artkit::palette;
use artkit::scene::Obj;
use artkit::shapes::shade;

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
    use artkit::fbox;
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

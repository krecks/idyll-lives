//! What the slob pack is built from: its palette, rounded boxes, the cans, chip bags and wrappers strewn about,
//! and the microwave's cavity.

use artkit::materials::Mat;
use artkit::math::{Axis, M4, P3, V3, radians};
use artkit::mesh::Mesh;
use artkit::place::{Coord, GameFrame};
use artkit::scene::Obj;
use artkit::shapes::{SHARP, shade};
use artkit::{boulder, fbox, loft, palette, revolve, ring, soft};

palette! { M:
    body = ("#D9533F", metal),       // the microwave cart's frame; see the per-model notes for each body
    bag = ("#E07A2E", satin),        // the beanbag's wipe-clean vinyl
    bag_light = ("#EB8C40", satin),
    bag_seam = ("#C8642A", satin),
    stand = ("#6E9FC0", satin),      // the nightstand
    stand_front = ("#5A8BAE", satin),
    pad = ("#F2D14E", fabric),       // the nightstand's padded top
    shelf = ("#9B59B6", satin),      // the spritz station's sides and mirror frame
    towel = ("#F7F7F7", fabric),
    towel_green = ("#6CC27A", fabric),
    laundry = ("#6E9FC0", fabric),
    laundry_red = ("#E8473B", fabric),
    cabinet = ("#4660C2", satin),    // the arcade cabinet's body
    side = ("#6A4BB5", gloss),
    marquee = ("#F2D14E", gloss, glow("#FFE08A")),   // the lit marquee: it glows at night
    screen = ("#1B2240", gloss),
    screen_lit = ("#3BA3E0", gloss, glow("#5CB8F0"), screen()),   // lit while it's played
    seat = ("#5A6680", satin),       // the recliner's leatherette
    stripe = ("#E8473B", satin),
    cyan = ("#3FD0E0", gloss),       // LED trim
    white = ("#E9E4DA", satin),
    gloss_white = ("#F2EDE2", gloss),
    panel = ("#C9CBCC", satin),
    display = ("#2E3A46", gloss),
    lcd = ("#7FE0A0", gloss),
    window = ("#3A4652", gloss),
    dark = ("#2B2C2E", satin),
    tyre = ("#3A3D41", satin),
    chrome = ("#D0D3D6", chrome),
    mirror = ("#DCE6EA", chrome),
    cardboard = ("#E9D8B4", matte),
    cookie = ("#8E5A3A", matte),
    pop_red = ("#E8473B", gloss),
    pop_orange = ("#F2A541", gloss),
    pop_blue = ("#3E8EDE", gloss),
    pop_green = ("#6CC27A", gloss),
    pop_purple = ("#9B59B6", gloss),
    pop_yellow = ("#F2D14E", gloss),
    bag_red = ("#E8473B", satin),    // chip bags and wrappers, plain
    bag_orange = ("#F2A541", satin),
    bag_green = ("#6CC27A", satin),
    bag_yellow = ("#F2D14E", satin),
    bag_purple = ("#9B59B6", satin),
}

/// The pop colours, by name.
pub const POP: [&str; 6] = [
    "pop_red",
    "pop_orange",
    "pop_blue",
    "pop_green",
    "pop_purple",
    "pop_yellow",
];
/// The wall's face in a 1-deep footprint, game depth.
pub const WALL: f64 = -0.43;

pub fn _m(name: &str) -> Mat {
    M.palette().get(name)
}

/// A rounded-cornered box lofted upright (a case, a cabinet), its top and bottom edges eased by `ease`.
#[allow(clippy::too_many_arguments)]
pub fn _box<X: Coord, Z: Coord>(
    f: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    d: f64,
    at: (X, f64, Z),
    material: Mat,
    sq: f64,
    verts: usize,
    ease: f64,
) -> Obj {
    let (x, y, z) = at;
    let b = f.at(x, y - h / 2.0, z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    let rings = [(0.0, -ease), (ease, 0.0), (h - ease, 0.0), (h, -ease)];
    let sections: Vec<Vec<P3>> = rings
        .iter()
        .map(|&(hh, e)| {
            ring!(
                w / 2.0 + e,
                d / 2.0 + e,
                z = bz + hh,
                verts = verts,
                square = sq,
                at = (bx, by)
            )
        })
        .collect();
    loft!(name, &sections, material = material)
}

/// Bakes each part's placing into its mesh, so it can be turned about the world's origin.
pub fn _bake(objs: &[Obj]) -> Vec<Obj> {
    for &o in objs {
        let m = o.matrix_world();
        o.edit(|p| p.mesh.transform(&m));
        o.set_matrix_world(M4::IDENTITY);
    }
    objs.to_vec()
}

/// Turns parts built about the origin (moved up `lift` first) `tilt` degrees about Blender X, then `yaw` about Z,
/// and moves them to `at`.
pub fn _place(objs: &[Obj], at: P3, yaw: f64, tilt: f64, lift: f64) -> Vec<Obj> {
    let m = M4::translation(V3::from(at))
        * M4::rotation(radians(yaw), Axis::Z)
        * M4::rotation(radians(tilt), Axis::X)
        * M4::translation(V3::from((0.0, 0.0, lift)));
    for o in _bake(objs) {
        o.edit(|p| p.mesh.transform(&m));
    }
    objs.to_vec()
}

/// A soda can on `y`, standing or lying turned `rot` degrees: a body necked in at both ends, a ring pull.
#[allow(clippy::too_many_arguments)]
pub fn _can(
    f: &GameFrame,
    name: &str,
    x: f64,
    z: f64,
    y: f64,
    material: Mat,
    lying: bool,
    rot: f64,
) {
    let (h, r) = (0.12, 0.033);
    let can = revolve!(
        name,
        &[
            (0.0, 0.0),
            (r * 0.8, 0.0),
            (r, 0.012),
            (r, h - 0.012),
            (r * 0.82, h - 0.002),
            (r * 0.82, h),
            (0.0, h - 0.002)
        ],
        material = material,
        segments = 12
    );
    let top = revolve!(
        format!("{name}_top"),
        &[
            (0.0, h - 0.002),
            (r * 0.8, h - 0.002),
            (r * 0.82, h + 0.002),
            (r * 0.7, h),
            (0.0, h)
        ],
        material = M.chrome,
        segments = 10
    );
    if lying {
        // turned about its middle, lying on its side
        _place(&[can, top], f.at(x, y + r, z), rot, 90.0, -h / 2.0);
    } else {
        _place(&[can, top], f.at(x, y, z), rot, 0.0, 0.0);
    }
}

/// A bag of chips: a puffed pillow, crimped flat at its top and bottom, its label band.
#[allow(clippy::too_many_arguments)]
pub fn _chip_bag(
    f: &GameFrame,
    name: &str,
    x: f64,
    z: f64,
    y: f64,
    material: Mat,
    rot: f64,
    tilt: f64,
    w: f64,
    h: f64,
) {
    let bag = soft!(
        f,
        name,
        w,
        h,
        0.06,
        (0, 0, 0),
        material,
        round = 0.02,
        puff = 0.02,
        flat = 2,
        round_segments = 1
    );
    let seals: Vec<Obj> = [-1, 1]
        .iter()
        .map(|&s| {
            fbox!(
                f,
                format!("{name}_seal{s}"),
                w,
                0.02,
                0.008,
                (0, s as f64 * (h / 2.0 - 0.006), 0),
                material
            )
        })
        .collect();
    let band = fbox!(
        f,
        format!("{name}_band"),
        w * 0.7,
        h * 0.3,
        0.064,
        (0, 0.01, 0),
        M.white
    );
    let mut parts = vec![bag, band];
    parts.extend(seals);
    _place(&parts, f.at(x, y + h / 2.0, z), -rot, tilt, 0.0);
}

/// A crumpled wrapper on the floor.
pub fn _wrapper(f: &GameFrame, name: &str, x: f64, z: f64, material: Mat, seed: i32, y: f64) {
    boulder!(
        name,
        0.04,
        at = f.at(x, y, z),
        material = material,
        scale = (1.1, 0.9, 0.45),
        seed = seed as f64,
        noise = 0.4,
        subdivisions = 1
    );
}

// KIT: joinery.recessed_box (G.cut of a box out of a loft's side)
/// What the script's boolean (`G.cut`, Blender's exact solver) makes of a lofted case `obj` with a box `cutter` run
/// into its front (Blender's -y) and not out the back: the box's top and bottom lie within one band of the loft's
/// upright quads, its sides in the band's front quads. Each quad it crosses loses the mouth (the first and last a
/// U of eight corners, those between a strip above and a strip below it, in the quad's place); the pocket's walls,
/// back, floor and ceiling come last, flat-shaded as the cutter's faces were, the floor and ceiling following the
/// case's curve along the mouth. Points, faces and their first corners are in the order Blender's solver left them
/// for the microwave's case (the cutter's four kept points among the case's at 3, 4, 8 and 10). Both unturned and
/// unscaled; the cutter goes.
pub fn cut_pocket(obj: Obj, cutter: Obj) -> Obj {
    let loc = obj.read(|o| o.loc);
    let (lo, hi) = cutter.read(|o| {
        let v = &o.mesh.verts;
        let lo = v
            .iter()
            .fold(V3::new(f32::MAX, f32::MAX, f32::MAX), |a, p| {
                V3::new(a.x.min(p.x), a.y.min(p.y), a.z.min(p.z))
            });
        let hi = v
            .iter()
            .fold(V3::new(f32::MIN, f32::MIN, f32::MIN), |a, p| {
                V3::new(a.x.max(p.x), a.y.max(p.y), a.z.max(p.z))
            });
        (o.loc + lo - loc, o.loc + hi - loc)
    });
    let cutter_mats = cutter.read(|o| o.mesh.mats.clone());
    cutter.remove();
    let (x0, x1, back, z0, z1) = (lo.x, hi.x, hi.y, lo.z, hi.z);
    obj.edit(|o| {
        let old = &o.mesh;
        let corners = |f: usize| old.corners(f).to_vec();
        // the band's front quads the mouth crosses, left to right: [lower i, lower i + 1, upper i + 1, upper i]
        let mut crossed: Vec<(usize, Vec<u32>)> = Vec::new();
        for f in 0..old.faces.len() {
            let c = corners(f);
            if c.len() != 4 {
                continue;
            }
            let p: Vec<V3> = c.iter().map(|&v| old.verts[v as usize]).collect();
            let (zl, zu) = (p[0].z.min(p[2].z), p[0].z.max(p[2].z));
            let (xa, xb) = (p[0].x.min(p[1].x), p[0].x.max(p[1].x));
            let upright = p[0].z == p[1].z && p[2].z == p[3].z;
            if upright && old.face_normal(f).y < -0.5 && zl < z0 && zu > z1 && xb > x0 && xa < x1 {
                crossed.push((f, c));
            }
        }
        crossed.sort_by(|a, b| {
            old.verts[a.1[0] as usize]
                .x
                .total_cmp(&old.verts[b.1[0] as usize].x)
        });
        let n = crossed.len();
        let on = |a: V3, b: V3, x: f32| a.y + (b.y - a.y) * (x - a.x) / (b.x - a.x);
        // the cutter's four points at the back, among the case's, as Blender's solver numbers them
        let mut verts = old.verts.clone();
        let remap = |v: u32| -> u32 {
            match v {
                0..=2 => v,
                3..=5 => v + 2,
                6 => v + 3,
                _ => v + 4,
            }
        };
        let back_pts = [
            V3::new(x0, back, z0),
            V3::new(x0, back, z1),
            V3::new(x1, back, z0),
            V3::new(x1, back, z1),
        ];
        for (at, p) in [
            (3, back_pts[0]),
            (4, back_pts[1]),
            (8, back_pts[2]),
            (10, back_pts[3]),
        ] {
            verts.insert(at, p);
        }
        let (bl, tl, br, tr) = (3u32, 4u32, 8u32, 10u32);
        // the mouth's new points: its sides where they cross the first and last quads, and the case's own points
        // between, at the floor's and the ceiling's height
        let first = &crossed[0].1;
        let last = &crossed[n - 1].1;
        let yl = on(
            old.verts[first[0] as usize],
            old.verts[first[1] as usize],
            x0,
        );
        let yr = on(old.verts[last[0] as usize], old.verts[last[1] as usize], x1);
        let mut add = |p: V3| {
            verts.push(p);
            verts.len() as u32 - 1
        };
        let mids: Vec<V3> = crossed[1..]
            .iter()
            .map(|(_, c)| old.verts[c[0] as usize])
            .collect();
        let (lb, m0t, m0b, lt) = (
            add(V3::new(x0, yl, z0)),
            add(V3::new(mids[0].x, mids[0].y, z1)),
            add(V3::new(mids[0].x, mids[0].y, z0)),
            add(V3::new(x0, yl, z1)),
        );
        let mut mb = vec![m0b];
        let mut mt = vec![m0t];
        for m in &mids[1..] {
            mb.push(add(V3::new(m.x, m.y, z0)));
            mt.push(add(V3::new(m.x, m.y, z1)));
        }
        let (rb, rt) = (add(V3::new(x1, yr, z0)), add(V3::new(x1, yr, z1)));
        let mut faces: Vec<Vec<u32>> = Vec::new();
        for f in 0..old.faces.len() {
            let c: Vec<u32> = corners(f).iter().map(|&v| remap(v)).collect();
            match crossed.iter().position(|(g, _)| *g == f) {
                None => faces.push(c),
                Some(k) => {
                    let (ll, lr, ur, ul) = (c[0], c[1], c[2], c[3]);
                    if k == 0 {
                        faces.push(vec![mt[0], ur, ul, ll, lr, mb[0], lb, lt]);
                    } else if k == n - 1 {
                        faces.push(vec![mb[k - 1], ll, lr, ur, ul, mt[k - 1], rt, rb]);
                    } else {
                        faces.push(vec![mt[k], ur, ul, mt[k - 1]]);
                        faces.push(vec![mb[k - 1], ll, lr, mb[k]]);
                    }
                }
            }
        }
        let mut bottom = vec![rb, br, bl, lb];
        bottom.extend(&mb);
        let mut top: Vec<u32> = vec![lt, tl, tr, rt];
        top.extend(mt.iter().rev());
        // (Blender's ceiling starts at the second of the case's points along the mouth)
        let start = top.len() - 2;
        top.rotate_left(start);
        for face in [
            vec![bl, tl, lt, lb],
            vec![bl, br, tr, tl],
            vec![tr, br, rb, rt],
            bottom,
            top,
        ] {
            faces.push(face);
        }
        // (Blender's solver starts the case's bottom five corners before the end it had, and its top at its last
        // corner: the caps' triangles follow from where they start)
        for face in faces.iter_mut().filter(|c| c.len() > 8) {
            let low = face.iter().all(|&v| verts[v as usize].z <= z0);
            let k = face.len() - if low { 5 } else { 1 };
            face.rotate_left(k);
        }
        let mut mesh = Mesh::from_pydata(verts, &faces);
        for m in old.mats.clone() {
            mesh.slot(m);
        }
        // (the cutter's material joins the case's, worn by none of its faces, as the solver adds it)
        for m in cutter_mats.clone() {
            mesh.slot(m);
        }
        o.mesh = mesh;
    });
    shade(obj, true, SHARP);
    let count = obj.read(|o| o.mesh.faces.len());
    obj.edit(|o| {
        for face in &mut o.mesh.faces[count - 5..] {
            face.smooth = false;
        }
    });
    obj
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_microwave_cavity_is_cut_as_blender_cuts_it() {
        artkit::scene::reset();
        let f = GameFrame::new(0.0);
        // the cart's microwave case and its cutter, as microwave_cart makes them
        let (x, y, z0, w, h, d, pw) = (-0.02, 0.815, -0.42, 0.44, 0.29, 0.36, 0.12);
        let zf = z0 + d;
        let case = _box(
            &f,
            "mw_case",
            w,
            h - 0.012,
            d - 0.01,
            (x, y + 0.006 + (h - 0.012) / 2.0, z0 + (d - 0.01) / 2.0),
            M.gloss_white,
            10.0,
            24,
            0.01,
        );
        let cav = (w - pw - 0.06, h - 0.07, d - 0.08);
        let cutter = fbox!(
            f,
            "mw_cut",
            cav.0,
            cav.1,
            cav.2 + 0.02,
            (x - pw / 2.0 - 0.01, y + h / 2.0, zf - cav.2 / 2.0),
            M.dark
        );
        let obj = cut_pocket(case, cutter);
        obj.read(|o| {
            let m = &o.mesh;
            assert_eq!((m.verts.len(), m.faces.len()), (110, 81));
            // Blender 5.2's exact boolean on the same two parts: the cutter's points kept at the back, the mouth's
            // new points, and the faces round the mouth, corner for corner
            for (i, p) in [
                (3, [-0.04, 0.35, 0.85]),
                (10, [0.22, 0.35, 1.07]),
                (100, [-0.04, 0.07043, 0.85]),
                (101, [0.01984, 0.07, 1.07]),
                (108, [0.22, 0.07913, 0.85]),
                (109, [0.22, 0.07913, 1.07]),
            ] {
                for k in 0..3 {
                    assert!(
                        (m.verts[i][k] - p[k]).abs() < 1e-5,
                        "point {i}: {:?}",
                        m.verts[i]
                    );
                }
            }
            let face = |c: &[u32]| (0..m.faces.len()).any(|f| m.corners(f) == c);
            for c in [
                &[101, 70, 69, 45, 46, 102, 100, 103][..],
                &[105, 71, 70, 101],
                &[102, 46, 47, 104],
                &[106, 48, 49, 73, 72, 107, 109, 108],
                &[3, 4, 103, 100],
                &[108, 8, 3, 100, 102, 104, 106],
                &[105, 101, 103, 4, 10, 109, 107],
                &[
                    6, 5, 2, 1, 0, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12,
                    11, 9, 7,
                ],
            ] {
                assert!(face(c), "no face {c:?}");
            }
            assert_eq!(m.faces.iter().filter(|f| !f.smooth).count(), 5);
            assert!(m.mats.iter().any(|m| m.name == "dark"));
        });
    }
}

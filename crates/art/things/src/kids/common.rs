//! What the kids' things are built from: the helpers kids.py shared between its models, and the kit's parts it
//! used (marked `KIT:` until the kernel has them).

use super::{F, M};
use artkit::forms::{Extrude, rounded_rect};
use artkit::materials::Mat;
use artkit::math::{Axis, P3, TAU, V3, cos, pow, radians, sin, v3};
use artkit::mesh::Mesh;
use artkit::motion::{Squash, squash};
use artkit::noise::noise;
use artkit::parts::{Settle, hinged, join, measure, new_parts, settle_with};
use artkit::place::Coord;
use artkit::py::sum;
use artkit::scene::{self, Obj, meshes};
use artkit::{jitter, revolve, soft, swing};

/// A game point (x, height, depth).
pub type P = (f64, f64, f64);

/// `_add`: the points summed, a coordinate at a time.
pub fn add(vs: &[P]) -> P {
    (
        sum(vs.iter().map(|v| v.0)),
        sum(vs.iter().map(|v| v.1)),
        sum(vs.iter().map(|v| v.2)),
    )
}

pub fn mul(v: P, k: f64) -> P {
    (v.0 * k, v.1 * k, v.2 * k)
}

/// A flat-shaded mesh from game points and faces, its normals made to face out.
pub fn mesh(name: &str, verts: &[P], faces: &[Vec<u32>], material: Mat) -> Obj {
    let vs: Vec<V3> = verts
        .iter()
        .map(|&(x, y, z)| F.at(x, y, z).into())
        .collect();
    let mut m = Mesh::from_pydata(vs, faces);
    m.set_material(Some(material));
    m.recalc_normals();
    scene::add(name, m)
}

/// A frame on a flat face: its point `o`, the across `s` and up `u` directions (game, unit), and its outward
/// normal `n`.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub o: P,
    pub s: P,
    pub u: P,
    pub n: P,
}

pub fn plane(origin: P, s: P, u: P) -> Frame {
    let n = (
        s.1 * u.2 - s.2 * u.1,
        s.2 * u.0 - s.0 * u.2,
        s.0 * u.1 - s.1 * u.0,
    );
    Frame { o: origin, s, u, n }
}

/// The frame on the front face of a board leaning back by `tilt` degrees, centred on `centre`.
pub fn board(centre: P, tilt: f64) -> Frame {
    let t = radians(tilt);
    plane(centre, (1.0, 0.0, 0.0), (0.0, cos(t), -sin(t)))
}

/// A flat piece: the convex outline `pts` [(across, up)] on `frame`, `out` in front of it, `thick` thick.
pub fn slab(
    name: &str,
    pts: &[(f64, f64)],
    frame: &Frame,
    material: Mat,
    out: f64,
    thick: f64,
) -> Obj {
    let at =
        |a: f64, b: f64, c: f64| add(&[frame.o, mul(frame.s, a), mul(frame.u, b), mul(frame.n, c)]);
    let k = pts.len() as u32;
    let mut verts: Vec<P> = pts.iter().map(|&(a, b)| at(a, b, out)).collect();
    verts.extend(pts.iter().map(|&(a, b)| at(a, b, out + thick)));
    let mut faces: Vec<Vec<u32>> = vec![(0..k).collect(), (k..2 * k).rev().collect()];
    faces.extend((0..k).map(|i| vec![i, (i + 1) % k, k + (i + 1) % k, k + i]));
    mesh(name, &verts, &faces, material)
}

/// `n` + 1 points on a circle of radius `r` round (cx, cy), from `a0` to `a1` degrees.
pub fn arc(r: f64, a0: f64, a1: f64, n: usize, cx: f64, cy: f64) -> Vec<(f64, f64)> {
    (0..=n)
        .map(|i| {
            let a = radians(a0 + (a1 - a0) * i as f64 / n as f64);
            (cx + r * cos(a), cy + r * sin(a))
        })
        .collect()
}

/// Stands the model on the floor where it was built (over where it stood) and says its centre; `forward` (game
/// depth, m) moves it towards the front first: its back onto the wall face.
pub fn place(forward: f64) {
    for o in meshes() {
        o.edit(|p| p.loc.y = (p.loc.y as f64 - forward) as f32);
    }
    let (lo, hi) = measure(&meshes());
    scene::log(format!(
        "[kids] centre (game x, depth) {:.4}, {:.4}",
        -(lo[0] + hi[0]) / 2.0,
        -(lo[1] + hi[1]) / 2.0
    ));
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
}

/// Joins what's in the scene (or `parts`) as the still body, its origin on the floor.
pub fn still(name: &str, parts: Option<&[Obj]>) -> Obj {
    match parts {
        Some(parts) => join(parts, name, Some(P3::ZERO)),
        None => join(&meshes(), name, Some(P3::ZERO)),
    }
}

/// A star's outline round (cx, cy), its points `r` out and its notches `inner` of that, a point up.
pub fn star(cx: f64, cy: f64, r: f64, inner: f64, n: usize, turn: f64) -> Vec<(f64, f64)> {
    (0..2 * n)
        .map(|i| {
            let k = if i % 2 == 0 { 1.0 } else { inner };
            let a = radians(turn + 180.0 * i as f64 / n as f64);
            (cx + r * k * cos(a), cy + r * k * sin(a))
        })
        .collect()
}

/// A heart's outline round (cx, cy), about `r` across from its middle.
pub fn heart(cx: f64, cy: f64, r: f64, n: usize) -> Vec<(f64, f64)> {
    let mut pts = Vec::new();
    for i in 0..n {
        let t = TAU * i as f64 / n as f64;
        let x = 16.0 * pow(sin(t), 3.0);
        let y = 13.0 * cos(t) - 5.0 * cos(2.0 * t) - 2.0 * cos(3.0 * t) - cos(4.0 * t);
        pts.push((cx + r * x / 16.0, cy + r * (y + 2.0) / 16.0));
    }
    pts.reverse();
    pts
}

/// A crescent moon's outline round (cx, cy): an outer arc and a narrower inner one, its horns to the right.
pub fn moon(cx: f64, cy: f64, r: f64, n: usize) -> Vec<(f64, f64)> {
    let mut outer = arc(r, 70.0, 290.0, n, cx, cy);
    let inner: Vec<(f64, f64)> = (0..=n)
        .map(|i| 255.0 - 150.0 * i as f64 / n as f64)
        .map(|a| {
            (
                cx + r * 0.42 + r * 0.82 * cos(radians(a)),
                cy + r * 0.82 * sin(radians(a)),
            )
        })
        .collect();
    outer.extend_from_slice(&inner[1..inner.len() - 1]);
    outer
}

/// A turned part: `profile` [(radius, height over y)] revolved about the upright through game (x, z).
pub fn turned(
    name: &str,
    x: impl Coord,
    z: impl Coord,
    profile: &[(f64, f64)],
    material: Mat,
    segments: usize,
    y: f64,
) -> Obj {
    revolve!(
        name,
        profile,
        at = F.at(x, y, z),
        material = material,
        segments = segments
    )
}

/// Rumples a cover's top: its vertices above height `above` moved up or down by smooth noise, up to `amount`,
/// never sideways, so its bounds across stay put (as bedroom.py's).
pub fn rumple(obj: Obj, amount: f64, above: f64, seed: f64, size: f64) -> Obj {
    let world = obj.world_points();
    obj.edit(|o| {
        for (v, p) in o.mesh.verts.iter_mut().zip(&world) {
            if p.z as f64 > above {
                let n = noise(v3(p.x as f64 / size + seed, p.y as f64 / size - seed, seed));
                v.z = (v.z as f64 + n * amount) as f32;
            }
        }
    });
    obj
}

/// A mattress `w` x `d`, `h` deep, the middle of its top at `top` (where the game lays the sleeper): rounded,
/// a hair puffed, a piped seam round its top and bottom.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn mattress(
    name: &str,
    w: f64,
    h: f64,
    d: f64,
    top: f64,
    z: f64,
    x: f64,
    material: Option<Mat>,
    puff: f64,
    piping: f64,
    round: f64,
    flat: usize,
) -> Obj {
    // (firm, its edges taped: no pinched corners)
    soft!(
        F,
        name,
        w,
        h - 2.0 * puff,
        d,
        (x, top - h / 2.0, z),
        material.unwrap_or(M.mattress),
        round = round,
        puff = puff,
        flat = flat,
        piping = piping,
        pinch = 0.0,
        resting = true
    )
}

/// A pillow on `top`, `h` thick in the middle: plump, thinning to its seams, its corners pinched in, a little
/// crumpled (as bedroom.py's).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn soft_pillow(
    name: &str,
    w: f64,
    h: f64,
    d: f64,
    x: impl Coord,
    top: f64,
    z: f64,
    material: Mat,
    tilt: f64,
    seed: f64,
) -> Obj {
    let p = soft!(
        F,
        name,
        w,
        h * 0.34,
        d,
        (x, top + h / 2.0 - 0.01, z),
        material,
        rot = (tilt, 0, 0),
        round = (h * 0.17).min(0.03),
        puff = h * 0.33,
        pinch = h * 0.26,
        waist = h * 0.04,
        resting = true
    );
    jitter!(p, 0.003, size = 0.12, seed = seed + x.f() * 7.0, up = 0.4)
}

// --- beds: a duvet whose top third turns down (as bedroom.py) -------------------------------------------
/// The duvet's top above the mattress top.
pub const DUVET: f64 = 0.05;
/// The flap's thickness.
pub const FLAP: f64 = 0.045;

/// The duvet's lower two thirds, from the fold to the foot, draped down the sides and the foot end, puffed on
/// top (its middle at `DUVET`) and rumpled as a duvet shaken out lies (the soft form's `rumple`, as bedroom.py's:
/// folds that only go down and in, so it stays under the game's quilt).
pub fn duvet(
    top: f64,
    half: f64,
    z_fold: f64,
    z_foot: f64,
    drape: f64,
    material: Mat,
    seed: f64,
) -> Obj {
    let (over, puff) = (0.02, 0.01);
    let (y0, y1) = (top - drape, top + DUVET);
    soft!(
        F,
        "duvet",
        2.0 * (half + over),
        y1 - y0 - 2.0 * puff,
        z_foot + over - z_fold,
        (0, (y0 + y1) / 2.0, (z_fold + z_foot + over) / 2.0),
        material,
        round = 0.05,
        puff = puff,
        pinch = 0.0,
        rumple = 0.025,
        rumple_size = 0.24,
        seed = seed + half,
        resting = true
    )
}

/// The duvet's top third with the sheet turned down over its head edge, hinged across its fold.
pub fn flap(top: f64, half: f64, z_top: f64, z_fold: f64, material: Mat) -> Obj {
    let parts = new_parts(|| {
        let y = top + DUVET - FLAP / 2.0;
        soft!(
            F,
            "flap",
            2.0 * (half + 0.012),
            FLAP - 0.01,
            z_fold - z_top,
            (0, y, (z_top + z_fold) / 2.0),
            material,
            round = 0.018,
            puff = 0.005,
            flat = 3
        );
        soft!(
            F,
            "flap_sheet",
            2.0 * (half + 0.016),
            FLAP + 0.008,
            0.12,
            (0, y, z_top + 0.05),
            M.sheet,
            round = 0.02
        );
    });
    hinged(
        join(&parts, "duvet_flap", None),
        F.at(0, top + DUVET + 0.003, z_fold),
    )
}

/// Builds the duvet (its still two thirds and the flap), joins the bed, settles, keys `turn_down`.
pub fn turn_down(
    top: f64,
    half: f64,
    z_top: f64,
    z_foot: f64,
    drape: f64,
    material: Mat,
) -> Vec<Obj> {
    let z_fold = z_top + (z_foot - z_top) / 3.0;
    duvet(top, half, z_fold, z_foot, drape, material, 0.0);
    still("bed", None);
    let flap = flap(top, half, z_top, z_fold, material);
    place(0.0);
    // As it turns it flattens and narrows to stay under the game's quilt (0.46 m from the bed's middle,
    // never lower than 8.5 cm over the mattress), as the bedroom family's beds do (bedroom's `_turn_down`).
    swing!(flap, "turn_down", 180.0, seconds = 1.0, axis = Axis::X);
    let narrow = (0.46 - 0.08) / (half + 0.016);
    squash(
        flap,
        "turn_down",
        (if narrow < 1.0 { narrow } else { 1.0 }, 1, 0.4),
        Squash {
            seconds: 1.0,
            ..Squash::default()
        },
    );
    // half way over it's gathered to a fifth of its length, so it rolls over as cloth, not a board standing up
    squash(
        flap,
        "turn_down",
        (1, 0.2, 1),
        Squash {
            seconds: 1.0,
            back: true,
            ..Squash::default()
        },
    );
    meshes()
}

/// A little teddy sitting at (x, y, z) (its seat), facing the front: a pear of a body, a head with a snout,
/// round ears, stubby arms and legs, bead eyes and nose.
pub fn teddy(x: f64, y: f64, z: f64, material: Mat, eye: Mat, r: f64) {
    use artkit::{fsphere, fsweep};
    fsphere!(
        F,
        "teddy_body",
        r,
        (x, y + r * 0.95, z),
        material,
        scale = (0.95, 0.85, 1.1),
        segments = 12,
        rings = 8
    );
    fsphere!(
        F,
        "teddy_head",
        r * 0.78,
        (x, y + r * 2.6, z + r * 0.05),
        material,
        segments = 12,
        rings = 8
    );
    fsphere!(
        F,
        "teddy_snout",
        r * 0.32,
        (x, y + r * 2.45, z + r * 0.75),
        material,
        scale = (1.1, 1, 0.8),
        segments = 8,
        rings = 6
    );
    fsphere!(
        F,
        "teddy_nose",
        r * 0.12,
        (x, y + r * 2.55, z + r * 1.0),
        eye,
        segments = 6,
        rings = 4
    );
    for s in [-1, 1] {
        let sf = s as f64;
        fsphere!(
            F,
            format!("teddy_ear{s}"),
            r * 0.28,
            (x + sf * r * 0.58, y + r * 3.2, z),
            material,
            scale = (1, 1, 0.6),
            segments = 8,
            rings = 6
        );
        fsphere!(
            F,
            format!("teddy_eye{s}"),
            r * 0.09,
            (x + sf * r * 0.3, y + r * 2.8, z + r * 0.68),
            eye,
            segments = 6,
            rings = 4
        );
        fsweep!(
            F,
            format!("teddy_arm{s}"),
            &[
                (x + sf * r * 0.7, y + r * 1.5, z),
                (x + sf * r * 0.95, y + r * 0.9, z + r * 0.5)
            ],
            r * 0.26,
            material,
            verts = 8,
            scale = [1.0, 0.9]
        );
        fsweep!(
            F,
            format!("teddy_leg{s}"),
            &[
                (x + sf * r * 0.45, y + r * 0.35, z + r * 0.2),
                (x + sf * r * 0.55, y + r * 0.3, z + r * 1.2)
            ],
            r * 0.32,
            material,
            verts = 8,
            scale = [1.0, 1.05]
        );
    }
}

pub fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Vec<(f64, f64)> {
    vec![(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
}

/// An open box's four walls in one piece, `t` thick, w x d outside, their corners rounded, from y0 up to y1.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn walls(
    name: &str,
    w: f64,
    d: f64,
    y0: f64,
    y1: f64,
    z: impl Coord,
    t: f64,
    material: Mat,
    r: f64,
    x: f64,
) -> Obj {
    use artkit::place::Facing;
    let out = rounded_rect(w, d, r, 3);
    let hole = rounded_rect(w - 2.0 * t, d - 2.0 * t, (r - t).max(0.004), 3);
    let o = Extrude {
        holes: artkit::forms::Holes(vec![hole]),
        plane: artkit::Arg::arg(Facing::Top),
        ..Extrude::default()
    };
    F.extrude(name, &out, y1 - y0, (x, (y0 + y1) / 2.0, z), material, o)
}

/// A toy block `s` across standing on its bottom at height `y`: rounded, a raised square on its faces.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn block(
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    s: f64,
    material: Mat,
    face: Mat,
    turn: f64,
    tip: f64,
) -> (Obj, Obj) {
    use artkit::fbox;
    let cube = fbox!(
        F,
        name,
        s,
        s,
        s,
        (x, y + s / 2.0, z),
        material,
        rot = (tip, 0, turn),
        bevel = s * 0.12,
        bevel_segments = 2
    );
    let face_obj = fbox!(
        F,
        format!("{name}_face"),
        s * 0.62,
        s * 0.62,
        s * 1.04,
        (x, y + s / 2.0, z),
        face,
        rot = (tip, 0, turn),
        bevel = s * 0.05,
        bevel_segments = 1
    );
    (cube, face_obj)
}

// --- the cut through the toy box's ends ------------------------------------------------------------------

/// What `G.cut` made of the toy box's walls and a grip through an end, built without a boolean: `cutter`, a slab drawn
/// on the side (its outline in Blender's y and z, thick along x), runs square through the faces of `obj` that lie
/// across it and hold its outline whole; each of those gets the outline as a hole, the hole's sides run between
/// them, and the cutter goes. For a part placed without a turn or a scale.
pub fn cut_through(obj: Obj, cutter: Obj) -> Obj {
    let points = cutter.world_points();
    let n = points.len() / 2;
    let (x0, x1) = points
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), p| {
            (lo.min(p.x), hi.max(p.x))
        });
    let loc = obj.location();
    let ring: Vec<[f32; 2]> = points[..n]
        .iter()
        .map(|p| [p.y - loc.y, p.z - loc.z])
        .collect();
    let centre = ring.iter().fold([0.0f32; 2], |a, p| {
        [a[0] + p[0] / n as f32, a[1] + p[1] / n as f32]
    });
    obj.edit(|o| {
        let old = &o.mesh;
        let mut verts = old.verts.clone();
        let mut faces: Vec<Vec<u32>> = Vec::new();
        let mut pierced: Vec<(f32, Vec<u32>)> = Vec::new();
        for f in 0..old.faces.len() {
            let c = old.corners(f).to_vec();
            let nrm = old.face_normal(f);
            let xs: Vec<f32> = c.iter().map(|&v| old.verts[v as usize].x + loc.x).collect();
            let face2d: Vec<[f32; 2]> = c
                .iter()
                .map(|&v| [old.verts[v as usize].y, old.verts[v as usize].z])
                .collect();
            let across = nrm.x.abs() > 0.9999 && xs.iter().all(|&x| x > x0 && x < x1);
            if !(across && ring.iter().all(|&p| inside(p, &face2d))) {
                faces.push(c);
                continue;
            }
            // the face with the hole in it, its triangles as the slabs' faces are cut (tri::fill), turned its way
            let x = old.verts[c[0] as usize].x;
            let start = verts.len() as u32;
            verts.extend(ring.iter().map(|p| V3::new(x, p[0], p[1])));
            let at = |i: u32| {
                if (i as usize) < c.len() {
                    c[i as usize]
                } else {
                    start + i - c.len() as u32
                }
            };
            let all: Vec<[f32; 2]> = face2d.iter().chain(&ring).copied().collect();
            for t in artkit::tri::fill(&[face2d.clone(), ring.clone()]) {
                let (a, b, d) = (all[t[0] as usize], all[t[1] as usize], all[t[2] as usize]);
                let mid = [(a[0] + b[0] + d[0]) / 3.0, (a[1] + b[1] + d[1]) / 3.0];
                if inside(mid, &ring) {
                    continue;
                }
                let way = (b[0] - a[0]) * (d[1] - a[1]) - (b[1] - a[1]) * (d[0] - a[0]);
                if (way > 0.0) == (nrm.x > 0.0) {
                    faces.push(vec![at(t[0]), at(t[1]), at(t[2])]);
                } else {
                    faces.push(vec![at(t[0]), at(t[2]), at(t[1])]);
                }
            }
            pierced.push((x, (start..start + n as u32).collect()));
        }
        // the hole's sides, between each face it enters by and the one it leaves by, looking into the hole
        pierced.sort_by(|a, b| a.0.total_cmp(&b.0));
        for pair in pierced.chunks(2) {
            let [(_, a), (_, b)] = pair else { continue };
            for i in 0..n {
                let j = (i + 1) % n;
                let quad = vec![a[i], a[j], b[j], b[i]];
                let p: Vec<V3> = quad.iter().map(|&v| verts[v as usize]).collect();
                let normal = artkit::mesh::normal_newell(&p);
                let mid = [(p[0].y + p[1].y) / 2.0, (p[0].z + p[1].z) / 2.0];
                let to_axis = (centre[0] - mid[0]) * normal.y + (centre[1] - mid[1]) * normal.z;
                faces.push(if to_axis > 0.0 {
                    quad
                } else {
                    vec![a[i], b[i], b[j], a[j]]
                });
            }
        }
        let mats = old.mats.clone();
        let mut new = Mesh::from_pydata(verts, &faces);
        for m in mats {
            new.slot(m);
        }
        o.mesh = new;
    });
    cutter.remove();
    obj
}

/// Whether the point is inside the closed outline (crossings of a ray along +u).
fn inside(p: [f32; 2], outline: &[[f32; 2]]) -> bool {
    let mut odd = false;
    let n = outline.len();
    for i in 0..n {
        let (a, b) = (outline[i], outline[(i + n - 1) % n]);
        if (a[1] > p[1]) != (b[1] > p[1])
            && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0]
        {
            odd = !odd;
        }
    }
    odd
}

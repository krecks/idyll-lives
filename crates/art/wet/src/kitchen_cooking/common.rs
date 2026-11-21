//! What the cookers and the small appliances are built from: parts every model here shares.

use super::M;
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{Axis, PI, TAU, V3, cos, pow, sin, sqrt};
use artkit::mesh::Mesh;
use artkit::motion::{Grow, grow};
use artkit::parts::{join, new_parts};
use artkit::place::{Coord, GameFrame};
use artkit::py;
use artkit::scene::Obj;
use artkit::{fbox, fcone, fcyl, fsweep, revolve, torus};

/// A polyline's corners rounded by arcs about `r` long each way (a quadratic curve through `steps` - 1 more
/// points), for `kit.tube(..., round=1)`: bent tube whose long straights don't make the curve overshoot its corners,
/// as the Catmull-Rom rounding does between a long span and a short one.
pub fn fillet(points: &[(f64, f64, f64)], r: f64, steps: usize) -> Vec<(f64, f64, f64)> {
    let mut out = vec![points[0]];
    for w in points.windows(3) {
        let (a, p, b) = (w[0], w[1], w[2]);
        let (a, p, b) = ([a.0, a.1, a.2], [p.0, p.1, p.2], [b.0, b.1, b.2]);
        let da: Vec<f64> = (0..3).map(|i| p[i] - a[i]).collect();
        let db: Vec<f64> = (0..3).map(|i| b[i] - p[i]).collect();
        let (la, lb) = (
            sqrt(py::sum(da.iter().map(|c| c * c))),
            sqrt(py::sum(db.iter().map(|c| c * c))),
        );
        let k = py::min(py::min(r, la / 2.0), lb / 2.0);
        let s: Vec<f64> = (0..3).map(|i| p[i] - da[i] / la * k).collect();
        let e: Vec<f64> = (0..3).map(|i| p[i] + db[i] / lb * k).collect();
        for j in 0..=steps {
            let t = j as f64 / steps as f64;
            let q: Vec<f64> = (0..3)
                .map(|i| pow(1.0 - t, 2.0) * s[i] + 2.0 * (1.0 - t) * t * p[i] + t * t * e[i])
                .collect();
            out.push((q[0], q[1], q[2]));
        }
    }
    out.push(points[points.len() - 1]);
    out
}

/// A control knob facing the front, `d` deep about game depth `z`: a skirt on the panel, a grip and a dished
/// cap, with a pointer line so a turn shows.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn knob(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    d: f64,
    material: Option<Mat>,
    mark: bool,
) {
    let prof = [
        (0.0, 0.0),
        (r * 1.2, 0.0),
        (r * 0.9, d * 0.3),
        (r * 0.88, d * 0.92),
        (r * 0.74, d),
        (0.0, d * 0.95),
    ];
    revolve!(
        name,
        &prof,
        at = f.at(x, y, z - d / 2.0),
        material = material.unwrap_or(M.chrome),
        segments = 10,
        rot = (90, 0, 0)
    );
    if mark {
        fbox!(
            f,
            format!("{name}_mark"),
            0.004,
            r * 0.75,
            0.004,
            (x, y + r * 0.42, z + d / 2.0 - 0.0005),
            M.dark
        );
    }
}

/// A knob as its own object, its origin at its centre, so a clip turns it in place.
pub fn knob_obj(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, r: f64, d: f64) -> Obj {
    let parts = new_parts(|| knob(f, name, x, y, z, r, d, None, true));
    join(&parts, name, Some(f.at(x, y, z)))
}

/// A gas flame: a ring of blue tongues round the burner, built full size but scaled to nothing, grown up in every
/// axis while the burner is on (the `water_on` way, so it is hidden at rest).
pub fn flame(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, r: f64) -> Obj {
    let parts = new_parts(|| {
        torus!(
            name,
            r,
            0.009,
            at = f.at(x, y, z),
            material = M.flame,
            major_segments = 14,
            minor_segments = 5
        );
        for i in 0..12 {
            let a = TAU * i as f64 / 12.0;
            fcone!(
                f,
                format!("{name}_tongue{i}"),
                0.008,
                0.0,
                0.03,
                (
                    x + (r + 0.004) * cos(a),
                    y + 0.016,
                    z + (r + 0.004) * sin(a)
                ),
                M.flame,
                verts = 5
            );
        }
    });
    let obj = join(&parts, name, Some(f.at(x, y, z)));
    obj.set_scale((0.001, 0.001, 0.001))
}

/// Grows a flame ring in all three axes, so `burner_on` shows it from nothing.
pub fn light(flame: Obj) {
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        grow(flame, "burner_on", Grow { seconds: 0.5, axis });
    }
}

/// A gas burner on a hob at height `y`: a spreader bowl, a black cap, and a cast-iron pan support (a square of bar
/// with four fingers reaching in), its top 2 cm up, where a pan stands.
pub fn burner(f: &GameFrame, name: &str, x: f64, z: f64, y: f64, r: f64) {
    let b = f.at(x, y - 0.01, z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    revolve!(
        format!("{name}_head"),
        &[
            (r * 0.85, 0.0),
            (r * 0.85, 0.008),
            (r * 0.62, 0.016),
            (0.0, 0.016)
        ],
        at = (bx, by, bz),
        material = M.grate,
        segments = 10
    );
    revolve!(
        format!("{name}_cap"),
        &[
            (r * 0.6, 0.0),
            (r * 0.6, 0.004),
            (r * 0.5, 0.008),
            (0.0, 0.009)
        ],
        at = (bx, by, bz + 0.016),
        material = M.pan,
        segments = 10
    );
    let side = 2.0 * r + 0.06;
    let lp = rounded_rect(side, side, 0.02, 1);
    let ring: Vec<(f64, f64, f64)> = lp.iter().map(|&(u, v)| (x + u, y + 0.016, z + v)).collect();
    fsweep!(
        f,
        format!("{name}_grate"),
        &ring,
        0.005,
        M.grate,
        verts = 4,
        closed = true
    );
    for i in 0..4 {
        let a = TAU * i as f64 / 4.0 + PI / 4.0;
        let (c, s) = (cos(a), sin(a));
        let reach = side / 2.0 * 1.38;
        fsweep!(
            f,
            format!("{name}_finger{i}"),
            &[
                (x + c * reach * 0.98, y + 0.016, z + s * reach * 0.98),
                (x + c * r * 0.72, y + 0.016, z + s * r * 0.72)
            ],
            0.005,
            M.grate,
            verts = 4
        );
    }
}

/// A ceramic ring glowing red: a thin disc on the glass, hidden at rest (scaled to nothing) and grown
/// by `light` while the ring is on, as a gas flame is.
pub fn hot_ring(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, r: f64) -> Obj {
    let parts = new_parts(|| {
        fcyl!(f, name, r, 0.003, (x, y, z), M.heat, verts = 20);
    });
    let obj = join(&parts, name, Some(f.at(x, y, z)));
    obj.set_scale((0.001, 0.001, 0.001))
}

/// A cooking zone printed on a ceramic/induction hob: a ring (and a smaller one inside it) on the glass.
pub fn hob_ring(f: &GameFrame, name: &str, x: f64, z: f64, y: f64, r: f64, inner: bool) {
    torus!(
        name,
        r,
        0.0035,
        at = f.at(x, y, z),
        material = M.grate,
        major_segments = 20,
        minor_segments = 4
    );
    if inner {
        torus!(
            format!("{name}_in"),
            r * 0.55,
            0.003,
            at = f.at(x, y, z),
            material = M.grate,
            major_segments = 14,
            minor_segments = 4
        );
    }
}

/// Cuts an oven's cavity out of `obj` (a carcass box) through its front: a w × h × d box at game `at`, its walls
/// taking the cavity's dark enamel (or `material`).
///
/// The cut is made as Blender's exact boolean made it, without one: the carcass's face the cavity opens through is
/// split round the opening (an eight-sided face and a quad beside it, or, where the opening's lower edge lies on the
/// face's own, one face with a notch and the face below given the opening's two corners), and five flat walls line
/// the cavity, its points numbered among the carcass's as Blender numbered them.
///
/// Not `artkit::kit::cut`, on purpose: that one numbers the points a cut makes and starts its faces in its own
/// order, and the baked shading, which hangs on both, then lies farther off Blender's on the microwave and the
/// gas range.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn hollow(
    obj: Obj,
    f: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    d: f64,
    at: (impl Coord, f64, f64),
    material: Option<Mat>,
) -> Obj {
    let cutter = fbox!(f, name, w, h, d, at, material.unwrap_or(M.dark));
    let mat = material.unwrap_or(M.dark);
    // the cutter's corners in the carcass's own frame
    artkit::bevel::apply(obj);
    let inv = obj
        .matrix_world()
        .inverted()
        .expect("a carcass is not flattened");
    let corners: Vec<V3> = cutter.world_points().iter().map(|&p| inv * p).collect();
    cutter.remove();
    let lo = |i: usize| corners.iter().map(|c| c[i]).fold(f32::INFINITY, f32::min);
    let hi = |i: usize| {
        corners
            .iter()
            .map(|c| c[i])
            .fold(f32::NEG_INFINITY, f32::max)
    };
    let (x0, x1, back, z0, z1) = (lo(0), hi(0), hi(1), lo(2), hi(2));
    obj.edit(|o| cut(&mut o.mesh, mat, x0, x1, back, z0, z1));
    obj
}

/// The cavity cut into `mesh` through its front (its face looking down -y), as `hollow` says: x0..x1 across, z0..z1
/// up, back to y `back`.
fn cut(mesh: &mut Mesh, mat: Mat, x0: f32, x1: f32, back: f32, z0: f32, z1: f32) {
    let slot = mesh.slot(mat);
    let front_y = mesh.verts.iter().map(|v| v.y).fold(f32::INFINITY, f32::min);
    let flat = |f: usize, m: &Mesh| {
        m.corners(f)
            .iter()
            .all(|&v| m.verts[v as usize].y == front_y)
    };
    // the front face the opening lies in, and (for a soft body) the face below it whose top edge the opening's
    // bottom edge lies on
    let face = (0..mesh.faces.len())
        .find(|&f| {
            flat(f, mesh) && {
                let c = mesh.corners(f);
                let zs: Vec<f32> = c.iter().map(|&v| mesh.verts[v as usize].z).collect();
                let xs: Vec<f32> = c.iter().map(|&v| mesh.verts[v as usize].x).collect();
                xs.iter().cloned().fold(f32::INFINITY, f32::min) <= x0
                    && xs.iter().cloned().fold(f32::NEG_INFINITY, f32::max) >= x1
                    && zs.iter().cloned().fold(f32::INFINITY, f32::min) <= z0
                    && zs.iter().cloned().fold(f32::NEG_INFINITY, f32::max) >= z1
            }
        })
        .expect("the cavity opens through one face of the front");
    let old = mesh.corners(face).to_vec();
    let fz0 = old
        .iter()
        .map(|&v| mesh.verts[v as usize].z)
        .fold(f32::INFINITY, f32::min);
    let on_edge = fz0 == z0;
    let below = if on_edge {
        (0..mesh.faces.len()).find(|&f| {
            f != face && {
                let c = mesh.corners(f);
                c.iter()
                    .filter(|&&v| {
                        let p = mesh.verts[v as usize];
                        p.y == front_y && p.z == z0
                    })
                    .count()
                    == 2
            }
        })
    } else {
        None
    };
    // the new points: the cavity's back corners among the old ones, the opening's corners after them all
    let b = |x: f32, z: f32| V3::new(x, back, z);
    let hpt = |x: f32, z: f32| V3::new(x, front_y, z);
    let (inserted, tail): ([(usize, V3); 4], [V3; 4]) = if on_edge {
        (
            [
                (3, b(x0, z0)),
                (4, b(x0, z1)),
                (8, b(x1, z0)),
                (10, b(x1, z1)),
            ],
            [hpt(x1, z0), hpt(x0, z0), hpt(x1, z1), hpt(x0, z1)],
        )
    } else {
        (
            [
                (3, b(x0, z0)),
                (5, b(x0, z1)),
                (9, b(x1, z0)),
                (10, b(x1, z1)),
            ],
            [hpt(x1, z0), hpt(x1, z1), hpt(x0, z0), hpt(x0, z1)],
        )
    };
    let n_old = mesh.verts.len();
    let mut remap = vec![0u32; n_old];
    let mut verts: Vec<V3> = Vec::with_capacity(n_old + 8);
    let mut next_old = 0;
    let mut slots = [0u32; 4];
    for (k, &(at, p)) in inserted.iter().enumerate() {
        while verts.len() < at {
            remap[next_old] = verts.len() as u32;
            verts.push(mesh.verts[next_old]);
            next_old += 1;
        }
        slots[k] = verts.len() as u32;
        verts.push(p);
    }
    while next_old < n_old {
        remap[next_old] = verts.len() as u32;
        verts.push(mesh.verts[next_old]);
        next_old += 1;
    }
    let first_tail = verts.len() as u32;
    verts.extend(tail);
    let (bll, blh, bhl, bhh) = (slots[0], slots[1], slots[2], slots[3]);
    let t = |p: V3| first_tail + tail.iter().position(|&q| q == p).unwrap() as u32;
    let (hll, hlh, hhl, hhh) = (
        t(hpt(x0, z0)),
        t(hpt(x0, z1)),
        t(hpt(x1, z0)),
        t(hpt(x1, z1)),
    );
    // the front face's corners, from its (+x, -z) one round: [(+x,-z), (+x,+z), (-x,+z), (-x,-z)]
    let o: Vec<u32> = old.iter().map(|&v| remap[v as usize]).collect();
    let start = (0..o.len())
        .find(|&i| {
            let p = verts[o[i] as usize];
            let q = verts[o[(i + 1) % o.len()] as usize];
            p.x > x1 && q.x > x1 && p.z < q.z
        })
        .expect("the front face runs round from its lower right");
    let ring: Vec<u32> = (0..o.len()).map(|i| o[(start + i) % o.len()]).collect();
    let normals = mesh.normals.take();
    let old_mesh = std::mem::take(mesh);
    let mut new = Mesh {
        verts,
        sharp: old_mesh
            .sharp
            .iter()
            .map(|&(a, b)| artkit::mesh::edge(remap[a as usize], remap[b as usize]))
            .collect(),
        mats: old_mesh.mats.clone(),
        ..Mesh::default()
    };
    let mut ns: Vec<V3> = Vec::new();
    let mut push = |m: &mut Mesh, c: &[u32], mat: u16, smooth: bool, n: &[V3]| {
        m.push_face(c, mat, smooth);
        ns.extend_from_slice(n);
    };
    let fwd = V3::new(0.0, -1.0, 0.0);
    let face_of = |f: usize| old_mesh.faces[f];
    let old_normals = |f: usize| -> Vec<V3> {
        let fc = face_of(f);
        match &normals {
            Some(n) => n[fc.start as usize..(fc.start + fc.len) as usize].to_vec(),
            None => vec![V3::ZERO; fc.len as usize],
        }
    };
    let smooth = face_of(face).smooth;
    let mut placed_front = false;
    let order: Vec<usize> = if on_edge {
        (0..old_mesh.faces.len()).collect()
    } else {
        // (the front's two pieces come after the face that followed it)
        let mut v: Vec<usize> = (0..old_mesh.faces.len()).filter(|&f| f != face).collect();
        let at = v
            .iter()
            .position(|&f| f == face + 1)
            .map_or(v.len(), |p| p + 1);
        v.insert(at, face);
        v
    };
    for f in order {
        let fc = face_of(f);
        if f == face {
            placed_front = true;
            let eight = [hhl, ring[0], ring[1], ring[2], ring[3], hll, hlh, hhh];
            push(&mut new, &eight, fc.mat, smooth, &[fwd; 8]);
            if !on_edge {
                push(
                    &mut new,
                    &[hll, ring[3], ring[0], hhl],
                    fc.mat,
                    smooth,
                    &[fwd; 4],
                );
            }
            continue;
        }
        let c: Vec<u32> = old_mesh
            .corners(f)
            .iter()
            .map(|&v| remap[v as usize])
            .collect();
        let n = old_normals(f);
        if Some(f) == below {
            // its top edge, from +x to -x, takes the opening's two corners
            let k = (0..c.len())
                .find(|&i| {
                    let p = new.verts[c[i] as usize];
                    let q = new.verts[c[(i + 1) % c.len()] as usize];
                    p.z == z0 && q.z == z0 && p.y == front_y && q.y == front_y && p.x > q.x
                })
                .expect("the face below shares the opening's edge");
            let mut cc: Vec<u32> = Vec::new();
            let mut nn: Vec<V3> = Vec::new();
            for i in 0..c.len() {
                let j = (k + 1 + i) % c.len();
                cc.push(c[j]);
                nn.push(n[j]);
            }
            cc.extend([hhl, hll]);
            nn.extend([nn[nn.len() - 1], nn[0]]);
            push(&mut new, &cc, fc.mat, fc.smooth, &nn);
            continue;
        }
        push(&mut new, &c, fc.mat, fc.smooth, &n);
    }
    assert!(placed_front, "the front face is kept");
    // the cavity's walls, flat
    let walls: [([u32; 4], V3); 5] = [
        ([bll, blh, hlh, hll], V3::new(1.0, 0.0, 0.0)),
        ([bll, bhl, bhh, blh], V3::new(0.0, -1.0, 0.0)),
        ([bhh, bhl, hhl, hhh], V3::new(-1.0, 0.0, 0.0)),
        ([hhl, bhl, bll, hll], V3::new(0.0, 0.0, 1.0)),
        ([hlh, blh, bhh, hhh], V3::new(0.0, 0.0, -1.0)),
    ];
    for (c, n) in walls {
        push(&mut new, &c, slot, false, &[n; 4]);
    }
    if normals.is_some() {
        new.normals = Some(ns);
    }
    *mesh = new;
}

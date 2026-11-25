//! Normals through a join, a move and a turn, against Blender 5.2 (tests/golden/normals_joined.txt, written by
//! a script outside the repo that builds the same parts with gamekit and writes each mesh after each step: its points,
//! faces, smooth flags, sharp edges, the normal at every corner as \`mesh.corner_normals\` gives it and at every point).
//!
//! \`legs\` is the craft table's top, plates and hairpin legs as hobbies.py joins them: each leg is a tube 5.5 mm in
//! radius bent back on itself 8 mm apart, so two of its quads inside the bend cross themselves and have no area. The
//! normal Blender works out for such a face is what the rounding of its points leaves, and it turns over when the
//! mesh is moved a hair: joined onto a part that has normals of its own (the top's bevel), the leg's are not kept as
//! they were at the join but worked out again on the mesh as it is, after its origin is set too.
//!
//! The others: a smooth tube joined onto a flat part and flat parts onto a tube, then hinged; parts with sharp edges
//! and a ball smooth all over joined after a bevelled box, then hinged, turned twice and moved back; and the
//! fireside lounge's rug with the rings of its log basket, scaled unevenly.

mod common;

use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{Axis, V3};
use artkit::mesh::{Mesh, edge};
use artkit::normals::{corner_normals, vertex_normals};
use artkit::parts::{hinged, join, new_parts, turn};
use artkit::place::Facing;
use artkit::place::GameFrame;
use artkit::scene::{self, Obj};
use artkit::{cube, cylinder, fbox, fcyl, fextrude, sphere, torus};
use common::{GoldenMesh, golden_meshes, pair_points};
use std::collections::{BTreeMap, BTreeSet};

const TOP: Mat = Mat::new("craft_top", "#E3CFA8", "wood");
const METAL: Mat = Mat::new("craft_metal", "#2B2C2E", "metal");

/// A hairpin leg of the craft table at game (x, z).
fn pin(f: &GameFrame, name: &str, x: f64, z: f64) -> Obj {
    let path = [
        (x, 0.712, z - 0.03),
        (x, 0.006, z - 0.004),
        (x, 0.006, z + 0.004),
        (x, 0.712, z + 0.03),
    ];
    kit::tube!(
        f,
        name,
        &kit::fillet(&path, 0.006, 3),
        0.0055,
        METAL,
        verts = 6,
        round = 1
    )
}

fn rail(f: &GameFrame) -> Obj {
    kit::tube!(
        f,
        "rail",
        &[(0.0, 0.2, 0.0), (0.0, 0.8, 0.0), (0.3, 0.9, 0.1)],
        0.012,
        METAL
    )
}

/// Builds a case as the script built it, handing each step's part to \`snap\`.
fn build(case: &str, snap: &mut dyn FnMut(&str, Obj)) {
    let f = GameFrame::new(0.0);
    match case {
        "legs" => {
            let still = new_parts(|| {
                fextrude!(
                    f,
                    "top",
                    &artkit::forms::rounded_rect(1.7, 0.5, 0.015, 2),
                    0.035,
                    (0, 0.735, 0.2),
                    TOP,
                    plane = Facing::Top,
                    bevel = 0.004,
                    bevel_segments = 1
                );
                for sx in [-1, 1] {
                    for z in [0.0, 0.4] {
                        let x = sx as f64 * 0.78;
                        fbox!(
                            f,
                            format!("plate{sx}{z:?}"),
                            0.08,
                            0.006,
                            0.08,
                            (x, 0.7145, z),
                            METAL,
                            bevel = 0.002,
                            bevel_segments = 1
                        );
                        pin(&f, &format!("pin{sx}{z:?}"), x, z);
                    }
                }
            });
            snap("joined", artkit::join!(&still, "body", origin = (0, 0, 0)));
        }
        "tube_on_flat" => {
            let p = new_parts(|| {
                cube!(
                    "block",
                    (0.2, 0.1, 0.3),
                    at = (0.3, -0.2, 0.4),
                    material = TOP,
                    rot = (0, 0, 20),
                    bevel = 0
                );
                pin(&f, "pin", -0.78, 0.4);
                rail(&f);
            });
            snap("joined", artkit::join!(&p, "body", origin = (0, 0, 0)));
        }
        "flat_on_tube" => {
            let p = new_parts(|| {
                rail(&f);
                pin(&f, "pin", 0.78, 0.0);
                cube!(
                    "block",
                    (0.2, 0.1, 0.3),
                    at = (0.3, -0.2, 0.4),
                    material = TOP,
                    rot = (0, 0, 20),
                    bevel = 0
                );
                cylinder!(
                    "post",
                    0.03,
                    0.2,
                    at = (-0.2, 0.1, 0.1),
                    material = TOP,
                    verts = 8,
                    smooth = false
                );
            });
            let body = join(&p, "body", None);
            snap("joined", body);
            snap("hinged", hinged(body, (0.1, 0.2, 0.3)));
        }
        "sharp_after_bevel" => {
            let p = new_parts(|| {
                fbox!(
                    f,
                    "plate",
                    0.3,
                    0.02,
                    0.2,
                    (0.1, 0.5, 0.05),
                    TOP,
                    bevel = 0.004
                );
                cylinder!(
                    "drum",
                    0.05,
                    0.12,
                    at = (0.2, 0.1, 0.3),
                    material = METAL,
                    verts = 8,
                    rot = (90, 0, 30)
                );
                sphere!(
                    "ball",
                    0.04,
                    at = (-0.1, 0.05, 0.2),
                    material = METAL,
                    segments = 8,
                    rings = 5
                );
                pin(&f, "pin", -0.78, 0.0);
            });
            let body = artkit::join!(&p, "body", origin = (0, 0, 0));
            snap("joined", body);
            snap("hinged", hinged(body, f.at(0.1, 0.5, 0.15)));
            snap("turned_z", turn(body, 37.0, Axis::Z));
            snap("turned_x", turn(body, -90.0, Axis::X));
            snap("origin", hinged(body, (0, 0, 0)));
        }
        "torus" => {
            let p = new_parts(|| {
                fcyl!(
                    f,
                    "rug",
                    0.85,
                    0.012,
                    (0, 0.006, 0.2),
                    TOP,
                    verts = 40,
                    scale = (1, 0.76, 1),
                    bevel = 0.004,
                    bevel_segments = 1
                );
                for (k, y) in [0.08, 0.16].into_iter().enumerate() {
                    let r = 0.15 + 0.02 * y / 0.24;
                    torus!(
                        format!("weave{k}"),
                        1.0,
                        0.006,
                        at = f.at(0.68, y, 0.62),
                        material = METAL,
                        major_segments = 18,
                        minor_segments = 3
                    )
                    .set_scale((r + 0.003, (r + 0.003) * 0.76, 1.0));
                }
                torus!(
                    "basket_rim",
                    1.0,
                    0.012,
                    at = f.at(0.68, 0.24, 0.62),
                    material = METAL,
                    major_segments = 18,
                    minor_segments = 4
                )
                .set_scale((0.165, 0.165 * 0.76, 1.0));
            });
            snap("joined", artkit::join!(&p, "body", origin = (0, 0, 0)));
        }
        other => panic!("no case {other}"),
    }
}

/// How a built mesh stands against Blender's.
#[derive(Debug, Default)]
struct Held {
    /// Points that are not Blender's to the bit, and the farthest off (metres).
    points: (usize, f32),
    /// Faces Blender has not, by their corners in order.
    faces: usize,
    /// Faces smooth where Blender's is flat or flat where it is smooth, and edges sharp in one and not the other.
    smooth: usize,
    sharp: usize,
    /// Corners whose normal looks the other way, corners with no normal where Blender has one (or one where it has
    /// none), and the farthest a corner's normal is from Blender's otherwise (a coordinate's difference).
    turned: usize,
    none: usize,
    corner: f32,
    /// The farthest a point's normal is from Blender's.
    vertex: f32,
}

fn vectors(g: &GoldenMesh, key: &str) -> Vec<V3> {
    g.f32s(key)
        .chunks(3)
        .map(|c| V3::new(c[0], c[1], c[2]))
        .collect()
}

fn apart(a: V3, b: V3) -> f32 {
    (0..3).map(|i| (a[i] - b[i]).abs()).fold(0.0, f32::max)
}

/// A face's corners from its lowest vertex on, and how far round that is.
fn from_lowest(c: &[u32]) -> (Vec<u32>, usize) {
    let at = (0..c.len()).min_by_key(|&i| c[i]).unwrap_or(0);
    (c[at..].iter().chain(&c[..at]).copied().collect(), at)
}

fn held(mesh: &Mesh, want: &GoldenMesh) -> Held {
    let mut out = Held::default();
    assert_eq!(mesh.verts.len(), want.verts.len(), "points");
    assert_eq!(mesh.faces.len(), want.faces.len(), "faces");
    // the points Blender's to the bit and in its order, or paired with its nearest (a bevel's are made in another
    // order now and then)
    let same = mesh
        .verts
        .iter()
        .zip(&want.verts)
        .all(|(g, w)| (0..3).all(|i| g[i] == w[i]));
    let map: Vec<u32> = if same {
        (0..mesh.verts.len() as u32).collect()
    } else {
        let (map, inexact, far) =
            pair_points(&mesh.verts, &want.verts, 2e-6).expect("the points pair up");
        out.points = (inexact, far);
        map
    };
    // Blender's faces by their corners from the lowest vertex on: (the face's first corner, how far round it starts)
    let mut theirs: BTreeMap<Vec<u32>, (usize, usize)> = BTreeMap::new();
    let mut start = 0;
    for face in &want.faces {
        let (key, at) = from_lowest(face);
        theirs.insert(key, (start, at));
        start += face.len();
    }
    let smooth = want
        .lines
        .get("s")
        .map(String::as_bytes)
        .unwrap_or_default();
    let (want_corner, want_vertex) = (vectors(want, "cn"), vectors(want, "vn"));
    let (got_corner, got_vertex) = (corner_normals(mesh), vertex_normals(mesh));
    let mut their_face = 0;
    let mut starts: BTreeMap<usize, usize> = BTreeMap::new();
    for (i, face) in want.faces.iter().enumerate() {
        starts.insert(their_face, i);
        their_face += face.len();
    }
    for f in 0..mesh.faces.len() {
        let renamed: Vec<u32> = mesh.corners(f).iter().map(|&v| map[v as usize]).collect();
        let (key, at) = from_lowest(&renamed);
        let Some(&(first, their_at)) = theirs.get(&key) else {
            out.faces += 1;
            continue;
        };
        let n = renamed.len();
        out.smooth += (mesh.faces[f].smooth != (smooth[starts[&first]] == b'1')) as usize;
        for k in 0..n {
            let got = got_corner[mesh.faces[f].start as usize + (at + k) % n];
            let want = want_corner[first + (their_at + k) % n];
            if (got == V3::ZERO) != (want == V3::ZERO) {
                out.none += 1;
            } else if got.x * want.x + got.y * want.y + got.z * want.z < 0.0 {
                out.turned += 1;
            } else {
                out.corner = out.corner.max(apart(got, want));
            }
        }
    }
    let sharp: BTreeSet<(u32, u32)> = mesh
        .sharp
        .iter()
        .map(|&(a, b)| edge(map[a as usize], map[b as usize]))
        .collect();
    let their_sharp: BTreeSet<(u32, u32)> = want
        .lines
        .get("e")
        .map(|l| {
            l.split(' ')
                .filter(|w| !w.is_empty())
                .map(|w| {
                    let (a, b) = w.split_once('-').unwrap();
                    edge(a.parse().unwrap(), b.parse().unwrap())
                })
                .collect()
        })
        .unwrap_or_default();
    out.sharp = sharp.symmetric_difference(&their_sharp).count();
    for (v, &got) in got_vertex.iter().enumerate() {
        out.vertex = out.vertex.max(apart(got, want_vertex[map[v] as usize]));
    }
    out
}

#[test]
fn joined_parts_have_blenders_normals() {
    let goldens = golden_meshes("normals_joined.txt");
    let mut case = "";
    let mut built: BTreeMap<String, (V3, Mesh)> = BTreeMap::new();
    let mut checked = 0;
    for want in &goldens {
        let (name, stage) = want.what.split_once(' ').unwrap();
        if name != case {
            case = name;
            scene::reset();
            built.clear();
            build(case, &mut |stage, obj| {
                built.insert(stage.to_string(), obj.read(|o| (o.loc, o.mesh.clone())));
            });
        }
        let (loc, mesh) = &built[stage];
        let what = &want.what;
        assert_eq!(
            common::bits(&loc.array()),
            common::bits(&want.f32s("loc")),
            "{what}: its place"
        );
        let got = held(mesh, want);
        // (the fireside lounge's rug: a bevel's stored normals read back a hair further off)
        let near = if case == "torus" { 2e-4 } else { 1e-4 };
        assert_eq!(
            got.points,
            (0, 0.0),
            "{what}: points not Blender's to the bit: {got:?}"
        );
        assert_eq!(
            (got.faces, got.smooth, got.sharp),
            (0, 0, 0),
            "{what}: faces, smooth flags or sharp edges not Blender's: {got:?}"
        );
        assert_eq!(
            (got.turned, got.none),
            (0, 0),
            "{what}: corners turned or without a normal: {got:?}"
        );
        assert!(
            got.corner <= near && got.vertex <= near,
            "{what}: normals off: {got:?}"
        );
        checked += 1;
    }
    assert_eq!(checked, 10, "every step of the golden");
}

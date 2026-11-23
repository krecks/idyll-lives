//! Joined scenes the kernel builds itself against the same scenes as gamekit builds them in Blender 5.2
//! (tests/golden/shade_built.txt, six scenes: each joined mesh as Blender held it: its points, its polygons in order from the
//! corner each starts at, the two angles stored for each corner's normal, the vertices' normals, and the levels
//! gamekit's own `shade.bake_ao` baked). Where a part lies exactly on another, a ray meets two faces at once and the
//! last bit of the direction it is cast in decides which: the shading is Blender's only if the mesh the kernel
//! joins is, down to the normals a bevel hardens, which Blender keeps as two angles of sixteen bits in each
//! corner's fan's frame.
mod common;

use artkit::materials::{Finish, Finishes, Mat};
use artkit::mesh::Mesh;
use artkit::normals::vertex_normals;
use artkit::parts::{join, join_all, new_parts, set_origin};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{self, Obj};
use artkit::soft::{Puff, SoftBox};
use artkit::{fbox, fcyl, kit_moulding, shade};
use common::*;

const BODY: Mat = Mat::new("body", "#C0683F", "satin");
const PAD: Mat = Mat::new("pad", "#3A3D41", "matte");
const KEY: Mat = Mat::new("key", "#2A2C30", "satin");
const WOOD: Mat = Mat::new("wood", "#CDB08A", "satin");

fn finishes() -> Finishes {
    ["satin", "matte"]
        .into_iter()
        .map(|k| {
            let finish = Finish {
                roughness: 0.5,
                metallic: 0.0,
                textured: false,
            };
            (k.to_string(), finish)
        })
        .collect()
}

/// Joins `parts` and turns them about the game point `pivot` (office.py's `_turned`): the turn is baked in when
/// the part is joined again.
fn turned(f: &GameFrame, parts: &[Obj], name: &str, pivot: (f64, f64, f64), x: f64, z: f64) -> Obj {
    let obj = join(parts, name, None);
    set_origin(obj, f.at(pivot.0, pivot.1, pivot.2));
    obj.set_rotation((x, 0, z))
}

/// The scene `name` as the capture script built it in Blender, joined.
fn build(name: &str) -> Obj {
    let f = GameFrame::default();
    match name {
        "pong" => {
            fbox!(
                f,
                "top",
                2.74,
                0.03,
                1.525,
                (0, 0.76 - 0.015, 0),
                WOOD,
                bevel = 0.006,
                bevel_segments = 1
            );
            kit_moulding!(
                f,
                "top_band",
                &[
                    (1.37, -0.7625),
                    (1.37, 0.7625),
                    (-1.37, 0.7625),
                    (-1.37, -0.7625)
                ],
                &[
                    (0.0, -0.035),
                    (0.0, -0.002),
                    (0.006, -0.002),
                    (0.006, -0.035)
                ],
                KEY,
                at = 0.76,
                plane = Facing::Top
            );
        }
        "bevels" => {
            fbox!(
                f,
                "base",
                0.5,
                0.3,
                0.4,
                (0, 0.15, 0),
                BODY,
                bevel = 0.01,
                bevel_segments = 2
            );
            fbox!(
                f,
                "top",
                0.2,
                0.2,
                0.2,
                (0.05, 0.4, 0.03),
                KEY,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
        "soft" => {
            fbox!(f, "seat", 0.5, 0.3, 0.4, (0, 0.15, 0), BODY, bevel = 0);
            let o = SoftBox {
                round: 0.02,
                puff: Puff::Faces(0.006),
                ..SoftBox::default()
            };
            f.soft("cushion", 0.4, 0.06, 0.3, (0, 0.33, 0), KEY, o);
        }
        "cyl" => {
            fbox!(f, "seat", 0.5, 0.3, 0.4, (0, 0.15, 0), BODY, bevel = 0);
            fcyl!(f, "can", 0.05, 0.12, (0.1, 0.36, 0.05), KEY, verts = 16);
        }
        "laptop" => {
            let (x, y, z) = (0.05, 0.74, -0.62);
            fbox!(
                f,
                "top",
                0.56,
                0.035,
                0.5,
                (0, y - 0.0175, -0.68),
                WOOD,
                bevel = 0.006,
                bevel_segments = 2
            );
            let parts = new_parts(|| {
                fbox!(
                    f,
                    "laptop",
                    0.32,
                    0.012,
                    0.22,
                    (x, y + 0.006, z),
                    BODY,
                    bevel = 0.004,
                    bevel_segments = 1
                );
                fbox!(
                    f,
                    "laptop_well",
                    0.29,
                    0.0006,
                    0.105,
                    (x, y + 0.0121, z - 0.04),
                    PAD,
                    bevel = 0
                );
                for r in 0..5 {
                    fbox!(
                        f,
                        format!("laptop_keys{r}"),
                        0.27,
                        0.0012,
                        0.013,
                        (x, y + 0.0126, z - 0.08 + 0.0168 * f64::from(r)),
                        KEY,
                        bevel = 0
                    );
                }
                fbox!(
                    f,
                    "laptop_pad",
                    0.1,
                    0.0006,
                    0.06,
                    (x, y + 0.0123, z + 0.06),
                    PAD,
                    bevel = 0
                );
                let lid = new_parts(|| {
                    fbox!(
                        f,
                        "lid",
                        0.32,
                        0.007,
                        0.22,
                        (x, y + 0.0165, z),
                        BODY,
                        bevel = 0.003,
                        bevel_segments = 1
                    );
                    fbox!(
                        f,
                        "lid_screen",
                        0.29,
                        0.0008,
                        0.18,
                        (x, y + 0.0127, z + 0.005),
                        PAD,
                        bevel = 0
                    );
                });
                turned(
                    &f,
                    &lid,
                    "laptop_lid",
                    (x, y + 0.013, z - 0.11),
                    -110.0,
                    0.0,
                );
            });
            turned(&f, &parts, "laptop", (x, y, z), 0.0, -20.0);
        }
        "sun" => {
            fbox!(
                f,
                "board",
                0.58,
                0.006,
                0.4,
                (0, 0.7555, 0.2),
                WOOD,
                bevel = 0.002,
                bevel_segments = 1
            );
            fcyl!(
                f,
                "sun",
                0.035,
                0.0015,
                (-0.17, 0.7585 + 0.0012, 0.08),
                KEY,
                verts = 16,
                smooth = false
            );
        }
        other => panic!("no scene {other}"),
    }
    join_all(name)
}

/// The scenes with a cylinder: its caps aren't where Blender has them among its sides (the same faces, in another
/// order), and a vertex normal is summed in the faces' order.
const CYLINDERS: [&str; 2] = ["cyl", "sun"];

/// The scene `g` built by the kernel: its joined mesh, and for each of its faces the place of the same face
/// (from the same corner) among Blender's.
fn built(g: &GoldenMesh) -> (Obj, Mesh, Vec<usize>) {
    scene::reset();
    scene::set_finishes(finishes());
    let obj = build(&g.name);
    let mesh = obj.read(|o| o.mesh.clone());
    // Blender's points, to the bit and in its order
    assert_eq!(mesh.verts.len(), g.verts.len(), "{}: points", g.name);
    for (i, v) in mesh.verts.iter().enumerate() {
        assert_eq!(bits(&v.array()), bits(&g.verts[i]), "{}: point {i}", g.name);
    }
    // Blender's polygons, each from the corner Blender's starts at
    assert_eq!(mesh.faces.len(), g.faces.len(), "{}: faces", g.name);
    let place: Vec<usize> = (0..mesh.faces.len())
        .map(|f| {
            g.faces
                .iter()
                .position(|b| b.as_slice() == mesh.corners(f))
                .unwrap_or_else(|| panic!("{}: face {f} isn't one of Blender's", g.name))
        })
        .collect();
    (obj, mesh, place)
}

/// Where each of Blender's faces' corners start among its mesh's.
fn starts(g: &GoldenMesh) -> Vec<usize> {
    g.faces
        .iter()
        .scan(0, |at, f| {
            let start = *at;
            *at += f.len();
            Some(start)
        })
        .collect()
}

#[test]
fn a_bevels_hardened_normals_are_stored_as_blender_stores_them() {
    let mut scenes = 0;
    for g in golden_meshes("shade_built.txt") {
        let Some(line) = g.lines.get("cs") else {
            continue;
        };
        let want: Vec<[i16; 2]> = line
            .split(' ')
            .map(|w| {
                let (a, b) = w.split_once(',').unwrap();
                [a.parse().unwrap(), b.parse().unwrap()]
            })
            .collect();
        let (_, mesh, place) = built(&g);
        let stored = mesh.stored.as_ref().expect("stored angles");
        let starts = starts(&g);
        let mut off = Vec::new();
        for (f, &b) in place.iter().enumerate() {
            let start = mesh.faces[f].start as usize;
            for c in 0..mesh.corners(f).len() {
                // (a corner of a part with no normals of its own: Blender's zeros)
                let got = stored[start + c].unwrap_or([0, 0]);
                if got != want[starts[b] + c] {
                    off.push((f, c, got, want[starts[b] + c]));
                }
            }
        }
        assert!(
            off.is_empty(),
            "{}: {} of {} corners' angles aren't Blender's (face, corner, got, Blender's): {:?}",
            g.name,
            off.len(),
            want.len(),
            &off[..off.len().min(8)]
        );
        scenes += 1;
    }
    assert_eq!(scenes, 4);
}

#[test]
fn the_built_scenes_are_blenders_and_bake_to_its_levels() {
    for g in golden_meshes("shade_built.txt") {
        let (obj, mesh, place) = built(&g);
        // the polygons in Blender's order: the tree rays are cast at is built of them in that order, and of two
        // faces a ray meets at once it is given the one the tree comes to first
        if !CYLINDERS.contains(&g.name.as_str()) {
            let moved: Vec<usize> = (0..place.len()).filter(|&f| place[f] != f).collect();
            assert!(
                moved.is_empty(),
                "{}: faces out of Blender's order: {moved:?}",
                g.name
            );
        }
        // the vertices' normals, which the rays are cast about: to the bit
        let normals = vertex_normals(&mesh);
        let want = g.f32s("vn");
        let off: Vec<usize> = (0..normals.len())
            .filter(|&i| bits(&normals[i].array()) != bits(&want[3 * i..3 * i + 3]))
            .collect();
        if !CYLINDERS.contains(&g.name.as_str()) {
            assert!(
                off.is_empty(),
                "{}: {} of {} vertex normals aren't Blender's bits: {:?}",
                g.name,
                off.len(),
                normals.len(),
                &off[..off.len().min(12)]
            );
        }
        // the levels baked
        let reach = g.lines.get("reach").map(|w| f64w(w));
        shade::bake_ao(&[obj], reach, g.lines["ground"] == "1");
        let got: Vec<i64> = obj
            .read(|o| o.mesh.ao.clone().unwrap())
            .iter()
            .map(|a| (a * 32.0).round() as i64)
            .collect();
        let want: Vec<i64> = g.lines["ao"]
            .split(' ')
            .map(|l| l.parse().unwrap())
            .collect();
        let off: Vec<(usize, i64, i64)> = (0..got.len())
            .filter(|&i| got[i] != want[i])
            .map(|i| (i, got[i], want[i]))
            .collect();
        assert!(
            off.is_empty(),
            "{}: {} of {} vertices off Blender's level (vertex, got, Blender's): {:?}",
            g.name,
            off.len(),
            got.len(),
            &off[..off.len().min(12)]
        );
    }
}

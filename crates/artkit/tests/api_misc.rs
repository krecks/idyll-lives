//! gamekit's fit and the other ways bmesh.ops.triangulate cuts a quad or a polygon, against Blender 5.2
//! (tests/golden/api_misc.txt), and the calls by name that no other test makes.

mod common;

use artkit::edit::{NgonMethod, QuadMethod, triangulate};
use artkit::materials::Mat;
use artkit::math::{Axis, V3};
use artkit::mesh::Mesh;
use artkit::motion::{self, Path};
use artkit::parts::fit;
use artkit::scene::{self, Obj};
use artkit::{cube, cylinder, join, rock, settle, squash, stages, wobble};
use common::{f32s, golden};

const BODY: Mat = Mat::new("body", "#F3F3F1", "gloss");

fn pipe(name: &str) -> Obj {
    cylinder!(
        name,
        0.1,
        0.4,
        at = (0.3, -0.2, 0.5),
        material = BODY,
        verts = 8,
        rot = (20, 0, 40)
    )
}

#[test]
fn fit_and_every_way_of_cutting_a_face_are_blenders() {
    let text = golden("api_misc.txt");
    let line = |start: &str| -> Vec<&str> {
        let l = text.lines().find(|l| l.starts_with(start)).unwrap();
        l.split(' ').collect()
    };
    // fit: by the part's matrix as it stands, each axis on its own
    {
        let label = "updated";
        scene::reset();
        let o = pipe("a");
        o.set_scale((1.5, 1.0, 0.7));
        fit(o, 0.5, 0.3, 0.2);
        let w = line(&format!("fit {label} "));
        let at = |key: &str| w.iter().position(|x| *x == key).unwrap();
        assert_eq!(
            o.location().array().to_vec(),
            f32s(&w[at("loc") + 1..at("loc") + 4]),
            "{label}: place"
        );
        assert_eq!(
            o.scale().array().to_vec(),
            f32s(&w[at("scale") + 1..at("scale") + 4]),
            "{label}: scale"
        );
        let got: Vec<f32> = o.read(|p| p.mesh.verts.iter().flat_map(|v| v.array()).collect());
        assert_eq!(got, f32s(&w[at("v") + 1..]), "{label}: points");
    }
    // triangulate: two quads, a polygon of nine corners with a hollow, one of five, cut every way
    let coords = f32s(&line("points ")[1..]);
    let verts: Vec<V3> = coords
        .chunks(3)
        .map(|c| V3::new(c[0], c[1], c[2]))
        .collect();
    let faces: Vec<Vec<u32>> = line("faces ")[1..]
        .iter()
        .map(|f| f.split(',').map(|i| i.parse().unwrap()).collect())
        .collect();
    let mut ways = 0;
    for l in text.lines().filter(|l| l.starts_with("tri ")) {
        let w: Vec<&str> = l.split(' ').collect();
        let quad = match w[1] {
            "BEAUTY" => QuadMethod::Beauty,
            "FIXED" => QuadMethod::Fixed,
            "ALTERNATE" => QuadMethod::Alternate,
            "SHORT_EDGE" => QuadMethod::ShortEdge,
            _ => QuadMethod::LongEdge,
        };
        let ngon = if w[2] == "BEAUTY" {
            NgonMethod::Beauty
        } else {
            NgonMethod::EarClip
        };
        scene::reset();
        let obj = scene::add("faces", Mesh::from_pydata(verts.clone(), &faces));
        triangulate(obj, quad, ngon);
        let got: Vec<String> = obj.read(|o| {
            (0..o.mesh.faces.len())
                .map(|f| {
                    let c: Vec<String> = o.mesh.corners(f).iter().map(u32::to_string).collect();
                    c.join(",")
                })
                .collect()
        });
        // every triangle Blender's, from the corner Blender's starts at and in Blender's order
        assert_eq!(got.join(" "), w[3..].join(" "), "{} {}", w[1], w[2]);
        ways += 1;
    }
    assert_eq!(ways, 10);
}

#[test]
fn the_calls_by_name_are_the_functions_with_their_options() {
    // join! and settle!
    scene::reset();
    let lid = [cube!(
        "lid",
        (0.4, 0.3, 0.02),
        at = (0, 0, 0.5),
        material = BODY
    )];
    let base = [cube!(
        "base",
        (0.4, 0.3, 0.5),
        at = (0, 0, 0.25),
        material = BODY
    )];
    let lid = join!(&lid, "lid", origin = (0, 0.15, 0.5));
    let base = join!(&base, "base");
    assert_eq!(lid.location().array(), [0.0, 0.15, 0.5]);
    let all = settle!(centre = false);
    assert_eq!(all, [lid, base]);
    let some = settle!(&[lid], centre = false, floor = false);
    assert_eq!(some, [lid]);
    assert_eq!(lid.location().z, 0.5);
    // the clips: each call by name keys what the function keys (stages: the three scales and the height)
    squash!(
        base,
        "sink",
        (1, 1, 0.7),
        seconds = 0.5,
        back = true,
        split = 0.3
    );
    rock!(lid, "rock", 4, seconds = 2.0, axis = Axis::Y);
    wobble!(lid, "wobble", 6, seconds = 1.0, swings = 2, delay = 0.2);
    stages!(
        base,
        "grow",
        picked = 1,
        growing = 0.6,
        planted = (1, 1, 0.25),
        sink = (0.0, 0.1, 0.2)
    );
    motion::peak(lid, "rock", motion::last_frame(1.5));
    let clips = |o: Obj| -> Vec<(String, usize)> {
        o.read(|p| {
            p.tracks
                .iter()
                .map(|t| (t.name.clone(), t.channels.len()))
                .collect()
        })
    };
    assert_eq!(
        clips(base),
        [("sink".to_string(), 1), ("grow".to_string(), 4)]
    );
    assert_eq!(
        clips(lid),
        [("rock".to_string(), 1), ("wobble".to_string(), 1)]
    );
    lid.read(|p| {
        assert_eq!(p.tracks[0].peak, Some(46));
        assert_eq!(p.tracks[0].channels[0].path, Path::Rotation);
    });
}

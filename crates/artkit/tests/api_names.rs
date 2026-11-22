//! The names Blender gives parts and their meshes, and the order its exporter writes nodes in
//! (tests/golden/api_names.txt, from Blender 5.2): a name in use gets a number after it (water.001), a part's
//! by the parts that are there, a mesh's by every mesh made since the scene was emptied (a part joined away
//! leaves its mesh's name taken); a primitive's mesh is named for its kind; the file's nodes come by name.

mod common;

use artkit::forms::mirrored;
use artkit::glb;
use artkit::materials::{Finish, Finishes, Mat};
use artkit::math::Axis;
use artkit::parts::{join, join_all, new_parts};
use artkit::scene::{self, Obj, meshes};
use artkit::{
    boulder, cone, cube, cylinder, extrude, loft, revolve, ring, rod, soft_box, sphere, sweep,
    torus,
};
use common::golden;
use std::collections::BTreeMap;

const BODY: Mat = Mat::new("body", "#F3F3F1", "gloss");
const WATER: Mat = Mat::new("water", "#9DC6CF", "gloss");

fn finishes() -> Finishes {
    let mut f = Finishes::new();
    f.insert(
        "gloss".to_string(),
        Finish {
            roughness: 0.3,
            metallic: 0.0,
            textured: false,
        },
    );
    f
}

/// The parts as the golden writes them: `part=mesh` each.
fn names(objs: &[Obj]) -> String {
    let each: Vec<String> = objs
        .iter()
        .map(|o| format!("{}={}", o.name(), o.data_name()))
        .collect();
    each.join(" ")
}

/// The file's nodes as the golden writes them: `node=mesh` each, in the file's order.
fn written(objs: &[Obj]) -> String {
    let model = glb::read(&glb::write(objs, false, &finishes())).unwrap();
    let each: Vec<String> = model
        .nodes
        .iter()
        .map(|n| format!("{}={}", n.name, model.meshes[n.mesh.unwrap()].name))
        .collect();
    each.join(" ")
}

fn knob() -> Obj {
    revolve!(
        "knob",
        &[(0.0, 0.0), (0.1, 0.0), (0.0, 0.1)],
        material = BODY,
        segments = 6
    )
}

#[test]
fn names_in_use_get_blenders_numbers_and_the_file_has_its_nodes_by_name() {
    let mut want: BTreeMap<String, String> = BTreeMap::new();
    let mut case = String::new();
    for line in golden("api_names.txt").lines() {
        let (kind, rest) = line.split_once(' ').unwrap();
        if kind == "case" {
            case = rest.to_string();
        } else {
            let (label, list) = rest.split_once(' ').unwrap();
            want.insert(format!("{case} {label}"), list.to_string());
        }
    }
    let mut checked = 0;
    let mut same = |key: &str, got: String| {
        assert_eq!(got, want[key], "{key}");
        checked += 1;
    };
    scene::set_finishes(finishes());

    // two taps' water, each joined as 'water' (the double sink)
    scene::reset();
    cube!(
        "body",
        (1.0, 0.5, 0.8),
        at = (0, 0, 0.4),
        material = BODY,
        bevel = 0
    );
    let body = join_all("body");
    let mut streams = Vec::new();
    for x in [-0.25, 0.25] {
        let parts = new_parts(|| {
            cylinder!(
                "water",
                0.012,
                0.3,
                at = (x, 0, 0.95),
                material = WATER,
                verts = 10
            );
        });
        same(&format!("water part{x}"), names(&parts));
        streams.push(join(&parts, "water", Some((x, 0, 1.1).into())));
    }
    same("water joined", names(&meshes()));
    same("water written", written(&[body, streams[0], streams[1]]));

    // a lathe named as the join it goes into, and not the first of its parts
    scene::reset();
    let still = new_parts(|| {
        cylinder!(
            "post",
            0.05,
            0.6,
            at = (0, 0, 0.3),
            material = BODY,
            verts = 8
        );
        revolve!(
            "hub",
            &[(0.0, 0.03), (0.17, 0.03), (0.17, 0.05), (0.0, 0.15)],
            material = BODY,
            segments = 8
        );
    });
    same("hub parts", names(&still));
    let hub = join(&still, "hub", Some((0, 0, 0).into()));
    let top = new_parts(|| {
        revolve!(
            "platform",
            &[(0.0, 0.2), (0.5, 0.2), (0.5, 0.25), (0.0, 0.25)],
            material = BODY,
            segments = 8
        );
        cube!(
            "rail",
            (0.02, 0.02, 0.3),
            at = (0.4, 0, 0.4),
            material = BODY,
            bevel = 0
        );
    });
    let platform = join(&top, "platform", None);
    same("hub joined", names(&meshes()));
    same("hub written", written(&[hub, platform]));

    // two joins under one name, the second renamed after (the bunk bed's flaps)
    scene::reset();
    let mut flaps = Vec::new();
    for z in [0.5, 1.5] {
        let parts = new_parts(|| {
            soft_box!("duvet", (0.8, 0.4, 0.05), at = (0, 0, z), material = BODY);
            cube!(
                "hem",
                (0.8, 0.02, 0.02),
                at = (0, 0.2, z),
                material = BODY,
                bevel = 0
            );
        });
        flaps.push(join(&parts, "duvet_flap", None));
    }
    same("flaps joined", names(&meshes()));
    flaps[1].set_name("duvet_flap_upper");
    same("flaps renamed", names(&meshes()));
    same("flaps written", written(&flaps));

    // parts under one name: primitives and built meshes, then one taken away and the name asked for again
    scene::reset();
    let a: Vec<Obj> = (0..3)
        .map(|_| cube!("leg", 0.1, material = BODY, bevel = 0))
        .collect();
    let b: Vec<Obj> = (0..3).map(|_| knob()).collect();
    same("parts made", names(&meshes()));
    a[1].remove();
    b[1].remove();
    cube!("leg", 0.1, material = BODY, bevel = 0);
    knob();
    same("parts again", names(&meshes()));
    cube!("leg.001", 0.1, material = BODY, bevel = 0);
    cube!("leg.007", 0.1, material = BODY, bevel = 0);
    cube!("leg.007", 0.1, material = BODY, bevel = 0);
    same("parts numbered", names(&meshes()));
    sphere!("ball", 0.1, material = BODY, segments = 8, rings = 4);
    sphere!("ball", 0.1, material = BODY, segments = 8, rings = 4);
    cone!("ball", 0.1, 0.0, 0.2, material = BODY, verts = 8);
    torus!(
        "ball",
        0.2,
        0.02,
        material = BODY,
        major_segments = 8,
        minor_segments = 4
    );
    boulder!("ball", 0.2, material = BODY, subdivisions = 1);
    let rings: Vec<_> = [0.0, 0.2]
        .iter()
        .map(|&z| ring!(0.1, 0.1, z = z, verts = 6))
        .collect();
    loft!("ball", &rings, material = BODY);
    sweep!(
        "ball",
        &[(0.0, 0.0, 0.0), (0.0, 0.0, 0.3)],
        0.02,
        material = BODY,
        verts = 6
    );
    extrude!(
        "ball",
        &[(0.0, 0.0), (0.2, 0.0), (0.2, 0.2)],
        0.05,
        material = BODY
    );
    rod!(
        "ball",
        (0, 0, 0),
        (0.1, 0.1, 0.1),
        0.01,
        material = BODY,
        verts = 6
    );
    same("parts kinds", names(&meshes()));

    // a mirrored copy, and what it is called
    scene::reset();
    let arm = cube!(
        "arm",
        (0.1, 0.4, 0.1),
        at = (0.3, 0, 0.5),
        material = BODY,
        bevel = 0
    );
    mirrored(arm, Axis::X, None);
    mirrored(arm, Axis::X, None);
    mirrored(arm, Axis::X, Some("arm"));
    same("mirror made", names(&meshes()));

    // the file's nodes come by name, however the parts were made or given
    scene::reset();
    let zed = join(
        &[cube!(
            "z0",
            0.1,
            at = (0, 0, 0.05),
            material = BODY,
            bevel = 0
        )],
        "zed",
        None,
    );
    let alpha = join(
        &[cube!(
            "a0",
            0.1,
            at = (0.3, 0, 0.05),
            material = BODY,
            bevel = 0
        )],
        "alpha",
        None,
    );
    let mid = join(
        &[cube!(
            "m0",
            0.1,
            at = (0.6, 0, 0.05),
            material = BODY,
            bevel = 0
        )],
        "mid",
        None,
    );
    same("order made", names(&meshes()));
    same("order as_made", written(&[zed, alpha, mid]));
    same("order given_otherwise", written(&[mid, zed, alpha]));
    mid.set_name("aaa");
    same("order renamed", written(&[zed, alpha, mid]));
    assert_eq!(checked, want.len());
}

//! The kernel from end to end on the Italian cypress, against what Blender 5.2 made of families/trees.py's
//! (tests/golden/cypress*.txt: the mesh its perches were found on, the rays cast at it and where they met it, the
//! perches found and as settled, the baked shading).
use crate::common::cypress;

use crate::common::*;
use artkit::bvh::Rays;
use artkit::family::{build_model, make, make_far};
use artkit::math::V3;
use artkit::scene;
use artkit::{perch, shade};

fn lines(name: &str, key: &str) -> Vec<String> {
    golden(name)
        .lines()
        .filter_map(|l| {
            l.strip_prefix(key)
                .and_then(|r| r.strip_prefix(' '))
                .map(String::from)
        })
        .collect()
}

fn points(name: &str, key: &str) -> Vec<Vec<u32>> {
    lines(name, key)
        .iter()
        .map(|l| bits(&f32s(&l.split(' ').collect::<Vec<_>>())))
        .collect()
}

#[test]
fn the_cypress_is_blenders_and_its_perches_stand_where_they_stood() {
    scene::set_finishes(cypress::finishes());
    let family = cypress::family();
    let model = &family.models[0];
    let objs = make(model).unwrap();
    assert_eq!(objs.len(), 1);
    // its perches, as found and as the pack has them: to the bit
    let settled: Vec<Vec<u32>> =
        scene::with(|s| s.perches.iter().map(|p| bits(&p.array())).collect());
    assert_eq!(settled, points("cypress.txt", "perch_settled"));
    let game = perch::game();
    let text: Vec<String> = game
        .iter()
        .map(|p| format!("[{}, {}, {}]", p[0], p[1], p[2]))
        .collect();
    assert_eq!(
        format!("[{}]", text.join(", ")),
        lines("cypress.txt", "game")[0]
    );
    // its mesh: Blender's points to the bit and in its order, its faces, its triangles
    let meshes = golden_meshes("cypress.scene.txt");
    let (found_on, last) = (&meshes[0], &meshes[1]);
    let mesh = objs[0].read(|o| o.mesh.clone());
    let (map, inexact, _) =
        pair_points(&mesh.verts, &found_on.verts, 0.0).expect("the cypress's points");
    assert_eq!(inexact, 0);
    assert!(
        map.iter().enumerate().all(|(i, &j)| i == j as usize),
        "the points' order"
    );
    let faces: Vec<Vec<u32>> = (0..mesh.faces.len())
        .map(|f| mesh.corners(f).to_vec())
        .collect();
    assert_eq!(sorted(&renamed(&faces, &map)), sorted(&found_on.faces));
    assert_eq!(
        as_set(&renamed(&mesh.triangle_verts(), &map)),
        as_set(&found_on.tris)
    );
    // where it stands, and its size
    let loc: Vec<&str> = golden("cypress.txt")
        .lines()
        .find(|l| l.starts_with("object cypress"))
        .unwrap()
        .split(' ')
        .skip(3)
        .take(3)
        .map(|w| &*Box::leak(w.to_string().into_boxed_str()))
        .collect();
    assert_eq!(bits(&objs[0].location().array()), bits(&f32s(&loc)));
    let size = f64s(
        &lines("cypress.txt", "size")[0]
            .split(' ')
            .collect::<Vec<_>>(),
    );
    assert_eq!(artkit::parts::game_size(&objs).to_vec(), size);
    // the baked shading: Blender's levels
    shade::bake_ao(&objs, None, true);
    let ao = objs[0].read(|o| o.mesh.ao.clone().unwrap());
    let want: Vec<i64> = last.lines["ao"]
        .split(' ')
        .map(|l| l.parse().unwrap())
        .collect();
    let (mut off, mut sum, mut far_off) = (0, 0i64, 0i64);
    for (i, a) in ao.iter().enumerate() {
        let d = ((a * 32.0).round() as i64 - want[map[i] as usize]).abs();
        off += (d > 0) as usize;
        sum += d;
        far_off = far_off.max(d);
    }
    assert_eq!((off, sum, far_off), (0, 0, 0), "of {} vertices", ao.len());
}

#[test]
fn rays_meet_the_cypress_where_blenders_did() {
    scene::set_finishes(cypress::finishes());
    scene::reset();
    // (the build, stopped before it is settled: the rays were cast where it was built)
    cypress::cypress();
    let obj = scene::meshes()[0];
    let back = -obj.location();
    obj.edit(|o| o.loc = V3::ZERO);
    let _ = back;
    let rays = Rays::of_scene();
    let (mut n, mut hits) = (0, 0);
    for line in lines("cypress.txt", "ray") {
        let (ray, met) = line.split_once(" | ").unwrap();
        let w: Vec<&str> = ray.split(' ').collect();
        let (o, d) = (f32s(&w[..3]), f32s(&w[3..6]));
        let got = rays.cast(
            V3::new(o[0], o[1], o[2]),
            V3::new(d[0], d[1], d[2]),
            f64w(w[6]),
        );
        let m: Vec<&str> = met.split(' ').collect();
        n += 1;
        if m[0] == "0" {
            assert!(
                got.is_none(),
                "ray {n}: met something, Blender's met nothing"
            );
            continue;
        }
        let hit = got.unwrap_or_else(|| panic!("ray {n}: met nothing"));
        assert_eq!(
            bits(&hit.at.array()),
            bits(&f32s(&m[1..4])),
            "ray {n}: where"
        );
        assert_eq!(
            bits(&hit.normal.array()),
            bits(&f32s(&m[4..7])),
            "ray {n}: the face's normal"
        );
        hits += 1;
    }
    assert!(n >= 42 && hits >= 30, "{hits} of {n} rays met it");
}

#[test]
fn the_far_cypress_is_blenders_and_two_builds_are_the_same_bytes() {
    scene::set_finishes(cypress::finishes());
    let family = cypress::family();
    let model = &family.models[0];
    let objs = make_far(model, model.far.unwrap()).unwrap();
    let mesh = objs[0].read(|o| o.mesh.clone());
    let golden = &golden_meshes("cypress.far.scene.txt")[0];
    let (map, inexact, _) =
        pair_points(&mesh.verts, &golden.verts, 0.0).expect("the far cypress's points");
    assert_eq!(inexact, 0);
    let faces: Vec<Vec<u32>> = (0..mesh.faces.len())
        .map(|f| mesh.corners(f).to_vec())
        .collect();
    assert_eq!(sorted(&renamed(&faces, &map)), sorted(&golden.faces));
    assert!(scene::with(|s| s.perches.is_empty()));
    // the whole build twice: the same files and the same entry
    let a = build_model(&family, model).unwrap();
    let b = build_model(&family, model).unwrap();
    assert!(a.glb == b.glb && a.far == b.far && a.entry == b.entry);
    assert_eq!(&a.glb[..4], b"glTF");
    assert!(a.far.is_some());
    assert_eq!(
        a.entry.dumps().split("\"perches\"").next().unwrap(),
        r#"{"url": "../../models/trees/cypress.glb", "materials": {"bark": {"finish": "material.finish.bark"}, "leaf": {"finish": "material.finish.foliage"}, "leaf2": {"finish": "material.finish.foliage"}, "leaf3": {"finish": "material.finish.foliage"}, "leaf4": {"finish": "material.finish.foliage"}}, "#
    );
    assert_eq!(
        a.entry.get("habitat").and_then(|h| h.str()),
        Some("conifer")
    );
    assert_eq!(
        a.entry.get("far").and_then(|h| h.str()),
        Some("../../models/trees/cypress.far.glb")
    );
}

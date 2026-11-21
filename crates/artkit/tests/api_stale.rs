//! `Obj::matrix_world_seen`: the matrix Blender's `obj.matrix_world` answers a script with, which is the part's
//! matrix as of the last time Blender worked the scene out, not as the part stands now. tests/golden/api_stale.txt
//! is Blender 5.2's answer after each thing a script does (two parts, the second just made and placed: is each
//! one's matrix up to date before, and after?); the kernel keeps the same account.

mod common;

use artkit::forms::mirrored;
use artkit::materials::Mat;
use artkit::math::{Axis, M4, v3};
use artkit::mesh::Mesh;
use artkit::motion::attach;
use artkit::parts::{aside, hinged, join, measure, settle_all, turn};
use artkit::scene::{self, Obj, meshes};
use artkit::{cube, extrude, find, jitter, loft, revolve, ring, rod, soft_box, sweep, swing};
use common::golden;

const BODY: Mat = Mat::new("body", "#F3F3F1", "gloss");

/// Whether the matrix Blender would answer with is the part's as it stands: `fresh`, `STALE`, or `gone`.
fn state(o: Obj) -> &'static str {
    if !o.is_alive() {
        return "gone";
    }
    let (seen, now) = (o.matrix_world_seen().rows(), o.matrix_world().rows());
    let off = seen
        .iter()
        .zip(&now)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0_f32, f32::max);
    if off < 1e-5 { "fresh" } else { "STALE" }
}

fn lathe(name: &str, at: (f64, f64, f64)) -> Obj {
    revolve!(name, &[(0.0, 0.0), (0.1, 0.0), (0.0, 0.1)], at = at)
}

#[test]
fn a_parts_matrix_is_as_blender_last_worked_it_out() {
    let mut checked = 0;
    for line in golden("api_stale.txt").lines() {
        let w: Vec<&str> = line.split('|').collect();
        let name = w[0];
        // (the kernel has no boolean cut)
        if name == "G.cut(a, b)" {
            continue;
        }
        scene::reset();
        let a = cube!(
            "a",
            0.2,
            at = (1, 2, 3),
            material = BODY,
            rot = (10, 20, 30)
        );
        let b = match name {
            "b = revolve (made last)" => lathe("b", (4.0, 5.0, 6.0)),
            "b = rod" => rod!("b", (0, 0, 0), (1, 1, 1), 0.02),
            "b = soft_box" => soft_box!("b", (0.4, 0.4, 0.1), at = (4, 5, 6)),
            "b = extrude" => extrude!(
                "b",
                &[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)],
                0.1,
                at = (4, 5, 6)
            ),
            "b = loft" => {
                let rings: Vec<_> = [0.0, 1.0]
                    .iter()
                    .map(|&z| ring!(0.1, 0.1, z = z, verts = 8))
                    .collect();
                loft!("b", &rings, at = (4, 5, 6))
            }
            // a mesh the script makes itself: no operator runs, and no part's matrix is worked out
            "b = raw objects.new" => scene::add("b", Mesh::default()).set_location((4, 5, 6)),
            _ => cube!("b", 0.2, at = (4, 5, 6), material = BODY),
        };
        let before = (state(a), state(b));
        match name {
            "nothing" | "select_set + active" => {}
            // an operator, an update asked for, the scene evaluated, a frame set, a ray cast: all work it out
            "bpy.ops.object.select_all"
            | "view_layer.update"
            | "evaluated_depsgraph_get"
            | "depsgraph.update"
            | "scene.frame_set(1)"
            | "scene.ray_cast" => scene::update(),
            "G.measure" => {
                measure(&meshes());
            }
            "G.settle" => {
                settle_all();
            }
            "mesh.transform + update" => {
                b.edit(|o| o.mesh.transform(&M4::translation(v3(1, 0, 0))))
            }
            "b.matrix_world = M" => {
                b.set_matrix_world(M4::translation(v3(7, 8, 9)) * M4::rotation(0.3, Axis::Z));
            }
            "b.location after update" => {
                scene::update();
                b.set_location((9, 9, 9));
            }
            "G.jitter(b)" => {
                jitter!(b, 0.001);
            }
            "G.hinged(b) (set_origin)" => {
                hinged(b, (0, 0, 0));
            }
            "G.join([a]) (one part)" => {
                join(&[a], "j", None);
            }
            "G.join([a, b])" => {
                join(&[a, b], "j", None);
            }
            "G.join(origin)" => {
                join(&[a], "j", Some((0.5, 0.5, 0.5).into()));
            }
            "G.swing(b)" => {
                swing!(b, "open", 30);
            }
            "G.attach(b, a)" => {
                attach(b, a);
            }
            "G.perch.find" => find!(2, &[(v3(1, 2, 3), v3(0, 0, 1))]),
            "new cube after" => {
                cube!("c", 0.1, at = (1, 1, 1));
            }
            "new revolve after" => {
                lathe("c", (1.0, 1.0, 1.0));
            }
            "new sweep after" => {
                sweep!("c", &[(0.0, 0.0, 0.0), (0.0, 0.0, 1.0)], 0.02);
            }
            "G.turn(b)" => {
                turn(b, 30.0, Axis::X);
            }
            "G.aside([b])" => {
                aside(&[b], "x");
            }
            "G.mirrored(b)" => {
                assert_eq!(state(mirrored(b, Axis::X, None)), "fresh");
            }
            made if made.starts_with("b = ") => {}
            other => panic!("no such step: {other}"),
        }
        // (a matrix just given is Blender's answer until the scene is worked out: up to date for this)
        let want = |s: &str| {
            if s == "assigned" {
                "fresh".to_string()
            } else {
                s.to_string()
            }
        };
        assert_eq!(
            (before.0, before.1, state(a), state(b)),
            (&*want(w[1]), &*want(w[2]), &*want(w[3]), &*want(w[4])),
            "{name}"
        );
        checked += 1;
    }
    assert_eq!(checked, 33);
}

#[test]
fn a_stone_is_cut_in_its_own_frame_before_blender_knows_where_it_stands() {
    // ground.py's _stone: a boulder made at a place, its fractures cut at once: the matrix the script reads is
    // still the identity (tests/golden/api_cut.txt has it so), and only an update makes it the stone's own
    scene::reset();
    let stone = artkit::boulder!(
        "rock_side",
        0.25,
        at = (-0.42, -0.22, 0),
        material = BODY,
        seed = 3.7
    );
    assert!(stone.matrix_world_seen().is_identity());
    assert_eq!(stone.matrix_world().location(), v3(-0.42, -0.22, 0));
    stone.set_location_axis(2, 0.1);
    assert!(stone.matrix_world_seen().is_identity());
    scene::update();
    assert_eq!(stone.matrix_world_seen(), stone.matrix_world());
}

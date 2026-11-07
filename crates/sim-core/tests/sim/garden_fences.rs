//! The garden fences, hedges and gates a town file puts up (`LotFile.fences`, `web/src/game/gardenFences.ts`):
//! real fences in their styles, so a resident walking to next door's back garden goes round by the
//! front (or through a gate), never through them; and a town file never puts one on a wall.

use crate::common;

use sim_core::lot::Edge;
use sim_core::{Command, World};

/// Two gardens side by side, the street to the south (z = 12): a picket fence between them from
/// behind the houses' fronts (z = 4..12, as laid out from the street that is the back half), and a
/// hedge along their backs. Ada stands in the left garden at (8.5, 8.5).
fn town(gate: bool) -> String {
    let side = if gate {
        r#"{"line":[10,4,10,6],"style":"picket"},{"line":[10,6,10,7],"style":"picket","gate":true},{"line":[10,7,10,12],"style":"picket"}"#
    } else {
        r#"{"line":[10,4,10,12],"style":"picket"}"#
    };
    format!(
        r#"{{"width":20,"depth":14,
        "plots":[{{"name":"Left","x":0,"z":0,"w":10,"d":12,"entry":[5.5,1.5]}},{{"name":"Right","x":10,"z":0,"w":10,"d":12,"entry":[15.5,1.5],"public":true}}],
        "households":[{{"name":"Player","plot":0,"player":true}}],
        "fences":[{side},{{"line":[0,12,20,12],"style":"hedge"}}],
        "sims":[{{"name":"Ada","x":8.5,"z":8.5,"traits":["cheerful"]}}]}}"#
    )
}

fn world(gate: bool) -> World {
    let mut w = World::from_json(&common::content(), &town(gate), 3).unwrap();
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    w
}

/// Walks Ada to (11.5, 8.5), next door, and returns where she went (each tick's place).
fn walk(w: &mut World) -> Vec<[f32; 2]> {
    w.apply(Command::MoveTo {
        sim: 0,
        x: 11.5,
        z: 8.5,
    })
    .unwrap();
    let mut way = vec![w.sims[0].pos];
    for _ in 0..common::secs(100.0) {
        w.tick_once();
        way.push(w.sims[0].pos);
        if !w.sims[0].is_moving() && (w.sims[0].pos[0] - 11.5).abs() < 0.3 {
            break;
        }
    }
    way
}

#[test]
fn a_garden_fence_is_walked_round_and_its_gate_through() {
    let mut w = world(false);
    // The town's fences, in their styles (the content's `fenceStyles`).
    let style = |id: &str| {
        w.content
            .build
            .fences
            .iter()
            .position(|s| s.id == id)
            .unwrap()
    };
    assert_eq!(w.lot.v_edge(10, 8), Edge::Fence);
    assert_eq!(w.lot.h_edge(3, 12), Edge::Fence);
    let structure: serde_json::Value =
        serde_json::from_str(&sim_core::view::structure_json(&w)).unwrap();
    let fences = structure["fences"].as_array().unwrap();
    assert_eq!(fences.len(), 8 + 20);
    let at = |axis: &str, x: u64, z: u64| {
        fences
            .iter()
            .find(|f| f["axis"] == axis && f["x"] == x && f["z"] == z)
            .unwrap()["style"]
            .as_u64()
            .unwrap_or(0) as usize
    };
    assert_eq!(at("v", 10, 8), style("picket"));
    assert_eq!(at("h", 3, 12), style("hedge"));
    assert!(
        w.lot.room_at(5, 8) == sim_core::lot::OUTDOORS,
        "a fenced garden stays outdoors"
    );

    // Next door, 3 m away across the fence: round its front end (z < 4), never through it.
    let way = walk(&mut w);
    let end = *way.last().unwrap();
    assert!(
        (end[0] - 11.5).abs() < 0.3 && (end[1] - 8.5).abs() < 0.3,
        "got there: {end:?}"
    );
    for p in way.windows(2) {
        let crossed = (p[0][0] - 10.0) * (p[1][0] - 10.0) < 0.0;
        let z = (p[0][1] + p[1][1]) / 2.0;
        assert!(!crossed || z < 4.0, "through the fence at {p:?}");
    }
    assert!(
        way.iter().any(|p| p[1] < 4.0),
        "round by the front: {:?}",
        way.iter().map(|p| p[1]).fold(f32::MAX, f32::min)
    );

    // With a gate at z 6..7: through it, the short way.
    let mut w = world(true);
    assert_eq!(w.lot.v_edge(10, 6), Edge::Gate);
    let way = walk(&mut w);
    assert!(way.iter().all(|p| p[1] > 5.0), "through the gate");
    assert!((way.last().unwrap()[0] - 11.5).abs() < 0.3);
}

#[test]
fn a_town_file_never_puts_a_fence_on_a_wall() {
    let file = town(false).replace(r#""fences":["#, r#""walls":[[10,8,10,10]],"fences":["#);
    let err = World::from_json(&common::content(), &file, 3)
        .err()
        .expect("a fence on a wall is refused");
    assert!(err.to_string().contains("on a wall"), "{err}");
    // Nor off the ground storey.
    let file = town(false).replace(r#"[0,12,20,12]"#, r#"[0,15,20,15]"#);
    assert!(World::from_json(&common::content(), &file, 3).is_err());
}

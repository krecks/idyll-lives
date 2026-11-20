use super::super::glb::tests::glb;
use super::*;

/// A part named `name` on `material`, nothing in it yet.
fn part(name: &str, material: &str) -> Part {
    Part {
        mesh: Mesh {
            part: name.into(),
            material: material.into(),
            finish: material.into(),
            edged: edged(material),
            opaque: true,
            ..Mesh::default()
        },
        ..Part::default()
    }
}

/// A flat face of `corners` (three or four, round its edge) in `colour`, wound to look `out`.
fn face(part: &mut Part, corners: &[V3], out: V3, colour: [u8; 3]) {
    let (a, b, c) = (corners[0], corners[1], corners[2]);
    let (e1, e2) = (
        [0, 1, 2].map(|k| b[k] - a[k]),
        [0, 1, 2].map(|k| c[k] - a[k]),
    );
    let n = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    let turned = dot(n, out) < 0.0;
    let l = dot(n, n).sqrt() * if turned { -1.0 } else { 1.0 };
    let first = part.mesh.positions.len() as u32;
    let order: Vec<V3> = if turned {
        corners.iter().rev().copied().collect()
    } else {
        corners.to_vec()
    };
    for p in &order {
        part.mesh.positions.push(*p);
        part.mesh.normals.push(n.map(|c| c / l));
        part.mesh.uvs.push([p[0] + p[2], p[1]]);
        part.colors.push([colour[0], colour[1], colour[2], 255]);
    }
    for k in 1..order.len() as u32 - 1 {
        part.mesh.triangles.push([first, first + k, first + k + 1]);
    }
}

/// A box from `lo` to `hi`, every face but those `skip` names (`py`: its top, `ny`: its underside).
fn boxed(part: &mut Part, lo: V3, hi: V3, colour: [u8; 3], skip: &str) {
    let at = |x: bool, y: bool, z: bool| {
        [
            if x { hi[0] } else { lo[0] },
            if y { hi[1] } else { lo[1] },
            if z { hi[2] } else { lo[2] },
        ]
    };
    for (name, out, corners) in [
        (
            "py",
            [0.0, 1.0, 0.0],
            [
                at(false, true, false),
                at(true, true, false),
                at(true, true, true),
                at(false, true, true),
            ],
        ),
        (
            "ny",
            [0.0, -1.0, 0.0],
            [
                at(false, false, false),
                at(true, false, false),
                at(true, false, true),
                at(false, false, true),
            ],
        ),
        (
            "px",
            [1.0, 0.0, 0.0],
            [
                at(true, false, false),
                at(true, true, false),
                at(true, true, true),
                at(true, false, true),
            ],
        ),
        (
            "nx",
            [-1.0, 0.0, 0.0],
            [
                at(false, false, false),
                at(false, true, false),
                at(false, true, true),
                at(false, false, true),
            ],
        ),
        (
            "pz",
            [0.0, 0.0, 1.0],
            [
                at(false, false, true),
                at(true, false, true),
                at(true, true, true),
                at(false, true, true),
            ],
        ),
        (
            "nz",
            [0.0, 0.0, -1.0],
            [
                at(false, false, false),
                at(true, false, false),
                at(true, true, false),
                at(false, true, false),
            ],
        ),
    ] {
        if !skip.contains(name) {
            face(part, &corners, out, colour);
        }
    }
}

const WALL: [u8; 3] = [200, 190, 170];
const TILES: [u8; 3] = [120, 60, 50];

/// A building 10 m by 8 m, its walls 2.8 m high, under a `roof` of the game's pitch (its ridge along x), all in
/// the plain finish.
fn building(roof: Roof) -> Vec<Part> {
    let mut walls = part("walls", "built.plain");
    boxed(&mut walls, [0.0, 0.0, 0.0], [10.0, 2.8, 8.0], WALL, "py ny");
    let mut top = part("roof", "built.plain");
    let rise = 4.0 * 0.6745;
    let (e0, e1, r) = (2.8, 2.8 + rise, 4.0);
    match roof {
        Roof::Flat => boxed(&mut top, [-0.1, 2.8, -0.1], [10.1, 3.0, 8.1], TILES, "ny"),
        Roof::Gable => {
            face(
                &mut top,
                &[[0.0, e0, 0.0], [10.0, e0, 0.0], [10.0, e1, r], [0.0, e1, r]],
                [0.0, 1.0, -1.0],
                TILES,
            );
            face(
                &mut top,
                &[[0.0, e0, 8.0], [10.0, e0, 8.0], [10.0, e1, r], [0.0, e1, r]],
                [0.0, 1.0, 1.0],
                TILES,
            );
            face(
                &mut walls,
                &[[0.0, e0, 0.0], [0.0, e0, 8.0], [0.0, e1, r]],
                [-1.0, 0.0, 0.0],
                WALL,
            );
            face(
                &mut walls,
                &[[10.0, e0, 0.0], [10.0, e0, 8.0], [10.0, e1, r]],
                [1.0, 0.0, 0.0],
                WALL,
            );
        }
        Roof::Hip => {
            face(
                &mut top,
                &[[0.0, e0, 0.0], [10.0, e0, 0.0], [6.0, e1, r], [4.0, e1, r]],
                [0.0, 1.0, -1.0],
                TILES,
            );
            face(
                &mut top,
                &[[0.0, e0, 8.0], [10.0, e0, 8.0], [6.0, e1, r], [4.0, e1, r]],
                [0.0, 1.0, 1.0],
                TILES,
            );
            face(
                &mut top,
                &[[0.0, e0, 0.0], [0.0, e0, 8.0], [4.0, e1, r]],
                [-1.0, 1.0, 0.0],
                TILES,
            );
            face(
                &mut top,
                &[[10.0, e0, 0.0], [10.0, e0, 8.0], [6.0, e1, r]],
                [1.0, 1.0, 0.0],
                TILES,
            );
        }
    }
    vec![walls, top]
}

fn looked(parts: &[Part]) -> Look {
    look(parts, &Surfaces::new(), &mut Pictures::default())
}

#[test]
fn the_play_camera_s_views_make_the_eight_ways_the_cutaway_stands() {
    let mut seen = std::collections::BTreeSet::new();
    for (x, z) in [(1.0, 0.0), (2.0, 1.0), (1.0, 2.0)] {
        for (x, z) in [(x, z), (-z, x), (-x, -z), (z, -x)] {
            seen.insert(look_bits([x, 1.5, z]));
        }
    }
    assert_eq!(
        seen.into_iter().collect::<Vec<_>>(),
        [1, 2, 4, 5, 6, 8, 9, 10]
    );
    // A camera out along +x looks along -x (bit 2); one out along the dump's +z, the game's -z, looks along +z (4).
    assert_eq!(look_bits([1.0, 0.3, 0.0]), 2);
    assert_eq!(look_bits([0.0, 0.3, 1.0]), 4);
}

#[test]
fn the_cutaway_drops_a_wall_to_its_stub_and_takes_what_goes_with_it() {
    let index = Index {
        storey: 2.8,
        stub: 0.35,
        samples: Vec::new(),
    };
    let mut wall = part("wallsExterior", "built.plain");
    boxed(&mut wall, [0.0, 2.8, 0.0], [4.0, 5.6, 0.14], WALL, "");
    wall.storey = 1;
    // (Drops when the camera looks along +x or +z: mask 5 on both.)
    wall.cut = vec![[5, 1, 5, 0]; wall.mesh.positions.len()];
    let mut lamp = part("trim", "built.plain");
    boxed(&mut lamp, [1.0, 4.0, 0.14], [1.2, 4.2, 0.3], WALL, "");
    lamp.storey = 1;
    lamp.cut = vec![[4, 2, 4, 0]; lamp.mesh.positions.len()];
    let mut roof = part("roof", "material.roof");
    boxed(&mut roof, [0.0, 5.6, 0.0], [4.0, 5.8, 4.0], TILES, "");
    (roof.storey, roof.roof) = (1, true);
    let mut lid = part("ceilingShadow", "built.plain");
    boxed(&mut lid, [0.0, 5.5, 0.0], [4.0, 5.6, 4.0], WALL, "");
    lid.shadow_only = true;
    let parts = [wall, lamp, roof, lid];
    let top = |m: &Mesh| {
        m.positions
            .iter()
            .fold(f64::NEG_INFINITY, |t, p| t.max(p[1]))
    };
    // Standing: the wall and the lamp as built; no roof, nothing of the shadow map's.
    let up = cut_away(&parts, &Surfaces::new(), &index, 1, 0);
    assert_eq!(
        up.iter().map(|m| m.part.as_str()).collect::<Vec<_>>(),
        ["wallsExterior", "trim"]
    );
    assert_eq!((top(&up[0]), up[1].triangles.len()), (5.6, 12));
    // Looking along +x: the wall at its stub over its own storey's floor, the lamp (mask 4) still there.
    let x = cut_away(&parts, &Surfaces::new(), &index, 1, 1);
    assert_eq!((top(&x[0]), x[1].triangles.len()), (2.8 + 0.35, 12));
    // Along +z as well: the lamp goes. Along -x: nothing drops.
    let xz = cut_away(&parts, &Surfaces::new(), &index, 1, 5);
    assert_eq!((top(&xz[0]), xz[1].triangles.len()), (2.8 + 0.35, 0));
    assert_eq!(
        top(&cut_away(&parts, &Surfaces::new(), &index, 1, 2)[0]),
        5.6
    );
    // From the storey below, the one above is hidden.
    assert!(cut_away(&parts, &Surfaces::new(), &index, 0, 0).is_empty());
    // From far off: everything a camera draws.
    assert_eq!(drawn(&parts, &Surfaces::new()).len(), 3);
}

#[test]
fn an_edge_on_the_same_line_in_two_states_counts_once() {
    let edge = |x: f64, length: f64| Edge {
        part: "trim".into(),
        material: "material.trim".into(),
        length,
        at: [x, 1.0, 2.0],
        along: [1.0, 0.0, 0.0],
    };
    // Two sills on one line in one state, more of them showing in another; an edge only the first has.
    let other = Edge {
        at: [0.0, 2.0, 2.0],
        ..edge(0.0, 0.5)
    };
    let mut all = Edges::of(vec![edge(1.0, 0.6), edge(3.0, 0.4), other]);
    assert_eq!((all.count(), all.length()), (3, 1.5));
    all.or(Edges::of(vec![
        edge(1.0, 0.8),
        edge(3.0, 0.8),
        edge(5.0, 0.2),
    ]));
    assert_eq!(all.count(), 4);
    assert!((all.length() - 2.3).abs() < 1e-9);
    all.or(Edges::of(vec![edge(-1.0, 0.1)]));
    assert_eq!(all.count(), 4);
    assert_eq!(all.longest()[0].length, 0.8);
    // The same line run the other way is the same line.
    let back = Edge {
        along: [-1.0, 0.0, 0.0],
        ..edge(2.0, 9.0)
    };
    all.or(Edges::of(vec![back]));
    assert_eq!((all.count(), all.longest()[0].length), (2, 9.0));
}

#[test]
fn a_house_s_edges_are_those_of_every_state_it_is_seen_in() {
    // A room 4 m square under a roof, a chest standing in it: with the roof on nothing of the chest shows.
    let index = Index {
        storey: 2.8,
        stub: 0.35,
        samples: Vec::new(),
    };
    let mut walls = part("wallsExterior", "built.plain");
    for (lo, hi) in [
        ([0.0, 0.0, 0.0], [4.0, 2.8, 0.14]),
        ([0.0, 0.0, 3.86], [4.0, 2.8, 4.0]),
        ([0.0, 0.0, 0.14], [0.14, 2.8, 3.86]),
        ([3.86, 0.0, 0.14], [4.0, 2.8, 3.86]),
    ] {
        boxed(&mut walls, lo, hi, WALL, "ny");
    }
    walls.cut = vec![[15, 1, 15, 0]; walls.mesh.positions.len()];
    let mut roof = part("roof", "material.roof");
    boxed(&mut roof, [-0.3, 2.8, -0.3], [4.3, 3.0, 4.3], TILES, "");
    roof.roof = true;
    let mut chest = part("chest", "built.plain");
    boxed(&mut chest, [1.5, 0.0, 1.5], [2.5, 0.8, 2.5], WALL, "ny");
    let parts = [walls, roof, chest];
    let of = |edges: &Edges, part: &str| edges.0.keys().filter(|line| line.0 == part).count();
    let whole = Edges::of(edges::hard_edges(&drawn(&parts, &Surfaces::new())));
    assert_eq!(of(&whole, "chest"), 0);
    assert!(of(&whole, "roof") > 0);
    let house = house_edges(&parts, &Surfaces::new(), &index, 1);
    // Its top's four edges from above, its uprights once the walls on the camera's side are down.
    assert_eq!(
        of(&house, "chest"),
        8,
        "{:?}",
        house.0.keys().collect::<Vec<_>>()
    );
    assert_eq!(of(&house, "roof"), of(&whole, "roof"));
    // The walls' stubs have edges of their own, 0.35 m up.
    assert!(
        house
            .0
            .keys()
            .any(|line| line.0 == "wallsExterior" && line.3[1] == 350)
    );
}

#[test]
fn a_building_s_look_is_measured_from_its_meshes() {
    let gable = looked(&building(Roof::Gable));
    assert_eq!(gable.roof, Roof::Gable);
    assert!((gable.pitch - 34.0).abs() < 0.5, "{}", gable.pitch);
    assert!(
        (gable.top - (2.8 + 4.0 * 0.6745)).abs() < 0.05,
        "{}",
        gable.top
    );
    assert!((gable.area - 80.0).abs() < 2.0, "{}", gable.area);
    assert!(
        (gable.plan[2] - gable.plan[0] - 10.0).abs() < 0.2
            && (gable.plan[3] - gable.plan[1] - 8.0).abs() < 0.2
    );
    assert_eq!(
        (gable.wall, gable.roof_colour, gable.chimneys),
        (Some(WALL), Some(TILES), 0)
    );
    let hip = looked(&building(Roof::Hip));
    assert_eq!(
        (hip.roof, hip.wall, hip.roof_colour),
        (Roof::Hip, Some(WALL), Some(TILES))
    );
    assert!((hip.pitch - 34.0).abs() < 0.5, "{}", hip.pitch);
    let flat = looked(&building(Roof::Flat));
    assert_eq!((flat.roof, flat.pitch), (Roof::Flat, 0.0));
    assert!((flat.top - 3.0).abs() < 1e-9);
    // A stack through the roof: its brick over the ridge, a cap on it.
    let mut stacked = building(Roof::Gable);
    let mut stack = part("brick", "built.plain");
    boxed(
        &mut stack,
        [2.0, 3.0, 3.2],
        [2.6, 6.3, 3.96],
        [150, 80, 60],
        "ny",
    );
    stacked.push(stack);
    let with = looked(&stacked);
    assert_eq!((with.chimneys, with.roof), (1, Roof::Gable));
    assert!((with.top - gable.top).abs() < 1e-9);
    // Glass and what only the shadow map draws are no wall's colour.
    let mut glazed = building(Roof::Gable);
    let mut panes = part("windows", "material.window");
    boxed(
        &mut panes,
        [-0.01, 0.5, -0.01],
        [10.01, 2.6, 8.01],
        [10, 20, 30],
        "py ny",
    );
    glazed.push(panes);
    assert_eq!(looked(&glazed).wall, Some(WALL));
}

#[test]
fn a_far_version_disagrees_where_it_is_off_its_near_one() {
    let near = looked(&building(Roof::Gable));
    assert_eq!(disagreements(&near, &near), Vec::<String>::new());
    // A hip for a gable, a stack the near one hasn't, walls another paint.
    let mut far = building(Roof::Hip);
    far[0].colors.fill([240, 240, 235, 255]);
    let mut stack = part("brick", "built.plain");
    boxed(
        &mut stack,
        [2.0, 3.0, 3.2],
        [2.6, 6.3, 3.96],
        [150, 80, 60],
        "ny",
    );
    far.push(stack);
    let found = disagreements(&near, &looked(&far));
    assert_eq!(found.len(), 3, "{found:?}");
    assert_eq!(found[0], "roof: a hip roof, near a gable roof");
    assert_eq!(found[1], "chimney: it has one, near none");
    assert_eq!(
        found[2],
        "walls: most of them are #F0F0EB as drawn, near #C8BEAA (a channel at most 28 of 255 apart)"
    );
    // Twice as long, a storey taller, another roof colour: its plan, its height and its roof's colour.
    let mut tall = building(Roof::Gable);
    for p in tall.iter_mut() {
        for at in &mut p.mesh.positions {
            *at = [
                at[0] * 2.0,
                at[1] + if at[1] > 0.0 { 2.8 } else { 0.0 },
                at[2],
            ];
        }
    }
    tall[1].colors.fill([60, 70, 90, 255]);
    let found = disagreements(&near, &looked(&tall));
    assert_eq!(
        found
            .iter()
            .map(|f| f.split(':').next().unwrap())
            .collect::<Vec<_>>(),
        ["plan", "height", "roof colour"],
        "{found:?}"
    );
    // Within the limits: eaves a little wider, a shade off.
    let mut close = building(Roof::Gable);
    for p in close.iter_mut() {
        for at in &mut p.mesh.positions {
            at[0] *= 1.03;
        }
        for c in &mut p.colors {
            *c = [c[0] - 10, c[1] - 10, c[2] - 10, 255];
        }
    }
    assert_eq!(disagreements(&near, &looked(&close)), Vec::<String>::new());
}

#[test]
fn a_house_s_detail_is_what_its_meshes_have() {
    let mut trim = part("trim", "material.trim");
    boxed(&mut trim, [0.0, 0.0, 0.0], [0.1, 2.0, 0.1], WALL, "ny");
    let plain = detail(&[trim.clone()], &Surfaces::new());
    assert!((plain.trim - (4.0 * 0.2 + 0.01)).abs() < 1e-9);
    assert_eq!(
        (plain.trim_shaped, plain.eased, plain.eased_upright),
        (0.0, 0.0, 0.0)
    );
    // A chamfer down one corner: a face neither plumb nor level it isn't, but two eased edges it has.
    face(
        &mut trim,
        &[
            [0.1, 0.0, 0.09],
            [0.11, 0.0, 0.1],
            [0.11, 2.0, 0.1],
            [0.1, 2.0, 0.09],
        ],
        [1.0, 0.0, -1.0],
        WALL,
    );
    // A weathered sill: a face that slopes.
    face(
        &mut trim,
        &[
            [0.0, 2.0, 0.0],
            [0.5, 2.0, 0.0],
            [0.5, 1.9, 0.2],
            [0.0, 1.9, 0.2],
        ],
        [0.0, 1.0, 0.5],
        WALL,
    );
    let shaped = detail(&[trim], &Surfaces::new());
    assert!(
        (shaped.trim_shaped - 0.5 * (0.05_f64).sqrt()).abs() < 1e-9,
        "{}",
        shaped.trim_shaped
    );
    // A hip roof: its ridge, its four hips, no cap shingles; with caps in another tone, their area.
    let mut roof = building(Roof::Hip).remove(1);
    (roof.mesh.material, roof.roof) = ("material.roof".into(), true);
    let bare = detail(&[roof.clone()], &Surfaces::new());
    let hip = (16.0_f64 + 16.0 + (4.0_f64 * 0.6745).powi(2)).sqrt();
    assert!(
        (bare.roofs[&0][0] - 2.0).abs() < 1e-6 && (bare.roofs[&0][1] - 4.0 * hip).abs() < 1e-6,
        "{:?}",
        bare.roofs
    );
    assert_eq!(bare.roofs[&0][2], 0.0);
    face(
        &mut roof,
        &[
            [4.0, 5.6, 3.9],
            [6.0, 5.6, 3.9],
            [6.0, 5.6, 4.1],
            [4.0, 5.6, 4.1],
        ],
        [0.0, 1.0, 0.0],
        [100, 50, 40],
    );
    let capped = detail(&[roof], &Surfaces::new());
    assert!((capped.roofs[&0][2] - 0.4).abs() < 1e-9);
    assert!((capped.roofs[&0][1] - 4.0 * hip).abs() < 1e-6);
    // A fascia at the eaves and the slopes on it; then a slope that dives 1.8 m under it.
    let mut fascia = part("roofTrim", "material.trim");
    let board = [
        [0.0, 2.8, 0.0],
        [10.0, 2.8, 0.0],
        [10.0, 2.94, 0.0],
        [0.0, 2.94, 0.0],
    ];
    face(&mut fascia, &board, [0.0, 0.0, -1.0], WALL);
    fascia.roof = true;
    let mut slopes = building(Roof::Hip).remove(1);
    (slopes.mesh.material, slopes.roof) = ("material.roof".into(), true);
    let sound = detail(&[fascia.clone(), slopes.clone()], &Surfaces::new());
    assert_eq!(sound.roofs[&0][3], 0.0);
    let dive = [[0.0, 1.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.5, 1.0]];
    face(&mut slopes, &dive, [0.0, 1.0, 0.0], TILES);
    let sunk = detail(&[fascia, slopes], &Surfaces::new());
    assert!((sunk.roofs[&0][3] - 1.8).abs() < 1e-9, "{:?}", sunk.roofs);
}

/// `parts` as the dump writes a sample's file (tools/built/glb.ts).
fn file_of(parts: &[Part]) -> Vec<u8> {
    let mut bin: Vec<u8> = Vec::new();
    let (mut views, mut accessors, mut meshes, mut nodes, mut materials) = (
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::<String>::new(),
    );
    let mut accessor = |bin: &mut Vec<u8>,
                        bytes: Vec<u8>,
                        count: usize,
                        kind: &str,
                        component: u32,
                        normalized: bool| {
        views.push(format!(
            "{{\"buffer\": 0, \"byteOffset\": {}, \"byteLength\": {}}}",
            bin.len(),
            bytes.len()
        ));
        bin.extend(bytes);
        while !bin.len().is_multiple_of(4) {
            bin.push(0);
        }
        accessors.push(format!(
                "{{\"bufferView\": {}, \"componentType\": {component}, \"count\": {count}, \"type\": \"{kind}\"{}}}",
                views.len() - 1,
                if normalized { ", \"normalized\": true" } else { "" }
            ));
        accessors.len() - 1
    };
    for p in parts {
        let floats = |v: Vec<f64>| {
            v.iter()
                .flat_map(|x| (*x as f32).to_le_bytes())
                .collect::<Vec<u8>>()
        };
        let n = p.mesh.positions.len();
        let mut attributes = vec![
            format!(
                "\"POSITION\": {}",
                accessor(
                    &mut bin,
                    floats(p.mesh.positions.concat()),
                    n,
                    "VEC3",
                    5126,
                    false
                )
            ),
            format!(
                "\"NORMAL\": {}",
                accessor(
                    &mut bin,
                    floats(p.mesh.normals.concat()),
                    n,
                    "VEC3",
                    5126,
                    false
                )
            ),
            format!(
                "\"TEXCOORD_0\": {}",
                accessor(
                    &mut bin,
                    floats(p.mesh.uvs.concat()),
                    n,
                    "VEC2",
                    5126,
                    false
                )
            ),
            format!(
                "\"COLOR_0\": {}",
                accessor(&mut bin, p.colors.concat(), n, "VEC4", 5121, true)
            ),
        ];
        if !p.cut.is_empty() {
            attributes.push(format!(
                "\"_CUT\": {}",
                accessor(&mut bin, p.cut.concat(), n, "VEC4", 5121, false)
            ));
        }
        let indices: Vec<u8> = p
            .mesh
            .triangles
            .concat()
            .iter()
            .flat_map(|i| i.to_le_bytes())
            .collect();
        let indices = accessor(
            &mut bin,
            indices,
            p.mesh.triangles.len() * 3,
            "SCALAR",
            5125,
            false,
        );
        let material = materials
            .iter()
            .position(|m| m == &p.mesh.material)
            .unwrap_or_else(|| {
                materials.push(p.mesh.material.clone());
                materials.len() - 1
            });
        meshes.push(format!(
                "{{\"primitives\": [{{\"attributes\": {{{}}}, \"indices\": {indices}, \"material\": {material}}}]}}",
                attributes.join(", ")
            ));
        nodes.push(format!(
                "{{\"name\": \"{}\", \"mesh\": {}, \"extras\": {{\"storey\": {}, \"roof\": {}, \"shadowOnly\": {}}}}}",
                p.mesh.part,
                meshes.len() - 1,
                p.storey,
                p.roof,
                p.shadow_only
            ));
    }
    let materials: Vec<String> = materials
        .iter()
        .map(|m| format!("{{\"name\": \"{m}\"}}"))
        .collect();
    let json = format!(
        "{{\"asset\": {{\"version\": \"2.0\"}}, \"nodes\": [{}], \"meshes\": [{}], \"materials\": [{}], \"accessors\": [{}], \"bufferViews\": [{}], \"buffers\": [{{\"byteLength\": {}}}]}}",
        nodes.join(", "),
        meshes.join(", "),
        materials.join(", "),
        accessors.join(", "),
        views.join(", "),
        bin.len()
    );
    glb(&json, &bin)
}

#[test]
fn a_sample_s_file_reads_back_as_its_parts() {
    let mut wall = part("wallsExterior", "material.wall.brick");
    boxed(&mut wall, [0.0, 0.0, 0.0], [4.0, 2.5, 0.25], WALL, "ny");
    wall.cut = vec![[5, 1, 5, 0]; wall.mesh.positions.len()];
    let mut roof = part("roof", "material.roof");
    boxed(&mut roof, [0.0, 2.5, 0.0], [4.0, 2.75, 4.0], TILES, "");
    (roof.storey, roof.roof) = (1, true);
    let mut glass = part("windows", "material.window.clear");
    boxed(&mut glass, [1.0, 1.0, 0.0], [2.0, 2.0, 0.25], [1, 2, 3], "");
    glass.shadow_only = true;
    let parts = vec![wall, roof, glass];
    let back = read_parts(&file_of(&parts)).unwrap();
    assert_eq!(back.len(), 3);
    for (a, b) in parts.iter().zip(&back) {
        assert_eq!(a.mesh, b.mesh);
        assert_eq!(
            (&a.colors, &a.cut, a.storey, a.roof, a.shadow_only),
            (&b.colors, &b.cut, b.storey, b.roof, b.shadow_only)
        );
    }
    // Glass has no edges to judge; a file that is no .glb is an error.
    assert!(!back[2].mesh.edged && back[0].mesh.edged);
    assert!(read_parts(b"no model").is_err());
}

/// The gate over a dump of two samples, a chest and its far version: the baseline's verdicts, `seeded` writing
/// the sections a baseline has none of, the lowering taking out and never adding.
#[test]
fn the_gate_judges_a_dump_by_the_baseline() {
    let dir = std::env::temp_dir().join(format!("ll-gate-built-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let at = |name: &str| dir.join(name).to_string_lossy().into_owned();
    let mut paths = Paths::repo();
    paths.baseline = at("gate-baseline.json");
    let dump = at("dump");
    std::fs::create_dir_all(&dump).unwrap();
    let mut chest = part("chest", "built.plain");
    boxed(&mut chest, [0.0, 0.0, 0.0], [1.0, 0.8, 1.0], WALL, "ny");
    std::fs::write(join(&dump, "car.chest.glb"), file_of(&[chest])).unwrap();
    std::fs::write(
        join(&dump, "house.box.glb"),
        file_of(&building(Roof::Gable)),
    )
    .unwrap();
    std::fs::write(join(&dump, "far.box.glb"), file_of(&building(Roof::Hip))).unwrap();
    std::fs::write(
            join(&dump, "built.json"),
            r#"{"storey": 2.8, "stub": 0.35, "samples": [
                {"key": "car.chest", "kind": "car", "about": "a chest"},
                {"key": "house.box", "kind": "house", "storeys": 1, "about": "a box"},
                {"key": "far.box", "kind": "far", "of": "house.box", "storeys": 1, "about": "a box"}]}"#,
        )
        .unwrap();
    let set = |text: &str| std::fs::write(&paths.baseline, text).unwrap();
    let run = || gate(&paths, &dump, &[]);
    // No baseline: every finding fails.
    let r = run();
    assert_eq!(r.failed, 3, "{}", r.text);
    assert!(r.text.starts_with("FAIL car.chest                    10 triangles, 1 draws\n       error: 8 hard edges of 5 cm or more show unbevelled at the play camera, 7.2 m"), "{}", r.text);
    assert!(r.text.contains("FAIL far.box                      14 triangles, 2 draws\n       error: roof: a hip roof, near a gable roof\n"), "{}", r.text);
    assert!(r.text.ends_with("3 samples, 3 failed\n"), "{}", r.text);
    assert_eq!(r.counts[EDGES]["car.chest"], 8);
    assert_eq!(
        (r.counts[FAR_NEAR]["far.box"], r.counts[FAR_NEAR].len()),
        (1, 1)
    );
    // (A far version is judged for neither edges nor texels.)
    assert!(!r.counts[EDGES].contains_key("far.box") && r.counts[EDGES].contains_key("house.box"));
    // Seeded: the sections it had none of, of the samples with findings; then it passes, its findings printed.
    let (seed, written) = seeded(&Baseline::default(), &r.counts).unwrap();
    assert_eq!(written.len(), 2, "{written:?}");
    assert_eq!(written[1], "built.farNear: 1 samples, 1 findings");
    seed.write(&paths.baseline).unwrap();
    let r = run();
    assert_eq!(r.failed, 0, "{}", r.text);
    assert!(r.text.contains("ok   car.chest                    10 triangles, 1 draws\n       warning: built.edges: 8 found, as the baseline has (tools/art/gate-baseline.json)\n"), "{}", r.text);
    // A section it has is left as it is; the models' sections too.
    let (again, written) = seeded(&seed, &r.counts).unwrap();
    assert_eq!((again, written), (seed.clone(), Vec::new()));
    // One sample alone, by its key; a key the dump hasn't.
    assert!(
        gate(&paths, &dump, &["car.chest".into()])
            .text
            .ends_with("1 samples, 0 failed\n")
    );
    assert_eq!(gate(&paths, &dump, &["car.none".into()]).failed, 1);
    // Higher than found: stale, with this gate's own command; lowered, never raised.
    set(
        "{\"built.edges\": {\"car.chest\": 9, \"house.box\": 1, \"fence.gone\": 2}, \"built.farNear\": {\"far.box\": 1}, \"edges\": {\"model.x\": 1}}\n",
    );
    let r = run();
    assert_eq!(r.failed, 3, "{}", r.text);
    assert!(r.text.contains("error: built.edges: 8 found, 9 in the baseline: lower it (`cargo check-glb --built <dir> --update-baseline`)"), "{}", r.text);
    assert!(r.text.contains("FAIL gate-baseline.json\n       error: built.edges: fence.gone is no sample this check ran on"), "{}", r.text);
    let (low, changes) = Baseline::read(&paths.baseline).unwrap().lowered(&r.counts);
    assert_eq!(
        changes,
        [
            "built.edges: car.chest 9 -> 8",
            "built.edges: fence.gone out (was 2)"
        ]
    );
    // (Over its count, a sample fails until it is fixed: the baseline is never raised.)
    assert_eq!(
        (low.get(EDGES, "house.box"), low.get("edges", "model.x")),
        (1, 1)
    );
    // A dump that isn't there.
    assert!(
        gate(&paths, &at("none"), &[])
            .text
            .starts_with("FAIL the built world\n       error: cannot read it: ")
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

/// The report has a row for every sample and every pair.
#[test]
fn the_report_lists_what_a_run_measured() {
    let dir = std::env::temp_dir().join(format!("ll-gate-built-report-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let dump = dir.to_string_lossy().into_owned();
    let mut paths = Paths::repo();
    paths.baseline = join(&dump, "gate-baseline.json");
    let mut near = building(Roof::Gable);
    near[0].mesh.material = "material.wall.siding".into();
    std::fs::write(join(&dump, "house.box.glb"), file_of(&near)).unwrap();
    std::fs::write(join(&dump, "far.box.glb"), file_of(&building(Roof::Gable))).unwrap();
    std::fs::write(
            join(&dump, "built.json"),
            r#"{"storey": 2.8, "stub": 0.35, "samples": [{"key": "house.box", "kind": "house", "storeys": 1}, {"key": "far.box", "kind": "far", "of": "house.box"}]}"#,
        )
        .unwrap();
    let text = report(&gate(&paths, &dump, &[]));
    assert!(text.contains("| house.box | 14 | 2 | "), "{text}");
    assert!(text.contains("| far.box | 14 | 2 | — | — | — | "), "{text}");
    assert!(
        text.contains("| house | walls | upright | 4 | 11.2 | "),
        "{text}"
    );
    // The siding as its UVs lay it: the test's are a metre a metre, so the manifest's 0.4 of 1024 px.
    assert!(
        text.contains("| material.wall.siding | 1024 | 0.4 | 1 | "),
        "{text}"
    );
    assert!(text.contains("| house.box | 80 | "), "{text}");
    assert!(text.contains("| far.box | 80 | "), "{text}");
    assert!(
        text.contains("| house.box | 0.0 | 0.00 | 0.0 | 20.0 | 0.00 |"),
        "{text}"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

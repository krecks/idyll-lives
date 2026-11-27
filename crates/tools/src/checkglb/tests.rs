//! Tests for the glb checks.

use super::*;

#[test]
fn centres_one_key_a_line_sorted_to_a_tenth_of_a_millimetre() {
    let mut c = BTreeMap::new();
    c.insert("model.b".to_string(), [0.0, -0.068_96]);
    c.insert("model.a".to_string(), [-0.000_01, 1.234_56]);
    c.insert("model.c".to_string(), [0.5, 2.0]);
    assert_eq!(
        centres_text(&c),
        "{\n  \"model.a\": [0.0, 1.2346],\n  \"model.b\": [0.0, -0.069],\n  \"model.c\": [0.5, 2.0]\n}\n"
    );
}

#[test]
fn the_committed_centres_are_in_that_format() {
    let paths = Paths::repo();
    let text = std::fs::read_to_string(&paths.centres).unwrap();
    let read = read_centres(&paths.centres).unwrap();
    let c: BTreeMap<String, [f64; 2]> = read
        .entries()
        .unwrap()
        .iter()
        .map(|(k, v)| {
            (
                k.clone(),
                [
                    v.at(0).unwrap().num().unwrap().f(),
                    v.at(1).unwrap().num().unwrap().f(),
                ],
            )
        })
        .collect();
    assert_eq!(c.len(), read.entries().unwrap().len(), "a key twice");
    assert_eq!(centres_text(&c), text);
}

/// A model's carried decoration (`<name>_aside`: the picture on a chest of drawers) is drawn but left out of
/// its own size: the chest is 0.45 m tall, the picture stands 0.9 m above it.
#[test]
fn what_a_model_carries_is_drawn_but_not_its_size() {
    let json = r#"{"asset": {"version": "2.0"}, "scenes": [{"nodes": [0, 1]}],
        "nodes": [{"name": "body", "mesh": 0}, {"name": "picture_aside", "mesh": 0, "translation": [0, 0.9, 0]}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0}}]}],
        "bufferViews": [{"byteOffset": 0, "byteLength": 36}],
        "accessors": [{"bufferView": 0, "componentType": 5126, "type": "VEC3", "count": 3, "min": [0, 0, 0], "max": [0.5, 0.45, 0.5]}]}"#;
    let bin: Vec<u8> = [0f32, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.5]
        .iter()
        .flat_map(|c| c.to_le_bytes())
        .collect();
    let g = parse_glb(&glb::tests::glb(json, &bin)).unwrap();
    let aside = aside_nodes(&g.gltf);
    assert_eq!(aside, HashSet::from([1]), "the picture's node");
    let ((_, drawn), _) = bounds(&g.gltf, &HashMap::new(), &HashSet::new()).unwrap();
    assert!((drawn[1].f() - 1.35).abs() < 1e-6, "drawn {drawn:?}");
    let ((lo, own), _) = bounds(&g.gltf, &HashMap::new(), &aside).unwrap();
    assert!(lo[1].f().abs() < 1e-6, "own {lo:?}");
    assert!((own[1].f() - 0.45).abs() < 1e-6, "own {own:?}");
}

/// A one-key-a-frame turn of `degrees` about y over 1/30 s, its first key at `start` seconds.
fn turning_clip(start: f32, degrees: f32) -> Glb {
    let json = r#"{"asset": {"version": "2.0"}, "nodes": [{"name": "door"}],
        "animations": [{"name": "open", "channels": [{"sampler": 0, "target": {"node": 0, "path": "rotation"}}],
            "samplers": [{"input": 0, "output": 1}]}],
        "bufferViews": [{"byteOffset": 0, "byteLength": 8}, {"byteOffset": 8, "byteLength": 32}],
        "accessors": [{"bufferView": 0, "componentType": 5126, "type": "SCALAR", "count": 2},
            {"bufferView": 1, "componentType": 5126, "type": "VEC4", "count": 2}]}"#;
    let half = (degrees / 2.0).to_radians();
    let bin: Vec<u8> = [
        start,
        start + 1.0 / 30.0,
        0.0,
        0.0,
        0.0,
        1.0,
        0.0,
        half.sin(),
        0.0,
        half.cos(),
    ]
    .iter()
    .flat_map(|c| c.to_le_bytes())
    .collect();
    parse_glb(&glb::tests::glb(json, &bin)).unwrap()
}

/// A clip that starts after 0 s is held at rest by the game until then: every loop paused a frame each round when
/// clips started at 1/24 s.
#[test]
fn a_clip_starts_at_0_s() {
    let late = |start| {
        let (errors, _) = check_clips(&turning_clip(start, 5.0), &J::Obj(Vec::new())).unwrap();
        errors
            .iter()
            .any(|e| e.contains("s in: the game holds it at rest"))
    };
    assert!(late(1.0 / 24.0), "a frame late");
    assert!(!late(0.0), "on time");
}

/// The game blends a clip's samples 1/60 s apart, which shrinks a part turning far between them.
#[test]
fn a_part_turning_too_fast_is_warned_of() {
    let fast = |degrees| {
        let (_, warnings) = check_clips(&turning_clip(0.0, degrees), &J::Obj(Vec::new())).unwrap();
        warnings.iter().any(|w| w.contains("in 1/60 s"))
    };
    assert!(fast(30.0), "30° in a frame (15° in 1/60 s)");
    assert!(!fast(24.0), "two turns a second");
}

/// A loop that ends a part moved at an even speed is a scroll only if the part looks the same moved: a belt of six
/// slats 5 cm apart moved one or two slats on passes, moved half a slat or a lone part moved 30 cm fails.
#[test]
fn a_scroll_moves_by_whole_repeats() {
    let belt = |slats: usize, moved: f32| {
        let mut bin: Vec<u8> = (0..slats)
            .flat_map(|k| {
                let x = k as f32 * 0.05;
                [[x, 0.0, 0.0], [x + 0.03, 0.0, 0.0], [x, 0.0, 0.4]]
            })
            .flatten()
            .flat_map(|c| c.to_le_bytes())
            .collect();
        let at = bin.len();
        for c in [
            0.0f32,
            0.5,
            1.0,
            0.0,
            0.0,
            0.0,
            moved / 2.0,
            0.0,
            0.0,
            moved,
            0.0,
            0.0,
        ] {
            bin.extend(c.to_le_bytes());
        }
        let json = format!(
            r#"{{"asset": {{"version": "2.0"}}, "scenes": [{{"nodes": [0]}}], "nodes": [{{"name": "slats", "mesh": 0}}],
            "meshes": [{{"primitives": [{{"attributes": {{"POSITION": 0}}}}]}}],
            "bufferViews": [{{"byteOffset": 0, "byteLength": {at}}}, {{"byteOffset": {at}, "byteLength": 12}}, {{"byteOffset": {}, "byteLength": 36}}],
            "accessors": [{{"bufferView": 0, "componentType": 5126, "type": "VEC3", "count": {}, "min": [0, 0, 0], "max": [1, 0, 0.4]}},
                {{"bufferView": 1, "componentType": 5126, "type": "SCALAR", "count": 3}}, {{"bufferView": 2, "componentType": 5126, "type": "VEC3", "count": 3}}],
            "animations": [{{"name": "belt_run", "channels": [{{"sampler": 0, "target": {{"node": 0, "path": "translation"}}}}], "samplers": [{{"input": 1, "output": 2}}]}}]}}"#,
            at + 12,
            slats * 3
        );
        let g = parse_glb(&glb::tests::glb(&json, &bin)).unwrap();
        let entry = json::loads_text(br#"{"motions": {"loops": {"run": "belt_run"}}}"#).unwrap();
        let (errors, warnings) = check_clips(&g, &entry).unwrap();
        assert!(warnings.iter().all(|w| !w.contains("loop")), "{warnings:?}");
        errors.iter().all(|e| !e.contains("loop"))
    };
    assert!(belt(6, 0.05), "one slat on");
    assert!(belt(6, 0.10), "two");
    assert!(!belt(6, 0.025), "half a slat");
    assert!(!belt(1, 0.3), "a jump");
}

/// A face whose normals point against its winding is lit from behind (a bevel's arc miter did it to a frame).
#[test]
fn faces_lit_from_behind_are_found() {
    let flipped = |nz: f32| {
        let json = r#"{"asset": {"version": "2.0"}, "nodes": [{"name": "body", "mesh": 0}],
            "meshes": [{"name": "body", "primitives": [{"attributes": {"POSITION": 0, "NORMAL": 1}}]}],
            "bufferViews": [{"byteOffset": 0, "byteLength": 36}, {"byteOffset": 36, "byteLength": 36}],
            "accessors": [{"bufferView": 0, "componentType": 5126, "type": "VEC3", "count": 3, "min": [0, 0, 0], "max": [0.1, 0.1, 0]},
                {"bufferView": 1, "componentType": 5126, "type": "VEC3", "count": 3}]}"#;
        let bin: Vec<u8> = [0f32, 0.0, 0.0, 0.1, 0.0, 0.0, 0.0, 0.1, 0.0]
            .iter()
            .chain([0f32, 0.0, nz].iter().cycle().take(9))
            .flat_map(|c| c.to_le_bytes())
            .collect();
        flipped_faces(&parse_glb(&glb::tests::glb(json, &bin)).unwrap()).unwrap()
    };
    assert_eq!(
        flipped(-1.0),
        vec![("body".to_string(), 1)],
        "normals down, winding up"
    );
    assert!(flipped(1.0).is_empty(), "both up");
}

/// A model may allow itself more draw calls than the four (the grandfather clock's trunk door is a fifth), and
/// only a whole number of them: nothing else goes.
#[test]
fn a_model_may_allow_itself_more_draws() {
    let allowed = |json: &str| draws_allowed(&json::loads_text(json.as_bytes()).unwrap());
    assert_eq!(
        allowed(r#"{"type": "model"}"#),
        Ok(DRAWS.1),
        "four by default"
    );
    assert_eq!(allowed(r#"{"draws": 5}"#), Ok(5));
    assert_eq!(allowed(r#"{"draws": 3}"#), Ok(3));
    assert!(allowed(r#"{"draws": 2}"#).is_err(), "fewer than the aim");
    assert!(allowed(r#"{"draws": 9}"#).is_err(), "past the eight");
    assert!(allowed(r#"{"draws": 4.5}"#).is_err(), "a whole number");
    assert!(allowed(r#"{"draws": "5"}"#).is_err(), "a number");
}

/// The aim of three is the still model's: each moving part, the glass and metal hardware alone on a still body
/// are a draw more; two finishes on one moving part, or a still body's paint, are not.
#[test]
fn moving_parts_glass_and_hardware_each_add_a_draw_to_the_aim() {
    let glass = |f: &str| f == "material.finish.glass";
    let groups = |g: &[(Option<usize>, &str)]| -> BTreeSet<(Option<usize>, String)> {
        g.iter().map(|(p, f)| (*p, f.to_string())).collect()
    };
    let plain = |f: &[&str]| -> BTreeSet<String> { f.iter().map(|s| s.to_string()).collect() };
    let wood = "material.finish.wood";
    let fabric = "material.finish.fabric";
    let aim = |g: &[(Option<usize>, &str)], p: &[&str], declared| {
        draws_aimed(&groups(g), &glass, &plain(p), declared)
    };
    let still = [(None, "plain"), (None, wood), (None, fabric)];
    assert_eq!(aim(&still, &["material.finish.satin"], None), 3);
    assert_eq!(
        aim(
            &[&still[..], &[(Some(4), wood)]].concat(),
            &["material.finish.satin"],
            None
        ),
        4,
        "a door"
    );
    assert_eq!(
        aim(
            &[&still[..], &[(Some(4), wood), (Some(4), fabric)]].concat(),
            &[],
            None
        ),
        4,
        "a door's second finish is not"
    );
    assert_eq!(
        aim(
            &[
                &still[..],
                &[(Some(4), "plain"), (Some(4), "material.finish.glass")]
            ]
            .concat(),
            &[],
            None
        ),
        5,
        "a glass door"
    );
    assert_eq!(
        aim(
            &[&still[..], &[(None, "material.finish.glass")]].concat(),
            &[],
            None
        ),
        4,
        "glass"
    );
    assert_eq!(
        aim(
            &still,
            &["material.finish.metal", "material.finish.chrome"],
            None
        ),
        4,
        "metal hardware alone"
    );
    assert_eq!(
        aim(
            &still,
            &["material.finish.metal", "material.finish.matte"],
            None
        ),
        3,
        "hardware with paint"
    );
    assert_eq!(aim(&still, &[], Some(5)), 5, "the entry's `draws`");
}

#[test]
fn named_finishes() {
    let finish = |n: &str| named_finish(n).map(|f| f.get("finish").unwrap().key());
    assert_eq!(
        finish("stone@matte").as_deref(),
        Some("material.finish.matte")
    );
    assert_eq!(
        finish("tunic@fabric#body").as_deref(),
        Some("material.finish.fabric")
    );
    assert_eq!(
        finish("tunic#body@fabric").as_deref(),
        Some("material.finish.fabric")
    );
    assert_eq!(
        finish("tunic#bodyx@fabric").as_deref(),
        Some("material.finish.fabric")
    );
    assert_eq!(
        finish("a@b#bodyx"),
        None,
        "only a slot before @ or at the end goes"
    );
    assert_eq!(finish("x@matte#hair#skin"), None);
    assert_eq!(finish("x@mätte").as_deref(), Some("material.finish.mätte"));
    assert_eq!(finish("@matte"), None);
    assert_eq!(finish("stone@Matte"), None);
    assert_eq!(finish("stone@matte2"), None);
    assert_eq!(finish("stone"), None);
}

/// One triangle whose vertex colour is `colour` at each corner (normalised u16 RGBA, as the models' writer writes it).
fn shaded(colour: [u16; 4]) -> Glb {
    let mut bin: Vec<u8> = [[0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]]
        .iter()
        .flatten()
        .flat_map(|c| c.to_le_bytes())
        .collect();
    for _ in 0..3 {
        bin.extend(colour.iter().flat_map(|c| c.to_le_bytes()));
    }
    let json = r#"{"asset": {"version": "2.0"}, "scenes": [{"nodes": [0]}], "nodes": [{"mesh": 0}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0, "COLOR_0": 1}}]}],
        "bufferViews": [{"byteOffset": 0, "byteLength": 36}, {"byteOffset": 36, "byteLength": 24}],
        "accessors": [{"bufferView": 0, "componentType": 5126, "type": "VEC3", "count": 3, "min": [0, 0, 0], "max": [1, 0, 1]},
            {"bufferView": 1, "componentType": 5123, "normalized": true, "type": "VEC4", "count": 3}]}"#;
    parse_glb(&glb::tests::glb(json, &bin)).unwrap()
}

#[test]
fn baked_shading_is_grey() {
    let prim = |g: &Glb| {
        key(item(&g.gltf, "meshes", &J::Int(0)).unwrap(), "primitives")
            .unwrap()
            .list()
            .unwrap()[0]
            .clone()
    };
    let grey = shaded([40000, 40000, 40000, 65535]);
    assert!(!coloured(&grey, &prim(&grey)).unwrap());
    let tinted = shaded([40000, 20000, 40000, 65535]);
    assert!(coloured(&tinted, &prim(&tinted)).unwrap());
}

/// A sofa bed of two quads facing up: the seat (glTF x -0.5 to 0.1, so game x -0.1 to 0.5) 0.41 m up, and the
/// mattress 0.40 m up under it, which `bed_out` slides 0.6 m forward.
fn sofa_bed() -> Glb {
    let mut bin = Vec::new();
    for (y, x1) in [(0.41f32, 0.1f32), (0.40, 0.5)] {
        for v in [[-0.5, y, -0.5], [-0.5, y, 0.5], [x1, y, 0.5], [x1, y, -0.5]] {
            bin.extend(v.iter().flat_map(|c: &f32| c.to_le_bytes()));
        }
    }
    bin.extend([0u16, 1, 2, 0, 2, 3].iter().flat_map(|i| i.to_le_bytes()));
    bin.extend(
        [0f32, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.6]
            .iter()
            .flat_map(|c| c.to_le_bytes()),
    );
    let json = r#"{"asset": {"version": "2.0"}, "scenes": [{"nodes": [0, 1]}],
        "nodes": [{"name": "seat", "mesh": 0}, {"name": "mattress", "mesh": 1}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0}, "indices": 2}]}, {"primitives": [{"attributes": {"POSITION": 1}, "indices": 2}]}],
        "bufferViews": [{"byteOffset": 0, "byteLength": 48}, {"byteOffset": 48, "byteLength": 48}, {"byteOffset": 96, "byteLength": 12},
            {"byteOffset": 108, "byteLength": 8}, {"byteOffset": 116, "byteLength": 24}],
        "accessors": [{"bufferView": 0, "componentType": 5126, "type": "VEC3", "count": 4, "min": [-0.5, 0.41, -0.5], "max": [0.1, 0.41, 0.5]},
            {"bufferView": 1, "componentType": 5126, "type": "VEC3", "count": 4, "min": [-0.5, 0.4, -0.5], "max": [0.5, 0.4, 0.5]},
            {"bufferView": 2, "componentType": 5123, "type": "SCALAR", "count": 6},
            {"bufferView": 3, "componentType": 5126, "type": "SCALAR", "count": 2}, {"bufferView": 4, "componentType": 5126, "type": "VEC3", "count": 2}],
        "animations": [{"name": "bed_out", "channels": [{"sampler": 0, "target": {"node": 1, "path": "translation"}}], "samplers": [{"input": 3, "output": 4}]}]}"#;
    parse_glb(&glb::tests::glb(json, &bin)).unwrap()
}

#[test]
fn the_sampler_reads_game_axes_and_the_open_pose() {
    let g = sofa_bed();
    let rest = Sampler::new(&g, &J::Obj(Vec::new()), None)
        .unwrap()
        .unwrap();
    // glTF x -> game -x: the seat reaches game x 0.5, not -0.5.
    assert!((rest.top(0.3, 0.0, 0.45).unwrap() - 0.41).abs() < 1e-6);
    assert_eq!(
        rest.top(-0.3, 0.0, 0.41).map(|h| (h * 100.0).round()),
        Some(40.0),
        "only the mattress under it there"
    );
    assert_eq!(rest.top(0.0, 0.8, 0.4), None, "nothing in front at rest");
    let open = Sampler::new(
        &g,
        &J::Obj(Vec::new()),
        Some(&open_pose(&g, &J::Str("bed_out".into())).unwrap()),
    )
    .unwrap()
    .unwrap();
    assert!(
        (open.top(0.0, 0.8, 0.44).unwrap() - 0.40).abs() < 1e-6,
        "pulled out"
    );
    // Scaled: the game's model is the file's times `scale`.
    let big = Sampler::new(&g, &json::loads_text(br#"{"scale": 2}"#).unwrap(), None)
        .unwrap()
        .unwrap();
    assert!((big.top(0.6, 0.0, 0.85).unwrap() - 0.82).abs() < 1e-6);
}

#[test]
fn heights_warn_off_the_model_and_when_their_sources_disagree() {
    let g = sofa_bed();
    let furniture = Furniture {
        seats: json::loads_text(br#"{"sofaBed": {"h": 0.44, "fwd": 0}}"#).unwrap(),
        default_seat: json::loads_text(br#"{"h": 0.45, "fwd": 0}"#).unwrap(),
        beds: json::loads_text(br#"{"sofaBed": {"h": 0.4, "fwd": 0.6}}"#).unwrap(),
        default_bed: json::loads_text(br#"{"h": 0.5}"#).unwrap(),
        bunks: J::Obj(Vec::new()),
    };
    let object = json::loads_text(br#"{"id": "sofaBed", "footprint": [1, 1], "seat": {"height": 0.41, "forward": 0.08}, "interactions": [{"pose": "lie"}]}"#).unwrap();
    let entry = json::loads_text(br#"{"motions": {"open": "bed_out"}}"#).unwrap();
    let found = heights(&g, &entry, &object, &furniture, true).unwrap();
    let warnings: Vec<String> = found.iter().flat_map(Height::warnings).collect();
    assert_eq!(
        warnings,
        [
            "seat 0.44 m (furniture.json `seats`), the model 0.410 m there (x 0.00, depth 0.00): 3.0 cm lower",
            "seat: content `seat` says 0.41 m, 0.08 forward; furniture.json `seats` says 0.44 m, 0.000 forward (furniture.json wins: characters.ts `seatOf`)",
        ]
    );
    let mattress = found.iter().find(|h| h.what == "mattress").unwrap();
    assert_eq!(
        (mattress.open.as_deref(), mattress.at),
        (Some("bed_out"), [0.0, 0.6])
    );
    assert!(mattress.fits());
    // Not held open (no `motions.open`): nothing under the sleepers in front.
    let shut = heights(&g, &J::Obj(Vec::new()), &object, &furniture, true).unwrap();
    let shut: Vec<String> = shut
        .iter()
        .filter(|h| h.what == "mattress")
        .flat_map(Height::warnings)
        .collect();
    assert_eq!(
        shut,
        ["mattress 0.40 m (furniture.json `beds`): no model under it (x 0.00, depth 0.60)"]
    );
}

/// A model of flat quads facing up, a node each: `[height, x from, x to, depth from, depth to]` (glTF axes: game
/// x is the file's -x).
#[test]
fn a_seat_front_is_where_the_seat_ends_in_front() {
    // A deep sofa's seat at 0.44 m from 0.4 m back to 0.38 m forward, its back at 0.8 m behind it, and a
    // cushion's rounded front lip 3 cm lower up to 0.4 m: the seat ends at 0.4 m.
    let sofa = quads(&[
        [0.44, -0.4, 0.4, -0.4, 0.38],
        [0.41, -0.4, 0.4, 0.38, 0.4],
        [0.8, -0.4, 0.4, -0.5, -0.4],
        [0.1, -0.4, 0.4, 0.4, 0.45],
    ]);
    let s = Sampler::new(&sofa, &J::Obj(Vec::new()), None)
        .unwrap()
        .unwrap();
    assert_eq!(
        s.front(0.0, -0.09, 0.44).map(|f| (f * 100.0).round()),
        Some(40.0)
    );
    assert_eq!(
        s.front(0.0, 0.6, 0.44),
        None,
        "nothing under hips in front of it"
    );
    let furniture = |front: f64| Furniture {
        seats: json::loads_text(
            format!(r#"{{"sofaThree": {{"h": 0.44, "fwd": -0.09, "front": {front}}}}}"#).as_bytes(),
        )
        .unwrap(),
        default_seat: json::loads_text(br#"{"h": 0.45, "fwd": 0}"#).unwrap(),
        beds: J::Obj(Vec::new()),
        default_bed: json::loads_text(br#"{"h": 0.5}"#).unwrap(),
        bunks: J::Obj(Vec::new()),
    };
    let object = json::loads_text(
        br#"{"id": "sofaThree", "footprint": [1, 1], "interactions": [{"pose": "sit"}]}"#,
    )
    .unwrap();
    let warn = |front: f64| -> Vec<String> {
        heights(&sofa, &J::Obj(Vec::new()), &object, &furniture(front), true)
            .unwrap()
            .iter()
            .flat_map(Height::warnings)
            .collect()
    };
    assert!(warn(0.41).is_empty(), "{:?}", warn(0.41));
    assert_eq!(
        warn(0.3),
        [
            "seat front 0.30 m forward (furniture.json `seats`), the model's seat ends 0.40 m forward there (x 0.00, depth -0.09): 10.0 cm further forward"
        ]
    );
}

fn quads(quads: &[[f32; 5]]) -> Glb {
    let names: Vec<String> = (0..quads.len()).map(|k| format!("q{k}")).collect();
    named_quads(&names, quads)
}

/// `quads` with their nodes named `names`.
fn named_quads(names: &[String], quads: &[[f32; 5]]) -> Glb {
    let mut bin = Vec::new();
    for [y, x0, x1, z0, z1] in quads {
        for v in [
            [*x0, *y, *z0],
            [*x0, *y, *z1],
            [*x1, *y, *z1],
            [*x1, *y, *z0],
        ] {
            bin.extend(v.iter().flat_map(|c: &f32| c.to_le_bytes()));
        }
    }
    let at = bin.len();
    bin.extend([0u16, 1, 2, 0, 2, 3].iter().flat_map(|i| i.to_le_bytes()));
    let n = quads.len();
    let nodes: Vec<String> = (0..n)
        .map(|k| format!(r#"{{"name": "{}", "mesh": {k}}}"#, names[k]))
        .collect();
    let meshes: Vec<String> = (0..n)
        .map(|k| {
            format!(r#"{{"primitives": [{{"attributes": {{"POSITION": {k}}}, "indices": {n}}}]}}"#)
        })
        .collect();
    let mut views: Vec<String> = (0..n)
        .map(|k| format!(r#"{{"byteOffset": {}, "byteLength": 48}}"#, k * 48))
        .collect();
    views.push(format!(r#"{{"byteOffset": {at}, "byteLength": 12}}"#));
    let mut accessors: Vec<String> = quads
        .iter()
        .enumerate()
        .map(|(k, [y, x0, x1, z0, z1])| format!(r#"{{"bufferView": {k}, "componentType": 5126, "type": "VEC3", "count": 4, "min": [{x0}, {y}, {z0}], "max": [{x1}, {y}, {z1}]}}"#))
        .collect();
    accessors.push(format!(
        r#"{{"bufferView": {n}, "componentType": 5123, "type": "SCALAR", "count": 6}}"#
    ));
    let json = format!(
        r#"{{"asset": {{"version": "2.0"}}, "scenes": [{{"nodes": [{}]}}], "nodes": [{}], "meshes": [{}], "bufferViews": [{}], "accessors": [{}]}}"#,
        (0..n).map(|k| k.to_string()).collect::<Vec<_>>().join(", "),
        nodes.join(", "),
        meshes.join(", "),
        views.join(", "),
        accessors.join(", ")
    );
    parse_glb(&glb::tests::glb(&json, &bin)).unwrap()
}

#[test]
fn a_top_the_content_hasnt_a_table_without_its_chairs_and_a_stairs_steps() {
    // A table: its top 0.74 m up (1.2 × 0.8), a shelf under it, a vase on it; a chair behind it, its seat 0.46
    // m up and its back's top rail 1.0 m.
    let table = quads(&[
        [0.74, -0.6, 0.6, -0.4, 0.4],
        [0.3, -0.6, 0.6, -0.4, 0.4],
        [0.95, -0.05, 0.05, -0.05, 0.05],
        [0.46, -0.25, 0.25, -0.7, -0.3],
        [1.0, -0.25, 0.25, -0.72, -0.68],
    ]);
    let s = Sampler::new(&table, &J::Obj(Vec::new()), None)
        .unwrap()
        .unwrap();
    // The vase on the middle line and the shelf under every one don't move it.
    let (at, top) = top_near(&s, [0.72, 0.76], &[]).unwrap();
    assert!((top - 0.74).abs() < 1e-6, "{at:?} {top}");
    // Out of the range, it is still the top (the review calls it a top if near enough): the vase, in the
    // range, stands on what the other lines found.
    assert!((top_near(&s, [0.85, 0.95], &[]).unwrap().1 - 0.74).abs() < 1e-6);
    // A sink: its worktop 0.9 m up at the back, the basin's floor 0.75 m in front (on six lines of nine).
    let sink = quads(&[[0.9, -0.5, 0.5, -0.5, -0.2], [0.75, -0.5, 0.5, -0.2, 0.5]]);
    let (at, top) = top_near(
        &Sampler::new(&sink, &J::Obj(Vec::new()), None)
            .unwrap()
            .unwrap(),
        [0.76, 0.91],
        &[],
    )
    .unwrap();
    assert!((top - 0.9).abs() < 1e-6 && at[1] < -0.2, "{at:?} {top}");
    // The lines: its middle (0, -0.16) and a quarter of its 1.2 × 1.12 m round it; those within 0.3 m of a
    // seat's hips are left out (over the chair's seat, and the middle), unless the others find nothing.
    let cm = |lines: Vec<[f64; 2]>| -> Vec<[i64; 2]> {
        lines
            .iter()
            .map(|p| p.map(|v| (v * 100.0).round() as i64))
            .collect()
    };
    let all = top_lines(&s, &[]);
    assert_eq!(
        cm(all),
        [
            [-30, -44],
            [-30, -16],
            [-30, 12],
            [0, -44],
            [0, -16],
            [0, 12],
            [30, -44],
            [30, -16],
            [30, 12]
        ]
    );
    assert_eq!(
        cm(top_lines(&s, &[[0.0, -0.4]])),
        [
            [-30, -44],
            [-30, -16],
            [-30, 12],
            [0, 12],
            [30, -44],
            [30, -16],
            [30, 12]
        ]
    );
    let seated = [[0.3, -0.1], [-0.05, -0.44], [-0.3, -0.16], [0.0, 0.0]];
    assert_eq!(
        cm(top_lines(&s, &seated)),
        [[30, -44]],
        "the one line left, off the table"
    );
    assert!(
        (top_near(&s, [0.72, 0.76], &seated).unwrap().1 - 0.74).abs() < 1e-6,
        "so all the lines"
    );
    // Without the chair's back: the vase, over the top, counts.
    assert!((s.height_by(0.74).unwrap() - 0.95).abs() < 1e-6);
    assert_eq!(s.height_by(0.5), None, "nothing flat there");

    // A stair of three treads, a rail at its side.
    let stair = quads(&[
        [0.2, -0.4, 0.4, 0.2, 0.6],
        [0.4, -0.4, 0.4, -0.2, 0.2],
        [0.6, -0.4, 0.4, -0.6, -0.2],
        [1.5, 0.45, 0.5, -0.6, 0.6],
    ]);
    let s = Sampler::new(&stair, &J::Obj(Vec::new()), None)
        .unwrap()
        .unwrap();
    let (top, n) = s.steps().unwrap();
    assert_eq!(((top * 100.0).round(), n), (60.0, 3));
}

#[test]
fn the_review_takes_a_cribs_mattress_at_slot_0() {
    // A crib's mattress 0.5 m up at slot 0 (game x -1 to 0: the file's 0 to 1), its changing pad 0.8 m at slot 1.
    let g = quads(&[[0.5, 0.0, 1.0, -0.5, 0.5], [0.8, -1.0, 0.0, -0.5, 0.5]]);
    let furniture = Furniture {
        seats: J::Obj(Vec::new()),
        default_seat: json::loads_text(br#"{"h": 0.45, "fwd": 0}"#).unwrap(),
        beds: json::loads_text(br#"{"crib": {"h": 0.5}}"#).unwrap(),
        default_bed: json::loads_text(br#"{"h": 0.5}"#).unwrap(),
        bunks: J::Obj(Vec::new()),
    };
    let object = json::loads_text(
        br#"{"id": "crib", "footprint": [2, 1], "slots": 2, "interactions": [{"pose": "lie"}]}"#,
    )
    .unwrap();
    let mattress = |every: bool| {
        heights(&g, &J::Obj(Vec::new()), &object, &furniture, every)
            .unwrap()
            .into_iter()
            .find(|h| h.what == "mattress")
            .map(|h| (h.at, h.mesh))
            .unwrap()
    };
    assert_eq!(mattress(false), ([-0.5, 0.0], Some(0.5)));
    let (at, mesh) = mattress(true);
    assert_eq!(at, [0.5, 0.0], "the gate takes the worst slot");
    assert!((mesh.unwrap() - 0.8).abs() < 1e-6);
}

/// A sofa lain on across is lain on along it (residents.ts `bedFwd`): its mattress is `fwd` along it at the slots'
/// depth, on the seat, not `fwd` back on the back's foot; a place stood at (a masseur's) is no mattress.
#[test]
fn a_mattress_is_where_the_renderer_lays_the_sleeper() {
    // A three-seat sofa: its seat 0.44 m up from 0.3 m back to 0.4 m forward, the back's foot 0.32 m behind it.
    let sofa = quads(&[[0.44, -1.4, 1.4, -0.3, 0.4], [0.32, -1.4, 1.4, -0.45, -0.3]]);
    let furniture = Furniture {
        seats: J::Obj(Vec::new()),
        default_seat: json::loads_text(br#"{"h": 0.45, "fwd": 0}"#).unwrap(),
        beds: json::loads_text(
            br#"{"sofa3": {"h": 0.44, "fwd": -0.42, "across": true}, "massage": {"h": 0.79}}"#,
        )
        .unwrap(),
        default_bed: json::loads_text(br#"{"h": 0.5}"#).unwrap(),
        bunks: J::Obj(Vec::new()),
    };
    let mattress = |g: &Glb, object: &[u8], furniture: &Furniture| {
        heights(
            g,
            &J::Obj(Vec::new()),
            &json::loads_text(object).unwrap(),
            furniture,
            true,
        )
        .unwrap()
        .into_iter()
        .find(|h| h.what == "mattress")
        .map(|h| (h.at, h.mesh.map(|m| (m * 100.0).round())))
        .unwrap()
    };
    let sofa3 =
        br#"{"id": "sofa3", "footprint": [3, 1], "slots": 3, "interactions": [{"pose": "lie"}]}"#;
    assert_eq!(
        mattress(&sofa, sofa3, &furniture),
        ([-0.42, 0.0], Some(44.0))
    );
    // A massage table's top 0.79 m up, game x -1 to 0; its masseur stands at x 0.5, off it.
    let table = quads(&[[0.79, 0.0, 1.0, -0.5, 0.5]]);
    let massage =
        br#"{"id": "massage", "footprint": [2, 1], "slots": 2, "interactions": [{"pose": "lie"}],
        "places": [{"at": [-0.5, 0]}, {"at": [0.5, 0], "facing": -90, "pose": "stand"}]}"#;
    assert_eq!(
        mattress(&table, massage, &furniture),
        ([-0.5, 0.0], Some(79.0))
    );
}

#[test]
fn motions_played_in_order_and_checked() {
    let m = json::loads_text(br#"{"uses": {"fetch": ["door_open", {"clip": "door_open", "hold": -1, "slot": true, "x": 1}], "b": "lid"},
        "loops": {"relax": "rock"}, "idle": "rock", "clock": {"hour": "h", "at": "noon"}}"#).unwrap();
    let p: Vec<(String, String)> = played(&m)
        .unwrap()
        .into_iter()
        .map(|(c, by)| (c.key(), by))
        .collect();
    assert_eq!(
        p,
        [
            ("door_open", "use 'fetch'"),
            ("lid", "use 'b'"),
            ("rock", "loop 'relax'"),
            ("h", "clock hour")
        ]
        .map(|(a, b)| (a.to_string(), b.to_string()))
    );
    assert_eq!(
        motions_errors(&m),
        [
            "`motions.uses` 'fetch': hold -1 is no number of seconds",
            "`motions.uses` 'fetch': slot True is no slot (0, 1, ...)",
            "`motions.uses` 'fetch': unknown x",
            "`motions.clock.at` 'noon' is no time (minutes past twelve)",
        ]
    );
    assert_eq!(
        loops(&m).unwrap(),
        HashSet::from(["rock".to_string(), "h".to_string()])
    );
}

fn j(text: &str) -> J {
    json::loads_text(text.as_bytes()).unwrap()
}

#[test]
fn a_birds_fields_are_its_sizes() {
    assert!(
        bird_fields(&j(
            r#"{"length": 0.12, "wingspan": 0.185, "perchHeight": 0.03, "pitch": 34}"#
        ))
        .is_empty()
    );
    assert!(
        bird_fields(&j(r#"{"length": 0.12, "wingspan": 0.185}"#)).is_empty(),
        "the rest is optional"
    );
    assert_eq!(
        bird_fields(&j(
            r#"{"length": 0, "span": 1, "pitch": 95, "perchHeight": true}"#
        )),
        [
            "`bird`: unknown span (it takes length, wingspan, perchHeight, pitch)",
            "`bird.length` 0 is no length (metres, over 0)",
            "`bird` has no `wingspan`",
            "`bird.perchHeight` True is no length (metres, over 0)",
            "`bird.pitch` 95 is no angle (degrees, -90 to 90)",
        ]
    );
    assert_eq!(
        bird_fields(&j("[0.12]")),
        ["`bird` is {\"length\", \"wingspan\", ...} (metres), not [0.12]"]
    );
}

/// The blue tit as built (crates/art/things/src/birds) against its `bird`, and against wrong ones.
#[test]
fn a_bird_is_checked_against_its_file() {
    let path = join(&repo_root(), "web/public/assets/models/birds/blueTit.glb");
    let g = parse_glb(&std::fs::read(&path).unwrap()).unwrap();
    let parts = bird_parts(&g).unwrap();
    assert_eq!(
        parts.names.iter().map(String::as_str).collect::<Vec<_>>(),
        ["body", "head", "tail", "wing.L", "wing.R"]
    );
    assert!(parts.flat.is_empty());
    let dims = [0.185, 0.072, 0.106];
    assert!(
        bird_errors(
            &j(r#"{"length": 0.12, "wingspan": 0.185, "perchHeight": 0.03, "pitch": 34}"#),
            &parts,
            dims
        )
        .is_empty()
    );
    let wrong = bird_errors(
        &j(r#"{"length": 0.2, "wingspan": 0.3, "perchHeight": 0.5}"#),
        &parts,
        dims,
    );
    assert_eq!(wrong.len(), 3, "{wrong:?}");
    assert!(
        wrong[0].starts_with("0.185 m across its spread wings, its `bird.wingspan` says 0.300 m"),
        "{}",
        wrong[0]
    );
    assert!(
        wrong[1].starts_with("0.12")
            && wrong[1].contains("from bill to tail tip, its `bird.length` says 0.200 m"),
        "{}",
        wrong[1]
    );
    assert_eq!(
        wrong[2],
        "`bird.perchHeight` 0.500 m is over the bird's height, 0.072 m"
    );
    // Its wings swapped over, one part gone and one saved flat.
    let mut points = parts.points;
    let left = points.remove("wing.L").unwrap();
    let right = points.insert("wing.R".into(), left).unwrap();
    points.insert("wing.L".into(), right);
    let mut names = parts.names;
    names.remove("tail");
    let broken = BirdParts {
        names,
        points,
        flat: vec!["wing.L".into()],
    };
    let e = bird_errors(&j(r#"{"length": 0.12, "wingspan": 0.185}"#), &broken, dims);
    assert_eq!(
        e[0],
        "a bird's parts are nodes named body, head, tail, wing.L, wing.R: it has no tail"
    );
    assert!(e[1].starts_with("wing.L is saved at scale 0"), "{}", e[1]);
    assert_eq!(
        &e[2..],
        [
            "wing.L reaches out the wrong way: a bird's wing.L is spread to its left, game -x (glTF +x)",
            "wing.R reaches out the wrong way: a bird's wing.R is spread to its right, game +x (glTF -x)",
        ]
    );
}

/// A strict check's finding is an error unless the entry `accept`s it by name with a reason; an accept that names
/// no check, gives no reason or isn't needed any more is an error itself; the rest stay warnings.
#[test]
fn strict_findings_fail_unless_accepted_with_a_reason() {
    let over = strict("footprint", "reaches 15 cm past its 1 x 1 footprint".into());
    let aim = "4 draw calls, aim for 3 at most".to_string();
    let run = |accept: &str, warnings: Vec<String>| {
        strictly(
            &j(&format!(r#"{{"url": "x.glb"{accept}}}"#)),
            vec!["a hard error".into()],
            warnings,
            &[],
        )
    };
    // Not accepted: an error, the aim a warning.
    let (e, w, a) = run("", vec![over.clone(), aim.clone()]);
    assert_eq!(
        e,
        ["a hard error", "reaches 15 cm past its 1 x 1 footprint"]
    );
    assert_eq!((w, a.len()), (vec![aim.clone()], 0));
    // Accepted, with its reason.
    let (e, w, a) = run(
        r#", "accept": {"footprint": "the rail runs past the window"}"#,
        vec![over.clone(), aim],
    );
    assert_eq!(e, ["a hard error"]);
    assert_eq!(w.len(), 1);
    assert_eq!(
        a,
        ["reaches 15 cm past its 1 x 1 footprint (footprint: the rail runs past the window)"]
    );
    // A far version's finding is the model's: still strict.
    let far = match untag(&over) {
        Some((kind, text)) => strict(kind, format!("far: {text}")),
        None => unreachable!(),
    };
    let (e, ..) = run("", vec![far]);
    assert_eq!(e[1], "far: reaches 15 cm past its 1 x 1 footprint");
    // Unused, unknown, without a reason.
    let (e, ..) = run(r#", "accept": {"radius": "wide"}"#, Vec::new());
    assert!(e[1].starts_with("`accept.radius` is unused"), "{e:?}");
    let (e, ..) = run(r#", "accept": {"looks": "fine"}"#, Vec::new());
    assert!(
        e[1].starts_with("`accept` names 'looks', no check of the gate's"),
        "{e:?}"
    );
    let (e, ..) = run(r#", "accept": {"footprint": " "}"#, vec![over]);
    assert_eq!(
        e[1..],
        [
            "reaches 15 cm past its 1 x 1 footprint",
            "`accept.footprint` needs its reason: why the model is right as it is"
        ]
    );
    let (e, ..) = run(r#", "accept": ["footprint"]"#, Vec::new());
    assert_eq!(e[1], "`accept` is {check: reason}");
    // Every strict check has its name once.
    let names: HashSet<&str> = STRICT.iter().map(|(k, _)| *k).collect();
    assert_eq!(names.len(), STRICT.len());
}

/// A texture is judged on a part at its largest: washing a clip grows on the line from flattened (scale y 0.001) is
/// measured grown, 1 m tall; a cloth a clip stretches to three times as built is measured stretched, and found so;
/// a cushion a clip squashes, at rest; the edges at rest.
#[test]
fn texels_are_measured_on_a_part_at_its_largest() {
    let tall = |rest: f32, grown: f32, read| {
        let mut bin: Vec<u8> = [[0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
            .iter()
            .flatten()
            .chain(&[0f32, 0.0, 1.0, 0.0, 0.0, 1.0])
            .flat_map(|c| c.to_le_bytes())
            .collect();
        for c in [0f32, 1.0, 1.0, rest, 1.0, 1.0, grown, 1.0] {
            bin.extend(c.to_le_bytes());
        }
        let json = format!(
            r#"{{"asset": {{"version": "2.0"}}, "scenes": [{{"nodes": [0]}}],
            "nodes": [{{"name": "washing", "mesh": 0, "scale": [1, {rest}, 1]}}],
            "materials": [{{"name": "cloth"}}],
            "meshes": [{{"primitives": [{{"attributes": {{"POSITION": 0, "TEXCOORD_0": 1}}, "material": 0}}]}}],
            "bufferViews": [{{"byteOffset": 0, "byteLength": 36}}, {{"byteOffset": 36, "byteLength": 24}}, {{"byteOffset": 60, "byteLength": 8}}, {{"byteOffset": 68, "byteLength": 24}}],
            "accessors": [{{"bufferView": 0, "componentType": 5126, "type": "VEC3", "count": 3, "min": [0, 0, 0], "max": [1, 1, 0]}},
                {{"bufferView": 1, "componentType": 5126, "type": "VEC2", "count": 3}},
                {{"bufferView": 2, "componentType": 5126, "type": "SCALAR", "count": 2}}, {{"bufferView": 3, "componentType": 5126, "type": "VEC3", "count": 2}}],
            "animations": [{{"name": "hang", "channels": [{{"sampler": 0, "target": {{"node": 0, "path": "scale"}}}}], "samplers": [{{"input": 2, "output": 3}}]}}]}}"#
        );
        let g = parse_glb(&glb::tests::glb(&json, &bin)).unwrap();
        let fin = Finishes {
            plain: HashMap::from([("material.finish.fabric".to_string(), false)]),
            see_through: HashSet::new(),
            textures: HashMap::from([("material.finish.fabric".to_string(), (1.0, String::new()))]),
        };
        let entry = j(r#"{"materials": {"cloth": {"finish": "material.finish.fabric"}}}"#);
        let file = ModelFile {
            key: "model.clothesline",
            entry: &entry,
            glb: &g,
            fin: &fin,
            far: false,
        };
        let m = meshes::read(&file, read).unwrap().unwrap();
        let textures = crate::texels::Textures::from([(
            "fabric".to_string(),
            crate::texels::Texture {
                uv_scale: 1.0,
                px: None,
            },
        )]);
        let stretched = crate::texels::measure(&m, &textures)
            .first()
            .is_some_and(|t| !crate::texels::findings(t).is_empty());
        let top = m[0].positions.iter().map(|p| p[1]).fold(0.0, f64::max);
        (top, stretched)
    };
    let near = |a: (f64, bool), b: f64| (a.0 - b).abs() < 1e-6;
    assert!(near(tall(0.001, 1.0, meshes::Read::Textured), 1.0), "grown");
    assert!(
        near(tall(0.001, 1.0, meshes::Read::Shape), 0.001),
        "at rest"
    );
    let cloth = tall(1.0, 3.0, meshes::Read::Textured);
    assert!(near(cloth, 3.0) && cloth.1, "stretched: {cloth:?}");
    assert!(!tall(1.0, 1.0, meshes::Read::Textured).1, "as built");
    assert!(
        near(tall(1.0, 0.5, meshes::Read::Textured), 1.0),
        "squashed"
    );
}

/// A held tool (a coin 2 mm thick) is never on a floor, so the floor drawn over thin things isn't held against
/// it; the same file as a floor model is.
#[test]
fn a_held_tool_is_not_thin_on_the_floor() {
    let fin = Finishes {
        plain: HashMap::new(),
        see_through: HashSet::new(),
        textures: HashMap::new(),
    };
    let path = join(&repo_root(), "web/public/assets/models/held/coin.glb");
    let entry = j(r#"{"type": "model", "url": "coin.glb"}"#);
    let thin = |key: &str| {
        let c = check(
            key,
            &fin,
            &entry,
            &path,
            None,
            None,
            None,
            None,
            &no_counting(),
        )
        .unwrap();
        c.errors
            .iter()
            .chain(&c.warnings)
            .any(|w| w.contains("reaches only"))
    };
    assert!(!thin("tool.coin"));
    assert!(thin("model.coin"));
    assert!(in_hand("tool.phone") && !in_hand("item.parcel") && !in_hand("model.yogaMat"));
}

#[test]
fn perches_are_points_inside_the_model() {
    let (lo, hi) = ([-1.0, 0.0, -1.0], [1.0, 4.0, 1.0]);
    assert!(
        perch_errors(&j("[[0.5, 3.9, -0.5], [-1.005, 0, 1]]"), lo, hi, 1.0).is_empty(),
        "a centimetre out is in"
    );
    assert_eq!(
        perch_errors(
            &j("[[0, 5, 0], [0.5, 1], [1, 1, \"x\"], [0, 2, 0], [3, 1, 0]]"),
            lo,
            hi,
            1.0
        ),
        [
            "`perches` 1: [0.5, 1] is no [x, y, z]",
            "`perches` 2: [1, 1, 'x'] is no [x, y, z]",
            "2 of 5 perches outside the model (the first, 0: [0, 5, 0]; its box x, y, z: -1.00 to 1.00, 0.00 to 4.00, -1.00 to 1.00)",
        ]
    );
    // Before the entry's `scale`: a tree drawn twice its size has its box twice as big, its perches as written.
    assert!(perch_errors(&j("[[0, 3, 0]]"), [-2.0, 0.0, -2.0], [2.0, 8.0, 2.0], 2.0).is_empty());
    assert_eq!(
        perch_errors(&j("[[0, 5, 0]]"), [-2.0, 0.0, -2.0], [2.0, 8.0, 2.0], 2.0).len(),
        1
    );
    assert_eq!(
        perch_errors(&j("[]"), lo, hi, 1.0),
        ["`perches` is empty: leave it out"]
    );
    assert_eq!(
        perch_errors(&j("{\"a\": 1}"), lo, hi, 1.0),
        ["`perches` is a list of [x, y, z] (metres), not {'a': 1}"]
    );
}

#[test]
fn a_habitat_is_one_of_four_and_goes_with_perches() {
    for h in HABITATS {
        assert!(
            habitat_errors(&j(&format!(
                r#"{{"perches": [[0, 1, 0]], "habitat": "{h}"}}"#
            )))
            .is_empty(),
            "{h}"
        );
    }
    assert!(
        habitat_errors(&j(r#"{"perches": [[0, 1, 0]]}"#)).is_empty(),
        "optional"
    );
    assert!(habitat_errors(&j(r#"{"url": "a.glb"}"#)).is_empty());
    assert_eq!(
        habitat_errors(&j(r#"{"perches": [[0, 1, 0]], "habitat": "shrub"}"#)),
        ["`habitat` 'shrub' is none of tree, conifer, bush, hedge"]
    );
    assert_eq!(
        habitat_errors(&j(r#"{"perches": [[0, 1, 0]], "habitat": ["tree"]}"#)),
        ["`habitat` ['tree'] is none of tree, conifer, bush, hedge"]
    );
    assert_eq!(
        habitat_errors(&j(r#"{"habitat": "Tree"}"#)),
        [
            "`habitat` 'Tree' is none of tree, conifer, bush, hedge",
            "`habitat` without `perches`: it says what the perches are on",
        ]
    );
}

/// No counted checks, an empty baseline.
fn no_counting() -> Counting<'static> {
    Counting {
        baseline: Box::leak(Box::default()),
        file: &[],
        batch: BTreeMap::new(),
    }
}

/// An example counted check, no rule of the gate's: a finding for each mesh of the file.
fn meshes(file: &ModelFile) -> R<Found> {
    let n = file
        .glb
        .gltf
        .get("meshes")
        .and_then(J::list)
        .map_or(0, |m| m.len());
    Ok(Found {
        count: n as u64,
        text: (0..n).map(|i| format!("mesh {i}")).collect(),
    })
}

/// An example counted check that finds nothing.
fn nothing(_: &ModelFile) -> R<Found> {
    Ok(Found::default())
}

#[test]
fn a_texture_of_its_own_is_512_px_or_1024_over_2_5_m() {
    assert_eq!(
        [0.3, 2.5, 2.51, 10.0].map(texture_most),
        [512, 512, 1024, 1024]
    );
}

/// A counted check's findings by the baseline: new or over, strict; at it, warnings; under it, stale; an entry
/// that accepts the check takes them whatever the baseline holds.
#[test]
fn a_counted_check_is_judged_by_the_baseline() {
    let fin = Finishes {
        plain: HashMap::new(),
        see_through: HashSet::new(),
        textures: HashMap::new(),
    };
    let path = join(&repo_root(), "web/public/assets/models/held/coin.glb");
    let run = |base: &str,
               accept: &str,
               file: &[(&'static str, FileCheck)],
               batch: u64,
               far: Option<&str>| {
        let baseline = Baseline::parse(base).unwrap();
        let mut counting = Counting {
            baseline: &baseline,
            file,
            batch: BTreeMap::new(),
        };
        if batch > 0 {
            let found = Found {
                count: batch,
                text: vec!["from the batch".into()],
            };
            counting
                .batch
                .insert("edges", BTreeMap::from([("tool.coin".to_string(), found)]));
        }
        let entry = j(&format!(
            r#"{{"type": "model", "url": "coin.glb"{accept}}}"#
        ));
        check(
            "tool.coin",
            &fin,
            &entry,
            &path,
            far,
            None,
            None,
            None,
            &counting,
        )
        .unwrap()
    };
    let mine = |lines: &[String]| -> Vec<String> {
        lines
            .iter()
            .filter(|l| l.contains("mesh ") || l.contains("edges") || l.contains("batch"))
            .cloned()
            .collect()
    };
    let edges: &[(&'static str, FileCheck)] = &[("edges", meshes)];
    let c = run("{}", "", edges, 0, None);
    let n = c.counts[0].1;
    assert_eq!(c.counts, [("edges", n)]);
    assert!(n >= 1);
    let each: Vec<String> = (0..n).map(|i| format!("mesh {i}")).collect();
    // New: an error each.
    assert_eq!(mine(&c.errors), each);
    assert!(mine(&c.warnings).is_empty());
    // Held: warnings, led by the count.
    let held = format!(r#"{{"edges": {{"tool.coin": {n}}}}}"#);
    let c = run(&held, "", edges, 0, None);
    assert!(mine(&c.errors).is_empty(), "{:?}", c.errors);
    assert_eq!(
        mine(&c.warnings)[0],
        format!("edges: {n} found, as the baseline has (tools/art/gate-baseline.json)")
    );
    assert_eq!(mine(&c.warnings)[1..], each);
    // Over: one more from a batch check, all strict.
    let c = run(&held, "", edges, 1, None);
    assert_eq!(
        mine(&c.errors)[0],
        format!(
            "edges: {} found, {n} in the baseline (tools/art/gate-baseline.json): fix the new ones",
            n + 1
        )
    );
    assert_eq!(mine(&c.errors).last().unwrap(), "from the batch");
    assert_eq!(c.counts, [("edges", n + 1)]);
    // Stale: warnings and an error to lower it.
    let c = run(
        &format!(r#"{{"edges": {{"tool.coin": {}}}}}"#, n + 2),
        "",
        edges,
        0,
        None,
    );
    assert_eq!(
        mine(&c.errors),
        [format!(
            "edges: {n} found, {} in the baseline: lower it (`cargo check-glb --update-baseline`)",
            n + 2
        )]
    );
    assert_eq!(mine(&c.warnings), each);
    // The far version's findings are the model's.
    let c = run("{}", "", edges, 0, Some(&path));
    assert_eq!(c.counts, [("edges", 2 * n)]);
    assert!(mine(&c.errors).contains(&"far: mesh 0".to_string()));
    // Accepted: whatever the baseline holds, and it may keep none.
    let why = r#", "accept": {"edges": "an example"}"#;
    let c = run("{}", why, edges, 0, None);
    assert!(mine(&c.errors).is_empty(), "{:?}", c.errors);
    assert_eq!(c.accepted[0], "mesh 0 (edges: an example)");
    assert_eq!(c.counts, [("edges", 0)]);
    let c = run(&held, why, edges, 0, None);
    assert_eq!(
        mine(&c.errors),
        [format!(
            "edges: its entry accepts it, and the baseline still has {n}: take it out (`cargo check-glb --update-baseline`)"
        )]
    );
    // An accept of a counted check is unused where it ran and found nothing, not where it didn't run.
    let c = run("{}", why, &[("edges", nothing)], 0, None);
    assert!(
        c.errors
            .iter()
            .any(|e| e.starts_with("`accept.edges` is unused")),
        "{:?}",
        c.errors
    );
    let c = run("{}", why, &[], 0, None);
    assert!(mine(&c.errors).is_empty(), "{:?}", c.errors);
    assert!(c.counts.is_empty());
}

/// The gate over a model with the example check: the baseline's verdicts in its report, and `--update-baseline`
/// lowering and taking out but never raising or adding.
#[test]
fn the_gate_judges_by_the_baseline_and_update_only_lowers() {
    let dir = std::env::temp_dir().join(format!("ll-gate-baseline-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut paths = Paths::repo();
    paths.baseline = dir
        .join("gate-baseline.json")
        .to_string_lossy()
        .into_owned();
    let set = |text: &str| std::fs::write(&paths.baseline, text).unwrap();
    let read = || std::fs::read_to_string(&paths.baseline).unwrap();
    let keys = vec!["model.fridge".to_string()];
    let edges: &[(&'static str, FileCheck)] = &[("edges", meshes)];
    let run = || run_gate(&paths, &keys, edges, &[]);
    // No file: strict, and nothing to add.
    let r = run();
    assert_eq!(r.failed, 1, "{}", r.text);
    assert!(r.text.starts_with("FAIL model.fridge "), "{}", r.text);
    assert!(r.text.contains("\n       error: mesh 0\n"), "{}", r.text);
    let n = r.counts["edges"]["model.fridge"];
    assert_eq!(
        update_with(&paths, &keys, edges, &[]).unwrap(),
        Vec::<String>::new()
    );
    assert!(!std::path::Path::new(&paths.baseline).exists());
    // At its count: it passes, its findings warnings.
    set(&format!("{{\"edges\": {{\"model.fridge\": {n}}}}}\n"));
    let r = run();
    assert_eq!(r.failed, 0, "{}", r.text);
    assert!(r.text.contains(&format!(
        "       warning: edges: {n} found, as the baseline has"
    )));
    // Higher: stale until lowered; lowered once, then as it was.
    set(&format!("{{\"edges\": {{\"model.fridge\": {}}}}}\n", n + 3));
    assert_eq!(run().failed, 1);
    assert_eq!(
        update_with(&paths, &keys, edges, &[]).unwrap(),
        [format!("edges: model.fridge {} -> {n}", n + 3)]
    );
    assert_eq!(
        read(),
        format!("{{\n  \"edges\": {{\n    \"model.fridge\": {n}\n  }}\n}}\n")
    );
    assert_eq!(run().failed, 0);
    // Lower: over; never raised.
    set(&format!(
        "{{\"edges\": {{\"model.fridge\": {}}}}}\n",
        n.max(2) - 1
    ));
    if n >= 2 {
        assert_eq!(run().failed, 1);
        assert!(update_with(&paths, &keys, edges, &[]).unwrap().is_empty());
    }
    // A section for no counted check.
    set("{\"looks\": {\"model.fridge\": 1}}\n");
    let r = run();
    assert!(
        r.text
            .contains("FAIL gate-baseline.json\n       error: 'looks' is no counted check"),
        "{}",
        r.text
    );
    // Over every model, a key that is no model is stale, and taken out.
    set("{\"edges\": {\"model.gone\": 2}}\n");
    let none: &[(&'static str, FileCheck)] = &[("edges", nothing)];
    // (A stub check finds nothing, so a model that accepts `edges` fails for its accept being unused; and this
    // baseline has no `opened` or `unmoved` section, strict for every model, so the models they find fail too.)
    let expected = |t: &str| {
        t.lines().filter(|l| l.contains("error: ")).all(|l| {
            l.contains("`accept.edges` is unused")
                || l.contains("error: opened (")
                || l.contains("error: part '")
                || l.contains("no model the gate checked")
        })
    };
    let r = run_gate(&paths, &[], none, &[]);
    assert!(expected(&r.text), "{}", r.text);
    assert!(
        r.text.contains(
            "FAIL gate-baseline.json\n       error: edges: model.gone is no model the gate checked"
        ),
        "{}",
        r.text
    );
    assert_eq!(
        update_with(&paths, &[], none, &[]).unwrap(),
        [
            "edges: model.gone out (was 2)",
            "edges: none left, strict for every model"
        ]
    );
    assert_eq!(read(), "{}\n");
    // The built world's sections are its own gate's to judge (`built::CHECKS`): left as they are.
    set("{\"built.edges\": {\"house.cottage\": 3}}\n");
    let r = run_gate(&paths, &[], none, &[]);
    assert!(
        expected(&r.text) && !r.text.contains("FAIL gate-baseline.json"),
        "{}",
        r.text
    );
    assert!(update_with(&paths, &[], none, &[]).unwrap().is_empty());
    assert_eq!(read(), "{\"built.edges\": {\"house.cottage\": 3}}\n");
    // A baseline that can't be read fails the gate.
    set("{\"edges\": {\"model.fridge\": 0}}");
    assert!(
        run()
            .text
            .starts_with("FAIL gate-baseline.json\n       error: cannot read it: ")
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn an_argument_it_does_not_know_is_a_usage_error() {
    for args in [["--valdate"], ["chair"]] {
        let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
        assert_eq!(main(&args), 2, "{args:?}");
    }
}

/// A door that opens back into the wall: at rest its back is clear of the wall's face, but opened it isn't (a display
/// cabinet's door swung -100° about its hinge). The same door swung forward is fine.
#[test]
fn a_door_opened_into_the_wall_fails() {
    let opened = |degrees: f32| {
        // a panel 0.9 wide hinged at the front's left (glTF x -0.25, depth 0.25), turned about y
        let json = r#"{"asset": {"version": "2.0"}, "scenes": [{"nodes": [0]}],
            "nodes": [{"name": "door", "mesh": 0, "translation": [-0.25, 0, 0.25]}],
            "meshes": [{"primitives": [{"attributes": {"POSITION": 0}}]}],
            "animations": [{"name": "door_open", "channels": [{"sampler": 0, "target": {"node": 0, "path": "rotation"}}],
                "samplers": [{"input": 1, "output": 2}]}],
            "bufferViews": [{"byteOffset": 0, "byteLength": 36}, {"byteOffset": 36, "byteLength": 8},
                {"byteOffset": 44, "byteLength": 32}],
            "accessors": [{"bufferView": 0, "componentType": 5126, "type": "VEC3", "count": 3, "min": [0, 0, 0], "max": [0.9, 0.8, 0]},
                {"bufferView": 1, "componentType": 5126, "type": "SCALAR", "count": 2},
                {"bufferView": 2, "componentType": 5126, "type": "VEC4", "count": 2}]}"#;
        let half = (degrees / 2.0).to_radians();
        let bin: Vec<u8> = [
            0f32,
            0.0,
            0.0,
            0.9,
            0.0,
            0.0,
            0.0,
            0.8,
            0.0, // the panel
            0.0,
            1.0, // the clip's times
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            half.sin(),
            0.0,
            half.cos(),
        ]
        .iter()
        .flat_map(|c| c.to_le_bytes())
        .collect();
        let g = parse_glb(&glb::tests::glb(json, &bin)).unwrap();
        let entry = j(r#"{"motions": {"uses": {"openDoor": "door_open"}}}"#);
        let facts = Facts {
            id: J::Str("cabinet".into()),
            footprint: j("[1, 1]"),
            layer: J::Str("floor".into()),
            outside: false,
            stairs: false,
            radius: 0.12,
            object: J::Obj(Vec::new()),
        };
        let pose = HashMap::new();
        let ((lo, hi), parts) = bounds(&g.gltf, &pose, &HashSet::new()).unwrap();
        let w = content_warnings(
            Some(&facts),
            &lo,
            &(lo, hi),
            &parts,
            Num::I(1),
            &g,
            &pose,
            &entry,
            false,
        )
        .unwrap();
        w.into_iter()
            .filter(|w| w.contains("opened ('door_open', use 'openDoor')"))
            .count()
    };
    assert_eq!(opened(100.0), 1, "swung back into the wall");
    assert_eq!(opened(-100.0), 0, "swung forward");
}

/// The treadmill's belt (`belt_run`) scrolls its slats one slat on, its ends hidden: most of it lands where it was.
#[test]
fn the_treadmills_belt_scrolls_by_whole_slats() {
    let path = join(
        &repo_root(),
        "web/public/assets/models/fitness/treadmill.glb",
    );
    let g = parse_glb(&std::fs::read(&path).unwrap()).unwrap();
    let nodes = g.gltf.get("nodes").unwrap();
    let a = g.gltf.get("animations").and_then(J::list).unwrap();
    let belt = a
        .iter()
        .find(|a| a.get("name").map(J::key).as_deref() == Some("belt_run"))
        .unwrap();
    let c = &key(belt, "channels").unwrap().list().unwrap()[0];
    let s = sampler(belt, c).unwrap();
    let out = key(s, "output").unwrap();
    let (first, last) = (
        first_value(&g, out, false).unwrap().unwrap(),
        first_value(&g, out, true).unwrap().unwrap(),
    );
    let node = node_index(nodes, key(key(c, "target").unwrap(), "node").unwrap()).unwrap();
    let landed = lands_on_itself(&g, node, &first, &last).unwrap().unwrap();
    assert!(landed > 0.5 && landed < 1.0, "{landed}");
}

/// `cargo review sizes` measures a model's own body: a desk's top and its height as a table leave out the screen it
/// carries (`_aside`), which the gate's heights (where residents sit) still see; a bench no content draws has the
/// seat its kind looks for.
#[test]
fn the_review_measures_the_body_not_what_it_carries() {
    let desk = named_quads(
        &["top".into(), "screen_aside".into()],
        &[[0.74, 0.0, 1.0, -0.4, 0.4], [1.2, 0.3, 0.7, -0.1, 0.1]],
    );
    let none = J::Obj(Vec::new());
    let all = Sampler::new(&desk, &none, None).unwrap().unwrap();
    let own = Sampler::own(&desk, &none).unwrap().unwrap();
    assert!(
        (all.height_by(0.74).unwrap() - 1.2).abs() < 1e-6,
        "the screen on it"
    );
    assert!(
        (own.height_by(0.74).unwrap() - 0.74).abs() < 1e-6,
        "the desk alone"
    );
    assert_eq!(
        top_near(&own, [0.72, 0.76], &[]).map(|(_, h)| (h * 100.0).round()),
        Some(74.0)
    );
    // A bench: its seat 0.45 m up, its back's top rail 0.9 m behind it.
    let bench = quads(&[[0.45, -0.8, 0.8, -0.2, 0.2], [0.9, -0.8, 0.8, -0.25, -0.2]]);
    let s = Sampler::own(&bench, &none).unwrap().unwrap();
    assert_eq!(
        seat_near(&s, [0.42, 0.48]).map(|h| (h * 100.0).round()),
        Some(45.0)
    );
}

/// A crib's changing table: its pad's top is its top, though the pad's underside is level with the top of a rail
/// in front, which itself stands on the mattress's height: only what stands on nothing found bears things.
#[test]
fn a_top_on_a_pad_is_not_taken_for_a_thing_on_a_rail() {
    // (file x is game -x) the mattress 0.56 m at game x -1 to 0; the changing board 0.78 at 0 to 1, its pad
    // 0.805 to 0.843 over the middle back; a rail in front 0.80, a bar under it 0.56.
    let crib = quads(&[
        [0.56, 0.02, 1.0, -0.5, 0.5],
        [0.78, -1.0, -0.02, -0.5, 0.5],
        [0.805, -0.7, -0.3, -0.4, 0.1],
        [0.843, -0.7, -0.3, -0.4, 0.1],
        [0.80, -0.6, -0.4, 0.2, 0.3],
        [0.56, -0.6, -0.4, 0.2, 0.3],
    ]);
    let s = Sampler::own(&crib, &J::Obj(Vec::new())).unwrap().unwrap();
    assert_eq!(
        top_near(&s, [0.85, 0.95], &[]).map(|(_, h)| (h * 1000.0).round()),
        Some(843.0)
    );
}

/// `unmoved`: a named part no clip moves, besides the still `body`, where a model has more than one still part;
/// not what it carries (`_aside`), the part its object takes in the hand (`held.part`), a held tool's, a bird's or a
/// vehicle's.
#[test]
fn a_still_part_besides_the_body_is_unmoved() {
    let model = |names: &[&str]| {
        let nodes: Vec<String> = names
            .iter()
            .map(|n| format!(r#"{{"name": "{n}", "mesh": 0}}"#))
            .collect();
        let json = format!(
            r#"{{"asset": {{"version": "2.0"}}, "scenes": [{{"nodes": {:?}}}], "nodes": [{}],
            "meshes": [{{"primitives": [{{"attributes": {{"POSITION": 0}}}}]}}],
            "animations": [{{"name": "open", "channels": [{{"sampler": 0, "target": {{"node": 1, "path": "rotation"}}}}], "samplers": []}}],
            "bufferViews": [{{"byteOffset": 0, "byteLength": 36}}],
            "accessors": [{{"bufferView": 0, "componentType": 5126, "type": "VEC3", "count": 3, "min": [0, 0, 0], "max": [0.5, 0.5, 0]}}]}}"#,
            (0..names.len()).collect::<Vec<_>>(),
            nodes.join(", ")
        );
        let bin: Vec<u8> = [0f32, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 0.0]
            .iter()
            .flat_map(|c| c.to_le_bytes())
            .collect();
        parse_glb(&glb::tests::glb(&json, &bin)).unwrap()
    };
    let facts = |object: &str| {
        vec![Facts {
            id: J::Str("thing".into()),
            footprint: j("[1, 1]"),
            layer: J::Null,
            outside: false,
            stairs: false,
            radius: 0.12,
            object: j(object),
        }]
    };
    let found = |key: &str, entry: &str, g: &Glb, f: Option<&Vec<Facts>>| -> Vec<String> {
        unmoved(key, &j(entry), g, f)
            .unwrap()
            .iter()
            .map(|w| {
                let (kind, text) = untag(w).unwrap();
                assert_eq!(kind, "unmoved");
                text.split('\'').nth(1).unwrap().to_string()
            })
            .collect()
    };
    // The door (node 1) moves; the lid and the ukulele don't; the picture is carried.
    let g = model(&["body", "door", "lid", "picture_aside", "ukulele"]);
    assert_eq!(found("model.x", "{}", &g, None), ["lid", "ukulele"]);
    let held = facts(r#"{"held": {"part": "ukulele"}}"#);
    assert_eq!(found("model.x", "{}", &g, Some(&held)), ["lid"]);
    // Moved by code, or only ever in the hand: none.
    assert!(found("tool.x", "{}", &g, None).is_empty());
    assert!(found("model.x", r#"{"bird": {"kind": "x"}}"#, &g, None).is_empty());
    let bike = facts(r#"{"vehicle": {"kind": "bike"}}"#);
    assert!(found("model.x", "{}", &g, Some(&bike)).is_empty());
    // One still part is the body, whatever its name.
    assert!(found("model.x", "{}", &model(&["cabinet", "door"]), None).is_empty());
}

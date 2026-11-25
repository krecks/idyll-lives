//! Save tests.

use crate::{Command, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.1},{"id":"energy","label":"Energy","decayPerHour":0.05}],
    "objects":[
      {"id":"fridge","name":"Fridge","interactions":[{"id":"snack","label":"Snack","minutes":10,"effects":{"hunger":0.4}}]},
      {"id":"bed","name":"Bed","footprint":[2,2],"interactions":[{"id":"sleep","label":"Sleep","minutes":480,"pose":"lie","effects":{"energy":1.0}}]}],
    "genders":[{"id":"female","label":"Female"},{"id":"male","label":"Male"}],
    "feelings":[{"id":"happy","label":"Happy","mood":0.1,"hours":3}],
    "bondPresets":{"friends":{"friendship":60}},
    "traits":[{"id":"lazy","label":"Lazy","effects":{"walkSpeed":0.8}}]}"#;

const LOT: &str = r##"{"width":12,"depth":12,"walls":[[6,0,6,10]],"doors":[{"x":6,"z":4,"axis":"z"}],
    "objects":[{"def":"fridge","x":1,"z":1,"rot":0},{"def":"bed","x":9,"z":2,"rot":3}],
    "households":[{"name":"Park","player":true},{"name":"Next door"}],
    "sims":[{"name":"Ada","gender":"female","attractedTo":["male"],"appearance":{"body":"#fff"},"traits":["lazy"],"x":3.5,"z":3.5},
            {"name":"Bo","gender":"male","household":1,"x":4.5,"z":6.5}],
    "relationships":[{"a":0,"b":1,"preset":"friends"}]}"##;

#[test]
pub(super) fn a_run_too_long_for_any_lot_is_corrupt() {
    assert!(super::expand(&format!("~01,0{}", usize::MAX)).is_err());
    assert_eq!(&*super::expand("~03,1,02").unwrap(), "000100");
}

#[test]
pub(super) fn round_trip_preserves_state() {
    let mut w = World::from_json(CONTENT, LOT, 3).unwrap();
    w.apply(Command::Use {
        sim: 1,
        object: 0,
        interaction: 0,
    })
    .unwrap();
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    crate::social::add_feeling(&mut w.sims[0].feelings, 0, &w.content.feelings, w.tick);
    // (Seven game minutes less a few ticks: not on a whole minute.)
    for _ in 0..crate::mins(7.0) - 3 {
        w.tick_once();
    }
    let json = w.save_json();
    let loaded = World::from_save_json(CONTENT, &json).unwrap();

    assert_eq!(loaded.tick, w.tick);
    assert!(loaded.autonomy, "the town-wide switch stays on");
    assert!(
        !loaded.households[0].free_will,
        "the player's household has free will off"
    );
    assert!(loaded.households[1].free_will);
    assert_eq!(loaded.objects.len(), 2);
    assert_eq!(loaded.households.len(), 2);
    assert_eq!(loaded.sims[1].household, 1);
    assert_eq!(loaded.lot.rooms(), w.lot.rooms());
    assert_eq!(loaded.sims[0].traits, ["lazy"]);
    assert_eq!(loaded.sims[0].gender, "female");
    assert_eq!(loaded.sims[0].attracted_to, ["male"]);
    assert_eq!(loaded.sims[0].appearance["body"], "#fff");
    assert!((loaded.sims[0].mods.walk_speed - 0.8).abs() < 1e-6);
    assert_eq!(loaded.sims[0].feelings.len(), 1);
    assert_eq!(loaded.relationships.get(0, 1), w.relationships.get(0, 1));
    assert_eq!(
        loaded.relationships.chemistry(0, 1),
        w.relationships.chemistry(0, 1)
    );
    for (a, b) in loaded.sims.iter().zip(&w.sims) {
        assert_eq!(a.needs, b.needs);
    }
    assert_eq!(loaded.sims[0].pos, w.sims[0].pos);
    assert_eq!(
        loaded.sims[1].queue().count() + loaded.sims[1].current().is_some() as usize,
        1
    );
    // Saving the loaded world again is stable.
    assert_eq!(
        World::from_save_json(CONTENT, &loaded.save_json())
            .unwrap()
            .save_json(),
        loaded.save_json()
    );
}

#[test]
pub(super) fn appearance_is_kept_whole() {
    // The web keeps more than colours in the appearance (a voice chosen in the creator).
    let look = serde_json::json!({"body":"#fff","voice":{"pitch":1.08,"speed":0.95,"seed":12345}});
    let lot = LOT.replace(r##"{"body":"#fff"}"##, &look.to_string());
    let w = World::from_json(CONTENT, &lot, 3).unwrap();
    assert_eq!(w.sims[0].appearance, look);
    let loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    assert_eq!(loaded.sims[0].appearance, look);
}

#[test]
pub(super) fn lying_sim_is_saved_standing_in_front_of_the_bed() {
    let mut w = World::from_json(CONTENT, LOT, 3).unwrap();
    w.autonomy = false;
    w.sims[0].needs[1] = 0.1;
    w.apply(Command::Use {
        sim: 0,
        object: 1,
        interaction: 0,
    })
    .unwrap();
    for _ in 0..crate::secs(40.0) {
        w.tick_once();
    }
    assert_eq!(w.sims[0].pose, crate::content::Pose::Lie);
    let loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    let (x, z) = loaded.sims[0].tile();
    assert!(
        !loaded.blocked()[loaded.lot.tile_index(x, z)],
        "must not spawn inside the bed"
    );
}

#[test]
pub(super) fn drops_objects_missing_from_content() {
    let w = World::from_json(CONTENT, LOT, 3).unwrap();
    let json = w
        .save_json()
        .replace(r#""def":"fridge""#, r#""def":"jukebox""#);
    let loaded = World::from_save_json(CONTENT, &json).unwrap();
    assert_eq!(loaded.objects.len(), 1);
}

#[test]
pub(super) fn old_upgrade_tasks_are_dropped_on_load() {
    let w = World::from_json(CONTENT, LOT, 3).unwrap();
    let json = w.save_json().replacen(
        r#""tasks":[]"#,
        r#""tasks":[{"type":"upgrade","object":1},{"type":"moveTo","x":2.5,"z":2.5,"directed":true}]"#,
        1,
    );
    let loaded = World::from_save_json(CONTENT, &json).unwrap();
    assert_eq!(loaded.sims[0].queue().count(), 1, "only the move is kept");
}

#[test]
pub(super) fn the_speed_and_the_one_under_super_speed_load() {
    // A file without the speed super speed drops back to: normal under super speed, else the
    // speed it's at.
    let version = super::SAVE_VERSION;
    for (speed, now, back) in [(4, 4, 1), (2, 2, 2)] {
        let file = format!(
            r#"{{"version":{version},"tick":10,"speed":{speed},"autonomy":true,"rng":5,
            "lot":{{"width":4,"depth":4,"hEdges":"00000000000000000000","vEdges":"00000000000000000000"}},
            "objects":[],"sims":[{{"name":"Old","appearance":null,"traits":[],"perks":[],"pos":[1.5,1.5],"yaw":0,
              "needs":{{"hunger":0.5}},"tasks":[]}}]}}"#
        );
        let mut w = World::from_save_json(CONTENT, &file).unwrap();
        assert_eq!(
            (w.speed, w.resume_speed),
            (now, back),
            "v{version} at {speed}"
        );
        let again = World::from_save_json(CONTENT, &w.save_json()).unwrap();
        assert_eq!((again.speed, again.resume_speed), (now, back));
        w.advance();
        assert_eq!(w.speed, now, "Old is up: the speed stays");
    }
}

#[test]
pub(super) fn diagonal_walls_survive_saving() {
    let lot = LOT.replacen(
        r#""doors":"#,
        r#""diagonals":[{"x":2,"z":8,"dir":"dp"},{"x":3,"z":9,"dir":"dp","kind":"window"},
            {"x":4,"z":8,"dir":"dn","kind":"door"}],"doors":"#,
        1,
    );
    let w = World::from_json(CONTENT, &lot, 3).unwrap();
    let json = w.save_json();
    let saved: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(saved["version"], super::SAVE_VERSION);
    let diags = saved["lot"]["diagonals"].as_str().unwrap();
    assert_eq!(diags.len(), 12 * 12);
    let loaded = World::from_save_json(CONTENT, &json).unwrap();
    use crate::lot::{DiagDir, Diagonal, Edge};
    assert_eq!(
        loaded.lot.diag(3, 9),
        Some(Diagonal {
            dir: DiagDir::Dp,
            edge: Edge::Window
        })
    );
    assert_eq!(
        loaded.lot.diag(4, 8),
        Some(Diagonal {
            dir: DiagDir::Dn,
            edge: Edge::Door
        })
    );
    assert_eq!(loaded.lot.half_rooms(), w.lot.half_rooms());
    assert_eq!(loaded.save_json(), json);
}

#[test]
pub(super) fn saves_without_diagonals_still_load() {
    // A lot without diagonal walls writes no `diagonals`.
    let w = World::from_json(CONTENT, LOT, 3).unwrap();
    let mut saved: serde_json::Value = serde_json::from_str(&w.save_json()).unwrap();
    assert!(
        saved["lot"].get("diagonals").is_none(),
        "lots without diagonals don't write them"
    );
    let loaded = World::from_save_json(CONTENT, &saved.to_string()).unwrap();
    assert!(loaded.lot.diagonals().iter().all(Option::is_none));
    assert_eq!(loaded.lot.rooms(), w.lot.rooms());
    // A corrupt or wrongly sized diagonal string is an error, not a silent change.
    saved["lot"]["diagonals"] = "01".into();
    assert!(World::from_save_json(CONTENT, &saved.to_string()).is_err());
}

#[test]
pub(super) fn windows_survive_saving() {
    let lot = LOT.replacen(
        r#""doors":"#,
        r#""windows":[{"x":6,"z":8,"axis":"z"}],"doors":"#,
        1,
    );
    let w = World::from_json(CONTENT, &lot, 3).unwrap();
    assert_eq!(w.lot.v_edge(6, 8), crate::lot::Edge::Window);
    let json = w.save_json();
    let saved: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(saved["lot"]["vEdges"].as_str().unwrap().contains('3'));
    let loaded = World::from_save_json(CONTENT, &json).unwrap();
    assert_eq!(loaded.lot.v_edge(6, 8), crate::lot::Edge::Window);
    assert_eq!(loaded.lot.v_edge(6, 4), crate::lot::Edge::Door);
    assert_eq!(loaded.save_json(), json);
}

/// A saved game as JSON, edited by `f`.
pub(super) fn edited(f: impl FnOnce(&mut serde_json::Value)) -> String {
    let w = World::from_json(CONTENT, LOT, 3).unwrap();
    let mut saved: serde_json::Value = serde_json::from_str(&w.save_json()).unwrap();
    f(&mut saved);
    saved.to_string()
}

/// Damaged saves are refused when they're read, not by a panic later: a household on a plot
/// the town doesn't have, a baby expected by someone who isn't there, an item id past the
/// last one there can be.
#[test]
pub(super) fn damaged_saves_are_refused_on_loading() {
    let refused = |json: String, why: &str| {
        let err = World::from_save_json(CONTENT, &json)
            .err()
            .unwrap_or_else(|| panic!("{why}: loaded"));
        assert!(err.to_string().starts_with("corrupt save"), "{why}: {err}");
    };
    refused(
        edited(|s| s["households"][0]["plot"] = 7.into()),
        "unknown plot",
    );
    refused(
        edited(|s| {
            s["households"][0]["expecting"] = serde_json::json!({"parents": [0, 9], "due": 3})
        }),
        "unknown parent",
    );
    let item = serde_json::json!([{"id": u32::MAX, "def": "x", "at": "floor", "x": 1, "z": 1, "quality": 1.0, "madeAt": 0, "household": 0}]);
    // (An item the content doesn't know is dropped: give the content one.)
    let content = CONTENT.replacen(
        r#""needs":"#,
        r#""items":[{"id":"x","label":"X","model":"x"}],"needs":"#,
        1,
    );
    let json = edited(|s| s["items"] = item);
    let err = World::from_save_json(&content, &json)
        .err()
        .map(|e| e.to_string());
    assert_eq!(
        err.as_deref(),
        Some("corrupt save: an item id is out of range")
    );
    // An expected baby by residents who are there loads.
    let json = edited(|s| {
        s["households"][0]["expecting"] = serde_json::json!({"parents": [0, 1], "due": 3})
    });
    assert!(World::from_save_json(CONTENT, &json).is_ok());
}

/// Two items saved with one id, or a next id with no room left, are refused: one item would be
/// lost without a word, or the next one added would have no id to take.
#[test]
pub(super) fn bad_item_ids_are_refused() {
    let item = |id: u32| {
        serde_json::json!({"id": id, "def": "x", "at": "floor", "x": 1, "z": 1, "quality": 1.0, "madeAt": 0, "household": 0})
    };
    // (An item the content doesn't know is dropped: give the content one.)
    let content = CONTENT.replacen(
        r#""needs":"#,
        r#""items":[{"id":"x","label":"X","model":"x"}],"needs":"#,
        1,
    );
    let json = edited(|s| s["items"] = serde_json::json!([item(3), item(3)]));
    let err = World::from_save_json(&content, &json)
        .err()
        .map(|e| e.to_string());
    assert_eq!(err.as_deref(), Some("corrupt save: two items share an id"));
    let json = edited(|s| s["nextItem"] = u32::MAX.into());
    let err = World::from_save_json(CONTENT, &json)
        .err()
        .map(|e| e.to_string());
    assert_eq!(
        err.as_deref(),
        Some("corrupt save: the next item id is out of range")
    );
}

/// Numbers out of range in a save (here written as 1e39, which reads as infinity) are brought
/// back in: a facing, friendship and romance (±100), a nap or a visit's end.
#[test]
pub(super) fn numbers_out_of_range_load_in_range() {
    let json = edited(|s| {
        s["sims"][0]["yaw"] = serde_json::json!(1e39);
        s["sims"][0]["napMinutes"] = serde_json::json!(1e39);
        s["sims"][0]["visiting"] = serde_json::json!([0, 1e39]);
        for r in s["relationships"].as_array_mut().unwrap() {
            r["friendship"] = serde_json::json!(1e39);
            r["romance"] = serde_json::json!(-1e39);
        }
    });
    let w = World::from_save_json(CONTENT, &json).unwrap();
    assert_eq!(w.sims[0].yaw, 0.0);
    let r = w.relationships.get(0, 1);
    assert_eq!((r.friendship, r.romance), (100.0, -100.0));
    assert_eq!(w.sims[0].nap_until, w.tick + 1, "no nap for ever");
}

/// Whatever goes wrong while playing, the save can be loaded: a value that isn't a number
/// (NaN, infinity) is written as a usable one, never as `null`.
#[test]
pub(super) fn broken_numbers_never_make_a_save_unloadable() {
    let mut w = World::from_json(CONTENT, LOT, 3).unwrap();
    let s = &mut w.sims[0];
    s.pos = [f32::NAN, 3.0];
    s.yaw = f32::INFINITY;
    s.age = f32::NAN;
    s.needs[0] = f32::NAN;
    s.skills[0] = f32::NEG_INFINITY;
    w.objects[0].wear = f32::NAN;
    w.relationships.adjust(0, 1, f32::NAN, 0.0);
    w.exits.push([f32::NAN, 1.0]);
    let loaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    assert!(loaded.sims[0].pos.iter().all(|p| p.is_finite()));
    assert_eq!(loaded.sims[0].needs[0], 0.75);
    assert_eq!(loaded.relationships.get(0, 1).friendship, 0.0);
    assert!(loaded.exits.is_empty());
}

/// A file older than this build opens (`OLDEST_SAVE_VERSION`) is refused, as one newer than
/// it is: nothing is migrated.
#[test]
pub(super) fn a_file_of_another_version_is_refused() {
    let w = World::from_json(CONTENT, LOT, 1).unwrap();
    let json = w.save_json();
    let mut file: serde_json::Value = serde_json::from_str(&json).unwrap();
    for v in [1, super::OLDEST_SAVE_VERSION - 1, super::SAVE_VERSION + 1] {
        file["version"] = v.into();
        assert!(
            World::from_save_json(CONTENT, &file.to_string()).is_err(),
            "version {v}"
        );
    }
    file["version"] = super::OLDEST_SAVE_VERSION.into();
    assert!(World::from_save_json(CONTENT, &file.to_string()).is_ok());
}

/// The seed the game started from is kept.
#[test]
pub(super) fn the_starting_seed_is_saved() {
    let w = World::from_json(CONTENT, LOT, 1234).unwrap();
    assert_eq!(w.seed, 1234);
    let json = w.save_json();
    assert_eq!(World::from_save_json(CONTENT, &json).unwrap().seed, 1234);
}

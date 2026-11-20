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
pub(super) fn saves_with_moodlets_still_load() {
    // Saves written before the moodlet → feeling rename store `moodlets` per Sim.
    let mut w = World::from_json(CONTENT, LOT, 3).unwrap();
    crate::social::add_feeling(&mut w.sims[0].feelings, 0, &w.content.feelings, w.tick);
    let mut saved: serde_json::Value = serde_json::from_str(&w.save_json()).unwrap();
    let sim = saved["sims"][0].as_object_mut().unwrap();
    let feelings = sim.remove("feelings").expect("new saves write `feelings`");
    assert_eq!(feelings[0]["id"], "happy");
    sim.insert("moodlets".into(), feelings);
    let loaded = World::from_save_json(CONTENT, &saved.to_string()).unwrap();
    assert_eq!(loaded.sims[0].feelings.len(), 1);
    assert_eq!(
        loaded.content.feelings[loaded.sims[0].feelings[0].def].id,
        "happy"
    );
}

#[test]
pub(super) fn version_one_saves_still_load() {
    let v1 = r#"{"version":1,"tick":10,"speed":1,"autonomy":true,"rng":5,
        "lot":{"width":4,"depth":4,"hEdges":"00000000000000000000","vEdges":"00000000000000000000"},
        "objects":[],"sims":[{"name":"Old","appearance":null,"traits":[],"perks":[],"pos":[1.5,1.5],"yaw":0,
          "needs":{"hunger":0.5},"tasks":[]}]}"#;
    let w = World::from_save_json(CONTENT, v1).unwrap();
    assert_eq!(w.sims[0].gender, "female", "defaults to the first gender");
    assert_eq!(w.sims[0].attracted_to.len(), 2);
}

#[test]
pub(super) fn the_speeds_of_older_saves() {
    // Before v40, 4 and 5 were 30× and 60×: super speed now (30× with Old awake), 10× under it;
    // the speed under super speed is saved.
    for (version, speed, now, back) in [(39, 5, 4, 3), (39, 4, 4, 3), (39, 2, 2, 2), (40, 4, 4, 1)]
    {
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
    // A version 4 save (before diagonal walls): no `diagonals` in the lot.
    let w = World::from_json(CONTENT, LOT, 3).unwrap();
    let mut saved: serde_json::Value = serde_json::from_str(&w.save_json()).unwrap();
    assert!(
        saved["lot"].get("diagonals").is_none(),
        "lots without diagonals don't write them"
    );
    saved["version"] = 4.into();
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

/// v36: an older file's ticks are rescaled as it loads, from 20 to a real second and 20 to a
/// game minute (`Rescale`); at those rates still, a v35 file loads just as it was saved, a car
/// on its way and the story as they were.
#[test]
pub(super) fn a_v35_file_loads_as_it_was_saved() {
    const CONTENT: &str = r#"{
        "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.02}],
        "objects":[{"id":"chair","name":"Chair","interactions":[{"id":"sit","label":"Sit","minutes":30,"tags":["lounge"]}]}],
        "careers":[{"id":"office","label":"Office","workEffects":{"hunger":-0.2},"levels":[
            {"title":"Intern","pay":100,"start":9,"hours":4,"days":[0,1,2,3,4,5,6]}]}],
        "economy":{"startingFunds":500}}"#;
    // A house on a street, its car parked beside it; two grown-ups at the same office.
    const TOWN: &str = r#"{"width":60,"depth":14,"exits":[[0.5,9.5],[59.5,9.5]],
        "plots":[{"name":"Home","x":20,"z":0,"w":20,"d":8,"entry":[25.5,7.5]}],
        "households":[{"name":"Ones","plot":0}],
        "sims":[{"name":"Ada","household":0,"job":{"career":"office"},"x":25.5,"z":3.5},
                {"name":"Bo","household":0,"job":{"career":"office"},"x":26.5,"z":3.5}],
        "cars":[{"plot":0,"x":34.0,"z":3.0,"yaw":0.0,"street":9.5,"len":4.4,"wid":1.8,"body":1,"paint":2}]}"#;
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.autonomy = false;
    let by = crate::clock::tick_at(1, 10.0 * 60.0).unwrap();
    while !(w.cars[0].state == crate::cars::CarState::Leaving && w.cars[0].progress > 0.0) {
        w.tick_once();
        assert!(w.tick < by, "the car never set off");
    }
    w.events
        .push(w.tick - 7, crate::social::EventKind::Met, 0, 1, None);
    let json = w.save_json();
    let mut old: serde_json::Value = serde_json::from_str(&json).unwrap();
    old["version"] = 35.into();
    let loaded = World::from_save_json(CONTENT, &old.to_string()).unwrap();
    // (As this build counts them: the same at 20 and 20.)
    let r = super::Rescale::of(35);
    assert_eq!(loaded.tick, r.tick(w.tick));
    assert_eq!(loaded.cars[0].since, r.tick(w.cars[0].since));
    assert_eq!(
        loaded.cars[0].progress.to_bits(),
        r.route(w.cars[0].progress).to_bits()
    );
    let ticks = |w: &World| w.events.iter().map(|e| e.tick).collect::<Vec<_>>();
    assert_eq!(
        ticks(&loaded),
        ticks(&w).into_iter().map(|t| r.tick(t)).collect::<Vec<_>>()
    );
    if r == super::Rescale::of(super::SAVE_VERSION) {
        assert_eq!(
            loaded.save_json(),
            World::from_save_json(CONTENT, &json).unwrap().save_json()
        );
    }
}

/// From 20 ticks a game minute to 75 (and 20 a real second to 60): each tick of the clock is
/// the same minute of the same day, the same tick stays the same and never stays never; places
/// along a way are three times the ticks.
#[test]
pub(super) fn older_ticks_keep_their_minute_at_another_rate() {
    let r = super::Rescale {
        game: (75, 20),
        drive: 3.0,
    };
    // (The clock by hand: tick 0 is 08:00 on day 1.)
    let at = |t: u64, per_minute: u64| {
        let m = t + 8 * 60 * per_minute;
        (
            m / (1440 * per_minute),
            m % (1440 * per_minute) / per_minute,
        )
    };
    for t in [0, 1, 19, 20, 21, 1_199, 9_599, 9_600, 28_813, 123_456_789] {
        assert_eq!(at(t, 20), at(r.tick(t), 75), "tick {t}");
    }
    assert_eq!(r.tick(0), 0);
    assert_eq!(r.tick(1_000), 3_750);
    // A traffic light's stage, at it for 100 ticks (5 seconds) of the file's clock at 1000: 5
    // seconds before the clock's tick still (300 ticks, not the game minutes' 375).
    assert_eq!(r.drive_since(900, 1_000), 3_450);
    assert_eq!(r.drive_since(1_000, 1_000), 3_750);
    assert_eq!(r.drive_since(0, 10), 8);
    let car: crate::cars::Car = serde_json::from_value(serde_json::json!({
        "plot":0,"household":0,"spot":[34.0,3.0],"yaw":0.0,"street":9.5,"len":4.4,"wid":1.8,
        "body":1,"paint":2,"owners":[0],"state":"leaving","since":1000,"readyAt":2000,
        "progress":40.5,"detour":{"home":false,"from":30.0,"back":12.0,"avoid":3},
        "passBy":{"home":false,"taxi":1,"from":50.0,"out":55.0,"back":70.0,"to":75.0,"off":-2.0,"at":[1.0,2.0]}}))
    .unwrap();
    let cars = r.cars(std::slice::from_ref(&car));
    let c = &cars[0];
    assert_eq!((c.since, c.ready_at), (3_750, 7_500));
    assert_eq!(c.progress, 121.5);
    let d = c.detour.unwrap();
    assert_eq!((d.from, d.back, d.avoid), (90.0, 36.0, 3));
    let p = c.pass_by.unwrap();
    assert_eq!(
        (p.from, p.out, p.back, p.to, p.off),
        (150.0, 165.0, 210.0, 225.0, -2.0)
    );
    let mut taxis: crate::taxi::Taxis = serde_json::from_value(serde_json::json!({
        "runs":[{"id":0,"group":[0,1],"enter":100,"dueOut":220,"plannedOut":200,"depart":0,
            "stops":[{"sim":0,"plot":0,"due":160,"leave":140,"state":"booked"}],
            "progress":12.0,"since":160}]}))
    .unwrap();
    r.taxis(&mut taxis);
    let run = &taxis.runs[0];
    assert_eq!(
        (
            run.enter,
            run.due_out,
            run.planned_out,
            run.depart,
            run.since
        ),
        (375, 825, 750, 0, 600)
    );
    assert_eq!((run.stops[0].due, run.stops[0].leave), (600, 525));
    assert_eq!(run.progress, 36.0);
    // Today's file needs nothing.
    let today = super::Rescale::of(super::SAVE_VERSION);
    assert_eq!(today.drive_since(900, 1_000), 900);
    assert_eq!(
        super::Rescale::of(super::SAVE_VERSION),
        super::Rescale {
            game: (1, 1),
            drive: 1.0
        }
    );
    // (A v36 file's cars timed in real seconds: a place along the way a quarter of the ticks
    // on, the clock as it was; a v35 file's both.)
    assert_eq!(
        super::Rescale::of(36),
        super::Rescale {
            game: (1, 1),
            drive: 1.0 / crate::DRIVE_SCALE
        }
    );
    assert_eq!(
        super::Rescale::of(35).drive,
        crate::DRIVE_TICKS_PER_SECOND / super::OLD_TICKS_PER_SECOND as f32
    );
}

/// v23: the seed the game started from is kept (absent in older saves: 0, unknown).
#[test]
pub(super) fn the_starting_seed_is_saved() {
    let w = World::from_json(CONTENT, LOT, 1234).unwrap();
    assert_eq!(w.seed, 1234);
    let json = w.save_json();
    assert_eq!(World::from_save_json(CONTENT, &json).unwrap().seed, 1234);
    let mut old: serde_json::Value = serde_json::from_str(&json).unwrap();
    old["version"] = 22.into();
    old.as_object_mut().unwrap().remove("seed");
    assert_eq!(
        World::from_save_json(CONTENT, &old.to_string())
            .unwrap()
            .seed,
        0
    );
}

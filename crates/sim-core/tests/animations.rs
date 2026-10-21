//! What the renderer is told a Sim is doing: the `object`, `action`, `objectDef` and
//! `interaction` snapshot fields, interaction `anim` tags and their fallbacks.

use sim_core::snapshot::{self, HEADER_LEN, SIM_STRIDE, sim};
use sim_core::{Command, Content, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.01},
             {"id":"energy","label":"Energy","decayPerHour":0.01},
             {"id":"social","label":"Social","decayPerHour":0.01}],
    "animations":["eat","cook","sleep","talk"],
    "objects":[
      {"id":"fridge","name":"Fridge","price":100,"interactions":[
        {"id":"snack","label":"Snack","anim":"eat","minutes":30,"effects":{"hunger":0.5},"tags":["food"]},
        {"id":"meal","label":"Meal","minutes":30,"effects":{"hunger":0.5},"tags":["food","cooking"]},
        {"id":"stare","label":"Stare","minutes":30,"tags":["food"]}]},
      {"id":"bed","name":"Bed","footprint":[2,2],"price":100,"interactions":[
        {"id":"sleep","label":"Sleep","minutes":480,"pose":"lie","effects":{"energy":1.0},"tags":["sleep"]}]}],
    "socials":[
      {"id":"chat","label":"Chat","minutes":10,"tags":["social"],"acceptance":{"base":1.0},
       "needs":{"social":0.4},"targetNeeds":{"social":0.4}}],
    "bondPresets":{"roommates":{"friendship":20}},"socialRules":{"defaultBond":"roommates"},
    "economy":{"startingFunds":100}}"#;

const TOWN: &str = r#"{"width":14,"depth":14,
    "plots":[{"name":"Home","x":0,"z":0,"w":14,"d":14,"entry":[6.5,12.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "objects":[{"def":"fridge","x":1,"z":1,"rot":0},{"def":"fridge","x":5,"z":1,"rot":0},
               {"def":"bed","x":9,"z":1,"rot":0}],
    "sims":[{"name":"Ada","x":6.5,"z":8.5},{"name":"Bo","x":10.5,"z":10.5}]}"#;

fn world() -> World {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.autonomy = false;
    for s in &mut w.sims {
        s.needs = [0.5; 8];
    }
    w
}

/// `(object, action)` from the snapshot row of Sim `i`.
fn row(w: &World, i: usize) -> (f32, f32) {
    let mut out = vec![0.0; snapshot::CAPACITY];
    snapshot::write(w, &mut out);
    let o = HEADER_LEN + i * SIM_STRIDE;
    (out[o + sim::OBJECT], out[o + sim::ACTION])
}

/// `(objectDef, interaction)` from the snapshot row of Sim `i`.
fn used(w: &World, i: usize) -> (f32, f32) {
    let mut out = vec![0.0; snapshot::CAPACITY];
    snapshot::write(w, &mut out);
    let o = HEADER_LEN + i * SIM_STRIDE;
    (out[o + sim::OBJECT_DEF], out[o + sim::INTERACTION])
}

fn action(w: &World, tag: &str) -> f32 {
    w.content.animations.iter().position(|a| a == tag).unwrap() as f32
}

fn use_object(w: &mut World, object: u32, interaction: usize) {
    w.apply(Command::Use {
        sim: 0,
        object,
        interaction,
    })
    .unwrap();
}

/// Ticks until Sim 0 is using its object (at most a minute of walking).
fn walk_up(w: &mut World) {
    for _ in 0..60 * sim_core::TICKS_PER_SECOND {
        w.tick_once();
        if !w.sims[0].is_moving() {
            return;
        }
    }
    panic!("never arrived");
}

#[test]
fn snapshot_carries_object_and_action_while_using() {
    let mut w = world();
    assert_eq!(row(&w, 0), (-1.0, -1.0), "idle");

    use_object(&mut w, 1, 0);
    w.tick_once();
    assert!(w.sims[0].is_moving());
    assert_eq!(
        row(&w, 0),
        (1.0, -1.0),
        "walking to the fridge: object set, no action yet"
    );
    walk_up(&mut w);
    assert_eq!(row(&w, 0), (1.0, action(&w, "eat")));
    assert_eq!(row(&w, 1), (-1.0, -1.0), "the other Sim is idle");

    // 40 game minutes (one per real second).
    for _ in 0..40 * sim_core::TICKS_PER_SECOND {
        w.tick_once();
    }
    assert!(w.sims[0].current().is_none(), "finished");
    assert_eq!(row(&w, 0), (-1.0, -1.0));
}

#[test]
fn snapshot_carries_the_definition_and_interaction_in_use() {
    let mut w = world();
    assert_eq!(used(&w, 0), (-1.0, -1.0), "idle");

    // The bed (definition 1), its only interaction: known while walking there already.
    use_object(&mut w, 2, 0);
    w.tick_once();
    assert_eq!(used(&w, 0), (1.0, 0.0), "walking to the bed");
    walk_up(&mut w);
    assert_eq!(used(&w, 0), (1.0, 0.0), "in bed");
    assert_eq!(used(&w, 1), (-1.0, -1.0), "the other Sim does nothing");

    // The second fridge (definition 0), its third interaction.
    let mut w = world();
    use_object(&mut w, 1, 2);
    walk_up(&mut w);
    assert_eq!(used(&w, 0), (0.0, 2.0));
}

#[test]
fn interactions_without_anim_fall_back_to_their_tags() {
    let mut w = world();
    let fridge = &w.content.objects[0].interactions;
    assert_eq!(fridge[1].anim, Some(1), "cooking tag: cook, not eat");
    assert_eq!(fridge[2].anim, None, "'drink' isn't declared: none");
    assert_eq!(w.content.objects[1].interactions[0].anim, Some(2), "sleep");

    use_object(&mut w, 2, 0);
    walk_up(&mut w);
    assert_eq!(row(&w, 0), (2.0, action(&w, "sleep")));
}

#[test]
fn conversations_show_talk() {
    let mut w = world();
    w.apply(Command::Social {
        sim: 0,
        target: 1,
        social: 0,
    })
    .unwrap();
    let talk = action(&w, "talk");
    let mut talked = false;
    for _ in 0..60 * sim_core::TICKS_PER_SECOND {
        w.tick_once();
        if w.sims[0].conversation(&w.content).is_some() {
            talked = true;
            assert_eq!(row(&w, 0), (-1.0, talk));
            assert_eq!(row(&w, 1), (-1.0, talk), "the target talks too");
        }
    }
    assert!(talked);
}

#[test]
fn object_ids_stay_valid_after_a_sell() {
    let mut w = world();
    use_object(&mut w, 1, 0);
    walk_up(&mut w);
    let (x, z) = (w.objects[1].x, w.objects[1].z);
    let fridge = w.objects[1].stable;

    // Selling an object listed before it doesn't change the fridge's id.
    w.apply(Command::Sell { household: 0, object: w.objects[0].stable }).unwrap();
    let (object, act) = row(&w, 0);
    assert_eq!(object, fridge as f32);
    assert_eq!(act, action(&w, "eat"), "still eating");
    let used = &w.objects[w.index_of(fridge).unwrap()];
    assert_eq!((used.x, used.z), (x, z), "the same fridge");
    assert!(used.users().any(|u| u == 0));
    // The UI's list of what they do names it the same way.
    let mut sync = sim_core::view::UiSync::new();
    sync.inspect(Some(0));
    let ui: serde_json::Value = serde_json::from_str(&sync.update_json(&w)).unwrap();
    let ada = ui["sims"].as_array().unwrap().iter().find(|s| s["id"] == 0).unwrap();
    assert_eq!(ada["actions"][0]["object"], fridge, "{}", ada["actions"]);

    // Selling the object in use stops the Sim.
    w.apply(Command::Sell { household: 0, object: fridge }).unwrap();
    assert_eq!(row(&w, 0), (-1.0, -1.0));

    // A second click on Sell, or a menu still open on it: nothing happens, and no error.
    let funds = w.households[0].funds;
    w.apply(Command::Sell { household: 0, object: fridge }).expect("already sold: nothing to do");
    w.apply(Command::Use { sim: 0, object: fridge, interaction: 0 }).expect("gone: nothing to do");
    assert_eq!(w.households[0].funds, funds);
    assert_eq!(row(&w, 0), (-1.0, -1.0));
    // An id never handed out is still an error.
    assert!(w.apply(Command::Sell { household: 0, object: 1000 }).is_err());
}

#[test]
fn unknown_anim_tags_are_rejected() {
    let bad = CONTENT.replace(r#""anim":"eat""#, r#""anim":"juggle""#);
    let err = Content::from_json(&bad).unwrap_err();
    assert!(
        err.0.contains("fridge.snack") && err.0.contains("unknown anim 'juggle'"),
        "{}",
        err.0
    );
    // Without an `animations` list, any `anim` is unknown.
    let undeclared = CONTENT.replace(r#""animations":["eat","cook","sleep","talk"],"#, "");
    assert!(Content::from_json(&undeclared).is_err());
}

#[test]
fn layout_lists_the_content_animations() {
    let w = world();
    let layout: serde_json::Value =
        serde_json::from_str(&snapshot::layout_json(&w.content)).unwrap();
    assert_eq!(
        layout["actions"],
        serde_json::json!(["eat", "cook", "sleep", "talk"])
    );
    assert_eq!(layout["sim"]["object"], sim::OBJECT);
    assert_eq!(layout["sim"]["action"], sim::ACTION);
    assert_eq!(layout["sim"]["objectDef"], sim::OBJECT_DEF);
    assert_eq!(layout["sim"]["interaction"], sim::INTERACTION);
}

#[test]
fn walkers_tell_where_their_way_goes() {
    let mut w = world();
    let at = |w: &World| {
        let mut out = vec![0.0; snapshot::CAPACITY];
        snapshot::write(w, &mut out);
        let o = HEADER_LEN;
        let f = |k: usize| out[o + k];
        ([f(sim::X), f(sim::Z)], [f(sim::WAY_X), f(sim::WAY_Z)], [f(sim::AHEAD_X), f(sim::AHEAD_Z)])
    };
    // Standing: all three where she stands.
    let (pos, way, ahead) = at(&w);
    assert!(way == pos && ahead == pos);
    // Walking 4 m straight along x: the way and the look ahead are on that line, 1.5 m on.
    w.apply(Command::MoveTo { sim: 0, x: 10.5, z: 8.5 }).unwrap();
    for _ in 0..10 {
        w.tick_once();
    }
    let (pos, way, ahead) = at(&w);
    assert!(w.sims[0].is_moving());
    assert!((way[0] - 10.5).abs() < 0.01 && (way[1] - 8.5).abs() < 0.01, "{way:?}");
    let on = (ahead[0] - pos[0]).hypot(ahead[1] - pos[1]);
    assert!((on - snapshot::AHEAD).abs() < 0.01 && (ahead[1] - 8.5).abs() < 0.01, "{pos:?} {ahead:?}");
    let layout: serde_json::Value = serde_json::from_str(&snapshot::layout_json(&w.content)).unwrap();
    assert_eq!(layout["sim"]["aheadZ"], sim::AHEAD_Z);
}

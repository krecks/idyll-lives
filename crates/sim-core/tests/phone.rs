//! Everyone's smartphone (`phone.rs`, content `phone`): texting and calling people elsewhere,
//! inviting them over or asking to come over to theirs, anywhere at all. The phone box is gone
//! from the market square, and residents call friends from home.

mod common;

use serde_json::Value;
use sim_core::phone::{self, Answer};
use sim_core::social::EventKind;
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, World, clock};

const CONTENT: &str = r#"{
    "needs":[{"id":"social","label":"Social","decayPerHour":0.01},{"id":"fun","label":"Fun","decayPerHour":0.01},
             {"id":"energy","label":"Energy","decayPerHour":0.01}],
    "objects":[
      {"id":"sofa","name":"Sofa","footprint":[2,1],"slots":2,"interactions":[{"id":"sit","label":"Sit","minutes":30,"pose":"sit","effects":{"fun":0.01},"tags":["lounge"]}]},
      {"id":"bed","name":"Bed","footprint":[2,2],"interactions":[{"id":"sleep","label":"Sleep","minutes":480,"pose":"lie","effects":{"energy":1.0},"tags":["sleep"]}]}],
    "feelings":[{"id":"turnedAway","label":"Turned away","mood":-0.15,"hours":6}],
    "animations":["knock","talk","phone"],
    "socials":[{"id":"chat","label":"Chat","minutes":10,"tags":["social"],"acceptance":{"base":1.0},"needs":{"social":0.3}}],
    "bondPresets":{"friends":{"friendship":60},"enemies":{"friendship":-60}},
    "dayRhythm":{"sleepTags":["sleep"]},
    "visits":{"needs":{"social":0.5},"hours":2,"minFriendship":10,"earliestHour":9,"latestHour":21,
      "door":{"waitMinutes":10,"welcomeFriendship":-20,"greet":"chat","turnedAwayFeeling":"turnedAway","knockAnim":"knock"}},
    "phone":{"anim":"phone","seatRange":0,"actions":[
      {"id":"text","label":"Text","minutes":10,"contact":true,"message":true,"tags":["social","phone"],"acceptance":{"base":1.0},
       "needs":{"social":0.1},"contactNeeds":{"social":0.05},"success":{"friendship":1,"targetFriendship":1}},
      {"id":"call","label":"Call","minutes":20,"contact":true,"tags":["social","phone"],"acceptance":{"base":1.0},
       "needs":{"social":0.3},"contactNeeds":{"social":0.2},"success":{"friendship":2,"targetFriendship":2}},
      {"id":"invite","label":"Invite over","minutes":5,"contact":true,"meet":"here","tags":["social","phone"],"acceptance":{"base":1.0},
       "needs":{"social":0.05},"success":{"friendship":1,"targetFriendship":1}},
      {"id":"askOver","label":"Ask to come over","minutes":5,"contact":true,"meet":"there","autonomous":false,"tags":["social","phone"],
       "acceptance":{"base":1.0},"needs":{"social":0.05}},
      {"id":"browse","label":"Scroll","minutes":20,"tags":["fun","phone"],"needs":{"fun":0.1}}]},
    "economy":{"startingFunds":500}}"#;

/// Two houses side by side: Ada's (west, with a sofa) and Bo's (east, with a bed).
fn town(bond: &str) -> String {
    format!(
        r#"{{"width":40,"depth":14,
        "plots":[{{"name":"West","x":0,"z":0,"w":20,"d":10,"entry":[10.5,9.5]}},{{"name":"East","x":20,"z":0,"w":20,"d":10,"entry":[30.5,9.5]}}],
        "households":[{{"name":"Ada","plot":0,"player":true}},{{"name":"Bo","plot":1}}],
        "objects":[{{"def":"sofa","x":3,"z":2,"rot":0}},{{"def":"bed","x":34,"z":2,"rot":0}}],
        "sims":[{{"name":"Ada","household":0,"x":4.5,"z":5.5}},{{"name":"Bo","household":1,"x":25.5,"z":5.5}}],
        "relationships":[{{"a":0,"b":1,"preset":"{bond}"}}]}}"#
    )
}

/// The town at 11:00 on day 1 (within visiting hours), nobody choosing on their own.
fn world(content: &str, bond: &str) -> World {
    let mut w = World::from_json(content, &town(bond), 7).unwrap();
    w.autonomy = false;
    while w.tick < clock::tick_at(1, 11.0 * 60.0).unwrap() {
        w.tick_once();
    }
    w
}

fn minutes(w: &mut World, m: f32) {
    let end = w.tick + (m / sim_core::MINUTES_PER_TICK) as u64;
    while w.tick < end {
        w.tick_once();
    }
}

fn action(w: &World, id: &str) -> usize {
    w.content.phone.action_index(id).unwrap_or_else(|| panic!("a phone action '{id}'"))
}

fn on_the_phone(w: &World, i: usize) -> Option<(usize, Option<u32>, Answer)> {
    match w.sims[i].current()?.task.kind {
        TaskKind::Phone { action, contact, answer, .. } => Some((action, contact, answer)),
        _ => None,
    }
}

fn plot_of(w: &World, i: usize) -> Option<u32> {
    let (x, z) = w.sims[i].tile();
    w.plot_at(x, z)
}

#[test]
fn every_resident_old_enough_has_a_phone() {
    let content = common::content();
    let w = World::from_json(&content, &common::town(&content, 7, 3), 7).unwrap();
    let ids: Vec<&str> = w.content.phone.actions.iter().map(|a| a.id.as_str()).collect();
    for id in ["text", "call", "invite", "askOver", "browse"] {
        assert!(ids.contains(&id), "the phone can {id}");
    }
    // Children and up (from the content's `fromStage`), not babies.
    let child = w.content.life.stages.iter().find(|s| s.id == "child").unwrap().from;
    assert_eq!(w.content.phone.from_age, child);
    assert!(phone::has_phone(&w.content, child) && !phone::has_phone(&w.content, 0.5));
    let mut grown_ups = 0;
    for s in w.sims.iter().filter(|s| s.adult(&w.content)) {
        grown_ups += 1;
        assert!(phone::has_phone(&w.content, s.age), "{} has no phone", s.name);
        assert_eq!(phone::unusable(&w, s.id as usize), None, "{} can't use theirs", s.name);
    }
    assert!(grown_ups > 10);
    // The UI is told who's in the phone of the player's residents.
    let ui: Value = serde_json::from_str(&sim_core::view::ui_state_json(&w)).unwrap();
    let mine = ui["sims"].as_array().unwrap().iter().find(|s| s["detail"] == true).expect("a resident with details");
    let contacts = mine["phone"]["contacts"].as_array().expect("a phone with contacts");
    assert!(!contacts.is_empty());
    assert_eq!(contacts[0]["why"].as_array().unwrap().len(), ids.len());
}

#[test]
fn a_call_cheers_both_up_and_brings_them_closer() {
    let mut w = world(CONTENT, "friends");
    w.sims[0].needs[0] = 0.2;
    w.sims[1].needs[0] = 0.2;
    let before = (w.relationships.get(0, 1).friendship, w.relationships.get(1, 0).friendship);
    let bo_at = w.sims[1].pos;
    w.apply(Command::Phone { sim: 0, action: action(&w, "call"), contact: Some(1) }).unwrap();
    minutes(&mut w, 1.0);
    let (_, contact, answer) = on_the_phone(&w, 0).expect("on the phone");
    assert_eq!((contact, answer), (Some(1), Answer::Yes));
    // On the spot: no walking over to anyone.
    assert_eq!(plot_of(&w, 0), Some(0));
    minutes(&mut w, 25.0);
    assert!(on_the_phone(&w, 0).is_none(), "hung up");
    assert!(w.sims[0].needs[0] > 0.45, "the caller's social: {}", w.sims[0].needs[0]);
    assert!(w.sims[1].needs[0] > 0.35, "the friend's social: {}", w.sims[1].needs[0]);
    assert_eq!(w.sims[1].pos, bo_at, "the friend stayed home");
    let after = (w.relationships.get(0, 1).friendship, w.relationships.get(1, 0).friendship);
    assert!(after.0 > before.0 + 1.5 && after.1 > before.1 + 1.5, "{before:?} -> {after:?}");
}

#[test]
fn nobody_answers_in_their_sleep() {
    let mut w = world(CONTENT, "friends");
    let bed = w.objects.iter().position(|o| w.content.objects[o.def].id == "bed").unwrap() as u32;
    w.apply(Command::Use { sim: 1, object: bed, interaction: 0 }).unwrap();
    minutes(&mut w, 30.0);
    assert!(w.sims[1].asleep(&w.content), "Bo is asleep");
    assert_eq!(phone::status(&w, 1), Some(phone::why::ASLEEP));
    let call = action(&w, "call");
    assert_eq!(phone::why_not(&w, 0, 1, call), Some(phone::why::ASLEEP));
    let before = *w.relationships.get(1, 0);
    let social = w.sims[1].needs[0];
    w.apply(Command::Phone { sim: 0, action: call, contact: Some(1) }).unwrap();
    minutes(&mut w, 1.0);
    assert_eq!(on_the_phone(&w, 0).map(|p| p.2), Some(Answer::NoAnswer));
    minutes(&mut w, 5.0);
    assert!(on_the_phone(&w, 0).is_none(), "gave up after a few rings");
    assert!(w.sims[1].asleep(&w.content), "still asleep");
    assert_eq!(w.relationships.get(1, 0).friendship, before.friendship);
    assert!(w.sims[1].needs[0] <= social);
}

#[test]
fn an_invitation_brings_the_friend_over() {
    let mut w = world(CONTENT, "friends");
    let invite = action(&w, "invite");
    assert_eq!(phone::why_not(&w, 0, 1, invite), None);
    w.apply(Command::Phone { sim: 0, action: invite, contact: Some(1) }).unwrap();
    minutes(&mut w, 10.0);
    assert!(w.events.iter().any(|e| e.kind == EventKind::Invited && (e.a, e.b) == (0, 1)), "the invitation is news");
    minutes(&mut w, 60.0);
    let visited = w.events.iter().find(|e| e.kind == EventKind::Visited).expect("Bo came over and was let in");
    assert_eq!((visited.a, visited.b), (1, 0));
    assert_eq!(w.sims[1].visiting.map(|v| v.plot), Some(0));
    assert_eq!(plot_of(&w, 1), Some(0));
    // Together: no phoning each other, they talk.
    assert_eq!(phone::why_not(&w, 0, 1, invite), Some(phone::why::RIGHT_HERE));
    assert_eq!(phone::why_not(&w, 0, 1, action(&w, "text")), Some(phone::why::RIGHT_HERE));
}

#[test]
fn asking_to_come_over_sends_the_resident_to_the_friend() {
    let mut w = world(CONTENT, "friends");
    let ask = action(&w, "askOver");
    w.apply(Command::Phone { sim: 0, action: ask, contact: Some(1) }).unwrap();
    minutes(&mut w, 70.0);
    let visited = w.events.iter().find(|e| e.kind == EventKind::Visited).expect("Ada went over and was let in");
    assert_eq!((visited.a, visited.b), (0, 1));
    assert_eq!(w.sims[0].visiting.map(|v| v.plot), Some(1));
    assert_eq!(plot_of(&w, 0), Some(1));
    assert_eq!(plot_of(&w, 1), Some(1), "Bo stayed home");
}

#[test]
fn people_who_cant_stand_you_say_no() {
    let content = CONTENT.replace(r#""meet":"here","tags":["social","phone"],"acceptance":{"base":1.0}"#, r#""meet":"here","tags":["social","phone"],"acceptance":{"base":0.5,"friendship":0.01}"#);
    let mut w = world(&content, "enemies");
    w.apply(Command::Phone { sim: 0, action: action(&w, "invite"), contact: Some(1) }).unwrap();
    minutes(&mut w, 1.0);
    assert_eq!(on_the_phone(&w, 0).map(|p| p.2), Some(Answer::No));
    minutes(&mut w, 60.0);
    assert!(w.events.iter().any(|e| e.kind == EventKind::Declined && (e.a, e.b) == (0, 1)));
    assert!(w.sims[1].visiting.is_none());
    assert_eq!(plot_of(&w, 1), Some(1));
}

#[test]
fn phone_orders_are_checked() {
    let mut w = world(CONTENT, "friends");
    let call = action(&w, "call");
    let browse = action(&w, "browse");
    assert!(w.apply(Command::Phone { sim: 0, action: call, contact: None }).is_err(), "a call needs someone");
    assert!(w.apply(Command::Phone { sim: 0, action: call, contact: Some(0) }).is_err(), "not themselves");
    assert!(w.apply(Command::Phone { sim: 0, action: 99, contact: Some(1) }).is_err());
    w.apply(Command::Phone { sim: 0, action: browse, contact: None }).unwrap();
    minutes(&mut w, 1.0);
    assert_eq!(on_the_phone(&w, 0).map(|p| p.0), Some(browse));
    // Someone who isn't at home can't be visited.
    assert_eq!(phone::why_not(&w, 0, 1, action(&w, "askOver")), None);
    w.sims[1].pos = [10.5, 12.5];
    assert_eq!(phone::why_not(&w, 0, 1, action(&w, "askOver")), Some(phone::why::NOT_HOME));
    // The JSON the UI sends.
    let cmd = sim_core::Command::from_json(&format!(r#"{{"type":"phone","sim":0,"action":{call},"contact":1}}"#)).unwrap();
    assert!(matches!(cmd, Command::Phone { sim: 0, contact: Some(1), .. }));
}

#[test]
fn on_a_sofa_nearby_if_there_is_one() {
    let content = CONTENT.replace(r#""seatRange":0"#, r#""seatRange":6"#);
    let mut w = world(&content, "friends");
    w.apply(Command::Phone { sim: 0, action: action(&w, "call"), contact: Some(1) }).unwrap();
    minutes(&mut w, 3.0);
    let sofa = w.objects.iter().position(|o| w.content.objects[o.def].id == "sofa").unwrap() as u32;
    let act = w.sims[0].current().expect("on the phone");
    assert!(matches!(act.task.kind, TaskKind::Phone { seat: Some(s), .. } if s == sofa));
    assert!(matches!(act.phase, Phase::Using { .. }));
    assert_eq!(w.sims[0].pose, sim_core::content::Pose::Sit);
    // The renderer shows the phone animation.
    let mut out = vec![0.0; sim_core::snapshot::CAPACITY];
    sim_core::snapshot::write(&w, &mut out);
    let row = &out[sim_core::snapshot::HEADER_LEN..];
    let anim = w.content.animations.iter().position(|a| a == "phone").unwrap() as f32;
    assert_eq!(row[sim_core::snapshot::sim::ACTION], anim);
    assert_eq!(row[sim_core::snapshot::sim::OBJECT], sofa as f32);
    // Saved mid-call: it's simply over after loading, and the sofa is free.
    let mut loaded = World::from_save_json(&content, &w.save_json()).unwrap();
    loaded.autonomy = false;
    common::check(&loaded);
    minutes(&mut w, 30.0);
    assert_eq!(w.sims[0].pose, sim_core::content::Pose::Stand, "up again after the call");
    assert!(w.objects[sofa as usize].users().next().is_none());
}

#[test]
fn a_lonely_resident_calls_a_friend_from_home() {
    let mut w = World::from_json(CONTENT, &town("friends"), 3).unwrap();
    while w.tick < clock::tick_at(1, 11.0 * 60.0).unwrap() {
        w.tick_once();
    }
    // Nothing else to do: Bo keeps to himself, Ada's lonely.
    w.apply(Command::SetAutonomy { enabled: false, household: Some(1) }).unwrap();
    w.sims[0].needs[0] = 0.15;
    let mut phoned = None;
    for _ in 0..(120.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        if let Some((a, Some(1), _)) = on_the_phone(&w, 0) {
            phoned = Some((a, plot_of(&w, 0)));
            break;
        }
    }
    let (a, at) = phoned.expect("Ada never got her phone out for Bo");
    assert_eq!(at, Some(0), "from home");
    assert!(w.content.phone.actions[a].contact);
}

#[test]
fn bored_with_nothing_to_do_they_scroll() {
    let mut w = World::from_json(CONTENT, &town("friends"), 3).unwrap();
    while w.tick < clock::tick_at(1, 11.0 * 60.0).unwrap() {
        w.tick_once();
    }
    w.apply(Command::SetAutonomy { enabled: false, household: Some(1) }).unwrap();
    w.sims[0].needs = [1.0, 0.2, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let browse = action(&w, "browse");
    let mut scrolled = false;
    for _ in 0..(60.0 / sim_core::MINUTES_PER_TICK) as u32 {
        w.tick_once();
        scrolled |= on_the_phone(&w, 0).is_some_and(|p| p.0 == browse);
    }
    assert!(scrolled, "Ada never scrolled");
    assert!(w.sims[0].needs[1] > 0.25, "and had some fun: {}", w.sims[0].needs[1]);
}

#[test]
fn the_phone_box_is_gone_from_the_market_square() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/public/content");
    let houses: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("houses.json")).unwrap()).unwrap();
    for place in houses["parks"].as_array().unwrap() {
        let objects = place["objects"].as_array().unwrap();
        assert!(!objects.iter().any(|o| o["def"] == "phoneBooth"), "a phone box in {}", place["id"]);
    }
    // Still in the catalog (old towns keep theirs, and it loads), but nobody walks to it on
    // their own any more: everyone has a phone.
    let content = common::content();
    let w = World::from_json(&content, &common::town(&content, 7, 2), 7).unwrap();
    let booth = &w.content.objects[w.content.object_index("phoneBooth").unwrap()];
    assert!(booth.interactions.iter().all(|i| !i.autonomous));
}

#[test]
fn a_town_phones_each_other() {
    let content = common::content();
    let mut w = World::from_json(&content, &common::town(&content, 7, 2), 7).unwrap();
    let (mut calls, mut at_home) = (0, 0);
    let mut seen = std::collections::HashSet::new();
    let end = clock::tick_at(3, 0.0).unwrap();
    while w.tick < end {
        w.tick_once();
        for s in &w.sims {
            let Some(act) = s.current() else { continue };
            let TaskKind::Phone { contact, answer, .. } = act.task.kind else { continue };
            if !seen.insert((s.id, w.tick / 60)) || !matches!(act.phase, Phase::Using { .. }) {
                continue;
            }
            match contact {
                Some(_) if answer != Answer::Pending => {
                    calls += 1;
                    let home = w.households[s.household as usize].plot;
                    at_home += usize::from(home.is_some() && w.plot_at(s.tile().0, s.tile().1) == home);
                }
                _ => {}
            }
        }
    }
    eprintln!("on the phone with someone: {calls} ({at_home} at home)");
    assert!(calls > 10, "residents keep in touch by phone");
    assert!(at_home * 2 > calls, "mostly from home");
    common::check(&w);
}

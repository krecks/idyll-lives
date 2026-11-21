//! The post at home (`post.rs`): a letterbox's post comes every morning and is fetched and read
//! (content `post`); a letter written to a friend (content `letter`) is in their letterbox the
//! next morning and the story says who wrote; a hose waters every growing crop on its plot at
//! once (content `waters`); a place may say its own pose (the one giving a massage stands). And
//! with the shipped content, a household with free will fetches its post by itself.

use crate::common;

use sim_core::social::EventKind;
use sim_core::world::{GrowthStage, Phase, TaskKind};
use sim_core::{Command, TICKS_PER_MINUTE, World, clock};

const CONTENT: &str = r#"{
    "needs":[{"id":"fun","label":"Fun","decayPerHour":0.01},
             {"id":"social","label":"Social","decayPerHour":0.01},
             {"id":"comfort","label":"Comfort","decayPerHour":0.01}],
    "objects":[
      {"id":"letterbox","name":"Letterbox","price":95,"outdoors":true,"interactions":[
        {"id":"fetchPost","label":"Fetch the post","minutes":8,"when":"open","leaves":"closed","post":true,
         "effects":{"fun":0.1}}]},
      {"id":"desk","name":"Desk","price":100,"interactions":[
        {"id":"letter","label":"Write a letter to a friend","minutes":30,"letter":true,"effects":{"fun":0.1}}]},
      {"id":"patch","name":"Vegetable patch","price":90,"outdoors":true,"grows":{"days":2},"interactions":[
        {"id":"tend","label":"Weed and water","minutes":30,"tends":0.15,"effects":{"fun":0.1}}]},
      {"id":"hose","name":"Hose reel","price":85,"outdoors":true,"interactions":[
        {"id":"waterGarden","label":"Water the garden","minutes":15,"waters":0.2,"effects":{"fun":0.08}}]},
      {"id":"massageTable","name":"Massage table","footprint":[2,2],"slots":2,
       "places":[{"at":[-0.5,0]},{"at":[0.15,-0.25],"facing":-90,"pose":"stand"}],
       "interactions":[{"id":"massage","label":"Massage","minutes":40,"pose":"lie","partner":true,
                        "effects":{"comfort":0.6,"social":0.2}}]}],
    "bondPresets":{"friends":{"friendship":70}},
    "economy":{"startingFunds":1000}}"#;

const LETTERBOX: u32 = 0;
const DESK: u32 = 1;
const PATCH: u32 = 2;
const HOSE: u32 = 3;
const MASSAGE: u32 = 4;
const BOX_2: u32 = 5;
const PATCH_2: u32 = 6;

// Ada and Bo at home on plot 0; Cy, their friend, next door on plot 1.
const ADA: u32 = 0;
const BO: u32 = 1;
const CY: u32 = 2;

const TOWN: &str = r#"{"width":40,"depth":16,
    "plots":[{"name":"Home","x":0,"z":0,"w":20,"d":16,"entry":[10.5,15.5]},
             {"name":"Next door","x":20,"z":0,"w":20,"d":16,"entry":[30.5,15.5]}],
    "households":[{"name":"Player","plot":0,"player":true},{"name":"Neighbours","plot":1}],
    "objects":[{"def":"letterbox","x":2,"z":12},{"def":"desk","x":6,"z":2},{"def":"patch","x":12,"z":10},
               {"def":"hose","x":15,"z":12},{"def":"massageTable","x":2,"z":2},
               {"def":"letterbox","x":22,"z":12},{"def":"patch","x":32,"z":10}],
    "sims":[{"name":"Ada","x":8.5,"z":8.5,"age":35},{"name":"Bo","x":9.5,"z":8.5,"age":35},
            {"name":"Cy","household":1,"x":28.5,"z":8.5,"age":35}],
    "relationships":[{"a":0,"b":2,"preset":"friends"},{"a":2,"b":0,"preset":"friends"}]}"#;

fn world() -> World {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.autonomy = false;
    w
}

fn order(w: &mut World, sim: u32, object: u32, interaction: usize) -> Result<(), String> {
    w.apply(Command::Use {
        sim,
        object,
        interaction,
    })
    .map_err(|e| e.to_string())
}

/// Ticks `w` for up to `minutes` until `done`; whether it got there.
fn until(w: &mut World, minutes: u64, mut done: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..minutes * TICKS_PER_MINUTE {
        if done(w) {
            return true;
        }
        w.tick_once();
    }
    done(w)
}

fn idle(w: &World, sim: u32) -> bool {
    w.sims[sim as usize].current().is_none() && w.sims[sim as usize].queue().next().is_none()
}

fn need(w: &World, id: &str) -> usize {
    w.content.needs.iter().position(|n| n.id == id).unwrap()
}

#[test]
fn the_post_comes_each_morning_and_is_fetched() {
    let mut w = world();
    assert!(
        !w.objects[LETTERBOX as usize].open,
        "empty when the game starts"
    );
    let err = order(&mut w, ADA, LETTERBOX, 0).unwrap_err();
    assert!(err.contains("closed"), "nothing to fetch yet: {err}");

    // It comes at seven the next morning, to both homes' letterboxes.
    assert!(until(&mut w, 24 * 60, |w| w.objects[LETTERBOX as usize].open));
    let minute = clock::minute_of_day(w.tick);
    assert!(
        (420.0..421.0).contains(&minute),
        "the post came at {minute}"
    );
    assert!(w.objects[BOX_2 as usize].open, "next door's too");

    let version = w.structure_version();
    order(&mut w, ADA, LETTERBOX, 0).unwrap();
    assert!(until(&mut w, 30, |w| !w.objects[LETTERBOX as usize].open));
    assert_ne!(
        w.structure_version(),
        version,
        "the post is out of its slot"
    );
    assert!(until(&mut w, 30, |w| idle(w, ADA)), "fetched and read");
    assert!(
        order(&mut w, BO, LETTERBOX, 0).is_err(),
        "nothing more to fetch till tomorrow"
    );
    // The open letterbox is saved as any open thing is.
    assert!(until(&mut w, 24 * 60, |w| w.objects[LETTERBOX as usize].open));
    let reloaded = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    assert!(reloaded.objects[LETTERBOX as usize].open);
}

#[test]
fn a_letter_to_a_friend_is_in_their_letterbox_next_morning() {
    let mut w = world();
    // Ada writes to Cy, her friend; Bo, nobody's friend, writes to nobody.
    order(&mut w, BO, DESK, 0).unwrap();
    assert!(until(&mut w, 60, |w| idle(w, BO)));
    assert!(
        w.households.iter().all(|h| h.letters.is_empty()),
        "Bo has no friend"
    );
    order(&mut w, ADA, DESK, 0).unwrap();
    assert!(until(&mut w, 60, |w| idle(w, ADA)));
    assert_eq!(w.households[1].letters, vec![ADA], "to Cy's household");
    // Saved on the way.
    let mut w = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    w.autonomy = false;
    assert_eq!(w.households[1].letters, vec![ADA]);

    // Next morning Cy fetches the post: Ada's letter, a little social, and the story says so.
    assert!(until(&mut w, 24 * 60, |w| w.objects[BOX_2 as usize].open));
    let social = need(&w, "social");
    w.sims[CY as usize].needs[social] = 0.2;
    let friendship = w.relationships.get(CY as usize, ADA as usize).friendship;
    order(&mut w, CY, BOX_2, 0).unwrap();
    assert!(until(&mut w, 30, |w| idle(w, CY)));
    assert!(w.households[1].letters.is_empty(), "read");
    assert!(w.sims[CY as usize].needs[social] > 0.3, "a lift");
    assert!(w.relationships.get(CY as usize, ADA as usize).friendship > friendship);
    let got = w
        .events
        .iter()
        .find(|e| e.kind == EventKind::GotLetter)
        .expect("the story tells of it");
    assert_eq!((got.a, got.b), (CY, ADA));
}

#[test]
fn a_hose_waters_every_growing_crop_on_its_plot() {
    let mut w = world();
    // Bought crops are just planted (the lot's come ripe): plant both.
    for p in [PATCH, PATCH_2] {
        w.objects[p as usize].growth = 0.2;
    }
    order(&mut w, ADA, HOSE, 0).unwrap();
    assert!(until(&mut w, 30, |w| idle(w, ADA)));
    let home = w.objects[PATCH as usize].growth;
    let next_door = w.objects[PATCH_2 as usize].growth;
    assert!(home > 0.39, "watered: {home}");
    assert!(next_door < 0.22, "next door's isn't: {next_door}");
    assert_eq!(
        w.objects[PATCH as usize].growth_stage(),
        GrowthStage::Growing
    );
}

#[test]
fn the_one_giving_a_massage_stands_at_their_place() {
    let mut w = world();
    order(&mut w, ADA, MASSAGE, 0).unwrap();
    let using = |w: &World, s: u32| {
        w.sims[s as usize].current().is_some_and(|a| {
            matches!(a.task.kind, TaskKind::Use { object, .. } if object == MASSAGE)
                && matches!(a.phase, Phase::Using { elapsed } if elapsed > 0.0)
        })
    };
    assert!(
        until(&mut w, 30, |w| using(w, ADA) && using(w, BO)),
        "Bo fetched"
    );
    let table = &w.objects[MASSAGE as usize];
    let poses: Vec<_> = [ADA, BO]
        .iter()
        .map(|&s| {
            let sim = &w.sims[s as usize];
            (
                table.users.iter().position(|u| *u == Some(sim.id)),
                sim.pose,
            )
        })
        .collect();
    for (slot, pose) in poses {
        let want = if slot == Some(1) {
            sim_core::content::Pose::Stand
        } else {
            sim_core::content::Pose::Lie
        };
        assert_eq!(pose, want, "slot {slot:?}");
    }
}

/// With the shipped content and free will, the household fetches its post by itself in the
/// morning, and uses the new garden tools and wellness things now and then.
#[test]
fn a_household_fetches_its_post_by_itself() {
    let (mut w, _) = common::garden_home();
    w.households[0].free_will = true;
    common::buy(&mut w, "letterbox", 4, 1);
    let letterbox = w.objects.len() - 1;
    let mut fetched = 0;
    let mut was_open = false;
    for _ in 0..2 * common::TICKS_PER_DAY {
        w.tick_once();
        let open = w.objects[letterbox].open;
        if was_open && !open {
            fetched += 1;
            let minute = clock::minute_of_day(w.tick);
            assert!(
                (420.0..14.0 * 60.0).contains(&minute),
                "fetched in the morning, not at {minute}"
            );
        }
        was_open = open;
    }
    assert_eq!(fetched, 2, "the post fetched each morning");
}

/// A lawnmower bought new mows at once (`grows.boughtRipe`); after a mow the grass grows back
/// over its days.
#[test]
fn a_new_lawnmower_mows_at_once_then_waits_for_the_grass() {
    let (mut w, _) = common::garden_home();
    common::buy(&mut w, "lawnMower", 6, 16);
    let mower = w.objects.len() as u32 - 1;
    assert!(w.objects[mower as usize].ripe(), "the lawn is there to mow");
    order(&mut w, 0, mower, 0).unwrap();
    assert!(until(&mut w, 60, |w| idle(w, 0)), "mowed");
    assert!(!w.objects[mower as usize].ripe(), "just mown");
    let err = order(&mut w, 0, mower, 0).unwrap_err();
    assert!(err.contains("the grass hasn't grown back yet"), "{err}");
}

//! Who may use what at their age (content `stages`): the kids' things are a child's, wine a
//! grown-up's, homework a pupil's, a baby's care a grown-up's, the upper bunk not an elder's,
//! and nobody starts a fight with a child. Free will and the player's orders keep the rules
//! alike, and a refused order never starts.

mod common;

use sim_core::content::Content;
use sim_core::path::NavGrid;
use sim_core::surface::{Links, dining_seat};
use sim_core::world::TaskKind;
use sim_core::{Command, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"fun","label":"Fun","decayPerHour":0.5},
             {"id":"energy","label":"Energy","decayPerHour":0.02},
             {"id":"social","label":"Social","decayPerHour":0.5}],
    "objects":[
      {"id":"rider","name":"Spring rider","stages":["child"],"interactions":[
        {"id":"ride","label":"Bounce","minutes":10,"effects":{"fun":0.5}}]},
      {"id":"wine","name":"Wine fridge","interactions":[
        {"id":"pour","label":"Pour a glass of wine","minutes":10,"effects":{"fun":0.5},"stages":["adult","elder"]}]},
      {"id":"crib","name":"Crib","slots":2,"interactions":[
        {"id":"lie","label":"Lie in the crib","minutes":600,"pose":"lie","baby":true,"effects":{"energy":0.5}},
        {"id":"feed","label":"Feed the baby","minutes":15,"care":{"fun":0.4}}]},
      {"id":"desk","name":"Desk","interactions":[
        {"id":"homework","label":"Do homework","minutes":45,"homework":6,"effects":{"fun":-0.05}}]},
      {"id":"bunk","name":"Bunk bed","footprint":[1,2],"slots":2,"bunk":true,"upperStages":["child","teen","adult"],
       "interactions":[{"id":"sleep","label":"Sleep","minutes":480,"pose":"lie","autonomous":false,"effects":{"energy":1.0}}]}],
    "socials":[{"id":"chat","label":"Chat","minutes":10,"acceptance":{"base":1.0},"needs":{"social":0.2}},
               {"id":"fight","label":"Start a fight","minutes":8,"prefer":"disliked","autonomyWeight":5,
                "requires":{"maxFriendship":-35},"acceptance":{"base":1.0},"needs":{"social":0.4},
                "stages":["teen","adult","elder"],"targetStages":["teen","adult","elder"]}],
    "socialRules":{"defaultBond":"roommates"},
    "bondPresets":{"roommates":{"friendship":20}},
    "life":{"daysPerYear":2,"stages":[
        {"id":"baby","label":"Baby","from":0,"baby":true},
        {"id":"child","label":"Child","from":2,"school":true},
        {"id":"teen","label":"Teen","from":13,"school":true},
        {"id":"adult","label":"Adult","from":18},
        {"id":"elder","label":"Elder","from":60}]},
    "economy":{"startingFunds":1000}}"#;

// Ada (35), Dee (8), Cy (15), Bo the baby, Gran (70).
const TOWN: &str = r#"{"width":20,"depth":12,
    "plots":[{"name":"Home","x":0,"z":0,"w":20,"d":12,"entry":[10.5,11.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "objects":[{"def":"rider","x":2,"z":2},{"def":"wine","x":5,"z":2},{"def":"crib","x":8,"z":2},
               {"def":"crib","x":11,"z":2},{"def":"desk","x":14,"z":2},{"def":"bunk","x":17,"z":2}],
    "sims":[{"name":"Ada","x":4.5,"z":6.5,"age":35},{"name":"Dee","x":6.5,"z":6.5,"age":8},
            {"name":"Cy","x":8.5,"z":6.5,"age":15},{"name":"Bo","x":9.5,"z":3.5,"age":0.5},
            {"name":"Gran","x":12.5,"z":6.5,"age":70}]}"#;

const ADA: u32 = 0;
const DEE: u32 = 1;
const CY: u32 = 2;
const BO: u32 = 3;
const GRAN: u32 = 4;

fn world() -> World {
    World::from_json(CONTENT, TOWN, 1).unwrap()
}

/// The `n`th object of kind `def`.
fn object(w: &World, def: &str, n: usize) -> u32 {
    w.objects.iter().filter(|o| w.content.objects[o.def].id == def).nth(n).unwrap().id
}

fn order(w: &mut World, sim: u32, def: &str, interaction: usize) -> Result<(), String> {
    let object = object(w, def, 0);
    w.apply(Command::Use { sim, object, interaction }).map(|_| ()).map_err(|e| e.to_string())
}

fn refused(w: &mut World, sim: u32, def: &str, interaction: usize, why: &str) {
    let err = order(w, sim, def, interaction).expect_err(def);
    assert!(err.contains(why), "{def}: {err}");
}

/// What everyone is using now: `(sim, object kind, interaction)`.
fn uses(w: &World) -> Vec<(u32, String, usize)> {
    w.sims
        .iter()
        .filter_map(|s| match s.current()?.task.kind {
            TaskKind::Use { object, interaction } => Some((s.id, w.content.objects[w.objects[object as usize].def].id.clone(), interaction)),
            _ => None,
        })
        .collect()
}

#[test]
fn an_order_out_of_their_age_is_refused_and_never_starts() {
    let mut w = world();
    refused(&mut w, DEE, "wine", 0, "Dee is too young to pour a glass of wine");
    refused(&mut w, CY, "wine", 0, "too young");
    refused(&mut w, ADA, "rider", 0, "Ada is too big for the spring rider");
    refused(&mut w, GRAN, "rider", 0, "too big");
    refused(&mut w, ADA, "crib", 0, "too big for the crib");
    refused(&mut w, DEE, "crib", 0, "too big for the crib");
    refused(&mut w, DEE, "crib", 1, "Dee is too young to look after a baby");
    refused(&mut w, CY, "crib", 1, "too young to look after a baby");
    refused(&mut w, ADA, "desk", 0, "Ada has no homework to do");
    refused(&mut w, BO, "rider", 0, "too young");
    // Caring for a baby needs one there: the second crib is empty.
    let empty = object(&w, "crib", 1);
    let err = w.apply(Command::Use { sim: ADA, object: empty, interaction: 1 }).unwrap_err();
    assert!(err.to_string().contains("no baby in the crib"), "{err}");
    assert!(w.sims.iter().all(|s| s.queue().next().is_none()), "nothing refused was queued");
    for _ in 0..600 {
        w.tick_once();
        for (sim, def, i) in uses(&w) {
            assert!(!matches!((sim, def.as_str(), i), (DEE | CY, "wine", _) | (ADA | GRAN, "rider", _) | (ADA, "desk", _)), "{sim} at the {def}");
        }
    }

    // What their age is for, they do.
    let mut w = world();
    order(&mut w, DEE, "rider", 0).unwrap();
    order(&mut w, ADA, "wine", 0).unwrap();
    order(&mut w, CY, "desk", 0).unwrap();
    order(&mut w, GRAN, "wine", 0).unwrap();
}

#[test]
fn free_will_never_picks_what_their_age_is_not_for() {
    let mut w = world();
    w.autonomy = true;
    let mut seen = Vec::new();
    for _ in 0..6000 {
        for s in w.sims.iter_mut() {
            s.needs[0] = s.needs[0].min(0.2);
        }
        w.tick_once();
        for (sim, def, i) in uses(&w) {
            let bad = match def.as_str() {
                "wine" => sim == DEE || sim == CY,
                "rider" => sim != DEE,
                "desk" => sim != DEE && sim != CY,
                "crib" => (i == 0) != (sim == BO) || (i == 1 && (sim == DEE || sim == CY)),
                _ => false,
            };
            assert!(!bad, "{} on their own at the {def} ({i})", w.sims[sim as usize].name);
            seen.push((sim, def));
        }
    }
    // (Not for want of trying: the grown-ups pour wine, the child rides.)
    assert!(seen.iter().any(|(s, d)| *s == DEE && d == "rider"), "Dee rode the spring rider");
    assert!(seen.iter().any(|(s, d)| (*s == ADA || *s == GRAN) && d == "wine"), "a grown-up poured some wine");
}

#[test]
fn nobody_starts_a_fight_with_a_child() {
    let mut w = world();
    for a in 0..5 {
        for b in 0..5 {
            if a != b {
                w.relationships.get_mut(a, b).friendship = -60.0;
            }
        }
    }
    let fight = 1;
    let offered = |w: &World, a: usize, b: usize| sim_core::conversation::options(w, a, b).iter().any(|&(s, _)| s == fight);
    assert!(!offered(&w, ADA as usize, DEE as usize) && !offered(&w, CY as usize, DEE as usize), "no fight with the child on offer");
    assert!(!offered(&w, DEE as usize, ADA as usize), "nor the child starting one");
    assert!(offered(&w, ADA as usize, CY as usize), "a grown-up and a teen who can't stand each other");
    for (a, b) in [(ADA, DEE), (CY, DEE), (GRAN, DEE), (DEE, CY)] {
        assert!(w.apply(Command::Social { sim: a, target: b, social: fight }).is_err(), "{a} and {b}");
    }
    w.apply(Command::Social { sim: ADA, target: CY, social: fight }).unwrap();

    // Nor on their own.
    let mut w = world();
    w.autonomy = true;
    for a in 0..5 {
        for b in 0..5 {
            if a != b {
                w.relationships.get_mut(a, b).friendship = -60.0;
            }
        }
    }
    for _ in 0..6000 {
        for s in w.sims.iter_mut() {
            s.needs[2] = s.needs[2].min(0.2);
        }
        w.tick_once();
        for s in &w.sims {
            if let Some(TaskKind::Social { target, social }) = s.current().map(|a| a.task.kind) {
                assert!(social != fight || (s.id != DEE && target != DEE && target != BO), "{} started a fight with {target}", s.name);
            }
        }
    }
}

#[test]
fn an_elder_doesnt_climb_to_the_upper_bunk() {
    let mut w = world();
    let bunk = object(&w, "bunk", 0);
    order(&mut w, DEE, "bunk", 0).unwrap();
    for _ in 0..2000 {
        if w.objects[bunk as usize].users().any(|u| u == DEE) {
            break;
        }
        w.tick_once();
    }
    assert!(w.objects[bunk as usize].users().any(|u| u == DEE), "Dee took the lower bunk");
    refused(&mut w, GRAN, "bunk", 0, "Gran is too old for the top bunk");
    order(&mut w, ADA, "bunk", 0).unwrap();
}

const DINING: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.02}],
    "tags":["food","lounge"],
    "objects":[
      {"id":"table","name":"Table","footprint":[2,1],"surface":{"height":0.75,"kind":"table"}},
      {"id":"kidsTable","name":"Kids' table","surface":{"height":0.5,"kind":"low"}},
      {"id":"chair","name":"Chair","approach":"around","seatAt":["table"],"interactions":[
        {"id":"sit","label":"Sit","minutes":20,"pose":"sit","tags":["lounge"]}]},
      {"id":"kidsChair","name":"Kids' chair","approach":"around","seatAt":["low"],"stages":["child"],"interactions":[
        {"id":"sit","label":"Sit","minutes":20,"pose":"sit","tags":["lounge"]}]}],
    "life":{"stages":[{"id":"baby","from":0,"baby":true},{"id":"child","from":2,"school":true},{"id":"adult","from":18}]}}"#;

#[test]
fn a_child_has_a_snack_at_the_kids_table_and_a_grown_up_never_does() {
    let lot = r#"{"width":16,"depth":16,"objects":[{"def":"table","x":2,"z":2},{"def":"chair","x":2,"z":1},
        {"def":"kidsTable","x":10,"z":10},{"def":"kidsChair","x":10,"z":9}]}"#;
    let w = World::from_json(DINING, lot, 1).unwrap();
    let (chair, kids_chair) = (object(&w, "chair", 0), object(&w, "kidsChair", 0));
    let nav = NavGrid { lot: &w.lot, blocked: w.blocked(), stairs: w.stair_map() };
    let links = Links::new(&w.content, &w.objects, &nav);
    let all: Vec<u32> = w.objects.iter().map(|o| o.id).collect();
    // From beside the dining table, the child still goes to the kids' table.
    let from = [3.5, 0.5];
    let (child, adult) = (1, 2);
    assert_eq!(dining_seat(&w.content, &w.objects, &links, &all, from, child).map(|d| d.object), Some(kids_chair));
    assert_eq!(dining_seat(&w.content, &w.objects, &links, &all, [10.5, 8.5], adult).map(|d| d.object), Some(chair));
    let kids_only = [object(&w, "kidsTable", 0), kids_chair];
    assert!(dining_seat(&w.content, &w.objects, &links, &kids_only, from, adult).is_none(), "the kids' chair is no seat for a grown-up");
}

#[test]
fn shipped_content_keeps_the_kids_things_the_alcohol_and_the_stove_to_their_ages() {
    let c = Content::from_json(&common::content()).unwrap();
    let stage = |id: &str| c.life.stages.iter().position(|s| s.id == id).unwrap();
    let (baby, child, teen, young, adult, elder) = (stage("baby"), stage("child"), stage("teen"), stage("youngAdult"), stage("adult"), stage("elder"));
    let use_of = |obj: &str, it: &str| {
        let def = &c.objects[c.object_index(obj).unwrap_or_else(|| panic!("{obj}"))];
        def.interactions.iter().find(|i| i.id == it).unwrap_or_else(|| panic!("{obj}.{it}"))
    };
    let only = |obj: &str, it: &str, stages: &[usize]| {
        let i = use_of(obj, it);
        for s in [baby, child, teen, young, adult, elder] {
            assert_eq!(i.for_stage(s), stages.contains(&s), "{obj}.{it} for {}", c.life.stages[s].id);
        }
    };
    let grown = [young, adult, elder];
    let cooks = [teen, young, adult, elder];
    let everyone = [child, teen, young, adult, elder];
    // The kids' furniture and the playground.
    for (obj, it) in [("kidsBedCar", "sleep"), ("rockingHorse", "ride"), ("kidsChair", "sit"), ("kidsChair", "homework"), ("kidsDesk", "homework"),
        ("beanBagKids", "flop"), ("ballPit", "jump"), ("playTent", "play"), ("toyBox", "play"), ("toyChest", "play"),
        ("springRider", "ride"), ("slide", "slide"), ("climbingFrame", "climb")]
    {
        only(obj, it, &[child]);
    }
    only("seesaw", "seesaw", &[child, teen]);
    only("merryGoRound", "spin", &[child, teen]);
    only("crib", "lie", &[baby]);
    // Alcohol, the stove (and grill and oven) and the standing desk.
    only("wineFridge", "wine", &grown);
    only("barCart", "mix", &grown);
    only("outgoing.partyBar", "mixDrinks", &grown);
    only("outgoing.partyBar", "soda", &everyone);
    for (obj, it) in [("fridge", "meal"), ("stoveGas", "gourmet"), ("stoveElectric", "bake"), ("grill", "cook"), ("wallOven", "roast"), ("foodie.pizzaOven", "bakePizza")] {
        only(obj, it, &cooks);
    }
    only("counter", "sandwich", &everyone);
    only("deskStanding", "browse", &cooks);
    // The loaded bar, the adult dumbbells, the ice bath and the sauna: teens and up.
    for (obj, it) in [("weightBench", "lift"), ("dumbbells", "lift"), ("hotHeaded.coldPlunge", "plunge"), ("loner.soloSauna", "sweat")] {
        only(obj, it, &cooks);
    }
    // A pupil's homework is anyone's to take by stage; the pupils-only rule is the code's.
    only("bookshelf", "homework", &everyone);
    // The upper bunk: not an elder's.
    let bunk = &c.objects[c.object_index("bedBunk").unwrap()];
    assert!(sim_core::content::stage_in(bunk.upper_stages, child) && !sim_core::content::stage_in(bunk.upper_stages, elder));
    // The climbing frame is a toy, as the other rides are (teens and up hardly choose them).
    let toys = c.tags.iter().position(|t| t == "toys").unwrap();
    assert_ne!(use_of("climbingFrame", "climb").tags & (1 << toys), 0);
    // A fight: never with a child.
    let fight = c.socials.iter().find(|s| s.id == "fight").unwrap();
    assert!(!sim_core::content::stage_in(fight.target_stages, child) && sim_core::content::stage_in(fight.target_stages, teen));
}

#[test]
fn a_stage_rule_names_life_stages_content_has() {
    let with = |object: &str| CONTENT.replacen(r#"{"id":"rider","name":"Spring rider","stages":["child"],"#, object, 1);
    let err = |c: String| Content::from_json(&c).unwrap_err().to_string();
    assert!(err(with(r#"{"id":"rider","name":"Spring rider","stages":["toddler"],"#)).contains("unknown life stage 'toddler'"));
    assert!(err(with(r#"{"id":"rider","name":"Spring rider","stages":[],"#)).contains("at least one"));
    assert!(err(with(r#"{"id":"rider","name":"Spring rider","upperStages":["child"],"#)).contains("bunk bed"));
    let baby = CONTENT.replacen(r#""baby":true,"effects""#, r#""baby":true,"stages":["child"],"effects""#, 1);
    assert!(err(baby).contains("a baby's only"));
}

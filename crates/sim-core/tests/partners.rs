//! Games for two (content `partner`, G-65): whoever starts one fetches someone free to play,
//! who goes to the other place; it starts once both are there, gives both what it gives, and is
//! over for both when either stops. With nobody free the player's order does the game's use for
//! one (`alone`), or is refused; free will doesn't choose it. Only someone the game's age is for
//! is fetched, and someone busy isn't.

use sim_core::snapshot::{self, HEADER_LEN, SIM_STRIDE, sim};
use sim_core::social::EventKind;
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, TICKS_PER_MINUTE, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"fun","label":"Fun","decayPerHour":0.05},
             {"id":"social","label":"Social","decayPerHour":0.01},
             {"id":"energy","label":"Energy","decayPerHour":0.01}],
    "objects":[
      {"id":"seesaw","name":"Seesaw","footprint":[3,1],"slots":2,"stages":["child","teen"],
       "places":[{"at":[-0.78,0],"facing":90},{"at":[0.78,0],"facing":-90}],
       "interactions":[{"id":"seesaw","label":"Ride the seesaw","minutes":15,"pose":"sit","partner":true,
                        "effects":{"fun":0.4,"social":0.1}}]},
      {"id":"pool","name":"Pool table","footprint":[3,2],"slots":2,
       "places":[{"at":[-0.55,1.5],"facing":180},{"at":[0.55,-1.5]}],
       "interactions":[{"id":"pool","label":"Shoot pool","minutes":30,"partner":true,"alone":"tricks",
                        "effects":{"fun":0.5,"social":0.08}},
                       {"id":"tricks","label":"Practise trick shots","minutes":30,"effects":{"fun":0.15}}]},
      {"id":"pingPong","name":"Ping-pong table","footprint":[3,2],"slots":2,
       "places":[{"at":[-1.4,0],"facing":90},{"at":[1.4,0],"facing":-90}],
       "interactions":[{"id":"pingPong","label":"Play ping-pong","minutes":30,"partner":true,
                        "effects":{"fun":0.45,"social":0.12}}]},
      {"id":"sofa","name":"Sofa","footprint":[2,1],"slots":2,
       "interactions":[{"id":"sit","label":"Sit","minutes":240,"pose":"sit","autonomous":false,"effects":{"energy":0.1}}]}],
    "life":{"daysPerYear":2,"stages":[
        {"id":"baby","label":"Baby","from":0,"baby":true},
        {"id":"child","label":"Child","from":2},
        {"id":"teen","label":"Teen","from":13},
        {"id":"adult","label":"Adult","from":18},
        {"id":"elder","label":"Elder","from":60}]},
    "economy":{"startingFunds":1000}}"#;

const SEESAW: u32 = 0;
const POOL: u32 = 1;
const PING_PONG: u32 = 2;
const SOFA: u32 = 3;

// Ada (35) and Bo (35), Dee (8) and Cy (15); Bo the nearest to the pool table.
const ADA: u32 = 0;
const BO: u32 = 1;
const DEE: u32 = 2;
const CY: u32 = 3;

fn town(sims: &str) -> String {
    format!(
        r#"{{"width":24,"depth":16,
        "plots":[{{"name":"Home","x":0,"z":0,"w":24,"d":16,"entry":[12.5,15.5]}}],
        "households":[{{"name":"Player","plot":0,"player":true}}],
        "objects":[{{"def":"seesaw","x":1,"z":2}},{{"def":"pool","x":8,"z":2}},{{"def":"pingPong","x":15,"z":2}},
                   {{"def":"sofa","x":2,"z":12}}],
        "sims":[{sims}]}}"#
    )
}

const EVERYONE: &str = r#"{"name":"Ada","x":4.5,"z":9.5,"age":35},{"name":"Bo","x":10.5,"z":7.5,"age":35},
                          {"name":"Dee","x":6.5,"z":13.5,"age":8},{"name":"Cy","x":20.5,"z":13.5,"age":15}"#;

fn world(sims: &str) -> World {
    let mut w = World::from_json(CONTENT, &town(sims), 1).unwrap();
    w.autonomy = false;
    w
}

fn order(w: &mut World, sim: u32, object: u32, interaction: usize) -> Result<(), String> {
    w.apply(Command::Use { sim, object, interaction }).map_err(|e| e.to_string())
}

/// What `sim` is doing: `(object, interaction, elapsed)`, elapsed None while on their way.
fn doing(w: &World, sim: u32) -> Option<(u32, usize, Option<f32>)> {
    let a = w.sims[sim as usize].current()?;
    let TaskKind::Use { object, interaction } = a.task.kind else { return None };
    Some((object, interaction, if let Phase::Using { elapsed } = a.phase { Some(elapsed) } else { None }))
}

fn playing(w: &World, sim: u32, object: u32, interaction: usize) -> bool {
    doing(w, sim).is_some_and(|(o, i, e)| (o, i) == (object, interaction) && e.is_some_and(|e| e > 0.0))
}

/// The snapshot's action for `sim` (-1: none drawn).
fn drawn(w: &World, sim: u32) -> f32 {
    let mut out = vec![0.0; snapshot::CAPACITY];
    snapshot::write(w, &mut out);
    out[HEADER_LEN + sim as usize * SIM_STRIDE + sim::ACTION]
}

/// Ticks `w` for up to `minutes` until `done`, checking every tick that nobody plays a game for
/// two on their own (a tick's grace for the one left when the other stops) and that whoever
/// waits for the other isn't drawn playing; true if it got there.
fn until(w: &mut World, minutes: u64, mut done: impl FnMut(&World) -> bool) -> bool {
    let mut alone = [0u32; 8];
    for _ in 0..minutes * TICKS_PER_MINUTE {
        if done(w) {
            return true;
        }
        w.tick_once();
        for s in 0..w.sims.len() as u32 {
            let Some((o, i, Some(elapsed))) = doing(w, s) else {
                alone[s as usize] = 0;
                continue;
            };
            if !w.content.objects[w.objects[o as usize].def].interactions[i].partner {
                continue;
            }
            let with = (0..w.sims.len() as u32).any(|t| t != s && doing(w, t).is_some_and(|(to, ti, te)| (to, ti) == (o, i) && te.is_some()));
            if elapsed > 0.0 && !with {
                alone[s as usize] += 1;
                assert!(alone[s as usize] <= 2, "{} plays on alone", w.sims[s as usize].name);
            } else {
                alone[s as usize] = 0;
            }
            if elapsed == 0.0 && !with {
                assert_eq!(drawn(w, s), -1.0, "{} waits, not drawn playing", w.sims[s as usize].name);
            }
        }
    }
    done(w)
}

#[test]
fn a_partner_is_fetched_to_the_other_place_and_both_get_what_it_gives() {
    let mut w = world(EVERYONE);
    let before = [ADA, BO].map(|s| w.sims[s as usize].needs);
    order(&mut w, ADA, POOL, 0).unwrap();
    // Bo, the nearest one free, comes to the other end; it doesn't start until both are there.
    assert!(until(&mut w, 2, |w| doing(w, BO).is_some_and(|(o, i, _)| (o, i) == (POOL, 0))), "Bo is fetched");
    for s in [DEE, CY] {
        assert!(doing(&w, s).is_none(), "only one is fetched");
    }
    assert!(until(&mut w, 15, |w| playing(w, ADA, POOL, 0) && playing(w, BO, POOL, 0)), "they play");
    let table = &w.objects[POOL as usize];
    assert!(table.users[0].is_some() && table.users[1].is_some() && table.users[0] != table.users[1], "a place each");
    let gap = (w.sims[ADA as usize].pos[0] - w.sims[BO as usize].pos[0]).hypot(w.sims[ADA as usize].pos[1] - w.sims[BO as usize].pos[1]);
    assert!(gap > 2.0, "across the table from each other: {gap}");
    // Both play to the end together, and both get its fun and social.
    let (ea, eb) = (doing(&w, ADA).unwrap().2.unwrap(), doing(&w, BO).unwrap().2.unwrap());
    assert!((ea - eb).abs() < 0.2, "in step: {ea} {eb}");
    assert!(until(&mut w, 40, |w| doing(w, ADA).is_none() && doing(w, BO).is_none()), "the game ends");
    for (k, s) in [ADA, BO].into_iter().enumerate() {
        let now = w.sims[s as usize].needs;
        assert!(now[0] > before[k][0] + 0.3, "{s}: fun {} -> {}", before[k][0], now[0]);
        assert!(now[1] > before[k][1] + 0.05, "{s}: social {} -> {}", before[k][1], now[1]);
    }
}

#[test]
fn whoever_is_fetched_drops_what_they_picked_but_not_what_they_were_told() {
    let mut w = world(EVERYONE);
    // Bo told to sit: not free. Dee, the next nearest, is fetched instead.
    order(&mut w, BO, SOFA, 0).unwrap();
    order(&mut w, ADA, POOL, 0).unwrap();
    assert!(until(&mut w, 2, |w| doing(w, DEE).is_some_and(|(o, i, _)| (o, i) == (POOL, 0))), "Dee is fetched");
    assert!(until(&mut w, 30, |w| playing(w, ADA, POOL, 0) && playing(w, DEE, POOL, 0)), "Ada and Dee play");
    assert_eq!(doing(&w, BO).map(|d| d.0), Some(SOFA), "Bo still sits");
    // Two told to play the same game play it together.
    let mut w = world(EVERYONE);
    order(&mut w, ADA, PING_PONG, 0).unwrap();
    order(&mut w, CY, PING_PONG, 0).unwrap();
    assert!(until(&mut w, 30, |w| playing(w, ADA, PING_PONG, 0) && playing(w, CY, PING_PONG, 0)), "Ada and Cy play");
    assert!(doing(&w, BO).is_none() && doing(&w, DEE).is_none(), "nobody else is fetched");
}

#[test]
fn with_nobody_free_an_order_does_the_use_for_one_or_is_refused() {
    let mut w = world(r#"{"name":"Ada","x":4.5,"z":9.5,"age":35}"#);
    let err = order(&mut w, ADA, PING_PONG, 0).unwrap_err();
    assert!(err.contains("nobody is free to play ping-pong with Ada"), "{err}");
    assert!(w.sims[0].queue().next().is_none(), "nothing refused was queued");
    // The pool table's game has trick shots for one: Ada practises, and the story says so.
    order(&mut w, ADA, POOL, 0).unwrap();
    assert!(until(&mut w, 15, |w| doing(w, ADA).is_some_and(|(o, i, e)| (o, i) == (POOL, 1) && e.is_some_and(|e| e > 0.0))), "trick shots");
    assert!(w.events.iter().any(|e| e.kind == EventKind::PlayedAlone && e.a == ADA), "told in the story");
    assert!(EventKind::PlayedAlone.importance() >= 1 && EventKind::NoPartner.importance() >= 1, "in the feed");

    // Everyone else busy with what they were told: the same.
    let mut w = world(EVERYONE);
    for s in [BO, DEE] {
        order(&mut w, s, SOFA, 0).unwrap();
    }
    order(&mut w, CY, POOL, 1).unwrap();
    let err = order(&mut w, ADA, PING_PONG, 0).unwrap_err();
    assert!(err.contains("nobody is free"), "{err}");
    // Someone at the table already (Cy practising): the game isn't started there.
    assert!(until(&mut w, 15, |w| doing(w, CY).is_some_and(|d| d.2.is_some())), "Cy practises");
    let err = order(&mut w, ADA, POOL, 0).unwrap_err();
    assert!(err.contains("someone is using the pool table"), "{err}");
}

#[test]
fn free_will_plays_a_game_for_two_only_with_someone() {
    // On their own: never the game, at most its use for one.
    let mut w = world(r#"{"name":"Ada","x":4.5,"z":9.5,"age":35}"#);
    w.autonomy = true;
    w.sims[0].needs[0] = 0.1;
    assert!(!until(&mut w, 240, |w| doing(w, ADA).is_some_and(|(o, i, _)| (o, i) == (POOL, 0) || o == PING_PONG)), "no game alone");
    // Two bored grown-ups: they play together.
    let mut w = world(r#"{"name":"Ada","x":4.5,"z":9.5,"age":35},{"name":"Bo","x":10.5,"z":7.5,"age":35}"#);
    w.autonomy = true;
    for s in &mut w.sims {
        s.needs[0] = 0.1;
    }
    let game = |w: &World| [(POOL, 0), (PING_PONG, 0)].into_iter().any(|(o, i)| playing(w, ADA, o, i) && playing(w, BO, o, i));
    assert!(until(&mut w, 240, game), "a game together");
    // (And a while on, nobody ever plays on alone: `until` checks it.)
    until(&mut w, 240, |_| false);
}

#[test]
fn only_someone_its_age_is_for_is_fetched() {
    // The seesaw is a child's or a teen's: Dee rides it with Cy, not a grown-up nearer.
    let mut w = world(EVERYONE);
    let err = order(&mut w, ADA, SEESAW, 0).unwrap_err();
    assert!(err.contains("too big"), "{err}");
    order(&mut w, DEE, SEESAW, 0).unwrap();
    assert!(until(&mut w, 30, |w| playing(w, DEE, SEESAW, 0) && playing(w, CY, SEESAW, 0)), "Dee and Cy ride");
    assert!(doing(&w, ADA).is_none() && doing(&w, BO).is_none(), "no grown-up fetched");
    // Cy busy: nobody may ride it with Dee.
    let mut w = world(EVERYONE);
    order(&mut w, CY, SOFA, 0).unwrap();
    let err = order(&mut w, DEE, SEESAW, 0).unwrap_err();
    assert!(err.contains("nobody is free to ride the seesaw with Dee"), "{err}");
}

#[test]
fn a_game_saved_halfway_goes_on_after_a_load() {
    let mut w = world(EVERYONE);
    order(&mut w, ADA, POOL, 0).unwrap();
    assert!(until(&mut w, 15, |w| playing(w, ADA, POOL, 0) && playing(w, BO, POOL, 0)), "they play");
    let mut w = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    w.autonomy = false;
    // (Each sets off for their place again, and they play on together.)
    assert!(until(&mut w, 15, |w| playing(w, ADA, POOL, 0) && playing(w, BO, POOL, 0)), "they play on");
    for s in [DEE, CY] {
        assert!(doing(&w, s).is_none(), "nobody else is fetched");
    }
}

#[test]
fn one_stopping_ends_it_for_both() {
    let mut w = world(EVERYONE);
    order(&mut w, ADA, POOL, 0).unwrap();
    assert!(until(&mut w, 15, |w| playing(w, ADA, POOL, 0) && playing(w, BO, POOL, 0)), "they play");
    w.apply(Command::Cancel { sim: ADA, index: 0 }).unwrap();
    assert!(until(&mut w, 1, |w| doing(w, BO).is_none()), "over for Bo too");
    assert!(w.objects[POOL as usize].users.iter().all(Option::is_none), "the table is free");

    // Before it starts: the one fetched called off, Ada (told to) practises instead; at a game
    // with nothing for one, she gives up.
    for (object, then) in [(POOL, Some(1)), (PING_PONG, None)] {
        let mut w = world(EVERYONE);
        order(&mut w, ADA, object, 0).unwrap();
        assert!(until(&mut w, 2, |w| doing(w, BO).is_some_and(|(o, ..)| o == object)), "Bo is fetched");
        w.apply(Command::Cancel { sim: BO, index: 0 }).unwrap();
        assert!(until(&mut w, 15, |w| doing(w, ADA).is_none_or(|(o, i, e)| o != object || i != 0 || e.is_some_and(|e| e > 0.0))), "not waiting on");
        assert_eq!(doing(&w, ADA).map(|(_, i, _)| i), then, "{object}");
        assert!(!playing(&w, ADA, object, 0), "never the game alone");
    }
}

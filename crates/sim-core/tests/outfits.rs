//! Changing clothes (content `changesOutfit`): halfway through picking an outfit at the wardrobe
//! the resident is in their next one, shown to views (a change of looks) and saved; the use lasts
//! its time even with comfort full; and after the last outfit comes the creator's again.

use sim_core::world::{OUTFITS, Phase};
use sim_core::{Command, TICKS_PER_MINUTE, World};

const CONTENT: &str = r#"{
    "needs":[{"id":"comfort","label":"Comfort","decayPerHour":0.0}],
    "objects":[
      {"id":"wardrobe","name":"Wardrobe","footprint":[2,1],"interactions":[
        {"id":"outfit","label":"Pick an outfit","changesOutfit":true,"minutes":10,"effects":{"comfort":0.05},"tags":["lounge"]}]},
      {"id":"cupboard","name":"Cupboard","interactions":[
        {"id":"look","label":"Look in","minutes":10,"effects":{"comfort":0.05},"tags":["lounge"]}]}]}"#;

const TOWN: &str = r#"{"width":12,"depth":12,
    "objects":[{"def":"wardrobe","x":4,"z":1},{"def":"cupboard","x":8,"z":1}],
    "sims":[{"name":"Ada","x":4.5,"z":6.5}]}"#;

fn world() -> World {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.apply(Command::SetAutonomy { enabled: false, household: None }).unwrap();
    w
}

#[test]
fn changing_at_the_wardrobe_puts_on_the_next_outfit_halfway() {
    let mut w = world();
    w.sims[0].needs[0] = 1.0;
    assert_eq!(w.sims[0].outfit, 0, "in the creator's to start with");
    w.apply(Command::Use { sim: 0, object: 0, interaction: 0 }).unwrap();
    let before = w.structure_version();
    // At the wardrobe: still in the old clothes until halfway, in the new ones after.
    let mut changed_at = None;
    let mut minutes = 0.0;
    for t in 0..60 * TICKS_PER_MINUTE {
        if let Some(a) = w.sims[0].current()
            && let Phase::Using { elapsed } = a.phase
        {
            minutes = elapsed;
            if changed_at.is_none() && w.sims[0].outfit == 1 {
                changed_at = Some(elapsed);
            }
        } else if t > 0 && w.sims[0].current().is_none() {
            break;
        }
        w.tick_once();
    }
    let at = changed_at.expect("changed into the next outfit");
    assert!((5.0..6.0).contains(&at), "halfway through the 10 minutes, not {at}");
    assert!(minutes >= 9.9, "with comfort full it still lasts its time ({minutes} minutes)");
    assert_eq!(w.sims[0].outfit, 1);
    assert_ne!(w.structure_version(), before, "views draw the new clothes");
    assert!(sim_core::view::structure_json(&w).contains(r#""outfit":1"#));

    // Saved and loaded in it.
    let back = World::from_save_json(CONTENT, &w.save_json()).unwrap();
    assert_eq!(back.sims[0].outfit, 1);

    // After the last one, the creator's again.
    for _ in 1..OUTFITS {
        w.apply(Command::Use { sim: 0, object: 0, interaction: 0 }).unwrap();
        for _ in 0..60 * TICKS_PER_MINUTE {
            w.tick_once();
            if w.sims[0].current().is_none() {
                break;
            }
        }
    }
    assert_eq!(w.sims[0].outfit, 0);
}

#[test]
fn other_uses_leave_the_clothes_alone() {
    let mut w = world();
    w.sims[0].needs[0] = 0.0;
    w.apply(Command::Use { sim: 0, object: 1, interaction: 0 }).unwrap();
    for _ in 0..60 * TICKS_PER_MINUTE {
        w.tick_once();
    }
    assert_eq!(w.sims[0].outfit, 0);
    assert!(!w.save_json().contains(r#""outfit""#), "the creator's isn't written");
}

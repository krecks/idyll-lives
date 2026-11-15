//! The structure as updates (`view::StructureSync`, the sim worker's `world` messages): the
//! first has everything, later ones only what changed, and a view that applies them in order
//! has what the whole structure gives.

use crate::common;

use serde_json::Value;
use sim_core::view::{StructureSync, structure_json_with};
use sim_core::{Command, World};

/// What the main thread does with an update (`bridge.ts` `mergeWorld`).
fn apply(view: &mut Value, update: &str) -> Value {
    let update: Value = serde_json::from_str(update).unwrap();
    let out = view.as_object_mut().unwrap();
    // (The splice first: the changes' indices are the list's after it.)
    if let Some(splice) = update.get("objectSplice") {
        let start = splice[0].as_u64().unwrap() as usize;
        let deleted = splice[1].as_u64().unwrap() as usize;
        let objects = out["objects"].as_array_mut().unwrap();
        objects.splice(
            start..start + deleted,
            splice[2].as_array().unwrap().iter().cloned(),
        );
    }
    for (key, v) in update.as_object().unwrap() {
        match key.as_str() {
            "objectSplice" => {}
            "objectChanges" => {
                let objects = out["objects"].as_array_mut().unwrap();
                for change in v.as_array().unwrap() {
                    objects[change[0].as_u64().unwrap() as usize] = change[1].clone();
                }
            }
            "unset" => {
                for k in v.as_array().unwrap() {
                    out.remove(k.as_str().unwrap());
                }
            }
            _ => {
                out.insert(key.clone(), v.clone());
            }
        }
    }
    update
}

/// The lean structure (no lot), as `structure_json_with` gives it.
fn lean(w: &World) -> Value {
    serde_json::from_str(&structure_json_with(w, false)).unwrap()
}

#[test]
fn updates_add_up_to_the_structure() {
    let content = common::content();
    let mut w = World::from_json(&content, &common::town(&content, 4, 2), 4).unwrap();
    let mut sync = StructureSync::default();
    let mut view = Value::Object(Default::default());
    // The first has everything, the lot too (but its rooms).
    let first = apply(&mut view, &sync.update_json(&w, true));
    let whole: Value = serde_json::from_str(&structure_json_with(&w, true)).unwrap();
    for (key, v) in whole.as_object().unwrap() {
        if key != "rooms" {
            assert_eq!(&first[key], v, "{key}");
        }
    }
    assert!(first.get("rooms").is_none());
    for key in ["walls", "openings", "fences", "diagonals", "floors"] {
        view.as_object_mut().unwrap().remove(key);
    }
    assert_eq!(view, lean(&w));

    // Nothing changed: nothing but the versions.
    let same = apply(&mut view, &sync.update_json(&w, false));
    assert_eq!(same.as_object().unwrap().len(), 2, "{same}");

    // A thing changed (worn, a door opened, a crop grown): only it comes.
    let k = w.objects.len() / 2;
    w.objects[k].quality += 1;
    let update = apply(&mut view, &sync.update_json(&w, false));
    assert_eq!(
        update["objectChanges"].as_array().unwrap().len(),
        1,
        "{update}"
    );
    assert_eq!(update["objectChanges"][0][0], k);
    assert!(update.get("objects").is_none() && update.get("meta").is_none());
    assert_eq!(view, lean(&w));

    // The town lives on: what changes comes.
    for _ in 0..common::mins(50.0) {
        let before = w.structure_version();
        w.tick_once();
        if w.structure_version() == before {
            continue;
        }
        let update = apply(&mut view, &sync.update_json(&w, false));
        assert!(update.get("meta").is_none(), "meta is sent once");
        assert_eq!(view, lean(&w));
    }

    // A thing bought, and sold again: the whole list.
    let home = w.plots[w.households[0].plot.unwrap() as usize].clone();
    let bought = (home.z..home.z + home.d)
        .flat_map(|z| (home.x..home.x + home.w).map(move |x| (x, z)))
        .find(|&(x, z)| {
            w.apply(Command::Buy {
                household: 0,
                object: "plant".into(),
                at: Some([x, z, 0]),
                colors: Default::default(),
                turn: None,
                offset: None,
                size: None,
                tall: None,
            })
            .is_ok()
        });
    assert!(bought.is_some(), "room for a plant");
    let update = apply(&mut view, &sync.update_json(&w, false));
    // (Only the plant, not the whole list again.)
    assert!(update.get("objects").is_none(), "{update}");
    let splice = &update["objectSplice"];
    assert_eq!(splice[0], w.objects.len() - 1, "{update}");
    assert_eq!(splice[1], 0, "{update}");
    assert_eq!(splice[2].as_array().unwrap().len(), 1, "{update}");
    assert_eq!(view, lean(&w));
    let plant = w.objects.last().unwrap().stable;
    w.apply(Command::Sell {
        household: 0,
        object: plant,
    })
    .unwrap();
    apply(&mut view, &sync.update_json(&w, false));
    assert_eq!(view, lean(&w));

    // A thing sold from the middle of the list: only its place goes.
    let gone = (0..w.objects.len())
        .rev()
        .find(|&i| {
            let stable = w.objects[i].stable;
            w.apply(Command::Sell {
                household: 0,
                object: stable,
            })
            .is_ok()
        })
        .expect("something to sell");
    let update = apply(&mut view, &sync.update_json(&w, false));
    assert_eq!(update["objectSplice"][0], gone, "{update}");
    assert_eq!(update["objectSplice"][1], 1, "{update}");
    assert!(
        update["objectSplice"][2].as_array().unwrap().is_empty(),
        "{update}"
    );
    assert_eq!(view, lean(&w));

    // The lot changed: only the parts of it that did come.
    let mut whole = view.clone();
    let lot = apply(&mut whole, &sync.update_json(&w, true));
    for key in ["walls", "openings", "fences", "diagonals", "floors"] {
        assert!(lot.get(key).is_none(), "{key} unchanged: {lot}");
    }

    // Starting over: everything again.
    sync = StructureSync::default();
    let again: Value = serde_json::from_str(&sync.update_json(&w, false)).unwrap();
    let mut fresh = again.clone();
    fresh.as_object_mut().unwrap().remove("unset");
    assert_eq!(fresh, lean(&w));
}

//! Washing with the shipped content: a shower, a bath, washing hands, washing up and the laundry,
//! all at once, each shown with its animation (the web side plays each one's water sound by it).
//! With `DUMP_STATES=1` it writes the moment they set off as `water-sounds`.

use crate::common;

use common::buy;
use sim_core::world::{Phase, TaskKind};
use sim_core::{Command, MINUTES_PER_TICK, World};

/// An open 16×14 plot with five residents.
const TOWN: &str = r#"{"width":16,"depth":16,
    "plots":[{"name":"Home","x":0,"z":0,"w":16,"d":14,"entry":[8.5,13.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "sims":[{"name":"Ada","household":0,"x":3.5,"z":10.5,"traits":["neat"]},{"name":"Bo","household":0,"x":5.5,"z":10.5,"traits":["neat"]},
        {"name":"Cy","household":0,"x":7.5,"z":10.5,"traits":["neat"]},{"name":"Di","household":0,"x":9.5,"z":10.5,"traits":["neat"]},
        {"name":"Ed","household":0,"x":11.5,"z":10.5,"traits":["neat"]}]}"#;

fn interaction(w: &World, object: u32, id: &str) -> usize {
    w.content.objects[w.objects[object as usize].def]
        .interactions
        .iter()
        .position(|i| i.id == id)
        .unwrap_or_else(|| panic!("{id}"))
}

#[test]
fn showering_bathing_washing_up_and_laundry_at_once() {
    let mut w = World::from_json(&common::content(), TOWN, 1).unwrap();
    w.autonomy = false;
    // A dirty plate to wash up (washing up needs dishes to wash, content `workOn`).
    let plate = w.content.item_index("plateDirty").unwrap();
    w.items.add(sim_core::items::ItemInstance {
        id: 0,
        def: plate,
        at: sim_core::items::ItemAt::Floor {
            x: 1,
            z: 1,
            offset: [0, 0],
        },
        servings: 0,
        quality: 1.0,
        made_at: 0,
        household: 0,
        spoiled: false,
        recipe: None,
    });
    let uses = [
        ("showerStall", 2, 3, "shower", "shower"),
        ("bathtub", 5, 3, "bath", "bath"),
        ("sink", 9, 3, "wash", "wash"),
        ("farmhouseSink", 11, 3, "washUp", "wash"),
        ("laundryWasher", 13, 3, "laundry", "clean"),
    ];
    let mut wanted = Vec::new();
    for (i, (def, x, z, id, anim)) in uses.into_iter().enumerate() {
        let object = buy(&mut w, def, x, z);
        let interaction = interaction(&w, object, id);
        w.apply(Command::Use {
            sim: i as u32,
            object,
            interaction,
        })
        .unwrap();
        wanted.push((object, interaction, anim));
    }
    common::dump_state(&w, "water-sounds");

    // Within a few minutes all five are at it, each shown with the animation for it.
    let mut at_it = [false; 5];
    for _ in 0..(10.0 / MINUTES_PER_TICK) as u32 {
        w.tick_once();
        for (i, &(object, interaction, anim)) in wanted.iter().enumerate() {
            let Some(a) = w.sims[i].current() else {
                continue;
            };
            if matches!(a.phase, Phase::Using { .. })
                && matches!(a.task.kind, TaskKind::Use { object: o, interaction: n } if o == object && n == interaction)
            {
                let shown = w.content.objects[w.objects[object as usize].def].interactions
                    [interaction]
                    .anim
                    .map(|a| w.content.animations[a].as_str());
                assert_eq!(shown, Some(anim), "{}", w.sims[i].name);
                at_it[i] = true;
            }
        }
    }
    assert_eq!(at_it, [true; 5], "all five got there");
    common::check(&w);
}

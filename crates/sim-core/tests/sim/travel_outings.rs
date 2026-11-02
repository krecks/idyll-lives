//! Choosing nearby things in an open-world town (`index::ObjectIndex::usable_near`,
//! `Choice::Outing`): residents use the public things within reach of where they are, and go on
//! outings to places farther off when they have the time.

use crate::common;

use common::hamlets::two_hamlets;
use sim_core::{MINUTES_PER_TICK, World};

const TICKS_PER_MINUTE: u64 = (1.0 / MINUTES_PER_TICK) as u64;

#[test]
fn residents_go_on_outings_to_places_out_of_reach() {
    let content = common::content();
    let h = two_hamlets(&content, 31);
    let mut w = World::from_json(&content, &h.town, 31).expect("the hamlets load");
    let reach = w.content.autonomy.reach;
    let mut outings = std::collections::BTreeSet::new();
    let end = w.tick + 3 * 24 * 60 * TICKS_PER_MINUTE;
    while w.tick < end {
        w.tick_once();
        for s in &w.sims {
            let Some(v) = s.visiting else { continue };
            let plot = &w.plots[v.plot as usize];
            let Some(home) = w.households[s.household as usize].plot else {
                continue;
            };
            let (a, b) = (plot.arrival_point(), w.plots[home as usize].arrival_point());
            if plot.public && sim_core::travel::dist(a, b) > reach {
                outings.insert((s.id, v.plot));
            }
        }
    }
    eprintln!("outings (resident, place): {outings:?}");
    assert!(!outings.is_empty(), "nobody went anywhere out of reach");
}

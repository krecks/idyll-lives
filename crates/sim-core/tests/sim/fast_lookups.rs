//! The quick answers the simulation keeps or works out in one go (what items are on or in, where
//! plots' doors get on the roads, every household's weekly costs) agree with asking afresh
//! (`common::check_lookups`, `costs`),
//! over a day of free will in two hamlets: an open world, with cars, shops and a park, kitchens
//! with food on the tops and in the fridges. (`common::check` asks the same wherever the other
//! tests call it, the generated worlds and houses among them.)

use crate::common;

use common::hamlets::{two_hamlets, with_cars};
use sim_core::World;

#[test]
fn quick_lookups_agree_with_asking_afresh() {
    let content = common::content();
    let h = two_hamlets(&content, 5);
    let mut w = World::from_json(&content, &with_cars(&h), 5).expect("the hamlets load");
    let (mut held, mut doors, mut charged) = (0, 0, 0);
    for _ in 0..24 {
        for _ in 0..common::TICKS_PER_HOUR {
            w.tick_once();
        }
        let (h, d) = common::check_lookups(&w);
        held += h;
        doors += d;
        charged += costs(&w);
    }
    assert!(held > 0, "no item was on or in anything");
    assert!(doors > 0, "no door got on the roads");
    assert!(charged > 0, "no household had costs");
}

/// Every household's weekly costs found together (`life::weekly_costs_all`, as the UI and rent
/// day do) against one household's at a time. How many households had costs.
fn costs(w: &World) -> usize {
    let all = sim_core::life::weekly_costs_all(w);
    assert_eq!(all.len(), w.households.len());
    for (h, &costs) in all.iter().enumerate() {
        assert_eq!(
            costs,
            sim_core::life::weekly_costs(w, h),
            "household {h}'s weekly costs"
        );
    }
    all.iter().flatten().count()
}

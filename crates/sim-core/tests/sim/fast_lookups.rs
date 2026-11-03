//! The quick answers the simulation keeps or works out in one go (what items are on or in, where
//! public places' doors get on the roads) agree with asking afresh (`common::check_lookups`),
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
    let (mut held, mut doors) = (0, 0);
    for _ in 0..24 {
        for _ in 0..common::TICKS_PER_HOUR {
            w.tick_once();
        }
        let (h, d) = common::check_lookups(&w);
        held += h;
        doors += d;
    }
    assert!(held > 0, "no item was on or in anything");
    assert!(doors > 0, "no public place's door got on the roads");
}

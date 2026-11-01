//! Changes to walking are logged with where they were (`World::nav_changed_since`): a walker
//! whose way lies far from a change keeps it, one whose way it touches finds its way again, and
//! one that last looked longer ago than the log goes back always does.

mod common;

use sim_core::World;
use sim_core::path::NAV_LOG;

#[test]
fn changes_to_walking_are_known_by_where_they_were() {
    let content = common::content();
    let town = common::town(&content, 3, 2);
    let mut w = World::from_json(&content, &town, 3).unwrap();
    let home = w.households[0].plot.unwrap() as usize;
    let p = w.plots[home].clone();
    let seen = w.nav_version();
    // A plant in the home's garden, wherever one fits.
    let placed = (p.z..p.z + p.d)
        .flat_map(|z| (p.x..p.x + p.w).map(move |x| (x, z)))
        .find(|&(x, z)| {
            w.apply(sim_core::Command::Buy {
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
        })
        .expect("room for a plant");
    assert_ne!(w.nav_version(), seen);
    let (x, z) = placed;
    assert!(
        w.nav_changed_since(seen, [x - 2, z - 2, x + 3, z + 3]),
        "near it"
    );
    let far = [x + 60, z, x + 70, z + 10];
    assert!(!w.nav_changed_since(seen, far), "far from it");
    assert!(
        !w.nav_changed_since(w.nav_version(), [x, z, x + 1, z + 1]),
        "nothing since"
    );
    // More changes than the log keeps (the plant sold and bought back): anything may have
    // changed since.
    for _ in 0..NAV_LOG {
        let plant = w.objects.last().unwrap().stable;
        w.apply(sim_core::Command::Sell {
            household: 0,
            object: plant,
        })
        .unwrap();
        common::buy(&mut w, "plant", x, z);
    }
    assert!(w.nav_version().wrapping_sub(seen) as usize > NAV_LOG);
    assert!(w.nav_changed_since(seen, far), "older than the log");
}

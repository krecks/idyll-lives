//! The commands for checks (`?debug`'s `__send`, `tools/bench/check.mjs`; no menu sends them): a
//! resident's need set, one need or all of them; how two residents stand with each other set both
//! ways, partners included (one partner each), courting only between grown-ups who aren't family. Bad
//! input is refused or kept in range.

mod common;

use sim_core::social::{BondPreset, Kin};
use sim_core::{Command, World};

const LOT: &str = r#"{"width":12,"depth":12,
    "households":[{"name":"Home","player":true}],
    "sims":[{"name":"Ada","traits":["cheerful"],"x":2.5,"z":2.5},{"name":"Ben","traits":["bookworm"],"x":4.5,"z":2.5},
             {"name":"Cal","traits":["cheerful"],"x":6.5,"z":2.5}]}"#;

fn world() -> World {
    World::from_json(&common::content(), LOT, 1).unwrap()
}

#[test]
fn set_need_sets_one_need_or_all_within_range() {
    let mut w = world();
    let bladder = w
        .content
        .needs
        .iter()
        .position(|n| n.id == "bladder")
        .unwrap();
    w.apply(Command::SetNeed {
        sim: 0,
        need: "bladder".into(),
        value: 0.05,
    })
    .unwrap();
    assert_eq!(w.sims[0].needs[bladder], 0.05);
    w.apply(Command::SetNeed {
        sim: 0,
        need: "all".into(),
        value: 2.0,
    })
    .unwrap();
    let n = w.content.needs.len();
    assert!(
        w.sims[0].needs[..n].iter().all(|&v| v == 1.0),
        "kept to 0..1: {:?}",
        &w.sims[0].needs[..n]
    );
    assert!(
        w.apply(Command::SetNeed {
            sim: 0,
            need: "joy".into(),
            value: 0.5
        })
        .is_err(),
        "an unknown need is refused"
    );
    assert!(
        w.apply(Command::SetNeed {
            sim: 9,
            need: "bladder".into(),
            value: 0.5
        })
        .is_err(),
        "an unknown resident is refused"
    );
}

#[test]
fn set_bond_sets_both_ways_and_keeps_one_partner_each() {
    let mut w = world();
    w.apply(Command::SetBond {
        a: 0,
        b: 1,
        friendship: Some(70.0),
        romance: Some(150.0),
        chemistry: Some(0.6),
        partners: Some(true),
    })
    .unwrap();
    for (x, y) in [(0, 1), (1, 0)] {
        let r = w.relationships.get(x, y);
        assert!(r.met && r.partners, "{x} and {y}: {r:?}");
        assert_eq!(
            (r.friendship, r.romance),
            (70.0, 100.0),
            "romance kept to -100..100"
        );
    }
    assert!((w.relationships.chemistry(0, 1) - 0.6).abs() < 1e-6);
    assert_eq!(w.relationships.partner_of(0), Some(1));
    // Only what's given changes.
    w.apply(Command::SetBond {
        a: 1,
        b: 0,
        friendship: Some(-20.0),
        romance: None,
        chemistry: None,
        partners: None,
    })
    .unwrap();
    assert_eq!(
        (
            w.relationships.get(0, 1).friendship,
            w.relationships.get(0, 1).romance
        ),
        (-20.0, 100.0)
    );
    // A new partner ends the old one.
    w.apply(Command::SetBond {
        a: 0,
        b: 2,
        friendship: None,
        romance: None,
        chemistry: None,
        partners: Some(true),
    })
    .unwrap();
    assert_eq!(w.relationships.partner_of(0), Some(2));
    assert_eq!(w.relationships.partner_of(1), None);
    assert!(!w.relationships.get(1, 0).partners);
    assert!(
        w.apply(Command::SetBond {
            a: 1,
            b: 1,
            friendship: Some(1.0),
            romance: None,
            chemistry: None,
            partners: None
        })
        .is_err(),
        "not with themselves"
    );
}

#[test]
fn set_bond_keeps_family_out_of_romance() {
    let mut w = world();
    // (Cal is Ben's child.)
    w.relationships.bond(
        1,
        2,
        &BondPreset {
            friendship: 50.0,
            romance: 0.0,
            partners: false,
            kin: Kin::Child,
        },
    );
    for (romance, partners) in [(Some(40.0), None), (None, Some(true))] {
        assert!(
            w.apply(Command::SetBond {
                a: 1,
                b: 2,
                friendship: None,
                romance,
                chemistry: None,
                partners
            })
            .is_err(),
            "{romance:?} {partners:?}"
        );
    }
    assert!(
        !w.relationships.get(1, 2).partners && w.relationships.get(2, 1).romance == 0.0,
        "nothing changed"
    );
    // Fond of each other, or falling out, is up to them.
    w.apply(Command::SetBond {
        a: 2,
        b: 1,
        friendship: Some(-30.0),
        romance: Some(0.0),
        chemistry: None,
        partners: Some(false),
    })
    .unwrap();
    assert_eq!(w.relationships.get(1, 2).friendship, -30.0);
}

#[test]
fn set_bond_keeps_children_out_of_romance() {
    let mut w = world();
    w.sims[2].age = 8.0;
    assert!(
        w.apply(Command::SetBond {
            a: 0,
            b: 2,
            friendship: None,
            romance: Some(40.0),
            chemistry: None,
            partners: None
        })
        .is_err()
    );
    assert!(
        w.apply(Command::SetBond {
            a: 2,
            b: 0,
            friendship: None,
            romance: None,
            chemistry: None,
            partners: Some(true)
        })
        .is_err()
    );
    w.apply(Command::SetBond {
        a: 0,
        b: 2,
        friendship: Some(60.0),
        romance: None,
        chemistry: None,
        partners: None,
    })
    .unwrap();
    assert_eq!(w.relationships.get(2, 0).friendship, 60.0);
}

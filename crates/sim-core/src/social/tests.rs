//! Tests for social life.

use super::*;

#[test]
fn levels() {
    assert_eq!(friend_level(-70.0), LEVEL_ENEMY);
    assert_eq!(friend_level(0.0), 0);
    assert_eq!(friend_level(40.0), LEVEL_FRIEND);
    assert_eq!(friend_level(90.0), LEVEL_BEST_FRIEND);
}

#[test]
fn matrix_grows_and_keeps_data() {
    let mut r = Relationships::default();
    r.grow(2);
    r.adjust(0, 1, 20.0, 5.0);
    r.set_chemistry(0, 1, 0.4);
    r.grow(3);
    assert_eq!(r.get(0, 1).friendship, 20.0);
    assert_eq!(r.get(1, 0).friendship, 0.0);
    assert_eq!(r.chemistry(1, 0), 0.4);
    assert_eq!(r.len(), 3);
    // (Past the room made for it, one resident at a time, as a town loads.)
    r.adjust(2, 1, -10.0, 0.0);
    for n in 4..=2 * GROW_BY + 1 {
        r.grow(n);
        r.adjust(n - 1, 0, n as f32 / 10.0, 0.0);
    }
    assert_eq!(r.len(), 2 * GROW_BY + 1);
    assert_eq!(r.get(0, 1).friendship, 20.0);
    assert_eq!(r.get(2, 1).friendship, -10.0);
    assert_eq!(r.chemistry(0, 1), 0.4);
    for n in 4..=2 * GROW_BY + 1 {
        assert_eq!(r.get(n - 1, 0).friendship, n as f32 / 10.0);
        assert_eq!(*r.get(0, n - 1), Relationship::default());
    }
}

#[test]
fn milestones_detect_friendship_and_crush() {
    let before = Relationship {
        friendship: 30.0,
        romance: 30.0,
        met: true,
        ..Default::default()
    };
    let after = Relationship {
        friendship: 40.0,
        romance: 36.0,
        met: true,
        ..Default::default()
    };
    let events = milestone_events((&before, &before), (&after, &after));
    assert!(events.contains(&(EventKind::BecameFriends, false)));
    assert!(events.contains(&(EventKind::Crush, false)));
    assert!(events.contains(&(EventKind::Crush, true)));
}

#[test]
fn decay_drifts_toward_neutral() {
    let mut r = Relationships::default();
    r.grow(2);
    r.adjust(0, 1, 50.0, 10.0);
    r.decay(&SocialRules::empty());
    assert!(r.get(0, 1).friendship < 50.0);
    assert!(r.get(0, 1).romance < 10.0);
}

use crate::World;
use crate::world::{Task, TaskKind};

const CONTENT: &str = r#"{
    "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.1}],
    "objects":[{"id":"fridge","name":"Fridge","interactions":[
        {"id":"snack","label":"Snack","minutes":10,"effects":{"hunger":0.4}}]}]}"#;

const LOT: &str = r#"{"width":12,"depth":12,
    "objects":[{"def":"fridge","x":1,"z":1,"rot":0},{"def":"fridge","x":4,"z":1,"rot":0},{"def":"fridge","x":7,"z":1,"rot":0}],
    "sims":[{"name":"Ada","x":3.5,"z":5.5}]}"#;

fn snack(object: u32) -> TaskKind {
    TaskKind::Use {
        object,
        interaction: 0,
    }
}

fn object(t: Option<TaskKind>) -> Option<u32> {
    match t {
        Some(TaskKind::Use { object, .. }) => Some(object),
        _ => None,
    }
}

/// What a resident does next out of habit, and what they couldn't get to, follow the
/// object they name when ids shift, and go with it.
#[test]
fn follow_ups_and_what_they_couldnt_reach_follow_the_object() {
    let mut w = World::from_json(CONTENT, LOT, 7).unwrap();
    w.sims[0].follow_up = Some(Task {
        kind: snack(2),
        directed: false,
        routine: true,
    });
    w.sims[0].remember_unreachable(snack(1), 1000);
    w.remove_object(0);
    assert_eq!(object(w.sims[0].follow_up.map(|t| t.kind)), Some(1));
    assert_eq!(object(w.sims[0].unreachable[0].map(|s| s.0)), Some(0));
    w.remove_object(0);
    assert_eq!(object(w.sims[0].follow_up.map(|t| t.kind)), Some(0));
    assert!(
        w.sims[0].unreachable[0].is_none(),
        "the one they couldn't reach is gone"
    );
    w.remove_object(0);
    assert!(
        w.sims[0].follow_up.is_none(),
        "the one they'd use next is gone"
    );
}

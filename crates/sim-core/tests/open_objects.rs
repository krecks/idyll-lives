//! Things that open and close (content `when` and `leaves`): a wall bed folds down for the night
//! and back up in the morning, or whenever someone is told to; it stays as it was left, saved.
//! And the shipped ones: the futon and the sofa bed (opened to sleep or nap in, put away after),
//! the washing line (the washing stays out until it's taken in) and the hot tub (its cover off for
//! a soak, and back on after it: `closeAfter`).

mod common;

use sim_core::social::EventKind;
use sim_core::world::TaskKind;
use sim_core::{Command, TICKS_PER_MINUTE, World, clock};

const CONTENT: &str = r#"{
    "needs":[{"id":"energy","label":"Energy","decayPerHour":0.01}],
    "dayRhythm":{"sleepTags":["sleep"]},
    "objects":[{"id":"wallBed","name":"Wall bed","price":900,"footprint":[2,3],"slots":2,"interactions":[
        {"id":"sleep","label":"Sleep","minutes":60,"pose":"lie","effects":{"energy":1.0},"tags":["sleep"],"leaves":"open"},
        {"id":"foldDown","label":"Fold down","minutes":2,"autonomous":false,"when":"closed","leaves":"open"},
        {"id":"foldUp","label":"Fold up","minutes":2,"autonomous":false,"when":"open","leaves":"closed"}]}],
    "economy":{"startingFunds":1000}}"#;

const TOWN: &str = r#"{"width":12,"depth":12,
    "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":12,"entry":[6.5,11.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "objects":[{"def":"wallBed","x":4,"z":1}],
    "sims":[{"name":"Ada","x":3.5,"z":7.5},{"name":"Bea","x":6.5,"z":7.5}]}"#;

const SLEEP: usize = 0;
const FOLD_DOWN: usize = 1;
const FOLD_UP: usize = 2;

fn world() -> World {
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    w
}

fn using(w: &World, sim: usize) -> Option<usize> {
    match w.sims[sim].current()?.task.kind {
        TaskKind::Use { interaction, .. } => Some(interaction),
        _ => None,
    }
}

/// Whether resident `sim` is at interaction `i`, using it (not on the way).
fn at_it(w: &World, sim: usize, i: usize) -> bool {
    using(w, sim) == Some(i)
        && w.sims[sim]
            .current()
            .is_some_and(|a| matches!(a.phase, sim_core::world::Phase::Using { .. }))
}

/// Ticks `w` for up to `minutes`, until `done`; true if it got there.
fn until(w: &mut World, minutes: u64, mut done: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..minutes * TICKS_PER_MINUTE {
        if done(w) {
            return true;
        }
        w.tick_once();
    }
    done(w)
}

#[test]
fn folding_down_and_up_when_told() {
    let mut w = world();
    assert!(!w.objects[0].open, "it's bought folded up");
    assert!(
        w.apply(Command::Use {
            sim: 0,
            object: 0,
            interaction: FOLD_UP
        })
        .is_err(),
        "nothing to fold up"
    );

    let version = w.structure_version();
    w.apply(Command::Use {
        sim: 0,
        object: 0,
        interaction: FOLD_DOWN,
    })
    .unwrap();
    assert!(until(&mut w, 30, |w| w.objects[0].open), "folded down");
    assert_ne!(w.structure_version(), version, "the house looks different");
    assert_eq!(
        using(&w, 0),
        Some(FOLD_DOWN),
        "as the folding starts, not once it's done"
    );
    assert!(
        until(&mut w, 10, |w| w.sims[0].current().is_none()),
        "done in a couple of minutes"
    );
    assert!(w.objects[0].open, "and it stays down");
    assert!(
        w.apply(Command::Use {
            sim: 0,
            object: 0,
            interaction: FOLD_DOWN
        })
        .is_err(),
        "down already"
    );

    let w = common::reload(&w, CONTENT);
    assert!(w.objects[0].open, "still down after loading");

    let mut w = w;
    w.apply(Command::Use {
        sim: 1,
        object: 0,
        interaction: FOLD_UP,
    })
    .unwrap();
    assert!(
        until(&mut w, 30, |w| !w.objects[0].open
            && w.sims[1].current().is_none()),
        "folded up again"
    );
}

#[test]
fn going_to_bed_folds_it_down_and_the_last_one_up_folds_it_up() {
    let mut w = world();
    // Ada is less tired: she's up first, with Bea still asleep.
    w.sims[0].needs[0] = 0.5;
    w.sims[1].needs[0] = 0.0;
    for sim in 0..2 {
        w.apply(Command::Use {
            sim,
            object: 0,
            interaction: SLEEP,
        })
        .unwrap();
    }
    assert!(
        until(&mut w, 30, |w| w.objects[0].open),
        "it folds down as they go to bed"
    );
    assert!(
        until(&mut w, 30, |w| w.sims.iter().all(|s| s.asleep(&w.content))),
        "both asleep"
    );

    let mut folded_by = None;
    let mut ada_up_with_bea_asleep = false;
    let done = until(&mut w, 120, |w| {
        let asleep = |s: usize| w.sims[s].asleep(&w.content);
        assert!(
            w.objects[0].open || !(asleep(0) || asleep(1)),
            "never folded up over a sleeper"
        );
        ada_up_with_bea_asleep |= !asleep(0) && asleep(1);
        for s in 0..2 {
            if using(w, s) == Some(FOLD_UP) {
                folded_by.get_or_insert(s);
            }
        }
        !w.objects[0].open && w.sims.iter().all(|s| s.current().is_none())
    });
    assert!(ada_up_with_bea_asleep, "Ada got up first");
    assert!(done, "folded up again once both are up");
    assert_eq!(folded_by, Some(1), "by Bea, the last one up");
}

#[test]
fn nobody_folds_it_up_over_someone_in_it() {
    let mut w = world();
    w.sims[0].needs[0] = 0.0;
    w.apply(Command::Use {
        sim: 0,
        object: 0,
        interaction: SLEEP,
    })
    .unwrap();
    assert!(
        until(&mut w, 30, |w| w.sims[0].asleep(&w.content)),
        "Ada is asleep"
    );
    let refused = w.apply(Command::Use {
        sim: 1,
        object: 0,
        interaction: FOLD_UP,
    });
    assert!(
        refused.is_err_and(|e| e.to_string().contains("someone is using")),
        "Bea can't fold it up on her"
    );
}

/// The shipped things that open and close, on one plot without walls.
const SHIPPED_TOWN: &str = r#"{"width":18,"depth":14,
    "plots":[{"name":"Home","x":0,"z":0,"w":18,"d":14,"entry":[8.5,13.5]}],
    "households":[{"name":"Player","plot":0,"player":true}],
    "objects":[{"def":"bedFuton","x":1,"z":1},{"def":"sofaBed","x":5,"z":1},
               {"def":"clothesline","x":9,"z":1},{"def":"hotTubCedar","x":14,"z":1}],
    "sims":[{"name":"Ada","traits":["bookworm"],"x":4.5,"z":8.5},{"name":"Bea","traits":["bookworm"],"x":9.5,"z":8.5}]}"#;

const FUTON: u32 = 0;
const SOFA_BED: u32 = 1;
const WASHING_LINE: u32 = 2;
const HOT_TUB: u32 = 3;

/// The shipped content and the town above at 9 am, everyone's needs full, nobody choosing for
/// themselves.
fn shipped() -> (World, String) {
    let content = common::content();
    let mut w = World::from_json(&content, SHIPPED_TOWN, 1).unwrap();
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    let morning = clock::tick_at(1, 9.0 * 60.0).unwrap();
    while w.tick < morning {
        w.tick_once();
    }
    let n = w.content.needs.len();
    for s in &mut w.sims {
        s.needs[..n].fill(1.0);
    }
    (w, content)
}

fn interaction(w: &World, object: u32, id: &str) -> usize {
    let def = w.objects[object as usize].def;
    w.content.objects[def]
        .interactions
        .iter()
        .position(|i| i.id == id)
        .unwrap_or_else(|| panic!("no {id}"))
}

fn energy(w: &World) -> usize {
    w.content
        .needs
        .iter()
        .position(|n| n.id == "energy")
        .unwrap()
}

/// One of the shipped things: what opens it when told, what closes it, and what's only offered
/// while it's closed.
struct Thing {
    object: u32,
    opens: &'static str,
    closes: &'static str,
    only_closed: &'static [&'static str],
    /// Whoever opened it closes it again when done (`closeAfter`).
    closed_after: bool,
    /// Opened and closed as the hands go, so once the use is done (`gradual`: the washing).
    gradual: bool,
}

const THINGS: [Thing; 4] = [
    Thing {
        object: FUTON,
        opens: "unfold",
        closes: "foldAway",
        only_closed: &["unfold"],
        closed_after: false,
        gradual: false,
    },
    Thing {
        object: SOFA_BED,
        opens: "pullOut",
        closes: "foldAway",
        only_closed: &["pullOut", "relax"],
        closed_after: false,
        gradual: false,
    },
    Thing {
        object: WASHING_LINE,
        opens: "washing",
        closes: "takeIn",
        only_closed: &["washing"],
        closed_after: false,
        gradual: true,
    },
    Thing {
        object: HOT_TUB,
        opens: "soak",
        closes: "coverOn",
        only_closed: &[],
        closed_after: true,
        gradual: false,
    },
];

#[test]
fn the_shipped_things_open_and_close_when_told() {
    for t in &THINGS {
        let (mut w, content) = shipped();
        let o = t.object as usize;
        let name = w.content.objects[w.objects[o].def].id.clone();
        assert!(!w.objects[o].open, "{name}: bought closed");
        let closes = interaction(&w, t.object, t.closes);
        let refused = w.apply(Command::Use {
            sim: 0,
            object: t.object,
            interaction: closes,
        });
        assert!(
            refused.is_err_and(|e| e.to_string().contains("is closed")),
            "{name}: nothing to close"
        );

        let opens = interaction(&w, t.object, t.opens);
        w.apply(Command::Use {
            sim: 0,
            object: t.object,
            interaction: opens,
        })
        .unwrap();
        if t.gradual {
            // Closed while the hands are at it, open once they're done.
            let mut shut_while_used = false;
            assert!(
                until(&mut w, 40, |w| {
                    shut_while_used |= at_it(w, 0, opens) && !w.objects[o].open;
                    w.objects[o].open
                }),
                "{name}: {} opens it",
                t.opens
            );
            assert!(
                shut_while_used && using(&w, 0).is_none(),
                "{name}: once the use is done"
            );
        } else {
            assert!(
                until(&mut w, 30, |w| w.objects[o].open),
                "{name}: {} opens it",
                t.opens
            );
            assert_eq!(using(&w, 0), Some(opens), "{name}: as the use starts");
        }
        for id in t.only_closed {
            let i = interaction(&w, t.object, id);
            let refused = w.apply(Command::Use {
                sim: 1,
                object: t.object,
                interaction: i,
            });
            assert!(
                refused.is_err_and(|e| e.to_string().contains("already open")),
                "{name}: no {id} while it's open"
            );
        }
        if t.closed_after {
            let mut closed_by_them = false;
            let done = until(&mut w, 90, |w| {
                closed_by_them |= using(w, 0) == Some(closes);
                !w.objects[o].open && w.sims[0].current().is_none()
            });
            assert!(
                done && closed_by_them,
                "{name}: closed again after the {} by them",
                t.opens
            );
            continue;
        }
        assert!(
            until(&mut w, 60, |w| w.sims[0].current().is_none()),
            "{name}: {} done",
            t.opens
        );
        assert!(
            !until(&mut w, 60, |w| !w.objects[o].open),
            "{name}: it stays open"
        );

        let mut w = common::reload(&w, &content);
        assert!(w.objects[o].open, "{name}: still open after loading");
        // (The washing dries on the line first: it comes in once its run is done, `machines.rs`.)
        assert!(
            !w.objects[o].running() || until(&mut w, 200, |w| w.objects[o].done()),
            "{name}: done"
        );
        w.apply(Command::Use {
            sim: 1,
            object: t.object,
            interaction: closes,
        })
        .unwrap();
        let mut open_while_used = false;
        assert!(
            until(&mut w, 30, |w| {
                open_while_used |= at_it(w, 1, closes) && w.objects[o].open;
                !w.objects[o].open && w.sims[1].current().is_none()
            }),
            "{name}: {} closes it",
            t.closes
        );
        assert_eq!(
            open_while_used, t.gradual,
            "{name}: closed as the hands go, or as it starts"
        );
        for id in t.only_closed {
            let i = interaction(&w, t.object, id);
            assert!(
                w.objects[o].allows(&w.content.objects[w.objects[o].def].interactions[i], 0),
                "{name}: {id} again"
            );
        }
    }
}

#[test]
fn nobody_closes_the_shipped_things_over_someone_using_them() {
    for (object, uses, closes) in [(SOFA_BED, "doze", "foldAway"), (HOT_TUB, "soak", "coverOn")] {
        let (mut w, _) = shipped();
        let e = energy(&w);
        w.sims[0].needs[e] = 0.3;
        let i = interaction(&w, object, uses);
        w.apply(Command::Use {
            sim: 0,
            object,
            interaction: i,
        })
        .unwrap();
        assert!(
            until(&mut w, 30, |w| w.objects[object as usize].open
                && using(w, 0) == Some(i)),
            "{uses} under way"
        );
        let close = interaction(&w, object, closes);
        let refused = w.apply(Command::Use {
            sim: 1,
            object,
            interaction: close,
        });
        assert!(
            refused.is_err_and(|e| e.to_string().contains("someone is using")),
            "no {closes} over them"
        );
    }
}

#[test]
fn nobody_opens_the_sofa_bed_under_someone_relaxing() {
    let (mut w, _) = shipped();
    let o = SOFA_BED as usize;
    let relax = interaction(&w, SOFA_BED, "relax");
    w.apply(Command::Use {
        sim: 0,
        object: SOFA_BED,
        interaction: relax,
    })
    .unwrap();
    assert!(
        until(&mut w, 30, |w| using(w, 0) == Some(relax)
            && !w.objects[o].open),
        "Ada relaxes on the closed sofa bed"
    );
    for id in ["doze", "pullOut"] {
        let i = interaction(&w, SOFA_BED, id);
        let refused = w.apply(Command::Use {
            sim: 1,
            object: SOFA_BED,
            interaction: i,
        });
        assert!(
            refused.is_err_and(|e| e.to_string().contains("someone is using")),
            "no {id} under the sitter"
        );
    }
    assert!(!w.objects[o].open, "still folded");
    assert_eq!(using(&w, 0), Some(relax), "Ada keeps relaxing");
}

/// The shipped content with only a sofa bed, close enough to sit on with the phone; nobody
/// choosing for themselves.
fn sofa_bed_only() -> World {
    const TOWN: &str = r#"{"width":12,"depth":10,
        "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":10,"entry":[6.5,9.5]}],
        "households":[{"name":"Player","plot":0,"player":true}],
        "objects":[{"def":"sofaBed","x":5,"z":1}],
        "sims":[{"name":"Ada","traits":["bookworm"],"x":4.5,"z":4.5},{"name":"Bea","traits":["bookworm"],"x":7.5,"z":4.5}]}"#;
    let mut w = World::from_json(&common::content(), TOWN, 1).unwrap();
    w.apply(Command::SetAutonomy {
        enabled: false,
        household: None,
    })
    .unwrap();
    w
}

/// The seat someone on the phone took (`Some(None)`: on it standing), if they're on it.
fn phone_seat(w: &World, sim: usize) -> Option<Option<u32>> {
    match w.sims[sim].current()?.task.kind {
        TaskKind::Phone { seat, .. } => Some(seat),
        _ => None,
    }
}

fn browse(w: &World) -> usize {
    w.content
        .phone
        .actions
        .iter()
        .position(|a| a.id == "browse")
        .unwrap()
}

#[test]
fn nobody_opens_the_sofa_bed_under_someone_sitting_on_it_for_something_else() {
    let mut w = sofa_bed_only();
    w.apply(Command::Phone {
        sim: 0,
        action: browse(&w),
        contact: None,
    })
    .unwrap();
    assert!(
        until(&mut w, 1, |w| phone_seat(w, 0) == Some(Some(0))),
        "Ada sits on the sofa bed with her phone"
    );
    for id in ["doze", "pullOut"] {
        let i = interaction(&w, 0, id);
        let inter = &w.content.objects[w.objects[0].def].interactions[i];
        assert!(!w.objects[0].allows(inter, 1), "no {id} for Bea");
        let refused = w.apply(Command::Use {
            sim: 1,
            object: 0,
            interaction: i,
        });
        assert!(
            refused.is_err_and(|e| e.to_string().contains("someone is using")),
            "no {id} under the sitter"
        );
    }
    // Tired, and left to herself: Bea doesn't pull it out under Ada either.
    let e = energy(&w);
    w.sims[1].needs[e] = 0.1;
    w.apply(Command::SetAutonomy {
        enabled: true,
        household: Some(0),
    })
    .unwrap();
    let opened = until(&mut w, 15, |w| w.objects[0].open);
    assert!(!opened, "still folded");
    assert_eq!(
        phone_seat(&w, 0),
        Some(Some(0)),
        "Ada still on her phone on it"
    );
}

#[test]
fn a_pulled_out_sofa_bed_is_no_seat() {
    let mut w = sofa_bed_only();
    let pull = interaction(&w, 0, "pullOut");
    w.apply(Command::Use {
        sim: 1,
        object: 0,
        interaction: pull,
    })
    .unwrap();
    assert!(
        until(&mut w, 30, |w| w.objects[0].open
            && w.sims[1].current().is_none()),
        "pulled out"
    );
    w.apply(Command::Phone {
        sim: 0,
        action: browse(&w),
        contact: None,
    })
    .unwrap();
    assert!(
        until(&mut w, 1, |w| phone_seat(w, 0).is_some()),
        "Ada gets her phone out"
    );
    assert_eq!(phone_seat(&w, 0), Some(None), "standing, not on the bed");
}

#[test]
fn a_couple_can_still_open_a_futon_together() {
    let (mut w, _) = shipped();
    let e = energy(&w);
    w.sims[0].needs[e] = 0.4;
    w.sims[1].needs[e] = 0.4;
    let sleep = interaction(&w, FUTON, "sleep");
    for sim in 0..2 {
        w.apply(Command::Use {
            sim,
            object: FUTON,
            interaction: sleep,
        })
        .unwrap();
    }
    assert!(
        until(&mut w, 30, |w| w.objects[FUTON as usize].open
            && (0..2).all(|s| using(w, s) == Some(sleep))),
        "both unfold it and get in together"
    );
}

#[test]
fn a_nap_pulls_out_the_bed_and_the_napper_puts_it_away() {
    for (object, nap, puts_away) in [(SOFA_BED, "doze", "foldAway"), (FUTON, "nap", "foldAway")] {
        let (mut w, _) = shipped();
        let o = object as usize;
        let e = energy(&w);
        w.sims[0].needs[e] = 0.4;
        let nap = interaction(&w, object, nap);
        let puts_away = interaction(&w, object, puts_away);
        w.apply(Command::Use {
            sim: 0,
            object,
            interaction: nap,
        })
        .unwrap();
        assert!(
            until(&mut w, 30, |w| w.objects[o].open
                && using(w, 0) == Some(nap)),
            "napping on it opened"
        );
        if object == SOFA_BED {
            let relax = interaction(&w, object, "relax");
            assert!(
                w.apply(Command::Use {
                    sim: 1,
                    object,
                    interaction: relax
                })
                .is_err(),
                "no sitting on a bed"
            );
        }
        let mut put_away = false;
        let done = until(&mut w, 90, |w| {
            put_away |= using(w, 0) == Some(puts_away);
            !w.objects[o].open && w.sims[0].current().is_none()
        });
        assert!(
            done && put_away,
            "put away again after the nap, by the napper"
        );
    }
}

#[test]
fn the_last_one_up_folds_the_futon_away() {
    let (mut w, _) = shipped();
    let e = energy(&w);
    // Ada is less tired: she's up first, with Bea still asleep.
    w.sims[0].needs[e] = 0.7;
    w.sims[1].needs[e] = 0.0;
    let sleep = interaction(&w, FUTON, "sleep");
    let fold = interaction(&w, FUTON, "foldAway");
    for sim in 0..2 {
        w.apply(Command::Use {
            sim,
            object: FUTON,
            interaction: sleep,
        })
        .unwrap();
    }
    assert!(
        until(&mut w, 30, |w| w.objects[0].open),
        "it's unfolded as they go to bed"
    );
    assert!(
        until(&mut w, 30, |w| w.sims.iter().all(|s| s.asleep(&w.content))),
        "both asleep"
    );

    let mut folded_by = None;
    let mut ada_up_with_bea_asleep = false;
    let done = until(&mut w, 14 * 60, |w| {
        let asleep = |s: usize| w.sims[s].asleep(&w.content);
        assert!(
            w.objects[0].open || !(asleep(0) || asleep(1)),
            "never folded away under a sleeper"
        );
        ada_up_with_bea_asleep |= !asleep(0) && asleep(1);
        for s in 0..2 {
            if using(w, s) == Some(fold) {
                folded_by.get_or_insert(s);
            }
        }
        !w.objects[0].open && w.sims.iter().all(|s| s.current().is_none())
    });
    assert!(ada_up_with_bea_asleep, "Ada got up first");
    assert!(done, "folded away once both are up");
    assert_eq!(folded_by, Some(1), "by Bea, the last one up");
}

#[test]
fn the_washing_stays_out_as_it_was_left() {
    let (mut w, _) = shipped();
    let i = interaction(&w, WASHING_LINE, "washing");
    w.apply(Command::Use {
        sim: 0,
        object: WASHING_LINE,
        interaction: i,
    })
    .unwrap();
    assert!(
        until(&mut w, 30, |w| w.objects[2].open),
        "out once it's hung"
    );
    assert!(
        until(&mut w, 120, |w| w
            .sims
            .iter()
            .all(|s| s.current().is_none())),
        "washing hung out"
    );
    assert!(
        !until(&mut w, 120, |w| !w.objects[2].open),
        "the washing stays out"
    );
    let take_in =
        &w.content.objects[w.objects[2].def].interactions[interaction(&w, WASHING_LINE, "takeIn")];
    assert!(
        take_in.autonomous,
        "residents take the washing in themselves"
    );
}

#[test]
fn the_last_one_out_of_the_hot_tub_puts_the_cover_on() {
    for free_will in [false, true] {
        let (mut w, _) = shipped();
        w.apply(Command::SetAutonomy {
            enabled: free_will,
            household: None,
        })
        .unwrap();
        let soak = interaction(&w, HOT_TUB, "soak");
        let cover = interaction(&w, HOT_TUB, "coverOn");
        // Ada soaks a little, Bea a long while.
        let wants = |w: &World, n: &str| w.content.needs.iter().position(|d| d.id == n).unwrap();
        for n in ["comfort", "hygiene", "fun"] {
            let i = wants(&w, n);
            w.sims[0].needs[i] = 0.97;
            w.sims[1].needs[i] = 0.2;
        }
        for sim in 0..2 {
            w.apply(Command::Use {
                sim,
                object: HOT_TUB,
                interaction: soak,
            })
            .unwrap();
        }
        assert!(
            until(&mut w, 30, |w| w.objects[3].open
                && (0..2).all(|s| using(w, s) == Some(soak))),
            "both in"
        );
        let mut covered_by = None;
        let mut ada_out_first = false;
        let done = until(&mut w, 120, |w| {
            ada_out_first |= using(w, 0) != Some(soak) && using(w, 1) == Some(soak);
            assert!(
                w.objects[3].open || using(w, 1) != Some(soak),
                "never covered on Bea"
            );
            for s in 0..2 {
                if using(w, s) == Some(cover) {
                    covered_by.get_or_insert(s);
                }
            }
            !w.objects[3].open
        });
        assert!(ada_out_first, "Ada got out first");
        assert!(done, "covered once both are out (free will {free_will})");
        assert_eq!(covered_by, Some(1), "by Bea, the last one out");
    }
}

/// On their way to it, or with the order still in their queue. With `DUMP_STATES=1`, writes
/// `hot-tub-cover`: the tub open, before the orders (to give them in the browser and see the
/// story's line).
#[test]
fn told_to_cover_the_hot_tub_as_someone_gets_in_first_the_player_hears_of_it() {
    for queued in [false, true] {
        let (mut w, _) = shipped();
        let soak = interaction(&w, HOT_TUB, "soak");
        let cover = interaction(&w, HOT_TUB, "coverOn");
        w.objects[HOT_TUB as usize].open = true;
        for n in ["comfort", "hygiene", "fun"] {
            let i = w.content.needs.iter().position(|d| d.id == n).unwrap();
            w.sims[1].needs[i] = 0.2;
        }
        if !queued {
            common::dump_state(&w, "hot-tub-cover");
        }
        // Ada is told to put the cover on (nobody's in it yet: after a walk to the far corner, if
        // queued), then Bea to soak; Bea is nearer.
        if queued {
            w.apply(Command::MoveTo {
                sim: 0,
                x: 1.5,
                z: 12.5,
            })
            .unwrap();
        }
        w.apply(Command::Use {
            sim: 0,
            object: HOT_TUB,
            interaction: cover,
        })
        .unwrap();
        w.apply(Command::Use {
            sim: 1,
            object: HOT_TUB,
            interaction: soak,
        })
        .unwrap();
        let told = |w: &World| {
            w.events
                .iter()
                .filter(|e| e.kind == EventKind::NotClosed && e.a == 0)
                .count()
        };
        assert!(
            until(&mut w, 30, |w| told(w) > 0),
            "Ada's order is dropped, and the player told (queued {queued})"
        );
        assert_eq!(told(&w), 1, "once");
        let def = w.objects[HOT_TUB as usize].def as i64;
        assert_eq!(
            w.events
                .iter()
                .rev()
                .find(|e| e.kind == EventKind::NotClosed)
                .and_then(|e| e.n),
            Some(def),
            "about the tub"
        );
        assert!(
            EventKind::NotClosed.importance() >= 1,
            "it shows in the feed"
        );
        assert!(
            until(&mut w, 30, |w| using(w, 1) == Some(soak)),
            "Bea soaks"
        );
        assert!(
            w.objects[HOT_TUB as usize].open && using(&w, 0).is_none(),
            "the cover off, Ada not at it"
        );
        assert!(
            until(&mut w, 120, |w| !w.objects[HOT_TUB as usize].open),
            "Bea puts it on as she gets out"
        );
        assert_eq!(told(&w), 1);
    }
}

#[test]
fn the_snapshot_says_how_far_the_washing_is_hung() {
    let (mut w, _) = shipped();
    let i = interaction(&w, WASHING_LINE, "washing");
    w.apply(Command::Use {
        sim: 0,
        object: WASHING_LINE,
        interaction: i,
    })
    .unwrap();
    let mut buf = vec![0.0; sim_core::snapshot::CAPACITY];
    let mut seen = Vec::new();
    until(&mut w, 40, |w| {
        sim_core::snapshot::write(w, &mut buf);
        let p = buf[sim_core::snapshot::HEADER_LEN + sim_core::snapshot::sim::PROGRESS];
        if at_it(w, 0, i) {
            seen.push(p);
        } else {
            assert_eq!(p, -1.0, "not at it");
        }
        w.objects[2].open
    });
    assert!(
        seen.first().is_some_and(|&p| p < 0.05) && seen.last().is_some_and(|&p| p > 0.95),
        "0 to 1 over the use: {:?}",
        (seen.first(), seen.last())
    );
    assert!(seen.windows(2).all(|p| p[1] >= p[0]), "on and on");
}

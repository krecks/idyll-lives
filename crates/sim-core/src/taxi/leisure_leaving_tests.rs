//! Tests for leisure runs leaving.

use super::*;
use crate::world::Activity;

const CONTENT: &str = r#"{
    "needs":[{"id":"social","label":"Social","decayPerHour":0.02}],
    "objects":[{"id":"chair","name":"Chair","interactions":[{"id":"sit","label":"Sit","minutes":30,"effects":{"social":0.01},"tags":["lounge"]}]}],
    "socials":[{"id":"chat","label":"Chat","minutes":10,"tags":["social"],"acceptance":{"base":1.0},"needs":{"social":0.3}}],
    "visits":{"needs":{"social":0.5},"hours":2,"minFriendship":10,"earliestHour":0,"latestHour":24},
    "economy":{"startingFunds":500}}"#;

/// Two houses on one street, a chair in each.
const TOWN: &str = r#"{"width":40,"depth":14,"exits":[[0.5,7.5]],
    "plots":[{"name":"West","x":0,"z":0,"w":20,"d":6,"entry":[5.5,5.5]},{"name":"East","x":20,"z":0,"w":20,"d":6,"entry":[25.5,5.5]}],
    "households":[{"name":"Host","plot":0,"player":true},{"name":"Guest","plot":1}],
    "objects":[{"def":"chair","x":3,"z":2,"rot":0},{"def":"chair","x":24,"z":2,"rot":0}],
    "sims":[{"name":"Host","household":0,"x":5.5,"z":3.5},{"name":"Guest","household":1,"x":3.5,"z":3.5}]}"#;

/// The two houses, friends next door, and a park along the street.
const TOWN_PARK: &str = r#"{"width":60,"depth":14,"exits":[[0.5,7.5]],
    "plots":[{"name":"West","x":0,"z":0,"w":20,"d":6,"entry":[5.5,5.5]},{"name":"East","x":20,"z":0,"w":20,"d":6,"entry":[25.5,5.5]},
             {"name":"Park","x":40,"z":0,"w":20,"d":6,"entry":[45.5,5.5],"public":true}],
    "households":[{"name":"Host","plot":0,"player":true},{"name":"Guest","plot":1}],
    "objects":[{"def":"chair","x":3,"z":2,"rot":0},{"def":"chair","x":24,"z":2,"rot":0}],
    "sims":[{"name":"Host","household":0,"x":5.5,"z":3.5},{"name":"Guest","household":1,"x":25.5,"z":3.5}],
    "relationships":[{"a":0,"b":1,"preset":"friends"}]}"#;

#[test]
fn one_waiting_at_home_for_a_taxi_out_doesnt_go_off_visiting_meanwhile() {
    // The guest, at home, is booked on a taxi out to the park an hour and a half from now,
    // and wants company: they wait at home for it, not go round to their friend's next
    // door meanwhile. (Gone, the taxi's time came while they were there, and it took them
    // away twenty minutes in, Ottertarn.)
    let content = CONTENT.replace(
        r#""economy""#,
        r#""bondPresets":{"friends":{"friendship":80}},"economy""#,
    );
    let mut w = World::from_json(&content, TOWN_PARK, 1).unwrap();
    let i = 1usize;
    let tick = w.tick;
    let out = tick + 90 * TICKS_PER_MINUTE;
    w.taxis.runs.push(Run {
        id: 9,
        home: false,
        place: Some(2),
        directed: false,
        group: (0, 0),
        enter: out - 30 * TICKS_PER_MINUTE,
        due_out: 0,
        planned_out: 0,
        depart: 0,
        stops: vec![Stop {
            sim: i as u32,
            plot: 1,
            due: out,
            leave: out - 10 * TICKS_PER_MINUTE,
            state: StopState::Booked,
            halt: 0,
        }],
        next: 0,
        progress: 0.0,
        detour: None,
        since: 0,
        car: None,
        at: Vec::new(),
        kerbs: Vec::new(),
        laid: None,
    });
    w.sims[i].taxi = Some(Booking { run: 9, stop: 0 });
    for _ in 0..60 * TICKS_PER_MINUTE {
        w.sims[i].needs[0] = 0.1;
        w.tick_once();
        assert!(
            !w.sims[i].visiting.is_some_and(|v| v.plot == 0),
            "off to their friend's while waiting for the taxi out"
        );
    }
}

#[test]
fn a_guest_off_to_the_taxi_home_mid_talk_from_a_seat_lets_go_of_the_seat() {
    // The guest, at West's, sat on its chair talking from it (what they sat down to do
    // paused), is due out to their kerb for the taxi home: off they go, the talk over and
    // the chair free. (Ending the talk took them back to the chair, and that was dropped
    // with the chair still theirs: "object 0 is held by Guest who isn't using it", Crown
    // Quarter.)
    let mut w = World::from_json(CONTENT, TOWN, 1).unwrap();
    let (i, chair) = (1usize, 0u32);
    let tick = w.tick;
    w.sims[i].visiting = Some(crate::life::Visit {
        plot: 0,
        until: tick + crate::mins(30.0),
        idle: 0,
        farewell: 0,
        offer_at: 0,
        arrived: tick,
        leaving: false,
        home_by: false,
    });
    assert!(w.objects[chair as usize].reserve(i as u32, 1));
    let task = |kind| Task {
        kind,
        directed: false,
        routine: false,
    };
    w.sims[i].paused = Some(Activity {
        task: task(TaskKind::Use {
            object: chair,
            interaction: 0,
        }),
        phase: Phase::Using { elapsed: 0.0 },
        tags: Default::default(),
    });
    w.sims[i].current = Some(Activity {
        task: task(TaskKind::Social {
            target: 0,
            social: 0,
        }),
        phase: Phase::Waiting { ticks: 0 },
        tags: Default::default(),
    });
    // Booked on the taxi home from West's, time to be out at the kerb for it.
    w.taxis.runs.push(Run {
        id: 7,
        home: true,
        place: Some(0),
        directed: false,
        group: (0, 0),
        enter: tick + crate::mins(50.0),
        due_out: 0,
        planned_out: 0,
        depart: 0,
        stops: vec![Stop {
            sim: i as u32,
            plot: 1,
            due: tick + crate::mins(0.5),
            leave: tick,
            state: StopState::Booked,
            halt: 0,
        }],
        next: 0,
        progress: 0.0,
        detour: None,
        since: 0,
        car: None,
        at: Vec::new(),
        kerbs: Vec::new(),
        laid: None,
    });
    w.sims[i].taxi = Some(Booking { run: 7, stop: 0 });
    leisure_leaving(&mut w);
    let s = &w.sims[i];
    assert!(
        s.queue
            .front()
            .is_some_and(|t| matches!(t.kind, TaskKind::GoHome)),
        "not off home"
    );
    assert!(s.current.is_none() && s.paused.is_none(), "still at it");
    assert_eq!(
        w.objects[chair as usize].users().count(),
        0,
        "the chair still held"
    );
}

#[test]
fn one_asked_over_gives_up_the_taxi_booked_out_to_a_place_of_their_own() {
    // The guest, at home, is booked on a taxi out to the park in half an hour, an outing of
    // their own; their friend next door phones to ask them over, and they say yes: they go
    // round to the friend's, the taxi out given up. (Kept, it came for them all the same, took
    // them to the place, and they were off to the friend's three minutes after getting
    // there, Alder Mill.)
    let content = CONTENT.replace(
        r#""economy""#,
        r#""bondPresets":{"friends":{"friendship":80}},"animations":["phone"],
        "phone":{"anim":"phone","seatRange":0,"actions":[
          {"id":"invite","label":"Invite over","minutes":5,"contact":true,"meet":"here","tags":["social"],
           "acceptance":{"base":1.0},"needs":{"social":0.05},"success":{"friendship":1,"targetFriendship":1}}]},
        "economy""#,
    );
    let mut w = World::from_json(&content, TOWN_PARK, 1).unwrap();
    w.autonomy = false;
    let (host, i) = (0usize, 1usize);
    let tick = w.tick;
    let out = tick + 30 * TICKS_PER_MINUTE;
    w.taxis.runs.push(Run {
        id: 9,
        home: false,
        place: Some(2),
        directed: false,
        group: (0, 0),
        enter: out - 20 * TICKS_PER_MINUTE,
        due_out: 0,
        planned_out: 0,
        depart: 0,
        stops: vec![Stop {
            sim: i as u32,
            plot: 1,
            due: out,
            leave: out - 10 * TICKS_PER_MINUTE,
            state: StopState::Booked,
            halt: 0,
        }],
        next: 0,
        progress: 0.0,
        detour: None,
        since: 0,
        car: None,
        at: Vec::new(),
        kerbs: Vec::new(),
        laid: None,
    });
    w.sims[i].taxi = Some(Booking { run: 9, stop: 0 });
    let invite = w.content.phone.action_index("invite").expect("invite");
    w.apply(crate::Command::Phone {
        sim: host as u32,
        action: invite,
        contact: Some(i as u32),
    })
    .unwrap();
    let mut asked = false;
    for _ in 0..20 * TICKS_PER_MINUTE {
        w.tick_once();
        asked |= w.sims[i]
            .queue
            .iter()
            .chain(w.sims[i].current.as_ref().map(|a| &a.task))
            .any(|t| matches!(t.kind, TaskKind::Visit { plot: 0 }));
    }
    assert!(asked, "never asked over");
    assert!(w.sims[i].taxi.is_none(), "still booked on the taxi out");
    assert_eq!(w.taxis.runs[0].stops[0].state, StopState::Done);
}

/// A leisure rider the taxi goes on without is booked again once (`Taxis::left`), as the
/// caller tells `release` (`left`), whether their stop is marked done before or after; one
/// let go otherwise isn't.
#[test]
fn a_rider_left_at_the_kerb_is_booked_again_whatever_the_stop_says() {
    let content = CONTENT.replace(
        r#""economy""#,
        r#""bondPresets":{"friends":{"friendship":80}},"economy""#,
    );
    for (done_first, left) in [(true, true), (false, true), (true, false), (false, false)] {
        let mut w = World::from_json(&content, TOWN_PARK, 1).unwrap();
        let i = 1usize;
        let tick = w.tick;
        let run: Run = serde_json::from_value(serde_json::json!({"id":5,"place":2,
            "group":[0,0],"enter":tick + 1000,"dueOut":0,"depart":0,
            "stops":[{"sim":i,"plot":1,"due":tick + 100,"leave":tick,"state":"booked"}]}))
        .unwrap();
        w.taxis.runs.push(run);
        w.sims[i].taxi = Some(Booking { run: 5, stop: 0 });
        if done_first {
            w.taxis.runs[0].stops[0].state = StopState::Done;
        }
        release(&mut w, i, left);
        if !done_first {
            w.taxis.runs[0].stops[0].state = StopState::Done;
        }
        assert_eq!(
            w.taxis.left.contains(&(i as u32, 2)),
            left,
            "left {left}, the stop done first {done_first}"
        );
        assert_eq!(w.taxis.runs[0].stops[0].state, StopState::Done);
    }
}

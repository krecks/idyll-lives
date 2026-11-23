//! Tests for the traffic model.

use super::*;

/// A car's body (4.6 by 1.88 m) at `at` facing `yaw`.
fn rect(at: [f32; 2], yaw: f32) -> Rect {
    Rect {
        at,
        yaw,
        half: [2.3, 0.94],
    }
}

/// A claim from 10 to 20 ticks along its route through `points` (each facing on to the next).
fn claim(points: &[[f32; 2]]) -> Claim {
    let n = points.len();
    let samples = (0..n)
        .map(|k| {
            let (a, b) = if k + 1 < n {
                (points[k], points[k + 1])
            } else {
                (points[k - 1], points[k])
            };
            let yaw = (b[0] - a[0]).atan2(b[1] - a[1]);
            Sample {
                t: 10.0 + 10.0 * k as f32 / (n - 1) as f32,
                d: k as f32,
                rect: rect(points[k], yaw),
                way: [yaw.sin(), yaw.cos()],
                pass: 0.0,
                layer: crate::travel::Layer::GROUND,
            }
        })
        .collect();
    Claim::new(10.0, 20.0, samples)
}

/// One standing at `at` facing `yaw`, waiting for nobody on the road.
fn standing(key: u64, at: [f32; 2], yaw: f32) -> Mover {
    Mover {
        key,
        rect: rect(at, yaw),
        way: [yaw.sin(), yaw.cos()],
        pass: 0.0,
        moving: false,
        ahead: Vec::new(),
        parked: false,
        waits: None,
        layer: crate::travel::Layer::GROUND,
    }
}

/// A lit junction at `at` with arms out along `ways`, each of phase `phases[k]`.
fn lit(at: [f32; 2], ways: &[[f32; 2]], phases: &[u8]) -> Signal {
    Signal {
        at,
        arms: ways
            .iter()
            .zip(phases)
            .map(|(&way, &phase)| SignalArm {
                way,
                pole: at,
                yaw: 0.0,
                phase,
                road: 0,
                kerb: 2.15,
                walk_from: 6.0,
                walk_to: 9.0,
                stop: 9.5,
                crosswalk: true,
                span: [
                    [at[0] + way[0] * 6.0, at[1] + way[1] * 6.0],
                    [
                        at[0] + way[0] * (9.5 + KEEP_OFF),
                        at[1] + way[1] * (9.5 + KEEP_OFF),
                    ],
                ],
            })
            .collect(),
        phases: phases.iter().max().map_or(0, |p| p + 1),
        roundabout: false,
        walk_clear: crate::secs(4.3),
        lit: true,
        rest: 0,
    }
}

#[test]
fn a_side_roads_crosswalk_is_walked_on_the_main_roads_green_never_stopping_it() {
    // A lit T: the main road along x (phase 0, which it rests on), a side road from z (phase
    // 1). Someone waiting at the side road's crosswalk waits for the main road's green, not the
    // walk: resting there, it stays green and shows them walk; at the side road's green, it
    // comes back round to the main road's, never the walk (every arm red). And the side road's
    // green doesn't begin while someone's on its crosswalk.
    let sig = lit(
        [0.0, 0.0],
        &[[-1.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
        &[0, 0, 1],
    );
    assert_eq!(
        sig.opens_with(2),
        Some(0),
        "the side road's on the main road's green"
    );
    assert_eq!(sig.opens_with(0), None, "the main road's on the walk");
    assert_eq!(sig.wants(2), Some(0));
    assert_eq!(sig.wants(0), sig.walk_phase());
    let signals = [sig];
    let mut t = Traffic::default();
    let walk = signals[0].walk_phase().unwrap();
    // (Resting on the main road's green, someone at the side road's kerb: on as it is.)
    t.walkers = vec![(0, signals[0].wants(2).unwrap())];
    for tick in 1..crate::secs(60.0) {
        t.change_lights(tick, &signals);
        assert_eq!(t.lights[0].phase, 0, "off the main road's green for them");
    }
    assert_eq!(t.light(&signals, 0, 2), Some(Head::Walk));
    assert_eq!(t.light(&signals, 0, 0), Some(Head::Green));
    assert!(walk_open(&signals, &t.lights, 0, 2));
    assert!(!walk_open(&signals, &t.lights, 0, 0));
    // (The side road's green: back to the main road's for them, never the walk.)
    let start = crate::secs(60.0);
    t.lights[0] = Light {
        phase: 1,
        stage: Stage::Green,
        since: start,
    };
    let mut seen = std::collections::BTreeSet::new();
    let mut tick = start;
    while !(t.lights[0].phase == 0 && t.lights[0].stage == Stage::Green) {
        tick += 1;
        assert!(
            tick < start + CYCLE * 2,
            "never back on the main road's green"
        );
        t.change_lights(tick, &signals);
        seen.insert(t.lights[0].phase);
        assert!(!walk_open(&signals, &t.lights, 0, 2) || t.lights[0].phase == 0);
    }
    assert!(!seen.contains(&walk), "the walk for them: {seen:?}");
    // (Someone on the side road's crosswalk as the main road's green ends for the side road,
    // wanted there (as by a car at its red): all red till they're off it.)
    t.crossing = vec![(0, 2)];
    t.walkers = vec![(0, 1)];
    let from = tick;
    loop {
        tick += 1;
        assert!(tick < from + CYCLE * 2, "the side road's green never came");
        t.change_lights(tick, &signals);
        let l = t.lights[0];
        if l.stage == Stage::Clear && tick - l.since == ALL_RED + 10 {
            t.crossing.clear();
        }
        if l.phase == 1 && l.stage == Stage::Green {
            assert!(
                t.crossing.is_empty(),
                "the side road's green with someone on its crosswalk"
            );
            break;
        }
    }
}

#[test]
fn a_crosswalk_and_the_road_kept_off_by_it_follow_its_road_not_the_way_out_from_the_middle() {
    // A junction's east arm, its road leaving the middle 20° off the way it goes on (a road
    // bending away, a middle merged off it): its crosswalk and the road kept off by its stop
    // line are where its road is, not along the way out from the middle (by then metres off
    // it, over the pavement, where it shut a house's gate in).
    let mut sig = lit(
        [0.0, 0.0],
        &[[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0]],
        &[0, 1, 0],
    );
    let (s, c) = 20f32.to_radians().sin_cos();
    sig.arms[0].way = [c, s];
    sig.arms[0].span = [[6.0, 0.0], [9.5 + KEEP_OFF, 0.0]];
    let kerb = sig.arms[0].kerb;
    assert!(sig.crossing([7.5, -1.0]), "on its zebra");
    assert!(!sig.crossing([7.5, kerb + 0.5]), "on the pavement by it");
    assert!(sig.off_limits([11.0, 0.0]), "on the road by the stop line");
    assert!(!sig.off_limits([11.0, kerb + 1.0]), "on the pavement by it");
    assert!(sig.off_limits([2.0, 0.0]), "in the box");
}

#[test]
fn a_group_goes_by_the_lights_of_the_junction_it_drives_through_not_the_first_near() {
    // Two lit junctions near each other: the first listed 12 m off the way, the second on it.
    // A group coming down through the second from the north takes its lights, on the phase
    // of the arm it comes in by (its north arm's, across from none: a phase of its own).
    let near = lit(
        [8.0, -6.0],
        &[[1.0, 0.0], [-1.0, 0.0], [0.0, 1.0]],
        &[0, 0, 1],
    );
    let through = lit(
        [20.0, 0.0],
        &[[-1.0, 0.0], [1.0, 0.0], [0.0, -1.0]],
        &[0, 0, 1],
    );
    let mut claims = vec![claim(&[
        [20.0, -12.0],
        [20.0, -6.0],
        [20.0, 0.0],
        [20.0, 6.0],
        [20.0, 12.0],
    ])];
    gate(&mut claims, &[near, through], &Trace::new(), 4.6);
    assert_eq!(claims[0].gate, Some((1, 1)));
}

#[test]
fn a_group_through_lights_begins_with_the_nose_at_its_arms_stop_line() {
    // Coming down the north arm (its stop line 9.5 m out) of a junction at the origin, its
    // body first touching the box (6 m round) with its middle about 8 m out: it waits further
    // back, its nose at the stop line, its middle 11.8 m out.
    let sig = lit(
        [0.0, 0.0],
        &[[0.0, -1.0], [0.0, 1.0], [1.0, 0.0]],
        &[0, 0, 1],
    );
    let trace: Trace = (0..=60)
        .map(|k| {
            let z = -30.0 + k as f32;
            (k as f32, [-1.05, z], 0.0, false, k as f32)
        })
        .collect();
    let mut claims = claims(&trace, &[], 4.6, 1.9, &[([0.0, 0.0], 6.0)], &[], &[]);
    let before = claims[0].from;
    gate(&mut claims, std::slice::from_ref(&sig), &trace, 4.6);
    assert_eq!(claims[0].gate, Some((0, 0)));
    let middle = |t: f32| trace[t as usize].1[1];
    assert!(
        middle(before) >= -9.0,
        "it touched the box at {}",
        middle(before)
    );
    let nose = middle(claims[0].from) + 2.3;
    assert!((-10.5..=-9.5).contains(&nose), "its nose at {nose}");
}

#[test]
fn an_old_ask_waiting_its_turn_after_one_held_up_by_a_later_one_lets_that_one_go() {
    // (Mirrormere's: the first, its way across where the third stands; the second, asked a
    // tick after it, its way across the first's; the third, asked long after, its way
    // across the second's only. Each waited for the next for good: the third for the second,
    // an old ask, the second its turn after the first, the first for the third's body.)
    let (first, second, third) = (1 << 63 | 11, 1 << 63 | 18, 1 << 63 | 9);
    let c_first = claim(&[
        [-12.0, 0.0],
        [-6.0, 0.0],
        [0.0, 0.0],
        [6.0, 0.0],
        [12.0, 0.0],
    ]);
    let c_second = claim(&[
        [6.0, -6.0],
        [6.0, 0.0],
        [6.0, 6.0],
        [6.0, 12.0],
        [6.0, 18.0],
    ]);
    let c_third = claim(&[
        [0.0, 6.0],
        [0.0, 9.0],
        [2.0, 12.0],
        [6.0, 14.0],
        [12.0, 14.0],
    ]);
    let movers = || {
        vec![
            standing(first, [-15.0, 0.0], std::f32::consts::FRAC_PI_2),
            standing(second, [6.0, -9.0], 0.0),
            standing(third, [0.0, 2.0], 0.0),
        ]
    };
    let places = [(first, 0, 9.5), (second, 0, 9.5), (third, 0, 9.5)];
    let mut traffic = Traffic::default();
    let late = 2 + OLD_ASK + 10;
    for tick in 1..late + 20 {
        traffic.may_go(tick, first, 0, std::slice::from_ref(&c_first), 9.5, 10.5);
        if tick >= 2 {
            traffic.may_go(tick, second, 0, std::slice::from_ref(&c_second), 9.5, 10.5);
        }
        if tick >= late && !traffic.holding(third) {
            traffic.may_go(tick, third, 0, std::slice::from_ref(&c_third), 9.5, 10.5);
        }
        traffic.begin(tick + 1, movers(), &places, &[]);
    }
    assert!(
        traffic.waits_for(first).contains(&third),
        "the first waits for the third's body"
    );
    assert!(
        traffic.holding(third),
        "the third goes first: waits {:?}",
        traffic.waits_for(third)
    );
}

#[test]
fn two_at_the_lights_each_in_the_others_way_wait_for_each_other_on_red_and_green() {
    // (Highholm's: a car and a taxi at their stop lines on two phases of a lit junction, each
    // standing in the other's way through it. On its green each waited for the other's body;
    // the other, at its red, named nobody: no ring for the waits to see, and they stood by
    // turns for good.) At red too each names whom it would wait for: the ring is seen.
    let (a, b) = (1 << 63 | 5, 1 << 63 | 22);
    let mut c_a = claim(&[
        [-6.0, 0.0],
        [-2.0, 0.0],
        [2.0, 0.0],
        [6.0, 0.0],
        [10.0, 0.0],
    ]);
    let mut c_b = claim(&[
        [6.0, 0.0],
        [2.0, 0.0],
        [-2.0, 0.0],
        [-6.0, 0.0],
        [-10.0, 0.0],
    ]);
    c_a.gate = Some((0, 0));
    c_b.gate = Some((0, 1));
    let sig = lit(
        [0.0, 0.0],
        &[[-1.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
        &[0, 1, 1],
    );
    let movers = || {
        vec![
            standing(a, [-9.0, 0.0], std::f32::consts::FRAC_PI_2),
            standing(b, [9.0, 0.0], -std::f32::consts::FRAC_PI_2),
        ]
    };
    let places = [(a, 0, 9.5), (b, 0, 9.5)];
    let mut traffic = Traffic::default();
    let mut both = false;
    for tick in 1..CYCLE * 3 {
        traffic.may_go(tick, a, 0, std::slice::from_ref(&c_a), 9.5, 10.5);
        traffic.may_go(tick, b, 0, std::slice::from_ref(&c_b), 9.5, 10.5);
        traffic.begin(tick + 1, movers(), &places, std::slice::from_ref(&sig));
        assert!(
            !traffic.holding(a) && !traffic.holding(b),
            "one went into the other"
        );
        both |= traffic.waits_for(a).contains(&b) && traffic.waits_for(b).contains(&a);
    }
    assert!(
        both,
        "they never both waited for each other: a ring nobody sees"
    );
}

#[test]
fn two_through_two_close_lit_junctions_each_way_both_get_their_greens() {
    // (Hollyglade's: two lit junctions too near to wait between. One waits at the first's red
    // to go through it on phase 1 and the second on phase 0; another the other way round.
    // Each light held its phase 0 for the one waiting through it from the other, and the two
    // went round in step for good: neither ever had both its greens.) One of the two leads,
    // the other follows it: each gets through on both its greens.
    let (x, y) = (1 << 63 | 3, 1 << 63 | 29);
    let mut c_x = claim(&[[-6.0, 50.0], [-2.0, 50.0], [2.0, 50.0], [6.0, 50.0]]);
    let mut c_y = claim(&[[-6.0, -50.0], [-2.0, -50.0], [2.0, -50.0], [6.0, -50.0]]);
    c_x.gate = Some((0, 1));
    c_x.gate2 = Some((1, 0));
    c_y.gate = Some((1, 1));
    c_y.gate2 = Some((0, 0));
    let ways = [[-1.0, 0.0], [1.0, 0.0], [0.0, 1.0]];
    let sigs = [
        lit([0.0, 0.0], &ways, &[0, 0, 1]),
        lit([20.0, 0.0], &ways, &[0, 0, 1]),
    ];
    let movers = || {
        vec![
            standing(x, [-9.0, 50.0], std::f32::consts::FRAC_PI_2),
            standing(y, [-9.0, -50.0], std::f32::consts::FRAC_PI_2),
        ]
    };
    let places = [(x, 0, 9.5), (y, 0, 9.5)];
    let mut traffic = Traffic::default();
    let (mut went_x, mut went_y) = (None, None);
    for tick in 1..CYCLE * 6 {
        if went_x.is_none() {
            traffic.may_go(tick, x, 0, std::slice::from_ref(&c_x), 9.5, 10.5);
        }
        if went_y.is_none() {
            traffic.may_go(tick, y, 0, std::slice::from_ref(&c_y), 9.5, 10.5);
        }
        traffic.begin(tick + 1, movers(), &places, &sigs);
        if went_x.is_none() && traffic.holding(x) {
            went_x = Some(tick);
        }
        if went_y.is_none() && traffic.holding(y) {
            went_y = Some(tick);
        }
    }
    assert!(
        went_x.is_some_and(|t| t < 2 * CYCLE) && went_y.is_some_and(|t| t < 2 * CYCLE),
        "through both lights: {went_x:?}, {went_y:?}"
    );
}

#[test]
fn one_whose_claims_are_worked_out_again_in_the_middle_of_a_group_holds_on_to_all_of_it() {
    // (Willowley's: a car through two lit junctions joined as one, the second's box a part of
    // its group still to come, had its claims worked out again in the first's box (its way
    // round a taxi standing at the kerb, `cars::pass_stands`): let go of all but the part it
    // was in, the rest was granted to one coming the other way and turning across it, and the
    // two stood nose to nose in the box for good.) Its group is held on to, all of it: the
    // other waits.
    let (a, b) = (1 << 63 | 21, 1 << 63 | 19);
    let first = claim(&[[-8.0, 0.0], [-4.0, 0.0], [0.0, 0.0]]);
    let mut second = claim(&[[0.0, 0.0], [4.0, 0.0], [8.0, 0.0], [12.0, 0.0]]);
    second.from += 10.0;
    second.to += 10.0;
    for s in &mut second.samples {
        s.t += 10.0;
    }
    let claims = [first, second];
    let across = claim(&[[6.0, 8.0], [6.0, 4.0], [6.0, 0.0], [6.0, -4.0], [6.0, -8.0]]);
    let mut traffic = Traffic::default();
    traffic.reclaimed(a, 0, &claims, 15.0);
    assert!(
        traffic.holds(a, tag(0, 0)) && traffic.holds(a, tag(0, 1)),
        "the group it's in let go"
    );
    let movers = || {
        vec![
            standing(a, [-4.0, 0.0], std::f32::consts::FRAC_PI_2),
            standing(b, [6.0, 11.0], std::f32::consts::PI),
        ]
    };
    let places = [(a, 0, 15.0), (b, 0, 10.0)];
    for tick in 1..4 {
        traffic.may_go(tick, b, 0, std::slice::from_ref(&across), 10.0, 11.0);
        traffic.begin(tick + 1, movers(), &places, &[]);
    }
    assert!(!traffic.holding(b), "granted across the rest of its group");
    // (Out of any part, nothing's held: it asks again as it goes on.)
    traffic.reclaimed(a, 0, &claims, 5.0);
    assert!(!traffic.holding(a));
}

#[test]
fn one_past_its_stop_line_asking_again_in_the_box_goes_on_at_red() {
    // (Elderbrook's: a taxi stood for its rider in a lit junction's box, letting go of all of
    // its group but the part it stood in; asking for the rest again it waited in the box for
    // its arm's green, while the light was held on the other phase for one waiting for the
    // box to clear of it.) Its stop line behind it, that light is too: it goes on, and it
    // calls no green.
    let k = 1 << 63 | 7;
    let mut c = claim(&[[0.0, -2.0], [0.0, 2.0], [0.0, 6.0], [0.0, 10.0]]);
    c.gate = Some((0, 0));
    c.stop_at = 4.0;
    let sig = lit(
        [0.0, 0.0],
        &[[0.0, -1.0], [0.0, 1.0], [1.0, 0.0]],
        &[0, 0, 1],
    );
    assert_eq!(c.lights(4.0), (Some((0, 0)), None), "at its stop line");
    assert_eq!(c.lights(9.5), (None, None), "past it");
    let mut traffic = Traffic {
        lights: vec![Light {
            phase: 1,
            stage: Stage::Green,
            since: 0,
        }],
        ..Default::default()
    };
    let movers = || vec![standing(k, [0.0, -2.5], 0.0)];
    let places = [(k, 0, 9.5)];
    for tick in 1..4 {
        traffic.may_go(tick, k, 0, std::slice::from_ref(&c), 9.5, 10.5);
        traffic.begin(tick + 1, movers(), &places, std::slice::from_ref(&sig));
    }
    assert!(traffic.holding(k), "it waited in the box at red");
    assert_eq!(traffic.lights[0].phase, 1, "it called its own green");
}

#[test]
fn one_coming_up_to_wait_in_the_way_of_an_old_ask_stops_short() {
    // (Crown Quarter's: a taxi waiting to turn across a junction's mouth, where those coming
    // up the other arm stand at its edge, each in its way and let go first as it asked: one
    // after another kept it waiting an hour. Asked long ago, the next stops short.)
    let (waiting, coming) = (1 << 63 | 40, 1 << 63 | 18);
    let across = claim(&[
        [-12.0, 0.0],
        [-6.0, 0.0],
        [0.0, 0.0],
        [6.0, 0.0],
        [12.0, 0.0],
    ]);
    let mine = claim(&[
        [0.0, -2.0],
        [0.0, 2.0],
        [0.0, 6.0],
        [0.0, 10.0],
        [0.0, 14.0],
    ]);
    // (Held up meanwhile by one standing in its way further on.)
    let right = std::f32::consts::FRAC_PI_2;
    let movers = || {
        vec![
            standing(waiting, [-15.0, 0.0], right),
            standing(1 << 63 | 7, [10.0, 0.0], right),
        ]
    };
    let mut traffic = Traffic::default();
    let mut stops = Vec::new();
    for tick in 1..OLD_ASK + 20 {
        traffic.may_go(tick, waiting, 0, std::slice::from_ref(&across), 9.5, 10.5);
        traffic.begin(tick + 1, movers(), &[(waiting, 0, 9.5)], &[]);
        // (Coming up, 3.4 m short of where it would wait, a tick on 2.9 m.)
        assert!(traffic.may_go(tick + 1, coming, 0, std::slice::from_ref(&mine), 5.0, 6.0));
        let (now, next) = (rect([0.0, -5.4], 0.0), rect([0.0, -4.9], 0.0));
        stops.push(traffic.blocked(
            coming,
            &now,
            &next,
            [0.0, 1.0],
            0.7,
            false,
            crate::travel::Layer::GROUND,
        ));
    }
    assert_eq!(stops[0], None, "a young ask: on up to it");
    assert_eq!(
        stops.last().copied().flatten(),
        Some(waiting),
        "an old one: it stops short"
    );
    // (Nearer than half that already, it came up before the ask was old: on.)
    let (now, next) = (rect([0.0, -3.4], 0.0), rect([0.0, -2.9], 0.0));
    assert_eq!(
        traffic.blocked(
            coming,
            &now,
            &next,
            [0.0, 1.0],
            0.7,
            false,
            crate::travel::Layer::GROUND
        ),
        None
    );
}

#[test]
fn one_isnt_let_into_a_stretch_whose_far_end_one_turning_round_still_backs_into() {
    // (Mirrorshore's: a taxi let into a narrow bend while a car turning round backed into its
    // far end, slowly, those on bikes about it; there the two came nose to nose, the taxi
    // waiting for the car's body and the car for the taxi's claim to drive on out, for good.)
    let (coming, turning) = (1 << 63 | 69, 1 << 63 | 15);
    let stretch = claim(&[
        [0.0, -12.0],
        [0.0, -6.0],
        [0.0, 0.0],
        [0.0, 6.0],
        [0.0, 12.0],
    ]);
    // (Facing +z, backing towards -z: as yet well clear of the stretch, standing a while.)
    let back = |z: f32| rect([0.0, z], 0.0);
    let mover = Mover {
        way: [0.0, -1.0],
        ..standing(turning, [0.0, 25.0], 0.0)
    };
    let mut traffic = Traffic {
        turning: vec![(
            turning,
            (0..=20).map(|k| back(25.0 - k as f32 * 0.5)).collect(),
        )],
        ..Default::default()
    };
    traffic.may_go(1, coming, 0, std::slice::from_ref(&stretch), 9.5, 10.5);
    traffic.begin(
        2,
        vec![standing(coming, [0.0, -15.0], 0.0), mover],
        &[(coming, 0, 9.5)],
        &[],
    );
    assert!(!traffic.holding(coming), "it waits short of the stretch");
    assert!(traffic.waits_for(coming).contains(&turning));
    // (Turned round, gone on, the ground clear: it's let in.)
    let mut traffic = Traffic::default();
    traffic.may_go(1, coming, 0, std::slice::from_ref(&stretch), 9.5, 10.5);
    traffic.begin(
        2,
        vec![
            standing(coming, [0.0, -15.0], 0.0),
            standing(turning, [0.0, 25.0], 0.0),
        ],
        &[(coming, 0, 9.5)],
        &[],
    );
    assert!(traffic.holding(coming));
}

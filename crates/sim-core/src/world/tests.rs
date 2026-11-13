use super::*;

const CONTENT: &str = r#"{
        "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.1},
                 {"id":"energy","label":"Energy","decayPerHour":0.05}],
        "objects":[
          {"id":"fridge","name":"Fridge","interactions":[
            {"id":"snack","label":"Snack","minutes":10,"effects":{"hunger":0.4}}]},
          {"id":"bed","name":"Bed","footprint":[2,2],"interactions":[
            {"id":"sleep","label":"Sleep","minutes":480,"pose":"lie","effects":{"energy":1.0}}]}]}"#;

const LOT: &str = r#"{"width":12,"depth":12,
        "walls":[[6,0,6,10]],
        "objects":[{"def":"fridge","x":1,"z":1,"rot":0},{"def":"bed","x":9,"z":2,"rot":3}],
        "sims":[{"name":"Ada","x":3.5,"z":3.5}]}"#;

fn world() -> World {
    World::from_json(CONTENT, LOT, 7).unwrap()
}

fn run_ticks(w: &mut World, n: u64) {
    for _ in 0..n {
        w.tick_once();
    }
}

#[test]
fn a_cook_on_the_way_to_the_first_station_can_be_called_but_not_once_started() {
    let content = Content::from_json(CONTENT).unwrap();
    let mut w = world();
    let cook = |step, servings, directed| Activity {
        task: Task {
            kind: TaskKind::Cook {
                recipe: 0,
                step,
                station: None,
                origin: None,
                servings,
                skipped: 0,
                minutes: 0,
            },
            directed,
            routine: false,
        },
        phase: Phase::Routing {
            waypoints: Vec::new(),
            next: 0,
        },
        tags: 0,
    };
    let s = &mut w.sims[0];
    s.current = Some(cook(0, 0, false));
    assert!(s.available_for_social(&content), "on the way to the fridge");
    s.current = Some(cook(1, 2, false));
    assert!(!s.available_for_social(&content), "ingredients bought");
    s.current = Some(cook(0, 0, true));
    assert!(!s.available_for_social(&content), "the player's order");
}

#[test]
fn objects_block_tiles_and_overlaps_fail() {
    let mut w = world();
    assert!(w.blocked()[w.lot.tile_index(1, 1)]);
    assert!(w.place_object("fridge", 1, 1, 0).is_err());
}

#[test]
fn hungry_sim_eats_autonomously() {
    let mut w = world();
    w.sims[0].needs[0] = 0.05;
    w.sims[0].needs[1] = 1.0;
    run_ticks(&mut w, crate::mins(60.0));
    assert!(
        w.sims[0].needs[0] > 0.3,
        "hunger was {}",
        w.sims[0].needs[0]
    );
}

#[test]
fn directed_command_walks_around_wall_and_lies_down() {
    let mut w = world();
    w.sims[0].needs[1] = 0.1;
    w.apply(Command::Use {
        sim: 0,
        object: 1,
        interaction: 0,
    })
    .unwrap();
    run_ticks(&mut w, crate::mins(30.0));
    let s = &w.sims[0];
    assert_eq!(s.pose, Pose::Lie);
    assert_eq!(w.objects[1].users().next(), Some(0));
    assert!(s.needs[1] > 0.1);
}

#[test]
fn cancel_releases_object() {
    let mut w = world();
    w.apply(Command::Use {
        sim: 0,
        object: 1,
        interaction: 0,
    })
    .unwrap();
    run_ticks(&mut w, 2);
    assert_eq!(w.objects[1].users().next(), Some(0));
    w.apply(Command::Cancel { sim: 0, index: 0 }).unwrap();
    assert!(!w.objects[1].in_use());
    assert_eq!(w.sims[0].pose, Pose::Stand);
}

#[test]
fn watching_tv_sits_on_a_nearby_sofa_and_broadcasts() {
    const TV: &str = r#"{
            "needs":[{"id":"fun","label":"Fun","decayPerHour":0.1},
                     {"id":"hunger","label":"Hunger","decayPerHour":0.1}],
            "objects":[
              {"id":"tv","name":"TV","interactions":[
                {"id":"watch","label":"Watch TV","minutes":60,"seatNear":5,"broadcast":0.5,
                 "broadcastRange":6,"effects":{"fun":0.6},"tags":["screen"]}]},
              {"id":"sofa","name":"Sofa","interactions":[
                {"id":"relax","label":"Relax","minutes":30,"pose":"sit",
                 "effects":{"fun":0.1},"tags":["lounge"]}]}]}"#;
    const TV_LOT: &str = r#"{"width":12,"depth":12,
            "objects":[{"def":"tv","x":2,"z":2,"rot":0},{"def":"sofa","x":5,"z":2,"rot":0}],
            "sims":[{"name":"Ada","x":7.5,"z":7.5},{"name":"Bo","x":3.5,"z":3.5}]}"#;
    let mut w = World::from_json(TV, TV_LOT, 7).unwrap();
    w.apply(Command::Use {
        sim: 0,
        object: 0,
        interaction: 0,
    })
    .unwrap();
    run_ticks(&mut w, crate::secs(30.0));
    assert_eq!(w.sims[0].use_seat, Some(1), "sat on the sofa");
    assert_eq!(w.sims[0].pose, Pose::Sit);
    assert_eq!(w.objects[0].users().next(), Some(0));
    // Bo, in the room doing something else, hears it too.
    w.sims[1].needs[0] = 0.5;
    w.broadcast();
    assert!(
        w.sims[1].needs[0] > 0.5,
        "heard the TV: {}",
        w.sims[1].needs[0]
    );

    // Saved mid-watch, the watch goes on after loading: the seat is found again.
    let mut loaded = World::from_save_json(TV, &w.save_json()).unwrap();
    run_ticks(&mut loaded, 5);
    assert_eq!(loaded.sims[0].use_seat, Some(1), "sat on the sofa again");
    assert_eq!(loaded.sims[0].pose, Pose::Sit);
}

#[test]
fn furniture_cannot_straddle_or_face_a_wall() {
    let mut w = world();
    // The wall runs along x = 6 for z 0..10: a 2-wide bed across it doesn't fit.
    let err = w.place_object("bed", 5, 5, 0).unwrap_err();
    assert!(err.to_string().contains("wall"), "{err}");
    // Facing +x (rot 1) from x = 5 means using it from across the wall.
    let err = w.place_object("fridge", 5, 7, 1).unwrap_err();
    assert!(err.to_string().contains("wall"), "{err}");
    // Turned away from the wall it's fine.
    w.place_object("fridge", 5, 7, 3).unwrap();
}

#[test]
fn speed_controls_tick_rate() {
    let mut w = world();
    w.apply(Command::SetSpeed { speed: 0 }).unwrap();
    w.advance();
    assert_eq!(w.tick, 0);
    w.apply(Command::SetSpeed { speed: 3 }).unwrap();
    w.advance();
    assert_eq!(w.tick, TICKS_PER_STEP[3] as u64);
    assert!(w.apply(Command::SetSpeed { speed: 9 }).is_err());
}

#[test]
fn quiet_hours_skip_ahead() {
    let content = CONTENT
        .replace(r#""pose":"lie","#, r#""pose":"lie","tags":["sleep"],"#)
        .replace(
            r#""needs":["#,
            r#""dayRhythm":{"sleepTags":["sleep"]},"needs":["#,
        );
    let mut w = World::from_json(&content, LOT, 7).unwrap();
    w.apply(Command::SetSpeed { speed: 1 }).unwrap();
    w.apply(Command::SetAutoFast { enabled: true }).unwrap();
    w.advance();
    assert_eq!(w.tick, 1, "awake: normal speed");
    w.sims[0].needs[1] = 0.0;
    w.apply(Command::Use {
        sim: 0,
        object: 1,
        interaction: 0,
    })
    .unwrap();
    while !w.sims[0].asleep(&w.content) {
        w.tick_once();
    }
    let before = w.tick;
    w.advance();
    let fast = TICKS_PER_STEP[AUTO_FAST_SPEED as usize] as u64;
    assert_eq!(w.tick, before + fast, "asleep: time-lapse");
    w.apply(Command::SetSpeed { speed: 0 }).unwrap();
    w.advance();
    assert_eq!(w.tick, before + fast, "paused stays paused");
}

#[test]
fn deterministic_for_same_seed() {
    let (mut a, mut b) = (world(), world());
    run_ticks(&mut a, crate::mins(100.0));
    run_ticks(&mut b, crate::mins(100.0));
    assert_eq!(a.sims[0].pos, b.sims[0].pos);
    assert_eq!(a.sims[0].needs, b.sims[0].needs);
}

#[test]
fn neighbours_do_not_use_other_households_objects() {
    let lot = r#"{"width":20,"depth":10,
            "plots":[{"name":"A","x":0,"z":0,"w":10,"d":10},{"name":"B","x":10,"z":0,"w":10,"d":10}],
            "households":[{"name":"A","plot":0,"player":true},{"name":"B","plot":1}],
            "objects":[{"def":"fridge","x":2,"z":2,"rot":0}],
            "sims":[{"name":"Neighbour","household":1,"x":14.5,"z":4.5}]}"#;
    let mut w = World::from_json(CONTENT, lot, 1).unwrap();
    w.sims[0].needs[0] = 0.0;
    run_ticks(&mut w, crate::mins(60.0));
    assert!(!w.objects[0].in_use());
    assert!(w.sims[0].pos[0] > 10.0, "stayed home");
}

#[test]
fn showering_happens_in_the_shower() {
    let content = r#"{
            "needs":[{"id":"hygiene","label":"Hygiene","decayPerHour":0.05}],
            "objects":[{"id":"shower","name":"Shower","interactions":[
              {"id":"shower","label":"Shower","minutes":20,"inside":true,"effects":{"hygiene":1.0}}]}]}"#;
    let lot = r#"{"width":10,"depth":10,
            "objects":[{"def":"shower","x":2,"z":2,"rot":0}],
            "sims":[{"name":"Ada","x":6.5,"z":6.5}]}"#;
    let mut w = World::from_json(content, lot, 1).unwrap();
    w.autonomy = false;
    w.sims[0].needs[0] = 0.1;
    w.apply(Command::Use {
        sim: 0,
        object: 0,
        interaction: 0,
    })
    .unwrap();
    let using = |w: &World| {
        w.sims[0]
            .current()
            .is_some_and(|a| matches!(a.phase, Phase::Using { .. }))
    };
    for _ in 0..crate::secs(60.0) {
        if using(&w) {
            break;
        }
        w.tick_once();
    }
    assert!(using(&w), "never got to the shower");
    let front = w.objects[0].front_tile(&w.content);
    let s = &w.sims[0];
    assert_eq!(
        s.tile(),
        (2, 2),
        "showers in the shower, not in front of it"
    );
    assert_eq!(s.pose, Pose::Stand);
    assert!(
        (s.yaw - w.objects[0].yaw()).abs() < 1e-4,
        "faces out of the shower"
    );
    assert_eq!(s.seated_from, Some(front));

    // Saved mid-shower: back out in front of it.
    let loaded = World::from_save_json(content, &w.save_json()).unwrap();
    assert_eq!(loaded.sims[0].tile(), front);

    // Done: out where they stepped in from.
    for _ in 0..crate::mins(600.0) {
        if !using(&w) {
            break;
        }
        w.tick_once();
    }
    assert!(!using(&w) && !w.objects[0].in_use(), "still showering");
    assert!(w.sims[0].needs[0] > 0.5);
    assert_eq!(w.sims[0].tile(), front);
    assert_eq!(w.sims[0].seated_from, None);
}

#[test]
fn a_full_queue_drops_neither_orders_nor_a_paid_recipe() {
    let task = |kind, directed| Task {
        kind,
        directed,
        routine: false,
    };
    let walk = TaskKind::MoveTo { x: 1.0, z: 1.0 };
    let cook = |servings| TaskKind::Cook {
        recipe: 0,
        step: 1,
        station: None,
        origin: None,
        servings,
        skipped: 0,
        minutes: 0,
    };
    let care = task(
        TaskKind::Use {
            object: 0,
            interaction: 1,
        },
        false,
    );
    // Chosen for them, unpaid: the last of those goes.
    let mut q: VecDeque<Task> = [
        care,
        task(walk, true),
        task(cook(2), false),
        task(walk, false),
        task(walk, true),
        task(cook(0), false),
        task(walk, true),
    ]
    .into();
    trim_queue(&mut q, 1);
    assert_eq!(q.len(), MAX_QUEUE);
    assert!(
        matches!(q[4].kind, TaskKind::MoveTo { .. })
            && q[4].directed
            && matches!(q[2].kind, TaskKind::Cook { servings: 2, .. })
    );
    // Nothing else to drop: over the cap.
    let mut q: VecDeque<Task> = [care, task(cook(3), false)]
        .into_iter()
        .chain([task(walk, true); MAX_QUEUE])
        .collect();
    trim_queue(&mut q, 1);
    assert_eq!(q.len(), MAX_QUEUE + 2);
}

/// A one-room home with a household and a fridge, for build and recolour.
const HOME_LOT: &str = r#"{"width":12,"depth":12,
        "plots":[{"name":"Home","x":0,"z":0,"w":12,"d":12,"entry":[6.5,11.5]}],
        "households":[{"name":"Ada","plot":0,"player":true}],
        "objects":[{"def":"fridge","x":1,"z":1,"rot":0}],
        "sims":[{"name":"Ada","household":0,"x":3.5,"z":5.5}]}"#;

#[test]
fn only_what_walkers_step_through_moves_the_navigation() {
    let mut w = World::from_json(CONTENT, HOME_LOT, 7).unwrap();
    let (nav, structure) = (w.nav_version, w.structure_version);
    // A new colour changes how it looks, not where anyone can walk.
    w.recolor(0, 0, Colors::new()).unwrap();
    assert_eq!(w.nav_version, nav, "a recolour isn't a change to walking");
    assert_ne!(w.structure_version, structure, "but views must rebuild");
    // A wall is.
    let wall = crate::command::EdgeEdit::new(
        crate::command::EdgeAxis::V,
        6,
        0,
        crate::command::EdgeKind::Wall,
    );
    w.build(0, &[wall]).unwrap();
    assert_eq!(w.nav_version, nav + 1);
    assert_eq!(w.structure_version, structure + 2);
}

#[test]
fn what_they_couldnt_get_to_is_remembered_a_while() {
    let mut w = world();
    let s = &mut w.sims[0];
    s.remember_unreachable(
        TaskKind::Use {
            object: 3,
            interaction: 1,
        },
        100,
    );
    s.remember_unreachable(TaskKind::Repair { object: 1 }, 100);
    s.remember_unreachable(TaskKind::Visit { plot: 2 }, 50);
    s.remember_unreachable(TaskKind::Clean { x: 4, z: 5 }, 200);
    // People are remembered too, for as short a while as the caller gives them.
    s.remember_unreachable(
        TaskKind::Social {
            target: 1,
            social: 0,
        },
        80,
    );
    assert_eq!(s.unreachable_now(60).count(), 4);
    assert!(
        s.unreachable_now(60)
            .any(|k| matches!(k, TaskKind::Social { target: 1, .. }))
    );
    assert!(
        !s.unreachable_now(60)
            .any(|k| matches!(k, TaskKind::Visit { .. })),
        "the shortest-lived made way"
    );
    assert!(
        s.couldnt_reach(3, 60)
            && s.couldnt_reach(1, 99)
            && !s.couldnt_reach(1, 100)
            && !s.couldnt_reach(2, 0)
    );
    // Saying the same thing again just lasts longer.
    s.remember_unreachable(TaskKind::Repair { object: 1 }, 300);
    assert!(s.couldnt_reach(1, 299));
    // Full: the one that runs out first makes way.
    s.remember_unreachable(
        TaskKind::Eat {
            item: Some(7),
            seat: None,
            plate: None,
            stage: 0,
        },
        300,
    );
    assert!(
        !s.unreachable_now(0)
            .any(|k| matches!(k, TaskKind::Social { .. })),
        "the shortest-lived made way"
    );
    // Object 1 is gone and the ids after it shift down.
    s.remap_unreachable(|o| {
        if o == 1 {
            None
        } else {
            Some(if o > 1 { o - 1 } else { o })
        }
    });
    assert!(s.couldnt_reach(2, 0) && !s.couldnt_reach(3, 0) && !s.couldnt_reach(1, 0));
    assert_eq!(s.unreachable_now(0).count(), 3);
}

/// Who a household's hosts are, and till when they're about (`hosts_about_till`): its
/// grown-ups not at work, till the latest of them leaves for work; a baby at home all day
/// keeps nobody about.
#[test]
fn the_hosts_about_are_the_grown_ups_till_the_last_leaves() {
    const CONTENT: &str = r#"{
            "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.01}],"objects":[],
            "careers":[{"id":"office","label":"Office","levels":[
                {"title":"Intern","pay":100,"start":12,"hours":4,"days":[0,1,2,3,4,5,6]}]}],
            "life":{"daysPerYear":2,"stages":[
                {"id":"baby","label":"Baby","from":0,"baby":true},
                {"id":"adult","label":"Adult","from":18}]},
            "economy":{"startingFunds":500}}"#;
    let town = |cleo: &str| {
        format!(
            r#"{{"width":40,"depth":14,
                "plots":[{{"name":"West","x":0,"z":0,"w":20,"d":10,"entry":[10.5,9.5]}},{{"name":"East","x":20,"z":0,"w":20,"d":10,"entry":[30.5,9.5]}}],
                "households":[{{"name":"Host","plot":0}},{{"name":"Guest","plot":1}}],
                "sims":[{{"name":"Ada","household":0,"x":4.5,"z":4.5,"age":30,"job":{{"career":"office"}}}},
                        {{"name":"Cleo","household":0,"x":6.5,"z":4.5,"age":{cleo}}},
                        {{"name":"Bo","household":1,"x":25.5,"z":4.5,"age":30}}]}}"#
        )
    };
    let mut w = World::from_json(CONTENT, &town("0.5"), 7).unwrap();
    w.autonomy = false;
    w.tick_once();
    let ada = leaves_for_work(&w.sims[0], &w.ctx()).expect("Ada works today");
    assert_eq!(hosts_about_till(&w.ctx(), 0), ada, "the baby is no host");
    // A grown-up at home all day keeps them about.
    let mut w = World::from_json(CONTENT, &town("30"), 7).unwrap();
    w.autonomy = false;
    w.tick_once();
    assert_eq!(hosts_about_till(&w.ctx(), 0), u64::MAX);
    // Gone to work, Ada is about no more: nobody is.
    let mut w = World::from_json(CONTENT, &town("0.5"), 7).unwrap();
    w.sims[0].away_until = Some(w.tick + 1000);
    w.tick_once();
    assert_eq!(hosts_about_till(&w.ctx(), 0), 0);
}

/// When a host leaves for work is when she really goes, as for a guest (`leaves_for_work`):
/// booked on a work taxi, at the taxi's time, not her walk's.
#[test]
fn a_host_on_a_work_taxi_leaves_at_its_time() {
    const CONTENT: &str = r#"{
            "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.01}],"objects":[],
            "careers":[{"id":"office","label":"Office","levels":[
                {"title":"Intern","pay":100,"start":12,"hours":4,"days":[0,1,2,3,4,5,6]}]}],
            "economy":{"startingFunds":500}}"#;
    const TOWN: &str = r#"{"width":40,"depth":14,
            "plots":[{"name":"West","x":0,"z":0,"w":20,"d":10,"entry":[10.5,9.5]}],
            "households":[{"name":"Host","plot":0}],
            "sims":[{"name":"Ada","household":0,"x":9.5,"z":8.5,"job":{"career":"office"}}]}"#;
    let mut w = World::from_json(CONTENT, TOWN, 7).unwrap();
    w.autonomy = false;
    w.tick_once();
    let walk = leaves_for_work(&w.sims[0], &w.ctx()).expect("Ada works today");
    assert_eq!(hosts_about_till(&w.ctx(), 0), walk);
    // Out to the kerb an hour sooner, for the taxi.
    let kerb = walk - crate::mins(60.0);
    let run = serde_json::json!({"id":3,"group":[0,0],"enter":kerb + 10_000,"dueOut":0,
            "depart":0,"stops":[{"sim":0,"plot":0,"due":kerb + 100,"leave":kerb,"state":"booked"}]});
    w.taxis.runs.push(serde_json::from_value(run).unwrap());
    w.sims[0].taxi = Some(crate::taxi::Booking { run: 3, stop: 0 });
    let ctx = w.ctx();
    assert_eq!(leaves_for_work(&w.sims[0], &ctx), Some(kerb));
    assert_eq!(hosts_about_till(&ctx, 0), kerb);
    let free = free_before_work(&w.sims[0], &ctx);
    assert_eq!(free, (kerb - w.tick) as f32 * MINUTES_PER_TICK);
}

/// The residents but the one being stepped (`Others`): each other one by index.
#[test]
fn others_are_everyone_but_the_one_stepped() {
    let w = World::from_json(CONTENT, LOT, 7).unwrap();
    let sims = std::slice::from_ref(&w.sims[0]);
    let all = Others::all(sims);
    assert!(all.get(0).is_some() && all.get(1).is_none());
    let (before, after) = (sims, sims);
    let others = Others { before, after };
    assert!(others.get(0).is_some() && others.get(1).is_none() && others.get(2).is_some());
    assert!(others.get(3).is_none());
}

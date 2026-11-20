use super::*;

/// What residents make of their home, once an hour: a room they love or dislike (a thought,
/// a feeling, and for a disliked one a wish about its weakest factor), broken things to fix,
/// and a busy essential they had to wait for (another bathroom). Wishes come true once the
/// room is better, the thing is fixed, or there is another room of that kind.
pub(crate) fn opinions(w: &mut World) {
    use crate::planner::{
        HomeWish, MAX_HOME_WISHES, ThoughtKind, WAIT_FORGET_DAYS, WAIT_REPEAT_DAYS, think,
    };
    let tick = w.tick;
    let today = crate::clock::day(tick);
    // By plot, its rooms and its objects, in order: gathered once for everyone (a big world's
    // thousands of each, asked after for every resident's home).
    let mut rooms_on: Vec<Vec<&RoomInfo>> = Vec::new();
    for r in &w.rooms {
        if let Some(p) = r.plot {
            let p = p as usize;
            if rooms_on.len() <= p {
                rooms_on.resize_with(p + 1, Vec::new);
            }
            rooms_on[p].push(r);
        }
    }
    let mut objects_on: Vec<Vec<usize>> = Vec::new();
    for (o, p) in w.object_plot.iter().enumerate().take(w.objects.len()) {
        if let Some(p) = *p {
            let p = p as usize;
            if objects_on.len() <= p {
                objects_on.resize_with(p + 1, Vec::new);
            }
            objects_on[p].push(o);
        }
    }
    for i in 0..w.sims.len() {
        let s = &w.sims[i];
        let Some(home) = w.households[s.household as usize].plot.filter(|_| s.here()) else {
            continue;
        };
        let (x, z) = s.tile();
        let at_home =
            s.away_until.is_none() && !s.asleep(&w.content) && w.plot_at(x, z) == Some(home);
        let rooms: &[&RoomInfo] = rooms_on.get(home as usize).map_or(&[], Vec::as_slice);
        let on_home: &[usize] = objects_on.get(home as usize).map_or(&[], Vec::as_slice);
        let broken: Vec<usize> = on_home
            .iter()
            .map(|&o| &w.objects[o])
            .filter(|o| o.broken())
            .map(|o| o.def)
            .collect();
        // Wishes that came true.
        let good = |kind: Option<usize>, garden: bool, factor: u8| {
            rooms.iter().any(|r| {
                r.garden == garden && r.kind == kind && r.scores.factors()[factor as usize] >= 0.6
            })
        };
        let kept: Vec<HomeWish> = w.sims[i]
            .planner
            .home_wishes
            .iter()
            .copied()
            .filter(|wish| match *wish {
                HomeWish::Room {
                    kind,
                    garden,
                    factor,
                } => !good(kind, garden, factor),
                HomeWish::Fix { def } => broken.contains(&def),
                HomeWish::Another { kind } => {
                    rooms
                        .iter()
                        .filter(|r| !r.garden && r.kind == Some(kind))
                        .count()
                        < 2
                        && s.planner
                            .waited_day
                            .is_some_and(|d| today <= d + WAIT_FORGET_DAYS)
                }
            })
            .collect();
        let mut wishes = kept;
        let mut waited = w.sims[i].planner.waited_day;
        let mut thought: Option<(ThoughtKind, usize)> = None;
        let mut feeling = None;
        if at_home {
            let rules = &w.content.room_rules;
            let roll = w.rng.next_f32();
            if let Some(room) = w.room_at_tile(x, z) {
                let s = room.scores;
                if s.overall >= rules.love && roll < 0.3 {
                    thought = Some((ThoughtKind::RoomLoved, room_subject(room, 0)));
                    feeling = rules.love_feeling;
                } else if s.overall <= rules.dislike && roll < 0.5 {
                    let factor = s.weakest();
                    thought = Some((ThoughtKind::RoomDisliked, room_subject(room, factor)));
                    feeling = rules.dislike_feeling;
                    let wish = HomeWish::Room {
                        kind: room.kind,
                        garden: room.garden,
                        factor: factor as u8,
                    };
                    if !wishes.contains(&wish) {
                        wishes.push(wish);
                    }
                }
            }
            // Something broke: they'd like it fixed.
            for &def in &broken {
                let wish = HomeWish::Fix { def };
                if !wishes.contains(&wish) {
                    wishes.push(wish);
                    thought = Some((ThoughtKind::Broken, def));
                }
            }
            // Badly needing what's taken (the toilet): another room for it, where content asks.
            let me = i as u32;
            for acc in &w.content.accidents {
                let Some(kind) = acc.another_room else {
                    continue;
                };
                let have = rooms
                    .iter()
                    .filter(|r| !r.garden && r.kind == Some(kind))
                    .count();
                if w.sims[i].needs[acc.need] >= 0.15 || have >= 2 {
                    continue;
                }
                let fillers: Vec<&crate::world::ObjectInstance> = on_home
                    .iter()
                    .map(|&o| &w.objects[o])
                    .filter(|o| {
                        !o.broken()
                            && w.content.objects[o.def]
                                .interactions
                                .iter()
                                .any(|it| it.autonomous && it.total_gain[acc.need] > 0.0)
                    })
                    .collect();
                // (Taken, or in a room someone else keeps shut: the bathroom with someone in it.)
                let here = w.lot.room_at(x, z);
                let shut = |o: &crate::world::ObjectInstance| {
                    let room = crate::privacy::room_of_object(&w.content, &w.lot, o);
                    room != here && crate::privacy::shut_to(&w.shut, me, None, room)
                };
                let busy = !fillers.is_empty()
                    && fillers.iter().all(|o| {
                        (o.users().any(|u| u != me)
                            && !o.has_free_slot(w.content.objects[o.def].slots))
                            || shut(o)
                    });
                if busy {
                    // Once is bad luck; again on another day this week, they'd like a second one.
                    let again = waited.is_some_and(|d| d < today && today <= d + WAIT_REPEAT_DAYS);
                    let wish = HomeWish::Another { kind };
                    if again && !wishes.contains(&wish) {
                        wishes.push(wish);
                    }
                    waited = Some(today);
                }
            }
        }
        while wishes.len() > MAX_HOME_WISHES {
            wishes.remove(0);
        }
        let content = &w.content;
        let sim = &mut w.sims[i];
        sim.planner.home_wishes = wishes;
        sim.planner.waited_day = waited;
        if let Some((kind, subject)) = thought
            && sim.planner.thought.is_none()
        {
            think(sim, kind, subject, tick);
        }
        if let Some(f) = feeling {
            crate::social::add_feeling(&mut sim.feelings, f, &content.feelings, tick);
        }
    }
}

//! Diagnostic: how residents' days actually go. A whole town lives a week on its own (like
//! `soak.rs`) and, watching only what's public each tick (current activity, task, tags, pose),
//! this prints for every resident and day: sleep blocks, naps, toilet visits and whether hands
//! were washed after, food eaten sitting or standing, how long each cooking step took and how
//! low and how high needs went; then a summary across the town, with what residents were doing
//! when a need ran low (under `LOW`) or out, and the accidents that came of it.
//!
//! `cargo test --release -p sim-core --test routines -- --ignored --nocapture`
//! (with `LOD=1` the rest of the town runs at lower detail, as in the game; `SEED=n` for another
//! town and other choices than seed 7's; `BY_DAY=1` for needs day by day). The test town's
//! houses have only a fridge to cook at; for generated houses with whole kitchens, make worlds
//! (`node --experimental-transform-types tools/worlds/sample.ts worlds.json`) and set
//! `WORLDS=worlds.json` (the first world, or the one of kind `WORLD`, e.g. `WORLD=medium`).

mod common;

use std::collections::BTreeMap;

use sim_core::content::{Pose, TagMask};
use sim_core::social::EventKind;
use sim_core::world::{Phase, Sim, TaskKind};
use sim_core::{MINUTES_PER_TICK, World, clock};

const DAYS: u32 = 7;
const TICKS_PER_MINUTE: u64 = (1.0 / MINUTES_PER_TICK) as u64;
/// Hands washed within this long after the toilet count as washed after it.
const WASH_WITHIN_MINUTES: u64 = 10;
/// A need this low is about to run out: residents should have done something before.
const LOW: f32 = 0.05;
/// Time spent under this counts as running low.
const LOWISH: f32 = 0.1;

/// What a resident is doing this tick, as far as the diagnostic cares.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Doing {
    Sleep {
        object: Option<u32>,
    },
    Nap {
        object: Option<u32>,
    },
    Toilet,
    Wash,
    /// A cooked serving (an `Eat` task), at a seat or standing.
    Eat {
        seated: bool,
    },
    /// A plain food interaction (a snack at the fridge), sitting or not.
    Food {
        object: u32,
        seated: bool,
    },
    Cook {
        recipe: usize,
        step: u8,
        minutes: u16,
    },
    Other,
}

/// Tag masks and need indices looked up by name (missing ones: 0 / none).
struct Lookup {
    sleep: TagMask,
    nap: TagMask,
    handwash: TagMask,
    hygiene: TagMask,
    food: TagMask,
    bladder: Option<usize>,
    hunger: Option<usize>,
}

impl Lookup {
    fn new(w: &World) -> Self {
        let tag = |name: &str| {
            w.content
                .tags
                .iter()
                .position(|t| t == name)
                .map_or(0, |i| 1 << i)
        };
        let need = |name: &str| w.content.needs.iter().position(|n| n.id == name);
        Self {
            sleep: tag("sleep"),
            nap: tag("nap"),
            handwash: tag("handwash"),
            hygiene: tag("hygiene"),
            food: tag("food"),
            bladder: need("bladder"),
            hunger: need("hunger"),
        }
    }

    fn doing(&self, w: &World, s: &Sim) -> Doing {
        let Some(a) = s.current() else {
            return Doing::Other;
        };
        if !matches!(a.phase, Phase::Using { .. }) {
            return Doing::Other;
        }
        let object = match a.task.kind {
            TaskKind::Use { object, .. } => Some(object),
            _ => None,
        };
        if a.tags & self.sleep != 0 {
            return Doing::Sleep { object };
        }
        if a.tags & self.nap != 0 {
            return Doing::Nap { object };
        }
        match a.task.kind {
            TaskKind::Eat { seat, stage: 1, .. } => Doing::Eat {
                seated: seat.is_some(),
            },
            TaskKind::Cook {
                recipe,
                step,
                minutes,
                ..
            } => Doing::Cook {
                recipe,
                step,
                minutes,
            },
            TaskKind::Use {
                object,
                interaction,
            } => {
                let i =
                    &w.content.objects[w.objects[object as usize].def].interactions[interaction];
                let gains = |n: Option<usize>| n.is_some_and(|n| i.total_gain[n] > 0.0);
                if gains(self.bladder) {
                    Doing::Toilet
                } else if i.tags & self.handwash != 0
                    || (self.handwash == 0 && i.tags & self.hygiene != 0 && i.minutes <= 10.0)
                {
                    // Without a `handwash` tag: a short hygiene interaction (a sink, not a shower).
                    Doing::Wash
                } else if i.tags & self.food != 0 && gains(self.hunger) && i.eats.is_none() {
                    // (Fetching food to eat sitting down counts when they eat it, as a serving.)
                    Doing::Food {
                        object,
                        seated: s.pose == Pose::Sit,
                    }
                } else {
                    Doing::Other
                }
            }
            _ => Doing::Other,
        }
    }
}

/// A stretch of sleep or a nap: when it started, how long (minutes), where.
#[derive(Debug, Clone)]
struct Rest {
    start: u64,
    minutes: f32,
    object: String,
}

/// One resident's routine day.
#[derive(Debug, Clone, Default)]
struct Day {
    sleeps: Vec<Rest>,
    naps: Vec<Rest>,
    toilets: u32,
    washed_after: u32,
    washes: u32,
    eat_seated: u32,
    eat_standing: u32,
    food_seated: u32,
    food_standing: u32,
    /// Step label → (planned minutes, actual minutes, count).
    cook: BTreeMap<String, (f32, f32, u32)>,
    need_min: Vec<f32>,
    need_sum: Vec<f64>,
    /// Minutes each need spent under `LOWISH`.
    need_lowish: Vec<u32>,
    samples: u32,
    /// Minutes spent at work or school, asleep, walking, doing nothing...
    time: BTreeMap<&'static str, u32>,
    /// The bladder when each toilet visit started, and whether it was out of habit (a routine).
    toilet_at: Vec<(f32, bool)>,
}

/// One resident's record.
#[derive(Debug, Clone, Default)]
struct Record {
    days: BTreeMap<u32, Day>,
    /// What they did last tick and since when.
    last: Option<(Doing, u64)>,
    /// When their last toilet visit ended (and whether hands were washed after it yet).
    toilet_end: Option<(u64, u32, bool)>,
    /// What they're doing (as `what`), since when and with which needs then.
    what: Option<(String, u64, Vec<f32>)>,
    /// Needs as last sampled (to see one go under `LOW` or run out).
    needs: Vec<f32>,
}

/// What residents were doing when a need went under `LOW` or ran out: (need, "low"/"out",
/// what) → how often, the minutes it had gone on and that need when it started (summed).
type Lows = BTreeMap<(usize, &'static str, String), (u32, f32, f32)>;

/// The routine day of `tick`: days run 06:00 to 06:00, so a night's sleep counts for the
/// evening it started.
fn routine_day(tick: u64) -> u32 {
    clock::day(tick.saturating_sub(6 * 60 * TICKS_PER_MINUTE))
}

fn clock_time(tick: u64) -> String {
    let h = clock::hour(tick);
    format!("{:02}:{:02}", h as u32, ((h.fract() * 60.0) as u32).min(59))
}

fn object_name(w: &World, object: Option<u32>) -> String {
    object.map_or("-".into(), |o| {
        w.content.objects[w.objects[o as usize].def].id.clone()
    })
}

impl Record {
    fn day(&mut self, d: u32, needs: usize) -> &mut Day {
        self.days.entry(d).or_insert_with(|| Day {
            need_min: vec![1.0; needs],
            need_sum: vec![0.0; needs],
            need_lowish: vec![0; needs],
            ..Default::default()
        })
    }

    /// `doing` ended at `tick` after starting at `start`.
    fn ended(&mut self, w: &World, doing: Doing, start: u64, tick: u64) {
        let needs = w.content.needs.len();
        let minutes = (tick - start) as f32 * MINUTES_PER_TICK;
        let d = routine_day(start);
        match doing {
            Doing::Sleep { object } => self.day(d, needs).sleeps.push(Rest {
                start,
                minutes,
                object: object_name(w, object),
            }),
            Doing::Nap { object } => self.day(d, needs).naps.push(Rest {
                start,
                minutes,
                object: object_name(w, object),
            }),
            Doing::Toilet => {
                self.day(d, needs).toilets += 1;
                self.toilet_end = Some((tick, d, false));
            }
            Doing::Cook {
                recipe,
                step,
                minutes: planned,
            } => {
                let label = w.content.recipes[recipe]
                    .steps
                    .get(step as usize)
                    .map_or("?".into(), |s| s.label.clone());
                let e = self.day(d, needs).cook.entry(label).or_default();
                *e = (e.0 + f32::from(planned), e.1 + minutes, e.2 + 1);
            }
            _ => {}
        }
    }

    /// `doing` started at `tick`.
    fn started(&mut self, w: &World, doing: Doing, tick: u64) {
        let needs = w.content.needs.len();
        let d = routine_day(tick);
        match doing {
            Doing::Wash => {
                self.day(d, needs).washes += 1;
                if let Some((end, toilet_day, false)) = self.toilet_end
                    && tick - end <= WASH_WITHIN_MINUTES * TICKS_PER_MINUTE
                {
                    self.day(toilet_day, needs).washed_after += 1;
                    self.toilet_end = Some((end, toilet_day, true));
                }
            }
            Doing::Eat { seated } => {
                let day = self.day(d, needs);
                if seated {
                    day.eat_seated += 1
                } else {
                    day.eat_standing += 1
                }
            }
            Doing::Food { seated, .. } => {
                let day = self.day(d, needs);
                if seated {
                    day.food_seated += 1
                } else {
                    day.food_standing += 1
                }
            }
            _ => {}
        }
    }
}

/// What a resident is up to, in a few words (for the lows): at work, asleep, walking to the
/// toilet, sitting at the computer...
fn what(w: &World, s: &Sim) -> String {
    if s.away_until.is_some() {
        return if s.job.is_some() {
            "at work".into()
        } else {
            "at school".into()
        };
    }
    let Some(a) = s.current() else {
        return if s.engaged_with.is_some() {
            "talked to".into()
        } else {
            "nothing".into()
        };
    };
    let task = match a.task.kind {
        TaskKind::Use {
            object,
            interaction,
        } => {
            let o = &w.content.objects[w.objects[object as usize].def];
            format!("{} ({})", o.interactions[interaction].label, o.id)
        }
        TaskKind::Social { social, .. } => format!("social {}", w.content.socials[social].id),
        TaskKind::Work => "off to work/school".into(),
        TaskKind::GoHome => "home".into(),
        TaskKind::Spot { accident } => format!("accident {}", w.content.accidents[accident].id),
        // (Cook, eat, dishes, clean, repair, visit...)
        other => format!("{other:?}")
            .split([' ', '{'])
            .next()
            .unwrap_or("?")
            .to_lowercase(),
    };
    let task = if a.task.directed {
        format!("{task} [directed]")
    } else {
        task
    };
    match a.phase {
        Phase::Routing { .. } => format!("walking: {task}"),
        Phase::Queued { .. } => format!("queued: {task}"),
        Phase::Knocking { .. } => "knocking".into(),
        _ => task,
    }
}

/// Whether their home has nothing that fills need `n`.
fn home_lacks(w: &World, s: &Sim, n: usize) -> bool {
    let Some(home) = w.households[s.household as usize].plot else {
        return true;
    };
    !w.objects.iter().any(|o| {
        let (x, z) = o.front_tile(&w.content);
        w.plot_at(x, z) == Some(home)
            && !o.broken()
            && w.content.objects[o.def]
                .interactions
                .iter()
                .any(|i| i.autonomous && i.total_gain[n] > 0.0)
    })
}

fn run(seed: u64, player_size: usize) -> (World, Vec<Record>, Lows, BTreeMap<String, u32>) {
    let content = common::content();
    let town = match std::env::var("WORLDS") {
        Ok(path) => {
            let worlds: Vec<serde_json::Value> =
                serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
            let kind = std::env::var("WORLD").ok();
            let world = worlds
                .iter()
                .find(|w| kind.as_ref().is_none_or(|k| w["kind"] == k.as_str()))
                .expect("a world of that kind");
            eprintln!("{} {} ({})", world["kind"], world["seed"], world["name"]);
            world["town"].to_string()
        }
        Err(_) => common::town(&content, seed, player_size),
    };
    let mut w = World::from_json(&content, &town, seed as u32).expect("town loads");
    common::lod_from_env(&mut w);
    let look = Lookup::new(&w);
    let needs = w.content.needs.len();
    let mut records: Vec<Record> = vec![Record::default(); w.sims.len()];
    let mut lows = Lows::new();
    let mut accidents: BTreeMap<String, u32> = BTreeMap::new();
    let mut seen = w.events.last_id();
    // A little past the last morning, so the last night's sleep is over.
    let end = clock::tick_at(DAYS + 1, 10.0 * 60.0).unwrap();
    let started = std::time::Instant::now();
    while w.tick < end {
        w.tick_once();
        let tick = w.tick;
        if records.len() < w.sims.len() {
            records.resize(w.sims.len(), Record::default());
        }
        for (i, s) in w.sims.iter().enumerate() {
            if !s.here() || w.content.life.baby(s.age) {
                continue;
            }
            let r = &mut records[i];
            let doing = look.doing(&w, s);
            match r.last {
                Some((last, _)) if last == doing => {}
                last => {
                    if let Some((prev, since)) = last {
                        r.ended(&w, prev, since, tick);
                    }
                    r.started(&w, doing, tick);
                    if let (Doing::Toilet, Some(b)) = (doing, look.bladder) {
                        let habit = s.current().is_some_and(|a| a.task.routine);
                        r.day(routine_day(tick), needs)
                            .toilet_at
                            .push((s.needs[b], habit));
                    }
                    r.last = Some((doing, tick));
                }
            }
            if tick.is_multiple_of(TICKS_PER_MINUTE) {
                let day = r.day(routine_day(tick), needs);
                for n in 0..needs {
                    day.need_min[n] = day.need_min[n].min(s.needs[n]);
                    day.need_sum[n] += f64::from(s.needs[n]);
                    day.need_lowish[n] += u32::from(s.needs[n] < LOWISH);
                }
                day.samples += 1;
                // What they're doing, and whether a need just went low or ran out meanwhile.
                let now = what(&w, s);
                let class = match now.as_str() {
                    "at work" | "at school" => "away",
                    "nothing" => "nothing",
                    _ if now.starts_with("walking") => "walking",
                    _ if matches!(look.doing(&w, s), Doing::Sleep { .. } | Doing::Nap { .. }) => {
                        "asleep"
                    }
                    _ if now.starts_with("social") || now == "talked to" => "talking",
                    "cook" | "eat" => "meals",
                    "dishes" | "clean" | "repair" => "chores",
                    _ if matches!(look.doing(&w, s), Doing::Toilet | Doing::Wash) => "toilet",
                    _ if matches!(look.doing(&w, s), Doing::Food { .. }) => "meals",
                    _ => "doing",
                };
                *day.time.entry(class).or_default() += 1;
                if r.what.as_ref().is_none_or(|(was, ..)| *was != now) {
                    r.what = Some((now, tick, s.needs[..needs].to_vec()));
                }
                if r.needs.len() == needs {
                    for n in (0..needs).filter(|&n| !w.content.needs[n].room) {
                        let (before, after) = (r.needs[n], s.needs[n]);
                        for (edge, label) in [(LOW, "low"), (1e-4, "out")] {
                            if before >= edge && after < edge {
                                let (doing, since, at) = r.what.as_ref().expect("set above");
                                let doing = if home_lacks(&w, s, n) {
                                    format!("{doing} (none at home)")
                                } else {
                                    doing.clone()
                                };
                                let e = lows.entry((n, label, doing)).or_default();
                                e.0 += 1;
                                e.1 += (tick - since) as f32 * MINUTES_PER_TICK;
                                e.2 += at[n];
                            }
                        }
                    }
                }
                r.needs = s.needs[..needs].to_vec();
            }
        }
        for e in w
            .events
            .iter()
            .filter(|e| e.id > seen && e.kind == EventKind::Accident)
        {
            let id =
                e.n.and_then(|a| w.content.accidents.get(a as usize))
                    .map_or("?".into(), |a| a.id.clone());
            let s = &w.sims[e.a as usize];
            eprintln!(
                "accident: {} {} (h{}) at {}, day {}",
                id,
                s.name,
                s.household,
                clock_time(e.tick),
                clock::day(e.tick)
            );
            *accidents.entry(id).or_default() += 1;
        }
        seen = w.events.last_id();
    }
    // Whatever is still going on ends now.
    for r in &mut records {
        if let Some((prev, since)) = r.last.take() {
            r.ended(&w, prev, since, w.tick);
        }
    }
    eprintln!(
        "{} residents, {DAYS} days in {:.1} s",
        w.sims.len(),
        started.elapsed().as_secs_f64()
    );
    (w, records, lows, accidents)
}

fn rests(list: &[Rest]) -> String {
    let all: Vec<String> = list
        .iter()
        .map(|r| {
            format!(
                "{} {:.1}h {}",
                clock_time(r.start),
                r.minutes / 60.0,
                r.object
            )
        })
        .collect();
    if all.is_empty() {
        "-".into()
    } else {
        all.join(", ")
    }
}

fn report(w: &World, records: &[Record], lows: &Lows, accidents: &BTreeMap<String, u32>) {
    let needs = w.content.needs.len();
    let short: Vec<String> = w
        .content
        .needs
        .iter()
        .map(|n| n.id.chars().take(4).collect())
        .collect();
    eprintln!("Routine days run 06:00 to 06:00; needs are min/avg.");
    for (i, s) in w.sims.iter().enumerate() {
        let r = &records[i];
        if r.days.is_empty() {
            continue;
        }
        let stage = w
            .content
            .life
            .stage_at(s.age)
            .map_or("-", |st| st.id.as_str());
        let job = s.job.as_ref().map_or("-".to_string(), |j| {
            w.content.careers[j.career].levels[j.level].title.clone()
        });
        eprintln!("\n{} (h{}, {stage}, {job})", s.name, s.household);
        for (d, day) in r.days.iter().filter(|(d, _)| (1..=DAYS).contains(*d)) {
            let cook: Vec<String> = day
                .cook
                .iter()
                .map(|(l, (p, a, n))| format!("{l} {:.0}/{:.0}", p / *n as f32, a / *n as f32))
                .collect();
            let need: Vec<String> = (0..needs)
                .map(|n| {
                    format!(
                        "{} {:.2}/{:.2}",
                        short[n],
                        day.need_min[n],
                        day.need_sum[n] / f64::from(day.samples.max(1))
                    )
                })
                .collect();
            eprintln!(
                "  d{d} sleep {} | naps {}: {} | toilet {} washed {} (sink {}) | eat seated {} standing {} · food sitting {} standing {}{}",
                rests(&day.sleeps),
                day.naps.len(),
                rests(&day.naps),
                day.toilets,
                day.washed_after,
                day.washes,
                day.eat_seated,
                day.eat_standing,
                day.food_seated,
                day.food_standing,
                if cook.is_empty() {
                    String::new()
                } else {
                    format!(" | cook (planned/actual min) {}", cook.join(", "))
                },
            );
            eprintln!("      {}", need.join("  "));
        }
    }
    summary(w, records, lows, accidents);
}

fn summary(w: &World, records: &[Record], lows: &Lows, accidents: &BTreeMap<String, u32>) {
    let needs = w.content.needs.len();
    let days: Vec<&Day> = records
        .iter()
        .flat_map(|r| {
            r.days
                .iter()
                .filter(|(d, _)| (1..=DAYS).contains(*d))
                .map(|(_, day)| day)
        })
        .collect();
    let n = days.len().max(1) as f32;
    let pct = |a: usize, b: usize| {
        if b == 0 {
            0.0
        } else {
            a as f32 * 100.0 / b as f32
        }
    };
    eprintln!("\n=== Summary: {} resident-days ===", days.len());
    // Sleep: blocks a night, the longest block's length and start.
    let mut blocks = [0usize; 4];
    let (mut main_len, mut main_ok, mut start_ok, mut mains) = (0.0, 0, 0, 0);
    let mut starts: Vec<f32> = Vec::new();
    for day in &days {
        let real: Vec<&Rest> = day.sleeps.iter().filter(|r| r.minutes >= 30.0).collect();
        blocks[real.len().min(3)] += 1;
        if let Some(main) = real.iter().max_by(|a, b| a.minutes.total_cmp(&b.minutes)) {
            mains += 1;
            main_len += main.minutes / 60.0;
            main_ok += usize::from((7.0 * 60.0..=9.0 * 60.0).contains(&main.minutes));
            let h = clock::hour(main.start);
            start_ok += usize::from(!(0.5..21.5).contains(&h));
            starts.push(if h < 12.0 { h + 24.0 } else { h });
        }
    }
    starts.sort_by(f32::total_cmp);
    let at = |q: f32| {
        starts
            .get(((starts.len() as f32 - 1.0) * q).round() as usize)
            .map_or("-".into(), |h| {
                let h = h % 24.0;
                format!("{:02}:{:02}", h as u32, (h.fract() * 60.0) as u32)
            })
    };
    eprintln!(
        "sleep blocks (>=30 min) a night: 0: {}  1: {}  2: {}  3+: {}  ({:.0}% exactly one)",
        blocks[0],
        blocks[1],
        blocks[2],
        blocks[3],
        pct(blocks[1], days.len())
    );
    eprintln!(
        "longest block: avg {:.1} h, {:.0}% 7-9 h; starts p10 {} median {} p90 {}, {:.0}% 21:30-00:30",
        main_len / mains.max(1) as f32,
        pct(main_ok, mains),
        at(0.1),
        at(0.5),
        at(0.9),
        pct(start_ok, mains)
    );
    let short_sleeps: usize = days
        .iter()
        .map(|d| d.sleeps.iter().filter(|r| r.minutes < 30.0).count())
        .sum();
    eprintln!("sleeps under 30 min (woken / interrupted): {short_sleeps}");
    // Naps.
    let naps: usize = days.iter().map(|d| d.naps.len()).sum();
    let nap_minutes: f32 = days
        .iter()
        .flat_map(|d| d.naps.iter().map(|r| r.minutes))
        .sum();
    let most = days.iter().map(|d| d.naps.len()).max().unwrap_or(0);
    let few = days.iter().filter(|d| d.naps.len() <= 1).count();
    eprintln!(
        "naps: {:.2} a day (avg {:.0} min), most in a day {most}, {:.0}% of days <= 1",
        naps as f32 / n,
        nap_minutes / naps.max(1) as f32,
        pct(few, days.len())
    );
    // Toilet and hands.
    let toilets: u32 = days.iter().map(|d| d.toilets).sum();
    let washed: u32 = days.iter().map(|d| d.washed_after).sum();
    let washes: u32 = days.iter().map(|d| d.washes).sum();
    eprintln!(
        "toilet: {:.1} visits a day, {:.0}% followed by washing hands within {WASH_WITHIN_MINUTES} min ({:.1} sink washes a day)",
        toilets as f32 / n,
        pct(washed as usize, toilets as usize),
        washes as f32 / n
    );
    let at: Vec<(f32, bool)> = days
        .iter()
        .flat_map(|d| d.toilet_at.iter().copied())
        .collect();
    let habits = at.iter().filter(|a| a.1).count();
    let chosen: Vec<f32> = at.iter().filter(|a| !a.1).map(|a| a.0).collect();
    let mut bands = [0usize; 4];
    for &b in &chosen {
        bands[if b < 0.2 {
            0
        } else if b < 0.4 {
            1
        } else if b < 0.6 {
            2
        } else {
            3
        }] += 1;
    }
    eprintln!(
        "toilet visits: {:.1} a day out of habit (getting up, bed, leaving, home, meals), {:.1} chosen, at bladder avg {:.2} (<0.2: {}, 0.2-0.4: {}, 0.4-0.6: {}, >=0.6: {})",
        habits as f32 / n,
        chosen.len() as f32 / n,
        chosen.iter().sum::<f32>() / chosen.len().max(1) as f32,
        bands[0],
        bands[1],
        bands[2],
        bands[3]
    );
    // Eating.
    let sum = |f: fn(&Day) -> u32| days.iter().map(|d| f(d)).sum::<u32>();
    let (es, et) = (sum(|d| d.eat_seated), sum(|d| d.eat_standing));
    let (fs, ft) = (sum(|d| d.food_seated), sum(|d| d.food_standing));
    eprintln!(
        "eating: servings {:.2} a day ({:.0}% seated), plain food {:.2} a day ({:.0}% sitting); {:.0}% of all seated",
        (es + et) as f32 / n,
        pct(es as usize, (es + et) as usize),
        (fs + ft) as f32 / n,
        pct(fs as usize, (fs + ft) as usize),
        pct((es + fs) as usize, (es + et + fs + ft) as usize)
    );
    // Cooking steps.
    let mut cook: BTreeMap<String, (f32, f32, u32)> = BTreeMap::new();
    for day in &days {
        for (l, (p, a, c)) in &day.cook {
            let e = cook.entry(l.clone()).or_default();
            *e = (e.0 + p, e.1 + a, e.2 + c);
        }
    }
    let line: Vec<String> = cook
        .iter()
        .map(|(l, (p, a, c))| format!("{l} {:.1}/{:.1} (x{c})", p / *c as f32, a / *c as f32))
        .collect();
    eprintln!(
        "cooking steps, planned/actual min: {}",
        if line.is_empty() {
            "-".into()
        } else {
            line.join(", ")
        }
    );
    // Needs.
    let line: Vec<String> = (0..needs)
        .map(|k| {
            let low = days.iter().map(|d| d.need_min[k]).fold(1.0f32, f32::min);
            let mins = days.iter().map(|d| d.need_min[k]).sum::<f32>() / n;
            let avg = days
                .iter()
                .map(|d| d.need_sum[k] / f64::from(d.samples.max(1)))
                .sum::<f64>()
                / f64::from(n);
            format!(
                "{} {:.2}/{:.2}/{:.2}",
                w.content.needs[k].id, low, mins, avg
            )
        })
        .collect();
    eprintln!("needs lowest/avg daily min/avg: {}", line.join("  "));
    // Running low: how often, how long, doing what.
    let real: Vec<usize> = (0..needs).filter(|&k| !w.content.needs[k].room).collect();
    let any = days
        .iter()
        .filter(|d| real.iter().any(|&k| d.need_min[k] < LOW))
        .count();
    let line: Vec<String> = real
        .iter()
        .map(|&k| {
            let under = days.iter().filter(|d| d.need_min[k] < LOW).count();
            let minutes = days.iter().map(|d| d.need_lowish[k]).sum::<u32>() as f32 / n;
            format!(
                "{} {:.0}%/{:.0}",
                w.content.needs[k].id,
                pct(under, days.len()),
                minutes
            )
        })
        .collect();
    eprintln!(
        "resident-days with a need under {LOW}: {any} ({:.1}%); per need, % of days under {LOW} / minutes a day under {LOWISH}: {}",
        pct(any, days.len()),
        line.join("  ")
    );
    let mut time: BTreeMap<&str, u32> = BTreeMap::new();
    for day in &days {
        for (k, m) in &day.time {
            *time.entry(k).or_default() += m;
        }
    }
    let line: Vec<String> = time
        .iter()
        .map(|(k, m)| format!("{k} {:.1}", *m as f32 / 60.0 / n))
        .collect();
    eprintln!("hours a day: {}", line.join("  "));
    let by_day: Vec<String> = (1..=DAYS)
        .map(|d| {
            let list: Vec<&Day> = records.iter().filter_map(|r| r.days.get(&d)).collect();
            let low = list
                .iter()
                .filter(|day| real.iter().any(|&k| day.need_min[k] < LOW))
                .count();
            format!("d{d} {:.0}%", pct(low, list.len()))
        })
        .collect();
    eprintln!("by day: {}", by_day.join("  "));
    if std::env::var("BY_DAY").is_ok() {
        for d in 1..=DAYS {
            let list: Vec<&Day> = records.iter().filter_map(|r| r.days.get(&d)).collect();
            let avg: Vec<String> = real
                .iter()
                .map(|&k| {
                    let a = list
                        .iter()
                        .map(|day| day.need_sum[k] / f64::from(day.samples.max(1)))
                        .sum::<f64>()
                        / list.len().max(1) as f64;
                    let low = list.iter().filter(|day| day.need_min[k] < LOW).count();
                    format!(
                        "{} {a:.2}/{:.0}%",
                        w.content.needs[k].id.chars().take(4).collect::<String>(),
                        pct(low, list.len())
                    )
                })
                .collect();
            eprintln!("  d{d} avg/low: {}", avg.join("  "));
        }
    }
    let line: Vec<String> = accidents
        .iter()
        .map(|(id, c)| format!("{id} {c}"))
        .collect();
    eprintln!(
        "accidents: {}",
        if line.is_empty() {
            "none".into()
        } else {
            line.join(", ")
        }
    );
    for (label, edge) in [("low", LOW), ("out", 0.0)] {
        let mut list: Vec<_> = lows.iter().filter(|(k, _)| k.1 == label).collect();
        list.sort_by_key(|e| std::cmp::Reverse(e.1.0));
        let total: u32 = list.iter().map(|(_, v)| v.0).sum();
        eprintln!(
            "needs under {edge}: {total} times, doing (count, avg minutes at it, avg need when it started):"
        );
        for ((k, _, doing), (c, minutes, at)) in list.iter().take(25) {
            let c = *c as f32;
            eprintln!(
                "  {:<8} x{:<4} {doing}: {:.0} min, from {:.2}",
                w.content.needs[*k].id,
                c,
                minutes / c,
                at / c
            );
        }
    }
}

#[test]
#[ignore]
fn a_week_of_routines() {
    let seed = std::env::var("SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(7);
    let (w, records, lows, accidents) = run(seed, 2);
    report(&w, &records, &lows, &accidents);
}

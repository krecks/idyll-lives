//! The world's own tick.

use super::*;
use crate::clock::{SUPER_BUSY, SUPER_SPEED, TICKS_PER_STEP};
use crate::content::Pose;
use crate::path::NavGrid;
use crate::world::{
    BroadcastScratch, BroadcastSource, Ctx, GROWTH_STEP_MINUTES, Others, Phase, ROOM_REFRESH_TICKS,
    SimBrief, TICKS_PER_DAY, TICKS_PER_HOUR, Task, TaskKind, end_activity, held_by, leaves_by,
};
use crate::{TICKS_PER_MINUTE, World, conversation, social};

impl World {
    /// Advances one real-time step (1 / TICKS_PER_SECOND s) at the current speed; at super
    /// speed, all of its most ticks (60 while it stays quiet at home, else `SUPER_BUSY`).
    pub fn advance(&mut self) {
        self.start_step();
        while self.advance_more() {}
    }

    /// Starts a real-time step: the speed's ticks, or at super speed none yet (`advance_more`
    /// runs them, as many as the caller has time for). With `auto_fast` on, the game goes to super
    /// speed as the household falls quiet, and drops back to the speed chosen before it once it
    /// isn't quiet at home (a super speed the player picked stays). The ticks it ran.
    pub fn start_step(&mut self) -> u32 {
        self.trails.begin_step(self.step_ticks);
        self.step_ticks = 0;
        if self.speed == 0 {
            return 0;
        }
        let calm = self.calm();
        if self.speed == SUPER_SPEED && self.auto_super && !calm {
            self.speed = self.resume_speed;
            self.auto_super = false;
        } else if self.auto_fast && calm && !self.was_calm && self.speed < SUPER_SPEED {
            self.resume_speed = self.speed;
            self.speed = SUPER_SPEED;
            self.auto_super = true;
        }
        self.was_calm = calm;
        if self.speed == SUPER_SPEED {
            return 0;
        }
        let ticks = TICKS_PER_STEP[self.speed as usize];
        for tick in 1..=ticks {
            self.tick_once();
            self.record_trails(tick);
        }
        self.step_ticks = ticks;
        ticks
    }

    /// One more tick of a super-speed step, if the game is at super speed and the step hasn't run
    /// its most: `TICKS_PER_STEP[SUPER_SPEED]` while it's quiet at home, else `SUPER_BUSY` (none
    /// more once the quiet ends, for a super speed gone to by itself: it drops back at the next
    /// step). Whether it ran.
    pub fn advance_more(&mut self) -> bool {
        if self.speed != SUPER_SPEED || self.step_ticks >= TICKS_PER_STEP[SUPER_SPEED as usize] {
            return false;
        }
        // (The step's start looked before its first tick; under 30 a step there is nothing to ask.)
        if (self.step_ticks >= SUPER_BUSY || (self.auto_super && self.step_ticks > 0))
            && !self.calm()
        {
            return false;
        }
        self.tick_once();
        self.step_ticks += 1;
        self.record_trails(self.step_ticks);
        true
    }

    /// The cars' poses after tick `tick` of the step under way, for their trails (`car::TRAIL`):
    /// those out of sight have none (they aren't drawn: `cars::hidden`), so a car's way starts
    /// where it came into sight.
    fn record_trails(&mut self, tick: u32) {
        let shown = self.cars.iter().map(|c| {
            c.shown
                .filter(|_| c.sight == crate::cars::Sight::Seen)
                .map(|s| (s.0, s.1))
        });
        self.trails.record(tick, shown);
    }

    /// Ticks a real step runs now: the speed's, or at super speed what its last step ran.
    pub fn step_rate(&self) -> u32 {
        if self.speed == SUPER_SPEED {
            self.step_ticks.max(1)
        } else {
            TICKS_PER_STEP[self.speed as usize]
        }
    }

    /// A baby crying (content `dayRhythm.babyWakes`) in the crib at home with every grown-up of
    /// the household there asleep wakes one of them: family first, then someone not worn out,
    /// then whoever's turn it is (woken longest ago: partners take turns), then the most rested.
    /// At night, a grown-up there who's up and free to drop what they're doing
    /// (`free_for_baby`) comes at once instead (a cook nearly done with a step finishes it
    /// first): family first, then the nearest; with nobody up who can (in the shower, told to
    /// do something, about to leave for work), one asleep is woken. They do the crib's care for
    /// each need that's low (the woken go back to bed after: night still picks it); and the same
    /// baby calls nobody again for a while. In the daytime, with someone at home awake, they see
    /// to the baby themselves (autonomy, `pick_autonomous`).
    fn wake_for_crying_babies(&mut self) {
        let Some(rules) = self.content.day_rhythm.baby_wakes else {
            return;
        };
        for b in 0..self.sims.len() {
            let Some((carer, tasks)) = self.woken_for(b, &rules) else {
                continue;
            };
            let content = &self.content;
            let sim = &mut self.sims[carer];
            let woken = sim.asleep(content);
            // A cook halfway through a step finishes it first (`cooking::called_away`).
            if !crate::cooking::called_away(sim, content, &mut self.objects) {
                end_activity(sim, content, &mut self.objects);
            }
            sim.follow_up = None;
            let care = tasks.len();
            for task in tasks.into_iter().rev() {
                sim.queue.push_front(task);
            }
            trim_queue(&mut sim.queue, care);
            if woken {
                sim.night_woken = self.tick;
                if let Some(f) = rules.feeling {
                    social::add_feeling(&mut sim.feelings, f, &content.feelings, self.tick);
                }
            }
            self.sims[b].night_woken = self.tick;
        }
    }

    /// Who sees to baby `b`'s crying (`wake_for_crying_babies`), and the care they do.
    fn woken_for(&self, b: usize, rules: &crate::content::BabyWakes) -> Option<(usize, Vec<Task>)> {
        let content = &self.content;
        let baby = &self.sims[b];
        let cooling = baby.night_woken != 0 && self.tick < baby.night_woken + rules.cooldown_ticks;
        if !baby.here() || baby.away_until.is_some() || !content.life.baby(baby.age) || cooling {
            return None;
        }
        let wants = |n: usize, below: f32| rules.needs & (1 << n) != 0 && baby.needs[n] < below;
        if !(0..content.needs.len()).any(|n| wants(n, rules.below)) {
            return None;
        }
        let h = baby.household as usize;
        let plot = self.households[h]
            .plot
            .filter(|_| self.autonomy && self.households[h].free_will)?;
        // In a crib at home, with care for what they're crying for (of what they hold a place
        // at, not every object in town: a baby nobody comes for asks this every tick).
        let crib =
            held_by(baby, &self.objects).find(|o| self.object_plot[o.id as usize] == Some(plot))?;
        let def = &content.objects[crib.def];
        let care = |i: &crate::content::Interaction, below: f32| {
            i.cares() && (0..content.needs.len()).any(|n| i.care[n] > 0.0 && wants(n, below))
        };
        if !def.interactions.iter().any(|i| care(i, rules.below)) || !crib.has_free_slot(def.slots)
        {
            return None;
        }
        // The grown-ups at home.
        let home: Vec<usize> = (0..self.sims.len())
            .filter(|&j| {
                let s = &self.sims[j];
                let (x, z) = s.tile();
                s.here()
                    && s.household as usize == h
                    && s.away_until.is_none()
                    && s.visiting.is_none()
                    && s.adult(content)
                    && self.plot_at(x, z) == Some(plot)
            })
            .collect();
        // Nothing to do if someone's already doing the crib's care (or on their way to).
        let tending = |t: &Task| matches!(t.kind, TaskKind::Use { object, interaction } if object == crib.id && def.interactions[interaction].cares());
        if home.iter().any(|&j| {
            self.sims[j].current().is_some_and(|a| tending(&a.task))
                || self.sims[j].queue.iter().any(tending)
        }) {
            return None;
        }
        let carer = if home.iter().all(|&j| self.sims[j].asleep(content)) {
            self.to_wake(b, &home)?
        } else {
            // Up at night: whoever's free drops what they're doing; by day, autonomy.
            let rhythm = &content.day_rhythm;
            if !rhythm.is_night(crate::clock::hour(self.tick), rhythm.wake_hour) {
                return None;
            }
            let family = |j: usize| self.relationships.kin(j, b) != crate::social::Kin::None;
            let (cx, cz) = crib.front_tile(content);
            let far = |j: usize| {
                let p = self.sims[j].pos;
                (p[0] - cx as f32 - 0.5).powi(2) + (p[1] - cz as f32 - 0.5).powi(2)
            };
            let up = home
                .iter()
                .copied()
                .filter(|&j| self.free_for_baby(j, rules))
                .max_by(|&x, &y| family(x).cmp(&family(y)).then(far(y).total_cmp(&far(x))));
            up.or_else(|| self.to_wake(b, &home))?
        };
        let tasks = def
            .interactions
            .iter()
            .enumerate()
            .filter(|(_, i)| care(i, rules.tend_below))
            .map(|(i, _)| Task {
                kind: TaskKind::Use {
                    object: crib.id,
                    interaction: i,
                },
                directed: false,
                routine: false,
            })
            .collect();
        Some((carer, tasks))
    }

    /// Which of the grown-ups at `home` who are asleep baby `b`'s crying wakes
    /// (`wake_for_crying_babies`).
    fn to_wake(&self, b: usize, home: &[usize]) -> Option<usize> {
        let content = &self.content;
        let rhythm = &content.day_rhythm;
        let rest = |j: usize| rhythm.rest_need.map_or(1.0, |n| self.sims[j].needs[n]);
        let key = |j: usize| {
            let family = self.relationships.kin(j, b) != crate::social::Kin::None;
            (
                family,
                rest(j) >= rhythm.exhausted_below,
                std::cmp::Reverse(self.sims[j].night_woken),
            )
        };
        home.iter()
            .copied()
            .filter(|&j| self.sims[j].asleep(content))
            .max_by(|&x, &y| key(x).cmp(&key(y)).then(rest(x).total_cmp(&rest(y))))
    }

    /// Whether grown-up `j`, up at home, can see to a crying baby now: doing nothing, or
    /// something they picked themselves (on their way to bed too; eating, the plate left where
    /// it is; cooking, with at most `BabyWakes::finish_minutes` of the step left, see
    /// `cooking::called_away`; not the toilet or shower, washing up, or a conversation), with
    /// nothing the player asked for next, not pressed themselves, and not leaving for work or
    /// school within `DayRhythm::ready_minutes`.
    fn free_for_baby(&self, j: usize, rules: &crate::content::BabyWakes) -> bool {
        let content = &self.content;
        let sim = &self.sims[j];
        if sim.asleep(content) || sim.engaged_with.is_some() || sim.pressed(content) {
            return false;
        }
        let picked = sim.current().is_none_or(|a| {
            let drop = match a.task.kind {
                TaskKind::Use { .. }
                | TaskKind::MoveTo { .. }
                | TaskKind::Clean { .. }
                | TaskKind::Eat { .. } => true,
                TaskKind::Cook { .. } => {
                    crate::cooking::step_left(sim).is_some_and(|m| m <= rules.finish_minutes)
                }
                _ => false,
            };
            !a.task.directed && drop && a.tags & content.social_rules.busy_tags == 0
        });
        let next = sim
            .queue
            .front()
            .is_some_and(|t| t.directed || matches!(t.kind, TaskKind::Work));
        let ready = social::minutes_to_ticks(content.day_rhythm.ready_minutes);
        let leaving = leaves_by(sim, content, self.tick, self.tick + ready);
        picked && !next && !leaving
    }

    pub fn tick_once(&mut self) {
        self.sync_tile_arrays();
        // Time moves on: edits can no longer be taken back.
        self.undo.clear();
        self.redo.clear();
        // (An edit still under way can't be taken back now: nothing more to keep of it.)
        self.lot.log_off();
        if self.rooms_version != self.room_layout_version
            || self.rooms_dirty
            || self.rooms_things
            || self.rooms_rescore
        {
            crate::profile::phase("update_rooms", || self.update_rooms());
        } else if self.tick.is_multiple_of(ROOM_REFRESH_TICKS) {
            crate::profile::phase("rescore_rooms", || self.rescore_rooms());
        }
        crate::profile::phase("refresh_reach", || self.refresh_reach());
        crate::profile::phase("refresh_index", || self.refresh_index());
        self.travel.note_doors(&self.plots);
        // A new game's first step: the player hears of anyone without a bed (told as the world was
        // made, it would be the story so far, which the view doesn't show as news).
        if self.tick == 0 {
            for h in 0..self.households.len() {
                crate::lifecycle::notify_missing_beds(self, h);
            }
        }
        self.tick += 1;
        let tick = self.tick;
        self.search_mark = crate::path::nodes_searched();

        crate::profile::phase("partners::refresh", || crate::partners::refresh(self));
        crate::profile::phase("staff::update", || crate::staff::update(self));
        self.build_briefs(tick);
        crate::profile::phase("privacy::refresh", || crate::privacy::refresh(self));
        crate::profile::phase("lod::plan", || crate::lod::plan(self));
        let steps_start = crate::profile::start();
        // Those who go first when the tick's searches run over budget: the player's household
        // and whoever is watched (see `SEARCH_BUDGET`).
        // (At full detail, as `lod::plan` found them: the player's household always is.)
        let first = std::mem::take(&mut self.first);
        let (search_mark, budget) = (self.search_mark, self.search_budget);
        let hour = crate::clock::hour(tick);
        let mut work = std::mem::take(&mut self.work);
        self.fill_work_waiting(&mut work);
        let room_in_town = self.room_in_town();
        {
            let World {
                content,
                lot,
                objects,
                items,
                sims,
                blocked,
                rng,
                relationships,
                briefs,
                members,
                autonomy,
                households,
                plots,
                tile_plot,
                halls,
                object_plot,
                object_index,
                exits,
                travel,
                cars,
                traffic,
                taxis,
                room_score,
                dirt,
                stairs,
                reach,
                nav_version,
                nav_log,
                shut,
                rooms,
                staff,
                paths,
                ..
            } = self;
            let base = Ctx {
                room_score,
                dirt,
                content,
                nav: NavGrid {
                    lot,
                    blocked,
                    stairs,
                    kept_off: &travel.junction_tiles,
                    steps: Some(reach),
                },
                reach,
                nav_version: *nav_version,
                nav_log,
                rels: relationships,
                briefs,
                members,
                households,
                plots,
                tile_plot,
                halls,
                object_plot,
                index: object_index,
                exits,
                travel,
                cars,
                lights: &traffic.lights,
                taxis,
                autonomy: *autonomy,
                room_in_town,
                hour,
                day: crate::clock::day(tick),
                minute: crate::clock::minute_of_day(tick),
                tick,
                work: &work,
                shut,
                rooms,
                staff,
                paths,
                others: Others::default(),
            };
            // (In lots nobody watches, only those whose turn it is; see `lod.rs`.)
            for i in 0..sims.len() {
                let (before, rest) = sims.split_at_mut(i);
                let Some((sim, after)) = rest.split_first_mut() else {
                    break;
                };
                if !sim.here() || sim.steps == 0 {
                    continue;
                }
                let ctx = Ctx {
                    others: Others { before, after },
                    ..base
                };
                // The tick's searches over budget: one who may look for a way now waits for
                // the next tick, unless they go first or waited last tick.
                if first.get(sim.id as usize) == Some(&false)
                    && !sim.deferred
                    && crate::path::nodes_searched() - search_mark > budget
                    && may_search(sim, &ctx)
                {
                    sim.owed += sim.steps;
                    sim.steps = 0;
                    sim.deferred = true;
                    continue;
                }
                sim.deferred = false;
                step_sim(sim, &ctx, objects, items, rng);
            }
        }
        // A serving promised to someone who is still fetching or eating it stays theirs; a task
        // that ended or was replaced must not keep it out of reach.
        self.items.release_stale(|item, sim| {
            self.sims.get(sim as usize).is_some_and(|s| {
                s.current().into_iter().chain(s.paused()).any(
                    |a| matches!(a.task.kind, TaskKind::Eat { item: Some(i), .. } if i == item),
                )
            })
        });
        self.work = work;
        self.first = first;
        crate::profile::end("step_sim", steps_start);
        crate::profile::phase("partners::update", || crate::partners::update(self));
        crate::profile::phase("broadcast", || self.broadcast());
        crate::profile::phase("apply_messes", || self.apply_messes());
        crate::profile::phase("carry::after_step", || crate::carry::after_step(self));
        crate::profile::phase("setup::update", || crate::setup::update(self));
        crate::profile::phase("post::after_step", || crate::post::after_step(self));
        crate::profile::phase("wake_for_crying_babies", || self.wake_for_crying_babies());
        crate::profile::phase("conversation::update", || conversation::update(self));
        crate::profile::phase("privacy::after_step", || crate::privacy::after_step(self));
        crate::profile::phase("hosting::update", || crate::hosting::update(self));
        crate::profile::phase("phone::update", || crate::phone::update(self));
        crate::profile::phase("life::update", || crate::life::update(self));
        crate::profile::phase("lifecycle::update", || crate::lifecycle::update(self));
        crate::profile::phase("traffic::begin", || crate::traffic::begin(self));
        crate::profile::phase("cars::update", || crate::cars::update(self));
        crate::profile::phase("taxi::update", || crate::taxi::update(self));
        crate::profile::phase("planner::update", || crate::planner::update(self));
        crate::profile::phase("cooking::after_step", || crate::cooking::after_step(self));
        crate::profile::phase("lights::update", || crate::lights::update(self));
        crate::profile::phase("lights::update_while", || crate::lights::update_while(self));
        crate::profile::phase("curtains::update", || crate::curtains::update(self));
        if self.tick.is_multiple_of(TICKS_PER_HOUR) {
            crate::rooms::opinions(self);
        }

        self.update_after_step(tick);
    }

    /// The tick's `SimBrief` for each resident, into `self.briefs` (`members`, and those near
    /// them, `tick_once`).
    fn build_briefs(&mut self, tick: u64) {
        let briefs_start = crate::profile::start();
        let mut briefs = std::mem::take(&mut self.briefs);
        briefs.clear();
        for s in &self.sims {
            let (x, z) = s.tile();
            let plot = self.plot_at(x, z).filter(|_| s.here());
            let hall = plot.is_some_and(|p| self.halls.at(p, x, self.ground_z(z)).is_some());
            let available = s.here() && s.available_for_social(&self.content);
            briefs.push(SimBrief {
                pos: s.pos,
                available,
                gender_ix: s.gender_ix,
                household: s.household,
                plot,
                away: s.away_until.is_some() || !s.here(),
                leaving: crate::life::leaving_home(self, s),
                baby: s.here() && self.content.life.baby(s.age),
                needs: s.needs,
                here: s.here(),
                partnered: self.relationships.partner_of(s.id as usize).is_some(),
                adult: s.adult(&self.content),
                age: s.age,
                conversing: None,
                cleaning: s.current().and_then(|a| match a.task.kind {
                    TaskKind::Clean { x, z } => Some((x, z)),
                    TaskKind::Use { .. } => s.sweep,
                    _ => None,
                }),
                seated: s.pose == Pose::Sit && !s.is_moving(),
                lying: s.pose == Pose::Lie,
                eating: s.eating_seated(),
                game: s.game,
                // (`partners::free_now`, its start just above.)
                free: {
                    let free = available && crate::partners::free_besides(&self.content, s, tick);
                    debug_assert_eq!(free, crate::partners::free_now(&self.content, s, tick));
                    free
                },
                bound_for_car: s.bound_for_car.filter(|_| {
                    s.current()
                        .is_some_and(|a| matches!(a.task.kind, TaskKind::Work))
                }),
                trip_car: s.bound_for_car.filter(|_| {
                    s.current().is_some_and(|a| {
                        matches!(a.task.kind, TaskKind::Visit { .. } | TaskKind::GoHome)
                    })
                }),
                bike: s.commute.and_then(|c| c.bike),
                bike_spot: s.bike_spot.map(|(at, _)| at),
                guest_at: s.visiting.map(|v| v.plot),
                eats_at: match s.current().map(|a| a.task.kind) {
                    Some(TaskKind::Eat { seat, stage: 1, .. }) => seat,
                    _ => None,
                },
                carried: s.carried_by.is_some(),
                private: crate::privacy::private_use(&self.content, &self.objects, s),
                hall,
            });
        }
        // Who's in a conversation, and whose (for joining in).
        for (i, s) in self.sims.iter().enumerate() {
            if let Some((t, social, ..)) = s.conversation(&self.content) {
                briefs[i].conversing = Some((i as u32, social));
                briefs[t as usize].conversing = Some((i as u32, social));
            }
        }
        for (i, s) in self.sims.iter().enumerate() {
            if let Some(l) = s.listening_to() {
                briefs[i].conversing = briefs[l as usize].conversing.filter(|c| c.0 == l);
            }
        }
        self.members.build(&briefs);
        self.briefs = briefs;
        crate::profile::end("briefs", briefs_start);
    }

    /// The tick's tail: feelings fade and buffs refresh, relationships decay by the day, crops
    /// grow and machines run (`tick_once`).
    fn update_after_step(&mut self, tick: u64) {
        for s in self.sims.iter_mut().filter(|s| s.steps > 0) {
            s.feelings.retain(|m| m.expires > tick);
            if self.content.buff_feelings {
                s.refresh_buffs(&self.content);
            }
        }
        if tick.is_multiple_of(TICKS_PER_DAY) {
            self.relationships.decay(&self.content.social_rules);
        }
        if tick.is_multiple_of(GROWTH_STEP_MINUTES * TICKS_PER_MINUTE) {
            self.grow(GROWTH_STEP_MINUTES as f32);
        }
        if tick.is_multiple_of(crate::machines::STEP_MINUTES * TICKS_PER_MINUTE) {
            crate::machines::run(self, crate::machines::STEP_MINUTES as f32);
        }
    }

    /// Gives nearby residents a share of what a use gives (content `broadcast`: hearing the TV
    /// from the sofa, or while eating at the table). Only positive gains carry, and not to the
    /// resident using it, who gets the whole of it.
    pub(crate) fn broadcast(&mut self) {
        let mut scratch = std::mem::take(&mut self.broadcast_scratch);
        self.broadcast_with(&mut scratch);
        self.broadcast_scratch = scratch;
    }

    /// `broadcast_with`'s sources: what each resident using something that carries
    /// (`broadcast`) is using, where, and what it gives.
    fn broadcast_sources(&self, sources: &mut Vec<BroadcastSource>) {
        let content = &self.content;
        sources.clear();
        for sim in &self.sims {
            if !sim.here() || sim.away_until.is_some() {
                continue;
            }
            let Some(act) = sim.current() else { continue };
            if !matches!(act.phase, Phase::Using { .. }) {
                continue;
            }
            let TaskKind::Use {
                object,
                interaction,
            } = act.task.kind
            else {
                continue;
            };
            let Some(obj) = self.objects.get(object as usize) else {
                continue;
            };
            let inter = &content.objects[obj.def].interactions[interaction];
            if inter.broadcast <= 0.0 {
                continue;
            }
            let pos = obj.centre(content);
            let (x, z) = (pos[0].floor() as i32, pos[1].floor() as i32);
            let ground = self.ground_z(z);
            let room = if self.lot.in_bounds(x, ground) {
                self.lot.room_at(x, ground)
            } else {
                crate::lot::OUTDOORS
            };
            let quality = content.object_rules.quality_factor(obj.quality);
            sources.push(BroadcastSource {
                object,
                room,
                storey: self.storey_of(z),
                plot: self.object_plot.get(object as usize).copied().flatten(),
                pos,
                gains: inter.gain_per_minute.map(|g| g * quality),
                share: inter.broadcast,
                range: inter.broadcast_range,
                user: sim.id,
            });
        }
    }

    /// `broadcast` with its buffers (`BroadcastScratch`) out of `self`.
    fn broadcast_with(&mut self, scratch: &mut BroadcastScratch) {
        let BroadcastScratch {
            sources,
            rooms,
            storeys,
            plots,
            asleep,
            heard,
        } = scratch;
        self.broadcast_sources(sources);
        if sources.is_empty() {
            return;
        }
        let content = &self.content;
        rooms.clear();
        rooms.extend(self.sims.iter().map(|s| {
            let (x, z) = s.tile();
            let ground = self.ground_z(z);
            if self.lot.in_bounds(x, ground) {
                self.lot.room_at(x, ground)
            } else {
                crate::lot::OUTDOORS
            }
        }));
        storeys.clear();
        storeys.extend(self.sims.iter().map(|s| self.storey_of(s.tile().1)));
        plots.clear();
        plots.extend(self.sims.iter().map(|s| {
            let (x, z) = s.tile();
            self.plot_at(x, z)
        }));
        asleep.clear();
        asleep.extend(self.sims.iter().map(|s| s.asleep(content)));
        for (i, sim) in self.sims.iter_mut().enumerate() {
            if !sim.here() || sim.away_until.is_some() || sim.steps == 0 || asleep[i] {
                continue;
            }
            let minutes = sim.minutes();
            // Two residents watching the same screen are one source, not two.
            heard.clear();
            for src in sources.iter() {
                if src.user == sim.id
                    || heard.contains(&src.object)
                    || (src.plot.is_some() && plots[i] != src.plot)
                {
                    continue;
                }
                // Another resident watching the same screen has their own share of it already.
                if let Some(act) = sim.current()
                    && let TaskKind::Use { object, .. } = act.task.kind
                    && object == src.object
                {
                    continue;
                }
                if storeys[i] != src.storey {
                    continue;
                }
                let same_room = src.room != crate::lot::OUTDOORS && rooms[i] == src.room;
                let near = (sim.pos[0] - src.pos[0]).hypot(sim.pos[1] - src.pos[1]) <= src.range;
                if !same_room && !near {
                    continue;
                }
                heard.push(src.object);
                for n in 0..content.needs.len() {
                    let gain = src.gains[n] * src.share * minutes * sim.mods.need_gain[n];
                    if gain > 0.0 {
                        sim.needs[n] = (sim.needs[n] + gain).clamp(0.0, 1.0);
                    }
                }
            }
        }
    }

    /// Crops grow by `minutes`: fallow ground is planted again, what's planted ripens. A crop
    /// that shows another stage changes how the lot looks.
    fn grow(&mut self, minutes: f32) {
        let mut changed = false;
        for o in &mut self.objects {
            let Some(g) = self.content.objects[o.def].grows else {
                continue;
            };
            if o.growth >= 1.0 {
                continue;
            }
            let before = o.growth_stage();
            o.growth = if o.growth < 0.0 {
                // Fallow, then planted afresh (from nothing: 0).
                let fallow = o.growth + minutes / (g.fallow_days.max(1.0 / 1440.0) * 1440.0);
                if fallow >= 0.0 { 0.0 } else { fallow }
            } else {
                (o.growth + minutes / (g.days * 1440.0)).min(1.0)
            };
            changed |= o.growth_stage() != before;
        }
        if changed {
            self.structure_version = self.structure_version.wrapping_add(1);
            self.rooms_need_clean_rescore();
        }
        self.pile_laundry(minutes);
    }

    /// Dirty washing piles up over `minutes`: each household's, as much as content `laundry` says
    /// for each of its residents in town, up to the most it says.
    fn pile_laundry(&mut self, minutes: f32) {
        let Some(rules) = self.content.laundry else {
            return;
        };
        for s in self.sims.iter().filter(|s| s.here()) {
            if let Some(h) = self.households.get_mut(s.household as usize) {
                h.laundry[0] = (h.laundry[0] + rules.per_day * minutes / 1440.0).min(rules.most);
            }
        }
    }

    /// What each household has to work on (`WorkOn` bits, by household): dirty dishes or food
    /// gone off of theirs, a load of dirty washing, a load washed.
    pub(crate) fn work_waiting(&self) -> Vec<u8> {
        let mut work = Vec::new();
        self.fill_work_waiting(&mut work);
        work
    }

    /// `work_waiting`, into `work`.
    fn fill_work_waiting(&self, work: &mut Vec<u8>) {
        use crate::content::WorkOn;
        work.clear();
        work.extend(self.households.iter().map(|h| {
            let has = |kind: WorkOn, loads: f32| if loads >= 1.0 { kind.bit() } else { 0 };
            has(WorkOn::Laundry, h.laundry[0])
                | has(WorkOn::Washed, h.laundry[1])
                | has(WorkOn::Dried, h.laundry[2])
        }));
        for it in self
            .items
            .list
            .iter()
            .filter(|it| it.to_clear(&self.content))
        {
            if let Some(w) = work.get_mut(it.household as usize) {
                *w |= WorkOn::Dishes.bit();
            }
        }
    }
}

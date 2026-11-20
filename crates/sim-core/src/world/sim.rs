//! A resident, and the set of them: what they are, wear and carry, and what they can do.

use super::*;

/// The most residents a town holds (a town file may start with this many; saves, slots, the
/// snapshot's layout). Sized to the largest Big world filled: a city with many villages has up to
/// about 390 homes, which their households fill to 700 or more, so a full town (680 neighbours)
/// leaves room for the player's household and births. A town file may ask for fewer
/// (`residentsMost`, `World::residents_most`).
pub const MAX_SIMS: usize = 768;

/// A set of residents by slot (`MAX_SIMS` of them, a bit each).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct SimSet([u64; SIM_WORDS]);

/// Words in a `SimSet`.
pub(crate) const SIM_WORDS: usize = MAX_SIMS.div_ceil(64);

impl SimSet {
    pub fn insert(&mut self, sim: u32) {
        if let Some(w) = self.0.get_mut(sim as usize / 64) {
            *w |= 1 << (sim % 64);
        }
    }

    pub fn remove(&mut self, sim: u32) {
        if let Some(w) = self.0.get_mut(sim as usize / 64) {
            *w &= !(1 << (sim % 64));
        }
    }

    pub fn contains(&self, sim: u32) -> bool {
        self.0
            .get(sim as usize / 64)
            .is_some_and(|w| w & (1 << (sim % 64)) != 0)
    }

    /// Whether anyone but `sim` is in it.
    pub fn any_but(&self, sim: u32) -> bool {
        let mut rest = *self;
        rest.remove(sim);
        rest.0.iter().any(|&w| w != 0)
    }

    /// The set's words, lowest residents first (for fingerprints).
    pub fn words(&self) -> &[u64] {
        &self.0
    }
}
pub const MAX_QUEUE: usize = 6;
/// How many things a resident remembers they couldn't get to (`Sim::unreachable`).
pub(crate) const UNREACHABLE: usize = 4;
/// How long a resident remembers that someone was out of reach: people move, so not long (half a
/// real second: as long as the one they wanted takes to walk a little way).
pub(crate) const PERSON_UNREACHABLE_TICKS: u64 = crate::secs(0.5);
/// Walking speed in tiles per tick (1.5 m per real second at 1×).
pub(crate) const WALK_SPEED: f32 = 1.5 / TICKS_PER_SECOND as f32;
/// Idle time before a Sim picks something on its own: a beat seen standing there, a real second
/// (counted in their steps, so a turn at lower detail covers it, `lod.rs`). Those listening in on a
/// talk wait longer for its next social (`conversation::REGROUP_TICKS`).
pub(crate) const AUTONOMY_DELAY_TICKS: u32 = crate::secs(1.0) as u32;
/// Getting up from the upper bunk: the time to climb down its ladder (the renderer's `bunkDown`, 6.5 s), in
/// which they do nothing else (`Sim::ladder_ticks`).
pub(crate) const LADDER_DOWN_TICKS: u32 = crate::secs(6.5) as u32;
/// Stepping aside out of a car's way (`cars::make_way`): at most this far in one step (m).
pub(crate) const SIDESTEP_MOST: f32 = 0.3;
pub(crate) const MIN_AUTONOMY_SCORE: f32 = 0.01;
/// How much less a meal nobody at home would eat tempts free will (the cooking practice it
/// gives aside): cooked only with nothing much else to do.
pub(crate) const UNWANTED_MEAL: f32 = 0.1;
/// A baby with a need below this cries for someone (and lies awake in the crib).
pub(crate) const BABY_CRIES_BELOW: f32 = 0.3;
pub const TICKS_PER_DAY: u64 = crate::clock::TICKS_PER_DAY;
pub(crate) const TICKS_PER_HOUR: u64 = 60 * TICKS_PER_MINUTE;
/// Rooms are re-scored this often (for dirt), and whenever the structure changes.
pub(crate) const ROOM_REFRESH_TICKS: u64 = 10 * TICKS_PER_MINUTE;
/// Past this many tiles where things changed between ticks, every room is scored again for what
/// stands in it (`World::rooms_things_changed`).
pub(crate) const MOST_THINGS_AT: usize = 64;

/// How many outfits each resident has to change between (`Sim::outfit`).
pub const OUTFITS: u8 = 3;

#[derive(Debug, Clone)]
pub struct Sim {
    /// Equal to the Sim's index in `World::sims`.
    pub id: u32,
    pub name: String,
    /// Opaque appearance data for the renderer/UI.
    pub appearance: serde_json::Value,
    /// Which of their `OUTFITS` they're dressed in (0: the one in `appearance`, as made in the
    /// creator; the renderer makes the others from the id): the next one once they change at
    /// a wardrobe (`Interaction::changes_outfit`), kept until they change again.
    pub outfit: u8,
    pub gender: String,
    /// Genders this Sim is romantically attracted to.
    pub attracted_to: Vec<String>,
    pub traits: Vec<String>,
    pub perks: Vec<String>,
    /// Combined effect of traits and perks.
    pub base_mods: Modifiers,
    /// Effective modifiers: `base_mods` combined with the effects of active feelings.
    pub mods: Modifiers,
    /// Feelings (defs, sorted) whose effects are folded into `mods`.
    pub(crate) buffs: Vec<usize>,
    /// Money the household owes for interactions started this tick (settled by `life::update`).
    pub(crate) pending_spend: i64,
    /// What they took up to work on this tick (`WorkOn` bits: a load of washing), settled with
    /// the household by `life::update`.
    pub(crate) worked: u8,
    /// Loads of washing they left to work on next this tick (`WorkOn` bits: the wet washing out
    /// of the washer; `Interaction::makes`), settled with `worked`.
    pub(crate) made: u8,
    /// The load of washing taken up for the use they're at (`WorkOn` bits): given back if the use
    /// doesn't get done (cancelled, called away: `end_activity`), none once it is
    /// (`machines::use_done`). Not saved: a save gives it back (a reloaded use starts again).
    pub(crate) taken: u8,
    pub pos: [f32; 2],
    pub yaw: f32,
    pub pose: Pose,
    /// The tile they sat or lay down from (where they stand back up): the tile an object is
    /// used from can change while they're on it (see `surface::approach_tile`).
    pub(crate) seated_from: Option<(i32, i32)>,
    /// The seat they took to use another object (content `seatNear`: the sofa while watching
    /// TV). Not saved: the task finds a seat again after a load.
    pub use_seat: Option<u32>,
    pub needs: [f32; MAX_NEEDS],
    pub feelings: Vec<ActiveFeeling>,
    /// Set while another Sim is talking to this one.
    pub engaged_with: Option<u32>,
    /// Someone called out their name: they turned to them and are on their way to meet them, or
    /// wait for them where they met (see `conversation::update`). Not saved.
    pub heeding: Option<u32>,
    pub(crate) gender_ix: Option<usize>,
    pub(crate) attraction: u32,
    pub(crate) queue: VecDeque<Task>,
    pub(crate) current: Option<Activity>,
    /// Sitting, what they sat down to do, held while they talk from the seat (a chat with the
    /// one beside them on the sofa, listening in at the table; `conversation::from_seat`): they
    /// carry on with it when the talk is over, still sitting (`end_activity`). Not saved: after
    /// a reload they stand up where they sat down from.
    pub(crate) paused: Option<Activity>,
    /// Joining in: the place in the group they're walking to (`conversation::take_place`). Not
    /// saved, as listening in isn't.
    pub(crate) join_spot: Option<[f32; 2]>,
    /// Ticks without anything to do (to choose something); sitting at something that leaves
    /// room for a word, ticks since they last thought of one (`from_seat`).
    pub(crate) idle_ticks: u32,
    /// What they do next, before choosing anything else (an `after` routine, like washing
    /// hands after the toilet; see `follow_up`). Not saved: losing it on a reload is harmless.
    pub(crate) follow_up: Option<Task>,
    /// The object they fetched what a use there needs for (`Interaction::fetch`: the coffee
    /// poured at the machine for the table), so that use, the `follow_up`, goes ahead without
    /// fetching it again. Any other task started clears it. Not saved.
    pub(crate) fetched: Option<u32>,
    /// The last few things they chose for themselves but couldn't get to (a plate on a counter
    /// nobody can reach, a bed behind a wall, a house across a river), each with until when they
    /// don't choose it again: uses, eating, cleaning, repairs, washing up, visits, and people
    /// for a short while (see `remember_unreachable`; object ids follow `remap_unreachable`).
    /// Not saved either.
    pub(crate) unreachable: [Option<(TaskKind, u64)>; UNREACHABLE],
    /// Ticks the social they're on has spent waiting for its target, and following them since
    /// it first got there (see `conversation::update`). Not saved.
    pub(crate) social_waited: u32,
    /// They called out to the target of the social they're on already (once per social). Not saved.
    pub(crate) called: bool,
    /// The `World::nav_version` their route was last checked against (see `step_sim`): a
    /// resident nobody watches takes a turn only every so often, and a change between two of
    /// them still catches up with them. Not saved (after a load, a walker routes again once).
    pub(crate) nav_seen: u32,
    /// Open-world towns: how long the way to work or school takes them today, firmed up shortly
    /// before they set off (`life::refresh_commute`); None: as `careerRules.commuteMinutes`
    /// says. Not saved.
    pub(crate) commute: Option<crate::life::Commute>,
    /// The trailhead (object id) whose trail they last walked (`trail_step`); they're walking it
    /// while they use it (`on_trail`). Not saved.
    pub(crate) on_trail: Option<u32>,
    /// The game for two (`Interaction::partner`) they're at, or on their way to, next: its
    /// object and interaction, and whether they're at their place. Set as each tick starts
    /// (`partners::refresh`), not saved (the task it comes from is).
    pub(crate) game: Option<(u32, usize, bool)>,
    /// They set off for a game for two this tick: someone to play with is to be found
    /// (`partners::update`). Not saved.
    pub(crate) wants_partner: bool,
    /// Fetched to play with this resident (`partners::update`): the game they're sent to is
    /// that one's, joined, never one of their own. Not saved.
    pub(crate) joins: Option<u32>,
    /// Ticks they've waited at their place for the one they play with to get to theirs (see
    /// `partners::hold`; 0: not waiting). Not saved.
    pub(crate) partner_wait: u32,
    /// At a post's front, being served by its server now (set as each tick starts,
    /// `staff::update`). Not saved.
    pub(crate) being_served: bool,
    /// Ticks they've waited at a post for its server (`staff::hold`). Not saved.
    pub(crate) serve_wait: u32,
    /// The bike (or e-scooter) they're on their way to for work or school, and the one they're
    /// on (`ride.rs`): an object at home, its slot theirs until they're back. Not saved (after a
    /// load it's at its place and they walk).
    pub(crate) bound_for_bike: Option<u32>,
    pub riding: Option<u32>,
    /// Out on a visit or an outing by bike: where they left it by the place (`world::arrive`), the
    /// bike theirs in `bound_for_bike` till they ride it home. Not saved.
    pub(crate) bike_parked: Option<[f32; 2]>,
    /// Where that bike stands by the place (`ride::park_spot`: its middle, which way it faces);
    /// None while it's not worked out yet, or there was no room (then where they got off). In the
    /// snapshot for the renderer. Not saved.
    pub(crate) bike_spot: Option<([f32; 2], f32)>,
    /// When they were last back home from a visit or an outing (`visits.againMinutes`). Not saved.
    pub(crate) back_at: u64,
    /// The car they're on their way to for work or school (`cars::ride_for`), their seat in it (0:
    /// they drive) and the door they get in by (`cars::DOORS`). Not saved (after a load, they set
    /// off again).
    pub(crate) bound_for_car: Option<u32>,
    /// The work taxi they're booked on (`taxi::Booking`): to work or school, or home from it. Not
    /// saved (the runs are: `taxi::settle`).
    pub(crate) taxi: Option<crate::taxi::Booking>,
    pub(crate) car_seat: u8,
    pub(crate) car_door: u8,
    /// Ticks they've waited for a car driving by to pass (`cars::in_the_way`). Not saved.
    pub(crate) car_wait: u32,
    /// The lit junction whose crosswalk they wait at the kerb of, the arm it's on (for its walk
    /// phase, or its main road's green: `Signal::opens_with`), and the tick they last did
    /// (`traffic::kerb_stop`). Not saved.
    pub(crate) kerb_wait: Option<(u16, u8, u64)>,
    /// The work shift (or school day) they're away for starts at this tick: until then they're on
    /// their way (`life::at_work`). Not saved (after a load, it's under way).
    pub(crate) shift_from: u64,
    /// Found nobody to play a game for two with they chose: none chosen again until this tick
    /// (`partners::alone`). Not saved.
    pub(crate) alone_until: u64,
    pub household: u32,
    pub job: Option<crate::life::Job>,
    /// At work until this tick (not on the map).
    pub away_until: Option<u64>,
    /// Currently a guest on another household's plot.
    pub visiting: Option<crate::life::Visit>,
    /// Set by Work/Visit/GoHome tasks; handled by `life::update`.
    pub(crate) transition: Option<crate::life::Transition>,
    /// Skill levels, `0..=max_level` (the whole number is the level shown to players).
    pub skills: [f32; MAX_SKILLS],
    /// Skills that reached a new level this tick (bitmask), reported by `life::update`.
    pub(crate) skill_ups: u32,
    /// Grows while practising and fades otherwise, so Sims don't train for hours on end.
    pub(crate) practice_fatigue: f32,
    /// No nap until this tick (the last one was too recent; see `DayRhythm::nap_cooldown_hours`).
    pub(crate) nap_until: u64,
    /// Up from the upper bunk: the ticks left climbing down its ladder, doing nothing else meanwhile
    /// (`LADDER_DOWN_TICKS`). Not saved: a load finds them down already.
    pub(crate) ladder_ticks: u32,
    /// The first day this Sim looks for work again (after quitting or being let go).
    pub job_search_from: u32,
    /// Routines, goals and how planned blocks went (see `planner`).
    pub planner: crate::planner::Planner,
    /// Mess made (or, negative, cleaned) around a tile this tick; applied after the step
    /// (residents can't change the dirt while stepping).
    pub(crate) mess: Option<(i32, i32, f32)>,
    /// The floor they're taking a tool to, or cleaning with it (`Interaction::cleans`,
    /// `sweep.rs`): where its use is done. Not saved: a use under way starts again after a load,
    /// and finds the dirtiest floor then.
    pub(crate) sweep: Option<(i32, i32)>,
    /// A floor cleaned with a tool this tick: where, and how (applied after the step).
    pub(crate) swept: Option<(i32, i32, crate::content::Sweep)>,
    /// Put on pressed clothes this tick (`content::Pressed`): an outfit's share of the
    /// household's ironed washing taken (settled with the laundry, `life::update`).
    pub(crate) wore_pressed: bool,
    /// Conceived with this partner this tick (the life cycle decides; see `lifecycle::conceive`).
    pub(crate) conceived: Option<u32>,
    /// Their *Try for a baby* with this partner went well this tick: the two go to bed
    /// (`privacy::after_step`). Not saved.
    pub(crate) to_bed: Option<u32>,
    /// Ticks they've stood outside a room kept shut by someone in it, on their way in (a
    /// bathroom in use; `privacy.rs`), 0 while not. Not saved: after a load they set off again.
    pub(crate) kept_out: u32,
    /// Care given this tick: the baby it's for and what it fills (applied after the step).
    pub(crate) care_out: Option<(u32, [f32; MAX_NEEDS])>,
    /// A baby lifted out of their place or put in one this tick (applied after the step,
    /// `carry::after_step`).
    pub(crate) carry_out: Option<crate::carry::CarryOut>,
    /// A baby in someone's arms: who carries them (`carry.rs`). In no place meanwhile, they go
    /// where the one carrying them goes. Not saved (after a load they're back in the crib).
    pub carried_by: Option<u32>,
    /// Opened or closed an object this tick (`Interaction::leaves`), or changed clothes
    /// (`Interaction::changes_outfit`): how the house or they look changed (applied after the
    /// step).
    pub(crate) reshaped: bool,
    /// Lit a fire this tick (sat down by one in the dark: see `lights.rs`): the lamps' state
    /// changed (applied after the step).
    pub(crate) lit: bool,
    /// When they last woke someone crying (a baby), or were woken by a crying baby (a
    /// grown-up), as a tick (0: never; see `World::wake_for_crying_babies`).
    pub(crate) night_woken: u64,
    /// An object they broke or repaired, or an accident they had, this tick (applied after
    /// the step).
    pub(crate) news: Option<News>,
    /// The post read or a letter written this tick (`post::after_step`). Not saved.
    pub(crate) mail: Option<crate::post::Mail>,
    /// Finished putting together something new this tick (an object type; `setup.rs` places it
    /// after the step). Not saved.
    pub(crate) set_up: Option<u32>,
    /// When each accident can happen again (`Content::accidents`).
    pub(crate) accident_at: [u64; crate::content::MAX_ACCIDENTS],
    /// Since when each accident's need has been empty (0: it isn't).
    pub(crate) empty_since: [u64; crate::content::MAX_ACCIDENTS],
    /// Age in years (see `lifecycle.rs`); the life stage follows from it.
    pub age: f32,
    /// No longer in town (the slot stays, so ids don't change; see `World::depart`).
    pub gone: Option<Gone>,
    /// What they carry (ingredients, a plate, dirty dishes; see `cooking.rs`).
    pub held: Option<crate::items::Held>,
    /// Told dinner's ready: keener to eat until this tick.
    pub(crate) invited_until: u64,
    /// Their offer of something to eat or drink was taken up this tick by this resident: they go
    /// and fetch it (`hosting::update`). Not saved.
    pub(crate) serve_for: Option<u32>,
    /// Handed what they hold to this resident this tick (`hosting::update` passes it over). Not
    /// saved.
    pub(crate) hand_to: Option<u32>,
    /// Said yes to a host's offer: something to eat is on its way to them until this tick (or
    /// till it's handed over), so they don't get food of their own meanwhile (`hosting.rs`). Not
    /// saved.
    pub(crate) expecting: u64,
    /// A dish they served this tick (an item; the household is told after the step).
    pub(crate) served: Option<u32>,
    /// A phone action that ended this tick (applied after the step, see `phone::update`).
    pub(crate) hung_up: Option<crate::phone::HungUp>,
    /// The last day they went to (or skipped) school.
    pub school_day: u32,
    /// How they do at school, 0..100 (pupils; see `SchoolRules::letter`).
    pub grade: f32,
    /// Retired from work: doesn't look for a job; the household gets `pension` a week.
    pub retired: bool,
    pub pension: i64,
    /// Ticks they catch up on this tick: 1 at full detail; in a lot nobody watches, 0 between
    /// turns and every tick since the last one on their turn (see `lod.rs`).
    pub(crate) steps: u32,
    /// Ticks since their last turn.
    pub(crate) owed: u32,
    /// Their turn last tick waited for the next, the tick's searches being over budget
    /// (`SEARCH_BUDGET`): this tick they go whatever it costs (`lod::plan` gives them a turn).
    /// Not saved.
    pub(crate) deferred: bool,
}

impl Sim {
    pub fn tile(&self) -> (i32, i32) {
        (self.pos[0].floor() as i32, self.pos[1].floor() as i32)
    }

    /// Walking: on their way somewhere, and not waiting outside a room kept shut (`kept_out`).
    pub fn is_moving(&self) -> bool {
        self.kept_out == 0
            && (matches!(
                self.current,
                Some(Activity {
                    phase: Phase::Routing { .. },
                    ..
                })
            ) || self.on_trail())
    }

    /// Walking a trail (`Content::trail_walks`): shown walking though it's a use.
    pub fn on_trail(&self) -> bool {
        self.on_trail.is_some()
            && matches!(
                self.current,
                Some(Activity {
                    phase: Phase::Using { .. },
                    task: Task {
                        kind: TaskKind::Use { object, .. },
                        ..
                    },
                    ..
                }) if Some(object) == self.on_trail
            )
    }

    /// Waiting outside a room kept shut by someone in it (a bathroom in use; `privacy.rs`).
    pub fn kept_out(&self) -> bool {
        self.kept_out > 0
    }

    pub fn current(&self) -> Option<&Activity> {
        self.current.as_ref()
    }

    /// How long their way to work or school takes today in an open-world town (`life::Commute`).
    pub fn commute(&self) -> Option<crate::life::Commute> {
        self.commute
    }

    /// Eating sitting down (at the table, on the sofa), not talking.
    pub(crate) fn eating_seated(&self) -> bool {
        self.pose == Pose::Sit
            && self.current.as_ref().is_some_and(|a| {
                matches!(
                    (a.task.kind, &a.phase),
                    (TaskKind::Eat { .. }, Phase::Using { .. })
                )
            })
    }

    /// Sitting, what they sat down to do while they talk from the seat (`paused`).
    pub fn paused(&self) -> Option<&Activity> {
        self.paused.as_ref()
    }

    /// The bike (or e-scooter) they're on, or on their way to for work or school (`ride.rs`): its
    /// slot is theirs until they're back.
    pub fn bike(&self) -> Option<u32> {
        self.riding.or(self.bound_for_bike)
    }

    /// In bed for the night (or a nap): using something tagged as sleep.
    pub fn asleep(&self, content: &Content) -> bool {
        self.current.as_ref().is_some_and(|a| {
            a.tags & content.day_rhythm.sleep_tags != 0 && matches!(a.phase, Phase::Using { .. })
        })
    }

    /// A baby in the crib lies awake while they need something (crying for it) or once they're
    /// rested in the daytime, and sleeps otherwise.
    pub fn awake_in_crib(&self, content: &Content, hour: f32) -> bool {
        let rhythm = &content.day_rhythm;
        let crying = (0..content.needs.len())
            .any(|n| !content.needs[n].room && self.needs[n] < BABY_CRIES_BELOW);
        let rested = rhythm.rest_need.is_none_or(|n| self.needs[n] >= 0.999);
        crying || (rested && !rhythm.is_night(hour, rhythm.wake_hour))
    }

    /// Stops whatever they're doing and drops what they meant to do (their object slots are
    /// released by the caller).
    pub(crate) fn clear_activity(&mut self) {
        self.current = None;
        self.queue.clear();
        self.follow_up = None;
        self.fetched = None;
        self.engaged_with = None;
        self.heeding = None;
        self.transition = None;
        self.visiting = None;
        self.pose = Pose::Stand;
        self.seated_from = None;
        self.use_seat = None;
    }

    /// Grown up (may work, fall in love, move, have children).
    pub fn adult(&self, content: &Content) -> bool {
        content.life.adult(self.age)
    }

    /// Still in town (not dead or moved away).
    pub fn here(&self) -> bool {
        self.gone.is_none()
    }

    /// The car they're on their way to (or in, on a trip), if any (`bound_for_car`).
    pub fn car_bound(&self) -> Option<u32> {
        self.bound_for_car
    }

    pub fn queue(&self) -> impl Iterator<Item = &Task> {
        self.queue.iter()
    }

    /// Game minutes their update covers this tick (see `steps`).
    pub(crate) fn minutes(&self) -> f32 {
        MINUTES_PER_TICK * self.steps as f32
    }

    /// Needs average plus trait and feeling effects, 0..1.
    pub fn mood(&self, content: &Content) -> f32 {
        let n = content.needs.len();
        let needs = self.needs[..n].iter().sum::<f32>() / n as f32;
        (needs + self.mods.mood + social::feeling_mood(&self.feelings, &content.feelings))
            .clamp(0.0, 1.0)
    }

    pub fn emotion(&self, content: &Content) -> Option<usize> {
        social::dominant_emotion(&self.feelings, &content.feelings)
    }

    /// How fast they walk, as a share of `WALK_SPEED`: their traits' and life stage's (`mods`)
    /// times their emotion's (content `emotions` `walkSpeed`: slower sad, quicker angry or
    /// energised; the renderer's body language shows the rest).
    pub fn walk_speed(&self, content: &Content) -> f32 {
        self.mods.walk_speed
            * self
                .emotion(content)
                .map_or(1.0, |e| content.emotions[e].mods.walk_speed)
    }

    /// How many metres they walk in a minute (`walk_speed`), as `travel::Travel::eta` asks.
    pub(crate) fn walk_per_minute(&self, content: &Content) -> f32 {
        WALK_SPEED * self.walk_speed(content) * TICKS_PER_MINUTE as f32
    }

    /// Recomputes `mods` when the set of active feelings with effects has changed.
    pub(crate) fn refresh_buffs(&mut self, content: &Content) {
        let defs = &content.feelings;
        let mut active = self
            .feelings
            .iter()
            .filter(|m| defs[m.def].effects.is_some());
        if active.clone().count() == self.buffs.len() && active.all(|m| self.buffs.contains(&m.def))
        {
            return;
        }
        self.buffs.clear();
        self.buffs.extend(
            self.feelings
                .iter()
                .filter(|m| defs[m.def].effects.is_some())
                .map(|m| m.def),
        );
        self.buffs.sort_unstable();
        self.mods = self.base_mods.clone();
        for &m in &self.buffs {
            if let Some(e) = &defs[m].effects {
                self.mods.combine(e);
            }
        }
    }

    pub fn attracted_to(&self, other: &Sim) -> bool {
        match other.gender_ix {
            Some(g) => self.attraction & (1 << g) != 0,
            None => true, // content defines no genders
        }
    }

    /// Whether a need a chat won't help with is about to run out (the bladder): no time to talk.
    pub fn pressed(&self, content: &Content) -> bool {
        let rules = &content.autonomy;
        // (Only the needs in the mask looked at: asked of everyone every tick, `SimBrief`.)
        let mut essentials = rules.urgent & !rules.social_fills;
        while essentials != 0 {
            let n = essentials.trailing_zeros() as usize;
            essentials &= essentials - 1;
            if n >= content.needs.len() {
                break;
            }
            if !content.needs[n].room && self.needs[n] < rules.urgent_below {
                return true;
            }
        }
        false
    }

    /// Whether another Sim can pull this one into a conversation right now.
    pub fn available_for_social(&self, content: &Content) -> bool {
        // (Nor someone at a game for two, or on their way to one: `partners.rs`.)
        if self.engaged_with.is_some()
            || self.heeding.is_some()
            || self.pose == Pose::Lie
            || self.away_until.is_some()
            || self.pressed(content)
            || self.game.is_some()
        {
            return false;
        }
        // Busy with a planned block (unless it's for socialising).
        if let Some(run) = &self.planner.run
            && run.skipped.is_none()
            && !run.no_place
            && !content
                .activities
                .get(run.activity)
                .is_some_and(|a| a.social || a.visit)
        {
            return false;
        }
        let Some(act) = &self.current else {
            return true;
        };
        if act.task.directed {
            return false;
        }
        match (act.task.kind, &act.phase) {
            // People on their way somewhere don't stop to chat (it would strand them), nor do
            // cooks halfway through a recipe or people eating or washing up. A cook on their way
            // to the first station (nothing bought yet, nothing in hand) does.
            (
                TaskKind::Work
                | TaskKind::Visit { .. }
                | TaskKind::GoHome
                | TaskKind::Answer { .. }
                | TaskKind::Serve { .. },
                _,
            ) => false,
            (
                TaskKind::Cook {
                    step: 0,
                    servings: 0,
                    ..
                },
                Phase::Routing { .. },
            ) if self.held.is_none() => true,
            // Eating sitting down: a word over the meal, from their seat (`conversation::keeps_seat`).
            (TaskKind::Eat { .. }, Phase::Using { .. }) if self.pose == Pose::Sit => true,
            (TaskKind::Cook { .. } | TaskKind::Eat { .. } | TaskKind::Dishes { .. }, _) => false,
            (TaskKind::Social { .. } | TaskKind::Join { .. }, Phase::Conversing { .. }) => false,
            // A guest on their way to say goodbye (`visits.farewell`) is leaving.
            (TaskKind::Social { social, .. }, _) if content.visits.farewell.contains(&social) => {
                false
            }
            (_, Phase::Routing { .. }) => true,
            _ => act.tags & content.social_rules.busy_tags == 0,
        }
    }

    /// For a Sim leading a conversation: (target, social, success, progress 0..1).
    pub fn conversation(&self, content: &Content) -> Option<(u32, usize, bool, f32)> {
        match &self.current {
            Some(Activity {
                task:
                    Task {
                        kind: TaskKind::Social { target, social },
                        ..
                    },
                phase: Phase::Conversing { elapsed, success },
                ..
            }) => {
                let s = &content.socials[*social];
                let duration = conversation::duration(s, *success);
                Some((*target, *social, *success, (elapsed / duration).min(1.0)))
            }
            _ => None,
        }
    }

    /// For a Sim listening in on someone else's conversation (a group conversation): the one
    /// who started it.
    pub fn listening_to(&self) -> Option<u32> {
        match &self.current {
            Some(Activity {
                task:
                    Task {
                        kind: TaskKind::Join { leader },
                        ..
                    },
                phase: Phase::Conversing { .. },
                ..
            }) => Some(*leader),
            _ => None,
        }
    }

    /// What they couldn't get to a moment ago and won't choose again yet (`unreachable`).
    pub(crate) fn unreachable_now(&self, tick: u64) -> impl Iterator<Item = TaskKind> + '_ {
        self.unreachable
            .iter()
            .flatten()
            .filter(move |u| tick < u.1)
            .map(|u| u.0)
    }

    /// Whether they couldn't get to object `id` a moment ago (`unreachable`), and won't try again yet.
    pub(crate) fn couldnt_reach(&self, id: u32, tick: u64) -> bool {
        self.unreachable_now(tick).any(|kind| matches!(kind, TaskKind::Use { object, .. } | TaskKind::Repair { object } if object == id))
    }

    /// They couldn't get to what they chose for themselves (`kind`): not again until `until`.
    /// Only what can't be reached for long is remembered (things, places, visits, and people,
    /// who move and so are given a short `until` by the caller); the oldest memory makes way.
    pub(crate) fn remember_unreachable(&mut self, kind: TaskKind, until: u64) {
        if !matches!(
            kind,
            TaskKind::Use { .. }
                | TaskKind::Eat { .. }
                | TaskKind::Clean { .. }
                | TaskKind::Repair { .. }
                | TaskKind::Dishes { .. }
                | TaskKind::Visit { .. }
                | TaskKind::Social { .. }
                | TaskKind::Join { .. }
        ) {
            return;
        }
        let same = self
            .unreachable
            .iter()
            .position(|u| u.is_some_and(|u| same_target(u.0, kind)));
        let slot = same.unwrap_or_else(|| {
            (0..UNREACHABLE)
                .min_by_key(|&k| self.unreachable[k].map_or(0, |u| u.1))
                .unwrap_or(0)
        });
        self.unreachable[slot] = Some((kind, until));
    }

    /// Objects moved (ids shifted, one gone): `remap` gives each object id in `unreachable` its
    /// new id, or None for one that's gone (forgotten then).
    pub(crate) fn remap_unreachable(&mut self, remap: impl Fn(u32) -> Option<u32>) {
        for slot in &mut self.unreachable {
            let Some((TaskKind::Use { object, .. } | TaskKind::Repair { object }, _)) = slot else {
                continue;
            };
            match remap(*object) {
                Some(new) => *object = new,
                None => *slot = None,
            }
        }
    }
}

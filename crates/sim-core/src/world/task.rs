//! What a resident is doing: tasks, their phases, and the activity around them.

use super::*;

#[derive(Debug, Clone, Copy)]
pub enum TaskKind {
    Use {
        object: u32,
        interaction: usize,
    },
    MoveTo {
        x: f32,
        z: f32,
    },
    /// Start a social interaction with another Sim.
    Social {
        target: u32,
        social: usize,
    },
    /// Join the conversation `leader` started and listen (a group conversation; see
    /// `conversation/`): `Routing` on the way, `Conversing` while listening, `Waiting` between
    /// two of the leader's socials.
    Join {
        leader: u32,
    },
    /// Head to the town exit for a work shift.
    Work,
    /// Walk to another household's front door and stay a while.
    Visit {
        plot: u32,
    },
    /// Walk back home.
    GoHome,
    /// Go to the front door to answer a visitor's knock (`visits.door`).
    Answer {
        guest: u32,
    },
    /// Tidy up around tile `(x, z)` (`roomRules.clean`).
    Clean {
        x: i32,
        z: i32,
    },
    /// Repair a broken object (`objectRules.repair`).
    Repair {
        object: u32,
    },
    /// An accident's little while on the spot (`Content::accidents`): asleep on the floor,
    /// eating takeout.
    Spot {
        accident: usize,
    },
    /// Cooking a recipe (`cooking.rs`; index into `Content::recipes`): at step `step`, at
    /// `station` once one is found, started from `origin` (the object whose interaction it was;
    /// its skills and feeling count). `servings` is 0 until the ingredients are paid for (at
    /// the first step), then how many it makes; `skipped` counts steps done without their
    /// station; `minutes` is how long the current step takes (until it starts, the minutes of
    /// the steps just done without, carried over to it).
    Cook {
        recipe: usize,
        step: u8,
        station: Option<u32>,
        origin: Option<u32>,
        servings: u8,
        skipped: u8,
        minutes: u16,
    },
    /// Eating: first fetching a serving of `item` (none: they hold their plate already), then
    /// at `seat` (none: standing), the plate put down there as item `plate` (none: in hand).
    Eat {
        item: Option<u32>,
        seat: Option<u32>,
        plate: Option<u32>,
        stage: u8,
    },
    /// Clearing away: picking up `item` and dirty things next to it (stage 0), scraping the
    /// food off `scraps` of them into a bin (1), washing up (2) and putting them away (3), each
    /// at `station`.
    Dishes {
        item: u32,
        station: Option<u32>,
        stage: u8,
        scraps: u8,
    },
    /// On the phone (`phone.rs`; index into `content.phone.actions`): with `contact`, if the
    /// action is with someone; on `seat` once one is found (none: standing); `answer`: how the
    /// other end took it.
    Phone {
        action: usize,
        contact: Option<u32>,
        seat: Option<u32>,
        answer: crate::phone::Answer,
    },
    /// Carrying something to `to` and putting it there (`carry.rs`): a baby (`baby`), first over
    /// to them in `from` and lifting them out (stage 0), then carrying them to `to` and putting
    /// them in (stage 1: the high chair to be fed, or back to a crib); or, `baby` none, what's in
    /// their hands (a harvest) to the fridge `to` (stage 1; `from` is `to`).
    Carry {
        baby: Option<u32>,
        from: u32,
        to: u32,
        stage: u8,
    },
    /// Setting up something new that was bought (`setup.rs`; `object` its type): over to its
    /// parcel at the front door (`parcel`, standing at `door`) and picking it up (stage 0), then
    /// carrying it to `stand`, beside where it goes (`at`, its middle), and putting it together
    /// there (stage 1).
    SetUp {
        object: u32,
        stage: u8,
        door: [f32; 2],
        parcel: [f32; 2],
        stand: (i32, i32),
        at: [f32; 2],
    },
    /// A host bringing `guest` something to eat or drink (`hosting.rs`): over to `from` to take
    /// it out (its `interaction`, an `eat` one; stage 0), then with it in hand to the guest and
    /// handing it over (stage 1 on; each stage past 1 is one more go at getting to them).
    Serve {
        guest: u32,
        from: u32,
        interaction: usize,
        stage: u8,
    },
}

impl TaskKind {
    /// Whether whoever does it keeps what's in their hands (the rest put it down): cooking,
    /// eating, washing up (`cooking::chain`), a harvest carried to the fridge (`carry.rs`), a
    /// parcel carried in to be set up (`setup.rs`) and a bite brought to a guest (`hosting.rs`).
    pub fn carries_held(&self) -> bool {
        matches!(
            self,
            TaskKind::Cook { .. }
                | TaskKind::Eat { .. }
                | TaskKind::Dishes { .. }
                | TaskKind::Carry { baby: None, .. }
                | TaskKind::SetUp { .. }
                | TaskKind::Serve { .. }
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Task {
    pub kind: TaskKind,
    /// Issued by the player (as opposed to chosen autonomously).
    pub directed: bool,
    /// Done as part of a routine (washing hands after the toilet; see `Interaction::after`):
    /// it takes its full time even with the needs it fills full.
    pub routine: bool,
}

#[derive(Debug, Clone)]
pub enum Phase {
    Routing {
        waypoints: Vec<[f32; 2]>,
        next: usize,
    },
    Using {
        elapsed: f32,
    },
    /// Calling out the social target's name from where they stand (they're within earshot); the
    /// two then walk to meet halfway (see `conversation::update`).
    Calling {
        ticks: u32,
    },
    /// Next to the social target, waiting for them to be free.
    Waiting {
        ticks: u32,
    },
    /// In a conversation; the outcome was decided when it started.
    Conversing {
        elapsed: f32,
        success: bool,
    },
    /// Waiting for a busy station (a cook or someone washing up; see `cooking.rs`).
    Queued {
        ticks: u32,
    },
    /// A visitor at the front door, waiting to be let in (`visits.door`, run by `life`).
    Knocking {
        ticks: u32,
        /// The resident on their way to answer.
        host: Option<u32>,
    },
    /// At the door of the car they go to work in (`Sim::bound_for_car`), getting in: it opens,
    /// they sit in, it shuts (`cars::getting`).
    Boarding {
        ticks: u32,
    },
    /// Back home in the car, getting out on their way in (a `GoHome` task): sat in it as its door
    /// opens, then out beside it (`cars::getting`).
    Alighting {
        ticks: u32,
    },
    /// At the wheel of their car on a trip to another hamlet or back (`cars::Trip`, a `Visit`
    /// or `GoHome` task): the car drives (`cars::update`), they sit in it.
    Driving,
    /// At their kerb on their way to work or school, waiting for the work taxi they're booked on
    /// (`Sim::taxi`, `taxi/`): it calls them to its door.
    AtKerb {
        ticks: u32,
    },
}

#[derive(Debug, Clone)]
pub struct Activity {
    pub task: Task,
    pub phase: Phase,
    /// Tags of the interaction or social being performed (for "busy" checks and autonomy).
    pub tags: TagMask,
}

/// Whether two tasks are after the same thing (`Sim::unreachable`): the same use of an object,
/// the same food, tile, broken object, dirty dish, house or person.
pub(super) fn same_target(a: TaskKind, b: TaskKind) -> bool {
    match (a, b) {
        (
            TaskKind::Use {
                object,
                interaction,
            },
            TaskKind::Use {
                object: o,
                interaction: i,
            },
        ) => (object, interaction) == (o, i),
        (TaskKind::Eat { item, .. }, TaskKind::Eat { item: it, .. }) => item == it,
        (TaskKind::Clean { x, z }, TaskKind::Clean { x: cx, z: cz }) => (x, z) == (cx, cz),
        (TaskKind::Repair { object }, TaskKind::Repair { object: o }) => object == o,
        (TaskKind::Dishes { item, .. }, TaskKind::Dishes { item: it, .. }) => item == it,
        (TaskKind::Visit { plot }, TaskKind::Visit { plot: p }) => plot == p,
        (
            TaskKind::Social { target, social },
            TaskKind::Social {
                target: t,
                social: s,
            },
        ) => (target, social) == (t, s),
        (TaskKind::Join { leader }, TaskKind::Join { leader: l }) => leader == l,
        _ => false,
    }
}

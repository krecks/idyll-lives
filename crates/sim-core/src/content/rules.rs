//! [(1576, 1691)]
//!
//! Split from `content.rs` (see `mod.rs`).

use super::*;

/// Weekly rent, charged from household funds.
#[derive(Debug, Clone)]
pub struct RentRules {
    /// 0 = Monday.
    pub weekday: u32,
    pub hour: f32,
    pub base: i64,
    /// Added per tile of the household's plot.
    pub per_tile: f32,
    /// Weekly bills (power, water, upkeep) charged with the rent: a flat part…
    pub bills_base: i64,
    /// …plus this share of what the household's objects are worth (price plus upgrades)…
    pub bills_rate: f32,
    /// …plus living costs (groceries, household goods) per resident.
    pub per_resident: i64,
    /// Given to every member each day the household is in debt.
    pub debt_feeling: Option<usize>,
}

impl RentRules {
    pub fn amount(&self, plot_tiles: i32) -> i64 {
        self.base + (self.per_tile * plot_tiles.max(0) as f32).round() as i64
    }

    pub fn bills(&self, home_value: i64, residents: usize) -> i64 {
        self.bills_base
            + (self.bills_rate * home_value.max(0) as f32).round() as i64
            + self.per_resident * residents as i64
    }
}

/// A time-of-day window that biases autonomy (sleep at night, socialise in the evening).
#[derive(Debug)]
pub struct ScheduleWindow {
    /// Hours; `from > to` wraps past midnight.
    pub from: f32,
    pub to: f32,
    pub mods: Modifiers,
}

impl ScheduleWindow {
    pub fn contains(&self, hour: f32) -> bool {
        in_hours([self.from, self.to], hour)
    }
}

/// How NPC households visit each other.
#[derive(Debug, Clone)]
pub struct VisitRules {
    /// Expected need gains from a visit (for autonomy scoring).
    pub gains: [f32; MAX_NEEDS],
    pub hours: f32,
    /// How long a visit the player asked for lasts.
    pub directed_hours: f32,
    pub min_friendship: f32,
    pub earliest_hour: f32,
    pub latest_hour: f32,
    /// An outing (open-world towns: a public place out of `autonomy.reach`) goes only where the
    /// way takes this many minutes at most (`outingMinutes`: by car for one who'd drive there,
    /// else on foot), and only with time there and the way back before `outing_home_by` o'clock
    /// (`outingHomeBy`); its stay ends in time for that.
    pub outing_minutes: f32,
    pub outing_home_by: f32,
    /// Once there (let in, or at the place), a guest stays this long at least (`leastMinutes`),
    /// unless a need is about to run out (`autonomy.urgentBelow`), it's late or the hosts go.
    pub least_minutes: f32,
    /// Back home from a visit or an outing, they don't set off on another of their own for this
    /// long (`againMinutes`).
    pub again_minutes: f32,
    /// Tag mask used for schedule/trait preferences of visiting.
    pub tags: TagMask,
    /// Visitors knock and wait to be let in (`visits.door`); without it they walk straight in.
    pub door: Option<DoorRules>,
    /// The least a friendly chat with one of the hosts scores for a guest (keeping them
    /// company is what a visit is for, even with nothing to talk about); 0: no more than usual.
    pub company: f32,
    /// A guest with free will left with nothing to do for this long heads home (`boredMinutes`);
    /// None: they stay until the visit is over.
    pub bored_minutes: Option<f32>,
    /// What a guest says goodbye to a host with as the visit ends (`farewell`, socials): the
    /// first of them the guest may do with that host. Empty: they just leave.
    pub farewell: Vec<usize>,
    /// How long a leaving guest spends getting to a host to say goodbye (`farewellMinutes`):
    /// hosts further away than that aren't asked, and a goodbye not started by then is skipped.
    pub farewell_minutes: f32,
    /// A host offers a guest something to eat or drink (`offer`; `hosting.rs`). None: they don't.
    pub offer: Option<OfferRules>,
}

/// A host offering a guest something to eat or drink (`visits.offer`, `hosting.rs`).
#[derive(Debug, Clone)]
pub struct OfferRules {
    /// What the host asks with (a social whose success has `"effect": "serve"`).
    pub social: usize,
    /// How long after the guest comes in the host first offers, and how long after an offer
    /// (taken up or not) they offer again.
    pub after_minutes: f32,
    pub again_minutes: f32,
    /// What the host is shown doing while handing it over (`handAnim`).
    pub hand_anim: Option<usize>,
}

/// A visitor at the door: someone at home answers and lets them in or turns them away.
#[derive(Debug, Clone)]
pub struct DoorRules {
    /// How long a visitor waits for someone to answer before going home.
    pub wait_minutes: f32,
    /// Hosts let in visitors they like at least this much (friendship).
    pub welcome_friendship: f32,
    /// What the host starts with a visitor they let in (a social).
    pub greet: Option<usize>,
    /// Feeling for a visitor who was turned away.
    pub turned_away: Option<usize>,
    /// Animation tag while knocking.
    pub knock_anim: Option<usize>,
}

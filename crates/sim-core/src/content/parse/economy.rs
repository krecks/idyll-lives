//! Economy, schedule and visit shapes (content `economy`, `schedule`, `visits`).
//!
//! Split from `content/parse.rs` (see `mod.rs`).

use super::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CareerLevelRaw {
    pub(crate) title: String,
    pub(crate) pay: i64,
    pub(crate) start: f32,
    pub(crate) hours: f32,
    /// Weekday indices, 0 = Monday.
    pub(crate) days: Vec<u8>,
    #[serde(default)]
    pub(crate) requires: HashMap<String, f32>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct CareerRulesRaw {
    pub(crate) commute_minutes: Option<f32>,
    pub(crate) late_minutes: Option<f32>,
    pub(crate) missed_penalty: Option<f32>,
    pub(crate) performance_per_shift: Option<f32>,
    pub(crate) performance_per_fit: Option<f32>,
    pub(crate) performance_per_mood: Option<f32>,
    pub(crate) performance_slowdown: Option<f32>,
    pub(crate) late_grace_minutes: Option<f32>,
    pub(crate) late_share: Option<f32>,
    pub(crate) probation_levels: Option<f32>,
    pub(crate) workweek: Vec<WorkweekRaw>,
    #[serde(alias = "promotionMoodlet")] // key before the moodlet → feeling rename
    pub(crate) promotion_feeling: Option<String>,
    #[serde(alias = "missedMoodlet")] // key before the moodlet → feeling rename
    pub(crate) missed_feeling: Option<String>,
    pub(crate) market: Option<JobMarketRaw>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct JobMarketRaw {
    pub(crate) hour: Option<f32>,
    pub(crate) daily_chance: Option<f32>,
    pub(crate) quit_below: Option<f32>,
    pub(crate) quit_chance: Option<f32>,
    pub(crate) min_shifts_before_quit: Option<u32>,
    pub(crate) fire_after_missed: Option<u8>,
    pub(crate) cooldown_days: Option<u32>,
    pub(crate) hired_feeling: Option<String>,
    pub(crate) quit_feeling: Option<String>,
    pub(crate) fired_feeling: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct WorkweekRaw {
    pub(crate) fit: f32,
    pub(crate) days: i8,
    pub(crate) label: String,
}

#[derive(Deserialize)]
pub(crate) struct ScheduleRaw {
    pub(crate) from: f32,
    pub(crate) to: f32,
    #[serde(default)]
    pub(crate) effects: ModifiersRaw,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct VisitRaw {
    pub(crate) needs: HashMap<String, f32>,
    pub(crate) hours: Option<f32>,
    pub(crate) directed_hours: Option<f32>,
    pub(crate) min_friendship: Option<f32>,
    pub(crate) earliest_hour: Option<f32>,
    pub(crate) latest_hour: Option<f32>,
    pub(crate) outing_minutes: Option<f32>,
    pub(crate) outing_home_by: Option<f32>,
    pub(crate) least_minutes: Option<f32>,
    pub(crate) again_minutes: Option<f32>,
    pub(crate) door: Option<DoorRaw>,
    pub(crate) company: Option<f32>,
    pub(crate) bored_minutes: Option<f32>,
    pub(crate) farewell: Vec<String>,
    pub(crate) farewell_minutes: Option<f32>,
    pub(crate) offer: Option<OfferRaw>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct OfferRaw {
    pub(crate) social: String,
    pub(crate) after_minutes: Option<f32>,
    pub(crate) again_minutes: Option<f32>,
    pub(crate) hand_anim: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct DoorRaw {
    pub(crate) wait_minutes: Option<f32>,
    pub(crate) welcome_friendship: Option<f32>,
    pub(crate) greet: Option<String>,
    pub(crate) turned_away_feeling: Option<String>,
    pub(crate) knock_anim: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct EconomyRaw {
    pub(crate) starting_funds: Option<i64>,
    pub(crate) rent: Option<RentRaw>,
}

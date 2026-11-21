//! Life, household and accident shapes (content `life`).

use super::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RentRaw {
    #[serde(default = "sunday")]
    pub(crate) weekday: u32,
    #[serde(default = "noon")]
    pub(crate) hour: f32,
    pub(crate) base: i64,
    #[serde(default)]
    pub(crate) per_tile: f32,
    #[serde(default)]
    pub(crate) bills_base: i64,
    #[serde(default)]
    pub(crate) bills_rate: f32,
    #[serde(default)]
    pub(crate) per_resident: i64,
    #[serde(default)]
    #[serde(alias = "debtMoodlet")] // key before the moodlet → feeling rename
    pub(crate) debt_feeling: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct LifeRaw {
    pub(crate) days_per_year: Option<f32>,
    pub(crate) stages: Vec<LifeStageRaw>,
    pub(crate) start_age: Option<[f32; 2]>,
    pub(crate) retire_at: Option<f32>,
    pub(crate) pension: Option<f32>,
    pub(crate) basic_pension: Option<i64>,
    pub(crate) retire_feeling: Option<String>,
    pub(crate) death: Option<DeathRaw>,
    pub(crate) grief: GriefRaw,
    pub(crate) moving: Option<MovingRaw>,
    pub(crate) newcomers: Option<NewcomersRaw>,
    pub(crate) school: Option<SchoolRaw>,
    pub(crate) pregnancy: Option<PregnancyRaw>,
    pub(crate) adoption: Option<AdoptionRaw>,
    pub(crate) childcare: Option<ChildcareRaw>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ChildcareRaw {
    pub(crate) cost: i64,
    #[serde(default)]
    pub(crate) below: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PregnancyRaw {
    pub(crate) chance: f32,
    pub(crate) days: u32,
    #[serde(default)]
    pub(crate) max_age: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AdoptionRaw {
    pub(crate) cost: i64,
    #[serde(default)]
    pub(crate) neighbours: f32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SchoolRaw {
    pub(crate) start: f32,
    pub(crate) hours: f32,
    pub(crate) days: Vec<u8>,
    #[serde(default)]
    pub(crate) skills: HashMap<String, f32>,
    #[serde(default)]
    pub(crate) grades: GradesRaw,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct GradesRaw {
    pub(crate) attend: Option<f32>,
    pub(crate) mood: Option<f32>,
    pub(crate) missed: Option<f32>,
    pub(crate) start: Option<f32>,
    pub(crate) good: Option<f32>,
    pub(crate) good_feeling: Option<String>,
    pub(crate) poor: Option<f32>,
    pub(crate) poor_feeling: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct NewcomersRaw {
    pub(crate) hour: Option<f32>,
    pub(crate) days: Option<f32>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct MovingRaw {
    pub(crate) hour: Option<f32>,
    pub(crate) partners: Option<f32>,
    pub(crate) romance: Option<f32>,
    pub(crate) leave_home_age: Option<f32>,
    pub(crate) leave_home: Option<f32>,
    pub(crate) leave_town_age: Option<f32>,
    pub(crate) leave_town: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeathRaw {
    pub(crate) from: f32,
    pub(crate) per_year: f32,
    #[serde(default)]
    pub(crate) growth: f32,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct GriefRaw {
    pub(crate) feeling: Option<String>,
    pub(crate) light_feeling: Option<String>,
    pub(crate) close: Option<f32>,
    pub(crate) friend: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LifeStageRaw {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) label: Option<String>,
    pub(crate) from: f32,
    #[serde(default)]
    pub(crate) effects: ModifiersRaw,
    #[serde(default)]
    pub(crate) feeling: Option<String>,
    #[serde(default)]
    pub(crate) adult: Option<bool>,
    #[serde(default)]
    pub(crate) baby: bool,
    #[serde(default)]
    pub(crate) school: bool,
    #[serde(default)]
    pub(crate) scale: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AccidentRaw {
    pub(crate) id: String,
    pub(crate) need: String,
    #[serde(default)]
    pub(crate) story: Option<String>,
    #[serde(default)]
    pub(crate) effects: HashMap<String, f32>,
    #[serde(default)]
    pub(crate) dirt: f32,
    #[serde(default)]
    pub(crate) feeling: Option<String>,
    #[serde(default)]
    pub(crate) cost: i64,
    #[serde(default)]
    pub(crate) cooldown_hours: Option<f32>,
    #[serde(default)]
    pub(crate) grace_minutes: Option<f32>,
    #[serde(default)]
    pub(crate) crowded_grace_minutes: Option<f32>,
    #[serde(default)]
    pub(crate) rest: Option<AccidentRestRaw>,
    #[serde(default)]
    pub(crate) another_room: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AccidentRestRaw {
    pub(crate) label: String,
    pub(crate) minutes: f32,
    #[serde(default)]
    pub(crate) pose: Pose,
    #[serde(default)]
    pub(crate) anim: Option<String>,
    #[serde(default)]
    pub(crate) gains: HashMap<String, f32>,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
}

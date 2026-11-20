//! [(2065, 3486)]
//!
//! Split from `content.rs` (see `mod.rs`).

use super::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContentFile {
    pub(crate) needs: Vec<NeedRaw>,
    pub(crate) objects: Vec<ObjectRaw>,
    #[serde(default)]
    pub(crate) traits: Vec<TraitRaw>,
    #[serde(default)]
    pub(crate) perks: Vec<PerkRaw>,
    #[serde(default)]
    pub(crate) genders: Vec<GenderRaw>,
    #[serde(default)]
    pub(crate) rules: CharacterRules,
    #[serde(default)]
    pub(crate) emotions: Vec<EmotionRaw>,
    #[serde(default)]
    #[serde(alias = "moodlets")] // key before the moodlet → feeling rename
    pub(crate) feelings: Vec<FeelingRaw>,
    #[serde(default)]
    pub(crate) socials: Vec<SocialRaw>,
    #[serde(default)]
    pub(crate) bond_presets: HashMap<String, BondPreset>,
    #[serde(default)]
    pub(crate) social_rules: SocialRulesRaw,
    #[serde(default)]
    pub(crate) careers: Vec<CareerRaw>,
    #[serde(default)]
    pub(crate) career_rules: CareerRulesRaw,
    #[serde(default)]
    pub(crate) schedule: Vec<ScheduleRaw>,
    #[serde(default)]
    pub(crate) visits: VisitRaw,
    #[serde(default)]
    pub(crate) economy: EconomyRaw,
    #[serde(default)]
    pub(crate) skills: Vec<SkillRaw>,
    #[serde(default)]
    pub(crate) skill_rules: SkillRulesRaw,
    #[serde(default)]
    pub(crate) grades: Vec<GradeRaw>,
    #[serde(default)]
    pub(crate) career_categories: Vec<CategoryRaw>,
    #[serde(default)]
    pub(crate) car_bodies: Vec<CarBodyRaw>,
    #[serde(default)]
    pub(crate) day_rhythm: DayRhythmRaw,
    #[serde(default)]
    pub(crate) autonomy: crate::ai::AutonomyRaw,
    #[serde(default)]
    pub(crate) object_rules: ObjectRulesRaw,
    #[serde(default)]
    pub(crate) room_kinds: Vec<RoomKindRaw>,
    #[serde(default)]
    pub(crate) accidents: Vec<AccidentRaw>,
    #[serde(default)]
    pub(crate) life: LifeRaw,
    #[serde(default)]
    pub(crate) names: Names,
    #[serde(default)]
    pub(crate) room_rules: RoomRulesRaw,
    #[serde(default)]
    pub(crate) build: BuildRaw,
    #[serde(default)]
    pub(crate) wall_coverings: Vec<BuildStyleRaw>,
    #[serde(default)]
    pub(crate) door_styles: Vec<BuildStyleRaw>,
    #[serde(default)]
    pub(crate) window_styles: Vec<BuildStyleRaw>,
    #[serde(default)]
    pub(crate) fence_styles: Vec<BuildStyleRaw>,
    #[serde(default)]
    pub(crate) roof_styles: Vec<BuildStyleRaw>,
    #[serde(default)]
    pub(crate) roof_colors: Vec<BuildStyleRaw>,
    #[serde(default)]
    pub(crate) floor_coverings: Vec<BuildStyleRaw>,
    /// Extra tags that no interaction or social uses (yet), so traits and emotions can refer to them.
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    /// Animation tags interactions may use as `anim`.
    #[serde(default)]
    pub(crate) animations: Vec<String>,
    #[serde(default)]
    pub(crate) activities: Vec<crate::planner::ActivityRaw>,
    #[serde(default)]
    pub(crate) planner: crate::planner::PlannerRaw,
    #[serde(default)]
    pub(crate) goals: Vec<crate::planner::GoalRaw>,
    #[serde(default)]
    pub(crate) cooking: crate::cooking::CookingRaw,
    #[serde(default)]
    pub(crate) items: Vec<crate::cooking::ItemRaw>,
    #[serde(default)]
    pub(crate) recipes: Vec<crate::cooking::RecipeRaw>,
    #[serde(default)]
    pub(crate) phone: crate::phone::PhoneRaw,
    #[serde(default)]
    pub(crate) service: Option<crate::staff::ServiceRaw>,
    #[serde(default)]
    pub(crate) laundry: Option<LaundryRaw>,
    #[serde(default)]
    pub(crate) cars: CarsRaw,
    #[serde(default)]
    pub(crate) taxi: TaxiRaw,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CareerRaw {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) levels: Vec<CareerLevelRaw>,
    #[serde(default)]
    pub(crate) work_effects: HashMap<String, f32>,
    /// Skills practised at work, with weights.
    #[serde(default)]
    pub(crate) skills: HashMap<String, f32>,
}

#[derive(Deserialize)]
pub(crate) struct SkillRaw {
    pub(crate) id: String,
    pub(crate) label: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct SkillRulesRaw {
    pub(crate) max_level: Option<f32>,
    pub(crate) work_gain_per_hour: Option<f32>,
    pub(crate) slowdown: Option<f32>,
    pub(crate) interest: Option<f32>,
    pub(crate) goal_interest: Option<f32>,
    pub(crate) effect_per_level: Option<f32>,
    pub(crate) fatigue_per_hour: Option<f32>,
    pub(crate) fatigue_recovery_per_hour: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GradeRaw {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) pay_per_hour: f32,
    pub(crate) skill_level: f32,
    /// How old the car a job at this grade calls for is (absent: used).
    #[serde(default)]
    pub(crate) car_age: CarAge,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CarBodyRaw {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) len: f32,
    pub(crate) wid: f32,
}

/// A car body's length and width (m): a small car's to a long pickup's, a narrow one's to a van's.
pub(crate) const CAR_LEN: RangeInclusive<f32> = 2.5..=6.0;
pub(crate) const CAR_WID: RangeInclusive<f32> = 1.4..=2.2;

/// A field's or a track's `car` (`what`): the look a job at each grade calls for, from the
/// grade a body is given at on (none before the first one given).
pub(crate) fn car_by_grade(
    what: &str,
    map: &HashMap<String, String>,
    grades: &[GradeDef],
    ages: &[CarAge],
    bodies: &[CarBodyDef],
) -> Result<Vec<Option<CarLook>>, Error> {
    let mut from: Vec<Option<u8>> = vec![None; grades.len()];
    for (grade, body) in map {
        let g = grades
            .iter()
            .position(|d| &d.id == grade)
            .ok_or_else(|| Error::new(format!("'{what}': car for unknown grade '{grade}'")))?;
        let b = bodies
            .iter()
            .position(|d| &d.id == body)
            .ok_or_else(|| Error::new(format!("'{what}': unknown car body '{body}'")))?;
        from[g] = Some(b as u8);
    }
    let mut body = None;
    Ok(from
        .into_iter()
        .zip(ages)
        .map(|(b, &age)| {
            body = b.or(body);
            body.map(|body| CarLook { body, age })
        })
        .collect())
}

/// Content's `value`, refused rather than clamped when it is outside `range`. `what` names the
/// figure (with the id or rule it belongs to), so the error tells a pack author exactly what to
/// fix.
pub(crate) fn checked_range(
    value: f32,
    what: &str,
    range: RangeInclusive<f32>,
) -> Result<f32, Error> {
    if !range.contains(&value) {
        return Err(Error::new(format!(
            "{what} {value} must be {}..{}",
            range.start(),
            range.end()
        )));
    }
    Ok(value)
}

#[derive(Deserialize, Clone, Copy)]
pub(crate) struct ShiftRaw {
    pub(crate) start: f32,
    pub(crate) hours: f32,
    pub(crate) days: Days,
}

/// Weekday indices (0 = Monday), stored inline so shifts stay `Copy`.
#[derive(Clone, Copy)]
pub(crate) struct Days(pub(crate) u8, pub(crate) bool);

impl<'de> Deserialize<'de> for Days {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let list = Vec::<u8>::deserialize(d)?;
        let valid = list.iter().all(|&x| x <= 6);
        Ok(Days(
            list.iter()
                .filter(|&&x| x <= 6)
                .fold(0, |m, x| m | (1 << x)),
            valid,
        ))
    }
}

/// A career category; each track in it becomes one career, one level per grade.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CategoryRaw {
    pub(crate) id: String,
    pub(crate) label: String,
    /// Pay for the whole field relative to the grades' own rates (1 = the grades' rates).
    #[serde(default = "one")]
    pub(crate) pay_factor: f32,
    pub(crate) shift: ShiftRaw,
    #[serde(default)]
    pub(crate) work_effects: HashMap<String, f32>,
    /// The car its jobs call for: a body (`carBodies` id) from each grade on (grade id → body).
    #[serde(default)]
    pub(crate) car: Option<HashMap<String, String>>,
    pub(crate) tracks: Vec<TrackRaw>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TrackRaw {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) skills: HashMap<String, f32>,
    /// Pay at the top of this track relative to the category's `payFactor` (1 = the
    /// category's rate). Pay rises in a straight line from the entry grade, which pays the
    /// category's rate.
    #[serde(default = "one")]
    pub(crate) pay_factor: f32,
    #[serde(default)]
    pub(crate) shift: Option<ShiftRaw>,
    /// As the category's `car`, for this track instead.
    #[serde(default)]
    pub(crate) car: Option<HashMap<String, String>>,
    pub(crate) titles: Vec<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct DayRhythmRaw {
    pub(crate) wake_hour: Option<f32>,
    pub(crate) bed_hour: Option<f32>,
    pub(crate) sleep_tags: Vec<String>,
    pub(crate) night_decay: HashMap<String, f32>,
    pub(crate) asleep_decay: HashMap<String, f32>,
    pub(crate) work_decay: HashMap<String, f32>,
    pub(crate) wake_for: Option<Vec<String>>,
    pub(crate) wake_before_work_minutes: Option<f32>,
    pub(crate) lie_in_hours: Option<f32>,
    pub(crate) sleep_quality_bonus: Option<f32>,
    pub(crate) nap_tags: Vec<String>,
    pub(crate) nap_cooldown_hours: Option<f32>,
    pub(crate) exhausted_below: Option<f32>,
    pub(crate) habits: Vec<HabitRaw>,
    pub(crate) ready_minutes: Option<f32>,
    pub(crate) work_breaks: Vec<WorkBreakRaw>,
    pub(crate) baby_wakes: Option<BabyWakesRaw>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BabyWakesRaw {
    #[serde(default)]
    pub(crate) needs: Option<Vec<String>>,
    #[serde(default)]
    pub(crate) below: Option<f32>,
    #[serde(default)]
    pub(crate) tend_below: Option<f32>,
    #[serde(default)]
    pub(crate) cooldown_minutes: Option<f32>,
    #[serde(default)]
    pub(crate) finish_minutes: Option<f32>,
    #[serde(default)]
    pub(crate) feeling: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct WorkBreakRaw {
    pub(crate) need: String,
    pub(crate) below: f32,
    pub(crate) gain: f32,
    #[serde(default)]
    pub(crate) cost: i64,
}

#[derive(Deserialize)]
pub(crate) struct HabitRaw {
    pub(crate) at: HabitTime,
    pub(crate) need: String,
    #[serde(default = "one_f32")]
    pub(crate) below: f32,
}

fn one_f32() -> f32 {
    1.0
}

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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RoomKindRaw {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) label: Option<String>,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    #[serde(default)]
    pub(crate) essentials: Vec<Vec<String>>,
    #[serde(default)]
    pub(crate) size: Option<[f32; 2]>,
    #[serde(default)]
    pub(crate) exclusive: bool,
    #[serde(default)]
    pub(crate) passage: bool,
    #[serde(default)]
    pub(crate) private: bool,
    #[serde(default)]
    pub(crate) claimed: bool,
    #[serde(default)]
    pub(crate) carpet: bool,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct WearRaw {
    pub(crate) per_use: Option<f32>,
    pub(crate) by_category: HashMap<String, f32>,
    pub(crate) per_quality: Option<f32>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct RepairRaw {
    pub(crate) label: Option<String>,
    pub(crate) minutes: Option<f32>,
    pub(crate) skill: Option<String>,
    pub(crate) skill_gain_per_hour: Option<f32>,
    pub(crate) chance: Option<f32>,
    pub(crate) chance_per_level: Option<f32>,
    pub(crate) interest: Option<f32>,
    pub(crate) cost: Option<f32>,
    pub(crate) tags: Option<Vec<String>>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct SetUpRaw {
    pub(crate) label: Option<String>,
    pub(crate) minutes: Option<f32>,
    pub(crate) fetch_minutes: Option<f32>,
    pub(crate) help_after_hours: Option<f32>,
    pub(crate) wait_hours: Option<f32>,
    pub(crate) parcel: Option<String>,
    pub(crate) tags: Option<Vec<String>>,
    pub(crate) anim: Option<String>,
    pub(crate) fetch_anim: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct RoomRulesRaw {
    pub(crate) weights: Option<[f32; 5]>,
    pub(crate) window_tiles: Option<f32>,
    pub(crate) lamp_tiles: Option<f32>,
    pub(crate) decor_per_tile: Option<f32>,
    pub(crate) garden_decor_per_tile: Option<f32>,
    pub(crate) love: Option<f32>,
    pub(crate) dislike: Option<f32>,
    pub(crate) drift: Option<f32>,
    pub(crate) away: Option<f32>,
    pub(crate) preference: Option<f32>,
    pub(crate) love_feeling: Option<String>,
    pub(crate) dislike_feeling: Option<String>,
    pub(crate) decor_by_category: HashMap<String, f32>,
    pub(crate) dirt: HashMap<String, f32>,
    pub(crate) clean: CleanRaw,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct CleanRaw {
    pub(crate) label: Option<String>,
    pub(crate) minutes: Option<f32>,
    pub(crate) threshold: Option<f32>,
    pub(crate) amount: Option<f32>,
    pub(crate) interest: Option<f32>,
    pub(crate) tags: Option<Vec<String>>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct ObjectRulesRaw {
    /// Catalog categories whose 1×1 objects turn freely.
    pub(crate) free_rotation: Option<Vec<String>>,
    pub(crate) wear: WearRaw,
    pub(crate) repair: RepairRaw,
    pub(crate) set_up: SetUpRaw,
    pub(crate) max_quality: Option<u8>,
    pub(crate) quality_bonus: Option<f32>,
    pub(crate) upgrade_cost: Option<f32>,
    pub(crate) resale: Option<f32>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct BuildRaw {
    pub(crate) wall: Option<i64>,
    pub(crate) door: Option<i64>,
    pub(crate) window: Option<i64>,
    pub(crate) remove: Option<i64>,
    /// Fence per metre and gate, without fence styles.
    pub(crate) fence: Option<i64>,
    pub(crate) gate: Option<i64>,
    /// A garage on the home lot.
    pub(crate) garage: Option<i64>,
    /// A porch over a door, a porch post, and a porch canopy and a step platform a tile.
    pub(crate) porch: Option<i64>,
    pub(crate) porch_post: Option<i64>,
    pub(crate) porch_canopy: Option<i64>,
    pub(crate) porch_step: Option<i64>,
    /// Shaping the land (`LandPrices`); a price left out is the default's.
    pub(crate) land: LandPricesRaw,
    pub(crate) storeys: Option<u8>,
    #[serde(default)]
    pub(crate) basement: bool,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub(crate) struct LandPricesRaw {
    pub(crate) earth: Option<f64>,
    pub(crate) mountain: Option<f64>,
    pub(crate) water: Option<f64>,
    pub(crate) woods: Option<f64>,
    pub(crate) meadow: Option<f64>,
    pub(crate) fields: Option<f64>,
    pub(crate) clear: Option<f64>,
}

impl LandPricesRaw {
    /// The prices, each a number ≥ 0 (left out: the default's).
    pub(crate) fn prices(&self) -> Result<LandPrices, Error> {
        let d = LandPrices::default();
        let price = |name: &str, v: Option<f64>, or: f64| {
            let v = v.unwrap_or(or);
            if v.is_finite() && v >= 0.0 {
                Ok(v)
            } else {
                Err(Error::new(format!(
                    "build.land.{name} {v} must be a number ≥ 0"
                )))
            }
        };
        Ok(LandPrices {
            earth: price("earth", self.earth, d.earth)?,
            mountain: price("mountain", self.mountain, d.mountain)?,
            water: price("water", self.water, d.water)?,
            woods: price("woods", self.woods, d.woods)?,
            meadow: price("meadow", self.meadow, d.meadow)?,
            fields: price("fields", self.fields, d.fields)?,
            clear: price("clear", self.clear, d.clear)?,
        })
    }
}

#[derive(Deserialize)]
pub(crate) struct BuildStyleRaw {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) price: i64,
    /// How it's drawn (a floor covering's `wood`, `tile`, `carpet`...; see `BuildStyle::soft`).
    #[serde(default)]
    pub(crate) finish: Option<String>,
}

fn one() -> f32 {
    1.0
}

fn sunday() -> u32 {
    6
}

fn noon() -> f32 {
    12.0
}

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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NeedRaw {
    pub(crate) id: String,
    pub(crate) label: String,
    #[serde(default)]
    pub(crate) decay_per_hour: f32,
    #[serde(default)]
    pub(crate) room: bool,
    #[serde(default)]
    pub(crate) comfortable: f32,
}

#[derive(Deserialize)]
pub(crate) struct ObjectRaw {
    pub(crate) id: String,
    pub(crate) name: String,
    #[serde(default = "one_by_one")]
    pub(crate) footprint: [u8; 2],
    #[serde(default)]
    pub(crate) interactions: Vec<InteractionRaw>,
    /// Things it provides while working (see `Interaction::requires`).
    #[serde(default)]
    pub(crate) provides: Vec<String>,
    #[serde(default)]
    pub(crate) price: Option<i64>,
    #[serde(default = "one_slot")]
    pub(crate) slots: u8,
    /// Each slot's place and facing (see `ObjectDef::places`).
    #[serde(default)]
    pub(crate) places: Vec<PlaceRaw>,
    #[serde(default)]
    pub(crate) outdoors: bool,
    /// See `ObjectDef::crown`.
    #[serde(default)]
    pub(crate) crown: f32,
    /// The household's own (see `ObjectDef::private`).
    #[serde(default)]
    pub(crate) private: bool,
    #[serde(default)]
    pub(crate) stairs: bool,
    /// See `ObjectDef::stair`: "spiral", else a straight flight.
    #[serde(default)]
    pub(crate) stair: Option<String>,
    /// Buy catalog category (decides free rotation, see `objectRules.freeRotation`).
    #[serde(default)]
    pub(crate) category: Option<String>,
    #[serde(default)]
    pub(crate) group: Option<String>,
    /// Overrides whether it turns freely.
    #[serde(default, rename = "freeRotation")]
    pub(crate) free_rotation: Option<bool>,
    /// Decor points (default: by category, `roomRules.decorByCategory`).
    #[serde(default)]
    pub(crate) decor: Option<f32>,
    /// Wear per finished use (default: `objectRules.wear`).
    #[serde(default, rename = "wearPerUse")]
    pub(crate) wear_per_use: Option<f32>,
    /// Lamps: the renderer reads the light's look; the simulation only its `tiles`, the floor it
    /// lights (default `roomRules.lampTiles`), `switch` (false: no switch, always lit at night)
    /// and `fire` (a fire, lit to sit by: see `ObjectDef::fire`).
    #[serde(default)]
    pub(crate) light: Option<serde_json::Value>,
    /// `wall`, `ceiling` or `rug` (default `floor`: furniture).
    #[serde(default)]
    pub(crate) layer: Option<String>,
    /// Bunk beds: the two slots lie one above the other.
    #[serde(default)]
    pub(crate) bunk: bool,
    #[serde(default)]
    pub(crate) surface: Option<SurfaceRaw>,
    #[serde(default)]
    pub(crate) radius: Option<f32>,
    /// `front` (default) or `around`.
    #[serde(default)]
    pub(crate) approach: Option<String>,
    #[serde(default, rename = "seatAt")]
    pub(crate) seat_at: Vec<String>,
    /// Cooking stations (`cooking.stations`).
    #[serde(default)]
    pub(crate) stations: Vec<String>,
    #[serde(default)]
    pub(crate) stores: bool,
    /// A crop that grows (see `ObjectDef::grows`).
    #[serde(default)]
    pub(crate) grows: Option<GrowsRaw>,
    /// A machine that runs on its own (see `ObjectDef::runs`).
    #[serde(default)]
    pub(crate) runs: Option<RunsRaw>,
    /// How many dirty things fill it (see `ObjectDef::rack`).
    #[serde(default)]
    pub(crate) rack: u8,
    #[serde(default, rename = "overWindow")]
    pub(crate) over_window: bool,
    #[serde(default, rename = "fillsWall")]
    pub(crate) fills_wall: bool,
    #[serde(default, rename = "hangsLow")]
    pub(crate) hangs_low: bool,
    /// Life-stage ids that may use it (see `Interaction::stages`).
    #[serde(default)]
    pub(crate) stages: Option<Vec<String>>,
    /// Life-stage ids that may take the upper bunk (see `ObjectDef::upper_stages`).
    #[serde(default, rename = "upperStages")]
    pub(crate) upper_stages: Option<Vec<String>>,
    #[serde(default, rename = "headHeight")]
    pub(crate) head_height: bool,
    #[serde(default, rename = "againstWall")]
    pub(crate) against_wall: bool,
    #[serde(default)]
    pub(crate) reach: f32,
    #[serde(default)]
    pub(crate) footstool: bool,
    #[serde(default)]
    pub(crate) overhang: Vec<[i32; 2]>,
    /// Someone serves here on a public lot (see `ObjectDef::staff`).
    #[serde(default)]
    pub(crate) staff: Option<crate::staff::PostRaw>,
    /// Something to ride (see `ObjectDef::vehicle`).
    #[serde(default)]
    pub(crate) vehicle: Option<VehicleRaw>,
}

/// An object's `vehicle`: `{"kind": "bike", "speed": 3, "stages": ["child", "teen", ...],
/// "minRide": 60, "roadsOnly": false, "reach": [1.2, 0.6, 0.3]}`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct VehicleRaw {
    pub(crate) kind: VehicleKind,
    /// Times walking (1 to 10), as a person on it would go at real time: it goes
    /// `crate::DRIVE_SCALE` times that, on the traffic's clock (`Vehicle::pace`).
    pub(crate) speed: f32,
    pub(crate) stages: Option<Vec<String>>,
    #[serde(default = "default_min_ride")]
    pub(crate) min_ride: f32,
    #[serde(default)]
    pub(crate) roads_only: bool,
    pub(crate) reach: Option<[f32; 3]>,
}

fn default_min_ride() -> f32 {
    60.0
}

/// A slot's place: `{"at": [-0.3, 0.7], "facing": 180}` (m off the middle, to its right and front;
/// degrees from its front towards its right, default 0).
#[derive(Deserialize)]
pub(crate) struct PlaceRaw {
    pub(crate) at: [f32; 2],
    #[serde(default)]
    pub(crate) facing: f32,
    #[serde(default)]
    pub(crate) pose: Option<Pose>,
}

/// An object's `grows`: `{"days": 2, "fallow": 0.5}` (game days to ripen, and fallow after picking).
#[derive(Deserialize)]
pub(crate) struct GrowsRaw {
    pub(crate) days: f32,
    #[serde(default = "half_a_day")]
    pub(crate) fallow: f32,
    /// Ripe when bought (see `Growth::bought_ripe`).
    #[serde(default, rename = "boughtRipe")]
    pub(crate) bought_ripe: bool,
    /// What an order says while it isn't ripe (see `ObjectDef::not_ripe`).
    #[serde(default, rename = "notRipe")]
    pub(crate) not_ripe: Option<String>,
}

/// An object's `runs`: `{"minutes": 60}` (and, for the renderer, the `sound` it makes running).
#[derive(Deserialize)]
pub(crate) struct RunsRaw {
    pub(crate) minutes: f32,
}

fn half_a_day() -> f32 {
    0.5
}

#[derive(Deserialize)]
pub(crate) struct SurfaceRaw {
    pub(crate) height: f32,
    pub(crate) kind: String,
    #[serde(default = "surface_inset")]
    pub(crate) inset: f32,
    #[serde(default)]
    pub(crate) size: Option<[f32; 2]>,
    #[serde(default)]
    pub(crate) at: [f32; 2],
}

fn surface_inset() -> f32 {
    0.04
}

fn one_slot() -> u8 {
    1
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InteractionRaw {
    pub(crate) id: String,
    pub(crate) label: String,
    /// Walked along the trail it stands at (`Content::trail_walks`).
    #[serde(default)]
    pub(crate) trail: bool,
    /// Not needed with a `recipe` (its steps take their own time).
    #[serde(default)]
    pub(crate) minutes: f32,
    #[serde(default)]
    pub(crate) recipe: Option<String>,
    /// Mess it leaves (default: by its tags, `roomRules.dirt`).
    #[serde(default)]
    pub(crate) dirt: Option<f32>,
    #[serde(default)]
    pub(crate) baby: bool,
    #[serde(default)]
    pub(crate) needs_table: bool,
    #[serde(default)]
    pub(crate) whole: bool,
    #[serde(default)]
    pub(crate) homework: f32,
    /// Life-stage ids that may take it (see `Interaction::stages`).
    #[serde(default)]
    pub(crate) stages: Option<Vec<String>>,
    #[serde(default)]
    pub(crate) care: HashMap<String, f32>,
    #[serde(default)]
    pub(crate) effects: HashMap<String, f32>,
    #[serde(default)]
    pub(crate) pose: Pose,
    /// Sit on a free seat within this many metres of the object (see `Interaction::seat_near`).
    #[serde(default)]
    pub(crate) seat_near: f32,
    /// That seat faces the object: `front` or `around` (see `Interaction::seat_facing`).
    #[serde(default)]
    pub(crate) seat_facing: Option<String>,
    /// Share of the gains nearby residents get too (see `Interaction::broadcast`).
    #[serde(default)]
    pub(crate) broadcast: f32,
    /// How far that carries when not in the same room (see `Interaction::broadcast_range`).
    #[serde(default)]
    pub(crate) broadcast_range: Option<f32>,
    /// Used standing on the object (see `Interaction::inside`).
    #[serde(default)]
    pub(crate) inside: bool,
    #[serde(default = "yes")]
    pub(crate) autonomous: bool,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    /// Things a working object on the same plot must provide (see `ObjectDef::provides`).
    #[serde(default)]
    pub(crate) requires: Vec<String>,
    /// Skill levels gained per hour.
    #[serde(default)]
    pub(crate) skills: HashMap<String, f32>,
    #[serde(default)]
    pub(crate) cost: i64,
    #[serde(default)]
    pub(crate) skill: Option<String>,
    #[serde(default)]
    #[serde(alias = "moodlet")] // key before the moodlet → feeling rename
    pub(crate) feeling: Option<String>,
    #[serde(default)]
    #[serde(alias = "moodletMinSkill")] // key before the moodlet → feeling rename
    pub(crate) feeling_min_skill: f32,
    /// Animation tag (one of the content's `animations`).
    #[serde(default)]
    pub(crate) anim: Option<String>,
    /// A tag: what's done straight afterwards (see `Interaction::after`).
    #[serde(default)]
    pub(crate) after: Option<String>,
    /// Food to eat sitting down once fetched (see `Interaction::eats`).
    #[serde(default)]
    pub(crate) eat: Option<EatRaw>,
    /// Only offered while the object is so (see `Interaction::when`).
    #[serde(default)]
    pub(crate) when: Option<OpenState>,
    /// What the use leaves the object (see `Interaction::leaves`).
    #[serde(default)]
    pub(crate) leaves: Option<OpenState>,
    #[serde(default)]
    pub(crate) close_after: bool,
    /// What it leaves comes about through the use (see `Interaction::gradual`).
    #[serde(default)]
    pub(crate) gradual: bool,
    /// Fetched first from what provides what it requires (see `Interaction::fetch`).
    #[serde(default)]
    pub(crate) fetch: Option<String>,
    /// Changing clothes (see `Interaction::changes_outfit`).
    #[serde(default)]
    pub(crate) changes_outfit: bool,
    /// Takes its time whatever it fills (see `Interaction::lasts`).
    #[serde(default)]
    pub(crate) lasts: bool,
    /// A spare bed's night (see `Interaction::spare`).
    #[serde(default)]
    pub(crate) spare: bool,
    /// Brings the crop on (see `Interaction::tends`).
    #[serde(default)]
    pub(crate) tends: f32,
    /// Waters every crop on the plot (see `Interaction::waters`).
    #[serde(default)]
    pub(crate) waters: f32,
    /// Fetching the post (see `Interaction::post`).
    #[serde(default)]
    pub(crate) post: bool,
    /// Writing a letter to a friend (see `Interaction::letter`).
    #[serde(default)]
    pub(crate) letter: bool,
    /// A harvest's produce, put in the fridge (see `Interaction::yields`).
    #[serde(default)]
    pub(crate) yields: Option<YieldRaw>,
    /// Done with the baby carried here (see `Interaction::carried`).
    #[serde(default)]
    pub(crate) carried: bool,
    /// Done in front of their own place on it (see `Interaction::at_slot`).
    #[serde(default)]
    pub(crate) at_slot: bool,
    /// Hours of the day it may start in (see `Interaction::hours`).
    #[serde(default)]
    pub(crate) hours: Option<[f32; 2]>,
    /// Only under open sky (see `Interaction::sky`).
    #[serde(default)]
    pub(crate) sky: bool,
    /// Something it works on (see `Interaction::work_on`).
    #[serde(default)]
    pub(crate) work_on: Option<WorkOn>,
    /// A game for two (see `Interaction::partner`).
    #[serde(default)]
    pub(crate) partner: bool,
    /// The use done instead with nobody to play (see `Interaction::alone`).
    #[serde(default)]
    pub(crate) alone: Option<String>,
    /// Loads the machine (see `Interaction::loads`).
    #[serde(default)]
    pub(crate) loads: bool,
    /// Empties the machine once it's done (see `Interaction::unloads`).
    #[serde(default)]
    pub(crate) unloads: bool,
    /// A load it leaves to work on next (see `Interaction::makes`).
    #[serde(default)]
    pub(crate) makes: Option<WorkOn>,
    /// An item carried to it (see `Interaction::brings`).
    #[serde(default)]
    pub(crate) brings: Option<String>,
    /// An item held while it's done (see `Interaction::holds`).
    #[serde(default)]
    pub(crate) holds: Option<String>,
    /// Done behind a shut door (see `Interaction::private`).
    #[serde(default)]
    pub(crate) private: bool,
    /// Where partners try for a baby (see `Interaction::conceive`).
    #[serde(default)]
    pub(crate) conceive: bool,
    /// Sold by someone at the object, and how it's done where nobody serves (see
    /// `Interaction::served`).
    #[serde(default)]
    pub(crate) self_serve: Option<crate::staff::SelfServeRaw>,
    /// Cleans the floor (see `Interaction::cleans`).
    #[serde(default)]
    pub(crate) cleans: Option<SweepRaw>,
}

/// An interaction's `cleans` (see `Sweep`).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SweepRaw {
    pub(crate) amount: f32,
    #[serde(default)]
    pub(crate) reach: u8,
    #[serde(default)]
    pub(crate) hard_floors: bool,
    #[serde(default)]
    pub(crate) outdoors: bool,
}

/// Content `laundry` (see `LaundryRules`).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LaundryRaw {
    pub(crate) per_day: f32,
    pub(crate) most: f32,
    #[serde(default)]
    pub(crate) pressed: Option<PressedRaw>,
}

/// Content `laundry.pressed` (see `Pressed`).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PressedRaw {
    pub(crate) feeling: String,
    pub(crate) outfits: f32,
}

/// An interaction's `yields`: the food item, how many servings, and how long eating one takes.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct YieldRaw {
    pub item: String,
    pub servings: u8,
    pub minutes: f32,
}

/// An object's state for `when` and `leaves` (see `ObjectInstance::open`), or its crop's
/// (`when`: "ripe", `leaves`: "picked"; see `ObjectInstance::growth`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum OpenState {
    Open,
    Closed,
    Ripe,
    Picked,
}

/// An interaction's `eat`: the item handed over and how long eating it takes.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EatRaw {
    pub item: String,
    pub minutes: f32,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct ModifiersRaw {
    pub(crate) need_decay: HashMap<String, f32>,
    pub(crate) need_gain: HashMap<String, f32>,
    pub(crate) tag_preference: HashMap<String, f32>,
    pub(crate) tag_acceptance: HashMap<String, f32>,
    pub(crate) tag_success: HashMap<String, f32>,
    pub(crate) walk_speed: Option<f32>,
    pub(crate) mood: f32,
    pub(crate) skill_gain: HashMap<String, f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TraitRaw {
    pub(crate) id: String,
    pub(crate) label: String,
    #[serde(default)]
    pub(crate) conflicts: Vec<String>,
    #[serde(default)]
    pub(crate) effects: ModifiersRaw,
    #[serde(default)]
    pub(crate) starting_skills: HashMap<String, f32>,
}

#[derive(Deserialize)]
pub(crate) struct PerkRaw {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) cost: u32,
    #[serde(default)]
    pub(crate) effects: ModifiersRaw,
}

#[derive(Deserialize)]
pub(crate) struct GenderRaw {
    pub(crate) id: String,
    pub(crate) label: String,
}

fn one_by_one() -> [u8; 2] {
    [1, 1]
}

fn yes() -> bool {
    true
}

pub(crate) fn intern(tags: &mut Vec<String>, tag: &str) -> Result<(), Error> {
    if !tags.iter().any(|t| t == tag) {
        if tags.len() == MAX_TAGS {
            return Err(Error::new(format!("more than {MAX_TAGS} tags")));
        }
        tags.push(tag.to_owned());
    }
    Ok(())
}

pub(crate) fn build_modifiers(
    raw: &ModifiersRaw,
    ix: &Indices,
    ctx: &str,
) -> Result<Modifiers, Error> {
    let tag = |t: &str| lookup(ix.tags, t, ctx, "tag");
    // Multipliers (decay, gain, preference, skill gain) are at least 0, everything a number.
    let factor = |v: f32, what: &str| {
        if v >= 0.0 && v.is_finite() {
            Ok(v)
        } else {
            Err(Error::new(format!(
                "{ctx}: {what} {v} must be a number ≥ 0"
            )))
        }
    };
    let number = |v: f32, what: &str| {
        if v.is_finite() {
            Ok(v)
        } else {
            Err(Error::new(format!("{ctx}: {what} must be a number")))
        }
    };
    let mut m = Modifiers::default();
    for (need, v) in &raw.need_decay {
        m.need_decay[lookup(ix.needs, need, ctx, "need")?] = factor(*v, "needDecay")?;
    }
    for (need, v) in &raw.need_gain {
        m.need_gain[lookup(ix.needs, need, ctx, "need")?] = factor(*v, "needGain")?;
    }
    for (t, v) in &raw.tag_preference {
        m.tag_preference[tag(t)?] = factor(*v, "tagPreference")?;
    }
    for (t, v) in &raw.tag_acceptance {
        m.tag_acceptance[tag(t)?] = number(*v, "tagAcceptance")?;
    }
    for (t, v) in &raw.tag_success {
        m.tag_success[tag(t)?] = number(*v, "tagSuccess")?;
    }
    for (skill, v) in &raw.skill_gain {
        m.skill_gain[lookup(ix.skills, skill, ctx, "skill")?] = factor(*v, "skillGain")?;
    }
    // Residents walk: a speed of 0 (or less) would leave them walking for ever.
    m.walk_speed = match raw.walk_speed {
        None => 1.0,
        Some(v) if v > 0.0 && v <= MAX_WALK_SPEED => v,
        Some(v) => {
            return Err(Error::new(format!(
                "{ctx}: walkSpeed {v} must be above 0 and at most {MAX_WALK_SPEED}"
            )));
        }
    };
    m.mood = number(raw.mood, "mood")?;
    Ok(m)
}

/// Animation tag for an interaction without an `anim`, from its tags, pose and needs.
/// The caller drops tags that content doesn't declare in `animations`.
pub(crate) fn default_anim(it: &InteractionRaw) -> &'static str {
    let has = |t: &str| it.tags.iter().any(|x| x == t);
    // The need it raises most (ties: first in id order, so the result is deterministic).
    let main_need = it
        .effects
        .iter()
        .filter(|(_, g)| **g > 0.0)
        .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(n, _)| n.as_str());
    let seated = it.pose != Pose::Stand;
    if it.eat.is_some() {
        // Getting food to eat elsewhere.
        "cook"
    } else if has("reading") {
        "read"
    } else if has("cooking") {
        // Cooking at a table is eating what was cooked.
        if seated { "eat" } else { "cook" }
    } else if has("sleep") {
        "sleep"
    } else if has("nap") {
        "nap"
    } else if has("bathroom") || main_need == Some("bladder") {
        "toilet"
    } else if main_need == Some("hunger") {
        "eat"
    } else if has("social") || main_need == Some("social") {
        "talk"
    } else if has("fitness") || has("sport") {
        "exercise"
    } else if has("gaming") {
        "play"
    } else if has("music") {
        "music"
    } else if has("garden") || (has("chores") && has("nature")) {
        "garden"
    } else if has("cleaning") || has("chores") {
        "clean"
    } else if has("spa") || has("hygiene") || main_need == Some("hygiene") {
        match (seated, it.minutes <= 10.0) {
            (true, _) => "bath",
            (false, true) => "wash",
            (false, false) => "shower",
        }
    } else if has("screen") {
        if seated && has("training") {
            "type"
        } else {
            "watch"
        }
    } else if has("food") {
        "drink"
    } else if has("creative") {
        if seated { "write" } else { "paint" }
    } else if has("rest") || has("lounge") || it.pose == Pose::Lie {
        "relax"
    } else if seated {
        "sit"
    } else {
        "idle"
    }
}

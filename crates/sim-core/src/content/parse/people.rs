//! Career, skill, grade and rhythm shapes (content `careers`, `skills`, `grades`).
//!
//! Split from `content/parse.rs` (see `mod.rs`).

use super::*;

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
/// Each fits inside a parking bay's lines with room to spare (`cars::BAY_LINES`, 2.8 by 5.6).
pub(crate) const CAR_LEN: RangeInclusive<f32> = 2.5..=5.4;
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

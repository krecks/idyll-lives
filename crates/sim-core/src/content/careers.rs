//! [(1270, 1465)]
//!
//! Split from `content.rs` (see `mod.rs`).

use super::*;

#[derive(Debug, Clone)]
pub struct CareerLevel {
    pub title: String,
    /// Index into `Content::grades`, for levels generated from career categories.
    pub grade: Option<usize>,
    /// Paid per shift on the standard schedule (`days`). Weekly pay stays the same when
    /// a Sim's skills shorten or lengthen their week; see `life::shift_pay`.
    pub pay: i64,
    pub start_hour: f32,
    pub hours: f32,
    /// Standard workdays as a bitmask (bit 0 = Monday).
    pub days: u8,
    /// Minimum skill levels `(skill, level)`, strongest first. Level 0 entries are kept so
    /// over-qualification can be measured even at entry level.
    pub requires: Vec<(usize, f32)>,
    /// The car the job calls for (`careers.json`: the field's or the track's `car`, by grade, and
    /// the grade's `carAge`); None: none in particular (a household keeps the car it has).
    pub car: Option<CarLook>,
}

/// How old a household's car is, by the grade of the job it fits (`grades[].carAge`): a better,
/// newer car higher up the ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CarAge {
    Old,
    /// Neither old nor new (every car before cars fitted the job).
    #[default]
    Used,
    New,
}

impl CarAge {
    /// As the renderer has it (`shared/cars.ts` `CarAge`): 0 old, 1 used, 2 new.
    pub fn index(self) -> u8 {
        match self {
            CarAge::Old => 0,
            CarAge::Used => 1,
            CarAge::New => 2,
        }
    }

    pub fn is_used(&self) -> bool {
        *self == CarAge::Used
    }
}

/// What a car looks like, but its paint: its body (index into `Content::car_bodies`, the
/// renderer's `CARS`) and how old it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarLook {
    pub body: u8,
    pub age: CarAge,
}

/// A body a household's car may have (`carBodies`, in the renderer's order: `shared/cars.ts`
/// `CARS`): its size, which parking and driving go by.
#[derive(Debug, Clone)]
pub struct CarBodyDef {
    pub id: String,
    /// For the story ("bought a new {car}").
    pub label: String,
    /// Length and width (m).
    pub len: f32,
    pub wid: f32,
}

#[derive(Debug)]
pub struct CareerDef {
    pub id: String,
    pub label: String,
    /// Index into `Content::career_categories`.
    pub category: Option<usize>,
    pub levels: Vec<CareerLevel>,
    /// Need changes over a whole shift (on top of normal decay).
    pub work_effects: [f32; MAX_NEEDS],
    /// Skills practised at work `(skill, weight)`.
    pub skills: Vec<(usize, f32)>,
}

/// A pay grade (A, B, C...): pay per hour and the skill level its jobs expect.
#[derive(Debug, Clone)]
pub struct GradeDef {
    pub id: String,
    pub label: String,
    pub pay_per_hour: f32,
    /// Required level of a track's main skill; other skills scale by their weight.
    pub skill_level: f32,
}

#[derive(Debug)]
pub struct CategoryDef {
    pub id: String,
    pub label: String,
}

/// How many days a Sim works, by how well their skills fit the job. The rule with the
/// highest `fit` at or below the Sim's fit applies.
#[derive(Debug, Clone)]
pub struct WorkweekRule {
    /// Skill levels above (+) or below (-) the job's requirements.
    pub fit: f32,
    /// Days added to (+) or removed from (-) the standard week.
    pub days: i8,
    pub label: String,
}

/// Global work rules.
#[derive(Debug, Clone)]
pub struct CareerRules {
    pub commute_minutes: f32,
    /// How late a Sim may leave before the shift counts as missed.
    pub late_minutes: f32,
    /// Performance lost for a missed shift (at the entry grade: less further up, as shifts count).
    pub missed_penalty: f32,
    /// Performance a shift is worth at the entry grade, in a middling mood with just the skills
    /// asked for (`life::shift_performance` puts it all together).
    pub performance_per_shift: f32,
    /// Extra performance per shift for each skill level above the requirements (or less below).
    pub performance_per_fit: f32,
    /// Extra performance per shift at full mood (and as much less at none).
    pub performance_per_mood: f32,
    /// How much less a shift counts each grade up: worth / (1 + grade index × this). 0: the same
    /// at every grade.
    pub performance_slowdown: f32,
    /// Arriving more than this many minutes after the start, a shift counts `late_share` as much.
    pub late_grace_minutes: f32,
    pub late_share: f32,
    /// How far below a job's requirements a Sim may still be hired (on probation).
    pub probation_levels: f32,
    /// Sorted by `fit`.
    pub workweek: Vec<WorkweekRule>,
    pub promotion_feeling: Option<usize>,
    pub missed_feeling: Option<usize>,
    /// Residents finding, quitting and losing jobs on their own (none without it).
    pub market: Option<JobMarket>,
}

/// How residents with free will find, quit and lose jobs.
#[derive(Debug, Clone)]
pub struct JobMarket {
    /// Hour of the day the job search happens.
    pub hour: f32,
    /// Chance per day that an unemployed resident finds a job (doubled when money is short).
    pub daily_chance: f32,
    /// Job satisfaction (0..1, the mood they leave for work in) below which they may quit.
    pub quit_below: f32,
    pub quit_chance: f32,
    pub min_shifts_before_quit: u32,
    /// Missed shifts in a row (worked shifts make up for them) before being let go.
    pub fire_after_missed: u8,
    /// Days before someone who quit or was let go looks again.
    pub cooldown_days: u32,
    pub hired_feeling: Option<usize>,
    pub quit_feeling: Option<usize>,
    pub fired_feeling: Option<usize>,
}

#[derive(Debug)]
pub struct SkillDef {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct SkillRules {
    pub max_level: f32,
    /// Levels per hour at work for a skill with weight 1.
    pub work_gain_per_hour: f32,
    /// Learning slows with level: gain / (1 + level * slowdown).
    pub slowdown: f32,
    /// How much idle Sims want to practise a skill at all (autonomy).
    pub interest: f32,
    /// Extra pull towards a skill their job needs for probation or the next promotion.
    pub goal_interest: f32,
    /// Interactions with a `skill` give this much more per level of it (0.05 = +5%).
    pub effect_per_level: f32,
    /// Practice gets boring: fatigue gained per hour of practice...
    pub fatigue_per_hour: f32,
    /// ...and lost per hour of doing something else. Interest is divided by `1 + fatigue`.
    pub fatigue_recovery_per_hour: f32,
}

impl SkillRules {
    /// Gain for `hours` of practice at `level`.
    pub fn gain(&self, per_hour: f32, hours: f32, level: f32) -> f32 {
        per_hour * hours / (1.0 + level * self.slowdown)
    }

    /// Multiplier on an interaction's gains for a Sim at `level` (whole levels count).
    pub fn effect_factor(&self, level: f32) -> f32 {
        1.0 + self.effect_per_level * level.floor()
    }
}

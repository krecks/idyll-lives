//! Gameplay content (needs, skills, objects, traits, perks, social life, careers), loaded from JSON.
//!
//! Content files may carry visual fields (`model`, `icon`, ...). The simulation
//! ignores them; they are resolved by the asset registry on the web side.

use std::collections::{HashMap, HashSet};
use std::ops::RangeInclusive;

use serde::Deserialize;

use crate::Error;
use crate::path::StairShape;
use crate::social::{
    self, BondPreset, EmotionDef, EmotionRaw, FeelingDef, FeelingRaw, Indices, SocialDef,
    SocialRaw, SocialRules, SocialRulesRaw,
};

/// Upper bound on the number of needs, so per-Sim needs fit in a fixed array.
pub const MAX_NEEDS: usize = 8;
/// Upper bound on distinct interaction tags (stored as a bitmask).
pub const MAX_TAGS: usize = 64;
/// Most genders content may have (who someone is attracted to is a bit per gender).
pub const MAX_GENDERS: usize = 32;
/// Fastest a trait, perk, feeling or anything else may make someone walk (times the normal pace).
pub const MAX_WALK_SPEED: f32 = 10.0;
/// The range a grade's `payPerHour` may be in: pay becomes shift pay and promotion bonuses, so a
/// pack's figure must stay a number the game can add up (the base game pays $12–60).
const PAY_PER_HOUR: RangeInclusive<f32> = 0.0..=10_000.0;
/// How far a `payFactor` may bend pay: within this a track's pay still resembles its field (the
/// base game keeps to 0.5–1.83), and a pack beyond it would make pay meaningless.
const PAY_FACTOR: RangeInclusive<f32> = 0.25..=3.0;
/// How short of the level a job is hired at on probation: more than a level is never close enough,
/// and a negative shortfall would ask for skills above the job's own.
const PROBATION: RangeInclusive<f32> = 0.0..=1.0;
/// `careerRules.performanceSlowdown`: below 0 later grades would come quicker; past 2 the top
/// grade's shifts count for almost nothing.
const PERFORMANCE_SLOWDOWN: RangeInclusive<f32> = 0.0..=2.0;
/// `careerRules.lateShare`: what a late shift counts for, from nothing to a full shift.
const LATE_SHARE: RangeInclusive<f32> = 0.0..=1.0;
/// Bitmask of indices into `Content::tags`.
pub type TagMask = u64;
/// Bitmask of indices into `LifeRules::stages` (content `stages` rules: who may use what).
pub type StageMask = u32;
/// Most life stages content may have (a `StageMask` bit each).
pub const MAX_STAGES: usize = 32;
/// Upper bound on the number of skills, so per-Sim skills fit in a fixed array.
pub const MAX_SKILLS: usize = 16;
/// Most Sims that can use one object at once (two in a double bed, three on a long sofa, four
/// round a merry-go-round).
pub const MAX_SLOTS: usize = 4;
/// Built-in tag for visiting another household.
pub const VISIT_TAG: &str = "visit";
/// Animation shown during conversations (when content declares it).
pub const TALK_ANIM: &str = "talk";

mod build;
mod careers;
mod cars;
mod character;
mod life;
mod load;
mod objects;
mod parse;
mod rhythm;
mod rooms;
mod rules;

pub use build::*;
pub use careers::*;
pub use cars::*;
pub use character::*;
pub use life::*;
pub(crate) use load::*;
pub use objects::*;
pub(crate) use parse::*;
pub use rhythm::*;
pub use rooms::*;
pub use rules::*;

#[derive(Debug)]
pub struct Content {
    pub needs: Vec<NeedDef>,
    pub objects: Vec<ObjectDef>,
    pub traits: Vec<TraitDef>,
    pub perks: Vec<PerkDef>,
    pub genders: Vec<GenderDef>,
    pub tags: Vec<String>,
    pub rules: CharacterRules,
    pub emotions: Vec<EmotionDef>,
    pub feelings: Vec<FeelingDef>,
    pub socials: Vec<SocialDef>,
    pub bond_presets: HashMap<String, BondPreset>,
    pub social_rules: SocialRules,
    pub careers: Vec<CareerDef>,
    pub career_rules: CareerRules,
    pub schedule: Vec<ScheduleWindow>,
    pub visits: VisitRules,
    pub starting_funds: i64,
    pub skills: Vec<SkillDef>,
    pub skill_rules: SkillRules,
    pub grades: Vec<GradeDef>,
    pub career_categories: Vec<CategoryDef>,
    /// The bodies households' cars come in (`carBodies`), in the renderer's order.
    pub car_bodies: Vec<CarBodyDef>,
    pub day_rhythm: DayRhythm,
    /// How residents weigh their needs when they choose for themselves (`"autonomy"`).
    pub autonomy: crate::ai::AutonomyRules,
    pub rent: Option<RentRules>,
    pub object_rules: ObjectRules,
    pub room_kinds: Vec<RoomKind>,
    pub room_rules: RoomRules,
    pub accidents: Vec<AccidentDef>,
    /// Ages, life stages and life events (content `life`; see `lifecycle.rs`).
    pub life: LifeRules,
    pub names: Names,
    pub build: BuildRules,
    /// Animation tags (`"animations"`): what a Sim can be shown doing. Interactions refer
    /// to them by `anim`; the render snapshot sends indices into this list.
    pub animations: Vec<String>,
    /// Index of the `talk` animation (conversations), if content defines it.
    pub talk_anim: Option<usize>,
    /// Index of the `cuddle` animation (a baby carried is shown as its lift), if content
    /// defines it.
    pub cuddle_anim: Option<usize>,
    /// Whether any feeling has `effects` (Sims then fold them into their modifiers).
    pub buff_feelings: bool,
    /// Whether any object has a use for trying for a baby (`Interaction::conceive`): then the
    /// social of the `conceive` effect goes on in bed, and needs one (`privacy.rs`).
    pub conceive_beds: bool,
    /// What routine blocks can be for, how plans steer residents, and life goals.
    pub activities: Vec<crate::planner::ActivityDef>,
    pub planner: crate::planner::PlannerRules,
    pub goals: Vec<crate::planner::GoalDef>,
    /// Stations, eating and washing up (content `cooking`), the things residents carry and
    /// put down (`items`) and what they cook (`recipes`); see `cooking.rs`.
    pub cooking: crate::cooking::CookingRules,
    pub items: Vec<crate::cooking::ItemDef>,
    pub recipes: Vec<crate::cooking::RecipeDef>,
    /// Everyone's smartphone: who has one and what they do with it (content `phone`; see
    /// `phone.rs`).
    pub phone: crate::phone::PhoneRules,
    /// How the town's vendors serve (content `service`; see `staff.rs`).
    pub service: crate::staff::ServiceRules,
    /// How dirty washing piles up (content `laundry`; None: it never does, and nothing works
    /// on it).
    pub laundry: Option<LaundryRules>,
    /// How cars drive (content `cars`; see `cars.rs`, `travel.rs`).
    pub cars: CarRules,
    /// The work taxi (content `taxi`; see `taxi.rs`).
    pub taxi: TaxiRules,
    /// The uses (object def, interaction) with `"trail": true`: done walking out along the trail
    /// the thing stands at and back (a trailhead's *Walk the trail*; `world::trail_step`).
    pub trail_walks: Vec<(usize, usize)>,
    object_index: HashMap<String, usize>,
}

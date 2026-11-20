//! Life rules: accidents, children, school, newcomers, moving, death, grief and life stages.

use super::*;

/// What happens when a need runs out (content `accidents`): a puddle, falling asleep on the
/// floor, takeout. Instant effects, a mess, a feeling, a cost, and maybe a little while of
/// doing it on the spot (`rest`).
#[derive(Debug, Clone)]
pub struct AccidentDef {
    pub id: String,
    pub need: usize,
    /// Told in the story (content `story` text); quiet ones only bring their feeling.
    pub story: bool,
    /// Instant changes to needs.
    pub effects: [f32; MAX_NEEDS],
    /// Mess on the tile where it happens.
    pub dirt: f32,
    pub feeling: Option<usize>,
    pub cost: i64,
    /// Not again for this long.
    pub cooldown_ticks: u64,
    /// Only after the need has been empty this long when nothing at home fills it (no toilet)...
    pub grace_ticks: u64,
    /// ...or this long when something does but it's busy or out of reach (a crowded bathroom);
    /// None: never then.
    pub crowded_grace_ticks: Option<u64>,
    pub rest: Option<AccidentRest>,
    /// Residents who badly need it and keep finding what fills it taken wish for another room
    /// of this kind (the toilet: a second bathroom).
    pub another_room: Option<usize>,
}

/// Ages and life stages (content `life`).
#[derive(Debug, Clone)]
pub struct LifeRules {
    /// Game days per year of age at the *Normal* lifespan.
    pub days_per_year: f32,
    /// Oldest first stage last; a resident is in the last stage whose `from` they have reached.
    pub stages: Vec<LifeStage>,
    /// Every stage but a baby's (0 without stages).
    pub not_baby: StageMask,
    /// Ages new residents get when nothing says otherwise.
    pub start_age: [f32; 2],
    /// Working residents retire at this age (None: never), with a weekly pension of
    /// `pension` × their last weekly pay (at least `basic_pension`), and the `retire_feeling`.
    pub retire_at: Option<f32>,
    pub pension: f32,
    pub basic_pension: i64,
    pub retire_feeling: Option<usize>,
    /// Passing away of old age (None: nobody does).
    pub death: Option<DeathRules>,
    /// How those left behind feel.
    pub grief: GriefRules,
    /// Moving in together, moving out and leaving town (None: nobody moves on their own).
    pub moving: Option<MovingRules>,
    /// New households for vacant houses (None: nobody new comes).
    pub newcomers: Option<NewcomerRules>,
    /// School for stages with `school` (None: no school).
    pub school: Option<SchoolRules>,
    /// Having children (None: nobody does).
    pub pregnancy: Option<PregnancyRules>,
    /// Adopting a baby or a child: what it costs (None: no adoption).
    pub adoption_cost: Option<i64>,
    /// The daily chance a household of grown-ups without children adopts on its own.
    pub adoption_chance: f32,
    /// A babysitter for a baby left with no grown-up at home while they're out (None: nobody
    /// looks after them, and nobody stays home for them either).
    pub childcare: Option<ChildcareRules>,
}

/// A babysitter (content `life.childcare`): booked for the day, for `cost`, once the last
/// grown-up at home leaves a baby (for work, a visit, anywhere); while no grown-up is home, they
/// feed, change and play with the baby (the care interactions of the baby's crib) whenever a
/// need those fill is below `below`. Without the money for one, the last grown-up stays home
/// from work, and can't be sent off the lot (on a visit, to the park). On their own, the last
/// one doesn't go out at all.
#[derive(Debug, Clone, Copy)]
pub struct ChildcareRules {
    pub cost: i64,
    pub below: f32,
}

/// A successful *Try for a baby* means a baby with a chance of `chance`, born `days` later, for
/// partners at home who are both at most `max_age`.
#[derive(Debug, Clone, Copy)]
pub struct PregnancyRules {
    pub chance: f32,
    pub days: u32,
    pub max_age: f32,
}

/// School days: away from `start` for `hours` on `days` (bit 0 = Monday), practising `skills`
/// (skill, weight) a little.
#[derive(Debug, Clone)]
pub struct SchoolRules {
    pub start: f32,
    pub hours: f32,
    pub days: u8,
    pub skills: Vec<(usize, f32)>,
    /// Grade points (0..100) for a day at school, and up to `mood` more (or less) by how they
    /// felt; `missed` off for a day not there. New pupils start at `start_grade`.
    pub attend: f32,
    pub mood: f32,
    pub missed: f32,
    pub start_grade: f32,
    /// On leaving school: the feeling for a final grade of at least `good`, else `poor_feeling`
    /// below `poor` (see `lifecycle`).
    pub good: f32,
    pub good_feeling: Option<usize>,
    pub poor: f32,
    pub poor_feeling: Option<usize>,
}

impl SchoolRules {
    /// A grade's letter: A from 85, B 70, C 50, D 30, else F.
    pub fn letter(grade: f32) -> char {
        match grade {
            g if g >= 85.0 => 'A',
            g if g >= 70.0 => 'B',
            g if g >= 50.0 => 'C',
            g if g >= 30.0 => 'D',
            _ => 'F',
        }
    }
}

/// Once a day at `hour`, each vacant house gets a household with a chance of 1 in `days`.
#[derive(Debug, Clone, Copy)]
pub struct NewcomerRules {
    pub hour: f32,
    pub days: f32,
}

/// Names for residents the simulation makes up (newcomers): content `names`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Names {
    /// First names for anyone, and by gender id.
    pub first: Vec<String>,
    pub by_gender: HashMap<String, Vec<String>>,
    /// Household (family) names.
    pub last: Vec<String>,
}

/// Once a day at `hour`: partners in different homes move in together (`partners` chance a
/// day once their romance is at least `romance`), residents living with a parent move out at
/// `leave_home_age` (`leave_home` chance a day, with a job), and with no house free, from
/// `leave_town_age` they may leave town (`leave_town` chance a day).
#[derive(Debug, Clone, Copy)]
pub struct MovingRules {
    pub hour: f32,
    pub partners: f32,
    pub romance: f32,
    pub leave_home_age: f32,
    pub leave_home: f32,
    pub leave_town_age: f32,
    pub leave_town: f32,
}

/// From `from` years, a yearly chance of `per_year`, growing by `growth` (×e^growth) a year.
#[derive(Debug, Clone, Copy)]
pub struct DeathRules {
    pub from: f32,
    pub per_year: f32,
    pub growth: f32,
}

impl DeathRules {
    /// The chance of passing away within a year at `age`.
    pub fn yearly_chance(&self, age: f32) -> f32 {
        if age < self.from {
            0.0
        } else {
            (self.per_year * (self.growth * (age - self.from)).exp()).min(1.0)
        }
    }
}

/// Family, partners and friends with at least `close` friendship get `feeling`; friends with at
/// least `friend` get `light_feeling`.
#[derive(Debug, Clone, Copy, Default)]
pub struct GriefRules {
    pub feeling: Option<usize>,
    pub light_feeling: Option<usize>,
    pub close: f32,
    pub friend: f32,
}

#[derive(Debug, Clone)]
pub struct LifeStage {
    pub id: String,
    pub label: String,
    /// Age in years the stage starts at.
    pub from: f32,
    /// Combined with traits and perks (elders tire sooner, walk slower).
    pub mods: Modifiers,
    /// Granted on reaching the stage.
    pub feeling: Option<usize>,
    /// Grown up: may work, fall in love, move, have children (default: unless `baby` or `school`).
    pub adult: bool,
    /// Lives in a crib and is cared for; doesn't choose anything.
    pub baby: bool,
    /// Goes to school on school days.
    pub school: bool,
    /// Their size against a grown-up's (content `scale`, which the renderer draws them at).
    pub scale: f32,
}

/// How high a grown-up's hand reaches standing (m): a wall cupboard's top shelf.
pub const ADULT_REACH: f32 = 2.1;

impl LifeStage {
    /// How high their hand reaches standing (m): a 1.1 m child about 1.3 m.
    pub fn reach(&self) -> f32 {
        ADULT_REACH * self.scale
    }
}

impl LifeRules {
    /// Index of the stage at `age` (0 without stages).
    pub fn stage(&self, age: f32) -> usize {
        self.stages.iter().rposition(|s| age >= s.from).unwrap_or(0)
    }

    /// The stage at `age`, if content has stages.
    pub fn stage_at(&self, age: f32) -> Option<&LifeStage> {
        self.stages.get(self.stage(age))
    }

    /// Grown up at `age` (everyone is, without stages).
    pub fn adult(&self, age: f32) -> bool {
        self.stage_at(age).is_none_or(|s| s.adult)
    }

    /// A baby at `age`.
    pub fn baby(&self, age: f32) -> bool {
        self.stage_at(age).is_some_and(|s| s.baby)
    }

    /// How high a hand reaches at `age` (m; a grown-up's without stages).
    pub fn reach(&self, age: f32) -> f32 {
        self.stage_at(age).map_or(ADULT_REACH, LifeStage::reach)
    }

    /// The age adults start at.
    pub fn adult_age(&self) -> f32 {
        self.stages.iter().find(|s| s.adult).map_or(0.0, |s| s.from)
    }
}

/// A little while of doing it on the spot (sleeping on the floor, eating takeout).
#[derive(Debug, Clone)]
pub struct AccidentRest {
    pub label: String,
    pub minutes: f32,
    pub pose: Pose,
    pub anim: Option<usize>,
    pub gain_per_minute: [f32; MAX_NEEDS],
    pub tags: TagMask,
}

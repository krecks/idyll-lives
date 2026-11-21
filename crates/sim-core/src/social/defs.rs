//! Definitions
//!
//! Social interaction, feeling, emotion and rule definitions, and their raw JSON shapes.

use super::*;

pub const ANIMATIONS: [&str; 8] = [
    "talk", "laugh", "flirt", "argue", "fight", "hug", "kiss", "wave",
];

/// Friendship levels used for statuses and milestone events.
pub const LEVEL_ENEMY: i8 = -2;
pub const LEVEL_DISLIKED: i8 = -1;
pub const LEVEL_FRIEND: i8 = 1;
pub const LEVEL_GOOD_FRIEND: i8 = 2;
pub const LEVEL_BEST_FRIEND: i8 = 3;
/// Romance at or above this is a crush.
pub const CRUSH_ROMANCE: f32 = 35.0;

pub fn friend_level(friendship: f32) -> i8 {
    match friendship {
        f if f <= -60.0 => LEVEL_ENEMY,
        f if f <= -25.0 => LEVEL_DISLIKED,
        f if f < 35.0 => 0,
        f if f < 60.0 => LEVEL_FRIEND,
        f if f < 80.0 => LEVEL_GOOD_FRIEND,
        _ => LEVEL_BEST_FRIEND,
    }
}

// ---- Definitions ---------------------------------------------------------------------

#[derive(Debug)]
pub struct EmotionDef {
    pub id: String,
    pub label: String,
    /// Applied on top of trait modifiers while this is the Sim's dominant emotion.
    pub mods: Modifiers,
}

#[derive(Debug)]
pub struct FeelingDef {
    pub id: String,
    pub label: String,
    pub emotion: Option<usize>,
    /// Added to mood while active.
    pub mood: f32,
    pub minutes: f32,
    /// Buff or debuff while active, folded into the Sim's modifiers (`Sim::mods`).
    pub effects: Option<Modifiers>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Prefer {
    /// No relationship bias.
    #[default]
    Any,
    /// More likely toward people the actor likes.
    Liked,
    /// More likely toward people the actor dislikes.
    Disliked,
    /// Driven by romance and chemistry.
    Romance,
    /// Only toward a partner the actor no longer loves.
    Unloved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Effect {
    #[default]
    None,
    BecomePartners,
    BreakUp,
    Fight,
    Kiss,
    /// Partners trying for a baby (the life cycle decides whether one is on the way).
    Conceive,
    /// The actor brings the target something to eat or drink from home (a host's offer taken
    /// up by a guest: `hosting.rs`).
    Serve,
}

/// Conditions on the actor's view of the target (and their shared status).
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Requirements {
    /// `Some(false)`: only between strangers; `Some(true)`: only once they've met.
    pub met: Option<bool>,
    pub min_friendship: f32,
    pub max_friendship: f32,
    pub min_romance: f32,
    pub max_romance: f32,
    pub partners: Option<bool>,
}

impl Default for Requirements {
    fn default() -> Self {
        Self {
            met: Some(true),
            min_friendship: -100.0,
            max_friendship: 100.0,
            min_romance: -100.0,
            max_romance: 100.0,
            partners: None,
        }
    }
}

impl Requirements {
    pub fn allows(&self, rel: &Relationship) -> bool {
        self.met.is_none_or(|m| m == rel.met)
            && self.partners.is_none_or(|p| p == rel.partners)
            && (self.min_friendship..=self.max_friendship).contains(&rel.friendship)
            && (self.min_romance..=self.max_romance).contains(&rel.romance)
    }
}

/// Success chance terms. Relationship values are the target's view of the actor.
#[derive(Debug, Clone, Copy)]
pub struct Acceptance {
    pub base: f32,
    /// Per friendship point.
    pub friendship: f32,
    /// Per romance point.
    pub romance: f32,
    /// Per unit of chemistry (-1..1).
    pub chemistry: f32,
    /// Per unit of target mood above 0.5.
    pub mood: f32,
    /// Subtracted when the target is partnered with someone else.
    pub taken_penalty: f32,
    /// Per unit the target is short of each need (1 - the need; content `needs`, need → weight):
    /// a hungry guest says yes to a bite to eat.
    pub needs: [f32; MAX_NEEDS],
}

impl Default for Acceptance {
    fn default() -> Self {
        Self {
            base: 0.8,
            friendship: 0.0,
            romance: 0.0,
            chemistry: 0.0,
            mood: 0.0,
            taken_penalty: 0.0,
            needs: [0.0; MAX_NEEDS],
        }
    }
}

/// `Acceptance` as content writes it (needs by id).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct AcceptanceRaw {
    pub(super) base: f32,
    pub(super) friendship: f32,
    pub(super) romance: f32,
    pub(super) chemistry: f32,
    pub(super) mood: f32,
    pub(super) taken_penalty: f32,
    pub(super) needs: HashMap<String, f32>,
}

impl Default for AcceptanceRaw {
    fn default() -> Self {
        let a = Acceptance::default();
        Self {
            base: a.base,
            friendship: a.friendship,
            romance: a.romance,
            chemistry: a.chemistry,
            mood: a.mood,
            taken_penalty: a.taken_penalty,
            needs: HashMap::new(),
        }
    }
}

pub(crate) fn parse_acceptance(
    raw: &AcceptanceRaw,
    ix: &Indices,
    ctx: &str,
) -> Result<Acceptance, Error> {
    Ok(Acceptance {
        base: raw.base,
        friendship: raw.friendship,
        romance: raw.romance,
        chemistry: raw.chemistry,
        mood: raw.mood,
        taken_penalty: raw.taken_penalty,
        needs: need_array(&raw.needs, ix, ctx)?,
    })
}

#[derive(Debug, Clone)]
pub struct Outcome {
    /// Change to the actor's feelings toward the target.
    pub friendship: f32,
    pub romance: f32,
    /// Change to the target's feelings toward the actor.
    pub target_friendship: f32,
    pub target_romance: f32,
    pub feeling: Option<usize>,
    pub target_feeling: Option<usize>,
    /// Immediate need changes (may be negative, e.g. after a fight).
    pub needs: [f32; MAX_NEEDS],
    pub target_needs: [f32; MAX_NEEDS],
    pub effect: Effect,
    /// For fights.
    pub winner_feeling: Option<usize>,
    pub loser_feeling: Option<usize>,
}

#[derive(Debug)]
pub struct SocialDef {
    pub id: String,
    pub label: String,
    pub category: String,
    pub minutes: f32,
    /// How close the two stand while interacting (metres).
    pub distance: f32,
    /// Snapshot animation code (index into `ANIMATIONS` + 1).
    pub anim: u8,
    pub autonomous: bool,
    pub autonomy_weight: f32,
    /// Others nearby may join in and listen (a group conversation; see `conversation/`).
    pub joinable: bool,
    pub tags: TagMask,
    pub requires: Requirements,
    /// The life stages that may start it, and that it may be started with (content `stages`,
    /// `targetStages`; default everyone): nobody starts a fight with a child.
    pub stages: StageMask,
    pub target_stages: StageMask,
    pub acceptance: Acceptance,
    pub prefer: Prefer,
    /// Need gains over the whole interaction.
    pub actor_gain: [f32; MAX_NEEDS],
    pub target_gain: [f32; MAX_NEEDS],
    pub success: Outcome,
    pub failure: Outcome,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BondPreset {
    pub friendship: f32,
    pub romance: f32,
    pub partners: bool,
    /// Family: what b is to a in a bond (a, b).
    pub kin: Kin,
}

/// Global social settings.
#[derive(Debug, Clone)]
pub struct SocialRules {
    /// Interactions with these tags can't be interrupted by socials (sleeping, bathroom...).
    pub busy_tags: TagMask,
    /// Of the busy tags, those a guest may still use at the host's (the toilet).
    pub guest_tags: TagMask,
    pub romantic_tags: TagMask,
    pub default_bond: Option<BondPreset>,
    pub jealousy_range: f32,
    pub jealousy_friendship: f32,
    pub jealousy_romance: f32,
    pub jealousy_feeling: Option<usize>,
    pub heartbreak_feeling: Option<usize>,
    /// Daily drift toward neutral.
    pub friendship_decay: f32,
    pub romance_decay: f32,
    /// How drawn two single grown-ups who are attracted to each other and have met are to
    /// romance before there is any (added to the flirting preference; see `prefer_factor`).
    pub single_spark: f32,
    /// Friendship each member of a group conversation gains with each other member (both ways)
    /// when a joinable social goes well with listeners there (the pair talking gains it with
    /// the listeners, not with each other: their social's outcome does that).
    pub join_friendship: f32,
}

// ---- Raw JSON --------------------------------------------------------------------------

#[derive(Deserialize)]
pub(crate) struct EmotionRaw {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub effects: ModifiersRaw,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FeelingRaw {
    pub(super) id: String,
    pub(super) label: String,
    #[serde(default)]
    pub(super) emotion: Option<String>,
    #[serde(default)]
    pub(super) mood: f32,
    pub(super) hours: f32,
    #[serde(default)]
    pub(super) effects: Option<ModifiersRaw>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SocialRaw {
    pub(super) id: String,
    pub(super) label: String,
    #[serde(default)]
    pub(super) category: String,
    pub(super) minutes: f32,
    #[serde(default = "default_distance")]
    pub(super) distance: f32,
    #[serde(default)]
    pub(super) animation: Option<String>,
    #[serde(default = "yes")]
    pub(super) autonomous: bool,
    #[serde(default = "one")]
    pub(super) autonomy_weight: f32,
    #[serde(default)]
    pub(super) joinable: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub(super) requires: Requirements,
    #[serde(default)]
    pub(super) acceptance: AcceptanceRaw,
    #[serde(default)]
    pub(super) prefer: Prefer,
    #[serde(default)]
    pub(super) needs: HashMap<String, f32>,
    #[serde(default)]
    pub(super) target_needs: HashMap<String, f32>,
    #[serde(default)]
    pub(super) success: OutcomeRaw,
    #[serde(default)]
    pub(super) failure: OutcomeRaw,
    /// Life-stage ids that may start it, and that it may be started with (see
    /// `SocialDef::stages`).
    #[serde(default)]
    pub(super) stages: Option<Vec<String>>,
    #[serde(default)]
    pub(super) target_stages: Option<Vec<String>>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct OutcomeRaw {
    pub(super) friendship: f32,
    pub(super) romance: f32,
    pub(super) target_friendship: f32,
    pub(super) target_romance: f32,
    #[serde(alias = "moodlet")] // key before the moodlet → feeling rename
    pub(super) feeling: Option<String>,
    #[serde(alias = "targetMoodlet")] // key before the moodlet → feeling rename
    pub(super) target_feeling: Option<String>,
    pub(super) needs: HashMap<String, f32>,
    pub(super) target_needs: HashMap<String, f32>,
    pub(super) effect: Effect,
    #[serde(alias = "winnerMoodlet")] // key before the moodlet → feeling rename
    pub(super) winner_feeling: Option<String>,
    #[serde(alias = "loserMoodlet")] // key before the moodlet → feeling rename
    pub(super) loser_feeling: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct SocialRulesRaw {
    pub(super) busy_tags: Vec<String>,
    pub(super) guest_tags: Vec<String>,
    pub(super) romantic_tags: Vec<String>,
    pub(super) default_bond: Option<String>,
    pub(super) jealousy_range: Option<f32>,
    pub(super) jealousy_friendship: Option<f32>,
    pub(super) jealousy_romance: Option<f32>,
    #[serde(alias = "jealousyMoodlet")] // key before the moodlet → feeling rename
    pub(super) jealousy_feeling: Option<String>,
    #[serde(alias = "heartbreakMoodlet")] // key before the moodlet → feeling rename
    pub(super) heartbreak_feeling: Option<String>,
    pub(super) friendship_decay: Option<f32>,
    pub(super) romance_decay: Option<f32>,
    pub(super) single_spark: Option<f32>,
    pub(super) join_friendship: Option<f32>,
}

fn default_distance() -> f32 {
    1.0
}
fn yes() -> bool {
    true
}
fn one() -> f32 {
    1.0
}

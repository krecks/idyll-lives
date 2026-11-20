//! Room kinds, room rules, cleaning, and the modifiers a room gives.

use super::*;

/// What a room is for, from what stands in it (content `roomKinds`, in order of priority).
#[derive(Debug, Clone)]
pub struct RoomKind {
    pub id: String,
    pub label: String,
    /// A room is of this kind if an object in it offers one of these tags.
    pub tags: TagMask,
    /// Each entry must be offered by something in the room for it to work fully (a bathroom:
    /// `bathroom` and `hygiene`).
    pub essentials: Vec<TagMask>,
    /// `[cramped, comfortable]` in tiles.
    pub size: [f32; 2],
    /// A room can't be two exclusive kinds at once (a bed in the kitchen makes it `mixed`).
    pub exclusive: bool,
    /// Else (no kind's tags offered in it), a room is of this kind if people pass through it:
    /// two doors or more, or stairs (a hallway).
    pub passage: bool,
    /// Kept shut while someone is in it (content `private`: a bathroom; see `privacy.rs`):
    /// others wait outside or go elsewhere, and its doors stay shut to them.
    pub private: bool,
    /// A private room of this kind is its people's own (content `claimed`: a bedroom): whoever has
    /// a place in it — somewhere to lie, or a baby's place and the care of it, with their partner
    /// and children — comes and goes, and its door keeps everyone else out (`privacy.rs`). Only
    /// with `private`, which is what keeps the door shut at all, and never a passage.
    pub claimed: bool,
    /// Its floor is carpet where no covering was laid (content `carpet`: a bedroom, as the
    /// renderer draws it): soft, what a mop or a broom leaves (`Sweep::hard`).
    pub carpet: bool,
}

/// How rooms are scored (content `roomRules`; see `rooms.rs`).
#[derive(Debug, Clone)]
pub struct RoomRules {
    /// Weights of size, light, decor, cleanliness and function in the overall score.
    pub weights: [f32; 5],
    /// Tiles one window lights fully by day, and one lamp by night (unless its `light` says how
    /// many: `ObjectDef::lamp`).
    pub window_tiles: f32,
    pub lamp_tiles: f32,
    /// Decor points per tile for full decor, indoors and in the garden.
    pub decor_per_tile: f32,
    pub garden_decor_per_tile: f32,
    /// A resident loves a room above this score, and dislikes one below this.
    pub love: f32,
    pub dislike: f32,
    /// How fast the environment need moves towards where the resident is (per hour).
    pub drift: f32,
    /// Environment need at work or away.
    pub away: f32,
    /// How much the AI prefers doing things in better rooms (see `World::pick_autonomous`).
    pub preference: f32,
    /// Feelings from loving or disliking the room they're in (see `rooms::opinions`).
    pub love_feeling: Option<usize>,
    pub dislike_feeling: Option<usize>,
    /// Mess per use by interaction tag (`roomRules.dirt`).
    pub dirt: Vec<(TagMask, f32)>,
    pub clean: CleanRules,
}

/// Tidying up: residents clean the dirtiest spot at home (`roomRules.clean`).
#[derive(Debug, Clone)]
pub struct CleanRules {
    pub label: String,
    pub minutes: f32,
    /// Spots dirtier than this are worth cleaning.
    pub threshold: f32,
    /// Dirt taken off the spot and the tiles around it.
    pub amount: f32,
    /// How keen residents are (like an interaction's need gain).
    pub interest: f32,
    /// Its tags (traits and the planner's chores see it like an interaction with these).
    pub tags: TagMask,
    pub anim: Option<usize>,
}

/// Multipliers and offsets a trait, perk or emotion applies to a Sim.
#[derive(Debug, Clone, PartialEq)]
pub struct Modifiers {
    pub need_decay: [f32; MAX_NEEDS],
    pub need_gain: [f32; MAX_NEEDS],
    /// Autonomy preference per tag: >1 seeks out, <1 avoids.
    pub tag_preference: [f32; MAX_TAGS],
    /// Added to the success chance of socials with this tag when this Sim is the *target*.
    pub tag_acceptance: [f32; MAX_TAGS],
    /// Added to the success chance of socials with this tag when this Sim is the *actor*.
    pub tag_success: [f32; MAX_TAGS],
    pub walk_speed: f32,
    /// Added to mood (0..1 scale).
    pub mood: f32,
    /// Learning speed per skill.
    pub skill_gain: [f32; MAX_SKILLS],
}

impl Default for Modifiers {
    fn default() -> Self {
        Self {
            need_decay: [1.0; MAX_NEEDS],
            need_gain: [1.0; MAX_NEEDS],
            tag_preference: [1.0; MAX_TAGS],
            tag_acceptance: [0.0; MAX_TAGS],
            tag_success: [0.0; MAX_TAGS],
            walk_speed: 1.0,
            mood: 0.0,
            skill_gain: [1.0; MAX_SKILLS],
        }
    }
}

impl Modifiers {
    pub fn combine(&mut self, other: &Modifiers) {
        for i in 0..MAX_NEEDS {
            self.need_decay[i] *= other.need_decay[i];
            self.need_gain[i] *= other.need_gain[i];
        }
        for i in 0..MAX_TAGS {
            self.tag_preference[i] *= other.tag_preference[i];
            self.tag_acceptance[i] += other.tag_acceptance[i];
            self.tag_success[i] += other.tag_success[i];
        }
        for i in 0..MAX_SKILLS {
            self.skill_gain[i] *= other.skill_gain[i];
        }
        self.walk_speed *= other.walk_speed;
        self.mood += other.mood;
    }

    /// Combined preference for an interaction with the given tag mask.
    pub fn preference(&self, tags: TagMask) -> f32 {
        each_tag(tags).map(|i| self.tag_preference[i]).product()
    }

    pub fn acceptance(&self, tags: TagMask) -> f32 {
        each_tag(tags).map(|i| self.tag_acceptance[i]).sum()
    }

    pub fn success(&self, tags: TagMask) -> f32 {
        each_tag(tags).map(|i| self.tag_success[i]).sum()
    }
}

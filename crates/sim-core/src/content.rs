//! Gameplay content (needs, skills, objects, traits, perks, social life, careers), loaded from JSON.
//!
//! Content files may carry visual fields (`model`, `icon`, ...). The simulation
//! ignores them; they are resolved by the asset registry on the web side.

use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use crate::Error;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Pose {
    #[default]
    Stand,
    Sit,
    Lie,
}

impl Pose {
    /// Stable numeric code used in the render snapshot.
    pub fn code(self) -> f32 {
        match self {
            Pose::Stand => 0.0,
            Pose::Sit => 1.0,
            Pose::Lie => 2.0,
        }
    }
}

#[derive(Debug)]
pub struct NeedDef {
    pub id: String,
    pub label: String,
    pub decay_per_minute: f32,
    /// The environment need: instead of decaying it drifts towards the score of the room the
    /// resident is in (`roomRules.drift`).
    pub room: bool,
    /// The level residents like to keep it at (content `comfortable`, 0 = none): below it they
    /// see to it when it's convenient, it weighs more (`AutonomyRules::comfort_boost`).
    pub comfortable: f32,
}

#[derive(Debug)]
pub struct Interaction {
    pub id: String,
    pub label: String,
    pub minutes: f32,
    /// Total gain per need over the full duration.
    pub total_gain: [f32; MAX_NEEDS],
    pub gain_per_minute: [f32; MAX_NEEDS],
    pub pose: Pose,
    /// Sit on a free seat within this many metres of the object, if there is one (content
    /// `seatNear`: watching TV from the sofa). 0: use it standing, as `pose` says.
    pub seat_near: f32,
    /// The seat taken for `seat_near` faces the object (content `seatFacing`): a screen's from
    /// in front of it, a fire's from any side. Without one like that they use it as `pose` says.
    pub seat_facing: Option<SeatFacing>,
    /// Share (0..1) of this use's need gains that nearby residents get too (content
    /// `broadcast`: hearing the TV while eating). 0: nobody else.
    pub broadcast: f32,
    /// How far (m) `broadcast` carries when it isn't in the same room (content `broadcastRange`).
    pub broadcast_range: f32,
    /// Used standing on the object rather than in front of it (content `inside`: a shower):
    /// the resident steps onto its centre, facing out, and back off where they came from.
    pub inside: bool,
    /// Whether Sims may choose this on their own.
    pub autonomous: bool,
    pub tags: TagMask,
    /// Things that must be provided by a working object on the same plot (content `requires`;
    /// see `ObjectDef::provides`): a table's coffee needs a coffee machine.
    pub requires: TagMask,
    /// What it `requires` is fetched first, with this interaction (by id) of the nearest object
    /// on the plot that provides it (content `fetch`): the coffee poured at the machine and
    /// brought to the table, the cake taken from the bakery case; then the use goes on here.
    pub fetch: Option<String>,
    /// Skill levels gained per hour of use (before the per-level slowdown).
    pub skill_gain: [f32; MAX_SKILLS],
    /// Dollars charged from the Sim's household when the Sim starts using the object.
    pub cost: i64,
    /// Skill that makes this interaction better: need and skill gains scale with its level.
    pub skill: Option<usize>,
    /// Feeling granted on finishing (after at least half the duration)...
    pub feeling: Option<usize>,
    /// ...if the Sim's level in `skill` is at least this.
    pub feeling_min_skill: f32,
    /// What the Sim is shown doing (index into `Content::animations`): the interaction's
    /// `anim`, else one derived from its tags, pose and needs (see `default_anim`).
    pub anim: Option<usize>,
    /// Mess it leaves around the object when finished (0..1 of a tile's dirt): its `dirt`,
    /// else the most any of its tags makes (`roomRules.dirt`).
    pub dirt: f32,
    /// The place a baby lies (only babies use it; nobody else ever).
    pub baby: bool,
    /// Only offered while the seat faces a table it tucks under (content `needsTable`; see
    /// `ObjectDef::seat_at` and `surface.rs`): dining at a table.
    pub needs_table: bool,
    /// Takes every slot of the object (content `whole`): only started with nobody else on it,
    /// and while it lasts nobody else gets on (a nap along a 2-seat window seat).
    pub whole: bool,
    /// Homework: grade points for a full use (only pupils do it).
    pub homework: f32,
    /// The life stages that may take it (content `stages` on the interaction, else on its
    /// recipe, else on its object; default every stage but a baby's, and for a `baby` use only
    /// a baby's): the kids' furniture is for children, wine for grown-ups. Free will and the
    /// player's orders keep it alike (`world::unfit`).
    pub stages: StageMask,
    /// Starts this recipe (index into `Content::recipes`): the resident goes round the
    /// recipe's stations (see `cooking.rs`) instead of using this object on its own. Its
    /// `total_gain` is then what a serving gives plus its own effects (what cooking gives the
    /// cook, spread over `minutes`, the whole recipe's time).
    pub recipe: Option<usize>,
    /// Needs this fills for the baby lying in the object, over the whole interaction (feeding,
    /// changing); only with a baby there, and only grown-ups do it.
    pub care: [f32; MAX_NEEDS],
    /// Done afterwards (content `after`, a tag): once a use finishes, the resident goes on to
    /// the nearest object with an interaction of that tag (washing hands after the toilet).
    pub after: TagMask,
    /// Food fetched here and eaten sitting down (content `eat`; index into
    /// `Content::recipes`, a hidden recipe made for it): `minutes` is the fetching, and what
    /// it gives (its `total_gain`) comes from eating, at a table if there is one.
    pub eats: Option<usize>,
    /// Only offered while the object is open (`Some(true)`) or closed (`Some(false)`) (content
    /// `when`: "open" / "closed"; see `ObjectInstance::open`).
    pub when: Option<bool>,
    /// Opens (`Some(true)`) or closes (`Some(false)`) the object as the use starts, and leaves it
    /// so (content `leaves`): folding a wall bed down, going to sleep in it. Nothing closes an
    /// object someone else is using.
    pub leaves: Option<bool>,
    /// Opening it is for this use only (content `closeAfter`, with `leaves`: "open"): when it
    /// ends, the last one done closes it again, as after a night's sleep (a hot tub's cover goes
    /// back on after a soak).
    pub close_after: bool,
    /// Only offered while the object's crop is ripe (content `when`: "ripe"; see
    /// `ObjectInstance::growth`).
    pub ripe: bool,
    /// Picks the crop (content `leaves`: "picked"): it lies fallow, then is planted again and
    /// grows (see `ObjectDef::grows`). Picking takes the hands the whole use: the crop goes when
    /// it ends, not as it starts.
    pub picks: bool,
    /// What it `leaves` comes about through the use, as the hands get there, and is so once it
    /// ends (content `gradual`): the washing hung out piece by piece, or taken in. The renderer
    /// plays the object's clip along with the use (a model's `motions.uses` entry with `over`).
    /// Without it, what it leaves is so as the use starts (a wall bed folded down to lie in).
    pub gradual: bool,
    /// Changing clothes (content `changesOutfit`: at a wardrobe, a chest of drawers): halfway
    /// through, the resident is in their next outfit (`Sim::outfit`), and the use lasts its
    /// time even with the needs it fills full.
    pub changes_outfit: bool,
    /// A spare bed's night (content `spare`, on a `dayRhythm.sleepTags` use: a sofa bed pulled
    /// out for a guest): it doesn't make the room a bedroom (its tags aren't the object's
    /// `tags`, what the room is for), and residents sleep in it only with no other bed free for
    /// them.
    pub spare: bool,
    /// Brings the object's crop on (content `tends`: weeding and watering): a use done to the
    /// end grows it this share of its way from planted to ripe at once (less for one cut
    /// short), while it grows.
    pub tends: f32,
    /// A harvest's produce (content `yields`): food carried in to the home's fridge and put away
    /// (with no fridge, eaten sitting down as `eats`); see `Yield`.
    pub yields: Option<Yield>,
    /// Done with the baby carried here from their crib (content `carried`: feeding at the high
    /// chair), and back after: a grown-up doing the crib's care of the same id by day does it
    /// here instead, when this is free (`carry.rs`).
    pub carried: bool,
    /// Done standing in front of the place the resident takes on the object (content `atSlot`):
    /// a crib's changing table beside it, not the cot.
    pub at_slot: bool,
    /// Only started between these hours of the day (content `hours`: `[from, to]`, 0..24; past
    /// midnight when `from` is later than `to`): stargazing after dark. Free will and the
    /// player's orders keep it alike (`world::unmet`), as all the conditions below.
    pub hours: Option<[f32; 2]>,
    /// Only under open sky (content `sky`): the object stands outdoors, every tile of it.
    pub sky: bool,
    /// Only with something to work on (content `workOn`): washing up with dirty dishes,
    /// the laundry with washing to do.
    pub work_on: Option<WorkOn>,
    /// A game for two (content `partner`, on an object with two places): the seesaw, ping-pong,
    /// a game of pool. Whoever starts it fetches someone free nearby to play (`partners.rs`), who
    /// goes to the other place; it starts once both are there, gives both what it gives, and is
    /// over for both when either stops. With nobody free it isn't started.
    pub partner: bool,
    /// What the player's order to play it does with nobody free (content `alone`: a use of the
    /// same object, by id; index into its `interactions`): trick shots at the pool table, a
    /// chess problem. Without one the order is refused.
    pub alone: Option<usize>,
}

/// Something a use works on (content `workOn`), which must be there for it to start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkOn {
    /// Dirty dishes or food gone off, the household's, to clear away (`ItemInstance::to_clear`).
    Dishes,
    /// A load of the household's dirty washing (`Household::laundry`, content `laundry`): the
    /// use washes it, and it waits to be dried (`Washed`).
    Laundry,
    /// A load the household washed (`Household::washed`): the use dries and folds it.
    Washed,
}

impl WorkOn {
    pub const ALL: [WorkOn; 3] = [WorkOn::Dishes, WorkOn::Laundry, WorkOn::Washed];

    /// Its bit in a mask of what there is to work on (`world::work_waiting`).
    pub fn bit(self) -> u8 {
        1 << self as u8
    }

    /// What there is none of, as the player is told.
    pub fn none(self) -> &'static str {
        match self {
            WorkOn::Dishes => "there are no dishes to wash",
            WorkOn::Laundry => "there's no washing to do",
            WorkOn::Washed => "there's no washed laundry to dry",
        }
    }
}

/// Whether `hour` (0..24) is in the window `[from, to]` (past midnight when `from` > `to`).
pub fn in_hours(window: [f32; 2], hour: f32) -> bool {
    let [from, to] = window;
    if from <= to { (from..to).contains(&hour) } else { hour >= from || hour < to }
}

/// Dirty washing piling up (content `laundry`): what the laundry's uses work on
/// (`WorkOn::Laundry`, `WorkOn::Washed`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaundryRules {
    /// Loads of washing each resident at home makes a day.
    pub per_day: f32,
    /// Most loads that pile up (dirty, and washed waiting to dry, each).
    pub most: f32,
}

/// A harvest's produce (content `yields`): `servings` of a food item (that of `recipe`, a hidden
/// recipe made for it, as for `eat`: eating a serving gives the use's own gains), put in the
/// fridge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Yield {
    pub recipe: usize,
    pub servings: u8,
}

impl Interaction {
    /// Whether this looks after a baby lying in the object.
    pub fn cares(&self) -> bool {
        self.care.iter().any(|&g| g > 0.0)
    }

    /// The place a baby lies, asleep or awake (a crib's): where babies are put, not a high
    /// chair they're carried to.
    pub fn baby_bed(&self) -> bool {
        self.baby && self.pose == Pose::Lie
    }

    /// Whether someone in life stage `stage` (`LifeRules::stage`) may take it (`stages`).
    pub fn for_stage(&self, stage: usize) -> bool {
        stage_in(self.stages, stage)
    }

    /// Bitmask of the needs this fills (a gain above 0).
    pub fn fills(&self) -> u32 {
        (0..MAX_NEEDS).filter(|&n| self.total_gain[n] > 0.0).fold(0, |m, n| m | 1 << n)
    }

    pub fn trains_skills(&self) -> bool {
        self.skill_gain.iter().any(|&g| g > 0.0)
    }

    /// Multiplier on gains for a Sim with `skills` (1 without a `skill`).
    pub fn skill_factor(&self, rules: &SkillRules, skills: &[f32; MAX_SKILLS]) -> f32 {
        self.skill.map_or(1.0, |s| rules.effect_factor(skills[s]))
    }

    /// Whether finishing after `elapsed` minutes earns the feeling.
    pub fn earns_feeling(&self, elapsed: f32, skills: &[f32; MAX_SKILLS]) -> Option<usize> {
        let skilled = self
            .skill
            .is_none_or(|s| skills[s].floor() >= self.feeling_min_skill);
        self.feeling
            .filter(|_| skilled && elapsed >= self.minutes * 0.5)
    }
}

/// How a crop grows (content `grows`): game days from planted to ripe, and lying fallow after
/// it is picked, before it is planted again.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Growth {
    pub days: f32,
    pub fallow_days: f32,
}

#[derive(Debug)]
pub struct ObjectDef {
    pub id: String,
    pub name: String,
    /// Footprint in tiles, `[width (x), depth (z)]`, before rotation. Local +z is the front.
    pub footprint: [u8; 2],
    pub interactions: Vec<Interaction>,
    /// Purchase price; objects without one can't be bought.
    pub price: Option<i64>,
    /// How many Sims can use it at once (1..=MAX_SLOTS).
    pub slots: u8,
    /// Where each slot's user is and which way they face (content `places`, one a slot), or empty:
    /// spread across its width, facing its front (`ObjectInstance::slot_position`).
    pub places: Vec<SlotPlace>,
    /// Can only be placed outdoors (trees, flower beds, ponds).
    pub outdoors: bool,
    /// A flight of stairs up to the storey above (one tile wide, climbed from the front; see
    /// `path::Stair`). Residents walk on it instead of around it.
    pub stairs: bool,
    /// Turns freely (decor, plants): besides its four facings it can stand at any angle
    /// (`ObjectInstance::turn`). The angle is only for looks; the footprint stays on tiles.
    pub turns: bool,
    /// Decor points it gives the room it stands in (plants, lamps, decor; see `RoomRules`).
    pub decor: f32,
    /// Floor tiles it lights at night, as its room's light is scored (content `light.tiles`, else
    /// `roomRules.lampTiles`): a chandelier lights more of a room than a table lamp. 0: it gives
    /// no light (no `light`).
    pub lamp: f32,
    /// A lamp that can be switched on and off (a `light` without `"switch": false`; see
    /// `lights.rs`).
    pub switch: bool,
    /// A fire (content `light.fire`): switched as a lamp is, but lit by someone who sits by it (a
    /// use of it lights it when it's dark) or the player, never by coming into the room; it burns
    /// down by morning (see `lights.rs`).
    pub fire: bool,
    /// Wear per finished use (it breaks at 1; see `ObjectRules::wear_per_quality`).
    pub wear_per_use: f32,
    /// Every tag its interactions have (what the room it stands in can be used for).
    pub tags: TagMask,
    /// Things it provides while it stands here working (content `provides`): a coffee machine
    /// provides coffee, so a table can offer *Have a coffee* (see `Interaction::requires`).
    pub provides: TagMask,
    /// Where it goes: on the floor like furniture, or on a wall, the ceiling or flat on the floor
    /// as a rug, where it takes no floor space (content `layer`).
    pub layer: Layer,
    /// Its two slots are bunks, one above the other (content `bunk`): both users lie in the
    /// middle of its width, and the renderer lifts the second to the upper bunk.
    pub bunk: bool,
    /// Who may take the upper bunk (content `upperStages`; default everyone who may use the
    /// bed): with the lower bunk someone else's, nobody else gets in.
    pub upper_stages: StageMask,
    /// A top that small things stand on (content `surface`): tables, counters, desks,
    /// nightstands, shelves. Things on the `Surface` layer stand on it; plated food too.
    pub surface: Option<Surface>,
    /// Things on the `Surface` layer: the radius (m) of the disc they take on their host's top
    /// (content `radius`, default `DEFAULT_RADIUS`).
    pub radius: f32,
    /// Where it is used from (content `approach`): the tile in front, or (chairs pushed under a
    /// table) any free tile around it.
    pub approach: Approach,
    /// Seats: the kinds of surface it tucks under (content `seatAt`, a mask of
    /// `SurfaceKind::bit`). Facing one of those it is a seat at that table.
    pub seat_at: u8,
    /// The cooking stations it is (content `stations`, a mask of `cooking.stations`): the
    /// fridge for ingredients, a counter to prep on, a stove, a sink to wash up (`cooking.rs`).
    pub stations: crate::cooking::StationMask,
    /// Keeps leftovers (content `stores`: fridges); food served where there's no free
    /// surface goes in.
    pub stores: bool,
    /// Buy catalog group (`""`: none): things of one group with a use in common are the same
    /// kind of thing (see `planner::wanted_object`).
    pub group: String,
    /// A crop that grows (content `grows`): bought just planted (ripe in a lot's or a house's
    /// file), picked by interactions that leave it "picked", then fallow, planted again and
    /// growing (see `ObjectInstance::growth`).
    pub grows: Option<Growth>,
    /// Things on a wall: it belongs over a window (content `overWindow`: curtains, blinds), so
    /// its back may be on a window as well as on a plain wall. Everything else on a wall needs
    /// plain wall behind it.
    pub over_window: bool,
    /// Furniture that fills the wall's height (content `fillsWall`: a wardrobe, a tall bookcase,
    /// a fireplace's chimney breast): nothing hangs on the wall or from the ceiling over its
    /// tiles.
    pub fills_wall: bool,
    /// Things from the ceiling that hang low enough to walk into (content `hangsLow`: a hanging
    /// plant): it takes the floor under it as furniture does.
    pub hangs_low: bool,
    /// Things from the ceiling that hang at head height (content `headHeight`: a chandelier, a
    /// low pendant), to hang over a table: nobody walks under it, but furniture may stand under
    /// it (not furniture as tall as the wall).
    pub head_height: bool,
    /// Furniture built to stand with its back on a wall (content `againstWall`: a wall-hung
    /// toilet's cistern box, a wall bed, a ladder leaning on the wall): it needs plain wall all
    /// along its back, and that wall can't come down (nor become a window or a door) under it.
    pub against_wall: bool,
    /// How high a hand goes to use it as a station (content `reach`, m: a wall cupboard's
    /// bottom shelf; 0, within anyone's reach): someone who can't reach that high goes to
    /// another station (`LifeStage::reach`).
    pub reach: f32,
    /// A footstool (content `footstool`: a pouf, an ottoman): someone sitting down alone on a
    /// sofa takes the seat with one in front (`world::footstool_slots`).
    pub footstool: bool,
    /// Tiles beside its own it reaches over at head height (content `overhang`, `[right, front]`
    /// tiles from it; a 1×1 thing on the floor only): an arc lamp's shade over the sofa. As under
    /// a lamp at head height (`head_height`), furniture may stand there and nobody walks there
    /// (`ObjectInstance::overhang_tiles`).
    pub overhang: Vec<(i32, i32)>,
}

impl ObjectDef {
    /// Whether building must leave a way to it: it has uses, and doesn't stand on a top (a
    /// kettle is reached as its counter is; nothing is refused for the way to it).
    pub fn reached(&self) -> bool {
        !self.interactions.is_empty() && self.layer != Layer::Surface
    }

    /// Off the floor and out of the way: residents walk under (or over) it (a picture, a ceiling
    /// lamp, a rug), unless it hangs low or at head height.
    pub fn walked_under(&self) -> bool {
        self.layer.mounted() && !self.hangs_low && !self.head_height
    }

    /// The slots interaction `i` takes one of. A crib's first slot is the baby's (the crib on
    /// its left, where `ObjectInstance::slot_position` lays them; the changing table beside it
    /// is where carers stand), whoever gets there first: after loading a save, a carer may set
    /// off for the crib before the baby is put back in it.
    pub fn slots_for(&self, i: usize) -> std::ops::Range<usize> {
        let slots = self.slots as usize;
        if self.interactions[i].baby {
            0..1
        } else if self.interactions.iter().any(|it| it.baby) {
            1.min(slots.saturating_sub(1))..slots
        } else {
            0..slots
        }
    }
}

/// Whether `stage` is one of `mask`'s.
pub fn stage_in(mask: StageMask, stage: usize) -> bool {
    1u32.checked_shl(stage as u32).is_some_and(|bit| mask & bit != 0)
}

/// Whether every stage of `mask` is older than `stage` (someone of `stage` is too young for it;
/// else too old, or too big).
pub fn all_older(mask: StageMask, stage: usize) -> bool {
    mask != 0 && mask & 1u32.checked_shl(stage as u32 + 1).map_or(u32::MAX, |b| b - 1) == 0
}

/// Life-stage ids (`stages` rules) as a mask of `stage_ids`, content `life.stages` in order.
pub(crate) fn stage_mask(stage_ids: &[&str], ids: &Option<Vec<String>>, ctx: &str) -> Result<Option<StageMask>, Error> {
    let Some(ids) = ids else { return Ok(None) };
    if ids.is_empty() {
        return Err(Error::new(format!("{ctx}: `stages` names at least one life stage")));
    }
    ids.iter()
        .try_fold(0, |m: StageMask, id| match stage_ids.iter().position(|s| s == id) {
            Some(i) => Ok(m | 1 << i),
            None => Err(Error::new(format!("{ctx}: unknown life stage '{id}' (see life.stages)"))),
        })
        .map(Some)
}

/// Where a slot's user sits, lies or stands (content `places`): metres off the footprint's middle
/// in the object's own frame (`[right, front]`, before turning and for its own size), and the way
/// they face, turned from its front towards its right (radians: `FRAC_PI_2` faces its right, `PI`
/// its back).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SlotPlace {
    pub at: [f32; 2],
    pub facing: f32,
}

/// Which seats a `seatNear` use takes (content `seatFacing`): one that looks at the thing from
/// in front of it (`front`: a screen) or from any side (`around`: a fire).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeatFacing {
    Front,
    Around,
}

/// How far (m) past its footprint's edge a slot's place may be (standing at a ping-pong table's end, at the
/// oche two metres out from the dartboard).
pub const PLACE_REACH: f32 = 2.0;

/// Radius of a thing on the `Surface` layer when content gives none (m).
pub const DEFAULT_RADIUS: f32 = 0.12;

/// A top that small things stand on (content `surface`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Surface {
    /// Height of the top above the floor (m).
    pub height: f32,
    pub kind: SurfaceKind,
    /// Margin (m) kept clear along the top's edges.
    pub inset: f32,
    /// Width and depth of the top (m; content `size`, default the footprint's): a desk's top
    /// is smaller than the floor it stands on.
    pub size: [f32; 2],
    /// Where the top's middle is off the footprint's middle (m; content `at`, to its right and
    /// front): a counter's top is at the back of its tile.
    pub at: [f32; 2],
}

/// What kind of top a surface is: which seats tuck under it (content `surface.kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceKind {
    Table,
    Bar,
    Counter,
    Desk,
    Low,
    Shelf,
}

impl SurfaceKind {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "table" => Self::Table,
            "bar" => Self::Bar,
            "counter" => Self::Counter,
            "desk" => Self::Desk,
            "low" => Self::Low,
            "shelf" => Self::Shelf,
            _ => return None,
        })
    }

    /// Its bit in `ObjectDef::seat_at`.
    pub fn bit(self) -> u8 {
        1 << self as u8
    }
}

/// Where an object is used from (content `approach`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Approach {
    /// The tile centred in front of it (`ObjectInstance::front_tile`).
    #[default]
    Front,
    /// Any free tile beside it, the front first (chairs pushed under a table).
    Around,
    /// Its seats and beds got onto from the side (`surface::seat_approach`): beside the slot (a
    /// bed beside its sleeper's middle; an upper bunk from the front), else the front. Used standing, from the front.
    Side,
    /// Its seat behind the work (a desk's chair, a drum kit's throne): from behind the slot, else
    /// beside it, else the front.
    Behind,
}

/// Where an object goes (content `layer`). Things on a wall, the ceiling or the floor's surface
/// (rugs) take no floor space: residents walk under and over them, furniture stands in front of,
/// under or on them, and only things on the same layer can't overlap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layer {
    /// Furniture: stands on its tiles and blocks them.
    #[default]
    Floor,
    /// Hangs on a wall (its back edge must be a wall or a window): pictures, shelves, sconces.
    Wall,
    /// Hangs from the ceiling of a room (indoors only): pendant lamps, fans.
    Ceiling,
    /// Lies flat on the floor: rugs.
    Rug,
    /// Stands on another object's `surface` (a table, counter, shelf): vases, books, lamps.
    Surface,
}

impl Layer {
    /// Off the furniture layer: it takes no floor space.
    pub fn mounted(self) -> bool {
        self != Layer::Floor
    }
}

/// Most accidents content can define (`Sim::accident_at` keeps one cooldown each).
pub const MAX_ACCIDENTS: usize = 8;

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

/// Ages and life stages (content `life`; see `docs/design/life-cycle.md`).
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
        if age < self.from { 0.0 } else { (self.per_year * (self.growth * (age - self.from)).exp()).min(1.0) }
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

fn each_tag(mask: TagMask) -> impl Iterator<Item = usize> {
    let mut bits = mask;
    std::iter::from_fn(move || {
        if bits == 0 {
            return None;
        }
        let i = bits.trailing_zeros() as usize;
        bits &= bits - 1;
        Some(i)
    })
}

#[derive(Debug)]
pub struct TraitDef {
    pub id: String,
    pub label: String,
    pub conflicts: Vec<String>,
    pub mods: Modifiers,
    /// Skill levels a new Sim with this trait starts with.
    pub starting_skills: [f32; MAX_SKILLS],
}

#[derive(Debug)]
pub struct PerkDef {
    pub id: String,
    pub label: String,
    pub cost: u32,
    pub mods: Modifiers,
}

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
    pub missed_penalty: f32,
    /// Performance gained per shift at neutral mood; mood shifts it up or down.
    pub performance_per_shift: f32,
    /// Extra performance per shift for each skill level above the requirements (or less below).
    pub performance_per_fit: f32,
    /// Extra performance per shift at full mood (and as much less at none).
    pub performance_per_mood: f32,
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

/// Day and night: when Sims go to bed and get up, and how sleep changes their needs.
#[derive(Debug, Clone)]
pub struct DayRhythm {
    pub wake_hour: f32,
    pub bed_hour: f32,
    /// Interactions with any of these tags count as sleep.
    pub sleep_tags: TagMask,
    /// Need decay multipliers while awake at night (tiredness builds up).
    pub night_decay: [f32; MAX_NEEDS],
    /// Need decay multipliers while asleep.
    pub asleep_decay: [f32; MAX_NEEDS],
    /// Need decay multipliers while at work (there's lunch and a bathroom there; the job's
    /// `workEffects` come on top).
    pub work_decay: [f32; MAX_NEEDS],
    /// Needs that wake a sleeper when they get urgent (bit per need; default: all).
    pub wake_for: u32,
    /// Sims with an early shift get up this long before they leave.
    pub wake_before_work_minutes: f32,
    /// A rested sleeper stays in bed at night until this many hours before they'd get up.
    pub lie_in_hours: f32,
    /// What each star of a bed adds to sleep (instead of `ObjectRules::quality_bonus`: a better
    /// bed makes a night a little shorter, not half as long).
    pub sleep_quality_bonus: f32,
    /// The need a night's sleep is for (the one beds fill most: energy). Sleepers get up once
    /// it's full.
    pub rest_need: Option<usize>,
    /// Interactions with any of these tags are naps (short, and not again too soon).
    pub nap_tags: TagMask,
    /// No nap for this long after the last one.
    pub nap_cooldown_hours: f32,
    /// Below this (in what a bed's sleep fills), daytime is no reason not to go to bed, and a
    /// nap won't do.
    pub exhausted_below: f32,
    /// What residents see to at the turning points of their day (content `habits`).
    pub habits: Vec<Habit>,
    /// Habits before leaving for work or school start this long before they leave.
    pub ready_minutes: f32,
    /// What residents see to while away at work or school (content `dayRhythm.workBreaks`).
    pub work_breaks: Vec<WorkBreak>,
    /// A crying baby wakes a grown-up asleep at home (content `dayRhythm.babyWakes`; None:
    /// nobody wakes for them).
    pub baby_wakes: Option<BabyWakes>,
}

/// A baby crying for one of `needs` (bit per need; default: all but surroundings) below
/// `below` while every grown-up at home is asleep wakes one of them, who does the crib's care
/// interactions for each of those needs below `tend_below` and goes back to bed, with the
/// `feeling` (optional); at night, a grown-up at home who's up and free comes instead (a cook
/// with at most `finish_minutes` of a step left finishes it first). The same baby calls nobody
/// again for `cooldown_ticks`.
#[derive(Debug, Clone, Copy)]
pub struct BabyWakes {
    pub needs: u32,
    pub below: f32,
    pub tend_below: f32,
    pub cooldown_ticks: u64,
    pub finish_minutes: f32,
    pub feeling: Option<usize>,
}

/// A break at work or school: once `need` drops below `below`, it gains `gain` (lunch, the
/// toilet, a coffee). `cost` is paid from the household at work (school lunch is free); a
/// break they can't afford is skipped.
#[derive(Debug, Clone, Copy)]
pub struct WorkBreak {
    pub need: usize,
    pub below: f32,
    pub gain: f32,
    pub cost: i64,
}

/// When a habit is kept (`Habit`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HabitTime {
    /// On getting up after a night's sleep.
    Wake,
    /// On the way to bed.
    Bed,
    /// In the `ready_minutes` before leaving for work or school.
    Leave,
    /// On getting home from work or school.
    Home,
    /// After a meal (a serving eaten).
    Meal,
}

/// Something residents see to at a turning point of their day (content `dayRhythm.habits`):
/// unless `need` is at `below` or better already, they first go to the nearest quick thing
/// that fills it (the toilet on getting up, before bed, before leaving, on getting home and
/// after a meal).
#[derive(Debug, Clone)]
pub struct Habit {
    pub at: HabitTime,
    pub need: usize,
    pub below: f32,
}

impl DayRhythm {
    /// Night for a Sim who gets up at `wake`.
    pub fn is_night(&self, hour: f32, wake: f32) -> bool {
        !(wake..self.bed_hour).contains(&hour)
    }

    /// Multiplier on a night's sleep in a bed of `quality` stars.
    pub fn sleep_quality_factor(&self, quality: u8) -> f32 {
        1.0 + self.sleep_quality_bonus * quality as f32
    }
}

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

#[derive(Debug)]
pub struct GenderDef {
    pub id: String,
    pub label: String,
}

/// Limits for character creation.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CharacterRules {
    pub min_traits: usize,
    pub max_traits: usize,
    pub perk_points: u32,
    pub max_household: usize,
}

impl Default for CharacterRules {
    fn default() -> Self {
        Self {
            min_traits: 0,
            max_traits: 3,
            perk_points: 5,
            max_household: 4,
        }
    }
}

/// Buying, upgrading and selling objects.
#[derive(Debug, Clone)]
pub struct ObjectRules {
    /// Each quality level takes this share off wear per use.
    pub wear_per_quality: f32,
    pub repair: RepairRules,
    pub max_quality: u8,
    /// Need and skill gains grow by this fraction per quality level.
    pub quality_bonus: f32,
    /// One quality level (bought instantly in Buy mode) costs this fraction of the price.
    pub upgrade_cost: f32,
    /// Share of the price and paid upgrades returned when selling.
    pub resale: f32,
}

/// Repairing broken objects: by a resident (a chore that trains a skill) or a paid quick fix.
#[derive(Debug, Clone)]
pub struct RepairRules {
    pub label: String,
    pub minutes: f32,
    /// Skill that helps (and is trained), e.g. handiness.
    pub skill: Option<usize>,
    /// Skill levels gained per hour of repairing.
    pub skill_gain: f32,
    /// Chance an attempt works at skill 0, and what each level adds (capped at 0.95).
    pub chance: f32,
    pub chance_per_level: f32,
    /// How keen residents are to fix things (like a need gain).
    pub interest: f32,
    /// Quick fix in Buy mode: this share of the price, times how worn it is.
    pub cost: f32,
    pub tags: TagMask,
    pub anim: Option<usize>,
}

impl RepairRules {
    /// Chance an attempt works for someone with `level` in the repair skill.
    pub fn chance_at(&self, level: f32) -> f32 {
        (self.chance + self.chance_per_level * level).min(0.95)
    }
}

impl ObjectRules {
    /// Multiplier on need and skill gains for an object of `quality`.
    pub fn quality_factor(&self, quality: u8) -> f32 {
        1.0 + self.quality_bonus * quality as f32
    }

    /// Parts cost to raise an object with `price` to the next quality level.
    pub fn upgrade_price(&self, price: i64) -> i64 {
        (price as f32 * self.upgrade_cost).round() as i64
    }
}

/// Prices for build mode, and the looks walls, doors and windows come in.
#[derive(Debug, Clone)]
pub struct BuildRules {
    pub wall: i64,
    pub door: i64,
    pub window: i64,
    /// Removing a wall, door or window.
    pub remove: i64,
    /// Wall coverings (`wallCoverings`): paint, wallpaper, brick... A wall face's covering is an
    /// index into this list + 1; 0 is the automatic look (siding outside, wallpaper inside).
    pub coverings: Vec<BuildStyle>,
    /// Door styles (`doorStyles`); a door's style is an index into this list (0 when empty).
    pub doors: Vec<BuildStyle>,
    /// Window styles (`windowStyles`), like doors.
    pub windows: Vec<BuildStyle>,
    /// Floor coverings (`floorCoverings`): wood, tile, carpet... A tile's covering is an index
    /// into this list + 1; 0 is the automatic look (by what the room is used for).
    pub floors: Vec<BuildStyle>,
    /// A metre of fence, and a gate, when there are no fence styles.
    pub fence: i64,
    pub gate: i64,
    /// Fence styles (`fenceStyles`): picket, ranch rails, slats... A fence's (or gate's) style is
    /// an index into this list; its `price` is per metre, a gate costs `gate` on top of it.
    pub fences: Vec<BuildStyle>,
    /// Roof styles (`roofStyles`: gable, hip, flat...) and colours (`roofColors`) a home's roof
    /// can have; the renderer reads the rest of each entry (shape, pitch, colour).
    pub roofs: Vec<BuildStyle>,
    pub roof_colors: Vec<BuildStyle>,
    /// How many storeys a house may have (`build.storeys`, 1..=4; 1: no stairs).
    pub storeys: u8,
    /// Whether a house may have a basement, a storey below the ground (`build.basement`).
    pub basement: bool,
}

/// A look for walls, doors or windows. The simulation only needs its id (saves) and price; the
/// renderer reads the rest of the content entry (finish, colour, glazing...).
#[derive(Debug, Clone)]
pub struct BuildStyle {
    pub id: String,
    pub price: i64,
}

impl BuildRules {
    /// A diagonal wall spans a tile corner to corner (√2 m): `wall × 1.414`, rounded.
    pub fn diagonal_wall(&self) -> i64 {
        (self.wall as f64 * 1.414).round() as i64
    }

    /// A metre of fence in `style`.
    pub fn fence_price(&self, style: u8) -> i64 {
        self.fences.get(style as usize).map_or(self.fence, |s| s.price)
    }

    /// A gate in `style`: a metre of that fence plus the gate.
    pub fn gate_price(&self, style: u8) -> i64 {
        self.fence_price(style) + self.gate
    }

    /// Price of a door in `style` (the plain door price without styles).
    pub fn door_price(&self, style: u8) -> i64 {
        self.doors
            .get(style as usize)
            .map_or(self.door, |s| s.price)
    }

    /// Price of a window in `style`.
    pub fn window_price(&self, style: u8) -> i64 {
        self.windows
            .get(style as usize)
            .map_or(self.window, |s| s.price)
    }

    /// Price of covering one wall face (0, the automatic look, is free).
    pub fn covering_price(&self, covering: u8) -> i64 {
        covering
            .checked_sub(1)
            .and_then(|i| self.coverings.get(i as usize))
            .map_or(0, |c| c.price)
    }

    /// Price of covering one floor tile (0, the automatic look, is free).
    pub fn floor_price(&self, covering: u8) -> i64 {
        covering
            .checked_sub(1)
            .and_then(|i| self.floors.get(i as usize))
            .map_or(0, |c| c.price)
    }
}

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
    /// How dirty washing piles up (content `laundry`; None: it never does, and nothing works
    /// on it).
    pub laundry: Option<LaundryRules>,
    object_index: HashMap<String, usize>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentFile {
    needs: Vec<NeedRaw>,
    objects: Vec<ObjectRaw>,
    #[serde(default)]
    traits: Vec<TraitRaw>,
    #[serde(default)]
    perks: Vec<PerkRaw>,
    #[serde(default)]
    genders: Vec<GenderRaw>,
    #[serde(default)]
    rules: CharacterRules,
    #[serde(default)]
    emotions: Vec<EmotionRaw>,
    #[serde(default)]
    #[serde(alias = "moodlets")] // key before the moodlet → feeling rename
    feelings: Vec<FeelingRaw>,
    #[serde(default)]
    socials: Vec<SocialRaw>,
    #[serde(default)]
    bond_presets: HashMap<String, BondPreset>,
    #[serde(default)]
    social_rules: SocialRulesRaw,
    #[serde(default)]
    careers: Vec<CareerRaw>,
    #[serde(default)]
    career_rules: CareerRulesRaw,
    #[serde(default)]
    schedule: Vec<ScheduleRaw>,
    #[serde(default)]
    visits: VisitRaw,
    #[serde(default)]
    economy: EconomyRaw,
    #[serde(default)]
    skills: Vec<SkillRaw>,
    #[serde(default)]
    skill_rules: SkillRulesRaw,
    #[serde(default)]
    grades: Vec<GradeRaw>,
    #[serde(default)]
    career_categories: Vec<CategoryRaw>,
    #[serde(default)]
    day_rhythm: DayRhythmRaw,
    #[serde(default)]
    autonomy: crate::ai::AutonomyRaw,
    #[serde(default)]
    object_rules: ObjectRulesRaw,
    #[serde(default)]
    room_kinds: Vec<RoomKindRaw>,
    #[serde(default)]
    accidents: Vec<AccidentRaw>,
    #[serde(default)]
    life: LifeRaw,
    #[serde(default)]
    names: Names,
    #[serde(default)]
    room_rules: RoomRulesRaw,
    #[serde(default)]
    build: BuildRaw,
    #[serde(default)]
    wall_coverings: Vec<BuildStyleRaw>,
    #[serde(default)]
    door_styles: Vec<BuildStyleRaw>,
    #[serde(default)]
    window_styles: Vec<BuildStyleRaw>,
    #[serde(default)]
    fence_styles: Vec<BuildStyleRaw>,
    #[serde(default)]
    roof_styles: Vec<BuildStyleRaw>,
    #[serde(default)]
    roof_colors: Vec<BuildStyleRaw>,
    #[serde(default)]
    floor_coverings: Vec<BuildStyleRaw>,
    /// Extra tags that no interaction or social uses (yet), so traits and emotions can refer to them.
    #[serde(default)]
    tags: Vec<String>,
    /// Animation tags interactions may use as `anim`.
    #[serde(default)]
    animations: Vec<String>,
    #[serde(default)]
    activities: Vec<crate::planner::ActivityRaw>,
    #[serde(default)]
    planner: crate::planner::PlannerRaw,
    #[serde(default)]
    goals: Vec<crate::planner::GoalRaw>,
    #[serde(default)]
    cooking: crate::cooking::CookingRaw,
    #[serde(default)]
    items: Vec<crate::cooking::ItemRaw>,
    #[serde(default)]
    recipes: Vec<crate::cooking::RecipeRaw>,
    #[serde(default)]
    phone: crate::phone::PhoneRaw,
    #[serde(default)]
    laundry: Option<LaundryRaw>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CareerRaw {
    id: String,
    label: String,
    levels: Vec<CareerLevelRaw>,
    #[serde(default)]
    work_effects: HashMap<String, f32>,
    /// Skills practised at work, with weights.
    #[serde(default)]
    skills: HashMap<String, f32>,
}

#[derive(Deserialize)]
struct SkillRaw {
    id: String,
    label: String,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct SkillRulesRaw {
    max_level: Option<f32>,
    work_gain_per_hour: Option<f32>,
    slowdown: Option<f32>,
    interest: Option<f32>,
    goal_interest: Option<f32>,
    effect_per_level: Option<f32>,
    fatigue_per_hour: Option<f32>,
    fatigue_recovery_per_hour: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GradeRaw {
    id: String,
    label: String,
    pay_per_hour: f32,
    skill_level: f32,
}

#[derive(Deserialize, Clone, Copy)]
struct ShiftRaw {
    start: f32,
    hours: f32,
    days: Days,
}

/// Weekday indices (0 = Monday), stored inline so shifts stay `Copy`.
#[derive(Clone, Copy)]
struct Days(u8, bool);

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
struct CategoryRaw {
    id: String,
    label: String,
    #[serde(default = "one")]
    pay_factor: f32,
    shift: ShiftRaw,
    #[serde(default)]
    work_effects: HashMap<String, f32>,
    tracks: Vec<TrackRaw>,
}

#[derive(Deserialize)]
struct TrackRaw {
    id: String,
    label: String,
    skills: HashMap<String, f32>,
    #[serde(default)]
    shift: Option<ShiftRaw>,
    titles: Vec<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct DayRhythmRaw {
    wake_hour: Option<f32>,
    bed_hour: Option<f32>,
    sleep_tags: Vec<String>,
    night_decay: HashMap<String, f32>,
    asleep_decay: HashMap<String, f32>,
    work_decay: HashMap<String, f32>,
    wake_for: Option<Vec<String>>,
    wake_before_work_minutes: Option<f32>,
    lie_in_hours: Option<f32>,
    sleep_quality_bonus: Option<f32>,
    nap_tags: Vec<String>,
    nap_cooldown_hours: Option<f32>,
    exhausted_below: Option<f32>,
    habits: Vec<HabitRaw>,
    ready_minutes: Option<f32>,
    work_breaks: Vec<WorkBreakRaw>,
    baby_wakes: Option<BabyWakesRaw>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BabyWakesRaw {
    #[serde(default)]
    needs: Option<Vec<String>>,
    #[serde(default)]
    below: Option<f32>,
    #[serde(default)]
    tend_below: Option<f32>,
    #[serde(default)]
    cooldown_minutes: Option<f32>,
    #[serde(default)]
    finish_minutes: Option<f32>,
    #[serde(default)]
    feeling: Option<String>,
}

#[derive(Deserialize)]
struct WorkBreakRaw {
    need: String,
    below: f32,
    gain: f32,
    #[serde(default)]
    cost: i64,
}

#[derive(Deserialize)]
struct HabitRaw {
    at: HabitTime,
    need: String,
    #[serde(default = "one_f32")]
    below: f32,
}

fn one_f32() -> f32 {
    1.0
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RentRaw {
    #[serde(default = "sunday")]
    weekday: u32,
    #[serde(default = "noon")]
    hour: f32,
    base: i64,
    #[serde(default)]
    per_tile: f32,
    #[serde(default)]
    bills_base: i64,
    #[serde(default)]
    bills_rate: f32,
    #[serde(default)]
    per_resident: i64,
    #[serde(default)]
    #[serde(alias = "debtMoodlet")] // key before the moodlet → feeling rename
    debt_feeling: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct LifeRaw {
    days_per_year: Option<f32>,
    stages: Vec<LifeStageRaw>,
    start_age: Option<[f32; 2]>,
    retire_at: Option<f32>,
    pension: Option<f32>,
    basic_pension: Option<i64>,
    retire_feeling: Option<String>,
    death: Option<DeathRaw>,
    grief: GriefRaw,
    moving: Option<MovingRaw>,
    newcomers: Option<NewcomersRaw>,
    school: Option<SchoolRaw>,
    pregnancy: Option<PregnancyRaw>,
    adoption: Option<AdoptionRaw>,
    childcare: Option<ChildcareRaw>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChildcareRaw {
    cost: i64,
    #[serde(default)]
    below: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PregnancyRaw {
    chance: f32,
    days: u32,
    #[serde(default)]
    max_age: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdoptionRaw {
    cost: i64,
    #[serde(default)]
    neighbours: f32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SchoolRaw {
    start: f32,
    hours: f32,
    days: Vec<u8>,
    #[serde(default)]
    skills: HashMap<String, f32>,
    #[serde(default)]
    grades: GradesRaw,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct GradesRaw {
    attend: Option<f32>,
    mood: Option<f32>,
    missed: Option<f32>,
    start: Option<f32>,
    good: Option<f32>,
    good_feeling: Option<String>,
    poor: Option<f32>,
    poor_feeling: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct NewcomersRaw {
    hour: Option<f32>,
    days: Option<f32>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct MovingRaw {
    hour: Option<f32>,
    partners: Option<f32>,
    romance: Option<f32>,
    leave_home_age: Option<f32>,
    leave_home: Option<f32>,
    leave_town_age: Option<f32>,
    leave_town: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeathRaw {
    from: f32,
    per_year: f32,
    #[serde(default)]
    growth: f32,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct GriefRaw {
    feeling: Option<String>,
    light_feeling: Option<String>,
    close: Option<f32>,
    friend: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LifeStageRaw {
    id: String,
    #[serde(default)]
    label: Option<String>,
    from: f32,
    #[serde(default)]
    effects: ModifiersRaw,
    #[serde(default)]
    feeling: Option<String>,
    #[serde(default)]
    adult: Option<bool>,
    #[serde(default)]
    baby: bool,
    #[serde(default)]
    school: bool,
    #[serde(default)]
    scale: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccidentRaw {
    id: String,
    need: String,
    #[serde(default)]
    story: Option<String>,
    #[serde(default)]
    effects: HashMap<String, f32>,
    #[serde(default)]
    dirt: f32,
    #[serde(default)]
    feeling: Option<String>,
    #[serde(default)]
    cost: i64,
    #[serde(default)]
    cooldown_hours: Option<f32>,
    #[serde(default)]
    grace_minutes: Option<f32>,
    #[serde(default)]
    crowded_grace_minutes: Option<f32>,
    #[serde(default)]
    rest: Option<AccidentRestRaw>,
    #[serde(default)]
    another_room: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccidentRestRaw {
    label: String,
    minutes: f32,
    #[serde(default)]
    pose: Pose,
    #[serde(default)]
    anim: Option<String>,
    #[serde(default)]
    gains: HashMap<String, f32>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RoomKindRaw {
    id: String,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    essentials: Vec<Vec<String>>,
    #[serde(default)]
    size: Option<[f32; 2]>,
    #[serde(default)]
    exclusive: bool,
    #[serde(default)]
    passage: bool,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct WearRaw {
    per_use: Option<f32>,
    by_category: HashMap<String, f32>,
    per_quality: Option<f32>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct RepairRaw {
    label: Option<String>,
    minutes: Option<f32>,
    skill: Option<String>,
    skill_gain_per_hour: Option<f32>,
    chance: Option<f32>,
    chance_per_level: Option<f32>,
    interest: Option<f32>,
    cost: Option<f32>,
    tags: Option<Vec<String>>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct RoomRulesRaw {
    weights: Option<[f32; 5]>,
    window_tiles: Option<f32>,
    lamp_tiles: Option<f32>,
    decor_per_tile: Option<f32>,
    garden_decor_per_tile: Option<f32>,
    love: Option<f32>,
    dislike: Option<f32>,
    drift: Option<f32>,
    away: Option<f32>,
    preference: Option<f32>,
    love_feeling: Option<String>,
    dislike_feeling: Option<String>,
    decor_by_category: HashMap<String, f32>,
    dirt: HashMap<String, f32>,
    clean: CleanRaw,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct CleanRaw {
    label: Option<String>,
    minutes: Option<f32>,
    threshold: Option<f32>,
    amount: Option<f32>,
    interest: Option<f32>,
    tags: Option<Vec<String>>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct ObjectRulesRaw {
    /// Catalog categories whose 1×1 objects turn freely.
    free_rotation: Option<Vec<String>>,
    wear: WearRaw,
    repair: RepairRaw,
    max_quality: Option<u8>,
    quality_bonus: Option<f32>,
    upgrade_cost: Option<f32>,
    resale: Option<f32>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct BuildRaw {
    wall: Option<i64>,
    door: Option<i64>,
    window: Option<i64>,
    remove: Option<i64>,
    /// Fence per metre and gate, without fence styles.
    fence: Option<i64>,
    gate: Option<i64>,
    storeys: Option<u8>,
    #[serde(default)]
    basement: bool,
}

#[derive(Deserialize)]
struct BuildStyleRaw {
    id: String,
    #[serde(default)]
    price: i64,
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
struct CareerLevelRaw {
    title: String,
    pay: i64,
    start: f32,
    hours: f32,
    /// Weekday indices, 0 = Monday.
    days: Vec<u8>,
    #[serde(default)]
    requires: HashMap<String, f32>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct CareerRulesRaw {
    commute_minutes: Option<f32>,
    late_minutes: Option<f32>,
    missed_penalty: Option<f32>,
    performance_per_shift: Option<f32>,
    performance_per_fit: Option<f32>,
    performance_per_mood: Option<f32>,
    probation_levels: Option<f32>,
    workweek: Vec<WorkweekRaw>,
    #[serde(alias = "promotionMoodlet")] // key before the moodlet → feeling rename
    promotion_feeling: Option<String>,
    #[serde(alias = "missedMoodlet")] // key before the moodlet → feeling rename
    missed_feeling: Option<String>,
    market: Option<JobMarketRaw>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct JobMarketRaw {
    hour: Option<f32>,
    daily_chance: Option<f32>,
    quit_below: Option<f32>,
    quit_chance: Option<f32>,
    min_shifts_before_quit: Option<u32>,
    fire_after_missed: Option<u8>,
    cooldown_days: Option<u32>,
    hired_feeling: Option<String>,
    quit_feeling: Option<String>,
    fired_feeling: Option<String>,
}

#[derive(Deserialize)]
struct WorkweekRaw {
    fit: f32,
    days: i8,
    label: String,
}

#[derive(Deserialize)]
struct ScheduleRaw {
    from: f32,
    to: f32,
    #[serde(default)]
    effects: ModifiersRaw,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct VisitRaw {
    needs: HashMap<String, f32>,
    hours: Option<f32>,
    directed_hours: Option<f32>,
    min_friendship: Option<f32>,
    earliest_hour: Option<f32>,
    latest_hour: Option<f32>,
    door: Option<DoorRaw>,
    company: Option<f32>,
    bored_minutes: Option<f32>,
    farewell: Vec<String>,
    farewell_minutes: Option<f32>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct DoorRaw {
    wait_minutes: Option<f32>,
    welcome_friendship: Option<f32>,
    greet: Option<String>,
    turned_away_feeling: Option<String>,
    knock_anim: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct EconomyRaw {
    starting_funds: Option<i64>,
    rent: Option<RentRaw>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NeedRaw {
    id: String,
    label: String,
    #[serde(default)]
    decay_per_hour: f32,
    #[serde(default)]
    room: bool,
    #[serde(default)]
    comfortable: f32,
}

#[derive(Deserialize)]
struct ObjectRaw {
    id: String,
    name: String,
    #[serde(default = "one_by_one")]
    footprint: [u8; 2],
    #[serde(default)]
    interactions: Vec<InteractionRaw>,
    /// Things it provides while working (see `Interaction::requires`).
    #[serde(default)]
    provides: Vec<String>,
    #[serde(default)]
    price: Option<i64>,
    #[serde(default = "one_slot")]
    slots: u8,
    /// Each slot's place and facing (see `ObjectDef::places`).
    #[serde(default)]
    places: Vec<PlaceRaw>,
    #[serde(default)]
    outdoors: bool,
    #[serde(default)]
    stairs: bool,
    /// Buy catalog category (decides free rotation, see `objectRules.freeRotation`).
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    group: Option<String>,
    /// Overrides whether it turns freely.
    #[serde(default, rename = "freeRotation")]
    free_rotation: Option<bool>,
    /// Decor points (default: by category, `roomRules.decorByCategory`).
    #[serde(default)]
    decor: Option<f32>,
    /// Wear per finished use (default: `objectRules.wear`).
    #[serde(default, rename = "wearPerUse")]
    wear_per_use: Option<f32>,
    /// Lamps: the renderer reads the light's look; the simulation only its `tiles`, the floor it
    /// lights (default `roomRules.lampTiles`), `switch` (false: no switch, always lit at night)
    /// and `fire` (a fire, lit to sit by: see `ObjectDef::fire`).
    #[serde(default)]
    light: Option<serde_json::Value>,
    /// `wall`, `ceiling` or `rug` (default `floor`: furniture).
    #[serde(default)]
    layer: Option<String>,
    /// Bunk beds: the two slots lie one above the other.
    #[serde(default)]
    bunk: bool,
    #[serde(default)]
    surface: Option<SurfaceRaw>,
    #[serde(default)]
    radius: Option<f32>,
    /// `front` (default) or `around`.
    #[serde(default)]
    approach: Option<String>,
    #[serde(default, rename = "seatAt")]
    seat_at: Vec<String>,
    /// Cooking stations (`cooking.stations`).
    #[serde(default)]
    stations: Vec<String>,
    #[serde(default)]
    stores: bool,
    /// A crop that grows (see `ObjectDef::grows`).
    #[serde(default)]
    grows: Option<GrowsRaw>,
    #[serde(default, rename = "overWindow")]
    over_window: bool,
    #[serde(default, rename = "fillsWall")]
    fills_wall: bool,
    #[serde(default, rename = "hangsLow")]
    hangs_low: bool,
    /// Life-stage ids that may use it (see `Interaction::stages`).
    #[serde(default)]
    stages: Option<Vec<String>>,
    /// Life-stage ids that may take the upper bunk (see `ObjectDef::upper_stages`).
    #[serde(default, rename = "upperStages")]
    upper_stages: Option<Vec<String>>,
    #[serde(default, rename = "headHeight")]
    head_height: bool,
    #[serde(default, rename = "againstWall")]
    against_wall: bool,
    #[serde(default)]
    reach: f32,
    #[serde(default)]
    footstool: bool,
    #[serde(default)]
    overhang: Vec<[i32; 2]>,
}

/// A slot's place: `{"at": [-0.3, 0.7], "facing": 180}` (m off the middle, to its right and front;
/// degrees from its front towards its right, default 0).
#[derive(Deserialize)]
struct PlaceRaw {
    at: [f32; 2],
    #[serde(default)]
    facing: f32,
}

/// An object's `grows`: `{"days": 2, "fallow": 0.5}` (game days to ripen, and fallow after picking).
#[derive(Deserialize)]
struct GrowsRaw {
    days: f32,
    #[serde(default = "half_a_day")]
    fallow: f32,
}

fn half_a_day() -> f32 {
    0.5
}

#[derive(Deserialize)]
struct SurfaceRaw {
    height: f32,
    kind: String,
    #[serde(default = "surface_inset")]
    inset: f32,
    #[serde(default)]
    size: Option<[f32; 2]>,
    #[serde(default)]
    at: [f32; 2],
}

fn surface_inset() -> f32 {
    0.04
}

fn one_slot() -> u8 {
    1
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InteractionRaw {
    id: String,
    label: String,
    /// Not needed with a `recipe` (its steps take their own time).
    #[serde(default)]
    minutes: f32,
    #[serde(default)]
    recipe: Option<String>,
    /// Mess it leaves (default: by its tags, `roomRules.dirt`).
    #[serde(default)]
    dirt: Option<f32>,
    #[serde(default)]
    baby: bool,
    #[serde(default)]
    needs_table: bool,
    #[serde(default)]
    whole: bool,
    #[serde(default)]
    homework: f32,
    /// Life-stage ids that may take it (see `Interaction::stages`).
    #[serde(default)]
    stages: Option<Vec<String>>,
    #[serde(default)]
    care: HashMap<String, f32>,
    #[serde(default)]
    effects: HashMap<String, f32>,
    #[serde(default)]
    pose: Pose,
    /// Sit on a free seat within this many metres of the object (see `Interaction::seat_near`).
    #[serde(default)]
    seat_near: f32,
    /// That seat faces the object: `front` or `around` (see `Interaction::seat_facing`).
    #[serde(default)]
    seat_facing: Option<String>,
    /// Share of the gains nearby residents get too (see `Interaction::broadcast`).
    #[serde(default)]
    broadcast: f32,
    /// How far that carries when not in the same room (see `Interaction::broadcast_range`).
    #[serde(default)]
    broadcast_range: Option<f32>,
    /// Used standing on the object (see `Interaction::inside`).
    #[serde(default)]
    inside: bool,
    #[serde(default = "yes")]
    autonomous: bool,
    #[serde(default)]
    tags: Vec<String>,
    /// Things a working object on the same plot must provide (see `ObjectDef::provides`).
    #[serde(default)]
    requires: Vec<String>,
    /// Skill levels gained per hour.
    #[serde(default)]
    skills: HashMap<String, f32>,
    #[serde(default)]
    cost: i64,
    #[serde(default)]
    skill: Option<String>,
    #[serde(default)]
    #[serde(alias = "moodlet")] // key before the moodlet → feeling rename
    feeling: Option<String>,
    #[serde(default)]
    #[serde(alias = "moodletMinSkill")] // key before the moodlet → feeling rename
    feeling_min_skill: f32,
    /// Animation tag (one of the content's `animations`).
    #[serde(default)]
    anim: Option<String>,
    /// A tag: what's done straight afterwards (see `Interaction::after`).
    #[serde(default)]
    after: Option<String>,
    /// Food to eat sitting down once fetched (see `Interaction::eats`).
    #[serde(default)]
    eat: Option<EatRaw>,
    /// Only offered while the object is so (see `Interaction::when`).
    #[serde(default)]
    when: Option<OpenState>,
    /// What the use leaves the object (see `Interaction::leaves`).
    #[serde(default)]
    leaves: Option<OpenState>,
    #[serde(default)]
    close_after: bool,
    /// What it leaves comes about through the use (see `Interaction::gradual`).
    #[serde(default)]
    gradual: bool,
    /// Fetched first from what provides what it requires (see `Interaction::fetch`).
    #[serde(default)]
    fetch: Option<String>,
    /// Changing clothes (see `Interaction::changes_outfit`).
    #[serde(default)]
    changes_outfit: bool,
    /// A spare bed's night (see `Interaction::spare`).
    #[serde(default)]
    spare: bool,
    /// Brings the crop on (see `Interaction::tends`).
    #[serde(default)]
    tends: f32,
    /// A harvest's produce, put in the fridge (see `Interaction::yields`).
    #[serde(default)]
    yields: Option<YieldRaw>,
    /// Done with the baby carried here (see `Interaction::carried`).
    #[serde(default)]
    carried: bool,
    /// Done in front of their own place on it (see `Interaction::at_slot`).
    #[serde(default)]
    at_slot: bool,
    /// Hours of the day it may start in (see `Interaction::hours`).
    #[serde(default)]
    hours: Option<[f32; 2]>,
    /// Only under open sky (see `Interaction::sky`).
    #[serde(default)]
    sky: bool,
    /// Something it works on (see `Interaction::work_on`).
    #[serde(default)]
    work_on: Option<WorkOn>,
    /// A game for two (see `Interaction::partner`).
    #[serde(default)]
    partner: bool,
    /// The use done instead with nobody to play (see `Interaction::alone`).
    #[serde(default)]
    alone: Option<String>,
}

/// Content `laundry` (see `LaundryRules`).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LaundryRaw {
    per_day: f32,
    most: f32,
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
enum OpenState {
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
    need_decay: HashMap<String, f32>,
    need_gain: HashMap<String, f32>,
    tag_preference: HashMap<String, f32>,
    tag_acceptance: HashMap<String, f32>,
    tag_success: HashMap<String, f32>,
    walk_speed: Option<f32>,
    mood: f32,
    skill_gain: HashMap<String, f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TraitRaw {
    id: String,
    label: String,
    #[serde(default)]
    conflicts: Vec<String>,
    #[serde(default)]
    effects: ModifiersRaw,
    #[serde(default)]
    starting_skills: HashMap<String, f32>,
}

#[derive(Deserialize)]
struct PerkRaw {
    id: String,
    label: String,
    cost: u32,
    #[serde(default)]
    effects: ModifiersRaw,
}

#[derive(Deserialize)]
struct GenderRaw {
    id: String,
    label: String,
}

fn one_by_one() -> [u8; 2] {
    [1, 1]
}

fn yes() -> bool {
    true
}

fn intern(tags: &mut Vec<String>, tag: &str) -> Result<(), Error> {
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
        if v >= 0.0 && v.is_finite() { Ok(v) } else { Err(Error::new(format!("{ctx}: {what} {v} must be a number ≥ 0"))) }
    };
    let number = |v: f32, what: &str| {
        if v.is_finite() { Ok(v) } else { Err(Error::new(format!("{ctx}: {what} must be a number"))) }
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
        Some(v) => return Err(Error::new(format!("{ctx}: walkSpeed {v} must be above 0 and at most {MAX_WALK_SPEED}"))),
    };
    m.mood = number(raw.mood, "mood")?;
    Ok(m)
}

/// Animation tag for an interaction without an `anim`, from its tags, pose and needs.
/// The caller drops tags that content doesn't declare in `animations`.
fn default_anim(it: &InteractionRaw) -> &'static str {
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

impl Content {
    pub fn from_json(json: &str) -> Result<Self, Error> {
        let raw: ContentFile = serde_json::from_str(json)?;
        if raw.needs.is_empty() || raw.needs.len() > MAX_NEEDS {
            return Err(Error::new(format!(
                "content must define 1..={MAX_NEEDS} needs"
            )));
        }
        // (Needs only fall over time: a negative decay would push one past full.)
        if let Some(n) = raw.needs.iter().find(|n| !(n.decay_per_hour >= 0.0 && n.decay_per_hour.is_finite())) {
            return Err(Error::new(format!("need '{}': decayPerHour {} must be a number ≥ 0", n.id, n.decay_per_hour)));
        }

        let needs: Vec<NeedDef> = raw
            .needs
            .iter()
            .map(|n| NeedDef {
                id: n.id.clone(),
                label: n.label.clone(),
                decay_per_minute: n.decay_per_hour / 60.0,
                room: n.room,
                comfortable: n.comfortable.clamp(0.0, 1.0),
            })
            .collect();
        let need_index: HashMap<&str, usize> = needs
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id.as_str(), i))
            .collect();
        if need_index.len() != needs.len() {
            return Err(Error::new("duplicate need id"));
        }

        // Tags come from object interactions and socials; everything else refers to them.
        let mut tags: Vec<String> = Vec::new();
        for obj in &raw.objects {
            for it in &obj.interactions {
                for t in &it.tags {
                    intern(&mut tags, t)?;
                }
            }
        }
        for s in &raw.socials {
            for t in &s.tags {
                intern(&mut tags, t)?;
            }
        }
        for t in &raw.tags {
            intern(&mut tags, t)?;
        }
        for a in &raw.phone.actions {
            for t in &a.tags {
                intern(&mut tags, t)?;
            }
        }
        // Built-in tag for visiting other households (schedules and traits can refer to it).
        intern(&mut tags, VISIT_TAG)?;
        // Tidying up is an action without an object; its tags exist all the same.
        let clean_tags = raw.room_rules.clean.tags.clone().unwrap_or_else(|| vec!["chores".into(), "cleaning".into()]);
        for t in &clean_tags {
            intern(&mut tags, t)?;
        }
        // Capabilities (what an object provides, what an interaction requires) share the tag
        // namespace, so content can refer to them by name without another list.
        for obj in &raw.objects {
            for t in &obj.provides {
                intern(&mut tags, t)?;
            }
            for it in &obj.interactions {
                for t in &it.requires {
                    intern(&mut tags, t)?;
                }
            }
        }
        let tag_index: HashMap<&str, usize> = tags
            .iter()
            .enumerate()
            .map(|(i, t)| (t.as_str(), i))
            .collect();

        // Animation tags: declared once (packs may append); repeats are merged.
        let mut animations: Vec<String> = Vec::new();
        for a in &raw.animations {
            if a.is_empty() {
                return Err(Error::new("animations: empty animation tag"));
            }
            if !animations.contains(a) {
                animations.push(a.clone());
            }
        }
        let anim_index = |a: &str| animations.iter().position(|x| x == a);

        let emotion_index: HashMap<String, usize> = raw
            .emotions
            .iter()
            .enumerate()
            .map(|(i, e)| (e.id.clone(), i))
            .collect();
        let feeling_index: HashMap<String, usize> = raw
            .feelings
            .iter()
            .enumerate()
            .map(|(i, m)| (social::feeling_id(m).to_owned(), i))
            .collect();
        if emotion_index.len() != raw.emotions.len() {
            return Err(Error::new("duplicate emotion id"));
        }
        if feeling_index.len() != raw.feelings.len() {
            return Err(Error::new("duplicate feeling id"));
        }
        if raw.skills.len() > MAX_SKILLS {
            return Err(Error::new(format!("at most {MAX_SKILLS} skills")));
        }
        let skills: Vec<SkillDef> = raw
            .skills
            .iter()
            .map(|s| SkillDef {
                id: s.id.clone(),
                label: s.label.clone(),
            })
            .collect();
        let skill_index: HashMap<&str, usize> = skills
            .iter()
            .enumerate()
            .map(|(i, s)| (s.id.as_str(), i))
            .collect();
        if skill_index.len() != skills.len() {
            return Err(Error::new("duplicate skill id"));
        }
        let skill_array =
            |map: &HashMap<String, f32>, ctx: &str| -> Result<[f32; MAX_SKILLS], Error> {
                let mut out = [0.0; MAX_SKILLS];
                for (k, v) in map {
                    out[lookup(&skill_index, k, ctx, "skill")?] = *v;
                }
                Ok(out)
            };
        // Skill weights, strongest first (ties by skill order) so output is deterministic.
        let skill_list =
            |map: &HashMap<String, f32>, ctx: &str| -> Result<Vec<(usize, f32)>, Error> {
                let mut out = map
                    .iter()
                    .map(|(k, v)| Ok((lookup(&skill_index, k, ctx, "skill")?, *v)))
                    .collect::<Result<Vec<_>, Error>>()?;
                out.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
                Ok(out)
            };

        let ix = Indices {
            needs: &need_index,
            tags: &tag_index,
            emotions: &emotion_index,
            feelings: &feeling_index,
            skills: &skill_index,
        };

        let (cooking, items, mut recipes) = crate::cooking::parse(
            &raw.cooking,
            &raw.items,
            &raw.recipes,
            &crate::cooking::Lookups { needs: &need_index, tags: &tag_index, feelings: &feeling_index, animations: &animations },
        )?;

        let lamp_tiles = raw.room_rules.lamp_tiles.unwrap_or(10.0).max(0.1);
        // Who may use what (`stages` rules): life stages by id, a bit each. Without stages in
        // content, everyone may use everything.
        let life_stages = &raw.life.stages;
        if life_stages.len() > MAX_STAGES {
            return Err(Error::new(format!("life: at most {MAX_STAGES} stages")));
        }
        let every_stage: StageMask = if life_stages.is_empty() { StageMask::MAX } else { StageMask::MAX >> (MAX_STAGES - life_stages.len()) };
        let babies: StageMask = life_stages.iter().enumerate().filter(|(_, s)| s.baby).fold(0, |m, (i, _)| m | 1 << i);
        let stage_ids: Vec<&str> = life_stages.iter().map(|s| s.id.as_str()).collect();
        let stages_of = |ids: &Option<Vec<String>>, ctx: &str| stage_mask(&stage_ids, ids, ctx);
        let recipe_stages = raw.recipes.iter().map(|r| stages_of(&r.stages, &format!("recipe '{}'", r.id))).collect::<Result<Vec<_>, _>>()?;
        let mut objects = Vec::with_capacity(raw.objects.len());
        let mut object_index = HashMap::new();
        for obj in &raw.objects {
            if obj.footprint[0] == 0 || obj.footprint[1] == 0 {
                return Err(Error::new(format!(
                    "object '{}' has an empty footprint",
                    obj.id
                )));
            }
            // (Nothing bigger can be bought, see `World::check_size`.)
            if obj.footprint.iter().any(|&f| f > crate::world::MAX_SIZE) {
                return Err(Error::new(format!("object '{}': a footprint is at most {MAX} × {MAX} tiles", obj.id, MAX = crate::world::MAX_SIZE)));
            }
            if let Some((i, it)) = obj.interactions.iter().enumerate().find(|(i, it)| obj.interactions[..*i].iter().any(|o| o.id == it.id)) {
                return Err(Error::new(format!("object '{}': interaction {i} repeats the id '{}'", obj.id, it.id)));
            }
            let object_stages = stages_of(&obj.stages, &format!("object '{}'", obj.id))?;
            let mut interactions = Vec::with_capacity(obj.interactions.len());
            for it in &obj.interactions {
                let ctx = format!("{}.{}", obj.id, it.id);
                let recipe = it
                    .recipe
                    .as_deref()
                    .map(|r| recipes.iter().position(|d| d.id == r).ok_or_else(|| Error::new(format!("{ctx}: unknown recipe '{r}'"))))
                    .transpose()?;
                // Its own stages, else its recipe's, else its object's; a baby's place is a baby's.
                let stages = stages_of(&it.stages, &ctx)?.or(recipe.and_then(|r| recipe_stages.get(r).copied().flatten())).or(object_stages);
                if it.baby && stages.is_some_and(|s| s & !babies != 0) {
                    return Err(Error::new(format!("{ctx}: a `baby` use is a baby's only: no other `stages`")));
                }
                let stages = stages.unwrap_or(if !it.baby {
                    every_stage & !babies
                } else if babies != 0 {
                    babies
                } else {
                    every_stage
                });
                // A recipe takes as long as its steps.
                let minutes = recipe.map_or(it.minutes, |r| recipes[r].minutes);
                if !(minutes > 0.0 && minutes.is_finite()) {
                    return Err(Error::new(format!("{ctx}: minutes must be > 0")));
                }
                let mut own_gain = [0.0; MAX_NEEDS];
                for (need, gain) in &it.effects {
                    own_gain[lookup(&need_index, need, &ctx, "need")?] = *gain;
                }
                // With a recipe, residents weigh it by what a serving gives, plus what cooking
                // itself does for the cook.
                let mut total_gain = own_gain;
                if let Some(r) = recipe {
                    for (t, e) in total_gain.iter_mut().zip(recipes[r].effects) {
                        *t += e;
                    }
                }
                // Food eaten sitting down: what it gives comes from eating it (a hidden recipe
                // of its own), and fetching it gives nothing but its costs. Residents weigh it
                // all the same.
                let eats = match &it.eat {
                    Some(_) if recipe.is_some() => {
                        return Err(Error::new(format!("{ctx}: `eat` and `recipe` don't go together")));
                    }
                    Some(_) if it.yields.is_some() => {
                        return Err(Error::new(format!("{ctx}: `eat` and `yields` don't go together")));
                    }
                    Some(e) => {
                        let mut eaten = [0.0; MAX_NEEDS];
                        for (own, eat) in own_gain.iter_mut().zip(eaten.iter_mut()).filter(|(g, _)| **g > 0.0) {
                            *eat = std::mem::take(own);
                        }
                        recipes.push(crate::cooking::eat_recipe(&ctx, &it.label, &items, &e.item, e.minutes, eaten)?);
                        Some(recipes.len() - 1)
                    }
                    None => None,
                };
                // A harvest's produce: what it gives comes from eating a serving of it later (as
                // `eat`), and residents weigh it all the same.
                let yields = match &it.yields {
                    Some(_) if recipe.is_some() => {
                        return Err(Error::new(format!("{ctx}: `yields` and `recipe` don't go together")));
                    }
                    Some(y) if y.servings == 0 => return Err(Error::new(format!("{ctx}: yields: servings must be at least 1"))),
                    Some(y) => {
                        let mut eaten = [0.0; MAX_NEEDS];
                        for (own, eat) in own_gain.iter_mut().zip(eaten.iter_mut()).filter(|(g, _)| **g > 0.0) {
                            *eat = std::mem::take(own);
                        }
                        recipes.push(crate::cooking::eat_recipe(&ctx, &it.label, &items, &y.item, y.minutes, eaten)?);
                        Some(Yield { recipe: recipes.len() - 1, servings: y.servings })
                    }
                    None => None,
                };
                if !(0.0..=1.0).contains(&it.tends) {
                    return Err(Error::new(format!("{ctx}: tends must be 0..=1")));
                }
                if it.tends > 0.0 && obj.grows.is_none() {
                    return Err(Error::new(format!("{ctx}: `tends` brings a crop on: the object needs `grows`")));
                }
                if it.carried && (it.care.values().all(|&g| g <= 0.0) || !obj.interactions.iter().any(|i| i.baby)) {
                    return Err(Error::new(format!("{ctx}: `carried` is for a baby's care (`care`) where a baby has a place (`baby`)")));
                }
                let after = match it.after.as_deref() {
                    Some(t) => 1 << lookup(&tag_index, t, &ctx, "tag")?,
                    None => 0,
                };
                let mut care = [0.0; MAX_NEEDS];
                for (need, gain) in &it.care {
                    care[lookup(&need_index, need, &ctx, "need")?] = gain.max(0.0);
                }
                let mut mask: TagMask = 0;
                for t in &it.tags {
                    mask |= 1 << tag_index[t.as_str()];
                }
                let mut requires = 0;
                for t in &it.requires {
                    requires |= 1 << tag_index[t.as_str()];
                }
                if it.cost < 0 {
                    return Err(Error::new(format!("{ctx}: negative cost")));
                }
                if it.seat_near < 0.0 {
                    return Err(Error::new(format!("{ctx}: negative seatNear")));
                }
                let seat_facing = match it.seat_facing.as_deref() {
                    None => None,
                    Some("front") => Some(SeatFacing::Front),
                    Some("around") => Some(SeatFacing::Around),
                    Some(other) => return Err(Error::new(format!("{ctx}: unknown seatFacing '{other}' (front or around)"))),
                };
                if seat_facing.is_some() && it.seat_near <= 0.0 {
                    return Err(Error::new(format!("{ctx}: seatFacing needs a seatNear")));
                }
                if !(0.0..=1.0).contains(&it.broadcast) {
                    return Err(Error::new(format!("{ctx}: broadcast must be 0..=1")));
                }
                if it.broadcast_range.is_some_and(|r| r < 0.0) {
                    return Err(Error::new(format!("{ctx}: negative broadcastRange")));
                }
                let skill = it
                    .skill
                    .as_deref()
                    .map(|s| lookup(&skill_index, s, &ctx, "skill"))
                    .transpose()?;
                let feeling = it
                    .feeling
                    .as_deref()
                    .map(|m| {
                        feeling_index
                            .get(m)
                            .copied()
                            .ok_or_else(|| Error::new(format!("{ctx}: unknown feeling '{m}'")))
                    })
                    .transpose()?;
                if it.feeling_min_skill > 0.0 && (skill.is_none() || feeling.is_none()) {
                    return Err(Error::new(format!(
                        "{ctx}: feelingMinSkill needs a skill and a feeling"
                    )));
                }
                let anim = match it.anim.as_deref() {
                    Some(a) => Some(anim_index(a).ok_or_else(|| {
                        Error::new(format!(
                            "{ctx}: unknown anim '{a}' (declare it in \"animations\")"
                        ))
                    })?),
                    None => anim_index(default_anim(it)),
                };
                interactions.push(Interaction {
                    id: it.id.clone(),
                    label: it.label.clone(),
                    minutes,
                    total_gain,
                    gain_per_minute: own_gain.map(|g| g / minutes),
                    pose: it.pose,
                    seat_near: it.seat_near,
                    seat_facing,
                    broadcast: it.broadcast,
                    broadcast_range: it.broadcast_range.unwrap_or(6.0),
                    inside: it.inside,
                    autonomous: it.autonomous,
                    tags: mask,
                    requires,
                    skill_gain: skill_array(&it.skills, &ctx)?,
                    // A recipe costs its ingredients (for its fewest servings; it charges for as
                    // many as it makes).
                    cost: recipe.map_or(it.cost, |r| recipes[r].cost(recipes[r].servings[0])),
                    skill,
                    feeling,
                    feeling_min_skill: it.feeling_min_skill,
                    anim,
                    dirt: it.dirt.unwrap_or(-1.0),
                    baby: it.baby,
                    needs_table: it.needs_table,
                    whole: it.whole,
                    homework: it.homework.max(0.0),
                    stages,
                    care,
                    recipe,
                    after,
                    eats,
                    when: match it.when {
                        Some(OpenState::Open) => Some(true),
                        Some(OpenState::Closed) => Some(false),
                        _ => None,
                    },
                    leaves: match it.leaves {
                        Some(OpenState::Open) => Some(true),
                        Some(OpenState::Closed) => Some(false),
                        _ => None,
                    },
                    close_after: it.close_after,
                    ripe: it.when == Some(OpenState::Ripe),
                    picks: it.leaves == Some(OpenState::Picked),
                    gradual: it.gradual,
                    fetch: it.fetch.clone(),
                    changes_outfit: it.changes_outfit,
                    spare: it.spare,
                    tends: it.tends,
                    yields,
                    carried: it.carried,
                    at_slot: it.at_slot,
                    hours: it.hours,
                    sky: it.sky,
                    work_on: it.work_on,
                    partner: it.partner,
                    alone: None,
                });
                // A game for two: at an object with two places, one each, used there as any use.
                if it.partner
                    && (obj.slots != 2
                        || obj.interactions.iter().any(|i| i.baby)
                        || it.whole
                        || it.inside
                        || it.seat_near > 0.0
                        || it.recipe.is_some()
                        || it.eat.is_some()
                        || it.yields.is_some()
                        || it.carried
                        || it.needs_table)
                {
                    return Err(Error::new(format!(
                        "{ctx}: a game for two (`partner`) is played at an object with two places (`slots`: 2), one each (no whole, inside, seatNear, recipe, eat, yields, carried or needsTable)"
                    )));
                }
                if it.alone.is_some() && !it.partner {
                    return Err(Error::new(format!("{ctx}: `alone` is what's done with nobody to play: it needs `partner`")));
                }
                if it.fetch.is_some() && it.requires.is_empty() {
                    return Err(Error::new(format!("{ctx}: `fetch` fetches what it `requires`: it requires nothing")));
                }
                if it.gradual && !matches!(it.leaves, Some(OpenState::Open | OpenState::Closed)) {
                    return Err(Error::new(format!("{ctx}: `gradual` is for a use that leaves it open or closed")));
                }
                if let Some(h) = it.hours
                    && (!h.iter().all(|v| (0.0..=24.0).contains(v)) || h[0] == h[1])
                {
                    return Err(Error::new(format!("{ctx}: `hours` is [from, to], two different hours of the day (0 to 24)")));
                }
                if matches!(it.work_on, Some(WorkOn::Laundry | WorkOn::Washed)) && raw.laundry.is_none() {
                    return Err(Error::new(format!("{ctx}: `workOn` laundry needs content `laundry` (how the washing piles up)")));
                }
                if it.close_after
                    && (it.leaves != Some(OpenState::Open)
                        || !obj.interactions.iter().any(|i| i.leaves == Some(OpenState::Closed)))
                {
                    return Err(Error::new(format!(
                        "{ctx}: `closeAfter` is for a use that leaves it open, on an object with a use that leaves it closed"
                    )));
                }
                if it.when == Some(OpenState::Picked) || it.leaves == Some(OpenState::Ripe) {
                    return Err(Error::new(format!("{ctx}: `when` takes open, closed or ripe; `leaves` open, closed or picked")));
                }
                if (it.when == Some(OpenState::Ripe) || it.leaves == Some(OpenState::Picked)) && obj.grows.is_none() {
                    return Err(Error::new(format!("{ctx}: \"ripe\" and \"picked\" are a crop's: give the object `grows`")));
                }
            }
            // A game's use for one (`alone`): another of this object's, not a game, done where
            // the game is (the same pose, so the place they're going to stays theirs).
            for (k, it) in obj.interactions.iter().enumerate() {
                let Some(alone) = &it.alone else { continue };
                let ctx = format!("{}.{}", obj.id, it.id);
                let Some(a) = interactions.iter().position(|i| &i.id == alone) else {
                    return Err(Error::new(format!("{ctx}: `alone` names no use of '{}': '{alone}'", obj.id)));
                };
                if interactions[a].partner || interactions[a].pose != interactions[k].pose {
                    return Err(Error::new(format!("{ctx}: `alone` is a use for one (no `partner`), done in the same pose as the game")));
                }
                interactions[k].alone = Some(a);
            }
            if object_index.insert(obj.id.clone(), objects.len()).is_some() {
                return Err(Error::new(format!("duplicate object id '{}'", obj.id)));
            }
            if obj.slots == 0 || obj.slots as usize > MAX_SLOTS {
                return Err(Error::new(format!(
                    "object '{}': slots must be 1..={MAX_SLOTS}",
                    obj.id
                )));
            }
            if obj.price.is_some_and(|p| p < 0) {
                return Err(Error::new(format!("object '{}': negative price", obj.id)));
            }
            // (A spare bed's night doesn't make its room a bedroom.)
            let offered = interactions.iter().filter(|it| !it.spare).fold(0, |m: TagMask, it| m | it.tags);
            let mut provides = 0;
            for t in &obj.provides {
                provides |= 1 << tag_index[t.as_str()];
            }
            let used = !interactions.is_empty();
            let layer = match obj.layer.as_deref() {
                None | Some("floor") => Layer::Floor,
                Some("wall") => Layer::Wall,
                Some("ceiling") => Layer::Ceiling,
                Some("rug") => Layer::Rug,
                Some("surface") => Layer::Surface,
                Some(other) => {
                    return Err(Error::new(format!(
                        "object '{}': unknown layer '{other}' (floor, wall, ceiling, rug or surface)",
                        obj.id
                    )));
                }
            };
            if obj.hangs_low && obj.head_height {
                return Err(Error::new(format!("object '{}': `hangsLow` or `headHeight`, not both", obj.id)));
            }
            for (set, flag, on) in [
                (obj.over_window, "overWindow", Layer::Wall),
                (obj.fills_wall, "fillsWall", Layer::Floor),
                (obj.hangs_low, "hangsLow", Layer::Ceiling),
                (obj.head_height, "headHeight", Layer::Ceiling),
                (obj.against_wall, "againstWall", Layer::Floor),
            ] {
                if set && layer != on {
                    let on = match on {
                        Layer::Wall => "a wall",
                        Layer::Floor => "the floor",
                        _ => "the ceiling",
                    };
                    return Err(Error::new(format!("object '{}': `{flag}` is for things on {on}", obj.id)));
                }
            }
            if !obj.overhang.is_empty() && (layer != Layer::Floor || obj.footprint != [1, 1] || obj.overhang.iter().any(|&[r, f]| (r, f) == (0, 0) || r.abs() > 2 || f.abs() > 2)) {
                return Err(Error::new(format!("object '{}': `overhang` is for a 1×1 thing on the floor, over tiles up to 2 from its own", obj.id)));
            }
            if layer == Layer::Surface && (obj.footprint != [1, 1] || obj.slots != 1 || obj.stairs) {
                return Err(Error::new(format!("object '{}': things on a surface are 1×1, with one slot", obj.id)));
            }
            // A thing on a top is used from beside its top (`surface::rider_tiles`), standing, at
            // a seat pulled up to the top or on one near (`seatNear`): not lain on, stood in,
            // cooked at as a station or dined at.
            if layer == Layer::Surface
                && let Some(it) = interactions.iter().find(|i| i.pose == Pose::Lie || i.inside || i.whole || i.baby || i.cares() || i.needs_table || i.recipe.is_some())
            {
                return Err(Error::new(format!(
                    "{}.{}: a thing on a surface is used standing or sitting at its top (no lie, inside, whole, baby, care, needsTable or recipe)",
                    obj.id, it.id
                )));
            }
            let surface = match &obj.surface {
                None => None,
                Some(_) if layer != Layer::Floor => {
                    return Err(Error::new(format!(
                        "object '{}': only furniture on the floor has a surface",
                        obj.id
                    )));
                }
                Some(s) => Some(Surface {
                    height: s.height.max(0.0),
                    kind: SurfaceKind::parse(&s.kind).ok_or_else(|| {
                        Error::new(format!(
                            "object '{}': unknown surface kind '{}' (table, bar, counter, desk, low or shelf)",
                            obj.id, s.kind
                        ))
                    })?,
                    inset: s.inset.max(0.0),
                    size: s
                        .size
                        .unwrap_or(obj.footprint.map(|f| f as f32))
                        .map(|v| v.max(0.0)),
                    at: s.at,
                }),
            };
            let approach = match obj.approach.as_deref() {
                None | Some("front") => Approach::Front,
                Some("around") => Approach::Around,
                Some("side") => Approach::Side,
                Some("behind") => Approach::Behind,
                Some(other) => {
                    return Err(Error::new(format!(
                        "object '{}': unknown approach '{other}' (front, around, side or behind)",
                        obj.id
                    )));
                }
            };
            let mut seat_at = 0;
            for k in &obj.seat_at {
                seat_at |= SurfaceKind::parse(k)
                    .ok_or_else(|| {
                        Error::new(format!("object '{}': unknown seatAt kind '{k}'", obj.id))
                    })?
                    .bit();
            }
            if interactions.iter().any(|i| i.needs_table) && seat_at == 0 {
                return Err(Error::new(format!(
                    "object '{}': needsTable interactions need a seatAt",
                    obj.id
                )));
            }
            let mut stations = 0;
            for name in &obj.stations {
                stations |= cooking.station(name).ok_or_else(|| {
                    Error::new(format!("object '{}': unknown station '{name}' (see cooking.stations)", obj.id))
                })?;
            }
            if obj.bunk && (obj.slots != 2 || layer.mounted()) {
                return Err(Error::new(format!(
                    "object '{}': a bunk bed has 2 slots and stands on the floor",
                    obj.id
                )));
            }
            let upper_stages = stages_of(&obj.upper_stages, &format!("object '{}' upperStages", obj.id))?;
            if upper_stages.is_some() && !obj.bunk {
                return Err(Error::new(format!("object '{}': `upperStages` is a bunk bed's", obj.id)));
            }
            if obj.stairs && layer.mounted() {
                return Err(Error::new(format!("object '{}': stairs stand on the floor", obj.id)));
            }
            if !obj.places.is_empty() {
                let half = obj.footprint.map(|f| f as f32 / 2.0 + PLACE_REACH);
                if obj.places.len() != obj.slots as usize || obj.bunk || !matches!(layer, Layer::Floor | Layer::Wall) {
                    return Err(Error::new(format!("object '{}': `places` gives one place a slot, on the floor or a wall (not for a bunk bed)", obj.id)));
                }
                if obj.places.iter().any(|p| !p.facing.is_finite() || (0..2).any(|k| !p.at[k].is_finite() || p.at[k].abs() > half[k])) {
                    return Err(Error::new(format!("object '{}': a place is more than {PLACE_REACH} m off its footprint", obj.id)));
                }
            }
            // Things on a wall hang square to it; things on a surface always turn.
            let turns = obj.free_rotation.unwrap_or_else(|| {
                layer == Layer::Surface
                    || layer != Layer::Wall
                        && obj.footprint == [1, 1]
                        && obj.category.as_ref().is_some_and(|c| raw.object_rules.free_rotation.iter().flatten().any(|f| f == c))
            });
            // (Its shade's tiles turn with it in quarter turns: turned between, it would hang over one not kept for it.)
            if turns && !obj.overhang.is_empty() {
                return Err(Error::new(format!("object '{}': `overhang` turns in quarter turns only (`freeRotation`: false)", obj.id)));
            }
            objects.push(ObjectDef {
                id: obj.id.clone(),
                name: obj.name.clone(),
                footprint: obj.footprint,
                interactions,
                price: obj.price,
                slots: obj.slots,
                places: obj.places.iter().map(|p| SlotPlace { at: p.at, facing: p.facing.to_radians() }).collect(),
                outdoors: obj.outdoors,
                stairs: obj.stairs,
                turns,
                decor: obj
                    .decor
                    .or_else(|| {
                        let by = &raw.room_rules.decor_by_category;
                        obj.category.as_ref().and_then(|c| by.get(c).copied())
                    })
                    .unwrap_or(0.0)
                    .max(0.0),
                lamp: obj.light.as_ref().map_or(0.0, |l| {
                    l.get("tiles").and_then(serde_json::Value::as_f64).map_or(lamp_tiles, |t| t as f32).max(0.1)
                }),
                switch: obj.light.as_ref().is_some_and(|l| l.get("switch").and_then(serde_json::Value::as_bool) != Some(false)),
                fire: obj.light.as_ref().is_some_and(|l| l.get("fire").and_then(serde_json::Value::as_bool) == Some(true) && l.get("switch").and_then(serde_json::Value::as_bool) != Some(false)),
                // Things nobody uses up (scenery, plants, decor) never wear out.
                wear_per_use: if !used {
                    0.0
                } else {
                    let w = &raw.object_rules.wear;
                    obj.wear_per_use
                        .or_else(|| obj.category.as_ref().and_then(|c| w.by_category.get(c).copied()))
                        .unwrap_or(w.per_use.unwrap_or(0.004))
                        .max(0.0)
                },
                tags: offered,
                provides,
                layer,
                bunk: obj.bunk,
                upper_stages: upper_stages.unwrap_or(every_stage),
                surface,
                radius: obj.radius.unwrap_or(DEFAULT_RADIUS).clamp(0.02, 0.45),
                approach,
                seat_at,
                stations,
                stores: obj.stores,
                group: obj.group.clone().unwrap_or_default(),
                grows: match &obj.grows {
                    Some(g) if g.days > 0.0 && g.fallow >= 0.0 => Some(Growth { days: g.days, fallow_days: g.fallow }),
                    Some(_) => {
                        return Err(Error::new(format!("object '{}': `grows` needs days > 0 and fallow >= 0", obj.id)));
                    }
                    None => None,
                },
                over_window: obj.over_window,
                fills_wall: obj.fills_wall,
                hangs_low: obj.hangs_low,
                head_height: obj.head_height,
                against_wall: obj.against_wall,
                reach: obj.reach.max(0.0),
                footstool: obj.footstool,
                overhang: obj.overhang.iter().map(|&[r, f]| (r, f)).collect(),
            });
        }

        let emotions = raw
            .emotions
            .iter()
            .map(|e| {
                Ok(EmotionDef {
                    id: e.id.clone(),
                    label: e.label.clone(),
                    mods: build_modifiers(&e.effects, &ix, &e.id)?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let feelings = raw
            .feelings
            .iter()
            .map(|m| social::parse_feeling(m, &ix))
            .collect::<Result<Vec<_>, _>>()?;
        let socials = raw
            .socials
            .iter()
            .map(|s| social::parse_social(s, &ix, &stage_ids))
            .collect::<Result<Vec<_>, _>>()?;
        if socials
            .iter()
            .map(|s| s.id.as_str())
            .collect::<HashSet<_>>()
            .len()
            != socials.len()
        {
            return Err(Error::new("duplicate social id"));
        }
        for (id, p) in &raw.bond_presets {
            if !((-100.0..=100.0).contains(&p.friendship) && (-100.0..=100.0).contains(&p.romance)) {
                return Err(Error::new(format!("bond preset '{id}': friendship and romance are -100..100")));
            }
        }
        let social_rules = social::parse_rules(&raw.social_rules, &raw.bond_presets, &ix)?;
        let chats = socials.iter().flat_map(|s| s.actor_gain.iter().chain(&s.target_gain).enumerate());
        let urgent = raw
            .autonomy
            .urgent
            .as_ref()
            .map(|list| list.iter().try_fold(0u32, |m, id| Ok::<_, Error>(m | 1 << lookup(&need_index, id, "autonomy.urgent", "need")?)))
            .transpose()?;
        let autonomy = raw.autonomy.rules(urgent, chats.filter(|(_, g)| **g > 0.0).fold(0, |m, (n, _)| m | 1 << n));

        let need_array =
            |map: &HashMap<String, f32>, ctx: &str| -> Result<[f32; MAX_NEEDS], Error> {
                let mut out = [0.0; MAX_NEEDS];
                for (k, v) in map {
                    out[lookup(&need_index, k, ctx, "need")?] = *v;
                }
                Ok(out)
            };
        let feeling = |id: &Option<String>, ctx: &str| -> Result<Option<usize>, Error> {
            id.as_deref()
                .map(|m| {
                    feeling_index
                        .get(m)
                        .copied()
                        .ok_or_else(|| Error::new(format!("{ctx}: unknown feeling '{m}'")))
                })
                .transpose()
        };
        let mut stages = Vec::with_capacity(raw.life.stages.len());
        for st in &raw.life.stages {
            let ctx = format!("life stage '{}'", st.id);
            if stages.last().is_some_and(|p: &LifeStage| st.from <= p.from) {
                return Err(Error::new(format!("{ctx}: stages go from young to old")));
            }
            stages.push(LifeStage {
                id: st.id.clone(),
                label: st.label.clone().unwrap_or_else(|| st.id.clone()),
                from: st.from,
                mods: build_modifiers(&st.effects, &ix, &ctx)?,
                feeling: feeling(&st.feeling, &ctx)?,
                adult: st.adult.unwrap_or(!st.baby && !st.school),
                baby: st.baby,
                school: st.school,
                scale: st.scale.filter(|s| s.is_finite() && *s > 0.0).unwrap_or(1.0),
            });
        }
        let start_age = raw.life.start_age.unwrap_or([25.0, 50.0]);
        let basic_pension = raw.life.basic_pension.unwrap_or(0);
        if basic_pension < 0 {
            return Err(Error::new(format!("life.basicPension {basic_pension} must be ≥ 0")));
        }
        let life = LifeRules {
            days_per_year: raw.life.days_per_year.unwrap_or(2.0).max(0.1),
            not_baby: if stages.is_empty() { 0 } else { every_stage & !babies },
            stages,
            start_age: [start_age[0].min(start_age[1]), start_age[0].max(start_age[1])],
            retire_at: raw.life.retire_at,
            pension: raw.life.pension.unwrap_or(0.0).max(0.0),
            basic_pension,
            retire_feeling: feeling(&raw.life.retire_feeling, "life")?,
            death: raw.life.death.as_ref().map(|d| DeathRules {
                from: d.from,
                per_year: d.per_year.clamp(0.0, 1.0),
                growth: d.growth.max(0.0),
            }),
            grief: GriefRules {
                feeling: feeling(&raw.life.grief.feeling, "life.grief")?,
                light_feeling: feeling(&raw.life.grief.light_feeling, "life.grief")?,
                close: raw.life.grief.close.unwrap_or(60.0),
                friend: raw.life.grief.friend.unwrap_or(35.0),
            },
            moving: raw.life.moving.as_ref().map(|m| MovingRules {
                hour: m.hour.unwrap_or(11.0).clamp(0.0, 23.9),
                partners: m.partners.unwrap_or(0.2).clamp(0.0, 1.0),
                romance: m.romance.unwrap_or(60.0),
                leave_home_age: m.leave_home_age.unwrap_or(23.0),
                leave_home: m.leave_home.unwrap_or(0.1).clamp(0.0, 1.0),
                leave_town_age: m.leave_town_age.unwrap_or(30.0),
                leave_town: m.leave_town.unwrap_or(0.05).clamp(0.0, 1.0),
            }),
            school: match &raw.life.school {
                None => None,
                Some(sc) => {
                    let mut skills = Vec::new();
                    for (id, w) in &sc.skills {
                        skills.push((lookup(&skill_index, id, "life.school", "skill")?, *w));
                    }
                    skills.sort_by_key(|s| s.0);
                    let g = &sc.grades;
                    Some(SchoolRules {
                        start: sc.start.clamp(0.0, 23.0),
                        hours: sc.hours.clamp(0.5, 12.0),
                        days: sc.days.iter().filter(|&&d| d < 7).fold(0, |m, d| m | (1 << d)),
                        skills,
                        attend: g.attend.unwrap_or(1.0),
                        mood: g.mood.unwrap_or(3.0),
                        missed: g.missed.unwrap_or(5.0),
                        start_grade: g.start.unwrap_or(60.0).clamp(0.0, 100.0),
                        good: g.good.unwrap_or(70.0),
                        good_feeling: feeling(&g.good_feeling, "life.school.grades")?,
                        poor: g.poor.unwrap_or(30.0),
                        poor_feeling: feeling(&g.poor_feeling, "life.school.grades")?,
                    })
                }
            },
            pregnancy: raw.life.pregnancy.as_ref().map(|p| PregnancyRules {
                chance: p.chance.clamp(0.0, 1.0),
                days: p.days.max(1),
                max_age: p.max_age.unwrap_or(50.0),
            }),
            adoption_cost: raw.life.adoption.as_ref().map(|a| a.cost.max(0)),
            adoption_chance: raw.life.adoption.as_ref().map_or(0.0, |a| a.neighbours.clamp(0.0, 1.0)),
            childcare: raw.life.childcare.as_ref().map(|c| ChildcareRules { cost: c.cost.max(0), below: c.below.unwrap_or(0.45).clamp(0.0, 1.0) }),
            newcomers: raw.life.newcomers.as_ref().map(|n| NewcomerRules {
                hour: n.hour.unwrap_or(12.0).clamp(0.0, 23.9),
                days: n.days.unwrap_or(3.0).max(1.0),
            }),
        };
        let valid_shift = |start: f32, hours: f32, days_ok: bool| {
            (0.0..24.0).contains(&start) && hours > 0.0 && hours <= 16.0 && days_ok
        };
        let mut careers = Vec::with_capacity(raw.careers.len());
        for c in &raw.careers {
            if c.levels.is_empty() {
                return Err(Error::new(format!("career '{}' has no levels", c.id)));
            }
            let levels = c
                .levels
                .iter()
                .map(|l| {
                    if !valid_shift(l.start, l.hours, l.days.iter().all(|d| *d <= 6)) {
                        return Err(Error::new(format!(
                            "career '{}': invalid shift for '{}'",
                            c.id, l.title
                        )));
                    }
                    Ok(CareerLevel {
                        title: l.title.clone(),
                        grade: None,
                        pay: l.pay,
                        start_hour: l.start,
                        hours: l.hours,
                        days: l.days.iter().fold(0u8, |m, d| m | (1 << d)),
                        requires: skill_list(&l.requires, &c.id)?,
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?;
            careers.push(CareerDef {
                id: c.id.clone(),
                label: c.label.clone(),
                category: None,
                levels,
                work_effects: need_array(&c.work_effects, &c.id)?,
                skills: skill_list(&c.skills, &c.id)?,
            });
        }

        // Career categories: every track becomes a career with one level per grade.
        let grades: Vec<GradeDef> = raw
            .grades
            .iter()
            .map(|g| GradeDef {
                id: g.id.clone(),
                label: g.label.clone(),
                pay_per_hour: g.pay_per_hour,
                skill_level: g.skill_level,
            })
            .collect();
        let mut career_categories = Vec::with_capacity(raw.career_categories.len());
        for (ci, cat) in raw.career_categories.iter().enumerate() {
            let work_effects = need_array(&cat.work_effects, &cat.id)?;
            for track in &cat.tracks {
                if track.titles.is_empty() || track.titles.len() > grades.len() {
                    return Err(Error::new(format!(
                        "track '{}' needs 1..={} titles (one per grade)",
                        track.id,
                        grades.len()
                    )));
                }
                let shift = track.shift.unwrap_or(cat.shift);
                if !valid_shift(shift.start, shift.hours, shift.days.1) || shift.days.0 == 0 {
                    return Err(Error::new(format!("track '{}': invalid shift", track.id)));
                }
                let skills = skill_list(&track.skills, &track.id)?;
                let levels = track
                    .titles
                    .iter()
                    .zip(&grades)
                    .enumerate()
                    .map(|(gi, (title, grade))| CareerLevel {
                        title: title.clone(),
                        grade: Some(gi),
                        pay: (grade.pay_per_hour * cat.pay_factor * shift.hours).round() as i64,
                        start_hour: shift.start,
                        hours: shift.hours,
                        days: shift.days.0,
                        requires: skills
                            .iter()
                            .map(|&(s, w)| (s, (grade.skill_level * w).round()))
                            .collect(),
                    })
                    .collect();
                careers.push(CareerDef {
                    id: track.id.clone(),
                    label: track.label.clone(),
                    category: Some(ci),
                    levels,
                    work_effects,
                    skills,
                });
            }
            career_categories.push(CategoryDef {
                id: cat.id.clone(),
                label: cat.label.clone(),
            });
        }
        if careers
            .iter()
            .map(|c| c.id.as_str())
            .collect::<HashSet<_>>()
            .len()
            != careers.len()
        {
            return Err(Error::new("duplicate career id"));
        }
        let cr = &raw.career_rules;
        let mut workweek: Vec<WorkweekRule> = cr
            .workweek
            .iter()
            .map(|w| WorkweekRule {
                fit: w.fit,
                days: w.days,
                label: w.label.clone(),
            })
            .collect();
        workweek.sort_by(|a, b| a.fit.total_cmp(&b.fit));
        let career_rules = CareerRules {
            commute_minutes: time(cr.commute_minutes.unwrap_or(30.0), "careerRules.commuteMinutes")?,
            late_minutes: cr.late_minutes.unwrap_or(60.0),
            missed_penalty: cr.missed_penalty.unwrap_or(20.0),
            performance_per_shift: cr.performance_per_shift.unwrap_or(12.0),
            performance_per_fit: cr.performance_per_fit.unwrap_or(0.0),
            performance_per_mood: cr.performance_per_mood.unwrap_or(15.0),
            probation_levels: cr.probation_levels.unwrap_or(0.0),
            workweek,
            promotion_feeling: feeling(&cr.promotion_feeling, "careerRules")?,
            missed_feeling: feeling(&cr.missed_feeling, "careerRules")?,
            market: match &cr.market {
                None => None,
                Some(m) => Some(JobMarket {
                    hour: m.hour.unwrap_or(9.5).clamp(0.0, 23.9),
                    daily_chance: m.daily_chance.unwrap_or(0.35).clamp(0.0, 1.0),
                    quit_below: m.quit_below.unwrap_or(0.3),
                    quit_chance: m.quit_chance.unwrap_or(0.15).clamp(0.0, 1.0),
                    min_shifts_before_quit: m.min_shifts_before_quit.unwrap_or(10),
                    fire_after_missed: m.fire_after_missed.unwrap_or(3).max(1),
                    cooldown_days: m.cooldown_days.unwrap_or(2),
                    hired_feeling: feeling(&m.hired_feeling, "careerRules.market")?,
                    quit_feeling: feeling(&m.quit_feeling, "careerRules.market")?,
                    fired_feeling: feeling(&m.fired_feeling, "careerRules.market")?,
                }),
            },
        };

        let sr = &raw.skill_rules;
        let skill_rules = SkillRules {
            max_level: sr.max_level.unwrap_or(10.0),
            work_gain_per_hour: sr.work_gain_per_hour.unwrap_or(0.05),
            slowdown: sr.slowdown.unwrap_or(0.25),
            interest: sr.interest.unwrap_or(0.05),
            goal_interest: sr.goal_interest.unwrap_or(0.25),
            effect_per_level: sr.effect_per_level.unwrap_or(0.05),
            fatigue_per_hour: sr.fatigue_per_hour.unwrap_or(0.0),
            fatigue_recovery_per_hour: sr.fatigue_recovery_per_hour.unwrap_or(0.0),
        };
        let dr = &raw.day_rhythm;
        // Tags content names here must exist (a typo would quietly switch something off).
        let tag_mask = |tags: &[String], ctx: &str| -> Result<TagMask, Error> {
            tags.iter().try_fold(0, |m: TagMask, t| Ok(m | (1 << lookup(&tag_index, t, ctx, "tag")?)))
        };
        let multipliers =
            |map: &HashMap<String, f32>, ctx: &str| -> Result<[f32; MAX_NEEDS], Error> {
                let mut out = [1.0; MAX_NEEDS];
                for (k, v) in map {
                    if !(*v >= 0.0 && v.is_finite()) {
                        return Err(Error::new(format!("{ctx}: the {k} multiplier {v} must be a number ≥ 0")));
                    }
                    out[lookup(&need_index, k, ctx, "need")?] = *v;
                }
                Ok(out)
            };
        // What's fetched (`fetch`) is fetched with an interaction of something that provides it.
        for (o, inter) in objects.iter().flat_map(|o: &ObjectDef| o.interactions.iter().map(move |i| (o, i))) {
            let Some(fetch) = inter.fetch.as_deref() else { continue };
            let source = |d: &ObjectDef| d.provides & inter.requires == inter.requires && d.interactions.iter().any(|i| i.id == fetch && i.recipe.is_none() && i.eats.is_none());
            if !objects.iter().any(source) {
                return Err(Error::new(format!(
                    "object '{}' interaction '{}': fetches with '{fetch}', which nothing that provides what it requires has (a plain use: no recipe, no eat)",
                    o.id, inter.id
                )));
            }
        }
        let sleep_tags = tag_mask(&dr.sleep_tags, "dayRhythm.sleepTags")?;
        if let Some((o, i)) = objects.iter().find_map(|o: &ObjectDef| o.interactions.iter().find(|i| i.spare && i.tags & sleep_tags == 0).map(|i| (o, i))) {
            return Err(Error::new(format!("{}.{}: `spare` is for a night's sleep (a `dayRhythm.sleepTags` tag)", o.id, i.id)));
        }
        let mut slept = [0.0f32; MAX_NEEDS];
        for inter in objects.iter().flat_map(|o: &ObjectDef| &o.interactions).filter(|i| i.tags & sleep_tags != 0) {
            for (n, g) in inter.total_gain.iter().enumerate() {
                slept[n] += g.max(0.0);
            }
        }
        let day_rhythm = DayRhythm {
            wake_hour: dr.wake_hour.unwrap_or(7.0),
            bed_hour: dr.bed_hour.unwrap_or(22.5),
            sleep_tags,
            night_decay: multipliers(&dr.night_decay, "dayRhythm")?,
            asleep_decay: multipliers(&dr.asleep_decay, "dayRhythm")?,
            work_decay: multipliers(&dr.work_decay, "dayRhythm")?,
            wake_for: match &dr.wake_for {
                None => u32::MAX,
                Some(list) => list.iter().try_fold(0u32, |m, id| {
                    Ok::<_, Error>(m | 1 << lookup(&need_index, id, "dayRhythm.wakeFor", "need")?)
                })?,
            },
            wake_before_work_minutes: dr.wake_before_work_minutes.unwrap_or(75.0),
            rest_need: (0..MAX_NEEDS).filter(|&n| slept[n] > 0.0).max_by(|&a, &b| slept[a].total_cmp(&slept[b])),
            lie_in_hours: dr.lie_in_hours.unwrap_or(0.5),
            sleep_quality_bonus: dr.sleep_quality_bonus.unwrap_or(0.06),
            nap_tags: tag_mask(&dr.nap_tags, "dayRhythm.napTags")?,
            nap_cooldown_hours: dr.nap_cooldown_hours.unwrap_or(4.0),
            exhausted_below: dr.exhausted_below.unwrap_or(0.2),
            habits: dr
                .habits
                .iter()
                .map(|h| {
                    let need = lookup(&need_index, &h.need, "dayRhythm.habits", "need")?;
                    Ok::<_, Error>(Habit { at: h.at, need, below: h.below.clamp(0.0, 1.0) })
                })
                .collect::<Result<_, _>>()?,
            ready_minutes: dr.ready_minutes.unwrap_or(30.0).max(0.0),
            work_breaks: dr
                .work_breaks
                .iter()
                .map(|b| {
                    let need = lookup(&need_index, &b.need, "dayRhythm.workBreaks", "need")?;
                    let below = b.below.clamp(0.0, 1.0);
                    Ok::<_, Error>(WorkBreak { need, below, gain: b.gain.max(0.0), cost: b.cost.max(0) })
                })
                .collect::<Result<_, _>>()?,
            baby_wakes: dr
                .baby_wakes
                .as_ref()
                .map(|b| {
                    let mask = match &b.needs {
                        None => (0..needs.len()).filter(|&n| !needs[n].room).fold(0u32, |m, n| m | 1 << n),
                        Some(list) => list.iter().try_fold(0u32, |m, id| {
                            Ok::<_, Error>(m | 1 << lookup(&need_index, id, "dayRhythm.babyWakes", "need")?)
                        })?,
                    };
                    let below = b.below.unwrap_or(0.3).clamp(0.0, 1.0);
                    Ok::<_, Error>(BabyWakes {
                        needs: mask,
                        below,
                        tend_below: b.tend_below.unwrap_or(0.6).clamp(below, 1.0),
                        cooldown_ticks: (time(b.cooldown_minutes.unwrap_or(90.0), "dayRhythm.babyWakes.cooldownMinutes")?.max(0.0) / crate::MINUTES_PER_TICK) as u64,
                        finish_minutes: b.finish_minutes.unwrap_or(15.0).max(0.0),
                        feeling: feeling(&b.feeling, "dayRhythm.babyWakes")?,
                    })
                })
                .transpose()?,
        };
        let rent = raw
            .economy
            .rent
            .as_ref()
            .map(|r| {
                Ok::<_, Error>(RentRules {
                    weekday: r.weekday.min(6),
                    hour: r.hour.clamp(0.0, 23.9),
                    base: r.base,
                    per_tile: r.per_tile,
                    bills_base: r.bills_base,
                    bills_rate: r.bills_rate.max(0.0),
                    per_resident: r.per_resident.max(0),
                    debt_feeling: feeling(&r.debt_feeling, "economy.rent")?,
                })
            })
            .transpose()?;
        if raw.accidents.len() > MAX_ACCIDENTS {
            return Err(Error::new(format!("at most {MAX_ACCIDENTS} accidents")));
        }
        let accidents = raw
            .accidents
            .iter()
            .map(|a| {
                let ctx = format!("accident '{}'", a.id);
                Ok(AccidentDef {
                    id: a.id.clone(),
                    need: lookup(&need_index, &a.need, &ctx, "need")?,
                    story: a.story.is_some(),
                    effects: need_array(&a.effects, &ctx)?,
                    dirt: a.dirt.clamp(0.0, 1.0),
                    feeling: feeling(&a.feeling, &ctx)?,
                    cost: a.cost.max(0),
                    cooldown_ticks: (time(a.cooldown_hours.unwrap_or(2.0), &ctx)?.max(0.0) * 60.0 / crate::MINUTES_PER_TICK) as u64,
                    grace_ticks: (time(a.grace_minutes.unwrap_or(30.0), &ctx)?.max(0.0) / crate::MINUTES_PER_TICK) as u64,
                    crowded_grace_ticks: a.crowded_grace_minutes.map(|m| time(m, &ctx)).transpose()?.map(|m| (m.max(0.0) / crate::MINUTES_PER_TICK) as u64),
                    another_room: a
                        .another_room
                        .as_ref()
                        .map(|id| {
                            raw.room_kinds
                                .iter()
                                .position(|k| &k.id == id)
                                .ok_or_else(|| Error::new(format!("{ctx}: unknown room kind '{id}'")))
                        })
                        .transpose()?,
                    rest: a
                        .rest
                        .as_ref()
                        .map(|r| -> Result<AccidentRest, Error> {
                            let minutes = r.minutes.max(1.0);
                            Ok(AccidentRest {
                                label: r.label.clone(),
                                minutes,
                                pose: r.pose,
                                anim: r
                                    .anim
                                    .as_deref()
                                    .map(|n| anim_index(n).ok_or_else(|| Error::new(format!("{ctx}: unknown anim '{n}' (declare it in \"animations\")"))))
                                    .transpose()?,
                                gain_per_minute: need_array(&r.gains, &ctx)?.map(|g| g / minutes),
                                tags: tag_mask(&r.tags, &ctx)?,
                            })
                        })
                        .transpose()?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let room_kinds = raw
            .room_kinds
            .iter()
            .map(|k| {
                let ctx = format!("room kind '{}'", k.id);
                Ok(RoomKind {
                    id: k.id.clone(),
                    label: k.label.clone().unwrap_or_else(|| k.id.clone()),
                    tags: tag_mask(&k.tags, &ctx)?,
                    essentials: k.essentials.iter().map(|e| tag_mask(e, &ctx)).collect::<Result<Vec<_>, Error>>()?.into_iter().filter(|&m| m != 0).collect(),
                    size: k.size.unwrap_or([4.0, 9.0]),
                    exclusive: k.exclusive,
                    passage: k.passage,
                })
            })
            .collect::<Result<_, Error>>()?;
        let rr = &raw.room_rules;
        let room_rules = RoomRules {
            weights: rr.weights.unwrap_or([0.2, 0.2, 0.2, 0.25, 0.15]),
            window_tiles: rr.window_tiles.unwrap_or(6.0).max(0.1),
            lamp_tiles,
            decor_per_tile: rr.decor_per_tile.unwrap_or(0.125).max(0.001),
            garden_decor_per_tile: rr.garden_decor_per_tile.unwrap_or(0.05).max(0.001),
            love: rr.love.unwrap_or(0.75),
            dislike: rr.dislike.unwrap_or(0.4),
            drift: rr.drift.unwrap_or(0.5).max(0.0),
            away: rr.away.unwrap_or(0.6),
            preference: rr.preference.unwrap_or(1.0).max(0.0),
            love_feeling: feeling(&rr.love_feeling, "roomRules")?,
            dislike_feeling: feeling(&rr.dislike_feeling, "roomRules")?,
            dirt: {
                let mut d: Vec<(TagMask, f32)> =
                    rr.dirt.iter().map(|(t, &v)| Ok((tag_mask(std::slice::from_ref(t), "roomRules.dirt")?, v.max(0.0)))).collect::<Result<_, Error>>()?;
                d.sort_by_key(|e| e.0);
                d
            },
            clean: CleanRules {
                label: rr.clean.label.clone().unwrap_or_else(|| "Tidy up".into()),
                minutes: rr.clean.minutes.unwrap_or(12.0).max(1.0),
                threshold: rr.clean.threshold.unwrap_or(0.15),
                amount: rr.clean.amount.unwrap_or(0.7),
                interest: rr.clean.interest.unwrap_or(0.6),
                tags: tag_mask(&clean_tags, "roomRules.clean")?,
                anim: animations.iter().position(|a| a == "clean"),
            },
        };
        // Interactions without their own mess make what their tags make.
        for o in &mut objects {
            for it in &mut o.interactions {
                if it.dirt < 0.0 {
                    it.dirt = room_rules.dirt.iter().filter(|(m, _)| it.tags & m != 0).map(|(_, v)| *v).fold(0.0, f32::max);
                }
            }
        }
        let or = &raw.object_rules;
        let rp = &or.repair;
        let object_rules = ObjectRules {
            wear_per_quality: or.wear.per_quality.unwrap_or(0.2).clamp(0.0, 0.3),
            repair: RepairRules {
                label: rp.label.clone().unwrap_or_else(|| "Repair".into()),
                minutes: rp.minutes.unwrap_or(45.0).max(1.0),
                skill: rp.skill.as_deref().map(|s| lookup(&skill_index, s, "objectRules.repair", "skill")).transpose()?,
                skill_gain: rp.skill_gain_per_hour.unwrap_or(0.3),
                chance: rp.chance.unwrap_or(0.5),
                chance_per_level: rp.chance_per_level.unwrap_or(0.08),
                interest: rp.interest.unwrap_or(0.7),
                cost: rp.cost.unwrap_or(0.3),
                // (By default its chores tag, if content has one.)
                tags: match &rp.tags {
                    Some(tags) => tag_mask(tags, "objectRules.repair")?,
                    None => tag_index.get("chores").map_or(0, |&i| 1 << i),
                },
                anim: animations.iter().position(|a| a == "tinker"),
            },
            max_quality: or.max_quality.unwrap_or(3),
            quality_bonus: or.quality_bonus.unwrap_or(0.25),
            upgrade_cost: or.upgrade_cost.unwrap_or(0.3),
            resale: or.resale.unwrap_or(0.5),
        };
        // (Selling for more than was paid would be a money machine; an upgrade costs something.)
        if !(0.0..=1.0).contains(&object_rules.resale) {
            return Err(Error::new(format!("objectRules.resale {} must be 0..1 (a share of what was paid)", object_rules.resale)));
        }
        if !(object_rules.upgrade_cost >= 0.0 && object_rules.upgrade_cost.is_finite()) {
            return Err(Error::new(format!("objectRules.upgradeCost {} must be a number ≥ 0", object_rules.upgrade_cost)));
        }
        let build = BuildRules {
            wall: raw.build.wall.unwrap_or(40),
            door: raw.build.door.unwrap_or(150),
            window: raw.build.window.unwrap_or(120),
            remove: raw.build.remove.unwrap_or(10),
            fence: raw.build.fence.unwrap_or(25),
            gate: raw.build.gate.unwrap_or(100),
            fences: build_styles(&raw.fence_styles, "fence style")?,
            roofs: build_styles(&raw.roof_styles, "roof style")?,
            roof_colors: build_styles(&raw.roof_colors, "roof colour")?,
            coverings: build_styles(&raw.wall_coverings, "wall covering")?,
            doors: build_styles(&raw.door_styles, "door style")?,
            windows: build_styles(&raw.window_styles, "window style")?,
            floors: build_styles(&raw.floor_coverings, "floor covering")?,
            storeys: raw.build.storeys.unwrap_or(1).clamp(1, 4),
            basement: raw.build.basement,
        };
        let schedule = raw
            .schedule
            .iter()
            .map(|w| {
                Ok(ScheduleWindow {
                    from: w.from,
                    to: w.to,
                    mods: build_modifiers(&w.effects, &ix, "schedule")?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let v = &raw.visits;
        let visits = VisitRules {
            gains: need_array(&v.needs, "visits")?,
            hours: v.hours.unwrap_or(3.0),
            directed_hours: v
                .directed_hours
                .unwrap_or(v.hours.unwrap_or(3.0) * 2.0),
            min_friendship: v.min_friendship.unwrap_or(15.0),
            earliest_hour: v.earliest_hour.unwrap_or(9.0),
            latest_hour: v.latest_hour.unwrap_or(21.0),
            tags: 1 << tag_index[VISIT_TAG],
            company: v.company.unwrap_or(0.0).max(0.0),
            bored_minutes: v.bored_minutes.map(|m| m.max(1.0)),
            farewell: v
                .farewell
                .iter()
                .map(|id| socials.iter().position(|s| &s.id == id).ok_or_else(|| Error::new(format!("visits.farewell: unknown social '{id}'"))))
                .collect::<Result<_, _>>()?,
            farewell_minutes: v.farewell_minutes.unwrap_or(10.0).max(1.0),
            door: match &v.door {
                None => None,
                Some(d) => Some(DoorRules {
                    wait_minutes: d.wait_minutes.unwrap_or(10.0).max(1.0),
                    welcome_friendship: d.welcome_friendship.unwrap_or(-20.0),
                    greet: match &d.greet {
                        None => None,
                        Some(id) => Some(
                            socials
                                .iter()
                                .position(|s| &s.id == id)
                                .ok_or_else(|| Error::new(format!("visits.door: unknown social '{id}'")))?,
                        ),
                    },
                    turned_away: feeling(&d.turned_away_feeling, "visits.door")?,
                    knock_anim: match &d.knock_anim {
                        None => None,
                        Some(a) => Some(
                            anim_index(a).ok_or_else(|| Error::new(format!("visits.door: unknown anim '{a}' (declare it in \"animations\")")))?,
                        ),
                    },
                }),
            },
        };

        let mut traits = Vec::with_capacity(raw.traits.len());
        for t in &raw.traits {
            traits.push(TraitDef {
                id: t.id.clone(),
                label: t.label.clone(),
                conflicts: t.conflicts.clone(),
                mods: build_modifiers(&t.effects, &ix, &t.id)?,
                starting_skills: skill_array(&t.starting_skills, &t.id)?,
            });
        }
        let trait_ids: HashSet<&str> = traits.iter().map(|t| t.id.as_str()).collect();
        if trait_ids.len() != traits.len() {
            return Err(Error::new("duplicate trait id"));
        }
        for t in &traits {
            if let Some(c) = t.conflicts.iter().find(|c| !trait_ids.contains(c.as_str())) {
                return Err(Error::new(format!(
                    "trait '{}' conflicts with unknown trait '{c}'",
                    t.id
                )));
            }
        }

        let mut perks = Vec::with_capacity(raw.perks.len());
        for p in &raw.perks {
            perks.push(PerkDef {
                id: p.id.clone(),
                label: p.label.clone(),
                cost: p.cost,
                mods: build_modifiers(&p.effects, &ix, &p.id)?,
            });
        }
        if perks
            .iter()
            .map(|p| p.id.as_str())
            .collect::<HashSet<_>>()
            .len()
            != perks.len()
        {
            return Err(Error::new("duplicate perk id"));
        }

        // (Who someone is attracted to is a bit per gender.)
        if raw.genders.len() > MAX_GENDERS {
            return Err(Error::new(format!("at most {MAX_GENDERS} genders")));
        }
        let genders: Vec<GenderDef> = raw
            .genders
            .iter()
            .map(|g| GenderDef {
                id: g.id.clone(),
                label: g.label.clone(),
            })
            .collect();

        let buff_feelings = feelings.iter().any(|m| m.effects.is_some());
        let talk_anim = anim_index(TALK_ANIM);
        let cuddle_anim = anim_index("cuddle");
        let (activities, planner, goals) =
            crate::planner::parse(&raw.activities, &raw.planner, &raw.goals, &tag_index, &feelings)?;
        let phone = crate::phone::parse(&raw.phone, &ix, &animations, &life.stages)?;
        let laundry = match &raw.laundry {
            Some(l) if l.per_day > 0.0 && l.per_day.is_finite() && l.most >= 1.0 && l.most.is_finite() => Some(LaundryRules { per_day: l.per_day, most: l.most }),
            Some(_) => return Err(Error::new("laundry: `perDay` above 0 and `most` at least 1 (loads)")),
            None => None,
        };
        Ok(Self {
            needs,
            objects,
            traits,
            perks,
            genders,
            tags,
            rules: raw.rules,
            emotions,
            feelings,
            socials,
            bond_presets: raw.bond_presets,
            social_rules,
            careers,
            career_rules,
            schedule,
            visits,
            starting_funds: raw.economy.starting_funds.unwrap_or(1500),
            skills,
            skill_rules,
            grades,
            career_categories,
            day_rhythm,
            autonomy,
            rent,
            object_rules,
            room_kinds,
            room_rules,
            accidents,
            life,
            names: raw.names.clone(),
            build,
            animations,
            talk_anim,
            cuddle_anim,
            buff_feelings,
            activities,
            planner,
            goals,
            cooking,
            items,
            recipes,
            phone,
            laundry,
            object_index,
        })
    }

    pub fn object_index(&self, id: &str) -> Option<usize> {
        self.object_index.get(id).copied()
    }

    pub fn recipe_index(&self, id: &str) -> Option<usize> {
        self.recipes.iter().position(|r| r.id == id)
    }

    pub fn item_index(&self, id: &str) -> Option<usize> {
        self.items.iter().position(|i| i.id == id)
    }

    pub fn feeling_index(&self, id: &str) -> Option<usize> {
        self.feelings.iter().position(|m| m.id == id)
    }

    pub fn career_index(&self, id: &str) -> Option<usize> {
        self.careers.iter().position(|c| c.id == id)
    }

    pub fn skill_index(&self, id: &str) -> Option<usize> {
        self.skills.iter().position(|s| s.id == id)
    }

    /// Starting skill levels for a Sim with these traits (highest trait value per skill).
    pub fn starting_skills(&self, traits: &[String]) -> [f32; MAX_SKILLS] {
        let mut out = [0.0f32; MAX_SKILLS];
        for t in traits
            .iter()
            .filter_map(|id| self.traits.iter().find(|d| &d.id == id))
        {
            for (o, s) in out.iter_mut().zip(t.starting_skills) {
                *o = o.max(s);
            }
        }
        out
    }

    /// Combined autonomy modifiers of every schedule window active at `hour`.
    pub fn schedule_at(&self, hour: f32) -> Modifiers {
        let mut mods = Modifiers::default();
        for w in self.schedule.iter().filter(|w| w.contains(hour)) {
            mods.combine(&w.mods);
        }
        mods
    }

    pub fn social_index(&self, id: &str) -> Option<usize> {
        self.socials.iter().position(|s| s.id == id)
    }

    /// Validates gender and attraction ids. With no genders defined, any value is accepted.
    pub fn validate_identity(&self, gender: &str, attracted_to: &[String]) -> Result<(), Error> {
        if self.genders.is_empty() {
            return Ok(());
        }
        let known = |g: &str| self.genders.iter().any(|d| d.id == g);
        if !known(gender) {
            return Err(Error::new(format!("unknown gender '{gender}'")));
        }
        if let Some(g) = attracted_to.iter().find(|g| !known(g)) {
            return Err(Error::new(format!("unknown gender '{g}' in attraction")));
        }
        Ok(())
    }

    /// Validates a character's traits and perks against the rules and returns their combined effect.
    pub fn character_modifiers(
        &self,
        traits: &[String],
        perks: &[String],
    ) -> Result<Modifiers, Error> {
        let rules = self.rules;
        if traits.len() < rules.min_traits || traits.len() > rules.max_traits {
            return Err(Error::new(format!(
                "pick {}..={} traits (got {})",
                rules.min_traits,
                rules.max_traits,
                traits.len()
            )));
        }
        let mut mods = Modifiers::default();
        for (i, id) in traits.iter().enumerate() {
            if traits[..i].contains(id) {
                return Err(Error::new(format!("trait '{id}' picked twice")));
            }
            let def = self
                .traits
                .iter()
                .find(|t| &t.id == id)
                .ok_or_else(|| Error::new(format!("unknown trait '{id}'")))?;
            if let Some(c) = def.conflicts.iter().find(|c| traits.contains(c)) {
                return Err(Error::new(format!("traits '{id}' and '{c}' conflict")));
            }
            mods.combine(&def.mods);
        }
        let mut spent = 0;
        for (i, id) in perks.iter().enumerate() {
            if perks[..i].contains(id) {
                return Err(Error::new(format!("perk '{id}' picked twice")));
            }
            let def = self
                .perks
                .iter()
                .find(|p| &p.id == id)
                .ok_or_else(|| Error::new(format!("unknown perk '{id}'")))?;
            spent += def.cost;
            mods.combine(&def.mods);
        }
        if spent > rules.perk_points {
            return Err(Error::new(format!(
                "perks cost {spent} points, budget is {}",
                rules.perk_points
            )));
        }
        Ok(mods)
    }
}

/// A time content gives (minutes or hours): a number (1e39 reads as infinity, which would
/// overflow the ticks it becomes).
fn time(v: f32, ctx: &str) -> Result<f32, Error> {
    if v.is_finite() { Ok(v) } else { Err(Error::new(format!("{ctx}: times must be numbers"))) }
}

fn lookup(index: &HashMap<&str, usize>, key: &str, ctx: &str, kind: &str) -> Result<usize, Error> {
    index
        .get(key)
        .copied()
        .ok_or_else(|| Error::new(format!("{ctx}: unknown {kind} '{key}'")))
}

/// Build looks from content: ids must be unique, prices not negative, and (stored as a byte) at
/// most 250 of each.
fn build_styles(raw: &[BuildStyleRaw], what: &str) -> Result<Vec<BuildStyle>, Error> {
    if raw.len() > 250 {
        return Err(Error::new(format!("too many {what}s (at most 250)")));
    }
    let mut out: Vec<BuildStyle> = Vec::with_capacity(raw.len());
    for r in raw {
        if out.iter().any(|s| s.id == r.id) {
            return Err(Error::new(format!("duplicate {what} '{}'", r.id)));
        }
        if r.price < 0 {
            return Err(Error::new(format!(
                "{what} '{}' has a negative price",
                r.id
            )));
        }
        out.push(BuildStyle {
            id: r.id.clone(),
            price: r.price,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTENT: &str = r#"{
        "needs":[{"id":"hunger","label":"Hunger","decayPerHour":0.06,"icon":"x"},
                 {"id":"fun","label":"Fun","decayPerHour":0.06},
                 {"id":"social","label":"Social","decayPerHour":0.06}],
        "objects":[{"id":"fridge","name":"Fridge","model":"model.fridge",
          "interactions":[{"id":"snack","label":"Snack","minutes":10,"effects":{"hunger":0.3},"tags":["food"]}]},
          {"id":"tv","name":"TV","interactions":[{"id":"watch","label":"Watch","minutes":30,"effects":{"fun":0.5},"tags":["screen","fun"]}]}],
        "genders":[{"id":"female","label":"Female"},{"id":"male","label":"Male"}],
        "emotions":[{"id":"happy","label":"Happy","effects":{"tagPreference":{"friendly":1.5}}}],
        "feelings":[{"id":"goodChat","label":"Good chat","emotion":"happy","mood":0.05,"hours":2}],
        "socials":[{"id":"chat","label":"Chat","minutes":15,"tags":["social","friendly"],
          "needs":{"social":0.3},"success":{"friendship":3,"targetFriendship":3,"feeling":"goodChat"}}],
        "bondPresets":{"roommates":{"friendship":25}},
        "socialRules":{"defaultBond":"roommates","busyTags":["food"]},
        "traits":[
          {"id":"foodie","label":"Foodie","effects":{"tagPreference":{"food":2},"needDecay":{"hunger":1.2}}},
          {"id":"neat","label":"Neat","conflicts":["slob"]},
          {"id":"slob","label":"Slob","conflicts":["neat"]},
          {"id":"lazy","label":"Lazy","effects":{"walkSpeed":0.8,"tagAcceptance":{"friendly":0.1}}}],
        "perks":[{"id":"iron","label":"Iron stomach","cost":3,"effects":{"needDecay":{"hunger":0.5}}},
                 {"id":"fast","label":"Fast","cost":3,"effects":{"walkSpeed":1.5}}],
        "rules":{"minTraits":1,"maxTraits":2,"perkPoints":5}}"#;

    #[test]
    fn parses_and_ignores_visual_fields() {
        let c = Content::from_json(CONTENT).unwrap();
        assert_eq!(c.objects[0].interactions[0].total_gain[0], 0.3);
        assert!((c.objects[0].interactions[0].gain_per_minute[0] - 0.03).abs() < 1e-6);
        assert_eq!(c.object_index("fridge"), Some(0));
        assert_eq!(
            c.tags,
            [
                "food", "screen", "fun", "social", "friendly", "visit", "chores", "cleaning"
            ]
        );
        assert_eq!(c.objects[1].interactions[0].tags, 0b110);
        assert_eq!(c.socials[0].success.feeling, Some(0));
        assert_eq!(c.feelings[0].emotion, Some(0));
        assert_eq!(c.social_rules.busy_tags, 1);
        assert_eq!(c.social_rules.default_bond.unwrap().friendship, 25.0);
    }

    #[test]
    fn content_with_moodlet_keys_still_loads() {
        // Every key as written before the moodlet → feeling rename (serde aliases).
        let old = r#"{
            "needs":[{"id":"fun","label":"Fun","decayPerHour":0.06}],
            "skills":[{"id":"cooking","label":"Cooking"}],
            "objects":[{"id":"tv","name":"TV","interactions":[{"id":"watch","label":"Watch",
              "minutes":30,"effects":{"fun":0.5},"skill":"cooking","moodlet":"b","moodletMinSkill":2}]}],
            "moodlets":[{"id":"a","label":"A","mood":0.1,"hours":2},{"id":"b","label":"B","hours":1}],
            "socials":[{"id":"chat","label":"Chat","minutes":15,
              "success":{"moodlet":"a","targetMoodlet":"b","winnerMoodlet":"a","loserMoodlet":"b"}}],
            "socialRules":{"jealousyMoodlet":"a","heartbreakMoodlet":"b"},
            "careerRules":{"promotionMoodlet":"a","missedMoodlet":"b"},
            "economy":{"rent":{"base":100,"debtMoodlet":"b"}}}"#;
        let c = Content::from_json(old).unwrap();
        assert_eq!(c.feelings.len(), 2);
        let watch = &c.objects[0].interactions[0];
        assert_eq!((watch.feeling, watch.feeling_min_skill), (Some(1), 2.0));
        let s = &c.socials[0].success;
        assert_eq!(
            (
                s.feeling,
                s.target_feeling,
                s.winner_feeling,
                s.loser_feeling
            ),
            (Some(0), Some(1), Some(0), Some(1))
        );
        assert_eq!(c.social_rules.jealousy_feeling, Some(0));
        assert_eq!(c.social_rules.heartbreak_feeling, Some(1));
        assert_eq!(c.career_rules.promotion_feeling, Some(0));
        assert_eq!(c.career_rules.missed_feeling, Some(1));
        assert_eq!(c.rent.as_ref().unwrap().debt_feeling, Some(1));
    }

    #[test]
    fn rejects_unknown_references() {
        let bad_need = CONTENT.replace(r#""effects":{"hunger":0.3}"#, r#""effects":{"sleep":0.3}"#);
        assert!(
            Content::from_json(&bad_need)
                .unwrap_err()
                .0
                .contains("unknown need")
        );
        let bad_tag = CONTENT.replace(
            r#""tagPreference":{"food":2}"#,
            r#""tagPreference":{"cooking":2}"#,
        );
        assert!(
            Content::from_json(&bad_tag)
                .unwrap_err()
                .0
                .contains("unknown tag")
        );
        let bad_feeling = CONTENT.replace(r#""feeling":"goodChat""#, r#""feeling":"nope""#);
        assert!(
            Content::from_json(&bad_feeling)
                .unwrap_err()
                .0
                .contains("unknown feeling")
        );
    }

    #[test]
    fn combines_trait_and_perk_modifiers() {
        let c = Content::from_json(CONTENT).unwrap();
        let m = c
            .character_modifiers(&["foodie".into(), "lazy".into()], &["iron".into()])
            .unwrap();
        assert!((m.need_decay[0] - 0.6).abs() < 1e-6);
        assert!((m.walk_speed - 0.8).abs() < 1e-6);
        assert_eq!(m.preference(c.objects[0].interactions[0].tags), 2.0);
        assert_eq!(m.preference(c.objects[1].interactions[0].tags), 1.0);
        assert!((m.acceptance(c.socials[0].tags) - 0.1).abs() < 1e-6);
    }

    #[test]
    fn enforces_character_rules() {
        let c = Content::from_json(CONTENT).unwrap();
        let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        assert!(
            c.character_modifiers(&s(&[]), &s(&[])).is_err(),
            "min traits"
        );
        assert!(
            c.character_modifiers(&s(&["neat", "slob"]), &s(&[]))
                .is_err(),
            "conflict"
        );
        assert!(
            c.character_modifiers(&s(&["neat", "lazy", "foodie"]), &s(&[]))
                .is_err(),
            "max traits"
        );
        assert!(
            c.character_modifiers(&s(&["neat"]), &s(&["iron", "fast"]))
                .is_err(),
            "budget"
        );
        assert!(c.character_modifiers(&s(&["neat"]), &s(&["iron"])).is_ok());
    }

    #[test]
    fn validates_gender_and_attraction() {
        let c = Content::from_json(CONTENT).unwrap();
        assert!(c.validate_identity("female", &["male".into()]).is_ok());
        assert!(c.validate_identity("robot", &[]).is_err());
        assert!(c.validate_identity("male", &["robot".into()]).is_err());
    }

    /// Once-a-day times are kept within the day, like moving's and newcomers': rent, the job
    /// market and the planner's review at hour 24 or later would never come round.
    #[test]
    fn once_a_day_hours_stay_within_the_day() {
        use serde_json::json;
        let mut c: serde_json::Value = serde_json::from_str(CONTENT).unwrap();
        c["economy"] = json!({"rent": {"hour": 24, "base": 100}});
        c["careerRules"] = json!({"market": {"hour": 30}});
        c["planner"] = json!({"reviewHour": -1});
        let c = Content::from_json(&c.to_string()).unwrap();
        let hours = [c.rent.as_ref().unwrap().hour, c.career_rules.market.as_ref().unwrap().hour, c.planner.review_hour];
        assert_eq!(hours, [23.9, 23.9, 0.0]);
        for h in hours {
            let t = crate::clock::tick_at(3, h * 60.0).unwrap();
            assert!(crate::clock::at_minute(t, h * 60.0) && crate::clock::day(t) == 3, "hour {h} comes on day 3");
        }
    }

    /// Content that would break the game is refused, saying what's wrong: walking speeds,
    /// decays and multipliers, too many genders, recipe steps, resale and upgrade prices, a basic
    /// pension below 0, a footprint, repeated ids, bond presets, names nothing has, and times too
    /// big to be numbers.
    #[test]
    fn refuses_content_that_would_break_the_game() {
        use serde_json::json;
        let refused = |edit: &dyn Fn(&mut serde_json::Value), says: &str| {
            let mut c: serde_json::Value = serde_json::from_str(CONTENT).unwrap();
            edit(&mut c);
            match Content::from_json(&c.to_string()) {
                Ok(_) => panic!("accepted, expected: {says}"),
                Err(e) => assert!(e.to_string().contains(says), "{e} (expected: {says})"),
            }
        };
        // (1e39 is past what an f32 holds: it reads as infinity.)
        let inf = || json!(1e39);
        refused(&|c| c["traits"][3]["effects"]["walkSpeed"] = json!(0), "lazy: walkSpeed 0 must be above 0");
        refused(&|c| c["traits"][3]["effects"]["walkSpeed"] = json!(-1.0), "walkSpeed -1 must be above 0");
        refused(&|c| c["perks"][1]["effects"]["walkSpeed"] = json!(11), "at most 10");
        refused(&|c| c["perks"][1]["effects"]["walkSpeed"] = inf(), "fast: walkSpeed inf");
        refused(&|c| c["emotions"][0]["effects"]["walkSpeed"] = json!(0), "happy: walkSpeed");
        refused(&|c| c["feelings"][0]["effects"] = json!({"walkSpeed": 0}), "goodChat: walkSpeed");
        refused(&|c| c["needs"][0]["decayPerHour"] = json!(-0.1), "need 'hunger': decayPerHour -0.1");
        refused(&|c| c["traits"][0]["effects"]["needDecay"]["hunger"] = json!(-1), "foodie: needDecay -1");
        refused(&|c| c["perks"][0]["effects"]["needGain"] = json!({"hunger": -1}), "iron: needGain -1");
        refused(&|c| c["emotions"][0]["effects"]["tagPreference"]["friendly"] = json!(-1), "happy: tagPreference -1");
        refused(&|c| c["dayRhythm"] = json!({"nightDecay": {"hunger": -0.5}}), "dayRhythm: the hunger multiplier -0.5");
        refused(&|c| c["genders"] = (0..33).map(|i| json!({"id": format!("g{i}"), "label": "G"})).collect(), "at most 32 genders");
        refused(&|c| c["objectRules"] = json!({"resale": 1.5}), "objectRules.resale 1.5 must be 0..1");
        refused(&|c| c["objectRules"] = json!({"resale": -0.5}), "objectRules.resale");
        refused(&|c| c["objectRules"] = json!({"upgradeCost": -0.3}), "objectRules.upgradeCost -0.3");
        refused(&|c| c["life"] = json!({"basicPension": -5}), "life.basicPension -5 must be ≥ 0");
        refused(&|c| c["objects"][1]["footprint"] = json!([7, 1]), "object 'tv': a footprint is at most 6 × 6 tiles");
        refused(
            &|c| {
                let it = c["objects"][0]["interactions"][0].clone();
                c["objects"][0]["interactions"].as_array_mut().unwrap().push(it);
            },
            "object 'fridge': interaction 1 repeats the id 'snack'",
        );
        refused(&|c| c["needs"].as_array_mut().unwrap().push(json!({"id": "fun", "label": "Fun", "decayPerHour": 0.1})), "duplicate need id");
        refused(&|c| c["emotions"].as_array_mut().unwrap().push(json!({"id": "happy", "label": "Happy"})), "duplicate emotion id");
        refused(&|c| c["feelings"].as_array_mut().unwrap().push(json!({"id": "goodChat", "label": "Good", "hours": 1})), "duplicate feeling id");
        refused(&|c| c["bondPresets"]["roommates"]["friendship"] = json!(150), "bond preset 'roommates': friendship and romance are -100..100");
        refused(&|c| c["bondPresets"]["roommates"]["romance"] = json!(-101), "bond preset 'roommates'");
        refused(&|c| c["socialRules"]["busyTags"] = json!(["food", "nonexistent"]), "socialRules.busyTags: unknown tag 'nonexistent'");
        refused(&|c| c["socialRules"]["guestTags"] = json!(["bathrom"]), "socialRules.guestTags: unknown tag 'bathrom'");
        refused(&|c| c["dayRhythm"] = json!({"sleepTags": ["slep"]}), "dayRhythm.sleepTags: unknown tag 'slep'");
        refused(&|c| c["dayRhythm"] = json!({"napTags": ["nap"]}), "dayRhythm.napTags: unknown tag 'nap'");
        refused(&|c| c["roomKinds"] = json!([{"id": "den", "tags": ["screen"], "essentials": [["scren"]]}]), "room kind 'den': unknown tag 'scren'");
        refused(&|c| c["roomRules"] = json!({"dirt": {"fod": 0.1}}), "roomRules.dirt: unknown tag 'fod'");
        refused(&|c| c["objectRules"] = json!({"repair": {"skill": "handiness"}}), "objectRules.repair: unknown skill 'handiness'");
        refused(&|c| c["objectRules"] = json!({"repair": {"tags": ["chore"]}}), "objectRules.repair: unknown tag 'chore'");
        refused(&|c| c["accidents"] = json!([{"id": "doze", "need": "fun", "rest": {"label": "Doze", "minutes": 10, "anim": "snooze"}}]), "accident 'doze': unknown anim 'snooze'");
        refused(&|c| c["accidents"] = json!([{"id": "doze", "need": "fun", "rest": {"label": "Doze", "minutes": 10, "tags": ["sleep"]}}]), "accident 'doze': unknown tag 'sleep'");
        refused(&|c| c["activities"] = json!([{"id": "tv", "label": "TV", "tags": ["scren"]}]), "activity 'tv': unknown tag 'scren'");
        refused(&|c| c["cooking"] = json!({"eatTags": ["fod"]}), "cooking.eatTags: unknown tag 'fod'");
        refused(
            &|c| {
                c["items"] = json!([{"id": "pot", "label": "Pot"}]);
                let step = json!({"at": ["counter"], "minutes": 1});
                c["recipes"] = json!([{"id": "stew", "label": "Stew", "item": "pot", "steps": vec![step; 256]}]);
            },
            "recipe 'stew': 1 to 255 steps",
        );
        refused(&|c| c["feelings"][0]["hours"] = inf(), "goodChat: hours must be > 0");
        refused(&|c| c["objects"][0]["interactions"][0]["minutes"] = inf(), "fridge.snack: minutes must be > 0");
        refused(&|c| c["socials"][0]["minutes"] = inf(), "chat: minutes must be > 0");
        refused(&|c| c["careerRules"] = json!({"commuteMinutes": 1e39}), "careerRules.commuteMinutes: times must be numbers");
        refused(&|c| c["accidents"] = json!([{"id": "doze", "need": "fun", "cooldownHours": 1e39}]), "accident 'doze': times must be numbers");
        refused(&|c| c["dayRhythm"] = json!({"babyWakes": {"cooldownMinutes": 1e39}}), "dayRhythm.babyWakes.cooldownMinutes");
        refused(&|c| c["cooking"] = json!({"inviteMinutes": 1e39}), "cooking.inviteMinutes must be a number");
        refused(&|c| c["objects"][0]["interactions"][0]["hours"] = json!([20, 20]), "fridge.snack: `hours` is [from, to]");
        refused(&|c| c["objects"][0]["interactions"][0]["hours"] = json!([20, 25]), "fridge.snack: `hours`");
        refused(&|c| c["objects"][0]["interactions"][0]["workOn"] = json!("laundry"), "fridge.snack: `workOn` laundry needs content `laundry`");
        refused(&|c| c["laundry"] = json!({"perDay": 0, "most": 4}), "laundry: `perDay` above 0");
        // A game for two: two places, and its use for one another of the object's, alike.
        refused(&|c| c["objects"][1]["interactions"][0]["partner"] = json!(true), "tv.watch: a game for two (`partner`) is played at an object with two places");
        refused(&|c| c["objects"][1]["interactions"][0]["alone"] = json!("watch"), "tv.watch: `alone` is what's done with nobody to play: it needs `partner`");
        let game = |alone: &str, pose: &str| {
            let alone = alone.to_owned();
            let pose = pose.to_owned();
            move |c: &mut serde_json::Value| {
                c["objects"][1]["slots"] = json!(2);
                c["objects"][1]["interactions"][0]["partner"] = json!(true);
                c["objects"][1]["interactions"][0]["alone"] = json!(alone);
                c["objects"][1]["interactions"].as_array_mut().unwrap().push(json!({"id":"solo","label":"Solo","minutes":10,"pose":pose,"effects":{"fun":0.1}}));
            }
        };
        refused(&game("nothing", "stand"), "tv.watch: `alone` names no use of 'tv': 'nothing'");
        refused(&game("watch", "stand"), "tv.watch: `alone` is a use for one (no `partner`), done in the same pose as the game");
        refused(&game("solo", "sit"), "tv.watch: `alone` is a use for one");
        let mut c: serde_json::Value = serde_json::from_str(CONTENT).unwrap();
        game("solo", "stand")(&mut c);
        let content = Content::from_json(&c.to_string()).unwrap();
        let tv = content.objects.iter().find(|o| o.id == "tv").unwrap();
        assert!(tv.interactions[0].partner && tv.interactions[0].alone == Some(1) && !tv.interactions[1].partner);
        refused(
            &|c| {
                c["objects"][1]["layer"] = json!("surface");
                c["objects"][1]["footprint"] = json!([1, 1]);
                c["objects"][1]["interactions"][0]["pose"] = json!("lie");
            },
            "a thing on a surface is used standing or sitting at its top",
        );
        // (Each of these on its own is fine.)
        let mut c: serde_json::Value = serde_json::from_str(CONTENT).unwrap();
        c["objectRules"] = json!({"resale": 1.0, "upgradeCost": 0.0});
        c["bondPresets"]["roommates"]["friendship"] = json!(-100);
        c["perks"][1]["effects"]["walkSpeed"] = json!(10);
        assert!(Content::from_json(&c.to_string()).is_ok());
    }
}

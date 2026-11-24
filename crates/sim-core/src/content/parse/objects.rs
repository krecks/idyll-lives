//! Need, object, vehicle, surface and interaction shapes (content `needs`, `objects`).

use super::*;

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
    pub(crate) public_cost: Option<i64>,
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
    pub(super) need_decay: HashMap<String, f32>,
    pub(super) need_gain: HashMap<String, f32>,
    pub(super) tag_preference: HashMap<String, f32>,
    pub(super) tag_acceptance: HashMap<String, f32>,
    pub(super) tag_success: HashMap<String, f32>,
    pub(super) walk_speed: Option<f32>,
    pub(super) mood: f32,
    pub(super) skill_gain: HashMap<String, f32>,
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

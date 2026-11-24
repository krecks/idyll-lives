//! Object definitions: needs, interactions, poses, vehicles, slots and surfaces.

use super::*;

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
    /// What it costs instead on a public lot (content `publicCost`): a home kitchen's food is free at
    /// home and a meal's price at a restaurant, whose kitchen is furnished as a home's (`cost_at`).
    pub public_cost: Option<i64>,
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
    /// Takes its time whatever it fills (content `lasts`): it goes on for its `minutes` even once
    /// the needs it fills are full (the toilet: sat on for a while, not got up from the moment
    /// the bladder is empty), unless there was nothing to fill at all or it's time to head home.
    pub lasts: bool,
    /// A spare bed's night (content `spare`, on a `dayRhythm.sleepTags` use: a sofa bed pulled
    /// out for a guest): it doesn't make the room a bedroom (its tags aren't the object's
    /// `tags`, what the room is for), and residents sleep in it only with no other bed free for
    /// them.
    pub spare: bool,
    /// Brings the object's crop on (content `tends`: weeding and watering): a use done to the
    /// end grows it this share of its way from planted to ripe at once (less for one cut
    /// short), while it grows.
    pub tends: f32,
    /// Waters the garden (content `waters`, on an object outdoors: a hose reel): a use done to
    /// the end brings every growing crop on its plot this share of its way to ripe at once (less
    /// for one cut short), as `tends` brings on the one it's done at (`post::water_garden`).
    pub waters: f32,
    /// Fetching the post (content `post`, at a letterbox, with `"when": "open"` and
    /// `"leaves": "closed"`): the post came this morning (`post::deliver`), and as the use ends
    /// it's read: a letter from a friend gives a little social and the story names them
    /// (`post::read`).
    pub post: bool,
    /// Writing a letter (content `letter`: at a writing desk): a use done to the end sends it to
    /// the writer's closest friend outside the household, whose letterbox has it next morning
    /// (`post::send`; nobody's, without one).
    pub letter: bool,
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
    /// Puts a load in the object, which then runs on its own (content `loads`, on an object that
    /// `runs`: the washing put in the washer): only while it's empty, and it runs once the use
    /// ends (see `machines.rs`).
    pub loads: bool,
    /// Takes the load out once the run is done (content `unloads`): only then, and it's empty
    /// again as the use ends.
    pub unloads: bool,
    /// Leaves the household a load to work on next (content `makes`: the wet washing taken out
    /// of the washer, to dry; the dry washing off the line, to fold), as the use ends; then on
    /// with it to the nearest thing that works on it (`machines::carry_on`).
    pub makes: Option<WorkOn>,
    /// Carried in the arms on the way to it (content `brings`, an item; index into
    /// `Content::items`): the basket of wet washing to the dryer. Only shown (the snapshot's
    /// `held`): nothing is put down.
    pub brings: Option<usize>,
    /// Held in the hands while it's done (content `holds`, an item): a shirt being folded.
    /// Only shown, as `brings`.
    pub holds: Option<usize>,
    /// Done behind a shut door (content `private`; see `privacy.rs`): while it lasts the room
    /// it's done in is kept shut, and others wait outside it or go elsewhere.
    pub private: bool,
    /// Where partners try for a baby (content `conceive`: a double bed's *Try for a baby*): a
    /// game for two (`partner`) done behind a shut door (`private`), started only by the social
    /// whose success has the `conceive` effect, with the one it was with. A baby may come of it
    /// once it has run its time (`lifecycle::conceive`).
    pub conceive: bool,
    /// Sold by someone at the object (content `selfServe`, the label and anim for where nobody
    /// serves): on a public lot whose `staff` has a server, they order, are handed it and pay,
    /// and it's closed outside the post's hours; elsewhere (a household's own cart) they help
    /// themselves, as the `selfServe` label says (`staff.rs`).
    pub served: Option<crate::staff::SelfServe>,
    /// Cleans the floor (content `cleans`: the vacuum cleaner, the mop, the broom): the tool is
    /// taken to the dirtiest floor at home it can clean and used there, and as it ends the dirt
    /// comes off that floor (`sweep.rs`). Only offered with such a floor to clean.
    pub cleans: Option<Sweep>,
}

/// How a use cleans the floor (content `cleans`; see `sweep.rs`): tidying up with a tool, over
/// more of the room than `roomRules.clean` takes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sweep {
    /// Dirt taken off each tile it cleans (0..1).
    pub amount: f32,
    /// Tiles it reaches each way from the spot it's used at, in the same room; 0: the whole room.
    pub reach: u8,
    /// Hard floors only (content `hardFloors`: a mop, a broom): not a carpet (a covering whose
    /// finish is `carpet`, or a room kind's `carpet` floor where none was laid).
    pub hard: bool,
    /// The garden too (content `outdoors`: a broom sweeping the patio).
    pub outdoors: bool,
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
    /// A load the household washed, wet (`Household::laundry[1]`): the use dries it (a dryer, a
    /// washing line).
    Washed,
    /// A load dried (`Household::laundry[2]`): the use folds it (or irons it, and `makes` it
    /// `Ironed`).
    Dried,
    /// A load ironed (`Household::laundry[3]`): clothes pressed, put on a few at a time as
    /// residents dress at home (`LaundryRules::pressed`). Nothing works on it: a use only
    /// `makes` it.
    Ironed,
}

impl WorkOn {
    pub const ALL: [WorkOn; 5] = [
        WorkOn::Dishes,
        WorkOn::Laundry,
        WorkOn::Washed,
        WorkOn::Dried,
        WorkOn::Ironed,
    ];

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
            WorkOn::Dried => "there's no dry laundry to fold",
            WorkOn::Ironed => "there are no pressed clothes",
        }
    }

    /// Which of a household's loads of washing it is (`Household::laundry`): dirty, washed,
    /// dried, ironed. None for the dishes.
    pub fn load(self) -> Option<usize> {
        match self {
            WorkOn::Dishes => None,
            WorkOn::Laundry => Some(0),
            WorkOn::Washed => Some(1),
            WorkOn::Dried => Some(2),
            WorkOn::Ironed => Some(3),
        }
    }
}

/// Whether `hour` (0..24) is in the window `[from, to]` (past midnight when `from` > `to`).
pub fn in_hours(window: [f32; 2], hour: f32) -> bool {
    let [from, to] = window;
    if from <= to {
        (from..to).contains(&hour)
    } else {
        hour >= from || hour < to
    }
}

/// Dirty washing piling up (content `laundry`): what the laundry's uses work on
/// (`WorkOn::Laundry`, `WorkOn::Washed`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LaundryRules {
    /// Loads of washing each resident at home makes a day.
    pub per_day: f32,
    /// Most loads that pile up (dirty, washed waiting to dry, dried waiting to be folded, and
    /// ironed waiting to be worn, each).
    pub most: f32,
    /// What ironed washing does (content `pressed`): none, nothing irons.
    pub pressed: Option<Pressed>,
}

/// Ironed washing (content `laundry.pressed`): a dry load ironed (an ironing board's use that
/// `makes` it `ironed`, instead of folding it) is pressed clothes, which residents put on as they
/// change at home (a use that `changesOutfit`), `1 / outfits` of a load each time, and feel the
/// better for (`feeling`). With none ironed, dressing is as it always was.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pressed {
    /// The feeling of putting pressed clothes on (index into `Content::feelings`).
    pub feeling: usize,
    /// Outfits a load makes.
    pub outfits: f32,
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
    /// What a use costs where the object stands: `public_cost` on a public lot, else `cost`.
    pub fn cost_at(&self, public: bool) -> i64 {
        if public {
            self.public_cost.unwrap_or(self.cost)
        } else {
            self.cost
        }
    }

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
        (0..MAX_NEEDS)
            .filter(|&n| self.total_gain[n] > 0.0)
            .fold(0, |m, n| m | 1 << n)
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
        self.earns_feeling_over(self.minutes, elapsed, skills)
    }

    /// Whether finishing after `elapsed` of `minutes` earns the feeling.
    pub fn earns_feeling_over(
        &self,
        minutes: f32,
        elapsed: f32,
        skills: &[f32; MAX_SKILLS],
    ) -> Option<usize> {
        let skilled = self
            .skill
            .is_none_or(|s| skills[s].floor() >= self.feeling_min_skill);
        self.feeling.filter(|_| skilled && elapsed >= minutes * 0.5)
    }

    /// How long this takes at an object where `served` (someone serves at it), or where nobody
    /// does: a sold use's own `minutes` are the ordering, the hand-over and the paying; helping
    /// oneself where there's nobody to serve is `selfServe.minutes` (a coffee poured at home).
    pub fn minutes_where(&self, served: bool) -> f32 {
        if served {
            return self.minutes;
        }
        self.served
            .as_ref()
            .and_then(|s| s.minutes)
            .unwrap_or(self.minutes)
    }
}

/// How a crop grows (content `grows`): game days from planted to ripe, and lying fallow after
/// it is picked, before it is planted again.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Growth {
    pub days: f32,
    pub fallow_days: f32,
    /// Bought ripe rather than just planted (content `boughtRipe`: the lawnmower's lawn, there to
    /// mow at once; its `days` only after a mow).
    pub bought_ripe: bool,
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
    /// How far a tree's crown reaches out from its middle (m, content `crown`; 0: no crown): no
    /// house wall may stand closer, or the crown grows into the house (`home.rs`).
    pub crown: f32,
    /// The household's own (content `private`: a wardrobe, a dresser, a bath): a visitor uses none
    /// of it, chosen or told (`world::guest_may`).
    pub private: bool,
    /// A flight of stairs up to the storey above (one tile wide, climbed from the front; see
    /// `path::Stair`). Residents walk on it instead of around it.
    pub stairs: bool,
    /// The stair's shape (content `stairShape`; only for stairs): a straight flight (the default)
    /// or a spiral winding round its square footprint.
    pub stair: StairShape,
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
    /// A light that follows a use (content `light.while`, animation tags): it's lit only while
    /// someone does one of those animations at an object on its tiles (a cooker hood's light over
    /// a hob: `["stir", "bake"]`). It has no switch of its own; the simulation decides
    /// (`lights.rs`). Animation indices, resolved at load.
    pub while_anims: Vec<usize>,
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
    /// What the player is told when ordered to do a use that wants it ripe while it isn't
    /// (content `grows.notRipe`: "the grass hasn't grown back yet"); none: "the … isn't ripe yet".
    pub not_ripe: Option<String>,
    /// A machine that runs on its own (content `runs`: `{"minutes": 60}`): loaded by a use that
    /// `loads`, it runs this many game minutes with nobody at it, then waits, done, for a use
    /// that `unloads` it (see `machines.rs`, `ObjectInstance::run`).
    pub runs: Option<f32>,
    /// A machine filled a dirty thing at a time (content `rack`: how many fill it; a
    /// dishwasher): what's washed up at it goes in and waits (`ObjectInstance::rack`), and it runs
    /// once full, or once the washing up is done with it a third full (`machines::rack_in`). 0:
    /// none (a sink washes up there and then).
    pub rack: u8,
    /// Things on a wall: it belongs over a window (content `overWindow`: curtains, blinds), so
    /// its back may be on a window as well as on a plain wall, and it is drawn shut and opened
    /// (`ObjectInstance::shut`, see `curtains.rs`). Everything else on a wall needs plain wall
    /// behind it.
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
    /// Someone serves here on a public lot (content `staff`): where they stand behind it and the
    /// hours it's open (`staff.rs`).
    pub staff: Option<crate::staff::Post>,
    /// Something to ride (content `vehicle`: a bicycle, an e-scooter): parked on the lot, a
    /// resident of the household it fits rides it out to work or school and back (`ride.rs`).
    pub vehicle: Option<Vehicle>,
}

/// What kind of thing is ridden (content `vehicle.kind`): how the rider is drawn on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum VehicleKind {
    /// Sitting and pedalling.
    Bike,
    /// Standing on its deck.
    Scooter,
}

/// A thing to ride (`ObjectDef::vehicle`).
#[derive(Debug, Clone, Copy)]
pub struct Vehicle {
    pub kind: VehicleKind,
    /// How many times as fast as walking it goes: its content `speed` (times walking), on the
    /// traffic's clock (`crate::DRIVE_SCALE` times that), as a car goes on its roads.
    pub pace: f32,
    /// Life stages that ride it.
    pub stages: StageMask,
    /// Ways shorter than this many metres are walked.
    pub min_ride: f32,
    /// Keeps to roads (an e-scooter): no trails.
    pub roads_only: bool,
    /// Ridden, how far it reaches in front of its rider and behind them, and out to each side
    /// (m: its front wheel, its back wheel, its handlebars), for keeping clear of cars
    /// (`cars::Body`). Unsaid: as far as someone on foot.
    pub reach: [f32; 3],
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
    1u32.checked_shl(stage as u32)
        .is_some_and(|bit| mask & bit != 0)
}

/// Whether every stage of `mask` is older than `stage` (someone of `stage` is too young for it;
/// else too old, or too big).
pub fn all_older(mask: StageMask, stage: usize) -> bool {
    mask != 0
        && mask
            & 1u32
                .checked_shl(stage as u32 + 1)
                .map_or(u32::MAX, |b| b - 1)
            == 0
}

/// Life-stage ids (`stages` rules) as a mask of `stage_ids`, content `life.stages` in order.
pub(crate) fn stage_mask(
    stage_ids: &[&str],
    ids: &Option<Vec<String>>,
    ctx: &str,
) -> Result<Option<StageMask>, Error> {
    let Some(ids) = ids else { return Ok(None) };
    if ids.is_empty() {
        return Err(Error::new(format!(
            "{ctx}: `stages` names at least one life stage"
        )));
    }
    ids.iter()
        .try_fold(0, |m: StageMask, id| {
            match stage_ids.iter().position(|s| s == id) {
                Some(i) => Ok(m | 1 << i),
                None => Err(Error::new(format!(
                    "{ctx}: unknown life stage '{id}' (see life.stages)"
                ))),
            }
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
    /// How the one there is, whatever the use's `pose` (content `pose`): the one giving a
    /// massage stands beside the table the other lies on. None: as the use says.
    pub pose: Option<Pose>,
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
    /// Got into through a door in its front (a sauna cabin): as `Front`, and its `places` too
    /// are come to from the tile in front, never round the back through its walls.
    Door,
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

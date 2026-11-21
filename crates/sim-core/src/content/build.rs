//! Buying, upgrading, repairing and selling objects, and the build rules (land prices, styles).

use super::*;

/// Buying, upgrading and selling objects.
#[derive(Debug, Clone)]
pub struct ObjectRules {
    /// Each quality level takes this share off wear per use.
    pub wear_per_quality: f32,
    pub repair: RepairRules,
    pub set_up: SetUpRules,
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

/// Setting up something a resident asked for (`setup.rs`): it comes as a parcel to the front
/// door, and they carry it in and put it together where it goes.
#[derive(Debug, Clone)]
pub struct SetUpRules {
    /// "Set up" (the action reads "Set up the new <thing>").
    pub label: String,
    /// Game minutes putting it together at its spot, and picking the parcel up at the door.
    pub minutes: f32,
    pub fetch_minutes: f32,
    /// Hours the one who asked has to it before someone else at home may do it.
    pub help_hours: f32,
    /// Hours before it's set up for them anyway (nobody got to it).
    pub wait_hours: f32,
    /// The parcel (an item, carried in the hands); none: they carry nothing.
    pub parcel: Option<usize>,
    pub tags: TagMask,
    /// Putting it together, and picking the parcel up.
    pub anim: Option<usize>,
    pub fetch_anim: Option<usize>,
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
    /// A garage on the home lot (Build mode's Garage tool; sold back for `objectRules.resale` of it).
    pub garage: i64,
    /// A porch over a door (Build mode's Porch tool, `porches.rs`; sold back as a garage is), and
    /// the porch pieces on their own: a post, and a canopy or a step platform by the tile.
    pub porch: i64,
    pub porch_post: i64,
    pub porch_canopy: i64,
    pub porch_step: i64,
    /// What shaping the land costs (Build mode's Land tab, `Command::ShapeLand`).
    pub land: LandPrices,
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

/// What shaping the land costs (`build.land`): the page works out a stroke's price from these and
/// the simulation charges it (`Command::ShapeLand`). `earth` is a cubic metre of earth raised or
/// lowered by the brushes, `mountain` one raised or lowered by the peak, range and level tools
/// (cheap: a range is a lot of earth); the rest are a square metre: of `water` made, and of
/// `woods`, `meadow` or `fields` planted, or of cover `clear`ed.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct LandPrices {
    pub earth: f64,
    pub mountain: f64,
    pub water: f64,
    pub woods: f64,
    pub meadow: f64,
    pub fields: f64,
    pub clear: f64,
}

impl Default for LandPrices {
    fn default() -> Self {
        Self {
            earth: 3.0,
            mountain: 0.02,
            water: 2.0,
            woods: 6.0,
            meadow: 1.0,
            fields: 2.0,
            clear: 1.5,
        }
    }
}

/// A look for walls, doors or windows. The simulation only needs its id (saves) and price; the
/// renderer reads the rest of the content entry (finish, colour, glazing...).
#[derive(Debug, Clone)]
pub struct BuildStyle {
    pub id: String,
    pub price: i64,
    /// A soft floor (a floor covering whose `finish` is `carpet` or `sisal`): what a mop or a broom leaves
    /// (`Sweep::hard`).
    pub soft: bool,
}

impl BuildRules {
    /// A diagonal wall spans a tile corner to corner (√2 m): `wall × 1.414`, rounded.
    pub fn diagonal_wall(&self) -> i64 {
        (self.wall as f64 * 1.414).round() as i64
    }

    /// A metre of fence in `style`.
    pub fn fence_price(&self, style: u8) -> i64 {
        self.fences
            .get(style as usize)
            .map_or(self.fence, |s| s.price)
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

use serde::{Deserialize, Serialize};

/// What a resident can do with an item (`Command::UseItem`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemAction {
    Eat,
    Clear,
}

/// Player commands. Serialized as `{"type": "...", ...}` from the UI.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Command {
    SetSpeed {
        speed: u8,
    },
    /// "Skip quiet hours": speed up while the player's household sleeps or is at work.
    SetAutoFast {
        enabled: bool,
    },
    /// Replaces a resident's own routine blocks (the whole list).
    SetRoutines {
        sim: u32,
        routines: Vec<crate::planner::RoutineIn>,
    },
    /// Replaces the routine template of the resident's household.
    SetHouseholdRoutines {
        sim: u32,
        routines: Vec<crate::planner::RoutineIn>,
    },
    /// A resident follows (or doesn't follow) a block of the household's template.
    SkipHouseholdRoutine {
        sim: u32,
        routine: u16,
        skip: bool,
    },
    AddGoal {
        sim: u32,
        goal: crate::planner::GoalIn,
    },
    RemoveGoal {
        sim: u32,
        index: usize,
    },
    /// Moves a goal to another place on the list (higher up steers more).
    MoveGoal {
        sim: u32,
        index: usize,
        to: usize,
    },
    /// Take on a goal the resident suggested.
    AcceptSuggestion {
        sim: u32,
        index: usize,
    },
    DismissSuggestion {
        sim: u32,
        index: usize,
    },
    /// Say yes to what the resident asked to buy (it's bought and placed at home).
    AcceptPurchase {
        sim: u32,
    },
    /// Say no to what the resident asked to buy.
    DeclinePurchase {
        sim: u32,
    },
    /// Turn "free will" on or off for a household (default: the player's).
    SetAutonomy {
        enabled: bool,
        #[serde(default)]
        household: Option<u32>,
    },
    /// How fast residents age (`off`, `short`, `normal`, `long`).
    SetLifespan {
        lifespan: crate::lifecycle::Lifespan,
    },
    /// Adopt a baby (or a `child`) into the household, for content `life.adoption.cost`.
    Adopt {
        household: u32,
        #[serde(default)]
        child: bool,
    },
    /// Whether the player's residents move in with partners and out of home on their own.
    SetPlayerMoves {
        enabled: bool,
    },
    /// Switch a lamp on or off (see `lights.rs`): at once, nobody walks over.
    SwitchLamp {
        object: u32,
        on: bool,
    },
    /// Adds `amount` to a household's funds (a negative one takes money away, down to 0). For
    /// tests and long checks in the browser (`?debug`'s `__addFunds`); no menu sends it.
    AddFunds {
        household: u32,
        amount: i64,
    },
    /// Sets a resident's skill (content `skills` id) to `level`, within 0 and the most there is.
    /// For tests and checks in the browser (`check.mjs play --skills`, through `?debug`'s
    /// `__send`); no menu sends it.
    SetSkill {
        sim: u32,
        skill: String,
        level: f32,
    },
    /// Sets a resident's need (content `needs` id) to `value`, within 0 and 1, or every need with
    /// `need` "all". For tests and checks in the browser (`check.mjs play --needs`, through
    /// `?debug`'s `__send`); no menu sends it.
    SetNeed {
        sim: u32,
        need: String,
        value: f32,
    },
    /// Sets how residents `a` and `b` stand with each other, both ways: friendship and romance
    /// (-100..100), chemistry (-1..1), partners; what's left out stays. They've met. Only grown-ups
    /// who aren't family can be set in love or partners. For tests and checks in the browser
    /// (`check.mjs play --bond`); no menu sends it.
    SetBond {
        a: u32,
        b: u32,
        #[serde(default)]
        friendship: Option<f32>,
        #[serde(default)]
        romance: Option<f32>,
        #[serde(default)]
        chemistry: Option<f32>,
        #[serde(default)]
        partners: Option<bool>,
    },
    /// Queue an interaction on an object for a Sim.
    Use {
        sim: u32,
        object: u32,
        interaction: usize,
    },
    /// Do something with food or a dish (an item, see `items.rs`): eat a serving, or clear it
    /// away to the sink.
    UseItem {
        sim: u32,
        item: u32,
        action: ItemAction,
    },
    MoveTo {
        sim: u32,
        x: f32,
        z: f32,
    },
    /// Start a social interaction with another Sim.
    Social {
        sim: u32,
        target: u32,
        social: usize,
    },
    /// Take a job: a career and level (grade), replacing any current job.
    JoinCareer {
        sim: u32,
        career: usize,
        #[serde(default)]
        level: usize,
    },
    QuitCareer {
        sim: u32,
    },
    /// Walk over to another household's plot.
    Visit {
        sim: u32,
        plot: u32,
    },
    GoHome {
        sim: u32,
    },
    /// Use the phone: action `action` (index into content `phone.actions`), with `contact` if
    /// it's with someone (text, call, invite them over, ask to come over).
    Phone {
        sim: u32,
        action: usize,
        #[serde(default)]
        contact: Option<u32>,
    },
    /// Cancel an action: index 0 is the current action, 1.. are queued ones.
    Cancel {
        sim: u32,
        index: usize,
    },
    /// Buy an object for the household's home: at `at` (`[x, z, rot]`) or wherever it fits.
    Buy {
        household: u32,
        object: String,
        #[serde(default)]
        at: Option<[i32; 3]>,
        /// Its parts' colours (`crate::world::Colors`); absent: as it's made.
        #[serde(default)]
        colors: crate::world::Colors,
        /// Degrees past the facing, for objects that turn freely (0..90).
        #[serde(default)]
        turn: Option<u8>,
        /// Centimetres (x, z) off the middle of its footprint (up to `MAX_OFFSET` each way;
        /// things on a wall only along it).
        #[serde(default)]
        offset: Option<[i8; 2]>,
        /// Its own size: centimetres across and deep before turning (`MIN_SIZE` to `MAX_SIZE`
        /// tiles; it takes them rounded up to whole tiles); absent: its kind's.
        #[serde(default)]
        size: Option<[u16; 2]>,
        /// Centimetres taller (negative: shorter) than its kind.
        #[serde(default)]
        tall: Option<i16>,
    },
    /// Sell an object at home for part of what was paid.
    Sell {
        household: u32,
        object: u32,
    },
    /// Move, rotate or resize an object at home.
    MoveObject {
        household: u32,
        object: u32,
        x: i32,
        z: i32,
        rot: u8,
        /// Degrees past the facing (objects that turn freely); absent keeps the angle it had.
        #[serde(default)]
        turn: Option<u8>,
        /// Centimetres (x, z) off the middle of its footprint; absent keeps the offset it had.
        #[serde(default)]
        offset: Option<[i8; 2]>,
        /// Centimetres across and deep before turning; absent keeps the size it had.
        #[serde(default)]
        size: Option<[u16; 2]>,
        /// Centimetres taller than its kind; absent keeps the height it had.
        #[serde(default)]
        tall: Option<i16>,
    },
    /// Change an object's colours (free; effects stay the same): all of them at once, 0xRRGGBB by
    /// part (`crate::world::Colors`); a part left out, or all of them when absent, as it's made.
    Recolor {
        household: u32,
        object: u32,
        #[serde(default)]
        colors: crate::world::Colors,
    },
    /// Buy mode: buy the next quality level for an object at home (instant).
    Upgrade {
        household: u32,
        object: u32,
    },
    /// Build mode: change wall edges on the household's home plot.
    Build {
        household: u32,
        edits: Vec<EdgeEdit>,
    },
    /// Build mode: cover wall faces on the home plot (paint, wallpaper, brick...).
    Paint {
        household: u32,
        faces: Vec<FacePaint>,
    },
    /// Build mode: cover floor tiles indoors on the home plot (wood, tile, carpet...).
    PaintFloor {
        household: u32,
        tiles: Vec<FloorPaint>,
    },
    /// Buy mode: a paid quick fix for a worn or broken object at home (see `World::repair`).
    Repair {
        household: u32,
        object: u32,
    },
    /// Build and buy mode: take back the household's last edit (see `World::undo`).
    Undo {
        household: u32,
    },
    /// Build mode: the look of the roof over the home (a roof style and colour; free).
    SetRoof {
        household: u32,
        style: u8,
        color: u8,
    },
    /// Build and buy mode: make the household's last undone edit again (see `World::redo`).
    Redo {
        household: u32,
    },
    /// Build mode: builds a saved house on the household's empty lot (see `World::build_blueprint`).
    BuildBlueprint {
        household: u32,
        blueprint: crate::blueprint::Blueprint,
    },
    /// Build mode: moves the room around tile `(x, z)` of the home by `(dx, dz)` tiles, with its
    /// walls, doors, windows, floors and everything standing in it (see `World::move_room`).
    MoveRoom {
        household: u32,
        x: i32,
        z: i32,
        dx: i32,
        dz: i32,
    },
    /// New residents move into a household that has a home (an empty one, after building
    /// first): they arrive where `sims` stand (their `household` is ignored), with `bonds`
    /// between them (indices into `sims`). `name` renames the household.
    MoveIn {
        household: u32,
        #[serde(default)]
        name: Option<String>,
        sims: Vec<crate::lot::SimSpawn>,
        #[serde(default)]
        bonds: Vec<crate::lot::BondRaw>,
    },
}

/// One wall edge to set. `h` edges run along x at the line `z`; `v` edges run along z at the
/// line `x`; `dp` (`/`) and `dn` (`\`) are diagonal walls across tile `(x, z)`.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct EdgeEdit {
    pub axis: EdgeAxis,
    pub x: i32,
    pub z: i32,
    pub kind: EdgeKind,
    /// New walls: the covering of both faces (0 = the automatic look).
    #[serde(default)]
    pub cover: Option<u8>,
    /// Walls: 0 full height, 1 half wall. Changing it on a standing wall rebuilds it.
    #[serde(default)]
    pub form: Option<u8>,
    /// Doors and windows: their style. Changing it on a standing door or window replaces it.
    #[serde(default)]
    pub style: Option<u8>,
    /// Windows: the sill, and doors and windows: the top, in centimetres above the floor (see
    /// `MAX_HEAD`); absent keeps what a standing one has, else the style's own. Changing only
    /// these is free.
    #[serde(default)]
    pub sill: Option<u16>,
    #[serde(default)]
    pub head: Option<u16>,
}

/// The highest top (centimetres) a door or window may have: just under the ceiling.
pub const MAX_HEAD: u16 = 270;
/// The lowest top of a door (centimetres): people walk through it.
pub const MIN_DOOR_HEAD: u16 = 180;
/// The least height (centimetres) of a window, sill to top.
pub const MIN_WINDOW: u16 = 30;

impl EdgeEdit {
    /// An edit with the default look (full-height, automatic covering, first style).
    pub fn new(axis: EdgeAxis, x: i32, z: i32, kind: EdgeKind) -> Self {
        Self {
            axis,
            x,
            z,
            kind,
            cover: None,
            form: None,
            style: None,
            sill: None,
            head: None,
        }
    }
}

/// One wall face to cover: face 0 looks towards `-z` (`h`), `-x` (`v`) or the tile's half 0
/// (diagonals), face 1 the other way. `covering`: 0 the automatic look, else a wall covering + 1.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct FacePaint {
    pub axis: EdgeAxis,
    pub x: i32,
    pub z: i32,
    pub side: u8,
    pub covering: u8,
}

/// One floor tile to cover. `covering`: 0 the automatic look, else a floor covering + 1.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct FloorPaint {
    pub x: i32,
    pub z: i32,
    pub covering: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeAxis {
    H,
    V,
    /// Diagonal `/` across tile `(x, z)`: from corner `(x, z)` to `(x + 1, z + 1)`.
    Dp,
    /// Diagonal `\` across tile `(x, z)`: from corner `(x, z + 1)` to `(x + 1, z)`.
    Dn,
}

impl EdgeAxis {
    /// The diagonal direction, for `dp` / `dn`.
    pub fn diagonal(self) -> Option<crate::lot::DiagDir> {
        match self {
            EdgeAxis::Dp => Some(crate::lot::DiagDir::Dp),
            EdgeAxis::Dn => Some(crate::lot::DiagDir::Dn),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EdgeKind {
    Wall,
    /// A door in a wall (Sims walk through).
    Door,
    /// A window in a wall (Sims can't pass).
    Window,
    /// No wall: tears down a wall, door, window, fence or gate.
    Open,
    /// A garden fence (blocks walking, makes no rooms); `style` is a fence style.
    Fence,
    /// A gate (walked through); `style` is a fence style.
    Gate,
}

impl Command {
    /// The object (stable id) this command names, if it names one, to rewrite.
    pub(crate) fn object_mut(&mut self) -> Option<&mut u32> {
        match self {
            Command::SwitchLamp { object, .. }
            | Command::Use { object, .. }
            | Command::Sell { object, .. }
            | Command::MoveObject { object, .. }
            | Command::Recolor { object, .. }
            | Command::Upgrade { object, .. }
            | Command::Repair { object, .. } => Some(object),
            _ => None,
        }
    }

    /// The household whose home a build or buy edit changes (edits can be undone), if this is one.
    pub fn home_edit(&self) -> Option<u32> {
        match *self {
            Command::Buy { household, .. }
            | Command::Sell { household, .. }
            | Command::MoveObject { household, .. }
            | Command::Recolor { household, .. }
            | Command::Upgrade { household, .. }
            | Command::Build { household, .. }
            | Command::Paint { household, .. }
            | Command::PaintFloor { household, .. }
            | Command::SetRoof { household, .. }
            | Command::BuildBlueprint { household, .. }
            | Command::MoveRoom { household, .. }
            | Command::Repair { household, .. } => Some(household),
            _ => None,
        }
    }

    pub fn from_json(json: &str) -> Result<Self, crate::Error> {
        serde_json::from_str(json).map_err(|e| crate::Error::new(format!("invalid command: {e}")))
    }
}

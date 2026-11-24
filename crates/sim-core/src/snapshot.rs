//! Compact per-tick state for the renderer, written into a flat `f32` buffer.
//!
//! The layout is defined only here; the web side reads it via `layout_json()`,
//! so the two sides cannot drift apart.

use crate::Content;
use crate::world::{Activity, MAX_SIMS, Phase, Sim, Task, TaskKind, World};
use crate::{DRIVE_SCALE, TICKS_PER_MINUTE, TICKS_PER_SECOND, clock};

pub const HEADER_LEN: usize = 12;
/// Floats per Sim (all 32 used).
pub const SIM_STRIDE: usize = 36;
/// The cars' rows (`header::CAR_COUNT`, `car`) start here: after room for every resident's and
/// server's.
pub const CAR_BASE: usize = HEADER_LEN + (MAX_SIMS + crate::staff::MAX_STAFF) * SIM_STRIDE;
/// Floats per car.
pub const CAR_STRIDE: usize = 8;
/// The traffic lights' rows (`header::LIGHT_COUNT`, `light`), after the cars': one per signal
/// head, `MAX_LIGHTS` at most.
pub const LIGHT_BASE: usize = CAR_BASE + crate::cars::MAX_CARS * CAR_STRIDE;
/// Floats per signal head.
pub const LIGHT_STRIDE: usize = 5;
/// Signal heads the snapshot has room for.
pub const MAX_LIGHTS: usize = 512;
/// The cars' ways (`car::TRAIL`), after the traffic lights': `TRAIL_POINTS` points at most, each
/// `TRAIL_STRIDE` floats (`trail`).
pub const TRAIL_BASE: usize = LIGHT_BASE + MAX_LIGHTS * LIGHT_STRIDE;
pub const TRAIL_STRIDE: usize = 4;
/// The residents' rows, then the town's servers' (`header::STAFF_COUNT`, `staff.rs`), in the same
/// layout; then the cars', the traffic lights' and the cars' ways.
pub const CAPACITY: usize = TRAIL_BASE + TRAIL_POINTS * TRAIL_STRIDE;

pub mod header {
    /// Tick counter modulo 2^24 (exact in f32); only used to detect change.
    pub const TICK: usize = 0;
    pub const DAY: usize = 1;
    pub const MINUTE: usize = 2;
    pub const SPEED: usize = 3;
    pub const SIM_COUNT: usize = 4;
    pub const STRUCTURE_VERSION: usize = 5;
    /// Ticks a real step runs now (`World::step_rate`): the speed's, at super speed what the
    /// last step ran (as many as the machine had time for); 0 paused.
    pub const RATE: usize = 6;
    /// Id of the latest milestone event (importance 2) modulo 2^24, 0 if none yet.
    pub const MAJOR_EVENT: usize = 7;
    /// `Items::version` modulo 2^24: food and dishes changed (fetch `items` again).
    pub const ITEMS_VERSION: usize = 8;
    /// Servers at the town's stalls and counters (`staff.rs`): as many rows as this right after
    /// the residents' (`SIM_COUNT` of them), each laid out as a resident's (`sim`). A row's `ID`
    /// is its row; `AWAY` is 1 while they're off work (closed); `ACTION` is `service.anim`
    /// while they serve someone, with `HELD` what they hand over and `PROGRESS` how far the
    /// customer's use is; `OBJECT` is -1 (they stand behind it, not at it).
    pub const STAFF_COUNT: usize = 9;
    /// The cars (`cars.rs`): as many rows as this from `CAR_BASE`, each laid out as `car`. The
    /// households' first, in the structure's `cars` order; then the work taxis' (`taxi/`, `KIND`
    /// a taxi: none in the structure), a row each for good once one has come (an unused one not to
    /// be seen).
    pub const CAR_COUNT: usize = 10;
    /// The traffic lights' signal heads (`traffic::Signal`, one per arm of each lit junction): as
    /// many rows as this from `LIGHT_BASE`, each laid out as `light`, the same from tick to tick
    /// (lit junction by lit junction, arm by arm round it).
    pub const LIGHT_COUNT: usize = 11;
}

/// A signal head's row (`traffic::SignalArm`: one per arm of a lit junction, facing those coming
/// in along it).
pub mod light {
    /// 1: to be seen (a pole on the kerb with its head).
    pub const SHOWN: usize = 0;
    /// Where its pole stands on the ground, and which way the head faces (as a resident's yaw:
    /// towards those coming in).
    pub const X: usize = 1;
    pub const Z: usize = 2;
    pub const YAW: usize = 3;
    /// What it shows: 0 red, 1 amber, 2 green, 3 walk (red, those on foot crossing).
    pub const STATE: usize = 4;
    /// `STATE`'s values.
    pub const RED: f32 = 0.0;
    pub const AMBER: f32 = 1.0;
    pub const GREEN: f32 = 2.0;
    pub const WALK: f32 = 3.0;
}

/// A car's row.
pub mod car {
    /// 0: not to be seen (out of town, its spot built over, or out of sight: `cars::hidden`),
    /// 1 parked, 2 driving.
    pub const SHOWN: usize = 0;
    /// Its middle on the ground, and which way it faces (as a resident's yaw).
    pub const X: usize = 1;
    pub const Z: usize = 2;
    pub const YAW: usize = 3;
    /// Its open doors: bit `d` for door `d` (`cars::DOORS`: front left, front right, back left,
    /// back right), while someone gets in or out.
    pub const DOORS: usize = 4;
    /// What it is: `HOUSEHOLD` (its look in the structure's `cars`) or `TAXI` (a work taxi,
    /// `taxi/`: its look the renderer's own).
    pub const KIND: usize = 5;
    /// `KIND`'s values.
    pub const HOUSEHOLD: f32 = 0.0;
    pub const TAXI: f32 = 1.0;
    /// The way it went into this snapshot over the last steps (`ways.rs`): `TRAIL_LEN` points
    /// from point `TRAIL` of the ways (`TRAIL_BASE`), oldest first, each laid out as `trail`; its
    /// pose in this snapshot (`X`, `Z`, `YAW`) is the way's end. At a faster speed, where a step
    /// runs many ticks, the renderer draws the car along them, not on the straight line from the
    /// snapshot before. None while it isn't shown, or stood still all along.
    pub const TRAIL: usize = 6;
    pub const TRAIL_LEN: usize = 7;
}

/// A point of a car's way (`car::TRAIL`): when (`T`, in real steps before the snapshot: -1 is
/// the end of the step before it, below 0 always), where its middle was and which way it faced.
pub mod trail {
    pub const T: usize = 0;
    pub const X: usize = 1;
    pub const Z: usize = 2;
    pub const YAW: usize = 3;
}

pub mod sim {
    pub const ID: usize = 0;
    pub const X: usize = 1;
    pub const Z: usize = 2;
    pub const YAW: usize = 3;
    /// 0 = stand, 1 = sit, 2 = lie.
    pub const POSE: usize = 4;
    /// 1 while walking.
    pub const MOVING: usize = 5;
    /// Social interaction index + 1 while in a conversation (as actor, target or listener), else 0.
    pub const SOCIAL: usize = 6;
    /// 0 = undecided/hidden, 1 = went well, 2 = went badly (revealed halfway through; at once for
    /// a hug or a kiss, which the body answers).
    pub const OUTCOME: usize = 7;
    /// Conversation partner's id (for a listener, the one who started it), or -1.
    pub const PARTNER: usize = 8;
    /// Conversation animation code: index into `social::ANIMATIONS` + 1 ("talk", "laugh",
    /// "flirt", ...) while in a conversation, else 0. See `ACTION` for everything else.
    pub const ANIM: usize = 9;
    /// Dominant emotion index + 1, or 0.
    pub const EMOTION: usize = 10;
    /// 1 = started the conversation, 2 = was approached, 3 = joined in and listens (a group
    /// conversation; they don't speak), 0 = not talking.
    pub const ROLE: usize = 11;
    /// 1 while away from the map (at work).
    pub const AWAY: usize = 12;
    /// Stable id of the object instance the Sim is using or walking to use (the structure's
    /// object ids), or -1.
    pub const OBJECT: usize = 13;
    /// What the Sim is doing: index into `Content::animations` (layout `actions`), or -1.
    /// Set while using an object (not while walking to it) and in conversations ("talk").
    pub const ACTION: usize = 14;
    /// Mood 0..1 (needs, traits and feelings), for the resting face.
    pub const MOOD: usize = 15;
    /// What the Sim is thinking about (a thought bubble): 0 nothing, 1 skipping a planned
    /// block, 2 wants somewhere to do one, 3 kept one, 4 reached a goal, 5 loves the room,
    /// 6 dislikes it, 7 something broke, 8 had an accident, 9 a baby crying, 10 dinner's ready
    /// (`planner::ThoughtKind`).
    pub const THOUGHT: usize = 16;
    /// The thought's subject: activity index (1-3), goal definition (4), room
    /// (`rooms::room_subject`, 5-6), object type (7), accident (8), need (9) or recipe (10).
    pub const THOUGHT_SUBJECT: usize = 17;
    /// How high the Sim stands, in storeys: 0 on the ground, fractions on the stairs (`Z` is on
    /// the lot's rows, which hold every storey; see `storeys.rs`).
    pub const HEIGHT: usize = 18;
    /// Object definition (index into `Content::objects`) of the object in `OBJECT` while the
    /// Sim is using it or walking to use it, else -1 (repairs too). Voices pick lines per item
    /// from it; the instance id alone would need the world structure. While setting up something
    /// new (`setup.rs`), the type of the thing being set up, `OBJECT` being -1.
    pub const OBJECT_DEF: usize = 19;
    /// The interaction (index into that definition's `interactions`) with `OBJECT_DEF`, or -1.
    pub const INTERACTION: usize = 20;
    /// What they carry: index into `Content::items` + 1 (ingredients, a plate, dirty dishes),
    /// else 0.
    pub const HELD: usize = 21;
    /// The recipe they're cooking: index into `Content::recipes` + 1, else 0. While cooking,
    /// `OBJECT` is the station they're at (or heading to) and `ACTION` the step's animation.
    pub const RECIPE: usize = 22;
    /// Calling out someone's name (to talk; see `conversation/`): their id + 1, else 0.
    pub const CALL: usize = 23;
    /// While walking, the waypoint they're walking to (`YAW` faces it); else where they stand.
    pub const WAY_X: usize = 24;
    pub const WAY_Z: usize = 25;
    /// While walking, where their way is `AHEAD` metres on from where they are (or its end;
    /// not past a step onto another storey); else where they stand. From it the renderer opens
    /// a door or a gate only for someone about to go through it (`doors.ts` `goingThrough`).
    pub const AHEAD_X: usize = 26;
    pub const AHEAD_Z: usize = 27;
    /// The baby they hold in the arms (`carry::carrying`: lifting them out of the crib, or on
    /// the way with them to the high chair or back): their id + 1, else 0.
    pub const CARRY: usize = 28;
    /// How far through what they're doing at `OBJECT` they are, 0..1 (a use's minutes, a
    /// recipe step's), while at it; else -1. The renderer plays a clip along with it (the washing
    /// hung out piece by piece, a crop picked).
    pub const PROGRESS: usize = 29;
    /// What they eat while eating, on the table in front of them or in their hands: index into
    /// `Content::items` + 1, else 0. The renderer puts cutlery in the hands for a plate that wants it.
    pub const PLATE: usize = 30;
    /// 1 while the room they're in is kept shut on their account (a bathroom they're in, the
    /// bedroom they try for a baby in; `privacy.rs`), else 0: the renderer keeps its doors shut
    /// to anyone but whoever goes through them (nobody else goes in).
    pub const SHUTS: usize = 31;
    /// Stable id of the thing they ride (a bike, an e-scooter: `ride.rs`), drawn under them and
    /// not at its place while they're on it (away at work too), or -1. Left out by a place they're
    /// visiting (`Sim::bike_parked`): -2 less its stable id, not drawn at its place either.
    pub const RIDE: usize = 32;
    /// That bike left out (`RIDE` -2 less its stable id): where its middle stands and which way it
    /// faces (rad, as a placement's turn: `ride::park_spot`); else 0.
    pub const PARK_X: usize = 33;
    pub const PARK_Z: usize = 34;
    pub const PARK_YAW: usize = 35;
}

/// How far ahead along their way `sim::AHEAD_X` looks, in metres.
pub const AHEAD: f32 = 1.5;

// Every field fits in a Sim's row.
const _: () = assert!(
    sim::HEIGHT < SIM_STRIDE
        && sim::INTERACTION < SIM_STRIDE
        && sim::RECIPE < SIM_STRIDE
        && sim::CALL < SIM_STRIDE
        && sim::AHEAD_Z < SIM_STRIDE
        && sim::PROGRESS < SIM_STRIDE
        && sim::CARRY < SIM_STRIDE
        && sim::PLATE < SIM_STRIDE
        && sim::SHUTS < SIM_STRIDE
        && sim::PARK_YAW < SIM_STRIDE
);
const _: () = assert!(
    header::ITEMS_VERSION < HEADER_LEN
        && header::CAR_COUNT < HEADER_LEN
        && header::LIGHT_COUNT < HEADER_LEN
        && car::KIND < CAR_STRIDE
        && car::TRAIL_LEN < CAR_STRIDE
        && trail::YAW < TRAIL_STRIDE
        && light::STATE < LIGHT_STRIDE
);

/// What a Sim does for the snapshot; see `sim::OBJECT`, `sim::ACTION`, `sim::OBJECT_DEF` and
/// `sim::INTERACTION`.
pub(crate) struct Doing {
    pub(crate) object: f32,
    pub(crate) action: f32,
    pub(crate) def: f32,
    pub(crate) interaction: f32,
}

impl Doing {
    const NOTHING: Doing = Doing {
        object: -1.0,
        action: -1.0,
        def: -1.0,
        interaction: -1.0,
    };

    /// At an object (an index into `World::objects`; the renderer gets its stable id) or a spot
    /// without an interaction of its own.
    fn plain(world: &World, object: Option<u32>, anim: Option<usize>) -> Doing {
        let object = object
            .and_then(|o| world.stable_of(o))
            .map_or(-1.0, |s| s as f32);
        Doing {
            object,
            action: anim.map_or(-1.0, |a| a as f32),
            ..Doing::NOTHING
        }
    }
}

mod ways;
mod write;
pub(crate) use ways::Trails;
pub use ways::{TRAIL_MOST, TRAIL_POINTS, TRAIL_STEPS, TRAIL_TOLERANCE};
pub(crate) use write::doing;
pub use write::write;

/// The layout for the web side. `actions` (the content's animation tags) names the values
/// of `sim.action`.
pub fn layout_json(content: &Content) -> String {
    let mut layout = serde_json::json!({
        "capacity": CAPACITY,
        "headerLen": HEADER_LEN,
        "simStride": SIM_STRIDE,
        "maxSims": MAX_SIMS,
        "maxStaff": crate::staff::MAX_STAFF,
        "ticksPerSecond": TICKS_PER_SECOND,
        "header": {
            "tick": header::TICK, "day": header::DAY, "minute": header::MINUTE,
            "speed": header::SPEED, "simCount": header::SIM_COUNT,
            "structureVersion": header::STRUCTURE_VERSION,
            "rate": header::RATE, "majorEvent": header::MAJOR_EVENT,
            "itemsVersion": header::ITEMS_VERSION, "staffCount": header::STAFF_COUNT,
        },
        "sim": {
            "id": sim::ID, "x": sim::X, "z": sim::Z, "yaw": sim::YAW,
            "pose": sim::POSE, "moving": sim::MOVING, "social": sim::SOCIAL,
            "outcome": sim::OUTCOME, "partner": sim::PARTNER, "anim": sim::ANIM, "emotion": sim::EMOTION,
            "role": sim::ROLE, "away": sim::AWAY, "object": sim::OBJECT, "action": sim::ACTION,
            "mood": sim::MOOD, "thought": sim::THOUGHT, "thoughtSubject": sim::THOUGHT_SUBJECT, "height": sim::HEIGHT,
            "objectDef": sim::OBJECT_DEF, "interaction": sim::INTERACTION,
            "held": sim::HELD, "recipe": sim::RECIPE, "call": sim::CALL,
            "wayX": sim::WAY_X, "wayZ": sim::WAY_Z, "aheadX": sim::AHEAD_X, "aheadZ": sim::AHEAD_Z,
            "carry": sim::CARRY,
            "progress": sim::PROGRESS, "plate": sim::PLATE,
            "shuts": sim::SHUTS, "ride": sim::RIDE,
        },
        "actions": content.animations,
        "items": content.items.iter().map(|i| &i.id).collect::<Vec<_>>(),
        "recipes": content.recipes.iter().map(|r| &r.id).collect::<Vec<_>>(),
    });
    // (Apart: one `json!` this long would pass the macro's recursion limit.)
    // Ticks in a game minute, beside `ticksPerSecond` in a real second; and how many times as fast
    // as real time vehicles go (`DRIVE_SCALE`: the traffic's clock).
    layout["ticksPerMinute"] = TICKS_PER_MINUTE.into();
    layout["driveScale"] = DRIVE_SCALE.into();
    layout["sim"]["parkX"] = sim::PARK_X.into();
    layout["sim"]["parkZ"] = sim::PARK_Z.into();
    layout["sim"]["parkYaw"] = sim::PARK_YAW.into();
    layout["header"]["carCount"] = header::CAR_COUNT.into();
    layout["carBase"] = CAR_BASE.into();
    layout["carStride"] = CAR_STRIDE.into();
    layout["car"] = serde_json::json!({ "shown": car::SHOWN, "x": car::X, "z": car::Z, "yaw": car::YAW, "doors": car::DOORS, "kind": car::KIND, "kinds": { "household": car::HOUSEHOLD, "taxi": car::TAXI }, "trail": car::TRAIL, "trailLen": car::TRAIL_LEN });
    layout["trailBase"] = TRAIL_BASE.into();
    layout["trailStride"] = TRAIL_STRIDE.into();
    layout["trail"] =
        serde_json::json!({ "t": trail::T, "x": trail::X, "z": trail::Z, "yaw": trail::YAW });
    layout["header"]["lightCount"] = header::LIGHT_COUNT.into();
    layout["lightBase"] = LIGHT_BASE.into();
    layout["lightStride"] = LIGHT_STRIDE.into();
    layout["maxLights"] = MAX_LIGHTS.into();
    layout["light"] = serde_json::json!({
        "shown": light::SHOWN, "x": light::X, "z": light::Z, "yaw": light::YAW, "state": light::STATE,
        "states": { "red": light::RED, "amber": light::AMBER, "green": light::GREEN, "walk": light::WALK },
    });
    layout.to_string()
}

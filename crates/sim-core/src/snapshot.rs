//! Compact per-tick state for the renderer, written into a flat `f32` buffer.
//!
//! The layout is defined only here; the web side reads it via `layout_json()`,
//! so the two sides cannot drift apart.

use crate::Content;
use crate::world::{Activity, MAX_SIMS, Phase, Sim, Task, TaskKind, World};
use crate::{TICKS_PER_SECOND, clock};

pub const HEADER_LEN: usize = 9;
/// Floats per Sim (all 28 used).
pub const SIM_STRIDE: usize = 28;
pub const CAPACITY: usize = HEADER_LEN + MAX_SIMS * SIM_STRIDE;

pub mod header {
    /// Tick counter modulo 2^24 (exact in f32); only used to detect change.
    pub const TICK: usize = 0;
    pub const DAY: usize = 1;
    pub const MINUTE: usize = 2;
    pub const SPEED: usize = 3;
    pub const SIM_COUNT: usize = 4;
    pub const STRUCTURE_VERSION: usize = 5;
    /// 1 while time skips ahead because it's quiet at home (`World::calm`), else 0.
    pub const CALM: usize = 6;
    /// Id of the latest milestone event (importance 2) modulo 2^24, 0 if none yet.
    pub const MAJOR_EVENT: usize = 7;
    /// `Items::version` modulo 2^24: food and dishes changed (fetch `items` again).
    pub const ITEMS_VERSION: usize = 8;
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
    /// 0 = undecided/hidden, 1 = went well, 2 = went badly (revealed halfway through).
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
    /// from it; the instance id alone would need the world structure.
    pub const OBJECT_DEF: usize = 19;
    /// The interaction (index into that definition's `interactions`) with `OBJECT_DEF`, or -1.
    pub const INTERACTION: usize = 20;
    /// What they carry: index into `Content::items` + 1 (ingredients, a plate, dirty dishes),
    /// else 0.
    pub const HELD: usize = 21;
    /// The recipe they're cooking: index into `Content::recipes` + 1, else 0. While cooking,
    /// `OBJECT` is the station they're at (or heading to) and `ACTION` the step's animation.
    pub const RECIPE: usize = 22;
    /// Calling out someone's name (to talk; see `conversation.rs`): their id + 1, else 0.
    pub const CALL: usize = 23;
    /// While walking, the waypoint they're walking to (`YAW` faces it); else where they stand.
    pub const WAY_X: usize = 24;
    pub const WAY_Z: usize = 25;
    /// While walking, where their way is `AHEAD` metres on from where they are (or its end;
    /// not past a step onto another storey); else where they stand. With `WAY_X`, the renderer
    /// opens a door or a gate only for someone about to go through it.
    pub const AHEAD_X: usize = 26;
    pub const AHEAD_Z: usize = 27;
}

/// How far ahead along their way `sim::AHEAD_X` looks, in metres.
pub const AHEAD: f32 = 1.5;

// Every field fits in a Sim's row.
const _: () = assert!(sim::HEIGHT < SIM_STRIDE && sim::INTERACTION < SIM_STRIDE && sim::RECIPE < SIM_STRIDE && sim::CALL < SIM_STRIDE && sim::AHEAD_Z < SIM_STRIDE);
const _: () = assert!(header::ITEMS_VERSION < HEADER_LEN);

/// What a Sim does for the snapshot; see `sim::OBJECT`, `sim::ACTION`, `sim::OBJECT_DEF` and
/// `sim::INTERACTION`.
struct Doing {
    object: f32,
    action: f32,
    def: f32,
    interaction: f32,
}

impl Doing {
    const NOTHING: Doing = Doing { object: -1.0, action: -1.0, def: -1.0, interaction: -1.0 };

    /// At an object (an index into `World::objects`; the renderer gets its stable id) or a spot
    /// without an interaction of its own.
    fn plain(world: &World, object: Option<u32>, anim: Option<usize>) -> Doing {
        let object = object.and_then(|o| world.stable_of(o)).map_or(-1.0, |s| s as f32);
        Doing { object, action: anim.map_or(-1.0, |a| a as f32), ..Doing::NOTHING }
    }
}

fn doing(world: &World, s: &Sim) -> Doing {
    if s.away_until.is_some() {
        return Doing::NOTHING;
    }
    // Repairing: at the object, the tinker animation while at it.
    if let Some(Activity { task: Task { kind: TaskKind::Repair { object }, .. }, phase, .. }) = s.current() {
        let anim = world.content.object_rules.repair.anim.filter(|_| matches!(phase, Phase::Using { .. }));
        return Doing::plain(world, Some(*object), anim);
    }
    // An accident's while on the spot (asleep on the floor, takeout).
    if let Some(Activity { task: Task { kind: TaskKind::Spot { accident }, .. }, .. }) = s.current() {
        return Doing::plain(world, None, world.content.accidents[*accident].rest.as_ref().and_then(|r| r.anim));
    }
    // A visitor knocking at the door.
    if let Some(Activity { phase: Phase::Knocking { .. }, .. }) = s.current() {
        return Doing::plain(world, None, world.content.visits.door.as_ref().and_then(|d| d.knock_anim));
    }
    // Cooking: at the station, the step's animation for that station while at it (a meal put in
    // the oven, stirred on a hob; the recipe was started from an interaction of `origin`: that's
    // the interaction shown while at it).
    if let Some(Activity { task: Task { kind: TaskKind::Cook { recipe, step, station, origin, .. }, .. }, phase, .. }) = s.current() {
        let Some(o) = station.and_then(|o| world.objects.get(o as usize)) else {
            return Doing::NOTHING;
        };
        let r = &world.content.recipes[*recipe];
        let def = &world.content.objects[o.def];
        let using = matches!(phase, Phase::Using { .. });
        let interaction = (*origin == Some(o.id)).then(|| def.interactions.iter().position(|i| i.recipe == Some(*recipe))).flatten();
        return Doing {
            object: o.stable as f32,
            action: r.steps.get(*step as usize).and_then(|s| s.anim_for(def.stations)).filter(|_| using).map_or(-1.0, |a| a as f32),
            def: o.def as f32,
            interaction: interaction.map_or(-1.0, |i| i as f32),
        };
    }
    // Eating: at the seat (or standing), the eat animation while at it.
    if let Some(Activity { task: Task { kind: TaskKind::Eat { seat, .. }, .. }, phase, .. }) = s.current() {
        let anim = world.content.cooking.eat_anim.filter(|_| matches!(phase, Phase::Using { .. }));
        return Doing::plain(world, *seat, anim);
    }
    // Clearing away: at the bin, the sink or the counter, scraping, washing or putting away (the
    // station's kind too: washing at a dishwasher sounds unlike a sink's running tap).
    if let Some(Activity { task: Task { kind: TaskKind::Dishes { station, stage, .. }, .. }, phase, .. }) = s.current() {
        let rules = &world.content.cooking;
        let anim = match stage {
            1 => rules.scrape_anim,
            3 => rules.put_away_anim,
            _ => rules.dishes_anim,
        };
        let anim = anim.filter(|_| matches!(phase, Phase::Using { .. }));
        let def = station.and_then(|o| world.objects.get(o as usize)).map_or(-1.0, |o| o.def as f32);
        return Doing { def, ..Doing::plain(world, *station, anim) };
    }
    // On the phone: on the seat (or standing), the phone animation while at it.
    if let Some(Activity { task: Task { kind: TaskKind::Phone { seat, .. }, .. }, phase, .. }) = s.current() {
        let anim = world.content.phone.anim.filter(|_| matches!(phase, Phase::Using { .. }));
        return Doing::plain(world, *seat, anim);
    }
    // Tidying up: no object, the clean animation while at it.
    if let Some(Activity { task: Task { kind: TaskKind::Clean { .. }, .. }, phase, .. }) = s.current() {
        return Doing::plain(world, None, world.content.room_rules.clean.anim.filter(|_| matches!(phase, Phase::Using { .. })));
    }
    let Some(Activity {
        task:
            Task {
                kind:
                    TaskKind::Use {
                        object,
                        interaction,
                    },
                ..
            },
        phase,
        ..
    }) = s.current()
    else {
        return Doing::NOTHING;
    };
    let Some(obj) = world.objects.get(*object as usize) else {
        return Doing::NOTHING;
    };
    let inter = &world.content.objects[obj.def].interactions[*interaction];
    // A baby lying awake in the crib isn't shown asleep.
    let action = match phase {
        Phase::Using { .. } if inter.baby && s.awake_in_crib(&world.content, clock::hour(world.tick)) => None,
        Phase::Using { .. } => inter.anim,
        _ => None,
    };
    Doing {
        object: obj.stable as f32,
        action: action.map_or(-1.0, |a| a as f32),
        def: obj.def as f32,
        interaction: *interaction as f32,
    }
}

/// The waypoint a Sim walks to and where their way is `AHEAD` metres on (`sim::WAY_X`,
/// `sim::AHEAD_X`), both where they stand when they aren't walking.
fn way_ahead(world: &World, s: &Sim) -> ([f32; 2], [f32; 2]) {
    let Some(Activity { phase: Phase::Routing { waypoints, next }, .. }) = s.current() else {
        return (s.pos, s.pos);
    };
    let storey = world.storey_of(s.pos[1].floor() as i32);
    let Some(&way) = waypoints.get(*next).filter(|wp| world.storey_of(wp[1].floor() as i32) == storey) else {
        return (s.pos, s.pos);
    };
    let (mut at, mut left) = (s.pos, AHEAD);
    for &wp in &waypoints[*next..] {
        if world.storey_of(wp[1].floor() as i32) != storey {
            break;
        }
        let d = (wp[0] - at[0]).hypot(wp[1] - at[1]);
        if d >= left {
            let k = left / d;
            return (way, [at[0] + (wp[0] - at[0]) * k, at[1] + (wp[1] - at[1]) * k]);
        }
        left -= d;
        at = wp;
    }
    (way, at)
}

/// Writes the snapshot into `out`, which must hold at least `CAPACITY` floats.
pub fn write(world: &World, out: &mut [f32]) {
    out[header::TICK] = (world.tick % (1 << 24)) as f32;
    out[header::DAY] = clock::day(world.tick) as f32;
    out[header::MINUTE] = clock::minute_of_day(world.tick);
    out[header::SPEED] = world.speed as f32;
    out[header::SIM_COUNT] = world.sims.len() as f32;
    out[header::STRUCTURE_VERSION] = world.structure_version() as f32;
    out[header::CALM] = (world.auto_fast && world.speed > 0 && world.calm()) as u8 as f32;
    out[header::MAJOR_EVENT] = world
        .events
        .iter()
        .rev()
        .find(|e| e.kind.importance() >= 2)
        .map_or(0.0, |e| (e.id % (1 << 24)) as f32);
    out[header::ITEMS_VERSION] = (world.items.version % (1 << 24)) as f32;
    for (i, s) in world.sims.iter().enumerate() {
        let o = &mut out[HEADER_LEN + i * SIM_STRIDE..HEADER_LEN + (i + 1) * SIM_STRIDE];
        o[sim::ID] = s.id as f32;
        o[sim::X] = s.pos[0];
        o[sim::Z] = s.pos[1];
        o[sim::YAW] = s.yaw;
        o[sim::POSE] = s.pose.code();
        o[sim::MOVING] = s.is_moving() as u8 as f32;
        o[sim::SOCIAL] = 0.0;
        o[sim::OUTCOME] = 0.0;
        o[sim::PARTNER] = -1.0;
        o[sim::ANIM] = 0.0;
        o[sim::EMOTION] = s.emotion(&world.content).map_or(0.0, |e| e as f32 + 1.0);
        o[sim::ROLE] = 0.0;
        o[sim::AWAY] = (s.away_until.is_some() || !s.here()) as u8 as f32;
        let doing = doing(world, s);
        o[sim::OBJECT] = doing.object;
        o[sim::ACTION] = doing.action;
        o[sim::OBJECT_DEF] = doing.def;
        o[sim::INTERACTION] = doing.interaction;
        o[sim::MOOD] = s.mood(&world.content);
        let thought = s.planner.thought.filter(|_| s.away_until.is_none());
        o[sim::THOUGHT] = thought.map_or(0.0, |t| t.kind as u8 as f32);
        o[sim::THOUGHT_SUBJECT] = thought.map_or(0.0, |t| t.subject as f32);
        o[sim::HEIGHT] = world.sim_height(s);
        o[sim::HELD] = s.held.map_or(0.0, |h| h.def as f32 + 1.0);
        o[sim::RECIPE] = match s.current().map(|a| a.task.kind) {
            Some(TaskKind::Cook { recipe, .. }) => recipe as f32 + 1.0,
            _ => 0.0,
        };
        o[sim::CALL] = match s.current() {
            Some(Activity { task: Task { kind: TaskKind::Social { target, .. }, .. }, phase: Phase::Calling { .. }, .. }) => *target as f32 + 1.0,
            _ => 0.0,
        };
        let (way, ahead) = way_ahead(world, s);
        [o[sim::WAY_X], o[sim::WAY_Z]] = way;
        [o[sim::AHEAD_X], o[sim::AHEAD_Z]] = ahead;
    }
    // Conversations: write the shared state into every participant's row.
    for (i, s) in world.sims.iter().enumerate() {
        let Some((t, social, success, progress)) = s.conversation(&world.content) else {
            continue;
        };
        let outcome = if progress < 0.5 {
            0.0
        } else if success {
            1.0
        } else {
            2.0
        };
        let anim = world.content.socials[social].anim as f32;
        let talk = world.content.talk_anim.map_or(-1.0, |a| a as f32);
        // Those listening in (a group conversation) have the one who started it as partner.
        let listeners = world.sims.iter().filter(|l| l.listening_to() == Some(i as u32)).map(|l| (l.id as usize, i, 3.0));
        for (me, other, role) in [(i, t as usize, 1.0), (t as usize, i, 2.0)].into_iter().chain(listeners) {
            let o = &mut out[HEADER_LEN + me * SIM_STRIDE..HEADER_LEN + (me + 1) * SIM_STRIDE];
            o[sim::SOCIAL] = social as f32 + 1.0;
            o[sim::OUTCOME] = outcome;
            o[sim::PARTNER] = other as f32;
            o[sim::ANIM] = anim;
            o[sim::ROLE] = role;
            o[sim::ACTION] = talk;
        }
    }
}

/// The layout for the web side. `actions` (the content's animation tags) names the values
/// of `sim.action`.
pub fn layout_json(content: &Content) -> String {
    serde_json::json!({
        "capacity": CAPACITY,
        "headerLen": HEADER_LEN,
        "simStride": SIM_STRIDE,
        "maxSims": MAX_SIMS,
        "ticksPerSecond": TICKS_PER_SECOND,
        "header": {
            "tick": header::TICK, "day": header::DAY, "minute": header::MINUTE,
            "speed": header::SPEED, "simCount": header::SIM_COUNT,
            "structureVersion": header::STRUCTURE_VERSION,
            "calm": header::CALM, "majorEvent": header::MAJOR_EVENT,
            "itemsVersion": header::ITEMS_VERSION,
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
        },
        "actions": content.animations,
        "items": content.items.iter().map(|i| &i.id).collect::<Vec<_>>(),
        "recipes": content.recipes.iter().map(|r| &r.id).collect::<Vec<_>>(),
    })
    .to_string()
}

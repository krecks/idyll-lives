//! Compact per-tick state for the renderer, written into a flat `f32` buffer.
//!
//! The layout is defined only here; the web side reads it via `layout_json()`,
//! so the two sides cannot drift apart.

use crate::Content;
use crate::world::{Activity, MAX_SIMS, Phase, Sim, Task, TaskKind, World};
use crate::{TICKS_PER_SECOND, clock};

pub const HEADER_LEN: usize = 10;
/// Floats per Sim (all 32 used).
pub const SIM_STRIDE: usize = 32;
/// The residents' rows, then the town's servers' (`header::STAFF_COUNT`, `staff.rs`), in the same
/// layout.
pub const CAPACITY: usize = HEADER_LEN + (MAX_SIMS + crate::staff::MAX_STAFF) * SIM_STRIDE;

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
    /// Servers at the town's stalls and counters (`staff.rs`): as many rows as this right after
    /// the residents' (`SIM_COUNT` of them), each laid out as a resident's (`sim`). A row's `ID`
    /// is its row; `AWAY` is 1 while they're off work (closed); `ACTION` is `service.anim`
    /// while they serve someone, with `HELD` what they hand over and `PROGRESS` how far the
    /// customer's use is; `OBJECT` is -1 (they stand behind it, not at it).
    pub const STAFF_COUNT: usize = 9;
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
);
const _: () = assert!(header::ITEMS_VERSION < HEADER_LEN && header::STAFF_COUNT < HEADER_LEN);

/// What a Sim does for the snapshot; see `sim::OBJECT`, `sim::ACTION`, `sim::OBJECT_DEF` and
/// `sim::INTERACTION`.
struct Doing {
    object: f32,
    action: f32,
    def: f32,
    interaction: f32,
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

/// What `s` eats while at it (`sim::PLATE`): the plate on the table in front of them, else what they hold.
fn eating(world: &World, s: &Sim) -> Option<usize> {
    let Some(Activity {
        task: Task {
            kind: TaskKind::Eat { plate, .. },
            ..
        },
        phase: Phase::Using { .. },
        ..
    }) = s.current()
    else {
        return None;
    };
    plate
        .and_then(|p| world.items.get(p))
        .map(|it| it.def)
        .or(s.held.map(|h| h.def))
}

fn doing(world: &World, s: &Sim) -> Doing {
    if s.away_until.is_some() {
        return Doing::NOTHING;
    }
    // Repairing: at the object, the tinker animation while at it.
    if let Some(Activity {
        task: Task {
            kind: TaskKind::Repair { object },
            ..
        },
        phase,
        ..
    }) = s.current()
    {
        let anim = world
            .content
            .object_rules
            .repair
            .anim
            .filter(|_| matches!(phase, Phase::Using { .. }));
        return Doing::plain(world, Some(*object), anim);
    }
    // An accident's while on the spot (asleep on the floor, takeout).
    if let Some(Activity {
        task: Task {
            kind: TaskKind::Spot { accident },
            ..
        },
        ..
    }) = s.current()
    {
        return Doing::plain(
            world,
            None,
            world.content.accidents[*accident]
                .rest
                .as_ref()
                .and_then(|r| r.anim),
        );
    }
    // A visitor knocking at the door.
    if let Some(Activity {
        phase: Phase::Knocking { .. },
        ..
    }) = s.current()
    {
        return Doing::plain(
            world,
            None,
            world
                .content
                .visits
                .door
                .as_ref()
                .and_then(|d| d.knock_anim),
        );
    }
    // Cooking: at the station, the step's animation for that station while at it (a meal put in
    // the oven, stirred on a hob; at the table, the dish put down); at `origin`, the object the
    // recipe was started from, its interaction is shown for the last step there.
    if let Some(Activity {
        task:
            Task {
                kind:
                    TaskKind::Cook {
                        recipe,
                        step,
                        station,
                        origin,
                        ..
                    },
                ..
            },
        phase,
        ..
    }) = s.current()
    {
        let Some(o) = station.and_then(|o| world.objects.get(o as usize)) else {
            return Doing::NOTHING;
        };
        let r = &world.content.recipes[*recipe];
        let def = &world.content.objects[o.def];
        let using = matches!(phase, Phase::Using { .. });
        // (The recipe's own interaction names its last step, the cooking itself: a grill station
        // lifts its hood to grill, not to chop on its counter first.)
        let last = *step as usize + 1 >= r.steps.len();
        let interaction = (*origin == Some(o.id) && last)
            .then(|| {
                def.interactions
                    .iter()
                    .position(|i| i.recipe == Some(*recipe))
            })
            .flatten();
        return Doing {
            object: o.stable as f32,
            // (Past the last step: putting the dish down on the table.)
            action: match r.steps.get(*step as usize) {
                Some(s) => s.anim_for(def.stations),
                None => world.content.cooking.serve_anim,
            }
            .filter(|_| using)
            .map_or(-1.0, |a| a as f32),
            def: o.def as f32,
            interaction: interaction.map_or(-1.0, |i| i as f32),
        };
    }
    // Eating: at the seat (or standing), the eat animation while at it.
    if let Some(Activity {
        task: Task {
            kind: TaskKind::Eat { seat, .. },
            ..
        },
        phase,
        ..
    }) = s.current()
    {
        let anim = world
            .content
            .cooking
            .eat_anim
            .filter(|_| matches!(phase, Phase::Using { .. }));
        return Doing::plain(world, *seat, anim);
    }
    // Carrying a baby (at the crib lifting them out, on the way to where they go, putting them
    // in: shown as the cuddle's lift, `sim::CARRY` says who) or a harvest to the fridge (putting
    // it away there).
    if let Some(Activity {
        task:
            Task {
                kind:
                    TaskKind::Carry {
                        baby,
                        from,
                        to,
                        stage,
                    },
                ..
            },
        phase,
        ..
    }) = s.current()
    {
        let at = if *stage == 0 { *from } else { *to };
        let using = matches!(phase, Phase::Using { .. });
        let anim = match baby {
            Some(_) => world.content.cuddle_anim,
            None => world.content.cooking.put_away_anim,
        }
        .filter(|_| using);
        let mut d = Doing::plain(world, Some(at), anim);
        d.def = world
            .objects
            .get(at as usize)
            .map_or(-1.0, |o| o.def as f32);
        return d;
    }
    // Clearing away: at the bin, the sink or the counter, scraping, washing or putting away (the
    // station's kind too: washing at a dishwasher sounds unlike a sink's running tap).
    if let Some(Activity {
        task:
            Task {
                kind: TaskKind::Dishes { station, stage, .. },
                ..
            },
        phase,
        ..
    }) = s.current()
    {
        let rules = &world.content.cooking;
        let anim = match stage {
            1 => rules.scrape_anim,
            3 => rules.put_away_anim,
            _ => rules.dishes_anim,
        };
        let anim = anim.filter(|_| matches!(phase, Phase::Using { .. }));
        let def = station
            .and_then(|o| world.objects.get(o as usize))
            .map_or(-1.0, |o| o.def as f32);
        return Doing {
            def,
            ..Doing::plain(world, *station, anim)
        };
    }
    // On the phone: on the seat (or standing), the action's animation while at it.
    if let Some(Activity {
        task:
            Task {
                kind: TaskKind::Phone { seat, action, .. },
                ..
            },
        phase,
        ..
    }) = s.current()
    {
        let phone = &world.content.phone;
        let anim = phone
            .actions
            .get(*action)
            .map_or(phone.anim, |a| a.anim)
            .filter(|_| matches!(phase, Phase::Using { .. }));
        return Doing::plain(world, *seat, anim);
    }
    // Tidying up: no object, the clean animation while at it.
    if let Some(Activity {
        task: Task {
            kind: TaskKind::Clean { .. },
            ..
        },
        phase,
        ..
    }) = s.current()
    {
        return Doing::plain(
            world,
            None,
            world
                .content
                .room_rules
                .clean
                .anim
                .filter(|_| matches!(phase, Phase::Using { .. })),
        );
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
    // A baby lying awake in the crib isn't shown asleep; nor anyone waiting at a game for two for
    // the one they play with as playing (`partners::hold`: a seesaw doesn't rock with one).
    // At a post: waiting for its server, standing; where nobody serves, helping themselves
    // (`staff.rs`).
    let server = inter
        .served
        .as_ref()
        .and_then(|_| world.staff.server_of(*object));
    let action = match phase {
        Phase::Using { .. }
            if inter.baby_bed() && s.awake_in_crib(&world.content, clock::hour(world.tick)) =>
        {
            None
        }
        Phase::Using { elapsed } if inter.partner && (s.partner_wait > 0 || *elapsed == 0.0) => {
            None
        }
        Phase::Using { .. } if server.is_some() && !s.being_served => None,
        Phase::Using { .. } if server.is_none() && inter.served.is_some() => {
            inter.served.as_ref().and_then(|f| f.anim).or(inter.anim)
        }
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

/// How far through a use or a recipe's step a Sim at it is (`sim::PROGRESS`), else -1.
fn progress(world: &World, s: &Sim) -> f32 {
    if s.away_until.is_some() {
        return -1.0;
    }
    let Some(Activity {
        task,
        phase: Phase::Using { elapsed },
        ..
    }) = s.current()
    else {
        return -1.0;
    };
    let minutes = match task.kind {
        TaskKind::Use {
            object,
            interaction,
        } => world
            .objects
            .get(object as usize)
            .and_then(|o| world.content.objects[o.def].interactions.get(interaction))
            .map_or(0.0, |i| {
                i.minutes_where(world.staff.server_of(object).is_some())
            }),
        TaskKind::Cook { minutes, .. } => f32::from(minutes),
        _ => return -1.0,
    };
    if minutes > 0.0 {
        (elapsed / minutes).clamp(0.0, 1.0)
    } else {
        1.0
    }
}

/// The waypoint a Sim walks to and where their way is `AHEAD` metres on (`sim::WAY_X`,
/// `sim::AHEAD_X`), both where they stand when they aren't walking (waiting outside a room kept
/// shut too: its door doesn't open for them).
fn way_ahead(world: &World, s: &Sim) -> ([f32; 2], [f32; 2]) {
    let Some(Activity {
        phase: Phase::Routing { waypoints, next },
        ..
    }) = s.current()
    else {
        return (s.pos, s.pos);
    };
    if s.kept_out() {
        return (s.pos, s.pos);
    }
    let storey = world.storey_of(s.pos[1].floor() as i32);
    let Some(&way) = waypoints
        .get(*next)
        .filter(|wp| world.storey_of(wp[1].floor() as i32) == storey)
    else {
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
            return (
                way,
                [at[0] + (wp[0] - at[0]) * k, at[1] + (wp[1] - at[1]) * k],
            );
        }
        left -= d;
        at = wp;
    }
    (way, at)
}

/// What a resident carries for a use without having it (content `brings`, `holds`; an item
/// index): the basket of washing in the arms on the way to the dryer, a shirt in the hands while
/// folding. Only shown.
fn shown_held(world: &World, s: &crate::world::Sim) -> Option<usize> {
    let a = s.current()?;
    let TaskKind::Use {
        object,
        interaction,
    } = a.task.kind
    else {
        return None;
    };
    let o = world.objects.get(object as usize)?;
    let inter = world.content.objects[o.def].interactions.get(interaction)?;
    match a.phase {
        Phase::Routing { .. } => inter.brings,
        Phase::Using { .. } => inter.holds,
        _ => None,
    }
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
    let staff = &world.staff.servers[..world.staff.servers.len().min(crate::staff::MAX_STAFF)];
    out[header::STAFF_COUNT] = staff.len() as f32;
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
        // Being served: what they're handed (the renderer shows it once it's theirs).
        o[sim::HELD] = s
            .held
            .map(|h| h.def)
            .or_else(|| shown_held(world, s))
            .or_else(|| serving(world, s))
            .map_or(0.0, |d| d as f32 + 1.0);
        o[sim::RECIPE] = match s.current().map(|a| a.task.kind) {
            Some(TaskKind::Cook { recipe, .. }) => recipe as f32 + 1.0,
            _ => 0.0,
        };
        o[sim::CALL] = match s.current() {
            Some(Activity {
                task:
                    Task {
                        kind: TaskKind::Social { target, .. },
                        ..
                    },
                phase: Phase::Calling { .. },
                ..
            }) => *target as f32 + 1.0,
            _ => 0.0,
        };
        let (way, ahead) = way_ahead(world, s);
        [o[sim::WAY_X], o[sim::WAY_Z]] = way;
        [o[sim::AHEAD_X], o[sim::AHEAD_Z]] = ahead;
        o[sim::CARRY] = crate::carry::carrying(s).map_or(0.0, |b| b as f32 + 1.0);
        o[sim::PROGRESS] = progress(world, s);
        // (Or what a use holds in the hands, a slice of cake at the café: eaten with a fork.)
        o[sim::PLATE] = eating(world, s)
            .or_else(|| shown_held(world, s))
            .map_or(0.0, |d| d as f32 + 1.0);
        o[sim::SHUTS] = crate::privacy::keeps_shut(world, s.id) as u8 as f32;
    }
    // Conversations: write the shared state into every participant's row.
    for (i, s) in world.sims.iter().enumerate() {
        let Some((t, social, success, progress)) = s.conversation(&world.content) else {
            continue;
        };
        // A hug or a kiss is answered at once, by the body (a refused one is leant away from); talk
        // halfway through.
        let def = &world.content.socials[social];
        let reveal = if matches!(
            crate::social::ANIMATIONS.get(def.anim as usize - 1),
            Some(&("hug" | "kiss"))
        ) {
            0.0
        } else {
            0.5
        };
        let outcome = if progress < reveal {
            0.0
        } else if success {
            1.0
        } else {
            2.0
        };
        let anim = def.anim as f32;
        let talk = world.content.talk_anim.map_or(-1.0, |a| a as f32);
        // Those listening in (a group conversation) have the one who started it as partner.
        let listeners = world
            .sims
            .iter()
            .filter(|l| l.listening_to() == Some(i as u32))
            .map(|l| (l.id as usize, i, 3.0));
        for (me, other, role) in [(i, t as usize, 1.0), (t as usize, i, 2.0)]
            .into_iter()
            .chain(listeners)
        {
            let o = &mut out[HEADER_LEN + me * SIM_STRIDE..HEADER_LEN + (me + 1) * SIM_STRIDE];
            o[sim::SOCIAL] = social as f32 + 1.0;
            o[sim::OUTCOME] = outcome;
            o[sim::PARTNER] = other as f32;
            o[sim::ANIM] = anim;
            o[sim::ROLE] = role;
            o[sim::ACTION] = talk;
        }
    }
    write_staff(world, out);
}

/// The servers' rows, after the residents' (`header::STAFF_COUNT`).
fn write_staff(world: &World, out: &mut [f32]) {
    let first = world.sims.len();
    for (k, server) in world
        .staff
        .servers
        .iter()
        .take(crate::staff::MAX_STAFF)
        .enumerate()
    {
        let row = first + k;
        let o = &mut out[HEADER_LEN + row * SIM_STRIDE..HEADER_LEN + (row + 1) * SIM_STRIDE];
        o.fill(0.0);
        o[sim::ID] = row as f32;
        o[sim::X] = server.pos[0];
        o[sim::Z] = server.pos[1];
        o[sim::YAW] = server.yaw;
        o[sim::MOVING] = server.moving as u8 as f32;
        o[sim::PARTNER] = -1.0;
        o[sim::AWAY] = (!server.present) as u8 as f32;
        o[sim::OBJECT] = -1.0;
        o[sim::OBJECT_DEF] = -1.0;
        o[sim::INTERACTION] = -1.0;
        o[sim::ACTION] = -1.0;
        o[sim::PROGRESS] = -1.0;
        o[sim::MOOD] = 0.75;
        [o[sim::WAY_X], o[sim::WAY_Z]] = server.pos;
        [o[sim::AHEAD_X], o[sim::AHEAD_Z]] = server.pos;
        // Serving: the customer's use as it goes, and what they hand over.
        if let Some(c) = server.serving.and_then(|c| world.sims.get(c as usize))
            && let Some(inter) = served_use(world, c)
        {
            o[sim::ACTION] = world.content.service.anim.map_or(-1.0, |a| a as f32);
            o[sim::PROGRESS] = progress(world, c);
            o[sim::HELD] =
                crate::staff::handed(&world.content, inter).map_or(0.0, |i| i as f32 + 1.0);
        }
    }
}

/// The use a resident is being served at a post (`staff.rs`), if they are.
fn served_use<'a>(world: &'a World, s: &Sim) -> Option<&'a crate::content::Interaction> {
    if !s.being_served {
        return None;
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
        phase: Phase::Using { .. },
        ..
    }) = s.current()
    else {
        return None;
    };
    let o = world.objects.get(*object as usize)?;
    world.content.objects[o.def]
        .interactions
        .get(*interaction)
        .filter(|i| i.served.is_some())
}

/// What a resident being served at a post is handed (`staff::handed`).
fn serving(world: &World, s: &Sim) -> Option<usize> {
    crate::staff::handed(&world.content, served_use(world, s)?)
}

/// The layout for the web side. `actions` (the content's animation tags) names the values
/// of `sim.action`.
pub fn layout_json(content: &Content) -> String {
    serde_json::json!({
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
            "calm": header::CALM, "majorEvent": header::MAJOR_EVENT,
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
            "shuts": sim::SHUTS,
        },
        "actions": content.animations,
        "items": content.items.iter().map(|i| &i.id).collect::<Vec<_>>(),
        "recipes": content.recipes.iter().map(|r| &r.id).collect::<Vec<_>>(),
    })
    .to_string()
}

//! What looks wrong in an object's uses: the flags and their medians.

use super::*;

/// What looks wrong in an object's uses (`cargo review uses`).
#[derive(Debug, Clone, PartialEq)]
pub enum Flag {
    /// No use at all, it isn't a lamp, and its kind isn't decor.
    NoUse,
    /// A use's animation that `ACTIONS` has no entry for (the resident stands idle at it).
    NoAction { using: String, anim: String },
    /// A `motions.uses` or `motions.loops` key that is neither a use's id nor an animation tag of the object.
    MotionKey { key: String },
    /// A use that opens or closes the object (`when`, `leaves`) on a model without `motions.open`.
    NoOpen { using: String },
    /// A use's minutes more than `DURATION_SPREAD` times from the median of the uses like it (`like_key`).
    Duration {
        using: String,
        minutes: f32,
        like: String,
        median: f32,
    },
    /// A use's sound that every object using it is of one other kind (a sizzle away from a hob).
    OddSound {
        using: String,
        sound: String,
        kind: String,
    },
    /// A `how` whose manner `MANNERS` has no entry for.
    Manner { tag: String, manner: String },
    /// A use that plays a `motions.uses` or `loops` clip but sounds of nothing, and isn't `"none"` by choice.
    Silent { using: String },
    /// Over a window (`overWindow`), shut at dusk with neither `motions.night` nor `sounds.shut`.
    NoShut,
    /// A seated use whose `sit` is the desk's hands (`sitHands`) on a thing with no top for them (`Object::top`).
    DeskHands { using: String, anim: String },
}

/// The baseline's sections (tools/art/gate-baseline.json) `cargo review uses --check` ratchets, by object id,
/// in `Flag::section`'s order.
pub const USES_CHECKS: [&str; 4] = ["uses.manner", "uses.sound", "uses.shut", "uses.hands"];

impl Flag {
    /// The column it is counted in.
    pub fn column(&self) -> &'static str {
        match self {
            Flag::NoUse => "no use",
            Flag::NoAction { .. } => "anim not in ACTIONS",
            Flag::MotionKey { .. } => "motions key",
            Flag::NoOpen { .. } => "opens, no motions.open",
            Flag::Duration { .. } => "duration",
            Flag::OddSound { .. } => "odd sound",
            Flag::Manner { .. } => "how not in MANNERS",
            Flag::Silent { .. } => "clip, no sound",
            Flag::NoShut => "overWindow, no shut",
            Flag::DeskHands { .. } => "sat, desk hands",
        }
    }

    /// The baseline section it is counted in (`USES_CHECKS`): it came into the gate after the objects it finds.
    pub fn section(&self) -> Option<&'static str> {
        match self {
            Flag::Manner { .. } => Some(USES_CHECKS[0]),
            Flag::Silent { .. } => Some(USES_CHECKS[1]),
            Flag::NoShut => Some(USES_CHECKS[2]),
            Flag::DeskHands { .. } => Some(USES_CHECKS[3]),
            _ => None,
        }
    }

    /// Whether the gate fails on it: a use the renderer can't play, a `motions` key no use answers to,
    /// or a model that opens without a clip. A missing use, a duration and a sound are the reviewer's to
    /// weigh — a table with no use of its own is still a table the sim uses as a station. The counted ones
    /// (`section`) fail by the baseline instead.
    pub fn hard(&self) -> bool {
        !matches!(
            self,
            Flag::NoUse | Flag::Duration { .. } | Flag::OddSound { .. }
        ) && self.section().is_none()
    }

    pub fn text(&self) -> String {
        match self {
            Flag::NoUse => "no use".into(),
            Flag::NoAction { using, anim } => {
                format!("{using}: anim `{anim}` has no `ACTIONS` entry")
            }
            Flag::MotionKey { key } => format!("`motions` key `{key}` is no use or anim of it"),
            Flag::NoOpen { using } => {
                format!("{using} opens or closes it, but its model has no `motions.open`")
            }
            Flag::Duration {
                using,
                minutes,
                like,
                median,
            } => format!("{using}: {minutes} min, the median for `{like}` {median} min"),
            Flag::OddSound { using, sound, kind } => {
                format!("{using}: sounds `{sound}`, which only a {kind} sounds")
            }
            Flag::Manner { tag, manner } => {
                format!("`how.{tag}` is `{manner}`, which `MANNERS` has no entry for")
            }
            Flag::Silent { using } => format!(
                "{using}: plays a `motions` clip but sounds of nothing (a `sound`, or `\"none\"` for silence)"
            ),
            Flag::NoShut => {
                "over a window, but shuts at dusk with no `motions.night` and no `sounds.shut`"
                    .into()
            }
            Flag::DeskHands { using, anim } => format!(
                "{using}: sat at `{anim}` with the desk's hands (`sitHands`), but it has no top for them (`surface`, `seatAt`, `layer: surface`)"
            ),
        }
    }
}

/// The columns of `Flag::column`, in order.
pub const FLAG_COLUMNS: [&str; 10] = [
    "no use",
    "anim not in ACTIONS",
    "motions key",
    "opens, no motions.open",
    "duration",
    "odd sound",
    "how not in MANNERS",
    "clip, no sound",
    "overWindow, no shut",
    "sat, desk hands",
];

/// What a use's duration is compared with: the uses with the same animation (a night's sleep with sleeps, a nap
/// with naps; a first tag lumps them together), else with the same first tag.
pub fn like_key(u: &Use) -> Option<&str> {
    if u.anim.is_empty() {
        u.tags.first().map(String::as_str)
    } else {
        Some(u.anim.as_str())
    }
}

/// The median minutes of the uses of each `like_key`, over `objects`.
pub fn tag_medians(objects: &[Object]) -> BTreeMap<String, f32> {
    let mut by_like: BTreeMap<String, Vec<f32>> = BTreeMap::new();
    for u in objects.iter().flat_map(|o| &o.uses) {
        if let Some(like) = like_key(u) {
            by_like.entry(like.to_string()).or_default().push(u.minutes);
        }
    }
    by_like
        .into_iter()
        .map(|(like, mut m)| {
            m.sort_by(f32::total_cmp);
            let n = m.len();
            (
                like,
                if n % 2 == 1 {
                    m[n / 2]
                } else {
                    (m[n / 2 - 1] + m[n / 2]) / 2.0
                },
            )
        })
        .collect()
}

/// What looks wrong in `object`'s uses: `motions` is its model's (None: none), `renderer` the renderer's tables, `medians` the minutes per `like_key` (`tag_medians`), and `decor` whether its kind expects no use.
/// What a use's sound is weighed against: the tag map (`ACTION_SOUNDS`), the sounds only one kind makes
/// (`sound_owners`) and this object's kind.
pub struct Sounds<'a> {
    pub tag: &'a BTreeMap<String, String>,
    pub owners: &'a BTreeMap<String, String>,
    pub kind: Option<&'a str>,
}

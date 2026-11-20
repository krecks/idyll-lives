//! What looks wrong in an object's uses: the flags and their medians.
//!
//! Split from `review.rs` (see `mod.rs`).

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
}

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
        }
    }

    /// Whether the gate fails on it: a use the renderer can't play, a `motions` key no use answers to,
    /// or a model that opens without a clip. A missing use, a duration and a sound are the reviewer's to
    /// weigh — a table with no use of its own is still a table the sim uses as a station.
    pub fn hard(&self) -> bool {
        !matches!(
            self,
            Flag::NoUse | Flag::Duration { .. } | Flag::OddSound { .. }
        )
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
        }
    }
}

/// The columns of `Flag::column`, in order.
pub const FLAG_COLUMNS: [&str; 6] = [
    "no use",
    "anim not in ACTIONS",
    "motions key",
    "opens, no motions.open",
    "duration",
    "odd sound",
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

/// What looks wrong in `object`'s uses: `motions` is its model's (None: none), `actions` the renderer's action
/// tags, `medians` the minutes per `like_key` (`tag_medians`), and `decor` whether its kind expects no use.
/// What a use's sound is weighed against: the tag map (`ACTION_SOUNDS`), the sounds only one kind makes
/// (`sound_owners`) and this object's kind.
pub struct Sounds<'a> {
    pub tag: &'a BTreeMap<String, String>,
    pub owners: &'a BTreeMap<String, String>,
    pub kind: Option<&'a str>,
}

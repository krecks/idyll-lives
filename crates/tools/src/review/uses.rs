//! The uses of every object, and who owns each sound.

use super::*;

pub fn use_flags(
    object: &Object,
    motions: Option<&Value>,
    actions: &BTreeSet<String>,
    medians: &BTreeMap<String, f32>,
    decor: bool,
    heard: &Sounds,
) -> Vec<Flag> {
    let mut out = Vec::new();
    if object.uses.is_empty() && !object.lamp && !decor {
        out.push(Flag::NoUse);
    }
    for u in &object.uses {
        if !u.anim.is_empty() && !actions.contains(&u.anim) {
            out.push(Flag::NoAction {
                using: u.id.clone(),
                anim: u.anim.clone(),
            });
        }
    }
    let named: BTreeSet<&str> = object
        .uses
        .iter()
        .flat_map(|u| [u.id.as_str(), u.anim.as_str()])
        .chain(object.station_anims.iter().map(String::as_str))
        .filter(|k| !k.is_empty())
        .collect();
    for part in ["uses", "loops"] {
        for key in motions
            .and_then(|m| m[part].as_object())
            .into_iter()
            .flat_map(|m| m.keys())
        {
            if !named.contains(key.as_str()) {
                out.push(Flag::MotionKey { key: key.clone() });
            }
        }
    }
    let opens = motions.is_some_and(|m| m.get("open").is_some_and(|o| !o.is_null()));
    for u in &object.uses {
        let open_or_closed = |s: &Option<String>| matches!(s.as_deref(), Some("open" | "closed"));
        if (open_or_closed(&u.when) || open_or_closed(&u.leaves)) && !opens {
            out.push(Flag::NoOpen {
                using: u.id.clone(),
            });
        }
    }
    for u in &object.uses {
        let Some(like) = like_key(u) else { continue };
        let Some(&median) = medians.get(like) else {
            continue;
        };
        if median > 0.0
            && (u.minutes > median * DURATION_SPREAD || u.minutes * DURATION_SPREAD < median)
        {
            out.push(Flag::Duration {
                using: u.id.clone(),
                minutes: u.minutes,
                like: like.to_string(),
                median,
            });
        }
    }
    if let Some(kind) = heard.kind {
        for u in &object.uses {
            let Some(sound) = sound_of(heard.tag, object, u) else {
                continue;
            };
            if let Some(owns) = heard.owners.get(&sound)
                && owns != kind
            {
                out.push(Flag::OddSound {
                    using: u.id.clone(),
                    sound,
                    kind: owns.clone(),
                });
            }
        }
    }
    out
}

/// The sounds only one kind of object makes: `sound` → that kind, for every sound `SOUND_OWNERS` objects of one
/// kind make and nothing else does. A sizzle belongs to the hobs; a toaster that sizzles is flagged.
pub const SOUND_OWNERS: usize = 3;

pub fn sound_owners(
    objects: &[Object],
    sounds: &BTreeMap<String, String>,
    kind_of: &dyn Fn(&Object) -> Option<String>,
) -> BTreeMap<String, String> {
    let mut by_sound: BTreeMap<String, (BTreeSet<String>, usize)> = BTreeMap::new();
    for o in objects {
        let Some(kind) = kind_of(o) else { continue };
        for u in &o.uses {
            let Some(sound) = sound_of(sounds, o, u) else {
                continue;
            };
            let entry = by_sound.entry(sound).or_default();
            entry.0.insert(kind.clone());
            entry.1 += 1;
        }
    }
    by_sound
        .into_iter()
        .filter(|(_, (kinds, n))| *n >= SOUND_OWNERS && kinds.len() == 1)
        .map(|(sound, (kinds, _))| (sound, kinds.into_iter().next().unwrap_or_default()))
        .collect()
}

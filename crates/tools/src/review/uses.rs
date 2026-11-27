//! The uses of every object, and who owns each sound.

use super::*;

pub fn use_flags(
    object: &Object,
    motions: Option<&Value>,
    renderer: &Renderer,
    medians: &BTreeMap<String, f32>,
    decor: bool,
    heard: &Sounds,
) -> Vec<Flag> {
    let mut out = Vec::new();
    if object.uses.is_empty() && !object.lamp && !decor {
        out.push(Flag::NoUse);
    }
    for u in &object.uses {
        if !u.anim.is_empty() && !renderer.actions.contains(&u.anim) {
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
    for (tag, manner) in &object.how {
        if !renderer.manners.contains_key(manner) {
            out.push(Flag::Manner {
                tag: tag.clone(),
                manner: manner.clone(),
            });
        }
    }
    for u in &object.uses {
        let clip = ["uses", "loops"].iter().any(|part| {
            motions
                .and_then(|m| m[*part].as_object())
                .is_some_and(|m| m.contains_key(&u.id) || m.contains_key(&u.anim))
        });
        if clip
            && u.end_sound.is_none()
            && sound_of(heard.tag, object, u).is_none()
            && !hushed(object, u)
        {
            out.push(Flag::Silent {
                using: u.id.clone(),
            });
        }
    }
    let night = motions.is_some_and(|m| m.get("night").is_some_and(|n| !n.is_null()));
    if object.over_window && !night && !object.sounds.contains_key("shut") {
        out.push(Flag::NoShut);
    }
    // (Astride a ride the hands are on its grips, whatever the tag sits with: residents.ts.)
    if !object.top && !renderer.rides.contains(&object.id) {
        for u in object.uses.iter().filter(|u| u.pose == "sit") {
            let manner = object
                .how
                .get(&u.anim)
                .and_then(|m| renderer.manners.get(m));
            let Some(sit) = manner
                .cloned()
                .flatten()
                .or_else(|| renderer.sits.get(&u.anim).cloned())
            else {
                continue;
            };
            if sit_clips(&sit, &object.id, &renderer.sets)
                .iter()
                .any(|c| c.starts_with("sitHands"))
            {
                out.push(Flag::DeskHands {
                    using: u.id.clone(),
                    anim: u.anim.clone(),
                });
            }
        }
    }
    out
}

/// Whether use `u` of `o` is silent by choice: `"none"` its own `sound`, else the object's for its animation (each
/// step's, for a recipe's steps at it).
fn hushed(o: &Object, u: &Use) -> bool {
    let none = |anim: &str, own: Option<&String>| {
        own.or_else(|| o.sounds.get(anim))
            .is_some_and(|s| s == "none")
    };
    match &u.steps_here {
        Some(steps) => {
            !steps.is_empty() && steps.iter().all(|(anim, own)| none(anim, own.as_ref()))
        }
        None => none(&u.anim, u.sound.as_ref()),
    }
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

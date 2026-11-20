//! [(456, 511), (1105, 1161), (1417, 1521), (2909, 2936)]
//!
//! Split from `view.rs` (see `mod.rs`).

use super::*;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FeelingView<'a> {
    pub id: &'a str,
    pub label: &'a str,
    pub mood: f32,
    pub minutes_left: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ActionView {
    pub label: String,
    /// The object it's done at (its stable id, as everywhere in the view).
    pub object: Option<u32>,
    /// The other Sim, for social actions.
    pub target: Option<u32>,
    pub progress: f32,
    pub directed: bool,
    pub active: bool,
}

/// A resident's phone (`phone.rs`): why they can't use it now, if so, and the people they can
/// reach.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PhoneView {
    pub off: Option<&'static str>,
    pub(crate) contacts: Vec<ContactView>,
}

/// Someone in a resident's phone: what keeps them from it (asleep, at work...), and per phone
/// action (content order) why it's out with them right now, if it is.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContactView {
    pub id: u32,
    pub status: Option<&'static str>,
    pub why: Vec<Option<&'static str>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RelView {
    pub a: u32,
    pub b: u32,
    pub friendship: f32,
    pub romance: f32,
    pub partners: bool,
    pub chemistry: f32,
    /// What b is to a in the family.
    #[serde(skip_serializing_if = "crate::social::Kin::is_none")]
    pub kin: crate::social::Kin,
}

// ---- UI updates ------------------------------------------------------------------------------

/// Their phone, if the content gives residents one: everyone they've met who has one too.
pub(crate) fn phone_view(world: &World, s: &Sim) -> Option<PhoneView> {
    let content = &world.content;
    if content.phone.actions.is_empty() {
        return None;
    }
    let me = s.id as usize;
    let off = crate::phone::unusable(world, me);
    let contacts = world
        .sims
        .iter()
        .filter(|t| {
            off.is_none() && t.id != s.id && t.here() && crate::phone::has_phone(content, t.age)
        })
        .filter(|t| world.relationships.get(me, t.id as usize).met)
        .map(|t| {
            let c = t.id as usize;
            ContactView {
                id: t.id,
                status: crate::phone::status_for(world, me, c),
                // (As the command handler decides: `phone::refusal`.)
                why: (0..content.phone.actions.len())
                    .map(|a| crate::phone::refusal(world, me, a, Some(c)))
                    .collect(),
            }
        })
        .collect();
    Some(PhoneView { off, contacts })
}

/// "Visit the Smiths", or the plot's name when nobody lives there.
pub(crate) fn host_name(world: &World, plot: u32) -> String {
    world
        .households
        .iter()
        .find(|h| h.plot == Some(plot))
        .map_or_else(
            || world.plots[plot as usize].name.clone(),
            |h| format!("the {}s", h.name),
        )
}

/// The recipe of what's being eaten (the dish's name is the pot's, not the plate's: "Eat
/// Home-cooked meal").
pub(crate) fn dish(
    world: &World,
    held: Option<crate::items::Held>,
    item: Option<u32>,
    plate: Option<u32>,
) -> Option<usize> {
    [plate, item]
        .into_iter()
        .flatten()
        .find_map(|i| world.items.get(i).and_then(|it| it.recipe))
        .or_else(|| held.and_then(|h| h.recipe))
}

/// What a resident is doing and has queued; work or school, and being talked to, come first.
pub(crate) fn actions(world: &World, s: &Sim) -> Vec<ActionView> {
    let content = &world.content;
    let mut actions = Vec::new();
    if let Some(until) = s.away_until {
        let left = until.saturating_sub(world.tick) as f32 * MINUTES_PER_TICK;
        let school = content.life.school.as_ref().filter(|_| s.job.is_none());
        let total = s
            .job
            .as_ref()
            .map_or(school.map_or(1.0, |sc| sc.hours * 60.0), |j| {
                content.careers[j.career].levels[j.level].hours * 60.0
            });
        actions.push(ActionView {
            label: if school.is_some() {
                "At school"
            } else {
                "At work"
            }
            .to_owned(),
            object: None,
            target: None,
            progress: round_to((1.0 - left / total).clamp(0.0, 1.0), PERCENT),
            directed: true,
            active: true,
        });
    }
    // A baby in someone's arms ("In the arms · Ada", the UI adds who; `carry.rs`).
    if let Some(c) = s.carried_by {
        actions.push(ActionView {
            label: "In the arms".to_owned(),
            object: None,
            target: Some(c),
            progress: 0.0,
            directed: false,
            active: true,
        });
    }
    // Being talked to shows up as the current action.
    if let Some(a) = s.engaged_with
        && let Some((_, social, _, progress)) = world.sims[a as usize].conversation(content)
    {
        actions.push(ActionView {
            label: content.socials[social].label.clone(),
            object: None,
            target: Some(a),
            progress: round_to(progress, PERCENT),
            directed: false,
            active: true,
        });
    }
    // Called over: on their way to the one who called (or waiting there) for what they called
    // them for, not a walk of their own.
    let called =
        s.heeding.and_then(
            |a| match world.sims[a as usize].current().map(|c| c.task.kind) {
                Some(TaskKind::Social { target, social }) if target == s.id => Some((a, social)),
                _ => None,
            },
        );
    if let Some((a, social)) = called {
        actions.push(ActionView {
            label: content.socials[social].label.clone(),
            object: None,
            target: Some(a),
            progress: 0.0,
            directed: false,
            active: true,
        });
    } else {
        actions.extend(s.current().map(|a| {
            let mut view = task_view(world, &a.task, Some(&a.phase), s.held);
            // A game for two: who with ("Shoot pool · Bo"; `partners.rs`).
            if let Some((o, i, _)) = s.game {
                view.target = world
                    .sims
                    .iter()
                    .find(|p| p.id != s.id && p.game.is_some_and(|g| (g.0, g.1) == (o, i)))
                    .map(|p| p.id);
            }
            view
        }));
    }
    actions.extend(s.queue().map(|t| task_view(world, t, None, None)));
    actions
}

pub(crate) fn feelings<'a>(world: &'a World, s: &Sim) -> Vec<FeelingView<'a>> {
    s.feelings
        .iter()
        .map(|m| {
            let def = &world.content.feelings[m.def];
            let left = m.expires.saturating_sub(world.tick) as f32 * MINUTES_PER_TICK;
            FeelingView {
                id: &def.id,
                label: &def.label,
                mood: def.mood,
                // Whole minutes: the panel shows minutes, or hours.
                minutes_left: round_to(left.min(def.minutes), 1.0),
            }
        })
        .collect()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OptionView<'a> {
    pub index: usize,
    pub id: &'a str,
    pub label: &'a str,
    pub category: &'a str,
    pub chance: f32,
}

/// What `actor` can do with `target`, for the social menu.
pub fn social_options_json(world: &World, actor: usize, target: usize) -> String {
    let options: Vec<OptionView> = conversation::options(world, actor, target)
        .into_iter()
        .map(|(i, chance)| {
            let s = &world.content.socials[i];
            OptionView {
                index: i,
                id: &s.id,
                label: &s.label,
                category: &s.category,
                chance,
            }
        })
        .collect();
    serde_json::to_string(&options).expect("options serialize")
}

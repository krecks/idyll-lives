//! Reading routines and goals from content (and the UI's own forms).
//!
//! Split from `planner.rs` (see `mod.rs`).

use super::*;

/// A routine as the UI sends it (ids are kept, new ones assigned).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutineIn {
    #[serde(default)]
    pub id: Option<u16>,
    pub activity: String,
    #[serde(default)]
    pub skill: Option<String>,
    /// Weekdays as a bitmask (bit 0 = Monday).
    pub days: u8,
    pub start: u16,
    pub minutes: u16,
}

/// A goal as the UI sends it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoalIn {
    pub def: String,
    #[serde(default)]
    pub skill: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub object: Option<String>,
    #[serde(default)]
    pub target: Option<f32>,
}

/// Checks routines (valid activity, times, no overlaps) and gives each an id.
pub(crate) fn routines_from(content: &Content, list: &[RoutineIn]) -> Result<Vec<Routine>, Error> {
    let rules = &content.planner;
    let mut out: Vec<Routine> = Vec::with_capacity(list.len());
    for r in list {
        let activity = content
            .activities
            .iter()
            .position(|a| a.id == r.activity)
            .ok_or_else(|| Error::new(format!("unknown activity '{}'", r.activity)))?;
        let def = &content.activities[activity];
        let skill = match &r.skill {
            Some(s) if def.skill => Some(
                content
                    .skill_index(s)
                    .ok_or_else(|| Error::new(format!("unknown skill '{s}'")))?,
            ),
            _ => None,
        };
        let longest = if def.sleep {
            rules.max_sleep_minutes
        } else {
            rules.max_minutes
        };
        if r.days & 0x7f == 0 || r.start >= 1440 || r.minutes < 15 || r.minutes > longest {
            return Err(Error::new(format!(
                "{}: pick at least one day and 15 min to {} h",
                def.label,
                longest / 60
            )));
        }
        let mut routine = Routine {
            id: r.id.unwrap_or(0),
            activity,
            skill,
            days: r.days & 0x7f,
            start: r.start,
            minutes: r.minutes,
        };
        if let Some(other) = out.iter().find(|o| o.overlaps(&routine)) {
            return Err(Error::new(format!(
                "{} overlaps {}",
                def.label, content.activities[other.activity].label
            )));
        }
        if r.id.is_none() || out.iter().any(|o| o.id == routine.id) {
            routine.id = 0;
        }
        out.push(routine);
    }
    // New blocks get fresh ids.
    let first = out.iter().map(|r| r.id).max().unwrap_or(0) + 1;
    for (id, r) in (first..).zip(out.iter_mut().filter(|r| r.id == 0)) {
        r.id = id;
    }
    Ok(out)
}

pub(crate) fn goal_from(content: &Content, g: &GoalIn, sim: &Sim, day: u32) -> Result<Goal, Error> {
    let def = content
        .goals
        .iter()
        .position(|d| d.id == g.def)
        .ok_or_else(|| Error::new(format!("unknown goal '{}'", g.def)))?;
    let kind = content.goals[def].kind;
    let skill = g
        .skill
        .as_deref()
        .map(|s| {
            content
                .skill_index(s)
                .ok_or_else(|| Error::new(format!("unknown skill '{s}'")))
        })
        .transpose()?;
    if kind == GoalKind::Skill && skill.is_none() {
        return Err(Error::new("pick a skill"));
    }
    let category = g
        .category
        .as_deref()
        .map(|c| {
            content
                .career_categories
                .iter()
                .position(|d| d.id == c)
                .ok_or_else(|| Error::new(format!("unknown career category '{c}'")))
        })
        .transpose()?;
    let object = g
        .object
        .as_deref()
        .map(|o| {
            content
                .object_index(o)
                .ok_or_else(|| Error::new(format!("unknown object '{o}'")))
        })
        .transpose()?;
    let price = match (kind, object) {
        (GoalKind::Own, None) => return Err(Error::new("pick an item")),
        (GoalKind::Own, Some(o)) => match content.objects[o].price {
            Some(p) => Some(p as f32),
            None => {
                return Err(Error::new(format!(
                    "{} isn't for sale",
                    content.objects[o].name
                )));
            }
        },
        _ => None,
    };
    let start = match kind {
        GoalKind::Promoted => sim.job.as_ref().map_or(0.0, |j| j.level as f32),
        _ => 0.0,
    };
    Ok(Goal {
        def,
        skill,
        category,
        object: object.filter(|_| kind == GoalKind::Own),
        target: price.unwrap_or(g.target.unwrap_or(1.0)).max(0.0),
        since_day: day,
        start,
        progress: 0.0,
        history: [0.0; 3],
    })
}

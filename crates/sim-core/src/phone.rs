//! Smartphones: everyone old enough carries one (content `phone`) and uses it wherever they are,
//! on a seat close by or standing: texting or calling someone they know, inviting them over,
//! asking to come over to theirs, or just scrolling.
//!
//! A phone action is a task (`TaskKind::Phone`) used on the spot. With someone on the other end
//! it is decided once the phone is out (`update`): someone asleep (or, for a call, at work, at
//! school or busy) doesn't answer; otherwise they say yes or no like the target of a social
//! (`phone.actions[].acceptance`, see `conversation::accepts`). While connected the caller's needs
//! fill; on hanging up the contact gets their share and the relationship changes both ways like
//! after a conversation (`success` / `failure` outcomes, milestones included). An invitation the
//! contact accepted sends them over (`meet: "here"`); a yes to "can I come over?" sends the
//! caller (`meet: "there"`). Both are ordinary visits (`TaskKind::Visit`): they knock at the
//! door and are let in (`life.rs`).
//!
//! Phone calls are what the phone box in the market square was for, so residents no longer
//! walk into town to call someone.

use std::collections::HashMap;

use serde::Deserialize;

use crate::Error;
use crate::content::{Content, MAX_NEEDS, Pose, TagMask};
use crate::social::{
    self, Acceptance, EventKind, Indices, Outcome, OutcomeRaw, Prefer, Requirements,
};
use crate::world::{
    Access, Activity, Ctx, ObjectInstance, Phase, Sim, Task, TaskKind, World, access, end_activity,
    prefer_factor, walk_route,
};

/// After a yes, who goes where (`phone.actions[].meet`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Meet {
    /// The contact comes over to the caller's home (an invitation).
    Here,
    /// The caller goes over to the contact's home.
    There,
}

/// How the other end took it, decided once the phone is out (`update`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answer {
    Pending,
    /// Picked up (or read it) and said yes.
    Yes,
    /// Asleep, or out of reach for a call: nobody answered.
    NoAnswer,
    /// Answered, and said no.
    No,
}

/// Something to do on the phone (content `phone.actions`).
#[derive(Debug, Clone)]
pub struct PhoneAction {
    pub id: String,
    pub label: String,
    pub minutes: f32,
    /// With someone (a contact): texting, calling, inviting; without: scrolling.
    pub contact: bool,
    /// A message rather than a call: reaches anyone awake (at work, at school, busy).
    pub message: bool,
    /// After a yes, a visit (none: just the talk).
    pub meet: Option<Meet>,
    /// Animation tag while at it (content `anim`; default the phone's `anim`): texting and
    /// scrolling look at the screen, a call holds it to the ear.
    pub anim: Option<usize>,
    pub autonomous: bool,
    pub autonomy_weight: f32,
    pub prefer: Prefer,
    pub tags: TagMask,
    /// Conditions on the caller's view of the contact (default: they have met).
    pub requires: Requirements,
    /// Whether the contact says yes (the contact's view of the caller counts).
    pub acceptance: Acceptance,
    /// Need gains over the whole action, for the one on the phone (only while connected, with a
    /// contact).
    pub gains: [f32; MAX_NEEDS],
    /// Need gains for the contact, on hanging up.
    pub contact_gains: [f32; MAX_NEEDS],
    /// Relationship changes, needs and feelings after a yes or a no (as for socials; a contact
    /// who didn't answer changes nothing).
    pub success: Outcome,
    pub failure: Outcome,
}

/// Content `phone`: who has one and what they do with it. No actions: nobody has a phone.
#[derive(Debug, Clone)]
pub struct PhoneRules {
    /// Residents this old or older have one (the `from` of content stage `fromStage`).
    pub from_age: f32,
    /// Animation tag while on the phone.
    pub anim: Option<usize>,
    /// Seats to sit on while on the phone: sitting interactions with any of these tags and none
    /// of `seat_avoid`, within `seat_range` metres (else standing).
    pub seat_tags: TagMask,
    pub seat_avoid: TagMask,
    pub seat_range: f32,
    /// How long they try before giving up when nobody answers.
    pub no_answer_minutes: f32,
    pub actions: Vec<PhoneAction>,
}

impl PhoneRules {
    pub fn action_index(&self, id: &str) -> Option<usize> {
        self.actions.iter().position(|a| a.id == id)
    }
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct PhoneRaw {
    from_stage: Option<String>,
    anim: Option<String>,
    seat_tags: Option<Vec<String>>,
    seat_avoid: Option<Vec<String>>,
    seat_range: Option<f32>,
    no_answer_minutes: Option<f32>,
    pub actions: Vec<PhoneActionRaw>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PhoneActionRaw {
    id: String,
    label: String,
    minutes: f32,
    #[serde(default)]
    contact: bool,
    #[serde(default)]
    message: bool,
    #[serde(default)]
    meet: Option<Meet>,
    #[serde(default)]
    anim: Option<String>,
    #[serde(default = "yes")]
    autonomous: bool,
    #[serde(default = "one")]
    autonomy_weight: f32,
    #[serde(default)]
    prefer: Prefer,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    requires: Requirements,
    #[serde(default)]
    acceptance: social::AcceptanceRaw,
    #[serde(default)]
    needs: HashMap<String, f32>,
    #[serde(default)]
    contact_needs: HashMap<String, f32>,
    #[serde(default)]
    success: OutcomeRaw,
    #[serde(default)]
    failure: OutcomeRaw,
}

fn yes() -> bool {
    true
}
fn one() -> f32 {
    1.0
}

/// Phone rules from content (`stages`: content `life.stages`, for `fromStage`).
pub(crate) fn parse(
    raw: &PhoneRaw,
    ix: &Indices,
    animations: &[String],
    stages: &[crate::content::LifeStage],
) -> Result<PhoneRules, Error> {
    let from_age = match &raw.from_stage {
        Some(id) => stages
            .iter()
            .find(|s| &s.id == id)
            .map(|s| s.from)
            .ok_or_else(|| Error::new(format!("phone.fromStage: unknown life stage '{id}'")))?,
        // Everyone but babies.
        None => stages.iter().find(|s| !s.baby).map_or(0.0, |s| s.from),
    };
    let tag = |a: &str, ctx: &str| {
        animations.iter().position(|x| x == a).ok_or_else(|| {
            Error::new(format!(
                "{ctx}.anim: unknown anim '{a}' (declare it in \"animations\")"
            ))
        })
    };
    let anim = match &raw.anim {
        Some(a) => Some(tag(a, "phone")?),
        None => animations.iter().position(|x| x == "phone"),
    };
    // (Seat tags content doesn't have are left out.)
    let mask = |tags: &[String]| {
        tags.iter()
            .filter_map(|t| ix.tags.get(t.as_str()))
            .fold(0, |m: TagMask, i| m | (1 << i))
    };
    let mut actions = Vec::with_capacity(raw.actions.len());
    for a in &raw.actions {
        let ctx = format!("phone.{}", a.id);
        if a.minutes <= 0.0 {
            return Err(Error::new(format!("{ctx}: minutes must be > 0")));
        }
        if a.meet.is_some() && !a.contact {
            return Err(Error::new(format!("{ctx}: `meet` needs a `contact`")));
        }
        if actions.iter().any(|b: &PhoneAction| b.id == a.id) {
            return Err(Error::new(format!("{ctx}: duplicate phone action")));
        }
        let success = social::parse_outcome(&a.success, ix, &ctx)?;
        let failure = social::parse_outcome(&a.failure, ix, &ctx)?;
        if success.effect != social::Effect::None || failure.effect != social::Effect::None {
            return Err(Error::new(format!(
                "{ctx}: phone outcomes have no `effect` (see `meet`)"
            )));
        }
        actions.push(PhoneAction {
            id: a.id.clone(),
            label: a.label.clone(),
            minutes: a.minutes,
            contact: a.contact,
            message: a.message,
            meet: a.meet,
            anim: match &a.anim {
                Some(t) => Some(tag(t, &ctx)?),
                None => anim,
            },
            autonomous: a.autonomous,
            autonomy_weight: a.autonomy_weight.max(0.0),
            prefer: a.prefer,
            tags: social::tag_mask(&a.tags, ix, &ctx)?,
            requires: a.requires,
            acceptance: social::parse_acceptance(&a.acceptance, ix, &ctx)?,
            gains: social::need_array(&a.needs, ix, &ctx)?,
            contact_gains: social::need_array(&a.contact_needs, ix, &ctx)?,
            success,
            failure,
        });
    }
    Ok(PhoneRules {
        from_age,
        anim,
        seat_tags: mask(
            &raw.seat_tags
                .clone()
                .unwrap_or_else(|| vec!["lounge".into()]),
        ),
        seat_avoid: mask(&raw.seat_avoid.clone().unwrap_or_else(|| {
            vec![
                "spa".into(),
                "hygiene".into(),
                "bathroom".into(),
                "sleep".into(),
            ]
        })),
        seat_range: raw.seat_range.unwrap_or(5.0).max(0.0),
        no_answer_minutes: raw.no_answer_minutes.unwrap_or(3.0).max(1.0),
        actions,
    })
}

/// Someone this old has a phone.
pub fn has_phone(content: &Content, age: f32) -> bool {
    !content.phone.actions.is_empty() && age >= content.phone.from_age && !content.life.baby(age)
}

/// A phone action that ended this tick (`Sim::hung_up`), applied after the step (`update`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct HungUp {
    action: usize,
    contact: Option<u32>,
    answer: Answer,
    directed: bool,
}

// ---- Who can be reached ----------------------------------------------------------------------

/// Why a contact can't be reached right now (or a phone action is out), for the UI and for
/// deciding whether anyone answers. Short, to show next to their name or on a button.
pub mod why {
    pub const TOO_YOUNG: &str = "Too young for a phone";
    pub const ASLEEP: &str = "Asleep";
    pub const AT_WORK: &str = "At work";
    pub const AT_SCHOOL: &str = "At school";
    pub const ON_THE_PHONE: &str = "On the phone";
    pub const BUSY: &str = "Busy";
    pub const NOT_MET: &str = "They haven't met";
    pub const NOT_CLOSE: &str = "Not close enough";
    pub const HOUSEMATE: &str = "Lives with them";
    pub const NOT_NOW: &str = "Not at this hour";
    pub const OUT_VISITING: &str = "Out visiting";
    pub const NOT_HOME: &str = "Not at home";
    pub const NO_HOME: &str = "No home to visit";
    pub const ONLY_FROM_HOME: &str = "Only from home";
    pub const ALREADY_THERE: &str = "Already together";
    pub const GONE: &str = "Not in town";
    pub const RIGHT_HERE: &str = "Right here";
}

/// What a contact is up to, if it keeps them from the phone: asleep, at work or school, on
/// another call, or busy (in a conversation, the bathroom, something the player asked for).
pub fn status(w: &World, contact: usize) -> Option<&'static str> {
    let content = &w.content;
    let t = w.sims.get(contact).filter(|s| s.here())?;
    if t.asleep(content) {
        Some(why::ASLEEP)
    } else if t.away_until.is_some() {
        Some(if t.job.is_some() {
            why::AT_WORK
        } else {
            why::AT_SCHOOL
        })
    } else if on_a_call(t) {
        Some(why::ON_THE_PHONE)
    } else if !t.available_for_social(content) {
        Some(why::BUSY)
    } else {
        None
    }
}

/// What keeps `contact` from `me`'s phone: `status`, or being right here (on the same lot:
/// they'd talk face to face).
pub fn status_for(w: &World, me: usize, contact: usize) -> Option<&'static str> {
    let on = |i: usize| {
        w.sims
            .get(i)
            .filter(|s| s.here() && s.away_until.is_none())
            .and_then(|s| w.plot_at(s.tile().0, s.tile().1))
    };
    match (on(me), on(contact)) {
        (Some(a), Some(b)) if a == b && me != contact => Some(why::RIGHT_HERE),
        _ => status(w, contact),
    }
}

/// On the phone with someone.
fn on_a_call(s: &Sim) -> bool {
    s.current().is_some_and(|a| {
        matches!(
            a.task.kind,
            TaskKind::Phone {
                contact: Some(_),
                ..
            }
        )
    })
}

/// Why `me` can't use the phone at all right now (too young, at work or school, gone).
pub fn unusable(w: &World, me: usize) -> Option<&'static str> {
    let s = w.sims.get(me).filter(|s| s.here())?;
    if !has_phone(&w.content, s.age) {
        Some(why::TOO_YOUNG)
    } else if s.away_until.is_some() {
        Some(if s.job.is_some() {
            why::AT_WORK
        } else {
            why::AT_SCHOOL
        })
    } else {
        None
    }
}

/// Why `me` can't do phone action `action` with `contact` (a resident) now: None if they can
/// (whether the contact says yes is another matter).
pub fn why_not(w: &World, me: usize, contact: usize, action: usize) -> Option<&'static str> {
    let content = &w.content;
    let act = content.phone.actions.get(action)?;
    if !act.contact {
        return None;
    }
    let (Some(s), Some(t)) = (
        w.sims.get(me),
        w.sims.get(contact).filter(|t| t.here() && contact != me),
    ) else {
        return Some(why::GONE);
    };
    if !has_phone(content, t.age) {
        return Some(why::TOO_YOUNG);
    }
    let rel = w.relationships.get(me, contact);
    if !act.requires.allows(rel) {
        return Some(if rel.met {
            why::NOT_CLOSE
        } else {
            why::NOT_MET
        });
    }
    if let Some(busy) = status_for(w, me, contact)
        && (busy == why::ASLEEP || busy == why::RIGHT_HERE || !act.message)
    {
        return Some(busy);
    }
    let meet = act.meet?;
    let visits = &content.visits;
    if !(visits.earliest_hour..visits.latest_hour).contains(&crate::clock::hour(w.tick)) {
        return Some(why::NOT_NOW);
    }
    if t.household == s.household {
        return Some(why::HOUSEMATE);
    }
    if t.visiting.is_some() {
        return Some(why::OUT_VISITING);
    }
    let on = |x: &Sim| w.plot_at(x.tile().0, x.tile().1);
    match meet {
        Meet::Here => {
            let home = w.households[s.household as usize].plot;
            if home.is_none() || s.visiting.is_some() || on(s) != home {
                return Some(why::ONLY_FROM_HOME);
            }
            if on(t) == home {
                return Some(why::ALREADY_THERE);
            }
        }
        Meet::There => {
            let home = w.households[t.household as usize].plot;
            if home.is_none() {
                return Some(why::NO_HOME);
            }
            if on(t) != home {
                return Some(why::NOT_HOME);
            }
            if s.visiting.is_some_and(|v| Some(v.plot) == home) {
                return Some(why::ALREADY_THERE);
            }
        }
    }
    None
}

/// Why a player's phone order (`action`, with `contact` if it takes one) is refused right now,
/// or None if it goes ahead (whether the contact says yes is another matter): the phone out of
/// use (`unusable`), the contact out of reach or the visit out (`why_not`), or going over to
/// theirs would leave a baby home alone (`life::homebound`). The command handler
/// (`check_order`) and the UI (`view/`) both ask this, so a button is greyed out exactly when
/// pressing it would be refused.
pub fn refusal(
    w: &World,
    me: usize,
    action: usize,
    contact: Option<usize>,
) -> Option<&'static str> {
    let act = w.content.phone.actions.get(action)?;
    if let Some(off) = unusable(w, me) {
        return Some(off);
    }
    if let Some(c) = contact.filter(|_| act.contact)
        && let Some(why) = why_not(w, me, c, action)
    {
        return Some(why);
    }
    if act.meet == Some(Meet::There) {
        return crate::life::homebound(w, me);
    }
    None
}

/// Checks a player's phone order: the action exists and takes a contact (a resident other
/// than `me`) if and only if it needs one, and it isn't refused right now (`refusal`).
/// Whether the contact is still free when the phone comes out is decided then (`update`).
pub(crate) fn check_order(
    w: &World,
    me: usize,
    action: usize,
    contact: Option<u32>,
) -> Result<(), Error> {
    let act = w
        .content
        .phone
        .actions
        .get(action)
        .ok_or_else(|| Error::new(format!("unknown phone action {action}")))?;
    if let Some(why) = unusable(w, me) {
        return Err(Error::new(format!(
            "{}: {}",
            w.sims[me].name,
            why.to_lowercase()
        )));
    }
    let contact = match (act.contact, contact) {
        (false, None) => None,
        (false, Some(_)) => {
            return Err(Error::new(format!(
                "{} is just for {}",
                act.label, w.sims[me].name
            )));
        }
        (true, None) => {
            return Err(Error::new(format!(
                "{} needs someone to contact",
                act.label
            )));
        }
        (true, Some(c)) => {
            let c = c as usize;
            w.sims
                .get(c)
                .filter(|t| t.here() && c != me)
                .ok_or_else(|| Error::new(format!("unknown resident {c}")))?;
            Some(c)
        }
    };
    match refusal(w, me, action, contact) {
        None => Ok(()),
        // (The longer story: whom, and what a babysitter costs.)
        Some(crate::life::NO_SITTER) => crate::life::check_cover(w, me),
        Some(why) => {
            let who = contact.map_or_else(String::new, |c| format!(" · {}", w.sims[c].name));
            Err(Error::new(format!(
                "{}{who}: {}",
                act.label,
                why.to_lowercase()
            )))
        }
    }
}

mod autonomy;
mod using;

pub(crate) use autonomy::*;
pub(crate) use using::*;

//! The content file's raw JSON shapes and the serde defaults they share.

use super::*;

mod economy;
mod life;
mod objects;
mod people;
mod rooms;

pub(crate) use economy::*;
pub(crate) use life::*;
pub(crate) use objects::*;
pub(crate) use people::*;
pub(crate) use rooms::*;

/// Serde default: one (a rate or a count).
pub(crate) fn one() -> f32 {
    1.0
}

/// Serde default: Sunday, the week's last day (content weeks start on Monday).
pub(crate) fn sunday() -> u32 {
    6
}

/// Serde default: noon.
pub(crate) fn noon() -> f32 {
    12.0
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContentFile {
    pub(crate) needs: Vec<NeedRaw>,
    pub(crate) objects: Vec<ObjectRaw>,
    #[serde(default)]
    pub(crate) traits: Vec<TraitRaw>,
    #[serde(default)]
    pub(crate) perks: Vec<PerkRaw>,
    #[serde(default)]
    pub(crate) genders: Vec<GenderRaw>,
    #[serde(default)]
    pub(crate) rules: CharacterRules,
    #[serde(default)]
    pub(crate) emotions: Vec<EmotionRaw>,
    #[serde(default)]
    #[serde(alias = "moodlets")] // key before the moodlet → feeling rename
    pub(crate) feelings: Vec<FeelingRaw>,
    #[serde(default)]
    pub(crate) socials: Vec<SocialRaw>,
    #[serde(default)]
    pub(crate) bond_presets: HashMap<String, BondPreset>,
    #[serde(default)]
    pub(crate) social_rules: SocialRulesRaw,
    #[serde(default)]
    pub(crate) careers: Vec<CareerRaw>,
    #[serde(default)]
    pub(crate) career_rules: CareerRulesRaw,
    #[serde(default)]
    pub(crate) schedule: Vec<ScheduleRaw>,
    #[serde(default)]
    pub(crate) visits: VisitRaw,
    #[serde(default)]
    pub(crate) economy: EconomyRaw,
    #[serde(default)]
    pub(crate) skills: Vec<SkillRaw>,
    #[serde(default)]
    pub(crate) skill_rules: SkillRulesRaw,
    #[serde(default)]
    pub(crate) grades: Vec<GradeRaw>,
    #[serde(default)]
    pub(crate) career_categories: Vec<CategoryRaw>,
    #[serde(default)]
    pub(crate) car_bodies: Vec<CarBodyRaw>,
    #[serde(default)]
    pub(crate) day_rhythm: DayRhythmRaw,
    #[serde(default)]
    pub(crate) autonomy: crate::ai::AutonomyRaw,
    #[serde(default)]
    pub(crate) object_rules: ObjectRulesRaw,
    #[serde(default)]
    pub(crate) room_kinds: Vec<RoomKindRaw>,
    #[serde(default)]
    pub(crate) accidents: Vec<AccidentRaw>,
    #[serde(default)]
    pub(crate) life: LifeRaw,
    #[serde(default)]
    pub(crate) names: Names,
    #[serde(default)]
    pub(crate) room_rules: RoomRulesRaw,
    #[serde(default)]
    pub(crate) build: BuildRaw,
    #[serde(default)]
    pub(crate) wall_coverings: Vec<BuildStyleRaw>,
    #[serde(default)]
    pub(crate) door_styles: Vec<BuildStyleRaw>,
    #[serde(default)]
    pub(crate) window_styles: Vec<BuildStyleRaw>,
    #[serde(default)]
    pub(crate) fence_styles: Vec<BuildStyleRaw>,
    #[serde(default)]
    pub(crate) roof_styles: Vec<BuildStyleRaw>,
    #[serde(default)]
    pub(crate) roof_colors: Vec<BuildStyleRaw>,
    #[serde(default)]
    pub(crate) floor_coverings: Vec<BuildStyleRaw>,
    /// Extra tags that no interaction or social uses (yet), so traits and emotions can refer to them.
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    /// Animation tags interactions may use as `anim`.
    #[serde(default)]
    pub(crate) animations: Vec<String>,
    #[serde(default)]
    pub(crate) activities: Vec<crate::planner::ActivityRaw>,
    #[serde(default)]
    pub(crate) planner: crate::planner::PlannerRaw,
    #[serde(default)]
    pub(crate) goals: Vec<crate::planner::GoalRaw>,
    #[serde(default)]
    pub(crate) cooking: crate::cooking::CookingRaw,
    #[serde(default)]
    pub(crate) items: Vec<crate::cooking::ItemRaw>,
    #[serde(default)]
    pub(crate) recipes: Vec<crate::cooking::RecipeRaw>,
    #[serde(default)]
    pub(crate) phone: crate::phone::PhoneRaw,
    #[serde(default)]
    pub(crate) service: Option<crate::staff::ServiceRaw>,
    #[serde(default)]
    pub(crate) laundry: Option<LaundryRaw>,
    #[serde(default)]
    pub(crate) cars: CarsRaw,
    #[serde(default)]
    pub(crate) taxi: TaxiRaw,
}

fn one_by_one() -> [u8; 2] {
    [1, 1]
}

fn yes() -> bool {
    true
}

pub(crate) fn intern(tags: &mut Vec<String>, tag: &str) -> Result<(), Error> {
    if !tags.iter().any(|t| t == tag) {
        if tags.len() == MAX_TAGS {
            return Err(Error::new(format!("more than {MAX_TAGS} tags")));
        }
        tags.push(tag.to_owned());
    }
    Ok(())
}

pub(crate) fn build_modifiers(
    raw: &ModifiersRaw,
    ix: &Indices,
    ctx: &str,
) -> Result<Modifiers, Error> {
    let tag = |t: &str| lookup(ix.tags, t, ctx, "tag");
    // Multipliers (decay, gain, preference, skill gain) are at least 0, everything a number.
    let factor = |v: f32, what: &str| {
        if v >= 0.0 && v.is_finite() {
            Ok(v)
        } else {
            Err(Error::new(format!(
                "{ctx}: {what} {v} must be a number ≥ 0"
            )))
        }
    };
    let number = |v: f32, what: &str| {
        if v.is_finite() {
            Ok(v)
        } else {
            Err(Error::new(format!("{ctx}: {what} must be a number")))
        }
    };
    let mut m = Modifiers::default();
    for (need, v) in &raw.need_decay {
        m.need_decay[lookup(ix.needs, need, ctx, "need")?] = factor(*v, "needDecay")?;
    }
    for (need, v) in &raw.need_gain {
        m.need_gain[lookup(ix.needs, need, ctx, "need")?] = factor(*v, "needGain")?;
    }
    for (t, v) in &raw.tag_preference {
        m.tag_preference[tag(t)?] = factor(*v, "tagPreference")?;
    }
    for (t, v) in &raw.tag_acceptance {
        m.tag_acceptance[tag(t)?] = number(*v, "tagAcceptance")?;
    }
    for (t, v) in &raw.tag_success {
        m.tag_success[tag(t)?] = number(*v, "tagSuccess")?;
    }
    for (skill, v) in &raw.skill_gain {
        m.skill_gain[lookup(ix.skills, skill, ctx, "skill")?] = factor(*v, "skillGain")?;
    }
    // Residents walk: a speed of 0 (or less) would leave them walking for ever.
    m.walk_speed = match raw.walk_speed {
        None => 1.0,
        Some(v) if v > 0.0 && v <= MAX_WALK_SPEED => v,
        Some(v) => {
            return Err(Error::new(format!(
                "{ctx}: walkSpeed {v} must be above 0 and at most {MAX_WALK_SPEED}"
            )));
        }
    };
    m.mood = number(raw.mood, "mood")?;
    Ok(m)
}

/// Animation tag for an interaction without an `anim`, from its tags, pose and needs.
/// The caller drops tags that content doesn't declare in `animations`.
pub(crate) fn default_anim(it: &InteractionRaw) -> &'static str {
    let has = |t: &str| it.tags.iter().any(|x| x == t);
    // The need it raises most (ties: first in id order, so the result is deterministic).
    let main_need = it
        .effects
        .iter()
        .filter(|(_, g)| **g > 0.0)
        .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(a.0)))
        .map(|(n, _)| n.as_str());
    let seated = it.pose != Pose::Stand;
    if it.eat.is_some() {
        // Getting food to eat elsewhere.
        "cook"
    } else if has("reading") {
        "read"
    } else if has("cooking") {
        // Cooking at a table is eating what was cooked.
        if seated { "eat" } else { "cook" }
    } else if has("sleep") {
        "sleep"
    } else if has("nap") {
        "nap"
    } else if has("bathroom") || main_need == Some("bladder") {
        "toilet"
    } else if main_need == Some("hunger") {
        "eat"
    } else if has("social") || main_need == Some("social") {
        "talk"
    } else if has("fitness") || has("sport") {
        "exercise"
    } else if has("gaming") {
        "play"
    } else if has("music") {
        "music"
    } else if has("garden") || (has("chores") && has("nature")) {
        "garden"
    } else if has("cleaning") || has("chores") {
        "clean"
    } else if has("spa") || has("hygiene") || main_need == Some("hygiene") {
        match (seated, it.minutes <= 10.0) {
            (true, _) => "bath",
            (false, true) => "wash",
            (false, false) => "shower",
        }
    } else if has("screen") {
        if seated && has("training") {
            "type"
        } else {
            "watch"
        }
    } else if has("food") {
        "drink"
    } else if has("creative") {
        if seated { "write" } else { "paint" }
    } else if has("rest") || has("lounge") || it.pose == Pose::Lie {
        "relax"
    } else if seated {
        "sit"
    } else {
        "idle"
    }
}

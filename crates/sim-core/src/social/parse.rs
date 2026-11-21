//! Parsing
//!
//! Turning the raw JSON into definitions, against the shared indices.

use super::*;

/// Lookups shared by the parsers.
pub(crate) struct Indices<'a> {
    pub needs: &'a HashMap<&'a str, usize>,
    pub tags: &'a HashMap<&'a str, usize>,
    pub emotions: &'a HashMap<String, usize>,
    pub feelings: &'a HashMap<String, usize>,
    pub skills: &'a HashMap<&'a str, usize>,
}

fn find(index: &HashMap<String, usize>, key: &str, ctx: &str, kind: &str) -> Result<usize, Error> {
    index
        .get(key)
        .copied()
        .ok_or_else(|| Error::new(format!("{ctx}: unknown {kind} '{key}'")))
}

pub(crate) fn need_array(
    map: &HashMap<String, f32>,
    ix: &Indices,
    ctx: &str,
) -> Result<[f32; MAX_NEEDS], Error> {
    let mut out = [0.0; MAX_NEEDS];
    for (k, v) in map {
        let i = ix
            .needs
            .get(k.as_str())
            .ok_or_else(|| Error::new(format!("{ctx}: unknown need '{k}'")))?;
        out[*i] = *v;
    }
    Ok(out)
}

pub(crate) fn tag_mask(tags: &[String], ix: &Indices, ctx: &str) -> Result<TagMask, Error> {
    let mut mask: TagMask = 0;
    for t in tags {
        let i = ix
            .tags
            .get(t.as_str())
            .ok_or_else(|| Error::new(format!("{ctx}: unknown tag '{t}'")))?;
        mask |= 1 << i;
    }
    Ok(mask)
}

fn opt_feeling(id: &Option<String>, ix: &Indices, ctx: &str) -> Result<Option<usize>, Error> {
    id.as_deref()
        .map(|m| find(ix.feelings, m, ctx, "feeling"))
        .transpose()
}

pub(crate) fn parse_feeling(raw: &FeelingRaw, ix: &Indices) -> Result<FeelingDef, Error> {
    let effects = raw
        .effects
        .as_ref()
        .map(|e| crate::content::build_modifiers(e, ix, &raw.id))
        .transpose()?;
    if effects.as_ref().is_some_and(|e| e.mood != 0.0) {
        return Err(Error::new(format!(
            "{}: use the feeling's own `mood`, not effects.mood",
            raw.id
        )));
    }
    if !(raw.hours > 0.0 && raw.hours.is_finite()) {
        return Err(Error::new(format!("{}: hours must be > 0", raw.id)));
    }
    Ok(FeelingDef {
        id: raw.id.clone(),
        label: raw.label.clone(),
        emotion: raw
            .emotion
            .as_deref()
            .map(|e| find(ix.emotions, e, &raw.id, "emotion"))
            .transpose()?,
        mood: raw.mood,
        minutes: raw.hours * 60.0,
        effects,
    })
}

pub(crate) fn feeling_id(raw: &FeelingRaw) -> &str {
    &raw.id
}

pub(crate) fn parse_outcome(raw: &OutcomeRaw, ix: &Indices, ctx: &str) -> Result<Outcome, Error> {
    Ok(Outcome {
        friendship: raw.friendship,
        romance: raw.romance,
        target_friendship: raw.target_friendship,
        target_romance: raw.target_romance,
        feeling: opt_feeling(&raw.feeling, ix, ctx)?,
        target_feeling: opt_feeling(&raw.target_feeling, ix, ctx)?,
        needs: need_array(&raw.needs, ix, ctx)?,
        target_needs: need_array(&raw.target_needs, ix, ctx)?,
        effect: raw.effect,
        winner_feeling: opt_feeling(&raw.winner_feeling, ix, ctx)?,
        loser_feeling: opt_feeling(&raw.loser_feeling, ix, ctx)?,
    })
}

pub(crate) fn parse_social(
    raw: &SocialRaw,
    ix: &Indices,
    stage_ids: &[&str],
) -> Result<SocialDef, Error> {
    let ctx = raw.id.as_str();
    if !(raw.minutes > 0.0 && raw.minutes.is_finite()) {
        return Err(Error::new(format!("{ctx}: minutes must be > 0")));
    }
    let anim = match raw.animation.as_deref() {
        None => 1,
        Some(a) => ANIMATIONS
            .iter()
            .position(|x| *x == a)
            .map(|i| i as u8 + 1)
            .ok_or_else(|| Error::new(format!("{ctx}: unknown animation '{a}'")))?,
    };
    Ok(SocialDef {
        id: raw.id.clone(),
        label: raw.label.clone(),
        category: raw.category.clone(),
        minutes: raw.minutes,
        distance: raw.distance.clamp(0.4, 2.0),
        anim,
        autonomous: raw.autonomous,
        autonomy_weight: raw.autonomy_weight,
        joinable: raw.joinable,
        tags: tag_mask(&raw.tags, ix, ctx)?,
        requires: raw.requires,
        stages: crate::content::stage_mask(stage_ids, &raw.stages, ctx)?.unwrap_or(StageMask::MAX),
        target_stages: crate::content::stage_mask(stage_ids, &raw.target_stages, ctx)?
            .unwrap_or(StageMask::MAX),
        acceptance: parse_acceptance(&raw.acceptance, ix, ctx)?,
        prefer: raw.prefer,
        actor_gain: need_array(&raw.needs, ix, ctx)?,
        target_gain: need_array(&raw.target_needs, ix, ctx)?,
        success: parse_outcome(&raw.success, ix, ctx)?,
        failure: parse_outcome(&raw.failure, ix, ctx)?,
    })
}

pub(crate) fn parse_rules(
    raw: &SocialRulesRaw,
    presets: &HashMap<String, BondPreset>,
    ix: &Indices,
) -> Result<SocialRules, Error> {
    let default_bond = match &raw.default_bond {
        None => None,
        Some(p) => Some(
            *presets
                .get(p)
                .ok_or_else(|| Error::new(format!("socialRules: unknown bond preset '{p}'")))?,
        ),
    };
    Ok(SocialRules {
        busy_tags: tag_mask(&raw.busy_tags, ix, "socialRules.busyTags")?,
        guest_tags: tag_mask(&raw.guest_tags, ix, "socialRules.guestTags")?,
        romantic_tags: tag_mask(&raw.romantic_tags, ix, "socialRules.romanticTags")?,
        default_bond,
        jealousy_range: raw.jealousy_range.unwrap_or(8.0),
        jealousy_friendship: raw.jealousy_friendship.unwrap_or(-10.0),
        jealousy_romance: raw.jealousy_romance.unwrap_or(-15.0),
        jealousy_feeling: opt_feeling(&raw.jealousy_feeling, ix, "socialRules")?,
        heartbreak_feeling: opt_feeling(&raw.heartbreak_feeling, ix, "socialRules")?,
        friendship_decay: raw.friendship_decay.unwrap_or(1.5),
        romance_decay: raw.romance_decay.unwrap_or(3.0),
        single_spark: raw.single_spark.unwrap_or(0.0).max(0.0),
        join_friendship: raw.join_friendship.unwrap_or(2.0),
    })
}

impl SocialRules {
    pub fn empty() -> Self {
        Self {
            busy_tags: 0,
            guest_tags: 0,
            romantic_tags: 0,
            default_bond: None,
            jealousy_range: 8.0,
            jealousy_friendship: -10.0,
            jealousy_romance: -15.0,
            jealousy_feeling: None,
            heartbreak_feeling: None,
            friendship_decay: 1.5,
            romance_decay: 3.0,
            single_spark: 0.0,
            join_friendship: 2.0,
        }
    }
}

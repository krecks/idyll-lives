//! [(3487, 6446)]
//!
//! Split from `content.rs` (see `mod.rs`).

use super::*;

impl Content {
    pub fn from_json(json: &str) -> Result<Self, Error> {
        let raw: ContentFile = serde_json::from_str(json)?;
        if raw.needs.is_empty() || raw.needs.len() > MAX_NEEDS {
            return Err(Error::new(format!(
                "content must define 1..={MAX_NEEDS} needs"
            )));
        }
        // (Needs only fall over time: a negative decay would push one past full.)
        if let Some(n) = raw
            .needs
            .iter()
            .find(|n| !(n.decay_per_hour >= 0.0 && n.decay_per_hour.is_finite()))
        {
            return Err(Error::new(format!(
                "need '{}': decayPerHour {} must be a number ≥ 0",
                n.id, n.decay_per_hour
            )));
        }

        let needs: Vec<NeedDef> = raw
            .needs
            .iter()
            .map(|n| NeedDef {
                id: n.id.clone(),
                label: n.label.clone(),
                decay_per_minute: n.decay_per_hour / 60.0,
                room: n.room,
                comfortable: n.comfortable.clamp(0.0, 1.0),
            })
            .collect();
        let need_index: HashMap<&str, usize> = needs
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id.as_str(), i))
            .collect();
        if need_index.len() != needs.len() {
            return Err(Error::new("duplicate need id"));
        }

        // Tags come from object interactions and socials; everything else refers to them.
        let mut tags: Vec<String> = Vec::new();
        for obj in &raw.objects {
            for it in &obj.interactions {
                for t in &it.tags {
                    intern(&mut tags, t)?;
                }
            }
        }
        for s in &raw.socials {
            for t in &s.tags {
                intern(&mut tags, t)?;
            }
        }
        for t in &raw.tags {
            intern(&mut tags, t)?;
        }
        for a in &raw.phone.actions {
            for t in &a.tags {
                intern(&mut tags, t)?;
            }
        }
        // Built-in tag for visiting other households (schedules and traits can refer to it).
        intern(&mut tags, VISIT_TAG)?;
        // Tidying up is an action without an object; its tags exist all the same.
        let clean_tags = raw
            .room_rules
            .clean
            .tags
            .clone()
            .unwrap_or_else(|| vec!["chores".into(), "cleaning".into()]);
        for t in &clean_tags {
            intern(&mut tags, t)?;
        }
        // Capabilities (what an object provides, what an interaction requires) share the tag
        // namespace, so content can refer to them by name without another list.
        for obj in &raw.objects {
            for t in &obj.provides {
                intern(&mut tags, t)?;
            }
            for it in &obj.interactions {
                for t in &it.requires {
                    intern(&mut tags, t)?;
                }
            }
        }
        let tag_index: HashMap<&str, usize> = tags
            .iter()
            .enumerate()
            .map(|(i, t)| (t.as_str(), i))
            .collect();

        // Animation tags: declared once (packs may append); repeats are merged.
        let mut animations: Vec<String> = Vec::new();
        for a in &raw.animations {
            if a.is_empty() {
                return Err(Error::new("animations: empty animation tag"));
            }
            if !animations.contains(a) {
                animations.push(a.clone());
            }
        }
        let anim_index = |a: &str| animations.iter().position(|x| x == a);

        let emotion_index: HashMap<String, usize> = raw
            .emotions
            .iter()
            .enumerate()
            .map(|(i, e)| (e.id.clone(), i))
            .collect();
        let feeling_index: HashMap<String, usize> = raw
            .feelings
            .iter()
            .enumerate()
            .map(|(i, m)| (social::feeling_id(m).to_owned(), i))
            .collect();
        if emotion_index.len() != raw.emotions.len() {
            return Err(Error::new("duplicate emotion id"));
        }
        if feeling_index.len() != raw.feelings.len() {
            return Err(Error::new("duplicate feeling id"));
        }
        if raw.skills.len() > MAX_SKILLS {
            return Err(Error::new(format!("at most {MAX_SKILLS} skills")));
        }
        let skills: Vec<SkillDef> = raw
            .skills
            .iter()
            .map(|s| SkillDef {
                id: s.id.clone(),
                label: s.label.clone(),
            })
            .collect();
        let skill_index: HashMap<&str, usize> = skills
            .iter()
            .enumerate()
            .map(|(i, s)| (s.id.as_str(), i))
            .collect();
        if skill_index.len() != skills.len() {
            return Err(Error::new("duplicate skill id"));
        }
        let skill_array =
            |map: &HashMap<String, f32>, ctx: &str| -> Result<[f32; MAX_SKILLS], Error> {
                let mut out = [0.0; MAX_SKILLS];
                for (k, v) in map {
                    out[lookup(&skill_index, k, ctx, "skill")?] = *v;
                }
                Ok(out)
            };
        // Skill weights, strongest first (ties by skill order) so output is deterministic.
        let skill_list =
            |map: &HashMap<String, f32>, ctx: &str| -> Result<Vec<(usize, f32)>, Error> {
                let mut out = map
                    .iter()
                    .map(|(k, v)| Ok((lookup(&skill_index, k, ctx, "skill")?, *v)))
                    .collect::<Result<Vec<_>, Error>>()?;
                out.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
                Ok(out)
            };

        let ix = Indices {
            needs: &need_index,
            tags: &tag_index,
            emotions: &emotion_index,
            feelings: &feeling_index,
            skills: &skill_index,
        };

        let (cooking, items, mut recipes) = crate::cooking::parse(
            &raw.cooking,
            &raw.items,
            &raw.recipes,
            &crate::cooking::Lookups {
                needs: &need_index,
                tags: &tag_index,
                feelings: &feeling_index,
                animations: &animations,
            },
        )?;

        let lamp_tiles = raw.room_rules.lamp_tiles.unwrap_or(10.0).max(0.1);
        // Who may use what (`stages` rules): life stages by id, a bit each. Without stages in
        // content, everyone may use everything.
        let life_stages = &raw.life.stages;
        if life_stages.len() > MAX_STAGES {
            return Err(Error::new(format!("life: at most {MAX_STAGES} stages")));
        }
        let every_stage: StageMask = if life_stages.is_empty() {
            StageMask::MAX
        } else {
            StageMask::MAX >> (MAX_STAGES - life_stages.len())
        };
        let babies: StageMask = life_stages
            .iter()
            .enumerate()
            .filter(|(_, s)| s.baby)
            .fold(0, |m, (i, _)| m | 1 << i);
        let stage_ids: Vec<&str> = life_stages.iter().map(|s| s.id.as_str()).collect();
        let stages_of = |ids: &Option<Vec<String>>, ctx: &str| stage_mask(&stage_ids, ids, ctx);
        let recipe_stages = raw
            .recipes
            .iter()
            .map(|r| stages_of(&r.stages, &format!("recipe '{}'", r.id)))
            .collect::<Result<Vec<_>, _>>()?;
        let mut objects = Vec::with_capacity(raw.objects.len());
        let mut object_index = HashMap::new();
        let mut trail_walks = Vec::new();
        for obj in &raw.objects {
            if obj.footprint[0] == 0 || obj.footprint[1] == 0 {
                return Err(Error::new(format!(
                    "object '{}' has an empty footprint",
                    obj.id
                )));
            }
            // (Nothing bigger can be bought, see `World::check_size`.)
            if obj.footprint.iter().any(|&f| f > crate::world::MAX_SIZE) {
                return Err(Error::new(format!(
                    "object '{}': a footprint is at most {MAX} × {MAX} tiles",
                    obj.id,
                    MAX = crate::world::MAX_SIZE
                )));
            }
            if let Some((i, it)) = obj
                .interactions
                .iter()
                .enumerate()
                .find(|(i, it)| obj.interactions[..*i].iter().any(|o| o.id == it.id))
            {
                return Err(Error::new(format!(
                    "object '{}': interaction {i} repeats the id '{}'",
                    obj.id, it.id
                )));
            }
            let object_stages = stages_of(&obj.stages, &format!("object '{}'", obj.id))?;
            let mut interactions = Vec::with_capacity(obj.interactions.len());
            for it in &obj.interactions {
                let ctx = format!("{}.{}", obj.id, it.id);
                let recipe = it
                    .recipe
                    .as_deref()
                    .map(|r| {
                        recipes
                            .iter()
                            .position(|d| d.id == r)
                            .ok_or_else(|| Error::new(format!("{ctx}: unknown recipe '{r}'")))
                    })
                    .transpose()?;
                // Its own stages, else its recipe's, else its object's; a baby's place is a baby's.
                let stages = stages_of(&it.stages, &ctx)?
                    .or(recipe.and_then(|r| recipe_stages.get(r).copied().flatten()))
                    .or(object_stages);
                if it.baby && stages.is_some_and(|s| s & !babies != 0) {
                    return Err(Error::new(format!(
                        "{ctx}: a `baby` use is a baby's only: no other `stages`"
                    )));
                }
                let stages = stages.unwrap_or(if !it.baby {
                    every_stage & !babies
                } else if babies != 0 {
                    babies
                } else {
                    every_stage
                });
                // A recipe takes as long as its steps.
                let minutes = recipe.map_or(it.minutes, |r| recipes[r].minutes);
                if !(minutes > 0.0 && minutes.is_finite()) {
                    return Err(Error::new(format!("{ctx}: minutes must be > 0")));
                }
                let mut own_gain = [0.0; MAX_NEEDS];
                for (need, gain) in &it.effects {
                    own_gain[lookup(&need_index, need, &ctx, "need")?] = *gain;
                }
                // With a recipe, residents weigh it by what a serving gives, plus what cooking
                // itself does for the cook.
                let mut total_gain = own_gain;
                if let Some(r) = recipe {
                    for (t, e) in total_gain.iter_mut().zip(recipes[r].effects) {
                        *t += e;
                    }
                }
                // Food eaten sitting down: what it gives comes from eating it (a hidden recipe
                // of its own), and fetching it gives nothing but its costs. Residents weigh it
                // all the same.
                let eats = match &it.eat {
                    Some(_) if recipe.is_some() => {
                        return Err(Error::new(format!(
                            "{ctx}: `eat` and `recipe` don't go together"
                        )));
                    }
                    Some(_) if it.yields.is_some() => {
                        return Err(Error::new(format!(
                            "{ctx}: `eat` and `yields` don't go together"
                        )));
                    }
                    Some(e) => {
                        let mut eaten = [0.0; MAX_NEEDS];
                        for (own, eat) in own_gain
                            .iter_mut()
                            .zip(eaten.iter_mut())
                            .filter(|(g, _)| **g > 0.0)
                        {
                            *eat = std::mem::take(own);
                        }
                        recipes.push(crate::cooking::eat_recipe(
                            &ctx, &it.label, &items, &e.item, e.minutes, eaten,
                        )?);
                        Some(recipes.len() - 1)
                    }
                    None => None,
                };
                // A harvest's produce: what it gives comes from eating a serving of it later (as
                // `eat`), and residents weigh it all the same.
                let yields = match &it.yields {
                    Some(_) if recipe.is_some() => {
                        return Err(Error::new(format!(
                            "{ctx}: `yields` and `recipe` don't go together"
                        )));
                    }
                    Some(y) if y.servings == 0 => {
                        return Err(Error::new(format!(
                            "{ctx}: yields: servings must be at least 1"
                        )));
                    }
                    Some(y) => {
                        let mut eaten = [0.0; MAX_NEEDS];
                        for (own, eat) in own_gain
                            .iter_mut()
                            .zip(eaten.iter_mut())
                            .filter(|(g, _)| **g > 0.0)
                        {
                            *eat = std::mem::take(own);
                        }
                        recipes.push(crate::cooking::eat_recipe(
                            &ctx, &it.label, &items, &y.item, y.minutes, eaten,
                        )?);
                        Some(Yield {
                            recipe: recipes.len() - 1,
                            servings: y.servings,
                        })
                    }
                    None => None,
                };
                if !(0.0..=1.0).contains(&it.tends) {
                    return Err(Error::new(format!("{ctx}: tends must be 0..=1")));
                }
                if it.tends > 0.0 && obj.grows.is_none() {
                    return Err(Error::new(format!(
                        "{ctx}: `tends` brings a crop on: the object needs `grows`"
                    )));
                }
                if !(0.0..=1.0).contains(&it.waters) || (it.waters > 0.0 && !obj.outdoors) {
                    return Err(Error::new(format!(
                        "{ctx}: `waters` (0..=1) waters the garden: the object stands `outdoors`"
                    )));
                }
                if it.post
                    && (it.when != Some(OpenState::Open) || it.leaves != Some(OpenState::Closed))
                {
                    return Err(Error::new(format!(
                        "{ctx}: `post` is fetched while the post is in: \"when\": \"open\", \"leaves\": \"closed\""
                    )));
                }
                if it.carried
                    && (it.care.values().all(|&g| g <= 0.0)
                        || !obj.interactions.iter().any(|i| i.baby))
                {
                    return Err(Error::new(format!(
                        "{ctx}: `carried` is for a baby's care (`care`) where a baby has a place (`baby`)"
                    )));
                }
                // An item only shown in the arms (`brings`, `holds`).
                let shown_item = |id: &Option<String>, key: &str| -> Result<Option<usize>, Error> {
                    id.as_ref()
                        .map(|id| {
                            items.iter().position(|i| &i.id == id).ok_or_else(|| {
                                Error::new(format!("{ctx}: {key}: unknown item '{id}'"))
                            })
                        })
                        .transpose()
                };
                let after = match it.after.as_deref() {
                    Some(t) => 1 << lookup(&tag_index, t, &ctx, "tag")?,
                    None => 0,
                };
                let mut care = [0.0; MAX_NEEDS];
                for (need, gain) in &it.care {
                    care[lookup(&need_index, need, &ctx, "need")?] = gain.max(0.0);
                }
                let mut mask: TagMask = 0;
                for t in &it.tags {
                    mask |= 1 << tag_index[t.as_str()];
                }
                let mut requires = 0;
                for t in &it.requires {
                    requires |= 1 << tag_index[t.as_str()];
                }
                if it.cost < 0 {
                    return Err(Error::new(format!("{ctx}: negative cost")));
                }
                if it.seat_near < 0.0 {
                    return Err(Error::new(format!("{ctx}: negative seatNear")));
                }
                let seat_facing = match it.seat_facing.as_deref() {
                    None => None,
                    Some("front") => Some(SeatFacing::Front),
                    Some("around") => Some(SeatFacing::Around),
                    Some(other) => {
                        return Err(Error::new(format!(
                            "{ctx}: unknown seatFacing '{other}' (front or around)"
                        )));
                    }
                };
                if seat_facing.is_some() && it.seat_near <= 0.0 {
                    return Err(Error::new(format!("{ctx}: seatFacing needs a seatNear")));
                }
                if !(0.0..=1.0).contains(&it.broadcast) {
                    return Err(Error::new(format!("{ctx}: broadcast must be 0..=1")));
                }
                if it.broadcast_range.is_some_and(|r| r < 0.0) {
                    return Err(Error::new(format!("{ctx}: negative broadcastRange")));
                }
                let skill = it
                    .skill
                    .as_deref()
                    .map(|s| lookup(&skill_index, s, &ctx, "skill"))
                    .transpose()?;
                let feeling = it
                    .feeling
                    .as_deref()
                    .map(|m| {
                        feeling_index
                            .get(m)
                            .copied()
                            .ok_or_else(|| Error::new(format!("{ctx}: unknown feeling '{m}'")))
                    })
                    .transpose()?;
                if it.feeling_min_skill > 0.0 && (skill.is_none() || feeling.is_none()) {
                    return Err(Error::new(format!(
                        "{ctx}: feelingMinSkill needs a skill and a feeling"
                    )));
                }
                let anim = match it.anim.as_deref() {
                    Some(a) => Some(anim_index(a).ok_or_else(|| {
                        Error::new(format!(
                            "{ctx}: unknown anim '{a}' (declare it in \"animations\")"
                        ))
                    })?),
                    None => anim_index(default_anim(it)),
                };
                interactions.push(Interaction {
                    id: it.id.clone(),
                    label: it.label.clone(),
                    minutes,
                    total_gain,
                    gain_per_minute: own_gain.map(|g| g / minutes),
                    pose: it.pose,
                    seat_near: it.seat_near,
                    seat_facing,
                    broadcast: it.broadcast,
                    broadcast_range: it.broadcast_range.unwrap_or(6.0),
                    inside: it.inside,
                    autonomous: it.autonomous,
                    tags: mask,
                    requires,
                    skill_gain: skill_array(&it.skills, &ctx)?,
                    // A recipe costs its ingredients (for its fewest servings; it charges for as
                    // many as it makes).
                    cost: recipe.map_or(it.cost, |r| recipes[r].cost(recipes[r].servings[0])),
                    skill,
                    feeling,
                    feeling_min_skill: it.feeling_min_skill,
                    anim,
                    dirt: it.dirt.unwrap_or(-1.0),
                    baby: it.baby,
                    needs_table: it.needs_table,
                    whole: it.whole,
                    homework: it.homework.max(0.0),
                    stages,
                    care,
                    recipe,
                    after,
                    eats,
                    when: match it.when {
                        Some(OpenState::Open) => Some(true),
                        Some(OpenState::Closed) => Some(false),
                        _ => None,
                    },
                    leaves: match it.leaves {
                        Some(OpenState::Open) => Some(true),
                        Some(OpenState::Closed) => Some(false),
                        _ => None,
                    },
                    close_after: it.close_after,
                    ripe: it.when == Some(OpenState::Ripe),
                    picks: it.leaves == Some(OpenState::Picked),
                    gradual: it.gradual,
                    fetch: it.fetch.clone(),
                    changes_outfit: it.changes_outfit,
                    lasts: it.lasts,
                    spare: it.spare,
                    tends: it.tends,
                    waters: it.waters,
                    post: it.post,
                    letter: it.letter,
                    yields,
                    carried: it.carried,
                    at_slot: it.at_slot,
                    hours: it.hours,
                    sky: it.sky,
                    work_on: it.work_on,
                    partner: it.partner,
                    alone: None,
                    loads: it.loads,
                    unloads: it.unloads,
                    makes: it.makes,
                    brings: shown_item(&it.brings, "brings")?,
                    holds: shown_item(&it.holds, "holds")?,
                    private: it.private || it.conceive,
                    conceive: it.conceive,
                    served: it
                        .self_serve
                        .as_ref()
                        .map(|r| crate::staff::self_serve(r, &anim_index, &ctx))
                        .transpose()?,
                    cleans: it.cleans.as_ref().map(|c| Sweep {
                        amount: c.amount,
                        reach: c.reach,
                        hard: c.hard_floors,
                        outdoors: c.outdoors,
                    }),
                });
                // A game for two: at an object with two places, one each, used there as any use.
                if it.partner
                    && (obj.slots != 2
                        || obj.interactions.iter().any(|i| i.baby)
                        || it.whole
                        || it.inside
                        || it.seat_near > 0.0
                        || it.recipe.is_some()
                        || it.eat.is_some()
                        || it.yields.is_some()
                        || it.carried
                        || it.needs_table)
                {
                    return Err(Error::new(format!(
                        "{ctx}: a game for two (`partner`) is played at an object with two places (`slots`: 2), one each (no whole, inside, seatNear, recipe, eat, yields, carried or needsTable)"
                    )));
                }
                // Trying for a baby: two in a bed, started by the social, nothing else.
                if it.conceive
                    && (!it.partner || it.autonomous || it.pose != Pose::Lie || it.alone.is_some())
                {
                    return Err(Error::new(format!(
                        "{ctx}: `conceive` is a use for two lying down (`partner`, `\"pose\": \"lie\"`), not chosen on its own (`\"autonomous\": false`), with no `alone`"
                    )));
                }
                // Cleaning the floor: done standing at the dirty spot, the tool in the hands.
                if let Some(c) = &it.cleans
                    && (!(c.amount > 0.0 && c.amount <= 1.0)
                        || it.pose != Pose::Stand
                        || it.inside
                        || it.whole
                        || it.partner
                        || it.seat_near > 0.0
                        || it.recipe.is_some()
                        || it.eat.is_some()
                        || it.yields.is_some()
                        || it.carried
                        || it.at_slot
                        || it.needs_table
                        || it.work_on.is_some()
                        || it.loads
                        || it.unloads
                        || it.makes.is_some()
                        || it.leaves.is_some())
                {
                    return Err(Error::new(format!(
                        "{ctx}: a use that `cleans` (an `amount` above 0, at most 1) is done standing at the dirty floor, the tool in the hands: no other pose, inside, whole, partner, seatNear, recipe, eat, yields, carried, atSlot, needsTable, workOn, loads, unloads, makes or leaves"
                    )));
                }
                if it.alone.is_some() && !it.partner {
                    return Err(Error::new(format!(
                        "{ctx}: `alone` is what's done with nobody to play: it needs `partner`"
                    )));
                }
                if it.fetch.is_some() && it.requires.is_empty() {
                    return Err(Error::new(format!(
                        "{ctx}: `fetch` fetches what it `requires`: it requires nothing"
                    )));
                }
                if it.gradual && !matches!(it.leaves, Some(OpenState::Open | OpenState::Closed)) {
                    return Err(Error::new(format!(
                        "{ctx}: `gradual` is for a use that leaves it open or closed"
                    )));
                }
                if let Some(h) = it.hours
                    && (!h.iter().all(|v| (0.0..=24.0).contains(v)) || h[0] == h[1])
                {
                    return Err(Error::new(format!(
                        "{ctx}: `hours` is [from, to], two different hours of the day (0 to 24)"
                    )));
                }
                if matches!(
                    it.work_on,
                    Some(WorkOn::Laundry | WorkOn::Washed | WorkOn::Dried)
                ) && raw.laundry.is_none()
                {
                    return Err(Error::new(format!(
                        "{ctx}: `workOn` laundry needs content `laundry` (how the washing piles up)"
                    )));
                }
                match it.makes {
                    Some(WorkOn::Dishes) => {
                        return Err(Error::new(format!(
                            "{ctx}: `makes` is a load of washing: laundry, washed, dried or ironed"
                        )));
                    }
                    Some(_) if raw.laundry.is_none() => {
                        return Err(Error::new(format!(
                            "{ctx}: `makes` needs content `laundry` (how the washing piles up)"
                        )));
                    }
                    Some(WorkOn::Ironed)
                        if raw.laundry.as_ref().is_none_or(|l| l.pressed.is_none()) =>
                    {
                        return Err(Error::new(format!(
                            "{ctx}: `makes` ironed needs content `laundry.pressed` (what pressed clothes do)"
                        )));
                    }
                    _ => {}
                }
                if it.work_on == Some(WorkOn::Ironed) {
                    return Err(Error::new(format!(
                        "{ctx}: nothing works on ironed washing (`workOn`): it's worn"
                    )));
                }
                if obj.rack > 0 && it.loads {
                    return Err(Error::new(format!(
                        "{ctx}: a machine with a `rack` is filled by washing up at it, not by a use that `loads`"
                    )));
                }
                if (it.loads || it.unloads) && obj.runs.is_none() {
                    return Err(Error::new(format!(
                        "{ctx}: `loads` and `unloads` are a machine's: give the object `runs`"
                    )));
                }
                if it.loads && it.unloads {
                    return Err(Error::new(format!(
                        "{ctx}: a use `loads` the machine or `unloads` it, not both"
                    )));
                }
                if it.close_after
                    && (it.leaves != Some(OpenState::Open)
                        || !obj
                            .interactions
                            .iter()
                            .any(|i| i.leaves == Some(OpenState::Closed)))
                {
                    return Err(Error::new(format!(
                        "{ctx}: `closeAfter` is for a use that leaves it open, on an object with a use that leaves it closed"
                    )));
                }
                if it.when == Some(OpenState::Picked) || it.leaves == Some(OpenState::Ripe) {
                    return Err(Error::new(format!(
                        "{ctx}: `when` takes open, closed or ripe; `leaves` open, closed or picked"
                    )));
                }
                if (it.when == Some(OpenState::Ripe) || it.leaves == Some(OpenState::Picked))
                    && obj.grows.is_none()
                {
                    return Err(Error::new(format!(
                        "{ctx}: \"ripe\" and \"picked\" are a crop's: give the object `grows`"
                    )));
                }
            }
            // A game's use for one (`alone`): another of this object's, not a game, done where
            // the game is (the same pose, so the place they're going to stays theirs).
            for (k, it) in obj.interactions.iter().enumerate() {
                let Some(alone) = &it.alone else { continue };
                let ctx = format!("{}.{}", obj.id, it.id);
                let Some(a) = interactions.iter().position(|i| &i.id == alone) else {
                    return Err(Error::new(format!(
                        "{ctx}: `alone` names no use of '{}': '{alone}'",
                        obj.id
                    )));
                };
                if interactions[a].partner || interactions[a].pose != interactions[k].pose {
                    return Err(Error::new(format!(
                        "{ctx}: `alone` is a use for one (no `partner`), done in the same pose as the game"
                    )));
                }
                interactions[k].alone = Some(a);
            }
            if object_index.insert(obj.id.clone(), objects.len()).is_some() {
                return Err(Error::new(format!("duplicate object id '{}'", obj.id)));
            }
            if obj.slots == 0 || obj.slots as usize > MAX_SLOTS {
                return Err(Error::new(format!(
                    "object '{}': slots must be 1..={MAX_SLOTS}",
                    obj.id
                )));
            }
            if obj.price.is_some_and(|p| p < 0) {
                return Err(Error::new(format!("object '{}': negative price", obj.id)));
            }
            // (A spare bed's night doesn't make its room a bedroom.)
            let offered = interactions
                .iter()
                .filter(|it| !it.spare)
                .fold(0, |m: TagMask, it| m | it.tags);
            let mut provides = 0;
            for t in &obj.provides {
                provides |= 1 << tag_index[t.as_str()];
            }
            let used = !interactions.is_empty();
            let layer = match obj.layer.as_deref() {
                None | Some("floor") => Layer::Floor,
                Some("wall") => Layer::Wall,
                Some("ceiling") => Layer::Ceiling,
                Some("rug") => Layer::Rug,
                Some("surface") => Layer::Surface,
                Some(other) => {
                    return Err(Error::new(format!(
                        "object '{}': unknown layer '{other}' (floor, wall, ceiling, rug or surface)",
                        obj.id
                    )));
                }
            };
            if obj.hangs_low && obj.head_height {
                return Err(Error::new(format!(
                    "object '{}': `hangsLow` or `headHeight`, not both",
                    obj.id
                )));
            }
            for (set, flag, on) in [
                (obj.over_window, "overWindow", Layer::Wall),
                (obj.fills_wall, "fillsWall", Layer::Floor),
                (obj.hangs_low, "hangsLow", Layer::Ceiling),
                (obj.head_height, "headHeight", Layer::Ceiling),
                (obj.against_wall, "againstWall", Layer::Floor),
            ] {
                if set && layer != on {
                    let on = match on {
                        Layer::Wall => "a wall",
                        Layer::Floor => "the floor",
                        _ => "the ceiling",
                    };
                    return Err(Error::new(format!(
                        "object '{}': `{flag}` is for things on {on}",
                        obj.id
                    )));
                }
            }
            if !obj.overhang.is_empty()
                && (layer != Layer::Floor
                    || obj.footprint != [1, 1]
                    || obj
                        .overhang
                        .iter()
                        .any(|&[r, f]| (r, f) == (0, 0) || r.abs() > 2 || f.abs() > 2))
            {
                return Err(Error::new(format!(
                    "object '{}': `overhang` is for a 1×1 thing on the floor, over tiles up to 2 from its own",
                    obj.id
                )));
            }
            if layer == Layer::Surface && (obj.footprint != [1, 1] || obj.slots != 1 || obj.stairs)
            {
                return Err(Error::new(format!(
                    "object '{}': things on a surface are 1×1, with one slot",
                    obj.id
                )));
            }
            // A thing on a top is used from beside its top (`surface::rider_tiles`), standing, at
            // a seat pulled up to the top or on one near (`seatNear`): not lain on, stood in,
            // cooked at as a station or dined at.
            if layer == Layer::Surface
                && let Some(it) = interactions.iter().find(|i| {
                    i.pose == Pose::Lie
                        || i.inside
                        || i.whole
                        || i.baby
                        || i.cares()
                        || i.needs_table
                        || i.recipe.is_some()
                })
            {
                return Err(Error::new(format!(
                    "{}.{}: a thing on a surface is used standing or sitting at its top (no lie, inside, whole, baby, care, needsTable or recipe)",
                    obj.id, it.id
                )));
            }
            let surface = match &obj.surface {
                None => None,
                Some(_) if layer != Layer::Floor => {
                    return Err(Error::new(format!(
                        "object '{}': only furniture on the floor has a surface",
                        obj.id
                    )));
                }
                Some(s) => Some(Surface {
                    height: s.height.max(0.0),
                    kind: SurfaceKind::parse(&s.kind).ok_or_else(|| {
                        Error::new(format!(
                            "object '{}': unknown surface kind '{}' (table, bar, counter, desk, low or shelf)",
                            obj.id, s.kind
                        ))
                    })?,
                    inset: s.inset.max(0.0),
                    size: s
                        .size
                        .unwrap_or(obj.footprint.map(|f| f as f32))
                        .map(|v| v.max(0.0)),
                    at: s.at,
                }),
            };
            let approach = match obj.approach.as_deref() {
                None | Some("front") => Approach::Front,
                Some("around") => Approach::Around,
                Some("side") => Approach::Side,
                Some("behind") => Approach::Behind,
                Some("door") => Approach::Door,
                Some(other) => {
                    return Err(Error::new(format!(
                        "object '{}': unknown approach '{other}' (front, around, side, behind or door)",
                        obj.id
                    )));
                }
            };
            let mut seat_at = 0;
            for k in &obj.seat_at {
                seat_at |= SurfaceKind::parse(k)
                    .ok_or_else(|| {
                        Error::new(format!("object '{}': unknown seatAt kind '{k}'", obj.id))
                    })?
                    .bit();
            }
            if interactions.iter().any(|i| i.needs_table) && seat_at == 0 {
                return Err(Error::new(format!(
                    "object '{}': needsTable interactions need a seatAt",
                    obj.id
                )));
            }
            let mut stations = 0;
            for name in &obj.stations {
                stations |= cooking.station(name).ok_or_else(|| {
                    Error::new(format!(
                        "object '{}': unknown station '{name}' (see cooking.stations)",
                        obj.id
                    ))
                })?;
            }
            if obj.bunk && (obj.slots != 2 || layer.mounted()) {
                return Err(Error::new(format!(
                    "object '{}': a bunk bed has 2 slots and stands on the floor",
                    obj.id
                )));
            }
            let vehicle = match &obj.vehicle {
                None => None,
                Some(v) => {
                    if !(1.0..=10.0).contains(&v.speed) || v.min_ride.is_nan() || v.min_ride < 0.0 {
                        return Err(Error::new(format!(
                            "object '{}': a vehicle's speed is 1 to 10 times walking, and its minRide 0 or more",
                            obj.id
                        )));
                    }
                    let reach = v.reach.unwrap_or([crate::cars::BODY; 3]);
                    if !reach.iter().all(|r| (0.0..=3.0).contains(r)) {
                        return Err(Error::new(format!(
                            "object '{}': a vehicle's reach is 0 to 3 m each way",
                            obj.id
                        )));
                    }
                    Some(Vehicle {
                        kind: v.kind,
                        pace: v.speed * crate::DRIVE_SCALE,
                        stages: stages_of(&v.stages, &format!("object '{}' vehicle", obj.id))?
                            .unwrap_or(every_stage),
                        min_ride: v.min_ride,
                        roads_only: v.roads_only,
                        reach,
                    })
                }
            };
            let upper_stages = stages_of(
                &obj.upper_stages,
                &format!("object '{}' upperStages", obj.id),
            )?;
            if upper_stages.is_some() && !obj.bunk {
                return Err(Error::new(format!(
                    "object '{}': `upperStages` is a bunk bed's",
                    obj.id
                )));
            }
            if obj.stairs && layer.mounted() {
                return Err(Error::new(format!(
                    "object '{}': stairs stand on the floor",
                    obj.id
                )));
            }
            // A spiral winds round a square footprint; a flight is a straight column of tiles. Only a
            // stair has a shape.
            let stair_shape = match obj.stair.as_deref() {
                None => StairShape::Flight,
                Some(shape) => {
                    if !obj.stairs {
                        return Err(Error::new(format!(
                            "object '{}': `stair` is a stair's",
                            obj.id
                        )));
                    }
                    match shape {
                        // A step is a corner of the square, so its two tiles square is the only size
                        // whose steps touch.
                        "spiral" if obj.footprint != [2, 2] => {
                            return Err(Error::new(format!(
                                "object '{}': a spiral winds round a footprint of two by two tiles",
                                obj.id
                            )));
                        }
                        "spiral" => StairShape::Spiral,
                        "flight" => StairShape::Flight,
                        other => {
                            return Err(Error::new(format!(
                                "object '{}': unknown `stair` '{other}' (flight or spiral)",
                                obj.id
                            )));
                        }
                    }
                }
            };
            if !obj.places.is_empty() {
                let half = obj.footprint.map(|f| f as f32 / 2.0 + PLACE_REACH);
                if obj.places.len() != obj.slots as usize
                    || obj.bunk
                    || !matches!(layer, Layer::Floor | Layer::Wall)
                {
                    return Err(Error::new(format!(
                        "object '{}': `places` gives one place a slot, on the floor or a wall (not for a bunk bed)",
                        obj.id
                    )));
                }
                if obj.places.iter().any(|p| {
                    !p.facing.is_finite()
                        || (0..2).any(|k| !p.at[k].is_finite() || p.at[k].abs() > half[k])
                }) {
                    return Err(Error::new(format!(
                        "object '{}': a place is more than {PLACE_REACH} m off its footprint",
                        obj.id
                    )));
                }
            }
            // Things on a wall hang square to it; things on a surface always turn.
            let turns = obj.free_rotation.unwrap_or_else(|| {
                layer == Layer::Surface
                    || layer != Layer::Wall
                        && obj.footprint == [1, 1]
                        && obj.category.as_ref().is_some_and(|c| {
                            raw.object_rules
                                .free_rotation
                                .iter()
                                .flatten()
                                .any(|f| f == c)
                        })
            });
            // (Its shade's tiles turn with it in quarter turns: turned between, it would hang over one not kept for it.)
            if turns && !obj.overhang.is_empty() {
                return Err(Error::new(format!(
                    "object '{}': `overhang` turns in quarter turns only (`freeRotation`: false)",
                    obj.id
                )));
            }
            let while_anims = match obj.light.as_ref().and_then(|l| l.get("while")) {
                Some(serde_json::Value::String(s)) => vec![s.as_str()],
                Some(serde_json::Value::Array(a)) => a.iter().filter_map(serde_json::Value::as_str).collect(),
                _ => Vec::new(),
            }
            .into_iter()
            .map(|a| {
                anim_index(a).ok_or_else(|| {
                    Error::new(format!(
                        "object '{}': light.while: unknown anim '{a}' (declare it in \"animations\")",
                        obj.id
                    ))
                })
            })
            .collect::<Result<Vec<usize>, Error>>()?;
            trail_walks.extend(
                obj.interactions
                    .iter()
                    .enumerate()
                    .filter(|(_, it)| it.trail)
                    .map(|(k, _)| (objects.len(), k)),
            );
            objects.push(ObjectDef {
                id: obj.id.clone(),
                name: obj.name.clone(),
                footprint: obj.footprint,
                interactions,
                price: obj.price,
                slots: obj.slots,
                places: obj.places.iter().map(|p| SlotPlace { at: p.at, facing: p.facing.to_radians(), pose: p.pose }).collect(),
                outdoors: obj.outdoors,
                crown: obj.crown.max(0.0),
                private: obj.private,
                stairs: obj.stairs,
                stair: stair_shape,
                turns,
                decor: obj
                    .decor
                    .or_else(|| {
                        let by = &raw.room_rules.decor_by_category;
                        obj.category.as_ref().and_then(|c| by.get(c).copied())
                    })
                    .unwrap_or(0.0)
                    .max(0.0),
                lamp: obj.light.as_ref().map_or(0.0, |l| {
                    l.get("tiles").and_then(serde_json::Value::as_f64).map_or(lamp_tiles, |t| t as f32).max(0.1)
                }),
                switch: obj.light.as_ref().is_some_and(|l| l.get("switch").and_then(serde_json::Value::as_bool) != Some(false) && l.get("while").is_none()),
                fire: obj.light.as_ref().is_some_and(|l| l.get("fire").and_then(serde_json::Value::as_bool) == Some(true) && l.get("switch").and_then(serde_json::Value::as_bool) != Some(false)),
                while_anims,
                // Things nobody uses up (scenery, plants, decor) never wear out.
                wear_per_use: if !used {
                    0.0
                } else {
                    let w = &raw.object_rules.wear;
                    obj.wear_per_use
                        .or_else(|| obj.category.as_ref().and_then(|c| w.by_category.get(c).copied()))
                        .unwrap_or(w.per_use.unwrap_or(0.004))
                        .max(0.0)
                },
                tags: offered,
                provides,
                layer,
                bunk: obj.bunk,
                upper_stages: upper_stages.unwrap_or(every_stage),
                surface,
                radius: obj.radius.unwrap_or(DEFAULT_RADIUS).clamp(0.02, 0.45),
                approach,
                seat_at,
                stations,
                stores: obj.stores,
                group: obj.group.clone().unwrap_or_default(),
                runs: match &obj.runs {
                    Some(r) if r.minutes > 0.0 && r.minutes.is_finite() => Some(r.minutes),
                    Some(_) => return Err(Error::new(format!("object '{}': `runs` needs minutes > 0", obj.id))),
                    None => None,
                },
                rack: match obj.rack {
                    0 => 0,
                    _ if obj.runs.is_none() || !obj.stations.iter().any(|s| s == "dishes") => {
                        return Err(Error::new(format!("object '{}': a `rack` is a machine's that is washed up at: give it `runs` and the `dishes` station", obj.id)));
                    }
                    n => n,
                },
                not_ripe: obj.grows.as_ref().and_then(|g| g.not_ripe.clone()),
                grows: match &obj.grows {
                    Some(g) if g.days > 0.0 && g.fallow >= 0.0 => Some(Growth { days: g.days, fallow_days: g.fallow, bought_ripe: g.bought_ripe }),
                    Some(_) => {
                        return Err(Error::new(format!("object '{}': `grows` needs days > 0 and fallow >= 0", obj.id)));
                    }
                    None => None,
                },
                over_window: obj.over_window,
                fills_wall: obj.fills_wall,
                hangs_low: obj.hangs_low,
                head_height: obj.head_height,
                against_wall: obj.against_wall,
                reach: obj.reach.max(0.0),
                footstool: obj.footstool,
                overhang: obj.overhang.iter().map(|&[r, f]| (r, f)).collect(),
                staff: obj.staff.as_ref().map(|r| crate::staff::post(r, obj.footprint, &obj.id)).transpose()?,
                vehicle,
            });
        }

        let emotions = raw
            .emotions
            .iter()
            .map(|e| {
                Ok(EmotionDef {
                    id: e.id.clone(),
                    label: e.label.clone(),
                    mods: build_modifiers(&e.effects, &ix, &e.id)?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let feelings = raw
            .feelings
            .iter()
            .map(|m| social::parse_feeling(m, &ix))
            .collect::<Result<Vec<_>, _>>()?;
        let socials = raw
            .socials
            .iter()
            .map(|s| social::parse_social(s, &ix, &stage_ids))
            .collect::<Result<Vec<_>, _>>()?;
        if socials
            .iter()
            .map(|s| s.id.as_str())
            .collect::<HashSet<_>>()
            .len()
            != socials.len()
        {
            return Err(Error::new("duplicate social id"));
        }
        for (id, p) in &raw.bond_presets {
            if !((-100.0..=100.0).contains(&p.friendship) && (-100.0..=100.0).contains(&p.romance))
            {
                return Err(Error::new(format!(
                    "bond preset '{id}': friendship and romance are -100..100"
                )));
            }
        }
        let social_rules = social::parse_rules(&raw.social_rules, &raw.bond_presets, &ix)?;
        let chats = socials
            .iter()
            .flat_map(|s| s.actor_gain.iter().chain(&s.target_gain).enumerate());
        let urgent = raw
            .autonomy
            .urgent
            .as_ref()
            .map(|list| {
                list.iter().try_fold(0u32, |m, id| {
                    Ok::<_, Error>(m | 1 << lookup(&need_index, id, "autonomy.urgent", "need")?)
                })
            })
            .transpose()?;
        let autonomy = raw.autonomy.rules(
            urgent,
            chats
                .filter(|(_, g)| **g > 0.0)
                .fold(0, |m, (n, _)| m | 1 << n),
        );

        let need_array =
            |map: &HashMap<String, f32>, ctx: &str| -> Result<[f32; MAX_NEEDS], Error> {
                let mut out = [0.0; MAX_NEEDS];
                for (k, v) in map {
                    out[lookup(&need_index, k, ctx, "need")?] = *v;
                }
                Ok(out)
            };
        let feeling = |id: &Option<String>, ctx: &str| -> Result<Option<usize>, Error> {
            id.as_deref()
                .map(|m| {
                    feeling_index
                        .get(m)
                        .copied()
                        .ok_or_else(|| Error::new(format!("{ctx}: unknown feeling '{m}'")))
                })
                .transpose()
        };
        let mut stages = Vec::with_capacity(raw.life.stages.len());
        for st in &raw.life.stages {
            let ctx = format!("life stage '{}'", st.id);
            if stages.last().is_some_and(|p: &LifeStage| st.from <= p.from) {
                return Err(Error::new(format!("{ctx}: stages go from young to old")));
            }
            stages.push(LifeStage {
                id: st.id.clone(),
                label: st.label.clone().unwrap_or_else(|| st.id.clone()),
                from: st.from,
                mods: build_modifiers(&st.effects, &ix, &ctx)?,
                feeling: feeling(&st.feeling, &ctx)?,
                adult: st.adult.unwrap_or(!st.baby && !st.school),
                baby: st.baby,
                school: st.school,
                scale: st
                    .scale
                    .filter(|s| s.is_finite() && *s > 0.0)
                    .unwrap_or(1.0),
            });
        }
        let start_age = raw.life.start_age.unwrap_or([25.0, 50.0]);
        let basic_pension = raw.life.basic_pension.unwrap_or(0);
        if basic_pension < 0 {
            return Err(Error::new(format!(
                "life.basicPension {basic_pension} must be ≥ 0"
            )));
        }
        let life = LifeRules {
            days_per_year: raw.life.days_per_year.unwrap_or(2.0).max(0.1),
            not_baby: if stages.is_empty() {
                0
            } else {
                every_stage & !babies
            },
            stages,
            start_age: [
                start_age[0].min(start_age[1]),
                start_age[0].max(start_age[1]),
            ],
            retire_at: raw.life.retire_at,
            pension: raw.life.pension.unwrap_or(0.0).max(0.0),
            basic_pension,
            retire_feeling: feeling(&raw.life.retire_feeling, "life")?,
            death: raw.life.death.as_ref().map(|d| DeathRules {
                from: d.from,
                per_year: d.per_year.clamp(0.0, 1.0),
                growth: d.growth.max(0.0),
            }),
            grief: GriefRules {
                feeling: feeling(&raw.life.grief.feeling, "life.grief")?,
                light_feeling: feeling(&raw.life.grief.light_feeling, "life.grief")?,
                close: raw.life.grief.close.unwrap_or(60.0),
                friend: raw.life.grief.friend.unwrap_or(35.0),
            },
            moving: raw.life.moving.as_ref().map(|m| MovingRules {
                hour: m.hour.unwrap_or(11.0).clamp(0.0, 23.9),
                partners: m.partners.unwrap_or(0.2).clamp(0.0, 1.0),
                romance: m.romance.unwrap_or(60.0),
                leave_home_age: m.leave_home_age.unwrap_or(23.0),
                leave_home: m.leave_home.unwrap_or(0.1).clamp(0.0, 1.0),
                leave_town_age: m.leave_town_age.unwrap_or(30.0),
                leave_town: m.leave_town.unwrap_or(0.05).clamp(0.0, 1.0),
            }),
            school: match &raw.life.school {
                None => None,
                Some(sc) => {
                    let mut skills = Vec::new();
                    for (id, w) in &sc.skills {
                        skills.push((lookup(&skill_index, id, "life.school", "skill")?, *w));
                    }
                    skills.sort_by_key(|s| s.0);
                    let g = &sc.grades;
                    Some(SchoolRules {
                        start: sc.start.clamp(0.0, 23.0),
                        hours: sc.hours.clamp(0.5, 12.0),
                        days: sc
                            .days
                            .iter()
                            .filter(|&&d| d < 7)
                            .fold(0, |m, d| m | (1 << d)),
                        skills,
                        attend: g.attend.unwrap_or(1.0),
                        mood: g.mood.unwrap_or(3.0),
                        missed: g.missed.unwrap_or(5.0),
                        start_grade: g.start.unwrap_or(60.0).clamp(0.0, 100.0),
                        good: g.good.unwrap_or(70.0),
                        good_feeling: feeling(&g.good_feeling, "life.school.grades")?,
                        poor: g.poor.unwrap_or(30.0),
                        poor_feeling: feeling(&g.poor_feeling, "life.school.grades")?,
                    })
                }
            },
            pregnancy: raw.life.pregnancy.as_ref().map(|p| PregnancyRules {
                chance: p.chance.clamp(0.0, 1.0),
                days: p.days.max(1),
                max_age: p.max_age.unwrap_or(50.0),
            }),
            adoption_cost: raw.life.adoption.as_ref().map(|a| a.cost.max(0)),
            adoption_chance: raw
                .life
                .adoption
                .as_ref()
                .map_or(0.0, |a| a.neighbours.clamp(0.0, 1.0)),
            childcare: raw.life.childcare.as_ref().map(|c| ChildcareRules {
                cost: c.cost.max(0),
                below: c.below.unwrap_or(0.45).clamp(0.0, 1.0),
            }),
            newcomers: raw.life.newcomers.as_ref().map(|n| NewcomerRules {
                hour: n.hour.unwrap_or(12.0).clamp(0.0, 23.9),
                days: n.days.unwrap_or(3.0).max(1.0),
            }),
        };
        let valid_shift = |start: f32, hours: f32, days_ok: bool| {
            (0.0..24.0).contains(&start) && hours > 0.0 && hours <= 16.0 && days_ok
        };
        let mut careers = Vec::with_capacity(raw.careers.len());
        for c in &raw.careers {
            if c.levels.is_empty() {
                return Err(Error::new(format!("career '{}' has no levels", c.id)));
            }
            let levels = c
                .levels
                .iter()
                .map(|l| {
                    if !valid_shift(l.start, l.hours, l.days.iter().all(|d| *d <= 6)) {
                        return Err(Error::new(format!(
                            "career '{}': invalid shift for '{}'",
                            c.id, l.title
                        )));
                    }
                    Ok(CareerLevel {
                        title: l.title.clone(),
                        grade: None,
                        pay: l.pay,
                        start_hour: l.start,
                        hours: l.hours,
                        days: l.days.iter().fold(0u8, |m, d| m | (1 << d)),
                        requires: skill_list(&l.requires, &c.id)?,
                        car: None,
                    })
                })
                .collect::<Result<Vec<_>, Error>>()?;
            careers.push(CareerDef {
                id: c.id.clone(),
                label: c.label.clone(),
                category: None,
                levels,
                work_effects: need_array(&c.work_effects, &c.id)?,
                skills: skill_list(&c.skills, &c.id)?,
            });
        }

        // Career categories: every track becomes a career with one level per grade.
        for g in &raw.grades {
            let what = format!("grade '{}': payPerHour", g.id);
            checked_range(g.pay_per_hour, &what, PAY_PER_HOUR)?;
        }
        let grades: Vec<GradeDef> = raw
            .grades
            .iter()
            .map(|g| GradeDef {
                id: g.id.clone(),
                label: g.label.clone(),
                pay_per_hour: g.pay_per_hour,
                skill_level: g.skill_level,
            })
            .collect();
        // The cars' bodies, and which the fields' jobs call for (`car`), grade by grade.
        if raw.car_bodies.len() > usize::from(u8::MAX) {
            return Err(Error::new("at most 255 carBodies"));
        }
        let mut car_bodies = Vec::with_capacity(raw.car_bodies.len());
        for b in &raw.car_bodies {
            let len = checked_range(b.len, &format!("car body '{}': len", b.id), CAR_LEN)?;
            let wid = checked_range(b.wid, &format!("car body '{}': wid", b.id), CAR_WID)?;
            if car_bodies.iter().any(|o: &CarBodyDef| o.id == b.id) {
                return Err(Error::new(format!("duplicate car body '{}'", b.id)));
            }
            car_bodies.push(CarBodyDef {
                id: b.id.clone(),
                label: b.label.clone(),
                len,
                wid,
            });
        }
        let ages: Vec<CarAge> = raw.grades.iter().map(|g| g.car_age).collect();
        let car_table = |what: &str, map: &HashMap<String, String>| {
            car_by_grade(what, map, &grades, &ages, &car_bodies)
        };
        let mut career_categories = Vec::with_capacity(raw.career_categories.len());
        for (ci, cat) in raw.career_categories.iter().enumerate() {
            let field_cars = cat
                .car
                .as_ref()
                .map(|m| car_table(&cat.id, m))
                .transpose()?;
            let field_pay = checked_range(
                cat.pay_factor,
                &format!("career category '{}': payFactor", cat.id),
                PAY_FACTOR,
            )?;
            let work_effects = need_array(&cat.work_effects, &cat.id)?;
            for track in &cat.tracks {
                if track.titles.is_empty() || track.titles.len() > grades.len() {
                    return Err(Error::new(format!(
                        "track '{}' needs 1..={} titles (one per grade)",
                        track.id,
                        grades.len()
                    )));
                }
                let shift = track.shift.unwrap_or(cat.shift);
                if !valid_shift(shift.start, shift.hours, shift.days.1) || shift.days.0 == 0 {
                    return Err(Error::new(format!("track '{}': invalid shift", track.id)));
                }
                let skills = skill_list(&track.skills, &track.id)?;
                let track_pay = checked_range(
                    track.pay_factor,
                    &format!("track '{}': payFactor", track.id),
                    PAY_FACTOR,
                )?;
                // Pay climbs in a straight line from the field's rate at the entry grade to the
                // track's factor at the top one: senior pay differs between tracks, entry pay never
                // does. A one-grade track has no top to reach: `top_grade` is 1 and the factor does
                // nothing to it.
                let top_grade = track.titles.len().saturating_sub(1).max(1) as f32;
                let cars = match &track.car {
                    Some(m) => Some(car_table(&track.id, m)?),
                    None => field_cars.clone(),
                };
                let levels = track
                    .titles
                    .iter()
                    .zip(&grades)
                    .enumerate()
                    .map(|(gi, (title, grade))| {
                        let step = (track_pay - 1.0) * gi as f32 / top_grade;
                        CareerLevel {
                            title: title.clone(),
                            grade: Some(gi),
                            pay: (grade.pay_per_hour * field_pay * (1.0 + step) * shift.hours)
                                .round() as i64,
                            start_hour: shift.start,
                            hours: shift.hours,
                            days: shift.days.0,
                            requires: skills
                                .iter()
                                .map(|&(s, w)| (s, (grade.skill_level * w).round()))
                                .collect(),
                            car: cars.as_ref().and_then(|c| c[gi]),
                        }
                    })
                    .collect();
                careers.push(CareerDef {
                    id: track.id.clone(),
                    label: track.label.clone(),
                    category: Some(ci),
                    levels,
                    work_effects,
                    skills,
                });
            }
            career_categories.push(CategoryDef {
                id: cat.id.clone(),
                label: cat.label.clone(),
            });
        }
        if careers
            .iter()
            .map(|c| c.id.as_str())
            .collect::<HashSet<_>>()
            .len()
            != careers.len()
        {
            return Err(Error::new("duplicate career id"));
        }
        let cr = &raw.career_rules;
        let mut workweek: Vec<WorkweekRule> = cr
            .workweek
            .iter()
            .map(|w| WorkweekRule {
                fit: w.fit,
                days: w.days,
                label: w.label.clone(),
            })
            .collect();
        workweek.sort_by(|a, b| a.fit.total_cmp(&b.fit));
        let probation_levels = checked_range(
            cr.probation_levels.unwrap_or(0.0),
            "careerRules.probationLevels",
            PROBATION,
        )?;
        let career_rules = CareerRules {
            commute_minutes: time(
                cr.commute_minutes.unwrap_or(30.0),
                "careerRules.commuteMinutes",
            )?,
            late_minutes: cr.late_minutes.unwrap_or(60.0),
            missed_penalty: cr.missed_penalty.unwrap_or(20.0),
            performance_per_shift: cr.performance_per_shift.unwrap_or(12.0),
            performance_per_fit: cr.performance_per_fit.unwrap_or(0.0),
            performance_per_mood: cr.performance_per_mood.unwrap_or(15.0),
            performance_slowdown: checked_range(
                cr.performance_slowdown.unwrap_or(0.0),
                "careerRules.performanceSlowdown",
                PERFORMANCE_SLOWDOWN,
            )?,
            late_grace_minutes: time(
                cr.late_grace_minutes.unwrap_or(0.0),
                "careerRules.lateGraceMinutes",
            )?,
            late_share: checked_range(
                cr.late_share.unwrap_or(1.0),
                "careerRules.lateShare",
                LATE_SHARE,
            )?,
            probation_levels,
            workweek,
            promotion_feeling: feeling(&cr.promotion_feeling, "careerRules")?,
            missed_feeling: feeling(&cr.missed_feeling, "careerRules")?,
            market: match &cr.market {
                None => None,
                Some(m) => Some(JobMarket {
                    hour: m.hour.unwrap_or(9.5).clamp(0.0, 23.9),
                    daily_chance: m.daily_chance.unwrap_or(0.35).clamp(0.0, 1.0),
                    quit_below: m.quit_below.unwrap_or(0.3),
                    quit_chance: m.quit_chance.unwrap_or(0.15).clamp(0.0, 1.0),
                    min_shifts_before_quit: m.min_shifts_before_quit.unwrap_or(10),
                    fire_after_missed: m.fire_after_missed.unwrap_or(3).max(1),
                    cooldown_days: m.cooldown_days.unwrap_or(2),
                    hired_feeling: feeling(&m.hired_feeling, "careerRules.market")?,
                    quit_feeling: feeling(&m.quit_feeling, "careerRules.market")?,
                    fired_feeling: feeling(&m.fired_feeling, "careerRules.market")?,
                }),
            },
        };

        let sr = &raw.skill_rules;
        let skill_rules = SkillRules {
            max_level: sr.max_level.unwrap_or(10.0),
            work_gain_per_hour: sr.work_gain_per_hour.unwrap_or(0.05),
            slowdown: sr.slowdown.unwrap_or(0.25),
            interest: sr.interest.unwrap_or(0.05),
            goal_interest: sr.goal_interest.unwrap_or(0.25),
            effect_per_level: sr.effect_per_level.unwrap_or(0.05),
            fatigue_per_hour: sr.fatigue_per_hour.unwrap_or(0.0),
            fatigue_recovery_per_hour: sr.fatigue_recovery_per_hour.unwrap_or(0.0),
        };
        let dr = &raw.day_rhythm;
        // Tags content names here must exist (a typo would quietly switch something off).
        let tag_mask = |tags: &[String], ctx: &str| -> Result<TagMask, Error> {
            tags.iter().try_fold(0, |m: TagMask, t| {
                Ok(m | (1 << lookup(&tag_index, t, ctx, "tag")?))
            })
        };
        let multipliers =
            |map: &HashMap<String, f32>, ctx: &str| -> Result<[f32; MAX_NEEDS], Error> {
                let mut out = [1.0; MAX_NEEDS];
                for (k, v) in map {
                    if !(*v >= 0.0 && v.is_finite()) {
                        return Err(Error::new(format!(
                            "{ctx}: the {k} multiplier {v} must be a number ≥ 0"
                        )));
                    }
                    out[lookup(&need_index, k, ctx, "need")?] = *v;
                }
                Ok(out)
            };
        // What's fetched (`fetch`) is fetched with an interaction of something that provides it.
        for (o, inter) in objects
            .iter()
            .flat_map(|o: &ObjectDef| o.interactions.iter().map(move |i| (o, i)))
        {
            let Some(fetch) = inter.fetch.as_deref() else {
                continue;
            };
            let source = |d: &ObjectDef| {
                d.provides & inter.requires == inter.requires
                    && d.interactions
                        .iter()
                        .any(|i| i.id == fetch && i.recipe.is_none() && i.eats.is_none())
            };
            if !objects.iter().any(source) {
                return Err(Error::new(format!(
                    "object '{}' interaction '{}': fetches with '{fetch}', which nothing that provides what it requires has (a plain use: no recipe, no eat)",
                    o.id, inter.id
                )));
            }
        }
        let sleep_tags = tag_mask(&dr.sleep_tags, "dayRhythm.sleepTags")?;
        if let Some((o, i)) = objects.iter().find_map(|o: &ObjectDef| {
            o.interactions
                .iter()
                .find(|i| i.spare && i.tags & sleep_tags == 0)
                .map(|i| (o, i))
        }) {
            return Err(Error::new(format!(
                "{}.{}: `spare` is for a night's sleep (a `dayRhythm.sleepTags` tag)",
                o.id, i.id
            )));
        }
        let mut slept = [0.0f32; MAX_NEEDS];
        for inter in objects
            .iter()
            .flat_map(|o: &ObjectDef| &o.interactions)
            .filter(|i| i.tags & sleep_tags != 0)
        {
            for (n, g) in inter.total_gain.iter().enumerate() {
                slept[n] += g.max(0.0);
            }
        }
        let day_rhythm = DayRhythm {
            wake_hour: dr.wake_hour.unwrap_or(7.0),
            bed_hour: dr.bed_hour.unwrap_or(22.5),
            sleep_tags,
            night_decay: multipliers(&dr.night_decay, "dayRhythm")?,
            asleep_decay: multipliers(&dr.asleep_decay, "dayRhythm")?,
            work_decay: multipliers(&dr.work_decay, "dayRhythm")?,
            wake_for: match &dr.wake_for {
                None => u32::MAX,
                Some(list) => list.iter().try_fold(0u32, |m, id| {
                    Ok::<_, Error>(m | 1 << lookup(&need_index, id, "dayRhythm.wakeFor", "need")?)
                })?,
            },
            wake_before_work_minutes: dr.wake_before_work_minutes.unwrap_or(75.0),
            rest_need: (0..MAX_NEEDS)
                .filter(|&n| slept[n] > 0.0)
                .max_by(|&a, &b| slept[a].total_cmp(&slept[b])),
            lie_in_hours: dr.lie_in_hours.unwrap_or(0.5),
            sleep_quality_bonus: dr.sleep_quality_bonus.unwrap_or(0.06),
            nap_tags: tag_mask(&dr.nap_tags, "dayRhythm.napTags")?,
            nap_cooldown_hours: dr.nap_cooldown_hours.unwrap_or(4.0),
            exhausted_below: dr.exhausted_below.unwrap_or(0.2),
            habits: dr
                .habits
                .iter()
                .map(|h| {
                    let need = lookup(&need_index, &h.need, "dayRhythm.habits", "need")?;
                    Ok::<_, Error>(Habit {
                        at: h.at,
                        need,
                        below: h.below.clamp(0.0, 1.0),
                    })
                })
                .collect::<Result<_, _>>()?,
            ready_minutes: dr.ready_minutes.unwrap_or(30.0).max(0.0),
            work_breaks: dr
                .work_breaks
                .iter()
                .map(|b| {
                    let need = lookup(&need_index, &b.need, "dayRhythm.workBreaks", "need")?;
                    let below = b.below.clamp(0.0, 1.0);
                    Ok::<_, Error>(WorkBreak {
                        need,
                        below,
                        gain: b.gain.max(0.0),
                        cost: b.cost.max(0),
                    })
                })
                .collect::<Result<_, _>>()?,
            baby_wakes: dr
                .baby_wakes
                .as_ref()
                .map(|b| {
                    let mask = match &b.needs {
                        None => (0..needs.len())
                            .filter(|&n| !needs[n].room)
                            .fold(0u32, |m, n| m | 1 << n),
                        Some(list) => list.iter().try_fold(0u32, |m, id| {
                            Ok::<_, Error>(
                                m | 1 << lookup(&need_index, id, "dayRhythm.babyWakes", "need")?,
                            )
                        })?,
                    };
                    let below = b.below.unwrap_or(0.3).clamp(0.0, 1.0);
                    Ok::<_, Error>(BabyWakes {
                        needs: mask,
                        below,
                        tend_below: b.tend_below.unwrap_or(0.6).clamp(below, 1.0),
                        cooldown_ticks: (time(
                            b.cooldown_minutes.unwrap_or(90.0),
                            "dayRhythm.babyWakes.cooldownMinutes",
                        )?
                        .max(0.0)
                            * crate::TICKS_PER_MINUTE as f32)
                            as u64,
                        finish_minutes: b.finish_minutes.unwrap_or(15.0).max(0.0),
                        feeling: feeling(&b.feeling, "dayRhythm.babyWakes")?,
                    })
                })
                .transpose()?,
        };
        let rent = raw
            .economy
            .rent
            .as_ref()
            .map(|r| {
                Ok::<_, Error>(RentRules {
                    weekday: r.weekday.min(6),
                    hour: r.hour.clamp(0.0, 23.9),
                    base: r.base,
                    per_tile: r.per_tile,
                    bills_base: r.bills_base,
                    bills_rate: r.bills_rate.max(0.0),
                    per_resident: r.per_resident.max(0),
                    debt_feeling: feeling(&r.debt_feeling, "economy.rent")?,
                })
            })
            .transpose()?;
        if raw.accidents.len() > MAX_ACCIDENTS {
            return Err(Error::new(format!("at most {MAX_ACCIDENTS} accidents")));
        }
        let accidents = raw
            .accidents
            .iter()
            .map(|a| {
                let ctx = format!("accident '{}'", a.id);
                Ok(AccidentDef {
                    id: a.id.clone(),
                    need: lookup(&need_index, &a.need, &ctx, "need")?,
                    story: a.story.is_some(),
                    effects: need_array(&a.effects, &ctx)?,
                    dirt: a.dirt.clamp(0.0, 1.0),
                    feeling: feeling(&a.feeling, &ctx)?,
                    cost: a.cost.max(0),
                    cooldown_ticks: (time(a.cooldown_hours.unwrap_or(2.0), &ctx)?.max(0.0) * 60.0 * crate::TICKS_PER_MINUTE as f32) as u64,
                    grace_ticks: (time(a.grace_minutes.unwrap_or(30.0), &ctx)?.max(0.0) * crate::TICKS_PER_MINUTE as f32) as u64,
                    crowded_grace_ticks: a.crowded_grace_minutes.map(|m| time(m, &ctx)).transpose()?.map(|m| (m.max(0.0) * crate::TICKS_PER_MINUTE as f32) as u64),
                    another_room: a
                        .another_room
                        .as_ref()
                        .map(|id| {
                            raw.room_kinds
                                .iter()
                                .position(|k| &k.id == id)
                                .ok_or_else(|| Error::new(format!("{ctx}: unknown room kind '{id}'")))
                        })
                        .transpose()?,
                    rest: a
                        .rest
                        .as_ref()
                        .map(|r| -> Result<AccidentRest, Error> {
                            let minutes = r.minutes.max(1.0);
                            Ok(AccidentRest {
                                label: r.label.clone(),
                                minutes,
                                pose: r.pose,
                                anim: r
                                    .anim
                                    .as_deref()
                                    .map(|n| anim_index(n).ok_or_else(|| Error::new(format!("{ctx}: unknown anim '{n}' (declare it in \"animations\")"))))
                                    .transpose()?,
                                gain_per_minute: need_array(&r.gains, &ctx)?.map(|g| g / minutes),
                                tags: tag_mask(&r.tags, &ctx)?,
                            })
                        })
                        .transpose()?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let room_kinds = raw
            .room_kinds
            .iter()
            .map(|k| {
                let ctx = format!("room kind '{}'", k.id);
                if k.claimed && k.passage {
                    return Err(Error::new(format!(
                        "{ctx}: a 'claimed' room is somebody's own: it can't be a passage (see privacy.rs)"
                    )));
                }
                if k.claimed && !k.private {
                    return Err(Error::new(format!(
                        "{ctx}: a 'claimed' room is one kept shut: it needs 'private' too (see privacy.rs)"
                    )));
                }
                Ok(RoomKind {
                    id: k.id.clone(),
                    label: k.label.clone().unwrap_or_else(|| k.id.clone()),
                    tags: tag_mask(&k.tags, &ctx)?,
                    essentials: k.essentials.iter().map(|e| tag_mask(e, &ctx)).collect::<Result<Vec<_>, Error>>()?.into_iter().filter(|&m| m != 0).collect(),
                    size: k.size.unwrap_or([4.0, 9.0]),
                    exclusive: k.exclusive,
                    passage: k.passage,
                    private: k.private,
                    claimed: k.claimed,
                    carpet: k.carpet,
                })
            })
            .collect::<Result<_, Error>>()?;
        let rr = &raw.room_rules;
        let room_rules = RoomRules {
            weights: rr.weights.unwrap_or([0.2, 0.2, 0.2, 0.25, 0.15]),
            window_tiles: rr.window_tiles.unwrap_or(6.0).max(0.1),
            lamp_tiles,
            decor_per_tile: rr.decor_per_tile.unwrap_or(0.125).max(0.001),
            garden_decor_per_tile: rr.garden_decor_per_tile.unwrap_or(0.05).max(0.001),
            love: rr.love.unwrap_or(0.75),
            dislike: rr.dislike.unwrap_or(0.4),
            drift: rr.drift.unwrap_or(0.5).max(0.0),
            away: rr.away.unwrap_or(0.6),
            preference: rr.preference.unwrap_or(1.0).max(0.0),
            love_feeling: feeling(&rr.love_feeling, "roomRules")?,
            dislike_feeling: feeling(&rr.dislike_feeling, "roomRules")?,
            dirt: {
                let mut d: Vec<(TagMask, f32)> = rr
                    .dirt
                    .iter()
                    .map(|(t, &v)| {
                        Ok((
                            tag_mask(std::slice::from_ref(t), "roomRules.dirt")?,
                            v.max(0.0),
                        ))
                    })
                    .collect::<Result<_, Error>>()?;
                d.sort_by_key(|e| e.0);
                d
            },
            clean: CleanRules {
                label: rr.clean.label.clone().unwrap_or_else(|| "Tidy up".into()),
                minutes: rr.clean.minutes.unwrap_or(12.0).max(1.0),
                threshold: rr.clean.threshold.unwrap_or(0.15),
                amount: rr.clean.amount.unwrap_or(0.7),
                interest: rr.clean.interest.unwrap_or(0.6),
                tags: tag_mask(&clean_tags, "roomRules.clean")?,
                anim: animations.iter().position(|a| a == "clean"),
            },
        };
        // Interactions without their own mess make what their tags make.
        for o in &mut objects {
            for it in &mut o.interactions {
                if it.dirt < 0.0 {
                    it.dirt = room_rules
                        .dirt
                        .iter()
                        .filter(|(m, _)| it.tags & m != 0)
                        .map(|(_, v)| *v)
                        .fold(0.0, f32::max);
                }
            }
        }
        let or = &raw.object_rules;
        let rp = &or.repair;
        let object_rules = ObjectRules {
            wear_per_quality: or.wear.per_quality.unwrap_or(0.2).clamp(0.0, 0.3),
            repair: RepairRules {
                label: rp.label.clone().unwrap_or_else(|| "Repair".into()),
                minutes: rp.minutes.unwrap_or(45.0).max(1.0),
                skill: rp
                    .skill
                    .as_deref()
                    .map(|s| lookup(&skill_index, s, "objectRules.repair", "skill"))
                    .transpose()?,
                skill_gain: rp.skill_gain_per_hour.unwrap_or(0.3),
                chance: rp.chance.unwrap_or(0.5),
                chance_per_level: rp.chance_per_level.unwrap_or(0.08),
                interest: rp.interest.unwrap_or(0.7),
                cost: rp.cost.unwrap_or(0.3),
                // (By default its chores tag, if content has one.)
                tags: match &rp.tags {
                    Some(tags) => tag_mask(tags, "objectRules.repair")?,
                    None => tag_index.get("chores").map_or(0, |&i| 1 << i),
                },
                anim: animations.iter().position(|a| a == "tinker"),
            },
            set_up: {
                let su = &or.set_up;
                let anim = |a: &Option<String>, default: &str| match a {
                    None => Ok(anim_index(default)),
                    Some(a) => anim_index(a).map(Some).ok_or_else(|| {
                        Error::new(format!(
                            "objectRules.setUp: unknown anim '{a}' (declare it in \"animations\")"
                        ))
                    }),
                };
                SetUpRules {
                    label: su.label.clone().unwrap_or_else(|| "Set up".into()),
                    minutes: su.minutes.unwrap_or(15.0).max(1.0),
                    fetch_minutes: su.fetch_minutes.unwrap_or(1.5).max(0.5),
                    help_hours: su.help_after_hours.unwrap_or(2.0).max(0.0),
                    wait_hours: su.wait_hours.unwrap_or(24.0).max(1.0),
                    parcel: match &su.parcel {
                        None => None,
                        Some(id) => {
                            Some(items.iter().position(|i| &i.id == id).ok_or_else(|| {
                                Error::new(format!("objectRules.setUp: unknown item '{id}'"))
                            })?)
                        }
                    },
                    tags: match &su.tags {
                        Some(tags) => tag_mask(tags, "objectRules.setUp")?,
                        None => tag_index.get("chores").map_or(0, |&i| 1 << i),
                    },
                    anim: anim(&su.anim, "tinker")?,
                    fetch_anim: anim(&su.fetch_anim, "serve")?,
                }
            },
            max_quality: or.max_quality.unwrap_or(3),
            quality_bonus: or.quality_bonus.unwrap_or(0.25),
            upgrade_cost: or.upgrade_cost.unwrap_or(0.3),
            resale: or.resale.unwrap_or(0.5),
        };
        // (Selling for more than was paid would be a money machine; an upgrade costs something.)
        if !(0.0..=1.0).contains(&object_rules.resale) {
            return Err(Error::new(format!(
                "objectRules.resale {} must be 0..1 (a share of what was paid)",
                object_rules.resale
            )));
        }
        if !(object_rules.upgrade_cost >= 0.0 && object_rules.upgrade_cost.is_finite()) {
            return Err(Error::new(format!(
                "objectRules.upgradeCost {} must be a number ≥ 0",
                object_rules.upgrade_cost
            )));
        }
        let build = BuildRules {
            wall: raw.build.wall.unwrap_or(40),
            door: raw.build.door.unwrap_or(150),
            window: raw.build.window.unwrap_or(120),
            remove: raw.build.remove.unwrap_or(10),
            fence: raw.build.fence.unwrap_or(25),
            gate: raw.build.gate.unwrap_or(100),
            garage: raw.build.garage.unwrap_or(2500),
            porch: raw.build.porch.unwrap_or(600),
            porch_post: raw.build.porch_post.unwrap_or(60),
            porch_canopy: raw.build.porch_canopy.unwrap_or(90),
            porch_step: raw.build.porch_step.unwrap_or(30),
            land: raw.build.land.prices()?,
            fences: build_styles(&raw.fence_styles, "fence style")?,
            roofs: build_styles(&raw.roof_styles, "roof style")?,
            roof_colors: build_styles(&raw.roof_colors, "roof colour")?,
            coverings: build_styles(&raw.wall_coverings, "wall covering")?,
            doors: build_styles(&raw.door_styles, "door style")?,
            windows: build_styles(&raw.window_styles, "window style")?,
            floors: build_styles(&raw.floor_coverings, "floor covering")?,
            storeys: raw.build.storeys.unwrap_or(1).clamp(1, 4),
            basement: raw.build.basement,
        };
        let schedule = raw
            .schedule
            .iter()
            .map(|w| {
                Ok(ScheduleWindow {
                    from: w.from,
                    to: w.to,
                    mods: build_modifiers(&w.effects, &ix, "schedule")?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        let visits = visits::parse(
            &raw.visits,
            &socials,
            1 << tag_index[VISIT_TAG],
            &need_array,
            &anim_index,
            &feeling,
        )?;

        let (traits, perks, genders) = people::parse(&raw, &ix, &skill_array)?;

        let buff_feelings = feelings.iter().any(|m| m.effects.is_some());
        let conceive_beds = objects
            .iter()
            .any(|o: &ObjectDef| o.interactions.iter().any(|i| i.conceive));
        let talk_anim = anim_index(TALK_ANIM);
        let cuddle_anim = anim_index("cuddle");
        let (activities, planner, goals) = crate::planner::parse(
            &raw.activities,
            &raw.planner,
            &raw.goals,
            &tag_index,
            &feelings,
        )?;
        let phone = crate::phone::parse(&raw.phone, &ix, &animations, &life.stages)?;
        let service = crate::staff::rules(raw.service.as_ref(), &anim_index)?;
        crate::staff::validate(&objects)?;
        let laundry = match &raw.laundry {
            Some(l)
                if l.per_day > 0.0
                    && l.per_day.is_finite()
                    && l.most >= 1.0
                    && l.most.is_finite() =>
            {
                let pressed = match &l.pressed {
                    Some(p) if p.outfits >= 1.0 && p.outfits.is_finite() => Some(Pressed {
                        feeling: feeling_index.get(&p.feeling).copied().ok_or_else(|| {
                            Error::new(format!("laundry.pressed: unknown feeling '{}'", p.feeling))
                        })?,
                        outfits: p.outfits,
                    }),
                    Some(_) => {
                        return Err(Error::new(
                            "laundry.pressed: `outfits` at least 1 (a load's worth)",
                        ));
                    }
                    None => None,
                };
                Some(LaundryRules {
                    per_day: l.per_day,
                    most: l.most,
                    pressed,
                })
            }
            Some(_) => {
                return Err(Error::new(
                    "laundry: `perDay` above 0 and `most` at least 1 (loads)",
                ));
            }
            None => None,
        };
        Ok(Self {
            needs,
            objects,
            traits,
            perks,
            genders,
            tags,
            rules: raw.rules,
            emotions,
            feelings,
            socials,
            bond_presets: raw.bond_presets,
            social_rules,
            careers,
            career_rules,
            schedule,
            visits,
            starting_funds: raw.economy.starting_funds.unwrap_or(1500),
            skills,
            skill_rules,
            grades,
            career_categories,
            car_bodies,
            day_rhythm,
            autonomy,
            rent,
            object_rules,
            room_kinds,
            room_rules,
            accidents,
            life,
            names: raw.names.clone(),
            build,
            animations,
            talk_anim,
            cuddle_anim,
            buff_feelings,
            conceive_beds,
            activities,
            planner,
            goals,
            cooking,
            items,
            recipes,
            phone,
            service,
            laundry,
            trail_walks,
            cars: CarRules {
                country_speed: match raw.cars.country_speed {
                    Some(v) if v.is_finite() && v > 0.0 => v.min(60.0),
                    Some(v) => {
                        return Err(Error::new(format!(
                            "cars.countrySpeed: {v} must be a positive number"
                        )));
                    }
                    None => crate::travel::COUNTRY_SPEED,
                },
            },
            taxi: {
                let d = TaxiRules::default();
                let rules = TaxiRules {
                    seats: raw.taxi.seats.unwrap_or(d.seats),
                    min_walk: raw.taxi.min_walk_minutes.unwrap_or(d.min_walk),
                    window: raw.taxi.window.unwrap_or(d.window),
                    wait: raw.taxi.wait_minutes.unwrap_or(d.wait),
                };
                let ok = |v: f32, most: f32| v.is_finite() && (0.0..=most).contains(&v);
                if !(1..=4).contains(&rules.seats)
                    || !ok(rules.min_walk, 600.0)
                    || !ok(rules.window, 120.0)
                    || !ok(rules.wait, 30.0)
                {
                    return Err(Error::new(
                        "taxi: seats is 1 to 4, minWalkMinutes 0 to 600, window 0 to 120 minutes and waitMinutes 0 to 30",
                    ));
                }
                rules
            },
            object_index,
        })
    }
}

mod access;
mod helpers;
mod people;
mod visits;

pub(crate) use helpers::*;

#[cfg(test)]
mod tests;

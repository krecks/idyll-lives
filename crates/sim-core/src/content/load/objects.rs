//! Objects: the content's objects, their interactions, and the way to them.

use super::*;

/// Reads skill weights by skill id.
type SkillArray<'a> = dyn Fn(&HashMap<String, f32>, &str) -> Result<[f32; MAX_SKILLS], Error> + 'a;
/// Reads which stages may use a thing (None: any).
type StagesOf<'a> = dyn Fn(&Option<Vec<String>>, &str) -> Result<Option<StageMask>, Error> + 'a;

/// What reading the objects needs: the content, what has been read so far, and the rules about
/// who may use what.
pub(super) struct Ctx<'a, 'r> {
    pub raw: &'a ContentFile,
    pub need_index: &'a HashMap<&'a str, usize>,
    pub tag_index: &'a HashMap<&'a str, usize>,
    pub anim_index: &'a (dyn Fn(&str) -> Option<usize> + 'a),
    pub feeling_index: &'a HashMap<String, usize>,
    pub skill_index: &'a HashMap<&'a str, usize>,
    pub skill_array: &'a SkillArray<'a>,
    pub lamp_tiles: f32,
    pub every_stage: StageMask,
    pub babies: StageMask,
    pub stages_of: &'a StagesOf<'a>,
    pub recipe_stages: &'a [Option<StageMask>],
    pub cooking: &'a crate::cooking::CookingRules,
    pub items: &'a [crate::cooking::ItemDef],
    pub recipes: &'r mut Vec<crate::cooking::RecipeDef>,
}

/// The objects read, the way to them by id, and the walks trails are walked.
pub(super) struct Parsed {
    pub objects: Vec<ObjectDef>,
    pub index: HashMap<String, usize>,
    pub trail_walks: Vec<(usize, usize)>,
}

pub(super) fn parse<'a, 'r>(load: &mut Ctx<'a, 'r>) -> Result<Parsed, Error> {
    let mut objects = Vec::with_capacity(load.raw.objects.len());
    let mut object_index = HashMap::new();
    let mut trail_walks = Vec::new();
    for obj in &load.raw.objects {
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
        let object_stages = (load.stages_of)(&obj.stages, &format!("object '{}'", obj.id))?;
        let mut interactions = Vec::with_capacity(obj.interactions.len());
        for it in &obj.interactions {
            let ctx = format!("{}.{}", obj.id, it.id);
            let recipe = it
                .recipe
                .as_deref()
                .map(|r| {
                    load.recipes
                        .iter()
                        .position(|d| d.id == r)
                        .ok_or_else(|| Error::new(format!("{ctx}: unknown recipe '{r}'")))
                })
                .transpose()?;
            // Its own stages, else its recipe's, else its object's; a baby's place is a baby's.
            let stages = (load.stages_of)(&it.stages, &ctx)?
                .or(recipe.and_then(|r| load.recipe_stages.get(r).copied().flatten()))
                .or(object_stages);
            if it.baby && stages.is_some_and(|s| s & !load.babies != 0) {
                return Err(Error::new(format!(
                    "{ctx}: a `baby` use is a baby's only: no other `stages`"
                )));
            }
            let stages = stages.unwrap_or(if !it.baby {
                load.every_stage & !load.babies
            } else if load.babies != 0 {
                load.babies
            } else {
                load.every_stage
            });
            // A recipe takes as long as its steps.
            let minutes = recipe.map_or(it.minutes, |r| load.recipes[r].minutes);
            if !(minutes > 0.0 && minutes.is_finite()) {
                return Err(Error::new(format!("{ctx}: minutes must be > 0")));
            }
            let mut own_gain = [0.0; MAX_NEEDS];
            for (need, gain) in &it.effects {
                own_gain[lookup(load.need_index, need, &ctx, "need")?] = *gain;
            }
            // With a recipe, residents weigh it by what a serving gives, plus what load.cooking
            // itself does for the cook.
            let mut total_gain = own_gain;
            if let Some(r) = recipe {
                for (t, e) in total_gain.iter_mut().zip(load.recipes[r].effects) {
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
                    load.recipes.push(crate::cooking::eat_recipe(
                        &ctx, &it.label, load.items, &e.item, e.minutes, eaten,
                    )?);
                    Some(load.recipes.len() - 1)
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
                    load.recipes.push(crate::cooking::eat_recipe(
                        &ctx, &it.label, load.items, &y.item, y.minutes, eaten,
                    )?);
                    Some(Yield {
                        recipe: load.recipes.len() - 1,
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
            if it.post && (it.when != Some(OpenState::Open) || it.leaves != Some(OpenState::Closed))
            {
                return Err(Error::new(format!(
                    "{ctx}: `post` is fetched while the post is in: \"when\": \"open\", \"leaves\": \"closed\""
                )));
            }
            if it.carried
                && (it.care.values().all(|&g| g <= 0.0) || !obj.interactions.iter().any(|i| i.baby))
            {
                return Err(Error::new(format!(
                    "{ctx}: `carried` is for a baby's care (`care`) where a baby has a place (`baby`)"
                )));
            }
            // An item only shown in the arms (`brings`, `holds`).
            let shown_item = |id: &Option<String>, key: &str| -> Result<Option<usize>, Error> {
                id.as_ref()
                    .map(|id| {
                        load.items
                            .iter()
                            .position(|i| &i.id == id)
                            .ok_or_else(|| Error::new(format!("{ctx}: {key}: unknown item '{id}'")))
                    })
                    .transpose()
            };
            let after = match it.after.as_deref() {
                Some(t) => 1 << lookup(load.tag_index, t, &ctx, "tag")?,
                None => 0,
            };
            let mut care = [0.0; MAX_NEEDS];
            for (need, gain) in &it.care {
                care[lookup(load.need_index, need, &ctx, "need")?] = gain.max(0.0);
            }
            let mut mask: TagMask = 0;
            for t in &it.tags {
                mask |= 1 << load.tag_index[t.as_str()];
            }
            let mut requires = 0;
            for t in &it.requires {
                requires |= 1 << load.tag_index[t.as_str()];
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
                .map(|s| lookup(load.skill_index, s, &ctx, "skill"))
                .transpose()?;
            let feeling = it
                .feeling
                .as_deref()
                .map(|m| {
                    load.feeling_index
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
                Some(a) => Some((load.anim_index)(a).ok_or_else(|| {
                    Error::new(format!(
                        "{ctx}: unknown anim '{a}' (declare it in \"animations\")"
                    ))
                })?),
                None => (load.anim_index)(default_anim(it)),
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
                skill_gain: (load.skill_array)(&it.skills, &ctx)?,
                // A recipe costs its ingredients (for its fewest servings; it charges for as
                // many as it makes).
                cost: recipe.map_or(it.cost, |r| {
                    load.recipes[r].cost(load.recipes[r].servings[0])
                }),
                public_cost: it.public_cost,
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
                    .map(|r| crate::staff::self_serve(r, &(load.anim_index), &ctx))
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
            ) && load.raw.laundry.is_none()
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
                Some(_) if load.raw.laundry.is_none() => {
                    return Err(Error::new(format!(
                        "{ctx}: `makes` needs content `laundry` (how the washing piles up)"
                    )));
                }
                Some(WorkOn::Ironed)
                    if load
                        .raw
                        .laundry
                        .as_ref()
                        .is_none_or(|l| l.pressed.is_none()) =>
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
            provides |= 1 << load.tag_index[t.as_str()];
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
        if layer == Layer::Surface && (obj.footprint != [1, 1] || obj.slots != 1 || obj.stairs) {
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
            stations |= load.cooking.station(name).ok_or_else(|| {
                Error::new(format!(
                    "object '{}': unknown station '{name}' (see load.cooking.stations)",
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
                    stages: (load.stages_of)(&v.stages, &format!("object '{}' vehicle", obj.id))?
                        .unwrap_or(load.every_stage),
                    min_ride: v.min_ride,
                    roads_only: v.roads_only,
                    reach,
                })
            }
        };
        let upper_stages = (load.stages_of)(
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
                        load.raw
                            .object_rules
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
            Some(serde_json::Value::Array(a)) => {
                a.iter().filter_map(serde_json::Value::as_str).collect()
            }
            _ => Vec::new(),
        }
        .into_iter()
        .map(|a| {
            (load.anim_index)(a).ok_or_else(|| {
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
                    let by = &load.raw.room_rules.decor_by_category;
                    obj.category.as_ref().and_then(|c| by.get(c).copied())
                })
                .unwrap_or(0.0)
                .max(0.0),
            lamp: obj.light.as_ref().map_or(0.0, |l| {
                l.get("tiles").and_then(serde_json::Value::as_f64).map_or(load.lamp_tiles, |t| t as f32).max(0.1)
            }),
            switch: obj.light.as_ref().is_some_and(|l| l.get("switch").and_then(serde_json::Value::as_bool) != Some(false) && l.get("while").is_none()),
            fire: obj.light.as_ref().is_some_and(|l| l.get("fire").and_then(serde_json::Value::as_bool) == Some(true) && l.get("switch").and_then(serde_json::Value::as_bool) != Some(false)),
            while_anims,
            // Things nobody uses up (scenery, plants, decor) never wear out.
            wear_per_use: if !used {
                0.0
            } else {
                let w = &load.raw.object_rules.wear;
                obj.wear_per_use
                    .or_else(|| obj.category.as_ref().and_then(|c| w.by_category.get(c).copied()))
                    .unwrap_or(w.per_use.unwrap_or(0.004))
                    .max(0.0)
            },
            tags: offered,
            provides,
            layer,
            bunk: obj.bunk,
            upper_stages: upper_stages.unwrap_or(load.every_stage),
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

    Ok(Parsed {
        objects,
        index: object_index,
        trail_walks,
    })
}

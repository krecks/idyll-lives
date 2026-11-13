//! What a resident chooses to do on their own, and how a task is started.
//!
//! Split from `world.rs` (see `mod.rs`).

use super::*;

/// What a Sim chooses to do on their own. `from_seat`: sitting, only a word with someone sitting
/// within that many metres, or listening in on a talk there (`conversation::from_seat`).
pub(crate) fn pick_autonomous(
    sim: &Sim,
    ctx: &Ctx,
    objects: &[ObjectInstance],
    items: &crate::items::Items,
    rng: &mut Rng,
    from_seat: Option<f32>,
) -> Option<Task> {
    let content = ctx.content;
    let needs = &sim.needs[..content.needs.len()];
    // How hard each need pulls (more below the level they like to keep it at).
    let felt = ai::felt(needs, &content.needs, &content.autonomy);
    let emotion = sim.emotion(content).map(|e| &content.emotions[e].mods);
    let schedule = content.schedule_at(ctx.hour);
    // Traits, the current emotion and the time of day all bias what feels right.
    let feel = |tags: TagMask| {
        sim.mods.preference(tags)
            * emotion.map_or(1.0, |m| m.preference(tags))
            * schedule.preference(tags)
    };
    let me = sim.id as usize;
    let here = ctx.briefs[me].plot;
    let mut candidates: Vec<(f32, Choice)> = Vec::new();
    let routines = &ctx.households[sim.household as usize].routines;
    let night = crate::planner::planned_night(content, sim, routines, ctx.day, ctx.minute);

    let object_rules = &content.object_rules;
    let funds = ctx.households[sim.household as usize].funds - sim.pending_spend;
    let environment = content
        .needs
        .iter()
        .position(|n| n.room)
        .map_or(1.0, |n| sim.needs[n]);
    // A need that wakes sleepers (bladder, hunger) is urgent: deal with it first, as a sleep
    // started now would end at once (see `use_object`), over and over.
    let rhythm = &content.day_rhythm;
    let woken = (0..content.needs.len())
        .any(|n| needs[n] < content.autonomy.urgent_below && rhythm.wake_for & (1 << n) != 0);
    // (Not a nap either: the sofa won't help.) Worn out, it's bed whatever the hour, not a nap;
    // and no nap again too soon after the last.
    let rest = rhythm.sleep_tags | rhythm.nap_tags;
    let exhausted = exhausted(sim, content);
    let napped = ctx.tick < sim.nap_until;
    // Nor back to bed rested, unless it's the middle of the night (it would end at once), nor
    // once the alarm's gone off for work or school.
    let rested = (!before_dawn(sim, ctx) && rhythm.rest_need.is_some_and(|n| needs[n] >= 0.95))
        || getting_up(sim, ctx);
    // Only what they may use: at home and in public places, or where they're a guest.
    let usable = usable_here(sim, ctx);
    let usable: &[u32] = &usable;
    // Cooking: the stations they have (recipes need theirs), and whether food is ready.
    let stations = crate::cooking::station_mask(sim, ctx, objects);
    let fresh = items
        .list
        .iter()
        .any(|it| it.household == sim.household && it.edible());
    // Needs about to run out (see `use_object`'s `drained`); the essentials among them that
    // home has something for; and all needs home has something for (what they look ahead to).
    let low = content.autonomy.urgent_below;
    let short: u32 = (0..content.needs.len())
        .filter(|&n| !content.needs[n].room && needs[n] < low)
        .fold(0, |m, n| m | 1 << n);
    let walk_per_minute = sim.walk_per_minute(content);
    let urgent = urgent_needs(sim, ctx, objects);
    let fillable = home_needs(
        sim,
        ctx,
        objects,
        (0..content.needs.len())
            .filter(|&n| !content.needs[n].room)
            .fold(0, |m, n| m | 1 << n),
        false,
    );
    // How far away the nearest thing for each need is, how long until they leave for work or
    // school, or head home from where they're visiting (an outing ends in time to be home by
    // `visits.outingHomeBy`), and how long until they get ready to leave for work or school
    // (`dayRhythm.readyMinutes` before it) (minutes).
    let reach = reach(sim, ctx, objects, usable, walk_per_minute);
    let visit_end = sim.visiting.filter(|v| v.arrived > 0).map(|v| v.until);
    let leave = leave_today(sim, ctx);
    let until_leave = leave
        .into_iter()
        .chain(visit_end)
        .min()
        .map(|t| t.saturating_sub(ctx.tick) as f32 * MINUTES_PER_TICK);
    let ready = social::minutes_to_ticks(content.day_rhythm.ready_minutes);
    let until_ready = leave.map(|t| t.saturating_sub(ctx.tick + ready) as f32 * MINUTES_PER_TICK);
    // The last grown-up at home with a baby doesn't go out (`life::minding_baby`), unless for
    // an essential need about to run out that home has nothing for.
    let minding = crate::life::minding_baby(sim, ctx);
    let own = ctx.households[sim.household as usize].plot;
    let stranded = short & !fillable & content.autonomy.urgent;
    // A spare bed's night (a sofa bed's, the sofa's) only for someone with no other bed free; a
    // couple's bed kept for the couple only for someone with nowhere else at all.
    let bedded = has_bed(sim, ctx, objects, usable, false);
    let elsewhere = std::cell::OnceCell::new();
    let elsewhere =
        || *elsewhere.get_or_init(|| bedded || has_bed(sim, ctx, objects, usable, true));
    for obj in usable
        .iter()
        .map(|&o| &objects[o as usize])
        .filter(|o| o.has_free_slot(content.objects[o.def].slots))
    {
        let plot = ctx.object_plot[obj.id as usize];
        let allowed = access(ctx, sim, plot);
        // (Off the home plot: a public place or the street.)
        let outing = || {
            plot != own
                && !content.objects[obj.def]
                    .interactions
                    .iter()
                    .any(|i| i.fills() & stranded != 0)
        };
        if allowed == Access::None || (minding && outing()) {
            continue;
        }
        if obj.broken() {
            // Broken things at home get repaired (handier residents are keener).
            if allowed == Access::Full {
                let rules = &object_rules.repair;
                let (fx, fz) = ctx
                    .index
                    .links
                    .approach_of(obj.id)
                    .unwrap_or_else(|| obj.front_tile(content));
                let distance = ctx
                    .nav
                    .stairs
                    .distance([fx as f32 + 0.5, fz as f32 + 0.5], sim.pos);
                let level = rules.skill.map_or(0.0, |s| sim.skills[s]);
                let (plan, floor) = crate::planner::tags_factor(content, sim, rules.tags);
                // More urgent the more they need what it gives (a broken toilet, bursting).
                let urgency = content.objects[obj.def]
                    .interactions
                    .iter()
                    .flat_map(|it| {
                        (0..content.needs.len()).filter(move |&n| it.total_gain[n] > 0.0)
                    })
                    .map(|n| 1.0 - needs[n])
                    .fold(0.0, f32::max);
                let s = (rules.interest * (1.0 + level * 0.1) * (1.0 + 2.0 * urgency)
                    / (1.0 + distance * 0.08))
                    .max(floor)
                    * feel(rules.tags)
                    * plan;
                if s > MIN_AUTONOMY_SCORE {
                    candidates.push((s, Choice::Repair(obj.id)));
                }
            }
            continue;
        }
        let (fx, fz) = ctx
            .index
            .links
            .approach_of(obj.id)
            .unwrap_or_else(|| obj.front_tile(content));
        let distance = ctx
            .nav
            .stairs
            .distance([fx as f32 + 0.5, fz as f32 + 0.5], sim.pos);
        let at_table = ctx.index.links.table_of(obj.id).is_some();
        // In a room someone keeps shut (a bathroom in use): only for a need about to run out
        // (they'll wait at the door), else something else or somewhere else.
        let shut_away = crate::privacy::shut_away(ctx, sim, (fx, fz));
        // Nicer rooms draw residents, the more so the less they like their surroundings.
        let place = 1.0
            + content.room_rules.preference
                * (ctx.room_at([fx as f32 + 0.5, fz as f32 + 0.5]) - 0.5)
                * 2.0
                * (1.0 - environment);
        for (i, inter) in content.objects[obj.def].interactions.iter().enumerate() {
            // A baby's place is only ever a baby's; caring for one is for grown-ups at home with
            // a baby in it, the more the baby needs it. Nothing their age isn't for (`unfit`).
            let mut care = 0.0;
            // Dining at a table only with a table to dine at.
            // (Nor one that takes the whole thing while someone else is on it.)
            if inter.baby
                || (inter.spare && bedded)
                || (inter.tags & rhythm.sleep_tags != 0
                    && crate::privacy::kept_for_couple(ctx, objects, sim, obj)
                    && elsewhere())
                || unfit(content, sim.age, inter).is_some()
                || upper_barred(content, obj, sim.id, sim.age)
                || (inter.needs_table && !at_table)
                || !requirements_met(ctx, sim, objects, obj.id, inter)
                || !obj.allows(inter, sim.id)
                || !obj.can_use_for(sim.id, &content.objects[obj.def], i)
                || (inter.partner
                    && !(crate::partners::may_take(
                        obj,
                        &content.objects[obj.def],
                        i,
                        sim.id,
                        None,
                        |_| false,
                    ) && crate::partners::free_near(ctx, sim, obj, inter)))
            {
                continue;
            }
            if shut_away && inter.fills() & short == 0 {
                continue;
            }
            // Costing a need about to run out, or planned with one it doesn't fill: it would stop
            // at once (`use_object`), and be chosen again, over and over.
            let costs = (0..content.needs.len())
                .any(|n| short & (1 << n) != 0 && inter.total_gain[n] < 0.0);
            let leaves = (0..content.needs.len())
                .any(|n| short & (1 << n) != 0 && inter.total_gain[n] <= 0.0);
            if costs || (leaves && crate::planner::planned_now(content, sim, inter)) {
                continue;
            }
            if inter.homework > 0.0 {
                care += inter.homework / 10.0 * (100.0 - sim.grade) / 100.0 * 1.5
                    / (1.0 + distance * 0.08);
            }
            if inter.cares() {
                let baby = obj
                    .users()
                    .find(|&u| ctx.briefs.get(u as usize).is_some_and(|b| b.baby));
                let Some(baby) = baby.filter(|_| allowed == Access::Full) else {
                    continue;
                };
                let b = &ctx.briefs[baby as usize];
                let family = if ctx.rels.kin(me, baby as usize) != crate::social::Kin::None {
                    1.5
                } else {
                    1.0
                };
                care = ai::score_gains(&b.needs[..content.needs.len()], &inter.care, distance)
                    * family;
                if care <= 0.0 {
                    continue;
                }
            }
            if !inter.autonomous
                || (woken && inter.tags & rest != 0)
                || (rested && inter.tags & rhythm.sleep_tags != 0)
                || (napped && !exhausted && inter.tags & rhythm.nap_tags != 0)
                || (allowed == Access::Guest
                    && !guest_may(content, &content.objects[obj.def], inter))
                || !affordable(inter.cost, funds)
            {
                continue;
            }
            let sleep = inter.tags & rhythm.sleep_tags != 0;
            let fills = inter.fills();
            let there = distance / walk_per_minute;
            let minutes = inter.minutes_where(ctx.served(obj.id)) + there;
            // A chosen use that would stop at once for an urgent need (`use_object`), then get
            // chosen again: don't choose it at all.
            if !sleep
                && breaks_off(sim, content, urgent, fills, minutes, inter, low)
                && !crate::planner::planned_now(content, sim, inter)
            {
                continue;
            }
            // Better objects and more skilled Sims get more out of it, and know it.
            let boost = quality_boost(content, inter, obj.quality)
                * inter.skill_factor(&content.skill_rules, &sim.skills);
            let gains =
                ai::score_felt(needs, &felt, &inter.total_gain.map(|g| g * boost), distance);
            // A meal nobody at home would eat (a household that has eaten): cooked only with
            // nothing much else to do, not to practise at mid-morning.
            let mut unwanted = false;
            let mut s = match inter.recipe {
                // A recipe: only at home with every station it needs. Long ones tempt less, a
                // hungry family more, and with food ready (or nobody to eat it) hardly at all.
                Some(r) => {
                    let r = &content.recipes[r];
                    let at_home = ctx.object_plot[obj.id as usize]
                        .is_some_and(|p| Some(p) == ctx.households[sim.household as usize].plot);
                    // (Nobody starts on a long recipe that something would run out before it's
                    // done for, hunger too, as it's eaten only at the end; nor shortly before
                    // leaving for work; nor one they couldn't get to, cook and eat before getting
                    // ready to leave, as breakfast went on into getting ready. It takes all its
                    // steps' time: those with no station here are done at one that is,
                    // `cooking::extra_minutes`.)
                    let minutes = r.minutes;
                    let urgent = runs_out(
                        sim,
                        content,
                        fillable & content.autonomy.urgent,
                        minutes,
                        false,
                        &[0.0; MAX_NEEDS],
                        &reach,
                        low,
                    ) || until_leave.is_some_and(|m| minutes > m)
                        || until_ready.is_some_and(|m| there + minutes + r.eat_minutes > m);
                    if urgent || !at_home || allowed != Access::Full || !r.possible(stations) {
                        continue;
                    }
                    let hungry = crate::cooking::hungry_housemates(sim, ctx, r) as f32;
                    unwanted = hungry == 0.0 && !crate::cooking::hungry_for(sim, content, r);
                    let s = gains / (1.0 + r.minutes_with(stations) / 60.0)
                        + content.cooking.feed_family * hungry;
                    if fresh { s * 0.2 } else { s }
                }
                None => gains,
            } + care;
            if inter.trains_skills() && allowed == Access::Full {
                s += training_interest(sim, content, inter) * boost / (1.0 + distance * 0.08);
            }
            // The washing waiting at home, the more the keener; a machine done, to be emptied.
            if allowed == Access::Full {
                s += crate::machines::pull(
                    ctx.households[sim.household as usize].laundry,
                    content,
                    obj,
                    inter,
                ) / (1.0 + distance * 0.08);
            }
            // The plan: a block's activity is the favourite (even with full needs); goals.
            let (plan, floor) = crate::planner::object_factor(content, sim, inter, night);
            let mut s = s.max(floor) * feel(inter.tags) * plan * place;
            if unwanted {
                s *= UNWANTED_MEAL;
            }
            if exhausted && inter.tags & rhythm.sleep_tags != 0 {
                // (The schedule's daytime damping on sleep doesn't apply.)
                s /= schedule
                    .preference(inter.tags & rhythm.sleep_tags)
                    .clamp(1e-3, 1.0);
            } else if exhausted && inter.tags & rhythm.nap_tags != 0 {
                s *= 0.25;
            }
            // Not something long another need would run out before the end of, counting the way
            // there and from there to what fills it (only what would wake them counts for a
            // night's sleep): that first. Nor what they couldn't finish before leaving for work,
            // or before an outing's time is up.
            let watch =
                fillable & !fills & !urgent & if sleep { rhythm.wake_for } else { u32::MAX };
            if runs_out(
                sim,
                content,
                watch,
                minutes,
                sleep,
                &inter.gain_per_minute,
                &reach.map(|r| r + there),
                low,
            ) || (!sleep && until_leave.is_some_and(|m| minutes > m))
            {
                s *= content.autonomy.look_ahead;
            }
            if s > MIN_AUTONOMY_SCORE {
                candidates.push((s, Choice::Object(obj.id, i)));
            }
        }
    }

    // About to drop (or burst): no time to talk, as a chat would break off at once and leave the
    // other one's activity dropped for nothing (`available_for_social` asks the same of them).
    let pressed = sim.pressed(content);
    // (Who hears them, worked out once for all of those asked about: `conversation::Earshot`.)
    let mut earshot = conversation::Earshot::new(&ctx.nav, sim.pos);
    for (j, other) in ctx.briefs.iter().enumerate() {
        if j == me || !other.available || pressed {
            continue;
        }
        // Chat with people within earshot (they call out to them); nobody crosses the house or
        // the lot for a chat, and crossing town is a visit.
        let distance = ctx.nav.stairs.distance(other.pos, sim.pos);
        let heard = earshot.hears(other.pos);
        // Nor out on the street, minding the baby.
        if !heard || (minding && other.plot != own) {
            continue;
        }
        // Not over the fence into someone else's garden to chat (a guest is welcome at the host's).
        if let Some(b) = other.plot
            && here != Some(b)
            && access(ctx, sim, Some(b)) == Access::None
        {
            continue;
        }
        let rel = ctx.rels.get(me, j);
        let chemistry = ctx.rels.chemistry(me, j);
        let attracted = other
            .gender_ix
            .is_none_or(|g| sim.attraction & (1 << g) != 0);
        let single = !sim.planner.goals.is_empty() && ctx.rels.partner_of(j).is_none();
        let grown_ups = ctx.briefs[me].adult && other.adult;
        // A guest keeps the hosts company, even with nothing in particular to talk about.
        let company = match sim.visiting {
            Some(v)
                if here == Some(v.plot)
                    && ctx.households[other.household as usize].plot == Some(v.plot) =>
            {
                content.visits.company
            }
            _ => 0.0,
        };
        // Two singles who like the look of each other: a spark before there's any romance.
        let spark = if conversation::may_court(grown_ups, rel)
            && attracted
            && rel.met
            && !ctx.briefs[me].partnered
            && !other.partnered
        {
            content.social_rules.single_spark
        } else {
            0.0
        };
        let (my_stage, their_stage) = (content.life.stage(sim.age), content.life.stage(other.age));
        for (k, s) in content.socials.iter().enumerate() {
            // (Nor with someone their age isn't for: no fights with a child; nor anything they'd
            // have to get up from their meal for.)
            if !s.autonomous
                || !s.requires.allows(rel)
                || !stage_in(s.stages, my_stage)
                || !stage_in(s.target_stages, their_stage)
                || (other.eating && s.distance < conversation::SEATED_DISTANCE)
            {
                continue;
            }
            // Romance only between grown-ups who aren't family.
            if s.tags & content.social_rules.romantic_tags != 0
                && (!attracted || !conversation::may_court(grown_ups, rel))
            {
                continue;
            }
            // Trying for a baby only where one could come of it, and with a bed behind a shut door
            // for it (where the content has beds for it: `privacy.rs`).
            if s.success.effect == crate::social::Effect::Conceive
                && (!could_conceive(ctx, me, j)
                    || (crate::privacy::beds_in_content(content)
                        && crate::privacy::bed_in_step(ctx, objects, me, j).is_none()))
            {
                continue;
            }
            let romantic = s.tags & content.social_rules.romantic_tags != 0;
            let friendly = matches!(s.prefer, Prefer::Any | Prefer::Liked);
            let gains = ai::score_felt(needs, &felt, &s.actor_gain, distance);
            let gains = if s.prefer == Prefer::Liked && !romantic {
                gains.max(company)
            } else {
                gains
            };
            let score = gains
                * s.autonomy_weight
                * prefer_factor(s.prefer, rel, chemistry, spark)
                * feel(s.tags)
                * crate::planner::social_factor(
                    content,
                    sim,
                    romantic,
                    friendly,
                    rel.friendship,
                    single,
                );
            if score > MIN_AUTONOMY_SCORE {
                candidates.push((score, Choice::Social(j as u32, k)));
            }
        }
    }

    // Joining in where people are talking (nearby, a friendly social, room for one more): for
    // the social it brings, more so with people they like.
    for (l, leader) in ctx.briefs.iter().enumerate() {
        let Some((_, social)) = leader.conversing.filter(|c| c.0 as usize == l && l != me) else {
            continue;
        };
        let s = &content.socials[social];
        if !s.joinable || !leader.here || leader.away || pressed {
            continue;
        }
        // (Only where they can hear them talking.)
        let distance = ctx.nav.stairs.distance(leader.pos, sim.pos);
        let heard = earshot.hears(leader.pos);
        if !heard || (minding && leader.plot != own) {
            continue;
        }
        if let Some(b) = leader.plot
            && here != Some(b)
            && access(ctx, sim, Some(b)) == Access::None
        {
            continue;
        }
        let members: Vec<usize> = (0..ctx.briefs.len())
            .filter(|&m| ctx.briefs[m].conversing.is_some_and(|c| c.0 as usize == l))
            .collect();
        if members.len() >= conversation::MAX_GROUP || members.contains(&me) {
            continue;
        }
        let count = members.len() as f32;
        let liking = members
            .iter()
            .map(|&m| prefer_factor(Prefer::Liked, ctx.rels.get(me, m), 0.0, 0.0))
            .sum::<f32>()
            / count;
        let friendship = members
            .iter()
            .map(|&m| ctx.rels.get(me, m).friendship)
            .sum::<f32>()
            / count;
        // A guest joins in where the hosts are talking (keeping them company).
        let company = match sim.visiting {
            Some(v)
                if here == Some(v.plot)
                    && members.iter().any(|&m| {
                        ctx.households[ctx.briefs[m].household as usize].plot == Some(v.plot)
                    }) =>
            {
                content.visits.company
            }
            _ => 0.0,
        };
        let score = ai::score_felt(needs, &felt, &s.target_gain, distance).max(company)
            * s.autonomy_weight
            * liking
            * feel(s.tags)
            * crate::planner::social_factor(content, sim, false, true, friendship, false);
        if score > MIN_AUTONOMY_SCORE {
            candidates.push((score, Choice::Join(l as u32)));
        }
    }

    // Tidying up: the dirtiest spot at home, if it's dirty enough. The worse their surroundings
    // feel, the keener they are; traits and planned chores do the rest.
    let home = ctx.households[sim.household as usize].plot;
    if let Some(plot) = home.filter(|&p| here == Some(p)) {
        let rules = &content.room_rules.clean;
        let p = &ctx.plots[plot as usize];
        let lot = ctx.nav.lot;
        let mut best: Option<(f32, i32, i32)> = None;
        // (Not where they couldn't get to a moment ago, nor where someone else is tidying up:
        // the next dirtiest spot instead.)
        let mut out_of_reach: Vec<(i32, i32)> = sim
            .unreachable_now(ctx.tick)
            .filter_map(|k| {
                if let TaskKind::Clean { x, z } = k {
                    Some((x, z))
                } else {
                    None
                }
            })
            .collect();
        out_of_reach.extend(
            ctx.briefs
                .iter()
                .enumerate()
                .filter(|&(j, _)| j != me)
                .filter_map(|(_, b)| b.cleaning),
        );
        for z in p.z..p.z + p.d {
            for x in p.x..p.x + p.w {
                if !lot.in_bounds(x, z) || out_of_reach.contains(&(x, z)) {
                    continue;
                }
                let i = lot.tile_index(x, z);
                let d = ctx.dirt.get(i).copied().unwrap_or(0.0);
                if d > rules.threshold
                    && !ctx.nav.blocked[i]
                    && !lot.diag_blocks(x, z)
                    && best.is_none_or(|b| d > b.0)
                {
                    best = Some((d, x, z));
                }
            }
        }
        if let Some((d, x, z)) = best {
            let distance = ctx
                .nav
                .stairs
                .distance([x as f32 + 0.5, z as f32 + 0.5], sim.pos);
            let (plan, floor) = crate::planner::tags_factor(content, sim, rules.tags);
            let s = (rules.interest * d * (1.5 - environment) / (1.0 + distance * 0.08)).max(floor)
                * feel(rules.tags)
                * plan;
            if s > MIN_AUTONOMY_SCORE {
                candidates.push((s, Choice::Clean(x, z)));
            }
        }
    }

    // Food at home: a serving of what's been cooked (leftovers, or the dinner they were called
    // to), and washing up where there's a sink or dishwasher.
    if home.is_some_and(|p| here == Some(p)) {
        let rules = &content.cooking;
        let invited = if sim.invited_until > ctx.tick {
            rules.invited_boost
        } else {
            1.0
        };
        let mut best: Option<(f32, u32)> = None;
        for it in items
            .list
            .iter()
            .filter(|it| it.household == sim.household && it.edible())
        {
            let Some(r) = it.recipe else { continue };
            let distance = ctx.nav.stairs.distance(it.at.centre(), sim.pos);
            let gains = content.recipes[r].effects.map(|g| g * it.quality);
            let s = ai::score_felt(needs, &felt, &gains, distance)
                * rules.leftover_preference
                * invited;
            if best.is_none_or(|b| s > b.0) {
                best = Some((s, it.id));
            }
        }
        if let Some((s, item)) = best {
            let (plan, floor) = crate::planner::tags_factor(content, sim, rules.eat_tags);
            let s = s.max(floor) * feel(rules.eat_tags) * plan;
            if s > MIN_AUTONOMY_SCORE {
                candidates.push((s, Choice::Eat(item)));
            }
        }
        // (With no sink, the dirty things go in the bin.)
        if stations & (rules.dishes | rules.bin) != 0 {
            let mut best: Option<(f32, u32)> = None;
            let mut count = 0;
            for it in items
                .list
                .iter()
                .filter(|it| it.household == sim.household && it.to_clear(content))
            {
                count += 1;
                let distance = ctx.nav.stairs.distance(it.at.centre(), sim.pos);
                if best.is_none_or(|b| distance < b.0) {
                    best = Some((distance, it.id));
                }
            }
            if let Some((distance, item)) = best {
                let (plan, floor) = crate::planner::tags_factor(content, sim, rules.dishes_tags);
                // The bigger the pile, the keener (a plate or two can wait).
                let pile = (count as f32 / f32::from(rules.carry_dishes)).clamp(0.4, 3.0);
                let s = (rules.dishes_interest * pile * (1.5 - environment)
                    / (1.0 + distance * 0.08))
                    .max(floor)
                    * feel(rules.dishes_tags)
                    * plan;
                if s > MIN_AUTONOMY_SCORE {
                    candidates.push((s, Choice::Dishes(item)));
                }
            }
        }
    }

    // The phone: texting, calling or inviting people who aren't here, scrolling (`phone.rs`).
    // (Nor a call or a text, pressed.)
    for (s, action, contact) in crate::phone::candidates(sim, ctx, &felt, &feel) {
        if s > MIN_AUTONOMY_SCORE && !(pressed && contact.is_some()) {
            candidates.push((s, Choice::Phone(action, contact)));
        }
    }

    // Residents visit friends who are home, or go out. (Not from a seat: only a word from it,
    // below.)
    let household = &ctx.households[sim.household as usize];
    let visits = &content.visits;
    if from_seat.is_none()
        && sim.visiting.is_none()
        && !crate::taxi::waiting_out(ctx.taxis, sim)
        && here.is_some()
        && here == household.plot
        && !minding
        && (visits.earliest_hour..visits.latest_hour).contains(&ctx.hour)
        && (sim.back_at == 0
            || ctx.tick >= sim.back_at + (visits.again_minutes * TICKS_PER_MINUTE as f32) as u64)
    {
        // Not a walk of hours, nor to arrive after visiting hours, by the way they'd go there
        // (`leisure_way`: the car, a bike of theirs, the taxi, else on foot; `visit_fits`), with
        // time for it before work or school (`free_before_work`).
        let free = free_before_work(sim, ctx);
        let goer = Goer {
            drives: crate::cars::could_drive(ctx, sim),
            bike: crate::ride::leisure_bike(
                content,
                ctx.index,
                ctx.briefs,
                ctx.households,
                objects,
                sim,
            )
            .map(|(_, speed, min)| (speed, min)),
        };
        // (From over the ground, upstairs too. Where they'd get on the roads from here, on foot
        // and by car, found once for all the places, and each place's door's as noted,
        // `Travel::door_entry`.)
        let pos = ctx.nav.stairs.ground_pos(sim.pos);
        let mut on: [Option<Option<crate::travel::RoadPoint>>; 2] = [None, None];
        let soon = crate::planner::plans_soon(
            content,
            sim,
            routines,
            ctx.day,
            ctx.minute,
            visits.hours * 60.0,
        );
        for other in ctx.households.iter().filter(|h| h.id != household.id) {
            let Some(plot) = other.plot else { continue };
            // The best friend of theirs at home, and someone to open the door: a grown-up at
            // home, up and about, not on their way out (`life::opens_door`).
            let mut friendship = f32::NEG_INFINITY;
            let mut opener = false;
            for (j, b) in ctx.briefs.iter().enumerate() {
                if b.household == other.id && b.plot == Some(plot) && !b.away {
                    friendship = friendship.max(ctx.rels.get(me, j).friendship);
                    opener = opener || crate::life::opens_door(b, other.id, plot);
                }
            }
            if friendship < visits.min_friendship || !opener {
                continue;
            }
            let door = ctx.plots[plot as usize].arrival_point();
            let distance = (door[0] - pos[0]).hypot(door[1] - pos[1]);
            // Travel is less of a deterrent than for a chat in the same room.
            let score = ai::score_felt(needs, &felt, &visits.gains, distance * 0.25)
                * (0.4 + friendship / 100.0)
                * feel(visits.tags)
                * crate::planner::visit_factor(content, sim, soon);
            // (The way there timed only for a visit worth making.)
            if score > MIN_AUTONOMY_SCORE {
                let taxi = ctx
                    .taxis
                    .takes(sim)
                    .then(|| {
                        crate::taxi::leisure_eta_via(
                            ctx.travel,
                            ctx.exits,
                            content,
                            sim,
                            pos,
                            (Some(plot), door),
                            &mut on,
                        )
                    })
                    .flatten();
                let (_, eta) = leisure_way(
                    ctx.travel,
                    content,
                    pos,
                    walk_per_minute,
                    plot,
                    door,
                    goer,
                    taxi,
                    &mut on,
                );
                // (Nor with a need that would be low by the time they could leave, that the
                // hosts' home has nothing a guest may use for: `lasts`. Nor to hosts all off to
                // work before they could stay that long: let in, the guest was gone again 23
                // minutes on, Stillpool.)
                let serves = place_serves(ctx, objects, sim, plot, true);
                let stay = eta + visits.least_minutes;
                if visit_fits(content, eta, ctx.hour, free)
                    && hosts_about_till(ctx, other.id) >= ctx.tick + crate::mins(stay)
                    && lasts(content, sim, &serves, stay)
                {
                    candidates.push((score, Choice::Visit(plot)));
                }
            }
        }

        // An outing: a public place out of reach, for the best it has, the less the longer the
        // way (`OUTING_K`): only one the way to takes `outingMinutes` at most, with time for there
        // and back and an hour there before work or school and before `outingHomeBy`
        // (`outing_fits`).
        let reach = content.autonomy.reach;
        for p in ctx.plots.iter().filter(|p| p.public) {
            let door = p.arrival_point();
            if (door[0] - pos[0]).hypot(door[1] - pos[1]) <= reach {
                continue;
            }
            // By the way they'd go (`leisure_way`).
            let taxi = ctx
                .taxis
                .takes(sim)
                .then(|| {
                    crate::taxi::leisure_eta_via(
                        ctx.travel,
                        ctx.exits,
                        content,
                        sim,
                        pos,
                        (Some(p.id), door),
                        &mut on,
                    )
                })
                .flatten();
            let (_, eta) = leisure_way(
                ctx.travel,
                content,
                pos,
                walk_per_minute,
                p.id,
                door,
                goer,
                taxi,
                &mut on,
            );
            if !outing_fits(content, eta, ctx.hour, free) {
                continue;
            }
            // The best the place has for each need, as one day out; not with a need that
            // would be low by the time they could leave, that it has nothing for (`lasts`).
            let best = place_serves(ctx, objects, sim, p.id, false);
            if !lasts(content, sim, &best, eta + visits.least_minutes) {
                continue;
            }
            let score = ai::score_felt(needs, &felt, &best, 0.0) * feel(visits.tags)
                / (1.0 + OUTING_K * eta);
            if score > MIN_AUTONOMY_SCORE {
                candidates.push((score, Choice::Outing(p.id)));
            }
        }
    }

    // Not what they couldn't get to a moment ago (and would try again and again, each time
    // looking for a way there).
    for kind in sim.unreachable_now(ctx.tick) {
        candidates.retain(|&(_, choice)| match (choice, kind) {
            (
                Choice::Object(o, i),
                TaskKind::Use {
                    object,
                    interaction,
                },
            ) => (o, i) != (object, interaction),
            (Choice::Eat(it), TaskKind::Eat { item, .. }) => Some(it) != item,
            (Choice::Clean(x, z), TaskKind::Clean { x: cx, z: cz }) => (x, z) != (cx, cz),
            (Choice::Repair(o), TaskKind::Repair { object }) => o != object,
            (Choice::Dishes(it), TaskKind::Dishes { item, .. }) => it != item,
            (Choice::Visit(p) | Choice::Outing(p), TaskKind::Visit { plot }) => p != plot,
            (Choice::Social(t, s), TaskKind::Social { target, social }) => {
                (t, s) != (target, social)
            }
            (Choice::Join(l), TaskKind::Join { leader }) => l != leader,
            _ => true,
        });
    }

    // Waiting for something to eat a host is bringing them (`hosting.rs`): nothing of their own.
    if sim.expecting > ctx.tick {
        candidates.retain(|&(_, choice)| match choice {
            Choice::Eat(_) => false,
            Choice::Object(o, i) => content.objects[objects[o as usize].def]
                .interactions
                .get(i)
                .is_none_or(|it| it.eats.is_none() && it.recipe.is_none()),
            _ => true,
        });
    }

    // From the seat: only a word with someone sitting near (nothing that takes them closer than
    // talking does: a hug, a kiss, a fight), or listening in on a talk going on beside them.
    if let Some(range) = from_seat {
        // (In sight, too: not the one on the sofa the other side of the wall.)
        let near = |p: [f32; 2]| {
            (p[0] - sim.pos[0]).hypot(p[1] - sim.pos[1]) <= range
                && conversation::in_sight(&ctx.nav, sim.pos, p)
        };
        candidates.retain(|&(_, choice)| match choice {
            Choice::Social(t, s) => {
                ctx.briefs[t as usize].seated
                    && near(ctx.briefs[t as usize].pos)
                    && content.socials[s].distance >= conversation::SEATED_DISTANCE
            }
            Choice::Join(l) => ctx
                .briefs
                .iter()
                .any(|b| b.conversing.is_some_and(|c| c.0 == l) && near(b.pos)),
            _ => false,
        });
    }

    // Needs about to run out come first: whatever doesn't fill one is put off.
    if urgent != 0 {
        for (s, choice) in &mut candidates {
            let gains = match *choice {
                Choice::Object(o, i) => {
                    content.objects[objects[o as usize].def].interactions[i].total_gain
                }
                Choice::Eat(item) => items
                    .get(item)
                    .and_then(|it| it.recipe)
                    .map_or([0.0; MAX_NEEDS], |r| content.recipes[r].effects),
                _ => [0.0; MAX_NEEDS],
            };
            if !(0..content.needs.len()).any(|n| urgent & (1 << n) != 0 && gains[n] > 0.0) {
                *s *= content.autonomy.look_ahead;
            }
        }
    }

    ai::choose(&mut candidates, rng.next_f32(), 3).map(|choice| Task {
        kind: match choice {
            Choice::Object(object, interaction) => TaskKind::Use {
                object,
                interaction,
            },
            Choice::Social(target, social) => TaskKind::Social { target, social },
            Choice::Join(leader) => TaskKind::Join { leader },
            Choice::Visit(plot) | Choice::Outing(plot) => TaskKind::Visit { plot },
            Choice::Clean(x, z) => TaskKind::Clean { x, z },
            Choice::Repair(object) => TaskKind::Repair { object },
            Choice::Eat(item) => TaskKind::Eat {
                item: Some(item),
                seat: None,
                plate: None,
                stage: 0,
            },
            Choice::Dishes(item) => TaskKind::Dishes {
                item,
                station: None,
                stage: 0,
                scraps: 0,
            },
            Choice::Phone(action, contact) => TaskKind::Phone {
                action,
                contact,
                seat: None,
                answer: crate::phone::Answer::Pending,
            },
        },
        directed: false,
        routine: false,
    })
}

/// How much a relationship invites a social of the given kind.
pub(crate) fn prefer_factor(
    prefer: Prefer,
    rel: &social::Relationship,
    chemistry: f32,
    spark: f32,
) -> f32 {
    match prefer {
        Prefer::Any => 1.0,
        Prefer::Liked => (0.5 + rel.friendship / 100.0).clamp(0.1, 1.5),
        Prefer::Disliked => (0.4 - rel.friendship / 100.0).clamp(0.0, 1.4),
        Prefer::Romance => (rel.romance / 50.0 + chemistry * 0.4 + spark).clamp(0.0, 1.6),
        Prefer::Unloved => {
            if rel.partners && rel.romance < 25.0 {
                (25.0 - rel.romance) / 25.0
            } else {
                0.0
            }
        }
    }
}

/// A walk's goal at `at`: the point, its tile, and no tags (`start_task`).
pub(crate) fn goal_at(at: [f32; 2]) -> ([f32; 2], (i32, i32), TagMask) {
    (at, (at[0].floor() as i32, at[1].floor() as i32), 0)
}

pub(crate) fn start_task(
    sim: &mut Sim,
    task: Task,
    ctx: &Ctx,
    objects: &mut [ObjectInstance],
    items: &mut crate::items::Items,
) {
    let content = ctx.content;
    let nav = &ctx.nav;
    // At the wheel of a car on a trip (on their way there or back, as a game was loaded): they
    // drive on.
    if matches!(task.kind, TaskKind::Visit { .. } | TaskKind::GoHome)
        && let Some(car) = crate::cars::driving(ctx.cars, sim.id)
    {
        sim.bound_for_car = Some(car);
        sim.current = Some(Activity {
            task,
            phase: Phase::Driving,
            tags: Default::default(),
        });
        return;
    }
    // (Fetched for this use, if it's the one: see `Sim::fetched`; sent to play with someone,
    // `Sim::joins`.)
    let fetched = sim.fetched.take();
    let joins = sim.joins.take();
    // Nothing their age isn't for, nor out of its hours, under a roof or with nothing to work
    // on, however it was chosen (`unfit`, `unmet`; an order was refused already).
    if let TaskKind::Use {
        object,
        interaction,
    } = task.kind
        && let Some(o) = objects.get(object as usize)
        && content.objects[o.def]
            .interactions
            .get(interaction)
            .is_some_and(|i| {
                unfit(content, sim.age, i).is_some()
                    || !conditions_met(ctx, sim, o, i)
                    || closed_now(ctx, objects, object, i)
            })
    {
        return;
    }
    // An outing of their own choosing weighed by car that they'd walk after all (the car taken
    // meanwhile, say), too far for the way they'd go now: not now (`outing_fits_now`). Nor are
    // they on it any more, set out for their car (`life::Transition::Visit`) and not there: else,
    // still on it at home, they'd wander off to the place on foot after all, hours away.
    if let TaskKind::Visit { plot } = task.kind
        && !task.directed
        && !outing_fits_now(sim, ctx, objects, plot)
    {
        if sim
            .visiting
            .is_some_and(|v| v.plot == plot && v.arrived == 0)
        {
            sim.visiting = None;
        }
        return;
    }
    // Off somewhere else from where they left their bike out (`Sim::bike_parked`): to work, or
    // to another place: it's back at its place, and they go on foot. (Home, they ride it.)
    let elsewhere = match task.kind {
        TaskKind::Work => true,
        TaskKind::Visit { plot } => sim.visiting.is_none_or(|v| v.plot != plot),
        _ => false,
    };
    if sim.bike_parked.is_some() && elsewhere {
        crate::ride::get_off(sim, objects);
    }
    // A crib's care done by day where the baby is carried to (the high chair's feed, `carry.rs`).
    if let Some(carry) = crate::carry::instead(sim, &task, ctx, objects) {
        return crate::carry::start(sim, carry, ctx, objects);
    }
    // A recipe isn't made where it was picked: the cook goes round its stations, starting with
    // the first (see `cooking.rs`).
    if let TaskKind::Use {
        object,
        interaction,
    } = task.kind
        && let Some(recipe) = objects
            .get(object as usize)
            .and_then(|o| content.objects[o.def].interactions.get(interaction))
            .and_then(|i| i.recipe)
    {
        let kind = TaskKind::Cook {
            recipe,
            step: 0,
            station: None,
            origin: Some(object),
            servings: 0,
            skipped: 0,
            minutes: 0,
        };
        return crate::cooking::start(sim, Task { kind, ..task }, ctx, objects, items);
    }
    let mut reserved_seat = None;
    // (A seat near the thing to sit on for its use, `seatNear`: without one they stand at it.)
    let mut seat_near = false;
    let (target, goal, tags) = match task.kind {
        TaskKind::Cook { .. } | TaskKind::Eat { .. } | TaskKind::Dishes { .. } => {
            return crate::cooking::start(sim, task, ctx, objects, items);
        }
        TaskKind::Phone { .. } => return crate::phone::start(sim, task, ctx, objects),
        TaskKind::Carry { .. } => return crate::carry::start(sim, task, ctx, objects),
        TaskKind::SetUp { .. } => return crate::setup::start(sim, task, ctx),
        TaskKind::Serve { .. } => return crate::hosting::start(sim, task, ctx, objects),
        TaskKind::Use {
            object,
            interaction,
        } => {
            let Some(obj) = objects.get(object as usize) else {
                return;
            };
            // (Not one it doesn't have: an id from before something was sold or moved.)
            let Some(inter) = content.objects[obj.def].interactions.get(interaction) else {
                return;
            };
            // An order to close it on someone who got in (or set off for it) after it was given:
            // the player is told.
            if task.directed && obj.closes_on_someone(inter, sim.id) {
                sim.news.get_or_insert(News::NotClosed(object));
                return;
            }
            if !obj.can_use_for(sim.id, &content.objects[obj.def], interaction)
                || obj.broken()
                || upper_barred(content, obj, sim.id, sim.age)
            {
                return;
            }
            // A game for two: both its places free, or joining whoever they play it with.
            let along = |u: u32| {
                ctx.briefs
                    .get(u as usize)
                    .and_then(|b| b.game)
                    .is_some_and(|(o, i, _)| (o, i) == (object, interaction))
            };
            if inter.partner
                && !crate::partners::may_take(
                    obj,
                    &content.objects[obj.def],
                    interaction,
                    sim.id,
                    joins,
                    along,
                )
            {
                return;
            }
            // Never other people's things; what the player asks a guest to do, only what guests
            // may (washing hands at the host's sink after the toilet is fine).
            let plot = ctx.object_plot.get(object as usize).copied().flatten();
            let allowed = if task.directed {
                order_access(ctx.households, ctx.plots, sim, plot)
            } else {
                access(ctx, sim, plot)
            };
            if allowed == Access::None
                || (allowed == Access::Guest
                    && task.directed
                    && !guest_may(content, &content.objects[obj.def], inter))
            {
                return;
            }
            if inter.needs_table && ctx.index.links.table_of(object).is_none() {
                return;
            }
            if !obj.allows(inter, sim.id) {
                return;
            }
            // What it needs fetched first (`Interaction::fetch`: a coffee poured at the machine):
            // off to get it, then back to this (the follow-up); with no way to it, without.
            if fetched != Some(object)
                && let Some((from, k)) = fetch_source(sim, ctx, objects, object, inter)
            {
                start_task(
                    sim,
                    Task {
                        kind: TaskKind::Use {
                            object: from,
                            interaction: k,
                        },
                        ..task
                    },
                    ctx,
                    objects,
                    items,
                );
                sim.fetched = Some(object);
                if sim.current.is_some() {
                    sim.follow_up = Some(Task {
                        routine: false,
                        ..task
                    });
                    return;
                }
                return start_task(sim, task, ctx, objects, items);
            }
            // Sitting to use a thing on a top: on a seat pulled up to the top, if one is free
            // (else on one near, `seatNear`, else they stand at it).
            // (Standing too, when the top has chairs all round: they sit down at it.)
            let on_top = content.objects[obj.def].layer == Layer::Surface;
            let boxed_in = || {
                ctx.index.links.host_of(object).is_some_and(|h| {
                    crate::surface::rider_tiles(content, objects, nav, obj, h).is_empty()
                })
            };
            if on_top && (inter.pose == Pose::Sit || boxed_in()) {
                reserved_seat = crate::surface::seat_at_top(
                    content,
                    objects,
                    &ctx.index.links,
                    object,
                    sim.id,
                    content.life.stage(sim.age),
                );
            }
            if inter.seat_near > 0.0 && reserved_seat.is_none() {
                reserved_seat = seat_near_object(sim, ctx, objects, object, inter);
                seat_near = reserved_seat.is_some();
            }
            // From the tile by the slot they'll get (round to its place, `places`; onto a bed
            // from the side, `seat_approach`), or to their own spot along the front of a thing
            // used standing (`stand_spot`).
            let footstool = if inter.pose == Pose::Sit {
                footstool_slots(content, objects, object)
            } else {
                0
            };
            let slot = obj.slot_for(
                sim.id,
                &content.objects[obj.def],
                interaction,
                sim.pos,
                footstool,
            );
            let spot = slot
                .filter(|_| {
                    reserved_seat.is_none()
                        && inter.pose == Pose::Stand
                        && !inter.inside
                        && !inter.carried
                        && !inter.at_slot
                })
                .and_then(|k| {
                    crate::surface::stand_spot(
                        content,
                        nav,
                        obj,
                        k,
                        inter.whole || inter.leaves.is_some(),
                    )
                });
            let t = match reserved_seat {
                Some(seat) => {
                    let s = &objects[seat as usize];
                    let def = &content.objects[s.def];
                    s.pick_slot(
                        sim.id,
                        0..def.slots as usize,
                        Some((def, sim.pos)),
                        footstool_slots(content, objects, seat),
                    )
                    .and_then(|k| crate::surface::seat_approach(content, nav, s, k, sim.pos))
                    .or_else(|| ctx.index.links.approach_of(seat))
                    .unwrap_or_else(|| s.front_tile(content))
                }
                // (A baby's care at the high chair from beside it; the change at the crib's
                // changing table, in front of it; then their own spot, or round to their seat or
                // bed, or a thing on a top from the top's front.)
                None if inter.carried => crate::carry::care_spot(ctx, obj),
                None => slot
                    .filter(|_| inter.at_slot)
                    .and_then(|k| crate::carry::slot_front(ctx, obj, k))
                    .or_else(|| spot.map(|(t, ..)| t))
                    .or_else(|| {
                        slot.and_then(|k| match inter.pose {
                            Pose::Stand if !inter.inside => {
                                crate::surface::slot_approach(content, nav, obj, k)
                            }
                            _ => crate::surface::seat_approach(content, nav, obj, k, sim.pos),
                        })
                    })
                    .or_else(|| ctx.index.links.approach_of(object))
                    .unwrap_or_else(|| obj.front_tile(content)),
            };
            let tags = inter.tags;
            (
                spot.map_or([t.0 as f32 + 0.5, t.1 as f32 + 0.5], |(_, p, _)| p),
                t,
                tags,
            )
        }
        TaskKind::MoveTo { x, z } => ([x, z], (x.floor() as i32, z.floor() as i32), 0),
        TaskKind::Clean { x, z } => (
            [x as f32 + 0.5, z as f32 + 0.5],
            (x, z),
            content.room_rules.clean.tags,
        ),
        // Accidents start on the spot (see `accident`), never from the queue.
        TaskKind::Spot { .. } => return,
        TaskKind::Repair { object } => {
            let Some(obj) = objects.get(object as usize).filter(|o| o.broken()) else {
                return;
            };
            let t = ctx
                .index
                .links
                .approach_of(object)
                .unwrap_or_else(|| obj.front_tile(content));
            (
                [t.0 as f32 + 0.5, t.1 as f32 + 0.5],
                t,
                content.object_rules.repair.tags,
            )
        }
        TaskKind::Work => {
            sim.bound_for_car = None;
            // By car (`cars.rs`): to its door. (Not when already on their bike.)
            let riding = sim.riding.is_some();
            // (Called to the work taxi standing at their halt: over to its door, `taxi::door_for`.)
            let car = if riding {
                None
            } else {
                crate::taxi::door_for(ctx, sim, &TaskKind::Work)
                    .or_else(|| crate::cars::ride_for(ctx, sim))
            };
            if let Some((car, seat, door, at)) = car {
                crate::ride::get_off(sim, objects);
                sim.bound_for_car = Some(car);
                sim.car_seat = seat as u8;
                sim.car_door = door as u8;
                goal_at(at)
            } else {
                // Nearest way out of town; with no exits, work starts on the spot.
                let Some(exit) = exit_on_foot(ctx, sim) else {
                    crate::ride::get_off(sim, objects);
                    sim.transition = Some(crate::life::Transition::Work);
                    return;
                };
                // (In an open-world town, a bike only where the way was timed by one: a ride of
                // `life::BIKE_MOST` at most; not before it's been timed, nor where it was timed on
                // foot or by car.)
                let no_bike = no_bike_out(sim);
                if riding && no_bike {
                    // On a bike of theirs from a visit or an outing where the way out isn't by
                    // one (a ride of over `life::BIKE_MOST`, or none timed): the ride cut short,
                    // and on to work as its way is timed, to their kerb for the taxi or on foot
                    // (on it to work, they rode hours out of town, Hollyshaw).
                    crate::ride::get_off(sim, objects);
                    match crate::taxi::kerb_for(ctx, sim, &TaskKind::Work) {
                        Some(kerb) => goal_at(kerb),
                        None => goal_at(exit),
                    }
                } else if riding {
                    // On their bike: on it to the way out.
                    goal_at(exit)
                } else if let Some(kerb) = crate::taxi::kerb_for(ctx, sim, &TaskKind::Work) {
                    // By the work taxi they're booked on (`taxi.rs`): to the kerb, to wait for it.
                    crate::ride::get_off(sim, objects);
                    goal_at(kerb)
                } else if let Some(bike) = sim.bound_for_bike.or_else(|| {
                    (!no_bike)
                        .then(|| crate::ride::bike_for(ctx, sim, objects, exit))
                        .flatten()
                }) {
                    // On a bike of theirs (`ride.rs`): over to it first.
                    crate::ride::claim(sim, objects, bike);
                    goal_at(crate::ride::spot(content, &objects[bike as usize]))
                } else {
                    goal_at(exit)
                }
            }
        }
        // (Their visit starts once there's a way there: see below.)
        // By the taxi they're booked on there (`taxi.rs`): to its door as it stands for them, else
        // out to their kerb to wait for it.
        TaskKind::Visit { .. } | TaskKind::GoHome
            if sim.riding.is_none()
                && let Some((car, seat, door, at)) =
                    crate::taxi::door_for(ctx, sim, &task.kind) =>
        {
            sim.bound_for_car = Some(car);
            sim.car_seat = seat as u8;
            sim.car_door = door as u8;
            goal_at(at)
        }
        TaskKind::Visit { .. } | TaskKind::GoHome
            if sim.riding.is_none()
                && let Some(kerb) = crate::taxi::kerb_for(ctx, sim, &task.kind) =>
        {
            sim.bound_for_car = None;
            goal_at(kerb)
        }
        // Beyond walking range, with the household's car at home: to its driver's door first,
        // to drive there (`cars::trip_car`).
        TaskKind::Visit { plot }
            if sim.riding.is_none()
                && let Some((car, door, at)) = crate::cars::trip_car(ctx, sim, plot) =>
        {
            sim.bound_for_car = Some(car);
            sim.car_seat = 0;
            sim.car_door = door as u8;
            (
                at,
                (at[0].floor() as i32, at[1].floor() as i32),
                content.visits.tags,
            )
        }
        // Or to a seat in it, the household's car waiting in the driveway to drive near there
        // (`cars::trip_ride`).
        TaskKind::Visit { plot }
            if sim.riding.is_none()
                && let Some((car, seat, door, at)) = crate::cars::trip_ride(ctx, sim, plot) =>
        {
            sim.bound_for_car = Some(car);
            sim.car_seat = seat as u8;
            sim.car_door = door as u8;
            (
                at,
                (at[0].floor() as i32, at[1].floor() as i32),
                content.visits.tags,
            )
        }
        // On their bike there: to where they'll leave it by the place (`ride::park_spot`), and on
        // to it on foot.
        TaskKind::Visit { plot }
            if sim.riding.is_some()
                && let Some((centre, yaw, stand)) =
                    crate::ride::park_spot(ctx, sim, objects, plot) =>
        {
            sim.bike_spot = Some((centre, yaw));
            (
                stand,
                (stand[0].floor() as i32, stand[1].floor() as i32),
                content.visits.tags,
            )
        }
        // Else beyond a short walk, on a bike of theirs (`ride.rs`, `leisure_ride`): over to it
        // first, to ride there.
        TaskKind::Visit { plot }
            if sim.riding.is_none()
                && sim.bike_parked.is_none()
                && let Some(bike) = sim
                    .bound_for_bike
                    .or_else(|| leisure_ride(sim, ctx, objects, plot)) =>
        {
            sim.bound_for_car = None;
            crate::ride::claim(sim, objects, bike);
            let at = crate::ride::spot(content, &objects[bike as usize]);
            (
                at,
                (at[0].floor() as i32, at[1].floor() as i32),
                content.visits.tags,
            )
        }
        // Else from home, the taxi where it beats walking (`leisure_way`, `taxi::leisure_eta`):
        // booked on one, they wait at home till it's near (`taxi::book_leisure`).
        TaskKind::Visit { plot }
            if sim.riding.is_none() && sim.taxi.is_none() && taxi_from_home(sim, ctx, plot) =>
        {
            sim.current = None;
            sim.transition = Some(crate::life::Transition::Taxi {
                plot,
                directed: task.directed,
            });
            return;
        }
        TaskKind::Visit { plot } => {
            // (On foot: not to a car, whatever they were heading for before.)
            sim.bound_for_car = None;
            // At a house with door rules, up to its front door to knock (`life::knock_spot`).
            let p = &ctx.plots[plot as usize];
            let knock = (content.visits.door.is_some() && !p.public)
                .then(|| crate::life::knock_spot(nav, p))
                .flatten();
            let mut door = p.arrival_point();
            // A public place without an entry: the open ground nearest its middle, not whatever
            // stands there.
            if p.public && p.entry.is_none() {
                door = open_ground_near(nav, p, door).unwrap_or(door);
            }
            match knock {
                Some((t, at, _)) => (at, t, content.visits.tags),
                None => (
                    door,
                    (door[0].floor() as i32, door[1].floor() as i32),
                    content.visits.tags,
                ),
            }
        }
        // Their car parked out on a trip: back to it, to drive home (`cars::car_out_for`).
        TaskKind::GoHome if let Some((car, door, at)) = crate::cars::car_out_for(ctx, sim) => {
            sim.bound_for_car = Some(car);
            sim.car_seat = 0;
            sim.car_door = door as u8;
            goal_at(at)
        }
        // The car they came in on a trip, parked out there: back to it, to ride home
        // (`cars::ride_home_for`).
        TaskKind::GoHome
            if let Some((car, seat, door, at)) = crate::cars::ride_home_for(ctx, sim) =>
        {
            sim.bound_for_car = Some(car);
            sim.car_seat = seat as u8;
            sim.car_door = door as u8;
            goal_at(at)
        }
        TaskKind::GoHome => {
            // On their bike: back to its place (`ride.rs`); left out by the place they're at:
            // over to it first, to ride home (`Sim::bike_parked`).
            if let Some(bike) = sim.riding.and_then(|b| objects.get(b as usize)) {
                goal_at(crate::ride::spot(content, bike))
            } else if let (Some(_), Some(at)) = (sim.bound_for_bike, sim.bike_parked) {
                goal_at(at)
            } else {
                let home = ctx.households[sim.household as usize].plot;
                let Some(door) = home.map(|p| ctx.plots[p as usize].arrival_point()) else {
                    sim.transition = Some(crate::life::Transition::Home);
                    return;
                };
                goal_at(door)
            }
        }
        TaskKind::Answer { guest } => {
            let Some(other) = ctx.briefs.get(guest as usize) else {
                return;
            };
            let Some(goal) = conversation::approach_tile(nav, other.pos, sim.pos) else {
                return;
            };
            ([goal.0 as f32 + 0.5, goal.1 as f32 + 0.5], goal, 0)
        }
        TaskKind::Social { target, social } => {
            let Some(other) = ctx.briefs.get(target as usize) else {
                return;
            };
            if target == sim.id {
                return;
            }
            sim.social_waited = 0;
            // Within earshot: they call out to them, and the two meet halfway (see
            // `conversation::update`); otherwise over to them until they are (or right there).
            sim.called = other.available
                && conversation::calls_out(nav, &content.socials[social], sim.pos, other.pos);
            if sim.called {
                sim.pose = Pose::Stand;
                let tags = content.socials[social].tags;
                sim.current = Some(Activity {
                    task,
                    phase: Phase::Calling { ticks: 0 },
                    tags,
                });
                return;
            }
            let Some(goal) = conversation::approach_tile(nav, other.pos, sim.pos) else {
                return;
            };
            (
                [goal.0 as f32 + 0.5, goal.1 as f32 + 0.5],
                goal,
                content.socials[social].tags,
            )
        }
        // Over to the group (`conversation::update` finds them a place in it).
        TaskKind::Join { leader } => {
            let Some(other) = ctx.briefs.get(leader as usize) else {
                return;
            };
            let Some((_, social)) = other
                .conversing
                .filter(|c| c.0 == leader && leader != sim.id)
            else {
                return;
            };
            let Some(goal) = conversation::approach_tile(nav, other.pos, sim.pos) else {
                return;
            };
            (
                [goal.0 as f32 + 0.5, goal.1 as f32 + 0.5],
                goal,
                content.socials[social].tags,
            )
        }
    };
    // A place on the seat for the use (two housemates don't both sit on the same sofa).
    if let Some(seat) = reserved_seat.take() {
        let def = &content.objects[objects[seat as usize].def];
        let footstool = footstool_slots(content, objects, seat);
        if objects[seat as usize].reserve_seat(sim.id, def, sim.pos, footstool) {
            sim.use_seat = Some(seat);
        }
    }
    // No way to the tile by their slot (beside a lounger, shut in by the beds round it), nor to
    // the seat near it they'd take: from another tile it's used from, standing, as buying left a
    // way to one (`keeps_reach`).
    let mut waypoints = walk_route(ctx, sim, goal, target);
    if waypoints.is_none()
        && let TaskKind::Use { object, .. } = task.kind
        && (sim.use_seat.is_none() || seat_near)
    {
        if let Some(seat) = sim.use_seat.take() {
            objects[seat as usize].release(sim.id);
        }
        let tiles = crate::surface::use_tiles(content, objects, nav, &objects[object as usize]);
        waypoints = tiles
            .into_iter()
            .filter(|&t| t != goal)
            .find_map(|t| walk_route(ctx, sim, t, [t.0 as f32 + 0.5, t.1 as f32 + 0.5]));
    }
    let Some(waypoints) = waypoints else {
        if let Some(seat) = sim.use_seat.take() {
            objects[seat as usize].release(sim.id);
        }
        return;
    };
    if let TaskKind::Visit { plot } = task.kind {
        sim.transition = Some(crate::life::Transition::Visit {
            plot,
            directed: task.directed,
        });
    }
    // A place at it for the repair (two housemates don't both set off for it).
    if let TaskKind::Repair { object } = task.kind {
        let obj = &mut objects[object as usize];
        if !obj.reserve(sim.id, content.objects[obj.def].slots) {
            return;
        }
    }
    if let TaskKind::Use {
        object,
        interaction,
    } = task.kind
    {
        let sit = content.objects[objects[object as usize].def].interactions[interaction].pose
            == Pose::Sit;
        let footstool = if sit {
            footstool_slots(content, objects, object)
        } else {
            0
        };
        let obj = &mut objects[object as usize];
        if !obj.reserve_for(
            sim.id,
            &content.objects[obj.def],
            interaction,
            sim.pos,
            footstool,
        ) {
            if let Some(seat) = sim.use_seat.take() {
                objects[seat as usize].release(sim.id);
            }
            return;
        }
        // Off to bed (they can get there): the toilet first, out of habit, and then they
        // choose bed again.
        let sleep = content.objects[obj.def].interactions[interaction].tags
            & content.day_rhythm.sleep_tags
            != 0;
        if sleep
            && !task.directed
            && let Some(first) = habit(
                sim,
                ctx,
                objects,
                HabitTime::Bed,
                ctx.object_plot[object as usize],
            )
        {
            objects[object as usize].release(sim.id);
            if let Some(seat) = sim.use_seat.take() {
                objects[seat as usize].release(sim.id);
            }
            return start_task(sim, first, ctx, objects, items);
        }
    }
    // Off to a game for two: someone to play with is found after the step (`partners::update`),
    // unless they're the one fetched.
    if let TaskKind::Use {
        object,
        interaction,
    } = task.kind
        && content.objects[objects[object as usize].def].interactions[interaction].partner
    {
        sim.wants_partner = joins.is_none();
        sim.partner_wait = 0;
    }
    sim.serve_wait = 0;
    sim.pose = Pose::Stand;
    sim.current = Some(Activity {
        task,
        phase: Phase::Routing { waypoints, next: 0 },
        tags,
    });
}

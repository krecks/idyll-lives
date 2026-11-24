//! Choosing: what a Sim does on their own when nothing is asked of them.

use super::*;
use crate::content::{MAX_NEEDS, TagMask, stage_in};
use crate::conversation;
use crate::rng::Rng;
use crate::social::{self, Prefer};
use crate::world::{
    Access, Choice, Ctx, Goer, MIN_AUTONOMY_SCORE, OUTING_K, ObjectInstance, Sim, Task, TaskKind,
    UNWANTED_MEAL, access, affordable, before_dawn, breaks_off, could_conceive, exhausted,
    free_before_work, getting_up, guest_may, has_bed, home_first, home_needs, hosts_about_till,
    lasts, leave_today, leisure_way, outing_fits, outing_pull, place_serves, quality_boost, reach,
    requirements_met, runs_out, training_interest, unfit, upper_barred, urgent_needs, usable_here,
    visit_fits,
};
use crate::{MINUTES_PER_TICK, TICKS_PER_MINUTE, ai};

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
    // (Each part's time, with the `profile` feature: `pick_autonomous/...`.)
    let mut part = crate::profile::start();
    let needs = &sim.needs[..content.needs.len()];
    // How hard each need pulls (more below the level they like to keep it at).
    let felt = ai::felt(needs, &content.needs, &content.autonomy);
    let emotion = sim.emotion(content).map(|e| &content.emotions[e].mods);
    let schedule = content.schedule_at(ctx.hour);
    let routines = &ctx.households[sim.household as usize].routines;
    let night = crate::planner::planned_night(content, sim, routines, ctx.day, ctx.minute);
    // Night quiet (content `dayRhythm.nightQuiet`): noisy things held back, quiet ones favoured;
    // urgent needs still win through the usual look-ahead scoring.
    let night_now = night.unwrap_or_else(|| crate::world::rhythm_night(sim, ctx));
    let quiet = &content.day_rhythm.night_quiet;
    // Traits, the current emotion and the time of day all bias what feels right.
    let feel = |tags: TagMask| {
        sim.mods.preference(tags)
            * emotion.map_or(1.0, |m| m.preference(tags))
            * schedule.preference(tags)
            * if night_now { quiet.factor(tags) } else { 1.0 }
    };
    let me = sim.id as usize;
    let here = ctx.briefs[me].plot;
    let mut candidates: Vec<(f32, Choice)> = Vec::new();

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
        || getting_up(sim, ctx, false);
    // Only what they may use: at home and in public places, or where they're a guest.
    let usable = usable_here(sim, ctx);
    let usable: &[u32] = &usable;
    // Cooking: the stations they have (recipes need theirs), and whether food is ready.
    let stations = crate::cooking::station_mask(sim, ctx, objects);
    let fresh = items.free_servings(sim.household, sim.id).next().is_some();
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
    // Home first: the essentials home has something free for (a bed, a snack, the toilet, a
    // wash) are seen to at home, not at the hotel or the café down the road (`home_first`).
    let away = |o: u32| {
        ctx.object_plot[o as usize]
            .is_some_and(|p| !crate::world::at_home(ctx.plots, own, p, false))
    };
    let homed = if sim.visiting.is_none() && usable.iter().any(|&o| away(o)) {
        home_first(sim, ctx, objects, walk_per_minute)
    } else {
        0
    };
    // (A bed elsewhere is no bed for them then.)
    let home_beds: Vec<u32>;
    let beds: &[u32] = match rhythm.rest_need {
        Some(n) if homed & (1 << n) != 0 => {
            home_beds = usable.iter().copied().filter(|&o| !away(o)).collect();
            &home_beds
        }
        _ => usable,
    };
    // A spare bed's night (a sofa bed's, the sofa's) only for someone with no other bed free; a
    // couple's bed kept for the couple only for someone with nowhere else at all.
    let bedded = has_bed(sim, ctx, objects, beds, false);
    let elsewhere = std::cell::OnceCell::new();
    let elsewhere = || *elsewhere.get_or_init(|| bedded || has_bed(sim, ctx, objects, beds, true));
    crate::profile::end("pick_autonomous/setup", part);
    part = crate::profile::start();
    for obj in usable
        .iter()
        .map(|&o| &objects[o as usize])
        .filter(|o| o.has_free_slot(content.objects[o.def].slots))
    {
        let plot = ctx.object_plot[obj.id as usize];
        let public = plot.is_some_and(|p| ctx.plots[p as usize].public);
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
                || (inter.fills() & homed != 0 && away(obj.id))
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
                || !affordable(inter.cost_at(public), funds)
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
            // A tool that cleans the floor: only with a floor at home it cleans dirty enough,
            // the keener the dirtier, as tidying up (`sweep.rs`), from as far as that floor is.
            if let Some(sweep) = &inter.cleans {
                let at_home =
                    here == own && ctx.object_plot[obj.id as usize].is_some_and(|p| Some(p) == own);
                let Some(spot) = at_home
                    .then(|| sweep_spot(sim, ctx, sweep, content.room_rules.clean.threshold))
                    .flatten()
                else {
                    continue;
                };
                let distance = ctx
                    .nav
                    .stairs
                    .distance([spot.x as f32 + 0.5, spot.z as f32 + 0.5], sim.pos);
                s += crate::sweep::keen(content, &spot) * (1.5 - environment)
                    / (1.0 + distance * 0.08);
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

    crate::profile::end("pick_autonomous/objects", part);
    part = crate::profile::start();
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
        let heard = earshot.hears(other.pos);
        // Nor out on the street, minding the baby.
        if !heard || (minding && other.plot != own) {
            continue;
        }
        let distance = ctx.nav.stairs.distance(other.pos, sim.pos);
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
        let heard = earshot.hears(leader.pos);
        if !heard || (minding && leader.plot != own) {
            continue;
        }
        let distance = ctx.nav.stairs.distance(leader.pos, sim.pos);
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

    crate::profile::end("pick_autonomous/socials", part);
    part = crate::profile::start();
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
        let out_of_reach = cleaning_skip(sim, ctx);
        // (A flat's on its own storey; a house's on the ground.)
        for z in p.rows(ctx.nav.stairs) {
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
        for it in items.free_servings(sim.household, sim.id) {
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

    crate::profile::end("pick_autonomous/home", part);
    part = crate::profile::start();
    // The phone: texting, calling or inviting people who aren't here, scrolling (`phone.rs`).
    // (Nor a call or a text, pressed.)
    for (s, action, contact) in crate::phone::candidates(sim, ctx, &felt, &feel) {
        if s > MIN_AUTONOMY_SCORE && !(pressed && contact.is_some()) {
            candidates.push((s, Choice::Phone(action, contact)));
        }
    }

    crate::profile::end("pick_autonomous/phone", part);
    part = crate::profile::start();
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
            // (Only theirs: `Ctx::members`.)
            let mut friendship = f32::NEG_INFINITY;
            let mut opener = false;
            for &j in ctx.members.of(other.id) {
                let b = &ctx.briefs[j as usize];
                if b.household == other.id && b.plot == Some(plot) && !b.away {
                    friendship = friendship.max(ctx.rels.get(me, j as usize).friendship);
                    opener = opener || crate::life::opens_door(b, other.id, plot);
                }
            }
            #[cfg(debug_assertions)]
            {
                let (mut was, mut opens) = (f32::NEG_INFINITY, false);
                for (j, b) in ctx.briefs.iter().enumerate() {
                    if b.household == other.id && b.plot == Some(plot) && !b.away {
                        was = was.max(ctx.rels.get(me, j).friendship);
                        opens = opens || crate::life::opens_door(b, other.id, plot);
                    }
                }
                debug_assert!(was.to_bits() == friendship.to_bits() && opens == opener);
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

        // An outing: a public place out of reach, for what it has that home hasn't (and a little
        // for what home has too; never for the essentials home sees to: `outing_pull`), the less
        // the longer the way (`OUTING_K`): only one the way to takes `outingMinutes` at most, with
        // time for there and back and an hour there before work or school and before
        // `outingHomeBy` (`outing_fits`).
        let reach = content.autonomy.reach;
        let at_home = own.map_or([0.0; MAX_NEEDS], |h| {
            place_serves(ctx, objects, sim, h, false)
        });
        let essentials = content.autonomy.urgent & fillable;
        // The least the way to a place could take, the fastest way they might go (on a bike of
        // theirs, by car or the taxi, along the straight line: `Travel::eta_at_least`): one past
        // `visits.outingMinutes` even so is one `outing_fits` would leave out, and its way isn't
        // looked for.
        let by_car = goer.drives || ctx.taxis.takes(sim);
        let pace = walk_per_minute * goer.bike.map_or(1.0, |(speed, _)| speed.max(1.0));
        let least = |door: [f32; 2]| {
            let foot = ctx
                .travel
                .eta_at_least(pos, door, crate::travel::Mode::Foot, pace);
            if by_car {
                let drive =
                    ctx.travel
                        .eta_at_least(pos, door, crate::travel::Mode::Car, walk_per_minute);
                foot.min(drive * crate::travel::DRIVE_MINUTES)
            } else {
                foot
            }
        };
        for p in ctx.plots.iter().filter(|p| p.public) {
            let door = p.arrival_point();
            // (Nor one shut now: they don't set off to wait at its door, `start_task`.)
            if (door[0] - pos[0]).hypot(door[1] - pos[1]) <= reach
                || least(door) > visits.outing_minutes
                || !p.open_at(ctx.day, ctx.minute)
            {
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
            // Nor to one shut when they'd get there, or shutting before they could stay
            // `visits.leastMinutes` (its hours, `Plot::open_for`).
            if p.hours.is_some() {
                let at = ctx.tick + crate::mins(eta);
                let open = p.open_for(crate::clock::day(at), crate::clock::minute_of_day(at));
                if open < visits.least_minutes {
                    continue;
                }
            }
            // The best the place has for each need, as one day out; not with a need that
            // would be low by the time they could leave, that it has nothing for (`lasts`).
            let best = place_serves(ctx, objects, sim, p.id, false);
            if !lasts(content, sim, &best, eta + visits.least_minutes) {
                continue;
            }
            let pull = outing_pull(&best, &at_home, essentials);
            let score = ai::score_felt(needs, &felt, &pull, 0.0) * feel(visits.tags)
                / (1.0 + OUTING_K * eta);
            if score > MIN_AUTONOMY_SCORE {
                candidates.push((score, Choice::Outing(p.id)));
            }
        }
    }

    crate::profile::end("pick_autonomous/visits", part);
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

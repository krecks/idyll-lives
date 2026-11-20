//! One tick of using an object.

use super::*;

/// One tick of using an object (`minutes` long: more when catching up on a turn, see
/// `lod.rs`), with needs `pressing` (`urgent_needs`). Returns whether the Sim is done.
pub(crate) fn use_object(
    sim: &mut Sim,
    ctx: &Ctx,
    obj: &ObjectInstance,
    interaction: usize,
    elapsed: f32,
    minutes: f32,
    pressing: u32,
) -> bool {
    let content = ctx.content;
    let inter = &content.objects[obj.def].interactions[interaction];
    // Helping oneself where nobody serves is quicker than being served (`minutes_where`).
    let total = inter.minutes_where(ctx.served(obj.id));
    // Quality and skill scale what the Sim gets out of it (not what it costs).
    let boost = quality_boost(content, inter, obj.quality)
        * inter.skill_factor(&content.skill_rules, &sim.skills);
    let rhythm = &content.day_rhythm;
    // Napping: no other nap until a while after this one.
    if inter.tags & rhythm.nap_tags != 0 {
        sim.nap_until = ctx.tick + social::minutes_to_ticks(rhythm.nap_cooldown_hours * 60.0);
    }
    if inter.cares()
        && let Some(baby) = obj.users().find(|&u| u != sim.id)
    {
        let per_tick = inter.care.map(|g| g / total * minutes);
        sim.care_out = Some((baby, per_tick));
    }
    let (mut any_gain, mut satisfied, mut urgent, mut wakes) = (false, true, false, false);
    let (mut fills, mut emptying) = (0u32, false);
    let low = content.autonomy.urgent_below;
    for n in 0..content.needs.len() {
        let g = inter.gain_per_minute[n];
        if g > 0.0 {
            any_gain = true;
            fills |= 1 << n;
            let g = g * sim.mods.need_gain[n] * boost;
            sim.needs[n] = (sim.needs[n] + g * minutes).clamp(0.0, 1.0);
            satisfied &= sim.needs[n] >= 0.999;
        } else {
            if g < 0.0 {
                // Costs (a workout makes you hungry and sweaty).
                sim.needs[n] = (sim.needs[n] + g * minutes).clamp(0.0, 1.0);
                emptying |= sim.needs[n] < low && !content.needs[n].room;
            }
            // (Surroundings are never urgent: they follow the room, nothing fills them. Nor is
            // what it's for: a snack fetched to eat sitting down fills hunger only then.)
            urgent |= sim.needs[n] < low && !content.needs[n].room && inter.total_gain[n] <= 0.0;
            wakes |= sim.needs[n] < low && content.day_rhythm.wake_for & (1 << n) != 0;
        }
    }
    for (s, &gain) in inter.skill_gain[..content.skills.len()].iter().enumerate() {
        if gain > 0.0 {
            practise(sim, content, s, gain * boost, minutes);
        }
    }
    let sleep = inter.tags & rhythm.sleep_tags != 0;
    let done = if inter.baby_bed() {
        // A baby lies in the crib until they grow out of it (`lifecycle.rs`), asleep or awake:
        // needing something doesn't get them up (they can't: they cry, and a carer comes), nor
        // does being rested.
        false
    } else if inter.baby {
        // Left in the high chair (`carry.rs`): back in the crib once its time is up.
        elapsed >= inter.minutes
    } else if sleep && crate::planner::planned_now(content, sim, inter) {
        // A planned night's sleep lasts the block unless a need that wakes people gets urgent.
        wakes
    } else if sleep {
        // One sleep until rested (only the need it's for counts: about 8 hours from tired,
        // less in a better bed), but nobody gets up in the middle of the night: once rested,
        // they lie in until a little before they'd get up anyway. On a work or school day the
        // alarm goes off in time to get ready, rested or not.
        let rested = rhythm.rest_need.is_none_or(|n| sim.needs[n] >= 0.999);
        let longest = f32::from(content.planner.max_sleep_minutes);
        wakes || elapsed >= longest || (rested && !before_dawn(sim, ctx)) || getting_up(sim, ctx)
    } else {
        // Workouts and other draining activities stop before they empty the needs they cost.
        // Planned activities go on for their time even once the needs they fill are full.
        // Whatever they chose themselves (not a routine, nor for one of them) stops for a need
        // about to run out that home has something for, if it would before the end (off to the
        // toilet halfway through a film, not off the toilet for a bite to eat: a use that `lasts`
        // isn't left for another need, or worn out with the bladder low they'd be up from it
        // for bed and from bed for it, round and round, until they fell asleep on the floor).
        let planned = crate::planner::planned_now(content, sim, inter);
        let chosen = sim
            .current
            .as_ref()
            .is_some_and(|a| !a.task.directed && !a.task.routine);
        let left = total - elapsed;
        // Time to head home from where they're visiting (an outing's end, `visits.outingHomeBy`):
        // what they chose themselves there ends there and then (up off the park bench), so
        // they're home by the hour.
        let heading_home = sim.visiting.is_some_and(|v| v.arrived > 0)
            && sim.queue.iter().any(|t| matches!(t.kind, TaskKind::GoHome));
        let drained = emptying
            || (urgent && planned)
            || (chosen && heading_home)
            || (chosen
                && !inter.lasts
                && breaks_off(sim, content, pressing, fills, left, inter, low));
        let routine = sim.current.as_ref().is_some_and(|a| a.task.routine);
        // Practising a skill they still want to (as when choosing: at home or in a public place)
        // goes on for its time too, until practice gets boring; stopping it as soon as the needs
        // are full would have them back at it a moment later, over and over.
        let practising = satisfied
            && inter.trains_skills()
            && access(ctx, sim, ctx.object_plot[obj.id as usize]) == Access::Full
            && training_interest(sim, content, inter) > MIN_AUTONOMY_SCORE;
        // A job that changes the thing as the hands go (hanging out the washing, a harvest) goes on to
        // its end, clothes are changed all the way, whatever it does for comfort, and a use that
        // `lasts` (the toilet) takes its time however soon the bladder is empty, once begun (one
        // with nothing to fill at all is given up on its first tick) and while it isn't time to
        // head home (an outing's end, `visits.outingHomeBy`); so does
        // loading a machine or emptying it, or anything that leaves a load to work on next.
        // So does a game for two: one stopping would end it for the other.
        let job = inter.gradual
            || inter.picks
            || inter.changes_outfit
            || (inter.lasts && elapsed > minutes && !heading_home)
            || inter.partner
            || inter.loads
            || inter.unloads
            || inter.makes.is_some()
            || inter.cleans.is_some();
        elapsed >= total
            || (any_gain && satisfied && !planned && !routine && !practising && !job)
            || drained
    };
    // Halfway through changing: in the next outfit, which views draw (a change of looks).
    let half = inter.minutes * 0.5;
    if inter.changes_outfit && elapsed >= half && elapsed - minutes < half {
        sim.outfit = (sim.outfit + 1) % OUTFITS;
        sim.reshaped = true;
        // At home with ironed washing waiting: pressed clothes put on (`content::Pressed`).
        let household = &ctx.households[sim.household as usize];
        if let Some(pressed) = content.laundry.and_then(|l| l.pressed)
            && household.laundry[crate::content::WorkOn::Ironed.load().unwrap_or(0)] > 0.0
            && household.plot.is_some()
            && ctx.object_plot.get(obj.id as usize).copied().flatten() == household.plot
        {
            sim.wore_pressed = true;
            social::add_feeling(
                &mut sim.feelings,
                pressed.feeling,
                &content.feelings,
                ctx.tick,
            );
        }
    }
    // Homework done: better at school (by how much of it they did).
    if done && inter.homework > 0.0 && content.life.stage_at(sim.age).is_some_and(|s| s.school) {
        let share = (elapsed / inter.minutes.max(1.0)).min(1.0);
        sim.grade = (sim.grade + inter.homework * share).clamp(0.0, 100.0);
    }
    // A floor cleaned with a tool: the dirt comes off it (after the step, `sweep::clean`), as
    // much as they got done of it.
    if done && let Some(sweep) = inter.cleans {
        let amount = sweep.amount * (elapsed / total.max(1.0)).min(1.0);
        sim.swept = sim
            .sweep
            .map(|(x, z)| (x, z, crate::content::Sweep { amount, ..sweep }));
    }
    // A finished cook (or wash, or snack) leaves a mess around the object.
    if done && inter.dirt > 0.0 {
        let (fx, fz) = ctx
            .index
            .links
            .approach_of(obj.id)
            .unwrap_or_else(|| obj.front_tile(content));
        sim.mess = Some((fx, fz, inter.dirt));
    }
    if done && let Some(m) = inter.earns_feeling_over(total, elapsed, &sim.skills) {
        social::add_feeling(&mut sim.feelings, m, &content.feelings, ctx.tick);
    }
    done
}

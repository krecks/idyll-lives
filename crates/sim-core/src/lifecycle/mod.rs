//! Life cycle: residents age once a day and move through life stages (content `life`).

use serde::{Deserialize, Serialize};

use crate::clock;
use crate::content::StageMask;
use crate::social::{self, EventKind, Kin};
use crate::world::{GoneWhy, Household, World};

/// How fast residents age (a per-game option): `Off` nobody does; the others scale content
/// `life.daysPerYear` (the *Normal* speed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Lifespan {
    Off,
    Short,
    #[default]
    Normal,
    Long,
}

impl Lifespan {
    /// Game days per year of age (`None`: nobody ages).
    pub fn days_per_year(self, normal: f32) -> Option<f32> {
        match self {
            Lifespan::Off => None,
            Lifespan::Short => Some(normal * 0.5),
            Lifespan::Normal => Some(normal),
            Lifespan::Long => Some(normal * 2.0),
        }
    }
}

/// A starting age for a resident nothing gave one: within content `life.startAge`, from a hash
/// of their name and slot (not the world's random numbers, so adding ages changed no run).
pub fn default_age(rules: &crate::content::LifeRules, name: &str, slot: usize) -> f32 {
    let mut h: u32 = 2_166_136_261 ^ slot as u32;
    for b in name.bytes() {
        h = (h ^ b as u32).wrapping_mul(16_777_619);
    }
    let [lo, hi] = rules.start_age;
    (lo + (h % 1000) as f32 / 1000.0 * (hi - lo)).floor()
}

/// Once a day at midnight: everyone a year-fraction older; a new life stage is a story event
/// and a feeling, and changes what the stage does to them.
pub(crate) fn update(w: &mut World) {
    // (Born tonight: not a day older already, below.)
    let mut born = Vec::new();
    if clock::at_minute(w.tick, 0.0) {
        born = births(w);
    }
    if let Some(rules) = w.content.life.moving
        && clock::at_minute(w.tick, rules.hour * 60.0)
        && w.autonomy
    {
        moving(w, &rules);
    }
    if let Some(rules) = w.content.life.newcomers
        && clock::at_minute(w.tick, rules.hour * 60.0)
        && w.autonomy
    {
        newcomers(w, &rules);
        neighbours_adopt(w);
    }
    if !clock::at_minute(w.tick, 0.0) {
        return;
    }
    // (None: nobody ages, so nobody reaches the retirement age or dies of old age.)
    let days_per_year = w.lifespan.days_per_year(w.content.life.days_per_year);
    let mut grew = Vec::new();
    let mut reached_retirement = Vec::new();
    for i in 0..w.sims.len() {
        let Some(days_per_year) = days_per_year else {
            break;
        };
        if !w.sims[i].here() || born.contains(&i) {
            continue;
        }
        let rules = &w.content.life;
        let sim = &mut w.sims[i];
        let before = rules.stage(sim.age);
        let was = sim.age;
        sim.age += 1.0 / days_per_year;
        if rules.retire_at.is_some_and(|at| was < at && sim.age >= at) {
            reached_retirement.push(i);
        }
        let stage = rules.stage(sim.age);
        if stage == before || rules.stages.is_empty() {
            continue;
        }
        if let Some(f) = rules.stages[stage].feeling {
            social::add_feeling(&mut sim.feelings, f, &w.content.feelings, w.tick);
        }
        // Into school, or out of it with a final grade.
        if let Some(school) = &rules.school {
            let (was, now) = (rules.stages[before].school, rules.stages[stage].school);
            if !was && now {
                sim.grade = school.start_grade;
            }
            if was && !now {
                let grade = sim.grade;
                let f = if grade >= school.good {
                    school.good_feeling
                } else if grade < school.poor {
                    school.poor_feeling
                } else {
                    None
                };
                if let Some(f) = f {
                    social::add_feeling(&mut sim.feelings, f, &w.content.feelings, w.tick);
                }
                w.events.push_detail(
                    w.tick,
                    EventKind::Graduated,
                    i,
                    Some(grade.round() as i64),
                    None,
                );
            }
        }
        let sim = &mut w.sims[i];
        // Out of the crib, onto their feet beside it (not left standing inside it, where
        // there's no way out).
        if rules.stages[before].baby && !rules.stages[stage].baby {
            crate::world::end_activity(sim, &w.content, &mut w.objects);
            sim.clear_activity();
            for o in &mut w.objects {
                o.release(i as u32);
            }
        }
        grew.push(i);
        w.refresh_base_mods(i);
        w.structure_version = w.structure_version.wrapping_add(1);
        w.events
            .push_detail(w.tick, EventKind::GrewOlder, i, Some(stage as i64), None);
    }
    // (Out of the crib, or into a stage their bed isn't for.)
    for i in grew {
        bed_for(w, i);
    }
    for i in 0..w.sims.len() {
        if w.sims[i].here() {
            retire_if_due(w, i, reached_retirement.contains(&i));
        }
    }
    let Some(days_per_year) = days_per_year else {
        return;
    };
    for i in 0..w.sims.len() {
        let Some(death) = w.content.life.death else {
            break;
        };
        if !w.sims[i].here() || w.sims[i].age < death.from {
            continue;
        }
        // A day's share of the year's chance.
        let chance = 1.0 - (1.0 - death.yearly_chance(w.sims[i].age)).powf(1.0 / days_per_year);
        if w.rng.next_f32() < chance {
            pass_away(w, i);
        }
    }
}

/// Retiring, with a weekly pension for the household: a working resident on the day they reach
/// the retirement age (`reached`; a job they take after that is theirs to keep), and anyone at or
/// past it without a job (who never had one, or lost or quit one) on any day.
fn retire_if_due(w: &mut World, i: usize, reached: bool) {
    let rules = &w.content.life;
    let Some(at) = rules.retire_at else { return };
    let sim = &w.sims[i];
    if sim.retired || sim.age < at || (sim.job.is_some() && !reached) {
        return;
    }
    let job = sim.job.as_ref().map(|j| (j.career, j.level));
    let feeling = rules.retire_feeling;
    let pension = pension(&w.content, sim.job.as_ref());
    let sim = &mut w.sims[i];
    sim.retired = true;
    sim.pension = pension;
    sim.job = None;
    crate::cars::job_lost(&mut w.cars, i as u32);
    if let Some(f) = feeling {
        social::add_feeling(&mut sim.feelings, f, &w.content.feelings, w.tick);
    }
    // (The story tells of leaving a job; retiring without one passes quietly.)
    if let Some((career, grade)) = job {
        w.events
            .push_career(w.tick, EventKind::Retired, i, career, Some(grade as i64));
    }
}

/// The weekly pension of someone retiring from `job` (None: they had none): content
/// `life.pension` × its weekly pay, and at least `life.basicPension`.
pub(crate) fn pension(content: &crate::content::Content, job: Option<&crate::life::Job>) -> i64 {
    let share = job.map_or(0, |job| {
        let level = &content.careers[job.career].levels[job.level];
        let weekly = level.pay * i64::from(level.days.count_ones());
        (weekly as f32 * content.life.pension).round() as i64
    });
    share.max(content.life.basic_pension)
}

/// Old age: the story tells it, family, partners and friends grieve, and they're gone.
pub fn pass_away(w: &mut World, i: usize) {
    let grief = w.content.life.grief;
    for j in 0..w.sims.len() {
        if j == i || !w.sims[j].here() {
            continue;
        }
        let r = *w.relationships.get(j, i);
        let feeling = if r.kin != Kin::None || r.partners || r.friendship >= grief.close {
            grief.feeling
        } else if r.friendship >= grief.friend {
            grief.light_feeling
        } else {
            None
        };
        if let Some(f) = feeling {
            social::add_feeling(&mut w.sims[j].feelings, f, &w.content.feelings, w.tick);
        }
    }
    w.events.push_detail(w.tick, EventKind::Died, i, None, None);
    w.depart(i, GoneWhy::Died);
    // The last grown-up of a household gone, its children go to their other parent in town, else
    // to family out of town: nobody would look after them, earn or pay the rent. (Not the
    // player's: they play on with the children.)
    let h = w.sims[i].household as usize;
    let left = members(w, h);
    if !w.households[h].player && !left.iter().any(|&j| w.sims[j].adult(&w.content)) {
        for j in left {
            let parent = (0..w.sims.len()).find(|&p| {
                w.sims[p].here()
                    && w.sims[p].adult(&w.content)
                    && w.relationships.kin(j, p) == Kin::Parent
            });
            if let Some(p) = parent {
                let to = w.sims[p].household as usize;
                moving::move_to(w, j, to);
                w.events.push(w.tick, EventKind::MovedInWith, j, p, None);
            } else {
                w.events
                    .push_detail(w.tick, EventKind::MovedAway, j, None, None);
                w.depart(j, GoneWhy::MovedAway);
            }
        }
    }
}

mod arrival;
mod babies;
mod children;
mod moving;

pub use arrival::arrive;
pub(crate) use arrival::*;
pub(crate) use babies::*;
pub use children::adopt;
pub(crate) use children::*;
use moving::*;

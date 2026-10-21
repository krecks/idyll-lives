//! Utility AI: objects advertise need gains; Sims score them against their current needs.

use serde::Deserialize;

use crate::content::{Interaction, MAX_NEEDS, NeedDef};

/// How residents weigh their needs when they choose for themselves (`"autonomy"` in content).
#[derive(Debug, Clone)]
pub struct AutonomyRules {
    /// Needs that get urgent (content `urgent`; bit per need, default none): the essentials.
    pub urgent: u32,
    /// One of those this low is urgent when home has something for it (a toilet, food):
    /// residents see to it before anything else, stop what they chose themselves for it, and
    /// don't stop to chat.
    pub urgent_below: f32,
    /// How much more a need below its `comfortable` level weighs: `need_weight` times
    /// (1 + this), eased in over the tenth below it.
    pub comfort_boost: f32,
    /// Multiplier on a choice that would let another need run down to `urgent_below` before
    /// it's over (a film started with the bladder at a quarter, a recipe started starving).
    pub look_ahead: f32,
    /// The longest a habit's top-up may take (the toilet, a snack; see `DayRhythm::habits`).
    pub quick_minutes: f32,
    /// Something they chose but couldn't get to isn't chosen again for this long.
    pub retry_minutes: f32,
    /// Needs a chat fills (any social's gains): running low on these is no reason to stop
    /// talking.
    pub social_fills: u32,
}

impl Default for AutonomyRules {
    fn default() -> Self {
        Self {
            urgent: 0,
            urgent_below: 0.08,
            comfort_boost: 0.0,
            look_ahead: 1.0,
            quick_minutes: 30.0,
            retry_minutes: 30.0,
            social_fills: 0,
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct AutonomyRaw {
    pub(crate) urgent: Option<Vec<String>>,
    urgent_below: Option<f32>,
    comfort_boost: Option<f32>,
    look_ahead: Option<f32>,
    quick_minutes: Option<f32>,
    retry_minutes: Option<f32>,
}

impl AutonomyRaw {
    /// The rules, with the `urgent` needs and those a chat fills (bit per need) worked out.
    pub(crate) fn rules(&self, urgent: Option<u32>, social_fills: u32) -> AutonomyRules {
        let d = AutonomyRules::default();
        AutonomyRules {
            urgent: urgent.unwrap_or(d.urgent),
            urgent_below: self.urgent_below.unwrap_or(d.urgent_below).clamp(0.0, 1.0),
            comfort_boost: self.comfort_boost.unwrap_or(d.comfort_boost).max(0.0),
            look_ahead: self.look_ahead.unwrap_or(d.look_ahead).clamp(0.0, 1.0),
            quick_minutes: self.quick_minutes.unwrap_or(d.quick_minutes).max(0.0),
            retry_minutes: self.retry_minutes.unwrap_or(d.retry_minutes).max(0.0),
            social_fills,
        }
    }
}

/// How strongly a need at `value` (0 = empty, 1 = full) pulls a Sim. Low needs dominate.
pub fn need_weight(value: f32) -> f32 {
    let deficit = (1.0 - value).clamp(0.0, 1.0);
    deficit * deficit * 3.0 + deficit * 0.5
}

/// How strongly each of `needs` pulls a resident: `need_weight`, more below the need's
/// `comfortable` level (the bladder around half: seen to when it's convenient, not once it's
/// bursting).
pub fn felt(needs: &[f32], defs: &[NeedDef], rules: &AutonomyRules) -> [f32; MAX_NEEDS] {
    let mut weights = [0.0; MAX_NEEDS];
    for (n, (&value, def)) in needs.iter().zip(defs).enumerate() {
        let below = ((def.comfortable - value) / 0.1).clamp(0.0, 1.0);
        weights[n] = need_weight(value) * (1.0 + rules.comfort_boost * below);
    }
    weights
}

/// Attractiveness of an interaction for a Sim with `needs`, `distance` metres away.
pub fn score(needs: &[f32], interaction: &Interaction, distance: f32) -> f32 {
    score_gains(needs, &interaction.total_gain, distance)
}

/// Attractiveness of total need gains `gains`, `distance` metres away.
pub fn score_gains(needs: &[f32], gains: &[f32; MAX_NEEDS], distance: f32) -> f32 {
    let mut weights = [0.0; MAX_NEEDS];
    for (w, &value) in weights.iter_mut().zip(needs) {
        *w = need_weight(value);
    }
    score_felt(needs, &weights, gains, distance)
}

/// Attractiveness of total need gains `gains`, `distance` metres away, for needs that pull
/// as hard as `weights` (see `felt`).
pub fn score_felt(needs: &[f32], weights: &[f32], gains: &[f32; MAX_NEEDS], distance: f32) -> f32 {
    let mut s = 0.0;
    for (i, &value) in needs.iter().enumerate() {
        let gain = gains[i].min(1.0 - value).max(0.0);
        s += gain * weights[i];
    }
    s / (1.0 + distance * 0.08)
}

/// Picks among the best candidates with weighted randomness, so Sims are not robotic.
/// `candidates` are `(score, payload)`; returns the chosen payload.
pub fn choose<T: Copy>(candidates: &mut [(f32, T)], roll: f32, top_n: usize) -> Option<T> {
    debug_assert!(candidates.iter().all(|c| c.0.is_finite()), "a choice scored {:?}", candidates.iter().find(|c| !c.0.is_finite()).map(|c| c.0));
    pick(candidates, roll, top_n)
}

/// `choose`, leaving out scores that aren't numbers (NaN or infinity would make the pick the
/// last of the best every time).
fn pick<T: Copy>(candidates: &mut [(f32, T)], roll: f32, top_n: usize) -> Option<T> {
    let score = |c: &(f32, T)| if c.0.is_finite() { c.0 } else { f32::NEG_INFINITY };
    candidates.sort_unstable_by(|a, b| score(b).total_cmp(&score(a)));
    let scored = candidates.iter().take_while(|c| c.0.is_finite()).count();
    let top = &candidates[..scored.min(top_n)];
    let total: f32 = top.iter().map(|c| c.0).sum();
    if total <= 0.0 {
        return None;
    }
    let mut pick = roll * total;
    for c in top {
        if pick < c.0 {
            return Some(c.1);
        }
        pick -= c.0;
    }
    top.last().map(|c| c.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::Pose;

    fn interaction(need: usize, gain: f32) -> Interaction {
        let mut total_gain = [0.0; MAX_NEEDS];
        total_gain[need] = gain;
        Interaction {
            id: "x".into(),
            label: "x".into(),
            minutes: 10.0,
            total_gain,
            gain_per_minute: total_gain.map(|g| g / 10.0),
            pose: Pose::Stand,
            inside: false,
            autonomous: true,
            tags: 0,
            requires: 0,
            skill_gain: [0.0; crate::content::MAX_SKILLS],
            cost: 0,
            skill: None,
            feeling: None,
            feeling_min_skill: 0.0,
            anim: None,
            dirt: 0.0,
            baby: false,
            needs_table: false,
            whole: false,
            stages: crate::content::StageMask::MAX,
            seat_near: 0.0,
            seat_facing: None,
            broadcast: 0.0,
            broadcast_range: 6.0,
            recipe: None,
            homework: 0.0,
            care: [0.0; MAX_NEEDS],
            after: 0,
            eats: None,
            when: None,
            leaves: None,
            close_after: false,
            gradual: false,
            fetch: None,
            ripe: false,
            picks: false,
            changes_outfit: false,
            spare: false,
            tends: 0.0,
            yields: None,
            carried: false,
            at_slot: false,
            hours: None,
            sky: false,
            work_on: None,
        }
    }

    #[test]
    fn low_need_beats_high_need() {
        let needs = [0.1, 0.9];
        assert!(
            score(&needs, &interaction(0, 0.5), 5.0) > score(&needs, &interaction(1, 0.5), 5.0)
        );
    }

    #[test]
    fn closer_is_better() {
        let needs = [0.3];
        assert!(
            score(&needs, &interaction(0, 0.5), 1.0) > score(&needs, &interaction(0, 0.5), 20.0)
        );
    }

    #[test]
    fn below_comfortable_weighs_more() {
        let need = |comfortable| NeedDef { id: "n".into(), label: "n".into(), decay_per_minute: 0.0, room: false, comfortable };
        let defs = [need(0.6), need(0.0)];
        let rules = AutonomyRules { comfort_boost: 2.0, ..Default::default() };
        let w = felt(&[0.45, 0.45], &defs, &rules);
        assert!((w[0] - need_weight(0.45) * 3.0).abs() < 1e-5, "well below comfortable: three times the weight");
        assert!((w[1] - need_weight(0.45)).abs() < 1e-5, "no comfortable level: as before");
        let eased = felt(&[0.55, 0.55], &defs, &rules);
        assert!(eased[0] > eased[1] && eased[0] < need_weight(0.55) * 3.0, "eased in just below it");
    }

    #[test]
    fn choose_respects_top_n() {
        let mut c = [(0.1, 'a'), (5.0, 'b'), (0.2, 'c')];
        assert_eq!(choose(&mut c, 0.99, 1), Some('b'));
    }

    /// A score that isn't a number never wins (it would have made every pick the last of the
    /// best): it's left out.
    #[test]
    fn a_score_that_isnt_a_number_never_wins() {
        for roll in [0.0, 0.5, 0.99] {
            let mut c = [(f32::NAN, 'n'), (1.0, 'a'), (f32::INFINITY, 'i'), (0.5, 'b')];
            let got = pick(&mut c, roll, 3);
            assert!(matches!(got, Some('a' | 'b')), "roll {roll}: {got:?}");
        }
        assert_eq!(pick(&mut [(f32::NAN, 'n')], 0.5, 3), None);
    }

    /// In debug builds a score that isn't a number is a bug to look into.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "a choice scored")]
    fn a_score_that_isnt_a_number_is_a_bug() {
        choose(&mut [(f32::NAN, 'n'), (1.0, 'a')], 0.5, 3);
    }
}

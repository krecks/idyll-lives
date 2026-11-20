//! Relationships and feelings
//!
//! The directional relationship matrix, family links and active feelings.
//!
//! Split from `social.rs` (see `mod.rs`).

use super::*;

// ---- Relationships ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Relationship {
    /// -100..100
    pub friendship: f32,
    /// -100..100
    pub romance: f32,
    pub met: bool,
    pub partners: bool,
    pub kissed: bool,
    /// Family: what the other one is to this one (`rel(a, b).kin == Parent`: b is a's parent).
    #[serde(default, skip_serializing_if = "Kin::is_none")]
    pub kin: Kin,
}

/// A family link (partners are `Relationship::partners`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Kin {
    #[default]
    None,
    Parent,
    Child,
    Sibling,
}

impl Kin {
    pub fn is_none(&self) -> bool {
        *self == Kin::None
    }

    /// The same link seen from the other side.
    pub fn inverse(self) -> Kin {
        match self {
            Kin::Parent => Kin::Child,
            Kin::Child => Kin::Parent,
            k => k,
        }
    }
}

/// An event reference with this bit set names a former resident (`World::former`, the rest of
/// the bits its index) rather than a resident slot: their slot has someone new in it.
pub const FORMER: u32 = 1 << 31;

/// Residents the relationship matrix makes room for at a time (`Relationships::grow`).
pub(super) const GROW_BY: usize = 64;

/// Directional relationship matrix plus symmetric chemistry. Sized to the Sim count, its rows
/// `stride` long: room is made `GROW_BY` residents at a time, so a town loading or someone moving
/// in doesn't copy the whole matrix for each one (a full town's is megabytes).
#[derive(Debug, Clone, Default)]
pub struct Relationships {
    n: usize,
    /// Residents the matrix has room for (`n` or more; the rows and columns past `n` untouched).
    stride: usize,
    rel: Vec<Relationship>,
    chemistry: Vec<f32>,
    /// Each one's partner, kept with `rel` (asked every tick for everyone; a row scan each
    /// time was a tenth of a big town's tick). Partners change through `set_partners`.
    partner: Vec<Option<u32>>,
    /// Bumped when a family link or a partnership changes (`World::claimants_cache` reads it).
    version: u64,
}

impl Relationships {
    pub fn len(&self) -> usize {
        self.n
    }

    /// Bumped when a family link or a partnership changes.
    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// Grows to `n` Sims, keeping existing data.
    pub fn grow(&mut self, n: usize) {
        if n <= self.n {
            return;
        }
        if n > self.stride {
            let stride = n.next_multiple_of(GROW_BY);
            let mut rel = vec![Relationship::default(); stride * stride];
            let mut chemistry = vec![0.0; stride * stride];
            for a in 0..self.n {
                let (from, to) = (a * self.stride, a * stride);
                rel[to..to + self.n].copy_from_slice(&self.rel[from..from + self.n]);
                chemistry[to..to + self.n].copy_from_slice(&self.chemistry[from..from + self.n]);
            }
            (self.stride, self.rel, self.chemistry) = (stride, rel, chemistry);
        }
        self.n = n;
        self.partner.resize(n, None);
        self.version = self.version.wrapping_add(1);
    }

    /// Forgets everything between `a` and everyone (they're gone, or someone new took the slot).
    pub fn clear(&mut self, a: usize) {
        for b in 0..self.n {
            self.rel[a * self.stride + b] = Relationship::default();
            self.rel[b * self.stride + a] = Relationship::default();
            self.chemistry[a * self.stride + b] = 0.0;
            self.chemistry[b * self.stride + a] = 0.0;
        }
        for b in 0..self.n {
            if b == a || self.partner[b] == Some(a as u32) {
                self.recount(b);
            }
        }
        self.version = self.version.wrapping_add(1);
    }

    pub fn get(&self, a: usize, b: usize) -> &Relationship {
        &self.rel[a * self.stride + b]
    }

    /// The relationship to change (partners change with `set_partners`).
    pub fn get_mut(&mut self, a: usize, b: usize) -> &mut Relationship {
        &mut self.rel[a * self.stride + b]
    }

    /// Replaces how `a` sees `b` (loading a game).
    pub fn set(&mut self, a: usize, b: usize, rel: Relationship) {
        *self.get_mut(a, b) = rel;
        self.recount(a);
        self.version = self.version.wrapping_add(1);
    }

    /// Makes `a` and `b` partners, or not, both ways.
    pub fn set_partners(&mut self, a: usize, b: usize, partners: bool) {
        self.get_mut(a, b).partners = partners;
        self.get_mut(b, a).partners = partners;
        self.recount(a);
        self.recount(b);
        self.version = self.version.wrapping_add(1);
    }

    /// Looks up `a`'s partner again after a change.
    fn recount(&mut self, a: usize) {
        self.partner[a] = (0..self.n)
            .find(|&b| b != a && self.get(a, b).partners)
            .map(|b| b as u32);
    }

    pub fn chemistry(&self, a: usize, b: usize) -> f32 {
        self.chemistry[a * self.stride + b]
    }

    pub fn set_chemistry(&mut self, a: usize, b: usize, value: f32) {
        let v = value.clamp(-1.0, 1.0);
        self.chemistry[a * self.stride + b] = v;
        self.chemistry[b * self.stride + a] = v;
    }

    /// Applies a preset in both directions and marks the pair as having met (family links
    /// are kept unless the preset sets one).
    pub fn bond(&mut self, a: usize, b: usize, preset: &BondPreset) {
        for (x, y, kin) in [(a, b, preset.kin), (b, a, preset.kin.inverse())] {
            let r = self.get_mut(x, y);
            r.met = true;
            r.friendship = preset.friendship.clamp(-100.0, 100.0);
            r.romance = preset.romance.clamp(-100.0, 100.0);
            r.partners = preset.partners;
            if !kin.is_none() {
                r.kin = kin;
            }
        }
        self.recount(a);
        self.recount(b);
        self.version = self.version.wrapping_add(1);
    }

    /// What `b` is to `a` in the family.
    pub fn kin(&self, a: usize, b: usize) -> Kin {
        self.get(a, b).kin
    }

    /// Current partner of `a`, if any.
    pub fn partner_of(&self, a: usize) -> Option<usize> {
        let partner = self.partner.get(a).copied().flatten().map(|b| b as usize);
        debug_assert_eq!(
            partner,
            (0..self.n).find(|&b| b != a && self.get(a, b).partners),
            "partners changed without `set_partners`"
        );
        partner
    }

    pub fn adjust(&mut self, a: usize, b: usize, friendship: f32, romance: f32) {
        let r = self.get_mut(a, b);
        r.friendship = (r.friendship + friendship).clamp(-100.0, 100.0);
        r.romance = (r.romance + romance).clamp(-100.0, 100.0);
    }

    /// Daily drift toward neutral; partners' romance fades more slowly.
    pub fn decay(&mut self, rules: &SocialRules) {
        for r in &mut self.rel {
            if r.friendship.abs() > 15.0 {
                r.friendship -=
                    r.friendship.signum() * rules.friendship_decay.min(r.friendship.abs() - 15.0);
            }
            let romance_decay = if r.partners {
                rules.romance_decay / 3.0
            } else {
                rules.romance_decay
            };
            if r.romance.abs() > 0.0 {
                r.romance -= r.romance.signum() * romance_decay.min(r.romance.abs());
            }
        }
    }
}

// ---- Feelings ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActiveFeeling {
    pub def: usize,
    pub expires: u64,
}

pub fn minutes_to_ticks(minutes: f32) -> u64 {
    (minutes * crate::TICKS_PER_MINUTE as f32).max(1.0) as u64
}

/// Adds a feeling, or refreshes it if already active.
pub fn add_feeling(list: &mut Vec<ActiveFeeling>, def: usize, defs: &[FeelingDef], tick: u64) {
    let expires = tick + minutes_to_ticks(defs[def].minutes);
    match list.iter_mut().find(|m| m.def == def) {
        Some(m) => m.expires = m.expires.max(expires),
        None => list.push(ActiveFeeling { def, expires }),
    }
}

/// The emotion of the strongest active feeling that has one.
pub fn dominant_emotion(list: &[ActiveFeeling], defs: &[FeelingDef]) -> Option<usize> {
    list.iter()
        .filter_map(|m| defs[m.def].emotion.map(|e| (defs[m.def].mood.abs(), e)))
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, e)| e)
}

pub fn feeling_mood(list: &[ActiveFeeling], defs: &[FeelingDef]) -> f32 {
    list.iter().map(|m| defs[m.def].mood).sum()
}

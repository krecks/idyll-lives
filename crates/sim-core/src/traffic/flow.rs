//! Traffic itself: who is moving, who is waiting and who has been asked to wait.

use super::*;

mod begin;
mod go;
mod lights;

/// A car on the road this tick, as the others keep out of its way (`Traffic::begin`): its
/// key (`cars::Driving::first`), its body, the way it goes and how far it can still pull over
/// (`Sample::pass`); whether it moved last tick, and where it'll be along its way in the next ticks
/// (`LOOK`); and whether it stands parked to all intents (a car still in its driveway, not yet let
/// out).
#[derive(Debug, Clone)]
pub(crate) struct Mover {
    pub key: u64,
    pub rect: Rect,
    pub way: [f32; 2],
    pub pass: f32,
    pub moving: bool,
    pub ahead: Vec<Rect>,
    pub parked: bool,
    /// Whom it waited for last tick (`Driving::first`), if anyone.
    pub waits: Option<u64>,
    /// Its level (`travel::Layer`): in the way only of claims on levels that meet it.
    pub layer: crate::travel::Layer,
}

/// Where a vehicle is on its route, for its claims (`Traffic::begin`): its key, the tag of its
/// route's claims (whether it's on its way home) and how far along it is (ticks).
pub(crate) type Place = (u64, u32, f32);

#[derive(Debug, Clone)]
pub(super) struct Hold {
    pub(super) key: u64,
    tag: u32,
    claim: Claim,
    /// How far along its route its holder was as the tick began.
    at: f32,
}

/// A vehicle's ask for the parts of a group of its route's claims (`Claim::group`), in order:
/// granted all at once, in the order the vehicles asked (`tick`).
#[derive(Debug, Clone)]
pub(super) struct Ask {
    tick: u64,
    pub(super) key: u64,
    route: u32,
    group: u32,
    parts: Vec<(u32, Claim)>,
    /// The last tick it waited at the start of the group (asking again: one not asked again is
    /// dropped).
    seen: u64,
    /// Whom it waited for as the tick began (their keys: their claims, bodies).
    pub(super) by: Vec<u64>,
    /// The earlier asks it waited for as the tick began (their keys).
    pub(super) fifo: Vec<u64>,
    /// Of `by`, those only on the move into its way, not in it: they're to stop short, it doesn't
    /// wait for them to go first (`begin`, rule 3).
    coming: Vec<u64>,
    /// Held up by nobody's claim nor an earlier ask, only by bodies: once those still to come
    /// keep clear (`blocked`), it's granted.
    bodies_only: bool,
    /// Long asked (`OLD_ASK`), whom it waits for, however far round (as the tick began, those only
    /// coming into its way left out): none of them keeps clear of it (`blocked`).
    reach: Vec<u64>,
    /// How far along its route its vehicle is (ticks, as the tick began or as it asked): past a
    /// stop line of its group's lights already, that light is behind it (`Claim::lights`).
    at: f32,
}

/// How far short of what's left of another's claim one waits (m, `Traffic::blocked`).
const KEEP_SHORT: f32 = 1.5;

/// How far short of the start of its next claim one not holding a claim stops, where waiting
/// there it would be in the way of a group asked for long ago (`OLD_ASK`, m: `Traffic::blocked`).
const STOP_SHORT: f32 = 3.0;

/// An ask not asked again stands while its vehicle is no further short of its group than this
/// (ticks along its route: backed up out of one's way, `cars::JAM_BACK`, and a little more,
/// `Traffic::begin`).
const KEPT_ASK: f32 = crate::drive_ticks(3.25);

/// An ask this old (ticks of drive time) yields only to those it waits for itself
/// (`Traffic::begin`, rule 2).
pub(super) const OLD_ASK: u64 = crate::drive_secs(20.0);

/// The claims held and asked for (see the module notes). Not saved.
#[derive(Debug, Clone, Default)]
pub struct Traffic {
    /// The tick (`begin`).
    pub(super) tick: u64,
    pub(super) holds: Vec<Hold>,
    pub(super) asks: Vec<Ask>,
    /// Each lit junction's lights (`Travel::signals`): the arm whose turn it is, how far through
    /// it (`Light`), and since when.
    pub(crate) lights: Vec<Light>,
    pub(super) movers: Vec<Mover>,
    /// `movers` by key: each key and the first of them with it, sorted (`mover`).
    pub(super) mover_ix: Vec<(u64, u32)>,
    /// Where each vehicle on its way to the start of its next claim, not yet there, will wait for
    /// it (its key, the claim's first sample: `may_go`, this tick).
    pub(super) starts: Vec<(u64, Sample)>,
    /// Ticks a car was kept from moving deeper into another by the last-resort check
    /// (`cars::guard`), since the game was loaded.
    pub guarded: u64,
    /// Cars held up out of the picture put on along their ways (`cars::unstick`), since the game
    /// was loaded.
    pub unstuck: u64,
    /// Cars held up in sight turned round to go another way (`cars::unjam`), and rings of them
    /// none of which could, by why, since the game was loaded.
    pub unjams: u64,
    pub unjam_impossible: std::collections::BTreeMap<&'static str, u64>,
    /// Rings `cars::unjam` last tried (by their least key), and when.
    pub(crate) unjam_tried: Vec<(u64, u64)>,
    /// Work taxis held still on their way so long they gave their way up (`taxi/`): in sight
    /// (driving off, `cars::drive_off`) and out of it (gone), since the game was loaded.
    pub give_ups: [u64; 2],
    /// Each car turning round (`cars::unjam`): the ground it still backs up over, its bodies
    /// along it this tick.
    pub(crate) turning: Vec<(u64, Vec<crate::cars::Rect>)>,
    /// The work taxis standing at the kerb in sight (`cars::pass_stands`: their keys, where), and
    /// a count that goes up each time they change.
    pub(crate) stands: Vec<(u64, [f32; 2])>,
    pub(crate) stand_gen: u64,
    /// Each car on its way round a standing taxi (`cars::PassBy`), and the taxi's key: it keeps
    /// clear of the taxi's claim as it goes by.
    pub(crate) passing: Vec<(u64, u64)>,
    /// Per lit junction, the phases anyone waits at and comes through (`change_lights`'s own,
    /// kept so it isn't made anew every tick).
    pub(super) lit_phases: Vec<[u64; 4]>,
    /// Per lit junction, since when its green is held for those through it from a lit junction
    /// just before (`Claim::gate2`, `GREEN_HELD`), and whether one may still be granted through
    /// it (`Traffic::green_through`).
    pub(super) held: Vec<Option<u64>>,
    pub(super) fresh: Vec<bool>,
    /// The lit junctions someone on foot waits at a crosswalk of (`Signal` indices, this tick:
    /// `kerb_stop`, `traffic::begin`), and the phase they wait for (`Signal::wants`: the walk, or
    /// on a side road's crosswalk its main road's green).
    pub(crate) walkers: Vec<Gate>,
    /// The lit junctions someone on foot is on a crosswalk of, and its arm (this tick,
    /// `on_crossings`), in order: their walk phase isn't over before they're across, nor does the
    /// phase of the arm they're on come round.
    pub(crate) crossing: Vec<(u16, u8)>,
    /// Those on foot on a zebra of a junction giving way, or stepping onto one, and those on a
    /// side road's crosswalk at a lit junction, open on its main road's green
    /// (`Signal::opens_with`), or stepping onto it then (`on_crossings`).
    pub(crate) zebra_walkers: Vec<OnZebra>,
    /// The lights cars on the road are coming up to (`Claim::gate`, `cars::movers`: within
    /// `APPROACH` of their stop line, not yet asking): its green kept on for them.
    pub(crate) approaching: Vec<Gate>,
}

/// Someone on foot on a zebra or stepping onto it (`Traffic::zebra_walkers`): at a lit junction
/// (`Travel::signals`, a side road's crosswalk: `Signal::opens_with`) or one giving way
/// (`Travel::give_way`), which, the arm, who, and whether on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OnZebra {
    pub lit: bool,
    pub junction: u16,
    pub arm: u8,
    pub id: u32,
    pub on: bool,
}

/// A car this near its next lights' stop line (ticks along its way: two seconds of drive time,
/// some 18 m at a town's 9 m/s) counts as coming to its green (`Traffic::approaching`).
pub(crate) const APPROACH: f32 = crate::drive_ticks(2.0);

/// Claim `k` of a route tagged `route` (`Place`).
pub(super) fn tag(route: u32, k: usize) -> u32 {
    route << 16 | k as u32
}

impl Traffic {
    /// Whether `key` holds its claim `tag`.
    pub(super) fn holds(&self, key: u64, tag: u32) -> bool {
        self.holds.iter().any(|h| h.key == key && h.tag == tag)
    }

    /// Whether `key` holds a claim.
    pub fn holding(&self, key: u64) -> bool {
        self.holds.iter().any(|h| h.key == key)
    }

    /// Whether `key` waits for a claim to be granted, at the start of it (this tick or the last).
    pub fn waiting(&self, key: u64) -> bool {
        self.asks
            .iter()
            .any(|a| a.key == key && a.seen + 1 >= self.tick)
    }

    /// Whom `key` waits for its claim for (`Ask::by`): those holding claims in its way, or on the
    /// road in it.
    pub fn waits_for(&self, key: u64) -> &[u64] {
        self.asks
            .iter()
            .find(|a| a.key == key && a.seen + 1 >= self.tick)
            .map_or(&[][..], |a| &a.by[..])
    }

    /// `key`, a taxi standing `t` ticks along its way at a stop (`cars::stand_taxi`): every claim
    /// it holds but the one it stands in let go, and its asks too; it asks again as it sets off.
    /// (Standing there for its riders, a claim on ahead held others up all the while.)
    pub(crate) fn stand(&mut self, key: u64, t: f32) {
        self.holds
            .retain(|h| h.key != key || h.claim.from <= t + 1e-3 && t < h.claim.to);
        self.asks.retain(|a| a.key != key);
        self.starts.retain(|s| s.0 != key);
        for a in &mut self.asks {
            a.by.retain(|&k| k != key);
            a.fifo.retain(|&k| k != key);
        }
    }

    /// `key`'s claims worked out again `t` ticks along its route (`route`, `claims`: a way laid
    /// round a taxi standing at the kerb, `cars::pass_stands`): what it held and asked for let go,
    /// as `forget`, but every part of the group it's in the middle of held at once, as it held it.
    /// It can't stand clear between them: let go, the rest of a way through two lit junctions
    /// joined as one was granted to one coming the other way, in its way, and the two stood nose
    /// to nose in the box for good (Willowley).
    pub(crate) fn reclaimed(&mut self, key: u64, route: u32, claims: &[Claim], t: f32) {
        self.forget(key);
        let Some(group) = claims
            .iter()
            .find(|c| c.from <= t + 1e-3 && t < c.to)
            .map(|c| c.group)
        else {
            return;
        };
        for (k, c) in claims.iter().enumerate() {
            if c.group == group && t < c.to {
                self.holds.push(Hold {
                    key,
                    tag: tag(route, k),
                    claim: c.clone(),
                    at: t,
                });
            }
        }
    }

    /// `key` gone out of sight (`cars::sight`): its claims, held or asked for, let go, and nobody
    /// waits for it.
    pub(crate) fn forget(&mut self, key: u64) {
        self.holds.retain(|h| h.key != key);
        self.asks.retain(|a| a.key != key);
        self.starts.retain(|s| s.0 != key);
        for a in &mut self.asks {
            a.by.retain(|&k| k != key);
            a.fifo.retain(|&k| k != key);
        }
    }

    fn mover(&self, key: u64) -> Option<&Mover> {
        let found = self
            .mover_ix
            .binary_search_by_key(&key, |e| e.0)
            .ok()
            .map(|k| &self.movers[self.mover_ix[k].1 as usize]);
        debug_assert!(
            found.map(|m| m as *const Mover)
                == self
                    .movers
                    .iter()
                    .find(|m| m.key == key)
                    .map(|m| m as *const Mover)
        );
        found
    }
}

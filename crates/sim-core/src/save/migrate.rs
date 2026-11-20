//! Bringing older saves up to date.
//!
//! Split from `save.rs` (see `mod.rs`).

use super::*;

/// Ticks as files before v36 counted them: 20 to a real second and 20 to a game minute.
pub(super) const OLD_TICKS_PER_SECOND: u32 = 20;
pub(super) const OLD_TICKS_PER_MINUTE: u64 = 20;

/// How a file's ticks become this build's (v36: before it, `OLD_TICKS_PER_SECOND` and
/// `OLD_TICKS_PER_MINUTE`; v37: before it, cars timed in real seconds). The clock's ticks
/// (`World::tick`, and the ticks things began, were made, happened or are due at) by the game
/// minute's ticks, now over then: tick 0 is 08:00 on day 1 either way, so each is the same minute
/// of the same day. Places along a car's or a taxi's way (ticks of its route, timed by its speeds)
/// by a second of drive time's ticks (`DRIVE_TICKS_PER_SECOND`) over the file's (its real second's
/// before v37: the same place along the same way). The tick a traffic light's stage began
/// (`traffic::Light`, timed in drive time, `drive_secs`) as far into its stage as it was
/// (`drive_since`). A car's state and a taxi standing at its stop began at a tick of the clock:
/// what they wait for is game minutes (a car boarding `cars::BOARD_WAIT_TICKS`, a taxi
/// `taxi.waitMinutes` and `taxi::COMING_WAIT`), and their seconds' floors and dwells
/// (`BOARD_MIN_TICKS`, `DWELL`, `PASS_BY_MOST`) end within the second. Nothing else is saved in
/// ticks: what's left of a nap, a visit, a feeling or a machine's run is saved in game minutes,
/// and the day things happened on in days.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Rescale {
    /// Ticks in a game minute, now and in the file.
    pub(super) game: (u64, u64),
    /// Ticks in a second of drive time now over those in the file (its real second's before v37).
    pub(super) drive: f32,
}

impl Rescale {
    /// For a file of save version `version`.
    pub(super) fn of(version: u32) -> Self {
        // (Ticks to the second cars were timed in, in the file.)
        let file_drive = match version {
            ..36 => OLD_TICKS_PER_SECOND as f32,
            36 => TICKS_PER_SECOND as f32,
            _ => DRIVE_TICKS_PER_SECOND,
        };
        Self {
            game: if version < 36 {
                (TICKS_PER_MINUTE, OLD_TICKS_PER_MINUTE)
            } else {
                (1, 1)
            },
            drive: DRIVE_TICKS_PER_SECOND / file_drive,
        }
    }

    /// A tick of the clock, to the nearest (0, never, stays 0; the same tick stays the same).
    pub(super) fn tick(self, t: u64) -> u64 {
        let (now, then) = self.game;
        if now == then {
            return t;
        }
        let scaled = (u128::from(t) * u128::from(now) + u128::from(then / 2)) / u128::from(then);
        u64::try_from(scaled).unwrap_or(u64::MAX)
    }

    /// A place along a way, in ticks of its route.
    pub(super) fn route(self, t: f32) -> f32 {
        t * self.drive
    }

    /// The tick something timed in drive time began (`since`; in real time before v37), the
    /// file's clock at `now`: as many of its seconds before the clock's tick (`tick`) as it was.
    /// (By the clock's ticks, a traffic light green for 4 seconds was at it for 5.)
    pub(super) fn drive_since(self, since: u64, now: u64) -> u64 {
        if since >= now {
            return self.tick(since);
        }
        let ago = ((now - since) as f64 * f64::from(self.drive)).round() as u64;
        self.tick(now).saturating_sub(ago)
    }

    pub(super) fn detour(self, d: &mut crate::cars::Detour) {
        d.from = self.route(d.from);
        d.back = self.route(d.back);
    }

    /// The households' cars: when their state began and when one away sets off back, where they
    /// are along their way, and their way turned round or round a taxi.
    pub(super) fn cars<'a>(
        self,
        cars: &'a [crate::cars::Car],
    ) -> std::borrow::Cow<'a, [crate::cars::Car]> {
        if self.game.0 == self.game.1 && self.drive == 1.0 {
            return cars.into();
        }
        let mut cars = cars.to_vec();
        for c in &mut cars {
            c.since = self.tick(c.since);
            c.ready_at = self.tick(c.ready_at);
            c.progress = self.route(c.progress);
            if let Some(d) = c.detour.as_mut() {
                self.detour(d);
            }
            if let Some(p) = c.pass_by.as_mut() {
                p.from = self.route(p.from);
                p.out = self.route(p.out);
                p.back = self.route(p.back);
                p.to = self.route(p.to);
            }
        }
        cars.into()
    }

    /// The work taxis' runs: when each comes in, is due out (and was planned to be), leaves work
    /// and stands at its stop since, when each stop is due and its rider leaves home for it, and
    /// where it is along its way.
    pub(super) fn taxis(self, taxis: &mut crate::taxi::Taxis) {
        for r in &mut taxis.runs {
            r.enter = self.tick(r.enter);
            r.due_out = self.tick(r.due_out);
            r.planned_out = self.tick(r.planned_out);
            r.depart = self.tick(r.depart);
            r.since = self.tick(r.since);
            r.progress = self.route(r.progress);
            if let Some(d) = r.detour.as_mut() {
                self.detour(d);
            }
            for st in &mut r.stops {
                st.due = self.tick(st.due);
                st.leave = self.tick(st.leave);
            }
        }
    }
}

/// Kinds whose footprint grew in save version 33: the home chess table, 1 × 1 to 2 × 1 (a chair
/// at each end, M-hobbies-11).
pub(super) const GROWN_IN_33: [&str; 1] = ["chessTable"];

/// Whether a saved object is one of `GROWN_IN_33` from an older file, at its kind's old size (a
/// resized one keeps its own size and loads as any).
pub(super) fn grown_since_32(version: u32, o: &ObjectSave) -> bool {
    version < 33 && o.size.is_none() && GROWN_IN_33.contains(&o.def.as_str())
}

/// Whether saved object `i` is one of the town's posts (content `staff`) from a file older than
/// v33, with nothing standing on it: placed after the rest, it may move out from a wall
/// (`World::nudge_post`).
pub(super) fn post_since_32(content: &Content, save: &SaveFile, i: usize, tops: &[bool]) -> bool {
    let o = &save.objects[i];
    let Some(def) = content.object_index(&o.def) else {
        return false;
    };
    let d = &content.objects[def];
    if save.version >= 33 || d.staff.is_none() {
        return false;
    }
    // (Anything on its top stays where it was saved: so does the post.)
    let (w, dd) = if o.rot.is_multiple_of(2) {
        (d.footprint[0], d.footprint[1])
    } else {
        (d.footprint[1], d.footprint[0])
    };
    let under =
        |t: &ObjectSave| t.x >= o.x && t.x < o.x + w as i32 && t.z >= o.z && t.z < o.z + dd as i32;
    !save
        .objects
        .iter()
        .enumerate()
        .any(|(j, t)| tops[j] && under(t))
}

/// v32 to v33: a thing whose kind grew (`grown_since_32`) goes over the tile it stood on where it
/// now fits and blocks nothing, else at its home's first good free spot (`World::place_grown`);
/// with no room for it anywhere there, it is sold back to the household living on its plot for
/// what it was worth, and left out. Never placed overlapping anything, nor shutting anyone or
/// anything off.
pub(super) fn migrate_32_grown(
    world: &mut World,
    def: usize,
    o: &ObjectSave,
    keep: Option<u32>,
) -> Result<u32, crate::Error> {
    let value = o
        .value
        .unwrap_or(world.content.objects[def].price.unwrap_or(0));
    let colors = crate::world::parse_colors(&o.colors, o.color.as_deref());
    if let Some(id) = world.place_grown(def, o.x, o.z, o.rot, colors, o.quality, value) {
        if let Some(stable) = keep {
            world.objects[id as usize].stable = stable;
        }
        return Ok(id);
    }
    let plot = world.plot_at(o.x, o.z);
    if let Some(h) = world
        .households
        .iter_mut()
        .find(|h| plot.is_some() && h.plot == plot)
    {
        h.funds += value;
    }
    Err(crate::Error::new(format!(
        "no room for the {} any more",
        world.content.objects[def].name.to_lowercase()
    )))
}

/// Events from before content ids were saved name content by index. The one change since: the
/// base game's crib (0.13, save 14) came at the end of the base objects, so objects from packs
/// (after it) moved up one.
pub(super) fn migrate_refs(content: &Content, e: &mut social::SocialEvent, version: u32) {
    use social::EventKind::*;
    if version < 14
        && matches!(e.kind, Broke | Repaired)
        && let (Some(crib), Some(n)) = (content.object_index("crib"), e.n)
        && n >= crib as i64
    {
        e.n = Some(n + 1);
    }
}

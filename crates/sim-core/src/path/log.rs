//! The log of navigation changes, for clients that patch their own copy.

/// The last changes to walking (`World::nav_changed_in`), with the navigation versions they made
/// and the tiles `[x0, z0, x1, z1)` they touched: a walker whose way doesn't go near any change
/// since it last looked needn't find its way again. The changes of one tick share a place (cars
/// coming and going, a build: several patches each), so the log goes back `NAV_LOG` ticks with
/// changes however many there were in each.
#[derive(Debug, Clone, Default)]
pub struct NavLog {
    entries: Vec<NavChanges>,
    next: usize,
}

/// Places `NavLog` keeps: at least as many ticks as a resident far from the view waits between
/// turns (`lod::FAR_TICKS`), so one that keeps walking while cars come and go every tick still
/// finds what changed since its last turn; rounded up to a power of two (64 for three game minutes
/// of 20 ticks).
pub const NAV_LOG: usize = (crate::lod::FAR_TICKS as usize).next_power_of_two();
const _: () = assert!(NAV_LOG as u64 >= crate::lod::FAR_TICKS);

/// The most separate patches one place in `NavLog` keeps (more: the two earlier ones whose box
/// round them grows least share one).
const NAV_LOG_PATCHES: usize = 16;

/// The changes to walking made in one tick (`group`): navigation versions `first..=last`, over
/// tiles `rects[..len]`, each with the last version that touched them.
#[derive(Debug, Clone, Copy)]
struct NavChanges {
    group: u64,
    first: u32,
    last: u32,
    rects: [([i32; 4], u32); NAV_LOG_PATCHES],
    len: usize,
}

impl NavChanges {
    /// Adds the change that made `version`, over tiles `r`. (With no room, two earlier patches
    /// make room, so the new one is kept as it is and a walker that looked between them isn't
    /// told of a box round older ones.)
    pub(super) fn add(&mut self, r: [i32; 4], version: u32) {
        if self.len < NAV_LOG_PATCHES {
            self.rects[self.len] = (r, version);
            self.len += 1;
            return;
        }
        let area = |a: [i32; 4]| i64::from(a[2] - a[0]) * i64::from(a[3] - a[1]);
        let cost = |a: [i32; 4], b: [i32; 4]| {
            let both = area(join(a, b));
            (both - area(a) - area(b), both)
        };
        let rects = &mut self.rects;
        let (mut bi, mut bj) = (0, 1);
        let mut best = cost(rects[0].0, rects[1].0);
        for i in 0..NAV_LOG_PATCHES {
            for j in i + 1..NAV_LOG_PATCHES {
                let c = cost(rects[i].0, rects[j].0);
                if c < best {
                    (bi, bj, best) = (i, j, c);
                }
            }
        }
        let ((a, va), (b, vb)) = (rects[bi], rects[bj]);
        // (The later of the two: the closer to `version`.)
        let later = if version.wrapping_sub(va) < version.wrapping_sub(vb) {
            va
        } else {
            vb
        };
        rects[bi] = (join(a, b), later);
        rects[bj] = (r, version);
    }
}

impl NavLog {
    /// Notes the change that made navigation version `version`, over tiles `rect`, in tick
    /// `group`: with the tick's other changes if it follows on from them.
    pub fn push(&mut self, group: u64, version: u32, rect: [i32; 4]) {
        if !self.entries.is_empty() {
            let newest = &mut self.entries[(self.next + NAV_LOG - 1) % NAV_LOG];
            if newest.group == group && newest.last.wrapping_add(1) == version {
                newest.last = version;
                newest.add(rect, version);
                return;
            }
        }
        let mut place = NavChanges {
            group,
            first: version,
            last: version,
            rects: [([0; 4], 0); NAV_LOG_PATCHES],
            len: 0,
        };
        place.add(rect, version);
        if self.entries.len() < NAV_LOG {
            self.entries.push(place);
        } else {
            self.entries[self.next] = place;
        }
        self.next = (self.next + 1) % NAV_LOG;
    }

    /// Whether a change after version `seen`, up to `now`, may touch tiles `[x0, z0, x1, z1)`:
    /// one does, or the log doesn't go back that far (then any might).
    pub fn changed_since(&self, seen: u32, now: u32, rect: [i32; 4]) -> bool {
        self.changes_since(seen, now)
            .is_none_or(|mut changes| changes.any(|r| overlaps(r, rect)))
    }

    /// The tiles the changes after version `seen`, up to `now`, touched (and maybe a few more,
    /// where a tick's patches were joined); None if the log doesn't go back that far (any tile
    /// may have changed).
    pub fn changes_since(&self, seen: u32, now: u32) -> Option<impl Iterator<Item = [i32; 4]>> {
        let since = now.wrapping_sub(seen);
        // (How many of the versions after `seen` each place made.)
        let after = move |e: &NavChanges| {
            let newest = now.wrapping_sub(e.last);
            if newest >= since {
                return 0;
            }
            now.wrapping_sub(e.first).min(since - 1) - newest + 1
        };
        let found: u64 = self.entries.iter().map(|e| u64::from(after(e))).sum();
        (found >= u64::from(since)).then(|| {
            self.entries
                .iter()
                // (A place whose last change is older skips its patches at once.)
                .filter(move |e| now.wrapping_sub(e.last) < since)
                .flat_map(|e| e.rects[..e.len].iter())
                .filter(move |r| now.wrapping_sub(r.1) < since)
                .map(|r| r.0)
        })
    }
}

/// The box round two tile rectangles `[x0, z0, x1, z1)`.
pub fn join(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    [
        a[0].min(b[0]),
        a[1].min(b[1]),
        a[2].max(b[2]),
        a[3].max(b[3]),
    ]
}

/// Whether two tile rectangles `[x0, z0, x1, z1)` share a tile.
pub fn overlaps(a: [i32; 4], b: [i32; 4]) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

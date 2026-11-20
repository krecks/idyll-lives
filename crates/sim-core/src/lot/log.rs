//! The log of what changed, so clients can patch and undo.

use super::*;

impl Lot {
    /// Notes an edge whose wall-ness changed (or a diagonal): the rooms round it are found again.
    pub(super) fn note(&mut self, e: EdgeRef) {
        if let Some(p) = &mut self.pending {
            if p.len() >= PENDING_MAX {
                self.pending = None;
            } else {
                p.push(e);
            }
        }
    }

    /// Something changed: the tiles round it may score differently (`RoomChanges::rect`), and
    /// while the log is on it keeps what was there.
    pub(super) fn changed(&mut self, change: LotChange) {
        if !self.changes.full {
            let edge = |e: EdgeRef| match e {
                EdgeRef::H(x, z) => [x as i32, z as i32 - 1, x as i32 + 1, z as i32 + 1],
                EdgeRef::V(x, z) => [x as i32 - 1, z as i32, x as i32 + 1, z as i32 + 1],
                EdgeRef::Diag(x, z) => [x as i32, z as i32, x as i32 + 1, z as i32 + 1],
            };
            let at = match change {
                LotChange::H(x, z, _) => edge(EdgeRef::H(x, z)),
                LotChange::V(x, z, _) => edge(EdgeRef::V(x, z)),
                LotChange::Diag(x, z, _) | LotChange::Floor(x, z, _) => edge(EdgeRef::Diag(x, z)),
                LotChange::Look(e, _) => edge(e),
            };
            self.changes.rect = Some(self.changes.rect.map_or(at, |r| join(r, at)));
        }
        if let Some(log) = &mut self.log {
            log.push(change);
        }
    }

    /// Starts keeping what's changed on the lot, as it was before (the log may be on already:
    /// an edit within an edit). `log_end` gives what changed since, `log_rollback` puts it back.
    pub fn log_begin(&mut self) -> LogMark {
        match &self.log {
            Some(log) => LogMark {
                at: log.len(),
                started: false,
            },
            None => {
                self.log = Some(Vec::new());
                LogMark {
                    at: 0,
                    started: true,
                }
            }
        }
    }

    /// What changed since `mark`, as it was before, oldest first; the log goes off if `mark`
    /// turned it on.
    pub fn log_end(&mut self, mark: LogMark) -> Vec<LotChange> {
        if mark.started {
            return self.log.take().unwrap_or_default();
        }
        self.log
            .as_ref()
            .and_then(|log| log.get(mark.at..))
            .map(<[LotChange]>::to_vec)
            .unwrap_or_default()
    }

    /// Puts back what changed since `mark` (call `compute_rooms` afterwards), and says whether
    /// anything had; the log goes off if `mark` turned it on.
    pub fn log_rollback(&mut self, mark: LogMark) -> bool {
        let changes = match &mut self.log {
            Some(log) if mark.at <= log.len() => log.split_off(mark.at),
            _ => Vec::new(),
        };
        let log = self.log.take();
        self.put_back(&changes);
        self.log = if mark.started { None } else { log };
        !changes.is_empty()
    }

    /// The edit begun at `mark` stands and nobody keeps it: the log goes off if `mark` turned
    /// it on.
    pub fn log_forget(&mut self, mark: LogMark) {
        if mark.started {
            self.log = None;
        }
    }

    /// Stops the log (time moves on: edits can no longer be taken back).
    pub fn log_off(&mut self) {
        self.log = None;
    }

    /// Puts back changes as `log_end` gave them, newest first (call `compute_rooms` afterwards).
    /// With the log on, it keeps what they replace: the changes that make them again.
    pub fn put_back(&mut self, changes: &[LotChange]) {
        for c in changes.iter().rev() {
            match *c {
                LotChange::H(x, z, e) => self.set_h(x.into(), z.into(), e),
                LotChange::V(x, z, e) => self.set_v(x.into(), z.into(), e),
                LotChange::Diag(x, z, d) => self.set_diag(x.into(), z.into(), d),
                LotChange::Look(at, look) => self.set_look(at, look),
                LotChange::Floor(x, z, covering) => self.set_floor(x, z, covering),
            }
        }
    }

    /// How the rooms changed since this was last asked (see `RoomChanges`).
    pub fn take_room_changes(&mut self) -> RoomChanges {
        std::mem::take(&mut self.changes)
    }
}

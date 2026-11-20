//! Low-frequency JSON views for the UI and for world structure.
//! Never used in the per-frame render path.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::hash::{DefaultHasher, Hasher};

use serde::Serialize;

use crate::clock;
use crate::content::Content;
use crate::life;
use crate::lot::Edge;
use crate::planner::{Goal, GoalKind, Outcome, Reason, Routine};
use crate::social::SocialEvent;
use crate::world::{Phase, Sim, Task, TaskKind, World};
use crate::{MINUTES_PER_TICK, conversation};

/// A room of the player's home (see `rooms.rs`). `scores`: size, light, decor, cleanliness,
/// function, overall (0..1).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RoomView {
    /// Lot room id (0 for the garden); changes when walls do.
    pub id: u16,
    pub garden: bool,
    /// Room kind (index into `roomKinds`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<usize>,
    pub mixed: bool,
    /// The kind's essential nothing offers (index into its `essentials`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing: Option<usize>,
    pub tiles: f32,
    pub windows: u16,
    pub doors: u16,
    pub lamps: u16,
    pub dirt: f32,
    pub centre: [f32; 2],
    pub scores: [f32; 6],
}

/// A story event plus how much it matters (0 everyday, 1 notable, 2 a milestone).
#[derive(Serialize)]
pub(crate) struct EventView<'a> {
    #[serde(flatten)]
    pub event: &'a SocialEvent,
    pub importance: u8,
}

impl<'a> From<&'a SocialEvent> for EventView<'a> {
    fn from(event: &'a SocialEvent) -> Self {
        Self {
            event,
            importance: event.kind.importance(),
        }
    }
}

/// Needs, mood, progress and scores go in 1 % steps (a bar's resolution).
const PERCENT: f64 = 100.0;

/// `v` rounded to steps of `1 / per_unit`: values that drift every tick are rounded to what the
/// UI shows, so they only count as changed (and are sent again) when the change is visible.
fn round_to(v: f32, per_unit: f64) -> f32 {
    ((f64::from(v) * per_unit).round() / per_unit) as f32
}

/// `round_to`, rounding down (skill levels: the whole number is the level).
fn floor_to(v: f32, per_unit: f64) -> f32 {
    ((f64::from(v) * per_unit).floor() / per_unit) as f32
}

/// `round_to`, rounding away from zero (a trend then passes a threshold just when it does).
fn away_from_zero(v: f32, per_unit: f64) -> f32 {
    let steps = (f64::from(v) * per_unit).abs().ceil();
    (steps.copysign(f64::from(v)) / per_unit) as f32
}

/// A fingerprint of what was sent, never 0 (0: nothing sent yet).
fn fingerprint(bytes: &[u8]) -> u64 {
    let mut h = DefaultHasher::new();
    h.write(bytes);
    h.finish() | 1
}

/// A JSON object written field by field into a buffer.
pub(crate) struct Obj<'a> {
    pub out: &'a mut Vec<u8>,
    pub fields: usize,
}

impl<'a> Obj<'a> {
    fn open(out: &'a mut Vec<u8>) -> Self {
        out.push(b'{');
        Obj { out, fields: 0 }
    }

    fn key(&mut self, key: &str) {
        if self.fields > 0 {
            self.out.push(b',');
        }
        self.fields += 1;
        self.out.push(b'"');
        self.out.extend_from_slice(key.as_bytes());
        self.out.extend_from_slice(b"\":");
    }

    fn field<T: Serialize + ?Sized>(&mut self, key: &str, value: &T) {
        self.key(key);
        serde_json::to_writer(&mut *self.out, value).expect("UI view serializes");
    }

    /// Writes the field only if its JSON differs from what `sent` remembers (and remembers it).
    fn changed<T: Serialize + ?Sized>(&mut self, key: &str, value: &T, sent: &mut u64) {
        let (mark, fields) = (self.out.len(), self.fields);
        self.key(key);
        let start = self.out.len();
        serde_json::to_writer(&mut *self.out, value).expect("UI view serializes");
        let print = fingerprint(&self.out[start..]);
        if print == *sent {
            self.out.truncate(mark);
            self.fields = fields;
        } else {
            *sent = print;
        }
    }

    /// `changed` for a field the UI leaves out when absent: becoming absent is sent as `null`.
    fn changed_opt<T: Serialize>(&mut self, key: &str, value: Option<&T>, sent: &mut u64) {
        match value {
            Some(v) => self.changed(key, v, sent),
            None if *sent != 0 => {
                self.field(key, &());
                *sent = 0;
            }
            None => {}
        }
    }

    /// Writes `list` as an array (empty lists only when `always`).
    fn list(&mut self, key: &str, list: &List, always: bool) {
        if always || list.len > 0 {
            self.key(key);
            self.out.push(b'[');
            self.out.extend_from_slice(&list.out);
            self.out.push(b']');
        }
    }

    /// Closes the object; returns how many fields it has.
    fn close(self) -> usize {
        self.out.push(b'}');
        self.fields
    }
}

/// The items of a JSON array, written one by one.
#[derive(Default)]
pub(crate) struct List {
    pub out: Vec<u8>,
    pub len: usize,
}

mod catalog;
mod plan;
mod social;
mod town;
mod ui;

pub use catalog::*;
pub(crate) use plan::*;
pub use social::*;
pub use town::*;
pub use ui::*;

//! Platform-independent simulation core.
//!
//! Rules for this crate:
//! - No rendering, browser, or asset-file knowledge. Visual data (models, icons,
//!   colours) is passed through as opaque strings at most.
//! - Deterministic: the same content, lot and seed always produce the same run.
//! - No allocation in the steady-state tick path where avoidable.

pub mod ai;
pub mod blueprint;
pub mod carry;
pub mod cars;
pub mod clock;
pub mod command;
pub mod content;
pub mod conversation;
pub mod cooking;
pub mod error;
pub mod garages;
pub mod home;
pub mod hosting;
mod index;
pub mod items;
pub mod life;
pub mod lifecycle;
pub mod lights;
pub mod lod;
pub mod lot;
pub mod machines;
pub mod mesh;
pub mod pack;
pub mod partners;
pub mod path;
pub mod phone;
pub mod planner;
pub mod privacy;
pub(crate) mod ride;
pub mod rng;
pub mod rooms;
pub mod save;
pub mod setup;
pub mod snapshot;
pub mod social;
pub mod staff;
pub mod storeys;
pub mod surface;
pub mod taxi;
pub mod traffic;
pub mod travel;
pub mod view;
pub mod world;

pub use command::Command;
pub use content::Content;
pub use error::Error;
pub use pack::merge_content;
pub use world::World;

// Two units of time, kept apart:
// - real time, `TICKS_PER_SECOND`: what is seen to move at its own pace whatever the clock does,
//   such as walking and driving speeds, a car's route position, doors, traffic lights and jams;
// - game time, `TICKS_PER_MINUTE`: the clock, needs, schedules and every duration a resident lives
//   through.
// `secs` and `mins` turn either into ticks; never count ticks with a bare number.

/// Fixed simulation rate in ticks per real second (at normal speed).
pub const TICKS_PER_SECOND: u32 = 20;
/// Ticks in a game minute. The clock counts these whole ticks (see `clock.rs`); `MINUTES_PER_TICK`
/// is only for durations.
pub const TICKS_PER_MINUTE: u64 = 20;
pub const MINUTES_PER_TICK: f32 = 1.0 / TICKS_PER_MINUTE as f32;
/// Game minutes that pass in a real second at normal speed.
pub const GAME_MINUTES_PER_SECOND: f32 = TICKS_PER_SECOND as f32 / TICKS_PER_MINUTE as f32;

/// Ticks in `seconds` of real time (at normal speed), rounded, at least 0.
pub const fn secs(seconds: f32) -> u64 {
    let t = seconds * TICKS_PER_SECOND as f32;
    if t <= 0.0 { 0 } else { (t + 0.5) as u64 }
}

/// Ticks in `minutes` of game time, rounded, at least 0.
pub const fn mins(minutes: f32) -> u64 {
    let t = minutes * TICKS_PER_MINUTE as f32;
    if t <= 0.0 { 0 } else { (t + 0.5) as u64 }
}

// A quarter of a second is a whole number of ticks (the lights).
const _: () = assert!(TICKS_PER_SECOND % 4 == 0);

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

// Three units of time, kept apart:
// - real time, `TICKS_PER_SECOND`: what is seen to move at its own pace whatever the clock does,
//   such as walking, doors and getting in and out of a car;
// - drive time, `DRIVE_TICKS_PER_SECOND`: the traffic's own clock, `DRIVE_SCALE` times as fast as
//   real time: vehicles' speeds (cars, the taxi, bikes and e-scooters), a car's route position,
//   its look-aheads, claims and jams, and the traffic lights, all in step with each other;
// - game time, `TICKS_PER_MINUTE`: the clock, needs, schedules and every duration a resident lives
//   through.
// `secs`, `drive_secs` and `mins` turn them into ticks; never count ticks with a bare number.

/// Fixed simulation rate in ticks per real second (at normal speed; 20 before 0.7.0).
pub const TICKS_PER_SECOND: u32 = 60;
/// Ticks in a game minute (20 before 0.7.0): a game minute is a second and a quarter at normal
/// speed, a game day 30 real minutes. The clock counts these whole ticks (see `clock.rs`); `MINUTES_PER_TICK`
/// is only for durations.
pub const TICKS_PER_MINUTE: u64 = 75;
pub const MINUTES_PER_TICK: f32 = 1.0 / TICKS_PER_MINUTE as f32;
/// Game minutes that pass in a real second at normal speed.
pub const GAME_MINUTES_PER_SECOND: f32 = TICKS_PER_SECOND as f32 / TICKS_PER_MINUTE as f32;

/// Ticks in `seconds` of real time (at normal speed), rounded, at least 0.
pub const fn secs(seconds: f32) -> u64 {
    let t = seconds * TICKS_PER_SECOND as f32;
    if t <= 0.0 { 0 } else { (t + 0.5) as u64 }
}

/// How many times as fast as real time the traffic's clock runs (drive time): a vehicle's speed of
/// so many m a second is so many m a second of drive time, `DRIVE_SCALE` times that a real second,
/// and everything that times how vehicles move and take turns (a route's timing, look-aheads,
/// claims, jams, the lights) goes by it, so they keep in step. At real speeds against the 48× game
/// clock a drive across a big world took two game hours; at 4 it takes about half an hour.
pub const DRIVE_SCALE: f32 = 4.0;
/// Ticks in a second of drive time (`DRIVE_SCALE`): a car's route is timed in these, each tick of
/// it a tick of the clock.
pub const DRIVE_TICKS_PER_SECOND: f32 = TICKS_PER_SECOND as f32 / DRIVE_SCALE;

/// Ticks in `seconds` of drive time (`DRIVE_SCALE`), rounded, at least 0.
pub const fn drive_secs(seconds: f32) -> u64 {
    secs(seconds / DRIVE_SCALE)
}

/// Ticks (not rounded: a place along a car's route, a look along it) in `seconds` of drive time.
pub const fn drive_ticks(seconds: f32) -> f32 {
    seconds * DRIVE_TICKS_PER_SECOND
}

/// Ticks in `minutes` of game time, rounded, at least 0.
pub const fn mins(minutes: f32) -> u64 {
    let t = minutes * TICKS_PER_MINUTE as f32;
    if t <= 0.0 { 0 } else { (t + 0.5) as u64 }
}

// A quarter of a second is a whole number of ticks (the lights).
const _: () = assert!(TICKS_PER_SECOND.is_multiple_of(4));

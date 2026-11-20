//! What a resident chooses to do on their own, and how a task is started.
//!
//! Split from `world.rs` (see `mod.rs`).

use super::*;
mod helpers;
mod pick;
mod start;

pub(crate) use helpers::*;
pub(crate) use pick::*;
pub(crate) use start::*;

//! Social life: relationships, social interactions, feelings and emotions.
//!
//! Everything here is data-driven from the content file. Relationships are
//! directional (A can have a crush on B without B feeling the same), and every
//! social interaction is a short conversation whose success depends on how the
//! *target* feels about the actor, their chemistry, mood and traits.
//!
//! By kind: `defs` (the definitions and their raw JSON shapes), `parse` (the raw JSON into
//! definitions), `relationships` (the relationship matrix, family links and active feelings) and
//! `events` (the event log and the milestone events a change implies).

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::Error;
use crate::content::{MAX_NEEDS, Modifiers, ModifiersRaw, StageMask, TagMask};

mod defs;
mod events;
mod parse;
mod relationships;

pub use defs::*;
pub use events::*;
pub(crate) use parse::*;
pub use relationships::*;

#[cfg(test)]
mod tests;

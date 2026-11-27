//! `cargo review sizes|uses [--md <dir>]`: the authenticity review's first pass over every buyable object.
//!
//!   cargo review sizes [--md <dir>]   each object's W x H x D and its seat, top and mattress heights against
//!                                     docs/review/reference.json: the outliers per category (`--md`: every
//!                                     object, `<dir>/sizes-<cat>.md`); a plant against its kind's `species`
//!                                     (`species.rs`); then the textures' patterns and the kerb against its
//!                                     `surfaces` (`surfaces.rs`)
//!         [--check]                   fails on a row with more outside its range than tools/art/gate-baseline.json
//!                                     holds for it (`reference`, by id), or fewer (stale)
//!         [--update-baseline]         first lowers that section to what it finds (only lowers, takes ids out)
//!         [--seed-baseline]           writes the section where the file has none (the day the check came in)
//!   cargo review colours [--all]      every model's colours against what real surfaces reflect (`colours.rs`)
//!   cargo review facets [--all]       every model's round parts whose outline shows its flat sides (`facets.rs`)
//!   cargo review edges [--all] [keys] every model's hard edges that show unbevelled at the play camera (`edges.rs`)
//!   cargo review texels [--all] [keys]
//!                                     every model's textured finishes laid too sparse, dense, stretched or smeared
//!                                     (`texels.rs`)
//!   cargo review uses [--md <dir>]    each object's uses, poses, minutes, effects, skills, slots, approach,
//!                                     motions and sounds, and what looks wrong there: a summary per category
//!                                     (`--md`: every object, `<dir>/uses-<cat>.md`)
//!         [--check]                   fails on a hard flag (an anim not in `ACTIONS`, a stray `motions` key, an
//!                                     open without `motions.open`), and on an object with more of a counted flag
//!                                     than tools/art/gate-baseline.json holds for it (`USES_CHECKS`, by id: a `how`
//!                                     not in `MANNERS`, a clip with no sound, an `overWindow` with no `night` clip
//!                                     or `shut` sound, a seat with the desk's hands and no `surface`), or fewer
//!         [--update-baseline] [--seed-baseline]
//!                                     as `sizes`'s, for those sections
//!
//! The content is read as the game reads it (sim-core merges base.json and its includes, so `default_anim` and
//! the defaults apply), and raw for what sim-core keeps no copy of (`model`, `category`, `group`, a use's `when`,
//! `leaves` and its tags in order). Sizes and heights are `cargo check-glb`'s (`checkglb::measures`), with what
//! the review measures besides: a top its kind has and its content hasn't, a table's H without its seats and a
//! stair's top step (`h_of`). The manifests give each model's
//! `motions`; the renderer's `ACTIONS`, `MANNERS` and `RIDES` (render/shared/characters/actions.ts) and action sounds (audio/world.ts) are read from its
//! source. A category is the object's buy category, or `packs` for a personality
//! pack's objects.

pub mod surfaces;

use crate::checkglb::baseline::{Baseline, Counts, Verdict};
use crate::checkglb::{self, Height, Measure, Paths};
use crate::species::{self, Atlas, Shape, Species};
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// The reference table, from the checkout's root.
pub const REFERENCE: &str = "docs/review/reference.json";
mod baseline;
mod flags;
mod main;
mod motions;
mod objects;
mod plant;
mod plants;
mod reference;
mod report;
mod sizes;
mod tables;
mod uses;

pub use baseline::*;
pub use flags::*;
pub use main::*;
pub use motions::*;
pub use objects::*;
pub use plant::*;
use plants::*;
pub use reference::*;
pub use report::*;
pub use sizes::*;
pub use tables::*;
pub use uses::*;

#[cfg(test)]
mod tests;

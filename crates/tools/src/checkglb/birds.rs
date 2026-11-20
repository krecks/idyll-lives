//! Birds: their parts, their size and where they may perch.
//!
//! Split from `checkglb.rs` (see `mod.rs`).

pub(crate) const SWEEP: f64 = 0.01;
/// How far (m) a model's middle at rest may be off its `centre` (centres.json).
pub(crate) const OFF_CENTRE: f64 = 0.02;
/// A bird's `bird` (crates/art/things/src/birds; the renderer flies it by code): its length bill to tail
/// tip and wingspan (m, both needed), its body's middle over its feet (`perchHeight`, m) and how far its perched
/// torso is raised from level (`pitch`, degrees).
pub(crate) const BIRD_KEYS: [&str; 4] = ["length", "wingspan", "perchHeight", "pitch"];
/// A bird's rigid parts, the nodes the renderer moves by name (a mesh's own node or one above it).
pub(crate) const BIRD_PARTS: [&str; 5] = ["body", "head", "tail", "wing.L", "wing.R"];
/// How far (a share) a bird's file may be off the length and wingspan its `bird` says.
pub(crate) const BIRD_FIT: f64 = 0.05;
/// How far (m) a perch (`perches`: where the game's birds sit on a tree or a hedge) may be outside the model's box.
pub(crate) const PERCH_OUT: f64 = 0.01;
/// What a model with `perches` is, its entry's `habitat` (crates/art/nature/src/trees and plants): a broadleaved or fruit tree,
/// a conifer, a bush or shrub, or a hedge.
pub(crate) const HABITATS: [&str; 4] = ["tree", "conifer", "bush", "hedge"];

//! What an edge is measured against, and the vector it is measured with.

/// Positions nearer than this (m) are one place.
pub(crate) const WELD: f64 = 1e-4;
/// The cosine of the least turn between two faces that makes their edge hard (60°).
pub(crate) const TURN: f64 = 0.5;
/// The cosine of the least angle between two corners' normals that splits them (40°, artkit's `shapes::SHARP`).
pub(crate) const SPLIT: f64 = 0.766_044_443_118_978;
/// The cosine of the most two faces turn and are still one flat surface (2°).
pub(crate) const FLAT: f64 = 0.999_390_827_019_095_8;
/// The cosine of the most a run of edges turns from one to the next (30°: a round thing of 12 sides or more).
pub(crate) const RUN: f64 = 0.866_025_403_784_438_6;
/// An edge shorter than this (m) hardly shows: 12 pixels at the play camera.
pub const LENGTH: f64 = 0.05;
/// A surface narrower than this (m) across an edge is too thin for a bevel that shows (three of 4 mm).
pub const WIDTH: f64 = 0.012;
/// A piece smaller than this (m) is a detail: a knob, a button, a bottle's cap.
pub const PIECE: f64 = 0.10;
/// An edge whose faces look down more than this together (the y of their normals' bisector) is an underside.
pub(crate) const DOWN: f64 = -0.3;
/// An edge wholly under this height (m) lies on the floor, which a house draws 1 cm up.
pub(crate) const FLOOR: f64 = 0.012;
/// How far (m) an edge is pushed out, and its faces looked at in from it, to see whether it is covered.
pub(crate) const OUT: f64 = 0.003;
/// What a measure may be short of its limit by (m, and of a cosine): a size built as 5 cm reads a hair under it.
pub(crate) const SLACK: f64 = 1e-5;

pub type V3 = [f64; 3];

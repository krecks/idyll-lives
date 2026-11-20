//! The ease of a key, and the overshoot it may take.
//!
//! Split from `motion.rs` (see `mod.rs`).

/// How a clip eases from one key to the next.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Ease {
    /// Blender's default: slow out of each key and into the next.
    #[default]
    Smooth,
    /// An even speed (spins, scrolls, loops).
    Linear,
    /// Quick off, slowing into the end.
    Out,
    In,
    InOut,
    /// A snap off, a long glide in (a drawer let run).
    ExpoOut,
    /// Past the end and back: a door swung open, settling.
    BackOut,
    /// Dropping onto the end and bouncing (a lid let fall).
    BounceOut,
}

/// How far past its end `BackOut` goes, as a share of the move (4°: a door opened 100°).
pub const OVERSHOOT: f64 = 0.04;

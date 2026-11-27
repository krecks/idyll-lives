//! OU2: the sun lounger, the hammock, the porch swing, the swing set and the cedar hot tub; each moves while it
//! is used.

mod hammock;
mod hot_tub;
mod porch_swing;
mod sun_lounger;
mod swing_set;

pub use hammock::hammock;
pub use hot_tub::hot_tub_cedar;
#[cfg(test)]
pub(super) use porch_swing::PORCH_SEAT;
pub use porch_swing::porch_swing;
pub use sun_lounger::sun_lounger;
pub use swing_set::swing_set;

//! What counts as fruit, and how far off a measure may be.

use crate::checkglb::against::{
    FRUIT, NORMAL_AWAY, NORMAL_REACH, NORMAL_TURN, SAME_PLACE, SAMPLES, SHADE_LEVELS, UV_OFF,
};
use crate::checkglb::near::{self, Across, Part, Surface, Tolerance};

/// Whether a material is fruit a resident reaches for (`FRUIT`).
pub(crate) fn fruit(material: &str) -> bool {
    FRUIT.iter().any(|f| material.starts_with(f))
}

pub(super) fn tolerance() -> Tolerance {
    Tolerance {
        per_m2: SAMPLES.0,
        few: SAMPLES.1,
        many: SAMPLES.2,
        reach: NORMAL_REACH,
        same: SAME_PLACE,
        turned: NORMAL_TURN,
        away: NORMAL_AWAY,
        uv_off: UV_OFF,
    }
}

/// Two builds of a part measured against each other, both ways: (old against new, new against old).
pub(super) fn both_ways(a: &Part, b: &Part) -> (Across, Across) {
    let (sa, sb) = (Surface::new(a), Surface::new(b));
    let (area, uv) = (a.area(), a.uv_area());
    let uv_metres = if uv > 0.0 { (area / uv).sqrt() } else { 0.0 };
    let tol = tolerance();
    (
        near::across(a, &sb, &tol, uv_metres, SHADE_LEVELS),
        near::across(b, &sa, &tol, uv_metres, SHADE_LEVELS),
    )
}

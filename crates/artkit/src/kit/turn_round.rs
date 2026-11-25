//! A model built in place, turned round to face the other way.

use crate::math::{Axis, V3};
use crate::parts::turn;
use crate::scene::Obj;

/// Turns a model built in place half a turn about the footprint's middle (each part about its own origin, which
/// moves round with it), then moves it `forward` m to the front. A part's clips then open the other way.
pub fn turn_round(objs: &[Obj], forward: f64) {
    for &o in objs {
        turn(o, 180.0, Axis::Z);
        o.edit(|p| {
            p.loc = V3::new(-p.loc.x, (-(p.loc.y as f64) - forward) as f32, p.loc.z);
        });
    }
}

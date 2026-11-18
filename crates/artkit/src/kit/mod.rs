//! The kit: furniture parts more than one family makes, built once here so a better one improves every model that
//! uses it. Each takes a `GameFrame` (`frame`, game axes: x, height, depth) and the family's material, and adds
//! its mesh. Called as the family scripts called them, the options by name:
//!
//!   kit::tube!(F, "rail", &[(0.0, 0.2, 0.0), (0.0, 0.8, 0.0), (0.3, 0.9, 0.0)], 0.012, M.chrome);
//!   kit::cabriole!(F, "leg_fl", 0.31, 0.3, 0.18, M.walnut, 0.02, 0.02);
//!   kit::pull!(F, "pull", 0, 0.45, 0.02, 0.12, M.brass);          // a bar pull on a front at depth 0.02
//!   kit::shaker!(F, "door", 0.6, 1.8, (0, 1, 0.15), 0.02, M.white);  // a frame-and-panel door
//!   kit::songbird!(F, "robin", 0.3, 1.47, -0.1, M.brown, M.red, M.beak, M.eye, turn = -20);   // perched, feet at the point
//!   kit::moulding!(F, "frame", &[(0.25, 1.0), (0.25, 1.7), (-0.25, 1.7), (-0.25, 1.0)], &[(0.0, 0.0), (0.0, 0.03), (0.03, 0.03), (0.03, 0.0)], M.gilt, at = -0.43);
//!   kit::scroll!(F, "curl", (0, 0.66, 0.2), (0.62, 0.2), M.iron, turns = -1.1);  // a scroll in the depth-height plane
//!   kit::bench_end!(F, "end", 0.8, 0.415, 0.19, -0.21, (0.885, -0.27), M.iron, arm = 0.68);  // a cast-iron bench end
//!   kit::cut!(carcass, fbox!(F, "oven_cut", 0.62, 0.42, 0.36, (-0.4, 0.42, 0.06), M.body));   // a recess cut in through a side
//!   kit::recessed_box!(F, "carcass", 0.6, 0.82, 0.6, (0, 0.46, 0), M.enamel, (0.48, 0.38, 0.46), (0, 0.43, 0.08), lining = M.dark);
//!
//! By kind: `hardware` (pulls, knobs), `turned` (legs), `joinery` (bent paths, frame-and-panel fronts, mouldings,
//! a bench's ends, a recess or a hole cut into a body), `metalwork` (tube, scrolls), `creatures` (a songbird),
//! `signs` (the taxi's word and chequers), `tableware`.

pub mod creatures;
pub mod hardware;
pub mod joinery;
mod macros;
pub mod metalwork;
pub mod signs;
pub mod tableware;
pub mod turned;

pub use creatures::*;
pub use hardware::*;
pub use joinery::*;
pub use metalwork::*;
pub use signs::*;
pub use turned::*;

// (the macros, exported at the crate's root under `kit_` names, by the names the scripts called: `kit::pull!`)
pub use crate::{
    kit_bench_end as bench_end, kit_cabriole as cabriole, kit_chequer as chequer, kit_cut as cut,
    kit_fillet as fillet, kit_knob as knob, kit_moulding as moulding, kit_pull as pull,
    kit_recessed_box as recessed_box, kit_scroll as scroll, kit_shaker as shaker,
    kit_songbird as songbird, kit_taxi_word as taxi_word, kit_tube as tube,
};

/// Python's `min` of floats: the first of the least.
pub(crate) fn least(xs: &[f64]) -> f64 {
    let mut m = xs[0];
    for &x in &xs[1..] {
        if x < m {
            m = x;
        }
    }
    m
}

/// Python's `max` of two floats: the first unless the second is greater.
pub(crate) fn most(a: f64, b: f64) -> f64 {
    if b > a { b } else { a }
}

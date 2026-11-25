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
//!   kit::letters!(F, "letters", "MARIE", 0.064, 0.003, (-0.04, 0.705, 0.115), M.gold, space = 1.25);   // a name in raised letters
//!   kit::turned!(F, "saucer", x, y, z, &[(0.0, 0.0), (0.03, 0.0), (0.05, 0.006), (0.0, 0.005)], M.porcelain, segments = 16);   // a profile revolved on a game point
//!   kit::lathe!(F, "bowl", x, y, z, &[(0.05, 0.0), (0.09, 0.05), (0.085, 0.06)], M.glaze, verts = 24, square = 7, sy = 0.68);   // rings lofted on one
//!   kit::paint(jar, M.label, |c| 0.03 < c.z && c.z < 0.06);   // a band on a part's own faces
//!   kit::mug!(F, "mug", 0.7, 0.72, -0.2, 0.038, 0.09, M.mug, 200, segments = 14, verts = 5);   // hollow, its handle at 200°
//!   kit::throw!(F, "throw", 0.86, 0.6, 0.458, -0.15, 0.375, M.throw, 0.1, turn = 4, seed = 3);   // folded over a seat's front
//!   kit::board(&F, "slat", (-0.6, 0.45, 0.1), (0.6, 0.45, 0.1), 0.08, 0.02, M.teak, (0, 1, 0), 0.003);   // a board at any angle
//!   kit::tiled_walls!(F, (-0.45, 0.45), (-0.45, 0.45), (0.06, 2.0), 0.03, 0.3, 0.005, M.grout, M.tile);   // a shower's tiled corner
//!   kit::steam_iron(&F, (0.69, 0.88, 0.1), (-1.0, 0.0), M.plate, M.body, M.trim);   // pointing to the left
//!   kit::duvet!(F, top, 0.86, z_fold, z_foot, 0.25, M.bedding, seed = 2);   // a duvet's still two thirds, rumpled
//!   kit::turn_down!(kit::duvet_flap!(F, top, 0.86, z_top, z_fold, M.bedding, M.sheet), "turn_down", 0.86, 1.0);   // its top third folding back
//!   let (verts, faces) = kit::pleats!(F, &rows, -0.8, -0.1, z, 6, 0.018, 0.01);   // a pleated curtain's points and faces
//!   let body = kit::figure8((0.17, 0.2, 0.17), (0.38, 0.145, 0.13), 8, 1);   // a guitar's outline
//!   kit::turn_round(&meshes(), 0.05);   // a model built in place, turned to face the other way
//!   fbox!(F, "back", 1.0, 0.8, 0.02, (0, 0.4, kit::WALL + 0.01), M.oak);   // against the wall's face
//!
//! By kind: `hardware` (pulls, knobs), `turned` (legs, turned and lofted round pieces), `joinery` (bent paths, frame-and-panel fronts, mouldings,
//! a bench's ends, a recess or a hole cut into a body), `metalwork` (tube, scrolls), `creatures` (a songbird),
//! `signs` (the taxi's word and chequers, raised block letters), `tableware`; and a piece a file: `board`, `duvet`
//! (and its flap and clip), `figure8`, `iron`, `mug`, `pleats`, `throw`, `tiled_walls`, `turn_round`, `wall` (`WALL`).

mod board;
pub mod creatures;
mod duvet;
mod figure8;
pub mod hardware;
mod iron;
pub mod joinery;
mod macros;
pub mod metalwork;
mod mug;
mod pleats;
pub mod signs;
pub mod tableware;
mod throw;
mod tiled_walls;
mod turn_round;
pub mod turned;
mod wall;

pub use board::*;
pub use creatures::*;
pub use duvet::*;
pub use figure8::*;
pub use hardware::*;
pub use iron::*;
pub use joinery::*;
pub use metalwork::*;
pub use mug::*;
pub use pleats::*;
pub use signs::*;
pub use throw::*;
pub use tiled_walls::*;
pub use turn_round::*;
pub use turned::*;
pub use wall::*;

// (the macros, exported at the crate's root under `kit_` names, by the names the scripts called: `kit::pull!`)
pub use crate::{
    kit_bench_end as bench_end, kit_cabriole as cabriole, kit_chequer as chequer, kit_cut as cut,
    kit_duvet as duvet, kit_duvet_flap as duvet_flap, kit_fillet as fillet, kit_knob as knob,
    kit_lathe as lathe, kit_letters as letters, kit_moulding as moulding, kit_mug as mug,
    kit_pleats as pleats, kit_pull as pull, kit_recessed_box as recessed_box, kit_scroll as scroll,
    kit_shaker as shaker, kit_songbird as songbird, kit_taxi_word as taxi_word, kit_throw as throw,
    kit_tiled_walls as tiled_walls, kit_tube as tube, kit_turn_down as turn_down,
    kit_turned as turned,
};

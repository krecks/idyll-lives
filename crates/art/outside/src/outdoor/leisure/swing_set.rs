use super::super::common::{F, M};
use super::porch_swing::a_frame;
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::math::{Axis, P3, pow};
use artkit::motion::{Rock, rock};
use artkit::parts::{hinged, join, new_parts};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fsweep, settle, torus};

// --- the swing set: two seats on chains from a round-timber A-frame; they swing while played on ---------------
pub fn swing_set() -> Vec<Obj> {
    let frame = new_parts(|| a_frame("", 1.92, M.swing_frame, true, 0.72)); // splayed for seats swinging 20°
    join(&frame, "frame", Some(P3::from((0, 0, 0))));
    let seats = new_parts(|| {
        for x in [-0.48, 0.48] {
            // a belt seat: a strap sagging between its chains, edges rolled
            let belt: Vec<(f64, f64, i32)> = [-0.21, -0.14, -0.07, 0.0, 0.07, 0.14, 0.21]
                .iter()
                .map(|&u| (x + u, 0.48 + 0.035 * pow(u / 0.2, 2.0), 0))
                .collect();
            artkit::bevel::eased(fsweep!(
                F,
                format!("seat{}", float_repr(x)),
                &belt,
                0,
                M.swing_seat,
                profile = rounded_rect(0.012, 0.2, 0.005, 1),
                up = (0, 0, 1),
                resolution = 2
            ));
            for dx in [-0.2, 0.2] {
                fsweep!(
                    F,
                    format!("chain{}{}", float_repr(x), float_repr(dx)),
                    &[(x + dx, 0.53, 0), (x + dx, 1.86, 0)],
                    0.006,
                    M.chain,
                    verts = 5
                );
                fbox!(
                    F,
                    format!("clip{}{}", float_repr(x), float_repr(dx)),
                    0.03,
                    0.04,
                    0.03,
                    (x + dx, 0.515, 0),
                    M.chain,
                    bevel = 0.008,
                    bevel_segments = 1
                );
                // the hanger round the beam: a ring the chain hangs from (it turns on its own axis as it swings)
                torus!(
                    format!("hanger{}{}", float_repr(x), float_repr(dx)),
                    0.052,
                    0.008,
                    at = F.at(x + dx, 1.92, 0),
                    material = M.chain,
                    rot = (0, 90, 0),
                    major_segments = 12,
                    minor_segments = 4
                );
            }
        }
    });
    let seats = hinged(join(&seats, "seats", None), F.at(0, 1.92, 0));
    settle!(centre = false);
    rock(
        seats,
        "swing",
        20.0,
        Rock {
            seconds: 1.6,
            axis: Axis::X,
        },
    );
    meshes()
}

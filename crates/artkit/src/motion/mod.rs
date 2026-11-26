//! Moving parts: a door, lid or drawer that moves while residents use the object (docs/design/objects.md,
//! "Moving parts"). Each part is its own part with its origin on the hinge (`parts::hinged`), and each clip is one
//! track on it, named for the pack's `motions`.
//!
//! A clip's first key (frame 1) is the rest pose: where the file has the part, and the game draws it closed. Only
//! the part's rotation, location or scale move. `grow` and `squash` go from rest to open (the game plays them
//! back); `spin`, `rock`, `turns` and `wobble` come home to rest within the clip, so they also serve as loops
//! (`motions.loops`, `idle`), and `scroll` moves one repeat on, home unseen; `stages` is a crop's growth, held where
//! its stage is; `path` and `keys` key what the others don't. The file's clips are sampled every frame (30 a
//! second, `scene::FPS`) from frame 1 as 0 s, and the game samples them 60 times a second (twice each key).
//!
//! The curves are Blender's F-curves to the bit: auto-clamped keys whose handles are smoothed for continuous
//! acceleration (Blender's defaults), or one of its easings from a key to the next (`Ease`), all in single
//! precision as Blender evaluates them.

use crate::math::{self, Axis, EulerOrder, M4, P3, PI, Quat, V3};
use crate::py;
use crate::scene::{FPS, Obj, RotationMode};
use crate::shapes::options;

mod clips;
mod decompose;
mod ease;
mod eval;
mod handles;
mod keypoints;
mod keys;
mod moves;
mod path;
mod peak;
mod pose;

pub use clips::*;
pub use decompose::*;
pub use ease::*;
pub use eval::*;
use handles::*;
pub use keypoints::*;
pub use keys::*;
pub use moves::*;
pub use path::*;
pub use peak::*;
pub use pose::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::Mesh;
    use crate::scene;

    fn part() -> Obj {
        scene::reset();
        scene::add("part", Mesh::default())
    }

    fn scale_at(obj: Obj, clip: &str, frame: i32) -> f64 {
        (pose(obj, Some(clip), frame as f64).2.z as f64 * 1e6).round() / 1e6
    }

    #[test]
    fn a_key_list_with_nothing_in_it_changes_nothing() {
        let obj = part();
        assert_eq!(
            key_channel(obj, "sway", Path::Location, 0, &[], Ease::Linear, 0.0),
            obj
        );
        assert!(clips(&[obj]).is_empty());
    }

    #[test]
    #[should_panic(expected = "not a channel of a 3-axis part")]
    fn a_channel_of_a_fourth_axis_names_the_clip_it_is_in() {
        let obj = part();
        key_channel(
            obj,
            "sway",
            Path::Location,
            3,
            &[(0, 0.0)],
            Ease::Linear,
            0.0,
        );
    }

    #[test]
    fn a_squash_eases_as_blenders_keys_do() {
        // the apple's `pick` (0.8 s to a thousandth), as Blender sampled it into models/trees/apple.glb
        let obj = squash(
            part(),
            "pick",
            0.001,
            Squash {
                seconds: 0.8,
                ..Squash::default()
            },
        );
        assert_eq!(clips(&[obj]), [(obj, "pick".to_string(), 25)]);
        let got: Vec<f64> = [1, 2, 3, 4, 5, 24, 25]
            .iter()
            .map(|&f| scale_at(obj, "pick", f))
            .collect();
        assert_eq!(
            got,
            [1.0, 0.994941, 0.980344, 0.957074, 0.926, 0.006059, 0.001]
        );
        assert!(moves_away(obj));
    }

    #[test]
    fn growth_stages_hold_a_third_of_the_way_each() {
        // the apple's `ripen`: gone from picked on, 1.5 s
        let obj = stages(part(), "ripen", 0.001, 0.001, 0.001, Stages::default());
        let got: Vec<f64> = [1, 2, 3, 4, 5, 16, 31, 46]
            .iter()
            .map(|&f| scale_at(obj, "ripen", f))
            .collect();
        assert_eq!(
            got,
            [
                1.0, 0.987272, 0.951456, 0.896104, 0.824768, 0.001, 0.001, 0.001
            ]
        );
        // a plant that only shrinks in height and sinks: its place keyed too
        let obj = stages(
            part(),
            "grow",
            1,
            (1, 1, 0.8),
            (1, 1, 0.5),
            Stages {
                sink: Some((0.0, 0.1, 0.32)),
                ..Stages::default()
            },
        );
        let track = obj.read(|o| o.tracks[0].clone());
        assert_eq!(
            track
                .channels
                .iter()
                .map(|c| (c.path, c.index))
                .collect::<Vec<_>>(),
            [(Path::Scale, 2), (Path::Location, 2)]
        );
        assert_eq!(pose(obj, Some("grow"), 46.0).0.z, -0.32);
        assert!((pose(obj, Some("grow"), 31.0).2.z - 0.8).abs() < 1e-7);
        assert!(!moves_away(squash(
            part(),
            "sit",
            (1, 1, 0.7),
            Squash::default()
        )));
    }

    #[test]
    fn frames_count_from_one_and_round_halves_up() {
        assert_eq!(
            (
                frame(0.0),
                frame(0.8),
                frame(1.5),
                frame(0.05),
                last_frame(0.0)
            ),
            (1, 25, 46, 3, 2)
        );
        assert!((back_amount(OVERSHOOT) - 1.0).abs() < 0.2);
    }
}

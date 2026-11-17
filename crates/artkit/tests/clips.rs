//! The clips a door and a drawer take against the keys Blender sampled into the committed kitchen models (30 a
//! second: models/kitchen/fridge.glb's `freezer_open`, wineFridge.glb's `rack_out`).
use artkit::math::Axis;
use artkit::mesh::Mesh;
use artkit::motion::{Ease, Slide, Swing, clips, moves_away, pose, slide, swing};
use artkit::scene::{self, Obj};

fn part() -> Obj {
    scene::reset();
    scene::add("part", Mesh::default())
}

#[test]
fn a_door_swings_past_open_and_settles_as_blenders_back_ease() {
    // the fridge's freezer door: 118 degrees in 1.75 s, 4 % past open on the way
    let door = swing(
        part(),
        "freezer_open",
        -118.0,
        Swing {
            seconds: 1.75,
            ease: Ease::BackOut,
            ..Swing::default()
        },
    );
    assert_eq!(clips(&[door]), [(door, "freezer_open".to_string(), 54)]);
    let at = |frame: i32| -(pose(door, Some("freezer_open"), frame as f64).1.z as f64).to_degrees();
    for (frame, want) in [
        (1, 0.0),
        (2, 8.78166),
        (3, 17.14637),
        (4, 25.10379),
        (5, 32.66357),
        (9, 59.12061),
        (54, 118.0),
    ] {
        assert!(
            (at(frame) - want).abs() < 2e-4,
            "frame {frame}: {} for {want}",
            at(frame)
        );
    }
    let widest = (1..=54).map(at).fold(0.0, f64::max);
    assert!((122.2..122.73).contains(&widest), "{widest}");
    assert!(moves_away(door));
    // about another axis, the default ease: slow out of rest and into the end
    let lever = swing(
        part(),
        "lever",
        -30.0,
        Swing {
            seconds: 0.3,
            axis: Axis::X,
            ..Swing::default()
        },
    );
    assert_eq!(clips(&[lever])[0].2, 10);
    let turned = |frame: f64| (pose(lever, Some("lever"), frame).1.x as f64).to_degrees();
    assert!((turned(10.0) + 30.0).abs() < 1e-5 && (turned(5.5) + 15.0).abs() < 1e-5);
    assert!(turned(2.0) > -1.5, "{}", turned(2.0));
}

#[test]
fn a_drawer_slides_as_blenders_default_keys() {
    // the wine fridge's rack: 15 cm to the front in 0.8 s, from where it rests
    let rack = part();
    rack.set_location((-0.22, 0.140_214_1, 0.74));
    slide(
        rack,
        "rack_out",
        (0.0, -0.15, 0.0),
        Slide {
            seconds: 0.8,
            ..Slide::default()
        },
    );
    assert_eq!(clips(&[rack]), [(rack, "rack_out".to_string(), 25)]);
    let at = |frame: i32| pose(rack, Some("rack_out"), frame as f64).0;
    for (frame, want) in [
        (1, 0.140_214_1),
        (2, 0.139_454_54),
        (3, 0.137_262_72),
        (4, 0.133_768_8),
        (5, 0.129_102_99),
        (25, -0.009_785_9),
    ] {
        assert!(
            (at(frame).y - want).abs() < 2e-7,
            "frame {frame}: {} for {want}",
            at(frame).y
        );
    }
    // only the way it moves is keyed
    assert_eq!((at(12).x, at(12).z), (-0.22, 0.74));
    let track = rack.read(|o| o.tracks[0].clone());
    assert_eq!(track.channels.len(), 1);
    assert!(moves_away(rack));
}

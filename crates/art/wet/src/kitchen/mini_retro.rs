//! The pastel-pink retro mini fridge, as in the reference photo. Not in the family yet: its raised gold letters are
//! made by Blender's text tool, and the kernel's block letters have not landed (`letters`).

use super::M;
use super::common::{apple, carton, gasket, jar, soft, tube};
use super::cut::cut_cabinet;
use super::fridge::{bottle, door_bin};
use super::sticker::unicorn;
use artkit::materials::Mat;
use artkit::motion::Ease;
use artkit::parts::{hinged, join, new_parts, settle_all};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{fbox, fcone, fcyl, swing};

/// Raised metal letters on a face at game depth `at[2]`, centred on (x, height), facing the front: Blender's text
/// (`size` high, 1.5 mm deep, letters spaced 1.25, three steps a curve) made a mesh, its faces turned outward.
/// It waits for the kernel's block letters.
fn letters(_f: &GameFrame, text: &str, _at: (f64, f64, f64), _size: f64, _material: Mat) -> Obj {
    unimplemented!("raised letters ({text:?}) wait for the kernel's block letters")
}

/// A pastel-pink retro mini fridge, as in the reference photo: a soft rounded cabinet on tapered brass
/// feet, vents at the back of its sides, and one door over the whole front with a brass D handle, raised
/// gold letters ("MARIE") and a die-cut unicorn sticker.
#[allow(dead_code)] // (out of the family until `letters` can be made)
pub fn fridge_mini_retro() -> Vec<Obj> {
    let fp = GameFrame::new(0.0);
    let front = 0.115; // the door's face; the handle reaches the old part's front (0.15)
    let still = new_parts(|| {
        // the cabinet: one rounded shell, its inside cut out through the front and lined
        let cab = fbox!(
            fp,
            "cabinet",
            0.57,
            0.81,
            0.50,
            (0, 0.455, -0.18),
            M.pink,
            bevel = 0.05,
            bevel_segments = 4
        );
        let cut = fbox!(
            fp,
            "cavity",
            0.47,
            0.66,
            0.51,
            (0, 0.46, -0.135),
            M.liner,
            bevel = 0.015
        );
        artkit::bevel::apply(cut);
        cut_cabinet(cab, cut);
        soft(cab);
        cut.remove();
        // a shelf and the freezer floor; food, seen once the door opens
        fbox!(fp, "divider", 0.47, 0.014, 0.44, (0, 0.62, -0.16), M.shelf);
        fbox!(fp, "shelf", 0.47, 0.008, 0.44, (0, 0.36, -0.16), M.shelf);
        fbox!(
            fp,
            "shelf_trim",
            0.47,
            0.016,
            0.008,
            (0, 0.358, 0.058),
            M.pink2,
            bevel = 0.003,
            bevel_segments = 1
        );
        carton(&fp, "milk", -0.12, 0.364, -0.20, 0.07, 0.13, M.carton, 0);
        for (i, x) in [0.05, 0.13].into_iter().enumerate() {
            apple(
                &fp,
                &format!("apple{i}"),
                x,
                0.364,
                -0.17 + 0.03 * i as f64,
                0.032,
                None,
            );
        }
        bottle(
            "juice",
            (0.08, 0.135, -0.2),
            0.026,
            0.15,
            M.bottle,
            M.carton,
            10,
            &fp,
        );
        fbox!(
            fp,
            "icecream",
            0.12,
            0.07,
            0.10,
            (-0.05, 0.662, -0.18),
            M.icecream,
            bevel = 0.01
        );
        // tapered brass feet, wide at the top
        for x in [-0.215, 0.215] {
            for z in [-0.37, 0.02] {
                fcone!(
                    fp,
                    "foot",
                    0.019,
                    0.027,
                    0.05,
                    (x, 0.025, z),
                    M.gold,
                    verts = 12
                );
            }
        }
        for side in [-1, 1] {
            let x = side as f64 * 0.285;
            // the compressor panel's seam low on each side
            fbox!(
                fp,
                "side_seam",
                0.003,
                0.006,
                0.39,
                (x, 0.165, -0.18),
                M.pink2,
                bevel = 0.0
            );
            // two groups of seven vent slots at the back of each side
            for y0 in [0.20, 0.61] {
                for i in 0..7 {
                    fbox!(
                        fp,
                        "vent",
                        0.004,
                        0.006,
                        0.07,
                        (x, y0 + i as f64 * 0.014, -0.33),
                        M.vent,
                        bevel = 0.0
                    );
                }
            }
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // the door over the whole front, hinged on the viewer's right (its handle on the left)
    let parts = new_parts(|| {
        let door = fbox!(
            fp,
            "door",
            0.555,
            0.785,
            0.045,
            (0, 0.4525, front - 0.0225),
            M.pink,
            bevel = 0.035,
            bevel_segments = 3
        );
        soft(door);
        fbox!(
            fp,
            "door_liner",
            0.45,
            0.64,
            0.008,
            (0, 0.46, 0.066),
            M.liner
        );
        gasket(&fp, "door_seal", 0.5, 0.72, 0.4525, 0.071, 0.006, 0.04, 0.0);
        door_bin("door_bin", 0.27, -0.2, 0.2, 0.022, 0.064, &fp);
        fcyl!(
            fp,
            "can",
            0.03,
            0.11,
            (0.12, 0.335, 0.043),
            M.produce,
            verts = 10
        );
        jar(
            &fp, "door_jar", -0.1, 0.28, 0.043, 0.026, 0.08, M.egg, M.trim, 10,
        );
        // the brass D handle in one piece: a bar on two curved arms into the door, a round pad under each
        let (hx, y0, y1, out) = (0.17, 0.35, 0.60, front + 0.025);
        let pts = [
            (hx, y0 - 0.02, front),
            (hx, y0, out - 0.006),
            (hx, y0 + 0.025, out),
            (hx, y1 - 0.025, out),
            (hx, y1, out - 0.006),
            (hx, y1 + 0.02, front),
        ];
        tube(&fp, "handle", &pts, 0.0095, M.gold, Some(8), 3, 1.0);
        for (i, y) in [y0 - 0.02, y1 + 0.02].into_iter().enumerate() {
            fcyl!(
                fp,
                format!("handle_pad{i}"),
                0.016,
                0.006,
                (hx, y, front + 0.002),
                M.gold,
                verts = 12,
                rot = (90, 0, 0)
            );
        }
        letters(&fp, "MARIE", (-0.04, 0.705, front), 0.064, M.gold);
        unicorn(&fp, (-0.06, 0.44, front));
    });
    let door = hinged(
        join(&parts, "door", None),
        fp.at(-0.2775, 0.4525, front - 0.0225),
    );
    settle_all();
    swing!(
        door,
        "fridge_open",
        100,
        seconds = 1.6,
        ease = Ease::BackOut
    );
    Vec::new()
}

//! The shop counter: 2 x 1, an oak counter with a till, a sweet jar and a magazine rack, sitting back on its tiles
//! as the game places it; the till's cash drawer slides out to the shopkeeper.

use super::common::{M, jar, tilted};
use artkit::forms::rounded_rect;
use artkit::math::{P3, PI, cos, radians, sin};
use artkit::motion::{Slide, slide};
use artkit::parts::{join, new_parts};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, kit_knob, kit_shaker, revolve, settle};

const TILL_SECONDS: f64 = 0.8; // the cash drawer pops out this fast

// --- the shop counter: 2x1, an oak counter with a till, a sweet jar and a magazine rack -------------
pub fn shop_counter() -> Vec<Obj> {
    let f = GameFrame::new(0.04); // 4 cm forward: the worktop's back on the wall face (-0.43)
    let (z, front, top) = (-0.08, 0.27, 0.97); // the counter's depth centre, front face, worktop
    let (tx, tz) = (0.5, -0.2); // the till on the worktop
    let still = new_parts(|| {
        fbox!(
            f,
            "plinth",
            1.84,
            0.08,
            0.64,
            (0, 0.04, z - 0.01),
            M.walnut,
            bevel = 0.004
        );
        fbox!(
            f,
            "carcass",
            1.9,
            0.84,
            0.7,
            (0, 0.5, z),
            M.counter,
            bevel = 0.008
        );
        // the worktop: walnut with a rounded nose along its front, a cove moulding under it
        artkit::bevel::eased(fextrude!(
            f,
            "worktop",
            &[
                (-0.47, 0.92),
                (0.29, 0.92),
                (0.302, 0.924),
                (0.31, 0.936),
                (0.312, 0.95),
                (0.308, 0.962),
                (0.296, 0.969),
                (0.28, 0.97),
                (-0.47, 0.97)
            ],
            1.96,
            (0, 0, 0),
            M.walnut,
            plane = Facing::Side
        ));
        fsweep!(
            f,
            "cove",
            &[(-0.95, 0.905, front + 0.008), (0.95, 0.905, front + 0.008)],
            0,
            M.walnut,
            profile = [
                (0.012, -0.01),
                (0.012, 0.012),
                (-0.012, 0.012),
                (-0.012, 0.004),
                (0.0, 0.0),
                (0.006, -0.01)
            ]
        );
        // the front: three frame-and-panel bays, walnut frames round raised oak panels, a walnut kick rail
        fbox!(
            f,
            "rail_bottom",
            1.9,
            0.08,
            0.02,
            (0, 0.12, front + 0.01),
            M.walnut,
            bevel = 0.004
        );
        for (i, x) in [-0.6175, 0.0, 0.6175].into_iter().enumerate() {
            kit_shaker!(
                f,
                format!("bay{i}"),
                0.615,
                0.74,
                (x, 0.53, front + 0.012),
                0.024,
                M.walnut,
                rail = 0.07,
                panel = M.counter,
                raised = true
            );
        }
        // the back, the shopkeeper's side: two cupboard doors with knobs
        for (i, x) in [-0.47, 0.47].into_iter().enumerate() {
            let (door, panel) = kit_shaker!(
                f,
                format!("back_door{i}"),
                0.86,
                0.7,
                (x, 0.5, z - 0.358),
                0.018,
                M.counter,
                rail = 0.07
            );
            for part in [door, panel] {
                part.set_rotation_euler_axis(2, PI); // facing the back
            }
            kit_knob!(
                f,
                format!("back_knob{i}"),
                x - 0.36 * (if x > 0.0 { 1.0 } else { -1.0 }),
                0.74,
                z - 0.367,
                0.014,
                M.trim
            )
            .set_rotation_euler_axis(0, radians(-90.0));
        }
        // the till, the shopkeeper's: a case holding the drawer, a keypad sloping up to the front (to the shopkeeper
        // behind the counter) and the customer's display on a post at its front, a receipt printer with its paper
        // curling out
        fbox!(
            f,
            "till_case",
            0.38,
            0.09,
            0.34,
            (tx, top + 0.045, tz),
            M.till,
            bevel = 0.008
        );
        let kc = (tx, top + 0.115, tz + 0.04);
        fbox!(
            f,
            "till_keypad",
            0.34,
            0.05,
            0.2,
            kc,
            M.till,
            bevel = 0.008,
            rot = (-12, 0, 0)
        );
        for r in 0..3 {
            for c in 0..4 {
                let at = tilted(
                    tx - 0.105 + c as f64 * 0.07,
                    kc.1,
                    kc.2,
                    -12.0,
                    -(0.06 - r as f64 * 0.05),
                    0.03,
                );
                fbox!(
                    f,
                    format!("key{r}{c}"),
                    0.05,
                    0.012,
                    0.035,
                    (at.0, at.1, at.2),
                    if (r, c) == (0, 3) { M.key2 } else { M.keys },
                    bevel = 0.003,
                    rot = (-12, 0, 0)
                );
            }
        }
        fcyl!(
            f,
            "till_post",
            0.015,
            0.1,
            (tx, top + 0.17, tz + 0.13),
            M.till,
            verts = 8
        );
        fbox!(
            f,
            "till_display",
            0.22,
            0.1,
            0.03,
            (tx, top + 0.255, tz + 0.13),
            M.till,
            bevel = 0.006,
            rot = (-15, 0, 0)
        );
        // its screen on the display's face, which leans back 15 degrees
        fbox!(
            f,
            "till_screen",
            0.18,
            0.07,
            0.006,
            (tx, top + 0.255 + 0.0047, tz + 0.13 + 0.0174),
            M.screen,
            rot = (-15, 0, 0)
        );
        fbox!(
            f,
            "printer",
            0.12,
            0.08,
            0.12,
            (tx - 0.27, top + 0.04, tz - 0.05),
            M.till,
            bevel = 0.012
        );
        fsweep!(
            f,
            "receipt",
            &[
                (tx - 0.27, top + 0.075, tz - 0.07),
                (tx - 0.27, top + 0.11, tz - 0.06),
                (tx - 0.27, top + 0.12, tz - 0.02),
                (tx - 0.27, top + 0.1, tz + 0.01)
            ],
            0,
            M.page,
            profile = rounded_rect(0.07, 0.002, 0.0, 4),
            up = (0, 0, 1),
            resolution = 2
        );
        // a bell to ring for service
        revolve!(
            "bell",
            &[
                (0.0, 0.0),
                (0.045, 0.0),
                (0.045, 0.008),
                (0.04, 0.012),
                (0.035, 0.03),
                (0.022, 0.045),
                (0.008, 0.05),
                (0.006, 0.062),
                (0.01, 0.066),
                (0.0, 0.068)
            ],
            at = f.at(0.05, top, 0.18),
            material = M.bell,
            segments = 14
        );
        // the sweet jar: a gloss jar with sweets pressed to its sides, as if seen through it
        let (jx, jz) = (-0.2, 0.05);
        jar(&f, "jar", jx, top, jz, 0.085, 0.23, M.jar, M.jarLid);
        let sweets = [M.sweet1, M.sweet2, M.sweet3, M.sweet4, M.sweet5];
        let mut k = 0;
        for (row, y) in [0.03, 0.075, 0.12, 0.165].into_iter().enumerate() {
            for j in 0..7 {
                let a = radians((j * 360) as f64 / 7.0 + (row * 25) as f64);
                fsphere!(
                    f,
                    format!("sweet{row}{j}"),
                    0.021,
                    (jx + 0.075 * sin(a), top + y, jz + 0.075 * cos(a)),
                    sweets[k % 5],
                    segments = 6,
                    rings = 4
                );
                k += 3;
            }
        }
        // the magazine rack: a walnut stand with a lip, four magazines leaning back
        let (rx, rz) = (-0.66, -0.24);
        fbox!(
            f,
            "rack_base",
            0.52,
            0.03,
            0.18,
            (rx, top + 0.015, rz),
            M.walnut,
            bevel = 0.004
        );
        fbox!(
            f,
            "rack_lip",
            0.52,
            0.04,
            0.012,
            (rx, top + 0.04, rz + 0.084),
            M.walnut,
            bevel = 0.004
        );
        fbox!(
            f,
            "rack_back",
            0.52,
            0.22,
            0.015,
            (rx, top + 0.13, rz - 0.09),
            M.walnut,
            bevel = 0.004,
            rot = (-15, 0, 0)
        );
        for s in [-1, 1] {
            artkit::bevel::eased(fextrude!(
                f,
                format!("rack_end{s}"),
                &[
                    (-0.09, 0.0),
                    (0.09, 0.0),
                    (0.09, 0.05),
                    (-0.06, 0.24),
                    (-0.09, 0.24)
                ],
                0.012,
                (rx + s as f64 * 0.254, top, rz),
                M.walnut,
                plane = Facing::Side
            ));
        }
        for (i, mag) in [M.mag1, M.mag2, M.mag3, M.mag4].into_iter().enumerate() {
            let (x, z) = (
                rx - 0.18 + i as f64 * 0.12,
                rz - 0.045 + (i % 2) as f64 * 0.03,
            );
            fbox!(
                f,
                format!("magazine{i}"),
                0.16,
                0.23,
                0.01,
                (x, top + 0.145, z),
                mag,
                rot = (-15, 0, 0)
            );
            let (up, t) = (0.09, radians(15.0)); // the title strip near the top of its leaning cover
            fbox!(
                f,
                format!("masthead{i}"),
                0.13,
                0.03,
                0.004,
                (
                    x,
                    top + 0.145 + up * cos(t) + 0.0018,
                    z - up * sin(t) + 0.0068
                ),
                M.page,
                rot = (-15, 0, 0)
            );
            fbox!(
                f,
                format!("cover{i}"),
                0.09,
                0.08,
                0.004,
                (x, top + 0.11, z + 0.0068 + 0.035 * sin(t)),
                M.page,
                rot = (-15, 0, 0)
            );
        }
    });
    join(&still, "counter", Some(P3::from((0, 0, 0))));

    // the cash drawer, opening to the shopkeeper: a front under the keypad and a tray of notes and coins hidden in
    // the case at rest
    let dz = tz - 0.17; // the case's back face
    let drawer = new_parts(|| {
        fbox!(
            f,
            "drawer_front",
            0.35,
            0.065,
            0.014,
            (tx, top + 0.042, dz - 0.007),
            M.till,
            bevel = 0.004
        );
        fbox!(
            f,
            "drawer_lip",
            0.12,
            0.012,
            0.012,
            (tx, top + 0.06, dz - 0.018),
            M.keys,
            bevel = 0.003
        );
        fbox!(
            f,
            "drawer_tray",
            0.33,
            0.045,
            0.3,
            (tx, top + 0.03, dz + 0.15),
            M.keys,
            bevel = 0.003
        );
        for i in 0..3 {
            fbox!(
                f,
                format!("divider{i}"),
                0.006,
                0.012,
                0.28,
                (tx - 0.09 + i as f64 * 0.09, top + 0.058, dz + 0.15),
                M.till
            );
        }
        for i in 0..4 {
            fbox!(
                f,
                format!("notes{i}"),
                0.07,
                0.008,
                0.13,
                (tx - 0.135 + i as f64 * 0.09, top + 0.056, dz + 0.22),
                M.note
            );
            for j in 0..2 {
                fcyl!(
                    f,
                    format!("coin{i}{j}"),
                    0.012,
                    0.006,
                    (
                        tx - 0.15 + i as f64 * 0.09 + j as f64 * 0.03,
                        top + 0.055,
                        dz + 0.07
                    ),
                    M.coin,
                    verts = 8
                );
            }
        }
    });
    let drawer = join(&drawer, "cash_drawer", Some(f.at(tx, top + 0.03, dz)));

    settle!(centre = false);
    // (10 cm, to the worktop's back: against a wall at home it stops at the wall's face.)
    slide(
        drawer,
        "till",
        (0, 0.1, 0),
        Slide {
            seconds: TILL_SECONDS,
            ..Slide::default()
        },
    );
    meshes()
}

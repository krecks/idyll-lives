//! The chef's island (2x1): a marble-topped island, a hob on it, drawers on the cook's side, two stools.

use super::common::M;
use artkit::bevel::ease;
use artkit::forms::rounded_rect;
use artkit::kit;
use artkit::math::{Axis, P3, cosd, sind};
use artkit::parts::{join, new_parts, numbered};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, grow, revolve, settle, slide, torus};

const TOP: f64 = 0.9; // the content's top (`surface.height`)
const RING: f64 = 0.145; // the hob's two rings' x (the left one the cook's)
const STOOL: f64 = 0.65; // the counter stools' seat top: 25 cm under the top (counter stools are 0.58-0.71 for it)
const STOOL_Z: f64 = -0.235; // their middles' depth, the knees under the top's back overhang (`furniture.json`)

/// A counter stool with its seat at STOOL, a foot ring and a low curved back, centred on (x, z), facing the front:
/// four splayed legs, each one tapering sweep, a saddle-dished seat with a cushion.
fn _stool(f: &GameFrame, x: f64, z: f64) {
    let y = STOOL - 0.045; // the wooden seat's middle
    fcyl!(
        f,
        numbered("stool_seat"),
        0.18,
        0.03,
        (x, y, z),
        M.oak,
        verts = 20,
        bevel = 0.008,
        bevel_segments = 1
    );
    fcyl!(
        f,
        numbered("stool_cushion"),
        0.165,
        0.03,
        (x, STOOL - 0.015, z),
        M.seat,
        verts = 18,
        bevel = 0.01,
        bevel_segments = 2
    );
    for k in 0..4 {
        let a = (45 + 90 * k) as f64;
        let (c, s) = (cosd(a), sind(a));
        fsweep!(
            f,
            numbered("stool_leg"),
            &[
                (x + 0.12 * c, y - 0.015, z + 0.12 * s),
                (x + 0.19 * c, 0.008, z + 0.19 * s)
            ],
            0.017,
            M.oak_dark,
            verts = 8,
            scale = [1.0, 0.7]
        );
    }
    torus!(
        numbered("stool_ring"),
        0.165,
        0.009,
        at = f.at(x, 0.3, z),
        material = M.oak_dark,
        major_segments = 16,
        minor_segments = 4
    );
    for dx in [-0.12, 0.12] {
        kit::tube!(
            f,
            numbered("stool_back_post"),
            &[
                (x + dx, STOOL - 0.03, z - 0.13),
                (x + dx, STOOL + 0.17, z - 0.17)
            ],
            0.012,
            M.oak_dark,
            verts = 6,
            round = 1
        );
    }
    ease(
        fsweep!(
            f,
            numbered("stool_back"),
            &[
                (x - 0.16, STOOL + 0.21, z - 0.15),
                (x, STOOL + 0.21, z - 0.183),
                (x + 0.16, STOOL + 0.21, z - 0.15)
            ],
            0.0,
            M.oak,
            profile = rounded_rect(0.1, 0.022, 0.008, 1),
            up = (0, 0, 1),
            resolution = 3
        ),
        60.0,
        1,
    );
}

pub fn chefs_island() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let zf = 0.43; // the cook's side
    let still = new_parts(|| {
        fextrude!(
            f,
            "top",
            &rounded_rect(1.86, 0.46, 0.02, 2),
            0.04,
            (0, TOP - 0.02, 0.225),
            M.marble,
            plane = Facing::Top,
            bevel = 0.01,
            bevel_segments = 2
        );
        for s in [-1, 1] {
            let x = s as f64 * 0.8;
            fbox!(
                f,
                numbered("end"),
                0.26,
                0.78,
                zf,
                (x, 0.47, zf / 2.0),
                M.cabinet,
                bevel = 0.006,
                bevel_segments = 1
            );
            fbox!(
                f,
                numbered("end_plinth"),
                0.24,
                0.08,
                zf - 0.03,
                (x, 0.04, zf / 2.0 - 0.015),
                M.dark
            );
            kit::shaker!(
                f,
                numbered("end_drawer"),
                0.22,
                0.16,
                (x, 0.75, zf + 0.007),
                0.014,
                M.cabinet,
                rail = 0.03
            );
            kit::shaker!(
                f,
                numbered("end_door"),
                0.22,
                0.54,
                (x, 0.37, zf + 0.007),
                0.014,
                M.cabinet,
                rail = 0.045
            );
            for y in [0.79, 0.6] {
                kit::pull!(
                    f,
                    numbered("end_pull"),
                    x,
                    y,
                    zf + 0.014,
                    0.09,
                    M.brass,
                    r = 0.005,
                    out = 0.012,
                    verts = 6
                );
            }
        }
        // between the ends, the drawer bank under the top; below it, the sitters' legroom open right through
        fbox!(f, "apron", 1.34, 0.16, 0.02, (0, 0.8, 0.3), M.cabinet); // the drawers' back
        fbox!(
            f,
            "apron_under",
            1.34,
            0.02,
            zf - 0.3,
            (0, 0.73, (zf + 0.3) / 2.0),
            M.cabinet
        );
        for x in [-0.45, 0.45] {
            kit::shaker!(
                f,
                numbered("side_drawer"),
                0.42,
                0.13,
                (x, 0.8, zf + 0.007),
                0.014,
                M.cabinet,
                rail = 0.03
            );
            kit::pull!(
                f,
                numbered("side_pull"),
                x,
                0.8,
                zf + 0.014,
                0.12,
                M.brass,
                r = 0.005,
                out = 0.012,
                verts = 6
            );
        }
        // the sitters' side: their knee space under the top, a brass foot rail on brackets
        kit::tube!(
            f,
            "foot_rail",
            &kit::fillet!(
                &[
                    (-0.67, 0.25, 0.2),
                    (-0.67, 0.25, 0.06),
                    (0.67, 0.25, 0.06),
                    (0.67, 0.25, 0.2)
                ],
                0.04
            ),
            0.012,
            M.brass,
            verts = 8,
            round = 1
        );
        // on the top: the hob with a stock pot on its right ring, two tasting plates. The left ring is the cook's
        // (`burner_on` glows under their pan, `STATION_SPOTS`: the copper pan modelled there showed through it), and
        // both ends are left free, one for each of two cooks side by side: their board or bowl at the end, what they
        // set down between it and the hob (`SET_DOWN_SPOTS`; a board of vegetables modelled on the right end was in
        // the way)
        fextrude!(
            f,
            "hob",
            &rounded_rect(0.58, 0.3, 0.02, 2),
            0.008,
            (0, TOP + 0.004, 0.27),
            M.hob,
            plane = Facing::Top,
            bevel = 0.002,
            bevel_segments = 1
        );
        for dx in [-RING, RING] {
            torus!(
                numbered("hob_ring"),
                0.08,
                0.003,
                at = f.at(dx, TOP + 0.009, 0.27),
                material = M.ring,
                major_segments = 16,
                minor_segments = 3
            );
            fcyl!(
                f,
                numbered("hob_mark"),
                0.05,
                0.001,
                (dx, TOP + 0.0085, 0.27),
                M.ring,
                verts = 12
            );
        }
        // (a saucepan-high pot, 12 cm: the island stands no higher than an island with it)
        revolve!(
            "stock_pot",
            &[
                (0.0, 0.0),
                (0.11, 0.0),
                (0.12, 0.01),
                (0.12, 0.12),
                (0.112, 0.12),
                (0.112, 0.012),
                (0.0, 0.012)
            ],
            at = f.at(RING, TOP + 0.008, 0.27),
            material = M.worktop,
            segments = 18
        );
        revolve!(
            "pot_lid",
            &[
                (0.0, 0.0),
                (0.122, 0.0),
                (0.12, 0.008),
                (0.08, 0.02),
                (0.0, 0.024)
            ],
            at = f.at(RING, TOP + 0.128, 0.27),
            material = M.worktop,
            segments = 18
        );
        revolve!(
            "pot_knob",
            &[
                (0.0, 0.0),
                (0.008, 0.0),
                (0.008, 0.01),
                (0.016, 0.016),
                (0.012, 0.024),
                (0.0, 0.025)
            ],
            at = f.at(RING, TOP + 0.15, 0.27),
            material = M.black,
            segments = 8
        );
        for s in [-1, 1] {
            let sf = s as f64;
            kit::tube!(
                f,
                numbered("pot_handle"),
                &kit::fillet!(
                    &[
                        (RING + sf * 0.118, TOP + 0.1, 0.25),
                        (RING + sf * 0.15, TOP + 0.1, 0.255),
                        (RING + sf * 0.15, TOP + 0.1, 0.285),
                        (RING + sf * 0.118, TOP + 0.1, 0.29)
                    ],
                    0.008
                ),
                0.005,
                M.chrome,
                verts = 5,
                round = 1
            );
        }
        for x in [-0.45, 0.45] {
            revolve!(
                numbered("plate"),
                &[
                    (0.0, 0.0),
                    (0.07, 0.0),
                    (0.11, 0.012),
                    (0.11, 0.0155), // (a 3.5 mm rim band: too narrow to show hard)
                    (0.105, 0.0165),
                    (0.068, 0.006),
                    (0.0, 0.006)
                ],
                at = f.at(x, TOP, 0.11),
                material = M.plate,
                segments = 16
            );
            fsphere!(
                f,
                numbered("bite"),
                0.03,
                (x - 0.02, TOP + 0.02, 0.11),
                M.salami,
                scale = (1, 1, 0.5),
                segments = 8,
                rings = 5
            );
            fextrude!(
                f,
                numbered("bite"),
                &[(0.0, 0.0), (0.05, -0.016), (0.05, 0.016)],
                0.025,
                (x + 0.01, TOP + 0.02, 0.1),
                M.cheese_light,
                plane = Facing::Top,
                rot = (0, 0, 30),
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "menu",
            0.14,
            0.18,
            0.006,
            (0.0, TOP + 0.08, 0.06),
            M.chalk,
            rot = (-12, 0, 0),
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            f,
            "menu_stand",
            0.02,
            0.12,
            0.006,
            (0.0, TOP + 0.055, 0.03),
            M.oak_dark,
            rot = (18, 0, 0)
        );
        _stool(f, -0.5, STOOL_Z);
        _stool(f, 0.5, STOOL_Z);
    });
    join(&still, "body", Some(P3::from((0, 0, 0))));
    // The middle drawer on the cook's side: `drawer_open` pulls it out 12 cm.
    let drawer = new_parts(|| {
        kit::shaker!(
            f,
            "drawer_front",
            0.42,
            0.13,
            (0, 0.8, zf + 0.007),
            0.014,
            M.cabinet,
            rail = 0.03
        );
        kit::pull!(
            f,
            "drawer_pull",
            0,
            0.8,
            zf + 0.014,
            0.12,
            M.brass,
            r = 0.005,
            out = 0.012
        );
        fbox!(
            f,
            "drawer_floor",
            0.38,
            0.01,
            0.12,
            (0, 0.745, zf - 0.06),
            M.cabinet
        );
        for x in [-0.19, 0.19] {
            fbox!(
                f,
                numbered("drawer_side"),
                0.01,
                0.08,
                0.12,
                (x, 0.785, zf - 0.06),
                M.cabinet
            );
        }
        for (k, x) in [-0.12, -0.05, 0.03, 0.1].into_iter().enumerate() {
            fbox!(
                f,
                numbered("utensil"),
                0.02,
                0.008,
                0.11,
                (x, 0.755, zf - 0.06),
                [M.chrome, M.oak_top][k % 2],
                bevel = 0.003,
                bevel_segments = 1
            );
        }
    });
    let drawer = join(&drawer, "drawer", None);
    // The left ring glowing hot under the cook's pan, scaled to nothing at rest: `burner_on` grows it while it cooks.
    let heat = new_parts(|| {
        fcyl!(
            f,
            "hot_ring",
            0.13,
            0.003,
            (-RING, TOP + 0.0085, 0.27),
            M.heat,
            verts = 24
        );
    });
    let heat = join(&heat, "hot_ring", Some(f.at(-RING, TOP + 0.0085, 0.27)));
    heat.set_scale((0.001, 0.001, 0.001));
    let objs = settle!(centre = false);
    slide!(drawer, "drawer_open", (0, -0.12, 0), seconds = 0.6);
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        grow!(heat, "burner_on", seconds = 0.5, axis = axis);
    }
    objs
}

//! The phone box: a red kiosk, glazed on every side, its door hinged on one side.

use super::common::own;
use artkit::materials::Mat;
use artkit::math::{PI, cos, sin};
use artkit::motion::{Ease, Swing, swing};
use artkit::parts::{hinged, join, new_parts, settle_all};
use artkit::place::N;
use artkit::place::{Coord, Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, kit_pull, loft, ring};

const KIOSK: f64 = 0.39; // the corner posts' middles, either side of the middle
const FACE: f64 = 0.41; // the glazing's plane, out from the middle
const DOOR_W: f64 = 0.66; // the door, between the front posts

/// A side's (across, height, out) point as the script's `at` gives it, each whole or not.
type At<'a> = &'a dyn Fn(N, f64, N) -> (N, f64, N);

/// One side's window: a pane and its glazing bars (three columns, eight rows), `at` its middle. `rotate` turns the
/// side's points: (x, z) -> (x', z') about the kiosk's middle.
fn glazing(f: &GameFrame, name: &str, material: Mat, bars: Mat, at: At, rotate: f64) {
    let (w, y0, y1) = (0.62, 0.58, 2.16);
    let pane = fbox!(
        f,
        format!("{name}_pane"),
        w,
        y1 - y0,
        0.012,
        at(N::I(0), (y0 + y1) / 2.0, N::I(0)),
        material
    );
    pane.set_rotation_euler_axis(2, rotate);
    for (k, x) in [-0.11, 0.11].into_iter().enumerate() {
        let b = fbox!(
            f,
            format!("{name}_bar{k}"),
            0.025,
            y1 - y0,
            0.02,
            at(N::F(x), (y0 + y1) / 2.0, N::F(0.012)),
            bars
        );
        b.set_rotation_euler_axis(2, rotate);
    }
    for k in 0..7 {
        let y = y0 + (y1 - y0) * (k + 1) as f64 / 8.0;
        let b = fbox!(
            f,
            format!("{name}_rail{k}"),
            w,
            0.022,
            0.02,
            at(N::I(0), y, N::F(0.012)),
            bars
        );
        b.set_rotation_euler_axis(2, rotate);
    }
}

/// One side's arched top over the sign band: a slab rising in a shallow arch from the cornice, a crown in relief
/// on its face. `face` is +1/-1, `along_z` for the front and back, not for the sides.
fn pediment(f: &GameFrame, name: &str, face: i32, along_z: bool, material: Mat, crown: Mat) {
    let mut pts = vec![(-0.46, 0.0), (0.46, 0.0), (0.46, 0.03)];
    pts.extend((1..10).map(|i| {
        (
            0.44 * cos(PI * i as f64 / 10.0),
            0.03 + 0.075 * sin(PI * i as f64 / 10.0),
        )
    }));
    pts.push((-0.46, 0.03));
    let cpts = [
        (-0.035, -0.02),
        (0.035, -0.02),
        (0.04, 0.012),
        (0.022, 0.0),
        (0.012, 0.02),
        (0.0, 0.004),
        (-0.012, 0.02),
        (-0.022, 0.0),
        (-0.04, 0.012),
    ];
    let face = face as f64;
    if along_z {
        artkit::bevel::eased(fextrude!(
            f,
            name,
            &pts,
            0.03,
            (0, 2.56, face * 0.455),
            material
        ));
        fextrude!(
            f,
            format!("{name}_crown"),
            &cpts,
            0.008,
            (0, 2.6, face * 0.472),
            crown
        );
    } else {
        artkit::bevel::eased(fextrude!(
            f,
            name,
            &pts,
            0.03,
            (face * 0.455, 2.56, 0),
            material,
            plane = Facing::Side
        ));
        fextrude!(
            f,
            format!("{name}_crown"),
            &cpts,
            0.008,
            (face * 0.472, 2.6, 0),
            crown,
            plane = Facing::Side
        );
    }
}

pub fn phone_booth() -> Vec<Obj> {
    let m = own("phoneBooth");
    let f = GameFrame::new(0.0);
    let red = m.body;
    let still = new_parts(|| {
        fbox!(
            f,
            "step",
            0.94,
            0.04,
            0.94,
            (0, 0.02, 0),
            red,
            bevel = 0.008,
            bevel_segments = 1
        );
        fbox!(f, "plinth", 0.9, 0.1, 0.9, (0, 0.07, 0), red, bevel = 0.01);
        for sx in [-1, 1] {
            for sz in [-1, 1] {
                fbox!(
                    f,
                    format!("post{sx}{sz}"),
                    0.12,
                    2.1,
                    0.12,
                    (sx as f64 * KIOSK, 1.17, sz as f64 * KIOSK),
                    red,
                    bevel = 0.012
                );
            }
        }
        // the band at the top, its TELEPHONE signs (lit at night), the cornice, an arched pediment on each side with
        // a crown on it, and the shallow dome rising between them to a crown knob
        fbox!(f, "band", 0.92, 0.3, 0.92, (0, 2.36, 0), red, bevel = 0.015);
        fbox!(
            f,
            "cornice",
            0.95,
            0.05,
            0.95,
            (0, 2.535, 0),
            red,
            bevel = 0.012
        );
        for s in [-1, 1] {
            pediment(&f, &format!("pediment_z{s}"), s, true, red, red);
            pediment(&f, &format!("pediment_x{s}"), s, false, red, red);
        }
        loft!(
            "dome",
            &[
                ring!(0.44, 0.44, z = 2.555, verts = 32, square = 5),
                ring!(0.4, 0.4, z = 2.6, verts = 32, square = 4),
                ring!(0.3, 0.3, z = 2.632, verts = 32, square = 3),
                ring!(0.16, 0.16, z = 2.652, verts = 32, square = 2.5),
                ring!(0.001, 0.001, z = 2.66, verts = 32)
            ],
            material = red
        );
        fsphere!(
            f,
            "crown",
            0.04,
            (0, 2.66, 0),
            red,
            segments = 10,
            rings = 6
        );
        // the back, the left and the right: a panelled kick panel below each window; the back's window frames the phone
        for (k, (rot, ox, oz)) in [(PI, 0, -1), (PI / 2.0, 1, 0), (-PI / 2.0, -1, 0)]
            .into_iter()
            .enumerate()
        {
            // a side's (across, height, out) point: on the face at (ox, oz) * FACE
            let at = move |x: N, y: f64, d: N| -> (N, f64, N) {
                if oz != 0 {
                    return (x * -oz, y, N::F(oz as f64 * (FACE + d.f())));
                }
                (N::F(0.0 + ox as f64 * (FACE + d.f())), y, x * ox)
            };
            glazing(&f, &format!("side{k}"), m.glass, red, &at, rot);
            let (kx, _, kz) = at(N::I(0), 0.0, N::I(0));
            let side = oz != 0;
            fbox!(
                f,
                format!("kick{k}"),
                if side { 0.68 } else { 0.04 },
                0.46,
                if side { 0.04 } else { 0.68 },
                (kx, 0.35, kz),
                red,
                bevel = 0.006
            );
            let (px, _, pz) = at(N::I(0), 0.0, N::F(0.024));
            fbox!(
                f,
                format!("kick_panel{k}"),
                if side { 0.5 } else { 0.012 },
                0.3,
                if side { 0.012 } else { 0.5 },
                (px, 0.35, pz),
                red,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
        for (k, (x, z, w, d)) in [
            (N::I(0), N::F(0.47), 0.6, 0.012),
            (N::I(0), N::F(-0.47), 0.6, 0.012),
            (N::F(0.47), N::I(0), 0.012, 0.6),
            (N::F(-0.47), N::I(0), 0.012, 0.6),
        ]
        .into_iter()
        .enumerate()
        {
            fbox!(f, format!("sign{k}"), w, 0.1, d, (x, 2.38, z), m.sign);
        }
        // the phone on a panel at the back: its case, a dial, the hook for the handset, the coin box under it, a
        // coiled cord; a shelf of directories under it
        fbox!(
            f,
            "phone_panel",
            0.4,
            0.5,
            0.02,
            (0, 1.35, -0.39),
            red,
            bevel = 0.006
        );
        fbox!(
            f,
            "phone",
            0.16,
            0.2,
            0.09,
            (0, 1.42, -0.335),
            m.dark,
            bevel = 0.012
        );
        fcyl!(
            f,
            "dial",
            0.04,
            0.01,
            (0, 1.44, -0.287),
            m.chrome,
            verts = 12,
            rot = (90, 0, 0)
        );
        fbox!(
            f,
            "coin_box",
            0.14,
            0.12,
            0.08,
            (0, 1.26, -0.34),
            m.dark,
            bevel = 0.008
        );
        fbox!(
            f,
            "coin_slot",
            0.04,
            0.006,
            0.01,
            (0, 1.29, -0.299),
            m.chrome
        );
        // (The handset on its hook is drawn by code, in the caller's hand while they call: the held things'
        // `receiver`, actions.ts `REST_PROPS`. A part of its own here would be a fifth draw.)
        fsweep!(
            f,
            "cord",
            &[
                (-0.1, 1.33, -0.32),
                (-0.08, 1.25, -0.33),
                (-0.06, 1.2, -0.34),
                (-0.07, 1.26, -0.35)
            ],
            0.006,
            m.dark,
            verts = 5,
            resolution = 3
        );
        fbox!(
            f,
            "shelf",
            0.4,
            0.03,
            0.18,
            (0, 1.06, -0.3),
            m.dark,
            bevel = 0.006
        );
        for (k, (x, w, book)) in [(-0.1, 0.05, m.paper), (-0.045, 0.04, m.blue)]
            .into_iter()
            .enumerate()
        {
            fbox!(
                f,
                format!("directory{k}"),
                w,
                0.24,
                0.16,
                (x, 1.195, -0.3),
                book,
                bevel = 0.004,
                bevel_segments = 1
            );
        }
    });
    join(&still, "phoneBooth", Some((0, 0, 0).into()));
    // the door: its frame, panelled kick panel, window and handle, hinged on its edge at the game's -x (on the right
    // as you face it in the game)
    let door = new_parts(|| {
        fbox!(
            f,
            "door_kick",
            DOOR_W,
            0.46,
            0.04,
            (0, 0.35, FACE),
            red,
            bevel = 0.006
        );
        fbox!(
            f,
            "door_kick_panel",
            0.5,
            0.3,
            0.012,
            (0, 0.35, FACE + 0.024),
            red,
            bevel = 0.004,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            fbox!(
                f,
                format!("door_stile{s}"),
                0.04,
                1.62,
                0.04,
                (s as f64 * (DOOR_W / 2.0 - 0.02), 1.37, FACE),
                red,
                bevel = 0.006
            );
        }
        fbox!(
            f,
            "door_head",
            DOOR_W,
            0.06,
            0.04,
            (0, 2.17, FACE),
            red,
            bevel = 0.006
        );
        glazing(
            &f,
            "door",
            m.pane,
            red,
            &|x: N, y: f64, d: N| (x, y, N::F(FACE + d.f())),
            0.0,
        );
        kit_pull!(
            f,
            "door_handle",
            DOOR_W / 2.0 - 0.06,
            1.15,
            FACE + 0.02,
            0.16,
            m.chrome,
            r = 0.008,
            out = 0.035,
            upright = true
        );
    });
    let door = hinged(join(&door, "door", None), f.at(-DOOR_W / 2.0, 1.0, FACE));
    settle_all();
    // Out, about its hinge, to stand square to the front, as a real kiosk's: the caller stands back from the tile in
    // front while it opens, steps inside and it shuts behind them (and it opens again to let them out: the motions'
    // `out`; `STEP_IN` in characters.ts times their steps to it). Opening in, it swept through them inside.
    swing(
        door,
        "door_open",
        90.0,
        Swing {
            seconds: 0.8,
            ease: Ease::BackOut,
            ..Swing::default()
        },
    );
    meshes()
}

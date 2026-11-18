//! H1: the digital piano. A home console of 88 full-size keys (52 white at 2.35 cm) on two side panels with a pedal
//! board of three pedals, a control strip and a music rest behind the keys, and a padded X-legged bench behind it,
//! built facing the front and turned round to stand against the wall like the upright, the player facing it (content
//! `seat.facing`). The keys' tops are at 0.745 m, as the upright's (0.72–0.76; review M-hobbies-04: they were at 0.845 on
//! a three-octave keyboard on an X-stand). Still.

use super::M;
use super::common::{P, flat_quads, sheet_music, turn_round};
use artkit::forms::smooth_outline;
use artkit::json::float_repr as r;
use artkit::kit::{self, fillet};
use artkit::math::{PI, cos, sin};
use artkit::parts::{Settle, join_all, settle_with};
use artkit::place::{Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, soft};

/// A white key's width.
const KB_KEY: f64 = 0.0235;
/// The white keys' top.
const KB_TOP: f64 = 0.745;
/// The keys' middle (depth), as built (before it is turned round).
const KB_KEYS_Z: f64 = 0.17;
/// Turned round, how far forward it stands: its back on the wall's face.
pub const KB_FORWARD: f64 = -0.02;
/// The keyboard's width, 1.222 m.
const KB_W: f64 = 52.0 * KB_KEY;

/// Game x of each black key of an 88-key keyboard (A0 to C8): after A0, then C, D, F, G and A of each octave from C1.
fn kb_black_keys() -> Vec<f64> {
    let x0 = -KB_W / 2.0;
    let mut edges = vec![1];
    edges.extend((0..7).flat_map(|o| [1, 2, 4, 5, 6].map(move |n| 2 + 7 * o + n)));
    // A0 at the player's left: game -x as built, facing the front (+z)
    edges.iter().map(|&e| x0 + KB_KEY * e as f64).collect()
}

pub fn keyboard_digital() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    // the side panels' middles
    let half = KB_W / 2.0 + 0.06;
    // (built facing the front, the player's side -z here; turned round at the end)
    // the keys, the key slip in front and the keybed under them
    fbox!(
        f,
        "keys",
        KB_W,
        0.022,
        0.15,
        (0, KB_TOP - 0.011, KB_KEYS_Z),
        M.ivory
    );
    let gaps: Vec<(f64, f64, f64, f64, f64)> = (1..52)
        .map(|k| {
            let x = -KB_W / 2.0 + KB_KEY * k as f64;
            (
                x - 0.00075,
                x + 0.00075,
                KB_KEYS_Z - 0.074,
                KB_KEYS_Z + 0.074,
                KB_TOP + 0.0003,
            )
        })
        .collect();
    flat_quads(&f, "gaps", &gaps, M.key_gap);
    for (k, x) in kb_black_keys().into_iter().enumerate() {
        fbox!(
            f,
            format!("black{k}"),
            0.012,
            0.012,
            0.09,
            (x, KB_TOP + 0.006, KB_KEYS_Z + 0.03),
            M.ebony
        );
    }
    fbox!(
        f,
        "keybed",
        KB_W + 0.02,
        0.05,
        0.31,
        (0, KB_TOP - 0.047, 0.245),
        M.kb_body,
        bevel = 0.006
    );
    fsweep!(
        f,
        "key_slip",
        &[
            (-KB_W / 2.0 - 0.01, KB_TOP - 0.02, KB_KEYS_Z - 0.081),
            (KB_W / 2.0 + 0.01, KB_TOP - 0.02, KB_KEYS_Z - 0.081)
        ],
        0,
        M.kb_body,
        profile = vec![
            (-0.015, -0.006),
            (0.015, -0.006),
            (0.015, 0.002),
            (0.011, 0.006),
            (-0.015, 0.006)
        ]
    );
    fbox!(
        f,
        "key_felt",
        KB_W,
        0.008,
        0.01,
        (0, KB_TOP + 0.002, KB_KEYS_Z + 0.08),
        M.ball_red
    );
    // the case behind the keys: a rounded top with its control strip, buttons, a screen, two speaker grilles
    fsweep!(
        f,
        "case_top",
        &[
            (-KB_W / 2.0 - 0.01, KB_TOP + 0.03, 0.335),
            (KB_W / 2.0 + 0.01, KB_TOP + 0.03, 0.335)
        ],
        0,
        M.kb_body,
        profile = vec![
            (-0.03, -0.065),
            (-0.03, 0.065),
            (0.02, 0.065),
            (0.03, 0.05),
            (0.03, -0.035),
            (0.022, -0.06),
            (0.008, -0.068)
        ]
    );
    fbox!(
        f,
        "strip",
        KB_W - 0.1,
        0.004,
        0.05,
        (0, KB_TOP + 0.062, 0.3),
        M.kb_trim,
        bevel = 0.0015,
        bevel_segments = 1
    );
    fbox!(
        f,
        "lcd",
        0.12,
        0.004,
        0.03,
        (0.2, KB_TOP + 0.065, 0.3),
        M.lcd,
        bevel = 0.0015,
        bevel_segments = 1
    );
    for k in 0..8 {
        fbox!(
            f,
            format!("button{k}"),
            0.018,
            0.006,
            0.012,
            (-0.45 + 0.03 * k as f64, KB_TOP + 0.066, 0.3),
            M.kb_body
        );
    }
    fbox!(
        f,
        "led",
        0.006,
        0.003,
        0.006,
        (0.29, KB_TOP + 0.065, 0.3),
        M.ball_red
    );
    for s in [-1, 1] {
        let sf = s as f64;
        fcyl!(
            f,
            format!("knob{s}"),
            0.01,
            0.012,
            (sf * 0.52, KB_TOP + 0.068, 0.3),
            M.kb_trim,
            verts = 12,
            bevel = 0.003,
            bevel_segments = 1
        );
        soft!(
            f,
            format!("speaker{s}"),
            0.14,
            0.004,
            0.035,
            (sf * 0.4, KB_TOP + 0.062, 0.345),
            M.kb_seat,
            round = 0.002,
            round_segments = 1
        );
    }
    // the music rest, leant back, with a ledge, and a sheet on it
    let rest = [
        (0.35, -0.12),
        (0.35, 0.1),
        (0.33, 0.12),
        (-0.33, 0.12),
        (-0.35, 0.1),
        (-0.35, -0.12),
    ];
    fextrude!(
        f,
        "rest",
        &rest,
        0.012,
        (0, KB_TOP + 0.18, 0.37),
        M.kb_body,
        rot = (12, 0, 0),
        bevel = 0.003,
        bevel_segments = 1
    );
    fsweep!(
        f,
        "rest_ledge",
        &[
            (-0.35, KB_TOP + 0.066, 0.355),
            (0.35, KB_TOP + 0.066, 0.355)
        ],
        0,
        M.kb_body,
        profile = vec![
            (-0.006, -0.015),
            (0.006, -0.015),
            (0.008, -0.01),
            (0.008, 0.015),
            (-0.006, 0.015)
        ]
    );
    sheet_music(
        &f,
        "music",
        0.4,
        0.2,
        (0.0, KB_TOP + 0.19, 0.362),
        (12.0, 0.0, 0.0),
        0.002,
    );
    // the side panels, cut back under the keys, with their key cheeks and feet; the back panel; the pedal board
    let side = smooth_outline(
        &[
            (0.40, 0.0),
            (0.40, 0.805),
            (0.15, 0.805),
            (0.135, 0.79),
            (0.135, 0.7),
            (0.16, 0.66),
            (0.2, 0.55),
            (0.21, 0.3),
            (0.2, 0.1),
            (0.19, 0.03),
            (0.19, 0.0),
        ],
        1,
        true,
    );
    let cheek = [
        (0.37, 0.8),
        (0.37, 0.84),
        (0.13, 0.84),
        (0.105, 0.832),
        (0.095, 0.815),
        (0.1, 0.795),
        (0.12, 0.79),
        (0.15, 0.8),
    ];
    for s in [-1, 1] {
        let sf = s as f64;
        fextrude!(
            f,
            format!("side{s}"),
            &side,
            0.04,
            (sf * half, 0, 0),
            M.kb_body,
            plane = Facing::Side,
            bevel = 0.006,
            bevel_segments = 1
        );
        fextrude!(
            f,
            format!("cheek{s}"),
            &smooth_outline(&cheek, 1, true),
            0.04,
            (sf * half, 0, 0),
            M.kb_body,
            plane = Facing::Side,
            bevel = 0.008,
            bevel_segments = 2
        );
        fextrude!(
            f,
            format!("toe{s}"),
            &[
                (0.41, 0.0),
                (0.41, 0.03),
                (0.06, 0.03),
                (0.02, 0.02),
                (0.01, 0.0)
            ],
            0.05,
            (sf * half, 0, 0),
            M.kb_trim,
            plane = Facing::Side,
            bevel = 0.005,
            bevel_segments = 1
        );
    }
    fbox!(
        f,
        "back",
        KB_W + 0.08,
        0.5,
        0.015,
        (0, 0.38, 0.39),
        M.kb_body
    );
    fbox!(
        f,
        "brace",
        KB_W + 0.05,
        0.06,
        0.02,
        (0, 0.6, 0.38),
        M.kb_trim,
        bevel = 0.004,
        bevel_segments = 1
    );
    fbox!(
        f,
        "pedal_board",
        0.36,
        0.09,
        0.12,
        (0, 0.065, 0.33),
        M.kb_body,
        bevel = 0.02,
        bevel_segments = 2
    );
    kit::tube!(
        f,
        "pedal_rod",
        &[(0, 0.11, 0.38), (0, KB_TOP - 0.07, 0.38)],
        0.008,
        M.kb_trim,
        verts = 8,
        round = 1
    );
    for x in [-0.07, 0.0, 0.07] {
        fsweep!(
            f,
            format!("pedal{}", r(x)),
            &[
                (x, 0.05, 0.3),
                (x, 0.049, 0.26),
                (x, 0.044, 0.222),
                (x, 0.04, 0.212)
            ],
            0,
            M.brass,
            profile = super::common::rr(0.008, 0.024, 0.003, 1),
            resolution = 2,
            scale = [0.8, 0.9, 1.15, 1.1]
        );
    }
    // headphones hung on a hook under the keybed, at the player's left
    kit::tube!(
        f,
        "hook",
        &fillet(
            &[
                (-0.5, KB_TOP - 0.072, 0.15),
                (-0.5, KB_TOP - 0.1, 0.15),
                (-0.5, KB_TOP - 0.1, 0.12)
            ],
            0.01,
            3
        ),
        0.003,
        M.kb_trim,
        verts = 6,
        round = 1
    );
    let band: Vec<P> = (0..11)
        .map(|i| {
            (
                -0.5 + 0.075 * cos(PI * i as f64 / 10.0),
                KB_TOP - 0.175 + 0.075 * sin(PI * i as f64 / 10.0),
                0.12,
            )
        })
        .collect();
    kit::tube!(f, "headband", &band, 0.008, M.ebony, verts = 6, round = 1);
    for s in [-1, 1] {
        fcyl!(
            f,
            format!("earcup{s}"),
            0.035,
            0.03,
            (-0.5 + s as f64 * 0.075, KB_TOP - 0.185, 0.12),
            M.ebony,
            rot = (0, 90, 0),
            verts = 14,
            bevel = 0.008,
            bevel_segments = 1
        );
    }
    // the bench, its top at 0.48 m (content `seat`; a piano bench is 0.46–0.56): a padded seat on X legs
    soft!(
        f,
        "seat",
        0.66,
        0.05,
        0.32,
        (0, 0.455, -0.2),
        M.kb_seat,
        round = 0.02,
        puff = 0.008,
        flat = 2,
        round_segments = 2,
        piping = 0.004
    );
    fbox!(
        f,
        "seat_frame",
        0.62,
        0.02,
        0.26,
        (0, 0.42, -0.2),
        M.stand,
        bevel = 0.004,
        bevel_segments = 1
    );
    for x in [-0.28, 0.28] {
        for (z0, z1) in [(-0.32, -0.08), (-0.08, -0.32)] {
            kit::tube!(
                f,
                format!("bench_leg{}{}", r(x), r(z0)),
                &[(x, 0.016, z0), (x, 0.41, z1)],
                0.013,
                M.stand,
                verts = 8,
                round = 1
            );
        }
        fcyl!(
            f,
            format!("bench_pivot{}", r(x)),
            0.014,
            0.035,
            (x, 0.213, -0.2),
            M.stand,
            rot = (0, 90, 0),
            verts = 10
        );
        for z in [-0.32, -0.08] {
            fbox!(
                f,
                format!("bench_cap{}{}", r(x), r(z)),
                0.035,
                0.016,
                0.05,
                (x, 0.008, z),
                M.rubber,
                bevel = 0.005
            );
        }
    }
    kit::tube!(
        f,
        "bench_bar",
        &[(-0.28, 0.213, -0.2), (0.28, 0.213, -0.2)],
        0.009,
        M.stand,
        verts = 8,
        round = 1
    );
    join_all("keyboardDigital");
    // built facing the front; it stands with its back on the wall's face
    turn_round(&meshes(), KB_FORWARD);
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    meshes()
}

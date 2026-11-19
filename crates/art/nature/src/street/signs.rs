//! The street-name sign, the taxi rank's sign and the car park's P sign.

use super::common::{_band, _box, _lettering, _post, _post_turned};
use super::{F, M};
use artkit::far::{is_low, low};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::kit::{Chequer, TaxiWord, chequer, taxi_word};
use artkit::parts::{Settle, join_all, settle_all, settle_with};
use artkit::place::Facing;
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{fextrude, revolve};
use std::f64::consts::{FRAC_PI_2, PI};

pub fn sign() -> Vec<Obj> {
    let mut rng = Random::new(7);
    let full = !is_low();
    _post(&F, 2.585, 0.075, 0.035);
    revolve!(
        "finial",
        &[
            (0.0, 2.58),
            (0.042, 2.58),
            (0.042, 2.592),
            (0.03, 2.6),
            (0.046, 2.625),
            (0.038, 2.648),
            (0.016, 2.662),
            (0.0, 2.665)
        ],
        material = M.pole,
        segments = low(12, 6)
    );
    for (y, along_x) in [(2.45, true), (2.25, false)] {
        // the blade: rounded at its corners, a white panel set into each face, held by bands above and below
        let outline = rounded_rect(0.9, 0.18, low(0.03, 0.0), low(3, 1));
        let panel = rounded_rect(0.84, 0.12, low(0.018, 0.0), low(3, 1));
        let plane = if along_x { Facing::Front } else { Facing::Side };
        let name = float_repr(y);
        fextrude!(
            F,
            format!("blade{name}"),
            &outline,
            0.03,
            (0, y, 0),
            M.sign,
            plane = plane
        );
        fextrude!(
            F,
            format!("panel{name}"),
            &panel,
            0.032,
            (0, y, 0),
            M.plate,
            plane = plane
        );
        for dy in [-0.1, 0.1] {
            _band(
                &F,
                format!("band{name}{}", float_repr(dy)),
                y + dy,
                0.043,
                0.03,
            );
        }
        if full {
            for face in [1, -1] {
                _lettering(
                    &F,
                    &format!("letter{name}{face}"),
                    0.62,
                    y,
                    along_x,
                    face,
                    &mut rng,
                );
            }
        }
    }
    join_all("sign");
    settle_all();
    meshes()
}

// --- the taxi rank's sign: a lit TAXI box on a post, the taxi's chequers, and the fares --------------------------
pub fn taxi_sign() -> Vec<Obj> {
    let full = !is_low();
    // the post up into the sign's case
    _post(&F, 2.3, 0.0375, 0.035);
    _band(&F, "collar", 2.18, 0.037, 0.04);
    // the sign: a yellow case, a light box lettered TAXI on both faces (lit at night), the chequers under it
    _box(&F, "case", 0.5, 0.3, 0.07, (0, 2.42, 0), M.taxi, 0.012);
    _box(
        &F,
        "light",
        0.44,
        0.15,
        0.076,
        (0, 2.475, 0),
        M.sign_light,
        0.004,
    );
    if full {
        for face in [1, -1] {
            let side = face as f64;
            taxi_word(
                &F,
                format!("word{face}"),
                (0, 2.475),
                0.095,
                side * 0.038,
                face,
                M.black,
                TaxiWord::default(),
            );
            chequer(
                &F,
                format!("check{face}"),
                -0.21,
                0.21,
                2.385,
                side * 0.035,
                0.03,
                face,
                M.black,
                Chequer::default(),
            );
        }
    } else {
        for face in [1, -1] {
            _box(
                &F,
                format!("band{face}"),
                0.42,
                0.06,
                0.003,
                (0, 2.355, face as f64 * 0.0365),
                M.black,
                0.0,
            );
        }
    }
    // the fares in their case, clamped to the post's front: a framed case under a little hood, a yellow head
    _box(
        &F,
        "case_back",
        0.29,
        0.41,
        0.02,
        (0, 1.5, 0.045),
        M.pole,
        0.004,
    );
    let frame = rounded_rect(0.3, 0.42, low(0.012, 0.0), low(2, 1));
    fextrude!(
        F,
        "fares_case",
        &frame,
        0.012,
        (0, 1.5, 0.061),
        M.pole,
        holes = [rounded_rect(0.264, 0.384, 0.004, 1)]
    );
    _box(
        &F,
        "hood",
        0.32,
        0.014,
        0.044,
        (0, 1.718, 0.0482),
        M.pole,
        0.004,
    );
    _box(
        &F,
        "fares",
        0.27,
        0.39,
        0.004,
        (0, 1.5, 0.057),
        M.plate,
        0.0,
    );
    _box(
        &F,
        "header",
        0.26,
        0.06,
        0.004,
        (0, 1.66, 0.06),
        M.taxi,
        0.0,
    );
    if full {
        taxi_word(
            &F,
            "header_word",
            (0, 1.66),
            0.035,
            0.062,
            1,
            M.black,
            TaxiWord::default(),
        );
        for k in 0..9 {
            let w = if k % 4 != 3 {
                0.16 - (k % 3) as f64 * 0.03
            } else {
                0.07
            };
            let y = 1.6 - k as f64 * 0.03;
            _box(
                &F,
                format!("line{k}"),
                w,
                0.01,
                0.003,
                (-0.11 + w / 2.0, y, 0.06),
                M.ink,
                0.0,
            );
            // the fares, at the right
            _box(
                &F,
                format!("fare{k}"),
                0.05,
                0.01,
                0.003,
                (0.085, y, 0.06),
                M.ink,
                0.0,
            );
        }
    }
    for y in [1.36, 1.64] {
        _band(&F, format!("clamp{}", float_repr(y)), y, 0.037, 0.03);
        _box(
            &F,
            format!("lug{}", float_repr(y)),
            0.05,
            0.03,
            0.03,
            (0, y, 0.03),
            M.pole,
            0.0,
        );
    }
    join_all("taxiSign");
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    meshes()
}

// --- the car park's P sign: a white P on a blue panel, white-bordered, on a plate clamped to a post ----------------
/// An outline in a face's plane (x, y).
type Outline = Vec<(f64, f64)>;

/// The P (game metres from the panel's middle, x right and y up): its stem, and its bowl round to the right of it, the
/// bowl's hole cut through. Its curves in `segments` steps a half turn.
fn letter_p(segments: usize) -> (Outline, Outline) {
    let (stem, top, foot, waist) = ((-0.085, -0.03), 0.14, -0.14, -0.005);
    let (cx, cy) = (0.03, (top + waist) / 2.0);
    let (outer, inner) = ((top - waist) / 2.0, (top - waist) / 2.0 - 0.052);
    let arc = |r: f64| -> Outline {
        (0..=segments)
            .map(|k| {
                let a = -FRAC_PI_2 + PI * k as f64 / segments as f64;
                (cx + r * a.cos(), cy + r * a.sin())
            })
            .collect()
    };
    let mut outline = vec![(stem.0, foot), (stem.1, foot), (stem.1, waist)];
    outline.extend(arc(outer));
    outline.push((stem.0, top));
    let mut hole = vec![(stem.1, cy - inner)];
    hole.extend(arc(inner));
    hole.push((stem.1, cy + inner));
    // (The hole the other way round: a hole in the outline. Both mirrored: an outline on the front is drawn as seen
    // from behind it, `GameFrame::extrude`.)
    hole.reverse();
    let mirror = |v: Outline| v.into_iter().map(|(x, y)| (-x, y)).collect();
    (mirror(outline), mirror(hole))
}

pub fn parking_sign() -> Vec<Obj> {
    let full = !is_low();
    // the post, capped, and the plate clamped to its front at the height a driver's eye finds it
    _post_turned(&F, 2.4, 0.075, 0.035, 8);
    _band(&F, "cap", 2.405, 0.04, 0.012);
    let y = 2.15;
    for dy in [-0.15, 0.15] {
        let name = float_repr(y + dy);
        if full {
            _band(&F, format!("clamp{name}"), y + dy, 0.037, 0.03);
        }
        _box(
            &F,
            format!("lug{name}"),
            0.05,
            0.03,
            0.016,
            (0, y + dy, 0.043),
            M.pole,
            0.0,
        );
    }
    // the plate: grey metal behind, white at its front with the blue panel set in
    _box(&F, "plate", 0.46, 0.46, 0.012, (0, y, 0.057), M.pole, 0.004);
    _box(&F, "border", 0.45, 0.45, 0.002, (0, y, 0.064), M.plate, 0.0);
    let panel = rounded_rect(0.4, 0.4, low(0.02, 0.0), low(3, 1));
    fextrude!(
        F,
        "panel",
        &panel,
        0.002,
        (0, y, 0.066),
        M.sign_blue,
        plane = Facing::Front
    );
    let (outline, hole) = letter_p(low(10, 4));
    fextrude!(
        F,
        "letter",
        &outline,
        0.002,
        (0, y, 0.068),
        M.plate,
        plane = Facing::Front,
        holes = [hole]
    );
    join_all("parkingSign");
    settle_with(
        &meshes(),
        Settle {
            centre: false,
            floor: true,
        },
    );
    meshes()
}

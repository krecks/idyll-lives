//! K7 continued: the tables on a pedestal: the round bistro table, the high café table and the round table for four.

use super::common::{finish, ring_loop, round_slab, turned};
use super::{BT, F, M, TT};
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{cosd, sind};
use artkit::scene::Obj;
use artkit::{fsweep, kit};

/// A round top on a column: a hub under the top, the column with a ring near its top, a domed cast foot (5 cm
/// high), the top 4 cm thick and 40-sided.
#[allow(clippy::too_many_arguments)]
pub(super) fn pedestal(
    h: f64,
    top_r: f64,
    top: Mat,
    base: Mat,
    hub_r: f64,
    col_r: f64,
    foot_r: f64,
) {
    let (foot_h, t, verts) = (0.05, 0.04, 40);
    round_slab("top", top_r, h, t, top, verts, None, (0.0, 0.0));
    turned(
        "hub",
        &[
            (0.0, 0.0),
            (hub_r * 0.6, 0.0),
            (hub_r, 0.014),
            (hub_r, 0.03),
            (0.0, 0.03),
        ],
        (0, h - t - 0.03, 0),
        base,
        18,
        35.0,
    );
    let ch = h - t - 0.03 - foot_h + 0.01;
    turned(
        "column",
        &[
            (0.0, 0.0),
            (col_r * 1.4, 0.0),
            (col_r, 0.04),
            (col_r, ch - 0.08),
            (col_r * 1.35, ch - 0.06),
            (col_r * 1.35, ch - 0.045),
            (col_r, ch - 0.03),
            (col_r, ch),
            (0.0, ch),
        ],
        (0, foot_h - 0.01, 0),
        base,
        14,
        35.0,
    );
    turned(
        "foot",
        &[
            (0.0, 0.0),
            (foot_r - 0.004, 0.0),
            (foot_r, 0.005),
            (foot_r - 0.003, 0.013),
            (foot_r * 0.6, foot_h * 0.5),
            (col_r * 2.2, foot_h * 0.9),
            (col_r * 1.6, foot_h),
            (0.0, foot_h),
        ],
        (0, 0, 0),
        base,
        32,
        35.0,
    );
}

/// Round bistro table (catalog2): a white gloss top with a black edge band, on a black column and cast foot.
pub fn table_bistro_round() -> Vec<Obj> {
    pedestal(TT, 0.355, M.top_white, M.legs_black, 0.1, 0.03, 0.25);
    turned(
        "edge",
        &[
            (0.35, 0.0),
            (0.358, 0.0),
            (0.36, 0.004),
            (0.36, 0.026),
            (0.358, 0.03),
            (0.35, 0.03),
        ],
        (0, TT - 0.034, 0),
        M.legs_black,
        48,
        35.0,
    );
    finish("tableBistroRound")
}

/// High café table (catalog2): a dark top at bar height on a column, a foot ring on four spokes for the stools.
pub fn table_bar_high() -> Vec<Obj> {
    pedestal(BT, 0.35, M.top_black, M.legs_black, 0.1, 0.035, 0.25);
    ring_loop(
        "foot_ring",
        0.185,
        0.185,
        0.3,
        0.013,
        M.legs_black,
        0.0,
        0.0,
        8,
        32,
    );
    turned(
        "ring_hub",
        &[
            (0.0, 0.0),
            (0.05, 0.0),
            (0.05, 0.027),
            (0.047, 0.03),
            (0.0, 0.03),
        ], // its top eased
        (0, 0.285, 0),
        M.legs_black,
        14,
        35.0,
    );
    for k in 0..4 {
        let a = f64::from(45 + 90 * k);
        let (c, s) = (cosd(a), sind(a));
        kit::tube!(
            F,
            format!("spoke{k}"),
            &[(c * 0.05, 0.3, s * 0.05), (c * 0.18, 0.3, s * 0.18)],
            0.009,
            M.legs_black,
            verts = 8
        );
    }
    finish("tableBarHigh")
}

/// Round pedestal table (catalog2, 2x2): a birch top with a moulded edge on a turned column, four feet curving
/// out from it to turned toes.
pub fn table_round4() -> Vec<Obj> {
    round_slab("top", 0.76, TT, 0.04, M.top_birch, 64, None, (0.0, 0.0));
    turned(
        "apron",
        &[
            (0.0, 0.0),
            (0.28, 0.0),
            (0.3, 0.01),
            (0.3, 0.05),
            (0.285, 0.06),
            (0.0, 0.06),
        ],
        (0, TT - 0.1, 0),
        M.legs_birch,
        32,
        35.0,
    );
    turned(
        "column",
        &[
            (0.0, 0.08),
            (0.13, 0.08),
            (0.13, 0.12),
            (0.1, 0.16),
            (0.075, 0.3),
            (0.09, 0.42),
            (0.08, 0.5),
            (0.065, 0.58),
            (0.072, 0.6),
            (0.065, 0.62),
            (0.09, 0.64),
            (0.09, 0.67),
            (0.0, 0.67),
        ],
        (0, 0, 0),
        M.legs_birch,
        24,
        35.0,
    );
    for k in 0..4 {
        let a = f64::from(45 + 90 * k);
        let (c, s) = (cosd(a), sind(a));
        fsweep!(
            F,
            format!("foot{k}"),
            &[
                (c * 0.06, 0.1, s * 0.06),
                (c * 0.17, 0.085, s * 0.17),
                (c * 0.28, 0.05, s * 0.28),
                (c * 0.36, 0.03, s * 0.36)
            ],
            0.0,
            M.legs_birch,
            profile = rounded_rect(0.07, 0.06, 0.015, 2),
            up = (0, 0, 1),
            resolution = 3,
            scale = [1.1, 1.0, 0.82, 0.7],
            angle = 60
        );
        turned(
            &format!("toe{k}"),
            &[
                (0.0, 0.0),
                (0.026, 0.0),
                (0.03, 0.008),
                (0.026, 0.02),
                (0.02, 0.03),
                (0.0, 0.03),
            ],
            (c * 0.36, 0, s * 0.36),
            M.legs_birch,
            12,
            35.0,
        );
    }
    finish("tableRound4")
}

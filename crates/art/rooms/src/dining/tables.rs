//! K7: dining tables.

use super::common::{apron, board, finish, slab, sq_leg, turned};
use super::{F, M, TT};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::{PI, sin};
use artkit::place::Facing;
use artkit::scene::Obj;
use artkit::{fbox, fextrude, fsweep, revolve};

/// A turned table leg's profile `h` tall: a pad foot, a long taper from `r_foot` to `r_top`, a ring under the
/// apron, the block that takes the apron; `collar`: a ring that high.
pub fn turned_leg(h: f64, r_top: f64, r_foot: f64, collar: Option<f64>) -> Vec<(f64, f64)> {
    let mut p = vec![
        (0.0, 0.0),
        (r_foot * 0.92, 0.0),
        (r_foot * 1.06, 0.008),
        (r_foot, 0.022),
        (r_foot * 0.9, 0.035),
    ];
    if let Some(c) = collar {
        let rc = r_foot + (r_top - r_foot) * c / h;
        p.extend([
            (rc * 0.95, c - 0.025),
            (rc * 1.28, c - 0.01),
            (rc * 1.3, c),
            (rc * 1.28, c + 0.01),
            (rc * 0.95, c + 0.025),
        ]);
    }
    p.extend([
        (r_top * 0.92, h - 0.14),
        (r_top * 1.15, h - 0.125),
        (r_top * 1.15, h - 0.112),
        (r_top, h - 0.1),
        (r_top, h),
        (0.0, h),
    ]);
    p
}

/// `rect_table`'s options.
struct RectTable {
    t: f64,
    leg: f64,
    inset: f64,
    turned: bool,
    taper: f64,
    apron: bool,
    h: f64,
    arch: f64,
    collar: Option<f64>,
    r: f64,
    end_arch: Option<f64>,
}

impl Default for RectTable {
    fn default() -> RectTable {
        RectTable {
            t: 0.045,
            leg: 0.06,
            inset: 0.06,
            turned: false,
            taper: 0.7,
            apron: true,
            h: TT,
            arch: 0.02,
            collar: None,
            r: 0.02,
            end_arch: None,
        }
    }
}

// --- the catalog2 tables: a top at 0.75 on four (or six) legs, aprons under it -------------------------------------
/// A top `w` by `d`, `t` thick with a moulded edge, on four legs (six past 2.6 m), square tapered or `turned`
/// (with a `collar`), joined by aprons arched by `arch` between them.
fn rect_table(w: f64, d: f64, top: Mat, legs: Mat, o: RectTable) -> (f64, f64) {
    let RectTable {
        t,
        leg,
        inset,
        turned: is_turned,
        taper,
        apron: has_apron,
        h,
        arch,
        collar,
        r,
        end_arch,
    } = o;
    slab("top", w, d, h, t, top, r, None, 3, (0.0, 0.0));
    let (lx, lz) = (w / 2.0 - inset - leg / 2.0, d / 2.0 - inset - leg / 2.0);
    if has_apron {
        for s in [-1.0, 1.0] {
            apron(
                &format!("apron{s}"),
                2.0 * lx - leg * 0.8,
                0.08,
                0.022,
                (0.0, h - t, s * (lz - leg / 2.0 + 0.011)),
                legs,
                Facing::Front,
                if 2.0 * lx < 2.0 { arch } else { 0.0 },
                10,
                0.003,
            );
            apron(
                &format!("apron_end{s}"),
                2.0 * lz - leg * 0.8,
                0.08,
                0.022,
                (s * (lx - leg / 2.0 + 0.011), h - t, 0.0),
                legs,
                Facing::Side,
                end_arch.unwrap_or(arch),
                10,
                0.003,
            );
        }
    }
    let mut xs = vec![-lx, lx];
    if w > 2.6 {
        xs.push(0.0);
    }
    for &x in &xs {
        for z in [-lz, lz] {
            let name = format!("leg{}{}", float_repr(x), float_repr(z));
            if is_turned {
                turned(
                    &name,
                    &turned_leg(h - t, leg / 2.0, leg / 2.0 * taper, collar),
                    (x, 0.0, z),
                    legs,
                    12,
                    35.0,
                );
            } else {
                sq_leg(
                    &name,
                    x,
                    z,
                    0.0,
                    h - t,
                    leg * taper,
                    leg,
                    legs,
                    0.006,
                    0.003,
                    None,
                );
            }
        }
    }
    if has_apron && w > 2.6 {
        // the middle legs' aprons across
        apron(
            "apron_mid",
            2.0 * lz - leg * 0.8,
            0.08,
            0.022,
            (0.0, h - t, 0.0),
            legs,
            Facing::Side,
            arch,
            10,
            0.003,
        );
    }
    (lx, lz)
}

/// Square table for two (catalog2): an oak top with a moulded edge on turned darker legs with collars, arched
/// aprons.
pub fn table_square2() -> Vec<Obj> {
    rect_table(
        0.82,
        0.82,
        M.top_oak,
        M.legs_oakd,
        RectTable {
            turned: true,
            taper: 0.75,
            collar: Some(0.52),
            ..RectTable::default()
        },
    );
    finish("tableSquare2")
}

/// Dining table for four (catalog2): an oak top on square tapered oak legs, arched aprons, a drawer at each end
/// with a turned knob.
pub fn table_dining4() -> Vec<Obj> {
    let (lx, _lz) = rect_table(
        1.84,
        0.88,
        M.top_oak,
        M.legs_oak,
        RectTable {
            leg: 0.065,
            taper: 0.65,
            end_arch: Some(0.0),
            ..RectTable::default()
        },
    );
    for s in [-1, 1] {
        let x = f64::from(s) * (lx - 0.0325 + 0.022 + 0.004);
        fextrude!(
            F,
            format!("drawer{s}"),
            &rounded_rect(0.4, 0.055, 0.004, 1),
            0.008,
            (x, TT - 0.045 - 0.04, 0),
            M.legs_oak,
            plane = Facing::Side,
            bevel = 0.002,
            bevel_segments = 1
        );
        let knob = revolve!(
            format!("knob{s}"),
            &[
                (0.0, 0.0),
                (0.008, 0.0),
                (0.007, 0.01),
                (0.014, 0.018),
                (0.012, 0.026),
                (0.0, 0.027)
            ],
            at = F.at(x + f64::from(s) * 0.004, TT - 0.085, 0),
            material = M.legs_oak,
            segments = 10,
            rot = (0, -90 * s, 0)
        );
        knob.edit(|o| o.mesh.shade_smooth(35.0));
    }
    finish("tableDining4")
}

/// Dining table for six (catalog2): a walnut top on six turned, tapered legs with collars, arched aprons.
pub fn table_dining6() -> Vec<Obj> {
    rect_table(
        2.84,
        0.88,
        M.top_walnut,
        M.legs_walnut,
        RectTable {
            leg: 0.07,
            turned: true,
            taper: 0.7,
            collar: Some(0.5),
            ..RectTable::default()
        },
    );
    finish("tableDining6")
}

/// Banquet table (catalog2): a long top of four planks on six chunky chamfered legs with feet, aprons and an
/// H-stretcher pegged through the legs.
pub fn table_banquet8() -> Vec<Obj> {
    let edge = [
        (0.004, 0.0),
        (0.0, 0.25),
        (0.0, 0.8),
        (0.0025, 0.95),
        (0.005, 1.0),
    ];
    for k in 0..5 {
        // five planks: the middle of the top is on one, not a joint
        slab(
            &format!("plank{k}"),
            3.84,
            0.176,
            TT,
            0.045,
            M.top_banquet,
            0.006,
            Some(&edge),
            3,
            (0.0, -0.352 + 0.176 * f64::from(k)),
        );
    }
    let (lx, lz) = (3.84 / 2.0 - 0.06 - 0.04, 0.88 / 2.0 - 0.06 - 0.04);
    for s in [-1, 1] {
        let sf = f64::from(s);
        apron(
            &format!("apron{s}"),
            2.0 * lx,
            0.08,
            0.022,
            (0, TT - 0.045, sf * (lz - 0.029)),
            M.legs_banquet,
            Facing::Front,
            0.0,
            10,
            0.003,
        );
        apron(
            &format!("apron_end{s}"),
            2.0 * lz - 0.06,
            0.08,
            0.022,
            (sf * (lx - 0.029), TT - 0.045, 0),
            M.legs_banquet,
            Facing::Side,
            0.015,
            10,
            0.003,
        );
    }
    for x in [-lx, 0.0, lx] {
        for z in [-lz, lz] {
            let name = format!("{}{}", float_repr(x), float_repr(z));
            sq_leg(
                &format!("leg{name}"),
                x,
                z,
                0.03,
                TT - 0.045,
                0.066,
                0.08,
                M.legs_banquet,
                0.01,
                0.0,
                None,
            );
            sq_leg(
                &format!("foot{name}"),
                x,
                z,
                0.0,
                0.03,
                0.075,
                0.082,
                M.legs_banquet,
                0.006,
                0.004,
                None,
            );
        }
        apron(
            &format!("cross{}", float_repr(x)),
            2.0 * lz,
            0.05,
            0.04,
            (x, 0.205, 0),
            M.legs_banquet,
            Facing::Side,
            0.0,
            10,
            0.003,
        );
    }
    fbox!(
        F,
        "stretcher",
        2.0 * lx,
        0.05,
        0.04,
        (0, 0.18, 0),
        M.legs_banquet,
        bevel = 0.008,
        bevel_segments = 2
    );
    for x in [-lx, lx] {
        let s = if x > 0.0 { 1.0 } else { -1.0 };
        fbox!(
            F,
            format!("tenon{}", float_repr(x)),
            0.03,
            0.04,
            0.03,
            (x + s * 0.05, 0.18, 0),
            M.legs_banquet,
            bevel = 0.005
        );
        fbox!(
            F,
            format!("wedge{}", float_repr(x)),
            0.014,
            0.06,
            0.012,
            (x + s * 0.058, 0.18, 0),
            M.legs_banquet,
            bevel = 0.003
        );
    }
    finish("tableBanquet8")
}

/// Square table for eight (catalog2): a white top with a softened edge on black square-tube legs near its
/// corners, a steel frame under the top and a square frame bracing the legs low down, on levelling feet.
pub fn table_square8() -> Vec<Obj> {
    slab(
        "top",
        1.86,
        1.86,
        TT,
        0.03,
        M.top_white_satin,
        0.015,
        Some(&[(0.003, 0.0), (0.0, 0.3), (0.0, 0.7), (0.003, 1.0)]),
        3,
        (0.0, 0.0),
    );
    for (y, h, t) in [(TT - 0.06, 0.06, 0.02), (0.1, 0.03, 0.03)] {
        let frame: Vec<(f64, f64, f64)> = rounded_rect(1.7, 1.7, 0.03, 2)
            .iter()
            .map(|&(u, v)| (u, y, v))
            .collect();
        fsweep!(
            F,
            format!("frame{}", float_repr(y)),
            &frame,
            0.0,
            M.legs_black,
            profile = rounded_rect(h, t, 0.004, 1),
            closed = true,
            angle = 60
        );
    }
    for x in [-0.85, 0.85] {
        for z in [-0.85, 0.85] {
            let name = format!("{}{}", float_repr(x), float_repr(z));
            sq_leg(
                &format!("leg{name}"),
                x,
                z,
                0.012,
                TT - 0.03,
                0.06,
                0.06,
                M.legs_black,
                0.006,
                0.0,
                None,
            );
            turned(
                &format!("foot{name}"),
                &[
                    (0.0, 0.0),
                    (0.026, 0.0),
                    (0.028, 0.004),
                    (0.026, 0.007),
                    (0.008, 0.008),
                    (0.008, 0.014),
                    (0.0, 0.014),
                ],
                (x, 0, z),
                M.legs_black,
                12,
                35.0,
            );
        }
    }
    finish("tableSquare8")
}

/// Farmhouse table (catalog2): a thick top of three planks on two trestles, each a chamfered post on a foot
/// shaped underneath and under a shaped bearer, braced, joined by a stretcher wedged through the posts.
pub fn table_farmhouse6() -> Vec<Obj> {
    let edge = [
        (0.004, 0.0),
        (0.0, 0.25),
        (0.0, 0.8),
        (0.0025, 0.95),
        (0.005, 1.0),
    ];
    for k in 0..3 {
        slab(
            &format!("plank{k}"),
            2.86,
            0.2925,
            TT,
            0.06,
            M.top_farm,
            0.008,
            Some(&edge),
            3,
            (0.0, f64::from(k - 1) * 0.2935),
        );
    }
    for x in [-1.15, 1.15] {
        let xs = float_repr(x);
        sq_leg(
            &format!("post{xs}"),
            x,
            0.0,
            0.07,
            TT - 0.12,
            0.1,
            0.1,
            M.legs_farm,
            0.012,
            0.0,
            None,
        );
        // the foot: arched up underneath between its pads, chamfered at its ends
        let mut foot = vec![(-0.4, 0.0), (-0.28, 0.0), (-0.26, 0.012)];
        foot.extend((0..9).map(|i| {
            let i = f64::from(i);
            (-0.24 + 0.48 * i / 8.0, 0.012 + 0.012 * sin(PI * i / 8.0))
        }));
        foot.extend([
            (0.26, 0.0),
            (0.28, 0.0),
            (0.4, 0.0),
            (0.4, 0.035),
            (0.33, 0.07),
            (-0.33, 0.07),
            (-0.4, 0.035),
        ]);
        fextrude!(
            F,
            format!("foot{xs}"),
            &foot,
            0.1,
            (x, 0, 0),
            M.legs_farm,
            plane = Facing::Side,
            bevel = 0.006,
            bevel_segments = 1
        );
        let bearer = [
            (-0.39, 0.06),
            (0.39, 0.06),
            (0.39, 0.03),
            (0.3, 0.0),
            (-0.3, 0.0),
            (-0.39, 0.03),
        ];
        fextrude!(
            F,
            format!("cleat{xs}"),
            &bearer,
            0.1,
            (x, TT - 0.12, 0),
            M.legs_farm,
            plane = Facing::Side,
            bevel = 0.006,
            bevel_segments = 1
        );
        for s in [-1, 1] {
            let sf = f64::from(s);
            board(
                &format!("brace{xs}{s}"),
                (x, 0.07, sf * 0.3),
                (x, 0.32, sf * 0.05),
                0.05,
                0.06,
                M.legs_farm,
                (1, 0, 0),
                0.008,
            );
            board(
                &format!("brace_top{xs}{s}"),
                (x, TT - 0.12, sf * 0.3),
                (x, 0.5, sf * 0.05),
                0.04,
                0.05,
                M.legs_farm,
                (1, 0, 0),
                0.008,
            );
        }
    }
    fbox!(
        F,
        "stretcher",
        2.5,
        0.08,
        0.06,
        (0, 0.32, 0),
        M.legs_farm,
        bevel = 0.012,
        bevel_segments = 2
    );
    for x in [-1.225, 1.225] {
        fbox!(
            F,
            format!("wedge{}", float_repr(x)),
            0.025,
            0.1,
            0.03,
            (x, 0.32, 0),
            M.groove,
            bevel = 0.005
        );
    }
    finish("tableFarmhouse6")
}

//! K7: dining tables.

use super::common::{apron, finish, slab, sq_leg, turned};
use super::{M, TT};
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::place::Facing;
use artkit::scene::Obj;

/// A turned table leg's profile `h` tall: a pad foot, a long taper from `r_foot` to `r_top`, a ring under the
/// apron, the block that takes the apron; `collar`: a ring that high.
fn turned_leg(h: f64, r_top: f64, r_foot: f64, collar: Option<f64>) -> Vec<(f64, f64)> {
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

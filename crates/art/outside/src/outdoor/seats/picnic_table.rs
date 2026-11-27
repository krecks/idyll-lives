use super::super::common::{F, M, cup, finish, side_slab};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::kit;
use artkit::math::{P3, cos, cosd, radians, sin};
use artkit::parts::{aside, new_parts};
use artkit::scene::Obj;
use artkit::{cylinder, fbox, fsweep, loft, ring, soft};

// --- the picnic table: a planked top on A-frames, a bench each side; a gingham cloth and a basket --------------
pub(super) const PICNIC_TOP: f64 = 0.73; // the top's and the benches' tops: content `surface`, `seats.picnicTable`
pub(super) const PICNIC_SEAT: f64 = 0.44;

/// A wicker picnic basket standing at game (x, y, z): a tapering body, a rim, two lid flaps, a bent-wood handle
/// and a strap.
pub(super) fn basket(x: f64, y: f64, z: f64, w: f64, h: f64, d: f64) {
    let b = F.at(x, y, z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    let rings = [(0.0, 0.9), (0.02, 0.94), (0.7, 1.0), (0.78, 1.0)];
    let sections: Vec<Vec<P3>> = rings
        .iter()
        .map(|&(t, k)| {
            ring!(
                w / 2.0 * k,
                d / 2.0 * k,
                z = bz + h * t,
                verts = 16,
                square = 7,
                at = (bx, by)
            )
        })
        .collect();
    artkit::bevel::eased(loft!("basket", &sections, material = M.wicker));
    let rim: Vec<(f64, f64, f64)> = rounded_rect(w + 0.012, d + 0.012, 0.03, 2)
        .iter()
        .map(|&(u, v)| (x + u, y + h * 0.78, z + v))
        .collect();
    fsweep!(
        F,
        "basket_rim",
        &rim,
        0.009,
        M.wicker_rim,
        verts = 5,
        closed = true
    );
    for s in [-1, 1] {
        // the two lid flaps, hinged along the middle
        soft!(
            F,
            format!("basket_lid{s}"),
            w + 0.004,
            0.03,
            d / 2.0 - 0.004,
            (x, y + h * 0.78 + 0.02, z + s as f64 * d / 4.0),
            M.wicker_rim,
            rot = (s * 3, 0, 0),
            round = 0.012,
            round_segments = 1,
            flat = 1
        );
    }
    let arc: Vec<(f64, f64, f64)> = (0..181)
        .step_by(20)
        .map(|a| {
            (
                x + 0.5 * w * cos(radians(a as f64)) * 0.82,
                y + h * 0.8 + 0.156 * sin(radians(a as f64)),
                z,
            )
        })
        .collect();
    fsweep!(
        F,
        "basket_handle",
        &arc,
        0,
        M.handle,
        profile = rounded_rect(0.012, 0.022, 0.005, 1),
        up = (0, 1, 0)
    );
    fbox!(
        F,
        "basket_strap",
        0.03,
        h * 0.45,
        0.006,
        (x, y + h * 0.62, z + d / 2.0 + 0.005),
        M.handle,
        bevel = 0.002,
        bevel_segments = 1
    );
}

pub fn picnic_table() -> Vec<Obj> {
    let (top, seat) = (PICNIC_TOP, PICNIC_SEAT);
    for (k, z) in [-0.29, -0.145, 0.0, 0.145, 0.29].into_iter().enumerate() {
        fbox!(
            F,
            format!("top{k}"),
            2.5,
            0.04,
            0.135,
            (0, top - 0.02, z),
            M.pine,
            bevel = 0.01
        );
    }
    for s in [-1, 1] {
        for (k, z) in [0.6, 0.745].into_iter().enumerate() {
            fbox!(
                F,
                format!("seat{s}{k}"),
                2.5,
                0.04,
                0.13,
                (0, seat - 0.02, s as f64 * z),
                M.pine,
                bevel = 0.01
            );
        }
    }
    // the cleats under the top and the bearers under the seats, their ends cut back underneath
    for x in [-0.95, 0.0, 0.95] {
        let yc = top - 0.065;
        side_slab(
            &format!("cleat{}", float_repr(x)),
            x,
            &[
                (-0.36, yc + 0.025),
                (0.36, yc + 0.025),
                (0.32, yc - 0.025),
                (-0.32, yc - 0.025),
            ],
            0.07,
            M.pine_dark,
            0.006,
        );
    }
    let hw = 0.035 / cosd(38.0);
    for s in [-1, 1] {
        let sf = s as f64;
        let x = sf * 0.95;
        let yb = seat - 0.07;
        side_slab(
            &format!("bearer{s}"),
            x,
            &[
                (-0.81, yb + 0.03),
                (0.81, yb + 0.03),
                (0.77, yb - 0.03),
                (-0.77, yb - 0.03),
            ],
            0.07,
            M.pine_dark,
            0.006,
        );
        // the A-frame's legs, cut level at the foot and under the cleat, either side of the cleat and bearer
        for e in [-1, 1] {
            let (zf, zt) = (e as f64 * 0.66, e as f64 * 0.12);
            side_slab(
                &format!("leg{s}{e}"),
                x + sf * 0.06,
                &[
                    (zf - hw, 0.0),
                    (zf + hw, 0.0),
                    (zt + hw, top - 0.04),
                    (zt - hw, top - 0.04),
                ],
                0.05,
                M.pine_dark,
                0.006,
            );
            // carriage bolts through leg and bearer, leg and cleat
            for y in [yb, top - 0.065] {
                let zl = zf + (zt - zf) * y / (top - 0.04);
                cylinder!(
                    format!("bolt{s}{e}{}", float_repr(y)),
                    0.011,
                    0.006,
                    at = F.at(x + sf * 0.087, y, zl),
                    material = M.grate,
                    rot = (0, 90, 0),
                    verts = 6
                );
            }
        }
        kit::board(
            &F,
            &format!("brace{s}"),
            (sf * 0.92, seat - 0.07, 0),
            (sf * 0.04, top - 0.07, 0),
            0.05,
            0.05,
            M.pine_dark,
            (0, 1, 0),
            0.006,
        );
    }
    // the cloth: red gingham laid across the table where the two sit, hanging over both long edges
    let (e, hang, ct) = (0.362, 0.07, 0.004);
    let drape = kit::fillet!(
        &[
            (-0.3, top - hang, -e),
            (-0.3, top + ct / 2.0, -e),
            (-0.3, top + ct / 2.0, e),
            (-0.3, top - hang, e),
        ],
        0.012,
        steps = 2
    );
    fsweep!(
        F,
        "cloth",
        &drape,
        0,
        M.cloth,
        profile = rounded_rect(0.7, ct, 0.0015, 1),
        up = (1, 0, 0)
    );
    let (cw, cd) = (0.7 / 8.0, 2.0 * e / 8.0);
    for i in 0..8 {
        for j in 0..8 {
            if (i + j) % 2 == 0 {
                fbox!(
                    F,
                    format!("check{i}{j}"),
                    cw,
                    0.0012,
                    cd,
                    (
                        -0.65 + cw * (i as f64 + 0.5),
                        top + ct + 0.0006,
                        -e + cd * (j as f64 + 0.5)
                    ),
                    M.cloth_check
                );
            }
        }
    }
    for s in [-1, 1] {
        // the checks down the hanging edges
        for i in 0..8 {
            if (i + (if s < 0 { 0 } else { 1 })) % 2 == 0 {
                fbox!(
                    F,
                    format!("check_hang{s}{i}"),
                    cw,
                    hang - 0.02,
                    0.0012,
                    (
                        -0.65 + cw * (i as f64 + 0.5),
                        top - hang / 2.0 - 0.006,
                        s as f64 * (e + ct / 2.0 + 0.0006)
                    ),
                    M.cloth_check
                );
            }
        }
    }
    // the basket on the table is carried, not its size (`aside`)
    let carried = new_parts(|| basket(0.55, top, 0.02, 0.4, 0.2, 0.28));
    aside(&carried, "basket");
    cup(
        "cup0",
        -0.45,
        top + ct,
        0.12,
        0.038,
        0.09,
        M.cup,
        true,
        200.0,
        12,
    );
    cup(
        "cup1",
        -0.15,
        top + ct,
        -0.1,
        0.035,
        0.1,
        M.jug,
        false,
        0.0,
        12,
    );
    finish("picnicTable", false)
}

//! What each kind of bike adds to the shape: stands, racks, baskets and motors.
//!
//! Split from `yard/rides.rs` (see `mod.rs`).

use super::*;

/// The city bike's own things: a chain guard, a wicker basket on the bars with a lamp, a carrier with a red
/// light, the centre stand's plate.
pub(crate) fn city_extras(s: &Bike) {
    let (z, bx, by) = (s.z, s.bb.0, s.bb.1);
    bb_shell(s, 0.075);
    // the chain guard over the top of the chain ring and the top run, between the ring and the crank
    let (ro, ri) = (s.ring + 0.028, s.ring - 0.012);
    let xe = s.rear + 0.075;
    let step = low(15, 30);
    let mut out: Vec<(f64, f64)> = (-30..91)
        .step_by(step)
        .map(|a| (bx + ro * cosd(a as f64), by + ro * sind(a as f64)))
        .collect();
    out.push((xe, chain_top(s, xe) + 0.028));
    out.push((xe, chain_top(s, xe) - 0.012));
    let mut a = 90;
    while a > -31 {
        out.push((bx + ri * cosd(a as f64), by + ri * sind(a as f64)));
        a -= step as i32;
    }
    let d = s.drive;
    fextrude!(
        F,
        "chain_guard",
        &out,
        0.004,
        (0, 0, z + 0.064 * d),
        s.colour
    );
    rod(
        "guard_stay",
        (xe + 0.01, chain_top(s, xe) - 0.005, z + 0.064 * d),
        (xe + 0.02, chain_top(s, xe) - 0.03, z + 0.03 * d),
        0.004,
        M.rim,
        4,
    );
    // the basket, wicker, on the bars' clamp, on two stays down to the fork ends; a lamp on its front
    let (cx, cy0, cy1, hx, hz) = (0.515, 0.735, 0.955, 0.135, 0.17);
    let p = F.at(cx, 0, z);
    let (px, pz) = (p.x, p.y);
    let n: usize = low(24, 8);
    loft!(
        "basket",
        &[
            ring!(
                hx * 0.92,
                hz * 0.92,
                z = cy0,
                verts = n,
                square = 4,
                at = (px, pz)
            ),
            ring!(hx, hz, z = cy1, verts = n, square = 4, at = (px, pz)),
            ring!(
                hx - 0.012,
                hz - 0.012,
                z = cy1,
                verts = n,
                square = 4,
                at = (px, pz)
            ),
            ring!(
                hx * 0.92 - 0.012,
                hz * 0.92 - 0.012,
                z = cy0 + 0.015,
                verts = n,
                square = 4,
                at = (px, pz)
            )
        ],
        material = M.basket
    );
    let rim: Vec<(f64, f64, f64)> =
        rounded_rect(2.0 * hx + 0.004, 2.0 * hz + 0.004, 0.05, low(3, 1))
            .iter()
            .map(|&(u, v)| (cx + u, cy1, z + v))
            .collect();
    fsweep!(
        F,
        "basket_rim",
        &rim,
        0.009,
        M.basket,
        verts = low(6, 4),
        closed = true
    );
    if !is_low() {
        // woven bands round it, the weave's ribs between them
        for f in [0.3, 0.62] {
            let sc_ = 0.92 + 0.08 * f + 0.004;
            let band: Vec<(f64, f64, f64)> = rounded_rect(2.0 * hx * sc_, 2.0 * hz * sc_, 0.045, 3)
                .iter()
                .map(|&(u, v)| (cx + u, cy0 + (cy1 - cy0) * f, z + v))
                .collect();
            fsweep!(
                F,
                numbered("basket_band"),
                &band,
                0.006,
                M.basket_dark,
                verts = 4,
                closed = true
            );
        }
        for i in 0..n {
            let a = 360.0 * (i as f64 + 0.5) / n as f64;
            let (c, s_) = (cosd(a), sind(a));
            let e = 2.0 / 4.0; // the rings' squareness (ring square=4)
            let (u, v) = (pow(c.abs(), e).copysign(c), pow(s_.abs(), e).copysign(s_));
            rod(
                &numbered("basket_rib"),
                (cx + hx * 0.925 * u, cy0 + 0.01, z + hz * 0.925 * v),
                (cx + hx * 1.003 * u, cy1 - 0.012, z + hz * 1.003 * v),
                0.0035,
                M.basket_dark,
                3,
            );
        }
    }
    for sg in [-1.0, 1.0] {
        rod(
            &numbered("basket_stay"),
            (cx + 0.08, cy0 + 0.005, z + sg * 0.07),
            (s.front, s.r, z + sg * 0.062),
            0.005,
            M.rim,
            low(6, 4),
        );
    }
    fbox!(
        F,
        "basket_bracket",
        0.075,
        0.03,
        0.06,
        (0.35, 0.94, z),
        M.rim,
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
    fcyl!(
        F,
        "lamp",
        0.026,
        0.045,
        (cx + hx + 0.022, 0.86, z),
        M.rim,
        rot = (0, 90, 0),
        verts = low(12, 6)
    );
    fcyl!(
        F,
        "lamp_glass",
        0.022,
        0.006,
        (cx + hx + 0.046, 0.86, z),
        M.lamp,
        rot = (0, 90, 0),
        verts = low(12, 6)
    );
    // the carrier over the back wheel, on stays to the axle and the seat stays; a red light at its back
    let (ch, x0, x1) = (0.735, -0.3, -0.8);
    for sg in [-1.0, 1.0] {
        let dz = sg * 0.065;
        bent(
            &numbered("carrier_rail"),
            &[
                (x0, ch - 0.02, z + dz * 0.5),
                (x0 - 0.04, ch, z + dz),
                (x1, ch, z + dz),
            ],
            0.0065,
            s.colour,
            0.03,
            low(6, 4),
            3,
        );
        rod(
            &numbered("carrier_strut"),
            (x0, ch - 0.02, z + dz * 0.5),
            (-0.245, 0.585, z + sg * 0.016),
            0.005,
            s.colour,
            low(6, 4),
        );
        for xa in [x1 + 0.02, -0.64] {
            rod(
                &numbered("carrier_stay"),
                (xa, ch, z + dz),
                (s.rear - 0.01, s.r + 0.01, z + sg * 0.046),
                0.0055,
                s.colour,
                low(6, 4),
            );
        }
    }
    for xa in [-0.42, -0.56, x1] {
        rod(
            &numbered("carrier_bar"),
            (xa, ch, z - 0.065),
            (xa, ch, z + 0.065),
            0.005,
            s.colour,
            low(6, 4),
        );
    }
    fbox!(
        F,
        "rear_light",
        0.025,
        0.035,
        0.07,
        (x1 - 0.014, ch - 0.025, z),
        M.reflector,
        bevel = low(0.006, 0.0),
        bevel_segments = 1
    );
    // the centre stand's plate under the chainstays
    fbox!(
        F,
        "stand_plate",
        0.05,
        0.016,
        0.06,
        (CITY_STAND.0, CITY_STAND.1 + 0.017, z),
        s.colour,
        bevel = low(0.003, 0.0),
        bevel_segments = 1
    );
}

/// A double-leg centre stand, down: a yoke on its pivot, two legs splayed to rubber feet either side.
pub(crate) fn city_stand(s: &Bike) {
    let (px, py) = CITY_STAND;
    let z = s.z;
    fcyl!(
        F,
        "stand_yoke",
        0.014,
        0.07,
        (px, py, z),
        M.rim,
        rot = (90, 0, 0),
        verts = low(10, 5)
    );
    for sg in [-1.0, 1.0] {
        fsweep!(
            F,
            numbered("stand_leg"),
            &[
                (px, py, z + sg * 0.03),
                (px + 0.02, 0.16, z + sg * 0.1),
                (px + 0.05, 0.016, z + sg * 0.155)
            ],
            0.011,
            M.rim,
            verts = low(8, 4),
            resolution = low(3, 1),
            scale = [1.0, 0.9, 0.8]
        );
        fbox!(
            F,
            numbered("stand_foot"),
            0.055,
            0.016,
            0.04,
            (px + 0.052, 0.008, z + sg * 0.158),
            M.tyre,
            bevel = low(0.004, 0.0),
            bevel_segments = 1
        );
    }
}

/// The kids' bike's own things: a chain case, a brace across the bars, streamers from the grips' ends, the
/// stabilisers (their brackets, their little wheels on the floor either side of the back wheel).
pub(crate) fn kid_extras(s: &Bike) {
    let (z, bx, by) = (s.z, s.bb.0, s.bb.1);
    bb_shell(s, 0.06);
    // the chain case: round the ring and the cog, between the ring and the crank
    let (ro, rc) = (s.ring + 0.018, s.cog + 0.018);
    let step = low(20, 40);
    let mut case: Vec<(f64, f64)> = (-80..81)
        .step_by(step)
        .map(|a| (bx + ro * cosd(a as f64), by + ro * sind(a as f64)))
        .collect();
    case.extend(
        (100..261)
            .step_by(step)
            .map(|a| (s.rear + rc * cosd(a as f64), s.r + rc * sind(a as f64))),
    );
    fextrude!(
        F,
        "chain_case",
        &case,
        0.004,
        (0, 0, z + (s.ring_dz + 0.012) * s.drive),
        M.kid_trim
    );
    rod(
        "bar_brace",
        (0.153, 0.615, z - 0.105),
        (0.153, 0.615, z + 0.105),
        0.007,
        M.rim,
        low(6, 4),
    );
    // streamers: four ribbons hanging from each grip's end, a cap over it
    let colours = [M.streamer_pink, M.streamer_yellow, M.streamer_blue];
    for sg in [-1, 1] {
        let sf = sg as f64;
        let (gx, gh, gz) = (0.16, 0.66, z + sf * 0.268);
        fcyl!(
            F,
            numbered("grip_cap"),
            0.019,
            0.012,
            (gx, gh, gz),
            M.streamer_pink,
            rot = (90, 0, 0),
            verts = low(10, 5)
        );
        for j in 0..3 {
            let a = (120 * j + 30 * sg) as f64;
            let (ox, oz) = (0.008 * cosd(a), 0.008 * sind(a));
            let pts = [
                (gx + ox, gh - 0.01, gz + sf * 0.004 + oz),
                (gx + ox - 0.012, gh - 0.06, gz + sf * 0.006 + oz * 1.4),
                (gx + ox - 0.03, gh - 0.11, gz + sf * 0.008 + oz * 1.8),
                (gx + ox - 0.05, gh - 0.15, gz + sf * 0.01 + oz * 2.0),
            ];
            fsweep!(
                F,
                numbered("streamer"),
                &pts,
                0,
                colours[j as usize],
                profile = vec![
                    (-0.004, -0.0006),
                    (0.004, -0.0006),
                    (0.004, 0.0006),
                    (-0.004, 0.0006)
                ],
                up = (1, 0, 0)
            );
        }
    }
    // the stabilisers: a bracket from the back axle out and down to each little wheel, standing on the floor
    let (sr, sx) = (0.06, s.rear - 0.03);
    for sg in [-1.0, 1.0] {
        let sz = z + sg * 0.175;
        fsweep!(
            F,
            numbered("stabiliser"),
            &[
                (s.rear, s.r, z + sg * 0.045),
                (s.rear - 0.012, 0.12, z + sg * 0.13),
                (sx, sr, sz - sg * 0.018)
            ],
            0.0075,
            M.rim,
            verts = low(6, 4)
        );
        torus!(
            numbered("stabiliser_tyre"),
            sr - 0.014,
            0.014,
            at = F.at(sx, sr, sz),
            material = M.tyre,
            rot = (90, 0, 0),
            major_segments = low(16, 8),
            minor_segments = low(5, 4)
        );
        fcyl!(
            F,
            numbered("stabiliser_hub"),
            sr - 0.022,
            0.02,
            (sx, sr, sz),
            M.trim,
            rot = (90, 0, 0),
            verts = low(12, 6)
        );
        fcyl!(
            F,
            numbered("stabiliser_axle"),
            0.007,
            0.04,
            (sx, sr, sz),
            M.rim,
            rot = (90, 0, 0),
            verts = low(6, 4)
        );
    }
}

/// The mountain bike's own things: a suspension fork's crown and stanchions, and a disc brake at each wheel.
pub(crate) fn mtb_extras(s: &Bike) {
    let z = s.z;
    let (cx, cy) = s.crown.unwrap_or(s.hb);
    fbox!(
        F,
        "fork_crown",
        0.19,
        0.05,
        0.11,
        (cx, cy, z),
        M.motor,
        bevel = low(0.008, 0.0),
        bevel_segments = 1
    );
    for sg in [-1.0, 1.0] {
        let dz = sg * 0.055;
        rod(
            "stanchion",
            (cx, cy - 0.01, z + dz),
            (s.fork_mid.0 - 0.02, s.fork_mid.1 + 0.02, z + dz),
            0.017,
            M.rim,
            low(8, 4),
        );
        rod(
            "fork_lower",
            (s.fork_mid.0 - 0.01, s.fork_mid.1 + 0.01, z + dz),
            (s.front, s.r, z + dz),
            0.026,
            M.tyre,
            low(8, 4),
        );
    }
    for x in [s.front, s.rear] {
        fcyl!(
            F,
            "disc",
            0.085,
            0.004,
            (x, s.r, z + 0.052 * s.drive),
            M.rim,
            rot = (90, 0, 0),
            verts = low(16, 8)
        );
        fbox!(
            F,
            "caliper",
            0.035,
            0.09,
            0.03,
            (x + 0.02, s.r + 0.06, z + 0.052 * s.drive),
            M.motor,
            bevel = low(0.005, 0.0),
            bevel_segments = 1
        );
    }
}

/// The road bike's own things: a bottle on the down tube, a brake caliper at each wheel.
pub(crate) fn road_extras(s: &Bike) {
    let z = s.z;
    bb_shell(s, 0.07);
    board(
        "bottle",
        (0.19, 0.49, z),
        (0.12, 0.37, z),
        0.055,
        0.055,
        M.lamp,
        (0, 1, 0),
        low(0.012, 0.0),
    );
    fcyl!(
        F,
        "bottle_cap",
        0.018,
        0.02,
        (0.115, 0.365, z),
        M.rim,
        rot = (0, 90, 0),
        verts = low(8, 4)
    );
    let crown = s.crown.unwrap_or(s.hb);
    fbox!(
        F,
        "front_brake",
        0.03,
        0.06,
        0.05,
        (crown.0 + 0.02, crown.1 - 0.02, z),
        M.motor,
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
    fbox!(
        F,
        "rear_brake",
        0.03,
        0.06,
        0.05,
        (-0.45, 0.48, z),
        M.motor,
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
}

/// The cargo bike's own things: the long box between the rider and the front wheel, with a padded bench, a
/// wooden floor and rails, a lamp on its front and a red light at the back.
pub(crate) fn cargo_extras(s: &Bike) {
    let z = s.z;
    bb_shell(s, 0.08);
    let (x0, x1, y0, y1, hz) = (0.42, 1.00, 0.30, 0.66, 0.30); // the box's ends (x), its floor and top rail (y), its half depth (z)
    fbox!(
        F,
        "box_floor",
        x1 - x0,
        0.025,
        2.0 * hz,
        ((x0 + x1) / 2.0, y0, z),
        M.box_wood,
        bevel = low(0.006, 0.0),
        bevel_segments = 1
    );
    for sg in [-1.0, 1.0] {
        fbox!(
            F,
            "box_side",
            x1 - x0,
            y1 - y0,
            0.03,
            ((x0 + x1) / 2.0, (y0 + y1) / 2.0, z + sg * (hz - 0.015)),
            s.colour,
            bevel = low(0.008, 0.0),
            bevel_segments = 1
        );
        rod(
            "box_rail",
            (x0, y1, z + sg * hz),
            (x1, y1, z + sg * hz),
            0.012,
            M.box_wood,
            6,
        );
    }
    fbox!(
        F,
        "box_front",
        0.03,
        y1 - y0,
        2.0 * hz,
        (x1, (y0 + y1) / 2.0, z),
        s.colour,
        bevel = low(0.008, 0.0),
        bevel_segments = 1
    );
    fbox!(
        F,
        "box_back",
        0.03,
        y1 - y0,
        2.0 * hz,
        (x0, (y0 + y1) / 2.0, z),
        s.colour,
        bevel = low(0.008, 0.0),
        bevel_segments = 1
    );
    fbox!(
        F,
        "box_pad",
        x1 - x0 - 0.09,
        0.05,
        2.0 * hz - 0.09,
        ((x0 + x1) / 2.0, y0 + 0.05, z),
        M.saddle,
        bevel = low(0.012, 0.0),
        bevel_segments = 1
    );
    fcyl!(
        F,
        "lamp",
        0.028,
        0.045,
        (x1 + 0.02, 0.56, z),
        M.rim,
        rot = (0, 90, 0),
        verts = low(12, 6)
    );
    fcyl!(
        F,
        "lamp_glass",
        0.024,
        0.006,
        (x1 + 0.044, 0.56, z),
        M.lamp,
        rot = (0, 90, 0),
        verts = low(12, 6)
    );
    fbox!(
        F,
        "rear_light",
        0.02,
        0.03,
        0.06,
        (-0.62, 0.70, z),
        M.reflector,
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
}

/// The e-bike's own things: the battery in the down tube, the motor at the bottom bracket, a display on the
/// bars, built-in lights, and a carrier over the back wheel.
pub(crate) fn ebike_extras(s: &Bike) {
    let z = s.z;
    bb_shell(s, 0.08);
    fcyl!(
        F,
        "motor",
        0.06,
        0.10,
        (s.bb.0, s.bb.1, z),
        M.motor,
        rot = (90, 0, 0),
        verts = low(14, 7)
    );
    board(
        "battery",
        (0.26, 0.55, z),
        (0.02, 0.37, z),
        0.13,
        0.085,
        M.battery,
        (0, 1, 0),
        low(0.01, 0.0),
    );
    fbox!(
        F,
        "display",
        0.085,
        0.03,
        0.05,
        (0.33, 1.005, z),
        M.screen,
        bevel = low(0.006, 0.0),
        bevel_segments = 1
    );
    let crown = s.crown.unwrap_or(s.hb);
    fbox!(
        F,
        "head_light",
        0.06,
        0.04,
        0.06,
        (crown.0 + 0.03, crown.1 + 0.02, z),
        M.lamp,
        bevel = low(0.008, 0.0),
        bevel_segments = 1
    );
    fbox!(
        F,
        "tail_light",
        0.02,
        0.03,
        0.06,
        (-0.56, 0.73, z),
        M.reflector,
        bevel = low(0.004, 0.0),
        bevel_segments = 1
    );
    let ch = 0.76;
    for sg in [-1.0, 1.0] {
        let dz = sg * 0.07;
        rod(
            "rack_rail",
            (-0.34, ch, z + dz),
            (-0.72, ch, z + dz),
            0.007,
            M.rim,
            low(6, 4),
        );
        rod(
            "rack_stay",
            (-0.68, ch, z + dz),
            (-0.56, s.r + 0.01, z + sg * 0.05),
            0.006,
            M.rim,
            low(6, 4),
        );
    }
    for x in [-0.40, -0.56, -0.70] {
        rod(
            "rack_bar",
            (x, ch, z - 0.07),
            (x, ch, z + 0.07),
            0.005,
            M.rim,
            low(6, 4),
        );
    }
}

use super::super::common::{F, M, arc_points, bent, rod};
use artkit::far::{is_low, low};
use artkit::materials::Mat;
use artkit::math::{Axis, P3, cosd, sind};
use artkit::motion::{Spin, Swing, attach, spin, swing};
use artkit::parts::numbered;
use artkit::parts::{hinged, join, join_all, new_parts};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fsweep, loft, revolve, settle, torus};

// --- the bike rack (2x1): four galvanised hoops, a bike parked in front; on a tune-up its back wheel lifts ------
pub const WHEEL_R: f64 = 0.31;
pub const REAR: f64 = -0.52; // the axles, game x
pub const FRONT: f64 = 0.5;
pub const BZ: f64 = 0.0; // the bike's middle, depth
pub(super) const RACK_Z: f64 = -0.3;
pub(super) const LIFT: f64 = 8.5; // degrees the bike tips up about the front tyre to rest its back axle on the stand

/// A step-through's curved tubes: (the top tube's path, the down tube's), game (x, height) points.
pub type Step = (Vec<(f64, f64)>, Vec<(f64, f64)>);
/// A grip from one game point to another.
pub type Grip = ((f64, f64, f64), (f64, f64, f64));

/// A bike's numbers, game metres along its side (x, the front wheel to +x; height), `z` its middle across; `k`
/// scales the small things (tyres, hubs, tube and rod sizes) from the rack's bike, whose numbers are the defaults.
#[derive(Clone)]
pub struct Bike {
    pub r: f64,
    pub k: f64,
    /// the tyre's tube (0.02 k)
    pub tyre: Option<f64>,
    pub rear: f64,
    pub front: f64,
    pub z: f64,
    pub spokes: usize,
    /// the tyre's points round (full, far)
    pub segments: (usize, usize),
    pub rim_segments: usize,
    pub tyre_minor: usize,
    /// knobbly tyre lugs round the tread (a mountain bike's)
    pub knobs: usize,
    pub bb: (f64, f64),
    pub sc: (f64, f64),
    pub ht: (f64, f64),
    pub hb: (f64, f64),
    /// the fork's crown (default `hb`)
    pub crown: Option<(f64, f64)>,
    /// the head tube's ends
    pub head: ((f64, f64), (f64, f64)),
    pub fork_mid: (f64, f64),
    /// seat tube, top tube, down tube
    pub tubes: (f64, f64, f64),
    /// a step-through: (top tube path, down tube path), curved
    pub step: Option<Step>,
    pub stay_dz: f64,
    /// the mudguards' arcs, front and back (degrees, 0 to +x)
    pub guards: Vec<(f64, f64)>,
    /// the seat post's top
    pub post: (f64, f64),
    /// the saddle's back end and its underside
    pub saddle: (f64, f64),
    /// along it, half its width, its thickness
    pub saddle_rows: Vec<(f64, f64, f64)>,
    pub stem: ((f64, f64), (f64, f64)),
    /// (x, h, dz)
    pub bars: Vec<(f64, f64, f64)>,
    pub bend: f64,
    pub grips: Vec<Grip>,
    pub grip_r: f64,
    pub bell: Option<(f64, f64, f64)>,
    pub ring: f64,
    pub ring_dz: f64,
    pub cog: f64,
    pub chain_drop: f64,
    /// the chain's side: +z (the rack's, to the front), -z (the rider's right)
    pub drive: f64,
    /// the drive side's pedal from the bottom bracket
    pub crank: (f64, f64),
    pub crank_dz: f64,
    pub pedal_dz: f64,
    pub pedal: (f64, f64, f64),
    pub pedal_segments: usize,
    /// the rack's bike: a kickstand folded up
    pub kickstand: bool,
    /// the frame's palette colour
    pub colour: Mat,
    /// the mudguards' (default the frame's)
    pub guard_colour: Option<Mat>,
    /// the mudguards' section's scale (default `k`)
    pub guard_k: Option<f64>,
}

impl Default for Bike {
    fn default() -> Bike {
        Bike {
            r: WHEEL_R,
            k: 1.0,
            tyre: None,
            rear: REAR,
            front: FRONT,
            z: BZ,
            spokes: 16,
            segments: (32, 10),
            rim_segments: 32,
            tyre_minor: 6,
            knobs: 0,
            bb: (-0.1, 0.27),
            sc: (-0.24, 0.66),
            ht: (0.3, 0.68),
            hb: (0.34, 0.56),
            crown: None,
            head: ((0.345, 0.55), (0.295, 0.7)),
            fork_mid: (0.4, 0.42),
            tubes: (0.018, 0.016, 0.02),
            step: None,
            stay_dz: 0.035,
            guards: vec![(20.0, 120.0), (60.0, 160.0)],
            post: (-0.27, 0.78),
            saddle: (-0.38, 0.785),
            saddle_rows: vec![
                (0.0, 0.03, 0.02),
                (0.03, 0.08, 0.035),
                (0.08, 0.075, 0.035),
                (0.14, 0.035, 0.03),
                (0.2, 0.015, 0.022),
                (0.23, 0.008, 0.012),
            ],
            stem: ((0.3, 0.68), (0.27, 0.8)),
            bars: vec![
                (0.21, 0.81, -0.2),
                (0.27, 0.8, -0.13),
                (0.27, 0.8, 0.13),
                (0.21, 0.81, 0.2),
            ],
            bend: 0.05,
            grips: vec![
                ((0.222, 0.808, -0.185), (0.206, 0.81, -0.211)),
                ((0.222, 0.808, 0.185), (0.206, 0.81, 0.211)),
            ],
            grip_r: 0.016,
            bell: Some((0.27, 0.82, 0.12)),
            ring: 0.075,
            ring_dz: 0.05,
            cog: 0.035,
            chain_drop: 0.47,
            drive: 1.0,
            crank: (0.1, -0.13),
            crank_dz: 0.06,
            pedal_dz: 0.04,
            pedal: (0.08, 0.02, 0.06),
            pedal_segments: 2,
            kickstand: true,
            colour: M.bike,
            guard_colour: None,
            guard_k: None,
        }
    }
}

/// A wheel at axle (x, S.r, z): a treaded tyre, a rim, a hub with its flanges and laced spokes, in the game's
/// x-height plane.
pub fn wheel(x: f64, z: f64, s: &Bike) {
    let (r, k) = (s.r, s.k);
    let t = s.tyre.unwrap_or(0.02 * k);
    let y = r;
    torus!(
        numbered("tyre"),
        r - t,
        t,
        at = F.at(x, y, z),
        material = M.tyre,
        rot = (90, 0, 0),
        major_segments = low(s.segments.0, s.segments.1),
        minor_segments = low(s.tyre_minor, 3)
    );
    if s.knobs != 0 && !is_low() {
        // lugs across the tread, in two staggered rows
        for i in 0..s.knobs {
            let a = 360.0 * (i as f64 + 0.5) / s.knobs as f64;
            let (c, sn) = (cosd(a), sind(a));
            for side in [-1.0, 1.0] {
                let dz = side * 0.012 * k;
                rod(
                    &numbered("knob"),
                    (x + (r - 0.008 * k) * c, y + (r - 0.008 * k) * sn, z + dz),
                    (x + (r + 0.014 * k) * c, y + (r + 0.014 * k) * sn, z + dz),
                    0.008 * k,
                    M.tyre,
                    4,
                );
            }
        }
    }
    if !is_low() {
        let rim = r - 2.0 * t - 0.003 * k;
        torus!(
            numbered("rim"),
            rim,
            0.009 * k,
            at = F.at(x, y, z),
            material = M.rim,
            rot = (90, 0, 0),
            major_segments = s.rim_segments,
            minor_segments = 4
        );
        let hub: Vec<(f64, f64)> = [
            (0.0, -0.05),
            (0.008, -0.05),
            (0.008, -0.035),
            (0.024, -0.033),
            (0.024, -0.026),
            (0.013, -0.022),
            (0.013, 0.022),
            (0.024, 0.026),
            (0.024, 0.033),
            (0.008, 0.035),
            (0.008, 0.05),
            (0.0, 0.05),
        ]
        .iter()
        .map(|&(u, v)| (u * k, v * k))
        .collect();
        revolve!(
            numbered("hub"),
            &hub,
            at = F.at(x, y, z),
            material = M.rim,
            segments = 8,
            rot = (90, 0, 0)
        );
        let n = s.spokes;
        for i in 0..n {
            // laced: each spoke leaves its flange tangentially
            let a = 360.0 / n as f64 * i as f64;
            let side = if i % 2 == 1 { 0.03 * k } else { -0.03 * k };
            let lead = if i % 2 == 1 { 12.0 } else { -12.0 };
            rod(
                &numbered("spoke"),
                (
                    x + 0.022 * k * cosd(a + lead),
                    y + 0.022 * k * sind(a + lead),
                    z + side,
                ),
                (
                    x + (rim - 0.007 * k) * cosd(a),
                    y + (rim - 0.007 * k) * sind(a),
                    z,
                ),
                0.0022 * k,
                M.rim,
                3,
            );
        }
    } else {
        fcyl!(
            F,
            numbered("hub"),
            0.022 * k,
            0.1 * k,
            (x, y, z),
            M.rim,
            rot = (90, 0, 0),
            verts = 5
        );
    }
}

/// The bike `S`, standing along x: its frame, front wheel, bars, saddle, cranks (the back wheel is the caller's).
/// Returns its parts made, as lists: (frame, front wheel, cranks).
pub fn bike(s: &Bike) -> (Vec<Obj>, Vec<Obj>, Vec<Obj>) {
    let (k, bz) = (s.k, s.z);
    let r = s.r;
    let (ra, fa) = ((s.rear, r), (s.front, r));
    let (bb, sc, ht, hb) = (s.bb, s.sc, s.ht, s.hb);
    let crown = s.crown.unwrap_or(hb);
    let rdz = s.ring_dz * s.drive;
    let body = s.colour;
    let at = |p: (f64, f64), dz: f64| (p.0, p.1, bz + dz);
    let frame = new_parts(|| {
        // the main triangle, the tubes tapering a little; a head tube; the stays in pairs. A step-through's top and
        // down tubes curve down from the head tube, the top one into the seat tube low down.
        fsweep!(
            F,
            numbered("tube"),
            &[at(bb, 0.0), at(sc, 0.0)],
            s.tubes.0,
            body,
            verts = low(10, 5),
            scale = [1.0, 0.85]
        );
        if let Some((top, down)) = &s.step {
            for (path, r) in [(top, s.tubes.1), (down, s.tubes.2)] {
                let pts: Vec<(f64, f64, f64)> = path.iter().map(|&p| at(p, 0.0)).collect();
                fsweep!(
                    F,
                    numbered("tube"),
                    &pts,
                    r,
                    body,
                    verts = low(10, 5),
                    scale = [1.0, 0.85],
                    resolution = low(4, 2)
                );
            }
        } else {
            for (a, b, r) in [(sc, ht, s.tubes.1), (bb, hb, s.tubes.2)] {
                fsweep!(
                    F,
                    numbered("tube"),
                    &[at(a, 0.0), at(b, 0.0)],
                    r,
                    body,
                    verts = low(10, 5),
                    scale = [1.0, 0.85]
                );
            }
        }
        fsweep!(
            F,
            "head_tube",
            &[at(s.head.0, 0.0), at(s.head.1, 0.0)],
            0.022 * k,
            body,
            verts = low(10, 5)
        );
        for dz in [-s.stay_dz, s.stay_dz] {
            fsweep!(
                F,
                numbered("chainstay"),
                &[at(ra, dz), at(bb, dz * 0.4)],
                0.011 * k,
                body,
                verts = low(6, 4),
                scale = [0.8, 1.0]
            );
            fsweep!(
                F,
                numbered("seatstay"),
                &[
                    at(ra, dz),
                    at(
                        ((ra.0 + sc.0) / 2.0, (ra.1 + sc.1) / 2.0 + 0.01 * k),
                        dz * 0.6
                    ),
                    at(sc, dz * 0.25)
                ],
                0.009 * k,
                body,
                verts = low(6, 4),
                resolution = low(2, 1)
            );
            // the fork's blades, raked forward to the axle
            fsweep!(
                F,
                numbered("fork"),
                &[at(crown, dz * 0.5), at(s.fork_mid, dz * 0.9), at(fa, dz)],
                0.011 * k,
                body,
                verts = low(6, 4),
                resolution = low(3, 1),
                scale = [1.1, 1.0, 0.8]
            );
        }
    });
    let front_wheel = new_parts(|| wheel(s.front, bz, s));
    let more = new_parts(|| {
        // mudguards: a C-section arc over each wheel, on stays
        let gk = s.guard_k.unwrap_or(k);
        for (x, &(a0, a1)) in [s.front, s.rear].into_iter().zip(&s.guards) {
            let pts = arc_points((x, r, bz), r + 0.03 * k, a0, a1, low(10, 4), true);
            let prof: Vec<(f64, f64)> = [
                (-0.025, 0.0),
                (-0.02, 0.008),
                (0.0, 0.01),
                (0.02, 0.008),
                (0.025, 0.0),
                (0.0, 0.004),
            ]
            .iter()
            .map(|&(u, v)| (u * gk, v * gk))
            .collect();
            fsweep!(
                F,
                numbered("guard"),
                &pts,
                0,
                s.guard_colour.unwrap_or(body),
                profile = prof,
                up = (0, 1, 0)
            );
        }
        // seat post and a saddle; stem and swept-back bars with grips; a bell
        rod(
            "seat_post",
            at(sc, 0.0),
            at(s.post, 0.0),
            0.012 * k,
            M.rim,
            8,
        );
        let (x0, y0) = s.saddle;
        let secs: Vec<Vec<P3>> = s
            .saddle_rows
            .iter()
            .map(|&(u, hw, th)| {
                (0..360)
                    // twelve round: at eight the flattened sides turned past 80°
                    .step_by(low(30, 90))
                    .map(|a| {
                        F.at(
                            x0 + u,
                            y0 + th * (0.5 + 0.5 * sind(a as f64)),
                            bz + hw * cosd(a as f64),
                        )
                    })
                    .collect()
            })
            .collect();
        // shaded smooth round its padded rows; only its ends stay sharp
        loft!("saddle", &secs, material = M.saddle, angle = 80);
        rod(
            "stem",
            at(s.stem.0, 0.0),
            at(s.stem.1, 0.0),
            0.013 * k,
            M.rim,
            8,
        );
        let bars: Vec<(f64, f64, f64)> = s.bars.iter().map(|&(x, h, dz)| (x, h, bz + dz)).collect();
        bent("bars", &bars, 0.011 * k, M.rim, s.bend, low(8, 5), 3, 1.0);
        for &(a, b) in &s.grips {
            rod(
                &numbered("grip"),
                (a.0, a.1, bz + a.2),
                (b.0, b.1, bz + b.2),
                s.grip_r,
                M.saddle,
                low(8, 5),
            );
        }
        if let Some(bell) = s.bell {
            fcyl!(
                F,
                "bell",
                0.02 * k,
                0.015 * k,
                (bell.0, bell.1, bz + bell.2),
                M.rim,
                verts = 8
            );
        }
        fcyl!(
            F,
            "cog",
            s.cog,
            0.008 * k,
            (s.rear, r, bz + rdz),
            M.rim,
            rot = (90, 0, 0),
            verts = low(10, 5)
        );
    });
    let cranks = new_parts(|| {
        // chain ring, cranks and pedals on the drive side (the front), the chain to the back cog
        torus!(
            "chainring",
            s.ring,
            0.006 * k,
            at = F.at(bb.0, bb.1, bz + rdz),
            material = M.rim,
            rot = (90, 0, 0),
            major_segments = low(24, 8),
            minor_segments = 4
        );
        for sg in [-1.0, 1.0] {
            let a = (s.crank.0 * sg, s.crank.1 * sg);
            let dz = s.crank_dz * sg;
            rod(
                &numbered("crank"),
                at(bb, dz),
                (bb.0 + a.0, bb.1 + a.1, bz + dz),
                0.009 * k,
                M.rim,
                6,
            );
            fbox!(
                F,
                numbered("pedal"),
                s.pedal.0,
                s.pedal.1,
                s.pedal.2,
                (bb.0 + a.0, bb.1 + a.1, bz + dz + s.pedal_dz * sg),
                M.tyre,
                bevel = 0.005 * k,
                bevel_segments = s.pedal_segments
            );
        }
    });
    let chain = new_parts(|| {
        for dy in [s.ring, -s.ring].into_iter().take(low(2, 0)) {
            rod(
                &numbered("chain"),
                (bb.0, bb.1 + dy, bz + rdz),
                (s.rear, r + dy * s.chain_drop, bz + rdz),
                0.004 * k,
                M.tyre,
                4,
            );
        }
        if s.kickstand {
            // a kickstand, folded up along the chainstay
            rod(
                "kickstand",
                (-0.14, 0.25, bz - 0.05),
                (-0.42, 0.24, bz - 0.06),
                0.008,
                M.rim,
                6,
            );
        }
    });
    let mut all = frame;
    all.extend(more);
    all.extend(chain);
    (all, front_wheel, cranks)
}

/// Where the back axle is once the bike tips up by LIFT about the front tyre's foot (game x, height).
pub(super) fn rear_axle_lifted() -> (f64, f64) {
    let (dx, dy) = (REAR - FRONT, WHEEL_R);
    let (c, s) = (cosd(LIFT), sind(LIFT));
    (FRONT + dx * c + dy * s, -dx * s + dy * c)
}

pub fn bike_rack() -> Vec<Obj> {
    let still = new_parts(|| {
        fbox!(
            F,
            "rail",
            1.8,
            0.05,
            0.12,
            (0, 0.025, RACK_Z),
            M.rack,
            bevel = 0.01,
            bevel_segments = 1
        );
        for x in [-0.6, -0.2, 0.2, 0.6] {
            // inverted-U hoops on the rail, each one bent tube, on its feet
            bent(
                &numbered("hoop"),
                &[
                    (x - 0.15, 0.04, RACK_Z),
                    (x - 0.15, 0.57, RACK_Z),
                    (x + 0.15, 0.57, RACK_Z),
                    (x + 0.15, 0.04, RACK_Z),
                ],
                0.02,
                M.rack,
                0.15,
                low(10, 6),
                low(5, 2),
                1.0,
            );
            for e in [-1.0, 1.0] {
                fbox!(
                    F,
                    numbered("hoop_foot"),
                    0.07,
                    0.008,
                    0.1,
                    (x + e * 0.15, 0.054, RACK_Z),
                    M.rack,
                    bevel = low(0.003, 0.0),
                    bevel_segments = 1
                );
            }
        }
        // the repair stand behind the back wheel: two legs either side of it, a cradle at the lifted axle
        let (ax, ay) = rear_axle_lifted();
        for dz in [-0.075, 0.075] {
            let z = BZ + dz;
            for e in [-1.0, 1.0] {
                rod(
                    &numbered("stand_leg"),
                    (ax + e * 0.16, 0.0, z + dz * 0.8),
                    (ax, ay - 0.03, z),
                    0.012,
                    M.rack,
                    6,
                );
            }
            fsweep!(
                F,
                numbered("cradle"),
                &arc_points((ax, ay, z), 0.022, 180.0, 360.0, 4, true),
                0.008,
                M.rack,
                verts = 4
            );
            for e in [-1.0, 1.0] {
                fcyl!(
                    F,
                    numbered("stand_foot"),
                    0.018,
                    0.012,
                    (ax + e * 0.16, 0.006, z + dz * 0.8),
                    M.tyre,
                    verts = 6
                );
            }
        }
    });
    if !is_low() {
        join(&still, "rack", Some(P3::from((0, 0, 0))));
    }
    let rack_bike = Bike::default();
    let bike_parts = new_parts(|| {
        bike(&rack_bike);
    });
    let wheel_parts = new_parts(|| wheel(REAR, BZ, &rack_bike));
    if is_low() {
        return settle!(&[join_all("bikeRack")], centre = false);
    }
    let b = hinged(join(&bike_parts, "bike", None), F.at(FRONT, 0, BZ));
    let w = hinged(
        join(&wheel_parts, "wheel_rear", None),
        F.at(REAR, WHEEL_R, BZ),
    );
    let objs = settle!(centre = false);
    attach(w, b);
    swing(
        b,
        "wheel_up",
        -LIFT,
        Swing {
            seconds: 0.8,
            axis: Axis::Y,
            ..Swing::default()
        },
    );
    spin(
        w,
        "wheel_turn",
        Spin {
            turns: 1.0,
            seconds: 1.2,
            axis: Axis::Y,
        },
    );
    objs
}

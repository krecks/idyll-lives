//! The fold-down wall bed and the sofa bed.

use super::common::{Cushion, book, cushion, fillet, leg, panel, pillow, plain};
use super::{F, M};
use artkit::math::{Axis, P3, PI, cos, sin};
use artkit::motion::{Grow, grow};
use artkit::parts::{hinged, join, new_parts, turn};
use artkit::place::Facing;
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsweep, jitter, settle, soft, swing};

// The footprint is 2 wide and 3 deep: depth -1.5 to +1.5; the wall's face is 7 cm in (objects.md).
const BACK: f64 = -1.43;
const CAB_FRONT: f64 = -0.94; // the cabinet's sides end here
const HINGE: (f64, f64) = (0.12, -0.96); // (height, depth) of the bed's hinge, along x: the folded bed clears the back
const LENGTH: f64 = 1.98; // the bed, head end to foot end
const BED_W: f64 = 1.56; // the bed panel's width, inside the cabinet's sides
const MATTRESS_TOP: f64 = 0.46; // characters.ts BEDS['wallBed']
const INNER_TOP: f64 = 2.14; // the cabinet's opening: the folded bed reaches HINGE height + LENGTH
const SECONDS: f64 = 1.0; // the bed turns down this fast, as the use that opens it starts

/// The housing and the side shelves: what stays still. A moulded cornice across the top, a recessed plinth, the
/// shelves edged and lined with books.
fn cabinet() {
    let depth = CAB_FRONT - BACK;
    let mid = (BACK + CAB_FRONT) / 2.0;
    fbox!(
        F,
        "back",
        1.66,
        INNER_TOP,
        0.02,
        (0, INNER_TOP / 2.0, BACK + 0.01),
        M.inside
    );
    for s in [-1, 1] {
        fbox!(
            F,
            format!("side{s}"),
            0.03,
            INNER_TOP + 0.03,
            depth,
            (f64::from(s) * 0.815, (INNER_TOP + 0.03) / 2.0, mid),
            M.body,
            bevel = 0.004
        );
    }
    fbox!(
        F,
        "top",
        1.66,
        0.03,
        depth,
        (0, INNER_TOP + 0.015, mid),
        M.body,
        bevel = 0.004
    );
    fbox!(
        F,
        "plinth",
        1.6,
        0.08,
        0.04,
        (0, 0.04, CAB_FRONT - 0.06),
        M.body
    );
    // the cornice: a cove rising to a square top, run across the front and returned along the sides
    let mut ogee = vec![
        (BACK, INNER_TOP + 0.03),
        (CAB_FRONT + 0.002, INNER_TOP + 0.03),
    ];
    ogee.extend((1..6).map(|i| {
        let a = PI / 2.0 * f64::from(i) / 5.0;
        (
            CAB_FRONT + 0.002 + 0.028 * (1.0 - cos(a)),
            INNER_TOP + 0.03 + 0.03 * sin(a),
        )
    }));
    ogee.extend([
        (CAB_FRONT + 0.03, INNER_TOP + 0.08),
        (BACK, INNER_TOP + 0.08),
    ]);
    fextrude!(
        F,
        "crown",
        &ogee,
        1.98,
        (0, 0, 0),
        M.body,
        plane = Facing::Side,
        bevel = 0.004,
        bevel_segments = 1
    );
    fbox!(
        F,
        "crown_bead",
        1.94,
        0.012,
        0.012,
        (0, INNER_TOP + 0.024, CAB_FRONT + 0.004),
        M.body,
        bevel = 0.005,
        bevel_segments = 2
    );
    // open shelves on both sides, edged, a few books on each shelf
    for s in [-1, 1] {
        let x = f64::from(s) * 0.905;
        fbox!(
            F,
            format!("tower_side{s}"),
            0.02,
            INNER_TOP + 0.03,
            depth,
            (f64::from(s) * 0.98, (INNER_TOP + 0.03) / 2.0, mid),
            M.body,
            bevel = 0.004
        );
        fbox!(
            F,
            format!("tower_back{s}"),
            0.13,
            INNER_TOP,
            0.02,
            (x, INNER_TOP / 2.0, BACK + 0.01),
            M.inside
        );
        for (i, y) in [0.0, 0.45, 0.9, 1.35, 1.8].into_iter().enumerate() {
            fbox!(
                F,
                format!("shelf{s}{i}"),
                0.13,
                0.02,
                depth - 0.02,
                (x, y + 0.01, mid),
                M.inside
            );
            fbox!(
                F,
                format!("shelf_edge{s}{i}"),
                0.13,
                0.03,
                0.012,
                (x, y + 0.012, CAB_FRONT - 0.016),
                M.body,
                bevel = 0.003,
                bevel_segments = 1
            );
            if i < 4 {
                books(x, y + 0.02, mid, i * 2 + usize::from(s > 0));
            }
        }
    }
}

/// Three or four books standing on a shelf at height `y`, across the 13 cm wide tower: covers, the pages showing
/// on top, a gilt band on some spines.
fn books(x: f64, y: f64, z: f64, seed: usize) {
    let colours = [M.book_a, M.book_b, M.book_c, M.book_d];
    let heights = [0.24, 0.2, 0.27, 0.22, 0.18];
    let n = 3 + seed % 2;
    for k in 0..n {
        let h = heights[(seed + k) % heights.len()];
        let lean = ((seed + k) % 3) as f64 - 1.0;
        book(
            x - 0.045 + k as f64 * 0.03,
            y,
            z + 0.09 + 0.02 * lean,
            0.028,
            h,
            0.18,
            colours[(seed + k) % 4],
            (seed + k).is_multiple_of(3),
        );
    }
}

/// The bed in its down pose: frame, a piped mattress, pillows and a folded throw on top; underneath, two raised
/// and fielded door panels, a bar handle on stand-offs and a folding foot with bracket ends. Returns it as one part
/// with its origin on the hinge.
fn bed() -> Obj {
    let (h0, z0) = HINGE;
    let (head, foot) = (z0 - 0.02, z0 - 0.02 + LENGTH);
    let mid = (head + foot) / 2.0;
    let parts = new_parts(|| {
        fbox!(
            F,
            "bed_base",
            BED_W,
            0.04,
            LENGTH,
            (0, h0 + 0.02, mid),
            M.body,
            bevel = 0.004
        );
        for s in [-1, 1] {
            fbox!(
                F,
                format!("rail{s}"),
                0.03,
                0.14,
                LENGTH,
                (f64::from(s) * (BED_W / 2.0 - 0.015), h0 + 0.11, mid),
                M.frame,
                bevel = 0.008,
                bevel_segments = 2
            );
        }
        for (end, z) in [("head", head + 0.015), ("foot", foot - 0.015)] {
            fbox!(
                F,
                format!("rail_{end}"),
                BED_W - 0.06,
                0.14,
                0.03,
                (0, h0 + 0.11, z),
                M.frame,
                bevel = 0.008,
                bevel_segments = 2
            );
        }
        let top = MATTRESS_TOP;
        let puff = 0.012; // the mattress's middle stands at its top, its edges this much lower
        soft!(
            F,
            "mattress",
            BED_W - 0.07,
            top - (h0 + 0.10) - puff,
            LENGTH - 0.07,
            (0, (top - puff + h0 + 0.10) / 2.0, mid),
            M.mattress,
            round = 0.04,
            puff = puff,
            piping = 0.007,
            flat = 4
        );
        // a throw folded across the foot, its fold rolled over towards the foot end
        let throw = soft!(
            F,
            "throw",
            BED_W - 0.03,
            0.05,
            0.44,
            (0, top + 0.02, foot - 0.26),
            M.throw,
            round = 0.022,
            puff = 0.01,
            flat = 3
        );
        jitter!(throw, 0.006, size = 0.18, seed = 1.5, up = 0.4);
        for s in [-1, 1] {
            soft!(
                F,
                format!("pillow{s}"),
                0.62,
                0.1,
                0.36,
                (f64::from(s) * 0.36, top + 0.035, head + 0.25),
                M.pillow,
                rot = (-6, 0, s * 2),
                round = 0.045,
                puff = 0.035,
                flat = 3
            );
        }
        // underneath, facing the room once folded: two door panels, each a flat frame round a raised field
        for (k, (z1, z2)) in [(head + 0.08, mid - 0.04), (mid + 0.04, foot - 0.16)]
            .into_iter()
            .enumerate()
        {
            fbox!(
                F,
                format!("door{k}"),
                BED_W - 0.16,
                0.015,
                z2 - z1,
                (0, h0 - 0.0075, (z1 + z2) / 2.0),
                M.body,
                bevel = 0.005,
                bevel_segments = 1
            );
            fbox!(
                F,
                format!("field{k}"),
                BED_W - 0.32,
                0.012,
                z2 - z1 - 0.16,
                (0, h0 - 0.018, (z1 + z2) / 2.0),
                M.body,
                bevel = 0.008,
                bevel_segments = 2
            );
        }
        let handle = head + 1.32; // about 1.45 m up once folded
        fsweep!(
            F,
            "handle",
            &fillet(
                &[
                    (-0.25, h0 - 0.024, handle),
                    (-0.25, h0 - 0.06, handle),
                    (0.25, h0 - 0.06, handle),
                    (0.25, h0 - 0.024, handle)
                ],
                0.014,
                3
            ),
            0.009,
            M.metal,
            verts = 8
        );
        for s in [-1, 1] {
            fcyl!(
                F,
                format!("handle_rose{s}"),
                0.016,
                0.008,
                (f64::from(s) * 0.25, h0 - 0.026, handle),
                M.metal,
                verts = 12,
                bevel = 0.003
            );
        }
        // the folding foot: a board with bracket ends, cut away between them
        let w = BED_W - 0.2;
        let mut foot_shape = vec![(-w / 2.0, 0.0), (-w / 2.0 + 0.1, 0.0)];
        foot_shape.extend((1..5).map(|i| {
            let a = PI / 2.0 * f64::from(i) / 4.0;
            (-w / 2.0 + 0.1 + 0.05 * (1.0 - cos(a)), 0.045 * sin(a))
        }));
        foot_shape.push((w / 2.0 - 0.15, 0.045));
        foot_shape.extend((1..5).map(|i| {
            let a = PI / 2.0 * f64::from(4 - i) / 4.0;
            (w / 2.0 - 0.1 - 0.05 * (1.0 - cos(a)), 0.045 * sin(a))
        }));
        foot_shape.extend([(w / 2.0, 0.0), (w / 2.0, h0), (-w / 2.0, h0)]);
        fextrude!(
            F,
            "foot_leg",
            &foot_shape,
            0.04,
            (0, 0, foot - 0.09),
            M.body,
            bevel = 0.004,
            bevel_segments = 1
        );
    });
    hinged(join(&parts, "bed", None), F.at(0, h0, z0))
}

pub fn wall_bed() -> Vec<Obj> {
    let cab = new_parts(cabinet);
    join(&cab, "cabinet", Some((0, 0, 0).into()));
    let bed = bed();
    // Built down; folded up is the rest pose (the clip's first key): its foot up, its mattress to the wall.
    turn(bed, -90.0, Axis::X);
    settle!(centre = false); // it stands at the back of its tiles, where it was built
    swing!(bed, "bed_down", 90.0, seconds = SECONDS, axis = Axis::X);
    Vec::new()
}

// The sofa bed: 2 x 1 tiles, its back on the wall's face (depth -0.43), 1.86 m wide like its old part.
const SB_BACK: f64 = -0.43;
const SB_FRONT: f64 = 0.40;
const SB_W: f64 = 1.86;
const SB_SEAT: f64 = 0.45; // the seat cushion's top (content seat height 0.44)
const SB_MATTRESS: f64 = 0.44; // the pulled-out mattress's top: characters.ts BEDS['sofaBed'].h
const SB_HEAD: f64 = -0.26; // the mattress out, head end (under the back cushion)
const SB_FOOT: f64 = 1.52; // to foot end
const SB_TUCKED: f64 = 0.2; // its share of its length at rest: tucked into the base under the seat, its leg too

pub fn sofa_bed() -> Vec<Obj> {
    let still = new_parts(|| {
        let mid = (SB_BACK + SB_FRONT) / 2.0;
        let depth = SB_FRONT - SB_BACK;
        panel(
            "base",
            SB_W - 0.04,
            0.22,
            depth,
            (0.0, 0.21, mid),
            M.upholstery,
            0.03,
            0.0,
            Some(0.005),
            plain(0.03),
        );
        panel(
            "back",
            SB_W,
            0.5,
            0.18,
            (0.0, 0.57, SB_BACK + 0.09),
            M.upholstery,
            0.05,
            0.0,
            None,
            plain(0.05),
        );
        for s in [-1, 1] {
            panel(
                &format!("arm{s}"),
                0.17,
                0.52,
                depth,
                (f64::from(s) * (SB_W - 0.17) / 2.0, 0.36, mid),
                M.upholstery,
                0.06,
                0.0,
                Some(0.006),
                plain(0.06),
            );
        }
        // one long seat cushion over the folded bed, sewn in two with a welt down the middle; two back cushions
        cushion(
            "seat",
            1.5,
            0.13,
            0.64,
            (0.0, SB_SEAT - 0.065, SB_FRONT - 0.33),
            M.cushion,
            Cushion {
                puff: 0.01,
                flat: Some(4),
                ..Cushion::default()
            },
        ); // its edges over the folded bed
        fsweep!(
            F,
            "seat_seam",
            &[
                (0, SB_SEAT - 0.03, SB_FRONT - 0.012),
                (0, SB_SEAT + 0.006, SB_FRONT - 0.03),
                (0, SB_SEAT + 0.012, SB_FRONT - 0.33),
                (0, SB_SEAT + 0.006, SB_FRONT - 0.63),
                (0, SB_SEAT - 0.03, SB_FRONT - 0.648)
            ],
            0.006,
            M.cushion,
            verts = 5,
            resolution = 2
        );
        for s in [-1, 1] {
            cushion(
                &format!("back_cushion{s}"),
                0.745,
                0.4,
                0.15,
                (f64::from(s) * 0.376, 0.63, SB_BACK + 0.24),
                M.cushion,
                Cushion {
                    rot: P3::from((-9, 0, 0)),
                    puff: 0.028,
                    ..Cushion::default()
                },
            );
        }
        pillow(
            "pillow_l",
            -0.55,
            0.64,
            SB_BACK + 0.36,
            M.pillow_rose,
            10,
            0.4,
            0.34,
        );
        pillow(
            "pillow_r",
            0.55,
            0.64,
            SB_BACK + 0.36,
            M.pillow_cream,
            -10,
            0.4,
            0.34,
        );
        for x in [-0.86, 0.86] {
            for z in [SB_BACK + 0.06, SB_FRONT - 0.06] {
                leg(
                    &format!(
                        "leg{}{}",
                        artkit::json::float_repr(x),
                        artkit::json::float_repr(z)
                    ),
                    x,
                    z,
                    0.1,
                    M.legs,
                    0.02,
                    (0.0, 0.0),
                    10,
                );
            }
        }
    });
    join(&still, "sofa", Some((0, 0, 0).into()));
    // The mattress, built out: made up with a sheet turned down at its head, on a thin frame, a folding leg at its
    // foot (upholstered like the sofa: one finish, one draw call); its origin at its head end.
    let length = SB_FOOT - SB_HEAD;
    let bed = new_parts(|| {
        soft!(
            F,
            "mattress",
            1.44,
            0.1,
            length,
            (0, SB_MATTRESS - 0.05, (SB_HEAD + SB_FOOT) / 2.0),
            M.bedding,
            round = 0.045,
            flat = 2
        );
        let sheet = soft!(
            F,
            "sheet",
            1.47,
            0.008,
            length * 0.62,
            (0, SB_MATTRESS + 0.004, SB_FOOT - length * 0.31 - 0.01),
            M.bedding,
            round = 0.004,
            puff = 0.002,
            flat = 3,
            round_segments = 1
        );
        jitter!(sheet, 0.002, size = 0.2, seed = 0.7, up = 0.3);
        fbox!(
            F,
            "bed_frame",
            1.48,
            0.03,
            length,
            (0, SB_MATTRESS - 0.115, (SB_HEAD + SB_FOOT) / 2.0),
            M.upholstery,
            bevel = 0.01,
            bevel_segments = 1
        );
        // the folding leg: one bent tube, its feet on the floor
        fsweep!(
            F,
            "bed_leg",
            &fillet(
                &[
                    (-0.68, 0.0, SB_FOOT - 0.07),
                    (-0.68, SB_MATTRESS - 0.13, SB_FOOT - 0.07),
                    (0.68, SB_MATTRESS - 0.13, SB_FOOT - 0.07),
                    (0.68, 0.0, SB_FOOT - 0.07)
                ],
                0.05,
                3
            ),
            0.014,
            M.upholstery,
            verts = 8
        );
    });
    let mattress = join(&bed, "bed", Some(F.at(0, SB_MATTRESS - 0.13, SB_HEAD)));
    // Folded away at rest: its length shortened into the base, under the seat (Blender y is the depth).
    mattress.set_scale((1, SB_TUCKED, 1));
    settle!(centre = false); // against the wall, as built
    grow(
        mattress,
        "bed_out",
        Grow {
            seconds: 1.2,
            axis: Axis::Y,
        },
    );
    Vec::new()
}

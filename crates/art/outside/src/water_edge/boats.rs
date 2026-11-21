//! The rowing boat: a painted hull with a sheer line, a fitted transom, a cream-lined well, two thwarts,
//! floorboards and a pair of oars stowed across the thwarts.

use super::{F, M};
use artkit::bevel::{EdgeOrder, bevel_edges};
use artkit::forms::catmull;
use artkit::math::{P3, cos, sin, v3};
use artkit::parts::{join_all, settle_all};
use artkit::scene::Obj;
use artkit::shapes::SHARP;
use artkit::{fbox, fsweep, loft};

const WALL: f64 = 0.04; // the planking's thickness

/// The gunwale's height at game depth `z`: low amidships, sweeping up towards the bow and the stern.
fn rim(z: f64) -> f64 {
    let t = z.abs() / 1.5;
    0.56 + 0.11 * t * t * t
}

/// The hull's stations, bow at +z: `(z, half beam, depth below the bilge, bilge height)`.
const STATIONS: [(f64, f64, f64, f64); 9] = [
    (-1.42, 0.28, 0.20, 0.22),
    (-1.25, 0.38, 0.25, 0.26),
    (-1.00, 0.48, 0.29, 0.30),
    (-0.60, 0.60, 0.30, 0.30),
    (0.00, 0.64, 0.32, 0.32),
    (0.60, 0.58, 0.30, 0.30),
    (1.10, 0.42, 0.24, 0.24),
    (1.35, 0.24, 0.17, 0.19),
    (1.50, 0.07, 0.07, 0.15),
];

/// The stations with two more on a smooth curve between each pair, so the sheer and the bilge run fair rather
/// than kinking at each station.
fn stations() -> Vec<(f64, f64, f64, f64)> {
    let beam: Vec<_> = STATIONS
        .iter()
        .map(|&(z, rx, ry, _)| v3(z, rx, ry))
        .collect();
    let bilge: Vec<_> = STATIONS.iter().map(|&(z, _, _, cy)| v3(z, cy, 0)).collect();
    let (beam, _) = catmull(&beam, 3, false);
    let (bilge, _) = catmull(&bilge, 3, false);
    beam.iter()
        .zip(bilge.iter())
        .map(|(b, c)| (b.x as f64, b.y as f64, b.z as f64, c.y as f64))
        .collect()
}

/// One cross-section of the hull at `z`: the closed outline of the planking, so the loft leaves an open well.
/// `rx` across, `ry` deep below the turn of the bilge at `cy`; `wall` thick, topped at `top`.
fn section(rx: f64, ry: f64, cy: f64, z: f64, wall: f64, top: f64) -> Vec<P3> {
    let half = std::f64::consts::FRAC_PI_2;
    let t = wall.min(rx * 0.5);
    let (ix, iy) = (rx - t, ry - t);
    let mut p = vec![F.at(0.0, cy - ry, z)];
    // the right side's bottom, from the keel up to the turn of the bilge
    for k in 1..=5 {
        let a = half * k as f64 / 5.0;
        p.push(F.at(rx * sin(a), cy - ry * cos(a), z));
    }
    p.push(F.at(rx, top, z)); // the outer gunwale
    p.push(F.at(ix, top, z)); // the inner gunwale
    p.push(F.at(ix, cy, z)); // the inner side
    for k in (1..=4).rev() {
        let a = half * k as f64 / 5.0;
        p.push(F.at(ix * sin(a), cy - iy * cos(a), z));
    }
    for k in 1..=4 {
        let a = half * k as f64 / 5.0;
        p.push(F.at(-ix * sin(a), cy - iy * cos(a), z));
    }
    p.push(F.at(-ix, cy, z));
    p.push(F.at(-ix, top, z));
    p.push(F.at(-rx, top, z));
    p.push(F.at(-rx, cy, z));
    for k in (1..=4).rev() {
        let a = half * k as f64 / 5.0;
        p.push(F.at(-rx * sin(a), cy - ry * cos(a), z));
    }
    p
}

/// The outline of the open well at `z`, closed across the gunwales: the shape of the transom board.
fn well(rx: f64, ry: f64, cy: f64, z: f64, top: f64) -> Vec<P3> {
    let half = std::f64::consts::FRAC_PI_2;
    let (ix, iy) = (rx - WALL, ry - WALL);
    let mut p = vec![F.at(ix, top, z), F.at(ix, cy, z)];
    for k in (1..=4).rev() {
        let a = half * k as f64 / 5.0;
        p.push(F.at(ix * sin(a), cy - iy * cos(a), z));
    }
    for k in 1..=4 {
        let a = half * k as f64 / 5.0;
        p.push(F.at(-ix * sin(a), cy - iy * cos(a), z));
    }
    p.push(F.at(-ix, cy, z));
    p.push(F.at(-ix, top, z));
    p
}

pub fn rowing_boat() -> Vec<Obj> {
    let stations = stations();
    let hull: Vec<Vec<P3>> = stations
        .iter()
        .map(|&(z, rx, ry, cy)| section(rx, ry, cy, z, WALL, rim(z)))
        .collect();
    // not bevelled (the kernel's bevel can't follow the fair stations' edges): the gunwale is under the cap rail,
    // the stern behind the transom and the bow's end behind the stem
    loft!("hull", &hull, material = M.hull);
    let (z, rx, ry, cy) = STATIONS[STATIONS.len() - 1];
    fbox!(
        F,
        "stem",
        2.0 * rx + 0.02,
        rim(z) + 0.01 - (cy - ry),
        0.04,
        (0, (rim(z) + 0.01 + cy - ry) / 2.0, z + 0.015),
        M.seat,
        bevel = 0.008
    );
    // the cream liner inside the planking, stopping short of the bow's tip
    let liner: Vec<Vec<P3>> = stations
        .iter()
        .filter(|s| s.0 <= STATIONS[7].0 + 1e-9)
        .map(|&(z, rx, ry, cy)| {
            section(
                rx - WALL - 0.002,
                ry - WALL - 0.002,
                cy,
                z.max(-1.40),
                0.012,
                rim(z),
            )
        })
        .collect();
    let liner = loft!("liner", &liner, material = M.inside);
    bevel_edges(liner, 0.004, 2, SHARP, &EdgeOrder::Corners);
    // the transom, closing the stern and standing a centimetre proud of the planking all round (its end hidden)
    let (_, rx, ry, cy) = STATIONS[0];
    let (rx, ry) = (rx + WALL + 0.01, ry + WALL + 0.01);
    let transom = loft!(
        "transom",
        &[
            well(rx, ry, cy, -1.41, rim(-1.42)),
            well(rx, ry, cy, -1.43, rim(-1.42)),
        ],
        material = M.seat
    );
    bevel_edges(transom, 0.004, 2, SHARP, &EdgeOrder::Corners);
    // a wood cap rail along each gunwale
    for (i, s) in [-1.0_f64, 1.0].into_iter().enumerate() {
        let rail: Vec<(f64, f64, f64)> = stations
            .iter()
            .map(|&(z, rx, _, _)| (s * (rx - 0.02), rim(z) + 0.008, z))
            .collect();
        fsweep!(F, format!("rail{i}"), &rail, 0.017, M.seat, verts = 8);
    }
    // the floorboards in the well
    for (i, x) in [-0.15, 0.15].into_iter().enumerate() {
        fbox!(
            F,
            format!("floor{i}"),
            0.30,
            0.02,
            1.10,
            (x, 0.062, 0.0),
            M.seat,
            bevel = 0.004
        );
    }
    // two thwarts across the boat, dropped below the gunwale
    for (i, z) in [-0.45, 0.30].into_iter().enumerate() {
        fbox!(
            F,
            format!("thwart{i}"),
            1.13,
            0.05,
            0.26,
            (0.0, rim(z) - 0.075, z),
            M.seat,
            bevel = 0.006
        );
    }
    // the oars, stowed one along each side with their blades forward
    for (i, s) in [-1.0_f64, 1.0].into_iter().enumerate() {
        let (x0, x1) = (0.46 * s, 0.30 * s);
        fsweep!(
            F,
            format!("oar{i}"),
            &[(x0, 0.528, -0.62), (x1, 0.528, 0.58)],
            0.018,
            M.oar,
            verts = 8
        );
        fbox!(
            F,
            format!("blade{i}"),
            0.16,
            0.03,
            0.28,
            (x1 - 0.021 * s, 0.528, 0.717),
            M.oar,
            bevel = 0.006,
            rot = (0.0, -7.6 * s, 0.0)
        );
    }
    join_all("rowingBoat");
    settle_all()
}

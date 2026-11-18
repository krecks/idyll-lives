//! The butcher block on its legs with its knife drawer, and the marble counter.

use super::common::{cutlery_outline, doors, drawer, flat, plain, pull, unit79};
use super::storage::basket;
use super::turned::canister;
use super::{COUNTER66, M, SECONDS};
use artkit::forms::rounded_rect;
use artkit::parts::{join, new_parts, settle_all};
use artkit::place::{Coord, Facing, GameFrame};
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fextrude, fsweep, slide};

/// A knife insert standing at height `y`, its front at drawn depth `z`: a wooden tray with a chef's knife, a bread
/// knife, a carving knife and a paring knife lying in it, their handles black.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn knives(p: &GameFrame, name: &str, x: impl Coord, y: f64, z: f64, width: f64, depth: f64) {
    let zc = z - depth / 2.0;
    fbox!(
        p,
        format!("{name}_floor"),
        width,
        0.006,
        depth,
        (x, y + 0.003, zc),
        M.spoon
    );
    for i in 0..6 {
        fbox!(
            p,
            format!("{name}_rib{i}"),
            0.008,
            0.02,
            depth - 0.02,
            (
                x.f() - width / 2.0 + 0.04 + i as f64 * (width - 0.08) / 5.0,
                y + 0.01,
                zc
            ),
            M.spoon
        );
    }
    for (i, (k, dx)) in [(1.45, 0.048), (1.6, 0.144), (1.35, 0.24), (0.95, 0.336)]
        .into_iter()
        .enumerate()
    {
        let cx = x.f() - width / 2.0 + dx;
        flat(
            p,
            format!("{name}_blade{i}"),
            &cutlery_outline("knife", k),
            cx,
            y + 0.006,
            zc - 0.005,
            M.steel,
            0.0025,
        );
        fbox!(
            p,
            format!("{name}_grip{i}"),
            0.0145 * k,
            0.009,
            0.08 * k,
            (cx, y + 0.0105, zc - 0.005 + 0.046 * k),
            M.metal,
            bevel = 0.003,
            bevel_segments = 1
        );
    }
}

// --- the butcher block: 1x1, a block on legs with a shelf, a basket and a knife drawer ----------------
pub fn butcher_block() -> Vec<Obj> {
    let f = &GameFrame::new(-0.0055);
    let still = new_parts(|| {
        // the block, its sides showing the staves it is glued up from, and the top layer of staves
        fbox!(
            f,
            "top",
            0.9,
            0.10,
            0.8,
            (0, 0.88, 0),
            M.block,
            bevel = 0.012
        );
        for i in 0..12 {
            fbox!(
                f,
                format!("top_layer{i}"),
                0.07,
                0.035,
                0.74,
                (-0.385 + 0.07 * i as f64, 0.9325, 0),
                M.block2,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        // rounded legs, aprons round the drawer, stretchers carrying the slatted shelf
        let leg = rounded_rect(0.07, 0.07, 0.012, 2);
        for sx in [-1, 1] {
            for sz in [-1, 1] {
                fextrude!(
                    f,
                    format!("leg{sx}{sz}"),
                    &leg,
                    0.83,
                    (sx as f64 * 0.41, 0.415, sz as f64 * 0.36),
                    M.leg,
                    plane = Facing::Top,
                    bevel = 0.004,
                    bevel_segments = 1
                );
            }
            fbox!(
                f,
                format!("apron_side{sx}"),
                0.022,
                0.18,
                0.65,
                (sx as f64 * 0.395, 0.74, 0),
                M.leg,
                bevel = 0.003,
                bevel_segments = 1
            );
            fbox!(
                f,
                format!("stretcher{sx}"),
                0.03,
                0.04,
                0.65,
                (sx as f64 * 0.41, 0.17, 0),
                M.leg,
                bevel = 0.004,
                bevel_segments = 1
            );
            fbox!(
                f,
                format!("apron_front{sx}"),
                0.03,
                0.18,
                0.022,
                (sx as f64 * 0.362, 0.74, 0.374),
                M.leg,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "apron_back",
            0.75,
            0.18,
            0.022,
            (0, 0.74, -0.374),
            M.leg,
            bevel = 0.003,
            bevel_segments = 1
        );
        for (i, z) in [-0.24, -0.08, 0.08, 0.24].into_iter().enumerate() {
            fbox!(
                f,
                format!("slat{i}"),
                0.8,
                0.025,
                0.12,
                (0, 0.2, z),
                M.leg,
                bevel = 0.004
            );
        }
        // a woven basket and a stoneware crock on the shelf
        basket(f, "basket", 0.4, 0.2, 0.3, -0.16, 0.2125, 0, None, 16);
        canister(
            f,
            "box",
            0.22,
            0.2125,
            0.06,
            0.085,
            0.17,
            M.porcelain,
            M.spoon,
            16,
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // the knife drawer under the block, sliding out for a sandwich: its front in the apron, a long dark pull
    let zf = 0.40;
    let parts = new_parts(|| {
        fbox!(
            f,
            "drawer_front",
            0.7,
            0.17,
            0.02,
            (0, 0.72, zf - 0.01),
            M.wood2,
            bevel = 0.006,
            bevel_segments = 2
        );
        pull(
            f,
            "drawer_pull",
            0,
            0.72,
            zf,
            0.56,
            Some(M.metal),
            false,
            0.019,
            0.005,
            0.0,
        );
        fbox!(
            f,
            "drawer_bottom",
            0.64,
            0.012,
            0.5,
            (0, 0.646, zf - 0.27),
            M.wood2
        );
        for s in [-1, 1] {
            fbox!(
                f,
                format!("drawer_side{s}"),
                0.014,
                0.12,
                0.5,
                (s as f64 * 0.313, 0.7, zf - 0.27),
                M.wood2,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        fbox!(
            f,
            "drawer_back",
            0.64,
            0.12,
            0.014,
            (0, 0.7, zf - 0.513),
            M.wood2,
            bevel = 0.003,
            bevel_segments = 1
        );
        knives(f, "drawer_knives", 0, 0.652, zf - 0.04, 0.56, 0.3);
    });
    let drawer = join(&parts, "drawer", Some(f.at(0, 0.72, zf - 0.01)));
    settle_all();
    slide!(drawer, "drawer_open", (0, -0.25, 0), seconds = SECONDS);
    meshes()
}

// --- the marble counter: 1x1, a navy unit with a marble top and brass handles, a drawer for sandwich --
pub fn counter_marble() -> Vec<Obj> {
    let f = &GameFrame::with_depths(0.085, &COUNTER66);
    let (p, front) = (&plain(f), f.depth(0.24));
    let still = new_parts(|| {
        unit79(f, 1.0, 0.0, Some(M.navy), Some(M.marble), true);
        doors(
            p,
            0.97,
            front,
            0.12,
            0.72,
            Some(M.navy),
            Some(M.brass),
            Some(0.62),
            0.0,
            0.019,
        );
        // veins wandering through the marble, thin and fading at their ends
        let vein = rounded_rect(0.0016, 0.012, 0.0007, 1);
        let veins: [&[(f64, f64)]; 3] = [
            &[
                (-0.42, -0.36),
                (-0.25, -0.2),
                (-0.12, -0.12),
                (0.05, 0.02),
                (0.2, 0.16),
                (0.3, 0.27),
            ],
            &[
                (0.05, -0.4),
                (0.12, -0.22),
                (0.2, -0.12),
                (0.34, 0.0),
                (0.46, 0.1),
            ],
            &[(-0.46, 0.05), (-0.33, 0.12), (-0.22, 0.2), (-0.12, 0.26)],
        ];
        for (i, pts) in veins.into_iter().enumerate() {
            let path: Vec<(f64, f64, f64)> = pts.iter().map(|&(x, z)| (x, 0.9505, z)).collect();
            let mut scale = vec![0.3];
            scale.extend((0..pts.len() - 2).map(|k| 1.0 - 0.25 * (k % 2) as f64));
            scale.push(0.3);
            fsweep!(
                f,
                format!("vein{i}"),
                &path,
                0.0,
                M.vein,
                profile = &vein,
                resolution = 3,
                smooth = false,
                scale = scale
            );
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let drawer = drawer(
        p,
        "drawer",
        0.97,
        0.815,
        front + 0.02,
        M.navy,
        Some(M.brass),
        0.18,
        true,
        0.019,
        0.04,
        0.17,
    );
    settle_all();
    slide!(drawer, "drawer_open", (0, -0.25, 0), seconds = SECONDS);
    meshes()
}

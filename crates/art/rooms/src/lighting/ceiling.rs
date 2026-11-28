//! LT1: the ceiling lights. The two pendants residents walk under hang clear of their heads: their lowest point
//! 2.1 m up (a room's 2.8 m ceiling, a 0.7 m drop; the reference's least clearance where people walk). The
//! chandelier and the industrial pendant hang lower, over a table or a sofa only (content `headHeight`, Q-12).

use super::common::{
    CEILING, DOME, F, Lathe, M, bulb, canopy, check_at, hang, lathe, loft_at, pear, ring_at, socket,
};
use artkit::kit;
use artkit::math::{cosd, sind};
use artkit::parts::{join_all, new_parts};
use artkit::scene::Obj;
use artkit::{fbox, fsphere, settle, torus};

/// The white dome on a brass socket and a black cord: a rolled lip round the dome, a brass collar on its crown, a
/// pear bulb showing below.
pub fn ceiling_pendant() -> Vec<Obj> {
    new_parts(|| {
        hang("ceilingPendant", 2.4, None, 0.06, None);
        socket("socket", 0.026, 0.05, (0, 2.35, 0), M.brass, 16);
        lathe(
            "dome",
            0.43,
            0.24,
            (0, 2.24, 0),
            &DOME,
            M.dome,
            Lathe {
                segments: 40,
                ..Lathe::default()
            },
        );
        ring_at("dome_lip", 0.209, (0, 2.124, 0), 0.006, M.dome, 40, 6);
        kit::turned!(
            F,
            "collar",
            0,
            2.344,
            0,
            &[
                (0.0, 0.0),
                (0.064, 0.0),
                (0.064, 0.004),
                (0.052, 0.012),
                (0.034, 0.018),
                (0.0, 0.018),
            ],
            M.brass,
            segments = 24
        );
        pear("ceilingPendant", 0.04, (0, 2.162, 0), 14);
    });
    join_all("ceilingPendant");
    settle!(floor = false);
    check_at("ceilingPendant")
}

/// A square lantern on a short stem: four frosted sides, each parted in four panes by black glazing bars, between
/// black corner posts, under a hipped lid with a knob; a diffuser below.
pub fn ceiling_light() -> Vec<Obj> {
    new_parts(|| {
        loft_at(
            "canopy",
            0,
            0,
            &[
                (2.772, 0.052, 0.052, 6.0),
                (2.785, 0.062, 0.062, 8.0),
                (2.797, 0.062, 0.062, 8.0),
                (2.8, 0.059, 0.059, 8.0), // its rim on the ceiling eased
            ],
            M.iron,
            16,
        );
        kit::tube!(
            F,
            "stem",
            &[(0, 2.772, 0), (0, 2.555, 0)],
            0.009,
            M.iron,
            verts = 8,
            round = 1
        );
        kit::turned!(
            F,
            "stem_collar",
            0,
            2.548,
            0,
            &[
                (0.0, 0.0),
                (0.016, 0.0),
                (0.016, 0.012),
                (0.01, 0.02),
                (0.0, 0.02),
            ],
            M.iron,
            segments = 12
        );
        // the hipped lid, curving up from its eaves to a knob
        loft_at(
            "lid",
            0,
            0,
            &[
                (2.5, 0.18, 0.18, 14.0),
                (2.512, 0.18, 0.18, 14.0),
                (2.515, 0.177, 0.177, 13.0), // the eaves eased
                (2.518, 0.168, 0.168, 12.0),
                (2.53, 0.12, 0.12, 9.0),
                (2.542, 0.065, 0.065, 6.0),
                (2.55, 0.03, 0.03, 4.0),
            ],
            M.iron,
            24,
        );
        for s in [-1, 1] {
            let sf = s as f64;
            fbox!(
                F,
                format!("side_z{s}"),
                0.32,
                0.29,
                0.006,
                (0, 2.355, sf * 0.163),
                M.panel
            );
            fbox!(
                F,
                format!("side_x{s}"),
                0.006,
                0.29,
                0.32,
                (sf * 0.163, 2.355, 0),
                M.panel
            );
            for t in [-1, 1] {
                fbox!(
                    F,
                    format!("post{s}{t}"),
                    0.016,
                    0.31,
                    0.016,
                    (sf * 0.17, 2.355, t as f64 * 0.17),
                    M.iron,
                    bevel = 0.004,
                    bevel_segments = 1
                );
            }
            // the glazing bars across each side: a cross over the frosted pane, and a rail under the lid
            fbox!(
                F,
                format!("bar_z{s}"),
                0.32,
                0.008,
                0.008,
                (0, 2.355, sf * 0.168),
                M.iron
            );
            fbox!(
                F,
                format!("mullion_z{s}"),
                0.008,
                0.29,
                0.008,
                (0, 2.355, sf * 0.168),
                M.iron
            );
            fbox!(
                F,
                format!("bar_x{s}"),
                0.008,
                0.008,
                0.32,
                (sf * 0.168, 2.355, 0),
                M.iron
            );
            fbox!(
                F,
                format!("mullion_x{s}"),
                0.008,
                0.29,
                0.008,
                (sf * 0.168, 2.355, 0),
                M.iron
            );
            fbox!(
                F,
                format!("head_z{s}"),
                0.34,
                0.016,
                0.01,
                (0, 2.494, sf * 0.168),
                M.iron,
                bevel = 0.003,
                bevel_segments = 1
            );
            fbox!(
                F,
                format!("head_x{s}"),
                0.01,
                0.016,
                0.34,
                (sf * 0.168, 2.494, 0),
                M.iron,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        fbox!(
            F,
            "rim",
            0.36,
            0.012,
            0.36,
            (0, 2.206, 0),
            M.iron,
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(F, "diffuser", 0.32, 0.004, 0.32, (0, 2.214, 0), M.bulb);
        bulb(
            "ceilingLight",
            0.05,
            (0, 2.32, 0),
            None,
            (1.0, 1.0, 1.2),
            14,
            8,
        );
    });
    join_all("ceilingLight");
    settle!(floor = false);
    check_at("ceilingLight")
}

/// A turned brass column on a rod, six S-scrolled arms down to one ring carrying six globes, each in a cup; a drop
/// globe and a finial under the column.
pub fn chandelier() -> Vec<Obj> {
    new_parts(|| {
        canopy("canopy", 0.08, 0.03, M.brass, 16, CEILING);
        kit::tube!(
            F,
            "rod",
            &[(0, CEILING - 0.03, 0), (0, 2.09, 0)],
            0.005,
            M.brass,
            verts = 8,
            round = 1
        );
        torus!(
            "loop",
            0.016,
            0.004,
            at = F.at(0, 2.515, 0),
            material = M.brass,
            rot = (90, 0, 0),
            major_segments = 12,
            minor_segments = 4
        );
        // the column, turned, its widest bulge the hub the arms spring from
        kit::turned!(
            F,
            "column",
            0,
            2.28,
            0,
            &[
                (0.0, 0.0),
                (0.01, 0.0),
                (0.02, 0.02),
                (0.028, 0.05),
                (0.03, 0.07),
                (0.026, 0.09),
                (0.012, 0.12),
                (0.009, 0.16),
                (0.014, 0.19),
                (0.008, 0.22),
                (0.0, 0.22),
            ],
            M.brass,
            segments = 12
        );
        kit::turned!(
            F,
            "collar",
            0,
            2.204,
            0,
            &[(0.0, 0.0), (0.014, 0.0), (0.014, 0.012), (0.0, 0.012)],
            M.brass,
            segments = 10
        );
        kit::turned!(
            F,
            "finial",
            0,
            2.04,
            0,
            &[
                (0.0, 0.0),
                (0.004, 0.0),
                (0.012, 0.012),
                (0.016, 0.024),
                (0.008, 0.036),
                (0.006, 0.05),
                (0.0, 0.05),
            ],
            M.brass,
            segments = 10
        );
        ring_at("ring_low", 0.435, (0, 2.05, 0), 0.013, M.brass, 36, 4);
        for a in [0, 60, 120, 180, 240, 300] {
            let (c, s) = (cosd(a as f64), sind(a as f64));
            let pts: Vec<(f64, f64, f64)> =
                [(0.02, 2.32), (0.16, 2.30), (0.3, 2.2), (0.435, 2.055)]
                    .iter()
                    .map(|&(r, y)| (r * c, y, r * s))
                    .collect();
            kit::tube!(
                F,
                format!("arm{a}"),
                &pts,
                0.007,
                M.brass,
                verts = 6,
                round = 4
            );
            // a cup on the ring and its globe
            let (x, z) = (0.435 * c, 0.435 * s);
            kit::turned!(
                F,
                format!("cup{a}"),
                x,
                2.051,
                z,
                &[(0.0, 0.0), (0.02, 0.012), (0.02, 0.017), (0.0, 0.017)],
                M.brass,
                segments = 8
            );
            fsphere!(
                F,
                format!("globe{a}"),
                0.035,
                (x, 2.1, z),
                M.bulb,
                segments = 8,
                rings = 6
            );
        }
        bulb(
            "chandelier",
            0.03,
            (0, 2.12, 0),
            None,
            (1.0, 1.0, 1.0),
            10,
            6,
        ); // a drop under the hub
    });
    join_all("chandelier");
    settle!(floor = false);
    check_at("chandelier")
}

/// A green enamel dome, white inside, a rolled rim, on a black fitter and a ribbed socket: an Edison bulb shows
/// below.
pub fn pendant_industrial() -> Vec<Obj> {
    new_parts(|| {
        hang("pendantIndustrial", 2.24, Some(M.iron), 0.06, None);
        socket("socket", 0.026, 0.05, (0, 2.205, 0), M.iron, 16);
        kit::turned!(
            F,
            "fitter",
            0,
            2.18,
            0,
            &[
                (0.0, 0.0),
                (0.058, 0.0),
                (0.058, 0.006),
                (0.044, 0.012),
                (0.032, 0.026),
                (0.03, 0.03),
                (0.0, 0.03),
            ],
            M.iron,
            segments = 24
        );
        lathe(
            "dome",
            0.38,
            0.24,
            (0, 2.07, 0),
            &DOME,
            M.industrial,
            Lathe {
                segments: 40,
                liner: Some(M.liner),
                ..Lathe::default()
            },
        );
        ring_at("dome_rim", 0.185, (0, 1.954, 0), 0.006, M.industrial, 40, 6);
        pear("pendantIndustrial", 0.045, (0, 1.975, 0), 16);
    });
    join_all("pendantIndustrial");
    settle!(floor = false);
    check_at("pendantIndustrial")
}

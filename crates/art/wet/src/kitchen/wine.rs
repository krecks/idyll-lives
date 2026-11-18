//! The wine fridge under a stone worktop: a smoked-glass door over racks of bottles lying necks out, three drawers
//! beside it, its top rack sliding out once the door is open.

use super::M;
use super::common::{gasket, kick};
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::P3;
use artkit::motion::Ease;
use artkit::parts::{hinged, join, new_parts, settle_all};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{fbox, fextrude, loft, ring, slide, swing};

/// A wine cooler under a stone worktop: a smoked-glass door in a dark frame over five shelves of bottles lying
/// necks out on wood-fronted racks, a stack of three drawers beside it, a vented kick plate; once the door is open,
/// the top shelf of bottles slides out to the front. Drawn in the counters' 0.66 m depth (its worktop -0.43..0.23).
pub fn wine_fridge() -> Vec<Obj> {
    let fw = GameFrame::new(0.0);
    let still = new_parts(|| {
        fbox!(
            fw,
            "plinth",
            0.996,
            0.10,
            0.55,
            (0, 0.05, -0.155),
            M.dark,
            bevel = 0.004
        );
        kick(&fw, "kick", 0.9, 0.06, 0.06, 0.125, Some(M.seal), 9);
        for s in [-1, 1] {
            fbox!(
                fw,
                format!("side{s}"),
                0.03,
                0.80,
                0.60,
                (s as f64 * 0.483, 0.50, -0.13),
                M.panel,
                bevel = 0.004
            );
        }
        fbox!(fw, "bottom", 0.94, 0.03, 0.60, (0, 0.115, -0.13), M.panel);
        fbox!(fw, "back", 0.94, 0.80, 0.03, (0, 0.50, -0.415), M.panel);
        fbox!(
            fw,
            "partition",
            0.02,
            0.76,
            0.58,
            (0.07, 0.50, -0.12),
            M.panel
        );
        fbox!(
            fw,
            "ceiling",
            0.54,
            0.02,
            0.58,
            (-0.2, 0.89, -0.12),
            M.panel
        ); // closes the gap over the door
        fbox!(
            fw,
            "top",
            1.00,
            0.05,
            0.66,
            (0, 0.925, -0.10),
            M.stone,
            bevel = 0.012,
            bevel_segments = 2
        );
        // the cellar: a pale liner, four racks of bottles (the top one slides out: below)
        fbox!(
            fw,
            "liner",
            0.52,
            0.74,
            0.008,
            (-0.2, 0.50, -0.395),
            M.liner
        );
        fbox!(
            fw,
            "liner_side",
            0.008,
            0.74,
            0.56,
            (-0.463, 0.50, -0.12),
            M.liner
        );
        for (r, &y) in WINE_ROWS[..WINE_ROWS.len() - 1].iter().enumerate() {
            wine_row(&fw, &format!("rack_left{r}"), y, r);
        }
        // three drawers beside it, flush with the door, each with a long pull
        for r in 0..3 {
            let y = 0.2445 + 0.255 * r as f64;
            fbox!(
                fw,
                format!("rack_right{r}"),
                0.404,
                0.249,
                0.02,
                (0.276, y, 0.19),
                M.panel,
                bevel = 0.005,
                bevel_segments = 1
            );
            pull!(
                &fw,
                format!("rack_handle{r}"),
                (0.18, y + 0.07),
                (0.372, y + 0.07),
                0.2,
                out = 0.026,
                w = 0.014,
                r = None,
                verts = 6
            );
        }
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // the door, hinged on its left edge: a dark frame round a pane of smoked glass the bottles show through, a small
    // display on its top rail, a bar handle on its free edge and a seal round its back
    let parts = new_parts(|| {
        fextrude!(
            fw,
            "frame",
            &rounded_rect(0.545, 0.76, 0.012, 2),
            0.02,
            (-0.2075, 0.50, 0.19),
            M.doorframe,
            holes = [rounded_rect(0.455, 0.64, 0.008, 2)],
            bevel = 0.003,
            bevel_segments = 1
        );
        fbox!(
            fw,
            "glass",
            0.463,
            0.648,
            0.006,
            (-0.2075, 0.50, 0.188),
            M.smoked
        );
        fbox!(
            fw,
            "display",
            0.08,
            0.018,
            0.004,
            (-0.2, 0.85, 0.201),
            M.water
        );
        pull!(
            &fw,
            "door_handle",
            (0.035, 0.30),
            (0.035, 0.70),
            0.2,
            out = 0.03,
            w = 0.016,
            r = None,
            verts = 6
        );
        gasket(
            &fw,
            "door_seal",
            0.52,
            0.73,
            0.50,
            0.178,
            0.005,
            0.02,
            -0.2075,
        );
    });
    let door = hinged(join(&parts, "door", None), fw.at(-0.49, 0.50, 0.19));
    // the top shelf of bottles, sliding out 15 cm once the door is open
    let parts = new_parts(|| {
        wine_row(
            &fw,
            "wine_rack",
            WINE_ROWS[WINE_ROWS.len() - 1],
            WINE_ROWS.len() - 1,
        );
    });
    let rack = join(
        &parts,
        "wine_rack",
        Some(fw.at(-0.22, WINE_ROWS[WINE_ROWS.len() - 1], -0.14)),
    );
    settle_all();
    swing!(door, "door_open", 100, seconds = 1.5, ease = Ease::BackOut);
    slide!(rack, "rack_out", (0, -0.15, 0), seconds = 0.8);
    Vec::new()
}

const WINE_ROWS: [f64; 5] = [0.22, 0.35, 0.48, 0.61, 0.74]; // the wine fridge's shelves

/// A wine bottle lying on its side at game (x, y: its lowest point, z: its base), its neck to the front: a body,
/// a shoulder, a neck and a foil capsule.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn wine_bottle(fw: &GameFrame, name: &str, x: f64, y: f64, z: f64, material: Mat, foil: Mat) {
    let (r, h) = (0.036, 0.30);
    let rings = [
        (0.0, 0.94),
        (0.6, 1.0),
        (0.72, 0.78),
        (0.8, 0.4),
        (0.86, 0.36),
        (0.861, 0.4),
        (1.0, 0.4),
    ];
    let sections = |rs: &[(f64, f64)]| -> Vec<Vec<P3>> {
        rs.iter()
            .map(|&(t, k)| ring!(r * k, r * k, z = h * t, verts = 8))
            .collect()
    };
    loft!(
        name,
        &sections(&rings[..5]),
        material = material,
        at = fw.at(x, y + r, z),
        rot = (90, 0, 0)
    );
    loft!(
        format!("{name}_foil"),
        &sections(&rings[5..]),
        material = foil,
        at = fw.at(x, y + r, z),
        rot = (90, 0, 0)
    );
}

/// A shelf of the wine fridge, its front a wooden lip, with three bottles lying on it, necks to the front.
fn wine_row(fw: &GameFrame, name: &str, y: f64, r: usize) {
    fbox!(fw, name, 0.47, 0.01, 0.50, (-0.215, y, -0.14), M.shelf);
    fbox!(
        fw,
        format!("{name}_lip"),
        0.45,
        0.03,
        0.016,
        (-0.215, y + 0.008, 0.09),
        M.board,
        bevel = 0.004,
        bevel_segments = 1
    );
    for (c, x) in [-0.38, -0.215, -0.05].into_iter().enumerate() {
        let red = (r + c).is_multiple_of(2);
        wine_bottle(
            fw,
            &format!("{name}_bottle{c}"),
            x,
            y + 0.005,
            -0.27,
            if red { M.wine } else { M.wine2 },
            if red { M.produce } else { M.gold },
        );
    }
}

//! The toilets: a bowl, a seat and a cistern; the lid lifts while a resident uses it.
//!
//! Built where the game places it, the cistern against the wall face (game depth -0.43), not centred: the game
//! sits the resident at the footprint's middle (characters.ts SEATS, the content's `seat`), so the bowl must stay
//! where the seat is. The seat's top is the seat height. The lid is the moving part, hinged at its back edge;
//! `lid_up` lifts it 100° to lean on the cistern (or over the smart seat's housing). The bowl, its seat and lid
//! are egg-shaped from above (`EGG`: narrower to the front), as an elongated bowl is.

use super::M;
use super::common::{LATHE, Lathe, WALL, lathe, rose, tube, wall_rose};
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::math::{Axis, TAU, cos, sin};
use artkit::parts::{hinged, join, new_parts};
use artkit::place::GameFrame;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fsphere, fsweep, settle, soft, swing};

pub const EGG: f64 = 0.1;

/// A ring of the bowl's shape (a `lathe` ring): `r` across (half), `ratio` times that deep.
fn egg(r: f64, ratio: f64, y: f64, rx_k: f64) -> (f64, f64, f64) {
    (y, r * rx_k, r * ratio * rx_k)
}

/// The bowl in one shell, egg-shaped: up its outside to a rounded rim and down the inside to the water.
fn bowl(f: &GameFrame, zb: f64, base_y: f64, rim: f64, r: f64, ratio: f64, water_y: f64) {
    let h = rim - base_y;
    let rings = [
        (base_y, r * 0.5),
        (base_y + h * 0.3, r * 0.78),
        (base_y + h * 0.7, r * 0.95),
        (rim - 0.018, r),
        (rim - 0.004, r - 0.001),
        (rim, r - 0.006),
        (rim, r - 0.03),
        (rim - 0.012, r - 0.04),
        (rim - 0.03, r - 0.05),
        (rim - h * 0.45, r * 0.62),
        (water_y + 0.02, r * 0.4),
        (water_y, r * 0.3),
    ];
    let shell: Vec<(f64, f64, f64)> = rings.iter().map(|&(y, k)| (y, k, k * ratio)).collect();
    lathe(
        f,
        "bowl",
        0.0,
        zb,
        &shell,
        M.porcelain,
        Lathe {
            verts: 32,
            egg: EGG,
            angle: 50.0,
            ..LATHE
        },
    );
    lathe(
        f,
        "bowl_water",
        0.0,
        zb,
        &[
            egg(r, ratio, water_y + 0.004, 0.36),
            egg(r, ratio, water_y + 0.008, 0.36),
        ],
        M.water,
        Lathe {
            verts: 20,
            egg: EGG,
            ..LATHE
        },
    );
}

/// The bowl's foot on the floor, waisted, flaring into the bowl's underside (`flare` widens its plinth), with a
/// bolt cap each side.
fn foot(f: &GameFrame, z: f64, top: f64, r: f64, ratio: f64, flare: f64) {
    let rings = [
        (0.0, flare),
        (0.012, flare),
        (0.03, 0.9 + 0.06 * flare),
        (top * 0.45, 0.8),
        (top * 0.8, 0.88),
        (top, 0.95),
    ];
    let shell: Vec<(f64, f64, f64)> = rings
        .iter()
        .map(|&(y, k)| (y, r * k, r * ratio * k))
        .collect();
    lathe(
        f,
        "foot",
        0.0,
        z,
        &shell,
        M.porcelain,
        Lathe {
            verts: 32,
            egg: EGG * 0.5,
            angle: 50.0,
            ..LATHE
        },
    );
    for s in [-1, 1] {
        lathe(
            f,
            format!("bolt_cap{s}"),
            s as f64 * (r * flare + 0.006),
            z,
            &[
                (0.0, 0.016, 0.016),
                (0.012, 0.013, 0.013),
                (0.02, 0.006, 0.006),
            ],
            M.porcelain,
            Lathe { verts: 8, ..LATHE },
        );
    }
}

/// The porcelain behind the bowl (the deck a cistern stands on, a wall-hung bowl's fixing): a rounded block from
/// game depth `z0` to `z1`, `w` wide, its edges rolled.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn column(f: &GameFrame, name: &str, y0: f64, y1: f64, z0: f64, z1: f64, w: f64) {
    let d = (z1 - z0) / 2.0;
    lathe(
        f,
        name,
        0.0,
        (z0 + z1) / 2.0,
        &[
            (y0, w / 2.0 - 0.03, d - 0.03, 3.0),
            (y0 + 0.012, w / 2.0 - 0.008, d - 0.008, 3.5),
            (y0 + 0.03, w / 2.0, d, 4.0),
            (y1 - 0.03, w / 2.0, d, 4.0),
            (y1 - 0.008, w / 2.0 - 0.008, d - 0.008, 3.5),
            (y1, w / 2.0 - 0.03, d - 0.03, 3.0),
        ],
        M.porcelain,
        Lathe {
            verts: 28,
            angle: 50.0,
            ..LATHE
        },
    );
}

/// Game points round the bowl's egg at height `y`, `r` across (half).
fn ring_path(zb: f64, y: f64, r: f64, ratio: f64, verts: usize) -> Vec<(f64, f64, f64)> {
    let mut out = Vec::new();
    for i in 0..verts {
        let a = TAU * i as f64 / verts as f64;
        let v = sin(a);
        out.push((r * cos(a) * (1.0 - EGG * v), y, zb + r * ratio * v));
    }
    out
}

/// The seat ring on the rim, from `y` up `t`, its outside `r` (half) wide: a rounded section swept round the
/// egg, with two rubber buffers under it.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn seat(f: &GameFrame, zb: f64, y: f64, r: f64, ratio: f64, material: Mat, t: f64) {
    let m = 0.034;
    fsweep!(
        f,
        "seat",
        &ring_path(zb, y + t / 2.0, r - m, ratio, 32),
        0.0,
        material,
        closed = true,
        profile = rounded_rect(t, 2.0 * m, t * 0.48, 2),
        angle = 70
    );
    for s in [-1, 1] {
        fcyl!(
            f,
            format!("buffer{s}"),
            0.008,
            0.006,
            (s as f64 * (r - m) * 0.92, y - 0.002, zb + 0.06),
            M.trim,
            verts = 8
        );
    }
}

/// The lid over the seat, from `y` up `t`, domed, hinged across its back edge: on its underside, or on its top
/// (`top`, where it lifts over a housing behind it). Returns it, ready to swing about X.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn lid(
    f: &GameFrame,
    zb: f64,
    y: f64,
    r: f64,
    ratio: f64,
    material: Mat,
    t: f64,
    top: bool,
) -> Obj {
    let parts = new_parts(|| {
        lathe(
            f,
            "lid",
            0.0,
            zb,
            &[
                egg(r, ratio, y, 0.985),
                egg(r, ratio, y + t * 0.3, 1.0),
                egg(r, ratio, y + t * 0.75, 0.97),
                egg(r, ratio, y + t, 0.88),
            ],
            material,
            Lathe {
                verts: 32,
                egg: EGG,
                angle: 60.0,
                ..LATHE
            },
        );
    });
    let lid = join(&parts, "lid", None);
    hinged(
        lid,
        f.at(0, y + (if top { t } else { 0.0 }), zb - r * ratio),
    )
}

/// The seat's two hinges on the bowl's back deck: a round knuckle on a domed post each.
fn hinges(f: &GameFrame, z: f64, y: f64, material: Mat, x: f64) {
    for s in [-1, 1] {
        lathe(
            f,
            format!("hinge_post{s}"),
            s as f64 * x,
            z - 0.004,
            &[
                (y - 0.002, 0.016, 0.016),
                (y + 0.006, 0.014, 0.014),
                (y + 0.012, 0.009, 0.009),
            ],
            material,
            Lathe { verts: 10, ..LATHE },
        );
        tube(
            f,
            format!("hinge{s}"),
            &[
                (s as f64 * x - 0.02, y + 0.014, z),
                (s as f64 * x + 0.02, y + 0.014, z),
            ],
            0.011,
            material,
            10,
            1,
            1.0,
        );
    }
}

/// A toilet roll on a chrome holder out from the wall (or the panel) at game depth `z`; `side` is the end the
/// arm comes from (the roll runs along x). Its loose end hangs down the front.
fn roll(f: &GameFrame, x: f64, y: f64, z: f64, side: i32) {
    let ax = x - side as f64 * 0.06;
    wall_rose(f, "roll_rose", ax, y, z, 0.024, M.chrome, 0.01, 14);
    tube(
        f,
        "roll_arm",
        &[
            (ax, y, z + 0.004),
            (ax, y, z + 0.065),
            (x + side as f64 * 0.055, y, z + 0.065),
        ],
        0.0055,
        M.chrome,
        8,
        2,
        1.0,
    );
    fcyl!(
        f,
        "roll",
        0.055,
        0.1,
        (x, y, z + 0.065),
        M.paper,
        rot = (0, 90, 0),
        verts = 18
    );
    fcyl!(
        f,
        "roll_core",
        0.02,
        0.102,
        (x, y, z + 0.065),
        M.dark,
        rot = (0, 90, 0),
        verts = 10
    );
    fbox!(
        f,
        "roll_sheet",
        0.098,
        0.07,
        0.002,
        (x, y - 0.035, z + 0.065 + 0.0555),
        M.paper
    );
}

/// A cistern from the wall to game depth `cz`, `cw` wide and `ch` tall from `y0`: a rounded block whose corners
/// soften towards its foot.
fn cistern(f: &GameFrame, cw: f64, ch: f64, y0: f64, cz: f64, material: Mat) {
    let d = (cz - WALL) / 2.0;
    let zc = (WALL + cz) / 2.0;
    lathe(
        f,
        "cistern",
        0.0,
        zc,
        &[
            (y0, cw / 2.0 - 0.02, d - 0.02, 4.0),
            (y0 + 0.02, cw / 2.0 - 0.004, d - 0.004, 5.0),
            (y0 + 0.05, cw / 2.0, d, 6.0),
            (y0 + ch - 0.012, cw / 2.0, d, 6.0),
            (y0 + ch, cw / 2.0 - 0.008, d - 0.008, 6.0),
        ],
        material,
        Lathe {
            verts: 32,
            angle: 50.0,
            ..LATHE
        },
    );
}

/// The cistern's lid, a little proud of it all round: a soft slab, and a raised step on it (`step`).
fn cistern_lid(f: &GameFrame, w: f64, y: f64, d: f64, material: Mat, t: f64, step: bool) {
    soft!(
        f,
        "cistern_lid",
        w,
        t,
        d,
        (0, y + t / 2.0, WALL + d / 2.0),
        material,
        round = 0.01,
        round_segments = 1
    );
    if step {
        soft!(
            f,
            "cistern_cap",
            w - 0.04,
            0.02,
            d - 0.035,
            (0, y + t + 0.01, WALL + d / 2.0 - 0.004),
            material,
            round = 0.008,
            round_segments = 1
        );
    }
}

/// A flush lever on the cistern's front at game depth `z`: a round boss and an arm towards the middle.
fn lever(f: &GameFrame, x: f64, y: f64, z: f64, material: Mat, ball: bool) {
    wall_rose(f, "lever_boss", x, y, z, 0.02, material, 0.012, 12);
    tube(
        f,
        "lever",
        &[
            (x, y, z + 0.008),
            (x, y, z + 0.016),
            (x + 0.04, y - 0.004, z + 0.016),
            (x + 0.075, y - 0.012, z + 0.014),
        ],
        0.006,
        material,
        8,
        2,
        [1.0, 1.0, 0.9, 0.8],
    );
    if ball {
        fsphere!(
            f,
            "lever_end",
            0.013,
            (x + 0.08, y - 0.013, z + 0.014),
            material,
            segments = 8,
            rings = 5
        );
    }
}

/// The bowl on its foot, the back deck from the bowl to the wall, the seat, and a cistern `cw` wide, `ch` tall,
/// from the wall to game depth `cz`, standing on the deck.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn close_coupled(
    f: &GameFrame,
    zb: f64,
    rim: f64,
    r: f64,
    ratio: f64,
    cz: f64,
    cw: f64,
    ch: f64,
    seat_material: Mat,
    foot_flare: f64,
) {
    foot(f, zb - 0.04, 0.26, 0.115, 1.3, foot_flare);
    bowl(f, zb, 0.17, rim, r, ratio, 0.27);
    let back = zb - r * ratio; // the seat's back edge, where the lid hinges
    column(f, "deck", 0.16, rim, WALL, back + 0.04, 0.26);
    seat(f, zb, rim, r - 0.002, ratio, seat_material, 0.024);
    hinges(f, back + 0.012, rim, M.chrome, 0.09);
    cistern(f, cw, ch, rim, cz, M.cistern);
}

pub fn toilet() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (r, ratio, rim) = (0.18, 1.2, 0.405); // the seat's top at 0.43 m (SEATS)
    let zb = -0.186 + r * ratio; // the lid hinges 9.4 cm in front of the cistern
    let still = new_parts(|| {
        close_coupled(&f, zb, rim, r, ratio, -0.28, 0.42, 0.38, M.seat, 1.0);
        cistern_lid(&f, 0.44, rim + 0.38, 0.155, M.cistern, 0.03, false);
        lever(&f, -0.15, 0.72, -0.28, M.chrome, false);
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let lid = lid(&f, zb, rim + 0.024, r, ratio, M.seat, 0.02, false);
    settle!(centre = false);
    swing!(lid, "lid_up", -100, seconds = 0.6, axis = Axis::X);
    meshes()
}

pub fn toilet_compact() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (r, ratio, rim) = (0.17, 1.22, 0.405);
    let zb = -0.214 + r * ratio; // a slim cistern, the lid hinged 8.6 cm in front of it
    let still = new_parts(|| {
        close_coupled(&f, zb, rim, r, ratio, -0.30, 0.4, 0.37, M.seat, 1.0);
        cistern_lid(&f, 0.41, rim + 0.37, 0.13, M.cistern, 0.025, false);
        // a dual flush button on the lid
        rose(
            &f,
            "button_ring",
            0,
            rim + 0.395,
            -0.365,
            0.034,
            M.chrome,
            0.006,
            20,
        );
        for s in [-1, 1] {
            fbox!(
                f,
                format!("button{s}"),
                0.024,
                0.006,
                0.04,
                (s as f64 * 0.0135, rim + 0.401, -0.365),
                M.chrome,
                bevel = 0.003,
                bevel_segments = 1
            );
        }
        roll(&f, 0.33, 0.62, WALL, 1);
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let lid = lid(&f, zb, rim + 0.024, r, ratio, M.seat, 0.02, false);
    settle!(centre = false);
    swing!(lid, "lid_up", -100, seconds = 0.6, axis = Axis::X);
    meshes()
}

pub fn toilet_classic() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (r, ratio, rim) = (0.18, 1.2, 0.425); // the wooden seat's top at 0.45 m (content `seat`)
    let zb = -0.15 + r * ratio;
    let still = new_parts(|| {
        close_coupled(&f, zb, rim, r, ratio, -0.25, 0.44, 0.4, M.seat_wood, 1.25);
        // a moulded two-step lid, a band round the cistern and a lever with a ball end
        cistern_lid(&f, 0.47, rim + 0.4, 0.185, M.cistern, 0.03, true);
        tube(
            &f,
            "cistern_band",
            &[(-0.195, rim + 0.06, -0.253), (0.195, rim + 0.06, -0.253)],
            0.007,
            M.cistern,
            8,
            1,
            1.0,
        );
        lever(&f, -0.15, 0.76, -0.25, M.chrome, true);
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let lid = lid(&f, zb, rim + 0.024, r, ratio, M.seat_wood, 0.02, false);
    settle!(centre = false);
    swing!(lid, "lid_up", -100, seconds = 0.6, axis = Axis::X);
    meshes()
}

/// A bowl off the floor, fixed to the boxed-in cistern at `panel_z` by a block behind it.
fn wall_hung(f: &GameFrame, zb: f64, rim: f64, r: f64, ratio: f64, panel_z: f64) {
    bowl(f, zb, 0.21, rim, r, ratio, 0.3);
    column(
        f,
        "neck",
        0.25,
        rim - 0.004,
        panel_z - 0.02,
        zb - r * ratio * 0.45,
        0.26,
    );
}

pub fn toilet_smart() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (r, ratio, rim) = (0.19, 1.22, 0.41); // the seat's top at 0.44 m (SEATS)
    let panel_z = -0.28;
    let housing = -0.18; // the seat's housing (the bidet) from the panel to here
    let zb = housing + r * ratio;
    let still = new_parts(|| {
        fbox!(
            f,
            "panel",
            0.9,
            1.2,
            panel_z - WALL,
            (0, 0.6, (WALL + panel_z) / 2.0),
            M.panel,
            bevel = 0.01
        );
        // a shelf along the panel's top, a display with a ready light, a remote in its holder, a roll
        soft!(
            f,
            "shelf",
            0.9,
            0.025,
            panel_z - WALL + 0.02,
            (0, 1.1875, (WALL + panel_z) / 2.0 + 0.01),
            M.panel,
            round = 0.008,
            round_segments = 2
        );
        soft!(
            f,
            "display",
            0.2,
            0.12,
            0.012,
            (0, 1.0, panel_z + 0.006),
            M.screen,
            round = 0.004,
            round_segments = 1
        );
        for k in 0..3 {
            fbox!(
                f,
                format!("key{k}"),
                0.03,
                0.012,
                0.004,
                (-0.05 + 0.05 * k as f64, 0.96, panel_z + 0.013),
                M.panel,
                bevel = 0.002,
                bevel_segments = 1
            );
        }
        fsphere!(
            f,
            "ready",
            0.008,
            (0.06, 1.035, panel_z + 0.013),
            M.led,
            segments = 8,
            rings = 5
        );
        soft!(
            f,
            "remote_holder",
            0.075,
            0.07,
            0.03,
            (0.34, 0.68, panel_z + 0.015),
            M.panel,
            round = 0.008,
            round_segments = 1
        );
        soft!(
            f,
            "remote",
            0.055,
            0.15,
            0.022,
            (0.34, 0.73, panel_z + 0.02),
            M.screen,
            round = 0.009,
            round_segments = 2
        );
        roll(&f, -0.34, 0.8, panel_z, -1);
        wall_hung(&f, zb, rim, r, ratio, panel_z);
        seat(&f, zb, rim, r - 0.002, ratio, M.seat, 0.03);
        soft!(
            f,
            "housing",
            0.4,
            0.055,
            housing - panel_z + 0.01,
            (0, rim + 0.0275, (panel_z + housing) / 2.0 - 0.005),
            M.seat,
            round = 0.02,
            round_segments = 2
        );
        fbox!(
            f,
            "housing_seam",
            0.36,
            0.004,
            0.004,
            (0, rim + 0.04, housing + 0.004),
            M.trim
        );
    });
    join(&still, "body", Some((0, 0, 0).into()));
    // the lid is level with the housing's top and hinged on its front edge, so it lifts over it
    let lid = lid(&f, zb, rim + 0.03, r, ratio, M.seat, 0.025, true);
    settle!(centre = false);
    swing!(lid, "lid_up", -100, seconds = 0.8, axis = Axis::X);
    meshes()
}

pub fn toilet_wall_hung() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let (r, ratio, rim) = (0.18, 1.25, 0.42); // the seat's top at 0.445 m (content `seat` 0.45)
    let panel_z = -0.31;
    let zb = panel_z + 0.095 + r * ratio;
    let still = new_parts(|| {
        fbox!(
            f,
            "panel",
            0.6,
            1.1,
            panel_z - WALL,
            (0, 0.55, (WALL + panel_z) / 2.0),
            M.boxing,
            bevel = 0.01
        );
        soft!(
            f,
            "panel_top",
            0.6,
            0.02,
            panel_z - WALL + 0.015,
            (0, 1.09, (WALL + panel_z) / 2.0 + 0.0075),
            M.stone,
            round = 0.006,
            round_segments = 1
        );
        soft!(
            f,
            "flush_plate",
            0.22,
            0.15,
            0.012,
            (0, 1.0, panel_z + 0.006),
            M.screen,
            round = 0.005,
            round_segments = 1
        );
        for s in [-1, 1] {
            soft!(
                f,
                format!("flush{s}"),
                0.09,
                0.12,
                0.008,
                (s as f64 * 0.05, 1.0, panel_z + 0.014),
                M.chrome,
                round = 0.004,
                round_segments = 1
            );
        }
        wall_hung(&f, zb, rim, r, ratio, panel_z);
        let back = zb - r * ratio;
        soft!(
            f,
            "deck",
            0.24,
            0.03,
            back - panel_z + 0.02,
            (0, rim - 0.015, (panel_z + back) / 2.0),
            M.porcelain,
            round = 0.012,
            round_segments = 2
        );
        seat(&f, zb, rim, r - 0.002, ratio, M.seat, 0.024);
        hinges(&f, back + 0.012, rim, M.chrome, 0.09);
    });
    join(&still, "body", Some((0, 0, 0).into()));
    let lid = lid(&f, zb, rim + 0.024, r, ratio, M.seat, 0.02, false);
    settle!(centre = false);
    swing!(lid, "lid_up", -100, seconds = 0.6, axis = Axis::X);
    meshes()
}

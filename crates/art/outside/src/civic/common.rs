//! What the named places' fittings are built from: the palette, the frame, the wall's face, and the small things
//! the desks, counters and tables carry (monitors, keyboards, phones, books, bells, plants).

use artkit::materials::Mat;
use artkit::math::radians;
use artkit::parts::{join, new_parts, set_origin};
use artkit::place::{Coord, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fsphere, fsweep, jitter, kit_fillet, kit_tube, loft, palette, revolve};

palette! { M:
    // the reception desk
    desk = ("#ECEAE5", satin),        // its white carcass and panels; the main part the player colours
    oak = ("#C9A77C", wood),          // its oak ledge and band; the teacher's desk; the wall bars
    plinth = ("#3A3D41", satin),      // dark kick plinths and recesses
    monitor = ("#2B2C2E", satin),     // monitors, keyboards' cases, phones
    screen = ("#1E2A36", gloss),
    keys = ("#D9DADB", satin),        // keyboards and mice
    acrylic = ("#DCE6EA", gloss),     // the leaflet stand (opaque gloss: glass would cost a draw call)
    leaflet1 = ("#3E8EDE", gloss), leaflet2 = ("#F2C14E", gloss), leaflet3 = ("#5EC28B", gloss),
    leaflet4 = ("#D9483B", gloss),
    bell = ("#C8CCD0", chrome),       // service bells, the clothes rail, the mannequin's stand
    pot = ("#F5F3EE", gloss),         // plant pots
    soil = ("#4A3A2C", matte),
    leaf = ("#5E8F4E", satin),        // a desk plant's leaves
    // the hotel's desk
    walnut = ("#5E3B26", wood),       // its walnut carcass and panels; the main part the player colours
    marble = ("#EDEAE4", gloss, roughness(0.28)),   // its ledge
    brass = ("#C9A54A", metal),       // its kick strip, lamp and hooks; the bar's rail; the trolley; the altar's
    book = ("#7A1F24", satin),        // the guest book's and the register's covers, hymn books
    page = ("#F2EEE3", matte),        // pages
    shade = ("#2F6B4A", gloss, glow("#FFD9A0")),    // the banker's lamp's green shade
    fob = ("#B8322B", satin),         // the room keys' fobs
    // the duty desk
    counter = ("#53657A", satin),     // its blue-grey carcass; the main part the player colours
    laminate = ("#D9D6CE", satin),    // its ledge and worktop
    glass = ("#DDEBF0", glass),       // the security screens
    frame = ("#B9BDC1", metal),       // aluminium: screen frames, the teller's grille, the locker frames
    police = ("#24407A", gloss),      // the duty desk's chequered band
    radio = ("#1E1F22", satin),
    // the teller counter
    teller = ("#B88A5A", wood),       // its oak front and screen rails; the main part the player colours
    note = ("#8FB98A", matte),        // banknotes
    coin = ("#C9A54A", metal),
    // the bar
    mahogany = ("#5A3220", wood),     // its carcass, panels and top; the main part the player colours
    handle = ("#F2EEE3", gloss),      // the pumps' ceramic handles
    clip1 = ("#2E5470", gloss), clip2 = ("#B8322B", gloss), clip3 = ("#D6A64A", gloss),   // pump clips
    beer = ("#B8741E", gloss),        // a pint (opaque: glass would cost a draw call)
    froth = ("#F4EBD6", matte),
    towel = ("#3E5470", matte),       // the bar runner, beer mats
    // the pupil's desk
    pine = ("#C9A26B", wood),         // its desk and seat; the main part the player colours
    iron = ("#2B302E", metal),        // its cast-iron frame; the vault door's wheel
    ink = ("#1E2A4A", gloss),
    // the pew and the altar
    pew_oak = ("#7A5234", wood),      // the pew; the lectern
    stone = ("#D8D2C4", satin),       // the altar's stone
    linen = ("#F7F3EA", fabric),      // its fair linen
    frontal = ("#5B2A6E", fabric),    // its frontal
    candle = ("#F7F3EA", satin),
    flame = ("#FFC85A", gloss, glow("#FFC070")),
    // the waiting bench
    shell = ("#3F8C8C", satin),       // its moulded seats; the main part the player colours
    steel = ("#9EA3A8", metal),       // its beam and legs
    mag1 = ("#3E5470", gloss), mag2 = ("#D9483B", gloss), mag3 = ("#D6A64A", gloss), mag4 = ("#9DAF8C", gloss),
    // the couch
    vinyl = ("#7FA7B8", satin),       // its wipe-clean pad; the main part the player colours
    white = ("#EEF0F0", satin),       // its frame
    paper = ("#FFFFFF", matte),       // its paper sheet
    // the clothes rail and the mannequin
    cloth1 = ("#C0392B", fabric), cloth2 = ("#2B3A55", fabric), cloth3 = ("#E9C46A", fabric),
    cloth4 = ("#F2EEE3", fabric), cloth5 = ("#5E8F6E", fabric), cloth6 = ("#C98F6B", fabric),
    dummy = ("#F2F0EA", gloss),       // the mannequin's body
    // the plant table
    slat = ("#B08850", wood),         // its slats; the main part the player colours
    galv = ("#A9AEB1", metal),        // its galvanised frame
    terracotta = ("#C0663E", satin),
    foliage = ("#4F7F3F", foliage),
    bloom1 = ("#E9A6B4", gloss), bloom2 = ("#F2C14E", gloss), bloom3 = ("#8E6CC9", gloss),
    label = ("#F7F5EE", matte),       // price tags, name cards
    tray = ("#2B2C2E", satin),        // seed trays
    // the cash machine
    atm = ("#4A4F55", satin),         // its cabinet; the main part the player colours
    fascia = ("#C9CCCF", satin),
    atm_screen = ("#2A4A6A", gloss, glow("#9FD3FF")),
    sign = ("#2E7D4F", gloss, glow("#BFF5D0")),     // its lit band
    slot = ("#1E1F22", matte),        // its slots, the lockers' vents
    // the lockers
    locker = ("#5C7A99", satin),      // their doors; the main part the player colours
    // the luggage trolley
    carpet = ("#8C1F28", fabric),     // its deck; with the brass, its colours
    case1 = ("#3E5470", satin), case2 = ("#C98F6B", satin),
    rubber = ("#1E1F22", satin),
    // the vault door
    vault = ("#8E959B", metal),       // its door and frame; the main part the player colours
    vault_dark = ("#5C6168", metal),
}

pub use crate::outdoor::common::rod;

pub const F: GameFrame = GameFrame::new(0.0);
pub use artkit::kit::WALL;

// Heights the content names (civic.json): kept here and checked against it.
pub const SEAT: f64 = 0.45; // pew and waitingBench seat.height (forward 0)

/// A bent tube through game points, its corners rounded without overshooting.
pub fn tube(name: &str, pts: &[(f64, f64, f64)], r: f64, m: Mat, corner: f64, verts: usize) -> Obj {
    kit_tube!(
        F,
        name,
        &kit_fillet!(pts, corner),
        r,
        m,
        verts = verts,
        round = 1
    )
}

/// A chrome service bell standing at game (x, y, z).
pub fn bell(name: &str, x: f64, y: f64, z: f64) -> Obj {
    revolve!(
        name,
        &[
            (0.0, 0.0),
            (0.045, 0.0),
            (0.045, 0.008),
            (0.04, 0.012),
            (0.035, 0.03),
            (0.022, 0.045),
            (0.008, 0.05),
            (0.006, 0.062),
            (0.01, 0.066),
            (0.0, 0.068)
        ],
        at = F.at(x, y, z),
        material = M.bell,
        segments = 14
    )
}

/// A flat monitor standing on a top at height `y`, its screen to the back (face -1, the staff's side).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn monitor(name: &str, x: f64, y: f64, z: f64, w: f64, h: f64, face: i32) -> Obj {
    let face = face as f64;
    fbox!(
        F,
        format!("{name}_foot"),
        0.2,
        0.012,
        0.15,
        (x, y + 0.006, z),
        M.monitor,
        bevel = 0.004
    );
    fbox!(
        F,
        format!("{name}_neck"),
        0.04,
        0.17,
        0.025,
        (x, y + 0.09, z - face * 0.035),
        M.monitor,
        bevel = 0.006
    );
    fbox!(
        F,
        name,
        w,
        h,
        0.025,
        (x, y + 0.13 + h / 2.0, z - face * 0.01),
        M.monitor,
        bevel = 0.006
    );
    fbox!(
        F,
        format!("{name}_screen"),
        w - 0.025,
        h - 0.025,
        0.004,
        (x, y + 0.13 + h / 2.0, z - face * 0.01 + face * 0.0125),
        M.screen
    )
}

pub fn keyboard(name: &str, x: f64, y: f64, z: f64, w: f64) -> Obj {
    fbox!(
        F,
        name,
        w,
        0.016,
        0.14,
        (x, y + 0.008, z),
        M.monitor,
        bevel = 0.004
    );
    fbox!(
        F,
        format!("{name}_keys"),
        w - 0.03,
        0.006,
        0.11,
        (x, y + 0.018, z),
        M.keys
    );
    fbox!(
        F,
        format!("{name}_mouse"),
        0.06,
        0.022,
        0.1,
        (x + w / 2.0 + 0.1, y + 0.011, z),
        M.keys,
        bevel = 0.01
    )
}

/// A desk phone, its keys to the one at it (turn 180: the back).
pub fn phone(name: &str, x: f64, y: f64, z: f64, turn: f64) -> Obj {
    let parts = new_parts(|| {
        fbox!(
            F,
            format!("{name}_base"),
            0.2,
            0.05,
            0.18,
            (0, 0.025, 0),
            M.monitor,
            bevel = 0.012
        );
        fbox!(
            F,
            format!("{name}_handset"),
            0.06,
            0.035,
            0.2,
            (-0.065, 0.065, 0),
            M.monitor,
            bevel = 0.014
        );
        for r in 0..4 {
            for c in 0..3 {
                fbox!(
                    F,
                    format!("{name}_key{r}{c}"),
                    0.022,
                    0.006,
                    0.018,
                    (0.01 + c as f64 * 0.03, 0.052, 0.05 - r as f64 * 0.028),
                    M.keys
                );
            }
        }
    });
    let ph = join(&parts, name, None);
    ph.set_rotation_euler_axis(2, radians(turn));
    ph.set_location(F.at(x, y, z))
}

/// A closed hardback `w` x `t` x `d` lying at game `at` (its middle): its boards and the pages between them.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn book(
    name: &str,
    w: f64,
    t: f64,
    d: f64,
    at: (f64, f64, f64),
    cover: Mat,
    rot: (f64, f64, f64),
) -> Obj {
    let (x, y, z) = at;
    let parts = new_parts(|| {
        fbox!(
            F,
            format!("{name}_boards"),
            w,
            t,
            d,
            (0, 0, 0),
            cover,
            // (2 mm: 3 met in the middle of a 6 mm board's edge, a ridge)
            bevel = 0.002,
            bevel_segments = 1
        );
        fbox!(
            F,
            format!("{name}_pages"),
            w - 0.012,
            t - 0.006,
            d - 0.006,
            (0.007, 0, 0),
            M.page
        );
    });
    let b = join(&parts, name, None);
    b.set_rotation_euler((radians(rot.0), radians(rot.1), radians(rot.2)));
    b.set_location(F.at(x, y, z))
}

/// An open book lying at game `at` (the middle of its spine, on what it lies on), `w` across both pages, its
/// page blocks rounding up from the spine; `tilt` degrees, its far (+z) edge up.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn open_book<X: Coord, Y: Coord, Z: Coord>(
    name: &str,
    w: f64,
    d: f64,
    at: (X, Y, Z),
    cover: Mat,
    tilt: f64,
    t: f64,
) -> Obj {
    let (x, y, z) = at;
    let parts = new_parts(|| {
        fbox!(
            F,
            format!("{name}_cover"),
            w + 0.02,
            0.004,
            d + 0.02,
            (0, 0.002, 0),
            cover
        );
        for s in [-1, 1] {
            let sf = s as f64;
            let sections: Vec<Vec<_>> = [0.004, 0.004 + t]
                .iter()
                .map(|&y0| {
                    [
                        (0.004, -d / 2.0),
                        (w / 2.0, -d / 2.0),
                        (w / 2.0, d / 2.0),
                        (0.004, d / 2.0),
                    ]
                    .iter()
                    .map(|&(u, v)| F.at(sf * u, y0, v))
                    .collect()
                })
                .collect();
            crate::eased(loft!(
                format!("{name}_pages{s}"),
                &sections,
                material = M.page
            ));
            // the top page curving up from the spine
            fsweep!(
                F,
                format!("{name}_curl{s}"),
                &[
                    (sf * 0.004, 0.004 + t * 0.4, 0),
                    (sf * w * 0.12, 0.004 + t * 1.25, 0),
                    (sf * w * 0.3, 0.004 + t * 1.05, 0),
                    (sf * w * 0.49, 0.004 + t, 0)
                ],
                0,
                M.page,
                profile = [
                    (0.0012, -d / 2.0),
                    (0.0012, d / 2.0),
                    (-0.0012, d / 2.0),
                    (-0.0012, -d / 2.0)
                ],
                up = (0, 0, 1),
                resolution = 2
            );
        }
    });
    let b = join(&parts, name, None);
    set_origin(b, F.at(0, 0, 0));
    b.set_rotation_euler_axis(0, radians(-tilt));
    b.set_location(F.at(x, y, z))
}

/// A small potted plant standing at game (x, y, z).
pub fn plant(name: &str, x: f64, y: f64, z: f64, r: f64) {
    crate::eased(revolve!(
        format!("{name}_pot"),
        &[
            (0.0, 0.0),
            (r * 0.7, 0.0),
            (r * 0.8, r * 0.1),
            (r, r * 1.3),
            (r * 0.9, r * 1.3),
            (0.0, r * 1.2)
        ],
        at = F.at(x, y, z),
        material = M.pot,
        segments = 16
    ));
    for (k, (dx, dy, dz, s)) in [
        (0.0, 1.9, 0.0, 0.9),
        (-0.45, 1.6, 0.2, 0.7),
        (0.45, 1.65, -0.2, 0.75),
    ]
    .into_iter()
    .enumerate()
    {
        jitter!(
            fsphere!(
                F,
                format!("{name}_leaf{k}"),
                r * s,
                (x + dx * r, y + dy * r, z + dz * r),
                M.leaf,
                scale = (1, 1, 0.8),
                segments = 10,
                rings = 6
            ),
            r * 0.15,
            size = r * 0.6,
            seed = k as f64 * 1.3 + x
        );
    }
}

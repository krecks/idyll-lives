//! What the shop fittings and the café furniture are built from: the palette, and the few shapes the family makes
//! itself.

use artkit::materials::Mat;
use artkit::math::{P3, PI, cos, radians, sin, v3};
use artkit::place::GameFrame;
use artkit::scene::Obj;
use artkit::{fcyl, loft, palette, revolve, ring};

palette! { M:
    // the shop counter
    counter = ("#CDB08A", wood),     // the oak carcass and panels; the main part the player colours
    walnut = ("#7A5234", wood),      // its walnut worktop, rails, stiles and plinth
    till = ("#3A3D41", satin),       // the till's case
    keys = ("#C9CCCF", satin),       // its keys and the drawer's tray
    key2 = ("#D9483B", gloss),       // its total key
    screen = ("#7FD1A8", gloss),     // its display
    note = ("#8FB98A", matte),       // banknotes in the drawer
    coin = ("#C9A54A", metal),       // coins in the drawer
    jar = ("#DDEBF0", gloss),        // the sweet jar (opaque gloss: glass would cost a draw call)
    jarLid = ("#B8322B", gloss),     // its lid
    sweet1 = ("#D9483B", gloss), sweet2 = ("#F2C14E", gloss), sweet3 = ("#5EC28B", gloss),
    sweet4 = ("#E9A6B4", gloss), sweet5 = ("#4E7A7F", gloss),
    mag1 = ("#3E5470", gloss), mag2 = ("#D9483B", gloss), mag3 = ("#D6A64A", gloss), mag4 = ("#9DAF8C", gloss),
    page = ("#F2F0EA", matte),       // a magazine's title strip
    // the bakery case
    case = ("#F1E6D0", satin),       // the cream cabinet; the main part the player colours
    rim = ("#7A5234", satin),        // its walnut deck, ends, rails and tracks (plain, with the body)
    stripe = ("#C98F6B", satin),     // the band along its front
    glass = ("#DDEBF0", glass),      // the display's glass and its back doors
    tray = ("#ECE8DF", gloss),       // paper doilies and trays
    bun = ("#C98F4E", satin), crust = ("#B97A40", satin), pastry = ("#D9A35F", satin),
    jam = ("#C0392B", gloss), lemon = ("#F2C14E", gloss, roughness(0.4)), sponge = ("#F2E1C1", satin),
    icing = ("#E9A6B4", gloss), choc = ("#5A3B26", gloss), cream = ("#F7F3EA", gloss),
    // the grocery shelves
    shelf = ("#D3D4D2", metal),      // the frame and shelves; the main part the player colours
    trim = ("#5C6168", metal),       // the kick plate, top cap and shelf lips
    label = ("#F7F5EE", matte),      // price tags
    bread = ("#E8D2A6", matte),      // sandwich packs
    filling = ("#7FA35A", matte),    // the lettuce in them
    good1 = ("#D9483B", gloss), good2 = ("#F2C14E", gloss), good3 = ("#4E7A7F", gloss),
    good4 = ("#9DAF8C", gloss), good5 = ("#E8902E", gloss), good6 = ("#3E5470", gloss),
    good7 = ("#F2F0EA", gloss), good8 = ("#8C5A3C", gloss), good9 = ("#E9A6B4", gloss),
    tinLid = ("#B9BDC1", metal),     // tins' and jars' lids
    bell = ("#C8CCD0", chrome),      // the counter's service bell
    // the café table and chair
    marble = ("#EDEAE4", gloss, roughness(0.28)),     // the table's top; its main part
    vein = ("#BDB8AF", gloss),       // the grey veins in its marble
    iron = ("#2B302E", metal),       // its cast-iron foot
    bentwood = ("#5A3B26", wood),    // the chair's frame; its main part
    cane = ("#D9C7A7", fabric),      // its woven seat
}

/// A slab from game depth a0 to a1, its face the convex polygon `profile` [(x, height)], in order (a sandwich
/// pack).
pub fn prism(name: &str, profile: &[(f64, f64)], a0: f64, a1: f64, material: Mat) -> Obj {
    let at = |a: f64| profile.iter().map(|&(x, y)| v3(-x, -a, y)).collect();
    artkit::bevel::eased(artkit::kit::prism(name, at(a0), at(a1), material))
}

/// A point on a plane through (cx, cy, cz) tilted `deg` back (its back edge higher): `dz` towards the front along
/// it, `up` off it.
pub fn tilted(cx: f64, cy: f64, cz: f64, deg: f64, dz: f64, up: f64) -> (f64, f64, f64) {
    let t = radians(deg);
    (
        cx,
        cy - dz * sin(t) + up * cos(t),
        cz + dz * cos(t) + up * sin(t),
    )
}

/// A sweet jar standing at game (x, y, z): a round body, a rounded shoulder, a neck and a domed lid with a knob.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn jar(
    frame: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    body: Mat,
    lid: Mat,
) -> Obj {
    let P3 {
        x: bx,
        y: by,
        z: bz,
    } = frame.at(x, y, z);
    let rows = [
        (0.0, 0.92),
        (0.01, 1.0),
        (0.78, 1.0),
        (0.88, 0.92),
        (0.93, 0.8),
        (0.95, 0.78),
    ];
    let secs: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(t, k)| ring!(r * k, r * k, z = bz + h * t, verts = 20, at = (bx, by)))
        .collect();
    loft!(name, &secs, material = body);
    revolve!(
        format!("{name}_lid"),
        &[
            (0.0, h * 0.94),
            (r * 0.82, h * 0.94),
            (r * 0.84, h * 0.97),
            (r * 0.8, h * 1.0),
            (r * 0.5, h * 1.06),
            (r * 0.18, h * 1.08),
            (r * 0.2, h * 1.12),
            (0.0, h * 1.14)
        ],
        at = (bx, by, bz),
        material = lid,
        segments = 16
    )
}

/// A quarter ellipse from (z0, y0) at the bottom front, curving up and back to (z1, y1): the display's curved
/// front (game depth, height), `inset` in from it.
pub fn arc(z0: f64, y0: f64, z1: f64, y1: f64, steps: usize, inset: f64) -> Vec<(f64, f64)> {
    (0..=steps)
        .map(|i| {
            let (i, steps) = (i as f64, steps as f64);
            (
                z1 + (z0 - z1 - inset) * cos(PI / 2.0 * i / steps),
                y0 + (y1 - y0 - inset) * sin(PI / 2.0 * i / steps),
            )
        })
        .collect()
}

/// A ring of `flutes` flutes round radius `r` at Blender height `z`, about the origin (a tart's fluted case, a
/// cupcake's paper cup, a gondola's post).
pub fn fluted(r: f64, z: f64, flutes: usize, depth: f64, per: usize) -> Vec<P3> {
    artkit::kit::fluted(r, z, flutes, depth, per, 0.0)
}

/// A bottle standing at game point `foot`: a round body, a shoulder, a neck and its cap.
pub fn bottle(
    name: &str,
    foot: (f64, f64, f64),
    r: f64,
    h: f64,
    material: Mat,
    cap: Mat,
    frame: &GameFrame,
) -> Obj {
    let P3 {
        x: bx,
        y: by,
        z: bz,
    } = frame.at(foot.0, foot.1, foot.2);
    let rings = [
        (0.0, 0.92),
        (0.03, 1.0),
        (0.6, 1.0),
        (0.76, 0.55),
        (0.86, 0.38),
        (0.94, 0.38),
    ];
    let secs: Vec<Vec<P3>> = rings
        .iter()
        .map(|&(t, k)| ring!(r * k, r * k, z = bz + h * t, verts = 8, at = (bx, by)))
        .collect();
    loft!(name, &secs, material = material);
    fcyl!(
        frame,
        format!("{name}_cap"),
        r * 0.42,
        h * 0.07,
        (foot.0, foot.1 + h * 0.965, foot.2),
        cap,
        verts = 8
    )
}

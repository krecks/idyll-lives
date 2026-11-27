//! What the street furniture is built from: the colours every model may use and each model's own `body`, and the
//! few shapes the family makes itself.

use artkit::far::decimated;
use artkit::forms::Scale;
pub use artkit::kit::fluted;
use artkit::materials::Mat;
use artkit::math::{P3, PI};
use artkit::parts::join;
use artkit::place::{GameFrame, Spot};
use artkit::scene::{Obj, meshes};
use artkit::{fsweep, palette};

// Colours every model may use. Each model adds its own `body` (the part the player colours), whose colour and finish
// differ from model to model (a red gloss post box, a cast-iron lamp, a stone fountain), so each model has a palette
// of its own: `own`.
palette! { SF:
    iron = ("#2B302E", metal),        // cast iron: posts, frames, the lamp's lantern frame
    stone = ("#C9C2B4", matte),       // pale stone
    granite = ("#8E8C88", matte),     // plinths and slabs
    water = ("#6FA8C0", gloss),       // standing water: opaque, as the old parts'
    spray = ("#BFDDE8", gloss),       // running water: jets, streams and arcs (gloss, never emissive)
    chrome = ("#C8CCD0", chrome),     // taps, handles, the pedal
    dark = ("#1E2120", matte),        // openings, slots, the phone
    paper = ("#F2F0EA", matte),       // labels, timetables, plates
    brass = ("#B08D4A", metal),       // crests and plaques
    wood = ("#9A6B45", wood),         // slats
    glass = ("#C9DDE3", glass),       // the shelter's and the phone box's glazing (see-through)
    pane = ("#6F8791", gloss),        // the phone box door's panes: opaque, so the moving door is one draw
    blue = ("#2E6FA8", gloss),        // the town map's routes
    // the taxi rank (the shelter): the taxi's yellow, its chequers and lettering, and the sign's light box, lit at
    // night (E8) as the taxi's roof sign looks lit
    taxi = ("#F2B81C", gloss),
    ink = ("#16181A", matte),
    sign_light = ("#FFF4C8", satin, glow("#FFE7A0")),
    map = ("#DCE6D2", matte),         // the shelter's map
    // notices pinned on the board: plain, so they cost no draw call
    note_white = ("#F4F1E6", satin),
    note_yellow = ("#F2D16B", satin),
    note_pink = ("#E9A6A6", satin),
    note_blue = ("#9FC5E8", satin),
    note_green = ("#B6D7A8", satin),
    pin = ("#B8322B", satin),
    cork = ("#B88A5A", matte),
    // at night: the lamp's lantern and the phone box's sign glow (E8), nothing by day
    lantern = ("#F6EBCF", satin, glow("#FFD596")),
    sign = ("#F2F0EA", matte, glow("#FFF1D2")),
    // the traffic light's backboard and hoods, and its three lenses: dark glass, each glowing its colour while it's
    // the one the light shows (by day too: render/pc/signals.ts), plain, so the head stays one draw
    board = ("#171A19", matte),
    lens_red = ("#4A1714", gloss, glow("#FF4A2E")),
    lens_amber = ("#4A3812", gloss, glow("#FFB21E")),
    lens_green = ("#103A2C", gloss, glow("#1FE58A")),
    // the motorway sign's face: its blue and its white (satin: the sheen of a reflective sheet, not lit)
    sign_blue = ("#1D4F9C", satin),
    sign_white = ("#F1F1EC", satin),
    // each model's own, last (`own` sets it)
    body = ("#2B302E", metal),
}

/// Each model's `body`: (id, colour, finish).
pub const BODIES: [(&str, &str, &str); 12] = [
    ("streetLamp", "#2B302E", "metal"),
    ("trafficLight", "#2B302E", "metal"),
    ("parkBench", "#2F3A33", "metal"),
    ("drinkingFountain", "#C9C2B4", "matte"),
    ("litterBin", "#3C6B3A", "metal"),
    ("busStop", "#5C6168", "metal"),
    ("postBox", "#B8322B", "gloss"),
    ("noticeBoard", "#5A3B26", "wood"),
    ("phoneBooth", "#B8322B", "gloss"),
    ("fountainTown", "#C9C2B4", "matte"),
    ("townStatue", "#5E7F6E", "metal"),
    ("motorwaySign", "#9CA2A6", "metal"),
];

/// The palette of model `id`: the shared colours and its own `body`.
pub fn own(id: &str) -> SF {
    let &(_, hex, finish) = BODIES
        .iter()
        .find(|b| b.0 == id)
        .unwrap_or_else(|| panic!("no body for {id:?}"));
    let mut m = SF;
    m.body = Mat::new("body", hex, finish);
    m
}

/// A bent tube in one piece through game points `pts`, curved through them (a stream of water, the bubbler's arc):
/// replaces the rods joined by balls.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn sweep_tube<P: Spot>(
    f: &GameFrame,
    name: &str,
    pts: &[P],
    r: f64,
    material: Mat,
    verts: usize,
    scale: Scale,
) -> Obj {
    fsweep!(
        f,
        name,
        pts,
        r,
        material,
        verts = verts,
        resolution = 3,
        scale = scale
    )
}

/// A square loop `h` either side of the middle at height `y`, its sides square to the axes (Blender).
pub fn square(h: f64, y: f64) -> Vec<P3> {
    vec![
        P3 { x: h, y: h, z: y },
        P3 { x: -h, y: h, z: y },
        P3 { x: -h, y: -h, z: y },
        P3 { x: h, y: -h, z: y },
    ]
}

/// A ring of `lobes` rounded lobes (gadroons) round a bowl's underside, radius `r` at height `y` (Blender).
pub fn lobed(r: f64, y: f64, lobes: usize, depth: f64, per: usize) -> Vec<P3> {
    fluted(r, y, lobes, depth, per, PI / lobes as f64)
}

/// Below this height (m) a far version keeps every vertex.
pub const FAR_FEET: f64 = 0.1;
/// E7: they stand on the town's lots, seen from afar.
pub const FAR: f64 = 0.25;

/// A far version that keeps its feet: as the family's `far_ratio` (the model at rest, still, without the water
/// hidden at rest, joined and decimated to `FAR`), but the faces up to `FAR_FEET` are left out of the decimation,
/// which otherwise lifted the drinking fountain's slab 4 cm off the ground and pushed it out 7 cm a side; the
/// vertices that sit on one another (turned parts' ends meeting on their axis, a loft's seams) are merged first, as
/// decimating them unmerged tore the fountains' pillars and copings into spikes; and the outermost faces are kept,
/// so its box stays the model's.
pub fn far_keeping_feet(build: fn() -> Vec<Obj>, id: &str) -> Vec<Obj> {
    build();
    let all = meshes();
    for &o in &all {
        o.edit(|p| p.tracks.clear());
    }
    for &o in &all {
        if o.read(|p| {
            p.scale
                .array()
                .iter()
                .map(|k| k.abs())
                .fold(f32::INFINITY, f32::min)
        }) < 0.01
        {
            o.remove();
        }
    }
    let obj = join(&meshes(), &format!("{id}_far"), Some(P3::ZERO));
    vec![decimated(obj, FAR, FAR_FEET, 0.003)]
}

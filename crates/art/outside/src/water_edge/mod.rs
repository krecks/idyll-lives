//! The water's edge: a jetty, a rowing boat, a clump of reeds, a beach hut and a bench that faces the water.
//! Notes: NOTES.md. Run: `cargo art water_edge [--only jetty] [--preview]`.

use artkit::family::{Family, Model};
use artkit::palette;

pub(crate) mod boats;
pub(crate) mod plants;
pub(crate) mod seats;
pub(crate) mod structures;

palette! { M:
    plank = ("#8A6A4A", wood),        // the jetty's decking
    post = ("#5E4632", wood),         // its posts, bearers and bollards
    hull = ("#2F6F7E", gloss),        // the rowing boat's painted hull
    inside = ("#E8E1CC", satin),      // its cream-painted interior
    seat = ("#B98A5A", wood),         // its thwarts
    oar = ("#C9A46A", wood),          // its oars
    blade = ("#5E8F4E", foliage),     // the reeds' blades
    stem = ("#6E8F4A", foliage),      // their stems
    head = ("#7A5A32", bark),         // the bulrushes' seed heads
    wall = ("#F2EAD8", satin),        // the beach hut's painted boards
    stripe = ("#3E8C8A", satin),      // its stripes
    roof = ("#8A4B3C", satin),        // its roof
    trim = ("#F7F3EA", satin),        // its window and door frames
    door = ("#2F6F6A", satin),        // its door
    glass = ("#CFE3E6", glass),       // its window
    timber = ("#7A5A3E", wood),       // its skids and floor
    brass = ("#C9A54A", metal),       // its knob and the bench's plaque
    bench_wood = ("#9A6B45", wood),   // the bench's slats
    bench_iron = ("#2B302E", metal),  // its cast-iron ends
}

const F: artkit::place::GameFrame = artkit::place::GameFrame::new(0.0);

/// The family.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "water_edge",
        M.palette(),
        vec![
            Model::new("jetty", structures::jetty, (1.92, 0.985, 3.82))
                .key("model.waterEdge.jetty")
                .paint(&[("plank", "Decking"), ("post", "Posts")]),
            Model::new("rowingBoat", boats::rowing_boat, (1.28, 0.695, 2.943))
                .key("model.waterEdge.rowingBoat")
                .paint(&[("hull", "Hull"), ("inside", "Interior"), ("seat", "Seats")]),
            Model::new("reeds", plants::reeds, (0.801, 1.539, 1.086)).key("model.waterEdge.reeds"),
            Model::new("beachHut", structures::beach_hut, (1.98, 2.185, 1.96))
                .key("model.waterEdge.beachHut")
                .paint(&[("wall", "Walls"), ("stripe", "Stripes"), ("door", "Door")]),
            Model::new("bench", seats::bench, (1.746, 0.928, 0.521))
                .key("model.waterEdge.bench")
                .paint(&[("bench_wood", "Slats"), ("bench_iron", "Frame")]),
        ],
        "The water's edge (crates/art/outside/src/water_edge, built by cargo art): a plank jetty on \
         posts, a painted rowing boat, a clump of reeds and bulrushes, a striped beach hut on skids and \
         a slatted bench that faces the water. Listed in the base manifest's packs.",
    ))
}

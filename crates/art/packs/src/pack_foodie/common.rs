//! What the foodie pack is built from: its palette, a counter for parts' names, faces painted another material,
//! lofted bodies, and the herbs and pots more than one of its models holds.

use artkit::forms::smooth_outline;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::{P3, V3, cosd, sind};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{boulder, fcyl, fextrude, fsweep, jitter, loft, palette, revolve, ring};
use std::cell::Cell;

palette! { M:
    // the herb garden
    cedar = ("#C49A6C", wood),       // the stand and the trough; its main part
    cedar_dark = ("#A07A52", wood),  // posts, rails and the trough's inside
    soil = ("#5A4030", matte),
    basil = ("#5E9E3E", satin),      // herbs are plain satin, so they cost no draw call of their own
    basil_light = ("#6FB04A", satin),
    thyme = ("#86A06E", satin),
    rosemary = ("#4E7C52", satin),
    terracotta = ("#C46A43", matte),
    chalk = ("#F2EDE2", matte),      // chalk tags, towels
    steel = ("#9EA3A8", metal),
    wicker = ("#C9A46E", satin),
    // the deli pantry
    cream = ("#EDE4D0", satin),      // the pantry's body; its main part
    cream_panel = ("#E4D9C1", satin),
    honey = ("#A8754A", satin),      // the pantry's shelves, plinth and crown (plain: the doors take the draws)
    back = ("#E2D6BC", matte),
    marble = ("#F3EFE8", gloss, roughness(0.28)),     // the pantry's ledge, the island's top
    brass = ("#B8925A", metal),
    jar_a = ("#C98A3A", gloss),
    jar_b = ("#9DB86A", gloss),
    jar_c = ("#B5653F", gloss),
    cheese = ("#D6A64A", satin),
    cheese_light = ("#E9C46A", satin),
    bread = ("#C8964F", matte),
    wine = ("#6E2B33", gloss),
    bottle = ("#3E5A3A", gloss),
    salami = ("#8E3B2E", matte),
    tomato = ("#C9483B", gloss, roughness(0.3)),
    board = ("#B08E66", satin),
    // the espresso bar
    teal = ("#2F5D62", satin),       // the cart's frame; its main part
    oak_top = ("#CDB08A", satin),    // the cart's top and shelves (plain: the stream and the wand take the draws)
    machine = ("#EFE6D4", gloss),
    copper = ("#C27A4A", metal),
    chrome = ("#D0D3D6", chrome),
    black = ("#2B2C2E", gloss),
    cup = ("#F6F3EA", gloss),
    coffee = ("#5A3420", gloss),
    beans = ("#5A3A26", satin),
    dark = ("#3A3D41", satin),
    sack = ("#C9A46E", satin),
    // the cookbook lectern
    walnut = ("#7A5234", wood),      // the lectern; its main part
    honey_wood = ("#A8754A", wood),  // its ledge and the stool
    page = ("#F6F1E4", matte),
    cover = ("#B33A32", satin),
    photo = ("#E9C46A", matte),
    book_teal = ("#4E7A7F", satin),
    book_mustard = ("#D6A64A", satin),
    book_sage = ("#9DAF8C", satin),
    // the pizza oven
    plaster = ("#E8DAC4", matte),    // the dome; its main part
    brick = ("#A4614A", matte),
    brick_light = ("#B8735A", matte),
    brick_dark = ("#6E4535", matte),
    stone = ("#C9C1B4", satin),
    worktop = ("#E9E3D8", gloss, roughness(0.3)),
    iron = ("#3E4146", metal),
    log = ("#8B6B4E", satin),        // plain, not bark: the door, the peel and the fire take the draws
    mouth = ("#2B2220", matte),
    ember = ("#F28C28", gloss),
    flame = ("#F28C28", gloss),
    flame_core = ("#FFC24A", gloss),
    dough = ("#E8C27A", matte),
    mozzarella = ("#F4EBD5", satin),
    oil = ("#7A8F3A", gloss),
    peel = ("#B08E66", satin),
    door_oak = ("#CDB08A", satin),
    // the chef's island
    cabinet = ("#5F9095", satin),    // the island's cabinets; its main part
    oak = ("#CDB08A", satin),        // the stools, plain (the hob's hot ring takes the third draw)
    oak_dark = ("#B08E66", satin),
    seat = ("#C9A46E", satin),
    hob = ("#22262B", gloss),
    ring = ("#7E848B", gloss),       // the hob's printed rings, grey (red, the cook's ring at rest looked lit)
    heat = ("#D2402A", gloss),       // the hob's ring glowing hot under the pan while it cooks
    carrot = ("#E2832E", satin),
    plate = ("#F6F3EA", gloss),
    // added with the detail pass
    crust = ("#D9A35A", matte),      // bread crusts, the pizza's rim
    log_end = ("#C4A27A", satin),    // the logs' cut ends
}

/// The wall's face in a 1-deep footprint, game depth.
pub const WALL: f64 = -0.43;

thread_local! {
    static N: Cell<u32> = const { Cell::new(0) };
}

/// `stem` numbered: each part its own name. (The script's count runs on from model to model, and the pantry's
/// doors are named by it: `_names_from` sets where a model's count starts.)
pub fn _name(stem: &str) -> String {
    N.with(|n| {
        n.set(n.get() + 1);
        format!("{stem}{}", n.get())
    })
}

/// Starts the count of `_name` after `n` names.
pub fn _names_from(n: u32) {
    N.with(|c| c.set(n));
}

/// A polygon's middle, as Blender's `MeshPolygon.center`: the mean of its corners.
fn center(obj: Obj, f: usize) -> V3 {
    obj.read(|o| {
        let c = o.mesh.corners(f);
        let mut s = V3::ZERO;
        for &v in c {
            s += o.mesh.verts[v as usize];
        }
        s / c.len() as f32
    })
}

/// Gives the faces of `obj` whose middle (its own axes, Blender) passes `test` another material: a band, a label, a
/// rind, with no more triangles.
pub fn _paint(obj: Obj, material: Mat, test: impl Fn(V3) -> bool) -> Obj {
    let count = obj.read(|o| o.mesh.faces.len());
    let hit: Vec<bool> = (0..count).map(|f| test(center(obj, f))).collect();
    obj.edit(|o| {
        let k = o.mesh.slot(material);
        for (face, &h) in o.mesh.faces.iter_mut().zip(&hit) {
            if h {
                face.mat = k;
            }
        }
    });
    obj
}

/// A body standing on `y` at (x, z), lofted through rings (height, radius) from its foot up.
#[allow(clippy::too_many_arguments)]
pub fn _lofted(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    rows: &[(f64, f64)],
    material: Mat,
    verts: usize,
) -> Obj {
    let b = f.at(x, y, z);
    let sections: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(h, r)| {
            ring!(
                r,
                r,
                z = b.z + h,
                verts = verts,
                square = 2.0,
                at = (b.x, b.y)
            )
        })
        .collect();
    loft!(name, &sections, material = material)
}

// --- herbs and pots, shared ------------------------------------------------------------------------------------
fn leaf_outline() -> Vec<(f64, f64)> {
    smooth_outline(
        &[
            (0.0, 0.0),
            (0.35, 0.25),
            (0.42, 0.6),
            (0.2, 0.92),
            (0.0, 1.0),
            (-0.2, 0.92),
            (-0.42, 0.6),
            (-0.35, 0.25),
        ],
        1,
        true,
    )
}

/// A leaf `size` long from its stalk at game point (x, y, z), pointing out at `turn` degrees round the up axis and
/// lifted `tilt` degrees from flat.
#[allow(clippy::too_many_arguments)]
pub fn _leaf(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    size: f64,
    turn: f64,
    tilt: f64,
    material: Mat,
) -> Obj {
    let pts: Vec<(f64, f64)> = leaf_outline()
        .iter()
        .map(|&(u, v)| (u * size, v * size))
        .collect();
    fextrude!(
        f,
        name,
        &pts,
        0.003,
        (x, y, z),
        material,
        plane = Facing::Top,
        rot = (-tilt, 0.0, turn)
    )
}

/// A basil plant on the soil at `y`: pairs of broad leaves in tiers up a short stem, a tuft at its top.
pub fn _basil(f: &GameFrame, x: f64, z: f64, y: f64, s: f64) {
    kit::tube!(
        f,
        _name("basil_stem"),
        &[(x, y, z), (x + 0.004 * s, y + 0.12 * s, z)],
        0.005 * s,
        M.basil,
        verts = 4,
        round = 1
    );
    for (k, (h, size, m)) in [
        (0.03, 0.07, M.basil),
        (0.065, 0.065, M.basil_light),
        (0.095, 0.055, M.basil),
        (0.12, 0.04, M.basil_light),
    ]
    .into_iter()
    .enumerate()
    {
        for j in 0..(if k < 3 { 2 } else { 3 }) {
            let a = k * 55 + j * if k < 3 { 180 } else { 120 };
            _leaf(
                f,
                &_name("basil"),
                x,
                y + h * s,
                z,
                size * s,
                a as f64,
                (30 - 8 * k as i32) as f64,
                m,
            );
        }
    }
}

/// A rosemary bush on the soil at `y`: woody stems rising and splaying, tapering to their tips.
pub fn _rosemary(f: &GameFrame, x: f64, z: f64, y: f64, s: f64) {
    for i in 0..7 {
        let a = i as f64 * 137.5;
        let (c, sn) = (cosd(a), sind(a));
        let h = (0.213 + 0.05 * (i % 3) as f64) * s;
        fsweep!(
            f,
            _name("rosemary"),
            &[
                (x + 0.02 * s * c, y, z + 0.02 * s * sn),
                (x + 0.05 * s * c, y + h * 0.5, z + 0.05 * s * sn),
                (x + 0.07 * s * c, y + h, z + 0.07 * s * sn)
            ],
            0.014 * s,
            M.rosemary,
            verts = 5,
            resolution = 2,
            scale = [1.0, 0.85, 0.35]
        );
    }
}

/// A low thyme mound on the soil at `y`: lumpy and fine-leaved.
pub fn _thyme(f: &GameFrame, x: f64, z: f64, y: f64, s: f64, seed: f64) {
    let mound = boulder!(
        _name("thyme"),
        0.07 * s,
        at = f.at(x, y - 0.01, z),
        material = M.thyme,
        scale = (1.0, 0.9, 0.7),
        seed = seed,
        noise = 0.25,
        subdivisions = 2
    );
    jitter!(mound, 0.006 * s, size = 0.02, seed = seed);
}

/// A terracotta pot standing on `y` with soil in it, its rim rolled; returns the soil's top.
pub fn _pot(f: &GameFrame, x: f64, z: f64, y: f64, r: f64, h: f64) -> f64 {
    revolve!(
        _name("pot"),
        &[
            (0.0, 0.0),
            (r * 0.72, 0.0),
            (r * 0.88, h * 0.75),
            (r, h * 0.76),
            (r * 1.02, h * 0.82),
            (r * 1.02, h),
            (r * 0.9, h),
            (r * 0.86, h * 0.9),
            (0.0, h * 0.9)
        ],
        at = f.at(x, y, z),
        material = M.terracotta,
        segments = 14
    );
    let soil = fcyl!(
        f,
        _name("pot_soil"),
        r * 0.84,
        0.008,
        (x, y + h * 0.9 + 0.003, z),
        M.soil,
        verts = 12
    );
    jitter!(soil, 0.002, size = 0.02);
    y + h * 0.9 + 0.006
}

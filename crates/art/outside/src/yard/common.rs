//! What the play and yard things are built from: the palette, the frame, and the shapes more than one of them
//! makes (ground, clumps, pots, slabs from outlines).

pub use crate::outdoor::common::{F, board, rod};
use artkit::far::low;
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::V3;
use artkit::noise::noise;
use artkit::palette;
use artkit::parts::numbered;
use artkit::place::N;
use artkit::scene::Obj;
use artkit::shapes::{CLOSED, SHARP};
use artkit::{boulder, revolve};

palette! { M:
    // the sandpit
    frame = ("#C9A27A", wood),           // its boards; the main part
    sand = ("#E9D5A8", matte),
    sand_damp = ("#D8BF8E", matte),      // the castle, moulded from damp sand
    bucket = ("#D9483B", satin),
    spade = ("#3E8EDE", satin),
    flag = ("#F2C14E", satin),
    // the bike rack
    rack = ("#5C6168", metal),           // galvanised hoops and the stand
    bike = ("#3E8EDE", gloss),           // the bike's frame
    tyre = ("#2A2C30", satin),
    rim = ("#C9CBCC", chrome),           // rims, hubs, bars, the chain ring
    saddle = ("#3A3530", satin),
    // the city bike, the kids' bike and the e-scooter (with the rack's tyre, rim and saddle): all plain, one draw
    city = ("#2F5D50", gloss),           // the city bike's frame, carrier and chain guard
    mtb = ("#D9822B", gloss),            // the mountain bike's frame
    road = ("#C8433A", gloss),           // the road bike's frame
    cargo = ("#3E6F8E", gloss),          // the cargo bike's frame and box
    ebike = ("#4C5866", gloss),          // the e-bike's frame
    battery = ("#30343A", satin),        // the e-bike's battery
    motor = ("#8E9499", metal),          // its motor, the mountain bike's fork crown and disc brakes
    basket_dark = ("#A8885C", satin),    // the basket's woven band (the basket: `basket`, the washing line's wicker)
    lamp = ("#F4F1E6", gloss),           // a lamp's glass
    reflector = ("#C8302A", gloss),      // a red back light
    kid = ("#E5484D", gloss),            // the kids' bike's frame
    kid_trim = ("#F2F0EA", gloss),       // its mudguards and chain case
    streamer_pink = ("#F28DC0", satin),
    streamer_yellow = ("#F7D154", satin),
    streamer_blue = ("#5BC0EB", satin),
    scooter = ("#3A3D42", satin),        // the e-scooter's deck, stem and bars
    scooter_metal = ("#8E9499", metal),  // its rims and the hub motor
    scooter_trim = ("#D9483B", satin),   // the latch, the deck's side strips, the grips' ends
    grip_tape = ("#232428", matte),      // the deck's grip tape, the grips
    screen = ("#1C2A33", gloss),         // the display
    // the raised bed
    bed = ("#9A7452", satin),            // its boards; the main part
    bed_post = ("#7A5A40", satin),
    soil = ("#6A4C38", matte),
    lettuce = ("#8CC05A", foliage),
    lettuce_dark = ("#6FA34A", foliage),
    leaf = ("#4F7A3E", foliage),         // tomato plants
    carrot_top = ("#5E8F4E", foliage),
    tomato = ("#D9483B", satin),
    carrot = ("#E07A2E", satin),
    // the shed
    shed = ("#8FA79A", wood),            // its weatherboards; the main part
    roof = ("#5A4A3E", matte),           // roofing felt
    trim = ("#F2F0EA", satin),           // corner boards, barge boards, the door and window frames
    shed_door = ("#6F8B7E", satin),      // the painted door
    iron = ("#2B2C2E", metal),           // hinges, latch, the log store's rack
    pane = ("#3A4A52", gloss),           // the window, dark from outside (opaque)
    inside = ("#4A4038", matte),         // the shed's inside, in shadow
    plinth = ("#6E6A64", matte),
    box_wood = ("#8E6E52", satin),       // the window box, shelves, handles
    flower_pink = ("#E07BC3", satin),
    flower_yellow = ("#F2C14E", satin),
    flower_red = ("#D9483B", satin),
    flower_lilac = ("#9B7FD1", satin),
    green = ("#5E8F4E", satin),          // plain leaves where a foliage draw would be one too many
    terra = ("#C26E4A", satin),          // flower pots
    // the greenhouse
    gh_frame = ("#F2F2F0", satin),       // its white frame; the main part
    gh_wall = ("#B5A48E", satin),        // the dwarf wall
    gravel = ("#B8A88E", matte),
    paving = ("#C9C2B4", matte),
    staging = ("#9C7A58", satin),        // the slatted benches
    mortar = ("#D8D0C2", matte),         // the dwarf wall's courses
    glass = ("#DCEBEA", glass),
    glass_bar = ("#FFFFFF", glass),      // the door's and the vent's edges: glass too, so they stay one draw each
    // the compost bin
    bin = ("#8E6E52", wood),             // its slats and lid; the main part
    compost = ("#4A3A2C", matte),
    peel = ("#E07A2E", satin),
    greens = ("#7FA35A", satin),
    shell = ("#EDE6D6", satin),          // eggshells on the heap
    // the rain barrel
    barrel = ("#3F6E4A", satin),         // the main part
    barrel_dark = ("#35603F", satin),
    brass = ("#B8925A", metal),
    can = ("#9EA7AD", metal),            // a galvanised watering can
    // the washing line: its still body all satin, one draw (the washing takes a draw a line)
    posts = ("#B9BCBE", satin),
    line = ("#F2F2F0", satin),
    basket = ("#C9A979", satin),         // wicker
    folded = ("#F2F0EA", satin),         // the washing folded in the basket
    folded_pink = ("#E4B8AE", satin),
    folded_sage = ("#9FB59A", satin),
    washing = ("#9DB7C9", fabric),       // the sheet; the washing's main colour
    towel = ("#9DC6CF", fabric),
    white = ("#F2F0EA", fabric),
    pink = ("#E4B8AE", fabric),
    denim = ("#3E5470", fabric),
    yellow = ("#F2C14E", fabric),
    red = ("#D9483B", fabric),
    sage = ("#9FB59A", fabric),
    peg = ("#C08A52", fabric),           // pegs: fabric too, so the washing stays one draw
    // the log store
    bark = ("#8E6A4A", bark),
    bark_dark = ("#7A5A3E", bark),
    bark_light = ("#9C7A58", bark),
    end_grain = ("#D9C3A0", matte),
    end_grain_dark = ("#C9AE86", matte),
    axe = ("#9A9EA2", metal),
    haft = ("#B08E66", satin),
}

/// Game points round an arc of radius `r` about game point `centre`, from `a0` to `a1` degrees, in the game's
/// x-height plane (`xy`, 0° along +x, 90° up) or its x-depth plane.
pub fn arc_points(
    centre: (f64, f64, f64),
    r: f64,
    a0: f64,
    a1: f64,
    n: usize,
    xy: bool,
) -> Vec<(f64, f64, f64)> {
    let (cx, cy, cz) = centre;
    (0..=n)
        .map(|k| {
            let a = a0 + (a1 - a0) * k as f64 / n as f64;
            let (c, s) = (artkit::math::cosd(a), artkit::math::sind(a));
            if xy {
                (cx + r * c, cy + r * s, cz)
            } else {
                (cx + r * c, cy, cz + r * s)
            }
        })
        .collect()
}

/// A slab: the polygon `pts` [(game x, height)] between game depths `z0` and `z1` (gables, a roof's ends).
pub fn prism(name: &str, pts: &[(N, f64)], z0: f64, z1: f64, material: Mat) -> Obj {
    let n = pts.len() as u32;
    let mut verts: Vec<V3> = pts.iter().map(|&(x, y)| V3::from(F.at(x, y, z1))).collect();
    verts.extend(pts.iter().map(|&(x, y)| V3::from(F.at(x, y, z0))));
    let mut faces = vec![(0..n).collect::<Vec<u32>>(), (n..2 * n).collect()];
    faces.extend((0..n).map(|i| vec![i, (i + 1) % n, n + (i + 1) % n, n + i]));
    artkit::forms::object(name, verts, &faces, Some(material), false, SHARP, false)
}

/// A lumpy clump standing on height `y` (a lettuce, a bush, a heap).
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn blob(
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    material: Mat,
    seed: f64,
    scale: (f64, f64, f64),
    noise: f64,
) -> Obj {
    boulder!(
        numbered("blob"),
        r,
        at = F.at(x, y, z),
        material = material,
        seed = seed,
        noise = noise,
        subdivisions = low(2, 1),
        scale = scale
    )
}

/// A flower pot standing on height `y`, `r` across the rim, with soil in it.
pub fn pot(x: f64, y: f64, z: f64, r: f64, h: f64, material: Option<Mat>) {
    revolve!(
        numbered("pot"),
        &[
            (r * 0.72, 0.0),
            (r, h * 0.86),
            (r * 1.06, h * 0.88),
            (r * 1.06, h),
            (r * 0.9, h),
            (r * 0.88, h * 0.86),
            (0.0, h * 0.86)
        ],
        at = F.at(x, y, z),
        material = material.unwrap_or(M.terra),
        segments = low(12, 6)
    );
}

/// One length of round bar or tube through game points, its corners bent round about `bend` (no overshoot):
/// hoops, handles, frames.
pub fn bent(
    name: &str,
    pts: &[(f64, f64, f64)],
    r: f64,
    material: Mat,
    bend: f64,
    verts: usize,
    steps: usize,
) -> Obj {
    kit::tube!(
        F,
        name,
        &kit::fillet!(pts, bend, steps),
        r,
        material,
        verts = verts,
        round = 1
    )
}

/// A ground surface `w` by `d` at height `y` (sand, soil): a grid raised in soft lumps up to `bump` (noise, the
/// same every run), `edge` metres in from its rim falling to `y`; closed underneath, 3 cm deep.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn ground(
    name: &str,
    w: f64,
    d: f64,
    y: f64,
    material: Mat,
    n: (usize, usize),
    bump: f64,
    seed: f64,
    at: (f64, f64),
    edge: f64,
) -> Obj {
    let (nx, nz) = n;
    let mut top = Vec::new();
    for i in 0..=nx {
        for j in 0..=nz {
            let (u, v) = (
                -w / 2.0 + w * i as f64 / nx as f64,
                -d / 2.0 + d * j as f64 / nz as f64,
            );
            let mut k = 1.0;
            if edge != 0.0 {
                k = 1.0_f64
                    .min((w / 2.0 - u.abs()) / edge)
                    .min((d / 2.0 - v.abs()) / edge);
            }
            let h =
                bump * k * (0.5 + 0.5 * noise(V3::from((u * 4.0 + seed, v * 4.0 - seed, seed))));
            top.push(F.at(at.0 + u, y + h, at.1 + v));
        }
    }
    let mut vs: Vec<V3> = top.iter().map(|&p| V3::from(p)).collect();
    vs.extend(top.iter().map(|p| V3::from((p.x, p.y, y - 0.03))));
    let m = (nz + 1) as u32;
    let off = top.len() as u32;
    let mut faces = Vec::new();
    for i in 0..nx as u32 {
        for j in 0..nz as u32 {
            let (a, b, c, e) = (
                i * m + j,
                (i + 1) * m + j,
                (i + 1) * m + j + 1,
                i * m + j + 1,
            );
            faces.push(vec![a, b, c, e]);
            faces.push(vec![off + e, off + c, off + b, off + a]);
        }
    }
    let (nx, nz) = (nx as u32, nz as u32);
    let mut rim: Vec<u32> = (0..=nx).map(|i| i * m).collect();
    rim.extend((1..=nz).map(|j| nx * m + j));
    rim.extend((0..nx).rev().map(|i| i * m + nz));
    rim.extend((1..nz).rev());
    for (i, &a) in rim.iter().enumerate() {
        let b = rim[(i + 1) % rim.len()];
        faces.push(vec![a, b, off + b, off + a]);
    }
    artkit::forms::object(name, vs, &faces, Some(material), true, CLOSED, false)
}

//! What the garden things are built from.

use artkit::json::{self, Json};
use artkit::materials::Mat;
use artkit::math::{V3, cosd, sind};
use artkit::mesh::Mesh;
use artkit::palette;
use artkit::parts::numbered;
use artkit::place::GameFrame;
use artkit::scene::{self, Obj};
use artkit::{fsphere, rod};
use std::cell::Cell;
use std::sync::OnceLock;

palette! { SPEC:
    // the pond
    water = ("#4F8CA6", gloss), // opaque, as the fountains' and the old part's
    water_deep = ("#3E7890", gloss),
    ripple = ("#A8D2DE", gloss),
    pebble = ("#8F887C", matte),
    pad = ("#5E8E3E", satin), // lily pads
    lily = ("#F2A6C8", satin),
    lily_white = ("#F6EEF2", satin),
    lily_heart = ("#F2C14E", satin),
    reed = ("#6E8E3A", satin), // still (satin, not foliage): it keeps the pond to three draws
    rush = ("#6A4A2E", matte), // the bulrushes' heads
    koi = ("#E8742E", satin),
    koi_white = ("#F6F2EC", satin),
    mud = ("#4A4234", matte), // the wet margin under the coping stones
    frog = ("#6E9A3A", satin),
    iris = ("#7A5CC4", satin), // the flag irises at the front
    // the planter box
    box_trim = ("#7A5A40", wood),
    soil = ("#4A3A2C", matte),
    bracket = ("#3A3D41", metal), // the planter box's steel corner brackets and screws
    leaf = ("#4F8A3E", foliage),
    leaf_dark = ("#3B7034", foliage),
    leaf_light = ("#6FAA52", foliage),
    bloom = ("#E07BC3", satin),
    bloom2 = ("#F2C14E", satin),
    bloom3 = ("#F2F0EA", satin),
    bloom4 = ("#D9483B", satin),
    bloom5 = ("#9B7FD1", satin),
    eye = ("#F2C14E", satin),
    // the birdhouse, the feeders and the birds
    post = ("#8E6E52", wood),
    roof = ("#C0473A", satin),
    dark = ("#2B2C2E", matte), // openings: the birdhouse's hole, the kennel's door
    iron = ("#2A2C30", metal), // the shepherd's hook, the lamp post
    feeder = ("#2F5A4E", satin), // the feeders' caps and trays
    feeder_glass = ("#DCE6EA", glass),
    seed = ("#C9A979", matte),
    robin = ("#7A5A40", satin),
    robin_breast = ("#D9603B", satin),
    tit = ("#3E6EB0", satin),
    tit_breast = ("#F2D14E", satin),
    beak = ("#3A3026", satin),
    // the dog house
    plinth = ("#6E6A64", matte),
    plate = ("#F2F0EA", satin),
    bowl = ("#3E8EDE", gloss),
    bowl_water = ("#9DC6CF", gloss),
    // the sundial
    dial = ("#B87333", metal), // copper
    dial_mark = ("#6E4A2A", metal),
    lichen = ("#C8C29A", matte), // pale patches on the sundial's steps
    moss = ("#5E7A3A", matte), // moss at the stepping stones' edges (plain: no foliage draw)
    // the lights
    lantern = ("#2F3236", metal), // the lamp's lantern frame and roof (the player's "Lantern")
    pane = ("#F6EBCF", satin, glow("#FFD596")), // frosted panes, lit at night
    planter = ("#A9A49A", matte), // the string lights' weighted pots
    wire = ("#2A2C30", matte),
    socket = ("#3A3D41", satin),
    bulb = ("#FFF1CF", gloss, glow("#FFD9A0")),
    // (each model's own: `BODIES`)
    body = ("#B4AEA2", matte),
}

/// Each model's `body` (the part the player colours) differs in colour and finish: stones, a wooden box, a cream
/// birdhouse, a red kennel, a stone plinth, iron posts. `_own` makes the palette's `body` that one for the model
/// being built; the pack's `materials` follow.
pub const BODIES: [(&str, &str, &str); 9] = [
    ("pond", "#B4AEA2", "matte"),
    ("planterBox", "#8A6A4E", "wood"),
    ("birdhouse", "#E9DCC4", "satin"),
    ("birdFeederPole", "#2A2C30", "metal"),
    ("dogHouse", "#B5653F", "wood"),
    ("sundial", "#C9C2B4", "matte"),
    ("steppingStones", "#ABA59A", "matte"),
    ("lampPost", "#2F3236", "metal"),
    ("stringLights", "#8E6E52", "wood"),
];

thread_local! {
    /// The palette of the model being built on this thread.
    static CURRENT: Cell<SPEC> = const { Cell::new(SPEC) };
}

/// The palette for model `id`: the family's colours and its own `body`.
pub fn palette_of(id: &str) -> SPEC {
    let (_, hex, finish) = BODIES
        .iter()
        .find(|b| b.0 == id)
        .unwrap_or_else(|| panic!("no body for {id:?}"));
    let mut m = SPEC;
    m.body = Mat::new("body", hex, finish);
    m
}

/// The palette for model `id`, made the one of the model being built (`M`).
pub fn _own(id: &str) -> SPEC {
    let m = palette_of(id);
    CURRENT.with(|c| c.set(m));
    m
}

/// The palette of the model being built.
#[allow(non_snake_case)]
pub fn M() -> SPEC {
    CURRENT.with(Cell::get)
}

pub const F: GameFrame = GameFrame::new(0.0);

/// {object id: light.height} from the content: the bulbs are placed against it.
fn _content_lights() -> Vec<(String, f64)> {
    let mut out: Vec<(String, f64)> = Vec::new();
    for text in [
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../web/public/content/furniture.json"
        )),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../web/public/content/catalog2/outdoor.json"
        )),
    ] {
        let content = json::loads(text).unwrap_or_else(|e| panic!("the content: {e}"));
        let Some(Json::Arr(objects)) = content.get("objects") else {
            continue;
        };
        for o in objects {
            let Some(height) = o
                .get("light")
                .and_then(|l| l.get("height"))
                .and_then(Json::num)
            else {
                continue;
            };
            let Some(id) = o.get("id").and_then(Json::str) else {
                continue;
            };
            match out.iter_mut().find(|(k, _)| k == id) {
                Some(entry) => entry.1 = height,
                None => out.push((id.to_string(), height)),
            }
        }
    }
    out
}

/// Asserts a bulb at height `y` is within 25 cm of where the content puts the lamp's light.
pub fn _near_light(id: &str, y: f64) {
    static LIGHTS: OnceLock<Vec<(String, f64)>> = OnceLock::new();
    let want = LIGHTS
        .get_or_init(_content_lights)
        .iter()
        .find(|(k, _)| k == id)
        .unwrap_or_else(|| panic!("{id}: the content gives it no light"))
        .1;
    assert!(
        (y - want).abs() <= 0.25,
        "{id}: bulb at {y:.2} m, the content's light at {want:.2} m"
    );
}

/// A part of the points `verts` and `faces`, wound as they are given.
pub fn _mesh(name: &str, verts: Vec<V3>, faces: &[Vec<u32>], material: Mat, smooth: bool) -> Obj {
    let mut mesh = Mesh::from_pydata(verts, faces);
    mesh.set_material(Some(material));
    for f in &mut mesh.faces {
        f.smooth = smooth;
    }
    scene::add(name, mesh)
}

/// A flat shape `outline` [(game x, height)], counter-clockwise seen from the front, standing from game depth
/// `z0` back to `z1` (a gable, a gnomon).
pub fn _prism(name: &str, outline: &[(f64, f64)], z0: f64, z1: f64, material: Mat) -> Obj {
    let n = outline.len() as u32;
    let mut verts: Vec<V3> = outline
        .iter()
        .map(|&(x, y)| F.at(x, y, z1).into())
        .collect();
    verts.extend(outline.iter().map(|&(x, y)| V3::from(F.at(x, y, z0))));
    let mut faces: Vec<Vec<u32>> = vec![(0..n).rev().collect(), (n..2 * n).collect()];
    for i in 0..n {
        let j = (i + 1) % n;
        faces.push(vec![i, j, n + j, n + i]);
    }
    let obj = _mesh(name, verts, &faces, material, false);
    obj.edit(|o| o.mesh.recalc_normals());
    obj
}

/// A little bird perched at height `y`, looking to the front (or `turn` degrees round towards +x).
#[allow(clippy::too_many_arguments)]
pub fn _bird(x: f64, y: f64, z: f64, body: Mat, breast: Mat, turn: f64) {
    let (c, s) = (cosd(turn), sind(turn));
    fsphere!(
        F,
        numbered("bird_body"),
        0.5,
        (x, y + 0.03, z),
        body,
        scale = (0.055, 0.085, 0.055),
        rot = (0, 0, -turn),
        segments = 12,
        rings = 8
    );
    fsphere!(
        F,
        numbered("bird_head"),
        0.023,
        (x + 0.034 * s, y + 0.072, z + 0.034 * c),
        body,
        segments = 10,
        rings = 7
    );
    fsphere!(
        F,
        numbered("bird_breast"),
        0.5,
        (x + 0.013 * s, y + 0.026, z + 0.016 * c),
        breast,
        scale = (0.045, 0.05, 0.05),
        segments = 10,
        rings = 6
    );
    rod!(
        numbered("beak"),
        F.at(x + 0.05 * s, y + 0.074, z + 0.05 * c),
        F.at(x + 0.068 * s, y + 0.071, z + 0.068 * c),
        0.006,
        material = M().beak,
        verts = 5
    );
    rod!(
        numbered("tail"),
        F.at(x - 0.035 * s, y + 0.035, z - 0.035 * c),
        F.at(x - 0.078 * s, y + 0.052, z - 0.078 * c),
        0.012,
        material = body,
        verts = 5
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_model_has_its_own_body_and_the_lamps_their_lights() {
        for (id, hex, finish) in BODIES {
            let m = _own(id);
            assert_eq!((m.body.name, m.body.finish), ("body", finish));
            assert_eq!(artkit::materials::hex_of(M().body.colour), hex);
        }
        _near_light("lampPost", 2.3);
        _near_light("stringLights", 2.0);
    }
}

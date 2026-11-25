//! The game's item models, generated: yards, playgrounds, farms, street furniture, stalls, shops and the named places'
//! fittings. One folder a family, each listed in `families()`.

use artkit::bevel::{EdgeOrder, bevel_edges};
use artkit::family::Family;
use artkit::far::is_low;
use artkit::math::V3;
use artkit::scene::Obj;
use artkit::shapes::AUTO_WIDTH;

pub mod civic;
pub mod civic_works;
pub mod farm;
pub mod fields;
pub mod outdoor;
pub mod playground;
pub mod shops;
pub mod stalls;
pub mod street_furniture;
pub mod water_edge;
pub mod yard;

/// The group's families by their names on the command line (`cargo art <name>`).
pub fn families() -> Vec<(&'static str, Family)> {
    vec![
        ("yard", yard::family()),
        ("outdoor", outdoor::family()),
        ("playground", playground::family()),
        ("farm", farm::family()),
        ("fields", fields::family()),
        ("street_furniture", street_furniture::family()),
        ("stalls", stalls::family()),
        ("shops", shops::family()),
        ("civic", civic::family()),
        ("civic_works", civic_works::family()),
        ("water_edge", water_edge::family()),
    ]
}

/// `obj` with its edges turning over 50° chamfered by a pixel at the play camera (what `cube!` gives a box), none
/// in a far version (a far version built by hand runs in `far::far_of`) nor on a part the gate's `edges` can't find hard: under
/// 12 mm one way (its edges' faces too narrow) or 10 cm every way (a detail, not a piece).
pub fn eased(obj: Obj) -> Obj {
    ease(obj, 50.0, 1)
}

/// `eased` on the edges turning over `angle` degrees, in `segments`: a lower angle where a turn the gate finds
/// meets one under 50° at a corner (a moulding's mitre), which a bevel can't stop at; two segments on a lip
/// turning past 120° (one would leave two turns of 60° and more).
pub fn ease(obj: Obj, angle: f64, segments: usize) -> Obj {
    let points = obj.world_points();
    let side = |k: fn(&V3) -> f32| {
        let (lo, hi) = points
            .iter()
            .map(k)
            .fold((f32::MAX, f32::MIN), |(lo, hi), v| (lo.min(v), hi.max(v)));
        f64::from(hi - lo)
    };
    let sides = [side(|p| p.x), side(|p| p.y), side(|p| p.z)];
    if is_low() || sides.iter().any(|&s| s < 0.0119) || sides.iter().all(|&s| s < 0.099) {
        return obj;
    }
    bevelled(obj, angle, segments)
}

/// `ease` whatever the part's size: a thin disc whose rim the gate finds all the same.
pub fn bevelled(obj: Obj, angle: f64, segments: usize) -> Obj {
    if is_low() {
        return obj;
    }
    // (the gate's `edges` finds turns of 60° or more: a rounded corner's 45° steps are left as they are)
    // (a profile closed on its first point, a revolve's, leaves a seam the bevel can't cross: welded first)
    obj.edit(|o| o.mesh.remove_doubles(1e-6));
    bevel_edges(obj, AUTO_WIDTH, segments, angle, &EdgeOrder::Corners)
}

/// `obj` given its bevel now and its box UVs laid on the bevelled mesh, a face at a time, so a chamfer's strip
/// takes the texture as its faces do (carried, as Blender carries them, the strip reads its edge's line smeared
/// across it: the gate's `texels` finds a rail's chamfers "smeared").
pub fn boxed(obj: Obj) -> Obj {
    artkit::bevel::apply(obj);
    obj.edit(|o| {
        artkit::uv::box_(&mut o.mesh);
        o.mesh.uv_metres = true;
    });
    obj
}

/// A content object by its id, from `file` under web/public/content: for the families' content-facts tests.
#[cfg(test)]
fn content_object(file: &str, id: &str) -> artkit::json::Json {
    use artkit::json::{Json, loads};
    let path = format!(
        "{}/../../../web/public/content/{file}",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let content = loads(&text).unwrap_or_else(|e| panic!("{path}: {e}"));
    let Some(Json::Arr(objects)) = content.get("objects") else {
        panic!("{file}: no objects")
    };
    objects
        .iter()
        .find(|o| o.get("id").and_then(Json::str) == Some(id))
        .unwrap_or_else(|| panic!("{id} in {file}"))
        .clone()
}

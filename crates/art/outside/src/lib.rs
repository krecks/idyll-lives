//! The game's item models, generated: yards, playgrounds, farms, street furniture, stalls, shops and the named places'
//! fittings. One folder a family, each listed in `families()`.

use artkit::family::Family;

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

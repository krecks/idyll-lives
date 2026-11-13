//! `cargo world-map`: draws the sample worlds (`tools/worlds/sample.ts`'s output) from above, one SVG
//! each, to see how a kind of world is laid out (was `tools/worlds/map.ts`): its roads (or an older
//! town's streets) and greens, the lots (public ones blue, farms' fields yellow, outdoor places orange;
//! a tick on the side they face), garden paths and driveways, the ground floor's walls, the things
//! standing about and the ways out of town (red). A big world also shows its land (water, woods, hills,
//! meadows, fields, orchards), its country roads and trails (dashed), and its areas' names; and the
//! living town (docs/design/living-town.md): its backdrop buildings (coloured by use, darker the more
//! storeys), each main town's centre (paved, dashed round), and the places the tags name (a dot at the
//! door, blue for a public lot, red for a backdrop place, with its name). An index.html shows them all
//! on one page.
//!
//! `cargo world-map worlds.json out-dir [more.json ...]`

use serde_json::Value as J;
use std::path::Path;

pub const USAGE: &str = "usage: cargo world-map <worlds.json> <out-dir> [more.json ...]";

/// 1.5 px a metre, labels in pixels whatever the scale.
const SCALE: f64 = 1.5;

fn px(n: f64) -> f64 {
    n / SCALE
}

/// A number as JavaScript writes it (whole numbers without a point).
fn n(v: f64) -> String {
    format!("{v}")
}

fn num(j: &J, key: &str) -> f64 {
    j.get(key).and_then(J::as_f64).unwrap_or(f64::NAN)
}

fn list<'a>(j: &'a J, key: &str) -> &'a [J] {
    j.get(key).and_then(J::as_array).map_or(&[], Vec::as_slice)
}

fn text<'a>(j: &'a J, key: &str) -> &'a str {
    j.get(key).and_then(J::as_str).unwrap_or("")
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;")
}

fn rect(r: &J, style: &str) -> String {
    format!(
        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" {style}/>",
        n(num(r, "x")),
        n(num(r, "z")),
        n(num(r, "w")),
        n(num(r, "d"))
    )
}

/// Points as `x,z x,z`.
fn line(points: &J) -> String {
    points
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .map(|p| {
            p.as_array()
                .map(Vec::as_slice)
                .unwrap_or_default()
                .iter()
                .map(|v| n(v.as_f64().unwrap_or(f64::NAN)))
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn zone_colour(kind: &str) -> &'static str {
    match kind {
        "water" => "#7fb2d9",
        "woods" => "#5d8a46",
        "hill" => "#b7c48a",
        "meadow" => "#c3d98f",
        "fields" => "#e2c86b",
        "orchard" => "#7fa653",
        _ => "#999",
    }
}

/// Backdrop buildings' colours, by use.
fn use_colour(kind: &str) -> &'static str {
    match kind {
        "flats" => "#c9a27e",
        "terrace" => "#d6b48e",
        "house" => "#e3c9a0",
        "shop" => "#e98f6b",
        "cafe" => "#f0b35a",
        "restaurant" => "#d9714e",
        "office" => "#9fb3c8",
        "school" => "#e5d36b",
        "church" => "#bfa6d6",
        "hall" => "#c7b79a",
        "workshop" => "#a3a392",
        "farm" => "#c98f5a",
        "barn" => "#a8583e",
        _ => "#999",
    }
}

/// One world's map as an SVG.
pub fn svg(world: &J) -> String {
    let (kind, name, town) = (text(world, "kind"), text(world, "name"), &world["town"]);
    let seed = world
        .get("seed")
        .map_or(String::new(), |s| n(s.as_f64().unwrap_or(f64::NAN)));
    let (width, depth) = (num(town, "width"), num(town, "depth"));
    let meta = &town["meta"];
    // Only the ground floor (the storeys above and the basement are on rows further down).
    let ground = |z: f64| z <= depth;
    let roads = list(meta, "roads");
    let trail = |r: &&J| text(r, "kind") == "trail";
    let order = ["meadow", "fields", "hill", "woods", "orchard", "water"];
    let mut zones: Vec<&J> = list(meta, "zones").iter().collect();
    zones.sort_by_key(|z| {
        order
            .iter()
            .position(|k| *k == text(z, "kind"))
            .map_or(-1, |i| i as i64)
    });
    let plots = list(town, "plots");
    let places = list(meta, "places");
    let areas = list(meta, "areas");
    let mut parts: Vec<String> = Vec::new();
    parts.push(format!(
        "<rect width=\"{}\" height=\"{}\" fill=\"#9fc27a\"/>",
        n(width),
        n(depth)
    ));
    for z in zones {
        let kind = text(z, "kind");
        parts.push(format!(
            "<polygon points=\"{}\" fill=\"{}\" fill-rule=\"evenodd\" opacity=\"{}\"/>",
            line(&z["poly"]),
            zone_colour(kind),
            if kind == "hill" { "0.6" } else { "0.85" }
        ));
    }
    for a in areas {
        parts.push(rect(
            a,
            &format!(
                "fill=\"none\" stroke=\"#4d6b33\" stroke-width=\"{}\" stroke-dasharray=\"{} {}\"",
                n(px(1.0)),
                n(px(4.0)),
                n(px(4.0))
            ),
        ));
    }
    for r in list(&meta["world"], "fields") {
        parts.push(rect(r, "fill=\"#e2c86b\""));
    }
    for g in list(meta, "greens") {
        parts.push(format!(
            "<polygon points=\"{}\" fill=\"#7fb35e\"/>",
            line(g)
        ));
    }
    for c in list(meta, "centre") {
        parts.push(format!(
            "<polygon points=\"{}\" fill=\"#e6dcc8\" opacity=\"0.75\" stroke=\"#8a6d3b\" stroke-width=\"{}\" stroke-dasharray=\"{} {}\"/>",
            line(&c["poly"]),
            n(px(1.5)),
            n(px(6.0)),
            n(px(3.0))
        ));
    }
    for p in plots {
        let fill = if text(p, "kind") == "countryside" {
            "#f2b366"
        } else if p.get("public").and_then(J::as_bool).unwrap_or(false) {
            "#a9c7e8"
        } else {
            "#b9d98f"
        };
        parts.push(rect(
            p,
            &format!("fill=\"{fill}\" stroke=\"#4d6b33\" stroke-width=\"0.3\""),
        ));
    }
    // The side each lot faces (its street's).
    let facing = list(meta, "facing");
    for (i, p) in plots.iter().enumerate() {
        let (x, z, w, d) = (num(p, "x"), num(p, "z"), num(p, "w"), num(p, "d"));
        // (Whole numbers, 0 to 3: the side's index.)
        let side = facing
            .get(i)
            .and_then(J::as_f64)
            .filter(|f| f.fract() == 0.0)
            .map(|f| f as i64);
        let at = match side {
            Some(0) => Some((x + w / 2.0, z)),
            Some(1) => Some((x, z + d / 2.0)),
            Some(2) => Some((x + w / 2.0, z + d)),
            Some(3) => Some((x + w, z + d / 2.0)),
            _ => None,
        };
        parts.push(at.map_or(String::new(), |(x, z)| {
            format!(
                "<circle cx=\"{}\" cy=\"{}\" r=\"0.9\" fill=\"#4d6b33\"/>",
                n(x),
                n(z)
            )
        }));
    }
    for r in list(meta, "streets") {
        parts.push(rect(r, "fill=\"#6b6b6b\""));
    }
    // Roads by kind: streets and lanes grey with their sidewalks, country roads brown, trails dashed.
    for (under, over, less) in [("#cdbb98", "#d9d4c8", 0.0), ("#8a7350", "#6b6b6b", 2.7)] {
        for r in roads.iter().filter(|r| !trail(r)) {
            parts.push(format!(
                "<polyline points=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\" stroke-linejoin=\"round\"/>",
                line(&r["points"]),
                if text(r, "kind") == "country" { under } else { over },
                n(num(r, "width") - less)
            ));
        }
    }
    for r in roads.iter().filter(trail) {
        parts.push(format!(
            "<polyline points=\"{}\" fill=\"none\" stroke=\"#7a5a32\" stroke-width=\"{}\" stroke-dasharray=\"{} {}\" stroke-linecap=\"round\"/>",
            line(&r["points"]),
            n((num(r, "width") * 0.6).max(px(1.6))),
            n(px(5.0)),
            n(px(3.0))
        ));
    }
    for r in list(meta, "paths") {
        parts.push(rect(r, "fill=\"#d8d2c4\""));
    }
    for o in list(town, "objects").iter().filter(|o| ground(num(o, "z"))) {
        parts.push(format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"0.35\" fill=\"#5a4a3a\"/>",
            n(num(o, "x") + 0.5),
            n(num(o, "z") + 0.5)
        ));
    }
    for w in list(town, "walls") {
        let v: Vec<f64> = w
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or_default()
            .iter()
            .map(|c| c.as_f64().unwrap_or(f64::NAN))
            .collect();
        if v.len() >= 4 && ground(v[1]) && ground(v[3]) {
            parts.push(format!(
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"#7a2e1f\" stroke-width=\"0.35\"/>",
                n(v[0]),
                n(v[1]),
                n(v[2]),
                n(v[3])
            ));
        }
    }
    // The backdrop: by use, darker the taller.
    for b in list(meta, "backdrop") {
        parts.push(format!(
            "<polygon points=\"{}\" fill=\"{}\" fill-opacity=\"{}\" stroke=\"#4a3426\" stroke-width=\"0.35\"/>",
            line(&b["poly"]),
            use_colour(text(b, "use")),
            n((0.55 + num(b, "storeys") * 0.1).min(1.0))
        ));
    }
    for p in places {
        parts.push(format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\"/>",
            n(num(p, "x")),
            n(num(p, "z")),
            n(px(2.5).max(0.9)),
            if p.get("plot").is_none() {
                "#c0392b"
            } else {
                "#2b5fa8"
            }
        ));
    }
    for p in places {
        parts.push(format!(
            "<text x=\"{}\" y=\"{}\" font-size=\"{}\" text-anchor=\"middle\" font-family=\"sans-serif\" fill=\"#2a1c12\" stroke=\"#fff\" stroke-width=\"{}\" paint-order=\"stroke\">{}</text>",
            n(num(p, "x")),
            n(num(p, "z") - px(4.0)),
            n(px(7.0)),
            n(px(1.5)),
            esc(text(p, "name"))
        ));
    }
    for p in list(town, "parking") {
        parts.push(format!(
            "<rect x=\"{}\" y=\"{}\" width=\"3\" height=\"3\" fill=\"#3f6fb5\"/>",
            n(num(p, "x") - 1.5),
            n(num(p, "z") - 1.5)
        ));
    }
    for e in list(town, "exits") {
        let (x, z) = match e.as_array() {
            Some(pair) => (
                pair.first().and_then(J::as_f64).unwrap_or(f64::NAN),
                pair.get(1).and_then(J::as_f64).unwrap_or(f64::NAN),
            ),
            None => (num(e, "x"), num(e, "z")),
        };
        parts.push(format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"#d33\"/>",
            n(x),
            n(z),
            n(px(5.0).max(1.5))
        ));
    }
    // Outdoor places' and areas' names.
    for p in plots.iter().filter(|p| text(p, "kind") == "countryside") {
        parts.push(format!(
            "<text x=\"{}\" y=\"{}\" font-size=\"{}\" text-anchor=\"middle\" font-family=\"sans-serif\" fill=\"#5a3a12\">{}</text>",
            n(num(p, "x") + num(p, "w") / 2.0),
            n(num(p, "z") - px(4.0)),
            n(px(11.0)),
            esc(text(p, "name"))
        ));
    }
    for a in areas {
        parts.push(format!(
            "<text x=\"{}\" y=\"{}\" font-size=\"{}\" font-weight=\"700\" text-anchor=\"middle\" font-family=\"sans-serif\" fill=\"#1d2230\" stroke=\"#fff\" stroke-width=\"{}\" paint-order=\"stroke\">{}</text>",
            n(num(a, "x") + num(a, "w") / 2.0),
            n(num(a, "z") + num(a, "d") / 2.0),
            n(px(18.0)),
            n(px(3.0)),
            esc(text(a, "name"))
        ));
    }
    parts.push(format!(
        "<text x=\"{}\" y=\"{}\" font-size=\"{}\" font-family=\"sans-serif\">{kind} {seed}: {} ({}×{} m)</text>",
        n(px(8.0)),
        n(px(22.0)),
        n(px(16.0)),
        esc(name),
        n(width),
        n(depth)
    ));
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\">{}</svg>",
        n(width),
        n(depth),
        n((width * SCALE).round()),
        n((depth * SCALE).round()),
        parts.concat()
    )
}

fn run(input: &str, dir: &str, more: &[&str]) -> Result<usize, String> {
    let mut worlds: Vec<J> = Vec::new();
    for file in std::iter::once(&input).chain(more) {
        let text = std::fs::read(file).map_err(|e| format!("{file}: {e}"))?;
        match serde_json::from_slice::<J>(&text).map_err(|e| format!("{file}: {e}"))? {
            J::Array(list) => worlds.extend(list),
            _ => return Err(format!("{file}: not a list of worlds")),
        }
    }
    let dir = Path::new(dir);
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut cards = String::new();
    for world in &worlds {
        let (kind, name, town) = (text(world, "kind"), text(world, "name"), &world["town"]);
        let seed = n(num(world, "seed"));
        let file = format!("{kind}-{seed}.svg");
        std::fs::write(dir.join(&file), svg(world)).map_err(|e| format!("{file}: {e}"))?;
        cards.push_str(&format!(
            "<figure><a href=\"{file}\"><img src=\"{file}\" loading=\"lazy\" alt=\"{}\"></a><figcaption>{kind} {seed}: {}, {}×{} m, {} plots</figcaption></figure>",
            esc(name),
            esc(name),
            n(num(town, "width")),
            n(num(town, "depth")),
            list(town, "plots").len()
        ));
    }
    std::fs::write(
        dir.join("index.html"),
        format!(
            "<!doctype html><meta charset=\"utf-8\"><title>Sample worlds</title><style>body{{font:13px sans-serif;margin:16px;background:#f4f1ec}}main{{display:grid;grid-template-columns:repeat(auto-fill,minmax(320px,1fr));gap:12px}}figure{{margin:0;background:#fff;padding:6px;border-radius:6px}}img{{width:100%;height:auto;display:block}}</style><h1>Sample worlds ({})</h1><main>{cards}</main>",
            worlds.len()
        ),
    )
    .map_err(|e| format!("index.html: {e}"))?;
    Ok(worlds.len())
}

pub fn main_with(args: &[String]) -> i32 {
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    match a.as_slice() {
        ["-h" | "--help" | "help", ..] => {
            println!("{USAGE}");
            0
        }
        [input, dir, more @ ..] => match run(input, dir, more) {
            Ok(count) => {
                println!("{count} maps → {dir} (index.html)");
                0
            }
            Err(e) => {
                eprintln!("world-map: {e}");
                1
            }
        },
        _ => {
            eprintln!("{USAGE}");
            2
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_world_is_drawn_part_by_part() {
        let world = json!({ "kind": "big", "seed": 7, "name": "Birch & <Co>", "town": {
            "width": 300, "depth": 200,
            "walls": [[1, 2, 3, 2], [1, 250, 3, 250]],
            "objects": [{ "x": 4, "z": 5 }, { "x": 4, "z": 260 }],
            "plots": [{ "x": 10, "z": 20, "w": 12, "d": 9, "name": "Home", "public": false }, { "x": 40, "z": 20, "w": 6, "d": 6, "name": "Pond", "public": true, "kind": "countryside" }],
            "exits": [[0, 50], { "x": 300, "z": 60, "kind": "road" }],
            "meta": {
                "facing": [2, 9],
                "roads": [{ "points": [[0, 50], [300, 50.5]], "width": 9, "kind": "street" }, { "points": [[5, 5], [9, 9]], "width": 2, "kind": "trail" }],
                "zones": [{ "kind": "water", "poly": [[1, 1], [2, 1], [2, 2]] }, { "kind": "hill", "poly": [[3, 3], [4, 3], [4, 4]] }],
                "places": [{ "name": "Bakery", "x": 12, "z": 30 }, { "name": "School", "x": 14, "z": 31, "plot": 1 }]
            }
        } });
        let svg = svg(&world);
        assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 300 200\" width=\"450\" height=\"300\"><rect width=\"300\" height=\"200\" fill=\"#9fc27a\"/>"));
        // The land in its order (a hill under the water), the lot's tick on the side it faces, one wall and one thing (the ground floor's).
        assert!(svg.find("fill=\"#b7c48a\"").unwrap() < svg.find("fill=\"#7fb2d9\"").unwrap());
        assert!(svg.contains("<circle cx=\"16\" cy=\"29\" r=\"0.9\" fill=\"#4d6b33\"/>"));
        assert_eq!(svg.matches("stroke=\"#7a2e1f\"").count(), 1);
        assert_eq!(svg.matches("fill=\"#5a4a3a\"").count(), 1);
        assert!(svg.contains("stroke-width=\"6.3\" stroke-linejoin=\"round\""));
        assert!(svg.contains("stroke-dasharray=\"3.3333333333333335 2\" stroke-linecap=\"round\""));
        assert!(svg.contains("fill=\"#c0392b\"") && svg.contains("fill=\"#2b5fa8\""));
        assert!(svg.ends_with(">big 7: Birch &amp; &lt;Co> (300×200 m)</text></svg>"));
    }
}

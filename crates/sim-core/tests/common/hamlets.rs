//! A synthetic open-world town for the travel tests: hamlets built like `town_with`'s streets,
//! set apart in open country and joined by a winding country road, with the open-world fields
//! of docs/design/open-world.md (`roads` with kinds, `areas`, typed `exits` with a bus stop in
//! each hamlet, `size`).

use serde_json::{Value, json};

use super::{Layout, town_with};

/// A hamlet: where its `town_with` block goes in the world.
#[derive(Debug, Clone, Copy)]
pub struct Place {
    pub x: i64,
    pub z: i64,
}

/// What `hamlets` built, besides the town file.
#[derive(Debug, Clone)]
pub struct Hamlets {
    pub town: String,
    /// Per hamlet: its plots (indices) and its street's middle (z) and ends (x).
    pub plots: Vec<std::ops::Range<usize>>,
    pub streets: Vec<(f64, f64, f64)>,
    /// The country road's points.
    pub country: Vec<[f64; 2]>,
}

/// The street of a `town_with` block with one street: its middle (z), plot rows either side.
fn street_z() -> f64 {
    // MARGIN + plot depth + half the street (see `town_with`).
    8.0 + 22.0 + 3.0
}

/// A line of points about a metre apart from `a` to `b`.
pub fn line(a: [f64; 2], b: [f64; 2]) -> Vec<[f64; 2]> {
    let n = ((a[0] - b[0]).hypot(a[1] - b[1])).ceil().max(1.0) as usize;
    (0..=n)
        .map(|i| {
            let t = i as f64 / n as f64;
            [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
        })
        .collect()
}

/// Hamlets of `columns` plots a side at `places`, `width × depth` in all; the first and the last
/// joined to each other (and on) by a country road with a bend, a road out at the far end of each
/// end hamlet, and a bus stop in the middle of every street.
pub fn hamlets(
    content: &str,
    seed: u64,
    columns: i64,
    places: &[Place],
    (width, depth): (i64, i64),
) -> Hamlets {
    let mut out = json!({
        "width": width, "depth": depth, "size": "big",
        "walls": [], "doors": [], "windows": [], "objects": [],
        "plots": [], "households": [], "sims": [], "relationships": [],
    });
    let mut roads = Vec::new();
    let mut exits = Vec::new();
    let mut areas = Vec::new();
    let mut plots = Vec::new();
    let mut streets = Vec::new();
    for (k, at) in places.iter().enumerate() {
        let block: Value = serde_json::from_str(&town_with(
            content,
            seed + k as u64,
            if k == 0 { 2 } else { 0 },
            Layout {
                columns,
                streets: 1,
                yards: false,
                max_residents: usize::MAX,
            },
        ))
        .unwrap();
        let (dx, dz) = (at.x, at.z);
        let (bw, bd) = (
            block["width"].as_i64().unwrap(),
            block["depth"].as_i64().unwrap(),
        );
        let shift = |v: &Value, keys: [&str; 2]| {
            let mut v = v.clone();
            for (key, d) in [(keys[0], dx), (keys[1], dz)] {
                v[key] = match v[key].as_i64() {
                    Some(n) => json!(n + d),
                    None => json!(v[key].as_f64().unwrap() + d as f64),
                };
            }
            v
        };
        let first_plot = out["plots"].as_array().unwrap().len();
        let first_household = out["households"].as_array().unwrap().len();
        let first_sim = out["sims"].as_array().unwrap().len();
        for w in block["walls"].as_array().unwrap() {
            let c: Vec<i64> = w
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_i64().unwrap())
                .collect();
            out["walls"].as_array_mut().unwrap().push(json!([
                c[0] + dx,
                c[1] + dz,
                c[2] + dx,
                c[3] + dz
            ]));
        }
        for key in ["doors", "windows", "objects", "sims"] {
            for v in block[key].as_array().unwrap() {
                let mut v = shift(v, ["x", "z"]);
                if key == "sims" {
                    v["household"] =
                        json!(v["household"].as_u64().unwrap() as usize + first_household);
                    if k > 0 {
                        v["name"] = json!(format!("H{k}{}", v["name"].as_str().unwrap()));
                    }
                }
                out[key].as_array_mut().unwrap().push(v);
            }
        }
        for p in block["plots"].as_array().unwrap() {
            let mut p = shift(p, ["x", "z"]);
            if let Some(e) = p["entry"].as_array() {
                let e = [
                    e[0].as_f64().unwrap() + dx as f64,
                    e[1].as_f64().unwrap() + dz as f64,
                ];
                p["entry"] = json!(e);
            }
            p["area"] = json!(k);
            out["plots"].as_array_mut().unwrap().push(p);
        }
        for h in block["households"].as_array().unwrap() {
            let mut h = h.clone();
            h["plot"] = json!(h["plot"].as_u64().unwrap() as usize + first_plot);
            if k > 0 {
                h["player"] = json!(false);
            }
            out["households"].as_array_mut().unwrap().push(h);
        }
        for r in block["relationships"].as_array().unwrap() {
            let mut r = r.clone();
            r["a"] = json!(r["a"].as_u64().unwrap() as usize + first_sim);
            r["b"] = json!(r["b"].as_u64().unwrap() as usize + first_sim);
            out["relationships"].as_array_mut().unwrap().push(r);
        }
        plots.push(first_plot..out["plots"].as_array().unwrap().len());
        let z = street_z() + dz as f64;
        let (x0, x1) = (dx as f64, (dx + bw) as f64);
        streets.push((z, x0, x1));
        let pts = line([x0 + 0.5, z], [x1 - 0.5, z]);
        roads.push(
            json!({"points": pts, "width": 6, "ends": ["out", "out"], "kind": "street", "area": k}),
        );
        exits.push(json!({"x": (x0 + x1) / 2.0, "z": z + 2.5, "kind": "stop"}));
        areas.push(json!({"name": format!("Hamlet {k}"), "kind": "hamlet", "x": dx, "z": dz, "w": bw, "d": bd}));
    }
    // Country roads from each hamlet's east end to the next one's west end, with two bends.
    let mut country = Vec::new();
    for k in 1..places.len() {
        let (za, _, xa) = streets[k - 1];
        let (zb, xb, _) = streets[k];
        let mid = (xa + xb) / 2.0;
        let mut pts = line([xa - 0.5, za], [mid, za]);
        pts.extend(line([mid, za], [mid, zb]).into_iter().skip(1));
        pts.extend(line([mid, zb], [xb + 0.5, zb]).into_iter().skip(1));
        country.extend(pts.iter().copied());
        roads.push(json!({"points": pts, "width": 5, "ends": ["join", "join"], "kind": "country"}));
    }
    let (z0, x0, _) = streets[0];
    let (zn, _, xn) = *streets.last().unwrap();
    exits.push(json!([x0 + 0.5, z0]));
    exits.push(json!([xn - 0.5, zn]));
    out["roads"] = Value::Array(roads.clone());
    out["meta"] = json!({"roads": roads});
    out["exits"] = Value::Array(exits);
    out["areas"] = Value::Array(areas);
    Hamlets {
        town: out.to_string(),
        plots,
        streets,
        country,
    }
}

/// Two hamlets of three plots a side, 160 m apart across open country.
pub fn two_hamlets(content: &str, seed: u64) -> Hamlets {
    hamlets(
        content,
        seed,
        3,
        &[Place { x: 0, z: 0 }, Place { x: 162, z: 60 }],
        (256, 128),
    )
}

/// `h` with a driveway in the front corner of every house's plot (the strip clear of the house
/// layouts) and a car on each, every other one parked nose in.
pub fn with_cars(h: &Hamlets) -> String {
    let mut town: Value = serde_json::from_str(&h.town).unwrap();
    let (mut drives, mut cars) = (Vec::new(), Vec::new());
    let plots = town["plots"].as_array().unwrap().clone();
    for (i, p) in plots.iter().enumerate() {
        if p["public"].as_bool() == Some(true) {
            continue;
        }
        let area = p["area"].as_u64().unwrap() as usize;
        let street = h.streets[area].0;
        let (x, z, w, d) = (
            p["x"].as_f64().unwrap(),
            p["z"].as_f64().unwrap(),
            p["w"].as_f64().unwrap(),
            p["d"].as_f64().unwrap(),
        );
        // The street in front: the north row (turned) faces +z, the south row -z.
        let north = z + d <= street;
        let (cx, inner, front) = if north {
            (x + w - 1.5, z + d - 5.5, 1.0)
        } else {
            (x + 1.5, z + 5.5, -1.0)
        };
        let (len, gap) = (4.4, 0.45);
        let spot = inner + front * (gap + len / 2.0);
        // Facing the street (yaw π heads -z), or nose in.
        let facing_street = if front > 0.0 {
            0.0
        } else {
            std::f64::consts::PI
        };
        let yaw = if i % 2 == 0 {
            facing_street
        } else {
            facing_street - std::f64::consts::PI
        };
        drives.push(json!({"plot": i, "x": cx, "z": inner, "front": front, "street": street}));
        cars.push(json!({"plot": i, "x": cx, "z": spot, "yaw": yaw, "street": street, "len": len, "wid": 1.8, "body": i % 3, "paint": i % 5}));
    }
    town["drives"] = Value::Array(drives);
    town["cars"] = Value::Array(cars);
    town.to_string()
}

/// Where `with_countryside` puts its countryside plot (`x, z, w, d`), its trail and its pond.
pub const MEADOW: (i64, i64, i64, i64) = (100, 44, 24, 36);
pub const POND: [(i64, i64); 4] = [(104, 50), (105, 50), (104, 51), (105, 51)];

/// The trail of `with_countryside`: from the country road west across the meadow, then south.
pub fn trail() -> Vec<[f64; 2]> {
    let mut pts = line([126.0, 60.5], [102.5, 60.5]);
    pts.extend(line([102.5, 60.5], [102.5, 77.5]).into_iter().skip(1));
    pts
}

/// `town` (a `two_hamlets` town file) with a stretch of countryside beside the country road
/// near the first hamlet: a public countryside plot with one of each countryside spot, a trail
/// from the road across it, and a pond in the terrain's water mask.
pub fn with_countryside(town: &str) -> String {
    let mut town: Value = serde_json::from_str(town).unwrap();
    let (x, z, w, d) = MEADOW;
    town["plots"].as_array_mut().unwrap().push(json!({
        "name": "Meadow", "x": x, "z": z, "w": w, "d": d, "public": true,
        "entry": [123.5, 61.5], "kind": "countryside", "area": 2,
    }));
    for (def, ox, oz, rot) in [
        ("countryside.fishingSpot", 106, 46, 0),
        ("countryside.lookout", 110, 52, 0),
        ("countryside.picnicSpot", 114, 66, 0),
        ("countryside.berryBush", 118, 48, 0),
        ("countryside.trailhead", 121, 58, 0),
    ] {
        town["objects"]
            .as_array_mut()
            .unwrap()
            .push(json!({"def": def, "x": ox, "z": oz, "rot": rot}));
    }
    let trail = json!({"points": trail(), "width": 2, "ends": ["join", "dead"], "kind": "trail"});
    town["roads"].as_array_mut().unwrap().push(trail.clone());
    town["meta"]["roads"].as_array_mut().unwrap().push(trail);
    town["areas"].as_array_mut().unwrap().push(json!({
        "name": "Meadow", "kind": "countryside", "x": x, "z": z, "w": w, "d": d,
    }));
    // The pond: water in the terrain mask (row-major runs over the ground).
    let width = town["width"].as_i64().unwrap();
    let depth = town["depth"].as_i64().unwrap();
    let mut rle = Vec::new();
    let (mut last, mut run) = (false, 0);
    for i in 0..width * depth {
        let wet = POND.contains(&(i % width, i / width));
        if wet != last {
            rle.extend([run, last as i64]);
            (last, run) = (wet, 0);
        }
        run += 1;
    }
    rle.extend([run, last as i64]);
    town["terrain"] = json!({"water": rle});
    town.to_string()
}

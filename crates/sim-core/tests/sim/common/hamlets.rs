//! Synthetic open-world towns: hamlets built like `town_with`'s streets (`paste`), set apart in
//! open country and joined by country roads, with the open-world fields of
//! docs/design/open-world.md (`roads` with kinds, `areas`, typed `exits`, `size`). `hamlets` places a few where a travel test wants them; `grid` fills a world
//! as big as a town file may be with them, for the big-world benchmark (`tests/big_world.rs`).

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
/// joined to each other (and on) by a country road with a bend, and a road out at the far end of
/// each end hamlet.
pub fn hamlets(
    content: &str,
    seed: u64,
    columns: i64,
    places: &[Place],
    size: (i64, i64),
) -> Hamlets {
    hamlets_with_parking(content, seed, columns, places, size, None)
}

/// How wide the gap `hamlets_with_parking` leaves beside a hamlet's park for its parking bay
/// (m): the bay's 3 m and the metre and a half either side of it `macro.ts` keeps clear (it takes
/// a metre round a bay).
pub const BAY_GAP: i64 = 6;

/// `hamlets`, with public parking in hamlet `bay` (if any) as a big world has it (`macro.ts`): a
/// bay off the street, square to it beyond its verge, on free ground by the park: the plots of
/// both rows east of the park moved on `BAY_GAP` m to leave it (the hamlet that much wider). The
/// bay's middle is half the street's width, half a metre and half its 6 m length off the
/// street's line on the park's side, its `rot` a facing (0-3) as `macro.ts` gives it: 0 a bay on
/// the street's +z side, 2 on its -z side.
pub fn hamlets_with_parking(
    content: &str,
    seed: u64,
    columns: i64,
    places: &[Place],
    (width, depth): (i64, i64),
    bay: Option<usize>,
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
        let mut block: Value = serde_json::from_str(&town_with(
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
        // (Where the bay goes, in the block: just east of its park; the park's middle (z).)
        let parking = (bay == Some(k)).then(|| open_gap(&mut block));
        let (dx, dz) = (at.x, at.z);
        let bw = block["width"].as_i64().unwrap();
        let bd = block["depth"].as_i64().unwrap();
        let first = paste(&mut out, &block, dx, dz);
        let first_plot = first.plots;
        for v in &mut out["sims"].as_array_mut().unwrap()[first.sims..] {
            if k > 0 {
                v["name"] = json!(format!("H{k}{}", v["name"].as_str().unwrap()));
            }
        }
        for p in &mut out["plots"].as_array_mut().unwrap()[first.plots..] {
            p["area"] = json!(k);
        }
        for h in &mut out["households"].as_array_mut().unwrap()[first.households..] {
            if k > 0 {
                h["player"] = json!(false);
            }
        }
        plots.push(first_plot..out["plots"].as_array().unwrap().len());
        let z = street_z() + dz as f64;
        let (x0, x1) = (dx as f64, (dx + bw) as f64);
        streets.push((z, x0, x1));
        if let Some((cut, park_z)) = parking {
            // (The street is 6 m wide.)
            let side = if park_z + dz as f64 > z { 1.0 } else { -1.0 };
            let off = 6.0 / 2.0 + 0.5 + 3.0;
            out["parking"] = json!([{
                "x": (dx + cut) as f64 + BAY_GAP as f64 / 2.0, "z": z + side * off,
                "rot": if side > 0.0 { 0 } else { 2 }, "area": k,
            }]);
        }
        let pts = line([x0 + 0.5, z], [x1 - 0.5, z]);
        roads.push(
            json!({"points": pts, "width": 6, "ends": ["out", "out"], "kind": "street", "area": k}),
        );
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

/// Opens a gap of `BAY_GAP` m in `block` (a `town_with` town file of one street) just east of
/// its park: everything of the plots east of it moved on that far, the block that much wider.
/// Returns where the gap starts (x) and the park's middle (z).
fn open_gap(block: &mut Value) -> (i64, f64) {
    let park = block["plots"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["public"].as_bool() == Some(true))
        .expect("a park in the hamlet");
    let num = |k: &str| park[k].as_f64().unwrap();
    let (cut, park_z) = ((num("x") + num("w")) as i64, num("z") + num("d") / 2.0);
    let past = |v: &Value| v.as_f64().unwrap() >= cut as f64;
    let on = |v: &mut Value| {
        *v = match v.as_i64() {
            Some(n) => json!(n + BAY_GAP),
            None => json!(v.as_f64().unwrap() + BAY_GAP as f64),
        }
    };
    for w in block["walls"].as_array_mut().unwrap() {
        if past(&w[0]) && past(&w[2]) {
            on(&mut w[0]);
            on(&mut w[2]);
        }
    }
    for key in ["doors", "windows", "objects", "sims", "plots"] {
        for v in block[key].as_array_mut().unwrap() {
            if past(&v["x"]) {
                on(&mut v["x"]);
                if v["entry"].is_array() {
                    on(&mut v["entry"][0]);
                }
            }
        }
    }
    on(&mut block["width"]);
    (cut, park_z)
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

/// `two_hamlets` with public parking in the second, a bay by its park (`hamlets_with_parking`;
/// the world `BAY_GAP` m wider for it).
pub fn two_hamlets_with_parking(content: &str, seed: u64) -> Hamlets {
    hamlets_with_parking(
        content,
        seed,
        3,
        &[Place { x: 0, z: 0 }, Place { x: 162, z: 60 }],
        (256 + BAY_GAP, 128),
        Some(1),
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

/// Where `paste` put a block's plots, households and sims in the town (the first index of each).
#[derive(Debug, Clone, Copy)]
pub struct Pasted {
    pub plots: usize,
    pub households: usize,
    pub sims: usize,
}

/// Adds a `town_with` town file (`block`) to `out` with its corner at `(dx, dz)`: its walls,
/// doors, windows, objects, sims (and their places), plots (and their entries), households and
/// relationships, their indices into each other moved on past what `out` already has.
pub fn paste(out: &mut Value, block: &Value, dx: i64, dz: i64) -> Pasted {
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
    let len = |out: &Value, key: &str| out[key].as_array().unwrap().len();
    let first = Pasted {
        plots: len(out, "plots"),
        households: len(out, "households"),
        sims: len(out, "sims"),
    };
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
                    json!(v["household"].as_u64().unwrap() as usize + first.households);
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
        out["plots"].as_array_mut().unwrap().push(p);
    }
    for h in block["households"].as_array().unwrap() {
        let mut h = h.clone();
        h["plot"] = json!(h["plot"].as_u64().unwrap() as usize + first.plots);
        out["households"].as_array_mut().unwrap().push(h);
    }
    for r in block["relationships"].as_array().unwrap() {
        let mut r = r.clone();
        r["a"] = json!(r["a"].as_u64().unwrap() as usize + first.sims);
        r["b"] = json!(r["b"].as_u64().unwrap() as usize + first.sims);
        out["relationships"].as_array_mut().unwrap().push(r);
    }
    first
}

/// A world `size × size` tiles filled with hamlets on a grid, for the big-world benchmark: each a
/// street of `columns` `town_with` houses a side (the shipped house layouts, with the content's
/// storeys and basement above and below them) and a park, with at least a hamlet's depth of open
/// country between them. Residents move in up to `max_residents`; the player's household is in
/// the first hamlet. With `open` it's a `big` town: every street a road, the
/// hamlets of a row joined by country roads that run on to a way out at both edges of the world,
/// and the rows joined by a country road down the west edge. Without, a town with no open-world
/// fields (cosy, as the benchmark's first baseline was), a way out at both edges of the world on
/// every street. Returns the town file and the hamlets' rectangles `[x0, z0, x1, z1)`.
pub fn grid(
    content: &str,
    seed: u64,
    size: i64,
    columns: i64,
    max_residents: usize,
    open: bool,
) -> (String, Vec<[i32; 4]>) {
    let layout = |max_residents| Layout {
        columns,
        streets: 1,
        yards: false,
        max_residents,
    };
    // One hamlet's town file, to measure it.
    let probe: Value = serde_json::from_str(&town_with(content, seed, 2, layout(0))).unwrap();
    let (hw, hd) = (
        probe["width"].as_i64().unwrap(),
        probe["depth"].as_i64().unwrap(),
    );
    // As many hamlets as fit with at least a hamlet's depth of country between them.
    let across = ((size - hw) / (hw + hd)).max(0) + 1;
    let down = ((size - hd) / (hd * 2)).max(0) + 1;
    let gap = |n: i64, span: i64| {
        if n > 1 {
            (size - n * span) / (n - 1)
        } else {
            0
        }
    };
    let (gx, gz) = (gap(across, hw), gap(down, hd));
    let mut out = json!({
        "width": size, "depth": size, "walls": [], "doors": [], "windows": [], "objects": [],
        "plots": [], "households": [], "sims": [], "relationships": [], "exits": [],
    });
    let (mut roads, mut areas, mut exits) = (Vec::new(), Vec::new(), Vec::new());
    let mut rects = Vec::new();
    // Per row: its street's middle (z).
    let mut rows: Vec<f64> = Vec::new();
    let edge = size as f64 - 0.5;
    for k in 0..across * down {
        let (col, row) = (k % across, k / across);
        let (ox, oz) = (col * (hw + gx), row * (hd + gz));
        let left = max_residents.saturating_sub(out["sims"].as_array().unwrap().len());
        let player = if k == 0 { 2 } else { 0 };
        let town = town_with(content, seed + k as u64, player, layout(left));
        let town: Value = serde_json::from_str(&town).unwrap();
        let first = paste(&mut out, &town, ox, oz);
        let sims = out["sims"].as_array_mut().unwrap();
        for (n, s) in sims.iter_mut().enumerate().skip(first.sims) {
            s["name"] = json!(format!("R{n}"));
        }
        for p in &mut out["plots"].as_array_mut().unwrap()[first.plots..] {
            p["name"] = json!(format!("{} ({k})", p["name"].as_str().unwrap()));
            if open {
                p["area"] = json!(k);
            }
        }
        for h in &mut out["households"].as_array_mut().unwrap()[first.households..] {
            h["name"] = json!(format!("{} ({k})", h["name"].as_str().unwrap()));
            h["player"] = json!(k == 0 && h["player"].as_bool() == Some(true));
        }
        // The street's middle, where `town_with` puts its ways out.
        let z = town["exits"][0][1].as_f64().unwrap() + oz as f64;
        let (x0, x1) = (ox as f64, (ox + hw) as f64);
        if open {
            if col == 0 {
                rows.push(z);
            }
            let pts = line([x0 + 0.5, z], [x1 - 0.5, z]);
            roads.push(json!({"points": pts, "width": 6, "kind": "street", "area": k}));
            areas.push(json!({"name": format!("Hamlet {k}"), "kind": "hamlet",
                "x": ox, "z": oz, "w": hw, "d": hd}));
            // Along the row: from the world's west edge to its first hamlet, on to the next, and
            // from its last to the east edge.
            let west = if col == 0 { 0.5 } else { x0 - gx as f64 - 0.5 };
            if west < x0 {
                roads.push(json!({"points": line([west, z], [x0 + 0.5, z]), "width": 5,
                    "kind": "country"}));
            }
            if col == across - 1 && x1 - 0.5 < edge {
                roads.push(json!({"points": line([x1 - 0.5, z], [edge, z]), "width": 5,
                    "kind": "country"}));
            }
        } else {
            exits.push(json!([0.5, z]));
            exits.push(json!([edge, z]));
        }
        rects.push([ox as i32, oz as i32, (ox + hw) as i32, (oz + hd) as i32]);
    }
    if open {
        // The rows joined down the west edge, and a way out at both ends of every row.
        for w in rows.windows(2) {
            roads.push(json!({"points": line([2.5, w[0]], [2.5, w[1]]), "width": 5,
                "kind": "country"}));
        }
        for &z in &rows {
            exits.push(json!([0.5, z]));
            exits.push(json!([edge, z]));
        }
        out["size"] = json!("big");
        out["roads"] = Value::Array(roads.clone());
        out["meta"] = json!({ "roads": roads });
        out["areas"] = Value::Array(areas);
    }
    out["exits"] = Value::Array(exits);
    (out.to_string(), rects)
}

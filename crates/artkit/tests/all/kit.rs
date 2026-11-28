//! The kit against gamekit's in Blender 5.2 (tests/golden/kit_*.txt): calls the families made of each kit function,
//! their arguments as the scripts gave them (whole numbers and floats apart; a far version's inside `far_of`), and
//! what Blender held after each: every part's points, faces, triangles, sharp edges, smooth flags, materials and
//! place, and for a part with a bevel the mesh its modifier made.
//!
//! The goldens are a choice of what the capture script wrote for every family (each kind of call of each
//! function; the script, outside the repo, wraps `gamekit.kit`'s functions and builds every model). Set
//! KIT_CAPTURE to a folder of its files to check them all.

use crate::common::*;
use artkit::bevel::apply;
use artkit::far::far_of;
use artkit::forms::{Profile, Scale, by};
use artkit::json::{Json, loads};
use artkit::kit;
use artkit::materials::Mat;
use artkit::math::V3;
use artkit::mesh::{Mesh, UvWay};
use artkit::place::{Coord, Facing, GameFrame};
use artkit::scene::{self, Obj};
use std::collections::BTreeMap;

/// A number as a script wrote it: whole or not.
#[derive(Clone, Copy, Debug)]
enum N {
    I(i64),
    F(f64),
}

impl Coord for N {
    fn f(self) -> f64 {
        match self {
            N::I(i) => i as f64,
            N::F(x) => x,
        }
    }
    fn negated(self) -> f64 {
        match self {
            N::I(i) => -i as f64,
            N::F(x) => -x,
        }
    }
}

fn n(j: &Json) -> N {
    match j {
        Json::Int(i) => N::I(*i),
        Json::Num(x) => N::F(*x),
        Json::Bool(b) => N::I(*b as i64),
        other => panic!("not a number: {other:?}"),
    }
}

fn f(j: &Json) -> f64 {
    n(j).f()
}

fn int(j: &Json) -> usize {
    match j {
        Json::Int(i) => *i as usize,
        Json::Bool(b) => *b as usize,
        other => panic!("not a whole number: {other:?}"),
    }
}

fn truth(j: &Json) -> bool {
    match j {
        Json::Bool(b) => *b,
        Json::Int(i) => *i != 0,
        Json::Null => false,
        other => panic!("not a truth: {other:?}"),
    }
}

fn items(j: &Json) -> &[Json] {
    match j {
        Json::Arr(a) => a,
        other => panic!("not a list: {other:?}"),
    }
}

fn pair(j: &Json) -> (f64, f64) {
    let a = items(j);
    (f(&a[0]), f(&a[1]))
}

fn pairs(j: &Json) -> Vec<(f64, f64)> {
    items(j).iter().map(pair).collect()
}

fn triple(j: &Json) -> (N, N, N) {
    let a = items(j);
    (n(&a[0]), n(&a[1]), n(&a[2]))
}

fn material(j: &Json) -> Mat {
    let leak = |s: &str| -> &'static str { Box::leak(s.to_string().into_boxed_str()) };
    let name = j.get("mat").and_then(Json::str).expect("a material");
    let finish = j.get("finish").and_then(Json::str).unwrap_or("matte");
    Mat::new(leak(name), "#808080", leak(finish))
}

fn frame(j: &Json) -> GameFrame {
    GameFrame {
        shift: f(j.get("shift").unwrap()),
        depths: match j.get("depths") {
            Some(Json::Null) | None => None,
            Some(d) => Some(pairs(d)),
        },
    }
}

/// A sweep's scale as the script gave it: a number, a list, or a function (what it said for what it was asked).
fn scale(j: &Json) -> Scale {
    match j {
        Json::Arr(list) => Scale::List(list.iter().map(f).collect()),
        Json::Obj(_) => {
            let said = pairs(j.get("fn").unwrap());
            by(move |w| {
                let nearest = said
                    .iter()
                    .min_by(|a, b| (a.0 - w).abs().total_cmp(&(b.0 - w).abs()))
                    .unwrap();
                assert_eq!(nearest.0.to_bits(), w.to_bits(), "asked how thick at {w}");
                nearest.1
            })
        }
        other => Scale::One(f(other)),
    }
}

/// One call: its arguments by name (the positional ones by the function's own order), and what Blender made.
struct Case {
    what: String,
    func: String,
    low: bool,
    args: BTreeMap<String, Json>,
    out: Vec<f64>,
    parts: Vec<GoldenMesh>,
}

impl Case {
    fn get(&self, key: &str) -> Option<&Json> {
        self.args.get(key)
    }
    fn at(&self, key: &str) -> &Json {
        self.get(key)
            .unwrap_or_else(|| panic!("{}: no {key}", self.what))
    }
    fn name(&self) -> &str {
        self.at("name").str().unwrap()
    }
}

fn params(func: &str) -> &'static [&'static str] {
    match func {
        "tube" => &[
            "frame", "name", "points", "r", "material", "verts", "round", "scale",
        ],
        "cabriole" => &[
            "frame", "name", "x", "z", "h", "material", "sx", "sz", "r", "verts",
        ],
        "fillet" => &["points", "radius", "steps"],
        "pull" => &[
            "frame", "name", "x", "y", "face", "length", "material", "r", "out", "upright", "verts",
        ],
        "knob" => &[
            "frame", "name", "x", "y", "face", "r", "material", "segments",
        ],
        "shaker" => &[
            "frame", "name", "w", "h", "at", "t", "material", "rail", "panel", "recess", "raised",
        ],
        "moulding" => &[
            "frame", "name", "outline", "profile", "material", "at", "plane", "angle",
        ],
        "songbird" => &[
            "frame", "name", "x", "y", "z", "body", "breast", "beak", "eye", "turn", "size",
            "head", "low",
        ],
        "scroll" => &[
            "frame", "name", "start", "centre", "material", "turns", "end", "t", "w", "steps",
            "profile",
        ],
        "bench_end" => &[
            "frame", "name", "x", "seat", "front", "back", "top", "material", "arm", "t", "w",
            "foot", "steps", "corner", "scrolls",
        ],
        "taxi_word" => &[
            "frame", "name", "centre", "h", "z", "facing", "material", "depth",
        ],
        "chequer" => &[
            "frame", "name", "x0", "x1", "y", "z", "size", "facing", "material", "rows", "depth",
        ],
        other => panic!("{other}: not the kit's"),
    }
}

fn cases(text: &str) -> Vec<Case> {
    let mut out: Vec<Case> = Vec::new();
    for line in text.lines() {
        let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
        match key {
            "#" => out.push(Case {
                what: rest.to_string(),
                func: String::new(),
                low: false,
                args: BTreeMap::new(),
                out: Vec::new(),
                parts: Vec::new(),
            }),
            "call" => {
                let case = out.last_mut().unwrap();
                let call = loads(rest).unwrap_or_else(|e| panic!("{}: {e}", case.what));
                case.func = call.get("fn").and_then(Json::str).unwrap().to_string();
                case.low = truth(call.get("low").unwrap());
                let names = params(&case.func);
                for (value, name) in items(call.get("args").unwrap()).iter().zip(names) {
                    case.args.insert(name.to_string(), value.clone());
                }
                if let Some(Json::Obj(kw)) = call.get("kw") {
                    for (name, value) in kw {
                        assert!(names.contains(&name.as_str()), "{}: {name}", case.what);
                        case.args.insert(name.clone(), value.clone());
                    }
                }
            }
            "out" => {
                let words: Vec<&str> = rest.split(' ').filter(|w| !w.is_empty()).collect();
                out.last_mut().unwrap().out = f64s(&words);
            }
            "mesh" => out.last_mut().unwrap().parts.push(GoldenMesh::default()),
            "v" => {
                let x: Vec<f32> = (0..rest.len() / 8)
                    .map(|i| f32w(&rest[i * 8..i * 8 + 8]))
                    .collect();
                let part = out.last_mut().unwrap().parts.last_mut().unwrap();
                part.verts = x.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
            }
            "f" => {
                out.last_mut().unwrap().parts.last_mut().unwrap().faces = faces_of(rest);
            }
            "t" => {
                let w: Vec<u32> = rest
                    .split(' ')
                    .filter(|w| !w.is_empty())
                    .map(|i| i.parse().unwrap())
                    .collect();
                let part = out.last_mut().unwrap().parts.last_mut().unwrap();
                part.tris = w.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
            }
            _ => {
                let part = out.last_mut().unwrap().parts.last_mut().unwrap();
                part.lines.insert(key.to_string(), rest.to_string());
            }
        }
    }
    out
}

fn faces_of(text: &str) -> Vec<Vec<u32>> {
    text.split(' ')
        .filter(|w| !w.is_empty())
        .map(|f| f.split(',').map(|i| i.parse().unwrap()).collect())
        .collect()
}

/// Singles written end to end, eight hex digits each.
fn singles(hex: &str) -> Vec<f32> {
    (0..hex.len() / 8)
        .map(|i| f32w(&hex[i * 8..i * 8 + 8]))
        .collect()
}

fn points(hex: &str) -> Vec<V3> {
    singles(hex)
        .chunks(3)
        .map(|c| V3::new(c[0], c[1], c[2]))
        .collect()
}

/// Makes the call with the kernel's kit: the parts it returns, or (`fillet`) the path.
fn build(c: &Case) -> (Vec<Obj>, Vec<f64>) {
    let fr = c.get("frame").map(frame).unwrap_or_default();
    let mut path = Vec::new();
    let made = match c.func.as_str() {
        "tube" => {
            let pts: Vec<(N, N, N)> = items(c.at("points")).iter().map(triple).collect();
            let mut o = kit::Tube::default();
            if let Some(v) = c.get("verts") {
                o.verts = (*v != Json::Null).then(|| int(v));
            }
            if let Some(v) = c.get("round") {
                o.round = int(v);
            }
            if let Some(v) = c.get("scale") {
                o.scale = scale(v);
            }
            vec![kit::tube(
                &fr,
                c.name(),
                &pts,
                f(c.at("r")),
                material(c.at("material")),
                o,
            )]
        }
        "cabriole" => {
            let mut o = kit::Cabriole::default();
            if let Some(v) = c.get("r") {
                o.r = f(v);
            }
            if let Some(v) = c.get("verts") {
                o.verts = int(v);
            }
            vec![kit::cabriole(
                &fr,
                c.name(),
                n(c.at("x")),
                n(c.at("z")),
                f(c.at("h")),
                material(c.at("material")),
                f(c.at("sx")),
                f(c.at("sz")),
                o,
            )]
        }
        "fillet" => {
            let steps = c.get("steps").map_or(3, int);
            let given = items(c.at("points"));
            let radius = f(c.at("radius"));
            // (a path drawn flat: points of two)
            path = if given.iter().all(|p| items(p).len() == 2) {
                kit::fillet(&pairs(c.at("points")), radius, steps)
                    .iter()
                    .flat_map(|p| [p.0, p.1])
                    .collect()
            } else {
                let pts: Vec<(f64, f64, f64)> = given
                    .iter()
                    .map(|p| {
                        let p = triple(p);
                        (p.0.f(), p.1.f(), p.2.f())
                    })
                    .collect();
                kit::fillet(&pts, radius, steps)
                    .iter()
                    .flat_map(|p| [p.0, p.1, p.2])
                    .collect()
            };
            Vec::new()
        }
        "pull" => {
            let mut o = kit::Pull::default();
            if let Some(v) = c.get("r") {
                o.r = f(v);
            }
            if let Some(v) = c.get("out") {
                o.out = f(v);
            }
            if let Some(v) = c.get("upright") {
                o.upright = truth(v);
            }
            if let Some(v) = c.get("verts") {
                o.verts = int(v);
            }
            vec![kit::pull(
                &fr,
                c.name(),
                f(c.at("x")),
                f(c.at("y")),
                f(c.at("face")),
                f(c.at("length")),
                material(c.at("material")),
                o,
            )]
        }
        "knob" => {
            let mut o = kit::Knob::default();
            if let Some(v) = c.get("segments") {
                o.segments = int(v);
            }
            vec![kit::knob(
                &fr,
                c.name(),
                n(c.at("x")),
                n(c.at("y")),
                n(c.at("face")),
                f(c.at("r")),
                material(c.at("material")),
                o,
            )]
        }
        "shaker" => {
            let mut o = kit::Shaker::default();
            if let Some(v) = c.get("rail") {
                o.rail = f(v);
            }
            if let Some(v) = c.get("panel") {
                o.panel = (*v != Json::Null).then(|| material(v));
            }
            if let Some(v) = c.get("recess") {
                o.recess = f(v);
            }
            if let Some(v) = c.get("raised") {
                o.raised = truth(v);
            }
            let (a, b) = kit::shaker(
                &fr,
                c.name(),
                f(c.at("w")),
                f(c.at("h")),
                triple(c.at("at")),
                f(c.at("t")),
                material(c.at("material")),
                o,
            );
            vec![a, b]
        }
        "moulding" => {
            let mut o = kit::Moulding::default();
            if let Some(v) = c.get("at") {
                o.at = f(v);
            }
            if let Some(v) = c.get("plane") {
                o.plane = match v.str().unwrap() {
                    "front" => Facing::Front,
                    "top" => Facing::Top,
                    other => panic!("{other}"),
                };
            }
            if let Some(v) = c.get("angle") {
                o.angle = f(v);
            }
            vec![kit::moulding(
                &fr,
                c.name(),
                &pairs(c.at("outline")),
                &pairs(c.at("profile")),
                material(c.at("material")),
                o,
            )]
        }
        "songbird" => {
            let mut o = kit::Songbird::default();
            if let Some(v) = c.get("turn") {
                o.turn = match n(v) {
                    N::I(i) => artkit::Arg::arg(i as i32),
                    N::F(x) => artkit::Arg::arg(x),
                };
            }
            if let Some(v) = c.get("size") {
                o.size = f(v);
            }
            if let Some(v) = c.get("head") {
                o.head = f(v);
            }
            if let Some(v) = c.get("low") {
                o.low = truth(v);
            }
            let eye = c.get("eye").filter(|v| **v != Json::Null).map(material);
            kit::songbird(
                &fr,
                c.name(),
                f(c.at("x")),
                f(c.at("y")),
                f(c.at("z")),
                material(c.at("body")),
                material(c.at("breast")),
                material(c.at("beak")),
                eye,
                o,
            )
        }
        "scroll" => {
            let mut o = kit::Scroll::default();
            if let Some(v) = c.get("turns") {
                o.turns = f(v);
            }
            if let Some(v) = c.get("end") {
                o.end = f(v);
            }
            if let Some(v) = c.get("t") {
                o.t = f(v);
            }
            if let Some(v) = c.get("w") {
                o.w = f(v);
            }
            if let Some(v) = c.get("steps") {
                o.steps = int(v);
            }
            if let Some(v) = c.get("profile") {
                o.profile = Profile((*v != Json::Null).then(|| pairs(v)));
            }
            let start = triple(c.at("start"));
            vec![kit::scroll(
                &fr,
                c.name(),
                (start.0, start.1.f(), start.2.f()),
                pair(c.at("centre")),
                material(c.at("material")),
                o,
            )]
        }
        "bench_end" => {
            let mut o = kit::BenchEnd::default();
            if let Some(v) = c.get("arm") {
                o.arm = (*v != Json::Null).then(|| f(v));
            }
            if let Some(v) = c.get("t") {
                o.t = f(v);
            }
            if let Some(v) = c.get("w") {
                o.w = f(v);
            }
            if let Some(v) = c.get("foot") {
                o.foot = f(v);
            }
            if let Some(v) = c.get("steps") {
                o.steps = int(v);
            }
            if let Some(v) = c.get("corner") {
                o.corner = int(v);
            }
            if let Some(v) = c.get("scrolls") {
                o.scrolls = truth(v);
            }
            kit::bench_end(
                &fr,
                c.name(),
                n(c.at("x")),
                f(c.at("seat")),
                f(c.at("front")),
                f(c.at("back")),
                pair(c.at("top")),
                material(c.at("material")),
                o,
            )
        }
        "taxi_word" => {
            let mut o = kit::TaxiWord::default();
            if let Some(v) = c.get("depth") {
                o.depth = f(v);
            }
            kit::taxi_word(
                &fr,
                c.name(),
                pair(c.at("centre")),
                f(c.at("h")),
                f(c.at("z")),
                f(c.at("facing")) as i32,
                material(c.at("material")),
                o,
            )
        }
        "chequer" => {
            let mut o = kit::Chequer::default();
            if let Some(v) = c.get("rows") {
                o.rows = int(v);
            }
            if let Some(v) = c.get("depth") {
                o.depth = f(v);
            }
            kit::chequer(
                &fr,
                c.name(),
                f(c.at("x0")),
                f(c.at("x1")),
                f(c.at("y")),
                f(c.at("z")),
                f(c.at("size")),
                f(c.at("facing")) as i32,
                material(c.at("material")),
                o,
            )
        }
        other => panic!("{other}"),
    };
    (made, path)
}

/// What a part's comparison found, for the report.
#[derive(Default)]
struct Found {
    faults: Vec<String>,
    /// Points that aren't Blender's to the bit, and the farthest of them (m).
    inexact: usize,
    far: f32,
    /// A bevelled part's points from Blender's (m), its corner normals (degrees) and the share of its area with
    /// a corner's normal more than a tenth of a degree from Blender's.
    bevel_far: Option<f32>,
    bevel_turn: f32,
    bevel_share: f64,
}

fn apart(a: V3, b: V3) -> f32 {
    (0..3).map(|i| (a[i] - b[i]).abs()).fold(0.0, f32::max)
}

/// The most any corner's normal of a bevelled mesh is turned from Blender's (degrees), over the faces that have an
/// area; the share of the mesh's area on faces with a corner turned more than a tenth of a degree; and how many of
/// the faces aren't Blender's.
fn normals_off(mesh: &Mesh, map: &[u32], of: &[Vec<u32>], on: &[V3]) -> (f32, f64, usize) {
    let mut golden: BTreeMap<Vec<u32>, Vec<(u32, V3)>> = BTreeMap::new();
    let mut at = 0;
    for face in of {
        let mut key = face.clone();
        key.sort_unstable();
        key.dedup();
        golden.insert(
            key,
            face.iter()
                .enumerate()
                .map(|(i, &v)| (v, on[at + i]))
                .collect(),
        );
        at += face.len();
    }
    let mine = mesh.normals.as_ref().expect("a bevel's normals");
    let (mut worst, mut missing) = (0.0_f32, 0);
    let (mut all, mut turned) = (0.0_f64, 0.0_f64);
    for f in 0..mesh.faces.len() {
        let c: Vec<u32> = mesh.corners(f).iter().map(|&v| map[v as usize]).collect();
        let p: Vec<V3> = mesh
            .corners(f)
            .iter()
            .map(|&v| mesh.verts[v as usize])
            .collect();
        let area: f64 = (1..p.len() - 1)
            .map(|i| (p[i] - p[0]).cross(p[i + 1] - p[0]).length() / 2.0)
            .sum();
        if area < 1e-10 {
            continue;
        }
        let mut key = c.clone();
        key.sort_unstable();
        key.dedup();
        let Some(g) = golden.get(&key) else {
            missing += 1;
            continue;
        };
        let start = mesh.faces[f].start as usize;
        let mut off = 0.0_f32;
        for (i, v) in c.iter().enumerate() {
            let want = g.iter().find(|(w, _)| w == v).unwrap().1;
            let cos = mine[start + i].dot(want).clamp(-1.0, 1.0);
            off = off.max(cos.acos().to_degrees() as f32);
        }
        all += area;
        if off > 0.1 {
            turned += area;
        }
        worst = worst.max(off);
    }
    (worst, turned / all, missing)
}

/// One part against Blender's. `slab`: an extruded outline, whose far face Blender numbers by its own triangles
/// (the same points, in an order of its own).
fn compare(obj: Obj, g: &GoldenMesh, slab: bool) -> Found {
    let mut found = Found::default();
    let (name, mesh, loc, rot, scale, way, quat, pending) = obj.read(|o| {
        (
            o.name.clone(),
            o.mesh.clone(),
            o.loc,
            o.rot,
            o.scale,
            o.uv,
            o.quat,
            o.bevel.clone(),
        )
    });
    let mut fail = |what: String| found.faults.push(what);
    if name != g.lines["name"] {
        fail(format!("named {name} for {}", g.lines["name"]));
    }
    // where it is: to the bit, a zero's sign too
    let placed: Vec<&str> = g.lines["loc"].split(' ').collect();
    let got = [loc.array(), rot.array(), scale.array()].concat();
    if bits(&got) != bits(&singles(placed[0])) {
        fail(format!("placed {got:?} for {:?}", singles(placed[0])));
    }
    let way_want = match placed[1] {
        "round" => UvWay::Round,
        "done" => UvWay::Done,
        _ => UvWay::Box,
    };
    if way != way_want || quat.is_some() || placed[2] != "XYZ" {
        fail(format!("uv way {way:?}, turned by {quat:?}"));
    }
    // its points: Blender's to the bit and in its order
    let in_order = mesh.verts.len() == g.verts.len()
        && mesh
            .verts
            .iter()
            .zip(&g.verts)
            .all(|(a, b)| a.array() == *b);
    let map: Vec<u32> = if in_order {
        (0..g.verts.len() as u32).collect()
    } else if mesh.verts.len() != g.verts.len() {
        fail(format!("{} points for {}", mesh.verts.len(), g.verts.len()));
        return found;
    } else if slab {
        match pair_points(&mesh.verts, &g.verts, 1e-6) {
            Some((map, inexact, far)) => {
                (found.inexact, found.far) = (inexact, far);
                map
            }
            None => {
                fail("not Blender's points".into());
                return found;
            }
        }
    } else {
        let (inexact, _) = points_off(&mesh.verts, &g.verts);
        found.inexact = inexact;
        found.far = mesh
            .verts
            .iter()
            .zip(&g.verts)
            .map(|(a, b)| apart(*a, V3::new(b[0], b[1], b[2])))
            .fold(0.0, f32::max);
        (0..g.verts.len() as u32).collect()
    };
    if found.inexact > 0 {
        fail(format!(
            "{} points off by up to {:e}",
            found.inexact, found.far
        ));
    }
    let faces: Vec<Vec<u32>> = (0..mesh.faces.len())
        .map(|f| mesh.corners(f).to_vec())
        .collect();
    // (a slab's triangles each start at a corner of Blender's fill's own)
    let same_faces = if slab {
        as_set(&renamed(&faces, &map)) == as_set(&g.faces)
    } else {
        sorted(&renamed(&faces, &map)) == sorted(&g.faces)
    };
    if !same_faces {
        fail(format!(
            "faces differ ({} for {})",
            faces.len(),
            g.faces.len()
        ));
    }
    if as_set(&renamed(&mesh.triangle_verts(), &map)) != as_set(&g.tris) {
        fail("triangles differ".into());
    }
    let mut sharp: Vec<(u32, u32)> = mesh
        .sharp
        .iter()
        .map(|&(a, b)| artkit::mesh::edge(map[a as usize], map[b as usize]))
        .collect();
    sharp.sort_unstable();
    let mut want: Vec<(u32, u32)> = g.lines["e"]
        .split(' ')
        .filter(|w| !w.is_empty())
        .map(|e| {
            let (a, b) = e.split_once('-').unwrap();
            artkit::mesh::edge(a.parse().unwrap(), b.parse().unwrap())
        })
        .collect();
    want.sort_unstable();
    if sharp != want {
        fail(format!(
            "sharp edges differ ({} for {})",
            sharp.len(),
            want.len()
        ));
    }
    // smooth flags and materials, a face each: all alike on a kit part, so their order doesn't matter
    let mut smooth: Vec<char> = mesh
        .faces
        .iter()
        .map(|f| if f.smooth { '1' } else { '0' })
        .collect();
    let mut want: Vec<char> = g.lines["s"].chars().collect();
    smooth.sort_unstable();
    want.sort_unstable();
    if smooth != want {
        fail("smooth flags differ".into());
    }
    let mut slots: Vec<char> = mesh
        .faces
        .iter()
        .map(|f| char::from_digit(f.mat as u32, 10).unwrap())
        .collect();
    let mut want: Vec<char> = g.lines["m"].chars().collect();
    slots.sort_unstable();
    want.sort_unstable();
    let mats: Vec<&str> = mesh.mats.iter().map(|m| m.name).collect();
    if slots != want || mats.join(" ") != g.lines["mats"] {
        fail(format!("materials {mats:?}"));
    }
    // per corner, where the golden has them and the faces came out in Blender's order: UVs and normals
    if renamed(&faces, &map) == g.faces {
        if let Some(want) = g.lines.get("uv") {
            let want = singles(want);
            match &mesh.uvs {
                Some(uvs) => {
                    let off = uvs
                        .iter()
                        .zip(want.chunks(2))
                        .map(|(u, w)| (u[0] - w[0]).abs().max((u[1] - w[1]).abs()))
                        .fold(0.0, f32::max);
                    if off > 2e-6 {
                        fail(format!("uvs off by {off}"));
                    }
                }
                // (Blender's own primitives come with an unwrap nothing reads: `uv::ensure` writes over it)
                None if way == UvWay::Done => fail("no uvs".into()),
                None => {}
            }
        }
        if let Some(want) = g.lines.get("cn") {
            let cn = artkit::normals::corner_normals(&mesh);
            let off = cn
                .iter()
                .zip(points(want))
                .map(|(a, b)| apart(*a, b))
                .fold(0.0, f32::max);
            if off > 2e-4 {
                fail(format!("corner normals off by {off}"));
            }
        }
    }
    // its bevel: asked for as Blender's modifier was, and the mesh it makes
    match (g.lines.get("bevel"), pending) {
        (None, None) => {}
        (Some(asked), Some(p)) => {
            let w: Vec<&str> = asked.split(' ').collect();
            let (width, segments, angle) = (f64w(w[0]), w[1].parse::<usize>().unwrap(), f64w(w[2]));
            // (the modifier keeps its width and its angle in single precision)
            if p.width as f32 != width as f32
                || p.segments != segments
                || (p.angle - angle).abs() > 1e-4
            {
                fail(format!(
                    "bevel {} x {} over {}",
                    p.width, p.segments, p.angle
                ));
            }
            let got = apply(obj).read(|o| o.mesh.clone());
            let want = points(&g.lines["ov"]);
            if got.normals.is_none() {
                fail("bevelled: no normals of its own".into());
                return found;
            }
            if got.verts.len() != want.len() {
                fail(format!(
                    "bevelled: {} points for {}",
                    got.verts.len(),
                    want.len()
                ));
                return found;
            }
            let (map, far) = if slab {
                let w: Vec<[f32; 3]> = want.iter().map(|p| p.array()).collect();
                match pair_points(&got.verts, &w, 1e-3) {
                    Some((map, _, far)) => (map, far),
                    None => {
                        fail("bevelled: not Blender's points".into());
                        return found;
                    }
                }
            } else {
                let far = got
                    .verts
                    .iter()
                    .zip(&want)
                    .map(|(a, b)| apart(*a, *b))
                    .fold(0.0, f32::max);
                ((0..want.len() as u32).collect(), far)
            };
            // (where a clamped bevel leaves two points in one place, either is the other)
            let same = |a: V3, b: V3| bits(&a.array()) == bits(&b.array());
            let canon: Vec<u32> = (0..want.len())
                .map(|j| (0..=j).find(|&i| same(want[i], want[j])).unwrap() as u32)
                .collect();
            let map: Vec<u32> = map.iter().map(|&j| canon[j as usize]).collect();
            let of = renamed(&faces_of(&g.lines["of"]), &canon);
            let (turn, share, missing) = normals_off(&got, &map, &of, &points(&g.lines["on"]));
            (found.bevel_far, found.bevel_turn, found.bevel_share) = (Some(far), turn, share);
            if missing > 0 {
                found
                    .faults
                    .push(format!("bevelled: {missing} faces not Blender's"));
            }
            if !got.faces.iter().all(|f| f.smooth) || g.lines["os"].contains('0') {
                found.faults.push("bevelled: not shaded smooth".into());
            }
        }
        (asked, p) => fail(format!("bevel {asked:?} in Blender, {} here", p.is_some())),
    }
    found
}

/// Checks every call in `text`; `totals`: a function's calls, parts, bevelled parts, and bevelled frames with a
/// corner's normal off Blender's; `worst`: the largest share of a frame's area with such corners.
fn check(
    text: &str,
    report: &mut BTreeMap<String, Vec<String>>,
    totals: &mut BTreeMap<String, [usize; 4]>,
    worst: &mut f64,
) {
    for c in cases(text) {
        scene::reset();
        let (made, path) = if c.low {
            far_of(|| build(&c))
        } else {
            build(&c)
        };
        let total = totals.entry(c.func.clone()).or_default();
        total[0] += 1;
        let mut said = Vec::new();
        if c.func == "fillet" {
            let same = path.len() == c.out.len()
                && path
                    .iter()
                    .zip(&c.out)
                    .all(|(a, b)| a.to_bits() == b.to_bits() || (*a == 0.0 && *b == 0.0));
            if !same {
                said.push(format!(
                    "{} numbers for {}, or not Blender's",
                    path.len(),
                    c.out.len()
                ));
            }
        } else if made.len() != c.parts.len() {
            said.push(format!("{} parts for {}", made.len(), c.parts.len()));
        } else {
            for (i, (&obj, g)) in made.iter().zip(&c.parts).enumerate() {
                total[1] += 1;
                // a shaker's frame is the kit's one slab
                let found = compare(obj, g, c.func == "shaker" && i == 0);
                for fault in &found.faults {
                    said.push(format!("{}: {fault}", g.lines["name"]));
                }
                if let Some(far) = found.bevel_far {
                    total[2] += 1;
                    // a bevelled box (a panel, a bench's foot): Blender's points to a tenth of a thousandth of
                    // a millimetre, in its order, its normals within a tenth of a degree. A bevelled slab (the
                    // frame) is the kernel's bevel on the kernel's fill, which tests/bevel.rs holds to Blender's:
                    // its points, and its normals but where a bevel wider than half a corner's chamfer leaves
                    // two points in one place (the corners of its opening), on a small share of its area
                    let frame = c.func == "shaker" && i == 0;
                    let fine = if frame {
                        if found.bevel_turn > 0.1 {
                            total[3] += 1;
                            *worst = worst.max(found.bevel_share);
                        }
                        far <= SLAB_REACH && found.bevel_share <= SLAB_SHARE
                    } else {
                        far <= 1e-7 && found.bevel_turn <= 0.1
                    };
                    if !fine {
                        said.push(format!(
                            "{}: bevelled, points within {far:e}, normals within {:.3} degrees ({:.2} % of its area)",
                            g.lines["name"],
                            found.bevel_turn,
                            100.0 * found.bevel_share
                        ));
                    }
                }
            }
        }
        if !said.is_empty() {
            report.entry(c.func.clone()).or_default().push(format!(
                "{}: {}",
                c.what,
                said.join("; ")
            ));
        }
    }
}

/// The goldens, a file a kind of part (`kit`'s modules; the shakers' frames in a file of their own).
const GOLDENS: [&str; 7] = [
    "kit_creatures.txt",
    "kit_hardware.txt",
    "kit_joinery.txt",
    "kit_joinery_shaker.txt",
    "kit_metalwork.txt",
    "kit_signs.txt",
    "kit_turned.txt",
];

/// How far a bevelled frame's points may be from Blender's (m), and the share of its area whose corners' normals
/// may be more than a tenth of a degree from Blender's (tests/bevel.rs: the slabs the kernel builds itself).
const SLAB_REACH: f32 = 2.5e-7;
const SLAB_SHARE: f64 = 0.014;

#[test]
fn the_kit_is_gamekits_in_blender() {
    let mut report = BTreeMap::new();
    let mut totals = BTreeMap::new();
    let mut worst = 0.0;
    let all = std::env::var("KIT_CAPTURE").ok();
    let files: Vec<std::path::PathBuf> = match &all {
        Some(dir) => {
            let mut files: Vec<_> = std::fs::read_dir(dir)
                .unwrap()
                .map(|e| e.unwrap().path())
                .filter(|p| p.extension().is_some_and(|e| e == "txt"))
                .collect();
            files.sort();
            files
        }
        None => GOLDENS
            .iter()
            .map(|name| format!("{}/tests/golden/{name}", env!("CARGO_MANIFEST_DIR")).into())
            .collect(),
    };
    for path in &files {
        check(
            &std::fs::read_to_string(path).unwrap(),
            &mut report,
            &mut totals,
            &mut worst,
        );
    }
    for (func, [calls, parts, bevels, turned]) in &totals {
        let bad = report.get(func).map_or(0, Vec::len);
        println!("{func}: {calls} calls, {parts} parts ({bevels} bevelled), {bad} not Blender's");
        if *turned > 0 {
            println!(
                "  {turned} frames with normals off Blender's at their opening's corners, on at most {:.2} % of a frame's area",
                100.0 * worst
            );
        }
    }
    let faults: Vec<String> = report.values().flatten().cloned().collect();
    assert!(faults.is_empty(), "\n{}", faults.join("\n"));
    if all.is_none() {
        assert_eq!(totals.len(), 12, "every function of the kit");
        assert_eq!(totals.values().map(|t| t[0]).sum::<usize>(), 176);
    }
}

/// One of the kit's own examples (the goldens' kit/full cases) against what a call written as the scripts wrote
/// it made.
fn example(cases: &[Case], name: &str, made: &[Obj]) {
    let c = cases
        .iter()
        .find(|c| c.what.contains(" kit/full/") && c.get("name").and_then(Json::str) == Some(name))
        .unwrap_or_else(|| panic!("no example {name}"));
    assert_eq!(made.len(), c.parts.len(), "{name}");
    for (i, (&obj, g)) in made.iter().zip(&c.parts).enumerate() {
        let found = compare(obj, g, c.func == "shaker" && i == 0);
        assert!(found.faults.is_empty(), "{name}: {:?}", found.faults);
    }
}

artkit::palette! { M:
    chrome = ("#D0D3D6", chrome), walnut = ("#6B4A2F", wood), brass = ("#C9A44C", metal),
    white = ("#F3F3F1", satin), brown = ("#7A5A3C", matte), red = ("#C0492F", matte),
    beak = ("#D9A441", satin), eye = ("#1E1E1E", gloss), gilt = ("#C9A44C", metal), iron = ("#2B2C2E", iron)
}

#[test]
fn the_macros_call_the_kit_as_the_scripts_did() {
    const F: GameFrame = GameFrame::new(0.0);
    let d = GameFrame::with_depths(0.094, &[(-0.4, -0.3), (0.4, 0.2)]);
    let all: Vec<Case> = GOLDENS
        .iter()
        .flat_map(|name| cases(&golden(name)))
        .collect();
    let one = |name: &str, obj: Obj| example(&all, name, &[obj]);
    scene::reset();
    // (a list of points with whole and other numbers down one column: each as the script wrote it)
    let bend = [(N::I(0), 0.2, 0), (N::I(0), 0.8, 0), (N::F(0.3), 0.9, 0)];
    one("rail", kit::tube!(F, "rail", &bend, 0.012, M.chrome));
    one("thick", kit::tube!(F, "thick", &bend, 0.04, M.chrome));
    one(
        "taper",
        kit::tube!(
            F,
            "taper",
            &bend,
            0.02,
            M.chrome,
            scale = by(|w| 1.0 - 0.5 * w),
            round = 2
        ),
    );
    one(
        "redrawn",
        kit::tube!(
            d,
            "redrawn",
            &[(0.1, 0.2, -0.5), (0.1, 0.8, 0.0), (0.3, 0.9, 0.5)],
            0.02,
            M.chrome
        ),
    );
    one(
        "leg_fl",
        kit::cabriole!(F, "leg_fl", 0.31, 0.3, 0.18, M.walnut, 0.02, 0.02),
    );
    one(
        "leg_mid",
        kit::cabriole!(F, "leg_mid", 0, 0, 0.3, M.walnut, -0.02, 0.02, verts = 8),
    );
    one("pull", kit::pull!(F, "pull", 0, 0.45, 0.02, 0.12, M.brass));
    one(
        "pull_redrawn",
        kit::pull!(
            d,
            "pull_redrawn",
            0,
            0.45,
            0.02,
            0.12,
            M.brass,
            upright = true
        ),
    );
    one("knob", kit::knob!(F, "knob", 0, 0.45, 0.02, 0.012, M.brass));
    one(
        "knob_float",
        kit::knob!(F, "knob_float", 0.0, 0.45, 0.0, 0.012, M.brass),
    );
    let (frame, panel) = kit::shaker!(F, "door", 0.6, 1.8, (0, 1, 0.15), 0.02, M.white);
    example(&all, "door", &[frame, panel]);
    let (frame, panel) = kit::shaker!(
        F,
        "door_raised",
        0.6,
        1.8,
        (0.0, 1, 0.15),
        0.02,
        M.white,
        raised = true
    );
    example(&all, "door_raised", &[frame, panel]);
    let (frame, panel) = kit::shaker!(
        d,
        "door_redrawn",
        0.6,
        1.8,
        (0, 1, 0.398),
        0.02,
        M.white,
        panel = M.walnut
    );
    example(&all, "door_redrawn", &[frame, panel]);
    example(
        &all,
        "robin",
        &kit::songbird!(
            F,
            "robin",
            0.3,
            1.47,
            -0.1,
            M.brown,
            M.red,
            M.beak,
            M.eye,
            turn = -20
        ),
    );
    example(
        &all,
        "far_robin",
        &kit::songbird!(
            F,
            "far_robin",
            0.3,
            1.47,
            -0.1,
            M.brown,
            M.red,
            M.beak,
            M.eye,
            low = true
        ),
    );
    example(
        &all,
        "blind",
        &kit::songbird!(F, "blind", 0, 1, 0, M.brown, M.red, M.beak),
    );
    let square = [(0.25, 1.0), (0.25, 1.7), (-0.25, 1.7), (-0.25, 1.0)];
    let section = [(0.0, 0.0), (0.0, 0.03), (0.03, 0.03), (0.03, 0.0)];
    one(
        "frame",
        kit::moulding!(F, "frame", &square, &section, M.gilt, at = -0.43),
    );
    let back: Vec<(f64, f64)> = square.iter().rev().copied().collect();
    one("plain", kit::moulding!(F, "plain", &back, &section, M.gilt));
    one(
        "curl",
        kit::scroll!(F, "curl", (0, 0.66, 0.2), (0.62, 0.2), M.iron, turns = -1.1),
    );
    one(
        "curl_plain",
        kit::scroll!(F, "curl_plain", (0.1, 0.66, 0.2), (0.62, 0.2), M.iron),
    );
    example(
        &all,
        "end",
        &kit::bench_end!(
            F,
            "end",
            0.8,
            0.415,
            0.19,
            -0.21,
            (0.885, -0.27),
            M.iron,
            arm = 0.68
        ),
    );
    example(
        &all,
        "end_plain",
        &kit::bench_end!(
            F,
            "end_plain",
            0,
            0.415,
            0.19,
            -0.21,
            (0.885, -0.27),
            M.iron
        ),
    );
    example(
        &all,
        "word",
        &kit::taxi_word!(F, "word", (0, 2.4), 0.1, 0.04, 1, M.iron, depth = 0.006),
    );
    example(
        &all,
        "word_back",
        &kit::taxi_word!(F, "word_back", (0.3, 2.4), 0.1, -0.04, -1, M.iron),
    );
    example(
        &all,
        "check",
        &kit::chequer!(
            F,
            "check",
            -0.2,
            0.2,
            2.3,
            0.04,
            0.03,
            -1,
            M.iron,
            rows = 3,
            depth = 0.004
        ),
    );
    example(
        &all,
        "check_plain",
        &kit::chequer!(F, "check_plain", -0.2, 0.2, 2.3, 0.04, 0.03, 1, M.iron),
    );
    // a path bent as the scripts asked: three points round a corner unless they said how many
    let path = kit::fillet!(&[(0, 0.5, 0)], 0.05);
    assert_eq!(path, [(0.0, 0.5, 0.0), (0.0, 0.5, 0.0)]);
    let corner = [(0.0, 0.0, 0.0), (0.0, 0.5, 0.0), (0.3, 0.5, 0.0)];
    assert_eq!(kit::fillet!(&corner, 0.05), kit::fillet(&corner, 0.05, 3));
    assert_eq!(
        kit::fillet!(&corner, 0.05, 2),
        kit::fillet(&corner, 0.05, 2)
    );
    assert_eq!(kit::fillet!(&corner, 0.05, steps = 2).len(), 5);
    let flat = [(0.0, 0.0), (0.5, 0.0), (0.5, 0.2)];
    assert_eq!(kit::fillet!(&flat, 0.05, steps = 1).len(), 4);
    // what is given by place is worked out before the options, as Python does: a script's random draws keep
    // their order
    let mut rng = artkit::rand::Random::new(7);
    let mut again = artkit::rand::Random::new(7);
    let (first, second) = (again.random(), again.random());
    let knob = kit::knob!(
        F,
        "drawn",
        0,
        0.45,
        0.02,
        0.01 + 0.01 * rng.random(),
        M.brass,
        segments = 6 + (4.0 * rng.random()) as usize
    );
    let plain = kit::knob(
        &F,
        "plain",
        0,
        0.45,
        0.02,
        0.01 + 0.01 * first,
        M.brass,
        kit::Knob {
            segments: 6 + (4.0 * second) as usize,
        },
    );
    assert_eq!(
        knob.read(|o| o.mesh.verts.clone()),
        plain.read(|o| o.mesh.verts.clone())
    );
}

/// The slabs three of the families' models are cut from (tests/golden/kit_slab_walls.txt: the fridge's sides, kick
/// plate and crisper fronts, the kids' chair's seat and back, the square table's aprons; facing the front, the side
/// and lying flat, with and without holes), as `forms.extrude` left them in Blender: a shaker's frame is such a
/// slab. Each wall starts at Blender's corner, so it is cut into Blender's two triangles.
#[test]
fn a_slabs_walls_start_where_blenders_do() {
    use artkit::forms::{Extrude, Holes, Plane, extrude};
    let text = golden("kit_slab_walls.txt");
    let mut blocks: Vec<BTreeMap<&str, &str>> = Vec::new();
    for line in text.lines() {
        let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
        if key == "#" {
            blocks.push(BTreeMap::new());
        }
        blocks.last_mut().unwrap().insert(key, rest);
    }
    assert_eq!(blocks.len(), 11);
    let mut walls = 0;
    for b in &blocks {
        let call = loads(b["call"]).unwrap();
        let args = items(call.get("args").unwrap());
        let kw = call.get("kw").unwrap();
        let mut o = Extrude::default();
        if let Some(holes) = kw.get("holes") {
            o.holes = Holes(items(holes).iter().map(pairs).collect());
        }
        if let Some(plane) = kw.get("plane") {
            o.plane = match plane.str().unwrap() {
                "XZ" => Plane::XZ,
                "XY" => Plane::XY,
                _ => Plane::YZ,
            };
        }
        scene::reset();
        let name = args[0].str().unwrap();
        let obj = extrude(name, &pairs(&args[1]), f(&args[2]), o);
        let mesh = obj.read(|o| o.mesh.clone());
        let want: Vec<[f32; 3]> = points(b["v"]).iter().map(|p| p.array()).collect();
        // (the far face's points are Blender's in an order of its own)
        let (map, inexact, _) = pair_points(&mesh.verts, &want, 0.0).expect(name);
        assert_eq!(inexact, 0, "{name}");
        let faces: Vec<Vec<u32>> = (0..mesh.faces.len())
            .map(|f| mesh.corners(f).to_vec())
            .collect();
        let (mine, theirs) = (renamed(&faces, &map), faces_of(b["f"]));
        let quads =
            |faces: &[Vec<u32>]| sorted(&faces.iter().filter(|f| f.len() == 4).collect::<Vec<_>>());
        assert_eq!(
            quads(&mine),
            quads(&theirs),
            "{name}: its walls, corner for corner"
        );
        assert_eq!(as_set(&mine), as_set(&theirs), "{name}: its faces");
        let tris: Vec<u32> = b["t"].split(' ').map(|i| i.parse().unwrap()).collect();
        let tris: Vec<[u32; 3]> = tris.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
        assert_eq!(
            as_set(&renamed(&mesh.triangle_verts(), &map)),
            as_set(&tris),
            "{name}: its triangles"
        );
        walls += mine.iter().filter(|f| f.len() == 4).count();
    }
    assert_eq!(walls, 208);
}

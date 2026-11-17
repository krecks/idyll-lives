//! The clips against Blender's (tests/golden/clips.txt): every clip call the family scripts made, and made-up ones for
//! what none makes (a wobble, every ease), with the rest of the parts they key, the F-curves Blender made (each key,
//! its handles and interpolation) and Blender's evaluation of each at every frame and half frame.
mod common;

use artkit::math::{Axis, P3, V3};
use artkit::mesh::Mesh;
use artkit::motion::{self, Channel, Ease, Key, Path};
use artkit::scene::{self, Obj};
use common::*;
use std::collections::BTreeMap;

/// A value as the golden writes one.
#[derive(Clone, Debug)]
enum Val {
    None,
    Bool(bool),
    Int(i64),
    Num(f64),
    Str(String),
    Obj(String),
    List(Vec<Val>),
}

impl Val {
    fn num(&self) -> f64 {
        match self {
            Val::Int(i) => *i as f64,
            Val::Num(x) => *x,
            Val::Bool(b) => *b as i64 as f64,
            v => panic!("not a number: {v:?}"),
        }
    }
    fn list(&self) -> &[Val] {
        match self {
            Val::List(l) => l,
            v => panic!("not a list: {v:?}"),
        }
    }
    fn p3(&self) -> P3 {
        let l = self.list();
        P3 {
            x: l[0].num(),
            y: l[1].num(),
            z: l[2].num(),
        }
    }
    fn str(&self) -> &str {
        match self {
            Val::Str(s) | Val::Obj(s) => s,
            v => panic!("not a string: {v:?}"),
        }
    }
}

fn parse(s: &str) -> (Val, &str) {
    if let Some(rest) = s.strip_prefix('[') {
        let mut items = Vec::new();
        let mut rest = rest;
        if let Some(r) = rest.strip_prefix(']') {
            return (Val::List(items), r);
        }
        loop {
            let (v, r) = parse(rest);
            items.push(v);
            match r.as_bytes()[0] {
                b',' => rest = &r[1..],
                b']' => return (Val::List(items), &r[1..]),
                _ => panic!("{r}"),
            }
        }
    }
    let end = s.find([',', ']']).unwrap_or(s.len());
    let (word, rest) = s.split_at(end);
    let v = match word {
        "N" => Val::None,
        "T" => Val::Bool(true),
        "F" => Val::Bool(false),
        _ => match word.split_once(':') {
            Some(("i", x)) => Val::Int(x.parse().unwrap()),
            Some(("d", x)) => Val::Num(f64w(x)),
            Some(("s", x)) => Val::Str(x.to_string()),
            Some(("o", x)) => Val::Obj(x.to_string()),
            _ => panic!("{word}"),
        },
    };
    (v, rest)
}

fn ease_of(interpolation: &str, easing: &str) -> Ease {
    match (interpolation, easing) {
        ("BEZIER", _) => Ease::Smooth,
        ("LINEAR", _) => Ease::Linear,
        ("QUAD", "EASE_OUT") => Ease::Out,
        ("QUAD", "EASE_IN") => Ease::In,
        ("QUAD", "EASE_IN_OUT") => Ease::InOut,
        ("EXPO", "EASE_OUT") => Ease::ExpoOut,
        ("BACK", "EASE_OUT") => Ease::BackOut,
        ("BOUNCE", "EASE_OUT") => Ease::BounceOut,
        other => panic!("an ease gamekit hasn't: {other:?}"),
    }
}

fn ease_named(name: &str) -> Ease {
    match name {
        "smooth" => Ease::Smooth,
        "linear" => Ease::Linear,
        "out" => Ease::Out,
        "in" => Ease::In,
        "in_out" => Ease::InOut,
        "expo_out" => Ease::ExpoOut,
        "back_out" => Ease::BackOut,
        "bounce_out" => Ease::BounceOut,
        other => panic!("{other}"),
    }
}

fn path_of(p: &str) -> Path {
    match p {
        "location" => Path::Location,
        "rotation_euler" => Path::Rotation,
        "scale" => Path::Scale,
        other => panic!("{other}"),
    }
}

fn axis_of(v: Option<&Val>, default: Axis) -> Axis {
    match v.map(Val::str) {
        None => default,
        Some("X") => Axis::X,
        Some("Y") => Axis::Y,
        Some(_) => Axis::Z,
    }
}

/// A key of the golden: frame, value, ease, back, left handle, right handle.
type KeyLine = (f32, f32, Ease, f32, [f32; 2], [f32; 2]);

/// One F-curve of the golden: its part, path and index, its keys (frame, value, ease, back, left handle, right
/// handle) and Blender's values at frame 1, 1.5, 2, ...
struct Curve {
    obj: String,
    path: Path,
    index: usize,
    keys: Vec<KeyLine>,
    values: Vec<f32>,
}

struct Call {
    n: String,
    at: String,
    func: String,
    args: Vec<(String, Val)>,
    rest: Vec<String>,
    curves: Vec<Curve>,
    peaks: BTreeMap<String, String>,
}

fn calls() -> Vec<Call> {
    let mut out: Vec<Call> = Vec::new();
    for line in golden("clips.txt").lines() {
        let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
        match key {
            "call" => {
                let w: Vec<&str> = rest.split(' ').collect();
                out.push(Call {
                    n: w[0].to_string(),
                    at: w[1].to_string(),
                    func: w[3].to_string(),
                    args: w[4..]
                        .iter()
                        .map(|a| {
                            let (k, v) = a.split_once('=').unwrap();
                            (k.to_string(), parse(v).0)
                        })
                        .collect(),
                    rest: Vec::new(),
                    curves: Vec::new(),
                    peaks: BTreeMap::new(),
                });
            }
            "obj" => out.last_mut().unwrap().rest.push(rest.to_string()),
            "peak" => {
                let w: Vec<&str> = rest.split(' ').collect();
                out.last_mut()
                    .unwrap()
                    .peaks
                    .insert(w[0].to_string(), w[1].to_string());
            }
            "fc" => {
                let w: Vec<&str> = rest.split(' ').collect();
                let keys = w[3]
                    .split(',')
                    .map(|k| {
                        let p: Vec<&str> = k.split(':').collect();
                        (
                            f32w(p[0]),
                            f32w(p[1]),
                            ease_of(p[2], p[3]),
                            f32w(p[4]),
                            [f32w(p[5]), f32w(p[6])],
                            [f32w(p[7]), f32w(p[8])],
                        )
                    })
                    .collect();
                out.last_mut().unwrap().curves.push(Curve {
                    obj: w[0].to_string(),
                    path: path_of(w[1]),
                    index: w[2].parse().unwrap(),
                    keys,
                    values: Vec::new(),
                });
            }
            "ev" => {
                let w: Vec<&str> = rest.split(' ').collect();
                out.last_mut().unwrap().curves.last_mut().unwrap().values = f32s(&w);
            }
            _ => panic!("{line}"),
        }
    }
    out
}

/// The overshoot a curve's back eases were made with (the default, or what a call gave).
fn overshoot_of(back: f32) -> f64 {
    for o in [motion::OVERSHOOT, 0.08] {
        if motion::back_amount(o) as f32 == back {
            return o;
        }
    }
    motion::OVERSHOOT
}

fn channel(c: &Curve) -> Channel {
    let back = c
        .keys
        .iter()
        .find(|k| k.2 == Ease::BackOut)
        .map_or(1.70158, |k| k.3);
    Channel {
        path: c.path,
        index: c.index,
        keys: c
            .keys
            .iter()
            .map(|k| Key {
                frame: k.0 as i32,
                value: k.1,
                ease: k.2,
            })
            .collect(),
        back: overshoot_of(back),
    }
}

/// The largest difference a curve's values may have from Blender's, in its own units (metres, radians, scale).
fn worst(got: &[f64], want: &[f32]) -> (f64, usize) {
    let mut far = (0.0_f64, 0);
    for (k, (g, w)) in got.iter().zip(want).enumerate() {
        let d = (g - *w as f64).abs();
        if d > far.0 {
            far = (d, k);
        }
    }
    far
}

#[test]
fn fcurves_evaluate_as_blenders() {
    // per clip kind and ease: (curves, to the bit, the largest difference in m / degrees / scale)
    let mut by: BTreeMap<String, (usize, usize, f64)> = BTreeMap::new();
    let mut handles_off = Vec::new();
    for call in calls() {
        for c in &call.curves {
            let ch = channel(c);
            // the handles
            for (k, (h, want)) in ch.handles().iter().zip(&c.keys).enumerate() {
                if h[0] != want.4 || h[2] != want.5 {
                    handles_off.push(format!(
                        "call {} {} {} {:?}[{}] key {k}: {:?} {:?}, Blender's {:?} {:?}",
                        call.n, call.at, call.func, c.path, c.index, h[0], h[2], want.4, want.5
                    ));
                }
            }
            let got: Vec<f64> = (0..c.values.len())
                .map(|k| ch.at(1.0 + k as f64 * 0.5))
                .collect();
            let exact = got
                .iter()
                .zip(&c.values)
                .all(|(g, w)| (*g as f32).to_bits() == w.to_bits());
            let (d, _) = worst(&got, &c.values);
            let d = match c.path {
                Path::Location => d * 1000.0,
                Path::Rotation => d.to_degrees(),
                Path::Scale => d,
            };
            let mut eases: Vec<&str> = c
                .keys
                .iter()
                .take(c.keys.len() - 1)
                .map(|k| match k.2 {
                    Ease::Smooth => "smooth",
                    Ease::Linear => "linear",
                    Ease::Out => "out",
                    Ease::In => "in",
                    Ease::InOut => "in_out",
                    Ease::ExpoOut => "expo_out",
                    Ease::BackOut => "back_out",
                    Ease::BounceOut => "bounce_out",
                })
                .collect();
            eases.sort_unstable();
            eases.dedup();
            let unit = match c.path {
                Path::Location => "mm",
                Path::Rotation => "deg",
                Path::Scale => "scale",
            };
            for e in eases {
                let e = by.entry(format!("{} {e} ({unit})", call.func)).or_default();
                e.0 += 1;
                e.1 += exact as usize;
                e.2 = e.2.max(d);
            }
        }
    }
    let report: Vec<String> = by
        .iter()
        .map(|(k, (n, exact, far))| {
            format!("{k}: {exact} of {n} curves to the bit, the largest difference {far:.3e}")
        })
        .collect();
    println!("{}", report.join("\n"));
    println!("{} keys' handles differ", handles_off.len());
    for h in handles_off.iter().take(20) {
        println!("  {h}");
    }
    let bad: Vec<&String> = report
        .iter()
        .filter(|r| !r.contains("largest difference 0.000e0"))
        .collect();
    assert!(handles_off.is_empty() && bad.is_empty(), "{bad:?}");
}

/// A part with the rest a golden \`obj\` line gives it.
fn part(line: &str) -> (String, Obj) {
    let w: Vec<&str> = line.split(' ').collect();
    let field = |key: &str| -> Vec<f32> {
        let v = w.iter().find_map(|x| x.strip_prefix(key)).unwrap();
        f32s(&v.split(',').collect::<Vec<_>>())
    };
    let obj = scene::add(w[0], Mesh::default());
    let (loc, rot, scale) = (field("loc="), field("rot="), field("scale="));
    obj.edit(|o| {
        o.loc = V3::new(loc[0], loc[1], loc[2]);
        o.rot = V3::new(rot[0], rot[1], rot[2]);
        o.scale = V3::new(scale[0], scale[1], scale[2]);
    });
    (w[0].to_string(), obj)
}

#[test]
fn clips_key_as_gamekits() {
    let mut by: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let (mut bad, mut near) = (Vec::new(), Vec::new());
    for call in calls() {
        if call.func == "swing" || call.func == "slide" {
            continue;
        }
        scene::reset();
        // (the rest of each part before the call; for one that keys nothing, its rest after it follows)
        let before = if call.curves.is_empty() && call.func != "flat" {
            call.rest.len() / 2
        } else {
            call.rest.len()
        };
        let before = if call.func == "flat" { 1 } else { before };
        let parts: BTreeMap<String, Obj> = call.rest[..before].iter().map(|l| part(l)).collect();
        let arg = |k: &str| call.args.iter().find(|(n, _)| n == k).map(|(_, v)| v);
        let num = |k: &str, default: f64| arg(k).map_or(default, Val::num);
        let opt = |k: &str| {
            arg(k).and_then(|v| match v {
                Val::None => None,
                v => Some(v.num()),
            })
        };
        let obj = |k: &str| parts[arg(k).unwrap().str()];
        let objs = |k: &str| -> Vec<Obj> {
            arg(k)
                .unwrap()
                .list()
                .iter()
                .map(|v| parts[v.str()])
                .collect()
        };
        let clip = arg("clip").map(|v| v.str().to_string()).unwrap_or_default();
        let factors = |v: &Val| -> motion::Factors {
            match v {
                Val::List(l) => motion::Factors([l[0].num(), l[1].num(), l[2].num()]),
                v => motion::Factors([v.num(); 3]),
            }
        };
        match call.func.as_str() {
            "flat" => {
                motion::flat(obj("obj"), axis_of(arg("axis"), Axis::Z));
            }
            "attach" => {
                motion::attach(obj("obj"), obj("parent"));
            }
            "grow" => {
                motion::grow(
                    obj("obj"),
                    &clip,
                    motion::Grow {
                        seconds: num("seconds", 0.4),
                        axis: axis_of(arg("axis"), Axis::Z),
                    },
                );
            }
            "grow_in_turn" => {
                motion::grow_in_turn(
                    &objs("objs"),
                    &clip,
                    opt("seconds"),
                    axis_of(arg("axis"), Axis::Z),
                );
            }
            "spin" => {
                motion::spin(
                    obj("obj"),
                    &clip,
                    motion::Spin {
                        turns: num("turns", 1.0),
                        seconds: num("seconds", 1.0),
                        axis: axis_of(arg("axis"), Axis::Z),
                    },
                );
            }
            "scroll" => {
                motion::scroll(
                    obj("obj"),
                    &clip,
                    arg("offset").unwrap().p3(),
                    opt("seconds"),
                );
            }
            "rock" => {
                motion::rock(
                    obj("obj"),
                    &clip,
                    num("degrees", 0.0),
                    motion::Rock {
                        seconds: num("seconds", 1.0),
                        axis: axis_of(arg("axis"), Axis::X),
                    },
                );
            }
            "squash" => {
                motion::squash(
                    obj("obj"),
                    &clip,
                    factors(arg("scale").unwrap()),
                    motion::Squash {
                        seconds: num("seconds", 0.6),
                        back: num("back", 0.0) != 0.0,
                        split: num("split", 0.5),
                    },
                );
            }
            "stages" => {
                let sink = arg("sink").and_then(|v| match v {
                    Val::None => None,
                    v => {
                        let l = v.list();
                        Some((l[0].num(), l[1].num(), l[2].num()))
                    }
                });
                motion::stages(
                    obj("obj"),
                    &clip,
                    factors(arg("picked").unwrap()),
                    factors(arg("growing").unwrap()),
                    factors(arg("planted").unwrap()),
                    motion::Stages {
                        seconds: num("seconds", 1.5),
                        sink,
                    },
                );
            }
            "turns" => {
                motion::turns(
                    &objs("objs"),
                    &clip,
                    arg("offset").unwrap().p3(),
                    opt("seconds"),
                );
            }
            "path" => {
                let keys: Vec<motion::PathKey> = arg("keys")
                    .unwrap()
                    .list()
                    .iter()
                    .map(|k| {
                        let k = k.list();
                        let p = k[1].p3();
                        let r = k[2].p3();
                        (k[0].num(), (p.x, p.y, p.z), (r.x, r.y, r.z))
                    })
                    .collect();
                motion::path(obj("obj"), &clip, &keys, num("linear", 0.0) != 0.0);
            }
            "wobble" => {
                motion::wobble(
                    obj("obj"),
                    &clip,
                    num("degrees", 0.0),
                    motion::Wobble {
                        seconds: num("seconds", 1.0),
                        axis: axis_of(arg("axis"), Axis::X),
                        swings: num("swings", 3.0),
                        delay: num("delay", 0.0),
                    },
                );
            }
            "keys" => {
                let points: Vec<(f64, f64, Ease)> = arg("points")
                    .unwrap()
                    .list()
                    .iter()
                    .map(|p| {
                        let p = p.list();
                        (
                            p[0].num(),
                            p[1].num(),
                            p.get(2).map_or(Ease::Smooth, |e| ease_named(e.str())),
                        )
                    })
                    .collect();
                motion::keys(
                    obj("obj"),
                    &clip,
                    path_of(arg("prop").unwrap().str()),
                    num("index", 0.0) as usize,
                    &points,
                );
            }
            other => panic!("{other}"),
        }
        // the keys it made, against Blender's (whose dump of the clip also has what earlier calls keyed in it)
        let mut said = Vec::new();
        let mut made = 0;
        for (name, &o) in &parts {
            for ch in o.read(|p| {
                p.tracks
                    .iter()
                    .filter(|t| t.name == clip)
                    .flat_map(|t| t.channels.clone())
                    .collect::<Vec<_>>()
            }) {
                made += 1;
                if !call
                    .curves
                    .iter()
                    .any(|c| &c.obj == name && c.path == ch.path && c.index == ch.index)
                {
                    said.push(format!("{name} {:?}[{}] Blender hasn't", ch.path, ch.index));
                }
            }
        }
        if made == 0 && !call.curves.is_empty() {
            said.push("nothing keyed".to_string());
        }
        for c in &call.curves {
            let o = parts[&c.obj];
            let ch = o.read(|p| {
                p.tracks
                    .iter()
                    .find(|t| t.name == clip)
                    .and_then(|t| {
                        t.channels
                            .iter()
                            .find(|ch| ch.path == c.path && ch.index == c.index)
                    })
                    .cloned()
            });
            let Some(ch) = ch else {
                continue;
            };
            let got: Vec<(u32, u32, Ease)> = ch
                .keys
                .iter()
                .map(|k| ((k.frame as f32).to_bits(), k.value.to_bits(), k.ease))
                .collect();
            let want: Vec<(u32, u32, Ease)> = c
                .keys
                .iter()
                .map(|k| (k.0.to_bits(), k.1.to_bits(), k.2))
                .collect();
            if got != want {
                said.push(format!(
                    "{} {:?}[{}] keys {:?}, Blender's {:?}",
                    c.obj,
                    c.path,
                    c.index,
                    ch.keys,
                    c.keys.iter().map(|k| (k.0, k.1, k.2)).collect::<Vec<_>>()
                ));
            }
        }
        for (name, peak) in &call.peaks {
            let got = parts[name].read(|p| {
                p.tracks
                    .iter()
                    .find(|t| t.name == clip)
                    .and_then(|t| t.peak)
            });
            let got = got.map_or("N".to_string(), |p| p.to_string());
            if &got != peak {
                said.push(format!("{name} peak {got}, Blender's {peak}"));
            }
        }
        if call.func == "flat" || call.func == "attach" {
            // these key nothing: each part's place, turn and scale after, against what Blender had after
            for line in &call.rest[before..] {
                let name = line.split(' ').next().unwrap();
                let (_, want) = part(line);
                let get = |o: Obj| o.read(|p| [p.loc.array(), p.rot.array(), p.scale.array()]);
                let (a, b) = (get(parts[name]), get(want));
                // (under a turned parent the place goes through the parent's inverse, which Blender takes with
                // Eigen's vectorised routine and the kernel by adjugates: a few units out, a hundred-thousandth of a millimetre)
                let off = a
                    .iter()
                    .flatten()
                    .zip(b.iter().flatten())
                    .map(|(x, y)| ulps(*x, *y))
                    .max()
                    .unwrap_or(0);
                if off > 0 {
                    near.push(format!(
                        "call {} {} {name}: {off} units out",
                        call.n, call.at
                    ));
                }
                if off > 16 || a[1] != b[1] || a[2] != b[2] {
                    said.push(format!("{name} {a:?}, Blender's {b:?}"));
                }
            }
        }
        let e = by.entry(call.func.clone()).or_default();
        e.0 += 1;
        if said.is_empty() {
            e.1 += 1;
        } else {
            bad.push(format!(
                "call {} {} {}: {}",
                call.n,
                call.at,
                call.func,
                said.join("; ")
            ));
        }
    }
    let report: Vec<String> = by
        .iter()
        .map(|(k, (n, ok))| format!("{k}: {ok} of {n} calls keyed as Blender's"))
        .collect();
    println!("{}\nnear: {near:?}", report.join("\n"));
    assert!(bad.is_empty(), "{}", bad.join("\n"));
    assert!(near.len() <= 1, "{near:?}");
}

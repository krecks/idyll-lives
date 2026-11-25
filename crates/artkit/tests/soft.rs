//! The soft boxes the families make, against the ones gamekit made in Blender 5.2 (tests/golden/soft.txt): every
//! distinct `soft_box` and `GameFrame.soft` call the family scripts made, replayed in a fresh scene, each with a digest of
//! what Blender held (points to the bit in order, faces, sharp edges, smooth flags, materials, its place).
mod common;

use artkit::materials::Mat;
use artkit::math::P3;
use artkit::mesh::Mesh;
use artkit::place::{Coord, GameFrame};
use artkit::scene::{self, Obj};
use artkit::soft::{Puff, Sag, SoftBox, soft_box};
use common::*;

/// A value as the golden writes one.
#[derive(Clone, Debug)]
enum Val {
    None,
    Bool(bool),
    Int(i64),
    Num(f64),
    Str(String),
    Mat(String),
    List(Vec<Val>),
}

impl Val {
    fn num(&self) -> f64 {
        match self {
            Val::Int(i) => *i as f64,
            Val::Num(x) => *x,
            v => panic!("not a number: {v:?}"),
        }
    }
    fn nums(&self) -> Vec<f64> {
        match self {
            Val::List(l) => l.iter().map(Val::num).collect(),
            v => panic!("not a list: {v:?}"),
        }
    }
    fn p3(&self) -> P3 {
        let n = self.nums();
        P3 {
            x: n[0],
            y: n[1],
            z: n[2],
        }
    }
    fn count(&self) -> Option<usize> {
        match self {
            Val::None => None,
            v => Some(v.num() as usize),
        }
    }
    fn opt(&self) -> Option<f64> {
        match self {
            Val::None => None,
            v => Some(v.num()),
        }
    }
    fn mat(&self) -> Option<Mat> {
        match self {
            Val::None => None,
            Val::Mat(name) => Some(Mat::new(
                Box::leak(name.clone().into_boxed_str()),
                "#808080",
                "fabric",
            )),
            v => panic!("not a material: {v:?}"),
        }
    }
}

/// A coordinate keeps whether the script wrote it whole.
#[derive(Clone, Copy)]
struct C(f64, bool);

impl Coord for C {
    fn f(self) -> f64 {
        self.0
    }
    fn negated(self) -> f64 {
        if self.1 {
            -(self.0 as i64) as f64
        } else {
            -self.0
        }
    }
}

fn coords(v: &Val) -> (C, C, C) {
    let Val::List(l) = v else { panic!("{v:?}") };
    let c = |v: &Val| C(v.num(), matches!(v, Val::Int(_)));
    (c(&l[0]), c(&l[1]), c(&l[2]))
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
            Some(("m", x)) => Val::Mat(x.to_string()),
            _ => panic!("{word}"),
        },
    };
    (v, rest)
}

fn args(words: &[&str]) -> Vec<(String, Val)> {
    words
        .iter()
        .map(|w| {
            let (k, v) = w.split_once('=').unwrap();
            (k.to_string(), parse(v).0)
        })
        .collect()
}

/// The options named, set on `o`.
fn options(o: &mut SoftBox, named: &[(String, Val)]) {
    for (k, v) in named {
        match k.as_str() {
            "name" | "size" | "material" | "w" | "h" | "d" => {}
            "at" => o.at = v.p3(),
            "rot" => o.rot = v.p3(),
            "round" => o.round = v.num(),
            "puff" => {
                o.puff = match v {
                    Val::None => Puff::None,
                    Val::List(_) => {
                        let n = v.nums();
                        Puff::Each([n[0], n[1], n[2]])
                    }
                    v => Puff::Faces(v.num()),
                }
            }
            "round_segments" => o.round_segments = v.count(),
            "flat" => o.flat = v.count(),
            "piping" => o.piping = v.opt(),
            "piping_material" => o.piping_material = v.mat(),
            "pinch" => o.pinch = v.opt(),
            "waist" => o.waist = v.num(),
            "sag" => {
                o.sag = match v {
                    Val::List(_) => {
                        let n = v.nums();
                        Sag::Towards([n[0], n[1], n[2]])
                    }
                    v => Sag::Front(v.num()),
                }
            }
            "overhang" => o.overhang = v.num(),
            "tufts" => {
                o.tufts = match v {
                    Val::None => None,
                    v => {
                        let n = v.nums();
                        Some((n[0] as usize, n[1] as usize))
                    }
                }
            }
            "tuft" => o.tuft = v.num(),
            "diamond" => o.diamond = matches!(v, Val::Bool(true)),
            "button" => o.button = v.mat(),
            "rumple" => o.rumple = v.num(),
            "rumple_size" => o.rumple_size = v.num(),
            "seed" => o.seed = v.num(),
            "calm" => {
                o.calm = match v {
                    Val::None => None,
                    v => {
                        let n = v.nums();
                        Some((n[0], n[1]))
                    }
                }
            }
            "resting" => o.resting = matches!(v, Val::Bool(true)),
            other => panic!("an option soft_box hasn't: {other}"),
        }
    }
}

fn get<'a>(named: &'a [(String, Val)], key: &str) -> Option<&'a Val> {
    named.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

/// Builds the call a case line names.
fn replay(words: &[&str]) -> Obj {
    let low = words[3] == "1";
    let build = || match words[4] {
        "soft_box" => {
            let named = args(&words[5..]);
            let mut o = SoftBox::default();
            options(&mut o, &named);
            o.material = get(&named, "material").and_then(Val::mat);
            let Some(Val::Str(name)) = get(&named, "name") else {
                panic!("a name")
            };
            soft_box(name, get(&named, "size").unwrap().p3(), o)
        }
        "frame" => {
            let bar = words.iter().position(|w| *w == "|").unwrap();
            let head = args(&words[5..bar]);
            let kw = args(&words[bar + 1..]);
            let shift = get(&head, "shift").unwrap().num();
            let frame = match get(&head, "depths").unwrap() {
                Val::None => GameFrame::new(shift),
                Val::List(pairs) => {
                    let pairs: Vec<(f64, f64)> = pairs
                        .iter()
                        .map(|p| {
                            let n = p.nums();
                            (n[0], n[1])
                        })
                        .collect();
                    GameFrame::with_depths(shift, &pairs)
                }
                v => panic!("{v:?}"),
            };
            let mut o = SoftBox::default();
            options(&mut o, &kw);
            if let Some(puff) = get(&head, "puff") {
                options(&mut o, &[("puff".to_string(), puff.clone())]);
            }
            let Some(Val::Str(name)) = get(&head, "name") else {
                panic!("a name")
            };
            let n = |k: &str| get(&head, k).unwrap().num();
            frame.soft(
                name,
                n("w"),
                n("h"),
                n("d"),
                coords(get(&head, "at").unwrap()),
                get(&head, "material").and_then(Val::mat),
                o,
            )
        }
        other => panic!("{other}"),
    };
    if low {
        artkit::far::far_of(build)
    } else {
        build()
    }
}

/// FNV-1a, 64 bits.
fn fnv(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{h:016x}")
}

/// The digest line the golden has for a mesh and its place: what must be Blender's, then (after ` | `) its faces in
/// their order, which Blender keeps from one run to the next only where it didn't make them in bmesh (a sphere's).
fn digest(mesh: &Mesh, place: [f32; 6]) -> String {
    let vb: Vec<u8> = mesh
        .verts
        .iter()
        .flat_map(|v| [v.x, v.y, v.z])
        .flat_map(|x| x.to_le_bytes())
        .collect();
    let mut fb = Vec::new();
    for f in 0..mesh.faces.len() {
        for &v in mesh.corners(f) {
            fb.extend(v.to_le_bytes());
        }
        fb.extend([0xff; 4]);
    }
    let eb: Vec<u8> = mesh
        .sharp
        .iter()
        .flat_map(|&(a, b)| [a, b])
        .flat_map(|x| x.to_le_bytes())
        .collect();
    let sm: String = mesh
        .faces
        .iter()
        .map(|f| if f.smooth { '1' } else { '0' })
        .collect();
    let mi: String = mesh.faces.iter().map(|f| f.mat.to_string()).collect();
    let mats: Vec<&str> = mesh.mats.iter().map(|m| m.name).collect();
    let at: Vec<String> = place
        .iter()
        .map(|x| format!("{:08x}", x.to_bits()))
        .collect();
    // the faces as a set, each with its material and smooth flag
    let mut set: Vec<(Vec<u32>, u32, u32)> = (0..mesh.faces.len())
        .map(|f| {
            let face = mesh.faces[f];
            (
                mesh.corners(f).to_vec(),
                face.mat as u32,
                face.smooth as u32,
            )
        })
        .collect();
    set.sort();
    let mut fsb = Vec::new();
    for (corners, mat, smooth) in &set {
        for &v in corners.iter().chain([mat, smooth]) {
            fsb.extend(v.to_le_bytes());
        }
        fsb.extend([0xff; 4]);
    }
    format!(
        "digest {} {} v={} e={} faces={} mats={} at={} | f={} s={} m={}",
        mesh.verts.len(),
        mesh.faces.len(),
        fnv(&vb),
        fnv(&eb),
        fnv(&fsb),
        mats.join(","),
        at.join(" "),
        fnv(&fb),
        fnv(sm.as_bytes()),
        fnv(mi.as_bytes()),
    )
}

#[test]
fn soft_boxes_are_gamekits_in_blender() {
    let text = golden("soft.txt");
    let lines: Vec<&str> = text.lines().collect();
    let (mut cases, mut bad, mut near, mut in_order) = (0, Vec::new(), Vec::new(), 0);
    let mut i = 0;
    while i < lines.len() {
        let words: Vec<&str> = lines[i].split(' ').collect();
        assert_eq!(words[0], "case", "{}", lines[i]);
        let want = lines[i + 1];
        i += 2;
        scene::reset();
        let obj = replay(&words);
        let (mesh, loc, rot) = obj.read(|o| (o.mesh.clone(), o.loc, o.rot));
        let got = digest(&mesh, [loc.x, loc.y, loc.z, rot.x, rot.y, rot.z]);
        cases += 1;
        // the whole mesh, where the golden has it: how many points are Blender's to the bit
        let mut points = String::new();
        let mut far_off = None;
        if i < lines.len() && lines[i].starts_with("v ") {
            let w: Vec<&str> = lines[i][2..].split(' ').collect();
            let want: Vec<[f32; 3]> = f32s(&w).chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
            if want.len() == mesh.verts.len() {
                let (off, far) = points_off(&mesh.verts, &want);
                far_off = Some(far);
                points = format!(
                    " ({} of {} points to the bit, the farthest {far} units off)",
                    want.len() - off,
                    want.len()
                );
            }
            while i < lines.len() && !lines[i].starts_with("case ") {
                i += 1;
            }
        }
        let (want, want_order) = want.split_once(" | ").unwrap();
        let (got, got_order) = got.split_once(" | ").unwrap();
        in_order += (want_order == got_order) as usize;
        if got != want {
            // which of the digest's parts differ
            let parts = |s: &str| -> Vec<String> {
                let (head, at) = s.split_once(" at=").unwrap_or((s, ""));
                let mut p: Vec<String> = head.split(' ').map(str::to_string).collect();
                p.push(format!("at={at}"));
                p
            };
            let (w, g) = (parts(want), parts(got));
            let differ: Vec<String> = w
                .iter()
                .zip(&g)
                .filter(|(a, b)| a != b)
                .map(|(a, b)| {
                    let key = a.split('=').next().unwrap_or(a);
                    if key == "at" || key == "mats" || !a.contains('=') {
                        format!("{a} (got {b})")
                    } else {
                        key.to_string()
                    }
                })
                .collect();
            let line = format!("{} {}{points}: {}", words[1], words[2], differ.join(", "));
            // A piping cord's points are swept through turns from one ring to the next, and Blender's C library
            // takes the angle's asinf and the half angle's sine and cosine (one call) its own way, a unit out now
            // and then (math/): a few of the cord's points a unit or two out, everything else to the bit.
            if differ == ["v"] && far_off.is_some_and(|f| f <= 2) {
                near.push(line);
            } else {
                bad.push(line);
            }
        }
    }
    println!(
        "{} of {cases} soft boxes are Blender's to the bit, {in_order} with their faces in Blender's order; {} a cord's \
         point or two a unit out:\n{}",
        cases - bad.len() - near.len(),
        near.len(),
        near.join("\n")
    );
    assert!(near.len() <= 3, "{near:?}");
    assert!(
        bad.is_empty(),
        "{} of {cases} differ:\n{}",
        bad.len(),
        bad.join("\n")
    );
}

#[test]
fn resting_on_a_face_no_longer_than_its_rounding_makes_no_plain_face() {
    for puff in [0.01, 0.02] {
        let faces = |resting| {
            scene::reset();
            let o = SoftBox {
                round: 0.1,
                puff: Puff::Faces(puff),
                resting,
                ..Default::default()
            };
            soft_box("bolster", (0.6, 0.2, 0.2), o).read(|p| p.mesh.faces.len())
        };
        assert_eq!(faces(true), faces(false), "puff {puff}");
    }
}

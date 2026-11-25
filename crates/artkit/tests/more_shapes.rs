//! The torus and the outlines against gamekit's in Blender 5.2 (tests/golden/more_shapes.txt): torus,
//! rounded_rect, smooth_outline and circle as the family scripts called them, with what they made.
mod common;

use artkit::forms::{circle, rounded_rect, smooth_outline};
use artkit::materials::Mat;
use artkit::math::P3;
use artkit::mesh::Mesh;
use artkit::scene::{self, Obj};
use artkit::shapes::{Torus, torus};
use common::*;

#[derive(Clone, Debug)]
enum Val {
    None,
    Bool(bool),
    Int(i64),
    Num(f64),
    Str(String),
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
            Some(("s", x)) | Some(("m", x)) => Val::Str(x.to_string()),
            _ => panic!("{word}"),
        },
    };
    (v, rest)
}

fn mat(name: &str) -> Mat {
    Mat::new(
        Box::leak(name.to_string().into_boxed_str()),
        "#808080",
        "satin",
    )
}

/// What differs between a part and a golden mesh dump (its \`place\`, \`v\`, \`f\`, \`s\`, \`e\`, \`mats\` lines).
fn mesh_off(obj: Obj, lines: &[&str]) -> Vec<String> {
    let (mesh, loc, rot, scale): (Mesh, _, _, _) =
        obj.read(|o| (o.mesh.clone(), o.loc, o.rot, o.scale));
    let get = |k: &str| {
        lines
            .iter()
            .find_map(|l| {
                l.strip_prefix(&format!("{k} "))
                    .or_else(|| (*l == k).then_some(""))
            })
            .unwrap_or("")
    };
    let mut said = Vec::new();
    let place: Vec<&str> = get("place").split(' ').take(9).collect();
    let got = [loc.array(), rot.array(), scale.array()].concat();
    if bits(&got) != bits(&f32s(&place)) {
        said.push(format!("placed {got:?}"));
    }
    let want: Vec<[f32; 3]> = f32s(
        &get("v")
            .split(' ')
            .filter(|w| !w.is_empty())
            .collect::<Vec<_>>(),
    )
    .chunks(3)
    .map(|c| [c[0], c[1], c[2]])
    .collect();
    if want.len() != mesh.verts.len() {
        said.push(format!("{} points for {}", mesh.verts.len(), want.len()));
        return said;
    }
    let (off, far) = points_off(&mesh.verts, &want);
    let apart = mesh
        .verts
        .iter()
        .zip(&want)
        .map(|(g, w)| (0..3).map(|k| (g[k] - w[k]).abs()).fold(0.0, f32::max))
        .fold(0.0, f32::max);
    if off > 0 {
        said.push(format!(
            "{off} of {} points off, by up to {far} units ({apart} m)",
            want.len()
        ));
    }
    let faces: Vec<String> = (0..mesh.faces.len())
        .map(|f| {
            mesh.corners(f)
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(",")
        })
        .collect();
    let want_faces: Vec<String> = get("f").split(' ').map(str::to_string).collect();
    if faces != want_faces {
        said.push("faces differ".to_string());
    }
    let smooth: String = mesh
        .faces
        .iter()
        .map(|f| if f.smooth { '1' } else { '0' })
        .collect();
    if smooth != get("s") {
        said.push("smooth flags differ".to_string());
    }
    let mut sharp: Vec<(u32, u32)> = get("e")
        .split(' ')
        .filter(|w| !w.is_empty())
        .map(|e| {
            let (a, b) = e.split_once('-').unwrap();
            artkit::mesh::edge(a.parse().unwrap(), b.parse().unwrap())
        })
        .collect();
    sharp.sort_unstable();
    if mesh.sharp.iter().copied().collect::<Vec<_>>() != sharp {
        said.push("sharp edges differ".to_string());
    }
    let mats: Vec<&str> = mesh.mats.iter().map(|m| m.name).collect();
    if mats.join(" ") != get("mats") {
        said.push(format!("materials {mats:?}"));
    }
    said
}

fn pts_off(got: &[(f64, f64)], line: &str) -> Vec<String> {
    let (vals, _) = line.split_once(" | ").unwrap();
    let want = f64s(
        &vals
            .split(' ')
            .filter(|w| !w.is_empty())
            .collect::<Vec<_>>(),
    );
    let got: Vec<f64> = got.iter().flat_map(|p| [p.0, p.1]).collect();
    if got.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
        == want.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
    {
        Vec::new()
    } else if got
        .iter()
        .zip(&want)
        .all(|(a, b)| (*a as f32).to_bits() == (*b as f32).to_bits())
    {
        vec!["single".to_string()]
    } else {
        vec![format!("points {got:?}, Blender's {want:?}")]
    }
}

#[test]
fn tori_and_outlines_are_gamekits_in_blender() {
    let text = golden("more_shapes.txt");
    let lines: Vec<&str> = text.lines().collect();
    // per function: (calls, to the bit, the same to single precision)
    let mut by: std::collections::BTreeMap<String, (usize, usize, usize)> = Default::default();
    let mut bad = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let head: Vec<&str> = lines[i].split(' ').collect();
        let mut j = i + 1;
        while j < lines.len() && !lines[j].starts_with("call ") {
            j += 1;
        }
        let body = &lines[i + 1..j];
        i = j;
        let func = head[4];
        let args: Vec<(String, Val)> = head[5..]
            .iter()
            .filter_map(|a| a.split_once('='))
            .map(|(k, v)| (k.to_string(), parse(v).0))
            .collect();
        let arg = |k: &str| args.iter().find(|(n, _)| n == k).map(|(_, v)| v);
        scene::reset();
        let said = match func {
            "torus" => {
                let mut o = Torus::default();
                if let Some(v) = arg("at") {
                    o.at = v.p3();
                }
                if let Some(v) = arg("rot") {
                    o.rot = v.p3();
                }
                if let Some(Val::Str(m)) = arg("material") {
                    o.material = Some(mat(m));
                }
                if let Some(v) = arg("major_segments") {
                    o.major_segments = v.num() as usize;
                }
                if let Some(v) = arg("minor_segments") {
                    o.minor_segments = v.num() as usize;
                }
                if let Some(Val::Bool(b)) = arg("smooth") {
                    o.smooth = *b;
                }
                let obj = torus(
                    "t",
                    arg("major_radius").unwrap().num(),
                    arg("minor_radius").unwrap().num(),
                    o,
                );
                mesh_off(obj, body)
            }
            "rounded_rect" => {
                let n = |k: &str| arg(k).unwrap().num();
                let corner = arg("corner").map_or(4, |v| v.num() as usize);
                pts_off(
                    &rounded_rect(n("w"), n("h"), n("r"), corner),
                    body[0].strip_prefix("pts ").unwrap(),
                )
            }
            "smooth_outline" => {
                let pts: Vec<(f64, f64)> = arg("points")
                    .unwrap()
                    .list()
                    .iter()
                    .map(|p| (p.list()[0].num(), p.list()[1].num()))
                    .collect();
                let passes = arg("passes").map_or(2, |v| v.num() as usize);
                let closed = !matches!(arg("closed"), Some(Val::Bool(false)));
                pts_off(
                    &smooth_outline(&pts, passes, closed),
                    body[0].strip_prefix("pts ").unwrap(),
                )
            }
            "circle" => {
                let verts = arg("verts").map_or(12, |v| v.num() as usize);
                pts_off(
                    &circle(arg("r").unwrap().num(), verts),
                    body[0].strip_prefix("pts ").unwrap(),
                )
            }
            other => panic!("{other}"),
        };
        let e = by.entry(func.to_string()).or_default();
        e.0 += 1;
        if said.is_empty() {
            e.1 += 1;
        } else if said == ["single"] {
            // the same points as singles: the double a unit out, from the C library's cosine or sine (Apple's in
            // Blender, the libm crate's here)
            e.2 += 1;
        } else {
            bad.push(format!(
                "{} {} {func}: {}",
                head[1],
                head[2],
                said.join("; ")
            ));
        }
    }
    let report: Vec<String> = by
        .iter()
        .map(|(k, (n, ok, single))| {
            format!("{k}: {ok} of {n} Blender's to the bit, {single} to single precision")
        })
        .collect();
    println!("{}", report.join("\n"));
    assert!(bad.is_empty(), "{}\n{}", report.join("\n"), bad.join("\n"));
}

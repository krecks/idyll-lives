//! The plants' library (families/plants.py's shapes: stems, pots, flowers, leaves cut from an outline, shrubs, beds)
//! against what Blender 5.2 made of it (tests/golden/botany_*.txt: each function at the arguments the families call
//! it with, at full detail and as a far version, with the random draws it takes; the shrubs and beds built whole,
//! fitted, joined and settled, their perches found and their clips keyed).
mod common;

use artkit::botany::{
    self, _fit_with_part, _flower, _with_part, MEADOW_FLOWERS, ShrubMass, fiddle, fit_to,
    hedge_top, lupin_spike, raised_bed, sword,
};
use artkit::far::{far_of, is_low, low};
use artkit::materials::{Finish, Finishes, Palette};
use artkit::math::{PI, V3, asin, atan2, cos, degrees, pow, radians, sin, v3};
use artkit::parts::{join_all, new_parts, settle};
use artkit::rand::Random;
use artkit::scene::{self, Obj};
use artkit::{
    _crop, _put, arc, blade, cone, cube, cut_leaf, dome, dots, florets, leaf, leafy_shrub, lobed,
    lump, mound, normals, on_masses, outline, perch, perches, pinnate, rays, ribbed, rose_head,
    rose_leaf, rosettes, slits, soil_top, sphere, spiral, stalk, star, stem, tulip_head, twig,
    vessel,
};
use common::*;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// The finishes of the base manifest the plants' palette wears.
fn finishes() -> Finishes {
    let f = |roughness, textured| Finish {
        roughness,
        metallic: 0.0,
        textured,
    };
    [
        ("bark", f(0.6, true)),
        ("foliage", f(0.9, true)),
        ("wood", f(0.6, true)),
        ("satin", f(0.42, false)),
        ("matte", f(0.75, false)),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect()
}

thread_local! {
    /// The perches as a model's build found them, before it was settled.
    static FOUND: RefCell<Vec<V3>> = const { RefCell::new(Vec::new()) };
}

fn keep_found() {
    FOUND.with(|f| *f.borrow_mut() = scene::with(|s| s.perches.clone()));
}

/// Metres: how far a model that was fitted to its size and then joined may be from Blender's (half a thousandth of
/// a millimetre). The functions are Blender's to the bit, and so is a model that is joined as it was built (the
/// rose bush); `fit_to` scales the parts, and the kernel's join then takes each part's points through the first
/// part's frame and back, where Blender leaves them be (it skips a move that is within a millionth of none): a
/// unit or two in the last place at most of the points.
const JOINED: f32 = 5e-7;

/// What a case returned: nothing, or its numbers in order.
type Ret = Vec<f64>;

fn flat2(pairs: &[(f64, f64)]) -> Vec<f64> {
    pairs.iter().flat_map(|&(a, b)| [a, b]).collect()
}

fn flat3(points: &[V3]) -> Vec<f64> {
    points
        .iter()
        .flat_map(|p| p.array().map(f64::from))
        .collect()
}

/// A golden's case: its own lines (what it returned, its size and perches) and its parts.
struct Case {
    name: String,
    lines: BTreeMap<String, String>,
    meshes: Vec<GoldenMesh>,
}

fn cases(file: &str) -> Vec<Case> {
    let mut out: Vec<Case> = Vec::new();
    for line in golden(file).lines() {
        let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
        match key {
            "#" => out.push(Case {
                name: rest.to_string(),
                lines: BTreeMap::new(),
                meshes: Vec::new(),
            }),
            "mesh" => out.last_mut().unwrap().meshes.push(GoldenMesh {
                name: rest.split(' ').next().unwrap().to_string(),
                ..GoldenMesh::default()
            }),
            _ => {
                let case = out.last_mut().unwrap();
                let Some(mesh) = case.meshes.last_mut() else {
                    case.lines.insert(key.to_string(), rest.to_string());
                    continue;
                };
                let words: Vec<&str> = rest.split(' ').filter(|w| !w.is_empty()).collect();
                match key {
                    "v" => {
                        mesh.verts = f32s(&words).chunks(3).map(|c| [c[0], c[1], c[2]]).collect()
                    }
                    "f" => {
                        mesh.faces = words
                            .iter()
                            .map(|f| f.split(',').map(|i| i.parse().unwrap()).collect())
                            .collect()
                    }
                    "t" => {
                        let w: Vec<u32> = words.iter().map(|i| i.parse().unwrap()).collect();
                        mesh.tris = w.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
                    }
                    _ => {
                        mesh.lines.insert(key.to_string(), rest.to_string());
                    }
                }
            }
        }
    }
    out
}

fn words(line: &str) -> Vec<&str> {
    line.split(' ').filter(|w| !w.is_empty()).collect()
}

/// Each golden face's own among a built mesh's (the faces are the same, in an order of Blender's), and each golden
/// corner's.
fn its_order(mesh: &artkit::mesh::Mesh, faces: &[Vec<u32>]) -> (Vec<usize>, Vec<usize>) {
    let at: BTreeMap<&[u32], usize> = (0..mesh.faces.len())
        .map(|f| (mesh.corners(f), f))
        .collect();
    let of: Vec<usize> = faces.iter().map(|f| at[f.as_slice()]).collect();
    let corner = faces
        .iter()
        .zip(&of)
        .flat_map(|(f, &mine)| {
            let start = mesh.faces[mine].start as usize;
            (0..f.len()).map(move |i| start + i)
        })
        .collect();
    (of, corner)
}

/// How a part is held to its golden: its faces in Blender's order too (a mesh a script made face by face), and how
/// far its points may be from Blender's, in units in the last place (0: to the bit).
#[derive(Clone, Copy, Default)]
struct Hold {
    ordered: bool,
    /// How far the numbers it returns may be from Blender's, in units in a double's last place.
    ret_ulps: u64,
    /// How far a model's size may be from Blender's, metres.
    size: f64,
    /// How far a part's turn may be from Blender's, in units in the last place.
    turn_ulps: u64,
    /// How far a joined model's points, place and perches may be from Blender's, metres (0: to the bit).
    metres: f32,
}

/// How far the built parts were from the goldens where they may be at all: the farthest point, in units in the last
/// place, and the normals' farthest.
#[derive(Default, Debug)]
struct Off {
    turn: u64,
    size: f64,
    returned: u64,
    /// The farthest a point or a perch was from Blender's, metres, and how many points of how many weren't its bits.
    metres: f32,
    moved: usize,
    points: usize,
    normals: f32,
    corners: usize,
    inexact: usize,
}

/// Checks the part `obj` against the golden `g`: its place, its points to the bit and in order, its faces (each
/// from the corner Blender's starts at) and triangles as sets, its shading, materials, sharp edges, UVs, the bevel
/// it is to be given and its clips' keys, to the bit; the normals in Blender's order of faces to the bit.
fn same(case: &str, obj: Obj, g: &GoldenMesh, hold: Hold, off: &mut Off) {
    let what = format!("{case}: {}", g.name);
    let (mesh, loc, rot, scale) = obj.read(|o| (o.mesh.clone(), o.loc, o.rot, o.scale));
    let place = [loc.array(), rot.array(), scale.array()].concat();
    // (a zero's sign isn't a difference: Blender leaves -0 in a turn it has applied)
    let want = g.f32s("loc");
    let turned = (0..9)
        .filter(|&i| place[i] != want[i] && (place[i] - want[i]).abs() > hold.metres)
        .map(|i| {
            if (3..6).contains(&i) {
                ulps(place[i], want[i])
            } else {
                u64::MAX
            }
        })
        .max()
        .unwrap_or(0);
    assert!(
        turned <= hold.turn_ulps,
        "{what}: its place {place:?}, Blender's {want:?}"
    );
    off.turn = off.turn.max(turned);
    assert_eq!(mesh.verts.len(), g.verts.len(), "{what}: points");
    let (n, far) = points_off(&mesh.verts, &g.verts);
    let apart = mesh
        .verts
        .iter()
        .zip(&g.verts)
        .map(|(p, w)| (0..3).map(|i| (p[i] - w[i]).abs()).fold(0.0, f32::max))
        .fold(0.0, f32::max);
    assert!(
        far == 0 || apart <= hold.metres,
        "{what}: {n} points differ, by up to {far} ulps, {apart:e} m"
    );
    if far > 0 {
        off.metres = off.metres.max(apart);
        off.moved += n;
    }
    off.points += mesh.verts.len();
    let faces: Vec<Vec<u32>> = (0..mesh.faces.len())
        .map(|f| mesh.corners(f).to_vec())
        .collect();
    if hold.ordered {
        assert_eq!(faces, g.faces, "{what}: faces, in order");
    } else {
        assert_eq!(sorted(&faces), sorted(&g.faces), "{what}: faces");
    }
    // (a big part's triangles aren't in the golden: how a quad is cut is the kernel's, proved on its own)
    if !g.tris.is_empty() {
        assert_eq!(
            as_set(&mesh.triangle_verts()),
            as_set(&g.tris),
            "{what}: triangles"
        );
    }
    let (of, corner) = its_order(&mesh, &g.faces);
    // (botany eases what the scripts left sharp, a pot's and a saucer's rims and a raised bed's boards: their bevel,
    // asked for, shades them smooth with their sharp edges kept; their points and faces are still Blender's)
    let eased = !g.lines.contains_key("bevel")
        && obj.read(|o| o.bevel.is_some())
        && ["pot", "saucer", "board"]
            .iter()
            .any(|n| g.name.starts_with(n));
    let smooth: String = of
        .iter()
        .map(|&f| if mesh.faces[f].smooth { '1' } else { '0' })
        .collect();
    assert!(eased || smooth == g.lines["s"], "{what}: smooth faces");
    let mats = words(&g.lines["mats"]);
    let wears: Vec<&str> = of
        .iter()
        .map(|&f| mesh.mats[mesh.faces[f].mat as usize].name)
        .collect();
    let want: Vec<&str> = words(&g.lines["m"])
        .iter()
        .map(|i| mats[i.parse::<usize>().unwrap()])
        .collect();
    assert_eq!(wears, want, "{what}: each face's material");
    assert_eq!(
        mesh.mats.iter().map(|m| m.name).collect::<Vec<_>>(),
        mats,
        "{what}: materials"
    );
    let mut sharp: Vec<(u32, u32)> = words(&g.lines["e"])
        .iter()
        .map(|e| {
            let (a, b) = e.split_once('-').unwrap();
            artkit::mesh::edge(a.parse().unwrap(), b.parse().unwrap())
        })
        .collect();
    sharp.sort_unstable();
    assert!(
        eased || mesh.sharp.iter().copied().collect::<Vec<_>>() == sharp,
        "{what}: sharp edges"
    );
    if let Some(uv) = g.lines.get("uv") {
        let want = f32s(&words(uv));
        assert_eq!(want.len(), 2 * mesh.loops.len(), "{what}: a UV a corner");
        let uvs = mesh
            .uvs
            .as_ref()
            .unwrap_or_else(|| panic!("{what}: its UVs"));
        let wrong = corner
            .iter()
            .enumerate()
            .filter(|&(l, &mine)| bits(&uvs[mine]) != bits(&want[2 * l..2 * l + 2]))
            .count();
        assert_eq!(wrong, 0, "{what}: UVs differ, of {}", corner.len());
        assert_eq!(
            mesh.uv_metres,
            g.lines["uvm"].starts_with('1'),
            "{what}: UVs made"
        );
    }
    let bevel = obj.read(|o| o.bevel.is_some());
    assert_eq!(
        bevel && !eased,
        g.lines.contains_key("bevel"),
        "{what}: a bevel asked for"
    );
    // The normals: Blender's to the bit with the faces round each vertex summed in its order (which for its own
    // primitives isn't the same from one run of it to the next); in the kernel's own order they are measured.
    let cn = g.f32s("cn");
    if !cn.is_empty() && !eased {
        let mine = normals::corner_normals(&mesh);
        let mut in_its_order = artkit::mesh::Mesh::from_pydata(mesh.verts.clone(), &g.faces);
        for (face, &f) in in_its_order.faces.iter_mut().zip(&of) {
            face.smooth = mesh.faces[f].smooth;
        }
        in_its_order.sharp = mesh.sharp.clone();
        let its = normals::corner_normals(&in_its_order);
        for (l, &c) in corner.iter().enumerate() {
            let want = &cn[3 * l..3 * l + 3];
            let d = (0..3)
                .map(|i| (mine[c][i] - want[i]).abs())
                .fold(0.0, f32::max);
            if hold.metres == 0.0 {
                assert_eq!(
                    bits(&its[l].array()),
                    bits(want),
                    "{what}: the normal at corner {l}"
                );
            }
            off.normals = off.normals.max(d);
            off.corners += 1;
            off.inexact += (bits(&mine[c].array()) != bits(want)) as usize;
        }
    }
    // its clips: each keyed channel's frames and values
    let mut clips: Vec<(String, String)> = Vec::new();
    obj.read(|o| {
        for t in &o.tracks {
            for ch in &t.channels {
                let path = match ch.path {
                    artkit::motion::Path::Location => "location",
                    artkit::motion::Path::Rotation => "rotation_euler",
                    artkit::motion::Path::Scale => "scale",
                };
                let keys: Vec<String> = ch
                    .keys
                    .iter()
                    .map(|k| format!("{}:{:08x}", k.frame, k.value.to_bits()))
                    .collect();
                clips.push((format!("fc.{}.{path}.{}", t.name, ch.index), keys.join(" ")));
            }
        }
    });
    clips.sort();
    let want: Vec<(String, String)> = g
        .lines
        .iter()
        .filter(|(k, _)| k.starts_with("fc."))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    assert_eq!(clips, want, "{what}: its clips");
}

/// Runs every case of the golden `file` through `build` (at full detail and as a far version) and holds what it
/// makes to it; how many cases there were.
fn check(
    file: &str,
    build: impl Fn(&str, &Palette) -> Option<Ret>,
    hold: impl Fn(&str) -> Hold,
    off: &mut Off,
) -> usize {
    scene::set_finishes(finishes());
    let all = cases(file);
    for case in &all {
        let (name, far) = match case.name.strip_suffix(".far") {
            Some(name) => (name, true),
            None => (case.name.as_str(), false),
        };
        scene::reset();
        perch::clear();
        botany::clear();
        FOUND.with(|f| f.borrow_mut().clear());
        let p = botany::palette(&[]);
        let ret = if far {
            far_of(|| build(name, &p))
        } else {
            build(name, &p)
        };
        match case.lines.get("ret") {
            Some(want) => {
                let got = ret.unwrap_or_else(|| panic!("{}: what it returns", case.name));
                let want = f64s(&words(want));
                assert_eq!(got.len(), want.len(), "{}: what it returns", case.name);
                let apart =
                    |a: f64, b: f64| (a.to_bits() as i64 - b.to_bits() as i64).unsigned_abs();
                let far = got.iter().zip(&want).map(|(&a, &b)| apart(a, b)).max();
                let wrong = got
                    .iter()
                    .zip(&want)
                    .filter(|&(&a, &b)| apart(a, b) > 0)
                    .count();
                assert!(
                    far.unwrap_or(0) <= hold(name).ret_ulps,
                    "{}: {wrong} of the {} numbers it returns differ, by up to {far:?} ulps",
                    case.name,
                    got.len()
                );
                off.returned = off.returned.max(far.unwrap_or(0));
            }
            None => assert!(
                ret.is_none_or(|r| r.is_empty()),
                "{}: returns nothing",
                case.name
            ),
        }
        if let Some(size) = case.lines.get("size") {
            // a model built whole: its size, its perches as found and as settled
            let made = scene::meshes();
            let got = artkit::parts::game_size(&made);
            let want = f64s(&words(size));
            let apart = (0..3).map(|i| (got[i] - want[i]).abs()).fold(0.0, f64::max);
            assert!(
                apart <= hold(name).size,
                "{}: its size {got:?}, {apart:e} off Blender's",
                case.name
            );
            off.size = off.size.max(apart);
            let flat = |ps: &[V3]| ps.iter().flat_map(|p| p.array()).collect::<Vec<f32>>();
            let reach = |got: &[f32], want: &[f32], what: &str| {
                assert_eq!(got.len(), want.len(), "{}: the perches {what}", case.name);
                let far = got
                    .iter()
                    .zip(want)
                    .map(|(&a, &b)| (a - b).abs())
                    .fold(0.0, f32::max);
                assert!(
                    far <= hold(name).metres,
                    "{}: the perches {what} are up to {far:e} m off",
                    case.name
                );
                far
            };
            let found = FOUND.with(|f| f.borrow().clone());
            let a = reach(
                &flat(&found),
                &f32s(&words(&case.lines["found"])),
                "as found",
            );
            let settled = scene::with(|s| s.perches.clone());
            let b = reach(
                &flat(&settled),
                &f32s(&words(&case.lines["settled"])),
                "as settled",
            );
            off.metres = off.metres.max(a).max(b);
            let game: Vec<String> = perch::game()
                .iter()
                .map(|p| format!("[{}, {}, {}]", p[0], p[1], p[2]))
                .collect();
            assert_eq!(
                format!("[{}]", game.join(", ")),
                case.lines["game"],
                "{}",
                case.name
            );
        }
        let made = scene::meshes();
        assert_eq!(
            made.iter().map(|o| o.name()).collect::<Vec<_>>(),
            case.meshes
                .iter()
                .map(|g| g.name.clone())
                .collect::<Vec<_>>(),
            "{}: its parts",
            case.name
        );
        for (&obj, g) in made.iter().zip(&case.meshes) {
            same(&case.name, obj, g, hold(name), off);
        }
    }
    all.len()
}

// ---- stems, pots and flowers ----

fn flowers(name: &str, p: &Palette) -> Option<Ret> {
    let m = |k: &str| p.get(k);
    match name {
        "twig" => {
            twig!(
                "stem",
                &[(0.0, 0.0, 0.14), (0.004, 0.002, 0.2), (0.012, 0.004, 0.25)],
                0.003,
                0.0026,
                m("leaf"),
                verts = 4,
                res = 1
            );
        }
        "twig_branch" => {
            twig!(
                "branch",
                &[
                    v3(0.02, 0, 0.47),
                    v3(0.08, 0.03, 0.6),
                    v3(0.14, 0.08, 0.642),
                    v3(0.2, 0.02, 0.7)
                ],
                0.015,
                0.006,
                m("wood"),
                verts = 5
            );
        }
        "twig_two" => {
            twig!(
                "stalk",
                &[(0.1, 0.2, 0.104), (0.3, 0.25, 0.3)],
                0.0028,
                0.0022,
                m("leaf"),
                verts = 3,
                res = 1
            );
        }
        "twig_leg" => {
            twig!(
                "stand_leg",
                &[
                    (0.12 * cos(0.7), 0.12 * sin(0.7), 0.545),
                    (0.185 * cos(0.7), 0.185 * sin(0.7), 0.0)
                ],
                0.02,
                0.013,
                m("wood"),
                verts = 6,
                res = 1
            );
        }
        "twig_stalk" => {
            twig!(
                "stalk",
                &[
                    v3(0.2, -0.1, 0.06),
                    v3(0.2, -0.1, 0).lerp(v3(0.25, -0.05, 1.7), 0.5) + v3(0.01, 0, 0),
                    v3(0.25, -0.05, 1.7)
                ],
                0.02,
                0.013,
                m("stem"),
                verts = 6,
                res = 2
            );
        }
        "twig_plain" => {
            twig!(
                "cane",
                &[
                    v3(0.03, 0.02, 0.05),
                    v3(0.11, 0.07, 0.42),
                    v3(0.13, 0.09, 0.68)
                ],
                0.01,
                0.005,
                m("stem")
            );
        }
        "rays" => {
            rays!(
                "flower",
                (0, 0, 0.308),
                0.022,
                10,
                m("bloom"),
                inner = 0.3,
                cup = 0.45,
                width = 0.9,
                centre = 0.003
            );
        }
        "rays_sepals" => {
            rays!(
                "sepals",
                (0.1, -0.2, 0.5),
                0.05,
                3,
                m("bloom"),
                yaw = 192,
                tilt = 90,
                inner = 0.15,
                cup = 0.12,
                width = 0.55,
                thick = 0.004
            );
        }
        "rays_petals" => {
            rays!(
                "petals",
                v3(0.1, -0.2, 0.5) + V3::from(botany::polar(0.002, 192.0, 0.0)),
                0.05 * 0.95,
                2,
                m("bloom"),
                yaw = 192,
                tilt = 90,
                inner = 0.1,
                cup = 0.1,
                width = 0.75,
                thick = 0.004
            );
        }
        "rays_lily" => {
            rays!(
                "lily",
                v3(0.2, 0.1, 0.8),
                0.075,
                6,
                m("bloom3"),
                yaw = 33.0,
                tilt = 55,
                inner = 0.18,
                cup = 1.1,
                droop = 0.55,
                width = 0.7,
                shoulder = 0.4
            );
        }
        "rays_sun" => {
            rays!(
                "petals",
                v3(0.05, 0.3, 1.9),
                0.17,
                18,
                m("bloom"),
                yaw = 75.5,
                tilt = 62,
                inner = 0.5,
                cup = 0.05,
                droop = 0.06,
                width = 0.7,
                thick = 0.012
            );
        }
        "rays_rose" => {
            rays!(
                "rose_in",
                v3(0.3, -0.1, 0.7) + v3(0, 0, 0.05 * 0.12),
                0.05 * 0.62,
                4,
                m("bloom"),
                yaw = 140.0 + 30.0,
                tilt = 17.5,
                inner = 0.2,
                cup = 1.6,
                width = 1.1,
                thick = 0.012,
                centre = 0.05 * 0.15
            );
        }
        "rays_rose_out" => {
            rays!(
                "rose_out",
                v3(0.3, -0.1, 0.7),
                0.05,
                5,
                m("bloom"),
                yaw = 140.0,
                tilt = 17.5,
                inner = 0.25,
                cup = 0.75,
                width = 1.05,
                thick = 0.01
            );
        }
        "rays_pansy" => {
            rays!(
                "pansy",
                (0.12, -0.3, 0.21),
                0.045,
                5,
                m("bloom3"),
                yaw = 301.0,
                tilt = 38.0,
                inner = 0.25,
                cup = 0.05,
                width = 1.35,
                thick = 0.006
            );
        }
        "rays_poppy" => {
            rays!(
                "poppy",
                (0.1, 0.1, 0.6),
                0.045,
                4,
                m("bloom"),
                yaw = 12.0,
                tilt = 21.0,
                inner = 0.2,
                cup = 0.75,
                width = 1.25,
                thick = 0.008
            );
        }
        "rays_calyx" => {
            rays!(
                "calyx",
                v3(0.3, 0.3, 0.04) + v3(0, 0, 0.058),
                0.022,
                5,
                m("stalk"),
                inner = 0.2,
                cup = -0.2,
                width = 0.5,
                thick = 0.003
            );
        }
        "rays_daisy" => {
            rays!(
                "daisy",
                (-0.2, 0.15, 0.33),
                0.045,
                10,
                m("bloom"),
                yaw = 250.0,
                tilt = 22.0,
                inner = 0.32,
                cup = 0.06,
                width = 0.75,
                shoulder = 0.0
            );
        }
        "rays_eye" => {
            rays!(
                "eye",
                (0.12, -0.3, 0.214),
                0.02,
                3,
                m("centre_dark"),
                yaw = 301.0,
                tilt = 38.0,
                inner = 0.3,
                cup = 0.05,
                width = 1.2,
                thick = 0.012
            );
        }
        "rays_picked" => {
            rays!(
                "picked_flower",
                (0.06, -0.22, 0.48),
                0.06,
                12,
                m("bloom5"),
                yaw = 180,
                tilt = 30,
                inner = 0.3,
                cup = 0.08,
                width = 0.75
            );
        }
        "rays_leaf" => {
            rays!(
                "leaf",
                v3(0.1, 0.1, 0.3),
                0.085,
                9,
                m("leaf"),
                yaw = 77.0,
                tilt = 13.0,
                inner = 0.08,
                cup = 0.15,
                droop = 0.2,
                width = 0.42,
                thick = 0.004
            );
        }
        "rays_plain" => {
            rays!(
                "crown",
                v3(0.2, 0.2, 0.365) + V3::from(botany::polar(0.025, 48.0, 0.0)),
                0.018,
                6,
                m("bloom_accent"),
                yaw = 48.0,
                tilt = 25,
                inner = 0.3,
                cup = 0.5
            );
        }
        "dome" => {
            dome!("flower_eye", (0.11, 0.05, 0.92), 0.011, 0.007, m("centre"));
        }
        "dome_leaning" => {
            let at = v3(0.03, 0.04, 0.2) + v3(0, 0, 0.003);
            dome!(
                "eye",
                (at.x as f64, at.y as f64, at.z as f64),
                0.008,
                0.005,
                m("centre"),
                yaw = degrees(0.9),
                tilt = 0.05 / 0.06 * 50.0
            );
        }
        "dome_heart" => {
            dome!(
                "heart",
                (0, 0, 0.098),
                0.012,
                0.018,
                m("flesh_light"),
                lift = 0.0
            );
        }
        "dome_placed" => {
            dome!(
                "eye",
                (0.02, -0.03, 0.31),
                0.02 * 0.3,
                0.006,
                m("centre"),
                yaw = 215.0,
                tilt = 31.0
            );
        }
        "star" => {
            star!(
                "flower",
                (0.11, 0.05, 0.92),
                0.05,
                5,
                m("bloom3"),
                yaw = 2 * 40,
                tilt = 13.7,
                inner = 0.55,
                cup = 0.25
            );
        }
        "star_six" => {
            star!(
                "flower",
                v3(0.03, 0.04, 0.2),
                0.032,
                6,
                m("bloom2"),
                yaw = degrees(0.9),
                tilt = 0.05 / 0.06 * 50.0,
                inner = 0.6,
                cup = 0.15
            );
        }
        "star_lily" => {
            star!(
                "lily",
                (-0.4, -0.3, 0.071),
                0.09,
                8,
                m("bloom"),
                yaw = 22.5 * 1.0,
                inner = 0.4,
                depth = 0.006,
                cup = 0.5 + 0.35 * 1.0
            );
        }
        "star_calyx" => {
            star!(
                "calyx",
                (0.1, 0.2, 0.071),
                0.014,
                5,
                m("leaf"),
                inner = 0.35,
                depth = 0.003,
                cup = 0.2
            );
        }
        "star_sepals" => {
            star!(
                "sepals",
                (0.012, 0.004, 0.249),
                0.013,
                5,
                m("leaf"),
                inner = 0.3,
                depth = 0.002,
                cup = -0.3
            );
        }
        "star_plain" => {
            star!("flower", v3(0.2, -0.2, 0.5), 0.04, 7, m("bloom4"));
        }
        "tulip_head" => {
            tulip_head!(
                "tulip",
                v3(0.1, 0.2, 0.45) - v3(0, 0, 0.01),
                0.032,
                0.075,
                m("bloom"),
                yaw = 211.3,
                tilt = 7.1
            );
        }
        "tulip_bud" => {
            tulip_head!(
                "rose_bud",
                (0.012, 0.004, 0.253),
                0.0105,
                0.024,
                m("bloom5")
            );
        }
        "tulip_cut" => {
            tulip_head!(
                "flower",
                (0.03, -0.02, 0.3 - 0.012),
                0.017,
                0.042,
                m("bloom2"),
                yaw = 40.0,
                tilt = 12.0 * 0.6
            );
        }
        "soil_top" => {
            soil_top!(
                p,
                0.278,
                0.104,
                segments = low(20, 10),
                mound = 0.01,
                seed = 2.0,
                material = m("sand")
            );
        }
        "soil_top_pot" => {
            soil_top!(
                p,
                0.077,
                0.31,
                segments = low(12, 6),
                mound = 0.012,
                seed = 4.0
            );
        }
        "soil_top_plain" => {
            soil_top!(p, 0.19 - 0.008 - 0.002, 0.33 - 0.028, segments = 18);
        }
        "soil_top_small" => {
            soil_top!(
                p,
                0.05 - 0.01,
                0.12,
                segments = low(8, 6),
                mound = 0.006,
                seed = 3,
                material = m("sand")
            );
        }
        "soil_top_table" => {
            soil_top!(
                None,
                0.062,
                0.066,
                segments = 12,
                mound = 0.006,
                seed = 1.0,
                material = m("sand")
            );
        }
        "soil_top_default" => {
            soil_top!(p, 0.1, 0.2);
        }
        "ribbed" => {
            return Some(
                [
                    ribbed!(0.05, 7),
                    ribbed!(0.04, 5, depth = 0.14),
                    ribbed!(0.03, 8 / 2, depth = 0.1, verts = 8),
                    ribbed!(0.045, 7, depth = 0.16, verts = 21),
                    ribbed!(0.016, 5, depth = 0.2),
                ]
                .iter()
                .flat_map(|r| flat2(r))
                .collect(),
            );
        }
        "arc" => {
            return Some(
                [
                    arc!((0, 0, 0.31), v3(0.2, 0.1, 0.7), 0.06, steps = 3),
                    arc!(
                        V3::from(botany::polar(0.03, 37.0, 0.3)),
                        v3(0.1, 0.12, 0.45),
                        0.03,
                        steps = 2
                    ),
                    arc!((0.1, 0.2, 0.3), (0.4, -0.1, 0.5), 0.05),
                ]
                .iter()
                .flat_map(|a| flat3(a))
                .collect(),
            );
        }
        "stem" => {
            stem!(
                "picked_stem",
                (0.06, -0.22, 0),
                (0.06, -0.22, 0.48),
                m("stalk"),
                r = 0.005
            );
        }
        "stem_plain" => {
            stem!(
                "seed_stalk",
                (0.1 * 0.85, 0.2 * 0.85, 0),
                (0.1 * 0.8 + 0.03, 0.2 * 0.8 - 0.03, 0.4 * 1.02),
                m("leaf_light")
            );
        }
        "blade" => {
            blade!(
                "needle",
                v3(0.05, 0.02, 0.4),
                123.0,
                35.0,
                0.07,
                0.014,
                m("leaf"),
                droop = 0.15
            );
        }
        "blade_plain" => {
            blade!(
                "grass",
                (0.05, 0.02, 0.0),
                -20.0,
                80.0,
                0.3,
                0.02,
                m("leaf_light")
            );
        }
        "stalk" => {
            stalk!(
                "stem",
                (0.1, 0.2, 0.06),
                v3(0.13, 0.26, 0.45),
                m("stem"),
                r = 0.005
            );
        }
        "stalk_upright" => {
            stalk!(
                "stem",
                (0.1, 0.2, 0.0),
                (0.1, 0.2, 0.37),
                m("stem"),
                r = 0.0035
            );
        }
        "stalk_bowed" => {
            stalk!(
                "petiole",
                (0, 0, 0.06),
                v3(0.2, -0.15, 0.25),
                m("stem"),
                r = 0.004,
                bow = 0.02
            );
        }
        "stalk_plain" => {
            stalk!("stem", v3(0.3, 0.1, 0.02), (0.25, 0.14, 0.5), m("stem"));
        }
        "lupin_spike" => {
            lupin_spike(
                "spike",
                (0.21, -0.13, 0.9 - 0.36),
                0.36,
                0.045,
                m("blossom4"),
            );
        }
        "lupin_spike_plain" => {
            lupin_spike("spike", v3(0.0, 0.1, 0.2), 0.25, 0.03, m("blossom"));
        }
        "_flower" => _flower("flower2", v3(0.03, -0.1, 0.9), 180.0 + 12.0, 0.05, p),
        "_flower_small" => _flower("flower6", (0.05, -0.12, 1.1), 180.0 - 12.0, 0.04, p),
        "rose_head" => rose_head!(
            "rose",
            v3(0.2, 0.15, 0.7),
            0.05,
            m("bloom"),
            yaw = degrees(0.6435),
            tilt = 17.5
        ),
        "rose_head_plain" => rose_head!("rose", (0.2, 0.15, 0.7), 0.04, m("bloom")),
        "florets" => {
            florets!(
                "mophead",
                0.12,
                v3(0.3, 0.2, 0.6) + v3(0.5, 0.5, 0.7) * 0.12 * 0.4 - v3(0, 0, 0.12 * 0.7),
                m("blossom"),
                bumps = 3.2,
                amount = 0.16,
                flat = 0.82,
                segments = 10,
                rings = 7,
                seed = 2.0 * 1.3
            );
        }
        "florets_seeds" => {
            florets!(
                "seeds",
                0.088,
                (0, 0, -0.012),
                m("centre_dark"),
                bumps = 6.0,
                amount = 0.05,
                flat = 0.28,
                segments = low(14, 8),
                rings = low(6, 4),
                seed = 2
            );
        }
        "florets_pom" => {
            florets!(
                "pom",
                0.05,
                (0.1, -0.2, 0.23),
                m("blossom2"),
                bumps = 4.5,
                amount = 0.18,
                flat = 0.75,
                segments = 8,
                rings = 5,
                seed = 4.0 * 0.7
            );
        }
        "florets_ball" => {
            florets!(
                "ball",
                0.36,
                (0, 0, 0.04),
                m("leaf"),
                bumps = 5.0,
                amount = 0.035,
                flat = 0.94,
                segments = low(22, 12),
                rings = low(14, 8),
                seed = 1.0
            );
        }
        "florets_plume" => {
            florets!(
                "plume",
                0.032,
                (0, 0, 0),
                m("bloom"),
                bumps = 2.6,
                amount = 0.3,
                flat = 4.0,
                segments = 6,
                rings = 7,
                seed = 3.0 * 1.1
            );
        }
        "florets_seed" => {
            florets!(
                "seed",
                0.01,
                (0.1, 0.13, 0.41),
                m("leaf_light"),
                bumps = 2.0,
                amount = 0.25,
                flat = 4.5,
                segments = low(5, 3),
                rings = low(5, 3),
                seed = 6
            );
        }
        "florets_chive" => {
            florets!(
                "chive_flower",
                0.02,
                (0.2, 0.1, 0.3 + 0.2 - 0.01),
                m("bloom"),
                bumps = 3.0,
                amount = 0.25,
                segments = low(6, 4),
                rings = low(5, 3),
                seed = 2
            );
        }
        "florets_plain" => {
            florets!("ball", 0.1, v3(0.1, 0.1, 0.46), m("leaf_light"));
        }
        "vessel" => {
            return Some(vec![vessel!(
                p,
                0.19,
                0.32,
                r_bottom = 0.14,
                lip = 0.016,
                foot = 0.0,
                saucer = 0.035,
                segments = 18
            )]);
        }
        "vessel_square" => {
            return Some(vec![vessel!(
                p,
                0.18,
                0.33,
                r_bottom = 0.14,
                lip = 0.0,
                foot = 0.02,
                segments = 18
            )]);
        }
        "vessel_raised" => {
            return Some(vec![vessel!(
                p,
                0.145,
                0.24,
                z0 = 0.572,
                r_bottom = 0.11,
                lip = 0.0,
                foot = 0.015,
                segments = 16
            )]);
        }
        "vessel_dry" => {
            return Some(vec![vessel!(
                p,
                0.085,
                0.16,
                z0 = 0.63,
                r_bottom = 0.07,
                lip = 0.008,
                segments = 16,
                soil = false
            )]);
        }
        "vessel_low" => {
            return Some(vec![vessel!(
                p,
                0.19,
                0.33,
                r_bottom = 0.15,
                lip = 0.014,
                segments = low(18, 10)
            )]);
        }
        "vessel_small" => {
            return Some(vec![vessel!(
                p,
                0.05,
                0.09,
                r_bottom = 0.05 * 0.75,
                lip = 0.01,
                segments = 10,
                soil = false
            )]);
        }
        "vessel_plain" => return Some(vec![vessel!(p, 0.2, 0.3)]),
        "vessel_belly" => {
            return Some(vec![vessel!(
                p,
                0.2,
                0.3,
                belly = 0.03,
                material = m("wood"),
                saucer = 0.02,
                foot = 0.01,
                segments = 12
            )]);
        }
        "palette" => {}
        other => panic!("no case {other:?}"),
    }
    None
}

#[test]
fn the_plants_palette_is_the_scripts() {
    let all = cases("botany_flowers.txt");
    let case = all
        .iter()
        .find(|c| c.name == "palette")
        .expect("the palette");
    let written = |p: &Palette| -> Vec<String> {
        p.colours
            .iter()
            .map(|m| {
                format!(
                    "{}={}:{}",
                    m.name,
                    artkit::materials::hex_of(m.colour),
                    m.finish
                )
            })
            .collect()
    };
    assert_eq!(
        written(&botany::BASE.palette()),
        words(&case.lines["colours"])
    );
    assert_eq!(
        written(&botany::palette(&[])),
        words(&case.lines["colours"])
    );
    let own = botany::palette(&[
        ("body", "#C46A43"),
        ("leaf_light", "#7DB85C"),
        ("twig", "#102030"),
    ]);
    assert_eq!(written(&own), words(&case.lines["recoloured"]));
}

#[test]
fn stems_pots_and_flowers_are_blenders_at_full_detail_and_as_far_versions() {
    let mut off = Off::default();
    // (a cross-section's numbers are doubles straight from sines and cosines, which the kernel works out in software
    // and Blender's Python with the machine's library: a unit or two in the last place of a double apart)
    let hold = |name: &str| Hold {
        ret_ulps: if name == "ribbed" { 4 } else { 0 },
        ..Hold::default()
    };
    let n = check("botany_flowers.txt", flowers, hold, &mut off);
    assert!(n >= 140, "{n} cases");
    // (the normals in the kernel's own order of faces: a unit or two in the last place)
    assert!(off.normals < 2.5e-7, "{off:?}");
}

// ---- leaves cut from an outline ----

fn round_leaf(t: f64) -> f64 {
    pow(sin(PI * 1.0_f64.min(0.1 + t * 0.9)), 0.5)
}

fn ivy(t: f64) -> f64 {
    pow(sin(PI * t), 0.6) * (1.15 - 0.6 * t)
}

fn outlines() -> Vec<Vec<(f64, f64)>> {
    vec![
        outline!("oval", 0.12, n = 4, wave = 0.18, waves = 5),
        outline!(fiddle, 0.09, n = 5),
        outline!(fiddle, 0.21, n = 7, wave = 0.05, waves = 5),
        outline!(sword, 0.062, n = 6),
        outline!("oval", 0.095, n = 6),
        outline!(round_leaf, 0.12, n = 6, wave = 0.07, waves = 7),
        outline!("heart", 0.2, n = 7, wave = 0.06, waves = 6),
        outline!("oval", 0.045, n = 3),
        outline!("lance", 0.035, n = 3),
        outline!("heart", 0.17, n = 6),
        outline!(sword, 0.062 * 1.14, n = 6),
        outline!("lance", 0.065 * 1.1, n = 3),
        outline!("lance", 0.075, n = 4),
        outline!("strap", 0.032, n = 3),
        outline!("heart", 0.18 - 2.0 * 0.012, n = 6, teeth = 0.08),
        outline!("lance", 0.03, n = 3),
        outline!(ivy, 0.05, n = 3, wave = 0.2, waves = 3),
        outline!("oval", 0.05, n = 3),
        outline!(lobed!(5), 0.27, n = 10, teeth = 0.05),
        outline!(sword, 0.03 * 0.9, n = 3),
        outline!("oval", 0.14, n = 4, wave = 0.08, waves = 3),
        outline!("petal", 0.1),
        outline!(lobed!(3, depth = 0.5), 0.2, n = 8),
    ]
}

fn pinnates() -> Vec<Vec<(f64, f64)>> {
    vec![
        pinnate!(
            1,
            0.13 * 0.75,
            "oval",
            lean = 0.75,
            fill = 0.85,
            start = 0.1,
            round_tip = true
        ),
        pinnate!(
            2,
            0.17 * 0.75,
            "oval",
            lean = 0.75,
            fill = 0.85,
            start = 0.1,
            round_tip = true
        ),
        pinnate!(8, 0.14, "lance", lean = 0.5, fill = 0.75),
        pinnate!(6, 0.1, "lance", lean = 0.5, fill = 0.75),
        pinnate!(10, 0.26, "lance", lean = 0.85, fill = 0.3),
        pinnate!(3, 0.06, "lance", lean = 0.6, fill = 0.6),
        pinnate!(5, 0.16, "lance", lean = 0.8, fill = 0.25),
        pinnate!(12, 0.3, "lance", lean = 0.5, fill = 0.8),
        pinnate!(8, 0.2, "lance", lean = 0.5, fill = 0.8),
        pinnate!(7, 0.045, "strap", lean = 0.9, fill = 0.25),
        pinnate!(3, 0.1 * 0.93, "lance", lean = 0.6, fill = 0.7),
        pinnate!(3, 0.08, "lance", lean = 0.7, fill = 0.35),
        pinnate!(4, 0.1),
        pinnate!(3, 0.12, fiddle, stalk = 0.002),
    ]
}

fn leaves(name: &str, p: &Palette) -> Option<Ret> {
    let m = |k: &str| p.get(k);
    let polar = |r: f64, deg: f64, z: f64| V3::from(botany::polar(r, deg, z));
    match name {
        "outline" => return Some(outlines().iter().flat_map(|o| flat2(o)).collect()),
        "pinnate" => return Some(pinnates().iter().flat_map(|o| flat2(o)).collect()),
        "slits" => {
            return Some(
                [
                    slits!("heart", 0.34, 5, depth = 0.62, gap = 0.22),
                    slits!("oval", 0.2, 4),
                    slits!(fiddle, 0.3, 3, depth = 0.5),
                ]
                .iter()
                .flat_map(|o| flat2(o))
                .collect(),
            );
        }
        "shapes" => {
            let at =
                |f: &dyn Fn(f64) -> f64| (0..17).map(|t| f(t as f64 / 16.0)).collect::<Vec<_>>();
            return Some(
                [
                    at(&fiddle),
                    at(&sword),
                    at(&lobed!(5)),
                    at(&lobed!(3, depth = 0.5)),
                ]
                .concat(),
            );
        }
        "cut_leaf1" => {
            cut_leaf!(
                "top",
                (0, 0, 1.49),
                20.0,
                70.0,
                0.14,
                &outline!(fiddle, 0.09, n = 5),
                m("leaf_light"),
                droop = 0.1,
                ribs = 3
            );
        }
        "cut_leaf2" => {
            cut_leaf!(
                "leaf",
                v3(0.2, 0.1, 0.4),
                133.0,
                12.0,
                0.13,
                &pinnate!(
                    1,
                    0.13 * 0.75,
                    "oval",
                    lean = 0.75,
                    fill = 0.85,
                    start = 0.1,
                    round_tip = true
                ),
                m("leaf"),
                droop = 0.25,
                fold = 0.2,
                ribs = 3,
                thick = 0.003
            );
        }
        "cut_leaf3" => {
            cut_leaf!(
                "leaf",
                v3(0.2, 0.1, 0.4),
                133.0,
                12.0,
                0.17,
                &pinnate!(
                    2,
                    0.17 * 0.75,
                    "oval",
                    lean = 0.75,
                    fill = 0.85,
                    start = 0.1,
                    round_tip = true
                ),
                m("leaf"),
                droop = 0.35,
                fold = 0.2,
                ribs = 3,
                thick = 0.003
            );
        }
        "cut_leaf4" => {
            cut_leaf!(
                "leaf",
                v3(0.21, 0.13, 0.52),
                57.0,
                31.0,
                0.36,
                &slits!("heart", 0.34, 5, depth = 0.62, gap = 0.22),
                m("leaf_dark"),
                droop = 0.5,
                fold = 0.1,
                ribs = 5,
                thick = 0.006
            );
        }
        "cut_leaf5" => {
            cut_leaf!(
                "leaf",
                v3(0.1, -0.05, 1.1),
                211.0,
                24.0,
                0.3,
                &outline!(fiddle, 0.21, n = 7, wave = 0.05, waves = 5),
                m("leaf_light"),
                droop = 0.22,
                fold = 0.16,
                ribs = 5,
                thick = 0.008
            );
        }
        "cut_leaf6" => {
            cut_leaf!(
                "blade",
                polar(0.04, 80.0, 0.3),
                80.0,
                78.0,
                0.7,
                &outline!(sword, 0.062, n = 6),
                m("leaf"),
                fold = 0.3,
                ribs = 5,
                thick = 0.014,
                twist = 35.0
            );
        }
        "cut_leaf7" => {
            cut_leaf!(
                "frond",
                (0, 0, 0.33 + 0.02),
                44.0,
                38.0,
                0.5,
                &pinnate!(8, 0.14, "lance", lean = 0.5, fill = 0.75),
                m("leaf"),
                droop = 0.45,
                fold = 0.12,
                ribs = 4
            );
        }
        "cut_leaf8" => {
            cut_leaf!(
                "crown",
                (0, 0, 0.33 + 0.02),
                200.0,
                68.0,
                0.28,
                &pinnate!(6, 0.1, "lance", lean = 0.5, fill = 0.75),
                m("leaf_light"),
                droop = 0.5,
                fold = 0.12,
                ribs = 3
            );
        }
        "cut_leaf9" => {
            cut_leaf!(
                "frond",
                v3(0.05, 0.02, 0.8),
                290.0,
                40.0 - 18.0,
                0.62,
                &pinnate!(10, 0.26, "lance", lean = 0.85, fill = 0.3),
                m("leaf_dark"),
                droop = 0.38,
                fold = 0.1,
                ribs = 5
            );
        }
        "cut_leaf10" => {
            cut_leaf!(
                "leaf",
                (0, 0, 0.21 + 0.01),
                75.0,
                18.0,
                0.22,
                &outline!("oval", 0.095, n = 6),
                m("leaf"),
                droop = 0.35,
                fold = 0.22,
                ribs = 5,
                thick = 0.012
            );
        }
        "cut_leaf11" => {
            cut_leaf!(
                "leaf",
                (0.1 * 0.8, -0.2 * 0.8, 0.07 + 0.06),
                12.0,
                55.0,
                0.13,
                &pinnate!(3, 0.06, "lance", lean = 0.6, fill = 0.6),
                m("leaf_light"),
                droop = 0.3,
                ribs = 2
            );
        }
        "cut_leaf12" => {
            cut_leaf!(
                "frond",
                (0.1, -0.2, 0.07),
                99.0,
                41.0,
                0.31,
                &pinnate!(5, 0.16, "lance", lean = 0.8, fill = 0.25),
                m("leaf"),
                droop = 0.3,
                fold = 0.1,
                ribs = 3
            );
        }
        "cut_leaf13" => {
            cut_leaf!(
                "leaf",
                v3(0.15, 0.1, 0.25),
                310.0,
                33.0,
                0.11,
                &outline!(round_leaf, 0.12, n = 6, wave = 0.07, waves = 7),
                m("leaf_dark"),
                droop = 0.25,
                fold = 0.15,
                ribs = 3
            );
        }
        "cut_leaf14" => {
            cut_leaf!(
                "frond",
                (0, 0, 0.1),
                161.0,
                52.0,
                0.7,
                &pinnate!(12, 0.3, "lance", lean = 0.5, fill = 0.8),
                m("leaf"),
                droop = 0.42,
                fold = 0.12,
                ribs = 6,
                twist = -25.0
            );
        }
        "cut_leaf15" => {
            cut_leaf!(
                "inner",
                (0, 0, 0.14),
                161.0 + 20.0,
                70.0,
                0.4,
                &pinnate!(8, 0.2, "lance", lean = 0.5, fill = 0.8),
                m("leaf_light"),
                droop = 0.45,
                fold = 0.12,
                ribs = 4
            );
        }
        "cut_leaf16" => {
            cut_leaf!(
                "leaf",
                v3(0.2, -0.1, 0.4),
                25.0,
                30.0,
                0.26,
                &outline!("heart", 0.2, n = 7, wave = 0.06, waves = 6),
                m("leaf"),
                droop = 0.3,
                fold = 0.32,
                ribs = 6,
                thick = 0.006
            );
        }
        "cut_leaf17" => {
            cut_leaf!(
                "basil2_leaf",
                v3(-0.25, -0.1, 0.74) + polar(0.045, 200.0, 0.02),
                200.0,
                35.0,
                0.07,
                &outline!("oval", 0.045, n = 3),
                m("leaf_dark"),
                droop = 0.4,
                fold = 0.5,
                ribs = 2
            );
        }
        "cut_leaf18" => {
            cut_leaf!(
                "rosemary",
                v3(0.3, 0.05, 0.75),
                15.0,
                62.0,
                0.2,
                &pinnate!(7, 0.045, "strap", lean = 0.9, fill = 0.25),
                m("stem"),
                droop = 0.08,
                fold = 0.1,
                ribs = 3
            );
        }
        "cut_leaf19" => {
            cut_leaf!(
                "sage_leaf",
                v3(0.0, 0.1, 0.78),
                250.0,
                28.0,
                0.08,
                &outline!("lance", 0.035, n = 3),
                m("leaf_light"),
                droop = 0.2,
                fold = 0.2,
                ribs = 2,
                thick = 0.005
            );
        }
        "cut_leaf20" => {
            cut_leaf!(
                "leaf",
                polar(0.03, 59.0, 0.292)
                    + polar(0.17 * cos(radians(30.0)), 59.0, 0.17 * sin(radians(30.0))),
                59.0,
                0.0,
                0.22,
                &outline!("heart", 0.17, n = 6),
                m("leaf_light"),
                droop = 0.32,
                fold = 0.22,
                ribs = 4
            );
        }
        "cut_leaf21" => {
            cut_leaf!(
                "edge",
                (
                    0.04 - 0.0015 * sin(radians(80.0)),
                    0.01 - 0.0015 * cos(radians(80.0)),
                    0.3 - 0.02
                ),
                80.0,
                78.0,
                0.7 * 1.01,
                &outline!(sword, 0.062 * 1.14, n = 6),
                m("leaf_light"),
                fold = 0.3,
                ribs = 2,
                thick = 0.004,
                twist = 35.0
            );
        }
        "cut_leaf22" => {
            cut_leaf!(
                "rosette",
                (0.2, -0.1, 0.06),
                222.0,
                31.0,
                0.17 * 1.1,
                &outline!("lance", 0.065 * 1.1, n = 3),
                m("leaf_dark"),
                droop = 0.3,
                fold = 0.25,
                ribs = 2,
                thick = 0.004
            );
        }
        "cut_leaf23" => {
            cut_leaf!(
                "rosette",
                (0.2, -0.1, 0.06),
                222.0,
                31.0,
                0.22 * 0.93,
                &pinnate!(3, 0.1 * 0.93, "lance", lean = 0.6, fill = 0.7),
                m("leaf_dark"),
                droop = 0.3,
                fold = 0.25,
                ribs = 2,
                thick = 0.004
            );
        }
        "cut_leaf24" => {
            cut_leaf!(
                "leaf",
                (0.1, 0.1, 0.07 - 0.005),
                301.0,
                48.0,
                0.21,
                &outline!("lance", 0.075, n = 4),
                m("leaf"),
                droop = 0.32,
                fold = 0.35,
                ribs = 3,
                thick = 0.005
            );
        }
        "cut_leaf25" => {
            cut_leaf!(
                "leaf",
                (0.1, 0.1, 0.07),
                19.0,
                66.0,
                0.19,
                &outline!("strap", 0.032, n = 3),
                m("leaf_light"),
                droop = 0.3,
                fold = 0.3,
                ribs = 2,
                thick = 0.005
            );
        }
        "cut_leaf26" => {
            cut_leaf!(
                "leaf",
                v3(0.12, 0.2, 0.3),
                147.0,
                25.0,
                0.22 - 2.0 * 0.015,
                &outline!("heart", 0.18 - 2.0 * 0.012, n = 6, teeth = 0.08),
                m("leaf_dark"),
                droop = 0.42,
                fold = 0.18,
                ribs = 4
            );
        }
        "cut_leaf27" => {
            cut_leaf!(
                "leaf",
                v3(0.1, 0.2, 0).lerp(v3(0.13, 0.26, 0.45), 0.4),
                88.0,
                35.0,
                0.17,
                &outline!("lance", 0.03, n = 3),
                m("leaf"),
                droop = 0.3,
                ribs = 2,
                thick = 0.004
            );
        }
        "cut_leaf28" => {
            cut_leaf!(
                "ivy",
                v3(0.3, -0.24, 0.2),
                180.0 + 50.0,
                -10.0,
                0.05,
                &outline!(ivy, 0.05, n = 3, wave = 0.2, waves = 3),
                m("leaf_light"),
                droop = 0.2,
                fold = 0.2,
                ribs = 1
            );
        }
        "cut_leaf29" => {
            cut_leaf!(
                "basil_leaf",
                v3(0.1, 0.0, 0.2) + polar(0.05, 144.0, 0.04),
                144.0,
                25.0 + 20.0,
                0.075,
                &outline!("oval", 0.05, n = 3),
                m("leaf"),
                droop = 0.4,
                fold = 0.5,
                ribs = 2
            );
        }
        "cut_leaf30" => {
            cut_leaf!(
                "leaf",
                v3(0.3, 0.2, 0.12),
                233.0,
                21.0,
                0.24,
                &outline!(lobed!(5), 0.27, n = 10, teeth = 0.05),
                m("leaf"),
                droop = 0.22,
                fold = 0.3,
                ribs = 4,
                thick = 0.008
            );
        }
        "cut_leaf31" => {
            cut_leaf!(
                "spike",
                (0.1, -0.1, 0.1),
                300.0,
                62.0 + 14.0,
                0.1 * 0.9,
                &outline!(sword, 0.03 * 0.9, n = 3),
                m("flesh"),
                fold = 0.3,
                ribs = 2,
                thick = 0.012
            );
        }
        "cut_leaf32" => {
            cut_leaf!(
                "lettuce",
                (0.25, 0.27, 0.18 + 0.03),
                5.0 * 360.0 / 2.0 + 25.0 + 40.0,
                60.0,
                0.11,
                &outline!("oval", 0.12, n = 4, wave = 0.18, waves = 5),
                m("leaf"),
                droop = 0.35,
                fold = 0.4,
                ribs = 2
            );
        }
        "cut_leaf33" => {
            cut_leaf!(
                "cabbage",
                (0.0, 0.01, 0.18 + 0.01),
                (3 * 72 + 20) as f64,
                38.0,
                0.15,
                &outline!("oval", 0.14, n = 4, wave = 0.08, waves = 3),
                m("leaf_dark"),
                droop = 0.4,
                fold = 0.45,
                ribs = 2,
                thick = 0.005
            );
        }
        "cut_leaf34" => {
            cut_leaf!(
                "carrot_top",
                (-0.27, 0.02, 0.18),
                157.0,
                70.0 - 8.0,
                0.25,
                &pinnate!(3, 0.08, "lance", lean = 0.7, fill = 0.35),
                m("stem"),
                droop = 0.3,
                fold = 0.1,
                ribs = 2
            );
        }
        "cut_leaf35" => {
            cut_leaf!(
                "odd",
                (0, 0, 0),
                0.0,
                0.0,
                0.3,
                &outline!("oval", 0.1, n = 5),
                m("leaf"),
                left = outline!("lance", 0.16, n = 4),
                ribs = 7,
                twist = 20
            );
        }
        "cut_leaf36" => {
            cut_leaf!(
                "plain",
                (0.1, 0.2, 0.3),
                30.0,
                40.0,
                0.5,
                &outline!("oval", 0.2),
                m("leaf")
            );
        }
        "cut_leaf37" => {
            cut_leaf!(
                "bare",
                (0.1, 0.2, 0.3),
                30.0,
                40.0,
                0.5,
                &outline!("oval", 0.2, n = 1),
                m("leaf"),
                ribs = 0,
                thick = 0.0
            );
        }
        "cut_leaf38" => {
            cut_leaf!(
                "split",
                (0, 0, 0.2),
                10.0,
                20.0,
                0.4,
                &slits!("oval", 0.2, 4),
                m("leaf"),
                droop = 0.2,
                ribs = 9
            );
        }
        "rose_leaf" => {
            rose_leaf!(
                "leaves",
                v3(0.05, 0.1, 0.3),
                111.0,
                17.0,
                0.13,
                m("leaf_dark")
            );
        }
        "rose_leaf_two" => {
            rose_leaf!(
                "leaf",
                (0.2, 0.25, 0.4),
                301.0,
                36.0,
                0.17,
                m("leaf"),
                pairs = 2,
                droop = 0.35
            );
        }
        "random" => {}
        other => panic!("no case {other:?}"),
    }
    None
}

/// 400 leaves of random outlines, every other one a far version: each as one number of its points, its faces in
/// their order and its sharp edges, and how many it has of each.
fn random_leaves() -> (Vec<String>, Vec<String>) {
    let p = botany::palette(&[]);
    let mut rng = Random::new(2026);
    let (mut hashes, mut sizes) = (Vec::new(), Vec::new());
    for i in 0..400 {
        scene::reset();
        let kind = rng.randrange(4);
        let shape = ["oval", "lance", "heart", "strap", "petal"][rng.randrange(5) as usize];
        let width = rng.uniform(0.02, 0.3);
        let mut make = || {
            let edge = match kind {
                0 => {
                    let n = rng.randrange_step(1, 12, 1) as usize;
                    let wave = rng.uniform(0.0, 0.3);
                    let waves = rng.randrange(8) as f64;
                    let teeth = rng.choice(&[0.0, 0.05, 0.1]);
                    outline!(
                        shape,
                        width,
                        n = n,
                        wave = wave,
                        waves = waves,
                        teeth = teeth
                    )
                }
                1 => {
                    let pairs = rng.randrange_step(1, 14, 1) as usize;
                    let lean = rng.uniform(0.3, 1.0);
                    let fill = rng.uniform(0.2, 0.9);
                    let round_tip = rng.random() < 0.5;
                    pinnate!(
                        pairs,
                        width,
                        shape,
                        lean = lean,
                        fill = fill,
                        round_tip = round_tip
                    )
                }
                2 => {
                    let n = rng.randrange_step(1, 8, 1) as usize;
                    let depth = rng.uniform(0.2, 0.9);
                    let gap = rng.uniform(0.1, 0.5);
                    slits!(shape, width, n, depth = depth, gap = gap)
                }
                _ => outline!(fiddle, width, n = rng.randrange_step(2, 10, 1) as usize),
            };
            let base = (
                rng.uniform(-1.0, 1.0),
                rng.uniform(-1.0, 1.0),
                rng.uniform(0.0, 2.0),
            );
            let (yaw, tilt, length) = (
                rng.uniform(0.0, 360.0),
                rng.uniform(-20.0, 90.0),
                rng.uniform(0.03, 0.8),
            );
            cut_leaf!(
                "leaf",
                base,
                yaw,
                tilt,
                length,
                &edge,
                p.get("leaf"),
                droop = rng.uniform(0.0, 0.6),
                fold = rng.uniform(0.0, 0.5),
                thick = rng.uniform(0.002, 0.014),
                ribs = rng.randrange(9) as usize,
                twist = rng.uniform(-40.0, 40.0)
            )
        };
        let obj = if i % 2 != 0 { far_of(make) } else { make() };
        let mesh = obj.read(|o| o.mesh.clone());
        let mut words: Vec<u32> = mesh
            .verts
            .iter()
            .flat_map(|v| v.array().map(f32::to_bits))
            .collect();
        words.extend(&mesh.loops);
        words.extend(mesh.sharp.iter().flat_map(|&(a, b)| [a, b]));
        let mut h = 0xcbf2_9ce4_8422_2325_u64;
        for b in words.iter().flat_map(|w| w.to_le_bytes()) {
            h = (h ^ b as u64).wrapping_mul(0x0100_0000_01b3);
        }
        hashes.push(format!("{h:016x}"));
        sizes.push(format!(
            "{},{},{}",
            mesh.verts.len(),
            mesh.faces.len(),
            mesh.sharp.len()
        ));
    }
    (hashes, sizes)
}

#[test]
fn four_hundred_random_leaves_are_blenders_too() {
    scene::set_finishes(finishes());
    let all = cases("botany_leaves.txt");
    let case = all
        .iter()
        .find(|c| c.name == "random")
        .expect("the random leaves");
    let (hashes, sizes) = random_leaves();
    let (want, want_sizes) = (words(&case.lines["hashes"]), words(&case.lines["sizes"]));
    let wrong: Vec<usize> = (0..hashes.len())
        .filter(|&i| hashes[i] != want[i] || sizes[i] != want_sizes[i])
        .collect();
    assert!(
        wrong.is_empty() && hashes.len() == want.len(),
        "{} of {} leaves aren't Blender's: {:?}",
        wrong.len(),
        want.len(),
        &wrong[..wrong.len().min(12)]
    );
}

#[test]
fn leaves_cut_from_an_outline_are_blenders_triangle_for_triangle() {
    let mut off = Off::default();
    // (a cut leaf's faces are in Blender's order too: it makes them one by one)
    // (and an outline's numbers are doubles straight from powers and sines, which the kernel works out in software
    // and Blender's Python with the machine's library: a few units in the last place of a double apart; the leaves
    // cut from them are single precision, and Blender's to the bit)
    let hold = |name: &str| Hold {
        ordered: name.starts_with("cut_leaf") || name.starts_with("rose_leaf"),
        ret_ulps: 4,
        ..Hold::default()
    };
    let n = check("botany_leaves.txt", leaves, hold, &mut off);
    assert!(n >= 80, "{n} cases");
    assert!(off.normals < 2.5e-7, "{off:?}");
}

// ---- shrubs and hedges ----

fn masses3() -> Vec<ShrubMass> {
    vec![
        (v3(0.3, 0.1, 0.5), 0.2, 0.7),
        (v3(-0.2, 0.25, 0.6), 0.17, 0.75),
        (v3(0.05, -0.3, 0.45), 0.22, 0.68),
    ]
}

fn flat_spots(spots: &[(V3, V3, f64)]) -> Vec<f64> {
    spots
        .iter()
        .flat_map(|&(at, d, s)| [flat3(&[at, d]), vec![s]].concat())
        .collect()
}

fn flat_triples(points: &[(f64, f64, f64)]) -> Vec<f64> {
    points.iter().flat_map(|&(a, b, c)| [a, b, c]).collect()
}

fn greens(p: &Palette, names: [&str; 5]) -> [artkit::materials::Mat; 5] {
    names.map(|k| p.get(k))
}

/// families/plants.py's flowering shrub: the round shrub hung with sprays of five-petalled flowers.
fn bush_flowering() {
    let p = botany::palette(&[
        ("blossom", "#F08CBC"),
        ("blossom2", "#F8B4D4"),
        ("blossom3", "#FFF0F6"),
        ("blossom4", "#E870A8"),
    ]);
    let cols = ["blossom", "blossom2", "blossom3", "blossom4"].map(|k| p.get(k));
    let blooms = |masses: &[ShrubMass], rng: &mut Random| {
        for (i, (at, d, _)) in on_masses!(masses, rng, 26, up = (-0.3, 1.2))
            .into_iter()
            .enumerate()
        {
            let (yaw, tilt) = (
                90.0 - degrees(atan2(d.y as f64, d.x as f64)),
                90.0 - degrees(asin((-1.0_f64).max(1.0_f64.min(d.z as f64)))),
            );
            for j in 0..3 {
                let off = v3(
                    rng.uniform(-1.0, 1.0),
                    rng.uniform(-1.0, 1.0),
                    rng.uniform(-0.5, 0.5),
                ) * 0.05;
                if is_low() && j != 0 {
                    continue; // a far version: one flower a spray
                }
                rays!(
                    format!("blossom{i}_{j}"),
                    at + off,
                    0.045,
                    5,
                    cols[(i + j) % 4],
                    yaw = yaw,
                    tilt = tilt * 0.8,
                    inner = 0.35,
                    cup = 0.25,
                    width = 0.95,
                    centre = 0.006
                );
            }
        }
    };
    let leafy = greens(&p, ["leaf_dark", "leaf_dark", "leaf", "stem", "leaf_light"]);
    leafy_shrub!(
        33,
        1.208,
        1.016,
        &leafy,
        p.get("twig"),
        name = "bush_flowering",
        depth = 1.268,
        masses = 10,
        sats = 1,
        leaves = 8,
        blooms = blooms
    );
    let still = join_all("bushFlowering");
    perches!(6, 133, &[(0.0, 0.0, 0.5)]);
    keep_found();
    settle(&[still]);
}

/// families/plants.py's rose bush: canes from a leafy base, compound leaves along them, a rose on each, the one at
/// the front cut by a prune.
fn rosebush() {
    let p = botany::palette(&[
        ("leaf", "#2C4C26"),
        ("leaf_dark", "#365A2C"),
        ("leaf_light", "#3F6632"),
        ("stem", "#4A7038"),
        ("bloom", "#C8203A"),
    ]);
    let mut rng = Random::new(73);
    let leafy = [p.get("leaf"), p.get("leaf_dark"), p.get("leaf_light")];
    for (i, (x, y, _)) in spiral!(3, 0.16, &mut rng).into_iter().enumerate() {
        lump!(
            format!("base{i}"),
            0.18,
            (x, y, 0),
            leafy[i % 3],
            scale = (1, 1, 0.7),
            seed = i as f64 * 1.3 + rng.random(),
            noise = 0.32,
            segments = 8,
            rings = 5
        );
    }
    let mut canes = Vec::new();
    for (x, y, a) in spiral!(9, 0.3, &mut rng) {
        let h = rng.uniform(0.6, 0.84);
        let (foot, tip) = (v3(x * 0.3, y * 0.3, 0.05), v3(x * 1.3, y * 1.3, h));
        let bend = v3(x * 1.05, y * 1.05, h * 0.6);
        canes.push((foot, bend, tip, a, rng.uniform(8.0, 28.0)));
    }
    // the cane nearest the front (-y)
    let front = (0..canes.len())
        .min_by(|&i, &j| canes[i].2.y.total_cmp(&canes[j].2.y))
        .unwrap();
    for (i, &(foot, bend, tip, a, tilt)) in canes.iter().enumerate() {
        twig!(
            format!("cane{i}"),
            &[foot, bend, tip - v3(0, 0, 0.02)],
            0.01,
            0.005,
            p.get("stem"),
            verts = 4
        );
        // leaves along the cane, turning round it
        for (k, f) in [0.35, 0.62, 0.86].into_iter().enumerate() {
            let at = if f < 0.6 {
                foot.lerp(bend, f / 0.6)
            } else {
                bend.lerp(tip, (f - 0.6) / 0.4)
            };
            let heading = degrees(a) + (k as f64 * 2.0 - 1.0) * 70.0 + rng.uniform(-20.0, 20.0);
            let tilt_k = rng.uniform(5.0, 30.0);
            if is_low() && k == 1 {
                continue; // a far version: the outer two
            }
            rose_leaf!(
                format!("leaves{i}_{k}"),
                at,
                heading,
                tilt_k,
                0.13,
                leafy[(i + k) % 3]
            );
        }
        if i != front {
            rose_head!(
                &format!("rose{i}"),
                tip,
                0.05,
                p.get("bloom"),
                yaw = degrees(a),
                tilt = tilt
            );
        }
    }
    // perches: on the canes (half way up their arch) and the leafy base
    let bare: Vec<V3> = canes
        .iter()
        .map(|&(_, bend, tip, _, _)| bend.lerp(tip, 0.45) + v3(0, 0, 0.012))
        .collect();
    perches!(5, 140, &[(0.0, 0.0, 0.3)], per = 8, bare = bare);
    keep_found();
    let (_, _, tip, a, tilt) = canes[front];
    _with_part(
        "rose",
        join_all("rosebush"),
        || {
            rose_head!(
                "cut_rose",
                tip,
                0.05,
                p.get("bloom"),
                yaw = degrees(a),
                tilt = tilt
            )
        },
        tip,
        "cut",
    );
}

/// The models built whole, fitted and joined. (The street hedge and `hedge_block` are no longer here: fa481d5d3
/// rebuilt them as leaf clumps, which Blender's goldens never had.)
const MODELS: [&str; 5] = [
    "bush_flowering",
    "rosebush",
    "tulips",
    "veg_patch",
    "meadow",
];

fn shrubs(name: &str, p: &Palette) -> Option<Ret> {
    let m = |k: &str| p.get(k);
    match name {
        "hedge_top" => {
            return Some(
                [
                    flat_triples(&hedge_top(0.915)),
                    flat_triples(&hedge_top(1.12)),
                ]
                .concat(),
            );
        }
        "on_masses" => {
            return Some(
                [
                    flat_spots(&on_masses!(
                        &masses3(),
                        &mut Random::new(5),
                        7,
                        up = (-0.3, 1.2)
                    )),
                    flat_spots(&on_masses!(&masses3(), &mut Random::new(6), 2)),
                ]
                .concat(),
            );
        }
        "spiral" => {
            return Some(
                [
                    flat_triples(&spiral!(7, 0.32, &mut Random::new(51))),
                    flat_triples(&spiral!(48, 0.36, &mut Random::new(62))),
                    flat_triples(&spiral!(5, 0.2, &mut Random::new(1), jitter = 0.6)),
                ]
                .concat(),
            );
        }
        "fit_to" => {
            cube!(
                "box",
                (0.3, 0.2, 0.1),
                at = (0.1, -0.2, 0.05),
                material = m("body"),
                bevel = 0
            );
            sphere!(
                "pad",
                0.5,
                at = (0.2, 0.1, 0.36),
                material = m("flesh_light"),
                segments = 8,
                rings = 5,
                scale = (0.11, 0.032, 0.14),
                rot = (0, 8, 0)
            );
            sphere!(
                "pad2",
                0.5,
                at = (-0.1, 0.1, 0.4),
                material = m("flesh_light"),
                segments = 6,
                rings = 4,
                scale = (0.08, 0.032, 0.1),
                rot = (0, -28, 0)
            );
            twig!(
                "stem",
                &[(0.0, 0.0, 0.0), (0.05, 0.02, 0.3), (0.1, 0.0, 0.5)],
                0.02,
                0.01,
                m("stem")
            );
            leaf!("leaf", (0.1, 0, 0.5), 30.0, 40.0, 0.3, 0.1, m("leaf"));
            cone!(
                "young",
                0.012,
                0.002,
                0.14,
                at = (0, 0.1, 0.2),
                material = m("leaf_light"),
                verts = 5,
                rot = (20, 0, 35)
            );
            return Some(fit_to(&scene::meshes(), 0.568, 0.521, 0.465).to_vec());
        }
        "fit_to_some" => {
            cube!(
                "box",
                (0.3, 0.2, 0.1),
                at = (0.1, -0.2, 0.05),
                material = m("body"),
                bevel = 0
            );
            let crown = new_parts(|| {
                lump!(
                    "crown",
                    0.3,
                    (0.02, 0.01, 0.4),
                    m("leaf"),
                    scale = (1, 1.1, 1.6),
                    seed = 2.0,
                    segments = 6,
                    rings = 5
                );
                twig!(
                    "twig",
                    &[(0.0, 0.0, 0.3), (0.3, 0.02, 0.5)],
                    0.02,
                    0.01,
                    m("stem"),
                    verts = 3,
                    res = 1
                );
            });
            let k = fit_to(&crown, 0.62, None, 0.65);
            return Some([k, fit_to(&scene::meshes(), None, 0.9, None)].concat());
        }
        "dots" => dots!(
            &mut Random::new(3),
            6,
            (0.1, 0.0, 0.5),
            (0.5, 0.45, 0.4),
            0.09,
            &[m("bloom"), m("bloom2")],
            "dot",
            far_every = 3
        ),
        "dots_plain" => dots!(
            &mut Random::new(4),
            4,
            (0.0, 0.0, 0.4),
            (0.3, 0.3, 0.3),
            0.06,
            &[m("blossom")],
            "bloom"
        ),
        "leafy_shrub_plain" => {
            let mut seen = Vec::new();
            let out = leafy_shrub!(
                7,
                0.9,
                0.8,
                &[m("leaf_dark"), m("leaf"), m("leaf_light")],
                m("twig"),
                leaf_cols = vec![m("stem"), m("leaf_light")],
                masses = 5,
                stems = 3,
                sats = 2,
                leaves = 12,
                blooms = |masses: &[ShrubMass], rng: &mut Random| {
                    seen.extend([rng.random(), masses.len() as f64]);
                }
            );
            let mut ret: Vec<f64> = out
                .iter()
                .flat_map(|&(at, r)| [flat3(&[at]), vec![r]].concat())
                .collect();
            ret.extend(seen);
            return Some(ret);
        }
        "bush_flowering" => bush_flowering(),
        "rosebush" => rosebush(),
        other => panic!("no case {other:?}"),
    }
    None
}

#[test]
fn shrubs_and_hedges_are_blenders_fitted_joined_and_settled_with_their_perches() {
    let mut off = Off::default();
    let hold = |name: &str| Hold {
        ret_ulps: if ["on_masses", "spiral", "hedge_top"].contains(&name) {
            4
        } else {
            0
        },
        turn_ulps: if name.starts_with("fit_to") { 1 } else { 0 },
        metres: if MODELS.contains(&name) { JOINED } else { 0.0 },
        size: JOINED as f64,
        ..Hold::default()
    };
    let n = check("botany_shrubs.txt", shrubs, hold, &mut off);
    assert!(n >= 20, "{n} cases");
    assert!(off.moved * 4 < off.points, "{off:?}");
}

// ---- beds and crops ----

/// families/plants.py's tulips: a soil mound of tulips on bowed stems, broad leaves at their feet.
fn tulips() {
    let p = botany::palette(&[("leaf", "#5E8A5A"), ("leaf_light", "#6E9A66")]);
    let mut rng = Random::new(51);
    let top = mound!(&p);
    let cols = ["bloom", "bloom2", "bloom3", "bloom4", "bloom", "bloom5"].map(|k| p.get(k));
    for (i, (x, y, a)) in spiral!(17, 0.34, &mut rng).into_iter().enumerate() {
        let h = rng.uniform(0.42, 0.55);
        let tip = (
            x + rng.uniform(-0.03, 0.03),
            y + rng.uniform(-0.03, 0.03),
            h,
        );
        stalk!(
            format!("stem{i}"),
            (x, y, top - 0.01),
            tip,
            p.get("stem"),
            r = 0.005
        );
        tulip_head!(
            format!("tulip{i}"),
            V3::from(tip) - v3(0, 0, 0.01),
            0.032,
            0.075,
            cols[i % cols.len()],
            yaw = rng.uniform(0.0, 360.0),
            tilt = rng.uniform(0.0, 10.0)
        );
        for k in 0..2 {
            let (yaw, t, length) = (
                a + 40.0 + k as f64 * 160.0,
                rng.uniform(55.0, 70.0),
                rng.uniform(0.2, 0.26),
            );
            if k != 0 && i % 2 != 0 {
                continue;
            }
            cut_leaf!(
                format!("leaf{i}_{k}"),
                (x, y, top - 0.005),
                yaw,
                t,
                length,
                &outline!("lance", 0.075, n = 4),
                [p.get("leaf"), p.get("leaf_light")][k],
                droop = 0.32,
                fold = 0.35,
                ribs = 3,
                thick = 0.005
            );
        }
    }
    fit_to(&scene::meshes(), 0.84, 0.622, 0.843);
    settle(&[join_all("tulips")]);
}

/// families/plants.py's vegetable patch: a raised bed of lettuces, cabbages and carrot tops, grown from seedlings.
fn veg_patch() {
    let p = botany::palette(&[
        ("body", "#8A6446"),
        ("leaf", "#9CC45A"),
        ("leaf_light", "#8AB44E"),
        ("leaf_dark", "#7EA48A"),
        ("stem", "#5E9A3A"),
        ("bloom", "#A8C8A0"),
        ("bloom4", "#E8782A"),
    ]);
    let mut rng = Random::new(81);
    let top = raised_bed(&p, 0.92, 0.92, 0.18);
    // (the script's boards were sharp and flat: botany's are eased now, the gate's `edges`)
    for part in scene::meshes() {
        if part.name().starts_with("board") {
            part.edit(|o| {
                o.bevel = None;
                o.mesh.shade_flat();
                o.mesh.sharp.clear();
            });
        }
    }
    let still = join_all("vegPatch");
    let mut spots: Vec<Vec<(f64, f64)>> = Vec::new();
    for x in [-0.28, 0.0, 0.28] {
        let mut row = Vec::new();
        for y in [0.27, 0.0, -0.27] {
            row.push((x + rng.uniform(-0.03, 0.03), y + rng.uniform(-0.03, 0.03)));
        }
        spots.push(row);
    }
    let mut tops: Vec<Vec<(f64, f64)>> = Vec::new();
    for _ in 0..3 {
        let mut row = Vec::new();
        for j in 0..3 {
            row.push((
                rng.uniform(0.22, 0.28),
                j as f64 * 120.0 + rng.uniform(0.0, 40.0),
            ));
        }
        tops.push(row);
    }
    let ruffled = outline!("oval", 0.12, n = 4, wave = 0.18, waves = 5);
    let leaves = || {
        // lettuces: rosettes of ruffled leaves, cupped up
        for (k, &(x, y)) in spots[0].iter().enumerate() {
            lump!(
                format!("lettuce_heart{k}"),
                0.045,
                (x, y, top - 0.01),
                p.get("leaf_light"),
                scale = (1, 1, 1.1),
                noise = 0.3,
                seed = k as f64,
                segments = 6,
                rings = 4
            );
            for j in 0..6 {
                let inner = j >= 4;
                cut_leaf!(
                    format!("lettuce{k}_{j}"),
                    (x, y, top + if inner { 0.03 } else { 0.01 }),
                    (j * 360) as f64 / if inner { 2.0 } else { 4.0 }
                        + (k * 25) as f64
                        + if inner { 40.0 } else { 0.0 },
                    if inner { 60.0 } else { 30.0 },
                    if inner { 0.11 } else { 0.13 },
                    &ruffled,
                    [p.get("leaf"), p.get("leaf_light")][(j + k) % 2],
                    droop = 0.35,
                    fold = 0.4,
                    ribs = 2
                );
            }
        }
        // cabbages' outer leaves
        for (k, &(x, y)) in spots[1].iter().enumerate() {
            for j in 0..5 {
                cut_leaf!(
                    format!("cabbage{k}_{j}"),
                    (x, y, top + 0.01),
                    (j * 72 + k * 20) as f64,
                    38.0,
                    0.15,
                    &outline!("oval", 0.14, n = 4, wave = 0.08, waves = 3),
                    p.get("leaf_dark"),
                    droop = 0.4,
                    fold = 0.45,
                    ribs = 2,
                    thick = 0.005
                );
            }
        }
        // carrot tops: feathery fronds
        for (k, &(x, y)) in spots[2].iter().enumerate() {
            for (j, &(length, yaw)) in tops[k].iter().enumerate() {
                cut_leaf!(
                    format!("carrot_top{k}_{j}"),
                    (x, y, top),
                    yaw,
                    70.0 - j as f64 * 8.0,
                    length,
                    &pinnate!(3, 0.08, "lance", lean = 0.7, fill = 0.35),
                    p.get("stem"),
                    droop = 0.3,
                    fold = 0.1,
                    ribs = 2
                );
            }
        }
    };
    let crop = || {
        // cabbage hearts
        for (k, &(x, y)) in spots[1].iter().enumerate() {
            lump!(
                format!("heart{k}"),
                0.07,
                (x, y, top + 0.04),
                p.get("bloom"),
                scale = (1, 1, 0.85),
                noise = 0.1,
                seed = (k + 6) as f64,
                segments = low(9, 6),
                rings = low(6, 4)
            );
        }
        // carrot shoulders
        for (k, &(x, y)) in spots[2].iter().enumerate() {
            cone!(
                format!("carrot{k}"),
                0.022,
                0.016,
                0.03,
                at = (x, y, top + 0.01),
                material = p.get("bloom4"),
                verts = low(8, 5)
            );
        }
    };
    // picked, the bed is bare
    _crop!(
        still,
        (0, 0, top),
        leaves,
        crop,
        harvested = true,
        size = (0.92, 0.398, 0.92)
    );
}

/// families/plants.py's wildflower meadow: tussocks, long grass and flowers, one of them picked.
fn meadow() {
    let p = botany::palette(&[
        ("leaf", "#7FA04A"),
        ("leaf_dark", "#6E9240"),
        ("leaf_light", "#A2B45E"),
        ("stem", "#8EAE52"),
        ("bloom", "#EC3A2E"),
        ("bloom2", "#F6C21E"),
        ("bloom3", "#A880E0"),
        ("bloom4", "#8CB4F8"),
        ("bloom5", "#FFFFFF"),
        ("centre", "#E8B436"),
    ]);
    let mut rng = Random::new(62);
    let grass = ["leaf", "stem", "leaf_dark", "leaf_light"].map(|k| p.get(k));
    // low tussocks at the grass's foot
    for (i, (x, y, _)) in spiral!(3, 0.3, &mut rng).into_iter().enumerate() {
        lump!(
            format!("tussock{i}"),
            0.12,
            (x, y, 0),
            grass[i % 4],
            scale = (1, 1, 0.4),
            seed = i as f64 * 1.1,
            segments = 8,
            rings = 5
        );
    }
    // long grass, and stalks gone to seed; a far version builds every third blade (drawing every random number)
    for (i, (x, y, a)) in spiral!(48, 0.36, &mut rng).into_iter().enumerate() {
        let length = rng.uniform(0.28, 0.48);
        let tilt = rng.uniform(70.0, 87.0);
        let yaw = a + rng.uniform(-50.0, 50.0);
        let lean = rng.uniform(-0.06, 0.06);
        if is_low() && i % 3 != 0 {
            continue;
        }
        leaf!(
            format!("grass{i}"),
            (x * 0.85, y * 0.85, 0),
            yaw,
            tilt,
            length,
            0.022,
            grass[i % 4],
            shape = "strap",
            droop = 0.24,
            thick = 0.004,
            fold = 0.35,
            rows = 2,
            ring = 3
        );
        if i % 6 == 0 {
            let head = (x * 0.8 + lean, y * 0.8 - lean, length * 1.02);
            stem!(
                format!("seed_stalk{i}"),
                (x * 0.85, y * 0.85, 0),
                head,
                p.get("leaf_light"),
                r = 0.003
            );
            florets!(
                format!("seed{i}"),
                0.01,
                head,
                p.get("leaf_light"),
                bumps = 2.0,
                amount = 0.25,
                flat = 4.5,
                segments = low(5, 3),
                rings = low(5, 3),
                seed = i as f64
            );
        }
    }
    for (i, (x, y, _)) in spiral!(14, 0.4, &mut rng).into_iter().enumerate() {
        let h = rng.uniform(0.26, 0.48);
        let (yaw, tilt) = (rng.uniform(0.0, 360.0), rng.uniform(0.0, 30.0));
        stalk!(
            format!("stem{i}"),
            (x, y, 0),
            (x, y, h),
            p.get("stem"),
            r = 0.0035
        );
        rays!(
            format!("flower{i}"),
            (x, y, h),
            0.04,
            5 + i % 3,
            p.get(MEADOW_FLOWERS[i % 5]),
            yaw = yaw,
            tilt = tilt,
            inner = 0.3,
            cup = 0.12,
            width = 0.85,
            centre = 0.005
        );
    }
    let (x, y, h) = (0.06, -0.22, 0.48);
    // the one picked: at the front, standing tall
    let picked = || {
        stem!(
            "picked_stem",
            (x, y, 0),
            (x, y, h),
            p.get("stalk"),
            r = 0.005
        );
        rays!(
            "picked_flower",
            (x, y, h),
            0.06,
            12,
            p.get("bloom5"),
            yaw = 180,
            tilt = 30,
            inner = 0.3,
            cup = 0.08,
            width = 0.75
        );
        let eye = sphere!(
            "picked_eye",
            1.0,
            material = p.get("centre"),
            segments = low(6, 4),
            rings = 3,
            scale = (0.018, 0.018, 0.011)
        );
        _put!(eye, (x, y, h), yaw = 180, tilt = 30, lift = 0.005);
    };
    _fit_with_part(
        "picked",
        "meadow",
        picked,
        (x, y, 0),
        "pick",
        (0.872, 0.575, 0.91),
    );
}

fn beds(name: &str, p: &Palette) -> Option<Ret> {
    let m = |k: &str| p.get(k);
    let greens = [m("leaf"), m("leaf_dark"), m("leaf_light")];
    match name {
        "mound" => return Some(vec![mound!(p)]),
        "mound_small" => return Some(vec![mound!(p, r = 0.4)]),
        "mound_low" => return Some(vec![mound!(p, r = 0.3, h = 0.05)]),
        "clumps" => botany::clumps(
            &mut Random::new(9),
            7,
            0.33,
            0.27,
            0.17,
            0.07 + 0.07,
            &greens,
        ),
        "rosettes" => rosettes!(
            &mut Random::new(11),
            3,
            0.3,
            0.07 - 0.01,
            &greens,
            leaves = 4,
            length = 0.17,
            width = 0.065,
            tilt = (15.0, 35.0)
        ),
        "rosettes_pinnae" => rosettes!(
            &mut Random::new(12),
            3,
            0.3,
            0.07 - 0.01,
            &[m("leaf"), m("leaf_light")],
            leaves = 3,
            length = 0.22,
            width = 0.1,
            tilt = (20.0, 40.0),
            pinnae = 3
        ),
        "rosettes_oval" => rosettes!(
            &mut Random::new(13),
            3,
            0.34,
            0.07 - 0.01,
            &greens,
            leaves = 5,
            length = 0.13,
            width = 0.08,
            shape = "oval",
            tilt = (10.0, 30.0)
        ),
        "rosettes_plain" => rosettes!(
            &mut Random::new(14),
            3,
            0.2,
            0.05,
            &[m("leaf")],
            name = "tuft",
            droop = 0.1
        ),
        "raised_bed" => return Some(vec![raised_bed(p, 0.92, 0.92, 0.18)]),
        "tulips" => tulips(),
        "veg_patch" => veg_patch(),
        "meadow" => meadow(),
        "_crop" => {
            let top = mound!(p, r = 0.3);
            let still = join_all("patch");
            let leaves = || {
                for (i, a) in [20.0, 140.0, 260.0].into_iter().enumerate() {
                    cut_leaf!(
                        format!("leaf{i}"),
                        botany::polar(0.1, a, top),
                        a,
                        40.0,
                        0.2,
                        &outline!("oval", 0.1, n = 4),
                        m("leaf"),
                        droop = 0.3,
                        ribs = 2
                    );
                }
            };
            let crop = || {
                lump!(
                    "berry",
                    0.03,
                    (0.05, 0.02, top + 0.02),
                    m("bloom3"),
                    seed = 1.0,
                    segments = 6,
                    rings = 4
                );
            };
            _crop!(still, (0, 0, 0.05), leaves, crop, sink = (0.0, 0.04, 0.12));
        }
        other => panic!("no case {other:?}"),
    }
    None
}

#[test]
fn beds_and_crops_are_blenders_with_their_growth_clips() {
    let mut off = Off::default();
    let hold = |name: &str| Hold {
        ordered: name.starts_with("rosettes"),
        metres: if MODELS.contains(&name) || name == "_crop" {
            JOINED
        } else {
            0.0
        },
        size: JOINED as f64,
        ..Hold::default()
    };
    let n = check("botany_beds.txt", beds, hold, &mut off);
    assert!(n >= 26, "{n} cases");
    assert!(off.moved * 2 < off.points, "{off:?}");
}

// ---- the macros' order ----

#[test]
fn a_macro_works_out_what_was_given_by_place_before_what_was_given_by_name() {
    scene::set_finishes(finishes());
    scene::reset();
    let p = botany::palette(&[]);
    let order = RefCell::new(Vec::new());
    // a number that says when it was worked out
    let at = |what: &'static str, x: f64| {
        order.borrow_mut().push(what);
        x
    };
    let took = |what: &str| {
        let got = order.borrow().join(" ");
        order.borrow_mut().clear();
        assert_eq!(got, "place name", "{what}!");
    };
    lump!(
        "a",
        at("place", 0.1),
        (0, 0, 0),
        p.get("leaf"),
        seed = at("name", 1.0)
    );
    took("lump");
    artkit::leaves!(
        "b",
        at("place", 0.1),
        (0, 0, 0),
        p.get("leaf"),
        seed = at("name", 1.0)
    );
    took("leaves");
    leaf!(
        "c",
        (0, 0, 0),
        0.0,
        at("place", 10.0),
        0.2,
        0.1,
        p.get("leaf"),
        droop = at("name", 0.1)
    );
    took("leaf");
    artkit::tube!(
        "d",
        &[(0.0, 0.0, 0.0), (0.0, 0.0, 0.1)],
        at("place", 0.01),
        0.01,
        p.get("stem"),
        verts = at("name", 3.0) as usize
    );
    took("tube");
    artkit::sprigs!(
        "e",
        &p,
        (0, 0, 0),
        at("place", 0.1),
        length = at("name", 0.1)
    );
    took("sprigs");
    artkit::pot!(&p, at("place", 0.1), 0.2, rim = at("name", 0.01));
    took("pot");
    let eye = sphere!("f", 0.1);
    _put!(eye, (0.0, 0.0, at("place", 0.1)), yaw = at("name", 10.0));
    took("_put");
    artkit::cupped!(
        "g",
        &[(0.0, 0.0), (0.02, 0.01), (0.03, 0.05)],
        (0.0, 0.0, at("place", 0.1)),
        p.get("bloom"),
        yaw = at("name", 10.0)
    );
    took("cupped");
    artkit::_mesh!(
        "h",
        [v3(0, 0, 0), v3(at("place", 1.0), 0, 0), v3(0, 1, 0)],
        [[0, 1, 2]],
        p.get("leaf"),
        smooth = at("name", 1.0) > 0.0
    );
    took("_mesh");
    rays!(
        "i",
        (0, 0, 0),
        at("place", 0.05),
        5,
        p.get("bloom"),
        yaw = at("name", 10.0)
    );
    took("rays");
    twig!(
        "j",
        &[(0.0, 0.0, 0.0), (0.0, 0.0, 0.1)],
        at("place", 0.01),
        0.005,
        p.get("stem"),
        verts = at("name", 3.0) as usize
    );
    took("twig");
    cut_leaf!(
        "k",
        (0, 0, 0),
        0.0,
        0.0,
        at("place", 0.2),
        &outline!("oval", 0.1),
        p.get("leaf"),
        droop = at("name", 0.1)
    );
    took("cut_leaf");
    outline!("oval", at("place", 0.1), wave = at("name", 0.1));
    took("outline");
    pinnate!(3, at("place", 0.1), "lance", lean = at("name", 0.5));
    took("pinnate");
    vessel!(&p, at("place", 0.1), 0.2, lip = at("name", 0.01));
    took("vessel");
    soil_top!(&p, at("place", 0.1), 0.2, mound = at("name", 0.01));
    took("soil_top");
    florets!(
        "l",
        at("place", 0.1),
        (0, 0, 0),
        p.get("bloom"),
        seed = at("name", 1.0)
    );
    took("florets");
    star!(
        "m",
        (0, 0, 0),
        at("place", 0.05),
        5,
        p.get("bloom"),
        yaw = at("name", 10.0)
    );
    took("star");
    tulip_head!(
        "n",
        (0, 0, 0),
        at("place", 0.03),
        0.07,
        p.get("bloom"),
        yaw = at("name", 10.0)
    );
    took("tulip_head");
    spiral!(
        3,
        at("place", 0.3),
        &mut Random::new(1),
        jitter = at("name", 0.2)
    );
    took("spiral");
    mound!(&p, r = at("place", 0.3), h = at("name", 0.05));
    took("mound");
    perches!(
        at("place", 2.0) as usize,
        1,
        &[(0.0, 0.0, 0.1)],
        gap = at("name", 0.2)
    );
    took("perches");
    leafy_shrub!(
        1,
        at("place", 0.5),
        0.4,
        &[p.get("leaf_dark"), p.get("leaf")],
        p.get("twig"),
        leaf_size = at("name", 0.1)
    );
    took("leafy_shrub");
}

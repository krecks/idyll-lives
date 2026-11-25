//! The botany (leaves, tubes, pots, lumps, leaf cards) against what Blender 5.2 made of families/plants.py's
//! (tests/golden/botany.txt: each shape at full detail and as a far version, and two little crowns built, their
//! perches found and their leaves clad in cards, as the trees do it; captured with plants.py and trees.py as they
//! were when the botany was ported, d8038732).
mod common;

use artkit::botany::{self, CARD_OF, Cards, Tiles, card_of, join_still, stand_in};
use artkit::far::{far_of, is_low, low};
use artkit::materials::{Finish, Finishes, Mat, Palette};
use artkit::math::{TAU, V3, cos, degrees, pow, sin, v3};
use artkit::parts::settle;
use artkit::rand::Random;
use artkit::scene::{self, Obj};
use artkit::{
    _mesh, _with_part, clad, cupped, find, jitter, leaf, leaves, lump, normals, perch, pot, py,
    shade, sprigs, sweep, tube,
};
use common::*;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// The trees' palette (families/trees.py `palette`): the names are what matter here.
fn palette() -> Palette {
    let mut colours = vec![
        Mat::new("bark", "#E4D8C8", "bark"),
        Mat::new("leaf", "#558238", "foliage"),
        Mat::new("leaf2", "#64923E", "foliage"),
        Mat::new("leaf3", "#76A246", "foliage"),
        Mat::new("leaf4", "#8AB250", "foliage"),
        Mat::new("leaf5", "#9EC25C", "foliage"),
        Mat::new("leaf_light", "#8AB250", "foliage"),
        Mat::new("green", "#6E9A44", "foliage"),
        Mat::new("green2", "#62903E", "foliage"),
        Mat::new("fruit", "#C8352B", "satin"),
        Mat::new("fruit2", "#D8402C", "satin"),
        Mat::new("body", "#C0683F", "satin"),
        Mat::new("soil", "#5A4030", "matte"),
        Mat::new("stalk", "#5E5040", "satin"),
        Mat::new("core", "#334E22", "leafcards"),
    ];
    for (_, card) in CARD_OF {
        colours.push(Mat::new(card, "#FFFFFF", "leafcards"));
    }
    Palette::new(&colours)
}

/// The finishes of the base manifest the palette wears.
fn finishes() -> Finishes {
    let f = |roughness, textured| Finish {
        roughness,
        metallic: 0.0,
        textured,
    };
    [
        ("bark", f(0.6, true)),
        ("foliage", f(0.9, true)),
        ("leafcards", f(0.9, true)),
        ("satin", f(0.42, false)),
        ("matte", f(0.75, false)),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v))
    .collect()
}

// ---- what families/trees.py makes of the botany, as far as the crowns here need it ----

const GOBLET: [(f64, f64); 7] = [
    (0.0, 0.0),
    (0.045, 0.01),
    (0.1, 0.08),
    (0.12, 0.18),
    (0.1, 0.26),
    (0.078, 0.235),
    (0.0, 0.13),
];

/// A willow's hanging strand: a thin leafy ribbon from `top`, out `reach` towards `heading` and down `drop`.
fn streamer(
    name: &str,
    top: (f64, f64, f64),
    heading: f64,
    reach: f64,
    drop: f64,
    width: f64,
    material: Mat,
) -> Obj {
    let rows = low(5, 3);
    let out = v3(cos(heading), sin(heading), 0);
    let side = v3(-out.y, out.x, 0);
    let (mut verts, mut faces) = (Vec::new(), Vec::new());
    for i in 0..=rows {
        let t = i as f64 / rows as f64;
        let c =
            V3::from(top) + out * reach * (1.0 - pow(1.0 - t, 3.0)) - v3(0, 0, drop * pow(t, 1.15));
        let w = width * (1.0 - 0.66 * t) / 2.0;
        verts.extend([c - side * w, c + out * 0.03_f64.min(w * 0.5), c + side * w]);
    }
    for i in 0..rows {
        let (a, b) = (3 * i, 3 * (i + 1));
        for j in 0..3 {
            faces.push(vec![a + j, a + (j + 1) % 3, b + (j + 1) % 3, b + j]);
        }
    }
    faces.extend([vec![0, 2, 1], vec![3 * rows, 3 * rows + 1, 3 * rows + 2]]);
    _mesh!(name, verts, faces, material)
}

const STRAND_STEP: f64 = 0.5;
const STRAND_LAP: f64 = 1.5;
const STRAND_WIDE: f64 = 1.5;

/// A willow's hanging strand (`streamer`'s line) as upright leaf cards down it.
fn strand(
    cards: &mut Cards,
    top: (f64, f64, f64),
    heading: f64,
    reach: f64,
    drop: f64,
    width: f64,
    material: Mat,
) {
    let out = v3(cos(heading), sin(heading), 0);
    let n = 2.max(py::round(drop / STRAND_STEP));
    for i in 0..n {
        let t = (i as f64 + 0.5) / n as f64;
        let c =
            V3::from(top) + out * reach * (1.0 - pow(1.0 - t, 3.0)) - v3(0, 0, drop * pow(t, 1.15));
        let turn = heading + cards.rng.uniform(-0.9, 0.9);
        let tall = drop / n as f64 * STRAND_LAP;
        let facing = (cos(turn), sin(turn), cards.rng.uniform(0.0, 0.3));
        let grey = 0.62 + 0.3 * (1.0 - t) + cards.rng.uniform(-0.08, 0.08);
        cards.hang(
            c,
            facing,
            width * STRAND_WIDE,
            tall,
            material,
            |p: V3| v3(p.x, p.y, 0).normalized() + v3(0, 0, 0.5),
            grey,
        );
    }
}

thread_local! {
    /// The perches as a crown's build found them, before it was settled.
    static FOUND: RefCell<Vec<V3>> = const { RefCell::new(Vec::new()) };
    /// The scene a crown's perches were found on: each part's name, points, faces and triangles.
    static SCENE: RefCell<Vec<(String, artkit::mesh::Mesh)>> = const { RefCell::new(Vec::new()) };
    /// A species' own atlas tiles, for the crown built next.
    static TILES: RefCell<Option<Tiles>> = const { RefCell::new(None) };
}

fn found_here() {
    SCENE.with(|s| {
        *s.borrow_mut() = scene::meshes()
            .into_iter()
            .map(|o| o.read(|p| (p.name.clone(), p.mesh.clone())))
            .collect()
    });
}

fn keep_found() {
    FOUND.with(|f| *f.borrow_mut() = scene::with(|s| s.perches.clone()));
}

const TRUNK: [(f64, f64, f64); 4] = [
    (0.0, 0.0, 0.0),
    (0.0, 0.0, 0.3),
    (0.04, -0.02, 1.2),
    (0.0, 0.03, 2.0),
];

/// An oak in little: a jittered trunk, two groups of three masses, a heart of no group, a spray without a core.
fn crown1() -> Vec<Obj> {
    let q = palette();
    let cols = ["leaf", "leaf2", "leaf3", "leaf4", "leaf5"].map(|k| q.get(k));
    let mut rng = Random::new(5);
    let stem = sweep!(
        "trunk",
        &TRUNK,
        0.2,
        material = q.get("bark"),
        verts = low(7, 5),
        resolution = low(3, 1),
        scale = [1.4, 1.0, 0.85, 0.7]
    );
    if !is_low() {
        jitter!(stem, 0.03, size = 0.45, seed = 2.0, up = 0.0);
    }
    let mut clumps: Vec<(f64, f64, f64)> = Vec::new();
    for (g, tip) in [v3(0.55, 0.1, 2.3), v3(-0.5, -0.25, 2.5)]
        .into_iter()
        .enumerate()
    {
        for m in 0..3 {
            let at = if m == 0 {
                tip
            } else {
                let b = g as f64 * 2.5 + (m as f64 - 1.5) * 1.6 + rng.uniform(-0.4, 0.4);
                tip + v3(0.42 * cos(b), 0.42 * sin(b), rng.uniform(-0.35, 0.05))
            };
            let s = rng.uniform(0.5, 0.64) * if m == 0 { 1.0 } else { 0.86 };
            let (sx, sy, sq) = (
                rng.uniform(0.9, 1.15),
                rng.uniform(0.9, 1.15),
                rng.uniform(0.75, 0.88),
            );
            let col = cols[rng.randrange(5) as usize];
            let seed = clumps.len() as f64 * 1.7 + rng.random();
            clumps.push((at.x as f64, at.y as f64, at.z as f64));
            leaves!(
                format!("clump{}", clumps.len()),
                s,
                (at.x as f64, at.y as f64, at.z as f64 - s * sq),
                col,
                scale = (sx, sy, sq),
                noise = 0.34,
                seed = seed,
                segments = 6,
                rings = 5,
                group = ("clump", g)
            );
        }
    }
    leaves!(
        "heart",
        0.4,
        (0.0, 0.05, 1.9),
        cols[0],
        scale = (1, 1, 0.85),
        noise = 0.3,
        seed = 3.3,
        segments = 6,
        rings = 5
    );
    leaves!(
        "leaves",
        0.3,
        v3(1.0, 0.5, 2.5),
        q.get("green"),
        scale = (1, 1, 0.75),
        seed = 1.0,
        cards = 6,
        core = false
    );
    let still = join_still("crown1");
    let mut prng = Random::new(105);
    let mut aims = Vec::new();
    for &c in &clumps {
        aims.extend(perch::all_around(c, &mut prng, 3, (0.17, 0.95)));
    }
    found_here();
    find!(8, &aims, gap = 0.3);
    keep_found();
    let clad = match TILES.with(|t| t.borrow().clone()) {
        Some(tiles) => clad!(&q, still, 77, tiles = tiles),
        None => clad!(&q, still, 77),
    };
    settle(&[clad])
}

/// A willow in little: a trunk, three masses in two groups, ribbons standing in for the strands of cards hung down
/// them (`clad`'s `more`).
fn crown2() -> Vec<Obj> {
    let q = palette();
    let cols = ["leaf", "leaf2", "leaf3", "leaf4", "leaf5"].map(|k| q.get(k));
    let mut rng = Random::new(18);
    sweep!(
        "trunk",
        &TRUNK,
        0.25,
        material = q.get("bark"),
        verts = low(8, 5),
        resolution = low(2, 1),
        scale = [1.4, 1.05, 0.9, 0.8]
    );
    for (i, (at, k)) in [
        ((0.3, 0.2, 2.1), 0),
        ((-0.35, 0.1, 2.2), 0),
        ((0.0, -0.4, 2.0), 1),
    ]
    .into_iter()
    .enumerate()
    {
        leaves!(
            format!("mass{i}"),
            0.6,
            at,
            cols[2 + i % 3],
            scale = (1.1, 1.0, 0.65),
            noise = 0.3,
            seed = i as f64 * 1.7 + 0.4,
            segments = 6,
            rings = 5,
            group = ("crown", k)
        );
    }
    let mut strands = Vec::new();
    let n = 10;
    for i in 0..n {
        let a = i as f64 / n as f64 * TAU + rng.uniform(-0.04, 0.04);
        let (rr, top, foot) = (
            rng.uniform(0.9, 1.1),
            rng.uniform(2.1, 2.5),
            rng.uniform(0.45, 0.8),
        );
        let (heading, reach, width, col) = (
            a + rng.uniform(-0.15, 0.15),
            rng.uniform(0.25, 0.45),
            rng.uniform(0.17, 0.23),
            cols[rng.randint(1, 4) as usize],
        );
        let ribbon = streamer(
            &format!("strand{i}"),
            (rr * cos(a), rr * sin(a), top),
            heading,
            reach,
            top - foot,
            width * low(1.0, 1.5),
            col,
        );
        stand_in(ribbon);
        strands.push((
            (rr * cos(a), rr * sin(a), top),
            heading,
            reach,
            top - foot,
            width,
            q.get(card_of(col.name)),
        ));
    }
    let still = join_still("crown2");
    let mut prng = Random::new(118);
    let mut aims = Vec::new();
    for c in [(0.3, 0.2, 2.5), (-0.35, 0.1, 2.6), (0.0, -0.4, 2.4)] {
        aims.extend(perch::all_around(c, &mut prng, 4, (0.55, 1.2)));
    }
    found_here();
    find!(
        6,
        &aims,
        limbs = vec![v3(0.02, 0.0, 2.02), v3(0.5, 0.2, 1.2)],
        limbs_n = 1,
        gap = 0.3
    );
    keep_found();
    let clad = clad!(
        &q,
        still,
        218,
        more = |cards: &mut Cards| {
            for &(top, heading, reach, drop, width, material) in &strands {
                strand(cards, top, heading, reach, drop, width, material);
            }
        }
    );
    settle(&[clad])
}

fn with_part() -> Vec<Obj> {
    let p = palette();
    let still = lump!(
        "bush",
        0.5,
        (0, 0, 0),
        p.get("leaf2"),
        scale = (1.2, 1, 0.8),
        seed = 2.0,
        segments = 6,
        rings = 5
    );
    let at = (0.1, -0.05, 0.7);
    _with_part!(
        "sprigs",
        still,
        || sprigs!("sprig", &p, at, 0.3, n = 4, length = 0.2),
        at,
        "trim"
    )
}

/// Builds the case named `name` in the scene in hand; what it returned, if it was a number.
fn build(name: &str) -> Option<f64> {
    let p = palette();
    let m = |k: &str| p.get(k);
    match name {
        "mesh_smooth" => {
            _mesh!(
                "m",
                [v3(0, 0, 0), v3(1, 0, 0), v3(0, 1, 0), v3(0.1, 0.2, 0.8)],
                [[0, 1, 2], [0, 1, 3], [1, 2, 3], [0, 2, 3]],
                m("leaf")
            );
        }
        "mesh_flat" => {
            _mesh!(
                "m",
                vec![
                    v3(0, 0, 0),
                    v3(1, 0, 0),
                    v3(1, 1, 0.2),
                    v3(0, 1, 0),
                    v3(0.5, 0.5, -0.6)
                ],
                vec![
                    vec![0, 1, 2, 3],
                    vec![1, 0, 4],
                    vec![1, 2, 4],
                    vec![3, 2, 4],
                    vec![0, 3, 4]
                ],
                m("leaf2"),
                smooth = false
            );
        }
        "streamer" => {
            streamer(
                "strand",
                (1.9, 0.4, 4.6),
                0.3,
                0.35,
                3.9,
                0.2 * low(1.0, 1.5),
                m("leaf3"),
            );
        }
        "leaf" => {
            leaf!("leaf", (0.1, 0.2, 0.3), 30.0, 40.0, 0.5, 0.2, m("leaf"));
        }
        "leaf_lance" => {
            leaf!(
                "leaf",
                v3(0.4, -0.2, 1.3),
                200.5,
                61.25,
                0.22,
                0.099,
                m("leaf_light"),
                shape = "lance",
                droop = 0.2,
                fold = 0.2,
                rows = 2,
                ring = 3
            );
        }
        "leaf_heart" => {
            leaf!(
                "leaf",
                (0, 0, 0.5),
                -75.0,
                15.0,
                0.6,
                0.45,
                m("leaf2"),
                shape = "heart",
                notch = (2, 0.5),
                twist = 40,
                rows = 8,
                ring = 5,
                droop = 0.35
            );
        }
        "leaf_strap" => {
            leaf!(
                "leaf",
                (0.3, 0.3, 0),
                110.0,
                80.0,
                1.1,
                0.08,
                m("leaf3"),
                shape = "strap",
                droop = 0.6,
                thick = 0.02,
                fold = 0.4
            );
        }
        "leaf_petal" => {
            leaf!(
                "leaf",
                (-0.2, 0.1, 0.9),
                0.0,
                0.0,
                0.12,
                0.07,
                m("fruit"),
                shape = "petal",
                droop = 0.00004,
                fold = 0.0,
                rows = 4,
                ring = 4
            );
        }
        "tube" => {
            tube!(
                "stalk",
                &[(0.1, -0.6, 1.74), (0.104, -0.604, 1.71)],
                0.011,
                0.008,
                m("stalk"),
                verts = 4
            );
        }
        "tube_bent" => {
            tube!(
                "stem",
                &[
                    v3(0, 0, 0),
                    v3(0.02, 0.01, 0.3),
                    v3(0.1, 0.0, 0.55),
                    v3(0.25, -0.05, 0.6)
                ],
                0.02,
                0.006,
                m("bark"),
                cap = false
            );
        }
        "tube_capped" => {
            tube!(
                "stem",
                &[(0.0, 0.0, 0.1), (0.0, 0.0, 0.5), (0.2, 0.2, 0.7)],
                0.03,
                0.01,
                m("bark"),
                verts = 5
            );
        }
        "pot" => return Some(pot!(&p, 0.3, 0.5, r_bottom = 0.22, rim = 0.016)),
        "pot_rimless" => {
            return Some(pot!(
                &p,
                0.15,
                0.25,
                z0 = 0.1,
                rim = 0,
                rim_h = 0.02,
                soil = false,
                segments = 10
            ));
        }
        "cupped" => {
            cupped!(
                "bloom",
                &GOBLET,
                v3(1.2, -0.7, 3.1),
                m("fruit"),
                yaw = 90.0 - degrees(0.8),
                tilt = 8.0 + 40.0 * 0.45,
                segments = low(6, 4)
            );
        }
        "cupped_plain" => {
            cupped!(
                "bloom",
                &[(0.0, 0.0), (0.03, 0.01), (0.05, 0.06), (0.02, 0.09)],
                (0.1, 0.2, 0.3),
                m("fruit2")
            );
        }
        "lump" => {
            lump!(
                "lump",
                0.45,
                (0.3, -0.2, 2.5),
                m("leaf4"),
                scale = (1, 1.1, 0.8),
                noise = 0.34,
                seed = 5.1,
                segments = 6,
                rings = 5
            );
        }
        "lump_plain" => {
            lump!("lump", 0.07, v3(0.1, -0.6, 1.62), m("fruit"));
        }
        "sprigs" => sprigs!("sprig", &p, (0.1, -0.05, 3.2), 0.35, n = 6, length = 0.22),
        "sprigs_plain" => sprigs!("tuft", &p, v3(0, 0, 1), 0.2, n = 2),
        "with_part" => {
            with_part();
        }
        other => panic!("no case {other:?}"),
    }
    None
}

// ---- comparing with the goldens ----

/// A golden's cases in order: (name, its meshes).
fn cases() -> Vec<(String, Vec<GoldenMesh>)> {
    let mut out: Vec<(String, Vec<GoldenMesh>)> = Vec::new();
    for mesh in golden_meshes("botany.txt") {
        match out.last_mut() {
            Some((name, meshes)) if *name == mesh.what => meshes.push(mesh),
            _ => out.push((mesh.what.clone(), vec![mesh])),
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

/// Checks the part's UVs against the golden's, to the bit.
fn same_uvs(what: &str, mesh: &artkit::mesh::Mesh, g: &GoldenMesh) {
    assert_eq!(mesh.faces.len(), g.faces.len(), "{what}: faces");
    let (_, corner) = its_order(mesh, &g.faces);
    let want = g.f32s("uv");
    assert_eq!(want.len(), 2 * mesh.loops.len(), "{what}: a UV a corner");
    let uvs = mesh.uvs.as_ref().expect("its UVs");
    let wrong = corner
        .iter()
        .enumerate()
        .filter(|&(l, &mine)| bits(&uvs[mine]) != bits(&want[2 * l..2 * l + 2]))
        .count();
    assert_eq!(wrong, 0, "{what}: UVs differ, of {}", corner.len());
}

/// How far a built part is from the golden one, where it may be at all: the normals' farthest and how many of the
/// corners' aren't Blender's bits (in the kernel's own order of faces), and how many vertices' baked shading is a
/// level out.
#[derive(Default, Debug)]
struct Off {
    normals: f32,
    corners: usize,
    inexact: usize,
    ao: usize,
    ao_far: i64,
    vertices: usize,
}

/// Checks the part `obj` against the golden `g`: its place, its points to the bit and in order, its faces (each
/// from the corner Blender's starts at) and triangles as sets, and what `g` has of its shading, materials, sharp
/// edges, UVs and the cards' own shading, to the bit; the normals and the baked shading are measured (`off`).
fn same(case: &str, obj: Obj, g: &GoldenMesh, off: &mut Off) {
    let what = format!("{case}: {}", g.name);
    let (mesh, loc, rot, scale) = obj.read(|o| (o.mesh.clone(), o.loc, o.rot, o.scale));
    let place = [loc.array(), rot.array(), scale.array()].concat();
    // (a zero's sign isn't a difference: Blender leaves -0 in a turn it has applied)
    assert_eq!(place, g.f32s("loc"), "{what}: its place");
    assert_eq!(mesh.verts.len(), g.verts.len(), "{what}: points");
    let (n, far) = points_off(&mesh.verts, &g.verts);
    assert_eq!(
        (n, far),
        (0, 0),
        "{what}: {n} points differ, by up to {far} ulps"
    );
    let faces: Vec<Vec<u32>> = (0..mesh.faces.len())
        .map(|f| mesh.corners(f).to_vec())
        .collect();
    assert_eq!(sorted(&faces), sorted(&g.faces), "{what}: faces");
    assert_eq!(
        as_set(&mesh.triangle_verts()),
        as_set(&g.tris),
        "{what}: triangles"
    );
    if !g.lines.contains_key("s") {
        return;
    }
    let (of, corner) = its_order(&mesh, &g.faces);
    let smooth: String = of
        .iter()
        .map(|&f| if mesh.faces[f].smooth { '1' } else { '0' })
        .collect();
    assert_eq!(smooth, g.lines["s"], "{what}: smooth faces");
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
    assert_eq!(
        mesh.sharp.iter().copied().collect::<Vec<_>>(),
        sharp,
        "{what}: sharp edges"
    );
    if g.lines.contains_key("uv") && mesh.uvs.is_some() {
        same_uvs(&what, &mesh, g);
        assert_eq!(
            mesh.uv_metres,
            g.lines["uvm"].starts_with('1'),
            "{what}: UVs made"
        );
    }
    let card = mesh.card_shade.clone();
    match g.lines.get("cs") {
        Some(cs) => assert_eq!(
            bits(card.as_deref().expect("the cards' shading")),
            bits(&f32s(&words(cs))),
            "{what}: the cards' shading"
        ),
        None => assert!(card.is_none(), "{what}: no cards"),
    }
    // The normals: Blender's to the bit where the order of the faces round a vertex is its own (a fan all the way
    // round a vertex, and a vertex of a mesh smooth all over, sum their faces in the mesh's order, which for
    // Blender's own primitives isn't the same from one run of it to the next); measured otherwise.
    let cn = g.f32s("cn");
    if !cn.is_empty() {
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
            let on_card = card
                .as_ref()
                .is_some_and(|s| s[mesh.loops[c] as usize] > 0.0);
            if on_card {
                assert_eq!(
                    bits(&mine[c].array()),
                    bits(want),
                    "{what}: a card's normal at corner {l}"
                );
            } else {
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
    if let Some(ao) = g.lines.get("ao") {
        shade::bake_ao(&[obj], None, true);
        let got = obj.read(|o| o.mesh.ao.clone().unwrap());
        for (a, w) in got.iter().zip(words(ao)) {
            let d = ((a * 32.0).round() as i64 - w.parse::<i64>().unwrap()).abs();
            off.ao += (d > 0) as usize;
            off.ao_far = off.ao_far.max(d);
        }
        off.vertices += got.len();
    }
    if let Some(clips) = g.lines.get("clips") {
        let got: Vec<String> = obj.read(|o| {
            o.tracks
                .iter()
                .map(|t| format!("{}:1:{}", t.name, t.end))
                .collect()
        });
        assert_eq!(got.join(" "), *clips, "{what}: clips");
    }
}

#[test]
fn the_shapes_are_blenders_at_full_detail_and_as_far_versions() {
    scene::set_finishes(finishes());
    let mut off = Off::default();
    let mut n = 0;
    for (case, meshes) in cases() {
        let (name, far) = match case.strip_suffix(".far") {
            Some(name) => (name, true),
            None => (case.as_str(), false),
        };
        if name.starts_with("crown") {
            continue;
        }
        scene::reset();
        botany::clear();
        let ret = if far {
            far_of(|| build(name))
        } else {
            build(name)
        };
        if let Some(want) = meshes[0].lines.get("ret") {
            assert_eq!(ret.map(f64::to_bits), Some(f64w(want).to_bits()), "{case}");
        }
        let made = scene::meshes();
        assert_eq!(
            made.iter().map(|o| o.name()).collect::<Vec<_>>(),
            meshes.iter().map(|g| g.name.clone()).collect::<Vec<_>>(),
            "{case}: its parts"
        );
        for (&obj, g) in made.iter().zip(&meshes) {
            same(&case, obj, g, &mut off);
        }
        n += 1;
    }
    assert!(n >= 38, "{n} cases");
    // (in the kernel's own order of faces: a unit or two in the last place, at one corner in fifty)
    assert!(
        off.normals < 2.5e-7 && off.inexact * 20 < off.corners,
        "{off:?}"
    );
}

/// Builds the crown `name` and checks it against its golden: its size, its perches as found and as settled, the
/// scene they were found on, and the clad tree.
fn crown(name: &str, build: fn() -> Vec<Obj>, tiles: Option<Tiles>, off: &mut Off) {
    let all = cases();
    for far in [false, true] {
        let case = format!("{name}{}", if far { ".far" } else { "" });
        let found = all.iter().find(|(n, _)| *n == case);
        // (a crown with tiles of its own is the crown but for its UVs, which are all its golden has, with its faces
        // in the order Blender had them that time)
        let mut own_uvs: Option<&Vec<GoldenMesh>> = None;
        let meshes: Vec<GoldenMesh> = match (&tiles, found) {
            (None, Some((_, meshes))) => meshes.clone(),
            (Some(_), Some((_, own))) => {
                own_uvs = Some(own);
                let plain = name.split('.').next().unwrap_or(name);
                let (_, meshes) = all.iter().find(|(n, _)| n == plain).expect("its crown");
                let mut meshes = meshes.clone();
                for mesh in &mut meshes {
                    mesh.lines.remove("uv");
                }
                meshes
            }
            (Some(_), None) if far => continue,
            _ => panic!("no golden for {case}"),
        };
        let meshes = &meshes;
        scene::reset();
        perch::clear();
        botany::clear();
        TILES.with(|t| *t.borrow_mut() = tiles.clone());
        FOUND.with(|f| f.borrow_mut().clear());
        let objs = if far { far_of(build) } else { build() };
        TILES.with(|t| *t.borrow_mut() = None);
        // the scene its perches were found on: the still part, the lumps and what stands in
        if let Some((_, lumps)) = all.iter().find(|(n, _)| *n == format!("{case}.scene")) {
            let scene = SCENE.with(|s| s.borrow().clone());
            assert_eq!(
                scene.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>(),
                lumps.iter().map(|g| g.name.clone()).collect::<Vec<_>>(),
                "{case}: the scene its perches were found on"
            );
            for ((part, mesh), g) in scene.iter().zip(lumps) {
                assert_eq!(
                    points_off(&mesh.verts, &g.verts),
                    (0, 0),
                    "{case}: {part} as the rays met it"
                );
                assert_eq!(
                    as_set(&mesh.triangle_verts()),
                    as_set(&g.tris),
                    "{case}: {part}'s triangles"
                );
            }
        }
        let first = &meshes[0];
        let size = f64s(&words(&first.lines["size"]));
        assert_eq!(artkit::parts::game_size(&objs).to_vec(), size, "{case}");
        let flat = |ps: &[V3]| bits(&ps.iter().flat_map(|p| p.array()).collect::<Vec<_>>());
        let found = FOUND.with(|f| f.borrow().clone());
        assert_eq!(
            flat(&found),
            bits(&first.f32s("found")),
            "{case}: the perches as found"
        );
        let settled = scene::with(|s| s.perches.clone());
        assert_eq!(
            flat(&settled),
            bits(&first.f32s("settled")),
            "{case}: the perches as settled"
        );
        let game: Vec<String> = perch::game()
            .iter()
            .map(|p| format!("[{}, {}, {}]", p[0], p[1], p[2]))
            .collect();
        assert_eq!(
            format!("[{}]", game.join(", ")),
            first.lines["game"],
            "{case}"
        );
        assert_eq!(objs.len(), meshes.len(), "{case}: its parts");
        for (&obj, g) in objs.iter().zip(meshes) {
            assert_eq!(obj.name(), g.name, "{case}");
            same(&case, obj, g, off);
        }
        for (&obj, g) in objs.iter().zip(own_uvs.into_iter().flatten()) {
            same_uvs(&case, &obj.read(|o| o.mesh.clone()), g);
        }
    }
}

#[test]
fn a_crown_clad_in_cards_is_blenders_and_its_perches_stand_where_they_stood() {
    scene::set_finishes(finishes());
    let mut off = Off::default();
    crown("crown1", crown1, None, &mut off);
    crown("crown2", crown2, None, &mut off);
    // (the lumps' and cores' normals in the kernel's own order of faces: a unit or two in the last place; the
    // cards' are Blender's to the bit, and the baked shading its level at every vertex)
    assert!(
        off.normals < 2.5e-7 && off.inexact * 10 < off.corners,
        "{off:?}"
    );
    assert_eq!((off.ao, off.ao_far), (0, 0), "of {} vertices", off.vertices);
}

#[test]
fn a_species_own_tiles_move_only_the_cards_and_cores_pictures() {
    scene::set_finishes(finishes());
    // (as many tiles as the shared atlas has: a card draws its tile, so with more or fewer the cards fall otherwise)
    let tiles = Tiles {
        cards: vec![(0.5, 0.0), (0.0, 0.0), (0.5, 0.5)],
        core: (0.0, 0.5),
        ..Tiles::default()
    };
    crown("crown1.tiles", crown1, Some(tiles), &mut Off::default());
}

#[test]
fn spreading_no_perches_keeps_none() {
    let points = [V3::new(0.0, 0.0, 1.0), V3::new(1.0, 0.0, 0.0)];
    assert!(perch::spread(&points, 0, 0.45).is_empty());
}

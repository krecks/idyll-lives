//! The kit's raised letters (`kit::letters`) against Blender 5.2's text (tests/golden/kit_letters.txt): the retro mini
//! fridge's name plate, the only text the family scripts set (kitchen.py's `_letters`: "MARIE", size 0.064, 1.5 mm
//! out each way, letters spaced 1.25, its curves in three steps, made a mesh). call: the word and, as doubles, the
//! game place and the size the script gave; the mesh is Blender's in the text's own frame, at loc (place, turn,
//! scale), turned a quarter about x to face the front.
//!
//! The kernel has no font: its block letters are held to where Blender's stand and how heavy they are. The word's
//! box within a millimetre, each letter's within two, the surface within a tenth, and no more triangles than
//! Blender's and a tenth.
//!
//! KIT_LETTERS_PICTURES=<folder> draws both from the front (Blender's on the left) into letters.ppm there.
mod common;

use artkit::kit::{self, Letters};
use artkit::materials::Mat;
use artkit::math::V3;
use artkit::place::GameFrame;
use artkit::scene;
use common::*;
use std::collections::BTreeMap;

const GOLD: Mat = Mat::new("gold", "#D2AE4E", "metal");

/// A letter of a word's mesh: its box, its surface (m2) and its triangles.
#[derive(Debug)]
struct Letter {
    lo: [f32; 3],
    hi: [f32; 3],
    area: f64,
    triangles: usize,
    /// Six times what its triangles close in: over zero where they look outwards.
    volume: f64,
}

/// The letters of a mesh, from the left: its triangles gathered by the points they share (points in one place
/// are one), each gathering a letter.
fn letters_of(points: &[V3], triangles: &[[u32; 3]]) -> Vec<Letter> {
    let mut group: Vec<usize> = (0..points.len()).collect();
    fn top(group: &mut [usize], mut i: usize) -> usize {
        while group[i] != i {
            group[i] = group[group[i]];
            i = group[i];
        }
        i
    }
    let mut at: BTreeMap<[i64; 3], usize> = BTreeMap::new();
    for (i, p) in points.iter().enumerate() {
        let key = [p.x, p.y, p.z].map(|c| (c as f64 * 1e6).round() as i64);
        let first = *at.entry(key).or_insert(i);
        let (a, b) = (top(&mut group, i), top(&mut group, first));
        group[a] = b;
    }
    for t in triangles {
        let a = top(&mut group, t[0] as usize);
        for &v in &t[1..] {
            let b = top(&mut group, v as usize);
            group[b] = a;
        }
    }
    let mut found: BTreeMap<usize, Letter> = BTreeMap::new();
    for t in triangles {
        let g = top(&mut group, t[0] as usize);
        let l = found.entry(g).or_insert(Letter {
            lo: [f32::INFINITY; 3],
            hi: [f32::NEG_INFINITY; 3],
            area: 0.0,
            triangles: 0,
            volume: 0.0,
        });
        let p = t.map(|v| points[v as usize]);
        for q in p {
            for k in 0..3 {
                l.lo[k] = l.lo[k].min(q[k]);
                l.hi[k] = l.hi[k].max(q[k]);
            }
        }
        l.area += (p[1] - p[0]).cross(p[2] - p[0]).length() / 2.0;
        l.triangles += 1;
        // (from the first point the letter was found at, so the sum stays small)
        let from = V3::new(l.lo[0], l.lo[1], l.lo[2]);
        l.volume += (p[0] - from).dot((p[1] - from).cross(p[2] - from));
    }
    let mut out: Vec<Letter> = found.into_values().collect();
    out.sort_by(|a, b| a.lo[0].total_cmp(&b.lo[0]));
    out
}

/// The whole word's box.
fn word(letters: &[Letter]) -> ([f32; 3], [f32; 3]) {
    let (mut lo, mut hi) = ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]);
    for l in letters {
        for k in 0..3 {
            lo[k] = lo[k].min(l.lo[k]);
            hi[k] = hi[k].max(l.hi[k]);
        }
    }
    (lo, hi)
}

/// The most two boxes' sides are apart, in millimetres.
fn apart(a: (&[f32; 3], &[f32; 3]), b: (&[f32; 3], &[f32; 3])) -> f32 {
    (0..3)
        .map(|k| (a.0[k] - b.0[k]).abs().max((a.1[k] - b.1[k]).abs()))
        .fold(0.0, f32::max)
        * 1000.0
}

#[test]
fn the_fridges_name_stands_where_blenders_does() {
    let g = &golden_meshes("kit_letters.txt")[0];
    let call: Vec<&str> = g.lines["call"].split(' ').collect();
    let (text, x, y, z, size) = (
        call[0],
        f64w(call[1]),
        f64w(call[2]),
        f64w(call[3]),
        f64w(call[4]),
    );
    // Blender's, in the world: turned a quarter about x, then moved
    let loc = g.f32s("loc");
    assert!((loc[3] - std::f32::consts::FRAC_PI_2).abs() < 1e-6 && loc[4] == 0.0 && loc[5] == 0.0);
    let theirs: Vec<V3> = g
        .verts
        .iter()
        .map(|p| V3::new(loc[0] + p[0], loc[1] - p[2], loc[2] + p[1]))
        .collect();
    let blender = letters_of(&theirs, &g.tris);

    scene::reset();
    let made = kit::letters(
        &GameFrame::new(0.0),
        "letters",
        text,
        size,
        0.003,
        (x, y, z),
        GOLD,
        Letters { space: 1.25 },
    );
    let (ours, triangles) = (made.world_points(), made.read(|o| o.mesh.triangle_verts()));
    let kernel = letters_of(&ours, &triangles);

    assert_eq!((blender.len(), kernel.len()), (5, 5), "five letters each");
    for ((ch, b), k) in text.chars().zip(&blender).zip(&kernel) {
        let off = apart((&b.lo, &b.hi), (&k.lo, &k.hi));
        println!(
            "{ch}: Blender's {:.2} .. {:.2} mm across, {:.2} .. {:.2} up, {:.0} mm2, {} triangles; the kernel's {:.2} .. {:.2}, {:.2} .. {:.2}, {:.0} mm2, {}: its box {off:.2} mm off at most, its surface {:+.1} %",
            b.lo[0] * 1e3,
            b.hi[0] * 1e3,
            b.lo[2] * 1e3,
            b.hi[2] * 1e3,
            b.area * 1e6,
            b.triangles,
            k.lo[0] * 1e3,
            k.hi[0] * 1e3,
            k.lo[2] * 1e3,
            k.hi[2] * 1e3,
            k.area * 1e6,
            k.triangles,
            (k.area / b.area - 1.0) * 100.0
        );
        assert!(off <= 2.0, "{ch}: its box {off} mm off Blender's");
        assert!(k.volume > 0.0, "{ch}: closed, its faces looking outwards");
    }
    let (b, k) = (word(&blender), word(&kernel));
    let off = apart((&b.0, &b.1), (&k.0, &k.1));
    let area = |l: &[Letter]| l.iter().map(|l| l.area).sum::<f64>();
    let count = |l: &[Letter]| l.iter().map(|l| l.triangles).sum::<usize>();
    println!(
        "{text}: the word's box {off:.2} mm off Blender's at most ({:?} .. {:?}, Blender's {:?} .. {:?}); its surface {:.0} mm2, Blender's {:.0} ({:+.1} %); {} triangles, Blender's {}",
        k.0,
        k.1,
        b.0,
        b.1,
        area(&kernel) * 1e6,
        area(&blender) * 1e6,
        (area(&kernel) / area(&blender) - 1.0) * 100.0,
        count(&kernel),
        count(&blender)
    );
    assert!(off <= 1.0, "the word's box {off} mm off Blender's");
    assert!(
        (area(&kernel) / area(&blender) - 1.0).abs() <= 0.1,
        "the surface"
    );
    assert!(
        count(&kernel) as f64 <= count(&blender) as f64 * 1.1,
        "the triangles"
    );
    assert_eq!(count(&blender), g.tris.len());

    if let Ok(folder) = std::env::var("KIT_LETTERS_PICTURES") {
        draw(&folder, &theirs, &g.tris, &ours, &triangles);
    }
}

/// Both words from the front on the door's pink, Blender's on the left, as a PPM.
fn draw(
    folder: &str,
    theirs: &[V3],
    their_triangles: &[[u32; 3]],
    ours: &[V3],
    our_triangles: &[[u32; 3]],
) {
    use artkit::preview::{View, compose, picture};
    use artkit::raster::{self, Surface};
    // (the file's axes: Blender's x, z, -y)
    let mesh = |points: &[V3], triangles: &[[u32; 3]], colour: [u8; 3]| raster::Mesh {
        positions: points.iter().map(|p| [p.x, p.z, -p.y]).collect(),
        indices: triangles.iter().flatten().copied().collect(),
        surface: Surface {
            colour: raster::linear(colour),
            ..Surface::default()
        },
        ..raster::Mesh::default()
    };
    let (lo, hi) = theirs.iter().fold(
        ([f32::INFINITY; 3], [f32::NEG_INFINITY; 3]),
        |(mut lo, mut hi), p| {
            for k in 0..3 {
                lo[k] = lo[k].min(p[k]);
                hi[k] = hi[k].max(p[k]);
            }
            (lo, hi)
        },
    );
    // the door's face behind the word, a little larger than it
    let (m, back) = (0.02, hi[1]);
    let plate = [
        V3::new(lo[0] - m, back, lo[2] - m),
        V3::new(hi[0] + m, back, lo[2] - m),
        V3::new(hi[0] + m, back, hi[2] + m),
        V3::new(lo[0] - m, back, hi[2] + m),
    ];
    let door = mesh(&plate, &[[0, 1, 2], [0, 2, 3]], [0xF2, 0xB8, 0xC6]);
    let pictures: Vec<_> = [(theirs, their_triangles), (ours, our_triangles)]
        .into_iter()
        .flat_map(|(points, triangles)| {
            let word = mesh(points, triangles, [0xD2, 0xAE, 0x4E]);
            [View::Front, View::ThreeQuarter]
                .map(|view| picture(&[door.clone(), word.clone()], &[], view, 720, false))
        })
        .collect();
    let sheet = compose(
        &[
            vec![pictures[0].clone(), pictures[2].clone()],
            vec![pictures[1].clone(), pictures[3].clone()],
        ],
        4,
    );
    let mut out = format!("P6\n{} {}\n255\n", sheet.width, sheet.height).into_bytes();
    for px in sheet.rgba.chunks(4) {
        out.extend_from_slice(&px[..3]);
    }
    std::fs::write(format!("{folder}/letters.ppm"), out).unwrap();
}

//! Signs: the taxi's livery on the taxi rank's signs (its word and its chequers), raised block letters, and a band or
//! a label painted onto a part's own faces.

use crate::forms::{self, Extrude, Holes, Plane};
use crate::materials::Mat;
use crate::math::{Num, V3, atan2, degrees};
use crate::place::{Coord, GameFrame};
use crate::py;
use crate::scene::Obj;
use crate::shapes::{Cube, options};

/// Gives the faces of `obj` whose middle (its own axes, Blender's; `Mesh::face_centre`) passes `test` another
/// material: a band, a surround, a label, a rind, with no more triangles (the packs' `_paint`).
pub fn paint(obj: Obj, material: Mat, test: impl Fn(V3) -> bool) -> Obj {
    obj.edit(|p| {
        let k = p.mesh.slot(material);
        for f in 0..p.mesh.faces.len() {
            if test(p.mesh.face_centre(f)) {
                p.mesh.faces[f].mat = k;
            }
        }
    });
    obj
}

options! {
    /// `taxi_word`'s options.
    TaxiWord { depth: f64 = 0.004 }
}

/// TAXI in block letters `h` high, its middle at game (x, height) `centre`, standing `depth` proud of a face at
/// depth `z` that looks `facing` (+1: to the front, +z; -1: to the back), so it reads from that side: seen from
/// the front, the game's +x is on the left (README, "Coordinates"), so the word runs from +x there, from -x
/// behind. The taxi rank's signs and fares board, as the taxi's own roof sign (render/shared/cars.ts `paintTaxi`).
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn taxi_word(
    frame: &GameFrame,
    name: impl AsRef<str>,
    centre: (impl Num, impl Num),
    h: f64,
    z: f64,
    facing: i32,
    material: Mat,
    o: TaxiWord,
) -> Vec<Obj> {
    let name = name.as_ref();
    let (depth, facing) = (o.depth, facing as f64);
    let (cx, cy) = (centre.0.f(), centre.1.f());
    let (s, w, gap) = (0.2 * h, 0.66 * h, 0.22 * h);
    let total = 4.0 * w + 3.0 * gap;
    let zz = z + facing * depth / 2.0;
    let lean = degrees(atan2(w / 2.0 - s / 2.0, h)); // an A's legs from upright
    let cross = degrees(atan2(w - s, h)); // an X's strokes from upright
    let mut out = Vec::new();
    let turned = |about_y: f64| Cube {
        rot: (0.0, about_y, 0.0).into(),
        ..Cube::default()
    };
    for (i, ch) in "TAXI".chars().enumerate() {
        // (Seen from the side it faces, letter by letter left to right.)
        let x = cx + facing * (total / 2.0 - i as f64 * (w + gap) - w / 2.0);
        let n = format!("{name}_{ch}{i}");
        match ch {
            'T' => {
                out.push(frame.box_(
                    format!("{n}_bar"),
                    w,
                    s,
                    depth,
                    (x, cy + h / 2.0 - s / 2.0, zz),
                    material,
                    Cube::default(),
                ));
                out.push(frame.box_(
                    format!("{n}_stem"),
                    s,
                    h - s,
                    depth,
                    (x, cy - s / 2.0, zz),
                    material,
                    Cube::default(),
                ));
            }
            'A' => {
                for k in [-1, 1] {
                    let leg = py::hypot(w / 2.0 - s / 2.0, h);
                    out.push(frame.box_(
                        format!("{n}_leg{k}"),
                        s,
                        leg,
                        depth,
                        (x + k as f64 * (w / 4.0 - s / 4.0), cy, zz),
                        material,
                        turned(k as f64 * lean),
                    ));
                }
                out.push(frame.box_(
                    format!("{n}_bar"),
                    w * 0.42,
                    s * 0.8,
                    depth,
                    (x, cy - h * 0.12, zz),
                    material,
                    Cube::default(),
                ));
            }
            'X' => {
                for k in [-1, 1] {
                    out.push(frame.box_(
                        format!("{n}_stroke{k}"),
                        s,
                        py::hypot(w - s, h) * 0.97,
                        depth,
                        (x, cy, zz),
                        material,
                        turned(k as f64 * cross),
                    ));
                }
            }
            _ => out.push(frame.box_(
                format!("{n}_stem"),
                s,
                h,
                depth,
                (x, cy, zz),
                material,
                Cube::default(),
            )),
        }
    }
    out
}

options! {
    /// `chequer`'s options.
    Chequer { rows: usize = 2, depth: f64 = 0.003 }
}

/// A chequered band's dark squares (`size` a side, `rows` of them, the band's top at height `y`) from x0 to x1
/// along a face at depth `z` looking `facing` (+1 front, -1 back), `depth` proud of it: the light squares are the
/// face behind. The taxi's livery, on the taxi rank's signs and shelter.
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn chequer(
    frame: &GameFrame,
    name: impl AsRef<str>,
    x0: f64,
    x1: f64,
    y: f64,
    z: f64,
    size: f64,
    facing: i32,
    material: Mat,
    o: Chequer,
) -> Vec<Obj> {
    let name = name.as_ref();
    let zz = z + facing as f64 * o.depth / 2.0;
    let mut out = Vec::new();
    for r in 0..o.rows {
        let mut k = 0;
        let mut x = x0;
        while x + size <= x1 + 1e-6 {
            if (k + r) % 2 == 0 {
                out.push(frame.box_(
                    format!("{name}{r}_{k}"),
                    size,
                    size,
                    o.depth,
                    (x + size / 2.0, y - size / 2.0 - r as f64 * size, zz),
                    material,
                    Cube::default(),
                ));
            }
            x += size;
            k += 1;
        }
    }
    out
}

// --- raised letters ----------------------------------------------------------------------------------------------

/// A letter drawn on the letters' grid (`GRID` of the text's size a square, a capital 112 squares high, its stems 16
/// wide): the space before it and its whole width on the line, then its outline and the holes in it, each from
/// the letter's left edge and the line it stands on.
struct Glyph {
    letter: char,
    lead: i32,
    advance: i32,
    outline: &'static [(i32, i32)],
    holes: &'static [&'static [(i32, i32)]],
}

/// A square of the letters' grid, in text sizes: a capital is 112 squares, 0.682 of the size, as Blender's own
/// font has it (0.39 mm a square on the mini fridge's name).
const GRID: f64 = 0.00609;

/// How far under the middle of the text the line the letters stand on is, in text sizes (Blender's text, centred
/// on its height).
const BASELINE: f64 = 0.3;

/// The letters the families write (the retro mini fridge's name), as Blender's built-in font draws them, its
/// curves (the R's bowl and leg) in a few straight strokes: where each stands, how wide and how heavy it is are
/// measured from Blender's own text, to the nearest square.
const GLYPHS: [Glyph; 5] = [
    Glyph {
        letter: 'A',
        lead: 0,
        advance: 110,
        outline: &[
            (93, 0),
            (110, 0),
            (59, 113),
            (48, 113),
            (0, 0),
            (16, 0),
            (31, 33),
            (78, 33),
        ],
        holes: &[&[(72, 47), (37, 47), (54, 88)]],
    },
    Glyph {
        letter: 'E',
        lead: 12,
        advance: 82,
        outline: &[
            (16, 98),
            (64, 98),
            (64, 112),
            (0, 112),
            (0, 0),
            (65, 0),
            (65, 14),
            (16, 14),
            (16, 49),
            (62, 49),
            (62, 64),
            (16, 64),
        ],
        holes: &[],
    },
    Glyph {
        letter: 'I',
        lead: 12,
        advance: 41,
        outline: &[(0, 0), (16, 0), (16, 112), (0, 112)],
        holes: &[],
    },
    Glyph {
        letter: 'M',
        lead: 12,
        advance: 129,
        outline: &[
            (88, 0),
            (104, 0),
            (104, 112),
            (89, 112),
            (52, 66),
            (15, 112),
            (0, 112),
            (0, 0),
            (16, 0),
            (16, 87),
            (50, 44),
            (54, 44),
            (88, 87),
        ],
        holes: &[],
    },
    Glyph {
        letter: 'R',
        lead: 12,
        advance: 99,
        outline: &[
            (48, 108),
            (28, 112),
            (0, 112),
            (0, 0),
            (16, 0),
            (16, 49),
            (23, 49),
            (31, 47),
            (40, 40),
            (53, 23),
            (67, 0),
            (87, 0),
            (55, 45),
            (46, 53),
            (56, 60),
            (63, 69),
            (65, 81),
            (61, 97),
        ],
        holes: &[&[
            (26, 63),
            (16, 63),
            (16, 98),
            (26, 98),
            (39, 96),
            (46, 90),
            (48, 81),
            (47, 72),
            (40, 66),
        ]],
    },
];

options! {
    /// `letters`' options (`space`: Blender's character spacing, which adds half of what it is over 1 of the text's
    /// size between letters).
    Letters { space: f64 = 1.0 }
}

/// `text` in raised block letters, one part: capitals of Blender's text `size` (0.682 of it high), standing
/// `depth` proud of a face at game depth `at.2` and facing the front, the word centred on game (x, height)
/// `at.0`, `at.1` as Blender centres a line of text. For the kitchen's `_letters`, a name plate in metal letters,
/// which Blender set in its built-in font: the kernel has no font, and draws the letters the families write (`A`,
/// `E`, `I`, `M`, `R`) on a grid, each where Blender's stands and as heavy, their curves in straight strokes.
#[allow(clippy::too_many_arguments)] // (as the scripts called it)
pub fn letters(
    frame: &GameFrame,
    name: impl AsRef<str>,
    text: &str,
    size: f64,
    depth: f64,
    at: (impl Coord, impl Coord, impl Coord),
    material: Mat,
    o: Letters,
) -> Obj {
    let name = name.as_ref();
    let glyphs: Vec<&Glyph> = text
        .chars()
        .map(|ch| {
            GLYPHS.iter().find(|g| g.letter == ch).unwrap_or_else(|| {
                panic!("{name}: the kit's letters are A, E, I, M and R, the ones the families write: {ch:?} is not drawn")
            })
        })
        .collect();
    assert!(!glyphs.is_empty(), "{name}: a word has a letter");
    let unit = GRID * size;
    // (Blender's line: each letter's width and the extra space after it, but for the last one's)
    let extra = (0.5 * o.space - 0.5) * size;
    let width = glyphs.iter().map(|g| g.advance as f64 * unit).sum::<f64>()
        + extra * (glyphs.len() - 1) as f64;
    let middle = frame.at(at.0, at.1.f(), at.2.f() + depth / 2.0);
    let mut pen = -width / 2.0;
    let mut word: Option<Obj> = None;
    for g in glyphs {
        // (drawn in Blender's x, which runs to the right seen from the front: the word reads from there)
        let place = |p: &(i32, i32)| {
            (
                pen + (g.lead + p.0) as f64 * unit,
                p.1 as f64 * unit - BASELINE * size,
            )
        };
        let outline: Vec<(f64, f64)> = g.outline.iter().map(place).collect();
        let holes: Vec<Vec<(f64, f64)>> = g
            .holes
            .iter()
            .map(|h| h.iter().map(place).collect())
            .collect();
        let letter = forms::extrude(
            name,
            &outline,
            depth,
            Extrude {
                at: middle,
                material: Some(material),
                holes: Holes(holes),
                plane: Plane::XZ,
                ..Extrude::default()
            },
        );
        pen += g.advance as f64 * unit + extra;
        // the letters are one part, as Blender's text is one mesh
        match word {
            None => word = Some(letter),
            Some(word) => {
                let mesh = letter.read(|l| l.mesh.clone());
                word.edit(|w| crate::parts::append(&mut w.mesh, &mesh));
                letter.remove();
            }
        }
    }
    word.unwrap()
}

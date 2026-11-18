//! The trees' leaf-card atlas (`cargo tex leafcards`): the texture every broadleaved tree's leaf cards are cut
//! from (the `leafcards` finish; artkit's `botany::cards`). Leaves are built with the kernel's botany
//! (`cut_leaf`, `outline`, `twig`), lit a point at a time by a sun and a sky with the leaves' own shadows on one
//! another (rays against the kernel's `Bvh`), and drawn from above with the kernel's rasteriser, whose samples
//! give the cut-out's alpha. Pale and nearly grey: a card's colour tints it. The leaves' colour is bled out into
//! the clear texels round them, so no mipmap mixes in a dark fringe. Nothing downloaded; the same source gives the
//! same bytes.

use artkit::botany::{
    ATLAS, ATLAS_PX, CutLeaf, OutlineOf, Palmate, Picture as Pictured, Sinuate, TILE_PX, Twig,
    cut_leaf, outline, palmate, sinuate, tile_px, twig,
};
use artkit::bvh::Bvh;
use artkit::materials::Mat;
use artkit::math::{PI, TAU, V3, cos, degrees, pow, sin, sqrt, v3};
use artkit::rand::Random;
use artkit::raster::{self, Camera, Look, Picture, Projection, Shading, Surface};
use artkit::scene::{self, Obj};
use std::path::Path;

/// A leaf's colour: linear RGB.
type Tone = [f32; 3];

/// What a build's parts wear until they are drawn (their colours are kept beside them: `Sketch`).
const BLANK: Mat = Mat::new("leaf", "#FFFFFF", "matte");

/// Where the sun stands (towards it; Blender's axes, z up), how strong it is on a face turned to it, how wide
/// (radians: its shadows' soft edge) and the sky's light on a face nothing hides.
const SUN: (f64, f64, f64) = (0.75, 0.4, 1.0);
const SUN_STRENGTH: f32 = 0.62;
const SUN_WIDE: f64 = 0.07;
const SKY: f32 = 0.34;
/// How much of the sky's light a point loses where the leaves over it hide all of the sky.
const SKY_HIDDEN: f32 = 0.5;
/// Rays a point: towards the sun's disc, and into the sky over it.
const SUN_RAYS: usize = 6;
const SKY_RAYS: usize = 8;
/// A face is lit at points no farther apart than this share of its tile.
const FINE: f64 = 0.025;

/// A point of a face being lit: where, its normal, its light and how much of the sun's disc it sees.
type Lit = (V3, V3, (f32, f32));

/// The parts of a tile in the making, each with its colour.
#[derive(Default)]
struct Sketch {
    /// (the part, its colour, how glossy it is: the sun's glint on it, of the sun's light)
    parts: Vec<(Obj, Tone, f32)>,
}

impl Sketch {
    fn new() -> Sketch {
        scene::reset();
        Sketch::default()
    }

    fn add(&mut self, obj: Obj, tone: Tone, gloss: f32) {
        self.parts.push((obj, tone, gloss));
    }

    /// The parts as triangles in the world: (its three corners, its part).
    fn triangles(&self) -> Vec<([V3; 3], usize)> {
        let mut out = Vec::new();
        for (i, (obj, _, _)) in self.parts.iter().enumerate() {
            let points = obj.world_points();
            for t in obj.read(|o| o.mesh.triangle_verts()) {
                out.push((t.map(|v| points[v as usize]), i));
            }
        }
        out
    }
}

/// `n` directions spread evenly over a cap of the unit sphere round +z, out to `wide` radians from it, the
/// cap's rim weighted as a lit face sees it when `cosine` (more near the pole).
fn cap(n: usize, wide: f64, cosine: bool) -> Vec<[f64; 3]> {
    let golden = PI * (3.0 - sqrt(5.0));
    (0..n)
        .map(|i| {
            let u = (i as f64 + 0.5) / n as f64;
            let (s, c) = if cosine {
                // (the sine of the angle from the pole runs as the square root: Malley's way)
                let s = sqrt(u) * sin(wide);
                (s, sqrt(1.0 - s * s))
            } else {
                let c = 1.0 - u * (1.0 - cos(wide));
                (sqrt(1.0 - c * c), c)
            };
            let a = golden * i as f64;
            [s * cos(a), s * sin(a), c]
        })
        .collect()
}

/// `d` (round +z) turned to lie round `axis`.
fn about(axis: V3, d: [f64; 3]) -> V3 {
    let u = axis.orthogonal().normalized();
    let w = axis.cross(u);
    (u * d[0] + w * d[1] + axis * d[2]).normalized()
}

/// The triangles lit, as the rasteriser's meshes (one a part, in its colour): each cut until its sides are
/// `fine` metres or less, each corner's light worked out there, on the face's side of the part: the sun's
/// where rays to its disc go free, by how the face is turned to it, and the sky's where rays over the face go
/// free. A part's faces within 70° of one another round a point shade as one surface there.
fn lit(tris: &[([V3; 3], usize)], tones: &[(Tone, f32)], fine: f64) -> Vec<raster::Mesh> {
    let bvh = Bvh::new(tris.iter().map(|(t, _)| *t).collect());
    let sun = V3::from(SUN).normalized();
    // (a glint is where a face mirrors the sun to the eye above)
    let glint = (sun + v3(0, 0, 1)).normalized();
    let to_sun: Vec<V3> = cap(SUN_RAYS, SUN_WIDE, false)
        .into_iter()
        .map(|d| about(sun, d))
        .collect();
    let over = cap(SKY_RAYS, PI / 2.0, true);
    let face = |t: &[V3; 3]| (t[1] - t[0]).cross(t[2] - t[0]);
    // each part's points: the faces round each (by its place, to the bit)
    let mut round: std::collections::BTreeMap<(usize, [u32; 3]), Vec<V3>> = Default::default();
    let key = |part: usize, p: V3| (part, [p.x.to_bits(), p.y.to_bits(), p.z.to_bits()]);
    for (t, part) in tris {
        let n = face(t);
        for &p in t {
            round.entry(key(*part, p)).or_default().push(n);
        }
    }
    let smooth = cos(70.0_f64.to_radians());
    let mut lights: std::collections::BTreeMap<[u32; 6], (f32, f32)> = Default::default();
    // (a part casts no shadow on itself: a leaf is a thin closed blade, and Blender left some of its faces looking
    // into it; a ray goes on through its own part)
    let light = |part: usize, p: V3, n: V3| -> (f32, f32) {
        let free = |d: V3| {
            let mut from = p + n * 0.0015;
            for _ in 0..8 {
                match bvh.ray_cast(from, d, 2.0) {
                    None => return true,
                    Some((hit, _)) if tris[hit as usize].1 != part => return false,
                    Some((_, far)) => from += d * (far as f64 + 0.0005),
                }
            }
            true
        };
        let facing = (n.dot(sun) as f32).max(0.0);
        let mut direct = 1.0;
        if facing > 0.0 {
            direct = to_sun.iter().filter(|d| free(**d)).count() as f32 / SUN_RAYS as f32;
        }
        let open = over.iter().filter(|d| free(about(n, **d))).count() as f32 / SKY_RAYS as f32;
        let shine = tones[part].1 * pow((n.dot(glint)).max(0.0), 40.0) as f32;
        (
            SKY * (1.0 - SKY_HIDDEN * (1.0 - open)) + SUN_STRENGTH * (facing + shine) * direct,
            direct,
        )
    };
    let mut meshes: Vec<raster::Mesh> = tones
        .iter()
        .map(|&(colour, _)| raster::Mesh {
            surface: Surface {
                colour,
                two_sided: true,
                ..Surface::default()
            },
            ..raster::Mesh::default()
        })
        .collect();
    for (t, part) in tris {
        let flat = face(t);
        if flat.length() < 1e-12 {
            continue;
        }
        let shade =
            |lights: &mut std::collections::BTreeMap<[u32; 6], (f32, f32)>, p: V3, n: V3| {
                let seen = [p.x, p.y, p.z, n.x, n.y, n.z].map(f32::to_bits);
                *lights.entry(seen).or_insert_with(|| light(*part, p, n))
            };
        // (seen from above, whichever way it is wound: lit on the side that looks up)
        let up = |n: V3| if n.z < 0.0 { -n } else { n };
        let flat = up(flat.normalized());
        // each corner's normal: the faces round it that lie as this one does, by their size
        let corners = t.map(|p| {
            let mut n = V3::ZERO;
            for &other in &round[&key(*part, p)] {
                if other.length() > 1e-12 && up(other.normalized()).dot(flat) >= smooth {
                    n += up(other);
                }
            }
            (p, n.normalized())
        });
        // First cut in two across its longest side over `fine`, until no side is. A piece no part of which shows from
        // above (its middle and its corners all under something: its own blade's other face, a leaf over it) is
        // left out. The others are lit, and cut on where a shadow's edge crosses a side (its ends see more and
        // less of the sun) down to a quarter of `fine`. A side is cut or kept by its ends alone, so the faces
        // either side of it are cut alike.
        let middle = |a: (V3, V3), b: (V3, V3)| {
            // (the middle of a side is the same from either end)
            let (lo, hi) = if key(0, a.0) < key(0, b.0) {
                (a, b)
            } else {
                (b, a)
            };
            ((lo.0 + hi.0) / 2, (lo.1 + hi.1).normalized())
        };
        let longest = |c: &[V3; 3], over: &dyn Fn(usize, f64) -> bool| {
            let mut cut: Option<(usize, f64)> = None;
            for i in 0..3 {
                let long = (c[i] - c[(i + 1) % 3]).length();
                if over(i, long) && cut.is_none_or(|(_, most)| long > most) {
                    cut = Some((i, long));
                }
            }
            cut.map(|(i, _)| i)
        };
        let mut pieces = vec![corners];
        let mesh = &mut meshes[*part];
        while let Some(c) = pieces.pop() {
            let at = c.map(|c| c.0);
            // (a piece of no size draws nothing, and cut along its own line it would never end)
            if (at[1] - at[0]).cross(at[2] - at[0]).length() < 1e-9 {
                continue;
            }
            if let Some(i) = longest(&at, &|_, long| long > fine) {
                let (a, b, far) = (c[i], c[(i + 1) % 3], c[(i + 2) % 3]);
                let mid = middle(a, b);
                pieces.extend([[a, mid, far], [mid, b, far]]);
                continue;
            }
            let centre = (at[0] + at[1] + at[2]) / 3;
            let shows = |p: V3| {
                bvh.ray_cast(p + v3(0, 0, 0.0003), v3(0, 0, 1), 2.0)
                    .is_none()
            };
            if !(shows(centre) || at.iter().any(|&p| shows(p.lerp(centre, 0.15)))) {
                continue;
            }
            let mut todo: Vec<[Lit; 3]> = vec![c.map(|(p, n)| (p, n, shade(&mut lights, p, n)))];
            while let Some(c) = todo.pop() {
                let at = c.map(|c| c.0);
                if (at[1] - at[0]).cross(at[2] - at[0]).length() < 1e-9 {
                    continue;
                }
                let edge = |i: usize, long: f64| {
                    long > fine / 4.0 && (c[i].2.1 - c[(i + 1) % 3].2.1).abs() > 0.2
                };
                if let Some(i) = longest(&at, &edge) {
                    let (a, b, far) = (c[i], c[(i + 1) % 3], c[(i + 2) % 3]);
                    let (p, n) = middle((a.0, a.1), (b.0, b.1));
                    let mid = (p, n, shade(&mut lights, p, n));
                    todo.extend([[a, mid, far], [mid, b, far]]);
                    continue;
                }
                for (p, _, (light, _)) in c {
                    let at = mesh.positions.len() as u32;
                    // (the rasteriser's axes: y up, the picture's top Blender's +y)
                    mesh.positions.push([p.x, p.z, -p.y]);
                    mesh.shading.push(light);
                    mesh.indices.push(at);
                }
            }
        }
    }
    meshes
}

/// The sketch seen from above: `across` metres round `centre`, `size` pixels a side, clear where nothing is, drawn
/// `stretch` times as wide as it is (lit as it is).
fn draw(
    sketch: &Sketch,
    centre: (f64, f64),
    across: f64,
    size: usize,
    fine: f64,
    stretch: f64,
) -> Picture {
    let tris = sketch.triangles();
    let tones: Vec<(Tone, f32)> = sketch
        .parts
        .iter()
        .map(|(_, tone, gloss)| (*tone, *gloss))
        .collect();
    let mut meshes = lit(&tris, &tones, fine);
    for p in meshes.iter_mut().flat_map(|m| &mut m.positions) {
        p[0] *= stretch as f32;
    }
    let (cx, cy) = ((centre.0 * stretch) as f32, centre.1 as f32);
    let camera = Camera {
        eye: [cx, 5.0, -cy],
        target: [cx, 0.0, -cy],
        up: [0.0, 0.0, -1.0],
        projection: Projection::Orthographic {
            span: across as f32,
        },
        near: 0.1,
        far: 10.0,
    };
    let look = Look {
        background: [0.0; 4],
        // (the light is each point's own, baked: no more is added)
        shading: Shading::Lit {
            sun: [0.0, 1.0, 0.0],
            sun_strength: 0.0,
            sky: 1.0,
            ground: 1.0,
        },
        floor: None,
        samples: 8,
    };
    raster::render(&meshes, &[], &camera, &look, size, size)
}

/// The leaves' colour bled out into the clear texels of `picture` (alpha a half or less), a texel a pass from
/// the texels round it that have one, 64 passes, the rest the leaves' mean: the mean is returned.
fn bleed(picture: &mut Picture) -> [f32; 3] {
    let (w, h) = (picture.width, picture.height);
    let mut known: Vec<bool> = picture
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| p[3] > 127)
        .collect();
    let mut rgb: Vec<[f32; 3]> = picture
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .map(|p| [p[0] as f32, p[1] as f32, p[2] as f32])
        .collect();
    let mut mean = [0.0f32; 3];
    let n = known.iter().filter(|k| **k).count().max(1) as f32;
    for (c, _) in rgb.iter().zip(&known).filter(|(_, k)| **k) {
        mean = [mean[0] + c[0] / n, mean[1] + c[1] / n, mean[2] + c[2] / n];
    }
    for _ in 0..64 {
        let mut grown = Vec::new();
        for y in 0..h {
            for x in 0..w {
                if known[y * w + x] {
                    continue;
                }
                let (mut sum, mut count) = ([0.0f32; 3], 0.0f32);
                for (dx, dy) in [
                    (0, 1),
                    (0, -1),
                    (1, 0),
                    (-1, 0),
                    (1, 1),
                    (1, -1),
                    (-1, 1),
                    (-1, -1),
                ] {
                    let (nx, ny) = (x as i64 + dx, y as i64 + dy);
                    if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                        continue;
                    }
                    let at = ny as usize * w + nx as usize;
                    if known[at] {
                        sum = [
                            sum[0] + rgb[at][0],
                            sum[1] + rgb[at][1],
                            sum[2] + rgb[at][2],
                        ];
                        count += 1.0;
                    }
                }
                if count > 0.0 {
                    grown.push((y * w + x, [sum[0] / count, sum[1] / count, sum[2] / count]));
                }
            }
        }
        if grown.is_empty() {
            break;
        }
        for (at, c) in grown {
            (known[at], rgb[at]) = (true, c);
        }
    }
    for (i, p) in picture.rgba.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        if p[3] <= 127 {
            let c = if known[i] { rgb[i] } else { mean };
            for k in 0..3 {
                p[k] = c[k].round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    mean
}

// ---- the leaves -------------------------------------------------------------------------------------------------

/// The dense wall's share of its tile, across and up.
const DENSE: f64 = 0.8;
/// A cluster keeps within this of its tile's middle, of half the tile: no leaf crosses into the next tile, nor
/// under a card's edge.
const REACH: f64 = 0.925;
/// The covered texels' mean luminance (linear) in every cluster's tile, and every texel's in a dense one: the first
/// atlas's, which the trees' card and core tones were matched to their far versions with.
const CLUSTER_LIGHT: f32 = 0.44;
const DENSE_LIGHT: f32 = 0.335;

/// A matte leaf colour: nearly grey (`value`), a little yellow (`warm` > 0) or blue-green (< 0).
fn tone(value: f64, warm: f64) -> Tone {
    [
        value * (1.0 + 0.10 * warm),
        value,
        value * (1.0 - 0.16 * warm - 0.04),
    ]
    .map(|c| c.min(1.0) as f32)
}

/// A leaf's colour `depth` (0..1) down in its cluster: three depths, the top the lightest, five tones each.
fn leaf_tone(rng: &mut Random, depth: f64) -> Tone {
    const TONES: [(f64, f64); 5] = [
        (0.8, 0.0),
        (0.7, 0.35),
        (0.74, -0.3),
        (0.86, 0.5),
        (0.66, -0.6),
    ];
    let k = [1.0, 0.76, 0.56][2.min((depth * 3.0) as usize)];
    let (value, warm) = TONES[rng.randrange(TONES.len() as i64) as usize];
    tone(value * k, warm)
}

/// How a blade is held: its tilt up (degrees), how it bends down along its length, how its halves lift into a V,
/// its turn about its midrib by the tip (degrees either way), and its gloss.
#[derive(Clone, Copy)]
struct Hold {
    tilt: (f64, f64),
    droop: (f64, f64),
    fold: (f64, f64),
    twist: f64,
    gloss: f32,
}

#[allow(clippy::too_many_arguments)]
/// A blade cut from `edge` (its right half, `cut_leaf`'s), heading `heading` (radians) from `at`.
fn blade(
    sketch: &mut Sketch,
    rng: &mut Random,
    edge: &[(f64, f64)],
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    tone: Tone,
    hold: Hold,
) {
    let tilt = rng.uniform(hold.tilt.0, hold.tilt.1);
    let droop = rng.uniform(hold.droop.0, hold.droop.1);
    let fold = rng.uniform(hold.fold.0, hold.fold.1);
    let twist = rng.uniform(-hold.twist, hold.twist);
    let o = CutLeaf {
        droop,
        fold,
        thick: 0.004_f64.min(length * 0.03),
        ribs: 5,
        twist,
        left: None,
    };
    sketch.add(
        cut_leaf(
            "leaf",
            at,
            90.0 - degrees(heading),
            tilt,
            length,
            edge,
            BLANK,
            o,
        ),
        tone,
        hold.gloss,
    );
}

/// A species' leaf: (sketch, random numbers, where, heading, length, how deep in its cluster 0..1).
type Leaf = fn(&mut Sketch, &mut Random, (f64, f64, f64), f64, f64, f64);

/// An English oak's: lobed, four or five rounded lobes a side, widest above the middle.
fn oak_leaf(
    sketch: &mut Sketch,
    rng: &mut Random,
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    depth: f64,
) {
    let width = length * rng.uniform(0.56, 0.66);
    let lobes = sinuate(
        rng.uniform(4.0, 5.0),
        Sinuate {
            depth: rng.uniform(0.4, 0.52),
        },
    );
    let edge = outline(
        lobes,
        width,
        OutlineOf {
            n: 30,
            ..OutlineOf::default()
        },
    );
    let tone = leaf_tone(rng, depth);
    let hold = Hold {
        tilt: (-6.0, 22.0),
        droop: (0.05, 0.3),
        fold: (0.15, 0.35),
        twist: 25.0,
        gloss: 0.0,
    };
    blade(sketch, rng, &edge, at, heading, length, tone, hold);
}

/// A Norway maple's: five pointed lobes with a tooth to each side, as wide as long, held flat.
fn maple_leaf(
    sketch: &mut Sketch,
    rng: &mut Random,
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    depth: f64,
) {
    let o = Palmate {
        cut: rng.uniform(0.36, 0.44),
        bulge: 0.075,
        teeth: 1,
        spread: rng.uniform(98.0, 108.0),
        fall: 0.84,
    };
    let edge = palmate(5, length, o);
    let tone = leaf_tone(rng, depth);
    let hold = Hold {
        tilt: (-4.0, 14.0),
        droop: (0.05, 0.22),
        fold: (0.06, 0.18),
        twist: 12.0,
        gloss: 0.0,
    };
    blade(sketch, rng, &edge, at, heading, length, tone, hold);
}

/// A silver birch's: small, triangular with a long point, toothed.
fn birch_leaf(
    sketch: &mut Sketch,
    rng: &mut Random,
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    depth: f64,
) {
    let width = length * rng.uniform(0.72, 0.84);
    let edge = outline(
        "deltoid",
        width,
        OutlineOf {
            n: 18,
            teeth: 0.2,
            ..OutlineOf::default()
        },
    );
    let tone = leaf_tone(rng, depth);
    let hold = Hold {
        tilt: (-10.0, 25.0),
        droop: (0.05, 0.3),
        fold: (0.2, 0.45),
        twist: 35.0,
        gloss: 0.0,
    };
    blade(sketch, rng, &edge, at, heading, length, tone, hold);
}

/// An apple's: ovate, finely toothed, its halves folded up.
fn apple_leaf(
    sketch: &mut Sketch,
    rng: &mut Random,
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    depth: f64,
) {
    let width = length * rng.uniform(0.58, 0.68);
    let edge = outline(
        "ovate",
        width,
        OutlineOf {
            n: 20,
            teeth: 0.1,
            ..OutlineOf::default()
        },
    );
    let tone = leaf_tone(rng, depth);
    let hold = Hold {
        tilt: (-6.0, 24.0),
        droop: (0.1, 0.35),
        fold: (0.3, 0.55),
        twist: 25.0,
        gloss: 0.0,
    };
    blade(sketch, rng, &edge, at, heading, length, tone, hold);
}

/// A Japanese maple's: seven narrow pointed lobes cut nearly to the stalk.
fn japanese_maple_leaf(
    sketch: &mut Sketch,
    rng: &mut Random,
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    depth: f64,
) {
    let o = Palmate {
        cut: rng.uniform(0.7, 0.78),
        bulge: 0.055,
        teeth: 0,
        spread: rng.uniform(118.0, 130.0),
        fall: 0.87,
    };
    let edge = palmate(7, length, o);
    let tone = leaf_tone(rng, depth);
    let hold = Hold {
        tilt: (-4.0, 12.0),
        droop: (0.05, 0.25),
        fold: (0.04, 0.14),
        twist: 10.0,
        gloss: 0.0,
    };
    blade(sketch, rng, &edge, at, heading, length, tone, hold);
}

/// A saucer magnolia's: large, widest above the middle, a wedge to the stalk.
fn magnolia_leaf(
    sketch: &mut Sketch,
    rng: &mut Random,
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    depth: f64,
) {
    let width = length * rng.uniform(0.48, 0.58);
    let wave = rng.uniform(0.03, 0.07);
    let waves = rng.choice(&[3.0, 5.0]);
    let edge = outline(
        "obovate",
        width,
        OutlineOf {
            n: 14,
            wave,
            waves,
            teeth: 0.0,
        },
    );
    let tone = leaf_tone(rng, depth);
    let hold = Hold {
        tilt: (-6.0, 20.0),
        droop: (0.1, 0.35),
        fold: (0.2, 0.4),
        twist: 20.0,
        gloss: 0.12,
    };
    blade(sketch, rng, &edge, at, heading, length, tone, hold);
}

/// A weeping willow's: long and narrow.
fn willow_leaf(
    sketch: &mut Sketch,
    rng: &mut Random,
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    depth: f64,
) {
    let width = length * rng.uniform(0.2, 0.25);
    let edge = outline(
        "lance",
        width,
        OutlineOf {
            n: 10,
            ..OutlineOf::default()
        },
    );
    let tone = leaf_tone(rng, depth);
    let hold = Hold {
        tilt: (-8.0, 20.0),
        droop: (0.1, 0.5),
        fold: (0.2, 0.4),
        twist: 40.0,
        gloss: 0.0,
    };
    blade(sketch, rng, &edge, at, heading, length, tone, hold);
}

/// A lemon's: elliptic, pointed, glossy.
fn lemon_leaf(
    sketch: &mut Sketch,
    rng: &mut Random,
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    depth: f64,
) {
    let width = length * rng.uniform(0.42, 0.5);
    let edge = outline(
        "elliptic",
        width,
        OutlineOf {
            n: 14,
            wave: 0.03,
            waves: 3.0,
            teeth: 0.0,
        },
    );
    let tone = leaf_tone(rng, depth);
    let hold = Hold {
        tilt: (-6.0, 24.0),
        droop: (0.05, 0.25),
        fold: (0.25, 0.45),
        twist: 22.0,
        gloss: 0.45,
    };
    blade(sketch, rng, &edge, at, heading, length, tone, hold);
}

/// A flowering cherry's young leaf among its blossom: ovate with a long point, folded, bronze. (Its colour is the
/// bronze over the petals' pink: the card's pink tints the whole tile, the petals pale in it.)
fn cherry_leaf(
    sketch: &mut Sketch,
    rng: &mut Random,
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    depth: f64,
) {
    let width = length * rng.uniform(0.42, 0.5);
    let edge = outline(
        "ovate",
        width,
        OutlineOf {
            n: 20,
            teeth: 0.1,
            ..OutlineOf::default()
        },
    );
    let k = (0.9 - 0.3 * depth) as f32 * rng.uniform(0.85, 1.0) as f32;
    let hold = Hold {
        tilt: (0.0, 28.0),
        droop: (0.15, 0.45),
        fold: (0.35, 0.6),
        twist: 25.0,
        gloss: 0.15,
    };
    blade(
        sketch,
        rng,
        &edge,
        at,
        heading,
        length,
        [0.42 * k, 0.5 * k, 0.13 * k],
        hold,
    );
}

/// A double cherry blossom `size` across at `at`: three rings of notched petals, the outer ones lying open, the
/// inner standing, paler outside and a deeper pink at its heart.
fn flower(sketch: &mut Sketch, rng: &mut Random, at: (f64, f64, f64), size: f64, depth: f64) {
    let turn = rng.uniform(0.0, TAU);
    let k = (1.0 - 0.3 * depth) as f32;
    for (ring, (petals, long, tilt, pink)) in [
        (9, 0.5, (8.0, 26.0), 0.0),
        (7, 0.4, (34.0, 54.0), 0.5),
        (5, 0.28, (58.0, 78.0), 1.0),
    ]
    .into_iter()
    .enumerate()
    {
        for i in 0..petals {
            let heading = turn
                + TAU * (i as f64 + 0.5 * ring as f64 + rng.uniform(-0.2, 0.2)) / petals as f64;
            let length = size * long * rng.uniform(0.9, 1.1);
            let wave = rng.uniform(0.1, 0.2);
            let edge = outline(
                "petal",
                length * 0.95,
                OutlineOf {
                    n: 8,
                    wave,
                    waves: 3.0,
                    teeth: 0.0,
                },
            );
            let pale = rng.uniform(0.86, 0.98) as f32 * k;
            let tone = [pale, pale * (1.0 - 0.2 * pink), pale * (1.0 - 0.12 * pink)];
            let hold = Hold {
                tilt,
                droop: (0.0, 0.2),
                fold: (0.1, 0.3),
                twist: 20.0,
                gloss: 0.0,
            };
            blade(
                sketch,
                rng,
                &edge,
                (at.0, at.1, at.2 + 0.004 * ring as f64),
                heading,
                length,
                tone,
                hold,
            );
        }
    }
}

/// The longest a leaf from `p` heading `heading` may be (up to `length`) and stay within `reach` (across, up)
/// of the tile's middle, its tip and both its shoulders (`wide` of its length across).
fn fits(p: (f64, f64), heading: f64, mut length: f64, wide: f64, reach: (f64, f64)) -> f64 {
    let (dx, dy) = (cos(heading), sin(heading));
    for _ in 0..14 {
        let inside = [
            (1.0, 0.0),
            (0.55, wide / 2.0),
            (0.55, -wide / 2.0),
            (0.0, wide / 3.0),
            (0.0, -wide / 3.0),
        ]
        .iter()
        .all(|(along, side)| {
            let (x, y) = (
                p.0 + (dx * along - dy * side) * length,
                p.1 + (dy * along + dx * side) * length,
            );
            x.abs() < reach.0 && y.abs() < reach.1
        });
        if inside {
            return length;
        }
        length *= 0.88;
    }
    0.0
}

/// How a species' leaves stand in a cluster: on twigs fanning from the tile's middle.
#[derive(Clone, Copy)]
struct Spray {
    leaf: Leaf,
    /// The tile, metres (the species' cards are about as wide).
    across: f64,
    /// A leaf's length, metres, and its width of that (for keeping it in the tile).
    size: (f64, f64),
    wide: f64,
    /// Twigs, how far the first is turned, and each one's length, of the reach.
    twigs: usize,
    turn: f64,
    long: (f64, f64),
    /// A twig's radius at its foot and its tip, metres, and its colour.
    wood: (f64, f64, Tone),
    /// Leaves along each twig (from `from` of the way out), in pairs when `opposite`, standing `lean` radians
    /// off the twig at its foot and at its tip; `tip` more fanned round its end; `fill` lying under them all.
    per: usize,
    from: f64,
    opposite: bool,
    lean: (f64, f64),
    tip: usize,
    fill: usize,
    /// How far out the leaves lying under lie, of the reach.
    fill_out: f64,
    /// Leaves laid over the twigs' foot last.
    top: usize,
    /// How wide the twigs fan (radians: all round, or to one side as a hanging spray's) and how far back from the
    /// tile's middle their foot is, of the reach.
    fan: f64,
    foot: f64,
}

/// A cluster in a tile: the leaves lying under it first, then each twig's, the later laid over the earlier.
fn spray(sketch: &mut Sketch, rng: &mut Random, s: Spray) {
    let reach = s.across / 2.0 * REACH;
    let total = s.fill + s.top + s.twigs * (s.per * if s.opposite { 2 } else { 1 } + s.tip);
    let mut n = 0;
    let mut add =
        |sketch: &mut Sketch, rng: &mut Random, p: (f64, f64), heading: f64, size: f64| {
            let size = fits(p, heading, size, s.wide, (reach, reach));
            if size >= s.size.0 * 0.5 {
                let at = (p.0, p.1, 0.012 + 0.22 * s.across * n as f64 / total as f64);
                (s.leaf)(
                    sketch,
                    rng,
                    at,
                    heading,
                    size,
                    1.0 - n as f64 / total as f64,
                );
            }
            n += 1;
        };
    for _ in 0..s.fill {
        let a = rng.uniform(0.0, TAU);
        let r = reach * rng.uniform(0.0, s.fill_out);
        let heading = a + rng.uniform(-1.2, 1.2);
        let size = rng.uniform(s.size.0, s.size.1);
        add(sketch, rng, (r * cos(a), r * sin(a)), heading, size);
    }
    // the twigs' foot: the tile's middle, or back from it for a spray that fans to one side
    let (fx, fy) = (-s.foot * reach * cos(s.turn), -s.foot * reach * sin(s.turn));
    for t in 0..s.twigs {
        let a = if s.fan >= TAU {
            s.turn + TAU * (t as f64 + rng.uniform(-0.25, 0.25)) / s.twigs as f64
        } else {
            s.turn + s.fan * ((t as f64 + rng.uniform(0.25, 0.75)) / s.twigs as f64 - 0.5)
        };
        // (as long as it may be and end in the tile)
        let mut length = reach * rng.uniform(s.long.0, s.long.1);
        while (fx + length * cos(a)).abs() > 0.94 * reach
            || (fy + length * sin(a)).abs() > 0.94 * reach
        {
            length *= 0.94;
        }
        let bow = rng.uniform(-0.3, 0.3);
        let pts = [
            v3(
                fx + 0.02 * s.across * cos(a),
                fy + 0.02 * s.across * sin(a),
                0.0,
            ),
            v3(
                fx + length * 0.5 * cos(a + bow * 0.5),
                fy + length * 0.5 * sin(a + bow * 0.5),
                0.012 * s.across,
            ),
            v3(fx + length * cos(a), fy + length * sin(a), 0.02 * s.across),
        ];
        sketch.add(
            twig(
                "twig",
                &pts,
                s.wood.0,
                s.wood.1,
                BLANK,
                Twig {
                    verts: 4,
                    ..Twig::default()
                },
            ),
            s.wood.2,
            0.0,
        );
        let along = |f: f64| {
            if f < 0.5 {
                pts[0].lerp(pts[1], f * 2.0)
            } else {
                pts[1].lerp(pts[2], f * 2.0 - 1.0)
            }
        };
        for i in 0..s.per {
            let f = s.from + (1.0 - s.from) * (i as f64 + 0.6) / s.per as f64;
            let p = along(f);
            let lean = s.lean.0 + (s.lean.1 - s.lean.0) * f;
            let sides: &[f64] = if s.opposite {
                &[1.0, -1.0]
            } else if i % 2 != 0 {
                &[-1.0]
            } else {
                &[1.0]
            };
            for side in sides {
                let heading = a + side * lean * rng.uniform(0.75, 1.25) + rng.uniform(-0.2, 0.2);
                let size = rng.uniform(s.size.0, s.size.1);
                add(sketch, rng, (p.x as f64, p.y as f64), heading, size);
            }
        }
        for i in 0..s.tip {
            let p = along(1.0);
            let heading = a
                + (i as f64 - (s.tip as f64 - 1.0) / 2.0) * TAU * 0.62 / s.tip as f64
                + rng.uniform(-0.2, 0.2);
            let size = rng.uniform(s.size.0, s.size.1);
            add(sketch, rng, (p.x as f64, p.y as f64), heading, size);
        }
    }
    for _ in 0..s.top {
        let a = rng.uniform(0.0, TAU);
        let r = reach * rng.uniform(0.0, 0.22);
        let heading = a + rng.uniform(-0.6, 0.6);
        let size = rng.uniform(s.size.0, s.size.1);
        add(
            sketch,
            rng,
            (fx + r * cos(a), fy + r * sin(a)),
            heading,
            size,
        );
    }
}

/// A wall of a species' leaves over `across` metres round the tile's middle (and a little past it), at every angle,
/// a few deep.
fn wall(
    sketch: &mut Sketch,
    rng: &mut Random,
    leaf: Leaf,
    across: f64,
    size: (f64, f64),
    wide: f64,
) {
    let half = across / 2.0 + 0.4 * size.1;
    let mean = (size.0 + size.1) / 2.0;
    let leaves = (2.6 * (2.0 * half) * (2.0 * half) / (mean * mean * wide * 0.62)) as usize;
    for i in 0..leaves {
        let at = (
            rng.uniform(-half, half),
            rng.uniform(-half, half),
            0.3 * across * i as f64 / leaves as f64,
        );
        let heading = rng.uniform(0.0, TAU);
        let length = rng.uniform(size.0, size.1);
        leaf(
            sketch,
            rng,
            at,
            heading,
            length,
            1.0 - i as f64 / leaves as f64,
        );
    }
}

/// A cluster of cherry blossom: bunches of flowers on short stalks along twigs fanning from the tile's middle,
/// a couple of bronze young leaves beside each bunch.
fn blossom(sketch: &mut Sketch, rng: &mut Random, across: f64, twigs: usize, turn: f64) {
    let reach = across / 2.0 * REACH;
    let size = (0.07, 0.085);
    let wood = tone(0.32, 1.4);
    let mut bunches: Vec<(f64, f64, f64)> = vec![(0.0, 0.0, rng.uniform(0.0, TAU))];
    for t in 0..twigs {
        let a = turn + TAU * (t as f64 + rng.uniform(-0.2, 0.2)) / twigs as f64;
        let length = reach * rng.uniform(0.62, 0.8);
        let bow = rng.uniform(-0.25, 0.25);
        let pts = [
            v3(0.0, 0.0, 0.0),
            v3(
                length * 0.5 * cos(a + bow),
                length * 0.5 * sin(a + bow),
                0.008,
            ),
            v3(length * cos(a), length * sin(a), 0.012),
        ];
        sketch.add(
            twig(
                "twig",
                &pts,
                0.007,
                0.004,
                BLANK,
                Twig {
                    verts: 4,
                    ..Twig::default()
                },
            ),
            wood,
            0.0,
        );
        for f in [0.38, 0.7, 1.0] {
            let p = if f < 1.0 {
                pts[0].lerp(pts[2], f) + (pts[1] - pts[0].lerp(pts[2], 0.5)) * 0.8
            } else {
                pts[2]
            };
            bunches.push((p.x as f64, p.y as f64, a));
        }
        // (and one between this twig and the next, on a stalk too short to show)
        let between = a + PI / twigs as f64;
        let r = reach * rng.uniform(0.5, 0.68);
        bunches.push((r * cos(between), r * sin(between), between));
    }
    let total = bunches.len();
    // the leaves first, under the flowers: two to a bunch, pointing out past it
    for (i, &(x, y, a)) in bunches.iter().enumerate() {
        for side in [-1.0, 1.0] {
            let heading = a + side * rng.uniform(0.5, 1.3);
            let length = fits((x, y), heading, rng.uniform(0.1, 0.13), 0.5, (reach, reach));
            if length > 0.06 {
                let depth = 0.6 * rng.random();
                cherry_leaf(
                    sketch,
                    rng,
                    (x, y, 0.01 + 0.002 * i as f64),
                    heading,
                    length,
                    depth,
                );
            }
        }
    }
    for (i, &(x, y, _)) in bunches.iter().enumerate() {
        let flowers = rng.randint(5, 7);
        let off = rng.uniform(0.0, TAU);
        for j in 0..flowers {
            let a = off + TAU * (j as f64 + rng.uniform(-0.2, 0.2)) / flowers as f64;
            let r = rng.uniform(0.45, 0.75) * size.1;
            let across = rng.uniform(size.0, size.1);
            let (fx, fy) = (x + r * cos(a), y + r * sin(a));
            if fx.abs() + across / 2.0 < reach && fy.abs() + across / 2.0 < reach {
                let depth = 1.0 - (i * 6 + j as usize) as f64 / (total * 6) as f64;
                flower(
                    sketch,
                    rng,
                    (
                        fx,
                        fy,
                        0.03 + 0.012 * (i * 6 + j as usize) as f64 / total as f64,
                    ),
                    across,
                    depth * 0.8,
                );
            }
        }
    }
}

/// A wall of cherry blossom with a few young leaves in it.
fn blossom_wall(sketch: &mut Sketch, rng: &mut Random, across: f64) {
    let half = across / 2.0 + 0.04;
    let flowers = (2.2 * (2.0 * half) * (2.0 * half) / (0.078 * 0.078 * 0.8)) as usize;
    for i in 0..flowers {
        let at = (
            rng.uniform(-half, half),
            rng.uniform(-half, half),
            0.3 * across * i as f64 / flowers as f64,
        );
        let depth = 1.0 - i as f64 / flowers as f64;
        if i % 6 == 0 {
            let heading = rng.uniform(0.0, TAU);
            let length = rng.uniform(0.1, 0.13);
            cherry_leaf(sketch, rng, at, heading, length, depth);
        } else {
            let size = rng.uniform(0.07, 0.085);
            flower(sketch, rng, at, size, depth);
        }
    }
}

/// A weeping willow's hanging shoots as its strands' cards show them, `wide` by `tall` metres (the tile's top is
/// up): slender shoots from the top edge to the bottom, swaying a little, narrow leaves down both sides of each,
/// pointing down and out.
fn strands(sketch: &mut Sketch, rng: &mut Random, wide: f64, tall: f64, shoots: usize) {
    let reach = (wide / 2.0 * REACH, tall / 2.0 * REACH);
    let wood = tone(0.7, 1.6);
    for s in 0..shoots {
        let x0 = wide * 0.7 * ((s as f64 + 0.5) / shoots as f64 - 0.5) + rng.uniform(-0.01, 0.01);
        let sway = rng.uniform(0.012, 0.028) * if s % 2 != 0 { -1.0 } else { 1.0 };
        let phase = rng.uniform(0.0, TAU);
        let x = |f: f64| x0 + sway * sin(phase + f * 4.2);
        // (a shoot ends short of the tile's top and bottom: the cards down a strand lap over one another)
        let pts: Vec<V3> = (0..=6)
            .map(|i| {
                let f = i as f64 / 6.0;
                v3(x(f), reach.1 * (1.0 - 2.0 * f), 0.02 + 0.01 * s as f64)
            })
            .collect();
        sketch.add(
            twig(
                "shoot",
                &pts,
                0.0035,
                0.002,
                BLANK,
                Twig {
                    verts: 4,
                    ..Twig::default()
                },
            ),
            wood,
            0.0,
        );
        let leaves = (tall / 0.03) as usize;
        for i in 0..leaves {
            let f = (i as f64 + rng.uniform(0.2, 0.8)) / leaves as f64;
            let side = if i % 2 != 0 { -1.0 } else { 1.0 };
            let p = (x(f), reach.1 * (1.0 - 2.0 * f));
            let heading = -PI / 2.0 + side * rng.uniform(0.2, 0.6);
            let length = fits(p, heading, rng.uniform(0.085, 0.12), 0.2, reach);
            if length > 0.05 {
                let depth = 1.0 - (s * leaves + i) as f64 / (shoots * leaves) as f64;
                willow_leaf(
                    sketch,
                    rng,
                    (p.0, p.1, 0.02 + 0.01 * s as f64 + 0.004),
                    heading,
                    length,
                    depth * 0.8,
                );
            }
        }
    }
}

// ---- the tiles --------------------------------------------------------------------------------------------------

/// A species' cards' mean width, metres: what its cluster tiles show (`cargo art trees`' models, measured), and the
/// size and width of its leaves there. A little larger than life where the leaf's shape is the species' mark and
/// would otherwise be a few pixels at the play camera.
struct Kind {
    name: &'static str,
    across: f64,
    leaf: Leaf,
    size: (f64, f64),
    wide: f64,
}

const KINDS: [Kind; 9] = [
    Kind {
        name: "oak",
        across: 0.62,
        leaf: oak_leaf,
        size: (0.135, 0.17),
        wide: 0.62,
    },
    Kind {
        name: "maple",
        across: 0.67,
        leaf: maple_leaf,
        size: (0.15, 0.18),
        wide: 1.5,
    },
    Kind {
        name: "birch",
        across: 0.49,
        leaf: birch_leaf,
        size: (0.058, 0.072),
        wide: 0.8,
    },
    Kind {
        name: "cherry",
        across: 0.55,
        leaf: cherry_leaf,
        size: (0.1, 0.13),
        wide: 0.5,
    },
    Kind {
        name: "apple",
        across: 0.6,
        leaf: apple_leaf,
        size: (0.095, 0.12),
        wide: 0.65,
    },
    Kind {
        name: "japaneseMaple",
        across: 0.39,
        leaf: japanese_maple_leaf,
        size: (0.085, 0.105),
        wide: 1.7,
    },
    Kind {
        name: "magnolia",
        across: 0.54,
        leaf: magnolia_leaf,
        size: (0.15, 0.185),
        wide: 0.55,
    },
    Kind {
        name: "willow",
        across: 0.74,
        leaf: willow_leaf,
        size: (0.11, 0.145),
        wide: 0.2,
    },
    Kind {
        name: "lemon",
        across: 0.21,
        leaf: lemon_leaf,
        size: (0.075, 0.09),
        wide: 0.48,
    },
];

/// A willow strand's card, metres (crates/art/nature/src/trees/willow.rs: a strand 0.2 wide, its cards half as wide
/// again and 0.75 tall).
const STRAND: (f64, f64) = (0.3, 0.75);

/// Tile `i` of the atlas (`ATLAS`'s order) drawn: `TILE_PX` a side.
fn tile(i: usize) -> Picture {
    let (name, _, pictured) = ATLAS[i];
    let kind = KINDS
        .iter()
        .find(|k| k.name == name)
        .unwrap_or_else(|| panic!("no leaves for {name}"));
    // (which of its species' pictures of the kind it is)
    let nth = (0..i)
        .filter(|&j| ATLAS[j].0 == name && ATLAS[j].2 == pictured)
        .count();
    let mut sketch = Sketch::new();
    let mut rng = Random::new(
        (1000 * (i / 4 + 1) + nth) as i128 + if pictured == Pictured::Core { 500 } else { 0 },
    );
    let fine = FINE * kind.across;
    let wood = tone(0.4, 1.2);
    let base = Spray {
        leaf: kind.leaf,
        across: kind.across,
        size: kind.size,
        wide: kind.wide,
        twigs: 6,
        turn: [0.0, 0.6, 2.2][nth % 3],
        long: (0.6, 0.85),
        wood: (0.006, 0.003, wood),
        per: 5,
        from: 0.0,
        opposite: false,
        lean: (1.1, 0.35),
        tip: 0,
        fill: 10,
        fill_out: 0.7,
        top: 3,
        fan: TAU,
        foot: 0.0,
    };
    match pictured {
        Pictured::Core => {
            let across = kind.across * 1.08;
            match name {
                "cherry" => blossom_wall(&mut sketch, &mut rng, across),
                _ => wall(
                    &mut sketch,
                    &mut rng,
                    kind.leaf,
                    across,
                    kind.size,
                    kind.wide.min(0.9),
                ),
            }
            let d = (TILE_PX as f64 * DENSE) as usize;
            dense(draw(&sketch, (0.0, 0.0), across, d, fine, 1.0))
        }
        Pictured::Strand => {
            strands(&mut sketch, &mut rng, STRAND.0, STRAND.1, 5 + nth % 2);
            cut_out(draw(
                &sketch,
                (0.0, 0.0),
                STRAND.1,
                TILE_PX,
                fine,
                STRAND.1 / STRAND.0,
            ))
        }
        Pictured::Blossom => {
            blossom(
                &mut sketch,
                &mut rng,
                kind.across,
                4 + nth % 2,
                [0.0, 0.7, 2.0][nth % 3],
            );
            cut_out(draw(&sketch, (0.0, 0.0), kind.across, TILE_PX, fine, 1.0))
        }
        Pictured::Cluster => {
            let s = match name {
                // bunches of leaves at its twigs' ends and along them
                "oak" => Spray {
                    twigs: 5 + nth % 2,
                    per: 3,
                    from: 0.15,
                    tip: 4,
                    fill: 12,
                    lean: (1.2, 0.7),
                    long: (0.55, 0.8),
                    ..base
                },
                // pairs of broad leaves held flat, in layers
                "maple" => Spray {
                    twigs: 4 + nth % 2,
                    per: 2,
                    from: 0.3,
                    opposite: true,
                    tip: 1,
                    fill: 4,
                    top: 1,
                    lean: (1.15, 0.8),
                    long: (0.5, 0.72),
                    ..base
                },
                // small leaves down slender dark twigs that hang to one side, the sky between them
                "birch" => Spray {
                    twigs: 9 + nth % 2,
                    per: 10,
                    from: 0.06,
                    tip: 1,
                    fill: 30,
                    fill_out: 0.9,
                    top: 0,
                    fan: 1.9,
                    foot: 0.82,
                    lean: (1.3, 0.8),
                    long: (1.2, 1.8),
                    wood: (0.0035, 0.0015, tone(0.22, 0.8)),
                    ..base
                },
                // rosettes on short spurs
                "apple" => Spray {
                    twigs: 6 + nth % 2,
                    per: 3,
                    from: 0.2,
                    tip: 4,
                    fill: 14,
                    lean: (1.2, 0.6),
                    ..base
                },
                // pairs of fine-fingered leaves in flat layers
                "japaneseMaple" => Spray {
                    twigs: 5 + nth % 2,
                    per: 2,
                    from: 0.35,
                    opposite: true,
                    tip: 1,
                    fill: 7,
                    top: 2,
                    lean: (1.0, 0.7),
                    long: (0.5, 0.75),
                    wood: (0.003, 0.0015, tone(0.3, 1.5)),
                    ..base
                },
                // a whorl of big leaves round each twig's end
                "magnolia" => Spray {
                    twigs: 4 + nth % 2,
                    per: 1,
                    from: 0.3,
                    tip: 4,
                    fill: 7,
                    fill_out: 0.5,
                    lean: (1.2, 0.8),
                    long: (0.35, 0.58),
                    wood: (0.008, 0.005, wood),
                    ..base
                },
                // long narrow leaves down arching golden shoots
                "willow" => Spray {
                    twigs: 14 + nth % 2,
                    per: 10,
                    from: 0.08,
                    tip: 1,
                    fill: 34,
                    fill_out: 0.85,
                    top: 6,
                    lean: (0.8, 0.45),
                    long: (0.7, 0.98),
                    wood: (0.004, 0.002, tone(0.7, 1.6)),
                    ..base
                },
                // a few glossy leaves on short stems
                "lemon" => Spray {
                    twigs: 4 + nth % 2,
                    per: 3,
                    from: 0.15,
                    tip: 1,
                    fill: 5,
                    fill_out: 0.6,
                    top: 2,
                    lean: (1.0, 0.6),
                    long: (0.45, 0.65),
                    wood: (0.004, 0.003, tone(0.55, 0.9)),
                    ..base
                },
                _ => base,
            };
            spray(&mut sketch, &mut rng, s);
            cut_out(draw(&sketch, (0.0, 0.0), kind.across, TILE_PX, fine, 1.0))
        }
    }
}

/// A texel's luminance, linear.
fn luminance(p: &[u8; 4]) -> f32 {
    let c = raster::linear([p[0], p[1], p[2]]);
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// `picture`'s texels `keep` says, lit by `k`.
fn relight(picture: &mut Picture, k: f32) {
    for p in picture.rgba.as_chunks_mut::<4>().0 {
        let c = raster::linear([p[0], p[1], p[2]]);
        for i in 0..3 {
            p[i] = encode(c[i] * k);
        }
    }
}

/// Linear light as an sRGB byte.
fn encode(c: f32) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let v = if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * pow(c as f64, 1.0 / 2.4) as f32 - 0.055
    };
    (v * 255.0).round() as u8
}

/// A cluster as drawn, as its tile: its covered texels' mean luminance `CLUSTER_LIGHT`, its colour bled out into the
/// clear texels round it. A cluster that reaches its tile's edge is a fault in its drawing.
fn cut_out(mut picture: Picture) -> Picture {
    let (w, h) = (picture.width, picture.height);
    let covered: Vec<&[u8; 4]> = picture
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[3] > 127)
        .collect();
    let mean = covered.iter().map(|p| luminance(p)).sum::<f32>() / covered.len().max(1) as f32;
    relight(&mut picture, CLUSTER_LIGHT / mean);
    for y in 0..h {
        for x in 0..w {
            let edge = x < 2 || y < 2 || x >= w - 2 || y >= h - 2;
            assert!(
                !edge || picture.at(x, y)[3] == 0,
                "a cluster reaches its tile's edge at {x}, {y}"
            );
        }
    }
    bleed(&mut picture);
    picture
}

/// A wall of leaves as drawn (`DENSE` of a tile across), as its tile: opaque in the tile's middle, its gaps the
/// dark of a crown's inside, every texel's mean luminance `DENSE_LIGHT`; round it a clear gutter in its mean colour.
fn dense(wall: Picture) -> Picture {
    let d = wall.width;
    let o = (TILE_PX - d) / 2;
    let covered: Vec<&[u8; 4]> = wall
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[3] > 127)
        .collect();
    let n = covered.len().max(1) as f32;
    let mean = covered.iter().fold([0.0f32; 3], |s, p| {
        [
            s[0] + p[0] as f32 / n,
            s[1] + p[1] as f32 / n,
            s[2] + p[2] as f32 / n,
        ]
    });
    // (mixed as the bytes are, as the first atlas was)
    let mut solid = Picture::filled(d, d, [0, 0, 0, 255]);
    for (to, p) in solid
        .rgba
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(wall.rgba.as_chunks::<4>().0)
    {
        let a = p[3] as f32 / 255.0;
        for k in 0..3 {
            to[k] = (p[k] as f32 * a + mean[k] * 0.4 * (1.0 - a))
                .round()
                .clamp(0.0, 255.0) as u8;
        }
    }
    let texels = solid.rgba.as_chunks::<4>().0;
    let light = texels.iter().map(luminance).sum::<f32>() / texels.len() as f32;
    relight(&mut solid, DENSE_LIGHT / light);
    let texels = solid.rgba.as_chunks::<4>().0;
    let m = texels.len() as f32;
    let all = texels.iter().fold([0.0f32; 3], |s, p| {
        [
            s[0] + p[0] as f32 / m,
            s[1] + p[1] as f32 / m,
            s[2] + p[2] as f32 / m,
        ]
    });
    let mut out = Picture::filled(
        TILE_PX,
        TILE_PX,
        [
            all[0].round() as u8,
            all[1].round() as u8,
            all[2].round() as u8,
            0,
        ],
    );
    out.paste(&solid, o, o);
    out
}

/// The atlas: `ATLAS_PX` a side, a tile of `ATLAS` a cell (each drawn on a thread of its own, from its own random
/// numbers), clear between and round them.
pub fn atlas() -> Picture {
    let tiles: Vec<Picture> = std::thread::scope(|scope| {
        let drawing: Vec<_> = (0..ATLAS.len())
            .map(|i| scope.spawn(move || tile(i)))
            .collect();
        drawing
            .into_iter()
            .map(|t| t.join().unwrap_or_else(|e| std::panic::resume_unwind(e)))
            .collect()
    });
    // (the clear texels of the edge and of the cells take the tiles' mean colour)
    let n = (tiles.len() * TILE_PX * TILE_PX) as f32;
    let mut mean = [0.0f32; 3];
    for p in tiles.iter().flat_map(|t| t.rgba.as_chunks::<4>().0) {
        for k in 0..3 {
            mean[k] += p[k] as f32 / n;
        }
    }
    let mut out = Picture::filled(
        ATLAS_PX,
        ATLAS_PX,
        [
            mean[0].round() as u8,
            mean[1].round() as u8,
            mean[2].round() as u8,
            0,
        ],
    );
    for (i, t) in tiles.iter().enumerate() {
        let [x, y, _, _] = tile_px(i);
        out.paste(t, x, y);
    }
    out
}

/// The atlas's tiles as crates/tools/src/species.rs reads them (tiles.json): each tile's name, its leaf's class
/// (`core` for a dense tile, with the leaf it is of), what it pictures, and its rectangle in a model's UVs (u right, v
/// down from the picture's top).
pub fn tiles_json() -> String {
    let mut out = format!("{{\n  \"size\": {ATLAS_PX},\n  \"tiles\": [\n");
    for (i, (name, class, pictured)) in ATLAS.iter().enumerate() {
        let nth = (0..i)
            .filter(|&j| ATLAS[j].0 == *name && ATLAS[j].2 == *pictured)
            .count()
            + 1;
        let [x, y, w, h] = tile_px(i).map(|v| v as f64 / ATLAS_PX as f64);
        let (kind, named, class, leaf) = match pictured {
            Pictured::Cluster => ("cluster", format!("{name} {nth}"), *class, String::new()),
            Pictured::Blossom => (
                "blossom",
                format!("{name} blossom {nth}"),
                *class,
                String::new(),
            ),
            Pictured::Strand => (
                "strand",
                format!("{name} strand {nth}"),
                *class,
                String::new(),
            ),
            Pictured::Core => (
                "core",
                format!("{name} core"),
                "core",
                format!(", \"leaf\": \"{class}\""),
            ),
        };
        out += &format!(
            "    {{\"name\": \"{named}\", \"class\": \"{class}\", \"kind\": \"{kind}\"{leaf}, \"rect\": [{}, {}, {}, {}]}}{}\n",
            x,
            y,
            x + w,
            y + h,
            if i + 1 < ATLAS.len() { "," } else { "" }
        );
    }
    out + "  ]\n}\n"
}

/// Writes `picture` as a PNG.
fn save(picture: &Picture, out: &Path) -> Result<(), String> {
    use image::ImageEncoder;
    let f = std::fs::File::create(out).map_err(|e| format!("{}: {e}", out.display()))?;
    image::codecs::png::PngEncoder::new_with_quality(
        std::io::BufWriter::new(f),
        image::codecs::png::CompressionType::Best,
        image::codecs::png::FilterType::Adaptive,
    )
    .write_image(
        &picture.rgba,
        picture.width as u32,
        picture.height as u32,
        image::ExtendedColorType::Rgba8,
    )
    .map_err(|e| format!("{}: {e}", out.display()))
}

/// `cargo tex leafcards [dir]`: the atlas (albedo.png) and its tiles (tiles.json), into the assets' textures/leafcards
/// unless a folder is given.
pub fn run(out: Option<&str>) -> Result<(), String> {
    let dir = match out {
        Some(dir) => Path::new(dir).to_path_buf(),
        None => crate::content::root().join("web/public/assets/textures/leafcards"),
    };
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let picture = atlas();
    save(&picture, &dir.join("albedo.png"))?;
    let tiles = dir.join("tiles.json");
    std::fs::write(&tiles, tiles_json()).map_err(|e| format!("{}: {e}", tiles.display()))?;
    for (i, (name, _, pictured)) in ATLAS.iter().enumerate() {
        let [x, y, w, h] = tile_px(i);
        let covered = (y..y + h)
            .flat_map(|py| (x..x + w).map(move |px| (px, py)))
            .filter(|&(px, py)| picture.at(px, py)[3] > 127)
            .count();
        println!(
            "{name} {pictured:?}: {:.0} % covered",
            100.0 * covered as f64 / (w * h) as f64
        );
    }
    println!("wrote {} and tiles.json", dir.join("albedo.png").display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn committed(file: &str) -> std::path::PathBuf {
        crate::content::root()
            .join("web/public/assets/textures/leafcards")
            .join(file)
    }

    /// The committed tiles.json is the table's (artkit's `botany::ATLAS`), and the species check reads it.
    #[test]
    fn the_committed_tiles_are_the_tables() {
        let text = std::fs::read_to_string(committed("tiles.json")).unwrap();
        assert!(text == tiles_json(), "run `cargo tex leafcards`");
        let atlas = crate::species::Atlas::parse(&text).unwrap();
        assert_eq!(atlas.tiles.len(), ATLAS.len());
        for (i, t) in atlas.tiles.iter().enumerate() {
            let [x, y, w, h] = tile_px(i).map(|v| v as f64 / ATLAS_PX as f64);
            assert_eq!(t.rect, [x, y, x + w, y + h]);
            assert_eq!(t.class == "core", ATLAS[i].2 == Pictured::Core);
        }
    }

    /// The committed atlas's tiles are what the generator draws: a cluster and a hanging strand drawn again, texel
    /// for texel (the whole atlas takes half a minute: `cargo tex leafcards`, then `git status`).
    #[test]
    fn the_committed_atlas_is_drawn_from_this_source() {
        let atlas = image::open(committed("albedo.png")).unwrap().to_rgba8();
        assert_eq!(atlas.dimensions(), (ATLAS_PX as u32, ATLAS_PX as u32));
        let strand = ATLAS.iter().position(|t| t.2 == Pictured::Strand).unwrap();
        for i in [0, strand] {
            let drawn = tile(i);
            let [x, y, w, h] = tile_px(i);
            for py in 0..h {
                for px in 0..w {
                    let here = atlas.get_pixel((x + px) as u32, (y + py) as u32).0;
                    assert!(
                        here == drawn.at(px, py),
                        "tile {i} at {px}, {py}: run `cargo tex leafcards`"
                    );
                }
            }
        }
    }

    /// Every cluster's covered texels have one mean luminance, and every dense tile's texels another: a card's
    /// colour tints each species' leaves as light as the next's.
    #[test]
    fn the_tiles_are_as_light_as_one_another() {
        let atlas = image::open(committed("albedo.png")).unwrap().to_rgba8();
        for (i, (name, _, pictured)) in ATLAS.iter().enumerate() {
            let [x, y, w, h] = tile_px(i);
            let (mut sum, mut n) = (0.0, 0.0);
            for py in y..y + h {
                for px in x..x + w {
                    let p = atlas.get_pixel(px as u32, py as u32).0;
                    if p[3] > 127 {
                        sum += luminance(&p);
                        n += 1.0;
                    }
                }
            }
            let aim = if *pictured == Pictured::Core {
                DENSE_LIGHT
            } else {
                CLUSTER_LIGHT
            };
            assert!(
                (sum / n - aim).abs() < 0.01,
                "{name} {pictured:?}: {} for {aim}",
                sum / n
            );
        }
    }
}

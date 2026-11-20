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
pub(crate) struct Sketch {
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

mod leaves;
mod tiles;

pub(crate) use leaves::*;
pub(crate) use tiles::*;

#[cfg(test)]
mod tests;

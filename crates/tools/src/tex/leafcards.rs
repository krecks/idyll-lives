//! The trees' leaf-card atlas (`cargo tex leafcards`): the texture every broadleaved tree's leaf cards are cut
//! from (the `leafcards` finish; artkit's `botany::cards`). Leaves are built with the kernel's botany
//! (`cut_leaf`, `outline`, `twig`), lit a point at a time by a sun and a sky with the leaves' own shadows on one
//! another (rays against the kernel's `Bvh`), and drawn from above with the kernel's rasteriser, whose samples
//! give the cut-out's alpha. Pale and nearly grey: a card's colour tints it. The leaves' colour is bled out into
//! the clear texels round them, so no mipmap mixes in a dark fringe. Nothing downloaded; the same source gives the
//! same bytes.

use artkit::botany::{CutLeaf, OutlineOf, Twig, cut_leaf, outline, twig};
use artkit::bvh::Bvh;
use artkit::materials::Mat;
use artkit::math::{PI, TAU, V3, cos, degrees, sin, sqrt, v3};
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
const SUN_RAYS: usize = 8;
const SKY_RAYS: usize = 12;
/// A face is lit at points no farther apart than this share of its tile.
const FINE: f64 = 0.025;

/// A point of a face being lit: where, its normal, its light and how much of the sun's disc it sees.
type Lit = (V3, V3, (f32, f32));

/// The parts of a tile in the making, each with its colour.
#[derive(Default)]
struct Sketch {
    parts: Vec<(Obj, Tone)>,
}

impl Sketch {
    fn new() -> Sketch {
        scene::reset();
        Sketch::default()
    }

    fn add(&mut self, obj: Obj, tone: Tone) {
        self.parts.push((obj, tone));
    }

    /// The parts as triangles in the world: (its three corners, its part).
    fn triangles(&self) -> Vec<([V3; 3], usize)> {
        let mut out = Vec::new();
        for (i, (obj, _)) in self.parts.iter().enumerate() {
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
fn lit(tris: &[([V3; 3], usize)], tones: &[Tone], fine: f64) -> Vec<raster::Mesh> {
    let bvh = Bvh::new(tris.iter().map(|(t, _)| *t).collect());
    let sun = V3::from(SUN).normalized();
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
        (
            SKY * (1.0 - SKY_HIDDEN * (1.0 - open)) + SUN_STRENGTH * facing * direct,
            direct,
        )
    };
    let mut meshes: Vec<raster::Mesh> = tones
        .iter()
        .map(|&colour| raster::Mesh {
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
        // cut in two across its longest side that is over `fine`, or over a quarter of that with a shadow's edge
        // across it (its ends see more and less of the sun), until no side is: a side is cut or kept by its
        // ends alone, so the faces either side of it are cut alike
        let mut todo: Vec<[Lit; 3]> = vec![corners.map(|(p, n)| (p, n, shade(&mut lights, p, n)))];
        let mesh = &mut meshes[*part];
        while let Some(c) = todo.pop() {
            // (a piece of no size draws nothing, and cut along its own line it would never end)
            if (c[1].0 - c[0].0).cross(c[2].0 - c[0].0).length() < 1e-9 {
                continue;
            }
            let mut cut: Option<(usize, f64)> = None;
            for i in 0..3 {
                let (a, b) = (c[i], c[(i + 1) % 3]);
                let long = (a.0 - b.0).length();
                if (long > fine || (long > fine / 4.0 && (a.2.1 - b.2.1).abs() > 0.2))
                    && cut.is_none_or(|(_, most)| long > most)
                {
                    cut = Some((i, long));
                }
            }
            if let Some((i, _)) = cut {
                let (a, b, far) = (c[i], c[(i + 1) % 3], c[(i + 2) % 3]);
                // (the middle of a side is the same from either end)
                let (lo, hi) = if key(0, a.0) < key(0, b.0) {
                    (a, b)
                } else {
                    (b, a)
                };
                let (p, n) = ((lo.0 + hi.0) / 2, (lo.1 + hi.1).normalized());
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
    meshes
}

/// The sketch seen from above: `across` metres round `centre`, `size` pixels a side, clear where nothing is;
/// with `show`, only the parts it says.
fn draw(
    sketch: &Sketch,
    centre: (f64, f64),
    across: f64,
    size: usize,
    fine: f64,
    show: impl Fn(usize) -> bool,
) -> Picture {
    let tris: Vec<([V3; 3], usize)> = sketch
        .triangles()
        .into_iter()
        .filter(|(_, part)| show(*part))
        .collect();
    let tones: Vec<Tone> = sketch.parts.iter().map(|(_, tone)| *tone).collect();
    let meshes = lit(&tris, &tones, fine);
    let (cx, cy) = (centre.0 as f32, centre.1 as f32);
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

// ---- the atlas as tools/art/blender/leafcards.py drew it (Blender's EEVEE then; the leaves are the same) ---------

/// A tile, metres as modelled, and the dense wall's share of its tile.
const TILE: f64 = 0.8;
const DENSE: f64 = 0.8;

/// A matte leaf colour: nearly grey (`value`), a little yellow (`warm` > 0) or blue-green (< 0).
fn tone(value: f64, warm: f64) -> Tone {
    [
        value * (1.0 + 0.10 * warm),
        value,
        value * (1.0 - 0.16 * warm - 0.04),
    ]
    .map(|c| c.min(1.0) as f32)
}

/// The leaves' colours: three depths, the top the lightest, five tones each.
fn tones() -> Vec<Vec<Tone>> {
    let tones = [
        (0.8, 0.0),
        (0.7, 0.35),
        (0.74, -0.3),
        (0.86, 0.5),
        (0.66, -0.6),
    ];
    [1.0, 0.76, 0.56]
        .iter()
        .map(|k| tones.iter().map(|(v, w)| tone(v * k, *w)).collect())
        .collect()
}

#[allow(clippy::too_many_arguments)]
/// A broad leaf with a wavy edge, lying nearly flat, heading `heading` (radians) from `at`; `depth` 0..1: how far
/// down in its cluster it lies (darker).
fn leaf(
    sketch: &mut Sketch,
    name: &str,
    rng: &mut Random,
    at: (f64, f64, f64),
    heading: f64,
    length: f64,
    mats: &[Vec<Tone>],
    depth: f64,
) {
    let width = length * rng.uniform(0.5, 0.62);
    let wave = rng.uniform(0.08, 0.16);
    let waves = rng.choice(&[5.0, 7.0]);
    let edge = outline(
        "oval",
        width,
        OutlineOf {
            n: 14,
            wave,
            waves,
            teeth: 0.0,
        },
    );
    let shade = (mats.len() - 1).min((depth * mats.len() as f64) as usize);
    let tilt = rng.uniform(-6.0, 22.0);
    let tone = mats[shade][rng.randrange(mats[shade].len() as i64) as usize];
    let droop = rng.uniform(0.05, 0.3);
    let fold = rng.uniform(0.25, 0.5);
    let twist = rng.uniform(-25.0, 25.0);
    let o = CutLeaf {
        droop,
        fold,
        thick: 0.004,
        ribs: 5,
        twist,
        left: None,
    };
    sketch.add(
        cut_leaf(
            name,
            at,
            90.0 - degrees(heading),
            tilt,
            length,
            &edge,
            BLANK,
            o,
        ),
        tone,
    );
}

/// The longest a leaf from `p` heading `heading` may be (up to `length`) and stay within `reach` of `centre`
/// across and up the tile.
fn fits(p: (f64, f64), heading: f64, mut length: f64, centre: (f64, f64), reach: f64) -> f64 {
    let (dx, dy) = (cos(heading), sin(heading));
    for _ in 0..12 {
        if (p.0 + dx * length - centre.0).abs() < reach
            && (p.1 + dy * length - centre.1).abs() < reach
        {
            return length;
        }
        length *= 0.88;
    }
    0.0
}

/// A cluster in a tile round `centre`: `fill` leaves lying under it, then twigs fanning from the middle with
/// `per` leaves along each, heading out and to the sides, the later ones laid over the earlier; everything within
/// `reach` of the middle across and up the tile, so no leaf crosses into the next.
#[allow(clippy::too_many_arguments)]
fn cluster(
    sketch: &mut Sketch,
    name: &str,
    rng: &mut Random,
    centre: (f64, f64),
    mats: &[Vec<Tone>],
    wood: Tone,
    twigs: usize,
    per: usize,
    fill: usize,
    turn: f64,
) {
    let reach = 0.37;
    let (cx, cy) = centre;
    let (mut n, total) = (0, fill + twigs * per);
    let mut add =
        |sketch: &mut Sketch, rng: &mut Random, p: (f64, f64), heading: f64, size: f64| {
            let size = fits(p, heading, size, centre, reach);
            if size >= 0.07 {
                let at = (p.0, p.1, 0.012 + 0.003 * n as f64);
                leaf(
                    sketch,
                    &format!("{name}_leaf{n}"),
                    rng,
                    at,
                    heading,
                    size,
                    mats,
                    1.0 - n as f64 / total as f64,
                );
            }
            n += 1;
        };
    for _ in 0..fill {
        let a = rng.uniform(0.0, TAU);
        let r = reach * rng.uniform(0.25, 0.8);
        let heading = a + rng.uniform(-1.2, 1.2);
        let size = rng.uniform(0.15, 0.2);
        add(
            sketch,
            rng,
            (cx + r * cos(a), cy + r * sin(a)),
            heading,
            size,
        );
    }
    for t in 0..twigs {
        let a = turn + TAU * (t as f64 + rng.uniform(-0.25, 0.25)) / twigs as f64;
        let length = reach * rng.uniform(0.6, 0.85);
        let bow = rng.uniform(-0.3, 0.3);
        let pts = [
            v3(cx + 0.02 * cos(a), cy + 0.02 * sin(a), 0.0),
            v3(
                cx + length * 0.5 * cos(a + bow),
                cy + length * 0.5 * sin(a + bow),
                0.01,
            ),
            v3(cx + length * cos(a), cy + length * sin(a), 0.015),
        ];
        sketch.add(
            twig(
                format!("{name}_twig{t}"),
                &pts,
                0.006,
                0.003,
                BLANK,
                Twig {
                    verts: 4,
                    ..Twig::default()
                },
            ),
            wood,
        );
        for i in 0..per {
            let f = (i as f64 + 0.6) / per as f64;
            let p = if f < 0.5 {
                pts[0].lerp(pts[1], f * 2.0)
            } else {
                pts[1].lerp(pts[2], f * 2.0 - 1.0)
            };
            let side =
                if i % 2 != 0 { -1.0 } else { 1.0 } * rng.uniform(0.5, 1.25) * (1.0 - 0.75 * f);
            let heading = a + side + rng.uniform(-0.2, 0.2);
            let size = rng.uniform(0.15, 0.21);
            add(sketch, rng, (p.x as f64, p.y as f64), heading, size);
        }
    }
}

/// A wall of leaves over the tile from `lo` to `hi` (and a little past it), at every angle, a few deep.
fn dense(
    sketch: &mut Sketch,
    name: &str,
    rng: &mut Random,
    lo: (f64, f64),
    hi: (f64, f64),
    mats: &[Vec<Tone>],
    leaves: usize,
) {
    for i in 0..leaves {
        let x = rng.uniform(lo.0 - 0.05, hi.0 + 0.05);
        let y = rng.uniform(lo.1 - 0.05, hi.1 + 0.05);
        let heading = rng.uniform(0.0, TAU);
        let length = rng.uniform(0.13, 0.18);
        leaf(
            sketch,
            &format!("{name}{i}"),
            rng,
            (x, y, 0.002 * i as f64),
            heading,
            length,
            mats,
            1.0 - i as f64 / leaves as f64,
        );
    }
}

/// The atlas: 512 x 512, four 256 px tiles: three clusters of leaves on twigs, cut out, and (bottom right) a dense
/// wall of leaves, opaque, in the middle `DENSE` of its tile with a clear gutter round it.
pub fn atlas() -> Picture {
    let size = 512;
    let mut sketch = Sketch::new();
    let mut rng = Random::new(7);
    let mats = tones();
    let wood = tone(0.4, 1.2);
    let h = TILE / 2.0;
    cluster(
        &mut sketch,
        "a",
        &mut rng,
        (-h, h),
        &mats,
        wood,
        6,
        6,
        10,
        0.0,
    );
    cluster(
        &mut sketch,
        "b",
        &mut rng,
        (h, h),
        &mats,
        wood,
        5,
        7,
        8,
        0.6,
    );
    cluster(
        &mut sketch,
        "c",
        &mut rng,
        (-h, -h),
        &mats,
        wood,
        7,
        5,
        12,
        2.2,
    );
    let clusters = sketch.parts.len();
    dense(
        &mut sketch,
        "d",
        &mut rng,
        (0.0, -TILE),
        (TILE, 0.0),
        &mats,
        170,
    );
    let mut out = draw(&sketch, (0.0, 0.0), 2.0 * TILE, size, FINE * TILE, |part| {
        part < clusters
    });
    let mean = bleed(&mut out);
    // the dense tile: opaque, its gaps the dark of a crown's inside; the gutter the wall's colour, clear
    let half = size / 2;
    let d = (half as f64 * DENSE) as usize;
    let o = (half - d) / 2;
    let wall = draw(&sketch, (h, -h), TILE * DENSE, d, FINE * TILE, |part| {
        part >= clusters
    });
    // (mixed as the bytes are, as the script mixed them)
    let mut solid = Vec::with_capacity(d * d);
    for p in wall.rgba.as_chunks::<4>().0 {
        let a = p[3] as f32 / 255.0;
        solid.push([0, 1, 2].map(|k| p[k] as f32 * a + mean[k] * 0.4 * (1.0 - a)));
    }
    let n = solid.len() as f32;
    let all = solid.iter().fold([0.0f32; 3], |s, c| {
        [s[0] + c[0] / n, s[1] + c[1] / n, s[2] + c[2] / n]
    });
    for y in 0..half {
        for x in 0..half {
            let inside = (o..o + d).contains(&x) && (o..o + d).contains(&y);
            let c = if inside {
                solid[(y - o) * d + (x - o)]
            } else {
                all
            };
            let at = ((half + y) * size + half + x) * 4;
            for k in 0..3 {
                out.rgba[at + k] = c[k].round().clamp(0.0, 255.0) as u8;
            }
            out.rgba[at + 3] = if inside { 255 } else { 0 };
        }
    }
    out
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

/// `cargo tex leafcards [out.png]`: the atlas, into the assets unless a path is given.
pub fn run(out: Option<&str>) -> Result<(), String> {
    let path = match out {
        Some(path) => Path::new(path).to_path_buf(),
        None => crate::content::root().join("web/public/assets/textures/leafcards/albedo.png"),
    };
    let picture = atlas();
    save(&picture, &path)?;
    println!("wrote {}", path.display());
    Ok(())
}

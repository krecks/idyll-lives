//! A far version against what it stands in for.

use super::*;
use crate::vec3::dot;

// ---- a far version against its near one ---------------------------------------------------------------

/// A triangle as a camera far off sees it: its corners, its normal and its colour as drawn (linear, 0..1: its
/// vertices' mean times its material's colour and texture).
pub(super) struct Seen {
    p: [V3; 3],
    n: V3,
    colour: [f64; 3],
}

/// What shows of a sample from far off along `w` (towards the camera): its triangles filed by the cells of a grid
/// across the view, to ask which of them is nearest the camera at a point.
pub(super) struct View<'a> {
    seen: &'a [Seen],
    /// Each triangle's corners in the view's own axes (across, up, towards the camera); None: not drawn from here.
    flat: Vec<Option<[V3; 3]>>,
    lo: [f64; 2],
    size: [usize; 2],
    cells: Vec<Vec<u32>>,
}

/// The grid's cells (m): the triangles a point may lie in are a cell's.
pub(super) const CELL: f64 = 0.5;
/// How far apart the views are sampled (m): a tenth of a metre shows a corner board, and a house is ten thousand points.
pub(super) const STEP: f64 = 0.1;

impl<'a> View<'a> {
    /// The view of `seen` along `w`, `u` across and `v` up the picture (the three at right angles).
    pub(super) fn new(seen: &'a [Seen], u: V3, v: V3, w: V3) -> View<'a> {
        let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
        for p in seen.iter().flat_map(|t| t.p) {
            for (k, axis) in [u, v].into_iter().enumerate() {
                lo[k] = lo[k].min(dot(p, axis));
                hi[k] = hi[k].max(dot(p, axis));
            }
        }
        if seen.is_empty() {
            (lo, hi) = ([0.0; 2], [0.0; 2]);
        }
        let size = [0, 1].map(|k| ((hi[k] - lo[k]) / CELL).floor() as usize + 1);
        let mut view = View {
            seen,
            // (One seen from behind isn't drawn; one seen edge on covers no point.)
            flat: seen
                .iter()
                .map(|t| {
                    (dot(t.n, w) > 1e-6).then(|| t.p.map(|p| [dot(p, u), dot(p, v), dot(p, w)]))
                })
                .collect(),
            lo,
            size,
            cells: vec![Vec::new(); size[0] * size[1]],
        };
        for (i, q) in view.flat.iter().enumerate() {
            let Some(q) = q else {
                continue;
            };
            let cell = |k: usize, most: bool| {
                let reach = q.iter().map(|p| p[k]);
                let far = if most {
                    reach.fold(f64::NEG_INFINITY, f64::max)
                } else {
                    reach.fold(f64::INFINITY, f64::min)
                };
                (((far - lo[k]) / CELL).floor().max(0.0) as usize).min(size[k] - 1)
            };
            for b in cell(1, false)..=cell(1, true) {
                for a in cell(0, false)..=cell(0, true) {
                    view.cells[b * size[0] + a].push(i as u32);
                }
            }
        }
        view
    }

    /// The triangle nearest the camera at (`a`, `b`) of the picture, and how far along `w` it is there.
    pub(super) fn first(&self, a: f64, b: f64) -> Option<(&'a Seen, f64)> {
        let (ca, cb) = (
            ((a - self.lo[0]) / CELL).floor(),
            ((b - self.lo[1]) / CELL).floor(),
        );
        if ca < 0.0 || cb < 0.0 || ca as usize >= self.size[0] || cb as usize >= self.size[1] {
            return None;
        }
        let mut best: Option<(&Seen, f64)> = None;
        for &i in &self.cells[cb as usize * self.size[0] + ca as usize] {
            let Some(q) = self.flat[i as usize] else {
                continue;
            };
            let d = (q[1][0] - q[0][0]) * (q[2][1] - q[0][1])
                - (q[2][0] - q[0][0]) * (q[1][1] - q[0][1]);
            if d.abs() < 1e-12 {
                continue;
            }
            let b1 =
                ((a - q[0][0]) * (q[2][1] - q[0][1]) - (q[2][0] - q[0][0]) * (b - q[0][1])) / d;
            let b2 =
                ((q[1][0] - q[0][0]) * (b - q[0][1]) - (a - q[0][0]) * (q[1][1] - q[0][1])) / d;
            if b1 < -1e-9 || b2 < -1e-9 || b1 + b2 > 1.0 + 1e-9 {
                continue;
            }
            let depth = q[0][2] + b1 * (q[1][2] - q[0][2]) + b2 * (q[2][2] - q[0][2]);
            if best.is_none_or(|(_, far)| depth > far) {
                best = Some((&self.seen[i as usize], depth));
            }
        }
        best
    }

    /// Every point of the picture `STEP` apart with what shows there: `each(a, b, triangle, depth)`.
    pub(super) fn points(&self, mut each: impl FnMut(f64, f64, &'a Seen, f64)) {
        let count = [0, 1].map(|k| (self.size[k] as f64 * CELL / STEP).ceil() as usize);
        for j in 0..count[1] {
            for i in 0..count[0] {
                let (a, b) = (
                    self.lo[0] + (i as f64 + 0.5) * STEP,
                    self.lo[1] + (j as f64 + 0.5) * STEP,
                );
                if let Some((t, depth)) = self.first(a, b) {
                    each(a, b, t, depth);
                }
            }
        }
    }
}

/// Colours by how much of a picture they cover: each (sRGB bytes) with its area, to find the one most of it is.
#[derive(Default)]
pub(super) struct Palette(BTreeMap<[u8; 3], (f64, [f64; 3])>);

impl Palette {
    /// `area` more of `colour` (linear).
    pub(super) fn add(&mut self, colour: [f64; 3], area: f64) {
        // (Sixteen steps a channel: a wall's shading by vertex stays one colour, another paint is another.)
        let bin = colour.map(|c| ((srgb(c.clamp(0.0, 1.0)) * 255.0).round() as u8) >> 4);
        let slot = self.0.entry(bin).or_default();
        slot.0 += area;
        for k in 0..3 {
            slot.1[k] += colour[k] * area;
        }
    }

    /// The colour most of it is (sRGB bytes: the mean of its bin and the bins beside it), and its share.
    pub(super) fn most(&self) -> Option<([u8; 3], f64)> {
        let total: f64 = self.0.values().map(|v| v.0).sum();
        let (bin, _) = self
            .0
            .iter()
            .max_by(|a, b| a.1.0.total_cmp(&b.1.0).then(b.0.cmp(a.0)))?;
        let (mut area, mut sum) = (0.0, [0.0; 3]);
        for (other, (a, s)) in &self.0 {
            if (0..3).all(|k| other[k].abs_diff(bin[k]) <= 1) {
                area += a;
                for k in 0..3 {
                    sum[k] += s[k];
                }
            }
        }
        Some((
            sum.map(|s| (srgb(s / area) * 255.0).round() as u8),
            area / total,
        ))
    }
}

/// A roof's kind, as its faces show it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Roof {
    Flat,
    Gable,
    Hip,
}

/// How a building looks from far off, measured from its meshes: what a far version has to agree with.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Look {
    /// What its roofs cover in plan: the box round them (x0, z0, x1, z1, m) and their area (m²).
    pub plan: [f64; 4],
    pub area: f64,
    /// The top of its roof (m).
    pub top: f64,
    pub roof: Roof,
    /// Its roof's pitch (degrees): the pitch half its sloped faces are under. 0: flat.
    pub pitch: f64,
    /// The pieces of chimney stacks standing over its roof (a stack's brick, its cap).
    pub chimneys: usize,
    /// The colour most of its walls are and most of its roof is, as drawn (sRGB: None where it has none).
    pub wall: Option<[u8; 3]>,
    pub roof_colour: Option<[u8; 3]>,
}

/// What a roof is at least this high over the ground (m): a porch's landing and a sill look up too.
pub(super) const ROOFS_FROM: f64 = 1.2;
/// A face looking up less steeply than this (the y of its normal) is a wall's, not a roof's.
pub(super) const UP: f64 = 0.2;
/// A roof pitched under this (rise over run) is flat.
pub(super) const LEVEL: f64 = 0.1;
/// A stack is no wider than this (m), and stands this far over the roof's top at least.
pub(super) const STACK: f64 = 1.6;
pub(super) const STACK_OVER: f64 = 0.3;
/// Gable ends are at least this share of a gable roof's plan (a porch's little gable is under it).
pub(super) const GABLES: f64 = 0.04;

/// `parts`' look (`Look`): the roofs from above, the walls from the four sides.
pub(crate) fn look(parts: &[Part], surfaces: &Surfaces, pictures: &mut Pictures) -> Look {
    let mut seen = Vec::new();
    for part in parts {
        let material = &part.mesh.material;
        if part.shadow_only || !opaque(material, surfaces) || glass(material) {
            continue;
        }
        let surface = surfaces.get(material);
        let tint = surface.map_or([1.0; 3], |s| s.color.map(linear));
        let picture = surface
            .and_then(|s| s.texture.as_ref())
            .and_then(|file| pictures.mean(file))
            .unwrap_or([1.0; 3]);
        for t in &part.mesh.triangles {
            let p = t.map(|i| part.mesh.positions[i as usize]);
            let (e1, e2) = (
                [0, 1, 2].map(|k| p[1][k] - p[0][k]),
                [0, 1, 2].map(|k| p[2][k] - p[0][k]),
            );
            let n = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            let l = dot(n, n).sqrt();
            if l < 1e-12 {
                continue;
            }
            let mut colour = [0.0; 3];
            for &i in t {
                let c = part.colors.get(i as usize).copied().unwrap_or([255; 4]);
                for k in 0..3 {
                    colour[k] += linear(c[k] as f64 / 255.0) / 3.0;
                }
            }
            seen.push(Seen {
                p,
                n: n.map(|c| c / l),
                colour: [0, 1, 2].map(|k| colour[k] * tint[k] * picture[k]),
            });
        }
    }
    // The roofs: what looks up at a camera overhead, over the ground storey's middle.
    let above = View::new(&seen, [1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]);
    let mut roof = Palette::default();
    let mut cells: Vec<(f64, f64, f64, &Seen)> = Vec::new();
    above.points(|x, z, t, y| {
        if t.n[1] > UP && y > ROOFS_FROM {
            roof.add(t.colour, STEP * STEP);
            cells.push((x, z, y, t));
        }
    });
    let roof_colour = roof.most().map(|(c, _)| c);
    let like = |a: [u8; 3], colour: [f64; 3]| {
        (0..3).all(|k| {
            a[k].abs_diff((srgb(colour[k].clamp(0.0, 1.0)) * 255.0).round() as u8) <= COLOUR
        })
    };
    let mut plan = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    let (mut top, mut level, mut pitches) = (0.0_f64, 0.0, Vec::new());
    for &(x, z, y, t) in &cells {
        plan = [
            plan[0].min(x),
            plan[1].min(z),
            plan[2].max(x),
            plan[3].max(z),
        ];
        // (The roof's own top: a stack's cap is another colour.)
        if roof_colour.is_some_and(|c| like(c, t.colour)) {
            top = top.max(y);
        }
        let run = (t.n[0] * t.n[0] + t.n[2] * t.n[2]).sqrt();
        if run < LEVEL * t.n[1] {
            level += 1.0;
        } else {
            pitches.push((run.atan2(t.n[1]).to_degrees(), 1.0));
        }
    }
    let area = cells.len() as f64 * STEP * STEP;
    if cells.is_empty() {
        plan = [0.0; 4];
    }
    // Gable ends: upright faces with a side that slants and is their own (no face goes on past it in their plane).
    let mut sides: HashMap<([i64; 3], [i64; 3]), Vec<usize>> = HashMap::new();
    let mm = |p: V3| p.map(|c| (c * 1e3).round() as i64);
    for (i, t) in seen.iter().enumerate().filter(|(_, t)| t.n[1].abs() < 0.05) {
        for k in 0..3 {
            let (a, b) = (mm(t.p[k]), mm(t.p[(k + 1) % 3]));
            sides.entry((a.min(b), a.max(b))).or_default().push(i);
        }
    }
    let mut gables = 0.0;
    for (i, t) in seen.iter().enumerate().filter(|(_, t)| t.n[1].abs() < 0.05) {
        let own_slant = (0..3).any(|k| {
            let (a, b) = (t.p[k], t.p[(k + 1) % 3]);
            let d = [0, 1, 2].map(|c| b[c] - a[c]);
            let l = dot(d, d).sqrt().max(1e-12);
            let rise = d[1].abs() / l;
            (0.15..0.98).contains(&rise)
                && !sides[&(mm(a).min(mm(b)), mm(a).max(mm(b)))]
                    .iter()
                    .any(|&j| j != i && dot(seen[j].n, t.n) > 0.999)
        });
        if own_slant {
            let (e1, e2) = (
                [0, 1, 2].map(|k| t.p[1][k] - t.p[0][k]),
                [0, 1, 2].map(|k| t.p[2][k] - t.p[0][k]),
            );
            let c = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            gables += dot(c, c).sqrt() / 2.0;
        }
    }
    let kind = if cells.is_empty() || level >= 0.9 * cells.len() as f64 {
        Roof::Flat
    } else if gables >= GABLES * area {
        Roof::Gable
    } else {
        Roof::Hip
    };
    // Stacks: small pieces (what a part's triangles join up) standing over the roof's top.
    let mut chimneys = 0;
    for part in parts
        .iter()
        .filter(|p| !p.shadow_only && !glass(&p.mesh.material))
    {
        let at = &part.mesh.positions;
        if cells.is_empty() || !at.iter().any(|p| p[1] > top + STACK_OVER) {
            continue;
        }
        let mut places: HashMap<[i64; 3], u32> = HashMap::new();
        let place: Vec<u32> = at
            .iter()
            .map(|p| {
                let n = places.len() as u32;
                *places.entry(mm(*p)).or_insert(n)
            })
            .collect();
        let mut parent: Vec<u32> = (0..places.len() as u32).collect();
        fn root(parent: &mut [u32], mut i: u32) -> u32 {
            while parent[i as usize] != i {
                parent[i as usize] = parent[parent[i as usize] as usize];
                i = parent[i as usize];
            }
            i
        }
        for t in &part.mesh.triangles {
            let a = root(&mut parent, place[t[0] as usize]);
            for &v in &t[1..] {
                let b = root(&mut parent, place[v as usize]);
                parent[b as usize] = a;
            }
        }
        let mut boxes: BTreeMap<u32, [f64; 5]> = BTreeMap::new();
        for (i, p) in at.iter().enumerate() {
            let b = boxes.entry(root(&mut parent, place[i])).or_insert([
                f64::INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
                f64::NEG_INFINITY,
            ]);
            *b = [
                b[0].min(p[0]),
                b[1].min(p[2]),
                b[2].max(p[0]),
                b[3].max(p[2]),
                b[4].max(p[1]),
            ];
        }
        chimneys += boxes
            .values()
            .filter(|b| b[4] > top + STACK_OVER && b[2] - b[0] <= STACK && b[3] - b[1] <= STACK)
            .filter(|b| inside(plan, [(b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0]))
            .count();
    }
    // The walls: what stands upright before a camera level with it on each of its four sides, under the roof's top.
    let mut wall = Palette::default();
    for (u, w) in [
        ([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
        ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0]),
    ] {
        for sign in [1.0, -1.0] {
            let view = View::new(&seen, u, [0.0, 1.0, 0.0], w.map(|c: f64| c * sign));
            view.points(|_, y, t, _| {
                if t.n[1].abs() <= UP && y > 0.4 && y < top.max(ROOFS_FROM) {
                    wall.add(t.colour, STEP * STEP);
                }
            });
        }
    }
    Look {
        plan,
        area,
        top,
        roof: kind,
        pitch: if kind == Roof::Flat {
            0.0
        } else {
            texels::median(&mut pitches, 0.5).unwrap_or(0.0)
        },
        chimneys,
        wall: wall.most().map(|(c, _)| c),
        roof_colour,
    }
}

pub(super) fn inside(plan: [f64; 4], at: [f64; 2]) -> bool {
    at[0] >= plan[0] && at[0] <= plan[2] && at[1] >= plan[1] && at[1] <= plan[3]
}

/// How far a far version may be off its near one: a side of its plan (m; a porch stands out less), its plan's
/// area (a share: the eaves' overhang differs by less), its top (m), its pitch (degrees), a colour's channel (of
/// 255: the same paint under a texture and in the plain finish).
pub(super) const PLAN: f64 = 1.5;
pub(super) const AREA: f64 = 0.1;
pub(super) const TOP: f64 = 0.5;
pub(super) const PITCH: f64 = 4.0;
pub(super) const COLOUR: u8 = 28;

pub(super) fn hex_of(c: Option<[u8; 3]>) -> String {
    c.map_or("none".into(), |c| {
        format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2])
    })
}

pub(super) fn roof_name(r: Roof) -> &'static str {
    match r {
        Roof::Flat => "flat",
        Roof::Gable => "gable",
        Roof::Hip => "hip",
    }
}

/// Where a far version's look disagrees with its near one's: a line each.
pub(crate) fn disagreements(near: &Look, far: &Look) -> Vec<String> {
    let mut out = Vec::new();
    let side = (0..4).fold(0.0_f64, |most, k| {
        most.max((near.plan[k] - far.plan[k]).abs())
    });
    let share = (far.area - near.area).abs() / near.area.max(1e-9);
    if side > PLAN || share > AREA {
        out.push(format!(
            "plan: its roofs cover {:.0} m² ({:.1} m by {:.1} m), near {:.0} m² ({:.1} m by {:.1} m): {:.0} % and {:.1} m off (at most {:.0} % and {PLAN} m)",
            far.area,
            far.plan[2] - far.plan[0],
            far.plan[3] - far.plan[1],
            near.area,
            near.plan[2] - near.plan[0],
            near.plan[3] - near.plan[1],
            share * 100.0,
            side,
            AREA * 100.0
        ));
    }
    if (near.top - far.top).abs() > TOP {
        out.push(format!(
            "height: its roof's top is at {:.2} m, near at {:.2} m (at most {TOP} m apart)",
            far.top, near.top
        ));
    }
    if near.roof != far.roof {
        out.push(format!(
            "roof: a {} roof, near a {} roof",
            roof_name(far.roof),
            roof_name(near.roof)
        ));
    } else if (near.pitch - far.pitch).abs() > PITCH {
        out.push(format!(
            "roof: pitched {:.0}°, near {:.0}° (at most {PITCH}° apart)",
            far.pitch, near.pitch
        ));
    }
    if (near.chimneys > 0) != (far.chimneys > 0) {
        out.push(format!(
            "chimney: {}, near {}",
            if far.chimneys > 0 {
                "it has one"
            } else {
                "none"
            },
            if near.chimneys > 0 { "has one" } else { "none" }
        ));
    }
    let apart = |a: Option<[u8; 3]>, b: Option<[u8; 3]>| match (a, b) {
        (Some(a), Some(b)) => (0..3).any(|k| a[k].abs_diff(b[k]) > COLOUR),
        (a, b) => a.is_some() != b.is_some(),
    };
    if apart(near.wall, far.wall) {
        out.push(format!(
            "walls: most of them are {} as drawn, near {} (a channel at most {COLOUR} of 255 apart)",
            hex_of(far.wall),
            hex_of(near.wall)
        ));
    }
    if apart(near.roof_colour, far.roof_colour) {
        out.push(format!(
            "roof colour: most of it is {} as drawn, near {} (a channel at most {COLOUR} of 255 apart)",
            hex_of(far.roof_colour),
            hex_of(near.roof_colour)
        ));
    }
    out
}

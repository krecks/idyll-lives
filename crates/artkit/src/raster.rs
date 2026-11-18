//! A small software rasteriser: what a model's pictures are drawn with (`preview`), where Blender rendered them.
//! Triangles in the model's space through an orthographic or a perspective camera into a z-buffer of 4×4 samples
//! a pixel (`Look::samples`), drawn from the front only as the game draws them (counter-clockwise is the front, as
//! glTF has it) unless a surface is two-sided. Each surface is its flat colour times its baked shading (COLOR_0)
//! times a sun and a sky on the corner normals, so smooth and sharp edges and bevels read; a cut-out texture (a
//! leaf card on its atlas) drops the samples under its cut; a floor takes a soft dark where the model meets it.
//! Single precision, one thread, no fused multiply-add, libm for powers and exponentials: the same triangles
//! give the same pixels on every machine. No files: it returns pixels (the tools write the PNGs).

pub type Vec3 = [f32; 3];

fn add(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: Vec3, k: f32) -> Vec3 {
    [a[0] * k, a[1] * k, a[2] * k]
}

fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// `a` at unit length (zero stays zero).
pub fn unit(a: Vec3) -> Vec3 {
    let len = dot(a, a).sqrt();
    if len == 0.0 { a } else { scale(a, 1.0 / len) }
}

fn mul(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2]]
}

/// An sRGB colour (0–255 a channel, as the manifests write them) in linear light.
pub fn linear(srgb: [u8; 3]) -> Vec3 {
    srgb.map(|c| decode(c as f32 / 255.0))
}

fn decode(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        libm::powf((c + 0.055) / 1.055, 2.4)
    }
}

fn encode(c: f32) -> f32 {
    if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * libm::powf(c, 1.0 / 2.4) - 0.055
    }
}

/// An image to draw on a surface: RGBA, 8 bits a channel, sRGB colour and straight alpha, rows top down (a
/// texture coordinate's v runs down from the top, as glTF's does).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

impl Texture {
    /// Linear RGB and alpha at `uv`, blended between the four nearest texels, repeating past the edges.
    fn sample(&self, uv: [f32; 2], srgb: &[f32; 256]) -> [f32; 4] {
        if self.width == 0 || self.height == 0 || self.rgba.len() < self.width * self.height * 4 {
            return [1.0; 4];
        }
        let (w, h) = (self.width as f32, self.height as f32);
        let (x, y) = (uv[0] * w - 0.5, uv[1] * h - 0.5);
        let (fx, fy) = (x.floor(), y.floor());
        let (tx, ty) = (x - fx, y - fy);
        let wrap = |i: f32, n: usize| (i as i64).rem_euclid(n as i64) as usize;
        let (x0, y0) = (wrap(fx, self.width), wrap(fy, self.height));
        let (x1, y1) = ((x0 + 1) % self.width, (y0 + 1) % self.height);
        let texel = |x: usize, y: usize| {
            let at = (y * self.width + x) * 4;
            let p = &self.rgba[at..at + 4];
            [
                srgb[p[0] as usize],
                srgb[p[1] as usize],
                srgb[p[2] as usize],
                p[3] as f32 / 255.0,
            ]
        };
        let (a, b, c, d) = (texel(x0, y0), texel(x1, y0), texel(x0, y1), texel(x1, y1));
        std::array::from_fn(|k| {
            let top = a[k] + (b[k] - a[k]) * tx;
            let bottom = c[k] + (d[k] - c[k]) * tx;
            top + (bottom - top) * ty
        })
    }
}

/// What a mesh is drawn with.
#[derive(Clone, Debug, PartialEq)]
pub struct Surface {
    /// Linear RGB.
    pub colour: Vec3,
    /// A texture (an index into `render`'s `textures`) on the mesh's UVs, times the colour.
    pub texture: Option<usize>,
    /// Drawn only where the texture's alpha is at least this (a leaf card on its atlas).
    pub cutout: Option<f32>,
    /// Seen from behind too (its normals turned to the viewer), where the game draws it so.
    pub two_sided: bool,
}

impl Default for Surface {
    fn default() -> Surface {
        Surface {
            colour: [0.8; 3],
            texture: None,
            cutout: None,
            two_sided: false,
        }
    }
}

/// Triangles to draw, placed: three indices a triangle; a normal, a UV and a baked shade a vertex where it has
/// them (no normals: each triangle flat; no shading: 1).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<[f32; 2]>,
    pub shading: Vec<f32>,
    pub indices: Vec<u32>,
    pub surface: Surface,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Projection {
    /// Parallel lines of sight; the picture's longer side spans `span` metres.
    Orthographic { span: f32 },
    /// Lines of sight from the eye; the picture's longer side spans `fov` radians.
    Perspective { fov: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    /// Which way is up in the picture (any vector not along the line of sight).
    pub up: Vec3,
    pub projection: Projection,
    /// What is nearer the eye than `near` or further than `far` (along the line of sight) isn't drawn.
    pub near: f32,
    pub far: f32,
}

/// How a surface is lit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shading {
    /// Colour × baked shading × (sky + sun): `sun` points towards the light; the sky gives `sky` to a face
    /// looking straight up and `ground` to one looking straight down.
    Lit {
        sun: Vec3,
        sun_strength: f32,
        sky: f32,
        ground: f32,
    },
    /// The baked shading alone, unlit (white where open, darker in creases).
    Baked,
    /// The colour (and texture) alone, unlit.
    Flat,
}

/// A floor at `height` (+Y up), a disc of `radius` around `centre` (x, z) fading into the background at its
/// edge, darker by up to `contact` where the model comes within `reach` metres of it (blurred over about as far).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Floor {
    pub height: f32,
    pub centre: [f32; 2],
    pub radius: f32,
    /// Linear RGB.
    pub colour: Vec3,
    pub contact: f32,
    pub reach: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    /// Linear RGB and alpha where nothing is drawn.
    pub background: [f32; 4],
    pub shading: Shading,
    pub floor: Option<Floor>,
    /// Samples a pixel along each side (4: 16 a pixel; 1 to 8).
    pub samples: usize,
}

/// RGBA pixels, 8 bits a channel, sRGB, rows top down.
#[derive(Clone, Debug, PartialEq)]
pub struct Picture {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

impl Picture {
    /// A picture all `rgba`.
    pub fn filled(width: usize, height: usize, rgba: [u8; 4]) -> Picture {
        Picture {
            width,
            height,
            rgba: rgba.repeat(width * height),
        }
    }

    /// Copies `other` in with its top left corner at (x, y), cut at the edges.
    pub fn paste(&mut self, other: &Picture, x: usize, y: usize) {
        for row in 0..other.height.min(self.height.saturating_sub(y)) {
            let n = other.width.min(self.width.saturating_sub(x)) * 4;
            let to = ((y + row) * self.width + x) * 4;
            let from = row * other.width * 4;
            self.rgba[to..to + n].copy_from_slice(&other.rgba[from..from + n]);
        }
    }

    /// The pixel at (x, y).
    pub fn at(&self, x: usize, y: usize) -> [u8; 4] {
        let i = (y * self.width + x) * 4;
        [
            self.rgba[i],
            self.rgba[i + 1],
            self.rgba[i + 2],
            self.rgba[i + 3],
        ]
    }
}

/// The camera's frame and the sample grid.
struct Eye {
    eye: Vec3,
    right: Vec3,
    up: Vec3,
    forward: Vec3,
    perspective: bool,
    /// Samples a metre (orthographic) or the focal length in samples (perspective).
    scale: f32,
    cx: f32,
    cy: f32,
    near: f32,
    far: f32,
}

impl Eye {
    fn new(camera: &Camera, width: usize, height: usize) -> Eye {
        let forward = unit(sub(camera.target, camera.eye));
        let right = unit(cross(forward, camera.up));
        let up = cross(right, forward);
        let longer = width.max(height) as f32;
        let (perspective, scale) = match camera.projection {
            Projection::Orthographic { span } => (false, longer / span),
            Projection::Perspective { fov } => (true, longer / 2.0 / libm::tanf(fov / 2.0)),
        };
        Eye {
            eye: camera.eye,
            right,
            up,
            forward,
            perspective,
            scale,
            cx: width as f32 / 2.0,
            cy: height as f32 / 2.0,
            near: camera.near,
            far: camera.far,
        }
    }

    /// A point in the camera's frame: (right, up, depth along the line of sight).
    fn view(&self, p: Vec3) -> Vec3 {
        let d = sub(p, self.eye);
        [dot(d, self.right), dot(d, self.up), dot(d, self.forward)]
    }

    /// A point of the camera's frame on the sample grid (x right, y down).
    fn screen(&self, v: Vec3) -> [f32; 2] {
        if self.perspective {
            [
                self.cx + v[0] / v[2] * self.scale,
                self.cy - v[1] / v[2] * self.scale,
            ]
        } else {
            [self.cx + v[0] * self.scale, self.cy - v[1] * self.scale]
        }
    }

    /// The line of sight through a point of the sample grid: its start and its direction, a metre of depth a unit.
    fn ray(&self, x: f32, y: f32) -> (Vec3, Vec3) {
        let (a, b) = ((x - self.cx) / self.scale, (self.cy - y) / self.scale);
        let side = add(scale(self.right, a), scale(self.up, b));
        if self.perspective {
            (self.eye, add(self.forward, side))
        } else {
            (add(self.eye, side), self.forward)
        }
    }
}

/// The signed area (doubled) of a, b, p: positive where p is left of a→b with y up. Always worked out from the
/// lesser end of the edge, so that the two triangles on an edge get exactly opposite values on it (no sample
/// falls between them).
fn edge(a: [f32; 2], b: [f32; 2], p: [f32; 2]) -> f32 {
    let raw =
        |a: [f32; 2], b: [f32; 2]| (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]);
    if (a[0], a[1]) > (b[0], b[1]) {
        -raw(b, a)
    } else {
        raw(a, b)
    }
}

/// A triangle on the sample grid, near-clipped: its corners, their depths and the weights (of the first two
/// corners) of the triangle it was cut from at each.
struct Tri {
    at: [[f32; 2]; 3],
    setup: Setup,
    depth: [f32; 3],
    weights: [[f32; 2]; 3],
    mesh: u32,
    /// The first of the source triangle's indices.
    first: u32,
    back: bool,
}

/// A triangle's edges made ready for testing samples against: each edge from its lesser end (see `edge`), which
/// way that turns it, and whether a sample exactly on it is the triangle's.
struct Setup {
    /// (start x, start y, run x, run y, ±1: the edge's own direction) for the edges opposite each corner
    edges: [[f32; 5]; 3],
    /// Whether a sample on the edge opposite each corner is this triangle's: the one of the two triangles on an
    /// edge whose inside lies to the left of it, taken from its lesser end.
    ties: [bool; 3],
    area: f32,
}

impl Setup {
    fn new(at: [[f32; 2]; 3]) -> Option<Setup> {
        let [a, b, c] = at;
        let area = edge(a, b, c);
        if area == 0.0 || !area.is_finite() {
            return None;
        }
        let ends = [(b, c, a), (c, a, b), (a, b, c)];
        let mut edges = [[0.0; 5]; 3];
        let mut ties = [false; 3];
        for (k, &(e0, e1, opposite)) in ends.iter().enumerate() {
            let swapped = (e0[0], e0[1]) > (e1[0], e1[1]);
            let (lo, hi) = if swapped { (e1, e0) } else { (e0, e1) };
            edges[k] = [
                lo[0],
                lo[1],
                hi[0] - lo[0],
                hi[1] - lo[1],
                if swapped { -1.0 } else { 1.0 },
            ];
            ties[k] = edge(lo, hi, opposite) > 0.0;
        }
        Some(Setup { edges, ties, area })
    }

    /// Where along the row of samples at `y` the triangle lies, about: (from, to) in x, or None if it misses the
    /// row.
    fn span(&self, y: f32) -> Option<(f32, f32)> {
        let (mut from, mut to) = (f32::NEG_INFINITY, f32::INFINITY);
        for [x0, y0, dx, dy, turn] in self.edges {
            // inside: sign · turn · (dx (y - y0) - dy (x - x0)) ≥ 0, a line in x
            let k = self.area.signum() * turn;
            let (a, b) = (-dy * k, dx * (y - y0) * k + dy * x0 * k);
            if a == 0.0 {
                if b < 0.0 {
                    return None;
                }
            } else if a > 0.0 {
                from = from.max(-b / a);
            } else {
                to = to.min(-b / a);
            }
        }
        (from <= to + 1.0).then_some((from, to))
    }

    /// Where the triangle covers a sample: barycentric weights of its corners (they sum to 1), or None.
    fn cover(&self, p: [f32; 2]) -> Option<[f32; 3]> {
        let mut w = [0.0; 3];
        for k in 0..3 {
            let [x, y, dx, dy, turn] = self.edges[k];
            let raw = dx * (p[1] - y) - dy * (p[0] - x);
            w[k] = raw * turn;
            let s = w[k] * self.area.signum();
            if s < 0.0 || (s == 0.0 && !self.ties[k]) {
                return None;
            }
        }
        Some(w.map(|x| x / self.area))
    }
}

/// A sample's depth and where on its source triangle it lies (weights of its first two corners), perspective-correct.
fn interpolate(t: &Tri, l: [f32; 3], perspective: bool) -> (f32, [f32; 2]) {
    if perspective {
        let q = [l[0] / t.depth[0], l[1] / t.depth[1], l[2] / t.depth[2]];
        let sum = q[0] + q[1] + q[2];
        let w = |k: usize| {
            (q[0] * t.weights[0][k] + q[1] * t.weights[1][k] + q[2] * t.weights[2][k]) / sum
        };
        (1.0 / sum, [w(0), w(1)])
    } else {
        let w = |k: usize| l[0] * t.weights[0][k] + l[1] * t.weights[1][k] + l[2] * t.weights[2][k];
        (
            l[0] * t.depth[0] + l[1] * t.depth[1] + l[2] * t.depth[2],
            [w(0), w(1)],
        )
    }
}

/// The triangles of `meshes` the camera can see the front of (or a two-sided one's back), cut at the near plane.
fn triangles(meshes: &[Mesh], eye: &Eye) -> Vec<Tri> {
    let mut out = Vec::new();
    for (m, mesh) in meshes.iter().enumerate() {
        let view: Vec<Vec3> = mesh.positions.iter().map(|&p| eye.view(p)).collect();
        for (t, idx) in mesh.indices.as_chunks::<3>().0.iter().enumerate() {
            let Some(corners) = idx
                .iter()
                .map(|&i| view.get(i as usize).copied())
                .collect::<Option<Vec<Vec3>>>()
            else {
                continue;
            };
            if corners.iter().all(|c| c[2] < eye.near) || corners.iter().all(|c| c[2] > eye.far) {
                continue;
            }
            // the polygon left of the triangle in front of the near plane, each corner with its source weights
            let source = [[1.0, 0.0], [0.0, 1.0], [0.0, 0.0]];
            let mut poly: Vec<(Vec3, [f32; 2])> = Vec::with_capacity(4);
            for k in 0..3 {
                let (p, q) = (corners[k], corners[(k + 1) % 3]);
                let (wp, wq) = (source[k], source[(k + 1) % 3]);
                let (inp, inq) = (p[2] >= eye.near, q[2] >= eye.near);
                if inp {
                    poly.push((p, wp));
                }
                if inp != inq {
                    let u = (eye.near - p[2]) / (q[2] - p[2]);
                    let at = add(p, scale(sub(q, p), u));
                    poly.push((
                        [at[0], at[1], eye.near],
                        [wp[0] + (wq[0] - wp[0]) * u, wp[1] + (wq[1] - wp[1]) * u],
                    ));
                }
            }
            for k in 1..poly.len().saturating_sub(1) {
                let corners = [poly[0], poly[k], poly[k + 1]];
                let at = corners.map(|(v, _)| eye.screen(v));
                let Some(setup) = Setup::new(at) else {
                    continue;
                };
                // (counter-clockwise seen from the eye is the front: clockwise on a grid whose y runs down)
                let back = setup.area > 0.0;
                if back && !mesh.surface.two_sided {
                    continue;
                }
                out.push(Tri {
                    at,
                    setup,
                    depth: corners.map(|(v, _)| v[2]),
                    weights: corners.map(|(_, w)| w),
                    mesh: m as u32,
                    first: (t * 3) as u32,
                    back,
                });
            }
        }
    }
    out
}

/// What a sample of a band holds: the nearest depth, the triangle there and where on it.
#[derive(Clone, Copy)]
struct Sample {
    depth: f32,
    tri: u32,
    weights: [f32; 2],
}

const EMPTY: Sample = Sample {
    depth: f32::INFINITY,
    tri: u32::MAX,
    weights: [0.0; 2],
};

/// The source triangle's corners: indices into its mesh's vertices.
fn corners(mesh: &Mesh, first: u32) -> [usize; 3] {
    let f = first as usize;
    [
        mesh.indices[f] as usize,
        mesh.indices[f + 1] as usize,
        mesh.indices[f + 2] as usize,
    ]
}

/// A vertex attribute at a point of a triangle: its corners' values, weighted.
fn blend<const N: usize>(v: &[[f32; N]], c: [usize; 3], w: [f32; 3]) -> [f32; N] {
    std::array::from_fn(|k| v[c[0]][k] * w[0] + v[c[1]][k] * w[1] + v[c[2]][k] * w[2])
}

/// How close the model comes to the floor, seen from below: a grid over the floor's disc, each cell how dark the
/// floor is there (0 to 1) before `contact` scales it.
struct Contact {
    cells: Vec<f32>,
    n: usize,
    lo: [f32; 2],
    size: f32,
}

impl Contact {
    const CELLS: usize = 128;

    fn new(meshes: &[Mesh], floor: &Floor) -> Contact {
        let n = Contact::CELLS;
        let size = floor.radius * 2.0;
        let lo = [
            floor.centre[0] - floor.radius,
            floor.centre[1] - floor.radius,
        ];
        let mut height = vec![f32::INFINITY; n * n];
        let cell = size / n as f32;
        for mesh in meshes {
            for idx in mesh.indices.as_chunks::<3>().0 {
                let Some(p) = idx
                    .iter()
                    .map(|&i| mesh.positions.get(i as usize).copied())
                    .collect::<Option<Vec<Vec3>>>()
                else {
                    continue;
                };
                let at: [[f32; 2]; 3] =
                    std::array::from_fn(|k| [(p[k][0] - lo[0]) / cell, (p[k][2] - lo[1]) / cell]);
                let Some(setup) = Setup::new(at) else {
                    continue;
                };
                let cells = |k: usize| {
                    let (lo, hi) = range(at.map(|a| a[k]));
                    (
                        lo.floor().max(0.0) as usize,
                        hi.ceil().min(n as f32) as usize,
                    )
                };
                let ((x0, x1), (y0, y1)) = (cells(0), cells(1));
                for y in y0..y1 {
                    for x in x0..x1 {
                        if let Some(l) = setup.cover([x as f32 + 0.5, y as f32 + 0.5]) {
                            let h = l[0] * p[0][1] + l[1] * p[1][1] + l[2] * p[2][1];
                            let cell = &mut height[y * n + x];
                            *cell = cell.min((h - floor.height).max(0.0));
                        }
                    }
                }
            }
        }
        let mut cells: Vec<f32> = height
            .iter()
            .map(|&h| {
                if h.is_finite() {
                    libm::expf(-h / floor.reach)
                } else {
                    0.0
                }
            })
            .collect();
        // a soft edge: a box blur twice each way, about `reach` wide
        let r = ((floor.reach / cell) as usize).clamp(1, n / 4);
        for _ in 0..2 {
            cells = blur(&cells, n, r, true);
            cells = blur(&cells, n, r, false);
        }
        Contact { cells, n, lo, size }
    }

    /// How dark the floor is at (x, z), blended between cells.
    fn at(&self, x: f32, z: f32) -> f32 {
        let n = self.n;
        let gx = (x - self.lo[0]) / self.size * n as f32 - 0.5;
        let gy = (z - self.lo[1]) / self.size * n as f32 - 0.5;
        let (fx, fy) = (gx.floor(), gy.floor());
        let (tx, ty) = (gx - fx, gy - fy);
        let get = |x: f32, y: f32| {
            if x < 0.0 || y < 0.0 || x >= n as f32 || y >= n as f32 {
                0.0
            } else {
                self.cells[y as usize * n + x as usize]
            }
        };
        let top = get(fx, fy) + (get(fx + 1.0, fy) - get(fx, fy)) * tx;
        let bottom = get(fx, fy + 1.0) + (get(fx + 1.0, fy + 1.0) - get(fx, fy + 1.0)) * tx;
        top + (bottom - top) * ty
    }
}

/// The least and the most of three numbers.
fn range(v: [f32; 3]) -> (f32, f32) {
    (v[0].min(v[1]).min(v[2]), v[0].max(v[1]).max(v[2]))
}

/// A box blur of `r` cells each side along rows (or columns), the grid's edge taken as empty.
fn blur(cells: &[f32], n: usize, r: usize, rows: bool) -> Vec<f32> {
    let mut out = vec![0.0; n * n];
    let width = (2 * r + 1) as f32;
    for line in 0..n {
        let at = |i: usize| if rows { line * n + i } else { i * n + line };
        for i in 0..n {
            let mut sum = 0.0;
            for j in i.saturating_sub(r)..(i + r + 1).min(n) {
                sum += cells[at(j)];
            }
            out[at(i)] = sum / width;
        }
    }
    out
}

/// Sample rows worked at once (the z-buffer is a band of the picture at a time).
const BAND: usize = 128;

/// Draws `meshes` (with `textures`, which their surfaces index) from `camera` into a picture `width` × `height`.
pub fn render(
    meshes: &[Mesh],
    textures: &[Texture],
    camera: &Camera,
    look: &Look,
    width: usize,
    height: usize,
) -> Picture {
    let s = look.samples.clamp(1, 8);
    let (sw, sh) = (width * s, height * s);
    let eye = Eye::new(camera, sw, sh);
    let srgb: [f32; 256] = std::array::from_fn(|i| decode(i as f32 / 255.0));
    let to_srgb: Vec<u8> = (0..4096)
        .map(|i| (encode(i as f32 / 4095.0) * 255.0).round() as u8)
        .collect();
    let tris = triangles(meshes, &eye);
    let mut bands: Vec<Vec<u32>> = vec![Vec::new(); sh.div_ceil(BAND)];
    for (i, t) in tris.iter().enumerate() {
        let (lo, hi) = range(t.at.map(|a| a[1]));
        if hi < 0.0 || lo > sh as f32 {
            continue;
        }
        let first = (lo.max(0.0) as usize / BAND).min(bands.len() - 1);
        let last = (hi.min(sh as f32 - 1.0).max(0.0) as usize / BAND).min(bands.len() - 1);
        for band in &mut bands[first..=last] {
            band.push(i as u32);
        }
    }
    let contact = look.floor.map(|f| (f, Contact::new(meshes, &f)));
    let texture = |m: &Mesh| m.surface.texture.and_then(|t| textures.get(t));
    let mut out = Picture::filled(width, height, [0, 0, 0, 0]);
    let mut buffer = vec![EMPTY; sw * BAND];
    for (b, band) in bands.iter().enumerate() {
        let (y0, y1) = (b * BAND, ((b + 1) * BAND).min(sh));
        buffer.fill(EMPTY);
        for &ti in band {
            let t = &tris[ti as usize];
            let mesh = &meshes[t.mesh as usize];
            // the samples (centres at i + 0.5) inside its box
            let column = |v: f32| (v - 0.5).ceil().clamp(0.0, sw as f32) as usize;
            let row = |v: f32| (v - 0.5).ceil().clamp(y0 as f32, y1 as f32) as usize;
            let (xs, ys) = (range(t.at.map(|a| a[0])), range(t.at.map(|a| a[1])));
            let (cx0, cx1) = (column(xs.0), column(xs.1 + 1.0));
            let (cy0, cy1) = (row(ys.0), row(ys.1 + 1.0));
            let cut = mesh
                .surface
                .cutout
                .zip(texture(mesh))
                .filter(|_| !mesh.uvs.is_empty());
            for y in cy0..cy1 {
                let Some((sx0, sx1)) = t.setup.span(y as f32 + 0.5) else {
                    continue;
                };
                // (a sample either side of the span worked out, which the test below settles exactly)
                let x0 = column(sx0 - 1.0).max(cx0);
                let x1 = column(sx1 + 2.0).min(cx1);
                for x in x0..x1 {
                    let Some(l) = t.setup.cover([x as f32 + 0.5, y as f32 + 0.5]) else {
                        continue;
                    };
                    let (depth, w) = interpolate(t, l, eye.perspective);
                    let at = (y - y0) * sw + x;
                    if depth < eye.near || depth > eye.far || depth >= buffer[at].depth {
                        continue;
                    }
                    if let Some((cutoff, tex)) = cut {
                        let c = corners(mesh, t.first);
                        let uv = blend(&mesh.uvs, c, [w[0], w[1], 1.0 - w[0] - w[1]]);
                        if tex.sample(uv, &srgb)[3] < cutoff {
                            continue;
                        }
                    }
                    buffer[at] = Sample {
                        depth,
                        tri: ti,
                        weights: w,
                    };
                }
            }
        }
        // each pixel: its samples counted by what they show (a triangle, the floor, nothing), each triangle
        // shaded once where its samples are on average, the floor once at the pixel's middle
        for py in (y0 / s)..y1.div_ceil(s).min(height) {
            for px in 0..width {
                let mut shown = [(u32::MAX, 0.0f32, [0.0f32; 2]); 64];
                let mut kinds = 0;
                let (mut on_floor, mut empty) = (0.0, 0.0);
                for sy in py * s..(py + 1) * s {
                    for sx in px * s..(px + 1) * s {
                        let sample = buffer[(sy - y0) * sw + sx];
                        if let Some((floor, _)) = &contact
                            && floor_at(floor, &eye, sx as f32 + 0.5, sy as f32 + 0.5)
                                .is_some_and(|t| t < sample.depth)
                        {
                            on_floor += 1.0;
                        } else if sample.tri == u32::MAX {
                            empty += 1.0;
                        } else {
                            match shown[..kinds].iter_mut().find(|g| g.0 == sample.tri) {
                                Some(g) => {
                                    g.1 += 1.0;
                                    g.2 = [g.2[0] + sample.weights[0], g.2[1] + sample.weights[1]];
                                }
                                None => {
                                    shown[kinds] = (sample.tri, 1.0, sample.weights);
                                    kinds += 1;
                                }
                            }
                        }
                    }
                }
                let mut sum = [0.0f32; 4];
                let mut add = |c: [f32; 4], n: f32| {
                    let a = c[3] * n;
                    sum = [
                        sum[0] + c[0] * a,
                        sum[1] + c[1] * a,
                        sum[2] + c[2] * a,
                        sum[3] + a,
                    ];
                };
                for &(tri, n, w) in &shown[..kinds] {
                    let t = &tris[tri as usize];
                    add(
                        surface(t, [w[0] / n, w[1] / n], meshes, textures, look, &srgb),
                        n,
                    );
                }
                if on_floor > 0.0
                    && let Some((floor, contact)) = &contact
                {
                    let middle = (
                        (px * s) as f32 + s as f32 / 2.0,
                        (py * s) as f32 + s as f32 / 2.0,
                    );
                    add(
                        floor_colour(floor, contact, &eye, middle, look.background),
                        on_floor,
                    );
                }
                if empty > 0.0 {
                    add(look.background, empty);
                }
                let n = (s * s) as f32;
                let rgb = if sum[3] > 0.0 {
                    [sum[0] / sum[3], sum[1] / sum[3], sum[2] / sum[3]]
                } else {
                    [look.background[0], look.background[1], look.background[2]]
                };
                let byte = |c: f32| to_srgb[(c.clamp(0.0, 1.0) * 4095.0).round() as usize];
                let i = (py * width + px) * 4;
                out.rgba[i..i + 4].copy_from_slice(&[
                    byte(rgb[0]),
                    byte(rgb[1]),
                    byte(rgb[2]),
                    ((sum[3] / n).clamp(0.0, 1.0) * 255.0).round() as u8,
                ]);
            }
        }
    }
    out
}

/// Where the line of sight through a sample meets the floor, from above and inside its disc: its depth.
fn floor_at(floor: &Floor, eye: &Eye, x: f32, y: f32) -> Option<f32> {
    let (from, dir) = eye.ray(x, y);
    if dir[1] >= 0.0 {
        return None;
    }
    let t = (floor.height - from[1]) / dir[1];
    let (dx, dz) = (
        from[0] + dir[0] * t - floor.centre[0],
        from[2] + dir[2] * t - floor.centre[1],
    );
    (t > eye.near && dx * dx + dz * dz < floor.radius * floor.radius).then_some(t)
}

/// The floor's colour (linear RGB) and alpha seen through a point of the sample grid: darker where the model
/// comes near it, fading into the background towards its edge.
fn floor_colour(
    floor: &Floor,
    contact: &Contact,
    eye: &Eye,
    (x, y): (f32, f32),
    background: [f32; 4],
) -> [f32; 4] {
    let (from, dir) = eye.ray(x, y);
    let t = if dir[1] < 0.0 {
        (floor.height - from[1]) / dir[1]
    } else {
        0.0
    };
    let p = add(from, scale(dir, t));
    let (dx, dz) = (p[0] - floor.centre[0], p[2] - floor.centre[1]);
    let d = (dx * dx + dz * dz).sqrt() / floor.radius;
    let dark = 1.0 - floor.contact * contact.at(p[0], p[2]).min(1.0);
    let fade = smoothstep(0.5, 1.0, d);
    let c = scale(floor.colour, dark);
    let bg = background;
    [
        c[0] + (bg[0] - c[0]) * fade,
        c[1] + (bg[1] - c[1]) * fade,
        c[2] + (bg[2] - c[2]) * fade,
        1.0 + (bg[3] - 1.0) * fade,
    ]
}

/// The colour (linear RGB) and alpha of triangle `t`'s surface at `weights` (of its source's first two corners).
fn surface(
    t: &Tri,
    weights: [f32; 2],
    meshes: &[Mesh],
    textures: &[Texture],
    look: &Look,
    srgb: &[f32; 256],
) -> [f32; 4] {
    let mesh = &meshes[t.mesh as usize];
    let c = corners(mesh, t.first);
    let w = [weights[0], weights[1], 1.0 - weights[0] - weights[1]];
    let grey = if mesh.shading.is_empty() {
        1.0
    } else {
        mesh.shading[c[0]] * w[0] + mesh.shading[c[1]] * w[1] + mesh.shading[c[2]] * w[2]
    };
    if look.shading == Shading::Baked {
        return [grey, grey, grey, 1.0];
    }
    let mut colour = mesh.surface.colour;
    if let Some(tex) = mesh.surface.texture.and_then(|i| textures.get(i))
        && !mesh.uvs.is_empty()
    {
        let texel = tex.sample(blend(&mesh.uvs, c, w), srgb);
        colour = mul(colour, [texel[0], texel[1], texel[2]]);
    }
    let Shading::Lit {
        sun,
        sun_strength,
        sky,
        ground,
    } = look.shading
    else {
        return [colour[0], colour[1], colour[2], 1.0];
    };
    let p = [
        mesh.positions[c[0]],
        mesh.positions[c[1]],
        mesh.positions[c[2]],
    ];
    let mut n = if mesh.normals.is_empty() {
        unit(cross(sub(p[1], p[0]), sub(p[2], p[0])))
    } else {
        unit(blend(&mesh.normals, c, w))
    };
    if t.back {
        n = scale(n, -1.0);
    }
    let up = n[1] * 0.5 + 0.5;
    let light = ground + (sky - ground) * up + sun_strength * dot(n, sun).max(0.0);
    let k = grey * light;
    [colour[0] * k, colour[1] * k, colour[2] * k, 1.0]
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(background: [f32; 4]) -> Look {
        Look {
            background,
            shading: Shading::Flat,
            floor: None,
            samples: 4,
        }
    }

    /// Looking down -z from z = 5 at the origin, 2 m across.
    fn above() -> Camera {
        Camera {
            eye: [0.0, 0.0, 5.0],
            target: [0.0; 3],
            up: [0.0, 1.0, 0.0],
            projection: Projection::Orthographic { span: 2.0 },
            near: 0.1,
            far: 10.0,
        }
    }

    fn triangle(points: [Vec3; 3], colour: Vec3) -> Mesh {
        Mesh {
            positions: points.to_vec(),
            indices: vec![0, 1, 2],
            surface: Surface {
                colour,
                ..Surface::default()
            },
            ..Mesh::default()
        }
    }

    const CCW: [Vec3; 3] = [[-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [0.0, 1.0, 0.0]];

    #[test]
    fn a_triangle_is_drawn_from_its_front_and_not_its_back_unless_two_sided() {
        let look = flat([0.0, 0.0, 0.0, 1.0]);
        let front = render(&[triangle(CCW, [1.0; 3])], &[], &above(), &look, 16, 16);
        assert_eq!(front.at(8, 9), [255, 255, 255, 255]);
        let [a, b, c] = CCW;
        let mut back = triangle([a, c, b], [1.0; 3]);
        assert_eq!(
            render(&[back.clone()], &[], &above(), &look, 16, 16).at(8, 9),
            [0, 0, 0, 255]
        );
        back.surface.two_sided = true;
        assert_eq!(
            render(&[back], &[], &above(), &look, 16, 16).at(8, 9),
            [255, 255, 255, 255]
        );
    }

    #[test]
    fn the_nearer_of_two_triangles_is_drawn_whichever_comes_first() {
        let look = flat([0.0; 4]);
        let lift = |z: f32| CCW.map(|p| [p[0], p[1], z]);
        let near = triangle(lift(1.0), [0.0, 1.0, 0.0]);
        let far = triangle(lift(0.0), [1.0, 0.0, 0.0]);
        for meshes in [[near.clone(), far.clone()], [far, near]] {
            assert_eq!(
                render(&meshes, &[], &above(), &look, 16, 16).at(8, 9),
                [0, 255, 0, 255]
            );
        }
    }

    #[test]
    fn two_triangles_on_an_edge_cover_every_sample_once() {
        // a square past the picture's edges, cut along a diagonal through sample centres
        let quad = Mesh {
            positions: vec![
                [-1.5, -1.5, 0.0],
                [1.5, -1.5, 0.0],
                [1.5, 1.5, 0.0],
                [-1.5, 1.5, 0.0],
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
            surface: Surface {
                colour: [1.0; 3],
                ..Surface::default()
            },
            ..Mesh::default()
        };
        let picture = render(&[quad], &[], &above(), &flat([0.0; 4]), 32, 32);
        assert!(picture.rgba.chunks(4).all(|p| p == [255, 255, 255, 255]));
    }

    #[test]
    fn a_cut_out_surface_drops_the_samples_under_its_cut() {
        // a texture clear on its left half: the quad's left half isn't drawn
        let texture = Texture {
            width: 4,
            height: 1,
            rgba: [
                [255, 0, 0, 0],
                [255, 0, 0, 0],
                [255, 0, 0, 255],
                [255, 0, 0, 255],
            ]
            .concat(),
        };
        let quad = Mesh {
            positions: vec![
                [-1.0, -1.0, 0.0],
                [1.0, -1.0, 0.0],
                [1.0, 1.0, 0.0],
                [-1.0, 1.0, 0.0],
            ],
            uvs: vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
            indices: vec![0, 1, 2, 0, 2, 3],
            surface: Surface {
                colour: [1.0; 3],
                texture: Some(0),
                cutout: Some(0.5),
                two_sided: false,
            },
            ..Mesh::default()
        };
        let picture = render(&[quad], &[texture], &above(), &flat([0.0; 4]), 16, 16);
        assert_eq!(picture.at(2, 8)[3], 0);
        assert_eq!(picture.at(13, 8), [255, 0, 0, 255]);
    }

    #[test]
    fn a_triangle_through_the_eye_is_cut_at_the_near_plane() {
        // a floor-like triangle running from behind the eye to far ahead, in perspective
        let camera = Camera {
            eye: [0.0, 1.0, 0.0],
            target: [0.0, 0.0, -5.0],
            up: [0.0, 1.0, 0.0],
            projection: Projection::Perspective { fov: 1.2 },
            near: 0.1,
            far: 50.0,
        };
        let ground = triangle(
            [[-20.0, 0.0, 10.0], [20.0, 0.0, 10.0], [0.0, 0.0, -40.0]],
            [1.0; 3],
        );
        let picture = render(&[ground], &[], &camera, &flat([0.0; 4]), 32, 32);
        assert_eq!(picture.at(16, 31), [255, 255, 255, 255], "under the eye");
        assert_eq!(picture.at(16, 0)[3], 0, "the sky");
    }

    #[test]
    fn the_floor_is_dark_under_what_stands_on_it_and_clear_away_from_it() {
        let floor = Floor {
            height: 0.0,
            centre: [0.0, 0.0],
            radius: 2.0,
            colour: [0.5; 3],
            contact: 0.5,
            reach: 0.1,
        };
        // a small square lying on the floor at the middle
        let pad = Mesh {
            positions: vec![
                [-0.2, 0.0, -0.2],
                [0.2, 0.0, -0.2],
                [0.2, 0.0, 0.2],
                [-0.2, 0.0, 0.2],
            ],
            indices: vec![0, 2, 1, 0, 3, 2],
            ..Mesh::default()
        };
        let contact = Contact::new(&[pad], &floor);
        assert!(contact.at(0.0, 0.0) > 0.5, "{}", contact.at(0.0, 0.0));
        assert!(contact.at(1.5, 0.0) < 0.01);
    }

    #[test]
    fn the_same_triangles_give_the_same_pixels() {
        let look = Look {
            background: [0.1, 0.1, 0.1, 1.0],
            shading: Shading::Lit {
                sun: unit([0.3, 0.8, 0.5]),
                sun_strength: 0.6,
                sky: 0.6,
                ground: 0.3,
            },
            floor: None,
            samples: 4,
        };
        let a = render(
            &[triangle(CCW, [0.7, 0.4, 0.2])],
            &[],
            &above(),
            &look,
            24,
            24,
        );
        let b = render(
            &[triangle(CCW, [0.7, 0.4, 0.2])],
            &[],
            &above(),
            &look,
            24,
            24,
        );
        assert_eq!(a, b);
    }
}

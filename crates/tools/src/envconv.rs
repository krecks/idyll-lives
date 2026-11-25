// `panorama_to_cube`, `cube_polynomial` and `babylon_irradiance` are ported from Babylon.js 9.30
// (`PanoramaToCubeMapTools.ConvertPanoramaToCubemap`, `CubeMapToSphericalPolynomialTools.ConvertCubeMapToSphericalPolynomial`,
// `SphericalPolynomial.FromHarmonics`, `SphericalHarmonics.FromPolynomial` and `preScaleForRendering`).
// Changed for Idyll Lives: in Rust, for the `.pcenv` it writes.
// Babylon.js: Copyright 2023 The Babylon.js team, under the Apache License 2.0; its licence and notice
// are in crates/tools/texts/Babylon.js-LICENSE and Babylon.js-NOTICE, and in a build's THIRD-PARTY-LICENSES.txt.
//! `cargo envconv`: the sky's image-based lighting for PlayCanvas, made from Poly Haven's CC0 HDRI
//! "Belfast Open Field" (1k, Radiance `.hdr`; https://polyhaven.com/a/belfast_open_field). Writes
//! `web/public/assets/env/belfast_open_field.pcenv`, which `web/src/render/pc/env.ts` loads:
//! PlayCanvas builds its environment atlas (reflections) from the cube faces at run time, once per
//! device, and takes the ambient light from the spherical harmonics. The output is committed; the
//! HDRI is downloaded (pinned by its SHA-256) into `web/.assets/env/`, which git ignores.
//!
//! `cargo envconv [in.hdr] [out.pcenv]` (an `in.hdr` of your own isn't downloaded or hash-checked).
//!
//! The game was lit by Babylon.js until its PlayCanvas port, and its look was calibrated on that
//! light, so the conversion makes what Babylon made of the HDRI and then re-expresses it:
//!
//! - Babylon's cube (`panorama_to_cube`): 256² faces, each texel the panorama's nearest texel, as
//!   `HDRCubeTexture` resampled it (the faces of the `.env` the Babylon renderer loaded were these,
//!   its prefiltering leaves mip 0 as it is).
//! - Its ambient (`cube_polynomial`): the sky's irradiance as Babylon's spherical polynomial of
//!   those faces, and then the harmonics Babylon made of the polynomial and lit with
//!   (`babylon_irradiance`): not true harmonics of the sky (for this one about a quarter brighter
//!   straight up and half as bright straight down), but what the game's look was tuned on.
//!
//! The game's world is Babylon's left-handed space, drawn by PlayCanvas under a root mirrored in z
//! (`render/pc/space.ts`), and the Babylon renderer turned the sky round the vertical (`rotationY`,
//! 0.35π; see git history up to f15b839b). Both are baked in here, so PlayCanvas samples the sky as
//! it is without a rotation of its own:
//!
//! - Babylon looked up its cube at `R·g`, `g` the game-space direction and `R` its `rotationY`.
//! - PlayCanvas looks up its environment at `(-d.x, d.y, d.z)` of its world direction `d` (its
//!   cubemaps are mirrored in x), and `g = (d.x, d.y, -d.z)`. So face texel direction `e` of the
//!   PlayCanvas cube shows Babylon's `R·(-e.x, e.y, -e.z)`: a half turn round y, then `R`.
//! - Its ambient harmonics are evaluated at `d` itself (no mirror), so they are Babylon's
//!   irradiance at `R·(d.x, d.y, -d.z)`, re-expressed in PlayCanvas's basis.
//!
//! The `.pcenv`: gzip of an 8-byte magic `IDLLENV1`, the header's length (u32, little endian), the
//! header (JSON: `version`, `size`, `encoding`, `sh`, `source`), then the six faces (+x, -x, +y,
//! -y, +z, -z), `size`² RGBA texels each, row 0 at the face's t = -1, RGBE encoded (PlayCanvas's
//! `decodeRGBE`: `rgb · 2^(a·255 − 128)`).

use crate::download;
use std::f64::consts::PI;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const PCENV_MAGIC: &[u8; 8] = b"IDLLENV1";
/// The HDRI, as Poly Haven serves it, and what it must be.
pub const HDRI_URL: &str =
    "https://dl.polyhaven.org/file/ph-assets/HDRIs/hdr/1k/belfast_open_field_1k.hdr";
pub const HDRI_SHA256: &str = "8acb87e692dc0176f22047a5a12859a83277e183bc91ffaaec5b7837331534b8";
/// Its size: a longer answer is refused as it comes.
pub const HDRI_BYTES: u64 = 1_647_689;
/// The face size of Babylon's cube (`HDRCubeTexture`'s `size`, the `.env`'s width).
pub const BABYLON_SIZE: usize = 256;
/// The faces written (PlayCanvas reprojects them into its 128 lighting source anyway).
pub const OUT_SIZE: usize = 128;
/// Babylon's `CubeTexture.rotationY` for the sky, as the Babylon renderer set it (`loadEnvironment`).
pub const ROTATION_Y: f64 = PI * 0.35;
/// The largest panorama read (a 16k HDRI is 16384 × 8192).
const MAX_SIDE: usize = 1 << 15;
const MAX_TEXELS: usize = 1 << 28;

/// Where the downloaded HDRI is kept (ignored by git, as `web/.assets/` is).
pub fn hdri_path() -> PathBuf {
    crate::repo_root().join("web/.assets/env/belfast_open_field_1k.hdr")
}

pub fn pcenv_path() -> PathBuf {
    crate::repo_root().join("web/public/assets/env/belfast_open_field.pcenv")
}

pub fn main_with(args: &[String]) -> i32 {
    match run(args) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("envconv: {e}");
            1
        }
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let input = match args.first() {
        Some(path) => PathBuf::from(path),
        None => fetch_hdri()?,
    };
    let output = args.get(1).map(PathBuf::from).unwrap_or_else(pcenv_path);
    let bytes = convert_file(&input, &output)?;
    println!(
        "envconv: {} -> {} ({:.0} KB)",
        input.display(),
        output.display(),
        bytes as f64 / 1024.0
    );
    Ok(())
}

/// The pinned HDRI at `hdri_path()`, downloaded if it isn't there or isn't the pinned file.
pub fn fetch_hdri() -> Result<PathBuf, String> {
    let path = hdri_path();
    if download::file_sha256(&path, false).as_deref() == Some(HDRI_SHA256) {
        return Ok(path);
    }
    download::download_to(
        &download::agent(),
        HDRI_URL,
        &path,
        HDRI_SHA256,
        Some(HDRI_BYTES),
    )?;
    println!("envconv: downloaded {HDRI_URL} to {}", path.display());
    Ok(path)
}

/// Converts `input` (a Radiance `.hdr` panorama) into `output` (a `.pcenv`); returns the bytes written.
pub fn convert_file(input: &Path, output: &Path) -> Result<usize, String> {
    let bytes = std::fs::read(input).map_err(|e| format!("{}: {e}", input.display()))?;
    let pano = read_hdr(&bytes).map_err(|e| format!("{}: {e}", input.display()))?;
    let name = input
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let out = write_pcenv(&convert(&babylon_sky(&pano), OUT_SIZE), &name)?;
    std::fs::write(output, &out).map_err(|e| format!("{}: {e}", output.display()))?;
    Ok(out.len())
}

/// An equirectangular panorama in linear RGB, row 0 the file's first (the top of the image).
pub struct Panorama {
    pub width: usize,
    pub height: usize,
    pub texels: Vec<[f32; 3]>,
}

/// A cube in linear RGB: six faces (+x, -x, +y, -y, +z, -z) of `size`² texels, row by row from t = -1.
pub struct Cube {
    pub size: usize,
    pub faces: [Vec<[f32; 3]>; 6],
}

/// What Babylon made of the HDRI: its cube, and the sky's irradiance as its polynomial per colour
/// channel (x, y, z, xx, yy, zz, yz, zx, xy).
pub struct BabylonSky {
    pub cube: Cube,
    pub polynomial: [[f64; 3]; 9],
}

/// The converted environment: PlayCanvas's cube and its ambient harmonics (`ambientSH`, 9 × RGB).
pub struct PcEnv {
    pub cube: Cube,
    pub sh: [[f64; 3]; 9],
}

/// A Radiance `.hdr` (`-Y h +X w`), decoded by `image` (`rgb · 2^(e − 136)`). For run-length
/// encoded files such as Poly Haven's, that is what Babylon's `RGBE_ReadPixels` read; a flat file
/// with texels that look like old-style run markers (1, 1, 1, n) is read as runs, which Babylon
/// didn't. A local `in.hdr` isn't hash-checked, so its size is limited.
pub fn read_hdr(bytes: &[u8]) -> Result<Panorama, String> {
    use image::ImageDecoder;
    let decoder = image::codecs::hdr::HdrDecoder::new(bytes).map_err(|e| format!(".hdr: {e}"))?;
    // `image` rejects any other FORMAT but takes a header with none; Babylon's reader didn't.
    if !decoder
        .metadata()
        .custom_attributes
        .iter()
        .any(|(k, _)| k == "FORMAT")
    {
        return Err(".hdr: no FORMAT=32-bit_rle_rgbe".into());
    }
    let (width, height) = decoder.dimensions();
    let (width, height) = (width as usize, height as usize);
    if width == 0
        || height == 0
        || width > MAX_SIDE
        || height > MAX_SIDE
        || width * height > MAX_TEXELS
    {
        return Err(format!(".hdr: unsupported size {width}x{height}"));
    }
    // Decoded straight into the texels: no second copy of a panorama that may be gigabytes.
    let mut texels = vec![[0.0f32; 3]; width * height];
    decoder
        .read_image(bytemuck::cast_slice_mut(&mut texels))
        .map_err(|e| format!(".hdr: {e}"))?;
    Ok(Panorama {
        width,
        height,
        texels,
    })
}

/// PlayCanvas's RGBE (`encodeRGBE`), as bytes.
pub fn to_rgbe(c: [f32; 3]) -> [u8; 4] {
    let max = c[0].max(c[1]).max(c[2]);
    if max < 1e-32 {
        return [0; 4];
    }
    let e = max.log2().ceil().clamp(-128.0, 127.0);
    let scale = (-e).exp2();
    let q = |v: f32| (v.max(0.0) * scale * 255.0).round().clamp(0.0, 255.0) as u8;
    [q(c[0]), q(c[1]), q(c[2]), (e + 128.0) as u8]
}

/// PlayCanvas's `decodeRGBE`.
pub fn from_rgbe(p: [u8; 4]) -> [f32; 3] {
    if p[3] == 0 {
        return [0.0; 3];
    }
    let s = (p[3] as f32 - 128.0).exp2() / 255.0;
    [p[0] as f32 * s, p[1] as f32 * s, p[2] as f32 * s]
}

/// What Babylon made of the panorama: its cube and its polynomial.
pub fn babylon_sky(pano: &Panorama) -> BabylonSky {
    let cube = panorama_to_cube(pano, BABYLON_SIZE);
    let polynomial = cube_polynomial(&cube);
    BabylonSky { cube, polynomial }
}

type V3 = [f64; 3];
fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn scale(a: V3, s: f64) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
/// Babylon's `Vector3.normalize` (by the length's reciprocal).
fn normalize(v: V3) -> V3 {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l == 0.0 || l == 1.0 {
        v
    } else {
        scale(v, 1.0 / l)
    }
}
/// JavaScript's `Math.round`.
fn js_round(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// Babylon's `ConvertPanoramaToCubemap` (no supersampling, `invertY`): each face from its four
/// corners' directions, one sample per texel taken at its corner (not its centre), the panorama's
/// nearest texel. Faces in the GPU's order, as Babylon uploaded its `right, left, up, down, front,
/// back`.
pub fn panorama_to_cube(pano: &Panorama, size: usize) -> Cube {
    const CORNERS: [[V3; 4]; 6] = [
        [
            [1.0, -1.0, 1.0],
            [-1.0, -1.0, 1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, 1.0],
        ], // right
        [
            [-1.0, -1.0, -1.0],
            [1.0, -1.0, -1.0],
            [-1.0, 1.0, -1.0],
            [1.0, 1.0, -1.0],
        ], // left
        [
            [-1.0, -1.0, -1.0],
            [-1.0, -1.0, 1.0],
            [1.0, -1.0, -1.0],
            [1.0, -1.0, 1.0],
        ], // up
        [
            [1.0, 1.0, -1.0],
            [1.0, 1.0, 1.0],
            [-1.0, 1.0, -1.0],
            [-1.0, 1.0, 1.0],
        ], // down
        [
            [1.0, -1.0, -1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, -1.0],
            [1.0, 1.0, 1.0],
        ], // front
        [
            [-1.0, -1.0, 1.0],
            [-1.0, -1.0, -1.0],
            [-1.0, 1.0, 1.0],
            [-1.0, 1.0, -1.0],
        ], // back
    ];
    let (w, h) = (pano.width, pano.height);
    let at = |v: V3| {
        // `CalcProjectionSpherical`.
        let mut theta = v[2].atan2(v[0]);
        let phi = v[1].clamp(-1.0, 1.0).acos();
        while theta < -PI {
            theta += 2.0 * PI;
        }
        while theta > PI {
            theta -= 2.0 * PI;
        }
        let dx = theta / PI * 0.5 + 0.5;
        let px = js_round(dx * w as f64).clamp(0.0, (w - 1) as f64) as usize;
        let py = js_round(phi / PI * h as f64).clamp(0.0, (h - 1) as f64) as usize;
        pano.texels[(h - py - 1) * w + px]
    };
    let faces = CORNERS.map(|c| {
        let mut face = Vec::with_capacity(size * size);
        let rot1 = scale(sub(c[1], c[0]), 1.0 / size as f64);
        let rot2 = scale(sub(c[3], c[2]), 1.0 / size as f64);
        let dy = 1.0 / size as f64;
        let mut fy = 0.0;
        for _ in 0..size {
            let (mut xv1, mut xv2) = (c[0], c[2]);
            for _ in 0..size {
                face.push(at(normalize(add(scale(sub(xv2, xv1), fy), xv1))));
                xv1 = add(xv1, rot1);
                xv2 = add(xv2, rot2);
            }
            fy += dy;
        }
        face
    });
    Cube { size, faces }
}

/// Babylon's real spherical harmonics basis constants (`SH3ylmBasisConstants`: l00, l1-1, l10, l11,
/// l2-2, l2-1, l20, l21, l22).
fn basis_constants() -> [f64; 9] {
    [
        (1.0 / (4.0 * PI)).sqrt(),
        -(3.0 / (4.0 * PI)).sqrt(),
        (3.0 / (4.0 * PI)).sqrt(),
        -(3.0 / (4.0 * PI)).sqrt(),
        (15.0 / (4.0 * PI)).sqrt(),
        -(15.0 / (4.0 * PI)).sqrt(),
        (5.0 / (16.0 * PI)).sqrt(),
        -(15.0 / (4.0 * PI)).sqrt(),
        (15.0 / (16.0 * PI)).sqrt(),
    ]
}

/// Babylon's `ConvertCubeMapToSphericalPolynomial` (as `HDRCubeTexture` ran it on its float faces):
/// every texel's radiance into harmonics by its solid angle (each colour clamped to 0..4096),
/// scaled to the whole sphere, convolved with the cosine lobe, over π, and then as its polynomial
/// (`SphericalPolynomial.FromHarmonics`).
#[allow(clippy::needless_range_loop)] // (Channels by index, as the harmonics are laid out.)
pub fn cube_polynomial(cube: &Cube) -> [[f64; 3]; 9] {
    // Per face: the normal, then the world axes along a row and down the rows (`_FileFaces`).
    const AXES: [[V3; 3]; 6] = [
        [[1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, -1.0, 0.0]],
        [[-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, -1.0, 0.0]],
        [[0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
        [[0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0]],
        [[0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, -1.0, 0.0]],
        [[0.0, 0.0, -1.0], [-1.0, 0.0, 0.0], [0.0, -1.0, 0.0]],
    ];
    let k = basis_constants();
    let area = |x: f64, y: f64| (x * y).atan2((x * x + y * y + 1.0).sqrt());
    let n = cube.size;
    let du = 2.0 / n as f64;
    let half = 0.5 * du;
    let min_uv = half - 1.0;
    // Each texel's solid angle: the same on every face.
    let mut solids = Vec::with_capacity(n * n);
    let mut v = min_uv;
    for _ in 0..n {
        let mut u = min_uv;
        for _ in 0..n {
            solids.push(
                area(u - half, v - half) - area(u - half, v + half) - area(u + half, v - half)
                    + area(u + half, v + half),
            );
            u += du;
        }
        v += du;
    }
    let mut l = [[0.0f64; 3]; 9];
    let mut total = 0.0;
    for (f, [normal, ax, ay]) in AXES.iter().enumerate() {
        let mut v = min_uv;
        for y in 0..n {
            let mut u = min_uv;
            for x in 0..n {
                let d = normalize(add(add(scale(*ax, u), scale(*ay, v)), *normal));
                let solid = solids[y * n + x];
                let c = cube.faces[f][y * n + x].map(|c| {
                    if c.is_nan() {
                        0.0
                    } else {
                        (c as f64).clamp(0.0, 4096.0)
                    }
                });
                let basis = [
                    1.0,
                    d[1],
                    d[2],
                    d[0],
                    d[0] * d[1],
                    d[1] * d[2],
                    3.0 * d[2] * d[2] - 1.0,
                    d[0] * d[2],
                    d[0] * d[0] - d[1] * d[1],
                ];
                for i in 0..9 {
                    for ch in 0..3 {
                        l[i][ch] += c[ch] * solid * (k[i] * basis[i]);
                    }
                }
                total += solid;
                u += du;
            }
            v += du;
        }
    }
    // The sphere's solid angle over the texels' sum; then the cosine lobe's convolution
    // (`SHCosKernelConvolution`), and 1/π.
    let correction = 4.0 * PI / total;
    let kernel = [
        PI,
        2.0 * PI / 3.0,
        2.0 * PI / 3.0,
        2.0 * PI / 3.0,
        PI / 4.0,
        PI / 4.0,
        PI / 4.0,
        PI / 4.0,
        PI / 4.0,
    ];
    for i in 0..9 {
        for ch in 0..3 {
            l[i][ch] = l[i][ch] * correction * kernel[i] * (1.0 / PI);
        }
    }
    // `FromHarmonics`.
    let mut p = [[0.0; 3]; 9];
    for ch in 0..3 {
        let [l00, l1m1, l10, l11, l2m2, l2m1, l20, l21, l22] =
            [0, 1, 2, 3, 4, 5, 6, 7, 8].map(|i| l[i][ch]);
        let poly = [
            -(l11 * 1.02333),
            -(l1m1 * 1.02333),
            l10 * 1.02333,
            l00 * 0.886277 - l20 * 0.247708 + l22 * 0.429043,
            l00 * 0.886277 - l20 * 0.247708 - l22 * 0.429043,
            l00 * 0.886277 + l20 * 0.495417,
            -(l2m1 * 0.858086),
            -(l21 * 0.858086),
            l2m2 * 0.858086,
        ];
        for i in 0..9 {
            p[i][ch] = poly[i] * (1.0 / PI);
        }
    }
    p
}

/// The direction through face `face` at (s, t) in -1..1 (the GPU's cube convention).
pub fn face_dir(face: usize, s: f64, t: f64) -> [f64; 3] {
    let v = match face {
        0 => [1.0, -t, -s],
        1 => [-1.0, -t, s],
        2 => [s, 1.0, t],
        3 => [s, -1.0, -t],
        4 => [s, -t, 1.0],
        _ => [-s, -t, -1.0],
    };
    normalize(v)
}

/// The face and (s, t) in -1..1 a direction goes through (the inverse of `face_dir`).
pub fn dir_face(d: [f64; 3]) -> (usize, f64, f64) {
    let [x, y, z] = d;
    let (ax, ay, az) = (x.abs(), y.abs(), z.abs());
    if ax >= ay && ax >= az {
        if x > 0.0 {
            (0, -z / ax, -y / ax)
        } else {
            (1, z / ax, -y / ax)
        }
    } else if ay >= az {
        if y > 0.0 {
            (2, x / ay, z / ay)
        } else {
            (3, x / ay, -z / ay)
        }
    } else if z > 0.0 {
        (4, x / az, -y / az)
    } else {
        (5, -x / az, -y / az)
    }
}

impl Cube {
    /// Bilinear within the face the direction meets (clamped at its edges).
    pub fn sample(&self, d: [f64; 3]) -> [f64; 3] {
        let (face, s, t) = dir_face(d);
        let n = self.size as f64;
        let fx = ((s + 1.0) * 0.5 * n - 0.5).clamp(0.0, n - 1.0);
        let fy = ((t + 1.0) * 0.5 * n - 0.5).clamp(0.0, n - 1.0);
        let (x0, y0) = (fx.floor() as usize, fy.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(self.size - 1), (y0 + 1).min(self.size - 1));
        let (wx, wy) = (fx - x0 as f64, fy - y0 as f64);
        let at = |x: usize, y: usize| self.faces[face][y * self.size + x];
        let mut out = [0.0; 3];
        for (c, o) in out.iter_mut().enumerate() {
            let top = at(x0, y0)[c] as f64 * (1.0 - wx) + at(x1, y0)[c] as f64 * wx;
            let bottom = at(x0, y1)[c] as f64 * (1.0 - wx) + at(x1, y1)[c] as f64 * wx;
            *o = top * (1.0 - wy) + bottom * wy;
        }
        out
    }
}

/// Babylon's `RotationY(angle)` as its shaders apply it (`reflectionMatrix * dir`).
pub fn rotate_y(angle: f64, v: [f64; 3]) -> [f64; 3] {
    let (s, c) = angle.sin_cos();
    [c * v[0] + s * v[2], v[1], -s * v[0] + c * v[2]]
}

/// Where Babylon looked up the sky for the PlayCanvas cube's direction `e` (see the module docs).
pub fn babylon_dir_for_cube(e: [f64; 3]) -> [f64; 3] {
    rotate_y(ROTATION_Y, [-e[0], e[1], -e[2]])
}

/// Where Babylon looked up its irradiance for the PlayCanvas world normal `n`.
pub fn babylon_dir_for_normal(n: [f64; 3]) -> [f64; 3] {
    rotate_y(ROTATION_Y, [n[0], n[1], -n[2]])
}

pub fn convert(sky: &BabylonSky, size: usize) -> PcEnv {
    // Each texel averages the source texels it covers (2 × 2 of the 256 faces into 128).
    let sub = (sky.cube.size / size).max(1);
    let mut faces: [Vec<[f32; 3]>; 6] = Default::default();
    for (f, face) in faces.iter_mut().enumerate() {
        *face = Vec::with_capacity(size * size);
        for y in 0..size {
            for x in 0..size {
                let mut acc = [0.0; 3];
                for sy in 0..sub {
                    let t = ((y * sub + sy) as f64 + 0.5) / (size * sub) as f64 * 2.0 - 1.0;
                    for sx in 0..sub {
                        let s = ((x * sub + sx) as f64 + 0.5) / (size * sub) as f64 * 2.0 - 1.0;
                        let c = sky.cube.sample(babylon_dir_for_cube(face_dir(f, s, t)));
                        for i in 0..3 {
                            acc[i] += c[i];
                        }
                    }
                }
                let k = 1.0 / (sub * sub) as f64;
                face.push([
                    (acc[0] * k) as f32,
                    (acc[1] * k) as f32,
                    (acc[2] * k) as f32,
                ]);
            }
        }
    }
    PcEnv {
        cube: Cube { size, faces },
        sh: sh_from_polynomial(&babylon_irradiance(&sky.polynomial)),
    }
}

/// The irradiance Babylon's PBR lit with, as a polynomial of the same form as `cube_polynomial`'s:
/// not that polynomial itself but the spherical harmonics Babylon made of it for its shaders
/// (`SPHERICAL_HARMONICS`, on by default: `SphericalHarmonics.FromPolynomial`, then
/// `preScaleForRendering`, evaluated by `harmonicsFunctions`' `computeEnvironmentIrradiance`).
/// They aren't the same function: the harmonics come out sharper (for this sky about a quarter
/// brighter straight up and half as bright straight down), and they are what the pictures showed.
pub fn babylon_irradiance(p: &[[f64; 3]; 9]) -> [[f64; 3]; 9] {
    let k = basis_constants();
    let mut out = [[0.0; 3]; 9];
    for ch in 0..3 {
        let [x, y, z, xx, yy, zz, yz, zx, xy] = [0, 1, 2, 3, 4, 5, 6, 7, 8].map(|i| p[i][ch]);
        // `FromPolynomial` (l00, l1-1, l10, l11, l2-2, l2-1, l20, l21, l22), times π and the basis constants.
        let l = [
            xx * 0.376127 + yy * 0.376127 + zz * 0.376126,
            -y * 0.977204,
            z * 0.977204,
            -x * 0.977204,
            xy * 1.16538,
            -yz * 1.16538,
            zz * 1.34567 - xx * 0.672834 - yy * 0.672834,
            -zx * 1.16538,
            xx * 1.16538 - yy * 1.16538,
        ];
        let l: [f64; 9] = std::array::from_fn(|i| l[i] * PI * k[i]);
        // l00 + l1-1·y + l10·z + l11·x + l2-2·xy + l2-1·yz + l20·(3z² − 1) + l21·zx + l22·(x² − y²),
        // its constant spread over x² + y² + z² (= 1 on the sphere).
        let c = l[0] - l[6];
        out[0][ch] = l[3];
        out[1][ch] = l[1];
        out[2][ch] = l[2];
        out[3][ch] = c + l[8];
        out[4][ch] = c - l[8];
        out[5][ch] = c + 3.0 * l[6];
        out[6][ch] = l[5];
        out[7][ch] = l[7];
        out[8][ch] = l[4];
    }
    out
}

/// Babylon's irradiance polynomial at `m` (`computeEnvironmentIrradiance`).
pub fn eval_polynomial(p: &[[f64; 3]; 9], m: [f64; 3], ch: usize) -> f64 {
    let [x, y, z] = m;
    p[0][ch] * x
        + p[1][ch] * y
        + p[2][ch] * z
        + p[3][ch] * x * x
        + p[4][ch] * y * y
        + p[5][ch] * z * z
        + p[6][ch] * y * z
        + p[7][ch] * z * x
        + p[8][ch] * x * y
}

/// PlayCanvas's ambient harmonics at `n` (`addAmbient`, `AMBIENTSH`).
pub fn eval_sh(sh: &[[f64; 3]; 9], n: [f64; 3], ch: usize) -> f64 {
    let [x, y, z] = n;
    sh[0][ch]
        + sh[1][ch] * x
        + sh[2][ch] * y
        + sh[3][ch] * z
        + sh[4][ch] * x * z
        + sh[5][ch] * z * y
        + sh[6][ch] * y * x
        + sh[7][ch] * (3.0 * z * z - 1.0)
        + sh[8][ch] * (x * x - y * y)
}

/// The polynomial, evaluated at `babylon_dir_for_normal(n)`, as PlayCanvas's harmonics of `n`.
///
/// It is a quadratic form `b·m + mᵀAm` with `m = M n` (`M` linear), so in `n` it is
/// `(Mᵀb)·n + nᵀ(MᵀAM)n`, and on the unit sphere (x² + y² + z² = 1) that is exactly the nine
/// functions PlayCanvas evaluates.
#[allow(clippy::needless_range_loop)] // (Matrix arithmetic by index.)
pub fn sh_from_polynomial(p: &[[f64; 3]; 9]) -> [[f64; 3]; 9] {
    // M's columns: where the axes go.
    let cols = [
        babylon_dir_for_normal([1.0, 0.0, 0.0]),
        babylon_dir_for_normal([0.0, 1.0, 0.0]),
        babylon_dir_for_normal([0.0, 0.0, 1.0]),
    ];
    let m = |r: usize, c: usize| cols[c][r];
    let mut sh = [[0.0; 3]; 9];
    for ch in 0..3 {
        let b = [p[0][ch], p[1][ch], p[2][ch]];
        let a = [
            [p[3][ch], p[8][ch] / 2.0, p[7][ch] / 2.0],
            [p[8][ch] / 2.0, p[4][ch], p[6][ch] / 2.0],
            [p[7][ch] / 2.0, p[6][ch] / 2.0, p[5][ch]],
        ];
        let mut lin = [0.0; 3];
        let mut q = [[0.0; 3]; 3];
        for i in 0..3 {
            for k in 0..3 {
                lin[i] += m(k, i) * b[k];
            }
            for j in 0..3 {
                for k in 0..3 {
                    for l in 0..3 {
                        q[i][j] += m(k, i) * a[k][l] * m(l, j);
                    }
                }
            }
        }
        // x²·Axx + y²·Ayy + z²·Azz = α + β(3z² − 1) + γ(x² − y²) on the sphere.
        let half = (q[0][0] + q[1][1]) / 2.0;
        let beta = (q[2][2] - half) / 3.0;
        sh[0][ch] = half + beta;
        sh[1][ch] = lin[0];
        sh[2][ch] = lin[1];
        sh[3][ch] = lin[2];
        sh[4][ch] = 2.0 * q[0][2];
        sh[5][ch] = 2.0 * q[2][1];
        sh[6][ch] = 2.0 * q[1][0];
        sh[7][ch] = beta;
        sh[8][ch] = (q[0][0] - q[1][1]) / 2.0;
    }
    sh
}

pub fn write_pcenv(env: &PcEnv, source: &str) -> Result<Vec<u8>, String> {
    let sh: Vec<f64> = env
        .sh
        .iter()
        .flatten()
        .map(|v| (v * 1e7).round() / 1e7)
        .collect();
    let header = serde_json::json!({ "version": 1, "size": env.cube.size, "encoding": "rgbe", "sh": sh, "source": source });
    let header = serde_json::to_vec(&header).map_err(|e| e.to_string())?;
    let mut raw = Vec::with_capacity(16 + header.len() + 6 * env.cube.size * env.cube.size * 4);
    raw.extend_from_slice(PCENV_MAGIC);
    raw.extend_from_slice(&(header.len() as u32).to_le_bytes());
    raw.extend_from_slice(&header);
    for face in &env.cube.faces {
        for &c in face {
            raw.extend_from_slice(&to_rgbe(c));
        }
    }
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
    gz.write_all(&raw).map_err(|e| e.to_string())?;
    gz.finish().map_err(|e| e.to_string())
}

/// Reads a `.pcenv` back: its cube (decoded) and harmonics.
pub fn read_pcenv(bytes: &[u8]) -> Result<PcEnv, String> {
    let mut raw = Vec::new();
    std::io::Read::read_to_end(&mut flate2::read::GzDecoder::new(bytes), &mut raw)
        .map_err(|e| format!(".pcenv: {e}"))?;
    if raw.len() < 12 || &raw[..8] != PCENV_MAGIC {
        return Err(".pcenv: magic".into());
    }
    let len = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
    let header: serde_json::Value =
        serde_json::from_slice(raw.get(12..12 + len).ok_or(".pcenv: header")?)
            .map_err(|e| e.to_string())?;
    let size = header["size"].as_u64().ok_or(".pcenv: size")? as usize;
    let shv = header["sh"].as_array().ok_or(".pcenv: sh")?;
    let mut sh = [[0.0; 3]; 9];
    for (i, v) in shv.iter().enumerate().take(27) {
        sh[i / 3][i % 3] = v.as_f64().unwrap_or(0.0);
    }
    let texels = raw.get(12 + len..).ok_or(".pcenv: faces")?;
    if texels.len() != 6 * size * size * 4 {
        return Err(".pcenv: face data size".into());
    }
    let mut faces: [Vec<[f32; 3]>; 6] = Default::default();
    for (f, face) in faces.iter_mut().enumerate() {
        *face = texels[f * size * size * 4..(f + 1) * size * size * 4]
            .chunks(4)
            .map(|p| from_rgbe([p[0], p[1], p[2], p[3]]))
            .collect();
    }
    Ok(PcEnv {
        cube: Cube { size, faces },
        sh,
    })
}

#[cfg(test)]
mod tests;

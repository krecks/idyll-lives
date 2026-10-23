//! `cargo envconv`: the sky's image-based lighting for PlayCanvas, converted from the Babylon `.env`
//! the game has lit its world with (`web/public/assets/env/belfast_open_field.env`, prefiltered
//! offline from a CC0 HDRI). Writes `<name>.pcenv` next to it, which `web/src/render/pc/env.ts`
//! loads: PlayCanvas builds its environment atlas (reflections) from the cube faces at run time,
//! once per device, and takes the ambient light from the spherical harmonics. The output is
//! committed, like the `.env` it comes from; run again only when the `.env` changes.
//!
//! `cargo envconv [in.env] [out.pcenv]`.
//!
//! The game's world is Babylon's left-handed space, drawn by PlayCanvas under a root mirrored in z
//! (`render/pc/space.ts`), and the `.env` was turned round the vertical (`rotationY`, 0.35π in
//! `babylon/materials.ts`). Both are baked in here, so PlayCanvas samples the sky as it is without
//! a rotation of its own:
//!
//! - Babylon looks up the cube at `R·g`, `g` the game-space direction and `R` its `rotationY`.
//! - PlayCanvas looks up its environment at `(-d.x, d.y, d.z)` of its world direction `d` (its
//!   cubemaps are mirrored in x), and `g = (d.x, d.y, -d.z)`. So face texel direction `e` of the
//!   PlayCanvas cube shows Babylon's `R·(-e.x, e.y, -e.z)`: a half turn round y, then `R`.
//! - Its ambient harmonics are evaluated at `d` itself (no mirror), so they are Babylon's
//!   irradiance at `R·(d.x, d.y, -d.z)`, re-expressed in PlayCanvas's basis: the harmonics Babylon
//!   made of the `.env`'s polynomial and lit with (`babylon_irradiance`), not the polynomial.
//!
//! The `.env` (version 2): an 8-byte magic, a JSON manifest ending at a NUL, then PNGs, six faces
//! per mip (+x, -x, +y, -y, +z, -z), bottom row first, RGBD encoded (`rgb^2.2 / a`), and the sky's
//! irradiance as a polynomial (what Babylon's ambient used). Only mip 0 is read: PlayCanvas
//! prefilters its own mips (GGX) from the faces.
//!
//! The `.pcenv`: gzip of an 8-byte magic `IDLLENV1`, the header's length (u32, little endian), the
//! header (JSON: `version`, `size`, `encoding`, `sh`, `source`), then the six faces in the same
//! order, `size`² RGBA texels each, row 0 at the face's t = 0, RGBE encoded (PlayCanvas's
//! `decodeRGBE`: `rgb · 2^(a·255 − 128)`).

use std::f64::consts::PI;
use std::io::Write;
use std::path::{Path, PathBuf};

const ENV_MAGIC: [u8; 8] = [0x86, 0x16, 0x87, 0x96, 0xf6, 0xd6, 0x96, 0x36];
pub const PCENV_MAGIC: &[u8; 8] = b"IDLLENV1";
/// The faces written (PlayCanvas reprojects them into its 128 lighting source anyway).
pub const OUT_SIZE: usize = 128;
/// Babylon's `CubeTexture.rotationY` for the sky (`babylon/materials.ts` `loadEnvironment`).
pub const ROTATION_Y: f64 = PI * 0.35;

pub fn main_with(args: &[String]) -> i32 {
    let root = crate::characters::repo_root();
    let input = args.first().map(PathBuf::from).unwrap_or_else(|| root.join("web/public/assets/env/belfast_open_field.env"));
    let output = args.get(1).map(PathBuf::from).unwrap_or_else(|| input.with_extension("pcenv"));
    match convert_file(&input, &output) {
        Ok(bytes) => {
            println!("envconv: {} -> {} ({:.0} KB)", input.display(), output.display(), bytes as f64 / 1024.0);
            0
        }
        Err(e) => {
            eprintln!("envconv: {e}");
            1
        }
    }
}

/// Converts `input` (a Babylon `.env`) into `output` (a `.pcenv`); returns the bytes written.
pub fn convert_file(input: &Path, output: &Path) -> Result<usize, String> {
    let bytes = std::fs::read(input).map_err(|e| format!("{}: {e}", input.display()))?;
    let env = read_env(&bytes)?;
    let name = input.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let out = write_pcenv(&convert(&env, OUT_SIZE), &name)?;
    std::fs::write(output, &out).map_err(|e| format!("{}: {e}", output.display()))?;
    Ok(out.len())
}

/// A cube in linear RGB: six faces (+x, -x, +y, -y, +z, -z) of `size`² texels, row by row from t = 0.
pub struct Cube {
    pub size: usize,
    pub faces: [Vec<[f32; 3]>; 6],
}

/// What a Babylon `.env` holds that the conversion needs.
pub struct BabylonEnv {
    pub cube: Cube,
    /// The irradiance polynomial per colour channel: x, y, z, xx, yy, zz, yz, zx, xy.
    pub polynomial: [[f64; 3]; 9],
}

/// The converted environment: PlayCanvas's cube and its ambient harmonics (`ambientSH`, 9 × RGB).
pub struct PcEnv {
    pub cube: Cube,
    pub sh: [[f64; 3]; 9],
}

pub fn read_env(bytes: &[u8]) -> Result<BabylonEnv, String> {
    if bytes.len() < 9 || bytes[..8] != ENV_MAGIC {
        return Err("not a Babylon .env (magic)".into());
    }
    let end = bytes[8..].iter().position(|&b| b == 0).ok_or(".env: no end to its manifest")? + 8;
    let manifest: serde_json::Value = serde_json::from_slice(&bytes[8..end]).map_err(|e| format!(".env manifest: {e}"))?;
    let data = end + 1;
    let version = manifest["version"].as_u64().unwrap_or(1);
    if version > 2 {
        return Err(format!(".env version {version}"));
    }
    if manifest["imageType"].as_str().is_some_and(|t| t != "image/png") {
        return Err(format!(".env images are {}, not PNG", manifest["imageType"]));
    }
    let size = manifest["width"].as_u64().ok_or(".env: no width")? as usize;
    let mips = manifest["specular"]["mipmaps"].as_array().ok_or(".env: no mipmaps")?;
    if mips.len() < 6 {
        return Err(".env: fewer than six images".into());
    }
    let mut faces: [Vec<[f32; 3]>; 6] = Default::default();
    for (f, face) in faces.iter_mut().enumerate() {
        let at = data + mips[f]["position"].as_u64().ok_or(".env: no position")? as usize;
        let len = mips[f]["length"].as_u64().ok_or(".env: no length")? as usize;
        let png = bytes.get(at..at + len).ok_or(".env: image out of the file")?;
        let img = image::load_from_memory_with_format(png, image::ImageFormat::Png).map_err(|e| format!(".env face {f}: {e}"))?.to_rgba8();
        if img.width() as usize != size || img.height() as usize != size {
            return Err(format!(".env face {f} is {}x{}, not {size}", img.width(), img.height()));
        }
        // The images are stored bottom row first (as Babylon read them back from the GPU): row 0
        // of the PNG is the face's t = 1. (`committed_sky` checks it against the polynomial.)
        *face = (0..size).flat_map(|y| (0..size).map(move |x| (x, size - 1 - y))).map(|(x, y)| from_rgbd(img.get_pixel(x as u32, y as u32).0)).collect();
    }
    let irr = &manifest["irradiance"];
    let mut polynomial = [[0.0; 3]; 9];
    for (i, key) in ["x", "y", "z", "xx", "yy", "zz", "yz", "zx", "xy"].iter().enumerate() {
        let c = irr[key].as_array().ok_or_else(|| format!(".env: no irradiance polynomial ({key}; spherical harmonics files aren't read)"))?;
        for ch in 0..3 {
            polynomial[i][ch] = c.get(ch).and_then(|v| v.as_f64()).ok_or(".env: bad irradiance")?;
        }
    }
    Ok(BabylonEnv { cube: Cube { size, faces }, polynomial })
}

/// Babylon's RGBD (`fromRGBD`: gamma 2.2 colour over the alpha's range).
pub fn from_rgbd(p: [u8; 4]) -> [f32; 3] {
    let d = p[3] as f32 / 255.0;
    if d <= 0.0 {
        return [0.0; 3];
    }
    let lin = |c: u8| (c as f32 / 255.0).powf(2.2) / d;
    [lin(p[0]), lin(p[1]), lin(p[2])]
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
        if x > 0.0 { (0, -z / ax, -y / ax) } else { (1, z / ax, -y / ax) }
    } else if ay >= az {
        if y > 0.0 { (2, x / ay, z / ay) } else { (3, x / ay, -z / ay) }
    } else if z > 0.0 {
        (4, x / az, -y / az)
    } else {
        (5, -x / az, -y / az)
    }
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / l, v[1] / l, v[2] / l]
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

/// Where Babylon looks up the sky for the PlayCanvas cube's direction `e` (see the module docs).
pub fn babylon_dir_for_cube(e: [f64; 3]) -> [f64; 3] {
    rotate_y(ROTATION_Y, [-e[0], e[1], -e[2]])
}

/// Where Babylon looks up its irradiance for the PlayCanvas world normal `n`.
pub fn babylon_dir_for_normal(n: [f64; 3]) -> [f64; 3] {
    rotate_y(ROTATION_Y, [n[0], n[1], -n[2]])
}

pub fn convert(env: &BabylonEnv, size: usize) -> PcEnv {
    // Each texel averages the source texels it covers (2 × 2 of the 256 faces into 128).
    let sub = (env.cube.size / size).max(1);
    let mut faces: [Vec<[f32; 3]>; 6] = Default::default();
    for (f, face) in faces.iter_mut().enumerate() {
        *face = Vec::with_capacity(size * size);
        for y in 0..size {
            for x in 0..size {
                let mut acc = [0.0; 3];
                for sy in 0..sub {
                    for sx in 0..sub {
                        let s = ((x * sub + sx) as f64 + 0.5) / (size * sub) as f64 * 2.0 - 1.0;
                        let t = ((y * sub + sy) as f64 + 0.5) / (size * sub) as f64 * 2.0 - 1.0;
                        let c = env.cube.sample(babylon_dir_for_cube(face_dir(f, s, t)));
                        for i in 0..3 {
                            acc[i] += c[i];
                        }
                    }
                }
                let k = 1.0 / (sub * sub) as f64;
                face.push([(acc[0] * k) as f32, (acc[1] * k) as f32, (acc[2] * k) as f32]);
            }
        }
    }
    PcEnv { cube: Cube { size, faces }, sh: sh_from_polynomial(&babylon_irradiance(&env.polynomial)) }
}

/// The irradiance Babylon's PBR lit with, as a polynomial of the same form as the `.env`'s: not
/// the `.env`'s polynomial itself but the spherical harmonics Babylon makes of it for its shaders
/// (`SPHERICAL_HARMONICS`, on by default: `SphericalHarmonics.FromPolynomial`, then
/// `preScaleForRendering`, evaluated by `harmonicsFunctions`' `computeEnvironmentIrradiance`).
/// They aren't the same function: the harmonics come out sharper (for this sky about a quarter
/// brighter straight up and half as bright straight down), and they are what the pictures showed.
pub fn babylon_irradiance(p: &[[f64; 3]; 9]) -> [[f64; 3]; 9] {
    let k = [
        (1.0 / (4.0 * PI)).sqrt(),
        -(3.0 / (4.0 * PI)).sqrt(),
        (3.0 / (4.0 * PI)).sqrt(),
        -(3.0 / (4.0 * PI)).sqrt(),
        (15.0 / (4.0 * PI)).sqrt(),
        -(15.0 / (4.0 * PI)).sqrt(),
        (5.0 / (16.0 * PI)).sqrt(),
        -(15.0 / (4.0 * PI)).sqrt(),
        (15.0 / (16.0 * PI)).sqrt(),
    ];
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
    p[0][ch] * x + p[1][ch] * y + p[2][ch] * z + p[3][ch] * x * x + p[4][ch] * y * y + p[5][ch] * z * z + p[6][ch] * y * z + p[7][ch] * z * x + p[8][ch] * x * y
}

/// PlayCanvas's ambient harmonics at `n` (`addAmbient`, `AMBIENTSH`).
pub fn eval_sh(sh: &[[f64; 3]; 9], n: [f64; 3], ch: usize) -> f64 {
    let [x, y, z] = n;
    sh[0][ch] + sh[1][ch] * x + sh[2][ch] * y + sh[3][ch] * z + sh[4][ch] * x * z + sh[5][ch] * z * y + sh[6][ch] * y * x + sh[7][ch] * (3.0 * z * z - 1.0) + sh[8][ch] * (x * x - y * y)
}

/// The polynomial, evaluated at `babylon_dir_for_normal(n)`, as PlayCanvas's harmonics of `n`.
///
/// It is a quadratic form `b·m + mᵀAm` with `m = M n` (`M` linear), so in `n` it is
/// `(Mᵀb)·n + nᵀ(MᵀAM)n`, and on the unit sphere (x² + y² + z² = 1) that is exactly the nine
/// functions PlayCanvas evaluates.
pub fn sh_from_polynomial(p: &[[f64; 3]; 9]) -> [[f64; 3]; 9] {
    // M's columns: where the axes go.
    let cols = [babylon_dir_for_normal([1.0, 0.0, 0.0]), babylon_dir_for_normal([0.0, 1.0, 0.0]), babylon_dir_for_normal([0.0, 0.0, 1.0])];
    let m = |r: usize, c: usize| cols[c][r];
    let mut sh = [[0.0; 3]; 9];
    for ch in 0..3 {
        let b = [p[0][ch], p[1][ch], p[2][ch]];
        let a = [[p[3][ch], p[8][ch] / 2.0, p[7][ch] / 2.0], [p[8][ch] / 2.0, p[4][ch], p[6][ch] / 2.0], [p[7][ch] / 2.0, p[6][ch] / 2.0, p[5][ch]]];
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
    let sh: Vec<f64> = env.sh.iter().flatten().map(|v| (v * 1e7).round() / 1e7).collect();
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
    std::io::Read::read_to_end(&mut flate2::read::GzDecoder::new(bytes), &mut raw).map_err(|e| format!(".pcenv: {e}"))?;
    if raw.len() < 12 || &raw[..8] != PCENV_MAGIC {
        return Err(".pcenv: magic".into());
    }
    let len = u32::from_le_bytes([raw[8], raw[9], raw[10], raw[11]]) as usize;
    let header: serde_json::Value = serde_json::from_slice(raw.get(12..12 + len).ok_or(".pcenv: header")?).map_err(|e| e.to_string())?;
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
        *face = texels[f * size * size * 4..(f + 1) * size * size * 4].chunks(4).map(|p| from_rgbe([p[0], p[1], p[2], p[3]])).collect();
    }
    Ok(PcEnv { cube: Cube { size, faces }, sh })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dirs() -> Vec<[f64; 3]> {
        // A deterministic spread over the sphere (a Fibonacci spiral).
        (0..200)
            .map(|i| {
                let y = 1.0 - 2.0 * (i as f64 + 0.5) / 200.0;
                let r = (1.0 - y * y).sqrt();
                let a = i as f64 * 2.399963229728653;
                [r * a.cos(), y, r * a.sin()]
            })
            .collect()
    }

    fn close(a: [f64; 3], b: [f64; 3], eps: f64) -> bool {
        (0..3).all(|i| (a[i] - b[i]).abs() < eps)
    }

    #[test]
    fn faces_and_directions_round_trip() {
        for d in dirs() {
            let (f, s, t) = dir_face(d);
            assert!(close(face_dir(f, s, t), d, 1e-12), "{d:?}");
        }
        // The GPU's convention: +x's s runs along -z, +y's t along +z, +z's s along +x.
        assert!(close(face_dir(0, 1.0, 0.0), normalize([1.0, 0.0, -1.0]), 1e-12));
        assert!(close(face_dir(2, 0.0, 1.0), normalize([0.0, 1.0, 1.0]), 1e-12));
        assert!(close(face_dir(4, 1.0, 0.0), normalize([1.0, 0.0, 1.0]), 1e-12));
    }

    #[test]
    fn rgbe_and_rgbd_codecs() {
        for c in [[0.01f32, 0.02, 0.03], [1.0, 0.5, 0.25], [40.0, 3.0, 0.1], [250.0, 250.0, 240.0]] {
            let back = from_rgbe(to_rgbe(c));
            let max = c[0].max(c[1]).max(c[2]);
            for i in 0..3 {
                assert!((back[i] - c[i]).abs() <= max / 255.0 + 1e-6, "{c:?} -> {back:?}");
            }
        }
        assert_eq!(from_rgbe(to_rgbe([0.0; 3])), [0.0; 3]);
        // Babylon's RGBD: rgb in gamma 2.2, divided by the range in alpha.
        let c = from_rgbd([255, 128, 0, 51]);
        assert!((c[0] - 5.0).abs() < 1e-4 && (c[1] - (128.0f32 / 255.0).powf(2.2) * 5.0).abs() < 1e-4 && c[2] == 0.0);
    }

    #[test]
    fn harmonics_are_the_polynomial_seen_from_playcanvas() {
        let p = [[0.06, 0.07, 0.08], [0.37, 0.4, 0.5], [0.03, -0.02, 0.04], [0.63, 0.6, 0.66], [0.67, 0.7, 0.71], [0.64, 0.65, 0.68], [0.01, 0.03, -0.02], [0.02, -0.01, 0.009], [0.05, 0.04, -0.03]];
        let sh = sh_from_polynomial(&p);
        for n in dirs() {
            for ch in 0..3 {
                let want = eval_polynomial(&p, babylon_dir_for_normal(n), ch);
                assert!((eval_sh(&sh, n, ch) - want).abs() < 1e-12, "{n:?}");
            }
        }
    }

    /// Babylon's runtime irradiance (its harmonics, `computeEnvironmentIrradiance`) as the polynomial
    /// `babylon_irradiance` gives, at a few directions worked out by hand from Babylon's code.
    #[test]
    fn babylon_irradiance_is_its_harmonics() {
        let p = [[0.06, 0.07, 0.08], [0.37, 0.4, 0.5], [0.03, -0.02, 0.04], [0.63, 0.6, 0.66], [0.67, 0.7, 0.71], [0.64, 0.65, 0.68], [0.01, 0.03, -0.02], [0.02, -0.01, 0.009], [0.05, 0.04, -0.03]];
        let q = babylon_irradiance(&p);
        let (c0, c1, c2) = ((1.0 / (4.0 * PI)).sqrt(), (3.0 / (4.0 * PI)).sqrt(), (5.0 / (16.0 * PI)).sqrt());
        for ch in 0..3 {
            let [x, y, _, xx, yy, zz, ..] = [0, 1, 2, 3, 4, 5, 6, 7, 8].map(|i| p[i][ch]);
            let l00 = (xx + yy) * 0.376127 + zz * 0.376126;
            let l20 = zz * 1.34567 - (xx + yy) * 0.672834;
            // Straight up: l00 + l1-1 − l20 − l22 (each times π and its constant).
            let up = PI * (l00 * c0 + y * 0.977204 * c1 - l20 * c2 - (xx - yy) * 1.16538 * (15.0 / (16.0 * PI)).sqrt());
            assert!((eval_polynomial(&q, [0.0, 1.0, 0.0], ch) - up).abs() < 1e-12);
            // Along +x: l00 + l11 − l20 + l22.
            let east = PI * (l00 * c0 + x * 0.977204 * c1 - l20 * c2 + (xx - yy) * 1.16538 * (15.0 / (16.0 * PI)).sqrt());
            assert!((eval_polynomial(&q, [1.0, 0.0, 0.0], ch) - east).abs() < 1e-12);
        }
    }

    /// A synthetic sky (a bright lobe and a gradient) through the whole conversion: the PlayCanvas
    /// cube shows at `e` what Babylon shows at `babylon_dir_for_cube(e)`.
    #[test]
    fn faces_are_the_sky_seen_from_playcanvas() {
        let sun = normalize([0.6, 0.5, -0.62]);
        let sky = |d: [f64; 3]| {
            let k = (d[0] * sun[0] + d[1] * sun[1] + d[2] * sun[2]).max(0.0).powi(8);
            [(1.0 + d[1] + 4.0 * k) as f32, (1.0 + 0.5 * d[0] + 3.0 * k) as f32, (1.0 + 0.5 * d[2] + 2.0 * k) as f32]
        };
        let size = 64;
        let mut faces: [Vec<[f32; 3]>; 6] = Default::default();
        for (f, face) in faces.iter_mut().enumerate() {
            for y in 0..size {
                for x in 0..size {
                    let s = (x as f64 + 0.5) / size as f64 * 2.0 - 1.0;
                    let t = (y as f64 + 0.5) / size as f64 * 2.0 - 1.0;
                    face.push(sky(face_dir(f, s, t)));
                }
            }
        }
        let env = BabylonEnv { cube: Cube { size, faces }, polynomial: [[0.0; 3]; 9] };
        let pc = convert(&env, 32);
        let back = read_pcenv(&write_pcenv(&pc, "test").unwrap()).unwrap();
        for e in dirs() {
            let want = sky(babylon_dir_for_cube(e));
            let got = back.cube.sample(e);
            for i in 0..3 {
                assert!((got[i] - want[i] as f64).abs() < 0.12 * want[i] as f64 + 0.05, "{e:?}: {got:?} vs {want:?}");
            }
        }
    }

    /// The committed sky: its faces and its harmonics agree on where the light comes from (both in
    /// the `.env` as Babylon reads it, and in the `.pcenv` as PlayCanvas reads it), and the
    /// `.pcenv` is what the `.env` converts to now.
    #[test]
    fn committed_sky() {
        let root = crate::characters::repo_root().join("web/public/assets/env");
        let src = std::fs::read(root.join("belfast_open_field.env")).expect("the .env");
        let env = read_env(&src).unwrap();
        assert_eq!(env.cube.size, 256);
        // The light's mean direction in the faces (every texel, by its solid angle) against the
        // polynomial's linear part (its gradient: the same band of the sky), in Babylon's lookup
        // space; then the same through PlayCanvas's lookups.
        let mean = |cube: &Cube, world: &dyn Fn([f64; 3]) -> [f64; 3]| {
            let mut m = [0.0; 3];
            let n = cube.size;
            for f in 0..6 {
                for y in 0..n {
                    for x in 0..n {
                        let s = (x as f64 + 0.5) / n as f64 * 2.0 - 1.0;
                        let t = (y as f64 + 0.5) / n as f64 * 2.0 - 1.0;
                        let w = (1.0 + s * s + t * t).powf(-1.5);
                        let c = cube.faces[f][y * n + x];
                        let l = w * (0.2126 * c[0] as f64 + 0.7152 * c[1] as f64 + 0.0722 * c[2] as f64);
                        let d = world(face_dir(f, s, t));
                        for i in 0..3 {
                            m[i] += d[i] * l;
                        }
                    }
                }
            }
            normalize(m)
        };
        let faces = mean(&env.cube, &|d| d);
        let poly = normalize([0usize, 1, 2].map(|i| (0..3).map(|ch| env.polynomial[i][ch] * [0.2126, 0.7152, 0.0722][ch]).sum::<f64>()));
        let cos = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        assert!(cos(faces, poly) > 0.99, "faces {faces:?} vs polynomial {poly:?}");
        let horizontal = |v: [f64; 3]| normalize([v[0], 0.0, v[2]]);
        assert!(cos(horizontal(faces), horizontal(poly)) > 0.95, "faces {faces:?} vs polynomial {poly:?}");

        let pc = read_pcenv(&std::fs::read(root.join("belfast_open_field.pcenv")).expect("the .pcenv (cargo envconv)")).unwrap();
        // PlayCanvas's environment lookup for world direction d reads its cube at (-d.x, d.y, d.z),
        // so cube direction e is world direction (-e.x, e.y, e.z).
        let atlas = mean(&pc.cube, &|e| [-e[0], e[1], e[2]]);
        let sh = normalize([1, 2, 3].map(|i| (0..3).map(|ch| pc.sh[i][ch] * [0.2126, 0.7152, 0.0722][ch]).sum::<f64>()));
        assert!(cos(atlas, sh) > 0.99, "cube {atlas:?} vs harmonics {sh:?}");
        assert!(cos(horizontal(atlas), horizontal(sh)) > 0.95, "cube {atlas:?} vs harmonics {sh:?}");

        let fresh = convert(&env, OUT_SIZE);
        for ch in 0..3 {
            for i in 0..9 {
                assert!((fresh.sh[i][ch] - pc.sh[i][ch]).abs() < 1e-6, "the .pcenv is stale: run cargo envconv");
            }
        }
        for f in 0..6 {
            for (a, b) in fresh.cube.faces[f].iter().zip(&pc.cube.faces[f]) {
                let max = a[0].max(a[1]).max(a[2]);
                assert!((0..3).all(|i| (a[i] - b[i]).abs() <= max / 128.0 + 1e-6), "the .pcenv is stale: run cargo envconv");
            }
        }
    }
}

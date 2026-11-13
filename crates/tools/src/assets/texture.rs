//! Textures to KTX2 (Basis Universal, through `basisu_c_sys`: the encoder's own C API), and back to
//! pixels for the check.
//!
//! What a texture is decides how it is encoded (`Role`, from its file name): a colour map, a normal map
//! or packed data (an `arm` map: occlusion, roughness, metal). All three are ETC1S with a mip chain made
//! by the encoder, wrapping at the edges as the game's textures tile. The GPU keeps them block-compressed
//! (BC7, ETC2 or ASTC, whatever it has), 4 to 8 times less memory than RGBA.

use basisu_c_sys::extra::types::Extent3d;
use basisu_c_sys::extra::{
    BasisuEncoder, BasisuEncoderParams, BasisuTranscoder, ChannelType, SourceImage,
    SourceImageFormat, SupportedTextureCompression, basisu_encoder_init, basisu_transcoder_init,
};
use basisu_c_sys::{BasisTextureFormat, common};

/// What a texture holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Color,
    Normal,
    Data,
}

impl Role {
    pub fn name(self) -> &'static str {
        match self {
            Role::Color => "color",
            Role::Normal => "normal",
            Role::Data => "data",
        }
    }

    /// By file name: `normal.jpg` and `*_normal.jpg` are normal maps, `arm.jpg` is data, the rest colour.
    pub fn of(rel: &str) -> Role {
        let file = rel.rsplit('/').next().unwrap_or(rel);
        let stem = file.rsplit_once('.').map_or(file, |(stem, _)| stem);
        if stem.eq_ignore_ascii_case("normal") || stem.to_ascii_lowercase().ends_with("_normal") {
            Role::Normal
        } else if stem.eq_ignore_ascii_case("arm") {
            Role::Data
        } else {
            Role::Color
        }
    }

    /// ETC1S's quality (1 to 255, the encoder's own scale) for this kind of texture.
    fn quality(self) -> u64 {
        match self {
            Role::Color => 160,
            Role::Normal => 224,
            Role::Data => 128,
        }
    }

    /// A colour map is encoded for the eye and carries the sRGB transfer function (the renderer loads it
    /// in a linear format as it does the JPEG it replaces, and decodes it itself: `textureWithFallback`
    /// in `web/src/render/pc/materials.ts`); a normal map and data are linear.
    fn srgb(self) -> bool {
        self == Role::Color
    }
}

/// The settings a role is encoded with, as text: part of a cached copy's key, so a change here encodes
/// its textures again.
pub fn settings(role: Role) -> String {
    format!(
        "etc1s q{} srgb={} mips=wrap ktx2 jpeg=turbo",
        role.quality(),
        role.srgb()
    )
}

/// An image file's pixels as RGBA: (width, height, pixels). A baseline JPEG is read as libjpeg-turbo reads it,
/// to the bit (`tex::jpeg`, as the texture tools do): the pixels the encoder gets are then the ones the
/// JavaScript tool's decoder gave it, and what it makes of them is the same too.
pub fn rgba(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), String> {
    if bytes.starts_with(&[0xFF, 0xD8])
        && let Some((w, h, channels, px)) = crate::tex::jpeg::decode(bytes)
        && (channels == 1 || channels == 3)
    {
        let mut out = Vec::with_capacity(w * h * 4);
        for p in px.chunks_exact(channels) {
            out.extend([p[0], p[channels / 2], p[channels - 1], 255]);
        }
        return Ok((w as u32, h as u32, out));
    }
    let image = image::load_from_memory(bytes)
        .map_err(|e| format!("can't read the image: {e}"))?
        .to_rgba8();
    Ok((image.width(), image.height(), image.into_raw()))
}

/// An image file's bytes (JPEG or PNG) as a KTX2 file.
pub fn encode(bytes: &[u8], role: Role) -> Result<Vec<u8>, String> {
    let (width, height, pixels) = rgba(bytes)?;
    basisu_encoder_init();
    let mut encoder = BasisuEncoder::new();
    encoder
        .set_image(SourceImage {
            data: &pixels,
            format: SourceImageFormat::Rgba8,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        })
        .map_err(|e| format!("the encoder refused the image: {e}"))?;
    // The quality in the flags' low byte, the encoder's own scale (`quality_level` -1: not the unified one).
    // (Not the encoder's own threads: the tool encodes as many files at once as it has cores.)
    let mut flags =
        common::BU_COMP_FLAGS_KTX2_OUTPUT | common::BU_COMP_FLAGS_GEN_MIPS_WRAP | role.quality();
    if role.srgb() {
        flags |= common::BU_COMP_FLAGS_SRGB;
    }
    encoder
        .compress(BasisuEncoderParams {
            basis_tex_format: BasisTextureFormat::Etc1s,
            quality_level: -1,
            effort_level: -1,
            flags_and_quality: flags,
            low_level_uastc_rdo_or_dct_quality: 0.0,
        })
        .map_err(|e| format!("the encoder failed: {e}"))
}

/// A KTX2 file's levels as RGBA pixels, the largest first: (width, height, pixels).
pub fn levels(ktx2: &[u8]) -> Result<Vec<(u32, u32, Vec<u8>)>, String> {
    basisu_transcoder_init();
    let transcoder = BasisuTranscoder::new(
        ktx2,
        SupportedTextureCompression::empty(),
        ChannelType::Auto,
    )
    .map_err(|e| format!("not a KTX2 file the transcoder reads: {e}"))?;
    let info = transcoder.get_info();
    let image = transcoder
        .transcode(None, None)
        .map_err(|e| format!("can't transcode: {e}"))?;
    let mut out = Vec::new();
    let mut at = 0usize;
    for level in 0..info.levels {
        let w = (info.width >> level).max(1);
        let h = (info.height >> level).max(1);
        let n = (w * h * 4) as usize;
        let pixels = image
            .data
            .get(at..at + n)
            .ok_or("the transcoded levels are shorter than the header says")?
            .to_vec();
        out.push((w, h, pixels));
        at += n;
    }
    Ok(out)
}

/// The mean of each of red, green and blue over `pixels` (RGBA), 0 to 255.
pub fn mean(pixels: &[u8]) -> [f64; 3] {
    let mut sum = [0u64; 3];
    for p in pixels.as_chunks::<4>().0.iter() {
        for c in 0..3 {
            sum[c] += p[c] as u64;
        }
    }
    let n = (pixels.len() / 4).max(1) as f64;
    [sum[0] as f64 / n, sum[1] as f64 / n, sum[2] as f64 / n]
}

/// The peak signal-to-noise ratio (dB) of `a` against `b` over red, green and blue (RGBA pixels of one
/// size); infinite when they are the same.
pub fn psnr(a: &[u8], b: &[u8]) -> f64 {
    let mut se = 0f64;
    let mut n = 0usize;
    for (p, q) in a.as_chunks::<4>().0.iter().zip(b.as_chunks::<4>().0.iter()) {
        for c in 0..3 {
            let d = p[c] as f64 - q[c] as f64;
            se += d * d;
        }
        n += 3;
    }
    if se == 0.0 || n == 0 {
        f64::INFINITY
    } else {
        10.0 * (255.0f64 * 255.0 / (se / n as f64)).log10()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roles_by_file_name() {
        assert_eq!(Role::of("textures/brick/normal.jpg"), Role::Normal);
        assert_eq!(Role::of("textures/brick/arm.jpg"), Role::Data);
        assert_eq!(Role::of("textures/brick/albedo.jpg"), Role::Color);
        assert_eq!(Role::of("models/x/leaf_normal.png"), Role::Normal);
        assert_eq!(Role::of("textures/abnormal.jpg"), Role::Color);
        assert_eq!(Role::of("textures/alarm.jpg"), Role::Color);
    }

    #[test]
    fn a_texture_comes_back_close_to_itself_with_every_level() {
        // A 64 px gradient with a little texture: smooth enough for ETC1S, not flat.
        let mut img = image::RgbaImage::new(64, 64);
        for (x, y, p) in img.enumerate_pixels_mut() {
            let v = (x * 3 + y) as u8;
            *p = image::Rgba([v, 255 - v, ((x ^ y) * 4) as u8, 255]);
        }
        let mut png = Vec::new();
        image::DynamicImage::ImageRgba8(img.clone())
            .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .unwrap();
        let ktx2 = encode(&png, Role::Color).unwrap();
        assert_eq!(&ktx2[..12], b"\xABKTX 20\xBB\r\n\x1A\n");
        let levels = levels(&ktx2).unwrap();
        assert_eq!(levels.len(), 7);
        assert_eq!((levels[0].0, levels[0].1), (64, 64));
        assert_eq!((levels[6].0, levels[6].1), (1, 1));
        assert!(psnr(&levels[0].2, img.as_raw()) > 24.0);
    }
}

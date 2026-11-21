//! Pictures of a model for its author to look at after a build (`cargo art --preview`, `cargo check-glb
//! --preview`), drawn by `raster` from the .glb as the game reads it (`glb::read`): what
//! tools/art/blender/gamekit/preview.py rendered in Blender's Workbench. The front three-quarter, each clip where
//! it is most open, the far version, the perches as red balls, and a sheet of four sides, the baked shading
//! alone and each clip along its length; the views, framing and sizes are those renders'. Each finish is its
//! flat colour times its baked shading under a sun and a sky, a cut-out one (leaf cards) on its texture; the
//! lit, textured view is the game's own (`bun tools/bench/check.mjs gallery`, the inspector).

use crate::glb::{self, Model, Pose};
use crate::raster::{
    self, Camera, Floor, Look, Picture, Projection, Shading, Surface, Texture, Vec3,
};
use crate::scene::FPS;
use std::collections::BTreeMap;

/// A picture's side (px).
pub const SIZE: usize = 640;
/// A sheet's tile's side (px).
pub const SHEET_TILE: usize = 320;
/// Where along each clip a sheet shows it.
pub const SHEET_ALONG: [f64; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];
/// White between a sheet's tiles (px).
const SHEET_GAP: usize = 4;

/// Where a picture looks from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    /// The front three-quarter, from the right.
    ThreeQuarter,
    /// The back three-quarter, from the left: the game turns things all four ways.
    BackThreeQuarter,
    /// From its left (a bird's left, its wing.L).
    Side,
    Front,
    /// Straight down, its front at the bottom of the picture.
    Top,
}

/// A sheet's first row.
pub const SHEET_VIEWS: [View; 4] = [
    View::ThreeQuarter,
    View::BackThreeQuarter,
    View::Side,
    View::Top,
];

impl View {
    /// Where the camera stands: (x, y, z) in the file's axes (+Y up, the front +Z) times the model's largest
    /// side, from its middle (Blender's (x, -z, y)).
    fn from(self) -> Vec3 {
        match self {
            View::ThreeQuarter => [0.95, 0.75, 1.05],
            View::BackThreeQuarter => [-0.95, 0.75, -1.05],
            View::Side => [1.6, 0.0, 0.0],
            View::Front => [0.0, 0.0, 1.6],
            View::Top => [0.0, 1.6, 0.0001],
        }
    }

    /// How much wider than the model's largest side the picture is.
    fn zoom(self) -> f32 {
        match self {
            View::ThreeQuarter | View::BackThreeQuarter => 1.55,
            _ => 1.15,
        }
    }
}

/// What a model's materials are drawn with beyond their colour, by material name: a cut-out finish's texture (a
/// tree's leaf cards on their atlas) and the alpha it is cut at; a textured finish's (wood, fabric, bark) tint, its
/// texture's average colour, which the game multiplies the colour by.
#[derive(Clone, Debug, Default)]
pub struct Maps {
    pub textures: Vec<Texture>,
    pub cutouts: BTreeMap<String, (usize, f32)>,
    /// Linear RGB.
    pub tints: BTreeMap<String, Vec3>,
}

/// The background, as Blender's Workbench had it: a dark grey.
const BACKGROUND: [u8; 3] = [61, 61, 61];
/// The floor, a little lighter.
const FLOOR: [u8; 3] = [78, 78, 76];
/// The perches' balls (perch.py's `perch_marker`).
const PERCH_RED: [u8; 3] = [0xFF, 0x20, 0x20];

/// `model`'s triangles in `pose`, placed: a mesh a primitive, each its material's colour (tinted, or a cut-out one
/// on its texture) and its baked shading.
pub fn meshes(model: &Model, pose: Pose, maps: &Maps) -> Vec<raster::Mesh> {
    let world = model.world(pose);
    let mut out = Vec::new();
    for n in model.drawn() {
        let Some(mesh) = model.nodes[n].mesh.and_then(|m| model.meshes.get(m)) else {
            continue;
        };
        let m = &world[n];
        for p in &mesh.primitives {
            let material = p.material.and_then(|i| model.materials.get(i));
            let cut = material.and_then(|mat| maps.cutouts.get(&mat.name));
            let base = material.map(|mat| mat.base).unwrap_or([1.0; 4]);
            let tint = material
                .and_then(|mat| maps.tints.get(&mat.name))
                .copied()
                .unwrap_or([1.0; 3]);
            out.push(raster::Mesh {
                positions: p.positions.iter().map(|&v| glb::apply(m, v)).collect(),
                normals: p.normals.iter().map(|&v| glb::apply_normal(m, v)).collect(),
                uvs: if cut.is_some() {
                    p.uvs.clone().unwrap_or_default()
                } else {
                    Vec::new()
                },
                shading: p
                    .colours
                    .as_ref()
                    .map(|c| c.iter().map(|c| c[0]).collect())
                    .unwrap_or_default(),
                indices: p.indices.clone(),
                surface: Surface {
                    colour: [base[0] * tint[0], base[1] * tint[1], base[2] * tint[2]],
                    texture: cut.map(|c| c.0),
                    cutout: cut.map(|c| c.1).or(material.and_then(|mat| mat.cutoff)),
                    two_sided: cut.is_some() || material.is_some_and(|mat| mat.double_sided),
                },
            });
        }
    }
    out
}

/// The box (lo, hi) round `meshes`' points.
pub fn bounds(meshes: &[raster::Mesh]) -> Option<(Vec3, Vec3)> {
    let mut points = meshes.iter().flat_map(|m| m.positions.iter());
    let first = *points.next()?;
    Some(points.fold((first, first), |(lo, hi), p| {
        (
            std::array::from_fn(|k| lo[k].min(p[k])),
            std::array::from_fn(|k| hi[k].max(p[k])),
        )
    }))
}

/// `meshes` (with `textures`) from `view`, framed on them as Blender framed its renders (orthographic, about the
/// middle of their box, the largest side times the view's zoom across), `size` pixels square: lit, on a floor,
/// or the baked shading alone.
pub fn picture(
    meshes: &[raster::Mesh],
    textures: &[Texture],
    view: View,
    size: usize,
    baked: bool,
) -> Picture {
    let background = raster::linear(BACKGROUND);
    let background = [background[0], background[1], background[2], 1.0];
    let Some((lo, hi)) = bounds(meshes) else {
        let [r, g, b] = BACKGROUND;
        return Picture::filled(size, size, [r, g, b, 255]);
    };
    let span = (0..3).map(|k| hi[k] - lo[k]).fold(0.0, f32::max).max(1e-3);
    let middle: Vec3 = std::array::from_fn(|k| (lo[k] + hi[k]) / 2.0);
    let from = view.from();
    let eye: Vec3 = std::array::from_fn(|k| middle[k] + from[k] * span);
    let up = if view == View::Top {
        [0.0, 0.0, -1.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let camera = Camera {
        eye,
        target: middle,
        up,
        projection: Projection::Orthographic {
            span: span * view.zoom(),
        },
        // (about 1.6 spans off the middle: clipped by the span, so a coin or an apple isn't cut away)
        near: span * 0.05,
        far: span * 12.0,
    };
    let look = if baked {
        Look {
            background,
            shading: Shading::Baked,
            floor: None,
            samples: 4,
        }
    } else {
        // the sun over the viewer's left shoulder, as Workbench's studio light falls, so the sides turned to the
        // camera differ
        let f: Vec3 = raster::unit(std::array::from_fn(|k| middle[k] - eye[k]));
        let right = raster::unit([
            f[1] * up[2] - f[2] * up[1],
            f[2] * up[0] - f[0] * up[2],
            f[0] * up[1] - f[1] * up[0],
        ]);
        let sun = raster::unit(std::array::from_fn(|k| {
            -0.45 * right[k] + 0.8 * up[k] - 0.55 * f[k]
        }));
        Look {
            background,
            shading: Shading::Lit {
                sun,
                sun_strength: 0.62,
                sky: 0.62,
                ground: 0.3,
            },
            floor: Some(Floor {
                height: lo[1],
                centre: [middle[0], middle[2]],
                radius: span * 1.1,
                colour: raster::linear(FLOOR),
                contact: 0.55,
                reach: span * 0.06,
            }),
            samples: 4,
        }
    };
    raster::render(meshes, textures, &camera, &look, size, size)
}

/// The frame a clip ends on (its first is 1): its last key's, `FPS` a second.
pub fn last_frame(model: &Model, clip: usize) -> i32 {
    (model.animations[clip].length() as f64 * FPS).round() as i32 + 1
}

/// The time (s) of a frame.
fn time(frame: i32) -> f32 {
    ((frame - 1) as f64 / FPS) as f32
}

/// How far each part has moved from its rest in `pose`: the sum over the parts of the furthest any corner of
/// its box went.
fn moved(model: &Model, pose: Pose) -> f32 {
    let (rest, posed) = (model.world(None), model.world(pose));
    let mut sum = 0.0;
    for n in model.drawn() {
        let Some(mesh) = model.nodes[n].mesh.and_then(|m| model.meshes.get(m)) else {
            continue;
        };
        let mut points = mesh.primitives.iter().flat_map(|p| p.positions.iter());
        let Some(&first) = points.next() else {
            continue;
        };
        let (lo, hi) = points.fold((first, first), |(lo, hi), p| {
            (
                std::array::from_fn::<f32, 3, _>(|k| lo[k].min(p[k])),
                std::array::from_fn::<f32, 3, _>(|k| hi[k].max(p[k])),
            )
        });
        let mut most: f32 = 0.0;
        for c in 0..8 {
            let corner = [
                if c & 1 == 0 { lo[0] } else { hi[0] },
                if c & 2 == 0 { lo[1] } else { hi[1] },
                if c & 4 == 0 { lo[2] } else { hi[2] },
            ];
            let (a, b) = (glb::apply(&rest[n], corner), glb::apply(&posed[n], corner));
            let d: f32 = (0..3).map(|k| (a[k] - b[k]) * (a[k] - b[k])).sum();
            most = most.max(d.sqrt());
        }
        sum += most;
    }
    sum
}

/// The time (s) a picture shows `clip` at: where it is most open. Its last frame, or for a clip that comes home
/// (a spin, a rock, a squash and back) the frame furthest from rest; Blender's builds marked that frame
/// (motion.py's `peak`), which a file doesn't keep.
pub fn peak(model: &Model, clip: usize) -> f32 {
    let end = last_frame(model, clip);
    let far: Vec<f32> = (1..=end)
        .map(|f| moved(model, Some((clip, time(f)))))
        .collect();
    let most = far.iter().copied().fold(0.0, f32::max);
    let home = far[far.len() - 1] <= most * 0.01;
    if !home {
        return time(end);
    }
    let at = far.iter().position(|&d| d == most).unwrap_or(0);
    time(at as i32 + 1)
}

/// A red ball 12 cm across on each perch (the pack's game axes: x, height, depth; the file's x the other way),
/// where a small bird would sit on it: 5 cm up.
pub fn perch_markers(perches: &[[f64; 3]]) -> Vec<raster::Mesh> {
    let (segments, rings, r) = (8usize, 5usize, 0.06f32);
    perches
        .iter()
        .map(|p| {
            let at = [-p[0] as f32, p[1] as f32 + 0.05, p[2] as f32];
            let point = |ring: usize, seg: usize| {
                let lat = std::f32::consts::PI * ring as f32 / rings as f32;
                let lon = std::f32::consts::TAU * seg as f32 / segments as f32;
                let (sl, cl) = (libm::sinf(lat), libm::cosf(lat));
                [
                    at[0] + r * sl * libm::cosf(lon),
                    at[1] + r * cl,
                    at[2] - r * sl * libm::sinf(lon),
                ]
            };
            let mut positions = Vec::new();
            let mut indices = Vec::new();
            for ring in 0..rings {
                for seg in 0..segments {
                    let q = [
                        point(ring, seg),
                        point(ring + 1, seg),
                        point(ring + 1, seg + 1),
                        point(ring, seg + 1),
                    ];
                    let base = positions.len() as u32;
                    positions.extend(q);
                    indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
                }
            }
            raster::Mesh {
                positions,
                indices,
                surface: Surface {
                    colour: raster::linear(PERCH_RED),
                    ..Surface::default()
                },
                ..raster::Mesh::default()
            }
        })
        .collect()
}

/// Pictures in rows, side by side and row under row, `gap` white pixels between them (tiles of one size).
pub fn compose(rows: &[Vec<Picture>], gap: usize) -> Picture {
    let (w, h) = rows
        .first()
        .and_then(|r| r.first())
        .map(|p| (p.width, p.height))
        .unwrap_or((0, 0));
    let cols = rows.iter().map(Vec::len).max().unwrap_or(0);
    let width = (cols * w + cols.saturating_sub(1) * gap).max(1);
    let height = (rows.len() * h + rows.len().saturating_sub(1) * gap).max(1);
    let mut out = Picture::filled(width, height, [255; 4]);
    for (r, row) in rows.iter().enumerate() {
        for (c, tile) in row.iter().enumerate() {
            out.paste(tile, c * (w + gap), r * (h + gap));
        }
    }
    out
}

/// The sheet: tiles `SHEET_TILE` square, the first row its four sides (`SHEET_VIEWS`) and its baked shading,
/// then a row per clip from the front three-quarter at 0, 25, 50, 75 and 100 % of the way through it (a door
/// through the cabinet half way, a lid that closes past its rest), each clip on its own.
pub fn sheet(model: &Model, maps: &Maps) -> Picture {
    let rest = meshes(model, None, maps);
    let mut first: Vec<Picture> = SHEET_VIEWS
        .iter()
        .map(|&v| picture(&rest, &maps.textures, v, SHEET_TILE, false))
        .collect();
    first.push(picture(
        &rest,
        &maps.textures,
        View::ThreeQuarter,
        SHEET_TILE,
        true,
    ));
    let mut rows = vec![first];
    for clip in 0..model.animations.len() {
        let end = last_frame(model, clip);
        rows.push(
            SHEET_ALONG
                .iter()
                .map(|&along| {
                    let frame = (1.0 + along * (end - 1) as f64).round_ties_even() as i32;
                    let posed = meshes(model, Some((clip, time(frame))), maps);
                    picture(
                        &posed,
                        &maps.textures,
                        View::ThreeQuarter,
                        SHEET_TILE,
                        false,
                    )
                })
                .collect(),
        );
    }
    compose(&rows, SHEET_GAP)
}

/// A model's pictures, as `cargo art --preview` names them after its id: ("", the front three-quarter),
/// (".sheet", the sheet) when asked, ("@<clip>", each clip where it is most open), ("@perches" and
/// "@perches.top", its perches from the front three-quarter and from above) where it has any, (".far", its
/// far version) where it has one.
pub fn previews(
    model: &Model,
    perches: &[[f64; 3]],
    far: Option<&Model>,
    with_sheet: bool,
    maps: &Maps,
) -> Vec<(String, Picture)> {
    let tex = &maps.textures;
    let rest = meshes(model, None, maps);
    let mut out = vec![(
        String::new(),
        picture(&rest, tex, View::ThreeQuarter, SIZE, false),
    )];
    if with_sheet {
        out.push((".sheet".into(), sheet(model, maps)));
    }
    for (clip, anim) in model.animations.iter().enumerate() {
        let posed = meshes(model, Some((clip, peak(model, clip))), maps);
        out.push((
            format!("@{}", anim.name),
            picture(&posed, tex, View::ThreeQuarter, SIZE, false),
        ));
    }
    if !perches.is_empty() {
        let mut marked = rest.clone();
        marked.extend(perch_markers(perches));
        out.push((
            "@perches".into(),
            picture(&marked, tex, View::ThreeQuarter, SIZE, false),
        ));
        out.push((
            "@perches.top".into(),
            picture(&marked, tex, View::Top, SIZE, false),
        ));
    }
    if let Some(far) = far {
        let far = meshes(far, None, maps);
        out.push((
            ".far".into(),
            picture(&far, tex, View::ThreeQuarter, SIZE, false),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glb::{Animation, Channel, Interpolation, Mesh, Node, Primitive, Target};

    /// A 10 cm box on a part, turned about its upright by `angle` (radians) at each frame's time.
    fn turning(angle: impl Fn(f32) -> f32, frames: usize) -> Model {
        let corners: Vec<[f32; 3]> = (0..8)
            .map(|c| {
                [
                    if c & 1 == 0 { 0.0 } else { 0.1 },
                    if c & 2 == 0 { 0.0 } else { 0.1 },
                    if c & 4 == 0 { 0.0 } else { 0.1 },
                ]
            })
            .collect();
        let times: Vec<f32> = (0..frames).map(|f| (f as f64 / FPS) as f32).collect();
        let values: Vec<f32> = times
            .iter()
            .flat_map(|&t| {
                let h = angle(t) / 2.0;
                [0.0, libm::sinf(h), 0.0, libm::cosf(h)]
            })
            .collect();
        Model {
            roots: vec![0],
            nodes: vec![Node {
                name: "lid".into(),
                mesh: Some(0),
                ..Node::default()
            }],
            meshes: vec![Mesh {
                name: "lid".into(),
                primitives: vec![Primitive {
                    positions: corners,
                    indices: vec![0, 1, 2],
                    ..Primitive::default()
                }],
            }],
            animations: vec![Animation {
                name: "spin".into(),
                channels: vec![Channel {
                    node: 0,
                    target: Target::Rotation,
                    interpolation: Interpolation::Linear,
                    times,
                    values,
                }],
            }],
            ..Model::default()
        }
    }

    #[test]
    fn a_clip_is_shown_at_its_end_or_where_it_turns_furthest_from_home() {
        // a spin all the way round: half way; a door that opens and stays: its end
        let spin = turning(|t| std::f32::consts::TAU * t, 31);
        assert_eq!(last_frame(&spin, 0), 31);
        assert_eq!(peak(&spin, 0), time(16));
        let door = turning(|t| (t * 6.0).min(1.5), 21);
        assert_eq!(peak(&door, 0), time(21));
    }

    #[test]
    fn a_sheet_is_a_row_of_sides_then_a_row_a_clip() {
        let model = turning(|t| t, 7);
        let sheet = sheet(&model, &Maps::default());
        assert_eq!(
            (sheet.width, sheet.height),
            (5 * SHEET_TILE + 4 * SHEET_GAP, 2 * SHEET_TILE + SHEET_GAP)
        );
        assert_eq!(
            sheet.at(SHEET_TILE + 1, 5),
            [255; 4],
            "white between the tiles"
        );
        let names: Vec<String> = previews(
            &model,
            &[[0.0, 0.05, 0.0]],
            Some(&model),
            true,
            &Maps::default(),
        )
        .into_iter()
        .map(|(n, p)| {
            if n != ".sheet" {
                assert_eq!((p.width, p.height), (SIZE, SIZE));
            }
            n
        })
        .collect();
        assert_eq!(
            names,
            ["", ".sheet", "@spin", "@perches", "@perches.top", ".far"]
        );
    }

    #[test]
    fn a_perch_ball_sits_5_cm_over_its_perch_in_the_files_axes() {
        let balls = perch_markers(&[[0.5, 2.0, -0.25]]);
        assert_eq!(balls.len(), 1);
        let (lo, hi) = bounds(&balls).unwrap();
        let middle: Vec<f32> = (0..3).map(|k| (lo[k] + hi[k]) / 2.0).collect();
        assert!(
            (middle[0] + 0.5).abs() < 1e-5
                && (middle[1] - 2.05).abs() < 1e-5
                && (middle[2] + 0.25).abs() < 1e-5,
            "{middle:?}"
        );
        assert!((hi[1] - lo[1] - 0.12).abs() < 1e-5);
    }
}

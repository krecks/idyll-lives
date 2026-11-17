//! The gate's check `texels` and `cargo review texels`: how a model's UVs lay a textured finish over it.
//!
//! A finish's texture tiles over a part by the part's UVs times the finish's `uvScale`, so a triangle shows
//! `sqrt(UV area / area) × uvScale × the texture's side` pixels a metre. The play camera shows 250 px a metre at its
//! nearest: a texture much sparser than that is a blur, one much denser shimmers and wastes its download. Where the
//! UVs lay it further one way than the other (the ratio of the mapping's two singular values) its grain is
//! stretched, and where they have no area at all one colour of it is smeared over the triangle.
//!
//! A finding is a material on a textured finish (`Textures`: the leaf cards, which pick pictures from an atlas, are
//! not one) whose
//!
//! - density, the one half its area is under, is outside 200 to 1600 px a metre (`sparse`, `dense`);
//! - mapping is stretched over 2× on more than 5 % of its area (`stretched`);
//! - UVs have no area on more than 1 % of its area (`smeared`).
//!
//! ```text
//! cargo review texels [--all] [--check] [model keys…]
//! ```

use crate::edges::Mesh;
use std::collections::BTreeMap;

/// Pixels a metre a texture should have at least and at most (the play camera shows 250).
pub const DENSITY: [f64; 2] = [200.0, 1600.0];
/// A mapping laid further one way than the other by more than this is stretched,
pub const STRETCH: f64 = 2.0;
/// and a material may be so on this share of its area;
pub const STRETCHED: f64 = 0.05;
/// its UVs may have no area on this share of it (a sliver: a cap's middle, a seam).
pub const SMEARED: f64 = 0.01;

/// A textured finish: how often its texture repeats over one unit of UV, and the texture's side in pixels (None:
/// its file wasn't read, so densities aren't known).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Texture {
    pub uv_scale: f64,
    pub px: Option<f64>,
}

/// The textured finishes whose UVs are judged, by name (`wood`).
pub type Textures = BTreeMap<String, Texture>;

/// How one material of a model wears its finish.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Material {
    pub material: String,
    pub finish: String,
    /// Its area, in m².
    pub area: f64,
    /// Pixels a metre: the density half its area is under (None: the texture's size isn't known, or all of it is
    /// smeared).
    pub density: Option<f64>,
    /// The share of its area stretched over `STRETCH`.
    pub stretched: f64,
    /// The share of its area whose UVs have no area, and how many triangles that is.
    pub smeared: f64,
    pub smeared_triangles: usize,
    /// Each triangle with UV area: (repeats of UV a metre, m²).
    pub spread: Vec<(f64, f64)>,
}

/// The value half the weight is under: `values` as (value, weight), sorted here.
pub fn median(values: &mut [(f64, f64)], share: f64) -> Option<f64> {
    values.sort_by(|a, b| a.0.total_cmp(&b.0));
    let total: f64 = values.iter().map(|v| v.1).sum();
    let mut under = 0.0;
    for (value, weight) in values.iter() {
        under += weight;
        if under >= total * share {
            return Some(*value);
        }
    }
    None
}

/// Each material on a textured finish (`textures`) over the meshes that have it, by material and finish.
pub fn measure(meshes: &[Mesh], textures: &Textures) -> Vec<Material> {
    let mut found: BTreeMap<(&str, &str), Material> = BTreeMap::new();
    for mesh in meshes {
        if !textures.contains_key(&mesh.finish) || mesh.uvs.len() != mesh.positions.len() {
            continue;
        }
        let m = found
            .entry((&mesh.material, &mesh.finish))
            .or_insert_with(|| Material {
                material: mesh.material.clone(),
                finish: mesh.finish.clone(),
                ..Material::default()
            });
        for t in &mesh.triangles {
            let corner = |k: usize| {
                let i = t[k] as usize;
                Some((*mesh.positions.get(i)?, *mesh.uvs.get(i)?))
            };
            let (Some((a, ua)), Some((b, ub)), Some((c, uc))) = (corner(0), corner(1), corner(2))
            else {
                continue;
            };
            let e1 = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let dot = |p: [f64; 3], q: [f64; 3]| p[0] * q[0] + p[1] * q[1] + p[2] * q[2];
            // The triangle laid flat: e1 along x, its length l; e2 at (p, h).
            let l = dot(e1, e1).sqrt();
            if l < 1e-9 {
                continue;
            }
            let p = dot(e1, e2) / l;
            let h = (dot(e2, e2) - p * p).max(0.0).sqrt();
            let area = l * h / 2.0;
            if area < 1e-10 {
                continue;
            }
            // The mapping from there to UV, a 2 × 2 matrix by columns, and its two singular values.
            let (d1, d2) = (
                [ub[0] - ua[0], ub[1] - ua[1]],
                [uc[0] - ua[0], uc[1] - ua[1]],
            );
            let x = [d1[0] / l, d1[1] / l];
            let y = [(d2[0] - p * x[0]) / h, (d2[1] - p * x[1]) / h];
            let sum = x[0] * x[0] + x[1] * x[1] + y[0] * y[0] + y[1] * y[1];
            let det = x[0] * y[1] - x[1] * y[0];
            let root = (sum * sum - 4.0 * det * det).max(0.0).sqrt();
            let (most, least) = (
                ((sum + root) / 2.0).sqrt(),
                ((sum - root) / 2.0).max(0.0).sqrt(),
            );
            m.area += area;
            if most < 1e-3 || least < 1e-3 * most {
                m.smeared += area;
                m.smeared_triangles += 1;
                continue;
            }
            if most > STRETCH * least {
                m.stretched += area;
            }
            m.spread.push(((most * least).sqrt(), area));
        }
    }
    found
        .into_values()
        .filter(|m| m.area > 0.0)
        .map(|mut m| {
            let texture = textures[&m.finish];
            m.density = texture
                .px
                .and_then(|px| Some(median(&mut m.spread, 0.5)? * texture.uv_scale * px));
            m.stretched /= m.area;
            m.smeared /= m.area;
            m
        })
        .collect()
}

/// A share as per cent, to a tenth under 10 %.
fn percent(share: f64) -> String {
    if share < 0.0995 && share > 0.0 {
        format!("{:.1} %", share * 100.0)
    } else {
        format!("{:.0} %", share * 100.0)
    }
}

/// What is wrong with how a material wears its finish (the module's rule): a line each.
pub fn findings(m: &Material) -> Vec<String> {
    let mut out = Vec::new();
    let name = format!("material {} ({})", m.material, m.finish);
    match m.density {
        Some(d) if d < DENSITY[0] => out.push(format!(
            "{name}: sparse, {d:.0} px a metre, under {} (the play camera shows 250)",
            DENSITY[0]
        )),
        Some(d) if d > DENSITY[1] => out.push(format!(
            "{name}: dense, {d:.0} px a metre, over {} (the play camera shows 250)",
            DENSITY[1]
        )),
        _ => {}
    }
    if m.stretched > STRETCHED {
        out.push(format!(
            "{name}: stretched over {STRETCH}× on {} of its area (at most {})",
            percent(m.stretched),
            percent(STRETCHED)
        ));
    }
    if m.smeared > SMEARED {
        out.push(format!(
            "{name}: smeared, its UVs have no area on {} of its area ({} triangles; at most {})",
            percent(m.smeared),
            m.smeared_triangles,
            percent(SMEARED)
        ));
    }
    out
}

/// One model's materials on textured finishes.
#[derive(Debug, Clone, Default)]
pub struct Found {
    pub key: String,
    pub family: String,
    pub materials: Vec<Material>,
}

impl Found {
    fn findings(&self) -> Vec<String> {
        self.materials.iter().flat_map(findings).collect()
    }
}

/// The report's text: each finish as the models wear it, then the models with findings (25, or `all`).
pub fn report(found: &[Found], textures: &Textures, all: bool) -> String {
    use std::fmt::Write as _;
    let mut text = String::new();
    let with: Vec<(&Found, Vec<String>)> = found
        .iter()
        .map(|f| (f, f.findings()))
        .filter(|(_, lines)| !lines.is_empty())
        .collect();
    let _ = writeln!(
        text,
        "{} models, {} with a textured finish laid outside {} to {} px a metre, stretched over {STRETCH}× on more than {} of a material's area, or smeared over more than {}: {} findings\n",
        found.len(),
        with.len(),
        DENSITY[0],
        DENSITY[1],
        percent(STRETCHED),
        percent(SMEARED),
        with.iter().map(|(_, lines)| lines.len()).sum::<usize>(),
    );
    let _ = writeln!(
        text,
        "| finish | texture px | uvScale | models | materials | m² | px a metre (median) | 5th to 95th | stretched | smeared | findings |\n|---|---|---|---|---|---|---|---|---|---|---|"
    );
    for (finish, texture) in textures {
        let materials: Vec<&Material> = found
            .iter()
            .flat_map(|f| &f.materials)
            .filter(|m| &m.finish == finish)
            .collect();
        let models = found
            .iter()
            .filter(|f| f.materials.iter().any(|m| &m.finish == finish))
            .count();
        let area: f64 = materials.iter().map(|m| m.area).sum();
        let share = |of: &dyn Fn(&Material) -> f64| {
            if area > 0.0 {
                percent(materials.iter().map(|m| of(m) * m.area).sum::<f64>() / area)
            } else {
                "—".into()
            }
        };
        let mut spread: Vec<(f64, f64)> = materials.iter().flat_map(|m| m.spread.clone()).collect();
        let px = |at: f64, spread: &mut [(f64, f64)]| match (texture.px, median(spread, at)) {
            (Some(px), Some(v)) => format!("{:.0}", v * texture.uv_scale * px),
            _ => "—".into(),
        };
        let _ = writeln!(
            text,
            "| {finish} | {} | {} | {models} | {} | {} | {} | {} to {} | {} | {} ({} triangles) | {} |",
            texture.px.map_or("unread".into(), |px| format!("{px:.0}")),
            texture.uv_scale,
            materials.len(),
            if area < 10.0 {
                format!("{area:.1}")
            } else {
                format!("{area:.0}")
            },
            px(0.5, &mut spread),
            px(0.05, &mut spread),
            px(0.95, &mut spread),
            share(&|m| m.stretched),
            share(&|m| m.smeared),
            materials.iter().map(|m| m.smeared_triangles).sum::<usize>(),
            materials.iter().map(|m| findings(m).len()).sum::<usize>(),
        );
    }
    let _ = writeln!(
        text,
        "\nThe models with findings{}:\n\n| model | family | findings |\n|---|---|---|",
        if all {
            ""
        } else {
            " (25; `--all` for every one)"
        }
    );
    for (f, lines) in with.iter().take(if all { usize::MAX } else { 25 }) {
        let _ = writeln!(text, "| {} | {} | {} |", f.key, f.family, lines.join("; "));
    }
    text
}

/// Every model's materials on textured finishes, or those of `keys`, and those finishes.
pub fn survey(
    paths: &crate::checkglb::Paths,
    keys: &[String],
) -> Result<(Vec<Found>, Textures), String> {
    let textures = crate::checkglb::meshes::textures_of(paths)?;
    let mut found = Vec::new();
    crate::checkglb::meshes::each(paths, keys, true, &mut |key, family, meshes| {
        found.push(Found {
            key: key.to_string(),
            family: family.to_string(),
            materials: measure(&meshes, &textures),
        });
    })?;
    Ok((found, textures))
}

/// `cargo review texels [--all] [--check] [keys…]`: the exit code (1 with `--check` when a model has a finding).
pub fn main(args: &[String]) -> i32 {
    let flag = |name: &str| args.iter().any(|a| a == name);
    let keys: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with("--"))
        .cloned()
        .collect();
    let paths = crate::checkglb::Paths::repo();
    let (found, textures) = match survey(&paths, &keys) {
        Ok(found) => found,
        Err(e) => {
            eprintln!("review texels: {e}");
            return 1;
        }
    };
    print!("{}", report(&found, &textures, flag("--all")));
    i32::from(flag("--check") && found.iter().any(|f| !f.findings().is_empty()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Squares of 10 cm side by side on the floor, the first `odd` of the `n` with their UVs through `odd_uv`
    /// and the rest through `uv` (each given the corner's place in metres).
    fn strip(
        n: usize,
        odd: usize,
        uv: impl Fn(f64, f64) -> [f64; 2],
        odd_uv: impl Fn(f64, f64) -> [f64; 2],
    ) -> Mesh {
        let mut mesh = Mesh {
            material: "top".into(),
            finish: "wood".into(),
            ..Mesh::default()
        };
        for i in 0..n {
            let at = mesh.positions.len() as u32;
            for (x, z) in [(0.0, 0.0), (0.1, 0.0), (0.1, 0.1), (0.0, 0.1)] {
                let x = x + 0.1 * i as f64;
                mesh.positions.push([x, 0.0, z]);
                mesh.uvs.push(if i < odd { odd_uv(x, z) } else { uv(x, z) });
            }
            mesh.triangles
                .extend([[at, at + 1, at + 2], [at, at + 2, at + 3]]);
        }
        mesh
    }

    fn wood() -> Textures {
        Textures::from([(
            "wood".to_string(),
            Texture {
                uv_scale: 1.2,
                px: Some(512.0),
            },
        )])
    }

    fn one(mesh: Mesh) -> Material {
        let mut found = measure(&[mesh], &wood());
        assert_eq!(found.len(), 1);
        found.remove(0)
    }

    #[test]
    fn uvs_in_metres_show_the_texture_at_its_size() {
        // 512 px tiled 1.2 times a metre: 614 px a metre, whichever way the UVs are turned.
        let m = one(strip(10, 0, |x, z| [x, z], |x, z| [x, z]));
        assert!((m.area - 0.1).abs() < 1e-12);
        assert!((m.density.unwrap() - 614.4).abs() < 1e-6, "{:?}", m.density);
        assert_eq!((m.stretched, m.smeared, m.smeared_triangles), (0.0, 0.0, 0));
        assert_eq!(findings(&m), Vec::<String>::new());
        let turned = one(strip(
            10,
            0,
            |x, z| [0.6 * x - 0.8 * z + 3.0, 0.8 * x + 0.6 * z],
            |x, z| [x, z],
        ));
        assert!((turned.density.unwrap() - 614.4).abs() < 1e-6);
        assert_eq!(turned.stretched, 0.0);
    }

    #[test]
    fn a_texture_laid_too_large_or_too_small_is_found() {
        // A quarter of its size: 154 px a metre; four times: 2458.
        let sparse = one(strip(10, 0, |x, z| [x / 4.0, z / 4.0], |x, z| [x, z]));
        assert_eq!(
            findings(&sparse),
            ["material top (wood): sparse, 154 px a metre, under 200 (the play camera shows 250)"]
        );
        let dense = one(strip(10, 0, |x, z| [x * 4.0, z * 4.0], |x, z| [x, z]));
        assert_eq!(
            findings(&dense),
            ["material top (wood): dense, 2458 px a metre, over 1600 (the play camera shows 250)"]
        );
        // Half of its area is what counts: three squares of ten laid sparse don't make it sparse.
        let some = one(strip(10, 3, |x, z| [x, z], |x, z| [x / 4.0, z / 4.0]));
        assert!((some.density.unwrap() - 614.4).abs() < 1e-6);
        assert_eq!(findings(&some), Vec::<String>::new());
    }

    #[test]
    fn a_stretched_or_smeared_texture_is_found() {
        // Laid three times as far one way on 1 square of 10: 10 % of the area; on 1 of 25: 4 %, which passes.
        let stretched = one(strip(10, 1, |x, z| [x, z], |x, z| [x / 3.0, z]));
        assert!((stretched.stretched - 0.1).abs() < 1e-12);
        assert_eq!(
            findings(&stretched),
            ["material top (wood): stretched over 2× on 10 % of its area (at most 5.0 %)"]
        );
        assert_eq!(
            findings(&one(strip(25, 1, |x, z| [x, z], |x, z| [x / 3.0, z]))),
            Vec::<String>::new()
        );
        // Twice as far is not yet stretched.
        assert_eq!(
            one(strip(10, 10, |x, z| [x, z], |x, z| [x / 1.99, z])).stretched,
            0.0
        );
        // One UV for a whole square, or UVs along one line: no area. 1 square of 50 is 2 %.
        let smeared = one(strip(50, 1, |x, z| [x, z], |_, _| [0.3, 0.3]));
        assert_eq!(
            findings(&smeared),
            [
                "material top (wood): smeared, its UVs have no area on 2.0 % of its area (2 triangles; at most 1.0 %)"
            ]
        );
        assert!((smeared.density.unwrap() - 614.4).abs() < 1e-6);
        let line = one(strip(50, 1, |x, z| [x, z], |x, _| [x, x]));
        assert_eq!(line.smeared_triangles, 2);
        // All of it smeared: no density to give.
        let all = one(strip(4, 4, |x, z| [x, z], |_, _| [0.0, 0.0]));
        assert_eq!((all.density, all.smeared), (None, 1.0));
    }

    #[test]
    fn only_textured_finishes_with_uvs_are_measured() {
        let mesh = strip(4, 0, |x, z| [x / 4.0, z / 4.0], |x, z| [x, z]);
        // A finish that isn't one of the textured ones (plain, or the leaf cards' atlas).
        let plain = Mesh {
            finish: "satin".into(),
            ..mesh.clone()
        };
        assert_eq!(measure(&[plain], &wood()), []);
        // No UVs: the game lays the texture itself.
        let bare = Mesh {
            uvs: Vec::new(),
            ..mesh.clone()
        };
        assert_eq!(measure(&[bare], &wood()), []);
        // A texture whose size isn't known: stretch is still judged, density isn't.
        let unread = Textures::from([(
            "wood".to_string(),
            Texture {
                uv_scale: 1.2,
                px: None,
            },
        )]);
        let found = measure(std::slice::from_ref(&mesh), &unread);
        assert_eq!((found[0].density, findings(&found[0]).len()), (None, 0));
        // Two parts of one material are one material; another material is apart.
        let other = Mesh {
            material: "legs".into(),
            ..mesh.clone()
        };
        let found = measure(&[mesh.clone(), mesh, other], &wood());
        assert_eq!(found.len(), 2);
        assert_eq!(
            (found[0].material.as_str(), found[1].material.as_str()),
            ("legs", "top")
        );
        assert!((found[1].area - 0.08).abs() < 1e-12);
    }

    #[test]
    fn the_report_gives_each_finish_and_the_models_with_findings() {
        let found = [
            Found {
                key: "model.table".into(),
                family: "dining".into(),
                materials: measure(&[strip(10, 2, |x, z| [x, z], |x, z| [x / 3.0, z])], &wood()),
            },
            Found {
                key: "model.stool".into(),
                family: "dining".into(),
                materials: measure(&[strip(10, 0, |x, z| [x, z], |x, z| [x, z])], &wood()),
            },
        ];
        let text = report(&found, &wood(), false);
        assert!(
            text.starts_with("2 models, 1 with a textured finish laid outside 200 to 1600 px a metre, stretched over 2× on more than 5.0 % of a material's area, or smeared over more than 1.0 %: 1 findings\n"),
            "{text}"
        );
        assert!(
            text.contains("| wood | 512 | 1.2 | 2 | 2 | 0.2 | 614 | 355 to 614 | 10 % | 0 % (0 triangles) | 1 |"),
            "{text}"
        );
        assert!(
            text.contains("| model.table | dining | material top (wood): stretched over 2× on 20 % of its area (at most 5.0 %) |"),
            "{text}"
        );
        assert!(!text.contains("model.stool"), "{text}");
    }

    #[test]
    fn the_shipped_models_are_read() {
        let (found, textures) = survey(&crate::checkglb::Paths::repo(), &[]).unwrap();
        assert!(found.len() > 600, "{} models", found.len());
        // The textured finishes, with their textures' sizes read from the files; not the leaf cards' atlas.
        assert!(
            textures.len() >= 5 && !textures.contains_key("leafcards"),
            "{textures:?}"
        );
        assert!(
            textures
                .values()
                .all(|t| t.px.is_some_and(|px| px >= 256.0)),
            "{textures:?}"
        );
        assert_eq!(textures["wood"].uv_scale, 1.2);
        // The kit lays UVs in metres: most of the wood is at its texture's size.
        let mut wood: Vec<(f64, f64)> = found
            .iter()
            .flat_map(|f| &f.materials)
            .filter(|m| m.finish == "wood")
            .flat_map(|m| m.spread.clone())
            .collect();
        assert!((median(&mut wood, 0.5).unwrap() - 1.0).abs() < 0.01);
    }
}

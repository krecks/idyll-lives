//! `materials.py`: the PBR material and environment entries (`materials.json`), and the JSON
//! writer Python's `json.dump(..., indent=2)` matches.

use super::*;

// --- materials.py ---------------------------------------------------------------------------------

/// A JSON value written the way Python's `json.dump(..., indent=2)` writes it, keys in insertion order.
#[derive(Clone, Debug)]
pub enum J {
    Str(&'static str),
    Owned(String),
    Int(i64),
    Float(f64),
    Obj(Vec<(&'static str, J)>),
}

impl J {
    fn set(&mut self, key: &'static str, v: J) {
        let J::Obj(m) = self else {
            panic!("set on a non-object")
        };
        match m.iter_mut().find(|(k, _)| *k == key) {
            Some(slot) => slot.1 = v,
            None => m.push((key, v)),
        }
    }

    fn write(&self, out: &mut String, depth: usize) {
        match self {
            J::Str(s) => out.push_str(&crate::content::json_str(s)),
            J::Owned(s) => out.push_str(&crate::content::json_str(s)),
            J::Int(i) => {
                let _ = write!(out, "{i}");
            }
            J::Float(f) => out.push_str(&py_float(*f)),
            J::Obj(m) if m.is_empty() => out.push_str("{}"),
            J::Obj(m) => {
                out.push('{');
                for (i, (k, v)) in m.iter().enumerate() {
                    out.push_str(if i == 0 { "\n" } else { ",\n" });
                    out.push_str(&"  ".repeat(depth + 1));
                    out.push_str(&crate::content::json_str(k));
                    out.push_str(": ");
                    v.write(out, depth + 1);
                }
                out.push('\n');
                out.push_str(&"  ".repeat(depth));
                out.push('}');
            }
        }
    }
}

/// Python's `repr(float)` for the plain values used here (shortest round-trip, always with a point).
pub fn py_float(f: f64) -> String {
    let s = format!("{f}");
    if s.contains(['.', 'e', 'i', 'N']) {
        s
    } else {
        format!("{s}.0")
    }
}

fn surf(color: &'static str, tex: &str, uv: f64, kw: &[(&'static str, J)]) -> J {
    let mut e = vec![
        ("type", J::Str("material")),
        ("color", J::Str(color)),
        ("texture", J::Owned(format!("textures/{tex}/albedo.jpg"))),
        ("normal", J::Str("normal.jpg")),
        ("arm", J::Str("arm.jpg")),
        ("uvScale", J::Float(uv)),
    ];
    e.extend(kw.iter().cloned());
    J::Obj(e)
}

/// A cut-out finish: one albedo on the mesh's own UVs, its alpha cutting the surface out where it is under
/// `cutout` (the trees' leaf cards; the atlas is a committed file, not synthesised here).
fn cut(color: &'static str, tex: &str, cutout: f64, kw: &[(&'static str, J)]) -> J {
    let mut e = vec![
        ("type", J::Str("material")),
        ("color", J::Str(color)),
        ("texture", J::Owned(format!("textures/{tex}/albedo.png"))),
        ("uvScale", J::Float(1.0)),
        ("cutout", J::Float(cutout)),
    ];
    e.extend(kw.iter().cloned());
    J::Obj(e)
}

fn plain(color: &'static str, kw: &[(&'static str, J)]) -> J {
    let mut e = vec![("type", J::Str("material")), ("color", J::Str(color))];
    e.extend(kw.iter().cloned());
    J::Obj(e)
}

/// A sheen's colour (sRGB, as the manifests' colours) and roughness: woven cloth's is faint, a pile's
/// (velvet) bright. Both wide: from the play camera a narrower one (0.4) is a dotted line along every seam.
const FABRIC_SHEEN: (&str, f64) = ("#595959", 0.5);
const VELVET_SHEEN: (&str, f64) = ("#A6A6A6", 0.55);

/// materials.py's entries, in its order, and the finishes added since.
pub fn materials() -> J {
    let f = J::Float;
    let i = J::Int;
    let s = J::Str;
    let with = |mut e: J, k: &'static str, v: J| {
        e.set(k, v);
        e
    };
    let mut e = J::Obj(vec![
        (
            "environment.sky",
            J::Obj(vec![
                ("type", s("image")),
                ("url", s("env/belfast_open_field.pcenv")),
            ]),
        ),
        (
            "material.wall",
            surf(
                "#F4F0E8",
                "wall_plaster",
                0.5,
                &[("normalStrength", f(0.7)), ("metallic", i(0))],
            ),
        ),
        (
            "material.floor",
            surf("#FFFFFF", "floor_laminate", 0.55, &[("metallic", i(0))]),
        ),
        (
            "material.ground",
            with(
                surf(
                    "#FFFFFF",
                    "grass",
                    0.3,
                    &[
                        ("normalStrength", f(0.8)),
                        ("projection", s("xz")),
                        ("metallic", i(0)),
                    ],
                ),
                "sideColor",
                s("#C9B391"),
            ),
        ),
        (
            "material.road",
            surf(
                "#FFFFFF",
                "road_asphalt",
                0.2,
                &[("projection", s("xz")), ("metallic", i(0))],
            ),
        ),
        (
            "material.path",
            surf(
                "#FFFFFF",
                "path_hexpaving",
                0.7,
                &[("projection", s("xz")), ("metallic", i(0))],
            ),
        ),
        // A house's own front path and drive (`game/town.ts` `pathLooks`): concrete (its joints the path's own,
        // `render/shared/paving.ts`), flagstones, gravel and brick pavers in herringbone (the floor's set).
        (
            "material.path.concrete",
            surf(
                "#FFFFFF",
                "path_concrete",
                0.5,
                &[("projection", s("xz")), ("metallic", i(0))],
            ),
        ),
        (
            "material.path.flagstone",
            surf(
                "#FFFFFF",
                "path_flagstone",
                0.4167,
                &[("projection", s("xz")), ("metallic", i(0))],
            ),
        ),
        (
            "material.path.gravel",
            surf(
                "#FFFFFF",
                "path_gravel",
                0.8333,
                &[("projection", s("xz")), ("metallic", i(0))],
            ),
        ),
        (
            "material.path.brick",
            surf(
                "#E2D2C8",
                "floor_pavers",
                std::f64::consts::FRAC_1_SQRT_2,
                &[("projection", s("xz")), ("metallic", i(0))],
            ),
        ),
        // Farm fields (town.ts `farmstead`): tilled earth, the greyscale grain tinted; furrows darker.
        (
            "material.soil",
            surf(
                "#8A6646",
                "grass",
                0.5,
                &[
                    ("projection", s("xz")),
                    ("roughness", i(1)),
                    ("metallic", i(0)),
                ],
            ),
        ),
        (
            "material.furrow",
            surf(
                "#664A33",
                "grass",
                0.5,
                &[
                    ("projection", s("xz")),
                    ("roughness", i(1)),
                    ("metallic", i(0)),
                ],
            ),
        ),
        (
            "material.finish.matte",
            plain("#FFFFFF", &[("roughness", f(0.75))]),
        ),
        (
            "material.finish.satin",
            plain("#FFFFFF", &[("roughness", f(0.42))]),
        ),
        (
            "material.finish.gloss",
            plain("#FFFFFF", &[("roughness", f(0.12))]),
        ),
        (
            "material.finish.metal",
            plain("#FFFFFF", &[("roughness", f(0.32)), ("metallic", i(1))]),
        ),
        (
            "material.finish.chrome",
            plain("#FFFFFF", &[("roughness", f(0.1)), ("metallic", i(1))]),
        ),
        // What a plain thing is made of: the same shared draw as the five above (a plain finish's roughness and
        // metalness go in the vertex colour). Brass and copper are `metal` in their colour.
        (
            "material.finish.enamel",
            plain("#FFFFFF", &[("roughness", f(0.18))]),
        ),
        (
            "material.finish.ceramic",
            plain("#FFFFFF", &[("roughness", f(0.1))]),
        ),
        (
            "material.finish.plastic",
            plain("#FFFFFF", &[("roughness", f(0.4))]),
        ),
        (
            "material.finish.rubber",
            plain("#FFFFFF", &[("roughness", f(0.9))]),
        ),
        (
            "material.finish.brushed",
            plain("#FFFFFF", &[("roughness", f(0.4)), ("metallic", i(1))]),
        ),
        (
            "material.finish.iron",
            plain("#FFFFFF", &[("roughness", f(0.65)), ("metallic", i(1))]),
        ),
        // Cloth has a sheen: its fibres catch the light where the surface turns away (`KHR_materials_sheen`).
        (
            "material.finish.fabric",
            surf(
                "#FFFFFF",
                "fabric_linen",
                2.5,
                &[
                    ("roughness", i(1)),
                    ("metallic", i(0)),
                    ("sheenColor", s(FABRIC_SHEEN.0)),
                    ("sheenRoughness", f(FABRIC_SHEEN.1)),
                ],
            ),
        ),
        // Velvet is the fabric's maps under a pile's sheen: no download, a draw where a model has both.
        (
            "material.finish.velvet",
            surf(
                "#FFFFFF",
                "fabric_linen",
                2.5,
                &[
                    ("roughness", i(1)),
                    ("metallic", i(0)),
                    ("sheenColor", s(VELVET_SHEEN.0)),
                    ("sheenRoughness", f(VELVET_SHEEN.1)),
                ],
            ),
        ),
        (
            "material.finish.leather",
            surf(
                "#FFFFFF",
                "leather",
                2.0,
                &[("roughness", i(1)), ("metallic", i(0))],
            ),
        ),
        (
            "material.finish.wood",
            surf(
                "#FFFFFF",
                "wood_oak",
                1.2,
                &[("roughness", f(0.8)), ("metallic", i(0))],
            ),
        ),
        // Lacquered wood is the wood's maps under a clear coat (`KHR_materials_clearcoat`): a piano, a polished table.
        (
            "material.finish.lacquer",
            surf(
                "#FFFFFF",
                "wood_oak",
                1.2,
                &[
                    ("roughness", f(0.8)),
                    ("metallic", i(0)),
                    ("clearcoat", i(1)),
                    ("clearcoatRoughness", f(0.1)),
                ],
            ),
        ),
        (
            "material.finish.bark",
            surf("#FFFFFF", "bark", 1.0, &[("metallic", i(0))]),
        ),
        (
            "material.finish.foliage",
            surf(
                "#FFFFFF",
                "foliage",
                0.8,
                &[("roughness", f(0.9)), ("metallic", i(0))],
            ),
        ),
        (
            "material.finish.leafcards",
            cut(
                "#FFFFFF",
                "leafcards",
                0.5,
                &[("roughness", f(0.9)), ("metallic", i(0))],
            ),
        ),
        (
            "material.finish.glass",
            plain("#DCE6E8", &[("roughness", f(0.05)), ("alpha", f(0.28))]),
        ),
        (
            "material.finish.skin",
            plain("#FFFFFF", &[("roughness", f(0.55))]),
        ),
    ]);
    // House exterior/interior surfaces. uvScale = 1 / real-world tile size in metres; greyscale detail maps
    // (colour #FFFFFF) are tinted by the game per house/room; roughness multiplies the arm map's roughness.
    // None of the textured surfaces is metal: `metallic` 0 (it multiplies the arm map's blue, made 0 but
    // never quite 0 out of lossy storage, where it would read as metal).
    let house = [
        (
            "material.wall.siding",
            surf(
                "#FFFFFF",
                "siding_wood",
                0.4,
                &[
                    ("roughness", i(1)),
                    ("metallic", i(0)),
                    ("normalStrength", i(1)),
                ],
            ),
        ),
        (
            "material.wall.brick",
            surf(
                "#FFFFFF",
                "brick",
                1.0,
                &[
                    ("roughness", i(1)),
                    ("metallic", i(0)),
                    ("normalStrength", i(1)),
                ],
            ),
        ),
        (
            "material.roof",
            surf(
                "#FFFFFF",
                "roof_shingles",
                0.667,
                &[
                    ("roughness", i(1)),
                    ("metallic", i(0)),
                    ("normalStrength", i(1)),
                ],
            ),
        ),
        (
            "material.wall.stone",
            surf(
                "#FFFFFF",
                "stone_foundation",
                0.667,
                &[
                    ("roughness", i(1)),
                    ("metallic", i(0)),
                    ("normalStrength", i(1)),
                ],
            ),
        ),
        (
            "material.floor.wood",
            surf(
                "#FFFFFF",
                "floor_wood",
                0.53,
                &[
                    ("roughness", i(1)),
                    ("metallic", i(0)),
                    ("normalStrength", f(0.8)),
                ],
            ),
        ),
        (
            "material.floor.tile",
            surf(
                "#FFFFFF",
                "floor_tile",
                0.5,
                &[
                    ("roughness", f(0.7)),
                    ("metallic", i(0)),
                    ("normalStrength", i(1)),
                ],
            ),
        ),
        (
            "material.floor.carpet",
            surf(
                "#FFFFFF",
                "carpet",
                1.0,
                &[
                    ("roughness", i(1)),
                    ("metallic", i(0)),
                    ("normalStrength", f(0.8)),
                ],
            ),
        ),
        (
            "material.wall.interior",
            surf(
                "#FFFFFF",
                "wallpaper",
                0.5,
                &[
                    ("roughness", i(1)),
                    ("metallic", i(0)),
                    ("normalStrength", f(0.5)),
                ],
            ),
        ),
        (
            "material.trim",
            surf(
                "#FFFFFF",
                "painted_wood",
                1.0,
                &[
                    ("roughness", i(1)),
                    ("metallic", i(0)),
                    ("normalStrength", f(0.6)),
                ],
            ),
        ),
    ];
    for (k, v) in house {
        e.set(k, v);
    }
    // Lawn: greyscale, blotch-flattened Grass004 (1.4 m) shown at 2 m for the 10-60 m camera; vertex colours tint it.
    e.set(
        "material.ground",
        with(
            surf(
                "#FFFFFF",
                "grass_lush",
                0.5,
                &[
                    ("normalStrength", f(0.8)),
                    ("projection", s("xz")),
                    ("metallic", i(0)),
                ],
            ),
            "sideColor",
            s("#C9B391"),
        ),
    );
    // Character and house finishes without textures (glossy hair, window glass).
    e.set(
        "material.finish.hair",
        plain("#FFFFFF", &[("roughness", f(0.34))]),
    );
    e.set(
        "material.finish.skin",
        plain("#FFFFFF", &[("roughness", f(0.48))]),
    );
    e.set(
        "material.window",
        plain("#22303E", &[("roughness", f(0.05)), ("metallic", f(0.1))]),
    );
    e.set(
        "material.window.inner",
        plain("#D8E8F4", &[("roughness", f(0.25))]),
    );
    e
}

/// `json.dump(E, f, indent=2)`: no trailing newline.
pub fn materials_json() -> String {
    let mut out = String::new();
    materials().write(&mut out, 0);
    out
}

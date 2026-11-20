//! Cuts: the paint a garment takes, and the layouts its shell is built on.
//!
//! Split from `characters/clothes.rs` (see `mod.rs`).

use super::*;

/// Shells: lift off the skin (m).
pub(crate) fn jacket_lift(gid: &str) -> f64 {
    if gid == "top.jacket" { 0.009 } else { 0.008 }
}
/// The jacket shells' sleeves end (sleeve parameter) on the upper arm; the rest is painted.
pub(crate) const SHELL_SLEEVE: f64 = 0.42;
/// The skirt's rings round and down. Seven rows left the flare between the hips and the hem three
/// kinks deep, which read as facets on the closest camera (1 px is 4 mm there); twelve take the
/// kinks below a pixel and cost 240 triangles, of a 1500 shell budget. Twenty-four sides round is
/// already past what the hem's 0.24 m radius shows as corners.
pub(crate) const SKIRT_SEGS: usize = 24;
pub(crate) const SKIRT_ROWS: usize = 12;

/// A garment's cut (the keys a kind has, in rig.json's order).
#[derive(Clone, Debug, Default, Serialize)]
pub struct Cut {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sleeve: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hem: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crew: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scoop: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub over: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waist: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leg: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skirt: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shoe: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colour: Option<&'static str>,
}

pub type Cuts = OMap<Cut>;

pub fn garment_cuts(family: &str, p: &[V3; NB]) -> Cuts {
    let lm = Landmarks::new(p, family);
    let thigh = lm.hip - lm.knee;
    let shin = lm.knee - lm.ankle;
    let hem = lm.hem;
    let top = |sleeve: f64, hem: f64, crew: Option<f64>, scoop: Option<f64>, over: bool| Cut {
        sleeve: Some(sleeve),
        hem: Some(hem),
        crew,
        scoop,
        over: over.then_some(true),
        ..Cut::default()
    };
    let bottom = |waist: f64, leg: f64, skirt: bool| Cut {
        waist: Some(waist),
        leg: Some(leg),
        skirt: skirt.then_some(true),
        ..Cut::default()
    };
    let mut out = Cuts::new();
    for &gid in garments(family) {
        let (kind, name) = gid.split_once('.').unwrap();
        let c = match (kind, name) {
            // (Worn out, as they are: the hem a hand below the waistband, over the bottom. The tank top and the
            // blouse are tucked in.)
            ("top", "tee") => top(0.3, lm.hip + 0.045, Some(0.0), None, true),
            ("top", "vneck") => top(0.3, lm.hip + 0.045, None, Some(0.0), true),
            ("top", "polo") => top(0.36, lm.hip + 0.04, Some(-0.006), None, true),
            ("top", "long") => top(0.965, lm.hip + 0.045, Some(0.0), None, true),
            ("top", "shirt") => top(0.965, hem - 0.07, Some(-0.012), None, true),
            // The apron a server wears: the shirt under it is painted as a shirt (long sleeves, its hem at the hip)
            // and the apron itself is a shell over the front (clothes.rs `apron`), from the waist down to above the
            // knees. The shell's heights are its own (`APRON_WAIST`, `APRON_HEM`): the cut is the shirt's.
            ("top", "apron") => top(0.965, lm.hip + 0.02, Some(-0.012), None, true),
            ("top", "jacket") => top(0.975, lm.hip + 0.025, Some(0.0), None, true),
            ("top", "suit") => top(0.975, lm.hip + 0.015, Some(-0.012), None, true),
            ("top", "tank") => top(-0.62, hem, None, Some(0.0), false),
            ("top", "blouse") => top(0.5, hem, None, Some(0.0), false),
            ("bottom", "trousers") => bottom(lm.waist, lm.ankle + 0.03, false),
            ("bottom", "shorts") => bottom(lm.waist, lm.knee + 0.45 * thigh, false),
            ("bottom", "suit") => bottom(lm.waist + 0.01, lm.ankle + 0.02, false),
            ("bottom", "capri") => bottom(lm.waist, lm.knee - 0.45 * shin, false),
            ("bottom", "skirt") => bottom(lm.waist, lm.knee + 0.05, true),
            // Painted up to the sock's top: under the shell, and the sock in its collar's opening.
            ("shoes", n) => Cut {
                shoe: Some(lm.ankle + shoe_style(gid).sock),
                colour: Some(shoe_colour(n)),
                ..Cut::default()
            },
            _ => panic!("no garment {gid}"),
        };
        // Floats rounded to 4 places (Python's round).
        let r = |v: Option<f64>| v.map(|v| py_round(v, 4));
        let c = Cut {
            sleeve: r(c.sleeve),
            hem: r(c.hem),
            crew: r(c.crew),
            scoop: r(c.scoop),
            waist: r(c.waist),
            leg: r(c.leg),
            shoe: r(c.shoe),
            ..c
        };
        out.insert(gid, c);
    }
    out
}

/// Signed coverage (m, > 0 covered) of a top's cut over body fields (as the shader paints it).
pub fn top_coverage(f: &[f64; 4], cut: &Cut) -> f64 {
    let mut g = ((cut.sleeve.unwrap() - f[0]) * ARM_M).min(f[1] - cut.hem.unwrap());
    if let Some(c) = cut.crew {
        g = g.min(f[2] - c);
    }
    if let Some(s) = cut.scoop {
        g = g.min(f[3] - s);
    }
    g
}

pub(crate) type Fields = (Vec<[f64; 4]>, Vec<[f64; 4]>);
pub(crate) static BASE_FIELDS: Cache<String, Fields> = Cache::new();

/// The base body's fields and its LOD's.
pub(crate) fn base_fields(family: &str) -> Arc<Fields> {
    BASE_FIELDS.get(family.to_string(), || {
        let s = surface(family, BASE_STAGE);
        let h = head_frame(family, BASE_STAGE);
        let f = fields_of(family, &s.pos, &s.tags.arm, &s.region, &s.p, &h);
        let fl = fields_of(family, &s.lod_pos, &s.lod_tags.arm, &s.lod_region, &s.p, &h);
        (f, fl)
    })
}

/// Values lerped along each new vertex's source edge.
pub(crate) fn lerp_src<T: Copy + std::ops::Mul<f64, Output = T> + std::ops::Add<Output = T>>(
    src: &Src,
    arr: &[T],
) -> Vec<T> {
    let (a, bv, t) = src;
    (0..a.len())
        .map(|i| arr[a[i]] * (1.0 - t[i]) + arr[bv[i]] * t[i])
        .collect()
}

pub(crate) fn shell_dense(src: &Src, dense: &[Dense]) -> Vec<Dense> {
    let (a, bv, t) = src;
    (0..a.len())
        .map(|i| {
            let mut d = [0.0; NB];
            for k in 0..NB {
                d[k] = dense[a[i]][k] * (1.0 - t[i]) + dense[bv[i]][k] * t[i];
            }
            d
        })
        .collect()
}

/// The jacket's piece of the base body: clipped faces and their source edges (stage independent).
pub(crate) struct Layout {
    pub(crate) src: Src,
    pub(crate) idx: Vec<usize>,
    pub(crate) hem_y: Vec<f64>,
    pub(crate) sleeve: Vec<f64>,
    pub(crate) cut: Cut,
}

pub(crate) static LAYOUTS: Cache<(String, String, bool), Layout> = Cache::new();

pub(crate) fn jacket_layout(family: &str, gid: &str, lod: bool) -> Arc<Layout> {
    LAYOUTS.get((family.to_string(), gid.to_string(), lod), || {
        let s = surface(family, BASE_STAGE);
        let fl = base_fields(family);
        let cuts = garment_cuts(family, &s.p);
        let cut = cuts.get(gid).unwrap().clone();
        let (pos, idx, ff): (&[V3], Vec<usize>, &[[f64; 4]]) = if lod {
            (&s.lod_pos, topology().lod_tris.concat(), &fl.1)
        } else {
            (&s.pos, tris(family).to_vec(), &fl.0)
        };
        // The lapels: open in a V at the front, from the neckline down to the button.
        let p = &s.p;
        let zn = p[b("neck_01")].z;
        let y_button = p[b("spine_02")].y + if gid == "top.suit" { 0.02 } else { 0.0 };
        let y_top = p[b("neck_01")].y - 0.01;
        let open_w = radians(46.0);
        let g: Vec<f64> = pos
            .iter()
            .zip(ff)
            .map(|(q, f)| {
                let g = top_coverage(f, &cut);
                let th = q.x.atan2(q.z - zn);
                let vline =
                    y_button + (y_top - y_button) * clip(th.abs() / open_w, 0.0, 1.0).powf(1.2);
                // The shell's sleeves stop on the upper arm, sinking into the painted sleeve.
                g.min(vline - q.y).min((SHELL_SLEEVE - f[0]) * ARM_M)
            })
            .collect();
        let (cidx, src) = clip_by_field(&idx, &g);
        let f1: Vec<f64> = ff.iter().map(|f| f[1]).collect();
        let f0: Vec<f64> = ff.iter().map(|f| f[0]).collect();
        let hem_y = lerp_src(&src, &f1);
        let sleeve = lerp_src(&src, &f0);
        Layout {
            src,
            idx: cidx,
            hem_y,
            sleeve,
            cut,
        }
    })
}

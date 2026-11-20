//! The reference: the accepted sizes and their tolerance.

use super::*;

/// How far (×) from the median `minutes` of uses with the same animation a use's duration may be before it is
/// flagged (owner decision: today's short durations are by design, only outliers are).
pub const DURATION_SPREAD: f32 = 3.0;

fn ten_percent() -> f64 {
    0.10
}

/// The reference table (docs/review/reference.json): real-world ranges per kind of thing, and which kind each
/// object is.
#[derive(Debug, Deserialize)]
pub struct Reference {
    /// The share overall W, H and D may stray past their range (0.10: ±10 %); functional heights are strict.
    #[serde(default = "ten_percent")]
    pub tolerance: f64,
    #[serde(default)]
    pub sources: BTreeMap<String, Value>,
    #[serde(default)]
    pub kinds: BTreeMap<String, Kind>,
    /// {content group: kind}; also "<category>/<group>", which wins (two categories may have a `seating`).
    #[serde(default)]
    pub groups: BTreeMap<String, String>,
    /// {object id or model key: kind}: wins over `groups`.
    #[serde(default)]
    pub ids: BTreeMap<String, String>,
    /// Kinds where no use is expected (decor stays decor).
    #[serde(default)]
    pub decor: Vec<String>,
    /// Real sizes of the surfaces' patterns and the kerb (`surfaces.rs`).
    #[serde(default)]
    pub surfaces: BTreeMap<String, surfaces::SurfaceKind>,
}

/// A kind of thing: its real-world ranges (m; each optional), uses, minutes per use and mechanisms' angles.
#[derive(Debug, Default, Deserialize)]
pub struct Kind {
    #[serde(default)]
    pub label: String,
    pub w: Option<[f64; 2]>,
    pub h: Option<[f64; 2]>,
    pub d: Option<[f64; 2]>,
    pub seat: Option<[f64; 2]>,
    pub surface: Option<[f64; 2]>,
    pub mattress: Option<[f64; 2]>,
    #[serde(default)]
    pub uses: Vec<String>,
    #[serde(default)]
    pub minutes: BTreeMap<String, [f64; 2]>,
    #[serde(default)]
    pub mechanisms: BTreeMap<String, [f64; 2]>,
    #[serde(default)]
    pub sources: Vec<String>,
    /// What a plant's kind is of (`species`): its W and H are then its crown width and height, checked strictly
    /// from its model with the rest of the block, in place of the overall W × H × D.
    pub species: Option<Species>,
}

impl Reference {
    pub fn parse(text: &str) -> Result<Reference, String> {
        serde_json::from_str(text).map_err(|e| format!("{REFERENCE}: {e}"))
    }

    /// The kind an object is: `ids` by its id, then by its model key; else `groups` by "<category>/<group>",
    /// then by its group. None: none (an error for `sizes`). It may name a kind `kinds` doesn't have (`kind`).
    pub fn kind_of(&self, id: &str, model: &str, category: &str, group: &str) -> Option<&str> {
        let by_group = || {
            if group.is_empty() {
                None
            } else {
                self.groups
                    .get(&format!("{category}/{group}"))
                    .or_else(|| self.groups.get(group))
            }
        };
        self.ids
            .get(id)
            .or_else(|| self.ids.get(model))
            .or_else(by_group)
            .map(String::as_str)
    }

    /// The kind `name` names.
    pub fn kind(&self, name: &str) -> Option<&Kind> {
        self.kinds.get(name)
    }
}

/// Whether `v` lies in `[lo, hi]` widened by `tolerance` (a share of each end: 0.10 lets it be 10 % under the low
/// end or over the high one; 0 is strict), give or take half a millimetre (the models' numbers are f32).
pub fn within(v: f64, [lo, hi]: [f64; 2], tolerance: f64) -> bool {
    const SLACK: f64 = 0.0005;
    v >= lo * (1.0 - tolerance) - SLACK && v <= hi * (1.0 + tolerance) + SLACK
}

/// What `within` found wrong, as a table cell, to the millimetre (to the centimetre, 1.2104 over 1.21 read "1.21 >
/// 1.21"): `H 1.215 > 1.100 (0.78–1.00 ±10 %)`, `top 0.766 > 0.760 (0.71–0.76)`.
pub(crate) fn outside(name: &str, v: f64, [lo, hi]: [f64; 2], tolerance: f64) -> String {
    let (l, h) = (lo * (1.0 - tolerance), hi * (1.0 + tolerance));
    let side = if v < l {
        format!("{v:.3} < {l:.3}")
    } else {
        format!("{v:.3} > {h:.3}")
    };
    let allowance = if tolerance > 0.0 {
        format!(" ±{:.0} %", tolerance * 100.0)
    } else {
        String::new()
    };
    format!("{name} {side} ({lo:.2}–{hi:.2}{allowance})")
}

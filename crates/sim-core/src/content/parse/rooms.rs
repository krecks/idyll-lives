//! Room, wear, build and land shapes (content `rooms`, `objectRules`, `build`).

use super::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RoomKindRaw {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) label: Option<String>,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
    #[serde(default)]
    pub(crate) essentials: Vec<Vec<String>>,
    #[serde(default)]
    pub(crate) size: Option<[f32; 2]>,
    #[serde(default)]
    pub(crate) exclusive: bool,
    #[serde(default)]
    pub(crate) passage: bool,
    #[serde(default)]
    pub(crate) private: bool,
    #[serde(default)]
    pub(crate) claimed: bool,
    #[serde(default)]
    pub(crate) carpet: bool,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct WearRaw {
    pub(crate) per_use: Option<f32>,
    pub(crate) by_category: HashMap<String, f32>,
    pub(crate) per_quality: Option<f32>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct RepairRaw {
    pub(crate) label: Option<String>,
    pub(crate) minutes: Option<f32>,
    pub(crate) skill: Option<String>,
    pub(crate) skill_gain_per_hour: Option<f32>,
    pub(crate) chance: Option<f32>,
    pub(crate) chance_per_level: Option<f32>,
    pub(crate) interest: Option<f32>,
    pub(crate) cost: Option<f32>,
    pub(crate) tags: Option<Vec<String>>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct SetUpRaw {
    pub(crate) label: Option<String>,
    pub(crate) minutes: Option<f32>,
    pub(crate) fetch_minutes: Option<f32>,
    pub(crate) help_after_hours: Option<f32>,
    pub(crate) wait_hours: Option<f32>,
    pub(crate) parcel: Option<String>,
    pub(crate) tags: Option<Vec<String>>,
    pub(crate) anim: Option<String>,
    pub(crate) fetch_anim: Option<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct RoomRulesRaw {
    pub(crate) weights: Option<[f32; 5]>,
    pub(crate) window_tiles: Option<f32>,
    pub(crate) lamp_tiles: Option<f32>,
    pub(crate) decor_per_tile: Option<f32>,
    pub(crate) garden_decor_per_tile: Option<f32>,
    pub(crate) love: Option<f32>,
    pub(crate) dislike: Option<f32>,
    pub(crate) drift: Option<f32>,
    pub(crate) away: Option<f32>,
    pub(crate) preference: Option<f32>,
    pub(crate) love_feeling: Option<String>,
    pub(crate) dislike_feeling: Option<String>,
    pub(crate) decor_by_category: HashMap<String, f32>,
    pub(crate) dirt: HashMap<String, f32>,
    pub(crate) clean: CleanRaw,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct CleanRaw {
    pub(crate) label: Option<String>,
    pub(crate) minutes: Option<f32>,
    pub(crate) threshold: Option<f32>,
    pub(crate) amount: Option<f32>,
    pub(crate) interest: Option<f32>,
    pub(crate) tags: Option<Vec<String>>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct ObjectRulesRaw {
    /// Catalog categories whose 1×1 objects turn freely.
    pub(crate) free_rotation: Option<Vec<String>>,
    pub(crate) wear: WearRaw,
    pub(crate) repair: RepairRaw,
    pub(crate) set_up: SetUpRaw,
    pub(crate) max_quality: Option<u8>,
    pub(crate) quality_bonus: Option<f32>,
    pub(crate) upgrade_cost: Option<f32>,
    pub(crate) resale: Option<f32>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct BuildRaw {
    pub(crate) wall: Option<i64>,
    pub(crate) door: Option<i64>,
    pub(crate) window: Option<i64>,
    pub(crate) remove: Option<i64>,
    /// Fence per metre and gate, without fence styles.
    pub(crate) fence: Option<i64>,
    pub(crate) gate: Option<i64>,
    /// A garage on the home lot.
    pub(crate) garage: Option<i64>,
    /// A porch over a door, a porch post, and a porch canopy and a step platform a tile.
    pub(crate) porch: Option<i64>,
    pub(crate) porch_post: Option<i64>,
    pub(crate) porch_canopy: Option<i64>,
    pub(crate) porch_step: Option<i64>,
    /// Shaping the land (`LandPrices`); a price left out is the default's.
    pub(crate) land: LandPricesRaw,
    pub(crate) storeys: Option<u8>,
    #[serde(default)]
    pub(crate) basement: bool,
}

#[derive(Deserialize, Default)]
#[serde(default)]
pub(crate) struct LandPricesRaw {
    pub(crate) earth: Option<f64>,
    pub(crate) mountain: Option<f64>,
    pub(crate) water: Option<f64>,
    pub(crate) woods: Option<f64>,
    pub(crate) meadow: Option<f64>,
    pub(crate) fields: Option<f64>,
    pub(crate) clear: Option<f64>,
}

impl LandPricesRaw {
    /// The prices, each a number ≥ 0 (left out: the default's).
    pub(crate) fn prices(&self) -> Result<LandPrices, Error> {
        let d = LandPrices::default();
        let price = |name: &str, v: Option<f64>, or: f64| {
            let v = v.unwrap_or(or);
            if v.is_finite() && v >= 0.0 {
                Ok(v)
            } else {
                Err(Error::new(format!(
                    "build.land.{name} {v} must be a number ≥ 0"
                )))
            }
        };
        Ok(LandPrices {
            earth: price("earth", self.earth, d.earth)?,
            mountain: price("mountain", self.mountain, d.mountain)?,
            water: price("water", self.water, d.water)?,
            woods: price("woods", self.woods, d.woods)?,
            meadow: price("meadow", self.meadow, d.meadow)?,
            fields: price("fields", self.fields, d.fields)?,
            clear: price("clear", self.clear, d.clear)?,
        })
    }
}

#[derive(Deserialize)]
pub(crate) struct BuildStyleRaw {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) price: i64,
    /// How it's drawn (a floor covering's `wood`, `tile`, `carpet`...; see `BuildStyle::soft`).
    #[serde(default)]
    pub(crate) finish: Option<String>,
}

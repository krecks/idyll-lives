//! [(2007, 2064)]

use super::*;

/// How cars drive (content `cars`).
#[derive(Debug, Clone)]
pub struct CarRules {
    /// Top speed on country roads (m a second of drive time, `crate::DRIVE_SCALE`; `countrySpeed`,
    /// default 16).
    pub country_speed: f32,
}

impl Default for CarRules {
    fn default() -> Self {
        Self {
            country_speed: crate::travel::COUNTRY_SPEED,
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct CarsRaw {
    pub(crate) country_speed: Option<f32>,
}

/// The work taxi (content `taxi`, `taxi/`): who rides it and how it runs.
#[derive(Debug, Clone, Copy)]
pub struct TaxiRules {
    /// Seats for riders (`seats`, 1 to 4, default 4): more riders than that take more taxis.
    pub seats: usize,
    /// A way out of town on foot taking longer than this is a long way, ridden when the taxi gets
    /// them out sooner than the walk, its wait at the kerb included (game minutes,
    /// `minWalkMinutes`, default 10).
    pub min_walk: f32,
    /// Colleagues whose work (or school) starts within this of each other share one (minutes,
    /// `window`, default 15).
    pub window: f32,
    /// How long it waits at a rider's kerb for them (minutes, `waitMinutes`, default 2).
    pub wait: f32,
}

impl Default for TaxiRules {
    fn default() -> Self {
        Self {
            seats: 4,
            min_walk: 10.0,
            window: 15.0,
            wait: 2.0,
        }
    }
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct TaxiRaw {
    pub(crate) seats: Option<usize>,
    pub(crate) min_walk_minutes: Option<f32>,
    pub(crate) window: Option<f32>,
    pub(crate) wait_minutes: Option<f32>,
}

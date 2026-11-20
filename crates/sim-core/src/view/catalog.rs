//! The catalog view: grades, careers, the workweek, and the object and build rules.

use super::*;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogView<'a> {
    pub(crate) grades: Vec<GradeView<'a>>,
    pub categories: Vec<&'a str>,
    pub(crate) careers: Vec<CareerView<'a>>,
    pub(crate) workweek: Vec<WorkweekView<'a>>,
    pub probation_levels: f32,
    pub max_skill: f32,
    pub(crate) object_rules: ObjectRulesView,
    pub(crate) build: BuildView,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GradeView<'a> {
    pub id: &'a str,
    pub label: &'a str,
    pub pay_per_hour: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CareerView<'a> {
    pub id: &'a str,
    pub label: &'a str,
    /// Category id.
    pub category: Option<&'a str>,
    /// Skill ids practised at work.
    pub skills: Vec<&'a str>,
    pub(crate) levels: Vec<LevelView<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LevelView<'a> {
    pub title: &'a str,
    pub grade: Option<&'a str>,
    pub pay: i64,
    pub start: f32,
    pub hours: f32,
    pub days: Vec<u8>,
    /// `[skill id, level]`.
    pub requires: Vec<(&'a str, f32)>,
}

#[derive(Serialize)]
pub(crate) struct WorkweekView<'a> {
    pub fit: f32,
    pub days: i8,
    pub label: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObjectRulesView {
    pub max_quality: u8,
    pub quality_bonus: f32,
    pub upgrade_cost: f32,
    pub resale: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BuildView {
    pub wall: i64,
    pub door: i64,
    pub window: i64,
    pub remove: i64,
    /// A diagonal wall across one tile (`wall × 1.414`, rounded).
    pub diagonal_wall: i64,
    /// A metre of fence and a gate, without fence styles (styles carry their own prices).
    pub fence: i64,
    pub gate: i64,
    /// A garage on the home lot.
    pub garage: i64,
    /// A porch over a door; a porch post; a porch canopy and a step platform, a tile.
    pub porch: i64,
    pub porch_post: i64,
    pub porch_canopy: i64,
    pub porch_step: i64,
    /// Shaping the land (`content::LandPrices`): per cubic metre (`earth`, `mountain`) or square
    /// metre (the rest).
    pub land: crate::content::LandPrices,
}

/// Every career with all its levels, expanded from the content (sent to the UI once).
pub fn catalog_json(content: &Content) -> String {
    let careers = content
        .careers
        .iter()
        .map(|c| CareerView {
            id: &c.id,
            label: &c.label,
            category: c.category.map(|i| content.career_categories[i].id.as_str()),
            skills: c
                .skills
                .iter()
                .map(|&(s, _)| content.skills[s].id.as_str())
                .collect(),
            levels: c
                .levels
                .iter()
                .map(|l| LevelView {
                    title: &l.title,
                    grade: l.grade.map(|g| content.grades[g].id.as_str()),
                    pay: l.pay,
                    start: l.start_hour,
                    hours: l.hours,
                    days: weekdays(l.days),
                    requires: l
                        .requires
                        .iter()
                        .map(|&(s, v)| (content.skills[s].id.as_str(), v))
                        .collect(),
                })
                .collect(),
        })
        .collect();
    let rules = &content.object_rules;
    serde_json::to_string(&CatalogView {
        grades: content
            .grades
            .iter()
            .map(|g| GradeView {
                id: &g.id,
                label: &g.label,
                pay_per_hour: g.pay_per_hour,
            })
            .collect(),
        categories: content
            .career_categories
            .iter()
            .map(|c| c.id.as_str())
            .collect(),
        careers,
        workweek: content
            .career_rules
            .workweek
            .iter()
            .map(|w| WorkweekView {
                fit: w.fit,
                days: w.days,
                label: &w.label,
            })
            .collect(),
        probation_levels: content.career_rules.probation_levels,
        max_skill: content.skill_rules.max_level,
        object_rules: ObjectRulesView {
            max_quality: rules.max_quality,
            quality_bonus: rules.quality_bonus,
            upgrade_cost: rules.upgrade_cost,
            resale: rules.resale,
        },
        build: BuildView {
            wall: content.build.wall,
            door: content.build.door,
            window: content.build.window,
            remove: content.build.remove,
            diagonal_wall: content.build.diagonal_wall(),
            fence: content.build.fence,
            gate: content.build.gate,
            garage: content.build.garage,
            porch: content.build.porch,
            porch_post: content.build.porch_post,
            porch_canopy: content.build.porch_canopy,
            porch_step: content.build.porch_step,
            land: content.build.land,
        },
    })
    .expect("catalog serializes")
}

pub(crate) fn is_level(tall: &i16) -> bool {
    *tall == 0
}

/// A thing named in a sentence: "a snack", "an ice cream", "some toast", "some noodles".
pub fn some_of(label: &str) -> String {
    let thing = label.to_lowercase();
    if thing.ends_with('s') || ["toast", "takeout"].contains(&thing.as_str()) {
        format!("some {thing}")
    } else if thing.starts_with(['a', 'e', 'i', 'o', 'u']) {
        format!("an {thing}")
    } else {
        format!("a {thing}")
    }
}

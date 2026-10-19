//! Garment vocabulary shared by the body's fields and the clothes (clothes.rs): the shell
//! categories and flags written into a shell's fields, the sleeve parameter's scale, and the
//! landmarks (heights on the base body) the cuts are made from.
//!
//! Shell fields: (hem distance (m), flag, category, 0); categories 0 top, 1 bottom, 2 shoes; flags
//! 1 sole (drawn darker), 2 waistband, 3 skirt.

use super::V3;
use super::rig::{NB, b};

/// Categories (fields.z of garment vertices; the runtime colours them with the top / bottom / shoe colour).
pub const TOP: f64 = 0.0;
pub const BOTTOM: f64 = 1.0;
pub const SHOES: f64 = 2.0;
/// Part flags (fields.y): 1 = sole / welt, 2 = waistband, 3 = skirt.
pub const FLAG_SOLE: f64 = 1.0;
pub const FLAG_SKIRT: f64 = 3.0;
/// The sleeve parameter is scaled to metres in the predicates (about an arm's length).
pub const ARM_M: f64 = 0.55;

/// Heights (m) on a body's rest pose: pelvis, hips, knees, ankles, the hem and the waistband.
pub struct Landmarks {
    pub pelvis: f64,
    pub hip: f64,
    pub knee: f64,
    pub ankle: f64,
    pub hem: f64,
    pub waist: f64,
}

impl Landmarks {
    pub fn new(p: &[V3; NB], gender: &str) -> Self {
        let pelvis = p[b("pelvis")].y;
        let male = gender == "male";
        Landmarks {
            pelvis,
            hip: p[b("thigh_l")].y,
            knee: p[b("calf_l")].y,
            ankle: p[b("foot_l")].y,
            hem: pelvis + if male { 0.02 } else { 0.045 },
            waist: pelvis + if male { 0.075 } else { 0.09 },
        }
    }
}

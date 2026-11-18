//! The foodie pack (P6): the windowsill herb garden, the deli pantry, the espresso bar, the cookbook lectern, the
//! wood-fired pizza oven and the chef's island. Notes: NOTES.md.
//!
//! Each is built where the game places it (`GameFrame`, game axes; settled on the floor where it was built, and
//! `centre` on its `Model`). A fresh sprig grows in a pot (`sprig`), the pantry's doors swing open (`doors_open`),
//! the coffee runs and the steam wand swings out (`brew`), a page of the cookbook turns over (`page`), the oven door
//! lifts aside and the peel slides in (`door_off`) as the fire grows (`flames`), and the island's drawer pulls out
//! (`drawer_open`).
//!
//! Built with the forms and the kit: frame-and-panel fronts (the kit's shaker) with bar pulls, crown mouldings, herbs
//! of extruded leaves, lofted jars, bottles, salami and china, logs and loaves wobbled by `jitter`, bent iron and
//! steel in one sweep.
//!
//! Run:  cargo art pack_foodie [--only pizzaOven] [--preview]

use artkit::family::{Family, Model};
use artkit::json;

mod cafe;
mod common;
mod deli;
mod garden;
mod island;
mod oven;

pub use common::M;

/// The family.
pub fn family() -> Option<Family> {
    Some(Family::new(
        "pack_foodie",
        M.palette(),
        vec![
            Model::new("herbGarden", garden::herb_garden, (0.86, 1.293, 0.763))
                .key("model.foodie.herbGarden")
                .centre(0.0, -0.054)
                .motions(json!({"uses": {"tendHerbs": "sprig"}}))
                .paint(&[("cedar", "Body"), ("terracotta", "Pots")]),
            Model::new("deliPantry", deli::deli_pantry, (0.88, 1.99, 0.57))
                .key("model.foodie.deliPantry")
                .centre(0.0, -0.155)
                .motions(json!({"uses": {"nibble": "doors_open", "cheeseBoard": "doors_open"}, "hold": 2.5}))
                .paint(&[("cream", "Body"), ("marble", "Worktop"), ("honey", "Shelves")]),
            Model::new("espressoBar", cafe::espresso_bar, (0.76, 1.509, 0.514))
                .key("model.foodie.espressoBar")
                .centre(0.0, -0.18)
                .motions(json!({"uses": {"espresso": "brew", "latteArt": "brew", "pour": "brew"}, "hold": 4.0}))
                .paint(&[("teal", "Body"), ("oak_top", "Worktop"), ("machine", "Machine")]),
            Model::new("cookbookLectern", cafe::cookbook_lectern, (0.836, 1.147, 0.588))
                .key("model.foodie.cookbookLectern")
                .centre(0.022, -0.123)
                .motions(json!({"uses": {"studyRecipes": "page", "foodMagazines": "page"}}))
                .paint(&[("walnut", "Body"), ("honey_wood", "Stool & ledge"), ("cover", "Cookbook")]),
            Model::new("pizzaOven", oven::pizza_oven, (1.89, 1.9, 0.916))
                .key("model.foodie.pizzaOven")
                .centre(0.005, 0.018)
                // a bake (the pizza, or any `bake` step of a recipe at it) takes the door off and lights the fire
                .motions(json!({"uses": {"bakePizza": ["door_off", "flames"], "bake": ["door_off", "flames"], "warmUp": "flames"}}))
                .paint(&[("plaster", "Body"), ("worktop", "Worktop"), ("brick", "Brick")]),
            Model::new("chefsIsland", island::chefs_island, (1.86, 1.128, 0.889))
                .key("model.foodie.chefsIsland")
                .centre(0.0, 0.0154)
                // the drawer opens for the prep step's utensils (chop), the ring under the pan glows while it cooks
                // (stir): keyed by the recipe's step tags, so each step at it plays its own (A-systems-03)
                .motions(json!({"uses": {"chop": {"clip": "drawer_open", "hold": 2.5}, "stir": "burner_on"}}))
                .paint(&[("cabinet", "Body"), ("marble", "Worktop"), ("oak", "Stools")]),
        ],
        "Foodie pack models (crates/art/packs/src/pack_foodie, built by cargo art). Listed in the base manifest's \
         packs. The materials map onto the game's finishes so a model costs one draw call per finish group. A sprig \
         grows, the pantry doors swing, the coffee runs, a page turns, the oven door lifts aside for the peel as the \
         fire grows, and the island's drawer pulls out.",
    ))
}

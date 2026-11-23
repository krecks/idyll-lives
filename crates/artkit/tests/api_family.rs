//! A model with a palette of its own: its pack entry's materials are read from it, not from the family's (the
//! garden things and the street furniture, whose `body` differs from model to model in colour and finish).

use artkit::cube;
use artkit::family::{Family, Model, build_model};
use artkit::materials::{Finish, Finishes, Mat, Palette};
use artkit::parts::{join_all, settle};
use artkit::scene::{self, Obj};

const STONE: Mat = Mat::new("body", "#B4AEA2", "matte");
const WOOD: Mat = Mat::new("body", "#8A6A4E", "wood");
const ROOF: Mat = Mat::new("roof", "#C0473A", "satin");

fn thing(body: Mat) -> Vec<Obj> {
    cube!("base", (0.4, 0.3, 0.2), material = body, bevel = 0);
    cube!(
        "roof",
        (0.5, 0.4, 0.02),
        at = (0, 0, 0.11),
        material = ROOF,
        bevel = 0
    );
    settle(&[join_all("thing")])
}

fn pond() -> Vec<Obj> {
    thing(STONE)
}

fn planter() -> Vec<Obj> {
    thing(WOOD)
}

#[test]
fn a_models_own_palette_names_its_materials() {
    let mut finishes = Finishes::new();
    for (name, textured) in [("matte", false), ("satin", false), ("wood", true)] {
        finishes.insert(
            name.to_string(),
            Finish {
                roughness: 0.6,
                metallic: 0.0,
                textured,
            },
        );
    }
    scene::set_finishes(finishes);
    let shared = Palette::new(&[STONE, ROOF]);
    let family = Family::new(
        "garden_things",
        shared.clone(),
        vec![
            Model::new("pond", pond, (0.5, 0.22, 0.4)),
            Model::new("planterBox", planter, (0.5, 0.22, 0.4)).palette(shared.with(&[WOOD])),
        ],
        "a test's garden things",
    );
    let body = |id: usize| {
        let out = build_model(&family, &family.models[id]).unwrap();
        out.entry.dumps()
    };
    assert!(
        body(0).contains(r#""body": {"finish": "material.finish.matte"}"#),
        "{}",
        body(0)
    );
    assert!(
        body(1).contains(r#""body": {"finish": "material.finish.wood"}"#),
        "{}",
        body(1)
    );
    assert_eq!(
        family.palette_of(&family.models[0]).get("body").finish,
        "matte"
    );
    assert_eq!(
        family.palette_of(&family.models[1]).get("body").finish,
        "wood"
    );
}

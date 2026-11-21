//! The template family (`ll_art::template`, the one a new family copies) builds: its size, its two clips, its
//! pack entry.

use artkit::family::build_model;
use artkit::scene;
use ll_tools::checkglb::Paths;

#[test]
fn the_template_chest_builds_with_its_lid_and_drawer() {
    let finishes = ll_art::finishes(&Paths::repo()).unwrap();
    let family = ll_art::template::family();
    family.palette.check(&finishes).unwrap();
    scene::set_finishes(finishes);
    let out = build_model(&family, &family.models[0]).unwrap();
    assert_eq!(out.clips, ["drawer_open", "lid_open"]);
    assert_eq!(&out.glb[0..4], b"glTF");
    let entry = format!("{:?}", out.entry);
    for field in [
        "../../models/template/chest.glb",
        "lid_open",
        "putAway",
        "Body",
    ] {
        assert!(entry.contains(field), "{field}: {entry}");
    }
    // nothing but its preview names it on the command line
    assert!(
        ll_art::registry()
            .iter()
            .all(|(n, _)| *n != ll_art::template::NAME)
    );
    let err = ll_art::pick(&["template".to_string()], &ll_art::registry(), false).unwrap_err();
    assert!(err.contains("only previews"), "{err}");
}

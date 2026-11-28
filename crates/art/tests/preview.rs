//! `cargo art <family> --preview --sheet`: the Italian cypress (the trees family's, kept with the kernel's tests),
//! drawn into a scratch folder.
#[path = "../../artkit/tests/all/common/cypress.rs"]
mod cypress;

use ll_tools::checkglb::Paths;

#[test]
fn the_cypress_is_drawn_with_its_perches_far_version_and_sheet() {
    let repo = Paths::repo();
    let into = std::env::temp_dir().join(format!("ll-art-preview-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&into);
    let family = cypress::family();
    // a PNG's size from its header (IHDR: width and height, big-endian, from byte 16), and its bytes
    let read = |name: &str| {
        let path = into.join("trees").join(name);
        let png = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert_eq!(&png[1..4], b"PNG", "{name}");
        let size = |at: usize| u32::from_be_bytes(png[at..at + 4].try_into().unwrap());
        (size(16), size(20), png.clone())
    };
    ll_art::preview::run(&repo, &family, None, true, &into).unwrap();
    let (w, h, first) = read("cypress.png");
    assert_eq!((w, h), (640, 640));
    for name in [
        "cypress@perches.png",
        "cypress@perches.top.png",
        "cypress.far.png",
    ] {
        assert_eq!(read(name).0, 640, "{name}");
    }
    // the sheet: its four sides and its baked shading, no clips
    assert_eq!(read("cypress.sheet.png").0, 5 * 320 + 4 * 4);
    // a second run draws the same pixels
    ll_art::preview::run(&repo, &family, Some(&["cypress".to_string()]), false, &into).unwrap();
    assert!(read("cypress.png").2 == first);
    assert!(
        ll_art::preview::run(&repo, &family, Some(&["nope".to_string()]), false, &into).is_err()
    );
    let _ = std::fs::remove_dir_all(&into);
}

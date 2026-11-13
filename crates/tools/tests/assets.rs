//! `cargo assets` over every shipped model: each is made smaller (none is left as it is), and read back
//! it is its source, every triangle within the quantisation's steps, its shading and animations exact.

use ll_tools::assets::{model, models};

#[test]
fn every_shipped_model_is_made_smaller_and_comes_back_as_it_was() {
    let models = models().expect("the assets");
    assert!(models.len() > 100, "only {} models", models.len());
    let mut foliage = 0;
    let mut before = 0;
    let mut after = 0;
    let mut triangles = 0;
    for (file, rel, leaves) in &models {
        let source = std::fs::read(file).unwrap_or_else(|e| panic!("{rel}: {e}"));
        let small = model::optimize(&source, *leaves)
            .unwrap_or_else(|why| panic!("{rel} is left as it is: {why}"));
        let drift = model::read(&source)
            .and_then(|s| model::read(&small).and_then(|m| model::compare(&s, &m)))
            .unwrap_or_else(|e| panic!("{rel}: {e}"));
        // (A model with foliage keeps its floats: nothing moves.)
        if *leaves {
            foliage += 1;
            assert_eq!(
                (drift.position, drift.normal, drift.uv),
                (0.0, 0.0, 0.0),
                "{rel}"
            );
        }
        // Half a millimetre at the very most (the largest model's box over 14 bits is 0.3 mm).
        assert!(drift.position < 0.0005, "{rel}: {drift:?}");
        before += source.len();
        after += small.len();
        triangles += drift.triangles;
    }
    assert!(foliage > 10, "only {foliage} models with foliage");
    assert!(triangles > 1_000_000, "only {triangles} triangles");
    // Under half the size, all together (46 % when this was written).
    assert!(after * 2 < before, "{after} bytes of {before}");
}

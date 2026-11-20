use super::*;

fn committed(file: &str) -> std::path::PathBuf {
    crate::content::root()
        .join("web/public/assets/textures/leafcards")
        .join(file)
}

/// The committed tiles.json is the table's (artkit's `botany::ATLAS`), and the species check reads it.
#[test]
fn the_committed_tiles_are_the_tables() {
    let text = std::fs::read_to_string(committed("tiles.json")).unwrap();
    assert!(text == tiles_json(), "run `cargo tex leafcards`");
    let atlas = crate::species::Atlas::parse(&text).unwrap();
    assert_eq!(atlas.tiles.len(), ATLAS.len());
    for (i, t) in atlas.tiles.iter().enumerate() {
        let [x, y, w, h] = tile_px(i).map(|v| v as f64 / ATLAS_PX as f64);
        assert_eq!(t.rect, [x, y, x + w, y + h]);
        assert_eq!(t.class == "core", ATLAS[i].2 == Pictured::Core);
    }
}

/// The committed atlas's tiles are what the generator draws: a cluster and a hanging strand drawn again, texel
/// for texel (the whole atlas takes half a minute: `cargo tex leafcards`, then `git status`).
#[test]
fn the_committed_atlas_is_drawn_from_this_source() {
    let atlas = image::open(committed("albedo.png")).unwrap().to_rgba8();
    assert_eq!(atlas.dimensions(), (ATLAS_PX as u32, ATLAS_PX as u32));
    let strand = ATLAS.iter().position(|t| t.2 == Pictured::Strand).unwrap();
    for i in [0, strand] {
        let drawn = tile(i);
        let [x, y, w, h] = tile_px(i);
        for py in 0..h {
            for px in 0..w {
                let here = atlas.get_pixel((x + px) as u32, (y + py) as u32).0;
                assert!(
                    here == drawn.at(px, py),
                    "tile {i} at {px}, {py}: run `cargo tex leafcards`"
                );
            }
        }
    }
}

/// Every cluster's covered texels have one mean luminance, and every dense tile's texels another: a card's
/// colour tints each species' leaves as light as the next's.
#[test]
fn the_tiles_are_as_light_as_one_another() {
    let atlas = image::open(committed("albedo.png")).unwrap().to_rgba8();
    for (i, (name, _, pictured)) in ATLAS.iter().enumerate() {
        let [x, y, w, h] = tile_px(i);
        let (mut sum, mut n) = (0.0, 0.0);
        for py in y..y + h {
            for px in x..x + w {
                let p = atlas.get_pixel(px as u32, py as u32).0;
                if p[3] > 127 {
                    sum += luminance(&p);
                    n += 1.0;
                }
            }
        }
        let aim = if *pictured == Pictured::Core {
            DENSE_LIGHT
        } else {
            CLUSTER_LIGHT
        };
        assert!(
            (sum / n - aim).abs() < 0.01,
            "{name} {pictured:?}: {} for {aim}",
            sum / n
        );
    }
}

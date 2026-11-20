//! The tiles: the atlas the leaves are packed into.

use super::*;

// ---- the tiles --------------------------------------------------------------------------------------------------

/// A species' cards' mean width, metres: what its cluster tiles show (`cargo art trees`' models, measured), and the
/// size and width of its leaves there. A little larger than life where the leaf's shape is the species' mark and
/// would otherwise be a few pixels at the play camera.
pub(super) struct Kind {
    name: &'static str,
    across: f64,
    leaf: Leaf,
    size: (f64, f64),
    wide: f64,
}

pub(super) const KINDS: [Kind; 9] = [
    Kind {
        name: "oak",
        across: 0.62,
        leaf: oak_leaf,
        size: (0.135, 0.17),
        wide: 0.62,
    },
    Kind {
        name: "maple",
        across: 0.67,
        leaf: maple_leaf,
        size: (0.15, 0.18),
        wide: 1.5,
    },
    Kind {
        name: "birch",
        across: 0.49,
        leaf: birch_leaf,
        size: (0.058, 0.072),
        wide: 0.8,
    },
    Kind {
        name: "cherry",
        across: 0.55,
        leaf: cherry_leaf,
        size: (0.1, 0.13),
        wide: 0.5,
    },
    Kind {
        name: "apple",
        across: 0.6,
        leaf: apple_leaf,
        size: (0.095, 0.12),
        wide: 0.65,
    },
    Kind {
        name: "japaneseMaple",
        across: 0.39,
        leaf: japanese_maple_leaf,
        size: (0.085, 0.105),
        wide: 1.7,
    },
    Kind {
        name: "magnolia",
        across: 0.54,
        leaf: magnolia_leaf,
        size: (0.15, 0.185),
        wide: 0.55,
    },
    Kind {
        name: "willow",
        across: 0.74,
        leaf: willow_leaf,
        size: (0.11, 0.145),
        wide: 0.2,
    },
    Kind {
        name: "lemon",
        across: 0.21,
        leaf: lemon_leaf,
        size: (0.075, 0.09),
        wide: 0.48,
    },
];

/// A willow strand's card, metres (crates/art/nature/src/trees/willow.rs: a strand 0.2 wide, its cards half as wide
/// again and 0.75 tall).
pub(super) const STRAND: (f64, f64) = (0.3, 0.75);

/// Tile `i` of the atlas (`ATLAS`'s order) drawn: `TILE_PX` a side.
pub(super) fn tile(i: usize) -> Picture {
    let (name, _, pictured) = ATLAS[i];
    let kind = KINDS
        .iter()
        .find(|k| k.name == name)
        .unwrap_or_else(|| panic!("no leaves for {name}"));
    // (which of its species' pictures of the kind it is)
    let nth = (0..i)
        .filter(|&j| ATLAS[j].0 == name && ATLAS[j].2 == pictured)
        .count();
    let mut sketch = Sketch::new();
    let mut rng = Random::new(
        (1000 * (i / 4 + 1) + nth) as i128 + if pictured == Pictured::Core { 500 } else { 0 },
    );
    let fine = FINE * kind.across;
    let wood = tone(0.4, 1.2);
    let base = Spray {
        leaf: kind.leaf,
        across: kind.across,
        size: kind.size,
        wide: kind.wide,
        twigs: 6,
        turn: [0.0, 0.6, 2.2][nth % 3],
        long: (0.6, 0.85),
        wood: (0.006, 0.003, wood),
        per: 5,
        from: 0.0,
        opposite: false,
        lean: (1.1, 0.35),
        tip: 0,
        fill: 10,
        fill_out: 0.7,
        top: 3,
        fan: TAU,
        foot: 0.0,
    };
    match pictured {
        Pictured::Core => {
            let across = kind.across * 1.08;
            match name {
                "cherry" => blossom_wall(&mut sketch, &mut rng, across),
                _ => wall(
                    &mut sketch,
                    &mut rng,
                    kind.leaf,
                    across,
                    kind.size,
                    kind.wide.min(0.9),
                ),
            }
            let d = (TILE_PX as f64 * DENSE) as usize;
            dense(draw(&sketch, (0.0, 0.0), across, d, fine, 1.0))
        }
        Pictured::Strand => {
            strands(&mut sketch, &mut rng, STRAND.0, STRAND.1, 5 + nth % 2);
            cut_out(draw(
                &sketch,
                (0.0, 0.0),
                STRAND.1,
                TILE_PX,
                fine,
                STRAND.1 / STRAND.0,
            ))
        }
        Pictured::Blossom => {
            blossom(
                &mut sketch,
                &mut rng,
                kind.across,
                4 + nth % 2,
                [0.0, 0.7, 2.0][nth % 3],
            );
            cut_out(draw(&sketch, (0.0, 0.0), kind.across, TILE_PX, fine, 1.0))
        }
        Pictured::Cluster => {
            let s = match name {
                // bunches of leaves at its twigs' ends and along them
                "oak" => Spray {
                    twigs: 5 + nth % 2,
                    per: 3,
                    from: 0.15,
                    tip: 4,
                    fill: 12,
                    lean: (1.2, 0.7),
                    long: (0.55, 0.8),
                    ..base
                },
                // pairs of broad leaves held flat, in layers
                "maple" => Spray {
                    twigs: 4 + nth % 2,
                    per: 2,
                    from: 0.3,
                    opposite: true,
                    tip: 1,
                    fill: 4,
                    top: 1,
                    lean: (1.15, 0.8),
                    long: (0.5, 0.72),
                    ..base
                },
                // small leaves down slender dark twigs that hang to one side, the sky between them
                "birch" => Spray {
                    twigs: 9 + nth % 2,
                    per: 10,
                    from: 0.06,
                    tip: 1,
                    fill: 30,
                    fill_out: 0.9,
                    top: 0,
                    fan: 1.9,
                    foot: 0.82,
                    lean: (1.3, 0.8),
                    long: (1.2, 1.8),
                    wood: (0.0035, 0.0015, tone(0.22, 0.8)),
                    ..base
                },
                // rosettes on short spurs
                "apple" => Spray {
                    twigs: 6 + nth % 2,
                    per: 3,
                    from: 0.2,
                    tip: 4,
                    fill: 14,
                    lean: (1.2, 0.6),
                    ..base
                },
                // pairs of fine-fingered leaves in flat layers
                "japaneseMaple" => Spray {
                    twigs: 5 + nth % 2,
                    per: 2,
                    from: 0.35,
                    opposite: true,
                    tip: 1,
                    fill: 10,
                    top: 2,
                    lean: (1.0, 0.7),
                    long: (0.5, 0.75),
                    wood: (0.003, 0.0015, tone(0.3, 1.5)),
                    ..base
                },
                // a whorl of big leaves round each twig's end
                "magnolia" => Spray {
                    twigs: 4 + nth % 2,
                    per: 1,
                    from: 0.3,
                    tip: 4,
                    fill: 7,
                    fill_out: 0.5,
                    lean: (1.2, 0.8),
                    long: (0.35, 0.58),
                    wood: (0.008, 0.005, wood),
                    ..base
                },
                // long narrow leaves down arching golden shoots
                "willow" => Spray {
                    twigs: 14 + nth % 2,
                    per: 10,
                    from: 0.08,
                    tip: 1,
                    fill: 34,
                    fill_out: 0.85,
                    top: 6,
                    lean: (0.8, 0.45),
                    long: (0.7, 0.98),
                    wood: (0.004, 0.002, tone(0.7, 1.6)),
                    ..base
                },
                // a few glossy leaves on short stems
                "lemon" => Spray {
                    twigs: 4 + nth % 2,
                    per: 3,
                    from: 0.15,
                    tip: 1,
                    fill: 5,
                    fill_out: 0.6,
                    top: 2,
                    lean: (1.0, 0.6),
                    long: (0.45, 0.65),
                    wood: (0.004, 0.003, tone(0.55, 0.9)),
                    ..base
                },
                _ => base,
            };
            spray(&mut sketch, &mut rng, s);
            cut_out(draw(&sketch, (0.0, 0.0), kind.across, TILE_PX, fine, 1.0))
        }
    }
}

/// A texel's luminance, linear.
pub(super) fn luminance(p: &[u8; 4]) -> f32 {
    let c = raster::linear([p[0], p[1], p[2]]);
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// `picture`'s texels `keep` says, lit by `k`.
pub(super) fn relight(picture: &mut Picture, k: f32) {
    for p in picture.rgba.as_chunks_mut::<4>().0 {
        let c = raster::linear([p[0], p[1], p[2]]);
        for i in 0..3 {
            p[i] = encode(c[i] * k);
        }
    }
}

/// Linear light as an sRGB byte.
pub(super) fn encode(c: f32) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let v = if c <= 0.003_130_8 {
        c * 12.92
    } else {
        1.055 * pow(c as f64, 1.0 / 2.4) as f32 - 0.055
    };
    (v * 255.0).round() as u8
}

/// A cluster as drawn, as its tile: its covered texels' mean luminance `CLUSTER_LIGHT`, its colour bled out into the
/// clear texels round it. A cluster that reaches its tile's edge is a fault in its drawing.
pub(super) fn cut_out(mut picture: Picture) -> Picture {
    let (w, h) = (picture.width, picture.height);
    let covered: Vec<&[u8; 4]> = picture
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[3] > 127)
        .collect();
    let mean = covered.iter().map(|p| luminance(p)).sum::<f32>() / covered.len().max(1) as f32;
    relight(&mut picture, CLUSTER_LIGHT / mean);
    for y in 0..h {
        for x in 0..w {
            let edge = x < 2 || y < 2 || x >= w - 2 || y >= h - 2;
            assert!(
                !edge || picture.at(x, y)[3] == 0,
                "a cluster reaches its tile's edge at {x}, {y}"
            );
        }
    }
    bleed(&mut picture);
    picture
}

/// A wall of leaves as drawn (`DENSE` of a tile across), as its tile: opaque in the tile's middle, its gaps the
/// dark of a crown's inside, every texel's mean luminance `DENSE_LIGHT`; round it a clear gutter in its mean colour.
pub(super) fn dense(wall: Picture) -> Picture {
    let d = wall.width;
    let o = (TILE_PX - d) / 2;
    let covered: Vec<&[u8; 4]> = wall
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[3] > 127)
        .collect();
    let n = covered.len().max(1) as f32;
    let mean = covered.iter().fold([0.0f32; 3], |s, p| {
        [
            s[0] + p[0] as f32 / n,
            s[1] + p[1] as f32 / n,
            s[2] + p[2] as f32 / n,
        ]
    });
    // (mixed as the bytes are, as the first atlas was)
    let mut solid = Picture::filled(d, d, [0, 0, 0, 255]);
    for (to, p) in solid
        .rgba
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(wall.rgba.as_chunks::<4>().0)
    {
        let a = p[3] as f32 / 255.0;
        for k in 0..3 {
            to[k] = (p[k] as f32 * a + mean[k] * 0.4 * (1.0 - a))
                .round()
                .clamp(0.0, 255.0) as u8;
        }
    }
    let texels = solid.rgba.as_chunks::<4>().0;
    let light = texels.iter().map(luminance).sum::<f32>() / texels.len() as f32;
    relight(&mut solid, DENSE_LIGHT / light);
    let texels = solid.rgba.as_chunks::<4>().0;
    let m = texels.len() as f32;
    let all = texels.iter().fold([0.0f32; 3], |s, p| {
        [
            s[0] + p[0] as f32 / m,
            s[1] + p[1] as f32 / m,
            s[2] + p[2] as f32 / m,
        ]
    });
    let mut out = Picture::filled(
        TILE_PX,
        TILE_PX,
        [
            all[0].round() as u8,
            all[1].round() as u8,
            all[2].round() as u8,
            0,
        ],
    );
    out.paste(&solid, o, o);
    out
}

/// The atlas: `ATLAS_PX` a side, a tile of `ATLAS` a cell (each drawn on a thread of its own, from its own random
/// numbers), clear between and round them.
pub fn atlas() -> Picture {
    let tiles: Vec<Picture> = std::thread::scope(|scope| {
        let drawing: Vec<_> = (0..ATLAS.len())
            .map(|i| scope.spawn(move || tile(i)))
            .collect();
        drawing
            .into_iter()
            .map(|t| t.join().unwrap_or_else(|e| std::panic::resume_unwind(e)))
            .collect()
    });
    // (the clear texels of the edge and of the cells take the tiles' mean colour)
    let n = (tiles.len() * TILE_PX * TILE_PX) as f32;
    let mut mean = [0.0f32; 3];
    for p in tiles.iter().flat_map(|t| t.rgba.as_chunks::<4>().0) {
        for k in 0..3 {
            mean[k] += p[k] as f32 / n;
        }
    }
    let mut out = Picture::filled(
        ATLAS_PX,
        ATLAS_PX,
        [
            mean[0].round() as u8,
            mean[1].round() as u8,
            mean[2].round() as u8,
            0,
        ],
    );
    for (i, t) in tiles.iter().enumerate() {
        let [x, y, _, _] = tile_px(i);
        out.paste(t, x, y);
    }
    out
}

/// The atlas's tiles as crates/tools/src/species.rs reads them (tiles.json): each tile's name, its leaf's class
/// (`core` for a dense tile, with the leaf it is of), what it pictures, and its rectangle in a model's UVs (u right, v
/// down from the picture's top).
pub fn tiles_json() -> String {
    let mut out = format!("{{\n  \"size\": {ATLAS_PX},\n  \"tiles\": [\n");
    for (i, (name, class, pictured)) in ATLAS.iter().enumerate() {
        let nth = (0..i)
            .filter(|&j| ATLAS[j].0 == *name && ATLAS[j].2 == *pictured)
            .count()
            + 1;
        let [x, y, w, h] = tile_px(i).map(|v| v as f64 / ATLAS_PX as f64);
        let (kind, named, class, leaf) = match pictured {
            Pictured::Cluster => ("cluster", format!("{name} {nth}"), *class, String::new()),
            Pictured::Blossom => (
                "blossom",
                format!("{name} blossom {nth}"),
                *class,
                String::new(),
            ),
            Pictured::Strand => (
                "strand",
                format!("{name} strand {nth}"),
                *class,
                String::new(),
            ),
            Pictured::Core => (
                "core",
                format!("{name} core"),
                "core",
                format!(", \"leaf\": \"{class}\""),
            ),
        };
        out += &format!(
            "    {{\"name\": \"{named}\", \"class\": \"{class}\", \"kind\": \"{kind}\"{leaf}, \"rect\": [{}, {}, {}, {}]}}{}\n",
            x,
            y,
            x + w,
            y + h,
            if i + 1 < ATLAS.len() { "," } else { "" }
        );
    }
    out + "  ]\n}\n"
}

/// Writes `picture` as a PNG.
pub(super) fn save(picture: &Picture, out: &Path) -> Result<(), String> {
    use image::ImageEncoder;
    let f = std::fs::File::create(out).map_err(|e| format!("{}: {e}", out.display()))?;
    image::codecs::png::PngEncoder::new_with_quality(
        std::io::BufWriter::new(f),
        image::codecs::png::CompressionType::Best,
        image::codecs::png::FilterType::Adaptive,
    )
    .write_image(
        &picture.rgba,
        picture.width as u32,
        picture.height as u32,
        image::ExtendedColorType::Rgba8,
    )
    .map_err(|e| format!("{}: {e}", out.display()))
}

/// `cargo tex leafcards [dir]`: the atlas (albedo.png) and its tiles (tiles.json), into the assets' textures/leafcards
/// unless a folder is given.
pub fn run(out: Option<&str>) -> Result<(), String> {
    let dir = match out {
        Some(dir) => Path::new(dir).to_path_buf(),
        None => crate::content::root().join("web/public/assets/textures/leafcards"),
    };
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let picture = atlas();
    save(&picture, &dir.join("albedo.png"))?;
    let tiles = dir.join("tiles.json");
    std::fs::write(&tiles, tiles_json()).map_err(|e| format!("{}: {e}", tiles.display()))?;
    for (i, (name, _, pictured)) in ATLAS.iter().enumerate() {
        let [x, y, w, h] = tile_px(i);
        let covered = (y..y + h)
            .flat_map(|py| (x..x + w).map(move |px| (px, py)))
            .filter(|&(px, py)| picture.at(px, py)[3] > 127)
            .count();
        println!(
            "{name} {pictured:?}: {:.0} % covered",
            100.0 * covered as f64 / (w * h) as f64
        );
    }
    println!("wrote {} and tiles.json", dir.join("albedo.png").display());
    Ok(())
}

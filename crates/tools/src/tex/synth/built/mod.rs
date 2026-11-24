//! The built surfaces: brick, stonework, tiles, pavers, boards, shingles, plaster, concrete and asphalt, and
//! the floor coverings households lay (`floors.rs`: parquet, cork, lino, concrete, sisal; `tiles.rs`: chequer,
//! hex mosaic, terrazzo, marble, brick pavers, Victorian geometric tiles).
//!
//! Each set is drawn in real units: the tile's size in metres comes from the manifest's `uvScale`
//! (`TILE`), so a 9 mm mortar joint is 9 mm whatever the texture size. A generator returns height,
//! colour and roughness per pixel (`Px`); ambient occlusion comes from the height's cavities at the
//! scales the set picks. On walls and roofs the game maps image-down to world-up (v grows upwards and
//! the texture is not flipped), so lap boards and shingles are drawn with their thick butt at the top
//! of their course in the image.

use super::{HeightMap, Img, Noise, Surface, lerp, mix, mul, par_map, rgb, smoothstep, to_u8};
mod brick;
mod floors;
mod garden;
mod leather;
mod paving;
mod plaster;
mod roof;
mod stone;
mod tiles;
mod wood;

use brick::brick;
use floors::{concrete, cork, lino, parquet, sisal};
use garden::{flagstone, gravel, walk_concrete};
use leather::leather;
use paving::{asphalt, floor_tile, hexpaving, sidewalk};
use plaster::plaster;
use roof::roof;
use stone::stone;
use tiles::{checker, encaustic, hex_mosaic, marble, pavers, terrazzo};
use wood::{floor_wood, laminate, siding};

/// The sets made here. `floor_wood.calm`: a set's variant ([`variants`]), drawn only when named.
pub fn generate(name: &str, n: &Noise, size: usize) -> Option<Surface> {
    let (name, variant) = name.split_once('.').unwrap_or((name, ""));
    Some(match name {
        _ if !variants(name).contains(&variant) => return None,
        "brick" => brick(n, size),
        "stone_foundation" => stone(n, size),
        "floor_tile" => floor_tile(n, size),
        "path_hexpaving" => hexpaving(n, size),
        "path_concrete" => walk_concrete(n, size),
        "path_flagstone" => flagstone(n, size),
        "path_gravel" => gravel(n, size),
        "floor_wood" => floor_wood(n, size, &boards(variant)),
        "floor_laminate" => laminate(n, size),
        "siding_wood" => siding(n, size),
        "roof_shingles" => roof(n, size),
        "wall_plaster" => plaster(n, size),
        "sidewalk" => sidewalk(n, size),
        "road_asphalt" => asphalt(n, size),
        "leather" => leather(n, size),
        "floor_parquet" => parquet(n, size),
        "floor_checker" => checker(n, size),
        "floor_hex" => hex_mosaic(n, size),
        "floor_terrazzo" => terrazzo(n, size),
        "floor_marble" => marble(n, size),
        "floor_cork" => cork(n, size),
        "floor_lino" => lino(n, size),
        "floor_concrete" => concrete(n, size),
        "floor_pavers" => pavers(n, size),
        "floor_encaustic" => encaustic(n, size),
        "floor_sisal" => sisal(n, size),
        _ => return None,
    })
}

// --- helpers --------------------------------------------------------------------------------------

/// One pixel: height (any units, stretched later), sRGB colour, roughness.
#[derive(Clone, Copy, Default)]
struct Px {
    h: f64,
    c: [f64; 3],
    r: f64,
}

fn sat(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

/// Periodic fractal noise centred on 0 (roughly -0.35..0.35).
fn fbm(n: &Noise, u: f64, v: f64, fx: i64, fy: i64, oct: u32) -> f64 {
    n.fbm2(u.rem_euclid(1.0), v.rem_euclid(1.0), fx, fy, oct) - 0.5
}

/// Blocky white noise on a `cells` grid (per pixel when `cells` is the texture size), centred on 0.
fn white(n: &Noise, u: f64, v: f64, cells: i64, salt: u32) -> f64 {
    let (i, j) = (
        (u.rem_euclid(1.0) * cells as f64) as i64,
        (v.rem_euclid(1.0) * cells as f64) as i64,
    );
    n.rand(i.min(cells - 1), j.min(cells - 1), salt) - 0.5
}

/// Non-periodic fractal noise in local coordinates (inside one board or stone), centred on 0.
fn fbm_free(n: &Noise, x: f64, y: f64, oct: u32) -> f64 {
    const P: i64 = 1 << 20;
    let (mut f, mut amp, mut sum, mut norm) = (1.0, 1.0, 0.0, 0.0);
    for o in 0..oct {
        sum += amp * n.perlin2(x * f + 1000.0, y * f + 1000.0, P, P, o);
        norm += amp;
        amp *= 0.5;
        f *= 2.0;
    }
    0.5 * sum / norm
}

/// The nearest feature point of a periodic jittered grid: its distance in cell units and its cell.
struct Worley {
    f1: f64,
    cell: (i64, i64),
}

fn worley(n: &Noise, u: f64, v: f64, nx: i64, ny: i64, jitter: f64, salt: u32) -> Worley {
    let x = u.rem_euclid(1.0) * nx as f64;
    let y = v.rem_euclid(1.0) * ny as f64;
    let (cx, cy) = (x.floor() as i64, y.floor() as i64);
    let (mut f1, mut cell) = (f64::INFINITY, (0, 0));
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (i, j) = (cx + dx, cy + dy);
            let (wi, wj) = (i.rem_euclid(nx), j.rem_euclid(ny));
            let px = i as f64 + 0.5 + (n.rand(wi, wj, salt) - 0.5) * jitter;
            let py = j as f64 + 0.5 + (n.rand(wi, wj, salt + 1) - 0.5) * jitter;
            let d = ((x - px).powi(2) + (y - py).powi(2)).sqrt();
            if d < f1 {
                f1 = d;
                cell = (wi, wj);
            }
        }
    }
    Worley { f1, cell }
}

/// Small grains (sand, aggregate, pores): 1 inside a grain, fading at its rim, `density` of the cells
/// hold one, sized `size` (0..1 of a cell). Returns (coverage, the grain's random 0..1 tone).
fn grains(n: &Noise, u: f64, v: f64, cells: i64, density: f64, size: f64, salt: u32) -> (f64, f64) {
    let w = worley(n, u, v, cells, cells, 0.85, salt);
    let (i, j) = w.cell;
    if n.rand(i, j, salt + 2) > density {
        return (0.0, 0.0);
    }
    let s = size * (0.6 + 0.8 * n.rand(i, j, salt + 3));
    (1.0 - smoothstep(s * 0.55, s, w.f1), n.rand(i, j, salt + 4))
}

/// Rounded-box inside distance: positive inside, `rc` corner radius, given the four edge distances.
fn rounded(dx: f64, dy: f64, rc: f64) -> f64 {
    if dx < rc && dy < rc {
        rc - ((rc - dx).powi(2) + (rc - dy).powi(2)).sqrt()
    } else {
        dx.min(dy)
    }
}

/// Paint every pixel and build the surface: AO from the height's cavities (`ao`: (radius in
/// metres, gain) per scale), roughness as given. `tile` is the texture's size in metres.
fn paint(
    w: usize,
    h: usize,
    tile: f64,
    ao: &[(f64, f64)],
    grey: bool,
    f: impl Fn(f64, f64) -> Px + Sync,
) -> Surface {
    let px = par_map(w, h, |x, y| {
        f((x as f64 + 0.5) / w as f64, (y as f64 + 0.5) / h as f64)
    });
    let mut height = HeightMap::new(w, h);
    let mut img = vec![0u8; w * h * 3];
    let mut rough = vec![0.0f64; w * h];
    for (i, p) in px.iter().enumerate() {
        height.v[i] = p.h;
        rough[i] = p.r;
        for c in 0..3 {
            img[i * 3 + c] = to_u8(p.c[c]);
        }
    }
    height.stretch();
    let terms: Vec<(usize, f64)> = ao
        .iter()
        .map(|&(m, g)| (((m / tile) * w as f64).round().max(1.0) as usize, g))
        .collect();
    let ao = height.cavity_ao(&terms);
    Surface {
        height,
        albedo: Img {
            w,
            h,
            ch: 3,
            px: img,
        },
        rough: Some(rough),
        ao: Some(ao),
        grey_from_albedo: grey,
    }
}

fn grey(l: f64) -> [f64; 3] {
    [l, l, l]
}

/// Stamp a soft disc of radius `r` (pixels) into `buf`, keeping the maximum, wrapping.
fn stamp(buf: &mut [f64], w: usize, h: usize, x: f64, y: f64, r: f64, amp: f64) {
    let ri = r.ceil() as isize + 1;
    let (xi, yi) = (x.floor() as isize, y.floor() as isize);
    for dy in -ri..=ri {
        for dx in -ri..=ri {
            let (px, py) = (xi + dx, yi + dy);
            let d = ((px as f64 + 0.5 - x).powi(2) + (py as f64 + 0.5 - y).powi(2)).sqrt();
            let a = amp * (1.0 - smoothstep(r * 0.4, r + 0.6, d));
            let i = py.rem_euclid(h as isize) as usize * w + px.rem_euclid(w as isize) as usize;
            if a > buf[i] {
                buf[i] = a;
            }
        }
    }
}

/// One crack (a wandering stroke) and, optionally, where its branches start.
struct Stroke {
    x: f64,
    y: f64,
    a: f64,
    len: f64,
    wid: f64,
    salt: u32,
}

/// Draw cracks as wandering strokes running every which way, the odd one branching (in pixels,
/// wrapping); returns per pixel the crack's depth 0..1 and a wider soft halo (sealant, dirt) 0..1.
fn cracks(
    n: &Noise,
    w: usize,
    h: usize,
    count: usize,
    len_px: f64,
    width_px: f64,
    halo_px: f64,
) -> (Vec<f64>, Vec<f64>) {
    let mut depth = vec![0.0f64; w * h];
    let mut halo = vec![0.0f64; w * h];
    let free = |x: f64, y: f64, o: u32| n.perlin2(x, y, 1 << 20, 1 << 20, o);
    let mut todo: Vec<(Stroke, bool)> = (0..count)
        .map(|c| {
            let r = |k: i64| n.rand(c as i64, k, 911);
            let stroke = Stroke {
                x: r(0) * w as f64,
                y: r(1) * h as f64,
                a: r(2) * std::f64::consts::TAU,
                len: len_px * (0.4 + 1.2 * r(3) * r(3)),
                wid: width_px * (0.6 + 0.6 * r(4)),
                salt: c as u32 * 7 + 1,
            };
            (stroke, true)
        })
        .collect();
    let mut next = 1000u32;
    while let Some((s, branches)) = todo.pop() {
        let (mut x, mut y, mut a) = (s.x, s.y, s.a);
        let steps = (s.len / 0.5) as usize;
        let sf = s.salt as f64 * 7.31;
        for k in 0..steps {
            let t = k as f64 / steps as f64;
            let d = t * s.len;
            // The direction wanders slowly, with small kinks.
            a += free(d / 80.0 + sf, 0.5, 0) * 0.006 + free(d / 14.0 + sf, 5.5, 5) * 0.03;
            x += a.cos() * 0.5;
            y += a.sin() * 0.5;
            // Jagged at small scale: offset across the path.
            let j = free(d / 12.0 + sf, 3.5, 3) * 7.0 + free(d / 3.0 + sf, 4.5, 4) * 2.0;
            let (x, y) = (x - a.sin() * j, y + a.cos() * j);
            let taper = (t * 6.0).min(1.0) * ((1.0 - t) * 6.0).min(1.0);
            let wv = s.wid * (0.6 + 1.2 * (free(d / 15.0 + sf, 2.5, 2) + 0.5)) * taper;
            stamp(
                &mut depth,
                w,
                h,
                x,
                y,
                wv.max(0.35),
                sat(0.4 + wv / s.wid * 0.6),
            );
            stamp(&mut halo, w, h, x, y, halo_px * taper.max(0.3), 1.0);
            let r = |q: u32| n.rand(s.salt as i64, k as i64, 900 + q);
            if branches && r(0) < 0.0008 {
                let side = if r(1) < 0.5 { -1.0 } else { 1.0 };
                next += 1;
                todo.push((
                    Stroke {
                        x,
                        y,
                        a: a + side * (0.5 + 0.7 * r(2)),
                        len: s.len * (0.05 + 0.12 * r(3)),
                        wid: s.wid * 0.5,
                        salt: next,
                    },
                    false,
                ));
            }
        }
    }
    (depth, halo)
}

// --- wood -----------------------------------------------------------------------------------------

/// One board's log: where the pith runs relative to the face, so growth rings cut by the face show
/// as cathedral arches and straight stripes, and its knots.
struct Log {
    /// Pith position across the board (m, from its edge) and depth below the face (m).
    cy: f64,
    z0: f64,
    /// How fast the pith dives away from the face along the board (cathedrals), and from which end.
    slope: f64,
    from_end: bool,
    len: f64,
    /// Ring spacing (m).
    ring: f64,
    knots: Vec<(f64, f64, f64)>,
    tone: f64,
    salt: f64,
}

fn log_for(
    n: &Noise,
    a: i64,
    b: i64,
    width: f64,
    length: f64,
    knots: f64,
    ring: (f64, f64),
) -> Log {
    let r = |k: u32| n.rand(a, b, 300 + k);
    let mut ks = Vec::new();
    let count = if r(0) < knots { 1 } else { 0 } + if r(1) < knots * 0.35 { 1 } else { 0 };
    for k in 0..count {
        ks.push((
            length * (0.1 + 0.8 * r(10 + k * 3)),
            width * (0.15 + 0.7 * r(11 + k * 3)),
            0.006 + 0.010 * r(12 + k * 3),
        ));
    }
    Log {
        cy: width * (-0.4 + 1.8 * r(2)),
        z0: 0.012 + 0.07 * r(3),
        slope: 0.01 + 0.05 * r(4),
        from_end: r(8) < 0.5,
        len: length,
        ring: ring.0 + (ring.1 - ring.0) * r(5),
        knots: ks,
        tone: r(6),
        salt: r(7) * 500.0,
    }
}

/// The grain at (x along the board, y across), metres: (latewood 0..1, fibre streaks ~-0.5..0.5,
/// knot 0..1, knot ring darkness).
fn grain_at(n: &Noise, lg: &Log, x: f64, y: f64, px_m: f64) -> (f64, f64, f64) {
    let s = lg.salt;
    // Rings: distance from the pith, wobbling along the board.
    let wob = fbm_free(n, x * 1.5 + s, y * 8.0, 3) * 0.008
        + fbm_free(n, x * 5.0 + s, y * 20.0, 3) * 0.003;
    let along = if lg.from_end { lg.len - x } else { x };
    let z = lg.z0 + lg.slope * along + fbm_free(n, x * 0.8 + s * 2.0, 0.3, 2) * 0.012;
    let mut r = ((y - lg.cy).powi(2) + z.powi(2)).sqrt() + wob;
    let mut knot = 0.0f64;
    for &(kx, ky, kr) in &lg.knots {
        let (dx, dy) = ((x - kx) / 2.2, y - ky);
        let d = (dx * dx + dy * dy).sqrt();
        // The grain swirls round the knot.
        r += 0.03 * kr / (d + kr) * (1.0 - smoothstep(0.0, kr * 6.0, d)) * 3.0;
        let kd = ((x - kx).powi(2) / 3.0 + dy * dy).sqrt();
        knot = knot.max(1.0 - smoothstep(kr * 0.75, kr * 1.05, kd));
    }
    // Growth varies year to year: uneven ring widths.
    let q = r / lg.ring;
    let q = q + fbm_free(n, q * 0.35 + s, 7.7, 2) * 1.6;
    let t = q.rem_euclid(1.0);
    let mut late = smoothstep(0.55, 0.9, t) * (1.0 - smoothstep(0.95, 1.0, t));
    // Fade rings that are too fine for the pixels (no aliasing).
    let dr = {
        let r2 = ((y + px_m - lg.cy).powi(2) + z.powi(2)).sqrt();
        ((r2 - r + wob).abs() / px_m).max(0.02)
    };
    let px_per_ring = lg.ring / (dr * px_m);
    late = lerp(0.35, late, smoothstep(2.0, 4.0, px_per_ring));
    let fibre =
        fbm_free(n, x * 6.0 + s, y * 900.0, 3) + 0.5 * fbm_free(n, x * 40.0 + s, y * 2500.0, 2);
    (late, fibre, knot)
}

// --- the sets -------------------------------------------------------------------------------------

/// Tile sizes in metres (1 / the manifest's uvScale).
const TILE_BRICK: f64 = 1.0;

const TILE_STONE: f64 = 1.5;

const TILE_FLOOR_TILE: f64 = 2.0;

const TILE_HEX: f64 = 1.0 / 0.7;

const TILE_FLOOR_WOOD: f64 = 1.0 / 0.53;

const TILE_LAMINATE: f64 = 1.0 / 0.55;

const TILE_SIDING: f64 = 2.5;

const TILE_ROOF: f64 = 1.5;

const TILE_PLASTER: f64 = 2.0;

const TILE_SIDEWALK: f64 = 1.0 / 0.556;

const TILE_ROAD: f64 = 5.0;

const TILE_LEATHER: f64 = 0.5;

/// The floor coverings' tiles (m), each a whole number of its pattern's repeats.
const TILE_PARQUET: f64 = 1.5;

const TILE_CHECKER: f64 = 1.8;

const TILE_HEX_MOSAIC: f64 = 1.5;

/// The garden paths' and drives' own (`garden.rs`): broom-finished concrete, flagstones and pea gravel (m).
const TILE_WALK: f64 = 2.0;

const TILE_FLAGSTONE: f64 = 2.4;

const TILE_GRAVEL: f64 = 1.2;

/// Flagstones across a tile (about 48 cm each), and gravel stones' cells across one (15 mm).
const FLAGSTONE_CELLS: i64 = 5;

const GRAVEL_CELLS: i64 = 80;

const TILE_TERRAZZO: f64 = 2.0;

const TILE_MARBLE: f64 = 2.4;

const TILE_CORK: f64 = 1.2;

const TILE_LINO: f64 = 2.0;

const TILE_CONCRETE: f64 = 2.4;

const TILE_PAVERS: f64 = 5.0 * PAVER.0 * std::f64::consts::SQRT_2;

const TILE_ENCAUSTIC: f64 = 1.36;

const TILE_SISAL: f64 = 1.0;

/// A set's tile in metres (the size it is drawn at: 1 / the manifest's `uvScale`), or `None` for one not made here.
#[cfg(test)]
pub fn tile(name: &str) -> Option<f64> {
    Some(match name {
        "brick" => TILE_BRICK,
        "stone_foundation" => TILE_STONE,
        "floor_tile" => TILE_FLOOR_TILE,
        "path_hexpaving" => TILE_HEX,
        "path_concrete" => TILE_WALK,
        "path_flagstone" => TILE_FLAGSTONE,
        "path_gravel" => TILE_GRAVEL,
        "floor_wood" => TILE_FLOOR_WOOD,
        "floor_laminate" => TILE_LAMINATE,
        "siding_wood" => TILE_SIDING,
        "roof_shingles" => TILE_ROOF,
        "wall_plaster" => TILE_PLASTER,
        "sidewalk" => TILE_SIDEWALK,
        "road_asphalt" => TILE_ROAD,
        "leather" => TILE_LEATHER,
        "floor_parquet" => TILE_PARQUET,
        "floor_checker" => TILE_CHECKER,
        "floor_hex" => TILE_HEX_MOSAIC,
        "floor_terrazzo" => TILE_TERRAZZO,
        "floor_marble" => TILE_MARBLE,
        "floor_cork" => TILE_CORK,
        "floor_lino" => TILE_LINO,
        "floor_concrete" => TILE_CONCRETE,
        "floor_pavers" => TILE_PAVERS,
        "floor_encaustic" => TILE_ENCAUSTIC,
        "floor_sisal" => TILE_SISAL,
        _ => return None,
    })
}

/// Brick courses and bricks along a tile, and half a mortar joint (m).
const BRICK_COURSES: i64 = 16;

const BRICK_ALONG: i64 = 5;

const BRICK_HALF_JOINT: f64 = 0.005;

/// Floor tiles across a tile, and half a grout joint (m).
const FLOOR_TILES: i64 = 6;

const FLOOR_TILE_HALF_JOINT: f64 = 0.0012;

/// Hex pavers across and rows of them on a tile, and half the joint between two (m).
const HEX_ACROSS: i64 = 7;

const HEX_ROWS: i64 = 8;

const HEX_HALF_JOINT: f64 = 0.0045;

/// Rows of boards across a tile: floorboards, laminate planks, siding boards, roof courses.
const FLOOR_WOOD_ROWS: i64 = 9;

const LAMINATE_ROWS: i64 = 8;

const SIDING_ROWS: i64 = 12;

const ROOF_COURSES: i64 = 11;

/// End joints in a row of laminate (one: each plank as long as the tile).
const LAMINATE_JOINTS: usize = 1;

/// Sidewalk slabs along and across a tile.
const SIDEWALK_SLABS: (i64, i64) = (3, 6);

/// Parquet blocks (m, joints included): three herringbone repeats (a block's length times √2) along the tile,
/// fifteen (its width times √2) up it.
const PARQUET_BLOCK: (f64, f64) = (
    TILE_PARQUET / (3.0 * std::f64::consts::SQRT_2),
    TILE_PARQUET / (15.0 * std::f64::consts::SQRT_2),
);

/// Chequerboard squares across a tile (30 cm), and half the grout joint (m).
const CHECKER_SQUARES: i64 = 6;

const CHECKER_HALF_JOINT: f64 = 0.001;

/// Mosaic hexagons across and rows of them on a tile (58 mm across the flats; rows must be even to wrap,
/// 0.1 % stretched), and half the grout joint (m).
const HEX_MOSAIC: (i64, i64) = (26, 30);

const HEX_MOSAIC_HALF_JOINT: f64 = 0.00125;

/// The terrazzo's bays between brass divider strips (m), and half a strip.
const TERRAZZO_BAY: f64 = 1.0;

const TERRAZZO_HALF_STRIP: f64 = 0.002;

/// Marble slabs across a tile (60 cm), and half the hairline joint (m).
const MARBLE_SLABS: i64 = 4;

const MARBLE_HALF_JOINT: f64 = 0.0005;

/// Cork tiles across a tile (30 cm), and half the butt joint (m).
const CORK_TILES: i64 = 4;

const CORK_HALF_JOINT: f64 = 0.0003;

/// Concrete's saw cuts: every 1.2 m, half a cut 1.5 mm.
const CONCRETE_JOINTS: f64 = 1.2;

const CONCRETE_HALF_CUT: f64 = 0.0015;

/// Brick pavers (m, joints included: 190 x 90 mm bricks) and half the mortar joint.
const PAVER: (f64, f64) = (0.2, 0.1);

const PAVER_HALF_JOINT: f64 = 0.005;

/// The geometric hall tiles' square (m: two make the pattern's repeat), its stars' centre square's half,
/// and half the joint between pieces.
const ENCAUSTIC_CELL: f64 = 0.17;

const ENCAUSTIC_CENTRE: f64 = 0.027;

/// Half the dot where the ground's four small squares meet (m).
const ENCAUSTIC_DOT: f64 = 0.013;

const ENCAUSTIC_HALF_JOINT: f64 = 0.001;

/// Sisal basketweave blocks across a tile (25 mm), and strands to a block.
const SISAL_BLOCKS: i64 = 40;

const SISAL_STRANDS: i64 = 4;

/// The sizes of a set's pattern that real ones are held to (`docs/review/reference.json` `surfaces`, checked
/// by `cargo review sizes`), as (name, size in UV units: a share of the tile); none for a set without one.
/// The size in metres is the share over the manifest's `uvScale`.
pub fn features(name: &str) -> Vec<(&'static str, f64)> {
    let share = |metres: f64, tile: f64| metres / tile;
    match name {
        "brick" => vec![
            (
                "length",
                1.0 / BRICK_ALONG as f64 - share(2.0 * BRICK_HALF_JOINT, TILE_BRICK),
            ),
            (
                "height",
                1.0 / BRICK_COURSES as f64 - share(2.0 * BRICK_HALF_JOINT, TILE_BRICK),
            ),
            ("joint", share(2.0 * BRICK_HALF_JOINT, TILE_BRICK)),
        ],
        "floor_tile" => vec![
            (
                "size",
                1.0 / FLOOR_TILES as f64 - share(2.0 * FLOOR_TILE_HALF_JOINT, TILE_FLOOR_TILE),
            ),
            ("joint", share(2.0 * FLOOR_TILE_HALF_JOINT, TILE_FLOOR_TILE)),
        ],
        // Pointy-top hexes: across the flats is the spacing along a row, less the joint.
        "path_hexpaving" => vec![
            (
                "across",
                1.0 / HEX_ACROSS as f64 - share(2.0 * HEX_HALF_JOINT, TILE_HEX),
            ),
            ("joint", share(2.0 * HEX_HALF_JOINT, TILE_HEX)),
        ],
        "floor_wood" => vec![("width", 1.0 / FLOOR_WOOD_ROWS as f64)],
        "floor_laminate" => vec![
            ("width", 1.0 / LAMINATE_ROWS as f64),
            ("length", 1.0 / LAMINATE_JOINTS as f64),
        ],
        "siding_wood" => vec![("exposure", 1.0 / SIDING_ROWS as f64)],
        "roof_shingles" => vec![("exposure", 1.0 / ROOF_COURSES as f64)],
        "sidewalk" => vec![
            ("length", 1.0 / SIDEWALK_SLABS.0 as f64),
            ("width", 1.0 / SIDEWALK_SLABS.1 as f64),
        ],
        _ => Vec::new(),
    }
}

/// A board within staggered rows: which row and segment, and the local position (m along, m across).
struct Board {
    row: i64,
    seg: i64,
    x: f64,
    y: f64,
    len: f64,
    /// Distance to the nearest end joint (m).
    end: f64,
}

/// Boards in `rows` rows across the tile's v, ends staggered with a random phase per row so no joint
/// lines up with the tile's edge. `cuts` joints per row, `len_min` the shortest board (fraction).
#[allow(clippy::too_many_arguments)]
fn board_at(
    n: &Noise,
    u: f64,
    v: f64,
    rows: i64,
    cuts: usize,
    len_min: f64,
    t: f64,
    salt: u32,
) -> Board {
    let yy = v.rem_euclid(1.0) * rows as f64;
    let row = yy.floor() as i64;
    let wr = row.rem_euclid(rows);
    let mut cs = [0.0f64; 16];
    let cuts = cuts.clamp(1, 15);
    // Evenly spaced joints, jittered, then phase-shifted.
    for (k, c) in cs.iter_mut().enumerate().take(cuts) {
        let j = (n.rand(wr, k as i64, salt) - 0.5) * (1.0 - len_min * cuts as f64).max(0.0)
            / cuts as f64;
        *c = (k as f64 + 0.5) / cuts as f64 + j;
    }
    let phase = n.rand(wr, 99, salt);
    let uu = (u + phase).rem_euclid(1.0);
    // Segment k runs from cs[k] to cs[k+1] (the last wraps to cs[0] + 1).
    let mut seg = cuts - 1;
    for k in 0..cuts {
        if uu >= cs[k] && (k + 1 == cuts || uu < cs[k + 1]) {
            seg = k;
        }
    }
    let start = cs[seg];
    let end = if seg + 1 == cuts {
        cs[0] + 1.0
    } else {
        cs[seg + 1]
    };
    let x = (uu - start).rem_euclid(1.0);
    let len = end - start;
    Board {
        row: wr,
        seg: seg as i64,
        x: x * t,
        y: (yy - row as f64) * t / rows as f64,
        len: len * t,
        end: x.min(len - x) * t,
    }
}

/// A set's variants (`cargo tex synth floor_wood.calm`, into `textures/floor_wood.calm`): candidates for a look
/// sheet (tools/bench/play-variants.json), on the set's own tile and seed, so the boards lie where the set's do.
/// Not in git and not in a build: the set itself is the variant `""`.
pub fn variants(name: &str) -> &'static [&'static str] {
    match name {
        "floor_wood" => &["", "pine", "calm", "wide", "oak"],
        _ => &[""],
    }
}

/// What the floorboards' variants vary.
struct Boards {
    /// Rows of boards across the tile.
    rows: i64,
    /// A board's own brightness: the darkest, and the range above it.
    bright: (f64, f64),
    /// The share drawn of the grain's figure (late wood, fibre, streaks) and of the knots.
    figure: f64,
    knots: f64,
    /// Early and late wood, each from a light board's to a dark one's.
    early: ([f64; 3], [f64; 3]),
    late: ([f64; 3], [f64; 3]),
}

fn boards(variant: &str) -> Boards {
    // The set: nine boards across, 21 cm each, the knots a quarter as dark as a pine board's are and the figure
    // half, the boards near each other in tone, so that from the play camera (a pixel 4 to 16 mm) a floor is
    // planks and not dots and streaks (2026-11-18). Its brightness is set so the tile's mean is the narrow
    // knotty boards' it replaced: a covering's tint (honey, walnut, whitewash) gives the tone it gave.
    let set = Boards {
        rows: FLOOR_WOOD_ROWS,
        bright: (0.946, 0.058),
        figure: 0.5,
        knots: 0.25,
        early: (rgb(212, 164, 106), rgb(194, 144, 88)),
        late: (rgb(178, 122, 70), rgb(160, 106, 58)),
    };
    // The boards before 2026-11-18: 13.5 cm, every knot and all the figure.
    let pine = |b: Boards| Boards {
        rows: 14,
        bright: (0.95, 0.10),
        figure: 1.0,
        knots: 1.0,
        ..b
    };
    match variant {
        "pine" => pine(set),
        // Calm at the old width; wide with the old knots and figure.
        "calm" => Boards { rows: 14, ..set },
        "wide" => Boards {
            rows: FLOOR_WOOD_ROWS,
            ..pine(set)
        },
        // Oak's own tone: less orange.
        "oak" => Boards {
            bright: (0.97, 0.06),
            early: (rgb(196, 162, 120), rgb(180, 147, 107)),
            late: (rgb(164, 128, 90), rgb(148, 114, 79)),
            ..set
        },
        _ => set,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tex::synth::SETS;
    use crate::tex::synth::standard::materials;

    /// A set draws its joints, boards and grains in metres at its tile: the manifest shows it at that size.
    #[test]
    fn the_manifest_shows_each_set_at_its_tile() {
        let mut seen = 0;
        for m in materials() {
            if let Some(t) = tile(m.set.name) {
                assert!(
                    (t * m.uv_scale - 1.0).abs() < 0.001,
                    "{}: {} is drawn at {t} m a tile, shown at {} m",
                    m.key,
                    m.set.name,
                    1.0 / m.uv_scale
                );
                seen += 1;
            }
        }
        assert!(seen >= 12, "{seen} materials draw the sets made here");
    }

    #[test]
    fn features_are_a_share_of_the_tile() {
        for set in SETS {
            for (what, uv) in features(set.name) {
                assert!(uv > 0.0 && uv <= 1.0, "{} {what}: {uv}", set.name);
            }
        }
        // The brick: 190 x 52.5 mm with 10 mm joints at its 1 m tile.
        let brick: Vec<f64> = features("brick").iter().map(|f| f.1).collect();
        assert!((brick[0] - 0.19).abs() < 1e-12 && (brick[1] - 0.0525).abs() < 1e-12);
        assert!((brick[2] - 0.01).abs() < 1e-12);
    }
}

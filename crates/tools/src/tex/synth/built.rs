//! The built surfaces: brick, stonework, tiles, pavers, boards, shingles, plaster, concrete and asphalt.
//!
//! Each set is drawn in real units: the tile's size in metres comes from the manifest's `uvScale`
//! (`TILE`), so a 9 mm mortar joint is 9 mm whatever the texture size. A generator returns height,
//! colour and roughness per pixel (`Px`); ambient occlusion comes from the height's cavities at the
//! scales the set picks. On walls and roofs the game maps image-down to world-up (v grows upwards and
//! the texture is not flipped), so lap boards and shingles are drawn with their thick butt at the top
//! of their course in the image.

use super::{HeightMap, Img, Noise, Surface, lerp, mix, mul, par_map, rgb, smoothstep, to_u8};

/// The sets made here.
pub fn generate(name: &str, n: &Noise, size: usize) -> Option<Surface> {
    Some(match name {
        "brick" => brick(n, size),
        "stone_foundation" => stone(n, size),
        "floor_tile" => floor_tile(n, size),
        "path_hexpaving" => hexpaving(n, size),
        "floor_wood" => floor_wood(n, size),
        "floor_laminate" => laminate(n, size),
        "siding_wood" => siding(n, size),
        "roof_shingles" => roof(n, size),
        "wall_plaster" => plaster(n, size),
        "sidewalk" => sidewalk(n, size),
        "road_asphalt" => asphalt(n, size),
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

/// Common brick in running bond: 16 courses and 5 bricks a metre, 9 mm recessed mortar.
fn brick(n: &Noise, size: usize) -> Surface {
    let (rows, cols) = (16i64, 5i64);
    let t = TILE_BRICK;
    let (bw, bh) = (t / cols as f64, t / rows as f64);
    let palette = [
        rgb(138, 72, 56),
        rgb(124, 64, 52),
        rgb(110, 58, 50),
        rgb(98, 62, 56),
        rgb(134, 88, 70),
        rgb(118, 74, 60),
        rgb(80, 50, 46),
        rgb(146, 96, 76),
    ];
    let nw = n.layer(1);
    let nc = n.layer(2);
    let ng = n.layer(3);
    let nm = n.layer(4);
    paint(
        size,
        size,
        t,
        &[(0.004, 1.2), (0.02, 0.5)],
        false,
        |u, v| {
            // Slightly wavy courses and perpends.
            let wu = u + fbm(&nw, u, v, 6, 6, 3) * 0.0016 / t;
            let wv = v + fbm(&nw.layer(9), u, v, 6, 6, 3) * 0.0012 / t;
            let row = (wv * rows as f64).floor() as i64;
            let off = if row.rem_euclid(2) == 0 { 0.0 } else { 0.5 }
                + (n.rand(row.rem_euclid(rows), 0, 5) - 0.5) * 0.06;
            let xx = wu * cols as f64 + off;
            let col = xx.floor() as i64;
            let (wi, wj) = (col.rem_euclid(cols), row.rem_euclid(rows));
            let r = |k: u32| n.rand(wi, wj, 20 + k);
            let x = (xx - col as f64) * bw;
            let y = (wv * rows as f64 - row as f64) * bh;
            let joint = 0.005;
            let (l, rr) = (joint + (r(0) - 0.5) * 0.002, joint + (r(1) - 0.5) * 0.002);
            let (tp, bt) = (joint + (r(2) - 0.5) * 0.0015, joint + (r(3) - 0.5) * 0.0015);
            let dx = (x - l).min(bw - rr - x);
            let dy = (y - tp).min(bh - bt - y);
            let chip = fbm(&nc, u, v, 90, 90, 3) * 0.004
                + fbm(&nc.layer(2), u, v, 12, 12, 2) * 0.004
                - smoothstep(0.08, 0.22, fbm(&nc.layer(1), u, v, 24, 24, 3)) * 0.006;
            let d = rounded(dx, dy, 0.005) + chip.min(0.001);
            let face = smoothstep(-0.0004, 0.0035, d);
            // Brick face: a slight random tilt and bulge, sandy texture, a few pits.
            let tilt = (x / bw - 0.5) * (r(4) - 0.5) * 0.25 + (y / bh - 0.5) * (r(5) - 0.5) * 0.25;
            let tex = fbm(&ng, u, v, 160, 160, 4) * 0.35 + fbm(&ng.layer(1), u, v, 40, 40, 3) * 0.2;
            let (pit, _) = grains(&ng.layer(2), u, v, 220, 0.05, 0.45, 7);
            let bh_ = 1.0 + tilt + tex - 0.12 * pit;
            // Mortar: recessed, sandy.
            let sand =
                fbm(&nm, u, v, 300, 300, 2) * 0.35 + fbm(&nm.layer(1), u, v, 60, 60, 3) * 0.2;
            let mh = 0.25 + sand;
            let h = lerp(mh, bh_, face);

            let base = palette[(r(6) * palette.len() as f64) as usize % palette.len()];
            let base = mul(base, 0.86 + 0.26 * r(7));
            // Kiln flashing: one end or face darker.
            let flash =
                sat(0.5
                    + (if r(8) < 0.5 { x / bw } else { 1.0 - x / bw } - 0.5) * 1.6 * (r(9) - 0.3));
            let mottle = fbm(&nc.layer(3), u, v, 30, 30, 4);
            let streak = fbm(&nc.layer(8), u, v, 12, 90, 3);
            let speck = fbm(&nc.layer(4), u, v, 200, 200, 2);
            let mut c = mul(
                base,
                (1.0 + 0.40 * mottle + 0.15 * streak + 0.18 * speck) * (0.92 + 0.12 * flash),
            );
            // Soot and lime bloom in soft patches.
            let soot = smoothstep(
                0.05,
                0.22,
                fbm(&nc.layer(6), u, v, 20, 20, 4) + (r(10) - 0.5) * 0.2,
            );
            c = mix(c, mul(c, 0.62), soot * 0.6);
            let (dark, _) = grains(&nc.layer(5), u, v, 140, 0.05, 0.4, 11);
            c = mix(c, rgb(52, 34, 30), dark * 0.4);
            c = mix(c, mul(c, 0.8), pit * 0.4);
            // Worn arrises read a little darker; fine sand in the fired skin.
            c = mul(
                c,
                (0.9 + 0.1 * smoothstep(0.0, 0.006, d))
                    * (1.0 + 0.12 * fbm(&ng.layer(4), u, v, 420, 420, 1)),
            );
            let lum = 0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2];
            c = mul(mix(c, [lum; 3], 0.22), 0.95);
            // Mortar smeared onto the arris here and there.
            let smear = (1.0 - smoothstep(0.0, 0.004, d))
                * smoothstep(0.05, 0.2, fbm(&nm.layer(2), u, v, 50, 50, 3));
            c = mix(c, rgb(170, 162, 150), smear * 0.6);
            // Soot and weathering across the wall (gentle, so the repeat does not show).
            let grime = fbm(&nc.layer(7), u, v, 2, 3, 4);
            c = mul(c, 1.0 + 0.10 * grime);
            let mortar = mul(
                rgb(124, 121, 115),
                1.0 + 0.5 * fbm(&nm.layer(3), u, v, 260, 260, 2)
                    + 0.2 * fbm(&nm.layer(4), u, v, 20, 20, 3),
            );
            let (mgrain, mt) = grains(&nm.layer(5), u, v, 300, 0.5, 0.5, 17);
            let mortar = mix(
                mortar,
                mul(rgb(160, 156, 148), 0.6 + 0.8 * mt),
                mgrain * 0.7,
            );
            let col = mix(mortar, c, face);
            let rough = lerp(0.88, 0.76 + 0.1 * speck + 0.08 * pit, face);
            Px {
                h,
                c: col,
                r: rough,
            }
        },
    )
}

/// Coursed rubble stone: thin split stones in uneven courses, dark recessed joints.
fn stone(n: &Noise, size: usize) -> Surface {
    let t = TILE_STONE;
    // Course heights, summing to the tile.
    const NC: usize = 25;
    let mut ys = [0.0f64; NC + 1];
    let mut acc = 0.0;
    for k in 0..NC {
        acc += 0.5
            + n.rand(k as i64, 0, 40)
                * (if n.rand(k as i64, 1, 40) < 0.2 {
                    1.4
                } else {
                    0.8
                });
        ys[k + 1] = acc;
    }
    for y in &mut ys {
        *y /= acc;
    }
    let palette = [
        rgb(170, 148, 116),
        rgb(150, 130, 104),
        rgb(128, 110, 88),
        rgb(182, 160, 124),
        rgb(140, 128, 112),
        rgb(158, 128, 94),
        rgb(112, 96, 80),
        rgb(166, 150, 128),
        rgb(146, 122, 96),
    ];
    let nw = n.layer(1);
    let ns = n.layer(2);
    let nc = n.layer(3);
    paint(
        size,
        size,
        t,
        &[(0.006, 1.6), (0.03, 0.7)],
        false,
        move |u, v| {
            let wu = u + fbm(&nw, u, v, 10, 10, 3) * 0.012 / t;
            let wv = v + fbm(&nw.layer(1), u, v, 10, 10, 3) * 0.008 / t;
            let wvv = wv.rem_euclid(1.0);
            let mut k = 0;
            while k + 1 < NC && wvv >= ys[k + 1] {
                k += 1;
            }
            let (y0, y1) = (ys[k], ys[k + 1]);
            let ch = (y1 - y0) * t;
            let y = (wvv - y0) * t;
            let ki = k as i64;
            // Stones along the course.
            let phase = n.rand(ki, 2, 41);
            // Perpends lean a little, the same way along a course.
            let slant = (n.rand(ki, 1, 41) - 0.5) * 0.8;
            let uu = (wu + phase + (y / ch - 0.5) * ch / t * slant).rem_euclid(1.0);
            let mut xs = [0.0f64; 12];
            let mut nx = 0;
            let mut a = 0.0;
            while nx < 11 {
                a += 0.3 + n.rand(ki, nx as i64 + 3, 42) * 0.9;
                nx += 1;
                xs[nx] = a;
                if a > 6.0 {
                    break;
                }
            }
            for x in xs.iter_mut().take(nx + 1) {
                *x /= a;
            }
            let mut s = 0;
            while s + 1 < nx && uu >= xs[s + 1] {
                s += 1;
            }
            let si = s as i64;
            let r = |q: u32| n.rand(ki, si, 50 + q);
            let sw = (xs[s + 1] - xs[s]) * t;
            let x = (uu - xs[s]) * t;
            // Some stones fall short of the course (mortar or a thin filler shows), some are split in two.
            let short = if r(14) < 0.35 { r(15) * 0.35 * ch } else { 0.0 };
            let (top_in, bot_in) = if r(16) < 0.5 {
                (short, 0.0)
            } else {
                (0.0, short)
            };
            let split = r(17) < 0.25 && ch > 0.05;
            let sy = 0.35 + 0.3 * r(18);
            let (y, ch, sub) = if split {
                if y < ch * sy {
                    (y, ch * sy, 0)
                } else {
                    (y - ch * sy, ch * (1.0 - sy), 1)
                }
            } else {
                (y, ch, 0)
            };
            let r = |q: u32| n.rand(ki, si * 2 + sub, 50 + q);
            let gap = 0.0015 + 0.003 * r(0);
            let (gl, gr) = (gap * (0.5 + r(1)), gap * (0.5 + r(2)));
            let (gt, gb) = (
                gap * (0.4 + 0.8 * r(3)) + if sub == 0 { top_in } else { 0.0 },
                gap * (0.4 + 0.8 * r(4)) + if split && sub == 0 { 0.0 } else { bot_in },
            );
            let dx = (x - gl).min(sw - gr - x);
            let dy = (y - gt).min(ch - gb - y);
            let rc = 0.003 + 0.008 * r(5);
            // Broken outlines: fine roughness plus larger bites out of the edges.
            let bite = smoothstep(0.15, 0.3, fbm(&ns.layer(5), u, v, 40, 40, 3)) * 0.008;
            let angular = fbm(&ns, u, v, 70, 70, 3).abs() - 0.08;
            let edge_noise = angular * 0.02 + fbm(&ns.layer(1), u, v, 20, 20, 2) * 0.006 - bite;
            let d = rounded(dx, dy, rc.min(ch * 0.45)) + edge_noise;
            let face = smoothstep(0.0, 0.003, d);
            let round = smoothstep(0.0, 0.008, d);
            // Split face: fine bedding striations, ledges, a random tilt and proud stones.
            let (lx, ly) = (x + r(6) * 10.0, y + r(7) * 10.0);
            let strata = fbm_free(&ns.layer(2), lx * 4.0, ly * 28.0, 4);
            let ledges = (strata * 6.0).floor() / 6.0 * 0.5 + strata * 0.5;
            let scratch = fbm_free(&ns.layer(6), lx * 6.0, ly * 160.0, 3);
            let bumps = fbm(&ns.layer(3), u, v, 50, 50, 4) * 0.3
                + fbm(&ns.layer(4), u, v, 200, 200, 2) * 0.1;
            let tilt = (x / sw - 0.5) * (r(8) - 0.5) * 0.4 + (y / ch - 0.5) * (r(9) - 0.5) * 0.4;
            let proud = r(10) * 0.35;
            let sh = 0.6 + 0.4 * round + ledges * 0.6 + 0.18 * scratch + bumps + tilt + proud;
            // Joints: earthy mortar, raked back, rough.
            let mortar_n = fbm(&nc, u, v, 150, 150, 2);
            let gh = 0.25 + mortar_n * 0.3 + 0.1 * fbm(&nc.layer(7), u, v, 30, 30, 3);
            let h = lerp(gh, sh, face);

            let base = palette[(r(11) * palette.len() as f64) as usize % palette.len()];
            let mut c = mul(base, 0.9 + 0.18 * r(12));
            let band = fbm_free(&nc.layer(1), lx * 3.0, ly * 40.0, 3);
            c = mul(c, 1.0 + 0.35 * band + 0.3 * scratch);
            // Iron staining and a second mineral hue.
            let rust = smoothstep(
                0.04,
                0.2,
                fbm(&nc.layer(2), u, v, 14, 14, 4) + (r(13) - 0.5) * 0.3,
            );
            c = mix(c, rgb(150, 96, 56), rust * 0.35);
            let greyish = smoothstep(
                0.05,
                0.2,
                fbm(&nc.layer(3), u, v, 9, 9, 3) + (r(19) - 0.5) * 0.2,
            );
            c = mix(c, rgb(128, 132, 124), greyish * 0.4);
            let fine = fbm(&nc.layer(4), u, v, 300, 300, 2);
            c = mul(c, 1.0 + 0.35 * fine);
            // Fresh, lighter broken edges; lichen; dark weathering in hollows.
            c = mix(
                c,
                mul(c, 1.12),
                (1.0 - smoothstep(0.0, 0.008, d)) * face * 0.6,
            );
            let (lich, lt) = grains(&nc.layer(5), u, v, 90, 0.05, 0.6, 19);
            c = mix(
                c,
                if lt < 0.5 {
                    rgb(196, 192, 170)
                } else {
                    rgb(70, 72, 58)
                },
                lich * 0.3,
            );
            c = mul(c, 1.0 + (bumps.min(0.0)) * 0.6);
            let gapc = mul(
                rgb(150, 122, 88),
                0.75 + 0.6 * mortar_n + 0.2 * fbm(&nc.layer(6), u, v, 20, 20, 3),
            );
            let gapc = mix(gapc, rgb(64, 52, 40), smoothstep(-0.004, 0.0, d) * 0.8);
            let col = mix(gapc, c, face);
            let rough = lerp(0.9, 0.68 + 0.1 * fine + 0.08 * scratch, face);
            Px {
                h,
                c: col,
                r: rough,
            }
        },
    )
}

/// Honed travertine floor tiles, 33 cm, thin grout.
fn floor_tile(n: &Noise, size: usize) -> Surface {
    let t = TILE_FLOOR_TILE;
    let k = 6i64;
    let s = t / k as f64;
    let nc = n.layer(1);
    let np = n.layer(2);
    paint(
        size,
        size,
        t,
        &[(0.003, 2.0), (0.02, 0.4)],
        false,
        |u, v| {
            let (ix, iy) = ((u * k as f64).floor() as i64, (v * k as f64).floor() as i64);
            let (wi, wj) = (ix.rem_euclid(k), iy.rem_euclid(k));
            let r = |q: u32| n.rand(wi, wj, 60 + q);
            let x = (u * k as f64 - ix as f64) * s;
            let y = (v * k as f64 - iy as f64) * s;
            let g = 0.0012;
            let d = rounded((x - g).min(s - g - x), (y - g).min(s - g - y), 0.0015);
            let face = smoothstep(0.0, 0.0015, d);
            // Bedding: each tile its own vein direction and clouds.
            let ang = (r(0) - 0.5) * 0.6
                + if r(1) < 0.5 {
                    0.0
                } else {
                    std::f64::consts::FRAC_PI_2
                };
            let (ca, sa) = (ang.cos(), ang.sin());
            let (lx, ly) = (x * ca + y * sa + r(2) * 7.0, -x * sa + y * ca + r(3) * 7.0);
            let warp = fbm_free(&nc, lx * 4.0, ly * 4.0, 3) * 0.04;
            let bands = fbm_free(&nc.layer(1), lx * 2.0, (ly + warp) * 30.0, 4);
            let clouds = fbm_free(&nc.layer(2), lx * 5.0, ly * 5.0, 4);
            let base = mix(rgb(230, 222, 214), rgb(214, 204, 194), r(4));
            let mut c = mul(
                base,
                1.0 + 0.06 * bands + 0.10 * clouds + (r(5) - 0.5) * 0.10,
            );
            // Grey cloudy shading, stronger on some tiles.
            let shade = smoothstep(0.0, 0.25, clouds + (r(9) - 0.5) * 0.25);
            c = mix(c, rgb(176, 170, 164), shade * 0.35);
            let vein = 1.0 - smoothstep(0.0, 0.012, (bands - 0.08).abs());
            c = mix(c, rgb(196, 184, 170), vein * 0.12);
            // Travertine holes: few, irregular, elongated along the bedding.
            let pore_n = fbm_free(&np, lx * 22.0, (ly + warp) * 70.0, 4) + clouds * 0.2;
            let pore = smoothstep(0.2, 0.26, pore_n);
            // Rust-red blotches and specks in clusters on some tiles.
            let cluster = smoothstep(0.04, 0.18, fbm_free(&np.layer(5), lx * 6.0, ly * 6.0, 3))
                * if r(8) < 0.6 { 1.0 } else { 0.0 };
            let blot = smoothstep(0.05, 0.16, fbm_free(&np.layer(7), lx * 40.0, ly * 40.0, 3));
            let (sp, st) = grains(&np.layer(6), u, v, 300, 0.45, 0.5, 53);
            let rust = mix(rgb(156, 96, 76), rgb(120, 80, 66), st);
            c = mix(c, rust, (sp * 0.8 + blot * 0.6).min(1.0) * cluster);
            let halo = smoothstep(0.12, 0.22, pore_n);
            c = mix(c, rgb(186, 150, 124), halo * 0.3);
            c = mix(c, rgb(120, 104, 90), pore * 0.8);
            let (dot, _) = grains(&np.layer(8), u, v, 240, 0.025, 0.3, 59);
            c = mix(c, rgb(130, 112, 100), dot * 0.6);
            c = mul(
                c,
                1.0 + 0.05 * fbm(&np.layer(9), u, v, 450, 450, 1)
                    + 0.03 * fbm(&np.layer(10), u, v, 120, 120, 2),
            );
            let lippage = (x / s - 0.5) * (r(6) - 0.5) * 0.06 + (y / s - 0.5) * (r(7) - 0.5) * 0.06;
            let micro = fbm(&np.layer(2), u, v, 400, 400, 2) * 0.03;
            let h = lerp(
                0.3 + fbm(&np.layer(3), u, v, 300, 300, 2) * 0.2,
                1.0 + lippage + micro - 0.25 * pore,
                face,
            );
            let grout = mul(
                rgb(150, 144, 134),
                1.0 + 0.3 * fbm(&np.layer(4), u, v, 200, 200, 2),
            );
            let col = mix(grout, c, face);
            let rough = lerp(
                0.9,
                0.48 + (r(10) - 0.5) * 0.12 + 0.12 * clouds + 0.4 * pore,
                face,
            );
            Px {
                h,
                c: col,
                r: rough,
            }
        },
    )
}

/// Hexagonal concrete pavers (20 cm), sand-filled joints with dirt.
fn hexpaving(n: &Noise, size: usize) -> Surface {
    let t = TILE_HEX;
    // Pointy-top hexes: 7 across, 8 rows (rows must be even to wrap; 1% stretch).
    let (cols, rows) = (7i64, 8i64);
    let (cw, rh) = (t / cols as f64, t / rows as f64);
    let nd = n.layer(1);
    let ng = n.layer(2);
    paint(
        size,
        size,
        t,
        &[(0.005, 0.8), (0.03, 0.3)],
        false,
        |u, v| {
            let (x, y) = (u * t, v * t);
            // Nearest lattice centre.
            let cj = (y / rh).floor() as i64;
            let (mut best, mut cell, mut lp) = (f64::INFINITY, (0, 0), (0.0, 0.0));
            for dj in -1..=1 {
                let j = cj + dj;
                let off = if j.rem_euclid(2) == 0 { 0.0 } else { 0.5 };
                let ci = (x / cw - off).floor() as i64;
                for di in -1..=1 {
                    let i = ci + di;
                    let (px, py) = ((i as f64 + 0.5 + off) * cw, (j as f64 + 0.5) * rh);
                    let d = (x - px).powi(2) + (y - py).powi(2);
                    if d < best {
                        best = d;
                        cell = (i.rem_euclid(cols), j.rem_euclid(rows));
                        lp = (x - px, y - py);
                    }
                }
            }
            let r = |q: u32| n.rand(cell.0, cell.1, 70 + q);
            // The paver: a little rotated and shifted in its bed.
            let ang = (r(0) - 0.5) * 0.05;
            let (ca, sa) = (ang.cos(), ang.sin());
            let (px, py) = (lp.0 - (r(1) - 0.5) * 0.003, lp.1 - (r(2) - 0.5) * 0.003);
            let (px, py) = (px * ca + py * sa, -px * sa + py * ca);
            let hexd = px
                .abs()
                .max((px * 0.5 + py * 0.866_025_4).abs())
                .max((px * 0.5 - py * 0.866_025_4).abs());
            let inr = cw * 0.5 - 0.0045;
            let wear = fbm(&ng, u, v, 60, 60, 3) * 0.004;
            let d = inr - hexd + wear.min(0.001);
            let face = smoothstep(0.0, 0.002, d);
            let chamfer = smoothstep(0.0, 0.007, d);
            let crown = 1.0 - (px * px + py * py) / (inr * inr) * 0.04;
            // Concrete: sand and small aggregate.
            let sand = fbm(&ng.layer(1), u, v, 360, 360, 2);
            let (agg, at) = grains(&ng.layer(2), u, v, 260, 0.45, 0.45, 23);
            let ph = 0.6 + 0.4 * chamfer + 0.05 * crown + 0.12 * sand - 0.08 * agg * (at - 0.3);
            let jh = 0.1 + fbm(&ng.layer(3), u, v, 200, 200, 3) * 0.3;
            let h = lerp(jh, ph, face);

            let base = mix(rgb(178, 164, 140), rgb(194, 180, 156), r(3));
            let grit = fbm(&ng.layer(5), u, v, 700, 700, 1);
            let mut c = mul(base, 1.0 + 0.35 * sand + 0.2 * grit + (r(4) - 0.5) * 0.08);
            c = mix(
                c,
                if at < 0.5 {
                    rgb(132, 122, 106)
                } else {
                    rgb(210, 200, 182)
                },
                agg * 0.2,
            );
            // Dirt creeping in from the joints, and broad stains across the path.
            let dirt = (1.0 - smoothstep(0.0, 0.025, d))
                * smoothstep(-0.15, 0.12, fbm(&nd, u, v, 30, 30, 3));
            c = mix(c, rgb(104, 94, 78), dirt * 0.5);
            let blotch = smoothstep(0.05, 0.2, fbm(&nd.layer(4), u, v, 20, 20, 4));
            c = mix(c, mul(c, 0.82), blotch * 0.6);
            let stain = fbm(&nd.layer(1), u, v, 4, 4, 4);
            c = mul(c, 1.0 + 0.16 * stain);
            let moss = smoothstep(0.08, 0.22, fbm(&nd.layer(2), u, v, 16, 16, 3));
            let joint = mix(
                mul(
                    rgb(86, 78, 66),
                    1.0 + 0.6 * fbm(&nd.layer(3), u, v, 220, 220, 2),
                ),
                rgb(70, 80, 50),
                moss * 0.6,
            );
            let col = mix(joint, c, face);
            let rough = lerp(1.0, 0.88 + 0.08 * sand, face);
            Px {
                h,
                c: col,
                r: rough,
            }
        },
    )
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

/// Pine floorboards, 13.5 cm wide, running along v.
fn floor_wood(n: &Noise, size: usize) -> Surface {
    let t = TILE_FLOOR_WOOD;
    let rows = 14i64;
    let bw = t / rows as f64;
    let px_m = t / size as f64;
    let ng = n.layer(1);
    paint(
        size,
        size,
        t,
        &[(0.002, 1.5), (0.02, 0.3)],
        false,
        |u0, v0| {
            let (u, v) = (v0, u0);
            let b = board_at(n, u, v, rows, 2, 0.3, t, 80);
            let lg = log_for(n, b.row, b.seg, bw, b.len, 0.25, (0.004, 0.007));
            let (late, fibre, knot) = grain_at(&ng, &lg, b.x, b.y, px_m);
            let side = b.y.min(bw - b.y);
            let joint = smoothstep(0.0003, 0.0011, side) * smoothstep(0.0003, 0.0011, b.end);
            let bevel = smoothstep(0.0, 0.003, side.min(b.end));
            let cup = -((b.y / bw - 0.5) * 2.0).powi(2) * 0.04;
            let h = lerp(
                0.0,
                0.8 + 0.2 * bevel + cup + 0.03 * late + 0.02 * fibre - 0.04 * knot,
                joint,
            );
            let tone = lg.tone;
            let early = mix(rgb(212, 164, 106), rgb(194, 144, 88), tone);
            let latec = mix(rgb(178, 122, 70), rgb(160, 106, 58), tone);
            let mut c = mix(early, latec, late * 0.75);
            let bright = 0.95 + 0.10 * n.rand(b.row, b.seg, 88);
            c = mul(
                c,
                bright
                    * (1.0
                        + 0.16 * fibre
                        + 0.16 * fbm_free(&ng.layer(2), b.x * 2.0 + lg.salt, b.y * 20.0, 3)),
            );
            c = mix(c, rgb(156, 94, 46), knot * 0.85);
            let kring = knot * (1.0 - knot) * 4.0;
            c = mix(c, rgb(84, 48, 26), kring * 0.8);
            let col = mix(rgb(70, 48, 30), c, joint);
            let rough = lerp(0.8, 0.58 + 0.06 * late + 0.06 * fibre, joint);
            Px {
                h,
                c: col,
                r: rough,
            }
        },
    )
}

/// Oak-print laminate: 23 cm planks running along u, micro-bevelled joints.
fn laminate(n: &Noise, size: usize) -> Surface {
    let t = TILE_LAMINATE;
    let rows = 8i64;
    let bw = t / rows as f64;
    let px_m = t / size as f64;
    let ng = n.layer(1);
    paint(size, size, t, &[(0.002, 2.0)], false, |u, v| {
        let b = board_at(n, u, v, rows, 1, 0.5, t, 90);
        let mut lg = log_for(n, b.row, b.seg, bw, b.len, 0.12, (0.004, 0.008));
        lg.z0 += 0.03;
        lg.slope *= 0.6;
        let (late, fibre, knot) = grain_at(&ng, &lg, b.x, b.y, px_m);
        // Oak's figure: long irregular streaks of light and dark along the board.
        let sx = b.x + lg.salt;
        let streak = fbm_free(
            &ng.layer(4),
            sx * 2.5 + fbm_free(&ng.layer(5), sx * 4.0, b.y * 12.0, 2) * 0.6,
            b.y * 45.0,
            5,
        );
        let flame = smoothstep(-0.05, 0.2, streak);
        // Oak pores: short dark dashes along the grain.
        let pore = smoothstep(
            0.17,
            0.26,
            fbm_free(&ng.layer(3), b.x * 60.0 + lg.salt, b.y * 1400.0, 2)
                + late * 0.08
                + streak * 0.2,
        );
        let side = b.y.min(bw - b.y);
        let edge = side.min(b.end);
        let joint = smoothstep(0.0, 0.0012, edge);
        let h = lerp(0.2, 1.0 - 0.05 * pore + 0.01 * fibre, joint);
        let tone = lg.tone;
        let early = mix(rgb(176, 148, 116), rgb(164, 136, 106), tone);
        let latec = mix(rgb(132, 102, 74), rgb(120, 92, 66), tone);
        let mut c = mix(early, latec, late * 0.45 + flame * 0.45);
        c = mul(
            c,
            1.0 + 0.12 * fibre + 0.08 * fbm_free(&ng.layer(2), b.x * 1.5 + lg.salt, b.y * 12.0, 3),
        );
        c = mix(c, rgb(104, 80, 58), pore * 0.3);
        c = mix(c, rgb(110, 80, 56), knot * 0.7);
        let col = mix(rgb(78, 60, 44), c, smoothstep(0.0003, 0.0014, edge));
        let rough = lerp(0.6, 0.36 + 0.12 * pore, joint);
        Px {
            h,
            c: col,
            r: rough,
        }
    })
}

/// Painted lap siding (grey detail): 12 boards of 21 cm across 2.5 m, each thicker at its butt (the
/// top of its course in the image, the bottom on the wall), with a few butt joints.
fn siding(n: &Noise, size: usize) -> Surface {
    let t = TILE_SIDING;
    let rows = 12i64;
    let bh = t / rows as f64;
    let px_m = t / size as f64;
    let ng = n.layer(1);
    paint(size, size, t, &[(0.004, 2.4), (0.03, 0.6)], true, |u, v| {
        // Half a board down, so the tile edge falls mid-board rather than on a butt.
        let b = board_at(n, u, v + 0.5 / rows as f64, rows, 1, 0.5, t, 95);
        let mut lg = log_for(n, b.row, b.seg, bh, b.len, 0.3, (0.005, 0.009));
        lg.z0 += 0.04;
        lg.slope *= 0.6;
        let (late, fibre, knot) = grain_at(&ng, &lg, b.x, b.y, px_m);
        let fy = b.y / bh;
        // Lap profile: thick butt at fy = 0, thin top under the next board at fy = 1.
        let butt = smoothstep(0.0, 0.006, b.y);
        let lap = 1.0 - 0.45 * fy;
        let end = smoothstep(0.0, 0.0018f64.max(px_m), b.end);
        let h = lerp(0.5 * lap, lap * (0.25 + 0.75 * butt), end) + 0.01 * late + 0.012 * fibre;
        // Paint over wood: grain shows faintly, dirt gathers under the butts.
        let streak = fbm_free(&ng.layer(4), b.x * 2.5 + lg.salt, b.y * 45.0, 4);
        let mut l = 0.80
            + 0.06 * late
            + 0.18 * fibre
            + 0.22 * streak
            + 0.05 * fbm(&ng.layer(2), u, v, 6, 24, 3);
        l -= 0.06 * knot + 0.25 * knot * (1.0 - knot);
        l -= 0.2 * (1.0 - smoothstep(0.0, 0.008, bh - b.y));
        l -= 0.25 * (1.0 - end);
        let streak = fbm(&ng.layer(3), u, v, 20, 3, 3);
        l *= 1.0 + 0.08 * streak;
        let rough = 0.72 + 0.1 * fibre + 0.1 * (1.0 - end);
        Px {
            h,
            c: grey(l),
            r: rough,
        }
    })
}

/// Shingles / shakes (grey detail): 11 courses on 1.5 m, random widths, ragged butts.
fn roof(n: &Noise, size: usize) -> Surface {
    let t = TILE_ROOF;
    let rows = 11i64;
    let ch = t / rows as f64;
    let ng = n.layer(1);
    let nb = n.layer(2);
    paint(size, size, t, &[(0.004, 4.0), (0.03, 2.0)], true, |u, v| {
        // Ragged butt line (top of each course in the image); side joints wander a little.
        let jag =
            fbm(&nb, u, v, 160, 4, 3) * 0.008 / t + fbm(&nb.layer(1), u, v, 30, 3, 2) * 0.006 / t;
        let vv = v - jag;
        let uu = u + fbm(&nb.layer(2), u, v, 6, 40, 3) * 0.006 / t;
        let b = board_at(n, uu, vv, rows, 11, 0.05, t, 85);
        let r = |q: u32| n.rand(b.row, b.seg, 100 + q);
        let fy = b.y / ch;
        let gap = 0.002 + 0.004 * r(0);
        let side = smoothstep(gap * 0.5, gap, b.end);
        let butt = smoothstep(0.0, 0.004, b.y);
        // The thick, frayed butt: a raised lip along the course's edge.
        let lipw = 0.006 + 0.004 * (fbm(&nb.layer(3), u, v, 90, 6, 2) + 0.5);
        let lip = 1.0 - smoothstep(lipw * 0.5, lipw, b.y);
        let fray = fbm(&nb.layer(4), u, v, 300, 60, 2);
        let thick = 0.6 + 0.4 * (1.0 - fy);
        let warp = (b.x / b.len - 0.5) * (r(1) - 0.5) * 0.3 + (r(2) - 0.5) * 0.2;
        // Riven/sawn face: strokes at a slant that differs per shingle.
        let ang = (r(6) - 0.5) * 1.2;
        let (ca, sa) = (ang.cos(), ang.sin());
        let (rx, ry) = (b.x * ca + b.y * sa + r(3) * 9.0, -b.x * sa + b.y * ca);
        let grain = fbm_free(&ng, rx * 10.0, ry * 260.0, 4)
            + 0.5 * fbm_free(&ng.layer(4), b.x * 400.0 + r(3) * 99.0, b.y * 20.0, 2);
        let split = smoothstep(
            0.2,
            0.3,
            fbm_free(&ng.layer(1), b.x * 90.0 + r(4) * 50.0, b.y * 4.0, 2),
        );
        let h = side
            * (thick * (0.3 + 0.7 * butt) + warp + 0.14 * grain - 0.15 * split
                + lip * (0.12 + 0.2 * fray) * butt);
        let mut l = 0.62 + (r(5) - 0.5) * 0.14 + 0.14 * grain;
        l *= 1.0 + 0.10 * fbm(&ng.layer(2), u, v, 8, 8, 3);
        // Weathering: the frayed butt lighter, the shadowed tail darker, dark gaps.
        l += 0.06 * lip * fray.max(0.0) * 4.0;
        l -= 0.05 * smoothstep(0.75, 1.0, fy);
        l -= 0.2 * split;
        l = lerp(0.18, l, side);
        let rough = 0.56 + 0.1 * grain.abs() + 0.08 * (1.0 - side);
        Px {
            h,
            c: grey(l),
            r: rough,
        }
    })
}

/// Painted interior/exterior plaster (grey detail): a fine, even float finish, faint mottling.
fn plaster(n: &Noise, size: usize) -> Surface {
    let t = TILE_PLASTER;
    let nf = n.layer(1);
    let nm = n.layer(2);
    paint(size, size, t, &[(0.003, 1.0)], true, |u, v| {
        let fine = fbm(&nf, u, v, 240, 240, 3);
        let grit = fbm(&nf.layer(1), u, v, 600, 600, 1);
        let swirl = fbm(&nf.layer(2), u, v, 12, 12, 4);
        let (pit, _) = grains(&nf.layer(3), u, v, 300, 0.04, 0.35, 31);
        let h = 0.5 + 0.06 * fine + 0.03 * grit + 0.15 * swirl - 0.2 * pit;
        let mottle = fbm(&nm, u, v, 3, 3, 5) + 0.5 * fbm(&nm.layer(1), u, v, 16, 16, 3);
        // Patchy float marks: faint lighter and darker clouds where the trowel went over again.
        let patch = smoothstep(0.02, 0.18, fbm(&nm.layer(2), u, v, 8, 8, 4))
            - smoothstep(0.04, 0.2, -fbm(&nm.layer(3), u, v, 10, 10, 4));
        let sponge = fbm(&nm.layer(4), u, v, 48, 48, 4);
        let l =
            0.86 * (1.0 + 0.10 * mottle + 0.04 * patch + 0.06 * sponge + 0.03 * fine) - 0.05 * pit;
        Px {
            h,
            c: grey(l),
            r: 0.9 + 0.05 * fine,
        }
    })
}

/// Concrete sidewalk slabs (grey detail): 60 x 30 cm in running bond, pepper-speckled aggregate.
fn sidewalk(n: &Noise, size: usize) -> Surface {
    let t = TILE_SIDEWALK;
    let (cols, rows) = (3i64, 6i64);
    let (sw, sh) = (t / cols as f64, t / rows as f64);
    let ng = n.layer(1);
    let nd = n.layer(2);
    paint(size, size, t, &[(0.008, 0.6), (0.04, 0.2)], true, |u, v| {
        let row = (v * rows as f64).floor() as i64;
        let off = if row.rem_euclid(2) == 0 { 0.0 } else { 0.5 };
        let xx = u * cols as f64 + off;
        let col = xx.floor() as i64;
        let (wi, wj) = (col.rem_euclid(cols), row.rem_euclid(rows));
        let r = |q: u32| n.rand(wi, wj, 120 + q);
        let x = (xx - col as f64) * sw;
        let y = (v * rows as f64 - row as f64) * sh;
        let g = 0.0022;
        let chip = fbm(&ng, u, v, 40, 40, 3) * 0.006;
        let d = rounded((x - g).min(sw - g - x), (y - g).min(sh - g - y), 0.004) + chip.min(0.0);
        let face = smoothstep(0.0, 0.004, d);
        let (pep, pt) = grains(&ng.layer(1), u, v, 200, 0.18, 0.45, 37);
        let sand = fbm(&ng.layer(2), u, v, 256, 256, 1);
        let tilt = (x / sw - 0.5) * (r(0) - 0.5) * 0.15 + (y / sh - 0.5) * (r(1) - 0.5) * 0.15;
        let h = lerp(
            0.2,
            1.0 + tilt + 0.15 * sand - 0.15 * pep * (1.0 - pt),
            face,
        );
        let mut l = 0.84 + (r(2) - 0.5) * 0.05 + 0.08 * sand;
        l = lerp(l, if pt < 0.7 { 0.5 } else { 0.97 }, pep * 0.45);
        l *= 1.0
            + 0.12 * white(&ng, u, v, 256, 61)
            + 0.08 * white(&ng, u, v, 512, 62)
            + 0.08 * fbm(&ng.layer(4), u, v, 128, 128, 2);
        let stain = fbm(&nd, u, v, 5, 5, 4);
        l *= 1.0 + 0.10 * stain;
        let blot = smoothstep(0.1, 0.25, fbm(&nd.layer(1), u, v, 10, 10, 3));
        l *= 1.0 - 0.08 * blot;
        l -= 0.08 * (1.0 - smoothstep(0.0, 0.015, d));
        l = lerp(0.6, l, face);
        Px {
            h,
            c: grey(l),
            r: 0.88 + 0.06 * sand,
        }
    })
}

/// Worn asphalt, 5 m: aggregate in dark binder, broad patches, a few hairline cracks running every
/// which way (some sealed), none of them standing out so the tile's repeat doesn't show.
fn asphalt(n: &Noise, size: usize) -> Surface {
    let t = TILE_ROAD;
    let k = size as f64 / 1024.0;
    let (cd, ch) = cracks(
        &n.layer(9),
        size,
        size,
        5,
        size as f64 * 0.14,
        k * 1.0,
        k * 4.0,
    );
    let ng = n.layer(1);
    let nd = n.layer(2);
    let w = size;
    paint(
        size,
        size,
        t,
        &[(0.012, 1.5), (0.05, 0.4)],
        false,
        move |u, v| {
            let i = ((v * w as f64) as usize).min(w - 1) * w + ((u * w as f64) as usize).min(w - 1);
            let (crack, halo) = (cd[i], ch[i]);
            // Aggregate: stones ~6-12 mm (1-3 px), some light.
            let (st, tone) = grains(&ng, u, v, 520, 0.7, 0.7, 41);
            let (big, bt) = grains(&ng.layer(1), u, v, 240, 0.25, 0.55, 43);
            let binder = fbm(&ng.layer(2), u, v, 400, 400, 2);
            let wear = fbm(&nd, u, v, 3, 3, 4) + 0.5 * fbm(&nd.layer(1), u, v, 12, 12, 3);
            // Worn surface: aggregate exposed and lighter where the binder wore off.
            let exposed = sat(0.5 + wear * 2.0);
            let mut c = mul(rgb(70, 70, 68), 1.0 + 0.4 * binder);
            let stone = mix(rgb(96, 94, 90), rgb(170, 166, 158), tone.powf(1.5));
            c = mix(c, stone, st * (0.45 + 0.4 * exposed));
            let bstone = mix(rgb(80, 78, 74), rgb(150, 146, 138), bt);
            c = mix(c, bstone, big * 0.6);
            let (white, _) = grains(&ng.layer(3), u, v, 330, 0.08, 0.5, 45);
            c = mix(c, rgb(196, 194, 186), white * 0.7);
            c = mul(c, 1.0 + 0.18 * wear);
            // Sealed cracks (dark, smoother tar band) and open cracks.
            let seal = smoothstep(0.3, 0.8, halo)
                * smoothstep(-0.05, 0.1, fbm(&nd.layer(2), u, v, 3, 3, 2));
            c = mix(c, rgb(44, 44, 43), seal * 0.4);
            c = mix(c, rgb(34, 34, 33), crack * 0.55);
            let oil = smoothstep(0.12, 0.28, fbm(&nd.layer(3), u, v, 6, 6, 4));
            c = mul(c, 1.0 - 0.12 * oil);
            let h = 0.5 + 0.35 * st * (0.5 + tone * 0.5) + 0.3 * big + 0.1 * binder
                - 0.6 * crack
                - 0.05 * seal;
            let rough =
                (0.86 - 0.1 * st * tone - 0.25 * seal - 0.1 * oil + 0.1 * crack).clamp(0.3, 1.0);
            Px { h, c, r: rough }
        },
    )
}

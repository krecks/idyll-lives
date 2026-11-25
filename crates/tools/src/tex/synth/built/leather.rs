use super::*;

/// Soft creases, as a hide takes where it is sat on and folded: `count` gently curving strokes about `len_px`
/// long (half to one and a half times), each a round-bottomed valley `width_px` in radius that fades out at
/// its ends, wrapping. Returns each pixel's depth, 0..1.
pub(super) fn creases(
    n: &Noise,
    w: usize,
    h: usize,
    count: usize,
    len_px: f64,
    width_px: f64,
) -> Vec<f64> {
    let mut depth = vec![0.0f64; w * h];
    for c in 0..count {
        let r = |k: i64| n.rand(c as i64, k, 733);
        let (mut x, mut y) = (r(0) * w as f64, r(1) * h as f64);
        let mut a = r(2) * std::f64::consts::TAU;
        // One bend all along (a radius of 80 px and up), and a slow wander on top of it.
        let bend = (r(3) - 0.5) * 0.012;
        let steps = (len_px * (0.5 + r(4))) as usize;
        let wid = width_px * (0.7 + 0.6 * r(5));
        for k in 0..steps {
            let t = k as f64 / steps as f64;
            a += bend + 0.01 * n.perlin2(t * 3.0 + c as f64 * 7.3, 0.5, 1 << 20, 1 << 20, 7);
            x += a.cos();
            y += a.sin();
            let taper = (t * 4.0).min(1.0) * ((1.0 - t) * 4.0).min(1.0);
            stamp(&mut depth, w, h, x, y, wid * (0.5 + 0.5 * taper), taper);
        }
    }
    depth
}

/// Leather (grey detail, 0.5 m): soft creases a hand's width apart running every which way, each a shallow
/// valley about a centimetre across and 6 to 18 cm long; the hide's own waviness; a pebble grain of cells
/// 2.5 mm across; and patches a hand's width across where it has been sat on and polished (roughness 0.35)
/// between duller ones (0.5). A pixel of the play camera at its nearest is 4 mm, so the grain only breaks the
/// sheen up: what shows there is the creases catching the light, the waviness and the uneven sheen. Its
/// colour is all but even (the part's own colour is the leather's): the creases are in the relief, not drawn.
pub(super) fn leather(n: &Noise, size: usize) -> Surface {
    let t = TILE_LEATHER;
    let px = size as f64 / t;
    let ng = n.layer(1);
    let nw = n.layer(2);
    let nm = n.layer(3);
    let crease = creases(&n.layer(4), size, size, 28, 0.12 * px, 0.005 * px);
    paint(size, size, t, &[(0.006, 0.25)], |u, v| {
        let at = |c: f64| ((c.rem_euclid(1.0) * size as f64) as usize).min(size - 1);
        let valley = crease[at(v) * size + at(u)];
        let cells = (t / 0.0025).round() as i64;
        let w = worley(&ng, u, v, cells, cells, 0.9, 11);
        let pebble = 1.0 - smoothstep(0.25, 0.62, w.f1);
        let tone = ng.rand(w.cell.0, w.cell.1, 17);
        let wave = fbm(&nw, u, v, 6, 6, 3);
        let worn = smoothstep(-0.08, 0.12, fbm(&nm, u, v, 5, 5, 3));
        Px {
            h: 0.5 + 0.9 * wave + 0.05 * pebble - 0.30 * valley,
            c: grey(0.985 - 0.015 * valley + 0.01 * (pebble - 0.5) + 0.01 * (tone - 0.5)),
            r: 0.46 - 0.10 * worn - 0.02 * pebble + 0.03 * valley,
        }
    })
}

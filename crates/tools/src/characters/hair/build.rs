//! Building a hairstyle from its parts.
//!
//! Split from `characters/hair.rs` (see `mod.rs`).

use super::*;

pub fn build_hair(
    family: &str,
    stage: &str,
    head: &HeadFrame,
    p: &[V3; NB],
) -> Vec<(String, Mesh)> {
    let h = head;
    let ears = Ears::of(family, stage, h);
    let hh = h.hh;
    let sc = Scalp::new(h, true).round(&ears);
    let skin = Scalp::new(h, false);
    let brow_y = (h.brows[0][0].y + h.brows[0][1].y + h.brows[1][0].y + h.brows[1][1].y) / 4.0;
    let mut styles: Vec<(&str, Vec<Comp>)> = Vec::new();
    let deg = radians;
    let neck_y = p[b("neck_01")].y;
    let chin = h.chin_y;
    // The body's back (m, z) at height y, roughly: behind the neck, then the shoulder blades.
    let sp = p[b("spine_03")];
    let neck_z = p[b("neck_01")].z;
    let back = move |y: f64| {
        interp(
            y,
            &[sp[1] - 0.25 * hh, sp[1], neck_y, chin],
            &[
                sp[2] - 0.5 * hh,
                sp[2] - 0.46 * hh,
                neck_z - 0.3 * hh,
                h.centre[2] - 0.2 * hh,
            ],
        ) - 0.012
    };

    /// A thickness over the head: top at the crown, side over the ears, back_ behind.
    fn thick(top: f64, side: f64, back_: f64, nape: Option<f64>) -> impl Fn(f64, f64) -> f64 {
        move |a: f64, u: f64| {
            let ca = a.cos();
            let s = top + (side - top) * smoothstep(0.2, 0.9, u);
            let bk = match nape {
                None => back_,
                Some(nv) => back_ + (nv - back_) * smoothstep(0.5, 1.0, u),
            };
            s + (bk - s) * smoothstep(0.2, -0.8, ca) * smoothstep(0.1, 0.8, u)
        }
    }
    // Wisps past a cap's rim (m, by azimuth): short over the forehead, longer at the temples and longest
    // at the nape; where curtains hang from the sides round the back, only in front of them; shorter
    // on hair drawn back tight.
    let rim_wisps = move |a: f64| {
        hh * (0.04
            + 0.015 * smoothstep(deg(40.0), deg(75.0), round(a))
            + 0.015 * smoothstep(deg(110.0), deg(140.0), round(a)))
    };
    let front_wisps = move |a: f64| {
        if round(a) < deg(60.0) {
            rim_wisps(a)
        } else {
            0.0
        }
    };
    let slick_wisps = move |a: f64| 0.6 * rim_wisps(a);
    // Short locks past the hairline at the temples and the nape (none over the forehead or the ears).
    let nape_tips = move |a: f64| {
        deg(4.5)
            * smoothstep(deg(105.0), deg(135.0), round(a)).max(
                smoothstep(deg(70.0), deg(50.0), round(a))
                    * smoothstep(deg(30.0), deg(45.0), round(a)),
            )
    };

    // Short: volume on top, combed away from a side parting (the resident's right) in locks, a quiff
    // over the forehead.
    let base = thick(0.05 * hh, 0.03 * hh, 0.028 * hh, Some(0.014 * hh));
    let quiffed = |a: f64, u: f64| {
        base(a, u)
            + 0.022
                * hh
                * (-(a / 0.6).powi(2)).exp()
                * smoothstep(0.35, 0.75, u)
                * smoothstep(1.0, 0.85, u)
    };
    styles.push((
        "short",
        vec![cap(
            &sc,
            Cap {
                thick: &quiffed,
                rows: 11,
                cols: 40,
                limit: None,
                edge: CLEAR,
                ridge: 0.35,
                tips: Some(&nape_tips),
                part: Some((deg(20.0), 0.3)),
                bands: None,
                ears: Meet::Round(17),
                wisps: Some(&rim_wisps),
            },
        )],
    ));

    // Crop: a thin cap, a squared hairline and a tiny fringe in short points.
    let front = sc.hairline(0.0) + deg(3.0);
    let square = |a: f64| {
        if a.abs() < deg(55.0) {
            front + deg(4.0) * (a.abs() / deg(55.0)).powf(6.0)
        } else {
            sc.hairline(a)
        }
    };
    let crop_tips =
        |a: f64| deg(3.0) * smoothstep(deg(60.0), deg(20.0), round(a)) + 0.5 * nape_tips(a);
    styles.push((
        "crop",
        vec![cap(
            &sc,
            Cap {
                thick: &thick(0.022 * hh, 0.016 * hh, 0.016 * hh, Some(0.011 * hh)),
                rows: 8,
                cols: 40,
                limit: Some(&square),
                edge: CLEAR,
                ridge: 0.25,
                tips: Some(&crop_tips),
                part: None,
                bands: None,
                ears: Meet::Round(17),
                wisps: Some(&rim_wisps),
            },
        )],
    ));

    // Layered: a cap and three flared tiers in pointed locks, the lowest to the jaw.
    let mut lay = vec![cap(
        &sc,
        Cap {
            thick: &thick(0.05 * hh, 0.04 * hh, 0.04 * hh, None),
            rows: 9,
            cols: 32,
            limit: None,
            edge: CLEAR,
            ridge: 0.3,
            tips: None,
            part: None,
            bands: None,
            ears: Meet::Round(14),
            wisps: Some(&front_wisps),
        },
    )];
    for (k, (yt, yb, off, fl)) in [
        (0.72, 0.5, 0.028, 0.06),
        (0.58, 0.3, 0.02, 0.09),
        (0.44, 0.08, 0.012, 0.12),
    ]
    .into_iter()
    .enumerate()
    {
        let span = deg(62.0 + 6.0 * k as f64);
        let (top, bot) = (chin + yt * hh, chin + yb * hh);
        lay.push(curtain(
            &sc,
            Curtain {
                a0: span,
                a1: TAU - span,
                top: &move |_| top,
                bottom: &move |_| bot,
                off: off * hh,
                locks: 10,
                rows: 4,
                flare: fl,
                thick: 0.022 * hh,
                curl: 0.5,
                tip: 0.1 * hh,
                sharp: 3.5,
                back: None,
                wave: 0.0,
                seed: k as i64,
                wisp: 0.06 * hh,
            },
        ));
    }
    styles.push(("layered", lay));

    // Tousled: a cap and seeded locks pointing every which way, their tips lifting, a few over the
    // forehead.
    let tb = thick(0.034 * hh, 0.028 * hh, 0.026 * hh, Some(0.016 * hh));
    let lumpy =
        |a: f64, u: f64| tb(a, u) * (1.0 + 0.35 * noise3(v3(a.cos(), 2.0 * u, a.sin()), 5, 2.5));
    let mut tou = vec![cap(
        &sc,
        Cap {
            thick: &lumpy,
            rows: 8,
            cols: 24,
            limit: None,
            edge: CLEAR,
            ridge: 0.3,
            tips: Some(&nape_tips),
            part: None,
            bands: None,
            ears: Meet::Round(10),
            wisps: Some(&rim_wisps),
        },
    )];
    let kept = tousled_kept(family);
    for (k, lock) in tousled_locks().into_iter().enumerate() {
        if !kept[k] {
            continue;
        }
        let [a, down, len, turn, lift, wide] = lock;
        let phi = sc.hairline(a) * down;
        // (Where it clears the ears on the base stage but at no length here: as short as it is cut, and
        // `clear` keeps it off them.)
        let len = tousled_cut(&sc, hh, lock).unwrap_or(len * 0.85f64.powi(7));
        tou.push(tuft(
            &sc,
            a,
            phi,
            len,
            turn,
            lift * hh,
            0.03 * hh,
            wide * hh,
            0.028 * hh,
            k as f64,
        ));
    }
    styles.push(("tousled", tou));

    // Side-swept: a cap with a high side parting and three locks swept from it across the forehead.
    let mut sw = vec![cap(
        &sc,
        Cap {
            thick: &thick(0.045 * hh, 0.03 * hh, 0.028 * hh, Some(0.014 * hh)),
            rows: 9,
            cols: 36,
            limit: None,
            edge: CLEAR,
            ridge: 0.3,
            tips: Some(&nape_tips),
            part: Some((deg(45.0), 0.35)),
            bands: None,
            ears: Meet::Round(15),
            wisps: Some(&rim_wisps),
        },
    )];
    let fa = [0.75, 0.35, 0.0, -0.4, -0.75, -1.05];
    let fk = [0.55, 0.78, 0.92, 0.98, 1.02, 1.08];
    let fo = [0.05, 0.05, 0.048, 0.045, 0.038, 0.028];
    for (k, (da, dk, len, size)) in [
        (0.0, 0.0, 1.0, 1.0),
        (0.07, -0.06, 0.8, 0.85),
        (-0.06, 0.05, 0.62, 0.8),
    ]
    .into_iter()
    .enumerate()
    {
        let n = 3 + (len * 3.0) as usize;
        let pts: Vec<V3> = (0..n)
            .map(|i| {
                sc.point(
                    fa[i] + da,
                    sc.hairline(fa[i] + da) * (fk[i] + dk),
                    fo[i] * hh * (0.9 + 0.1 * size),
                )
            })
            .collect();
        let pts = spline(&pts, 9);
        let t9 = linspace(0.0, 1.0, 9);
        let ry: Vec<f64> = t9
            .iter()
            .map(|&t| interp(t, &[0.0, 0.4, 1.0], &[0.05, 0.068, 0.016]) * hh * size)
            .collect();
        let rz: Vec<f64> = t9
            .iter()
            .map(|&t| interp(t, &[0.0, 0.5, 1.0], &[0.022, 0.028, 0.01]) * hh * size)
            .collect();
        sw.push(lobe(
            &pts,
            &ry,
            &rz,
            8,
            v3(0.0, 1.0, 0.3),
            9.0,
            k as f64,
            0.0,
            None,
        ));
    }
    styles.push(("sideSwept", sw));

    // Afro: a big rounded cap of round puffs.
    let puffs: Vec<V3> = (0..46)
        .map(|i| {
            // A Fibonacci spiral over the head, in the skull ellipsoid's normalised frame.
            let y = 1.0 - 2.0 * (i as f64 + 0.5) / 46.0;
            let r = (1.0 - y * y).sqrt();
            let t = i as f64 * PI * (3.0 - 5f64.sqrt());
            v3(r * t.sin(), y, r * t.cos())
        })
        .collect();
    // How far up a puff a direction is (0 between puffs .. 1 on a puff's top).
    let puff = |e: V3| {
        puffs
            .iter()
            .map(|q| {
                (1.0 - (e.dot(q).clamp(-1.0, 1.0).acos() / 0.32).powi(2))
                    .max(0.0)
                    .sqrt()
            })
            .fold(0.0, f64::max)
    };
    let dir = |a: f64, u: f64| {
        let e = Scalp::dirs(a, u * sc.hairline(a));
        e / e.norm()
    };
    let rounded = |a: f64, u: f64| {
        // Out to a rounded shell 1.35 times the skull, a little up and back, in puffs.
        let e = Scalp::dirs(a, u * sc.hairline(a)).component_mul(&h.radii);
        let e = e / e.norm();
        let q = e.component_div(&h.radii);
        let r_s = 1.0 / (q.x * q.x + q.y * q.y + q.z * q.z).sqrt();
        let rr = h.radii.component_mul(&v3(1.36, 1.3, 1.36));
        let w = e.component_div(&rr).map(|v| v.abs().powf(2.1));
        let sh = v3(0.0, 0.1, -0.08).component_mul(&h.radii);
        let r_t =
            1.0 / (w.x + w.y + w.z).sqrt().powf(1.0 / 1.2) + (e.x * sh.x + e.y * sh.y + e.z * sh.z);
        ((r_t - r_s - 0.02 * hh).max(0.02 * hh) + 0.06 * hh * puff(dir(a, u)))
            * (0.3 + 0.7 * smoothstep(1.0, 0.55, u))
    };
    let puff_bands = |a: f64, u: f64| 0.15 + 0.75 * puff(dir(a, u));
    let mut afro = cap(
        &sc,
        Cap {
            thick: &rounded,
            rows: 18,
            cols: 52,
            limit: None,
            edge: CLEAR + 0.004 * hh,
            ridge: 0.0,
            tips: None,
            part: None,
            bands: Some(&puff_bands),
            ears: Meet::Over,
            wisps: None,
        },
    );
    // (Under the afro's rim are the three rings of its hem.)
    afro.wisps = shingles(&afro.pos, h.centre, dense_cols(52), 3, 3, 0.06 * hh);
    styles.push(("afro", vec![afro]));

    // Bob: a shell to the jaw in blunt locks with a curled-in hem, and a straight fringe.
    let mut bob = vec![cap(
        &sc,
        Cap {
            thick: &thick(0.05 * hh, 0.04 * hh, 0.04 * hh, None),
            rows: 9,
            cols: 32,
            limit: None,
            edge: CLEAR,
            ridge: 0.15,
            tips: None,
            part: None,
            bands: None,
            ears: Meet::Round(14),
            wisps: None,
        },
    )];
    let (bt, bb) = (chin + 0.72 * hh, chin + 0.06 * hh);
    bob.push(curtain(
        &sc,
        Curtain {
            a0: deg(58.0),
            a1: TAU - deg(58.0),
            top: &move |_| bt,
            bottom: &move |_| bb,
            off: 0.012 * hh,
            locks: 9,
            rows: 5,
            flare: 0.12,
            thick: 0.03 * hh,
            curl: 1.0,
            tip: 0.03 * hh,
            sharp: 1.5,
            back: None,
            wave: 0.0,
            seed: 0,
            wisp: 0.05 * hh,
        },
    ));
    let under = |a: f64| sc.hairline(a) * 0.8;
    let bob_bot = |a: f64| brow_y + hh * (0.05 + 0.03 * (a / deg(64.0)).powi(2));
    bob.push(fringe(
        &skin,
        Fringe {
            a0: -deg(64.0),
            a1: deg(64.0),
            top: &under,
            bottom: &bob_bot,
            off: 0.03 * hh,
            thin: 0.012 * hh,
            locks: 8,
            rows: 4,
            thick: 0.022 * hh,
            tip: 0.03 * hh,
            // A blunt fringe: at 3 the eight locks' points made a sawtooth hem, a comb of teeth over
            // the brow; 1.5 leaves scallops you have to look for.
            sharp: 1.5,
            flare: 0.15,
            seed: 2.0,
            wisp: 0.035 * hh,
        },
    ));
    styles.push(("bob", bob));

    // Long: a cap parted in the middle, the sides to the shoulders, the back to mid-back, all in locks
    // with pointed tips; a fringe either side of the parting framing the face.
    let mut lng = vec![cap(
        &sc,
        Cap {
            thick: &thick(0.045 * hh, 0.035 * hh, 0.035 * hh, None),
            rows: 9,
            cols: 32,
            limit: None,
            edge: CLEAR,
            ridge: 0.15,
            tips: None,
            part: Some((0.0, 0.3)),
            bands: None,
            ears: Meet::Round(14),
            wisps: Some(&front_wisps),
        },
    )];
    let shoulder = p[b("clavicle_l")].y;
    let side_bot = |a: f64| {
        interp(
            round(a),
            &[deg(70.0), deg(110.0), deg(150.0)],
            &[shoulder + 0.25 * hh, shoulder - 0.1 * hh, sp[1] - 0.5 * hh],
        )
    };
    let lt = chin + 0.72 * hh;
    // Further off the back at the shoulders, which rise and roll back under it when sitting down.
    let long_back = |y: f64| back(y) - 0.036 * hh * (-((y - shoulder) / (0.2 * hh)).powi(2)).exp();
    lng.push(loose(curtain(
        &sc,
        Curtain {
            a0: deg(66.0),
            a1: TAU - deg(66.0),
            top: &move |_| lt,
            bottom: &side_bot,
            off: 0.012 * hh,
            locks: 8,
            rows: 7,
            flare: 0.18,
            thick: 0.032 * hh,
            curl: 0.6,
            tip: 0.07 * hh,
            sharp: 2.5,
            back: Some(&long_back),
            wave: 0.004 * hh,
            seed: 0,
            wisp: 0.08 * hh,
        },
    )));
    for s in [-1.0, 1.0] {
        // Two locks either side of the parting, the outer one down past the cheek to the jaw.
        let outer = [
            (0.45 * s, brow_y + 0.14 * hh, 0.036 * hh),
            (0.85 * s, brow_y + 0.0 * hh, 0.038 * hh),
            (1.04 * s, brow_y - 0.2 * hh, 0.036 * hh),
            (1.13 * s, brow_y - 0.36 * hh, 0.032 * hh),
        ];
        lng.push(face_lock(
            &skin,
            0.1 * s,
            sc.hairline(0.1 * s) * 0.7,
            &outer,
            0.065 * hh,
            0.028 * hh,
            1.0 + s,
        ));
        let inner = [
            (0.35 * s, brow_y + 0.15 * hh, 0.032 * hh),
            (0.62 * s, brow_y + 0.05 * hh, 0.034 * hh),
            (0.86 * s, brow_y - 0.08 * hh, 0.032 * hh),
        ];
        lng.push(face_lock(
            &skin,
            0.06 * s,
            sc.hairline(0.06 * s) * 0.7,
            &inner,
            0.055 * hh,
            0.025 * hh,
            2.0 + s,
        ));
    }
    styles.push(("long", lng));

    // Ponytail: a slicked cap, a band at the back of the head and a tapering tail in twisting locks.
    let b15 = |a: f64, _u: f64| 0.5 + 0.5 * (a * 15.0).sin();
    let mut pt = vec![cap(
        &sc,
        Cap {
            thick: &thick(0.024 * hh, 0.018 * hh, 0.02 * hh, Some(0.012 * hh)),
            rows: 9,
            cols: 32,
            limit: None,
            edge: CLEAR,
            ridge: 0.2,
            tips: None,
            part: None,
            bands: None,
            ears: Meet::Round(14),
            wisps: Some(&slick_wisps),
        },
    )];
    let tie = sc.point(PI, deg(95.0), 0.03 * hh);
    pt.push(tor(torus(
        tie + v3(0.0, 0.0, -0.012 * hh),
        0.05 * hh,
        0.02 * hh,
        12,
        6,
        v3(0.0, 0.25, -1.0),
    )));
    let mut tail = [
        tie + v3(0.0, 0.0, -0.03 * hh),
        tie + v3(0.0, -0.08 * hh, -0.13 * hh),
        tie + v3(0.0, -0.3 * hh, -0.16 * hh),
        tie + v3(0.0, -0.6 * hh, -0.1 * hh),
        tie + v3(0.0, -0.85 * hh, -0.07 * hh),
    ];
    for q in &mut tail {
        q[2] = q[2].min(back(q[1]) - 0.035 * hh);
    }
    let tp = spline(&tail, 12);
    let t12 = linspace(0.0, 1.0, 12);
    let ry: Vec<f64> = t12
        .iter()
        .map(|&t| interp(t, &[0.0, 0.25, 1.0], &[0.08, 0.105, 0.025]) * hh)
        .collect();
    let rz: Vec<f64> = t12
        .iter()
        .map(|&t| interp(t, &[0.0, 0.25, 1.0], &[0.07, 0.085, 0.02]) * hh)
        .collect();
    let mut m = loft(&tp, &ry, &rz, 12, v3(1.0, 0.0, 0.0), caps(2));
    let z = grooved(&mut m, 12, 4.0, 0.16, 2.5);
    let f = m.t.iter().map(|&t| smoothstep(0.1, 0.9, t)).collect();
    let tip = brush(&m.pos, 12, 3, 0.07 * hh, TIP_TAPER, None);
    pt.push(loose(Comp {
        wisps: vec![tip],
        ..comp(m.pos, m.idx, None, Some(z), None, Some(f))
    }));
    styles.push(("ponytail", pt));

    // Braid: a cap and three strands plaited down the back, tied off with a tuft below the tie.
    let mut br = vec![cap(
        &sc,
        Cap {
            thick: &thick(0.03 * hh, 0.024 * hh, 0.026 * hh, Some(0.018 * hh)),
            rows: 9,
            cols: 32,
            limit: None,
            edge: CLEAR,
            ridge: 0.2,
            tips: None,
            part: None,
            bands: Some(&b15),
            ears: Meet::Round(14),
            wisps: Some(&slick_wisps),
        },
    )];
    let start = sc.point(PI, deg(120.0), 0.03 * hh);
    let len = 0.82 * hh;
    // The braid's middle at s (0 at the nape .. 1 at the tie).
    let axis = |s: f64| {
        let y = start[1] - s * len;
        v3(0.0, y, (back(y) - 0.055 * hh).min(start[2] - 0.09 * hh * s))
    };
    for k in 0..3 {
        let ph = TAU * k as f64 / 3.0;
        let s = linspace(0.0, 1.0, 22);
        let pts: Vec<V3> = s
            .iter()
            .map(|&s| {
                let th = TAU * 3.5 * s + ph;
                let w = 0.05 * hh * (0.75 - 0.25 * s) * smoothstep(-0.1, 0.12, s);
                axis(s)
                    + v3(
                        w * th.sin(),
                        0.0,
                        -0.022 * hh * (2.0 * th).sin() * smoothstep(-0.1, 0.12, s),
                    )
            })
            .collect();
        let r: Vec<f64> = s.iter().map(|&s| 0.036 * hh * (1.0 - 0.35 * s)).collect();
        let m = loft(&pts, &r, &r, 6, v3(0.0, 0.0, -1.0), caps(1));
        let z =
            m.t.iter()
                .map(|&t| 0.5 + 0.4 * (TAU * 7.0 * t + ph).sin())
                .collect();
        let f = m.t.iter().map(|&t| smoothstep(0.0, 0.67, t)).collect();
        br.push(loose(comp(
            m.pos,
            m.idx,
            None,
            Some(z),
            Some(vec![0.1; m.t.len()]),
            Some(f),
        )));
    }
    let end = axis(1.0);
    let mut knot = tor(torus(
        end + v3(0.0, -0.01 * hh, 0.0),
        0.03 * hh,
        0.012 * hh,
        10,
        5,
        v3(0.0, 1.0, 0.0),
    ));
    knot.f = vec![1.0; knot.pos.len()];
    br.push(loose(knot));
    let tp: Vec<V3> = linspace(0.0, 1.0, 5)
        .iter()
        .map(|&s| end + v3(0.0, -0.13 * hh * s - 0.015 * hh, -0.01 * hh * s))
        .collect();
    let r: Vec<f64> = linspace(0.0, 1.0, 5)
        .iter()
        .map(|&s| interp(s, &[0.0, 0.4, 1.0], &[0.025, 0.04, 0.012]) * hh)
        .collect();
    let mut m = loft(&tp, &r, &r, 8, v3(0.0, 0.0, -1.0), caps(1));
    let z = grooved(&mut m, 8, 3.0, 0.2, 1.0);
    let tip = brush(&m.pos, 8, 2, 0.05 * hh, TIP_TAPER, None);
    br.push(loose(Comp {
        wisps: vec![tip],
        ..comp(
            m.pos,
            m.idx,
            None,
            Some(z),
            None,
            Some(vec![1.0; m.t.len()]),
        )
    }));
    styles.push(("braid", br));

    // Buns: a cap and two buns high on each side, twisted, each with a band round its foot.
    let mut bun = vec![cap(
        &sc,
        Cap {
            thick: &thick(0.026 * hh, 0.02 * hh, 0.022 * hh, Some(0.014 * hh)),
            rows: 9,
            cols: 32,
            limit: None,
            edge: CLEAR,
            ridge: 0.2,
            tips: None,
            part: Some((0.0, 0.3)),
            bands: Some(&b15),
            ears: Meet::Round(14),
            wisps: Some(&slick_wisps),
        },
    )];
    for s in [-1.0, 1.0] {
        let c = sc.point(s * deg(118.0), deg(38.0), 0.13 * hh);
        let ax = (c - h.centre) / (c - h.centre).norm();
        let e = ellipsoid(c, v3(0.15, 0.14, 0.15) * hh, 9, 16);
        // A twist round the bun's axis (out from the head): coils that wind up to its top.
        let (u, v) = super::prims::frame(ax, v3(0.0, 1.0, 0.0));
        let mut z = Vec::new();
        let pos: Vec<V3> = e
            .pos
            .iter()
            .map(|q| {
                let d = q - c;
                let up = d.dot(&ax) / (0.15 * hh);
                let g = (3.0 * d.dot(&v).atan2(d.dot(&u)) + 6.0 * up).cos();
                z.push(0.5 + 0.4 * g);
                c + d * (1.0 + 0.09 * g * (1.0 - up.abs().powi(4)))
            })
            .collect();
        let out = pos.iter().map(|q| q - c).collect();
        bun.push(comp(pos, e.idx, Some(out), Some(z), None, None));
        bun.push(tor(torus(
            c - ax * (0.115 * hh),
            0.085 * hh,
            0.02 * hh,
            12,
            5,
            ax,
        )));
    }
    styles.push(("bun", bun));

    let mut names: Vec<&str> = styles.iter().map(|s| s.0).collect();
    names.push("none");
    assert!(HAIR_STYLES.iter().all(|s| names.contains(s)) && names.len() == HAIR_STYLES.len());
    if BEARD_FAMILIES.contains(&family) {
        styles.push(("beard", vec![beard(&skin)]));
    }
    styles
        .into_iter()
        .flat_map(|(name, comps)| {
            let part = format!("hair.{name}");
            let solid = finish(&comps, h, &ears, p, name == "beard");
            let soft = (wisps_part(&part), wisps(&comps, &solid, h, &ears));
            [(part, solid), soft]
        })
        .collect()
}

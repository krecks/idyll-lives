//! The cage itself, fitted to a family and a stage.

use super::*;

/// The cage with a face of its own (`id`; identity.rs): the head's part of it differs.
pub(super) fn body_cage_of(family: &str, stage: &str, id: &Identity) -> (Cage, [V3; NB]) {
    let pr = proportions(family, stage);
    let p = joints(family, stage);
    let pj = |n: &str| p[b(n)];
    let s = shape(family, stage);
    let (h, yg, female) = (s.h, s.yg, s.female);
    let (r_th, r_a, r_n) = (s.r_th, s.r_a, s.r_n);
    let bel = s.belly;
    let sw = spine_weights(&p);
    let mut c = Cage::new();
    let (x, y, z) = (v3(1.0, 0.0, 0.0), v3(0.0, 1.0, 0.0), v3(0.0, 0.0, 1.0));
    let y_hip = pj("thigh_l").y;
    let (y_s1, y_s2, y_s3) = (pj("spine_01").y, pj("spine_02").y, pj("spine_03").y);
    let y_neck = pj("neck_01").y;
    let hip_x = pj("thigh_r").x;
    let sh = pj("upperarm_r");

    // Arm root loops (right, left): the 2 x 2 patch on the chest rings T5-T7 the arm leaves through.
    let mut roots: HashMap<&str, Vec<V3>> = HashMap::new();
    for (sd, sx) in [("r", 1.0), ("l", -1.0)] {
        let jsh = pj(&format!("upperarm_{sd}"));
        let o = v3(sx, 0.0, 0.0);
        // (chi: 135 180 225 | 270 | 315 0 45 | 90): bottom front .. round the loop.
        let spec = [
            (-0.75, -1.5, 0.75),
            (-0.82, -1.95, 0.0),
            (-0.75, -1.5, -0.72),
            (-0.6, -0.75, -1.0),
            (-0.4, 0.05, -0.78),
            (-0.2, 0.25, 0.0),
            (-0.4, 0.05, 0.8),
            (-0.6, -0.75, 1.02),
        ];
        roots.insert(
            sd,
            spec.iter()
                .map(|&(dx, dy, dz)| jsh + (o * dx + y * dy + z * dz) * r_a)
                .collect(),
        );
    }

    // Torso rings T1 (hips) .. T7 (shoulders); (y, half width, front depth, back depth, z,
    // squareness). T7 is all neck loop and arm loops (below).
    let f = |a: f64, m: f64| if female { a } else { m };
    // Grown (`Shape::m`), on top of the families' own numbers: a woman's hips wider under a narrower
    // chest and shoulders, her waist about 0.6 of her hips across (it was 0.52, 21 cm: a wasp's); a
    // man's chest and shoulders broader over narrower hips, his waist nearly as wide as them (0.85; it
    // was 0.73, an hourglass of his own, which is much of why men and women looked alike).
    let g = |a: f64, m: f64| f(a, m) * s.m;
    let hw = hip_x + r_th * 0.85 + g(0.011, -0.003) * h;
    let rows = [
        (
            y_hip + 0.012 * h,
            hw,
            0.064 * h + bel * 0.3,
            f(0.084, 0.078) * h,
            -0.006 * h,
            2.3,
        ),
        (
            y_hip + 0.55 * (y_s1 - y_hip),
            (f(0.106, 0.095) + g(0.003, -0.004)) * h * (1.0 + 0.1 * yg),
            0.062 * h + bel * 0.7,
            f(0.075, 0.07) * h,
            0.0,
            2.3,
        ),
        (
            (y_s1 + y_s2) / 2.0,
            (f(0.068, 0.08) + g(0.005, 0.01)) * h * s.waist * (1.0 + 0.25 * yg),
            0.062 * h + bel,
            0.056 * h,
            0.003 * h,
            2.3,
        ),
        (
            y_s2 + 0.45 * (y_s3 - y_s2),
            (f(0.085, 0.098) + g(-0.002, 0.006)) * h * (1.0 + 0.12 * yg),
            0.068 * h + bel * 0.6,
            0.060 * h,
            0.002 * h,
            2.2,
        ),
        (
            sh[1] - 1.95 * r_a,
            (f(0.092, 0.108) + g(-0.004, 0.007)) * h,
            (f(0.07, 0.074) + g(0.0, 0.004)) * h + bel * 0.2,
            0.064 * h,
            0.0,
            2.2,
        ),
        (
            sh[1] - 0.75 * r_a,
            (f(0.096, 0.112) + g(-0.005, 0.008)) * h,
            (0.07 + g(0.0, 0.003)) * h,
            0.064 * h,
            -0.002 * h,
            2.2,
        ),
        (
            sh[1] + 0.25 * r_a,
            (f(0.09, 0.1) + g(-0.004, 0.006)) * h,
            0.05 * h,
            0.055 * h,
            -0.006 * h,
            2.2,
        ),
    ];
    let th12: Vec<f64> = (0..12).map(|i| radians(i as f64 * 30.0)).collect();
    let mut t: Vec<Vec<V3>> = rows
        .iter()
        .map(|&(yy, a, bf, bb, zc, pw)| ring_pts(v3(0.0, yy, zc), x, z, a, bf, bb, &th12, pw))
        .collect();
    // Bust (female, gentle), shoulder blades, buttocks, belly.
    if female {
        for k in [1, 11] {
            t[4][k][2] += (0.016 * (1.0 - yg) + 0.012 * s.m) * h;
            t[4][k][1] -= 0.008 * h;
            t[3][k][2] += (0.005 * (1.0 - yg) + 0.004 * s.m) * h;
        }
        t[4][0][2] += 0.006 * s.m * h;
    }
    for k in [5, 7] {
        t[5][k][2] -= 0.006 * h;
        t[0][k][2] -= f(0.008, 0.004) * h;
        t[1][k][2] -= 0.004 * h;
    }
    t[2][0][2] += bel * 0.4;
    // The arm loops replace the chest rings' sides.
    for sd in ["l", "r"] {
        let (k0, k1, k2) = arm_k(sd);
        let r = &roots[sd];
        t[4][k0] = r[0];
        t[4][k1] = r[1];
        t[4][k2] = r[2];
        t[5][k2] = r[3];
        t[6][k2] = r[4];
        t[6][k1] = r[5];
        t[6][k0] = r[6];
        t[5][k0] = r[7];
    }
    // The shoulder ring's front and back meet the neck: they are the neck loop's.
    let zn = pj("neck_01").z;
    let neck_loop = [
        (0, (0.0, y_neck - 0.016 * h, zn + r_n * 1.3)),
        (1, (r_n * 0.95, y_neck - 0.008 * h, zn + r_n * 0.95)),
        (5, (r_n * 0.95, y_neck + 0.006 * h, zn - r_n * 0.95)),
        (6, (0.0, y_neck + 0.002 * h, zn - r_n * 1.3)),
    ];
    for (k, (xx, yy, zz)) in neck_loop {
        t[6][k] = v3(xx, yy, zz);
        if k == 1 || k == 5 {
            t[6][12 - k] = v3(-xx, yy, zz);
        }
    }

    let tw = |r: usize, k: usize| -> Weights {
        let yy = t[r][k][1];
        let w = sw(yy);
        let xx = t[r][k][0];
        let side = if xx > 0.0 { "r" } else { "l" };
        let thigh = one(bone(format!("thigh_{side}")));
        if r == 0 {
            return if xx.abs() > 0.3 * hw {
                mix(&[(0.62, &w), (0.38, &thigh)])
            } else {
                mix(&[(0.76, &w), (0.12, &one("thigh_l")), (0.12, &one("thigh_r"))])
            };
        }
        if r == 1 {
            return if xx.abs() > 0.3 * hw {
                mix(&[(0.9, &w), (0.1, &thigh)])
            } else {
                w
            };
        }
        let clav = bone(format!("clavicle_{side}"));
        if r == 6 {
            if k == 0 || k == 6 {
                return vec![("spine_03", 0.6), ("neck_01", 0.4)];
            }
            if [1, 5, 7, 11].contains(&k) {
                return vec![("spine_03", 0.5), ("neck_01", 0.25), (clav, 0.25)];
            }
        }
        if r >= 4 && [1, 5, 7, 11].contains(&k) {
            return mix(&[(0.85, &w), (0.15, &one(clav))]);
        }
        w
    };

    let arm_w = |chi_index: usize, side: &str| -> Weights {
        let (cl, ua) = (
            bone(format!("clavicle_{side}")),
            bone(format!("upperarm_{side}")),
        );
        // Root loop entries: 0-2 bottom (armpit), 3 back, 4-6 top, 7 front.
        if chi_index <= 2 {
            return vec![("spine_03", 0.5), (cl, 0.15), (ua, 0.35)];
        }
        if chi_index == 3 || chi_index == 7 {
            return vec![("spine_03", 0.35), (cl, 0.3), (ua, 0.35)];
        }
        vec![("spine_03", 0.1), (cl, 0.5), (ua, 0.4)]
    };

    let mut root_index: HashMap<(usize, usize), (usize, &str)> = HashMap::new();
    for sd in ["l", "r"] {
        let (k0, k1, k2) = arm_k(sd);
        for (i, rk) in [
            (4, k0),
            (4, k1),
            (4, k2),
            (5, k2),
            (6, k2),
            (6, k1),
            (6, k0),
            (5, k0),
        ]
        .into_iter()
        .enumerate()
        {
            root_index.insert(rk, (i, sd));
        }
    }
    let mut tid: Vec<Vec<usize>> = Vec::new();
    for (r, row) in t.iter().enumerate() {
        let mut ids = Vec::new();
        for (k, &q) in row.iter().enumerate() {
            if r == 5 && (k == 3 || k == 9) {
                ids.push(usize::MAX);
                continue;
            }
            if let Some(&(i, sd)) = root_index.get(&(r, k)) {
                ids.push(c.add(
                    q,
                    arm_w(i, sd),
                    "torso",
                    None,
                    Tags {
                        arm: 0.5,
                        blush: 0.0,
                    },
                ));
            } else {
                ids.push(c.add(q, tw(r, k), "torso", None, Tags::default()));
            }
        }
        tid.push(ids);
    }
    let crotch = c.add(
        v3(0.0, y_hip - (0.05 - 0.012 * yg) * h, -0.006 * h),
        vec![("pelvis", 0.4), ("thigh_l", 0.3), ("thigh_r", 0.3)],
        "torso",
        None,
        Tags::default(),
    );
    // Shoulder cap: the neck's sides.
    let mut cap: HashMap<&str, usize> = HashMap::new();
    for (sd, sx) in [("r", 1.0), ("l", -1.0)] {
        let id = c.add(
            v3(sx * r_n * 1.3, y_neck + 0.012 * h, zn - 0.1 * r_n),
            vec![
                ("spine_03", 0.35),
                ("neck_01", 0.3),
                (bone(format!("clavicle_{sd}")), 0.35),
            ],
            "torso",
            None,
            Tags::default(),
        );
        cap.insert(sd, id);
    }
    for r in 0..rows.len() - 1 {
        for k in 0..12 {
            let j = (k + 1) % 12;
            if (r == 4 || r == 5) && [2, 3, 8, 9].contains(&k) {
                continue;
            }
            c.face(&[tid[r][k], tid[r][j], tid[r + 1][j], tid[r + 1][k]]);
        }
    }
    let t7 = tid[6].clone();
    c.face(&[t7[1], t7[2], t7[3], cap["r"]]);
    c.face(&[cap["r"], t7[3], t7[4], t7[5]]);
    c.face(&[t7[11], t7[10], t7[9], cap["l"]]);
    c.face(&[cap["l"], t7[9], t7[8], t7[7]]);

    // Neck: the shoulder cap's loop up to a ring under the jaw, then the head.
    let n0 = vec![
        t7[0], t7[1], cap["r"], t7[5], t7[6], t7[7], cap["l"], t7[11],
    ];
    c.neck_loop = n0.clone();
    let pr_head = pr.hh * h;
    let chin = h - pr_head;
    let y_n = chin - 0.01 * h + 0.005 * h * yg;
    let psi: Vec<f64> = (0..8).map(|i| radians(i as f64 * 45.0)).collect();
    // (Its middle a little in front of the neck's joint: the nape lies under the skull's base, in front
    // of the back of the head, not behind it.)
    let mut nh = ring_pts(
        v3(0.0, y_n, zn + NAPE * r_n),
        x,
        z,
        r_n,
        r_n * 0.95,
        r_n * 1.0,
        &psi,
        2.0,
    );
    for k in [3, 4, 5] {
        // The nape is higher than the throat.
        nh[k][1] += 0.012 * h;
    }
    let nw: Weights = vec![("neck_01", 0.55), ("Head", 0.45)];
    let nh_id = c.ring(&nh, Per::All(&nw), "neck", None, RingTags::default());
    c.band(&n0, &nh_id, &[]);
    let jaw = head_cage(&mut c, family, stage, id);
    c.join(&nh_id, &jaw, &[0, 2, 4, 5, 6, 7, 8, 10]);

    // Arms and hands.
    for sd in ["r", "l"] {
        let (k0, k1, k2) = arm_k(sd);
        let lp = vec![
            tid[4][k0], tid[4][k1], tid[4][k2], tid[5][k2], tid[6][k2], tid[6][k1], tid[6][k0],
            tid[5][k0],
        ];
        let chi: Vec<f64> = [135.0, 180.0, 225.0, 270.0, 315.0, 0.0, 45.0, 90.0]
            .map(radians)
            .to_vec();
        let nm = |n: &str| bone(format!("{n}_{sd}"));
        let (jsh, jel, jwr) = (pj(nm("upperarm")), pj(nm("lowerarm")), pj(nm("hand")));
        let d = unit(jwr - jsh);
        let u = unit(y - d * y.dot(&d));
        let upper = (jel - jsh).norm();
        let lower = (jwr - jel).norm();
        let rings: [(V3, f64, f64, Weights); 4] = [
            (
                jsh + d * (0.32 * upper),
                1.0,
                0.98,
                vec![(nm("upperarm"), 0.9), (nm("clavicle"), 0.1)],
            ),
            (
                jel,
                0.84,
                0.84,
                vec![(nm("upperarm"), 0.5), (nm("lowerarm"), 0.5)],
            ),
            (
                jel + d * (0.45 * lower),
                0.80,
                0.86,
                vec![(nm("lowerarm"), 1.0)],
            ),
            (
                jwr - d * (0.02 * lower),
                0.5,
                0.68,
                vec![(nm("lowerarm"), 0.45), (nm("hand"), 0.55)],
            ),
        ];
        let mut prev = lp;
        for (cc, ru, rf, w) in &rings {
            let pts: Vec<V3> = chi
                .iter()
                .map(|&a| {
                    cc + u * (r_a * ru * se(a.cos(), 2.2)) + z * (r_a * rf * se(a.sin(), 2.2))
                })
                .collect();
            let cur = c.ring(
                &pts,
                Per::All(w),
                "arm",
                None,
                RingTags {
                    arm: Per::All(1.0),
                    ..Default::default()
                },
            );
            c.band(&prev, &cur, &[]);
            prev = cur;
        }
        let wrist = prev;
        // Palm: two 10-rings (4 across from the back of the hand (-z) to the thumb side, 1 deep),
        // the ball of the thumb swelling on the palm side, then the knuckle ring: the same 5
        // columns plus a vertex at each side.
        let k1j = pj(nm("fingers_01"));
        let lpm = (k1j - jwr).norm();
        // (A small child's hand is a little wider, its fingers plumper and shorter.)
        let (wp, tp) = ((0.84 + 0.12 * yg) * lpm, 0.30 * lpm);
        let zs: Vec<f64> = [-0.5, -0.25, 0.0, 0.25, 0.5]
            .iter()
            .zip([0.9, 1.0, 1.0, 1.0, 0.92])
            .map(|(&a, m)| a * wp * m)
            .collect();
        let order: [(usize, f64); 10] = [
            (3, -1.0),
            (2, -1.0),
            (1, -1.0),
            (0, -1.0),
            (0, 1.0),
            (1, 1.0),
            (2, 1.0),
            (3, 1.0),
            (4, 1.0),
            (4, -1.0),
        ];
        // (The thumb's ball: palm-side offsets (towards the palm, forward) of columns 3 and 4.)
        let palm_ring = |tt: f64, scale: f64, ball: f64| -> Vec<V3> {
            let cc = jwr + d * (tt * lpm) - u * 0.06 * tp;
            order
                .iter()
                .map(|&(k, v)| {
                    let swell = if v < 0.0 && k >= 3 {
                        ball * [0.0, 0.0, 0.0, 0.6, 1.0][k] * lpm
                    } else {
                        0.0
                    };
                    cc + z * (zs[k] * scale + 0.5 * swell)
                        + u * ((tp / 2.0) * v * (if k == 0 || k == 4 { 0.85 } else { 1.0 }) - swell)
                })
                .collect()
        };
        let arm1 = || RingTags {
            arm: Per::All(1.0),
            ..Default::default()
        };
        let hand_w = one(nm("hand"));
        let p1 = c.ring(
            &palm_ring(0.32, 0.92, 0.05),
            Per::All(&hand_w),
            "hand",
            None,
            arm1(),
        );
        c.join(&wrist, &p1, &[0, 1, 2, 3, 5, 6, 7, 8]);
        let p2 = c.ring(
            &palm_ring(0.64, 1.0, 0.04),
            Per::All(&hand_w),
            "hand",
            None,
            arm1(),
        );
        c.band(&p1, &p2, &[8, 9]);
        // The knuckle ring: the back over the knuckles (about half hand, half fingers_01), the
        // palm's pads a little further out (more the fingers'), so the knuckles bend there;
        // between the fingers the back dips a little.
        let kn: Vec<V3> = {
            let cc = jwr - u * 0.02 * lpm;
            let (top, pad) = (0.11 * lpm, 0.15 * lpm);
            let mut out: Vec<V3> = order
                .iter()
                .map(|&(k, v)| {
                    let edge = if k == 0 || k == 4 { 0.85 } else { 1.0 };
                    if v > 0.0 {
                        cc + d * (1.0 * lpm)
                            + z * (zs[k] * KNUCKLE_SPREAD)
                            + u * (top * edge - if k == 0 || k == 4 { 0.0 } else { 0.012 * lpm })
                    } else {
                        cc + d * (1.08 * lpm) + z * (zs[k] * KNUCKLE_SPREAD) - u * (pad * edge)
                    }
                })
                .collect();
            // Order: palm side 3 .. 0, the pinky's side, back 0 .. 4, the index's side, palm side 4.
            out.insert(
                4,
                cc + d * (1.04 * lpm) + z * (zs[0] * KNUCKLE_SPREAD - 0.02 * lpm)
                    - u * (0.01 * lpm),
            );
            out.insert(
                10,
                cc + d * (1.04 * lpm) + z * (zs[4] * KNUCKLE_SPREAD + 0.02 * lpm)
                    - u * (0.01 * lpm),
            );
            out
        };
        let wk = |f: f64| -> Weights { vec![(nm("hand"), 1.0 - f), (nm("fingers_01"), f)] };
        let wks: Vec<Weights> = (0..12)
            .map(|i| {
                wk(if (5..10).contains(&i) {
                    0.45
                } else if i == 4 || i == 10 {
                    0.6
                } else {
                    0.7
                })
            })
            .collect();
        let p3 = c.ring(
            &kn,
            Per::Each(&wks.iter().collect::<Vec<_>>()),
            "hand",
            None,
            arm1(),
        );
        c.join(&p2, &p3, &[0, 1, 2, 3, 5, 6, 7, 8, 9, 11]);
        // Fingers, pinky (-z) to index: each leaves the knuckle ring between two of its columns,
        // a 6-ring (back two, the side towards the thumb, palm two, the other side) whose sides
        // between two fingers are one vertex in the crotch (towards the palm, past the knuckles),
        // so neighbours share the edges down into it and stand apart beyond it. Straight from
        // the knuckle to a ring just short of the middle joint (fingers_01), and a ring before
        // the tip (fingers_02) turned a little about that joint towards the palm (the rest pose's
        // curl), narrowing; a 6-sided cap. The middle joint is fingers_02's (rig.rs), so a fist
        // bends at the knuckle and there, and the outer band, about as long as the inner one,
        // folds its tip back onto the palm (`rigid_fingertips` keeps it round on level 1).
        // (`ki`: the knuckle ring's index of column k, back or palm;
        // its sides are 4 and 10.)
        let ki = |k: usize, v: f64| {
            let i = order.iter().position(|&(a, b)| a == k && b == v).unwrap();
            i + usize::from(i >= 4) + usize::from(i >= 9)
        };
        let k2j = pj(nm("fingers_02"));
        let (pivot_d, pivot_u) = ((k2j - jwr).dot(&d), (k2j - jwr).dot(&u));
        let chub = 1.0 + 0.15 * yg;
        let wc = wk(0.9);
        let crotch: Vec<usize> = (1..4)
            .map(|k| {
                c.add(
                    jwr + d * (1.08 * lpm) + z * (zs[k] * KNUCKLE_SPREAD) - u * (0.10 * lpm),
                    wc.clone(),
                    "hand",
                    None,
                    Tags {
                        arm: 1.0,
                        blush: 0.0,
                    },
                )
            })
            .collect();
        let w01 = one(nm("fingers_01"));
        let w02 = one(nm("fingers_02"));
        let (d0, u0) = (d, u);
        for fi in 0..4 {
            let side = |k: isize| {
                if k < 0 {
                    p3[4]
                } else if k > 2 {
                    p3[10]
                } else {
                    crotch[k as usize]
                }
            };
            let loop_f = vec![
                p3[ki(fi, 1.0)],
                p3[ki(fi + 1, 1.0)],
                side(fi as isize),
                p3[ki(fi + 1, -1.0)],
                p3[ki(fi, -1.0)],
                side(fi as isize - 1),
            ];
            // (Half width, half depth; distal length past the joint, in palm lengths. The palm side is fuller.)
            let (hw, hd) = (
                [0.080, 0.089, 0.091, 0.089][fi] * chub,
                [0.095, 0.105, 0.107, 0.105][fi] * chub,
            );
            let tip = [0.25, 0.29, 0.31, 0.29][fi] * (1.0 - 0.3 * yg);
            let zf = (zs[fi] + zs[fi + 1]) / 2.0 * KNUCKLE_SPREAD;
            // (The fingers fan a little, so the gaps between them widen towards the tips, where the
            // silhouette shows them; at rest a hand's fingers are not parallel.)
            let spread = (fi as f64 - 1.5) * 0.07;
            // (A 6-ring round cc facing ax: half width, half depth on the back and on the palm side.)
            let hex = |cc: V3, ax: V3, up: V3, hw: f64, back: f64, palm: f64| -> Vec<V3> {
                let sd = unit(up.cross(&ax)) * sign(up.cross(&ax).dot(&z));
                [
                    (-0.5, 0.87),
                    (0.5, 0.87),
                    (1.0, 0.0),
                    (0.5, -0.87),
                    (-0.5, -0.87),
                    (-1.0, 0.0),
                ]
                .iter()
                .map(|&(a, b): &(f64, f64)| {
                    cc + sd * (a * hw * lpm) + up * (b * if b > 0.0 { back } else { palm } * lpm)
                })
                .collect()
            };
            let straight = jwr + d * pivot_d + u * pivot_u + z * (zf + spread * (pivot_d - lpm));
            // A hand at rest curls more towards the little finger: each finger turned a little further in at
            // its knuckle (`knuckle`, rad, the pinky to the index) and at its middle joint (`rho`). Four fingers
            // curled alike are one flat slab, a mitten; the bones turn them all together from here, so the
            // step from finger to finger stays in every pose.
            let knuckle: f64 = [0.2, 0.13, 0.07, 0.0][fi];
            let (d, u) = (
                d * knuckle.cos() - u * knuckle.sin(),
                u * knuckle.cos() + d * knuckle.sin(),
            );
            let at = jwr + d0 * lpm + u0 * pivot_u + z * (zf + spread * (pivot_d - lpm));
            let pivot = at + d * (straight - at).dot(&d0) + u * (straight - at).dot(&u0);
            let a = hex(
                pivot - d * (0.04 * lpm),
                d,
                u,
                hw * 0.98,
                hd * 1.05,
                hd * 1.5,
            );
            let ra = c.ring(&a, Per::All(&w01), "hand", None, arm1());
            c.band(&loop_f, &ra, &[]);
            let rho: f64 = [0.42, 0.34, 0.27, 0.2][fi];
            let (ad, au) = (d * rho.cos() - u * rho.sin(), u * rho.cos() + d * rho.sin());
            let b = hex(
                pivot + ad * (tip * lpm) + z * (spread * tip * lpm),
                ad,
                au,
                hw * 0.90,
                hd * 0.85,
                hd * 1.15,
            );
            let rb = c.ring(&b, Per::All(&w02), "hand", None, arm1());
            c.band(&ra, &rb, &[]);
            c.cap(&rb);
        }
        // Thumb: from the palm between the two rings, round its thumb side and the palm side next
        // to it (the ball of the thumb), a 6-ring at its knuckle (thumb_02) and one before the
        // tip, narrowing like a finger; a 6-sided cap. Each ring is the base loop turned from its
        // own plane to facing along the thumb (wrist / tip of the hand, back / palm keep their
        // sides), so the bands don't twist.
        let tl = vec![p1[8], p1[9], p1[0], p2[0], p2[9], p2[8]];
        let base: Vec<V3> = tl.iter().map(|&i| c.p[i]).collect();
        let m = mean_rows(&base);
        let up = unit((base[0] - base[2]) - d * (base[0] - base[2]).dot(&d));
        let out = unit(d.cross(&up));
        let out = out * sign(out.dot(&(z - u)));
        let t2 = pj(nm("thumb_02"));
        let tip = unit(d * 0.85 - u * 0.45 + z * 0.22);
        let mut prev_t = tl;
        let thumb: [(V3, V3, f64, f64, Weights); 2] = [
            (
                t2,
                unit(unit(t2 - m) + tip),
                0.15,
                0.135,
                vec![(nm("thumb_01"), 0.5), (nm("thumb_02"), 0.5)],
            ),
            (
                t2 + tip * (0.42 * lpm),
                tip,
                0.11,
                0.1,
                vec![(nm("thumb_02"), 1.0)],
            ),
        ];
        for (cc, ax, thw, tht, w) in &thumb {
            let r = turn(out, *ax);
            let (rd, ru) = (r * d, r * up);
            let pts: Vec<V3> = [
                (-0.5, 0.87),
                (-1.0, 0.0),
                (-0.5, -0.87),
                (0.5, -0.87),
                (1.0, 0.0),
                (0.5, 0.87),
            ]
            .iter()
            .map(|&(a, b)| cc + (rd * (a * thw) + ru * (b * tht)) * lpm)
            .collect();
            let cur = c.ring(&pts, Per::All(w), "hand", None, arm1());
            c.band(&prev_t, &cur, &[]);
            prev_t = cur;
        }
        c.cap(&prev_t);
    }

    // Legs and feet.
    for sd in ["r", "l"] {
        let (lp, psi): (Vec<usize>, Vec<f64>) = if sd == "r" {
            let mut l = tid[0][0..7].to_vec();
            l.push(crotch);
            (
                l,
                [-45.0, 0.0, 45.0, 90.0, 135.0, 180.0, -135.0, -90.0]
                    .map(radians)
                    .to_vec(),
            )
        } else {
            let mut l = tid[0][6..12].to_vec();
            l.extend([tid[0][0], crotch]);
            (
                l,
                [135.0, 180.0, -135.0, -90.0, -45.0, 0.0, 45.0, 90.0]
                    .map(radians)
                    .to_vec(),
            )
        };
        let nm = |n: &str| bone(format!("{n}_{sd}"));
        let (jt, jk, ja) = (pj(nm("thigh")), pj(nm("calf")), pj(nm("foot")));
        let rings: [(V3, f64, f64, f64, Weights); 4] = [
            (
                jt + (jk - jt) * 0.24,
                1.0,
                0.98,
                1.06,
                vec![(nm("thigh"), 0.92), ("pelvis", 0.08)],
            ),
            (
                jt + (jk - jt) * 0.86,
                0.68,
                0.70,
                0.64,
                vec![(nm("thigh"), 0.75), (nm("calf"), 0.25)],
            ),
            (
                jk + (ja - jk) * 0.22,
                0.64,
                0.64,
                0.74,
                vec![(nm("calf"), 0.85), (nm("thigh"), 0.15)],
            ),
            (
                ja + (jk - ja) * 0.1,
                0.42,
                0.42,
                0.44,
                vec![(nm("calf"), 0.5), (nm("foot"), 0.5)],
            ),
        ];
        let mut prev = lp;
        let sx = if sd == "r" { 1.0 } else { -1.0 };
        for (k, (cc, a, bf, bb, w)) in rings.iter().enumerate() {
            let mut pts = ring_pts(*cc, x, z, r_th * a, r_th * bf, r_th * bb, &psi, 2.1);
            if k == 0 {
                // The thigh's top stops short of the midline (a baby's plump thighs would cross
                // it): the strip between it and the crotch would fold over when the legs spread.
                for q in &mut pts {
                    q[0] = sx * (sx * q[0]).max(CROTCH_GAP * r_th);
                }
            }
            let cur = c.ring(&pts, Per::All(w), "leg", None, RingTags::default());
            c.band(&prev, &cur, &[]);
            prev = cur;
        }
        // The foot: rings turning from the ankle to point forward, flat underneath, the toes past the
        // ball's joint and a round tip. A real foot's length and breadth (0.15 and 0.055 of the height;
        // it ended at the ball, a seventh shorter and a fifth wider: a clog, and the shoes with it).
        let jb = pj(nm("ball"));
        let fl = jb[2] - ja[2];
        let fw = 0.285 * fl * (if female { 0.94 } else { 1.0 });
        let fx = ja[0];
        let feet: [(V3, f64, f64, f64, f64, Weights); 3] = [
            (
                v3(fx, 0.5 * ja[1], ja[2] - 0.02 * fl),
                45.0,
                fw * 0.92,
                0.38 * fl,
                0.38 * fl,
                one(nm("foot")),
            ),
            (
                v3(fx, 0.42 * ja[1], jb[2] - 0.2 * fl),
                86.0,
                fw * 1.0,
                0.29 * fl,
                0.30 * fl,
                vec![(nm("ball"), 0.5), (nm("foot"), 0.5)],
            ),
            // (The toes: lower and a little narrower.)
            (
                v3(fx, 0.3 * ja[1], jb[2] + 0.1 * fl),
                90.0,
                fw * 0.9,
                0.2 * fl,
                0.22 * fl,
                one(nm("ball")),
            ),
        ];
        for (cc, al, a, bt, bb, w) in &feet {
            let al = radians(*al);
            let vv = v3(0.0, al.sin(), al.cos());
            let mut pts = ring_pts(*cc, x, vv, *a, *bt, *bb, &psi, 2.6);
            for q in &mut pts {
                q[1] = q[1].max(0.004 * h);
            }
            let cur = c.ring(&pts, Per::All(w), "foot", None, RingTags::default());
            c.band(&prev, &cur, &[]);
            prev = cur;
        }
        let tip = c.add(
            v3(fx, 0.26 * ja[1], jb[2] + 0.27 * fl),
            one(nm("ball")),
            "foot",
            None,
            Tags::default(),
        );
        for i in [1, 3, 5, 7] {
            c.face(&[prev[i], prev[(i + 1) % 8], prev[(i + 2) % 8], tip]);
        }
    }
    (c, p)
}

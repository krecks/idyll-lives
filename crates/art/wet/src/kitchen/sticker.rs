//! The retro mini fridge's die-cut unicorn sticker, drawn in the reference photo's pixels (1024² image,
//! the sticker's middle at (652, 515)), each layer a thin flat plate a little prouder than the last.

use super::M;
use artkit::materials::Mat;
use artkit::math::{PI, V3, cos, sin};
use artkit::mesh::Mesh;
use artkit::place::GameFrame;
use artkit::py::hypot;
use artkit::scene::{self, Obj};

const STICKER_PX: (f64, f64) = (652.0, 515.0);
const STICKER_SCALE: (f64, f64) = (0.0012, 0.001); // metres per pixel across (the photo sees the door at an angle) and up

/// Rounds a polygon (or an open line) by cutting its corners.
fn chaikin(pts: &[(f64, f64)], closed: bool, passes: usize) -> Vec<(f64, f64)> {
    let mut pts = pts.to_vec();
    for _ in 0..passes {
        let mut out = Vec::new();
        let n = pts.len();
        for i in 0..(if closed { n } else { n - 1 }) {
            let ((ax, ay), (bx, by)) = (pts[i], pts[(i + 1) % n]);
            out.extend([
                (0.75 * ax + 0.25 * bx, 0.75 * ay + 0.25 * by),
                (0.25 * ax + 0.75 * bx, 0.25 * ay + 0.75 * by),
            ]);
        }
        pts = if closed {
            out
        } else {
            [vec![pts[0]], out, vec![pts[n - 1]]].concat()
        };
    }
    pts
}

/// A tapered strand round a centre line (pixels): its outline, for `plate`.
fn ribbon(line: &[(f64, f64)], width: f64) -> Vec<(f64, f64)> {
    let line = chaikin(line, false, 1);
    let (mut left, mut right) = (Vec::new(), Vec::new());
    let n = line.len();
    for (i, &(x, y)) in line.iter().enumerate() {
        let ((ax, ay), (bx, by)) = (line[i.saturating_sub(1)], line[(i + 1).min(n - 1)]);
        let (dx, dy) = (bx - ax, by - ay);
        let k = hypot(dx, dy);
        let k = if k != 0.0 { k } else { 1.0 };
        let w = width / 2.0 * (0.25 + 0.75 * sin(PI * i as f64 / (n - 1) as f64));
        left.push((x - dy / k * w, y + dx / k * w));
        right.push((x + dy / k * w, y - dx / k * w));
    }
    right.reverse();
    [left, right].concat()
}

fn oval(cx: f64, cy: f64, rx: f64, ry: f64, n: usize) -> Vec<(f64, f64)> {
    (0..n)
        .map(|k| {
            (
                cx + rx * cos(2.0 * PI * k as f64 / n as f64),
                cy + ry * sin(2.0 * PI * k as f64 / n as f64),
            )
        })
        .collect()
}

/// A flat plate of the sticker, its outline in photo pixels, `layer` × 1.2 mm proud of the door face
/// `at` (game x, height, depth of the sticker's middle). Front and edges only: its back is on the door.
#[allow(clippy::too_many_arguments)] // (as the script called it)
fn plate(
    f: &GameFrame,
    name: &str,
    px: &[(f64, f64)],
    at: (f64, f64, f64),
    layer: i32,
    material: Mat,
    round_passes: usize,
) -> Obj {
    let px = if round_passes != 0 {
        chaikin(px, true, round_passes)
    } else {
        px.to_vec()
    };
    let ((ox, oy), (su, sv)) = (STICKER_PX, STICKER_SCALE);
    let (x0, y0, z0) = at;
    let z = z0 + 0.0008 + 0.0012 * layer as f64; // the border's back just into the door, each layer 1.2 mm on
    // the viewer's right is game -x
    let pts: Vec<(f64, f64)> = px
        .iter()
        .map(|&(u, v)| (x0 - (u - ox) * su, y0 - (v - oy) * sv))
        .collect();
    let n = pts.len();
    let verts: Vec<V3> = pts
        .iter()
        .map(|&(x, y)| f.at(x, y, z - 0.0016))
        .chain(pts.iter().map(|&(x, y)| f.at(x, y, z)))
        .map(V3::from)
        .collect();
    let mut faces: Vec<Vec<u32>> = vec![(n as u32..2 * n as u32).collect()];
    for i in 0..n {
        let j = (i + 1) % n;
        faces.push(vec![i as u32, j as u32, (n + j) as u32, (n + i) as u32]);
    }
    let mut mesh = Mesh::from_pydata(verts, &faces);
    if mesh.face_normal(0).y > 0.0 {
        // face the front (-y), the edges outward with it
        mesh.flip_normals();
    }
    mesh.set_material(Some(material));
    scene::add(name, mesh)
}

/// The unicorn sticker on the mini fridge's door, as in the photo: a white die-cut border round a
/// unicorn's head in profile, facing the viewer's right, its pink mane streaked blue and lilac.
pub fn unicorn(f: &GameFrame, at: (f64, f64, f64)) {
    let px = |p: &[(i32, i32)]| -> Vec<(f64, f64)> {
        p.iter().map(|&(u, v)| (u as f64, v as f64)).collect()
    };
    plate(
        f,
        "uni_border",
        &px(&[
            (748, 360),
            (738, 400),
            (730, 426),
            (744, 450),
            (754, 480),
            (762, 508),
            (770, 530),
            (766, 554),
            (748, 568),
            (726, 570),
            (710, 566),
            (704, 582),
            (708, 610),
            (716, 648),
            (692, 666),
            (650, 674),
            (610, 674),
            (582, 662),
            (558, 632),
            (544, 595),
            (540, 555),
            (546, 514),
            (558, 476),
            (576, 446),
            (598, 420),
            (622, 400),
            (646, 386),
            (662, 396),
            (678, 392),
            (692, 410),
            (706, 418),
            (726, 384),
        ]),
        at,
        0,
        M.uni_white,
        1,
    );
    // the mane behind the head, and the head and neck
    plate(
        f,
        "uni_mane",
        &px(&[
            (652, 402),
            (668, 412),
            (680, 428),
            (668, 452),
            (656, 490),
            (648, 535),
            (646, 580),
            (652, 625),
            (664, 656),
            (640, 664),
            (612, 664),
            (588, 652),
            (566, 624),
            (553, 590),
            (550, 555),
            (555, 518),
            (566, 484),
            (584, 455),
            (604, 431),
            (628, 413),
        ]),
        at,
        1,
        M.uni_pink,
        1,
    );
    plate(
        f,
        "uni_head",
        &px(&[
            (700, 428),
            (726, 436),
            (740, 458),
            (749, 486),
            (756, 512),
            (761, 532),
            (756, 548),
            (742, 558),
            (724, 559),
            (710, 553),
            (697, 561),
            (691, 583),
            (696, 613),
            (704, 648),
            (680, 657),
            (664, 656),
            (652, 625),
            (646, 580),
            (648, 535),
            (656, 490),
            (668, 452),
            (682, 432),
        ]),
        at,
        1,
        M.uni_face,
        1,
    );
    // streaks through the mane
    #[allow(clippy::type_complexity)]
    // (the script's tuples: a centre line, its width, its material)
    let streaks: [(&[(i32, i32)], f64, Mat); 4] = [
        (
            &[
                (640, 420),
                (610, 452),
                (590, 492),
                (582, 540),
                (588, 590),
                (606, 634),
            ],
            9.0,
            M.uni_blue,
        ),
        (
            &[
                (660, 432),
                (636, 472),
                (621, 520),
                (619, 570),
                (628, 618),
                (642, 652),
            ],
            8.0,
            M.uni_lilac,
        ),
        (
            &[(624, 420), (598, 456), (576, 506), (571, 560), (582, 612)],
            5.0,
            M.uni_white,
        ),
        (
            &[(667, 458), (650, 502), (640, 552), (641, 604), (651, 642)],
            6.0,
            M.uni_rose,
        ),
    ];
    for (i, (line, w, mat)) in streaks.into_iter().enumerate() {
        plate(
            f,
            &format!("uni_streak{i}"),
            &ribbon(&px(line), w),
            at,
            2,
            mat,
            0,
        );
    }
    // the ear, the horn with its spiral, the muzzle, nostril and eye
    plate(
        f,
        "uni_ear",
        &px(&[
            (650, 392),
            (664, 402),
            (678, 398),
            (686, 424),
            (670, 434),
            (657, 418),
        ]),
        at,
        2,
        M.uni_face,
        1,
    );
    plate(
        f,
        "uni_ear_in",
        &px(&[(658, 402), (667, 410), (674, 425), (664, 422)]),
        at,
        3,
        M.uni_pink,
        1,
    );
    let (base_l, base_r, tip) = ((708.0, 424.0), (731.0, 431.0), (746.0, 366.0));
    plate(f, "uni_horn", &[base_l, tip, base_r], at, 2, M.uni_white, 0);
    let lerp =
        |a: (f64, f64), b: (f64, f64), t: f64| (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
    for (i, t) in [0.12, 0.36, 0.60].into_iter().enumerate() {
        plate(
            f,
            &format!("uni_horn_stripe{i}"),
            &[
                lerp(base_l, tip, t),
                lerp(base_l, tip, t + 0.07),
                lerp(base_r, tip, t + 0.15),
                lerp(base_r, tip, t + 0.08),
            ],
            at,
            3,
            M.uni_lilac,
            0,
        );
    }
    plate(
        f,
        "uni_muzzle",
        &oval(744.0, 540.0, 15.0, 13.0, 12),
        at,
        2,
        M.uni_blush,
        0,
    );
    plate(
        f,
        "uni_nostril",
        &oval(752.0, 533.0, 3.5, 2.5, 8),
        at,
        3,
        M.uni_rose,
        0,
    );
    plate(
        f,
        "uni_eye",
        &px(&[
            (695, 481),
            (703, 474),
            (714, 475),
            (719, 483),
            (709, 489),
            (699, 487),
        ]),
        at,
        2,
        M.uni_dark,
        0,
    );
    plate(
        f,
        "uni_eye_lid",
        &ribbon(&px(&[(692, 476), (704, 469), (717, 472)]), 3.0),
        at,
        3,
        M.uni_rose,
        0,
    );
}

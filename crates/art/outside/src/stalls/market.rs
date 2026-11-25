//! The market stall: a table of fruit crates under a striped awning on four posts.

use super::common::{M, valance};
use artkit::forms::rounded_rect;
use artkit::json::float_repr;
use artkit::materials::Mat;
use artkit::math::radians;
use artkit::parts::{join, join_all, new_parts};
use artkit::place::GameFrame;
use artkit::rand::Random;
use artkit::scene::{Obj, meshes};
use artkit::{fbox, fcyl, fextrude, fsweep, revolve, settle, sphere};

/// An open slatted crate on the table, tilted to the front, heaped with fruit; joined and turned whole.
fn crate_(f: &GameFrame, name: &str, x: f64, fruit: Mat, rng: &mut Random) -> Obj {
    let (y0, z0) = (0.84, 0.05);
    let parts = new_parts(|| {
        fbox!(
            f,
            format!("{name}_floor"),
            0.36,
            0.012,
            0.5,
            (x, y0, z0),
            M.crate_
        );
        for s in [-1, 1] {
            let sf = s as f64;
            for (k, (yy, hh)) in [(0.035, 0.05), (0.1, 0.05)].into_iter().enumerate() {
                // two slats a side, a gap between
                fbox!(
                    f,
                    format!("{name}_side{s}{k}"),
                    0.012,
                    hh,
                    0.5,
                    (x + sf * 0.174, y0 + yy, z0),
                    M.crate_
                );
            }
            fbox!(
                f,
                format!("{name}_end{s}"),
                0.36,
                if s > 0 { 0.1 } else { 0.13 },
                0.012,
                (x, y0 + (if s > 0 { 0.05 } else { 0.065 }), z0 + sf * 0.244),
                M.crate_
            );
            for e in [-1, 1] {
                // corner posts
                fbox!(
                    f,
                    format!("{name}_post{s}{e}"),
                    0.022,
                    if e < 0 { 0.13 } else { 0.1 },
                    0.022,
                    (
                        x + sf * 0.163,
                        y0 + (if e < 0 { 0.065 } else { 0.05 }),
                        z0 + e as f64 * 0.233
                    ),
                    M.crate_
                );
            }
        }
        // the fruit: a heap, a low row and a high row between, each a little turned and jostled
        let lemon = fruit.name == M.lemon.name;
        let r = if lemon { 0.045 } else { 0.05 };
        let scale = if lemon {
            (1.25, 1.0, 0.9)
        } else {
            (1.0, 1.0, 0.9)
        };
        for i in 0..3 {
            for j in 0..3 {
                let fx =
                    x - 0.1 + i as f64 * 0.1 + 0.015 * (j % 2) as f64 + rng.uniform(-0.008, 0.008);
                let fz = z0 - 0.15 + j as f64 * 0.15 + rng.uniform(-0.01, 0.01);
                let fy = y0 + 0.085 + 0.012 * ((i + j) % 2) as f64;
                sphere!(
                    format!("{name}_fruit{i}{j}"),
                    r,
                    at = f.at(fx, fy, fz),
                    material = fruit,
                    scale = scale,
                    segments = 6,
                    rings = 4,
                    rot = (0.0, 0.0, rng.uniform(0.0, 180.0))
                );
            }
        }
        for i in 0..2 {
            // a second layer on top of the first
            for j in 0..2 {
                let (fx, fz) = (x - 0.05 + i as f64 * 0.1, z0 - 0.075 + j as f64 * 0.15);
                sphere!(
                    format!("{name}_top{i}{j}"),
                    r,
                    at = f.at(fx, y0 + 0.155, fz),
                    material = fruit,
                    scale = scale,
                    segments = 6,
                    rings = 4,
                    rot = (0.0, 0.0, rng.uniform(0.0, 180.0))
                );
            }
        }
        // a price card on a stick
        fbox!(
            f,
            format!("{name}_stick"),
            0.006,
            0.12,
            0.006,
            (x + 0.12, y0 + 0.17, z0 + 0.2),
            M.crate_
        );
        fbox!(
            f,
            format!("{name}_card"),
            0.08,
            0.05,
            0.004,
            (x + 0.12, y0 + 0.24, z0 + 0.2),
            M.chalk
        );
        fbox!(
            f,
            format!("{name}_price"),
            0.05,
            0.012,
            0.002,
            (x + 0.12, y0 + 0.24, z0 + 0.203),
            M.slate
        );
    });
    let crate_ = join(&parts, name, Some(f.at(x, y0, z0)));
    crate_.set_rotation_euler_axis(0, radians(10.0)) // its front down, so the fruit shows
}

pub fn market_stall() -> Vec<Obj> {
    let f = GameFrame::new(0.0);
    let mut rng = Random::new(5);
    new_parts(|| {
        // the table: a top of four planks, an apron, a slatted lower shelf
        for k in 0..4 {
            fbox!(
                f,
                format!("plank{k}"),
                1.84,
                0.05,
                0.196,
                (0, 0.8, 0.02 - 0.3 + 0.2 * k as f64),
                M.board,
                bevel = 0.005,
                bevel_segments = 1
            );
        }
        fbox!(f, "apron_front", 1.7, 0.1, 0.03, (0, 0.725, 0.4), M.timber);
        fbox!(f, "apron_back", 1.7, 0.1, 0.03, (0, 0.725, -0.36), M.timber);
        for k in 0..4 {
            fbox!(
                f,
                format!("shelf{k}"),
                1.7,
                0.03,
                0.13,
                (0, 0.2, 0.02 - 0.225 + 0.15 * k as f64),
                M.timber
            );
        }
        for z in [-0.38, 0.38] {
            crate::boxed(fbox!(
                f,
                format!("shelf_rail{}", float_repr(z)),
                1.76,
                0.04,
                0.04,
                (0, 0.17, z),
                M.timber
            ));
        }
        for x in [-0.88, 0.88] {
            for (z, h) in [(-0.4, 2.1), (0.4, 1.9)] {
                crate::boxed(fbox!(
                    f,
                    format!("post{}{}", float_repr(x), float_repr(z)),
                    0.06,
                    h,
                    0.06,
                    (x, h / 2.0, z),
                    M.timber,
                    bevel = 0.006,
                    bevel_segments = 1
                ));
            }
            // knee braces under the beams
            let sign = if x > 0.0 { 1 } else { -1 };
            for (z, y) in [(0.4, 1.84), (-0.4, 2.04)] {
                crate::boxed(fbox!(
                    f,
                    format!("brace{}{}", float_repr(x), float_repr(z)),
                    0.04,
                    0.2,
                    0.04,
                    (x - 0.08 * sign as f64, y - 0.07, z),
                    M.timber,
                    rot = (0, 45 * sign, 0)
                ));
            }
        }
        fbox!(
            f,
            "beam_front",
            1.82,
            0.06,
            0.05,
            (0, 1.87, 0.4),
            M.timber,
            bevel = 0.006,
            bevel_segments = 1
        );
        fbox!(
            f,
            "beam_back",
            1.82,
            0.06,
            0.05,
            (0, 2.07, -0.4),
            M.timber,
            bevel = 0.006,
            bevel_segments = 1
        );
        let mut mats = Vec::new();
        for i in 0..7 {
            let x = -0.92 + 0.263 * i as f64 + 0.1315;
            let m = if i % 2 == 0 {
                M.awning_green
            } else {
                M.awning_white
            };
            mats.push(m);
            fbox!(
                f,
                format!("stripe{i}"),
                0.263,
                0.02,
                1.0,
                (x, 2.03, 0.02),
                m,
                rot = (12, 0, 0)
            );
        }
        valance(&f, "flap", -0.9205, 0.9205, 1.93, 0.5, 0.1, &mats, 1);
        // a chalk board hung from the front beam: a framed slate, chalked lines and a drawn apple
        for dx in [-0.14, 0.14] {
            fsweep!(
                f,
                format!("string{}", float_repr(dx)),
                &[(0.5 + dx, 1.84, 0.4), (0.5 + dx * 0.9, 1.72, 0.4)],
                0.004,
                M.chalk,
                verts = 4
            );
        }
        fbox!(f, "board", 0.34, 0.18, 0.016, (0.5, 1.62, 0.4), M.slate);
        artkit::bevel::eased(fextrude!(
            f,
            "board_frame",
            &rounded_rect(0.38, 0.22, 0.006, 1),
            0.024,
            (0.5, 1.62, 0.4),
            M.timber,
            holes = [rounded_rect(0.34, 0.18, 0.004, 1)]
        ));
        for (k, (w, dy)) in [(0.18, 0.04), (0.12, 0.0), (0.16, -0.045)]
            .into_iter()
            .enumerate()
        {
            fbox!(
                f,
                format!("chalk{k}"),
                w,
                0.012,
                0.004,
                (0.46 - (0.18 - w) / 2.0, 1.62 + dy, 0.41),
                M.chalk
            );
        }
        fcyl!(
            f,
            "chalk_apple",
            0.022,
            0.004,
            (0.62, 1.625, 0.41),
            M.tomato,
            verts = 8,
            rot = (90, 0, 0)
        );
        // a set of scales on the table's right end, paper bags beside them
        revolve!(
            "scale_base",
            &[
                (0.0, 0.0),
                (0.06, 0.0),
                (0.06, 0.03),
                (0.035, 0.05),
                (0.018, 0.18),
                (0.0, 0.18)
            ],
            at = f.at(0.8, 0.825, -0.31),
            material = M.steel,
            segments = 10
        );
        artkit::bevel::ease(
            revolve!(
                "scale_pan",
                &[
                    (0.0, 0.18),
                    (0.08, 0.18),
                    (0.1, 0.21),
                    (0.092, 0.212),
                    (0.075, 0.192),
                    (0.0, 0.192)
                ],
                at = f.at(0.8, 0.825, -0.31),
                material = M.steel,
                segments = 12
            ),
            50.0,
            2,
        );
        fcyl!(
            f,
            "scale_dial",
            0.04,
            0.01,
            (0.8, 0.94, -0.268),
            M.chalk,
            verts = 10,
            rot = (90, 0, 0)
        );
        for k in 0..3 {
            fbox!(
                f,
                format!("bag{k}"),
                0.12,
                0.012,
                0.16,
                (0.84, 0.832 + 0.012 * k as f64, -0.33),
                M.crate_,
                rot = (0, 5 * k, 0)
            );
        }
        // spare crates under the table, slatted
        for (k, x) in [-0.6, -0.32].into_iter().enumerate() {
            for (j, y) in [0.235, 0.315].into_iter().enumerate() {
                fbox!(
                    f,
                    format!("spare{k}{j}"),
                    0.24,
                    0.05,
                    0.42,
                    (x, y, -0.12),
                    M.crate_
                );
            }
        }
        for (k, (x, m)) in [
            (-0.6, M.tomato),
            (-0.2, M.orange),
            (0.2, M.apple),
            (0.6, M.lemon),
        ]
        .into_iter()
        .enumerate()
        {
            crate_(&f, &format!("crate{k}"), x, m, &mut rng);
        }
    });
    join_all("marketStall");
    settle!(centre = false);
    meshes()
}

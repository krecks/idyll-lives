//! A moulding.
//!
//! Split from `kit/joinery.rs` (see `mod.rs`).

use super::*;

/// A moulding run round a closed `outline` with mitred corners: a picture or mirror frame, a panel's bead, a
/// plinth's or a cornice's band. `outline` is game (x, height) points drawn at depth `at` (`Facing::Front`), or
/// (x, depth) points at height `at` (`Facing::Top`); `profile` is the moulding's closed cross-section
/// [(inset, out), ...]: `inset` metres in from the outline, `out` metres forward of `at` (front) or up from it
/// (top). The corners stay square however wide the profile (a sweep pinches them).
pub fn moulding(
    frame: &GameFrame,
    name: impl AsRef<str>,
    outline: &[(f64, f64)],
    profile: &[(f64, f64)],
    material: Mat,
    o: Moulding,
) -> Obj {
    let n = outline.len();
    let area =
        py::sum((0..n).map(|i| {
            outline[i].0 * outline[(i + 1) % n].1 - outline[(i + 1) % n].0 * outline[i].1
        }));
    let turn = if area > 0.0 { 1.0 } else { -1.0 }; // inward is to the left of an anticlockwise outline
    let inward = |a: (f64, f64), b: (f64, f64)| {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let mut length = pow(dx * dx + dy * dy, 0.5);
        if length == 0.0 {
            length = 1.0;
        }
        (-dy / length * turn, dx / length * turn)
    };
    let m = profile.len();
    let mut vs: Vec<V3> = Vec::with_capacity(n * m);
    for i in 0..n {
        let p = outline[i];
        let (n0, n1) = (
            inward(outline[(i + n - 1) % n], p),
            inward(p, outline[(i + 1) % n]),
        );
        let k = 1.0 + n0.0 * n1.0 + n0.1 * n1.1;
        let (mx, my) = ((n0.0 + n1.0) / k, (n0.1 + n1.1) / k);
        for &(inset, out) in profile {
            let (u, v) = (p.0 + inset * mx, p.1 + inset * my);
            let at = if o.plane == Facing::Front {
                frame.at(u, v, o.at + out)
            } else {
                frame.at(u, o.at + out, v)
            };
            vs.push(at.into());
        }
    }
    let mut faces: Vec<Vec<u32>> = Vec::with_capacity(n * m);
    for i in 0..n {
        let (a, b) = (i * m, ((i + 1) % n) * m);
        for j in 0..m {
            let j2 = (j + 1) % m;
            faces.push(vec![
                (a + j) as u32,
                (a + j2) as u32,
                (b + j2) as u32,
                (b + j) as u32,
            ]);
        }
    }
    forms::object(
        name.as_ref(),
        vs,
        &faces,
        Some(material),
        true,
        o.angle,
        false,
    )
}

options! {
    /// `bench_end`'s options.
    BenchEnd { arm: Option<f64> = None, t: f64 = 0.026, w: f64 = 0.045, foot: f64 = 0.06, steps: usize = 3, corner: usize = 1, scrolls: bool = true }
}

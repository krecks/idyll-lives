//! The gate's check `edges` and `cargo review edges`: the hard edges a model shows unbevelled at the play camera.
//!
//! Nothing made has an edge sharp to the millimetre: a board's arris is eased, sheet metal is folded round, paint
//! and moulded plastic round every corner. Such an edge catches the light as a thin bright line. Two faces of a
//! mesh that just meet have none, and read as a drawn box. At the play camera a pixel is about 4 mm at its
//! nearest (250 px a metre), so a bevel of 4 mm is that line, one pixel wide.
//!
//! A hard edge is a mesh edge
//!
//! - shared by exactly two triangles once positions are welded at 0.1 mm (over the whole shape: a part's faces
//!   may be of two materials), wound the same way round;
//! - turning 60° or more between their faces, and convex (a corner sticking out, not a crease);
//! - split in the file at both ends: its two faces' corners there have normals 40° or more apart (artkit's
//!   `shapes::SHARP`), so a bevel shaded with hardened normals passes by construction, as does a chamfer (45°);
//! - with both its surfaces 12 mm or more wide across it (a narrower one is a board's edge, too thin to bevel),
//!   measured over the surface from the edge's middle until the surface ends at another such edge;
//! - on a piece (what its triangles are joined to) 10 cm or more across.
//!
//! Hard edges run on as one from each to the next that goes on within 30° of straight, so a round top's rim is one
//! edge and a box's corner ends three. A finding is such a run of which 5 cm or more shows. Of a hard edge, looked at
//! every 10 cm or so, what shows is what
//!
//! - doesn't look down (an underside: the middle of its faces' normals more than 17° under level) nor lies on the
//!   floor, under the 1 cm a house draws its floor at;
//! - is seen, pushed 3 mm out, from two or more of the play camera's views on its side (`views`: 12 round, at four
//!   heights between the camera's lowest and steepest) with no face that looks at the camera between: not inside a
//!   solid, nor in a closed cabinet;
//! - has both its faces open beside it: 3 mm in from the edge no other face lies on it or within 3 mm in front of
//!   it (a door on its carcass, a box against its neighbour), and it doesn't go into another solid before it is
//!   12 mm wide (a frame standing 5 mm proud of its panel).
//!
//! Leaves, bark, glass and what glows are not judged (`Mesh::edged`); glass and cut-out cards hide nothing
//! (`Mesh::opaque`).
//!
//! The rule reads plain meshes (`Mesh`), so it runs the same on a model file (`checkglb::meshes`), on geometry
//! the game builds and on a generator's meshes before they are written. It uses no function that differs from one
//! machine to the next (roots and divisions only), so a count is the same wherever it is taken.
//!
//! ```text
//! cargo review edges [--all] [--check] [model keys…]
//! ```

use std::collections::{BTreeMap, HashMap};
use std::hash::{BuildHasherDefault, Hasher};
mod hard;
mod main;
mod mesh;
mod report;
mod shape;
mod solid;
mod types;

pub use hard::*;
pub use main::*;
pub use mesh::*;
pub use report::*;
pub use solid::*;
pub use types::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::vec3::{cross, dot, length, sub, unit};

    /// A flat face of four corners, its own corners with its own normal, wound to look along `n`.
    fn quad(mesh: &mut Mesh, corners: [V3; 4], n: V3) {
        let at = mesh.positions.len() as u32;
        mesh.positions.extend(corners);
        mesh.normals.extend([n; 4]);
        let turned = dot(
            cross(sub(corners[1], corners[0]), sub(corners[2], corners[0])),
            n,
        ) < 0.0;
        for t in [[0, 1, 2], [0, 2, 3]] {
            let t = if turned { [t[0], t[2], t[1]] } else { t };
            mesh.triangles.push(t.map(|i| at + i));
        }
    }

    fn solid_mesh() -> Mesh {
        Mesh {
            part: "body".into(),
            material: "paint".into(),
            edged: true,
            opaque: true,
            ..Mesh::default()
        }
    }

    /// A box from `lo` to `hi`, flat shaded, without the sides named in `but` (0-5: -x, +x, -y, +y, -z, +z).
    pub(crate) fn cuboid_but(lo: V3, hi: V3, but: &[usize]) -> Mesh {
        let mut mesh = solid_mesh();
        for axis in 0..3 {
            for (side, at) in [lo[axis], hi[axis]].into_iter().enumerate() {
                if but.contains(&(axis * 2 + side)) {
                    continue;
                }
                let (u, v) = ((axis + 1) % 3, (axis + 2) % 3);
                let corner = |a: f64, b: f64| {
                    let mut p = [0.0; 3];
                    (p[axis], p[u], p[v]) = (at, a, b);
                    p
                };
                let mut n = [0.0; 3];
                n[axis] = if side == 0 { -1.0 } else { 1.0 };
                quad(
                    &mut mesh,
                    [
                        corner(lo[u], lo[v]),
                        corner(hi[u], lo[v]),
                        corner(hi[u], hi[v]),
                        corner(lo[u], hi[v]),
                    ],
                    n,
                );
            }
        }
        mesh
    }

    pub(crate) fn cuboid(lo: V3, hi: V3) -> Mesh {
        cuboid_but(lo, hi, &[])
    }

    /// The sides of an outline (x, z; going round with its inside on the left) drawn up from `y0` to `y1`, open at
    /// both ends; `normal(side, point)`: the normal of that side's corners at that point of the outline (None: the
    /// side's own, flat).
    fn prism(
        outline: &[[f64; 2]],
        y0: f64,
        y1: f64,
        normal: impl Fn(usize, usize) -> Option<V3>,
    ) -> Mesh {
        let mut mesh = solid_mesh();
        for i in 0..outline.len() {
            let j = (i + 1) % outline.len();
            let (a, b) = (outline[i], outline[j]);
            let flat = unit([b[1] - a[1], 0.0, a[0] - b[0]]);
            let at = mesh.positions.len() as u32;
            mesh.positions.extend([
                [a[0], y0, a[1]],
                [b[0], y0, b[1]],
                [b[0], y1, b[1]],
                [a[0], y1, a[1]],
            ]);
            let (na, nb) = (normal(i, i).unwrap_or(flat), normal(i, j).unwrap_or(flat));
            mesh.normals.extend([na, nb, nb, na]);
            let corners: Vec<V3> = mesh.positions[at as usize..].to_vec();
            let turned = dot(
                cross(sub(corners[1], corners[0]), sub(corners[2], corners[0])),
                flat,
            ) < 0.0;
            for t in [[0, 1, 2], [0, 2, 3]] {
                let t = if turned { [t[0], t[2], t[1]] } else { t };
                mesh.triangles.push(t.map(|k| at + k));
            }
        }
        mesh
    }

    /// A flat cap on an outline at height `y`, a fan from its middle, looking up or down.
    fn cap(mesh: &mut Mesh, outline: &[[f64; 2]], y: f64, up: bool) {
        let n = [0.0, if up { 1.0 } else { -1.0 }, 0.0];
        let middle = outline.iter().fold([0.0, y, 0.0], |m, p| {
            [
                m[0] + p[0] / outline.len() as f64,
                y,
                m[2] + p[1] / outline.len() as f64,
            ]
        });
        for i in 0..outline.len() {
            let (a, b) = (outline[i], outline[(i + 1) % outline.len()]);
            let at = mesh.positions.len() as u32;
            mesh.positions
                .extend([middle, [a[0], y, a[1]], [b[0], y, b[1]]]);
            mesh.normals.extend([n; 3]);
            let turned = dot(
                cross(sub([a[0], y, a[1]], middle), sub([b[0], y, b[1]], middle)),
                n,
            ) < 0.0;
            mesh.triangles.push(if turned {
                [at, at + 2, at + 1]
            } else {
                [at, at + 1, at + 2]
            });
        }
    }

    fn square(half: f64) -> Vec<[f64; 2]> {
        vec![[half, half], [-half, half], [-half, -half], [half, -half]]
    }

    fn lengths(edges: &[Edge]) -> Vec<f64> {
        edges
            .iter()
            .map(|e| (e.length * 1e4).round() / 1e4)
            .collect()
    }

    #[test]
    fn a_box_on_the_floor_shows_its_top_and_its_upright_edges() {
        // 30 cm wide, 20 cm high, 40 cm deep: not the four on the floor, which look down.
        let found = hard_edges(&[cuboid([-0.15, 0.0, -0.2], [0.15, 0.2, 0.2])]);
        assert_eq!(lengths(&found), [0.4, 0.4, 0.3, 0.3, 0.2, 0.2, 0.2, 0.2]);
        assert!(
            found
                .iter()
                .all(|e| e.part == "body" && e.material == "paint")
        );
        assert!(found[..4].iter().all(|e| (e.at[1] - 0.2).abs() < 1e-9));
        assert!(found[4..].iter().all(|e| e.along[1].abs() > 0.999));
        // Hung in the air its underside still doesn't count.
        let hung = hard_edges(&[cuboid([-0.15, 1.0, -0.2], [0.15, 1.2, 0.2])]);
        assert_eq!(hung.len(), 8);
    }

    #[test]
    fn a_chamfer_passes_and_so_does_an_edge_shaded_round() {
        // A post 30 cm square and 50 cm high, its four upright edges sharp: four findings.
        let sharp = prism(&square(0.15), 0.1, 0.6, |_, _| None);
        assert_eq!(lengths(&hard_edges(&[sharp])), [0.5; 4]);
        // Each cut back 4 mm, one segment: the turns are 45°, hardened normals or flat.
        let (h, c) = (0.15, 0.004);
        let cut = vec![
            [h, h - c],
            [h - c, h],
            [c - h, h],
            [-h, h - c],
            [-h, c - h],
            [c - h, -h],
            [h - c, -h],
            [h, c - h],
        ];
        let flat_of = |side: usize| {
            let (a, b) = (cut[side], cut[(side + 1) % 8]);
            unit([b[1] - a[1], 0.0, a[0] - b[0]])
        };
        // (Odd sides are the long flat ones; a chamfer's corners take the flat side's normal each end.)
        let hardened = prism(&cut, 0.1, 0.6, |side, point| {
            Some(if side % 2 == 1 {
                flat_of(side)
            } else if point == side {
                flat_of((side + 7) % 8)
            } else {
                flat_of(side + 1)
            })
        });
        assert_eq!(hard_edges(&[hardened]), []);
        assert_eq!(hard_edges(&[prism(&cut, 0.1, 0.6, |_, _| None)]), []);
        // The sharp post shaded smooth round its corners: not split, so not this rule's (its shading is wrong
        // another way).
        let round = prism(&square(0.15), 0.1, 0.6, |_, point| {
            let p = square(0.15)[point];
            Some(unit([p[0], 0.0, p[1]]))
        });
        assert_eq!(hard_edges(&[round]), []);
    }

    #[test]
    fn a_crease_is_not_an_edge() {
        // An L: five corners stick out, the one inside doesn't.
        let l = [
            [0.3, 0.0],
            [0.3, 0.15],
            [0.15, 0.15],
            [0.15, 0.3],
            [0.0, 0.3],
            [0.0, 0.0],
        ];
        let found = hard_edges(&[prism(&l, 0.1, 0.6, |_, _| None)]);
        assert_eq!(found.len(), 5);
        assert!(
            found
                .iter()
                .all(|e| (e.at[0] - 0.15).abs() > 1e-6 || (e.at[2] - 0.15).abs() > 1e-6)
        );
    }

    #[test]
    fn thin_short_and_small_things_pass() {
        let count = |lo: V3, hi: V3| hard_edges(&[cuboid(lo, hi)]).len();
        // A board 10 mm thick: its edge is too narrow to bevel. At 20 mm its top's four edges count (its
        // uprights are 2 cm long).
        assert_eq!(count([0.0, 0.5, 0.0], [0.5, 0.51, 0.3]), 0);
        assert_eq!(count([0.0, 0.5, 0.0], [0.5, 0.52, 0.3]), 4);
        // A block 8 cm across is a detail; one of 10 cm is a piece.
        assert_eq!(count([0.0, 0.5, 0.0], [0.08, 0.58, 0.08]), 0);
        assert_eq!(count([0.0, 0.5, 0.0], [0.1, 0.6, 0.1]), 8);
        // Edges of 4 cm on a piece of 30: only the long ones.
        assert_eq!(count([0.0, 0.5, 0.0], [0.3, 0.54, 0.04]), 2);
        // Lying on the floor, 1 cm high: under the floor a house draws.
        assert_eq!(count([0.0, 0.0, 0.0], [0.5, 0.012, 0.5]), 0);
    }

    #[test]
    fn a_covered_edge_passes() {
        let low = cuboid([0.0, 0.0, 0.0], [0.4, 0.3, 0.4]);
        // Under a wider box standing on it: its top edges are covered, its uprights show; the upper box's twelve
        // but its underside's four.
        let on = cuboid([-0.1, 0.3, -0.1], [0.5, 0.5, 0.5]);
        let found = hard_edges(&[low.clone(), on]);
        assert_eq!(found.len(), 4 + 8);
        assert!(
            !found
                .iter()
                .any(|e| (e.at[1] - 0.3).abs() < 1e-9 && e.along[1].abs() < 0.5 && e.length < 0.5)
        );
        // Beside a box of its size, face to face: the faces between them are no edges, and the top's edges run
        // on from one to the other.
        let beside = cuboid([0.4, 0.0, 0.0], [0.8, 0.3, 0.4]);
        assert_eq!(
            lengths(&hard_edges(&[low.clone(), beside])),
            [0.8, 0.8, 0.4, 0.4, 0.3, 0.3, 0.3, 0.3]
        );
        // Beside a shorter box that covers half of its side: half of that top edge shows.
        let half = cuboid([0.4, 0.0, 0.0], [0.8, 0.3, 0.2]);
        let found = hard_edges(&[low.clone(), half]);
        let edge = found
            .iter()
            .find(|e| {
                (e.at[0] - 0.4).abs() < 1e-9
                    && (e.at[1] - 0.3).abs() < 1e-9
                    && e.along[2].abs() > 0.5
            })
            .unwrap();
        assert!((edge.length - 0.2).abs() < 1e-9, "{}", edge.length);
        // A face 2 mm in front of its side covers it too (a door on its carcass), 2 cm off it doesn't.
        let count_at = |gap: f64| {
            let door = cuboid([0.4 + gap, 0.0, 0.0], [0.42 + gap, 0.3, 0.4]);
            hard_edges(&[low.clone(), door])
                .iter()
                .filter(|e| (e.at[0] - 0.4).abs() < 1e-9)
                .count()
        };
        assert_eq!((count_at(0.002), count_at(0.02)), (0, 3));
    }

    #[test]
    fn what_no_camera_sees_passes() {
        // A box in a chest with walls 5 cm thick: seen from nowhere. The lid off, from above; a chest of glass, as
        // if nothing were there.
        let inner = cuboid([-0.15, 0.1, -0.15], [0.15, 0.4, 0.15]);
        let chest = |lid: bool| {
            let mut parts = vec![
                cuboid([-0.5, 0.0, -0.5], [0.5, 0.05, 0.5]),
                cuboid([-0.5, 0.05, -0.5], [-0.45, 1.0, 0.5]),
                cuboid([0.45, 0.05, -0.5], [0.5, 1.0, 0.5]),
                cuboid([-0.45, 0.05, -0.5], [0.45, 1.0, -0.45]),
                cuboid([-0.45, 0.05, 0.45], [0.45, 1.0, 0.5]),
            ];
            if lid {
                parts.push(cuboid([-0.5, 1.0, -0.5], [0.5, 1.05, 0.5]));
            }
            parts
        };
        let count = |chest: Vec<Mesh>| {
            let meshes: Vec<Mesh> = std::iter::once(inner.clone()).chain(chest).collect();
            hard_edges(&meshes)
                .iter()
                .filter(|e| e.at.iter().all(|c| c.abs() < 0.42))
                .count()
        };
        assert_eq!(count(chest(true)), 0);
        assert_eq!(count(chest(false)), 8);
        let of = |edged: bool, opaque: bool| {
            chest(true)
                .into_iter()
                .map(|part| Mesh {
                    edged,
                    opaque,
                    ..part
                })
                .collect::<Vec<Mesh>>()
        };
        assert_eq!(count(of(false, false)), 8);
        // What isn't judged has no edges of its own, but hides (a hollow trunk: bark).
        assert_eq!(count(of(false, true)), 0);
    }

    #[test]
    fn a_round_rim_is_one_edge() {
        // A drum 24 round, 10 cm in radius, shaded smooth round its side: its top rim is one edge of 24 pieces,
        // each 2.6 cm; its bottom rim looks down.
        let ring: Vec<[f64; 2]> = (0..24)
            .map(|i| {
                let a = std::f64::consts::TAU * i as f64 / 24.0;
                [0.1 * a.cos(), 0.1 * a.sin()]
            })
            .collect();
        let mut drum = prism(&ring, 0.2, 0.5, |_, point| {
            Some(unit([ring[point][0], 0.0, ring[point][1]]))
        });
        cap(&mut drum, &ring, 0.5, true);
        cap(&mut drum, &ring, 0.2, false);
        let found = hard_edges(&[drum]);
        assert_eq!(found.len(), 1);
        let rim = 24.0 * 2.0 * 0.1 * (std::f64::consts::PI / 24.0).sin();
        assert!((found[0].length - rim).abs() < 1e-9, "{}", found[0].length);
        // Eight sides, each its own flat face: eight corners, and its rim eight edges of 7.7 cm.
        let eight: Vec<[f64; 2]> = ring.iter().step_by(3).copied().collect();
        let mut post = prism(&eight, 0.2, 0.5, |_, _| None);
        cap(&mut post, &eight, 0.5, true);
        cap(&mut post, &eight, 0.2, false);
        let found = hard_edges(&[post]);
        assert_eq!(found.len(), 8, "{:?}", lengths(&found));
        assert!(found.iter().all(|e| e.along[1].abs() < 0.5));
    }

    #[test]
    fn a_box_of_two_materials_is_one_shape() {
        // Its top another material's mesh: the edges between the two are found, by the first's name.
        let whole = cuboid([0.0, 0.5, 0.0], [0.3, 0.7, 0.3]);
        let mut top = Mesh {
            material: "top".into(),
            ..cuboid_but([0.0, 0.5, 0.0], [0.3, 0.7, 0.3], &[0, 1, 2, 4, 5])
        };
        let rest = cuboid_but([0.0, 0.5, 0.0], [0.3, 0.7, 0.3], &[3]);
        assert_eq!(
            top.triangles.len() + rest.triangles.len(),
            whole.triangles.len()
        );
        assert_eq!(hard_edges(&[rest.clone(), top.clone()]).len(), 8);
        // A top that isn't judged (a glowing panel) takes its edges with it.
        top.edged = false;
        assert_eq!(hard_edges(&[rest, top]).len(), 4);
    }

    #[test]
    fn a_point_is_inside_or_seen() {
        let solid = Solid::new(&[cuboid([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])]);
        let up = [0.0, 1.0, 0.0];
        assert!(solid.inside([0.5, 0.5, 0.5], up));
        // (Down a seam of its top's two triangles too.)
        assert!(solid.inside([0.25, 0.5, 0.25], up) && solid.inside([0.5, 0.999, 0.5], up));
        assert!(!solid.inside([0.5, 1.5, 0.5], up) && !solid.inside([1.5, 0.5, 0.5], up));
        assert!(solid.near([0.5, 1.002, 0.5], [0.0, -1.0, 0.0], 0.003));
        assert!(!solid.near([0.5, 1.02, 0.5], [0.0, -1.0, 0.0], 0.003));
        // From inside, its top hides the sky; from over it, nothing does; from under it, its faces do.
        assert!(solid.hides([0.5, 0.5, 0.5], up, None).is_some());
        assert_eq!(solid.hides([0.5, 1.5, 0.5], up, None), None);
        let hid = solid.hides([0.5, -0.5, 0.5], up, None);
        assert!(hid.is_some());
        assert!(solid.hides([0.4, -0.5, 0.6], up, hid).is_some());
        assert_eq!(views().len(), 48);
        assert!(
            views()
                .iter()
                .all(|v| (length(*v) - 1.0).abs() < 1e-12 && v[1] > 0.24 && v[1] < 0.96)
        );
    }

    #[test]
    fn the_report_names_each_edge() {
        let found = Found {
            key: "model.crate".into(),
            family: "yard".into(),
            triangles: 12,
            edges: hard_edges(&[cuboid([-0.15, 0.0, -0.2], [0.15, 0.2, 0.2])]),
        };
        let line = line(&found.key, &found.edges).unwrap();
        assert!(
            line.starts_with("8 hard edges of 5 cm or more show unbevelled at the play camera, 2.2 m (`cargo review edges model.crate` lists them; as built, z up): the longest body (paint) 0.40 m at ("),
            "{line}"
        );
        assert_eq!(super::line("model.none", &[]), None);
        // As built: a file's z is its depth turned round, its y its height.
        assert_eq!(
            place(&Edge {
                part: String::new(),
                material: String::new(),
                length: 0.4,
                at: [0.15, 0.2, -0.1],
                along: [0.0, 0.0, 1.0],
            }),
            "(0.15, 0.10, 0.20) along y"
        );
        let clean = Found {
            key: "model.ball".into(),
            family: "kids".into(),
            ..Found::default()
        };
        let text = report(&[found.clone(), clean.clone()], false, false);
        assert!(
            text.starts_with("2 models, 1 with hard edges of 5 cm or more that show unbevelled at the play camera: 8 edges, 2 m of them\n"),
            "{text}"
        );
        assert!(
            text.contains("| yard | 1 | 1 | 8 | 2.2 |\n| kids | 1 | 0 | 0 | 0.0 |"),
            "{text}"
        );
        assert!(
            text.contains("| model.crate | 12 | 8 | 2.2 | body (paint) 0.40 m at ("),
            "{text}"
        );
        let listed = report(&[found], false, true);
        assert_eq!(
            listed
                .lines()
                .filter(|l| l.starts_with("  body (paint) "))
                .count(),
            8,
            "{listed}"
        );
    }

    #[test]
    fn the_shipped_models_are_read() {
        let paths = crate::checkglb::Paths::repo();
        let found = survey(&paths, &[]).unwrap();
        assert!(found.len() > 550, "{} models", found.len());
        assert!(
            found
                .iter()
                .all(|f| f.triangles > 0 && !f.key.starts_with("tool."))
        );
        // The same edges each time, whatever the threads.
        let keys: Vec<String> = found.iter().take(40).map(|f| f.key.clone()).collect();
        let again = survey(&paths, &keys).unwrap();
        for (a, b) in again.iter().zip(&found) {
            assert_eq!((&a.key, &a.edges), (&b.key, &b.edges));
        }
        assert!(survey(&paths, &["model.noSuchThing".into()]).is_err());
    }
}

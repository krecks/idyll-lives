//! A prism: a slab between two copies of one convex outline.

use crate::forms;
use crate::materials::Mat;
use crate::math::V3;
use crate::scene::Obj;
use crate::shapes::SHARP;

/// A prism between the convex outline `near` and its copy `far` (the same points, moved along one direction): its
/// two ends and a side for each edge, flat, its normals made to face out. A gable, a slab, a sandwich pack.
pub fn prism(name: &str, near: Vec<V3>, far: Vec<V3>, material: Mat) -> Obj {
    let n = near.len() as u32;
    let mut faces: Vec<Vec<u32>> = vec![(0..n).collect(), (n..2 * n).rev().collect()];
    faces.extend((0..n).map(|i| vec![i, (i + 1) % n, n + (i + 1) % n, n + i]));
    let verts = near.into_iter().chain(far).collect();
    forms::object(name, verts, &faces, Some(material), false, SHARP, false)
}

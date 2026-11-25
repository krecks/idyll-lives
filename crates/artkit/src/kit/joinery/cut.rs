//! Cutting one object out of another.

use super::*;
use crate::bevel::EdgeOrder;
use crate::materials::Mat;
use crate::math::V3;
use crate::scene::Obj;
use crate::shapes::options;

/// Cuts `cutter`'s volume out of `obj` and takes the cutter away: what gamekit's `forms.cut` and the kitchen's
/// boolean modifiers (Blender's exact difference) made of the families' bodies, without a boolean. One of two
/// cuts, by how the cutter lies in the body's own box:
///
/// - it reaches out of the body on one side only: a recess pushed in through that side (an oven's mouth, a
///   fridge's inside), or a hole right through a wall there (a toy box's grip). The cutter is a box, plain or
///   bevelled, or any slab drawn across that side: what counts is its outline seen from there, which is cut out of
///   every face of the body it passes (a flat front, the strips of a lofted shell), each face keeping its place
///   among the body's; the cutter's own faces inside the body line the cut, turned inward;
/// - it covers the body on every side but one: the body is cut off flat at the cutter's face inside it and
///   closed there (half of a hot tub's cover), `edit::bisect_plane` and `edit::holes_fill`.
///
/// The body is cut as its mesh is: a bevel it is still to be given rounds the cut body afterwards, the mouth's
/// rim too (`forms.cut` on a bevelled box); `bevel::apply` first cuts the rounded body (the kitchen's
/// `_hollow`). The cutter is cut with as it would be written, its own bevel made. The body's UVs are made
/// afterwards, as any part's.
///
/// The points are Blender's (the body's own, the cutter's inside it, and where the outline's sides cross the
/// body's edges or its corners land on a face), and so are the faces, each with Blender's corners round it: a face
/// with the whole opening inside it becomes two, one between the opening and the face's nearest side, one round
/// the rest. The corner a face starts at is the kernel's, and so is the order of the points the cut makes (the
/// body's and the cutter's keep theirs, as Blender numbers them): the baked shading, which turns its rays by a
/// point's number, is a level or two of 32 off Blender's at a few points round the cut.
pub fn cut(obj: Obj, cutter: Obj, o: Cut) -> Obj {
    crate::bevel::apply(cutter);
    let into = obj
        .matrix_world()
        .inverted()
        .expect("a body is not flattened")
        * cutter.matrix_world();
    let tool = cutter.read(|c| c.mesh.clone());
    let pts: Vec<V3> = tool.verts.iter().map(|&p| into * p).collect();
    let (lo, hi) = obj.read(|b| extent(&b.mesh.verts));
    let (clo, chi) = extent(&pts);
    // the sides the cutter reaches out of the body on: (axis, -1 below or +1 above)
    let mut out: Vec<(usize, f32)> = Vec::new();
    for k in 0..3 {
        if clo[k] < lo[k] {
            out.push((k, -1.0));
        }
        if chi[k] > hi[k] {
            out.push((k, 1.0));
        }
    }
    let name = obj.name();
    match out.len() {
        1 => {
            let (k, side) = out[0];
            obj.edit(|part| {
                // (a bevel still to be given rounds the cut mesh, whose points are numbered anew)
                if let Some(bevel) = &mut part.bevel {
                    (bevel.order, bevel.sides_from) = (EdgeOrder::Corners, None);
                }
                part.mesh = carve(&part.mesh, &tool, &pts, k, side, o.lined, &name);
            });
        }
        5 => {
            let (k, side) = (0..3)
                .flat_map(|k| [(k, -1.0_f32), (k, 1.0)])
                .find(|s| !out.contains(s))
                .unwrap();
            // the cutter's face inside the body, looking back into the cutter: what lies that way goes
            let mut co = V3::ZERO;
            co[k] = if side > 0.0 { chi[k] } else { clo[k] };
            let mut no = V3::ZERO;
            no[k] = -side;
            obj.edit(|part| {
                // (a bevel still to be given rounds the cut mesh, whose points are numbered anew)
                if let Some(bevel) = &mut part.bevel {
                    (bevel.order, bevel.sides_from) = (EdgeOrder::Corners, None);
                }
            });
            let edges = crate::edit::bisect_plane(
                obj,
                co,
                no,
                crate::edit::Bisect {
                    clear_outer: true,
                    ..Default::default()
                },
            );
            let made = crate::edit::holes_fill(obj, &edges);
            obj.edit(|part| {
                let slot = (o.lined && !tool.mats.is_empty()).then(|| part.mesh.slot(tool.mats[0]));
                for f in made {
                    part.mesh.faces[f].smooth = false;
                    if let Some(slot) = slot {
                        part.mesh.faces[f].mat = slot;
                    }
                }
            });
        }
        _ => panic!(
            "{name}: a cutter reaches out of the body on one side (a recess, a hole through) or on all but one (a cut across), not on {}",
            out.len()
        ),
    }
    // (the cutter's materials join the body's, worn or not, as the modifier adds them)
    obj.edit(|part| {
        for &mat in &tool.mats {
            part.mesh.slot(mat);
        }
    });
    if !o.keep_cutter {
        cutter.remove();
    }
    obj
}

options! {
    /// `recessed_box`'s options: the box's `bevel`; `lining`, the recess's own material; `round_mouth`, the bevel
    /// made after the cut, so it rounds the mouth's rim and the recess's corners too (`forms.cut` on a bevelled
    /// box) and not the box alone.
    RecessedBox { bevel: f64 = 0.0, bevel_segments: usize = 2, lining: Option<Mat> = None, round_mouth: bool = false }
}

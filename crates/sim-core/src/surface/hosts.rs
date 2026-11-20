//! Hosts: which object a thing stands on, and whether it fits there.

use super::*;

/// Which way an object faces, turn included (radians; see `ObjectInstance::angle`).
pub(crate) fn angle(o: &ObjectInstance) -> f32 {
    o.angle()
}

/// Where an object's middle is, its offset included (tiles).
pub fn middle(content: &Content, o: &ObjectInstance) -> [f32; 2] {
    let c = o.centre(content);
    [
        c[0] + o.offset[0] as f32 / 100.0,
        c[1] + o.offset[1] as f32 / 100.0,
    ]
}

/// Things standing on surfaces near a top: their middles and radii.
pub(crate) fn clutter_on(
    content: &Content,
    objects: &[ObjectInstance],
    top: &Top,
    out: &mut Vec<([f32; 2], f32)>,
) {
    out.clear();
    for o in objects {
        let def = &content.objects[o.def];
        if def.layer != Layer::Surface {
            continue;
        }
        let p = middle(content, o);
        // Near enough to touch something on this top.
        let reach = top.half[0].max(top.half[1]) * std::f32::consts::SQRT_2 + def.radius + 0.5;
        if (p[0] - top.centre[0]).abs() < reach && (p[1] - top.centre[1]).abs() < reach {
            out.push((p, def.radius));
        }
    }
}

pub(crate) fn overlaps(discs: &[([f32; 2], f32)], p: [f32; 2], radius: f32) -> bool {
    discs
        .iter()
        .any(|(q, r)| (q[0] - p[0]).hypot(q[1] - p[1]) < r + radius - SLACK)
}

/// The object whose top covers tile `(x, z)`.
pub fn host_at(content: &Content, objects: &[ObjectInstance], x: i32, z: i32) -> Option<u32> {
    objects.iter().find_map(|o| {
        content.objects[o.def].surface?;
        let (w, d) = o.size(content);
        (x >= o.x && x < o.x + w && z >= o.z && z < o.z + d).then_some(o.id)
    })
}

/// The object a thing on the `Surface` layer stands on (None for anything else, or a thing
/// left with nothing under it).
pub fn host_of(content: &Content, objects: &[ObjectInstance], obj: &ObjectInstance) -> Option<u32> {
    host_among(
        content,
        objects.iter().filter_map(|o| Top::of(content, o)),
        obj,
    )
}

/// `host_of`, of `tops` (in id order).
pub(crate) fn host_among(
    content: &Content,
    mut tops: impl Iterator<Item = Top>,
    obj: &ObjectInstance,
) -> Option<u32> {
    if content.objects[obj.def].layer != Layer::Surface {
        return None;
    }
    let p = middle(content, obj);
    tops.find(|t| t.host != obj.id && t.covers(p))
        .map(|t| t.host)
}

/// Things on the `Surface` layer standing on `host`, in id order.
pub fn riders(content: &Content, objects: &[ObjectInstance], host: u32) -> Vec<u32> {
    let Some(top) = objects.get(host as usize).and_then(|o| Top::of(content, o)) else {
        return Vec::new();
    };
    objects
        .iter()
        .filter(|o| {
            content.objects[o.def].layer == Layer::Surface && top.covers(middle(content, o))
        })
        .map(|o| o.id)
        .collect()
}

/// Why a thing can't stand somewhere on a surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Misfit {
    /// Nothing with a top under it.
    NoTop,
    /// It would hang over a top's edge (or into the margin along it).
    OverEdge,
    /// Something else stands there.
    Crowded,
}

/// Where a thing of `radius` standing with its middle at `p` would stand: the host whose top
/// takes the whole disc, with nothing else standing in it (`extra`: more things on surfaces,
/// like plated food).
pub fn fit_on_top(
    content: &Content,
    objects: &[ObjectInstance],
    p: [f32; 2],
    radius: f32,
    extra: &[([f32; 2], f32)],
) -> Result<u32, Misfit> {
    let mut under = None;
    let mut host = None;
    for o in objects {
        let Some(top) = Top::of(content, o) else {
            continue;
        };
        if !top.covers(p) {
            continue;
        }
        under.get_or_insert(top.host);
        if top.holds(p, radius) {
            host = Some(top);
            break;
        }
    }
    let Some(top) = host else {
        return Err(if under.is_some() {
            Misfit::OverEdge
        } else {
            Misfit::NoTop
        });
    };
    let mut discs = Vec::new();
    clutter_on(content, objects, &top, &mut discs);
    if overlaps(&discs, p, radius) || overlaps(extra, p, radius) {
        return Err(Misfit::Crowded);
    }
    Ok(top.host)
}

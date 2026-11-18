//! Assembling a model from its parts: joining them into one mesh (or one per moving part), putting a moving
//! part's origin on its hinge, standing the model on the floor and checking its size.

use crate::math::{Axis, M4, P3, Quat, V3, mul_m4_m4m4, radians, v3};
use crate::mesh::Mesh;
use crate::perch;
use crate::scene::{self, Obj, meshes};

/// What a model draws but isn't its own size: the chair at a desk, the picture on a chest of drawers, the blocks
/// left out in front of a toy box. `aside` joins them into one node named `<what>_aside`; they are drawn and
/// written as usual, and left out of the size (`check_size` here, the gate's `model()`: docs/design/objects.md,
/// "Scale, axes, origin and footprint").
pub const ASIDE: &str = "_aside";

/// The parts made inside `build`, in order: `let lid = new_parts(|| { cube!("lid", ...); cube!("lid_handle", ...); });`
/// then `join(&lid, "lid", None)`.
pub fn new_parts(build: impl FnOnce()) -> Vec<Obj> {
    let before = meshes();
    build();
    meshes()
        .into_iter()
        .filter(|o| !before.contains(o))
        .collect()
}

/// Applies every transform, joins every mesh in the scene, returns the one part. Its origin is the world
/// origin: for a model with no moving parts.
pub fn join_all(name: &str) -> Obj {
    join(&meshes(), name, Some(P3::ZERO))
}

/// Joins `parts` into one part named `name`, rotation and scale applied, so the part is only placed (a clip
/// turns it from rest). It keeps the first part's origin, or takes `origin` (a Blender point) if given; the
/// mesh stays where it is either way.
pub fn join(parts: &[Obj], name: &str, origin: Option<P3>) -> Obj {
    assert!(!parts.is_empty(), "{name}: nothing to join");
    let first = parts[0];
    // each part's UVs in its own frame, before its frame is gone, then the bevel it was to be given
    crate::bevel::apply_all(parts);
    // the others go into the first in the order they were made
    let mut others: Vec<Obj> = parts[1..].iter().copied().filter(|&o| o != first).collect();
    others.sort();
    others.dedup();
    join_into(first, &others);
    // rotation and scale into the mesh
    first.edit(|o| {
        if !o.unturned() {
            let mut m = o.matrix_basis();
            m.0[3] = [0.0, 0.0, 0.0, 1.0];
            o.mesh.transform(&m);
        }
        // (it keeps its mode, as in Blender: a part joined onto a rod is still turned by a quaternion, by none)
        (o.rot, o.scale) = (V3::ZERO, v3(1, 1, 1));
        o.quat = o.quat.map(|_| Quat::IDENTITY);
    });
    // (the part and its mesh take the name; one already in use, by a part or by the mesh of a part that is
    // gone, gets Blender's number after it)
    first.set_name(name).set_data_name(name);
    scene::update(); // (the join's operators have worked the scene out)
    if let Some(origin) = origin {
        set_origin(first, origin);
    }
    first
}

/// `bpy.ops.object.join()` with `first` the active part: the meshes of `others` go into its own, in its frame, in the
/// order given, and leave the scene. `first` keeps its place, turn and scale.
pub(crate) fn join_into(first: Obj, others: &[Obj]) {
    if !others.is_empty() {
        // every part's points go from its own frame to the world and back into the first's, the first's own
        // too: through Blender's inverse of the first's matrix (Eigen's) times the part's matrix, and not at all
        // where that product is the identity within a millionth (`Mesh::transform`)
        let world = first.matrix_world();
        let inverse = crate::math::invert_m4(&world).unwrap_or(M4::IDENTITY);
        let own = mul_m4_m4m4(&inverse, &world);
        first.edit(|o| o.mesh.transform(&own));
        for &other in others {
            let m = mul_m4_m4m4(&inverse, &other.matrix_world());
            let mut mesh = other.read(|o| o.mesh.clone());
            mesh.transform(&m);
            first.edit(|o| append(&mut o.mesh, &mesh));
            other.remove();
        }
    }
}

/// `b`'s points, faces and materials added to `a`.
fn append(a: &mut Mesh, b: &Mesh) {
    let (v0, l0) = (a.verts.len() as u32, a.loops.len());
    let slots: Vec<u16> = b.mats.iter().map(|&m| a.slot(m)).collect();
    a.verts.extend_from_slice(&b.verts);
    for (f, face) in b.faces.iter().enumerate() {
        let corners: Vec<u32> = b.corners(f).iter().map(|&v| v + v0).collect();
        a.push_face(
            &corners,
            slots.get(face.mat as usize).copied().unwrap_or(0),
            face.smooth,
        );
    }
    a.sharp
        .extend(b.sharp.iter().map(|&(x, y)| (x + v0, y + v0)));
    // what one has at its corners or points and the other hasn't is zero on the other
    fn merge<T: Clone>(
        a: &mut Option<Vec<T>>,
        b: &Option<Vec<T>>,
        a_len: usize,
        b_len: usize,
        zero: T,
    ) {
        if a.is_none() && b.is_none() {
            return;
        }
        let mut out = a.take().unwrap_or_else(|| vec![zero.clone(); a_len]);
        match b {
            Some(b) => out.extend_from_slice(b),
            None => out.extend(std::iter::repeat_n(zero, b_len)),
        }
        *a = Some(out);
    }
    merge(&mut a.uvs, &b.uvs, l0, b.loops.len(), [0.0, 0.0]);
    merge(&mut a.stored, &b.stored, l0, b.loops.len(), None);
    merge(
        &mut a.card_shade,
        &b.card_shade,
        v0 as usize,
        b.verts.len(),
        0.0,
    );
    merge(&mut a.ao, &b.ao, v0 as usize, b.verts.len(), 0.0);
    if a.normals.is_some() || b.normals.is_some() {
        // (a part without its own normals keeps the ones worked out for it as it was)
        let mut out = a.normals.take().unwrap_or_else(|| {
            let mut own = a.clone();
            own.loops.truncate(l0);
            own.faces.truncate(own.faces.len() - b.faces.len());
            crate::normals::joined_normals(&own)
        });
        out.extend(
            b.normals
                .clone()
                .unwrap_or_else(|| crate::normals::joined_normals(b)),
        );
        a.normals = Some(out);
    }
}

/// Moves `obj`'s origin to `point` (Blender world, metres) without moving its mesh.
pub fn set_origin(obj: Obj, point: impl Into<P3>) -> Obj {
    let point: P3 = point.into();
    let world = obj.matrix_world();
    let local = world.inverted().unwrap_or(M4::IDENTITY) * V3::from(point);
    let moved = world * M4::translation(local);
    obj.edit(|o| o.mesh.transform(&M4::translation(-local)));
    if obj.read(|o| o.unturned() && o.parent.is_none()) {
        obj.edit(|o| (o.loc, o.seen) = (moved.location(), moved));
    } else {
        // (a turned or scaled part's matrix is taken apart again, as Blender takes `obj.matrix_world = ...`)
        obj.set_matrix_world(moved);
    }
    obj
}

/// A moving part turning about `pivot` (Blender world): its origin goes there, so a clip's rotation swings it
/// about the hinge (docs/design/objects.md, "Moving parts"). For a slide, the pivot is where it slides from; any
/// point of the part will do.
pub fn hinged(obj: Obj, pivot: impl Into<P3>) -> Obj {
    set_origin(obj, pivot)
}

/// Turns `obj` about its own origin by `degrees` and bakes the turn into its mesh, so the part is unrotated. For
/// a part built in its open pose: build it open, `hinged` it, `turn` it to rest (closed), then `swing` it back
/// open by the opposite angle.
pub fn turn(obj: Obj, degrees: f64, axis: Axis) -> Obj {
    obj.edit(|o| o.mesh.transform(&M4::rotation(radians(degrees), axis)));
    obj
}

/// Joins `parts` into the one part the model draws that isn't part of its size (`ASIDE`).
pub fn aside(parts: &[Obj], name: &str) -> Obj {
    join(parts, &format!("{name}{ASIDE}"), None)
}

/// The model's own parts: every one of `objs` but the `aside` ones.
pub fn own(objs: &[Obj]) -> Vec<Obj> {
    objs.iter()
        .copied()
        .filter(|o| !o.name().ends_with(ASIDE))
        .collect()
}

/// (lo, hi): the Blender world bounds of `objs`, a bevel still to be given included.
pub fn measure(objs: &[Obj]) -> ([f64; 3], [f64; 3]) {
    scene::update(); // (Blender works the scene out to measure it)
    let (mut lo, mut hi) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
    for &o in objs {
        let points = if o.read(|p| p.bevel.is_some()) {
            let m = o.matrix_world();
            let local = crate::bevel::points(o);
            if m.is_identity() {
                local
            } else {
                local.iter().map(|&v| m * v).collect()
            }
        } else {
            o.world_points()
        };
        for p in points {
            for i in 0..3 {
                lo[i] = lo[i].min(p[i] as f64);
                hi[i] = hi[i].max(p[i] as f64);
            }
        }
    }
    (lo, hi)
}

/// (width, height, depth) in metres, the game's way round.
pub fn game_size(objs: &[Obj]) -> [f64; 3] {
    let (lo, hi) = measure(objs);
    extent(lo, hi)
}

fn extent(lo: [f64; 3], hi: [f64; 3]) -> [f64; 3] {
    [hi[0] - lo[0], hi[2] - lo[2], hi[1] - lo[1]]
}

/// How `settle` stands a model: `centre` false only stands it on the floor (a model built in place off the
/// footprint's centre: a wall bed at the back of its tiles); `floor` false leaves its height alone (a thing on a
/// wall, built at its real height).
#[derive(Clone, Copy, Debug)]
pub struct Settle {
    pub centre: bool,
    pub floor: bool,
}

impl Default for Settle {
    fn default() -> Settle {
        Settle {
            centre: true,
            floor: true,
        }
    }
}

/// Stands the model on the floor (lowest point at z = 0) and centres it on the footprint in x and y, or on its
/// Model's `centre` (game x, depth), by moving it (never scaling). Call it before adding clips: they key the rest
/// pose where it is. A moving part moves whole, its hinge with it.
pub fn settle_with(objs: &[Obj], how: Settle) -> Vec<Obj> {
    for &o in objs {
        assert!(
            o.read(|p| p.tracks.is_empty()),
            "settle() before swing()/slide(): a clip keys the rest pose where it is"
        );
    }
    let (lo, hi) = measure(objs);
    let drop = if how.floor { -lo[2] } else { 0.0 };
    let centre = scene::with(|s| s.centre);
    let shift = if how.centre {
        v3(
            -centre[0] - (lo[0] + hi[0]) / 2.0,
            -centre[1] - (lo[1] + hi[1]) / 2.0,
            drop,
        )
    } else {
        v3(0, 0, drop)
    };
    for &o in objs {
        o.edit(|p| p.loc += shift);
    }
    perch::shift(shift); // the perches found on it move with it
    objs.to_vec()
}

/// `settle_with` as it nearly always is: on the floor, centred.
pub fn settle(objs: &[Obj]) -> Vec<Obj> {
    settle_with(objs, Settle::default())
}

/// Every mesh in the scene settled.
pub fn settle_all() -> Vec<Obj> {
    settle(&meshes())
}

/// A tolerance: one fraction for all three sides, or one a side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tolerance(pub [f64; 3]);

impl From<f64> for Tolerance {
    fn from(t: f64) -> Tolerance {
        Tolerance([t; 3])
    }
}
impl From<(f64, f64, f64)> for Tolerance {
    fn from(t: (f64, f64, f64)) -> Tolerance {
        Tolerance([t.0, t.1, t.2])
    }
}

/// Checks the model is `size` (game width, height, depth, metres) within `tolerance` (a fraction a side), on the
/// floor, and centred on its footprint, or on `centre` (game x, depth) for a model that stands off the middle of
/// its tiles. `floor` false skips the floor: for a thing on a wall, built at its real height. The measured size
/// and the line that says it, or what is wrong.
pub fn check_size(
    size: [f64; 3],
    tolerance: Tolerance,
    objs: &[Obj],
    name: &str,
    centre: [f64; 2],
    floor: bool,
) -> Result<([f64; 3], String), String> {
    let (lo, hi) = measure(objs);
    let drawn = extent(lo, hi);
    // The size is the model's own: what it carries (`aside`) is drawn but isn't the piece of furniture.
    let mine = own(objs);
    let got = if mine.len() == objs.len() {
        drawn
    } else {
        game_size(&mine)
    };
    // What it measures, said when a check fails, so `size` and `centre` can be copied from it.
    let measured = format!(
        "measured {:.3} x {:.3} x {:.3} m (w x h x d), centred on game ({:.4}, {:.4}) (x, depth)",
        got[0],
        got[1],
        got[2],
        -(lo[0] + hi[0]) / 2.0,
        -(lo[1] + hi[1]) / 2.0
    );
    for (i, axis) in ["width", "height", "depth"].iter().enumerate() {
        let (g, want, t) = (got[i], size[i], tolerance.0[i]);
        if (g - want).abs() > t * want {
            return Err(format!(
                "{name}: {axis} {g:.3} m, want {want:.3} m (±{:.0} %); {measured}",
                t * 100.0
            ));
        }
    }
    let want = [-centre[0], -centre[1]]; // game (x, depth) in Blender x, y
    for (i, axis) in ["x", "y"].iter().enumerate() {
        let c = (lo[i] + hi[i]) / 2.0 - want[i];
        if c.abs() > 0.002 {
            return Err(format!(
                "{name}: off its centre in Blender {axis} by {:.1} cm (settle, or the Model's centre); {measured}",
                c * 100.0
            ));
        }
    }
    if floor && lo[2].abs() > 0.002 {
        return Err(format!(
            "{name}: bottom {:.1} cm off the floor (settle)",
            lo[2] * 100.0
        ));
    }
    let wher = if centre == [0.0, 0.0] {
        "centred".to_string()
    } else {
        format!(
            "centred on game ({}, {})",
            g_format(centre[0]),
            g_format(centre[1])
        )
    };
    let carried = if drawn == got {
        String::new()
    } else {
        format!(
            "; drawn with what it carries, {:.3} x {:.3} x {:.3} m",
            drawn[0], drawn[1], drawn[2]
        )
    };
    Ok((
        got,
        format!(
            "[gamekit] {name} {:.3} x {:.3} x {:.3} m (w x h x d), {wher}, on the floor{carried}",
            got[0], got[1], got[2]
        ),
    ))
}

/// Scales `obj` into an exact w × h × d box (game metres), bottom on z = 0, centred. It stretches each axis on its
/// own, so it is only for third-party files; a model built here is made to its real size and checked with
/// `check_size` (docs/design/objects.md, "Real size, uniform scale").
pub fn fit(obj: Obj, w: f64, h: f64, d: f64) -> Obj {
    let ws = obj.world_points();
    let (mut mn, mut mx) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
    for p in &ws {
        for i in 0..3 {
            mn[i] = mn[i].min(p[i] as f64);
            mx[i] = mx[i].max(p[i] as f64);
        }
    }
    let ext = [mx[0] - mn[0], mx[1] - mn[1], mx[2] - mn[2]];
    let s = [w / ext[0], d / ext[1], h / ext[2]];
    obj.edit(|o| {
        for v in &mut o.mesh.verts {
            v.x = ((v.x as f64 - (mn[0] + mx[0]) / 2.0) * s[0]) as f32;
            v.y = ((v.y as f64 - (mn[1] + mx[1]) / 2.0) * s[1]) as f32;
            v.z = ((v.z as f64 - mn[2]) * s[2]) as f32;
        }
        (o.loc, o.scale) = (V3::ZERO, v3(1, 1, 1));
    });
    obj
}

/// `parts::join` as the family scripts wrote it: `join!(parts, name)` or `join!(parts, name, origin = point)`.
#[macro_export]
macro_rules! join {
    ($parts:expr, $name:expr) => {
        $crate::parts::join($parts, $name, None)
    };
    ($parts:expr, $name:expr, origin = $origin:expr) => {
        $crate::parts::join(
            $parts,
            $name,
            Some($crate::Arg::<$crate::math::P3>::arg($origin)),
        )
    };
}

/// `parts::settle` with its options by name, as the family scripts wrote them: `settle!()` (every mesh),
/// `settle!(centre = false)`, `settle!(objs, centre = false, floor = false)`.
#[macro_export]
macro_rules! settle {
    ($($k:ident = $v:expr),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut how = $crate::parts::Settle::default();
        $(how.$k = $v;)*
        $crate::parts::settle_with(&$crate::scene::meshes(), how)
    }};
    ($objs:expr $(, $k:ident = $v:expr)* $(,)?) => {{
        // (as Python: what is given by place first, then the options in the order written)
        let args = ($objs,);
        #[allow(unused_mut)]
        let mut how = $crate::parts::Settle::default();
        $(how.$k = $v;)*
        $crate::parts::settle_with(args.0, how)
    }};
}

/// A number as Python's `{:g}` writes the centres it is given (whole numbers without a point).
fn g_format(x: f64) -> String {
    let s = format!("{x:.6}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".to_string()
    } else {
        s.to_string()
    }
}

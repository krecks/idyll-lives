//! The scene a model is built in: its parts, in the order they were made, each a mesh placed by a location, a
//! turn and a scale, as Blender's objects were. One scene a thread, emptied between models (`reset`), so the
//! shapes' builders keep the signatures the family scripts called: they add to the scene and hand back the part.

use crate::materials::Finishes;
use crate::math::{
    EulerOrder, M4, P3, Quat, V3, eul_to_quat, mul_m4_m4m4, object_matrix_in, object_matrix_quat,
    radians, v3,
};
use crate::mesh::{Mesh, UvWay};
use crate::motion::Track;
use std::cell::RefCell;

/// Frames a second the clips are keyed at: half the game's sampling rate, so its samples fall on the keys.
pub const FPS: f64 = 30.0;

/// A part in the scene.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Obj(pub(crate) u32);

#[derive(Clone, Debug)]
pub struct Object {
    pub name: String,
    /// Its mesh's own name, which the file's mesh carries: the part's name where its mesh was made under it, a
    /// primitive's kind (`Cube`, `Cylinder.003`) otherwise, until a join names it.
    pub data_name: String,
    pub mesh: Mesh,
    pub loc: V3,
    /// Its Euler angles in `order`, radians (not read while it is turned by a quaternion).
    pub rot: V3,
    /// The order its Euler angles turn it in: XYZ unless a script set its `rotation_mode`.
    pub order: EulerOrder,
    /// Set for a part turned by a quaternion instead (`rod`, or its `rotation_mode` set to it): a part joined
    /// onto such a one stays so, as in Blender, and an Euler turn does nothing to it.
    pub quat: Option<Quat>,
    pub scale: V3,
    /// The moving part this one goes with (`attach`): its place, turn and scale are then in that part's frame.
    pub parent: Option<Obj>,
    pub uv: UvWay,
    /// The bevel it is still to be given (`bevel::bevel_edges`), on its mesh as it is when it is joined or written.
    pub bevel: Option<crate::bevel::Pending>,
    /// Its clips, one track a name.
    pub tracks: Vec<Track>,
    pub(crate) alive: bool,
}

impl Object {
    pub(crate) fn new(name: String, data_name: String, mesh: Mesh) -> Object {
        Object {
            name,
            data_name,
            mesh,
            loc: V3::ZERO,
            rot: V3::ZERO,
            order: EulerOrder::XYZ,
            quat: None,
            scale: v3(1, 1, 1),
            parent: None,
            uv: UvWay::Box,
            bevel: None,
            tracks: Vec::new(),
            alive: true,
        }
    }

    /// Its own matrix (in its parent's frame, if it has one).
    pub fn matrix_basis(&self) -> M4 {
        match self.quat {
            None => object_matrix_in(self.loc, self.rot, self.order, self.scale),
            Some(q) => object_matrix_quat(self.loc, q, self.scale),
        }
    }

    /// Whether it is neither turned nor scaled (only placed).
    pub(crate) fn unturned(&self) -> bool {
        let turned = match self.quat {
            Some(q) => q != Quat::IDENTITY,
            None => self.rot != V3::ZERO,
        };
        !turned && self.scale == v3(1, 1, 1)
    }
}

/// How a part is turned (its `rotation_mode`): by a quaternion, or by Euler angles in one of six orders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RotationMode {
    Quaternion,
    XYZ,
    XZY,
    YXZ,
    YZX,
    ZXY,
    ZYX,
}

impl RotationMode {
    fn order(self) -> Option<EulerOrder> {
        match self {
            RotationMode::Quaternion => None,
            RotationMode::XYZ => Some(EulerOrder::XYZ),
            RotationMode::XZY => Some(EulerOrder::XZY),
            RotationMode::YXZ => Some(EulerOrder::YXZ),
            RotationMode::YZX => Some(EulerOrder::YZX),
            RotationMode::ZXY => Some(EulerOrder::ZXY),
            RotationMode::ZYX => Some(EulerOrder::ZYX),
        }
    }
}

#[derive(Default)]
pub struct Scene {
    pub objects: Vec<Object>,
    /// The perches found on the model being built (Blender points), in the order kept.
    pub perches: Vec<V3>,
    /// Whether a far version is being built (`far`).
    pub low: bool,
    /// The (game x, depth) the model being built is centred on: its Model's `centre`.
    pub centre: [f64; 2],
    pub finishes: Finishes,
    /// What the build said, a line each, for the runner to print.
    pub log: Vec<String>,
    /// Every mesh name in use: the parts', and those of parts that are gone (joined into another), whose meshes
    /// Blender keeps until the scene is emptied.
    pub(crate) mesh_names: std::collections::BTreeSet<String>,
}

/// `name`, or the name Blender gives where `taken` says it is in use: its stem (`name` without a `.123` at its
/// end) with the lowest number free, three digits at least: `water.001`.
pub(crate) fn unique_name(name: &str, taken: impl Fn(&str) -> bool) -> String {
    if !taken(name) {
        return name.to_string();
    }
    let stem = match name.rsplit_once('.') {
        Some((stem, digits))
            if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) =>
        {
            stem
        }
        _ => name,
    };
    (1..)
        .map(|n| format!("{stem}.{n:03}"))
        .find(|candidate| !taken(candidate))
        .unwrap()
}

thread_local! {
    static SCENE: RefCell<Scene> = RefCell::new(Scene::default());
}

/// The scene in hand.
pub fn with<R>(f: impl FnOnce(&mut Scene) -> R) -> R {
    SCENE.with(|s| f(&mut s.borrow_mut()))
}

/// The game's finishes, for the builds on this thread (the runner reads them from the base manifest).
pub fn set_finishes(finishes: Finishes) {
    with(|s| s.finishes = finishes);
}

/// An empty scene: no parts, no perches. The finishes stay.
pub fn reset() {
    with(|s| {
        s.objects.clear();
        s.mesh_names.clear();
        s.perches.clear();
        s.low = false;
        s.centre = [0.0, 0.0];
    });
}

/// The scene's parts, in the order they were made.
pub fn meshes() -> Vec<Obj> {
    with(|s| {
        (0..s.objects.len() as u32)
            .filter(|&i| s.objects[i as usize].alive)
            .map(Obj)
            .collect()
    })
}

/// A line for the runner to print.
pub fn log(line: String) {
    with(|s| s.log.push(line));
}

/// What the builds on this thread said since it was last asked.
pub fn take_log() -> Vec<String> {
    with(|s| std::mem::take(&mut s.log))
}

/// A new part of `mesh` in the scene, the mesh made under the part's name (`bpy.data.meshes.new(name)`, then
/// `bpy.data.objects.new(name, mesh)`). A name in use gets Blender's number: a second part `leg` is `leg.001`.
pub fn add(name: &str, mesh: Mesh) -> Obj {
    add_as(name, name, mesh)
}

/// A new part of `mesh` made by one of Blender's primitive operators: its mesh is named for the primitive's
/// `kind` (`Cube`, `Sphere`, `Cylinder`, `Cone`, `Torus`, `Icosphere`), the part as the script names it after.
pub fn add_primitive(kind: &str, name: &str, mesh: Mesh) -> Obj {
    add_as(name, kind, mesh)
}

/// A new part named `name` with its mesh named `data_name`, each made one of a kind as Blender does.
pub(crate) fn add_as(name: &str, data_name: &str, mesh: Mesh) -> Obj {
    with(|s| {
        let name = unique_name(name, |n| s.objects.iter().any(|o| o.alive && o.name == n));
        let data_name = unique_name(data_name, |n| s.mesh_names.contains(n));
        s.mesh_names.insert(data_name.clone());
        s.objects.push(Object::new(name, data_name, mesh));
        Obj(s.objects.len() as u32 - 1)
    })
}

/// The part named `name`.
pub fn obj(name: &str) -> Obj {
    with(
        |s| match s.objects.iter().position(|o| o.alive && o.name == name) {
            Some(i) => Obj(i as u32),
            None => panic!("no part named {name:?} in the scene"),
        },
    )
}

impl Scene {
    pub fn get(&self, o: Obj) -> &Object {
        let obj = &self.objects[o.0 as usize];
        assert!(
            obj.alive,
            "{}: the part is gone (joined into another)",
            obj.name
        );
        obj
    }

    pub fn get_mut(&mut self, o: Obj) -> &mut Object {
        let obj = &mut self.objects[o.0 as usize];
        assert!(
            obj.alive,
            "{}: the part is gone (joined into another)",
            obj.name
        );
        obj
    }

    /// A part's matrix in the world, at rest.
    pub fn matrix_world(&self, o: Obj) -> M4 {
        let obj = self.get(o);
        match obj.parent {
            None => obj.matrix_basis(),
            Some(p) => mul_m4_m4m4(&self.matrix_world(p), &obj.matrix_basis()),
        }
    }
}

impl Obj {
    /// Reads the part.
    pub fn read<R>(self, f: impl FnOnce(&Object) -> R) -> R {
        with(|s| f(s.get(self)))
    }

    /// Changes the part.
    pub fn edit<R>(self, f: impl FnOnce(&mut Object) -> R) -> R {
        with(|s| f(s.get_mut(self)))
    }

    pub fn name(self) -> String {
        self.read(|o| o.name.clone())
    }

    pub fn location(self) -> V3 {
        self.read(|o| o.loc)
    }

    /// `obj.location = at`.
    pub fn set_location(self, at: impl Into<P3>) -> Obj {
        let at: P3 = at.into();
        self.edit(|o| o.loc = at.into());
        self
    }

    /// `obj.rotation_euler = radians(rot)`: degrees about X, Y and Z.
    pub fn set_rotation(self, rot: impl Into<P3>) -> Obj {
        let r: P3 = rot.into();
        self.edit(|o| o.rot = v3(radians(r.x), radians(r.y), radians(r.z)));
        self
    }

    /// `obj.scale = scale`.
    pub fn set_scale(self, scale: impl Into<P3>) -> Obj {
        let k: P3 = scale.into();
        self.edit(|o| o.scale = k.into());
        self
    }

    pub fn scale(self) -> V3 {
        self.read(|o| o.scale)
    }

    /// `obj.name = name`: under a name another part has, Blender's number after it (`water.001`).
    pub fn set_name(self, name: &str) -> Obj {
        with(|s| {
            let taken = |n: &str| {
                s.objects
                    .iter()
                    .enumerate()
                    .any(|(i, o)| o.alive && i as u32 != self.0 && o.name == n)
            };
            let name = unique_name(name, taken);
            s.get_mut(self).name = name;
        });
        self
    }

    /// `obj.data.name`: the name the file's mesh carries.
    pub fn data_name(self) -> String {
        self.read(|o| o.data_name.clone())
    }

    /// `obj.data.name = name`: one of a kind among the meshes, those of parts joined away too.
    pub fn set_data_name(self, name: &str) -> Obj {
        with(|s| {
            let own = s.get(self).data_name.clone();
            if own != name {
                s.mesh_names.remove(&own);
                let name = unique_name(name, |n| s.mesh_names.contains(n));
                s.mesh_names.insert(name.clone());
                s.get_mut(self).data_name = name;
            }
        });
        self
    }

    /// `obj.location[i] = x` (`obj.location.y -= d` is `o.set_location_axis(1, o.location().y as f64 - d)`: Python
    /// adds in double precision, the part keeps a single).
    pub fn set_location_axis(self, i: usize, x: f64) -> Obj {
        self.edit(|o| o.loc[i] = x as f32);
        self
    }

    /// `obj.scale[i] = k`.
    pub fn set_scale_axis(self, i: usize, k: f64) -> Obj {
        self.edit(|o| o.scale[i] = k as f32);
        self
    }

    /// `obj.rotation_euler`: its Euler angles, radians.
    pub fn rotation_euler(self) -> V3 {
        self.read(|o| o.rot)
    }

    /// `obj.rotation_euler = rot`: radians about X, Y and Z, turned in the part's order (`set_rotation` takes
    /// degrees, as the builders' `rot` does). Kept, but not read, while the part is turned by a quaternion.
    pub fn set_rotation_euler(self, rot: impl Into<P3>) -> Obj {
        let r: P3 = rot.into();
        self.edit(|o| o.rot = r.into());
        self
    }

    /// `obj.rotation_euler[i] = radians`.
    pub fn set_rotation_euler_axis(self, i: usize, radians: f64) -> Obj {
        self.edit(|o| o.rot[i] = radians as f32);
        self
    }

    /// `obj.rotation_mode`.
    pub fn rotation_mode(self) -> RotationMode {
        self.read(|o| match (o.quat, o.order) {
            (Some(_), _) => RotationMode::Quaternion,
            (None, EulerOrder::XYZ) => RotationMode::XYZ,
            (None, EulerOrder::XZY) => RotationMode::XZY,
            (None, EulerOrder::YXZ) => RotationMode::YXZ,
            (None, EulerOrder::YZX) => RotationMode::YZX,
            (None, EulerOrder::ZXY) => RotationMode::ZXY,
            (None, EulerOrder::ZYX) => RotationMode::ZYX,
        })
    }

    /// `obj.rotation_mode = mode`, as Blender changes it: a part turned by a quaternion takes that turn's Euler
    /// angles in the new order (whatever angles it was given meanwhile are gone), one turned by Euler angles takes
    /// their quaternion, and between two Euler orders the angles stay as they are (the part turns).
    pub fn set_rotation_mode(self, mode: RotationMode) -> Obj {
        self.edit(|o| match (mode.order(), o.quat) {
            (Some(order), Some(q)) => {
                o.rot = q.normalized().to_euler(order);
                (o.order, o.quat) = (order, None);
            }
            (Some(order), None) => o.order = order,
            (None, None) => o.quat = Some(eul_to_quat(o.rot, o.order)),
            (None, Some(_)) => {}
        });
        self
    }

    /// `obj.rotation_quaternion` of a part turned by one.
    pub fn rotation_quaternion(self) -> Quat {
        self.read(|o| match o.quat {
            Some(q) => q,
            None => panic!("{}: not turned by a quaternion", o.name),
        })
    }

    /// `obj.rotation_quaternion = q`, for a part whose mode is `RotationMode::Quaternion` (set it first, as the
    /// scripts do: Blender keeps a quaternion given in another mode without reading it).
    pub fn set_rotation_quaternion(self, q: Quat) -> Obj {
        self.edit(|o| {
            assert!(
                o.quat.is_some(),
                "{}: set_rotation_mode(RotationMode::Quaternion) before its quaternion",
                o.name
            );
            o.quat = Some(q);
        });
        self
    }

    /// `obj.matrix_world = m`: the part's place, turn (in its own mode) and scale taken apart from `m` (in its
    /// parent's frame, if it has one), as Blender's `BKE_object_apply_mat4` does: what the part's matrix is
    /// afterwards is those put together again, a unit in the last place off `m` here and there.
    pub fn set_matrix_world(self, m: M4) -> Obj {
        let local = match self.read(|o| o.parent) {
            Some(parent) => {
                let inverse =
                    crate::math::invert_m4(&parent.matrix_world()).unwrap_or(M4::IDENTITY);
                mul_m4_m4m4(&inverse, &m)
            }
            None => m,
        };
        self.edit(|o| {
            let (loc, quat, scale) =
                crate::motion::decompose_quat(&local, o.quat.is_none().then_some(o.order));
            (o.loc, o.scale) = (loc, scale);
            match o.quat {
                Some(_) => o.quat = Some(quat),
                None => o.rot = quat.to_euler(o.order),
            }
        });
        self
    }

    /// The part's matrix in the world, from where it stands now.
    pub fn matrix_world(self) -> M4 {
        with(|s| s.matrix_world(self))
    }

    /// The part's points in the world, at rest (`obj.matrix_world @ v.co`).
    pub fn world_points(self) -> Vec<V3> {
        let m = self.matrix_world();
        self.read(|o| {
            o.mesh
                .verts
                .iter()
                .map(|v| if m.is_identity() { *v } else { m * *v })
                .collect()
        })
    }

    /// Takes the part out of the scene.
    pub fn remove(self) {
        self.edit(|o| {
            o.alive = false;
            o.mesh = Mesh::default();
        });
    }

    pub fn is_alive(self) -> bool {
        with(|s| s.objects[self.0 as usize].alive)
    }
}

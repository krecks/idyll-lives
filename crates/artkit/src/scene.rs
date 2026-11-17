//! The scene a model is built in: its parts, in the order they were made, each a mesh placed by a location, a
//! turn and a scale, as Blender's objects were. One scene a thread, emptied between models (`reset`), so the
//! shapes' builders keep the signatures the family scripts called: they add to the scene and hand back the part.

use crate::materials::Finishes;
use crate::math::{M4, P3, Quat, V3, mul_m4_m4m4, object_matrix, radians, v3};
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
    pub mesh: Mesh,
    pub loc: V3,
    /// Euler XYZ, radians.
    pub rot: V3,
    /// Set for a part turned by a quaternion instead (`rod`).
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
    pub(crate) fn alive_with_tracks(&self) -> bool {
        self.alive && !self.tracks.is_empty()
    }

    pub(crate) fn new(name: &str, mesh: Mesh) -> Object {
        Object {
            name: name.to_string(),
            mesh,
            loc: V3::ZERO,
            rot: V3::ZERO,
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
            None => object_matrix(self.loc, self.rot, self.scale),
            Some(q) => {
                let r = q.to_matrix().0;
                let s = self.scale;
                let mut m = M4::IDENTITY;
                for c in 0..3 {
                    for (row, cell) in r[c].iter().enumerate() {
                        m.0[c][row] = cell * s[c];
                    }
                }
                m.0[3] = [self.loc.x, self.loc.y, self.loc.z, 1.0];
                m
            }
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

/// A new part of `mesh` in the scene.
pub fn add(name: &str, mesh: Mesh) -> Obj {
    with(|s| {
        s.objects.push(Object::new(name, mesh));
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

    /// `obj.matrix_world`.
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

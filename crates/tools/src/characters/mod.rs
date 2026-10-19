//! `cargo characters`: the residents generator and its checks (was `tools/characters/prim/`).
//!
//! Builds the residents' character set from code (rig.json version 4 and its binaries; see
//! docs/characters-format.md): per family and life stage the head interface (head.rs) says where
//! the face is, then the body (body.rs), the face (face.rs), the hair (hair.rs) and the clothes
//! (clothes.rs) are made, and write.rs writes the base stage whole and the other stages as deltas.
//! check.rs holds a set to the contract; diff.rs compares two sets. Everything is deterministic.
//!
//! The port keeps the Python's structure, one module per file, and its arithmetic order where
//! it is cheap to (the numpy and scipy pieces are in np.rs, kdtree.rs and sparse.rs).

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub mod body;
pub mod build;
pub mod cage;
pub mod check;
pub mod clothes;
pub mod contract;
pub mod diff;
pub mod face;
pub mod garments;
pub mod hair;
pub mod head;
pub mod json;
pub mod kdtree;
pub mod meshops;
pub mod np;
pub mod prims;
pub mod rig;
pub mod sparse;
pub mod subdiv;
pub mod write;

pub type V3 = nalgebra::Vector3<f64>;
pub type M3 = nalgebra::Matrix3<f64>;
/// Skin weights of a vertex, per bone.
pub type Dense = [f64; rig::NB];

pub const fn v3(x: f64, y: f64, z: f64) -> V3 {
    V3::new(x, y, z)
}

/// The repo's root (the Python found it from `__file__`).
pub fn repo_root() -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    std::fs::canonicalize(&root).unwrap_or(root)
}

/// The committed set.
pub fn default_out() -> PathBuf {
    repo_root().join("web").join("public").join("assets").join("characters")
}

/// A part: positions and normals (counter-clockwise from outside), flat triangle indices, skin
/// weights, fields and tone (absent: zeros), the sleeve tag (bodies), its face morphs.
#[derive(Clone, Default)]
pub struct Mesh {
    pub pos: Vec<V3>,
    pub nor: Vec<V3>,
    pub idx: Vec<usize>,
    pub dense: Vec<Dense>,
    pub fields: Vec<[f64; 4]>,
    pub tone: Vec<[f64; 4]>,
    pub arm: Vec<f64>,
    pub morph: Option<write::Morph>,
    /// The face's skin pieces: true for the lids (they take the skin's tone, build.rs).
    pub skin_tone: bool,
}

impl Mesh {
    pub fn len(&self) -> usize {
        self.pos.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pos.is_empty()
    }
}

/// `prims.merge` / `face._cat` of whole meshes: indices offset, per-vertex arrays concatenated
/// (an array some piece lacks is dropped).
pub fn merge(meshes: &[&Mesh]) -> Mesh {
    let mut out = Mesh::default();
    let all = |f: fn(&Mesh) -> bool| meshes.iter().all(|m| f(m));
    let (has_nor, has_dense, has_f, has_t, has_arm) =
        (all(|m| m.nor.len() == m.len()), all(|m| m.dense.len() == m.len()), all(|m| m.fields.len() == m.len()), all(|m| m.tone.len() == m.len()), all(|m| m.arm.len() == m.len()));
    for m in meshes {
        let n = out.pos.len();
        out.idx.extend(m.idx.iter().map(|&i| i + n));
        out.pos.extend(&m.pos);
        if has_nor {
            out.nor.extend(&m.nor);
        }
        if has_dense {
            out.dense.extend(&m.dense);
        }
        if has_f {
            out.fields.extend(&m.fields);
        }
        if has_t {
            out.tone.extend(&m.tone);
        }
        if has_arm {
            out.arm.extend(&m.arm);
        }
    }
    out
}

/// A one-hot skin (`face._skin`).
pub fn skin(n: usize, bone: &str) -> Vec<Dense> {
    let mut d = [0.0; rig::NB];
    d[rig::b(bone)] = 1.0;
    vec![d; n]
}

/// A memo table (the Python's `lru_cache`s and module caches), keyed by (family, stage) and the like.
pub struct Cache<K, V>(Mutex<BTreeMap<K, Arc<V>>>);

impl<K: Ord, V> Cache<K, V> {
    pub const fn new() -> Self {
        Cache(Mutex::new(BTreeMap::new()))
    }

    /// The cached value, made (without holding the lock: makers may use other caches) if missing.
    pub fn get(&self, k: K, make: impl FnOnce() -> V) -> Arc<V> {
        if let Some(v) = self.0.lock().unwrap().get(&k) {
            return v.clone();
        }
        let v = Arc::new(make());
        self.0.lock().unwrap().entry(k).or_insert(v).clone()
    }
}

impl<K: Ord, V> Default for Cache<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

/// A cache key of a family and a stage.
pub fn key(family: &str, stage: &str) -> (String, String) {
    (family.to_string(), stage.to_string())
}

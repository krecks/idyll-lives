//! The parts of the driving world: a drive, a garage and where things sit on one.

use super::*;

/// A house's driveway, where a car parks (`LotFile::drives`): the middle of its inner end (x, z),
/// which way it runs from there to the street (`front`: +1 towards +z; `frontX`: +1 towards +x,
/// for a house whose street is east or west of it), and the middle of that street (z).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Drive {
    pub plot: u32,
    pub x: f32,
    pub z: f32,
    pub front: f32,
    /// A driveway running along x (its street east or west of the house; towns made since cars
    /// drive winding roads). Absent: 0, it runs along z (`front`), so files and saves without it
    /// stay byte for byte as they were.
    #[serde(rename = "frontX", default, skip_serializing_if = "is_zero_f32")]
    pub front_x: f32,
    pub street: f32,
    /// The road (`roads` index) the driveway meets and how far along it (m): the sign that its
    /// cars drive the town's roads (`settle_all`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub road: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub s: Option<f32>,
}

impl Drive {
    pub(super) fn meets(&self) -> Option<(u32, f32)> {
        Some((self.road?, self.s.filter(|s| s.is_finite())?))
    }

    /// The way it runs from its inner end to the street (a unit vector: `front`, `front_x`).
    pub(super) fn way(&self) -> Option<[f32; 2]> {
        let (x, z) = (self.front_x, self.front);
        let l = x.hypot(z);
        (l.is_finite() && l > 0.5).then(|| [x / l, z / l])
    }

    /// Where a car `len` long parks on it: its middle `PARK_GAP` on from the inner end.
    pub(super) fn spot(&self, len: f32) -> [f32; 2] {
        let [x, z] = self.way().unwrap_or([0.0, 0.0]);
        let k = PARK_GAP + len / 2.0;
        [self.x + x * k, self.z + z * k]
    }
}

/// A garage at the inner end of a house's driveway (`LotFile::garages`, as the town placed it:
/// wall centre lines), or one the player built on their lot (`World::build_garage`). It stands
/// where it is; nobody walks through it and nothing is put there (`reblock`).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Garage {
    pub plot: u32,
    pub x0: f32,
    pub z0: f32,
    pub x1: f32,
    pub z1: f32,
    /// Which way its door faces, as a thing's facing (0 towards +z, 1 +x, 2 -z, 3 -x): one the
    /// player built (Build mode's Garage tool). Absent: towards the street, as a town places them
    /// (and so files and saves without it stay as they were).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rot: Option<u8>,
}

/// Half a garage's wall (m), outside its centre lines (the renderer's `WALL_HALF`).
pub const GARAGE_WALL_HALF: f32 = 0.07;

impl Garage {
    /// The tiles it stands on.
    pub(crate) fn tiles(&self) -> Vec<(i32, i32)> {
        covered_tiles(self.x0, self.z0, self.x1, self.z1)
    }

    /// The middle of its door outside (on the outer face of its wall) and the way it faces out
    /// (a unit vector), for a garage with a facing (`rot`).
    pub fn door(&self) -> Option<([f32; 2], [f32; 2])> {
        let (cx, cz) = ((self.x0 + self.x1) / 2.0, (self.z0 + self.z1) / 2.0);
        let t = GARAGE_WALL_HALF;
        Some(match self.rot? % 4 {
            0 => ([cx, self.z1 + t], [0.0, 1.0]),
            1 => ([self.x1 + t, cz], [1.0, 0.0]),
            2 => ([cx, self.z0 - t], [0.0, -1.0]),
            _ => ([self.x0 - t, cz], [-1.0, 0.0]),
        })
    }
}

/// The tiles the box from (x0, z0) to (x1, z1) covers at least half of each way.
pub(super) fn covered_tiles(x0: f32, z0: f32, x1: f32, z1: f32) -> Vec<(i32, i32)> {
    let covers = |a: f32, b: f32, t: i32| (b.min(t as f32 + 1.0) - a.max(t as f32)) >= 0.5;
    let mut out = Vec::new();
    for z in z0.floor() as i32..=z1.floor() as i32 {
        for x in x0.floor() as i32..=x1.floor() as i32 {
            if covers(x0, x1, x) && covers(z0, z1, z) {
                out.push((x, z));
            }
        }
    }
    out
}

/// Of `sorted` (by `x_of`), those whose x is within `reach` of `x`: all that may be within
/// `reach` of a point there (`Near`, `hidden`), without looking at the rest.
pub(crate) fn along_x<T>(sorted: &[T], x_of: impl Fn(&T) -> f32, x: f32, reach: f32) -> &[T] {
    let lo = sorted.partition_point(|e| x_of(e) < x - reach);
    let hi = sorted.partition_point(|e| x_of(e) <= x + reach);
    &sorted[lo..hi]
}

/// The point `along` the length (forward) and `left` across (to the left) of a car standing at
/// `spot` facing `yaw`.
pub(super) fn local_at(spot: [f32; 2], yaw: f32, along: f32, left: f32) -> [f32; 2] {
    let (s, c) = yaw.sin_cos();
    [
        spot[0] + s * along - c * left,
        spot[1] + c * along + s * left,
    ]
}

//! What the fridges and the sinks are built from: parts every model here shares (each in the model's own
//! GameFrame), and the glossy round shading of a bevelled shell.

use super::M;
use artkit::forms::rounded_rect;
use artkit::materials::Mat;
use artkit::place::{Facing, GameFrame};
use artkit::py;
use artkit::scene::Obj;
use artkit::{fcyl, fextrude, fsphere, fsweep, loft, ring};

// --- parts the K1 fridges and the sinks share (each in the model's own GameFrame) ---------------------------

/// A door's rubber seal: a cord round the back of a w × h door whose middle is at (x, y), at depth `z`.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn gasket(
    f: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    y: f64,
    z: f64,
    r: f64,
    corner: f64,
    x: f64,
) -> Obj {
    let lp = rounded_rect(w, h, corner, 2);
    let pts: Vec<(f64, f64, f64)> = lp.iter().map(|&(u, v)| (x + u, y + v, z)).collect();
    fsweep!(f, name, &pts, r, M.gasket, verts = 5, closed = true)
}

/// A kick plate with a row of vent slots, over a dark recess, at height `y` (its middle) and depth `z`.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn kick(
    f: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    y: f64,
    z: f64,
    material: Option<Mat>,
    slots: usize,
) -> Obj {
    artkit::fbox!(
        f,
        format!("{name}_recess"),
        w - 0.02,
        h,
        0.01,
        (0, y, z - 0.009),
        M.dark
    );
    let slot = rounded_rect(
        py::min(0.07, w / slots as f64 - 0.03),
        py::min(0.012, h * 0.3),
        0.005,
        1,
    );
    let step = (w - 0.06) / slots as f64;
    let holes: Vec<Vec<(f64, f64)>> = (0..slots)
        .map(|i| {
            slot.iter()
                .map(|&(u, v)| (u + (i as f64 - (slots - 1) as f64 / 2.0) * step, v))
                .collect()
        })
        .collect();
    fextrude!(
        f,
        name,
        &rounded_rect(w, h, 0.006, 1),
        0.008,
        (0, y, z),
        material.unwrap_or(M.seal),
        holes = holes
    )
}

/// A gable-top carton standing on game height `y`: its side outline (a house with a fin) extruded across.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn carton(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
    h: f64,
    material: Mat,
    turn: i32,
) -> Obj {
    let (d, g) = (w, w * 0.42);
    let outline = [
        (-d / 2.0, 0.0),
        (d / 2.0, 0.0),
        (d / 2.0, h),
        (0.003, h + g),
        (0.003, h + g + 0.014),
        (-0.003, h + g + 0.014),
        (-0.003, h + g),
        (-d / 2.0, h),
    ];
    fextrude!(
        f,
        name,
        &outline,
        w,
        (x, y, z),
        material,
        plane = Facing::Side,
        rot = (0, 0, turn),
        bevel = 0.002,
        bevel_segments = 1
    )
}

/// An apple sitting on game height `y`: a squat ball with a dimple's stalk.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn apple(f: &GameFrame, name: &str, x: f64, y: f64, z: f64, r: f64, material: Option<Mat>) {
    fsphere!(
        f,
        name,
        r,
        (x, y + r * 0.92, z),
        material.unwrap_or(M.produce),
        segments = 10,
        rings = 7,
        scale = (1, 1, 0.92)
    );
    fcyl!(
        f,
        format!("{name}_stalk"),
        0.0028,
        0.016,
        (x + 0.002, y + r * 1.78 + 0.006, z),
        M.board,
        verts = 5,
        smooth = false
    );
}

/// A jar on game height `y`: a rounded body, a shoulder and a screw lid.
#[allow(clippy::too_many_arguments)] // (as the script called it)
pub fn jar(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    r: f64,
    h: f64,
    material: Mat,
    lid: Mat,
    verts: usize,
) {
    let b = f.at(x, y, z);
    let (bx, by, bz) = (b.x, b.y, b.z);
    let rings = [
        (0.0, 0.86),
        (0.04, 1.0),
        (0.8, 1.0),
        (0.9, 0.84),
        (0.92, 0.8),
    ];
    let sections: Vec<_> = rings
        .iter()
        .map(|&(t, k)| ring!(r * k, r * k, z = bz + h * t, verts = verts, at = (bx, by)))
        .collect();
    loft!(name, &sections, material = material);
    fcyl!(
        f,
        format!("{name}_lid"),
        r * 0.84,
        h * 0.12,
        (x, y + h * 0.97, z),
        lid,
        verts = verts,
        bevel = 0.002,
        bevel_segments = 1
    );
}

/// Glossy round shading for a bevelled shell: smooth over the bevels, flat on the broad faces. Its bevel is made
/// (a cut is made before, by the caller), it is shaded smooth with the edges turning more than 40° kept sharp as
/// well as those that were, and each smooth run of faces round a point takes the mean of their normals weighed by
/// their areas (Blender's weighted-normal modifier, by face area, keeping the sharp edges).
pub fn soft(obj: Obj) -> Obj {
    artkit::bevel::apply(obj);
    artkit::shapes::shade_smooth_by_angle(obj, 40.0);
    artkit::weighted_normal!(obj, keep_sharp = true)
}

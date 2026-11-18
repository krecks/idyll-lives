//! What the energetic pack is built from: its palette, meshes from points, lofted bodies, and the kit's parts it
//! uses.

use artkit::materials::Mat;
use artkit::math::{P3, V3};
use artkit::mesh::Mesh;
use artkit::place::{GameFrame, Spot};
use artkit::scene::{self, Obj};
use artkit::{loft, palette, ring};
use std::cell::Cell;

palette! { M:
    // the bouldering wall
    plywood = ("#E3C79C", wood),          // the climbing panel; its main part
    timber = ("#C9A274", wood),
    climb_frame = ("#6C7A89", metal),     // the steel A-frame behind
    crash_mat = ("#2F6FA8", fabric),
    mat_band = ("#F2C14E", fabric),
    chalk = ("#E8692E", fabric),
    hold_a = ("#E4572E", satin),
    hold_b = ("#F2C14E", satin),
    hold_c = ("#4CB5AE", satin),
    hold_d = ("#7B5EA7", satin),
    hold_e = ("#7FB800", satin),
    hold_f = ("#F25F9C", satin),
    hold_g = ("#3A86FF", satin),
    bolt = ("#3A3D41", metal),
    // the spin bike
    bike_frame = ("#F2F2F0", gloss),      // the frame; its main part
    bike_pad = ("#2F3B4A", fabric),       // the seat and backrest
    bike_grey = ("#9EA3A8", satin),
    flywheel = ("#E8692E", satin),
    steel = ("#C3C7CB", metal),
    dark = ("#3A3D41", satin),
    route = ("#1E4F7A", gloss),
    route_line = ("#7FB800", gloss),
    bottle = ("#3A86FF", gloss),
    // the trampoline
    tramp_frame = ("#8A9096", metal),     // the frame and legs; its main part
    tramp_pad = ("#2F8F83", fabric),      // the safety pad over the springs
    tramp_mat = ("#4F6585", matte),
    ball = ("#E8692E", gloss),
    ball_seam = ("#3A2A20", matte),
    // the streetball hoop
    hoop_base = ("#2F5D8A", satin),
    hoop_pole = ("#7D858D", metal),       // the pole; its main part
    board = ("#F4F4F2", gloss),
    pad_red = ("#D7263D", satin),         // the board's border and the pole pad
    rim = ("#E8692E", metal),
    net = ("#F4F4F2", satin),
    cap = ("#E8E8E5", satin),
    // the blender station
    blend_cab = ("#DDEFE6", satin),       // the cabinet; its main part
    blend_door = ("#CDE6DA", satin),
    plinth = ("#B9C7C0", satin),
    worktop = ("#F4F4F2", gloss, roughness(0.3)),
    chrome = ("#D0D3D6", chrome),
    motor = ("#F2F2F0", gloss),
    lime = ("#8CC63F", gloss),
    jar = ("#E6F0F2", glass),             // the jug and the glasses (glass: the smoothies show through)
    smoothie = ("#9BCB5A", gloss),
    drink_pink = ("#F07AA0", gloss),
    drink_orange = ("#F2A93B", gloss),
    bowl = ("#C8956B", satin),
    chopping = ("#D9B98C", satin),
    orange = ("#E8692E", gloss),
    apple = ("#D7263D", gloss),
    banana = ("#F2C94C", satin),
    lemon = ("#F2E05C", satin),
    // the contrast shower
    shower_frame = ("#8E979E", metal),    // the posts and rails; its main part
    tray = ("#EEF3F2", gloss),
    grout = ("#BFD6D2", satin),
    tile = ("#E2F0EE", satin),
    glass = ("#D6E8EC", glass),
    fittings = ("#D0D3D6", chrome),
    rain_face = ("#B9DDF0", gloss),
    hot = ("#D9483B", gloss),
    cold = ("#3A86FF", gloss),
    shower_mat = ("#7FC4C0", satin),
    towel = ("#F2F2F0", satin),
    water = ("#BFE3F2", gloss),
    // the recovery pod
    pod_shell = ("#F2F2F0", gloss),       // the shell; its main part
    pod_glow = ("#3FA7A0", satin),
    pod_pad = ("#D8DDE2", satin),         // the mattress
    pod_head = ("#ECEFF2", satin),
    pod_boots = ("#3FA7A0", satin),
    canopy_glass = ("#8FB9C9", glass),
    canopy_rim = ("#F2F2F0", gloss),
    pod_screen = ("#1E4F7A", gloss),
    pod_line = ("#7FE0D8", gloss),
    // added with the detail pass
    stem = ("#6E5A3A", satin),            // fruit stalks
}

/// The holds' colours, in turn.
pub const HOLDS: [Mat; 7] = [
    M.hold_a, M.hold_b, M.hold_c, M.hold_d, M.hold_e, M.hold_f, M.hold_g,
];
/// The wall's face in a 1-deep footprint, game depth.
pub const WALL: f64 = -0.43;

thread_local! {
    static N: Cell<u32> = const { Cell::new(0) };
}

/// `stem` numbered: each part its own name.
pub fn name(stem: &str) -> String {
    N.with(|n| {
        n.set(n.get() + 1);
        format!("{stem}{}", n.get())
    })
}

/// A mesh object from Blender-space `verts` and `faces`, normals made outward, smooth below `sharp` degrees.
#[allow(dead_code)] // (the script has it too, unused)
pub fn obj(name: &str, verts: &[P3], faces: &[Vec<u32>], mats: &[Mat], sharp: f64) -> Obj {
    let mut mesh = Mesh::from_pydata(verts.iter().map(|&p| V3::from(p)).collect(), faces);
    for &m in mats {
        mesh.slot(m);
    }
    mesh.recalc_normals();
    let o = scene::add(name, mesh);
    o.edit(|p| p.mesh.shade_smooth(sharp));
    o
}

/// A polygon's middle, as Blender gives it: the mean of its corners.
fn centre(mesh: &Mesh, f: usize) -> V3 {
    let c = mesh.corners(f);
    let mut s = V3::ZERO;
    for &v in c {
        s += mesh.verts[v as usize];
    }
    let n = c.len() as f32;
    V3 {
        x: s.x / n,
        y: s.y / n,
        z: s.z / n,
    }
}

/// Gives the faces of `obj` whose middle (its own axes, Blender) passes `test` another material: a band, a stripe,
/// a surround, with no more triangles.
pub fn paint(obj: Obj, material: Mat, test: impl Fn(V3) -> bool) -> Obj {
    obj.edit(|p| {
        let k = p.mesh.slot(material);
        for f in 0..p.mesh.faces.len() {
            if test(centre(&p.mesh, f)) {
                p.mesh.faces[f].mat = k;
            }
        }
    });
    obj
}

/// A body standing on `y` at (x, z), lofted through rings (height, half width, half depth) from its foot up.
#[allow(clippy::too_many_arguments)]
pub fn lofted(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    z: f64,
    rows: &[(f64, f64, f64)],
    material: Mat,
    verts: usize,
    square: f64,
) -> Obj {
    let P3 {
        x: bx,
        y: by,
        z: bz,
    } = f.at(x, y, z);
    let sections: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(h, rx, rz)| {
            ring!(
                rx,
                rz,
                z = bz + h,
                verts = verts,
                square = square,
                at = (bx, by)
            )
        })
        .collect();
    loft!(name, &sections, material = material)
}

/// The kit's tube (`artkit::kit::tube`), as the script called it.
pub fn tube<P: Spot>(
    f: &GameFrame,
    name: &str,
    points: &[P],
    r: f64,
    material: Mat,
    verts: Option<usize>,
    round: usize,
) -> Obj {
    artkit::kit::tube(
        f,
        name,
        points,
        r,
        material,
        artkit::kit::Tube {
            verts,
            round,
            ..artkit::kit::Tube::default()
        },
    )
}

/// The kit's fillet (`artkit::kit::fillet`): the path through `points` with each inner corner rounded.
pub fn fillet(points: &[(f64, f64, f64)], radius: f64, steps: usize) -> Vec<(f64, f64, f64)> {
    artkit::kit::fillet(points, radius, steps)
}

/// The kit's bar pull (`artkit::kit::pull`), as the script called it.
#[allow(clippy::too_many_arguments)]
pub fn pull(
    f: &GameFrame,
    name: &str,
    x: f64,
    y: f64,
    face: f64,
    length: f64,
    material: Mat,
    r: f64,
    out: f64,
    upright: bool,
    verts: usize,
) -> Obj {
    artkit::kit::pull(
        f,
        name,
        x,
        y,
        face,
        length,
        material,
        artkit::kit::Pull {
            r,
            out,
            upright,
            verts,
        },
    )
}

/// The kit's frame-and-panel front (`artkit::kit::shaker`), as the script called it: (frame, panel).
#[allow(clippy::too_many_arguments)]
pub fn shaker(
    f: &GameFrame,
    name: &str,
    w: f64,
    h: f64,
    at: (f64, f64, f64),
    t: f64,
    material: Mat,
    rail: f64,
    panel: Option<Mat>,
    recess: f64,
    raised: bool,
) -> (Obj, Obj) {
    artkit::kit::shaker(
        f,
        name,
        w,
        h,
        at,
        t,
        material,
        artkit::kit::Shaker {
            rail,
            panel,
            recess,
            raised,
        },
    )
}

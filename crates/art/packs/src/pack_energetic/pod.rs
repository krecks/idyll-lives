//! The recovery pod (1x2): a padded lounger (top 0.5) in a shell, a tinted canopy over the head end.

use super::common::{M, lofted, name};
use super::play::settle_in_place;
use artkit::forms::rounded_rect;
use artkit::math::{Axis, P3, TAU, V3, cos, sin};
use artkit::mesh::Mesh;
use artkit::parts::{hinged, join, new_parts, turn};
use artkit::place::{Facing, GameFrame};
use artkit::scene::Obj;
use artkit::{fbox, fcyl, fextrude, fsphere, fsweep, loft, soft, swing, torus};
use std::collections::BTreeMap;

const CANOPY: (f64, f64, f64, f64, f64) = (0.42, -0.35, 0.4, 0.4, 0.5); // the canopy's rim height, middle depth, half-width, height, half-length
const CANOPY_UP: f64 = 40.0; // how far it stands open at rest (degrees)

/// A compression boot lying along the mattress, toe to the front: a lofted leg sleeve, quilted in rings.
fn boot(f: &GameFrame, x: f64, y: f64, z: f64) {
    let P3 {
        x: bx,
        y: by,
        z: bz,
    } = f.at(x, y, z);
    let rows = [
        (-0.31, 0.04, 0.035),
        (-0.29, 0.1, 0.055),
        (-0.1, 0.11, 0.06),
        (0.12, 0.1, 0.06),
        (0.24, 0.09, 0.07),
        (0.29, 0.07, 0.065),
        (0.31, 0.02, 0.03),
    ];
    let secs: Vec<Vec<P3>> = rows
        .iter()
        .map(|&(u, rx, rz)| {
            (0..12)
                .map(|k| P3 {
                    x: bx + rx * cos(TAU * k as f64 / 12.0),
                    y: by - u,
                    z: bz + rz * sin(TAU * k as f64 / 12.0),
                })
                .collect()
        })
        .collect();
    loft!(name("boot"), &secs, material = M.pod_boots);
    for u in [-0.2, -0.06, 0.08] {
        torus!(
            name("boot_quilt"),
            1.0,
            0.006,
            at = (bx, by - u, bz),
            material = M.pod_boots,
            rot = (90, 0, 0),
            major_segments = 12,
            minor_segments = 3
        )
        .set_scale((0.112, 0.062, 1.0));
    }
}

// KIT: bmesh.ops.delete(context='VERTS')
/// The mesh without its points below `z` (its own axes) and the faces on them.
fn delete_below(mesh: &mut Mesh, z: f32) {
    let keep: Vec<bool> = mesh.verts.iter().map(|v| v.z >= z).collect();
    let mut at = vec![u32::MAX; mesh.verts.len()];
    let mut out = Mesh {
        mats: mesh.mats.clone(),
        ..Mesh::default()
    };
    for (i, v) in mesh.verts.iter().enumerate() {
        if keep[i] {
            at[i] = out.verts.len() as u32;
            out.verts.push(*v);
        }
    }
    for f in 0..mesh.faces.len() {
        let c = mesh.corners(f);
        if c.iter().all(|&v| keep[v as usize]) {
            let c: Vec<u32> = c.iter().map(|&v| at[v as usize]).collect();
            out.push_face(&c, mesh.faces[f].mat, mesh.faces[f].smooth);
        }
    }
    out.sharp = mesh
        .sharp
        .iter()
        .filter(|e| keep[e.0 as usize] && keep[e.1 as usize])
        .map(|e| artkit::mesh::edge(at[e.0 as usize], at[e.1 as usize]))
        .collect();
    *mesh = out;
}

/// A face's run along an edge: from, to, the face.
type Run = (u32, u32, usize);

// KIT: the Solidify modifier (simple, its offset -1, rim filled)
/// The surface thickened `t` inwards along its points' normals: the inner surface faces in, a rim closes its open
/// edges.
fn solidify(mesh: &mut Mesh, t: f32) {
    let n = mesh.verts.len() as u32;
    let normals = artkit::normals::vertex_normals(mesh);
    let inner: Vec<V3> = mesh
        .verts
        .iter()
        .zip(&normals)
        .map(|(v, k)| V3 {
            x: v.x - k.x * t,
            y: v.y - k.y * t,
            z: v.z - k.z * t,
        })
        .collect();
    let faces = mesh.faces.len();
    // each edge's faces' runs along it, for the open edges (one face) the rim closes
    let mut runs: BTreeMap<(u32, u32), Vec<Run>> = BTreeMap::new();
    for f in 0..faces {
        let c = mesh.corners(f).to_vec();
        for i in 0..c.len() {
            let (a, b) = (c[i], c[(i + 1) % c.len()]);
            runs.entry(artkit::mesh::edge(a, b))
                .or_default()
                .push((a, b, f));
        }
    }
    mesh.verts.extend(inner);
    for f in 0..faces {
        let mut c: Vec<u32> = mesh.corners(f).iter().map(|&v| v + n).collect();
        c.reverse();
        let (mat, smooth) = (mesh.faces[f].mat, mesh.faces[f].smooth);
        mesh.push_face(&c, mat, smooth);
    }
    let sharp: Vec<(u32, u32)> = mesh.sharp.iter().map(|&(a, b)| (a + n, b + n)).collect();
    mesh.sharp.extend(sharp);
    for run in runs.values() {
        if let [(a, b, f)] = run[..] {
            let (mat, smooth) = (mesh.faces[f].mat, mesh.faces[f].smooth);
            mesh.push_face(&[b, a, a + n, b + n], mat, smooth);
        }
    }
}

pub fn recovery_pod() -> Vec<Obj> {
    let f = &GameFrame::new(0.0);
    let still = new_parts(|| {
        // the shell: a tub lofted through rounded rings, a glowing strip round its foot
        lofted(
            f,
            "shell",
            0.0,
            0.04,
            0.0,
            &[
                (0.0, 0.39, 0.91),
                (0.02, 0.405, 0.925),
                (0.16, 0.41, 0.93),
                (0.27, 0.4, 0.92),
                (0.29, 0.385, 0.905),
            ],
            M.pod_shell,
            32,
            7.0,
        );
        let strip: Vec<(f64, f64, f64)> = rounded_rect(0.78, 1.82, 0.08, 2)
            .iter()
            .map(|&(x, z)| (x, 0.025, z))
            .collect();
        fsweep!(
            f,
            "glow_strip",
            &strip,
            0.015,
            M.pod_glow,
            verts = 5,
            closed = true
        );
        soft!(
            f,
            "mattress",
            0.68,
            0.132,
            1.66,
            (0, 0.426, 0.04),
            M.pod_pad,
            round = 0.05,
            puff = 0.008,
            piping = 0.006,
            flat = 2
        );
        soft!(
            f,
            "headrest",
            0.5,
            0.1,
            0.3,
            (0, 0.53, -0.7),
            M.pod_head,
            round = 0.04,
            puff = 0.012,
            flat = 2
        );
        for x in [-0.13, 0.13] {
            boot(f, x, 0.56, 0.5);
        }
        fextrude!(
            f,
            "control",
            &rounded_rect(0.3, 0.3, 0.05, 2),
            0.06,
            (0.38, 0.5, 0.55),
            M.pod_shell,
            plane = Facing::Side,
            bevel = 0.015,
            bevel_segments = 1
        );
        fbox!(
            f,
            "control_screen",
            0.004,
            0.16,
            0.2,
            (0.412, 0.54, 0.55),
            M.pod_screen
        );
        fbox!(
            f,
            "control_line",
            0.004,
            0.03,
            0.12,
            (0.414, 0.6, 0.55),
            M.pod_line
        );
        for k in 0..3 {
            fcyl!(
                f,
                name("control_key"),
                0.012,
                0.004,
                (0.412, 0.44, 0.49 + 0.06 * k as f64),
                M.pod_line,
                rot = (0, 90, 0),
                verts = 8
            );
        }
        for x in [-0.33, 0.33] {
            fextrude!(
                f,
                name("hinge_block"),
                &rounded_rect(0.06, 0.07, 0.02, 2),
                0.06,
                (x, 0.42, -0.86),
                M.pod_shell,
                plane = Facing::Side,
                bevel = 0.008,
                bevel_segments = 1
            );
        }
    });
    join(&still, "body", Some(P3::ZERO));
    // The canopy: a tinted glass half-shell with a white rim, built lowered, hinged at the head end and turned up
    // to stand open at rest; `canopy` lowers it over the resident.
    let (ry, cz, hw, hh, hl) = CANOPY;
    let can = new_parts(|| {
        let shell = fsphere!(
            f,
            "canopy",
            1.0,
            (0, ry, cz),
            M.canopy_glass,
            scale = (hw, hl, hh),
            segments = 28,
            rings = 14
        );
        shell.edit(|o| {
            delete_below(&mut o.mesh, -1e-4);
            solidify(&mut o.mesh, 0.008);
        });
        let rim = torus!(
            "canopy_rim",
            1.0,
            0.06,
            at = f.at(0, ry, cz),
            material = M.canopy_rim,
            major_segments = 32,
            minor_segments = 5
        );
        rim.set_scale((hw + 0.005, hl + 0.005, 0.4));
        for x in [-0.33, 0.33] {
            fbox!(
                f,
                name("canopy_arm"),
                0.02,
                0.03,
                0.12,
                (x, ry + 0.01, cz - hl + 0.05),
                M.canopy_rim,
                bevel = 0.006,
                bevel_segments = 1
            );
        }
    });
    let can = hinged(join(&can, "canopy", None), f.at(0, ry, cz - hl));
    turn(can, -CANOPY_UP, Axis::X);
    let objs = settle_in_place();
    swing!(can, "canopy", CANOPY_UP, seconds = 1.2, axis = Axis::X);
    objs
}

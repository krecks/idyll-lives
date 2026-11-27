//! The woods' rock: boulders roughened, split and cut off flat under the ground.

use super::M;
use artkit::edit::{NgonMethod, QuadMethod, holes_fill, triangulate};
use artkit::far::low;
use artkit::materials::Mat;
use artkit::math::{M4, V3, v3};
use artkit::noise::noise;
use artkit::parts::{join_all, measure};
use artkit::scene::Obj;
use artkit::{bisect_plane, boulder};

/// Game (x, depth): the old part's middle.
pub const ROCK_CENTRE: (f64, f64) = (0.085, 0.01);
/// The rock's lumps reach this far under the ground: it is cut flat there.
pub const GROUND: f64 = -0.115;

/// Cuts `obj` (in place) by the world's plane through `co` facing `no`, dropping what lies in front of it and
/// closing the cut with a face of its own: the ground. The cut face is drawn flat. What is left is all triangles.
pub fn cut(obj: Obj, co: impl Into<V3>, no: impl Into<V3>) -> Obj {
    cut_in(obj, obj.matrix_world(), co.into(), no.into())
}

/// `cut` by a plane in the part's own frame, wherever it stands: a fracture across a stone.
pub fn cut_own(obj: Obj, co: impl Into<V3>, no: impl Into<V3>) -> Obj {
    cut_in(obj, M4::IDENTITY, co.into(), no.into())
}

/// `cut` by the plane through `co` facing `no` in the frame `world` takes the part's points into.
fn cut_in(obj: Obj, world: M4, co: V3, no: V3) -> Obj {
    obj.edit(|o| o.mesh.transform(&world));
    let edges = bisect_plane!(obj, co, no, clear_outer = true);
    if !edges.is_empty() {
        let made = holes_fill(obj, &edges);
        obj.edit(|o| {
            for f in made {
                o.mesh.faces[f].smooth = false;
            }
        });
    }
    triangulate(obj, QuadMethod::Beauty, NgonMethod::Beauty);
    obj.edit(|o| o.mesh.transform(&world.inverted().unwrap_or(M4::IDENTITY)));
    obj
}

/// Roughens a boulder's surface with finer noise than `boulder`'s (its own metres, before its scale).
pub fn lump(obj: Obj, amount: f64, scale: f64, seed: f64) {
    obj.edit(|o| {
        for v in &mut o.mesh.verts {
            let p = *v * scale + v3(seed, -seed, seed * 0.5);
            *v *= 1.0 + amount * noise(p);
        }
    });
}

/// A boulder `top` high (Blender z), roughened (`rough`: its noise and how lumpy it is; `grain`: and pitted finer
/// still, metres), with `fractures` (direction, depth: how far in from its outside, metres) cut flat across it, then
/// cut off flat at `ground` (Blender z), where it goes into the ground (the garden decor's stones too).
#[allow(clippy::too_many_arguments)]
pub fn stone(
    name: &str,
    radius: f64,
    at: (f64, f64, f64),
    material: Mat,
    scale: (f64, f64, f64),
    seed: f64,
    top: f64,
    subdivisions: usize,
    fractures: &[((f64, f64, f64), f64)],
    grain: f64,
    rough: (f64, f64),
    ground: f64,
) -> Obj {
    let o = boulder!(
        name,
        radius,
        at = at,
        material = material,
        scale = (1, 1, 1),
        seed = seed,
        noise = rough.0,
        subdivisions = subdivisions
    );
    lump(o, rough.1, 4.0, seed);
    if grain != 0.0 {
        lump(o, grain, 14.0, seed + 5.1);
    }
    o.edit(|p| p.mesh.transform(&M4::diagonal(scale.into())));
    for &(direction, depth) in fractures {
        let n = V3::from(direction).normalized();
        let reach = o.read(|p| {
            p.mesh
                .verts
                .iter()
                .map(|v| v.dot(n))
                .fold(f64::NEG_INFINITY, f64::max)
        });
        cut_own(o, n * (reach - depth), n);
    }
    o.edit(|p| {
        let highest = p
            .mesh
            .verts
            .iter()
            .map(|v| v.z)
            .fold(f32::NEG_INFINITY, f32::max);
        p.loc.z = (top - highest as f64) as f32;
    });
    cut(o, (0, 0, ground), (0, 0, -1))
}

/// Colours a stone's weathered faces (its rounded ones, not the fresh flat breaks): lichen in patches on what
/// faces the sky (smooth noise over `cover`), and the damp, earthy band where it goes into the ground.
fn weather(obj: Obj, seed: f64, lichen: Mat, soil: Mat, cover: f64) {
    let world = obj.matrix_world();
    obj.edit(|o| {
        let (lichen, soil) = (o.mesh.slot(lichen), o.mesh.slot(soil));
        for f in 0..o.mesh.faces.len() {
            if !o.mesh.faces[f].smooth {
                continue;
            }
            let c = world * o.mesh.face_centre(f);
            if (c.z as f64) < 0.004 {
                o.mesh.faces[f].mat = soil;
            } else if o.mesh.face_normal(f).z as f64 > 0.45
                && noise(v3(
                    c.x as f64 * 7.0 + seed,
                    c.y as f64 * 7.0 - seed,
                    c.z as f64 * 7.0,
                )) > cover
            {
                o.mesh.faces[f].mat = lichen;
            }
        }
    });
}

/// A grey boulder and a lighter one leaning on it, lumpy and pitted (the old part's two blobs, where they stand),
/// faces split off flat, lichen in patches on their tops and the damp soil line round their foot, cut off under
/// the ground, two pebbles at the foot, all plain matte: one draw call. Its far version, what the woods draw (some
/// hundreds of them), is two icosahedra lumped the same way, without the split faces (each cut adds triangles) or
/// the pebbles, lichen or soil line (a patch would take a whole face): 56 triangles, few enough for the woods to
/// keep drawing them as one mesh (nature.ts `SPLIT_TRIANGLES`), as the old part's.
pub fn rock() -> Vec<Obj> {
    let fractures = [
        ((0.35, -0.2, 1.0), 0.07),
        ((-0.9, 0.3, 0.35), 0.06),
        ((0.2, 1.0, 0.1), 0.05),
        ((0.95, -0.15, 0.25), 0.035),
        ((-0.3, -1.0, 0.45), 0.03),
    ];
    // (Far off, an icosahedron, its corners on the round and its faces inside it: a little wider to fill the same
    // box.)
    let main = stone(
        "rock_main",
        0.5,
        (0.0, 0.0, 0.0),
        M.stone,
        low((1.0, 0.85, 0.8), (1.08, 0.95, 0.8)),
        1.3,
        0.435,
        low(4, 1),
        low(&fractures[..], &[][..]),
        low(0.025, 0.0),
        (0.26, 0.08),
        GROUND,
    );
    let side = stone(
        "rock_side",
        0.25,
        (-0.42, -0.22, 0.0),
        M.stone_light,
        (1.0, 0.9, 0.9),
        3.7,
        0.24,
        low(3, 1),
        low(
            &[((-0.5, -0.6, 1.0), 0.04), ((-1.0, 0.2, 0.3), 0.025)][..],
            &[][..],
        ),
        low(0.03, 0.0),
        (0.26, 0.08),
        GROUND,
    );
    if low(true, false) {
        weather(main, 1.3, M.lichen, M.soil, 0.25);
        weather(side, 3.7, M.lichen, M.soil, 0.1);
    }
    let (lo, hi) = measure(&[main, side]);
    if low(true, false) {
        // Two pebbles broken off it, half sunk at its foot (inside its box, so it is centred as it was).
        for (k, (x, y, r, sd)) in [(0.36, -0.3, 0.07, 6.2), (-0.56, 0.08, 0.055, 8.9)]
            .into_iter()
            .enumerate()
        {
            let peb = stone(
                &format!("pebble{k}"),
                r,
                (x, y, 0.0),
                if k != 0 { M.stone_light } else { M.stone },
                (1.0, 0.8, 0.6),
                sd,
                0.035,
                2,
                &[],
                0.0,
                (0.26, 0.08),
                GROUND,
            );
            weather(peb, sd, M.lichen, M.soil, 0.6);
        }
    }
    // Centred where the old part's lumps are (game x 8.5 cm, depth 1 cm), by hand: a far version is built
    // without the Model's `centre` set.
    let obj = join_all("rock");
    obj.edit(|o| {
        o.loc += v3(
            -ROCK_CENTRE.0 - (lo[0] + hi[0]) / 2.0,
            -ROCK_CENTRE.1 - (lo[1] + hi[1]) / 2.0,
            0,
        )
    });
    vec![obj]
}

#[cfg(test)]
mod tests {
    use super::*;
    use artkit::mesh::{Mesh, edge};
    use artkit::scene;
    use std::collections::BTreeMap;

    /// How many faces run each side, by its two points.
    fn sides(mesh: &Mesh) -> BTreeMap<(u32, u32), usize> {
        let mut sides = BTreeMap::new();
        for f in 0..mesh.faces.len() {
            let c = mesh.corners(f);
            for i in 0..c.len() {
                *sides.entry(edge(c[i], c[(i + 1) % c.len()])).or_insert(0) += 1;
            }
        }
        sides
    }

    #[test]
    fn a_cut_stone_is_closed_triangles_behind_the_plane() {
        scene::reset();
        let stone = boulder!("stone", 0.3, seed = 1.5, subdivisions = 2);
        lump(stone, 0.08, 4.0, 1.5);
        let no = v3(0.3, -0.2, 1).normalized();
        cut(stone, no * 0.4, no);
        cut(stone, (0, 0, 0.1), (0, 0, -1));
        stone.read(|o| {
            assert!(o.mesh.faces.iter().all(|f| f.len == 3));
            assert!(
                sides(&o.mesh).values().all(|&n| n == 2),
                "the cuts are closed"
            );
            for v in &o.mesh.verts {
                assert!(v.dot(no) <= 0.4 + 1e-6 && v.z >= 0.1 - 1e-6);
            }
            // the faces that close the cuts are flat, the stone's own smooth
            let flat = o.mesh.faces.iter().filter(|f| !f.smooth).count();
            assert!(flat > 0 && flat < o.mesh.faces.len());
        });
    }
}

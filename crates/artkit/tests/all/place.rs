//! Outlines, the game's frame and the cut across a part.

use crate::common::*;
use artkit::forms::{bisect, rounded_rect, smooth_outline};
use artkit::math::V3;
use artkit::place::{Facing, GameFrame};
use artkit::scene;
use artkit::shapes::{Cube, cube};
use artkit::{fbox, fcyl, fextrude};

fn mat() -> artkit::materials::Mat {
    artkit::materials::Mat::new("body", "#F3F3F1", "gloss")
}

#[test]
fn a_rounded_rect_is_the_scripts_point_for_point() {
    // the fridge's side panel as the kitchen drew it, through its frame ('top': both ways mirrored): the outline
    // the golden slab was made from
    let side = rounded_rect(0.03, 0.622, 0.012, 2);
    assert_eq!(side.len(), 12);
    let golden = golden("bevel.txt");
    let line = golden
        .lines()
        .skip_while(|l| !l.starts_with("rec kitchen/fridge/side_left "))
        .find(|l| l.starts_with("loops "))
        .unwrap();
    let want: Vec<(f64, f64)> = line[6..]
        .split(' ')
        .map(|p| {
            let (a, b) = p.split_once(',').unwrap();
            (f64w(a), f64w(b))
        })
        .collect();
    let flipped: Vec<(f64, f64)> = side.iter().map(|p| (-p.0, -p.1)).collect();
    assert_eq!(flipped, want);
    // corners no wider than half a side; a corner of one segment is a chamfer
    assert_eq!(rounded_rect(0.2, 0.1, 0.5, 4).len(), 18);
    assert_eq!(rounded_rect(0.07, 0.012, 0.006, 1).len(), 6);
    // corner cutting: two points a quarter in from each corner
    let cut = smooth_outline(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)], 1, true);
    assert_eq!(cut.len(), 8);
    assert_eq!(
        (cut[0], cut[1], cut[2]),
        ((0.25, 0.0), (0.75, 0.0), (1.0, 0.25))
    );
    let open = smooth_outline(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0)], 2, false);
    assert_eq!(
        (open.len(), open[0], open[open.len() - 1]),
        (12, (0.0, 0.0), (1.0, 1.0))
    );
}

#[test]
fn the_games_frame_places_parts_in_blenders_axes() {
    const F: GameFrame = GameFrame::new(0.094);
    let at = F.at(0.364, 0.643, 0.195);
    assert_eq!((at.x, at.y, at.z), (-0.364, -0.195 - 0.094, 0.643));
    scene::reset();
    // width, height and depth become Blender's x, z and y
    let b = fbox!(
        F,
        "top",
        0.74,
        0.03,
        0.622,
        (0, 1.785, -0.144),
        mat(),
        bevel = 0.012
    );
    let (lo, hi) = artkit::parts::measure(&[b]);
    let size: Vec<f64> = (0..3)
        .map(|i| ((hi[i] - lo[i]) * 1e6).round() / 1e6)
        .collect();
    assert_eq!(size, [0.74, 0.622, 0.03]);
    // (measured with the bevel it is still to be given; rounded when it is joined or written)
    assert_eq!(b.read(|o| o.mesh.verts.len()), 8);
    assert_eq!(artkit::bevel::apply(b).read(|o| o.mesh.verts.len()), 56);
    let c = fcyl!(
        F,
        "jar",
        0.034,
        0.09,
        (0.1, 0.605, 0.134),
        mat(),
        verts = 10,
        bevel = 0.004,
        bevel_segments = 1
    );
    assert_eq!(
        artkit::bevel::apply(c).read(|o| (o.mesh.verts.len(), o.loc.z)),
        (40, 0.605)
    );
    // a slab lying flat: its outline's x and depth mirrored into Blender's x and y
    let s = fextrude!(
        F,
        "side",
        &rounded_rect(0.03, 0.622, 0.012, 2),
        1.74,
        (-0.355, 0.93, -0.144),
        mat(),
        plane = Facing::Top
    );
    let (lo, hi) = artkit::parts::measure(&[s]);
    assert!((hi[2] - lo[2] - 1.74).abs() < 1e-6 && (hi[1] - lo[1] - 0.622).abs() < 1e-6);
    // depths redrawn: a box keeps its place between its mapped back and front, its bevel no wider than fits
    let shallow = GameFrame::with_depths(0.0, &[(-0.4, -0.3), (0.4, 0.3)]);
    assert_eq!(
        (
            shallow.depth(-0.5),
            shallow.depth(0.0),
            shallow.depth(0.2),
            shallow.depth(0.6)
        ),
        (-0.4, 0.0, 0.15000000000000002, 0.4999999999999999)
    );
    scene::reset();
    let b = fbox!(
        shallow,
        "thin",
        0.5,
        0.2,
        0.02,
        (0, 0.1, 0.0),
        mat(),
        bevel = 0.02,
        bevel_segments = 1
    );
    let (lo, hi) = artkit::parts::measure(&[b]);
    assert!((hi[1] - lo[1] - 0.015).abs() < 1e-6, "{}", hi[1] - lo[1]);
}

#[test]
fn a_part_is_cut_across_and_closed() {
    scene::reset();
    let whole = cube("block", 1.0, Cube::default());
    let cut = bisect(whole, (0.0, 0.0, 0.1), (0.0, 0.0, 1.0), true);
    let mesh = cut.read(|o| o.mesh.clone());
    // the four top corners gone, four new ones where the sides cross the plane, the cut closed with one face
    assert_eq!((mesh.verts.len(), mesh.faces.len()), (8, 6));
    let top = mesh.verts.iter().map(|v| v.z).fold(f32::MIN, f32::max);
    assert!((top - 0.1).abs() < 1e-6, "{top}");
    let cap = (0..6)
        .find(|&f| {
            mesh.corners(f)
                .iter()
                .all(|&v| mesh.verts[v as usize].z > 0.09)
        })
        .unwrap();
    assert!(mesh.face_normal(cap).z > 0.999);
    // open: the same without its cap
    scene::reset();
    let open = bisect(
        cube("block", 1.0, Cube::default()),
        (0.0, 0.0, 0.1),
        (0.0, 0.0, 1.0),
        false,
    );
    assert_eq!(open.read(|o| o.mesh.faces.len()), 5);
    // a plane that misses leaves the part whole; one given in the world cuts a placed part where it stands
    scene::reset();
    let missed = bisect(
        cube("block", 1.0, Cube::default()),
        (0.0, 0.0, 2.0),
        (0.0, 0.0, 1.0),
        true,
    );
    assert_eq!(
        missed.read(|o| (o.mesh.verts.len(), o.mesh.faces.len())),
        (8, 6)
    );
    scene::reset();
    let placed = cube(
        "block",
        1.0,
        Cube {
            at: (3.0, 0.0, 0.0).into(),
            rot: (0.0, 0.0, 45.0).into(),
            ..Cube::default()
        },
    );
    // (the plane goes through four of its corners: they count as on it within a tenth of a millimetre, which
    // Blender's `dist` says; with none, as `bisect` cuts, each would be a sliver's corner)
    let world = placed.matrix_world();
    placed.edit(|o| o.mesh.transform(&world));
    let cut = artkit::bisect_plane!(
        placed,
        (3.0, 0.0, 0.0),
        (1.0, 0.0, 0.0),
        dist = 0.0001,
        clear_outer = true
    );
    artkit::edit::holes_fill(placed, &cut);
    placed.edit(|o| o.mesh.transform(&world.inverted().unwrap()));
    let world: Vec<V3> = placed.world_points();
    assert_eq!(world.len(), 6);
    assert!(world.iter().all(|p| p.x <= 3.0 + 1e-6));
}

#[test]
fn a_rounded_rect_with_no_corner_segments_is_still_a_rectangle() {
    let outline = rounded_rect(1.0, 0.5, 0.1, 0);
    assert_eq!(outline.len(), 8);
    assert!(outline.iter().all(|p| p.0.is_finite() && p.1.is_finite()));
}

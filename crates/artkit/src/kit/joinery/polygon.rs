//! Polygons in a plane: their turns, areas and where a point lies.

/// A point seen along the way the cutter is pushed in.
pub(super) type P2 = [f64; 2];

/// Twice the area of the triangle a, b, c: over zero where c is to the left of a to b. Its sign is exact for
/// points that were singles.
pub(super) fn turn(a: P2, b: P2, c: P2) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

/// Twice a polygon's area, over zero where it runs anticlockwise.
pub(super) fn area2(p: &[P2]) -> f64 {
    (0..p.len())
        .map(|i| {
            let (a, b) = (p[i], p[(i + 1) % p.len()]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum()
}

/// How far along a to b the point p lies (0 at a, 1 at b).
pub(super) fn along(a: P2, b: P2, p: P2) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / (dx * dx + dy * dy)
}

/// Cutter points this near seen from the front are one corner of its outline (m): a bevelled box's front and
/// back corners, each rounded by itself, a unit in the last place apart.
pub(super) const SNAP: f64 = 1e-6;

/// A new point of a cut body, by what makes it: where a body's edge crosses a side of the outline, an outline's
/// corner on a body's edge or inside a face, a cutter's own point.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Made {
    Crossing(u32, u32, usize),
    OnEdge(u32, u32, usize),
    InFace(usize, usize),
    Tool(usize),
}

/// A point of a face's part inside the outline: a corner of the face, a corner of the outline, or where a side
/// of each cross; the face's sides and the outline's it lies on.
pub(super) struct Meet {
    pub(super) at: P2,
    pub(super) corner: Option<usize>,
    pub(super) crossing: Option<(usize, usize)>,
    pub(super) outline: Option<usize>,
    pub(super) on_face: Vec<usize>,
    pub(super) on_outline: Vec<usize>,
}

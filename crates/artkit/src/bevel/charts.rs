//! Charts: which old face a strip or a corner's polygon belongs to.

use super::*;

/// Which old face a strip or a corner's polygon takes its texture from where the faces round it don't share one
/// (Blender's `choose_rep_face`): each old face's number among the groups of faces whose UVs run on across their
/// shared edges, its material and its middle.
pub(super) struct Charts<'a> {
    pub(super) old: &'a Mesh,
    pub(super) uvs: &'a [[f32; 2]],
    /// (Only for an odd number of strips, as in Blender; none otherwise.)
    pub(super) group: Option<Vec<u32>>,
    pub(super) sides: Sides,
}

impl<'a> Charts<'a> {
    pub(super) fn new(
        old: &'a Mesh,
        uvs: &'a [[f32; 2]],
        odd: bool,
        sides_from: Option<usize>,
    ) -> Charts<'a> {
        let mut sides = Sides::new();
        for f in 0..old.faces.len() {
            let c = old.corners(f);
            let at = old.faces[f].start as usize;
            for i in 0..c.len() {
                let j = (i + 1) % c.len();
                let (a, b) = if c[i] < c[j] { (i, j) } else { (j, i) };
                sides
                    .entry(edge(c[i], c[j]))
                    .or_default()
                    .push((f as u32, at + a, at + b));
            }
        }
        let mut charts = Charts {
            old,
            uvs,
            group: None,
            sides,
        };
        if odd {
            charts.group = Some(charts.groups(sides_from));
        }
        charts
    }

    /// Whether the UVs run on from face `f` to face `g` across the edge `e` they share: the same at both its ends.
    pub(super) fn runs_on(&self, e: (u32, u32), f: u32, g: u32) -> bool {
        let Some(faces) = self.sides.get(&edge(e.0, e.1)) else {
            return false;
        };
        if faces.len() != 2 {
            return false;
        }
        let of = |face: u32| faces.iter().find(|s| s.0 == face);
        match (of(f), of(g)) {
            (Some(a), Some(b)) if f != g => {
                self.uvs[a.1] == self.uvs[b.1] && self.uvs[a.2] == self.uvs[b.2]
            }
            _ => false,
        }
    }

    /// The middle of a face's bounds.
    fn middle(&self, f: u32) -> V3 {
        let c = self.old.corners(f as usize);
        let (mut lo, mut hi) = (self.old.verts[c[0] as usize], self.old.verts[c[0] as usize]);
        for &v in c {
            let p = self.old.verts[v as usize];
            lo = V3::new(lo.x.min(p.x), lo.y.min(p.y), lo.z.min(p.z));
            hi = V3::new(hi.x.max(p.x), hi.y.max(p.y), hi.z.max(p.z));
        }
        V3::new(
            0.5 * (lo.x + hi.x),
            0.5 * (lo.y + hi.y),
            0.5 * (lo.z + hi.z),
        )
    }

    /// Each face's group, numbered in the order the faces come to them; then the topmost face's group takes the
    /// first face's number and the bottom-most's the second's, as Blender has it. The groups that start among a
    /// slab's side faces (`sides_from`) come in an order of Blender's that can't be told: they are numbered the
    /// larger first, which is the likelier (a group of more faces is likelier to hold the first of them).
    fn groups(&self, sides_from: Option<usize>) -> Vec<u32> {
        let nf = self.old.faces.len();
        let mut group = vec![u32::MAX; nf];
        let mut count = 0;
        for start in 0..nf {
            if group[start] != u32::MAX {
                continue;
            }
            let mut stack = vec![start as u32];
            while let Some(f) = stack.pop() {
                if group[f as usize] != u32::MAX {
                    continue;
                }
                group[f as usize] = count;
                let c = self.old.corners(f as usize);
                for i in 0..c.len() {
                    let e = (c[i], c[(i + 1) % c.len()]);
                    for &(g, _, _) in &self.sides[&edge(e.0, e.1)] {
                        if g != f && group[g as usize] == u32::MAX && self.runs_on(e, f, g) {
                            stack.push(g);
                        }
                    }
                }
            }
            count += 1;
        }
        if let Some(from) = sides_from {
            let mut first = vec![usize::MAX; count as usize];
            let mut size = vec![0_usize; count as usize];
            for (f, &g) in group.iter().enumerate() {
                first[g as usize] = first[g as usize].min(f);
                size[g as usize] += 1;
            }
            let fixed = first.iter().filter(|&&f| f < from).count();
            let mut loose: Vec<usize> = (0..count as usize).filter(|&g| first[g] >= from).collect();
            loose.sort_by_key(|&g| (std::cmp::Reverse(size[g]), first[g]));
            let mut renumbered = vec![0_u32; count as usize];
            let mut next = 0;
            for g in 0..count as usize {
                if first[g] < from {
                    renumbered[g] = next;
                    next += 1;
                }
            }
            debug_assert_eq!(next as usize, fixed);
            for (i, &g) in loose.iter().enumerate() {
                renumbered[g] = (fixed + i) as u32;
            }
            for g in group.iter_mut() {
                *g = renumbered[*g as usize];
            }
        }
        if count <= 1 {
            return group;
        }
        let (mut top, mut bottom) = ((f32::MIN, 0), (f32::MAX, 0));
        for (f, &g) in group.iter().enumerate() {
            let z = self.middle(f as u32).z;
            if z > top.0 {
                top = (z, g);
            }
            if z < bottom.0 {
                bottom = (z, g);
            }
        }
        let swap = |group: &mut Vec<u32>, a: u32, b: u32| {
            for g in group.iter_mut() {
                *g = if *g == a {
                    b
                } else if *g == b {
                    a
                } else {
                    *g
                };
            }
        };
        let first = group[0];
        swap(&mut group, first, top.1);
        if bottom.1 != top.1 {
            // (the first face's group, if it was the bottom-most, has the topmost's number now)
            let lowest = if bottom.1 == 0 { top.1 } else { bottom.1 };
            let second = group[1.min(nf - 1)];
            swap(&mut group, second, lowest);
        }
        group
    }

    /// The one of `faces` a texture is taken from: the lowest group, then material, then middle (z, x, y), the
    /// first of them where all are alike.
    pub(super) fn pick(&self, faces: &[u32]) -> u32 {
        let values: Vec<[f32; 5]> = faces
            .iter()
            .map(|&f| {
                let m = self.middle(f);
                [
                    self.group.as_ref().map_or(0.0, |g| g[f as usize] as f32),
                    self.old.faces[f as usize].mat as f32,
                    m.z,
                    m.x,
                    m.y,
                ]
            })
            .collect();
        let mut viable = vec![true; faces.len()];
        let mut left = faces.len();
        let mut best: Option<usize> = None;
        for value in 0..5 {
            if left <= 1 {
                break;
            }
            for f in 0..faces.len() {
                if !viable[f] || Some(f) == best {
                    continue;
                }
                let Some(b) = best else {
                    best = Some(f);
                    continue;
                };
                if values[f][value] < values[b][value] {
                    best = Some(f);
                    for earlier in viable.iter_mut().take(f) {
                        if std::mem::replace(earlier, false) {
                            left -= 1;
                        }
                    }
                } else if values[f][value] > values[b][value] {
                    viable[f] = false;
                    left -= 1;
                }
            }
        }
        faces[best.unwrap_or(0)]
    }
}

impl Charts<'_> {
    /// The old face a corner's polygon takes its texture from (Blender's `frep_for_center_poly`): of the faces
    /// picked either side of each bevelled edge, in Blender's order of the vertex's edges, the pick of those the
    /// corner's new points, laid onto the face, keep an area on; the first of them all where none does (a corner
    /// cut back by under 14 mm at right angles has too little on any).
    pub(super) fn centre(&self, new: &Mesh, vert: u32, centre: &Centre) -> u32 {
        let (edges, ring) = centre;
        let points: Vec<V3> = ring.iter().map(|&v| new.verts[v as usize]).collect();
        let (mut choices, mut any): (Vec<u32>, Option<u32>) = (Vec::new(), None);
        for &(_, before, after) in edges {
            let f = self.pick(&[before, after]);
            any.get_or_insert(f);
            if !choices.contains(&f) && !self.bad_poly(vert, f, &points) {
                choices.push(f);
            }
        }
        if choices.is_empty() {
            any.unwrap_or(0)
        } else {
            self.pick(&choices)
        }
    }

    /// Whether a corner's new points `points`, each inside the old face `f` as it is and the others on the
    /// nearer of the face's two sides at the vertex, span next to no area (Blender's `is_bad_uv_poly`): a
    /// texture read from that face would have none on the corner's polygon.
    fn bad_poly(&self, vert: u32, f: u32, points: &[V3]) -> bool {
        let c = self.old.corners(f as usize);
        let Some(at) = c.iter().position(|&v| v == vert) else {
            return false;
        };
        let n = c.len();
        let co = |v: u32| self.old.verts[v as usize];
        let corners: Vec<V3> = c.iter().map(|&v| co(v)).collect();
        let normal = normal_newell(&corners);
        // two axes across the face
        let (x, _) = normalize_v3(co(c[(at + 1) % n]) - co(vert));
        let y = normal.cross(x);
        let flat = |p: V3| [dot_v3(p, x) as f64, dot_v3(p, y) as f64];
        let outline: Vec<[f64; 2]> = corners.iter().map(|&p| flat(p)).collect();
        // (inside: the outline turns all the way round the point; a point on its edge is not)
        let inside = |p: [f64; 2]| {
            let mut turned = 0.0_f64;
            for i in 0..outline.len() {
                let (a, b) = (outline[i], outline[(i + 1) % outline.len()]);
                let (a, b) = ([a[0] - p[0], a[1] - p[1]], [b[0] - p[0], b[1] - p[1]]);
                turned += (a[0] * b[1] - a[1] * b[0]).atan2(a[0] * b[0] + a[1] * b[1]);
            }
            turned.abs() > 4.0
        };
        let sides = [
            (co(vert), co(c[(at + 1) % n])),
            (co(c[(at + n - 1) % n]), co(vert)),
        ];
        let laid: Vec<[f64; 2]> = points
            .iter()
            .map(|&p| {
                if inside(flat(p)) {
                    return flat(p);
                }
                let (a, b) = (
                    on_segment(p, sides[0].0, sides[0].1),
                    on_segment(p, sides[1].0, sides[1].1),
                );
                flat(if dot_v3(a - p, a - p) <= dot_v3(b - p, b - p) {
                    a
                } else {
                    b
                })
            })
            .collect();
        let mut area = 0.0_f64;
        for i in 0..laid.len() {
            let (a, b) = (laid[i], laid[(i + 1) % laid.len()]);
            area += a[0] * b[1] - a[1] * b[0];
        }
        (area.abs() * 0.5) < EPSILON_BIG as f64
    }
}

//! A corner's patch of an odd number of segments, and what is carried across.

use super::*;

/// A corner's patch of an odd number of segments, for its texture (Blender's bevel_build_rings): which old face
/// each corner of its quads reads, and the old edge a corner on the patch's middle lines is read on.
pub(super) struct Patch<'a> {
    pub(super) vert: u32,
    pub(super) across: &'a [(u32, u32, u32)],
    /// For each new point, whether its face is taken before the next point's where the two meet.
    pub(super) beats: Vec<bool>,
    /// Whether the UVs break across each new point's bevelled edge.
    pub(super) seam: Vec<bool>,
    /// The face the patch's middle takes its texture from.
    pub(super) centre: u32,
}

impl<'a> Patch<'a> {
    pub(super) fn new(
        charts: &Charts,
        vert: u32,
        across: &'a [(u32, u32, u32)],
        centre: u32,
    ) -> Patch<'a> {
        let n = across.len();
        let beats = (0..n)
            .map(|i| charts.pick(&[across[i].1, across[(i + 1) % n].1]) == across[i].1)
            .collect();
        let seam = across
            .iter()
            .map(|&(to, before, after)| !charts.runs_on((vert, to), before, after))
            .collect();
        Patch {
            vert,
            across,
            beats,
            seam,
            centre,
        }
    }

    fn any_seam(&self) -> bool {
        self.seam.iter().any(|&s| s)
    }

    /// The face of the new point `i`.
    fn rep(&self, i: usize) -> u32 {
        self.across[i % self.across.len()].1
    }

    /// The bevelled edge across from the new point `i`.
    fn old_edge(&self, i: usize) -> (u32, u32) {
        (self.vert, self.across[i % self.across.len()].0)
    }

    /// That edge, if the UVs break across it.
    fn edge(&self, i: usize) -> Option<(u32, u32)> {
        self.seam[i % self.across.len()].then(|| self.old_edge(i))
    }

    /// The face corner `corner` of the quad at step `k` from the new point `i` reads.
    fn face(&self, i: usize, k: usize, corner: usize, half: usize) -> u32 {
        if k < half {
            self.rep(i)
        } else if self.seam[i] {
            self.rep(if self.beats[i] { i } else { i + 1 })
        } else {
            self.rep(if corner == 0 || corner == 3 { i } else { i + 1 })
        }
    }

    /// The edge a point on the patch's middle lines is read on: the point at ring `jj`, step `kk` from `i`.
    fn snap(&self, i: usize, jj: usize, kk: usize, half: usize) -> Option<(u32, u32)> {
        let n = self.across.len();
        let prev = (i + n - 1) % n;
        let centre = |i: usize| {
            let (prev, next) = ((i + n - 1) % n, (i + 1) % n);
            if self.beats[prev] && self.rep(prev) == self.centre {
                self.edge(prev)
            } else if !self.beats[i] && self.rep(next) == self.centre {
                self.edge(i)
            } else {
                None
            }
        };
        match (jj < half, kk) {
            (true, kk) if kk < half => None,
            (true, kk) if kk == half => (!self.beats[i]).then(|| self.edge(i)).flatten(),
            (true, _) => self.beats[i].then(|| self.edge(i)).flatten(),
            (false, kk) if kk < half => self.beats[prev].then(|| self.edge(prev)).flatten(),
            (false, kk) if kk == half => centre(i),
            (false, _) => centre((i + 1) % n),
        }
    }
}

/// The point of the segment a-b nearest to `p`.
pub(super) fn on_segment(p: V3, a: V3, b: V3) -> V3 {
    let d = b - a;
    let length = dot_v3(d, d);
    if length == 0.0 {
        return a;
    }
    let t = (dot_v3(p - a, d) / length).clamp(0.0, 1.0);
    V3::new(a.x + d.x * t, a.y + d.y * t, a.z + d.z * t)
}

/// Each new corner's UV, as Blender's bevel carries them: its source face's own way across the face, at the
/// corner's point. A strip between two faces whose UVs don't run on across the edge takes one face's texture
/// (the middle strip of an odd number; of an even number, half the strips each), its points beyond that face
/// read on the old edge; so does a corner's polygon, its points off the face read on the face's nearer edge at
/// the vertex.
pub fn carried(
    old: &Mesh,
    uvs: &[[f32; 2]],
    new: &Mesh,
    made: &Bevelled,
    sides_from: Option<usize>,
) -> Vec<[f32; 2]> {
    /// A face's UVs as a point of it, two of its sides and how the UV changes along each.
    struct Frame {
        p0: V3,
        u0: [f64; 2],
        sides: [V3; 2],
        /// The inverse of the sides' dot products with each other.
        inverse: [f64; 3],
        rates: [[f64; 2]; 2],
    }
    let frames: Vec<Option<Frame>> = (0..old.faces.len())
        .map(|f| {
            let c = old.corners(f);
            let at = old.faces[f].start as usize;
            let p0 = old.verts[c[0] as usize];
            let uv = |i: usize| [uvs[at + i][0] as f64, uvs[at + i][1] as f64];
            let e1 = old.verts[c[1] as usize] - p0;
            // with the corner farthest from the first side's line
            let height = |i: usize| e1.cross(old.verts[c[i] as usize] - p0).length();
            let far = (2..c.len()).max_by(|&a, &b| height(a).total_cmp(&height(b)))?;
            let e2 = old.verts[c[far] as usize] - p0;
            let (g11, g12, g22) = (e1.dot(e1), e1.dot(e2), e2.dot(e2));
            let det = g11 * g22 - g12 * g12;
            if det.abs() < 1e-30 {
                return None;
            }
            let (u0, u1, u2) = (uv(0), uv(1), uv(far));
            Some(Frame {
                p0,
                u0,
                sides: [e1, e2],
                inverse: [g22 / det, -g12 / det, g11 / det],
                rates: [
                    [u1[0] - u0[0], u1[1] - u0[1]],
                    [u2[0] - u0[0], u2[1] - u0[1]],
                ],
            })
        })
        .collect();
    let ns = made.segments;
    let (odd, half) = (ns % 2 == 1, ns / 2);
    let charts = Charts::new(old, uvs, odd, sides_from);
    let mut source = made.source.clone();
    let mut read_at: Vec<Option<(u32, u32)>> = vec![None; source.len()];
    let centre_of = |vert: u32| charts.centre(new, vert, &made.centres[&vert]);
    for pick in &made.picks {
        match pick {
            Pick::Strip { at, edge, faces, k } => {
                let (f1, f2) = *faces;
                // (corners 0 and 1 are the nearer to f1, 2 and 3 to f2)
                let (from, far): (u32, &[usize]) = if odd && *k == half + 1 {
                    if charts.runs_on(*edge, f1, f2) {
                        source[*at..*at + 2].fill(f1);
                        source[*at + 2..*at + 4].fill(f2);
                        continue;
                    }
                    let from = charts.pick(&[f1, f2]);
                    (from, if from == f1 { &[2, 3] } else { &[0, 1] })
                } else if !odd && *k == half {
                    (f1, &[2, 3])
                } else if !odd && *k == half + 1 {
                    (f2, &[0, 1])
                } else if odd
                    && ns > 1
                    && (*k == half || *k == half + 2)
                    && !charts.runs_on(*edge, f1, f2)
                    && (charts.pick(&[f1, f2]) == f1) == (*k == half + 2)
                {
                    // the strip beside the middle one, on the face the middle one doesn't read: its points on
                    // the middle strip's side are read on the old edge too, so its face's texture runs up to it
                    if *k == half {
                        (f1, &[2, 3])
                    } else {
                        (f2, &[0, 1])
                    }
                } else {
                    (if *k <= half { f1 } else { f2 }, &[])
                };
                source[*at..*at + 4].fill(from);
                for &i in far {
                    read_at[*at + i] = Some(*edge);
                }
            }
            Pick::Patch {
                at,
                vert,
                place,
                across,
            } => {
                let patch = Patch::new(&charts, *vert, across, centre_of(*vert));
                let (i, j, k) = *place;
                if !odd {
                    // an even number: the quads that touch the patch's middle lines read their points there on
                    // the old edges
                    let n = across.len();
                    let on_next = (k + 1 == half).then(|| patch.old_edge(i));
                    let on_prev = (j + 1 == half).then(|| patch.old_edge(i + n - 1));
                    source[*at..*at + 4].fill(patch.rep(i));
                    read_at[*at + 1] = on_next;
                    read_at[*at + 2] = on_next.or(on_prev);
                    read_at[*at + 3] = on_prev;
                    continue;
                }
                for corner in 0..4 {
                    let jj = j + (corner >= 2) as usize;
                    let kk = k + (corner == 1 || corner == 2) as usize;
                    source[*at + corner] = patch.face(i, k, corner, half);
                    read_at[*at + corner] = patch.snap(i, jj, kk, half);
                }
            }
            Pick::Corner {
                at,
                vert,
                on,
                edges,
                across,
            } => {
                if !across.is_empty() && odd {
                    // the polygon in the middle of a patch: its corners as the quads round it have them
                    let patch = Patch::new(&charts, *vert, across, centre_of(*vert));
                    for i in 0..on.len() {
                        source[*at + i] = if patch.any_seam() {
                            patch.centre
                        } else {
                            patch.rep(i)
                        };
                        read_at[*at + i] = patch.snap(i, half, half, half);
                    }
                    continue;
                }
                if edges
                    .iter()
                    .all(|&(to, before, after)| charts.runs_on((*vert, to), before, after))
                {
                    continue;
                }
                let from = centre_of(*vert);
                // the face's two sides at the vertex, in the face's order of them
                let c = old.corners(from as usize);
                let Some(p) = c.iter().position(|&v| v == *vert) else {
                    continue;
                };
                let n = c.len();
                let sides = if p == 0 {
                    [(c[0], c[1]), (c[n - 1], c[0])]
                } else {
                    [(c[p - 1], c[p]), (c[p], c[(p + 1) % n])]
                };
                for (i, &face) in on.iter().enumerate() {
                    source[*at + i] = from;
                    if face != from {
                        let point = new.verts[new.loops[*at + i] as usize];
                        let apart = |s: (u32, u32)| {
                            let q =
                                on_segment(point, old.verts[s.0 as usize], old.verts[s.1 as usize]);
                            dot_v3(point - q, point - q)
                        };
                        read_at[*at + i] = Some(if apart(sides[1]) < apart(sides[0]) {
                            sides[1]
                        } else {
                            sides[0]
                        });
                    }
                }
            }
        }
    }
    let mut out = vec![[0.0_f32; 2]; new.loops.len()];
    for (l, &f) in source.iter().enumerate() {
        if let Some(frame) = &frames[f as usize] {
            let mut point = new.verts[new.loops[l] as usize];
            if let Some((a, b)) = read_at[l] {
                point = on_segment(point, old.verts[a as usize], old.verts[b as usize]);
            }
            let d = point - frame.p0;
            let (x, y) = (d.dot(frame.sides[0]), d.dot(frame.sides[1]));
            let a = frame.inverse[0] * x + frame.inverse[1] * y;
            let b = frame.inverse[1] * x + frame.inverse[2] * y;
            out[l] = [
                (frame.u0[0] + a * frame.rates[0][0] + b * frame.rates[1][0]) as f32,
                (frame.u0[1] + a * frame.rates[0][1] + b * frame.rates[1][1]) as f32,
            ];
        }
    }
    merge_uvs(old, uvs, new, made, &source, &mut out);
    out
}

/// Two UVs Blender takes for one: within a ten-thousandth each way.
pub(super) fn uv_joined(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).abs() <= 0.0001 && (a[1] - b[1]).abs() <= 0.0001
}

/// Blender's last pass over a bevel's UVs (`bevel_merge_uvs`): at each new point, the corners of the new faces
/// whose textures come from one old face, or from old faces whose UVs were joined at the old vertex the point
/// was made at, are given the mean of their UVs. Where every face has a chart of its own nothing is joined and
/// nothing moves; where two faces share their UVs along an edge (a box of square section under the box
/// projection), the corner polygon's corner, read on that edge, is averaged in with the two beside it, which
/// pulls a strip that would have no area in the texture a little off its line.
/// `source`: the old face each new corner's UV was read from.
pub(super) fn merge_uvs(
    old: &Mesh,
    uvs: &[[f32; 2]],
    new: &Mesh,
    made: &Bevelled,
    source: &[u32],
    out: &mut [[f32; 2]],
) {
    // each old corner's group among the corners of its vertex whose UVs are joined (the first it joins)
    let mut at_vertex: BTreeMap<u32, Vec<usize>> = BTreeMap::new();
    for (l, &v) in old.loops.iter().enumerate() {
        at_vertex.entry(v).or_default().push(l);
    }
    let mut group = vec![0_usize; old.loops.len()];
    for corners in at_vertex.values() {
        let mut groups: Vec<Vec<usize>> = Vec::new();
        for &l in corners {
            let joins = groups
                .iter()
                .position(|g| g.iter().any(|&m| uv_joined(uvs[l], uvs[m])));
            let g = joins.unwrap_or_else(|| {
                groups.push(Vec::new());
                groups.len() - 1
            });
            groups[g].push(l);
            group[l] = g;
        }
    }
    // the new corners at each new point, by the group of the face they stand for (its first corner's source)
    let mut buckets: BTreeMap<(u32, usize), Vec<usize>> = BTreeMap::new();
    for face in &new.faces {
        let (start, end) = (face.start as usize, (face.start + face.len) as usize);
        let from = source[start] as usize;
        for (l, &v) in new.loops.iter().enumerate().take(end).skip(start) {
            let old_vertex = made.made_at[v as usize];
            if old_vertex == u32::MAX {
                continue;
            }
            let c = old.corners(from);
            if let Some(i) = c.iter().position(|&w| w == old_vertex) {
                let g = group[old.faces[from].start as usize + i];
                buckets.entry((v, g)).or_default().push(l);
            }
        }
    }
    for corners in buckets.values().filter(|c| c.len() > 1) {
        let mut sum = [0.0_f32; 2];
        for &l in corners {
            sum = [sum[0] + out[l][0], sum[1] + out[l][1]];
        }
        let k = 1.0 / corners.len() as f32;
        for &l in corners {
            out[l] = [sum[0] * k, sum[1] * k];
        }
    }
}

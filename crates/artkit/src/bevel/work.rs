//! The bevel's working state: what it collects as it walks the mesh.
//!
//! Split from `bevel.rs` (see `mod.rs`).

use super::*;

pub(crate) struct Work<'a> {
    pub(crate) mesh: &'a Mesh,
    pub(crate) normals: Vec<V3>,
    pub(crate) vertex_normals: Vec<V3>,
    /// The face and corner a directed edge leaves from.
    pub(crate) leaving: BTreeMap<(u32, u32), (u32, u32)>,
    pub(crate) corners: BTreeMap<u32, Corner>,
}

impl Work<'_> {
    pub(crate) fn co(&self, v: u32) -> V3 {
        self.mesh.verts[v as usize]
    }

    /// The angle at `b` between `a` and `c`.
    fn angle3(&self, a: u32, b: u32, c: u32) -> f32 {
        angle(self.co(a) - self.co(b), self.co(c) - self.co(b))
    }

    /// The largest offset the edge `i` of `corner` has room for before the cut faces collide (Blender's
    /// `geometry_collide_offset`, in its arithmetic: a slab's clamped corners close to the last bit or don't).
    pub(crate) fn collide_offset(&self, corner: &Corner, i: usize, offset: f32) -> f32 {
        let no_collide = offset + 1e6;
        let mut limit = no_collide;
        let eb = &corner.edges[i];
        let ea = &corner.edges[(i + 1) % corner.edges.len()];
        let k = |bev: bool| if bev { offset } else { 0.0 };
        let (ka, kb) = (k(ea.bev), k(eb.bev));
        let (va, vb, vc) = (ea.to, corner.v, eb.to);
        let Some(&(f, at)) = self.leaving.get(&(vb, vc)) else {
            return limit;
        };
        // the face between the two edges, round from the corner before this vertex
        let c = self.mesh.corners(f as usize);
        let n = c.len();
        let round = |j: usize| c[(at as usize + n - 1 + j) % n];
        let vd = round(3);
        let ec = self.corners.get(&vc).map(|c| {
            let other = c.half_to(vb);
            &c.edges[(other + c.edges.len() - 1) % c.edges.len()]
        });
        let kc = ec.map(|e| k(e.bev)).unwrap_or(0.0);
        if va == vc || vd == vb {
            return limit;
        }
        let (th1, th2) = (self.angle3(va, vb, vc), self.angle3(vb, vc, vd));
        let (sin1, sin2, cos1, cos2) = (sinf(th1), sinf(th2), cosf(th1), cosf(th2));
        let safe = |a: f32, b: f32| if b == 0.0 { 0.0 } else { a / b };
        // where the edge closes up: its two ends' cuts meeting on it
        let on_b = safe(ka + cos1 * kb, sin1) + safe(kc + cos2 * kb, sin2);
        if on_b > EPSILON {
            let room = offset * (len_v3(self.co(vb) - self.co(vc)) / on_b);
            if room > EPSILON {
                limit = room;
            }
        }
        // where a point sliding along an edge left as it is reaches that edge's end (on along the edges that
        // run straight on from it)
        if kb > f32::EPSILON && (ka == 0.0 || kc == 0.0) {
            let kb = offset / kb;
            let pi = std::f32::consts::PI;
            let side = |j: usize| len_v3(self.co(round(j)) - self.co(round(j + 1)));
            if ka == 0.0 && round(0) == va {
                // (back from the edge before the vertex: corner n - k is k corners before it)
                let (mut back, mut exterior) = (0, pi - th1);
                let mut slide = side(0) * sinf(exterior);
                while exterior < 0.0001 && back < n {
                    back += 1;
                    let j = n - back;
                    exterior += pi - self.angle3(round(j), round(j + 1), round(j + 2));
                    slide += side(j) * sinf(exterior);
                }
                limit = (slide * kb).min(limit);
            }
            if kc == 0.0 {
                let (mut j, mut exterior) = (2, pi - th2);
                let mut slide = side(j) * sinf(exterior);
                while exterior < 0.0001 && j < n + 2 {
                    j += 1;
                    exterior += pi - self.angle3(round(j - 1), round(j), round(j + 1));
                    slide += side(j) * sinf(exterior);
                }
                limit = (slide * kb).min(limit);
            }
        }
        limit
    }
    /// Where the offset lines of two bevelled edges of a vertex meet, on the face (or faces) between them.
    fn offset_meet(
        &self,
        corner: &Corner,
        i1: usize,
        i2: usize,
        face: Option<u32>,
        between: bool,
    ) -> V3 {
        let n = corner.edges.len();
        let (e1, e2) = (&corner.edges[i1], &corner.edges[i2]);
        let v = self.co(corner.v);
        let dir1 = v - self.co(e1.to);
        let dir2 = self.co(e2.to) - v;
        let ang = angle(dir1, dir2);
        let face_no = |f: Option<u32>| match f {
            Some(f) => self.normals[f as usize],
            None => self.vertex_normals[corner.v as usize],
        };
        if ang < EPSILON_ANG {
            // one line: square to both, from the vertex
            let norm_v = face_no(face);
            let (perp, _) = normalize_v3((dir1 + dir2).cross(norm_v));
            let d = e1.offset_r.max(e2.offset_l) / cosf(ang / 2.0);
            return madd(v, perp, d);
        }
        if (ang - std::f32::consts::PI).abs() < EPSILON_ANG {
            // back along itself: on the line, the offset away
            let d = e1.offset_r.max(e2.offset_l);
            let (dir, _) = normalize_v3(self.co(e2.to) - v);
            return madd(v, dir, d);
        }
        let (norm_v1, norm_v2);
        if let (Some(f), true) = (face, ang < SMALL_ANG) {
            norm_v1 = self.normals[f as usize];
            norm_v2 = norm_v1;
        } else if !between {
            let (mut nv, _) = normalize_v3(dir2.cross(dir1));
            if dot_v3(nv, face_no(face)) < 0.0 {
                nv = -nv;
            }
            norm_v1 = nv;
            norm_v2 = nv;
        } else {
            // other faces between: each edge's own face at the corner
            let e1next = &corner.edges[(i1 + 1) % n];
            let e2prev = &corner.edges[(i2 + n - 1) % n];
            let dir1n = self.co(e1next.to) - v;
            let dir2p = v - self.co(e2prev.to);
            let (mut n1, _) = normalize_v3(dir1n.cross(dir1));
            if dot_v3(n1, self.normals[e1.fnext as usize]) < 0.0 {
                n1 = -n1;
            }
            let (mut n2, _) = normalize_v3(dir2.cross(dir2p));
            if dot_v3(n2, self.normals[e2prev.fnext as usize]) < 0.0 {
                n2 = -n2;
            }
            norm_v1 = n1;
            norm_v2 = n2;
        }
        // square to each edge in its face, pointing into the face
        let (perp1, _) = normalize_v3(dir1.cross(norm_v1));
        let (perp2, _) = normalize_v3(dir2.cross(norm_v2));
        let off1a = madd(v, perp1, e1.offset_r);
        let off1b = off1a + dir1;
        let off2a = madd(v, perp2, e2.offset_l);
        let off2b = off2a + dir2;
        match isect_line_line(off1a, off1b, off2a, off2b) {
            (0, _, _) => off1a,
            // (lines that pass each other, with other faces between: half way between their nearest points)
            (2, a, b) if between => mid(a, b),
            (_, a, _) => a,
        }
    }

    /// Where the offset line of the bevelled edge `bev` meets the edge `mid` left as it is (`first`: the bevelled
    /// one comes before it round the vertex), if it does, and the angle between the two.
    fn offset_meet_edge(
        &self,
        corner: &Corner,
        bev: usize,
        mid: usize,
        first: bool,
    ) -> Option<(V3, f32)> {
        let v = self.co(corner.v);
        let (e1, e2) = if first { (bev, mid) } else { (mid, bev) };
        let (dir1, _) = normalize_v3(self.co(corner.edges[e1].to) - v);
        let (dir2, _) = normalize_v3(self.co(corner.edges[e2].to) - v);
        let ang = angle_normalized(dir1, dir2);
        if ang.abs() < GOOD_ANGLE {
            return None;
        }
        let fno = dir1.cross(dir2);
        if dot_v3(fno, self.vertex_normals[corner.v as usize]) < 0.0 {
            return None; // a reflex angle
        }
        if (ang - std::f32::consts::PI).abs() < GOOD_ANGLE {
            return None;
        }
        let sinang = sinf(ang);
        Some(if first {
            (madd(v, dir2, corner.edges[bev].offset_r / sinang), ang)
        } else {
            (madd(v, dir1, corner.edges[bev].offset_l / sinang), ang)
        })
    }

    /// The new points round a vertex, one between each bevelled edge and the next: made, or (`again`) moved to
    /// where the edges' offsets now put them.
    pub(crate) fn build_boundary(&self, corner: &mut Corner, again: bool) {
        let n = corner.edges.len();
        if !again {
            corner.bounds.clear();
        }
        let mut e = 0;
        let mut at = 0;
        loop {
            // the edges left as they are up to the next bevelled one: between two faces of one plane, or not
            // (the last of each kind stands for it)
            let mut e2 = (e + 1) % n;
            let (mut in_plane, mut not_in_plane, mut eip, mut enip) = (0, 0, None, None);
            while !corner.edges[e2].bev {
                let before = self.normals[corner.edges[(e2 + n - 1) % n].fnext as usize];
                let after = self.normals[corner.edges[e2].fnext as usize];
                let dot = dot_v3(before, after);
                if (dot + 1.0).abs() <= EPSILON_BIG || (dot - 1.0).abs() <= EPSILON_BIG {
                    in_plane += 1;
                    eip = Some(e2);
                } else {
                    not_in_plane += 1;
                    enip = Some(e2);
                }
                e2 = (e2 + 1) % n;
            }
            let face = corner.edges[e].fnext;
            // a single edge between them takes the point, slid along it, if both offset lines reach it
            let on_edge = |mid_edge: Option<usize>| {
                let m = mid_edge?;
                let (a, ang1) = self.offset_meet_edge(corner, e, m, true)?;
                let (b, ang2) = self.offset_meet_edge(corner, e2, m, false)?;
                let ratio = if ang1 == 0.0 {
                    1.0
                } else {
                    sinf(ang2) / sinf(ang1)
                };
                Some((mid(a, b), m, ratio))
            };
            let slid = if in_plane == 0 && not_in_plane == 0 {
                None
            } else if not_in_plane > 0 {
                on_edge(enip.filter(|_| not_in_plane == 1))
            } else {
                on_edge(eip.filter(|_| in_plane == 1))
            };
            let co = match slid {
                Some((co, _, _)) => co,
                None if in_plane == 0 && not_in_plane == 0 => {
                    self.offset_meet(corner, e, e2, Some(face), false)
                }
                None if not_in_plane > 0 => self.offset_meet(corner, e, e2, None, true),
                None => self.offset_meet(corner, e, e2, Some(face), true),
            };
            if again {
                corner.bounds[at].co = co;
            } else {
                corner.bounds.push(Bound {
                    co,
                    efirst: e,
                    ebev: e2,
                    eon: slid.map(|s| s.1),
                    sinratio: slid.map(|s| s.2).unwrap_or(1.0),
                    face,
                    visited: false,
                    chain: None,
                });
                corner.edges[e].right = at;
                corner.edges[e2].left = at;
                let mut between = (e + 1) % n;
                while between != e2 {
                    (corner.edges[between].left, corner.edges[between].right) = (at, at);
                    between = (between + 1) % n;
                }
            }
            at += 1;
            e = e2;
            if e == 0 {
                break;
            }
        }
    }
}

impl Work<'_> {
    /// The profile from a vertex's new point `i` to the next, across the bevelled edge between them: its points
    /// between the two, on `plane` (a normal) if given.
    pub(crate) fn profile_of(
        &self,
        corner: &Corner,
        i: usize,
        plane: Option<V3>,
        ns: usize,
    ) -> Vec<V3> {
        let n = corner.bounds.len();
        let (start, end) = (corner.bounds[i].co, corner.bounds[(i + 1) % n].co);
        let to = corner.edges[corner.bounds[i].ebev].to;
        let (vco, other) = (self.co(corner.v), self.co(to));
        // the point of the edge nearest the line between the two
        let (lo, hi) = if corner.v < to {
            (vco, other)
        } else {
            (other, vco)
        };
        let middle = match isect_line_line(lo, hi, start, end) {
            (0, _, _) => lo,
            (_, on, _) => on,
        };
        let (along, _) = normalize_v3(vco - other);
        let plane_no = plane.unwrap_or_else(|| {
            let (d1, _) = normalize_v3(middle - start);
            let (d2, _) = normalize_v3(middle - end);
            normalize_v3(d1.cross(d2)).0
        });
        profile(start, middle, end, along, plane_no, ns)
    }

    /// Whether a corner of three bevelled edges is near enough a cube's to take a cube corner's patch.
    fn cube_like(&self, corner: &Corner) -> bool {
        if corner.bounds.len() != 3 {
            return false;
        }
        let v = corner.v;
        let mut total = 0.0_f32;
        let mut in_plane = 0;
        for h in &corner.edges {
            let (&(f, i), &(g, _)) = (&self.leaving[&(v, h.to)], &self.leaving[&(h.to, v)]);
            let (n1, n2) = (self.normals[f as usize], self.normals[g as usize]);
            let mut ang = angle_normalized(n1, n2);
            if n1 != n2 {
                let c = self.mesh.corners(f as usize);
                let dir = self.co(c[(i as usize + 1) % c.len()]) - self.co(c[i as usize]);
                if dot_v3(dir, n1.cross(n2)) <= 0.0 {
                    ang = -ang;
                }
            }
            let abs = ang.abs();
            // (against a quarter of pi in double precision, as Blender has it: just under the single)
            if abs as f64 <= std::f64::consts::FRAC_PI_4 {
                in_plane += 1;
            } else if abs >= 3.0 * std::f32::consts::FRAC_PI_4 {
                return false;
            }
            // (a cube corner's patch is for edges all cut back alike)
            if h.bev && (h.offset_l - corner.edges[0].offset_l).abs() > EPSILON {
                return false;
            }
            total += ang;
        }
        if in_plane != corner.edges.len() - 3 {
            return false;
        }
        // (with other edges lying in its faces too, a slab's corner where its face's triangles meet: Blender
        // tells such a corner apart, and gives it the cube corner's patch all the same)
        let off = (total.abs() - 3.0 * std::f32::consts::FRAC_PI_2).abs();
        off <= std::f32::consts::FRAC_PI_4
    }

    /// The points of the patch where three or more bevelled edges meet, by (bound, ring, step).
    pub(crate) fn patch(
        &self,
        corner: &Corner,
        ns: usize,
    ) -> Result<BTreeMap<(usize, usize, usize), V3>, String> {
        let n = corner.bounds.len();
        let v = self.co(corner.v);
        let mut out = BTreeMap::new();
        if self.cube_like(corner) {
            // a cube corner's patch, its three axes laid onto the three new points and the vertex
            let (va, vb, vc) = (
                corner.bounds[0].co,
                corner.bounds[1].co,
                corner.bounds[2].co,
            );
            let half = |p: V3, q: V3, r: V3| mul_v3_fl(p - q - r + v, 0.5);
            let (m0, m1, m2) = (half(va, vb, vc), half(vb, va, vc), half(vc, va, vb));
            let m3 = mul_v3_fl(va + vb + vc - v, 0.5);
            for (key, p) in cube_corner(ns)? {
                out.insert(
                    key,
                    V3::new(
                        p.x * m0.x + p.y * m1.x + m2.x * p.z + m3.x,
                        p.x * m0.y + p.y * m1.y + m2.y * p.z + m3.y,
                        p.x * m0.z + p.y * m1.z + m2.z * p.z + m3.z,
                    ),
                );
            }
            return Ok(out);
        }
        // any other corner: a patch of two segments (each side's middle on its profile, one point in the middle,
        // towards the vertex by the profile's fullness), subdivided, and read off at the segments asked for
        if ns > 4 {
            return Err(format!(
                "a patch of {ns} segments where {n} bevelled edges meet isn't built"
            ));
        }
        let mut vm = Grid::new(n, 2);
        let mut middle = V3::ZERO;
        for i in 0..n {
            *vm.raw(i, 0, 0) = corner.bounds[i].co;
            *vm.raw(i, 0, 1) = self.profile_of(corner, i, None, 2)[0];
            middle += corner.bounds[i].co;
        }
        let middle = mul_v3_fl(middle, 1.0 / n as f32);
        let towards = v - middle;
        *vm.raw(0, 1, 1) = if dot_v3(towards, towards) > 1e-12 {
            madd(middle, towards, FULLNESS[ns - 1])
        } else {
            middle
        };
        vm.copy_equiv();
        loop {
            vm = self.subdivided(corner, vm);
            if vm.ns >= ns {
                break;
            }
        }
        if vm.ns != ns {
            vm = self.read_off(corner, &vm, ns);
        }
        let ns2 = ns / 2;
        for i in 0..n {
            for j in 0..=ns2 {
                for k in 0..=ns {
                    if canon(n, ns, i, j, k) == (i, j, k) {
                        out.insert((i, j, k), *vm.raw(i, j, k));
                    }
                }
            }
        }
        Ok(out)
    }

    /// One step of Blender's cubic subdivision of a patch (Catmull-Clark, its sides kept on their profiles).
    fn subdivided(&self, corner: &Corner, mut vm_in: Grid) -> Grid {
        let (n, ns_in) = (vm_in.n, vm_in.ns);
        let (ns_in2, ns_out) = (ns_in / 2, 2 * ns_in);
        let mut out = Grid::new(n, ns_out);
        let avg4 = |a: V3, b: V3, c: V3, d: V3| mul_v3_fl(a + b + c + d, 0.25);
        let smooth = |co: V3, co1: V3, co2: V3| {
            let acc = madd(co1 + co2, co, -2.0);
            madd(co, acc, -1.0 / 6.0)
        };
        // the sides: the points there, smoothed
        for i in 0..n {
            *out.raw(i, 0, 0) = *vm_in.raw(i, 0, 0);
            for k in 1..ns_in {
                let co = smooth(
                    *vm_in.raw(i, 0, k),
                    *vm_in.raw(i, 0, k - 1),
                    *vm_in.raw(i, 0, k + 1),
                );
                out.set(i, 0, 2 * k, co);
            }
        }
        // and the new ones between them, from the profile
        let profiles: Vec<Vec<V3>> = (0..n)
            .map(|i| self.profile_of(corner, i, None, ns_out))
            .collect();
        for (i, profile) in profiles.iter().enumerate() {
            for k in (1..ns_out).step_by(2) {
                let co = smooth(profile[k - 1], out.get(i, 0, k - 1), out.get(i, 0, k + 1));
                out.set(i, 0, k, co);
            }
        }
        out.copy_equiv();
        for i in 0..n {
            for k in 0..ns_in {
                *vm_in.raw(i, 0, k) = *out.raw(i, 0, 2 * k);
            }
        }
        vm_in.copy_equiv();
        // the new points in the faces
        for i in 0..n {
            for j in 0..ns_in2 {
                for k in 0..ns_in2 {
                    let co = avg4(
                        *vm_in.raw(i, j, k),
                        *vm_in.raw(i, j, k + 1),
                        *vm_in.raw(i, j + 1, k),
                        *vm_in.raw(i, j + 1, k + 1),
                    );
                    *out.raw(i, 2 * j + 1, 2 * k + 1) = co;
                }
            }
        }
        // on the edges across the rings
        for i in 0..n {
            for j in 0..ns_in2 {
                for k in 1..=ns_in2 {
                    let co = avg4(
                        *vm_in.raw(i, j, k),
                        *vm_in.raw(i, j + 1, k),
                        out.get(i, 2 * j + 1, 2 * k - 1),
                        out.get(i, 2 * j + 1, 2 * k + 1),
                    );
                    *out.raw(i, 2 * j + 1, 2 * k) = co;
                }
            }
        }
        // on the edges along them
        for i in 0..n {
            for j in 1..ns_in2 {
                for k in 0..ns_in2 {
                    let co = avg4(
                        *vm_in.raw(i, j, k),
                        *vm_in.raw(i, j, k + 1),
                        out.get(i, 2 * j - 1, 2 * k + 1),
                        out.get(i, 2 * j + 1, 2 * k + 1),
                    );
                    *out.raw(i, 2 * j, 2 * k + 1) = co;
                }
            }
        }
        // the old points inside
        let (gamma, beta) = (0.25_f32, -0.25_f32);
        for i in 0..n {
            for j in 1..ns_in2 {
                for k in 1..=ns_in2 {
                    let co1 = avg4(
                        out.get(i, 2 * j, 2 * k - 1),
                        out.get(i, 2 * j, 2 * k + 1),
                        out.get(i, 2 * j - 1, 2 * k),
                        out.get(i, 2 * j + 1, 2 * k),
                    );
                    let co2 = avg4(
                        out.get(i, 2 * j - 1, 2 * k - 1),
                        out.get(i, 2 * j + 1, 2 * k - 1),
                        out.get(i, 2 * j - 1, 2 * k + 1),
                        out.get(i, 2 * j + 1, 2 * k + 1),
                    );
                    let co = madd(madd(co1, co2, beta), *vm_in.raw(i, j, k), gamma);
                    *out.raw(i, 2 * j, 2 * k) = co;
                }
            }
        }
        out.copy_equiv();
        // the middle
        let gamma = sabin_gamma(n);
        let beta = -gamma;
        let (mut co1, mut co2) = (V3::ZERO, V3::ZERO);
        for i in 0..n {
            co1 += *out.raw(i, ns_in, ns_in - 1);
            co2 += *out.raw(i, ns_in - 1, ns_in - 1);
            co2 += *out.raw(i, ns_in - 1, ns_in + 1);
        }
        let mut co = mul_v3_fl(co1, 1.0 / n as f32);
        co = madd(co, co2, beta / (2.0 * n as f32));
        co = madd(co, *vm_in.raw(0, ns_in2, ns_in2), gamma);
        for i in 0..n {
            *out.raw(i, ns_in, ns_in) = co;
        }
        // the sides back on their profiles
        for (i, profile) in profiles.iter().enumerate() {
            let inext = (i + 1) % n;
            for k in 1..ns_out {
                let co = profile[k - 1];
                *out.raw(i, 0, k) = co;
                if k >= ns_in {
                    *out.raw(inext, ns_out - k, 0) = co;
                }
            }
            *out.raw(i, 0, ns_out) = *out.raw(inext, 0, 0);
        }
        out
    }

    /// A patch of `nseg` segments read off one of more: each point where its share of the way along the sides
    /// falls among the finer patch's.
    fn read_off(&self, corner: &Corner, vm_in: &Grid, nseg: usize) -> Grid {
        let (n, ns_in) = (vm_in.n, vm_in.ns);
        let (nseg2, odd) = (nseg / 2, nseg % 2);
        let mut out = Grid::new(n, nseg);
        // how far along a side each of its points is, 0 to 1
        let shares = |points: &[V3]| {
            let mut frac = vec![0.0_f32; points.len()];
            let mut total = 0.0_f32;
            for k in 1..points.len() {
                total += len_v3(points[k - 1] - points[k]);
                frac[k] = total;
            }
            let last = points.len() - 1;
            if total > 0.0 {
                frac[1..].iter_mut().for_each(|f| *f /= total);
            } else {
                frac[last] = 1.0;
            }
            frac
        };
        let side_in =
            |i: usize| shares(&(0..=ns_in).map(|k| vm_in.at(i, 0, k)).collect::<Vec<_>>());
        let side_new = |i: usize| {
            let mut points = vec![corner.bounds[i].co];
            points.extend(self.profile_of(corner, i, None, nseg));
            points.push(corner.bounds[(i + 1) % n].co);
            shares(&points)
        };
        // the finer point at or before a share, and the rest of the way to the next
        let range = |frac: &[f32], f: f32| -> (usize, f32) {
            let count = frac.len() - 1;
            for i in 0..count {
                if f <= frac[i + 1] {
                    let rest = f - frac[i];
                    let rest = if rest == 0.0 {
                        0.0
                    } else {
                        rest / (frac[i + 1] - frac[i])
                    };
                    if i == count - 1 && rest == 1.0 {
                        return (count, 0.0);
                    }
                    return (i, rest);
                }
            }
            (count, 0.0)
        };
        let mut prev_frac = side_in(n - 1);
        let mut prev_new_frac = side_new(n - 1);
        for i in 0..n {
            let frac = side_in(i);
            let new_frac = side_new(i);
            for j in 0..nseg2 + odd {
                for (k, &share) in new_frac.iter().enumerate().take(nseg2 + 1) {
                    let (k_in, restk) = range(&frac, share);
                    let (k_in_prev, restkprev) = range(&prev_frac, prev_new_frac[nseg - j]);
                    let mut j_in = ns_in - k_in_prev;
                    let mut restj = -restkprev;
                    if restj > -EPSILON {
                        restj = 0.0;
                    } else {
                        j_in -= 1;
                        restj += 1.0;
                    }
                    let co = if restj < EPSILON && restk < EPSILON {
                        vm_in.get(i, j_in, k_in)
                    } else {
                        let j0inc = if restj < EPSILON || j_in == ns_in {
                            0
                        } else {
                            1
                        };
                        let k0inc = if restk < EPSILON || k_in == ns_in {
                            0
                        } else {
                            1
                        };
                        let quad = [
                            vm_in.get(i, j_in, k_in),
                            vm_in.get(i, j_in, k_in + k0inc),
                            vm_in.get(i, j_in + j0inc, k_in + k0inc),
                            vm_in.get(i, j_in + j0inc, k_in),
                        ];
                        let (u, v) = (restj, restk);
                        let mut res = V3::ZERO;
                        res += mul_v3_fl(quad[0], (1.0 - u) * (1.0 - v));
                        res += mul_v3_fl(quad[1], (1.0 - u) * v);
                        res += mul_v3_fl(quad[2], u * v);
                        res += mul_v3_fl(quad[3], u * (1.0 - v));
                        res
                    };
                    *out.raw(i, j, k) = co;
                }
            }
            prev_frac = frac;
            prev_new_frac = new_frac;
        }
        if odd == 0 {
            *out.raw(0, nseg2, nseg2) = vm_in.at(0, ns_in / 2, ns_in / 2);
        }
        out.copy_equiv();
        out
    }
}

//! Hardening: shading a bevel's new edges to keep their crease.

use super::*;

/// How much of its old place a patch's middle keeps when it is subdivided, by its sides (Sabin's weights).
pub(super) fn sabin_gamma(n: usize) -> f32 {
    match n {
        0..=2 => 0.0,
        3 => 0.065_247_58,
        4 => 0.25,
        5 => 0.401_983_44,
        6 => 0.523_423_3,
        _ => {
            let k = libm::cos(PI / n as f64);
            let (k2, k4, k6) = (k * k, k * k * k * k, k * k * k * k * k * k);
            let y = libm::pow(
                3.0_f64.sqrt() * (64.0 * k6 - 144.0 * k4 + 135.0 * k2 - 27.0).sqrt() + 9.0 * k,
                1.0 / 3.0,
            );
            let x = 0.480_749_856_769_136 * y - (0.231_120_424_783_545 * (12.0 * k2 - 9.0)) / y;
            ((k * x + 2.0 * k2 - 1.0) / (x * x * (k * x + 1.0))) as f32
        }
    }
}

/// A patch's points as Blender keeps them: a block for each side by (ring, step), the points the blocks share
/// kept in one of them and copied to the others.
#[derive(Clone)]
pub(super) struct Grid {
    pub(super) n: usize,
    pub(super) ns: usize,
    pub(super) co: Vec<V3>,
}

impl Grid {
    pub(super) fn new(n: usize, ns: usize) -> Grid {
        Grid {
            n,
            ns,
            co: vec![V3::ZERO; n * (ns / 2 + 1) * (ns + 1)],
        }
    }
    fn index(&self, i: usize, j: usize, k: usize) -> usize {
        (i * (self.ns / 2 + 1) + j) * (self.ns + 1) + k
    }
    /// The slot itself.
    pub(super) fn raw(&mut self, i: usize, j: usize, k: usize) -> &mut V3 {
        let at = self.index(i, j, k);
        &mut self.co[at]
    }
    pub(super) fn at(&self, i: usize, j: usize, k: usize) -> V3 {
        self.co[self.index(i, j, k)]
    }
    /// The point that stands for the slot.
    pub(super) fn get(&self, i: usize, j: usize, k: usize) -> V3 {
        let (i, j, k) = canon(self.n, self.ns, i, j, k);
        self.at(i, j, k)
    }
    pub(super) fn set(&mut self, i: usize, j: usize, k: usize, co: V3) {
        let (i, j, k) = canon(self.n, self.ns, i, j, k);
        *self.raw(i, j, k) = co;
    }
    pub(super) fn copy_equiv(&mut self) {
        for i in 0..self.n {
            for j in 0..=self.ns / 2 {
                for k in 0..=self.ns {
                    let c = canon(self.n, self.ns, i, j, k);
                    if c != (i, j, k) {
                        let co = self.at(c.0, c.1, c.2);
                        *self.raw(i, j, k) = co;
                    }
                }
            }
        }
    }
}

/// Hardened normals: a face of the mesh keeps its own (smooth across the edges left smooth), a strip's corners
/// take the face they touch or the mean of the strips either side, a corner patch's the face, strips or
/// vertex they stand on.
pub(super) fn harden(
    out: &mut Mesh,
    kinds: &[Kind],
    long: &BTreeSet<(u32, u32)>,
    old: &Mesh,
    kept: &[u32],
) {
    let nf = out.faces.len();
    let flat = |k: Kind| matches!(k, Kind::Kept | Kind::Cut);
    let mut leaving: BTreeMap<(u32, u32), (u32, u32)> = BTreeMap::new();
    for f in 0..nf {
        let c = out.corners(f);
        for (i, &a) in c.iter().enumerate() {
            leaving.insert((a, c[(i + 1) % c.len()]), (f as u32, i as u32));
        }
    }
    // sharp: where a face of the mesh meets the bevel, and the mesh's own sharp edges that are left
    out.sharp = old
        .sharp
        .iter()
        .filter(|e| kept[e.0 as usize] != u32::MAX && kept[e.1 as usize] != u32::MAX)
        .map(|e| edge(kept[e.0 as usize], kept[e.1 as usize]))
        .collect();
    for (&(a, b), &(f, _)) in &leaving {
        if let Some(&(g, _)) = leaving.get(&(b, a))
            && flat(kinds[f as usize])
            && !flat(kinds[g as usize])
            && out.faces[f as usize].smooth
        {
            out.sharp.insert(edge(a, b));
        }
    }
    out.normals = None;
    let face_normals = face_normals(out);
    // a vertex's normal: its faces' weighed by their angles at it (by Blender's approximate arc cosine, as
    // every angle here is: the fans' weights, their frames and the angles stored in them)
    let mut vertex_normals = vec![V3::ZERO; out.verts.len()];
    for (f, &normal) in face_normals.iter().enumerate() {
        let c = out.corners(f);
        let n = c.len();
        for i in 0..n {
            let p = out.verts[c[i] as usize];
            let (e1, _) = normalize_v3(out.verts[c[(i + n - 1) % n] as usize] - p);
            let (e2, _) = normalize_v3(out.verts[c[(i + 1) % n] as usize] - p);
            let k = crate::normals::acos_approx(dot_v3(e1, e2));
            let s = &mut vertex_normals[c[i] as usize];
            *s = madd(*s, normal, k);
        }
    }
    // the corner of the face across the side leaving (or entering) corner (f, i), at the same vertex
    let across_leaving = |f: u32, i: u32| -> Option<(u32, u32)> {
        let c = out.corners(f as usize);
        let (a, b) = (c[i as usize], c[(i as usize + 1) % c.len()]);
        let &(g, j) = leaving.get(&(b, a))?;
        Some((g, (j + 1) % out.faces[g as usize].len))
    };
    let across_entering = |f: u32, i: u32| -> Option<(u32, u32)> {
        let c = out.corners(f as usize);
        let (p, a) = (c[(i as usize + c.len() - 1) % c.len()], c[i as usize]);
        leaving.get(&(a, p)).copied()
    };
    let kind = |l: Option<(u32, u32)>| l.map(|(f, _)| kinds[f as usize]);
    let no = |l: Option<(u32, u32)>| face_normals[l.expect("a face").0 as usize];
    let mean = |a: V3, b: V3| normalize_v3(a + b).0;
    // each bevel face's corners: the normal it is given, or none to give
    let mut given: Vec<Option<Option<V3>>> = vec![None; out.loops.len()];
    let prev_corner = |f: u32, i: u32| {
        let n = out.faces[f as usize].len;
        (f, (i + n - 1) % n)
    };
    for f in 0..nf as u32 {
        let k = kinds[f as usize];
        if flat(k) {
            continue;
        }
        let own = face_normals[f as usize];
        for i in 0..out.faces[f as usize].len {
            let lprev = across_leaving(f, i);
            let lnext = across_entering(f, i);
            let (kprev, knext) = (kind(lprev), kind(lnext));
            let is_flat = |k: Option<Kind>| k.is_some_and(flat);
            let pnorm = if k == Kind::Strip {
                if kprev == Some(Kind::Strip) && long.contains(&(f, i)) {
                    Some(mean(own, no(lprev)))
                } else if knext == Some(Kind::Strip) && lnext.is_some_and(|l| long.contains(&l)) {
                    Some(mean(own, no(lnext)))
                } else if is_flat(kprev) && long.contains(&(f, i)) {
                    Some(no(lprev))
                } else if is_flat(knext) && long.contains(&prev_corner(f, i)) {
                    Some(no(lnext))
                } else {
                    None
                }
            } else if kprev == Some(Kind::Corner) && knext == Some(Kind::Corner) {
                let v = out.corners(f as usize)[i as usize];
                Some(normalize_v3(vertex_normals[v as usize]).0)
            } else if is_flat(kprev) {
                Some(no(lprev))
            } else if is_flat(knext) {
                Some(no(lnext))
            } else {
                let lprevprev = lprev.and_then(|(g, j)| across_leaving(g, j));
                let lnextnext = lnext.and_then(|(g, j)| across_entering(g, j));
                let (kpp, knn) = (kind(lprevprev), kind(lnextnext));
                if kprev == Some(Kind::Strip) && is_flat(kpp) {
                    Some(no(lprevprev))
                } else if kprev == Some(Kind::Strip)
                    && knext == Some(Kind::Corner)
                    && kpp == Some(Kind::Strip)
                {
                    Some(mean(no(lprev), no(lprevprev)))
                } else if knext == Some(Kind::Strip)
                    && kprev == Some(Kind::Corner)
                    && knn == Some(Kind::Strip)
                {
                    Some(mean(no(lnext), no(lnextnext)))
                } else {
                    None
                }
            };
            // (a face of no area has no normal to give: the corner keeps its own)
            let l = (out.faces[f as usize].start + i) as usize;
            given[l] = Some(pnorm.filter(|p| dot_v3(*p, *p) > 0.0));
        }
    }
    // Blender's bevel stores each given normal in the frame of its corner's fan on the mesh it is working on (a
    // fan: the corners of one vertex joined across smooth edges; two angles, from the fan's normal and round it
    // from the edge the fan starts at, each as a share of the frame's own angle in 16 bits). The mesh it leaves
    // reads them back in its own fans' frames (`normals::from_stored`), which start elsewhere now and then, and
    // where a fan's corners were given unlike normals it reads the mean of what was stored: the normals are
    // Blender's, not always the ones given.
    let mut stored = vec![[0_i16; 2]; out.loops.len()];
    for fan in fans(out, &face_normals) {
        if !fan.corners.iter().any(|&l| given[l].flatten().is_some()) {
            continue;
        }
        let space = Space::new(&fan);
        for &l in &fan.corners {
            if let Some(p) = given[l].flatten() {
                let s = space.store(p);
                stored[l] = [s[0] as i16, s[1] as i16];
            }
        }
    }
    out.normals = Some(crate::normals::from_stored(out, &stored));
    out.stored = Some(stored.into_iter().map(Some).collect());
}

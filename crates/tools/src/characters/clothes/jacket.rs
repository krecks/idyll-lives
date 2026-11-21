//! Jackets: the shell over the torso, and the collar on its neckline.

use super::*;

/// A band turned back from each open edge towards the skin (the cloth's thickness); edges with
/// both ends in `skip` get none.
pub(super) fn rim(
    pos: &[V3],
    nor: &[V3],
    idx: &[usize],
    depth: f64,
    skip: Option<&[bool]>,
) -> (Vec<V3>, Vec<usize>, Vec<usize>) {
    let be: Vec<[usize; 2]> = boundary_edges(idx)
        .into_iter()
        .filter(|e| skip.is_none_or(|k| !(k[e[0]] && k[e[1]])))
        .collect();
    let verts = unique(&be.concat());
    let mut newi = vec![usize::MAX; pos.len()];
    for (i, &v) in verts.iter().enumerate() {
        newi[v] = pos.len() + i;
    }
    let mut p = pos.to_vec();
    p.extend(verts.iter().map(|&v| pos[v] - nor[v] * depth));
    let mut out = idx.to_vec();
    for [a, bb] in be {
        out.extend([bb, a, newi[a], bb, newi[a], newi[bb]]);
    }
    (p, out, verts)
}

/// A shell's positions, unit normals and skinning on `stage`'s body, from its source edges `src` on the base body.
pub(super) fn shell_on(
    family: &str,
    stage: &str,
    src: &Src,
    lod: bool,
) -> (Vec<V3>, Vec<V3>, Vec<Dense>) {
    let s = surface(family, stage);
    let (bpos, bidx, dense): (&[V3], Vec<usize>, &[Dense]) = if lod {
        (&s.lod_pos, topology().lod_tris.concat(), &s.lod_dense)
    } else {
        (&s.pos, tris(family).to_vec(), &s.dense)
    };
    let bnor = smooth_normals(bpos, &bidx);
    let pos = lerp_src(src, bpos);
    let nor = lerp_src(src, &bnor).iter().map(|n| n / n.norm()).collect();
    (pos, nor, shell_dense(src, dense))
}

pub(super) fn jacket(family: &str, stage: &str, gid: &str, lod: bool) -> Mesh {
    let lay = jacket_layout(family, gid, lod);
    let (pos, nor, mut d) = shell_on(family, stage, &lay.src, lod);
    let cut = &lay.cut;
    let h = proportions(family, stage).h;
    let k = h / 1.7;
    // Lift: thicker over the hips (the flare of the hem) and a little looser on the sleeves.
    let hem = cut.hem.unwrap();
    let cuts = garment_cuts(family, &surface(family, BASE_STAGE).p);
    let skip: Vec<bool> = lay
        .sleeve
        .iter()
        .map(|&v| v > SHELL_SLEEVE - 0.06)
        .collect();
    // `flared`: as it hangs standing; else as it lies seated (no flare, over the skirt as that lies).
    let shape = |flared: bool| {
        let mut pos: Vec<V3> = (0..pos.len())
            .map(|i| {
                let flare = if flared {
                    smoothstep(hem + 0.14, hem, lay.hem_y[i])
                } else {
                    0.0
                };
                let taper = smoothstep(SHELL_SLEEVE, SHELL_SLEEVE - 0.24, lay.sleeve[i]);
                let lift = (jacket_lift(gid) + 0.012 * flare) * k * taper;
                pos[i] + nor[i] * lift
            })
            .collect();
        if cuts.contains("bottom.skirt") {
            // The hem lies over a skirt (it is a shell too).
            pos = over_skirt(&pos, family, stage, &cuts, 0.005 * k, !flared);
        }
        let n0 = pos.len();
        let (pos, idx, verts) = rim(
            &pos,
            &nor,
            &lay.idx,
            0.6 * jacket_lift(gid) * k,
            Some(&skip),
        );
        (pos, idx, verts, n0)
    };
    let (pos, idx, verts, n0) = shape(true);
    let nor_all = smooth_normals(&pos, &idx);
    let extra: Vec<Dense> = verts.iter().map(|&v| d[v]).collect();
    d.extend(extra);
    // The hem distance (m, on the base body, so every stage shares it): from each vertex to the nearest open
    // edge (hem, lapels, neck; not a sleeve's end, which sinks into the painted sleeve), for the shader's band.
    let hem_d = edge_distance(family, &lay.src, &lay.idx, lod, Some(&skip));
    let n = pos.len();
    let fields = (0..n)
        .map(|i| [if i < n0 { hem_d[i] } else { 0.0 }, 0.0, TOP, 0.0])
        .collect();
    let clear = if cuts.contains("bottom.skirt") {
        JACKET_CLEAR
    } else {
        SKIRT_CLEAR
    };
    let tone = seated_tone(family, stage, &pos, &d, &shape(false).0, clear * k);
    Mesh {
        pos,
        nor: nor_all,
        idx,
        dense: d,
        fields,
        tone,
        ..Mesh::default()
    }
}

/// A turned-down collar on the neckline (open at the front).
pub(super) fn collar(family: &str, stage: &str, gid: &str, cuts: &Cuts) -> Mesh {
    let s = surface(family, stage);
    let sb = surface(family, BASE_STAGE);
    let p = &s.p;
    let cut = cuts.get(gid).unwrap();
    let fl = base_fields(family);
    let f = &fl.0;
    // Points on the neckline of the base body (crew field = cut), carried to the stage by the
    // nearest base vertices.
    let zn = sb.p[b("neck_01")].z;
    let th: Vec<f64> = linspace(28.0, 332.0, 15)
        .iter()
        .map(|&d| radians(d))
        .collect();
    let (rh, re) = (region("head"), region("ear"));
    let crew = cut.crew.unwrap();
    let cand: Vec<usize> = (0..sb.pos.len())
        .filter(|&i| {
            (f[i][2] - crew).abs() < 0.012
                && sb.region[i][rh] + sb.region[i][re] < 0.5
                && sb.tags.arm[i] < 0.3
        })
        .collect();
    let ang: Vec<f64> = cand
        .iter()
        .map(|&i| py_mod(sb.pos[i].x.atan2(sb.pos[i].z - zn), 2.0 * PI))
        .collect();
    let picks: Vec<[usize; 3]> = th
        .iter()
        .map(|&t| {
            let da: Vec<f64> = ang
                .iter()
                .map(|&a| (py_mod(a - t + PI, 2.0 * PI) - PI).abs())
                .collect();
            let o = super::np::argsort(&da);
            [cand[o[0]], cand[o[1]], cand[o[2]]]
        })
        .collect();
    let base_pts: Vec<V3> = picks
        .iter()
        .map(|pk| (s.pos[pk[0]] + s.pos[pk[1]] + s.pos[pk[2]]) / 3.0)
        .collect();
    let sn = smooth_normals(&s.pos, &surface_tris(family));
    let nor: Vec<V3> = picks
        .iter()
        .map(|pk| (sn[pk[0]] + sn[pk[1]] + sn[pk[2]]) / 3.0)
        .map(|n| n / n.norm())
        .collect();
    let up = v3(0.0, 1.0, 0.0);
    let h = proportions(family, stage).h;
    let k = h / 1.7;
    let shirt = gid == "top.shirt";
    let stand = (if shirt { 0.022 } else { 0.018 }) * k;
    let fall = (if shirt { 0.034 } else { 0.028 }) * k;
    let (mut r0, mut r1, mut r2) = (Vec::new(), Vec::new(), Vec::new());
    for i in 0..th.len() {
        let front = clip(th[i].cos(), 0.0, 1.0);
        let a = base_pts[i] + nor[i] * 0.003 * k;
        let bb = a + up * stand + nor[i] * 0.004 * k;
        let c = bb + nor[i] * 0.012 * k - up * (fall * (1.0 + 0.5 * (front * front)))
            + nor[i] * (0.01 * front) * k;
        r0.push(a);
        r1.push(bb);
        r2.push(c);
    }
    let (pos, idx) = strip(&[r0, r1, r2], false);
    let idx = outward(&pos, idx, (0.0, p[b("neck_01")].z));
    let mut dense = follow(&pos, &s);
    // The collar's back is on the midline, where the nearest body vertices come in mirror pairs:
    // its weights are made exactly symmetric (each bone and its mirror get the mean), so rounding
    // noise can't favour a side. Of a clavicle pair tied for the last of the four joints, `top4`
    // then keeps one by its fixed order, as on the body's own midline.
    let mirror = super::rig::mirror_bones();
    for (q, d) in pos.iter().zip(dense.iter_mut()) {
        if q.x.abs() < 1e-9 {
            let m = *d;
            for (bi, v) in d.iter_mut().enumerate() {
                *v = (m[bi] + m[mirror[bi]]) / 2.0;
            }
        }
    }
    let nor = smooth_normals(&pos, &idx);
    let n = pos.len();
    let fields = vec![[0.05, 0.0, TOP, 0.0]; n];
    Mesh {
        pos,
        idx,
        nor,
        dense,
        fields,
        tone: vec![[0.0; 4]; n],
        ..Mesh::default()
    }
}

pub(super) fn surface_tris(family: &str) -> Arc<Vec<usize>> {
    tris(family)
}

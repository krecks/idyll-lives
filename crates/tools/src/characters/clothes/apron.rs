//! Aprons: the apron's layout and its shell.

use super::*;

/// How far (m) each of a shell's source vertices is from the shell's nearest open edge, on the base body's surface
/// (so every stage shares it), for the shader's hem band. Edges between two vertices `skip` marks are left out (a
/// jacket's sleeve ends, which sink into the painted sleeve).
pub(crate) fn edge_distance(
    family: &str,
    src: &Src,
    idx: &[usize],
    lod: bool,
    skip: Option<&[bool]>,
) -> Vec<f64> {
    let base = surface(family, BASE_STAGE);
    let bp = lerp_src(src, if lod { &base.lod_pos } else { &base.pos });
    let open: Vec<[usize; 2]> = boundary_edges(idx)
        .into_iter()
        .filter(|e| skip.is_none_or(|k| !(k[e[0]] && k[e[1]])))
        .collect();
    let to_edge = |q: &V3, [a, c]: [usize; 2]| {
        let e = bp[c] - bp[a];
        (q - (bp[a] + e * clip((q - bp[a]).dot(&e) / e.norm_squared().max(1e-12), 0.0, 1.0))).norm()
    };
    bp.iter()
        .map(|q| open.iter().map(|&e| to_edge(q, e)).fold(0.05, f64::min))
        .collect()
}

/// The apron's piece of the base body (the front of the torso and thighs, from the waistband to above the knees),
/// clipped faces and their source edges: stage independent.
pub(crate) struct ApronLayout {
    pub(crate) src: Src,
    pub(crate) idx: Vec<usize>,
}

pub(crate) static APRON_LAYOUTS: Cache<(String, bool), ApronLayout> = Cache::new();
/// How far the apron reaches round the body from the front (radians either side).
pub(crate) const APRON_SPAN: f64 = 1.28;
/// Where the apron sits: its waistband above the trousers' (m, on the base body, over the pelvis) and its hem
/// above the knees.
pub(crate) const APRON_WAIST: f64 = 0.085;
pub(crate) const APRON_HEM: f64 = 0.03;

fn apron_layout(family: &str, lod: bool) -> Arc<ApronLayout> {
    APRON_LAYOUTS.get((family.to_string(), lod), || {
        let s = surface(family, BASE_STAGE);
        let (pos, idx): (&[V3], Vec<usize>) = if lod {
            (&s.lod_pos, topology().lod_tris.concat())
        } else {
            (&s.pos, tris(family).to_vec())
        };
        let (tags, regs) = if lod {
            (&s.lod_tags, &s.lod_region)
        } else {
            (&s.tags, &s.region)
        };
        let p = &s.p;
        let cz = p[b("pelvis")].z;
        let (rt, rl) = (region("torso"), region("leg"));
        let lm = Landmarks::new(p, family);
        let (waist, hem) = (lm.waist + APRON_WAIST, lm.knee + APRON_HEM);
        // Inside where the torso's or the legs' front is between the waistband and the hem: the height to the
        // nearer edge, and the arc to the panel's side, as metres.
        let g: Vec<f64> = pos
            .iter()
            .zip(tags.arm.iter().zip(regs.iter()))
            .map(|(q, (arm, rr))| {
                if *arm >= 0.1 || rr[rt] + rr[rl] <= 0.5 {
                    return -1.0;
                }
                let side = (APRON_SPAN - q.x.atan2(q.z - cz).abs()) * 0.12;
                (q.y - hem).min(waist - q.y).min(side)
            })
            .collect();
        let (cidx, src) = clip_by_field(&idx, &g);
        ApronLayout { src, idx: cidx }
    })
}

/// The apron's shell: the body's own front surface from the waistband to above the knees, lifted off the trousers,
/// its edges turned back to the cloth's thickness. It is painted in the top's colour, with the shirt painted on the
/// body under it in white (the runtime's `SHIRTS.apron`); the waist tie is the top edge's band.
pub(crate) fn apron(family: &str, stage: &str, lod: bool) -> Mesh {
    let lay = apron_layout(family, lod);
    let (pos, nor, mut d) = shell_on(family, stage, &lay.src, lod);
    let k = proportions(family, stage).h / 1.7;
    // A hand's breadth off the trousers, and it hangs straight (no flare).
    let lift = 0.008 * k;
    let hang: Vec<V3> = pos.iter().zip(&nor).map(|(q, n)| *q + *n * lift).collect();
    let n0 = hang.len();
    // Every edge is open (the hem, the sides, the waistband): all of them turn back to the cloth's thickness.
    let (pos, idx, verts) = rim(&hang, &nor, &lay.idx, 0.5 * lift, None);
    let nor_all = smooth_normals(&pos, &idx);
    let extra: Vec<Dense> = verts.iter().map(|&v| d[v]).collect();
    d.extend(extra);
    // The distance to the nearest open edge (m, on the base body, so every stage shares it) for the shader's band.
    let edge_d = edge_distance(family, &lay.src, &lay.idx, lod, None);
    let n = pos.len();
    // In the top's colour (the shirt under it is white: the runtime's `SHIRTS.apron`), with a hem band (fields y =
    // 3: the skirt's flag for one).
    let fields = (0..n)
        .map(|i| [if i < n0 { edge_d[i] } else { 0.0 }, 3.0, TOP, 0.0])
        .collect();
    // Seated, it lies on the lap: no lower than the seat and clear of the skin (as the skirt does).
    let tone = seated_tone(family, stage, &pos, &d, &hang, SKIRT_CLEAR * k);
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

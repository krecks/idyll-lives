//! Finishing: clearing the ears, the wisps and the final mesh.

use super::*;

/// A point within EAR_CLEAR of an ear pushed off it.
fn off_ears(q: &mut V3, ears: &Ears) {
    if let Some((d, out)) = ears.depth(*q)
        && d > -EAR_CLEAR
    {
        *q += out * (EAR_CLEAR + 0.0002 + d);
    }
}

/// Points pushed clear of the ears and the head.
fn clear(pos: &mut [V3], h: &HeadFrame, ears: &Ears) {
    // The shapes go round or over the ears; whatever still comes within EAR_CLEAR of one is pushed off it.
    for q in pos.iter_mut() {
        off_ears(q, ears);
    }
    // Nothing closer than CLEAR to the head: push such vertices out along the field's gradient
    // (measured a little round each point, so the clearance holds after the positions are rounded
    // in the files, wherever the field is rough). Only the vertices pushed can still be near.
    let mut near: Vec<usize> = (0..pos.len()).collect();
    for _ in 0..24 {
        let d: Vec<(usize, f64)> = near
            .iter()
            .map(|&i| (i, near_sdf(h, pos[i])))
            .filter(|&(_, d)| d < CLEAR)
            .collect();
        if d.is_empty() {
            break;
        }
        for &(i, di) in &d {
            let g = grad(h, pos[i], 1e-3);
            let o = pos[i] - h.centre;
            let o = o / o.norm();
            let g = if g.dot(&o) > 0.2 { g } else { o };
            pos[i] += g * clip(CLEAR + 0.0004 - di, 0.0003, 0.01);
        }
        near = d.iter().map(|e| e.0).collect();
    }
}

/// Where a style's strips start along the edge (fields.x, m), strip by strip: apart, so no two of
/// them draw the same strands.
const STRIP_STARTS: f64 = 2.0;

/// A style's wisps (its components' `Strip`s) as a part, from its solid hair as finished (`solid`:
/// the components' vertices in their order): per edge vertex one at the root, the solid hair's own
/// vertex again, and one where its wisp ends (kept clear of the ears and the head as the hair is),
/// in that order, both with its normal, skin and tone (the occlusion follows in build.rs). Fields: x =
/// metres along the edge, y = 0 at the root and 1 at the end, z = how loose the strands are, w = how
/// far along they stay one.
pub(super) fn wisps(comps: &[Comp], solid: &Mesh, h: &HeadFrame, ears: &Ears) -> Mesh {
    let mut m = Mesh::default();
    let mut first = 0;
    let mut strips = 0;
    for c in comps {
        for s in &c.wisps {
            let base = m.pos.len();
            let n = s.at.len() + usize::from(s.closed);
            let mut u = STRIP_STARTS * jitter(strips, 7.0);
            strips += 1;
            for k in 0..n {
                let j = k % s.at.len();
                let i = first + s.at[j];
                if k > 0 {
                    u += (solid.pos[i] - solid.pos[first + s.at[k - 1]]).norm();
                }
                let mut end = solid.pos[i] + s.by[j];
                off_ears(&mut end, ears);
                // (Off the head by its field's own way out, whatever `clear` would take: wisps reach down
                // the neck, where the way out is not from the head's centre.)
                for _ in 0..8 {
                    let d = near_sdf(h, end);
                    if d >= CLEAR {
                        break;
                    }
                    end += grad(h, end, 1e-3) * (CLEAR + 0.0004 - d);
                }
                for (v, p) in [(0.0, solid.pos[i]), (1.0, end)] {
                    m.pos.push(p);
                    m.nor.push(solid.nor[i]);
                    m.dense.push(solid.dense[i]);
                    m.fields.push([u, v, s.loose, s.rooted[j]]);
                    m.tone.push(solid.tone[i]);
                }
                if k > 0 {
                    let (a, b) = (base + 2 * (k - 1), base + 2 * k);
                    m.idx.extend([a, b, b + 1, a, b + 1, a + 1]);
                }
            }
        }
        first += c.pos.len();
    }
    m
}

/// The components merged, pushed clear of the head, skinned, with fields and tone.
pub(super) fn finish(
    comps: &[Comp],
    h: &HeadFrame,
    ears: &Ears,
    p: &[V3; NB],
    head_only: bool,
) -> Mesh {
    let mut pos = Vec::new();
    let (mut idx, mut zz, mut yy, mut ff) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut sway = Vec::new();
    for c in comps {
        let n = pos.len();
        idx.extend(c.idx.iter().map(|&i| i + n));
        pos.extend(&c.pos);
        zz.extend(&c.z);
        yy.extend(&c.y);
        ff.extend(&c.f);
        sway.extend(std::iter::repeat_n(c.sway, c.pos.len()));
    }
    clear(&mut pos, h, ears);
    let n = pos.len();
    let nor = smooth_normals(&pos, &idx);
    let mut dense = vec![[0.0; NB]; n];
    let (hd, nk, s3) = (b("Head"), b("neck_01"), b("spine_03"));
    for i in 0..n {
        if head_only {
            dense[i][hd] = 1.0;
            continue;
        }
        let neck_y = p[nk].y;
        let (mut a, mut bb) = (
            smoothstep(h.chin_y + 0.05 * h.hh, neck_y, pos[i].y),
            smoothstep(neck_y, p[s3].y, pos[i].y),
        );
        if ff[i] >= 0.0 {
            a = clip(ff[i] * 1.6, 0.0, 1.0);
            bb = smoothstep(0.4, 1.0, ff[i]);
        }
        dense[i][hd] = 1.0 - a;
        dense[i][nk] = a * (1.0 - bb);
        dense[i][s3] = a * bb;
    }
    let fields = vec![[0.1, 0.0, 0.0, 0.0]; n];
    // How loose each vertex hangs (tone.x).
    let from = h.chin_y + SWAY_FROM * h.hh;
    let tone = (0..n)
        .map(|i| {
            let down = if ff[i] >= 0.0 {
                clip(ff[i], 0.0, 1.0).powf(SWAY_TAIL)
            } else {
                clip((from - pos[i].y) / (SWAY_OVER * h.hh), 0.0, 1.0).powf(SWAY_FALL)
            };
            [
                sway[i] * down,
                clip(yy[i], 0.0, 1.0),
                clip(zz[i], 0.0, 1.0),
                0.0,
            ]
        })
        .collect();
    Mesh {
        pos,
        idx,
        nor,
        dense,
        fields,
        tone,
        ..Mesh::default()
    }
}

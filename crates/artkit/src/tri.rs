//! Triangles of a polygon of five corners or more (a cylinder's cap, a swept profile's end): ears clipped in the
//! order Blender clips them, every other corner round the polygon, so a round cap comes out as Blender's does.

use crate::math::V3;

/// The triangles of the polygon `p` (its `normal` given), as places among its corners.
pub fn polygon(p: &[V3], normal: V3) -> Vec<[u32; 3]> {
    let n = p.len();
    // the polygon flat, seen from the side its normal points to: anticlockwise
    let nrm = [normal.x as f64, normal.y as f64, normal.z as f64];
    let a = if nrm[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let cross = |u: [f64; 3], v: [f64; 3]| {
        [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ]
    };
    let dot = |u: [f64; 3], v: [f64; 3]| u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
    let x = cross(a, nrm);
    let y = cross(nrm, x);
    let flat: Vec<[f64; 2]> = p
        .iter()
        .map(|v| {
            let q = [v.x as f64, v.y as f64, v.z as f64];
            [dot(q, x), dot(q, y)]
        })
        .collect();
    let turn = |i: usize, j: usize, k: usize| {
        (flat[j][0] - flat[i][0]) * (flat[k][1] - flat[i][1])
            - (flat[j][1] - flat[i][1]) * (flat[k][0] - flat[i][0])
    };
    let mut next: Vec<usize> = (0..n).map(|i| (i + 1) % n).collect();
    let mut prev: Vec<usize> = (0..n).map(|i| (i + n - 1) % n).collect();
    let mut left = n;
    let mut out = Vec::with_capacity(n - 2);
    let mut head = 0usize;
    let mut init = 0usize;
    let mut reverse = false;
    while left > 3 {
        // the first corner from `init` that is an ear: it turns the polygon's way and holds no other corner
        let is_ear = |i: usize, next: &[usize], prev: &[usize]| {
            let (a, b) = (prev[i], next[i]);
            if turn(a, i, b) <= 0.0 {
                return false;
            }
            let mut k = next[b];
            while k != a {
                if turn(a, i, k) >= 0.0
                    && turn(i, b, k) >= 0.0
                    && turn(b, a, k) >= 0.0
                    && turn(prev[k], k, next[k]) <= 0.0
                {
                    return false;
                }
                k = next[k];
            }
            true
        };
        let mut ear = init;
        let mut tried = 0;
        while !is_ear(ear, &next, &prev) && tried < left {
            ear = if reverse { prev[ear] } else { next[ear] };
            tried += 1;
        }
        let (a, b) = (prev[ear], next[ear]);
        out.push([a as u32, ear as u32, b as u32]);
        next[a] = b;
        prev[b] = a;
        left -= 1;
        if head == ear {
            head = b;
        }
        init = if reverse { prev[a] } else { next[b] };
        if turn(prev[init], init, next[init]) <= 0.0 {
            init = if reverse { prev[init] } else { next[init] };
            reverse = !reverse;
        }
    }
    out.push([head as u32, next[head] as u32, next[next[head]] as u32]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::v3;

    #[test]
    fn a_hexagon_is_clipped_at_every_other_corner() {
        let p: Vec<V3> = (0..6)
            .map(|i| {
                let a = std::f64::consts::TAU * i as f64 / 6.0;
                v3(a.cos(), a.sin(), 0.0)
            })
            .collect();
        assert_eq!(
            polygon(&p, V3::Z),
            [[5, 0, 1], [1, 2, 3], [3, 4, 5], [1, 3, 5]]
        );
    }

    #[test]
    fn a_dented_polygon_is_filled_inside_its_outline() {
        // an arrow head: the dent at corner 3
        let p = [
            v3(0, 0, 0),
            v3(2, 0, 0),
            v3(2, 2, 0),
            v3(1, 0.5, 0),
            v3(0, 2, 0),
        ];
        let tris = polygon(&p, V3::Z);
        assert_eq!(tris.len(), 3);
        let area: f64 = tris
            .iter()
            .map(|t| {
                let (a, b, c) = (p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]);
                ((b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)) as f64 / 2.0
            })
            .sum();
        assert!(tris.iter().all(|t| {
            let (a, b, c) = (p[t[0] as usize], p[t[1] as usize], p[t[2] as usize]);
            (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x) > 0.0
        }));
        assert!((area - 2.5).abs() < 1e-6, "{area}");
    }
}

pub type P3 = [f64; 3];

pub(super) fn sub(a: P3, b: P3) -> P3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub(super) fn dot(a: P3, b: P3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub(super) fn cross(a: P3, b: P3) -> P3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub(super) fn length(a: P3) -> f64 {
    dot(a, a).sqrt()
}

/// `a` at length 1, or zero for one that has none.
pub(super) fn unit(a: P3) -> P3 {
    let l = length(a);
    if l > 0.0 { a.map(|v| v / l) } else { [0.0; 3] }
}

/// The corners' values at the weights `w`.
pub(super) fn mix<const N: usize>(v: [[f64; N]; 3], w: [f64; 3]) -> [f64; N] {
    std::array::from_fn(|k| v[0][k] * w[0] + v[1][k] * w[1] + v[2][k] * w[2])
}

/// The angle between two directions, in degrees (0 for one without a direction).
pub fn angle(a: P3, b: P3) -> f64 {
    let (la, lb) = (length(a), length(b));
    if la == 0.0 || lb == 0.0 {
        return 0.0;
    }
    (dot(a, b) / (la * lb)).clamp(-1.0, 1.0).acos().to_degrees()
}

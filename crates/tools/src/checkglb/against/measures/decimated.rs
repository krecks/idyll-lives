//! Measuring a decimated model.

use super::*;

/// A far version held to `decimated`, the whole of it at rest as one surface: each side of its box within
/// `DECIMATED_SIZE` of the old one's, its lowest point within `DECIMATED_FLOOR` of `floor` (the full model's),
/// at most `DECIMATED_TRIANGLES` more triangles, and every point of it within `DECIMATED_SURFACE` of the old one.
pub(crate) fn measure_decimated(
    o: &Glb,
    n: &Glb,
    floor: Option<f64>,
    out: &mut Vec<Finding>,
    numbers: &mut Numbers,
) -> R<()> {
    let (a, b) = (whole(o)?, whole(n)?);
    numbers.triangles = if a.triangles.is_empty() {
        0.0
    } else {
        b.triangles.len() as f64 / a.triangles.len() as f64 - 1.0
    };
    if b.triangles.len() as f64 > a.triangles.len() as f64 * (1.0 + DECIMATED_TRIANGLES) {
        out.push(found(
            "triangles",
            format!(
                "{} triangles, the decimated one had {} ({:+.1} %, at most {:+.0} %)",
                b.triangles.len(),
                a.triangles.len(),
                numbers.triangles * 100.0,
                DECIMATED_TRIANGLES * 100.0
            ),
        ));
    }
    if let (Some((alo, ahi)), Some((blo, bhi))) = (a.bounds(), b.bounds()) {
        let size = |lo: P3, hi: P3| -> P3 { std::array::from_fn(|k| hi[k] - lo[k]) };
        let (was, is) = (size(alo, ahi), size(blo, bhi));
        if (0..3).any(|k| (was[k] - is[k]).abs() > DECIMATED_SIZE * was[k] + 1e-6) {
            let text = |s: P3| format!("{:.4} x {:.4} x {:.4} m", s[0], s[1], s[2]);
            out.push(found(
                "size",
                format!(
                    "{}, the decimated one was {} (each side within {})",
                    text(is),
                    text(was),
                    share(DECIMATED_SIZE)
                ),
            ));
        }
        if let Some(floor) = floor.filter(|floor| (blo[1] - floor).abs() > DECIMATED_FLOOR) {
            out.push(found(
                "bounds",
                format!(
                    "its lowest point is at {:.4} m, the model's at {floor:.4} (at most {} apart)",
                    blo[1],
                    mm(DECIMATED_FLOOR)
                ),
            ));
        }
    }
    let (ab, ba) = both_ways(&a, &b);
    let (max, at, of, from) = if ab.max >= ba.max {
        (ab.max, ab.max_at, "old", "new")
    } else {
        (ba.max, ba.max_at, "new", "old")
    };
    numbers.surface_mean = ab.mean.max(ba.mean);
    numbers.surface_max = max;
    if max > DECIMATED_SURFACE {
        out.push(found(
            "surface",
            format!(
                "the {of} file's point {} is {} from the {from} surface, at most {}",
                point(at),
                mm(max),
                mm(DECIMATED_SURFACE)
            ),
        ));
    }
    Ok(())
}

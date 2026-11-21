use super::*;

/// At the toe, where the sole springs up off the floor, every shoe's upper (not its lining) stays
/// above the sole's underside (with the upper cut where the flat sole's top was, a spring taller
/// than the sole, the dress shoe's 11 mm on 10 mm, would leave room for it to come out under there).
#[test]
fn the_upper_stays_above_a_sprung_sole() {
    for family in ["male", "female"] {
        let k = proportions(family, BASE_STAGE).h / 1.7;
        for gid in [
            "shoes.sneakers",
            "shoes.trainers",
            "shoes.boots",
            "shoes.dress",
        ] {
            let st = shoe_style(gid);
            let full = upper(family, BASE_STAGE, gid, false);
            for side in [-1.0, 1.0] {
                let pts: Vec<V3> = full
                    .pos
                    .iter()
                    .zip(&full.fields)
                    .filter(|(q, f)| sign(q.x) == side && f[0] >= 0.0)
                    .map(|(q, _)| *q)
                    .collect();
                // Along the foot as `sole` measures it (over the upper's length), past the arch.
                let (z0, z1) = pts
                    .iter()
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), q| {
                        (a.min(q.z), b.max(q.z))
                    });
                for q in pts {
                    let u = clip((q.z - z0) / (z1 - z0), 0.0, 1.0);
                    let under = st.spring_at(u) * k;
                    assert!(
                        u < 0.7 || q.y > under,
                        "{family} {gid}: the upper at {q:?} is under its sole ({under:.4} m)"
                    );
                }
            }
        }
    }
}

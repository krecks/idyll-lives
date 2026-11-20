use super::*;

/// Painted interior/exterior plaster (grey detail): a fine, even float finish, faint mottling.
pub(super) fn plaster(n: &Noise, size: usize) -> Surface {
    let t = TILE_PLASTER;
    let nf = n.layer(1);
    let nm = n.layer(2);
    paint(size, size, t, &[(0.003, 1.0)], true, |u, v| {
        let fine = fbm(&nf, u, v, 240, 240, 3);
        let grit = fbm(&nf.layer(1), u, v, 600, 600, 1);
        let swirl = fbm(&nf.layer(2), u, v, 12, 12, 4);
        let (pit, _) = grains(&nf.layer(3), u, v, 300, 0.04, 0.35, 31);
        let h = 0.5 + 0.06 * fine + 0.03 * grit + 0.15 * swirl - 0.2 * pit;
        let mottle = fbm(&nm, u, v, 3, 3, 5) + 0.5 * fbm(&nm.layer(1), u, v, 16, 16, 3);
        // Patchy float marks: faint lighter and darker clouds where the trowel went over again.
        let patch = smoothstep(0.02, 0.18, fbm(&nm.layer(2), u, v, 8, 8, 4))
            - smoothstep(0.04, 0.2, -fbm(&nm.layer(3), u, v, 10, 10, 4));
        let sponge = fbm(&nm.layer(4), u, v, 48, 48, 4);
        let l =
            0.86 * (1.0 + 0.10 * mottle + 0.04 * patch + 0.06 * sponge + 0.03 * fine) - 0.05 * pit;
        Px {
            h,
            c: grey(l),
            r: 0.9 + 0.05 * fine,
        }
    })
}

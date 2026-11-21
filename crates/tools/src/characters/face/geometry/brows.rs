//! The face's brows: flattened capsules along the brow line.

use super::super::*;

impl Face {
    // --- brows ---------------------------------------------------------------------------------

    pub(in crate::characters::face) fn brow(&self, p: &Params, k: usize) -> Lip {
        let (h, mm, st) = (&self.h, self.mm, &self.st);
        let [a, b] = h.brows[k];
        let t = linspace(0.0, 1.0, 8);
        // Towards the middle: +x for the left brow.
        let inward = if k == 0 { 1.0 } else { -1.0 };
        let inner =
            11.0 * p.brow_inner + 1.0 * p.brow_outer[k] - 8.0 * p.brow_down[k] - 3.0 * p.sneer[k];
        let mid =
            5.0 * p.brow_inner + 4.5 * p.brow_outer[k] - 4.0 * p.brow_down[k] - 1.2 * p.sneer[k];
        let outer = 1.0 * p.brow_inner + 9.0 * p.brow_outer[k] - 1.0 * p.brow_down[k];
        let (mut pts, mut ry, mut rz) = (Vec::new(), Vec::new(), Vec::new());
        for &t in &t {
            let dy = interp(t, &[0.0, 0.45, 1.0], &[inner, mid, outer]) * mm
                + st.arch * mm * (PI * t.powf(st.arch_at)).sin();
            let dx = (3.5 * p.brow_down[k] + 1.2 * p.sneer[k]) * mm * (1.0 - t).powf(1.5) * inward;
            let x = a[0] + (b[0] - a[0]) * t + dx;
            let y = a[1] + (b[1] - a[1]) * t + dy;
            ry.push(
                st.brow_h
                    * mm
                    * interp(
                        t,
                        &[0.0, 0.25, 0.7, 1.0],
                        &[1.25, 1.1, 0.75, 0.8 * st.brow_tip],
                    ),
            );
            let z_r = st.brow_z * mm * interp(t, &[0.0, 1.0], &[1.25, 0.6]);
            rz.push(z_r);
            let z = self.surf(x, y);
            pts.push(self.project(v3(x, y, z), z_r * 0.75));
        }
        (pts, ry, rz)
    }
}

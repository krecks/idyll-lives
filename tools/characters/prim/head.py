"""
The head interface (`head_frame`): where the skull, face, eyes, mouth, brows and hairline of a
family at a life stage are. The body builds the head from it, the face puts its features on it and
the hair fits over it, so each can be rebuilt on its own.

This stub is an ellipsoid skull; stream A replaces `skull_sdf` (and the numbers it implies) with the
real head's, keeping the interface.
"""

from dataclasses import dataclass, field

import numpy as np

from rig import B, joints, proportions


@dataclass
class HeadFrame:
    family: str
    stage: str
    # Head height (chin to crown), chin height and the crown (m, model space at rest).
    hh: float
    chin_y: float
    top_y: float
    # The skull ellipsoid: centre and semi-axes (x, y, z).
    centre: np.ndarray
    radii: np.ndarray
    # Face plane: z of the face surface at eye level (the eyes, brows and mouth sit on it).
    face_z: float
    # Eyeball centres (left at -x, then right) and radius.
    eyes: np.ndarray
    eye_r: float
    # Iris and pupil radius (m, on the eye's front).
    iris_r: float
    pupil_r: float
    # Mouth: centre of the lip line on the surface, and its width (corner to corner).
    mouth: np.ndarray
    mouth_w: float
    # Brows: per side (left, right), inner and outer end points on the surface.
    brows: np.ndarray = field(default_factory=lambda: np.zeros((2, 2, 3)))
    # Hairline: a closed loop of points on the skull (front, temples, behind the ears, nape).
    hairline: np.ndarray = field(default_factory=lambda: np.zeros((0, 3)))
    chin: np.ndarray = field(default_factory=lambda: np.zeros(3))

    def sdf(self, p):
        """Signed distance (m, approximate; < 0 inside) of points (n, 3) to the skull."""
        return skull_sdf(self, np.atleast_2d(p))

    def surface_z(self, x, y):
        """z of the skull's front surface at (x, y) (the centre's z where (x, y) misses it)."""
        c, r = self.centre, self.radii
        u = 1 - ((np.asarray(x) - c[0]) / r[0]) ** 2 - ((np.asarray(y) - c[1]) / r[1]) ** 2
        return c[2] + r[2] * np.sqrt(np.clip(u, 0, None))

    def scalp(self, p):
        """Scalp weight (0..1) of points (n, 3): 1 well above the hairline, 0 on the face and neck."""
        p = np.atleast_2d(p)
        return np.clip((p[:, 1] - hairline_y(self, p)) / (0.06 * self.hh) + 0.5, 0, 1)


def hairline_y(h, p):
    """Height of the hairline above points (n, 3), by their direction around the head's axis."""
    a = np.arctan2(p[:, 0] - h.centre[0], p[:, 2] - h.centre[2])  # 0 = front, pi = back
    t = np.abs(a) / np.pi
    front, side, back = h.chin_y + 0.74 * h.hh, h.chin_y + 0.56 * h.hh, h.chin_y + 0.18 * h.hh
    return np.where(t < 0.5, front + (side - front) * (t / 0.5) ** 1.5, side + (back - side) * ((t - 0.5) / 0.5))


def skull_sdf(h, p):
    """Ellipsoid distance (scaled by the smallest radius; exact enough near the surface)."""
    q = (p - h.centre) / h.radii
    k = np.linalg.norm(q, axis=1)
    return (k - 1) * h.radii.min()


def head_frame(family, stage):
    """The head of `family` at life stage `stage` (see `HeadFrame`)."""
    pr = proportions(family, stage)
    J = joints(family, stage)
    H, hh = pr['H'], pr['hh'] * pr['H']
    chin_y = H - hh
    # Rounder skulls on small children.
    young = np.clip((pr['hh'] - 1 / 6.5) / (1 / 3.4 - 1 / 6.5), 0, 1)
    radii = np.array([0.37 + 0.06 * young, 0.52, 0.43 + 0.03 * young]) * hh
    centre = np.array([0, chin_y + 0.5 * hh, J[B['Head'], 2] - 0.03 * hh])
    eyes = np.array([J[B['eye_l']], J[B['eye_r']]])
    eye_r = (0.075 + 0.02 * young) * hh
    h = HeadFrame(family, stage, hh, chin_y, H, centre, radii, 0.0, eyes, eye_r, eye_r * 0.5, eye_r * 0.2, np.zeros(3), 0.2 * hh)
    h.face_z = float(h.surface_z(eyes[0, 0], eyes[0, 1]))
    my = chin_y + 0.2 * hh
    h.mouth = np.array([0, my, float(h.surface_z(0, my))])
    by = chin_y + 0.6 * hh
    for k, sx in enumerate((-1, 1)):
        inner = np.array([sx * 0.07 * hh, by, 0])
        outer = np.array([sx * 0.3 * hh, by - 0.02 * hh, 0])
        for e, pt in enumerate((inner, outer)):
            pt[2] = float(h.surface_z(pt[0], pt[1]))
            h.brows[k, e] = pt
    # Hairline loop on the skull surface.
    a = np.linspace(-np.pi, np.pi, 48, endpoint=False)
    dirs = np.stack([np.sin(a), np.zeros_like(a), np.cos(a)], -1)
    pts = centre + dirs * radii
    pts[:, 1] = hairline_y(h, pts)
    # Project each point onto the ellipsoid at its height (keep its direction around the axis).
    v = np.clip(1 - ((pts[:, 1] - centre[1]) / radii[1]) ** 2, 0.02, None) ** 0.5
    pts[:, 0] = centre[0] + np.sin(a) * radii[0] * v
    pts[:, 2] = centre[2] + np.cos(a) * radii[2] * v
    h.hairline = pts
    h.chin = np.array([0, chin_y, float(h.surface_z(0, chin_y + 0.08 * hh))])
    return h

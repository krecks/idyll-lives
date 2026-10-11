"""
STUB hair (stream B replaces this module): every hairstyle as a skull cap over the head
interface's scalp plus a few primitives (sheets, ellipsoids, lofts), and the beard. Contract:

    build_hair(family, stage, head, joints) -> {'hair.<style>': mesh, ...}   # every style but `none`; `hair.beard` where the family has it
    mesh: pos, nor, idx (CCW out), dense (n, NB), fields, tone (z: clump bands 0..1)

The same calls give the same topology at every stage. Skinning: Head, blending to neck_01 and
spine_03 below the jaw so long hair follows the shoulders. The beard's morphs are copied from the
mouth by `build.py` (`follow_mouth`).
"""

import numpy as np

from contract import BEARD_FAMILIES, HAIR_STYLES
from prims import ellipsoid, loft, merge, rings_mesh, rot_x, smoothstep, transform
from rig import B, NB


def _cap(h, offset, scale=(1, 1, 1), shift=(0, 0, 0), rings=7, lon=24, drop=0.0):
    """The skull's scalp (inside the hairline, lowered by `drop` hh) pushed out by `offset` (m)."""
    c = h.centre + np.asarray(shift) * h.hh
    R = (h.radii + offset) * np.asarray(scale)
    a = np.linspace(0, 2 * np.pi, lon, endpoint=False)
    probe = h.centre + np.stack([np.sin(a), np.zeros_like(a), np.cos(a)], -1) * h.radii
    from head import hairline_y
    hy = hairline_y(h, probe) - drop * h.hh
    phi_max = np.arccos(np.clip((hy - h.centre[1]) / h.radii[1], -0.98, 0.98))
    rings_ = []
    for u in np.linspace(1.0, 1.0 / rings, rings):
        phi = phi_max * u
        rings_.append(c + np.stack([np.sin(phi) * np.sin(a) * R[0], np.cos(phi) * R[1], np.sin(phi) * np.cos(a) * R[2]], -1))
    pos, idx, _ = rings_mesh(rings_, end=c + [0, R[1], 0])
    m = dict(pos=pos, idx=idx)
    return _normals(m)


def _sheet(h, a0, a1, y0, y1, offset, flare=0.0, cols=14, rows=6, flat_back=None):
    """A hanging sheet round the head between azimuths a0..a1 (0 = front) from y0 down to y1 (m)."""
    a = np.linspace(a0, a1, cols)
    pts = []
    for k, y in enumerate(np.linspace(y0, y1, rows)):
        u = k / (rows - 1)
        v = np.sqrt(np.clip(1 - ((min(y, h.centre[1]) - h.centre[1]) / h.radii[1]) ** 2, 0.55, 1))
        rx = (h.radii[0] * v + offset) * (1 + flare * u)
        rz = (h.radii[2] * v + offset) * (1 + flare * u)
        row = np.stack([h.centre[0] + np.sin(a) * rx, np.full_like(a, y), h.centre[2] + np.cos(a) * rz], -1)
        pts.append(row)
    P = np.array(pts)
    idx = []
    for i in range(rows - 1):
        for j in range(cols - 1):
            p, q, r, s = i * cols + j, i * cols + j + 1, (i + 1) * cols + j + 1, (i + 1) * cols + j
            # Rows go down, azimuth goes round towards +x: these face out.
            idx += [p, r, q, p, s, r]
    return _normals(dict(pos=P.reshape(-1, 3), idx=np.array(idx, np.int64)))


def _normals(m):
    from meshops import smooth_normals
    m['nor'] = smooth_normals(m['pos'], m['idx'])
    return m


def build_hair(family, stage, head, P):
    h = head
    hh = h.hh
    back = np.pi
    styles = {}
    rng = np.random.default_rng(7)
    cap = lambda off=0.03, **k: _cap(h, off * hh, **k)
    styles['short'] = [cap(0.035, scale=(1, 1.04, 1))]
    styles['crop'] = [cap(0.018)]
    styles['layered'] = [cap(0.03), _sheet(h, np.radians(70), np.radians(290), h.chin_y + 0.62 * hh, h.chin_y + 0.08 * hh, 0.04 * hh, flare=0.15)]
    petals = []
    for k in range(16):
        az = rng.uniform(-2.4, 2.4)
        pol = rng.uniform(0.15, 1.0)
        d = np.array([np.sin(pol) * np.sin(az), np.cos(pol), np.sin(pol) * np.cos(az)])
        c = h.centre + d * (h.radii + 0.04 * hh)
        petals.append(ellipsoid(c, np.array([0.12, 0.07, 0.12]) * hh, 4, 6))
    styles['tousled'] = [cap(0.03)] + petals
    lobe = ellipsoid(h.centre + np.array([0.08, 0.36, 0.3]) * hh, np.array([0.28, 0.08, 0.12]) * hh, 5, 10)
    styles['sideSwept'] = [cap(0.03), transform(lobe, rot_x(-0.3), about=h.centre)]
    styles['afro'] = [cap(0.05, scale=(1.32, 1.22, 1.25), shift=(0, 0.06, -0.06), rings=8, lon=28)]
    styles['bob'] = [cap(0.03), _sheet(h, np.radians(60), np.radians(300), h.chin_y + 0.6 * hh, h.chin_y + 0.02 * hh, 0.045 * hh, flare=0.08)]
    styles['long'] = [cap(0.03), _sheet(h, np.radians(75), np.radians(285), h.chin_y + 0.6 * hh, h.chin_y - 1.6 * hh, 0.045 * hh, flare=0.1, rows=9)]
    tail = h.centre + np.array([0, 0.05, -1]) * h.radii + [0, 0, -0.03 * hh]
    pts = tail + np.array([[0, 0, 0], [0, -0.25, -0.12], [0, -0.6, -0.15], [0, -1.0, -0.1]]) * hh
    styles['ponytail'] = [cap(0.02), ellipsoid(tail, np.array([0.1, 0.1, 0.07]) * hh, 4, 8), loft(pts, np.array([0.1, 0.12, 0.09, 0.03]) * hh, np.array([0.1, 0.12, 0.09, 0.03]) * hh, 8, caps=1)]
    chain = [ellipsoid(h.centre + np.array([0, -0.15 - 0.17 * k, -1.02]) * [1, hh, h.radii[2]] + [0, 0, -0.02 * hh * k], np.array([0.09, 0.11, 0.08]) * hh * (1 - 0.04 * k), 4, 8) for k in range(9)]
    styles['braid'] = [cap(0.02)] + chain
    buns = [ellipsoid(h.centre + np.array([sx * 0.62, 0.72, -0.25]) * h.radii, np.array([0.17, 0.17, 0.17]) * hh, 6, 10) for sx in (-1, 1)]
    styles['bun'] = [cap(0.025)] + buns
    out = {}
    assert set(styles) | {'none'} == set(HAIR_STYLES)
    for name, comps in styles.items():
        out[f'hair.{name}'] = _finish(merge(comps), h, P)
    if family in BEARD_FAMILIES:
        # Over the jaw and chin, below the mouth (it follows the mouth's morphs, see build.py).
        beard = _sheet(h, np.radians(-105), np.radians(105), h.mouth[1] - 0.06 * hh, h.chin_y - 0.06 * hh, 0.03 * hh, cols=16, rows=5)
        out['hair.beard'] = _finish(beard, h, P)
    return out


def _finish(m, h, P):
    # Nothing closer than 3 mm to the skull: push such vertices out along the ellipsoid's rays.
    pos = m['pos'].copy()
    q = (pos - h.centre) / h.radii
    k = np.linalg.norm(q, axis=1)
    lim = 1 + 0.003 / h.radii.min()
    inside = k < lim
    pos[inside] = h.centre + q[inside] / np.maximum(k[inside, None], 1e-9) * lim * h.radii
    if inside.any():
        from meshops import smooth_normals
        m['nor'] = smooth_normals(pos, m['idx'])
    m['pos'] = pos
    n = len(pos)
    neck_y = P['neck_01'][1]
    a = smoothstep(h.chin_y, neck_y, pos[:, 1])
    b = smoothstep(neck_y, neck_y - 0.12 * h.top_y, pos[:, 1])
    d = np.zeros((n, NB))
    d[:, B['Head']] = 1 - a
    d[:, B['neck_01']] = a * (1 - b)
    d[:, B['spine_03']] = a * b
    m['dense'] = d
    f = np.zeros((n, 4))
    f[:, 0] = 0.1
    m['fields'] = f
    tone = np.zeros((n, 4))
    az = np.arctan2(pos[:, 0] - h.centre[0], pos[:, 2] - h.centre[2])
    tone[:, 2] = 0.5 + 0.5 * np.sin(az * 9 + pos[:, 1] / h.hh * 3)
    m['tone'] = tone
    return m

"""
Mesh primitives for the generated residents: lofts (cross-sections along a curve, with rounded or
flat ends; along a smooth spline they make bent capsules), ellipsoids, spherical caps, lathes
(a profile turned round an axis), tori, sheets through grids of points, seeded value noise and
merging. Each has a fixed vertex layout for given counts, so the same call at every life stage
gives the same topology (stage deltas need it).

Winding: every primitive is counter-clockwise seen from outside (`cross(b - a, c - a)` points
out, as `meshops.smooth_normals` expects); `write.py` turns faces round for Babylon.
"""

import numpy as np

from meshops import smooth_normals


def frame(t, hint):
    """Unit side vector U (towards `hint`, square to t) and V = t x U, so U x V = t."""
    u = hint - t * (hint @ t)
    if np.linalg.norm(u) < 1e-6:
        u = np.cross(t, [1.0, 0, 0] if abs(t[0]) < 0.9 else [0, 1.0, 0])
    u = u / np.linalg.norm(u)
    return u, np.cross(t, u)


def rings_mesh(rings, start=None, end=None):
    """
    Stacked rings (k, segs, 3), ordered along the axis, closed around; `start` / `end` add a pole
    vertex fanned to the first / last ring. Returns pos, idx and the ring index of each vertex.
    """
    rings = np.asarray(rings, np.float64)
    k, s, _ = rings.shape
    pos = [rings.reshape(-1, 3)]
    ring = [np.repeat(np.arange(k, dtype=np.float64), s)]
    idx = []
    j = np.arange(s)
    j1 = (j + 1) % s
    for i in range(k - 1):
        a, b, c, d = i * s + j, i * s + j1, (i + 1) * s + j1, (i + 1) * s + j
        idx.append(np.stack([a, b, c, a, c, d], 1).ravel())
    n = k * s
    if start is not None:
        pos.append(np.asarray(start, np.float64)[None])
        ring.append(np.array([-1.0]))
        idx.append(np.stack([np.full(s, n), j1, j], 1).ravel())
        n += 1
    if end is not None:
        pos.append(np.asarray(end, np.float64)[None])
        ring.append(np.array([k]))
        base = (k - 1) * s
        idx.append(np.stack([np.full(s, n), base + j, base + j1], 1).ravel())
    return np.concatenate(pos), np.concatenate(idx).astype(np.int64), np.concatenate(ring)


def loft(points, rx, rz, segs, hint=(0, 0, 1), caps=2, cap_start=True, cap_end=True, power=2.0, twist=0.0):
    """
    A tube along `points` (k, 3) with elliptic (superelliptic for `power` > 2) cross-sections of
    semi-axes `rx` (along the side vector, towards `hint`) and `rz` (the other way), and rounded
    ends of `caps` rings each (False: open). Returns dict(pos, nor, idx, t): t is the axial
    parameter (0 at the first point, 1 at the last; the caps reach a little beyond).
    """
    P = np.asarray(points, np.float64)
    k = len(P)
    rx = np.broadcast_to(np.asarray(rx, np.float64), (k,))
    rz = np.broadcast_to(np.asarray(rz, np.float64), (k,))
    T = np.gradient(P, axis=0)
    T /= np.linalg.norm(T, axis=1, keepdims=True)
    seg = np.linalg.norm(np.diff(P, axis=0), axis=1)
    L = max(seg.sum(), 1e-9)
    tt = np.concatenate([[0], np.cumsum(seg)]) / L
    th = np.linspace(0, 2 * np.pi, segs, endpoint=False) + twist
    c, s = np.cos(th), np.sin(th)
    e = 2 / power
    cs, ss = np.sign(c) * np.abs(c) ** e, np.sign(s) * np.abs(s) ** e
    hint = np.asarray(hint, np.float64)
    frames = [frame(T[i], hint) for i in range(k)]

    def ring(i, scale=1.0, shift=0.0):
        U, V = frames[i]
        return P[i] + T[i] * shift + (U[None] * cs[:, None] * rx[i] + V[None] * ss[:, None] * rz[i]) * scale

    rings, params = [], []
    if cap_start and caps:
        for q in range(caps, 0, -1):
            a = q / (caps + 1) * np.pi / 2
            r = min(rx[0], rz[0])
            rings.append(ring(0, np.cos(a), -np.sin(a) * r))
            params.append(-np.sin(a) * r / L)
    for i in range(k):
        rings.append(ring(i))
        params.append(tt[i])
    if cap_end and caps:
        for q in range(1, caps + 1):
            a = q / (caps + 1) * np.pi / 2
            r = min(rx[-1], rz[-1])
            rings.append(ring(k - 1, np.cos(a), np.sin(a) * r))
            params.append(1 + np.sin(a) * r / L)
    start = P[0] - T[0] * min(rx[0], rz[0]) if (cap_start and caps) else None
    end = P[-1] + T[-1] * min(rx[-1], rz[-1]) if (cap_end and caps) else None
    pos, idx, ri = rings_mesh(rings, start, end)
    params = np.array(params)
    t = np.where(ri < 0, params[0] - 0.02, np.where(ri >= len(params), params[-1] + 0.02, params[np.clip(ri, 0, len(params) - 1).astype(int)]))
    return dict(pos=pos, idx=idx, t=t, nor=smooth_normals(pos, idx))


def ellipsoid(centre, radii, lat, lon, top=0.0, bottom=np.pi, axis_hint=(0, 0, 1)):
    """
    An ellipsoid (or a band of one: latitudes `top`..`bottom`, radians from the +y pole) of `lat`
    rings x `lon` segments. Poles are added where the band reaches them. Returns dict(pos, nor, idx, t)
    with t = 0 at the top, 1 at the bottom.
    """
    C = np.asarray(centre, np.float64)
    R = np.asarray(radii, np.float64)
    full_top, full_bot = top <= 1e-6, bottom >= np.pi - 1e-6
    # Rings from the bottom up (the loft axis is +y, so faces wind outward).
    lo = bottom if not full_bot else np.pi - np.pi / (lat + 1)
    hi = top if not full_top else np.pi / (lat + 1)
    phi = np.linspace(lo, hi, lat)
    th = np.linspace(0, 2 * np.pi, lon, endpoint=False)
    # Ring points: x = sin(phi) sin(th), z = sin(phi) cos(th) (U = z, V = x: U x V = +y).
    rings = [C + np.stack([np.sin(p) * np.sin(th) * R[0], np.full(lon, np.cos(p) * R[1]), np.sin(p) * np.cos(th) * R[2]], -1) for p in phi]
    pos, idx, ri = rings_mesh(rings, C - [0, R[1], 0] if full_bot else None, C + [0, R[1], 0] if full_top else None)
    t = 1 - (pos[:, 1] - (C[1] - R[1])) / (2 * R[1])
    return dict(pos=pos, idx=idx, t=t, nor=smooth_normals(pos, idx))


def transform(m, rot=None, move=(0, 0, 0), scale=(1, 1, 1), about=(0, 0, 0)):
    """A copy of mesh `m` scaled and rotated (3x3) about `about`, then moved."""
    out = dict(m)
    p = (m['pos'] - about) * np.asarray(scale)
    n = m['nor'] / np.asarray(scale)
    if rot is not None:
        p = p @ np.asarray(rot).T
        n = n @ np.asarray(rot).T
    out['pos'] = p + about + np.asarray(move)
    out['nor'] = n / np.maximum(np.linalg.norm(n, axis=1, keepdims=True), 1e-12)
    return out


def rot_x(a):
    c, s = np.cos(a), np.sin(a)
    return np.array([[1, 0, 0], [0, c, -s], [0, s, c]])


def rot_y(a):
    c, s = np.cos(a), np.sin(a)
    return np.array([[c, 0, s], [0, 1, 0], [-s, 0, c]])


def rot_z(a):
    c, s = np.cos(a), np.sin(a)
    return np.array([[c, -s, 0], [s, c, 0], [0, 0, 1]])


def merge(meshes, keys=('pos', 'nor')):
    """One mesh from several (indices offset); per-vertex arrays named in `keys` are concatenated."""
    out = {k: np.concatenate([m[k] for m in meshes]) for k in keys}
    idx, n = [], 0
    for m in meshes:
        idx.append(m['idx'] + n)
        n += len(m['pos'])
    out['idx'] = np.concatenate(idx).astype(np.int64)
    return out


def smoothstep(a, b, x):
    t = np.clip((np.asarray(x, np.float64) - a) / (b - a), 0, 1)
    return t * t * (3 - 2 * t)


def grid(P, wrap=False):
    """
    Faces of a sheet through a grid of points P (rows, cols, 3): counter-clockwise seen from the
    side where the columns run right and the rows run up (`wrap`: the last column joins the first).
    Returns pos (rows * cols, 3) and idx.
    """
    P = np.asarray(P, np.float64)
    r, c, _ = P.shape
    i, j = np.meshgrid(np.arange(r - 1), np.arange(c if wrap else c - 1), indexing='ij')
    i, j = i.ravel(), j.ravel()
    j1 = (j + 1) % c
    a, b, cc, d = i * c + j, i * c + j1, (i + 1) * c + j1, (i + 1) * c + j
    idx = np.stack([a, b, cc, a, cc, d], 1).ravel()
    return P.reshape(-1, 3), idx.astype(np.int64)


def orient(pos, idx, out):
    """`idx` turned round if its faces mostly point against `out` (per vertex directions, (n, 3))."""
    t = np.asarray(idx).reshape(-1, 3)
    fn = np.cross(pos[t[:, 1]] - pos[t[:, 0]], pos[t[:, 2]] - pos[t[:, 0]])
    o = out[t[:, 0]] + out[t[:, 1]] + out[t[:, 2]]
    return idx if (fn * o).sum() >= 0 else t[:, [0, 2, 1]].ravel()


def spline(points, n):
    """`n` points along a Catmull-Rom curve through `points` (k, 3), evenly spaced in parameter."""
    P = np.asarray(points, np.float64)
    Q = np.concatenate([2 * P[:1] - P[1:2], P, 2 * P[-1:] - P[-2:-1]])
    u = np.linspace(0, len(P) - 1, n)
    k = np.minimum(u.astype(int), len(P) - 2)
    t = (u - k)[:, None]
    p0, p1, p2, p3 = Q[k], Q[k + 1], Q[k + 2], Q[k + 3]
    return 0.5 * ((2 * p1) + (p2 - p0) * t + (2 * p0 - 5 * p1 + 4 * p2 - p3) * t * t + (3 * p1 - p0 - 3 * p2 + p3) * t ** 3)


def lathe(profile, segs, centre=(0, 0, 0), start=None, end=None, a0=0.0, a1=2 * np.pi):
    """
    A profile turned round the y axis through `centre`: rows of (rx, y, rz) (an elliptic ring of
    semi-axes rx, rz at height y, from the top down), `segs` around (a0..a1, 0 = +z, towards +x;
    closed when the arc is whole). `start` / `end` cap the first / last ring with a pole.
    Faces point out. Returns dict(pos, idx, nor, t) with t the row (0 .. 1 down the profile).
    """
    C = np.asarray(centre, np.float64)
    prof = np.asarray(profile, np.float64)
    whole = abs(a1 - a0 - 2 * np.pi) < 1e-9
    a = np.linspace(a0, a1, segs, endpoint=not whole)
    rings = [C + np.stack([np.sin(a) * rx, np.full_like(a, y), np.cos(a) * rz], -1) for rx, y, rz in prof]
    if whole:
        # rings_mesh winds outward for rings stacked up the axis; the profile runs down.
        pos, idx, ri = rings_mesh(rings[::-1], C + [0, end, 0] if end is not None else None, C + [0, start, 0] if start is not None else None)
        t = 1 - ri / max(len(prof) - 1, 1)
    else:
        pos, idx = grid(np.array(rings[::-1]))
        t = 1 - np.repeat(np.arange(len(prof)), segs) / max(len(prof) - 1, 1)
        out = pos - C
        out[:, 1] = 0
        idx = orient(pos, idx, out)
    return dict(pos=pos, idx=idx, nor=smooth_normals(pos, idx), t=np.clip(t, 0, 1))


def torus(centre, R, r, segs, sides, axis=(0, 1, 0), rz=None):
    """A torus round `axis` through `centre`: ring radius R, tube radius r (rz across, default r)."""
    C = np.asarray(centre, np.float64)
    ax = np.asarray(axis, np.float64)
    ax = ax / np.linalg.norm(ax)
    U, V = frame(ax, np.array([0, 0, 1.0]) if abs(ax[2]) < 0.9 else np.array([1.0, 0, 0]))
    a = np.linspace(0, 2 * np.pi, segs, endpoint=False)
    b = np.linspace(0, 2 * np.pi, sides, endpoint=False)
    rz = r if rz is None else rz
    d = np.cos(a)[:, None] * U + np.sin(a)[:, None] * V
    P = C + d[:, None] * (R + np.cos(b)[None, :, None] * r) + ax * np.sin(b)[None, :, None] * rz
    pos, idx = grid(P, wrap=True)
    # Close the ring of tubes too.
    n = segs * sides
    last = np.arange(sides)
    j1 = (last + 1) % sides
    a_, b_, c_, d_ = (segs - 1) * sides + last, (segs - 1) * sides + j1, j1, last
    idx = np.concatenate([idx, np.stack([a_, b_, c_, a_, c_, d_], 1).ravel()])
    cen = C + d * R
    out = pos - np.repeat(cen, sides, 0)
    idx = orient(pos, idx, out)
    assert len(pos) == n
    return dict(pos=pos, idx=idx, nor=smooth_normals(pos, idx))


def noise3(p, seed=0, scale=1.0):
    """Seeded smooth value noise (-1..1) at points p (n, 3), features about 1 / `scale` apart."""
    q = np.asarray(p, np.float64) * scale
    i = np.floor(q).astype(np.int64)
    f = q - i
    f = f * f * (3 - 2 * f)

    def h(x, y, z):
        v = (x * 73856093) ^ (y * 19349663) ^ (z * 83492791) ^ (seed * 2654435761)
        v = (v ^ (v >> 13)) * 1274126177
        return ((v ^ (v >> 16)) & 0xFFFF) / 32767.5 - 1

    out = 0
    for dx in (0, 1):
        for dy in (0, 1):
            for dz in (0, 1):
                w = (f[:, 0] if dx else 1 - f[:, 0]) * (f[:, 1] if dy else 1 - f[:, 1]) * (f[:, 2] if dz else 1 - f[:, 2])
                out = out + w * h(i[:, 0] + dx, i[:, 1] + dy, i[:, 2] + dz)
    return out

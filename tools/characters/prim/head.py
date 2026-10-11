"""
The head: its cage (part of the body's, see cage.py) and the head interface (`head_frame`): where
the skull, face, eyes, mouth, brows and hairline of a family at a life stage are. The body builds
the head from the cage here, the face puts its features on the frame and the hair fits over it, so
each can be rebuilt on its own.

The head cage is 8 rings of 12 vertices (`ROWS`, columns at `COLS` degrees round the head, 0 = the
face, 90 = the resident's right) under a crown patch. Its vertices lie where rays from the head's
middle leave a smooth head (`head_shape`: a soft union of a cranium, cheeks-and-jaw and chin
ellipsoids, rounder on small children); an ear is pulled out of each side, and a small nose, a soft
brow and an eye socket round each eyeball (its front a few mm proud of the face; the lids close
over it) are offsets on single vertices. There is no mouth hole: the lips and the mouth plate sit on
the face.

`HeadFrame.sdf` and `surface_z` are measured on the real (subdivided) head and neck without the
ears, so the face features and the hair sit on what is drawn; `ears` bound the ears. `centre` /
`radii` are a loose ellipsoid round the skull for code that wants a simple one.
"""

from dataclasses import dataclass, field
from functools import lru_cache

import numpy as np
from scipy.spatial import cKDTree

from rig import B, joints, proportions

# Columns (degrees round the head's vertical axis; 0 = the face, + = the resident's right, +x).
COLS = np.array([0, 14, 28, 50, 92, 122, 180, -122, -92, -50, -28, -14], np.float64)
EYE_COL = 2       # the socket vertex (and its mirror, 12 - 2)
EAR_COLS = (4, 5)  # the ear leaves through the face between these columns, rows 3-4
# Rows from the jaw up: elevation (degrees) of the ray from the head's middle (`MIDDLE`) at the
# front and at the back. The smooth head is a soft union of ellipsoids (`SHAPES`): the cranium,
# the cheeks and jaw, the chin; the rows' vertices are where the rays leave it. The features
# (nose, brow, eye sockets, ears) are cage offsets on top (see cage.py).
ROWS = [(-52, -35), (-42, -20), (-20, -2), (-8, 10), (6, 22), (24, 36), (46, 52), (68, 70)]
MIDDLE = (0.40, 0.0)  # (height above the chin, z), fractions of hh
# Per family: (centre, radii) of the cranium, the cheeks and jaw, the chin (fractions of hh from
# the chin and the Head joint's z), and the blend width.
SHAPES = {
    'male': dict(cranium=((0, 0.56, -0.02), (0.375, 0.44, 0.445)), cheeks=((0, 0.32, 0.07), (0.335, 0.31, 0.33)),
                 chin=((0, 0.12, 0.21), (0.15, 0.11, 0.12)), blend=0.14),
    'female': dict(cranium=((0, 0.56, -0.02), (0.37, 0.44, 0.44)), cheeks=((0, 0.32, 0.07), (0.325, 0.30, 0.325)),
                   chin=((0, 0.13, 0.20), (0.13, 0.10, 0.11)), blend=0.14),
}
# How far each feature's cage vertex moves (fractions of hh; the surface follows by about half).
NOSE = {'male': 0.085, 'female': 0.07}
BROW = {'male': 0.025, 'female': 0.014}
SOCKET = 2.2


def youth(stage_hh):
    """0 for adult heads (1 / 6.5 of the height), 1 for babies (1 / 3.4)."""
    return float(np.clip((stage_hh - 1 / 6.5) / (1 / 3.4 - 1 / 6.5), 0, 1))


def _ellipsoid(p, c, r):
    """Approximate signed distance to an ellipsoid (exact on its surface)."""
    q = (p - c) / r
    k0 = np.linalg.norm(q, axis=-1)
    k1 = np.linalg.norm(q / r, axis=-1)
    return k0 * (k0 - 1) / np.maximum(k1, 1e-9)


def _smin(a, b, k):
    h = np.clip(0.5 + 0.5 * (b - a) / k, 0, 1)
    return b + (a - b) * h - k * h * (1 - h)


def head_shape(family, stage):
    """The smooth head (no features) as a signed distance function of points in hh units
    (origin: the chin's height and the Head joint's z)."""
    pr = proportions(family, stage)
    yg = youth(pr['hh'])
    S = SHAPES[family]
    cc, cr = np.array(S['cranium'][0], float), np.array(S['cranium'][1], float)
    kc, kr = np.array(S['cheeks'][0], float), np.array(S['cheeks'][1], float)
    nc, nr = np.array(S['chin'][0], float), np.array(S['chin'][1], float)
    # Small children: a bigger, rounder cranium, chubby cheeks low on a short face, a small chin.
    cr = cr + yg * np.array([0.035, 0.0, 0.02])
    kc = kc + yg * np.array([0, -0.03, -0.01])
    kr = kr + yg * np.array([0.04, 0.0, 0.0])
    nr = nr * (1 - 0.3 * yg)
    if stage == 'elder':
        kr = kr + [0.008, 0, 0]
    b = S['blend']

    def f(p):
        d = _smin(_ellipsoid(p, cc, cr), _ellipsoid(p, kc, kr), b)
        return _smin(d, _ellipsoid(p, nc, nr), 0.09)
    return f


def _cast(f, o, d):
    """Where rays from inside (o) along unit directions d (n, 3) leave the shape f (bisection)."""
    lo = np.zeros(len(d))
    hi = np.full(len(d), 1.2)
    for _ in range(40):
        mid = (lo + hi) / 2
        inside = f(o + d * mid[:, None]) < 0
        lo = np.where(inside, mid, lo)
        hi = np.where(inside, hi, mid)
    return o + d * lo[:, None]


def head_cage(cage, family, stage):
    """Adds the head to `cage`; returns its jaw ring (12 ids, `COLS` order) for the neck."""
    pr = proportions(family, stage)
    J = joints(family, stage)
    H = pr['H']
    hh = pr['hh'] * H
    chin = H - hh
    yg = youth(pr['hh'])
    f = head_shape(family, stage)
    th = np.radians(COLS)
    s = (1 - np.cos(th)) / 2
    o = np.array([0, MIDDLE[0], MIDDLE[1]])
    eyes = np.array([J[B['eye_l']], J[B['eye_r']]])
    eye_r = (0.075 + 0.02 * yg) * hh
    W = {'Head': 1.0}
    grid = []
    for ef, eb in ROWS:
        el = np.radians(ef + (eb - ef) * s)
        d = np.stack([np.cos(el) * np.sin(th), np.sin(el), np.cos(el) * np.cos(th)], -1)
        grid.append(_cast(f, o, d))
    origin = np.array([0, chin, 0])
    grid = np.array(grid) * hh + origin  # (8, 12, 3) the smooth head
    D = np.zeros_like(grid)
    Z = np.array([0, 0, 1.0])
    young = 1 - 0.55 * yg
    # A small round nose (tip, base, wings, bridge) and a soft brow.
    n = NOSE[family] * young * hh * (1.1 if stage == 'elder' else 1.0)
    D[3, 0] += [0, 0.05 * n, n]
    D[2, 0] += Z * 0.3 * n
    D[3, [1, 11]] += Z * 0.06 * n
    D[4, 0] += Z * 0.15 * n
    b = BROW[family] * (1 - 0.6 * yg) * hh
    D[5, [1, 2, 3, 9, 10, 11]] += Z * b
    D[5, 0] += Z * 0.5 * b
    # Eye sockets: the socket vertex pulls the face in behind the eyeball (its front stays proud).
    for k, cc in ((0, 12 - EYE_COL), (1, EYE_COL)):
        e = eyes[k]
        sx = np.sign(e[0])
        D[4, cc] = SOCKET * (e + [-sx * 0.3 * eye_r, 0, 0.16 * eye_r] - grid[4, cc])
        inner = (cc - int(sx)) % 12
        D[4, inner] += [-sx * 0.1 * eye_r, 0, -0.45 * eye_r]
        D[3, cc] += [0, -0.1 * eye_r, -0.15 * eye_r]
    ids = []
    for r in range(len(ROWS)):
        cav = np.zeros(12)
        blush = np.zeros(12)
        # (Tone is the base stage's for every stage; the shader deepens cavities with age.)
        if r == 3:
            blush[[2, 10]] = 0.32 + 0.2 * yg
            blush[[3, 9]] = 0.2 + 0.15 * yg
            blush[0] = 0.15
            cav[[2, 10]] = 0.08     # under the eyes
        if r == 2:
            cav[[1, 11]] = 0.15     # beside the nose
            cav[[2, 10]] = 0.1      # cheek folds
        ids.append(cage.ring(grid[r], W, 'head', disp=list(D[r]), cav=list(cav), blush=list(blush)))
    for r in range(len(ROWS) - 1):
        skip = (EAR_COLS[0], 12 - EAR_COLS[1]) if r == 3 else ()
        cage.band(ids[r], ids[r + 1], skip=skip)
    # Ears: from the face left open between columns 92 and 122 (and -122 .. -92), rows 3-4. The
    # smooth head has a low dome there (the skull the hair fits over); the ear is the offset.
    for c0, c1 in ((EAR_COLS[0], EAR_COLS[1]), (12 - EAR_COLS[1], 12 - EAR_COLS[0])):
        loop = [ids[3][c0], ids[3][c1], ids[4][c1], ids[4][c0]]
        P = np.array([cage.P[i] for i in loop])
        m = P.mean(0)
        sx = np.sign(m[0])
        out = np.array([sx, 0, 0.0])
        base, offs = [], []
        for i, p in enumerate(P):
            top = i >= 2
            front = i in (0, 3) if sx > 0 else i in (1, 2)
            b = m + (p - m) * 0.6 + out * 0.012 * hh
            q = m + (p - m) * [0.6, 1.0, 0.55]
            q = q + out * (0.04 + 0.015 * yg) * hh * (0.7 if front else 1.0)
            q[1] += (0.085 if top else -0.06) * hh * (1 - 0.3 * yg)
            q[2] -= 0.015 * hh
            base.append(b)
            offs.append(q - b)
        e = cage.ring(np.array(base), W, 'ear', disp=offs, blush=0.35, cav=0.25)
        cage.band(loop, e)
        cage.cap(e)
    # Crown: a 2 x 4 patch over row 7 (right edge columns 14..130, left edge -14..-130), its middle
    # line on the head's top.
    top = ids[-1]
    up = []
    for el, az in ((80, 0.0), (90, 0.0), (80, np.pi)):
        e = np.radians(el)
        up.append([np.cos(e) * np.sin(az), np.sin(e), np.cos(e) * np.cos(az)])
    mid = [cage.add(p, W, 'head') for p in _cast(f, o, np.array(up)) * hh + origin]
    right = [top[1], top[2], top[3], top[4], top[5]]
    left = [top[11], top[10], top[9], top[8], top[7]]
    centre = [top[0]] + mid + [top[6]]
    for j in range(4):
        cage.face(left[j], centre[j], centre[j + 1], left[j + 1])
        cage.face(centre[j], right[j], right[j + 1], centre[j + 1])
    return ids[0]


@dataclass
class HeadFrame:
    family: str
    stage: str
    # Head height (chin to crown), chin height and the crown (m, model space at rest).
    hh: float
    chin_y: float
    top_y: float
    # A loose ellipsoid round the skull: centre and semi-axes (x, y, z). It encloses the cranium
    # with a margin (more front to back, clearing the brow and chin); fit close things to `sdf`.
    centre: np.ndarray
    radii: np.ndarray
    # Face plane: z of the face surface at eye level (round the eye sockets).
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
    # Ears (left, right): centre and half extents (x, y, z) of a box round each. They stand off the
    # skull and are not part of `sdf` / `surface_z`; hair should go round or over them.
    ears: np.ndarray = field(default_factory=lambda: np.zeros((2, 2, 3)))
    # The real head and neck surface (dense samples, their normals, and its triangles).
    samples: np.ndarray = field(default_factory=lambda: np.zeros((0, 3)), repr=False)
    normals: np.ndarray = field(default_factory=lambda: np.zeros((0, 3)), repr=False)
    tris: np.ndarray = field(default_factory=lambda: np.zeros((0, 3, 3)), repr=False)

    def sdf(self, p):
        """Signed distance (m; < 0 inside) of points (n, 3) to the head and neck as drawn."""
        return skull_sdf(self, np.atleast_2d(np.asarray(p, np.float64)))

    def surface_z(self, x, y):
        """z of the head's front surface at (x, y) (the skull centre's z where (x, y) misses it)."""
        x, y = np.broadcast_arrays(np.asarray(x, np.float64), np.asarray(y, np.float64))
        out = front_z(self.tris, x.ravel(), y.ravel(), self.centre[2])
        return out.reshape(x.shape) if x.ndim else float(out[0])

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
    """Distance to the nearest surface sample, signed by the samples' normals (exact enough near
    the surface; the ears are not part of the skull, see `HeadFrame.ears`)."""
    tree = _tree(h)
    d, i = tree.query(p, k=16)
    # A smooth blend of the nearest samples' tangent-plane distances (moving least squares, Gaussian
    # weights about the nearest one), so the field is continuous wherever it is evaluated.
    side = np.einsum('ijk,ijk->ij', p[:, None, :] - h.samples[i], h.normals[i])
    sigma = 1.5 * _spacing(h)
    w = np.exp(-(d * d - d[:, :1] ** 2) / (sigma * sigma))
    s = (w * side).sum(1) / w.sum(1)
    # Farther off, the distance to the nearest sample, with that sign (blended, so still smooth).
    t = np.clip((d[:, 0] - sigma) / (2 * sigma), 0, 1) * np.clip(np.abs(s) / sigma, 0, 1)
    t = t * t * (3 - 2 * t)
    s = s + t * (np.sign(s) * d[:, 0] - s)
    # The samples stop at the base of the neck: below that is outside.
    return np.maximum(s, h.samples[:, 1].min() - p[:, 1])


_trees = {}
_spacings = {}


def _spacing(h):
    """Typical distance between neighbouring surface samples."""
    k = (h.family, h.stage)
    if k not in _spacings:
        _spacings[k] = float(np.median(_tree(h).query(h.samples, k=2)[0][:, 1]))
    return _spacings[k]


def _tree(h):
    k = (h.family, h.stage)
    t = _trees.get(k)
    if t is None:
        t = _trees[k] = cKDTree(h.samples)
    return t


def front_z(tris, x, y, miss):
    """The largest z where vertical lines (x, y) cross triangles (t, 3, 3); `miss` where none does."""
    out = np.full(len(x), -np.inf)
    a, b, c = tris[:, 0], tris[:, 1], tris[:, 2]
    lo = np.minimum(np.minimum(a, b), c)[:, :2]
    hi = np.maximum(np.maximum(a, b), c)[:, :2]
    v0 = b[:, :2] - a[:, :2]
    v1 = c[:, :2] - a[:, :2]
    den = v0[:, 0] * v1[:, 1] - v1[:, 0] * v0[:, 1]
    ok = np.abs(den) > 1e-14
    for k in range(len(x)):
        q = np.array([x[k], y[k]])
        m = ok & (lo[:, 0] <= q[0]) & (q[0] <= hi[:, 0]) & (lo[:, 1] <= q[1]) & (q[1] <= hi[:, 1])
        if not m.any():
            continue
        v2 = q - a[m, :2]
        d = den[m]
        u = (v2[:, 0] * v1[m, 1] - v1[m, 0] * v2[:, 1]) / d
        v = (v0[m, 0] * v2[:, 1] - v2[:, 0] * v0[m, 1]) / d
        hit = (u >= -1e-9) & (v >= -1e-9) & (u + v <= 1 + 1e-9)
        if hit.any():
            z = a[m, 2] + u * (b[m, 2] - a[m, 2]) + v * (c[m, 2] - a[m, 2])
            out[k] = z[hit].max()
    return np.where(np.isfinite(out), out, miss)


@lru_cache(maxsize=None)
def head_frame(family, stage):
    """The head of `family` at life stage `stage` (see `HeadFrame`)."""
    from body import surface  # the head is drawn as part of the body
    pr = proportions(family, stage)
    J = joints(family, stage)
    H, hh = pr['H'], pr['hh'] * pr['H']
    chin_y = H - hh
    yg = youth(pr['hh'])
    eyes = np.array([J[B['eye_l']], J[B['eye_r']]])
    eye_r = (0.075 + 0.02 * yg) * hh
    srf = surface(family, stage)
    samples, normals, tris = srf['head_samples'], srf['head_normals'], srf['head_tris']
    # The cranium's ellipsoid: round the head above the ears (ears and face excluded).
    cran = srf['cranium']
    centre = np.array([0, chin_y + 0.56 * hh, float(cran[:, 2].min() + cran[:, 2].max()) / 2])
    radii = np.array([np.abs(cran[:, 0]).max(), H - centre[1], (cran[:, 2].max() - cran[:, 2].min()) / 2])
    k = np.linalg.norm((cran - centre) / radii, axis=1).max()
    radii = radii * max(k, 1.0) * np.array([1.05, 1.03, 1.12])
    h = HeadFrame(family, stage, hh, chin_y, H, centre, radii, 0.0, eyes, eye_r, eye_r * 0.5, eye_r * 0.2, np.zeros(3), 0.2 * hh,
                  samples=samples, normals=normals, tris=tris)
    a = np.linspace(0, 2 * np.pi, 8, endpoint=False)
    rim = eyes[1] + np.stack([np.cos(a), np.sin(a), np.zeros_like(a)], -1) * 1.45 * eye_r
    h.face_z = float(np.mean(h.surface_z(rim[:, 0], rim[:, 1])))
    my = chin_y + 0.2 * hh
    h.mouth = np.array([0, my, float(h.surface_z(0, my))])
    by = chin_y + 0.6 * hh
    for k, sx in enumerate((-1, 1)):
        inner = np.array([sx * 0.07 * hh, by, 0])
        outer = np.array([sx * 0.3 * hh, by - 0.02 * hh, 0])
        for e, pt in enumerate((inner, outer)):
            pt[2] = float(h.surface_z(pt[0], pt[1]))
            h.brows[k, e] = pt
    # Hairline loop: on the surface, at the hairline's height in each direction round the head.
    a = np.linspace(-np.pi, np.pi, 48, endpoint=False)
    dirs = np.stack([np.sin(a), np.zeros_like(a), np.cos(a)], -1)
    probe = centre + dirs * radii
    hy = hairline_y(h, probe)
    pts = []
    for d, y in zip(dirs, hy):
        pts.append(_ray(h, np.array([centre[0], y, centre[2]]), d))
    h.hairline = np.array(pts)
    h.chin = _ray(h, np.array([0, chin_y + 0.03 * hh, centre[2]]), np.array([0, -0.45, 1.0]) / np.hypot(0.45, 1))
    h.ears = srf['ears']
    return h


def _ray(h, o, d):
    """Where a ray from inside the head leaves its surface (bisection on the sdf)."""
    lo, hi = 0.0, 0.5 * h.top_y
    for _ in range(40):
        mid = (lo + hi) / 2
        if h.sdf(o + d * mid)[0] < 0:
            lo = mid
        else:
            hi = mid
    return o + d * lo

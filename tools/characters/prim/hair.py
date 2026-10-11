"""
The residents' hair, built from primitives over the head interface. Contract:

    build_hair(family, stage, head, joints) -> {'hair.<style>': mesh, ...}   # every style but `none`; `hair.beard` where the family has it
    mesh: pos, nor, idx (CCW out), dense (n, NB), fields, tone (y: shade deep in the hair, z: clump bands 0..1)

Every style starts from a skull cap: the scalp inside the hair's edge (the head interface's hairline,
smoothed round and brought a little lower over the forehead and temples), cast onto the head's
distance field, rounded part way out to the skull ellipsoid and pushed out by a thickness that
varies over the head (volume on top, thin at the edge). On it go lathe-like curtains (sheets round
the head with a thickness and a curled hem), fringes that follow the forehead as drawn, lofted lobes
and bent capsules, ellipsoid petals, tori and noise lumps. The beard is a shell over the jaw, chin
and upper lip with a hole round the lips; its morphs copy the lips' (face.follow_mouth). Each style
has one fixed layout, so the same calls give the same topology at every life stage.

Skinning: Head, blending below the jaw to neck_01 and spine_03 by height (tails and braids along
their length), so hair that falls past the jaw follows the shoulders when the head turns. Nothing
comes within 2.6 mm of the head.
"""

import numpy as np

from contract import BEARD_FAMILIES, HAIR_STYLES
from meshops import smooth_normals
from prims import ellipsoid, grid, loft, merge, noise3, orient, rings_mesh, rot_z, smoothstep, spline, torus
from rig import B, NB

TAU = 2 * np.pi
# Nearest the hair comes to the skull (m); the check wants more than 2 mm.
CLEAR = 0.0026
# How far the scalp the hair sits on is rounded out from the skull to its ellipsoid (0..1).
ROUND = 0.5


class Scalp:
    """Points on the skull by direction (azimuth a round the vertical axis, 0 = front, + = the
    resident's right; polar angle phi from the top, in the skull ellipsoid's normalised frame);
    `cranium`: no further out than the skull ellipsoid (else the face and jaw as drawn)."""

    def __init__(self, h, cranium=True):
        self.h = h
        self.cranium = cranium
        q = (h.hairline - h.centre) / h.radii
        a = np.arctan2(q[:, 0], q[:, 2])
        phi = np.arccos(np.clip(q[:, 1] / np.linalg.norm(q, axis=1), -1, 1))
        o = np.argsort(a)
        # The hair's own edge: the head's hairline smoothed round (no point at the nape) and brought
        # a little lower over the forehead and temples (fuller hair, a softer face).
        fa = np.linspace(-np.pi, np.pi, 180, endpoint=False)
        fp = np.interp(fa, a[o], phi[o], period=TAU)
        k = np.exp(-0.5 * (np.minimum(np.abs(fa), TAU - np.abs(fa)) / np.radians(16)) ** 2)
        k /= k.sum()
        fp = np.real(np.fft.ifft(np.fft.fft(fp) * np.fft.fft(np.fft.ifftshift(k))))
        t = np.abs(fa)
        fp += np.radians(5) * smoothstep(np.radians(70), 0, t) + np.radians(8) * np.exp(-((t - np.radians(70)) / np.radians(28)) ** 2)
        self.hl_a, self.hl_phi = fa, fp

    def hairline(self, a):
        """Polar angle of the hairline at azimuth a."""
        return np.interp(np.mod(np.asarray(a) + np.pi, TAU) - np.pi, self.hl_a, self.hl_phi, period=TAU)

    def dirs(self, a, phi):
        a, phi = np.broadcast_arrays(np.asarray(a, np.float64), np.asarray(phi, np.float64))
        return np.stack([np.sin(phi) * np.sin(a), np.cos(phi), np.sin(phi) * np.cos(a)], -1)

    def surface(self, a, phi):
        """Skull surface points and normals at (a, phi) (any shape; cast along the normalised direction)."""
        h = self.h
        d = self.dirs(a, phi)
        shape = d.shape[:-1]
        d = d.reshape(-1, 3) * h.radii
        # Over the ears no further out than the skull ellipsoid (which holds the cranium), so the
        # ears don't push the hair about; `_finish` keeps it off them.
        if self.cranium:
            # Past the ellipsoid only away from the ears (the brow and forehead may stand out of it).
            a_, phi_ = (v.ravel() for v in np.broadcast_arrays(np.asarray(a, np.float64), np.asarray(phi, np.float64)))
            aa = np.abs(np.mod(a_ + np.pi, TAU) - np.pi)
            ear = np.exp(-((aa - np.pi / 2) / 0.6) ** 2) * np.exp(-((phi_ - np.pi / 2) / 0.5) ** 2)
            hi = 1.0 + 0.6 * (1 - smoothstep(0.15, 0.5, ear))
        else:
            hi = np.full(len(d), 2.5)
        # March out to the first step outside (the field far from the head can't be trusted), then
        # bisect within that step.
        steps = np.linspace(0.2, 2.5, 47)
        out = np.stack([h.sdf(h.centre + d * s_) >= 0 for s_ in steps], 1) | (steps[None] >= hi[:, None])
        k = np.argmax(out, 1)
        hi = np.minimum(steps[k], hi)
        lo = steps[np.maximum(k - 1, 0)]
        for _ in range(20):
            mid = (lo + hi) / 2
            inside = h.sdf(h.centre + d * mid[:, None]) < 0
            lo = np.where(inside, mid, lo)
            hi = np.where(inside, hi, mid)
        t = (lo + hi) / 2
        if self.cranium:
            # Hair doesn't follow every bump of the skull: part way out to the smooth ellipsoid.
            t = t + ROUND * np.maximum(1 - t, 0)
        p = h.centre + d * t[:, None]
        n = (p - h.centre) / h.radii ** 2
        n /= np.linalg.norm(n, axis=1, keepdims=True)
        if not self.cranium:
            # The face's own normal (measured over a few mm: the field is rough), unless it is wild.
            g = _grad(h, p, 0.003)
            g = np.where((g * n).sum(1, keepdims=True) > 0.3, g, n)
            n = g + 0.5 * n
            n /= np.linalg.norm(n, axis=1, keepdims=True)
        return p.reshape(shape + (3,)), n.reshape(shape + (3,))

    def point(self, a, phi, off):
        p, n = self.surface(a, phi)
        return p + n * np.asarray(off)[..., None]


def _near_sdf(h, p, r=0.0004):
    """The least distance to the head (sdf) at and round points p (n, 3)."""
    d = h.sdf(p)
    for e in np.concatenate([np.eye(3), -np.eye(3)]) * r:
        d = np.minimum(d, h.sdf(p + e))
    return d


def _grad(h, p, e=1e-4):
    g = np.stack([(h.sdf(p + d) - h.sdf(p - d)) / (2 * e) for d in np.eye(3) * e], -1)
    return g / np.maximum(np.linalg.norm(g, axis=1, keepdims=True), 1e-9)


def _mesh(pos, idx, out=None, z=None, y=None, follow=None):
    """
    A component: faces turned to point along `out` (if given). `follow` (0..1 per vertex, optional):
    how far it follows the body rather than the head (default: by height, see `_finish`).
    """
    m = dict(pos=pos, idx=orient(pos, idx, out) if out is not None else idx)
    n = len(pos)
    m['f'] = np.full(n, -1.0) if follow is None else np.asarray(follow, np.float64)
    m['z'] = np.full(n, 0.5) if z is None else np.asarray(z, np.float64)
    m['y'] = np.zeros(n) if y is None else np.asarray(y, np.float64)
    return m


def cap(sc, thick, rows=9, cols=28, limit=None, edge=CLEAR, bands=None):
    """
    The skull cap: rows from the crown down to the hairline (`limit(a)` overrides it), pushed out by
    `thick(a, u)` (m; u = 0 at the crown .. 1 at the hairline), tapering to `edge` at the rim.
    """
    h = sc.h
    a = np.linspace(-np.pi, np.pi, cols, endpoint=False)
    lim = sc.hairline(a) if limit is None else limit(a)
    us = np.linspace(1, 1 / rows, rows)
    A, U = np.meshgrid(a, us)
    phi = U * lim[None]
    T = thick(A, U)
    T = T + (edge - T) * smoothstep(0.82, 1.0, U)
    P = sc.point(A, phi, np.maximum(T, edge))
    top = sc.point(np.zeros(1), np.zeros(1), thick(np.zeros(1), np.zeros(1)))[0]
    pos, idx, _ = rings_mesh(P, end=top)
    au = np.r_[A.ravel(), 0.0]
    uu = np.r_[U.ravel(), 0.0]
    z = bands(au, uu) if bands else 0.5 + 0.5 * np.sin(au * 13 + 2 * np.sin(uu * 3))
    return _mesh(pos, idx, pos - h.centre, z=z, y=0.25 * smoothstep(0.85, 1.0, uu))


def _ell(h, y):
    """Horizontal scale of the skull ellipsoid's section at height y (1 below its widest)."""
    q = (np.asarray(y) - h.centre[1]) / h.radii[1]
    return np.where(q > 0, np.sqrt(np.clip(1 - q * q, 0.0, 1)), 1.0)


def curtain(sc, a0, a1, top, bottom, off, cols=16, rows=7, flare=0.0, thick=0.008, curl=0.5, back=None, bands=13, wave=0.0, seed=0):
    """
    A sheet of hair hanging round the head between azimuths a0..a1, from height top(a) down to
    bottom(a) (m), `off` (m) out from the skull ellipsoid, flaring out by `flare` (fraction) towards
    the hem; `thick` (m) thick with a hem curling in by `curl`. `back(y)` (optional): the furthest
    forward the sheet's back may come at height y (so it clears the shoulders and back).
    """
    h = sc.h
    a = np.linspace(a0, a1, cols)
    T, Bt = np.broadcast_to(top(a) if callable(top) else top, a.shape), np.broadcast_to(bottom(a) if callable(bottom) else bottom, a.shape)
    us = np.linspace(0, 1, rows)
    layers = []
    for layer, (o_add, du) in enumerate(((0.0, 0.0), (-thick, 0.0))):
        P = []
        for u in us:
            y = T + (Bt - T) * u
            v = _ell(h, y)
            f = 1 + flare * u ** 1.5
            w = wave * np.sin(a * 7 + seed) * u
            rx = (h.radii[0] * v + off + o_add + w) * f
            rz = (h.radii[2] * v + off + o_add + w) * f
            p = np.stack([h.centre[0] + np.sin(a) * rx, y, h.centre[2] + np.cos(a) * rz], -1)
            if back is not None:
                p[:, 2] = np.minimum(p[:, 2], back(y) + (0 if layer == 0 else thick))
            P.append(p)
        layers.append(np.array(P))
    outer, inner = layers
    # Hem: the outer layer's last row curls in to the inner one.
    hem = outer[-1] + (inner[-1] - outer[-1]) * 0.5 + np.array([0, thick * curl, 0])
    Pall = np.concatenate([outer, hem[None] - [0, thick * 0.4, 0], inner[::-1]])
    pos, idx = grid(Pall)
    cen = np.array(h.centre) + 0 * pos
    cen[:, 1] = pos[:, 1]
    k = len(Pall)
    side = np.repeat(np.r_[np.ones(rows + 1), -np.ones(rows)], cols)
    out = (pos - cen) * side[:, None]
    z = 0.5 + 0.5 * np.sin(np.tile(a, k) * bands + noise3(pos * 40, seed) * 1.5)
    y = np.repeat(np.r_[np.zeros(rows), [0.2], np.full(rows, 0.55)], cols)
    return _mesh(pos, idx, out, z=z, y=y)


def fringe(face, a0, a1, top, bottom, off, cols=11, rows=4, thick=0.008, flare=0.0, bands=21, seed=0):
    """
    Hair falling over the forehead, on the face as drawn (`face`: a Scalp that isn't held to the
    cranium): between azimuths a0..a1, from polar angle top(a) (under the cap) down to height
    bottom(a) (m), `off` (m) off the skin (more by `flare` towards the hem), `thick` thick.
    """
    h = face.h
    a = np.linspace(a0, a1, cols)
    t0 = top(a)
    want = bottom(a)
    lo, hi = t0 + 0.02, np.full(cols, np.radians(150))
    for _ in range(24):
        mid = (lo + hi) / 2
        below = face.surface(a, mid)[0][:, 1] < want
        hi = np.where(below, mid, hi)
        lo = np.where(below, lo, mid)
    t1 = (lo + hi) / 2
    us = np.linspace(0, 1, rows)
    A = np.broadcast_to(a, (rows, cols))
    PH = t0[None] + (t1 - t0)[None] * us[:, None]
    o = off * (1 + flare * us[:, None] ** 1.5) * np.ones_like(PH)
    outer = face.point(A, PH, o)
    inner = face.point(A, PH, np.maximum(o - thick, CLEAR + 0.0005))[::-1]
    hem = (outer[-1] + inner[0]) / 2 + (outer[-1] - face.point(A[-2], PH[-2], o[-2])) * 0.25
    pos, idx = grid(np.concatenate([outer, hem[None], inner]))
    side = np.repeat(np.r_[np.ones(rows + 1), -np.ones(rows)], cols)
    out = (pos - h.centre) * side[:, None]
    z = 0.5 + 0.5 * np.sin(np.tile(a, 2 * rows + 1) * bands + seed)
    y = np.repeat(np.r_[np.zeros(rows), [0.2], np.full(rows, 0.55)], cols)
    return _mesh(pos, idx, out, z=z, y=y)


def petal(sc, a, phi, size, turn, lift, off, seed=0):
    """A flattened ellipsoid lying on the scalp at (a, phi): size (long, wide, thick) (m), its long
    axis turned `turn` from straight down the head, tipped up by `lift` (rad)."""
    p, n = sc.surface(np.array([a]), np.array([phi]))
    p, n = p[0], n[0]
    down = sc.point(np.array([a]), np.array([phi + 0.05]), 0)[0] - p
    down -= n * (down @ n)
    down /= np.linalg.norm(down)
    side = np.cross(n, down)
    t = down * np.cos(turn) + side * np.sin(turn)
    t = t * np.cos(lift) + n * np.sin(lift)
    s = np.cross(n, t)
    s /= np.linalg.norm(s)
    nn = np.cross(t, s)
    m = ellipsoid([0, 0, 0], [size[1], size[0], size[2]], 4, 7)
    R = np.stack([s, t, nn], 1)  # local x -> side, y -> along, z -> normal
    c = p + n * (off + size[2] * 0.6) + t * size[0] * 0.55
    pos = m['pos'] @ R.T + c
    z = np.clip(0.5 + 0.35 * np.sin(seed * 2.3) + 0.25 * np.cos(m['pos'][:, 0] / size[1] * 3), 0, 1)
    return _mesh(pos, m['idx'], pos - c, z=z, y=0.15 * (1 - m['t']))


def lobe(points, ry, rz, segs=8, hint=(0, 1, 0), bands=9, seed=0, y=0.0, hang=None):
    """A bent capsule (lofted along a smooth curve through `points`); `hang` (a, b): it follows the
    body more and more from a to b along its length (a tail)."""
    m = loft(points, ry, rz, segs, hint=hint, caps=2)
    z = 0.5 + 0.5 * np.sin(np.arange(len(m['pos'])) % segs / segs * TAU * 2 + seed + m['t'] * bands)
    follow = smoothstep(hang[0], hang[1], m['t']) if hang else None
    return _mesh(m['pos'], m['idx'], z=z, y=np.full(len(m['pos']), y), follow=follow)


def _lumps(m, sc, amount, scale, seed):
    """Noise lumps: vertices pushed out along their direction from the head's centre."""
    h = sc.h
    d = m['pos'] - h.centre
    d /= np.linalg.norm(d, axis=1, keepdims=True)
    k = noise3(m['pos'], seed, scale) + 0.5 * noise3(m['pos'], seed + 1, scale * 2.3)
    m['pos'] = m['pos'] + d * (amount * k)[:, None]
    m['z'] = np.clip(0.5 + 0.6 * k, 0, 1)
    return m


def build_hair(family, stage, head, P):
    h = head
    hh = h.hh
    sc = Scalp(h)
    skin = Scalp(h, cranium=False)
    brow_y = float(h.brows[:, :, 1].mean())
    styles = {}
    deg = np.radians
    neck_y = P['neck_01'][1]
    chin = h.chin_y
    # The body's back (m, z) at height y, roughly: behind the neck, then the shoulder blades.
    sp = P['spine_03']
    back = lambda y: np.interp(y, [sp[1] - 0.25 * hh, sp[1], neck_y, chin], [sp[2] - 0.5 * hh, sp[2] - 0.46 * hh, P['neck_01'][2] - 0.3 * hh, h.centre[2] - 0.2 * hh]) - 0.012

    def thick(top, side, back_, nape=None):
        """A thickness over the head: top at the crown, side over the ears, back_ behind."""
        def f(A, U):
            ca = np.cos(A)
            s = top + (side - top) * smoothstep(0.2, 0.9, U)
            b = back_ if nape is None else back_ + (nape - back_) * smoothstep(0.5, 1.0, U)
            return s + (b - s) * smoothstep(0.2, -0.8, ca) * smoothstep(0.1, 0.8, U)
        return f

    # Parted: volume on top, combed away from a side parting at x = +0.3 (the resident's right).
    base = thick(0.05 * hh, 0.03 * hh, 0.028 * hh, 0.014 * hh)

    def parted(A, U):
        t = base(A, U)
        x = np.sin(A) * np.sin(U * 1.1)
        crease = np.exp(-((x - 0.3) / 0.07) ** 2) * smoothstep(-0.2, 0.6, np.cos(A)) * smoothstep(0.1, 0.4, U)
        quiff = 0.022 * hh * np.exp(-(A / 0.6) ** 2) * smoothstep(0.35, 0.75, U) * smoothstep(1.0, 0.85, U)
        return t * (1 - 0.6 * crease) + quiff

    def part_bands(A, U):
        x = np.sin(A) * np.sin(U * 1.1)
        return 0.5 + 0.5 * np.sin((x - 0.3) * 40 * np.sign(x - 0.3 + 1e-6) + U * 4)

    styles['short'] = [cap(sc, parted, rows=10, cols=32, bands=part_bands)]

    # Crop: a thin cap, a squared hairline and a tiny fringe.
    def square(a):
        hl = sc.hairline(a)
        front = sc.hairline(np.zeros(1))[0] + deg(3)
        return np.where(np.abs(a) < deg(55), front + deg(4) * (np.abs(a) / deg(55)) ** 6, hl)

    styles['crop'] = [cap(sc, thick(0.022 * hh, 0.016 * hh, 0.016 * hh, 0.011 * hh), rows=8, cols=28, limit=square,
                          bands=lambda A, U: 0.5 + 0.5 * np.sin(A * 23 + U * 9))]

    # Layered: a cap and three flared tiers, the lowest to the jaw.
    lay = [cap(sc, thick(0.05 * hh, 0.04 * hh, 0.04 * hh), rows=9, cols=28)]
    for k, (yt, yb, off, fl) in enumerate(((0.72, 0.5, 0.04, 0.06), (0.58, 0.3, 0.033, 0.09), (0.44, 0.08, 0.026, 0.12))):
        span = deg(62 + 6 * k)
        lay.append(curtain(sc, span, TAU - span, chin + yt * hh, chin + yb * hh, off * hh, cols=15, rows=5, flare=fl, thick=0.01 * hh, bands=17, seed=k))
    styles['layered'] = lay

    # Tousled: a cap and seeded petals pointing every which way, a few over the forehead.
    rng = np.random.default_rng(11)
    pet = [cap(sc, thick(0.034 * hh, 0.028 * hh, 0.026 * hh, 0.016 * hh), rows=8, cols=24)]
    for k in range(22):
        a = rng.uniform(-np.pi, np.pi) if k > 3 else rng.uniform(-0.7, 0.7)
        lim = sc.hairline(np.array([a]))[0]
        phi = rng.uniform(0.15, 0.85) * lim if k > 3 else lim * rng.uniform(0.75, 0.9)
        size = np.array([rng.uniform(0.15, 0.21), rng.uniform(0.07, 0.095), rng.uniform(0.02, 0.026)]) * hh
        pet.append(petal(sc, a, phi, size, rng.uniform(-0.9, 0.9), rng.uniform(-0.12, 0.12), 0.008 * hh, seed=k))
    styles['tousled'] = pet

    # Side-swept: a cap and one fat lobe from a high side parting across the forehead.
    sw = [cap(sc, thick(0.045 * hh, 0.03 * hh, 0.028 * hh, 0.014 * hh), rows=9, cols=28)]
    fa = np.array([0.75, 0.35, 0.0, -0.4, -0.75, -1.05])
    fphi = sc.hairline(fa) * np.array([0.55, 0.78, 0.92, 0.98, 1.02, 1.08])
    pts = sc.point(fa, fphi, np.array([0.05, 0.05, 0.048, 0.045, 0.038, 0.028]) * hh)
    pts = spline(pts, 9)
    sw.append(lobe(pts, np.interp(np.linspace(0, 1, 9), [0, 0.4, 1], [0.05, 0.065, 0.03]) * hh,
                   np.interp(np.linspace(0, 1, 9), [0, 0.5, 1], [0.022, 0.028, 0.014]) * hh, segs=9, hint=(0, 1, 0.3)))
    styles['sideSwept'] = sw

    # Afro: a big rounded cap with noise lumps.
    def round_(A, U):
        # Out to a rounded shell 1.35 times the skull, a little up and back.
        e = sc.dirs(A, U * sc.hairline(A)) * h.radii
        e /= np.linalg.norm(e, axis=-1, keepdims=True)
        r_s = 1 / np.sqrt(((e / h.radii) ** 2).sum(-1))
        R = h.radii * [1.42, 1.3, 1.36]
        r_t = 1 / np.sqrt((np.abs(e / R) ** 2.4).sum(-1)) ** (1 / 1.2) + e @ (np.array([0, 0.1, -0.08]) * h.radii)
        return np.maximum(r_t - r_s, 0.03 * hh)

    afro = cap(sc, round_, rows=13, cols=34, edge=CLEAR + 0.004 * hh)
    styles['afro'] = [_lumps(afro, sc, 0.03 * hh, 7 / hh, 3)]

    # Bob: a shell to the jaw with a curled-in hem, and a straight fringe.
    bob = [cap(sc, thick(0.05 * hh, 0.04 * hh, 0.04 * hh), rows=9, cols=28)]
    bob.append(curtain(sc, deg(58), TAU - deg(58), chin + 0.72 * hh, chin + 0.06 * hh, 0.036 * hh, cols=19, rows=7, flare=0.12, thick=0.014 * hh, curl=1.2, bands=19))
    under = lambda a: sc.hairline(a) * 0.8
    bob.append(fringe(skin, -deg(64), deg(64), under, lambda a: brow_y + hh * (0.035 + 0.03 * (a / deg(64)) ** 2), 0.03 * hh, cols=13, rows=4, thick=0.012 * hh, flare=0.15, seed=2))
    styles['bob'] = bob

    # Long: a cap, the sides to the shoulders, the back to mid-back; a parted fringe framing the face.
    lng = [cap(sc, thick(0.045 * hh, 0.035 * hh, 0.035 * hh), rows=9, cols=28)]
    shoulder = P['clavicle_l'][1]
    side_bot = lambda a: np.interp(np.abs(np.mod(a + np.pi, TAU) - np.pi), [deg(70), deg(110), deg(150)], [shoulder + 0.25 * hh, shoulder - 0.1 * hh, sp[1] - 0.5 * hh])
    lng.append(curtain(sc, deg(66), TAU - deg(66), chin + 0.72 * hh, side_bot, 0.034 * hh, cols=21, rows=9, flare=0.18, thick=0.014 * hh, curl=0.8, back=back, bands=21, wave=0.004 * hh))
    for s in (-1, 1):
        a0, a1 = (deg(4), deg(70)) if s > 0 else (-deg(70), -deg(4))
        bot = lambda a, s=s: brow_y + hh * np.interp(np.abs(a), [deg(4), deg(35), deg(55), deg(72)], [0.2, 0.08, -0.1, -0.3])
        lng.append(fringe(skin, a0, a1, lambda a: sc.hairline(a) * 0.8, bot, 0.028 * hh, cols=7, rows=5, thick=0.012 * hh, flare=0.2, bands=23, seed=4 + s))
    styles['long'] = lng

    # Ponytail: a slicked cap, a band at the back of the head and a tapering tail.
    pt = [cap(sc, thick(0.024 * hh, 0.018 * hh, 0.02 * hh, 0.012 * hh), rows=9, cols=28, bands=lambda A, U: 0.5 + 0.5 * np.sin(A * 15))]
    tie = sc.point(np.array([np.pi]), np.array([deg(95)]), np.array([0.03 * hh]))[0]
    pt.append(_mesh(**_tor(torus(tie + [0, 0, -0.012 * hh], 0.05 * hh, 0.02 * hh, 12, 6, axis=(0, 0.25, -1)))))
    tail = np.array([tie + [0, 0, -0.03 * hh], tie + [0, -0.08 * hh, -0.13 * hh], tie + [0, -0.3 * hh, -0.16 * hh], tie + [0, -0.6 * hh, -0.1 * hh], tie + [0, -0.85 * hh, -0.07 * hh]])
    tail[:, 2] = np.minimum(tail[:, 2], back(tail[:, 1]) - 0.035 * hh)
    tp = spline(tail, 10)
    pt.append(lobe(tp, np.interp(np.linspace(0, 1, 10), [0, 0.25, 1], [0.08, 0.105, 0.025]) * hh, np.interp(np.linspace(0, 1, 10), [0, 0.25, 1], [0.07, 0.085, 0.02]) * hh, segs=10, hint=(1, 0, 0), bands=14, hang=(0.1, 0.9)))
    styles['ponytail'] = pt

    # Braid: a cap and a chain of nine overlapping lobes down the back.
    br = [cap(sc, thick(0.03 * hh, 0.024 * hh, 0.026 * hh, 0.018 * hh), rows=9, cols=28, bands=lambda A, U: 0.5 + 0.5 * np.sin(A * 15))]
    start = sc.point(np.array([np.pi]), np.array([deg(120)]), np.array([0.03 * hh]))[0]
    for k in range(9):
        y = start[1] - k * 0.095 * hh
        c = np.array([0.012 * hh * (-1) ** k, y, 0.0])
        c[2] = min(back(y) - 0.045 * hh, start[2] - 0.01 * hh * k)
        r = 0.07 * hh * (1 - 0.045 * k)
        e = ellipsoid(c, [r, r * 1.05, r * 0.75], 4, 8)
        pos = (e['pos'] - c) @ rot_z(0.45 * (-1) ** k).T + c
        br.append(_mesh(pos, e['idx'], pos - c, z=0.5 + 0.5 * np.sin((pos[:, 1] - c[1]) / r * 3 + k), y=0.3 * (1 - np.abs(pos[:, 1] - c[1]) / r), follow=np.full(len(pos), smoothstep(0, 6, k))))
    br.append(_with_follow(_mesh(**_tor(torus([0, start[1] - 9.1 * 0.095 * hh, back(start[1] - 9 * 0.095 * hh) - 0.045 * hh], 0.03 * hh, 0.012 * hh, 10, 5))), 1))
    styles['braid'] = br

    # Buns: a cap and two buns high on each side.
    bun = [cap(sc, thick(0.026 * hh, 0.02 * hh, 0.022 * hh, 0.014 * hh), rows=9, cols=28, bands=lambda A, U: 0.5 + 0.5 * np.sin(A * 15))]
    for s in (-1, 1):
        c = sc.point(np.array([s * deg(118)]), np.array([deg(38)]), np.array([0.13 * hh]))[0]
        e = ellipsoid(c, np.array([0.15, 0.14, 0.15]) * hh, 7, 12)
        m = _mesh(e['pos'], e['idx'], e['pos'] - c)
        bun.append(_lumps(m, sc, 0.008 * hh, 14 / hh, 7 + s))
    styles['bun'] = bun

    out = {}
    assert set(styles) | {'none'} == set(HAIR_STYLES)
    for name, comps in styles.items():
        out[f'hair.{name}'] = _finish(comps, h, P)
    if family in BEARD_FAMILIES:
        out['hair.beard'] = _finish([beard(skin)], h, P, head_only=True)
    return out


def _tor(m):
    return dict(pos=m['pos'], idx=m['idx'], z=0.3 + 0.2 * np.sin(np.arange(len(m['pos'])) * 0.7), y=np.full(len(m['pos']), 0.4))


def _with_follow(m, f):
    m['f'] = np.full(len(m['pos']), float(f))
    return m


def beard(sc):
    """A shell over the jaw, chin and upper lip with a hole round the lips (cells inside it collapse onto its edge)."""
    h = sc.h
    hh = h.hh
    cols, rows = 23, 12
    a = np.linspace(-np.radians(94), np.radians(94), cols)
    # Mouth direction (polar angle) and the hole round it.
    qm = (h.mouth - h.centre) / h.radii
    phm = np.arccos(qm[1] / np.linalg.norm(qm))
    top = np.interp(np.abs(a), np.radians([0, 22, 45, 70, 85, 94]), [phm - 0.17, phm - 0.15, phm - 0.02, phm - 0.1, np.radians(96), np.radians(94)])
    # The lower edge: just under the jaw, rising to the jaw's corners at the sides.
    want = h.chin_y + hh * np.interp(np.abs(a), np.radians([0, 45, 80, 94]), [-0.01, 0.02, 0.15, 0.22])
    lo, hi = top + 0.1, np.full(cols, np.radians(178))
    for _ in range(24):
        mid = (lo + hi) / 2
        below = sc.surface(a, mid)[0][:, 1] < want
        hi = np.where(below, mid, hi)
        lo = np.where(below, lo, mid)
    bot = (lo + hi) / 2
    A, U = np.meshgrid(a, np.linspace(0, 1, rows))
    phi = top[None] + (bot - top)[None] * U
    # The hole: an ellipse in (side distance, polar angle) round the mouth.
    wx = (h.mouth_w / 2 + 0.04 * hh) / h.radii[0]
    hx = np.sin(A) * np.sin(phi) / wx
    hy = (phi - phm) / np.where(phi < phm, 0.1, 0.13)
    r = np.hypot(hx, hy)
    inside = r < 1
    scale = np.where(inside, 1 / np.maximum(r, 1e-6), 1)
    A2 = np.where(inside, np.arcsin(np.clip(hx * scale * wx / np.maximum(np.sin(phi), 1e-3), -1, 1)), A)
    phi2 = np.where(inside, phm + hy * scale * np.where(phi < phm, 0.1, 0.13), phi)
    edge = np.minimum(smoothstep(1.0, 1.6, r), smoothstep(0, 0.2, U))
    edge = np.minimum(edge, smoothstep(np.radians(94), np.radians(78), np.abs(A)))
    off = CLEAR + (0.02 * hh) * edge
    P = sc.point(A2, phi2, off)
    pos, idx = grid(P)
    z = 0.5 + 0.5 * np.sin(A.ravel() * 26 + phi.ravel() * 8)
    return _mesh(pos, idx, pos - h.centre, z=z, y=0.2 * (1 - edge.ravel()))


def _finish(comps, h, P, head_only=False):
    m = merge(comps, keys=('pos', 'z', 'y', 'f'))
    pos = m['pos'].copy()
    # Nothing closer than CLEAR to the head: push such vertices out along the field's gradient
    # (measured a little round each point, so the clearance holds after the positions are rounded
    # in the files, wherever the field is rough).
    for _ in range(24):
        d = _near_sdf(h, pos)
        near = d < CLEAR
        if not near.any():
            break
        g = _grad(h, pos[near], 1e-3)
        out = pos[near] - h.centre
        out /= np.linalg.norm(out, axis=1, keepdims=True)
        g = np.where((g * out).sum(1, keepdims=True) > 0.2, g, out)
        pos[near] += g * np.clip(CLEAR + 0.0004 - d[near], 0.0003, 0.01)[:, None]
    idx = m['idx']
    n = len(pos)
    out = dict(pos=pos, idx=idx, nor=smooth_normals(pos, idx))
    d = np.zeros((n, NB))
    if head_only:
        d[:, B['Head']] = 1
    else:
        neck_y = P['neck_01'][1]
        a = smoothstep(h.chin_y + 0.05 * h.hh, neck_y, pos[:, 1])
        b = smoothstep(neck_y, P['spine_03'][1], pos[:, 1])
        own = m['f'] >= 0
        a = np.where(own, np.clip(m['f'] * 1.6, 0, 1), a)
        b = np.where(own, smoothstep(0.4, 1.0, m['f']), b)
        d[:, B['Head']] = 1 - a
        d[:, B['neck_01']] = a * (1 - b)
        d[:, B['spine_03']] = a * b
    out['dense'] = d
    f = np.zeros((n, 4))
    f[:, 0] = 0.1
    out['fields'] = f
    tone = np.zeros((n, 4))
    tone[:, 1] = np.clip(m['y'], 0, 1)
    tone[:, 2] = np.clip(m['z'], 0, 1)
    out['tone'] = tone
    return out

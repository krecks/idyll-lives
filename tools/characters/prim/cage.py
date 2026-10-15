"""
The residents' control cage: a closed polygon mesh of cross-section rings (superellipses) joined
by fixed patches at the shoulders, the crotch and the neck, plus the palm, fingers, thumb, shoe-like
feet and the head (head.py). `subdiv.py` fits the cage so its limit surface passes through the
ring points, then subdivides it once for the `body` part; the cage itself at its limit is the LOD.

    Cage.add(point, weights, region, **tags) -> vertex id
    Cage.ring(points, ...) -> ids; Cage.band(a, b) joins two rings with quads; Cage.join(a, b, match)
    joins rings of different sizes (some faces become pentagons).

Every life stage and both families build the same vertices in the same order, so the topology is
fixed and stages are deltas. Per vertex the cage carries its target point (on the designed
surface), its skin weights (bone -> weight), a region name and tags (`arm`: 0 torso .. 1 arm, used
by the sleeve field; `cav`: cavity darkening; `blush`), which subdivide like positions.

Torso rings have 12 vertices at 30 degree steps (0 = front, 90 = the resident's right, +x): the
arms leave through the 2 x 2 patches at 60..120 / 240..300 degrees of the chest rings T5-T7, the
legs through the halves of the hip ring T1 and the crotch vertex, the neck through the middle of
the shoulder cap. Limbs and the neck have 8-vertex rings, the palm 10, fingers and thumbs 4.
"""

import numpy as np

from rig import B, NB

TAGS = ('arm', 'cav', 'blush')


def se(t, p):
    """Superellipse coordinate: sign(t) |t|^(2 / p)."""
    return np.sign(t) * np.abs(t) ** (2.0 / p)


def unit(v):
    v = np.asarray(v, np.float64)
    return v / np.linalg.norm(v)


def turn(a, b):
    """The smallest rotation (3x3) taking unit vector `a` to unit vector `b` (not opposite)."""
    v, cs = np.cross(a, b), float(np.dot(a, b))
    K = np.array([[0, -v[2], v[1]], [v[2], 0, -v[0]], [-v[1], v[0], 0]])
    return np.eye(3) + K + K @ K / (1 + cs)


class Cage:
    def __init__(self):
        self.P = []
        self.D = []
        self.W = []
        self.region = []
        self.tags = {k: [] for k in TAGS}
        self.faces = []
        self.named = {}

    def add(self, p, w, region, name=None, disp=None, **tags):
        """A vertex: `p` on the smooth surface (fitted exactly), `disp` a feature offset added to
        the fitted cage as it is (a soft bump of about 0.45 x its size, no ringing round it)."""
        k = len(self.P)
        self.P.append(np.asarray(p, np.float64))
        self.D.append(np.zeros(3) if disp is None else np.asarray(disp, np.float64))
        self.W.append(dict(w))
        self.region.append(region)
        for t in TAGS:
            self.tags[t].append(float(tags.get(t, 0.0)))
        if name:
            self.named[name] = k
        return k

    def ring(self, pts, w, region, disp=None, **tags):
        """Vertices for `pts` (k, 3); `w`, `disp` and tags may be per vertex (lists) or shared."""
        ids = []
        for i, p in enumerate(pts):
            wi = w[i] if isinstance(w, (list, tuple)) else w
            ti = {k: (v[i] if isinstance(v, (list, tuple, np.ndarray)) else v) for k, v in tags.items()}
            ids.append(self.add(p, wi, region, disp=None if disp is None else disp[i], **ti))
        return ids

    def face(self, *ids):
        assert len(set(ids)) == len(ids), ids
        self.faces.append(list(ids))

    def band(self, a, b, skip=()):
        """Quads between rings a and b (same size, same direction, cyclic); `skip`: edge indices left open."""
        n = len(a)
        for i in range(n):
            if i in skip:
                continue
            j = (i + 1) % n
            self.face(a[i], a[j], b[j], b[i])

    def join(self, a, b, match):
        """Faces between ring a and a larger ring b: a[i] meets b[match[i]] (increasing, cyclic)."""
        n, m = len(a), len(b)
        for i in range(n):
            j = (i + 1) % n
            s, e = match[i], match[j]
            path = []
            k = e
            while True:
                path.append(b[k])
                if k == s:
                    break
                k = (k - 1) % m
            self.face(a[i], a[j], *path)

    def cap(self, ring):
        """Closes a 4-ring with one quad."""
        self.face(*ring)

    def arrays(self):
        n = len(self.P)
        dense = np.zeros((n, NB))
        for i, w in enumerate(self.W):
            for name, v in w.items():
                dense[i, B[name]] += v
        dense /= dense.sum(1, keepdims=True)
        tags = {k: np.array(v) for k, v in self.tags.items()}
        return np.array(self.P), np.array(self.D), dense, tags

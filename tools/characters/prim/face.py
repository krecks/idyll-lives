"""
The residents' faces: small feature shapes on a head that never deforms. Contract:

    build_face(family, stage, head) -> dict(
        parts={'eyes', 'lashes', 'brows', 'teeth', 'tongue': mesh},  # mesh: pos, nor, idx, dense, fields, tone
        body=[mesh, ...],         # components merged into `body` (lids, lips), skinned to Head, with their tone
        morphs={part: {channel: (dpos (n, 3), dnor (n, 3))}},  # 'body' indexes the merged components only
        follow=dict(pos, upper, morphs),  # the lip lines and their morphs (the beard copies them, `follow_mouth`)
    )

- Eyes: a sphere per eye on its eye bone, its pole to the front so the rings run round the iris.
  Fields xy = offset from the iris centre on the eye's front (m), z = iris radius, w = pupil radius;
  the shader draws sclera, iris (the resident's eye colour), pupil and a catchlight.
- Lids: upper and lower skin caps hugging the eyeball, their edges almond-shaped curves on it with a
  rounded rim, shaded like the face round them (normals leaning to it; build.py gives them the tone
  of the skin they lie on); lashes a dark band along the upper rim (a flick at the outer corner on
  women). Blinks meet a little below the middle; squints raise the lower lid's middle most.
- Brows: flattened capsules along the brow line, on the skin.
- Mouth: upper and lower lip tubes meeting at the corners (tone.x tints them); between them a dark
  plate (tone.y = 1) with the tongue on it, and the upper and lower teeth. Everything inside the
  mouth stays between the lips' inner lines, so a closed mouth shows nothing of it.

Every piece is made by one function of the face's parameters (`_pieces`), with a fixed vertex layout;
a channel is the pieces made with its parameters minus the pieces at rest, so units and visemes are
authored as shapes (`_unit_params`, `VISEME_PARAMS`), not as sums of deltas. Units are at ARKit
strength (face.ts uses them unscaled). Morphs are built at the base stage only and shared by the
other stages (the runtime adds them to every stage).
"""

import numpy as np

from contract import CHANNELS
from meshops import smooth_normals
from prims import ellipsoid, grid, loft, orient, rot_x, smoothstep
from rig import B, BASE_STAGE, NB

# The adult head height the millimetres below are measured on; other heads scale them.
ADULT_HH = 0.2723

# Per family: lip half-heights (upper, lower) and depth (mm), how far the lips stand off the skin,
# brow thickness, arch and taper, lash length and outer-corner flick, the lids' resting edges.
STYLE = {
    'male': dict(lip_u=2.4, lip_l=3.2, lip_z=2.5, proud=1.6, brow_h=3.2, brow_z=1.4, arch=1.0, brow_tip=0.5, lash=1.2, flick=0.3, upper=63.0, lower=121.0, tilt=0.0),
    'female': dict(lip_u=2.9, lip_l=3.8, lip_z=2.7, proud=1.8, brow_h=2.5, brow_z=1.1, arch=2.8, brow_tip=0.35, lash=2.0, flick=1.6, upper=61.0, lower=120.0, tilt=-3.0),
}

# Lid columns: azimuth round the eye (degrees, 0 = front, + towards the outer corner).
LID_THETA = np.linspace(-104, 104, 9)
LASH_THETA = np.linspace(-62, 80, 9)
# Lip columns along the mouth (-1 = the resident's left corner), denser at the corners.
LIP_S = np.sin(np.linspace(-np.pi / 2, np.pi / 2, 9))
TEETH_S = np.linspace(-0.62, 0.62, 9)
TONGUE_S = np.linspace(-0.6, 0.6, 7)

SIDES = ('Left', 'Right')
# How far the lower lip drops with the jaw fully open (mm).
JAW = 26.0


def _params(**kw):
    """Face parameters (all 0 at rest): per side pairs [left, right] or scalars (see `_pieces`)."""
    p = dict(
        blink=[0, 0], wide=[0, 0], squint=[0, 0], cheek=[0, 0], lookDown=[0, 0], lookUp=[0, 0],
        browDown=[0, 0], browInner=0, browOuter=[0, 0], sneer=[0, 0],
        smile=[0, 0], frown=[0, 0], dimple=[0, 0], stretch=[0, 0], press=[0, 0], lowDown=[0, 0], upUp=[0, 0],
        jaw=0, jawFwd=0, close=0, funnel=0, pucker=0, left=0, rollL=0, rollU=0, shrugL=0, shrugU=0,
        tuck=0, tongue=0,
    )
    for k, v in kw.items():
        assert k in p, k
        p[k] = [v, v] if isinstance(p[k], list) and not isinstance(v, (list, tuple)) else v
    return p


def _unit_params(c):
    """The parameters of ARKit unit `c`."""
    for k, s in enumerate(SIDES):
        if c.endswith(s) and c != 'mouthLeft':
            name, side = c[:-len(s)], k
            break
    else:
        name, side = c, None
    key = {
        'eyeBlink': 'blink', 'eyeWide': 'wide', 'eyeSquint': 'squint', 'cheekSquint': 'cheek',
        'eyeLookDown': 'lookDown', 'eyeLookUp': 'lookUp', 'browDown': 'browDown', 'browInnerUp': 'browInner',
        'browOuterUp': 'browOuter', 'noseSneer': 'sneer', 'mouthSmile': 'smile', 'mouthFrown': 'frown',
        'mouthDimple': 'dimple', 'mouthStretch': 'stretch', 'mouthPress': 'press', 'mouthLowerDown': 'lowDown',
        'mouthUpperUp': 'upUp', 'jawOpen': 'jaw', 'jawForward': 'jawFwd', 'mouthClose': 'close',
        'mouthFunnel': 'funnel', 'mouthPucker': 'pucker', 'mouthLeft': 'left', 'mouthRollLower': 'rollL',
        'mouthRollUpper': 'rollU', 'mouthShrugLower': 'shrugL', 'mouthShrugUpper': 'shrugU',
    }[name]
    if side is None:
        return _params(**{key: 1})
    v = [0, 0]
    v[side] = 1
    return _params(**{key: v})


# Visemes (the voice engine's order) as mouth shapes. Clear, slightly exaggerated shapes read at a
# distance: open vowels drop the jaw, the wide ones show the teeth, the round ones push the lips out.
VISEME_PARAMS = {
    'sil': {},
    'PP': dict(press=1, rollL=0.25, rollU=0.25, shrugL=0.3),
    'FF': dict(jaw=0.07, tuck=1, upUp=0.35),
    'TH': dict(jaw=0.16, tongue=1, upUp=0.15),
    'DD': dict(jaw=0.22, upUp=0.25, lowDown=0.15),
    'kk': dict(jaw=0.3, upUp=0.1),
    'CH': dict(jaw=0.12, funnel=0.65, upUp=0.4, lowDown=0.4),
    'SS': dict(jaw=0.05, stretch=0.5, upUp=0.45, lowDown=0.45),
    'nn': dict(jaw=0.16, upUp=0.15),
    'RR': dict(jaw=0.17, funnel=0.45),
    'aa': dict(jaw=0.62, upUp=0.1),
    'E': dict(jaw=0.34, stretch=0.45, upUp=0.25),
    'I': dict(jaw=0.2, stretch=0.6, upUp=0.35, lowDown=0.25),
    'O': dict(jaw=0.45, funnel=0.85),
    'U': dict(jaw=0.16, pucker=0.95),
}


class Face:
    """One face (a family at a stage): where its features are, and how they are shaped."""

    def __init__(self, family, head):
        self.h = head
        self.st = STYLE[family]
        self.mm = 0.001 * head.hh / ADULT_HH
        self.hw = head.mouth_w / 2 * 0.92
        self._fit_mouth()

    # --- the skin --------------------------------------------------------------------------------
    def surf(self, x, y):
        """z of the face's skin at (x, y) (refined on the head's distance field)."""
        h = self.h
        x, y = np.broadcast_arrays(np.asarray(x, np.float64), np.asarray(y, np.float64))
        z = np.asarray(h.surface_z(x, y), np.float64) * np.ones_like(x)
        p = np.stack([x.ravel(), y.ravel(), z.ravel()], -1)
        for _ in range(4):
            d = h.sdf(p)
            dz = (h.sdf(p + [0, 0, 1e-4]) - d) / 1e-4
            p[:, 2] -= d / np.clip(dz, 0.3, None)
        return p[:, 2].reshape(x.shape)

    def project(self, p, lift):
        """
        Points (n, 3) moved onto the head's surface along the skull ellipsoid's normal (smooth,
        whatever the head's tessellation), then `lift` (m, per point) out along it.
        """
        h = self.h
        p = np.array(p, np.float64)
        n = (p - h.centre) / h.radii ** 2
        n /= np.linalg.norm(n, axis=1, keepdims=True)
        lo, hi = np.full(len(p), -0.04), np.full(len(p), 0.04)
        for _ in range(30):
            mid = (lo + hi) / 2
            inside = h.sdf(p + n * mid[:, None]) < 0
            lo = np.where(inside, mid, lo)
            hi = np.where(inside, hi, mid)
        return p + n * ((lo + hi) / 2 + np.broadcast_to(lift, lo.shape))[:, None]

    def _fit_mouth(self):
        """
        A smooth stand-in for the skin round the mouth (a quadratic in x, y fitted to it), raised so
        the real skin is nowhere in front of it: the mouth's pieces are placed on it, so they are
        smooth however the head is tessellated and never sink into it.
        """
        h, mm, hw = self.h, self.mm, self.hw
        my = h.mouth[1]
        gx, gy = np.meshgrid(np.linspace(-1.35, 1.35, 15) * hw, my + np.linspace(-34, 12, 14) * mm)
        z = self.surf(gx, gy)
        self.m0 = np.array([h.mouth[0], my])
        F = self._feat(gx.ravel(), gy.ravel())
        self.mcoef = np.linalg.lstsq(F, z.ravel(), rcond=None)[0]
        self.mfloor = max(0.0, float((z.ravel() - F @ self.mcoef).max()))

    def _feat(self, x, y):
        u = (np.asarray(x) - self.m0[0]) / self.hw
        v = (np.asarray(y) - self.m0[1]) / self.hw
        return np.stack([np.ones_like(u), u * u, u ** 4, v, v * v, v ** 3, u * u * v], -1)

    def mz(self, x, y):
        """z of the smoothed skin round the mouth."""
        x, y = np.broadcast_arrays(np.asarray(x, np.float64), np.asarray(y, np.float64))
        return self._feat(x, y) @ self.mcoef + self.mfloor

    # --- eyes ------------------------------------------------------------------------------------
    def eyeballs(self):
        h = self.h
        out = []
        for k, side in enumerate('lr'):
            c = h.eyes[k]
            m = ellipsoid(c, [h.eye_r] * 3, 12, 16)
            p = (m['pos'] - c) @ rot_x(np.pi / 2).T + c
            f = np.zeros((len(p), 4))
            front = (p[:, 2] - c[2]) > 0
            f[:, 0] = np.where(front, p[:, 0] - c[0], 1.0)
            f[:, 1] = np.where(front, p[:, 1] - c[1], 1.0)
            f[:, 2] = h.iris_r * 1.08
            f[:, 3] = h.pupil_r * 1.1
            out.append(dict(pos=p, idx=m['idx'], nor=(p - c) / h.eye_r, fields=f, dense=_skin(len(p), f'eye_{side}')))
        return _cat(out)

    def lid_edges(self, p, k):
        """Upper and lower lid edges of eye k (polar angle from the top of the eye, degrees) per `LID_THETA`."""
        st = self.st
        t = to = LID_THETA / 90.0  # + towards the outer corner
        inner = smoothstep(0.1, -0.9, to)
        U0 = st['upper'] + 27 * t * t + st['tilt'] * to
        L0 = st['lower'] - 21 * t * t + 2.0 * to
        g = lambda name: p[name][k]
        U = U0 - 12 * g('wide') - 10 * g('lookUp') + 20 * g('lookDown') + g('browDown') * (3 + 12 * inner) + 5 * g('sneer') - 5 * p['browInner'] * inner
        L = L0 - 18 * g('squint') * (1 - 0.45 * t * t) - 8 * g('cheek') * (1 - 0.3 * t * t) + 8 * g('lookDown') + 3 * g('wide') - 7 * g('sneer')
        b = g('blink')
        if b:
            C = 0.52 * L0 + 0.48 * U0
            U = U + b * (C + 4 - U0)
            L = L + b * (C - L0)
        return U, L

    def _sph(self, k, phi, theta, r):
        """Points on eye k's sphere frame: polar angle phi from +y, azimuth theta (deg, + outer), radius r."""
        c = self.h.eyes[k]
        out = 1 if k == 1 else -1
        ph, th = np.radians(phi), np.radians(theta) * out
        return c + np.stack([np.sin(ph) * np.sin(th), np.cos(ph), np.sin(ph) * np.cos(th)], -1) * np.asarray(r)[..., None]

    def lids(self, p, k):
        h, mm = self.h, self.mm
        U, L = self.lid_edges(p, k)
        R = h.eye_r
        Ru, Rl = R * 1.105, R * 1.095
        n = len(LID_THETA)
        # Upper: from close over the top of the eye (so it rises out of the socket rather than
        # standing on the face) down to the edge, then the rim turning in to the eye.
        rows = [(np.full(n, 6.0), R * 1.05), (6 + (U - 6) * 0.45, R * 1.09), (6 + (U - 6) * 0.78, R * 1.1), (U, Ru), (U + 3.0, R * 1.075)]
        up = np.array([self._sph(k, ph, LID_THETA, np.full(n, r)) for ph, r in rows])
        # Lower: from under the eye up to the edge, then its rim.
        rows = [(np.full(n, 168.0), Rl), ((168 + L) / 2, Rl), (L, Rl), (L - 2.5, R * 1.075)]
        lo = np.array([self._sph(k, ph, LID_THETA, np.full(n, r)) for ph, r in rows])
        # Lashes: a dark band along the upper rim, standing out and up; longer towards the outer corner.
        st = self.st
        Ue = np.interp(LASH_THETA, LID_THETA, U)
        to = LASH_THETA / 90.0
        ln = (st['lash'] * (0.55 + 0.45 * smoothstep(-0.7, 0.6, to)) + st['flick'] * smoothstep(0.3, 0.95, to)) * mm
        lift = 5 + 6 * smoothstep(0.2, 0.95, to) * (st['flick'] > 1)
        rows = [(Ue + 1.5, Ru + 0.15 * mm), (Ue - 1.0, Ru + 0.55 * ln), (Ue - lift * 0.6 - 1, Ru + ln)]
        lash = np.array([self._sph(k, ph, LASH_THETA, r) for ph, r in rows])
        return up, lo, lash

    # --- brows -----------------------------------------------------------------------------------
    def brow(self, p, k):
        h, mm, st = self.h, self.mm, self.st
        a, b = h.brows[k]
        t = np.linspace(0, 1, 6)
        inward = 1 if k == 0 else -1  # towards the middle: +x for the left brow
        g = lambda name: p[name][k]
        inner = 11 * p['browInner'] + 1 * g('browOuter') - 8 * g('browDown') - 3 * g('sneer')
        mid = 5 * p['browInner'] + 4.5 * g('browOuter') - 4 * g('browDown') - 1.2 * g('sneer')
        outer = 1 * p['browInner'] + 9 * g('browOuter') - 1 * g('browDown')
        dy = np.interp(t, [0, 0.45, 1], [inner, mid, outer]) * mm + st['arch'] * mm * np.sin(np.pi * t ** 0.8)
        dx = (3.5 * g('browDown') + 1.2 * g('sneer')) * mm * (1 - t) ** 1.5 * inward
        x = a[0] + (b[0] - a[0]) * t + dx
        y = a[1] + (b[1] - a[1]) * t + dy
        ry = st['brow_h'] * mm * np.interp(t, [0, 0.25, 0.7, 1], [0.85, 1.0, 0.8, st['brow_tip']])
        rz = st['brow_z'] * mm * np.interp(t, [0, 1], [1, 0.7])
        z = self.surf(x, y)
        pts = self.project(np.stack([x, y, z], -1), rz * 0.75)
        return pts, ry, rz

    # --- mouth -----------------------------------------------------------------------------------
    def mouth(self, p):
        """Lip centre lines and radii, the inner lines, teeth, tongue (positions) for parameters p."""
        h, mm, st, hw = self.h, self.mm, self.st, self.hw
        s = LIP_S
        mL = smoothstep(0.35, -0.35, s)
        sv = lambda name: p[name][0] * mL + p[name][1] * (1 - mL)
        cw = s * s
        cen = np.clip(1 - s * s, 0, 1)
        o = cen ** 0.7
        rc = np.sqrt(cen)
        sm, fr, st_, dim = sv('smile'), sv('frown'), sv('stretch'), sv('dimple')
        x = s * hw * (1 - 0.34 * p['funnel'] - 0.45 * p['pucker']) + np.sign(s) * np.abs(s) ** 1.5 * mm * (3.5 * sm + 4 * st_ + 1.5 * dim - 1 * fr)
        x = x - 4 * mm * p['left']
        y = h.mouth[1] + mm * (cw * (0.6 + 6.5 * sm - 7 * fr - 1.8 * st_ + 0.8 * dim) + 2.2 * p['shrugL'] * cen)
        dz = -mm * cw * (2 * dim + 1 * sm)
        low = mm * (JAW * p['jaw'] * o + 4.5 * sv('lowDown') * o ** 0.6 + 5 * p['funnel'] * rc - 0.8 * JAW * p['close'] * o)
        up = mm * (3 * p['jaw'] * o + 4.5 * sv('upUp') * o ** 0.6 + 1.5 * p['shrugU'] * o + 2.5 * sv('sneer') * o ** 0.5 + 3 * p['funnel'] * rc + 1.6 * p['tuck'] * o)
        low = low - 1.2 * mm * p['tuck'] * o
        taper = 0.22 + 0.78 * cen ** 0.55
        ry_u = st['lip_u'] * mm * taper * (1 - 0.4 * sv('press') - 0.55 * p['rollU'] + 0.25 * p['pucker'] + 0.1 * p['funnel'])
        ry_l = st['lip_l'] * mm * taper * (1 - 0.4 * sv('press') - 0.55 * p['rollL'] + 0.25 * p['pucker'] + 0.1 * p['funnel'] + 0.15 * p['shrugL'] - 0.3 * p['tuck'])
        rz = st['lip_z'] * mm * (0.35 + 0.65 * rc) * (1 + 0.25 * p['pucker'])
        fwd = mm * rc * (4.5 * p['funnel'] + 5.5 * p['pucker'])
        proud = st['proud'] * mm * (0.3 + 0.7 * rc)
        yu = y + up + 0.75 * ry_u
        yl = y - low - 0.75 * ry_l
        zu = self.mz(x, yu) + proud + fwd + dz - 1.5 * mm * p['rollU'] * rc
        zl = self.mz(x, yl) + proud + fwd + dz + mm * rc * (3 * p['jawFwd'] - 2.5 * p['tuck'] - 1.5 * p['rollL'] + 1.0 * p['shrugL'])
        lips = (np.stack([x, yu, zu], -1), ry_u, rz), (np.stack([x, yl, zl], -1), ry_l, rz)
        # The opening: between the lips' inner lines (a little inside each lip, so its edge hides).
        up_in = y + up + 0.4 * ry_u
        low_in = y - low - 0.4 * ry_l
        v = np.linspace(0, 1, 4)[:, None]
        py = low_in[None] + (up_in - low_in)[None] * v
        px = np.broadcast_to(x, py.shape)
        # Everything inside sits on the plate's depth (which comes forward with pursed lips).
        pz = lambda xx, yy: self.mz(xx, yy) + 0.5 * mm + 0.4 * np.interp(xx, x, fwd)
        plate = np.stack([px, py, pz(px, py)], -1)
        clamp = lambda xx, yy: np.clip(yy, np.interp(xx, x, low_in), np.interp(xx, x, up_in))
        # Upper teeth: fixed to the skull; lower teeth: on the jaw.
        mx = h.mouth[1]
        ts = TEETH_S
        tx = ts * h.mouth_w / 2
        ub = mx - 0.8 * mm + 1.4 * mm * ts * ts
        drop = 0.86 * JAW * p['jaw'] * mm
        lt = mx - 1.4 * mm - drop + 0.6 * mm * ts * ts - 1.2 * mm * p['tuck']
        teeth = []
        for y0, y1, zf in ((ub, ub + 7 * mm, 0), (lt - 6 * mm, lt, 3 * p['jawFwd'] - 1.0 * p['tuck'])):
            rows = []
            for yy in (y0, y1):
                yy = clamp(tx, yy)
                rows.append(np.stack([tx, yy, pz(tx, yy) + (1.5 - 0.8 * ts * ts + zf) * mm], -1))
            teeth.append(np.array(rows))
        # Tongue: on the floor of the mouth, rising and coming forward between the teeth for TH.
        gs = TONGUE_S
        gx = gs * hw
        bot = np.interp(gx, x, low_in)
        top = mx - mm * (2.5 + 0.6 * JAW * p['jaw']) + mm * (9 * p['tongue']) - 3 * mm * (gs / 0.6) ** 2
        top = clamp(gx, top)
        rows = []
        tall = np.clip((top - bot) / (4 * mm), 0, 1)
        for f, bulge in ((0, 0.0), (0.55, 0.6), (1, 0.25)):
            yy = bot + (top - bot) * f
            rows.append(np.stack([gx, yy, pz(gx, yy) + (0.4 + (bulge + 2.2 * p['tongue'] * f) * tall) * mm], -1))
        tongue = np.array(rows)
        return lips, plate, teeth, tongue


def _lip_tube(line, ry, rz):
    # Open ends: the corners are thin enough not to show them.
    return loft(line, ry, rz, 6, hint=(0, 1, 0), caps=False)


def _pieces(face, p):
    """Every morphing piece of the face for parameters p: name -> (pos, idx)."""
    out = {}
    for k in range(2):
        up, lo, lash = face.lids(p, k)
        pos, idx = grid(up)
        out[f'lidU{k}'] = (pos, orient(pos, idx, pos - face.h.eyes[k]))
        pos, idx = grid(lo)
        out[f'lidL{k}'] = (pos, orient(pos, idx, pos - face.h.eyes[k]))
        pos, idx = grid(lash)
        out[f'lash{k}'] = (pos, orient(pos, idx, pos - face.h.eyes[k]))
        line, ry, rz = face.brow(p, k)
        m = loft(line, ry, rz, 6, hint=(0, 1, 0), caps=1)
        out[f'brow{k}'] = (m['pos'], m['idx'])
    lips, plate, teeth, tongue = face.mouth(p)
    for name, (line, ry, rz) in zip(('lipU', 'lipL'), lips):
        m = _lip_tube(line, ry, rz)
        out[name] = (m['pos'], m['idx'])
    fwd = lambda pos: np.tile([0, 0, 1.0], (len(pos), 1))
    for name, P in (('plate', plate), ('teethU', teeth[0]), ('teethL', teeth[1]), ('tongue', tongue)):
        pos, idx = grid(P)
        out[name] = (pos, orient(pos, idx, fwd(pos)))
    return out


# Which pieces make which part (in order).
PARTS = {
    'body': ['lidU0', 'lidL0', 'lidU1', 'lidL1', 'lipU', 'lipL'],
    'lashes': ['lash0', 'lash1'],
    'brows': ['brow0', 'brow1'],
    'tongue': ['plate', 'tongue'],
    'teeth': ['teethU', 'teethL'],
}


# The lids' shading normals lean towards the face's (they are skin round the eye, not a ball).
LID_FACING = {'lidU': np.array([0, 0.45, 1.0]), 'lidL': np.array([0, 0.35, 1.0])}


def _normals(name, pos, idx):
    n = smooth_normals(pos, idx)
    f = LID_FACING.get(name[:4])
    if f is not None:
        n = 0.2 * n + 0.8 * f / np.linalg.norm(f)
        n /= np.linalg.norm(n, axis=1, keepdims=True)
    return n


def _tone(name, pos, idx, face):
    """Per piece tone: lips blushed (x), the mouth plate dark (y), the tongue red, teeth shaded at the sides."""
    n = len(pos)
    t = np.zeros((n, 4))
    if name in ('lipU', 'lipL'):
        s = np.clip(np.abs(pos[:, 0] - face.h.mouth[0]) / face.hw, 0, 1)
        t[:, 0] = (0.75 if name == 'lipU' else 0.85) * (1 - 0.35 * s ** 3)
        # The sides facing the other lip in shade: a dark line between the lips, so the mouth's
        # shape (a smile, a frown) reads even when it is a few pixels wide.
        ny = smooth_normals(pos, idx)[:, 1] * (-1 if name == 'lipU' else 1)
        t[:, 1] = 0.75 * np.clip(ny, 0, 1) ** 1.5
    elif name == 'plate':
        t[:, 1] = 1
    elif name == 'tongue':
        t[:, 0] = 1
        t[:, 1] = np.repeat([0.45, 0.15, 0.05], len(TONGUE_S))
    elif name.startswith('teeth'):
        s = np.abs(pos[:, 0] - face.h.mouth[0]) / face.hw
        t[:, 1] = 0.05 + 0.5 * np.clip(s / 0.62, 0, 1) ** 2
    return t


def build_face(family, stage, head):
    h = head
    face = Face(family, h)
    rest = _pieces(face, _params())
    parts, body = {}, []
    parts['eyes'] = face.eyeballs()
    for part, names in PARTS.items():
        meshes = []
        for name in names:
            pos, idx = rest[name]
            m = dict(pos=pos, idx=idx, nor=_normals(name, pos, idx), dense=_skin(len(pos), 'Head'), tone=_tone(name, pos, idx, face))
            m['fields'] = np.zeros((len(pos), 4))
            if part == 'body':
                m['skin_tone'] = name.startswith('lid')
            meshes.append(m)
        if part == 'body':
            body.extend(meshes)
        else:
            parts[part] = _cat(meshes)
    out = dict(parts=parts, body=body, morphs={})
    if stage != BASE_STAGE:
        return out

    # Morphs: each channel's pieces minus the rest.
    def shape(p):
        return _pieces(face, p)

    rest_n = {k: _normals(k, *v) for k, v in rest.items()}

    def deltas(pieces, base=None):
        base = base or rest
        base_n = rest_n if base is rest else {k: _normals(k, *v) for k, v in base.items()}
        d = {}
        for part, names in PARTS.items():
            dp = np.concatenate([pieces[n][0] - base[n][0] for n in names])
            dn = np.concatenate([_normals(n, *pieces[n]) - base_n[n] for n in names])
            d[part] = (dp, dn)
        return d

    channels = {}
    for c in CHANNELS:
        if c in VISEME_PARAMS:
            channels[c] = deltas(shape(_params(**VISEME_PARAMS[c]))) if VISEME_PARAMS[c] else {}
        elif c == 'mouthClose':
            # A corrective (as in ARKit): the lips closing over an open jaw.
            channels[c] = deltas(shape(_params(jaw=1, close=1)), shape(_params(jaw=1)))
        else:
            channels[c] = deltas(shape(_unit_params(c)))
    morphs = {}
    for c in CHANNELS:
        for part, (dp, dn) in channels[c].items():
            if np.abs(dp).max() > 1e-6:
                morphs.setdefault(part, {})[c] = (dp, dn)
    out['morphs'] = morphs
    # The lip lines (the beard follows them).
    lip_n = [len(rest[n][0]) for n in PARTS['body']]
    k0 = sum(lip_n[:4])
    lips = np.concatenate([rest['lipU'][0], rest['lipL'][0]])
    upper = np.r_[np.ones(len(rest['lipU'][0]), bool), np.zeros(len(rest['lipL'][0]), bool)]
    follow = {}
    for c, (dp, dn) in morphs.get('body', {}).items():
        follow[c] = dp[k0:k0 + len(lips)]
    out['follow'] = dict(pos=lips, upper=upper, morphs=follow)
    return out


def follow_mouth(beard, follow, head):
    """
    The beard's morphs: each vertex copies the morphs of the lip vertices nearest it (the moustache
    the upper lip's, the rest the lower lip's), fading with distance, so it moves with the mouth.
    """
    from scipy.spatial import cKDTree
    pos = beard['pos']
    above = pos[:, 1] > head.mouth[1]
    fade = np.zeros(len(pos))
    near = []
    for sel, lip in ((above, follow['upper']), (~above, ~follow['upper'])):
        ids = np.where(lip)[0]
        d, k = cKDTree(follow['pos'][ids]).query(pos[sel], k=6)
        wt = 1 / (d + 1e-4)
        near.append((sel, ids[k], wt / wt.sum(1, keepdims=True)))
        fade[sel] = np.clip(1 - (d.min(1) - 0.015 * head.hh) / (0.32 * head.hh), 0, 1) ** 1.2
    out = {}
    for c, dp in follow['morphs'].items():
        acc = np.zeros_like(pos)
        for sel, nn, wt in near:
            acc[sel] = (dp[nn] * wt[..., None]).sum(1)
        acc *= fade[:, None]
        if np.abs(acc).max() > 1e-6:
            out[c] = (acc, np.zeros_like(pos))
    return out


def _skin(n, bone):
    d = np.zeros((n, NB))
    d[:, B[bone]] = 1
    return d


def _cat(meshes):
    out = {}
    n = 0
    idx = []
    for m in meshes:
        idx.append(m['idx'] + n)
        n += len(m['pos'])
    for k in meshes[0]:
        if k in ('idx', 't'):
            continue
        out[k] = np.concatenate([m[k] for m in meshes])
    out['idx'] = np.concatenate(idx).astype(np.int64)
    return out

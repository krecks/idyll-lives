"""
STUB face (stream B replaces this module): eyeballs, eyelid caps, lashes, brows, a mouth plate
and teeth on the head interface, with simple deltas for every morph channel. Contract:

    build_face(family, stage, head) -> dict(
        parts={'eyes', 'lashes', 'brows', 'teeth', 'tongue': mesh},  # mesh: pos, nor, idx, dense, fields, tone
        body=[mesh, ...],         # components merged into `body` (lids, lips), skinned to Head
        morphs={part: {channel: (dpos (n, 3), dnor (n, 3))}},  # 'body' indexes the merged components only
    )

Morphs are built at the base stage and shared by the other stages (they only move the face's own
small parts). Eye fields: xy = offset from the iris centre on the eye's front (m), z = iris
radius, w = pupil radius. The mouth plate (`tongue`) is dark through tone.y = 1.
"""

import numpy as np

from contract import CHANNELS
from prims import ellipsoid, loft, rings_mesh, rot_x
from rig import B, NB


def _skin(n, bone):
    d = np.zeros((n, NB))
    d[:, B[bone]] = 1
    return d


def _grid(rows):
    """Faces of a grid of points (r, c, 3); seen from the side where rows go down and columns go right they are clockwise (`_flip` turns them)."""
    r, c, _ = rows.shape
    idx = []
    for i in range(r - 1):
        for j in range(c - 1):
            a, b, cc, d = i * c + j, i * c + j + 1, (i + 1) * c + j + 1, (i + 1) * c + j
            idx += [a, b, cc, a, cc, d]
    return np.array(idx, np.int64)


def _flip(idx):
    t = idx.reshape(-1, 3)
    return t[:, [0, 2, 1]].ravel()


def build_face(family, stage, head):
    h = head
    hh = h.hh
    parts, body, morphs = {}, [], {}

    # Eyes: one sphere per eye, on its eye bone.
    eyes = []
    for k, side in enumerate('lr'):
        c = h.eyes[k]
        m = ellipsoid(c, [h.eye_r] * 3, 12, 16)
        # Turn the sphere's pole to the front so its rings are concentric round the iris.
        p = (m['pos'] - c) @ rot_x(np.pi / 2).T + c
        m['pos'] = p
        m['nor'] = (p - c) / h.eye_r
        f = np.zeros((len(p), 4))
        front = (p[:, 2] - c[2]) > 0
        f[:, 0] = np.where(front, p[:, 0] - c[0], 1.0)
        f[:, 1] = np.where(front, p[:, 1] - c[1], 1.0)
        f[:, 2] = h.iris_r
        f[:, 3] = h.pupil_r
        m['fields'] = f
        m['dense'] = _skin(len(p), f'eye_{side}')
        eyes.append(m)
    parts['eyes'] = _cat(eyes)

    # Lids: caps over the top and bottom of each eyeball (in the body's skin), rotated about the
    # eye's x axis by the blink / squint units; the lashes ride on the upper lid's rim.
    lids, lashes = [], []
    lid_meta, lash_meta = [], []
    for k in range(2):
        c = h.eyes[k]
        r = h.eye_r + 0.0015
        up = ellipsoid(c, [r] * 3, 4, 16, top=0.0, bottom=np.radians(62))
        lo = ellipsoid(c, [r] * 3, 3, 16, top=np.radians(128), bottom=np.pi)
        for m, kind in ((up, 'upper'), (lo, 'lower')):
            m['dense'] = _skin(len(m['pos']), 'Head')
            lids.append(m)
            lid_meta.append((k, kind, len(m['pos'])))
        # Lashes: a short dark band just outside the upper lid's front rim.
        a = np.radians(np.linspace(-70, 70, 9))
        rows = []
        for phi, rr in ((np.radians(57), r + 0.0006), (np.radians(64), r + 0.0022)):
            rows.append(c + np.stack([np.sin(phi) * np.sin(a), np.full_like(a, np.cos(phi)), np.sin(phi) * np.cos(a)], -1) * rr)
        rows = np.array(rows)
        m = dict(pos=rows.reshape(-1, 3), idx=_flip(_grid(rows)))
        m['nor'] = (m['pos'] - c) / np.linalg.norm(m['pos'] - c, axis=1, keepdims=True)
        m['dense'] = _skin(len(m['pos']), 'Head')
        lashes.append(m)
        lash_meta.append((k, len(m['pos'])))
    body.extend(lids)
    parts['lashes'] = _cat(lashes)

    # Brows: flattened capsules along the brow line.
    brows, brow_t = [], []
    for k in range(2):
        a, b = h.brows[k]
        pts = a + (b - a) * np.linspace(0, 1, 5)[:, None]
        pts[:, 2] = h.surface_z(pts[:, 0], pts[:, 1]) + 0.012 * hh
        m = loft(pts, 0.013 * hh * np.array([1, 1, 0.9, 0.8, 0.6]), 0.006 * hh, 6, hint=(0, 1, 0), caps=1)
        m['dense'] = _skin(len(m['pos']), 'Head')
        brows.append(m)
        brow_t.append(np.clip(m['t'], 0, 1))
    parts['brows'] = _cat(brows)

    # Mouth plate (dark) and teeth (white strips that open with the jaw).
    mc, hw = h.mouth, h.mouth_w / 2
    th = 0.0045 * hh
    ang = np.linspace(0, 2 * np.pi, 20, endpoint=False)
    ring = lambda s: np.stack([mc[0] + np.sin(ang) * hw * s, mc[1] + np.cos(ang) * th * s, np.zeros_like(ang)], -1)
    rings = np.array([ring(0.5), ring(1.0)])
    pos, idx, _ = rings_mesh(rings[::-1], end=mc * [1, 1, 0])
    idx = _flip(idx)
    pos[:, 2] = h.surface_z(pos[:, 0], pos[:, 1]) + 0.0015
    plate = dict(pos=pos, idx=idx, nor=np.tile([0, 0, 1.0], (len(pos), 1)))
    plate['dense'] = _skin(len(pos), 'Head')
    plate['tone'] = np.tile([0, 1.0, 0, 0], (len(pos), 1))
    parts['tongue'] = plate
    tx = np.linspace(-0.8, 0.8, 9) * hw
    teeth = []
    for y0 in (mc[1] + th * 0.6, mc[1] - th * 0.6):
        rows = np.array([np.stack([tx, np.full_like(tx, y0), np.zeros_like(tx)], -1)] * 2)
        p = rows.reshape(-1, 3).copy()
        p[:, 2] = h.surface_z(p[:, 0], p[:, 1]) + 0.0022
        teeth.append(dict(pos=p, idx=_flip(_grid(rows)), nor=np.tile([0, 0, 1.0], (len(p), 1))))
    tm = _cat([dict(m, dense=_skin(len(m['pos']), 'Head')) for m in teeth])
    parts['teeth'] = tm

    # --- morphs ------------------------------------------------------------------------------
    lid_pos = np.concatenate([m['pos'] for m in lids])
    lid_nor = np.concatenate([m['nor'] for m in lids])
    lash_pos, lash_nor = parts['lashes']['pos'], parts['lashes']['nor']

    def lid_rot(which, eye, deg):
        """Deltas of rotating lid `which` ('upper' / 'lower') of eye k by `deg` (+ = front down)."""
        dp, dn = np.zeros_like(lid_pos), np.zeros_like(lid_pos)
        lp, ln = np.zeros_like(lash_pos), np.zeros_like(lash_pos)
        R = rot_x(np.radians(deg))
        o = 0
        for k, kind, n in lid_meta:
            if k == eye and kind == which:
                c = h.eyes[k]
                dp[o:o + n] = (lid_pos[o:o + n] - c) @ R.T + c - lid_pos[o:o + n]
                dn[o:o + n] = lid_nor[o:o + n] @ R.T - lid_nor[o:o + n]
            o += n
        if which == 'upper':
            o = 0
            for k, n in lash_meta:
                if k == eye:
                    c = h.eyes[k]
                    lp[o:o + n] = (lash_pos[o:o + n] - c) @ R.T + c - lash_pos[o:o + n]
                    ln[o:o + n] = lash_nor[o:o + n] @ R.T - lash_nor[o:o + n]
                o += n
        return {'body': (dp, dn), 'lashes': (lp, ln)}

    bp = parts['brows']['pos']
    side_of_brow = np.concatenate([np.full(len(m['pos']), k) for k, m in enumerate(brows)])
    bt = np.concatenate(brow_t)

    def brow(eye, inner=0.0, outer=0.0, inward=0.0):
        d = np.zeros_like(bp)
        sel = side_of_brow == eye if eye is not None else np.ones(len(bp), bool)
        d[:, 1] = sel * (inner * (1 - bt) + outer * bt) * hh
        sx = np.where(side_of_brow == 0, 1, -1)  # towards the middle
        d[:, 0] = sel * inward * (1 - bt) * hh * sx
        return {'brows': (d, np.zeros_like(d))}

    pp, tp = plate['pos'], tm['pos']
    nplate = len(pp)

    def mouth(open_=0.0, round_=0.0, forward=0.0, shift=0.0, corner=None, up=0.0, out=0.0, back=0.0, upper=0.0, lower=0.0, squash=0.0, side=None):
        """Plate and teeth deltas (hh units) for a mouth movement; `side` 0 / 1 limits it to the left / right half."""
        out_d = {}
        for name, P in (('tongue', pp), ('teeth', tp)):
            x = (P[:, 0] - mc[0]) / hw
            below = P[:, 1] < mc[1] - 1e-6
            above = P[:, 1] > mc[1] + 1e-6
            if name == 'teeth':
                n0 = len(teeth[0]['pos'])
                upper_teeth = np.arange(len(P)) < n0
                bottom_row = (np.arange(len(P)) % (2 * 9)) >= 9
                below = ~upper_teeth
                above = upper_teeth & ~bottom_row
            d = np.zeros_like(P)
            sel = np.ones(len(P), bool) if side is None else ((x < 0) if side == 0 else (x > 0))
            env = np.sqrt(np.clip(1 - x * x, 0, 1))
            # Jaw: the lower half drops (an oval opening); the upper teeth show a little.
            if name == 'tongue':
                d[:, 1] -= below * open_ * hh * (0.3 + 0.7 * env)
            else:
                d[:, 1] -= below * open_ * hh * (0.75 + 0.2 * (np.arange(len(P)) % 18 >= 9))
                d[:, 1] -= (upper_teeth & bottom_row) * open_ * hh * 0.25
            d[:, 0] -= round_ * x * hw
            d[:, 1] += round_ * (P[:, 1] - mc[1]) * 1.2
            d[:, 2] += forward * hh * env
            d[:, 0] += shift * hh
            c = np.abs(x) ** 2
            d[:, 1] += c * up * hh
            d[:, 0] += c * np.sign(x) * out * hw
            d[:, 2] -= c * back * hh
            d[:, 1] += above * upper * hh + below * lower * hh
            d[:, 1] -= squash * (P[:, 1] - mc[1])
            d *= sel[:, None]
            out_d[name] = (d, np.zeros_like(d))
        return out_d

    U = {}
    for k, s in enumerate(('Left', 'Right')):
        U[f'eyeBlink{s}'] = lid_rot('upper', k, 66)
        U[f'eyeWide{s}'] = lid_rot('upper', k, -12)
        U[f'eyeSquint{s}'] = lid_rot('lower', k, -18)
        U[f'cheekSquint{s}'] = lid_rot('lower', k, -8)
        U[f'eyeLookDown{s}'] = _sum(lid_rot('upper', k, 20), lid_rot('lower', k, 8))
        U[f'eyeLookUp{s}'] = lid_rot('upper', k, -10)
        U[f'browOuterUp{s}'] = brow(k, outer=0.018)
        U[f'browDown{s}'] = brow(k, inner=-0.016, outer=-0.012, inward=0.007)
        U[f'mouthSmile{s}'] = mouth(up=0.018, out=0.08, side=k)
        U[f'mouthFrown{s}'] = mouth(up=-0.018, side=k)
        U[f'mouthDimple{s}'] = mouth(back=0.008, out=0.04, side=k)
        U[f'mouthStretch{s}'] = mouth(out=0.1, up=-0.008, side=k)
        U[f'mouthPress{s}'] = mouth(squash=0.6, side=k)
        U[f'mouthLowerDown{s}'] = mouth(lower=-0.012, side=k)
        U[f'mouthUpperUp{s}'] = mouth(upper=0.012, side=k)
        U[f'noseSneer{s}'] = _sum(brow(k, inner=-0.008), mouth(upper=0.004, side=k))
    U['browInnerUp'] = brow(None, inner=0.022)
    U['jawOpen'] = mouth(open_=0.066)
    U['jawForward'] = mouth(forward=0.011)
    U['mouthClose'] = mouth(squash=0.4)
    U['mouthFunnel'] = mouth(round_=0.3, forward=0.012, open_=0.012)
    U['mouthPucker'] = mouth(round_=0.42, forward=0.016)
    U['mouthLeft'] = mouth(shift=-0.022)
    U['mouthRollLower'] = mouth(lower=0.004, squash=0.3)
    U['mouthRollUpper'] = mouth(upper=-0.004, squash=0.3)
    U['mouthShrugLower'] = mouth(lower=0.007)
    U['mouthShrugUpper'] = mouth(upper=0.007)

    def mix(**w):
        out = {}
        for unit, a in w.items():
            parts_ = U[unit]
            for part, (dp, dn) in parts_.items():
                if part in out:
                    out[part] = (out[part][0] + dp * a, out[part][1] + dn * a)
                else:
                    out[part] = (dp * a, dn * a)
        return out

    stretch = dict(mouthStretchLeft=0.5, mouthStretchRight=0.5)
    V = {
        'sil': {},
        'PP': mix(mouthPressLeft=1, mouthPressRight=1, mouthRollLower=0.5, mouthClose=0.5),
        'FF': mix(mouthRollLower=0.8, jawOpen=0.1, mouthUpperUpLeft=0.3, mouthUpperUpRight=0.3),
        'TH': mix(jawOpen=0.15),
        'DD': mix(jawOpen=0.2),
        'kk': mix(jawOpen=0.3),
        'CH': mix(jawOpen=0.15, mouthFunnel=0.6),
        'SS': mix(jawOpen=0.08, **stretch),
        'nn': mix(jawOpen=0.15),
        'RR': mix(jawOpen=0.18, mouthFunnel=0.3),
        'aa': mix(jawOpen=0.6),
        'E': mix(jawOpen=0.35, **stretch),
        'I': mix(jawOpen=0.2, mouthStretchLeft=0.6, mouthStretchRight=0.6),
        'O': mix(jawOpen=0.45, mouthFunnel=0.7),
        'U': mix(jawOpen=0.25, mouthPucker=0.8),
    }
    channels = dict(V, **U)
    assert set(channels) == set(CHANNELS), set(CHANNELS) ^ set(channels)
    for c in CHANNELS:
        for part, (dp, dn) in channels[c].items():
            morphs.setdefault(part, {})[c] = (dp, dn)
    return dict(parts=parts, body=body, morphs=morphs)


def _sum(*ds):
    out = {}
    for d in ds:
        for k, (p, n) in d.items():
            out[k] = (out[k][0] + p, out[k][1] + n) if k in out else (p, n)
    return out


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

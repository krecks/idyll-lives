"""
STUB body (stream A replaces this module): rounded capsules per segment on the rig, skinned by
their position along the segment, with the clothes fields and tone. Same contract as the real one:

    build_body(family, stage, head) -> dict(
        parts={'body': mesh, 'lod.body': mesh},   # mesh: pos, nor, idx (CCW out), dense (n, NB), fields, tone
        meta={'seatDrop': m, 'eyeY': m, 'grip': [x, y, z]},
    )

The vertex layout is the same at every stage of a family (fixed counts), so stages are deltas.
`fields` are only read at the base stage (see garments.body_fields); `tone` likewise.
"""

import numpy as np

from garments import Landmarks, body_fields
from prims import ellipsoid, loft, merge, smoothstep
from rig import B, BONES, NB, joints, proportions


def _weights(n, pairs):
    """Dense (n, NB) weights from [(bone name, weight array)] (normalised later)."""
    d = np.zeros((n, NB))
    for name, w in pairs:
        d[:, B[name]] += w
    return d


def _limb(a, b, ra, rb, rings, segs, caps, own, parent=None, child=None, hint=(0, 0, 1), rz=None, split=(0.25, 0.8), power=2.0):
    pts = a + (b - a) * np.linspace(0, 1, rings)[:, None]
    rr = np.linspace(ra, rb, rings)
    m = loft(pts, rr, rr if rz is None else np.linspace(rz[0], rz[1], rings), segs, hint=hint, caps=caps, power=power)
    t = m['t']
    wp = 0.5 * smoothstep(split[0], 0, t) if parent else 0 * t
    wc = 0.5 * smoothstep(split[1], 1.05, t) if child else 0 * t
    pairs = [(own, 1 - wp - wc)]
    if parent:
        pairs.append((parent, wp))
    if child:
        pairs.append((child, wc))
    m['dense'] = _weights(len(t), pairs)
    return m


def build_body(family, stage, head, lod=False):
    """The body of `family` at `stage` (and its low-detail copy); see the module docstring."""
    pr = proportions(family, stage)
    H, g = pr['H'], pr['girth']
    J = joints(family, stage)
    P = {n: J[B[n]] for n in BONES}
    female = family == 'female'
    parts = {}
    for name, low in (('body', False), ('lod.body', True)):
        segs = 6 if low else 12
        caps = 1 if low else 2
        comps, arm = [], []

        def add(m, is_arm=False):
            comps.append(m)
            arm.append(np.full(len(m['pos']), 1.0 if is_arm else 0.0))

        # Torso: hips to the neck, superelliptic sections.
        thigh_r = (0.062 if female else 0.058) * H * g
        hip_half = pr['hip_w'] * H + thigh_r * 0.95
        sh = pr['shoulder'] * H
        waist_half = (0.072 if female else 0.083) * H * pr['waist']
        chest_half = sh * (0.8 if female else 0.86)
        neck_r = (0.03 if female else 0.034) * H * g ** 0.5
        bel = pr['belly'] * H
        y = lambda n: P[n][1]
        st = [
            (y('thigh_l') - 0.035 * H * g, hip_half * 0.55, 0.05 * H * g, 0.0),
            (y('thigh_l') + 0.012 * H, hip_half, (0.075 if female else 0.068) * H, -0.006 * H),
            ((y('pelvis') + y('spine_02')) / 2, waist_half, 0.062 * H + bel, 0.004 * H + bel * 0.5),
            (y('spine_02'), chest_half * 0.93, 0.068 * H + bel * 0.4, 0.004 * H),
            (y('spine_03'), chest_half, (0.08 if female else 0.074) * H, 0.004 * H),
            (y('upperarm_l') - 0.005 * H, sh * 0.96, 0.06 * H, -0.004 * H),
            (y('neck_01') - 0.004 * H, neck_r * 1.8, neck_r * 1.35, P['neck_01'][2]),
        ]
        k = 5 if low else 9
        ys = np.array([s[0] for s in st])
        yy = np.linspace(ys[0], ys[-1], k)
        hw = np.interp(yy, ys, [s[1] for s in st])
        hd = np.interp(yy, ys, [s[2] for s in st])
        zz = np.interp(yy, ys, [s[3] for s in st])
        torso = loft(np.stack([np.zeros(k), yy, zz], -1), hw, hd, 8 if low else 18, hint=(1, 0, 0), caps=caps, power=2.4)
        tp = torso['pos']
        spine = ['pelvis', 'spine_01', 'spine_02', 'spine_03']
        sy = np.array([y(n) for n in spine])
        ws = [np.interp(tp[:, 1], sy, np.eye(4)[i]) for i in range(4)]
        pairs = list(zip(spine, ws))
        ax = np.abs(tp[:, 0])
        for s, sx in (('l', -1), ('r', 1)):
            side = (tp[:, 0] * sx) > 0
            pairs.append((f'thigh_{s}', side * 0.5 * smoothstep(y('pelvis'), y('thigh_l') - 0.02 * H, tp[:, 1]) * smoothstep(0.2, 0.7, ax / hip_half)))
            pairs.append((f'clavicle_{s}', side * 0.5 * smoothstep(y('spine_03'), y('upperarm_l'), tp[:, 1]) * smoothstep(0.4, 0.9, ax / sh)))
        torso['dense'] = _weights(len(tp), pairs)
        add(torso)
        # Neck and head.
        add(_limb(P['neck_01'] - [0, 0.02 * H, 0], P['Head'] + [0, 0.035 * H, 0], neck_r, neck_r * 0.95, 3 if low else 4, segs, caps, 'neck_01', 'spine_03', 'Head', split=(0.3, 0.7)))
        hm = ellipsoid(head.centre, head.radii, 5 if low else 11, 8 if low else 18)
        hm['dense'] = _weights(len(hm['pos']), [('Head', np.ones(len(hm['pos'])))])
        add(hm)
        # Arms and hands.
        ar = (0.036 if female else 0.042) * H * g
        for s in 'lr':
            a = lambda n: P[f'{n}_{s}']
            rings = 3 if low else 6
            add(_limb(a('upperarm'), a('lowerarm'), ar, ar * 0.8, rings, segs, caps, f'upperarm_{s}', f'clavicle_{s}', f'lowerarm_{s}'), True)
            add(_limb(a('lowerarm'), a('hand'), ar * 0.78, ar * 0.58, rings, segs, caps, f'lowerarm_{s}', f'upperarm_{s}', f'hand_{s}'), True)
            hs = np.linalg.norm(a('fingers_01') - a('hand'))
            d = (a('fingers_01') - a('hand')) / hs
            hseg = 4 if low else 8
            add(_limb(a('hand'), a('fingers_01'), hs * 0.42, hs * 0.48, 3, hseg, 1, f'hand_{s}', f'lowerarm_{s}', f'fingers_01_{s}', rz=(hs * 0.2, hs * 0.17), split=(0.2, 0.95)), True)
            tip = a('fingers_01') + d * hs * 0.8
            add(_limb(a('fingers_01'), tip, hs * 0.44, hs * 0.3, 3, hseg, 1, f'fingers_01_{s}', None, f'fingers_02_{s}', rz=(hs * 0.15, hs * 0.12), split=(0, 0.45)), True)
            td = (a('thumb_02') - a('thumb_01'))
            add(_limb(a('thumb_01'), a('thumb_02') + td * 0.6, hs * 0.16, hs * 0.12, 3, 4 if low else 6, 1, f'thumb_01_{s}', f'hand_{s}', f'thumb_02_{s}', split=(0.15, 0.6)), True)
        # Legs and feet.
        for s in 'lr':
            a = lambda n: P[f'{n}_{s}']
            rings = 3 if low else 7
            add(_limb(a('thigh'), a('calf'), thigh_r, thigh_r * 0.68, rings, segs, caps, f'thigh_{s}', 'pelvis', f'calf_{s}', split=(0.15, 0.85)))
            add(_limb(a('calf'), a('foot'), thigh_r * 0.66, thigh_r * 0.46, rings, segs, caps, f'calf_{s}', f'thigh_{s}', f'foot_{s}', split=(0.15, 0.9)))
            fl = a('ball')[2] - a('foot')[2]
            heel = a('foot') + [0, -0.6 * a('foot')[1], -0.35 * fl]
            toe = a('ball') + [0, 0.0, 0.45 * fl]
            fy = np.array([heel[1], toe[1]])
            m = _limb(heel * [1, 0, 1] + [0, fy.mean(), 0], toe * [1, 0, 1] + [0, fy.mean(), 0], thigh_r * 0.5, thigh_r * 0.55, 3 if low else 5, 6 if low else 10, 1, f'foot_{s}', None, f'ball_{s}', hint=(1, 0, 0), rz=(thigh_r * 0.42, thigh_r * 0.3), split=(0, 0.75))
            m['pos'][:, 1] = np.maximum(m['pos'][:, 1], 0.002)
            add(m)
        mesh = merge(comps, keys=('pos', 'nor', 'dense'))
        mesh['arm'] = np.concatenate(arm)
        parts[name] = mesh
    meta = dict(
        seatDrop=float(P['pelvis'][1] - P['thigh_l'][1] + thigh_r * 0.85),
        eyeY=float(P['eye_l'][1]),
        grip=[float(v) for v in (P['fingers_01_l'] - P['hand_l']) * 0.55 + np.array([0.01, -0.01, 0]) * H],
    )
    return dict(parts=parts, meta=meta, joints=P)


def body_fields_and_tone(family, mesh, P, head):
    """Clothes fields and tone of a (base stage) body mesh."""
    lm = Landmarks(P, family)
    f = body_fields(mesh, lm, mesh['arm'])
    pos = mesh['pos']
    in_head = mesh['dense'][:, B['Head']] > 0.99
    scalp = head.scalp(pos)
    f[in_head, 2] = -1 - scalp[in_head]
    f[in_head, 3] = -1 - scalp[in_head]
    tone = np.zeros((len(pos), 4))
    # Cheek blush.
    for sx in (-1, 1):
        c = np.array([sx * 0.22 * head.hh, head.chin_y + 0.33 * head.hh, head.face_z - 0.02 * head.hh])
        tone[:, 0] = np.maximum(tone[:, 0], in_head * 0.35 * np.exp(-(np.linalg.norm(pos - c, axis=1) / (0.09 * head.hh)) ** 2))
    # Cavities: armpits, under the chin, between the fingers.
    for s in 'lr':
        c = P[f'upperarm_{s}'] + [0, -0.05, -0.01]
        tone[:, 1] = np.maximum(tone[:, 1], 0.45 * np.exp(-(np.linalg.norm(pos - c, axis=1) / 0.05) ** 2))
    tone[:, 1] = np.maximum(tone[:, 1], 0.3 * (mesh['dense'][:, B['neck_01']] > 0.3) * smoothstep(P['neck_01'][1], head.chin_y, pos[:, 1]))
    for s in 'lr':
        tone[:, 1] = np.maximum(tone[:, 1], 0.15 * (mesh['dense'][:, B[f'fingers_01_{s}']] + mesh['dense'][:, B[f'fingers_02_{s}']]))
    return f, tone

"""
The residents' bodies: a control cage of superellipse rings (cage.py) fitted so its limit surface
passes through the ring points, subdivided once (subdiv.py, about 3.5k triangles) for `body`; the
cage at its limit (about 0.9k triangles) is `lod.body`. Contract:

    build_body(family, stage, head) -> dict(
        parts={'body': mesh, 'lod.body': mesh},   # mesh: pos, nor, idx (CCW out), dense (n, NB), arm
        meta={'seatDrop': m, 'eyeY': m, 'grip': [x, y, z]},
        joints={bone: (3,)},
    )
    body_fields_and_tone(family, mesh, P, head) -> fields (n, 4), tone (n, 4)   # base stage only

Proportions come from rig.py (`PROPORTIONS`: height, head size, hips, shoulders, limb lengths and
girth, waist and belly) plus the shapes below, per family. Every stage of both families builds the
same cage, so the topology is fixed and stages are deltas.

Skin weights are set per ring (a ring's bone, blended 50/50 or so with the neighbour across a
joint; the shoulders blend spine_03, clavicle and upperarm, the hips pelvis and thigh, the neck
spine_03, neck_01 and Head) and subdivide with the positions, so they fall off smoothly.

Fields (docs/characters-format.md): x = sleeve parameter (-1 torso, 0 shoulder .. 1 wrist, > 1
hand: the arm's share of a vertex blends from the torso's -1 at the armhole), y = height, z / w =
the crew and scoop (male: V) necklines, metres below a smooth curve round the neck (> 0 covered);
on the head -1 - scalp weight. Tone: x = blush (cheeks, nose, ears), y = cavities (armpits, the
crotch, between the fingers, under the chin, the eye sockets; stronger on elders).
"""

from functools import lru_cache

import numpy as np

from cage import Cage, se, turn, unit
from head import head_cage, youth
from meshops import smooth_normals
from rig import B, BASE_STAGE, BONES, joints, proportions
from subdiv import check_closed, limit_matrix, orient, polygon_triangles, subdivision, triangles

SPINE = ['pelvis', 'spine_01', 'spine_02', 'spine_03']
REGIONS = ['torso', 'arm', 'hand', 'leg', 'foot', 'neck', 'head', 'ear']
TH12 = np.radians(np.arange(12) * 30.0)
# The arm's root loop on the chest rings (k: torso ring index): right side, then mirrored.
ARM_K = {'r': (2, 3, 4), 'l': (10, 9, 8)}
# The thighs' first rings keep this far (x thigh radius) from the midline, either side of the crotch.
CROTCH_GAP = 0.1
# Seated: the clips' seat (characters.ts SEATS) is 0.45 m; `seatDrop` puts the hips on it.
SEAT = 0.45


def mix(*pairs):
    """Weighted sum of bone -> weight dicts: mix((0.5, a), (0.5, b))."""
    out = {}
    for k, w in pairs:
        for n, v in w.items():
            out[n] = out.get(n, 0.0) + k * v
    return out


def _spine(P):
    """Bone weights along the spine by height (hat functions between the segments' middles)."""
    ys = np.array([P[n][1] for n in SPINE + ['neck_01']])
    mids = (ys[:-1] + ys[1:]) / 2

    def w(y):
        if y <= mids[0]:
            return {'pelvis': 1.0}
        if y >= mids[-1]:
            return {'spine_03': 1.0}
        i = int(np.searchsorted(mids, y)) - 1
        t = (y - mids[i]) / (mids[i + 1] - mids[i])
        return {SPINE[i]: 1 - t, SPINE[i + 1]: t}
    return w


def shape(family, stage):
    """The numbers the cage is built from (metres), per family and stage."""
    pr = proportions(family, stage)
    H, g = pr['H'], pr['girth']
    female = family == 'female'
    yg = youth(pr['hh'])
    S = dict(H=H, female=female, yg=yg, stage=stage)
    S['r_th'] = (0.058 if female else 0.056) * H * g * (1 + 0.08 * yg)
    S['r_a'] = (0.029 if female else 0.033) * H * g
    S['r_n'] = (0.031 if female else 0.035) * H * g ** 0.5
    S['waist'] = pr['waist']
    S['belly'] = pr['belly'] * H
    return S


def _ring(c, U, V, a, bf, bb, ang, p=2.2):
    """Ring points round centre c: angle 0 towards +V, 90 towards +U; depth bf in front (+V), bb behind."""
    s, co = np.sin(ang), np.cos(ang)
    return c + np.outer(a * se(s, p), U) + np.outer(np.where(co >= 0, bf, bb) * se(co, p), V)


def body_cage(family, stage):
    """The control cage (targets on the designed surface) of `family` at `stage`."""
    pr = proportions(family, stage)
    J = joints(family, stage)
    P = {n: J[B[n]] for n in BONES}
    S = shape(family, stage)
    H, yg, female = S['H'], S['yg'], S['female']
    r_th, r_a, r_n = S['r_th'], S['r_a'], S['r_n']
    bel = S['belly']
    sw = _spine(P)
    c = Cage()
    X, Y, Z = np.eye(3)
    y_hip = P['thigh_l'][1]
    y_s1, y_s2, y_s3 = P['spine_01'][1], P['spine_02'][1], P['spine_03'][1]
    y_neck = P['neck_01'][1]
    hip_x = P['thigh_r'][0]
    sh = P['upperarm_r']

    # Arm root loops (right, left): the 2 x 2 patch on the chest rings T5-T7 the arm leaves through.
    roots = {}
    for s, sx in (('r', 1), ('l', -1)):
        Jsh = P[f'upperarm_{s}']
        o = np.array([sx, 0, 0.0])
        # (chi: 135 180 225 | 270 | 315 0 45 | 90): bottom front .. round the loop.
        spec = [(-0.75, -1.5, 0.75), (-0.82, -1.95, 0.0), (-0.75, -1.5, -0.72), (-0.6, -0.75, -1.0),
                (-0.4, 0.05, -0.78), (-0.2, 0.25, 0.0), (-0.4, 0.05, 0.8), (-0.6, -0.75, 1.02)]
        roots[s] = [Jsh + (o * dx + Y * dy + Z * dz) * r_a for dx, dy, dz in spec]

    # Torso rings T1 (hips) .. T7 (shoulders); (y, half width, front depth, back depth, z, squareness).
    # T7 is all neck loop and arm loops (below).
    hw = hip_x + r_th * 0.85
    rows = [
        (y_hip + 0.012 * H, hw, 0.064 * H + bel * 0.3, (0.084 if female else 0.078) * H, -0.006 * H, 2.3),
        (y_hip + 0.55 * (y_s1 - y_hip), (0.106 if female else 0.095) * H * (1 + 0.1 * yg), 0.062 * H + bel * 0.7, (0.075 if female else 0.07) * H, 0.0, 2.3),
        ((y_s1 + y_s2) / 2, (0.068 if female else 0.08) * H * S['waist'] * (1 + 0.25 * yg), 0.062 * H + bel, 0.056 * H, 0.003 * H, 2.3),
        (y_s2 + 0.45 * (y_s3 - y_s2), (0.085 if female else 0.098) * H * (1 + 0.12 * yg), 0.068 * H + bel * 0.6, 0.060 * H, 0.002 * H, 2.2),
        (sh[1] - 1.95 * r_a, (0.092 if female else 0.108) * H, (0.07 if female else 0.074) * H + bel * 0.2, 0.064 * H, 0.0, 2.2),
        (sh[1] - 0.75 * r_a, (0.096 if female else 0.112) * H, 0.07 * H, 0.064 * H, -0.002 * H, 2.2),
        (sh[1] + 0.25 * r_a, (0.09 if female else 0.1) * H, 0.05 * H, 0.055 * H, -0.006 * H, 2.2),
    ]
    T = []
    for r, (y, a, bf, bb, zc, p) in enumerate(rows):
        pts = _ring(np.array([0, y, zc]), X, Z, a, bf, bb, TH12, p)
        T.append(pts)
    T = np.array(T)
    # Bust (female, gentle), shoulder blades, buttocks, belly.
    if female:
        T[4, [1, 11], 2] += 0.016 * H * (1 - yg)
        T[4, [1, 11], 1] -= 0.008 * H
        T[3, [1, 11], 2] += 0.005 * H * (1 - yg)
    T[5, [5, 7], 2] -= 0.006 * H
    T[0, [5, 7], 2] -= (0.008 if female else 0.004) * H
    T[1, [5, 7], 2] -= 0.004 * H
    T[2, 0, 2] += bel * 0.4
    # The arm loops replace the chest rings' sides.
    for s in 'lr':
        k0, k1, k2 = ARM_K[s]
        R = roots[s]
        T[4, k0], T[4, k1], T[4, k2] = R[0], R[1], R[2]
        T[5, k2], T[6, k2], T[6, k1], T[6, k0], T[5, k0] = R[3], R[4], R[5], R[6], R[7]
    # The shoulder ring's front and back meet the neck: they are the neck loop's.
    zn = P['neck_01'][2]
    neck_loop = {
        0: (0, y_neck - 0.016 * H, zn + r_n * 1.3),
        1: (r_n * 0.95, y_neck - 0.008 * H, zn + r_n * 0.95),
        5: (r_n * 0.95, y_neck + 0.006 * H, zn - r_n * 0.95),
        6: (0, y_neck + 0.002 * H, zn - r_n * 1.3),
    }
    for k, (x, y, z) in neck_loop.items():
        T[6, k] = (x, y, z)
        if k in (1, 5):
            T[6, 12 - k] = (-x, y, z)

    def tw(r, k):
        y = T[r, k, 1]
        w = sw(y)
        x = T[r, k, 0]
        side = 'r' if x > 0 else 'l'
        if r == 0:
            return mix((0.62, w), (0.38, {f'thigh_{side}': 1.0})) if abs(x) > 0.3 * hw else mix((0.76, w), (0.12, {'thigh_l': 1.0}), (0.12, {'thigh_r': 1.0}))
        if r == 1:
            return mix((0.9, w), (0.1, {f'thigh_{side}': 1.0})) if abs(x) > 0.3 * hw else w
        if r == 6:
            if k in (0, 6):
                return {'spine_03': 0.6, 'neck_01': 0.4}
            if k in (1, 5, 7, 11):
                return {'spine_03': 0.5, 'neck_01': 0.25, f'clavicle_{side}': 0.25}
        if r >= 4 and k in (1, 5, 7, 11):
            return mix((0.85, w), (0.15, {f'clavicle_{side}': 1.0}))
        return w

    def arm_w(chi_index, side):
        # Root loop entries: 0-2 bottom (armpit), 3 back, 4-6 top, 7 front.
        if chi_index in (0, 1, 2):
            return {'spine_03': 0.5, f'clavicle_{side}': 0.15, f'upperarm_{side}': 0.35}
        if chi_index in (3, 7):
            return {'spine_03': 0.35, f'clavicle_{side}': 0.3, f'upperarm_{side}': 0.35}
        return {'spine_03': 0.1, f'clavicle_{side}': 0.5, f'upperarm_{side}': 0.4}

    root_index = {}
    for s in 'lr':
        k0, k1, k2 = ARM_K[s]
        for i, (r, k) in enumerate([(4, k0), (4, k1), (4, k2), (5, k2), (6, k2), (6, k1), (6, k0), (5, k0)]):
            root_index[(r, k)] = (i, s)
    Tid = []
    for r in range(len(rows)):
        ids = []
        for k in range(12):
            if r == 5 and k in (3, 9):
                ids.append(None)
                continue
            key = (r, k)
            if key in root_index:
                i, s = root_index[key]
                cav = 0.55 if i == 1 else 0.25 if i in (0, 2) else 0.0
                ids.append(c.add(T[r, k], arm_w(i, s), 'torso', arm=0.5, cav=cav))
            else:
                ids.append(c.add(T[r, k], tw(r, k), 'torso'))
        Tid.append(ids)
    crotch = c.add((0, y_hip - (0.05 - 0.012 * yg) * H, -0.006 * H), {'pelvis': 0.4, 'thigh_l': 0.3, 'thigh_r': 0.3}, 'torso', cav=0.35)
    # Shoulder cap: the neck's sides.
    cap = {}
    for s, sx in (('r', 1), ('l', -1)):
        cap[s] = c.add((sx * r_n * 1.3, y_neck + 0.012 * H, zn - 0.1 * r_n), {'spine_03': 0.35, 'neck_01': 0.3, f'clavicle_{s}': 0.35}, 'torso')
    for r in range(len(rows) - 1):
        a, b = Tid[r], Tid[r + 1]
        for k in range(12):
            j = (k + 1) % 12
            if r in (4, 5) and k in (2, 3, 8, 9):
                continue
            c.face(a[k], a[j], b[j], b[k])
    t7 = Tid[6]
    c.face(t7[1], t7[2], t7[3], cap['r'])
    c.face(cap['r'], t7[3], t7[4], t7[5])
    c.face(t7[11], t7[10], t7[9], cap['l'])
    c.face(cap['l'], t7[9], t7[8], t7[7])

    # Neck: the shoulder cap's loop up to a ring under the jaw, then the head.
    n0 = [t7[0], t7[1], cap['r'], t7[5], t7[6], t7[7], cap['l'], t7[11]]
    c.neck_loop = n0
    pr_head = pr['hh'] * H
    chin = H - pr_head
    yN = chin - 0.01 * H + 0.005 * H * yg
    psi = np.radians(np.arange(8) * 45.0)
    nh = _ring(np.array([0, yN, zn - 0.2 * r_n]), X, Z, r_n, r_n * 0.95, r_n * 1.0, psi, 2.0)
    nh[[3, 4, 5], 1] += 0.012 * H   # the nape is higher than the throat
    nh_id = c.ring(nh, {'neck_01': 0.55, 'Head': 0.45}, 'neck', cav=[0.45, 0.25, 0, 0, 0, 0, 0, 0.25])
    c.band(n0, nh_id)
    jaw = head_cage(c, family, stage)
    c.join(nh_id, jaw, [0, 2, 4, 5, 6, 7, 8, 10])

    # Arms and hands.
    for s, sx in (('r', 1), ('l', -1)):
        k0, k1, k2 = ARM_K[s]
        loop = [Tid[4][k0], Tid[4][k1], Tid[4][k2], Tid[5][k2], Tid[6][k2], Tid[6][k1], Tid[6][k0], Tid[5][k0]]
        chi = np.radians([135, 180, 225, 270, 315, 0, 45, 90])
        Jsh, Jel, Jwr = P[f'upperarm_{s}'], P[f'lowerarm_{s}'], P[f'hand_{s}']
        d = unit(Jwr - Jsh)
        u = unit(Y - (Y @ d) * d)
        upper = np.linalg.norm(Jel - Jsh)
        lower = np.linalg.norm(Jwr - Jel)
        rings = [
            (Jsh + 0.32 * upper * d, 1.0, 0.98, {'upperarm_' + s: 0.9, 'clavicle_' + s: 0.1}),
            (Jel, 0.84, 0.84, {'upperarm_' + s: 0.5, 'lowerarm_' + s: 0.5}),
            (Jel + 0.45 * lower * d, 0.80, 0.86, {'lowerarm_' + s: 1.0}),
            (Jwr - 0.02 * lower * d, 0.5, 0.68, {'lowerarm_' + s: 0.45, 'hand_' + s: 0.55}),
        ]
        prev = loop
        for k, (cc, ru, rf, w) in enumerate(rings):
            pts = cc + np.outer(r_a * ru * se(np.cos(chi), 2.2), u) + np.outer(r_a * rf * se(np.sin(chi), 2.2), Z)
            cur = c.ring(pts, w, 'arm', arm=1.0)
            c.band(prev, cur)
            prev = cur
        wrist = prev
        # Palm: two 10-rings (4 across from the back of the hand (-z) to the thumb side, 1 deep).
        k1j = P[f'fingers_01_{s}']
        Lp = np.linalg.norm(k1j - Jwr)
        Wp, Tp = 0.80 * Lp, 0.30 * Lp
        zs = np.array([-0.5, -0.25, 0, 0.25, 0.5]) * Wp * np.array([0.9, 1, 1, 1, 0.92])
        order = [(3, -1), (2, -1), (1, -1), (0, -1), (0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (4, -1)]

        def palm_ring(t, scale, w, lift=0.0):
            cc = Jwr + t * Lp * d - u * 0.06 * Tp
            pts = [cc + zs[k] * scale * Z + u * (Tp / 2) * v * (0.85 if k in (0, 4) else 1.0) + d * lift * (k == 4) for k, v in order]
            return np.array(pts)

        p1 = c.ring(palm_ring(0.32, 0.92, None), {'hand_' + s: 1.0}, 'hand', arm=1.0)
        c.join(wrist, p1, [0, 1, 2, 3, 5, 6, 7, 8])
        p2_pts = palm_ring(0.9, 1.0, None)
        w2 = {'hand_' + s: 0.5, 'fingers_01_' + s: 0.5}
        p2 = c.ring(p2_pts, w2, 'hand', arm=1.0, cav=[0.0, 0.2, 0.2, 0.0, 0.0, 0.12, 0.12, 0.12, 0.0, 0.0])
        c.band(p1, p2, skip=(8,))
        at = {kv: i for i, kv in enumerate(order)}
        # Fingers: one per distal quad of the knuckle ring, curled a little towards the palm.
        Lf = 0.034 * H * (pr['upper'] / 0.165) ** 0.5
        for f in range(4):
            loop_f = [p2[at[(f, 1)]], p2[at[(f + 1, 1)]], p2[at[(f + 1, -1)]], p2[at[(f, -1)]]]
            base = p2_pts[[at[(f, 1)], at[(f + 1, 1)], at[(f + 1, -1)], at[(f, -1)]]]
            m = base.mean(0)
            spread = (f - 1.5) * 0.06
            L = Lf * (0.82, 0.95, 1.0, 0.93)[f]
            prev_f = loop_f
            for t, sc, curl, w in ((0.78, 0.98, 0.25, {'fingers_01_' + s: 0.3, 'fingers_02_' + s: 0.7}),):
                dirf = unit(d * np.cos(curl) - u * np.sin(curl) + Z * spread)
                cc = m + dirf * L * t
                pts = cc + (base - m) * sc - dirf * ((base - m) @ dirf)[:, None] * sc
                cur = c.ring(pts, w, 'hand', arm=1.0)
                c.band(prev_f, cur)
                prev_f = cur
            c.cap(prev_f)
        # Thumb: from the palm's thumb side between the two rings, a ring at its knuckle (thumb_02)
        # and one before the tip, narrowing like a finger. Each ring is the base quad turned from
        # facing forward to facing along the thumb (its corners keep their sides: wrist / tip of the
        # hand, back / palm), so the bands don't twist.
        tl = [p1[8], p1[9], p2[9], p2[8]]
        base = np.array([c.P[i] for i in tl])
        m = base.mean(0)
        side_d, side_u = np.sign((base - m) @ d), np.sign((base - m) @ u)
        t2 = P[f'thumb_02_{s}']
        tip = unit(0.8 * d - 0.35 * u + 0.4 * Z)
        prev_t = tl
        for cc, ax, hw, ht, w in (
            (t2, unit(unit(t2 - m) + tip), 0.15, 0.14, {'thumb_01_' + s: 0.5, 'thumb_02_' + s: 0.5}),
            (t2 + 0.42 * Lp * tip, tip, 0.11, 0.105, {'thumb_02_' + s: 1.0}),
        ):
            R = turn(Z, ax)
            cur = c.ring(cc + Lp * (np.outer(side_d * hw, R @ d) + np.outer(side_u * ht, R @ u)), w, 'hand', arm=1.0)
            c.band(prev_t, cur)
            prev_t = cur
        c.cap(prev_t)

    # Legs and feet.
    for s in 'rl':
        if s == 'r':
            loop = Tid[0][0:7] + [crotch]
            psi = np.radians([-45, 0, 45, 90, 135, 180, -135, -90])
        else:
            loop = Tid[0][6:12] + [Tid[0][0], crotch]
            psi = np.radians([135, 180, -135, -90, -45, 0, 45, 90])
        Jt, Jk, Ja = P[f'thigh_{s}'], P[f'calf_{s}'], P[f'foot_{s}']
        rings = [
            (Jt + 0.24 * (Jk - Jt), 1.0, 0.98, 1.06, {f'thigh_{s}': 0.92, 'pelvis': 0.08}, 0),
            (Jt + 0.86 * (Jk - Jt), 0.68, 0.70, 0.64, {f'thigh_{s}': 0.75, f'calf_{s}': 0.25}, 0),
            (Jk + 0.22 * (Ja - Jk), 0.64, 0.64, 0.74, {f'calf_{s}': 0.85, f'thigh_{s}': 0.15}, 0.1),
            (Ja + 0.1 * (Jk - Ja), 0.42, 0.42, 0.44, {f'calf_{s}': 0.5, f'foot_{s}': 0.5}, 0),
        ]
        prev = loop
        sx = 1 if s == 'r' else -1
        for k, (cc, a, bf, bb, w, cav) in enumerate(rings):
            cc = cc.copy()
            pts = _ring(cc, X, Z, r_th * a, r_th * bf, r_th * bb, psi, 2.1)
            if k == 0:
                # The thigh's top stops short of the midline (a baby's plump thighs would cross it):
                # the strip between it and the crotch would fold over when the legs spread.
                pts[:, 0] = sx * np.maximum(sx * pts[:, 0], CROTCH_GAP * r_th)
            back = np.cos(psi) < -0.5
            cur = c.ring(pts, w, 'leg', cav=list(back * cav))
            c.band(prev, cur)
            prev = cur
        # The foot: rings turning from the ankle to point forward, flat underneath, a round toe.
        Jb = P[f'ball_{s}']
        fl = Jb[2] - Ja[2]
        fw = 0.33 * fl * (1.0 if not female else 0.94)
        fx = Ja[0]
        feet = [
            (np.array([fx, 0.5 * Ja[1], Ja[2] - 0.02 * fl]), 45, fw * 0.92, 0.38 * fl, 0.38 * fl, {f'foot_{s}': 1.0}),
            (np.array([fx, 0.42 * Ja[1], Jb[2] - 0.2 * fl]), 86, fw * 1.0, 0.29 * fl, 0.30 * fl, {f'ball_{s}': 0.5, f'foot_{s}': 0.5}),
        ]
        for cc, al, a, bt, bb, w in feet:
            al = np.radians(al)
            V = np.array([0, np.sin(al), np.cos(al)])
            pts = _ring(cc, X, V, a, bt, bb, psi, 2.6)
            pts[:, 1] = np.maximum(pts[:, 1], 0.004 * H)
            cur = c.ring(pts, w, 'foot')
            c.band(prev, cur)
            prev = cur
        tip = c.add(np.array([fx, 0.4 * Ja[1], Jb[2] + 0.04 * fl]), {f'ball_{s}': 1.0}, 'foot')
        for i in (1, 3, 5, 7):
            c.face(prev[i], prev[(i + 1) % 8], prev[(i + 2) % 8], tip)
    return c, P


class Topology:
    """The cage's faces (oriented), its subdivision and limit operators (shared by every body)."""

    def __init__(self, cage):
        pos = np.array(cage.P)
        n = len(pos)
        faces = orient(cage.faces, pos)
        check_closed(faces, n)
        self.faces = faces
        self.n = n
        self.S = subdivision(faces, n)
        n1 = self.S.M.shape[0]
        L1 = limit_matrix(self.S.quads, n1)
        self.body = (L1 @ self.S.M).tocsr()            # cage -> level 1 on the limit surface
        self.A = self.body[:n].tocsc()                 # cage -> its own limit points
        self.lod_tris = polygon_triangles(faces)
        # A second level, only to sample the head densely (head_frame).
        self.S2 = subdivision([list(q) for q in self.S.quads], n1)
        L2 = limit_matrix(self.S2.quads, self.S2.M.shape[0])
        self.fine = (L2 @ self.S2.M @ self.S.M).tocsr()
        self.fine_M = (self.S2.M @ self.S.M).tocsr()


@lru_cache(maxsize=None)
def topology():
    cage, _ = body_cage('male', BASE_STAGE)
    return Topology(cage)


def _solve(A, T):
    from scipy.sparse.linalg import splu
    lu = splu(A)
    return np.column_stack([lu.solve(T[:, k]) for k in range(T.shape[1])])


@lru_cache(maxsize=None)
def surface(family, stage):
    """The body of `family` at `stage`: cage, level-1 mesh, LOD and the dense head samples."""
    topo = topology()
    cage, P = body_cage(family, stage)
    T, D, dense, tags = cage.arrays()
    assert len(T) == topo.n
    # The smooth shape is interpolated exactly; features are offsets on the fitted cage.
    ctrl = _solve(topo.A, T) + D
    S = topo.S
    pos = topo.body @ ctrl
    region = np.zeros((topo.n, len(REGIONS)))
    for i, r in enumerate(cage.region):
        region[i, REGIONS.index(r)] = 1
    out = dict(P=P, cage=cage, ctrl=ctrl, lod_pos=topo.A @ ctrl, lod_dense=dense, lod_tags=tags, lod_region=region)
    out['pos'] = pos
    out['dense'] = S.M @ dense
    out['tags'] = {k: S.M @ v for k, v in tags.items()}
    out['region'] = S.M @ region
    # Head and neck, densely, without the ears (for the head interface's sdf and surface_z).
    fpos = topo.fine @ ctrl
    # Normals from the finer (level-2) limit surface: smooth shading on the coarse level 1 (whose
    # vertices are the first of level 2's).
    fnor = smooth_normals(fpos, triangles(topo.S2.quads, fpos))
    out['nor'] = fnor[:len(pos)]
    out['lod_nor'] = fnor[:topo.n]
    freg = topo.fine_M @ region
    ear = freg[:, REGIONS.index('ear')] > 0.3
    ears = []
    for sx in (-1, 1):
        q = fpos[ear & (np.sign(fpos[:, 0]) == sx)]
        ears.append([(q.max(0) + q.min(0)) / 2, (q.max(0) - q.min(0)) / 2])
    out['ears'] = np.array(ears)
    flat = np.array([r == 'ear' for r in cage.region])
    fpos = topo.fine @ (ctrl - D * flat[:, None])
    hmask = (freg[:, REGIONS.index('head')] + freg[:, REGIONS.index('ear')] + freg[:, REGIONS.index('neck')]) > 0.5
    q = topo.S2.quads
    qm = hmask[q].all(1)
    tri = triangles(q[qm], fpos)
    nor = smooth_normals(fpos, triangles(q, fpos))
    used = np.unique(tri)
    # Samples: the vertices and the triangles' centres (about 4 mm apart on an adult head).
    cn = nor[tri].sum(1)
    cn /= np.linalg.norm(cn, axis=1, keepdims=True)
    out['head_samples'] = np.concatenate([fpos[used], fpos[tri].mean(1)])
    out['head_normals'] = np.concatenate([nor[used], cn])
    out['head_tris'] = fpos[tri]
    pr = proportions(family, stage)
    hh = pr['hh'] * pr['H']
    cran = (freg[:, REGIONS.index('head')] > 0.99) & (fpos[:, 1] > pr['H'] - hh + 0.6 * hh)
    out['cranium'] = fpos[cran]
    return out


def _tris(family):
    """Level-1 triangles: quads split along their shorter diagonal on the family's base body."""
    return triangles(topology().S.quads, surface(family, BASE_STAGE)['pos'])


def build_body(family, stage, head=None):
    """The body of `family` at `stage` (and its low-detail copy); see the module docstring."""
    s = surface(family, stage)
    topo = topology()
    P = s['P']
    idx = _tris(family).ravel()
    body = dict(pos=s['pos'], idx=idx, dense=s['dense'], arm=s['tags']['arm'])
    body['nor'] = s['nor']
    lod_idx = topo.lod_tris.ravel()
    lod = dict(pos=s['lod_pos'], idx=lod_idx, dense=s['lod_dense'], arm=s['lod_tags']['arm'])
    lod['nor'] = s['lod_nor']
    meta = dict(seatDrop=seat_drop(body, P), eyeY=float(P['eye_l'][1]), grip=grip(family, stage))
    return dict(parts={'body': body, 'lod.body': lod}, meta=meta, joints=P)


def _rx(a):
    c, s = np.cos(a), np.sin(a)
    return np.array([[1, 0, 0], [0, c, -s], [0, s, c]])


def seat_drop(body, P):
    """Seated (thighs forward, shins down): the pelvis joint's height above the lowest skin under the hips."""
    pos, dense = body['pos'], body['dense']
    up = _rx(np.radians(-90))
    down = _rx(np.radians(90))
    mats = {b: (np.eye(3), np.zeros(3)) for b in BONES}
    for s in 'lr':
        Jt, Jk = P[f'thigh_{s}'], P[f'calf_{s}']
        mats[f'thigh_{s}'] = (up, Jt - up @ Jt)
        knee = up @ Jk + (Jt - up @ Jt)
        R = up @ down
        for b in ('calf', 'foot', 'ball'):
            mats[f'{b}_{s}'] = (R, knee - R @ Jk)
    out = np.zeros_like(pos)
    for b, (R, t) in mats.items():
        w = dense[:, B[b]]
        if w.any():
            out += w[:, None] * (pos @ R.T + t)
    hips = sum(dense[:, B[n]] for n in ('pelvis', 'thigh_l', 'thigh_r', 'spine_01')) > 0.6
    return float(P['pelvis'][1] - out[hips, 1].min())


def grip(family, stage):
    """The left palm's centre (just off the skin) relative to the wrist."""
    s = surface(family, stage)
    P = s['P']
    wr, k1 = P['hand_l'], P['fingers_01_l']
    d = unit(k1 - wr)
    u = unit(np.array([0, 1.0, 0]) - d[1] * d)
    Lp = np.linalg.norm(k1 - wr)
    g = d * 0.62 * Lp - u * 0.28 * Lp
    return [float(v) for v in g]


# Fields and tone -------------------------------------------------------------------------------


def _fourier(theta, y, n=3):
    """A smooth even function of the angle round the neck through the samples (least squares)."""
    A = np.column_stack([np.cos(k * theta) for k in range(n)])
    coef, *_ = np.linalg.lstsq(A, y, rcond=None)
    return lambda t: np.column_stack([np.cos(k * t) for k in range(n)]) @ coef


def fields_of(family, pos, arm, region, P, head):
    """(n, 4) body fields of positions on the base body (see the module docstring)."""
    female = family == 'female'
    x, y, z = pos[:, 0], pos[:, 1], pos[:, 2]
    sh = P['upperarm_r']
    wr = P['hand_r']
    axis = wr - sh
    L = np.linalg.norm(axis)
    pa = np.stack([np.abs(x), y, z], -1)
    t = ((pa - sh) @ axis) / (L * L)
    sleeve = arm * t + (1 - arm) * -1.0
    # Necklines: cones round the neck (distance from its axis plus depth below a curve through
    # the shoulder cap's neck loop), so the edge crosses the sloping shoulders steeply and stays
    # clean on coarse triangles (a level cut would graze them).
    s = surface(family, BASE_STAGE)
    cage = s['cage']
    loop = np.array([cage.P[i] for i in cage.neck_loop])
    zn = P['neck_01'][2]
    rho_l = np.hypot(loop[:, 0], loop[:, 2] - zn)
    th_l = np.arctan2(loop[:, 0], loop[:, 2] - zn)
    rho0 = rho_l.mean()
    ref = _fourier(np.r_[th_l, -th_l], np.r_[loop[:, 1] - (rho_l - rho0), loop[:, 1] - (rho_l - rho0)])
    rho = np.hypot(x, z - zn)
    th = np.arctan2(x, z - zn)
    y0 = ref(th)
    crew = (rho - rho0) + (y0 - 0.008) - y
    front = np.clip(np.cos(th), 0, 1)
    if female:
        scoop = (rho - 1.2 * rho0) + (y0 - 0.012 - 0.07 * front ** 2) - y
    else:
        scoop = (rho - rho0) + (y0 - 0.01 - 0.085 * np.clip(1 - np.abs(th) / 0.85, 0, 1)) - y
    f = np.stack([sleeve, y, crew, scoop], -1)
    hd = region[:, REGIONS.index('head')] + region[:, REGIONS.index('ear')]
    scalp = head.scalp(pos) * (region[:, REGIONS.index('ear')] < 0.5)
    on_head = hd > 0.5
    f[on_head, 2] = -1 - scalp[on_head]
    f[on_head, 3] = -1 - scalp[on_head]
    return f


def body_fields_and_tone(family, mesh, P, head):
    """Clothes fields and tone of the (base stage) body or its LOD; the face's pieces after the skin."""
    s = surface(family, BASE_STAGE)
    n = len(mesh['pos'])
    lod = n < len(s['pos'])
    pos = s['lod_pos'] if lod else s['pos']
    k = len(pos)
    arm = (s['lod_tags'] if lod else s['tags'])['arm']
    region = s['lod_region'] if lod else s['region']
    tags = s['lod_tags'] if lod else s['tags']
    f = np.zeros((n, 4))
    f[:k] = fields_of(family, pos, arm, region, P, head)
    tone = np.zeros((n, 4))
    tone[:k, 0] = np.clip(tags['blush'], 0, 1)
    tone[:k, 1] = np.clip(tags['cav'], 0, 1)
    if n > k:
        # The face's pieces (lids, lips) inside the body: head fields; the nearest skin's tone.
        from scipy.spatial import cKDTree
        extra = mesh['pos'][k:]
        f[k:] = fields_of(family, extra, np.zeros(n - k), np.tile(np.eye(len(REGIONS))[REGIONS.index('head')], (n - k, 1)), P, head)
        _, nn = cKDTree(pos).query(extra)
        tone[k:] = tone[nn]
    return f, tone

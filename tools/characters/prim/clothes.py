"""
STUB clothes (stream A replaces this module). Clothes are painted on the body by the shader from
the body's fields and each garment's cut (`garments` in rig.json); shells are added only where the
silhouette needs them. Contract:

    garment_cuts(family, P) -> {'top.tee': {sleeve, hem, crew, ...}, ...}   # every id of contract.GARMENTS, in order
    build_clothes(family, stage, head, P, body) -> {'top.jacket': mesh, 'lod.top.jacket': mesh, ...}
    mesh: pos, nor, idx (CCW out), dense (n, NB), fields (hem distance, flag, category, 0), tone

Cut keys (metres on the base body's fields; see garments.py): top `sleeve`, `hem`, `crew` /
`scoop`, `over` (its hem is below the waistband: it is painted over the bottom); bottom `waist`,
`leg`, `skirt`; shoes `shoe` and `colour` (sRGB hex). Shells keep a fixed layout per family, so
stages are deltas. Categories: 0 top, 1 bottom, 2 shoes; flag 1 = sole.
"""

import numpy as np
from scipy.spatial import cKDTree

from contract import GARMENTS
from garments import BOTTOM, FLAG_SOLE, SHOES, TOP, Landmarks, skirt
from prims import loft, merge
from rig import B, NB

SHOE_COLOURS = {'sneakers': '#EDEBE6', 'trainers': '#3D4B66', 'boots': '#5B3B28', 'dress': '#221C19'}


def garment_cuts(family, P):
    lm = Landmarks(P, family)
    thigh = lm.hip - lm.knee
    shin = lm.knee - lm.ankle
    hem = lm.hem
    tops = {
        'tee': dict(sleeve=0.3, hem=hem, crew=0.0),
        'vneck': dict(sleeve=0.3, hem=hem, scoop=0.0),
        'polo': dict(sleeve=0.42, hem=hem - 0.02, crew=0.0),
        'long': dict(sleeve=0.955, hem=hem, crew=0.0),
        'shirt': dict(sleeve=0.955, hem=hem - 0.07, crew=-0.01, over=True),
        'jacket': dict(sleeve=0.97, hem=lm.hip - 0.03, crew=0.0, over=True),
        'suit': dict(sleeve=0.97, hem=lm.hip - 0.05, crew=-0.01, over=True),
        'tank': dict(sleeve=-0.55, hem=hem, scoop=0.0),
        'blouse': dict(sleeve=0.58, hem=hem, scoop=0.0),
    }
    bottoms = {
        'trousers': dict(waist=lm.waist, leg=lm.ankle + 0.03),
        'shorts': dict(waist=lm.waist, leg=lm.knee + 0.45 * thigh),
        'suit': dict(waist=lm.waist + 0.01, leg=lm.ankle + 0.02),
        'capri': dict(waist=lm.waist, leg=lm.knee - 0.45 * shin),
        'skirt': dict(waist=lm.waist, leg=lm.knee + 0.05, skirt=True),
    }
    shoes = {
        'sneakers': dict(shoe=lm.ankle + 0.03),
        'trainers': dict(shoe=lm.ankle + 0.03),
        'boots': dict(shoe=lm.ankle + 0.13),
        'dress': dict(shoe=lm.ankle + 0.01),
    }
    for k, v in shoes.items():
        v['colour'] = SHOE_COLOURS[k]
    table = {'top': tops, 'bottom': bottoms, 'shoes': shoes}
    out = {}
    for gid in GARMENTS[family]:
        kind, name = gid.split('.')
        out[gid] = {k: (round(float(v), 4) if isinstance(v, float) else v) for k, v in table[kind][name].items()}
    return out


def _follow(m, body, k=4):
    """Skin weights from the nearest body vertices."""
    d, nn = cKDTree(body['pos']).query(m['pos'], k=k)
    w = 1 / np.maximum(d, 1e-4)
    w /= w.sum(1, keepdims=True)
    m['dense'] = (body['dense'][nn] * w[..., None]).sum(1)
    return m


def _fields(m, cat, sole_y=None):
    n = len(m['pos'])
    f = np.zeros((n, 4))
    f[:, 0] = 0.05
    f[:, 2] = cat
    if sole_y is not None:
        f[:, 1] = (m['pos'][:, 1] < sole_y) * FLAG_SOLE
    m['fields'] = f
    m['tone'] = np.zeros((n, 4))
    return m


def _torso_shell(body, y0, y1, offset, rings=6, segs=18):
    """An open tube round the torso from y0 up to y1, `offset` off the skin."""
    torso = (body['arm'] == 0) & (body['dense'][:, B['Head']] < 0.5) & (body['dense'][:, B['neck_01']] < 0.5)
    legs = sum(body['dense'][:, B[f'{b}_{s}']] for b in ('thigh', 'calf', 'foot', 'ball') for s in 'lr') > 0.6
    P = body['pos'][torso & ~legs]
    ys = np.linspace(y0, y1, rings)
    rx, rz, zc = [], [], []
    for y in ys:
        band = P[np.argsort(np.abs(P[:, 1] - y))[:40]]
        rx.append(np.abs(band[:, 0]).max() + offset)
        lo, hi = band[:, 2].min(), band[:, 2].max()
        zc.append((lo + hi) / 2)
        rz.append((hi - lo) / 2 + offset)
    pts = np.stack([np.zeros(rings), ys, zc], -1)
    return loft(pts, np.array(rx), np.array(rz), segs, hint=(1, 0, 0), caps=0, power=2.4)


def build_clothes(family, stage, head, P, body, cuts):
    lm = Landmarks(P, family)
    H = head.top_y
    out = {}
    sh = P['upperarm_l'][1]
    # Jacket / suit: a shell over the torso (the sleeves are painted).
    for gid in ('top.jacket', 'top.suit'):
        if gid not in cuts:
            continue
        hem = cuts[gid]['hem'] * (H / _base_height(family))
        m = _torso_shell(body, max(hem, P['thigh_l'][1] - 0.06 * H), sh - 0.01 * H, 0.008 + 0.002 * (gid == 'top.suit'))
        out[gid] = _fields(_follow(m, body), TOP)
    if 'bottom.skirt' in cuts:
        # (Only the torso and legs shape it, not the hands hanging beside the hips.)
        sk = skirt(dict(pos=body['pos'][body['arm'] == 0]), P, lm, B, NB)
        sk['dense'] = np.zeros((len(sk['pos']), NB))
        _dense_from(sk)
        sk['idx'] = sk['idx'].astype(np.int64)
        out['bottom.skirt'] = _fields(sk, BOTTOM)
    # Shoes: a rounded block round each foot; boots add a shaft.
    for gid in [g for g in cuts if g.startswith('shoes.')]:
        comps = []
        for s in 'lr':
            fl = P[f'ball_{s}'][2] - P[f'foot_{s}'][2]
            r = (P[f'ball_{s}'] - P[f'foot_{s}'])
            heel = P[f'foot_{s}'] * [1, 0, 1] + [0, 0.03 * H, -0.42 * fl]
            toe = P[f'ball_{s}'] * [1, 0, 1] + [0, 0.025 * H, 0.55 * fl]
            w = 0.034 * H * (P['thigh_l'][1] / (0.5 * H)) ** 0.3
            m = loft(np.array([heel, (heel + toe) / 2, toe]), w * np.array([0.92, 1.05, 0.9]), w * np.array([0.9, 0.8, 0.62]), 12, hint=(1, 0, 0), caps=2)
            m['pos'][:, 1] = np.maximum(m['pos'][:, 1], 0.0)
            comps.append(m)
            if gid == 'shoes.boots':
                top = P[f'foot_{s}'][1] + 0.13 * (H / _base_height(family))
                shaft = loft(np.array([P[f'foot_{s}'] + [0, -0.01, 0], P[f'foot_{s}'] * [1, 0, 1] + [0, top, 0]]), w * 0.85, w * 0.85, 12, caps=0)
                comps.append(shaft)
            del r
        m = merge(comps)
        out[gid] = _fields(_follow(m, body), SHOES, sole_y=0.012 * H / 1.7 + 0.008)
    for k in list(out):
        out[f'lod.{k}'] = dict(out[k])
    return out


def _dense_from(sk):
    j, w = sk['joints'], sk['weights']
    for s in range(j.shape[1]):
        np.add.at(sk['dense'], (np.arange(len(j)), j[:, s]), w[:, s])


def _base_height(family):
    from rig import BASE_STAGE, proportions
    return proportions(family, BASE_STAGE)['H']

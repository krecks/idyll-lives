"""
The residents' clothes. They are painted on the body by its shader from the body's fields and each
garment's cut (`garments` in rig.json, see docs/characters-format.md); shells are added only where
the silhouette needs them:

- `top.jacket`, `top.suit`: the body's own surface over the region the cut covers (torso and
  sleeves), cut exactly along the fields' iso-lines, lifted 8-10 mm (more towards the hem, which
  flares a little), open in a V at the front (the lapels), with a rim turned back to the skin;
- `top.polo`, `top.shirt`: a turned-down collar round the neckline;
- `bottom.skirt`: an A-line loft from the waistband to above the knees, swinging with the thighs;
- `shoes.*`: a sole under each foot (darker), and a cuff round the shin at the top of `boots`.

Contract:

    garment_cuts(family, P) -> {'top.tee': {sleeve, hem, crew, ...}, ...}   # every id of contract.GARMENTS, in order
    build_clothes(family, stage, head, P, body, cuts) -> {'top.jacket': mesh, 'lod.top.jacket': mesh, ...}
    mesh: pos, nor, idx (CCW out), dense (n, NB), fields (hem distance, flag, category, 0), tone

Cut keys (metres on the base body's fields): top `sleeve` (sleeve parameter), `hem`, `crew` /
`scoop`, `over` (its hem is below the waistband: painted over the bottom); bottom `waist`, `leg`,
`skirt`; shoes `shoe` and `colour` (sRGB hex). Each shell is cut and laid out on the base body, so
it has the same vertices at every life stage (stages are deltas).
"""

from functools import lru_cache

import numpy as np
from scipy.spatial import cKDTree

from body import REGIONS, fields_of, surface
from contract import GARMENTS
from garments import ARM_M, BOTTOM, FLAG_SKIRT, FLAG_SOLE, SHOES, TOP, Landmarks, smoothstep
from head import head_frame
from meshops import boundary_edges, clip_by_field, smooth_normals
from rig import B, BASE_STAGE, NB, proportions

SHOE_COLOURS = {'sneakers': '#EDEBE6', 'trainers': '#3D4B66', 'boots': '#5B3B28', 'dress': '#221C19'}
# Shells: lift off the skin (m) and the rim's depth.
JACKET_LIFT = {'top.jacket': 0.009, 'top.suit': 0.008}
# The jacket shells' sleeves end (sleeve parameter) on the upper arm; the rest is painted.
SHELL_SLEEVE = 0.42
SKIRT_SEGS, SKIRT_ROWS = 24, 7


def garment_cuts(family, P):
    lm = Landmarks(P, family)
    thigh = lm.hip - lm.knee
    shin = lm.knee - lm.ankle
    hem = lm.hem
    tops = {
        'tee': dict(sleeve=0.3, hem=hem, crew=0.0),
        'vneck': dict(sleeve=0.3, hem=hem, scoop=0.0),
        'polo': dict(sleeve=0.36, hem=hem - 0.02, crew=-0.006),
        'long': dict(sleeve=0.965, hem=hem, crew=0.0),
        'shirt': dict(sleeve=0.965, hem=hem - 0.07, crew=-0.012, over=True),
        'jacket': dict(sleeve=0.975, hem=lm.hip + 0.025, crew=0.0, over=True),
        'suit': dict(sleeve=0.975, hem=lm.hip + 0.015, crew=-0.012, over=True),
        'tank': dict(sleeve=-0.62, hem=hem, scoop=0.0),
        'blouse': dict(sleeve=0.5, hem=hem, scoop=0.0),
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
        'trainers': dict(shoe=lm.ankle + 0.035),
        'boots': dict(shoe=lm.ankle + 0.13),
        'dress': dict(shoe=lm.ankle + 0.012),
    }
    for k, v in shoes.items():
        v['colour'] = SHOE_COLOURS[k]
    table = {'top': tops, 'bottom': bottoms, 'shoes': shoes}
    out = {}
    for gid in GARMENTS[family]:
        kind, name = gid.split('.')
        out[gid] = {k: (round(float(v), 4) if isinstance(v, float) else v) for k, v in table[kind][name].items()}
    return out


def top_coverage(f, cut):
    """Signed coverage (m, > 0 covered) of a top's cut over body fields (as the shader paints it)."""
    g = np.minimum((cut['sleeve'] - f[:, 0]) * ARM_M, f[:, 1] - cut['hem'])
    if 'crew' in cut:
        g = np.minimum(g, f[:, 2] - cut['crew'])
    if 'scoop' in cut:
        g = np.minimum(g, f[:, 3] - cut['scoop'])
    return g


@lru_cache(maxsize=None)
def _base_fields(family):
    s = surface(family, BASE_STAGE)
    h = head_frame(family, BASE_STAGE)
    f = fields_of(family, s['pos'], s['tags']['arm'], s['region'], s['P'], h)
    fl = fields_of(family, s['lod_pos'], s['lod_tags']['arm'], s['lod_region'], s['P'], h)
    return f, fl


def _shell_dense(src, dense):
    A, Bv, T = src
    return dense[A] * (1 - T)[:, None] + dense[Bv] * T[:, None]


@lru_cache(maxsize=None)
def _jacket_layout(family, gid, lod):
    """The jacket's piece of the base body: clipped faces and their source edges (stage independent)."""
    from body import _tris, topology
    s = surface(family, BASE_STAGE)
    f, fl = _base_fields(family)
    cuts = garment_cuts(family, s['P'])
    cut = cuts[gid]
    if lod:
        pos, idx, ff = s['lod_pos'], topology().lod_tris.ravel(), fl
    else:
        pos, idx, ff = s['pos'], _tris(family).ravel(), f
    g = top_coverage(ff, cut)
    # The lapels: open in a V at the front, from the neckline down to the button.
    P = s['P']
    zn = P['neck_01'][2]
    th = np.arctan2(pos[:, 0], pos[:, 2] - zn)
    y_button = P['spine_02'][1] + (0.02 if gid == 'top.suit' else 0.0)
    y_top = P['neck_01'][1] - 0.01
    open_w = np.radians(46)
    vline = y_button + (y_top - y_button) * np.clip(np.abs(th) / open_w, 0, 1) ** 1.2
    v = vline - pos[:, 1]
    g = np.minimum(g, v)
    # The shell's sleeves stop on the upper arm, sinking into the painted sleeve.
    g = np.minimum(g, (SHELL_SLEEVE - ff[:, 0]) * ARM_M)
    part = dict(pos=pos, nor=np.zeros_like(pos), uv=np.zeros((len(pos), 2)), idx=idx)
    cl = clip_by_field(part, g, np.zeros((len(pos), 1)))
    A, Bv, T = cl['src']
    hem_y = (ff[A, 1] * (1 - T) + ff[Bv, 1] * T)
    sleeve = (ff[A, 0] * (1 - T) + ff[Bv, 0] * T)
    return dict(src=cl['src'], idx=cl['idx'], hem_y=hem_y, sleeve=sleeve, cut=cut)


def _rim(pos, nor, idx, depth, skip=None):
    """A band turned back from each open edge towards the skin (the cloth's thickness); edges with
    both ends in `skip` get none."""
    be = boundary_edges(idx)
    if skip is not None:
        be = be[~(skip[be[:, 0]] & skip[be[:, 1]])]
    verts = np.unique(be)
    new = {v: len(pos) + i for i, v in enumerate(verts)}
    rp = pos[verts] - nor[verts] * depth
    faces = []
    for a, b in be:
        faces += [b, a, new[a], b, new[a], new[b]]
    return np.concatenate([pos, rp]), np.concatenate([idx, np.array(faces, np.int64)]), verts


def jacket(family, stage, gid, lod=False):
    lay = _jacket_layout(family, gid, lod)
    s = surface(family, stage)
    from body import _tris, topology
    if lod:
        bpos, bidx, dense = s['lod_pos'], topology().lod_tris.ravel(), s['lod_dense']
    else:
        bpos, bidx, dense = s['pos'], _tris(family).ravel(), s['dense']
    bnor = smooth_normals(bpos, bidx)
    A, Bv, T = lay['src']
    pos = bpos[A] * (1 - T)[:, None] + bpos[Bv] * T[:, None]
    nor = bnor[A] * (1 - T)[:, None] + bnor[Bv] * T[:, None]
    nor /= np.linalg.norm(nor, axis=1, keepdims=True)
    cut = lay['cut']
    H = proportions(family, stage)['H']
    k = H / 1.7
    # Lift: thicker over the hips (the flare of the hem) and a little looser on the sleeves.
    hem = cut['hem']
    flare = smoothstep(hem + 0.14, hem, lay['hem_y'])
    taper = smoothstep(SHELL_SLEEVE, SHELL_SLEEVE - 0.24, lay['sleeve'])
    lift = (JACKET_LIFT[gid] + 0.012 * flare) * k * taper
    pos = pos + nor * lift[:, None]
    cuts = garment_cuts(family, surface(family, BASE_STAGE)['P'])
    if 'bottom.skirt' in cuts:
        # The hem lies over a skirt (it is a shell too).
        pos = _over_skirt(pos, family, stage, cuts, 0.005 * k)
    idx = lay['idx']
    n0 = len(pos)
    pos, idx, verts = _rim(pos, nor, idx, 0.6 * JACKET_LIFT[gid] * k, skip=lay['sleeve'] > SHELL_SLEEVE - 0.06)
    nor_all = smooth_normals(pos, idx)
    d = _shell_dense(lay['src'], dense)
    d = np.concatenate([d, d[verts]])
    n = len(pos)
    f = np.zeros((n, 4))
    f[:, 0] = 0.05
    f[n0:, 0] = 0.0
    f[:, 2] = TOP
    return dict(pos=pos, nor=nor_all, idx=idx, dense=d, fields=f, tone=np.zeros((n, 4)))


def _outward(pos, idx, centre):
    """Triangles turned to face away from a vertical axis through `centre` (x, z) where most don't."""
    t = idx.reshape(-1, 3)
    n = np.cross(pos[t[:, 1]] - pos[t[:, 0]], pos[t[:, 2]] - pos[t[:, 0]])
    c = pos[t].mean(1)
    radial = c - np.array([centre[0], 0, centre[1]])
    radial[:, 1] = 0
    if (np.einsum('ij,ij->i', n, radial) < 0).mean() > 0.5:
        t = t[:, [0, 2, 1]]
    return t.ravel()


def _closed_outward(pos, idx):
    """A closed mesh's triangles, turned round if they enclose a negative volume."""
    t = idx.reshape(-1, 3)
    vol = np.einsum('ij,ij->i', pos[t[:, 0]], np.cross(pos[t[:, 1]], pos[t[:, 2]])).sum()
    return (t[:, [0, 2, 1]] if vol < 0 else t).ravel()


def _section(pos, tris, y, keep):
    """Points where the mesh's edges (of triangles whose corners are all in `keep`) cross height y."""
    t = tris.reshape(-1, 3)
    t = t[keep[t].all(1)]
    e = np.concatenate([t[:, [0, 1]], t[:, [1, 2]], t[:, [2, 0]]])
    a, b = pos[e[:, 0]], pos[e[:, 1]]
    m = (a[:, 1] - y) * (b[:, 1] - y) < 0
    a, b = a[m], b[m]
    f = (y - a[:, 1]) / (b[:, 1] - a[:, 1])
    return a + (b - a) * f[:, None]


def _follow(m, s, k=4):
    """Skin weights from the nearest body vertices."""
    d, nn = cKDTree(s['pos']).query(m['pos'], k=k)
    w = 1 / np.maximum(d, 1e-4)
    w /= w.sum(1, keepdims=True)
    m['dense'] = (s['dense'][nn] * w[..., None]).sum(1)
    return m


def _strip(rows, closed=False):
    """Quads of a grid of points (r, c, 3) (rows along the strip's length)."""
    r, c, _ = rows.shape
    idx = []
    cols = c if closed else c - 1
    for i in range(r - 1):
        for j in range(cols):
            a, b = i * c + j, i * c + (j + 1) % c
            idx += [a, b, b + c, a, b + c, a + c]
    return rows.reshape(-1, 3), np.array(idx, np.int64)


def collar(family, stage, gid, cuts):
    """A turned-down collar on the neckline (open at the front)."""
    s = surface(family, stage)
    sb = surface(family, BASE_STAGE)
    P = s['P']
    cut = cuts[gid]
    f, _ = _base_fields(family)
    # Points on the neckline of the base body (crew field = cut), carried to the stage by the
    # nearest base vertices.
    zn = sb['P']['neck_01'][2]
    th = np.radians(np.linspace(28, 332, 15))
    hd = sb['region'][:, REGIONS.index('head')] + sb['region'][:, REGIONS.index('ear')]
    near = (np.abs(f[:, 2] - cut['crew']) < 0.012) & (hd < 0.5) & (sb['tags']['arm'] < 0.3)
    cand = np.where(near)[0]
    ang = np.arctan2(sb['pos'][cand, 0], sb['pos'][cand, 2] - zn) % (2 * np.pi)
    picks = []
    for t in th:
        da = np.abs((ang - t + np.pi) % (2 * np.pi) - np.pi)
        picks.append(cand[np.argsort(da)[:3]])
    picks = np.array(picks)
    base_pts = s['pos'][picks].mean(1)
    nor = smooth_normals(s['pos'], surface_tris(family))[picks].mean(1)
    nor /= np.linalg.norm(nor, axis=1, keepdims=True)
    up = np.array([0, 1.0, 0])
    H = proportions(family, stage)['H']
    k = H / 1.7
    stand = (0.022 if gid == 'top.shirt' else 0.018) * k
    fall = (0.034 if gid == 'top.shirt' else 0.028) * k
    tipn = np.clip(1 - np.abs(np.cos(th)) ** 6, 0, 1)
    front = np.clip(np.cos(th), 0, 1)
    r0 = base_pts + nor * 0.003 * k
    r1 = r0 + up * stand + nor * 0.004 * k
    r2 = r1 + nor * 0.012 * k - up * (fall * (1 + 0.5 * front ** 2))[:, None] + nor * (0.01 * front)[:, None] * k
    del tipn
    rows = np.array([r0, r1, r2])
    pos, idx = _strip(rows)
    idx = _outward(pos, idx, (0, P['neck_01'][2]))
    m = dict(pos=pos, idx=idx)
    _follow(m, s)
    m['nor'] = smooth_normals(pos, idx)
    n = len(pos)
    f = np.zeros((n, 4))
    f[:, 0] = np.repeat([0.03, 0.0, 0.0], len(th))
    f[:, 2] = TOP
    m['fields'] = f
    m['tone'] = np.zeros((n, 4))
    return m


@lru_cache(maxsize=None)
def surface_tris(family):
    from body import _tris
    return _tris(family).ravel()


@lru_cache(maxsize=None)
def _skirt_profile(family, stage, waist, segs, rows):
    """The skirt's rows (heights), axis z, angles and radii: the body's sections (torso and legs,
    sampled between the rows) plus ease, snug over the hips and flaring below."""
    s = surface(family, stage)
    sb = surface(family, BASE_STAGE)
    P = s['P']
    Pb = sb['P']
    # Heights on the base body, carried to the stage proportionally between the hips and knees.
    yb0, yb1 = waist + 0.004, Pb['calf_l'][1] + 0.07

    def stage_y(yb):
        t = (yb - Pb['calf_l'][1]) / (Pb['thigh_l'][1] - Pb['calf_l'][1])
        return P['calf_l'][1] + t * (P['thigh_l'][1] - P['calf_l'][1])

    ys = np.array([stage_y(yb0 + (yb1 - yb0) * (i / (rows - 1))) for i in range(rows)])
    cz = P['pelvis'][2]
    ang = np.linspace(0, 2 * np.pi, segs, endpoint=False)
    keep = (s['tags']['arm'] < 0.1) & (s['region'][:, REGIONS.index('torso')] + s['region'][:, REGIONS.index('leg')] > 0.5)
    tris = surface_tris(family)
    step = (ys[0] - ys[-1]) / (rows - 1)
    R = np.zeros((rows, segs))
    width = np.pi / segs * 1.6
    for i, y in enumerate(ys):
        offs = (-0.5, -0.25, 0, 0.25, 0.5) + ((0.75,) if i == 0 else ())
        pts = np.concatenate([_section(s['pos'], tris, y + f * step, keep) for f in offs])
        va = np.arctan2(pts[:, 0], pts[:, 2] - cz) % (2 * np.pi)
        vr = np.hypot(pts[:, 0], pts[:, 2] - cz)
        for j, a in enumerate(ang):
            da = np.abs((va - a + np.pi) % (2 * np.pi) - np.pi)
            m = da < width
            R[i, j] = vr[m].max() if m.any() else (R[i - 1, j] if i else 0.1)
    R = np.maximum(R, (np.roll(R, 1, 1) + 2 * R + np.roll(R, -1, 1)) / 4)
    k = proportions(family, stage)['H'] / 1.7
    t = np.linspace(0, 1, rows)[:, None]
    top = R[0] + 0.006 * k
    hem = R.max(0) + 0.05 * k
    R = np.maximum(R + 0.006 * k, top + (hem - top) * np.clip((t - 0.2) / 0.8, 0, 1) ** 0.9)
    return ys, cz, ang, R


def _over_skirt(pos, family, stage, cuts, lift):
    """Points pushed out from the pelvis' axis to lie `lift` outside the skirt where it is."""
    ys, cz, ang, R = _skirt_profile(family, stage, cuts['bottom.skirt']['waist'], SKIRT_SEGS, SKIRT_ROWS)
    a = np.arctan2(pos[:, 0], pos[:, 2] - cz) % (2 * np.pi)
    r = np.hypot(pos[:, 0], pos[:, 2] - cz)
    j = a / (2 * np.pi) * len(ang)
    j0 = np.floor(j).astype(int) % len(ang)
    f = j - np.floor(j)
    yi = np.clip((ys[0] - pos[:, 1]) / (ys[0] - ys[-1]) * (len(ys) - 1), 0, len(ys) - 1)
    i0 = np.minimum(np.floor(yi).astype(int), len(ys) - 2)
    g = yi - i0
    Rv = sum(w * R[ii % len(ys), jj % len(ang)] for w, ii, jj in (((1 - g) * (1 - f), i0, j0), ((1 - g) * f, i0, j0 + 1), (g * (1 - f), i0 + 1, j0), (g * f, i0 + 1, j0 + 1)))
    inside = (pos[:, 1] <= ys[0] + 0.03) & (pos[:, 1] >= ys[-1])
    need = np.where(inside, np.maximum(Rv + lift - r, 0), 0)
    d = np.stack([np.sin(a), np.zeros_like(a), np.cos(a)], -1)
    return pos + d * need[:, None]


def skirt_shell(family, stage, cuts, segs=SKIRT_SEGS, rows=SKIRT_ROWS):
    """An A-line skirt from the waistband to above the knees."""
    cut = cuts['bottom.skirt']
    ys, cz, ang, R = _skirt_profile(family, stage, cut['waist'], segs, rows)
    k = proportions(family, stage)['H'] / 1.7
    yb0, yb1 = ys[0], ys[-1]
    grid = np.stack([np.sin(ang)[None] * R, np.broadcast_to(ys[:, None], R.shape), cz + np.cos(ang)[None] * R], -1)
    # Hem: a band turned up inside the bottom edge (the cloth's thickness).
    hem_in = grid[-1] - np.stack([np.sin(ang), np.zeros(segs), np.cos(ang)], -1) * 0.004 * k + [0, 0.012 * k, 0]
    pos, idx = _strip(grid, closed=True)
    hp, hi = _strip(np.array([grid[-1], hem_in]), closed=True)
    pos = np.concatenate([pos, hp])
    idx = np.concatenate([idx, hi + rows * segs])
    n = len(pos)
    dense = np.zeros((n, NB))
    t = np.concatenate([np.repeat(np.linspace(0, 1, rows), segs), np.ones(2 * segs)])
    side = np.concatenate([np.tile(np.sin(ang), rows), np.sin(ang), np.sin(ang)])
    tw = smoothstep(0.1, 1.0, t) * 0.9
    sr = np.clip(0.5 + side * 0.9, 0, 1)
    dense[:, B['thigh_r']] = tw * sr
    dense[:, B['thigh_l']] = tw * (1 - sr)
    dense[:, B['pelvis']] = 1 - tw
    f = np.zeros((n, 4))
    f[:, 0] = (1 - t) * (yb0 - yb1)
    f[:, 1] = FLAG_SKIRT
    f[:, 2] = BOTTOM
    tri = _outward(pos[:rows * segs], idx[:(rows - 1) * segs * 6], (0, cz))
    if not np.array_equal(tri, idx[:(rows - 1) * segs * 6]):
        tri = idx.reshape(-1, 3)[:, [0, 2, 1]].ravel()
    else:
        tri = idx
    return dict(pos=pos, idx=tri, nor=smooth_normals(pos, tri), dense=dense, fields=f, tone=np.zeros((n, 4)))


def _prism(outline, y0, y1, centre):
    """A closed prism: the outline (k, 2: x, z) from y0 to y1, capped (counter-clockwise outside)."""
    k = len(outline)
    bot = np.c_[outline[:, 0], np.full(k, y0), outline[:, 1]]
    top = np.c_[outline[:, 0], np.full(k, y1), outline[:, 1]]
    pos = np.concatenate([bot, top, [[centre[0], y0, centre[1]], [centre[0], y1, centre[1]]]])
    idx = []
    for j in range(k):
        a, b = j, (j + 1) % k
        idx += [a, b, b + k, a, b + k, a + k]
        idx += [2 * k, b, a]
        idx += [2 * k + 1, a + k, b + k]
    return pos, np.array(idx, np.int64)


def soles(family, stage, gid, cuts, lod=False):
    """A sole under each foot (a rounded slab a little wider than the foot), and boots' cuffs."""
    s = surface(family, stage)
    P = s['P']
    H = proportions(family, stage)['H']
    k = H / 1.7
    thick = {'shoes.sneakers': 0.016, 'shoes.trainers': 0.018, 'shoes.boots': 0.02, 'shoes.dress': 0.011}[gid] * k
    pos_all, idx_all, flag_all, hem_all = [], [], [], []
    n = 0
    foot = s['region'][:, REGIONS.index('foot')] > 0.5
    segs = 10 if lod else 16
    for side in ('l', 'r'):
        sx = -1 if side == 'l' else 1
        pts = s['pos'][foot & (np.sign(s['pos'][:, 0]) == sx)]
        low = pts[pts[:, 1] < np.percentile(pts[:, 1], 40)]
        cx = low[:, 0].mean()
        z0, z1 = low[:, 2].min(), low[:, 2].max()
        w = (low[:, 0].max() - low[:, 0].min()) / 2
        a = np.linspace(0, 2 * np.pi, segs, endpoint=False)
        L = (z1 - z0) / 2 + 0.005 * k
        cz = (z0 + z1) / 2
        # A rounded rectangle (superellipse), a little narrower at the heel.
        e = 2 / 3.2
        ox = np.sign(np.sin(a)) * np.abs(np.sin(a)) ** e * (w + 0.005 * k) * (1 - 0.1 * (np.cos(a) < 0) * np.abs(np.cos(a)))
        oz = np.sign(np.cos(a)) * np.abs(np.cos(a)) ** e * L
        outline = np.c_[cx + ox, cz + oz]
        p, idx = _prism(outline, 0.0, thick, (cx, cz))
        idx = _closed_outward(p, idx)
        pos_all.append(p)
        idx_all.append(idx + n)
        flag_all.append(np.full(len(p), FLAG_SOLE))
        hem_all.append(np.zeros(len(p)))
        n += len(p)
        if gid == 'shoes.boots':
            # The cuff: a short tube round the shin at the boot's top, a little proud of the leg.
            Pb = surface(family, BASE_STAGE)['P']
            top_b = cuts[gid]['shoe']
            t = (top_b - Pb[f'foot_{side}'][1]) / (Pb[f'calf_{side}'][1] - Pb[f'foot_{side}'][1])
            yc = P[f'foot_{side}'][1] + t * (P[f'calf_{side}'][1] - P[f'foot_{side}'][1])
            keep = (s['region'][:, REGIONS.index('leg')] > 0.3) & (np.sign(s['pos'][:, 0]) == sx)
            lp = _section(s['pos'], surface_tris(family), yc, keep)
            c = np.array([lp[:, 0].mean(), yc, lp[:, 2].mean()])
            rx = (lp[:, 0].max() - lp[:, 0].min()) / 2 + 0.006 * k
            rz = (lp[:, 2].max() - lp[:, 2].min()) / 2 + 0.006 * k
            cs = 8 if lod else 12
            b = np.linspace(0, 2 * np.pi, cs, endpoint=False)
            ring = lambda y, grow: np.c_[c[0] + np.sin(b) * (rx + grow), np.full(cs, y), c[2] + np.cos(b) * (rz + grow)]
            rows = np.array([ring(yc - 0.022 * k, -0.004 * k), ring(yc - 0.016 * k, 0.0), ring(yc + 0.002 * k, 0.0), ring(yc + 0.004 * k, -0.005 * k)])
            p, idx = _strip(rows, closed=True)
            idx = _outward(p, idx, (c[0], c[2]))
            pos_all.append(p)
            idx_all.append(idx + n)
            flag_all.append(np.zeros(len(p)))
            hem_all.append(np.zeros(len(p)))
            n += len(p)
    pos = np.concatenate(pos_all)
    idx = np.concatenate(idx_all)
    m = dict(pos=pos, idx=idx)
    _follow(m, s)
    m['nor'] = smooth_normals(pos, idx)
    f = np.zeros((len(pos), 4))
    f[:, 0] = np.concatenate(hem_all)
    f[:, 1] = np.concatenate(flag_all)
    f[:, 2] = SHOES
    m['fields'] = f
    m['tone'] = np.zeros((len(pos), 4))
    return m


def build_clothes(family, stage, head, P, body, cuts):
    """Every shell of `family` at `stage`, each with a low-detail copy (`lod.<id>`)."""
    out = {}
    for gid in ('top.jacket', 'top.suit'):
        if gid in cuts:
            out[gid] = jacket(family, stage, gid)
            out[f'lod.{gid}'] = jacket(family, stage, gid, lod=True)
    for gid in ('top.polo', 'top.shirt'):
        if gid in cuts:
            out[gid] = collar(family, stage, gid, cuts)
            out[f'lod.{gid}'] = dict(out[gid])
    if 'bottom.skirt' in cuts:
        out['bottom.skirt'] = skirt_shell(family, stage, cuts)
        out['lod.bottom.skirt'] = skirt_shell(family, stage, cuts, segs=12, rows=4)
    for gid in [g for g in cuts if g.startswith('shoes.')]:
        out[gid] = soles(family, stage, gid, cuts)
        out[f'lod.{gid}'] = soles(family, stage, gid, cuts, lod=True)
    return out

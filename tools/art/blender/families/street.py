# The street's dressing props (`model.street.*`): the kerbside bench, the planter with its little tree, the flower
# tub, the street-name sign, the bus-stop sign, the pot by a front door, the garden path light, and the foliage of
# the street (`flowers`). Notes: street.md.
#
# render/babylon/street.ts draws them as thin instances along the kerbs and in the neighbours' front gardens, dozens
# to hundreds of copies, and the town overview draws them again from afar: each is small (a few hundred to a
# thousand triangles), takes one or two draw calls, and has a far version, its own build at low detail
# (`G.far_of`). They are still; the leaves and flowers are on the `foliage` finish, so the nature shader sways them
# in the wind as it did the old parts'. The path light's lens glows at night.
#
# Run:  cargo art street [--only bench] [--preview]
import math
import os
import random
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import bpy  # noqa: E402

import gamekit as G  # noqa: E402
from gamekit import far_of, low  # noqa: E402
from plants import lump  # noqa: E402

M = G.Palette(
    iron=('#2B302E', 'metal'),         # the bench's cast-iron ends
    wood=('#9A6B45', 'wood'),          # its slats
    concrete=('#9A958C', 'matte'),     # the planter and the tub
    soil=('#4A3526', 'matte'),
    trunk=('#6B4F3A', 'satin'),        # the planter's tree: plain, so the planter stays two draws
    stake=('#8A6A48', 'satin'),        # its stakes, plain too
    tie=('#2B2C2E', 'satin'),          # the rubber tie round the trunk and stakes
    leaf=('#5E8F4E', 'foliage'),
    leaf_light=('#6E9C58', 'foliage'),
    leaf_dark=('#527F45', 'foliage'),
    bed=('#4E7432', 'foliage'),        # the flowers' leafy clumps
    bed_light=('#5C8438', 'foliage'),
    red=('#D9483B', 'foliage'),        # flowers: on foliage, as the old parts', so they sway with the leaves
    yellow=('#F2C14E', 'foliage'),
    pink=('#E9A6B4', 'foliage'),
    rose=('#E8678A', 'foliage'),
    blue=('#9FC5E8', 'foliage'),
    violet=('#B07AD8', 'foliage'),
    white=('#F2F0EA', 'foliage'),
    eye=('#F2B935', 'foliage'),        # the flowers' eyes
    eye_dark=('#7A4E2A', 'foliage'),
    pole=('#5C6168', 'metal'),         # the signs' poles
    sign=('#2F5D3A', 'gloss'),         # the street-name sign's green blades and lettering
    plate=('#F2F2F0', 'gloss'),        # white panels
    flag=('#2E6FA8', 'gloss'),         # the bus stop's blue
    ink=('#3A3F45', 'matte'),        # the timetable's lines, the bus's wheels
    terracotta=('#B5653F', 'satin'),   # the door pot
    lamp=('#2B2C2E', 'metal'),         # the path light's post and cap
    # the path light's frosted lens: glows at night (E8), where the old part's glass bulb was
    lens=('#FFF1CF', 'satin', {'glow': '#FFD9A0'}),
)


def _box(F, name, w, h, d, at, material, bevel=0.0, **kw):
    """A box in game axes; bevelled with one segment (half the triangles of two), none in a far version."""
    return F.box(name, w, h, d, at, material, bevel=low(bevel, 0.0), bevel_segments=1, **kw)


# --- the bench: four slats on two cast-iron ends, its back to the road ---------------------------------------
def bench():
    F = G.GameFrame()
    full = not G.is_low()
    # the ends: each cast in one piece (the back leg curving up into the upright, the front leg bending back into
    # the armrest, its front curled in a scroll, the seat rail between), on pad feet
    for x in (-0.8, 0.8):
        G.kit.bench_end(F, f'end{x}', x, 0.415, 0.19, -0.21, (0.885, -0.272), M.iron, arm=0.68, t=0.026, w=0.042,
                        steps=low(3, 1), corner=low(1, 0), scrolls=full)
    # the slats: rounded edges, held down by domed bolts over each end
    for k, z in enumerate((-0.1408, -0.0484, 0.044, 0.1364)):
        F.box(f'slat{k}', 1.7, 0.035, 0.085, (0, 0.4325, z), M.wood, bevel=low(0.009, 0.0), bevel_segments=2)
        if full:
            for x in (-0.8, 0.8):
                F.sphere(f'bolt{k}{x}', 0.009, (x, 0.45, z), M.iron, segments=6, rings=3, scale=(1, 1, 0.5))
    for k, (y, z) in enumerate(((0.6, -0.18), (0.73, -0.225), (0.86, -0.27))):
        F.box(f'back{k}', 1.7, 0.085, 0.028, (0, y, z), M.wood, bevel=low(0.009, 0.0), bevel_segments=2, rot=(-12, 0, 0))
        if full:
            for x in (-0.8, 0.8):
                F.sphere(f'back_bolt{k}{x}', 0.009, (x, y, z + 0.016), M.iron, segments=6, rings=3, scale=(1, 0.5, 1))
    G.join_all('bench')
    G.settle(centre=False)
    return G.meshes()


# --- the planter and the flower tub: a concrete tub, a little tree or flowers ---------------------------------
# The tub's section (half-width, height): a chamfered foot, the body tapering out to a groove, a rim proud of it
# with chamfered edges, and inside down to under the soil.
TUB = [(0.300, 0.0), (0.311, 0.011), (0.311, 0.03), (0.304, 0.036), (0.322, 0.30), (0.318, 0.306), (0.324, 0.312),
       (0.33, 0.36), (0.347, 0.367), (0.35, 0.41), (0.343, 0.42), (0.302, 0.42), (0.294, 0.41), (0.289, 0.37)]
TUB_FAR = [(0.300, 0.0), (0.311, 0.011), (0.33, 0.36), (0.35, 0.37), (0.35, 0.42), (0.298, 0.42), (0.289, 0.37)]


def _tub(F, groove=True):
    """The concrete tub (0.7 m square, 0.42 high) in one piece, hollow at the top, and its soil 2 cm below the rim;
    `groove`: a groove round it under the rim."""
    n = low(24 if groove else 16, 12)
    rows = TUB_FAR if G.is_low() else TUB if groove else TUB[:4] + TUB[7:]
    G.loft('tub', [G.ring(r, r, y, n, square=9) for r, y in rows], material=M.concrete)
    soil = G.loft('soil', [G.ring(0.291, 0.291, 0.37, n, square=9), G.ring(0.292, 0.292, 0.395, n, square=9)],
                  material=M.soil)
    if not G.is_low():
        G.jitter(soil, 0.004, 0.08, seed=3.0)


def planter():
    F = G.GameFrame()
    full = not G.is_low()
    _tub(F)
    # a young street tree: a slim trunk (plain satin), tapering, forking under a crown of leafy lumps
    F.sweep('trunk', [(0, 0.37, 0), (0.008, 0.9, 0), (-0.006, 1.4, 0.006), (0.0, 1.85, 0.0)], 0.038, M.trunk,
            verts=low(8, 4), resolution=low(2, 1), scale=[1.0, 0.8, 0.68, 0.55])
    F.cone('flare', 0.06, 0.036, 0.06, (0, 0.42, 0), M.trunk, verts=low(8, 4))
    for i, (x, z) in enumerate(((0.16, 0.06), (-0.15, -0.1), (0.02, -0.14), (-0.05, 0.15))):
        y0 = 1.45 + i * 0.07
        F.sweep(f'branch{i}', [(0, y0, 0), (x * 0.5, y0 + 0.2, z * 0.5), (x, y0 + 0.38, z)], 0.017, M.trunk,
                verts=low(5, 3), resolution=low(2, 1), scale=[1.0, 0.8, 0.55])
    # two stakes either side of the trunk, tied to it with a rubber band on a spacer
    for s in (-1, 1):
        F.cyl(f'stake{s}', 0.018, 1.0, (s * 0.12, 0.89, 0.02 * s), M.stake, verts=low(8, 4))
        if full:
            F.cone(f'stake_top{s}', 0.018, 0.012, 0.012, (s * 0.12, 1.396, 0.02 * s), M.stake, verts=8)
    F.box('tie', 0.24, 0.03, 0.012, (0, 1.2, 0.0), M.tie, bevel=low(0.003, 0.0), bevel_segments=1, rot=(0, 0, 9))
    crown = [  # radius, bottom (game x, height, depth), material, height share
        (0.36, (0.0, 1.64, 0.0), M.leaf, 0.86),
        (0.24, (0.15, 1.98, 0.08), M.leaf_light, 0.9),
        (0.22, (-0.16, 1.9, -0.12), M.leaf_dark, 0.9),
        (0.2, (0.2, 1.76, -0.13), M.leaf_dark, 0.85),
        (0.2, (-0.2, 1.72, 0.13), M.leaf_light, 0.85),
        (0.18, (0.0, 2.1, -0.02), M.leaf_light, 0.9),
        (0.16, (0.1, 1.62, 0.1), M.leaf_dark, 0.8),     # small lumps filling the crown's underside and its gaps
        (0.15, (-0.14, 1.62, -0.08), M.leaf, 0.8),
        (0.15, (-0.04, 2.0, 0.08), M.leaf, 0.85),
    ]
    for i, (r, (x, y, z), mat, h) in enumerate(crown):
        lump(f'crown{i}', r, F.at(x, y, z), mat, scale=(1, 1, h), noise=0.14, seed=i * 1.7, segments=12, rings=7)
    G.join_all('planter')
    G.settle()
    return G.meshes()


def _bloom(F, name, x, y, z, r, petals, mat, eye, tilt=(0, 0, 0), detail=3):
    """A flower open to the sky: `petals` rounded petals round a raised eye, at game (x, y, z), `r` across its
    petals."""
    pts = []
    n = petals * detail
    for i in range(n):
        a = math.tau * i / n
        k = abs(math.cos(petals * a / 2)) ** 0.7
        rr = r * (0.42 + 0.58 * k)
        pts.append((rr * math.cos(a), rr * math.sin(a)))
    F.extrude(name, pts, 0.01, (x, y, z), mat, plane='top', rot=tilt)
    F.cyl(f'{name}_eye', r * 0.3, 0.014, (x, y + 0.002, z), eye, verts=5, rot=tilt)


def flowerTub():
    F = G.GameFrame()
    full = not G.is_low()
    _tub(F, groove=False)
    rng = random.Random(81)
    # a leafy mound over the soil, and flowers in five colours on it, the old part's five where it had them
    lump('mound', 0.27, F.at(0, 0.39, 0), M.leaf, scale=(1, 1, 0.25), noise=0.16, segments=12, rings=6)
    for i, (x, z) in enumerate(((0.2, 0.2), (-0.21, 0.19), (0.21, -0.2), (-0.2, -0.2))):
        lump(f'leaves{i}', 0.11, F.at(x, 0.4, z), [M.leaf_dark, M.leaf_light][i % 2], scale=(1, 1, 0.6), noise=0.2, seed=i * 2.3,
             segments=7, rings=4)
    big = [(-0.15, -0.12, M.red), (0.14, -0.1, M.yellow), (0.0, 0.14, M.pink), (-0.16, 0.16, M.blue), (0.17, 0.15, M.white)]
    cols = [M.red, M.yellow, M.pink, M.blue, M.white, M.violet]
    spots = big + [(rng.uniform(-0.24, 0.24), rng.uniform(-0.24, 0.24), cols[i % len(cols)]) for i in range(12)]
    for i, (x, z, mat) in enumerate(spots):
        drop = 0 if i < len(big) else rng.uniform(0.0, 0.015)
        tilt = (rng.uniform(-12, 12), rng.uniform(-12, 12), rng.uniform(0, 72))
        y = (0.562 if i < len(big) else 0.55 - drop) - 0.06 * (x * x + z * z) / 0.08
        r = 0.07 if i < len(big) else 0.045
        if full:
            _bloom(F, f'bloom{i}', x, y, z, r, 5 if i % 3 else 6, mat, M.eye if mat is not M.yellow else M.eye_dark, tilt,
                   detail=5 if i < len(big) else 2)
        elif i < len(big):                       # far off, the big blooms only, as lumps
            lump(f'bloom{i}', r * 0.8, F.at(x, y - 0.045, z), mat, scale=(1, 1, 0.5), noise=0.2, seed=i * 0.9,
                 segments=5, rings=3)
    G.join_all('flowerTub')
    G.settle()
    return G.meshes()


# --- the street-name sign: two green blades crossing on a pole -----------------------------------------------
def _lettering(F, name, length, y, along_x, face, rng):
    """Words of dark green lettering on a blade's white panel, on one face (+1 front, -1 back): each word a capital,
    then small letters on the same baseline, a few with ascenders."""
    x = -length / 2
    k = 0
    start = True
    while x < length / 2 - 0.04:
        w = rng.choice((0.03, 0.035, 0.04))
        h = rng.choice((0.06, 0.06, 0.07))           # (drawn as before, so the rest of the sign's numbers stay put)
        if rng.random() < 0.14 and k:
            x += 0.04                                     # a space between words
            start = True
        h = 0.075 if start else (0.068 if h > 0.065 else 0.05)
        start = False
        c = x + w / 2
        at = (c, y - 0.0375 + h / 2, face * 0.019) if along_x else (face * 0.019, y - 0.0375 + h / 2, c)
        size = (w, h, 0.004) if along_x else (0.004, h, w)
        _box(F, f'{name}{k}', *size, at, M.sign)
        x += w + 0.012
        k += 1


def _post(F, top, foot=0.075, r=0.035):
    """A round steel post from a flanged foot bolted to the ground up to `top`: the foot's profile turned, a collar
    where the post leaves it."""
    n = low(12, 6)
    c = min(r + 0.01, foot)
    prof = [(0, 0), (foot, 0), (foot, 0.01), (max(c, foot - 0.012), 0.016), (c, 0.03), (min(c, r + 0.006), 0.09), (c, 0.1),
            (c, 0.115), (r, 0.125), (r, top), (0, top)]
    G.revolve('post', prof if not G.is_low() else [(0, 0), (foot, 0), (c, 0.03), (r, 0.12), (r, top), (0, top)],
              material=M.pole, segments=n)
    if not G.is_low() and foot > r + 0.03:
        for k in range(4):                                # the foot's four bolts
            a = math.tau * (k + 0.5) / 4
            F.cyl(f'bolt{k}', 0.008, 0.012, ((foot - 0.018) * math.cos(a), 0.014, (foot - 0.018) * math.sin(a)), M.pole,
                  verts=6)


def _band(F, name, y, r=0.043, h=0.03):
    """A clamp band round the post at height y."""
    F.cyl(name, r, h, (0, y, 0), M.pole, verts=low(12, 6), bevel=low(0.004, 0.0), bevel_segments=1)


def sign():
    F = G.GameFrame()
    rng = random.Random(7)
    full = not G.is_low()
    _post(F, 2.585)
    G.revolve('finial', [(0, 2.58), (0.042, 2.58), (0.042, 2.592), (0.03, 2.6), (0.046, 2.625), (0.038, 2.648),
                         (0.016, 2.662), (0, 2.665)], material=M.pole, segments=low(12, 6))
    for y, along_x in ((2.45, True), (2.25, False)):
        # the blade: rounded at its corners, a white panel set into each face, held by bands above and below
        outline = G.rounded_rect(0.9, 0.18, low(0.03, 0.0), corner=low(3, 1))
        panel = G.rounded_rect(0.84, 0.12, low(0.018, 0.0), corner=low(3, 1))
        plane = 'front' if along_x else 'side'
        F.extrude(f'blade{y}', outline, 0.03, (0, y, 0), M.sign, plane=plane)
        F.extrude(f'panel{y}', panel, 0.032, (0, y, 0), M.plate, plane=plane)
        for dy in (-0.1, 0.1):
            _band(F, f'band{y}{dy}', y + dy)
        if full:
            for face in (1, -1):
                _lettering(F, f'letter{y}{face}', 0.62, y, along_x, face, rng)
    G.join_all('sign')
    G.settle()
    return G.meshes()


# --- the bus-stop sign: the round blue flag with a bus on it, and the timetable -------------------------------
def _bus(F, name, y, z, face):
    """The bus pictogram on one face of the flag (+1 front, -1 back): a blue bus side, its windows and door cut out of
    it so the white face shows through, two dark wheels."""
    s = face
    outline = [(-0.1, -0.035), (0.1, -0.035), (0.1, 0.025), (0.085, 0.045), (-0.095, 0.045), (-0.1, 0.035)]
    holes = [[(u - 0.072 + k * 0.036, v + 0.016) for u, v in G.rounded_rect(0.028, 0.026, 0.004, corner=1)]
             for k in range(4)]
    holes.append([(0.068, -0.024), (0.088, -0.024), (0.088, 0.0), (0.068, 0.0)])          # the door's window
    outline = [(u * s, v) for u, v in outline]
    holes = [[(u * s, v) for u, v in h] for h in holes]
    if s < 0:                                             # mirrored: keep the loops anticlockwise
        outline = outline[::-1]
        holes = [h[::-1] for h in holes]
    F.extrude(name, outline, 0.004, (0, y, z), M.flag, holes=holes)
    for x in (-0.06, 0.055):
        G.cylinder(f'{name}_wheel{x}', 0.017, 0.006, at=F.at(x * s, y - 0.035, z), material=M.ink, rot=(90, 0, 0), verts=8)


def busSign():
    F = G.GameFrame()
    full = not G.is_low()
    # the post stops under the flag (into its rim), so the flag's faces stay whole
    _post(F, 2.2, foot=0.0375)
    _band(F, 'collar', 2.15, r=0.037, h=0.04)
    # the flag: a disc with a rolled rim, a white face proud of it on both sides, the bus on each face
    G.revolve('flag', [(0, -0.017), (0.21, -0.017), (0.226, -0.012), (0.23, 0.0), (0.226, 0.012), (0.21, 0.017), (0, 0.017)]
              if full else [(0, -0.02), (0.23, -0.02), (0.23, 0.02), (0, 0.02)],
              at=F.at(0, 2.4, 0), material=M.flag, segments=low(32, 12), rot=(90, 0, 0))
    G.cylinder('flag_face', 0.17, 0.042, at=F.at(0, 2.4, 0), material=M.plate, rot=(90, 0, 0), verts=low(28, 12))
    if full:
        for face in (1, -1):
            _bus(F, f'bus{face}', 2.41, face * 0.023, face)
    # the timetable in its case, clamped to the pole's front: a framed case under a little hood
    _box(F, 'case_back', 0.29, 0.41, 0.02, (0, 1.5, 0.045), M.pole, bevel=0.004)
    frame = G.rounded_rect(0.3, 0.42, low(0.012, 0.0), corner=low(2, 1))
    F.extrude('case', frame, 0.012, (0, 1.5, 0.061), M.pole, holes=[G.rounded_rect(0.264, 0.384, 0.004, corner=1)])
    _box(F, 'hood', 0.32, 0.014, 0.044, (0, 1.718, 0.0482), M.pole, bevel=0.004)
    _box(F, 'timetable', 0.27, 0.39, 0.004, (0, 1.5, 0.057), M.plate)
    _box(F, 'header', 0.26, 0.06, 0.004, (0, 1.66, 0.06), M.flag)
    if full:
        for k in range(10):
            w = 0.16 - (k % 3) * 0.03 if k % 4 != 3 else 0.07
            _box(F, f'line{k}', w, 0.01, 0.003, (-0.11 + w / 2, 1.6 - k * 0.03, 0.06), M.ink)
            _box(F, f'time{k}', 0.05, 0.01, 0.003, (0.085, 1.6 - k * 0.03, 0.06), M.ink)    # the times, at the right
    for y in (1.36, 1.64):
        _band(F, f'clamp{y}', y, r=0.037, h=0.03)
        _box(F, f'lug{y}', 0.05, 0.03, 0.03, (0, y, 0.03), M.pole)
    G.join_all('busSign')
    G.settle(centre=False)
    return G.meshes()


# --- the pot by a front door: terracotta on a saucer, a clipped leafy ball ------------------------------------
def pot():
    F = G.GameFrame()
    full = not G.is_low()
    n = low(20, 8)
    # the pot: a foot ring, the body flaring up to a rolled rim, and inside down to the soil
    G.revolve('pot', [(0, 0.004), (0.142, 0.004), (0.15, 0.0), (0.156, 0.012), (0.152, 0.024), (0.178, 0.3),
                      (0.184, 0.306), (0.2, 0.312), (0.205, 0.34), (0.2, 0.366), (0.186, 0.372), (0.172, 0.366),
                      (0.168, 0.352), (0, 0.352)]
              if full else [(0, 0), (0.155, 0), (0.18, 0.31), (0.2, 0.312), (0.2, 0.37), (0.17, 0.37), (0, 0.35)],
              material=M.terracotta, segments=n)
    if full:
        G.revolve('saucer', [(0.13, 0.002), (0.182, 0.0), (0.198, 0.026), (0.192, 0.03), (0.178, 0.012), (0.13, 0.012)],
                  material=M.terracotta, segments=n)
    soil = G.cylinder('soil', 0.168, 0.012, at=(0, 0, 0.356), material=M.soil, verts=n)
    if full:
        G.jitter(soil, 0.003, 0.05, seed=1.0)
    lump('ball', low(0.205, 0.218), F.at(0, 0.32, 0), M.leaf, scale=(low(1.01, 1.06), 1.005, 1.0), noise=0.1, segments=16,
         rings=8)
    lump('top', 0.13, F.at(0.08, 0.63, 0.04), M.leaf_light, scale=(1, 1, 0.94), noise=0.15, seed=2.0, segments=12, rings=7)
    lump('side', 0.1, F.at(-0.12, 0.5, -0.03), M.leaf_dark, scale=(1, 1, 0.9), noise=0.15, seed=4.0, segments=10, rings=6)
    G.join_all('pot')
    G.settle()
    return G.meshes()


# --- the garden path light: a dark bollard with a frosted lens under louvres and a cap ---------------------------
def pathLight():
    F = G.GameFrame()
    full = not G.is_low()
    n = low(16, 8)
    foot = [(0.08, 0.008), (0.068, 0.014), (0.058, 0.04), (0.054, 0.07)] if full else [(0.07, 0.02)]
    G.revolve('post', [(0, 0), (0.08 if full else 0.07, 0), *foot, (0.054, 0.4), (0.06, 0.41), (0.06, 0.42), (0, 0.42)],
              material=M.lamp, segments=n)
    F.cyl('lens', 0.052, 0.1, (0, 0.47, 0), M.lens, verts=n)
    if full:
        # three louvre rings round the lens, on four thin bars
        for k, y in enumerate((0.444, 0.47, 0.496)):
            G.revolve(f'louvre{k}', [(0.054, y - 0.004), (0.068, y - 0.009), (0.07, y - 0.006), (0.058, y + 0.002),
                                     (0.054, y + 0.002)], material=M.lamp, segments=n)
        for k in range(4):
            a = math.tau * (k + 0.5) / 4
            F.box(f'bar{k}', 0.008, 0.1, 0.008, (0.066 * math.cos(a), 0.47, 0.066 * math.sin(a)), M.lamp)
        F.cyl('screw', 0.006, 0.01, (0, 0.2, 0.054), M.lamp, verts=6, rot=(90, 0, 0))   # the access hatch's screw
    rim = [(0.09, 0.525), (0.075, 0.54), (0.045, 0.553), (0.015, 0.559)] if full else [(0.09, 0.525)]
    G.revolve('cap', [(0, 0.515), (0.09, 0.515), *rim, (0, 0.56)], material=M.lamp, segments=n)
    G.join_all('pathLight')
    G.settle()
    return G.meshes()


# --- the street foliage --------------------------------------------------------------------------------------
def flowers():
    """A clump of leaves and little flowers in many colours, for the beds along the house fronts."""
    F = G.GameFrame()
    rng = random.Random(44)
    lump('clump', 0.29, F.at(0, 0.0, 0), M.bed, scale=(1, 0.9, 0.45), noise=0.2, segments=16, rings=7)
    lump('clump2', 0.17, F.at(0.12, 0.08, -0.06), M.bed_light, scale=(1, 0.9, 0.6), noise=0.2, seed=1.5, segments=10, rings=6)
    lump('clump3', 0.14, F.at(-0.16, 0.05, 0.08), M.bed_light, scale=(1, 0.9, 0.55), noise=0.2, seed=3.5, segments=8, rings=5)
    if not G.is_low():                                   # smaller leafy lumps breaking up its front and sides
        for k, (x, z, r) in enumerate(((0.06, 0.15, 0.1), (0.19, 0.05, 0.09), (-0.2, -0.08, 0.09))):
            lump(f'tuft{k}', r, F.at(x, 0.06, z), [M.bed_light, M.bed, M.bed_light][k], scale=(1, 1, 0.75), noise=0.25,
                 seed=5.0 + k, segments=8, rings=5)
    cols = [M.rose, M.yellow, M.white, M.violet, M.red, M.pink, M.blue]
    bpy.context.view_layer.update()
    dg = bpy.context.evaluated_depsgraph_get()
    for i in range(14):
        a = i * 2.39996 + rng.uniform(-0.3, 0.3)
        rr = 0.24 * math.sqrt((i + 0.5) / 14)
        x, z = rr * math.cos(a) * 1.05, rr * math.sin(a) * 0.95
        y = 0.295 - (rr / 0.24) ** 2 * 0.07 + rng.uniform(-0.015, 0.015)
        tilt = (rng.uniform(-15, 15), rng.uniform(-15, 15), rng.uniform(0, 72))
        mat = cols[i % len(cols)]
        if G.is_low():
            if i % 2:                                  # far off, half the flowers (each drawn, so the rest stand put)
                continue
            lump(f'bloom{i}', 0.036, F.at(x, y, z), mat, scale=(1, 1, 0.7), noise=0.25, seed=i * 0.7, segments=4, rings=3)
            continue
        # each flower held a little over the leaves on its stem, from where the clump's surface is under it
        hit, at, *_ = bpy.context.scene.ray_cast(dg, F.at(x, 1.0, z), (0, 0, -1))
        ground = at.z if hit else y
        top = max(ground + 0.012, y + 0.008 + 0.03 * (1 - rr / 0.24))
        F.sweep(f'stem{i}', [(x, ground - 0.01, z), (x * 1.02, top, z * 1.02)], 0.004, M.bed, verts=4, caps=False)
        _bloom(F, f'bloom{i}', x * 1.02, top, z * 1.02, 0.05, 5 if i % 3 else 4, mat,
               M.eye if mat is not M.yellow else M.eye_dark, (tilt[0] + 40 * z, tilt[1] - 40 * x, tilt[2]), detail=4)
    G.join_all('flowers')
    G.settle()
    return G.meshes()


FAMILY = G.Family(
    'street', M,
    [G.Model('bench', bench, size=(1.7, 0.905, 0.514), centre=(0, -0.037), far=far_of(bench)),
     G.Model('planter', planter, size=(0.8, 2.445, 0.72), far=far_of(planter)),
     G.Model('flowerTub', flowerTub, size=(0.7, 0.57, 0.7), far=far_of(flowerTub)),
     G.Model('sign', sign, size=(0.9, 2.665, 0.9), far=far_of(sign)),
     G.Model('busSign', busSign, size=(0.46, 2.627, 0.108), centre=(0, 0.0163), far=far_of(busSign)),
     G.Model('pot', pot, size=(0.432, 0.88, 0.421), far=far_of(pot)),
     G.Model('pathLight', pathLight, size=(0.18, 0.56, 0.18), far=far_of(pathLight)),
     G.Model('flowers', flowers, size=(0.6, 0.343, 0.52), far=far_of(flowers))],
    comment='Blender-built street dressing (tools/art/blender/families/street.py): the kerbside bench, '
            'the planter with its little tree, the flower tub, the street-name sign, the bus-stop sign, '
            'the door pot, the garden path light and the flower clump that '
            'render/babylon/street.ts draws as thin instances along the kerbs and front gardens. Listed '
            'in the base manifest\'s packs. Each has a far version for the town overview. Leaves and '
            'flowers are on the foliage finish, so they sway in the wind; the path light\'s lens glows at '
            'night.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

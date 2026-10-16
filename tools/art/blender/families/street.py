# The street's dressing props (`model.street.*`): the kerbside bench, the planter with its little tree, the flower
# tub, the street-name sign, the bus-stop sign, the pot by a front door, the garden path light, and the foliage of
# the street (`hedge`, `flowers`), over the primitive-part placeholders of `tools/art/community.py` (the props) and
# `web/public/assets/street/manifest.json` (the hedge and the flowers), which stay the fallback. Notes: street.md.
#
# render/babylon/street.ts draws them as thin instances along the kerbs and in the neighbours' front gardens, dozens
# to hundreds of copies, and the town overview draws them again from afar: each is small (a few hundred to a
# thousand triangles), takes one or two draw calls, and has a far version, its own build at low detail
# (`G.far_of`). They are still; the leaves and flowers are on the `foliage` finish, so the nature shader sways them
# in the wind as it did the placeholders'. The path light's lens glows at night.
#
# Run:  python3 -B tools/art/blender/build.py street [--only bench] [--preview]
import math
import os
import random
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gamekit as G  # noqa: E402
from gamekit import far_of, low  # noqa: E402
from plants import lump, tube  # noqa: E402

M = G.Palette(
    iron=('#2B302E', 'metal'),         # the bench's cast-iron ends
    wood=('#9A6B45', 'wood'),          # its slats
    concrete=('#9A958C', 'matte'),     # the planter and the tub
    soil=('#4A3526', 'matte'),
    trunk=('#6B4F3A', 'satin'),        # the planter's tree: plain, so the planter stays two draws
    leaf=('#5E8F4E', 'foliage'),
    leaf_light=('#6E9C58', 'foliage'),
    leaf_dark=('#527F45', 'foliage'),
    hedge=('#557F36', 'foliage'),
    hedge_light=('#618C3C', 'foliage'),
    hedge_dark=('#4D7531', 'foliage'),
    bed=('#4E7432', 'foliage'),        # the flowers' leafy clumps
    bed_light=('#5C8438', 'foliage'),
    red=('#D9483B', 'foliage'),        # flowers: on foliage, as the placeholders', so they sway with the leaves
    yellow=('#F2C14E', 'foliage'),
    pink=('#E9A6B4', 'foliage'),
    rose=('#E8678A', 'foliage'),
    blue=('#9FC5E8', 'foliage'),
    violet=('#B07AD8', 'foliage'),
    white=('#F2F0EA', 'foliage'),
    pole=('#5C6168', 'metal'),         # the signs' poles
    sign=('#2F5D3A', 'gloss'),         # the street-name sign's green blades and lettering
    plate=('#F2F2F0', 'gloss'),        # white panels
    flag=('#2E6FA8', 'gloss'),         # the bus stop's blue
    ink=('#3A3F45', 'matte'),        # the timetable's lines, the bus's wheels
    terracotta=('#B5653F', 'satin'),   # the door pot
    lamp=('#2B2C2E', 'metal'),         # the path light's post and cap
    # the path light's frosted lens: glows at night (E8), where the placeholder's glass bulb was
    lens=('#FFF1CF', 'satin', {'glow': '#FFD9A0'}),
)


def _box(F, name, w, h, d, at, material, bevel=0.0, **kw):
    """A box in game axes; bevelled with one segment (half the triangles of two), none in a far version."""
    return F.box(name, w, h, d, at, material, bevel=low(bevel, 0.0), bevel_segments=1, **kw)


# --- the bench: four slats on two cast-iron ends, its back to the road ---------------------------------------
def bench():
    F = G.GameFrame()
    v = low(8, 4)
    for x in (-0.8, 0.8):
        G.rod(f'leg_front{x}', F.at(x, 0.0, 0.19), F.at(x, 0.44, 0.17), 0.022, material=M.iron, verts=v)
        G.rod(f'leg_back{x}', F.at(x, 0.0, -0.21), F.at(x, 0.44, -0.17), 0.022, material=M.iron, verts=v)
        G.rod(f'upright{x}', F.at(x, 0.44, -0.17), F.at(x, 0.9, -0.275), 0.02, material=M.iron, verts=v)
        G.rod(f'rail{x}', F.at(x, 0.415, -0.19), F.at(x, 0.415, 0.19), 0.018, material=M.iron, verts=v)
        G.rod(f'arm{x}', F.at(x, 0.66, -0.2), F.at(x, 0.66, 0.17), 0.02, material=M.iron, verts=v)
        G.rod(f'arm_post{x}', F.at(x, 0.415, 0.165), F.at(x, 0.66, 0.175), 0.017, material=M.iron, verts=v)
        F.sphere(f'arm_end{x}', 0.028, (x, 0.66, 0.18), M.iron, segments=low(8, 4), rings=low(5, 3))
        for z in (0.19, -0.21):
            _box(F, f'foot{x}{z}', 0.05, 0.025, 0.06, (x, 0.0125, z), M.iron)
    for k, z in enumerate((-0.1408, -0.0484, 0.044, 0.1364)):
        _box(F, f'slat{k}', 1.7, 0.035, 0.085, (0, 0.4325, z), M.wood, bevel=0.008)
    for k, (y, z) in enumerate(((0.6, -0.18), (0.73, -0.225), (0.86, -0.27))):
        _box(F, f'back{k}', 1.7, 0.085, 0.028, (0, y, z), M.wood, bevel=0.008, rot=(-12, 0, 0))
    G.join_all('bench')
    G.settle(centre=False)
    return G.meshes()


# --- the planter and the flower tub: a concrete tub, a little tree or flowers ---------------------------------
def _tub(F):
    """The concrete tub (0.7 m square, 0.42 high) with a lip, and its soil 2 cm below the top."""
    _box(F, 'tub', 0.66, 0.38, 0.66, (0, 0.19, 0), M.concrete, bevel=0.025)
    _box(F, 'lip', 0.7, 0.05, 0.7, (0, 0.395, 0), M.concrete, bevel=0.015)
    _box(F, 'plinth', 0.62, 0.03, 0.62, (0, 0.015, 0), M.concrete)
    _box(F, 'soil', 0.6, 0.02, 0.6, (0, 0.41, 0), M.soil)


def planter():
    F = G.GameFrame()
    _tub(F)
    # a young street tree: a slim trunk (plain satin) forking under a crown of leafy lumps
    tube('trunk', [F.at(0, 0.4, 0), F.at(0.01, 1.0, 0), F.at(-0.01, 1.6, 0.01), F.at(0, 1.85, 0)], 0.036, 0.022, M.trunk,
         verts=low(7, 4))
    for i, (x, z) in enumerate(((0.16, 0.06), (-0.15, -0.1), (0.02, -0.14))):
        tube(f'branch{i}', [F.at(0, 1.5 + i * 0.08, 0), F.at(x, 1.85 + i * 0.08, z)], 0.016, 0.01, M.trunk, verts=low(5, 3))
    crown = [  # radius, bottom (game x, height, depth), material, height share
        (0.36, (0.0, 1.64, 0.0), M.leaf, 0.86),
        (0.24, (0.15, 1.98, 0.08), M.leaf_light, 0.9),
        (0.22, (-0.16, 1.9, -0.12), M.leaf_dark, 0.9),
        (0.2, (0.2, 1.76, -0.13), M.leaf_dark, 0.85),
        (0.2, (-0.2, 1.72, 0.13), M.leaf_light, 0.85),
        (0.18, (0.0, 2.1, -0.02), M.leaf_light, 0.9),
    ]
    for i, (r, (x, y, z), mat, h) in enumerate(crown):
        lump(f'crown{i}', r, F.at(x, y, z), mat, scale=(1, 1, h), noise=0.14, seed=i * 1.7, segments=12, rings=7)
    G.join_all('planter')
    G.settle()
    return G.meshes()


def flowerTub():
    F = G.GameFrame()
    _tub(F)
    rng = random.Random(81)
    # a leafy mound over the soil, and flowers in five colours on it, the placeholder's five where it had them
    lump('mound', 0.27, F.at(0, 0.39, 0), M.leaf, scale=(1, 1, 0.25), noise=0.16, segments=12, rings=7)
    for i, (x, z) in enumerate(((0.2, 0.2), (-0.21, 0.19), (0.21, -0.2), (-0.2, -0.2))):
        lump(f'leaves{i}', 0.11, F.at(x, 0.4, z), [M.leaf_dark, M.leaf_light][i % 2], scale=(1, 1, 0.6), noise=0.2, seed=i * 2.3,
             segments=8, rings=5)
    big = [(-0.15, -0.12, M.red), (0.14, -0.1, M.yellow), (0.0, 0.14, M.pink), (-0.16, 0.16, M.blue), (0.17, 0.15, M.white)]
    cols = [M.red, M.yellow, M.pink, M.blue, M.white, M.violet]
    spots = big + [(rng.uniform(-0.24, 0.24), rng.uniform(-0.24, 0.24), cols[i % len(cols)]) for i in range(9)]
    for i, (x, z, mat) in enumerate(spots):
        r = 0.075 if i < len(big) else 0.05
        y = 0.564 - 2 * r * 0.8 - (0 if i < len(big) else rng.uniform(0.0, 0.03))
        lump(f'bloom{i}', r, F.at(x, y, z), mat, scale=(1, 1, 0.8), noise=0.25, seed=i * 0.9, segments=low(8, 5), rings=low(5, 3))
    G.join_all('flowerTub')
    G.settle()
    return G.meshes()


# --- the street-name sign: two green blades crossing on a pole -----------------------------------------------
def _lettering(F, name, length, y, along_x, face, rng):
    """Words of white-on-green lettering as dark blocks on a blade's white panel, on one face (+1 front, -1 back)."""
    x = -length / 2
    k = 0
    while x < length / 2 - 0.04:
        w = rng.choice((0.03, 0.035, 0.04))
        h = rng.choice((0.06, 0.06, 0.07))
        if rng.random() < 0.14 and k:
            x += 0.04                                     # a space between words
        c = x + w / 2
        at = (c, y, face * 0.019) if along_x else (face * 0.019, y, c)
        size = (w, h, 0.004) if along_x else (0.004, h, w)
        _box(F, f'{name}{k}', *size, at, M.sign)
        x += w + 0.012
        k += 1


def sign():
    F = G.GameFrame()
    rng = random.Random(7)
    F.cyl('pole', 0.035, 2.6, (0, 1.3, 0), M.pole, verts=low(10, 6))
    F.cyl('foot', 0.05, 0.08, (0, 0.04, 0), M.pole, verts=low(10, 6))
    F.sphere('finial', 0.045, (0, 2.62, 0), M.pole, segments=low(10, 6), rings=low(6, 4))
    for y, along_x in ((2.45, True), (2.25, False)):
        w, d = (0.9, 0.03) if along_x else (0.03, 0.9)
        _box(F, f'blade{y}', w, 0.18, d, (0, y, 0), M.sign, bevel=0.008)
        pw, pd = (0.84, 0.032) if along_x else (0.032, 0.84)
        _box(F, f'panel{y}', pw, 0.12, pd, (0, y, 0), M.plate)
        F.cyl(f'clamp{y}', 0.045, 0.2, (0, y, 0), M.pole, verts=low(10, 6))
        if not G.is_low():
            for face in (1, -1):
                _lettering(F, f'letter{y}{face}', 0.62, y, along_x, face, rng)
    G.join_all('sign')
    G.settle()
    return G.meshes()


# --- the bus-stop sign: the round blue flag with a bus on it, and the timetable -------------------------------
def _disc(F, name, r, depth, y, material, verts):
    """A disc facing the street (round about the game's z)."""
    return G.cylinder(name, r, depth, at=F.at(0, y, 0), material=material, rot=(90, 0, 0), verts=verts)


def busSign():
    F = G.GameFrame()
    # the pole stops under the flag (into its rim), so the flag's faces stay whole
    F.cyl('pole', 0.035, 2.2, (0, 1.1, 0), M.pole, verts=low(10, 6))
    F.cyl('foot', 0.038, 0.08, (0, 0.04, 0), M.pole, verts=low(10, 6))
    F.cyl('collar', 0.036, 0.04, (0, 2.15, 0), M.pole, verts=low(10, 6))
    _disc(F, 'flag', 0.23, 0.04, 2.4, M.flag, low(24, 12))
    _disc(F, 'flag_face', 0.17, 0.046, 2.4, M.plate, low(24, 12))
    if not G.is_low():
        for face in (1, -1):                             # a little blue bus on each face
            z = face * 0.024
            _box(F, f'bus{face}', 0.2, 0.085, 0.004, (0, 2.405, z), M.flag)
            for k in range(4):
                _box(F, f'window{face}{k}', 0.032, 0.03, 0.005, (-0.07 + k * 0.045, 2.42, z), M.plate)
            for x in (-0.06, 0.06):
                G.cylinder(f'wheel{face}{x}', 0.017, 0.006, at=F.at(x, 2.36, z), material=M.ink, rot=(90, 0, 0), verts=8)
    # the timetable in its case, clamped to the pole's front
    _box(F, 'case', 0.3, 0.42, 0.03, (0, 1.5, 0.05), M.pole, bevel=0.006)
    _box(F, 'timetable', 0.27, 0.39, 0.004, (0, 1.5, 0.066), M.plate)
    _box(F, 'header', 0.26, 0.06, 0.006, (0, 1.66, 0.067), M.flag)
    if not G.is_low():
        for k in range(7):
            _box(F, f'line{k}', 0.2 - (k % 3) * 0.03, 0.012, 0.005, (-0.02 + (k % 3) * 0.015, 1.59 - k * 0.035, 0.068), M.ink)
    for y in (1.36, 1.64):
        _box(F, f'clamp{y}', 0.09, 0.03, 0.04, (0, y, 0.025), M.pole)
    G.join_all('busSign')
    G.settle(centre=False)
    return G.meshes()


# --- the pot by a front door: terracotta, a clipped leafy ball ------------------------------------------------
def pot():
    F = G.GameFrame()
    n = low(16, 8)
    G.revolve('pot', [(0, 0), (0.15, 0), (0.155, 0.01), (0.18, 0.33), (0.2, 0.335), (0.2, 0.37), (0.17, 0.37),
                      (0.165, 0.35), (0, 0.35)], material=M.terracotta, segments=n)
    G.cylinder('soil', 0.168, 0.01, at=(0, 0, 0.352), material=M.soil, verts=n)
    lump('ball', low(0.205, 0.218), F.at(0, 0.32, 0), M.leaf, scale=(low(1.01, 1.06), 1.005, 1.0), noise=0.1, segments=16, rings=8)
    lump('top', 0.13, F.at(0.08, 0.63, 0.04), M.leaf_light, scale=(1, 1, 0.94), noise=0.15, seed=2.0, segments=10, rings=6)
    lump('side', 0.1, F.at(-0.12, 0.5, -0.03), M.leaf_dark, scale=(1, 1, 0.9), noise=0.15, seed=4.0, segments=8, rings=5)
    G.join_all('pot')
    G.settle()
    return G.meshes()


# --- the garden path light: a dark bollard with a frosted lens under a cap -------------------------------------
def pathLight():
    F = G.GameFrame()
    n = low(12, 8)
    foot = low([(0.07, 0.015), (0.056, 0.025)], [(0.07, 0.02)])
    G.revolve('post', [(0, 0), (0.07, 0), *foot, (0.055, 0.42), (0, 0.42)], material=M.lamp, segments=n)
    F.cyl('lens', 0.062, 0.1, (0, 0.47, 0), M.lens, verts=n)
    for k in range(low(4, 0)):                            # four thin bars round the lens
        a = math.tau * (k + 0.5) / 4
        F.box(f'bar{k}', 0.008, 0.1, 0.008, (0.064 * math.cos(a), 0.47, 0.064 * math.sin(a)), M.lamp)
    rim = low([(0.09, 0.525), (0.045, 0.555)], [(0.09, 0.525)])
    G.revolve('cap', [(0, 0.515), (0.09, 0.515), *rim, (0, 0.56)], material=M.lamp, segments=n)
    G.join_all('pathLight')
    G.settle()
    return G.meshes()


# --- the street foliage --------------------------------------------------------------------------------------
def hedge():
    """A clipped hedge block with a softer, lumpy top (the placeholder's three blobs)."""
    F = G.GameFrame()
    F.box('block', 1.25, 0.84, 0.62, (0, 0.42, 0), M.hedge, bevel=low(0.14, 0.0), bevel_segments=2)
    for i, (x, w, mat) in enumerate(((-0.34, 0.3, M.hedge_dark), (0.0, 0.3, M.hedge), (0.32, 0.32, M.hedge_light))):
        lump(f'top{i}', w, F.at(x, 0.62, (i - 1) * 0.03), mat, scale=(1, 0.28 / w, 0.42 / w * 0.48), noise=0.16, seed=i * 3.1,
             segments=12, rings=7)
    G.join_all('hedge')
    G.settle()
    return G.meshes()


def flowers():
    """A clump of leaves and little flowers in many colours, for the beds along the house fronts."""
    F = G.GameFrame()
    rng = random.Random(44)
    lump('clump', 0.29, F.at(0, 0.0, 0), M.bed, scale=(1, 0.9, 0.45), noise=0.2, segments=16, rings=7)
    lump('clump2', 0.17, F.at(0.12, 0.08, -0.06), M.bed_light, scale=(1, 0.9, 0.6), noise=0.2, seed=1.5, segments=10, rings=6)
    lump('clump3', 0.14, F.at(-0.16, 0.05, 0.08), M.bed_light, scale=(1, 0.9, 0.55), noise=0.2, seed=3.5, segments=8, rings=5)
    cols = [M.rose, M.yellow, M.white, M.violet, M.red, M.pink, M.blue]
    for i in range(14):
        a = i * 2.39996 + rng.uniform(-0.3, 0.3)
        rr = 0.24 * math.sqrt((i + 0.5) / 14)
        x, z = rr * math.cos(a) * 1.05, rr * math.sin(a) * 0.95
        y = 0.295 - (rr / 0.24) ** 2 * 0.07 + rng.uniform(-0.015, 0.015)
        if G.is_low() and i % 2:                        # far off, half the flowers (each drawn, so the rest stand put)
            continue
        lump(f'bloom{i}', 0.036, F.at(x, y, z), cols[i % len(cols)], scale=(1, 1, 0.7), noise=0.25, seed=i * 0.7,
             segments=low(6, 4), rings=low(4, 3))
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
     G.Model('hedge', hedge, size=(1.275, 1.035, 0.62), far=far_of(hedge)),
     G.Model('flowers', flowers, size=(0.6, 0.343, 0.52), far=far_of(flowers))],
    comment='Blender-built street dressing (tools/art/blender/families/street.py): the kerbside bench, the planter '
            'with its little tree, the flower tub, the street-name sign, the bus-stop sign, the door pot, the garden '
            'path light, the street hedge and the flower clump that render/babylon/street.ts draws as thin instances '
            'along the kerbs and front gardens. Replaces the primitive-part placeholders of tools/art/community.py and '
            'street/manifest.json by key; listed in the base manifest\'s packs after them. The placeholder is kept as '
            'a fallback if a file fails to load. Each has a far version for the town overview. Leaves and flowers are '
            'on the foliage finish, so they sway in the wind; the path light\'s lens glows at night.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

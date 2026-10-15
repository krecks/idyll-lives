# Trees, bushes, hedges, lawn tufts, wildflowers and the garden catalog's models (web/public/content/garden.json),
# all built from primitive parts (original, CC0): `model.tree*`, `model.pine*`, `model.spruce*`, `model.cypress*`,
# `model.bush*`, `model.hedge`, `model.grassTuft`, `model.flowers`, `model.flowerbed` and `model.garden.*`.
#
# Crowns are clusters of lumpy `blob`s on the `foliage` finish (a leafy detail texture with a normal map, tinted
# by the part colour): a dark core, clumps spread over one or more ellipsoids, lighter on top and darker below
# (blobs also darken their own undersides), with bark limbs reaching into the crown. Conifers are stacked cones.
# Flowers are small discs, cups and balls over foliage; pots, stones and water are plain finishes.
# `render/babylon/nature.ts` gives the `foliage` and `bark` finishes wind sway and a per-instance tint.
#
# Budgets: at most four material groups per model (textured finishes each, plain finishes together). Trees ~1-1.6k
# triangles (the far woods draw the Blender trees' far versions; with no file, these at lod 1); the lawn's
# grass tuft and wildflowers (thousands of instances) a few dozen. `vary` (placed plants get their own turn and
# size) is kept per key. Same conventions as the other scripts: 1 unit = 1 m, origin at the footprint centre on the
# ground, front +Z; trees overhang their tile.
#
# Run: python3 -I -B nature.py <manifest.json>     replaces these entries in place (others untouched)
#      python3 -I -B nature.py --pack <out.json>   writes them as a pack (preview with `?pack=` in the game or gallery)
import json
import math
import random
import re
import sys

F = 'material.finish.'
E = {}
VARY = {}

# ---- parts ----------------------------------------------------------------------------------------------


def r3(v):
    return [round(x, 3) for x in v]


def part(shape, size, at, color, finish='matte', **kw):
    p = {'shape': shape, 'size': r3(size), 'at': r3(at), 'color': color, 'material': F + finish}
    if 'rotate' in kw:
        kw['rotate'] = [round(a, 1) for a in kw['rotate']]
        if not any(kw['rotate']):
            del kw['rotate']
    p.update({k: v for k, v in kw.items() if v is not None})
    return p


def box(size, at, color, finish='matte', **kw): return part('box', size, at, color, finish, **kw)
def cyl(size, at, color, finish='matte', **kw): return part('cylinder', size, at, color, finish, **kw)
def cone(size, at, color, finish='matte', **kw): return part('cone', size, at, color, finish, **kw)
def sph(size, at, color, finish='matte', **kw): return part('sphere', size, at, color, finish, **kw)
def torus(size, at, color, finish='matte', **kw): return part('torus', size, at, color, finish, **kw)


def blob(size, at, color, finish='foliage', noise=0.28, seg=None, **kw):
    return part('blob', size, at, color, finish, noise=noise, segments=seg, **kw)


def lathe(size, at, color, finish, profile, **kw):
    return part('lathe', size, at, color, finish, profile=[[round(r, 3), round(y, 3)] for r, y in profile], **kw)


# Rotations match Babylon's Quaternion.FromEulerAngles(x, y, z): roll z, then pitch x, then yaw y.
def rot(v, r):
    x, y, z = v
    ax, ay, az = (math.radians(a) for a in r)
    c, s = math.cos(az), math.sin(az)
    x, y = x * c - y * s, x * s + y * c
    c, s = math.cos(ax), math.sin(ax)
    y, z = y * c - z * s, y * s + z * c
    c, s = math.cos(ay), math.sin(ay)
    x, z = x * c + z * s, -x * s + z * c
    return [x, y, z]


def align_y(d):
    """Euler rotation (deg) that turns the local +Y axis to direction d."""
    n = math.sqrt(sum(c * c for c in d))
    dx, dy, dz = (c / n for c in d)
    return [math.degrees(math.acos(max(-1.0, min(1.0, dy)))), math.degrees(math.atan2(dx, dz)), 0]


def add(a, b, k=1.0):
    return [a[i] + b[i] * k for i in range(3)]


def rod(a, b, t, color, finish='bark', taper=None, seg=None, shape='cylinder'):
    """A cylinder of diameter t from point a to point b (`taper`: the end at b, as a fraction)."""
    d = [b[i] - a[i] for i in range(3)]
    length = math.sqrt(sum(c * c for c in d))
    mid = [(a[i] + b[i]) / 2 for i in range(3)]
    return part(shape, [t, length, t], mid, color, finish, rotate=align_y(d), taper=taper, segments=seg)


def facing(tilt, yaw):
    """Rotation and axis of a part tipped `tilt` degrees from upright towards the compass direction `yaw`."""
    r = [tilt, yaw, 0]
    return r, rot([0, 1, 0], r)


def spiral(n, radius, rng, jitter=0.25):
    """n points spread evenly over a disc (sunflower spiral), jittered."""
    ga = math.pi * (3 - math.sqrt(5))
    off = rng.random() * math.tau
    out = []
    for i in range(n):
        rr = radius * math.sqrt((i + 0.5) / n)
        a = off + i * ga + rng.uniform(-jitter, jitter)
        out.append((rr * math.cos(a), rr * math.sin(a), a))
    return out


def pick(cols, t, rng, spread=0.8):
    """A colour from a dark-to-light list: by t in [0, 1] (0 = darkest), jittered."""
    i = int(round(t * (len(cols) - 1) + rng.uniform(-spread, spread)))
    return cols[min(len(cols) - 1, max(0, i))]


# ---- palettes (sRGB; the foliage texture is pale, so leaves are given a little lighter than they look) ----
OAK = ['#558238', '#64923E', '#76A246', '#8AB250', '#9EC25C']
MAPLE = ['#5A8A36', '#6A9A3E', '#7EAA48', '#92BA52', '#A6C85E']
BIRCH = ['#76963E', '#86A646', '#98B650', '#AAC45C', '#BCD26A']
CHERRY = ['#E07AAE', '#EA90BE', '#F2A8CE', '#F8BEDC', '#FCD4E8']
APPLE = ['#52803A', '#629040', '#74A048', '#86AE52', '#98BC5E']
JMAPLE = ['#6E1C1C', '#8A2424', '#A2302A', '#B83F2F', '#CC5636']
MAGNOLIA = ['#EAAACA', '#F2BED6', '#F8D2E2', '#FCE2EE', '#FFF2F8']
WILLOW = ['#6A8A36', '#7A9A3E', '#8AAA48', '#9CBA54', '#AEC862']
PINE = ['#335C38', '#3D683E', '#477444', '#51804A']
SPRUCE = ['#3A5E52', '#446A5C', '#4E7666', '#5A8270']
CYPRESS = ['#2E5034', '#385C3A', '#426842', '#4C744A']
SHRUB = ['#44703A', '#517E3E', '#5E8C44', '#6E9A4A', '#7EA852']
LEAF = ['#4A7A36', '#5A8A3C', '#6A9A44', '#7AA84C']
BARK, BARK_DARK, BARK_GREY = '#E4D8C8', '#B09A8A', '#D2CBC0'
BIRCH_BARK, BIRCH_MARK = '#ECE8DF', '#36322E'
SOIL, SOIL_DARK, MULCH = '#6A4C38', '#47331F', '#6E5240'
TERRACOTTA, TERRACOTTA_DARK = '#C0683F', '#A85634'
STONE, STONE_DARK, WATER, WATER_DEEP = '#CFC9BE', '#A9A39A', '#7FB2C6', '#4F8CA6'
WOOD = '#8A6446'


# ---- building blocks -----------------------------------------------------------------------------------

def crown(rng, c, r, n, cols, size=0.75, spread=0.72, squash=0.85, noise=0.3, seg=3, bottom=-0.45, core=0.0):
    """Leaf clumps over an ellipsoid crown (centre c, radii r): n blobs of about `size` x the mean radius,
    `spread` x the radius out from the centre, lighter towards the top; `core` adds a dark filling blob.
    Clumps are finely tessellated (`seg` 3) for round silhouettes; seg=None leaves them at lod's (lod 1: 20 triangles)."""
    out = []
    m = (r[0] + r[1] + r[2]) / 3
    if core:
        out.append(blob([r[0] * 2 * core, r[1] * 2 * core, r[2] * 2 * core], c, cols[0], noise=noise * 0.8, seg=None if seg is None else 2))
    ga = math.pi * (3 - math.sqrt(5))
    off = rng.random() * math.tau
    for i in range(n):
        y = 1 - (i + 0.5) / n * (1 - bottom)
        rad = math.sqrt(max(0.0, 1 - y * y))
        a = off + ga * i + rng.uniform(-0.35, 0.35)
        k = spread * rng.uniform(0.88, 1.1)
        at = [c[0] + r[0] * rad * math.cos(a) * k, c[1] + r[1] * y * k, c[2] + r[2] * rad * math.sin(a) * k]
        s = size * m * rng.uniform(0.85, 1.18)
        sz = [s * rng.uniform(0.9, 1.15), s * squash * rng.uniform(0.9, 1.1), s * rng.uniform(0.9, 1.15)]
        out.append(blob(sz, at, pick(cols, (y - bottom) / (1 - bottom), rng), noise=noise, seg=seg))
    return out


def surface_dots(rng, c, r, n, size, colors, finish='satin', band=(-0.55, 0.75), out=1.0, seg=2, noise=0.05, shape='blob'):
    """n small things (fruit, flowers) on an ellipsoid's surface between heights band (as fractions of r)."""
    parts = []
    ga = math.pi * (3 - math.sqrt(5))
    off = rng.random() * math.tau
    for i in range(n):
        y = band[1] - (i + 0.5) / n * (band[1] - band[0])
        rad = math.sqrt(max(0.0, 1 - y * y))
        a = off + ga * i + rng.uniform(-0.3, 0.3)
        k = out * rng.uniform(0.95, 1.04)
        at = [c[0] + r[0] * rad * math.cos(a) * k, c[1] + r[1] * y * k, c[2] + r[2] * rad * math.sin(a) * k]
        s = size * rng.uniform(0.85, 1.15)
        col = colors[i % len(colors)]
        if shape == 'blob':
            parts.append(blob([s, s * 1.05, s], at, col, finish, noise=noise, seg=seg))
        else:
            parts.append(sph([s, s, s], at, col, finish, segments=seg))
    return parts


def trunk(h, d, color=BARK, taper=0.62, seg=9, flare=True, finish='bark'):
    p = [cyl([d, h, d], [0, h / 2, 0], color, finish, taper=taper, segments=seg)]
    if flare:
        p.append(cone([d * 1.75, d * 0.9, d * 1.75], [0, d * 0.45, 0], color, finish, taper=0.5, segments=seg))
    return p


def limbs(rng, base, tips, d, color=BARK, seg=6):
    return [rod(base, t, d * rng.uniform(0.85, 1.1), color, 'bark', taper=0.5, seg=seg) for t in tips]


def mound(d=0.84, h=0.12, color=SOIL):
    """A low soil mound under a planting (half below ground)."""
    return [blob([d, h, d * 0.96], [0, 0.0, 0], color, 'matte', noise=0.12)]


def pot(d, h, color=TERRACOTTA, finish='satin', soil=SOIL, y0=0.0, seg=16, rim=True):
    """A flowerpot (lathe) with its soil, standing on y0; returns (parts, soil height)."""
    prof = [[0, -0.5], [0.36, -0.5], [0.47, 0.32]]
    if rim:
        prof += [[0.5, 0.33], [0.5, 0.5], [0.45, 0.5]]
    else:
        prof += [[0.5, 0.5], [0.46, 0.5]]
    prof += [[0.43, 0.36], [0.0, 0.36]]
    top = y0 + h * 0.86
    return [lathe([d, h, d], [0, y0 + h / 2, 0], color, finish, prof, segments=seg),
            cyl([d * 0.86, 0.02, d * 0.86], [0, top, 0], soil, 'matte', segments=seg)], top


def leaf_blade(base, length, width, tilt, yaw, color, finish='foliage', seg=3, thick=None):
    """A strap leaf / grass blade: a thin cone from base, tipped `tilt` degrees towards yaw."""
    r, ax = facing(tilt, yaw)
    return cone([width, length, thick or width * 0.35], add(base, ax, length / 2), color, finish, segments=seg, rotate=r)


def leaf_pad(base, length, width, tilt, yaw, color, thick=0.03, seg=None, noise=0.05, finish='foliage'):
    """A broad flat leaf (flattened blob) reaching from base towards yaw, tipped down by `tilt` from flat."""
    # Long axis along local z, tipped down (positive tilt) and turned to yaw.
    r = [tilt, yaw, 0]
    ax = rot([0, 0, 1], r)
    return blob([width, thick, length], add(base, ax, length / 2), color, finish, noise=noise, seg=seg, rotate=r)


def stem(base, top, d=0.012, color='#4E7A32', seg=4):
    return rod(base, top, d, color, 'foliage', seg=seg)


def flower_disc(at, d, tilt, yaw, petal, centre=None, cd=None, h=0.012, seg=7):
    """A daisy-like head: a petal disc with a raised centre, facing up-and-out."""
    r, ax = facing(tilt, yaw)
    p = [cyl([d, h, d], at, petal, 'satin', segments=seg, rotate=r)]
    if centre:
        cdia = cd or d * 0.36
        p.append(cyl([cdia, h * 1.6, cdia], add(at, ax, h * 0.5), centre, 'satin', segments=5, rotate=r))
    return p


def cup(at, d, h, tilt, yaw, color, flare=1.35, seg=6, finish='satin'):
    """A flower cup (tulip, poppy, trumpet): narrow at the base, opening up along its axis."""
    r, ax = facing(tilt, yaw)
    return cyl([d, h, d], add(at, ax, h / 2), color, finish, taper=flare, segments=seg, rotate=r)


def model(key, parts, vary=None):
    E[key] = parts
    if vary:
        VARY[key] = vary


# =========================================================================================================
# Trees (forest, lot and garden)
# =========================================================================================================

def oak():
    rng = random.Random(11)
    p = trunk(3.4, 0.62)
    p += limbs(rng, [0, 2.9, 0], [[1.5, 4.6, 0.5], [-1.4, 4.5, -0.6], [0.3, 5.0, -1.3], [-0.4, 5.1, 1.2]], 0.3)
    p += crown(rng, [0, 5.5, 0], [2.7, 1.8, 2.7], 10, OAK, size=0.7, spread=0.71, core=0.82)
    p += crown(rng, [1.7, 4.6, 0.7], [1.4, 1.0, 1.4], 3, OAK, size=0.85, bottom=-0.6, core=0.7)
    p += crown(rng, [-1.6, 4.7, -0.8], [1.4, 1.1, 1.4], 3, OAK, size=0.85, bottom=-0.6, core=0.7)
    return p


def maple():
    rng = random.Random(12)
    p = trunk(3.2, 0.5)
    p += limbs(rng, [0, 2.8, 0], [[1.1, 4.3, 0.4], [-1.0, 4.4, -0.3], [0.1, 4.6, -1.1]], 0.24)
    p += crown(rng, [0, 5.1, 0], [2.5, 2.4, 2.5], 14, MAPLE, size=0.6, spread=0.74, core=0.85)
    return p


def birch():
    rng = random.Random(13)
    p = [cyl([0.3, 7.2, 0.3], [0, 3.6, 0], BIRCH_BARK, 'matte', taper=0.42, segments=8),
         cone([0.46, 0.4, 0.46], [0, 0.2, 0], '#5A524A', 'bark', taper=0.6, segments=8)]
    for i in range(9):
        y = 0.6 + i * 0.55 + rng.uniform(-0.15, 0.15)
        a = rng.uniform(0, 360)
        rr = 0.15 - y * 0.012
        p.append(box([0.11, 0.035, 0.03], [rr * math.sin(math.radians(a)), y, rr * math.cos(math.radians(a))], BIRCH_MARK, 'matte', radius=0, rotate=[0, a + 90, 0]))
    p += limbs(rng, [0, 3.6, 0], [[0.9, 5.0, 0.3], [-0.8, 5.2, -0.3], [0.2, 5.6, 0.9], [-0.3, 4.9, -0.9]], 0.1, BIRCH_BARK, seg=5)
    p += crown(rng, [0, 5.8, 0], [1.6, 2.4, 1.6], 13, BIRCH, size=0.52, spread=0.8, noise=0.34, core=0.72)
    return p


def blossom():
    rng = random.Random(14)
    p = trunk(1.9, 0.45, BARK_DARK, taper=0.7)
    p += limbs(rng, [0, 1.7, 0], [[1.6, 3.2, 0.5], [-1.5, 3.1, -0.4], [0.3, 3.4, -1.5], [-0.4, 3.3, 1.4]], 0.22, BARK_DARK)
    p += crown(rng, [0, 3.9, 0], [2.7, 1.45, 2.7], 14, CHERRY, size=0.6, squash=0.75, noise=0.32, spread=0.78, core=0.8, bottom=-0.55)
    p += [blob([0.8, 0.5, 0.8], [2.0, 3.5, -0.9], '#6E9A44', noise=0.3), blob([0.7, 0.45, 0.7], [-1.8, 3.6, 1.2], '#62903E', noise=0.3)]
    return p


def apple():
    rng = random.Random(15)
    p = trunk(2.1, 0.4, taper=0.7)
    p += limbs(rng, [0, 1.9, 0], [[1.3, 3.1, 0.4], [-1.2, 3.0, -0.4], [0.2, 3.3, -1.3], [-0.2, 3.2, 1.2]], 0.2)
    p += crown(rng, [0, 3.6, 0], [2.2, 1.65, 2.2], 12, APPLE, size=0.64, spread=0.76, core=0.84)
    p += surface_dots(rng, [0, 3.6, 0], [2.2, 1.65, 2.2], 18, 0.2, ['#C8352B', '#D8402C', '#B42A26', '#E2B03C'], band=(-0.65, 0.55), out=0.98)
    return p


def japanese_maple():
    rng = random.Random(16)
    p = limbs(rng, [0, 0, 0], [[0.7, 1.7, 0.3], [-0.6, 1.6, -0.4], [0.1, 1.9, -0.7], [-0.2, 1.5, 0.7]], 0.17, BARK_DARK)
    p.append(cone([0.42, 0.3, 0.42], [0, 0.15, 0], BARK_DARK, 'bark', taper=0.5, segments=7))
    p += crown(rng, [0, 2.0, 0], [1.95, 0.6, 1.95], 9, JMAPLE, size=0.66, squash=0.68, noise=0.32, spread=0.8, core=0.75)
    p += crown(rng, [0.2, 2.7, -0.1], [1.45, 0.55, 1.45], 7, JMAPLE[1:], size=0.7, squash=0.68, noise=0.32, spread=0.8, core=0.7)
    p += crown(rng, [-0.1, 3.2, 0.1], [0.85, 0.4, 0.85], 3, JMAPLE[2:], size=0.9, squash=0.7, noise=0.3)
    return p


def magnolia():
    rng = random.Random(17)
    p = trunk(1.6, 0.36, BARK_GREY, taper=0.7)
    p += limbs(rng, [0, 1.4, 0], [[1.2, 3.0, 0.3], [-1.1, 3.1, -0.4], [0.2, 3.5, -1.0], [-0.3, 3.3, 1.0], [0.1, 3.9, 0.1]], 0.16, BARK_GREY)
    p += crown(rng, [0, 3.4, 0], [1.9, 1.5, 1.8], 11, MAGNOLIA, size=0.58, spread=0.8, noise=0.3, core=0.7)
    p += [blob([0.6, 0.45, 0.6], [1.5, 2.9, 0.8], '#6E9A48', noise=0.3), blob([0.55, 0.4, 0.55], [-1.3, 3.6, -0.9], '#628E40', noise=0.3)]
    # Big blooms standing out of the crown.
    p += surface_dots(rng, [0, 3.45, 0], [1.95, 1.55, 1.85], 16, 0.24, ['#FFFFFF', '#F2A2C4', '#E888B0', '#FFFFFF'], band=(-0.3, 0.95), out=0.98, seg=2, noise=0.3)
    return p


def willow():
    rng = random.Random(18)
    p = trunk(2.7, 0.62, taper=0.65)
    tips = [[2.0 * math.cos(a), 4.3, 2.0 * math.sin(a)] for a in (0.3, 1.6, 2.9, 4.2, 5.4)]
    p += limbs(rng, [0, 2.4, 0], tips, 0.24)
    p += crown(rng, [0, 4.7, 0], [2.3, 1.4, 2.3], 9, WILLOW, size=0.68, squash=0.7, core=0.85, bottom=-0.3)
    # Hanging curtains: long strands narrowing downwards all round, down to about a metre off the ground.
    for i in range(26):
        a = i / 26 * math.tau + rng.uniform(-0.1, 0.1)
        rr = rng.uniform(1.75, 2.2)
        length = rng.uniform(2.4, 3.5)
        top = rng.uniform(4.6, 5.0)
        p.append(cone([rng.uniform(0.5, 0.7), length, 0.4], [rr * math.cos(a), top - length / 2, rr * math.sin(a)], pick(WILLOW, rng.random(), rng, 1.2),
                      'foliage', segments=6, rotate=[180 + rng.uniform(-6, 6), -math.degrees(a) + 90, 0]))
    for i in range(8):
        a = i / 8 * math.tau + 0.3
        p.append(cone([0.8, 2.6, 0.6], [1.35 * math.cos(a), 3.7, 1.35 * math.sin(a)], WILLOW[0], 'foliage', segments=6, rotate=[180, -math.degrees(a) + 90, 0]))
    return p



def conifer(rng, tiers, base_d, y0, step, h, cols, top_d=0.5, tilt=4.0, offset=0.12, seg=11):
    """Stacked cones from y0 (the first tier's bottom), each `step` above the last, shrinking to top_d."""
    p = []
    for i in range(tiers):
        t = i / max(1, tiers - 1)
        d = base_d + (top_d - base_d) * t ** 0.9
        hh = h * (1 - 0.35 * t)
        y = y0 + i * step + hh / 2
        p.append(cone([d * rng.uniform(0.94, 1.06), hh, d * rng.uniform(0.94, 1.06)], [rng.uniform(-offset, offset), y, rng.uniform(-offset, offset)],
                      cols[(i + rng.randint(0, 1)) % len(cols)], 'foliage', segments=seg, rotate=[rng.uniform(-tilt, tilt), rng.uniform(0, 360), rng.uniform(-tilt, tilt)]))
    return p


def pine():
    rng = random.Random(19)
    p = trunk(3.4, 0.38, '#B59A82', taper=0.55, seg=8)
    p += conifer(rng, 6, 5.6, 1.4, 1.12, 2.1, PINE, top_d=0.9, tilt=6, offset=0.2)
    p.append(cone([0.5, 1.3, 0.5], [0, 8.2, 0], PINE[2], 'foliage', segments=8))
    return p


def spruce():
    rng = random.Random(20)
    p = trunk(1.2, 0.36, '#8E7A6A', seg=7, flare=False)
    p += conifer(rng, 9, 5.4, 0.35, 0.98, 1.85, SPRUCE, top_d=0.7, tilt=3, offset=0.08)
    p.append(cone([0.42, 1.2, 0.42], [0, 9.6, 0], SPRUCE[3], 'foliage', segments=8))
    return p


def cypress():
    rng = random.Random(21)
    p = [cyl([0.28, 0.8, 0.28], [0, 0.4, 0], '#8E7A6A', 'bark', taper=0.8, segments=7)]
    for i, (d, h, y) in enumerate([(2.0, 2.4, 1.6), (1.9, 2.4, 2.9), (1.7, 2.3, 4.1), (1.4, 2.1, 5.3), (1.05, 1.8, 6.3)]):
        p.append(blob([d * rng.uniform(0.95, 1.05), h, d * rng.uniform(0.95, 1.05)], [rng.uniform(-0.06, 0.06), y, rng.uniform(-0.06, 0.06)],
                      CYPRESS[i % 4], noise=0.18, rotate=[0, rng.uniform(0, 360), 0]))
    for i in range(6):
        a = i / 6 * math.tau
        y = 1.6 + i * 0.85
        rr = 0.75 - i * 0.07
        p.append(blob([0.7, 1.2, 0.7], [rr * math.cos(a), y, rr * math.sin(a)], CYPRESS[(i + 2) % 4], noise=0.25))
    p.append(cone([0.6, 1.3, 0.6], [0, 7.3, 0], CYPRESS[3], 'foliage', segments=8))
    return p


for key, fn in [('model.tree', oak), ('model.tree.maple', maple), ('model.tree.birch', birch), ('model.tree.blossom', blossom),
                ('model.tree.fruit', apple), ('model.tree.japaneseMaple', japanese_maple), ('model.tree.magnolia', magnolia),
                ('model.tree.willow', willow), ('model.pine', pine), ('model.spruce', spruce), ('model.cypress', cypress)]:
    model(key, fn(), 0.12)


# =========================================================================================================
# Bushes, hedges and the lawn
# =========================================================================================================

def bush(rng, n, w, h, cols, top=None):
    """A rounded shrub w wide and h tall: a dark core and n clumps around and on top."""
    p = [blob([w * 0.8, h * 0.75, w * 0.8], [0, h * 0.38, 0], cols[0], noise=0.25)]
    for i in range(n):
        a = i / n * math.tau + rng.uniform(-0.3, 0.3)
        rr = w * 0.26 * rng.uniform(0.8, 1.1)
        s = w * 0.5 * rng.uniform(0.85, 1.1)
        y = h * rng.uniform(0.38, 0.55)
        p.append(blob([s, s * 0.85, s], [rr * math.cos(a), y, rr * math.sin(a)], pick(cols, 0.4 + 0.5 * (y / h), rng), noise=0.3))
    s = w * 0.5
    p.append(blob([s, s * 0.8, s], [rng.uniform(-0.05, 0.05), h * 0.72, rng.uniform(-0.05, 0.05)], top or cols[-1], noise=0.3))
    return p


model('model.bush', bush(random.Random(31), 5, 1.15, 1.0, SHRUB), 0.12)
model('model.bush.small', bush(random.Random(32), 4, 0.9, 0.75, SHRUB[:4]))
rng = random.Random(33)
model('model.bush.flowering', bush(rng, 5, 1.15, 1.0, SHRUB)
      + surface_dots(rng, [0, 0.52, 0], [0.56, 0.48, 0.56], 14, 0.17, ['#F08CBC', '#F8B4D4', '#FFF0F6', '#E870A8'], 'foliage', band=(-0.2, 0.95), noise=0.25))


def hedge(length, h, depth):
    """A clipped hedge segment along x (rows overlap: street.ts spaces them ~1.05 m apart)."""
    rng = random.Random(int(length * 100 + h * 10))
    p = [box([length, h * 0.9, depth], [0, h * 0.45, 0], SHRUB[1], 'foliage', radius=min(0.2, depth * 0.3))]
    for i, x in enumerate((-length * 0.3, 0.02, length * 0.3)):
        p.append(blob([length * 0.44, h * 0.22, depth * 0.86], [x, h * 0.86, rng.uniform(-0.03, 0.03)], SHRUB[2 + i % 2], noise=0.12))
    return p


model('model.hedge', hedge(1.55, 1.0, 0.85))


def grass_tuft():
    rng = random.Random(41)
    cols = ['#7C9E44', '#8AAE4C', '#98BA56', '#84A648']
    return [leaf_blade([rng.uniform(-0.04, 0.04), 0, rng.uniform(-0.04, 0.04)], rng.uniform(0.2, 0.32), 0.05, rng.uniform(8, 28), i * 120 + rng.uniform(-25, 25), cols[i % 4])
            for i in range(3)]


model('model.grassTuft', grass_tuft())


def wildflowers():
    rng = random.Random(42)
    p = [leaf_blade([0, 0, 0], 0.16, 0.05, 30, a, '#5E8436') for a in (20, 150, 260)]
    for (x, z), h, col in [((0.03, 0.02), 0.26, '#F4F1EA'), ((-0.04, 0.03), 0.22, '#F2C14E'), ((0.0, -0.05), 0.19, '#9A78C8')]:
        p.append(stem([x * 0.3, 0, z * 0.3], [x, h, z], 0.01, '#4F7A30', seg=3))
        p += flower_disc([x, h, z], 0.06, rng.uniform(0, 25), rng.uniform(0, 360), col, '#E8B436' if col != '#F2C14E' else '#C77A22', seg=5)
    return p


model('model.flowers', wildflowers())


# =========================================================================================================
# Garden: flowers
# =========================================================================================================

def leaves_low(rng, n, radius, cols=LEAF, size=0.22, h=0.12, y=0.08):
    """Low leafy clumps between the flowers."""
    return [blob([size * rng.uniform(0.8, 1.2), h * rng.uniform(0.8, 1.2), size * rng.uniform(0.8, 1.2)], [x, y, z], cols[i % len(cols)], noise=0.28)
            for i, (x, z, _) in enumerate(spiral(n, radius, rng))]


def tulips():
    rng = random.Random(51)
    p = mound()
    cols = ['#D7263D', '#F2C12E', '#EF6F9C', '#F08A24', '#D7263D', '#F4F1EA']
    for i, (x, z, a) in enumerate(spiral(17, 0.36, rng)):
        h = rng.uniform(0.36, 0.52)
        lean = [x + rng.uniform(-0.03, 0.03), h, z + rng.uniform(-0.03, 0.03)]
        p.append(stem([x, 0.04, z], lean, 0.014))
        p.append(lathe([0.08, 0.1, 0.08], add(lean, [0, 0.04, 0]), cols[i % len(cols)], 'satin',
                       [[0, -0.5], [0.36, -0.36], [0.5, 0.05], [0.44, 0.5], [0, 0.4]], segments=6))
        if i % 2 == 0:
            p.append(leaf_blade([x, 0.04, z], 0.24, 0.07, 28, math.degrees(a) + 40, LEAF[2], seg=4))
    return p


def daffodils():
    rng = random.Random(52)
    p = mound()
    for i, (x, z, a) in enumerate(spiral(15, 0.36, rng)):
        h = rng.uniform(0.3, 0.44)
        top = [x, h, z]
        p.append(stem([x, 0.04, z], top, 0.012, '#5E8C3A'))
        yaw = math.degrees(a) + rng.uniform(-40, 40)
        r, ax = facing(70, yaw)
        p.append(cyl([0.1, 0.012, 0.1], top, '#FAE68A', 'satin', segments=6, rotate=r))
        p.append(cyl([0.04, 0.05, 0.04], add(top, ax, 0.028), '#F6AE1C', 'satin', taper=1.4, segments=6, rotate=r))
        p.append(leaf_blade([x, 0.04, z], 0.3, 0.05, 14, yaw + 150, LEAF[2 + i % 2], seg=4))
    return p


def daisies():
    rng = random.Random(53)
    p = mound() + leaves_low(rng, 7, 0.32, size=0.24, h=0.14, y=0.09)
    for i, (x, z, a) in enumerate(spiral(24, 0.4, rng)):
        h = rng.uniform(0.22, 0.34)
        p.append(stem([x, 0.08, z], [x, h, z], 0.01, '#5E8C3A', seg=3))
        p += flower_disc([x, h, z], 0.09, rng.uniform(0, 30), rng.uniform(0, 360), '#FFFFFF', '#F6C21E', seg=8)
    return p


def sunflowers():
    rng = random.Random(54)
    p = mound()
    for i, (x, z) in enumerate([(-0.18, -0.12), (0.17, -0.06), (0.0, 0.18), (-0.04, -0.3)]):
        h = [1.85, 1.6, 1.4, 1.95][i]
        top = [x * 1.2, h, z * 1.2 + 0.04]
        p.append(rod([x, 0, z], top, 0.04, '#5E8C3A', 'foliage', seg=5))
        for k in range(3):
            t = 0.3 + k * 0.2
            base = [x + (top[0] - x) * t, h * t, z + (top[2] - z) * t]
            p.append(leaf_pad(base, 0.24, 0.18, 25, rng.uniform(0, 360) + k * 120, LEAF[1 + k % 3], seg=1))
        yaw = rng.uniform(-35, 35)
        r, ax = facing(62, yaw)
        p.append(cyl([0.34, 0.03, 0.34], top, '#F5C518', 'satin', segments=12, rotate=r))
        p.append(cyl([0.16, 0.05, 0.16], add(top, ax, 0.02), '#5A3A1E', 'matte', segments=10, rotate=r))
    return p


def poppies():
    rng = random.Random(55)
    p = mound() + leaves_low(rng, 6, 0.3, ['#5E8456', '#6E9464'], size=0.22, h=0.12, y=0.08)
    for i, (x, z, a) in enumerate(spiral(16, 0.4, rng)):
        h = rng.uniform(0.38, 0.6)
        top = [x + rng.uniform(-0.04, 0.04), h, z + rng.uniform(-0.04, 0.04)]
        p.append(stem([x, 0.05, z], top, 0.009, '#6E9464', seg=3))
        tilt, yaw = rng.uniform(0, 30), rng.uniform(0, 360)
        p.append(cup(top, 0.06, 0.05, tilt, yaw, ['#EC3A2E', '#F2582E', '#E02C34'][i % 3], flare=1.9, seg=6))
        if i % 3 == 0:
            r, ax = facing(tilt, yaw)
            p.append(cyl([0.024, 0.012, 0.024], add(top, ax, 0.04), '#1E1A1A', 'matte', segments=5, rotate=r))
    return p


def marigolds():
    rng = random.Random(56)
    p = mound() + leaves_low(rng, 7, 0.33, ['#2F5A28', '#3A6A2E', '#467634'], size=0.26, h=0.16, y=0.1)
    for i, (x, z, a) in enumerate(spiral(18, 0.4, rng)):
        y = rng.uniform(0.19, 0.28)
        p.append(blob([0.11, 0.08, 0.11], [x, y, z], ['#F8901C', '#FCBA1E', '#F0761A'][i % 3], 'foliage', noise=0.3))
    return p


def pansies():
    rng = random.Random(57)
    p = mound() + leaves_low(rng, 7, 0.34, size=0.22, h=0.1, y=0.07)
    cols = [('#5B3A8E', '#F2C94C'), ('#F2C94C', '#3A2A4E'), ('#3F5FB0', '#F2E8A0'), ('#8E2A4A', '#F2C94C'), ('#F4F0E8', '#6E4FB8')]
    for i, (x, z, a) in enumerate(spiral(15, 0.38, rng)):
        h = rng.uniform(0.13, 0.2)
        petal, centre = cols[i % len(cols)]
        p += flower_disc([x, h, z], 0.09, rng.uniform(35, 60), math.degrees(a) + rng.uniform(-30, 30), petal, centre, cd=0.035, seg=6)
    return p


def lilies():
    rng = random.Random(58)
    p = mound()
    cols = ['#F07A2A', '#F5EFE4', '#E98AA8']
    for i, (x, z, a) in enumerate(spiral(6, 0.28, rng)):
        h = rng.uniform(0.7, 0.92)
        top = [x * 1.15, h, z * 1.15]
        p.append(stem([x, 0, z], top, 0.016))
        for k in range(3):
            t = 0.25 + k * 0.2
            p.append(leaf_blade([x * (1 + 0.15 * t), h * t, z * (1 + 0.15 * t)], 0.22, 0.04, 55, rng.uniform(0, 360), LEAF[2], seg=3))
        for k in range(2):
            yaw = math.degrees(a) + (k * 2 - 1) * 50 + rng.uniform(-15, 15)
            p.append(cup(add(top, [0, -0.03 * k, 0]), 0.03, 0.13, 55, yaw, cols[i % 3], flare=4.2, seg=6))
    return p


def lupins():
    rng = random.Random(59)
    p = mound() + leaves_low(rng, 6, 0.3, size=0.26, h=0.16, y=0.1)
    cols = ['#8E6CE0', '#F08CC4', '#FFFFFF', '#6C8CF0', '#D068C0', '#A080F0', '#F8A8D4']
    for i, (x, z, a) in enumerate(spiral(9, 0.32, rng)):
        h = rng.uniform(0.62, 0.98)
        top = [x + rng.uniform(-0.03, 0.03), h, z + rng.uniform(-0.03, 0.03)]
        p.append(stem([x, 0.1, z], add(top, [0, -0.3, 0]), 0.016))
        p.append(cone([0.11, 0.4, 0.11], add(top, [0, -0.16, 0]), cols[i % len(cols)], 'foliage', taper=0.15, segments=7))
    return p


def forget_me_nots():
    rng = random.Random(60)
    p = mound() + leaves_low(rng, 7, 0.33, size=0.28, h=0.16, y=0.08)
    for i, (x, z, a) in enumerate(spiral(16, 0.38, rng)):
        p.append(blob([0.12, 0.05, 0.12], [x, rng.uniform(0.16, 0.22), z], ['#8CB4F8', '#A4C4FC', '#7AA0F0'][i % 3], 'foliage', noise=0.3))
    return p


def cosmos():
    rng = random.Random(61)
    p = mound()
    for i, (x, z, a) in enumerate(spiral(9, 0.3, rng)):
        p.append(leaf_blade([x, 0.03, z], rng.uniform(0.35, 0.5), 0.05, rng.uniform(10, 30), rng.uniform(0, 360), ['#6E9E44', '#7EAA4C'][i % 2]))
    cols = ['#E77FB0', '#F6EEF2', '#C2367A', '#F2A6CA']
    for i, (x, z, a) in enumerate(spiral(15, 0.38, rng)):
        h = rng.uniform(0.55, 0.88)
        top = [x * 1.1, h, z * 1.1]
        p.append(stem([x, 0, z], top, 0.009, '#6E9E44', seg=3))
        p += flower_disc(top, 0.1, rng.uniform(10, 40), rng.uniform(0, 360), cols[i % 4], '#F2C12E', seg=8)
    return p


def meadow():
    rng = random.Random(62)
    cols = ['#7FA04A', '#8EAE52', '#6E9240', '#A2B45E']
    p = [blob([0.3, 0.16, 0.3], [x, 0.03, z], cols[i % 4], noise=0.3) for i, (x, z, _) in enumerate(spiral(5, 0.3, rng))]
    for i, (x, z, a) in enumerate(spiral(22, 0.42, rng)):
        p.append(leaf_blade([x, 0, z], rng.uniform(0.28, 0.5), 0.05, rng.uniform(5, 30), rng.uniform(0, 360), cols[i % 4]))
    flowers = ['#FFFFFF', '#F6C21E', '#A880E0', '#EC3A2E', '#8CB4F8', '#FFFFFF', '#F08CC4']
    for i, (x, z, a) in enumerate(spiral(16, 0.4, rng)):
        h = rng.uniform(0.26, 0.5)
        p.append(stem([x, 0, z], [x, h, z], 0.009, '#5E8C3A', seg=3))
        p += flower_disc([x, h, z], 0.075, rng.uniform(0, 30), rng.uniform(0, 360), flowers[i % len(flowers)], '#E8B436', seg=6)
    return p


def geraniums():
    rng = random.Random(63)
    p, top = pot(0.4, 0.34)
    p += [blob([0.3, 0.2, 0.3], [x, top + 0.09, z], LEAF[i % 3], noise=0.3) for i, (x, z, _) in enumerate(spiral(4, 0.12, rng))]
    for i, (x, z, a) in enumerate(spiral(6, 0.15, rng)):
        h = top + rng.uniform(0.2, 0.3)
        p.append(stem([x * 0.5, top, z * 0.5], [x, h, z], 0.01, '#4E7A32', seg=3))
        p.append(blob([0.11, 0.09, 0.11], [x, h + 0.03, z], ['#D6283A', '#E23A48', '#C41E32'][i % 3], 'satin', noise=0.35, seg=1))
    return p


def flower_box():
    rng = random.Random(64)
    white = '#ECE9E1'
    p = [box([0.9, 0.26, 0.42], [0, 0.45, 0], white, 'wood', radius=0.012)]
    for x in (-0.4, 0.4):
        for z in (-0.17, 0.17):
            p.append(box([0.055, 0.34, 0.055], [x, 0.17, z], white, 'wood', radius=0.008))
    p.append(box([0.84, 0.02, 0.36], [0, 0.575, 0], SOIL, 'matte', radius=0))
    for i in range(6):
        x = -0.36 + i * 0.144
        p.append(blob([0.2, 0.15, 0.24], [x, 0.63, rng.uniform(-0.05, 0.05)], LEAF[i % 3], noise=0.3))
    cols = ['#E85A9A', '#F4F0F2', '#9A5AC8', '#E85A9A', '#F2C14E']
    for i in range(14):
        x, z = rng.uniform(-0.4, 0.4), rng.uniform(-0.15, 0.17)
        p.append(blob([0.07, 0.05, 0.07], [x, rng.uniform(0.66, 0.72), z], cols[i % len(cols)], 'satin', noise=0.25, seg=1))
    for x in (-0.3, 0.05, 0.32):                                                                  # trailing stems
        p.append(blob([0.12, 0.22, 0.08], [x, 0.5, 0.23], LEAF[2], noise=0.3))
    return p


def flowerbed():
    rng = random.Random(65)
    p = []
    for z in (-0.44, 0.44):
        p.append(box([0.94, 0.24, 0.06], [0, 0.12, z], WOOD, 'wood', radius=0.012))
    for x in (-0.44, 0.44):
        p.append(box([0.06, 0.24, 0.82], [x, 0.12, 0], WOOD, 'wood', radius=0.012))
    p.append(box([0.84, 0.03, 0.84], [0, 0.22, 0], SOIL, 'matte', radius=0))
    p += leaves_low(rng, 8, 0.32, size=0.22, h=0.16, y=0.3)
    cols = ['#E8873A', '#8E6BBF', '#F0EDE6', '#D9483B', '#F2C14E', '#C2477A']
    for i, (x, z, a) in enumerate(spiral(18, 0.36, rng)):
        p.append(blob([0.075, 0.055, 0.075], [x, rng.uniform(0.38, 0.46), z], cols[i % len(cols)], 'satin', noise=0.25, seg=1))
    return p


for key, fn in [('tulips', tulips), ('daffodils', daffodils), ('daisies', daisies), ('sunflowers', sunflowers), ('poppies', poppies),
                ('marigolds', marigolds), ('pansies', pansies), ('lilies', lilies), ('lupins', lupins), ('forgetMeNots', forget_me_nots),
                ('cosmos', cosmos), ('meadow', meadow)]:
    model('model.garden.' + key, fn(), 0.1)
model('model.garden.geraniums', geraniums())
model('model.garden.flowerBox', flower_box())
model('model.flowerbed', flowerbed())


# =========================================================================================================
# Garden: shrubs and hedges
# =========================================================================================================

def boxwood():
    return [blob([0.72, 0.66, 0.72], [0, 0.36, 0], '#3E6B33', noise=0.07, seg=3),
            blob([0.3, 0.2, 0.3], [0.12, 0.62, -0.05], '#4C7A3A', noise=0.15),
            blob([0.26, 0.2, 0.26], [-0.2, 0.5, 0.15], '#47753A', noise=0.15)]


def topiary_cone():
    p, top = pot(0.5, 0.36, '#B9B2A6', 'matte')
    p += [cyl([0.06, 0.25, 0.06], [0, top + 0.1, 0], BARK, 'bark', segments=6),
          cone([0.66, 1.3, 0.66], [0, top + 0.82, 0], '#3E6B33', 'foliage', taper=0.04, segments=14),
          cone([0.5, 0.9, 0.5], [0.02, top + 0.95, -0.02], '#4A7838', 'foliage', taper=0.04, segments=9, rotate=[0, 20, 0])]
    return p


def hydrangea():
    rng = random.Random(72)
    return bush(rng, 5, 1.05, 0.85, SHRUB[:4]) + surface_dots(rng, [0, 0.48, 0], [0.5, 0.42, 0.5], 11, 0.3, ['#9CB8F4', '#B8A6F0', '#8CA8EC', '#D4BCF4'],
                                                             finish='foliage', band=(-0.3, 0.95), noise=0.22)


def rosebush():
    rng = random.Random(73)
    return bush(rng, 5, 0.9, 0.9, ['#2C4C26', '#365A2C', '#3F6632', '#4A7038']) + surface_dots(
        rng, [0, 0.5, 0], [0.45, 0.42, 0.45], 16, 0.11, ['#C8203A', '#DC3448', '#E84A5E', '#B8182E'], band=(-0.35, 0.95), noise=0.25, seg=1)


def azalea():
    rng = random.Random(74)
    return bush(rng, 6, 1.15, 0.95, SHRUB[:4]) + surface_dots(rng, [0, 0.5, 0], [0.56, 0.46, 0.56], 20, 0.24, ['#F0549C', '#F878B4', '#E04090', '#FC9CC8'],
                                                             'foliage', band=(-0.25, 0.95), noise=0.25)


def lavender():
    rng = random.Random(75)
    p = [blob([0.55, 0.32, 0.55], [0, 0.14, 0], '#748C70', noise=0.25), blob([0.4, 0.3, 0.4], [0.12, 0.22, 0.05], '#82987C', noise=0.25)]
    for i, (x, z, a) in enumerate(spiral(24, 0.24, rng)):
        r, ax = facing(rng.uniform(4, 22) * (0.4 + math.hypot(x, z) * 3), math.degrees(math.atan2(x, z)))
        base = [x * 0.5, 0.2, z * 0.5]
        length = rng.uniform(0.38, 0.55)
        p.append(cyl([0.012, length, 0.012], add(base, ax, length / 2), '#7E947A', 'foliage', segments=3, rotate=r))
        p.append(cone([0.04, 0.17, 0.04], add(base, ax, length + 0.06), ['#7E64B8', '#8F74C8', '#6E58A8'][i % 3], 'satin', taper=0.2, segments=4, rotate=r))
    return p


def fern():
    rng = random.Random(76)
    p = [blob([0.18, 0.12, 0.18], [0, 0.06, 0], '#4A6A2E', noise=0.2)]
    for i in range(9):
        yaw = i / 9 * 360 + rng.uniform(-12, 12)
        p.append(leaf_pad([0, 0.12, 0], rng.uniform(0.42, 0.55), 0.17, -rng.uniform(22, 38), yaw, ['#5E9A3E', '#6FAA46', '#558E38'][i % 3], seg=2, noise=0.12))
        p.append(leaf_pad([0, 0.2, 0], rng.uniform(0.3, 0.38), 0.14, -rng.uniform(45, 60), yaw + 20, '#78B24E', seg=1, noise=0.12))
    return p


def ornamental_grass():
    rng = random.Random(77)
    cols = ['#9AAA5A', '#B2B86A', '#8A9E4E', '#A6B062']
    p = [blob([0.32, 0.14, 0.32], [0, 0.04, 0], '#7E8E46', noise=0.2)]
    for i, (x, z, a) in enumerate(spiral(22, 0.12, rng)):
        p.append(leaf_blade([x, 0, z], rng.uniform(0.75, 1.1), 0.045, 12 + math.hypot(x, z) * 170 + rng.uniform(0, 8), math.degrees(math.atan2(x, z)) + rng.uniform(-20, 20), cols[i % 4]))
    for i in range(6):
        yaw = i * 60 + rng.uniform(-15, 15)
        r, ax = facing(rng.uniform(10, 20), yaw)
        p.append(cyl([0.012, 1.0, 0.012], add([0, 0, 0], ax, 0.5), '#B8B070', 'foliage', segments=3, rotate=r))
        p.append(cone([0.06, 0.24, 0.06], add([0, 0, 0], ax, 1.05), '#E2D6A8', 'satin', taper=0.2, segments=5, rotate=r))
    return p


def hosta():
    rng = random.Random(78)
    p = [blob([0.3, 0.14, 0.3], [0, 0.06, 0], '#4E7A4A', noise=0.2)]
    for i in range(12):
        yaw = i / 12 * 360 * 2 + rng.uniform(-10, 10)
        up = i >= 6
        p.append(leaf_pad([0, 0.1 + (0.08 if up else 0), 0], rng.uniform(0.33, 0.39), 0.2, -rng.uniform(25, 40) if up else -rng.uniform(5, 15), yaw,
                          ['#5E8A5E', '#8EAE6C', '#6A9464'][i % 3], seg=2, noise=0.08))
    for i in range(3):
        top = [rng.uniform(-0.08, 0.08), rng.uniform(0.62, 0.8), rng.uniform(-0.08, 0.08)]
        p.append(stem([0, 0.1, 0], top, 0.012, '#6A9464', seg=3))
        p.append(cone([0.05, 0.18, 0.05], add(top, [0, -0.04, 0]), '#B8A6D8', 'satin', taper=0.2, segments=5))
    return p


def garden_hedge():
    return hedge(1.5, 1.12, 0.92)


for key, fn, v in [('boxwood', boxwood, 0.1), ('topiaryCone', topiary_cone, None), ('hedge', garden_hedge, None), ('hydrangea', hydrangea, 0.1),
                   ('rosebush', rosebush, 0.1), ('azalea', azalea, 0.1), ('lavender', lavender, 0.1), ('fern', fern, 0.1), ('grass', ornamental_grass, 0.1),
                   ('hosta', hosta, 0.1)]:
    model('model.garden.' + key, fn(), v)


# =========================================================================================================
# Garden: fruit and vegetables
# =========================================================================================================

def raised_bed(w, d, h, wood=WOOD):
    p = []
    for z in (-d / 2 + 0.03, d / 2 - 0.03):
        p.append(box([w, h, 0.06], [0, h / 2, z], wood, 'wood', radius=0.012))
    for x in (-w / 2 + 0.03, w / 2 - 0.03):
        p.append(box([0.06, h, d - 0.12], [x, h / 2, 0], wood, 'wood', radius=0.012))
    p.append(box([w - 0.1, 0.03, d - 0.1], [0, h - 0.03, 0], SOIL, 'matte', radius=0))
    return p


def veg_patch():
    rng = random.Random(81)
    p = raised_bed(0.92, 0.92, 0.18)
    for i, x in enumerate((-0.28, 0.0, 0.28)):
        for k, z in enumerate((-0.27, 0.0, 0.27)):
            at = [x + rng.uniform(-0.03, 0.03), 0.22, z + rng.uniform(-0.03, 0.03)]
            if i == 0:                                                                            # lettuce
                p.append(blob([0.2, 0.13, 0.2], add(at, [0, 0.03, 0]), ['#9CC45A', '#8AB44E'][k % 2], noise=0.35))
            elif i == 1:                                                                          # cabbage
                p.append(blob([0.22, 0.16, 0.22], add(at, [0, 0.05, 0]), '#7EA48A', noise=0.2, seg=2))
                p.append(blob([0.13, 0.11, 0.13], add(at, [0, 0.1, 0]), '#A8C8A0', 'satin', noise=0.1, seg=1))
            else:                                                                                 # carrot tops
                for j in range(3):
                    p.append(leaf_blade(at, rng.uniform(0.16, 0.22), 0.045, 12 + j * 8, j * 120 + rng.uniform(0, 40), '#5E9A3A'))
                p.append(cone([0.04, 0.04, 0.04], add(at, [0, -0.005, 0]), '#E8782A', 'satin', segments=5))
    return p


def tomatoes():
    rng = random.Random(82)
    p = mound(0.8)
    for x, z in [(-0.2, -0.15), (0.2, -0.12), (0.0, 0.2)]:
        p.append(cyl([0.025, 1.2, 0.025], [x, 0.6, z], '#C9A86A', 'satin', segments=5))
        for k in range(4):
            y = 0.22 + k * 0.24 + rng.uniform(-0.04, 0.04)
            p.append(blob([0.3, 0.22, 0.3], [x + rng.uniform(-0.06, 0.06), y, z + rng.uniform(-0.06, 0.06)], LEAF[(k + 1) % 4], noise=0.35))
    for i in range(15):
        x, z = rng.choice([(-0.2, -0.15), (0.2, -0.12), (0.0, 0.2)])
        a = rng.uniform(0, math.tau)
        at = [x + 0.15 * math.cos(a), rng.uniform(0.2, 0.95), z + 0.15 * math.sin(a)]
        p.append(blob([0.075, 0.07, 0.075], at, ['#D63A2A', '#E04A30', '#C8302A', '#E8902E', '#8DB04A'][i % 5], 'gloss', noise=0.05, seg=1))
    return p


def strawberries():
    rng = random.Random(83)
    p = [blob([0.88, 0.1, 0.88], [0, 0.0, 0], '#D8C27A', 'matte', noise=0.12)]                    # straw
    for i, (x, z, a) in enumerate(spiral(7, 0.32, rng)):
        p.append(blob([0.22, 0.12, 0.22], [x, 0.08, z], LEAF[1 + i % 3], noise=0.35))
    for i, (x, z, a) in enumerate(spiral(16, 0.4, rng)):
        p.append(cone([0.045, 0.055, 0.045], [x, 0.06, z], ['#D8263A', '#C41E30', '#E8E0A0'][0 if i % 5 else 2], 'gloss', segments=5, rotate=[180, 0, 0]))
    for i, (x, z, a) in enumerate(spiral(4, 0.3, rng)):
        p += flower_disc([x, 0.16, z], 0.04, 10, 0, '#F6F4EE', '#F2C12E', seg=5)
    return p


def herbs():
    rng = random.Random(84)
    p = [box([0.84, 0.22, 0.46], [0, 0.62, 0], WOOD, 'wood', radius=0.012)]
    for x in (-0.38, 0.38):
        for z in (-0.19, 0.19):
            p.append(box([0.055, 0.52, 0.055], [x, 0.26, z], WOOD, 'wood', radius=0.008))
    p.append(box([0.78, 0.02, 0.4], [0, 0.725, 0], SOIL, 'matte', radius=0))                    # a little above the rim
    p += [blob([0.2, 0.15, 0.2], [-0.29, 0.79, -0.05], '#6AAE3E', noise=0.35), blob([0.16, 0.12, 0.16], [-0.25, 0.8, 0.1], '#5E9E38', noise=0.35)]   # basil
    for i in range(7):                                                                            # rosemary
        p.append(leaf_blade([-0.06 + rng.uniform(-0.05, 0.05), 0.72, rng.uniform(-0.1, 0.1)], rng.uniform(0.2, 0.28), 0.04, rng.uniform(0, 18), rng.uniform(0, 360), '#3E5E3A'))
    for i in range(6):                                                                            # chives with flowers
        base = [0.14 + rng.uniform(-0.04, 0.04), 0.72, rng.uniform(-0.1, 0.1)]
        p.append(leaf_blade(base, rng.uniform(0.2, 0.26), 0.02, rng.uniform(0, 12), rng.uniform(0, 360), '#4E8A3A'))
        if i % 2 == 0:
            p.append(blob([0.04, 0.04, 0.04], add(base, [0, 0.24, 0]), '#B88AD0', 'satin', noise=0.2, seg=1))
    p += [blob([0.2, 0.13, 0.2], [0.31, 0.78, -0.04], '#8EA088', noise=0.3), blob([0.14, 0.1, 0.14], [0.3, 0.78, 0.12], '#9AAC92', noise=0.3)]   # sage
    return p


def pumpkins():
    rng = random.Random(85)
    p = []
    for i in range(9):
        a = i / 9 * 360 + rng.uniform(-15, 15)
        rr = rng.uniform(0.12, 0.34)
        base = [rr * math.sin(math.radians(a)), 0.04, rr * math.cos(math.radians(a))]
        p.append(leaf_pad(base, 0.2, 0.22, -rng.uniform(5, 20), a, LEAF[i % 4], seg=1, thick=0.04))
    for x, z, s in [(-0.18, 0.12, 0.34), (0.2, -0.05, 0.28), (0.02, -0.3, 0.22)]:
        p.append(torus([s, s * 0.62, s], [x, s * 0.3, z], '#E8822A', 'satin', segments=14))
        p.append(cyl([0.03, 0.06, 0.03], [x, s * 0.62, z], '#5E6A2E', 'matte', segments=5, rotate=[12, 0, 0]))
    return p


def lemon_tree():
    rng = random.Random(86)
    p, top = pot(0.62, 0.5)
    p.append(rod([0, top, 0], [0.04, 1.45, 0.02], 0.07, BARK, 'bark', taper=0.6, seg=6))
    p += crown(rng, [0, 1.85, 0], [0.66, 0.56, 0.66], 10, ['#3A6A30', '#467838', '#548640', '#62944A'], size=0.62, core=0.8)
    p += surface_dots(rng, [0, 1.85, 0], [0.66, 0.56, 0.66], 12, 0.13, ['#F2D23C', '#F5DC50', '#E8C42E'], band=(-0.6, 0.6), out=0.98)
    return p


for key, fn, v in [('vegPatch', veg_patch, None), ('tomatoes', tomatoes, 0.1), ('strawberries', strawberries, None), ('herbs', herbs, None),
                   ('pumpkins', pumpkins, 0.1), ('lemonTree', lemon_tree, None)]:
    model('model.garden.' + key, fn(), v)


# =========================================================================================================
# Garden: houseplants
# =========================================================================================================

WHITE_POT, CREAM_POT = '#EDEBE6', '#D9CDB4'


def monstera():
    rng = random.Random(91)
    p, top = pot(0.38, 0.34, WHITE_POT, 'satin', rim=False)
    for i in range(9):
        yaw = i / 9 * 360 * 2 + rng.uniform(-15, 15)
        h = top + rng.uniform(0.3, 0.6)
        out = rng.uniform(0.07, 0.13)
        base = [out * math.sin(math.radians(yaw)), h, out * math.cos(math.radians(yaw))]
        p.append(stem([0, top, 0], base, 0.016, '#4E7A3A'))
        p.append(leaf_pad(add(base, [0, 0, 0]), rng.uniform(0.29, 0.33), rng.uniform(0.26, 0.3), rng.uniform(18, 36), yaw, ['#2F6B35', '#3B7A3C', '#347040'][i % 3], seg=2))
    return p


def fiddle_fig():
    rng = random.Random(92)
    p, top = pot(0.44, 0.4, CREAM_POT, 'matte', rim=False)
    p.append(rod([0, top, 0], [0.03, 1.55, 0.0], 0.04, '#8A7058', 'bark', taper=0.6, seg=6))
    for i in range(15):
        t = i / 15
        y = top + 0.35 + t * (1.62 - top - 0.35)
        yaw = i * 137.5 + rng.uniform(-10, 10)
        base = [0.02 * t, y, 0]
        p.append(leaf_pad(base, rng.uniform(0.22, 0.28) * (1.1 - t * 0.3), rng.uniform(0.16, 0.2) * (1.1 - t * 0.3), rng.uniform(-10, 25), yaw,
                          ['#2E5A2A', '#386832', '#2A5226'][i % 3], seg=2, thick=0.025))
    return p


def snake_plant():
    rng = random.Random(93)
    p, top = pot(0.34, 0.3, '#2E2E30', 'satin', rim=False)
    for i in range(10):
        x, z = rng.uniform(-0.08, 0.08), rng.uniform(-0.08, 0.08)
        yaw = math.degrees(math.atan2(x, z)) + rng.uniform(-20, 20)
        p.append(leaf_blade([x, top - 0.02, z], rng.uniform(0.5, 0.72), rng.uniform(0.07, 0.09), rng.uniform(2, 10), yaw,
                            ['#2F5A34', '#3D6B3A', '#2A4E30', '#5E7E3A'][i % 4], seg=4, thick=0.022))
    return p


def cacti():
    rng = random.Random(94)
    p = []
    for (x, z, d, h) in [(-0.17, 0.05, 0.22, 0.18), (0.14, -0.08, 0.24, 0.2), (0.12, 0.17, 0.16, 0.13)]:
        q, top = pot(d, h)
        for part_ in q:
            part_['at'][0] = round(part_['at'][0] + x, 3)
            part_['at'][2] = round(part_['at'][2] + z, 3)
        p += q
    green, light = '#4E8A4A', '#6A9E5A'
    p += [cyl([0.1, 0.36, 0.1], [-0.17, 0.33, 0.05], green, 'satin', segments=8), sph([0.1, 0.08, 0.1], [-0.17, 0.51, 0.05], green, 'satin', segments=3),
          rod([-0.17, 0.3, 0.05], [-0.28, 0.42, 0.05], 0.06, green, 'satin', seg=6), sph([0.06, 0.05, 0.06], [-0.28, 0.43, 0.05], green, 'satin', segments=2),
          rod([-0.17, 0.36, 0.05], [-0.07, 0.46, 0.05], 0.055, green, 'satin', seg=6), sph([0.055, 0.045, 0.055], [-0.07, 0.47, 0.05], green, 'satin', segments=2),
          sph([0.03, 0.03, 0.03], [-0.17, 0.555, 0.05], '#E8508A', 'satin', segments=2)]
    p += [blob([0.2, 0.17, 0.2], [0.14, 0.26, -0.08], light, 'satin', noise=0.08, seg=2), sph([0.035, 0.03, 0.035], [0.14, 0.35, -0.08], '#F2C14E', 'satin', segments=2)]
    p += [blob([0.12, 0.15, 0.04], [0.12, 0.2, 0.17], light, 'satin', noise=0.08, seg=1, rotate=[0, 30, 10]),
          blob([0.09, 0.11, 0.035], [0.17, 0.3, 0.17], light, 'satin', noise=0.08, seg=1, rotate=[0, 40, -25])]
    return p


def succulents():
    rng = random.Random(95)
    p = [lathe([0.62, 0.12, 0.62], [0, 0.06, 0], '#7FA6B8', 'satin', [[0, -0.5], [0.4, -0.5], [0.5, 0.5], [0.46, 0.5], [0.42, 0.2], [0, 0.2]], segments=18),
         cyl([0.5, 0.02, 0.5], [0, 0.11, 0], '#B8A88A', 'matte', segments=16)]
    cols = ['#8FB5A0', '#C99AA8', '#5E8F55', '#A8C49A', '#7EA088']
    for i, (x, z, a) in enumerate(spiral(7, 0.2, rng)):
        s = rng.uniform(0.09, 0.13)
        p.append(cone([s, s * 0.55, s], [x, 0.14, z], cols[i % 5], 'satin', taper=1.7, segments=7))
        p.append(cone([s * 0.6, s * 0.45, s * 0.6], [x, 0.16, z], cols[(i + 2) % 5], 'satin', taper=1.6, segments=5, rotate=[0, 25, 0]))
    return p


def boston_fern():
    rng = random.Random(96)
    wood = '#7A5A40'
    p = [cyl([0.4, 0.04, 0.4], [0, 0.55, 0], wood, 'wood', segments=16)]
    for i in range(3):
        a = i / 3 * math.tau
        p.append(rod([0.14 * math.cos(a), 0.54, 0.14 * math.sin(a)], [0.18 * math.cos(a), 0, 0.18 * math.sin(a)], 0.04, wood, 'wood', seg=6))
    q, top = pot(0.3, 0.24, WHITE_POT, 'satin', y0=0.57, rim=False)
    p += q
    for i in range(12):
        yaw = i / 12 * 360 + rng.uniform(-10, 10)
        p.append(leaf_pad([0, top + 0.04, 0], rng.uniform(0.38, 0.46), 0.13, rng.uniform(25, 50), yaw, ['#5E9A3E', '#6FAA46', '#558E38'][i % 3], seg=2, noise=0.15))
    for i in range(6):
        p.append(leaf_pad([0, top + 0.06, 0], 0.3, 0.11, -rng.uniform(30, 50), i * 60 + 30, '#78B24E', seg=1, noise=0.15))
    return p


def palm():
    rng = random.Random(97)
    p, top = pot(0.4, 0.34, CREAM_POT, 'matte', rim=False)
    for i in range(7):
        yaw = i / 7 * 360 + rng.uniform(-15, 15)
        r, ax = facing(rng.uniform(10, 20), yaw)
        length = rng.uniform(0.4, 0.6)
        tip = add([0, top, 0], ax, length)
        p.append(rod([0, top, 0], tip, 0.014, '#5E7E3A', 'foliage', seg=3))
        p.append(leaf_pad(add(tip, [0, -0.02, 0]), rng.uniform(0.28, 0.34), 0.2, rng.uniform(15, 35), yaw, ['#4A8A3A', '#5A9A42', '#3E7A34'][i % 3], seg=2, thick=0.02, noise=0.2))
    return p


def orchid():
    rng = random.Random(98)
    wood = '#5A4030'
    p = [cyl([0.28, 0.6, 0.28], [0, 0.3, 0], wood, 'wood', taper=0.85, segments=12), cyl([0.32, 0.03, 0.32], [0, 0.615, 0], wood, 'wood', segments=12)]
    q, top = pot(0.18, 0.16, WHITE_POT, 'gloss', y0=0.63, rim=False, soil='#8A6A48')
    p += q
    for i, yaw in enumerate((0, 140, 250)):
        p.append(leaf_pad([0, top + 0.02, 0], 0.2, 0.09, rng.uniform(10, 25), yaw, '#2E5A30', seg=1, thick=0.02))
    pts = [[0, top, 0], [0.04, top + 0.3, -0.02], [0.12, top + 0.42, 0.0], [0.22, top + 0.4, 0.02]]
    for a, b in zip(pts, pts[1:]):
        p.append(rod(a, b, 0.01, '#4E6A3A', 'foliage', seg=3))
    for i in range(6):
        t = 0.35 + i * 0.13
        k = min(2, int(t * 3))
        a, b = pts[k], pts[min(3, k + 1)]
        f = t * 3 - k
        at = [a[j] + (b[j] - a[j]) * f for j in range(3)]
        p += flower_disc(add(at, [0, -0.03, 0.03]), 0.075, 70, rng.uniform(-30, 30), ['#F6F0F4', '#E8A8C8'][i % 2], '#C2367A', cd=0.025, seg=5)
    return p


def bonsai():
    wood = '#6A4A32'
    p = [box([0.62, 0.04, 0.4], [0, 0.38, 0], wood, 'wood', radius=0.01)]
    for x in (-0.27, 0.27):
        for z in (-0.16, 0.16):
            p.append(box([0.05, 0.36, 0.05], [x, 0.18, z], wood, 'wood', radius=0.006))
    p += [box([0.46, 0.07, 0.3], [0, 0.435, 0], '#3E5A78', 'gloss', radius=0.012), box([0.42, 0.02, 0.26], [0, 0.47, 0], '#4E6A34', 'matte', radius=0)]
    pts = [[-0.02, 0.47, 0], [0.06, 0.6, 0.02], [-0.05, 0.72, -0.01], [0.08, 0.82, 0.0]]
    for a, b in zip(pts, pts[1:]):
        p.append(rod(a, b, 0.045, '#7A6658', 'bark', seg=6))
    p.append(rod([0.06, 0.6, 0.02], [-0.18, 0.7, 0.04], 0.025, '#7A6658', 'bark', seg=5))
    p += [blob([0.3, 0.1, 0.22], [0.1, 0.86, 0.0], '#3A6A30', noise=0.25), blob([0.24, 0.08, 0.18], [-0.2, 0.73, 0.04], '#467A38', noise=0.25),
          blob([0.18, 0.07, 0.14], [0.0, 0.7, -0.08], '#3E7034', noise=0.25)]
    return p


for key, fn in [('monstera', monstera), ('fiddleFig', fiddle_fig), ('snakePlant', snake_plant), ('cacti', cacti), ('succulents', succulents),
                ('bostonFern', boston_fern), ('palm', palm), ('orchid', orchid), ('bonsai', bonsai)]:
    model('model.garden.' + key, fn())


# =========================================================================================================
# Garden: decor
# =========================================================================================================

def birdbath():
    return [cyl([0.34, 0.06, 0.34], [0, 0.03, 0], STONE_DARK, 'matte', segments=12),
            lathe([0.22, 0.66, 0.22], [0, 0.39, 0], STONE, 'matte', [[0, -0.5], [0.5, -0.5], [0.32, -0.38], [0.24, 0.0], [0.3, 0.38], [0.5, 0.5], [0, 0.5]], segments=12),
            lathe([0.68, 0.12, 0.68], [0, 0.78, 0], STONE, 'matte', [[0, -0.5], [0.2, -0.5], [0.5, 0.3], [0.5, 0.5], [0.44, 0.5], [0.4, 0.2], [0, 0.2]], segments=20),
            cyl([0.56, 0.01, 0.56], [0, 0.8, 0], WATER, 'gloss', segments=20),
            blob([0.07, 0.06, 0.11], [0.26, 0.87, 0.02], '#7A5A40', 'satin', noise=0.1, seg=1),
            sph([0.05, 0.05, 0.05], [0.26, 0.91, 0.07], '#7A5A40', 'satin', segments=3),
            cone([0.02, 0.03, 0.02], [0.26, 0.91, 0.105], '#E8A23A', 'satin', segments=4, rotate=[90, 0, 0])]


def fountain():
    return [lathe([0.92, 0.34, 0.92], [0, 0.17, 0], STONE, 'matte', [[0, -0.5], [0.5, -0.5], [0.5, 0.4], [0.47, 0.5], [0.43, 0.5], [0.42, 0.25], [0, 0.25]], segments=24),
            cyl([0.84, 0.01, 0.84], [0, 0.26, 0], WATER, 'gloss', segments=24),
            lathe([0.18, 0.56, 0.18], [0, 0.54, 0], STONE, 'matte', [[0, -0.5], [0.5, -0.5], [0.34, -0.3], [0.3, 0.3], [0.5, 0.5], [0, 0.5]], segments=12),
            lathe([0.5, 0.12, 0.5], [0, 0.86, 0], STONE, 'matte', [[0, -0.5], [0.2, -0.5], [0.5, 0.3], [0.5, 0.5], [0.44, 0.5], [0.4, 0.2], [0, 0.2]], segments=18),
            cyl([0.42, 0.01, 0.42], [0, 0.885, 0], WATER, 'gloss', segments=18),
            cyl([0.08, 0.14, 0.08], [0, 0.96, 0], STONE, 'matte', segments=8, taper=0.7),
            sph([0.08, 0.08, 0.08], [0, 1.05, 0], STONE, 'matte', segments=6),
            cyl([0.025, 0.1, 0.025], [0, 1.12, 0], '#DCEBF0', 'glass', segments=6)]


def gnome():
    return [box([0.07, 0.04, 0.09], [-0.045, 0.02, 0.01], '#3A2A20', 'satin', radius=0.015), box([0.07, 0.04, 0.09], [0.045, 0.02, 0.01], '#3A2A20', 'satin', radius=0.015),
            cyl([0.2, 0.24, 0.18], [0, 0.16, 0], '#3E63B0', 'satin', taper=0.6, segments=12),
            cyl([0.15, 0.03, 0.14], [0, 0.22, 0], '#5A3A20', 'satin', segments=12),
            rod([-0.07, 0.27, 0], [-0.1, 0.17, 0.05], 0.045, '#3E63B0', 'satin', seg=6), rod([0.07, 0.27, 0], [0.1, 0.17, 0.05], 0.045, '#3E63B0', 'satin', seg=6),
            sph([0.04, 0.04, 0.04], [-0.1, 0.16, 0.06], '#F2C6A6', 'satin', segments=3), sph([0.04, 0.04, 0.04], [0.1, 0.16, 0.06], '#F2C6A6', 'satin', segments=3),
            sph([0.12, 0.11, 0.11], [0, 0.34, 0.0], '#F2C6A6', 'satin', segments=6),
            sph([0.035, 0.03, 0.035], [0, 0.34, 0.06], '#E8A08A', 'satin', segments=3),
            cone([0.12, 0.12, 0.08], [0, 0.27, 0.04], '#F4F2EE', 'satin', segments=8, rotate=[180, 0, 0]),
            cone([0.14, 0.2, 0.14], [0, 0.47, -0.01], '#D3322B', 'satin', segments=12, rotate=[-8, 0, 0])]


def pond():
    rng = random.Random(101)
    p = [cyl([1.66, 0.02, 1.46], [0, 0.01, 0], WATER_DEEP, 'gloss', segments=28), cyl([1.2, 0.02, 1.0], [0.05, 0.015, 0.03], '#3E7890', 'gloss', segments=24)]
    for i in range(22):
        a = i / 22 * math.tau
        s = rng.uniform(0.17, 0.25)
        p.append(blob([s, s * 0.6, s * 0.85], [0.88 * math.cos(a), 0.04, 0.78 * math.sin(a)], [STONE, '#BDB6AA', '#D8D2C6', STONE_DARK][i % 4], 'matte', noise=0.2, seg=1,
                      rotate=[0, -math.degrees(a), 0]))
    for x, z, s in [(-0.3, 0.2, 0.2), (0.25, -0.2, 0.17), (0.4, 0.25, 0.15), (-0.15, -0.35, 0.16), (0.0, 0.42, 0.13)]:
        p.append(cyl([s, 0.012, s], [x, 0.03, z], '#5E8E3E', 'satin', segments=8))
    p += [cone([0.07, 0.05, 0.07], [-0.27, 0.06, 0.2], '#F2A6C8', 'satin', taper=1.6, segments=6), cone([0.06, 0.05, 0.06], [0.24, 0.06, -0.19], '#F6EEF2', 'satin', taper=1.6, segments=6)]
    for i in range(8):                                                                            # reeds and bulrushes
        base = [-0.62 + rng.uniform(-0.1, 0.1), 0.02, -0.42 + rng.uniform(-0.1, 0.1)]
        p.append(leaf_blade(base, rng.uniform(0.45, 0.7), 0.04, rng.uniform(0, 15), rng.uniform(0, 360), ['#6E8E3A', '#7E9E44'][i % 2]))
    for i in range(3):
        at = [-0.62 + i * 0.07, 0.62 + i * 0.05, -0.44 + (i % 2) * 0.06]
        p.append(cyl([0.03, 0.12, 0.03], at, '#6A4A2E', 'matte', segments=5))
    return p


def lantern():
    p = [cyl([0.44, 0.08, 0.44], [0, 0.04, 0], STONE_DARK, 'matte', segments=6), cyl([0.16, 0.42, 0.16], [0, 0.29, 0], STONE, 'matte', segments=6),
         cyl([0.36, 0.06, 0.36], [0, 0.53, 0], STONE, 'matte', segments=6), box([0.2, 0.18, 0.2], [0, 0.65, 0], '#FFE6A8', 'satin', radius=0)]
    for x in (-0.11, 0.11):
        for z in (-0.11, 0.11):
            p.append(box([0.045, 0.2, 0.045], [x, 0.66, z], STONE, 'matte', radius=0))
    p += [cone([0.54, 0.16, 0.54], [0, 0.84, 0], STONE, 'matte', taper=0.22, segments=6), cyl([0.1, 0.05, 0.1], [0, 0.94, 0], STONE, 'matte', segments=6),
          sph([0.08, 0.09, 0.08], [0, 0.99, 0], STONE, 'matte', segments=4)]
    return p


def wheelbarrow():
    rng = random.Random(102)
    green, wood = '#3E7A6A', '#A8835A'
    p = [box([0.46, 0.04, 0.58], [0, 0.4, 0.02], green, 'metal', radius=0.01)]
    for s in (-1, 1):
        p.append(box([0.03, 0.24, 0.62], [s * 0.25, 0.52, 0.02], green, 'metal', radius=0.008, rotate=[0, 0, -s * 14]))
    p += [box([0.52, 0.24, 0.03], [0, 0.52, -0.3], green, 'metal', radius=0.008, rotate=[-22, 0, 0]),
          box([0.5, 0.22, 0.03], [0, 0.51, 0.33], green, 'metal', radius=0.008, rotate=[14, 0, 0]),
          box([0.44, 0.03, 0.52], [0, 0.6, 0.02], SOIL, 'matte', radius=0)]
    for s in (-1, 1):
        p.append(rod([s * 0.17, 0.36, -0.36], [s * 0.21, 0.58, 0.49], 0.04, wood, 'wood', seg=6))
        p.append(rod([s * 0.17, 0.38, 0.2], [s * 0.19, 0.0, 0.26], 0.03, '#2E3236', 'metal', seg=6))
    p += [torus([0.32, 0.07, 0.32], [0, 0.16, -0.38], '#24272A', 'matte', segments=16, rotate=[0, 0, 90]),
          cyl([0.12, 0.08, 0.12], [0, 0.16, -0.38], '#8A9096', 'metal', segments=10, rotate=[0, 0, 90])]
    p += [blob([0.2, 0.14, 0.2], [x, 0.66, z], LEAF[i % 3], noise=0.3) for i, (x, z) in enumerate([(-0.1, -0.1), (0.1, 0.05), (-0.05, 0.18)])]
    p += [blob([0.06, 0.045, 0.06], [rng.uniform(-0.17, 0.17), rng.uniform(0.7, 0.74), rng.uniform(-0.2, 0.24)], ['#E85A9A', '#F2C14E', '#F4F0F2'][i % 3], 'satin', noise=0.2, seg=1)
          for i in range(7)]
    return p


def boulder():
    rng = random.Random(103)
    p = [blob([0.78, 0.5, 0.7], [0, 0.17, 0], '#8F8B82', 'matte', noise=0.3), blob([0.36, 0.26, 0.32], [0.33, 0.08, 0.2], '#9A958C', 'matte', noise=0.3),
         blob([0.42, 0.07, 0.32], [-0.06, 0.4, -0.04], '#6E8A44', 'foliage', noise=0.25)]
    for i in range(6):
        a = i / 6 * 360 + rng.uniform(-20, 20)
        base = [0.42 * math.sin(math.radians(a)), 0, 0.38 * math.cos(math.radians(a))]
        for k in range(2):
            p.append(leaf_blade(base, rng.uniform(0.14, 0.22), 0.035, rng.uniform(10, 30), a + k * 70, ['#6E8E3C', '#7E9C44'][k]))
    return p


for key, fn, v in [('birdbath', birdbath, None), ('fountain', fountain, None), ('gnome', gnome, None), ('pond', pond, None),
                   ('lantern', lantern, None), ('wheelbarrow', wheelbarrow, None), ('boulder', boulder, 0.1)]:
    model('model.garden.' + key, fn(), v)


# =========================================================================================================
# Checks and output
# =========================================================================================================

PLAIN = {'matte', 'satin', 'gloss', 'metal', 'chrome', 'skin', 'hair'}
HEX = re.compile(r'^#[0-9A-Fa-f]{6}$')


def triangles(p, lod):
    s = p.get('segments')
    shape = p['shape']
    if shape == 'blob':
        n = s or (1 if lod else 2)
        return 20 * n * n
    if shape in ('cylinder', 'cone'):
        return 4 * (s or (8 if lod else 16 if shape == 'cone' else 20))
    if shape == 'sphere':
        z = 2 + (s or (6 if lod else 12))
        return 2 * z * 2 * z
    if shape == 'torus':
        t = s or (12 if lod else 24)
        return 2 * t * t
    if shape == 'lathe':
        return 2 * (s or (12 if lod else 24)) * (len(p['profile']) - 1)
    if shape == 'box':
        return 12 if p.get('radius', 0.012) == 0 or lod else 588
    return 0


def groups(parts):
    fins = {p['material'].split('.')[-1] for p in parts}
    return {'plain' if f in PLAIN else f for f in fins}


def check():
    bad = 0
    print(f'{"key":34} {"parts":>5} {"tris":>6}  groups')
    for k, parts in E.items():
        lod = 1 if k.endswith('.far') else 0
        for i, p in enumerate(parts):
            if not HEX.match(p['color']) or min(p['size']) <= 0 or len(p['size']) != 3:
                print(f'  {k} part {i}: bad colour or size {p}')
                bad += 1
        g = groups(parts)
        if len(g) > 4:
            print(f'  {k}: {len(g)} material groups {sorted(g)}')
            bad += 1
        print(f'{k:34} {len(parts):5} {sum(triangles(p, lod) for p in parts):6}  {",".join(sorted(g))}')
    return bad


def entry(key):
    e = {'type': 'model', 'placeholder': E[key]}
    if key in VARY:
        e['vary'] = VARY[key]
    return e


if __name__ == '__main__':
    args = sys.argv[1:]
    if check():
        sys.exit(1)
    if args[:1] == ['--pack']:
        json.dump({'version': 1, 'entries': {k: entry(k) for k in E}}, open(args[1], 'w'), indent=1)
        print('wrote', len(E), 'entries to', args[1])
    elif args:
        path = args[0]
        d = json.load(open(path))
        entries = d['entries']
        missing = [k for k in E if k not in entries]
        if missing:
            sys.exit(f'not in the manifest: {missing}')
        for k in E:
            entries[k] = entry(k)
        left = [k for k, v in entries.items() if str(v.get('url', '')).startswith('nature/')]
        if left:
            print('still pointing at nature/ files:', left)
        s = json.dumps(d, indent=2) + '\n'
        json.loads(s)
        open(path, 'w').write(s)
        print('replaced', len(E), 'entries in', path)

# The garden's trees (G5): the oak (`model.tree`), maple, silver birch, cherry blossom, apple, Japanese maple,
# magnolia and weeping willow (`model.tree.*`), the pine, blue spruce and Italian cypress (`model.pine`,
# `model.spruce`, `model.cypress`) and the potted lemon tree (`model.garden.lemonTree`), over the primitive-part
# placeholders of `nature.py`, which stay the fallback.
#
# Crowns are leafy lumps (`plants.lump`) spread over an ellipsoid, lighter on top, round a dark core, on bark limbs;
# conifers stacked cones. Leaves are on `foliage` and trunks on `bark`, which the nature shader sways and tints.
# Trees stand on every lot and by the thousand in the woods, so each costs what its placeholder does, and each has a
# far version (`far_of`: the same build at low detail, a third of the clumps), which the far woods and the town
# overview draw in place of the `*.far` placeholder keys.
#
# Run:  python3 -B tools/art/blender/build.py trees [--only oak] [--preview]
import math
import os
import random
import sys

import bpy
from mathutils import Vector

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gamekit as G  # noqa: E402
from plants import LOW, _with_part, dots, far_of, low, lump, pot, sprigs, tube  # noqa: E402

BASE = dict(
    bark=('#E4D8C8', 'bark'),           # the bark texture is dark: its colours are pale
    birch_bark=('#ECE8DF', 'matte'),
    birch_mark=('#36322E', 'matte'),
    leaf=('#558238', 'foliage'),        # five shades, darkest first (the foliage texture is pale too)
    leaf2=('#64923E', 'foliage'),
    leaf3=('#76A246', 'foliage'),
    leaf4=('#8AB250', 'foliage'),
    leaf5=('#9EC25C', 'foliage'),
    leaf_light=('#8AB250', 'foliage'),  # sprigs (plants.sprigs)
    green=('#6E9A44', 'foliage'),       # green clumps among blossom
    green2=('#62903E', 'foliage'),
    fruit=('#C8352B', 'satin'),         # apples, lemons, blooms
    fruit2=('#D8402C', 'satin'),
    fruit3=('#B42A26', 'satin'),
    fruit4=('#E2B03C', 'satin'),
    body=('#C0683F', 'satin'),          # the lemon tree's pot
    soil=('#5A4030', 'matte'),
    stalk=('#5E5040', 'satin'),         # the picked fruit's stalk: one draw call with the fruit
)
M = G.Palette(**BASE)
LEAVES = ('leaf', 'leaf2', 'leaf3', 'leaf4', 'leaf5')


def palette(leaves=None, **colours):
    """The family's palette with some colours changed; `leaves` gives the five leaf shades, darkest first."""
    if leaves:
        colours.update(zip(LEAVES, leaves))
    return G.Palette(**{**BASE, **{k: (v, BASE[k][1]) for k, v in colours.items()}})


def shades(P, n=5):
    return [getattr(P, k) for k in LEAVES[:n]]


# ---- parts (Blender axes: x, y across, z up; the trunk on the origin) ---------------------------------

def crown(name, rng, c, r, n, cols, size=0.75, spread=0.72, squash=0.85, noise=0.3, bottom=-0.45, core=0.0):
    """Leaf clumps over an ellipsoid crown (centre `c`, radii `r`): `n` lumps about `size` x the mean radius,
    `spread` x the radius out from the centre, lighter towards the top; `core` adds a dark filling lump. A far
    version has every third clump round a smooth core spanning all the full crown's clumps, so it keeps its size."""
    m = sum(r) / 3
    cx, cy, cz = c
    full_core = core
    far, core = LOW[0], (0 if LOW[0] else core)
    if far:
        seed = rng.random() * 9 if full_core else 0.0                  # drawn where the full crown draws its core's
        lo, hi = [math.inf] * 3, [-math.inf] * 3
    if core:
        lump(f'{name}_core', r[0] * core, (cx, cy, cz - r[2] * core), cols[0], scale=(1, r[1] / r[0], r[2] / r[0]),
             noise=noise * 0.8, seed=rng.random() * 9, segments=10, rings=6)
    ga = math.pi * (3 - math.sqrt(5))
    off = rng.random() * math.tau
    for i in range(n):
        t = 1 - (i + 0.5) / n * (1 - bottom)                          # height on the ellipsoid, top first
        ring = math.sqrt(max(0.0, 1 - t * t))
        a = off + ga * i + rng.uniform(-0.35, 0.35)
        k = spread * rng.uniform(0.88, 1.1)
        s = size * m * rng.uniform(0.85, 1.18) / 2
        z = cz + r[2] * t * k
        light = (t - bottom) / (1 - bottom)
        col = cols[max(0, min(len(cols) - 1, round(light * (len(cols) - 1) + rng.uniform(-0.6, 0.6))))]
        sx, sy, clump_seed = rng.uniform(0.9, 1.15), rng.uniform(0.9, 1.15), i * 1.7 + rng.random()
        at = (cx + r[0] * ring * math.cos(a) * k, cy + r[1] * ring * math.sin(a) * k, z)
        if far:                                                        # the clumps' reach, for the core
            half = (s * sx, s * sy, s * squash)
            lo = [min(lo[j], at[j] - half[j]) for j in range(3)]
            hi = [max(hi[j], at[j] + half[j]) for j in range(3)]
            if i % 3:                                                  # every third clump
                continue
        lump(f'{name}{i}', s, (at[0], at[1], z - s * squash), col, scale=(sx, sy, squash), noise=noise, seed=clump_seed,
             segments=8, rings=5)
    if far:
        rx, ry, rz = ((hi[j] - lo[j]) / 2 for j in range(3))
        lump(f'{name}_core', rx, ((lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2, lo[2]), cols[0], scale=(1, ry / rx, rz / rx),
             noise=0.0, seed=seed, segments=14, rings=7)               # smooth, 8 x 4 (its equator kept)


def trunk(P, h, d, material=None, taper=0.62, flare=True, verts=9):
    """A tapering trunk `h` tall, `d` across at the foot, with a flared base."""
    material = material or P.bark
    tube('trunk', [(0, 0, 0), (0, 0, h)], d / 2, d / 2 * taper, material, verts=verts)
    if flare:
        G.cone('flare', d * 0.875, d * 0.45, d * 0.9, at=(0, 0, d * 0.45), material=material, verts=low(verts, 5))


def limbs(P, rng, base, tips, d, material=None):
    """Limbs from `base` out to each of `tips`, tapering."""
    for i, tip in enumerate(tips):
        w = d * rng.uniform(0.85, 1.1)
        tube(f'limb{i}', [base, tip], w / 2, w / 4, material or P.bark, verts=6)


def xy(p):
    """A point from the placeholders' game axes (x, height, depth) to Blender's (x, depth, height); trees are round,
    so the mirror doesn't matter."""
    return (p[0], p[2], p[1])


# ---- broadleaves --------------------------------------------------------------------------------------

def oak():
    """Oak: a stout trunk, four big limbs and a broad, billowing crown with two low side crowns."""
    P = palette(['#558238', '#64923E', '#76A246', '#8AB250', '#9EC25C'])
    rng = random.Random(11)
    trunk(P, 3.4, 0.62)
    limbs(P, rng, (0, 0, 2.9), [xy(t) for t in ([1.5, 4.6, 0.5], [-1.4, 4.5, -0.6], [0.3, 5.0, -1.3], [-0.4, 5.1, 1.2])], 0.3)
    crown('crown', rng, (0, 0, 5.5), (2.7, 2.7, 1.8), 10, shades(P), size=0.7, spread=0.75, core=0.82)
    crown('side', rng, (1.7, 0.7, 4.6), (1.4, 1.4, 1.0), 3, shades(P), size=0.85, bottom=-0.6, core=0.7)
    crown('side2', rng, (-1.6, -0.8, 4.7), (1.4, 1.4, 1.1), 3, shades(P), size=0.85, bottom=-0.6, core=0.7)
    return G.settle([G.join_all('oak')])


def maple():
    """Maple: a straight trunk and a tall, rounded crown of many clumps."""
    P = palette(['#5A8A36', '#6A9A3E', '#7EAA48', '#92BA52', '#A6C85E'])
    rng = random.Random(12)
    trunk(P, 3.2, 0.5)
    limbs(P, rng, (0, 0, 2.8), [xy(t) for t in ([1.1, 4.3, 0.4], [-1.0, 4.4, -0.3], [0.1, 4.6, -1.1])], 0.24)
    crown('crown', rng, (0, 0, 5.1), (2.5, 2.5, 2.4), 14, shades(P), size=0.6, spread=0.78, core=0.85)
    return G.settle([G.join_all('maple')])


def birch():
    """Silver birch: a slim white trunk with dark marks, light limbs and an airy, upright crown."""
    P = palette(['#76963E', '#86A646', '#98B650', '#AAC45C', '#BCD26A'], bark='#5A524A')
    rng = random.Random(13)
    tube('trunk', [(0, 0, 0), (0, 0, 7.2)], 0.15, 0.063, P.birch_bark, verts=8)
    G.cone('foot', 0.23, 0.14, 0.4, at=(0, 0, 0.2), material=P.bark, verts=low(8, 5))
    for i in range(9):
        z = 0.6 + i * 0.55 + rng.uniform(-0.15, 0.15)
        a = rng.uniform(0, 360)
        rr = 0.15 - z * 0.012
        G.cube(f'mark{i}', (0.11, 0.03, 0.035), at=(rr * math.sin(math.radians(a)), rr * math.cos(math.radians(a)), z),
               material=P.birch_mark, rot=(0, 0, -a))
    limbs(P, rng, (0, 0, 3.6), [xy(t) for t in ([0.9, 5.0, 0.3], [-0.8, 5.2, -0.3], [0.2, 5.6, 0.9], [-0.3, 4.9, -0.9])], 0.1,
          P.birch_bark)
    crown('crown', rng, (0, 0, 5.8), (1.6, 1.6, 2.4), 13, shades(P), size=0.52, spread=0.8, noise=0.34, core=0.72)
    return G.settle([G.join_all('birch')])


def cherry():
    """Cherry blossom: a short dark trunk, spreading limbs and a wide, flat crown of pink blossom."""
    P = palette(['#E07AAE', '#EA90BE', '#F2A8CE', '#F8BEDC', '#FCD4E8'], bark='#B09A8A')
    rng = random.Random(14)
    trunk(P, 1.9, 0.45, taper=0.7)
    limbs(P, rng, (0, 0, 1.7), [xy(t) for t in ([1.6, 3.2, 0.5], [-1.5, 3.1, -0.4], [0.3, 3.4, -1.5], [-0.4, 3.3, 1.4])], 0.22)
    crown('crown', rng, (0, 0, 3.9), (2.7, 2.7, 1.45), 14, shades(P), size=0.6, squash=0.75, noise=0.32, spread=0.78, core=0.8,
          bottom=-0.55)
    lump('leaves', 0.4, (2.0, -0.9, 3.25), P.green, scale=(1, 1, 0.62), seed=1.0)
    lump('leaves2', 0.35, (-1.8, 1.2, 3.38), P.green2, scale=(1, 1, 0.64), seed=2.0)
    return G.settle([G.join_all('cherry')])


def apple():
    """Apple tree: a short trunk, a round crown hung with red and yellow apples; one at the front, which a resident
    picks (it shrinks away)."""
    P = palette(['#52803A', '#629040', '#74A048', '#86AE52', '#98BC5E'])
    rng = random.Random(15)
    trunk(P, 2.1, 0.4, taper=0.7)
    limbs(P, rng, (0, 0, 1.9), [xy(t) for t in ([1.3, 3.1, 0.4], [-1.2, 3.0, -0.4], [0.2, 3.3, -1.3], [-0.2, 3.2, 1.2])], 0.2)
    crown('crown', rng, (0, 0, 3.6), (2.2, 2.2, 1.65), 12, shades(P), size=0.64, spread=0.76, core=0.84)
    dots(rng, low(18, 8), (0, 0, 3.6), (2.2, 2.2, 1.65), 0.2, [P.fruit, P.fruit2, P.fruit3, P.fruit4], 'apple',
         band=(-0.65, 0.55), out=0.9, flat=1.0, segments=low(6, 5), rings=low(4, 3))
    still = G.join_all('apple')
    at = (0.2, -1.95, 2.95)

    def picked():
        lump('picked_apple', 0.11, at, P.fruit, segments=8, rings=5, noise=0.05)
        tube('picked_stalk', [(at[0], at[1], at[2] + 0.2), (at[0], at[1], at[2] + 0.28)], 0.012, 0.01, P.stalk, verts=4)
    return _with_part('apple_part', still, picked, (at[0], at[1], at[2] + 0.1), 'pick')


def japanese_maple():
    """Japanese maple: low, spreading limbs and layered red crowns; a few stray sprigs on top, which shaping
    takes off."""
    P = palette(['#6E1C1C', '#8A2424', '#A2302A', '#B83F2F', '#CC5636'], bark='#B09A8A', leaf_light='#B83F2F')
    rng = random.Random(16)
    limbs(P, rng, (0, 0, 0), [xy(t) for t in ([0.7, 1.7, 0.3], [-0.6, 1.6, -0.4], [0.1, 1.9, -0.7], [-0.2, 1.5, 0.7])], 0.17)
    G.cone('foot', 0.21, 0.105, 0.3, at=(0, 0, 0.15), material=P.bark, verts=low(7, 5))
    crown('crown', rng, (0, 0, 2.0), (1.95, 1.95, 0.6), 9, shades(P), size=0.66, squash=0.68, noise=0.32, spread=0.8, core=0.75)
    crown('crown2', rng, (0.2, -0.1, 2.7), (1.45, 1.45, 0.55), 7, shades(P)[1:], size=0.7, squash=0.68, noise=0.32, spread=0.8,
          core=0.7)
    crown('crown3', rng, (-0.1, 0.1, 3.2), (0.85, 0.85, 0.4), 3, shades(P)[2:], size=0.9, squash=0.7, noise=0.3)
    still = G.join_all('japaneseMaple')
    at = (0.0, 0.0, 3.42)
    return _with_part('sprigs', still, lambda: sprigs('sprig', P, at, 0.35, n=6, length=0.22), at, 'trim')


def magnolia():
    """Magnolia: a grey trunk, upswept limbs and a crown dotted with big white and pink blooms."""
    P = palette(['#EAAACA', '#F2BED6', '#F8D2E2', '#FCE2EE', '#FFF2F8'], bark='#D2CBC0', fruit='#FFFFFF', fruit2='#F2A2C4',
                fruit3='#E888B0', green='#6E9A48', green2='#628E40')
    rng = random.Random(17)
    trunk(P, 1.6, 0.36, taper=0.7)
    limbs(P, rng, (0, 0, 1.4), [xy(t) for t in ([1.2, 3.0, 0.3], [-1.1, 3.1, -0.4], [0.2, 3.5, -1.0], [-0.3, 3.3, 1.0],
                                                 [0.1, 3.9, 0.1])], 0.16)
    crown('crown', rng, (0, 0, 3.4), (1.9, 1.8, 1.5), 11, shades(P), size=0.58, spread=0.8, noise=0.3, core=0.7)
    lump('leaves', 0.3, (1.5, 0.8, 2.72), P.green, scale=(1, 1, 0.75), seed=1.0)
    lump('leaves2', 0.28, (-1.3, -0.9, 3.44), P.green2, scale=(1, 1, 0.73), seed=2.0)
    dots(rng, low(16, 7), (0, 0, 3.45), (1.95, 1.85, 1.55), 0.24, [P.fruit, P.fruit2, P.fruit3, P.fruit], 'bloom', band=(-0.3, 0.95),
         out=0.98, flat=1.0, segments=low(7, 5), rings=low(4, 3))
    return G.settle([G.join_all('magnolia')])


def willow():
    """Weeping willow: a thick trunk, limbs reaching out and a crown with curtains of leaves hanging nearly to
    the ground."""
    P = palette(['#6A8A36', '#7A9A3E', '#8AAA48', '#9CBA54', '#AEC862'])
    rng = random.Random(18)
    cols = shades(P)
    trunk(P, 2.7, 0.62, taper=0.65)
    limbs(P, rng, (0, 0, 2.4), [(2.0 * math.cos(a), 2.0 * math.sin(a), 4.3) for a in (0.3, 1.6, 2.9, 4.2, 5.4)], 0.24)
    crown('crown', rng, (0, 0, 4.7), (2.3, 2.3, 1.4), 9, cols, size=0.68, squash=0.7, core=0.85, bottom=-0.3)
    # Hanging curtains: long strands narrowing downwards all round, down to about a metre off the ground.
    for i in range(low(26, 10)):
        a = i / low(26, 10) * math.tau + rng.uniform(-0.1, 0.1)
        rr = rng.uniform(1.75, 2.2)
        length = rng.uniform(2.4, 3.5)
        top = rng.uniform(4.6, 5.0)
        G.cone(f'strand{i}', 0.03, rng.uniform(0.25, 0.35), length, at=(rr * math.cos(a), rr * math.sin(a), top - length / 2),
               material=cols[rng.randint(1, 4)], scale=(0.6, 1, 1), rot=(rng.uniform(-6, 6), 0, math.degrees(a)),
               verts=low(6, 4))
    for i in range(low(8, 4)):
        a = i / low(8, 4) * math.tau + 0.3
        G.cone(f'inner{i}', 0.03, 0.4, 2.6, at=(1.35 * math.cos(a), 1.35 * math.sin(a), 3.7), material=cols[0],
               scale=(0.75, 1, 1), rot=(0, 0, math.degrees(a)), verts=low(6, 4))
    return G.settle([G.join_all('willow')])


# ---- conifers -----------------------------------------------------------------------------------------

def conifer(rng, tiers, base_d, z0, step, h, cols, top_d=0.5, tilt=4.0, offset=0.12, verts=11):
    """Stacked cones from `z0` (the first tier's foot), each `step` above the last, narrowing to `top_d`."""
    for i in range(tiers):
        t = i / max(1, tiers - 1)
        d = base_d + (top_d - base_d) * t ** 0.9
        hh = h * (1 - 0.35 * t)
        G.cone(f'tier{i}', d / 2 * rng.uniform(0.94, 1.06), low(0.03, 0.0), hh,
               at=(rng.uniform(-offset, offset), rng.uniform(-offset, offset), z0 + i * step + hh / 2),
               material=cols[(i + rng.randint(0, 1)) % len(cols)], verts=low(verts, 8),
               rot=(rng.uniform(-tilt, tilt), rng.uniform(-tilt, tilt), rng.uniform(0, 360)))


def pine():
    """Pine: a tall bare trunk and six broad tiers narrowing to a point."""
    P = palette(['#335C38', '#3D683E', '#477444', '#51804A'], bark='#B59A82')
    rng = random.Random(19)
    trunk(P, 3.4, 0.38, taper=0.55, verts=8)
    conifer(rng, 6, 6.0, 1.4, 1.12, 2.1, shades(P, 4), top_d=0.9, tilt=6, offset=0.2)
    G.cone('tip', 0.25, 0.02, 1.3, at=(0, 0, 8.2), material=P.leaf3, verts=low(8, 5))
    return G.settle([G.join_all('pine')])


def spruce():
    """Blue spruce: a dense blue-green cone of nine tiers right down to the ground."""
    P = palette(['#3A5E52', '#446A5C', '#4E7666', '#5A8270'], bark='#8E7A6A')
    rng = random.Random(20)
    trunk(P, 1.2, 0.36, flare=False, verts=7)
    conifer(rng, 9, 5.4, 0.35, 0.98, 1.85, shades(P, 4), top_d=0.7, tilt=3, offset=0.08)
    G.cone('tip', 0.21, 0.02, 1.2, at=(0, 0, 9.6), material=P.leaf4, verts=low(8, 5))
    return G.settle([G.join_all('spruce')])


def cypress():
    """Italian cypress: a narrow dark column of leaves, a little ragged, tapering to a point."""
    P = palette(['#2E5034', '#385C3A', '#426842', '#4C744A'], bark='#8E7A6A')
    rng = random.Random(21)
    cols = shades(P, 4)
    tube('trunk', [(0, 0, 0), (0, 0, 0.8)], 0.14, 0.11, P.bark, verts=7)
    for i, (d, h, z) in enumerate([(2.0, 2.4, 1.6), (1.9, 2.4, 2.9), (1.7, 2.3, 4.1), (1.4, 2.1, 5.3), (1.05, 1.8, 6.3)]):
        lump(f'column{i}', d / 2, (rng.uniform(-0.06, 0.06), rng.uniform(-0.06, 0.06), z - h / 2), cols[i % 4],
             scale=(rng.uniform(0.95, 1.05), rng.uniform(0.95, 1.05), h / d), noise=0.18, seed=i * 1.3, segments=14, rings=7)
    for i in range(low(6, 3)):
        a = i / low(6, 3) * math.tau
        z = 1.6 + i * low(0.85, 1.7)
        rr = 0.75 - i * low(0.07, 0.14)
        lump(f'tuft{i}', 0.35, (rr * math.cos(a), rr * math.sin(a), z - 0.6), cols[(i + 2) % 4], scale=(1, 1, 1.7), noise=0.25,
             seed=i * 2.1)
    G.cone('tip', 0.3, 0.02, 1.3, at=(0, 0, 7.3), material=cols[3], verts=low(8, 5))
    return G.settle([G.join_all('cypress')])


# ---- in a pot -----------------------------------------------------------------------------------------

def lemon_tree():
    """Potted lemon tree: a terracotta pot, a slim trunk and a round crown hung with lemons; one at the front,
    which a resident picks (it shrinks away)."""
    P = palette(['#3A6A30', '#467838', '#548640', '#62944A'], fruit='#F2D23C', fruit2='#F5DC50', fruit3='#E8C42E')
    rng = random.Random(86)
    top = pot(P, 0.3, 0.5, r_bottom=0.22, rim=0.016)
    tube('trunk', [(0, 0, top - 0.02), (0.04, 0.02, 1.45)], 0.035, 0.02, P.bark, verts=6)
    crown('crown', rng, (0, 0, 1.85), (0.66, 0.66, 0.56), 10, shades(P, 4), size=0.62, core=0.8)
    dots(rng, 12, (0, 0, 1.85), (0.66, 0.66, 0.56), 0.13, [P.fruit, P.fruit2, P.fruit3], 'lemon', band=(-0.6, 0.6),
         out=0.92, flat=1.25, segments=low(6, 5), rings=low(4, 3))
    still = G.join_all('lemonTree')
    at = (0.1, -0.6, 1.62)
    return _with_part('lemon_part', still, lambda: lump('picked_lemon', 0.07, at, P.fruit, scale=(1, 1, 1.3), segments=8, rings=5,
                                                        noise=0.05), (at[0], at[1], at[2] + 0.09), 'pick')


# (width, height, depth) as measured. A tree's far version may differ from it by up to `TOLERANCE`.
TOLERANCE = 0.1

FAMILY = G.Family(
    'trees', M,
    [G.Model('oak', oak, size=(5.488, 7.473, 5.23), key='model.tree', tolerance=TOLERANCE, far=far_of(oak)),
     G.Model('maple', maple, size=(4.848, 7.625, 4.971), key='model.tree.maple', tolerance=TOLERANCE, far=far_of(maple)),
     G.Model('birch', birch, size=(3.446, 8.184, 3.293), key='model.tree.birch', tolerance=TOLERANCE, far=far_of(birch)),
     G.Model('cherry', cherry, size=(5.251, 5.454, 5.14), key='model.tree.blossom', tolerance=TOLERANCE, far=far_of(cherry)),
     G.Model('apple', apple, size=(4.467, 5.569, 4.378), key='model.tree.fruit', tolerance=TOLERANCE, far=far_of(apple),
             motions={'uses': {'pick': 'pick'}}),
     G.Model('japaneseMaple', japanese_maple, size=(3.894, 3.722, 4.025), key='model.tree.japaneseMaple', tolerance=TOLERANCE,
             far=far_of(japanese_maple), motions={'uses': {'shape': 'trim'}}),
     G.Model('magnolia', magnolia, size=(4.1, 4.948, 3.805), key='model.tree.magnolia', tolerance=TOLERANCE, far=far_of(magnolia)),
     G.Model('willow', willow, size=(4.482, 6.1, 4.664), key='model.tree.willow', tolerance=TOLERANCE, far=far_of(willow)),
     G.Model('pine', pine, size=(6.026, 8.85, 6.034), key='model.pine', tolerance=TOLERANCE, far=far_of(pine)),
     G.Model('spruce', spruce, size=(5.601, 10.2, 5.567), key='model.spruce', tolerance=TOLERANCE, far=far_of(spruce)),
     G.Model('cypress', cypress, size=(2.12, 7.95, 2.05), key='model.cypress', tolerance=TOLERANCE, far=far_of(cypress)),
     G.Model('lemonTree', lemon_tree, size=(1.288, 2.374, 1.344), key='model.garden.lemonTree', tolerance=TOLERANCE, far=far_of(lemon_tree),
             paint=[('body', 'Pot')], motions={'uses': {'pick': 'pick'}})],
    comment='Blender-built trees (tools/art/blender/families/trees.py): the G5 garden trees, the woods\' oaks, maples, '
            'birches, pines and spruces, and the potted lemon tree. Replaces the primitive-part placeholders of nature.py '
            'by key; listed in the base manifest\'s packs. The placeholder is kept as a fallback if a file fails to load. '
            'Leaves are on `foliage` and trunks on `bark`, which the nature shader sways and tints. Each has a far '
            'version (`far`), which the far woods and the town overview draw. Clips: an apple or a lemon shrinks away '
            'while it is picked, the Japanese maple\'s stray sprigs while it is shaped.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

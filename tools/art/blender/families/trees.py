# The garden's trees (G5): the oak (`model.tree`), maple, silver birch, cherry blossom, apple, Japanese maple,
# magnolia and weeping willow (`model.tree.*`), the pine, blue spruce and Italian cypress (`model.pine`,
# `model.spruce`, `model.cypress`) and the potted lemon tree (`model.garden.lemonTree`).
#
# Broadleaves are real branching skeletons: a swept, tapering trunk on root buttresses, limbs found from where the
# crown's leaf masses are, each forking into curving boughs, and a mass of leafy lumps (`plants.lump`) at each bough's
# tip, lighter on top, with gaps between (`_branches`, `_mass`; the oak first, by hand). Conifers are whorls of
# drooping sprays of needles round a trunk (`conifer`), the cypress a flame of upright sprays. Leaves are on `foliage`
# and trunks on `bark`, which the nature shader sways and tints. Trees stand on every lot and by the thousand in the
# woods, so each keeps under 4,500 triangles, and each has a far version (`far_of`: the same build at low detail,
# every random number drawn, one lump a mass, no boughs; about a fifth of the triangles), which the far woods and
# the town overview draw.
#
# Run:  cargo art trees [--only oak] [--preview]
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
from plants import LOW, _mesh, _with_part, cupped, far_of, low, lump, noise_at, pot, sprigs, tube  # noqa: E402

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


# ---- perches: where the game's birds sit (the entry's `perches`, gamekit/perch.py) ---------------------------
# Found on the built tree, never drawn: lines in from outside towards each clump of its crown (out from the trunk,
# a little up) meet its leaves, and a perch is a few centimetres in from there, on the outer twigs, with nothing
# hanging over it; conifers' on their tiers' rims, a few on bare limbs. Each tree's own random numbers (`seed`),
# so the tree's are as they were and the meshes don't change.

def clump_aims(clumps, rng, per=3, up=(0.17, 0.95)):
    """`per` lines in towards each drawn clump of a crown (`_mass`'s clumps), out from the trunk +- 50 degrees,
    `up` radians up from level (steeper over the crown's top)."""
    aims = []
    for (mx, my, mz), _, t, drawn in clumps:
        if not drawn:
            continue
        out = math.hypot(mx, my) > 0.25
        lift = (up[0], up[1] + 0.35 * max(0.0, t))
        aims += G.perch.around((mx, my, mz), rng, per, lift, math.atan2(my, mx) if out else None, 0.9 if out else math.pi)
    return aims


def tier_aims(rng, feet, per=4):
    """`per` lines in from round each conifer tier towards its axis a little over its foot (`feet`: (x, y, z) each),
    from a little above: they meet the tier near its rim, where its boughs end."""
    aims = []
    for foot in feet:
        aims += G.perch.around(foot, rng, per, (0.15, 0.5))
    return aims


# ---- broadleaves --------------------------------------------------------------------------------------

def _bough(name, pts, r0, r1, P, verts=6, res=3, material=None):
    """A tapering, curving bark tube (`G.sweep`) through Blender `pts`, `r0` across at its start, `r1` at its end."""
    k = r1 / r0
    return G.sweep(name, pts, r0, material=material or P.bark, verts=low(verts, 3), resolution=low(res, 1),
                   scale=lambda w: 1 + (k - 1) * w)


def _bend(a, b, rng, lift=0.25, wander=0.18):
    """Four points from `a` to `b`, bowed up by `lift` x its length and wandering sideways: a limb's line."""
    a, b = Vector(a), Vector(b)
    d = b - a
    side = Vector((-d.y, d.x, 0)).normalized() if d.xy.length > 1e-6 else Vector((1, 0, 0))
    pts = [a]
    for f in (0.33, 0.66):
        pts.append(a + d * f + Vector((0, 0, lift * d.length * math.sin(math.pi * f))) + side * rng.uniform(-wander, wander) * d.length * 0.3)
    return pts + [b]


# The oak's way for the other broadleaves: a swept trunk on root buttresses, limbs found from where the crown's
# leaf masses are (each limb runs to the middle of the masses on its side and forks into a bough to each), a mass of
# clumps at each bough's tip. Every random number is drawn the same at low detail, so a far version's masses stand
# where the model's do.

def _trunk(P, pts, r, scale, seed, verts=10, wobble=0.03, material=None):
    """The trunk in one swept, tapering piece through Blender `pts` (radius `r` times `scale`, one per point), its bark
    wobbled by noise sideways (not on a far version: its foot stays where the model's is)."""
    stem = G.sweep('trunk', pts, r, material=material or P.bark, verts=low(verts, 5), resolution=low(3, 1), scale=scale)
    if not G.is_low() and wobble:
        G.jitter(stem, wobble, 0.45, seed=seed, up=0.0)
    return stem


def _roots(P, rng, n, r, h, reach, material=None, verts=5):
    """`n` root buttresses flaring from a trunk `r` round at its foot: from `h` up its side out to the ground about `reach`
    from its middle, ending just above it."""
    for i in range(n):
        a = math.tau * (i + rng.uniform(-0.2, 0.2)) / n
        out = reach * rng.uniform(0.85, 1.15)
        c, s = math.cos(a), math.sin(a)
        _bough(f'root{i}', [(0.3 * r * c, 0.3 * r * s, h), (0.8 * r * c, 0.8 * r * s, h * 0.36), (out * c, out * s, r * 0.2)],
               r * 0.38, r * 0.16, P, verts=verts, res=1, material=material)


def _mass(name, rng, tip, heading, cols, light, out, k=3, size=(0.5, 0.64), ring=0.42, drop=(-0.35, 0.05),
          squash=(0.75, 0.88), noise=0.34, far=1.12, segments=6, rings=5, side=0.86):
    """A leaf mass at a bough's tip (Blender `tip`, the bough heading `heading` radians): a clump on the tip and `k` - 1
    round it, `ring` out and `drop` below, each `size` metres round (`side` x that off the tip), squashed by `squash`,
    lighter as `light(z)` (0..1) rises. A far version draws one clump spanning the mass's (`far` x their reach), so it
    keeps the crown's size. Appends each clump to `out`: (middle, half sizes, -1..1 up the crown, drawn), for the
    perches (`clump_aims`) and what sits on the leaves (blooms)."""
    tip = Vector(tip)
    lo, hi, first = [math.inf] * 3, [-math.inf] * 3, None
    for m in range(k):
        if m == 0:
            at = tip
        else:
            b = heading + (m - k / 2) * (3.2 / max(1, k - 1)) + rng.uniform(-0.4, 0.4)
            at = tip + Vector((ring * math.cos(b), ring * math.sin(b), rng.uniform(*drop)))
        s = rng.uniform(*size) * (1.0 if m == 0 else side)
        sx, sy, sq = rng.uniform(0.9, 1.15), rng.uniform(0.9, 1.15), rng.uniform(*squash)
        li = max(0.0, min(1.0, light(at.z)))
        col = cols[max(0, min(len(cols) - 1, round(li * (len(cols) - 1) + rng.uniform(-0.6, 0.6))))]
        seed = len(out) * 1.7 + rng.random()
        half = (s * sx, s * sy, s * sq)
        out.append(((at.x, at.y, at.z), half, 2 * li - 1, True))
        lo = [min(lo[j], at[j] - half[j]) for j in range(3)]
        hi = [max(hi[j], at[j] + half[j]) for j in range(3)]
        if m == 0:
            first = (col, seed)
        if not G.is_low():
            lump(f'{name}{len(out)}', s, (at.x, at.y, at.z - s * sq), col, scale=(sx, sy, sq), noise=noise, seed=seed,
                 segments=segments, rings=rings)
    if G.is_low():
        rx, ry, rz = ((hi[j] - lo[j]) / 2 * far for j in range(3))
        lump(f'{name}{len(out)}', rx, ((lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2, (lo[2] + hi[2]) / 2 - rz), first[0],
             scale=(1, ry / rx, rz / rx), noise=noise, seed=first[1], segments=segments, rings=rings)


def _branches(P, rng, cols, centre, radii, tips, limbs, base_z, leader_z=None, bottom=-0.3, spread=0.9, axis=0.35,
              limb_r=(0.12, 0.05), bough_r=(0.05, 0.018), fork=(0.45, 0.6), lift=0.15, wander=0.2, rise=0.15,
              verts=5, bough_verts=4, material=None, mass=None, prefix=''):
    """
    A broadleaf's limbs and leaf masses. `tips` bough tips spread evenly over the crown's ellipsoid (`centre`, `radii`;
    `spread` x out, from its top down to `bottom`, -1..1); `limbs` limbs from the trunk at heights `base_z` (a range),
    each carrying the tips on its side: it runs `fork` of the way to their middle, bowed up by `lift` and wandering
    `wander`, and forks into a bough to each tip, bowed up `rise`. The tips over the middle (nearer the axis than `axis`
    x the radius) go on a leader from `leader_z` (none: the nearest limb's). A leaf mass at each tip (`_mass`,
    `mass` its options). Returns (the clumps, as `_mass` lists them; the limbs as (their line, radius at each end)). A far version leaves
    out the boughs (drawing their numbers).
    """
    cx, cy, cz = centre
    rx, ry, rz = radii
    material = material or P.bark
    full = not G.is_low()
    ga = math.pi * (3 - math.sqrt(5))
    off = rng.random() * math.tau
    points = []
    for i in range(tips):
        t = 1 - (i + 0.5) / tips * (1 - bottom)
        ring = math.sqrt(max(0.0, 1 - t * t))
        a = off + ga * i + rng.uniform(-0.3, 0.3)
        k = spread * rng.uniform(0.9, 1.08)
        points.append((Vector((cx + rx * ring * math.cos(a) * k, cy + ry * ring * math.sin(a) * k, cz + rz * t * k)), a, ring * k))
    heads = [off + math.tau * (j + 0.5) / limbs + rng.uniform(-0.3, 0.3) for j in range(limbs)]
    groups = [[] for _ in range(limbs + 1)]
    for p, a, rr in points:
        if leader_z is not None and rr < axis:
            groups[-1].append(p)
        else:
            groups[min(range(limbs), key=lambda j: abs((a - heads[j] + math.pi) % math.tau - math.pi))].append(p)

    def light(z):
        return (z - (cz - rz)) / (2 * rz)
    clumps, made = [], []
    for j, group in enumerate(groups):
        z0 = rng.uniform(*base_z) if j < limbs else leader_z
        if not group:
            continue
        base = Vector((0.0, 0.0, z0))
        mid = sum(group, Vector()) / len(group)
        f = base + (mid - base) * rng.uniform(*fork)
        r0 = limb_r[0] * (1.15 if j == limbs else 1.0)
        line = _bend(base, f, rng, lift=lift, wander=wander)
        _bough(f'{prefix}limb{j}', line, r0, limb_r[1], P, verts=verts, res=2, material=material)
        made.append((line, r0, limb_r[1]))
        for i, tip in enumerate(group):
            m = f + (tip - f) * 0.5 + Vector((rng.uniform(-0.15, 0.15), rng.uniform(-0.15, 0.15), rise * (tip - f).length))
            if full:
                _bough(f'{prefix}b{j}_{i}', [f, m, tip], bough_r[0], bough_r[1], P, verts=bough_verts, res=2, material=material)
            _mass(f'{prefix}clump', rng, tip, math.atan2(tip.y, tip.x), cols, light, clumps, **(mass or {}))
    return clumps, made


def _along(line, values, z):
    """A value given at each point of an upright `line` (Blender points), at height `z`, linear between."""
    for (a, va), (b, vb) in zip(zip(line, values), zip(line[1:], values[1:])):
        if z <= b[2]:
            return va + (vb - va) * max(0.0, (z - a[2]) / (b[2] - a[2]))
    return values[-1]


def _hearts(rng, cols, n, centre, radii, size, out=0.4, prefix='heart'):
    """`n` dark lumps round the crown's middle, `out` x its radius out: they close its heart, the gaps between the masses
    stay."""
    cx, cy, cz = centre
    off = rng.random() * math.tau
    for i in range(n):
        a = off + math.tau * (i + 0.5) / n + rng.uniform(-0.3, 0.3)
        s, seed = size * rng.uniform(0.9, 1.1), rng.random() * 9
        at = (cx + radii[0] * out * math.cos(a), cy + radii[1] * out * math.sin(a), cz + radii[2] * rng.uniform(-0.45, 0.05) - s * 0.85)
        lump(f'{prefix}{i}', s, at, cols[rng.randrange(2)], scale=(1, 1, 0.85), noise=0.3, seed=seed, segments=6, rings=5)


def limb_points(limbs, along=(0.45, 0.6, 0.75)):
    """Points on top of the limbs `_branches` returns, `along` them (for perches on bare limbs)."""
    out = []
    for line, r0, r1 in limbs:
        for f in along:
            i = min(int(f * (len(line) - 1)), len(line) - 2)
            k = f * (len(line) - 1) - i
            d = (line[i + 1] - line[i]).normalized()                  # (on a steep limb its top is higher over its axis)
            lift = (r0 + (r1 - r0) * f) / max(0.35, math.sqrt(max(0.0, 1 - d.z * d.z)))
            out.append(line[i] + (line[i + 1] - line[i]) * k + Vector((0, 0, lift + 0.01)))
    return out


def oak():
    """Oak: a stout trunk flared on root buttresses, branching low into five wide limbs that fork into boughs, each
    bough ending in a mass of leaf clumps, lighter on top and darker below: a broad, broken crown whose limbs show
    through the gaps between its masses."""
    P = palette(['#558238', '#64923E', '#76A246', '#8AB250', '#9EC25C'])
    cols = shades(P)
    rng = random.Random(11)
    full = not G.is_low()
    # the trunk: a slight lean and wave, flared at the foot, its base flat on the ground
    trunk_pts = [(0, 0, 0), (0, 0, 0.3), (0.05, -0.03, 1.1), (-0.02, 0.04, 1.9), (0.03, 0.0, 2.7)]
    stem = G.sweep('trunk', trunk_pts, 0.3, material=P.bark, verts=low(11, 5), resolution=low(3, 1),
                   scale=[1.5, 1.08, 0.95, 0.88, 0.72])
    if full:
        G.jitter(stem, 0.035, 0.45, seed=4.0, up=0.0)
    # root buttresses running out from the foot, ending just above the ground
    for i in range(5):
        a = math.tau * (i + rng.uniform(-0.2, 0.2)) / 5
        reach = rng.uniform(0.5, 0.7)
        root = [(0.12 * math.cos(a), 0.12 * math.sin(a), 0.55), (0.34 * math.cos(a), 0.34 * math.sin(a), 0.2),
                (reach * math.cos(a), reach * math.sin(a), 0.085)]
        _bough(f'root{i}', root, 0.17, 0.07, P, verts=5, res=1)
    # five limbs from the top of the trunk, wide and only a little up; a leader carries on up the middle
    off = rng.random() * math.tau + math.pi / 2                       # (turned: its broad side across)
    ends = []
    for i in range(5):
        a = off + math.tau * i / 5 + rng.uniform(-0.25, 0.25)
        z0 = rng.uniform(1.95, 2.6)
        out, up = rng.uniform(1.15, 1.45), rng.uniform(3.6, 4.4)
        end = Vector((out * math.cos(a), out * math.sin(a), up))
        _bough(f'limb{i}', _bend((0.02, 0.0, z0), end, rng, lift=0.15), 0.16, 0.075, P, verts=5)
        ends.append((a, end, out))
    leader = Vector((0.1, -0.05, 4.9))
    _bough('leader', _bend((0.03, 0.0, 2.6), leader, rng, lift=0.0, wander=0.3), 0.17, 0.07, P, verts=5)
    # each limb forks into boughs, out and up; the leader into boughs fanning over the top. A mass of clumps at
    # each bough's tip.
    forks = [(f'b{i}_', end, a, 2 if i % 2 else 3, (1.65, 1.95), (0.75, 1.95)) for i, (a, end, _) in enumerate(ends)]
    forks.append(('top_', leader, off, 4, (0.5, 1.2), (1.55, 2.2)))
    clumps, n = [], 0
    for prefix, base, a0, count, reach, rise in forks:
        for j in range(count):
            a = (a0 + (j - (count - 1) / 2) * 0.75 + rng.uniform(-0.2, 0.2)) if prefix != 'top_' else \
                (a0 + math.tau * j / count + rng.uniform(-0.3, 0.3))
            out = rng.uniform(*reach)
            tip = Vector((out * math.cos(a), out * math.sin(a), base.z + rng.uniform(*rise)))
            mid = base + (tip - base) * 0.5 + Vector((rng.uniform(-0.15, 0.15), rng.uniform(-0.15, 0.15), 0.2))
            if full:
                _bough(f'{prefix}{j}', [base, mid, tip], 0.07, 0.025, P, verts=5, res=2)
            # the mass: a clump on the tip and two round it, a little below and out
            for m in range(3):
                if m == 0:
                    at = tip
                else:
                    b = a + (m - 1.5) * 1.6 + rng.uniform(-0.4, 0.4)
                    at = tip + Vector((0.42 * math.cos(b), 0.42 * math.sin(b), rng.uniform(-0.35, 0.05)))
                size = rng.uniform(0.5, 0.64) * (1.0 if m == 0 else 0.86)
                sx, sy, sq = rng.uniform(0.9, 1.15), rng.uniform(0.9, 1.15), rng.uniform(0.75, 0.88)
                light = max(0.0, min(1.0, (at.z - 3.4) / 3.4))
                col = cols[max(0, min(len(cols) - 1, round(light * (len(cols) - 1) + rng.uniform(-0.6, 0.6))))]
                seed = n * 1.7 + rng.random()
                half = (size * sx, size * sy, size * sq)
                clumps.append(((at.x, at.y, at.z), half, 2 * light - 1, True))
                if not full:                                         # far: one bigger clump stands for the mass
                    if m:
                        n += 1
                        continue
                    size *= 1.4
                    half = (size * sx, size * sy, size * sq)
                lump(f'clump{n}', size, (at.x, at.y, at.z - half[2]), col, scale=(sx, sy, sq), noise=0.34, seed=seed,
                     segments=6, rings=5)
                n += 1
    # the crown's heart, dark, round the leader: it closes the middle, the gaps between the masses stay
    for i in range(4):
        a = off + math.tau * (i + 0.5) / 4 + rng.uniform(-0.3, 0.3)
        size, seed = rng.uniform(0.75, 0.9), rng.random() * 9
        at = (0.75 * math.cos(a), 0.75 * math.sin(a), rng.uniform(4.0, 4.7))
        lump(f'heart{i}', size, at, cols[rng.randrange(2)], scale=(1, 1, 0.85), noise=0.3, seed=seed, segments=6, rings=5)
    still = G.join_all('oak')
    G.perch.find(24, clump_aims(clumps, random.Random(111)))
    return G.settle([still])


def maple():
    """Maple (a Norway maple): a straight trunk with a leader and six steep limbs forking into boughs under one dense,
    even dome of eighteen leaf masses round a dark heart, in a cool, deep green; not the oak's few big billows."""
    P = palette(['#3E6E34', '#4A7C3A', '#588A42', '#68984A', '#7AA654'])
    cols = shades(P)
    rng = random.Random(12)
    # a straight trunk, a little flared, carrying on as the leader into the crown
    _trunk(P, [(0, 0, 0), (0, 0, 0.3), (0.02, -0.01, 1.4), (-0.01, 0.02, 2.6), (0.0, 0.0, 3.5)], 0.24,
           [1.4, 1.05, 0.98, 0.9, 0.78], seed=5.0)
    _roots(P, rng, 5, 0.33, 0.45, 0.55)
    # steep limbs all round from 2.5-3.4 m, forking into boughs that fill an even dome; a dark heart closes it
    centre, radii = (0, 0, 4.95), (1.9, 1.9, 1.85)
    clumps, _ = _branches(P, rng, cols, centre, radii, 18, 6, (2.4, 3.2), leader_z=3.3, bottom=-0.5, spread=0.98,
                          limb_r=(0.1, 0.045), bough_r=(0.042, 0.015), fork=(0.45, 0.55), lift=0.1, rise=0.12,
                          mass=dict(k=3, size=(0.5, 0.62), ring=0.4, drop=(-0.25, 0.1), squash=(0.82, 0.95), noise=0.26))
    _hearts(rng, cols, 6, centre, radii, 1.15, out=0.36)
    still = G.join_all('maple')
    G.perch.find(24, clump_aims(clumps, random.Random(112), per=2))
    return G.settle([still])


def birch():
    """Silver birch: a slim white stem with dark lenticels, black and fissured at its foot, slender white limbs up it
    from 3 m and an airy, upright crown of small masses hanging from their twigs."""
    P = palette(['#76963E', '#86A646', '#98B650', '#AAC45C', '#BCD26A'], bark='#8E867C')
    cols = shades(P)
    rng = random.Random(13)
    full = not G.is_low()
    # a slim white stem with a slight lean and wave all the way up as the leader, black and fissured at its foot
    line = [(0, 0, 0), (0, 0, 0.6), (0.04, -0.02, 2.2), (0.02, 0.03, 4.0), (-0.03, 0.02, 5.8), (0.0, -0.02, 7.6)]
    scale = [1.12, 1.0, 0.88, 0.7, 0.5, 0.28]
    G.sweep('trunk', line, 0.15, material=P.birch_bark, verts=low(9, 5), resolution=low(3, 1), scale=scale)
    foot = G.sweep('foot', [(0, 0, 0), (0, 0, 0.35), (0.01, 0, 0.75)], 0.172, material=P.bark, verts=low(10, 5),
                   resolution=1, scale=[1.04, 0.94, 0.86], caps=False)
    if full:
        G.jitter(foot, 0.018, 0.12, seed=3.0, up=0.0)
        for v in foot.data.vertices:                                   # its top edge ragged, as the black bark climbs
            if v.co.z > 0.5:
                v.co.z += 0.3 * noise_at(v.co.x * 14, v.co.y * 14, 1.7)
    # the dark marks: short lenticels across the bark, and a scar under each limb
    for i in range(22):
        z = 0.9 + i * 0.27 + rng.uniform(-0.1, 0.1)
        a, span = rng.uniform(0, math.tau), rng.uniform(0.9, 1.8)
        rr = 0.15 * _along(line, scale, z) + 0.004
        if full:
            G.sweep(f'mark{i}', [(rr * math.cos(a + span * (k / 3 - 0.5)), rr * math.sin(a + span * (k / 3 - 0.5)), z) for k in range(4)],
                    0.024, material=P.birch_mark, verts=3, scale=[0.4, 1.0, 1.0, 0.4])
    # slender limbs up the stem from 3 m, rising, then their twigs hanging in long, light masses: an airy, upright crown
    centre, radii = (0, 0, 5.75), (1.28, 1.28, 2.15)
    clumps, made = _branches(P, rng, cols, centre, radii, 16, 7, (3.1, 6.0), leader_z=6.2, bottom=-0.55, spread=0.95,
                             axis=0.3, limb_r=(0.055, 0.025), bough_r=(0.024, 0.01), fork=(0.55, 0.7), lift=0.3,
                             wander=0.25, rise=0.05, material=P.birch_bark,
                             mass=dict(k=3, size=(0.34, 0.44), ring=0.3, drop=(-0.5, -0.15), squash=(1.1, 1.35),
                                       noise=0.36, side=0.9))
    _hearts(rng, cols, 3, centre, radii, 0.62, out=0.25)
    still = G.join_all('birch')
    G.perch.find(20, clump_aims(clumps, random.Random(113), per=2), limbs=limb_points(made), limbs_n=2)
    return G.settle([still])


def cherry():
    """Cherry blossom: a short, leaning trunk opening into wide, low limbs, and a broad, flat crown of pink blossom
    clouds at their boughs' tips."""
    P = palette(['#E07AAE', '#EA90BE', '#F2A8CE', '#F8BEDC', '#FCD4E8'], bark='#B09A8A')
    cols = shades(P)
    rng = random.Random(14)
    # a short trunk, leaning a little, that opens at 1.6-2 m into wide, low limbs reaching out nearly level and
    # turning up at their ends: a broad vase under a flat crown of blossom clouds
    _trunk(P, [(0, 0, 0), (0, 0, 0.3), (0.05, 0.02, 1.0), (0.1, 0.0, 1.8)], 0.225, [1.4, 1.05, 0.98, 0.88], seed=6.0)
    _roots(P, rng, 5, 0.3, 0.4, 0.55)
    centre, radii = (0, 0, 3.85), (1.95, 1.95, 1.15)
    clumps, _ = _branches(P, rng, cols, centre, radii, 16, 5, (1.55, 1.95), leader_z=None, bottom=-0.45, spread=0.95,
                          limb_r=(0.11, 0.05), bough_r=(0.045, 0.016), fork=(0.5, 0.62), lift=0.22, wander=0.3, rise=0.2,
                          mass=dict(k=3, size=(0.5, 0.62), ring=0.42, drop=(-0.25, 0.05), squash=(0.6, 0.72), noise=0.36))
    _hearts(rng, cols, 3, centre, radii, 0.75, out=0.35)
    lump('leaves', 0.4, (2.0, -0.9, 3.25), P.green, scale=(1, 1, 0.62), seed=1.0)
    lump('leaves2', 0.35, (-1.8, 1.2, 3.38), P.green2, scale=(1, 1, 0.64), seed=2.0)
    still = G.join_all('cherry')
    green = [((2.0, -0.9, 3.5), None, 0.0, True), ((-1.8, 1.2, 3.6), None, 0.0, True)]   # the two green clumps
    G.perch.find(22, clump_aims(clumps + green, random.Random(114)))
    return G.settle([still])


# A resident picking fruit (`pickApple*`, `pickLemon*` in web/src/render/shared/characters/clips/uses.ts, fitted to
# where the picked fruit hangs): it shrinks away as the hand closes on it, this long into the use.
PICK_DELAY = 1.6

# A fruit tree's fruit follows its ripeness (content `grows`, G-68): the fruit hung over the crown and the one at the
# front a resident picks (`pick`: it shrinks away) are moving parts with one growth clip, `ripen` (`motions.growth`,
# G.stages): in fruit at rest; picked bare, and growing its fruit again, none (they shrink into the crown's middle).
BARE = dict(picked=0.001, growing=0.001, planted=0.001)


def _fruiting(name, still, crown_mid, fruit, picked=None, at=None):
    """The still part, and the fruit `fruit()` hangs over the crown (origin `crown_mid`) as a moving part
    following the tree's ripeness; with `picked`, the one at the front it makes (origin `at`) a second one that
    also shrinks away while it's picked (an apple picked to eat). The far version all one still part, in fruit."""
    if LOW[0]:
        fruit()
        if picked:
            picked()
        return G.settle([G.join_all(still.name)])
    with G.new_parts() as parts:
        fruit()
    hung = G.join(parts, f'{name}_fruit', origin=crown_mid)
    if not picked:
        G.settle([still, hung])
        G.stages(hung, 'ripen', **BARE)
        return [still, hung]
    with G.new_parts() as parts:
        picked()
    front = G.join(parts, f'{name}_part', origin=at)
    G.settle([still, hung, front])
    G.squash(front, 'pick', 0.001, seconds=0.8)
    G.stages(hung, 'ripen', **BARE)
    G.stages(front, 'ripen', **BARE)
    return [still, hung, front]


# The apple tree's low fruit (I-garden-11, I-garden-12): a garden apple (a semi-dwarf) carries its fruit from about
# 1.1 m up, so its crown's middle stands at 2.6 m (its leaves from 1.6 m up) and laden boughs hang out of it in front,
# under the crown's edge, apples on spurs along them. Each apple is (to the right of a picker facing the tree, out in front of the
# trunk, its bottom), in metres. `PICKED` is the one a resident picks to eat (its own part: it shrinks away), low
# enough for every size of picker, a child on tiptoe too (`FRUIT.apple` and `pickApple*` in characters/clips/uses.ts).
# The harvest's reaches (`pickHigh*` in the same file, from the tile in front, 1 m out) close on `HARVEST`'s: a
# grown-up's, a teen's and a child's. Placed, the tree isn't turned or sized at random (no `vary` in the base
# manifest), so the boughs face the tile in front at these heights.
PICKED = (-0.06, 0.62, 1.13)
HARVEST = {'177': (0.16, 0.62, 1.75), '147': (0.3, 0.62, 1.45), '112': (0.24, 0.78, 1.12)}
OVER = 0.27      # a bough passes this far over the bottom of each apple it carries (an apple 0.2 m, its stalk)
# The boughs: from the trunk out over their apples (named) to a tip, through points (r, out, h) as above, leaves near
# the trunk and at the tip. They end within 0.85 m of the trunk, short of where a picker stands (1 m out).
LOW_BOUGHS = [
    # straight out in front over the picked apple, dipping
    [(-0.02, 0.1, 1.48), (-0.07, 0.36, 1.62), 'picked', (-0.12, 0.74, 1.38)],
    # to the right, up under the crown: the grown-up's apple and another
    [(0.06, 0.1, 1.74), (0.12, 0.34, 1.98), '177', 'a', (0.4, 0.84, 1.9)],
    # to the right and low, bowed down by its fruit: the teen's and the child's
    [(0.08, 0.1, 1.44), (0.27, 0.32, 1.64), '147', '112', (0.2, 0.84, 1.36)],
    # to the left, two more
    [(-0.06, 0.1, 1.6), (-0.34, 0.28, 1.84), 'b', 'c', (-0.68, 0.58, 1.6)],
]
LOW_APPLES = {'a': (0.34, 0.78, 1.66), 'b': (-0.45, 0.4, 1.55), 'c': (-0.6, 0.5, 1.42)}


def _low_boughs(P, cols):
    """The laden boughs out of the apple tree's crown in front (`LOW_BOUGHS`): each a bark sweep from the trunk over
    its apples, with leaves where it leaves the crown and a sprig at its tip."""
    named = {'picked': PICKED, **HARVEST, **LOW_APPLES}
    for i, path in enumerate(LOW_BOUGHS):
        pts = []
        for p in path:
            r, out, h = (named[p][0], named[p][1], named[p][2] + OVER) if isinstance(p, str) else p
            pts.append((-r, -out, h))
        _bough(f'low_bough{i}', pts, 0.034, 0.012, P, verts=4, res=2)
        near, tip = pts[1], pts[-1]
        lump(f'low_leaves{i}', 0.2, (near[0], near[1], near[2] - 0.08), cols[1 + i % 2], scale=(1.15, 1.0, 0.75), noise=0.3,
             seed=4.2 + i, segments=6, rings=5)
        lump(f'low_tip{i}', 0.1, (tip[0], tip[1], tip[2] - 0.06), cols[2 + i % 2], scale=(1.2, 1.0, 0.7), noise=0.3,
             seed=7.7 + i, segments=5, rings=3)


def _low_apple(name, at, material, r=0.1, segments=6, rings=4):
    """An apple (radius `r`) hanging from a bough on its stalk, its bottom at `at` (r, out, h as `LOW_BOUGHS`)."""
    x, y, z = -at[0], -at[1], at[2]
    lump(name, r, (x, y, z), material, segments=segments, rings=rings, noise=0.05)
    tube(f'{name}_stalk', [(x, y, z + 2 * r - 0.02), (x + 0.004, y - 0.004, z + OVER + 0.01)], 0.011, 0.008, M.stalk, verts=4)


def apple():
    """Apple tree: a garden apple, a semi-dwarf: a short, crooked trunk, gnarled limbs and a round crown, its middle
    2.6 m up, hung with red and yellow apples (set where lines in meet its leaves, `_hang`), and laden boughs
    out of it in front with apples on them from 1.12 to 1.75 m up (`LOW_BOUGHS`): one a resident picks (it shrinks
    away), low enough for a child on tiptoe, and those the harvest's reaches close on. Picked bare, no apples until it
    fruits again (`ripen`)."""
    P = palette(['#52803A', '#629040', '#74A048', '#86AE52', '#98BC5E'])
    cols = shades(P)
    rng = random.Random(15)
    # a short, crooked trunk opening low into gnarled limbs that spread wide, a round crown on them
    _trunk(P, [(0, 0, 0), (0, 0, 0.25), (0.07, 0.04, 0.75), (0.02, 0.08, 1.15), (0.05, 0.05, 1.5)], 0.2,
           [1.4, 1.05, 1.0, 0.92, 0.85], seed=7.0, wobble=0.04)
    _roots(P, rng, 4, 0.28, 0.4, 0.5)
    centre, radii = (0, 0, 2.6), (1.5, 1.5, 1.15)
    clumps, _ = _branches(P, rng, cols, centre, radii, 13, 5, (1.2, 1.5), leader_z=1.5, bottom=-0.4, spread=0.95,
                          limb_r=(0.1, 0.045), bough_r=(0.042, 0.015), fork=(0.5, 0.62), lift=0.22, wander=0.45, rise=0.15,
                          mass=dict(k=3, size=(0.5, 0.62), ring=0.4, drop=(-0.15, 0.05), squash=(0.78, 0.9), noise=0.32))
    _hearts(rng, cols, 3, centre, radii, 0.75, out=0.35)
    _low_boughs(P, cols)
    still = G.join_all('apple')
    G.perch.find(20, clump_aims(clumps, random.Random(115), per=3))                  # (on the leaves, before the fruit)

    def fruit():
        _hang(rng, low(9, 6), centre, radii, 0.2, [P.fruit, P.fruit2, P.fruit3, P.fruit4], 'apple', band=(-0.7, 0.45),
              segments=low(6, 5), rings=low(4, 3))
        for i, at in enumerate([*HARVEST.values(), *LOW_APPLES.values()]):
            _low_apple(f'low_apple{i}', at, [P.fruit, P.fruit2, P.fruit4][i % 3], segments=low(6, 5), rings=low(4, 3))

    def picked():
        _low_apple('picked_apple', PICKED, P.fruit, r=0.11, segments=8, rings=5)
    x, out, h = PICKED
    return _fruiting('apple', still, centre, fruit, picked, (-x, -out, h + 0.11))


def _hang(rng, n, centre, radii, size, materials, name, band=(-0.6, 0.5), flat=1.0, segments=6, rings=4, sink=-0.1):
    """Fruit hung over a crown as it stands: `n` lines in towards the crown's middle (`centre`) from round its ellipsoid
    (`radii`), from `band` (-1..1) up it, each meeting its leaves; a fruit `size` across there, its middle `sink` of itself
    in from them (out, if less than 0), so none floats in a gap or hides inside."""
    for o in bpy.context.scene.objects:
        o.data.update()
    depsgraph = bpy.context.evaluated_depsgraph_get()
    depsgraph.update()
    c = Vector(centre)
    ga = math.pi * (3 - math.sqrt(5))
    spots = []
    for i in range(n):
        t = band[0] + (band[1] - band[0]) * (i + 0.5) / n
        a = i * ga + rng.uniform(-0.3, 0.3)
        ring = math.sqrt(max(0.0, 1 - t * t))
        d = Vector((radii[0] * ring * math.cos(a), radii[1] * ring * math.sin(a), radii[2] * t)).normalized()
        hit, p = bpy.context.scene.ray_cast(depsgraph, c + d * 8, -d, distance=8)[:2]
        if hit:
            spots.append(p - d * size * sink)
    for i, p in enumerate(spots):
        lump(f'{name}{i}', size / 2, (p.x, p.y, p.z - size / 2 * flat), materials[i % len(materials)], scale=(1, 1, flat),
             seed=i * 0.77, segments=segments, rings=rings, noise=0.06)


def japanese_maple():
    """Japanese maple: a short trunk branching at knee height into six slender stems rising in long S curves, a vase of
    flat red clouds in layers at their boughs' tips: a narrow
    first layer inside its tile at chest height, the broad tiers over head height (from 2.1 m), so residents walk past
    and work at it from the next tile; a few stray sprigs on top, which shaping takes off."""
    P = palette(['#6E1C1C', '#8A2424', '#A2302A', '#B83F2F', '#CC5636'], bark='#B09A8A', leaf_light='#B83F2F')
    cols = shades(P)
    rng = random.Random(16)
    full = not G.is_low()
    # a short trunk splitting at knee height into six slender stems, each rising in a long S out to the vase's rim
    _trunk(P, [(0, 0, 0), (0, 0, 0.25), (0.02, 0.0, 0.55)], 0.1, [1.5, 1.1, 1.0], seed=8.0, verts=8, wobble=0.012)
    _roots(P, rng, 3, 0.14, 0.22, 0.26)
    forks = []
    for j in range(6):
        a = math.tau * (j + 0.5) / 6 + rng.uniform(-0.25, 0.25)
        r, z = rng.uniform(0.5, 0.72), rng.uniform(2.2, 2.5)
        c, s_ = math.cos(a), math.sin(a)
        w = rng.uniform(-0.12, 0.12)                                   # the S: out, back in a little, out again
        stem = [(0.03 * c, 0.03 * s_, 0.45), ((0.2 + w) * c - w * s_, (0.2 + w) * s_ + w * c, 1.0),
                ((r * 0.55 - w) * c + w * s_, (r * 0.55 - w) * s_ - w * c, 1.65), (r * c, r * s_, z)]
        _bough(f'stem{j}', stem, 0.055, 0.03, P, verts=5, res=2)
        forks.append((Vector(stem[-1]), a, Vector(stem[2])))
    # the layers: flat red clouds at the boughs' tips, broad at 2.7 m, narrower over it, the narrow first layer at chest
    # height on short shoots inside the tile
    def light(z):
        return (z - 1.3) / 2.6
    clumps = []
    for k, (z, reach, n) in enumerate(((2.7, 1.3, 7), (3.2, 0.95, 5), (3.6, 0.45, 3))):
        off = rng.random() * math.tau
        for i in range(n):
            a = off + math.tau * i / n + rng.uniform(-0.2, 0.2)
            tip = Vector((reach * rng.uniform(0.9, 1.08) * math.cos(a), reach * rng.uniform(0.9, 1.08) * math.sin(a),
                          z + rng.uniform(-0.08, 0.08)))
            f, _, _ = min(forks, key=lambda q: abs((a - q[1] + math.pi) % math.tau - math.pi))
            mid = f + (tip - f) * 0.5 + Vector((0, 0, 0.12 + rng.uniform(-0.04, 0.04)))
            if full:
                _bough(f'bough{k}_{i}', [f, mid, tip], 0.028, 0.012, P, verts=4, res=2)
            _mass('cloud', rng, tip, a, cols, light, clumps, k=3, size=(0.4, 0.5) if k < 2 else (0.36, 0.44), ring=0.34,
                  drop=(-0.12, 0.02), squash=(0.48, 0.58), noise=0.3)
    for i, (f, a, low_pt) in enumerate(forks[::2]):
        tip = Vector((low_pt.x * 0.7, low_pt.y * 0.7, low_pt.z - 0.08))
        _mass('first', rng, tip, a, cols[1:], light, clumps, k=2, size=(0.26, 0.32), ring=0.15, drop=(-0.08, 0.0),
              squash=(0.6, 0.7), noise=0.3)
    still = G.join_all('japaneseMaple')
    G.perch.find(18, clump_aims(clumps, random.Random(116)), gap=0.35)  # (before the sprigs)
    at = (0.0, 0.0, 3.87)
    return _with_part('sprigs', still, lambda: sprigs('sprig', P, at, 0.35, n=6, length=0.22), at, 'trim')


# A magnolia's bloom: a tall cup of petals, open at the top (its inside turned back down to the middle).
GOBLET = [(0, 0), (0.045, 0.01), (0.1, 0.08), (0.12, 0.18), (0.1, 0.26), (0.078, 0.235), (0, 0.13)]


def magnolia():
    """Magnolia: a smooth grey trunk parting low into upswept stems, a rounded pale pink crown of masses with big
    upright white and pink goblets on them."""
    P = palette(['#EAAACA', '#F2BED6', '#F8D2E2', '#FCE2EE', '#FFF2F8'], bark='#D2CBC0', fruit='#FFFFFF', fruit2='#F2A2C4',
                fruit3='#E888B0', green='#6E9A48', green2='#628E40')
    rng = random.Random(17)
    # a smooth grey trunk parting low into upswept stems, a rounded crown on them
    _trunk(P, [(0, 0, 0), (0, 0, 0.3), (0.04, 0.02, 0.9), (0.02, 0.0, 1.3)], 0.18, [1.4, 1.05, 0.98, 0.92], seed=9.0,
           wobble=0.015)
    _roots(P, rng, 4, 0.25, 0.35, 0.45)
    centre, radii = (0, 0, 3.45), (1.42, 1.22, 1.3)
    clumps, _ = _branches(P, rng, shades(P), centre, radii, 11, 5, (1.0, 1.35), leader_z=1.3, bottom=-0.55, spread=0.95,
                          limb_r=(0.085, 0.04), bough_r=(0.036, 0.014), fork=(0.55, 0.68), lift=0.12, wander=0.3, rise=0.1,
                          mass=dict(k=3, size=(0.46, 0.56), ring=0.38, drop=(-0.28, 0.05), squash=(0.78, 0.9), noise=0.3))
    _hearts(rng, shades(P), 2, centre, radii, 0.75, out=0.3)
    lump('leaves', 0.3, (1.5, 0.8, 2.72), P.green, scale=(1, 1, 0.75), seed=1.0)
    lump('leaves2', 0.28, (-1.3, -0.9, 3.44), P.green2, scale=(1, 1, 0.73), seed=2.0)
    # Blooms: upright goblets set into the crown's top and outer shoulders over each clump of its upper part (the
    # branch tips), two to a mass (one on the side of a lower one), white, pink and deep pink as a saucer magnolia's. Each stands where a line out
    # from its clump's middle leaves the crown, sunk a little into it, so none stands off it (a far version: over
    # the clumps it draws, on its smooth core).
    cols = [P.fruit, P.fruit2, P.fruit, P.fruit3]
    for o in bpy.context.scene.objects:
        o.data.update()
    depsgraph = bpy.context.evaluated_depsgraph_get()
    depsgraph.update()
    blooms = []
    for i, ((mx, my, mz), _, t, drawn) in enumerate(clumps[::3]):                 # each mass's tip clump
        if t < -0.7 or not drawn:
            continue
        heading = math.atan2(my, mx)
        for turn, up in ((0.3, 0.92), ((0.9, 0.55), (-1.0, 0.6))[i % 2]) if t >= -0.25 else ((0.0, 0.42),):
            a = heading + turn
            lean = 1 - up                                              # how far down the clump's side: 0 on top
            d = Vector((math.cos(a) * lean, math.sin(a) * lean, up)).normalized()
            hit, at = bpy.context.scene.ray_cast(depsgraph, Vector((mx, my, mz)) + d * 4, -d, distance=6)[:2]
            if hit:
                blooms.append((at - d * 0.05, a, lean))
    for i, (at, a, lean) in enumerate(blooms):
        if G.is_low() and i % 2:                                       # far: half of them, on the tops
            continue
        cupped(f'bloom{i}', GOBLET, at, cols[i % 4], 90 - math.degrees(a), 8 + 40 * lean, segments=low(6, 4))
    still = G.join_all('magnolia')
    green = [((1.5, 0.8, 2.95), None, 0.0, True), ((-1.3, -0.9, 3.65), None, 0.0, True)]   # the two green clumps
    G.perch.find(20, clump_aims(clumps + green, random.Random(117), per=2), gap=0.35)
    return G.settle([still])


def streamer(name, top, heading, reach, drop, width, material):
    """A willow's hanging strand: a thin leafy ribbon from `top` (Blender metres), out `reach` towards `heading`
    (radians) and down `drop`, falling straight by its foot, its face outwards, narrowing to a third of `width`: a
    thin wedge across (three points a row, its ridge out, smooth), so it shades as a soft strip."""
    rows = low(5, 3)
    out = Vector((math.cos(heading), math.sin(heading), 0))
    side = Vector((-out.y, out.x, 0))
    verts, faces = [], []
    for i in range(rows + 1):
        t = i / rows
        c = Vector(top) + out * reach * (1 - (1 - t) ** 3) - Vector((0, 0, drop * t ** 1.15))
        w = width * (1 - 0.66 * t) / 2
        verts += [c - side * w, c + out * min(0.03, w * 0.5), c + side * w]
    for i in range(rows):
        a, b = 3 * i, 3 * (i + 1)
        faces += [(a + j, a + (j + 1) % 3, b + (j + 1) % 3, b + j) for j in range(3)]
    faces += [(0, 2, 1), (3 * rows, 3 * rows + 1, 3 * rows + 2)]
    return _mesh(name, verts, faces, material)


def willow():
    """Weeping willow: a thick, rough trunk on root buttresses, five limbs rising out of it and arching over, a domed
    crown of leaf masses along their arches, and curtains of leaves hanging from them nearly to the ground."""
    P = palette(['#6A8A36', '#7A9A3E', '#8AAA48', '#9CBA54', '#AEC862'])
    rng = random.Random(18)
    cols = shades(P)
    full = not G.is_low()
    _trunk(P, [(0, 0, 0), (0, 0, 0.3), (0.06, 0.03, 1.2), (0.02, -0.04, 2.1), (0.05, 0.0, 2.7)], 0.31,
           [1.4, 1.05, 0.97, 0.9, 0.8], seed=10.0, wobble=0.045)
    _roots(P, rng, 6, 0.43, 0.6, 0.75)
    # the limbs: up and out of the trunk's top, then over in an arch and down at their ends
    made, masses = [], []
    off = rng.random() * math.tau
    for j in range(5):
        a = off + math.tau * j / 5 + rng.uniform(-0.2, 0.2)
        reach, z0, top = rng.uniform(1.85, 2.05), rng.uniform(2.3, 2.6), rng.uniform(4.8, 5.1)
        c, s_ = math.cos(a), math.sin(a)
        line = [Vector((0.1 * c, 0.1 * s_, z0)), Vector((0.45 * reach * c, 0.45 * reach * s_, z0 + (top - z0) * 0.62)),
                Vector((0.72 * reach * c, 0.72 * reach * s_, top)), Vector((reach * c, reach * s_, top - 0.35))]
        _bough(f'limb{j}', line, 0.13, 0.04, P, verts=6, res=2)
        made.append((line[:3], 0.13, 0.07))
        # a bough from the arch's crest over to each side, arching down too
        for k, turn in enumerate((-0.55, 0.55)):
            b = a + turn
            end = Vector((reach * 0.92 * math.cos(b), reach * 0.92 * math.sin(b), top - 0.4 + rng.uniform(-0.1, 0.1)))
            crest = line[1] + (line[2] - line[1]) * 0.5
            mid = (crest + end) / 2 + Vector((0, 0, 0.3))
            if full:
                _bough(f'bough{j}_{k}', [crest, mid, end], 0.045, 0.015, P, verts=4, res=2)
            masses.append((mid, b, 1))
        masses.append((line[2], a, 2))
    masses.append((Vector((0.05, 0.0, 5.0)), off, 3))
    # the crown: leaf masses along the arches' crests and over the middle, drooping
    clumps = []

    def light(z):
        return (z - 4.2) / 1.6
    for i, (at, a, k) in enumerate(masses):
        _mass('crown', rng, at + Vector((0, 0, 0.12)), a, cols, light, clumps, k=k, size=(0.62, 0.74) if k < 3 else (0.85, 0.95),
              ring=0.42 if k < 3 else 0.6, drop=(-0.2, 0.0), squash=(0.6, 0.72), noise=0.3)
    # Hanging curtains: long, thin leafy streamers all round, out of the crown's edge and straight down to about half
    # a metre off the lawn, in two staggered rows with gaps between them, so whoever sits by the trunk shows through.
    n = 60
    for i in range(n):
        a = i / n * math.tau + rng.uniform(-0.04, 0.04)
        outer = i % 2 == 0
        rr = rng.uniform(2.0, 2.25) if outer else rng.uniform(1.6, 1.85)
        top = rng.uniform(4.5, 5.1) if outer else rng.uniform(4.1, 4.6)
        foot = rng.uniform(0.45, 0.8) if outer else rng.uniform(0.9, 1.5)
        heading, reach, width, col = a + rng.uniform(-0.15, 0.15), rng.uniform(0.25, 0.45), rng.uniform(0.17, 0.23), \
            cols[rng.randint(1, 4)]
        if full or outer:                                              # far: the outer row, wider
            streamer(f'strand{i}', (rr * math.cos(a), rr * math.sin(a), top), heading, reach, top - foot,
                     width * low(1.0, 1.5), col)
    still = G.join_all('willow')
    # On the crown's top, from above (lower, the lines would meet the curtains), and on the limbs under it, inside
    # the curtain.
    G.perch.find(20, clump_aims(clumps, random.Random(118), per=4, up=(0.55, 1.2)),
                 limbs=limb_points(made, (0.3, 0.45, 0.6)), limbs_n=4)
    return G.settle([still])


# ---- conifers -----------------------------------------------------------------------------------------

def _pad_profile(w, h):
    """A conifer bough's spray across (u up, v across): a ridge along the bough, the needles falling away to each side
    and turning under, like a gull's wing; a far version's a plain wedge."""
    if G.is_low():
        return [(h * 0.5, 0.0), (-0.6 * h, w), (-0.6 * h, -w)]
    return [(h * 0.5, 0.0), (-0.05 * h, 0.6 * w), (-0.6 * h, w), (-0.3 * h, 0.0), (-0.6 * h, -w), (-0.05 * h, -0.6 * w)]


def conifer(P, rng, tiers, base_d, z0, step, h, cols, boughs, top_d=0.5, droop=0.35, lift=0.0, thick=0.32, fill=0.62,
            bare=2, offset=0.1):
    """
    A conifer's tiers from `z0` (the first tier's foot), each `step` above the last, `base_d` across narrowing to
    `top_d`, `h` tall: each a whorl of `boughs[i]` boughs out of the trunk, each a spray of needles (`_pad_profile`)
    along a bough that runs out from high in the tier, level, then droops `droop` x the tier's height to its foot at the
    rim (its tip turned up by `lift`), wide in the middle and narrowing to its tip; a dark cone inside closes the whorl.
    The lowest `bare` tiers show their bare boughs from below. Returns each tier's foot (for perches).
    """
    feet = []
    full = not G.is_low()
    for i in range(tiers):
        t = i / max(1, tiers - 1)
        r = (base_d + (top_d - base_d) * t ** 0.9) / 2 * rng.uniform(0.95, 1.05)
        hh = h * (1 - 0.35 * t)
        foot = z0 + i * step
        cx, cy = rng.uniform(-offset, offset), rng.uniform(-offset, offset)
        n = boughs[i]
        off = rng.random() * math.tau
        top = foot + hh * 0.55
        G.cone(f'tier{i}', r * 0.72, low(0.03, 0.0), hh * 0.85, at=(cx * 0.5, cy * 0.5, foot + 0.15 + hh * 0.425),
               material=cols[0], verts=low(10, 6))
        for k in range(n):
            a = off + math.tau * (k + rng.uniform(-0.2, 0.2)) / n
            reach = r * rng.uniform(0.88, 1.06)
            fall = (top - foot) * droop * 2 * rng.uniform(0.85, 1.15)
            col = cols[(i + k + rng.randint(0, 1)) % len(cols)]
            c, s_ = math.cos(a), math.sin(a)

            def at(f, dz):
                return Vector((cx * f + reach * f * c, cy * f + reach * f * s_, dz))
            line = [at(0.04, top + 0.05), at(0.32, top), at(0.6, top - fall * 0.35), at(0.84, top - fall * 0.8),
                    at(1.0, top - fall + lift * hh)]
            w = min(reach * math.tau / n * 0.62, reach * 0.55) * rng.uniform(0.9, 1.1)
            G.sweep(f'spray{i}_{k}', line if full else line[::2], None, material=col, profile=_pad_profile(w, thick * hh / 2.1),
                    scale=[0.35, 0.9, 1.0, 0.75, 0.25] if full else [0.35, 1.0, 0.3], up=(0, 0, 1))
            if full and i < bare:
                G.sweep(f'bough{i}_{k}', [line[0] + Vector((0, 0, -0.06)), line[2] + Vector((0, 0, -0.1)),
                                          line[3] + Vector((0, 0, -0.12))], 0.035, material=P.bark, verts=4,
                        scale=[1.0, 0.6, 0.3])
        feet.append((cx, cy, foot + 0.2))
    return feet


# A conifer's or the cypress's top, from straight above (a blackbird's song post).
TOP = [((0.02, 0.0, 6.0), (0, 0, 1))]


def pine():
    """Pine: a tall bare trunk and six whorls of drooping sprays narrowing to a point, the lowest from 2.35 m (its bare
    boughs showing from below): residents walk under them."""
    P = palette(['#335C38', '#3D683E', '#477444', '#51804A'], bark='#B59A82')
    rng = random.Random(19)
    _trunk(P, [(0, 0, 0), (0, 0, 0.3), (0.03, 0.02, 2.4), (-0.02, 0.0, 5.0), (0.0, 0.02, 8.4)], 0.19,
           [1.35, 1.0, 0.9, 0.6, 0.15], seed=11.0, verts=8, wobble=0.02)
    _roots(P, rng, 4, 0.26, 0.35, 0.42)
    feet = conifer(P, rng, 6, 6.5, 2.35, 1.0, 2.1, shades(P, 4), [11, 10, 9, 8, 7, 6], top_d=0.9, droop=0.4, lift=0.04,
                   thick=0.36, offset=0.15)
    G.cone('tip', 0.25, 0.02, 1.3, at=(0, 0, 8.55), material=P.leaf3, verts=low(8, 5))
    still = G.join_all('pine')
    G.perch.find(20, tier_aims(random.Random(119), feet), inset=0.04, extra=TOP)
    return G.settle([still])


def spruce():
    """Blue spruce: a dense blue-green cone of nine whorls of drooping sprays, their tips turned up, limbed up to a bare
    trunk to 2.2 m, as a garden spruce by a path is: residents walk under its skirt."""
    P = palette(['#3A5E52', '#446A5C', '#4E7666', '#5A8270'], bark='#8E7A6A')
    rng = random.Random(20)
    _trunk(P, [(0, 0, 0), (0, 0, 0.3), (0.02, 0.0, 2.2), (0.0, 0.02, 6.0), (0.0, 0.0, 9.6)], 0.18,
           [1.35, 1.0, 0.9, 0.55, 0.12], seed=12.0, verts=7, wobble=0.02)
    _roots(P, rng, 4, 0.24, 0.32, 0.4)
    feet = conifer(P, rng, 9, 5.5, 2.2, 0.82, 1.85, shades(P, 4), [9, 9, 8, 8, 7, 6, 5, 4, 4], top_d=0.7, droop=0.45,
                   lift=0.05, thick=0.42, offset=0.06, bare=1)
    G.cone('tip', 0.21, 0.02, 1.2, at=(0, 0, 9.9), material=P.leaf4, verts=low(8, 5))
    still = G.join_all('spruce')
    G.perch.find(22, tier_aims(random.Random(120), feet, per=3), inset=0.04, extra=TOP)
    return G.settle([still])


def cypress():
    """Italian cypress: a pencil of dark leaves about a metre across and eight tall, a lofted flame of foliage covered in
    upright sprays, a little ragged, tapering to a point."""
    P = palette(['#2E5034', '#385C3A', '#426842', '#4C744A'], bark='#8E7A6A')
    rng = random.Random(21)
    cols = shades(P, 4)
    full = not G.is_low()
    _trunk(P, [(0, 0, 0), (0, 0, 0.4), (0.0, 0.01, 1.0)], 0.1, [1.3, 1.0, 0.85], seed=13.0, verts=7, wobble=0.01)
    # the column: a dark flame of foliage, fullest at 1.5-3 m and tapering to a point, its sides a little ragged
    def radius(z):
        if z < 1.4:
            return 0.2 + 0.2 * (z - 0.75) / 0.65
        return 0.4 * (1 - ((z - 1.4) / 6.55) ** 1.6) + 0.02
    zs = [0.75 + (7.95 - 0.75) * (k / 13) ** 0.9 for k in range(14)] if full else [0.75 + 7.2 * k / 7 for k in range(8)]
    rings = []
    for z in zs:
        r = radius(z)
        loop = []
        for v in range(low(10, 6)):
            a = math.tau * v / low(10, 6)
            wob = 1 + (0.12 * noise_at(math.cos(a) * 2, math.sin(a) * 2, z * 1.3) if full else 0.0)
            loop.append((r * wob * math.cos(a), r * wob * math.sin(a), z))
        rings.append(loop)
    rings[-1] = [(0.0, 0.0, 7.95)] * len(rings[-1])                    # (a point at the top)
    G.loft('column', rings, material=cols[0])
    # upright sprays round it in a loose spiral, each a slim flame, lighter towards the top
    ga = math.pi * (3 - math.sqrt(5))
    for i in range(44):
        z = 1.0 + 6.2 * (i + 0.5) / 44 + rng.uniform(-0.1, 0.1)
        a = i * ga + rng.uniform(-0.3, 0.3)
        r = radius(z + 0.3)
        size = min(0.21, 0.08 + r * 0.38) * rng.uniform(0.85, 1.15)
        col = cols[min(3, int((z - 1.0) / 6.2 * 3 + rng.uniform(0, 1.2)))]
        seed = i * 2.1 + rng.random()
        if full or i % 3 == 0:
            lump(f'spray{i}', size * low(1.0, 1.3), (r * 0.82 * math.cos(a), r * 0.82 * math.sin(a), z - size * 1.2), col,
                 scale=(1, 1, 2.6), noise=0.25, seed=seed, segments=6, rings=5)
    still = G.join_all('cypress')
    prng = random.Random(121)
    aims = [a for z in (1.9, 2.7, 3.5, 4.3, 5.1, 5.9, 6.6) for a in G.perch.around((0, 0, z), prng, 6, (0.1, 0.6))]
    G.perch.find(15, aims, gap=0.4, extra=TOP, facing=-0.1)
    return G.settle([still])


# ---- in a pot -----------------------------------------------------------------------------------------

def lemon_tree():
    """Potted lemon tree: a banded terracotta pot, a slim stem forking into a round, clipped crown of small masses hung
    with lemons, two metres tall in all (the ones picked at the front 1.62 m up for a grown-up, 1.42 for a teen and
    1.1 for a child), all picked at once (they shrink away, `ripen`); picked bare, no lemons until it fruits again."""
    P = palette(['#3A6A30', '#467838', '#548640', '#62944A'], fruit='#F2D23C', fruit2='#F5DC50', fruit3='#E8C42E')
    cols = shades(P, 4)
    rng = random.Random(86)
    top = pot(P, 0.3, 0.5, r_bottom=0.22, rim=0.016)
    # a raised band round the pot under its rim
    zb, rb = 0.36, 0.22 + 0.004 + (0.3 - 0.226) * (0.36 - 0.012) / (0.47 - 0.012)
    G.sweep('pot_band', [(rb * math.cos(math.tau * k / 24), rb * math.sin(math.tau * k / 24), zb) for k in range(24)], 0.012,
            material=P.body, verts=low(5, 3), closed=True)
    # a slim stem with a little bend, forking at a metre into four boughs under a round, clipped crown
    _trunk(P, [(0, 0, top - 0.03), (0.01, 0.0, 0.62), (0.04, 0.02, 0.85), (0.03, 0.03, 1.02)], 0.032, [1.2, 1.0, 0.9, 0.8],
           seed=14.0, verts=6, wobble=0.0)
    centre, radii = (0.0, 0.0, 1.5), (0.4, 0.4, 0.34)
    clumps, _ = _branches(P, rng, cols, centre, radii, 12, 4, (0.95, 1.02), leader_z=1.0, bottom=-0.6, spread=0.95,
                          limb_r=(0.02, 0.011), bough_r=(0.011, 0.005), fork=(0.4, 0.5), lift=0.1, wander=0.2, rise=0.1,
                          mass=dict(k=3, size=(0.17, 0.21), ring=0.13, drop=(-0.08, 0.02), squash=(0.8, 0.95), noise=0.3))
    _hearts(rng, cols, 3, centre, radii, 0.24, out=0.3)
    still = G.join_all('lemonTree')
    G.perch.find(12, clump_aims(clumps, random.Random(122), per=2), inset=0.02, gap=0.2)   # (on the leaves, before the lemons)
    at = (0.1, -0.6, 1.62)
    # a teen's and a child's (I-garden-11): lower on the crown's front, where they reach (`FRUIT.lemon`)
    # (each 0.03 m in front of where a line in from the front meets the leaves: at 1.51 m up for the teen's, at 1.29 m,
    # the crown's underside, for the child's, which hangs under it)
    teen_at = (0.15, -0.566, 1.42)
    low_at = (-0.12, -0.4, 1.1)

    def fruit():
        _hang(rng, 12, centre, radii, 0.13, [P.fruit, P.fruit2, P.fruit3], 'lemon', band=(-0.6, 0.6), flat=1.25,
              segments=low(6, 5), rings=low(4, 3))
        lump('picked_lemon', 0.07, at, P.fruit, scale=(1, 1, 1.3), segments=8, rings=5, noise=0.05)
        lump('teen_lemon', 0.07, teen_at, P.fruit3, scale=(1, 1, 1.3), segments=8, rings=5, noise=0.05)
        lump('low_lemon', 0.07, low_at, P.fruit2, scale=(1, 1, 1.3), segments=8, rings=5, noise=0.05)
    return _fruiting('lemon', still, centre, fruit)


# (width, height, depth) as measured. A tree's far version may differ from it by up to `TOLERANCE`.
TOLERANCE = 0.1

FAMILY = G.Family(
    'trees', M,
    [G.Model('oak', oak, size=(5.488, 7.473, 5.23), key='model.tree', tolerance=TOLERANCE, far=far_of(oak), habitat='tree'),
     G.Model('maple', maple, size=(4.983, 7.312, 5.335), key='model.tree.maple', tolerance=TOLERANCE, far=far_of(maple), habitat='tree'),
     G.Model('birch', birch, size=(3.446, 8.184, 3.293), key='model.tree.birch', tolerance=TOLERANCE, far=far_of(birch), habitat='tree'),
     G.Model('cherry', cherry, size=(5.251, 5.454, 5.14), key='model.tree.blossom', tolerance=TOLERANCE, far=far_of(cherry), habitat='tree'),
     G.Model('apple', apple, size=(4.29, 4.106, 4.642), key='model.tree.fruit', tolerance=TOLERANCE, far=far_of(apple), habitat='tree',
             motions={'uses': {'pick': {'clip': 'pick', 'delay': PICK_DELAY}, 'harvest': {'clip': 'pick', 'delay': PICK_DELAY}},
                      'growth': 'ripen'}),
     G.Model('japaneseMaple', japanese_maple, size=(3.723, 4.153, 3.828), key='model.tree.japaneseMaple', tolerance=TOLERANCE,
             far=far_of(japanese_maple), habitat='tree', motions={'uses': {'shape': 'trim'}}),
     G.Model('magnolia', magnolia, size=(4.1, 5.076, 3.756), key='model.tree.magnolia', tolerance=TOLERANCE, far=far_of(magnolia), habitat='tree'),
     G.Model('willow', willow, size=(4.947, 6.1, 5.052), key='model.tree.willow', tolerance=TOLERANCE, far=far_of(willow), habitat='tree'),
     G.Model('pine', pine, size=(6.031, 9.2, 6.054), key='model.pine', tolerance=TOLERANCE, far=far_of(pine), habitat='conifer'),
     G.Model('spruce', spruce, size=(5.188, 10.5, 5.154), key='model.spruce', tolerance=TOLERANCE, far=far_of(spruce), habitat='conifer'),
     G.Model('cypress', cypress, size=(1.012, 7.95, 1.025), key='model.cypress', tolerance=TOLERANCE, far=far_of(cypress), habitat='conifer'),
     G.Model('lemonTree', lemon_tree, size=(1.288, 1.994, 1.344), key='model.garden.lemonTree', tolerance=TOLERANCE, far=far_of(lemon_tree), habitat='tree',
             paint=[('body', 'Pot')], motions={'growth': 'ripen'})],
    comment='Blender-built trees (tools/art/blender/families/trees.py): the G5 garden trees, the woods\' '
            'oaks, maples, birches, pines and spruces, and the potted lemon tree. Listed in the base '
            'manifest\'s packs. Leaves are on `foliage` and trunks on `bark`, which the nature shader '
            'sways and tints. Each has a far version (`far`), which the far woods and the town overview '
            'draw. Clips: an apple shrinks away while it is picked, the fruit follows the tree\'s ripeness (picked bare, the lemons all go), the Japanese maple\'s stray '
            'sprigs while it is shaped.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

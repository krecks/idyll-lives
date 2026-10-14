# The garden's potted plants and flowers: G1, the houseplants (`model.plant` and `model.garden.monstera`,
# `fiddleFig`, `snakePlant`, `cacti`, `succulents`, `bostonFern`, `palm`, `orchid`, `bonsai`), over the
# primitive-part placeholders of `furniture.py` and `nature.py`, which stay the fallback.
#
# Leaves are closed, thin, bent blades (`leaf`) on the `foliage` finish, stems and trunks swept tubes
# (`tube`), pots surfaces of revolution (`pot`). The game's nature shader gives every part on `foliage`
# (and `bark`) its wind sway and a per-instance tint (render/babylon/nature.ts); the pots, stands and
# flowers stay still. Each model's pot is its `body`, the part the player colours. Only the bonsai moves:
# a stray sprig shrinks away while a resident prunes it (`trim`).
#
# Run:  python3 -B tools/art/blender/build.py plants [--only monstera] [--preview]
import math
import os
import random
import sys

import bmesh
import bpy
from mathutils import Matrix, Vector

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

# Every model's materials, with their finishes; each model takes its own colours (`palette`).
BASE = dict(
    body=('#C0683F', 'satin'),          # the pot (the bonsai's tray); the main part the player colours
    soil=('#5A4030', 'matte'),
    leaf=('#4F8A3E', 'foliage'),
    leaf_dark=('#3B7034', 'foliage'),
    leaf_light=('#6FAA52', 'foliage'),
    stem=('#6E9248', 'foliage'),
    bark=('#8A7058', 'bark'),
    wood=('#7A5A40', 'wood'),           # plant stands
    flesh=('#4E8A4A', 'satin'),         # cacti and succulents: still, no sway
    flesh_light=('#7EA88A', 'satin'),
    flesh_pink=('#C99AA8', 'satin'),
    bloom=('#F6F0F4', 'satin'),         # flowers
    bloom_accent=('#C2367A', 'satin'),
    sand=('#C8B894', 'matte'),
    moss=('#5E7A34', 'matte'),
)
M = G.Palette(**BASE)


def palette(**colours):
    """The family's palette with some colours changed (finishes stay the family's)."""
    return G.Palette(**{**BASE, **{k: (v, BASE[k][1]) for k, v in colours.items()}})


# ---- shapes ------------------------------------------------------------------------------------------

def _mesh(name, verts, faces, material, smooth=True):
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata([tuple(v) for v in verts], [], faces)
    mesh.validate()
    bm = bmesh.new()
    bm.from_mesh(mesh)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    for p in mesh.polygons:
        p.use_smooth = smooth
    return obj


PROFILES = {
    'oval': lambda t: math.sin(math.pi * t) ** 0.7 * (1 - 0.2 * t),
    'lance': lambda t: math.sin(math.pi * t) ** 0.9 * (1 - 0.45 * t),
    'heart': lambda t: math.sin(math.pi * (0.22 + 0.78 * t)) ** 0.6,
    'strap': lambda t: min(1.0, t * 6) * min(1.0, (1 - t) * 3) ** 0.8,
    'petal': lambda t: math.sin(math.pi * t ** 0.8) ** 0.6,
}


def leaf(name, base, yaw, tilt, length, width, material, shape='oval', droop=0.0, thick=0.008, fold=0.25,
         rows=6, ring=6, notch=None, twist=0.0):
    """
    A closed, thin leaf from `base` (Blender metres), heading `yaw` degrees round from the back (+y) and
    `tilt` degrees up from level, `length` long and `width` across at its widest (`shape`: PROFILES).
    `droop` bends it down evenly along its length, keeping the length (about `droop` x 115 degrees by the
    tip), `fold` lifts its edges into a V,
    `notch` (every, factor) narrows every nth row (a monstera's cuts, a fern's or palm's leaflets),
    `twist` turns it about its midrib by the tip (degrees).
    """
    prof = PROFILES[shape]
    bend = 2 * droop

    def along(t):
        """The midrib's point and the normal square to it at `t`: an arc turning down by `bend` radians."""
        a = -bend * t
        if bend < 1e-4:
            return Vector((0, length * t, 0)), Vector((0, 0, 1))
        return (Vector((0, length * math.sin(bend * t) / bend, -length * (1 - math.cos(bend * t)) / bend)),
                Vector((0, -math.sin(a), math.cos(a))))

    verts = [Vector((0, 0, 0))]
    for i in range(1, rows + 1):
        t = i / (rows + 1)
        hw = width / 2 * prof(t)
        if notch and i % notch[0] == 0:
            hw *= notch[1]
        ht = max(thick / 2 * prof(t), 0.0012)
        c, n = along(t)
        tw = math.radians(twist * t)
        for j in range(ring):
            a = math.tau * j / ring
            x, z = hw * math.cos(a), ht * math.sin(a) + fold * abs(hw * math.cos(a))
            verts.append(c + Vector((x * math.cos(tw) - z * math.sin(tw), 0, 0)) + n * (x * math.sin(tw) + z * math.cos(tw)))
    verts.append(along(1)[0])
    faces, tip = [], len(verts) - 1
    for j in range(ring):
        k = (j + 1) % ring
        faces.append((0, 1 + k, 1 + j))
        last = 1 + (rows - 1) * ring
        faces.append((tip, last + j, last + k))
        for i in range(rows - 1):
            a0, b0 = 1 + i * ring, 1 + (i + 1) * ring
            faces.append((a0 + j, a0 + k, b0 + k, b0 + j))
    m = (Matrix.Translation(base) @ Matrix.Rotation(math.radians(-yaw), 4, 'Z')
         @ Matrix.Rotation(math.radians(tilt), 4, 'X'))
    return _mesh(name, [m @ v for v in verts], faces, material)


def tube(name, points, r0, r1, material, verts=6, cap=True):
    """A swept tube through `points` (Blender metres), radius `r0` at the first and `r1` at the last."""
    pts = [Vector(p) for p in points]
    n = len(pts)
    ring_verts, faces = [], []
    normal = None
    for i, p in enumerate(pts):
        t = (pts[min(i + 1, n - 1)] - pts[max(i - 1, 0)]).normalized()
        if normal is None:
            normal = t.orthogonal().normalized()
        normal = (normal - t * normal.dot(t)).normalized()    # carried along, square to the tangent
        side = t.cross(normal)
        r = r0 + (r1 - r0) * i / (n - 1)
        for j in range(verts):
            a = math.tau * j / verts
            ring_verts.append(p + (normal * math.cos(a) + side * math.sin(a)) * r)
    for i in range(n - 1):
        for j in range(verts):
            k = (j + 1) % verts
            faces.append((i * verts + j, i * verts + k, (i + 1) * verts + k, (i + 1) * verts + j))
    if cap:
        ring_verts += [pts[0], pts[-1]]
        a, b = len(ring_verts) - 2, len(ring_verts) - 1
        for j in range(verts):
            k = (j + 1) % verts
            faces.append((a, k, j))
            faces.append((b, (n - 1) * verts + j, (n - 1) * verts + k))
    return _mesh(name, ring_verts, faces, material)


def arc(a, b, lift, steps=4):
    """Points from `a` to `b` bowed up by `lift` metres in the middle (an arching stem)."""
    a, b = Vector(a), Vector(b)
    return [a + (b - a) * (i / steps) + Vector((0, 0, lift * math.sin(math.pi * i / steps))) for i in range(steps + 1)]


def pot(P, r_top, h, z0=0.0, r_bottom=None, rim=0.012, rim_h=0.03, segments=24, soil=True):
    """A pot standing at height `z0`, `h` tall, its top `r_top` across the rim's inside: returns its soil height."""
    rb = r_bottom if r_bottom is not None else r_top * 0.72
    top = z0 + h
    profile = [(0, z0), (rb, z0), (rb + 0.004, z0 + 0.012), (r_top, top - rim_h)]
    if rim:
        profile += [(r_top + rim, top - rim_h), (r_top + rim, top)]
    else:
        profile += [(r_top, top)]
    inner = r_top - 0.012
    profile += [(inner, top), (inner - 0.006, top - 0.035), (0, top - 0.035)]
    G.revolve('pot', profile, material=P.body, segments=segments)
    if soil:
        G.cylinder('soil', inner - 0.004, 0.01, at=(0, 0, top - 0.03), material=P.soil, verts=segments)
    return top - 0.025


def polar(r, deg, z):
    return (r * math.sin(math.radians(deg)), r * math.cos(math.radians(deg)), z)


# ---- the houseplants ---------------------------------------------------------------------------------

def plant():
    """Potted plant: a terracotta pot with a lush arrowhead plant, three rings of leaves on stalks."""
    P = palette(body='#C46A43', leaf='#4F8A3E', leaf_dark='#5E9A48', leaf_light='#7DB85C', stem='#7FA35A')
    ys = pot(P, 0.19, 0.34, r_bottom=0.14, rim=0.014)
    cols = [P.leaf, P.leaf_dark, P.leaf_light]
    k = 0
    # count, heading offset, stalk length, stalk tilt, leaf length, width, tilt, droop
    for n, off, sl, st, ll, lw, lt, dr in [(8, 0, 0.16, 32, 0.22, 0.17, 2, 0.35), (8, 22, 0.28, 58, 0.23, 0.17, 25, 0.22),
                                           (6, 10, 0.37, 76, 0.22, 0.16, 48, 0.08)]:
        for i in range(n):
            yaw = off + i * 360 / n + (k * 37) % 17
            a = Vector(polar(0.03, yaw, ys))
            tip = a + Vector(polar(sl * math.cos(math.radians(st)), yaw, sl * math.sin(math.radians(st))))
            tube(f'stalk{k}', arc(a, tip, 0.02, 2), 0.006, 0.005, P.stem, verts=4)
            leaf(f'leaf{k}', tip, yaw, lt, ll, lw, cols[k % 3], shape='heart', droop=dr, fold=0.2)
            k += 1
    return G.settle([G.join_all('plant')])


def monstera():
    """Monstera: a white pot and nine split leaves on long arching stems."""
    P = palette(body='#EDEBE6', leaf='#2F6B35', leaf_dark='#28602E', leaf_light='#3B7A3C', stem='#4E7A3A')
    ys = pot(P, 0.21, 0.38, r_bottom=0.16, rim=0)
    rng = random.Random(91)
    for i in range(9):
        yaw = i / 9 * 720 + rng.uniform(-15, 15)
        out = rng.uniform(0.18, 0.34)
        h = rng.uniform(0.2, 0.5)
        tip = Vector(polar(out, yaw, ys + h))
        tube(f'stem{i}', arc((0, 0, ys), tip, 0.06, 3), 0.009, 0.007, P.stem, verts=5)
        leaf(f'leaf{i}', tip, yaw, rng.uniform(20, 40), rng.uniform(0.36, 0.44), rng.uniform(0.32, 0.38),
             [P.leaf, P.leaf_dark, P.leaf_light][i % 3], shape='heart', droop=0.35, fold=0.12, rows=19, ring=6,
             notch=(4, 0.5))
    return G.settle([G.join_all('monstera')])


def fiddle_fig():
    """Fiddle-leaf fig: a cream pot, a slim trunk and big leathery leaves spiralling up it."""
    P = palette(body='#D9CDB4', leaf='#2E5A2A', leaf_dark='#2A5226', leaf_light='#386832', bark='#8A7058')
    ys = pot(P, 0.2, 0.4, r_bottom=0.15, rim=0)
    rng = random.Random(92)
    tube('trunk', [(0, 0, ys - 0.02), (0.01, 0.0, 0.8), (-0.01, 0.01, 1.2), (0.0, 0.0, 1.5)], 0.022, 0.01, P.bark, verts=7)
    n = 19
    for i in range(n):
        t = i / n
        z = ys + 0.3 + t * (1.46 - ys - 0.3)
        yaw = i * 137.5 + rng.uniform(-10, 10)
        s = 1.05 - t * 0.3
        leaf(f'leaf{i}', (0, 0, z), yaw, rng.uniform(15, 40), rng.uniform(0.26, 0.3) * s, rng.uniform(0.19, 0.22) * s,
             [P.leaf, P.leaf_dark, P.leaf_light][i % 3], shape='oval', droop=0.25, thick=0.01, fold=0.18)
    leaf('top', (0, 0, 1.5), 20, 70, 0.14, 0.09, P.leaf_light, shape='oval', droop=0.1)
    return G.settle([G.join_all('fiddleFig')])


def snake_plant():
    """Snake plant: a charcoal pot and stiff, upright, slightly twisting blades."""
    P = palette(body='#2E2E30', leaf='#2F5A34', leaf_dark='#2A4E30', leaf_light='#5E7E3A')
    ys = pot(P, 0.16, 0.3, r_bottom=0.13, rim=0)
    rng = random.Random(93)
    cols = [P.leaf, P.leaf_dark, P.leaf, P.leaf_light]
    for i in range(11):
        x, y = rng.uniform(-0.08, 0.08), rng.uniform(-0.08, 0.08)
        yaw = math.degrees(math.atan2(x, y)) + rng.uniform(-20, 20)
        leaf(f'blade{i}', (x, y, ys - 0.02), yaw, rng.uniform(78, 88), rng.uniform(0.48, 0.66), rng.uniform(0.07, 0.09),
             cols[i % 4], shape='strap', thick=0.016, fold=0.35, rows=6, ring=6, twist=rng.uniform(-50, 50))
    return G.settle([G.join_all('snakePlant')])


def _capsule(name, r, h, at, material, segments=10):
    x, y, z = at
    return G.revolve(name, [(0, 0), (r, 0), (r, h - r * 0.7), (r * 0.72, h - r * 0.18), (0, h)], at=(x, y, z),
                     material=material, segments=segments)


def cacti():
    """Cactus trio: three little terracotta pots, a column cactus with arms, a barrel and a prickly pear."""
    P = palette(body='#C0683F', flesh='#4E8A4A', flesh_light='#6A9E5A', bloom='#E8508A', bloom_accent='#F2C14E')
    # game (x, depth) -> Blender (-x, -depth)
    pots = [((0.17, -0.05), 0.105, 0.18), ((-0.14, 0.08), 0.115, 0.2), ((-0.12, -0.17), 0.08, 0.13)]
    tops = []
    for i, ((x, y), r, h) in enumerate(pots):
        with G.new_parts() as parts:
            top = pot(P, r, h, rim=0.01, rim_h=0.022, segments=16)
        for o in parts:
            o.location.x += x
            o.location.y += y
        tops.append(((x, y), top))
    (x, y), z = tops[0]                              # the column, two arms and a pink flower
    _capsule('column', 0.045, 0.36, (x, y, z - 0.01), P.flesh)
    tube('armL', [(x + 0.04, y, z + 0.12), (x + 0.1, y, z + 0.13), (x + 0.11, y, z + 0.24)], 0.026, 0.024, P.flesh, verts=8)
    G.sphere('armL_top', 0.024, at=(x + 0.11, y, z + 0.24), material=P.flesh, segments=8, rings=5)
    tube('armR', [(x - 0.04, y, z + 0.18), (x - 0.09, y, z + 0.19), (x - 0.095, y, z + 0.27)], 0.022, 0.02, P.flesh, verts=8)
    G.sphere('armR_top', 0.02, at=(x - 0.095, y, z + 0.27), material=P.flesh, segments=8, rings=5)
    G.sphere('flower', 0.018, at=(x, y, z + 0.355), material=P.bloom, segments=8, rings=5, scale=(1, 1, 0.6))
    (x, y), z = tops[1]                              # the barrel, ribbed, with a yellow crown
    G.sphere('barrel', 0.1, at=(x, y, z + 0.07), material=P.flesh_light, segments=14, rings=8, scale=(1, 1, 0.85))
    G.sphere('crown', 0.025, at=(x, y, z + 0.155), material=P.bloom_accent, segments=8, rings=4, scale=(1, 1, 0.6))
    (x, y), z = tops[2]                              # the prickly pear's pads
    for i, (dx, dz, w, h, r) in enumerate([(0, 0.06, 0.11, 0.14, 8), (0.05, 0.16, 0.08, 0.1, -28), (-0.045, 0.14, 0.07, 0.09, 30)]):
        G.sphere(f'pad{i}', 0.5, at=(x + dx, y, z + dz), material=P.flesh_light, segments=10, rings=6,
                 scale=(w, 0.035, h), rot=(0, r, 0))
    return G.settle([G.join_all('cacti')])


def succulents():
    """Succulent bowl: a shallow glazed bowl of sand and seven little rosettes."""
    P = palette(body='#7FA6B8', flesh='#5E8F55', flesh_light='#8FB5A0', flesh_pink='#C99AA8')
    G.revolve('bowl', [(0, 0), (0.19, 0), (0.29, 0.1), (0.31, 0.12), (0.295, 0.12), (0.275, 0.1), (0, 0.09)],
              material=P.body, segments=28)
    G.cylinder('sand', 0.275, 0.01, at=(0, 0, 0.095), material=P.sand, verts=28)
    rng = random.Random(95)
    cols = [P.flesh_light, P.flesh_pink, P.flesh, P.flesh_light, P.flesh_pink, P.flesh, P.flesh_light]
    spots = [(0, 0)] + [(0.17 * math.sin(a), 0.17 * math.cos(a)) for a in (i * math.tau / 6 + 0.3 for i in range(6))]
    for n, (x, y) in enumerate(spots):
        s = rng.uniform(0.85, 1.15) * (1.15 if n == 0 else 1.0)
        for tier, (k, length, tilt) in enumerate([(7, 0.085, 28), (6, 0.065, 50), (4, 0.04, 72)]):
            for i in range(k):
                yaw = i * 360 / k + tier * 25 + n * 13
                leaf(f'r{n}_{tier}_{i}', (x, y, 0.098 + tier * 0.01), yaw, tilt, length * s, length * 0.6 * s, cols[n],
                     shape='oval', thick=0.022 * s, fold=0.15, rows=3, ring=4)
    return G.settle([G.join_all('succulents')])


def boston_fern():
    """Fern on a stand: a white pot on a three-legged wooden stand, long fronds arching over the rim."""
    P = palette(body='#EDEBE6', leaf='#5E9A3E', leaf_dark='#558E38', leaf_light='#78B24E', wood='#7A5A40')
    G.cylinder('stand_top', 0.2, 0.035, at=(0, 0, 0.555), material=P.wood, verts=20)
    for i in range(3):
        a = i * math.tau / 3 + 0.5
        G.rod(f'stand_leg{i}', (0.13 * math.cos(a), 0.13 * math.sin(a), 0.54), (0.18 * math.cos(a), 0.18 * math.sin(a), 0),
              0.018, material=P.wood, verts=8)
    ys = pot(P, 0.145, 0.24, z0=0.572, r_bottom=0.11, rim=0)
    rng = random.Random(96)
    cols = [P.leaf, P.leaf_dark, P.leaf_light]
    for i in range(14):
        yaw = i * 360 / 14 + rng.uniform(-10, 10)
        leaf(f'frond{i}', (0, 0, ys + 0.02), yaw, rng.uniform(35, 60), rng.uniform(0.48, 0.56), 0.13, cols[i % 3],
             shape='lance', droop=rng.uniform(0.75, 0.9), fold=0.15, rows=15, ring=4, notch=(2, 0.45))
    for i in range(6):
        yaw = i * 60 + 30
        leaf(f'crown{i}', (0, 0, ys + 0.02), yaw, 65, 0.3, 0.1, P.leaf_light, shape='lance', droop=0.5, fold=0.15,
             rows=11, ring=4, notch=(2, 0.45))
    return G.settle([G.join_all('bostonFern')])


def palm():
    """Parlour palm: a cream pot and arching stalks, each with a feathery frond."""
    P = palette(body='#D9CDB4', leaf='#4A8A3A', leaf_dark='#3E7A34', leaf_light='#5A9A42', stem='#5E7E3A')
    ys = pot(P, 0.19, 0.34, r_bottom=0.14, rim=0)
    rng = random.Random(97)
    cols = [P.leaf, P.leaf_dark, P.leaf_light]
    for i in range(8):
        yaw = i * 45 + rng.uniform(-15, 15)
        up = rng.uniform(64, 80)
        length = rng.uniform(0.28, 0.46)
        tip = Vector(polar(length * math.cos(math.radians(up)), yaw, ys + length * math.sin(math.radians(up))))
        tube(f'stalk{i}', [(0, 0, ys - 0.01), tip], 0.006, 0.005, P.stem, verts=4)
        leaf(f'frond{i}', tip, yaw, up - 18, rng.uniform(0.4, 0.5), 0.26, cols[i % 3], shape='lance',
             droop=rng.uniform(0.75, 0.95), fold=0.1, rows=17, ring=4, notch=(2, 0.22))
    return G.settle([G.join_all('palm')])


def _flower(name, at, facing, size, P):
    """An orchid flower: five petals round a dark lip, facing `facing` degrees round (yaw)."""
    for i in range(5):
        a = i * 72 + 36
        # petals in the plane square to the facing direction: tilt them up from that plane
        d = Matrix.Rotation(math.radians(-facing), 3, 'Z') @ Vector((math.sin(math.radians(a)), -0.25, math.cos(math.radians(a))))
        yaw = math.degrees(math.atan2(d.x, d.y))
        tilt = math.degrees(math.asin(max(-1, min(1, d.z / d.length))))
        leaf(f"{name}_p{i}", at, yaw, tilt, size, size * 0.8, P.bloom, shape='petal', thick=0.004,
             fold=0.05, rows=3, ring=4)
    G.sphere(f'{name}_lip', size * 0.28, at=at, material=P.bloom_accent, segments=6, rings=4)


def orchid():
    """Orchid: a small white pot on a wooden pedestal, three broad leaves and an arching spike of flowers."""
    P = palette(body='#F2F0EC', leaf='#2E5A30', stem='#4E6A3A', wood='#5A4030', bloom='#F6F0F4', bloom_accent='#C2367A',
                soil='#8A6A48')
    G.cone('pedestal', 0.14, 0.12, 0.6, at=(0, 0, 0.3), material=P.wood, verts=16)
    G.cylinder('pedestal_top', 0.16, 0.03, at=(0, 0, 0.615), material=P.wood, verts=16)
    ys = pot(P, 0.085, 0.16, z0=0.63, r_bottom=0.07, rim=0, segments=18)
    for i, yaw in enumerate((10, 150, 260)):
        leaf(f'leaf{i}', (0, 0, ys + 0.01), yaw, 18, 0.21, 0.09, P.leaf, shape='oval', droop=0.35, thick=0.012, fold=0.2)
    spike = [Vector(p) for p in [(0, 0, ys), (-0.03, 0.01, ys + 0.18), (-0.06, 0.0, ys + 0.34), (-0.16, -0.01, ys + 0.43),
                                 (-0.27, -0.02, ys + 0.4)]]
    tube('spike', spike, 0.005, 0.004, P.stem, verts=4)
    for i in range(6):
        t = 0.42 + i * 0.1
        k = min(len(spike) - 2, int(t * (len(spike) - 1)))
        f = t * (len(spike) - 1) - k
        at = spike[k].lerp(spike[k + 1], f) + Vector((0, -0.03, -0.03))
        _flower(f'flower{i}', at, 10 * (i % 3 - 1), 0.05 if i < 5 else 0.038, P)
    return G.settle([G.join_all('orchid')])


def bonsai():
    """Bonsai: a blue glazed tray on a little wooden table, a twisting trunk, cloud-pruned foliage pads and
    one stray sprig, which shrinks away while a resident prunes it."""
    P = palette(body='#3E5A78', wood='#6A4A32', bark='#6E5A4C', leaf='#3A6A30', leaf_dark='#346028', leaf_light='#467A38',
                stem='#5E5040')
    G.cube('table_top', (0.62, 0.4, 0.04), at=(0, 0, 0.36), material=P.wood, bevel=0.008)
    for x in (-0.27, 0.27):
        for y in (-0.16, 0.16):
            G.cube(f'table_leg{x}{y}', (0.045, 0.045, 0.34), at=(x, y, 0.17), material=P.wood, bevel=0.005)
    G.cube('tray', (0.46, 0.3, 0.07), at=(0, 0, 0.415), material=P.body, bevel=0.012)
    G.cube('moss', (0.42, 0.26, 0.02), at=(0, 0, 0.45), material=P.moss)
    # Bonsai trunks are wood here, not bark: one draw call fewer, and a trunk this short hardly bends in the wind.
    tube('trunk', [(0.02, 0, 0.44), (-0.05, 0.0, 0.58), (0.04, 0.01, 0.7), (-0.06, 0.0, 0.8)], 0.032, 0.016, P.wood, verts=8)
    tube('branch', [(-0.05, 0.0, 0.58), (0.08, 0.02, 0.64), (0.18, 0.03, 0.68)], 0.016, 0.008, P.wood, verts=6)
    tube('branch2', [(0.03, 0.01, 0.69), (-0.12, -0.02, 0.73), (-0.2, -0.03, 0.72)], 0.012, 0.007, P.wood, verts=6)
    for i, (x, y, z, r, sx, sy) in enumerate([(-0.07, 0.0, 0.83, 0.12, 1.4, 1.0), (0.17, 0.03, 0.7, 0.09, 1.3, 1.0),
                                              (-0.19, -0.03, 0.75, 0.08, 1.3, 0.9), (0.02, 0.06, 0.73, 0.07, 1.2, 1.0)]):
        G.boulder(f'pad{i}', r, at=(x, y, z - 0.035), material=[P.leaf, P.leaf_dark, P.leaf_light][i % 3],
                  scale=(sx, sy, 0.45), seed=i * 1.9 + 0.3, noise=0.3, subdivisions=2)
    still = G.join_all('bonsai')
    # The stray sprig, sticking up out of the top pad: a twig and a tuft, all on foliage (one draw call).
    with G.new_parts() as parts:
        tube('sprig_twig', [(-0.06, -0.03, 0.83), (-0.11, -0.09, 0.88), (-0.17, -0.15, 0.92)], 0.006, 0.004, P.stem, verts=4)
        for i, (yaw, tilt) in enumerate([(200, 20), (260, 35), (320, 15), (230, 65), (150, 30)]):
            leaf(f'sprig_leaf{i}', (-0.17, -0.15, 0.915), yaw, tilt, 0.065, 0.04, P.leaf_light, shape='oval', rows=3, ring=4)
    sprig = G.join(parts, 'sprig', origin=(-0.06, -0.03, 0.83))
    G.settle([still, sprig])
    G.squash(sprig, 'trim', 0.001, seconds=0.8)
    return [still, sprig]


# (width, height, depth) as measured; each within 10 % of its placeholder's bounds (check_glb warns beyond).
TOLERANCE = 0.03

FAMILY = G.Family(
    'plants', M,
    [G.Model('plant', plant, size=(0.772, 0.825, 0.791), key='model.plant', tolerance=TOLERANCE, paint=[('body', 'Pot')]),
     G.Model('monstera', monstera, size=(1.328, 0.922, 1.344), key='model.garden.monstera', tolerance=TOLERANCE, paint=[('body', 'Pot')]),
     G.Model('fiddleFig', fiddle_fig, size=(0.528, 1.626, 0.527), key='model.garden.fiddleFig', tolerance=TOLERANCE, paint=[('body', 'Pot')]),
     G.Model('snakePlant', snake_plant, size=(0.341, 0.905, 0.32), key='model.garden.snakePlant', tolerance=TOLERANCE, paint=[('body', 'Pot')]),
     G.Model('cacti', cacti, size=(0.568, 0.521, 0.465), key='model.garden.cacti', tolerance=TOLERANCE, paint=[('body', 'Pot')]),
     G.Model('succulents', succulents, size=(0.62, 0.17, 0.62), key='model.garden.succulents', tolerance=TOLERANCE, paint=[('body', 'Pot')]),
     G.Model('bostonFern', boston_fern, size=(0.876, 0.979, 0.94), key='model.garden.bostonFern', tolerance=TOLERANCE, paint=[('body', 'Pot')]),
     G.Model('palm', palm, size=(1.074, 0.869, 0.982), key='model.garden.palm', tolerance=TOLERANCE, paint=[('body', 'Pot')]),
     G.Model('orchid', orchid, size=(0.431, 1.198, 0.381), key='model.garden.orchid', tolerance=TOLERANCE, paint=[('body', 'Pot')]),
     G.Model('bonsai', bonsai, size=(0.62, 0.974, 0.407), key='model.garden.bonsai', tolerance=TOLERANCE, paint=[('body', 'Pot')],
             motions={'uses': {'prune': 'trim'}})],
    comment='Blender-built potted plants (tools/art/blender/families/plants.py): the G1 houseplants. Replaces the '
            'primitive-part placeholders of furniture.py and nature.py by key; listed in the base manifest\'s packs. '
            'The placeholder is kept as a fallback if a file fails to load. Leaves and stems are on the `foliage` '
            'finish, so the nature shader sways and tints them as it does the placeholders\'; the pot is `body`, '
            'the part the player colours. The bonsai\'s stray sprig shrinks away while a resident prunes it.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

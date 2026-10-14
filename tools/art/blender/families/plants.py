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
from mathutils.noise import noise as _noise

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
    bed=('#6A4C38', 'matte'),           # a flower bed's soil mound: the part the player colours ("Bed")
    bloom2=('#F2C12E', 'satin'),
    bloom3=('#EF6F9C', 'satin'),
    bloom4=('#F08A24', 'satin'),
    bloom5=('#F4F1EA', 'satin'),
    centre=('#F6C21E', 'satin'),        # flower centres
    centre_dark=('#3A2A1E', 'matte'),
    blossom=('#F8901C', 'foliage'),     # flowers drawn leafy, as the placeholders' (marigolds, lupins, forget-me-nots)
    blossom2=('#FCBA1E', 'foliage'),
    blossom3=('#F4F0F2', 'foliage'),
    blossom4=('#6C8CF0', 'foliage'),
    stalk=('#5E8C3A', 'satin'),         # the meadow's picked flower's stem: one draw call with its bloom
)
M = G.Palette(**BASE)


def palette(**colours):
    """The family's palette with some colours changed (finishes stay the family's)."""
    return G.Palette(**{**BASE, **{k: (v, BASE[k][1]) for k, v in colours.items()}})


# ---- shapes ------------------------------------------------------------------------------------------

def noise_at(x, y, z):
    return _noise(Vector((x, y, z)))


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


# ---- flowers (G2) -------------------------------------------------------------------------------------

def _put(obj, at, yaw=0.0, tilt=0.0, lift=0.0):
    """Moves a part made at the origin (facing up) to `at`, its face leaning `tilt` degrees towards `yaw`."""
    obj.data.transform(Matrix.Translation(at) @ Matrix.Rotation(math.radians(-yaw), 4, 'Z')
                       @ Matrix.Rotation(math.radians(-tilt), 4, 'X') @ Matrix.Translation((0, 0, lift)))
    return obj


def star(name, at, r, n, material, yaw=0.0, tilt=0.0, inner=0.5, depth=0.008, cup=0.2):
    """A flower's face: a thin star of `n` petals, `r` across its radius, the petals lifted by `cup` (a fraction
    of their reach), facing up and leaning `tilt` degrees towards `yaw`."""
    top, bottom = [], []
    for k in range(2 * n):
        rr = r if k % 2 == 0 else r * inner
        a = math.pi * k / n
        z = cup * rr
        top.append(Vector((rr * math.cos(a), rr * math.sin(a), z + depth / 2)))
        bottom.append(Vector((rr * math.cos(a), rr * math.sin(a), z - depth / 2)))
    verts = top + bottom + [Vector((0, 0, depth / 2)), Vector((0, 0, -depth / 2))]
    m = 2 * n
    ct, cb = 2 * m, 2 * m + 1
    faces = []
    for k in range(m):
        j = (k + 1) % m
        faces += [(ct, k, j), (cb, m + j, m + k), (k, m + k, m + j, j)]
    return _put(_mesh(name, verts, faces, material, smooth=False), at, yaw, tilt)


def dome(name, at, r, h, material, yaw=0.0, tilt=0.0, lift=0.004):
    """A flower's centre: a low dome on its face."""
    G.sphere(name, 1.0, material=material, segments=6, rings=3, scale=(r, r, h))
    return _put(bpy.context.object, at, yaw, tilt, lift)


def cupped(name, profile, at, material, yaw=0.0, tilt=0.0, segments=6):
    """A bloom turned from `profile` [(r, z)] (a tulip, a poppy, a lily's trumpet), leaning `tilt` towards `yaw`."""
    return _put(G.revolve(name, profile, material=material, segments=segments), at, yaw, tilt)


def lump(name, r, at, material, scale=(1, 1, 1), seed=0.0, noise=0.25, segments=10, rings=6):
    """A soft, lumpy ball (a leafy clump, a pompom, a cluster of florets): a smooth sphere pushed about by noise,
    its bottom at `at`."""
    G.sphere(name, r, material=material, segments=segments, rings=rings, scale=scale)
    obj = bpy.context.object
    for v in obj.data.vertices:
        p = v.co
        n = noise_at(p.x / r * 1.3 + seed, p.y / r * 1.3 - seed, p.z / r * 1.3 + seed * 0.5)
        v.co = p * (1.0 + noise * n)
    low = min(v.co.z for v in obj.data.vertices)
    obj.data.transform(Matrix.Translation(Vector(at) - Vector((0, 0, low))))
    return obj


def spiral(n, radius, rng, jitter=0.25):
    """`n` points spread evenly over a disc (a sunflower's spiral), jittered: (x, y, degrees round)."""
    ga = math.pi * (3 - math.sqrt(5))
    off = rng.random() * math.tau
    out = []
    for i in range(n):
        rr = radius * math.sqrt((i + 0.5) / n)
        a = off + i * ga + rng.uniform(-jitter, jitter)
        out.append((rr * math.sin(a), rr * math.cos(a), math.degrees(a)))
    return out


def mound(P, r=0.42, h=0.07):
    """A bed's low soil mound (`bed`, the part the player colours): returns its top."""
    G.revolve('bed', [(0, 0), (r, 0), (r * 0.93, h * 0.4), (r * 0.65, h * 0.85), (0, h)], material=P.bed, segments=20)
    return h


def clumps(P, rng, n, radius, size, h, z, cols):
    """Low leafy clumps between the flowers."""
    for i, (x, y, _) in enumerate(spiral(n, radius, rng)):
        s = rng.uniform(0.8, 1.2)
        lump(f'clump{i}', size / 2 * s, (x, y, z - h * 0.6), cols[i % len(cols)], scale=(1, 1, h / size), seed=i * 1.3 + rng.random())


def stem(name, a, b, material, r=0.006):
    return tube(name, [a, b], r, r * 0.8, material, verts=3)


def blade(name, base, yaw, tilt, length, width, material, droop=0.3):
    """A narrow leaf (grass, a bulb's leaf), cheap: three rows of three."""
    return leaf(name, base, yaw, tilt, length, width, material, shape='strap', droop=droop, thick=0.006, fold=0.3,
                rows=3, ring=3)


def tulips():
    """Tulips: a soil mound of tulips in five colours on straight stems, a broad leaf to every other one."""
    P = palette()
    rng = random.Random(51)
    top = mound(P)
    cols = [P.bloom, P.bloom2, P.bloom3, P.bloom4, P.bloom, P.bloom5]
    profile = [(0, 0), (0.016, 0.004), (0.036, 0.03), (0.038, 0.06), (0.03, 0.082), (0, 0.068)]
    for i, (x, y, a) in enumerate(spiral(17, 0.34, rng)):
        h = rng.uniform(0.4, 0.55)
        tip = (x + rng.uniform(-0.03, 0.03), y + rng.uniform(-0.03, 0.03), h)
        stem(f'stem{i}', (x, y, top - 0.01), tip, P.stem)
        cupped(f'tulip{i}', profile, tip, cols[i % len(cols)], segments=6)
        if i % 2 == 0:
            leaf(f'leaf{i}', (x, y, top), a + 40, 62, 0.24, 0.07, P.leaf, shape='lance', droop=0.35, fold=0.3, rows=4, ring=4)
    return G.settle([G.join_all('tulips')])


def daffodils():
    """Daffodils: pale six-petalled flowers with orange trumpets, nodding outwards, and strap leaves."""
    P = palette(bloom='#FAE68A', centre='#F6AE1C', stem='#5E8C3A')
    rng = random.Random(52)
    top = mound(P)
    for i, (x, y, a) in enumerate(spiral(15, 0.34, rng)):
        h = rng.uniform(0.34, 0.46)
        tip = (x, y, h)
        yaw = a + rng.uniform(-40, 40)
        stem(f'stem{i}', (x, y, top - 0.01), tip, P.stem)
        star(f'petals{i}', tip, 0.05, 6, P.bloom, yaw, 70, inner=0.62, cup=0.1)
        cupped(f'trumpet{i}', [(0, 0), (0.016, 0.0), (0.02, 0.04), (0.026, 0.05), (0.021, 0.05), (0, 0.012)], tip, P.centre,
               yaw, 70, segments=8)
        blade(f'leaf{i}', (x, y, top), yaw + 150, 75, 0.3, 0.04, [P.leaf, P.leaf_light][i % 2])
    return G.settle([G.join_all('daffodils')])


def daisies():
    """Daisies: low leafy clumps and white daisies with yellow middles."""
    P = palette(bloom='#FFFFFF', centre='#F6C21E', stem='#5E8C3A')
    rng = random.Random(53)
    top = mound(P)
    clumps(P, rng, 7, 0.32, 0.24, 0.13, top + 0.04, [P.leaf, P.leaf_dark, P.leaf_light])
    for i, (x, y, a) in enumerate(spiral(24, 0.38, rng)):
        h = rng.uniform(0.25, 0.36)
        yaw, tilt = rng.uniform(0, 360), rng.uniform(0, 30)
        stem(f'stem{i}', (x, y, top + 0.02), (x, y, h), P.stem, r=0.005)
        star(f'daisy{i}', (x, y, h), 0.042, 9, P.bloom, yaw, tilt, inner=0.6, cup=0.08)
        dome(f'eye{i}', (x, y, h), 0.014, 0.008, P.centre, yaw, tilt)
    return G.settle([G.join_all('daisies')])


def sunflowers():
    """Sunflowers: four tall stalks with broad leaves, their big yellow heads turned to the front."""
    P = palette(bloom='#F5C518', centre_dark='#5A3A1E', stem='#5E8C3A')
    rng = random.Random(54)
    top = mound(P)
    for i, ((x, y), h) in enumerate(zip([(0.18, 0.12), (-0.17, 0.06), (0.0, -0.18), (0.04, 0.3)], [1.85, 1.6, 1.4, 1.95])):
        tip = Vector((x * 1.2, y * 1.2 - 0.04, h))
        tube(f'stalk{i}', [(x, y, top - 0.01), tip], 0.02, 0.014, P.stem, verts=6)
        for k in range(3):
            t = 0.3 + k * 0.2
            base = Vector((x, y, 0)).lerp(tip, t)
            leaf(f'leaf{i}_{k}', base, rng.uniform(0, 360) + k * 120, 30, 0.2, 0.16, [P.leaf, P.leaf_dark, P.leaf_light][k],
                 shape='heart', droop=0.35, fold=0.15, rows=5, ring=4)
        yaw = 180 + rng.uniform(-35, 35)
        star(f'petals{i}', tip, 0.17, 16, P.bloom, yaw, 62, inner=0.62, cup=0.06, depth=0.012)
        dome(f'seeds{i}', tip, 0.085, 0.025, P.centre_dark, yaw, 62)
    return G.settle([G.join_all('sunflowers')])


def poppies():
    """Poppies: grey-green leaves and red cups on thin wavering stems, some with dark middles."""
    P = palette(bloom='#EC3A2E', stem='#6E9464', leaf='#5E8456', leaf_light='#6E9464', centre_dark='#1E1A1A')
    rng = random.Random(55)
    top = mound(P)
    clumps(P, rng, 6, 0.3, 0.22, 0.12, top + 0.03, [P.leaf, P.leaf_light])
    profile = [(0, 0), (0.01, 0.004), (0.03, 0.026), (0.042, 0.05), (0.036, 0.052), (0, 0.022)]
    for i, (x, y, a) in enumerate(spiral(16, 0.38, rng)):
        h = rng.uniform(0.4, 0.61)
        tip = (x + rng.uniform(-0.04, 0.04), y + rng.uniform(-0.04, 0.04), h)
        yaw, tilt = rng.uniform(0, 360), rng.uniform(0, 30)
        tube(f'stem{i}', [(x, y, top), ((x + tip[0]) / 2 + 0.01, (y + tip[1]) / 2, h * 0.55), tip], 0.004, 0.004, P.stem, verts=3)
        cupped(f'poppy{i}', profile, tip, P.bloom, yaw, tilt, segments=6)
        if i % 3 == 0:
            dome(f'eye{i}', tip, 0.012, 0.01, P.centre_dark, yaw, tilt, lift=0.022)
    return G.settle([G.join_all('poppies')])


def marigolds():
    """Marigolds: dark leafy clumps covered in ruffled orange and yellow pompoms."""
    P = palette(leaf='#2F5A28', leaf_dark='#3A6A2E', leaf_light='#467634', blossom='#F8901C', blossom2='#FCBA1E',
                blossom3='#F0761A')
    rng = random.Random(56)
    top = mound(P)
    clumps(P, rng, 7, 0.33, 0.27, 0.17, top + 0.07, [P.leaf, P.leaf_dark, P.leaf_light])
    for i, (x, y, a) in enumerate(spiral(18, 0.38, rng)):
        lump(f'pom{i}', 0.05, (x, y, rng.uniform(0.19, 0.27)), [P.blossom, P.blossom2, P.blossom3][i % 3], scale=(1, 1, 0.75), seed=i * 0.7)
    return G.settle([G.join_all('marigolds')])


def pansies():
    """Pansies: a low carpet of leaves and two-tone faces turned outwards."""
    P = palette(bloom='#5B3A8E', bloom2='#F2C94C', bloom3='#3F5FB0', bloom4='#8E2A4A', bloom5='#F4F0E8',
                centre='#F2C94C', centre_dark='#3A2A4E')
    rng = random.Random(57)
    top = mound(P)
    clumps(P, rng, 7, 0.34, 0.22, 0.09, top + 0.02, [P.leaf, P.leaf_dark, P.leaf_light])
    faces = [(P.bloom, P.centre), (P.bloom2, P.centre_dark), (P.bloom3, P.centre), (P.bloom4, P.centre), (P.bloom5, P.centre_dark)]
    for i, (x, y, a) in enumerate(spiral(15, 0.36, rng)):
        h = rng.uniform(0.17, 0.23)
        petal, eye = faces[i % len(faces)]
        yaw, tilt = a + rng.uniform(-30, 30), rng.uniform(35, 60)
        star(f'pansy{i}', (x, y, h), 0.045, 5, petal, yaw, tilt, inner=0.75, cup=0.05)
        dome(f'eye{i}', (x, y, h), 0.017, 0.006, eye, yaw, tilt)
    return G.settle([G.join_all('pansies')])


def lilies():
    """Lilies: six tall stems with narrow leaves, each with two flared trumpets."""
    P = palette(bloom='#F07A2A', bloom2='#F5EFE4', bloom3='#E98AA8')
    rng = random.Random(58)
    top = mound(P)
    cols = [P.bloom, P.bloom2, P.bloom3]
    trumpet = [(0, 0), (0.008, 0.0), (0.016, 0.05), (0.04, 0.1), (0.06, 0.125), (0.054, 0.128), (0, 0.06)]
    for i, (x, y, a) in enumerate(spiral(6, 0.27, rng)):
        h = rng.uniform(0.71, 0.87)
        tip = Vector((x * 1.15, y * 1.15, h))
        tube(f'stem{i}', [(x, y, top - 0.01), tip], 0.007, 0.006, P.stem, verts=4)
        for k in range(3):
            t = 0.25 + k * 0.2
            blade(f'leaf{i}_{k}', Vector((x, y, 0)).lerp(tip, t), rng.uniform(0, 360), 35, 0.2, 0.035, P.leaf, droop=0.25)
        for k in range(2):
            yaw = a + (k * 2 - 1) * 50 + rng.uniform(-15, 15)
            cupped(f'lily{i}_{k}', trumpet, tip - Vector((0, 0, 0.03 * k)), cols[i % 3], yaw, 55, segments=6)
    return G.settle([G.join_all('lilies')])


def lupins():
    """Lupins: a leafy base and tall tapering spikes of florets in purples, pinks, white and blue."""
    P = palette(blossom='#8E6CE0', blossom2='#F08CC4', blossom3='#F4F0F2', blossom4='#6C8CF0')
    rng = random.Random(59)
    top = mound(P)
    clumps(P, rng, 6, 0.3, 0.26, 0.16, top + 0.06, [P.leaf, P.leaf_dark, P.leaf_light])
    cols = [P.blossom, P.blossom2, P.blossom3, P.blossom4]
    for i, (x, y, a) in enumerate(spiral(9, 0.3, rng)):
        h = rng.uniform(0.66, 0.99)
        tip = (x + rng.uniform(-0.03, 0.03), y + rng.uniform(-0.03, 0.03), h)
        stem(f'stem{i}', (x, y, top + 0.05), (tip[0], tip[1], h - 0.3), P.stem, r=0.008)
        G.cone(f'spike{i}', 0.05, 0.008, 0.36, at=(tip[0], tip[1], h - 0.18), material=cols[i % 4], verts=8)
    return G.settle([G.join_all('lupins')])


def forget_me_nots():
    """Forget-me-nots: leafy clumps under drifts of tiny sky-blue flowers."""
    P = palette(blossom='#8CB4F8', blossom2='#A4C4FC', blossom3='#7AA0F0')
    rng = random.Random(60)
    top = mound(P)
    clumps(P, rng, 7, 0.33, 0.28, 0.15, top + 0.05, [P.leaf, P.leaf_dark, P.leaf_light])
    for i, (x, y, a) in enumerate(spiral(16, 0.36, rng)):
        lump(f'drift{i}', 0.06, (x, y, rng.uniform(0.17, 0.23)), [P.blossom, P.blossom2, P.blossom3][i % 3], scale=(1, 1, 0.4), seed=i * 0.9)
    return G.settle([G.join_all('forgetMeNots')])


def cosmos():
    """Cosmos: feathery leaves and tall thin stems of pink, white and magenta daisies."""
    P = palette(bloom='#E77FB0', bloom2='#F6EEF2', bloom3='#C2367A', bloom4='#F2A6CA', centre='#F2C12E', stem='#6E9E44')
    rng = random.Random(61)
    top = mound(P)
    for i, (x, y, a) in enumerate(spiral(9, 0.3, rng)):
        leaf(f'frond{i}', (x, y, top), rng.uniform(0, 360), rng.uniform(58, 78), rng.uniform(0.28, 0.38), 0.07,
             [P.leaf_light, P.leaf][i % 2], shape='lance', droop=0.3, fold=0.1, rows=9, ring=3, notch=(2, 0.25))
    cols = [P.bloom, P.bloom2, P.bloom3, P.bloom4]
    for i, (x, y, a) in enumerate(spiral(15, 0.36, rng)):
        h = rng.uniform(0.55, 0.87)
        tip = (x * 1.1, y * 1.1, h)
        yaw, tilt = rng.uniform(0, 360), rng.uniform(10, 40)
        stem(f'stem{i}', (x, y, top - 0.01), tip, P.stem, r=0.004)
        star(f'cosmos{i}', tip, 0.05, 8, cols[i % 4], yaw, tilt, inner=0.7, cup=0.1)
        dome(f'eye{i}', tip, 0.013, 0.008, P.centre, yaw, tilt)
    return G.settle([G.join_all('cosmos')])


MEADOW_FLOWERS = ['bloom5', 'bloom2', 'bloom3', 'bloom', 'bloom4']


def meadow():
    """Wildflower meadow: tussocks and long grass with flowers of every colour; one flower, at the front, is picked
    (it shrinks away while a resident picks it, and is back after)."""
    P = palette(leaf='#7FA04A', leaf_dark='#6E9240', leaf_light='#A2B45E', stem='#8EAE52', bloom='#EC3A2E',
                bloom2='#F6C21E', bloom3='#A880E0', bloom4='#8CB4F8', bloom5='#FFFFFF', centre='#E8B436')
    rng = random.Random(62)
    grass = [P.leaf, P.stem, P.leaf_dark, P.leaf_light]
    for i, (x, y, _) in enumerate(spiral(5, 0.3, rng)):
        lump(f'tussock{i}', 0.15, (x, y, 0), grass[i % 4], scale=(1, 1, 0.55), seed=i * 1.1)
    for i, (x, y, a) in enumerate(spiral(26, 0.42, rng)):
        blade(f'grass{i}', (x * 0.76, y * 0.76, 0), a + rng.uniform(-40, 40), rng.uniform(68, 85), rng.uniform(0.25, 0.4), 0.04,
              grass[i % 4], droop=0.3)
    for i, (x, y, a) in enumerate(spiral(16, 0.4, rng)):
        h = rng.uniform(0.26, 0.48)
        yaw, tilt = rng.uniform(0, 360), rng.uniform(0, 30)
        stem(f'stem{i}', (x, y, 0), (x, y, h), P.stem, r=0.004)
        star(f'flower{i}', (x, y, h), 0.042, 6 + i % 3, getattr(P, MEADOW_FLOWERS[i % 5]), yaw, tilt, inner=0.6, cup=0.1)
        dome(f'eye{i}', (x, y, h), 0.011, 0.007, P.centre, yaw, tilt)
    still = G.join_all('meadow')
    with G.new_parts() as parts:                                      # the one picked: at the front, standing tall
        x, y, h = 0.06, -0.22, 0.48
        stem('picked_stem', (x, y, 0), (x, y, h), P.stalk, r=0.006)
        star('picked_flower', (x, y, h), 0.06, 10, P.bloom5, 180, 30, inner=0.6, cup=0.1)
        dome('picked_eye', (x, y, h), 0.02, 0.012, P.centre, 180, 30)
    flower = G.join(parts, 'picked', origin=(x, y, 0))
    G.settle([still, flower])
    G.squash(flower, 'pick', 0.001, seconds=0.8)
    return [still, flower]


def geraniums():
    """Potted geraniums: a terracotta pot of round leaves and clusters of red flowers."""
    P = palette(body='#C0683F', bloom='#D6283A', bloom2='#E23A48', bloom3='#C41E32')
    rng = random.Random(63)
    top = pot(P, 0.2, 0.34, r_bottom=0.15, rim=0.012)
    for i, (x, y, _) in enumerate(spiral(5, 0.13, rng)):
        lump(f'leaves{i}', 0.13, (x, y, top - 0.02), [P.leaf, P.leaf_dark, P.leaf_light][i % 3], scale=(1, 1, 0.75), seed=i * 1.7)
    for i, (x, y, a) in enumerate(spiral(6, 0.15, rng)):
        h = top + rng.uniform(0.2, 0.27)
        stem(f'stem{i}', (x * 0.5, y * 0.5, top), (x, y, h), P.stem, r=0.005)
        lump(f'flowers{i}', 0.05, (x, y, h - 0.02), [P.bloom, P.bloom2, P.bloom3][i % 3], scale=(1, 1, 0.85), seed=i * 0.6)
    return G.settle([G.join_all('geraniums')])


def flower_box():
    """Flower box: a white wooden box on legs, full of leaves and bright flowers, a few trailing over the front."""
    P = palette(body='#ECE9E1', bloom='#E85A9A', bloom2='#F2C14E', bloom3='#9A5AC8', bloom5='#F4F0F2')
    rng = random.Random(64)
    G.cube('box', (0.9, 0.42, 0.26), at=(0, 0.03, 0.45), material=P.body, bevel=0.01)
    for x in (-0.4, 0.4):
        for y in (-0.14, 0.2):
            G.cube(f'leg{x}{y}', (0.055, 0.055, 0.34), at=(x, y, 0.17), material=P.body, bevel=0.006)
    G.cube('soil', (0.84, 0.36, 0.02), at=(0, 0.03, 0.575), material=P.soil)
    for i in range(6):
        x = -0.36 + i * 0.144
        lump(f'leaves{i}', 0.11, (x, 0.03 + rng.uniform(-0.05, 0.05), 0.56), [P.leaf, P.leaf_dark, P.leaf_light][i % 3], scale=(1, 1.1, 0.7), seed=i * 1.3)
    cols = [P.bloom, P.bloom5, P.bloom3, P.bloom, P.bloom2]
    for i in range(14):
        x, y = rng.uniform(-0.4, 0.4), rng.uniform(-0.12, 0.18)
        lump(f'flower{i}', 0.036, (x, y, rng.uniform(0.64, 0.69)), cols[i % 5], scale=(1, 1, 0.7), seed=i * 0.8)
    for i, x in enumerate((-0.3, -0.12, 0.05, 0.2, 0.32)):         # trails hanging over the front
        for k in range(2):
            leaf(f'trail{i}_{k}', (x + k * 0.04, -0.17, 0.6), 180 + (k * 2 - 1) * 15, -35, rng.uniform(0.2, 0.26), 0.06,
                 [P.leaf_light, P.leaf][k], shape='lance', droop=0.6, fold=0.2, rows=6, ring=3, notch=(2, 0.4))
    return G.settle([G.join_all('flowerBox')])


def flowerbed():
    """Raised flower bed: a wooden box of soil, leafy clumps and flowers in six colours."""
    P = palette(wood='#8A6446', bloom='#E8873A', bloom2='#8E6BBF', bloom3='#F0EDE6', bloom4='#D9483B', bloom5='#F2C14E')
    rng = random.Random(65)
    for y in (-0.44, 0.44):
        G.cube(f'side{y}', (0.94, 0.06, 0.24), at=(0, y, 0.12), material=P.wood, bevel=0.008)
    for x in (-0.44, 0.44):
        G.cube(f'end{x}', (0.06, 0.82, 0.24), at=(x, 0, 0.12), material=P.wood, bevel=0.008)
    G.cube('soil', (0.84, 0.84, 0.03), at=(0, 0, 0.215), material=P.soil)
    clumps(P, rng, 8, 0.32, 0.22, 0.16, 0.3, [P.leaf, P.leaf_dark, P.leaf_light])
    cols = [P.bloom, P.bloom2, P.bloom3, P.bloom4, P.bloom5, P.bloom3]
    for i, (x, y, a) in enumerate(spiral(18, 0.34, rng)):
        lump(f'flower{i}', 0.04, (x, y, rng.uniform(0.35, 0.42)), cols[i % 6], scale=(1, 1, 0.7), seed=i * 0.8)
    return G.settle([G.join_all('flowerbed')])


# (width, height, depth) as measured; each within 10 % of its placeholder's bounds (check_glb warns beyond).
TOLERANCE = 0.03
BED = [('bed', 'Bed')]

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
             motions={'uses': {'prune': 'trim'}}),
     # G2: flowers
     G.Model('flowerbed', flowerbed, size=(0.94, 0.475, 0.94), key='model.flowerbed', tolerance=TOLERANCE, paint=[('wood', 'Box')]),
     G.Model('tulips', tulips, size=(0.84, 0.622, 0.843), key='model.garden.tulips', tolerance=TOLERANCE, paint=BED),
     G.Model('daffodils', daffodils, size=(0.84, 0.491, 0.84), key='model.garden.daffodils', tolerance=TOLERANCE, paint=BED + [('bloom', 'Petals')]),
     G.Model('daisies', daisies, size=(0.84, 0.376, 0.84), key='model.garden.daisies', tolerance=TOLERANCE, paint=BED + [('bloom', 'Petals')]),
     G.Model('sunflowers', sunflowers, size=(0.84, 2.108, 0.907), key='model.garden.sunflowers', tolerance=TOLERANCE, paint=BED + [('bloom', 'Petals')]),
     G.Model('poppies', poppies, size=(0.84, 0.657, 0.84), key='model.garden.poppies', tolerance=TOLERANCE, paint=BED + [('bloom', 'Petals')]),
     G.Model('marigolds', marigolds, size=(0.885, 0.34, 0.84), key='model.garden.marigolds', tolerance=TOLERANCE, paint=BED),
     G.Model('pansies', pansies, size=(0.84, 0.264, 0.84), key='model.garden.pansies', tolerance=TOLERANCE, paint=BED),
     G.Model('lilies', lilies, size=(0.84, 0.939, 0.869), key='model.garden.lilies', tolerance=TOLERANCE, paint=BED),
     G.Model('lupins', lupins, size=(0.84, 0.987, 0.84), key='model.garden.lupins', tolerance=TOLERANCE, paint=BED),
     G.Model('forgetMeNots', forget_me_nots, size=(0.866, 0.28, 0.84), key='model.garden.forgetMeNots', tolerance=TOLERANCE, paint=BED),
     G.Model('cosmos', cosmos, size=(0.885, 0.879, 0.84), key='model.garden.cosmos', tolerance=TOLERANCE, paint=BED),
     G.Model('meadow', meadow, size=(0.811, 0.517, 0.919), key='model.garden.meadow', tolerance=TOLERANCE,
             motions={'uses': {'pickFlower': 'pick'}}),
     G.Model('geraniums', geraniums, size=(0.424, 0.65, 0.503), key='model.garden.geraniums', tolerance=TOLERANCE, paint=[('body', 'Pot'), ('bloom', 'Flowers')]),
     G.Model('flowerBox', flower_box, size=(0.945, 0.732, 0.505), key='model.garden.flowerBox', tolerance=TOLERANCE, paint=[('body', 'Box')])],
    comment='Blender-built potted plants (tools/art/blender/families/plants.py): the G1 houseplants. Replaces the '
            'primitive-part placeholders of furniture.py and nature.py by key; listed in the base manifest\'s packs. '
            'The placeholder is kept as a fallback if a file fails to load. Leaves and stems are on the `foliage` '
            'finish, so the nature shader sways and tints them as it does the placeholders\'; the pot is `body`, '
            'the part the player colours. The bonsai\'s stray sprig shrinks away while a resident prunes it.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

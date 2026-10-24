# The ground cover, the woods' rocks and the selection marker: the lawn's grass tuft (`model.grassTuft`) and
# wildflowers (`model.flowers`), drawn by the thousand as thin instances on every unpaved tile and round the town
# (render/shared/nature.ts `lawns`); the rock scattered in the woods (`model.rock`, nature.ts `forests`, always
# its far version there, without a shadow); and the sparkle star over the selected resident (`model.marker`,
# render/pc/selectionMarker.ts). Notes: ground.md.
#
# The lawn's two stay cheap (the tuft 30 triangles, of which the camera draws half, where the old part's 36 were
# mostly degenerate cone tips and hidden caps; the flowers 266 in one draw call where the old part took 192 in
# two): every part on `foliage`, which the nature shader sways and tints per instance, the flowers' heads with
# their stems. Each has a far version (`far_of`). The rock is two lumps cut flat a little under the ground,
# weathered, with pebbles at its foot; its far version (what the woods draw) 56 triangles. The marker is one closed pillow-shaped
# star, which the game draws in its own unlit material with an outline pushed out along its normals (inverted
# hull): built smooth, its normals worked out from its shape, its origin at its middle, as the old part's.
#
# Run:  cargo art ground [--only marker] [--preview]
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
from gamekit.far import far_of, low  # noqa: E402

M = G.Palette(
    # the grass tuft: the old part's three greens and the fourth
    blade=('#7C9E44', 'foliage'),
    blade_light=('#8AAE4C', 'foliage'),
    blade_pale=('#98BA56', 'foliage'),
    blade_mid=('#84A648', 'foliage'),
    # the wildflowers: on `foliage` too, so the heads sway with their stems (one draw call)
    leaf=('#5E8436', 'foliage'),
    stem=('#4F7A30', 'foliage'),
    # (brighter than the old part's satin heads: the foliage finish's leaf texture darkens what it is on)
    petal_white=('#FFFFFF', 'foliage'),
    petal_yellow=('#FFD84A', 'foliage'),
    petal_violet=('#B48CF0', 'foliage'),
    eye=('#FFC72E', 'foliage'),
    eye_orange=('#F08A1E', 'foliage'),
    eye_pale=('#F2ECF6', 'foliage'),     # a cranesbill's pale centre
    # the rock
    stone=('#8E8A82', 'matte'),
    stone_light=('#9A958C', 'matte'),
    lichen=('#9FA28E', 'matte'),     # pale grey-green lichen on the tops
    soil=('#6F675C', 'matte'),       # the damp, earthy foot
    # the marker: the game draws it in its own unlit material, tinted by the resident's mood
    star=('#FFFFFF', 'satin'),
)


def _mesh(name, verts, faces, material, smooth=True, sky=0.0, recalc=True):
    """An object from vertices (Blender metres) and faces, normals outward. `sky` bends every face's normal that
    far towards straight up (custom normals: the shape stays), as a lawn's grass is shaded, so near-upright
    blades take the light the ground round them does rather than drawing dark from above. `recalc=False` keeps the
    faces wound as given (a ribbon's two sides, which have no outside to turn to)."""
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata([tuple(v) for v in verts], [], faces)
    mesh.validate()
    if recalc:
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
    if sky:
        up = Vector((0, 0, 1))
        loops = [None] * len(mesh.loops)
        for p in mesh.polygons:
            # (Never past the face's side: the game winds each triangle by its normals, models.ts `pushTri`.)
            k = sky if p.normal.z >= 0 else min(sky, 0.7 / -p.normal.z)
            n = (p.normal + up * k).normalized()
            for i in p.loop_indices:
                loops[i] = tuple(n)
        mesh.normals_split_custom_set(loops)
    return obj


# How far a blade's or leaf's normals bend to the sky (`_mesh`).
SKY = 1.5


def blade(name, base, yaw, lean, bend, length, width, material, mid=0.5, joint=True, fold=0.0):
    """
    A grass blade (or a flower's leaf) from `base` (game x, depth) on the ground, `width` across there, leaning
    `lean` degrees from upright towards `yaw` (degrees round from game +x) and bending `bend` more by its tip,
    `length` long, with one joint (`joint`; without, a straight spike: a short blade). A flat ribbon drawn from both
    sides: two faces back to back (each side its own vertices and normals), of which the camera only ever draws the
    one turned to it, so a tuft costs the GPU what the old part's thin cones did (a closed blade of three sides drew
    three times the triangles and measured 0.9 ms more over the village's lawns on WebGL). 6 triangles; 2 in a far
    version (no joint). `fold`: folded along its midrib from the joint up, the midrib that share of its width out of
    the ribbon (a leaf's channel): 12 triangles.
    """
    h = Vector((-math.cos(math.radians(yaw)), -math.sin(math.radians(yaw)), 0))   # game (cos, sin) on the ground
    up = Vector((0, 0, 1))
    across = up.cross(h).normalized()

    def heading(a):
        return (h * math.sin(math.radians(a)) + up * math.cos(math.radians(a))).normalized()

    p0 = Vector((-base[0], -base[1], 0.0))
    d0, d1 = heading(lean), heading(lean + bend)
    p1 = p0 + d0 * (length * mid)
    tip = p1 + d1 * (length * (1 - mid))
    if low(True, False) and fold:
        out = across.cross(d0).normalized() * (fold * width)
        side = [p0 + across * (width / 2), p0, p0 - across * (width / 2),
                p1 + across * (width * 0.36), p1 + out, p1 - across * (width * 0.36), tip]
        front = [(0, 1, 4, 3), (1, 2, 5, 4), (3, 4, 6), (4, 5, 6)]
    elif low(True, False) and joint:
        side = [p0 + across * (width / 2), p0 - across * (width / 2), p1 - across * (width * 0.31), p1 + across * (width * 0.31), tip]
        front = [(0, 1, 2, 3), (3, 2, 4)]
    else:
        # (A far version goes straight to the same tip.)
        side = [p0 + across * (width / 2), p0 - across * (width / 2), tip]
        front = [(0, 1, 2)]
    n = len(side)
    faces = front + [tuple(n + i for i in reversed(f)) for f in front]
    # Flat, its normals bent to the sky: upright, its faces drew dark against the lawn from above.
    return _mesh(name, side + side, faces, material, smooth=False, sky=SKY, recalc=False)


# ---- the grass tuft --------------------------------------------------------------------------------------------

def grassTuft():
    """Four bent blades from a few millimetres round the middle, leaning out, in four greens (the old part's
    three cones and the fourth colour), and three short, narrow ones between them, the new growth: 30 triangles,
    15 of them facing the camera (the old four: 24)."""
    rng = random.Random(41)
    colours = [M.blade, M.blade_light, M.blade_pale, M.blade_mid]
    for i in range(4):
        yaw = i * 90 + rng.uniform(-25, 25)
        r = rng.uniform(0.003, 0.008)
        base = (math.cos(math.radians(yaw)) * r, math.sin(math.radians(yaw)) * r)
        blade(f'blade{i}', base, yaw, rng.uniform(4, 8), rng.uniform(10, 16), rng.uniform(0.27, 0.31),
              rng.uniform(0.032, 0.04), colours[i])
    # (drawn after the four, so they stand where they did)
    for i in range(3):
        yaw = 45 + i * 120 + rng.uniform(-20, 20)
        r = rng.uniform(0.002, 0.005)
        base = (math.cos(math.radians(yaw)) * r, math.sin(math.radians(yaw)) * r)
        blade(f'short{i}', base, yaw, rng.uniform(13, 18), 0, rng.uniform(0.13, 0.16), rng.uniform(0.02, 0.026),
              colours[(i + 1) % 4], joint=False)
    return G.settle([G.join_all('grassTuft')])


# ---- the wildflowers -------------------------------------------------------------------------------------------

def stem(name, base, top, radius, material, sway=0.012):
    """A stem from `base` to `top` (Blender points): a three-sided tube in two lengths, kinked a little at its
    middle, open at both ends (in the lawn, and under its flower): 12 triangles; 6 in a far version."""
    axis = (top - base).normalized()
    side = axis.cross(Vector((0, 0, 1)))
    side = side.normalized() if side.length > 1e-6 else Vector((1, 0, 0))
    other = axis.cross(side).normalized()
    centres = [base, (base + top) / 2 + side * sway, top] if low(True, False) else [base, top]
    verts = []
    for c in centres:
        for j in range(3):
            a = math.tau * j / 3
            verts.append(c + (side * math.cos(a) + other * math.sin(a)) * radius)
    faces = []
    for i in range(len(centres) - 1):
        for j in range(3):
            k = (j + 1) % 3
            faces.append((i * 3 + j, i * 3 + k, (i + 1) * 3 + k, (i + 1) * 3 + j))
    return _mesh(name, verts, faces, material)


def flower(name, at, radius, tilt, yaw, petals, eye, n=5, notch=0.8, cup=0.0, disc=0.32, dome=0.011):
    """
    A flower head of `n` petals at `at` (a Blender point, the top of its stem) turned up `tilt` degrees towards
    `yaw`: petals round a raised eye on top (its own colour, `disc` of the radius across, domed `dome` high), a
    shallow cone underneath, one closed shell of 6n triangles. `notch`: how far out the notches between the petals
    are (0.8: broad petals; 0.4: a daisy's separate rays); `cup`: how far the petals' tips rise (a buttercup's bowl).
    A far version keeps the petal tips (at most eight) and leaves out the eye: 2 a tip.
    """
    full = low(True, False)
    tips = n if full else min(n, 8)
    rim = []
    for i in range(tips * (2 if full else 1)):
        a = math.tau * i / (tips * 2 if full else tips)
        tip = i % 2 == 0 or not full
        r = radius if tip else radius * notch
        rim.append(Vector((r * math.cos(a), r * math.sin(a), (0.004 + cup) if tip else cup * 0.6)))
    turn = Matrix.Rotation(math.radians(yaw), 4, 'Z') @ Matrix.Rotation(math.radians(tilt), 4, 'Y')
    place = Matrix.Translation(at) @ turn
    verts, faces, colours = [], [], []
    if full:
        inner = [Vector((radius * disc * math.cos(math.tau * i / n), radius * disc * math.sin(math.tau * i / n), 0.005 + cup * 0.3))
                 for i in range(n)]
        apex = Vector((0, 0, dome + cup * 0.3))
        bottom = Vector((0, 0, -0.008))
        verts = rim + inner + [apex, bottom]
        R, I, A, B = 0, len(rim), len(rim) + n, len(rim) + n + 1
        for i in range(n):
            j = (i + 1) % n
            # the petals: each eye edge to its two rim segments (tip, notch, next tip)
            faces += [(I + i, R + 2 * i, R + 2 * i + 1), (I + i, R + 2 * i + 1, I + j), (I + j, R + 2 * i + 1, R + (2 * i + 2) % (2 * n))]
            colours += [0, 0, 0]
            faces.append((A, I + i, I + j))
            colours.append(1)
        for i in range(2 * n):
            faces.append((B, R + (i + 1) % (2 * n), R + i))
            colours.append(0)
    else:
        top, bottom = Vector((0, 0, 0.006 + cup * 0.5)), Vector((0, 0, -0.008))
        verts = rim + [top, bottom]
        T, B = tips, tips + 1
        for i in range(tips):
            faces += [(T, i, (i + 1) % tips), (B, (i + 1) % tips, i)]
            colours += [0, 0]
    obj = _mesh(name, [place @ v for v in verts], faces, petals)
    if full:
        obj.data.materials.append(eye)
        for p, c in zip(obj.data.polygons, colours):
            p.material_index = c
    return obj


def bud(name, at, r, h, tilt, yaw, sepals, petals):
    """A closed flower bud on its stem's top `at` (Blender): green sepals cupping it from below, the white of its
    petals showing at the top, five sides round, `h` long, leaning `tilt` degrees towards `yaw`: 15 triangles."""
    turn = Matrix.Rotation(math.radians(yaw), 4, 'Z') @ Matrix.Rotation(math.radians(tilt), 4, 'Y')
    place = Matrix.Translation(at) @ turn
    ring = [Vector((r * math.cos(math.tau * i / 5), r * math.sin(math.tau * i / 5), h * 0.45)) for i in range(5)]
    neck = [Vector((r * 0.55 * math.cos(math.tau * (i + 0.5) / 5), r * 0.55 * math.sin(math.tau * (i + 0.5) / 5), h * 0.8))
            for i in range(5)]
    verts = ring + neck + [Vector((0, 0, h)), Vector((0, 0, -0.002))]
    T, B = 10, 11
    faces, colours = [], []
    for i in range(5):
        j = (i + 1) % 5
        faces += [(B, j, i), (i, j, 5 + i)]
        colours += [0, 0]
        faces += [(j, 5 + j, 5 + i), (5 + i, 5 + j, T)]
        colours += [0, 1]
    obj = _mesh(name, [place @ v for v in verts], faces, sepals)
    obj.data.materials.append(petals)
    for p, c in zip(obj.data.polygons, colours):
        p.material_index = c
    return obj


# Where the old part's three flowers stand: (game x, depth), height, petals, eye, and the head's shape: an ox-eye
# daisy's thirteen white rays round a broad yellow disc, a buttercup's five glossy yellow petals cupped up, a
# cranesbill's five broad violet ones, flat, round a pale eye.
FLOWERS = [((0.03, 0.02), 0.26, 'petal_white', 'eye', dict(n=13, notch=0.42, disc=0.36, dome=0.009)),
           ((-0.04, 0.03), 0.22, 'petal_yellow', 'eye_orange', dict(n=5, notch=0.82, cup=0.008, disc=0.28)),
           ((0.0, -0.05), 0.19, 'petal_violet', 'eye_pale', dict(n=5, notch=0.86, disc=0.26, dome=0.007))]


def flowers():
    """Wildflowers: five low leaves, each folded along its midrib, round three stems with an ox-eye daisy, a
    buttercup and a cranesbill where the old part's three heads stand, and a daisy's bud on a fourth, every part on
    `foliage`, in one draw call."""
    rng = random.Random(42)
    for i in range(5):
        yaw = i * 72 + rng.uniform(-20, 20)
        blade(f'leaf{i}', (math.cos(math.radians(yaw)) * 0.006, math.sin(math.radians(yaw)) * 0.006), yaw,
              rng.uniform(20, 28), rng.uniform(16, 24), rng.uniform(0.12, 0.15), rng.uniform(0.034, 0.042), M.leaf,
              fold=0.22)
    for i, ((x, z), h, petals, eye, shape) in enumerate(FLOWERS):
        top = Vector((-x, -z, h))
        stem(f'stem{i}', Vector((-x * 0.3, -z * 0.3, 0)), top, 0.0045, M.stem)
        tilt, yaw = rng.uniform(8, 22), math.degrees(math.atan2(-z, -x)) + rng.uniform(-30, 30)
        flower(f'flower{i}', top, 0.037, tilt, yaw, getattr(M, petals), getattr(M, eye), **shape)
    # the bud, nodding on its own stem (left out far off; its numbers drawn either way)
    tilt, yaw = rng.uniform(25, 35), rng.uniform(-30, 30)
    if low(True, False):
        top = Vector((-0.045, 0.035, 0.15))
        stem('stem_bud', Vector((-0.012, 0.01, 0)), top, 0.0035, M.stem, sway=0.008)
        bud('bud', top, 0.008, 0.022, tilt, 180 + yaw, M.leaf, M.petal_white)
    return G.settle([G.join_all('flowers')])


# ---- the rock --------------------------------------------------------------------------------------------------

ROCK_CENTRE = (0.085, 0.01)   # game (x, depth): the old part's middle
GROUND = -0.115   # the rock's lumps reach this far under the ground: it is cut flat there


def _cut(obj, co, no, flat=True):
    """Cuts `obj` (in place, Blender) by the plane through `co` facing `no`, dropping what lies in front of it
    and closing the cut with a face of its own: a fracture, or the ground. The cut face is drawn flat."""
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bm.transform(obj.matrix_world)
    cut = bmesh.ops.bisect_plane(bm, geom=bm.verts[:] + bm.edges[:] + bm.faces[:], plane_co=co, plane_no=no,
                                 clear_outer=True)
    edges = [e for e in cut['geom_cut'] if isinstance(e, bmesh.types.BMEdge)]
    made = bmesh.ops.holes_fill(bm, edges=edges)['faces'] if edges else []
    for f in made:
        f.smooth = not flat
    bmesh.ops.triangulate(bm, faces=bm.faces[:])
    bm.transform(obj.matrix_world.inverted())
    bm.to_mesh(obj.data)
    bm.free()
    return obj


def _lump(obj, amount, scale, seed):
    """Roughens a boulder's surface with finer noise than `G.boulder`'s (its own metres, before its scale)."""
    for v in obj.data.vertices:
        p = v.co * scale + Vector((seed, -seed, seed * 0.5))
        v.co *= 1.0 + amount * _noise(p)


def _stone(name, radius, at, material, scale, seed, top, subdivisions, fractures, grain=0.0):
    """A boulder `top` high (Blender z), roughened (`grain`: and pitted finer still, metres), with `fractures`
    (direction, depth: how far in from its outside, metres) cut flat across it, then cut off under the ground."""
    o = G.boulder(name, radius, at=at, material=material, scale=(1, 1, 1), seed=seed, noise=0.26, subdivisions=subdivisions)
    _lump(o, 0.08, 4.0, seed)
    if grain:
        _lump(o, grain, 14.0, seed + 5.1)
    o.data.transform(Matrix.Diagonal((*scale, 1.0)))
    for direction, depth in fractures:
        n = Vector(direction).normalized()
        reach = max(v.co.dot(n) for v in o.data.vertices)
        _cut(o, o.matrix_world @ (n * (reach - depth)), n)
    o.location.z = top - max(v.co.z for v in o.data.vertices)
    bpy.context.view_layer.update()
    return _cut(o, (0, 0, GROUND), (0, 0, -1))


def _weather(obj, seed, lichen, soil, cover=0.25):
    """Colours a stone's weathered faces (its rounded ones, not the fresh flat breaks): lichen in patches on what
    faces the sky (smooth noise over `cover`), and the damp, earthy band where it goes into the ground."""
    mats = list(obj.data.materials)
    for m in (lichen, soil):
        if m.name not in [x.name for x in mats]:
            obj.data.materials.append(m)
    names = [m.name for m in obj.data.materials]
    world = obj.matrix_world
    for p in obj.data.polygons:
        if not p.use_smooth:
            continue
        c = world @ p.center
        if c.z < 0.004:
            p.material_index = names.index(soil.name)
        elif p.normal.z > 0.45 and _noise(Vector((c.x * 7 + seed, c.y * 7 - seed, c.z * 7))) > cover:
            p.material_index = names.index(lichen.name)


def rock():
    """A grey boulder and a lighter one leaning on it, lumpy and pitted (the old part's two blobs, where they stand),
    faces split off flat, lichen in patches on their tops and the damp soil line round their foot, cut off under the
    ground, two pebbles at the foot, all plain matte: one draw call. Its far version, what the woods draw (some
    hundreds of them), is two icosahedra lumped the same way, without the split faces (each cut adds triangles) or
    the pebbles, lichen or soil line (a patch would take a whole face): 56 triangles, few enough for the woods to keep drawing them as
    one mesh (nature.ts `SPLIT_TRIANGLES`), as the old part's."""
    fractures = [((0.35, -0.2, 1), 0.07), ((-0.9, 0.3, 0.35), 0.06), ((0.2, 1, 0.1), 0.05), ((0.95, -0.15, 0.25), 0.035),
                 ((-0.3, -1, 0.45), 0.03)]
    # (Far off, an icosahedron, its corners on the round and its faces inside it: a little wider to fill the same box.)
    main = _stone('rock_main', 0.5, (0, 0, 0), M.stone, low((1.0, 0.85, 0.8), (1.08, 0.95, 0.8)), 1.3, 0.435, low(4, 1),
                  low(fractures, []), grain=low(0.025, 0.0))
    side = _stone('rock_side', 0.25, (-0.42, -0.22, 0), M.stone_light, (1.0, 0.9, 0.9), 3.7, 0.24, low(3, 1),
                  low([((-0.5, -0.6, 1), 0.04), ((-1, 0.2, 0.3), 0.025)], []), grain=low(0.03, 0.0))
    if low(True, False):
        _weather(main, 1.3, M.lichen, M.soil)
        _weather(side, 3.7, M.lichen, M.soil, cover=0.1)
    lo, hi = G.measure([main, side])
    if low(True, False):
        # Two pebbles broken off it, half sunk at its foot (inside its box, so it is centred as it was).
        for k, (x, y, r, sd) in enumerate(((0.36, -0.3, 0.07, 6.2), (-0.56, 0.08, 0.055, 8.9))):
            peb = _stone(f'pebble{k}', r, (x, y, 0), M.stone_light if k else M.stone, (1.0, 0.8, 0.6), sd, 0.035, 2, [])
            _weather(peb, sd, M.lichen, M.soil, cover=0.6)
    # Centred where the old part's lumps are (game x 8.5 cm, depth 1 cm), by hand: a far version is built
    # without the Model's `centre` set.
    obj = G.join_all('rock')
    obj.location += Vector((-ROCK_CENTRE[0] - (lo[0] + hi[0]) / 2, -ROCK_CENTRE[1] - (lo[1] + hi[1]) / 2, 0))
    bpy.context.view_layer.update()
    return [obj]


# ---- the selection marker --------------------------------------------------------------------------------------

# The old part's star: rays (half-length, half-width at the middle, half-depth at the middle) up, down and
# to the sides, as lathes whose radius eases to the tip as (1 - u) ** 1.6, round a core 5 cm round and 3 cm deep.
RAYS = [(0.17, 0.0425, 0.025), (0.115, 0.0375, 0.0225)]
CORE = (0.05, 0.03)


def _depth(a, b):
    """The star's half-depth at (a across, b up) in its plane: a smooth union of the rays' and the core's."""
    parts = []
    for (L, w0, d0), (u, l) in ((RAYS[0], (abs(b), a)), (RAYS[1], (abs(a), b))):
        if u < L:
            f = (1 - u / L) ** 1.6
            w, d = w0 * f, d0 * f
            if abs(l) < w:
                parts.append(d * math.sqrt(1 - (l / w) ** 2))
    r = math.hypot(a, b)
    if r < CORE[0]:
        parts.append(CORE[1] * math.sqrt(1 - (r / CORE[0]) ** 2))
    return sum(p ** 6 for p in parts) ** (1 / 6) if parts else 0.0


def _outline(samples=4000, keep=96):
    """The star's outline as `keep` points evenly spaced along it (so the tips get as many as the flanks),
    each the furthest point inside it along a direction from the middle."""
    pts = []
    for i in range(samples):
        t = math.tau * i / samples
        c, s = math.cos(t), math.sin(t)
        lo, hi = 0.0, 0.18
        for _ in range(30):
            m = (lo + hi) / 2
            lo, hi = (m, hi) if _depth(c * m, s * m) > 0 else (lo, m)
        pts.append((c * lo, s * lo))
    # the tips exactly: the outline's samples there fall a little short
    for k in range(4):
        L = RAYS[0][0] if k % 2 else RAYS[1][0]
        pts[k * samples // 4] = (math.cos(k * math.pi / 2) * L, math.sin(k * math.pi / 2) * L)
    lengths = [0.0]
    for i in range(1, samples + 1):
        p, q = pts[i - 1], pts[i % samples]
        lengths.append(lengths[-1] + math.hypot(q[0] - p[0], q[1] - p[1]))
    out, j = [], 0
    for k in range(keep):
        want = lengths[-1] * k / keep
        while lengths[j + 1] < want:
            j += 1
        f = (want - lengths[j]) / max(lengths[j + 1] - lengths[j], 1e-9)
        p, q = pts[j], pts[(j + 1) % samples]
        out.append((p[0] + (q[0] - p[0]) * f, p[1] + (q[1] - p[1]) * f))
    # each tip on a point of its own
    for k in range(4):
        L = RAYS[0][0] if k % 2 else RAYS[1][0]
        tip = (math.cos(k * math.pi / 2) * L, math.sin(k * math.pi / 2) * L)
        i = min(range(keep), key=lambda n: math.hypot(out[n][0] - tip[0], out[n][1] - tip[1]))
        out[i] = tip
    return out


def marker():
    """
    The selection marker: a soft four-point sparkle star, tall rays up and down and shorter ones to the sides,
    their flanks concave, round a rounder core, as the old part's lathes and sphere but one closed, smooth
    shell (its front and back meet at the outline), which gives the game's pushed-out outline no seams. In the
    game's frame it lies across x (right) and y (up), thin in z, which the game turns to the camera; its origin
    is its middle, as the old part's. Its normals come from the shell's shape (`normal`). 1,344 triangles
    (the old part: 1,936).
    """
    outline = _outline()
    n, rings = len(outline), 4
    verts, faces, normals = [], [], []
    h = 1e-4

    def normal(a, b, side):
        """The shell's normal (game) from its depth's slope: the game pushes its outline out along it, so it is
        worked out, not left to the triangles (thin ones along the rays folded the outline through the front)."""
        da = (_depth(a + h, b) - _depth(a - h, b)) / (2 * h)
        db = (_depth(a, b + h) - _depth(a, b - h)) / (2 * h)
        return Vector((-da, -db, side)).normalized()

    def to_blender(v):
        """Game (a, b, depth) -> Blender (-a, -depth, b)."""
        return Vector((-v[0], -v[2], v[1]))

    for side in (1, -1):
        verts.append(Vector((0, -side * _depth(0, 0), 0)))
        normals.append(to_blender(Vector((0, 0, side))))
        for j in range(1, rings + 1):
            f = 1 - (1 - j / rings) ** 2     # closer together towards the outline, where the shell turns
            for i, (a, b) in enumerate(outline):
                d = 0.0 if j == rings else _depth(a * f, b * f)
                verts.append(Vector((-a * f, -side * d, b * f)))
                if j == rings:
                    # On the outline: square to it, in the star's plane.
                    (pa, pb), (qa, qb) = outline[i - 1], outline[(i + 1) % n]
                    out = Vector((qb - pb, -(qa - pa), 0))
                    if out.length < 1e-9 or out.dot(Vector((a, b, 0))) <= 0:
                        out = Vector((a, b, 0))
                    normals.append(to_blender(out.normalized()))
                else:
                    normals.append(to_blender(normal(a * f, b * f, side)))
    per = 1 + rings * n
    # the back shares the outline (the last ring) with the front
    rim = [1 + (rings - 1) * n + i for i in range(n)]

    def at(side, j, i):
        if j == rings:
            return rim[i % n]
        return side * per + 1 + (j - 1) * n + i % n

    for side in (0, 1):
        c = side * per
        for i in range(n):
            faces.append((c, at(side, 1, i), at(side, 1, i + 1)))
        for j in range(1, rings):
            for i in range(n):
                faces.append((at(side, j, i), at(side, j + 1, i), at(side, j + 1, i + 1), at(side, j, i + 1)))
    used = sorted({v for f in faces for v in f})
    index = {v: k for k, v in enumerate(used)}
    obj = _mesh('marker', [verts[v] for v in used], [tuple(index[v] for v in f) for f in faces], M.star)
    # Quads to triangles along their shorter diagonal, so the outline and the rays' flanks stay smooth.
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bmesh.ops.triangulate(bm, faces=bm.faces[:], quad_method='SHORT_EDGE')
    bm.to_mesh(obj.data)
    bm.free()
    for p in obj.data.polygons:
        p.use_smooth = True
    obj.data.normals_split_custom_set_from_vertices([tuple(normals[v]) for v in used])
    return [obj]


FAMILY = G.Family(
    'ground', M,
    [G.Model('grassTuft', grassTuft, size=(0.121, 0.293, 0.135), key='model.grassTuft', far=far_of(grassTuft)),
     G.Model('flowers', flowers, size=(0.143, 0.274, 0.165), key='model.flowers', far=far_of(flowers)),
     G.Model('rock', rock, size=(1.137, 0.55, 0.829), key='model.rock', centre=ROCK_CENTRE, floor=False, far=far_of(rock),
             tolerance=0.05, sink=-GROUND),
     G.Model('marker', marker, size=(0.23, 0.34, 0.064), key='model.marker', floor=False, sink=0.17)],
    comment='The lawn\'s grass tuft and wildflowers, the woods\' rocks and the selection marker, built in '
            'Blender (tools/art/blender/families/ground.py).',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

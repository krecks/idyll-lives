# The garden's plants: G1, the houseplants (`model.plant` and `model.garden.monstera`, `fiddleFig`, `snakePlant`,
# `cacti`, `succulents`, `bostonFern`, `palm`, `orchid`, `bonsai`), G2 the flowers, G3 the shrubs and G4 the edibles (below). The outdoor ones each have
# a far version: the same build at low detail (`far_of`).
#
# Leaves are closed, thin, bent blades (`leaf`; since the detail pass mostly `cut_leaf`, cut from an outline: lobes,
# slits, leaflets) on the `foliage` finish, stems and trunks swept tubes (`tube`, `twig`), pots surfaces of revolution
# (`pot`, `vessel`), shrubs woody skeletons under broken leaf masses (`leafy_shrub`), flowers separate petals (`rays`). The game's nature shader gives every part on `foliage`
# (and `bark`) its wind sway and a per-instance tint (render/babylon/nature.ts); the pots, stands and
# flowers stay still. Each model's pot is its `body`, the part the player colours. Only the bonsai moves:
# a stray sprig shrinks away while a resident prunes it (`trim`).
#
# Run:  cargo art plants [--only monstera] [--preview]
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
    blossom=('#F8901C', 'foliage'),     # flowers drawn leafy, as the old parts' (marigolds, lupins, forget-me-nots)
    blossom2=('#FCBA1E', 'foliage'),
    blossom3=('#F4F0F2', 'foliage'),
    blossom4=('#6C8CF0', 'foliage'),
    stalk=('#5E8C3A', 'satin'),         # the meadow's picked flower's stem: one draw call with its bloom
    twig=('#5C4936', 'foliage'),        # a shrub's woody stems: on foliage, so they sway with the leaves in one draw
)
M = G.Palette(**BASE)


# Building a far version (gamekit's `far_of`): every shape with fewer segments (`low`) and no moving part
# (`LOW[0]`, gamekit's flag, set while it builds).
from gamekit.far import LOW, far_of, low  # noqa: E402,F401  (trees.py imports them from here too)


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
    if LOW[0]:
        rows, ring = max(2, rows // 2), 3
        notch = notch if rows >= 4 else None
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
    verts, cap = low(verts, 3), low(cap, False)              # far: ends hidden in the ground or a flower
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
    segments = low(segments, max(8, segments // 2))
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


# ---- the detail pass's shapes (2026-10-22) -----------------------------------------------------------------
# A leaf cut from its outline (`cut_leaf`: lobes, slits, leaflets, a wavy or toothed edge), a flower's face of
# separate rounded petals (`rays`), a tapering, curving stem or twig (`twig`), a pot with a rolled rim, a foot and
# a saucer (`vessel`), a soil surface that isn't a disc (`soil_top`) and a shrub built as a woody skeleton under
# broken leaf masses with leaves breaking its outline (`leafy_shrub`), as `trees.py` `oak`. The helpers trees.py
# imports above are as they were.

def cut_leaf(name, base, yaw, tilt, length, edge, material, droop=0.0, fold=0.25, thick=0.006, ribs=5, twist=0.0,
             left=None):
    """
    A leaf cut from an outline: `edge` is its right half [(t, w), ...] from the stalk (t 0) to the tip (t 1), `w`
    metres out from the midrib (a lobe, a slit, a leaflet: t may go back), the left half its mirror (or `left`). The
    blade is closed and thin: the top and the underside meet at the edge and stand `thick` apart along the midrib
    (`ribs` points), with no walls, so it costs two triangles a point of its outline and four a midrib point. It
    bends down along its length (`droop`, as `leaf`), lifts its halves into a V (`fold`) and turns about its midrib
    by the tip (`twist`, degrees); it starts at `base` (Blender metres), heading `yaw` round from the back and `tilt`
    up. A far version takes every third point of the outline and a third of the midrib.
    """
    right = list(edge)
    lft = list(left) if left is not None else list(edge)
    if LOW[0]:
        right, lft = right[1::3] or right[:1], lft[1::3] or lft[:1]
        ribs = max(1, ribs // 3)
    bend = 2 * droop

    def place(x, t, z):
        """The point `x` across, `t` along and `z` up off the flat leaf, bent along its midrib and twisted."""
        a = -bend * t
        if bend < 1e-4:
            c, n = Vector((0, length * t, 0)), Vector((0, 0, 1))
        else:
            c = Vector((0, length * math.sin(bend * t) / bend, -length * (1 - math.cos(bend * t)) / bend))
            n = Vector((0, -math.sin(a), math.cos(a)))
        tw = math.radians(twist * t)
        return c + Vector((x * math.cos(tw) - z * math.sin(tw), 0, 0)) + n * (x * math.sin(tw) + z * math.cos(tw))

    bm = bmesh.new()
    flat, final = [], []                                  # faces drawn flat (to triangulate), (vertex, where it goes)

    def vert(x, t, z):
        v = bm.verts.new((x, t * length, 0.0))
        final.append((v, place(x, t, z)))
        return v
    ts = [(i + 1) / (ribs + 1) for i in range(ribs)]
    ht = max(thick / 2, 0.0008)
    b, tip = vert(0, 0, 0), vert(0, 1, 0)
    top = [vert(0, t, ht * math.sin(math.pi * min(1, t * 1.6)) ** 0.5) for t in ts]
    under = [vert(0, t, -ht * math.sin(math.pi * min(1, t * 1.6)) ** 0.5) for t in ts]
    rv = [vert(w, t, fold * w) for t, w in right]
    lv = [vert(-w, t, fold * w) for t, w in lft]
    for mid, up in ((top, True), (under, False)):
        for side in (rv, lv):
            ring = [b] + mid + [tip] + list(reversed(side))
            if side is lv:
                ring = list(reversed(ring))
            if not up:
                ring = list(reversed(ring))
            try:
                f = bm.faces.new(ring)
            except ValueError:
                continue
            flat.append(f)
    bm.normal_update()
    bmesh.ops.triangulate(bm, faces=flat, quad_method='BEAUTY', ngon_method='EAR_CLIP')
    for v, p in final:
        v.co = p
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces[:])
    m = (Matrix.Translation(base) @ Matrix.Rotation(math.radians(-yaw), 4, 'Z')
         @ Matrix.Rotation(math.radians(tilt), 4, 'X'))
    bmesh.ops.transform(bm, matrix=m, verts=bm.verts[:])
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    obj.data.shade_smooth()
    obj.data.set_sharp_from_angle(angle=math.radians(70))
    return obj


def outline(shape, width, n=6, wave=0.0, waves=0, teeth=0.0):
    """A leaf's right half for `cut_leaf`: `n` points of `shape` (PROFILES, or a function of t) `width` across, its
    edge waving (`wave` of the width, `waves` times along) or toothed (`teeth`: every other point drawn in by that
    share)."""
    prof = PROFILES[shape] if isinstance(shape, str) else shape
    out = []
    for i in range(n):
        t = (i + 0.5) / n
        w = width / 2 * prof(t) * (1 + wave * math.sin(math.pi * waves * t))
        if teeth and i % 2:
            w *= 1 - teeth
        out.append((t, max(w, 0.0005)))
    return out


def pinnate(pairs, width, shape='lance', lean=0.8, fill=0.45, start=0.06, round_tip=False, stalk=0.0012):
    """A frond's right half for `cut_leaf`: `pairs` leaflets along it, each from the midrib (`stalk` wide there) out to
    a tip swept forward by `lean` of a leaflet's spacing, back to the midrib `fill` of the spacing on, so the leaflets
    stand apart; their reach follows `shape`. `round_tip`: two more points round each (broad leaflets)."""
    prof = PROFILES[shape] if isinstance(shape, str) else shape
    step = (1 - start) / pairs
    out = []
    for i in range(pairs):
        t0 = start + i * step
        tt = min(0.985, t0 + step * lean)
        reach = width / 2 * max(0.05, prof(tt))
        out.append((t0, stalk))
        if round_tip:
            out.append((t0 + step * lean * 0.35, reach * 0.75))
        out.append((tt, reach))
        if round_tip:
            out.append((min(0.99, t0 + step * (fill * 0.5 + lean * 0.6)), reach * 0.7))
        out.append((t0 + step * fill, stalk))
    return out


def slits(shape, width, n, depth=0.7, gap=0.25):
    """A split leaf's right half (a monstera's): `n` lobes, each cut from the edge `depth` of the way in to the
    midrib, the cuts `gap` of a lobe wide."""
    prof = PROFILES[shape] if isinstance(shape, str) else shape
    out = []
    for i in range(n):
        t0, t1 = (i + 0.08) / n, (i + 1) / n
        w = width / 2 * prof((t0 + t1) / 2)
        out += [(t0, w * 0.92), ((t0 * 0.4 + t1 * 0.6), w), (t1 - gap / n, w * 0.9)]
        if i < n - 1:
            out.append((t1 - gap / n * 0.4, max(0.004, w * (1 - depth))))
    return out


def rays(name, at, r, n, material, yaw=0.0, tilt=0.0, inner=0.3, cup=0.15, droop=0.0, thick=0.006, width=0.75,
         centre=0.0, shoulder=0.0):
    """A flower's face of `n` separate petals out to `r`, each rounded at its tip (two points) and narrowing to the
    middle (`inner` of `r`), cupped up by `cup` of their reach or hanging back by `droop` at their tips, `thick` at
    the middle; a closed sheet with no walls (6 triangles a petal; 10 with a `shoulder`, where a strap-shaped petal's
    sides start, a share of `r`). Faces up, leaning `tilt` towards `yaw`. A far
    version has a third of the petals, wider. `centre`: a raised boss over the middle, that high."""
    if LOW[0]:
        n, width = max(3, n // 3), min(1.0, width * 1.8)
    pts = []
    for k in range(n):
        a = math.tau * k / n
        half = math.pi / n * width
        if shoulder:
            pts.append((r * shoulder, a - half * 0.8))
        pts += [(r, a - half * 0.6), (r, a + half * 0.6)]
        if shoulder:
            pts.append((r * shoulder, a + half * 0.8))
        pts.append((r * inner, a + math.pi / n))                   # between two petals, in at the middle
    verts = []
    for rr, a in pts:
        f = rr / r
        z = (cup - droop) * rr * f
        verts.append(Vector((rr * math.cos(a), rr * math.sin(a), z)))
    m = len(verts)
    verts += [Vector((0, 0, thick / 2 + centre)), Vector((0, 0, -thick / 2))]
    ct, cb = m, m + 1
    faces = []
    for k in range(m):
        j = (k + 1) % m
        faces += [(ct, k, j), (cb, j, k)]
    obj = _mesh(name, verts, faces, material, smooth=True)
    obj.data.set_sharp_from_angle(angle=math.radians(70))
    return _put(obj, at, yaw, tilt)


def twig(name, points, r0, r1, material, verts=5, res=2):
    """A tapering stem, cane or twig (`G.sweep`) through Blender `points`, `r0` across at the first and `r1` at the
    last, curved through them (`res`); a far version three-sided and straight between them."""
    k = r1 / r0
    return G.sweep(name, points, r0, material=material, verts=low(verts, 3), resolution=low(res, 1),
                   scale=lambda w: 1 + (k - 1) * w)


def vessel(P, r_top, h, z0=0.0, r_bottom=None, lip=0.012, foot=0.0, saucer=0.0, segments=20, soil=True, belly=0.0,
           material=None):
    """
    A pot standing at height `z0`, `h` tall, its rim `r_top` across the inside: a rolled lip `lip` thick round the
    top (0: a plain, square-cut rim), a foot band `foot` high at the bottom, a body swelling by `belly`, a hollow
    top and its soil (`soil_top`) a little below the rim, and a `saucer` under it (its overhang). Returns the soil's
    height. Turned in `P.body` (or `material`), so the player colours it.
    """
    segments = low(segments, max(8, segments // 2))
    mat = material or P.body
    rb = r_bottom if r_bottom is not None else r_top * 0.72
    top = z0 + h
    prof = [(0, z0), (rb - 0.006, z0), (rb, z0 + 0.006)]
    if foot:
        prof += [(rb + 0.004, z0 + foot * 0.6), (rb + 0.002, z0 + foot)]
    body_top = top - (lip * 2.2 if lip else 0.0)
    mid = z0 + (body_top - z0) * 0.55
    rmid = rb + (r_top - rb) * 0.55 + belly
    prof += [(rmid, mid)]
    if lip:
        prof += [(r_top, body_top), (r_top + lip * 0.9, body_top + lip * 0.25), (r_top + lip, top - lip * 0.5),
                 (r_top + lip * 0.6, top), (r_top - 0.004, top)]
    else:
        prof += [(r_top + 0.008, top), (r_top - 0.004, top)]
    inner = r_top - 0.008
    prof += [(inner - 0.004, top - 0.04), (0, top - 0.04)]
    G.revolve('pot', prof, material=mat, segments=segments)
    if saucer:
        rs = rb + saucer
        G.revolve('saucer', [(0, z0 - 0.002), (rs - 0.01, z0 - 0.002), (rs, z0 + 0.008), (rs + 0.004, z0 + 0.022),
                             (rs - 0.006, z0 + 0.022), (rs - 0.012, z0 + 0.008), (0, z0 + 0.008)],
                  material=mat, segments=segments)
    if soil:
        soil_top(P, inner - 0.002, top - 0.028, segments=segments)
    return top - 0.028


def leafy_shrub(P, seed, w, h, cols, wood, name='bush', stems=5, masses=9, sats=1, leaves=36, leaf_size=0.15,
                band=(-0.7, 1.0), skirt=0.08, depth=None, mass_size=0.36, out=0.68, leaf_cols=None, blooms=None):
    """
    A shrub `w` wide (`depth` deep: `w` by default) and `h` tall (fitted to it once built), as `trees.py` `oak` is a tree: `stems` woody stems
    (`wood`) rise from the ground, part and fork out into the crown; `masses` leaf masses on an ellipsoid shell round
    it (`out` of the crown's radii), each a clump (`mass_size` of the mean radius) with `sats` smaller ones broken off
    it, lighter higher up (`cols`, darkest first); a dark heart fills the middle from `skirt` x `h` up, so the stems
    show under it; and `leaves` leaves in sprays of four (`leaf_cols`) stick out of the masses, breaking the outline;
    `blooms(masses, rng)` adds flowers on them (each mass (middle, radius, squash)) before it is fitted to its size. Its own
    random numbers (`seed`). Returns the masses (middle, radius) for flowers and perches. A far version: the stems
    unforked and three-sided, the clumps coarser (and a little bigger, for the leaves' reach), no single leaves (their random numbers still drawn).
    """
    with G.new_parts() as made:
        rng = random.Random(seed)
        far = LOW[0]
        d = depth or w
        rx, ry, rz = w / 2, d / 2, h / 2
        cz = h / 2
        m = (rx + ry + rz) / 3
        # the masses, round the shell top first (where they sit decides where the stems go)
        ga = math.pi * (3 - math.sqrt(5))
        off = rng.random() * math.tau
        spots = []
        for i in range(masses):
            t = band[1] - (band[1] - band[0]) * (i + 0.5) / masses
            ring = math.sqrt(max(0.0, 1 - t * t))
            a = off + ga * i + rng.uniform(-0.3, 0.3)
            k = out * rng.uniform(0.9, 1.08)
            s = mass_size * m * rng.uniform(0.85, 1.15)
            at = Vector((rx * ring * math.cos(a) * k, ry * ring * math.sin(a) * k, cz + rz * t * k))
            spots.append((at, s, t, a))
        # the heart: dark, filling the middle
        heart_seed = rng.random() * 9
        lump(f'{name}_heart', m * 0.66, (0, 0, h * skirt), cols[0], scale=(rx / m, ry / m, (h * (0.86 - skirt)) / (m * 1.32)),
             noise=0.25, seed=heart_seed, segments=low(9, 7), rings=low(6, 4))
        # the stems: from the ground, upright at first, parting towards the masses; each forks under two of them
        for i in range(stems):
            a = off + math.tau * (i + rng.uniform(-0.2, 0.2)) / stems
            foot = Vector((m * 0.08 * math.cos(a), m * 0.08 * math.sin(a), 0.0))
            near = sorted(spots, key=lambda sp: (sp[0].xy - Vector((math.cos(a), math.sin(a))) * m).length)[:2]
            mid = foot.lerp(near[0][0], 0.45) + Vector((0, 0, h * 0.05))
            r0 = m * rng.uniform(0.028, 0.036)
            twig(f'{name}_stem{i}', [foot, foot + Vector((0, 0, h * 0.08)), mid, near[0][0]], r0, r0 * 0.45, wood, verts=4)
            fork = near[1][0].lerp(near[0][0], 0.25)
            if not far:
                twig(f'{name}_fork{i}', [mid, mid.lerp(fork, 0.5) + Vector((0, 0, h * 0.04)), fork], r0 * 0.6, r0 * 0.3,
                     wood, verts=3, res=1)
        # the masses
        masses_out = []
        for i, (at, s, t, a) in enumerate(spots):
            light = (t - band[0]) / (band[1] - band[0])
            col = cols[max(1, min(len(cols) - 1, round(1 + light * (len(cols) - 2) + rng.uniform(-0.5, 0.5))))]
            sq, seed_i = rng.uniform(0.66, 0.8), i * 1.9 + rng.random()
            lump(f'{name}_mass{i}', s * low(1.0, 1.08), (at.x, at.y, at.z - s * sq), col, scale=(1, 1, sq), noise=0.3,
                 seed=seed_i, segments=8, rings=5)
            masses_out.append((at, s, sq))
            for j in range(sats):
                b = a + rng.uniform(-1.3, 1.3)
                e = rng.uniform(-0.4, 0.7)
                ss = s * rng.uniform(0.55, 0.72)
                p = at + Vector((math.cos(b) * math.cos(e), math.sin(b) * math.cos(e), math.sin(e))) * s * 0.85
                c2 = cols[max(1, min(len(cols) - 1, cols.index(col) + rng.choice((-1, 0, 1))))]
                sd = i * 3.1 + j + rng.random()
                lump(f'{name}_sat{i}_{j}', ss * low(1.0, 1.08), (p.x, p.y, p.z - ss * sq), c2, scale=(1, 1, sq), noise=0.32, seed=sd,
                     segments=6, rings=4)
        # sprays of four leaves breaking the outline: out of a mass's outer side, heading away from the middle
        lc = leaf_cols or cols[1:]
        for i in range(leaves // 4):
            at, s, sq = masses_out[rng.randrange(len(masses_out))]
            heading = math.atan2(at.y, at.x) + rng.uniform(-0.7, 0.7)
            e = rng.uniform(-0.1, 0.9)
            dirn = Vector((math.cos(heading) * math.cos(e), math.sin(heading) * math.cos(e), math.sin(e)))
            base = at + Vector((dirn.x, dirn.y, dirn.z * sq)) * s * rng.uniform(0.92, 1.05)
            yaw = 90 - math.degrees(heading)
            tilt = math.degrees(e) * 0.6 + rng.uniform(0, 30)
            size = leaf_size * rng.uniform(0.8, 1.2)
            if far:
                continue
            for j in range(4):
                leaf(f'{name}_leaf{i}_{j}', base, yaw + (j - 1.5) * 32, tilt + (10 if j in (1, 2) else -8),
                     size * (1.1 if j in (1, 2) else 0.85), size * 0.55, lc[(i + j) % len(lc)], shape='oval', droop=0.3,
                     thick=0.004, fold=0.3, rows=2, ring=3)
        if blooms:
            blooms(masses_out, rng)
    k = fit_to(made, w, h, d)
    return [(Vector((p.x * k[0], p.y * k[1], p.z * k[2])), r * sum(k) / 3) for p, r, _ in masses_out]


def florets(name, r, at, material, bumps=3.0, amount=0.14, flat=0.85, segments=10, rings=7, seed=0.0):
    """A ball of florets (a hydrangea's mophead, a marigold's pompom, a geranium's truss): a sphere `r` round, its
    surface bumpy with small heads (`bumps` across it, standing `amount` of `r` proud), squashed to `flat`, its
    bottom at `at`."""
    segments, rings = low(segments, max(6, segments // 2)), low(rings, max(3, rings // 2))
    G.sphere(name, r, material=material, segments=segments, rings=rings, scale=(1, 1, flat))
    obj = bpy.context.object
    if not LOW[0]:
        for v in obj.data.vertices:
            p = v.co
            n = noise_at(p.x / r * bumps + seed, p.y / r * bumps - seed, p.z / r * bumps + seed * 0.7)
            v.co = p * (1.0 + amount * (abs(n) * 2 - 0.6))
    bottom = min(v.co.z for v in obj.data.vertices)
    obj.data.transform(Matrix.Translation(Vector(at) - Vector((0, 0, bottom))))
    return obj


def on_masses(masses, rng, n, up=(0.1, 1.0)):
    """`n` spots on a shrub's masses (`leafy_shrub`), on their outer, upper sides: (point, the outward direction,
    the mass's radius), drawing the same random numbers every build."""
    out = []
    for i in range(n):
        at, s, sq = masses[i % len(masses)] if i < len(masses) else masses[rng.randrange(len(masses))]
        heading = math.atan2(at.y, at.x) + rng.uniform(-0.9, 0.9)
        e = rng.uniform(*up)
        d = Vector((math.cos(heading) * math.cos(e), math.sin(heading) * math.cos(e), math.sin(e)))
        out.append((at + Vector((d.x, d.y, d.z * sq)) * s * 1.1, d, s))
    return out


def fit_to(parts, w, h, d):
    """Scales `parts` about the origin so that together they are `w` x `h` x `d` (game: width, height, depth): for a
    plant grown from random numbers, whose exact reach only shows once it's built (None: as it is). Returns the scale
    (Blender x, y, z)."""
    bpy.context.view_layer.update()
    lo, hi = G.measure(parts)
    k = tuple(1.0 if want is None else want / (hi[i] - lo[i]) for i, want in enumerate((w, d, h)))
    scale = Matrix.Diagonal((*k, 1.0))
    for o in parts:
        o.matrix_world = scale @ o.matrix_world
    bpy.context.view_layer.update()
    return k


def soil_top(P, r, z, segments=16, mound=0.012, seed=0.0, material=None):
    """A pot's soil: a shallow mound `r` across at height `z`, a little uneven, its edge tucked into the pot."""
    prof = [(0, z - 0.012), (r, z - 0.012), (r, z - 0.004), (r * 0.8, z + mound * 0.45), (r * 0.4, z + mound * 0.9),
            (0, z + mound)]
    obj = G.revolve('soil', prof, material=material or P.soil, segments=segments)
    if not LOW[0]:
        G.jitter(obj, mound * 0.35, r * 0.6, seed=seed + 3.1, up=1.0)
    return obj


# ---- the houseplants ---------------------------------------------------------------------------------

def fiddle(t):
    """A fiddle-leaf fig's leaf: broad near its tip, waisted below the middle, narrowing to the stalk."""
    return max(0.0, math.sin(math.pi * min(1.0, t * 1.08)) ** 0.45 * (0.55 + 0.45 * t) * (1 - 0.22 * math.exp(-((t - 0.42) / 0.12) ** 2)))


def sword(t):
    """A snake plant's blade: full width from low down, narrowing to a sharp point."""
    return min(1.0, t * 6) ** 0.5 * (1 - t) ** 0.45


def plant():
    """Potted plant: a terracotta pot with a rolled rim on its saucer, and a lush arrowhead plant, three rings of
    heart-shaped leaves on arching stalks round a few young ones still rolled up in the middle."""
    P = palette(body='#C46A43', leaf='#4F8A3E', leaf_dark='#5E9A48', leaf_light='#7DB85C', stem='#7FA35A')
    ys = vessel(P, 0.19, 0.32, r_bottom=0.14, lip=0.016, foot=0.0, saucer=0.035, segments=18)
    cols = [P.leaf, P.leaf_dark, P.leaf_light]
    k = 0
    # count, heading offset, stalk length, stalk tilt, leaf length, width, tilt, droop
    for n, off, sl, st, ll, lw, lt, dr in [(8, 0, 0.17, 30, 0.22, 0.17, 0, 0.32), (7, 22, 0.28, 56, 0.23, 0.17, 22, 0.24),
                                           (5, 10, 0.37, 74, 0.22, 0.16, 44, 0.12)]:
        for i in range(n):
            yaw = off + i * 360 / n + (k * 37) % 17
            a = Vector(polar(0.03, yaw, ys))
            tip = a + Vector(polar(sl * math.cos(math.radians(st)), yaw, sl * math.sin(math.radians(st))))
            twig(f'stalk{k}', arc(a, tip, 0.03, 2), 0.006, 0.004, P.stem, verts=3, res=1)
            cut_leaf(f'leaf{k}', tip, yaw, lt, ll, outline('heart', lw, n=6), cols[k % 3], droop=dr, fold=0.22, ribs=4)
            k += 1
    for i in range(low(3, 0)):                                        # young leaves, still rolled, in the middle
        yaw = i * 120 + 40
        G.cone(f'young{i}', 0.012, 0.002, 0.14, at=(0, 0, 0), material=P.leaf_light, verts=5)
        d = Vector(polar(math.sin(0.35), yaw, math.cos(0.35)))
        bpy.context.object.data.transform(Matrix.Translation(Vector((0, 0, ys + 0.01)) + d * 0.08) @ d.to_track_quat('Z', 'Y').to_matrix().to_4x4())
    fit_to(G.meshes(), 0.772, 0.825, 0.791)
    return G.settle([G.join_all('plant')])


def monstera():
    """Monstera: a white pot, a moss pole, and big glossy leaves split from their edges almost to the midrib, on long
    arching stems, with a couple of aerial roots reaching down."""
    P = palette(body='#EDEBE6', leaf='#2F6B35', leaf_dark='#28602E', leaf_light='#3B7A3C', stem='#4E7A3A', moss='#6E6A3E')
    ys = vessel(P, 0.18, 0.33, r_bottom=0.14, lip=0.0, foot=0.02, segments=18)
    pole = G.cylinder('pole', 0.024, 0.62, at=(0.02, 0.04, ys + 0.28), material=P.moss, verts=low(8, 5))
    if not G.is_low():
        G.jitter(pole, 0.006, 0.05, seed=3.0)
    rng = random.Random(91)
    cols = [P.leaf, P.leaf_dark, P.leaf_light]
    for i in range(8):
        yaw = i / 8 * 720 + rng.uniform(-15, 15)
        out = rng.uniform(0.07, 0.13)
        h = rng.uniform(0.3, 0.6)
        tilt, length, width = rng.uniform(5, 25), rng.uniform(0.29, 0.33), rng.uniform(0.27, 0.31)
        tip = Vector(polar(out, yaw, ys + h))
        twig(f'stem{i}', arc((0, 0, ys), tip, 0.06, 3), 0.009, 0.006, P.stem, verts=4, res=1)
        cut_leaf(f'leaf{i}', tip, yaw, tilt, length, slits('heart', width, 5, depth=0.62, gap=0.22), cols[i % 3],
                 droop=0.5, fold=0.1, ribs=5, thick=0.006)
    for i in range(low(2, 0)):                                        # aerial roots
        a = 40 + i * 150
        top = Vector(polar(0.05, a, ys + 0.22))
        twig(f'root{i}', [top, top + Vector(polar(0.08, a, -0.06)), Vector(polar(0.15, a, ys - 0.005))], 0.005, 0.004,
             P.stem, verts=3)
    fit_to(G.meshes(), 0.784, 0.911, 0.791)
    return G.settle([G.join_all('monstera')])


def fiddle_fig():
    """Fiddle-leaf fig: a cream pot with a rolled rim, a slim, slightly crooked trunk in bark, and big leathery,
    wavy-edged fiddle-shaped leaves spiralling up it on short stalks."""
    P = palette(body='#D9CDB4', leaf='#2E5A2A', leaf_dark='#2A5226', leaf_light='#386832', bark='#8A7058')
    ys = vessel(P, 0.2, 0.38, r_bottom=0.15, lip=0.016, foot=0.025, segments=20)
    rng = random.Random(92)
    trunk_pts = [(0, 0, ys - 0.02), (0.01, 0.0, 0.8), (-0.012, 0.01, 1.2), (0.0, 0.0, 1.5)]
    stem = G.sweep('trunk', trunk_pts, 0.022, material=P.bark, verts=low(7, 4), resolution=low(3, 1),
                   scale=lambda w: 1 - 0.55 * w)
    if not G.is_low():
        G.jitter(stem, 0.004, 0.08, seed=2.0, up=0.0)
    n = 22
    for i in range(n):
        t = i / n
        z = ys + 0.28 + t * (1.46 - ys - 0.28)
        yaw = i * 137.5 + rng.uniform(-10, 10)
        s = 1.05 - t * 0.3
        tilt, length, width = rng.uniform(15, 40), rng.uniform(0.26, 0.3) * s, rng.uniform(0.2, 0.23) * s
        r_at = 0.02 * (1 - 0.5 * t)
        base = Vector(polar(r_at, yaw, z))
        end = base + Vector(polar(0.035, yaw, 0.02))
        twig(f'stalk{i}', [base, end], 0.005, 0.004, P.leaf_light, verts=3, res=1)
        cut_leaf(f'leaf{i}', end, yaw, tilt, length, outline(fiddle, width, n=7, wave=0.05, waves=5),
                 [P.leaf, P.leaf_dark, P.leaf_light][i % 3], droop=0.22, fold=0.16, ribs=5, thick=0.008)
    cut_leaf('top', (0, 0, 1.49), 20, 70, 0.14, outline(fiddle, 0.09, n=5), P.leaf_light, droop=0.1, ribs=3)
    fit_to(G.meshes(), 0.528, 1.626, 0.527)
    return G.settle([G.join_all('fiddleFig')])


def snake_plant():
    """Snake plant: a charcoal pot and stiff, upright, slightly twisting sword-shaped blades, banded dark green with
    pale yellow-green edges, and a couple of young ones coming up."""
    P = palette(body='#2E2E30', leaf='#2F5A34', leaf_dark='#2A4E30', leaf_light='#B8C060')
    ys = vessel(P, 0.15, 0.29, r_bottom=0.13, lip=0.012, foot=0.0, segments=18)
    rng = random.Random(93)
    cols = [P.leaf, P.leaf_dark]
    for i in range(12):
        x, y = rng.uniform(-0.08, 0.08), rng.uniform(-0.08, 0.08)
        yaw = math.degrees(math.atan2(x, y)) + rng.uniform(-20, 20)
        tilt, length, width, twist = rng.uniform(78, 88), rng.uniform(0.46, 0.64), rng.uniform(0.07, 0.09), rng.uniform(-50, 50)
        if i >= 10:                                                     # young ones
            length *= 0.55
        base = (x, y, ys - 0.02)
        cut_leaf(f'blade{i}', base, yaw, tilt, length, outline(sword, width, n=6), cols[i % 2], fold=0.3, ribs=5,
                 thick=0.014, twist=twist)
        if not G.is_low():                                              # the pale edge, just wider, just behind
            cut_leaf(f'edge{i}', (x - 0.0015 * math.sin(math.radians(yaw)), y - 0.0015 * math.cos(math.radians(yaw)), ys - 0.02),
                     yaw, tilt, length * 1.01, outline(sword, width * 1.14, n=6), P.leaf_light, fold=0.3, ribs=2,
                     thick=0.004, twist=twist)
    fit_to(G.meshes(), 0.341, 0.905, 0.32)
    return G.settle([G.join_all('snakePlant')])


def ribbed(r, ribs, depth=0.12, verts=None):
    """A cactus's ribbed cross-section: `ribs` ribs round, `depth` of `r` deep between them."""
    verts = verts or ribs * 2
    return [(r * (1 - depth * (0.5 - 0.5 * math.cos(math.tau * ribs * i / verts))) * math.cos(math.tau * i / verts),
             r * (1 - depth * (0.5 - 0.5 * math.cos(math.tau * ribs * i / verts))) * math.sin(math.tau * i / verts))
            for i in range(verts)]


def cacti():
    """Cactus trio: three little terracotta pots with rolled rims, topped with grit: a ribbed column cactus with two
    arms and a pink flower, a ribbed barrel with a crown of yellow flowers, and a prickly pear's pads."""
    P = palette(body='#C0683F', flesh='#4E8A4A', flesh_light='#6A9E5A', bloom='#E8508A', bloom_accent='#F2C14E', sand='#B8A88A')
    # game (x, depth) -> Blender (-x, -depth)
    pots = [((0.17, -0.05), 0.105, 0.18), ((-0.14, 0.08), 0.115, 0.2), ((-0.12, -0.17), 0.08, 0.13)]
    tops = []
    for i, ((x, y), r, h) in enumerate(pots):
        with G.new_parts() as parts:
            top = vessel(P, r, h, r_bottom=r * 0.75, lip=0.01, segments=10, soil=False)
            soil_top(P, r - 0.01, top, segments=low(8, 6), mound=0.006, seed=i, material=P.sand)
        for o in parts:
            o.location.x += x
            o.location.y += y
        tops.append(((x, y), top))
    (x, y), z = tops[0]                              # the column, two arms and a pink flower
    n = low(6, 4)
    secs = [[(x + u, y + v, z - 0.01 + hz) for u, v in ribbed(rr, n)] for hz, rr in
            [(0, 0.042), (0.12, 0.045), (0.26, 0.044), (0.32, 0.038), (0.35, 0.024), (0.36, 0.008)]]
    G.loft('column', secs, material=P.flesh)
    for name, pts, r in [('armL', [(x + 0.03, y, z + 0.12), (x + 0.1, y, z + 0.13), (x + 0.11, y, z + 0.2), (x + 0.11, y, z + 0.25)], 0.024),
                         ('armR', [(x - 0.03, y, z + 0.18), (x - 0.09, y, z + 0.19), (x - 0.095, y, z + 0.25), (x - 0.095, y, z + 0.28)], 0.02)]:
        G.sweep(name, G.kit.fillet(pts, 0.03, steps=2) if not G.is_low() else pts, profile=ribbed(r, n), material=P.flesh,
                scale=lambda w: 1 - 0.35 * max(0.0, (w - 0.75) / 0.25))
    rays('flower', (x, y, z + 0.345), 0.026, 8, P.bloom, inner=0.3, cup=0.6, width=0.9, centre=0.004)
    (x, y), z = tops[1]                              # the barrel, ribbed, with a crown of yellow flowers
    m = low(10, 6)
    secs = [[(x + u, y + v, z + hz) for u, v in ribbed(rr, m, 0.14)] for hz, rr in
            [(0, 0.06), (0.03, 0.09), (0.08, 0.1), (0.13, 0.085), (0.16, 0.05), (0.17, 0.02)]]
    G.loft('barrel', secs, material=P.flesh_light)
    for i in range(low(3, 1)):
        a = i * 120
        rays(f'crown{i}', Vector((x, y, z + 0.165)) + Vector(polar(0.025, a, 0)), 0.018, 6, P.bloom_accent, a, 25,
             inner=0.3, cup=0.5)
    (x, y), z = tops[2]                              # the prickly pear's pads
    for i, (dx, dz, w, h, r) in enumerate([(0, 0.06, 0.11, 0.14, 8), (0.05, 0.16, 0.08, 0.1, -28), (-0.045, 0.14, 0.07, 0.09, 30)]):
        G.sphere(f'pad{i}', 0.5, at=(x + dx, y, z + dz), material=P.flesh_light, segments=low(8, 6), rings=low(5, 4),
                 scale=(w, 0.032, h), rot=(0, r, 0))
    fit_to(G.meshes(), 0.568, 0.521, 0.465)
    return G.settle([G.join_all('cacti')])


def succulents():
    """Succulent bowl: a shallow glazed bowl with a rolled rim, grit and pebbles, five fat-leaved rosettes in blue-green
    and pink, and a spiky upright one."""
    P = palette(body='#7FA6B8', flesh='#5E8F55', flesh_light='#8FB5A0', flesh_pink='#C99AA8', sand='#C8B894')
    G.revolve('bowl', [(0, 0), (0.17, 0), (0.19, 0.008), (0.27, 0.085), (0.3, 0.11), (0.312, 0.122), (0.306, 0.13),
                       (0.292, 0.128), (0.276, 0.104), (0, 0.094)], material=P.body, segments=low(20, 12))
    soil_top(P, 0.278, 0.104, segments=low(20, 10), mound=0.01, seed=2.0, material=P.sand)
    rng = random.Random(95)
    for i in range(low(7, 0)):                                         # pebbles
        a, rr = rng.uniform(0, math.tau), rng.uniform(0.08, 0.24)
        lump(f'pebble{i}', 0.012, (rr * math.cos(a), rr * math.sin(a), 0.104), P.body, scale=(1.4, 1, 0.6), seed=i,
             noise=0.2, segments=5, rings=3)
    cols = [P.flesh_light, P.flesh_pink, P.flesh, P.flesh_light, P.flesh_pink]
    spots = [(0, 0)] + [(0.17 * math.sin(a), 0.17 * math.cos(a)) for a in (i * math.tau / 6 + 0.3 for i in range(6))]
    for n, (x, y) in enumerate(spots):
        s = rng.uniform(0.85, 1.15) * (1.15 if n == 0 else 1.0)
        if n in (3, 6):                                                # spiky ones: narrow upright leaves
            for i in range(6):
                yaw = i * 60 + n * 13
                cut_leaf(f'spike{n}_{i}', (x, y, 0.1), yaw, 62 + (i % 2) * 14, 0.1 * s, outline(sword, 0.03 * s, n=3),
                         P.flesh, fold=0.3, ribs=2, thick=0.012)
            continue
        for tier, (k, length, tilt) in enumerate([(6, 0.085, 24), (4, 0.064, 46), (3, 0.04, 70)]):
            for i in range(k):
                yaw = i * 360 / k + tier * 30 + n * 13
                leaf(f'r{n}_{tier}_{i}', (x, y, 0.1 + tier * 0.01), yaw, tilt, length * s, length * 0.62 * s,
                     cols[n % len(cols)], shape='oval', thick=0.024 * s, fold=0.2, rows=2, ring=4, droop=-0.1)
    fit_to(G.meshes(), 0.62, 0.17, 0.62)
    return G.settle([G.join_all('succulents')])


def boston_fern():
    """Fern on a stand: a white pot on a round wooden plant stand with three splayed, tapering legs, long fronds of
    leaflets arching over the rim and hanging down."""
    P = palette(body='#EDEBE6', leaf='#5E9A3E', leaf_dark='#558E38', leaf_light='#78B24E', wood='#7A5A40')
    G.revolve('stand_top', [(0, 0.538), (0.19, 0.538), (0.2, 0.548), (0.2, 0.562), (0.192, 0.572), (0, 0.572)],
              material=P.wood, segments=low(20, 10))
    for i in range(3):
        a = i * math.tau / 3 + 0.5
        twig(f'stand_leg{i}', [(0.12 * math.cos(a), 0.12 * math.sin(a), 0.545), (0.185 * math.cos(a), 0.185 * math.sin(a), 0)],
             0.02, 0.013, P.wood, verts=6, res=1)
    ys = vessel(P, 0.145, 0.24, z0=0.572, r_bottom=0.11, lip=0.0, foot=0.015, segments=16)
    rng = random.Random(96)
    cols = [P.leaf, P.leaf_dark, P.leaf_light]
    for i in range(9):
        yaw = i * 40 + rng.uniform(-10, 10)
        tilt, length, droop = rng.uniform(40, 62), rng.uniform(0.43, 0.49), rng.uniform(0.92, 1.04)
        cut_leaf(f'frond{i}', (0, 0, ys + 0.02), yaw, tilt, length, pinnate(8, 0.14, 'lance', lean=0.5, fill=0.75),
                 cols[i % 3], droop=droop, fold=0.12, ribs=4)
    for i in range(3):
        yaw = i * 120 + 30
        cut_leaf(f'crown{i}', (0, 0, ys + 0.02), yaw, 68, 0.28, pinnate(6, 0.1, 'lance', lean=0.5, fill=0.75),
                 P.leaf_light, droop=0.5, fold=0.12, ribs=3)
    fit_to(G.meshes(), 0.734, 0.979, 0.792)
    return G.settle([G.join_all('bostonFern')])


def palm():
    """Parlour palm: a cream pot with a rolled rim, slender arching stalks, each a feathery frond of narrow leaflets,
    and a young frond still upright in the middle."""
    P = palette(body='#D9CDB4', leaf='#4A8A3A', leaf_dark='#3E7A34', leaf_light='#5A9A42', stem='#5E7E3A')
    ys = vessel(P, 0.19, 0.33, r_bottom=0.14, lip=0.014, foot=0.0, segments=18)
    rng = random.Random(97)
    cols = [P.leaf, P.leaf_dark, P.leaf_light]
    for i in range(9):
        young = i == 8
        yaw = i * 45 + rng.uniform(-15, 15)
        up = rng.uniform(72, 86)
        length = rng.uniform(0.3, 0.48)
        flen, droop = rng.uniform(0.3, 0.36), rng.uniform(0.75, 0.95)
        if young:
            up, length, flen, droop = 88, 0.32, 0.22, 0.2
        tip = Vector(polar(length * math.cos(math.radians(up)), yaw, ys + length * math.sin(math.radians(up))))
        twig(f'stalk{i}', [(0, 0, ys - 0.01), Vector((0, 0, ys)).lerp(tip, 0.5) + Vector(polar(0.01, yaw, 0)), tip], 0.006, 0.004,
             P.stem, verts=3, res=1)
        cut_leaf(f'frond{i}', tip, yaw, up - 18, flen, pinnate(10, 0.26, 'lance', lean=0.85, fill=0.3), cols[i % 3],
                 droop=droop, fold=0.1, ribs=5)
    fit_to(G.meshes(), 0.770, 0.893, 0.693)
    return G.settle([G.join_all('palm')])


def _flower(name, at, facing, size, P):
    """A moth orchid's flower: three narrow sepals, two broad round petals across them and a dark lip, facing `facing`
    degrees round (yaw), upright."""
    rays(f'{name}_sepals', at, size, 3, P.bloom, facing, 90, inner=0.15, cup=0.12, width=0.55, thick=0.004)
    rays(f'{name}_petals', Vector(at) + Vector(polar(0.002, facing, 0)), size * 0.95, 2, P.bloom, facing, 90, inner=0.1,
         cup=0.1, width=0.75, thick=0.004)
    G.sphere(f'{name}_lip', size * 0.26, at=Vector(at) + Vector(polar(0.008, facing, -size * 0.2)), material=P.bloom_accent,
             segments=low(6, 4), rings=low(4, 3), scale=(1, 1, 1.3))


def orchid():
    """Orchid: a small white pot of bark on a turned wooden pedestal, three broad, fleshy leaves, silvery aerial roots,
    and a staked, arching spike of moth-orchid flowers and buds."""
    P = palette(body='#F2F0EC', leaf='#2E5A30', stem='#4E6A3A', wood='#5A4030', bloom='#F6F0F4', bloom_accent='#C2367A',
                soil='#6A4A30', flesh='#AAB8A0')
    G.revolve('pedestal', [(0, 0), (0.15, 0), (0.15, 0.03), (0.13, 0.05), (0.105, 0.08), (0.095, 0.32), (0.1, 0.52),
                           (0.12, 0.56), (0.13, 0.585), (0.16, 0.6), (0.16, 0.63), (0, 0.63)], material=P.wood,
              segments=low(16, 8))
    ys = vessel(P, 0.085, 0.16, z0=0.63, r_bottom=0.07, lip=0.008, segments=16, soil=False)
    soil_top(P, 0.077, ys, segments=low(12, 6), mound=0.012, seed=4.0)
    for i, yaw in enumerate((10, 150, 260)):
        cut_leaf(f'leaf{i}', (0, 0, ys + 0.01), yaw, 18, 0.22, outline('oval', 0.095, n=6), P.leaf, droop=0.35, fold=0.22,
                 ribs=5, thick=0.012)
    for i in range(low(3, 0)):                                         # aerial roots, over the rim
        a = 60 + i * 110
        top = Vector(polar(0.03, a, ys + 0.015))
        twig(f'root{i}', [top, top + Vector(polar(0.06, a, 0.015)), top + Vector(polar(0.1, a, -0.03))], 0.005, 0.004,
             P.flesh, verts=3)
    G.cylinder('stake', 0.003, 0.36, at=(-0.03, 0.01, ys + 0.18), material=P.wood, verts=low(5, 3))
    spike = [Vector(p) for p in [(0, 0, ys), (-0.03, 0.01, ys + 0.18), (-0.06, 0.0, ys + 0.34), (-0.16, -0.01, ys + 0.43),
                                 (-0.27, -0.02, ys + 0.4)]]
    twig('spike', spike, 0.005, 0.003, P.stem, verts=4, res=2)
    for i in range(8):
        t = 0.42 + i * 0.075
        k = min(len(spike) - 2, int(t * (len(spike) - 1)))
        f = t * (len(spike) - 1) - k
        at = spike[k].lerp(spike[k + 1], f) + Vector((0, -0.03, -0.03))
        if i < 6:
            _flower(f'flower{i}', at, 180 + 12 * (i % 3 - 1), 0.05 if i < 5 else 0.04, P)
        else:                                                           # buds at the end
            G.sphere(f'bud{i}', 0.012, at=at + Vector((0, 0.02, 0.025)), material=P.bloom, segments=low(6, 4), rings=low(4, 3),
                     scale=(1, 1, 1.3))
    fit_to(G.meshes(), 0.431, 1.198, 0.381)
    return G.settle([G.join_all('orchid')])


def bonsai():
    """Bonsai: a blue glazed tray with a rim and little feet, moss and two stones, on a wooden bonsai stand at bench
    height (its top 0.76 m, a shelf below, rails under the top); a gnarled trunk on spreading surface roots, its
    branches carrying cloud-pruned pads of foliage, and one stray sprig, which shrinks away while a resident prunes it."""
    P = palette(body='#3E5A78', wood='#6A4A32', bark='#6E5A4C', leaf='#3A6A30', leaf_dark='#346028', leaf_light='#467A38',
                stem='#5E5040', moss='#5E7A34')
    up = 0.38                                                          # the stand's top over the old knee-high table's
    G.cube('table_top', (0.62, 0.4, 0.04), at=(0, 0, 0.36 + up), material=P.wood, bevel=low(0.008, 0))
    for x in (-0.27, 0.27):
        for y in (-0.16, 0.16):
            G.cube(f'table_leg{x}{y}', (0.045, 0.045, 0.34 + up), at=(x, y, (0.34 + up) / 2), material=P.wood, bevel=low(0.005, 0))
    for y in (-0.16, 0.16):
        G.cube(f'rail{y}', (0.5, 0.022, 0.06), at=(0, y, 0.31 + up), material=P.wood)
    for x in (-0.27, 0.27):
        G.cube(f'rail_end{x}', (0.022, 0.28, 0.06), at=(x, 0, 0.31 + up), material=P.wood)
    G.cube('table_shelf', (0.56, 0.34, 0.025), at=(0, 0, 0.16), material=P.wood, bevel=low(0.005, 0))
    z0 = 0.38 + up
    secs = [G.ring(rx, ry, z0 + z, verts=low(24, 12), square=6) for z, rx, ry in
            [(0.012, 0.2, 0.125), (0.02, 0.215, 0.138), (0.062, 0.232, 0.152), (0.07, 0.232, 0.152), (0.07, 0.218, 0.138),
             (0.055, 0.215, 0.135)]]
    G.loft('tray', secs, material=P.body)
    for x in (-0.16, 0.16):
        for y in (-0.09, 0.09):
            G.cube(f'tray_foot{x}{y}', (0.03, 0.03, 0.014), at=(x, y, z0 + 0.007), material=P.body)
    G.cube('moss', (0.42, 0.26, 0.02), at=(0, 0, z0 + 0.06), material=P.moss, bevel=low(0.008, 0))
    for i, (x, y, r) in enumerate([(0.13, -0.06, 0.03), (-0.15, 0.05, 0.022)]):
        G.boulder(f'stone{i}', r, at=(x, y, z0 + 0.062), material=P.body, scale=(1.3, 1, 0.7), seed=i + 1.0, subdivisions=low(2, 1))
    # Bonsai trunks are wood here, not bark: one draw call fewer, and a trunk this short hardly bends in the wind.
    trunk = G.sweep('trunk', [(0.02, 0, 0.44 + up), (-0.05, 0.0, 0.58 + up), (0.04, 0.01, 0.7 + up), (-0.06, 0.0, 0.8 + up)],
                    0.034, material=P.wood, verts=low(9, 5), resolution=low(3, 1), scale=lambda w: 1 - 0.55 * w)
    if not G.is_low():
        G.jitter(trunk, 0.006, 0.05, seed=1.0)
    for i in range(low(4, 0)):                                        # surface roots, spreading over the moss
        a = i * 1.6 + 0.3
        twig(f'root{i}', [(0.02, 0, 0.47 + up), (0.02 + 0.06 * math.cos(a), 0.05 * math.sin(a), 0.45 + up),
                          (0.02 + 0.12 * math.cos(a), 0.08 * math.sin(a), 0.442 + up)], 0.014, 0.004, P.wood, verts=4)
    branches = [[(-0.05, 0.0, 0.58 + up), (0.08, 0.02, 0.64 + up), (0.18, 0.03, 0.68 + up)],
                [(0.03, 0.01, 0.69 + up), (-0.12, -0.02, 0.73 + up), (-0.2, -0.03, 0.72 + up)],
                [(0.0, 0.01, 0.66 + up), (0.03, 0.05, 0.71 + up), (0.03, 0.08, 0.73 + up)],
                [(-0.05, 0.0, 0.79 + up), (-0.08, -0.02, 0.82 + up), (-0.1, 0.0, 0.84 + up)]]
    for i, pts in enumerate(branches):
        twig(f'branch{i}', pts, 0.015 if i < 2 else 0.01, 0.006, P.wood, verts=5)
    # cloud-pruned pads: each a few flattened clumps, lighter on top
    for i, (x, y, z, r, sx, sy) in enumerate([(-0.07, 0.0, 0.83, 0.12, 1.4, 1.0), (0.17, 0.03, 0.7, 0.09, 1.3, 1.0),
                                              (-0.19, -0.03, 0.75, 0.08, 1.3, 0.9), (0.03, 0.07, 0.73, 0.07, 1.2, 1.0)]):
        for j in range(3):
            dx, dy = (j - 1) * r * 0.55 * sx, (0.3 if j == 1 else -0.2) * r * 0.5
            florets(f'pad{i}_{j}', r * (0.75 if j != 1 else 0.85), (x + dx, y + dy, z + up - 0.035 + (0.012 if j == 1 else 0)),
                    [P.leaf, P.leaf_dark, P.leaf_light][(i + j) % 3], bumps=3.5, amount=0.12, flat=0.45, segments=10, rings=6,
                    seed=i * 1.9 + j)
    still = G.join_all('bonsai')
    # The stray sprig, sticking up out of the top pad: a twig and a tuft, all on foliage (one draw call).
    with G.new_parts() as parts:
        tube('sprig_twig', [(-0.06, -0.03, 0.83 + up), (-0.11, -0.09, 0.88 + up), (-0.17, -0.15, 0.92 + up)], 0.006, 0.004, P.stem,
             verts=4)
        for i, (yaw, tilt) in enumerate([(200, 20), (260, 35), (320, 15), (230, 65), (150, 30)]):
            leaf(f'sprig_leaf{i}', (-0.17, -0.15, 0.915 + up), yaw, tilt, 0.065, 0.04, P.leaf_light, shape='oval', rows=3, ring=4)
    if G.is_low():                                                     # a far version: all one still part
        return G.settle([G.join_all('bonsai')])
    sprig = G.join(parts, 'sprig', origin=(-0.06, -0.03, 0.83 + up))
    G.settle([still, sprig])
    G.squash(sprig, 'trim', 0.001, seconds=0.8)
    return [still, sprig]


# ---- flowers (G2) -------------------------------------------------------------------------------------

def _put(obj, at, yaw=0.0, tilt=0.0, lift=0.0):
    """Moves a part made at the origin (facing up) to `at`, its face leaning `tilt` degrees towards `yaw`."""
    obj.data.transform(Matrix.Translation(at) @ Matrix.Rotation(math.radians(-yaw), 4, 'Z')
                       @ Matrix.Rotation(math.radians(-tilt), 4, 'X') @ Matrix.Translation((0, 0, lift)))
    return obj


def dome(name, at, r, h, material, yaw=0.0, tilt=0.0, lift=0.004):
    """A flower's centre: a low dome on its face (none on a far version: too small to see)."""
    if LOW[0]:
        return None
    G.sphere(name, 1.0, material=material, segments=low(6, 4), rings=3, scale=(r, r, h))
    return _put(bpy.context.object, at, yaw, tilt, lift)


def star(name, at, r, n, material, yaw=0.0, tilt=0.0, inner=0.5, depth=0.008, cup=0.2):
    """A flower's face: a thin star of `n` petals, `r` across its radius, the petals lifted by `cup` (a fraction
    of their reach), facing up and leaning `tilt` degrees towards `yaw`."""
    n = low(n, max(3, n // 3))
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


def cupped(name, profile, at, material, yaw=0.0, tilt=0.0, segments=6):
    """A bloom turned from `profile` [(r, z)] (a tulip, a poppy, a lily's trumpet), leaning `tilt` towards `yaw`."""
    return _put(G.revolve(name, profile, material=material, segments=low(segments, 3)), at, yaw, tilt)


def lump(name, r, at, material, scale=(1, 1, 1), seed=0.0, noise=0.25, segments=10, rings=6):
    """A soft, lumpy ball (a leafy clump, a pompom, a cluster of florets): a smooth sphere pushed about by noise,
    its bottom at `at`."""
    segments, rings = low(segments, max(6, round(segments * 0.6))), low(rings, max(3, round(rings * 0.6)))
    G.sphere(name, r, material=material, segments=segments, rings=rings, scale=scale)
    obj = bpy.context.object
    for v in obj.data.vertices:
        p = v.co
        n = noise_at(p.x / r * 1.3 + seed, p.y / r * 1.3 - seed, p.z / r * 1.3 + seed * 0.5)
        v.co = p * (1.0 + noise * n)
    bottom = min(v.co.z for v in obj.data.vertices)
    obj.data.transform(Matrix.Translation(Vector(at) - Vector((0, 0, bottom))))
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
    """A bed's low soil mound (`bed`, the part the player colours), a little uneven on top, its edge lifted where the
    soil was banked up: returns its top."""
    obj = G.revolve('bed', [(0, 0), (r, 0), (r * 0.94, h * 0.4), (r * 0.65, h * 0.85), (0, h)], material=P.bed,
                    segments=low(18, 9))
    for v in obj.data.vertices:                                       # the same on a far version: it keeps its floor
        if v.co.z > 0.004:
            v.co.z += h * 0.18 * noise_at(v.co.x * 7, v.co.y * 7, 0.5)
    return h


def clumps(P, rng, n, radius, size, h, z, cols):
    """Low leafy clumps between the flowers."""
    for i, (x, y, _) in enumerate(spiral(n, radius, rng)):
        s = rng.uniform(0.8, 1.2)
        lump(f'clump{i}', size / 2 * s, (x, y, z - h * 0.6), cols[i % len(cols)], scale=(1, 1, h / size), seed=i * 1.3 + rng.random(),
             segments=8, rings=5)


def rosettes(P, rng, n, radius, z, cols, leaves=5, length=0.12, width=0.06, shape='lance', tilt=(20, 45), droop=0.3,
             name='rosette', pinnae=0):
    """`n` low rosettes of `leaves` leaves each (or cut into `pinnae` leaflet pairs) spread over a bed: the flowers'
    own leaves, where the old beds had lumps."""
    for i, (x, y, a) in enumerate(spiral(n, radius, rng)):
        s = rng.uniform(0.8, 1.2)
        for j in range(leaves):
            yaw = a + j * 360 / leaves + rng.uniform(-15, 15)
            t = rng.uniform(*tilt)
            edge = pinnate(pinnae, width * s, shape, lean=0.6, fill=0.7) if pinnae else outline(shape, width * s, n=3)
            cut_leaf(f'{name}{i}_{j}', (x, y, z), yaw, t, length * s, edge, cols[(i + j) % len(cols)], droop=droop,
                     fold=0.25, ribs=2, thick=0.004)


def stem(name, a, b, material, r=0.006):
    return tube(name, [a, b], r, r * 0.8, material, verts=3)


def blade(name, base, yaw, tilt, length, width, material, droop=0.3):
    """A narrow leaf (grass, a bulb's leaf), cheap: three rows of three."""
    return leaf(name, base, yaw, tilt, length, width, material, shape='strap', droop=droop, thick=0.006, fold=0.3,
                rows=3, ring=3)


def stalk(name, a, b, material, r=0.005, bow=0.03):
    """A flower's stalk from `a` up to `b`, bowed a little sideways on its way (two segments, three-sided); none on a
    far version (too thin to see from afar; nothing stands on the floor on one alone)."""
    if LOW[0]:
        return None
    a, b = Vector(a), Vector(b)
    side = (b - a).cross(Vector((0, 0, 1)))
    side = side.normalized() * bow if side.length > 1e-6 else Vector((bow, 0, 0))
    return twig(name, [a, a.lerp(b, 0.5) + side, b], r, r * 0.75, material, verts=3, res=1)


def tulip_head(name, at, r, h, material, yaw=0.0, tilt=0.0):
    """A tulip: a closed cup of three petals round, their tips standing up over the gaps between them."""
    n = low(6, 3)
    secs = []
    for z, rr, tips in [(0, r * 0.3, 0), (h * 0.3, r * 0.85, 0), (h * 0.65, r, 0), (h, r * 0.7, h * 0.16)]:
        secs.append([(rr * math.cos(math.tau * k / n), rr * math.sin(math.tau * k / n),
                      z + (tips if k % 2 == 0 else 0)) for k in range(n)])
    obj = G.loft(name, secs, material=material)
    obj.data.transform(Matrix.Rotation(math.radians(30), 4, 'Z'))
    return _put(obj, at, yaw, tilt)


def tulips():
    """Tulips: a soil mound of tulips in five colours, closed cups of petals on straight, slightly bowed stems, broad
    grey-green leaves at their feet."""
    P = palette(leaf='#5E8A5A', leaf_light='#6E9A66')
    rng = random.Random(51)
    top = mound(P)
    cols = [P.bloom, P.bloom2, P.bloom3, P.bloom4, P.bloom, P.bloom5]
    for i, (x, y, a) in enumerate(spiral(17, 0.34, rng)):
        h = rng.uniform(0.42, 0.55)
        tip = (x + rng.uniform(-0.03, 0.03), y + rng.uniform(-0.03, 0.03), h)
        stalk(f'stem{i}', (x, y, top - 0.01), tip, P.stem, r=0.005)
        tulip_head(f'tulip{i}', Vector(tip) - Vector((0, 0, 0.01)), 0.032, 0.075, cols[i % len(cols)], rng.uniform(0, 360),
                   rng.uniform(0, 10))
        for k in range(2):
            yaw, t, length = a + 40 + k * 160, rng.uniform(55, 70), rng.uniform(0.2, 0.26)
            if k and i % 2:
                continue
            cut_leaf(f'leaf{i}_{k}', (x, y, top - 0.005), yaw, t, length, outline('lance', 0.075, n=4), [P.leaf, P.leaf_light][k],
                     droop=0.32, fold=0.35, ribs=3, thick=0.005)
    fit_to(G.meshes(), 0.84, 0.622, 0.843)
    return G.settle([G.join_all('tulips')])


def daffodils():
    """Daffodils: pale six-petalled flowers with ruffled orange trumpets, nodding outwards on bowed stems, and blue-green
    strap leaves."""
    P = palette(bloom='#FAE68A', centre='#F6AE1C', stem='#5E8C3A', leaf='#5E8A5A', leaf_light='#6E9A66')
    rng = random.Random(52)
    top = mound(P)
    for i, (x, y, a) in enumerate(spiral(13, 0.34, rng)):
        h = rng.uniform(0.34, 0.46)
        tip = (x, y, h)
        yaw = a + rng.uniform(-40, 40)
        stalk(f'stem{i}', (x, y, top - 0.01), tip, P.stem, r=0.004)
        rays(f'petals{i}', tip, 0.05, 6, P.bloom, yaw, 70, inner=0.3, cup=0.12, width=0.8, shoulder=0.0)
        cupped(f'trumpet{i}', [(0, 0), (0.014, 0.0), (0.018, 0.035), (0.026, 0.05), (0.02, 0.045)], tip, P.centre, yaw, 70,
               segments=low(6, 3))
        for k in range(2):
            yaw_k, t, length = yaw + 150 + k * 60, rng.uniform(70, 82), rng.uniform(0.26, 0.32)
            if LOW[0] and k:
                continue
            cut_leaf(f'leaf{i}_{k}', (x, y, top), yaw_k, t, length, outline('strap', 0.032, n=3), [P.leaf, P.leaf_light][(i + k) % 2],
                     droop=0.3, fold=0.3, ribs=2, thick=0.005)
    fit_to(G.meshes(), 0.84, 0.491, 0.84)
    return G.settle([G.join_all('daffodils')])


def daisies():
    """Daisies: low rosettes of leaves and white daisies of separate ray petals round a raised yellow eye, on thin
    stems."""
    P = palette(bloom='#FFFFFF', centre='#F6C21E', stem='#5E8C3A')
    rng = random.Random(53)
    top = mound(P)
    rosettes(P, rng, 5, 0.3, top - 0.01, [P.leaf, P.leaf_dark, P.leaf_light], leaves=4, length=0.17, width=0.065, tilt=(15, 35))
    for i, (x, y, a) in enumerate(spiral(15, 0.38, rng)):
        h = rng.uniform(0.25, 0.36)
        yaw, tilt = rng.uniform(0, 360), rng.uniform(0, 30)
        stalk(f'stem{i}', (x, y, top), (x, y, h), P.stem, r=0.004)
        rays(f'daisy{i}', (x, y, h), 0.045, 10, P.bloom, yaw, tilt, inner=0.32, cup=0.06, width=0.75, shoulder=0.0)
        if not LOW[0]:
            G.sphere(f'eye{i}', 1.0, material=P.centre, segments=5, rings=3, scale=(0.015, 0.015, 0.009))
            _put(bpy.context.object, (x, y, h), yaw, tilt, 0.004)
    fit_to(G.meshes(), 0.84, 0.376, 0.84)
    return G.settle([G.join_all('daisies')])


def sunflowers():
    """Sunflowers: four tall, hairy stalks with big, drooping heart-shaped leaves, their heavy heads turned to the
    front: two rows of golden petals round a domed brown disc of seeds, green behind."""
    P = palette(bloom='#F5C518', bloom2='#E8A812', centre_dark='#5A3A1E', stem='#5E8C3A')
    rng = random.Random(54)
    top = mound(P)
    for i, ((x, y), h) in enumerate(zip([(0.18, 0.12), (-0.17, 0.06), (0.0, -0.18), (0.04, 0.3)], [1.85, 1.6, 1.4, 1.95])):
        tip = Vector((x * 1.2, y * 1.2 - 0.04, h))
        twig(f'stalk{i}', [(x, y, top - 0.01), Vector((x, y, 0)).lerp(tip, 0.5) + Vector((0.01, 0, 0)), tip], 0.02, 0.013,
             P.stem, verts=6, res=2)
        for k in range(4):
            t = 0.25 + k * 0.16
            base = Vector((x, y, 0)).lerp(tip, t)
            yaw = rng.uniform(0, 360) + k * 120
            end = base + Vector(polar(0.05, yaw, 0.02))
            twig(f'petiole{i}_{k}', [base, end], 0.006, 0.004, P.stem, verts=3, res=1)
            cut_leaf(f'leaf{i}_{k}', end, yaw, 25, 0.22 - k * 0.015, outline('heart', 0.18 - k * 0.012, n=6, teeth=0.08),
                     [P.leaf, P.leaf_dark, P.leaf_light][k % 3], droop=0.42, fold=0.18, ribs=4)
        yaw = 180 + rng.uniform(-35, 35)
        rays(f'petals{i}', tip, 0.17, 18, P.bloom, yaw, 62, inner=0.5, cup=0.05, droop=0.06, width=0.7, thick=0.012)
        if not LOW[0]:
            rays(f'petals_b{i}', tip - Vector(polar(0.006, yaw, 0.0)), 0.15, 18, P.bloom2, yaw + 10, 62, inner=0.5, cup=0.08,
                 width=0.75, thick=0.01)
        disc = florets(f'seeds{i}', 0.088, (0, 0, -0.012), P.centre_dark, bumps=6.0, amount=0.05, flat=0.28,
                       segments=low(14, 8), rings=low(6, 4), seed=i)
        _put(disc, tip, yaw, 62)
        cupped(f'back{i}', [(0, -0.03), (0.04, -0.028), (0.08, -0.012), (0.09, 0.0), (0, 0.0)], tip, P.stem, yaw, 62,
               segments=low(10, 5))
    fit_to(G.meshes(), 0.84, 2.108, 0.907)
    return G.settle([G.join_all('sunflowers')])


def poppies():
    """Poppies: feathery grey-green leaves and red cups of four broad petals with dark middles on thin wavering stems,
    a few nodding buds."""
    P = palette(bloom='#EC3A2E', stem='#6E9464', leaf='#5E8456', leaf_light='#6E9464', centre_dark='#1E1A1A')
    rng = random.Random(55)
    top = mound(P)
    rosettes(P, rng, 6, 0.3, top - 0.01, [P.leaf, P.leaf_light], leaves=3, length=0.22, width=0.1, tilt=(20, 40),
             pinnae=3)
    for i, (x, y, a) in enumerate(spiral(16, 0.38, rng)):
        h = rng.uniform(0.4, 0.61)
        tip = (x + rng.uniform(-0.04, 0.04), y + rng.uniform(-0.04, 0.04), h)
        yaw, tilt = rng.uniform(0, 360), rng.uniform(0, 30)
        stalk(f'stem{i}', (x, y, top), tip, P.stem, r=0.0035, bow=0.04)
        if i % 5 == 4:                                                 # a bud, nodding
            lump(f'bud{i}', 0.014, Vector(tip) - Vector((0, 0, 0.03)), P.stem, scale=(1, 1, 1.4), segments=low(6, 4), rings=low(4, 3))
            continue
        rays(f'poppy{i}', tip, 0.045, 4, P.bloom, yaw, tilt, inner=0.2, cup=0.75, width=1.25, thick=0.008)
        if not LOW[0]:
            G.sphere(f'eye{i}', 1.0, material=P.centre_dark, segments=5, rings=3, scale=(0.011, 0.011, 0.012))
            _put(bpy.context.object, tip, yaw, tilt, 0.01)
    fit_to(G.meshes(), 0.84, 0.657, 0.84)
    return G.settle([G.join_all('poppies')])


def marigolds():
    """Marigolds: bushy clumps of dark, finely cut leaves covered in ruffled orange and yellow pompoms."""
    P = palette(leaf='#2F5A28', leaf_dark='#3A6A2E', leaf_light='#467634', blossom='#F8901C', blossom2='#FCBA1E',
                blossom3='#F0761A')
    rng = random.Random(56)
    top = mound(P)
    clumps(P, rng, 7, 0.33, 0.27, 0.17, top + 0.07, [P.leaf, P.leaf_dark, P.leaf_light])
    for i, (x, y, a) in enumerate(spiral(7, 0.38, rng)):              # cut leaves sticking out round the clumps
        yaw, t = a + rng.uniform(-30, 30), rng.uniform(15, 40)
        if LOW[0]:
            continue
        cut_leaf(f'leaf{i}', (x * 0.8, y * 0.8, top + 0.06), yaw, t, 0.13, pinnate(3, 0.06, 'lance', lean=0.6, fill=0.6),
                 P.leaf_light, droop=0.3, ribs=2)
    for i, (x, y, a) in enumerate(spiral(16, 0.38, rng)):
        florets(f'pom{i}', 0.05, (x, y, rng.uniform(0.19, 0.27)), [P.blossom, P.blossom2, P.blossom3][i % 3], bumps=4.5,
                amount=0.18, flat=0.75, segments=8, rings=5, seed=i * 0.7)
    fit_to(G.meshes(), 0.885, 0.34, 0.84)
    return G.settle([G.join_all('marigolds')])


def pansies():
    """Pansies: a low carpet of small round leaves and flat, overlapping five-petalled faces turned outwards, each with a
    dark blotch at its heart."""
    P = palette(bloom='#5B3A8E', bloom2='#F2C94C', bloom3='#3F5FB0', bloom4='#8E2A4A', bloom5='#F4F0E8',
                centre='#F2C94C', centre_dark='#3A2A4E')
    rng = random.Random(57)
    top = mound(P)
    rosettes(P, rng, 8, 0.34, top - 0.01, [P.leaf, P.leaf_dark, P.leaf_light], leaves=5, length=0.13, width=0.08,
             shape='oval', tilt=(10, 30))
    faces = [(P.bloom, P.centre), (P.bloom2, P.centre_dark), (P.bloom3, P.centre), (P.bloom4, P.centre), (P.bloom5, P.centre_dark)]
    for i, (x, y, a) in enumerate(spiral(15, 0.36, rng)):
        h = rng.uniform(0.17, 0.23)
        petal, eye = faces[i % len(faces)]
        yaw, tilt = a + rng.uniform(-30, 30), rng.uniform(35, 60)
        stalk(f'stem{i}', (x, y, top + 0.02), (x, y, h), P.stem, r=0.004)   # on stems: without, the faces stood in the air
        rays(f'pansy{i}', (x, y, h), 0.045, 5, petal, yaw, tilt, inner=0.25, cup=0.05, width=1.35, thick=0.006)
        if not LOW[0]:
            rays(f'eye{i}', (x, y, h), 0.02, 3, eye, yaw, tilt, inner=0.3, cup=0.05, width=1.2, thick=0.012)
    fit_to(G.meshes(), 0.84, 0.264, 0.84)
    return G.settle([G.join_all('pansies')])


def lilies():
    """Lilies: six tall stems clothed in narrow leaves, each with two open lilies, six petals curling back from a
    trumpet, and a bud."""
    P = palette(bloom='#F07A2A', bloom2='#F5EFE4', bloom3='#E98AA8')
    rng = random.Random(58)
    top = mound(P)
    cols = [P.bloom, P.bloom2, P.bloom3]
    for i, (x, y, a) in enumerate(spiral(6, 0.27, rng)):
        h = rng.uniform(0.71, 0.87)
        tip = Vector((x * 1.15, y * 1.15, h))
        stalk(f'stem{i}', (x, y, top - 0.01), tip, P.stem, r=0.007, bow=0.02)
        for k in range(4):
            t, yaw = 0.15 + k * 0.17, rng.uniform(0, 360)
            if LOW[0] and k % 2:                                       # a far version: three leaves a stem
                continue
            cut_leaf(f'leaf{i}_{k}', Vector((x, y, 0)).lerp(tip, t), yaw, 35, 0.17, outline('lance', 0.03, n=3), P.leaf,
                     droop=0.3, ribs=2, thick=0.004)
        for k in range(2):
            yaw = a + (k * 2 - 1) * 50 + rng.uniform(-15, 15)
            at = tip - Vector((0, 0, 0.03 * k))
            rays(f'lily{i}_{k}', at, 0.075, 6, cols[i % 3], yaw, 55, inner=0.18, cup=1.1, droop=0.55, width=0.7, shoulder=0.4)
            if not LOW[0]:
                cupped(f'throat{i}_{k}', [(0, 0), (0.008, 0.0), (0.016, 0.03), (0.02, 0.04), (0, 0.02)], at, cols[i % 3], yaw, 55,
                       segments=4)
        lump(f'bud{i}', 0.012, tip + Vector((0, 0, 0.005)), P.stem, scale=(1, 1, 2.4), segments=low(5, 4), rings=3)
    fit_to(G.meshes(), 0.84, 0.939, 0.869)
    return G.settle([G.join_all('lilies')])


def lupin_spike(name, at, h, r, material, seed=0.0):
    """A lupin's spike of pea flowers: a tapering column, bumpy with whorls of florets, `h` tall from `at`."""
    rings = low(8, 4)
    n = low(6, 4)
    secs = []
    for k in range(rings + 1):
        t = k / rings
        rr = r * (1 - t) ** 0.8 * (1 + (0.25 * math.sin(t * math.pi * 9) if not LOW[0] else 0)) + 0.002
        secs.append(G.ring(rr, rr, at[2] + h * t, verts=n, at=(at[0], at[1])))
    return G.loft(name, secs, material=material)


def lupins():
    """Lupins: hand-shaped leaves of narrow leaflets on stalks and tall tapering spikes of pea flowers in purples, pinks,
    white and blue."""
    P = palette(blossom='#8E6CE0', blossom2='#F08CC4', blossom3='#F4F0F2', blossom4='#6C8CF0')
    rng = random.Random(59)
    top = mound(P)
    cols = [P.leaf, P.leaf_dark, P.leaf_light]
    for i, (x, y, a) in enumerate(spiral(10, 0.3, rng)):              # the leaves: leaflets fanned round a stalk's end
        h = rng.uniform(0.12, 0.22)
        end = Vector((x * 1.15, y * 1.15, h))
        stalk(f'petiole{i}', (x * 0.6, y * 0.6, top), end, P.stem, r=0.004)
        rays(f'leaf{i}', end, 0.085, 9, cols[i % 3], a, rng.uniform(5, 25), inner=0.08, cup=0.15, droop=0.2, width=0.42,
             thick=0.004)
    cols = [P.blossom, P.blossom2, P.blossom3, P.blossom4]
    for i, (x, y, a) in enumerate(spiral(9, 0.3, rng)):
        h = rng.uniform(0.66, 0.99)
        tip = (x + rng.uniform(-0.03, 0.03), y + rng.uniform(-0.03, 0.03), h)
        stalk(f'stem{i}', (x, y, top + 0.02), (tip[0], tip[1], h - 0.34), P.stem, r=0.007)
        lupin_spike(f'spike{i}', (tip[0], tip[1], h - 0.36), 0.36, 0.045, cols[i % 4], seed=i)
    fit_to(G.meshes(), 0.84, 0.987, 0.84)
    return G.settle([G.join_all('lupins')])


def forget_me_nots():
    """Forget-me-nots: leafy clumps under drifts of tiny sky-blue flowers, a few open ones standing out."""
    P = palette(blossom='#8CB4F8', blossom2='#A4C4FC', blossom3='#7AA0F0', centre='#F6E27A')
    rng = random.Random(60)
    top = mound(P)
    clumps(P, rng, 7, 0.33, 0.28, 0.15, top + 0.05, [P.leaf, P.leaf_dark, P.leaf_light])
    cols = [P.blossom, P.blossom2, P.blossom3]
    for i, (x, y, a) in enumerate(spiral(16, 0.36, rng)):
        florets(f'drift{i}', 0.06, (x, y, rng.uniform(0.17, 0.23)), cols[i % 3], bumps=6.0, amount=0.22, flat=0.4,
                segments=8, rings=4, seed=i * 0.9)
    for i, (x, y, a) in enumerate(spiral(10, 0.34, rng)):            # single flowers standing out of the drifts
        h, yaw, tilt = rng.uniform(0.22, 0.27), rng.uniform(0, 360), rng.uniform(0, 30)
        if LOW[0]:
            continue
        rays(f'flower{i}', (x, y, h), 0.02, 5, cols[i % 3], yaw, tilt, inner=0.3, cup=0.1, width=1.1, centre=0.003)
    fit_to(G.meshes(), 0.866, 0.28, 0.84)
    return G.settle([G.join_all('forgetMeNots')])


def cosmos():
    """Cosmos: feathery leaves cut into threads and tall thin stems of pink, white and magenta flowers, eight broad
    petals round a yellow eye."""
    P = palette(bloom='#E77FB0', bloom2='#F6EEF2', bloom3='#C2367A', bloom4='#F2A6CA', centre='#F2C12E', stem='#6E9E44')
    rng = random.Random(61)
    top = mound(P)
    for i, (x, y, a) in enumerate(spiral(9, 0.3, rng)):
        yaw, t, length = rng.uniform(0, 360), rng.uniform(58, 78), rng.uniform(0.28, 0.38)
        cut_leaf(f'frond{i}', (x, y, top), yaw, t, length, pinnate(5, 0.16, 'lance', lean=0.8, fill=0.25),
                 [P.leaf_light, P.leaf][i % 2], droop=0.3, fold=0.1, ribs=3)
    cols = [P.bloom, P.bloom2, P.bloom3, P.bloom4]
    for i, (x, y, a) in enumerate(spiral(14, 0.36, rng)):
        h = rng.uniform(0.55, 0.87)
        tip = (x * 1.1, y * 1.1, h)
        yaw, tilt = rng.uniform(0, 360), rng.uniform(10, 40)
        stalk(f'stem{i}', (x, y, top - 0.01), tip, P.stem, r=0.0035, bow=0.04)
        rays(f'cosmos{i}', tip, 0.05, 8, cols[i % 4], yaw, tilt, inner=0.25, cup=0.1, width=0.9, shoulder=0.0)
        if not LOW[0]:
            G.sphere(f'eye{i}', 1.0, material=P.centre, segments=5, rings=3, scale=(0.012, 0.012, 0.008))
            _put(bpy.context.object, tip, yaw, tilt, 0.004)
    fit_to(G.meshes(), 0.885, 0.879, 0.84)
    return G.settle([G.join_all('cosmos')])


MEADOW_FLOWERS = ['bloom5', 'bloom2', 'bloom3', 'bloom', 'bloom4']


def meadow():
    """Wildflower meadow: tussocks and long grass with flowers of every colour; one flower, at the front, is picked
    (it shrinks away while a resident picks it, and is back after)."""
    P = palette(leaf='#7FA04A', leaf_dark='#6E9240', leaf_light='#A2B45E', stem='#8EAE52', bloom='#EC3A2E',
                bloom2='#F6C21E', bloom3='#A880E0', bloom4='#8CB4F8', bloom5='#FFFFFF', centre='#E8B436')
    rng = random.Random(62)
    grass = [P.leaf, P.stem, P.leaf_dark, P.leaf_light]
    for i, (x, y, _) in enumerate(spiral(3, 0.3, rng)):                # low tussocks at the grass's foot
        lump(f'tussock{i}', 0.12, (x, y, 0), grass[i % 4], scale=(1, 1, 0.4), seed=i * 1.1, segments=8, rings=5)
    # Long grass, the most of it: tall, thin blades leaning every way, and stalks gone to seed (a pale, thin head);
    # a far version builds every third blade (drawing every random number, so the flowers stand where they do).
    for i, (x, y, a) in enumerate(spiral(48, 0.36, rng)):
        length = rng.uniform(0.28, 0.48)
        tilt = rng.uniform(70, 87)
        yaw = a + rng.uniform(-50, 50)
        lean = rng.uniform(-0.06, 0.06)
        if LOW[0] and i % 3:
            continue
        leaf(f'grass{i}', (x * 0.85, y * 0.85, 0), yaw, tilt, length, 0.022, grass[i % 4], shape='strap', droop=0.24,
             thick=0.004, fold=0.35, rows=2, ring=3)
        if i % 6 == 0:
            head = (x * 0.8 + lean, y * 0.8 - lean, length * 1.02)
            stem(f'seed_stalk{i}', (x * 0.85, y * 0.85, 0), head, P.leaf_light, r=0.003)
            florets(f'seed{i}', 0.01, head, P.leaf_light, bumps=2.0, amount=0.25, flat=4.5, segments=low(5, 3), rings=low(5, 3),
                    seed=i)
    for i, (x, y, a) in enumerate(spiral(14, 0.4, rng)):
        h = rng.uniform(0.26, 0.48)
        yaw, tilt = rng.uniform(0, 360), rng.uniform(0, 30)
        stalk(f'stem{i}', (x, y, 0), (x, y, h), P.stem, r=0.0035)
        rays(f'flower{i}', (x, y, h), 0.04, 5 + i % 3, getattr(P, MEADOW_FLOWERS[i % 5]), yaw, tilt, inner=0.3, cup=0.12,
             width=0.85, centre=0.005)
    x, y, h = 0.06, -0.22, 0.48

    def picked():                                                     # the one picked: at the front, standing tall
        stem('picked_stem', (x, y, 0), (x, y, h), P.stalk, r=0.005)
        rays('picked_flower', (x, y, h), 0.06, 12, P.bloom5, 180, 30, inner=0.3, cup=0.08, width=0.75)
        G.sphere('picked_eye', 1.0, material=P.centre, segments=low(6, 4), rings=3, scale=(0.018, 0.018, 0.011))
        _put(bpy.context.object, (x, y, h), 180, 30, 0.005)
    return _fit_with_part('picked', 'meadow', picked, (x, y, 0), 'pick', (0.872, 0.575, 0.91))


def geraniums():
    """Potted geraniums: a terracotta pot with a rolled rim, round, scalloped leaves on stalks, and round trusses of red
    florets held up on stems over them (all `bloom`: "Flowers")."""
    P = palette(body='#C0683F', bloom='#D6283A')
    rng = random.Random(63)
    top = vessel(P, 0.19, 0.33, r_bottom=0.15, lip=0.014, segments=low(18, 10))
    round_leaf = lambda t: math.sin(math.pi * min(1.0, 0.1 + t * 0.9)) ** 0.5   # noqa: E731
    cols = [P.leaf, P.leaf_dark, P.leaf_light]
    for i in range(16):                                                # the leaves, in two rings
        yaw = i * 137.5 + rng.uniform(-10, 10)
        up = i % 2
        reach = rng.uniform(0.06, 0.13)
        end = Vector(polar(reach, yaw, top + (0.1 if up else 0.05) + rng.uniform(0, 0.04)))
        t = rng.uniform(15, 35) if up else rng.uniform(0, 15)
        stalk(f'petiole{i}', (0, 0, top - 0.01), end, P.stem, r=0.004, bow=0.02)
        cut_leaf(f'leaf{i}', end, yaw, t, 0.11, outline(round_leaf, 0.12, n=6, wave=0.07, waves=7), cols[i % 3],
                 droop=0.25, fold=0.15, ribs=3)
    for i, (x, y, a) in enumerate(spiral(6, 0.15, rng)):
        h = top + rng.uniform(0.2, 0.27)
        stalk(f'stem{i}', (x * 0.5, y * 0.5, top), (x, y, h), P.stem, r=0.004)
        florets(f'flowers{i}', 0.05, (x, y, h - 0.02), P.bloom, bumps=3.5, amount=0.2, flat=0.85, segments=9, rings=6, seed=i * 0.6)
    fit_to(G.meshes(), 0.424, 0.65, 0.468)
    return G.settle([G.join_all('geraniums')])


def flower_box():
    """Flower box: a white window box of boards on square legs, full of leafy plants and bright flowers, ivy trailing
    over the front."""
    P = palette(body='#ECE9E1', bloom='#E85A9A', bloom2='#F2C14E', bloom3='#9A5AC8', bloom5='#F4F0F2')
    rng = random.Random(64)
    # the box: boards (two to a side, a gap between), a rim round the top, square legs
    for y in (-0.1, 0.16):
        for k in range(2):
            G.cube(f'board{y}_{k}', (0.9, 0.025, 0.104), at=(0, y + (0.0125 if y > 0 else -0.0125), 0.355 + k * 0.106), material=P.body)
    for x in (-0.44, 0.44):
        G.cube(f'end{x}', (0.025, 0.26, 0.21), at=(x, 0.03, 0.408), material=P.body)
    G.cube('bottom', (0.86, 0.24, 0.02), at=(0, 0.03, 0.31), material=P.body)
    for y in (-0.122, 0.182):
        G.cube(f'rim{y}', (0.94, 0.035, 0.022), at=(0, y, 0.522), material=P.body, bevel=low(0.004, 0), bevel_segments=1)
    for x in (-0.462, 0.462):
        G.cube(f'rim_end{x}', (0.035, 0.27, 0.022), at=(x, 0.03, 0.522), material=P.body, bevel=low(0.004, 0), bevel_segments=1)
    for x in (-0.4, 0.4):
        for y in (-0.08, 0.14):
            G.cube(f'leg{x}{y}', (0.05, 0.05, 0.34), at=(x, y, 0.17), material=P.body, bevel=low(0.006, 0), bevel_segments=1)
    G.cube('soil', (0.84, 0.22, 0.02), at=(0, 0.03, 0.5), material=P.soil)
    for i in range(6):
        x = -0.36 + i * 0.144
        lump(f'leaves{i}', 0.1, (x, 0.03 + rng.uniform(-0.03, 0.03), 0.49), [P.leaf, P.leaf_dark, P.leaf_light][i % 3],
             scale=(1, 1.1, 0.7), seed=i * 1.3, segments=8, rings=5)
    cols = [P.bloom, P.bloom5, P.bloom3, P.bloom, P.bloom2]
    for i in range(14):
        x, y = rng.uniform(-0.4, 0.4), rng.uniform(-0.06, 0.12)
        h, yaw, tilt = rng.uniform(0.6, 0.66), rng.uniform(0, 360), rng.uniform(0, 30)
        stalk(f'stem{i}', (x, y, 0.5), (x, y, h), P.stem, r=0.003)
        rays(f'flower{i}', (x, y, h), 0.035, 5, cols[i % 5], yaw, tilt, inner=0.3, cup=0.25, width=1.0, centre=0.005)
    for i, x in enumerate((-0.3, -0.12, 0.05, 0.2, 0.32)):         # ivy trailing over the front
        top = Vector((x, -0.13, 0.53))
        pts = [top + Vector((0, 0.04, -0.01)), top, top + Vector((0.02, -0.07, -0.06)), top + Vector((0.01, -0.12, -0.2)),
               top + Vector((0.0, -0.15, -0.3))]
        twig(f'trail{i}', pts, 0.003, 0.002, P.stem, verts=3, res=1)
        for k in range(low(4, 2)):
            at = Vector(pts[1]).lerp(Vector(pts[4]), (k + 0.5) / low(4, 2))
            cut_leaf(f'ivy{i}_{k}', at, 180 + (k % 2 * 2 - 1) * 50, -10, 0.05,
                     outline(lambda t: math.sin(math.pi * t) ** 0.6 * (1.15 - 0.6 * t), 0.05, n=3, wave=0.2, waves=3),
                     [P.leaf_light, P.leaf][k % 2], droop=0.2, fold=0.2, ribs=1)
    fit_to(G.meshes(), 0.945, 0.732, 0.505)
    return G.settle([G.join_all('flowerBox')])


def flowerbed():
    """Raised flower bed: a box of boards, two to a side, with corner posts and a cap rail, full of soil, low rosettes
    of leaves and flowers in six colours on short stems."""
    P = palette(wood='#8A6446', bloom='#E8873A', bloom2='#8E6BBF', bloom3='#F0EDE6', bloom4='#D9483B', bloom5='#F2C14E')
    rng = random.Random(65)
    for s in (-1, 1):                                                  # boards, two to a side
        for k in range(2):
            G.cube(f'side{s}_{k}', (0.88, 0.04, 0.105), at=(0, s * 0.44, 0.055 + k * 0.11), material=P.wood)
            G.cube(f'end{s}_{k}', (0.04, 0.84, 0.105), at=(s * 0.44, 0, 0.055 + k * 0.11), material=P.wood)
    for x in (-0.44, 0.44):                                            # corner posts
        for y in (-0.44, 0.44):
            G.cube(f'post{x}{y}', (0.06, 0.06, 0.24), at=(x, y, 0.12), material=P.wood, bevel=low(0.006, 0), bevel_segments=1)
    for s in (-1, 1):                                                  # the cap rail round the top
        G.cube(f'cap{s}', (0.94, 0.06, 0.02), at=(0, s * 0.44, 0.235), material=P.wood)
        G.cube(f'cap_end{s}', (0.06, 0.82, 0.02), at=(s * 0.44, 0, 0.235), material=P.wood)
    G.cube('soil', (0.84, 0.84, 0.03), at=(0, 0, 0.21), material=P.soil)
    rosettes(P, rng, 8, 0.3, 0.225, [P.leaf, P.leaf_dark, P.leaf_light], leaves=4, length=0.15, width=0.07, shape='oval',
             tilt=(15, 40))
    cols = [P.bloom, P.bloom2, P.bloom3, P.bloom4, P.bloom5, P.bloom3]
    for i, (x, y, a) in enumerate(spiral(18, 0.34, rng)):
        h, yaw, tilt = rng.uniform(0.36, 0.45), rng.uniform(0, 360), rng.uniform(0, 30)
        stalk(f'stem{i}', (x, y, 0.22), (x, y, h), P.stem, r=0.003)
        rays(f'flower{i}', (x, y, h), 0.055, 6, cols[i % 6], yaw, tilt, inner=0.3, cup=0.2, width=0.95, centre=0.006)
    fit_to(G.meshes(), 0.94, 0.475, 0.94)
    return G.settle([G.join_all('flowerbed')])


# ---- shrubs (G3) -------------------------------------------------------------------------------------
# Shrubs dress every lot (nature.ts `natureDecor`: the bush, hedge, hydrangea, roses, lavender and boxwood) and the
# bush the woods, as thin instances, so they cost what the old parts did (a few hundred triangles) and each has
# a far version, a quarter of that.

SHRUB = ['#44703A', '#517E3E', '#5E8C44', '#6E9A4A', '#7EA852']


def dots(rng, n, centre, radii, size, materials, name, band=(-0.3, 0.95), out=1.0, flat=0.8, segments=8, rings=5, far_every=1):
    """Flowers spread over a shrub's crown (an ellipsoid at `centre`, `radii`), on its upper part; a far version builds
    every `far_every`th (drawing every random number)."""
    cx, cy, cz = centre
    ga = math.pi * (3 - math.sqrt(5))
    for i in range(n):
        t = band[0] + (band[1] - band[0]) * (i + 0.5) / n          # height on the ellipsoid, -1 .. 1
        a = i * ga + rng.uniform(-0.3, 0.3)
        ring = math.sqrt(max(0.0, 1 - t * t))
        at = (cx + radii[0] * ring * math.cos(a) * out, cy + radii[1] * ring * math.sin(a) * out, cz + radii[2] * t * out - size / 2)
        if LOW[0] and i % far_every:
            continue
        lump(f'{name}{i}', size / 2, at, materials[i % len(materials)], scale=(1, 1, flat), seed=i * 0.77,
             segments=segments, rings=rings)


def sprigs(name, P, centre, spread, n=5, length=0.09):
    """Shaggy sprigs sticking up out of a clipped crown round `centre` (the part a trim takes off)."""
    rng = random.Random(len(name))
    for i in range(n):
        a = i / n * 360 + rng.uniform(-20, 20)
        base = Vector(centre) + Vector(polar(spread * rng.uniform(0.3, 1.0), a, 0))
        for k in range(3):
            tilt, size = rng.uniform(45, 75), length * rng.uniform(0.8, 1.2)
            if LOW[0] and k:                                           # a far version: one leaf a sprig
                continue
            leaf(f'{name}{i}_{k}', base, a + (k - 1) * 70, tilt, size, length * 0.45,
                 [P.leaf_light, P.leaf][k % 2], shape='lance', droop=0.2, fold=0.2, rows=2, ring=3)


def perches(n, seed, centres, per=14, up=(0.35, 1.25), inset=0.02, gap=0.2, facing=0.2, bare=()):
    """A shrub's or a hedge's perches (the entry's `perches`, gamekit/perch.py), found on it as it stands: where `per`
    lines in from round and above towards each of `centres` meet its leaves, `inset` metres in, facing up (their
    normal's z over `facing`), with nothing over them, spread over it at least `gap` apart; `bare` adds points on bare
    stems that stand in the open (a rose's canes). Up to `n`. Its own random numbers (`seed`), so the shrub's are as
    they were; nothing on a far version (`G.perch.find`)."""
    rng = random.Random(seed)
    aims = [a for c in centres for a in G.perch.around(c, rng, per, up)]
    G.perch.find(n, aims, inset, gap, facing, reach=4.0, headroom=0.15, limbs=bare, limbs_headroom=0.15)


def _fit_with_part(name, still_name, build, origin, clip, size):
    """As `_with_part`, for a model fitted to its `size` (`fit_to`) with its moving part: everything built, fitted
    together, then the part joined apart (origin at `origin`, as fitted), settled and shrunk away."""
    with G.new_parts() as part:
        build()
    k = fit_to(G.meshes(), *size)
    if LOW[0]:                                                         # a far version: all one still part
        return G.settle([G.join_all(still_name)])
    still = G.join([o for o in G.meshes() if o not in part], still_name, origin=(0, 0, 0))
    moving = G.join(part, name, origin=(origin[0] * k[0], origin[1] * k[1], origin[2] * k[2]))
    G.settle([still, moving])
    G.squash(moving, clip, 0.001, seconds=0.8)
    return [still, moving]


def _with_part(name, still, build, origin, clip, kind='squash'):
    """Joins what `build` makes into a moving part `name` (origin at `origin`), settles, and shrinks it away."""
    if LOW[0]:                                                         # a far version: all one still part
        build()
        return G.settle([G.join_all(still.name)])
    with G.new_parts() as parts:
        build()
    part = G.join(parts, name, origin=origin)
    G.settle([still, part])
    G.squash(part, clip, 0.001, seconds=0.8)
    return [still, part]


def bush():
    """Round shrub: woody stems parting from the ground into broken leaf masses, lighter on top, leaves breaking its
    outline."""
    P = palette()
    leafy_shrub(P, 31, 1.043, 0.91, [getattr(P, k) for k in ('leaf_dark', 'leaf_dark', 'leaf', 'stem', 'leaf_light')], P.twig,
                depth=1.095, masses=12, sats=1, leaves=80)
    still = G.join_all('bush')
    perches(6, 131, [(0, 0, 0.45)])
    return G.settle([still])


def bush_small():
    """Small shrub: the round shrub's smaller, darker cousin (the lot borders and the woods' undergrowth)."""
    P = palette()
    leafy_shrub(P, 32, 0.832, 0.707, [getattr(P, k) for k in ('leaf_dark', 'leaf_dark', 'leaf', 'leaf', 'leaf_light')], P.twig,
                name='bush_small', depth=0.782, stems=4, masses=9, sats=1, leaves=56, leaf_size=0.12)
    still = G.join_all('bushSmall')
    perches(5, 132, [(0, 0, 0.35)], per=24)
    return G.settle([still])


def bush_flowering():
    """Flowering shrub: the round shrub hung with clusters of pink blossom, five-petalled flowers in small sprays (the lot
    borders)."""
    P = palette(blossom='#F08CBC', blossom2='#F8B4D4', blossom3='#FFF0F6', blossom4='#E870A8')
    cols = [P.blossom, P.blossom2, P.blossom3, P.blossom4]

    def blooms(masses, rng):
        for i, (at, d, s) in enumerate(on_masses(masses, rng, 26, up=(-0.3, 1.2))):
            yaw, tilt = 90 - math.degrees(math.atan2(d.y, d.x)), 90 - math.degrees(math.asin(max(-1, min(1, d.z))))
            for j in range(3):
                off = Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-0.5, 0.5))) * 0.05
                if G.is_low() and j:                                   # a far version: one flower a spray
                    continue
                rays(f'blossom{i}_{j}', at + off, 0.045, 5, cols[(i + j) % 4], yaw, tilt * 0.8, inner=0.35, cup=0.25,
                     width=0.95, centre=0.006)
    leafy_shrub(P, 33, 1.208, 1.016, [getattr(P, k) for k in ('leaf_dark', 'leaf_dark', 'leaf', 'stem', 'leaf_light')], P.twig,
                name='bush_flowering', depth=1.268, masses=10, sats=1, leaves=8, blooms=blooms)
    still = G.join_all('bushFlowering')
    perches(6, 133, [(0, 0, 0.5)])
    return G.settle([still])


def hedge_top(h):
    """Where a hedge segment's perch lines aim: along its top, a little under it."""
    return [(x, 0.0, h * 0.55) for x in (-0.55, -0.18, 0.18, 0.55)]


def street_hedge():
    """Clipped hedge segment of the streets and hedged lots: the garden hedge's block, still (rows overlap,
    street.ts spaces them about 1.05 m apart)."""
    P = palette(leaf=SHRUB[1], leaf_light=SHRUB[2], stem=SHRUB[3])
    hedge_block(P, 1.547, 1.019, 0.85, seed=134)               # 1.55 x 1.0 x 0.85 m over all, as its old part
    still = G.join_all('hedge')
    perches(6, 134, hedge_top(0.915), per=6, up=(0.6, 1.4))
    return G.settle([still])


def boxwood():
    """Boxwood ball: a clipped ball of small leaves, a little lumpy where the shears went round, on a few stems; a few
    shaggy sprigs on top, which a trim takes off."""
    P = palette(leaf='#3E6B33', leaf_dark='#365F2D', leaf_light='#4C7A3A')
    with G.new_parts() as made:
        for i in range(4):                                             # the stems under it
            a = i * math.tau / 4 + 0.4
            twig(f'stem{i}', [(0.03 * math.cos(a), 0.03 * math.sin(a), 0), (0.04 * math.cos(a), 0.04 * math.sin(a), 0.05),
                              (0.12 * math.cos(a), 0.12 * math.sin(a), 0.16)], 0.016, 0.01, P.twig, verts=5)
        ball = florets('ball', 0.36, (0, 0, 0.04), P.leaf, bumps=5.0, amount=0.035, flat=0.94, segments=low(22, 12),
                       rings=low(14, 8), seed=1.0)
        G.jitter(ball, 0.03, 0.3, seed=2.0)                           # where the shears went round
        florets('ball2', 0.16, (0.1, 0.1, 0.46), P.leaf_light, bumps=4.0, amount=0.06, flat=0.7, segments=10, rings=6, seed=3.0)
        florets('ball3', 0.14, (-0.18, -0.12, 0.36), P.leaf_dark, bumps=4.0, amount=0.06, flat=0.75, segments=10, rings=6, seed=4.0)
    fit_to(made, 0.69, 0.71, 0.72)
    still = G.join_all('boxwood')
    perches(4, 135, [(0, 0, 0.34)])
    return _with_part('sprigs', still, lambda: sprigs('sprig', P, (0.02, -0.03, 0.66), 0.12), (0.02, -0.03, 0.66), 'trim')


def topiary_cone():
    """Topiary cone: a stone pot with a rolled rim on a foot, a short bare trunk out of mossy soil and a clipped cone of
    small leaves, a little lumpy, in two greens; shaggy sprigs near its tip, which a trim takes off."""
    P = palette(body='#B9B2A6', leaf='#3E6B33', leaf_light='#4A7838', stalk='#8A7058', soil='#4E4030')
    top = vessel(P, 0.24, 0.36, r_bottom=0.18, lip=0.018, foot=0.03, segments=20)
    tube('trunk', [(0, 0, top - 0.02), (0.005, 0, top + 0.12), (0, 0, top + 0.2)], 0.03, 0.024, P.stalk, verts=6)   # plain: three draw calls
    n = low(26, 14)
    secs = []
    for z, r in [(0.18, 0.05), (0.2, 0.19), (0.27, 0.285), (0.42, 0.29), (0.68, 0.25), (0.96, 0.18), (1.2, 0.11), (1.38, 0.045),
                 (1.46, 0.0)]:
        secs.append(G.ring(max(r, 0.004), max(r, 0.004), top + z, verts=n))
    with G.new_parts() as crown:
        cone = G.loft('cone', secs, material=P.leaf)
        if not G.is_low():
            G.jitter(cone, 0.022, 0.16, seed=5.0)
    for i in range(low(14, 0)):                                       # lighter tufts where it grew back since
        a, z = i * 2.4, 0.32 + (i % 7) * 0.15
        r = 0.285 - (z - 0.32) * 0.22
        florets(f'tuft{i}', 0.05, (r * math.cos(a), r * math.sin(a), top + z), P.leaf_light, bumps=3.0, amount=0.1,
                flat=0.6, segments=6, rings=4, seed=i)
    fit_to(crown, 0.62, None, 0.65)
    still = G.join_all('topiaryCone')
    perches(4, 136, [(0, 0, top + z) for z in (0.45, 0.75, 1.05)], up=(0.15, 0.7), facing=0.1)
    tip = (0.0, 0.0, top + 1.36)                                       # sprigs reach out of the cone's sides there
    return _with_part('sprigs', still, lambda: sprigs('sprig', P, tip, 0.1, n=5, length=0.12), tip, 'trim')


def hedge_block(P, length, h, depth, name='hedge', seed=0):
    """A clipped hedge `length` x `h` x `depth` (fitted to it): a block of small leaves battered in at the foot and
    rounded over the top, lumpy where the shears went, on bare stems; tufts grown back since the last trim."""
    rng = random.Random(seed)
    with G.new_parts() as made:
        n = low(44, 20)
        secs = []
        for z, k in [(0.06, 0.8), (0.13, 0.95), (0.4, 1.0), (0.78, 0.99), (0.92, 0.9), (0.98, 0.7), (1.0, 0.3)]:
            secs.append(G.ring(length / 2 * (0.97 + 0.03 * k), depth / 2 * k, h * z, verts=n, square=low(7, 5)))
        block = G.loft(f'{name}_block', secs, material=P.leaf)
        G.jitter(block, 0.03, 0.22, seed=seed * 0.01 + 1.0)
        for i in range(7):                                           # stems under it
            x = (i - 3) / 3 * length * 0.4 + rng.uniform(-0.05, 0.05)
            y = rng.uniform(-0.12, 0.12) * depth
            twig(f'{name}_stem{i}', [(x, y, 0), (x, y, 0.06), (x * 1.05, y * 1.6, h * 0.16)], 0.017, 0.012, P.twig, verts=4)
        for i in range(low(22, 0)):                                  # tufts grown back, lighter, on the top and sides
            x = rng.uniform(-0.45, 0.45) * length
            side = rng.choice((-1, 1))
            top = rng.random() < 0.5
            at = (x, side * depth * (0.25 if top else 0.47), h * (0.99 if top else rng.uniform(0.35, 0.85)))
            florets(f'{name}_tuft{i}', 0.07, (at[0], at[1], at[2] - 0.04), [P.leaf_light, P.stem][i % 2], bumps=3.0,
                    amount=0.12, flat=0.65, segments=6, rings=4, seed=i * 0.7)
    fit_to(made, length, h, depth)


def garden_hedge():
    """Clipped hedge: a squared-off block of small leaves with a rounded top on bare stems; shaggy sprigs along it, which
    a trim takes off."""
    P = palette(leaf=SHRUB[1], leaf_light=SHRUB[2], stem=SHRUB[3])
    hedge_block(P, 1.546, 1.12, 0.92, seed=137)
    still = G.join_all('hedge')
    perches(6, 137, hedge_top(1.12), per=6, up=(0.6, 1.4))
    at = (0.1, 0.0, 1.06)
    return _with_part('sprigs', still, lambda: sprigs('sprig', P, at, 0.4, n=7), at, 'trim')


def hydrangea():
    """Hydrangea: a leafy shrub of broad leaves on woody stems, covered in big blue mopheads of florets (all one
    material: "Flowers" colours every one)."""
    P = palette(blossom='#9CB8F4')

    def blooms(masses, rng):
        for i, (at, d, s) in enumerate(on_masses(masses, rng, 12, up=(0.0, 1.2))):
            r = rng.uniform(0.11, 0.14)
            florets(f'mophead{i}', r, at + d * r * 0.4 - Vector((0, 0, r * 0.7)), P.blossom, bumps=3.2, amount=0.16,
                    flat=0.82, segments=10, rings=7, seed=i * 1.3)
    leafy_shrub(P, 72, 1.196, 0.957, [getattr(P, k) for k in ('leaf_dark', 'leaf_dark', 'leaf', 'stem', 'leaf_light')], P.twig,
                name='hydrangea', depth=1.182, masses=11, sats=1, leaves=48, leaf_size=0.17, blooms=blooms)
    still = G.join_all('hydrangea')
    perches(5, 138, [(0, 0, 0.45)])
    return G.settle([still])


# A rose: a cup of petals, its rim curled in over a tight middle (the profile turns back in and up again).
ROSE = [(0, 0), (0.02, 0.004), (0.048, 0.026), (0.058, 0.052), (0.046, 0.068), (0.026, 0.056), (0.012, 0.064), (0, 0.054)]


def rose_leaf(name, base, yaw, tilt, size, material, pairs=1, droop=0.25):
    """A rose's (or a strawberry's, a tomato's) compound leaf: `pairs` pairs of broad leaflets and one at the tip, cut
    from one outline (`cut_leaf`), `size` long."""
    return cut_leaf(name, base, yaw, tilt, size, pinnate(pairs, size * 0.75, 'oval', lean=0.75, fill=0.85, start=0.1,
                                                         round_tip=True), material, droop=droop, fold=0.2, ribs=3,
                    thick=0.003)


def rose_head(name, at, r, material, yaw=0.0, tilt=0.0):
    """A rose: an open cup of five broad outer petals round a tighter cup of four, turned a little."""
    rays(f'{name}_out', at, r, 5, material, yaw, tilt, inner=0.25, cup=0.75, width=1.05, thick=0.01)
    if LOW[0]:
        return
    rays(f'{name}_in', Vector(at) + Vector((0, 0, r * 0.12)), r * 0.62, 4, material, yaw + 30, tilt, inner=0.2, cup=1.6,
         width=1.1, thick=0.012, centre=r * 0.15)


def rosebush():
    """Rose bush: an open shrub of upright, arching canes from a low leafy base, compound leaves of broad leaflets along
    them, each cane tipped by an open red rose; the rose at the front, which a prune cuts."""
    P = palette(leaf='#2C4C26', leaf_dark='#365A2C', leaf_light='#3F6632', stem='#4A7038', bloom='#C8203A')
    rng = random.Random(73)
    leafy = [P.leaf, P.leaf_dark, P.leaf_light]
    for i, (x, y, _) in enumerate(spiral(3, 0.16, rng)):
        lump(f'base{i}', 0.18, (x, y, 0), leafy[i % 3], scale=(1, 1, 0.7), seed=i * 1.3 + rng.random(), noise=0.32,
             segments=8, rings=5)
    canes = []
    for i, (x, y, a) in enumerate(spiral(9, 0.3, rng)):
        h = rng.uniform(0.6, 0.84)
        foot, tip = Vector((x * 0.3, y * 0.3, 0.05)), Vector((x * 1.3, y * 1.3, h))
        bend = Vector((x * 1.05, y * 1.05, h * 0.6))
        canes.append((foot, bend, tip, a, rng.uniform(8, 28)))
    front = min(range(len(canes)), key=lambda i: canes[i][2].y)              # the cane nearest the front (-y)
    for i, (foot, bend, tip, a, tilt) in enumerate(canes):
        twig(f'cane{i}', [foot, bend, tip - Vector((0, 0, 0.02))], 0.01, 0.005, P.stem, verts=4)
        for k, f in enumerate((0.35, 0.62, 0.86)):                     # leaves along the cane, turning round it
            at = foot.lerp(bend, f / 0.6) if f < 0.6 else bend.lerp(tip, (f - 0.6) / 0.4)
            heading = math.degrees(a) + (k * 2 - 1) * 70 + rng.uniform(-20, 20)
            tilt_k = rng.uniform(5, 30)
            if LOW[0] and k == 1:                                      # a far version: the outer two
                continue
            rose_leaf(f'leaves{i}_{k}', at, heading, tilt_k, 0.13, leafy[(i + k) % 3])
        if i != front:
            rose_head(f'rose{i}', tip, 0.05, P.bloom, math.degrees(a), tilt)
    # Perches: on the canes (half way up their arch) and the leafy base.
    perches(5, 140, [(0, 0, 0.3)], per=8, bare=[bend.lerp(tip, 0.45) + Vector((0, 0, 0.012)) for _, bend, tip, _, _ in canes])
    foot, bend, tip, a, tilt = canes[front]
    return _with_part('rose', G.join_all('rosebush'), lambda: rose_head('cut_rose', tip, 0.05, P.bloom, math.degrees(a), tilt),
                      tuple(tip), 'cut')


def azalea():
    """Azalea: a wide, low mound of small dark leaves smothered in trusses of pink flowers, each a few open five-petalled
    flowers (one material, the colour part), down its sides as well as on top; shaggy sprigs on top, which a trim takes
    off."""
    P = palette(blossom='#F0549C')

    def blooms(masses, rng):
        for i, (at, d, s) in enumerate(on_masses(masses, rng, 34, up=(-0.35, 1.2))):
            yaw, tilt = 90 - math.degrees(math.atan2(d.y, d.x)), 90 - math.degrees(math.asin(max(-1, min(1, d.z))))
            for j in range(2):
                off = Vector((rng.uniform(-1, 1), rng.uniform(-1, 1), rng.uniform(-0.4, 0.4))) * 0.04
                if G.is_low() and j:                                   # a far version: one flower a truss
                    continue
                rays(f'flowers{i}_{j}', at + off, 0.06, 5, P.blossom, yaw + j * 36, tilt * 0.85, inner=0.3, cup=0.3,
                     width=0.95, centre=0.008)
    leafy_shrub(P, 74, 1.158, 1.013, [getattr(P, k) for k in ('leaf_dark', 'leaf_dark', 'leaf', 'stem', 'leaf_light')], P.twig,
                name='azalea', depth=1.152, masses=11, sats=1, leaves=16, leaf_size=0.1, blooms=blooms, band=(-0.6, 1.0))
    still = G.join_all('azalea')
    perches(5, 139, [(0, 0, 0.45)])
    at = (-0.05, 0.05, 0.94)
    return _with_part('sprigs', still, lambda: sprigs('sprig', P, at, 0.14), at, 'trim')


def lavender():
    """Lavender: a grey-green mound of narrow leaves and a spray of thin stems, each with a spike of purple whorls; a few
    at the front, which are cut."""
    P = palette(leaf='#748C70', leaf_light='#82987C', stem='#7E947A', stalk='#7E947A', bloom='#7E64B8')  # spikes all `bloom`: the colour part
    rng = random.Random(75)
    lump('mound', 0.29, (0, 0, 0), P.leaf, scale=(1, 1, 0.55), seed=1.0, segments=10, rings=6)
    r2 = random.Random(175)
    for i in range(22):                                                # narrow leaves standing out of it
        a = i * 137.5 + r2.uniform(-10, 10)
        rr = r2.uniform(0.07, 0.2)
        base = Vector(polar(rr, a, 0.04 + (0.24 - rr) * 0.5))
        yaw, tilt, length = a + r2.uniform(-25, 25), r2.uniform(35, 70), r2.uniform(0.08, 0.13)
        if LOW[0]:
            continue
        blade(f'needle{i}', base, yaw, tilt, length, 0.014, [P.leaf, P.leaf_light][i % 2], droop=0.15)

    def spray(points, stem_mat, cut=False):
        for i, (x, y, a) in points:
            heading = math.degrees(math.atan2(x, y))
            lean = math.radians(rng.uniform(4, 22) * (0.4 + math.hypot(x, y) * 3))
            d = Vector((math.sin(math.radians(heading)) * math.sin(lean), math.cos(math.radians(heading)) * math.sin(lean), math.cos(lean)))
            base = Vector((x * 0.5, y * 0.5, 0.16))
            length = rng.uniform(0.4, 0.56)
            tip = base + d * length
            if LOW[0]:
                G.cone(f'stem{i}', 0.012, 0.002, length + 0.14, material=P.bloom, verts=3)
                bpy.context.object.data.transform(Matrix.Translation(base + d * (length + 0.14) / 2) @ d.to_track_quat('Z', 'Y').to_matrix().to_4x4())
                continue
            pre = 'cut_' if cut else ''
            tube(f'{pre}stem{i}', [base, tip], 0.004, 0.003, stem_mat, verts=3)
            spike = florets(f'{pre}spike{i}', 0.017, (0, 0, 0), P.bloom, bumps=2.2, amount=0.3, flat=4.2, segments=5,
                            rings=6, seed=i * 0.9)
            spike.data.transform(Matrix.Translation(tip - d * 0.01) @ d.to_track_quat('Z', 'Y').to_matrix().to_4x4())

    pts = list(enumerate(spiral(24, 0.27, rng)))
    front = sorted(pts, key=lambda p: p[1][1])[:3]                     # the three nearest the front (-y)
    spray([p for p in pts if p not in front], P.stem)
    with G.new_parts() as cut:                                         # (built here, so all of it is fitted together)
        spray(front, P.stalk, cut=True)
    k = fit_to(G.meshes(), 0.607, 0.865, 0.63)
    if LOW[0]:
        return G.settle([G.join_all('lavender')])
    still = G.join([o for o in G.meshes() if o not in cut], 'lavender', origin=(0, 0, 0))
    stems = G.join(cut, 'stems', origin=(0, -0.12 * k[1], 0.2 * k[2]))
    G.settle([still, stems])
    G.squash(stems, 'cut', 0.001, seconds=0.8)
    return [still, stems]


def garden_fern():
    """Garden fern: a shuttlecock of arching fronds, each cut into leaflets along its length, round a low tuft, with a
    few young fronds still coiled up in the middle."""
    P = palette(leaf='#5E9A3E', leaf_dark='#558E38', leaf_light='#78B24E')
    rng = random.Random(76)
    lump('crown', 0.09, (0, 0, 0), P.leaf_dark, scale=(1, 1, 0.7), segments=8, rings=5)
    for i in range(8):
        yaw = 90 + i / 8 * 360 + rng.uniform(-12, 12)
        tilt, length, droop, twist = rng.uniform(42, 56), rng.uniform(0.54, 0.64), rng.uniform(0.38, 0.48), rng.uniform(-15, 15)
        cut_leaf(f'frond{i}', (0, 0, 0.1), yaw, tilt, length, pinnate(12, 0.3, 'lance', lean=0.5, fill=0.8),
                 [P.leaf, P.leaf_dark, P.leaf][i % 3], droop=droop, fold=0.12, ribs=6, twist=twist)
        tilt, length = rng.uniform(65, 78), rng.uniform(0.32, 0.38)
        if i % 2:                                                      # four inner fronds
            continue
        cut_leaf(f'inner{i}', (0, 0, 0.14), yaw + 20, tilt, length, pinnate(8, 0.2, 'lance', lean=0.5, fill=0.8),
                 P.leaf_light, droop=0.45, fold=0.12, ribs=4)
    for i in range(low(2, 0)):                                         # fiddleheads: young fronds still coiled
        a = i * 2.1
        base = Vector((0.03 * math.cos(a), 0.03 * math.sin(a), 0.08))
        pts = [base, base + Vector((0, 0, 0.12))]
        for k in range(6):
            t = k / 5 * math.pi * 1.6
            rr = 0.03 * (1 - k / 7)
            pts.append(base + Vector((math.cos(a) * rr * math.sin(t), math.sin(a) * rr * math.sin(t), 0.15 + rr * math.cos(t))))
        G.sweep(f'fiddle{i}', pts, 0.008, material=P.leaf_light, verts=4, scale=lambda w: 1 - 0.5 * w)
    fit_to(G.meshes(), 0.793, 0.478, 0.932)
    return G.settle([G.join_all('fern')])


def ornamental_grass():
    """Ornamental grass: a fountain of long, thin, arching blades and feathery pale plumes on tall stems."""
    P = palette(leaf='#9AAA5A', leaf_dark='#8A9E4E', leaf_light='#B2B86A', stem='#A6B062', bloom='#E2D6A8', stalk='#B8B070')
    rng = random.Random(77)
    lump('tuft', 0.15, (0, 0, 0), P.leaf_dark, scale=(1, 1, 0.5), segments=8, rings=5)
    cols = [P.leaf, P.leaf_light, P.leaf_dark, P.stem]
    for i, (x, y, a) in enumerate(spiral(54, 0.13, rng)):
        out = math.hypot(x, y)
        length, yaw, droop = rng.uniform(0.7, 1.08), math.degrees(math.atan2(x, y)) + rng.uniform(-25, 25), rng.uniform(0.18, 0.3)
        tilt = 88 - out * 150 - rng.uniform(0, 10)
        if LOW[0] and i % 3:
            continue
        leaf(f'blade{i}', (x, y, 0), yaw, tilt, length, low(0.024, 0.04), cols[i % 4], shape='strap', droop=droop, thick=0.004,
             fold=0.35, rows=4, ring=3)
    for i in range(7):
        yaw = i * 360 / 7 + rng.uniform(-15, 15)
        lean = math.radians(rng.uniform(6, 16))
        d = Vector((math.sin(math.radians(yaw)) * math.sin(lean), math.cos(math.radians(yaw)) * math.sin(lean), math.cos(lean)))
        tip = d * 0.96
        twig(f'plume_stem{i}', [(0, 0, 0), d * 0.5, tip], 0.004, 0.003, P.stem, verts=3, res=1)
        plume = florets(f'plume{i}', 0.032, (0, 0, 0), P.bloom, bumps=2.6, amount=0.3, flat=4.0, segments=6, rings=7,
                        seed=i * 1.1)
        bend = Matrix.Rotation(math.radians(12), 4, 'X')               # nodding a little
        plume.data.transform(Matrix.Translation(tip - d * 0.03) @ d.to_track_quat('Z', 'Y').to_matrix().to_4x4() @ bend)
    fit_to(G.meshes(), 1.218, 1.21, 1.28)
    return G.settle([G.join_all('grass')])


def hosta():
    """Hosta: a clump of broad heart-shaped leaves with wavy edges, folded along their midribs, on stalks, and a few tall
    stems of nodding lilac bells."""
    P = palette(leaf='#5E8A5E', leaf_dark='#4E7A4A', leaf_light='#8EAE6C', stem='#6A9464', bloom='#B8A6D8')
    rng = random.Random(78)
    lump('crown', 0.13, (0, 0, 0), P.leaf_dark, scale=(1, 1, 0.5), segments=8, rings=5)
    for i in range(14):
        yaw = i / 14 * 720 + rng.uniform(-10, 10)
        up = i >= 7
        length = rng.uniform(0.26, 0.32)
        reach = 0.1 if up else 0.13
        rise = math.radians(rng.uniform(40, 55) if up else rng.uniform(22, 32))
        tilt = rng.uniform(25, 40) if up else rng.uniform(5, 15)
        base = Vector((0, 0, 0.05))
        end = base + Vector(polar(reach * math.cos(rise), yaw, reach * math.sin(rise) + 0.04))
        twig(f'stalk{i}', [base, base.lerp(end, 0.5) + Vector((0, 0, 0.02)), end], 0.008, 0.006, P.stem, verts=3, res=1)
        cut_leaf(f'leaf{i}', end, yaw, tilt, length, outline('heart', 0.2, n=7, wave=0.06, waves=6),
                 [P.leaf, P.leaf_light, P.leaf][i % 3], droop=0.3, fold=0.32, ribs=6, thick=0.006)
    for i in range(3):
        top = Vector((rng.uniform(-0.08, 0.08), rng.uniform(-0.08, 0.08), rng.uniform(0.66, 0.8)))
        twig(f'scape{i}', [(0, 0, 0.1), top * 0.5 + Vector((0, 0, 0.05)), top], 0.005, 0.004, P.stem, verts=3)
        for k in range(6):                                             # bells nodding off one side, up the stem
            if LOW[0] and k % 3:
                continue
            z = top.z - 0.02 - k * 0.035
            side = Vector(polar(0.022, i * 90 + k * 50, 0))
            at = Vector((top.x, top.y, z)) + side
            cupped(f'bell{i}_{k}', [(0, 0), (0.006, 0.0), (0.013, 0.02), (0.018, 0.036), (0.012, 0.033), (0, 0.012)], at,
                   P.bloom, i * 90 + k * 50, 150, segments=5)
    fit_to(G.meshes(), 0.621, 0.842, 0.675)
    return G.settle([G.join_all('hosta')])


# ---- edibles (G4) ------------------------------------------------------------------------------------
# A crop's leaves and its crop are two moving parts with one growth clip, `grow` (`motions.growth`, G.stages): ripe
# at rest; picked, the crop gone; growing, the leaves lower and no crop; planted, seedlings. The leaves are one part
# (one draw call), so rather than shrink about the bed's middle (which drew the seedlings together there) they
# flatten in height only and sink into the soil (`SINK`, shares of their height): what shows is each plant's top,
# small, where the plant stands.

PLANTS = dict(picked=1, growing=(1, 1, 0.8), planted=(1, 1, 0.5))
SINK = (0, 0.1, 0.32)
# A bed whose crop is the plants themselves (lettuces, cabbages, carrots): picked, they are gone under the soil.
HARVESTED = dict(picked=(1, 1, 0.5), sink=0.6)
CROP = dict(picked=0.001, growing=0.001, planted=0.001)


def _crop(still, origin, leaves, crop, harvested=False, sink=SINK, size=None):
    """The still part, and the leaves and crop `leaves()` and `crop()` make as the growth clip's two parts.
    `harvested`: picking takes the plants too (a vegetable patch), not only their crop (tomatoes); `sink`: how far
    the plants go into the soil at each stage, shares of their height (low plants on a bed sink less: `SINK` would
    bury them); `size`: the whole fitted to it first (`fit_to`)."""
    if LOW[0]:                                                         # a far version: all one still part, ripe
        leaves()
        crop()
        if size:
            fit_to(G.meshes(), *size)
        return G.settle([G.join_all(still.name)])
    with G.new_parts() as lp:
        leaves()
    with G.new_parts() as cp:
        crop()
    k = fit_to(G.meshes(), *size) if size else (1.0, 1.0, 1.0)
    origin = (origin[0] * k[0], origin[1] * k[1], origin[2] * k[2])
    plants = G.join(lp, 'plants', origin=origin)
    fruit = G.join(cp, 'crop', origin=origin)
    G.settle([still, plants, fruit])
    height = G.measure([plants])[1][2] - plants.location[2]           # the plants' top over their origin
    sink = [s * height for s in sink]
    stages = dict(PLANTS)
    if harvested:
        stages['picked'] = HARVESTED['picked']
        sink[0] = HARVESTED['sink'] * height
    G.stages(plants, 'grow', **stages, sink=sink)
    G.stages(fruit, 'grow', **CROP)
    return [still, plants, fruit]


def raised_bed(P, w, d, h):
    """A raised bed of boards (`body`, the part the player colours), two to a side, with corner posts, full of soil
    mounded into rows: returns the soil's height."""
    for s in (-1, 1):
        for k in range(2):
            G.cube(f'board{s}_{k}', (w - 0.08, 0.04, h / 2 - 0.004), at=(0, s * (d / 2 - 0.02), h / 4 + k * h / 2), material=P.body)
            G.cube(f'board_end{s}_{k}', (0.04, d - 0.08, h / 2 - 0.004), at=(s * (w / 2 - 0.02), 0, h / 4 + k * h / 2), material=P.body)
    for x in (-1, 1):
        for y in (-1, 1):
            G.cube(f'post{x}{y}', (0.05, 0.05, h + 0.02), at=(x * (w / 2 - 0.025), y * (d / 2 - 0.025), (h + 0.02) / 2),
                   material=P.body, bevel=low(0.005, 0), bevel_segments=1)
    G.cube('soil', (w - 0.08, d - 0.08, 0.03), at=(0, 0, h - 0.015), material=P.soil)
    return h


def veg_patch():
    """Vegetable patch: a raised bed of ruffled lettuces, cabbages wrapped in broad outer leaves and feathery carrot
    tops, grown from seedlings to ripe."""
    P = palette(body='#8A6446', leaf='#9CC45A', leaf_light='#8AB44E', leaf_dark='#7EA48A', stem='#5E9A3A',
                bloom='#A8C8A0', bloom4='#E8782A')
    rng = random.Random(81)
    top = raised_bed(P, 0.92, 0.92, 0.18)
    still = G.join_all('vegPatch')
    spots = [[(x + rng.uniform(-0.03, 0.03), y + rng.uniform(-0.03, 0.03)) for y in (0.27, 0.0, -0.27)] for x in (-0.28, 0.0, 0.28)]
    tops = [[(rng.uniform(0.22, 0.28), j * 120 + rng.uniform(0, 40)) for j in range(3)] for _ in range(3)]
    ruffled = outline('oval', 0.12, n=4, wave=0.18, waves=5)

    def leaves():
        for k, (x, y) in enumerate(spots[0]):                          # lettuces: rosettes of ruffled leaves, cupped up
            lump(f'lettuce_heart{k}', 0.045, (x, y, top - 0.01), P.leaf_light, scale=(1, 1, 1.1), noise=0.3, seed=k,
                 segments=6, rings=4)
            for j in range(6):
                inner = j >= 4
                cut_leaf(f'lettuce{k}_{j}', (x, y, top + (0.03 if inner else 0.01)), j * 360 / (2 if inner else 4) + k * 25 + (40 if inner else 0),
                         60 if inner else 30, 0.11 if inner else 0.13, ruffled, [P.leaf, P.leaf_light][(j + k) % 2],
                         droop=0.35, fold=0.4, ribs=2)
        for k, (x, y) in enumerate(spots[1]):                          # cabbages' outer leaves
            for j in range(5):
                cut_leaf(f'cabbage{k}_{j}', (x, y, top + 0.01), j * 72 + k * 20, 38, 0.15, outline('oval', 0.14, n=4, wave=0.08, waves=3),
                         P.leaf_dark, droop=0.4, fold=0.45, ribs=2, thick=0.005)
        for k, (x, y) in enumerate(spots[2]):                          # carrot tops: feathery fronds
            for j, (length, yaw) in enumerate(tops[k]):
                cut_leaf(f'carrot_top{k}_{j}', (x, y, top), yaw, 70 - j * 8, length, pinnate(3, 0.08, 'lance', lean=0.7, fill=0.35),
                         P.stem, droop=0.3, fold=0.1, ribs=2)

    def crop():
        for k, (x, y) in enumerate(spots[1]):                          # cabbage hearts
            lump(f'heart{k}', 0.07, (x, y, top + 0.04), P.bloom, scale=(1, 1, 0.85), noise=0.1, seed=k + 6, segments=low(9, 6),
                 rings=low(6, 4))
        for k, (x, y) in enumerate(spots[2]):                          # carrot shoulders
            G.cone(f'carrot{k}', 0.022, 0.016, 0.03, at=(x, y, top + 0.01), material=P.bloom4, verts=low(8, 5))
    return _crop(still, (0, 0, top), leaves, crop, harvested=True, size=(0.92, 0.398, 0.92))   # picked, the bed is bare


def tomatoes():
    """Tomato plants: three plants tied to canes on a soil mound, a stem climbing each cane with leaves of toothed
    leaflets off it, hung with trusses of red, orange and green tomatoes, each with its green star of a calyx."""
    P = palette(leaf='#4A7A36', leaf_dark='#5A8A3C', leaf_light='#6A9A44', stem='#7AA84C', flesh='#C9A86A',
                bloom='#D63A2A', bloom2='#E04A30', bloom3='#C8302A', bloom4='#E8902E', flesh_light='#8DB04A')
    rng = random.Random(82)
    top = mound(P, r=0.4)
    canes = [(0.2, 0.15), (-0.2, 0.12), (0.0, -0.2)]
    for i, (x, y) in enumerate(canes):
        G.cylinder(f'cane{i}', 0.011, 1.18, at=(x, y, 0.6), material=P.flesh, verts=low(6, 4))
        for k in range(low(3, 0)):                                     # ties
            G.cylinder(f'tie{i}_{k}', 0.016, 0.012, at=(x, y, 0.35 + k * 0.28), material=P.flesh, verts=6)
    still = G.join_all('tomatoes')
    clumps_at = [[(x + rng.uniform(-0.06, 0.06), y + rng.uniform(-0.06, 0.06), 0.14 + k * 0.24 + rng.uniform(-0.04, 0.04))
                  for k in range(4)] for x, y in canes]
    fruit_at = []
    for i in range(15):
        x, y = rng.choice(canes)
        a = rng.uniform(0, math.tau)
        fruit_at.append((x + 0.13 * math.cos(a), y + 0.13 * math.sin(a), rng.uniform(0.2, 0.92)))
    cols = [P.leaf, P.leaf_dark, P.leaf_light, P.stem]

    def leaves():
        r2 = random.Random(182)
        for i, cane in enumerate(clumps_at):
            x, y = canes[i]
            twig(f'stem{i}', [(x + 0.02, y, top - 0.01), (x + 0.025, y + 0.01, 0.4), (x + 0.02, y - 0.01, 0.8), (x + 0.015, y, 1.08)],
                 0.012, 0.006, P.stem, verts=4)
            for k, at in enumerate(cane):
                lump(f'leaves{i}_{k}', 0.1, at, cols[(k + 1) % 4], scale=(1, 1, 0.75), noise=0.35, seed=i * 4 + k,
                     segments=low(7, 6), rings=low(5, 3))
                for j in range(2):                                     # leaves of leaflets standing out of the clump
                    yaw, t = r2.uniform(0, 360), r2.uniform(-5, 25)
                    base = Vector(at) + Vector((0, 0, 0.06))
                    if LOW[0]:
                        continue
                    rose_leaf(f'leaf{i}_{k}_{j}', base, yaw, t, 0.17, cols[(j + k) % 4], pairs=2, droop=0.35)

    def crop():
        cols_f = [P.bloom, P.bloom2, P.bloom3, P.bloom4, P.flesh_light]
        for i, at in enumerate(fruit_at):
            lump(f'tomato{i}', 0.035, at, cols_f[i % 5], noise=0.05, segments=low(7, 5), rings=low(5, 4), scale=(1, 1, 0.85))
            if not LOW[0]:
                rays(f'calyx{i}', Vector(at) + Vector((0, 0, 0.058)), 0.022, 5, P.stalk, inner=0.2, cup=-0.2, width=0.5,
                     thick=0.003)
    return _crop(still, (0, 0, top), leaves, crop, size=(0.8, 1.19, 0.8))


def strawberries():
    """Strawberry bed: a bed of straw, plants of three-lobed, toothed leaves on stalks round a dark heart, white flowers
    and red berries with green caps tucked in."""
    P = palette(bed='#D8C27A', leaf='#4A7A36', leaf_dark='#5A8A3C', leaf_light='#6A9A44', bloom='#D8263A',
                bloom5='#F6F4EE', centre='#F2C12E', stalk='#4A7A36')
    rng = random.Random(83)
    straw = G.revolve('bed', [(0, 0), (0.44, 0), (0.43, 0.012), (0.4, 0.026), (0.3, 0.045), (0, 0.05)], material=P.bed,
                      segments=low(18, 10))
    for v in straw.data.vertices:                                     # loose straw: the same on a far version
        if v.co.z > 0.004:
            v.co.z += 0.006 * noise_at(v.co.x * 14, v.co.y * 14, 0.3)
    still = G.join_all('strawberries')
    crowns = spiral(7, 0.32, rng)
    berries = spiral(16, 0.4, rng)
    flowers = spiral(4, 0.3, rng)

    def leaves():
        # Each plant a dense rosette round its crown: leaves of three toothed leaflets spreading on stalks, two standing
        # up in the middle, over a dark heart, so it closes even flattened to a seedling.
        for i, (x, y, a) in enumerate(crowns):
            if not LOW[0]:                                             # a far version: hidden under the leaves
                lump(f'heart{i}', 0.04, (x, y, 0.03), P.leaf, scale=(1, 1, 0.8), noise=0.2, seed=i * 1.7, segments=6, rings=4)
            for j in range(4):
                yaw = a + j * 90
                end = Vector((x, y, 0.045)) + Vector(polar(0.06, yaw, 0.04))
                if LOW[0] and j % 2:                                   # a far version: two leaves a plant, bigger
                    continue
                rays(f'crown{i}_{j}', end, low(0.065, 0.08), 3, [P.leaf, P.leaf_dark, P.leaf_light][(i + j) % 3], yaw + 90, 20,
                     inner=0.12, cup=0.2, droop=0.12, width=0.95, thick=0.004)
            for j in range(low(2, 0)):
                yaw = a + 36 + j * 180
                rays(f'inner{i}_{j}', Vector((x, y, 0.07)) + Vector(polar(0.02, yaw, 0.03)), 0.05, 3, P.leaf_light, yaw + 90, 55,
                     inner=0.12, cup=0.3, width=0.95, thick=0.004)

    def crop():
        for i, (x, y, a) in enumerate(berries):
            G.cone(f'berry{i}', 0.006, 0.024, 0.05, at=(x, y, 0.07), material=P.bloom, verts=low(6, 4))
            if not LOW[0]:
                rays(f'cap{i}', (x, y, 0.094), 0.02, 3, P.stalk, i * 30, 0, inner=0.2, cup=0.1, width=0.6, thick=0.003)
        for i, (x, y, a) in enumerate(flowers):
            stem(f'flower_stem{i}', (x, y, 0.04), (x, y, 0.2), P.stalk, r=0.003)   # plain, as the berries
            rays(f'flower{i}', (x, y, 0.2), 0.024, 5, P.bloom5, 0, 10, inner=0.3, cup=0.12, width=1.1, centre=0.005)
    return _crop(still, (0, 0, 0.05), leaves, crop, sink=(0, 0.04, 0.12), size=(0.88, 0.21, 0.88))   # seedlings: flat on the straw


def herbs():
    """Herb planter: a planter of boards on square legs, with a rim, of basil, upright rosemary, chives (some in flower)
    and grey sage; a resident picking herbs takes a few sprigs of basil."""
    P = palette(body='#8A6446', leaf='#6AAE3E', leaf_dark='#5E9E38', leaf_light='#8EA088', stem='#3E5E3A',
                blossom='#4E8A3A', bloom='#B88AD0')
    rng = random.Random(84)
    for y in (-0.21, 0.21):                                            # boards, two to a side
        for k in range(2):
            G.cube(f'board{y}_{k}', (0.84, 0.04, 0.106), at=(0, y, 0.564 + k * 0.11), material=P.body)
    for x in (-0.4, 0.4):
        G.cube(f'board_end{x}', (0.04, 0.38, 0.216), at=(x, 0, 0.62), material=P.body)
    G.cube('bottom', (0.76, 0.38, 0.02), at=(0, 0, 0.52), material=P.body)
    for x in (-0.38, 0.38):
        for y in (-0.19, 0.19):
            G.cube(f'leg{x}{y}', (0.055, 0.055, 0.52), at=(x, y, 0.26), material=P.body, bevel=low(0.006, 0), bevel_segments=1)
    for x in (-0.36, 0.36):                                            # stretchers between the legs
        G.cube(f'stretcher{x}', (0.03, 0.33, 0.04), at=(x, 0, 0.12), material=P.body)
    G.cube('soil', (0.76, 0.38, 0.02), at=(0, 0, 0.715), material=P.soil)
    lump('basil2', 0.07, (-0.25, -0.1, 0.71), P.leaf_dark, scale=(1, 1, 0.75), noise=0.35, seed=1, segments=7, rings=5)
    r2 = random.Random(184)
    for i in range(8):                                                 # basil leaves on it
        a = i * 45
        yaw, t = a + r2.uniform(-15, 15), r2.uniform(10, 40)
        if LOW[0] and i % 3:
            continue
        cut_leaf(f'basil2_leaf{i}', Vector((-0.25, -0.1, 0.74)) + Vector(polar(0.045, a, 0.02)), yaw, t, 0.07,
                 outline('oval', 0.045, n=3), P.leaf_dark, droop=0.4, fold=0.5, ribs=2)
    for i in range(7):                                                                            # rosemary
        base = (-0.06 + rng.uniform(-0.05, 0.05), rng.uniform(-0.1, 0.1), 0.72)
        yaw, t, length = rng.uniform(0, 360), rng.uniform(72, 90), rng.uniform(0.2, 0.28)
        cut_leaf(f'rosemary{i}', base, yaw, t, length, pinnate(7, 0.045, 'strap', lean=0.9, fill=0.25), P.stem, droop=0.08,
                 fold=0.1, ribs=3)
    for i in range(6):                                                                            # chives, some in flower
        base = (0.14 + rng.uniform(-0.04, 0.04), rng.uniform(-0.1, 0.1), 0.72)
        h = rng.uniform(0.2, 0.26)
        yaw, t = rng.uniform(0, 360), rng.uniform(78, 90)
        for j in range(low(3, 1)):
            d = Vector(polar(math.cos(math.radians(t)) * (1 + j * 0.4), yaw + j * 120, math.sin(math.radians(t))))
            twig(f'chive{i}_{j}', [base, Vector(base) + d * h * (0.9 - j * 0.1)], 0.004, 0.0015, P.blossom, verts=3, res=1)
        if i % 2 == 0:
            florets(f'chive_flower{i}', 0.02, (base[0], base[1], base[2] + h - 0.01), P.bloom, bumps=3.0, amount=0.25,
                    segments=low(6, 4), rings=low(5, 3), seed=i)
    lump('sage', 0.085, (0.31, 0.04, 0.71), P.leaf_light, scale=(1, 1, 0.65), noise=0.3, seed=3, segments=7, rings=5)   # sage
    lump('sage2', 0.065, (0.3, -0.12, 0.71), P.leaf_light, scale=(1, 1, 0.7), noise=0.3, seed=4, segments=6, rings=4)
    for i in range(9):
        a = i * 40
        yaw, t = a + r2.uniform(-15, 15), r2.uniform(15, 45)
        if LOW[0] and i % 3:
            continue
        at = Vector((0.31, 0.04, 0.72) if i < 6 else (0.3, -0.12, 0.72)) + Vector(polar(0.05, a, 0.03))
        cut_leaf(f'sage_leaf{i}', at, yaw, t, 0.08, outline('lance', 0.035, n=3), P.leaf_light, droop=0.2, fold=0.2, ribs=2,
                 thick=0.005)
    at = (-0.29, 0.05, 0.72)

    def basil():
        lump('basil', 0.085, at, P.leaf, scale=(1, 1, 0.75), noise=0.35, seed=2, segments=7, rings=5)
        for i in range(low(8, 3)):
            a = i * 45 + 20
            cut_leaf(f'basil_leaf{i}', Vector(at) + Vector(polar(0.05, a, 0.04)), a, 25 + (i % 3) * 10, 0.075,
                     outline('oval', 0.05, n=3), P.leaf, droop=0.4, fold=0.5, ribs=2)
    return _fit_with_part('sprigs', 'herbs', basil, at, 'harvest', (0.84, 1.01, 0.46))


def lobed(lobes, depth=0.35):
    """A palmate leaf's right half, as a width profile: `lobes` rounded lobes (on both halves together), cut in by
    `depth`."""
    return lambda t: math.sin(math.pi * min(1.0, 0.15 + t * 0.85)) ** 0.6 * (1 - depth * abs(math.sin(math.pi * lobes / 2 * t)) ** 3)


def pumpkins():
    """Pumpkin patch: a patch of soil, vines with curling tendrils, big lobed leaves on long stalks and a yellow flower,
    and three ribbed pumpkins on their stalks, grown from seedlings to ripe."""
    P = palette(bed='#6A4C38', leaf='#4A7A36', leaf_dark='#5A8A3C', leaf_light='#6A9A44', stem='#7AA84C', bloom='#E8822A',
                stalk='#5E6A2E', blossom='#F2C12E')
    rng = random.Random(85)
    G.revolve('bed', [(0, 0), (0.46, 0), (0.42, 0.02), (0.3, 0.035), (0, 0.04)], material=P.bed, segments=low(20, 12),
              scale=(1.1, 1.0, 1))
    still = G.join_all('pumpkins')
    spots = []
    for i in range(9):
        a = i / 9 * 360 + rng.uniform(-15, 15)
        rr = rng.uniform(0.12, 0.34)
        spots.append((polar(rr, a, 0.03), a, rng.uniform(5, 20)))
    cols = [P.leaf, P.leaf_dark, P.leaf_light, P.stem]

    def leaves():
        for i in range(3):                                              # the vines, wandering over the soil
            a = i * 120 + 30
            pts = [Vector(polar(0.02 + 0.13 * k, a + 25 * math.sin(k * 1.3 + i), 0.045)) for k in range(4)]
            twig(f'vine{i}', pts, 0.008, 0.005, P.stem, verts=4)
            if not LOW[0]:                                              # a tendril curling off its end
                end = pts[-1]
                curl = [end + Vector((0.03 * math.cos(t * 2.5), 0.03 * math.sin(t * 2.5), 0.03 * t)) * (1 - t * 0.5)
                        for t in (k / 6 for k in range(7))]
                G.sweep(f'tendril{i}', curl, 0.002, material=P.stem, verts=3)
        for i, (base, a, tilt) in enumerate(spots):
            end = Vector(base) + Vector(polar(0.05, a, 0.1))
            stalk(f'petiole{i}', base, end, P.stem, r=0.005, bow=0.02)
            cut_leaf(f'leaf{i}', end, a, tilt, 0.24, outline(lobed(5), 0.27, n=10, teeth=0.05), cols[i % 4], droop=0.22,
                     fold=0.3, ribs=4, thick=0.008)
        cupped('flower', [(0, 0), (0.01, 0.0), (0.02, 0.04), (0.045, 0.07), (0.035, 0.07)], (-0.05, -0.3, 0.04), P.blossom, 200, 55,
               segments=low(6, 3))

    def crop():
        for i, (x, y, r) in enumerate([(0.18, -0.12, 0.17), (-0.2, 0.05, 0.14), (-0.02, 0.3, 0.11)]):
            n = low(18, 8)
            secs = [[(x + u, y + v, 0.02 + z) for u, v in ribbed(rr, n // 2, 0.1, verts=n)] for z, rr in
                    [(0.0, r * 0.3), (r * 0.12, r * 0.75), (r * 0.45, r), (r * 0.85, r * 0.9), (r * 1.08, r * 0.55), (r * 1.15, r * 0.18)]]
            G.loft(f'pumpkin{i}', secs, material=P.bloom)
            twig(f'stalk{i}', [(x, y, 0.02 + r * 1.1), (x + 0.008, y, 0.02 + r * 1.1 + 0.04), (x + 0.025, y, 0.02 + r * 1.1 + 0.06)],
                 0.013, 0.009, P.stalk, verts=5, res=1)
    return _crop(still, (0, 0, 0.04), leaves, crop, size=(1.112, 0.269, 0.947))


# (width, height, depth) as measured; each within 10 % of the old part's bounds (`cargo check-glb` warns beyond).
TOLERANCE = 0.03
BED = [('bed', 'Bed')]

FAMILY = G.Family(
    'plants', M,
    [G.Model('plant', plant, size=(0.772, 0.825, 0.791), key='model.plant', tolerance=TOLERANCE, paint=[('body', 'Pot')]),
     G.Model('monstera', monstera, size=(0.784, 0.911, 0.791), key='model.garden.monstera', tolerance=TOLERANCE, paint=[('body', 'Pot')]),
     G.Model('fiddleFig', fiddle_fig, size=(0.528, 1.626, 0.527), key='model.garden.fiddleFig', tolerance=TOLERANCE, paint=[('body', 'Pot')]),
     G.Model('snakePlant', snake_plant, size=(0.341, 0.905, 0.32), key='model.garden.snakePlant', tolerance=TOLERANCE, paint=[('body', 'Pot')]),
     G.Model('cacti', cacti, size=(0.568, 0.521, 0.465), key='model.garden.cacti', tolerance=TOLERANCE, paint=[('body', 'Pot')]),
     G.Model('succulents', succulents, size=(0.62, 0.17, 0.62), key='model.garden.succulents', tolerance=TOLERANCE, paint=[('body', 'Pot')]),
     G.Model('bostonFern', boston_fern, size=(0.734, 0.979, 0.792), key='model.garden.bostonFern', tolerance=TOLERANCE, paint=[('body', 'Pot')]),
     G.Model('palm', palm, size=(0.770, 0.893, 0.693), key='model.garden.palm', tolerance=TOLERANCE, paint=[('body', 'Pot')]),
     G.Model('orchid', orchid, size=(0.431, 1.198, 0.381), key='model.garden.orchid', tolerance=TOLERANCE, paint=[('body', 'Pot')]),
     G.Model('bonsai', bonsai, size=(0.62, 1.354, 0.407), key='model.garden.bonsai', tolerance=TOLERANCE, paint=[('body', 'Pot')],
             motions={'uses': {'prune': 'trim'}}),
     # G2: flowers (with G3 and G4, each with a far version: they stand outside, seen in the town overview)
     G.Model('flowerbed', flowerbed, size=(0.94, 0.475, 0.94), key='model.flowerbed', tolerance=TOLERANCE, far=far_of(flowerbed), paint=[('wood', 'Box')]),
     G.Model('tulips', tulips, size=(0.84, 0.622, 0.843), key='model.garden.tulips', tolerance=TOLERANCE, far=far_of(tulips), paint=BED),
     G.Model('daffodils', daffodils, size=(0.84, 0.491, 0.84), key='model.garden.daffodils', tolerance=TOLERANCE, far=far_of(daffodils), paint=BED + [('bloom', 'Petals')]),
     G.Model('daisies', daisies, size=(0.84, 0.376, 0.84), key='model.garden.daisies', tolerance=TOLERANCE, far=far_of(daisies), paint=BED + [('bloom', 'Petals')]),
     G.Model('sunflowers', sunflowers, size=(0.84, 2.108, 0.907), key='model.garden.sunflowers', tolerance=TOLERANCE, far=far_of(sunflowers), paint=BED + [('bloom', 'Petals')]),
     G.Model('poppies', poppies, size=(0.84, 0.657, 0.84), key='model.garden.poppies', tolerance=TOLERANCE, far=far_of(poppies), paint=BED + [('bloom', 'Petals')]),
     G.Model('marigolds', marigolds, size=(0.885, 0.34, 0.84), key='model.garden.marigolds', tolerance=TOLERANCE, far=far_of(marigolds), paint=BED),
     G.Model('pansies', pansies, size=(0.84, 0.264, 0.84), key='model.garden.pansies', tolerance=TOLERANCE, far=far_of(pansies), paint=BED),
     G.Model('lilies', lilies, size=(0.84, 0.939, 0.869), key='model.garden.lilies', tolerance=TOLERANCE, far=far_of(lilies), paint=BED),
     G.Model('lupins', lupins, size=(0.84, 0.987, 0.84), key='model.garden.lupins', tolerance=TOLERANCE, far=far_of(lupins), paint=BED),
     G.Model('forgetMeNots', forget_me_nots, size=(0.866, 0.28, 0.84), key='model.garden.forgetMeNots', tolerance=TOLERANCE, far=far_of(forget_me_nots), paint=BED),
     G.Model('cosmos', cosmos, size=(0.885, 0.879, 0.84), key='model.garden.cosmos', tolerance=TOLERANCE, far=far_of(cosmos), paint=BED),
     G.Model('meadow', meadow, size=(0.872, 0.575, 0.91), key='model.garden.meadow', tolerance=TOLERANCE, far=far_of(meadow),
             motions={'uses': {'pickFlower': 'pick'}}),
     G.Model('geraniums', geraniums, size=(0.424, 0.65, 0.468), key='model.garden.geraniums', tolerance=TOLERANCE, far=far_of(geraniums), paint=[('body', 'Pot'), ('bloom', 'Flowers')]),
     G.Model('flowerBox', flower_box, size=(0.945, 0.732, 0.505), key='model.garden.flowerBox', tolerance=TOLERANCE, far=far_of(flower_box), paint=[('body', 'Box')]),
     # G3: shrubs (they dress every lot)
     G.Model('bush', bush, size=(1.043, 0.91, 1.095), key='model.bush', tolerance=TOLERANCE, far=far_of(bush), habitat='bush'),
     G.Model('bushSmall', bush_small, size=(0.832, 0.707, 0.782), key='model.bush.small', tolerance=TOLERANCE, far=far_of(bush_small), habitat='bush'),
     G.Model('bushFlowering', bush_flowering, size=(1.208, 1.016, 1.268), key='model.bush.flowering', tolerance=TOLERANCE,
             far=far_of(bush_flowering), habitat='bush'),
     G.Model('streetHedge', street_hedge, size=(1.547, 1.019, 0.85), key='model.hedge', tolerance=TOLERANCE, far=far_of(street_hedge), habitat='hedge'),
     G.Model('boxwood', boxwood, size=(0.696, 0.752, 0.722), key='model.garden.boxwood', tolerance=TOLERANCE, far=far_of(boxwood), habitat='bush',
             motions={'uses': {'trim': 'trim'}}),
     G.Model('topiaryCone', topiary_cone, size=(0.643, 1.805, 0.66), key='model.garden.topiaryCone', tolerance=TOLERANCE, far=far_of(topiary_cone), habitat='bush',
             paint=[('body', 'Pot')], motions={'uses': {'trim': 'trim'}}),
     G.Model('hedge', garden_hedge, size=(1.546, 1.167, 0.92), key='model.garden.hedge', tolerance=TOLERANCE, far=far_of(garden_hedge), habitat='hedge',
             motions={'uses': {'trim': 'trim'}}),
     G.Model('hydrangea', hydrangea, size=(1.196, 0.957, 1.182), key='model.garden.hydrangea', tolerance=TOLERANCE, far=far_of(hydrangea), habitat='bush',
             paint=[('blossom', 'Flowers')]),
     G.Model('rosebush', rosebush, size=(0.771, 0.903, 0.722), key='model.garden.rosebush', tolerance=TOLERANCE, far=far_of(rosebush), habitat='bush',
             paint=[('bloom', 'Roses')], motions={'uses': {'prune': 'cut'}}),
     G.Model('azalea', azalea, size=(1.158, 1.013, 1.152), key='model.garden.azalea', tolerance=TOLERANCE, far=far_of(azalea), habitat='bush',
             paint=[('blossom', 'Flowers')], motions={'uses': {'trim': 'trim'}}),
     G.Model('lavender', lavender, size=(0.607, 0.865, 0.63), key='model.garden.lavender', tolerance=TOLERANCE, far=far_of(lavender),
             paint=[('bloom', 'Flowers')], motions={'uses': {'cut': 'cut'}}),
     G.Model('fern', garden_fern, size=(0.793, 0.478, 0.932), key='model.garden.fern', tolerance=TOLERANCE, far=far_of(garden_fern)),
     G.Model('grass', ornamental_grass, size=(1.218, 1.21, 1.28), key='model.garden.grass', tolerance=TOLERANCE, far=far_of(ornamental_grass)),
     G.Model('hosta', hosta, size=(0.621, 0.842, 0.675), key='model.garden.hosta', tolerance=TOLERANCE, far=far_of(hosta)),
     # G4: edibles
     G.Model('vegPatch', veg_patch, size=(0.92, 0.398, 0.92), key='model.garden.vegPatch', tolerance=TOLERANCE, far=far_of(veg_patch),
             paint=[('body', 'Bed')], motions={'growth': 'grow'}),
     G.Model('tomatoes', tomatoes, size=(0.8, 1.19, 0.8), key='model.garden.tomatoes', tolerance=TOLERANCE, far=far_of(tomatoes),
             paint=BED, motions={'growth': 'grow'}),
     G.Model('strawberries', strawberries, size=(0.88, 0.21, 0.88), key='model.garden.strawberries', tolerance=TOLERANCE,
             far=far_of(strawberries), paint=BED, motions={'growth': 'grow'}),
     G.Model('herbs', herbs, size=(0.84, 1.01, 0.46), key='model.garden.herbs', tolerance=TOLERANCE, far=far_of(herbs),
             paint=[('body', 'Bed')], motions={'uses': {'pickHerbs': 'harvest'}}),
     G.Model('pumpkins', pumpkins, size=(1.112, 0.269, 0.947), key='model.garden.pumpkins', tolerance=TOLERANCE, far=far_of(pumpkins),
             paint=BED, motions={'growth': 'grow'})],
    comment='Blender-built plants (tools/art/blender/families/plants.py): the G1 houseplants, G2 flowers, '
            'G3 shrubs and G4 edibles. Listed in the base manifest\'s packs. Leaves and stems are on the '
            '`foliage` finish, so the nature shader sways and tints them. The outdoor ones have a far '
            'version (`far`). Clips: the bonsai\'s sprig and the clipped shrubs\' sprigs shrink away while '
            'they are trimmed, a rose and lavender stems while they are cut, a meadow flower and basil '
            'while they are picked; the crops grow from seedlings to ripe and lose their crop when '
            'harvested (`motions.growth`).'
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

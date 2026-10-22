# Perches: where the game's birds may sit on a model (a tree's outer twigs, a hedge's top), the pack entry's
# `perches`: [[x, y, z], ...] in the model's own game axes (x, height, depth: Blender (-x, z, -y)), metres, at rest.
#
# A build finds them on its own geometry (`surface`: a line in from outside to where it meets the leaves, a little
# way in) and keeps a spread of them (`spread`), then `keep`s them; `G.settle` moves them with the model and the
# family writes them into the entry. They are data only: finding them casts rays at the built meshes and draws
# no random number of the build's (each build that has perches gives its own `random.Random`), so the meshes are
# what they were. A far version (`G.is_low()`) has none.
import math

import bpy
from mathutils import Vector

from .far import LOW

# The perches of the model being built (Blender points), in the order kept.
PERCHES = []


def clear():
    PERCHES.clear()


def keep(points):
    """Keeps `points` (Blender metres) as the model's perches; nothing while a far version is built."""
    if not LOW[0]:
        PERCHES.extend(Vector(p) for p in points)


def shift(by):
    """Moves the perches with the model (`G.settle`)."""
    for p in PERCHES:
        p += by


def game():
    """The perches as the pack writes them: game (x, height, depth), to the millimetre."""
    return [[round(-p.x, 3) + 0.0, round(p.z, 3) + 0.0, round(-p.y, 3) + 0.0] for p in PERCHES]


def _scene():
    for o in bpy.context.scene.objects:
        if o.type == 'MESH':
            o.data.update()
    depsgraph = bpy.context.evaluated_depsgraph_get()
    depsgraph.update()
    return bpy.context.scene, depsgraph


def _cast(scene, depsgraph, origin, direction, distance):
    hit, at, normal = scene.ray_cast(depsgraph, origin, direction, distance=distance)[:3]
    return (at, normal) if hit else None


def surface(aims, inset, reach=8.0, headroom=0.25, facing=0.2):
    """
    Where lines in from outside meet the model as it stands now: for each (target, direction) in `aims` (Blender
    points and unit vectors), the line from `reach` metres out along `direction` back in towards `target` meets
    its first face. A perch is `inset` metres further in from there (a bird's feet among the outer twigs, its body
    showing), kept only where that face looks out (towards the line) and not down (its normal's z over `facing`)
    and nothing hangs over it within `headroom` metres (a clump above it, a tier's rim). Returns
    [(perch, surface point)].
    """
    scene, depsgraph = _scene()
    out = []
    up = Vector((0, 0, 1))
    for target, d in aims:
        target, d = Vector(target), Vector(d).normalized()
        hit = _cast(scene, depsgraph, target + d * reach, -d, reach * 2)
        if not hit:
            continue
        at, normal = hit
        if normal.dot(d) <= 0.05 or normal.z < facing:
            continue
        if headroom and _cast(scene, depsgraph, at + normal * 0.02 + up * 0.01, up, headroom):
            continue
        out.append((at - d * inset, at))
    return out


def bare(points, headroom=0.3):
    """Which of `points` (on a bare limb's top, Blender metres) stand in the open: nothing over them within
    `headroom` and not inside the leaves (the first face straight up from them doesn't face up: that is a clump's
    inside)."""
    scene, depsgraph = _scene()
    up = Vector((0, 0, 1))
    out = []
    for p in points:
        p = Vector(p)
        hit = _cast(scene, depsgraph, p + up * 0.01, up, 30.0)
        if hit and (hit[1].z > 0 or (hit[0] - p).length < headroom):
            continue
        out.append(p)
    return out


def spread(points, n, gap=0.0):
    """Up to `n` of `points` spread round the model: the highest first, then each the farthest from those kept
    (none nearer than `gap` metres to another). Deterministic."""
    pts = [Vector(p) for p in points]
    if not pts:
        return []
    kept = [max(pts, key=lambda p: (p.z, -p.x, -p.y))]
    near = [(p - kept[0]).length for p in pts]
    while len(kept) < n:
        i = max(range(len(pts)), key=lambda k: near[k])
        if near[i] <= max(gap, 1e-6):
            break
        kept.append(pts[i])
        near = [min(near[k], (pts[k] - pts[i]).length) for k in range(len(pts))]
    return kept


def around(centre, rng, n, elevation=(0.15, 0.9), heading=None, turn=math.pi):
    """`n` lines in towards `centre` from round it, `elevation` radians up from level (drawn from `rng`, the
    perches' own), each heading `heading` +- `turn` (default anywhere)."""
    aims = []
    for _ in range(n):
        a = (heading if heading is not None else 0.0) + rng.uniform(-turn, turn)
        e = rng.uniform(*elevation)
        aims.append((Vector(centre), Vector((math.cos(a) * math.cos(e), math.sin(a) * math.cos(e), math.sin(e)))))
    return aims

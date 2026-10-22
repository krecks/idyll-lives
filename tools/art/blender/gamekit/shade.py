# Shading baked into the vertices: how much of the sky each vertex sees (ambient occlusion), written as a grey vertex
# colour (`ao`, glTF COLOR_0) that the game multiplies into the part's colour. Creases, seams, the gaps between
# cushions, drawer reveals and undersides darken; open faces stay as they are. No UVs, no texture, no draw call.
#
# A part that moves away from where it rests (a door, a drawer, something that grows or is hidden) is shaded only by
# itself, so it isn't left with the shadow of what it moved away from; everything else (the still part, a cushion
# that only sinks) is shaded by all of those together, at rest.
import math

import bpy
from mathutils import Matrix, Vector
from mathutils.bvhtree import BVHTree

from . import motion
from .parts import measure

ATTRIBUTE = 'ao'
SAMPLES = 48
STRENGTH = 0.8      # how dark a fully enclosed vertex goes: 1 - STRENGTH
LIFT = 0.0015       # rays start this far off the surface (m), so they don't hit their own face
LEVELS = 32         # greys it is rounded to: neighbours share a value, which the build's compression packs tightly


def _directions(n):
    """`n` directions over the hemisphere about +Z, cosine-weighted (more towards the top), evenly spread (a
    Fibonacci spiral): the same every build."""
    out = []
    golden = math.pi * (3 - math.sqrt(5))
    for i in range(n):
        r = math.sqrt((i + 0.5) / n)
        a = golden * i
        out.append(Vector((r * math.cos(a), r * math.sin(a), math.sqrt(max(0.0, 1 - r * r)))))
    return out


DIRECTIONS = _directions(SAMPLES)


def _moves_away(obj):
    """Whether a clip takes `obj` away from its rest: moved over 3 cm, turned over 8 degrees, or scaled to under half
    or over twice (grown from nothing, hidden while held). A cushion squashed a little stays."""
    if not (obj.animation_data and obj.animation_data.nla_tracks):
        return False
    scene = bpy.context.scene
    motion.solo(None)
    scene.frame_set(1)
    rest = obj.matrix_world.copy()
    away = False
    for o, clip, last in motion.clips([obj]):
        motion.solo(clip)
        for f in range(1, last + 1, max(1, last // 12)):
            scene.frame_set(f)
            m = obj.matrix_world
            if (m.translation - rest.translation).length > 0.03:
                away = True
            if rest.to_quaternion().rotation_difference(m.to_quaternion()).angle > math.radians(8):
                away = True
            ratios = [a / max(b, 1e-9) for a, b in zip(m.to_scale(), rest.to_scale())]
            if min(ratios) < 0.5 or max(ratios) > 2 or min(rest.to_scale()) < 0.01:
                away = True
        motion.solo(None)
        scene.frame_set(1)
        if away:
            break
    return away


def _tree(objs, local):
    """A BVH of `objs`' triangles, in world space at rest, or in the one object's own space (`local`)."""
    verts, polys = [], []
    for o in objs:
        m = Matrix.Identity(4) if local else o.matrix_world
        base = len(verts)
        verts += [m @ v.co for v in o.data.vertices]
        polys += [[base + i for i in p.vertices] for p in o.data.polygons]
    return BVHTree.FromPolygons(verts, polys, all_triangles=False, epsilon=0.0)


def _occlusion(obj, tree, local, reach):
    """Per vertex of `obj`: the share of the sky it sees, 0 (enclosed) to 1 (open), from rays out over the
    hemisphere about its normal, a hit counting less the farther away it is (up to `reach`)."""
    m = Matrix.Identity(4) if local else obj.matrix_world
    nm = m.to_3x3().inverted().transposed()
    seen = []
    for v in obj.data.vertices:
        p = m @ v.co
        n = (nm @ v.normal).normalized()
        # a turn taking +Z onto the normal, twisted by the vertex's index so neighbours don't band
        turn = Vector((0, 0, 1)).rotation_difference(n).to_matrix() @ Matrix.Rotation(v.index * 2.399963, 3, 'Z')
        start = p + n * LIFT
        blocked, inside = 0.0, 0
        for d in DIRECTIONS:
            ray = turn @ d
            hit, face, _, dist = tree.ray_cast(start, ray, reach)
            if hit is None:
                continue
            if face.dot(ray) > 0:    # the back of a face: this vertex is inside another part, not under it
                inside += 1
                continue
            blocked += 1 - (dist / reach) ** 2
        # A vertex buried in another part (a panel's corner inside its frame, mirror glass in its rim) is hidden;
        # shaded, it would darken the whole face it shares, so it stays open.
        seen.append(1.0 if inside > len(DIRECTIONS) // 2 else 1 - blocked / len(DIRECTIONS))
    return seen


def bake_ao(objs, reach=None, strength=STRENGTH):
    """Bakes each of `objs`' (the model at rest, clips made) occlusion into its `ao` vertex colour, which the export
    writes as COLOR_0. `reach`: how far a crease is looked for, by default a fifth of the model's largest side
    (4 cm to 40 cm)."""
    scene = bpy.context.scene
    motion.solo(None)
    scene.frame_set(1)
    if reach is None:
        lo, hi = measure(objs)
        reach = min(0.4, max(0.04, 0.2 * max(h - l for l, h in zip(lo, hi))))
    away = {o for o in objs if _moves_away(o)}
    together = [o for o in objs if o not in away]
    shared = _tree(together, local=False) if together else None
    for o in objs:
        local = o in away
        seen = _occlusion(o, _tree([o], True) if local else shared, local, reach)
        mesh = o.data
        attr = mesh.color_attributes.get(ATTRIBUTE) or mesh.color_attributes.new(ATTRIBUTE, 'FLOAT_COLOR', 'POINT')
        for v, s in zip(attr.data, seen):
            a = round((1 - strength * (1 - s)) * LEVELS) / LEVELS
            v.color = (a, a, a, 1.0)
        mesh.color_attributes.active_color = attr
    return objs

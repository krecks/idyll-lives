# Assembling a model from its parts: joining them into one mesh (or one per moving part), putting a
# moving part's origin on its hinge, standing the model on the floor and checking its size.
import contextlib
import math

import bpy
from mathutils import Matrix, Vector

from .scene import meshes

# The (game x, depth) the model being built is centred on: its Model's `centre`, set by Family.make, so
# `settle()` stands a floor model where its placeholder stands (a counter's back on the wall).
CENTRE = (0.0, 0.0)


@contextlib.contextmanager
def new_parts():
    """Collects the meshes made inside the block, in order:

        with G.new_parts() as lid:
            G.cube('lid', ...)
            G.cube('lid_handle', ...)
        G.join(lid, 'lid')
    """
    before = set(meshes())
    made = []
    yield made
    made.extend(o for o in meshes() if o not in before)


def join_all(name):
    """Applies every transform and modifier, joins every mesh in the scene, returns the one object.
    Its origin is the world origin: for a model with no moving parts."""
    return join(meshes(), name, origin=(0, 0, 0))


def join(parts, name, origin=None):
    """
    Joins `parts` into one object named `name`, modifiers, rotation and scale applied, so the object is
    only placed (a clip turns it from rest). It keeps the first part's origin, or takes `origin` (a
    Blender point) if given; the mesh stays where it is either way.
    """
    first = parts[0]
    bpy.ops.object.select_all(action='DESELECT')
    for o in parts:
        o.select_set(True)
    bpy.context.view_layer.objects.active = first
    bpy.ops.object.convert(target='MESH')
    if len(parts) > 1:
        bpy.ops.object.join()
    obj = bpy.context.view_layer.objects.active
    bpy.ops.object.transform_apply(location=False, rotation=True, scale=True)
    obj.name = name
    obj.data.name = name
    if origin is not None:
        set_origin(obj, origin)
    return obj


def set_origin(obj, point):
    """Moves `obj`'s origin to `point` (Blender world, metres) without moving its mesh."""
    local = obj.matrix_world.inverted() @ Vector(point)
    obj.data.transform(Matrix.Translation(-local))
    obj.matrix_world = obj.matrix_world @ Matrix.Translation(local)
    return obj


def hinged(obj, pivot):
    """A moving part turning about `pivot` (Blender world): its origin goes there, so a clip's rotation
    swings it about the hinge (docs/design/objects.md, "Moving parts"). For a slide, the pivot is where
    it slides from; any point of the part will do."""
    return set_origin(obj, pivot)


def turn(obj, degrees, axis='X'):
    """Turns `obj` about its own origin by `degrees` and bakes the turn into its mesh, so the object is
    unrotated. For a part built in its open pose: build it open, `hinged` it, `turn` it to rest (closed),
    then `swing` it back open by the opposite angle."""
    obj.data.transform(Matrix.Rotation(math.radians(degrees), 4, axis))
    obj.data.update()
    return obj


def measure(objs=None):
    """(lo, hi): the Blender world bounds of `objs` (default: every mesh), modifiers included."""
    objs = objs if objs is not None else meshes()
    depsgraph = bpy.context.evaluated_depsgraph_get()
    lo, hi = [float('inf')] * 3, [float('-inf')] * 3
    for o in objs:
        ev = o.evaluated_get(depsgraph)
        mesh = ev.to_mesh()
        for v in mesh.vertices:
            p = ev.matrix_world @ v.co
            lo = [min(lo[i], p[i]) for i in range(3)]
            hi = [max(hi[i], p[i]) for i in range(3)]
        ev.to_mesh_clear()
    return lo, hi


def game_size(objs=None):
    """(width, height, depth) in metres, the game's way round."""
    lo, hi = measure(objs)
    return hi[0] - lo[0], hi[2] - lo[2], hi[1] - lo[1]


def settle(objs=None, centre=True, floor=True):
    """Stands the model on the floor (lowest point at z = 0) and centres it on the footprint in x and y,
    or on its Model's `centre` (game x, depth), by moving it (never scaling). `centre=False` only stands
    it on the floor: for a model built in place off the footprint's centre (a wall bed at the back of its
    tiles). `floor=False` leaves its height
    alone, for a thing on a wall (docs/design/objects.md, "The origin"), built at its real height. Call
    it before adding clips: they key the rest pose where it is. A moving part moves whole, its hinge
    with it."""
    objs = objs if objs is not None else meshes()
    if any(o.animation_data and o.animation_data.nla_tracks for o in objs):
        raise RuntimeError('settle() before swing()/slide(): a clip keys the rest pose where it is')
    lo, hi = measure(objs)
    drop = -lo[2] if floor else 0.0
    if centre:
        shift = Vector((-CENTRE[0] - (lo[0] + hi[0]) / 2, -CENTRE[1] - (lo[1] + hi[1]) / 2, drop))
    else:
        shift = Vector((0, 0, drop))
    for o in objs:
        o.location += shift
    return objs


def check_size(size, tolerance=0.02, objs=None, name='model', centre=(0.0, 0.0), floor=True):
    """
    Asserts the model is `size` (game width, height, depth, metres) within `tolerance` (a fraction, or
    one per axis), on the floor, and centred on its footprint, or on `centre` (game x, depth) for a model
    that stands off the middle of its tiles. `floor=False` skips the floor: for a thing on a wall, built
    at its real height. Returns the measured size.
    """
    lo, hi = measure(objs)
    got = game_size(objs)
    tol = tolerance if isinstance(tolerance, (tuple, list)) else (tolerance,) * 3
    for axis, g, want, t in zip(('width', 'height', 'depth'), got, size, tol):
        assert abs(g - want) <= t * want, f'{name}: {axis} {g:.3f} m, want {want:.3f} m (±{t * 100:.0f} %)'
    want = (-centre[0], -centre[1])  # game (x, depth) in Blender x, y
    for i, axis in enumerate('xy'):
        c = (lo[i] + hi[i]) / 2 - want[i]
        assert abs(c) <= 0.002, f'{name}: off its centre in Blender {axis} by {c * 100:.1f} cm (G.settle)'
    if floor:
        assert abs(lo[2]) <= 0.002, f'{name}: bottom {lo[2] * 100:.1f} cm off the floor (G.settle)'
    where = 'centred' if centre == (0.0, 0.0) else f'centred on game ({centre[0]:g}, {centre[1]:g})'
    print(f'[gamekit] {name} {got[0]:.3f} x {got[1]:.3f} x {got[2]:.3f} m (w x h x d), {where}, on the floor')
    return got


def fit(obj, w, h, d):
    """
    Scales `obj` into an exact w × h × d box (game metres), bottom on z = 0, centred. It stretches each
    axis on its own, so it is only for third-party files; a model built here is made to its real size
    and checked with `check_size` (docs/design/objects.md, "Real size, uniform scale").
    """
    ws = [obj.matrix_world @ v.co for v in obj.data.vertices]
    mn = [min(v[i] for v in ws) for i in range(3)]
    mx = [max(v[i] for v in ws) for i in range(3)]
    ext = [mx[i] - mn[i] for i in range(3)]
    s = (w / ext[0], d / ext[1], h / ext[2])
    for v in obj.data.vertices:
        v.co.x = (v.co.x - (mn[0] + mx[0]) / 2) * s[0]
        v.co.y = (v.co.y - (mn[1] + mx[1]) / 2) * s[1]
        v.co.z = (v.co.z - mn[2]) * s[2]
    obj.location = (0, 0, 0)
    obj.scale = (1, 1, 1)
    obj.data.update()
    return obj

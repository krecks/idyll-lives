# Primitives: every builder adds one mesh object centred on `at` (Blender axes, metres) with its
# material, and returns it. Sizes are full extents (a cube of size 0.4 is 0.4 m across), matching the
# game's `box`/`cylinder` parts. Sizes and scales go into the mesh itself, so a `bevel` is in metres on
# every side. `rot` is in degrees.
#
# Shading: a smooth part is smooth across its edges up to `SHARP` degrees and sharp past them (a cylinder's
# rim, a box's corners), so a cap stays flat instead of shading into the side. Something round with few sides
# (a 6-sided leg) stays smooth round its sides (`round_angle`). A `bevel` is shaded smooth with hardened
# normals: the faces either side stay flat and the bevel rounds the edge between them, for the same triangles.
import math

import bpy
from mathutils import Matrix, Vector
from mathutils import noise as vnoise

from . import uv
from .scene import TAU

SHARP = 40  # degrees: an edge turning more than this stays sharp on a smooth part (a profile's 45° chamfer stays crisp)


CLOSED = 180  # a closed round thing (a sphere, a ring, a boulder) has no edge to keep sharp: smooth all over


def round_angle(sides):
    """The sharp angle for a smooth thing `sides` round: its sides stay smooth (a hexagonal leg turns 60° a
    side), its caps' rims (90°) stay sharp."""
    return min(80.0, max(SHARP, 360.0 / max(sides, 1) + 10.0))


def shade(obj, smooth=True, angle=SHARP):
    """Shades `obj` smooth, its edges turning more than `angle` degrees kept sharp, or (`smooth` False) flat."""
    if smooth:
        obj.data.shade_smooth()
        obj.data.set_sharp_from_angle(angle=math.radians(angle))
    else:
        obj.data.shade_flat()
    return obj


def bevel_edges(obj, width, segments=2, angle=40):
    """Rounds `obj`'s edges turning more than `angle` degrees by `width` (metres): a bevel modifier, shaded smooth
    with hardened normals, so the faces it joins stay flat and the bevel reads as a rounded edge, not a facet."""
    if not obj.data.polygons or not obj.data.polygons[0].use_smooth:
        shade(obj, True, SHARP)
    m = obj.modifiers.new('bevel', 'BEVEL')
    m.width = width
    m.segments = segments
    m.limit_method = 'ANGLE'
    m.angle_limit = math.radians(angle)
    m.harden_normals = True   # (with miter_outer 'MITER_ARC', a frame's inner corners got normals turned round)
    return m


def cube(name, size, at=(0, 0, 0), material=None, rot=(0, 0, 0), smooth=False, bevel=0.0, bevel_segments=2):
    """A box of `size` (x, y, z) or one number for all three."""
    bpy.ops.mesh.primitive_cube_add(size=1.0)
    _scale_mesh(size if isinstance(size, (tuple, list)) else (size, size, size))
    return _finish(name, at, rot, material, smooth, bevel, bevel_segments)


def sphere(name, radius, at=(0, 0, 0), material=None, scale=(1, 1, 1), rot=(0, 0, 0),
           segments=24, rings=16, smooth=True, bevel=0.0, bevel_segments=2):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=segments, ring_count=rings, radius=radius)
    _scale_mesh(scale)
    return _finish(name, at, rot, material, smooth, bevel, bevel_segments, CLOSED)


def cylinder(name, radius, depth, at=(0, 0, 0), material=None, scale=(1, 1, 1), rot=(0, 0, 0),
             verts=24, smooth=True, bevel=0.0, bevel_segments=2):
    """A cylinder along z, `depth` long."""
    bpy.ops.mesh.primitive_cylinder_add(vertices=verts, radius=radius, depth=depth)
    _scale_mesh(scale)
    bpy.context.object['uv'] = uv.ROUND   # (gamekit/uv.py: round about its own z)
    return _finish(name, at, rot, material, smooth, bevel, bevel_segments, round_angle(verts))


def cone(name, r1, r2, depth, at=(0, 0, 0), material=None, scale=(1, 1, 1), rot=(0, 0, 0),
         verts=24, smooth=True, bevel=0.0, bevel_segments=2):
    """A cone or frustum along z, radius `r1` at the bottom and `r2` at the top."""
    bpy.ops.mesh.primitive_cone_add(vertices=verts, radius1=r1, radius2=r2, depth=depth)
    _scale_mesh(scale)
    bpy.context.object['uv'] = uv.ROUND
    return _finish(name, at, rot, material, smooth, bevel, bevel_segments, round_angle(verts))


def torus(name, major_radius, minor_radius, at=(0, 0, 0), material=None, rot=(0, 0, 0),
          major_segments=24, minor_segments=12, smooth=True):
    """A ring lying in the xy plane."""
    bpy.ops.mesh.primitive_torus_add(major_radius=major_radius, minor_radius=minor_radius,
                                     major_segments=major_segments, minor_segments=minor_segments)
    return _finish(name, at, rot, material, smooth, 0.0, 2, CLOSED)


def rod(name, a, b, radius, material=None, verts=12, smooth=True):
    """A cylinder of `radius` from point `a` to point `b` (both in metres)."""
    va, vb = Vector(a), Vector(b)
    d = vb - va
    obj = cylinder(name, radius, d.length, at=tuple((va + vb) / 2), material=material,
                   verts=verts, smooth=smooth)
    obj.rotation_mode = 'QUATERNION'
    obj.rotation_quaternion = d.to_track_quat('Z', 'Y')
    return obj


LATHE_SHARP = 70  # degrees: a lathe's profile is often a few points round a curve (a jar's shoulder turns 53°)


def revolve(name, profile, at=(0, 0, 0), material=None, segments=24, scale=(1, 1, 1),
            rot=(0, 0, 0), smooth=True, angle=LATHE_SHARP):
    """Surface of revolution around z. `profile` is [(r, z), ...] in metres, top last. Smooth, it stays smooth
    where the profile turns less than `angle` degrees (a few points round a jar's shoulder) and sharp where it turns
    more (a rim, a step, the edge of its base)."""
    verts, faces = [], []
    n = len(profile)
    for s in range(segments):
        a = TAU * s / segments
        ca, sa = math.cos(a), math.sin(a)
        for r, z in profile:
            verts.append((r * ca, r * sa, z))
    for s in range(segments):
        s2 = (s + 1) % segments
        for i in range(n - 1):
            a, b = s * n + i, s * n + i + 1
            c, d = s2 * n + i + 1, s2 * n + i
            if abs(profile[i][0]) < 1e-6 and abs(profile[i + 1][0]) < 1e-6:
                continue  # both ends on the axis: the quad collapses
            faces.append((a, b, c, d))
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    if material:
        obj.data.materials.append(material)
    _recalc_normals(obj)
    shade(obj, smooth, max(angle, round_angle(segments)))
    obj['uv'] = uv.ROUND
    obj.location = at
    obj.rotation_euler = tuple(math.radians(a) for a in rot)
    obj.scale = scale
    return obj


def boulder(name, radius, at=(0, 0, 0), material=None, scale=(1, 1, 1), rot=(0, 0, 0),
         seed=0.0, noise=0.22, subdivisions=3, smooth=True):
    """A lumpy, deterministic boulder: an icosphere displaced by value noise, bottom on z = 0."""
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=subdivisions, radius=radius)
    obj = _finish(name, at, rot, material, smooth, 0.0, 2, CLOSED)
    for v in obj.data.vertices:
        p = v.co
        n = vnoise.noise(Vector((p.x * 1.6 + seed, p.y * 1.6 - seed, p.z * 1.6 + seed * 0.5)))
        v.co = p * (1.0 + noise * n)
    low = min(v.co.z for v in obj.data.vertices)
    for v in obj.data.vertices:
        v.co.z -= low
    obj.scale = scale
    return obj


def _scale_mesh(scale):
    """Scales the new object's mesh (not the object): a modifier added later works in metres."""
    if tuple(scale) != (1, 1, 1):
        bpy.context.object.data.transform(Matrix.Diagonal((*scale, 1.0)))


def _finish(name, at, rot, material, smooth, bevel, bevel_segments, angle=SHARP):
    obj = bpy.context.object
    obj.name = name
    if material:
        obj.data.materials.append(material)
    if smooth:
        shade(obj, True, angle)
    if bevel:
        bevel_edges(obj, bevel, bevel_segments)
    obj.location = at
    obj.rotation_euler = tuple(math.radians(a) for a in rot)
    return obj


def _recalc_normals(obj):
    bpy.ops.object.select_all(action='DESELECT')
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.mesh.normals_make_consistent(inside=False)
    bpy.ops.object.mode_set(mode='OBJECT')


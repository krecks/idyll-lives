# Primitives: every builder adds one mesh object centred on `at` (Blender axes, metres) with its
# material, and returns it. Sizes are full extents (a cube of size 0.4 is 0.4 m across), matching the
# game's `box`/`cylinder` parts. Sizes and scales go into the mesh itself, so a `bevel` is in metres on
# every side. `rot` is in degrees.
import math

import bpy
from mathutils import Matrix, Vector
from mathutils import noise as vnoise

from .scene import TAU


def cube(name, size, at=(0, 0, 0), material=None, rot=(0, 0, 0), smooth=False, bevel=0.0, bevel_segments=2):
    """A box of `size` (x, y, z) or one number for all three."""
    bpy.ops.mesh.primitive_cube_add(size=1.0)
    _scale_mesh(size if isinstance(size, (tuple, list)) else (size, size, size))
    return _finish(name, at, rot, material, smooth, bevel, bevel_segments)


def sphere(name, radius, at=(0, 0, 0), material=None, scale=(1, 1, 1), rot=(0, 0, 0),
           segments=24, rings=16, smooth=True, bevel=0.0, bevel_segments=2):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=segments, ring_count=rings, radius=radius)
    _scale_mesh(scale)
    return _finish(name, at, rot, material, smooth, bevel, bevel_segments)


def cylinder(name, radius, depth, at=(0, 0, 0), material=None, scale=(1, 1, 1), rot=(0, 0, 0),
             verts=24, smooth=True, bevel=0.0, bevel_segments=2):
    """A cylinder along z, `depth` long."""
    bpy.ops.mesh.primitive_cylinder_add(vertices=verts, radius=radius, depth=depth)
    _scale_mesh(scale)
    return _finish(name, at, rot, material, smooth, bevel, bevel_segments)


def cone(name, r1, r2, depth, at=(0, 0, 0), material=None, scale=(1, 1, 1), rot=(0, 0, 0),
         verts=24, smooth=True, bevel=0.0, bevel_segments=2):
    """A cone or frustum along z, radius `r1` at the bottom and `r2` at the top."""
    bpy.ops.mesh.primitive_cone_add(vertices=verts, radius1=r1, radius2=r2, depth=depth)
    _scale_mesh(scale)
    return _finish(name, at, rot, material, smooth, bevel, bevel_segments)


def torus(name, major_radius, minor_radius, at=(0, 0, 0), material=None, rot=(0, 0, 0),
          major_segments=24, minor_segments=12, smooth=True):
    """A ring lying in the xy plane."""
    bpy.ops.mesh.primitive_torus_add(major_radius=major_radius, minor_radius=minor_radius,
                                     major_segments=major_segments, minor_segments=minor_segments)
    return _finish(name, at, rot, material, smooth, 0.0, 2)


def rod(name, a, b, radius, material=None, verts=12, smooth=True):
    """A cylinder of `radius` from point `a` to point `b` (both in metres)."""
    va, vb = Vector(a), Vector(b)
    d = vb - va
    obj = cylinder(name, radius, d.length, at=tuple((va + vb) / 2), material=material,
                   verts=verts, smooth=smooth)
    obj.rotation_mode = 'QUATERNION'
    obj.rotation_quaternion = d.to_track_quat('Z', 'Y')
    return obj


def revolve(name, profile, at=(0, 0, 0), material=None, segments=24, scale=(1, 1, 1),
            rot=(0, 0, 0), smooth=True):
    """Surface of revolution around z. `profile` is [(r, z), ...] in metres, top last."""
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
    if smooth:
        for p in obj.data.polygons:
            p.use_smooth = True
    obj.location = at
    obj.rotation_euler = tuple(math.radians(a) for a in rot)
    obj.scale = scale
    return obj


def rock(name, radius, at=(0, 0, 0), material=None, scale=(1, 1, 1), rot=(0, 0, 0),
         seed=0.0, noise=0.22, subdivisions=3, smooth=True):
    """A lumpy, deterministic boulder: an icosphere displaced by value noise, bottom on z = 0."""
    bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=subdivisions, radius=radius)
    obj = _finish(name, at, rot, material, smooth, 0.0, 2)
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


def _finish(name, at, rot, material, smooth, bevel, bevel_segments):
    obj = bpy.context.object
    obj.name = name
    if material:
        obj.data.materials.append(material)
    if smooth:
        bpy.ops.object.shade_smooth()
    if bevel:
        m = obj.modifiers.new('bevel', 'BEVEL')
        m.width = bevel
        m.segments = bevel_segments
        m.limit_method = 'ANGLE'
        m.angle_limit = math.radians(40)
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

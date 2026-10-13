# Shared helpers for the Blender-built models (see README.md). Blender 4.2+ / 5.x.
#
# Conventions, matching the game (docs: "Replacing art"):
#   1 unit = 1 metre, origin at the footprint centre on the floor (z = 0), front facing -Y in
#   Blender. The glTF exporter's default +Y up turns Blender -Y into the game's +Z front.
#
# A model script imports this module, calls `reset()`, builds meshes with the helpers below
# (each returns a Blender object) and calls `export_glb(path, objects)`.
import math
import os

import bpy

TAU = math.tau


def reset():
    """Empty scene, metres, no default cube/camera/light. Safe to call between models."""
    if bpy.context.object and bpy.context.object.mode != 'OBJECT':
        bpy.ops.object.mode_set(mode='OBJECT')
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete()
    for block in list(bpy.data.meshes):
        bpy.data.meshes.remove(block)
    for block in list(bpy.data.materials):
        bpy.data.materials.remove(block)
    _MATERIALS.clear()
    scene = bpy.context.scene
    scene.unit_settings.system = 'METRIC'
    scene.unit_settings.scale_length = 1.0
    return scene


# ---- materials ---------------------------------------------------------------------------

_MATERIALS = {}


def mat(name, color, roughness=0.6, metallic=0.0, alpha=1.0, emission=0.0):
    """A cached Principled material. `color` is an sRGB hex string or an (r, g, b) 0..1 tuple."""
    key = (name, color, roughness, metallic, alpha, emission)
    if key in _MATERIALS:
        return _MATERIALS[key]
    m = bpy.data.materials.new(name)
    m.use_nodes = True
    bsdf = m.node_tree.nodes.get('Principled BSDF')
    rgb = _rgb(color)
    bsdf.inputs['Base Color'].default_value = (*rgb, 1.0)
    bsdf.inputs['Roughness'].default_value = roughness
    bsdf.inputs['Metallic'].default_value = metallic
    if alpha < 1.0:
        bsdf.inputs['Alpha'].default_value = alpha
        m.blend_method = 'BLEND'
    if emission:
        for socket in ('Emission Color', 'Emission'):
            if socket in bsdf.inputs:
                bsdf.inputs[socket].default_value = (*rgb, 1.0)
                break
        if 'Emission Strength' in bsdf.inputs:
            bsdf.inputs['Emission Strength'].default_value = emission
    _MATERIALS[key] = m
    return m


def _rgb(color):
    if isinstance(color, str):
        c = color.lstrip('#')
        return tuple(int(c[i:i + 2], 16) / 255 for i in (0, 2, 4))
    return tuple(color)


# ---- primitives --------------------------------------------------------------------------
# Every builder returns a Blender object centred on `at`, with its material appended. Sizes are
# full extents (a cube of size 0.4 is 0.4 m across), matching the game's `box`/`cylinder` parts.

def cube(name, size, at=(0, 0, 0), material=None, rot=(0, 0, 0), smooth=False, bevel=0.0, bevel_segments=2):
    bpy.ops.mesh.primitive_cube_add(size=1.0)
    obj = _finish(name, at, rot, material, smooth, bevel, bevel_segments)
    obj.scale = size
    return obj


def sphere(name, radius, at=(0, 0, 0), material=None, scale=(1, 1, 1), rot=(0, 0, 0),
           segments=24, rings=16, smooth=True, bevel=0.0, bevel_segments=2):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=segments, ring_count=rings, radius=radius)
    obj = _finish(name, at, rot, material, smooth, bevel, bevel_segments)
    obj.scale = scale
    return obj


def cylinder(name, radius, depth, at=(0, 0, 0), material=None, scale=(1, 1, 1), rot=(0, 0, 0),
             verts=24, smooth=True, bevel=0.0, bevel_segments=2):
    bpy.ops.mesh.primitive_cylinder_add(vertices=verts, radius=radius, depth=depth)
    obj = _finish(name, at, rot, material, smooth, bevel, bevel_segments)
    obj.scale = scale
    return obj


def cone(name, r1, r2, depth, at=(0, 0, 0), material=None, scale=(1, 1, 1), rot=(0, 0, 0),
         verts=24, smooth=True, bevel=0.0, bevel_segments=2):
    bpy.ops.mesh.primitive_cone_add(vertices=verts, radius1=r1, radius2=r2, depth=depth)
    obj = _finish(name, at, rot, material, smooth, bevel, bevel_segments)
    obj.scale = scale
    return obj


def torus(name, major_radius, minor_radius, at=(0, 0, 0), material=None, rot=(0, 0, 0),
          major_segments=24, minor_segments=12, smooth=True):
    bpy.ops.mesh.primitive_torus_add(major_radius=major_radius, minor_radius=minor_radius,
                                     major_segments=major_segments, minor_segments=minor_segments)
    return _finish(name, at, rot, material, smooth, 0.0, 2)


def rod(name, a, b, radius, material=None, verts=12, smooth=True):
    """A cylinder of `radius` from point `a` to point `b` (both in metres)."""
    from mathutils import Vector
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
    obj.data.materials.append(material) if material else None
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
    from mathutils import Vector
    from mathutils import noise as vnoise
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


# ---- assembly and export ------------------------------------------------------------------

def join_all(name):
    """Applies every transform and modifier, joins every mesh in the scene, returns the one object."""
    objs = [o for o in bpy.context.scene.objects if o.type == 'MESH']
    bpy.ops.object.select_all(action='DESELECT')
    for o in objs:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objs[0]
    bpy.ops.object.convert(target='MESH')
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
    bpy.ops.object.join()
    obj = bpy.context.object
    obj.name = name
    return obj


def fit(obj, w, h, d):
    """Scales the object into an exact w × h × d box (game metres), bottom on z = 0, centred."""
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


def export_glb(path, objects=None):
    """Writes a .glb (all mesh objects unless given), +Y up so Blender -Y is the game's +Z front."""
    os.makedirs(os.path.dirname(path), exist_ok=True)
    bpy.ops.object.select_all(action='DESELECT')
    objects = objects if objects is not None else [o for o in bpy.context.scene.objects if o.type == 'MESH']
    for o in objects:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
    bpy.ops.export_scene.gltf(filepath=path, export_format='GLB', use_selection=True,
                              export_apply=True, export_yup=True, export_materials='EXPORT')
    return path


def stats(path=None):
    tris = 0
    for o in bpy.context.scene.objects:
        if o.type == 'MESH':
            o.data.calc_loop_triangles()
            tris += len(o.data.loop_triangles)
    print(f'[gamekit] {tris} triangles in {len([o for o in bpy.context.scene.objects if o.type == "MESH"])} meshes')
    return tris


# ---- preview (not part of the pipeline; for checking the build) ------------------------------

def preview(path, size=1.2, samples=16):
    """Renders the scene to `path` from a three-quarter front view, for eyeballing a build."""
    scene = bpy.context.scene
    scene.render.engine = 'BLENDER_WORKBENCH'
    scene.display.shading.light = 'STUDIO'
    scene.display.shading.color_type = 'MATERIAL'
    scene.display.shading.show_shadows = True
    scene.render.resolution_x = 640
    scene.render.resolution_y = 640
    scene.render.film_transparent = False

    from mathutils import Vector
    ws = [o.matrix_world @ Vector(c) for o in bpy.context.scene.objects if o.type == 'MESH'
          for c in [v.co for v in o.data.vertices]]
    mn = Vector((min(v[i] for v in ws) for i in range(3)))
    mx = Vector((max(v[i] for v in ws) for i in range(3)))
    span = max(mx - mn)
    target = (mn + mx) / 2
    loc = target + Vector((span * 0.95, span * 1.05, span * 0.75))
    cam = bpy.data.cameras.new('preview')
    cam.type = 'ORTHO'
    cam.ortho_scale = span * 1.55
    obj = bpy.data.objects.new('preview', cam)
    bpy.context.collection.objects.link(obj)
    obj.location = loc
    obj.rotation_euler = (target - loc).to_track_quat('-Z', 'Y').to_euler()
    scene.camera = obj
    scene.render.filepath = path
    scene.render.image_settings.file_format = 'PNG'
    bpy.ops.render.render(write_still=True)
    bpy.data.objects.remove(obj, do_unlink=True)
    return path


def cosd(a):
    return math.cos(math.radians(a))


def sind(a):
    return math.sin(math.radians(a))

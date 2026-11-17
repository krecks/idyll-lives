# Writing a model out: the .glb (docs/design/objects.md, "Export settings"). The family's pack entry
# ("Optimisation and registration") is written by the runner (`cargo art`: gamekit/runner.py).
import os

import bpy

from . import uv
from .scene import meshes
from .shade import ATTRIBUTE, bake_ao


def export_glb(path, objects=None, texcoords=False, clips=False, shade=True, ground=False):
    """
    Writes a .glb (all mesh objects unless given), +Y up so Blender -Y is the game's +Z front.
    UVs are written for a model with parts on textured finishes (wood, fabric, bark, foliage: gamekit/uv.py),
    and otherwise left out (`texcoords` writes them anyway).
    `clips`: a model with moving parts writes its clips, one per NLA track name, and the scene's
    current frame as the rest pose (frame 1, each clip's first key). Each clip starts at 0 s: the game
    holds a clip at its first key until then, which paused every loop at rest for a frame each round.
    `ground`: the model stands on the floor, which shades the foot of it too. `shade` bakes the creases'
    shading into each vertex first (gamekit/shade.py), written as COLOR_0.
    """
    os.makedirs(os.path.dirname(path), exist_ok=True)
    objects = objects if objects is not None else meshes()
    # The textured finishes' UVs (gamekit/uv.py), made when the parts were joined; a part never joined gets its own.
    texcoords = uv.ensure(objects) or texcoords
    if shade:
        bake_ao(objects, ground=ground)
    bpy.ops.object.select_all(action='DESELECT')
    for o in objects:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
    extra = ({'export_animation_mode': 'NLA_TRACKS', 'export_anim_slide_to_zero': True} if clips
             else {'export_animations': False})
    bpy.ops.export_scene.gltf(filepath=path, export_format='GLB', use_selection=True,
                              export_apply=True, export_yup=True, export_materials='EXPORT',
                              export_texcoords=texcoords, export_vertex_color='NAME', export_vertex_color_name=ATTRIBUTE,
                              export_all_vertex_colors=False, **extra)
    return path


def stats(name='model'):
    """Prints and returns the scene's triangles."""
    objs = meshes()
    tris = 0
    for o in objs:
        o.data.calc_loop_triangles()
        tris += len(o.data.loop_triangles)
    print(f'[gamekit] {name}: {tris} triangles in {len(objs)} meshes')
    return tris


def materials_used(objs=None):
    """The names of the materials on `objs` (default: every mesh)."""
    return {s.material.name for o in (objs if objs is not None else meshes()) for s in o.material_slots if s.material}

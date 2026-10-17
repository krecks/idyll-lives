# The scene a model is built in: emptied between models, in metres.
import math
import os

import bpy

TAU = math.tau

# The committed models were exported with Blender 5.2 LTS (glTF I/O 5.2.40); other versions warn.
BLENDER = (5, 2)
if tuple(bpy.app.version[:2]) != BLENDER:
    print(f'[gamekit] warning: Blender {bpy.app.version_string}; the models were made with {BLENDER[0]}.{BLENDER[1]} LTS')

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))  # tools/art/blender
ASSETS = os.path.normpath(os.path.join(HERE, '..', '..', '..', 'web', 'public', 'assets'))
PREVIEWS = os.path.join(HERE, 'previews')


def reset():
    """Empty scene (no default cube, camera or light, no materials or clips), metres, frame 1."""
    if bpy.context.object and bpy.context.object.mode != 'OBJECT':
        bpy.ops.object.mode_set(mode='OBJECT')
    for obj in list(bpy.data.objects):
        bpy.data.objects.remove(obj, do_unlink=True)
    for blocks in (bpy.data.meshes, bpy.data.materials, bpy.data.actions, bpy.data.cameras, bpy.data.lights):
        for block in list(blocks):
            blocks.remove(block)
    scene = bpy.context.scene
    scene.unit_settings.system = 'METRIC'
    scene.unit_settings.scale_length = 1.0
    scene.frame_set(1)
    return scene


def meshes():
    """The scene's mesh objects, in the order they were made."""
    return [o for o in bpy.context.scene.objects if o.type == 'MESH']


def cosd(a):
    return math.cos(math.radians(a))


def sind(a):
    return math.sin(math.radians(a))

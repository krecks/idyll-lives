# Previews: Workbench renders for eyeballing a build (not part of the pipeline; previews/ is ignored by
# git). The game is the real check: docs/design/objects.md, "Validation".
import bpy
from mathutils import Vector

from . import motion
from .parts import measure


# Where `preview` looks from: (x, y, z) times the model's largest side, from its middle.
VIEWS = {
    'three_quarter': (0.95, -1.05, 0.75),   # the front three-quarter (Blender -Y, the game's front), from the right
    'side': (1.6, 0.0, 0.0),                # from its left (Blender +X; a bird's left, its wing.L)
    'front': (0.0, -1.6, 0.0),
    'top': (0.0, -0.0001, 1.6),             # straight down, its front at the bottom of the picture
}


def preview(path, view='three_quarter'):
    """Renders the scene to `path` from the front three-quarter view (Blender -Y, the game's front), or another of
    `VIEWS`."""
    scene = bpy.context.scene
    scene.render.engine = 'BLENDER_WORKBENCH'
    scene.display.shading.light = 'STUDIO'
    scene.display.shading.color_type = 'MATERIAL'
    scene.display.shading.show_shadows = True
    scene.render.resolution_x = 640
    scene.render.resolution_y = 640
    scene.render.film_transparent = False

    lo, hi = (Vector(v) for v in measure())
    span = max(hi - lo)
    target = (lo + hi) / 2
    loc = target + Vector(VIEWS[view]) * span
    cam = bpy.data.cameras.new('preview')
    cam.type = 'ORTHO'
    cam.ortho_scale = span * (1.55 if view == 'three_quarter' else 1.15)
    obj = bpy.data.objects.new('preview', cam)
    bpy.context.collection.objects.link(obj)
    obj.location = loc
    # (Straight down, unturned: the model's back (+Y) at the top of the picture.)
    obj.rotation_euler = (0, 0, 0) if view == 'top' else (target - loc).to_track_quat('-Z', 'Y').to_euler()
    scene.camera = obj
    scene.render.filepath = path
    scene.render.image_settings.file_format = 'PNG'
    bpy.ops.render.render(write_still=True)
    bpy.data.objects.remove(obj, do_unlink=True)
    bpy.data.cameras.remove(cam)
    return path


def preview_clips(stem):
    """Renders `<stem>@<clip>.png` with each clip on its own where it is most open (its last key, or the
    middle of a clip that comes home: `spin`, `rock`), then goes back to rest."""
    scene = bpy.context.scene
    paths = []
    for clip in dict.fromkeys(name for _, name, _ in motion.clips()):
        # back to rest first: a muted track leaves its part where it last put it
        motion.solo(None)
        scene.frame_set(1)
        motion.solo(clip)
        scene.frame_set(motion.peak(clip))
        paths.append(preview(f'{stem}@{clip}.png'))
    motion.solo(None)
    scene.frame_set(1)
    return paths


def preview_perches(path):
    """Renders `path` with a red ball 12 cm across on each of the model's perches (`gamekit/perch.py`), where a small
    bird would sit on it, from the front three-quarter, and the same from above (`.top` before `.png`); then takes the balls away."""
    from . import perch
    from .materials import mat
    from .scene import meshes
    before = set(meshes())
    red = mat('perch_marker', '#FF2020', roughness=0.5)
    for i, p in enumerate(perch.PERCHES):
        bpy.ops.mesh.primitive_uv_sphere_add(radius=0.06, location=p + Vector((0, 0, 0.05)), segments=8, ring_count=5)
        bpy.context.object.name = f'perch_marker{i}'
        bpy.context.object.data.materials.append(red)
    preview(path)
    preview(path[:-4] + '.top.png', 'top')
    for o in meshes():
        if o not in before:
            bpy.data.objects.remove(o, do_unlink=True)

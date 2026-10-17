# Previews: Workbench renders for eyeballing a build (not part of the pipeline; previews/ is ignored by
# git). The game is the real check: docs/design/objects.md, "Validation".
import bpy
from mathutils import Vector

from . import motion
from .parts import measure


def preview(path):
    """Renders the scene to `path` from the front three-quarter view (Blender -Y, the game's front)."""
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
    loc = target + Vector((span * 0.95, -span * 1.05, span * 0.75))
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

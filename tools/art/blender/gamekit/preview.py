# Previews: Workbench renders for eyeballing a build (not part of the pipeline; previews/ is ignored by
# git), and `lit` ones, lit and textured roughly as the game draws them. The game is the real check:
# docs/design/objects.md, "Validation".
import math
import os

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
    return _render(path, view)


def lit(path, view='three_quarter'):
    """Renders the scene to `path` as the game lights it, roughly: EEVEE, a low sun from the front right and a pale
    sky, each finish's roughness and metalness, the textured finishes' maps (fabric, wood, bark, foliage)
    box-projected at their scale and tinted by the colour, on a floor that takes the shadow. What a player sees,
    where `preview` shows the shapes. It adds nodes to the materials, so only a preview run (never an export)
    calls it."""
    scene = bpy.context.scene
    scene.render.engine = 'BLENDER_EEVEE'
    scene.eevee.taa_render_samples = 32
    scene.view_settings.view_transform = 'AgX'
    world = bpy.data.worlds.get('lit') or bpy.data.worlds.new('lit')
    world.use_nodes = True
    world.node_tree.nodes['Background'].inputs['Color'].default_value = (0.62, 0.72, 0.86, 1)
    world.node_tree.nodes['Background'].inputs['Strength'].default_value = 0.9
    scene.world = world
    for m in bpy.data.materials:
        _textured(m)
        _shaded(m)
    lo, hi = (Vector(v) for v in measure())
    span = max(hi - lo)
    added = []
    sun = bpy.data.lights.new('lit_sun', 'SUN')
    sun.energy = 3.2
    sun.angle = math.radians(3)
    added.append(bpy.data.objects.new('lit_sun', sun))
    added[-1].rotation_euler = Vector((0.6, -0.9, 1.0)).to_track_quat('Z', 'Y').to_euler()   # the light comes from there
    floor = bpy.data.meshes.new('lit_floor')
    r = span * 4
    floor.from_pydata([(-r, -r, lo.z), (r, -r, lo.z), (r, r, lo.z), (-r, r, lo.z)], [], [(0, 1, 2, 3)])
    floor.materials.append(_floor())
    added.append(bpy.data.objects.new('lit_floor', floor))
    for o in added:
        bpy.context.collection.objects.link(o)
    try:
        return _render(path, view, (lo, hi))
    finally:
        for o in added:
            data, blocks = o.data, bpy.data.lights if o.type == 'LIGHT' else bpy.data.meshes
            bpy.data.objects.remove(o, do_unlink=True)
            blocks.remove(data)


def _floor():
    m = bpy.data.materials.get('lit_floor')
    if m is None:
        m = bpy.data.materials.new('lit_floor')
        m.use_nodes = True
        bsdf = m.node_tree.nodes['Principled BSDF']
        bsdf.inputs['Base Color'].default_value = (0.42, 0.40, 0.36, 1)
        bsdf.inputs['Roughness'].default_value = 0.9
    return m


def _textured(m):
    """Gives a material on a textured finish its maps, once: the albedo (times the colour) and the normal map,
    box-projected in the object's metres times the finish's `uvScale`, as the game's `boxProjectUVs` does."""
    from .materials import TEXTURES
    finish = m.get('finish')
    if finish not in TEXTURES or m.get('lit') or not m.use_nodes:
        return
    m['lit'] = True
    albedo, scale = TEXTURES[finish]
    nodes, links = m.node_tree.nodes, m.node_tree.links
    bsdf = nodes['Principled BSDF']
    coords = nodes.new('ShaderNodeTexCoord')
    mapping = nodes.new('ShaderNodeMapping')
    mapping.inputs['Scale'].default_value = (scale, scale, scale)
    links.new(coords.outputs['Object'], mapping.inputs['Vector'])

    def image(name, colour):
        tex = nodes.new('ShaderNodeTexImage')
        tex.image = bpy.data.images.load(os.path.join(os.path.dirname(albedo), name), check_existing=True)
        if not colour:
            tex.image.colorspace_settings.name = 'Non-Color'
        tex.projection = 'BOX'
        tex.projection_blend = 0.2
        links.new(mapping.outputs['Vector'], tex.inputs['Vector'])
        return tex
    mix = nodes.new('ShaderNodeMix')
    mix.data_type = 'RGBA'
    mix.blend_type = 'MULTIPLY'
    mix.inputs['Factor'].default_value = 1.0
    socket = {s.identifier: s for s in mix.inputs}
    socket['A_Color'].default_value = bsdf.inputs['Base Color'].default_value
    links.new(image(os.path.basename(albedo), True).outputs['Color'], socket['B_Color'])
    links.new(next(s for s in mix.outputs if s.identifier == 'Result_Color'), bsdf.inputs['Base Color'])
    normal = nodes.new('ShaderNodeNormalMap')
    links.new(image('normal.jpg', False).outputs['Color'], normal.inputs['Color'])
    links.new(normal.outputs['Normal'], bsdf.inputs['Normal'])


def _shaded(m):
    """Multiplies a material's colour by the baked shading (`ao`, gamekit/shade.py), as the game does, once."""
    from .shade import ATTRIBUTE
    if m.get('shaded') or not m.use_nodes or m.name.startswith('lit_'):
        return
    m['shaded'] = True
    nodes, links = m.node_tree.nodes, m.node_tree.links
    bsdf = nodes['Principled BSDF']
    ao = nodes.new('ShaderNodeVertexColor')
    ao.layer_name = ATTRIBUTE
    mix = nodes.new('ShaderNodeMix')
    mix.data_type = 'RGBA'
    mix.blend_type = 'MULTIPLY'
    mix.inputs['Factor'].default_value = 1.0
    socket = {s.identifier: s for s in mix.inputs}
    base = bsdf.inputs['Base Color']
    if base.links:
        links.new(base.links[0].from_socket, socket['A_Color'])
    else:
        socket['A_Color'].default_value = base.default_value
    links.new(ao.outputs['Color'], socket['B_Color'])
    links.new(next(s for s in mix.outputs if s.identifier == 'Result_Color'), base)


def _render(path, view, bounds=None):
    scene = bpy.context.scene
    scene.render.resolution_x = 640
    scene.render.resolution_y = 640
    scene.render.film_transparent = False

    lo, hi = bounds or (Vector(v) for v in measure())
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
    from .shapes import sphere
    before = set(meshes())
    red = mat('perch_marker', '#FF2020', roughness=0.5)
    for i, p in enumerate(perch.PERCHES):
        sphere(f'perch_marker{i}', 0.06, at=p + Vector((0, 0, 0.05)), material=red, segments=8, rings=5, smooth=False)
    preview(path)
    preview(path[:-4] + '.top.png', 'top')
    for o in meshes():
        if o not in before:
            bpy.data.objects.remove(o, do_unlink=True)

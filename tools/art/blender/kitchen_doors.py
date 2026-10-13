# The fridge's doors as separate nodes with their origins on the hinge, plus a clip per door (each with its
# own angle) so the game can open the freezer or the fridge on its own: this is the file `model.fridge`
# draws (the pack's `url`, written here and by `kitchen.py`), its doors played by the pack's `motions`.
# At rest it draws closed and identical to `kitchen.py`'s flat build.
#
# Run:  blender --background --factory-startup --python tools/art/blender/kitchen_doors.py
#       blender --background --factory-startup --python tools/art/blender/kitchen_doors.py -- --preview
import math
import os
import sys

import bpy
from mathutils import Matrix, Vector

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gamekit as G  # noqa: E402
import kitchen as K  # noqa: E402

OUT = os.path.join(K.ASSETS, 'models', 'kitchen')
PREVIEW = K.PREVIEW
# Both doors open and close at the same speed (the fridge door's), so the pair feels consistent; only the
# angle differs — the short freezer door swings wide, the tall fridge door stops a little shorter.
FREEZER = {'angle': math.radians(-118), 'frames': 42}  # 1.75 s, wide
FRIDGE = {'angle': math.radians(-102), 'frames': 42}   # 1.75 s, shorter swing
FRAMES = max(FREEZER['frames'], FRIDGE['frames'])      # the longest clip, for previews


def _meshes():
    return [o for o in bpy.context.scene.objects if o.type == 'MESH']


def _join_as(name, active, parts):
    """Joins `parts` into `active`, keeping `active`'s origin (unlike `gamekit.join_all`)."""
    bpy.ops.object.select_all(action='DESELECT')
    for o in parts:
        o.select_set(True)
    bpy.context.view_layer.objects.active = active
    bpy.ops.object.convert(target='MESH')
    bpy.ops.object.join()
    active.name = name
    return active


def _hinged(name, w, h, d, at, hinge, material, **kw):
    """A game-sized panel whose Blender origin sits on the hinge edge, so Z rotation swings it."""
    obj = K.gbox(name, w, h, d, at, material, **kw)
    pivot = Vector(K.P(*hinge))
    obj.data.transform(Matrix.Translation(Vector(K.P(*at)) - pivot))
    obj.location = pivot
    return obj


def _cabinet():
    before = set(_meshes())
    K.gbox('plinth', 0.68, 0.06, 0.57, (0, 0.03, -0.17), K.TRIM, bevel=0.006)
    K._shell()
    K._interior()
    K.gbox('gap', 0.7, 0.014, 0.03, (0, 1.224, 0.187), K.SEAL, bevel=0.004)
    parts = [o for o in _meshes() if o not in before]
    return _join_as('cabinet', parts[0], parts)


def _door(name, y, h, handle_y0, handle_y1, badge=False):
    before = set(_meshes())
    door = _hinged(name, 0.728, h, 0.04, (0, y, 0.195), (0.364, y, 0.195), K.BODY, bevel=0.010)
    K._bar(f'handle_{name}', -0.3, handle_y0, handle_y1, 0.215)
    if badge:
        K.gbox('badge', 0.1, 0.018, 0.004, (0.25, 1.74, 0.217), K.TRIM, bevel=0.002)
    parts = [o for o in _meshes() if o not in before]
    return _join_as(name, door, parts)


def _swing(door, name, a0, a1, frames):
    """An action `name` turning `door` about its Z (up) axis from `a0` to `a1` over `frames` frames."""
    door.rotation_mode = 'XYZ'
    door.animation_data_create()
    action = bpy.data.actions.new(name)
    door.animation_data.action = action
    for frame, angle in ((1, a0), (frames, a1)):
        door.rotation_euler[2] = angle
        door.keyframe_insert('rotation_euler', index=2, frame=frame)
    track = door.animation_data.nla_tracks.new()
    track.name = name
    track.strips.new(name, 1, action)
    door.animation_data.action = None


def _fridge_doors():
    _cabinet()
    _door('door_freezer', 1.51, 0.556, 1.274, 1.524, badge=True)
    _door('door_fridge', 0.643, 1.146, 0.804, 1.164)
    # one clip per door, both over the same 42 frames so the two doors open and close at one speed, but with
    # their own angle so the two doors still read differently. Each door's first key is closed, so the rest
    # pose is closed. The engine can open either on its own.
    _swing(bpy.context.scene.objects['door_freezer'], 'freezer_open', 0.0,
           FREEZER['angle'], FREEZER['frames'])
    _swing(bpy.context.scene.objects['door_fridge'], 'fridge_open', 0.0,
           FRIDGE['angle'], FRIDGE['frames'])
    return _meshes()


def _check(objs, w, h, d, tol=0.02):
    ws = [o.matrix_world @ v.co for o in objs for v in o.data.vertices]
    lo = [min(v[i] for v in ws) for i in range(3)]
    hi = [max(v[i] for v in ws) for i in range(3)]
    got = [hi[i] - lo[i] for i in range(3)]
    want = [w, d, h]
    for i, axis in enumerate('xyz'):
        assert abs(got[i] - want[i]) <= tol * want[i], f'{axis} size {got[i]:.3f} m, want {want[i]:.3f} m'
    for i, axis in enumerate('xyz'):
        c = (lo[i] + hi[i]) / 2
        if i < 2:
            assert abs(c) <= 0.002, f'{axis} off centre by {c * 100:.1f} cm'
        else:
            assert abs(lo[i]) <= 0.002, f'bottom {lo[i] * 100:.1f} cm off the floor'
    print(f'[doors] fridge {got[0]:.3f} x {got[2]:.3f} x {got[1]:.3f} m (w x h x d), centred, on the floor')


def main(argv):
    preview = '--preview' in argv
    os.makedirs(OUT, exist_ok=True)
    os.makedirs(PREVIEW, exist_ok=True)
    G.reset()
    K._palette()
    objs = _fridge_doors()
    _check(objs, *K.TARGET)
    G.stats()
    if preview:
        for frame, name in ((1, 'doors_closed.png'), (FRAMES, 'doors_open.png')):
            bpy.context.scene.frame_set(frame)
            G.preview(os.path.join(PREVIEW, name), size=1.3)
    else:
        # The rest pose is each clip's first key: the doors closed.
        bpy.context.scene.frame_set(1)
        G.export_glb(os.path.join(OUT, 'fridge.doors.glb'), objs, clips=True)
        K._write_pack()
        print('built fridge.doors.glb')


if __name__ == '__main__':
    main(sys.argv)

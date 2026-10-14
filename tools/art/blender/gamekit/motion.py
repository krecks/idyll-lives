# Moving parts: a door, lid or drawer that moves while residents use the object (docs/design/objects.md,
# "Moving parts"). Each part is its own object with its origin on the hinge (`G.hinged`), and each clip
# is one NLA track on it, named for the pack's `motions`:
#
#   door = G.hinged(G.join(parts, 'door_fridge'), F.at(0.364, 0.643, 0.195))
#   G.swing(door, 'fridge_open', -102, seconds=1.75)
#
# A clip's first key (frame 1) is the rest pose: where the file has the part, and the game draws it
# closed. Only the node's rotation, location or scale move; two keys are enough (the game samples the
# clip 30 times a second). `G.run` exports the clips when a model has any.
import math

import bpy

from .scene import meshes


def fps():
    scene = bpy.context.scene
    return scene.render.fps / scene.render.fps_base


def last_frame(seconds):
    """The frame of a clip's last key, for a clip that ends `seconds` in (the exporter's clock)."""
    return max(2, round(seconds * fps()))


def swing(obj, clip, degrees, seconds=1.0, axis='Z'):
    """A clip `clip` turning `obj` about its own `axis` ('X', 'Y' or 'Z', Blender) by `degrees` from
    rest. A door hinged on its right edge (seen from the front) opens with a negative angle about Z."""
    i = 'XYZ'.index(axis)
    obj.rotation_mode = 'XYZ'
    rest = obj.rotation_euler[i]
    return _clip(obj, clip, 'rotation_euler', i, rest, rest + math.radians(degrees), seconds)


def slide(obj, clip, offset, seconds=1.0):
    """A clip `clip` moving `obj` by `offset` (Blender x, y, z, metres) from rest: a drawer pulls out
    to the front with a negative y."""
    for i, d in enumerate(offset):
        if d:
            rest = obj.location[i]
            _clip(obj, clip, 'location', i, rest, rest + d, seconds)
    return obj


def _clip(obj, clip, path, index, a, b, seconds):
    """Keys `obj`.`path`[`index`] from `a` (frame 1) to `b`, in an action pushed to the NLA track `clip`
    (one action per track; several channels of one clip share it)."""
    obj.animation_data_create()
    track = obj.animation_data.nla_tracks.get(clip)
    action = track.strips[0].action if track else bpy.data.actions.new(f'{obj.name}.{clip}')
    obj.animation_data.action = action
    end = last_frame(seconds)
    for frame, value in ((1, a), (end, b)):
        getattr(obj, path)[index] = value
        obj.keyframe_insert(path, index=index, frame=frame)
    getattr(obj, path)[index] = a
    if not track:
        track = obj.animation_data.nla_tracks.new()
        track.name = clip
        track.strips.new(clip, 1, action)
    else:
        strip = track.strips[0]
        strip.action_frame_end = max(strip.action_frame_end, end)
        strip.frame_end = max(strip.frame_end, end)
    obj.animation_data.action = None
    return obj


def clips(objs=None):
    """[(object, clip name, last frame)] for every clip in the scene (or on `objs`)."""
    out = []
    for o in objs if objs is not None else meshes():
        if o.animation_data:
            for t in o.animation_data.nla_tracks:
                out.append((o, t.name, max(int(s.frame_end) for s in t.strips)))
    return out


def solo(clip):
    """Plays only the tracks named `clip` (None: all of them again), for previews."""
    for o, name, _ in clips():
        o.animation_data.nla_tracks[name].mute = clip is not None and name != clip

# Moving parts: a door, lid or drawer that moves while residents use the object (docs/design/objects.md,
# "Moving parts"). Each part is its own object with its origin on the hinge (`G.hinged`), and each clip
# is one NLA track on it, named for the pack's `motions`:
#
#   door = G.hinged(G.join(parts, 'door_fridge'), F.at(0.364, 0.643, 0.195))
#   G.swing(door, 'fridge_open', -102, seconds=1.75)
#
# A clip's first key (frame 1) is the rest pose: where the file has the part, and the game draws it
# closed. Only the node's rotation, location or scale move. `swing`, `slide`, `grow` and `squash` go from
# rest to open (the game plays them back); `spin` and `rock` come home to rest within the clip, so they
# also serve as loops (`motions.loops`, `idle`). `G.run` exports the clips when a model has any; the
# exporter samples every frame, and the game samples the clip 30 times a second.
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


def spin(obj, clip, turns=1, seconds=1.0, axis='Z'):
    """A clip `clip` turning `obj` about its own `axis` ('X', 'Y' or 'Z', Blender) by whole `turns`
    (negative: the other way) at an even speed, ending where it started: a wheel, a record platter, a
    fan. Keep it under about one turn a second (the game blends between its samples); a part that looks
    the same turned (a fan's blades) can turn by less, e.g. a quarter turn with four blades, and loop."""
    i = 'XYZ'.index(axis)
    obj.rotation_mode = 'XYZ'
    rest = obj.rotation_euler[i]
    end = last_frame(seconds)
    _clip(obj, clip, 'rotation_euler', i, [(1, rest), (end, rest + turns * 2 * math.pi)], linear=True)
    return _peak(obj, clip, (1 + end) / 2)


def rock(obj, clip, degrees, seconds=1.0, axis='X'):
    """A clip `clip` rocking `obj` about its own `axis` ('X', 'Y' or 'Z', Blender): out by `degrees`, back
    past rest to the other side and home, as a pendulum (a rocking chair, a swing, the seesaw). It ends
    where it started at the speed it started with, so it loops smoothly."""
    i = 'XYZ'.index(axis)
    obj.rotation_mode = 'XYZ'
    rest = obj.rotation_euler[i]
    end = last_frame(seconds)
    a = math.radians(degrees)
    keys = [(f, rest + a * math.sin(2 * math.pi * (f - 1) / (end - 1))) for f in range(1, end + 1)]
    _clip(obj, clip, 'rotation_euler', i, keys, linear=True)
    return _peak(obj, clip, 1 + (end - 1) / 4)


def squash(obj, clip, scale, seconds=0.6, back=False):
    """A clip `clip` scaling `obj` about its own origin from rest to `scale` times its size: a number for
    every axis, or (x, y, z) Blender. A cushion sinking (0.7 along Z), a bag flattening, a held instrument
    or weight hidden while in the hands (0.001). The game plays a use's clip back on its own; `back` makes
    the clip come back to rest by itself (a bounce, for a loop)."""
    factors = scale if isinstance(scale, (tuple, list)) else (scale,) * 3
    end = last_frame(seconds)
    for i, k in enumerate(factors):
        if k == 1:
            continue
        rest = obj.scale[i]
        keys = [(1, rest), ((1 + end) // 2, rest * k), (end, rest)] if back else [(1, rest), (end, rest * k)]
        _clip(obj, clip, 'scale', i, keys)
    assert obj.animation_data and obj.animation_data.nla_tracks.get(clip), f'squash {clip!r}: a scale of 1 does nothing'
    return _peak(obj, clip, (1 + end) // 2 if back else end)


def turns(objs, clip, offset, seconds=1.0):
    """A clip `clip` in which `objs` take turns: each in its share of `seconds` rises by `offset` (Blender x, y, z,
    metres) and settles back, the first one first; it ends where it starts, so it loops (`motions.loops`, `idle`).
    A dance floor's two checkerboards of tiles lifting one after the other."""
    end = last_frame(seconds)
    bounds = [1 + round((end - 1) * k / len(objs)) for k in range(len(objs) + 1)]
    for obj, a, b in zip(objs, bounds, bounds[1:]):
        for i, d in enumerate(offset):
            if d:
                rest = obj.location[i]
                keys = [(1, rest), (a, rest), ((a + b) // 2, rest + d), (b, rest), (end, rest)]
                keys = [kv for n, kv in enumerate(keys) if n == 0 or kv[0] != keys[n - 1][0]]   # one key a frame
                _clip(obj, clip, 'location', i, keys)
    return _peak(objs[0], clip, (bounds[0] + bounds[1]) // 2)


def stages(obj, clip, picked, growing, planted, seconds=1.5):
    """A crop's growth clip `clip` (`motions.growth`) for `obj`: the scale of it at each growth stage,
    times its size at rest, which is ripe. A number for every axis or (x, y, z) Blender; 0.001 hides
    it. The game holds the clip where the stage is: ripe at rest, picked a third of the way, growing two
    thirds, planted at the end, and plays it on from stage to stage. The fruit: picked=0.001,
    growing=0.001, planted=0.001; the plants: picked=1, growing=0.6, planted=0.25."""
    end = last_frame(seconds)
    frames = [1, 1 + (end - 1) / 3, 1 + 2 * (end - 1) / 3, end]
    each = [k if isinstance(k, (tuple, list)) else (k,) * 3 for k in (1, picked, growing, planted)]
    for i in range(3):
        if any(k[i] != 1 for k in each):
            rest = obj.scale[i]
            _clip(obj, clip, 'scale', i, [(round(f), rest * k[i]) for f, k in zip(frames, each)])
    assert obj.animation_data and obj.animation_data.nla_tracks.get(clip), f'stages {clip!r}: the same at every stage'
    return obj


def grow(obj, clip, seconds=0.4, axis='Z'):
    """A clip `clip` scaling `obj` about its own origin along `axis` ('X', 'Y' or 'Z', Blender) from
    where it is (built small and hidden) to full size (scale 1): a part that appears while a resident
    uses the object (a stream of water)."""
    i = 'XYZ'.index(axis)
    return _clip(obj, clip, 'scale', i, obj.scale[i], 1.0, seconds)


def _clip(obj, clip, path, index, a, b=None, seconds=None, linear=False):
    """Keys `obj`.`path`[`index`] from `a` (frame 1) to `b` at the end of `seconds`, or at the frames of
    `a` when it is a list of (frame, value); in an action pushed to the NLA track `clip` (one action per
    track; several channels of one clip share it). `linear`: an even speed between keys (else eased)."""
    keys = a if isinstance(a, list) else [(1, a), (last_frame(seconds), b)]
    obj.animation_data_create()
    track = obj.animation_data.nla_tracks.get(clip)
    action = track.strips[0].action if track else bpy.data.actions.new(f'{obj.name}.{clip}')
    obj.animation_data.action = action
    end = keys[-1][0]
    for frame, value in keys:
        getattr(obj, path)[index] = value
        obj.keyframe_insert(path, index=index, frame=frame)
    if linear:
        for fc in _fcurves(action, obj):
            if fc.data_path == path and fc.array_index == index:
                for k in fc.keyframe_points:
                    k.interpolation = 'LINEAR'
    getattr(obj, path)[index] = keys[0][1]
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


def _fcurves(action, obj):
    """The F-curves of `action` (Blender 4.4+ keeps them in layered channelbags)."""
    if hasattr(action, 'fcurves'):
        return list(action.fcurves)
    out = []
    for layer in action.layers:
        for strip in layer.strips:
            for bag in strip.channelbags:
                out += list(bag.fcurves)
    return out


def _peak(obj, clip, frame):
    """Marks where clip `clip` is most open, for its preview (default: its last key)."""
    action = obj.animation_data.nla_tracks[clip].strips[0].action
    action['peak'] = round(frame)
    return obj


def peak(clip):
    """The frame a preview shows clip `clip` at: where it is most open."""
    frames = [o.animation_data.nla_tracks[name].strips[0].action.get('peak', end) for o, name, end in clips() if name == clip]
    return max(frames, default=1)


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

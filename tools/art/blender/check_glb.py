# The gate for the game's glTF models (docs/design/objects.md, "Validation"): reads every .glb a
# model entry of the manifests uses (the base manifest and its packs, later ones overriding) with
# that entry, and checks what the game needs of it. Errors exit 1; warnings are printed.
#
#   python3 -I -B tools/art/blender/check_glb.py                       every .glb the manifests use
#   python3 -I -B tools/art/blender/check_glb.py model.garden.gnome …  only these keys
#
# Plain Python, no Blender needed. Run inside Blender with `-- --preview`, it also re-imports each
# file and renders it to tools/art/blender/previews/rt_<name>.png, so export problems show:
#   Blender --background --python tools/art/blender/check_glb.py -- --preview
#
# Run the Khronos glTF validator too (the standard says how); this checks the game's own rules.
import json
import math
import re
import os
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ASSETS = os.path.normpath(os.path.join(HERE, '..', '..', '..', 'web', 'public', 'assets'))
PREVIEW = os.path.join(HERE, 'previews')
CONTENT = os.path.normpath(os.path.join(HERE, '..', '..', '..', 'web', 'public', 'content'))

# Triangles at full detail by the model's largest side (m): (up to, target, most). About the
# placeholders' median and 90th percentile in each class, so a model from Blender costs what the
# primitive models cost today (docs/design/objects.md, "Budgets").
TRIANGLES = [(0.35, 1500, 3000), (1.0, 4000, 8000), (2.5, 8000, 16000), (math.inf, 12000, 24000)]
# Draw calls per model (each its own in the sun's shadow map too): placeholders take 1-3.
DRAWS = (3, 4)
# Extensions the game's loader can read (no Draco: its decoder isn't shipped).
EXTENSIONS = {'KHR_materials_emissive_strength', 'KHR_mesh_quantization', 'KHR_texture_transform',
              'KHR_texture_basisu', 'EXT_meshopt_compression', 'KHR_materials_ior', 'KHR_materials_specular'}
# Largest texture side (px).
TEXTURE_MAX = 1024
# The wall's face lies this far into a footprint's back tile (m): a wall-standing thing's back is there.
WALL = 0.07
# A house's floor is drawn 1 cm up: a flat part on the floor must reach this high (m) to show over it.
FLOOR_TOP = 0.012
# A use's clip longer than this (s) reads as slow ("about 1 s"); loops and ambient clips keep their own pace.
USE_CLIP = 3.0


def manifests(path):
    """[(path, manifest)] for a manifest and the packs it lists, depth first, in order."""
    with open(path) as f:
        m = json.load(f)
    out = [(path, m)]
    for pack in m.get('packs', []):
        out += manifests(os.path.normpath(os.path.join(os.path.dirname(path), pack)))
    return out


def read_glb(path):
    with open(path, 'rb') as f:
        data = f.read()
    magic, version, _length = struct.unpack_from('<4sII', data)
    if magic != b'glTF' or version != 2:
        raise ValueError('not a glTF 2.0 binary')
    gltf, binary, off = None, b'', 12
    while off < len(data):
        n, kind = struct.unpack_from('<II', data, off)
        chunk = data[off + 8:off + 8 + n]
        if kind == 0x4E4F534A:
            gltf = json.loads(chunk)
        elif kind == 0x004E4942:
            binary = chunk
        off += 8 + n
    return gltf, binary, len(data)


def image_size(blob):
    """(width, height) of a PNG or JPEG, or None."""
    if blob[:8] == b'\x89PNG\r\n\x1a\n':
        return struct.unpack('>II', blob[16:24])
    if blob[:2] == b'\xff\xd8':
        i = 2
        while i + 9 < len(blob):
            marker, size = blob[i + 1], struct.unpack('>H', blob[i + 2:i + 4])[0]
            if 0xC0 <= marker <= 0xCF and marker not in (0xC4, 0xC8, 0xCC):
                h, w = struct.unpack('>HH', blob[i + 5:i + 9])
                return w, h
            i += 2 + size
    return None


def node_matrices(gltf, pose=None):
    """(mesh, world matrix (column-major 4x4 list), moving part, node) per mesh node, for the default scene.
    The moving part is the nearest node above it (or itself) that an animation moves, or None. `pose`
    ({node: {'scale': …}}) puts nodes elsewhere than at rest (a part grown, `grown`)."""
    def local(n, i):
        if 'matrix' in n:
            return n['matrix']
        n = {**n, **(pose or {}).get(i, {})}
        tx, ty, tz = n.get('translation', [0, 0, 0])
        qx, qy, qz, qw = n.get('rotation', [0, 0, 0, 1])
        sx, sy, sz = n.get('scale', [1, 1, 1])
        r = [1 - 2 * (qy * qy + qz * qz), 2 * (qx * qy + qz * qw), 2 * (qx * qz - qy * qw),
             2 * (qx * qy - qz * qw), 1 - 2 * (qx * qx + qz * qz), 2 * (qy * qz + qx * qw),
             2 * (qx * qz + qy * qw), 2 * (qy * qz - qx * qw), 1 - 2 * (qx * qx + qy * qy)]
        return [r[0] * sx, r[1] * sx, r[2] * sx, 0, r[3] * sy, r[4] * sy, r[5] * sy, 0,
                r[6] * sz, r[7] * sz, r[8] * sz, 0, tx, ty, tz, 1]

    def mul(a, b):
        return [sum(a[k * 4 + r] * b[c * 4 + k] for k in range(4)) for c in range(4) for r in range(4)]

    out = []
    nodes = gltf.get('nodes', [])
    scene = gltf.get('scenes', [{}])[gltf.get('scene', 0)]
    moved = moved_nodes(gltf)

    def walk(i, parent, part):
        m = mul(parent, local(nodes[i], i))
        part = i if i in moved else part
        if 'mesh' in nodes[i]:
            out.append((nodes[i]['mesh'], m, part, i))
        for c in nodes[i].get('children', []):
            walk(c, m, part)
    for i in scene.get('nodes', []):
        walk(i, [1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1], None)
    return out


def moved_nodes(gltf):
    """The nodes the file's animations move."""
    return {c['target']['node'] for a in gltf.get('animations', []) for c in a['channels'] if 'node' in c['target']}


def first_value(gltf, binary, accessor, last=False):
    """The first (or `last`) element of a float accessor (a clip's first or last key)."""
    acc = gltf['accessors'][accessor]
    if acc.get('componentType') != 5126:
        return None
    view = gltf['bufferViews'][acc['bufferView']]
    n = {'SCALAR': 1, 'VEC3': 3, 'VEC4': 4}[acc['type']]
    at = (acc['count'] - 1) * (view.get('byteStride') or 4 * n) if last else 0
    return struct.unpack_from(f'<{n}f', binary, view.get('byteOffset', 0) + acc.get('byteOffset', 0) + at)


def values(gltf, binary, accessor):
    """Every element of a float accessor (a clip's keys), as tuples."""
    acc = gltf['accessors'][accessor]
    view = gltf['bufferViews'][acc['bufferView']]
    n = {'SCALAR': 1, 'VEC3': 3, 'VEC4': 4}[acc['type']]
    stride = view.get('byteStride') or 4 * n
    at = view.get('byteOffset', 0) + acc.get('byteOffset', 0)
    return [struct.unpack_from(f'<{n}f', binary, at + k * stride) for k in range(acc['count'])]


def even(gltf, binary, sampler):
    """Whether a sampler moves at an even speed (each key as far on from the last, per second)."""
    times = [t[0] for t in values(gltf, binary, sampler['input'])]
    keys = values(gltf, binary, sampler['output'])
    if len(keys) < 3 or times[-1] <= times[0]:
        return False
    rate = [(b - a) / (times[-1] - times[0]) for a, b in zip(keys[0], keys[-1])]
    return all(abs(k[i] - keys[0][i] - rate[i] * (t - times[0])) <= 1e-3 for k, t in zip(keys, times) for i in range(len(rate)))


def named_finish(name):
    """A material named for its finish (`stone@matte`, `tunic@fabric#body`) maps itself to it (models.ts
    `namedFinish`; a tint slot `#body`, `#skin` or `#hair` in the name aside)."""
    head, at, tail = re.sub(r'#(body|skin|hair)(?=@|$)', '', name).rpartition('@')
    return {'finish': f'material.finish.{tail}'} if head and at and tail.isalpha() and tail.islower() else None


def uses(motions):
    """[(use, {clip, hold, delay, every, slot})] for each clip an entry's `motions.uses` plays (a clip,
    {clip, ...} or a list of them)."""
    return [(use, u if isinstance(u, dict) else {'clip': u})
            for use, value in (motions.get('uses') or {}).items() for u in (value if isinstance(value, list) else [value])]


def played(motions):
    """{clip: what plays it} for an entry's `motions`: `uses`, `loops`, `open`, `idle`, `night`, `growth`,
    `ride` (the clip whose part carries its users) and `clock` (`hour`, `minute`: hands set by the time)."""
    out = {}
    for use, u in uses(motions):
        out.setdefault(u['clip'], f'use {use!r}')
    for use, clip in (motions.get('loops') or {}).items():
        out.setdefault(clip, f'loop {use!r}')
    for kind in ('open', 'idle', 'night', 'growth', 'ride'):
        if motions.get(kind):
            out.setdefault(motions[kind], kind)
    for hand in ('hour', 'minute'):
        if (motions.get('clock') or {}).get(hand):
            out.setdefault(motions['clock'][hand], f'clock {hand}')
    return out


def loops(motions):
    """The clips of `motions` that go round, or from rest to rest (and so must end where they start): loops,
    `idle`, a use's clip that repeats (`every`) and a clock's hands (a whole turn)."""
    clock = motions.get('clock') or {}
    return (set((motions.get('loops') or {}).values()) | ({motions['idle']} if motions.get('idle') else set())
            | {u['clip'] for _, u in uses(motions) if u.get('every')} | {clock[h] for h in ('hour', 'minute') if clock.get(h)})


def motions_errors(motions):
    """What's wrong with the fields of an entry's `motions` (objectMotion.ts): a use's `delay` and `hold` are
    seconds (0 or more), `every` more than 0, `slot` a slot number; `clock` names its hands' clips."""
    errors = []
    for use, u in uses(motions):
        for key in ('hold', 'delay'):
            if key in u and not (isinstance(u[key], (int, float)) and u[key] >= 0):
                errors.append(f'`motions.uses` {use!r}: {key} {u[key]!r} is no number of seconds')
        if 'every' in u and not (isinstance(u['every'], (int, float)) and u['every'] > 0):
            errors.append(f'`motions.uses` {use!r}: every {u["every"]!r} is no number of seconds above 0')
        if 'slot' in u and not (isinstance(u['slot'], int) and not isinstance(u['slot'], bool) and u['slot'] >= 0):
            errors.append(f'`motions.uses` {use!r}: slot {u["slot"]!r} is no slot (0, 1, ...)')
        unknown = set(u) - {'clip', 'hold', 'delay', 'every', 'slot'}
        if unknown:
            errors.append(f'`motions.uses` {use!r}: unknown {", ".join(sorted(unknown))}')
    clock = motions.get('clock')
    if clock is not None:
        if not isinstance(clock, dict) or not (clock.get('hour') or clock.get('minute')) or set(clock) - {'hour', 'minute', 'at'}:
            errors.append('`motions.clock` names its hands\' clips: {"hour", "minute", "at"}')
        elif 'at' in clock and not isinstance(clock['at'], (int, float)):
            errors.append(f'`motions.clock.at` {clock["at"]!r} is no time (minutes past twelve)')
    return errors


def check_clips(gltf, binary, entry):
    """(errors, warnings) for a file's animations: moving parts turned or slid by node clips
    (docs/design/objects.md, "Moving parts"), each starting at rest, each played by the entry's `motions`."""
    errors, warnings = [], []
    animations, nodes = gltf.get('animations', []), gltf.get('nodes', [])
    rest = {'rotation': [0, 0, 0, 1], 'translation': [0, 0, 0], 'scale': [1, 1, 1]}
    for a in animations:
        name = a.get('name', '')
        for c in a['channels']:
            path, node = c['target'].get('path'), c['target'].get('node')
            if path not in rest or node is None:
                errors.append(f'clip {name!r} animates {path} (only a node\'s rotation, translation or scale)')
                continue
            first = first_value(gltf, binary, a['samplers'][c['sampler']]['output'])
            at_rest = nodes[node].get(path, rest[path])
            if first is None or any(abs(first[k] - at_rest[k]) > 1e-3 for k in range(len(at_rest))):
                errors.append(f'clip {name!r} starts {nodes[node].get("name", node)} away from where the file has it (the first key is the rest pose)')
    # (A part inside one that moves, a lever on a door, moves with it and by its own clips too.)
    names = {a.get('name', '') for a in animations}
    motions = entry.get('motions') or {}
    errors += motions_errors(motions)
    plays = played(motions)
    for clip, by in sorted(plays.items()):
        if clip not in names:
            errors.append(f'`motions` plays {clip!r} ({by}), which is no clip of the file')
    for name in sorted(names - set(plays)):
        warnings.append(f'clip {name!r} plays for no use (name it in the entry\'s `motions`)')
    # A use's clip plays as the use starts: one much longer than about 1 s reads as slow. Loops, ambient,
    # night, open and growth clips keep their own pace (a merry-go-round's round, a clock's minute).
    uses = {c for c, by in plays.items() if by.startswith('use ')} - loops(motions)
    for a in animations:
        if a.get('name', '') in uses:
            length = max((first_value(gltf, binary, s['input'], last=True) or (0,))[0] for s in a['samplers'])
            if length > USE_CLIP:
                warnings.append(f'clip {a["name"]!r} takes {length:.1f} s, played by a use: keep a use\'s clip about 1 s '
                                f'(over {USE_CLIP:g} s, a loop or an ambient clip instead)')
    # A loop goes round from rest to rest: each channel ends where it starts (a turn, either sign).
    # A turn by a whole fraction of a circle (a fan's four blades a quarter round) loops if the part
    # looks the same turned, and a move at an even speed (G.scroll: a belt's slats) if it looks the same
    # moved: warnings to check that.
    for a in animations:
        if a.get('name', '') not in loops(motions):
            continue
        for c in a['channels']:
            out = a['samplers'][c['sampler']]['output']
            first, last = first_value(gltf, binary, out), first_value(gltf, binary, out, last=True)
            if first is None or last is None:
                continue
            node = nodes[c['target']['node']].get('name', c['target']['node'])
            if c['target'].get('path') == 'rotation':
                turned = math.degrees(2 * math.acos(min(1.0, abs(sum(x * y for x, y in zip(first, last))))))
                if turned < 0.1:
                    continue
                n = round(360 / turned)
                if 2 <= n <= 12 and abs(turned * n - 360) < 0.5:
                    warnings.append(f'loop {a["name"]!r} ends {node} turned {turned:.0f}° from where it starts: right only if it looks the same turned (1/{n} of a circle)')
                    continue
            elif max(abs(x - y) for x, y in zip(first, last)) <= 1e-3:
                continue
            elif c['target'].get('path') == 'translation' and even(gltf, binary, a['samplers'][c['sampler']]):
                moved = math.dist(first, last)
                warnings.append(f'loop {a["name"]!r} ends {node} moved {moved * 100:.0f} cm from where it starts: right only if it looks the same moved (a belt\'s slats one slat on, its ends hidden)')
                continue
            errors.append(f'loop {a["name"]!r} ends {node} away from where it starts ({c["target"]["path"]}): a loop goes from rest to rest (G.spin, G.rock, G.scroll)')
    return errors, warnings


def part_turn(p):
    """A placeholder part's turn as rows (row j: where its own axis j goes), as models.ts `partBox` turns it:
    Babylon's Quaternion.FromEulerAngles(x, y, z), yaw y, pitch x, roll z."""
    rx, ry, rz = (math.radians(a) for a in p.get('rotate', [0, 0, 0]))
    cy, sy_, cp, sp, cr, sr = math.cos(ry / 2), math.sin(ry / 2), math.cos(rx / 2), math.sin(rx / 2), math.cos(rz / 2), math.sin(rz / 2)
    x = cy * sp * cr + sy_ * cp * sr
    y = sy_ * cp * cr - cy * sp * sr
    z = cy * cp * sr - sy_ * sp * cr
    w = cy * cp * cr + sy_ * sp * sr
    return [[1 - 2 * (y * y + z * z), 2 * (x * y + z * w), 2 * (z * x - y * w)],
            [2 * (x * y - z * w), 1 - 2 * (z * z + x * x), 2 * (y * z + x * w)],
            [2 * (z * x + y * w), 2 * (y * z - x * w), 1 - 2 * (y * y + x * x)]]


def part_box(p):
    """A placeholder part's exact bounds (lo, hi) in game axes, turned (models.ts `partBox`, which this
    must match): a box by its corners, a cylinder or cone by its rings' corners (its segments), a sphere,
    blob, capsule, torus or lathe by the round shape's own extent."""
    m = part_turn(p)
    sx, sy, sz = p['size']
    shape = p.get('shape', 'box')
    def ellipsoid(k, rx, ry, rz):
        return math.sqrt((m[0][k] * rx) ** 2 + (m[1][k] * ry) ** 2 + (m[2][k] * rz) ** 2)
    if shape in ('cylinder', 'cone'):
        # Babylon's CreateCylinder: rings of `segments` corners from angle 0, scaled in x and z.
        n = p.get('segments') or (16 if shape == 'cone' else 20)
        top = p.get('taper', 0 if shape == 'cone' else 1)
        points = [((r / 2) * sx * math.cos(2 * math.pi * j / n), y, (r / 2) * sz * math.sin(2 * math.pi * j / n))
                  for r, y in ((1, -sy / 2), (top, sy / 2)) for j in range(n)]
        lo = [min(sum(q[j] * m[j][k] for j in range(3)) for q in points) for k in range(3)]
        hi = [max(sum(q[j] * m[j][k] for j in range(3)) for q in points) for k in range(3)]
        return [p['at'][k] + lo[k] for k in range(3)], [p['at'][k] + hi[k] for k in range(3)]
    if shape in ('sphere', 'blob'):
        half = [ellipsoid(k, sx / 2, sy / 2, sz / 2) for k in range(3)]
    elif shape == 'capsule':
        # Babylon's CreateCapsule: `sy` tall overall, radius sx/2, then scaled to sz in z.
        half = [abs(m[1][k]) * max(0.0, sy / 2 - sx / 2) + ellipsoid(k, sx / 2, sx / 2, sz / 2) for k in range(3)]
    elif shape == 'torus':
        # A ring (sx - sy across) swept by a tube sy thick, then scaled to sz in z.
        q, ring, tube = sz / sx, (sx - sy) / 2, sy / 2
        half = [math.hypot(m[0][k] * ring, m[2][k] * ring * q) + ellipsoid(k, tube, tube, tube * q) for k in range(3)]
    elif shape == 'lathe':
        # Each profile point (radius, y) is a ring, scaled by the size.
        profile = p.get('profile') or [[0, -0.5], [0.5, -0.5], [0.5, 0.5], [0, 0.5]]
        lo = [min(m[1][k] * y * sy - r * math.hypot(m[0][k] * sx, m[2][k] * sz) for r, y in profile) for k in range(3)]
        hi = [max(m[1][k] * y * sy + r * math.hypot(m[0][k] * sx, m[2][k] * sz) for r, y in profile) for k in range(3)]
        return [p['at'][k] + lo[k] for k in range(3)], [p['at'][k] + hi[k] for k in range(3)]
    else:
        half = [sum(abs(m[j][k]) * p['size'][j] for j in range(3)) / 2 for k in range(3)]
    return [p['at'][k] - half[k] for k in range(3)], [p['at'][k] + half[k] for k in range(3)]


def placeholder_box(parts):
    """The placeholder's bounds (lo, hi) in game axes, from its parts' exact turned bounds (`part_box`)."""
    lo, hi = [math.inf] * 3, [-math.inf] * 3
    for p in parts:
        plo, phi = part_box(p)
        lo = [min(lo[k], plo[k]) for k in range(3)]
        hi = [max(hi[k], phi[k]) for k in range(3)]
    return lo, hi


def content_objects():
    """{model key: {'footprint', 'layer', 'outside'}} for the content's objects (web/public/content), for
    the wall checks: where the wall's face is (a footprint's back, `WALL` in) and what stands against one."""
    found, categories = {}, {}
    for root, _dirs, files in os.walk(CONTENT):
        for name in sorted(files):
            if not name.endswith('.json'):
                continue
            try:
                with open(os.path.join(root, name)) as f:
                    data = json.load(f)
            except (OSError, ValueError):
                continue
            if not isinstance(data, dict):
                continue
            for c in data.get('buyCategories') or []:
                categories[c.get('id')] = c
            for o in data.get('objects') or []:
                if isinstance(o, dict) and o.get('model'):
                    found.setdefault(o['model'], o)
    out = {}
    for key, o in found.items():
        c = categories.get(o.get('category'), {})
        group = next((g for g in c.get('groups') or [] if g.get('id') == o.get('group')), {})
        out[key] = {'footprint': o.get('footprint', [1, 1]), 'layer': o.get('layer', 'floor'),
                    'outside': group.get('outside', c.get('outside', False))}
    return out


def accessor(gltf, binary, index):
    """Every element of an accessor (floats, or integers read as the spec says: normalised if flagged),
    as tuples; None if it can't be read here (compressed, sparse)."""
    acc = gltf['accessors'][index]
    if 'bufferView' not in acc or acc.get('sparse'):
        return None
    view = gltf['bufferViews'][acc['bufferView']]
    if view.get('extensions'):
        return None
    n = {'SCALAR': 1, 'VEC2': 2, 'VEC3': 3, 'VEC4': 4}[acc['type']]
    fmt, size, top = {5126: ('f', 4, None), 5120: ('b', 1, 127), 5121: ('B', 1, 255), 5122: ('h', 2, 32767),
                      5123: ('H', 2, 65535), 5125: ('I', 4, None)}[acc['componentType']]
    stride = view.get('byteStride') or size * n
    at = view.get('byteOffset', 0) + acc.get('byteOffset', 0)
    out = [struct.unpack_from(f'<{n}{fmt}', binary, at + k * stride) for k in range(acc['count'])]
    if acc.get('normalized') and top:
        out = [tuple(max(v / top, -1.0) for v in e) for e in out]
    return out


def grown(gltf, binary, held=()):
    """{node: {'scale': …}}: the parts a clip scales up from rest (G.grow: darts, a pot thrown, water), at
    their largest key, so what is only small or flattened at rest is measured as it shows. Not by the clips
    `held` names (`open`, `night`: a state of its own, a sofa bed pulled out; the placeholder is at rest)."""
    nodes, out = gltf.get('nodes', []), {}
    for a in gltf.get('animations', []):
        if a.get('name') in held:
            continue
        for c in a['channels']:
            node = c['target'].get('node')
            if c['target'].get('path') != 'scale' or node is None:
                continue
            rest = nodes[node].get('scale', [1, 1, 1])
            best = max(values(gltf, binary, a['samplers'][c['sampler']]['output']), key=lambda k: sum(abs(v) for v in k))
            if any(abs(b) > 1.5 * abs(r) + 1e-6 for b, r in zip(best, rest)):
                if node not in out or sum(map(abs, best)) > sum(map(abs, out[node]['scale'])):
                    out[node] = {'scale': list(best)}
    return out


def bounds(gltf, pose=None):
    """(lo, hi, {part: (lo, hi)}): the file's bounds in game axes (glTF x -> game -x, Babylon's loader),
    from each primitive's box through its node, and each moving part's (None: the still body)."""
    accessors = gltf.get('accessors', [])
    lo, hi, parts = [math.inf] * 3, [-math.inf] * 3, {}
    for mesh_index, m, part, _node in node_matrices(gltf, pose):
        plo, phi = parts.setdefault(part, ([math.inf] * 3, [-math.inf] * 3))
        for prim in gltf['meshes'][mesh_index]['primitives']:
            pos = accessors[prim['attributes']['POSITION']]
            for cx in (pos['min'][0], pos['max'][0]):
                for cy in (pos['min'][1], pos['max'][1]):
                    for cz in (pos['min'][2], pos['max'][2]):
                        q = [-(m[0] * cx + m[4] * cy + m[8] * cz + m[12]), m[1] * cx + m[5] * cy + m[9] * cz + m[13],
                             m[2] * cx + m[6] * cy + m[10] * cz + m[14]]
                        for k in range(3):
                            lo[k], hi[k] = min(lo[k], q[k]), max(hi[k], q[k])
                            plo[k], phi[k] = min(plo[k], q[k]), max(phi[k], q[k])
    return lo, hi, parts


def flattened_in_view(gltf, binary, opaque):
    """[(part, share of it in view)] for the parts flattened to nothing at rest (a node scaled under 0.01
    along an axis, G.grow from 0.001): it still shows as a flat shape, if it's 3 cm across or more, where its
    flattened plane isn't inside something opaque (`opaque(material index)`), as the dartboard's flights did as crosses on its face. Each
    of its points is looked from along the flattened axis both ways: inside a solid, the nearest face either
    way is one leaving it (its normal, from the winding, pointing on)."""
    nodes = gltf.get('nodes', [])
    moved = moved_nodes(gltf)
    flat = {i: min(range(3), key=lambda k: abs(nodes[i].get('scale', [1, 1, 1])[k])) for i in moved
            if 'mesh' in nodes[i] and min(abs(v) for v in nodes[i].get('scale', [1, 1, 1])) < 0.01}
    if not flat:
        return []
    placed = node_matrices(gltf)
    def apply(m, v):
        return [m[0] * v[0] + m[4] * v[1] + m[8] * v[2] + m[12], m[1] * v[0] + m[5] * v[1] + m[9] * v[2] + m[13],
                m[2] * v[0] + m[6] * v[1] + m[10] * v[2] + m[14]]
    def det(m):
        return (m[0] * (m[5] * m[10] - m[9] * m[6]) - m[4] * (m[1] * m[10] - m[9] * m[2]) + m[8] * (m[1] * m[6] - m[5] * m[2]))
    # Every opaque triangle of the model at rest: (a, edge1, edge2, normal), with the node it belongs to.
    tris = []
    for mesh_index, m, _part, node in placed:
        sign = 1 if det(m) >= 0 else -1
        for prim in gltf['meshes'][mesh_index]['primitives']:
            if prim.get('mode', 4) != 4 or not opaque(prim.get('material')):
                continue
            pos = accessor(gltf, binary, prim['attributes']['POSITION'])
            idx = accessor(gltf, binary, prim['indices']) if 'indices' in prim else [(k,) for k in range(len(pos or []))]
            if pos is None or idx is None:
                return []
            world = [apply(m, v) for v in pos]
            for k in range(0, len(idx) - 2, 3):
                a, b, c = world[idx[k][0]], world[idx[k + 1][0]], world[idx[k + 2][0]]
                e1 = [b[j] - a[j] for j in range(3)]
                e2 = [c[j] - a[j] for j in range(3)]
                nrm = [sign * (e1[1] * e2[2] - e1[2] * e2[1]), sign * (e1[2] * e2[0] - e1[0] * e2[2]), sign * (e1[0] * e2[1] - e1[1] * e2[0])]
                tris.append((node, a, e1, e2, nrm))
    def cast(p, d, faces):
        """The nearest face each way along d from p (Möller-Trumbore): {1: …, -1: …}, each (distance, whether
        it is a face leaving a solid that way)."""
        near = {1: (math.inf, False), -1: (math.inf, False)}
        for _n, a, e1, e2, nrm in faces:
            h = [d[1] * e2[2] - d[2] * e2[1], d[2] * e2[0] - d[0] * e2[2], d[0] * e2[1] - d[1] * e2[0]]
            det_ = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2]
            if abs(det_) < 1e-12:
                continue
            s = [p[j] - a[j] for j in range(3)]
            u = (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]) / det_
            if u < 0 or u > 1:
                continue
            q = [s[1] * e1[2] - s[2] * e1[1], s[2] * e1[0] - s[0] * e1[2], s[0] * e1[1] - s[1] * e1[0]]
            v = (d[0] * q[0] + d[1] * q[1] + d[2] * q[2]) / det_
            if v < 0 or u + v > 1:
                continue
            t = (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]) / det_
            way = 1 if t > 0 else -1
            if 1e-4 < abs(t) < near[way][0]:
                near[way] = (abs(t), (nrm[0] * d[0] + nrm[1] * d[1] + nrm[2] * d[2]) * way > 0)
        return near

    out = []
    for node, axis in sorted(flat.items()):
        m = next(mm for _mi, mm, _p, n in placed if n == node)
        d = m[axis * 4:axis * 4 + 3]
        length = math.sqrt(sum(v * v for v in d)) or 1
        d = [v / length for v in d]
        points = []
        for prim in gltf['meshes'][nodes[node]['mesh']]['primitives']:
            pos = accessor(gltf, binary, prim['attributes']['POSITION'])
            if pos is None:
                return out
            points += [apply(m, v) for v in pos]
        if not points:
            continue
        # Its flat shape's two sides (across its plane, along its own main lines): one under 3 cm across, or
        # a sliver under 1 cm (a stream's end, a painting's edge before it spreads), hardly shows.
        u = [0, 1, 0] if abs(d[1]) < 0.9 else [1, 0, 0]
        u = [u[k] - d[k] * sum(u[j] * d[j] for j in range(3)) for k in range(3)]
        u = [v / math.sqrt(sum(w * w for w in u)) for v in u]
        w = [d[1] * u[2] - d[2] * u[1], d[2] * u[0] - d[0] * u[2], d[0] * u[1] - d[1] * u[0]]
        flat2 = [(sum(q[k] * u[k] for k in range(3)), sum(q[k] * w[k] for k in range(3))) for q in points]
        mu = [sum(c[k] for c in flat2) / len(flat2) for k in range(2)]
        cxx = sum((c[0] - mu[0]) ** 2 for c in flat2)
        cyy = sum((c[1] - mu[1]) ** 2 for c in flat2)
        cxy = sum((c[0] - mu[0]) * (c[1] - mu[1]) for c in flat2)
        angle = 0.5 * math.atan2(2 * cxy, cxx - cyy)
        axes = [(math.cos(angle), math.sin(angle)), (-math.sin(angle), math.cos(angle))]
        side = sorted(max(c[0] * a[0] + c[1] * a[1] for c in flat2) - min(c[0] * a[0] + c[1] * a[1] for c in flat2) for a in axes)
        if side[0] < 0.01 or side[1] < 0.03:
            continue
        points = points[::max(1, len(points) // 24)]
        others = [t for t in tris if t[0] != node]
        shown = 0
        for p in points:
            # Hidden inside a solid (the nearest face either way along its flattened axis leaves one), or,
            # lying level, under something (the game looks from above: glTF +y; a shower head's rain).
            near = cast(p, d, others)
            if near[1][1] and near[-1][1]:
                continue
            if abs(d[1]) > 0.7 and cast(p, [0, 1, 0], others)[1][0] < math.inf:
                continue
            shown += 1
        if shown / len(points) > 0.25:
            out.append((nodes[node].get('name', node), shown / len(points)))
    return out


# The finishes that are see-through (`alpha` under 1, the glass), filled as the manifests are read.
SEE_THROUGH = set()


def check(entry, finishes, path, far=None, facts=None):
    """(errors, warnings, summary) for one model entry and its file (and its far version's, at `far`);
    `facts`: its content's (`content_objects`)."""
    errors, warnings, info = check_file(entry, finishes, path, facts)
    summary = f'{info["size"] / 1024:6.1f} KB {info["tris"]:6d} tris {info["draws"]} draws {info["materials"]:2d} materials  ' \
              + ' x '.join(f'{d:.2f}' for d in info['dims']) + ' m'
    if info['clips']:
        summary += f'  {info["moving"]} moving parts, {info["clips"]} clips'
    if far:
        # Its far version (`far`): the same model, still, with far fewer triangles.
        try:
            far_errors, far_warnings, far_info = check_file({k: v for k, v in entry.items() if k not in ('motions', 'paint')}, finishes, far)
        except (OSError, ValueError, KeyError, IndexError) as err:
            far_errors, far_warnings, far_info = [f'cannot read {far}: {err}'], [], None
        if far_info:
            if far_info['clips']:
                far_errors.append('has clips: a far version stands still')
            if far_info['tris'] > info['tris'] / 2:
                far_warnings.append(f'{far_info["tris"]} triangles, over half the full model\'s {info["tris"]} (aim for a quarter)')
            # It stands where the model stands: decimating can drop the lowest points and lift it off the floor.
            if abs(far_info['lo'][1] - info['lo'][1]) > 0.005:
                far_errors.append(f'its lowest point is {(far_info["lo"][1] - info["lo"][1]) * 100:+.1f} cm off the full model\'s: '
                                  'a far version stands on the floor as the model does (G.far_of, or far= keeps the floor)')
            if any(abs(a - b) > max(0.05 * b, 0.02) for a, b in zip(far_info['dims'], info['dims'])):
                far_warnings.append(' x '.join(f'{d:.2f}' for d in far_info['dims']) + ' m, the full model ' + ' x '.join(f'{d:.2f}' for d in info['dims']) + ' m')
            summary += f'  (far: {far_info["tris"]} tris)'
        errors += [f'far: {e}' for e in far_errors]
        # (What it says of the placeholder, it says of the full model too.)
        warnings += [f'far: {w}' for w in far_warnings if 'placeholder' not in w]
    return errors, warnings, summary


def check_file(entry, finishes, path, facts=None):
    """(errors, warnings, numbers) for one model entry and a file of it; `facts`: its content's (`content_objects`)."""
    errors, warnings = [], []
    gltf, binary, size = read_glb(path)
    required = set(gltf.get('extensionsRequired', []))
    for ext in sorted(required - EXTENSIONS):
        errors.append(f'requires {ext}, which the game cannot read')
    if gltf.get('skins'):
        errors.append('has a skin: placed things are drawn as thin instances, without bones')
    if any(prim.get('targets') for mesh in gltf.get('meshes', []) for prim in mesh['primitives']):
        errors.append('has morph targets: placed things are drawn as thin instances, without them')
    if gltf.get('cameras'):
        warnings.append('has cameras; the game drops them')
    clip_errors, clip_warnings = check_clips(gltf, binary, entry)
    errors += clip_errors
    warnings += clip_warnings
    # The parts the player may colour (`paint`: a material's name, or {material, label}): materials of the file.
    names = {m.get('name', '') for m in gltf.get('materials', [])}
    for slot in entry.get('paint', []):
        name = slot.get('material') if isinstance(slot, dict) else slot
        if isinstance(slot, dict) and not slot.get('label'):
            errors.append(f'`paint` part {name!r} has no label')
        if name not in names:
            errors.append(f'`paint` names {name!r}, which is no material of the file')
    if 'KHR_lights_punctual' in gltf.get('extensionsUsed', []):
        warnings.append('has lights; the game drops them (a lamp\'s light is the content\'s `light`)')

    accessors, materials = gltf.get('accessors', []), gltf.get('materials', [])
    overrides = entry.get('materials', {})
    # Mirror glass (a material with `mirror`): flat, on one part (models.ts `fileMirror`).
    glass = {}
    for name, ov in overrides.items():
        if ov.get('mirror') not in (None, 'front', 'back'):
            errors.append(f'material {name!r}: `mirror` is front or back, not {ov["mirror"]!r}')
        elif ov.get('mirror') and name not in names:
            errors.append(f'material {name!r} is mirror glass, but no material of the file')
        elif ov.get('mirror') and ov.get('finish') in SEE_THROUGH:
            warnings.append(f'mirror glass {name!r} is see-through ({ov["finish"]}): where nothing reflects (Medium, Low) '
                            'it shows what is behind it; map it to gloss, as the placeholders\' mirrors')
        glow = ov.get('glow')
        if glow is not None and not (isinstance(glow, str) and len(glow) == 7 and glow[0] == '#' and all(c in '0123456789abcdefABCDEF' for c in glow[1:])):
            errors.append(f'material {name!r}: `glow` is a colour, #rrggbb, not {glow!r}')
    lo, hi = [math.inf] * 3, [-math.inf] * 3
    tris, groups = 0, set()
    # Per problem, the materials that have it.
    flags = {k: set() for k in ('unmapped', 'double', 'blend', 'emissive', 'uvs')}
    # Draw calls: one per finish group, the still body's and each moving part's apart.
    for mesh_index, m, part, _node in node_matrices(gltf):
        for prim in gltf['meshes'][mesh_index]['primitives']:
            if prim.get('mode', 4) != 4:
                errors.append('a primitive that is not triangles')
                continue
            pos = accessors[prim['attributes']['POSITION']]
            tris += (accessors[prim['indices']]['count'] if 'indices' in prim else pos['count']) // 3
            # Bounds: the accessor's box through the node matrix, then glTF x -> game -x (Babylon's loader).
            for cx in (pos['min'][0], pos['max'][0]):
                for cy in (pos['min'][1], pos['max'][1]):
                    for cz in (pos['min'][2], pos['max'][2]):
                        p = [m[0] * cx + m[4] * cy + m[8] * cz + m[12], m[1] * cx + m[5] * cy + m[9] * cz + m[13],
                             m[2] * cx + m[6] * cy + m[10] * cz + m[14]]
                        p[0] = -p[0]
                        lo = [min(lo[k], p[k]) for k in range(3)]
                        hi = [max(hi[k], p[k]) for k in range(3)]
            mat = materials[prim['material']] if 'material' in prim else {'name': ''}
            name = mat.get('name', '')
            ov = overrides.get(name, named_finish(name) or overrides.get('*'))
            if ov and ov.get('mirror'):
                box = glass.setdefault(name, [[math.inf] * 3, [-math.inf] * 3, part])
                for k in range(3):
                    box[0][k] = min(box[0][k], sum(m[j * 4 + k] * pos['min' if m[j * 4 + k] >= 0 else 'max'][j] for j in range(3)) + m[12 + k])
                    box[1][k] = max(box[1][k], sum(m[j * 4 + k] * pos['max' if m[j * 4 + k] >= 0 else 'min'][j] for j in range(3)) + m[12 + k])
                if box[2] != part:
                    errors.append(f'mirror glass {name!r} is on two parts: one material per mirror')
            textured = any(k in mat.get('pbrMetallicRoughness', {}) for k in ('baseColorTexture', 'metallicRoughnessTexture')) or any(
                k in mat for k in ('normalTexture', 'occlusionTexture', 'emissiveTexture'))
            if 'TEXCOORD_0' in prim['attributes'] and (not textured or (ov and ov.get('finish'))):
                flags['uvs'].add(name)
            if ov and ov.get('hide'):
                continue
            if ov and ov.get('finish'):
                finish = ov['finish']
                if finish not in finishes:
                    errors.append(f'material {name!r} maps to {finish}, which is no finish in the manifests')
                groups.add((part, 'plain' if finishes.get(finish) else finish))
                continue
            groups.add((part, f'file:{name}'))
            flags['unmapped'].add(name)
            if mat.get('doubleSided'):
                flags['double'].add(name)
            if mat.get('alphaMode') == 'BLEND':
                flags['blend'].add(name)
            if any(mat.get('emissiveFactor', [0, 0, 0])):
                flags['emissive'].add(name)
    for name, (glo, ghi, part) in sorted(glass.items()):
        extent = sorted(ghi[k] - glo[k] for k in range(3))   # not `size`: that is the file's, for the summary
        if extent[0] > 0.1 * extent[1]:
            warnings.append(f'mirror glass {name!r} isn\'t flat ({extent[0] * 100:.1f} cm thick, {extent[1] * 100:.0f} cm across): the reflection lies on its box\'s thinnest side')
    for flag, text in (('unmapped', 'not mapped to a finish in the entry\'s `materials` (each its own draw call and look)'),
                       ('double', 'double-sided (every back face drawn too)'),
                       ('blend', 'see-through (BLEND: drawn after everything else, unsorted between instances)'),
                       ('emissive', 'glowing by day too (emissive; map it to a finish, the lamp lights it at night)'),
                       ('uvs', 'carrying UVs nothing reads (export without them)')):
        if flags[flag]:
            warnings.append(f'materials {text}: ' + ', '.join(sorted(flags[flag])))

    # Units and origin: on the floor, centred on the footprint (models.ts lines files up by neither).
    scale = 1 if entry.get('fit') else entry.get('scale', 1)
    lo, hi = [v * scale for v in lo], [v * scale for v in hi]
    dims = [hi[k] - lo[k] for k in range(3)]
    # As it shows: the parts a clip grows (hidden or small at rest: darts, a pot thrown) grown, for what
    # is compared with the placeholder (sizes, centre, the wall).
    motions = entry.get('motions') or {}
    slo, shi, sparts = bounds(gltf, grown(gltf, binary, {motions.get('open'), motions.get('night')}))
    slo, shi = [v * scale for v in slo], [v * scale for v in shi]
    sdims = [shi[k] - slo[k] for k in range(3)]
    layer = (facts or {}).get('layer', 'floor')
    if not entry.get('fit') and not entry.get('offset'):
        # Centred on the footprint, or where its placeholder is (a wall bed at the back of its tiles).
        plo, phi = placeholder_box(entry['placeholder']) if entry.get('placeholder') else ([0] * 3, [0] * 3)
        if lo[1] < -0.005:
            errors.append(f'reaches {-lo[1] * 100:.1f} cm below the floor (origin on the floor, y = 0)')
        elif lo[1] > 0.005:
            warnings.append(f'floats {lo[1] * 100:.1f} cm above its origin (fine only on a wall or ceiling)')
        for k, axis in ((0, 'x'), (2, 'z')):
            c = (slo[k] + shi[k]) / 2
            if abs(c) > 0.02 and abs(c - (plo[k] + phi[k]) / 2) > 0.02:
                warnings.append(f'off centre by {c * 100:.1f} cm in {axis}: centred on (x {(slo[0] + shi[0]) / 2:.3f}, depth {(slo[2] + shi[2]) / 2:.3f}), '
                                f'its placeholder on ({(plo[0] + phi[0]) / 2:.3f}, {(plo[2] + phi[2]) / 2:.3f}) (origin at the footprint centre, or its placeholder\'s)')
        # Against a wall: the wall's face is `WALL` into the footprint's back tile. A wall thing, or a floor
        # thing indoors whose placeholder stands there, has its back on it, the model's and the placeholder's.
        # (A rug lies under the wall: it may reach in.)
        if facts and (layer == 'wall' or (layer == 'floor' and not facts['outside'])):
            face = -facts['footprint'][1] / 2 + WALL
            if slo[2] < face - 0.01:
                warnings.append(f'reaches {(face - slo[2]) * 100:.1f} cm into the wall (its back at depth {slo[2]:.3f}, the wall\'s face at {face:.2f})')
            if entry.get('placeholder'):
                if plo[2] < face - 0.01:
                    warnings.append(f'its placeholder reaches {(face - plo[2]) * 100:.1f} cm into the wall (its back at depth {plo[2]:.3f}, the wall\'s face at {face:.2f})')
                if plo[2] <= face + 0.03 and slo[2] > max(plo[2], face) + 0.02:
                    warnings.append(f'stands {(slo[2] - max(plo[2], face)) * 100:.1f} cm off the wall, where its placeholder stands against it '
                                    f'(its back at depth {slo[2]:.3f}, the wall\'s face at {face:.2f}; README trap "Against a wall, on the floor")')
        # Thin on the floor: a house's floor hides what lies under `FLOOR_TOP` (a mat, a belt), the whole
        # model or a moving part (as it shows, grown).
        if layer in ('floor', 'rug') and lo[1] <= 0.005:
            flat = [(None, shi[1])] if shi[1] < FLOOR_TOP else []
            flat += [(part, b[1][1] * scale) for part, b in sparts.items() if part is not None and b[1][1] * scale < FLOOR_TOP]
            for part, top in flat:
                what = 'it' if part is None else f'moving part {gltf["nodes"][part].get("name", part)}'
                warnings.append(f'{what} reaches only {top * 100:.1f} cm up: a house\'s floor is drawn 1 cm up and hides it (make it reach {FLOOR_TOP * 100:.1f} cm)')
    if entry.get('placeholder') and not entry.get('fit'):
        plo, phi = placeholder_box(entry['placeholder'])
        for k, axis in enumerate('xyz'):
            want = phi[k] - plo[k]
            if want > 0.05 and abs(sdims[k] - want) > 0.1 * want:
                warnings.append(f'{axis} size {sdims[k]:.2f} m, its placeholder {want:.2f} m (only the fallback, drawn if the file fails to load, '
                                'has the placeholder\'s size: clicking follows the drawn model, the contact shadow the footprint)')
    # Parts flattened to nothing at rest whose flat shape shows (README trap "Grown from nothing").
    def opaque(index):
        mat = materials[index] if index is not None else {'name': ''}
        ov = overrides.get(mat.get('name', ''), named_finish(mat.get('name', '')) or overrides.get('*'))
        if ov and ov.get('hide'):
            return False
        return ov['finish'] not in SEE_THROUGH if ov and ov.get('finish') else mat.get('alphaMode') != 'BLEND'
    for name, share in flattened_in_view(gltf, binary, opaque):
        warnings.append(f'{name} lies flattened at rest with {share:.0%} of it outside anything opaque, so its flat shape shows: '
                        'put its origin inside something opaque (README trap "Grown from nothing")')

    # Budgets.
    side = max(dims)
    target, most = next((t, m) for up, t, m in TRIANGLES if side <= up)
    if tris > most:
        errors.append(f'{tris} triangles, at most {most} for a model {side:.2f} m across (aim for {target})')
    elif tris > target:
        warnings.append(f'{tris} triangles, aim for {target} for a model {side:.2f} m across (at most {most})')
    if len(groups) > DRAWS[1]:
        errors.append(f'{len(groups)} draw calls, at most {DRAWS[1]} (one per finish group: map materials to finishes)')
    elif len(groups) > DRAWS[0]:
        warnings.append(f'{len(groups)} draw calls, aim for {DRAWS[0]} at most')
    for i, img in enumerate(gltf.get('images', [])):
        if 'bufferView' in img:
            view = gltf['bufferViews'][img['bufferView']]
            start = view.get('byteOffset', 0)
            wh = image_size(binary[start:start + view['byteLength']])
        else:
            wh = None
        if wh and max(wh) > TEXTURE_MAX:
            errors.append(f'texture {i} is {wh[0]}x{wh[1]}, at most {TEXTURE_MAX} px a side')
        warnings.append(f'texture {i} ({img.get("name", img.get("uri", "embedded"))}): a texture of its own, prefer the game\'s finishes')

    info = {'size': size, 'tris': tris, 'draws': len(groups), 'materials': len(materials), 'dims': dims, 'lo': lo,
            'clips': len(gltf.get('animations', [])), 'moving': len(moved_nodes(gltf))}
    return errors, warnings, info


def main(argv):
    keys = [a for a in argv if a.startswith('model.')]
    stack = manifests(os.path.join(ASSETS, 'manifest.json'))
    entries, finishes, stale = {}, {}, {}
    facts = content_objects()
    for path, m in stack:
        for key, entry in m['entries'].items():
            if entry.get('type') == 'model':
                # A pack's entry replaces the entry, keeping its placeholder and `vary` unless it gives its own
                # (registry.ts `extended`): a copy of the placeholder must follow the original.
                old = entries.get(key, (None, None))[0]
                if old and old.get('placeholder') and entry.get('placeholder') and old['placeholder'] != entry['placeholder']:
                    stale[key] = os.path.relpath(path, ASSETS)
                if old:
                    entry = {**{k: old[k] for k in ('placeholder', 'vary') if k in old}, **entry}
                entries[key] = (entry, path)
            elif entry.get('type') == 'material' and key.startswith('material.finish.'):
                finishes[key] = not entry.get('texture') and entry.get('alpha', 1) >= 1
                if entry.get('alpha', 1) < 1:
                    SEE_THROUGH.add(key)
    failed = 0
    checked = []
    for key, (entry, manifest) in sorted(entries.items()):
        url = entry.get('url', '')
        if (keys and key not in keys) or not url.lower().endswith('.glb') or '://' in url:
            continue
        path = os.path.normpath(os.path.join(os.path.dirname(manifest), url))
        far = os.path.normpath(os.path.join(os.path.dirname(manifest), entry['far'])) if entry.get('far') else None
        try:
            errors, warnings, summary = check(entry, finishes, path, far, facts.get(key))
        except (OSError, ValueError, KeyError, IndexError) as err:
            errors, warnings, summary = [f'cannot read {path}: {err}'], [], ''
        if key in stale:
            warnings.append(f'its placeholder in {stale[key]} differs from the one it overrides (copy the current one)')
        failed += bool(errors)
        checked.append(path)
        print(f'{"FAIL" if errors else "ok  "} {key:28s} {summary}')
        for e in errors:
            print(f'       error: {e}')
        for w in warnings:
            print(f'       warning: {w}')
    print(f'{len(checked)} models, {failed} failed')
    if '--preview' in argv:
        preview(checked)
    return 1 if failed else 0


def preview(paths):
    """Inside Blender: re-imports each file and renders it next to the build's previews."""
    try:
        import bpy
    except ImportError:
        print('--preview needs Blender: Blender --background --python check_glb.py -- --preview')
        return
    sys.path.insert(0, HERE)
    import gamekit as G
    for path in paths:
        G.reset()
        bpy.ops.import_scene.gltf(filepath=path)
        G.preview(os.path.join(PREVIEW, 'rt_' + os.path.splitext(os.path.basename(path))[0] + '.png'))


if __name__ == '__main__':
    code = main(sys.argv[1:])
    if 'bpy' not in sys.modules:
        sys.exit(code)

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


def node_matrices(gltf):
    """(mesh, world matrix (column-major 4x4 list), moving part) per mesh node, for the default scene.
    The moving part is the nearest node above it (or itself) that an animation moves, or None."""
    def local(n):
        if 'matrix' in n:
            return n['matrix']
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
        m = mul(parent, local(nodes[i]))
        part = i if i in moved else part
        if 'mesh' in nodes[i]:
            out.append((nodes[i]['mesh'], m, part))
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


def placeholder_box(parts):
    """The placeholder's bounds (lo, hi) in game axes, from its parts' boxes as models.ts `partBox`."""
    lo, hi = [math.inf] * 3, [-math.inf] * 3
    for p in parts:
        rx, ry, rz = (math.radians(a) for a in p.get('rotate', [0, 0, 0]))
        # Babylon's Quaternion.FromEulerAngles(x, y, z): yaw y, pitch x, roll z; matrix rows are the turned axes.
        cy, sy_, cp, sp, cr, sr = math.cos(ry / 2), math.sin(ry / 2), math.cos(rx / 2), math.sin(rx / 2), math.cos(rz / 2), math.sin(rz / 2)
        x = cy * sp * cr + sy_ * cp * sr
        y = sy_ * cp * cr - cy * sp * sr
        z = cy * cp * sr - sy_ * sp * cr
        w = cy * cp * cr + sy_ * sp * sr
        m = [[1 - 2 * (y * y + z * z), 2 * (x * y + z * w), 2 * (z * x - y * w)],
             [2 * (x * y - z * w), 1 - 2 * (z * z + x * x), 2 * (y * z + x * w)],
             [2 * (z * x + y * w), 2 * (y * z - x * w), 1 - 2 * (y * y + x * x)]]
        for k in range(3):
            half = sum(abs(m[j][k]) * p['size'][j] for j in range(3)) / 2
            lo[k] = min(lo[k], p['at'][k] - half)
            hi[k] = max(hi[k], p['at'][k] + half)
    return lo, hi


# The finishes that are see-through (`alpha` under 1, the glass), filled as the manifests are read.
SEE_THROUGH = set()


def check(entry, finishes, path, far=None):
    """(errors, warnings, summary) for one model entry and its file (and its far version's, at `far`)."""
    errors, warnings, info = check_file(entry, finishes, path)
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
            if any(abs(a - b) > max(0.05 * b, 0.02) for a, b in zip(far_info['dims'], info['dims'])):
                far_warnings.append(' x '.join(f'{d:.2f}' for d in far_info['dims']) + ' m, the full model ' + ' x '.join(f'{d:.2f}' for d in info['dims']) + ' m')
            summary += f'  (far: {far_info["tris"]} tris)'
        errors += [f'far: {e}' for e in far_errors]
        # (What it says of the placeholder, it says of the full model too.)
        warnings += [f'far: {w}' for w in far_warnings if 'placeholder' not in w]
    return errors, warnings, summary


def check_file(entry, finishes, path):
    """(errors, warnings, numbers) for one model entry and a file of it."""
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
    for mesh_index, m, part in node_matrices(gltf):
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
    if not entry.get('fit') and not entry.get('offset'):
        # Centred on the footprint, or where its placeholder is (a wall bed at the back of its tiles).
        plo, phi = placeholder_box(entry['placeholder']) if entry.get('placeholder') else ([0] * 3, [0] * 3)
        if lo[1] < -0.005:
            errors.append(f'reaches {-lo[1] * 100:.1f} cm below the floor (origin on the floor, y = 0)')
        elif lo[1] > 0.005:
            warnings.append(f'floats {lo[1] * 100:.1f} cm above its origin (fine only on a wall or ceiling)')
        for k, axis in ((0, 'x'), (2, 'z')):
            c = (lo[k] + hi[k]) / 2
            if abs(c) > 0.02 and abs(c - (plo[k] + phi[k]) / 2) > 0.02:
                warnings.append(f'off centre by {c * 100:.1f} cm in {axis} (origin at the footprint centre, or its placeholder\'s)')
    if entry.get('placeholder') and not entry.get('fit'):
        plo, phi = placeholder_box(entry['placeholder'])
        for k, axis in enumerate('xyz'):
            want = phi[k] - plo[k]
            if want > 0.05 and abs(dims[k] - want) > 0.1 * want:
                warnings.append(f'{axis} size {dims[k]:.2f} m, its placeholder {want:.2f} m (fallback, picking and shadow follow the placeholder)')

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

    info = {'size': size, 'tris': tris, 'draws': len(groups), 'materials': len(materials), 'dims': dims,
            'clips': len(gltf.get('animations', [])), 'moving': len(moved_nodes(gltf))}
    return errors, warnings, info


def main(argv):
    keys = [a for a in argv if a.startswith('model.')]
    stack = manifests(os.path.join(ASSETS, 'manifest.json'))
    entries, finishes, stale = {}, {}, {}
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
            errors, warnings, summary = check(entry, finishes, path, far)
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

"""
Checks a built character set against the contract (docs/characters-format.md) and the runtime's
names; exits non-zero on any violation. Runs after every build:

    python3 -B tools/characters/prim/check.py [set dir]
"""

import json
import os
import re
import sys

import numpy as np

from contract import CHANNELS, GARMENTS, HAIR_STYLES, REQUIRED_PARTS, VISEMES, hair_parts
from head import head_frame
from rig import BASE_STAGE, BONES, CLIP_BONES, RUNTIME_BONES, STAGES, TARGET_HEIGHT

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..', '..'))
WEB = os.path.join(ROOT, 'web', 'src')
BUDGET = {'body': 4500, 'lod.body': 1100}
HAIR_BUDGET = 3000
SHELL_BUDGET = 1500
CUT_KEYS = {'top': {'sleeve', 'hem'}, 'bottom': {'waist', 'leg'}, 'shoes': {'shoe', 'colour'}}

errors = []


def fail(msg):
    errors.append(msg)


def ts_list(src, name):
    """The string items of `const NAME ... = [ ... ]` in TypeScript source."""
    m = re.search(rf'{name}[^=]*=\s*\[(.*?)\]', src, re.S)
    return re.findall(r"'([^']+)'", m.group(1)) if m else []


def runtime_names():
    face = open(os.path.join(WEB, 'render/babylon/characters/face.ts')).read()
    units = ts_list(face, 'const UNITS')
    moods = face[face.index('export const MOODS'):face.index('/** Idle micro-movements')]
    mood_names = set(re.findall(r'\b([a-z]+[A-Z][A-Za-z]+)\s*:', moods))
    visemes = ts_list(open(os.path.join(WEB, 'voice/visemes.ts')).read(), 'export const VISEMES')
    outfit = open(os.path.join(WEB, 'render/babylon/characters/outfit.ts')).read()
    garments = {}
    for fam in ('male', 'female'):
        ids = []
        for table in ('TOPS', 'BOTTOMS'):
            block = re.search(rf'const {table}[^=]*=\s*\{{(.*?)\n\}};', outfit, re.S).group(1)
            ids += re.findall(r"'([a-z]+\.[A-Za-z]+)'", re.search(rf'(?<![a-z]){fam}:\s*\[(.*?)\]', block, re.S).group(1))
        ids += ts_list(outfit, 'const SHOE_PARTS')
        garments[fam] = set(ids)
    hair = ts_list(open(os.path.join(WEB, 'game/household.ts')).read(), 'export const HAIR_STYLES')
    return units, mood_names, visemes, garments, hair


def resolve(name, channels):
    """A MOODS name is covered by the channel itself or by its Left / Right pair."""
    return name in channels or (f'{name}Left' in channels and f'{name}Right' in channels)


def read_part(buf, e, n_comp, key, dtype):
    return np.frombuffer(buf, dtype, e['vertices'] * n_comp, e[key]).reshape(-1, n_comp)


def main(argv):
    out = argv[0] if argv else os.path.join(ROOT, 'web', 'public', 'assets', 'characters')
    rig = json.load(open(os.path.join(out, 'rig.json')))
    units, moods, visemes, garments, hair_styles = runtime_names()

    # Skeleton.
    if rig.get('version') != 4:
        fail('rig.json is not version 4')
    bones = rig['bones']
    if bones != BONES:
        fail('bone list differs from rig.py')
    for b in RUNTIME_BONES:
        if b not in bones:
            fail(f'missing runtime bone {b}')
    if any(p >= i for i, p in enumerate(rig['parents'])):
        fail('a parent comes after its child')
    if rig['clipBones'] != CLIP_BONES or any(not b.startswith('eye_') for b in bones[CLIP_BONES:]):
        fail('clip bones: the eyes must come last')
    q = np.array(rig['sourceRestQ']).reshape(-1, 4)
    if not np.allclose(q, [0, 0, 0, 1]):
        fail('sourceRestQ is not identity')

    # Channels.
    ch = rig['morphs']['channels']
    if ch[:15] != visemes or visemes != VISEMES:
        fail('visemes are not first, in the voice engine order')
    for u in units:
        if u not in ch:
            fail(f'missing face unit {u}')
    for m in moods:
        if not resolve(m, ch) and m not in ('mouthPressLeft',):
            fail(f'MOODS name {m} has no channel')
    if ch != CHANNELS:
        fail('channel list differs from contract.py')
    if hair_styles != HAIR_STYLES:
        fail('hairstyles differ from household.ts')

    moved = set()
    for fam, body in rig['bodies'].items():
        buf = open(os.path.join(out, body['mesh']), 'rb').read()
        parts = body['parts']
        need = REQUIRED_PARTS + hair_parts(fam)
        for p in need:
            if p not in parts:
                fail(f'{fam}: missing part {p}')
        # Garments: the runtime's ids, every cut, a lod copy of every shell.
        if set(body['garments']) != garments[fam] or list(body['garments']) != GARMENTS[fam]:
            fail(f'{fam}: garment ids differ from outfit.ts / contract.py')
        for gid, cut in body['garments'].items():
            miss = CUT_KEYS[gid.split('.')[0]] - set(cut)
            if miss:
                fail(f'{fam}: {gid} lacks {sorted(miss)}')
        for p in parts:
            if re.match(r'^(top|bottom|shoes)\.', p):
                if p not in body['garments']:
                    fail(f'{fam}: shell {p} has no garment entry')
                if f'lod.{p}' not in parts:
                    fail(f'{fam}: shell {p} has no lod copy')
        if body.get('bodyChunks') != {'body': []}:
            fail(f'{fam}: bodyChunks must be {{body: []}}')
        for name, e in parts.items():
            n = e['vertices']
            tris = e['indices'] // 3
            if n >= 65536:
                fail(f'{fam}:{name} has {n} vertices')
            pos = read_part(buf, e, 3, 'positions', '<f4')
            w = read_part(buf, e, 4, 'weights', np.uint8).astype(int)
            idx = np.frombuffer(buf, '<u2', e['indices'], e['index']).reshape(-1, 3)
            if not np.isfinite(pos).all():
                fail(f'{fam}:{name} has NaN positions')
            if (w.sum(1) != 255).any():
                fail(f'{fam}:{name} weights do not sum to 255')
            if idx.max() >= n:
                fail(f'{fam}:{name} index out of range')
            budget = BUDGET.get(name, HAIR_BUDGET if name.startswith('hair.') else SHELL_BUDGET if re.match(r'^(lod\.)?(top|bottom|shoes)\.', name) else None)
            if budget and tris > budget:
                fail(f'{fam}:{name} has {tris} triangles (budget {budget})')
            # Babylon front faces are clockwise: cross(b - a, c - a) points against the normals.
            nrm = read_part(buf, e, 4, 'normals', '<i2')[:, :3].astype(float)
            a, b, c = pos[idx[:, 0]], pos[idx[:, 1]], pos[idx[:, 2]]
            cr = np.cross(b - a, c - a)
            big = np.linalg.norm(cr, axis=1) > 1e-10
            wrong = ((cr * (nrm[idx[:, 0]] + nrm[idx[:, 1]] + nrm[idx[:, 2]])).sum(1) > 0)[big].mean() if big.any() else 0
            if wrong > 0.05:
                fail(f'{fam}:{name}: {wrong:.0%} of the faces wind the wrong way for Babylon')
            m = e.get('morph')
            if m:
                r = m['ranges']
                for k, c in enumerate(ch):
                    if r[k + 1] > r[k]:
                        moved.add(c)
        # Stages: every part, same topology, heights.
        for stage in STAGES:
            info = body if stage == BASE_STAGE else body['stages'].get(stage)
            if info is None:
                fail(f'{fam}: missing stage {stage}')
                continue
            target = TARGET_HEIGHT[fam][stage]
            if abs(info['height'] / target - 1) > 0.03:
                fail(f'{fam}.{stage}: height {info["height"]} is more than 3% off {target}')
            drop = info['seatDrop'] / info['height']
            if not 0.04 < drop < 0.12:
                fail(f'{fam}.{stage}: seatDrop {info["seatDrop"]} is implausible')
            if stage == BASE_STAGE:
                continue
            sbuf = open(os.path.join(out, info['mesh']), 'rb').read()
            if set(info['parts']) != set(parts):
                fail(f'{fam}.{stage}: parts differ from the base')
            for name, e in info['parts'].items():
                n = parts[name]['vertices']
                end = e['positions'] + n * 8
                if end > len(sbuf):
                    fail(f'{fam}.{stage}:{name} topology differs from the base')
        # Eyes in their sockets, hair off the skull (base stage).
        h = head_frame(fam, BASE_STAGE)
        if (h.sdf(h.eyes) >= 0).any():
            fail(f'{fam}: an eye centre is outside the skull')
        front = h.eyes + [0, 0, h.eye_r]
        if (h.sdf(front) <= 0).any():
            fail(f'{fam}: an eye does not show in front of the skull')
        for name, e in parts.items():
            if name.startswith('hair.'):
                pos = read_part(buf, e, 3, 'positions', '<f4')
                if (h.sdf(pos) < 0.002).any():
                    fail(f'{fam}:{name} comes within 2 mm of the skull')
    for c in ch:
        if c != 'sil' and c not in moved:
            fail(f'channel {c} moves nothing')

    if errors:
        for e in errors:
            print('FAIL', e)
        sys.exit(1)
    print(f'check: {out} ok ({len(rig["bodies"])} bodies, {len(ch)} channels)')


if __name__ == '__main__':
    main(sys.argv[1:])

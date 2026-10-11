"""
Writes the character set (rig.json version 4 and its binaries; docs/characters-format.md).

Each part in a family's `<family>.bin`: positions f32x3, normals i16x4 (w unused), joints u8x4,
weights u8x4 (summing to 255), fields i16x4 (x FIELD_SCALE), tone i16x4 (x TONE_SCALE), indices u16;
then, for parts with face morphs, `morphVerts` u16 (the vertices any channel moves), and per
channel in order the moved vertices' slots in that list (`morphSlots` u16), position deltas
(`morphPositions` i16x4, x MORPH_SCALE) and normal deltas (`morphNormals` i8x4, x NORMAL_SCALE);
`morph.ranges[c] .. ranges[c + 1]` are channel c's entries. No UVs.

Each life stage other than the base: `<family>.<stage>.bin`, per part int16 position deltas from
the base (x STAGE_SCALE) and int8 normals (x 127).

Meshes come in counter-clockwise from outside (prims.py); Babylon's front faces are clockwise in
its left-handed space, so the writer turns every triangle round.
"""

import json
import os

import numpy as np

from meshops import top4

FIELD_SCALE = 4000.0
TONE_SCALE = 4000.0
STAGE_SCALE = 5000.0
MORPH_SCALE = 30000.0
NORMAL_SCALE = 63.0
MORPH_EPS = 1e-5


def _unit(n):
    return n / np.maximum(np.linalg.norm(n, axis=1, keepdims=True), 1e-9)


def pack_morph(part, deltas, channels):
    """Sparse morph block of a part from per channel (dpos, dnor) dense deltas (see the docstring)."""
    n = len(part['pos'])
    moved = np.zeros(n, bool)
    used = {}
    for c in channels:
        if c not in deltas:
            continue
        dp, dn = deltas[c]
        m = (np.abs(dp).max(1) > MORPH_EPS) | (np.abs(dn).max(1) > MORPH_EPS * 100)
        if m.any():
            used[c] = m
            moved |= m
    if not moved.any():
        return None
    verts = np.where(moved)[0]
    slot_of = np.full(n, -1)
    slot_of[verts] = np.arange(len(verts))
    slots, dpos, dnor, ranges = [], [], [], [0]
    for c in channels:
        if c in used:
            sel = np.where(used[c])[0]
            dp, dn = deltas[c]
            slots.append(slot_of[sel])
            dpos.append(dp[sel])
            dnor.append(dn[sel])
            ranges.append(ranges[-1] + len(sel))
        else:
            ranges.append(ranges[-1])
    return dict(verts=verts, slots=np.concatenate(slots), dpos=np.concatenate(dpos), dnor=np.concatenate(dnor), ranges=ranges)


def write_parts(path, parts):
    """The family's main binary; returns the part table for rig.json."""
    blob = bytearray()
    table = {}
    for name, part in parts.items():
        n = len(part['pos'])
        assert n < 65536, name
        j, w = top4(part['dense'])
        wq = np.round(w * 255).astype(np.int64)
        wq[np.arange(n), np.argmax(wq, 1)] += 255 - wq.sum(1)
        nor = _unit(part['nor'])
        entry = {'vertices': n, 'indices': len(part['idx'])}

        def add(key, arr):
            nonlocal blob
            while len(blob) % 4:
                blob += b'\0'
            entry[key] = len(blob)
            blob += arr.tobytes()

        add('positions', part['pos'].astype('<f4'))
        add('normals', np.c_[np.round(nor * 32767), np.zeros(n)].astype('<i2'))
        add('joints', j.astype(np.uint8))
        add('weights', wq.astype(np.uint8))
        f = part.get('fields')
        f = np.zeros((n, 4)) if f is None else np.asarray(f, np.float64)
        add('fields', np.clip(np.round(f * FIELD_SCALE), -32767, 32767).astype('<i2'))
        t = part.get('tone')
        t = np.zeros((n, 4)) if t is None else np.asarray(t, np.float64)
        add('tone', np.clip(np.round(t * TONE_SCALE), -32767, 32767).astype('<i2'))
        # Clockwise for Babylon.
        add('index', part['idx'].reshape(-1, 3)[:, [0, 2, 1]].ravel().astype('<u2'))
        m = part.get('morph')
        if m is not None:
            e = len(m['slots'])
            entry['morph'] = {'count': len(m['verts']), 'ranges': [int(r) for r in m['ranges']]}
            add('morphVerts', np.asarray(m['verts']).astype('<u2'))
            add('morphSlots', np.asarray(m['slots']).astype('<u2'))
            add('morphPositions', np.clip(np.round(np.c_[m['dpos'], np.zeros(e)] * MORPH_SCALE), -32767, 32767).astype('<i2'))
            add('morphNormals', np.clip(np.round(np.c_[m['dnor'], np.zeros(e)] * NORMAL_SCALE), -127, 127).astype('i1'))
            for k in ('morphVerts', 'morphSlots', 'morphPositions', 'morphNormals'):
                entry['morph'][k] = entry.pop(k)
        entry['min'] = np.round(part['pos'].min(0), 4).tolist()
        entry['max'] = np.round(part['pos'].max(0), 4).tolist()
        table[name] = entry
    with open(path, 'wb') as fh:
        fh.write(bytes(blob))
    return table


def write_stage(path, base_parts, stage_parts):
    """A life stage: per part, int16 position deltas from the base (x STAGE_SCALE) and int8 normals."""
    blob = bytearray()
    table = {}
    for name, part in stage_parts.items():
        base = base_parts[name]
        n = len(part['pos'])
        assert n == len(base['pos']), name
        entry = {}

        def add(key, arr):
            nonlocal blob
            while len(blob) % 4:
                blob += b'\0'
            entry[key] = len(blob)
            blob += arr.tobytes()

        d = part['pos'] - base['pos']
        add('positions', np.clip(np.round(np.c_[d, np.zeros(n)] * STAGE_SCALE), -32767, 32767).astype('<i2'))
        add('normals', np.round(np.c_[_unit(part['nor']), np.zeros(n)] * 127).astype('i1'))
        table[name] = entry
    with open(path, 'wb') as fh:
        fh.write(bytes(blob))
    return table


def r4(a):
    return [round(float(v), 6) for v in np.asarray(a).ravel()]


def write_rig(out, rig, bodies):
    """rig.json: `rig` holds the shared keys, `bodies` the per family entries (already with part tables)."""
    data = dict(version=4, **rig, bodies=bodies)
    with open(os.path.join(out, 'rig.json'), 'w') as fh:
        json.dump(data, fh, separators=(',', ':'))

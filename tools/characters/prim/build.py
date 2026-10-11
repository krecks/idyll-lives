"""
Builds the residents' character set from code (rig.json version 4 and its binaries; see
docs/characters-format.md):

    python3 -B tools/characters/prim/build.py [out dir]      # default web/public/assets/characters

Per family and life stage it asks the head interface (head.py) where the face is, then builds the
body (body.py), the face (face.py: eyes, lids, lashes, brows, mouth and the morph channels), the
hair (hair.py) and the clothes (clothes.py), merges the face's skin pieces into the body, and
writes the base stage (young adult) whole and the other stages as deltas. Then check.py runs.
Everything is deterministic (seeded).

Module ownership: body.py / clothes.py (stream A), face.py / hair.py / prims.py (stream B); rig.py,
head.py (interface), contract.py, write.py, build.py and check.py are the contract.
"""

import os
import sys

import numpy as np
from scipy.spatial import cKDTree

from body import body_fields_and_tone, build_body
from clothes import build_clothes, garment_cuts
from contract import CHANNELS
from face import build_face, follow_mouth
from hair import build_hair
from head import head_frame
from prims import merge
from rig import BASE_STAGE, BONES, CLIP_BONES, FACE_BONES, FAMILIES, HEAD, NB, PARENTS, STAGES, leg_lengths, skeleton, source_pelvis
from write import FIELD_SCALE, MORPH_SCALE, NORMAL_SCALE, STAGE_SCALE, TONE_SCALE, pack_morph, r4, write_parts, write_rig, write_stage

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..', '..'))
OUT = os.path.join(ROOT, 'web', 'public', 'assets', 'characters')


def assemble(family, stage, cuts=None):
    """Every part of `family` at `stage` (no fields / tone / morphs yet), with the body's metadata."""
    head = head_frame(family, stage)
    b = build_body(family, stage, head)
    P = b['joints']
    f = build_face(family, stage, head)
    parts = dict(b['parts'])
    body = parts['body']
    n_skin = len(body['pos'])
    extras = [dict(m, arm=np.zeros(len(m['pos']))) for m in f['body']]
    parts['body'] = merge([body] + extras, keys=('pos', 'nor', 'dense', 'arm'))
    # The low-detail body has the eyelids too (they come first among the face's skin pieces).
    lids = [m for m in extras if m.get('skin_tone')]
    parts['lod.body'] = merge([parts['lod.body']] + lids, keys=('pos', 'nor', 'dense', 'arm'))
    parts.update(f['parts'])
    parts.update(build_hair(family, stage, head, P))
    if cuts is None:
        cuts = garment_cuts(family, P)
    parts.update(build_clothes(family, stage, head, P, parts['body'], cuts))
    return dict(parts=parts, meta=b['meta'], P=P, head=head, face=f, n_skin=n_skin, n_lod=len(b['parts']['lod.body']['pos']), cuts=cuts)


def skeleton_json(family, stage, meta):
    sk = skeleton(family, stage)
    return dict(
        height=round(float(sk['height']), 4),
        restT=r4(sk['restT']),
        inverseBind=r4(sk['inverseBind']),
        pelvisScale=round(float(sk['pelvisScale']), 5),
        seatDrop=round(meta['seatDrop'], 4),
        eyeY=round(meta['eyeY'], 4),
        grip=[round(v, 4) for v in meta['grip']],
    )


def build_family(family, out):
    base = assemble(family, BASE_STAGE)
    parts = base['parts']
    head = base['head']
    # Fields and tone (base stage only; the stages share them).
    for name in ('body', 'lod.body'):
        f, t = body_fields_and_tone(family, parts[name], base['P'], head)
        parts[name]['fields'] = f
        parts[name]['tone'] = t
    # The face's skin pieces: the lids take the tone of the skin they lie on, the lips keep theirs.
    for name, k in (('body', base['n_skin']), ('lod.body', base['n_lod'])):
        tone = parts[name]['tone']
        near = cKDTree(parts[name]['pos'][:k])
        for m in base['face']['body']:
            if name == 'lod.body' and not m.get('skin_tone'):
                continue
            n = len(m['pos'])
            # (The lightest of the skin round each lid vertex: not the shade deep in the socket.)
            tone[k:k + n] = tone[near.query(m['pos'], k=16)[1]].min(1) if m.get('skin_tone') else m['tone']
            k += n
    # Face morphs: the face's own parts, the body's lids (after its skin), the beard.
    morphs = dict(base['face']['morphs'])
    if 'body' in morphs:
        n_lids = sum(len(m['pos']) for m in base['face']['body'] if m.get('skin_tone'))
        face_d = morphs['body']
        for name, k, count in (('body', base['n_skin'], None), ('lod.body', base['n_lod'], n_lids)):
            n = len(parts[name]['pos'])
            d = {}
            for c, (dp, dn) in face_d.items():
                dp, dn = dp[:count], dn[:count]
                if not np.abs(dp).max() > 0:
                    continue
                fp, fn = np.zeros((n, 3)), np.zeros((n, 3))
                fp[k:k + len(dp)] = dp
                fn[k:k + len(dn)] = dn
                d[c] = (fp, fn)
            morphs[name] = d
    if 'hair.beard' in parts:
        morphs['hair.beard'] = follow_mouth(parts['hair.beard'], base['face']['follow'], head)
    for name, d in morphs.items():
        m = pack_morph(parts[name], d, CHANNELS)
        if m is not None:
            parts[name]['morph'] = m
    table = write_parts(os.path.join(out, f'{family}.bin'), parts)
    stages = {}
    for stage in STAGES:
        if stage == BASE_STAGE:
            continue
        st = assemble(family, stage, base['cuts'])
        assert set(st['parts']) == set(parts), (family, stage, set(st['parts']) ^ set(parts))
        mesh = f'{family}.{stage}.bin'
        t = write_stage(os.path.join(out, mesh), parts, st['parts'])
        stages[stage] = dict(mesh=mesh, **skeleton_json(family, stage, st['meta']), parts=t)
    entry = dict(
        mesh=f'{family}.bin',
        stage=BASE_STAGE,
        **skeleton_json(family, BASE_STAGE, base['meta']),
        skinRef=[1, 1, 1],
        garments=base['cuts'],
        bodyChunks={'body': []},
        parts=table,
        stages=stages,
    )
    return entry


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else OUT
    os.makedirs(out, exist_ok=True)
    bodies = {family: build_family(family, out) for family in FAMILIES}
    thigh, shin = leg_lengths()
    rig = dict(
        bones=BONES,
        parents=PARENTS,
        clipBones=CLIP_BONES,
        face={'bones': FACE_BONES},
        head=HEAD,
        fps=30,
        sourcePelvis=r4(source_pelvis()),
        sourceRestQ=[0, 0, 0, 1] * NB,
        reference={'body': 'male', 'thigh': round(thigh, 5), 'shin': round(shin, 5)},
        fieldScale=FIELD_SCALE,
        toneScale=TONE_SCALE,
        stageScale=STAGE_SCALE,
        morphs={'channels': CHANNELS, 'scale': MORPH_SCALE, 'normalScale': NORMAL_SCALE},
    )
    write_rig(out, rig, bodies)
    print(f'wrote {out}')
    import check
    check.main([out])


if __name__ == '__main__':
    main()

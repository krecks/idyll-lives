"""
The residents' skeleton: 33 bones, the names the runtime uses, and the joint positions of every
family (`male`, `female`) at every life stage.

Conventions (see docs/characters-format.md):
- Babylon's left-handed space, metres, y up, the character faces +z, its left is -x;
- every rest local rotation is the identity, so a bone's frame at rest is model space and a joint's
  local translation is just the offset from its parent's joint;
- the rest pose is an A-pose: arms 45 degrees down, legs straight and vertical;
- the clips store the first `CLIP_BONES` bones (the eyes come last and are posed procedurally).

`PROPORTIONS[family][stage]` holds the numbers each stage is built from (fractions of the height H
unless noted). Positions scale with H, then the stage's overrides (head size, hip height, shoulder
and hip width, arm length) apply.
"""

import numpy as np

BONES = [
    'root', 'pelvis', 'spine_01', 'spine_02', 'spine_03', 'neck_01', 'Head',
    'clavicle_l', 'upperarm_l', 'lowerarm_l', 'hand_l', 'thumb_01_l', 'thumb_02_l', 'fingers_01_l', 'fingers_02_l',
    'clavicle_r', 'upperarm_r', 'lowerarm_r', 'hand_r', 'thumb_01_r', 'thumb_02_r', 'fingers_01_r', 'fingers_02_r',
    'thigh_l', 'calf_l', 'foot_l', 'ball_l',
    'thigh_r', 'calf_r', 'foot_r', 'ball_r',
    'eye_l', 'eye_r',
]
PARENT = {
    'root': None, 'pelvis': 'root', 'spine_01': 'pelvis', 'spine_02': 'spine_01', 'spine_03': 'spine_02',
    'neck_01': 'spine_03', 'Head': 'neck_01', 'eye_l': 'Head', 'eye_r': 'Head',
}
for _s in 'lr':
    PARENT.update({
        f'clavicle_{_s}': 'spine_03', f'upperarm_{_s}': f'clavicle_{_s}', f'lowerarm_{_s}': f'upperarm_{_s}',
        f'hand_{_s}': f'lowerarm_{_s}', f'thumb_01_{_s}': f'hand_{_s}', f'thumb_02_{_s}': f'thumb_01_{_s}',
        f'fingers_01_{_s}': f'hand_{_s}', f'fingers_02_{_s}': f'fingers_01_{_s}',
        f'thigh_{_s}': 'pelvis', f'calf_{_s}': f'thigh_{_s}', f'foot_{_s}': f'calf_{_s}', f'ball_{_s}': f'foot_{_s}',
    })
B = {n: i for i, n in enumerate(BONES)}
PARENTS = [B[PARENT[n]] if PARENT[n] else -1 for n in BONES]
NB = len(BONES)
CLIP_BONES = 31
HEAD = B['Head']
FACE_BONES = ['eye_l', 'eye_r']

# Bones the runtime looks up by name (characters.ts, reactions.ts, data.ts).
RUNTIME_BONES = [
    'pelvis', 'spine_01', 'spine_02', 'spine_03', 'neck_01', 'Head', 'clavicle_l', 'clavicle_r', 'upperarm_l', 'upperarm_r',
    'lowerarm_l', 'lowerarm_r', 'hand_l', 'hand_r', 'thigh_l', 'thigh_r', 'calf_l', 'calf_r', 'foot_l', 'foot_r', 'eye_l', 'eye_r',
]

FAMILIES = ('male', 'female')
STAGES = ('baby', 'child', 'teen', 'youngAdult', 'adult', 'elder')
BASE_STAGE = 'youngAdult'
# The reference skeleton the clips are authored on (`sourcePelvis`, leg lengths for IK).
REFERENCE = ('male', BASE_STAGE)

# Rendered heights before this rig (m): the stages must stay within 3% so cribs, kids' beds, desks
# and seats still fit.
TARGET_HEIGHT = {
    'male': {'baby': 0.663, 'child': 1.128, 'teen': 1.512, 'youngAdult': 1.77, 'adult': 1.755, 'elder': 1.742},
    'female': {'baby': 0.675, 'child': 1.094, 'teen': 1.442, 'youngAdult': 1.67, 'adult': 1.672, 'elder': 1.661},
}

# Adult layout (fractions of H; left side, the right mirrors x).
ADULT = dict(
    hh=1 / 6.5,       # head height (chin to crown)
    hip=0.50,         # thigh joints
    shoulder=0.13,    # upperarm joint x (female 0.115)
    hip_w=0.054,      # thigh joint x (female 0.058)
    upper=0.165,      # upper arm length
    lower=0.145,      # forearm length (elbow to wrist)
    girth=1.0,        # limb radius factor
    waist=1.0,        # waist width factor
    neck_z=-0.012,
    belly=0.0,        # extra belly depth (fraction of H)
)

# Per family and stage: H (m) and the overrides of `ADULT`.
PROPORTIONS = {
    'male': {
        'baby': dict(H=0.67, hh=1 / 3.4, hip=0.38, shoulder=0.12, hip_w=0.075, upper=0.12, lower=0.105, girth=1.35, belly=0.03),
        'child': dict(H=1.13, hh=1 / 4.6, hip=0.45, shoulder=0.11, hip_w=0.06, upper=0.15, lower=0.13, girth=1.1),
        'teen': dict(H=1.51, hh=1 / 6.0, hip=0.50, shoulder=0.12, hip_w=0.054, upper=0.162, lower=0.142, girth=0.95),
        'youngAdult': dict(H=1.77),
        'adult': dict(H=1.76, waist=1.08, belly=0.006),
        'elder': dict(H=1.74, hh=1 / 6.4, hip=0.49, shoulder=0.13 * 0.95, waist=1.12, belly=0.01, neck_z=-0.002),
    },
    'female': {
        'baby': dict(H=0.67, hh=1 / 3.4, hip=0.38, shoulder=0.12, hip_w=0.075, upper=0.12, lower=0.105, girth=1.35, belly=0.03),
        'child': dict(H=1.09, hh=1 / 4.6, hip=0.45, shoulder=0.11, hip_w=0.062, upper=0.15, lower=0.13, girth=1.1),
        'teen': dict(H=1.44, hh=1 / 6.0, hip=0.50, shoulder=0.105, hip_w=0.058, upper=0.162, lower=0.142, girth=0.95),
        'youngAdult': dict(H=1.67, shoulder=0.115, hip_w=0.058),
        'adult': dict(H=1.66, shoulder=0.115, hip_w=0.058, waist=1.08, belly=0.006),
        'elder': dict(H=1.64, hh=1 / 6.4, hip=0.49, shoulder=0.115 * 0.95, hip_w=0.058, waist=1.12, belly=0.01, neck_z=-0.002),
    },
}


def proportions(family, stage):
    """The stage's numbers: `ADULT` with the family / stage overrides (H in metres)."""
    p = dict(ADULT)
    p.update(PROPORTIONS[family][stage])
    return p


def joints(family, stage):
    """Joint positions (NB, 3) in metres, model space at rest (identity rotations)."""
    p = proportions(family, stage)
    H, hh = p['H'], p['hh']
    J = {}
    chin = 1 - hh
    neck = chin - (0.27 if hh < 0.2 else 0.12) * hh
    hip = p['hip']
    pelvis = hip + 0.035 * (hip / 0.5)
    J['root'] = (0, 0, 0)
    J['pelvis'] = (0, pelvis, 0)
    span = neck - pelvis
    J['spine_01'] = (0, pelvis + 0.15 * span, -0.005)
    J['spine_02'] = (0, pelvis + 0.37 * span, -0.01)
    J['spine_03'] = (0, pelvis + 0.61 * span, -0.01)
    J['neck_01'] = (0, neck, p['neck_z'])
    J['Head'] = (0, chin + 0.06 * hh, 0)
    sh_y = neck - 0.02 * (hh / ADULT['hh']) ** 0.3
    d = np.array([-np.cos(np.radians(45)), -np.sin(np.radians(45)), 0])
    for s, sx in (('l', 1), ('r', -1)):
        M = np.array([sx, 1, 1])
        J[f'clavicle_{s}'] = (-0.012 * sx, neck - 0.015, 0.01)
        up = np.array([-p['shoulder'], sh_y, -0.005])
        el = up + p['upper'] * d
        wr = el + p['lower'] * d
        # Hands and feet are 1.15x real ones (the toy look), and relatively bigger on small people.
        hs = (p['upper'] / ADULT['upper']) ** 0.5
        k1 = wr + 0.055 * hs * d
        k2 = k1 + 0.03 * hs * d
        t1 = wr + np.array([0, -0.01, 0.02]) * hs
        t2 = t1 + 0.025 * hs * (d + np.array([0, 0, 1])) / np.sqrt(2)
        for name, v in (('upperarm', up), ('lowerarm', el), ('hand', wr), ('fingers_01', k1), ('fingers_02', k2), ('thumb_01', t1), ('thumb_02', t2)):
            J[f'{name}_{s}'] = tuple(v * M)
        hx = -p['hip_w'] * sx
        fs = (hip / ADULT['hip']) ** 0.5
        J[f'thigh_{s}'] = (hx, hip, 0)
        J[f'calf_{s}'] = (hx, hip * 0.56, 0.005)
        J[f'foot_{s}'] = (hx, hip * 0.096, -0.01)
        J[f'ball_{s}'] = (hx, 0.012 * fs, -0.01 + 0.095 * fs)
    # Eyes: sphere centres, head-relative.
    for s, sx in (('l', 1), ('r', -1)):
        J[f'eye_{s}'] = (-0.17 * hh * sx, chin + 0.44 * hh, 0.275 * hh)
    out = np.zeros((NB, 3))
    for n, v in J.items():
        out[B[n]] = np.asarray(v, np.float64)
    return out * H


def skeleton(family, stage):
    """Rest pose of a body: local translations, inverse bind matrices (column-major), joints."""
    J = joints(family, stage)
    restT = np.array([J[b] - (J[p] if p >= 0 else 0) for b, p in enumerate(PARENTS)])
    ibm = np.zeros((NB, 16))
    for b in range(NB):
        m = np.eye(4)
        m[:3, 3] = -J[b]
        ibm[b] = m.T.ravel()  # column-major
    ref = joints(*REFERENCE)
    return dict(
        joints=J,
        restT=restT,
        inverseBind=ibm,
        height=proportions(family, stage)['H'],
        pelvisScale=J[B['pelvis'], 1] / ref[B['pelvis'], 1],
    )


def source_pelvis():
    """The reference pelvis rest position (clip pelvis translations are relative to it)."""
    return joints(*REFERENCE)[B['pelvis']]


def leg_lengths():
    """Reference thigh and shin lengths (hip to knee, knee to ankle), for the clips' leg IK."""
    J = joints(*REFERENCE)
    return float(np.linalg.norm(J[B['calf_l']] - J[B['thigh_l']])), float(np.linalg.norm(J[B['foot_l']] - J[B['calf_l']]))


def mirror_bone(name):
    return name[:-1] + ('r' if name.endswith('_l') else 'l') if name.endswith(('_l', '_r')) else name

"""
Names the runtime and the generator agree on (docs/characters-format.md): morph channels, hairstyles,
garments and the parts every body has. `check.py` holds the build to them.
"""

# The 15 visemes in the voice engine's order (crates/voice/src/visemes.rs = web/src/voice/visemes.ts).
VISEMES = ['sil', 'PP', 'FF', 'TH', 'DD', 'kk', 'CH', 'SS', 'nn', 'RR', 'aa', 'E', 'I', 'O', 'U']

# ARKit face units (the subset face.ts uses: `UNITS` and the names in `MOODS`).
UNITS = [
    'browDownLeft', 'browDownRight', 'browInnerUp', 'browOuterUpLeft', 'browOuterUpRight',
    'cheekSquintLeft', 'cheekSquintRight', 'eyeBlinkLeft', 'eyeBlinkRight',
    'eyeLookDownLeft', 'eyeLookDownRight', 'eyeLookUpLeft', 'eyeLookUpRight',
    'eyeSquintLeft', 'eyeSquintRight', 'eyeWideLeft', 'eyeWideRight',
    'jawForward', 'jawOpen', 'mouthClose', 'mouthDimpleLeft', 'mouthDimpleRight',
    'mouthFrownLeft', 'mouthFrownRight', 'mouthFunnel', 'mouthLeft',
    'mouthLowerDownLeft', 'mouthLowerDownRight', 'mouthPressLeft', 'mouthPressRight', 'mouthPucker',
    'mouthRollLower', 'mouthRollUpper', 'mouthShrugLower', 'mouthShrugUpper',
    'mouthSmileLeft', 'mouthSmileRight', 'mouthStretchLeft', 'mouthStretchRight',
    'mouthUpperUpLeft', 'mouthUpperUpRight', 'noseSneerLeft', 'noseSneerRight',
]
CHANNELS = VISEMES + UNITS

# Hairstyles (household.ts HAIR_STYLES): each but `none` is a part `hair.<style>`; `hair.beard`
# is the facial hair of the families that have it.
HAIR_STYLES = ['short', 'crop', 'layered', 'tousled', 'sideSwept', 'afro', 'bob', 'long', 'ponytail', 'braid', 'bun', 'none']
BEARD_FAMILIES = ('male',)

# Garments per family, in the creator's order (outfit.ts TOPS / BOTTOMS / SHOE_PARTS).
GARMENTS = {
    'male': ['top.tee', 'top.vneck', 'top.polo', 'top.long', 'top.shirt', 'top.jacket', 'top.suit',
             'bottom.trousers', 'bottom.shorts', 'bottom.suit',
             'shoes.sneakers', 'shoes.trainers', 'shoes.boots', 'shoes.dress'],
    'female': ['top.tee', 'top.tank', 'top.blouse', 'top.long', 'top.shirt', 'top.jacket',
               'bottom.trousers', 'bottom.shorts', 'bottom.capri', 'bottom.skirt',
               'shoes.sneakers', 'shoes.trainers', 'shoes.boots', 'shoes.dress'],
}

# Parts every body has (the face parts are characters.ts FACE_PARTS).
FACE_PARTS = ['eyes', 'brows', 'lashes', 'teeth', 'tongue']
REQUIRED_PARTS = ['body', 'lod.body'] + FACE_PARTS


def hair_parts(family):
    return [f'hair.{s}' for s in HAIR_STYLES if s != 'none'] + (['hair.beard'] if family in BEARD_FAMILIES else [])

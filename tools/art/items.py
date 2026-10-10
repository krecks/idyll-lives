# Food, dishes and cooking tools from primitive parts (original, CC0; no downloads): the models of content
# `items` (`item.*`: ingredients, a pot of stew, plates, dirty plates) and the props drawn on a station while
# a resident cooks there (`tool.*`: a chopping board, a pan, food on the grill).
#
# Conventions as in catalog.py: 1 unit = 1 m, origin at the middle of the base, front +Z. Items stand on a
# top (or are carried): the renderer lifts them to the height of what they're on.
#
# Run: python3 -B items.py   (writes web/public/assets/items/manifest.json, listed in the base manifest's
# `packs`).
import json
import os

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '../../web/public/assets/items/manifest.json')

GLOSS = 'material.finish.gloss'
SATIN = 'material.finish.satin'
MATTE = 'material.finish.matte'
METAL = 'material.finish.metal'
WOOD = 'material.finish.wood'

CHINA = '#F1EEE6'
STEEL = '#8E949B'
DARK = '#3A3D42'


def part(shape, size, at, color, material=SATIN, **extra):
    p = {'shape': shape, 'size': size, 'at': at, 'color': color, 'material': material}
    p.update(extra)
    return p


def plate(y=0.0, r=0.24):
    return [
        part('cylinder', [r, 0.016, r], [0, y + 0.008, 0], CHINA, GLOSS, taper=1.2, segments=20),
        part('cylinder', [r * 0.62, 0.004, r * 0.62], [0, y + 0.017, 0], '#E4DFD4', GLOSS, segments=20),
    ]


def pot(food, dirty=False):
    inside = '#5B4636' if dirty else food
    return [
        part('cylinder', [0.27, 0.15, 0.27], [0, 0.075, 0], DARK, METAL, segments=20),
        part('cylinder', [0.235, 0.012, 0.235], [0, 0.14 if not dirty else 0.05, 0], inside, SATIN, segments=20),
        part('box', [0.06, 0.02, 0.03], [0.165, 0.12, 0], DARK, METAL),
        part('box', [0.06, 0.02, 0.03], [-0.165, 0.12, 0], DARK, METAL),
    ] + ([] if dirty else [
        part('blob', [0.06, 0.03, 0.05], [0.04, 0.15, 0.03], '#D9772B', SATIN, noise=0.25),
        part('blob', [0.05, 0.03, 0.05], [-0.05, 0.15, -0.02], '#6BA33A', SATIN, noise=0.25),
    ])


def platter(dirty=False):
    out = [part('cylinder', [0.4, 0.02, 0.28], [0, 0.01, 0], CHINA, GLOSS, segments=24)]
    if dirty:
        return out + [part('cylinder', [0.24, 0.004, 0.16], [0.02, 0.022, 0], '#7A5A3C', MATTE, segments=16)]
    return out + [
        part('capsule', [0.045, 0.16, 0.045], [-0.08, 0.04, -0.04], '#8A4A2A', SATIN, rotate=[0, 20, 90]),
        part('capsule', [0.045, 0.16, 0.045], [-0.08, 0.04, 0.04], '#93512E', SATIN, rotate=[0, -10, 90]),
        part('cylinder', [0.1, 0.035, 0.1], [0.08, 0.035, 0.03], '#5E3A24', SATIN, segments=14),
        part('capsule', [0.05, 0.14, 0.05], [0.07, 0.04, -0.07], '#F2C230', SATIN, rotate=[0, 70, 90]),
    ]


def tray(food, dirty=False):
    out = [part('box', [0.38, 0.025, 0.28], [0, 0.0125, 0], STEEL, METAL, radius=0.01)]
    if dirty:
        return out + [part('cylinder', [0.2, 0.003, 0.14], [0, 0.027, 0], '#6E5338', MATTE, segments=12)]
    for i, (x, z) in enumerate([(-0.11, -0.07), (0, -0.07), (0.11, -0.07), (-0.11, 0.07), (0, 0.07), (0.11, 0.07)]):
        out.append(part('cylinder', [0.08, 0.025, 0.08], [x, 0.037, z], food if i % 2 else '#B5743A', SATIN, segments=12))
    return out


def on_plate(*food):
    return plate() + list(food)


ITEMS = {
    'item.ingredients': [
        part('box', [0.2, 0.22, 0.14], [0, 0.11, 0], '#C49A6C', MATTE, radius=0.01),
        part('blob', [0.1, 0.08, 0.08], [0.02, 0.24, 0], '#5FA33A', SATIN, noise=0.3),
        part('capsule', [0.035, 0.18, 0.035], [-0.06, 0.24, 0.02], '#E8772E', SATIN, rotate=[0, 0, 20]),
        part('sphere', [0.07, 0.07, 0.07], [0.06, 0.23, 0.03], '#D93A2B', SATIN, segments=10),
    ],
    'item.prepBowl': [
        part('cylinder', [0.24, 0.09, 0.24], [0, 0.045, 0], '#E9E4D8', GLOSS, taper=1.35, segments=20),
        part('blob', [0.07, 0.03, 0.06], [0.03, 0.09, 0.02], '#E8772E', SATIN, noise=0.3),
        part('blob', [0.07, 0.03, 0.06], [-0.04, 0.09, -0.02], '#6BA33A', SATIN, noise=0.3),
        part('blob', [0.05, 0.03, 0.05], [0.0, 0.095, -0.05], '#E8D9A0', SATIN, noise=0.3),
    ],
    'item.mealPot': pot('#B0562C'),
    'item.potDirty': pot('#B0562C', dirty=True),
    'item.tastingBoard': [
        part('box', [0.4, 0.03, 0.24], [0, 0.015, 0], '#9A6A3E', WOOD, radius=0.01),
        part('blob', [0.08, 0.04, 0.07], [-0.12, 0.05, 0], '#C8452E', SATIN, noise=0.2),
        part('blob', [0.08, 0.04, 0.07], [0, 0.05, 0.02], '#E9C46A', SATIN, noise=0.2),
        part('blob', [0.08, 0.04, 0.07], [0.12, 0.05, -0.01], '#5E8C3A', SATIN, noise=0.2),
    ],
    'item.grillPlatter': platter(),
    'item.platterDirty': platter(dirty=True),
    'item.pizza': [
        part('cylinder', [0.4, 0.02, 0.4], [0, 0.01, 0], '#A8784A', WOOD, segments=24),
        part('cylinder', [0.34, 0.025, 0.34], [0, 0.032, 0], '#E2B064', SATIN, segments=24),
        part('cylinder', [0.3, 0.008, 0.3], [0, 0.046, 0], '#C8452E', SATIN, segments=24),
        part('cylinder', [0.05, 0.006, 0.05], [0.06, 0.052, 0.05], '#F3E3B0', SATIN, segments=10),
        part('cylinder', [0.05, 0.006, 0.05], [-0.07, 0.052, 0.02], '#F3E3B0', SATIN, segments=10),
        part('cylinder', [0.04, 0.006, 0.04], [0.0, 0.052, -0.08], '#7C2A20', SATIN, segments=10),
    ],
    'item.bakingTray': tray('#D69A55'),
    'item.trayDirty': tray('#D69A55', dirty=True),
    'item.pieDish': [
        part('cylinder', [0.28, 0.06, 0.28], [0, 0.03, 0], '#E6E0D2', GLOSS, taper=1.2, segments=22),
        part('cylinder', [0.26, 0.03, 0.26], [0, 0.07, 0], '#D8A257', SATIN, segments=22),
        part('cylinder', [0.08, 0.006, 0.02], [0.04, 0.087, 0], '#B07A3A', SATIN),
    ],
    'item.mealPlate': on_plate(
        part('blob', [0.1, 0.05, 0.08], [0.02, 0.04, 0.01], '#B0562C', SATIN, noise=0.2),
        part('blob', [0.06, 0.04, 0.05], [-0.05, 0.035, -0.03], '#6BA33A', SATIN, noise=0.3),
    ),
    'item.pizzaSlice': on_plate(
        part('cone', [0.12, 0.02, 0.16], [0, 0.03, 0], '#E2B064', SATIN, segments=3, rotate=[90, 0, 0]),
    ),
    'item.treatPlate': on_plate(
        part('cylinder', [0.07, 0.025, 0.07], [-0.04, 0.03, 0], '#C98A45', SATIN, segments=12),
        part('cylinder', [0.07, 0.025, 0.07], [0.04, 0.03, 0.02], '#8A5530', SATIN, segments=12),
        part('cylinder', [0.07, 0.025, 0.07], [0.0, 0.055, -0.01], '#D69A55', SATIN, segments=12),
    ),
    'item.pieSlice': on_plate(
        part('box', [0.1, 0.045, 0.08], [0, 0.04, 0], '#D8A257', SATIN, radius=0.01, rotate=[0, 30, 0]),
    ),
    'item.sandwich': on_plate(
        part('box', [0.12, 0.02, 0.1], [0, 0.03, 0], '#E3C08A', SATIN, radius=0.01),
        part('box', [0.13, 0.012, 0.11], [0, 0.045, 0], '#6BA33A', SATIN, radius=0.004),
        part('box', [0.12, 0.02, 0.1], [0, 0.06, 0], '#E3C08A', SATIN, radius=0.01),
    ),
    'item.plateDirty': plate() + [
        part('cylinder', [0.13, 0.004, 0.09], [0.02, 0.02, 0.01], '#7A5A3C', MATTE, segments=12),
        part('box', [0.16, 0.006, 0.014], [0.0, 0.024, -0.04], STEEL, METAL, rotate=[0, 25, 0]),
    ],
}

TOOLS = {
    # Chopping at the counter.
    'tool.board': [
        part('box', [0.34, 0.025, 0.24], [0, 0.0125, 0], '#B88A57', WOOD, radius=0.008),
        part('box', [0.18, 0.006, 0.03], [0.06, 0.03, -0.07], '#C9CED4', METAL, rotate=[0, 20, 0]),
        part('blob', [0.07, 0.04, 0.06], [-0.06, 0.04, 0.03], '#E8772E', SATIN, noise=0.3),
        part('blob', [0.06, 0.04, 0.05], [0.02, 0.04, 0.05], '#6BA33A', SATIN, noise=0.3),
    ],
    # A pan on the stove.
    'tool.pan': [
        part('cylinder', [0.28, 0.05, 0.28], [0, 0.025, 0], DARK, METAL, segments=20),
        part('box', [0.2, 0.02, 0.03], [0, 0.04, 0.22], '#2B2D31', METAL),
        part('blob', [0.12, 0.03, 0.1], [0.02, 0.05, -0.01], '#B0562C', SATIN, noise=0.25),
        part('blob', [0.06, 0.03, 0.05], [-0.06, 0.05, 0.04], '#6BA33A', SATIN, noise=0.3),
    ],
    # Sausages and burgers on the grill.
    'tool.grillFood': [
        part('capsule', [0.045, 0.16, 0.045], [-0.08, 0.025, -0.05], '#8A4A2A', SATIN, rotate=[0, 0, 90]),
        part('capsule', [0.045, 0.16, 0.045], [-0.08, 0.025, 0.04], '#93512E', SATIN, rotate=[0, 0, 90]),
        part('cylinder', [0.1, 0.03, 0.1], [0.08, 0.015, 0.0], '#5E3A24', SATIN, segments=14),
    ],
}

manifest = {
    '$comment': 'Food, dishes and cooking tools (content `items` models, and the props drawn on a station while cooking), drawn from primitive parts by tools/art/items.py. Listed in the base manifest\'s packs.',
    'version': 1,
    'entries': {k: {'type': 'model', 'placeholder': v} for k, v in {**ITEMS, **TOOLS}.items()},
}
os.makedirs(os.path.dirname(OUT), exist_ok=True)
with open(OUT, 'w') as f:
    json.dump(manifest, f, indent=2)
    f.write('\n')
print('wrote', OUT, len(manifest['entries']))

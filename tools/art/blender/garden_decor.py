# Blender-built models for the garden decor props, replacing the primitive-part placeholders of
# `nature.py` (`model.garden.gnome`, `birdbath`, `fountain`, `lantern`, `wheelbarrow`, `boulder`).
#
# Sizes match the placeholders' bounding boxes so footprints, lights and the selection marker
# stay put. Convention: metres, origin at the footprint centre on the floor, front facing -Y in
# Blender (the glTF export's +Y up turns that into the game's +Z front).
#
# Run:  blender --background --python tools/art/blender/garden_decor.py
#       blender --background --python tools/art/blender/garden_decor.py -- --preview
import os
import sys

import bpy

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gamekit as G  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.abspath(os.path.join(HERE, '../../../web/public/assets/models/garden'))
PREVIEW = os.path.join(HERE, 'previews')

# palette (rebuilt after every reset, which drops the old materials)
def _palette():
    global STONE, STONE_DARK, STONE_WARM, WATER, MOSS, LEAF, LEAF_LIGHT, WOOD, WOOD_DARK
    global METAL, METAL_DARK, SKIN, NOSE, BEARD, TUNIC, HAT, SOIL, BIRD_BROWN, BEAK
    global FLOWER_PINK, FLOWER_YELLOW, FLOWER_WHITE, GLOW
    STONE = G.mat('stone', '#B9B2A6', 0.85)
    STONE_DARK = G.mat('stone_dark', '#8F887C', 0.9)
    STONE_WARM = G.mat('stone_warm', '#C6BCA9', 0.85)
    WATER = G.mat('water', '#6FA8C0', 0.12, alpha=0.85)
    MOSS = G.mat('moss', '#5E8A3C', 0.95)
    LEAF = G.mat('leaf', '#4E7A34', 0.9)
    LEAF_LIGHT = G.mat('leaf_light', '#6E9A44', 0.9)
    WOOD = G.mat('wood', '#9A7444', 0.75)
    WOOD_DARK = G.mat('wood_dark', '#6E5232', 0.8)
    METAL = G.mat('metal', '#5A6068', 0.45, 0.9)
    METAL_DARK = G.mat('metal_dark', '#2E3236', 0.5, 0.8)
    SKIN = G.mat('skin', '#F0C6A4', 0.55)
    NOSE = G.mat('nose', '#E8A08A', 0.6)
    BEARD = G.mat('beard', '#EDE8DE', 0.9)
    TUNIC = G.mat('tunic', '#3E63B0', 0.7)
    HAT = G.mat('hat', '#C8322A', 0.65)
    SOIL = G.mat('soil', '#4A3826', 0.98)
    BIRD_BROWN = G.mat('bird_brown', '#7A5A40', 0.85)
    BEAK = G.mat('beak', '#E8A23A', 0.6)
    FLOWER_PINK = G.mat('flower_pink', '#E85A9A', 0.7)
    FLOWER_YELLOW = G.mat('flower_yellow', '#F2C14E', 0.7)
    FLOWER_WHITE = G.mat('flower_white', '#F4F0F2', 0.7)
    GLOW = G.mat('glow', '#FFE6A8', 0.4, emission=1.5)


STONE = STONE_DARK = STONE_WARM = WATER = MOSS = LEAF = LEAF_LIGHT = None
WOOD = WOOD_DARK = METAL = METAL_DARK = SKIN = NOSE = BEARD = TUNIC = HAT = None
SOIL = BIRD_BROWN = BEAK = FLOWER_PINK = FLOWER_YELLOW = FLOWER_WHITE = GLOW = None


def _gnome():
    G.sphere('footL', 0.032, at=(-0.042, 0.012, 0.026), material=WOOD_DARK, scale=(1.0, 1.5, 0.7))
    G.sphere('footR', 0.032, at=(0.042, 0.012, 0.026), material=WOOD_DARK, scale=(1.0, 1.5, 0.7))
    # tunic: a bell
    G.revolve('tunic', [(0, 0.02), (0.09, 0.02), (0.095, 0.05), (0.082, 0.14),
                       (0.075, 0.24), (0.078, 0.27), (0, 0.29)], material=TUNIC, segments=20)
    G.torus('belt', 0.079, 0.013, at=(0, 0, 0.20), material=WOOD_DARK, major_segments=20, minor_segments=8)
    # arms: tapered cylinders from the shoulders down and forward
    for s in (-1, 1):
        G.cylinder(f'arm{s}', 0.030, 0.135, at=(s * 0.078, -0.012, 0.225), material=TUNIC,
                    rot=(0, -s * 22, 0), verts=12)
        G.sphere(f'hand{s}', 0.026, at=(s * 0.102, -0.02, 0.168), material=SKIN, scale=(1, 1.1, 1))
    # head, nose, beard, hat
    G.sphere('head', 0.062, at=(0, -0.006, 0.345), material=SKIN, scale=(1, 1.02, 1))
    G.sphere('nose', 0.020, at=(0, -0.058, 0.338), material=NOSE)
    G.sphere('beard', 0.058, at=(0, -0.035, 0.305), material=BEARD, scale=(1.0, 0.85, 0.85))
    G.cylinder('hat_brim', 0.072, 0.016, at=(0, -0.004, 0.388), material=HAT, verts=20)
    G.cone('hat', 0.070, 0.004, 0.190, at=(0, -0.012, 0.487), material=HAT, verts=20, rot=(-8, 0, 0))
    G.join_all('gnome')


def _birdbath():
    # plinth, tapering column and a shallow bowl whose rim sits at about 0.84 m
    G.revolve('plinth', [(0, 0), (0.18, 0), (0.18, 0.035), (0.10, 0.06), (0, 0.06)], material=STONE_DARK, segments=24)
    G.revolve('column', [(0, 0.05), (0.10, 0.05), (0.075, 0.42), (0.10, 0.60), (0.11, 0.66), (0, 0.66)], material=STONE, segments=24)
    G.revolve('bowl', [(0, 0.62), (0.27, 0.64), (0.33, 0.72), (0.34, 0.80), (0.30, 0.83),
                      (0.26, 0.80), (0.25, 0.72), (0, 0.70)], material=STONE, segments=32)
    G.cylinder('water', 0.25, 0.012, at=(0, 0, 0.795), material=WATER, verts=32)
    # a small bird perched on the rim
    G.sphere('bird', 0.045, at=(0.20, 0.02, 0.86), material=BIRD_BROWN, scale=(1.0, 1.4, 1.0))
    G.sphere('bird_head', 0.030, at=(0.225, -0.045, 0.895), material=BIRD_BROWN)
    G.cone('beak', 0.010, 0.0, 0.028, at=(0.248, -0.065, 0.892), material=BEAK, rot=(90, 0, 0), verts=8)
    G.join_all('birdbath')


def _fountain():
    G.revolve('plinth', [(0, 0), (0.46, 0), (0.46, 0.05), (0.40, 0.09), (0, 0.09)], material=STONE_DARK, segments=32)
    G.revolve('basin', [(0.34, 0.09), (0.44, 0.11), (0.46, 0.30), (0.43, 0.34), (0.38, 0.36),
                       (0.37, 0.30), (0.39, 0.12), (0.34, 0.11)], material=STONE, segments=32)
    G.cylinder('water', 0.38, 0.014, at=(0, 0, 0.29), material=WATER, verts=32)
    G.revolve('stem', [(0, 0.28), (0.12, 0.30), (0.10, 0.50), (0.13, 0.56), (0.20, 0.60),
                       (0.22, 0.63), (0.19, 0.66), (0, 0.65)], material=STONE, segments=24)
    G.revolve('bowl2', [(0, 0.62), (0.20, 0.64), (0.24, 0.70), (0.25, 0.73), (0.22, 0.75),
                       (0.19, 0.73), (0.17, 0.69), (0, 0.68)], material=STONE, segments=32)
    G.cylinder('water2', 0.185, 0.012, at=(0, 0, 0.735), material=WATER, verts=32)
    G.revolve('finial', [(0, 0.74), (0.06, 0.76), (0.05, 0.86), (0.04, 0.90), (0, 0.92)], material=STONE, segments=20)
    G.sphere('knob', 0.055, at=(0, 0, 0.945), material=STONE)
    G.cylinder('jet', 0.018, 0.13, at=(0, 0, 1.02), material=WATER, verts=12)
    G.sphere('drop', 0.028, at=(0, 0, 1.095), material=WATER, scale=(1, 1, 1.3))
    G.join_all('fountain')


def _lantern():
    # a Japanese stone lantern: hexagonal base, column, fire box and roof
    G.cone('base', 0.23, 0.21, 0.09, at=(0, 0, 0.045), material=STONE_DARK, verts=6, smooth=False)
    G.cone('base2', 0.16, 0.13, 0.05, at=(0, 0, 0.115), material=STONE_DARK, verts=6, smooth=False)
    G.cone('column', 0.085, 0.075, 0.36, at=(0, 0, 0.32), material=STONE, verts=6, smooth=False)
    G.cone('platform', 0.20, 0.185, 0.07, at=(0, 0, 0.535), material=STONE, verts=6, smooth=False)
    G.cone('firebox', 0.15, 0.145, 0.20, at=(0, 0, 0.67), material=GLOW, verts=6, smooth=False)
    for i in range(6):
        a = 60 * i + 30
        G.cone(f'post{i}', 0.026, 0.024, 0.20, at=(0.115 * G.cosd(a), 0.115 * G.sind(a), 0.67),
               material=STONE, verts=4, smooth=False)
    G.cone('roof', 0.27, 0.045, 0.17, at=(0, 0, 0.855), material=STONE, verts=6, smooth=False)
    G.cone('roof2', 0.15, 0.065, 0.05, at=(0, 0, 0.955), material=STONE, verts=6, smooth=False)
    G.sphere('knob', 0.045, at=(0, 0, 1.005), material=STONE, scale=(1, 1, 1.3))
    G.join_all('lantern')


def _wheelbarrow():
    green = METAL
    # tray: a bottom and four tilted side panels
    G.cube('tray', (0.44, 0.56, 0.035), at=(0, 0.02, 0.40), material=green, bevel=0.008)
    G.cube('tray_back', (0.50, 0.03, 0.24), at=(0, 0.27, 0.51), material=green, rot=(-20, 0, 0), bevel=0.008)
    G.cube('tray_front', (0.48, 0.03, 0.22), at=(0, -0.30, 0.50), material=green, rot=(14, 0, 0), bevel=0.008)
    for s in (-1, 1):
        G.cube(f'tray_side{s}', (0.03, 0.58, 0.22), at=(s * 0.23, -0.02, 0.52), material=green,
               rot=(0, s * 14, 0), bevel=0.008)
    G.cube('soil', (0.42, 0.50, 0.03), at=(0, -0.02, 0.60), material=SOIL)
    # wheel at the back (+Y) and handles reaching to the front (-Y)
    G.torus('tyre', 0.13, 0.035, at=(0, 0.36, 0.16), material=METAL_DARK, rot=(0, 90, 0), major_segments=20)
    G.cylinder('hub', 0.05, 0.06, at=(0, 0.36, 0.16), material=METAL, rot=(0, 90, 0), verts=12)
    for s in (-1, 1):
        G.rod(f'handle{s}', (s * 0.17, 0.28, 0.44), (s * 0.21, -0.44, 0.58), 0.020, material=WOOD, verts=10)
        G.rod(f'leg{s}', (s * 0.17, 0.14, 0.42), (s * 0.19, -0.20, 0.0), 0.018, material=METAL_DARK, verts=8)
    # flowers and leaves in the tray
    for i, (x, y, mat_) in enumerate([(-0.12, -0.14, FLOWER_PINK), (0.12, -0.02, FLOWER_YELLOW),
                                       (-0.06, 0.14, FLOWER_WHITE), (0.10, 0.18, FLOWER_PINK),
                                       (-0.16, 0.02, FLOWER_YELLOW), (0.02, -0.20, FLOWER_WHITE),
                                       (0.16, -0.16, FLOWER_PINK)]):
        G.sphere(f'flower{i}', 0.055, at=(x, y, 0.70), material=mat_, scale=(1, 1, 0.8))
    for i, (x, y) in enumerate([(-0.10, -0.10), (0.10, 0.05), (-0.05, 0.18)]):
        G.rock(f'leaf{i}', 0.09, at=(x, y, 0.64), material=LEAF, scale=(1, 1, 0.5), seed=i * 1.7, noise=0.35)
    G.join_all('wheelbarrow')


def _boulder():
    G.rock('boulder_main', 0.40, at=(0, 0, 0), material=STONE, scale=(1.15, 0.9, 0.62), seed=0.4)
    G.rock('boulder_side', 0.17, at=(0.34, -0.22, 0), material=STONE_DARK, scale=(1.0, 0.8, 0.85), seed=2.1)
    G.rock('boulder_small', 0.10, at=(-0.40, 0.24, 0), material=STONE_WARM, scale=(1.0, 0.8, 1.0), seed=4.2)
    # moss on top and a few tufts around the base
    G.rock('moss', 0.22, at=(-0.06, -0.04, 0.34), material=MOSS, scale=(1.5, 0.75, 0.20), seed=7.3, noise=0.35)
    G.rock('moss2', 0.13, at=(0.26, 0.18, 0.16), material=MOSS, scale=(1.3, 0.75, 0.25), seed=9.1, noise=0.4)
    for i, (x, y) in enumerate([(0.42, 0.04), (-0.44, 0.10), (0.10, 0.42), (-0.16, -0.40)]):
        G.rock(f'tuft{i}', 0.09, at=(x, y, 0), material=LEAF, scale=(0.7, 1.6, 0.6), seed=i * 2.3, noise=0.45)
    G.join_all('boulder')


# exact placeholder bounds (game w, h, d), so the models drop into the existing footprints
TARGETS = {
    'gnome': (0.26, 0.58, 0.18),
    'birdbath': (0.68, 0.935, 0.68),
    'fountain': (0.92, 1.17, 0.92),
    'lantern': (0.54, 1.035, 0.54),
    'wheelbarrow': (0.587, 0.757, 1.036),
    'boulder': (1.003, 0.59, 0.851),
}


def _make(model):
    G.reset()
    _palette()
    {'gnome': _gnome, 'birdbath': _birdbath, 'fountain': _fountain,
     'lantern': _lantern, 'wheelbarrow': _wheelbarrow, 'boulder': _boulder}[model]()
    obj = bpy.context.scene.objects[0]
    G.fit(obj, *TARGETS[model])
    return obj


def main(argv):
    preview = '--preview' in argv
    models = ['gnome', 'birdbath', 'fountain', 'lantern', 'wheelbarrow', 'boulder']
    os.makedirs(OUT, exist_ok=True)
    if preview:
        os.makedirs(PREVIEW, exist_ok=True)
    for name in models:
        _make(name)
        G.stats()
        if preview:
            G.preview(os.path.join(PREVIEW, f'{name}.png'), size=1.3)
        else:
            G.export_glb(os.path.join(OUT, f'{name}.glb'))
        print(f'built {name}')


if __name__ == '__main__':
    main(sys.argv)

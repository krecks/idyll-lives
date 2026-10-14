# Blender-built model for the kitchen fridge (`model.fridge`), replacing the primitive-part
# placeholder of `furniture_prims.py` (`fridge_classic`). The @cozy / @minimal / @modern style
# variants follow the same pattern (the colour-only ones reuse this file with their own `materials`).
#
# Sizes match the placeholder's bounding box so footprints, lights and the selection marker stay put.
# Convention (docs/design/objects.md): metres, origin at the footprint centre on the floor, front facing
# -Y in Blender (the glTF export's +Y up turns that into the game's +Z front). Built to the real size.
#
# Run:  blender --background --factory-startup --python tools/art/blender/kitchen.py
#       blender --background --factory-startup --python tools/art/blender/kitchen.py -- --preview
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gamekit as G  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
ASSETS = os.path.abspath(os.path.join(HERE, '..', '..', '..', 'web', 'public', 'assets'))
OUT = os.path.join(ASSETS, 'models', 'kitchen')
PACK = os.path.join(ASSETS, 'packs', 'kitchen', 'manifest.json')
PREVIEW = os.path.join(HERE, 'previews')

# The placeholder's back sits 9.4 cm behind the middle of its tile (`furniture_prims.BACK = -0.455`);
# the model is centred like the garden decor, so every game depth is shifted back by that much.
SHIFT = 0.094
# Seconds (the residents' animation time) a door stays opening or open once a use starts: it opens in
# 1.75 s (`kitchen_doors.py`), stays open while they reach in, then closes.
HOLD = 3.5


def P(x, y, z):
    """A game position (x, height, depth) as a Blender position (front -Y, up +Z), centred on the tile."""
    return (-x, -z - SHIFT, y)


def gbox(name, w, h, d, at, material, **kw):
    """A game-sized box (width, height, depth) at a game position, in Blender axes."""
    return G.cube(name, (w, d, h), at=P(*at), material=material, **kw)


def gcyl(name, r, h, at, material, **kw):
    """A game-sized cylinder (radius, height) at a game position, in Blender axes."""
    return G.cylinder(name, r, h, at=P(*at), material=material, **kw)


def gsphere(name, r, at, material, **kw):
    """A game-sized sphere (radius) at a game position, in Blender axes."""
    return G.sphere(name, r, at=P(*at), material=material, **kw)


# palette (rebuilt after every reset, which drops the old materials)
def _palette():
    global BODY, TRIM, SEAL, CHROME, LINER, SHELF, CARTON, BOTTLE, PRODUCE, PRODUCE2, EGG
    global FROZEN, ICECREAM, PEAS
    BODY = G.mat('body', '#F3F3F1', 0.12)              # white cabinet and doors (gloss)
    TRIM = G.mat('trim', '#B9BCBF', 0.42)              # plinth and badge (satin)
    SEAL = G.mat('seal', '#3A3D41', 0.42)              # door reveal and gap (satin)
    CHROME = G.mat('chrome', '#D0D3D6', 0.2, 0.9)      # bar handles (chrome)
    LINER = G.mat('liner', '#DCE0E3', 0.7)              # fridge interior walls (matte)
    SHELF = G.mat('shelf', '#E9ECEE', 0.18)             # shelves, tubs (gloss)
    CARTON = G.mat('carton', '#EAF0F4', 0.35)           # milk / egg cartons (satin)
    BOTTLE = G.mat('bottle', '#4E8A54', 0.2)            # juice bottle (gloss)
    PRODUCE = G.mat('produce', '#C0392B', 0.35)          # apples (satin)
    PRODUCE2 = G.mat('produce2', '#E08A2E', 0.35)        # orange (satin)
    EGG = G.mat('egg', '#F2E6C9', 0.5)                  # eggs (matte)
    FROZEN = G.mat('frozen', '#8FB6D6', 0.35)           # frozen pizza box (satin)
    ICECREAM = G.mat('icecream', '#E8B4C8', 0.35)        # ice cream tub (satin)
    PEAS = G.mat('peas', '#6FA96F', 0.35)                # frozen peas (satin)


BODY = TRIM = SEAL = CHROME = LINER = SHELF = CARTON = BOTTLE = PRODUCE = PRODUCE2 = EGG = None
FROZEN = ICECREAM = PEAS = None


def _bar(name, x, y0, y1, z, material=None, t=0.022, stand=0.03):
    """A vertical bar handle on two stand-offs, its back on the door face at game depth `z`."""
    h = y1 - y0
    gbox(f'{name}_bar', t, h, t, (x, (y0 + y1) / 2, z + stand + t / 2), material or CHROME, bevel=t * 0.45)
    for i, y in enumerate((y0 + 0.03, y1 - 0.03)):
        gbox(f'{name}_stand{i}', t * 0.8, t * 0.8, stand, (x, y, z + stand / 2), material or CHROME, bevel=0.004)


def _shell():
    """The cabinet as five panels, so the doors open onto a real interior instead of a solid block."""
    gbox('side_left', 0.03, 1.74, 0.622, (-0.355, 0.93, -0.144), BODY, bevel=0.006)
    gbox('side_right', 0.03, 1.74, 0.622, (0.355, 0.93, -0.144), BODY, bevel=0.006)
    gbox('top', 0.74, 0.03, 0.622, (0, 1.785, -0.144), BODY, bevel=0.006)
    gbox('bottom', 0.74, 0.03, 0.622, (0, 0.075, -0.144), BODY, bevel=0.006)
    gbox('back', 0.74, 1.74, 0.03, (0, 0.93, -0.44), BODY, bevel=0.006)


def _interior():
    """The liner, shelves and food, seen once the doors are open."""
    gbox('liner_back', 0.678, 1.678, 0.006, (0, 0.93, -0.421), LINER, bevel=0.002)
    gbox('liner_left', 0.006, 1.678, 0.578, (-0.337, 0.93, -0.134), LINER, bevel=0.002)
    gbox('liner_right', 0.006, 1.678, 0.578, (0.337, 0.93, -0.134), LINER, bevel=0.002)
    gbox('liner_top', 0.678, 0.006, 0.578, (0, 1.767, -0.134), LINER, bevel=0.002)
    gbox('liner_bottom', 0.678, 0.006, 0.578, (0, 0.093, -0.134), LINER, bevel=0.002)
    gbox('liner_bottom', 0.678, 0.006, 0.578, (0, 0.093, -0.134), LINER, bevel=0.002)
    # the divider between the freezer (top) and the fridge (bottom), the freezer's floor
    gbox('divider', 0.672, 0.02, 0.55, (0, 1.234, -0.13), SHELF, bevel=0.002)
    # three shelves in the fridge, one in the freezer
    for i, y in enumerate((0.37, 0.66, 0.95)):
        gbox(f'shelf{i}', 0.672, 0.008, 0.55, (0, y, -0.13), SHELF, bevel=0.002)
    gbox('shelf_freezer', 0.672, 0.008, 0.55, (0, 1.52, -0.13), SHELF, bevel=0.002)
    _fridge_food()
    _freezer_food()


def _fridge_food():
    """Normal food, in the bottom (fridge) compartment: dairy, produce and eggs."""
    top, mid, low = 0.954, 0.664, 0.374  # the fridge shelf surfaces
    gbox('milk', 0.085, 0.20, 0.085, (-0.20, top + 0.10, -0.14), CARTON, bevel=0.006)
    G.cone('milk_gable', 0.062, 0.0, 0.055, at=P(-0.20, top + 0.2275, -0.14), material=CARTON,
           verts=4, rot=(0, 0, 45))
    gcyl('juice', 0.032, 0.20, (0.02, top + 0.10, -0.14), BOTTLE, verts=16, bevel=0.004)
    gcyl('juice_cap', 0.014, 0.025, (0.02, top + 0.2125, -0.14), CARTON, verts=12)
    for i, x in enumerate((-0.20, -0.11, -0.02)):
        gsphere(f'apple{i}', 0.038, (x, mid + 0.038, -0.15), PRODUCE, segments=10, rings=6)
    gsphere('orange', 0.042, (0.13, mid + 0.042, -0.15), PRODUCE2, segments=10, rings=6)
    gbox('egg_carton', 0.16, 0.03, 0.11, (-0.16, low + 0.015, -0.14), CARTON, bevel=0.006)
    for i, x in enumerate((-0.205, -0.155, -0.105)):
        gsphere(f'egg{i}', 0.017, (x, low + 0.03 + 0.013, -0.16), EGG, segments=8, rings=5)
    gbox('tub', 0.13, 0.06, 0.13, (0.14, low + 0.03, -0.14), SHELF, bevel=0.01)


def _freezer_food():
    """Frozen food, in the top (freezer) compartment: a pizza, ice cream and peas."""
    floor, shelf = 1.244, 1.524  # the divider's top and the freezer shelf
    gbox('pizza', 0.26, 0.035, 0.26, (-0.13, floor + 0.0175, -0.14), FROZEN, bevel=0.006)
    gbox('ice_cream', 0.12, 0.10, 0.12, (0.14, shelf + 0.05, -0.14), ICECREAM, bevel=0.01)
    gbox('peas', 0.11, 0.15, 0.06, (-0.14, shelf + 0.075, -0.16), PEAS, bevel=0.008)


def _fridge():
    # cabinet on a recessed plinth, its doors opening onto a liner, shelves and food
    gbox('plinth', 0.68, 0.06, 0.57, (0, 0.03, -0.17), TRIM, bevel=0.006)
    _shell()
    _interior()
    # freezer door over the fridge door, both rounded and proud of the opening
    gbox('door_freezer', 0.728, 0.556, 0.04, (0, 1.51, 0.195), BODY, bevel=0.010)
    gbox('door_fridge', 0.728, 1.146, 0.04, (0, 0.643, 0.195), BODY, bevel=0.010)
    gbox('gap', 0.7, 0.014, 0.03, (0, 1.224, 0.187), SEAL, bevel=0.004)
    # vertical chrome bar handles on the left of each door (hinges right)
    _bar('handle_freezer', -0.3, 1.274, 1.524, 0.215)
    _bar('handle_fridge', -0.3, 0.804, 1.164, 0.215)
    # a badge on the freezer door, top right
    gbox('badge', 0.1, 0.018, 0.004, (0.25, 1.74, 0.217), TRIM, bevel=0.002)
    return G.join_all('fridge')


# exact placeholder bounds (game w, h, d): the model is built to these and only asserted against them
TARGET = (0.74, 1.8, 0.722)


def _check(obj, w, h, d, tol=0.02):
    """Asserts the model is the real size, centred on its tile, its bottom on the floor."""
    ws = [obj.matrix_world @ v.co for v in obj.data.vertices]
    lo = [min(v[i] for v in ws) for i in range(3)]
    hi = [max(v[i] for v in ws) for i in range(3)]
    got = [hi[i] - lo[i] for i in range(3)]
    want = [w, d, h]  # Blender x, y, z
    for i, axis in enumerate('xyz'):
        assert abs(got[i] - want[i]) <= tol * want[i], f'{axis} size {got[i]:.3f} m, want {want[i]:.3f} m'
    for i, axis in enumerate('xyz'):
        c = (lo[i] + hi[i]) / 2
        if i < 2:
            assert abs(c) <= 0.002, f'{axis} off centre by {c * 100:.1f} cm'
        else:
            assert abs(lo[i]) <= 0.002, f'bottom {lo[i] * 100:.1f} cm off the floor'
    print(f'[kitchen] fridge {got[0]:.3f} x {got[2]:.3f} x {got[1]:.3f} m (w x h x d), centred, on the floor')


def _write_pack():
    """Writes the pack entry from the base manifest's current placeholder, so it can't go stale."""
    with open(os.path.join(ASSETS, 'manifest.json')) as f:
        base = json.load(f)
    entry = {
        'type': 'model',
        # The doors' build (`kitchen_doors.py`): the same fridge, its doors opening while it's used.
        'url': '../../models/kitchen/fridge.doors.glb',
        'materials': {
            '*': {'finish': 'material.finish.gloss'},
            'trim': {'finish': 'material.finish.satin'},
            'seal': {'finish': 'material.finish.satin'},
            'chrome': {'finish': 'material.finish.chrome'},
            'liner': {'finish': 'material.finish.matte'},
            'shelf': {'finish': 'material.finish.gloss'},
            'carton': {'finish': 'material.finish.satin'},
            'bottle': {'finish': 'material.finish.gloss'},
            'produce': {'finish': 'material.finish.satin'},
            'produce2': {'finish': 'material.finish.satin'},
            'egg': {'finish': 'material.finish.matte'},
            'frozen': {'finish': 'material.finish.satin'},
            'icecream': {'finish': 'material.finish.satin'},
            'peas': {'finish': 'material.finish.satin'},
        },
        # Taking food or a drink out opens the fridge door (snacks, a recipe's ingredients, a cold
        # drink, putting dishes away); nothing opens the freezer yet: no use takes frozen food.
        'motions': {'uses': {'fetch': 'fridge_open', 'drink': 'fridge_open'}, 'hold': HOLD},
        # The colour the player gives it goes on the cabinet and doors; the inside, the food and the
        # handles stay as they are.
        'paint': ['body'],
        'placeholder': base['entries']['model.fridge']['placeholder'],
    }
    manifest = {
        '$comment': 'Blender-built kitchen models (tools/art/blender/kitchen.py). Replaces the '
                    'primitive-part placeholders by key; listed in the base manifest\'s packs. The '
                    'placeholder is kept as a fallback if a file fails to load, and the materials map '
                    'onto the game\'s finishes so a model costs one draw call per finish group.',
        'version': 1,
        'entries': {'model.fridge': entry},
    }
    os.makedirs(os.path.dirname(PACK), exist_ok=True)
    with open(PACK, 'w') as f:
        json.dump(manifest, f, indent=2)
        f.write('\n')
    rel = os.path.relpath(PACK, ASSETS)
    if rel not in base['packs']:
        base['packs'].append(rel)
        with open(os.path.join(ASSETS, 'manifest.json'), 'w') as f:
            json.dump(base, f, indent=2)
            f.write('\n')


def _make():
    G.reset()
    _palette()
    obj = _fridge()
    _check(obj, *TARGET)
    return obj


def main(argv):
    preview = '--preview' in argv
    os.makedirs(OUT, exist_ok=True)
    if preview:
        os.makedirs(PREVIEW, exist_ok=True)
    _make()
    G.stats()
    if preview:
        G.preview(os.path.join(PREVIEW, 'fridge.png'), size=1.3)
    else:
        G.export_glb(os.path.join(OUT, 'fridge.glb'))
        _write_pack()
    print('built fridge')


if __name__ == '__main__':
    main(sys.argv)

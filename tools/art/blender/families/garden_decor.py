# The garden decor (`model.garden.gnome`, `birdbath`, `fountain`, `lantern`, `wheelbarrow`, `boulder`).
#
# Built at their real size, in Blender axes (front -Y), each stood on the floor and centred by
# `G.settle`; the sizes are the old parts', within each model's tolerance. The fountain's water moves all the
# time near the camera (`flow`, ambient: the jet bubbling up, rings on the basins); the rest are still.
#
# Run:  cargo art garden_decor [--only gnome] [--preview]
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    stone=('#B9B2A6', 'matte'),
    stone_dark=('#8F887C', 'matte'),
    stone_warm=('#C6BCA9', 'matte'),
    water=('#6FA8C0', 'gloss'),         # opaque, like the old parts' water
    moss=('#5E8A3C', 'foliage'),
    leaf=('#4E7A34', 'foliage'),
    wood=('#9A7444', 'wood'),
    wood_dark=('#6E5232', 'satin'),     # the gnome's boots and belt
    metal=('#5A6068', 'metal'),
    metal_dark=('#2E3236', 'matte'),
    paint=('#3E7A6A', 'metal'),         # the wheelbarrow's green tray
    skin=('#F0C6A4', 'satin'),
    nose=('#E8A08A', 'satin'),
    beard=('#EDE8DE', 'satin'),
    tunic=('#3E63B0', 'satin'),
    hat=('#C8322A', 'satin'),
    soil=('#4A3826', 'matte'),
    bird_brown=('#7A5A40', 'satin'),
    beak=('#E8A23A', 'satin'),
    flower_pink=('#E85A9A', 'satin'),
    flower_yellow=('#F2C14E', 'satin'),
    flower_white=('#F4F0F2', 'satin'),
    glow=('#FFE6A8', 'satin'),          # the lantern's fire box; the content's `light` lights it at night
)


def gnome():
    G.sphere('footL', 0.032, at=(-0.042, 0.012, 0.026), material=M.wood_dark, scale=(1.0, 1.5, 0.7))
    G.sphere('footR', 0.032, at=(0.042, 0.012, 0.026), material=M.wood_dark, scale=(1.0, 1.5, 0.7))
    # tunic: a bell
    G.revolve('tunic', [(0, 0.02), (0.09, 0.02), (0.095, 0.05), (0.082, 0.14),
                        (0.075, 0.24), (0.078, 0.27), (0, 0.29)], material=M.tunic, segments=20)
    G.torus('belt', 0.079, 0.013, at=(0, 0, 0.20), material=M.wood_dark, major_segments=20, minor_segments=8)
    # arms: tapered cylinders from the shoulders down and forward
    for s in (-1, 1):
        G.cylinder(f'arm{s}', 0.030, 0.135, at=(s * 0.078, -0.012, 0.225), material=M.tunic,
                   rot=(0, -s * 22, 0), verts=12)
        G.sphere(f'hand{s}', 0.026, at=(s * 0.102, -0.02, 0.168), material=M.skin, scale=(1, 1.1, 1))
    # head, nose, beard, hat
    G.sphere('head', 0.062, at=(0, -0.006, 0.345), material=M.skin, scale=(1, 1.02, 1))
    G.sphere('nose', 0.020, at=(0, -0.058, 0.338), material=M.nose)
    G.sphere('beard', 0.058, at=(0, -0.035, 0.305), material=M.beard, scale=(1.0, 0.85, 0.85))
    G.cylinder('hat_brim', 0.072, 0.016, at=(0, -0.004, 0.388), material=M.hat, verts=20)
    G.cone('hat', 0.070, 0.004, 0.190, at=(0, -0.012, 0.487), material=M.hat, verts=20, rot=(-8, 0, 0))
    return G.settle([G.join_all('gnome')])


def birdbath():
    # plinth, tapering column and a shallow bowl whose rim sits at about 0.84 m
    G.revolve('plinth', [(0, 0), (0.18, 0), (0.18, 0.035), (0.10, 0.06), (0, 0.06)], material=M.stone_dark, segments=24)
    G.revolve('column', [(0, 0.05), (0.10, 0.05), (0.075, 0.42), (0.10, 0.60), (0.11, 0.66), (0, 0.66)],
              material=M.stone, segments=24)
    G.revolve('bowl', [(0, 0.62), (0.27, 0.64), (0.33, 0.72), (0.34, 0.80), (0.30, 0.83),
                       (0.26, 0.80), (0.25, 0.72), (0, 0.70)], material=M.stone, segments=32)
    G.cylinder('water', 0.25, 0.012, at=(0, 0, 0.795), material=M.water, verts=32)
    # a small bird perched on the rim
    G.sphere('bird', 0.045, at=(0.20, 0.02, 0.86), material=M.bird_brown, scale=(1.0, 1.4, 1.0))
    G.sphere('bird_head', 0.030, at=(0.225, -0.045, 0.895), material=M.bird_brown)
    G.cone('beak', 0.010, 0.0, 0.028, at=(0.248, -0.065, 0.892), material=M.beak, rot=(90, 0, 0), verts=8)
    return G.settle([G.join_all('birdbath')])


def fountain():
    G.revolve('plinth', [(0, 0), (0.46, 0), (0.46, 0.05), (0.40, 0.09), (0, 0.09)], material=M.stone_dark, segments=32)
    G.revolve('basin', [(0.34, 0.09), (0.44, 0.11), (0.46, 0.30), (0.43, 0.34), (0.38, 0.36),
                        (0.37, 0.30), (0.39, 0.12), (0.34, 0.11)], material=M.stone, segments=32)
    G.cylinder('water', 0.38, 0.014, at=(0, 0, 0.29), material=M.water, verts=32)
    G.revolve('stem', [(0, 0.28), (0.12, 0.30), (0.10, 0.50), (0.13, 0.56), (0.20, 0.60),
                       (0.22, 0.63), (0.19, 0.66), (0, 0.65)], material=M.stone, segments=24)
    G.revolve('bowl2', [(0, 0.62), (0.20, 0.64), (0.24, 0.70), (0.25, 0.73), (0.22, 0.75),
                        (0.19, 0.73), (0.17, 0.69), (0, 0.68)], material=M.stone, segments=32)
    G.cylinder('water2', 0.185, 0.012, at=(0, 0, 0.735), material=M.water, verts=32)
    G.revolve('finial', [(0, 0.74), (0.06, 0.76), (0.05, 0.86), (0.04, 0.90), (0, 0.92)], material=M.stone, segments=20)
    G.sphere('knob', 0.055, at=(0, 0, 0.945), material=M.stone)
    body = G.join(G.meshes(), 'fountain', origin=(0, 0, 0))
    # The water moves all the time (`flow`, ambient): the jet bubbles up from the knob and falls back, and a ring
    # on each basin's water swells out from where the water falls and settles back.
    with G.new_parts() as jet:
        G.cylinder('jet', 0.018, 0.13, at=(0, 0, 1.02), material=M.water, verts=12)
        G.sphere('drop', 0.028, at=(0, 0, 1.095), material=M.water, scale=(1, 1, 1.3))
    jet = G.join(jet, 'jet', origin=(0, 0, 0.955))
    with G.new_parts() as rings:
        G.torus('ripple2', 0.1, 0.006, at=(0, 0, 0.742), material=M.water, major_segments=24, minor_segments=4)
        G.torus('ripple', 0.24, 0.007, at=(0, 0, 0.298), material=M.water, major_segments=32, minor_segments=4)
    rings = G.join(rings, 'ripples', origin=(0, 0, 0.3))
    objs = G.settle([body, jet, rings])
    G.squash(jet, 'flow', (1, 1, 1.35), seconds=1.2, back=True)
    G.squash(rings, 'flow', (1.4, 1.4, 1), seconds=1.2, back=True)
    return objs


# The lantern's hexagons are regular and turned as the old part's (Babylon cylinders): corners to the
# sides (x), flats to the front and back, so it is 0.54 m across its roof's corners and 0.47 deep, as the
# old part. (Blender starts a circle at +y; `rot` turns it 30 degrees.)


def lantern():
    # a Japanese stone lantern: hexagonal base, column, fire box and roof
    def hexa(name, r1, r2, depth, z, material, verts=6):
        G.cone(name, r1, r2, depth, at=(0, 0, z), material=material, verts=verts, smooth=False, rot=(0, 0, 30))
    hexa('base', 0.23, 0.21, 0.09, 0.045, M.stone_dark)
    hexa('base2', 0.16, 0.13, 0.05, 0.115, M.stone_dark)
    hexa('column', 0.085, 0.075, 0.36, 0.32, M.stone)
    hexa('platform', 0.20, 0.185, 0.07, 0.535, M.stone)
    hexa('firebox', 0.15, 0.145, 0.20, 0.67, M.glow)
    for i in range(6):
        a = 60 * i
        r = 0.115
        G.cone(f'post{i}', 0.026, 0.024, 0.20, at=(r * G.cosd(a), r * G.sind(a), 0.67),
               material=M.stone, verts=4, smooth=False)
    hexa('roof', 0.27, 0.045, 0.17, 0.855, M.stone)
    hexa('roof2', 0.15, 0.065, 0.05, 0.955, M.stone)
    G.sphere('knob', 0.045, at=(0, 0, 1.005), material=M.stone, scale=(1, 1, 1.3))
    return G.settle([G.join_all('lantern')])


def wheelbarrow():
    green = M.paint
    # tray: a bottom and four tilted side panels
    G.cube('tray', (0.44, 0.56, 0.035), at=(0, 0.02, 0.40), material=green, bevel=0.008)
    G.cube('tray_back', (0.50, 0.03, 0.24), at=(0, 0.27, 0.51), material=green, rot=(-20, 0, 0), bevel=0.008)
    G.cube('tray_front', (0.48, 0.03, 0.22), at=(0, -0.30, 0.50), material=green, rot=(14, 0, 0), bevel=0.008)
    for s in (-1, 1):
        G.cube(f'tray_side{s}', (0.03, 0.58, 0.22), at=(s * 0.23, -0.02, 0.52), material=green,
               rot=(0, s * 14, 0), bevel=0.008)
    G.cube('soil', (0.42, 0.50, 0.03), at=(0, -0.02, 0.60), material=M.soil)
    # wheel at the back (+Y), its tyre on the floor like the legs, and handles reaching to the front (-Y)
    wheel = 0.165
    G.torus('tyre', 0.13, 0.035, at=(0, 0.36, wheel), material=M.metal_dark, rot=(0, 90, 0), major_segments=20)
    G.cylinder('hub', 0.05, 0.06, at=(0, 0.36, wheel), material=M.metal, rot=(0, 90, 0), verts=12)
    for s in (-1, 1):
        G.rod(f'handle{s}', (s * 0.17, 0.28, 0.44), (s * 0.21, -0.44, 0.58), 0.020, material=M.wood, verts=10)
        G.rod(f'leg{s}', (s * 0.17, 0.14, 0.42), (s * 0.19, -0.20, 0.0), 0.018, material=M.metal_dark, verts=8)
    # flowers and leaves in the tray
    for i, (x, y, colour) in enumerate([(-0.12, -0.14, M.flower_pink), (0.12, -0.02, M.flower_yellow),
                                        (-0.06, 0.14, M.flower_white), (0.10, 0.18, M.flower_pink),
                                        (-0.16, 0.02, M.flower_yellow), (0.02, -0.20, M.flower_white),
                                        (0.16, -0.16, M.flower_pink)]):
        G.sphere(f'flower{i}', 0.055, at=(x, y, 0.70), material=colour, scale=(1, 1, 0.8))
    for i, (x, y) in enumerate([(-0.10, -0.10), (0.10, 0.05), (-0.05, 0.18)]):
        G.boulder(f'leaf{i}', 0.09, at=(x, y, 0.64), material=M.leaf, scale=(1, 1, 0.5), seed=i * 1.7, noise=0.35)
    return G.settle([G.join_all('wheelbarrow')])


def boulder():
    G.boulder('boulder_main', 0.40, at=(0, 0, 0), material=M.stone, scale=(1.05, 0.9, 0.62), seed=0.4)
    G.boulder('boulder_side', 0.17, at=(0.30, -0.20, 0), material=M.stone_dark, scale=(1.0, 0.8, 0.85), seed=2.1)
    G.boulder('boulder_small', 0.10, at=(-0.36, 0.20, 0), material=M.stone_warm, scale=(1.0, 0.8, 1.0), seed=4.2)
    # moss on top and a few tufts around the base
    G.boulder('moss', 0.22, at=(-0.06, -0.04, 0.41), material=M.moss, scale=(1.5, 0.75, 0.20), seed=7.3, noise=0.35)
    G.boulder('moss2', 0.13, at=(0.24, 0.14, 0.16), material=M.moss, scale=(1.3, 0.75, 0.25), seed=9.1, noise=0.4)
    for i, (x, y) in enumerate([(0.38, 0.04), (-0.40, 0.08), (0.10, 0.33), (-0.16, -0.33)]):
        G.boulder(f'tuft{i}', 0.09, at=(x, y, 0), material=M.leaf, scale=(1.6 if i > 1 else 0.7, 0.7 if i > 1 else 1.6, 0.6),
               seed=i * 2.3, noise=0.45)
    return G.settle([G.join_all('boulder')])


# (width, height, depth): the old parts' bounds, and how far each model may be from them
TOLERANCE = 0.1

FAMILY = G.Family(
    'garden', M,
    [G.Model('gnome', gnome, size=(0.26, 0.58, 0.18), tolerance=TOLERANCE,
             paint=[('hat', 'Hat'), ('tunic', 'Tunic'), ('wood_dark', 'Belt')]),
     G.Model('birdbath', birdbath, size=(0.68, 0.935, 0.68), tolerance=TOLERANCE,
             paint=[('stone', 'Bowl & column'), ('stone_dark', 'Plinth')]),
     G.Model('fountain', fountain, size=(0.92, 1.17, 0.92), tolerance=TOLERANCE, motions={'idle': 'flow'},
             paint=[('stone', 'Fountain'), ('stone_dark', 'Plinth')]),
     G.Model('lantern', lantern, size=(0.54, 1.035, 0.468), tolerance=TOLERANCE,
             paint=[('stone', 'Lantern'), ('stone_dark', 'Base')]),
     G.Model('wheelbarrow', wheelbarrow, size=(0.587, 0.754, 1.036), tolerance=TOLERANCE,
             paint=[('paint', 'Tray'), ('wood', 'Handles'), ('metal_dark', 'Legs & tyre')]),
     G.Model('boulder', boulder, size=(0.903, 0.515, 0.848), tolerance=TOLERANCE,
             paint=[('stone', 'Large boulder'), ('stone_dark', 'Middle boulder'), ('stone_warm', 'Small boulder')])],
    pack='gardenDecor',
    comment='Blender-built garden decor (tools/art/blender/families/garden_decor.py): a gnome, a bird '
            'bath, a two-tier fountain, a stone lantern, a wheelbarrow planter and boulders. Listed in '
            'the base manifest\'s packs. Each file\'s materials map by name onto the game\'s finishes '
            '(`materials`, see render/babylon/models.ts), so a model costs one draw call per finish group '
            'and lights like the rest of the garden.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

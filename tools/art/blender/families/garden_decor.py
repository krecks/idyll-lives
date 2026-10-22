# The garden decor (`model.garden.gnome`, `birdbath`, `fountain`, `lantern`, `wheelbarrow`, `boulder`). Notes: garden_decor.md.
#
# Built at their real size, in Blender axes (front -Y), each stood on the floor and centred by
# `G.settle`; the sizes are the old parts', within each model's tolerance. The fountain's water moves all the
# time near the camera (`flow`, ambient: the jet bubbling up, rings on the basins); the rest are still.
#
# Run:  cargo art garden_decor [--only gnome] [--preview]
import math
import os
import random
import sys

import bpy
from mathutils import Matrix, Vector

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gamekit as G  # noqa: E402
from ground import _cut, _lump  # noqa: E402  (the woods' rock's cuts and roughening)
from plants import dome, leaf, lump, star  # noqa: E402  (leaves and flowers)

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
    eye=('#2A2622', 'gloss'),           # the gnome's eyes
    buckle=('#C9A24A', 'metal'),        # his belt buckle and buttons (brass)
    tunic=('#3E63B0', 'satin'),
    hat=('#C8322A', 'satin'),
    soil=('#4A3826', 'matte'),
    bird_brown=('#7A5A40', 'satin'),
    beak=('#E8A23A', 'satin'),
    bird_breast=('#D9703B', 'satin'),   # the bird bath's robin
    flower_pink=('#E85A9A', 'satin'),
    flower_yellow=('#F2C14E', 'satin'),
    flower_white=('#F4F0F2', 'satin'),
    glow=('#FFE6A8', 'satin'),          # the lantern's fire box; the content's `light` lights it at night
    rock=('#7A756C', 'matte'),          # the boulders: weathered greys, darker than the dressed stone
    rock_dark=('#605C55', 'matte'),
    rock_warm=('#8A8174', 'matte'),
    lichen=('#BDB894', 'matte'),
)
F = G.GameFrame()   # for the kit's parts, which take game axes


def gnome():
    """A garden gnome: boots with turned-up toes, a belted tunic with a brass buckle and two buttons, hands on his belly,
    a round face with rosy cheeks, a nose, eyes and brows, a flowing beard and moustache, a floppy red hat."""
    for s in (-1, 1):
        G.sweep(f'boot{s}', [(s * 0.042, 0.03, 0.02), (s * 0.044, -0.03, 0.02), (s * 0.046, -0.065, 0.024),
                             (s * 0.047, -0.08, 0.038)], None, material=M.wood_dark,
                profile=G.rounded_rect(0.04, 0.052, 0.017, corner=2), up=(0, 0, 1), resolution=2,
                scale=[1.0, 1.05, 0.85, 0.45])
    G.revolve('tunic', [(0, 0.03), (0.09, 0.03), (0.095, 0.042), (0.09, 0.08), (0.082, 0.15), (0.075, 0.22),
                        (0.077, 0.255), (0.072, 0.272), (0.055, 0.29), (0, 0.297)], material=M.tunic, segments=20)
    G.torus('hem', 0.088, 0.008, at=(0, 0, 0.04), material=M.tunic, major_segments=20, minor_segments=4)
    G.torus('belt', 0.08, 0.013, at=(0, 0, 0.20), material=M.wood_dark, major_segments=20, minor_segments=4)
    G.cube('buckle', (0.042, 0.01, 0.034), at=(0, -0.086, 0.2), material=M.buckle, bevel=0.003, bevel_segments=1)
    G.cube('buckle_hole', (0.026, 0.01, 0.018), at=(0, -0.088, 0.2), material=M.wood_dark)
    for k, z in enumerate((0.235, 0.262)):
        G.sphere(f'button{k}', 0.008, at=(0, -0.075 + (z - 0.235) * 0.1, z), material=M.buckle, segments=8, rings=5)
    # the arms, sleeves in one piece from the shoulder, round to the belly, the hands resting on it
    for s in (-1, 1):
        G.sweep(f'arm{s}', [(s * 0.062, 0.0, 0.272), (s * 0.095, -0.012, 0.235), (s * 0.09, -0.04, 0.205),
                            (s * 0.066, -0.064, 0.21)], 0.027, material=M.tunic, verts=8, resolution=2,
                scale=[0.9, 1.0, 0.9, 1.05])
        G.sphere(f'hand{s}', 0.022, at=(s * 0.045, -0.078, 0.212), material=M.skin, scale=(1.1, 0.9, 1.0), segments=10,
                 rings=6)
    # the head: a round face, ears, a big nose, rosy cheeks, eyes under bushy brows
    G.sphere('head', 0.062, at=(0, -0.006, 0.345), material=M.skin, scale=(1, 1.02, 1), segments=14, rings=9)
    for s in (-1, 1):
        G.sphere(f'ear{s}', 0.017, at=(s * 0.06, 0.0, 0.35), material=M.skin, scale=(0.5, 1, 1.3), segments=8, rings=5)
        G.sphere(f'cheek{s}', 0.017, at=(s * 0.03, -0.05, 0.332), material=M.nose, segments=8, rings=5)
        G.sphere(f'eye{s}', 0.0075, at=(s * 0.022, -0.058, 0.36), material=M.eye, segments=8, rings=5)
        G.sweep(f'brow{s}', [(s * 0.012, -0.06, 0.37), (s * 0.026, -0.062, 0.376), (s * 0.038, -0.056, 0.371)], 0.0055,
                material=M.beard, verts=5, resolution=2)
    G.sphere('nose', 0.021, at=(0, -0.066, 0.338), material=M.nose, segments=10, rings=6)
    # the beard: a long wedge from the cheeks to a point on his chest, a little wavy; a moustache over it
    rows = [(0.345, 0.055, 0.032, -0.026), (0.315, 0.062, 0.04, -0.04), (0.28, 0.056, 0.038, -0.055),
            (0.245, 0.042, 0.03, -0.068), (0.215, 0.022, 0.018, -0.078), (0.195, 0.004, 0.004, -0.082)]
    beard = G.loft('beard', [G.ring(rx, ry, z, 10, at=(0, y)) for z, rx, ry, y in rows], material=M.beard)
    G.jitter(beard, 0.004, size=0.03, seed=2.0)
    for s in (-1, 1):
        G.sweep(f'moustache{s}', [(0, -0.07, 0.326), (s * 0.025, -0.07, 0.322), (s * 0.045, -0.062, 0.31)], 0.011,
                material=M.beard, verts=8, resolution=2, scale=[1.0, 0.9, 0.4])
    # the hat: a rolled brim and a tall cone that flops back towards its tip
    G.torus('hat_brim', 0.064, 0.013, at=(0, -0.004, 0.388), material=M.hat, major_segments=20, minor_segments=6)
    G.sweep('hat', [(0, -0.004, 0.385), (0, -0.002, 0.45), (0, 0.01, 0.51), (0, 0.032, 0.56), (0, 0.058, 0.583),
                    (0, 0.078, 0.582)], 0.07, material=M.hat, verts=12, resolution=3,
            scale=[1.0, 0.78, 0.55, 0.32, 0.14, 0.03])
    return G.settle([G.join_all('gnome')])


def _scalloped(rows, lobes, depth, verts=48, scallop_from=0.0):
    """Loft rings for a turned body whose rim is scalloped: `rows` (radius, height), each ring's radius waving by
    `depth` (a fraction) round `lobes` lobes, fading in from rows at or above `scallop_from` height."""
    out = []
    for r, z in rows:
        k = depth if z >= scallop_from else 0.0
        out.append([(r * (1 + k * math.cos(lobes * math.tau * i / verts)) * math.cos(math.tau * i / verts),
                     r * (1 + k * math.cos(lobes * math.tau * i / verts)) * math.sin(math.tau * i / verts), z)
                    for i in range(verts)])
    return out


def birdbath():
    """A cast-stone bird bath: a moulded round plinth, a fluted column under a turned capital, a shallow bowl whose
    rim waves in twelve scallops, water in it, a robin dipping its beak at the rim."""
    G.revolve('plinth', [(0, 0), (0.19, 0), (0.19, 0.02), (0.18, 0.03), (0.17, 0.032), (0.15, 0.05), (0.12, 0.062),
                         (0.11, 0.075), (0, 0.075)], material=M.stone_dark, segments=24)
    G.revolve('column_foot', [(0, 0.07), (0.105, 0.07), (0.108, 0.085), (0.096, 0.1), (0.085, 0.105), (0, 0.105)],
              material=M.stone, segments=20)
    flutes = [((0.078 if k % 2 == 0 else 0.07) * math.cos(math.pi * k / 8), (0.078 if k % 2 == 0 else 0.07)
               * math.sin(math.pi * k / 8)) for k in range(16)]
    G.sweep('column', [(0, 0, 0.1), (0, 0, 0.36), (0, 0, 0.56)], None, material=M.stone, profile=flutes,
            scale=[1.0, 0.86, 0.95], angle=60)
    G.revolve('capital', [(0, 0.55), (0.08, 0.55), (0.09, 0.565), (0.082, 0.58), (0.095, 0.6), (0.13, 0.625),
                          (0.14, 0.64), (0, 0.64)], material=M.stone, segments=20)
    # the bowl: under it, out and up to a rolled, scalloped rim, then down inside to the water
    rows = [(0.12, 0.62), (0.2, 0.635), (0.28, 0.66), (0.33, 0.72), (0.345, 0.79), (0.34, 0.82), (0.325, 0.832),
            (0.305, 0.828), (0.29, 0.81), (0.27, 0.77), (0.2, 0.75), (0.1, 0.745)]
    G.loft('bowl', _scalloped(rows, 12, 0.025, scallop_from=0.7), material=M.stone, angle=70)
    G.cylinder('water', 0.268, 0.012, at=(0, 0, 0.792), material=M.water, verts=40)
    G.torus('ripple', 0.06, 0.003, at=(-0.185, 0.175, 0.799), material=M.water, major_segments=16, minor_segments=3)
    # a robin on the rim at the back, turned in to drink
    G.kit.songbird(F, 'robin', 0.24, 0.832, -0.225, M.bird_brown, M.bird_breast, M.beak, M.eye, turn=-47, head=35)
    return G.settle([G.join_all('birdbath')])


def fountain():
    """A two-tier stone fountain: an octagonal step, a round basin with a moulded rim, a turned stem with four
    spouts pouring into it, an upper bowl with a scalloped lip spilling over in four streams, a tiered finial. The
    water moves all the time near the camera (`flow`, ambient): the jet bubbling up, rings on the basins."""
    G.revolve('plinth', [(0, 0), (0.47, 0), (0.47, 0.035), (0.455, 0.05), (0.42, 0.06), (0.41, 0.09), (0, 0.09)],
              material=M.stone_dark, segments=8, smooth=False)
    G.revolve('basin', [(0.34, 0.09), (0.43, 0.09), (0.44, 0.105), (0.43, 0.12), (0.445, 0.27), (0.465, 0.285),
                        (0.465, 0.31), (0.45, 0.335), (0.42, 0.35), (0.385, 0.35), (0.37, 0.33), (0.375, 0.3),
                        (0.39, 0.12), (0.34, 0.11)], material=M.stone, segments=32)
    G.cylinder('water', 0.385, 0.014, at=(0, 0, 0.29), material=M.water, verts=32)
    G.revolve('stem', [(0, 0.28), (0.13, 0.29), (0.135, 0.31), (0.115, 0.33), (0.1, 0.38), (0.092, 0.45),
                       (0.1, 0.5), (0.115, 0.52), (0.105, 0.535), (0.13, 0.56), (0.2, 0.6), (0.21, 0.625), (0, 0.625)],
              material=M.stone, segments=24)
    # four spouts round the stem, each pouring an arc of water down into the basin
    for k in range(4):
        a = math.radians(45 + 90 * k)
        c, s = math.cos(a), math.sin(a)
        G.sweep(f'spout{k}', [(0.08 * c, 0.08 * s, 0.47), (0.115 * c, 0.115 * s, 0.47), (0.13 * c, 0.13 * s, 0.462)], 0.014,
                material=M.stone, verts=8, scale=[1.3, 1.0, 0.8])
        G.sweep(f'stream{k}', [(0.135 * c, 0.135 * s, 0.458), (0.17 * c, 0.17 * s, 0.44), (0.2 * c, 0.2 * s, 0.38),
                               (0.215 * c, 0.215 * s, 0.296)], 0.007, material=M.water, verts=6, resolution=3,
                scale=[1.0, 1.1, 1.2, 1.5])
    # the upper bowl: out and up to a scalloped lip, down inside to its water; four streams spill over the lip
    rows = [(0.12, 0.6), (0.19, 0.625), (0.235, 0.67), (0.25, 0.715), (0.245, 0.745), (0.225, 0.752), (0.205, 0.735),
            (0.18, 0.7), (0.08, 0.69)]
    G.loft('bowl2', _scalloped(rows, 8, 0.04, verts=32, scallop_from=0.66), material=M.stone, angle=70)
    G.cylinder('water2', 0.195, 0.012, at=(0, 0, 0.73), material=M.water, verts=32)
    for k in range(4):
        a = math.tau * k / 4
        c, s = math.cos(a), math.sin(a)
        G.sweep(f'spill{k}', [(0.225 * c, 0.225 * s, 0.738), (0.262 * c, 0.262 * s, 0.742), (0.285 * c, 0.285 * s, 0.7),
                              (0.31 * c, 0.31 * s, 0.5), (0.325 * c, 0.325 * s, 0.296)], None, material=M.water,
                profile=[(-0.004, -0.02), (0.004, -0.02), (0.004, 0.02), (-0.004, 0.02)], up=(0, 0, 1), resolution=3,
                scale=[1.0, 1.0, 0.8, 0.65, 0.9])
    # the finial: a turned spindle in tiers, a knob on top (the jet comes up out of it)
    G.revolve('finial', [(0, 0.735), (0.07, 0.74), (0.072, 0.758), (0.05, 0.775), (0.045, 0.81), (0.062, 0.83),
                         (0.06, 0.845), (0.04, 0.86), (0.036, 0.895), (0.045, 0.905), (0.03, 0.915), (0, 0.92)],
              material=M.stone, segments=16)
    G.sphere('knob', 0.05, at=(0, 0, 0.945), material=M.stone, segments=14, rings=8)
    body = G.join(G.meshes(), 'fountain', origin=(0, 0, 0))
    # The water moves all the time (`flow`, ambient): the jet bubbles up from the knob and falls back, and a ring
    # on each basin's water swells out from where the water falls and settles back.
    with G.new_parts() as jet:
        G.loft('jet', [G.ring(r, r, z, 10) for z, r in ((0.97, 0.022), (1.0, 0.014), (1.06, 0.012), (1.085, 0.02),
                                                         (1.11, 0.03), (1.13, 0.024), (1.142, 0.008))], material=M.water)
    jet = G.join(jet, 'jet', origin=(0, 0, 0.955))
    with G.new_parts() as rings:
        G.torus('ripple2', 0.1, 0.006, at=(0, 0, 0.742), material=M.water, major_segments=24, minor_segments=4)
        G.torus('ripple', 0.24, 0.007, at=(0, 0, 0.298), material=M.water, major_segments=32, minor_segments=4)
    rings = G.join(rings, 'ripples', origin=(0, 0, 0.3))
    objs = G.settle([body, jet, rings])
    G.squash(jet, 'flow', (1, 1, 1.35), seconds=1.2, back=True)
    G.squash(rings, 'flow', (1.4, 1.4, 1), seconds=1.2, back=True)
    return objs


def _hex(r, z, n=6, flat=None, lift=0.0, phase=0.0):
    """A loop round the lantern's axis at height `z`: a hexagon `r` to its corners (corners to the sides, flats to the
    front and back), or with `n` 12 its corners and the middles of its flats (`flat`: their radius; `lift`: the
    corners raised, a roof's upturned eaves); `phase` turns it (degrees)."""
    out = []
    for j in range(n):
        a = math.radians(phase + 360 * j / n)
        corner = n == 6 or j % 2 == 0
        rr = r if corner else (flat if flat is not None else r * math.cos(math.pi / 6))
        out.append((rr * math.cos(a), rr * math.sin(a), z + (lift if corner else 0.0)))
    return out


def _petals(r0, r1, z0, z1, n=12, depth=0.12):
    """A ring of carved lotus petals from radius r0 at z0 to r1 at z1: rings of `n` x 3 points, each petal's middle
    proud by `depth` (a fraction), its edges a little in."""
    def ring(r, z):
        out = []
        for j in range(3 * n):
            k = r * (1 + depth if j % 3 == 1 else 1 - depth * 0.25)
            out.append((k * math.cos(math.tau * j / (3 * n)), k * math.sin(math.tau * j / (3 * n)), z))
        return out
    return [ring(r0, z0), ring(r1, z1)]


def lantern():
    """A Japanese stone lantern (tōrō): a hexagonal footing under a ring of lotus petals, a round post with a band, a
    platform on petals, the fire box between six posts (a moon window in four sides, open to the front and back,
    the warm light inside), a roof with upturned corners curling up, and the jewel on a lotus cup."""
    G.loft('base', [_hex(0.23, 0.0), _hex(0.23, 0.06), _hex(0.21, 0.085), _hex(0.17, 0.095)], material=M.stone_dark,
           smooth=False)
    G.loft('base_petals', _petals(0.165, 0.11, 0.09, 0.14), material=M.stone_dark, angle=60)
    G.revolve('column', [(0, 0.13), (0.085, 0.13), (0.08, 0.15), (0.072, 0.28), (0.082, 0.29), (0.082, 0.31), (0.072, 0.32),
                         (0.068, 0.48), (0.075, 0.5), (0, 0.5)], material=M.stone, segments=16)
    G.loft('platform_petals', _petals(0.09, 0.17, 0.48, 0.535), material=M.stone, angle=60)
    G.loft('platform', [_hex(0.16, 0.53), _hex(0.2, 0.545), _hex(0.2, 0.57), _hex(0.15, 0.575)], material=M.stone,
           smooth=False)
    # the fire box: the light inside, six corner posts, side panels pierced by moon windows, open front and back
    G.loft('firebox', [_hex(0.11, 0.575), _hex(0.11, 0.77)], material=M.glow, smooth=False)
    for i in range(6):
        a = 60 * i
        r = 0.135
        G.cone(f'post{i}', 0.026, 0.024, 0.2, at=(r * G.cosd(a), r * G.sind(a), 0.67), material=M.stone, verts=4,
               smooth=False)
    for i in range(6):
        mid = 60 * i + 30
        if mid in (90, 270):            # the front and back stay open
            continue
        hole = [(0.034 * math.cos(math.tau * k / 12), 0.01 + 0.034 * math.sin(math.tau * k / 12)) for k in range(12)]
        panel = G.extrude(f'panel{i}', [(-0.055, -0.09), (0.055, -0.09), (0.055, 0.09), (-0.055, 0.09)], 0.018,
                          material=M.stone, holes=[hole])
        panel.rotation_euler = (0, 0, math.radians(mid - 90 + 180))
        panel.location = (0.122 * G.cosd(mid), 0.122 * G.sind(mid), 0.67)
    G.loft('firebox_top', [_hex(0.15, 0.77), _hex(0.16, 0.785)], material=M.stone, smooth=False)
    # the roof: a hexagon whose corners turn up at the eaves, sweeping in and up to its top, a curl at each corner
    G.loft('roof', [_hex(0.15, 0.78, 12, 0.13), _hex(0.258, 0.81, 12, 0.224, lift=0.022),
                    _hex(0.27, 0.835, 12, 0.234, lift=0.03), _hex(0.2, 0.872, 12, 0.18, lift=0.012),
                    _hex(0.12, 0.915, 12, 0.105), _hex(0.06, 0.945, 12, 0.052)], material=M.stone, angle=55)
    for i in range(6):
        c, s = G.cosd(60 * i), G.sind(60 * i)
        G.sweep(f'curl{i}', [(0.255 * c, 0.255 * s, 0.86), (0.272 * c, 0.272 * s, 0.876), (0.268 * c, 0.268 * s, 0.9),
                             (0.252 * c, 0.252 * s, 0.9), (0.252 * c, 0.252 * s, 0.885)], 0.011, material=M.stone,
                verts=6, resolution=2, scale=[1.2, 1.1, 1.0, 0.9, 0.8])
    G.revolve('cup', [(0, 0.94), (0.045, 0.94), (0.05, 0.952), (0.062, 0.962), (0.058, 0.972), (0.035, 0.97), (0, 0.97)],
              material=M.stone, segments=12)
    G.revolve('jewel', [(0, 0.96), (0.03, 0.965), (0.042, 0.985), (0.04, 1.005), (0.028, 1.02), (0.012, 1.03),
                        (0, 1.035)], material=M.stone, segments=12)
    return G.settle([G.join_all('lantern')])


def wheelbarrow():
    """A wheelbarrow planter: a pressed steel tray with a rolled rim, sloping out over the wheel, on two wooden handles
    with rubber grips that run from the axle, two bent steel legs with stays, a spoked wheel with a treaded tyre; the
    tray full of earth, leafy plants and flowers."""
    rng = random.Random(5)
    green = M.paint
    yc = 0.0
    # the tray: its walls lofted from a rounded bottom out to the rim, deeper over the wheel; the earth in it
    rings = [G.ring(0.19, 0.24, 0.39, 24, square=5, at=(0, -0.03)), G.ring(0.205, 0.26, 0.41, 24, square=5, at=(0, -0.025)),
             G.ring(0.265, 0.33, 0.615, 24, square=5, at=(0, 0.01)), G.ring(0.245, 0.31, 0.6, 24, square=5, at=(0, 0.01))]
    G.loft('tray', rings, material=green, angle=50)
    G.sweep('tray_rim', G.ring(0.265, 0.33, 0.617, 24, square=5, at=(0, 0.01)), 0.013, material=green, verts=6, closed=True)
    G.loft('soil', [G.ring(0.25, 0.315, 0.602, 16, square=5, at=(0, 0.01)), G.ring(0.2, 0.26, 0.625, 16, square=4, at=(0, 0.01)),
                    G.ring(0.1, 0.13, 0.64, 16, square=3, at=(0, 0.01))], material=M.soil)
    # the handles: from the axle back under the tray and out past it, the grips on their ends
    for s in (-1, 1):
        path = G.kit.fillet([(s * 0.1, 0.36, 0.165), (s * 0.15, 0.22, 0.37), (s * 0.17, -0.1, 0.42), (s * 0.205, -0.42, 0.57)],
                            0.06)
        G.sweep(f'handle{s}', path, None, material=M.wood, profile=G.rounded_rect(0.04, 0.034, 0.01, corner=1), up=(0, 0, 1))
        G.sweep(f'grip{s}', [(s * 0.199, -0.39, 0.555), (s * 0.212, -0.47, 0.595)], 0.021, material=M.metal_dark, verts=8,
                scale=[1.0, 0.9])
        # a bent steel leg from under the handle down to a foot, a stay up to the tray
        G.sweep(f'leg{s}', G.kit.fillet([(s * 0.17, -0.08, 0.42), (s * 0.18, -0.19, 0.03), (s * 0.18, -0.26, 0.012)], 0.03),
                0.014, material=M.metal_dark, verts=8)
        G.sweep(f'stay{s}', [(s * 0.178, -0.16, 0.15), (s * 0.12, -0.1, 0.395)], 0.008, material=M.metal_dark, verts=6)
        G.sweep(f'foot{s}', [(s * 0.18, -0.235, 0.008), (s * 0.18, -0.285, 0.008)], 0.009, material=M.metal_dark, verts=6)
        # the axle's bracket
        G.cube(f'bracket{s}', (0.012, 0.06, 0.05), at=(s * 0.1, 0.35, 0.18), material=M.metal, bevel=0.003, bevel_segments=1)
    # the wheel: a treaded tyre, a dished steel rim on six spokes, a hub on its axle
    wheel = (0, 0.36, 0.165)
    G.torus('tyre', 0.13, 0.035, at=wheel, material=M.metal_dark, rot=(0, 90, 0), major_segments=20, minor_segments=6)
    for k in range(16):
        a = math.tau * (k + 0.5) / 16
        G.cube(f'tread{k}', (0.055, 0.014, 0.008), at=(0, 0.36 + 0.161 * math.cos(a), 0.165 + 0.161 * math.sin(a)),
               material=M.metal_dark, rot=(math.degrees(a) - 90, 0, 0))
    G.torus('rim', 0.1, 0.012, at=wheel, material=M.metal, rot=(0, 90, 0), major_segments=20, minor_segments=4)
    for k in range(6):
        a = math.tau * k / 6
        G.sweep(f'spoke{k}', [(0.0, 0.36 + 0.03 * math.cos(a), 0.165 + 0.03 * math.sin(a)),
                              (0.0, 0.36 + 0.095 * math.cos(a), 0.165 + 0.095 * math.sin(a))], 0.006, material=M.metal,
                verts=5)
    G.cylinder('hub', 0.035, 0.08, at=wheel, material=M.metal, rot=(0, 90, 0), verts=12, bevel=0.006, bevel_segments=1)
    G.cylinder('axle', 0.012, 0.22, at=wheel, material=M.metal, rot=(0, 90, 0), verts=8)
    # in the earth: leafy rosettes, flowers over them on short stems, a few leaves trailing over the rim
    cols = [M.flower_pink, M.flower_yellow, M.flower_white]
    for i, (x, y, r) in enumerate([(-0.11, -0.17, 0.09), (0.11, -0.1, 0.085), (-0.08, 0.08, 0.09), (0.1, 0.15, 0.08),
                                   (0.0, -0.02, 0.08), (-0.14, 0.22, 0.07), (0.15, 0.27, 0.06)]):
        lump(f'clump{i}', r, (x, y, 0.6), M.leaf, scale=(1.1, 1.0, 0.75), seed=i * 1.9, noise=0.35)
    for i, (x, y) in enumerate([(-0.12, -0.16), (0.12, -0.04), (-0.07, 0.12), (0.1, 0.18), (-0.16, 0.0), (0.03, -0.22),
                                (0.15, -0.18), (0.0, 0.03)]):
        for k in range(3):
            leaf(f'leaf{i}_{k}', (x, y, 0.64), 120 * k + i * 17, rng.uniform(30, 50), rng.uniform(0.07, 0.1), 0.045, M.leaf,
                 shape='oval', droop=0.3, thick=0.006, fold=0.2, rows=3, ring=3)
        h = rng.uniform(0.1, 0.15)
        star(f'flower{i}', (x + 0.01, y, 0.62 + h), 0.05, 5 + i % 2, cols[i % 3], yaw=i * 40, tilt=rng.uniform(0, 25),
             inner=0.55, cup=0.25)
        dome(f'flower_eye{i}', (x + 0.01, y, 0.62 + h), 0.011, 0.007, M.flower_yellow if i % 3 != 1 else M.wood_dark)
        G.sweep(f'stem{i}', [(x, y, 0.61), (x + 0.01, y, 0.62 + h)], 0.004, material=M.leaf, verts=4)
    for i, (x, y, yaw) in enumerate([(-0.22, -0.2, 260), (0.22, 0.1, 95), (0.1, -0.3, 180)]):
        leaf(f'trail{i}', (x, y, 0.62), yaw, 25, 0.14, 0.05, M.leaf, shape='lance', droop=1.0, thick=0.006, fold=0.2, rows=5,
             ring=3)
    return G.settle([G.join_all('wheelbarrow')])


def _stone(name, radius, at, material, scale, seed, top, fractures, subdivisions=3):
    """A weathered stone `top` high: a boulder roughened (as the woods' rock, families/ground.py), faces split off
    flat by `fractures` (direction, depth in from its outside, metres), and its foot cut off flat where it goes into
    the lawn, so it sits broad and heavy in it, not on it."""
    o = G.boulder(name, radius, at=(at[0], at[1], 0), material=material, scale=(1, 1, 1), seed=seed, noise=0.3,
                  subdivisions=subdivisions)
    _lump(o, 0.1, 4.0, seed)
    o.data.transform(Matrix.Diagonal((*scale, 1.0)))
    for direction, depth in fractures:
        n = Vector(direction).normalized()
        reach = max(v.co.dot(n) for v in o.data.vertices)
        _cut(o, o.matrix_world @ (n * (reach - depth)), n)
    o.location.z = top - max(v.co.z for v in o.data.vertices)
    bpy.context.view_layer.update()
    return _cut(o, (0, 0, 0), (0, 0, -1))


def _on_top(points, sink):
    """Where lines straight down through `points` (x, y) first meet what is built so far, `sink` below it."""
    for o in bpy.context.scene.objects:
        o.data.update()
    depsgraph = bpy.context.evaluated_depsgraph_get()
    depsgraph.update()
    hits = [bpy.context.scene.ray_cast(depsgraph, Vector((x, y, 5)), Vector((0, 0, -1)), distance=10)[:2] for x, y in points]
    return [at - Vector((0, 0, sink)) for hit, at in hits if hit]


def boulder():
    """Three weathered garden stones, broader than tall, in three greys: rough, faces split off flat, their feet cut
    flat where they sink into the lawn; moss in patches on the shaded tops and in the crack between the two big
    ones, pale lichen on the faces, tufts of long grass round them and a few pebbles."""
    # The two smaller stones huddle on one side of the big one (front and back of it, they read as a head and a tail).
    _stone('boulder_main', 0.34, (-0.06, 0.02), M.rock, (1.12, 0.94, 0.86), 0.4, 0.43,
           [((0.1, -0.05, 1), 0.07), ((0.35, -0.2, 1), 0.05), ((-0.9, 0.3, 0.35), 0.06), ((0.2, 1, 0.1), 0.05),
            ((0.6, -0.8, 0.25), 0.05), ((-0.5, -0.9, 0.1), 0.04)])
    _stone('boulder_side', 0.21, (0.3, 0.17), M.rock_dark, (1.0, 0.92, 0.8), 2.1, 0.24, [((0.5, -0.6, 1), 0.03),
                                                                                       ((1, 0.2, 0.3), 0.04)])
    _stone('boulder_small', 0.13, (0.26, -0.27), M.rock_warm, (1.05, 0.85, 0.75), 4.2, 0.12, [((0.3, -0.4, 1), 0.02)],
           subdivisions=2)
    moss = _on_top([(-0.14, 0.12), (0.02, 0.18), (0.32, 0.24)], 0.03)
    lichen = _on_top([(0.08, -0.08), (-0.28, -0.1), (0.26, -0.27)], 0.012)
    for i, at in enumerate(moss):
        G.boulder(f'moss{i}', (0.1, 0.07, 0.06)[i], at=tuple(at), material=M.moss, scale=(1.3, 1.0, 0.45), seed=7.3 + i, noise=0.45,
                  subdivisions=2)
    for i, at in enumerate(lichen):
        G.boulder(f'lichen{i}', (0.06, 0.05, 0.035)[i], at=tuple(at), material=M.lichen, scale=(1.3, 1.0, 0.25), seed=3.1 + i,
                  noise=0.35, subdivisions=2)
    G.boulder('moss_crack', 0.08, at=(0.19, 0.06, 0), material=M.moss, scale=(0.8, 1.3, 0.8), seed=9.1, noise=0.4, subdivisions=2)
    # tufts of long grass where the stones meet the lawn: a low clump, blades fanning out of it
    rng = random.Random(17)
    for i, (x, y) in enumerate([(0.45, 0.0), (-0.41, 0.06), (0.0, 0.37), (-0.14, -0.33), (0.06, -0.4)]):
        G.boulder(f'tuft{i}', 0.055, at=(x, y, 0), material=M.leaf, scale=(1.4 if i % 2 else 0.9, 0.9 if i % 2 else 1.4, 0.5),
                  seed=i * 2.3, noise=0.45, subdivisions=1)
        for k in range(9):
            leaf(f'blade{i}_{k}', (x + rng.uniform(-0.03, 0.03), y + rng.uniform(-0.03, 0.03), 0.0), rng.uniform(0, 360),
                 rng.uniform(50, 80), rng.uniform(0.12, 0.2), 0.016, M.leaf, shape='strap', droop=rng.uniform(0.15, 0.4),
                 thick=0.004, fold=0.3, rows=3, ring=3)
    # a few loose pebbles at the big stone's foot
    for i, (x, y) in enumerate([(-0.3, -0.24), (-0.22, -0.31), (0.38, -0.16), (-0.42, -0.08)]):
        G.boulder(f'pebble{i}', 0.022 + 0.006 * (i % 2), at=(x, y, 0), material=M.rock_warm if i % 2 else M.rock_dark,
                  scale=(1.3, 1.0, 0.6), seed=5.5 + i, noise=0.3, subdivisions=1)
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
     G.Model('boulder', boulder, size=(1.008, 0.471, 0.922), tolerance=TOLERANCE,
             paint=[('rock', 'Large boulder'), ('rock_dark', 'Middle boulder'), ('rock_warm', 'Small boulder')])],
    pack='gardenDecor',
    comment='Blender-built garden decor (tools/art/blender/families/garden_decor.py): a gnome, a bird '
            'bath, a two-tier fountain, a stone lantern, a wheelbarrow planter and boulders. Listed in '
            'the base manifest\'s packs. Each file\'s materials map by name onto the game\'s finishes '
            '(`materials`, see render/babylon/models.ts), so a model costs one draw call per finish group '
            'and lights like the rest of the garden.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

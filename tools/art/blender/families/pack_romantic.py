# The romantic pack (P15): the candlelit bistro table, the heart-shaped petal tub, the rose-gold vanity, the
# moonlight gramophone, the love-letter bureau and the rose-arbour swing.
# Notes: pack_romantic.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`), with its moving parts: the candle flames grow (`candles`), the tub fills with petals on the water
# (`fill`), the vanity's drawer opens (`drawer_open`), the gramophone's tone arm swings onto the record (`needle`)
# while the platter turns (`platter`, a loop), the bureau's writing flap drops (`fall_front`) and the arbour's
# swing sways with its riders (`swing`, a loop) while a bloom is cut (`cut`).
#
# Run:  python3 -B tools/art/blender/build.py pack_romantic [--only heartTub] [--preview]
import math
import os
import sys

import bpy

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402
from gamekit import shapes  # noqa: E402

M = G.Palette(
    body=('#C9A54A', 'metal'),       # gold: the table's and chairs' legs; the main part of the table
    fabric=('#E8B4B8', 'fabric'),    # blush velvet: chair seats and backs
    cloth=('#F4EDE4', 'fabric'),     # the tablecloth, towels
    runner=('#9E2F44', 'fabric'),
    ivory=('#F4EDE4', 'satin'),      # the bureau, the vanity, the tub's plinth
    tub=('#E8B4B8', 'gloss'),        # the heart tub's shell
    blush=('#E8B4B8', 'satin'),
    blushd=('#D99AA0', 'satin'),
    velvet=('#E8B4B8', 'satin'),     # the vanity's pouf (plain)
    rose=('#C8475A', 'satin'),
    rose_pink=('#E07A8A', 'satin'),
    rosed=('#9E2F44', 'gloss'),      # wax seal, wine
    rosegold=('#D4A28C', 'metal'),
    gold=('#C9A54A', 'metal'),
    oak=('#CDB08A', 'satin'),
    wax=('#FBF6EC', 'satin'),
    flame=('#F28C28', 'gloss'),
    flame_core=('#FFC24A', 'gloss'),
    porcelain=('#FBFAF7', 'gloss'),
    glassy=('#E4EEEE', 'gloss'),
    dessert=('#C77D4F', 'matte'),
    water=('#C7E6EE', 'gloss'),
    bubble=('#FBFAF7', 'gloss'),
    chrome=('#C9CCD0', 'chrome'),
    bottle=('#2E5B3A', 'gloss'),
    leaf=('#5E8F4E', 'satin'),
    leaf_dark=('#4F7F3E', 'satin'),
    mirror=('#DCE6EA', 'gloss', 'front'),
    perfume_a=('#F2B8C6', 'gloss'),
    perfume_b=('#F2C77F', 'gloss'),
    perfume_c=('#E8C9F0', 'gloss'),
    cabinet=('#A8754A', 'satin'),    # the gramophone's record cabinet; its main part
    walnut=('#8A5E3C', 'satin'),
    door=('#B5835A', 'satin'),
    horn=('#C9A54A', 'metal'),
    record=('#2A2422', 'gloss'),
    sleeve_a=('#C8475A', 'matte'),
    sleeve_b=('#3E6F9E', 'matte'),
    paper=('#FBF7EE', 'matte'),
    envelope=('#F2D7C6', 'matte'),
    ink=('#2E3C6E', 'gloss'),
    frame=('#F2EFE8', 'satin'),      # the arbour's paint; its main part
    cushion=('#E8B4B8', 'satin'),    # the swing's seat cushion
)

WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


def _candle(F, x, z, y, h=0.22, holder=None):
    """A candlestick standing on `y`; returns the wick's top (x, height, z) for its flame."""
    G.revolve(_name('candlestick'), [(0, 0), (0.035, 0), (0.033, 0.012), (0.012, 0.025), (0.012, 0.045), (0.022, 0.06),
                                     (0, 0.06)], at=F.at(x, y, z), material=holder or M.gold, segments=14)
    F.cyl(_name('candle'), 0.014, h, (x, y + 0.06 + h / 2, z), M.wax, verts=12)
    return (x, y + 0.06 + h, z)


def _flame(F, x, y, z, s=1.0):
    F.sphere(_name('flame'), 0.012 * s, (x, y + 0.02 * s, z), M.flame, scale=(1, 1, 2.2), segments=10, rings=6)
    F.sphere(_name('flame_core'), 0.007 * s, (x, y + 0.014 * s, z), M.flame_core, scale=(1, 1, 1.8), segments=8, rings=6)


def _rose(F, x, y, z, s=0.05, material=None):
    """A bloom: an outer cup and a tighter heart."""
    m = material or M.rose
    F.sphere(_name('rose'), s, (x, y, z), m, scale=(1, 1, 0.8), segments=10, rings=6)
    G.torus(_name('petals'), s * 0.75, s * 0.32, at=F.at(x, y + s * 0.35, z), material=m, major_segments=10,
            minor_segments=5)


def _wine_glass(F, x, y, z, wine=True):
    G.revolve(_name('wine_glass'), [(0, 0), (0.03, 0), (0.03, 0.005), (0.004, 0.01), (0.004, 0.08), (0.032, 0.11),
                                    (0.035, 0.17), (0.031, 0.17), (0, 0.09)], at=F.at(x, y, z), material=M.glassy, segments=12)
    if wine:
        F.sphere(_name('wine'), 0.028, (x, y + 0.115, z), M.rosed, scale=(1, 1, 0.6), segments=10, rings=6)


def _velvet_chair(F, x, z=0.0, frame=None, seat=None):
    """A slipper chair centred on (x, z), facing the front: seat top 0.45."""
    frame, seat = frame or M.body, seat or M.fabric
    F.box(_name('chair_seat'), 0.44, 0.1, 0.42, (x, 0.4, z), seat, bevel=0.04, bevel_segments=3)
    F.box(_name('chair_back'), 0.42, 0.46, 0.09, (x, 0.7, z - 0.21), seat, rot=(-8, 0, 0), bevel=0.04, bevel_segments=3)
    for dx in (-0.18, 0.18):
        for dz in (0.16, -0.17):
            F.cone(_name('chair_leg'), 0.012, 0.018, 0.36, (x + dx, 0.18, z + dz), frame, verts=8)


# --- the candlelit bistro table (2x1): two diners side by side at x = +-0.5, the table before them ------------
CANDLES = ((-0.12, 0.33, 0.26), (0.12, 0.33, 0.22))   # (x, depth, candle height)


def candlelitTable():
    F = G.GameFrame()
    top = 0.775
    with G.new_parts() as still:
        F.box('table_top', 1.62, 0.03, 0.3, (0, 0.745, 0.29), M.oak, bevel=0.008)
        F.box('cloth', 1.68, 0.012, 0.32, (0, 0.766, 0.29), M.cloth, bevel=0.004)
        F.box('cloth_front', 1.68, 0.2, 0.012, (0, 0.67, 0.449), M.cloth)
        for x in (-0.84, 0.84):
            F.box(_name('cloth_side'), 0.012, 0.2, 0.32, (x, 0.67, 0.29), M.cloth)
        F.box('runner', 1.7, 0.004, 0.12, (0, 0.774, 0.3), M.runner)
        for x in (-0.78, 0.78):
            F.cone(_name('table_leg'), 0.02, 0.014, 0.73, (x, 0.365, 0.2), M.body, verts=10)
        for s in (-1, 1):
            x = s * 0.5
            F.cyl(_name('plate'), 0.13, 0.012, (x, 0.782, 0.26), M.porcelain, verts=22)
            G.torus(_name('plate_rim'), 0.122, 0.005, at=F.at(x, 0.788, 0.26), material=M.gold, major_segments=22,
                    minor_segments=4)
            G.revolve(_name('dessert'), [(0, 0), (0.05, 0), (0.045, 0.04), (0.02, 0.055), (0, 0.055)],
                      at=F.at(x, 0.788, 0.26), material=M.dessert, segments=12)
            F.sphere(_name('berry'), 0.012, (x, 0.85, 0.26), M.rose, segments=8, rings=6)
            for dx in (-0.16, 0.16):
                F.box(_name('cutlery'), 0.012, 0.004, 0.17, (x + dx, 0.778, 0.26), M.gold)
            _wine_glass(F, x + s * 0.2, top, 0.38)
            _velvet_chair(F, x, 0.0)
        for x, z, h in CANDLES:
            _candle(F, x, z, top, h)
        G.revolve('bud_vase', [(0, 0), (0.025, 0), (0.03, 0.05), (0.012, 0.13), (0.015, 0.18), (0, 0.17)],
                  at=F.at(0, top, 0.37), material=M.glassy, segments=12)
        G.rod('stem', F.at(0, 0.86, 0.37), F.at(0.01, 1.06, 0.37), 0.005, material=M.leaf, verts=6)
        _rose(F, 0.01, 1.08, 0.37, 0.045)
        for x, z in ((-0.3, 0.38), (0.25, 0.4), (-0.05, 0.22), (0.38, 0.33)):
            F.sphere(_name('petal'), 0.025, (x, 0.778, z), M.rose, scale=(1.2, 0.9, 0.25), segments=8, rings=4)
    G.join(still, 'body', origin=(0, 0, 0))
    # The two flames, joined with their origin on the shorter wick and built full size, then flattened:
    # `candles` grows them while the two dine.
    with G.new_parts() as flames:
        for x, z, h in CANDLES:
            _flame(F, x, top + 0.06 + h, z, 1.1)
    low = min(h for _, _, h in CANDLES)
    flames = G.join(flames, 'flames', origin=F.at(0, top + 0.05 + low, 0.33))   # flattened inside the wax at rest
    flames.scale[2] = 0.001
    objs = G.settle(centre=False)
    G.grow(flames, 'candles', seconds=0.6)
    return objs


# --- the heart-shaped petal tub (2x2): two bathers sit in it at x = +-0.5 -----------------------------------
def _heart(k=1.0, n=72):
    """The heart's outline in game (x, depth), its point to the front, scaled by `k` about its middle."""
    pts = []
    for i in range(n):
        t = 2 * math.pi * i / n
        hx = 16 * math.sin(t) ** 3
        hy = 13 * math.cos(t) - 5 * math.cos(2 * t) - 2 * math.cos(3 * t) - math.cos(4 * t)
        pts.append((hx * 0.055 * k, -(hy + 2.5) * 0.0552 * k + 0.08))
    return pts


def _prism(name, outer, y0, y1, material, inner=None):
    """A mesh between heights `y0` and `y1` over the outline `outer` (game x, depth), hollow inside `inner`."""
    F = G.GameFrame()
    n = len(outer)
    verts = [F.at(x, y, z) for y in (y0, y1) for x, z in outer]
    faces = [(i, (i + 1) % n, n + (i + 1) % n, n + i) for i in range(n)]
    if inner is None:
        faces += [tuple(range(n)), tuple(range(2 * n - 1, n - 1, -1))]
    else:
        verts += [F.at(x, y, z) for y in (y0, y1) for x, z in inner]
        o = 2 * n
        faces += [(o + i, o + n + i, o + n + (i + 1) % n, o + (i + 1) % n) for i in range(n)]     # the inner wall
        faces += [(n + i, n + (i + 1) % n, o + n + (i + 1) % n, o + n + i) for i in range(n)]     # the rim
        faces += [(i, o + i, o + (i + 1) % n, (i + 1) % n) for i in range(n)]                     # the underside
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    shapes._recalc_normals(obj)
    return obj


RIM, FLOOR, WATER = 0.54, 0.2, 0.47


def heartTub():
    F = G.GameFrame()
    with G.new_parts() as still:
        _prism('plinth', _heart(1.04), 0.0, 0.08, M.ivory)
        _prism('shell', _heart(0.96), 0.08, RIM, M.tub, inner=_heart(0.84))
        _prism('tub_floor', _heart(0.85), 0.08, FLOOR, M.tub)
        # an ice bucket with a bottle on the rim, towels, candles and roses on the floor
        G.revolve('ice_bucket', [(0, 0), (0.045, 0), (0.06, 0.14), (0.055, 0.14), (0.04, 0.012), (0, 0.012)],
                  at=F.at(0.6, RIM, 0.3), material=M.chrome, segments=16)
        F.cyl('bottle', 0.03, 0.2, (0.6, RIM + 0.14, 0.3), M.bottle, rot=(0, 15, 0), verts=12)
        F.box('towel_a', 0.34, 0.08, 0.24, (-0.78, 0.04, 0.78), M.cloth, bevel=0.03, bevel_segments=2)
        F.box('towel_b', 0.32, 0.07, 0.22, (-0.78, 0.115, 0.78), M.fabric, bevel=0.03, bevel_segments=2)
        for x, z, h in ((-0.85, -0.82, 0.3), (-0.72, -0.86, 0.2), (0.85, -0.82, 0.26), (0.73, -0.87, 0.18)):
            wx, wy, wz = _candle(F, x, z, 0.0, h, holder=M.rosegold)
            F.sphere(_name('flame'), 0.012, (wx, wy + 0.02, wz), M.flame_core, scale=(1, 1, 2), segments=8, rings=6)
        G.revolve('floor_vase', [(0, 0), (0.06, 0), (0.08, 0.15), (0.05, 0.27), (0.055, 0.3), (0, 0.29)],
                  at=F.at(0.7, 0, 0.82), material=M.ivory, segments=16)
        for x, z, m in ((0.68, 0.84, M.rose), (0.74, 0.8, M.rose_pink)):
            G.rod(_name('stem'), F.at(0.7, 0.25, 0.82), F.at(x, 0.42, z), 0.006, material=M.leaf, verts=6)
            _rose(F, x, 0.44, z, 0.05, m)
    G.join(still, 'body', origin=(0, 0, 0))
    # The water with its petals and bubbles, built full and flattened onto the tub's floor: `fill` raises it.
    with G.new_parts() as water:
        _prism('water', _heart(0.845), FLOOR - 0.01, WATER, M.water)
        for i in range(26):
            a = i * 2.39996
            rr = 0.1 + 0.5 * math.sqrt((i + 0.5) / 26)
            x, z = rr * math.cos(a) * 0.95, -0.05 + rr * math.sin(a) * 0.72
            F.sphere(_name('petal'), 0.03, (x, WATER + 0.004, z), M.rose if i % 3 else M.rose_pink,
                     scale=(1.3, 1.0, 0.2), rot=(0, 0, i * 37 % 180), segments=8, rings=4)
        for x, z in ((-0.18, -0.35), (0.25, -0.42), (0.05, 0.3), (-0.4, 0.05)):
            F.sphere(_name('bubble'), 0.05, (x, WATER, z), M.bubble, scale=(1, 1, 0.5), segments=10, rings=6)
    water = G.join(water, 'water', origin=F.at(0, FLOOR - 0.01, 0))   # hidden in the floor at rest
    water.scale[2] = 0.001
    objs = G.settle(centre=False)
    G.grow(water, 'fill', seconds=1.2)
    return objs


# --- the rose-gold vanity (1x1): a dressing table with an oval mirror, a pouf before it ---------------------
def vanity():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('vanity_top', 0.8, 0.05, 0.36, (0, 0.735, -0.25), M.ivory, bevel=0.012)
        F.box('apron', 0.76, 0.14, 0.33, (0, 0.64, -0.25), M.ivory, bevel=0.008)
        F.box('drawer_hole', 0.36, 0.11, 0.01, (0, 0.635, -0.082), M.blushd)
        for x in (-0.36, 0.36):
            for z in (-0.4, -0.11):
                F.cone(_name('leg'), 0.012, 0.02, 0.57, (x, 0.285, z), M.rosegold, verts=10)
        # the oval mirror in a rose-gold frame on a little stand at the back
        F.cyl('mirror_back', 0.25, 0.02, (0, 1.24, -0.415), M.rosegold, rot=(90, 0, 0), scale=(1, 1.44, 1), verts=36)
        G.torus('mirror_frame', 0.26, 0.018, at=F.at(0, 1.24, -0.405), material=M.rosegold, rot=(90, 0, 0),
                major_segments=36, minor_segments=6).scale = (1, 1.44, 1)
        F.box('mirror_stand', 0.05, 0.14, 0.04, (0, 0.82, -0.41), M.rosegold, bevel=0.01)
        F.cyl('mirror', 0.235, 0.004, (0, 1.24, -0.398), M.mirror, rot=(90, 0, 0), scale=(1, 1.44, 1), verts=36)
        for i, (x, h, m) in enumerate(((-0.3, 0.12, M.perfume_a), (-0.22, 0.09, M.perfume_b), (-0.15, 0.14, M.perfume_c))):
            G.revolve(_name('perfume'), [(0, 0), (0.03, 0), (0.03, h * 0.7), (0.01, h * 0.85), (0.01, h), (0, h)],
                      at=F.at(x, 0.76, -0.22), material=m, segments=12)
            F.sphere(_name('stopper'), 0.017, (x, 0.775 + h, -0.22), M.rosegold, segments=10, rings=6)
        F.box('jewellery_box', 0.16, 0.08, 0.12, (0.25, 0.8, -0.22), M.blushd, bevel=0.015)
        F.box('jewellery_lid', 0.17, 0.012, 0.13, (0.25, 0.846, -0.22), M.rosegold, bevel=0.004)
        G.revolve('vase', [(0, 0), (0.03, 0), (0.04, 0.07), (0.025, 0.12), (0.03, 0.14), (0, 0.13)],
                  at=F.at(0.1, 0.76, -0.36), material=M.ivory, segments=14)
        for x, y, z, m in ((0.07, 0.95, -0.34, M.rose), (0.13, 0.96, -0.37, M.rose_pink), (0.1, 0.98, -0.33, M.ivory)):
            _rose(F, x, y, z, 0.035, m)
        for x, z in ((0.04, -0.36), (0.16, -0.35)):
            F.sphere(_name('leaf'), 0.03, (x, 0.92, z), M.leaf, scale=(1.3, 0.6, 1), segments=8, rings=6)
        # the pouf before it, tufted, on a rose-gold ring
        G.revolve('pouf', [(0, 0), (0.19, 0), (0.21, 0.04), (0.21, 0.34), (0.2, 0.42), (0.17, 0.44), (0, 0.44)],
                  at=F.at(0, 0.0, 0.17), material=M.velvet, segments=28)
        G.torus('pouf_ring', 0.21, 0.012, at=F.at(0, 0.03, 0.17), material=M.rosegold, major_segments=28,
                minor_segments=6)
        for a in range(0, 360, 60):
            F.sphere(_name('tuft'), 0.013, (0.11 * G.cosd(a), 0.44, 0.17 + 0.11 * G.sind(a)), M.blushd, segments=8,
                     rings=6)
    G.join(still, 'body', origin=(0, 0, 0))
    # The middle drawer, pulled out to the front by `drawer_open`.
    with G.new_parts() as drawer:
        F.box('drawer_front', 0.34, 0.1, 0.014, (0, 0.635, -0.075), M.ivory, bevel=0.005)
        F.box('drawer_box', 0.32, 0.08, 0.26, (0, 0.635, -0.2), M.ivory)
        F.sphere('drawer_knob', 0.018, (0, 0.635, -0.063), M.rosegold, segments=10, rings=6)
        F.box('lipstick', 0.02, 0.06, 0.02, (-0.08, 0.69, -0.18), M.rose)
        F.cyl('compact', 0.035, 0.012, (0.06, 0.68, -0.2), M.rosegold, verts=14)
    drawer = G.hinged(G.join(drawer, 'drawer'), F.at(0, 0.635, -0.075))
    objs = G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, -0.2, 0), seconds=0.7)
    return objs


# --- the moonlight gramophone (1x1): a horn gramophone on a record cabinet -----------------------------------
RECORD = (0.02, 0.886, -0.16)
ARM = (0.17, 0.9, -0.31)   # the tone arm's pivot (x, height, depth)


def gramophone():
    F = G.GameFrame()
    rx, ry, rz = RECORD
    with G.new_parts() as still:
        F.box('cabinet', 0.62, 0.66, 0.4, (0, 0.38, WALL + 0.21), M.cabinet, bevel=0.012)
        F.box('cabinet_top', 0.66, 0.03, 0.44, (0, 0.725, WALL + 0.22), M.walnut, bevel=0.008)
        for s in (-1, 1):
            F.box(_name('cabinet_door'), 0.28, 0.5, 0.012, (s * 0.15, 0.4, WALL + 0.415), M.door, bevel=0.006)
            F.box(_name('door_inset'), 0.2, 0.4, 0.006, (s * 0.15, 0.4, WALL + 0.423), M.cabinet, bevel=0.003)
            F.sphere(_name('door_knob'), 0.016, (s * 0.03, 0.42, WALL + 0.43), M.gold, segments=10, rings=6)
        for x in (-0.27, 0.27):
            for z in (WALL + 0.06, WALL + 0.36):
                G.revolve(_name('foot'), [(0, 0), (0.02, 0), (0.03, 0.03), (0.025, 0.05), (0, 0.05)],
                          at=F.at(x, 0, z), material=M.gold, segments=10)
        # the gramophone: a walnut case, the horn on its neck from the back
        F.box('case', 0.38, 0.13, 0.36, (0, 0.805, -0.18), M.walnut, bevel=0.012)
        F.box('case_band', 0.385, 0.02, 0.365, (0, 0.82, -0.18), M.gold)
        F.cyl('crank_boss', 0.02, 0.03, (-0.205, 0.8, -0.18), M.gold, rot=(0, 90, 0), verts=10)
        G.rod('crank', F.at(-0.22, 0.8, -0.18), F.at(-0.25, 0.74, -0.18), 0.006, material=M.gold, verts=6)
        F.cyl('arm_base', 0.03, 0.03, (ARM[0], 0.885, ARM[2]), M.gold, verts=12)
        G.rod('horn_neck', F.at(0.0, 0.87, -0.34), F.at(0.0, 1.06, -0.32), 0.022, material=M.horn, verts=12)
        G.revolve('horn', [(0.02, 0), (0.03, 0.08), (0.06, 0.2), (0.12, 0.32), (0.23, 0.42), (0.24, 0.43),
                           (0.225, 0.43), (0.115, 0.335), (0.055, 0.215), (0.025, 0.09), (0.012, 0.0)],
                  at=F.at(0.0, 1.05, -0.32), material=M.horn, segments=28, rot=(34, 0, 0))
        # record sleeves leant on the cabinet and a rose on its top
        F.box('sleeve_a', 0.3, 0.3, 0.012, (0.22, 0.15, 0.2), M.sleeve_a, rot=(-12, 20, 0))
        F.box('sleeve_b', 0.3, 0.3, 0.012, (0.2, 0.15, 0.22), M.sleeve_b, rot=(-18, 14, 0))
        G.rod('rose_stem', F.at(-0.22, 0.75, -0.06), F.at(-0.1, 0.752, 0.0), 0.006, material=M.leaf, verts=6)
        _rose(F, -0.22, 0.765, -0.06, 0.05)
    G.join(still, 'body', origin=(0, 0, 0))
    # The platter and record, origin on the spindle: `platter` turns them while the record plays.
    with G.new_parts() as platter:
        F.cyl('platter', 0.15, 0.012, (rx, ry - 0.008, rz), M.gold, verts=32)
        F.cyl('record', 0.145, 0.008, (rx, ry + 0.002, rz), M.record, verts=32)
        F.cyl('label', 0.05, 0.009, (rx, ry + 0.003, rz), M.rose, verts=20)
        F.box('label_mark', 0.04, 0.0095, 0.008, (rx + 0.02, ry + 0.003, rz), M.paper)
        F.cyl('spindle', 0.005, 0.02, (rx, ry + 0.01, rz), M.gold, verts=6)
    platter = G.hinged(G.join(platter, 'platter'), F.at(rx, ry, rz))
    # The tone arm, resting off the record beside it: `needle` swings it onto the record about its pivot.
    ax, ay, az = ARM
    with G.new_parts() as arm:
        F.cyl('arm_pivot', 0.018, 0.04, (ax, ay + 0.005, az), M.gold, verts=10)
        G.rod('arm_tube', F.at(ax, ay + 0.02, az), F.at(ax + 0.02, ay + 0.03, az + 0.2), 0.008, material=M.gold, verts=8)
        F.cyl('sound_box', 0.03, 0.018, (ax + 0.02, ay + 0.025, az + 0.22), M.gold, rot=(90, 0, 0), verts=14)
        G.rod('stylus', F.at(ax + 0.02, ay + 0.02, az + 0.22), F.at(ax + 0.02, ry + 0.006, az + 0.22), 0.002,
              material=M.chrome, verts=4)
    arm = G.hinged(G.join(arm, 'tone_arm'), F.at(ax, ay, az))
    objs = G.settle(centre=False)
    G.swing(arm, 'needle', 30, seconds=0.8)
    G.spin(platter, 'platter', turns=1, seconds=1.8)
    return objs


# --- the love-letter bureau (1x1): the writer sits in the middle, the desk and its pigeonholes before them ---
def letterBureau():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('desk_top', 0.84, 0.03, 0.3, (0, 0.745, 0.3), M.ivory, bevel=0.01)
        F.box('apron', 0.8, 0.1, 0.27, (0, 0.68, 0.31), M.ivory, bevel=0.008)
        F.box('drawer', 0.3, 0.06, 0.01, (0, 0.68, 0.172), M.blush, bevel=0.004)
        F.sphere('drawer_knob', 0.014, (0, 0.68, 0.165), M.gold, segments=8, rings=6)
        for x in (-0.39, 0.39):
            for z in (0.18, 0.42):
                F.cone(_name('leg'), 0.012, 0.02, 0.63, (x, 0.315, z), M.ivory, verts=10)
        # the pigeonhole hutch facing the writer, a gold crest
        F.box('hutch_back', 0.8, 0.38, 0.02, (0, 0.95, 0.44), M.ivory, bevel=0.004)
        for x in (-0.4, 0.4):
            F.box(_name('hutch_side'), 0.02, 0.38, 0.1, (x, 0.95, 0.4), M.ivory, bevel=0.004)
        F.box('hutch_top', 0.84, 0.025, 0.11, (0, 1.15, 0.4), M.ivory, bevel=0.006)
        for x in (-0.2, 0.2):                              # panels on its back, which faces the room
            F.box(_name('hutch_panel'), 0.34, 0.28, 0.008, (x, 0.95, 0.452), M.blush, bevel=0.003)
            F.box(_name('panel_bead'), 0.3, 0.24, 0.004, (x, 0.95, 0.457), M.ivory, bevel=0.002)
        F.box('crest', 0.84, 0.02, 0.11, (0, 1.17, 0.4), M.gold, bevel=0.004)
        for x in (-0.2, 0.0, 0.2):
            F.box(_name('divider'), 0.012, 0.36, 0.09, (x, 0.95, 0.395), M.ivory)
        F.box('shelf', 0.78, 0.012, 0.09, (0, 0.97, 0.395), M.ivory)
        for x, y, m in ((-0.3, 1.0, M.paper), (-0.1, 1.0, M.blush), (0.3, 1.0, M.envelope), (-0.3, 0.8, M.paper),
                        (0.1, 0.8, M.envelope)):
            F.box(_name('letters'), 0.14, 0.08, 0.06, (x, y + 0.015, 0.4), m)
        # on the desk: an inkpot and a quill, a wax seal, a candle
        G.revolve('inkpot', [(0, 0), (0.03, 0), (0.03, 0.035), (0.012, 0.045), (0.012, 0.06), (0, 0.06)],
                  at=F.at(0.3, 0.76, 0.34), material=M.ink, segments=12)
        G.rod('quill', F.at(0.3, 0.8, 0.34), F.at(0.2, 0.98, 0.3), 0.004, material=M.paper, verts=6)
        F.sphere('quill_vane', 0.02, (0.23, 0.93, 0.31), M.paper, scale=(0.5, 0.25, 3.5), rot=(0, 30, 0), segments=8,
                 rings=6)
        F.cyl('wax_seal', 0.025, 0.012, (0.18, 0.766, 0.2), M.rosed, verts=12)
        F.box('wax_stick', 0.012, 0.012, 0.1, (0.3, 0.766, 0.22), M.rose, rot=(0, 0, 30))
        wx, wy, wz = _candle(F, -0.33, 0.22, 0.76, 0.14)
        F.sphere('candle_flame', 0.011, (wx, wy + 0.02, wz), M.flame_core, scale=(1, 1, 2), segments=8, rings=6)
        _velvet_chair(F, 0.0, 0.0, frame=M.ivory)
    G.join(still, 'body', origin=(0, 0, 0))
    # The writing flap, closed up over the lower pigeonholes, hinged on its foot: `fall_front` drops it towards
    # the writer onto the desk, a letter on it.
    with G.new_parts() as flap:
        F.box('flap', 0.76, 0.2, 0.018, (0, 0.87, 0.34), M.ivory, bevel=0.005)
        F.box('flap_panel', 0.66, 0.14, 0.006, (0, 0.87, 0.33), M.blush, bevel=0.003)
        F.sphere('flap_knob', 0.012, (0, 0.95, 0.325), M.gold, segments=8, rings=6)
        F.box('flap_letter', 0.22, 0.14, 0.003, (-0.05, 0.87, 0.35), M.paper, rot=(0, 0, 6))
        F.box('flap_envelope', 0.16, 0.1, 0.003, (0.2, 0.86, 0.35), M.envelope, rot=(0, 0, -8))
    flap = G.hinged(G.join(flap, 'flap'), F.at(0, 0.77, 0.34))
    objs = G.settle(centre=False)
    G.swing(flap, 'fall_front', -90, seconds=0.8, axis='X')
    return objs


# --- the rose-arbour swing (2x1): a porch swing for two under an arch of climbing roses ---------------------
BEAM = 2.1
ARCH = [(-0.88 + 1.76 * t / 8, 2.12 + 0.3 * math.sin(math.pi * t / 8)) for t in range(9)]
BLOOM = (-0.84, 1.36, 0.35)   # the bloom that is cut


def roseArbourSwing():
    F = G.GameFrame()
    with G.new_parts() as still:
        for x in (-0.88, 0.88):
            for z in (-0.36, 0.36):
                F.box(_name('post'), 0.07, 2.12, 0.07, (x, 1.06, z), M.frame, bevel=0.01)
            for i in range(5):                                                   # the lattice
                y0 = 0.2 + i * 0.36
                for za, zb in ((-0.33, 0.33), (0.33, -0.33)):
                    G.rod(_name('lattice'), F.at(x, y0, za), F.at(x, y0 + 0.36, zb), 0.012, material=M.frame, verts=4)
        for z in (-0.36, 0.36):                                                  # the arch
            for (x0, y0), (x1, y1) in zip(ARCH, ARCH[1:]):
                G.rod(_name('arch'), F.at(x0, y0, z), F.at(x1, y1, z), 0.035, material=M.frame, verts=6)
        for x, y in ARCH[1:-1]:
            F.box(_name('slat'), 0.04, 0.04, 0.82, (x, y + 0.03, 0), M.frame, bevel=0.006)
        F.box('beam', 1.82, 0.08, 0.08, (0, BEAM, 0.0), M.frame, bevel=0.012)
        # climbing roses: leafy clumps up the posts and over the arch, blooms among them
        k = 0
        jitter = (0.13, 0.71, 0.42, 0.88, 0.27, 0.56, 0.05, 0.93, 0.36, 0.64, 0.19, 0.81)
        for x in (-0.82, 0.82):
            for z in (-0.3, 0.3):
                for y in (0.5, 1.1, 1.65):
                    yy = y + jitter[k % 12] * 0.2
                    G.boulder(_name('leaves'), 0.13, at=F.at(x, yy - 0.12, z), material=M.leaf if k % 2 else M.leaf_dark,
                              scale=(0.8, 0.7, 1.15), seed=k * 1.7, noise=0.3, subdivisions=1)
                    if (x, z, y) != (-0.82, 0.3, 1.1):
                        _rose(F, x + (0.08 if x < 0 else -0.08), yy + 0.06, z + (0.06 if z < 0 else -0.06),
                              0.045, M.rose if k % 3 else M.rose_pink)
                    k += 1
        for i, (x, y) in enumerate(ARCH[1:-1]):
            G.boulder(_name('leaves'), 0.18, at=F.at(x, y - 0.02, 0), material=M.leaf_dark if i % 2 else M.leaf,
                      scale=(0.9, 2.2, 0.55), seed=i * 2.3, noise=0.3, subdivisions=1)
            _rose(F, x, y + 0.17, -0.25 + 0.17 * (i % 4), 0.05, M.rose if i % 2 else M.rose_pink)
    G.join(still, 'body', origin=(0, 0, 0))
    # The swing (seat top 0.45 over z = 0) with its arms, cushions and chains, hung from the beam: `swing` rocks
    # it, and the residents sitting on it ride it.
    with G.new_parts() as swing:
        F.box('seat', 1.6, 0.05, 0.42, (0, 0.385, -0.01), M.frame, bevel=0.01)
        F.box('seat_cushion', 1.56, 0.06, 0.4, (0, 0.42, 0.0), M.cushion, bevel=0.025, bevel_segments=2)
        F.box('swing_back', 1.6, 0.42, 0.04, (0, 0.66, -0.24), M.frame, rot=(-10, 0, 0), bevel=0.01)
        for x in (-0.55, -0.28, 0.0, 0.28, 0.55):
            F.box(_name('back_rail'), 0.03, 0.4, 0.02, (x, 0.66, -0.215), M.frame, rot=(-10, 0, 0))
        F.box('pillow_l', 0.36, 0.3, 0.1, (-0.6, 0.64, -0.15), M.rose, rot=(-12, 0, -15), bevel=0.04, bevel_segments=2)
        F.box('pillow_r', 0.34, 0.28, 0.1, (0.62, 0.63, -0.15), M.ivory, rot=(-12, 0, 15), bevel=0.04, bevel_segments=2)
        for s in (-1, 1):
            F.box(_name('arm'), 0.05, 0.05, 0.42, (s * 0.78, 0.62, 0.0), M.frame, bevel=0.01)
            F.box(_name('arm_post'), 0.04, 0.2, 0.04, (s * 0.78, 0.5, 0.17), M.frame)
            G.rod(_name('chain'), F.at(s * 0.78, 0.64, 0.17), F.at(s * 0.78, BEAM - 0.04, 0.0), 0.008, material=M.gold,
                  verts=6)
            G.rod(_name('chain'), F.at(s * 0.78, 0.86, -0.26), F.at(s * 0.78, BEAM - 0.04, 0.0), 0.008, material=M.gold,
                  verts=6)
    swing = G.hinged(G.join(swing, 'swing'), F.at(0, BEAM, 0.0))
    # The bloom a resident cuts, origin on its stem: `cut` shrinks it away.
    with G.new_parts() as bloom:
        _rose(F, *BLOOM, s=0.06)
        F.sphere('bloom_leaf', 0.03, (BLOOM[0] + 0.02, BLOOM[1] - 0.05, BLOOM[2] - 0.02), M.leaf, scale=(1.4, 0.6, 1),
                 segments=8, rings=6)
    bloom = G.hinged(G.join(bloom, 'bloom'), F.at(BLOOM[0], BLOOM[1] - 0.06, BLOOM[2]))
    objs = G.settle(centre=False)
    G.rock(swing, 'swing', 7, seconds=2.0)
    G.squash(bloom, 'cut', 0.001, seconds=0.6)
    return objs


FAMILY = G.Family(
    'pack_romantic', M,
    [G.Model('candlelitTable', candlelitTable, size=(1.7, 1.116, 0.736), key='model.romantic.candlelitTable',
             centre=(0, 0.087), motions={'uses': {'candlelitDinner': 'candles', 'shareDessert': 'candles'}},
             paint=[('body', 'Body'), ('fabric', 'Fabric'), ('cloth', 'Tablecloth')]),
     G.Model('heartTub', heartTub, size=(1.865, 0.784, 1.817), key='model.romantic.heartTub',
             centre=(-0.017, 0.004), motions={'uses': {'petalBath': 'fill'}},
             paint=[('tub', 'Body'), ('cloth', 'Fabric')]),
     G.Model('vanity', vanity, size=(0.8, 1.64, 0.822), key='model.romantic.vanity',
             centre=(0, -0.019), motions={'uses': {'primp': 'drawer_open'}, 'hold': 2.0},
             paint=[('ivory', 'Body'), ('velvet', 'Fabric'), ('rosegold', 'Rose gold')]),
     G.Model('gramophone', gramophone, size=(0.742, 1.581, 0.702), key='model.romantic.gramophone',
             centre=(0.041, -0.079), motions={'uses': {'listen': 'needle', 'waltz': 'needle'},
                                          'loops': {'listen': 'platter', 'waltz': 'platter'}},
             paint=[('cabinet', 'Body'), ('horn', 'Horn')]),
     G.Model('letterBureau', letterBureau, size=(0.84, 1.18, 0.736), key='model.romantic.letterBureau',
             centre=(0, 0.087), motions={'uses': {'writeLetter': 'fall_front', 'writePoem': 'fall_front'}},
             paint=[('ivory', 'Body'), ('fabric', 'Fabric')]),
     G.Model('roseArbourSwing', roseArbourSwing, size=(1.844, 2.63, 0.821), key='model.romantic.roseArbourSwing',
             centre=(0, 0.0), motions={'uses': {'tendRoses': 'cut'}, 'loops': {'swing': 'swing'}, 'ride': 'swing'},
             paint=[('frame', 'Body'), ('cushion', 'Fabric')])],
    comment='Blender-built romantic pack models (tools/art/blender/families/pack_romantic.py). Listed in '
            'the base manifest\'s packs. The materials map onto the game\'s finishes so a model costs one '
            'draw call per finish group. Each moves while it is used: the candles light, the tub fills '
            'with petals, the vanity\'s drawer opens, the gramophone\'s arm drops onto the turning record, '
            'the bureau\'s flap drops and the arbour swing sways while a bloom is cut.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

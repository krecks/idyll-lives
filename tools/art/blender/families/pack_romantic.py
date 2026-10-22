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
# Roses are turned cups of petals round a bud (`_rose`), their leaves `plants.leaf`; the tablecloth is lofted and
# falls in folds; the heart tub's shell is lofted through heart-shaped rings, its rim rolled; chairs and the pouf
# are stuffed (`G.soft_box`), the vanity stands on `G.kit.cabriole` legs, its oval mirror in a swept frame; the
# arbour's arch and the swing's arms are swept in one piece, its lattice too.
#
# Run:  cargo art pack_romantic [--only heartTub] [--preview] [--lit]
import math
import os
import sys

import bpy
from mathutils import Matrix, Vector

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gamekit as G  # noqa: E402
from plants import leaf  # noqa: E402  (leaf blades: rose leaves, petals, the quill's vane)

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
    flame=('#F28C28', 'gloss', {'glow': '#FFB050'}),        # candle flames: they glow at night
    flame_core=('#FFC24A', 'gloss', {'glow': '#FFD27A'}),
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


def _sq(w, h=None):
    """A square bar's profile, `w` across (`h` the other way)."""
    h = w if h is None else h
    return [(-w / 2, -h / 2), (w / 2, -h / 2), (w / 2, h / 2), (-w / 2, h / 2)]


def _candle(F, name, x, z, y, h=0.22, holder=None):
    """A candlestick standing on `y`, its candle a little melted at the top; returns the wick's top (x, height, z)."""
    G.revolve(f'{name}_stick', [(0, 0), (0.038, 0), (0.036, 0.01), (0.016, 0.022), (0.011, 0.04), (0.015, 0.05),
                                (0.024, 0.056), (0.024, 0.062), (0, 0.062)], at=F.at(x, y, z), material=holder or M.gold,
              segments=12)
    G.revolve(name, [(0, 0), (0.014, 0), (0.014, h - 0.006), (0.011, h), (0.006, h - 0.004), (0, h - 0.003)],
              at=F.at(x, y + 0.06, z), material=M.wax, segments=10)
    F.cyl(f'{name}_wick', 0.0015, 0.012, (x, y + 0.06 + h + 0.003, z), M.ink, verts=4)
    return (x, y + 0.06 + h, z)


def _flame(F, name, x, y, z, s=1.0):
    """A candle flame: a teardrop turned round, a brighter core inside it."""
    G.revolve(name, [(0, 0), (0.008 * s, 0.006 * s), (0.011 * s, 0.016 * s), (0.008 * s, 0.03 * s), (0, 0.048 * s)],
              at=F.at(x, y, z), material=M.flame, segments=8)
    G.revolve(f'{name}_core', [(0, 0.002), (0.006 * s, 0.008 * s), (0.007 * s, 0.016 * s), (0, 0.03 * s)],
              at=F.at(x, y - 0.001, z), material=M.flame_core, segments=6)


def _rose(F, name, x, y, z, s=0.05, material=None, segments=7, tilt=0.0, yaw=0.0):
    """A bloom `s` round, its foot at (x, y, z): a tight bud in two cups of petals, each a thick turned cup (seen into
    from above, so it has an inside), the outer one flaring; turned `tilt` towards `yaw`."""
    m = material or M.rose
    made = [G.revolve(f'{name}_outer', [(0, 0), (s * 0.55, s * 0.1), (s * 0.95, s * 0.45), (s * 1.0, s * 0.62),
                                       (s * 0.86, s * 0.58), (s * 0.5, s * 0.3), (0, s * 0.22)], material=m, segments=segments),
            G.revolve(f'{name}_inner', [(0, s * 0.18), (s * 0.5, s * 0.3), (s * 0.66, s * 0.7), (s * 0.56, s * 0.74),
                                       (s * 0.3, s * 0.52), (0, s * 0.5)], material=m, segments=segments - 1),
            G.revolve(f'{name}_bud', [(0, s * 0.45), (s * 0.3, s * 0.55), (s * 0.28, s * 0.8), (0, s * 0.86)], material=m,
                      segments=5)]
    turn = (Matrix.Translation(F.at(x, y, z)) @ Matrix.Rotation(math.radians(-yaw), 4, 'Z') @
            Matrix.Rotation(math.radians(-tilt), 4, 'X') @ Matrix.Rotation(math.radians(len(name) * 23), 4, 'Z'))
    for o in made:
        o.data.transform(turn)
    return made


def _rose_leaves(F, name, x, y, z, n=3, size=0.05, yaw0=0.0, material=None):
    """A few rose leaves round a bloom's foot."""
    for k in range(n):
        leaf(f'{name}{k}', F.at(x, y, z), yaw0 + k * 360 / n, -10, size, size * 0.6, material or M.leaf, droop=0.25,
             rows=3, ring=3)


def _wine_glass(F, name, x, y, z, wine=True):
    G.revolve(name, [(0, 0), (0.03, 0), (0.03, 0.005), (0.004, 0.01), (0.004, 0.08), (0.032, 0.11), (0.035, 0.17),
                     (0.031, 0.17), (0, 0.09)], at=F.at(x, y, z), material=M.glassy, segments=10)
    if wine:
        G.revolve(f'{name}_wine', [(0, 0.088), (0.022, 0.1), (0.031, 0.125), (0, 0.125)], at=F.at(x, y, z), material=M.rosed,
                  segments=10)


def _velvet_chair(F, name, x, z=0.0, frame=None, seat=None, facing='front'):
    """A slipper chair centred on (x, z): a stuffed seat (top 0.45) and a tall rounded back in velvet, on tapered
    legs, facing the object's `facing` ('front' or 'back': its back on the far side)."""
    frame, seat = frame or M.body, seat or M.fabric
    s = 1 if facing == 'front' else -1
    F.soft(f'{name}_seat', 0.44, 0.1, 0.42, (x, 0.392, z), seat, round=0.04, puff=0.012, flat=2)
    F.extrude(f'{name}_back', G.smooth_outline([(-0.21, 0.0), (0.21, 0.0), (0.21, 0.36), (0.12, 0.46), (-0.12, 0.46),
                                                (-0.21, 0.36)], passes=2), 0.08, (x, 0.48, z - s * 0.177), seat,
              rot=(-8 * s, 0, 0), bevel=0.03, bevel_segments=2)
    for dx in (-0.18, 0.18):
        for dz in (0.16, -0.17):
            F.sweep(f'{name}_leg{dx}{dz}', [(x + dx, 0.36, z + s * dz), (x + dx * 1.08, 0.0, z + s * dz * 1.1)], 0.015, frame,
                    verts=8, scale=[1.0, 0.6])


# --- the candlelit bistro table (2x1): a diner at each end (x = +-0.62, z = 0.1) facing in across the top -------
TZ = 0.1                                                   # the table's and the diners' depth (content `places`)
CANDLES = ((0, TZ - 0.2, 0.26), (0, TZ + 0.2, 0.22))       # (x, depth, candle height): either side of their sightline


def _end_chair(F, x, z):
    """A slipper chair at the table's end, seat centred on (x, z), its sitter facing in (+x from the left end, -x from
    the right): built facing the front, then turned a quarter about its seat's middle."""
    with G.new_parts() as parts:
        _velvet_chair(F, f'chair{x}', 0.0, 0.0)
    bpy.context.view_layer.update()                     # the last part's matrix is set only on an update
    turn = Matrix.Translation(F.at(x, 0, z)) @ Matrix.Rotation(math.radians(-90 if x < 0 else 90), 4, 'Z')
    for o in parts:
        o.matrix_world = turn @ o.matrix_world


def _tablecloth(F, top, hw, hd, drop):
    """The cloth over the 0.8 x 0.72 top, lofted in one piece: its flat top, the edge rolling over, a skirt falling
    `drop` in soft folds that flare at the hem, then up its inside to under the table top (hidden)."""
    bx, by, bz = F.at(0, top, TZ)
    n = 48

    def ring(rx, ry, y, fold=0.0, sq=9):
        pts = G.ring(rx, ry, bz + y, n, square=sq, at=(bx, by))
        if fold:
            pts = [(bx + (px - bx) * (1 + fold * math.sin(10 * math.atan2(py - by, px - bx))), by + (py - by) *
                    (1 + fold * math.sin(10 * math.atan2(py - by, px - bx))), pz) for px, py, pz in pts]
        return pts
    G.loft('cloth', [ring(0.0, 0.0, 0.0), ring(hw - 0.01, hd - 0.01, 0.0), ring(hw + 0.004, hd + 0.004, -0.006),
                     ring(hw + 0.006, hd + 0.006, -0.03, 0.006), ring(hw + 0.009, hd + 0.009, -drop * 0.6, 0.014),
                     ring(hw + 0.014, hd + 0.014, -drop, 0.02), ring(hw + 0.004, hd + 0.004, -drop, 0.02),
                     ring(hw, hd, -0.03)], material=M.cloth)


def candlelitTable():
    F = G.GameFrame()
    top = 0.775
    with G.new_parts() as still:
        # a 0.8 x 0.72 top under a cloth that drops 12 cm all round, clear of the diners' knees
        F.box('table_top', 0.8, 0.03, 0.72, (0, 0.745, TZ), M.oak)
        _tablecloth(F, 0.772, 0.41, 0.38, 0.12)
        F.sweep('runner', G.kit.fillet([(0, 0.67, TZ - 0.396), (0, 0.775, TZ - 0.393), (0, 0.775, TZ + 0.393),
                                        (0, 0.67, TZ + 0.396)], 0.012), None, M.runner, profile=_sq(0.003, 0.12), up=(0, 0, 1))
        for x in (-0.33, 0.33):
            for dz in (-0.27, 0.27):
                F.sweep(f'table_leg{x}{dz}', [(x, 0.73, TZ + dz), (x, 0.0, TZ + dz)], 0.02, M.body, verts=10, scale=[1.0, 0.65])
        for s in (-1, 1):
            x = s * 0.22
            G.revolve(f'plate{s}', [(0, 0), (0.08, 0), (0.09, 0.006), (0.128, 0.014), (0.13, 0.016), (0.124, 0.016),
                                    (0.085, 0.01), (0, 0.009)], at=F.at(x, top, TZ), material=M.porcelain, segments=22)
            G.torus(f'plate_rim{s}', 0.126, 0.0025, at=F.at(x, top + 0.016, TZ), material=M.gold, major_segments=22,
                    minor_segments=3)
            G.revolve(f'dessert{s}', [(0, 0), (0.05, 0), (0.05, 0.035), (0.045, 0.045), (0.02, 0.052), (0, 0.053)],
                      at=F.at(x, top + 0.009, TZ), material=M.dessert, segments=12)
            G.revolve(f'cream{s}', [(0, 0), (0.03, 0), (0.026, 0.01), (0.012, 0.02), (0, 0.03)], at=F.at(x, top + 0.06, TZ),
                      material=M.wax, segments=8)
            F.sphere(f'berry{s}', 0.012, (x, top + 0.092, TZ), M.rose, segments=8, rings=5)
            for k, dz in enumerate((-0.16, 0.16)):          # a fork and a knife either side of the plate
                F.sweep(f'cutlery{s}{k}', [(x - 0.08, top + 0.004, TZ + dz), (x + 0.08, top + 0.004, TZ + dz)], None, M.gold,
                        profile=_sq(0.003, 0.014 if k else 0.018), up=(0, 0, 1), scale=[1.0, 0.7] if k else [1.0, 1.3])
            _wine_glass(F, f'wine_glass{s}', x - s * 0.1, top, TZ - s * 0.22)          # at the diner's right hand
            _end_chair(F, s * 0.62, TZ)
        for k, (x, z, h) in enumerate(CANDLES):
            _candle(F, f'candle{k}', x, z, top, h)
        G.revolve('bud_vase', [(0, 0), (0.025, 0), (0.03, 0.05), (0.012, 0.13), (0.015, 0.18), (0.011, 0.18), (0, 0.17)],
                  at=F.at(0, top, TZ), material=M.glassy, segments=12)
        F.sweep('stem', [(0, 0.86, TZ), (0.006, 0.96, TZ), (0.01, 1.04, TZ)], 0.004, M.leaf, verts=5, resolution=2)
        _rose(F, 'vase_rose', 0.01, 1.03, TZ, 0.045)
        _rose_leaves(F, 'vase_leaf', 0.005, 0.98, TZ, n=2, size=0.045)
        for k, (x, z) in enumerate(((-0.08, TZ + 0.3), (0.1, TZ - 0.32), (-0.3, TZ - 0.25), (0.28, TZ + 0.27), (0.05, TZ + 0.28))):
            leaf(f'petal{k}', F.at(x, top + 0.004, z), k * 70, 4, 0.04, 0.035, M.rose, shape='petal', fold=0.4, rows=2, ring=3)
    G.join(still, 'body', origin=(0, 0, 0))
    # The two flames, joined with their origin on the shorter wick and built full size, then flattened:
    # `candles` grows them while the two dine; they glow at night (`flame`'s `glow`).
    with G.new_parts() as flames:
        for k, (x, z, h) in enumerate(CANDLES):
            _flame(F, f'flame{k}', x, top + 0.06 + h + 0.002, z, 1.1)
    low = min(h for _, _, h in CANDLES)
    flames = G.join(flames, 'flames', origin=F.at(0, top + 0.05 + low, TZ))   # flattened inside the wax at rest
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


def _heart_ring(F, k, y, n=72):
    """The heart outline scaled by `k` at height `y`, as a loft's ring (Blender points)."""
    return [F.at(x, y, z) for x, z in _heart(k, n)]


RIM, FLOOR, WATER = 0.54, 0.2, 0.47


def heartTub():
    F = G.GameFrame()
    with G.new_parts() as still:
        # the plinth, its edge eased; the shell in one loft: up the outside, over a rolled rim, down to the floor
        G.loft('plinth', [_heart_ring(F, 1.03, 0.0), _heart_ring(F, 1.04, 0.01), _heart_ring(F, 1.04, 0.07),
                          _heart_ring(F, 1.025, 0.08)], material=M.ivory)
        G.loft('shell', [_heart_ring(F, 0.95, 0.075), _heart_ring(F, 0.96, 0.1), _heart_ring(F, 0.96, RIM - 0.03),
                         _heart_ring(F, 0.955, RIM - 0.008), _heart_ring(F, 0.945, RIM), _heart_ring(F, 0.86, RIM),
                         _heart_ring(F, 0.845, RIM - 0.012), _heart_ring(F, 0.84, RIM - 0.04), _heart_ring(F, 0.835, FLOOR + 0.04),
                         _heart_ring(F, 0.815, FLOOR + 0.006), _heart_ring(F, 0.78, FLOOR), _heart_ring(F, 0.0, FLOOR)],
               material=M.tub)
        # an ice bucket with a bottle on the rim, rolled towels, candles and roses on the floor
        G.revolve('ice_bucket', [(0, 0), (0.045, 0), (0.06, 0.13), (0.064, 0.135), (0.064, 0.142), (0.056, 0.142),
                                 (0.052, 0.13), (0.04, 0.012), (0, 0.012)], at=F.at(0.6, RIM, 0.3), material=M.chrome,
                  segments=16)
        for s in (-1, 1):
            G.torus(f'bucket_ring{s}', 0.012, 0.003, at=F.at(0.6 + s * 0.062, RIM + 0.11, 0.3), material=M.chrome,
                    rot=(90, 0, 90), major_segments=8, minor_segments=3)
        bx, by, bz = F.at(0.6, RIM + 0.012, 0.3)
        bottle = G.loft('bottle', [G.ring(0.03 * k, 0.03 * k, bz + h, 10, at=(bx, by)) for h, k in
                                   ((0.0, 0.95), (0.01, 1.0), (0.15, 1.0), (0.19, 0.42), (0.23, 0.38), (0.24, 0.0))],
                        material=M.bottle)
        bottle.data.transform(Matrix.Translation((bx, by, bz)) @ Matrix.Rotation(math.radians(15), 4, 'Y') @
                              Matrix.Translation((-bx, -by, -bz)))
        G.loft('bottle_foil', [G.ring(0.0135, 0.0135, bz + h, 10, at=(bx, by)) for h in (0.19, 0.236)], material=M.gold
               ).data.transform(Matrix.Translation((bx, by, bz)) @ Matrix.Rotation(math.radians(15), 4, 'Y') @
                                Matrix.Translation((-bx, -by, -bz)))
        for k, (x, y, z, m) in enumerate(((-0.78, 0.045, 0.74, M.cloth), (-0.78, 0.045, 0.83, M.fabric), (-0.78, 0.13, 0.785, M.cloth))):
            F.sweep(f'towel{k}', [(x - 0.17, y, z), (x + 0.16, y, z)], 0.045, m, verts=10)    # rolled towels, stacked
            G.revolve(f'towel_end{k}', [(0, 0), (0.042, 0), (0.044, 0.004), (0.03, 0.006), (0, 0.004)],
                      at=F.at(x + 0.16, y, z), material=m, segments=10, rot=(0, -90, 0))
        for k, (x, z, h) in enumerate(((-0.85, -0.82, 0.3), (-0.72, -0.86, 0.2), (0.85, -0.82, 0.26), (0.73, -0.87, 0.18))):
            wx, wy, wz = _candle(F, f'floor_candle{k}', x, z, 0.0, h, holder=M.rosegold)
            G.revolve(f'floor_flame{k}', [(0, 0), (0.008, 0.006), (0.01, 0.016), (0, 0.04)], at=F.at(wx, wy + 0.003, wz),
                      material=M.flame_core, segments=6)
        G.revolve('floor_vase', [(0, 0), (0.06, 0), (0.08, 0.15), (0.05, 0.27), (0.055, 0.3), (0.05, 0.3), (0, 0.29)],
                  at=F.at(0.7, 0, 0.82), material=M.ivory, segments=16)
        for x, z, m in ((0.68, 0.84, M.rose), (0.74, 0.8, M.rose_pink)):
            F.sweep(f'vase_stem{x}', [(0.7, 0.25, 0.82), (x, 0.42, z)], 0.005, M.leaf, verts=5)
            _rose(F, f'vase_rose{x}', x, 0.41, z, 0.05, m)
            _rose_leaves(F, f'vase_leaf{x}', 0.5 * (0.7 + x), 0.36, 0.5 * (0.82 + z), n=2, size=0.05)
    G.join(still, 'body', origin=(0, 0, 0))
    # The water with its petals and bubbles, built full and flattened onto the tub's floor: `fill` raises it.
    with G.new_parts() as water:
        G.loft('water', [_heart_ring(F, 0.845, FLOOR - 0.01, 48), _heart_ring(F, 0.845, WATER, 48)], material=M.water)
        for i in range(26):
            a = i * 2.39996
            rr = 0.1 + 0.5 * math.sqrt((i + 0.5) / 26)
            x, z = rr * math.cos(a) * 0.95, -0.05 + rr * math.sin(a) * 0.72
            leaf(f'petal{i}', F.at(x, WATER + 0.002, z), i * 37, 6, 0.05, 0.045, M.rose if i % 3 else M.rose_pink,
                 shape='petal', fold=0.45, rows=2, ring=3)
        for x, z in ((-0.18, -0.35), (0.25, -0.42), (0.05, 0.3), (-0.4, 0.05)):
            for k in range(4):                                     # foam: a little heap of bubbles
                F.sphere(f'bubble{x}{z}{k}', 0.04 - 0.008 * k, (x + 0.03 * math.cos(k * 2.1), WATER + 0.004, z + 0.03 * math.sin(k * 2.1)),
                         M.bubble, scale=(1, 1, 0.6), segments=8, rings=4)
    water = G.join(water, 'water', origin=F.at(0, FLOOR - 0.01, 0))   # hidden in the floor at rest
    water.scale[2] = 0.001
    objs = G.settle(centre=False)
    G.grow(water, 'fill', seconds=1.2)
    return objs


# --- the rose-gold vanity (1x1): a dressing table with an oval mirror, a pouf before it ---------------------
def vanity():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.extrude('vanity_top', G.rounded_rect(0.8, 0.36, 0.06, corner=4), 0.05, (0, 0.735, -0.25), M.ivory, plane='top',
                  bevel=0.012, bevel_segments=2)
        # the apron, its lower edge scalloped either side of the drawer; the drawer's opening
        edge = [(-0.38 + 0.76 * i / 20, 0.0) for i in range(21)]
        edge = [(x, 0.012 * abs(math.sin(math.pi * x / 0.19))) for x, _ in edge]
        F.extrude('apron', [(0.38, 0.14), (-0.38, 0.14)] + edge, 0.3, (0, 0.57, -0.25), M.ivory, plane='front')
        F.box('drawer_hole', 0.36, 0.11, 0.01, (0, 0.635, -0.098), M.blushd)
        for x in (-0.34, 0.34):
            for z in (-0.38, -0.12):
                G.kit.cabriole(F, f'leg{x}{z}', x, z, 0.57, M.rosegold, 0.022 * (1 if x > 0 else -1), 0.02 * (1 if z > -0.25 else -1),
                               r=0.018)
        # the oval mirror in a moulded rose-gold frame, on a stand rising from the top's back
        oval = [(0.25 * math.cos(math.tau * k / 36), 1.24 + 0.36 * math.sin(math.tau * k / 36)) for k in range(36)]
        F.sweep('mirror_frame', [(x, y, -0.405) for x, y in oval], None, M.rosegold, closed=True,
                profile=[(-0.012, -0.022), (0.012, -0.022), (0.016, 0.0), (0.012, 0.022), (-0.012, 0.022)], up=(0, 1, 0))
        F.cyl('mirror_back', 0.25, 0.012, (0, 1.24, -0.418), M.rosegold, rot=(90, 0, 0), scale=(1, 1.44, 1), verts=36)
        F.cyl('mirror', 0.235, 0.004, (0, 1.24, -0.398), M.mirror, rot=(90, 0, 0), scale=(1, 1.44, 1), verts=36)
        F.sweep('mirror_stand', [(0, 0.76, -0.41), (0, 0.84, -0.412), (0, 0.9, -0.41)], None, M.rosegold,
                profile=G.rounded_rect(0.012, 0.05, 0.005, corner=1), up=(0, 0, 1), scale=[1.3, 1.0, 1.1])
        for k, (x, h, m) in enumerate(((-0.3, 0.12, M.perfume_a), (-0.22, 0.09, M.perfume_b), (-0.15, 0.14, M.perfume_c))):
            bx, by, bz = F.at(x, 0.76, -0.22)
            G.loft(f'perfume{k}', [G.ring(0.03 * kk, 0.022 * kk, bz + h * t, 8, square=sq, at=(bx, by)) for t, kk, sq in
                                   ((0.0, 0.9, 4), (0.04, 1.0, 4), (0.7, 1.0, 4), (0.82, 0.4, 2), (1.0, 0.36, 2))],
                   material=m, smooth=False)
            G.revolve(f'stopper{k}', [(0, 0), (0.008, 0), (0.008, 0.008), (0.018, 0.02), (0.012, 0.036), (0, 0.04)],
                      at=F.at(x, 0.76 + h, -0.22), material=M.rosegold, segments=8)
        F.soft('jewellery_box', 0.16, 0.08, 0.12, (0.25, 0.8, -0.22), M.blushd, round=0.012, round_segments=1)
        F.soft('jewellery_lid', 0.17, 0.016, 0.13, (0.25, 0.846, -0.22), M.rosegold, round=0.006, round_segments=1)
        F.sweep('necklace', [(0.2, 0.86, -0.18), (0.24, 0.855, -0.155), (0.28, 0.86, -0.16), (0.3, 0.84, -0.16), (0.31, 0.77, -0.15)],
                0.002, M.gold, verts=3, resolution=2)
        G.revolve('vase', [(0, 0), (0.03, 0), (0.04, 0.07), (0.025, 0.12), (0.03, 0.14), (0.026, 0.14), (0, 0.13)],
                  at=F.at(0.1, 0.76, -0.36), material=M.ivory, segments=12)
        for k, (x, y, z, m) in enumerate(((0.07, 0.92, -0.34, M.rose), (0.13, 0.93, -0.37, M.rose_pink), (0.1, 0.95, -0.33, M.ivory))):
            F.sweep(f'vase_stem{k}', [(0.1, 0.88, -0.36), (x, y, z)], 0.003, M.leaf, verts=4)
            _rose(F, f'vase_rose{k}', x, y - 0.005, z, 0.035, m, tilt=15, yaw=k * 120)
        _rose_leaves(F, 'vase_leaf', 0.1, 0.9, -0.36, n=3, size=0.045)
        # the pouf before it: buttoned, on a rose-gold ring with four little feet
        G.revolve('pouf', [(0, 0.03), (0.19, 0.03), (0.21, 0.07), (0.212, 0.34), (0.2, 0.42), (0.16, 0.44), (0.06, 0.43),
                           (0, 0.425)], at=F.at(0, 0.0, 0.17), material=M.velvet, segments=24)
        G.torus('pouf_ring', 0.212, 0.01, at=F.at(0, 0.04, 0.17), material=M.rosegold, major_segments=24, minor_segments=5)
        for a in range(45, 360, 90):
            F.sweep(f'pouf_foot{a}', [(0.17 * G.cosd(a), 0.04, 0.17 + 0.17 * G.sind(a)), (0.19 * G.cosd(a), 0.004, 0.17 + 0.19 * G.sind(a))],
                    0.009, M.rosegold, verts=6, scale=[1.0, 0.7])
        for a in range(0, 360, 60):
            F.sphere(f'tuft{a}', 0.012, (0.11 * G.cosd(a), 0.432, 0.17 + 0.11 * G.sind(a)), M.blushd, scale=(1, 1, 0.6),
                     segments=6, rings=4)
        F.sphere('tuft_mid', 0.013, (0, 0.428, 0.17), M.blushd, scale=(1, 1, 0.6), segments=6, rings=4)
    G.join(still, 'body', origin=(0, 0, 0))
    # The middle drawer, pulled out to the front by `drawer_open`.
    with G.new_parts() as drawer:
        F.box('drawer_front', 0.34, 0.1, 0.014, (0, 0.635, -0.091), M.ivory, bevel=0.004, bevel_segments=1)
        F.box('drawer_box', 0.32, 0.08, 0.26, (0, 0.635, -0.22), M.ivory)
        G.kit.knob(F, 'drawer_knob', 0, 0.635, -0.084, 0.012, M.rosegold, segments=10)
        F.sweep('lipstick', [(-0.08, 0.676, -0.18), (-0.08, 0.676, -0.12)], 0.009, M.rosegold, verts=8)
        F.sweep('lipstick_tip', [(-0.08, 0.676, -0.12), (-0.08, 0.676, -0.1)], 0.007, M.rose, verts=8, scale=[1.0, 0.6])
        G.revolve('compact', [(0, 0), (0.035, 0), (0.036, 0.006), (0.034, 0.012), (0, 0.013)], at=F.at(0.06, 0.676, -0.2),
                  material=M.rosegold, segments=14)
    drawer = G.hinged(G.join(drawer, 'drawer'), F.at(0, 0.635, -0.084))
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
        F.box('cabinet', 0.62, 0.66, 0.4, (0, 0.38, WALL + 0.21), M.cabinet, bevel=0.008, bevel_segments=1)
        G.kit.moulding(F, 'cabinet_top', [(-0.33, WALL), (0.33, WALL), (0.33, WALL + 0.44), (-0.33, WALL + 0.44)],
                       [(0.0, 0.0), (0.02, 0.0), (0.02, 0.03), (0.006, 0.03), (0.0, 0.02)], M.walnut, at=0.71, plane='top')
        F.box('cabinet_top_board', 0.62, 0.03, 0.4, (0, 0.725, WALL + 0.22), M.walnut)
        for s in (-1, 1):
            G.kit.shaker(F, f'cabinet_door{s}', 0.28, 0.5, (s * 0.15, 0.4, WALL + 0.416), 0.012, M.door, rail=0.045,
                         panel=M.cabinet, raised=True)
            G.kit.knob(F, f'door_knob{s}', s * 0.03, 0.42, WALL + 0.422, 0.011, M.gold, segments=8)
        for x in (-0.27, 0.27):
            for z in (WALL + 0.06, WALL + 0.36):
                G.revolve(f'foot{x}{z}', [(0, 0), (0.02, 0), (0.03, 0.03), (0.025, 0.05), (0, 0.05)],
                          at=F.at(x, 0, z), material=M.gold, segments=10)
        # the gramophone: a walnut case with brass corners and band, its crank; the horn on a swept neck from the back
        F.box('case', 0.38, 0.13, 0.36, (0, 0.805, -0.18), M.walnut, bevel=0.01, bevel_segments=2)
        F.box('case_band', 0.385, 0.02, 0.365, (0, 0.82, -0.18), M.gold)
        for x in (-0.19, 0.19):
            for z in (-0.36, 0.0):
                F.box(f'case_corner{x}{z}', 0.022, 0.13, 0.022, (x, 0.805, z), M.gold, bevel=0.004, bevel_segments=1)
        G.revolve('crank_boss', [(0, 0), (0.024, 0), (0.024, 0.01), (0.014, 0.02), (0, 0.02)], at=F.at(-0.19, 0.8, -0.18),
                  material=M.gold, segments=10, rot=(0, 90, 0))
        F.sweep('crank', G.kit.fillet([(-0.205, 0.8, -0.18), (-0.24, 0.8, -0.18), (-0.24, 0.74, -0.15), (-0.27, 0.74, -0.15)], 0.012),
                0.005, M.gold, verts=5)
        F.sweep('crank_handle', [(-0.27, 0.74, -0.15), (-0.31, 0.74, -0.15)], 0.009, M.walnut, verts=6)
        G.revolve('arm_base', [(0, 0), (0.032, 0), (0.03, 0.012), (0.018, 0.02), (0.016, 0.03), (0, 0.03)],
                  at=F.at(ARM[0], 0.87, ARM[2]), material=M.gold, segments=12)
        F.sweep('horn_neck', [(0.0, 0.85, -0.3), (0.0, 0.93, -0.36), (0.0, 1.01, -0.35), (0.0, 1.06, -0.32)], 0.022, M.horn,
                verts=10, resolution=3, scale=[1.0, 0.9, 0.8, 0.75])
        horn = [(0.02, 0), (0.03, 0.08), (0.06, 0.2), (0.12, 0.32), (0.23, 0.42), (0.24, 0.43), (0.225, 0.43), (0.115, 0.335),
                (0.055, 0.215), (0.025, 0.09), (0.012, 0.0)]
        G.revolve('horn', horn, at=F.at(0.0, 1.05, -0.32), material=M.horn, segments=28, rot=(34, 0, 0))
        hx, hy, hz = F.at(0.0, 1.05, -0.32)
        tilt = Matrix.Translation((hx, hy, hz)) @ Matrix.Rotation(math.radians(34), 4, 'X')
        for k in range(8):                                         # the seams between the horn's petals
            a = math.tau * (k + 0.5) / 8
            pts = [tuple(tilt @ Vector((r * math.cos(a) * 1.01, r * math.sin(a) * 1.01, h))) for r, h in horn[:6]]
            G.sweep(f'horn_seam{k}', pts, 0.0035, material=M.gold, verts=4)
        # record sleeves leant on the cabinet and a rose on its top
        F.box('sleeve_a', 0.3, 0.3, 0.012, (0.22, 0.15, 0.2), M.sleeve_a, rot=(-12, 20, 0))
        F.box('sleeve_b', 0.3, 0.3, 0.012, (0.2, 0.15, 0.22), M.sleeve_b, rot=(-18, 14, 0))
        F.cyl('sleeve_disc', 0.07, 0.004, (0.22, 0.155, 0.208), M.paper, rot=(-12 + 90, 20, 0), verts=14)
        F.sweep('rose_stem', [(-0.22, 0.745, -0.06), (-0.16, 0.746, -0.03), (-0.1, 0.745, 0.0)], 0.004, M.leaf, verts=5,
                resolution=2)
        _rose(F, 'cabinet_rose', -0.24, 0.742, -0.07, 0.05, tilt=70, yaw=240)
        _rose_leaves(F, 'rose_leaf', -0.15, 0.748, -0.025, n=2, size=0.04)
    G.join(still, 'body', origin=(0, 0, 0))
    # The platter and record, origin on the spindle: `platter` turns them while the record plays.
    with G.new_parts() as platter:
        F.cyl('platter', 0.15, 0.012, (rx, ry - 0.008, rz), M.gold, verts=32)
        grooves = [(0, 0), (0.145, 0)] + [(0.145 - 0.012 * k - (0.006 if j else 0), 0.006 if j else 0.0075)
                                          for k in range(7) for j in (0, 1)] + [(0.05, 0.007), (0, 0.007)]
        G.revolve('record', grooves, at=F.at(rx, ry - 0.002, rz), material=M.record, segments=32)
        F.cyl('label', 0.05, 0.009, (rx, ry + 0.003, rz), M.rose, verts=20)
        F.box('label_mark', 0.04, 0.0095, 0.008, (rx + 0.02, ry + 0.003, rz), M.paper)
        F.cyl('spindle', 0.005, 0.02, (rx, ry + 0.01, rz), M.gold, verts=6)
    platter = G.hinged(G.join(platter, 'platter'), F.at(rx, ry, rz))
    # The tone arm, resting off the record beside it: `needle` swings it onto the record about its pivot.
    ax, ay, az = ARM
    with G.new_parts() as arm:
        G.revolve('arm_pivot', [(0, 0), (0.02, 0), (0.02, 0.03), (0.014, 0.04), (0, 0.042)], at=F.at(ax, ay - 0.012, az),
                  material=M.gold, segments=10)
        F.sweep('arm_tube', [(ax, ay + 0.02, az), (ax + 0.012, ay + 0.03, az + 0.1), (ax + 0.02, ay + 0.03, az + 0.2)], 0.008,
                M.gold, verts=8, resolution=3, scale=[1.0, 0.85, 0.7])
        G.revolve('sound_box', [(0, 0), (0.03, 0), (0.032, 0.006), (0.032, 0.014), (0.03, 0.018), (0, 0.018)],
                  at=F.at(ax + 0.02, ay + 0.025, az + 0.212), material=M.gold, segments=14, rot=(90, 0, 0))
        F.cyl('sound_face', 0.024, 0.004, (ax + 0.02, ay + 0.025, az + 0.231), M.paper, rot=(90, 0, 0), verts=12)
        G.rod('stylus', F.at(ax + 0.02, ay + 0.02, az + 0.22), F.at(ax + 0.02, ry + 0.006, az + 0.22), 0.002,
              material=M.chrome, verts=4)
    arm = G.hinged(G.join(arm, 'tone_arm'), F.at(ax, ay, az))
    objs = G.settle(centre=False)
    G.swing(arm, 'needle', 30, seconds=0.8)
    G.spin(platter, 'platter', turns=1, seconds=1.8)
    return objs


# --- the love-letter bureau (1x1): its back on the wall, the desk and its pigeonholes there, the writer's chair in front ---
DESK_D = 0.45     # the desk's depth from the wall face
CHAIR_Z = 0.2     # the chair's middle depth (the hips at 0.25: `furniture.json`)


def letterBureau():
    F = G.GameFrame()
    # Its back on the wall, the pigeonhole hutch on the desk's back facing the writer and the room, the chair in front,
    # the writer facing the wall (`furniture.json` `back`; 2026-10-21: it stood the other way round, the hutch's
    # panelled back and the writer's back to the room).
    hz = WALL + 0.012 + 0.05          # the hutch's middle depth (0.1 deep)
    hf = hz + 0.05                    # its front
    with G.new_parts() as still:
        F.box('desk_top', 0.84, 0.03, DESK_D, (0, 0.745, WALL + DESK_D / 2), M.ivory, bevel=0.01, bevel_segments=2)
        edge = [(-0.4 + 0.8 * i / 16, 0.0) for i in range(17)]
        edge = [(x, 0.02 + 0.018 * math.cos(math.pi * x / 0.4) ** 2) for x, _ in edge]
        F.extrude('apron', [(0.4, 0.1), (-0.4, 0.1)] + edge, DESK_D - 0.03, (0, 0.63, WALL + (DESK_D - 0.03) / 2), M.ivory)
        F.box('drawer', 0.3, 0.06, 0.012, (0, 0.69, WALL + DESK_D - 0.023), M.blush, bevel=0.004, bevel_segments=1)
        G.kit.knob(F, 'drawer_knob', 0, 0.69, WALL + DESK_D - 0.017, 0.01, M.gold, segments=8)
        for x in (-0.39, 0.39):
            for z in (WALL + 0.03, WALL + DESK_D - 0.04):
                F.sweep(f'leg{x}{z}', [(x, 0.73, z), (x, 0.08, z), (x, 0.0, z)], None, M.ivory,
                        profile=G.rounded_rect(0.034, 0.034, 0.008, corner=1), up=(0, 1, 0), scale=[1.0, 0.55, 0.62])
        # the pigeonhole hutch facing the writer, its back on the wall, an arched gold crest
        F.box('hutch_back', 0.8, 0.38, 0.02, (0, 0.95, WALL + 0.01), M.ivory, bevel=0.004, bevel_segments=1)
        for x in (-0.4, 0.4):
            F.box(f'hutch_side{x}', 0.02, 0.38, 0.1, (x, 0.95, hz), M.ivory, bevel=0.004, bevel_segments=1)
            F.box(f'hutch_panel{x}', 0.004, 0.28, 0.07, (x * 1.026, 0.95, hz), M.blush, bevel=0.002, bevel_segments=1)
        F.box('hutch_top', 0.84, 0.025, 0.11, (0, 1.15, hz), M.ivory, bevel=0.006, bevel_segments=1)
        crest = [(-0.42, 0.0), (0.42, 0.0)] + [(0.42 - 0.84 * i / 16, 0.012 + 0.012 * math.sin(math.pi * i / 16)) for i in range(17)]
        F.extrude('crest', crest[::-1], 0.11, (0, 1.1625, hz), M.gold)
        for x in (-0.2, 0.0, 0.2):
            F.box(f'divider{x}', 0.012, 0.36, 0.09, (x, 0.95, hz + 0.005), M.ivory)
        F.box('shelf', 0.78, 0.012, 0.09, (0, 0.97, hz + 0.005), M.ivory)
        for k, (x, y, m) in enumerate(((-0.3, 1.0, M.paper), (-0.1, 1.0, M.blush), (0.3, 1.0, M.envelope), (-0.3, 0.8, M.paper),
                                       (0.1, 0.8, M.envelope))):
            for j in range(3):                                      # envelopes standing in the pigeonhole, a ribbon round them
                F.box(f'letter{k}{j}', 0.13, 0.085 - 0.01 * j, 0.004, (x + 0.004 * j, y + 0.018, hz - 0.02 + 0.016 * j), m)
            F.box(f'letter_ribbon{k}', 0.01, 0.09, 0.05, (x, y + 0.02, hz - 0.004), M.rose)
        # on the desk either side of the flap: an inkpot and a quill, a wax seal, a candle
        G.revolve('inkpot', [(0, 0), (0.03, 0), (0.03, 0.035), (0.012, 0.045), (0.012, 0.06), (0.015, 0.062), (0, 0.062)],
                  at=F.at(0.33, 0.76, -0.2), material=M.ink, segments=12)
        F.sweep('quill', [(0.33, 0.8, -0.2), (0.29, 0.9, -0.225), (0.25, 0.98, -0.25)], 0.003, M.paper, verts=4, resolution=2)
        leaf('quill_vane', F.at(0.31, 0.86, -0.212), 125, 62, 0.15, 0.04, M.paper, shape='lance', droop=0.15, fold=0.05,
             rows=5, ring=3)
        G.revolve('wax_seal', [(0, 0), (0.026, 0), (0.028, 0.006), (0.024, 0.012), (0, 0.011)], at=F.at(0.32, 0.76, -0.07),
                  material=M.rosed, segments=12)
        F.sweep('wax_stick', [(0.29, 0.766, -0.15), (0.31, 0.766, -0.06)], 0.006, M.rose, verts=6)
        wx, wy, wz = _candle(F, 'desk_candle', -0.33, -0.15, 0.76, 0.14)
        G.revolve('candle_flame', [(0, 0), (0.008, 0.006), (0.01, 0.016), (0, 0.04)], at=F.at(wx, wy + 0.003, wz),
                  material=M.flame_core, segments=6)
        _velvet_chair(F, 'chair', 0.0, CHAIR_Z, frame=M.ivory, facing='back')
    G.join(still, 'body', origin=(0, 0, 0))
    # The writing flap, closed up over the lower middle pigeonholes, hinged on its foot: `fall_front` drops it
    # towards the writer onto the desk, a letter on its inside.
    with G.new_parts() as flap:
        G.kit.shaker(F, 'flap', 0.5, 0.2, (0, 0.87, hf + 0.009), 0.018, M.ivory, rail=0.04, panel=M.blush, recess=0.004)
        G.kit.knob(F, 'flap_knob', 0, 0.95, hf + 0.018, 0.01, M.gold, segments=8)
        F.box('flap_letter', 0.2, 0.14, 0.003, (-0.08, 0.87, hf - 0.002), M.paper, rot=(0, 0, 6))
        F.box('flap_envelope', 0.14, 0.1, 0.003, (0.13, 0.86, hf - 0.002), M.envelope, rot=(0, 0, -8))
    flap = G.hinged(G.join(flap, 'flap'), F.at(0, 0.77, hf + 0.009))
    objs = G.settle(centre=False)
    G.swing(flap, 'fall_front', 90, seconds=0.8, axis='X')
    return objs


# --- the rose-arbour swing (2x1): a porch swing for two under an arch of climbing roses ---------------------
BEAM = 2.1
ARCH = [(-0.88 + 1.76 * t / 8, 2.12 + 0.3 * math.sin(math.pi * t / 8)) for t in range(9)]
BLOOM = (-0.84, 1.36, 0.35)   # the bloom that is cut


def _climber(F, name, x, z, y0, y1, k0):
    """Climbing roses up a post: a cane winding up it, leaves in threes along it, a bloom every so often."""
    n = 13
    F.sweep(f'{name}_cane', [(x + 0.03 * math.cos(t * 1.7), y0 + (y1 - y0) * t / 6 - 0.3 * (t == 0), z + 0.03 * math.sin(t * 1.7))
                             for t in range(7)], 0.006, M.leaf_dark, verts=4, resolution=2)
    for i in range(n):
        y = y0 + (y1 - y0) * i / (n - 1)
        a = i * 137.5 + k0 * 40
        for j in range(3):
            leaf(f'{name}_leaf{i}_{j}', F.at(x, y + 0.03 * j, z), a + 120 * j, 10 + 15 * ((i + j) % 2), 0.1 - 0.008 * j, 0.07,
                 M.leaf if (i + j + k0) % 2 else M.leaf_dark, droop=0.35, rows=3, ring=3)
        if i % 3 == 1:
            _rose(F, f'{name}_rose{i}', x + 0.07 * G.cosd(a), y + 0.02, z + 0.07 * G.sind(a), 0.045,
                  M.rose if (i + k0) % 2 else M.rose_pink, tilt=40, yaw=-a + 90)


def roseArbourSwing():
    F = G.GameFrame()
    with G.new_parts() as still:
        for x in (-0.88, 0.88):
            for z in (-0.36, 0.36):
                F.box(f'post{x}{z}', 0.07, 2.12, 0.07, (x, 1.06, z), M.frame, bevel=0.008, bevel_segments=1)
            # the lattice between the posts: diagonal laths each way, one sweep each
            for i in range(-3, 5):
                for sgn in (-1, 1):
                    za, zb = -0.33, 0.33
                    ya = 0.2 + i * 0.36
                    pts = [(x, ya, za * sgn), (x, ya + 0.66, zb * sgn)]
                    lo, hi = 0.2, 1.95                             # clip to the panel
                    (_, y0, z0), (_, y1, z1) = pts
                    t0, t1 = max(0.0, (lo - y0) / (y1 - y0)), min(1.0, (hi - y0) / (y1 - y0))
                    if t1 <= t0:
                        continue
                    a = (x, y0 + (y1 - y0) * t0, z0 + (z1 - z0) * t0)
                    b = (x, y0 + (y1 - y0) * t1, z0 + (z1 - z0) * t1)
                    F.sweep(f'lattice{x}{i}{sgn}', [a, b], None, M.frame, profile=_sq(0.01, 0.022), up=(1, 0, 0))
            for y in (0.2, 1.95):
                F.box(f'lattice_rail{x}{y}', 0.03, 0.04, 0.66, (x, y, 0), M.frame, bevel=0.006, bevel_segments=1)
        for z in (-0.36, 0.36):                                  # the arch, a curved rafter each side in one sweep
            F.sweep(f'arch{z}', [(x, y, z) for x, y in ARCH], None, M.frame, profile=_sq(0.05, 0.06), up=(0, 0, 1),
                    resolution=2)
        for x, y in ARCH[1:-1]:
            F.box(f'slat{x:.2f}', 0.04, 0.04, 0.82, (x, y + 0.03, 0), M.frame, bevel=0.006, bevel_segments=1)
        F.box('beam', 1.82, 0.08, 0.08, (0, BEAM, 0.0), M.frame, bevel=0.012, bevel_segments=1)
        for s in (-1, 1):
            F.sweep(f'beam_eye{s}', [(s * 0.78, BEAM - 0.04, 0.0), (s * 0.78, BEAM - 0.06, 0.0)], 0.01, M.gold, verts=6)
        # climbing roses up the posts and over the arch
        k = 0
        for x in (-0.82, 0.82):
            for z in (-0.3, 0.3):
                _climber(F, f'climber{x}{z}', x, z, 0.35, 1.9, k)
                k += 1
        for i, (x, y) in enumerate(ARCH[1:-1]):
            for z in (-0.27, -0.13, 0.0, 0.13, 0.27):
                leaf(f'arch_leaf{i}{z}', F.at(x, y + 0.06, z), i * 77 + z * 300, 15, 0.13, 0.09, M.leaf_dark if i % 2 else M.leaf,
                     droop=0.4, rows=3, ring=3)
                leaf(f'arch_leaf{i}{z}b', F.at(x, y + 0.06, z), i * 77 + z * 300 + 150, 20, 0.11, 0.08, M.leaf if i % 2 else M.leaf_dark,
                     droop=0.4, rows=3, ring=3)
            _rose(F, f'arch_rose{i}', x, y + 0.15, -0.25 + 0.17 * (i % 4), 0.05, M.rose if i % 2 else M.rose_pink)
    G.join(still, 'body', origin=(0, 0, 0))
    # The swing (seat top 0.45 over z = 0) with its arms, cushions and chains, hung from the beam: `swing` rocks
    # it, and the residents sitting on it ride it.
    with G.new_parts() as swing:
        for k in range(5):                                         # the seat's slats, a frame round them
            F.box(f'seat_slat{k}', 1.56, 0.03, 0.075, (0, 0.385, -0.17 + k * 0.08), M.frame, bevel=0.005, bevel_segments=1)
        for z in (-0.21, 0.19):
            F.box(f'seat_rail{z}', 1.6, 0.06, 0.03, (0, 0.37, z), M.frame, bevel=0.005, bevel_segments=1)
        F.soft('seat_cushion', 1.5, 0.05, 0.38, (0, 0.425, -0.005), M.cushion, round=0.02, puff=0.008, flat=2)
        for k in range(7):                                         # the back's spindles between two rails
            x = -0.6 + k * 0.2
            F.box(f'back_spindle{k}', 0.03, 0.38, 0.02, (x, 0.66, -0.215), M.frame, rot=(-10, 0, 0), bevel=0.004, bevel_segments=1)
        for y in (0.48, 0.86):
            F.box(f'back_rail{y}', 1.6, 0.04, 0.035, (0, y, -0.215 - (y - 0.66) * 0.18), M.frame, rot=(-10, 0, 0), bevel=0.006,
                  bevel_segments=1)
        F.soft('pillow_l', 0.36, 0.3, 0.1, (-0.6, 0.64, -0.15), M.rose, rot=(-12, 0, -15), round=0.04, puff=0.03, flat=2,
               round_segments=1)
        F.soft('pillow_r', 0.34, 0.28, 0.1, (0.62, 0.63, -0.15), M.ivory, rot=(-12, 0, 15), round=0.04, puff=0.03, flat=2,
               round_segments=1)
        for s in (-1, 1):
            F.sweep(f'arm{s}', [(s * 0.78, 0.86, -0.25), (s * 0.78, 0.63, -0.18), (s * 0.78, 0.62, 0.12), (s * 0.78, 0.6, 0.19)], None,
                    M.frame, profile=G.rounded_rect(0.05, 0.04, 0.012, corner=1), up=(1, 0, 0), resolution=2)
            F.box(f'arm_post{s}', 0.04, 0.2, 0.04, (s * 0.78, 0.5, 0.17), M.frame, bevel=0.005, bevel_segments=1)
            for zc, yc in ((0.17, 0.64), (-0.26, 0.86)):
                F.sweep(f'chain{s}{zc}', [(s * 0.78, yc, zc), (s * 0.78, BEAM - 0.06, 0.0)], 0.007, M.gold, verts=5)
    swing = G.hinged(G.join(swing, 'swing'), F.at(0, BEAM, 0.0))
    # The bloom a resident cuts, origin on its stem: `cut` shrinks it away.
    with G.new_parts() as bloom:
        _rose(F, 'bloom', BLOOM[0], BLOOM[1] - 0.03, BLOOM[2], 0.06, M.rose, tilt=35, yaw=90)
        leaf('bloom_leaf', F.at(BLOOM[0] + 0.01, BLOOM[1] - 0.05, BLOOM[2] - 0.02), 200, 10, 0.07, 0.045, M.leaf, droop=0.3,
             rows=3, ring=3)
    bloom = G.hinged(G.join(bloom, 'bloom'), F.at(BLOOM[0], BLOOM[1] - 0.06, BLOOM[2]))
    objs = G.settle(centre=False)
    G.rock(swing, 'swing', 7, seconds=2.0)
    G.squash(bloom, 'cut', 0.001, seconds=0.6)
    return objs


FAMILY = G.Family(
    'pack_romantic', M,
    [G.Model('candlelitTable', candlelitTable, size=(1.802, 1.096, 0.8), key='model.romantic.candlelitTable',
             centre=(0, 0.1), motions={'uses': {'candlelitDinner': 'candles', 'shareDessert': 'candles'}},
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
     G.Model('letterBureau', letterBureau, size=(0.84, 1.18, 0.911), key='model.romantic.letterBureau',
             centre=(0, 0.0255), motions={'uses': {'writeLetter': 'fall_front', 'writePoem': 'fall_front'}},
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

# The nature lover pack (P12): the raised veggie bed, the potting bench, the bird feeder post, the garden
# hammock, the cedar outdoor shower, the moss terrarium and the fern glasshouse.
# Notes: pack_natureLover.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`): a lettuce is picked from the veggie bed (`harvest`) whose crop follows its growth stage (`grow`), a
# seedling comes up in a pot (`seedling`), a bird lands on the feeder (`bird`), the hammock sways (`sway`, a
# loop the resident rides), the shower runs (`water_on`), the terrarium's glass lid lifts (`lid_open`) and the
# glasshouse's door opens to let the resident in and out (`door_open`) and its roof vent airs it (`vent_open`).
#
# Run:  cargo art pack_natureLover [--only veggieBed] [--preview]
import math
import os
import random
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    cedar=('#C27C4E', 'satin'),      # plain cedar: the veggie bed, the feeder, the glasshouse bench; the main part
    cedar_dark=('#A3653E', 'satin'),
    cedar_wood=('#C27C4E', 'wood'),  # textured cedar: the hammock's stand and the shower's slats
    cedar_wood_dark=('#A3653E', 'wood'),
    bench=('#D9B98C', 'satin'),      # the potting bench's worktop and shelves
    sage=('#8FA88A', 'satin'),       # the potting bench's frame, the feeder's roof trim
    walnut=('#8A5E3C', 'satin'),     # the terrarium's pedestal
    frame=('#F2F2F0', 'satin'),      # the glasshouse's white frame
    stone=('#C9C2B4', 'satin'),
    gravel=('#B8A88E', 'matte'),
    soil=('#5A4030', 'matte'),
    terra=('#C7744F', 'satin'),      # terracotta pots
    porcelain=('#F2F2F0', 'gloss'),
    canvas=('#EDE4D3', 'fabric'),    # the hammock's sling
    stripe=('#7FA87A', 'fabric'),
    rope=('#D9CDB4', 'fabric'),
    pillow=('#E8B07A', 'fabric'),
    cushion=('#E8B07A', 'satin'),    # the glasshouse bench's cushions
    copper=('#C27A4A', 'metal'),
    steel=('#8A9096', 'metal'),
    can=('#8EA5A0', 'metal'),
    dark=('#3A3D41', 'satin'),
    label=('#F2F2F0', 'matte'),
    towel=('#F2F2F0', 'satin'),
    soap=('#F2D6A0', 'satin'),
    sack=('#D8C49C', 'matte'),
    seed=('#C9A66B', 'matte'),
    house=('#E8E0CF', 'satin'),      # the feeder's little house
    roof=('#5E7F9A', 'satin'),
    tube=('#E6F0F2', 'gloss'),       # the seed tube, plain (no glass draw)
    pebble=('#B8B4AE', 'satin'),
    pebble_b=('#9EA0A6', 'satin'),
    rug=('#7FA87A', 'matte'),
    water=('#BFE3F2', 'gloss'),
    mister=('#8EC6D8', 'gloss'),
    glass=('#DCEBEA', 'glass'),
    # greens: foliage where it takes the model's textured draw, satin where the budget is spent elsewhere
    leaf=('#5E8F4E', 'foliage'),
    leaf_light=('#78A85A', 'foliage'),
    leaf_pale=('#9BCB5A', 'foliage'),
    leaf_dark=('#4F8A45', 'foliage'),
    moss=('#4F8A3C', 'foliage'),
    green=('#5E8F4E', 'satin'),
    green_light=('#78A85A', 'satin'),
    green_dark=('#4F8A45', 'satin'),
    stem=('#5C7A3A', 'satin'),
    # fruit, flowers and birds
    tomato=('#D9483B', 'gloss'),
    carrot=('#F28C28', 'satin'),
    courgette=('#4F7D3A', 'satin'),
    pink=('#F28DB2', 'satin'),
    yellow=('#F6C744', 'satin'),
    lilac=('#B8A1E3', 'satin'),
    white=('#FFFFFF', 'satin'),
    orchid=('#E88CC2', 'satin'),
    orchid_b=('#C98BE0', 'satin'),
    bird_blue=('#4F7FC4', 'satin'),
    bird_breast=('#D9733B', 'satin'),
    bird_yellow=('#E8C23A', 'satin'),
    beak=('#E8B34A', 'satin'),
    toadstool=('#E4572E', 'satin'),
)

WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


def _m(name):
    return getattr(M, name)


def _pot(F, x, y, z, d=0.14, h=0.12, material=None, soil=True, trim=False):
    """A flower pot standing on `y`, `d` across the rim, with soil in it. `trim` leaves out what is never seen:
    the underside, on the top it stands on, and with soil, the inside below it (the wall ends just under the soil)."""
    r = d / 2
    inside = [(r * 0.66, 0.012), (0, 0.012)]
    if trim and soil:
        t = (h * 0.8 - 0.012) / (h - 0.012)                     # the inner wall, 0.8 of the way up
        inside = [(r * (0.66 + 0.24 * t), h * 0.8)]
    G.revolve(_name('pot'), [(0, 0)] * (not trim) + [(r * 0.72, 0), (r, h * 0.86), (r * 1.04, h * 0.88), (r * 1.04, h),
                                                    (r * 0.9, h)] + inside,
              at=F.at(x, y, z), material=material or M.terra, segments=16)
    if soil:
        F.cyl(_name('pot_soil'), r * 0.88, 0.01, (x, y + h * 0.86, z), M.soil, verts=14)


def _blob(F, x, y, z, r, material, seed, scale=(1, 1, 1), noise=0.5):
    """A lumpy clump of leaves standing on `y` (a lettuce, a shrub)."""
    return G.boulder(_name('blob'), r, at=F.at(x, y, z), material=material, seed=seed, noise=noise, subdivisions=2,
                     scale=scale)


def _bush(F, x, y, z, r, h, material, seed, n=5):
    """A leafy plant `h` tall standing on `y`: `n` clumps of radius about `r` climbing round its middle."""
    rnd = random.Random(seed)
    for k in range(n):
        t = k / max(1, n - 1)
        a = k * 137.5 + rnd.uniform(-20, 20)
        d = r * 0.5 * (1 - 0.5 * t)
        rr = r * (1.0 - 0.35 * t) * rnd.uniform(0.85, 1.1)
        _blob(F, x + d * G.cosd(a), y + t * (h - 1.6 * rr), z + d * G.sind(a), rr, material, seed * 10 + k)


def _flower(F, x, y, z, material, r=0.03):
    F.sphere(_name('flower'), r, (x, y, z), material, segments=8, rings=6)


# --- the raised veggie bed (2x1): a cedar box of lettuces, carrots, a courgette and staked tomatoes ----------
H = 0.45        # the box's height
SOIL = H - 0.03


def veggieBed():
    F = G.GameFrame()
    rnd = random.Random(2)
    with G.new_parts() as still:
        for z in (-0.41, 0.41):                                    # boards
            for y in (0.11, 0.33):
                F.box(_name('board'), 1.8, 0.21, 0.04, (0, y, z), M.cedar, bevel=0.006)
        for x in (-0.89, 0.89):
            for y in (0.11, 0.33):
                F.box(_name('board'), 0.04, 0.21, 0.78, (x, y, 0), M.cedar, bevel=0.006)
            for z in (-0.41, 0.41):
                F.box(_name('post'), 0.07, H + 0.04, 0.07, (x * 0.99, (H + 0.04) / 2, z * 0.99), M.cedar_dark, bevel=0.01)
        for z in (-0.425, 0.425):
            F.box(_name('rail'), 1.86, 0.035, 0.06, (0, H, z), M.cedar_dark, bevel=0.008)
        for x in (-0.905, 0.905):
            F.box(_name('rail'), 0.06, 0.035, 0.8, (x, H, 0), M.cedar_dark, bevel=0.008)
        F.box('soil', 1.74, 0.03, 0.78, (0, SOIL - 0.015, 0), M.soil)
        for x in (0.3, 0.68):                                      # tomato stakes
            G.rod(_name('stake'), F.at(x, SOIL - 0.02, -0.22), F.at(x, H + 0.85, -0.22), 0.012, material=M.cedar_dark,
                  verts=6)
        F.box('plant_label', 0.08, 0.14, 0.012, (-0.85, H + 0.07, 0.36), M.label)
        F.box('label_stick', 0.015, 0.1, 0.008, (-0.85, H - 0.01, 0.36), M.label)
    G.join(still, 'body', origin=(0, 0, 0))
    # The crop, origin on the soil: the plants (foliage) and the fruit (plain), each following the crop's growth
    # stage (`grow`): the plants low when just planted, the fruit only when ripe.
    with G.new_parts() as plants:
        for i in range(3):                                         # lettuces, front row (the fourth is picked)
            _blob(F, -0.7 + i * 0.22, SOIL, 0.22, 0.1, M.leaf_pale if i % 2 else M.leaf_light, i * 1.3,
                  scale=(1, 1, 0.7))
        for i in range(6):                                         # carrot tops, middle row
            x = -0.72 + i * 0.13
            for k in range(3):
                a = k * 120 + i * 20
                G.rod(_name('carrot_top'), F.at(x, SOIL, -0.05),
                      F.at(x + 0.04 * G.cosd(a), SOIL + 0.16, -0.05 + 0.04 * G.sind(a)), 0.012, material=M.leaf_light,
                      verts=4)
        for x in (0.3, 0.68):                                      # tomato plants on their stakes
            _bush(F, x, SOIL, -0.2, 0.13, 0.82, M.leaf_dark, int(x * 10), n=6)
        for k in range(3):                                         # courgette leaves
            _blob(F, 0.42 + 0.12 * k, SOIL, 0.12 + 0.05 * (k % 2), 0.11, M.leaf, 9 + k, scale=(1.2, 1, 0.45))
    plants = G.join(plants, 'plants', origin=F.at(0, SOIL, 0))
    with G.new_parts() as fruit:
        for x in (0.3, 0.68):
            for j in range(5):
                a = j * 1.3 + x
                F.sphere(_name('tomato'), 0.035, (x + math.cos(a) * 0.14, SOIL + 0.22 + j * 0.1, -0.2 + math.sin(a) * 0.13),
                         M.tomato, segments=10, rings=6)
        for i in range(6):
            F.cyl(_name('carrot'), 0.022, 0.03, (-0.72 + i * 0.13, SOIL + 0.01, -0.05), M.carrot, verts=8)
        F.cyl('courgette', 0.035, 0.26, (0.46, SOIL + 0.05, 0.3), M.courgette, rot=(80, 0, 30), verts=10)
        F.cyl('courgette_b', 0.03, 0.2, (0.6, SOIL + 0.04, 0.28), M.courgette, rot=(85, 0, -50), verts=10)
        del rnd
    fruit = G.join(fruit, 'fruit', origin=F.at(0, SOIL, 0))
    # The lettuce picked for a snack, origin just under the soil: `harvest` shrinks it away; it follows the
    # plants' growth too.
    with G.new_parts() as lettuce:
        _blob(F, -0.04, SOIL - 0.02, 0.22, 0.1, M.leaf_light, 7.1, scale=(1, 1, 0.8))
    lettuce = G.join(lettuce, 'lettuce', origin=F.at(-0.04, SOIL - 0.02, 0.22))   # flattened, it is under the soil
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.stages(plants, 'grow', picked=1, growing=(1, 1, 0.6), planted=(1, 1, 0.25))
    G.stages(lettuce, 'grow', picked=1, growing=(1, 1, 0.6), planted=(1, 1, 0.25))
    G.stages(fruit, 'grow', picked=0.001, growing=0.001, planted=0.001)
    G.squash(lettuce, 'harvest', (1, 1, 0.001), seconds=0.8)   # down into the soil (only Z: see the notes)
    # Tending: the plants and their fruit rustle, swaying 3° back and forth about the soil line through the bed's
    # middle (their shared origin), on top of the growth stage (objectMotion.ts plays a use clip after it).
    for part in (plants, fruit):
        G.rock(part, 'rustle', 3, seconds=0.8, axis='X')
    return objs


# --- the potting bench (2x1): a sage bench against the wall, pots and seedlings, tools on the back board -------
TOP = 0.88
SEEDLING = (-0.62, -0.02)    # the pot the seedling comes up in (x, depth)
BOUQUET = (-0.02, 0.09)      # where the tied bouquet lies on the worktop's front (x, depth: its twine)


def pottingBench():
    F = G.GameFrame()
    rnd = random.Random(4)
    with G.new_parts() as still:
        F.box('worktop', 1.76, 0.05, 0.6, (0, TOP - 0.025, -0.13), M.bench, bevel=0.01)
        F.box('lower_shelf', 1.7, 0.03, 0.5, (0, 0.22, -0.15), M.bench, bevel=0.008)
        F.box('back_board', 1.76, 0.86, 0.04, (0, TOP + 0.43, WALL + 0.02), M.sage, bevel=0.012)
        F.box('top_shelf', 1.76, 0.03, 0.18, (0, TOP + 0.62, -0.33), M.bench, bevel=0.008)
        for x in (-0.6, 0.0, 0.6):
            F.box(_name('shelf_bracket'), 0.025, 0.1, 0.14, (x, TOP + 0.56, -0.35), M.sage, bevel=0.005)
        for x in (-0.84, 0.84):
            for z in (-0.39, 0.12):
                F.box(_name('leg'), 0.06, TOP - 0.05, 0.06, (x, (TOP - 0.05) / 2, z), M.sage, bevel=0.01)
            F.box(_name('side_rail'), 0.04, 0.06, 0.45, (x, 0.12, -0.135), M.sage, bevel=0.006)
        F.box('front_rail', 1.62, 0.06, 0.04, (0, 0.12, 0.12), M.sage, bevel=0.006)
        for i, x in enumerate((-0.6, -0.3, 0.0, 0.3, 0.6)):          # pots of herbs on the top shelf
            _pot(F, x, TOP + 0.635, -0.33, d=0.13 - (i % 2) * 0.02, h=0.11, trim=True)
            _bush(F, x, TOP + 0.72, -0.33, 0.055, 0.17, (M.leaf, M.leaf_light, M.leaf_pale)[i % 3], i, n=3)
        # tools hung on the back board, coiled twine
        G.rod('trowel_handle', F.at(-0.75, TOP + 0.35, -0.385), F.at(-0.75, TOP + 0.16, -0.385), 0.014,
              material=M.cedar_dark, verts=8)
        F.box('trowel_blade', 0.07, 0.14, 0.008, (-0.75, TOP + 0.08, -0.39), M.steel, bevel=0.02)
        G.rod('fork_handle', F.at(-0.6, TOP + 0.35, -0.385), F.at(-0.6, TOP + 0.16, -0.385), 0.014,
              material=M.cedar_dark, verts=8)
        for dx in (-0.025, 0.0, 0.025):
            F.box(_name('fork_tine'), 0.008, 0.12, 0.006, (-0.6 + dx, TOP + 0.09, -0.39), M.steel)
        G.torus('twine', 0.08, 0.025, at=F.at(0.72, TOP + 0.3, -0.385), material=M.green_dark, rot=(90, 0, 0),
                major_segments=16, minor_segments=8)
        # on the worktop: a seedling tray, a flowering pot, the watering can, the empty pot for the seedling
        F.box('tray', 0.5, 0.06, 0.3, (-0.15, TOP + 0.03, -0.14), M.dark, bevel=0.008)
        for i in range(4):
            for j in range(2):
                _blob(F, -0.33 + i * 0.12, TOP + 0.05, -0.2 + j * 0.13, 0.04, (M.leaf_pale, M.leaf_light)[(i + j) % 2],
                      i * 3 + j, scale=(1, 1, 0.9))
        _pot(F, 0.25, TOP, -0.15, d=0.2, h=0.18, trim=True)
        _bush(F, 0.25, TOP + 0.14, -0.15, 0.1, 0.3, M.leaf_light, 21, n=4)
        for k, m in enumerate(('pink', 'yellow', 'pink', 'lilac', 'yellow')):
            a = k * 72
            _flower(F, 0.25 + 0.1 * G.cosd(a), TOP + 0.33 + 0.03 * (k % 2), -0.15 + 0.1 * G.sind(a), _m(m))
        G.revolve('watering_can', [(0, 0), (0.08, 0), (0.09, 0.14), (0.06, 0.18), (0, 0.18)],
                  at=F.at(0.62, TOP, -0.1), material=M.can, segments=16)
        G.rod('can_spout', F.at(0.68, TOP + 0.06, -0.1), F.at(0.84, TOP + 0.2, -0.1), 0.012, material=M.can, verts=8)
        G.torus('can_handle', 0.07, 0.012, at=F.at(0.6, TOP + 0.2, -0.1), material=M.can, rot=(90, 0, 0),
                major_segments=14, minor_segments=6)
        _pot(F, SEEDLING[0], TOP, SEEDLING[1], d=0.14, h=0.12, trim=True)
        # below: sacks of compost and a stack of pots
        for i in range(3):
            F.box(_name('sack'), 0.32, 0.3, 0.26, (-0.55 + i * 0.36, 0.385, -0.16), (M.sack, M.seed, M.sack)[i],
                  rot=(0, 0, (i - 1) * 12), bevel=0.06, bevel_segments=2)
        _pot(F, 0.55, 0.235, -0.12, d=0.24, h=0.16, soil=False, trim=True)
        _pot(F, 0.55, 0.36, -0.12, d=0.22, h=0.12, soil=False, trim=True)
        del rnd
    G.join(still, 'body', origin=(0, 0, 0))
    # The seedling: a stem, two leaves and a tuft, origin on the pot's soil: `seedling` grows it up. Plain satin:
    # a foliage part would grow as well since the renderer's fixes (see the notes).
    sx, sz = SEEDLING
    sy = TOP + 0.103
    with G.new_parts() as seedling:
        G.rod('seedling_stem', F.at(sx, sy, sz), F.at(sx + 0.01, sy + 0.16, sz), 0.006, material=M.green, verts=6)
        for s in (-1, 1):
            F.sphere(_name('seed_leaf'), 0.5, (sx + s * 0.04, sy + 0.12, sz), M.green_light, scale=(0.07, 0.03, 0.012),
                     rot=(0, s * 20, 0), segments=10, rings=6)
        _blob(F, sx + 0.01, sy + 0.14, sz, 0.035, M.green, 5)
    seedling = G.join(seedling, 'seedling', origin=F.at(sx, sy, sz))
    seedling.scale[2] = 0.001
    # The bouquet a skilled gardener ties: stems lying along the worktop's front, flower heads, a twine wrap.
    # Its origin is inside the worktop, flattened out of sight: `bouquet` grows it while it is tied.
    bx, bz = BOUQUET
    with G.new_parts() as bouquet:
        for k, dz in enumerate((-0.025, 0.0, 0.025)):
            G.rod(_name('bouquet_stem'), F.at(bx - 0.15, TOP + 0.012, bz + dz * 0.4),
                  F.at(bx + 0.1, TOP + 0.014 + 0.006 * (k % 2), bz + dz), 0.006, material=M.green, verts=4)
        for k, (dx, dy, dz, m) in enumerate(((0.12, 0.03, -0.03, 'pink'), (0.135, 0.04, 0.005, 'yellow'),
                                             (0.115, 0.032, 0.035, 'lilac'))):
            F.sphere(_name('bouquet_flower'), 0.03, (bx + dx, TOP + dy, bz + dz), _m(m), segments=7, rings=5)
        G.torus('bouquet_twine', 0.02, 0.006, at=F.at(bx - 0.02, TOP + 0.017, bz), material=M.green_dark, rot=(0, 90, 0),
                major_segments=8, minor_segments=3)
    bouquet = G.join(bouquet, 'bouquet', origin=F.at(bx, TOP - 0.01, bz))
    bouquet.scale[2] = 0.001
    objs = G.settle(centre=False)
    G.grow(seedling, 'seedling', seconds=1.0)
    G.grow(bouquet, 'bouquet', seconds=0.8)
    return objs


# --- the bird feeder post (1x1): a cedar post, a feeder house on a tray, a seed tube, flowers round its foot -----
def _bird(F, x, y, z, body, breast, turn=0):
    """A little bird perched on `y`, looking to the front (or `turn` degrees round)."""
    c, s = G.cosd(turn), G.sind(turn)
    F.sphere(_name('bird_body'), 0.5, (x, y + 0.03, z), body, scale=(0.06, 0.09, 0.055), rot=(0, 0, -turn),
             segments=12, rings=8)
    F.sphere(_name('bird_head'), 0.024, (x + 0.035 * s, y + 0.075, z + 0.035 * c), body, segments=10, rings=8)
    F.sphere(_name('bird_breast'), 0.5, (x + 0.012 * s, y + 0.025, z + 0.015 * c), breast, scale=(0.045, 0.05, 0.05),
             segments=10, rings=6)
    G.rod(_name('beak'), F.at(x + 0.05 * s, y + 0.077, z + 0.05 * c), F.at(x + 0.07 * s, y + 0.074, z + 0.07 * c), 0.007,
          material=M.beak, verts=5)
    G.rod(_name('tail'), F.at(x - 0.035 * s, y + 0.035, z - 0.035 * c), F.at(x - 0.08 * s, y + 0.05, z - 0.08 * c),
          0.012, material=body, verts=5)


def birdFeeder():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('post', 0.08, 1.5, 0.08, (0, 0.75, -0.05), M.cedar_dark, bevel=0.012)
        F.box('tray', 0.36, 0.03, 0.3, (0, 1.5, -0.05), M.cedar, bevel=0.008)
        for z in (0.1, -0.2):
            F.box(_name('tray_lip'), 0.38, 0.03, 0.03, (0, 1.53, z), M.cedar, bevel=0.006)
        F.box('house', 0.24, 0.17, 0.18, (0, 1.6, -0.05), M.house, bevel=0.008)
        for s in (-1, 1):                                          # the roof's two slopes
            F.box(_name('roof'), 0.32, 0.025, 0.17, (0, 1.725, -0.05 + s * 0.065), M.roof, rot=(s * 30, 0, 0),
                  bevel=0.006)
        F.box('ridge', 0.33, 0.025, 0.025, (0, 1.77, -0.05), M.cedar_dark, bevel=0.006)
        F.cyl('entry', 0.03, 0.01, (0, 1.62, 0.04), M.dark, rot=(90, 0, 0), verts=14)
        F.box('seed', 0.3, 0.012, 0.22, (0, 1.521, -0.05), M.seed)
        G.rod('arm', F.at(0.04, 1.3, -0.05), F.at(0.25, 1.32, -0.05), 0.016, material=M.cedar_dark, verts=8)
        G.rod('hanger', F.at(0.25, 1.32, -0.05), F.at(0.25, 1.27, -0.05), 0.004, material=M.steel, verts=6)
        G.revolve('seed_tube', [(0, 0), (0.04, 0), (0.04, 0.24), (0.03, 0.27), (0, 0.27)],
                  at=F.at(0.25, 0.99, -0.05), material=M.tube, segments=14)
        F.cyl('tube_seed', 0.036, 0.2, (0.25, 1.1, -0.05), M.seed, verts=12)    # seen through its window
        F.cyl('tube_dish', 0.06, 0.015, (0.25, 0.985, -0.05), M.roof, verts=16)
        G.rod('perch', F.at(0.17, 1.02, -0.05), F.at(0.33, 1.02, -0.05), 0.005, material=M.steel, verts=6)
        _bird(F, 0.32, 1.0, 0.0, M.bird_yellow, M.bird_yellow, turn=30)
        # a ring of flowering clumps round the foot
        for i in range(8):
            a = i * 45
            x, z = G.cosd(a) * 0.26, -0.05 + G.sind(a) * 0.26
            _blob(F, x, 0.0, z, 0.08, (M.leaf, M.leaf_light)[i % 2], i, scale=(1, 1, 1.1))
            _flower(F, x, 0.17, z, _m(('pink', 'yellow', 'lilac', 'white')[i % 4]), r=0.028)
    G.join(still, 'body', origin=(0, 0, 0))
    # The bird that comes to the tray, built at full size and scaled to nothing: `bird` brings it in.
    with G.new_parts() as bird:
        _bird(F, -0.1, 1.535, 0.1, M.bird_blue, M.bird_breast, turn=-20)
    bird = G.join(bird, 'bird', origin=F.at(-0.1, 1.535, 0.1))
    bird.scale = (0.001, 0.001, 0.001)
    objs = G.settle(centre=False)
    G.squash(bird, 'bird', 1000, seconds=0.5)
    return objs


# --- the garden hammock (2x1): a canvas sling hung from a curved cedar stand, rocking for the resident ----------
HOOK = (0.88, 1.02)    # the hooks' (x, height): the sling swings about the line between them
LOW, END, XS = 0.475, 0.86, 0.72   # the sling's low point, its ends' height and its ends' x


def gardenHammock():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('beam', 1.7, 0.08, 0.1, (0, 0.05, 0.0), M.cedar_wood, bevel=0.02)
        for s in (-1, 1):
            F.box(_name('foot'), 0.1, 0.05, 0.62, (s * 0.66, 0.035, 0.0), M.cedar_wood_dark, bevel=0.015)
            # the curved arm, in three pieces from the beam's end up to its hook
            pts = [(s * 0.6, 0.07), (s * 0.74, 0.4), (s * 0.85, 0.75), (s * 0.9, 1.08)]
            for a, b in zip(pts, pts[1:]):
                G.rod(_name('arm'), F.at(a[0], a[1], 0), F.at(b[0], b[1], 0), 0.04, material=M.cedar_wood, verts=8)
                F.sphere(_name('arm_joint'), 0.04, (b[0], b[1], 0), M.cedar_wood, segments=8, rings=6)
            G.torus(_name('hook'), 0.025, 0.006, at=F.at(s * HOOK[0], HOOK[1], 0), material=M.cedar_wood_dark,
                    rot=(90, 0, 0), major_segments=10, minor_segments=4)
    G.join(still, 'body', origin=(0, 0, 0))
    # The sling: striped canvas sagging between two spreader bars, roped to the hooks, a pillow at one end.
    with G.new_parts() as sling:
        segs, width = 12, 0.62
        stripes = ('canvas', 'stripe', 'canvas', 'canvas', 'stripe', 'canvas')
        for i in range(segs):
            u0, u1 = i / segs, (i + 1) / segs
            xa, xb = -XS + 2 * XS * u0, -XS + 2 * XS * u1
            ya = LOW + (END - LOW) * (2 * u0 - 1) ** 2
            yb = LOW + (END - LOW) * (2 * u1 - 1) ** 2
            length = math.hypot(xb - xa, yb - ya) + 0.004
            ang = math.degrees(math.atan2(yb - ya, xb - xa))
            zw = width / len(stripes)
            for j, m in enumerate(stripes):
                F.box(_name('sling'), length, 0.025, zw + 0.002, ((xa + xb) / 2, (ya + yb) / 2, -width / 2 + zw * (j + 0.5)),
                      _m(m), rot=(0, ang, 0))
        for s in (-1, 1):
            F.box(_name('spreader'), 0.04, 0.04, 0.68, (s * XS, END, 0), M.cedar_wood_dark, bevel=0.012)
            for z in (-0.3, 0.0, 0.3):
                G.rod(_name('rope'), F.at(s * HOOK[0], HOOK[1], 0), F.at(s * XS, END + 0.01, z), 0.008, material=M.rope,
                      verts=5)
        F.box('pillow', 0.36, 0.11, 0.3, (-0.54, 0.73, 0.0), M.pillow, rot=(0, 28, 0), bevel=0.045, bevel_segments=2)
    sling = G.hinged(G.join(sling, 'sling'), F.at(0, HOOK[1], 0))
    objs = G.settle(centre=False)
    G.rock(sling, 'sway', 8, seconds=2.4, axis='X')
    return objs


# --- the cedar outdoor shower (1x1): slatted cedar walls on two sides, a duckboard, a copper rain head ---------
HEAD = (0.15, 2.0, -0.08)   # the rain head's underside


def cedarShower():
    F = G.GameFrame()
    with G.new_parts() as still:
        for i in range(12):                                         # back wall slats
            F.box(_name('slat'), 0.065, 1.9, 0.03, (-0.41 + i * 0.074, 1.0, WALL + 0.015),
                  M.cedar_wood if i % 2 else M.cedar_wood_dark, bevel=0.005)
        for i in range(11):                                         # left wall slats
            F.box(_name('slat'), 0.03, 1.9, 0.065, (-0.43, 1.0, -0.35 + i * 0.077),
                  M.cedar_wood if i % 2 else M.cedar_wood_dark, bevel=0.005)
        for y in (0.06, 1.97):
            F.box(_name('rail'), 0.9, 0.05, 0.04, (0.0, y, WALL + 0.045), M.cedar_wood_dark, bevel=0.01)
            F.box(_name('rail'), 0.04, 0.05, 0.86, (-0.4, y, 0.0), M.cedar_wood_dark, bevel=0.01)
        for i in range(7):                                          # the duckboard on two bearers
            F.box(_name('duck'), 0.78, 0.03, 0.08, (0.02, 0.05, -0.33 + i * 0.11), M.cedar_wood, bevel=0.006)
        for x in (-0.3, 0.32):
            F.box(_name('bearer'), 0.06, 0.035, 0.74, (x, 0.0175, 0.0), M.cedar_wood_dark, bevel=0.006)
        # the copper riser, arm, rain head and valve
        G.rod('riser', F.at(HEAD[0], 0.05, -0.395), F.at(HEAD[0], 2.06, -0.395), 0.018, material=M.copper, verts=10)
        G.rod('arm', F.at(HEAD[0], 2.06, -0.395), F.at(HEAD[0], 2.06, -0.08), 0.016, material=M.copper, verts=10)
        F.sphere('elbow', 0.022, (HEAD[0], 2.06, -0.395), M.copper, segments=10, rings=6)
        F.cyl('rain_head', 0.15, 0.03, (HEAD[0], HEAD[1] + 0.025, HEAD[2]), M.copper, verts=24, bevel=0.008)
        F.cyl('rain_face', 0.13, 0.004, (HEAD[0], HEAD[1] + 0.009, HEAD[2]), M.steel, verts=24)
        F.cyl('valve', 0.04, 0.04, (HEAD[0], 1.1, -0.37), M.copper, rot=(90, 0, 0), verts=14)
        G.rod('valve_lever', F.at(HEAD[0], 1.1, -0.35), F.at(HEAD[0] + 0.09, 1.1, -0.35), 0.008, material=M.copper, verts=6)
        # a towel on the left wall, a soap shelf, a potted plant and pebbles
        F.box('towel', 0.04, 0.4, 0.2, (-0.39, 1.3, 0.25), M.towel, bevel=0.015)
        F.box('towel_bar', 0.02, 0.02, 0.26, (-0.4, 1.5, 0.25), M.copper)
        F.box('soap_shelf', 0.12, 0.03, 0.1, (-0.36, 1.0, -0.2), M.cedar_wood_dark, bevel=0.006)
        F.box('soap', 0.08, 0.03, 0.05, (-0.36, 1.03, -0.2), M.soap, bevel=0.012)
        _pot(F, 0.3, 0.0, 0.3, d=0.2, h=0.18)
        _bush(F, 0.3, 0.16, 0.3, 0.1, 0.4, M.green_light, 3, n=4)
        for i, (x, z) in enumerate(((0.12, 0.4), (0.38, 0.06), (-0.2, 0.4), (0.38, -0.14))):
            G.boulder(_name('pebble'), 0.04, at=F.at(x, 0.0, z), material=(M.pebble, M.pebble_b)[i % 2], seed=i,
                      subdivisions=1, scale=(1.2, 1, 0.6))
    G.join(still, 'body', origin=(0, 0, 0))
    # The rain: streaks falling from the head to the duckboard, origin at the head: `water_on` grows them down.
    with G.new_parts() as rain:
        rnd = random.Random(6)
        x, y, z = HEAD
        for k in range(18):
            r = 0.11 * math.sqrt((k + 0.5) / 18)
            a = k * 137.5
            px, pz = x + r * G.cosd(a), z + r * G.sind(a)
            h = rnd.uniform(1.86, 1.93)
            G.rod(_name('stream'), F.at(px, y, pz), F.at(px, y - h, pz), 0.005, material=M.water, verts=4)
    rain = G.join(rain, 'water', origin=F.at(x, y, z))
    rain.scale[2] = 0.001
    objs = G.settle(centre=False)
    G.grow(rain, 'water_on', seconds=0.6)
    return objs


# --- the moss terrarium (1x1): a little forest under a glass cloche, on a walnut pedestal ----------------------
TZ = -0.1      # the pedestal's depth
TOPY = 0.78    # the soil's top


def mossTerrarium():
    F = G.GameFrame()
    with G.new_parts() as still:
        G.revolve('pedestal', [(0, 0), (0.17, 0), (0.155, 0.03), (0.07, 0.07), (0.045, 0.3), (0.045, 0.56), (0.1, 0.64),
                               (0.17, 0.68), (0, 0.68)], at=F.at(0, 0, TZ), material=M.walnut, segments=24)
        F.cyl('plate', 0.2, 0.04, (0, 0.7, TZ), M.walnut, verts=28, bevel=0.008)
        F.cyl('soil', 0.18, 0.04, (0, 0.74, TZ), M.soil, verts=24)
        G.boulder('moss_mound', 0.15, at=F.at(0, 0.755, TZ), material=M.moss, seed=1.3, noise=0.35, subdivisions=2,
                  scale=(1, 1, 0.35))
        G.boulder('rock', 0.05, at=F.at(0.08, 0.77, TZ + 0.05), material=M.pebble, seed=2, subdivisions=1,
                  scale=(1.2, 1, 0.8))
        F.cone('fir', 0.05, 0.0, 0.22, (-0.07, 0.88, TZ - 0.02), M.leaf_dark, verts=8)
        F.cone('fir_b', 0.035, 0.0, 0.15, (0.06, 0.84, TZ - 0.06), M.leaf, verts=8)
        F.cyl('fir_trunk', 0.008, 0.04, (-0.07, 0.77, TZ - 0.02), M.cedar_dark, verts=6)
        F.cyl('toadstool_stem', 0.006, 0.03, (-0.02, 0.785, TZ + 0.1), M.white, verts=6)
        F.sphere('toadstool', 0.5, (-0.02, 0.8, TZ + 0.1), M.toadstool, scale=(0.04, 0.04, 0.02), segments=10, rings=5)
        # the mister on the floor beside it, and a round mossy rug
        G.revolve('mister', [(0, 0), (0.03, 0), (0.03, 0.12), (0.012, 0.15), (0.012, 0.17), (0, 0.17)],
                  at=F.at(0.24, 0.012, 0.15), material=M.mister, segments=14)
        F.box('mister_head', 0.04, 0.04, 0.03, (0.24, 0.2, 0.15), M.white, bevel=0.008)
        F.cyl('rug', 0.21, 0.012, (0, 0.006, 0.24), M.rug, verts=28, scale=(1, 0.75, 1))
    G.join(still, 'body', origin=(0, 0, 0))
    # The glass cloche and its knob, resting on the plate: `lid_open` lifts it.
    with G.new_parts() as lid:
        G.revolve('cloche', [(0.185, 0), (0.19, 0.01), (0.19, 0.23), (0.16, 0.33), (0.1, 0.39), (0, 0.41),
                             (0, 0.4), (0.098, 0.384), (0.156, 0.326), (0.182, 0.23), (0.182, 0.01), (0.178, 0)],
                  at=F.at(0, 0.72, TZ), material=M.glass, segments=28)
        F.sphere('cloche_knob', 0.03, (0, 1.15, TZ), M.glass, scale=(1, 1, 0.8), segments=12, rings=8)
    lid = G.join(lid, 'cloche', origin=F.at(0, 0.72, TZ))
    objs = G.settle(centre=False)
    G.slide(lid, 'lid_open', (0, 0, 0.22), seconds=0.7)
    return objs


# --- the fern glasshouse (2x2): a white-framed glass room, a bench for two, ferns, orchids and palms -----------
GW, GD, HW, HR = 1.86, 1.86, 2.0, 2.45      # width, depth, wall height, ridge height
DOOR = 0.8                                   # the doorway's width, in the middle of the front
VENT = (0.3, 0.55)                           # the roof vent: half its length (depth), its length down the slope


def _fern(F, x, y, z, material, spread=0.25, n=7, seed=0):
    """A fern: fronds arching out from a pot's soil at (x, y, z)."""
    rnd = random.Random(seed)
    for k in range(n):
        a = k * 360 / n + rnd.uniform(-15, 15)
        L = spread * rnd.uniform(0.8, 1.1)
        cx, cz = x + 0.5 * L * G.cosd(a), z + 0.5 * L * G.sind(a)
        F.sphere(_name('frond'), 0.5, (cx, y + 0.08, cz), material, scale=(L, 0.07, 0.012),
                 rot=(rnd.uniform(-25, 25), 0, -a), segments=8, rings=4)


def glasshouse():
    F = G.GameFrame()
    hx, hz = GW / 2 - 0.03, GD / 2 - 0.03
    slope = math.degrees(math.atan2(HR - HW, GW / 2))
    run = math.hypot(GW / 2, HR - HW)
    with G.new_parts() as still:
        F.box('base', GW, 0.12, GD, (0, 0.06, 0), M.stone, bevel=0.01)
        F.box('gravel', GW - 0.06, 0.012, GD - 0.06, (0, 0.126, 0), M.gravel)
        # the white frame: corner posts, eaves, gable rafters, the ridge, glazing bars and the doorway
        for x in (-hx, hx):
            for z in (-hz, hz):
                F.box(_name('post'), 0.05, HW, 0.05, (x, HW / 2, z), M.frame)
        for z in (-hz, hz):
            F.box(_name('eave'), GW, 0.05, 0.05, (0, HW, z), M.frame)
            for s in (-1, 1):
                F.box(_name('rafter'), run, 0.05, 0.05, (s * GW / 4, (HW + HR) / 2, z), M.frame, rot=(0, -s * slope, 0))
        for x in (-hx, hx):
            F.box(_name('eave'), 0.05, 0.05, GD, (x, HW, 0), M.frame)
        F.box('ridge', 0.06, 0.06, GD, (0, HR, 0), M.frame)
        for z in (-0.45, 0.0, 0.45):
            for x in (-hx, hx):
                F.box(_name('bar'), 0.03, HW - 0.15, 0.03, (x, HW / 2 + 0.05, z), M.frame)
            for s in (-1, 1):
                F.box(_name('roof_bar'), run, 0.03, 0.03, (s * GW / 4, (HW + HR) / 2 + 0.01, z), M.frame,
                      rot=(0, -s * slope, 0))
        for x in (-0.45, 0.0, 0.45):
            F.box(_name('bar'), 0.03, HW - 0.15, 0.03, (x, HW / 2 + 0.05, -hz), M.frame)
        F.box('back_rail', GW, 0.04, 0.04, (0, 0.95, -hz), M.frame)
        for s in (-1, 1):
            F.box(_name('door_post'), 0.05, HW, 0.05, (s * (DOOR / 2 + 0.025), HW / 2, hz), M.frame)
            F.box(_name('front_rail'), (GW - DOOR) / 2 - 0.08, 0.04, 0.04, (s * (GW + DOOR) / 4, 0.95, hz), M.frame)
        F.box('lintel', DOOR + 0.1, 0.05, 0.05, (0, 1.92, hz), M.frame)
        # the glass: back and side walls, the front either side of the door, the gable ends, the roof
        F.box('glass_back', GW - 0.08, HW - 0.15, 0.012, (0, HW / 2 + 0.05, -hz), M.glass)
        for x in (-hx, hx):
            F.box(_name('glass_side'), 0.012, HW - 0.15, GD - 0.08, (x, HW / 2 + 0.05, 0), M.glass)
        for s in (-1, 1):
            F.box(_name('glass_front'), (GW - DOOR) / 2 - 0.08, HW - 0.15, 0.012, (s * (GW + DOOR) / 4, HW / 2 + 0.05, hz),
                  M.glass)
        F.box('glass_transom', DOOR, 0.06, 0.012, (0, 1.97, hz), M.glass)
        for z in (-hz, hz):                                        # the gables: thin triangular prisms
            r = (HR - HW - 0.03) / 1.5                             # apex r above the axis, base r / 2 below
            G.revolve(_name('glass_gable'), [(0, -0.006), (r, -0.006), (r, 0.006), (0, 0.006)],
                      at=F.at(0, HW + 0.025 + r / 2, z), material=M.glass, segments=3, rot=(90, -90, 0),
                      scale=(1, (GW / 2 - 0.05) / (0.866 * r), 1))
        # the roof: the left slope whole; the right one round its vent
        F.box('glass_roof_l', run, 0.012, GD - 0.04, (-GW / 4, (HW + HR) / 2 + 0.02, 0), M.glass, rot=(0, slope, 0))
        vz, vl = VENT
        for s in (-1, 1):
            d = (GD - 0.04) / 2 - vz
            F.box(_name('glass_roof_r'), run, 0.012, d, (GW / 4, (HW + HR) / 2 + 0.02, s * (vz + d / 2)), M.glass,
                  rot=(0, -slope, 0))
        low = run - vl                                             # the part of the slope below the vent
        t = 1 - (low / 2) / run                                    # where its middle is, from the ridge
        F.box('glass_roof_r_low', low, 0.012, 2 * vz, (GW / 2 * t, HR - (HR - HW) * t + 0.02, 0), M.glass,
              rot=(0, -slope, 0))
        # the bench for two (seat top 0.45 at depth 0), its cushions
        F.box('bench_seat', 1.4, 0.05, 0.42, (0, 0.415, -0.02), M.cedar, bevel=0.012)
        F.box('bench_back', 1.4, 0.3, 0.04, (0, 0.66, -0.25), M.cedar, rot=(10, 0, 0), bevel=0.012)
        for x in (-0.64, 0.64):
            F.box(_name('bench_end'), 0.05, 0.3, 0.38, (x, 0.27, -0.02), M.cedar_dark, bevel=0.01)
            G.rod(_name('bench_back_post'), F.at(x, 0.42, -0.21), F.at(x, 0.8, -0.28), 0.02, material=M.cedar_dark, verts=6)
        for x in (-0.5, 0.5):
            F.box(_name('cushion'), 0.6, 0.04, 0.36, (x, 0.46, -0.01), M.cushion, bevel=0.015)
        # the back shelf: orchids and ferns
        F.box('shelf', 1.7, 0.04, 0.3, (0, 0.78, -0.72), M.cedar, bevel=0.008)
        for x in (-0.72, 0.72):
            F.box(_name('shelf_end'), 0.04, 0.66, 0.26, (x, 0.45, -0.72), M.cedar_dark, bevel=0.006)
        for i, x in enumerate((-0.32, 0.18, 0.68)):
            _pot(F, x, 0.8, -0.66, d=0.12, h=0.1, material=M.porcelain)
            for k in range(2):
                F.sphere(_name('orchid_leaf'), 0.5, (x + (k - 0.5) * 0.06, 0.92, -0.66), M.green_dark,
                         scale=(0.12, 0.05, 0.015), rot=(0, (k - 0.5) * 50, 0), segments=8, rings=4)
            G.rod(_name('orchid_stem'), F.at(x, 0.9, -0.66), F.at(x + 0.03, 1.18, -0.65), 0.005, material=M.stem, verts=5)
            for k in range(3):
                F.sphere(_name('orchid'), 0.5, (x + 0.01 + 0.025 * k, 1.08 + 0.04 * k, -0.635),
                         _m(('orchid', 'white', 'orchid_b')[i]), scale=(0.06, 0.05, 0.025), segments=8, rings=6)
        for k, x in enumerate((-0.6, -0.07, 0.43)):
            _pot(F, x, 0.8, -0.7, d=0.16, h=0.12)
            _fern(F, x, 0.9, -0.7, (M.green, M.green_dark, M.green_light)[k], spread=0.18, seed=k)
        # potted palms in the front corners, hanging baskets
        for s in (-1, 1):
            _pot(F, s * 0.64, 0.13, 0.6, d=0.3, h=0.26)
            G.rod(_name('palm_trunk'), F.at(s * 0.64, 0.35, 0.6), F.at(s * 0.64, 0.85, 0.6), 0.03, material=M.cedar_dark,
                  verts=8)
            for k in range(6):
                a = k * 60 + 15
                F.sphere(_name('palm_leaf'), 0.5, (s * 0.64 + 0.16 * G.cosd(a), 0.82, 0.6 + 0.16 * G.sind(a)), M.green_light,
                         scale=(0.34, 0.1, 0.014), rot=(0, -25, -a), segments=8, rings=4)
        for x in (-0.68, 0.68):
            G.rod(_name('basket_chain'), F.at(x, HW, -0.3), F.at(x, 1.76, -0.3), 0.005, material=M.steel, verts=5)
            _pot(F, x, 1.6, -0.3, d=0.2, h=0.14)
            _blob(F, x, 1.62, -0.3, 0.14, M.green, x * 3, scale=(1, 1, 0.8))
            for k in range(4):
                a = k * 90 + 45
                G.rod(_name('trailing'), F.at(x + 0.09 * G.cosd(a), 1.7, -0.3 + 0.09 * G.sind(a)),
                      F.at(x + 0.13 * G.cosd(a), 1.42, -0.3 + 0.13 * G.sind(a)), 0.012, material=M.green_dark, verts=5)
    G.join(still, 'body', origin=(0, 0, 0))
    # The door: a glass leaf hinged on the left post, swinging out; the roof vent: a glass panel hinged at
    # the ridge, lifting. `door_open` lets the resident in and shuts behind them (and lets them out), `vent_open`
    # airs it while they're in.
    with G.new_parts() as door:
        F.box('door_glass', DOOR - 0.01, 1.86, 0.014, (0, 0.06 + 0.93, hz), M.glass)
    door = G.hinged(G.join(door, 'door'), F.at(-DOOR / 2, 1.0, hz))
    with G.new_parts() as vent:
        t = (vl / 2) / run                                         # its middle, from the ridge
        F.box('vent_glass', vl, 0.014, 2 * vz, (GW / 2 * t, HR - (HR - HW) * t + 0.025, 0), M.glass, rot=(0, -slope, 0))
    vent = G.hinged(G.join(vent, 'vent'), F.at(0.0, HR + 0.01, 0))
    objs = G.settle(centre=False)
    G.swing(door, 'door_open', 70, seconds=0.9)
    G.swing(vent, 'vent_open', 25, seconds=0.9, axis='Y')
    return objs


FAMILY = G.Family(
    'pack_natureLover', M,
    [G.Model('veggieBed', veggieBed, size=(1.87, 1.3, 0.91), centre=(0, 0.025), key='model.natureLover.veggieBed',
             motions={'uses': {'pickSnack': 'harvest', 'tendVeggies': {'clip': 'rustle', 'delay': 0.6, 'every': 2.4}},
                      'growth': 'grow'},
             paint=[('cedar', 'Wood'), ('cedar_dark', 'Posts & rails')]),
     G.Model('pottingBench', pottingBench, size=(1.76, 1.785, 0.6), key='model.natureLover.pottingBench',
             centre=(0, -0.13), motions={'uses': {'potSeedlings': 'seedling', 'tieBouquet': 'bouquet'}},
             paint=[('bench', 'Wood'), ('sage', 'Frame')]),
     G.Model('birdFeeder', birdFeeder, size=(0.705, 1.783, 0.679), key='model.natureLover.birdFeeder',
             centre=(0.01, -0.05), motions={'uses': {'birdwatch': 'bird'}},
             paint=[('cedar', 'Wood'), ('roof', 'Roof')]),
     G.Model('gardenHammock', gardenHammock, size=(1.88, 1.11, 0.68), key='model.natureLover.gardenHammock',
             motions={'loops': {'swing': 'sway', 'doze': 'sway'}, 'ride': 'sway'},
             paint=[('cedar_wood', 'Wood'), ('canvas', 'Fabric'), ('stripe', 'Stripes')]),
     G.Model('cedarShower', cedarShower, size=(0.9, 2.082, 0.882), key='model.natureLover.cedarShower',
             centre=(0, 0.011),
             motions={'uses': {'rinse': 'water_on'}},
             paint=[('cedar_wood', 'Wood'), ('copper', 'Copper')]),
     G.Model('mossTerrarium', mossTerrarium, size=(0.48, 1.174, 0.698), key='model.natureLover.mossTerrarium',
             centre=(0.03, 0.049), motions={'uses': {'mist': 'lid_open'}},
             paint=[('walnut', 'Wood')]),
     G.Model('glasshouse', glasshouse, size=(1.887, 2.481, 1.86), key='model.natureLover.glasshouse',
             # the door lets the resident in and shuts behind them (`hold`), and opens again to let them out
             # (`out`); the vent stays open while they're in
             motions={'uses': {use: [{'clip': 'door_open', 'hold': 1.5, 'out': 1.5}, 'vent_open'] for use in ('unwind', 'orchids')}},
             paint=[('cedar', 'Wood'), ('frame', 'Frame'), ('cushion', 'Cushions')])],
    comment='Blender-built nature lover pack models (tools/art/blender/families/pack_natureLover.py). '
            'Listed in the base manifest\'s packs. The materials map onto the game\'s finishes so a model '
            'costs one draw call per finish group. Each moves while it is used: a lettuce is picked from '
            'the veggie bed (whose crop follows its growth stage), a seedling comes up, a bird lands, the '
            'hammock sways, the shower runs, the cloche lifts and the glasshouse\'s door and vent open.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

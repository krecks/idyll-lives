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
# The plants are leaves (`plants.leaf`: closed, bent blades, cut into leaflets with `notch`) and flower faces
# (`plants.star`), stems and canes swept; pots turned with a rolled rim; boards and posts chamfered, rails mitred
# round the bed (`G.kit.moulding`), tubes bent in one piece (`G.kit.tube`), cushions, sacks and towels stuffed or
# draped (`G.soft_box`, `G.sweep`), birds `G.kit.songbird`.
#
# Run:  cargo art pack_natureLover [--only veggieBed] [--preview] [--lit]
import math
import os
import random
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gamekit as G  # noqa: E402
from plants import leaf, star  # noqa: E402  (leaf blades and flower faces)

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
    eye=('#1E1C1A', 'gloss'),        # the birds' eyes
    toadstool=('#E4572E', 'satin'),
)

WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
SQUARE = [(-0.5, -0.5), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)]


def _m(name):
    return getattr(M, name)


def _sq(w, h=None):
    """A square bar's profile, `w` across (`h` the other way)."""
    h = w if h is None else h
    return [(u * w, v * h) for u, v in SQUARE]


def _pot(F, name, x, y, z, d=0.14, h=0.12, material=None, soil=True, segments=None):
    """A flower pot standing on `y`, `d` across its rim: a tapered wall under a rolled rim, open below the soil (never
    seen), and its soil a little mound inside the rim."""
    r = d / 2
    seg = segments or (10 if d < 0.15 else 14)
    G.revolve(name, [(r * 0.72, 0), (r * 0.94, h * 0.78), (r * 1.06, h * 0.8), (r * 1.08, h * 0.9), (r * 1.04, h),
                     (r * 0.9, h), (r * 0.86, h * 0.8)], at=F.at(x, y, z), material=material or M.terra, segments=seg)
    if soil:
        G.revolve(f'{name}_soil', [(r * 0.9, h * 0.8), (r * 0.88, h * 0.86), (r * 0.5, h * 0.88), (0, h * 0.9)],
                  at=F.at(x, y, z), material=M.soil, segments=seg)
    return y + h * 0.88


def _rosette(F, name, x, y, z, n, length, width, mats, tilt=(35, 65), droop=0.3, shape='oval', fold=0.3, notch=None,
             rows=3, ring=3, seed=0, spin=0.0):
    """`n` leaves fanning out from (x, y, z) all round, alternately steeper and flatter (`tilt`), longer outside:
    a lettuce, a herb, a clump at a flower's foot."""
    rnd = random.Random(seed)
    base = F.at(x, y, z)
    for k in range(n):
        inner = k % 2
        leaf(f'{name}{k}', base, spin + k * 137.5 + rnd.uniform(-12, 12), tilt[1] if inner else tilt[0],
             length * (0.78 if inner else 1.0) * rnd.uniform(0.9, 1.1), width, mats[k % len(mats)], shape=shape,
             droop=droop * (0.6 if inner else 1.0), fold=fold, rows=rows, ring=ring, notch=notch)


def _flower(F, name, x, y, z, r, material, centre=None, petals=5, tilt=0.0, yaw=0.0):
    """A flower's face (`plants.star`) on (x, y, z), a dot of `centre` in its middle."""
    star(name, F.at(x, y, z), r, petals, material, yaw=yaw, tilt=tilt, inner=0.45, depth=0.006, cup=0.25)
    if centre is not None:
        F.sphere(f'{name}_eye', r * 0.3, (x, y + 0.006, z), centre, scale=(1, 1, 0.6), segments=5, rings=3)


# --- the raised veggie bed (2x1): a cedar box of lettuces, carrots, a courgette and staked tomatoes ----------
H = 0.45        # the box's height
SOIL = H - 0.03
TOMATOES = (0.3, 0.68)   # the staked plants' x, at depth -0.2


def _bed_box(F):
    """Two boards a side between corner posts with pyramid caps, a mitred cap rail round the top, and the soil, mounded
    and uneven; two stakes for the tomatoes and a seed packet on a stick."""
    for z in (-0.41, 0.41):
        for y in (0.11, 0.33):
            F.box(f'board{z}{y}', 1.8, 0.205, 0.04, (0, y, z), M.cedar, bevel=0.006, bevel_segments=1)
    for x in (-0.89, 0.89):
        for y in (0.11, 0.33):
            F.box(f'end{x}{y}', 0.04, 0.205, 0.78, (x, y, 0), M.cedar, bevel=0.006, bevel_segments=1)
        for z in (-0.41, 0.41):
            px, pz = x * 0.99, z * 0.99
            F.box(f'post{x}{z}', 0.07, H + 0.02, 0.07, (px, (H + 0.02) / 2, pz), M.cedar_dark, bevel=0.008, bevel_segments=1)
            bx, by, bz = F.at(px, H + 0.02, pz)
            G.loft(f'post_cap{x}{z}', [G.ring(0.04, 0.04, bz, 4, square=2, at=(bx, by)), G.ring(0.04, 0.04, bz + 0.008, 4, at=(bx, by)),
                                       G.ring(0.004, 0.004, bz + 0.022, 4, at=(bx, by))], material=M.cedar_dark, smooth=False,
                   rot=(0, 0, 0))
    cap = [(0.0, 0.0), (0.06, 0.0), (0.06, 0.026), (0.052, 0.035), (0.008, 0.035), (0.0, 0.026)]   # (in, up)
    G.kit.moulding(F, 'cap_rail', [(-0.935, -0.455), (0.935, -0.455), (0.935, 0.455), (-0.935, 0.455)], cap,
                   M.cedar_dark, at=H - 0.0175, plane='top')
    soil = F.soft('soil', 1.74, 0.03, 0.78, (0, SOIL - 0.015, 0), M.soil, round=0.01, puff=(0, 0.012, 0), flat=6,
                  round_segments=1)
    G.jitter(soil, 0.006, size=0.08, seed=2.0, up=1.0)
    for x in TOMATOES:                                      # the stakes, tapering to their tops
        F.sweep(f'stake{x}', [(x, SOIL - 0.02, -0.22), (x, H + 0.85, -0.22)], None, M.cedar_dark, profile=_sq(0.022),
                up=(0, 1, 0), scale=[1.0, 0.7])
    F.sweep('label_stick', [(-0.85, SOIL - 0.02, 0.36), (-0.85, H + 0.02, 0.36)], None, M.label, profile=_sq(0.012, 0.006),
            up=(0, 1, 0))
    F.box('seed_packet', 0.08, 0.12, 0.006, (-0.85, H + 0.08, 0.364), M.label, bevel=0.002, bevel_segments=1)
    F.box('packet_picture', 0.06, 0.05, 0.002, (-0.85, H + 0.09, 0.368), M.tomato)


def _lettuce(F, name, x, z, y, mats, seed):
    """A lettuce: cupped leaves round a tight heart."""
    _rosette(F, name, x, y, z, 12, 0.14, 0.12, mats, tilt=(20, 62), droop=0.3, fold=0.5, rows=3, ring=4, seed=seed)
    F.sphere(f'{name}heart', 0.035, (x, y + 0.035, z), mats[0], scale=(1, 1, 0.9), segments=8, rings=5)


def _tomato_plant(F, x, z):
    """A tomato plant climbing its stake: a stem winding up it, side shoots and toothed leaves all the way."""
    rnd = random.Random(int(x * 100))
    pts = [(x + 0.03 * math.cos(k * 1.4), SOIL + k * 0.14, z + 0.03 * math.sin(k * 1.4)) for k in range(7)]
    F.sweep(f'tomato_stem{x}', pts, 0.008, M.leaf_dark, verts=5, resolution=2, scale=[1.0, 0.6])
    for k in range(18):
        t = (k + 1) / 19
        a = k * 137.5 + rnd.uniform(-20, 20)
        base = F.at(x + 0.03 * math.cos(t * 6 * 1.4), SOIL + 0.06 + t * 0.78, z + 0.03 * math.sin(t * 6 * 1.4))
        leaf(f'tomato_leaf{x}_{k}', base, a, rnd.uniform(10, 35), 0.2 * (1.1 - 0.4 * t), 0.12, M.leaf_dark, shape='lance',
             droop=0.4, fold=0.2, rows=5, ring=3, notch=(2, 0.5))


def _courgette_plant(F):
    """A courgette's big lobed leaves on long stalks, sprawling over the bed's front right."""
    for k, (x, z, a) in enumerate(((0.42, 0.14, 150), (0.54, 0.2, 200), (0.66, 0.12, 120), (0.5, 0.06, 60), (0.6, 0.26, 240))):
        base = F.at(0.54, SOIL, 0.17)
        tip = F.at(x, SOIL + 0.1, z)
        G.sweep(f'courgette_stalk{k}', [base, ((base[0] + tip[0]) / 2, (base[1] + tip[1]) / 2, tip[2] + 0.04), tip], 0.006,
                material=M.leaf, verts=4, resolution=2)
        leaf(f'courgette_leaf{k}', tip, a, 10, 0.17, 0.19, M.leaf, shape='heart', droop=0.3, fold=0.15, rows=4, ring=4)


def veggieBed():
    F = G.GameFrame()
    with G.new_parts() as still:
        _bed_box(F)
    G.join(still, 'body', origin=(0, 0, 0))
    # The crop, origin on the soil: the plants (foliage) and the fruit (plain), each following the crop's growth
    # stage (`grow`): the plants low when just planted, the fruit only when ripe.
    with G.new_parts() as plants:
        for i in range(3):                                         # lettuces, front row (the fourth is picked)
            _lettuce(F, f'lettuce{i}_', -0.7 + i * 0.22, 0.22, SOIL - 0.005,
                     (M.leaf_pale, M.leaf_light) if i % 2 else (M.leaf_light, M.leaf_pale), i * 3)
        for i in range(6):                                         # feathery carrot tops, middle row
            _rosette(F, f'carrot_top{i}_', -0.72 + i * 0.13, SOIL + 0.01, -0.05, 4, 0.17, 0.05, (M.leaf_light, M.leaf),
                     tilt=(62, 78), droop=0.25, shape='lance', fold=0.1, notch=(2, 0.3), rows=5, ring=3, seed=10 + i)
        for x in TOMATOES:
            _tomato_plant(F, x, -0.2)
        _courgette_plant(F)
    plants = G.join(plants, 'plants', origin=F.at(0, SOIL, 0))
    with G.new_parts() as fruit:
        for x in TOMATOES:                                         # trusses of tomatoes hanging off the plant
            for j in range(5):
                a = j * 1.3 + x
                tx, ty, tz = x + math.cos(a) * 0.1, SOIL + 0.24 + j * 0.11, -0.2 + math.sin(a) * 0.1
                F.sphere(f'tomato{x}{j}', 0.032, (tx, ty, tz), M.tomato, scale=(1, 1, 0.85), segments=10, rings=6)
                star(f'calyx{x}{j}', F.at(tx, ty + 0.026, tz), 0.02, 5, M.stem, inner=0.3, depth=0.003, cup=0.1)
        for i in range(6):                                         # the carrots' orange shoulders out of the soil
            G.revolve(f'carrot{i}', [(0, -0.03), (0.016, -0.01), (0.021, 0.008), (0.014, 0.018), (0, 0.02)],
                      at=F.at(-0.72 + i * 0.13, SOIL, -0.05), material=M.carrot, segments=8)
        for k, (pts, r) in enumerate((([(0.36, SOIL + 0.035, 0.36), (0.46, SOIL + 0.04, 0.34), (0.58, SOIL + 0.035, 0.3)], 0.034),
                                      ([(0.66, SOIL + 0.03, 0.36), (0.72, SOIL + 0.032, 0.28), (0.75, SOIL + 0.03, 0.2)], 0.028))):
            F.sweep(f'courgette{k}', pts, r, M.courgette, verts=8, resolution=3, scale=[0.75, 1.0, 0.85])
            _flower(F, f'courgette_flower{k}', pts[0][0] - 0.03, SOIL + 0.04, pts[0][2] + 0.01, 0.04, M.yellow, tilt=60,
                    yaw=200)
    fruit = G.join(fruit, 'fruit', origin=F.at(0, SOIL, 0))
    # The lettuce picked for a snack, origin just under the soil: `harvest` presses it away; it follows the
    # plants' growth too.
    with G.new_parts() as lettuce:
        _lettuce(F, 'picked', -0.04, 0.22, SOIL - 0.02, (M.leaf_light, M.leaf_pale), 7)
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


def _bench_frame(F):
    """The frame and boards: chamfered legs, rails, a worktop of three planks, a slatted shelf below, a tongue-and-
    groove back board with a peg rail, and the top shelf on shaped brackets."""
    for x in (-0.84, 0.84):
        for z in (-0.39, 0.12):
            F.box(f'leg{x}{z}', 0.06, TOP - 0.05, 0.06, (x, (TOP - 0.05) / 2, z), M.sage, bevel=0.008, bevel_segments=1)
        for y in (0.12, TOP - 0.09):
            F.box(f'side_rail{x}{y}', 0.04, 0.06, 0.45, (x, y, -0.135), M.sage, bevel=0.005, bevel_segments=1)
    for y in (0.12, TOP - 0.09):
        F.box(f'front_rail{y}', 1.62, 0.06, 0.04, (0, y, 0.12), M.sage, bevel=0.005, bevel_segments=1)
    for k in range(3):                                      # the worktop's planks, a hair apart
        F.box(f'plank{k}', 1.76, 0.05, 0.198, (0, TOP - 0.025, -0.331 + k * 0.2), M.bench, bevel=0.006,
              bevel_segments=1)
    for k in range(4):
        F.box(f'slat{k}', 1.7, 0.025, 0.11, (0, 0.2, -0.37 + k * 0.125), M.bench, bevel=0.005, bevel_segments=1)
    # the back board: upright boards, a groove between each, extruded upright from a grooved outline
    n, w = 12, 1.76
    pts = [(-w / 2, WALL)]
    for k in range(n):
        x0 = -w / 2 + k * w / n
        pts += [(x0 + 0.005, WALL + 0.04), (x0 + w / n - 0.005, WALL + 0.04), (x0 + w / n, WALL + 0.032)]
    pts[-1] = (w / 2, WALL)
    F.extrude('back_board', pts[::-1], 0.86, (0, TOP + 0.43, 0), M.sage, plane='top')
    F.box('peg_rail', 1.7, 0.05, 0.025, (0, TOP + 0.36, WALL + 0.052), M.bench, bevel=0.006, bevel_segments=1)
    for x in (-0.75, -0.6, 0.72):
        F.cyl(f'peg{x}', 0.008, 0.05, (x, TOP + 0.36, WALL + 0.085), M.cedar_dark, rot=(90, 0, 0), verts=6)
    F.box('top_shelf', 1.76, 0.03, 0.18, (0, TOP + 0.62, -0.33), M.bench, bevel=0.006, bevel_segments=1)
    bracket = G.smooth_outline([(0.0, 0.0), (0.0, 0.12), (0.15, 0.12), (0.15, 0.1), (0.05, 0.07), (0.02, 0.0)], passes=2)
    for x in (-0.6, 0.0, 0.6):
        F.extrude(f'bracket{x}', [(WALL + 0.04 + u, TOP + 0.485 + v) for u, v in bracket], 0.025, (x, 0, 0), M.sage,
                  plane='side')


def _hand_tools(F):
    """A trowel and a hand fork hanging from the peg rail, a reel of twine on its peg."""
    z = WALL + 0.07
    for x, kind in ((-0.75, 'trowel'), (-0.6, 'fork')):
        F.sweep(f'{kind}_handle', [(x, TOP + 0.35, z), (x, TOP + 0.2, z)], 0.015, M.cedar_dark, verts=8,
                scale=[0.85, 1.0])
        F.sweep(f'{kind}_ferrule', [(x, TOP + 0.2, z), (x, TOP + 0.16, z)], 0.012, M.steel, verts=8, scale=[1.0, 0.6])
        if kind == 'trowel':
            blade = G.smooth_outline([(-0.035, 0.16), (0.035, 0.16), (0.03, 0.06), (0.0, 0.0), (-0.03, 0.06)], passes=1)
            F.extrude('trowel_blade', [(x + u, TOP + v) for u, v in blade], 0.004, (0, 0, z), M.steel, bevel=0.0015,
                      bevel_segments=1)
        else:
            for dx in (-0.022, 0.0, 0.022):
                F.sweep(f'fork_tine{dx}', [(x, TOP + 0.165, z), (x + dx, TOP + 0.12, z), (x + dx * 1.1, TOP + 0.05, z + 0.012)],
                        0.0035, M.steel, verts=4, resolution=2)
    G.revolve('twine', [(0.012, -0.05), (0.03, -0.05), (0.03, -0.04), (0.024, -0.035), (0.024, 0.035), (0.03, 0.04),
                        (0.03, 0.05), (0.012, 0.05)], at=F.at(0.72, TOP + 0.36, WALL + 0.11), material=M.cedar_dark,
              segments=14, rot=(90, 0, 0))
    G.revolve('twine_wound', [(0.024, -0.034), (0.046, -0.03), (0.05, 0.0), (0.046, 0.03), (0.024, 0.034)],
              at=F.at(0.72, TOP + 0.36, WALL + 0.11), material=M.green_dark, segments=14, rot=(90, 0, 0))


def _watering_can(F, x, y, z):
    """A galvanised can: an oval body, a long spout with its rose, an arching handle over the top."""
    bx, by, bz = F.at(x, y, z)
    G.loft('can_body', [G.ring(0.075 * k, 0.06 * k, bz + h, 16, at=(bx, by)) for h, k in
                        ((0.0, 0.95), (0.01, 1.0), (0.15, 1.0), (0.17, 0.92), (0.18, 0.6), (0.185, 0.0))], material=M.can)
    F.sweep('can_spout', [(x + 0.06, y + 0.03, z), (x + 0.14, y + 0.1, z), (x + 0.2, y + 0.19, z)], 0.012, M.can, verts=8,
            scale=[1.3, 0.8, 0.7])
    G.revolve('can_rose', [(0, 0), (0.012, 0), (0.03, 0.03), (0.03, 0.036), (0, 0.036)], at=F.at(x + 0.2, y + 0.19, z),
              material=M.can, segments=10, rot=(0, -50, 0))
    F.sweep('can_handle', G.kit.fillet([(x - 0.06, y + 0.17, z), (x - 0.03, y + 0.26, z), (x + 0.05, y + 0.26, z),
                                        (x + 0.07, y + 0.18, z)], 0.03), None, M.can,
            profile=G.rounded_rect(0.012, 0.02, 0.004, corner=1), up=(0, 0, 1))


def pottingBench():
    F = G.GameFrame()
    with G.new_parts() as still:
        _bench_frame(F)
        for i, x in enumerate((-0.6, -0.3, 0.0, 0.3, 0.6)):          # pots of herbs on the top shelf
            sy = _pot(F, f'herb_pot{i}', x, TOP + 0.635, -0.32, d=0.13 - (i % 2) * 0.02, h=0.11)
            _rosette(F, f'herb{i}_', x, sy, -0.32, 8, 0.19 - 0.02 * (i % 2), 0.05, ((M.leaf, M.leaf_light), (M.leaf_pale,),
                     (M.leaf_light, M.leaf_dark))[i % 3], tilt=(68, 82), droop=0.15, shape='lance' if i % 2 else 'oval',
                     seed=20 + i)
        _hand_tools(F)
        # on the worktop: a seedling tray, a flowering pot, the watering can, the empty pot for the seedling
        F.box('tray', 0.5, 0.05, 0.3, (-0.15, TOP + 0.025, -0.14), M.dark, bevel=0.006, bevel_segments=1)
        for i in range(4):
            for j in range(2):
                cx, cz = -0.33 + i * 0.12, -0.2 + j * 0.13
                F.box(f'cell{i}{j}', 0.105, 0.012, 0.115, (cx, TOP + 0.046, cz), M.soil)
                for s in (-1, 1):
                    leaf(f'sprout{i}{j}{s}', F.at(cx, TOP + 0.05, cz), 90 * s + 30 * j, 35, 0.045, 0.03,
                         (M.leaf_pale, M.leaf_light)[(i + j) % 2], droop=0.2, rows=2, ring=3)
        sy = _pot(F, 'flower_pot', 0.25, TOP, -0.15, d=0.2, h=0.18)
        _rosette(F, 'flower_leaf', 0.25, sy, -0.15, 9, 0.14, 0.07, (M.leaf_light, M.leaf), tilt=(30, 60), droop=0.3, seed=21)
        for k, m in enumerate(('pink', 'yellow', 'pink', 'lilac', 'yellow', 'lilac')):
            a = k * 60 + 15
            fx, fz = 0.25 + 0.075 * G.cosd(a), -0.15 + 0.075 * G.sind(a)
            F.sweep(f'flower_stem{k}', [(0.25 + 0.02 * G.cosd(a), sy, -0.15 + 0.02 * G.sind(a)), (fx, sy + 0.17 + 0.03 * (k % 2), fz)],
                    0.003, M.stem, verts=3)
            _flower(F, f'flower{k}', fx, sy + 0.17 + 0.03 * (k % 2), fz, 0.035, _m(m), centre=M.yellow if m != 'yellow' else M.carrot,
                    tilt=20, yaw=-a + 90)
        _watering_can(F, 0.62, TOP, -0.1)
        _pot(F, 'seedling_pot', SEEDLING[0], TOP, SEEDLING[1], d=0.14, h=0.12)
        # below: sacks of compost, their tops rolled over, and a stack of pots
        for i in range(3):
            sack = F.soft(f'sack{i}', 0.32, 0.3, 0.26, (-0.55 + i * 0.36, 0.362, -0.16), (M.sack, M.seed, M.sack)[i],
                          rot=(0, 0, (i - 1) * 12), round=0.06, puff=(0.02, 0.0, 0.03), flat=2, round_segments=1)
            G.jitter(sack, 0.01, size=0.12, seed=i * 1.3)
            F.sweep(f'sack_roll{i}', [(-0.55 + i * 0.36 - 0.15, 0.5, -0.16), (-0.55 + i * 0.36 + 0.15, 0.5, -0.16)], 0.025,
                    (M.sack, M.seed, M.sack)[i], verts=6)
        for k, (yy, d) in enumerate(((0.2125, 0.24), (0.3, 0.23), (0.385, 0.22))):
            _pot(F, f'stack_pot{k}', 0.55, yy, -0.12, d=d, h=0.15, soil=False)
    G.join(still, 'body', origin=(0, 0, 0))
    # The seedling: a stem, two seed leaves and a tuft of true leaves, origin on the pot's soil: `seedling` grows it
    # up. Plain satin: a foliage part would grow as well since the renderer's fixes (see the notes).
    sx, sz = SEEDLING
    sy = TOP + 0.106
    with G.new_parts() as seedling:
        F.sweep('seedling_stem', [(sx, sy, sz), (sx + 0.005, sy + 0.08, sz), (sx + 0.01, sy + 0.15, sz)], 0.005, M.green,
                verts=5, resolution=2)
        for s in (-1, 1):
            leaf(f'seed_leaf{s}', F.at(sx + 0.006, sy + 0.1, sz), 90 * s, 25, 0.07, 0.035, M.green_light, droop=0.2,
                 rows=3, ring=3)
        for k in range(4):
            leaf(f'true_leaf{k}', F.at(sx + 0.01, sy + 0.15, sz), 45 + k * 90, 55, 0.05, 0.03, M.green, shape='lance',
                 droop=0.15, rows=2, ring=3)
    seedling = G.join(seedling, 'seedling', origin=F.at(sx, sy, sz))
    seedling.scale[2] = 0.001
    # The bouquet a skilled gardener ties: stems lying along the worktop's front, flower heads, a twine wrap.
    # Its origin is inside the worktop, flattened out of sight: `bouquet` grows it while it is tied.
    bx, bz = BOUQUET
    with G.new_parts() as bouquet:
        for k, dz in enumerate((-0.025, -0.008, 0.008, 0.025)):
            F.sweep(f'bouquet_stem{k}', [(bx - 0.15, TOP + 0.008, bz + dz * 0.3), (bx + 0.1, TOP + 0.012 + 0.005 * (k % 2), bz + dz)],
                    0.004, M.green, verts=4)
        for k, (dx, dy, dz, m) in enumerate(((0.12, 0.03, -0.03, 'pink'), (0.135, 0.045, 0.005, 'yellow'),
                                             (0.115, 0.032, 0.035, 'lilac'), (0.1, 0.05, -0.005, 'pink'))):
            _flower(F, f'bouquet_flower{k}', bx + dx, TOP + dy, bz + dz, 0.03, _m(m), centre=M.yellow, petals=6, tilt=70,
                    yaw=270)
        leaf('bouquet_leaf', F.at(bx + 0.05, TOP + 0.02, bz), 280, 10, 0.08, 0.035, M.green_light, rows=3, ring=3)
        G.torus('bouquet_twine', 0.016, 0.004, at=F.at(bx - 0.02, TOP + 0.017, bz), material=M.green_dark, rot=(0, 90, 0),
                major_segments=8, minor_segments=3)
    bouquet = G.join(bouquet, 'bouquet', origin=F.at(bx, TOP - 0.01, bz))
    bouquet.scale[2] = 0.001
    objs = G.settle(centre=False)
    G.grow(seedling, 'seedling', seconds=1.0)
    G.grow(bouquet, 'bouquet', seconds=0.8)
    return objs


# --- the bird feeder post (1x1): a cedar post, a feeder house on a tray, a seed tube, flowers round its foot -----
PZ = -0.05      # the post's depth


def _feeder_house(F):
    """The tray on braces, its lip mitred round; the house on it: gable walls cut with a round entry, a pitched roof
    of two boards with a ridge cap; seed heaped on the tray."""
    F.box('tray', 0.36, 0.03, 0.3, (0, 1.5, PZ), M.cedar, bevel=0.006, bevel_segments=1)
    G.kit.moulding(F, 'tray_lip', [(-0.18, PZ - 0.15), (0.18, PZ - 0.15), (0.18, PZ + 0.15), (-0.18, PZ + 0.15)],
                   _sq_profile(0.018, 0.03), M.cedar, at=1.515, plane='top')
    for s in (-1, 1):                                       # braces under the tray
        F.sweep(f'brace{s}', [(0.0, 1.32, PZ + s * 0.03), (0.0, 1.48, PZ + s * 0.13)], None, M.cedar_dark, profile=_sq(0.03),
                up=(1, 0, 0))
    seed = F.soft('seed', 0.3, 0.02, 0.22, (0, 1.524, PZ + 0.03), M.seed, round=0.008, puff=(0, 0.008, 0), flat=3,
                  round_segments=1)
    G.jitter(seed, 0.004, size=0.03, seed=1.0)
    gable = [(-0.12, 0.0), (0.12, 0.0), (0.12, 0.17), (0.0, 0.24), (-0.12, 0.17)]
    hole = [(0.03 * math.cos(math.tau * k / 12), 0.11 + 0.03 * math.sin(math.tau * k / 12)) for k in range(12)]
    for z, holes in ((PZ + 0.08, [hole]), (PZ - 0.08, [])):
        F.extrude(f'gable{z}', gable, 0.02, (0, 1.515, z), M.house, holes=holes, bevel=0.003, bevel_segments=1)
    for s in (-1, 1):
        F.box(f'house_side{s}', 0.02, 0.17, 0.14, (s * 0.11, 1.6, PZ), M.house)
    F.box('house_inside', 0.2, 0.15, 0.004, (0, 1.6, PZ + 0.065), M.dark)       # seen through the entry
    F.sweep('perch_dowel', [(0, 1.585, PZ + 0.09), (0, 1.585, PZ + 0.14)], 0.005, M.cedar_dark, verts=6)
    for s in (-1, 1):                                       # the roof's two boards, then its ridge cap
        F.box(f'roof{s}', 0.17, 0.022, 0.29, (s * 0.075, 1.712, PZ), M.roof, rot=(0, -s * 30, 0), bevel=0.005,
              bevel_segments=1)
    F.sweep('ridge', [(0, 1.77, PZ - 0.155), (0, 1.77, PZ + 0.155)], None, M.cedar_dark,
            profile=[(0.0, 0.013), (0.022, -0.006), (-0.022, -0.006)], up=(0, 0, 1))


def _sq_profile(w, h):
    """A moulding's profile (inset, up): a square-edged lip `w` wide and `h` high, its top edge eased."""
    return [(0, 0), (w, 0), (w, h - 0.004), (w - 0.004, h), (0.004, h), (0, h - 0.004)]


def _seed_tube(F, x):
    """A seed tube hanging from the post's arm: a clear body full of seed, a domed lid, a dish with two feeding ports
    and a perch through them."""
    F.sweep('arm', [(0.04, 1.3, PZ), (x - 0.04, 1.31, PZ), (x, 1.33, PZ)], None, M.cedar_dark,
            profile=_sq(0.03, 0.035), up=(0, 0, 1), resolution=2)
    F.sweep('hanger', G.kit.fillet([(x, 1.33, PZ), (x, 1.3, PZ), (x - 0.01, 1.27, PZ)], 0.01), 0.003, M.steel, verts=4)
    G.revolve('seed_tube', [(0, 0), (0.04, 0), (0.04, 0.24), (0, 0.24)], at=F.at(x, 1.01, PZ), material=M.tube,
              segments=14)
    G.revolve('tube_lid', [(0, 0), (0.046, 0), (0.046, 0.012), (0.03, 0.03), (0.008, 0.034), (0, 0.034)],
              at=F.at(x, 1.245, PZ), material=M.roof, segments=14)
    F.cyl('tube_seed', 0.036, 0.2, (x, 1.11, PZ), M.seed, verts=12)
    G.revolve('tube_dish', [(0, -0.01), (0.05, -0.01), (0.062, 0.012), (0.056, 0.016), (0.042, 0.004), (0, 0.004)],
              at=F.at(x, 0.995, PZ), material=M.roof, segments=14)
    for s in (-1, 1):
        F.cyl(f'port{s}', 0.014, 0.012, (x + s * 0.042, 1.04, PZ), M.roof, rot=(0, 90, 0), verts=10)
    F.sweep('perch', [(x - 0.09, 1.02, PZ), (x + 0.09, 1.02, PZ)], 0.0045, M.steel, verts=6)


def birdFeeder():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('post', 0.08, 1.49, 0.08, (0, 0.745, PZ), M.cedar_dark, bevel=0.01, bevel_segments=1)
        F.box('post_socket', 0.11, 0.14, 0.11, (0, 0.07, PZ), M.steel, bevel=0.006, bevel_segments=1)
        _feeder_house(F)
        _seed_tube(F, 0.25)
        G.kit.songbird(F, 'goldfinch', 0.3, 1.025, PZ, M.bird_yellow, M.bird_yellow, M.beak, M.eye, turn=60, size=0.85)
        # a round bed at its foot, edged with stone, a ring of flowering clumps in it
        ring = [(0.33 * G.cosd(a), PZ + 0.33 * G.sind(a)) for a in range(0, 360, 15)]
        G.kit.moulding(F, 'bed_edge', ring, [(-0.015, 0.0), (0.02, 0.0), (0.02, 0.025), (0.012, 0.04), (-0.007, 0.04),
                                             (-0.015, 0.025)], M.pebble, at=0.0, plane='top')
        F.cyl('bed_soil', 0.32, 0.024, (0, 0.012, PZ), M.soil, verts=24)
        for i in range(8):
            a = i * 45 + 22.5
            x, z = G.cosd(a) * 0.22, PZ + G.sind(a) * 0.22
            _rosette(F, f'clump{i}_', x, 0.02, z, 10, 0.09, 0.06, (M.leaf, M.leaf_light), tilt=(32, 66), droop=0.3,
                     seed=30 + i)
            for k in range(3):
                fx, fz = x + 0.035 * G.cosd(a + 120 * k), z + 0.035 * G.sind(a + 120 * k)
                fy = 0.14 + 0.03 * (k % 2)
                F.sweep(f'clump_stem{i}{k}', [(x, 0.02, z), (fx, fy, fz)], 0.003, M.stem, verts=3)
                _flower(F, f'clump_flower{i}{k}', fx, fy, fz, 0.034, _m(('pink', 'yellow', 'lilac', 'white')[(i + k) % 4]),
                        centre=M.yellow if (i + k) % 4 != 1 else M.carrot, tilt=25, yaw=-a + 90)
    G.join(still, 'body', origin=(0, 0, 0))
    # The bluebird that comes to the tray, built at full size and scaled to nothing: `bird` brings it in.
    with G.new_parts() as bird:
        G.kit.songbird(F, 'bluebird', -0.1, 1.535, 0.1, M.bird_blue, M.bird_breast, M.beak, M.eye, turn=-20, head=20)
    bird = G.join(bird, 'bird', origin=F.at(-0.1, 1.535, 0.1))
    bird.scale = (0.001, 0.001, 0.001)
    objs = G.settle(centre=False)
    G.squash(bird, 'bird', 1000, seconds=0.5)
    return objs


# --- the garden hammock (3x1): a canvas sling 2 m long hung from a curved cedar stand, rocking for the resident
# (2026-10-21: 2x1, its sling 1.44 m, grown-ups lying in it with the knees up: M-packs-06, Q-07) ------------------
HOOK = (1.38, 1.1)     # the hooks' (x, height): the sling swings about the line between them
LOW, END, XS = 0.49, 0.9, 1.0      # the sling's low point, its ends' height and its ends' x
PILLOW = 0.76          # the pillow's x (towards the left end): `furniture.json` `head`


def _sag(x):
    """The sling's height at x: a parabola from LOW in the middle to END at the spreader bars."""
    u = x / XS
    return LOW + (END - LOW) * u * u


def gardenHammock():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.sweep('beam', [(-1.35, 0.05, 0), (1.35, 0.05, 0)], None, M.cedar_wood, profile=G.rounded_rect(0.08, 0.1, 0.015, corner=2),
                up=(0, 0, 1))
        for s in (-1, 1):
            F.box(f'foot{s}', 0.1, 0.05, 0.62, (s * 1.16, 0.035, 0.0), M.cedar_wood_dark, bevel=0.012, bevel_segments=1)
            for z in (-0.3, 0.3):
                F.box(f'foot_pad{s}{z}', 0.1, 0.012, 0.06, (s * 1.16, 0.006, z), M.cedar_wood_dark)
            # the laminated arm, curving up in one piece from the beam's end to its hook
            F.sweep(f'arm{s}', [(s * 1.08, 0.04, 0), (s * 1.24, 0.4, 0), (s * 1.35, 0.78, 0), (s * 1.41, 1.18, 0)], None,
                    M.cedar_wood, profile=G.rounded_rect(0.07, 0.08, 0.02, corner=2), up=(0, 0, 1), resolution=4,
                    scale=[1.15, 1.0, 0.95, 0.85])
            # (the hook's eye in the stand's dark wood, as the old part's: steel would be a draw of its own)
            F.sweep(f'eye_bolt{s}', [(s * 1.4, 1.17, 0), (s * HOOK[0], HOOK[1] + 0.025, 0)], 0.006, M.cedar_wood_dark, verts=6)
            G.torus(f'hook{s}', 0.025, 0.006, at=F.at(s * HOOK[0], HOOK[1], 0), material=M.cedar_wood_dark, rot=(90, 0, 0),
                    major_segments=10, minor_segments=4)
    G.join(still, 'body', origin=(0, 0, 0))
    # The sling: striped canvas sagging between two spreader bars in one sweep a stripe, hemmed along its sides, roped
    # to a ring at each end and on to the hooks, a pillow at one end.
    with G.new_parts() as sling:
        width = 0.62
        stripes = ('canvas', 'stripe', 'canvas', 'canvas', 'stripe', 'canvas')
        zw = width / len(stripes)
        xs = [-XS + 2 * XS * i / 16 for i in range(17)]
        for j, m in enumerate(stripes):
            zc = -width / 2 + zw * (j + 0.5)
            F.sweep(f'sling{j}', [(x, _sag(x), zc) for x in xs], None, _m(m), profile=_sq(0.012, zw + 0.002), up=(0, 0, 1))
        for z in (-width / 2, width / 2):
            F.sweep(f'hem{z}', [(x, _sag(x) + 0.003, z) for x in xs], 0.008, M.canvas, verts=5)
        for s in (-1, 1):
            F.sweep(f'spreader{s}', [(s * XS, END, -0.34), (s * XS, END, 0.34)], 0.02, M.cedar_wood_dark, verts=8)
            ring = (s * (HOOK[0] - 0.12), HOOK[1] - 0.08, 0)
            G.torus(f'rope_ring{s}', 0.025, 0.007, at=F.at(*ring), material=M.rope, rot=(90, 0, 0), major_segments=10,
                    minor_segments=4)
            for k in range(7):
                z = -0.3 + k * 0.1
                F.sweep(f'cord{s}{k}', [(s * XS, END + 0.01, z), (ring[0] - s * 0.02, ring[1] - 0.01, z * 0.08)], 0.004, M.rope,
                        verts=4)
            F.sweep(f'rope{s}', [ring, (s * HOOK[0], HOOK[1] - 0.02, 0)], 0.008, M.rope, verts=5)
        y = _sag(-PILLOW) + 0.0125                       # the sling's top and slope under the pillow
        slope = math.degrees(math.atan(2 * (END - LOW) * PILLOW / XS ** 2))
        F.soft('pillow', 0.36, 0.11, 0.3, (-PILLOW, y + 0.045, 0.0), M.pillow, rot=(0, slope, 0), round=0.045, puff=0.02,
               flat=2)
    sling = G.hinged(G.join(sling, 'sling'), F.at(0, HOOK[1], 0))
    objs = G.settle(centre=False)
    G.rock(sling, 'sway', 8, seconds=2.4, axis='X')
    return objs


# --- the cedar outdoor shower (1x1): slatted cedar walls on two sides, a duckboard, a copper rain head ---------
HEAD = (0.15, 2.0, -0.08)   # the rain head's underside


def _towel(F, name, x, top, z0, z1, mat, hang=0.35):
    """A towel folded over a bar at height `top` (running along depth from z0 to z1 at x): hanging down both sides."""
    t = 0.012
    pts = [(x - 0.03, top - hang, 0), (x - 0.028, top - 0.04, 0), (x - 0.018, top + 0.004, 0), (x, top + 0.014, 0),
           (x + 0.018, top + 0.004, 0), (x + 0.028, top - 0.04, 0), (x + 0.03, top - hang * 0.8, 0)]
    obj = F.sweep(name, [(px, py, (z0 + z1) / 2) for px, py, _ in pts], None, mat, profile=G.rounded_rect(t, z1 - z0, t / 2, corner=1),
                  up=(1, 0, 0), resolution=2)
    G.jitter(obj, 0.003, size=0.1, seed=len(name), up=0.3)
    return obj


def cedarShower():
    F = G.GameFrame()
    with G.new_parts() as still:
        for i in range(12):                                         # back wall slats
            F.box(f'slat_b{i}', 0.065, 1.9, 0.03, (-0.41 + i * 0.074, 1.0, WALL + 0.015),
                  M.cedar_wood if i % 2 else M.cedar_wood_dark, bevel=0.006, bevel_segments=1)
        for i in range(11):                                         # left wall slats
            F.box(f'slat_l{i}', 0.03, 1.9, 0.065, (-0.43, 1.0, -0.35 + i * 0.077),
                  M.cedar_wood if i % 2 else M.cedar_wood_dark, bevel=0.006, bevel_segments=1)
        for y in (0.06, 1.97):                                      # the rails behind the slats, a post at the open corner
            F.box(f'rail_b{y}', 0.9, 0.05, 0.04, (0.0, y, WALL + 0.045), M.cedar_wood_dark, bevel=0.008, bevel_segments=1)
            F.box(f'rail_l{y}', 0.04, 0.05, 0.86, (-0.4, y, 0.0), M.cedar_wood_dark, bevel=0.008, bevel_segments=1)
        F.box('corner_post', 0.06, 1.98, 0.06, (-0.42, 0.99, 0.42), M.cedar_wood_dark, bevel=0.008, bevel_segments=1)
        for i in range(7):                                          # the duckboard on two bearers
            F.box(f'duck{i}', 0.78, 0.03, 0.085, (0.02, 0.05, -0.33 + i * 0.11), M.cedar_wood, bevel=0.008, bevel_segments=1)
        for x in (-0.3, 0.32):
            F.box(f'bearer{x}', 0.06, 0.035, 0.74, (x, 0.0175, 0.0), M.cedar_wood_dark, bevel=0.006, bevel_segments=1)
        # the copper pipe in one piece: up from its floor flange, bent forward under the top and down to the rain head
        x, top = HEAD[0], 2.064
        G.kit.tube(F, 'pipe', G.kit.fillet([(x, 0.05, -0.395), (x, top, -0.395), (x, top, HEAD[2]), (x, HEAD[1] + 0.03, HEAD[2])],
                                           0.05, steps=4), 0.018, M.copper, verts=10, round=1)
        for y in (0.6, 1.6):                                        # clips holding it to the back wall
            F.box(f'pipe_clip{y}', 0.05, 0.025, 0.035, (x, y, -0.405), M.copper, bevel=0.004, bevel_segments=1)
        G.revolve('flange', [(0, 0), (0.034, 0), (0.034, 0.008), (0.022, 0.02), (0, 0.02)], at=F.at(x, 0.065, -0.395),
                  material=M.copper, segments=12)
        G.revolve('rain_head', [(0, 0.03), (0.035, 0.03), (0.06, 0.02), (0.15, 0.006), (0.152, 0.0), (0, 0.0)],
                  at=F.at(x, HEAD[1] + 0.002, HEAD[2]), material=M.copper, segments=24)
        F.cyl('rain_face', 0.135, 0.004, (x, HEAD[1], HEAD[2]), M.steel, verts=24)
        G.revolve('valve', [(0, 0), (0.045, 0), (0.045, 0.006), (0.028, 0.02), (0.022, 0.04), (0, 0.042)],
                  at=F.at(x, 1.1, -0.395), material=M.copper, segments=14, rot=(90, 0, 0))
        F.sweep('valve_lever', [(x, 1.1, -0.36), (x + 0.04, 1.1, -0.35), (x + 0.1, 1.09, -0.345)], None, M.copper,
                profile=G.rounded_rect(0.01, 0.016, 0.004, corner=1), up=(0, 1, 0), resolution=2)
        # a towel over a bar on the left wall, a soap shelf, a potted plant and pebbles
        F.sweep('towel_bar', [(-0.385, 1.5, 0.13), (-0.37, 1.5, 0.13), (-0.37, 1.5, 0.37), (-0.385, 1.5, 0.37)], 0.009,
                M.copper, verts=8)
        _towel(F, 'towel', -0.37, 1.5, 0.15, 0.35, M.towel)
        F.extrude('soap_shelf', G.smooth_outline([(-0.415, -0.25), (-0.3, -0.25), (-0.3, -0.15), (-0.415, -0.15)], passes=1),
                  0.025, (0, 1.0, 0), M.cedar_wood_dark, plane='top', bevel=0.004, bevel_segments=1)
        F.soft('soap', 0.08, 0.03, 0.05, (-0.36, 1.027, -0.2), M.soap, round=0.012, round_segments=1)
        sy = _pot(F, 'plant_pot', 0.3, 0.0, 0.3, d=0.2, h=0.18)
        _rosette(F, 'plant', 0.3, sy, 0.3, 9, 0.2, 0.08, (M.green_light, M.green), tilt=(55, 78), droop=0.35, shape='lance',
                 rows=4, ring=3, seed=3)
        for i, (px, pz) in enumerate(((0.12, 0.4), (0.38, 0.06), (-0.2, 0.4), (0.38, -0.14))):
            G.boulder(f'pebble{i}', 0.04, at=F.at(px, 0.0, pz), material=(M.pebble, M.pebble_b)[i % 2], seed=i,
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
            G.rod(f'stream{k}', F.at(px, y, pz), F.at(px, y - h, pz), 0.005, material=M.water, verts=4)
    rain = G.join(rain, 'water', origin=F.at(x, y, z))
    rain.scale[2] = 0.001
    objs = G.settle(centre=False)
    G.grow(rain, 'water_on', seconds=0.6)
    return objs


# --- the moss terrarium (1x1): a little forest under a glass cloche, on a walnut pedestal ----------------------
TZ = -0.1      # the pedestal's depth
TOPY = 0.78    # the soil's top


def _fir(F, name, x, y, z, h, r, material):
    """A little fir: a trunk under tiers of drooping branches, turned from a zigzag profile."""
    tiers = 4
    prof = [(0, 0)]
    for k in range(tiers):
        y0 = 0.15 * h + k * 0.85 * h / tiers
        rr = r * (1 - k / tiers)
        prof += [(rr * 0.35, y0 + 0.04 * h), (rr, y0)]
    prof += [(0, h)]
    G.revolve(name, prof, at=F.at(x, y, z), material=material, segments=8)
    F.cyl(f'{name}_trunk', 0.006, 0.15 * h + 0.01, (x, y + 0.075 * h, z), M.cedar_dark, verts=5)


def mossTerrarium():
    F = G.GameFrame()
    with G.new_parts() as still:
        G.revolve('pedestal', [(0, 0), (0.17, 0), (0.17, 0.012), (0.155, 0.03), (0.09, 0.05), (0.07, 0.075), (0.08, 0.09),
                               (0.05, 0.12), (0.042, 0.3), (0.048, 0.42), (0.04, 0.54), (0.06, 0.56), (0.09, 0.6),
                               (0.15, 0.65), (0.17, 0.67), (0.17, 0.68), (0, 0.68)], at=F.at(0, 0, TZ), material=M.walnut,
                  segments=20)
        G.revolve('plate', [(0, 0), (0.19, 0), (0.2, 0.008), (0.2, 0.03), (0.192, 0.04), (0.185, 0.04), (0.185, 0.03),
                            (0, 0.03)], at=F.at(0, 0.68, TZ), material=M.walnut, segments=28)
        mound = G.revolve('moss_mound', [(0, 0.71), (0.18, 0.71), (0.18, 0.74), (0.15, 0.765), (0.08, 0.79), (0, 0.8)],
                          at=F.at(0, 0, TZ), material=M.moss, segments=20)
        G.jitter(mound, 0.008, size=0.05, seed=1.3)
        G.boulder('rock', 0.05, at=F.at(0.08, 0.77, TZ + 0.05), material=M.pebble, seed=2, subdivisions=1, scale=(1.2, 1, 0.8))
        _fir(F, 'fir', -0.07, 0.77, TZ - 0.02, 0.24, 0.055, M.leaf_dark)
        _fir(F, 'fir_b', 0.06, 0.765, TZ - 0.07, 0.16, 0.04, M.leaf)
        for k in range(5):                                         # a little fern at the front
            leaf(f'fern{k}', F.at(-0.06, 0.78, TZ + 0.08), 150 + k * 35, 40, 0.09, 0.035, M.leaf_light, shape='lance',
                 droop=0.4, notch=(2, 0.35), rows=6, ring=3)
        for k, (x, z, h) in enumerate(((-0.02, TZ + 0.1, 0.035), (0.01, TZ + 0.12, 0.022))):    # toadstools
            F.cyl(f'toadstool_stem{k}', 0.005, h, (x, 0.77 + h / 2, z), M.white, verts=6)
            G.revolve(f'toadstool{k}', [(0, 0), (0.022, 0), (0.024, 0.006), (0.016, 0.016), (0, 0.02)],
                      at=F.at(x, 0.77 + h - 0.002, z), material=M.toadstool, segments=10)
        # the mister on the floor beside it, and a round braided rug
        bx, by, bz = F.at(0.24, 0.012, 0.15)
        G.loft('mister', [G.ring(0.03 * k, 0.03 * k, bz + h, 12, at=(bx, by)) for h, k in
                          ((0.0, 0.9), (0.01, 1.0), (0.11, 1.0), (0.14, 0.45), (0.165, 0.42))], material=M.mister)
        F.box('mister_head', 0.035, 0.035, 0.05, (0.24, 0.195, 0.16), M.white, bevel=0.008, bevel_segments=1)
        F.sweep('mister_trigger', [(0.24, 0.19, 0.18), (0.24, 0.15, 0.19)], None, M.white, profile=_sq(0.012, 0.008),
                up=(1, 0, 0))
        braid = [(0, 0), (0.21, 0), (0.21, 0.006)]               # rings of braid, from the edge in
        for k in range(4, -1, -1):
            braid += [(0.21 * (k + 0.5) / 5, 0.012), (0.21 * k / 5 + 0.002, 0.008)]
        G.revolve('rug', braid[:-1] + [(0, 0.012)], at=F.at(0, 0.0, 0.24), material=M.rug, segments=28, scale=(1, 0.75, 1))
    G.join(still, 'body', origin=(0, 0, 0))
    # The glass cloche and its knob, resting on the plate: `lid_open` lifts it.
    with G.new_parts() as lid:
        G.revolve('cloche', [(0.185, 0), (0.192, 0.006), (0.192, 0.015), (0.19, 0.23), (0.16, 0.33), (0.1, 0.39), (0.02, 0.41),
                             (0.02, 0.4), (0.098, 0.384), (0.156, 0.326), (0.182, 0.23), (0.182, 0.01), (0.178, 0)],
                  at=F.at(0, 0.72, TZ), material=M.glass, segments=28)
        G.revolve('cloche_knob', [(0, 0), (0.014, 0), (0.01, 0.012), (0.026, 0.026), (0.022, 0.044), (0, 0.048)],
                  at=F.at(0, 1.126, TZ), material=M.glass, segments=12)
    lid = G.join(lid, 'cloche', origin=F.at(0, 0.72, TZ))
    objs = G.settle(centre=False)
    G.slide(lid, 'lid_open', (0, 0, 0.22), seconds=0.7)
    return objs


# --- the fern glasshouse (2x2): a white-framed glass room, a bench for two, ferns, orchids and palms -----------
GW, GD, HW, HR = 1.86, 1.86, 2.0, 2.45      # width, depth, wall height, ridge height
DOOR = 0.8                                   # the doorway's width, in the middle of the front
VENT = (0.3, 0.55)                           # the roof vent: half its length (depth), its length down the slope


def _fern(F, name, x, y, z, material, spread=0.25, n=7, seed=0):
    """A fern: fronds cut into leaflets, arching out from a pot's soil at (x, y, z)."""
    rnd = random.Random(seed)
    for k in range(n):
        leaf(f'{name}{k}', F.at(x, y, z), k * 360 / n + rnd.uniform(-15, 15), rnd.uniform(35, 60), spread * rnd.uniform(0.85, 1.1),
             spread * 0.32, material, shape='lance', droop=0.45, fold=0.1, notch=(2, 0.35), rows=5, ring=3)


def _palm(F, name, x, y, z, s):
    """A potted palm: a ringed trunk and fronds arching out from its top, each cut into leaflets."""
    F.sweep(f'{name}_trunk', [(x, y, z), (x + s * 0.02, y + 0.25, z), (x, y + 0.5, z)], 0.03, M.cedar_dark, verts=6,
            resolution=2, scale=[1.1, 0.8])
    for k in range(7):
        leaf(f'{name}_frond{k}', F.at(x, y + 0.49, z), k * 360 / 7 + 20, 35 + 12 * (k % 2), 0.3, 0.12, M.green_light,
             shape='lance', droop=0.5, fold=0.15, notch=(2, 0.3), rows=6, ring=3)


def _orchid(F, name, x, y, z, bloom):
    """An orchid: two broad leaves at the pot's rim, an arching spike of blooms tied to a stick."""
    for k in (-1, 1):
        leaf(f'{name}_leaf{k}', F.at(x, y, z), 90 * k + 10, 20, 0.12, 0.05, M.green_dark, droop=0.3, rows=3, ring=3)
    F.sweep(f'{name}_spike', [(x, y, z), (x + 0.01, y + 0.18, z + 0.01), (x + 0.04, y + 0.28, z + 0.03)], 0.004, M.stem,
            verts=4, resolution=2)
    for k in range(3):
        _flower(F, f'{name}_bloom{k}', x + 0.012 + 0.014 * k, y + 0.19 + 0.04 * k, z + 0.025 + 0.004 * k, 0.03, bloom,
                centre=M.yellow, tilt=75, yaw=180)


def _glasshouse_frame(F, hx, hz, slope, run):
    """The white frame: corner posts, eaves, gable rafters, the ridge, glazing bars, the doorway, gutters along the
    eaves."""
    for x in (-hx, hx):
        for z in (-hz, hz):
            F.box(f'post{x}{z}', 0.05, HW, 0.05, (x, HW / 2, z), M.frame)
    for z in (-hz, hz):
        F.box(f'eave{z}', GW, 0.05, 0.05, (0, HW, z), M.frame)
        for s in (-1, 1):
            F.box(f'rafter{z}{s}', run, 0.05, 0.05, (s * GW / 4, (HW + HR) / 2, z), M.frame, rot=(0, -s * slope, 0))
    for x in (-hx, hx):
        F.box(f'eave_side{x}', 0.05, 0.05, GD, (x, HW, 0), M.frame)
        F.sweep(f'gutter{x}', [(x * 1.015, HW + 0.01, -GD / 2), (x * 1.015, HW + 0.01, GD / 2)], None, M.frame,
                profile=[(0.0, 0.02), (-0.02, 0.0), (0.0, -0.02), (0.003, -0.016), (-0.014, 0.0), (0.003, 0.016)], up=(1, 0, 0))
    F.box('ridge', 0.06, 0.06, GD, (0, HR, 0), M.frame)
    for z in (-0.45, 0.0, 0.45):
        for x in (-hx, hx):
            F.box(f'bar{z}{x}', 0.03, HW - 0.15, 0.03, (x, HW / 2 + 0.05, z), M.frame)
        for s in (-1, 1):
            F.box(f'roof_bar{z}{s}', run, 0.03, 0.03, (s * GW / 4, (HW + HR) / 2 + 0.01, z), M.frame, rot=(0, -s * slope, 0))
    for x in (-0.45, 0.0, 0.45):
        F.box(f'bar_back{x}', 0.03, HW - 0.15, 0.03, (x, HW / 2 + 0.05, -hz), M.frame)
    F.box('back_rail', GW, 0.04, 0.04, (0, 0.95, -hz), M.frame)
    for s in (-1, 1):
        F.box(f'door_post{s}', 0.05, HW, 0.05, (s * (DOOR / 2 + 0.025), HW / 2, hz), M.frame)
        F.box(f'front_rail{s}', (GW - DOOR) / 2 - 0.08, 0.04, 0.04, (s * (GW + DOOR) / 4, 0.95, hz), M.frame)
        F.box(f'side_rail{s}', 0.04, 0.04, GD - 0.08, (s * hx, 0.95, 0), M.frame)
    F.box('lintel', DOOR + 0.1, 0.05, 0.05, (0, 1.92, hz), M.frame)


def _glasshouse_glass(F, hx, hz, slope, run):
    """The glass: back and side walls, the front either side of the door, the gable ends, the roof round its vent."""
    vz, vl = VENT
    F.box('glass_back', GW - 0.08, HW - 0.15, 0.012, (0, HW / 2 + 0.05, -hz), M.glass)
    for x in (-hx, hx):
        F.box(f'glass_side{x}', 0.012, HW - 0.15, GD - 0.08, (x, HW / 2 + 0.05, 0), M.glass)
    for s in (-1, 1):
        F.box(f'glass_front{s}', (GW - DOOR) / 2 - 0.08, HW - 0.15, 0.012, (s * (GW + DOOR) / 4, HW / 2 + 0.05, hz), M.glass)
    F.box('glass_transom', DOOR, 0.06, 0.012, (0, 1.97, hz), M.glass)
    for z in (-hz, hz):                                        # the gables: thin triangular slabs
        F.extrude(f'glass_gable{z}', [(-GW / 2 + 0.05, HW + 0.025), (GW / 2 - 0.05, HW + 0.025), (0, HR - 0.02)], 0.012,
                  (0, 0, z), M.glass)
    F.box('glass_roof_l', run, 0.012, GD - 0.04, (-GW / 4, (HW + HR) / 2 + 0.02, 0), M.glass, rot=(0, slope, 0))
    for s in (-1, 1):
        d = (GD - 0.04) / 2 - vz
        F.box(f'glass_roof_r{s}', run, 0.012, d, (GW / 4, (HW + HR) / 2 + 0.02, s * (vz + d / 2)), M.glass, rot=(0, -slope, 0))
    low = run - vl                                             # the part of the slope below the vent
    t = 1 - (low / 2) / run                                    # where its middle is, from the ridge
    F.box('glass_roof_r_low', low, 0.012, 2 * vz, (GW / 2 * t, HR - (HR - HW) * t + 0.02, 0), M.glass, rot=(0, -slope, 0))


def glasshouse():
    F = G.GameFrame()
    hx, hz = GW / 2 - 0.03, GD / 2 - 0.03
    slope = math.degrees(math.atan2(HR - HW, GW / 2))
    run = math.hypot(GW / 2, HR - HW)
    with G.new_parts() as still:
        F.box('base', GW, 0.12, GD, (0, 0.06, 0), M.stone, bevel=0.01, bevel_segments=1)
        F.box('gravel', GW - 0.06, 0.012, GD - 0.06, (0, 0.126, 0), M.gravel)
        for k in range(4):                                         # stepping stones from the door to the bench
            G.boulder(f'step{k}', 0.11, at=F.at(0.05 * (k % 2), 0.124, 0.75 - k * 0.2), material=M.stone, seed=k * 1.7,
                      subdivisions=1, scale=(1.3, 1.0, 0.12))
        _glasshouse_frame(F, hx, hz, slope, run)
        _glasshouse_glass(F, hx, hz, slope, run)
        # the bench for two (seat top 0.45 at depth 0): slats on shaped ends, a slatted back, stuffed cushions
        for k in range(4):
            F.box(f'seat_slat{k}', 1.4, 0.03, 0.095, (0, 0.405, -0.17 + k * 0.1), M.cedar, bevel=0.006, bevel_segments=1)
        for k in range(3):
            F.box(f'back_slat{k}', 1.4, 0.07, 0.025, (0, 0.56 + k * 0.1, -0.25 - k * 0.018), M.cedar, rot=(10, 0, 0),
                  bevel=0.006, bevel_segments=1)
        end = G.smooth_outline([(-0.19, 0.0), (-0.13, 0.0), (-0.1, 0.06), (0.1, 0.06), (0.13, 0.0), (0.19, 0.0), (0.19, 0.39),
                                (-0.17, 0.39), (-0.19, 0.39)], passes=1)
        for x in (-0.64, 0.64):
            F.extrude(f'bench_end{x}', [(-0.02 + u, v) for u, v in end], 0.05, (x, 0, 0), M.cedar_dark, plane='side',
                      bevel=0.006, bevel_segments=1)
            F.sweep(f'bench_back_post{x}', [(x, 0.39, -0.21), (x, 0.82, -0.29)], None, M.cedar_dark, profile=_sq(0.04),
                    up=(0, 1, 0))
        for x in (-0.36, 0.36):
            F.soft(f'cushion{x}', 0.62, 0.03, 0.38, (x, 0.435, -0.01), M.cushion, round=0.014, puff=0.008, flat=2, round_segments=1)
        # the back shelf: orchids and ferns
        F.box('shelf', 1.7, 0.04, 0.3, (0, 0.78, -0.72), M.cedar, bevel=0.006, bevel_segments=1)
        for x in (-0.72, 0.72):
            F.box(f'shelf_end{x}', 0.04, 0.66, 0.26, (x, 0.45, -0.72), M.cedar_dark, bevel=0.006, bevel_segments=1)
        for i, x in enumerate((-0.32, 0.18, 0.68)):
            sy = _pot(F, f'orchid_pot{i}', x, 0.8, -0.66, d=0.12, h=0.1, material=M.porcelain)
            _orchid(F, f'orchid{i}', x, sy, -0.66, _m(('orchid', 'white', 'orchid_b')[i]))
        for k, x in enumerate((-0.6, -0.07, 0.43)):
            sy = _pot(F, f'fern_pot{k}', x, 0.8, -0.7, d=0.16, h=0.12)
            _fern(F, f'fern{k}', x, sy, -0.7, (M.green, M.green_dark, M.green_light)[k], spread=0.24, seed=k)
        # potted palms in the front corners, hanging baskets
        for s in (-1, 1):
            sy = _pot(F, f'palm_pot{s}', s * 0.64, 0.13, 0.6, d=0.3, h=0.26)
            _palm(F, f'palm{s}', s * 0.64, sy, 0.6, s)
        for x in (-0.68, 0.68):
            for k in range(3):
                a = k * 120 + 30
                F.sweep(f'basket_chain{x}{k}', [(x, HW - 0.02, -0.3), (x + 0.1 * G.cosd(a), 1.74, -0.3 + 0.1 * G.sind(a))], 0.003,
                        M.steel, verts=3)
            sy = _pot(F, f'basket{x}', x, 1.6, -0.3, d=0.2, h=0.14)
            _rosette(F, f'basket_leaf{x}', x, sy, -0.3, 7, 0.12, 0.06, (M.green, M.green_light), tilt=(15, 45), droop=0.5,
                     seed=int(x * 10))
            for k in range(4):
                a = k * 90 + 45
                F.sweep(f'trailing{x}{k}', [(x + 0.09 * G.cosd(a), 1.72, -0.3 + 0.09 * G.sind(a)),
                                            (x + 0.13 * G.cosd(a), 1.6, -0.3 + 0.13 * G.sind(a)),
                                            (x + 0.12 * G.cosd(a + 20), 1.42, -0.3 + 0.12 * G.sind(a + 20))], 0.012,
                        M.green_dark, verts=4, resolution=2, scale=[1.0, 0.6])
    G.join(still, 'body', origin=(0, 0, 0))
    # The door: a glass leaf hinged on the left post, swinging out; the roof vent: a glass panel hinged at
    # the ridge, lifting. `door_open` lets the resident in and shuts behind them (and lets them out), `vent_open`
    # airs it while they're in.
    vz, vl = VENT
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
             motions={'uses': {'pickSnack': {'clip': 'harvest', 'over': 1}, 'tendVeggies': {'clip': 'rustle', 'delay': 0.6, 'every': 2.4}},
                      'growth': 'grow'},
             paint=[('cedar', 'Wood'), ('cedar_dark', 'Posts & rails')]),
     G.Model('pottingBench', pottingBench, size=(1.76, 1.785, 0.6), key='model.natureLover.pottingBench',
             centre=(0, -0.13), motions={'uses': {'potSeedlings': 'seedling', 'tieBouquet': 'bouquet'}},
             paint=[('bench', 'Wood'), ('sage', 'Frame')]),
     G.Model('birdFeeder', birdFeeder, size=(0.705, 1.783, 0.679), key='model.natureLover.birdFeeder',
             centre=(0.01, -0.05), motions={'uses': {'birdwatch': 'bird'}},
             paint=[('cedar', 'Wood'), ('roof', 'Roof')]),
     G.Model('gardenHammock', gardenHammock, size=(2.88, 1.19, 0.68), key='model.natureLover.gardenHammock',
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

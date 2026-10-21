# The foodie pack (P6): the windowsill herb garden, the deli pantry, the espresso bar, the cookbook lectern, the
# wood-fired pizza oven and the chef's island. Notes: pack_foodie.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`). A fresh sprig grows in a pot (`sprig`), the pantry's doors swing open (`doors_open`), the coffee runs
# and the steam wand swings out (`brew`), a page of the cookbook turns over (`page`), the oven door lifts aside
# and the peel slides in (`door_off`) as the fire grows (`flames`), and the island's drawer pulls out
# (`drawer_open`).
#
# Run:  cargo art pack_foodie [--only pizzaOven] [--preview]
import math
import os
import random
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402
from gamekit.motion import _clip, last_frame  # noqa: E402

M = G.Palette(
    # the herb garden
    cedar=('#C49A6C', 'wood'),       # the stand and the trough; its main part
    cedar_dark=('#A07A52', 'wood'),  # posts, rails and the trough's inside
    soil=('#5A4030', 'matte'),
    basil=('#5E9E3E', 'satin'),      # herbs are plain satin, so they cost no draw call of their own
    basil_light=('#6FB04A', 'satin'),
    thyme=('#86A06E', 'satin'),
    rosemary=('#4E7C52', 'satin'),
    terracotta=('#C46A43', 'matte'),
    chalk=('#F2EDE2', 'matte'),      # chalk tags, towels
    steel=('#9EA3A8', 'metal'),
    wicker=('#C9A46E', 'satin'),
    # the deli pantry
    cream=('#EDE4D0', 'satin'),      # the pantry's body; its main part
    cream_panel=('#E4D9C1', 'satin'),
    honey=('#A8754A', 'satin'),      # the pantry's shelves, plinth and crown (plain: the doors take the draws)
    back=('#E2D6BC', 'matte'),
    marble=('#F3EFE8', 'gloss'),     # the pantry's ledge, the island's top
    brass=('#B8925A', 'metal'),
    jar_a=('#C98A3A', 'gloss'),
    jar_b=('#9DB86A', 'gloss'),
    jar_c=('#B5653F', 'gloss'),
    cheese=('#D6A64A', 'satin'),
    cheese_light=('#E9C46A', 'satin'),
    bread=('#C8964F', 'matte'),
    wine=('#6E2B33', 'gloss'),
    bottle=('#3E5A3A', 'gloss'),
    salami=('#8E3B2E', 'matte'),
    tomato=('#C9483B', 'gloss'),
    board=('#B08E66', 'satin'),
    # the espresso bar
    teal=('#2F5D62', 'satin'),       # the cart's frame; its main part
    oak_top=('#CDB08A', 'satin'),    # the cart's top and shelves (plain: the stream and the wand take the draws)
    machine=('#EFE6D4', 'gloss'),
    copper=('#C27A4A', 'metal'),
    chrome=('#D0D3D6', 'chrome'),
    black=('#2B2C2E', 'gloss'),
    cup=('#F6F3EA', 'gloss'),
    coffee=('#5A3420', 'gloss'),
    beans=('#5A3A26', 'satin'),
    dark=('#3A3D41', 'satin'),
    sack=('#C9A46E', 'satin'),
    # the cookbook lectern
    walnut=('#7A5234', 'wood'),      # the lectern; its main part
    honey_wood=('#A8754A', 'wood'),  # its ledge and the stool
    page=('#F6F1E4', 'matte'),
    cover=('#B33A32', 'satin'),
    photo=('#E9C46A', 'matte'),
    book_teal=('#4E7A7F', 'satin'),
    book_mustard=('#D6A64A', 'satin'),
    book_sage=('#9DAF8C', 'satin'),
    # the pizza oven
    plaster=('#E8DAC4', 'matte'),    # the dome; its main part
    brick=('#A4614A', 'matte'),
    brick_light=('#B8735A', 'matte'),
    brick_dark=('#6E4535', 'matte'),
    stone=('#C9C1B4', 'satin'),
    worktop=('#E9E3D8', 'gloss'),
    iron=('#3E4146', 'metal'),
    log=('#8B6B4E', 'satin'),        # plain, not bark: the door, the peel and the fire take the draws
    mouth=('#2B2220', 'matte'),
    ember=('#F28C28', 'gloss'),
    flame=('#F28C28', 'gloss'),
    flame_core=('#FFC24A', 'gloss'),
    dough=('#E8C27A', 'matte'),
    mozzarella=('#F4EBD5', 'satin'),
    oil=('#7A8F3A', 'gloss'),
    peel=('#B08E66', 'satin'),
    door_oak=('#CDB08A', 'satin'),
    # the chef's island
    cabinet=('#5F9095', 'satin'),    # the island's cabinets; its main part
    oak=('#CDB08A', 'satin'),        # the stools, plain (the hob's hot ring takes the third draw)
    oak_dark=('#B08E66', 'satin'),
    seat=('#C9A46E', 'satin'),
    hob=('#22262B', 'gloss'),
    ring=('#C9483B', 'gloss'),
    heat=('#D2402A', 'gloss'),       # the hob's ring glowing hot under the pan while it cooks
    carrot=('#E2832E', 'satin'),
    plate=('#F6F3EA', 'gloss'),
)

WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


def _pot(F, x, z, y=0.0, r=0.09, h=0.16, material=None):
    """A terracotta pot standing on `y` with soil in it; returns the soil's top."""
    G.revolve(_name('pot'), [(0, 0), (r * 0.72, 0), (r * 0.88, h * 0.75), (r, h * 0.78), (r, h), (r * 0.9, h),
                             (r * 0.84, h * 0.9), (0, h * 0.9)], at=F.at(x, y, z), material=material or M.terracotta,
              segments=16)
    F.cyl(_name('pot_soil'), r * 0.84, 0.006, (x, y + h * 0.9 + 0.003, z), M.soil, verts=14)
    return y + h * 0.9 + 0.006


def _basil(F, x, z, y, s=1.0):
    for k, (dx, dz, dy, r, m) in enumerate(((0, 0, 0.05, 0.1, M.basil), (0.04, -0.03, 0.11, 0.065, M.basil_light),
                                            (-0.05, 0.02, 0.09, 0.06, M.basil_light), (0.0, 0.05, 0.08, 0.055, M.basil))):
        F.sphere(_name('basil'), r * s, (x + dx * s, y + dy * s, z + dz * s), m, scale=(1, 1, 0.7), segments=10, rings=6)


def _rosemary(F, x, z, y, s=1.0):
    for i in range(7):
        a = i * 137.5
        dx, dz = 0.045 * s * G.cosd(a), 0.045 * s * G.sind(a)
        h = (0.2 + 0.05 * (i % 3)) * s
        G.rod(_name('rosemary'), F.at(x + dx * 0.5, y, z + dz * 0.5), F.at(x + dx * 1.6, y + h, z + dz * 1.6), 0.014 * s,
              material=M.rosemary, verts=6)


def _thyme(F, x, z, y, s=1.0):
    for k in range(6):
        a = k * 60 + 15
        r = 0.045 if k % 2 else 0.055
        F.sphere(_name('thyme'), r * s, (x + 0.06 * s * G.cosd(a), y + 0.035 * s, z + 0.05 * s * G.sind(a)), M.thyme,
                 scale=(1, 1, 0.6), segments=8, rings=5)
    F.sphere(_name('thyme'), 0.06 * s, (x, y + 0.06 * s, z), M.thyme, scale=(1, 1, 0.7), segments=8, rings=5)


# --- the windowsill herb garden (1x1): a slatted cedar trough on a stand, pots on the shelf below ------------
def herbGarden():
    F = G.GameFrame()
    with G.new_parts() as still:
        for x in (-0.38, 0.38):
            for z in (-0.40, -0.06):
                F.box(_name('leg'), 0.045, 0.615, 0.045, (x, 0.3075, z), M.cedar_dark, bevel=0.006)
            F.box(_name('side_rail'), 0.04, 0.03, 0.36, (x, 0.6, -0.23), M.cedar_dark, bevel=0.004)
            F.box(_name('shelf_rail'), 0.03, 0.025, 0.36, (x, 0.165, -0.23), M.cedar_dark)
        for z in (-0.36, -0.23, -0.10):
            F.box(_name('shelf_slat'), 0.78, 0.02, 0.09, (0, 0.185, z), M.cedar, bevel=0.003)
        for z in (-0.40, -0.06):
            F.box(_name('top_rail'), 0.8, 0.03, 0.05, (0, 0.6, z), M.cedar_dark, bevel=0.004)
        # the trough: a dark inside, corner posts, three slats a side and a rim; soil and herbs on top
        y0, w, h, d = 0.615, 0.84, 0.39, 0.38
        z0, z1 = WALL, WALL + d
        zc = (z0 + z1) / 2
        F.box('trough_inside', w - 0.05, h - 0.06, d - 0.05, (0, y0 + (h - 0.06) / 2, zc), M.cedar_dark)
        for px in (-w / 2 + 0.025, w / 2 - 0.025):
            for pz in (z0 + 0.025, z1 - 0.025):
                F.box(_name('trough_post'), 0.05, h, 0.05, (px, y0 + h / 2, pz), M.cedar_dark, bevel=0.006)
        sh = h / 3
        for k in range(3):
            y = y0 + sh * (k + 0.5)
            for z in (z0 + 0.01, z1 - 0.01):
                F.box(_name('trough_slat'), w - 0.02, sh - 0.014, 0.02, (0, y, z), M.cedar, bevel=0.003)
            for x in (-w / 2 + 0.01, w / 2 - 0.01):
                F.box(_name('trough_slat'), 0.02, sh - 0.014, d - 0.02, (x, y, zc), M.cedar, bevel=0.003)
        for z in (z0 + 0.02, z1 - 0.02):
            F.box(_name('trough_rim'), w + 0.02, 0.02, 0.05, (0, y0 + h + 0.01, z), M.cedar_dark, bevel=0.004)
        soil = y0 + h - 0.03
        F.box('trough_soil', w - 0.06, 0.02, d - 0.06, (0, soil - 0.01, zc), M.soil)
        _basil(F, -0.26, zc, soil, 1.1)
        _rosemary(F, 0.0, zc, soil, 1.0)
        _thyme(F, 0.27, zc, soil, 1.1)
        for x in (-0.26, 0.0, 0.27):                   # chalk tags on the trough's front
            F.box(_name('tag'), 0.09, 0.05, 0.006, (x, y0 + h - 0.08, z1 + 0.004), M.chalk, bevel=0.002)
            F.box(_name('tag_line'), 0.05, 0.006, 0.002, (x, y0 + h - 0.08, z1 + 0.008), M.soil)
        # the shelf below: the sprig's pot, a pot of basil and a trowel
        _pot(F, -0.2, -0.24, 0.195, r=0.08, h=0.15)
        top = _pot(F, 0.18, -0.25, 0.195, r=0.07, h=0.13)
        _basil(F, 0.18, -0.25, top, 0.8)
        F.box('trowel_blade', 0.05, 0.008, 0.12, (0.34, 0.2, -0.17), M.steel, rot=(0, 0, 25), bevel=0.003)
        F.box('trowel_grip', 0.03, 0.025, 0.09, (0.3, 0.21, -0.27), M.cedar_dark, rot=(0, 0, 25), bevel=0.01)
        # a basket of picked herbs on the floor in front
        G.revolve('basket', [(0, 0), (0.09, 0), (0.12, 0.09), (0.11, 0.09), (0.08, 0.012), (0, 0.012)],
                  at=F.at(-0.2, 0, 0.22), material=M.wicker, segments=16, scale=(1, 0.72, 1))
        G.torus('basket_handle', 0.1, 0.008, at=F.at(-0.2, 0.09, 0.22), material=M.wicker, rot=(90, 0, 90),
                major_segments=14, minor_segments=4)
        _basil(F, -0.2, 0.22, 0.04, 0.8)
    G.join(still, 'body', origin=(0, 0, 0))
    # The sprig, a stem and leaves in the left pot, built full size and scaled to nothing on the soil:
    # `sprig` grows it while a resident tends the herbs.
    sx, sz, sy = -0.2, -0.24, 0.195 + 0.15 * 0.9 + 0.006
    with G.new_parts() as sprig:
        G.rod('sprig_stem', F.at(sx, sy, sz), F.at(sx + 0.01, sy + 0.17, sz), 0.006, material=M.rosemary, verts=6)
        for k, (dy, a) in enumerate(((0.06, 0), (0.09, 140), (0.12, 260), (0.15, 30), (0.175, 200))):
            F.sphere(_name('sprig_leaf'), 0.03, (sx + 0.03 * G.cosd(a), sy + dy, sz + 0.03 * G.sind(a)),
                     M.basil_light, scale=(1.4, 0.7, 0.35), rot=(0, 20, -a), segments=8, rings=5)
    sprig = G.join(sprig, 'sprig', origin=F.at(sx, sy, sz))
    sprig.scale = (0.001, 0.001, 0.001)
    objs = G.settle(centre=False)
    for axis in 'XYZ':
        G.grow(sprig, 'sprig', seconds=0.9, axis=axis)
    return objs


# --- the deli pantry (1x1): a cream cupboard under a marble ledge, open shelves of deli things above -------
def _jar(F, x, y, z, m, r=0.045, h=0.15):
    F.cyl(_name('jar'), r, h, (x, y + h / 2, z), m, verts=14)
    F.cyl(_name('jar_lid'), r + 0.003, 0.025, (x, y + h + 0.012, z), M.brass, verts=14)


def _loaf(F, x, y, z, length=0.28, r=0.06, turn=0):
    F.sphere(_name('loaf'), r, (x, y + r * 0.8, z), M.bread, scale=(length / (2 * r), 1, 0.8), rot=(0, 0, turn),
             segments=12, rings=7)


def _pantry_door(F, x, hinge_x, zf):
    with G.new_parts() as parts:
        F.box(_name('door'), 0.395, 0.78, 0.02, (x, 0.5, zf + 0.01), M.cream, bevel=0.006)
        F.box(_name('door_panel'), 0.29, 0.62, 0.012, (x, 0.5, zf + 0.024), M.cream_panel, bevel=0.008)
        px = x + (0.15 if hinge_x < x else -0.15)
        F.box(_name('door_pull'), 0.016, 0.12, 0.016, (px, 0.62, zf + 0.045), M.brass, bevel=0.005)
        for y in (0.58, 0.66):
            F.box(_name('door_pull_post'), 0.012, 0.012, 0.024, (px, y, zf + 0.032), M.brass)
    return G.hinged(G.join(parts, _name('door')), F.at(hinge_x, 0.5, zf + 0.01))


def deliPantry():
    F = G.GameFrame()
    rnd = random.Random(2)
    zf = 0.06                       # the carcass's front
    with G.new_parts() as still:
        F.box('plinth', 0.82, 0.08, 0.42, (0, 0.04, WALL + 0.21), M.honey)
        for x in (-0.41, 0.41):
            F.box(_name('side'), 0.02, 0.84, zf - WALL, (x, 0.5, (zf + WALL) / 2), M.cream, bevel=0.004)
        F.box('back', 0.8, 0.84, 0.01, (0, 0.5, WALL + 0.005), M.back)
        F.box('bottom', 0.8, 0.02, zf - WALL, (0, 0.09, (zf + WALL) / 2), M.cream)
        F.box('mid_shelf', 0.8, 0.02, 0.4, (0, 0.5, WALL + 0.2), M.cream)
        F.box('stile', 0.02, 0.84, 0.02, (0, 0.5, zf - 0.01), M.cream)
        # inside, behind the doors: jars and a crock below, tins, oil and a cheese above
        for k, (x, m) in enumerate(((-0.3, M.jar_a), (-0.2, M.jar_b), (-0.1, M.jar_c))):
            _jar(F, x, 0.1, -0.2, m, h=0.18)
        F.cyl('crock', 0.1, 0.2, (0.2, 0.2, -0.22), M.cream_panel, verts=16)
        F.cyl('crock_lid', 0.105, 0.02, (0.2, 0.31, -0.22), M.honey, verts=16)
        for x in (-0.28, -0.18):
            F.cyl(_name('tin'), 0.045, 0.12, (x, 0.57, -0.2), M.jar_c, verts=12)
        F.cyl('oil', 0.035, 0.24, (0.0, 0.63, -0.25), M.oil, verts=10)
        F.cyl('oil_neck', 0.012, 0.06, (0.0, 0.78, -0.25), M.oil, verts=8)
        F.cyl('cheese_in', 0.11, 0.08, (0.22, 0.55, -0.2), M.cheese, verts=18)
        # the marble ledge
        F.box('ledge', 0.88, 0.04, 0.56, (0, 0.94, WALL + 0.28), M.marble, bevel=0.008)
        # the open shelves above
        for x in (-0.405, 0.405):
            F.box(_name('upper_side'), 0.03, 0.92, 0.4, (x, 1.42, WALL + 0.2), M.cream, bevel=0.005)
        F.box('upper_back', 0.78, 0.92, 0.01, (0, 1.42, WALL + 0.005), M.back)
        for y in (1.2, 1.5):
            F.box(_name('shelf'), 0.78, 0.02, 0.36, (0, y, WALL + 0.19), M.honey, bevel=0.003)
        F.box('upper_top', 0.84, 0.03, 0.42, (0, 1.895, WALL + 0.21), M.cream)
        F.box('crown', 0.88, 0.06, 0.46, (0, 1.96, WALL + 0.22), M.honey, bevel=0.01)
        F.box('header', 0.78, 0.07, 0.025, (0, 1.845, -0.045), M.cream, bevel=0.004)
        for k, (x, m) in enumerate(((-0.3, M.jar_a), (-0.19, M.jar_b), (-0.08, M.jar_c))):
            _jar(F, x, 1.21, -0.24, m, r=0.045, h=0.14)
        F.cyl('wheel_a', 0.11, 0.08, (0.2, 1.25, -0.25), M.cheese, verts=20)
        F.cyl('wheel_b', 0.1, 0.07, (0.21, 1.325, -0.25), M.cheese_light, verts=20)
        _loaf(F, -0.14, 1.51, -0.24, 0.3, 0.055)
        _loaf(F, 0.16, 1.51, -0.22, 0.24, 0.05, turn=25)
        for x, m in ((-0.33, M.wine), (-0.26, M.bottle)):
            F.cyl(_name('bottle'), 0.032, 0.2, (x, 1.61, -0.28), m, verts=12)
            F.cyl(_name('bottle_neck'), 0.012, 0.08, (x, 1.75, -0.28), m, verts=8)
        # hanging salami on a brass rail under the header
        G.rod('rail', F.at(-0.37, 1.79, -0.1), F.at(0.37, 1.79, -0.1), 0.008, material=M.brass, verts=8)
        for x in (-0.24, -0.08, 0.1, 0.25):
            L = rnd.uniform(0.16, 0.22)
            F.cyl(_name('string'), 0.003, 0.05, (x, 1.765, -0.1), M.chalk, verts=4)
            F.sphere(_name('salami'), 0.03, (x, 1.74 - L / 2, -0.1), M.salami, scale=(1, 1, L / 0.06), segments=10, rings=8)
        # on the ledge: a board with cheese and bread, two tomatoes
        F.box('board', 0.4, 0.02, 0.22, (0.08, 0.97, -0.01), M.board, bevel=0.006)
        F.cone('wedge', 0.08, 0.0, 0.06, (0.0, 1.01, -0.01), M.cheese_light, rot=(0, 0, 30), verts=3)
        _loaf(F, 0.17, 0.98, 0.0, 0.18, 0.04, turn=70)
        for x, z in ((-0.3, 0.02), (-0.24, 0.06)):
            F.sphere(_name('tomato'), 0.028, (x, 0.988, z), M.tomato, segments=10, rings=6)
    G.join(still, 'body', origin=(0, 0, 0))
    # The two doors, hinged on their outer edges: `doors_open` swings them out.
    left = _pantry_door(F, -0.2025, -0.40, zf)
    right = _pantry_door(F, 0.2025, 0.40, zf)
    objs = G.settle(centre=False)
    G.swing(left, 'doors_open', 100, seconds=0.8)
    G.swing(right, 'doors_open', -100, seconds=0.8)
    return objs


# --- the espresso bar (1x1): a lever espresso machine on a teal-and-oak cart ----------------------------------
GROUPS = (-0.16, 0.04)   # the group heads' x


def _cup(F, x, y, z, m=None):
    G.revolve(_name('cup'), [(0, 0), (0.022, 0), (0.03, 0.05), (0.027, 0.05), (0.02, 0.006), (0, 0.006)],
              at=F.at(x, y, z), material=m or M.cup, segments=14)
    G.torus(_name('cup_handle'), 0.014, 0.004, at=F.at(x + 0.032, y + 0.027, z), material=m or M.cup, rot=(90, 0, 0),
            major_segments=10, minor_segments=4)


def espressoBar():
    F = G.GameFrame()
    zc = WALL + 0.25
    with G.new_parts() as still:
        F.box('top', 0.76, 0.04, 0.5, (0, 0.88, zc), M.oak_top, bevel=0.01)
        for y in (0.42, 0.13):
            F.box(_name('shelf'), 0.7, 0.03, 0.44, (0, y, zc), M.oak_top, bevel=0.006)
        for x in (-0.35, 0.35):
            for z in (WALL + 0.03, zc + 0.22):
                F.box(_name('post'), 0.04, 0.8, 0.04, (x, 0.47, z), M.teal, bevel=0.008)
                F.cyl(_name('fork'), 0.012, 0.03, (x, 0.07, z), M.chrome, verts=8)
                G.torus(_name('castor'), 0.025, 0.012, at=F.at(x, 0.037, z), material=M.dark, rot=(90, 0, 90),
                        major_segments=14, minor_segments=6)
        G.rod('towel_rail', F.at(-0.33, 0.75, 0.064), F.at(0.33, 0.75, 0.064), 0.008, material=M.brass, verts=8)
        F.box('towel', 0.16, 0.16, 0.01, (0.16, 0.675, 0.07), M.chalk, bevel=0.003)
        F.box('towel_fold', 0.16, 0.012, 0.022, (0.16, 0.756, 0.064), M.chalk, bevel=0.004)
        # the machine: a cream body, a copper boiler dome with a brass eagle, gauge, group heads and levers
        F.box('machine_base', 0.46, 0.03, 0.32, (-0.06, 0.915, -0.26), M.chrome, bevel=0.008)
        F.box('machine', 0.42, 0.3, 0.3, (-0.06, 1.08, -0.26), M.machine, bevel=0.03)
        G.revolve('boiler', [(0, 0), (0.12, 0), (0.12, 0.06), (0.1, 0.15), (0.06, 0.21), (0.02, 0.23), (0, 0.23)],
                  at=F.at(-0.06, 1.23, -0.27), material=M.copper, segments=22)
        F.sphere('eagle', 0.03, (-0.06, 1.48, -0.27), M.brass, segments=10, rings=6)
        F.cyl('eagle_post', 0.008, 0.03, (-0.06, 1.455, -0.27), M.brass, verts=6)
        F.cyl('gauge', 0.03, 0.01, (0.1, 1.13, -0.105), M.chrome, rot=(90, 0, 0), verts=14)
        F.cyl('gauge_face', 0.024, 0.004, (0.1, 1.13, -0.099), M.cup, rot=(90, 0, 0), verts=14)
        F.box('drip_tray', 0.4, 0.02, 0.13, (-0.06, 0.915, -0.05), M.chrome, bevel=0.004)
        for x in GROUPS:
            F.box(_name('group_neck'), 0.05, 0.04, 0.05, (x, 1.12, -0.09), M.chrome, bevel=0.008)
            F.cyl(_name('group'), 0.036, 0.04, (x, 1.09, -0.07), M.chrome, verts=14)
            F.cyl(_name('portafilter'), 0.032, 0.02, (x, 1.06, -0.07), M.chrome, verts=14)
            F.cone(_name('spout'), 0.012, 0.006, 0.02, (x, 1.042, -0.07), M.chrome, verts=8)
            G.rod(_name('pf_handle'), F.at(x, 1.06, -0.04), F.at(x, 1.06, 0.07), 0.011, material=M.black, verts=8)
            G.rod(_name('lever'), F.at(x, 1.16, -0.1), F.at(x, 1.34, 0.0), 0.012, material=M.chrome, verts=8)
            F.sphere(_name('lever_knob'), 0.025, (x, 1.35, 0.005), M.black, scale=(1, 1, 1.3), segments=10, rings=6)
            _cup(F, x, 0.925, -0.07)
        # the grinder at the back right, a milk jug before it
        F.box('grinder_base', 0.12, 0.12, 0.14, (0.26, 0.96, -0.33), M.black, bevel=0.012)
        G.revolve('hopper', [(0, 0), (0.035, 0), (0.035, 0.03), (0.07, 0.15), (0, 0.15)], at=F.at(0.26, 1.02, -0.33),
                  material=M.machine, segments=14)
        F.sphere('hopper_beans', 0.068, (0.26, 1.165, -0.33), M.beans, scale=(1, 1, 0.35), segments=12, rings=6)
        F.cone('milk_jug', 0.045, 0.035, 0.11, (0.24, 0.955, -0.1), M.chrome, verts=14)
        # below: cups on the middle shelf, bean sacks and a syrup bottle
        for i, x in enumerate((-0.26, -0.18, -0.1)):
            _cup(F, x, 0.435, -0.18, M.book_teal if i == 1 else None)
        F.cyl('syrup', 0.035, 0.18, (0.2, 0.525, -0.25), M.bottle, verts=12)
        F.cyl('syrup_neck', 0.012, 0.05, (0.2, 0.64, -0.25), M.bottle, verts=8)
        F.box('sack_a', 0.24, 0.2, 0.16, (0.17, 0.245, -0.24), M.sack, bevel=0.05, bevel_segments=2)
        F.box('sack_b', 0.2, 0.16, 0.14, (-0.16, 0.225, -0.22), M.wicker, bevel=0.045, bevel_segments=2)
    G.join(still, 'body', origin=(0, 0, 0))
    # The coffee running from the left group's spout into its cup, built full length and scaled to nothing
    # at the spout: `brew` grows it.
    gx = GROUPS[0]
    with G.new_parts() as stream:
        for dx in (-0.006, 0.006):
            F.cyl(_name('stream'), 0.004, 0.08, (gx + dx, 0.992, -0.07), M.coffee, verts=6)
    stream = G.join(stream, 'stream', origin=F.at(gx, 1.032, -0.07))
    stream.scale[2] = 0.001
    # The steam wand on the machine's right side, hanging from its ball joint: `brew` swings it out.
    with G.new_parts() as wand:
        F.sphere('wand_joint', 0.018, (0.165, 1.12, -0.16), M.chrome, segments=10, rings=6)
        G.rod('wand', F.at(0.165, 1.12, -0.16), F.at(0.175, 0.96, -0.13), 0.007, material=M.chrome, verts=8)
        F.cone('wand_tip', 0.008, 0.004, 0.02, (0.176, 0.95, -0.128), M.chrome, verts=8)
    wand = G.hinged(G.join(wand, 'wand'), F.at(0.165, 1.12, -0.16))
    objs = G.settle(centre=False)
    G.grow(stream, 'brew', seconds=0.4)
    G.swing(wand, 'brew', 40, seconds=0.6, axis='Y')
    return objs


# --- the cookbook lectern (1x1): a turned walnut lectern with an open cookbook; a stool of cookbooks -------
DESK, TILT = (0.0, 1.02, -0.04), 25     # the reading desk's middle (game) and its slope (degrees, front down)


def _on_desk(lx, ly, lz):
    """A point on the desk, `ly` up off it and `lz` down its slope towards the front, as game coordinates."""
    c, s = G.cosd(TILT), G.sind(TILT)
    x, y, z = DESK
    return (x + lx, y + ly * c - lz * s, z + ly * s + lz * c)


def cookbookLectern():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.cyl('foot', 0.18, 0.05, (0, 0.025, -0.05), M.walnut, verts=24, bevel=0.01)
        G.revolve('column', [(0, 0), (0.06, 0), (0.045, 0.06), (0.03, 0.15), (0.034, 0.45), (0.025, 0.72),
                             (0.04, 0.8), (0.06, 0.88), (0, 0.88)], at=F.at(0, 0.05, -0.05), material=M.walnut,
                  segments=16)
        F.box('desk', 0.56, 0.04, 0.42, DESK, M.walnut, rot=(TILT, 0, 0), bevel=0.01)
        F.box('ledge', 0.5, 0.03, 0.03, _on_desk(0, 0.03, 0.2), M.honey_wood, rot=(TILT, 0, 0), bevel=0.008)
        # the open cookbook: its cover, two page blocks, a recipe photo and a ribbon
        F.box('book_cover', 0.48, 0.012, 0.34, _on_desk(0, 0.026, 0), M.cover, rot=(TILT, 0, 0), bevel=0.004)
        for s in (-1, 1):
            F.box(_name('pages'), 0.225, 0.03, 0.31, _on_desk(s * 0.115, 0.046, 0), M.page, rot=(TILT, 0, 0), bevel=0.01)
        F.box('photo', 0.13, 0.002, 0.09, _on_desk(0.11, 0.062, -0.06), M.photo, rot=(TILT, 0, 0))
        for k in range(4):
            F.box(_name('text'), 0.15, 0.002, 0.008, _on_desk(0.11, 0.062, 0.03 + 0.025 * k), M.board, rot=(TILT, 0, 0))
        F.box('ribbon', 0.012, 0.004, 0.36, _on_desk(-0.02, 0.063, 0.02), M.tomato, rot=(TILT, 0, 0))
        # a stool with a pile of cookbooks, and a jar of wooden spoons on the floor
        G.revolve('stool', [(0, 0), (0.13, 0), (0.12, 0.4), (0.15, 0.4), (0.15, 0.42), (0, 0.42)],
                  at=F.at(0.29, 0, -0.27), material=M.honey_wood, segments=18)
        y = 0.42
        for w, h, m, r in ((0.26, 0.05, M.book_teal, 6), (0.24, 0.04, M.book_mustard, -8), (0.25, 0.06, M.cover, 3),
                           (0.22, 0.04, M.book_sage, 12)):
            F.box(_name('cookbook'), w, h, w * 0.75, (0.29, y + h / 2, -0.27), m, rot=(0, 0, r), bevel=0.006)
            F.box(_name('cookbook_pages'), w - 0.01, h - 0.012, w * 0.75 - 0.012, (0.29 + 0.006, y + h / 2, -0.27),
                  M.page, rot=(0, 0, r))
            y += h
        G.revolve('spoon_jar', [(0, 0), (0.045, 0), (0.05, 0.14), (0.045, 0.14), (0.04, 0.012), (0, 0.012)],
                  at=F.at(-0.31, 0, -0.3), material=M.cup, segments=14)
        for dx, dz, lean in ((-0.015, 0, -10), (0.015, 0.01, 8), (0, -0.015, 3)):
            G.rod(_name('spoon'), F.at(-0.31 + dx, 0.03, -0.3 + dz), F.at(-0.31 + dx * 3 + lean * 0.002, 0.3, -0.3 + dz * 2),
                  0.007, material=M.honey_wood, verts=6)
            F.sphere(_name('spoon_bowl'), 0.022, (-0.31 + dx * 3 + lean * 0.002, 0.31, -0.3 + dz * 2), M.honey_wood,
                     scale=(1, 0.5, 1.3), segments=8, rings=5)
    G.join(still, 'body', origin=(0, 0, 0))
    # The page on the right-hand block (game -x), built flat with its spine on the book's middle, its origin on
    # the spine, then tilted onto the desk; `page` turns it over about the spine (its own Y, after the tilt).
    hx, hy, hz = _on_desk(0, 0.0625, 0)
    with G.new_parts() as page:
        F.box('page_leaf', 0.215, 0.003, 0.3, (hx - 0.11, hy, hz), M.page)
        for k in range(5):
            F.box(_name('page_text'), 0.15, 0.002, 0.008, (hx - 0.11, hy + 0.002, hz - 0.08 + 0.035 * k), M.board)
    page = G.join(page, 'page', origin=F.at(hx, hy, hz))
    page.rotation_mode = 'YXZ'            # the tilt (X) after the turn about the spine (Y)
    page.rotation_euler = (math.radians(TILT), 0, 0)
    objs = G.settle(centre=False)
    _clip(page, 'page', 'rotation_euler', 1, 0.0, math.radians(-172), 0.9)
    return objs


# --- the wood-fired pizza oven (2x1): a brick stand and plastered dome, a prep counter at its right ---------
OX = -0.38          # the oven's middle (game x)
HEARTH = 0.92       # the oven floor and the mouth's sill
MOUTH_Z = 0.403     # the mouth's dark face


def _bricks(F, x0, x1, y0, y1, z, rows, length=0.15):
    """Brick faces in running bond on a front at depth `z`, over the box x0..x1, y0..y1."""
    h = (y1 - y0) / rows
    for r in range(rows):
        y = y0 + h * (r + 0.5)
        x = x0 - (length / 2 if r % 2 else 0)
        k = 0
        while x < x1 - 0.01:
            a, b = max(x, x0), min(x + length, x1)
            if b - a > 0.02:
                m = (M.brick, M.brick_light, M.brick)[(r + k) % 3]
                F.box(_name('brick'), b - a - 0.008, h - 0.01, 0.01, ((a + b) / 2, y, z), m)
            x += length
            k += 1


def pizzaOven():
    F = G.GameFrame()
    front = 0.43
    with G.new_parts() as still:
        # the stand: two brick pillars, a lintel over the log niche, the hearth slab on top
        for s in (-1, 1):
            x = OX + s * 0.45
            F.box(_name('pillar'), 0.16, 0.7, front - WALL, (x, 0.35, (front + WALL) / 2), M.brick)
            _bricks(F, x - 0.08, x + 0.08, 0.0, 0.7, front + 0.004, 9, 0.16)
        F.box('lintel', 1.06, 0.16, front - WALL, (OX, 0.78, (front + WALL) / 2), M.brick)
        _bricks(F, OX - 0.53, OX + 0.53, 0.7, 0.86, front + 0.004, 2, 0.2)
        F.box('niche_back', 0.76, 0.7, 0.02, (OX, 0.35, WALL + 0.01), M.brick_dark)
        F.box('niche_floor', 0.76, 0.03, front - WALL - 0.02, (OX, 0.015, (front + WALL) / 2 - 0.01), M.stone)
        for i in range(3):
            for j in range(4 - i):
                F.cyl(_name('log'), 0.05, 0.6, (OX - 0.2 + j * 0.13 + i * 0.065, 0.08 + i * 0.088, 0.04), M.log,
                      rot=(90, 0, 0), verts=10)
                F.cyl(_name('log_end'), 0.042, 0.004, (OX - 0.2 + j * 0.13 + i * 0.065, 0.08 + i * 0.088, 0.342),
                      M.wicker, rot=(90, 0, 0), verts=10)
        F.box('hearth', 1.12, 0.06, front - WALL + 0.02, (OX, HEARTH - 0.03, (front + WALL) / 2 + 0.01), M.stone,
              bevel=0.01)
        # the dome, the brick arch of its mouth and the flue
        G.revolve('dome', [(0, 0), (0.46, 0), (0.455, 0.12), (0.42, 0.3), (0.34, 0.45), (0.2, 0.56), (0, 0.6)],
                  at=F.at(OX, HEARTH, -0.06), material=M.plaster, segments=32, scale=(1, 0.82, 1))
        F.cyl('arch', 0.3, 0.14, (OX, 1.04, 0.33), M.brick, rot=(90, 0, 0), verts=28)
        F.cyl('mouth', 0.2, 0.006, (OX, HEARTH, MOUTH_Z), M.mouth, rot=(90, 0, 0), verts=24)
        for k in range(9):
            a = 10 + 20 * k
            F.box(_name('voussoir'), 0.075, 0.09, 0.012,
                  (OX + 0.25 * G.cosd(a), HEARTH + 0.25 * G.sind(a), MOUTH_Z + 0.003),
                  (M.brick_light, M.brick_dark)[k % 2], rot=(0, a - 90, 0))
        for k, (dx, r) in enumerate(((-0.07, 0.04), (0.0, 0.05), (0.08, 0.035))):
            F.sphere(_name('ember'), r, (OX + dx, HEARTH + 0.004, MOUTH_Z - 0.01), M.ember, scale=(1, 0.3, 0.45),
                     segments=10, rings=5)
        F.cyl('flue', 0.07, 0.42, (OX + 0.12, 1.65, -0.2), M.iron, verts=16)
        F.cone('flue_cap', 0.13, 0.03, 0.08, (OX + 0.12, 1.86, -0.2), M.iron, verts=16)
        # the prep counter: a stone cabinet with an oak door and a worktop
        cx = 0.56
        F.box('counter', 0.74, 0.84, 0.62, (cx, 0.42, WALL + 0.31), M.stone, bevel=0.01)
        F.box('counter_top', 0.78, 0.05, 0.66, (cx, 0.865, WALL + 0.32), M.worktop, bevel=0.01)
        F.box('counter_door', 0.6, 0.62, 0.02, (cx, 0.43, WALL + 0.63), M.door_oak, bevel=0.006)
        F.box('counter_pull', 0.1, 0.015, 0.02, (cx, 0.66, WALL + 0.65), M.iron, bevel=0.005)
        # on it: a ball of dough, toppings and oil, a pot of basil
        F.sphere('dough_ball', 0.07, (cx + 0.04, 0.91, -0.25), M.dough, scale=(1, 1, 0.55), segments=14, rings=8)
        F.cyl('flour_board', 0.16, 0.012, (cx + 0.04, 0.896, -0.25), M.board, verts=18)
        F.cyl('bowl_sauce', 0.06, 0.05, (cx + 0.26, 0.915, -0.08), M.cup, verts=14)
        F.cyl('sauce', 0.052, 0.004, (cx + 0.26, 0.94, -0.08), M.tomato, verts=14)
        F.cyl('oil', 0.035, 0.18, (cx + 0.14, 0.98, -0.37), M.oil, verts=12)
        F.cyl('oil_neck', 0.012, 0.05, (cx + 0.14, 1.09, -0.37), M.oil, verts=8)
        top = _pot(F, cx + 0.27, -0.33, 0.89, r=0.06, h=0.1)
        _basil(F, cx + 0.27, -0.33, top, 0.75)
    G.join(still, 'body', origin=(0, 0, 0))
    # The door: an iron grille over the mouth, the fire showing through it; `door_off` lifts it aside.
    r, dz = 0.19, MOUTH_Z + 0.012
    with G.new_parts() as door:
        pts = [(OX + r * G.cosd(a), HEARTH + 0.01 + r * G.sind(a)) for a in range(0, 181, 20)]
        for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
            G.rod(_name('door_arc'), F.at(x0, y0, dz), F.at(x1, y1, dz), 0.009, material=M.iron, verts=6)
        G.rod('door_sill', F.at(OX - r, HEARTH + 0.012, dz), F.at(OX + r, HEARTH + 0.012, dz), 0.009, material=M.iron, verts=6)
        for dx in (-0.12, -0.06, 0.0, 0.06, 0.12):
            top = HEARTH + 0.01 + math.sqrt(r * r - dx * dx)
            G.rod(_name('door_bar'), F.at(OX + dx, HEARTH + 0.012, dz), F.at(OX + dx, top, dz), 0.006, material=M.iron, verts=6)
        for dx in (-0.07, 0.07):
            G.rod(_name('door_post'), F.at(OX + dx, HEARTH + 0.1, dz), F.at(OX + dx, HEARTH + 0.1, dz + 0.05), 0.007,
                  material=M.iron, verts=6)
        G.rod('door_handle', F.at(OX - 0.08, HEARTH + 0.1, dz + 0.05), F.at(OX + 0.08, HEARTH + 0.1, dz + 0.05), 0.011,
              material=M.door_oak, verts=8)
    door = G.join(door, 'oven_door', origin=F.at(OX, HEARTH, dz))
    # The peel, a pizza on it, lying on the counter: `door_off` carries it round to the mouth and in.
    px, pz = 0.48, -0.1
    with G.new_parts() as peel:
        F.box('peel_blade', 0.3, 0.012, 0.3, (px, 0.896, pz), M.peel, bevel=0.004)
        F.box('peel_handle', 0.04, 0.02, 0.36, (px, 0.9, pz + 0.32), M.peel, bevel=0.008)
        F.cyl('pizza', 0.13, 0.016, (px, 0.91, pz), M.dough, verts=20)
        F.cyl('pizza_sauce', 0.11, 0.004, (px, 0.919, pz), M.tomato, verts=20)
        for k in range(7):
            a, d = k * 51, 0.04 + 0.04 * (k % 2)
            F.cyl(_name('mozzarella'), 0.018, 0.006, (px + d * G.cosd(a), 0.922, pz + d * G.sind(a)), M.mozzarella, verts=8)
        for k in range(4):
            a = k * 90 + 30
            F.sphere(_name('pizza_basil'), 0.014, (px + 0.06 * G.cosd(a), 0.925, pz + 0.06 * G.sind(a)), M.basil,
                     scale=(1.3, 1, 0.4), segments=6, rings=4)
    peel = G.join(peel, 'peel', origin=F.at(px, 0.896, pz))
    # The fire in the mouth, flat flames behind the grille, built full height and flattened onto the sill:
    # `flames` grows them.
    with G.new_parts() as fire:
        for k, (dx, rr, h, m) in enumerate(((-0.1, 0.035, 0.12, M.flame), (-0.04, 0.045, 0.2, M.flame),
                                            (0.03, 0.045, 0.17, M.flame), (0.1, 0.035, 0.11, M.flame),
                                            (-0.02, 0.03, 0.12, M.flame_core), (0.05, 0.025, 0.09, M.flame_core))):
            F.cone(_name('flame'), rr, 0.0, h, (OX + dx, HEARTH + h / 2, MOUTH_Z + 0.004 + 0.002 * (k > 3)), m,
                   scale=(1, 0.2, 1), verts=10)
    fire = G.join(fire, 'flames', origin=F.at(OX, HEARTH, MOUTH_Z))
    fire.scale[2] = 0.001
    objs = G.settle(centre=False)
    # door_off: the door lifts off the mouth and is set down on the ground before the stand's left pillar; the
    # peel follows, round in front of the oven and in through the mouth (Blender axes: x = -game x, y = -game
    # depth, z = height).
    f = last_frame
    for i, (lift, down) in enumerate(((0.0, 0.42), (-0.06, -0.06), (0.03, -(HEARTH - 0.005)))):
        rest = door.location[i]
        _clip(door, 'door_off', 'location', i, [(1, rest), (f(0.2), rest + lift), (f(0.6), rest + down), (f(1.4), rest + down)])
    lx, ly, lz = peel.location
    # game path: up off the counter, forward, along to the oven's middle, back into the mouth
    gx, gz = OX - px, 0.25 - pz
    keys_x = [(1, lx), (f(0.35), lx), (f(0.6), lx), (f(1.0), lx - gx), (f(1.4), lx - gx)]
    keys_y = [(1, ly), (f(0.35), ly), (f(0.6), ly - 0.62), (f(1.0), ly - 0.62), (f(1.4), ly - gz)]
    keys_z = [(1, lz), (f(0.35), lz + 0.06), (f(0.6), lz + 0.06), (f(1.0), lz + 0.06), (f(1.4), lz + 0.03)]
    for i, keys in enumerate((keys_x, keys_y, keys_z)):
        _clip(peel, 'door_off', 'location', i, keys)
    G.grow(fire, 'flames', seconds=0.8)
    return objs


# --- the chef's island (2x1): a marble-topped island, a hob on it, drawers on the cook's side, two stools ----
TOP = 0.9   # the content's top (`surface.height`)


STOOL = 0.65         # the counter stools' seat top: 25 cm under the top (counter stools are 0.58-0.71 for it)
STOOL_Z = -0.235     # their middles' depth, the knees under the top's back overhang (`furniture.json`)


def _stool(F, x, z):
    """A counter stool with its seat at STOOL, a foot ring and a low back, centred on (x, z), facing the front."""
    y = STOOL - 0.045                                             # the wooden seat's middle
    F.cyl(_name('stool_seat'), 0.18, 0.03, (x, y, z), M.oak, verts=20, bevel=0.008)
    F.cyl(_name('stool_cushion'), 0.165, 0.03, (x, STOOL - 0.015, z), M.seat, verts=20, bevel=0.012)
    for k in range(4):
        a = 45 + 90 * k
        G.rod(_name('stool_leg'), F.at(x + 0.19 * G.cosd(a), 0.0, z + 0.19 * G.sind(a)),
              F.at(x + 0.12 * G.cosd(a), y - 0.015, z + 0.12 * G.sind(a)), 0.016, material=M.oak_dark, verts=8)
    G.torus(_name('stool_ring'), 0.165, 0.009, at=F.at(x, 0.3, z), material=M.oak_dark, major_segments=20,
            minor_segments=4)
    for dx in (-0.12, 0.12):
        G.rod(_name('stool_back_post'), F.at(x + dx, STOOL - 0.03, z - 0.13), F.at(x + dx, STOOL + 0.17, z - 0.17),
              0.012, material=M.oak_dark, verts=6)
    F.box(_name('stool_back'), 0.32, 0.1, 0.025, (x, STOOL + 0.21, z - 0.175), M.oak, rot=(-10, 0, 0), bevel=0.01)


def chefsIsland():
    F = G.GameFrame()
    zf = 0.43            # the cook's side
    with G.new_parts() as still:
        F.box('top', 1.86, 0.04, 0.46, (0, TOP - 0.02, 0.225), M.marble, bevel=0.01)
        for s in (-1, 1):
            x = s * 0.8
            F.box(_name('end'), 0.26, 0.78, zf, (x, 0.47, zf / 2), M.cabinet, bevel=0.008)
            F.box(_name('end_plinth'), 0.24, 0.08, zf - 0.03, (x, 0.04, zf / 2 - 0.015), M.dark)
            F.box(_name('end_drawer'), 0.22, 0.16, 0.014, (x, 0.75, zf + 0.007), M.cabinet, bevel=0.005)
            F.box(_name('end_door'), 0.22, 0.54, 0.014, (x, 0.37, zf + 0.007), M.cabinet, bevel=0.005)
            for y in (0.79, 0.6):
                F.box(_name('end_pull'), 0.09, 0.012, 0.016, (x, y, zf + 0.022), M.brass, bevel=0.005)
        # between the ends, the drawer bank under the top; below it, the sitters' legroom open right through
        F.box('apron', 1.34, 0.16, 0.02, (0, 0.8, 0.3), M.cabinet)                 # the drawers' back
        F.box('apron_under', 1.34, 0.02, zf - 0.3, (0, 0.73, (zf + 0.3) / 2), M.cabinet)
        for x in (-0.45, 0.45):
            F.box(_name('side_drawer'), 0.42, 0.13, 0.014, (x, 0.8, zf + 0.007), M.cabinet, bevel=0.005)
            F.box(_name('side_pull'), 0.12, 0.012, 0.016, (x, 0.8, zf + 0.022), M.brass, bevel=0.005)
        # the sitters' side: their knee space under the top, a brass foot rail
        G.rod('foot_rail', F.at(-0.67, 0.25, 0.06), F.at(0.67, 0.25, 0.06), 0.012, material=M.brass, verts=10)
        # on the top: the hob with a copper pan and a stock pot, a board of vegetables, two tasting plates
        F.box('hob', 0.5, 0.008, 0.28, (0, TOP + 0.004, 0.27), M.hob, bevel=0.003)
        for dx in (-0.12, 0.12):
            G.torus(_name('hob_ring'), 0.08, 0.003, at=F.at(dx, TOP + 0.009, 0.27), material=M.ring, major_segments=20,
                    minor_segments=4)
        G.revolve('pan', [(0, 0), (0.1, 0), (0.115, 0.05), (0.108, 0.05), (0.094, 0.008), (0, 0.008)],
                  at=F.at(-0.12, TOP + 0.008, 0.27), material=M.copper, segments=20)
        G.rod('pan_handle', F.at(-0.23, TOP + 0.04, 0.27), F.at(-0.4, TOP + 0.05, 0.3), 0.012, material=M.oak_dark, verts=8)
        G.revolve('stock_pot', [(0, 0), (0.11, 0), (0.12, 0.01), (0.12, 0.18), (0.112, 0.18), (0.112, 0.012), (0, 0.012)],
                  at=F.at(0.12, TOP + 0.008, 0.27), material=M.worktop, segments=20)
        F.cyl('pot_lid', 0.118, 0.012, (0.12, TOP + 0.194, 0.27), M.worktop, verts=20)
        F.sphere('pot_knob', 0.016, (0.12, TOP + 0.21, 0.27), M.black, segments=8, rings=5)
        F.box('chop_board', 0.34, 0.02, 0.22, (0.62, TOP + 0.01, 0.3), M.oak, bevel=0.006)
        F.sphere('tomato', 0.035, (0.56, TOP + 0.05, 0.3), M.tomato, segments=10, rings=6)
        F.sphere('carrot', 0.018, (0.66, TOP + 0.04, 0.27), M.carrot, scale=(4, 1, 1), rot=(0, 0, 40), segments=8, rings=5)
        _basil(F, 0.72, 0.33, TOP + 0.02, 0.6)
        F.box('knife', 0.16, 0.006, 0.025, (0.6, TOP + 0.023, 0.37), M.chrome, bevel=0.002)
        for x in (-0.5, 0.5):
            G.revolve(_name('plate'), [(0, 0), (0.07, 0), (0.11, 0.012), (0.105, 0.016), (0, 0.006)],
                      at=F.at(x, TOP, 0.11), material=M.plate, segments=20)
            F.sphere(_name('bite'), 0.03, (x - 0.02, TOP + 0.02, 0.11), M.salami, scale=(1, 1, 0.5), segments=8, rings=5)
            F.sphere(_name('bite'), 0.025, (x + 0.03, TOP + 0.02, 0.1), M.cheese_light, scale=(1, 1, 0.6), segments=8,
                     rings=5)
        F.box('menu', 0.14, 0.18, 0.006, (0.0, TOP + 0.08, 0.06), M.chalk, rot=(-12, 0, 0), bevel=0.002)
        _stool(F, -0.5, STOOL_Z)
        _stool(F, 0.5, STOOL_Z)
    G.join(still, 'body', origin=(0, 0, 0))
    # The middle drawer on the cook's side: `drawer_open` pulls it out 12 cm.
    with G.new_parts() as drawer:
        F.box('drawer_front', 0.42, 0.13, 0.014, (0, 0.8, zf + 0.007), M.cabinet, bevel=0.005)
        F.box('drawer_pull', 0.12, 0.012, 0.016, (0, 0.8, zf + 0.022), M.brass, bevel=0.005)
        F.box('drawer_floor', 0.38, 0.01, 0.12, (0, 0.745, zf - 0.06), M.cabinet)
        for x in (-0.19, 0.19):
            F.box(_name('drawer_side'), 0.01, 0.08, 0.12, (x, 0.785, zf - 0.06), M.cabinet)
        for k, x in enumerate((-0.12, -0.05, 0.03, 0.1)):
            F.box(_name('utensil'), 0.02, 0.008, 0.11, (x, 0.755, zf - 0.06), (M.chrome, M.oak_top)[k % 2])
    drawer = G.join(drawer, 'drawer')
    # The ring under the copper pan glowing hot, scaled to nothing at rest: `burner_on` grows it while it cooks.
    with G.new_parts() as heat:
        F.cyl('hot_ring', 0.13, 0.003, (-0.12, TOP + 0.0085, 0.27), M.heat, verts=24)
    heat = G.join(heat, 'hot_ring', origin=F.at(-0.12, TOP + 0.0085, 0.27))
    heat.scale = (0.001, 0.001, 0.001)
    objs = G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, -0.12, 0), seconds=0.6)
    for axis in 'XYZ':
        G.grow(heat, 'burner_on', seconds=0.5, axis=axis)
    return objs


FAMILY = G.Family(
    'pack_foodie', M,
    [G.Model('herbGarden', herbGarden, size=(0.86, 1.293, 0.763), key='model.foodie.herbGarden', centre=(0, -0.054),
             motions={'uses': {'tendHerbs': 'sprig'}},
             paint=[('cedar', 'Body'), ('terracotta', 'Pots')]),
     G.Model('deliPantry', deliPantry, size=(0.88, 1.99, 0.57), key='model.foodie.deliPantry', centre=(0, -0.155),
             motions={'uses': {'nibble': 'doors_open', 'cheeseBoard': 'doors_open'}, 'hold': 2.5},
             paint=[('cream', 'Body'), ('marble', 'Worktop'), ('honey', 'Shelves')]),
     G.Model('espressoBar', espressoBar, size=(0.76, 1.509, 0.514), key='model.foodie.espressoBar', centre=(0, -0.18),
             motions={'uses': {'espresso': 'brew', 'latteArt': 'brew'}, 'hold': 4.0},
             paint=[('teal', 'Body'), ('oak_top', 'Worktop'), ('machine', 'Machine')]),
     G.Model('cookbookLectern', cookbookLectern, size=(0.836, 1.147, 0.588), key='model.foodie.cookbookLectern',
             centre=(0.022, -0.123), motions={'uses': {'studyRecipes': 'page', 'foodMagazines': 'page'}},
             paint=[('walnut', 'Body'), ('honey_wood', 'Stool & ledge'), ('cover', 'Cookbook')]),
     G.Model('pizzaOven', pizzaOven, size=(1.89, 1.9, 0.916), key='model.foodie.pizzaOven', centre=(0.005, 0.018),
             # a bake (the pizza, or any `bake` step of a recipe at it) takes the door off and lights the fire
             motions={'uses': {'bakePizza': ['door_off', 'flames'], 'bake': ['door_off', 'flames'], 'warmUp': 'flames'}},
             paint=[('plaster', 'Body'), ('worktop', 'Worktop'), ('brick', 'Brick')]),
     G.Model('chefsIsland', chefsIsland, size=(1.86, 1.128, 0.889), key='model.foodie.chefsIsland', centre=(0, 0.0154),
             # the drawer opens for the prep step's utensils (chop), the ring under the pan glows while it cooks (stir):
             # keyed by the recipe's step tags, so each step at it plays its own (A-systems-03)
             motions={'uses': {'chop': {'clip': 'drawer_open', 'hold': 2.5}, 'stir': 'burner_on'}},
             paint=[('cabinet', 'Body'), ('marble', 'Worktop'), ('oak', 'Stools')])],
    comment='Blender-built foodie pack models (tools/art/blender/families/pack_foodie.py). Listed in the '
            'base manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw '
            'call per finish group. A sprig grows, the pantry doors swing, the coffee runs, a page turns, '
            'the oven door lifts aside for the peel as the fire grows, and the island\'s drawer pulls out.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

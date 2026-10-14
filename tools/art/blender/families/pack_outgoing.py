# The outgoing pack (P14): the home party bar, the conversation sectional, the karaoke stage, the video-call
# corner, the light-up dance floor and the party photo wall, over their primitive-part placeholders
# (`tools/art/pack_outgoing.py`, `packs/outgoing/manifest.json`), which stay the fallback.
# Notes: pack_outgoing.md.
#
# Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`), with its moving parts: the bar's beer-tap lever pulls (`tap`), the sectional's seat cushions sink
# (`sink_in`), the karaoke mics rise on their stands (`mic_up`), the ring light swings up to face the caller
# (`ring_light`), the dance floor's tiles pulse up and down in a checkerboard (`tiles`, a loop) and a new
# photo grows on the board (`photo`).
#
# Run:  python3 -B tools/art/blender/build.py pack_outgoing [--only partyBar] [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402
from gamekit import motion  # noqa: E402

M = G.Palette(
    body=('#E8735A', 'satin'),       # coral: the bar's counter, the desk frame, the stage, the floor's frame
    upholstery=('#3FA9A5', 'fabric'),  # the sectional's cushions (fabric); see the per-model notes
    vinyl=('#E8735A', 'satin'),      # bar stool tops
    cushion=('#3FA9A5', 'satin'),    # the desk chair's seat and back
    frame=('#3FA9A5', 'fabric'),     # the sectional's frame, back and arms
    wood=('#8A5E3C', 'wood'),        # the back bar, the stage deck, the desk top, the photo board's frame
    oak=('#CDB08A', 'wood'),         # the bar's counter top and shelves
    oak_plain=('#CDB08A', 'satin'),  # the sectional's console (plain: the sofa takes the textured draw)
    brass=('#B8925A', 'metal'),
    chrome=('#D0D3D6', 'chrome'),
    steel=('#C9CCD0', 'metal'),      # the dance floor's gantry, mic bases
    dark=('#3A3D41', 'satin'),       # speaker cones, the ring light's stand
    speaker=('#E4E1DA', 'satin'),    # speaker cabinets
    screen=('#26305A', 'gloss'),
    lyric=('#F2F2F0', 'gloss'),
    sunny=('#F2C14E', 'satin'),
    coral=('#E8735A', 'gloss'),
    magenta=('#D64B8A', 'gloss'),
    turq=('#3FA9A5', 'gloss'),
    plum=('#6B4A7E', 'satin'),
    pouf=('#F2C14E', 'fabric'),
    cream=('#F4EDE4', 'satin'),
    pot=('#E9E6E0', 'satin'),
    leaf=('#5E8F4E', 'satin'),       # plants plain, so no foliage draw
    lime=('#9CCB4A', 'satin'),
    porcelain=('#F5F3EE', 'gloss'),
    glassy=('#DDEBEC', 'gloss'),     # glasses, plain gloss (no see-through draw)
    beer=('#E8B04A', 'gloss'),
    laptop=('#D9DADB', 'metal'),
    face=('#E8B892', 'gloss'),
    ring=('#FFF6E2', 'gloss'),
    cork=('#C9A27A', 'matte'),
    paper=('#FAFAF7', 'satin'),
    bulb=('#FFE9A8', 'gloss'),
    string=('#3A3D41', 'satin'),
    base=('#E4E1DA', 'satin'),       # the dance floor's base
    mirror=('#E8EEF2', 'chrome'),
    tile_a=('#D64B8A', 'gloss'),
    tile_b=('#3FA9A5', 'gloss'),
    tile_c=('#F2C14E', 'gloss'),
    tile_d=('#8E6BD6', 'gloss'),
    tile_e=('#58C7E8', 'gloss'),
    tile_f=('#E8735A', 'gloss'),
    bottle_a=('#2E7D5B', 'gloss'),
    bottle_b=('#B5372E', 'gloss'),
    bottle_c=('#E8B04A', 'gloss'),
    bottle_d=('#6B4A7E', 'gloss'),
    bottle_e=('#2E5B8E', 'gloss'),
    bottle_f=('#E07A2E', 'gloss'),
    wine_a=('#2E4A30', 'gloss'),
    wine_b=('#5A1F2A', 'gloss'),
    label=('#EFE5D3', 'satin'),
    photo_a=('#7FB069', 'gloss'),
    photo_b=('#8E6BD6', 'gloss'),
    photo_c=('#58C7E8', 'gloss'),
    pin_a=('#D9483B', 'gloss'),
    pin_b=('#3E8EDE', 'gloss'),
)

# By name: palette materials are made anew after each reset.
TILES = ('tile_a', 'tile_b', 'tile_c', 'tile_d', 'tile_e', 'tile_f')
BOTTLES = ('bottle_a', 'bottle_b', 'bottle_c', 'bottle_d', 'bottle_e', 'bottle_f', 'magenta', 'turq')
PHOTOS = ('coral', 'turq', 'sunny', 'magenta', 'photo_b', 'photo_a', 'photo_c', 'beer')
WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
WALL2 = -0.93  # in a 2-deep one
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


def _m(name):
    return getattr(M, name)


def _bottle(F, x, y, z, h, material, segments=10):
    G.revolve(_name('bottle'), [(0, 0), (0.036, 0), (0.037, h * 0.6), (0.027, h * 0.72), (0.012, h * 0.8),
                                (0.012, h), (0, h)], at=F.at(x, y, z), material=material, segments=segments)
    F.cyl(_name('label'), 0.0385, h * 0.22, (x, y + h * 0.32, z), M.label, verts=segments)
    F.cyl(_name('cap'), 0.014, 0.025, (x, y + h + 0.01, z), M.brass, verts=8)


def _glass(F, x, y, z, h=0.14, r=0.035, drink=None):
    """A stemmed cocktail glass standing on `y`, a drink in its bowl."""
    G.revolve(_name('glass'), [(0, 0), (r * 0.8, 0), (r * 0.8, 0.006), (0.006, 0.012), (0.005, h * 0.5),
                               (r, h), (r * 0.85, h), (0, h * 0.55)], at=F.at(x, y, z), material=M.glassy, segments=10)
    if drink:
        F.cone(_name('drink'), 0.006, r * 0.8, h * 0.36, (x, y + h * 0.75, z), drink, verts=12)


def _stool(F, x, z, h=0.74):
    F.cyl(_name('stool_top'), 0.165, 0.06, (x, h - 0.03, z), M.vinyl, verts=22, bevel=0.02, bevel_segments=2)
    F.cyl(_name('stool_ring'), 0.15, 0.03, (x, h - 0.075, z), M.brass, verts=20)
    F.cyl(_name('stool_post'), 0.022, h - 0.1, (x, (h - 0.1) / 2 + 0.02, z), M.brass, verts=10)
    G.torus(_name('stool_foot'), 0.13, 0.01, at=F.at(x, 0.28, z), material=M.brass, major_segments=18, minor_segments=6)
    G.revolve(_name('stool_base'), [(0, 0), (0.15, 0), (0.14, 0.012), (0.03, 0.03), (0, 0.03)], at=F.at(x, 0, z),
              material=M.brass, segments=20)


def _speaker(F, x, y, z, w, h, d, cone_ys, facing=1):
    """A speaker cabinet centred on (x, y, z), its cones on the face towards `facing` (+1: the front)."""
    F.box(_name('speaker'), w, h, d, (x, y, z), M.speaker, bevel=0.015)
    zf = z + facing * d / 2
    for k, cy in enumerate(cone_ys):
        r = w * (0.36 if k == 0 else 0.22)
        F.cyl(_name('surround'), r, 0.016, (x, cy, zf), M.dark, rot=(90, 0, 0), verts=18)
        F.cone(_name('cone'), r * 0.75, r * 0.2, 0.02, (x, cy, zf + facing * 0.008), M.coral if k == 0 else M.dark,
               rot=(-90 * facing, 0, 0), verts=16)


# --- the home party bar (2x1): a back bar of bottles, the front counter with a beer tap, two stools ---------
TAP = (0.24, 1.29, 0.04)   # the tap lever's pivot (x, height, depth)


def partyBar():
    F = G.GameFrame()
    with G.new_parts() as still:
        # the back bar against the wall: a walnut cabinet with doors, shelves of bottles over it
        F.box('back_cabinet', 1.82, 0.86, 0.25, (0, 0.43, WALL + 0.125), M.wood, bevel=0.01)
        for x in (-0.68, -0.23, 0.23, 0.68):
            F.box(_name('cabinet_door'), 0.42, 0.7, 0.012, (x, 0.43, WALL + 0.255), M.oak, bevel=0.005)
            F.box(_name('door_pull'), 0.012, 0.12, 0.016, (x + (0.17 if x < 0 else -0.17), 0.55, WALL + 0.266), M.brass)
        F.box('back_top', 1.84, 0.03, 0.27, (0, 0.875, WALL + 0.135), M.oak, bevel=0.006)
        for y in (1.28, 1.62):
            F.box(_name('shelf'), 1.8, 0.03, 0.18, (0, y, WALL + 0.09), M.oak, bevel=0.005)
            F.box(_name('led'), 1.78, 0.014, 0.01, (0, y - 0.022, WALL + 0.17), M.sunny)
        for x in (-0.89, 0.89):
            F.box(_name('upright'), 0.03, 0.78, 0.16, (x, 1.43, WALL + 0.08), M.brass, bevel=0.006)
        F.box('mirror_back', 1.76, 0.62, 0.008, (0, 1.36, WALL + 0.004), M.plum)
        for i, x in enumerate((-0.76, -0.64, -0.52, -0.4, 0.34, 0.46, 0.6, 0.74)):
            _bottle(F, x, 1.635, WALL + 0.09, 0.26 + 0.04 * (i % 3), _m(BOTTLES[i]))
        for x in (-0.68, -0.46, 0.42, 0.68):
            _glass(F, x, 1.295, WALL + 0.09, h=0.13)
        for i, x in enumerate((-0.1, 0.0, 0.1, 0.2)):
            _bottle(F, x, 0.89, WALL + 0.13, 0.3 - 0.02 * (i % 2), M.wine_a if i % 2 else M.wine_b)
        # the front counter: a coral front of fluted panels, an oak top, a brass foot rail
        F.box('counter', 1.5, 0.96, 0.24, (0, 0.48, 0.0), M.body, bevel=0.025)
        for x in (-0.55, -0.18, 0.18, 0.55):
            for k in range(3):
                F.cyl(_name('flute'), 0.045, 0.84, (x + (k - 1) * 0.09, 0.48, 0.115), M.body, verts=8,
                      scale=(1, 0.4, 1))
        F.box('counter_top', 1.62, 0.05, 0.38, (0, 0.985, 0.06), M.oak, bevel=0.012)
        G.rod('foot_rail', F.at(-0.72, 0.2, 0.17), F.at(0.72, 0.2, 0.17), 0.015, material=M.brass, verts=10)
        for x in (-0.6, 0.6):
            G.rod(_name('rail_post'), F.at(x, 0.08, 0.13), F.at(x, 0.2, 0.17), 0.012, material=M.brass, verts=8)
        # on the counter: the beer tap, drinks, a shaker, a bowl of limes
        ty = 1.01
        F.box('drip_tray', 0.16, 0.02, 0.1, (TAP[0], ty + 0.01, 0.13), M.chrome, bevel=0.004)
        F.cyl('tap_tower', 0.035, 0.24, (TAP[0], ty + 0.12, 0.0), M.chrome, verts=16)
        F.sphere('tap_dome', 0.04, (TAP[0], ty + 0.24, 0.0), M.chrome, segments=14, rings=8)
        G.rod('tap_arm', F.at(TAP[0], TAP[1] - 0.01, 0.0), F.at(TAP[0], TAP[1] - 0.01, TAP[2] + 0.02), 0.014,
              material=M.chrome, verts=10)
        F.cone('tap_spout', 0.012, 0.007, 0.08, (TAP[0], TAP[1] - 0.06, TAP[2] + 0.03), M.chrome, verts=10)
        G.revolve('pint', [(0, 0), (0.03, 0), (0.036, 0.14), (0.032, 0.14), (0, 0.02)], at=F.at(TAP[0], ty + 0.02, 0.13),
                  material=M.glassy, segments=14)
        F.cone('pint_beer', 0.026, 0.031, 0.1, (TAP[0], ty + 0.08, 0.13), M.beer, verts=14)
        _glass(F, -0.42, ty, 0.1, drink=M.magenta)
        _glass(F, 0.02, ty, 0.15, drink=M.sunny)
        F.cone('shaker', 0.04, 0.032, 0.18, (0.5, ty + 0.09, 0.04), M.chrome, verts=14)
        F.sphere('shaker_cap', 0.032, (0.5, ty + 0.19, 0.04), M.chrome, segments=12, rings=6)
        G.revolve('bowl', [(0, 0), (0.05, 0), (0.09, 0.06), (0.084, 0.06), (0.045, 0.008), (0, 0.008)],
                  at=F.at(-0.15, ty, 0.04), material=M.porcelain, segments=16)
        for dx, dz, m in ((-0.03, 0.0, M.lime), (0.03, 0.02, M.lime), (0.0, -0.03, M.sunny)):
            F.sphere(_name('lime'), 0.03, (-0.15 + dx, ty + 0.06, 0.04 + dz), m, segments=10, rings=6)
        _stool(F, -0.5, 0.28)
        _stool(F, 0.5, 0.28)
    G.join(still, 'body', origin=(0, 0, 0))
    # The tap's lever: a tall handle on the faucet, its origin on the pivot: `tap` pulls it towards the front.
    with G.new_parts() as lever:
        F.cyl('lever_collar', 0.016, 0.03, (TAP[0], TAP[1] + 0.015, TAP[2]), M.chrome, verts=10)
        F.cone('lever_handle', 0.014, 0.022, 0.15, (TAP[0], TAP[1] + 0.1, TAP[2]), M.body, verts=12)
        F.sphere('lever_knob', 0.024, (TAP[0], TAP[1] + 0.18, TAP[2]), M.body, segments=12, rings=6)
    lever = G.hinged(G.join(lever, 'lever'), F.at(*TAP))
    objs = G.settle(centre=False)
    G.swing(lever, 'tap', 40, seconds=0.5, axis='X')
    return objs


# --- the conversation sectional (2x2): an L-sofa, a console behind it, a pouf in the free corner ----------
def conversationSectional():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('base', 1.9, 0.28, 0.86, (0, 0.18, -0.02), M.frame, bevel=0.05, bevel_segments=3)
        F.box('chaise_base', 0.56, 0.28, 0.54, (0.66, 0.18, 0.66), M.frame, bevel=0.05, bevel_segments=3)
        F.box('back', 1.9, 0.4, 0.2, (0, 0.6, -0.36), M.frame, bevel=0.07, bevel_segments=3)
        F.box('arm_l', 0.18, 0.3, 1.38, (-0.86, 0.47, 0.06), M.frame, bevel=0.07, bevel_segments=3)
        F.box('arm_r', 0.16, 0.24, 1.36, (0.86, 0.44, 0.25), M.frame, bevel=0.06, bevel_segments=3)
        for x in (-0.42, 0.3):
            F.box(_name('back_cushion'), 0.68, 0.36, 0.16, (x, 0.68, -0.2), M.upholstery, rot=(-12, 0, 0),
                  bevel=0.06, bevel_segments=3)
        F.box('pillow_l', 0.34, 0.3, 0.1, (-0.6, 0.66, -0.08), M.sunny, rot=(-15, 0, -20), bevel=0.045,
              bevel_segments=3)
        F.box('pillow_r', 0.32, 0.3, 0.1, (0.6, 0.66, -0.06), M.body, rot=(-15, 0, 18), bevel=0.045, bevel_segments=3)
        F.box('pillow_c', 0.28, 0.26, 0.1, (0.74, 0.6, 0.32), M.cream, rot=(-10, 0, 80), bevel=0.045, bevel_segments=3)
        for x, z in ((-0.86, -0.4), (-0.86, 0.36), (0.86, -0.4), (0.86, 0.88), (0.42, 0.88)):
            F.cone(_name('foot'), 0.025, 0.02, 0.04, (x, 0.02, z), M.brass, verts=10)
        # the console behind it: oak top and shelf on brass legs, a lamp, a plant, books and magazines
        F.box('console_top', 1.8, 0.04, 0.32, (0, 0.82, -0.75), M.oak_plain, bevel=0.01)
        F.box('console_shelf', 1.76, 0.03, 0.28, (0, 0.2, -0.75), M.oak_plain, bevel=0.006)
        for x in (-0.86, 0.86):
            for z in (-0.88, -0.62):
                F.cyl(_name('console_leg'), 0.014, 0.8, (x, 0.4, z), M.brass, verts=8)
        G.revolve('lamp_base', [(0, 0), (0.06, 0), (0.11, 0.1), (0.08, 0.22), (0.02, 0.3), (0, 0.3)],
                  at=F.at(-0.6, 0.84, -0.76), material=M.porcelain, segments=18)
        G.revolve('lamp_shade', [(0.17, 0), (0.135, 0.24), (0.13, 0.24), (0.165, 0.005)], at=F.at(-0.6, 1.18, -0.76),
                  material=M.cream, segments=24)
        G.revolve('plant_pot', [(0, 0), (0.07, 0), (0.09, 0.18), (0.082, 0.18), (0, 0.17)], at=F.at(0.55, 0.84, -0.76),
                  material=M.pot, segments=16)
        for k in range(6):
            a = 60 * k
            F.sphere(_name('leaf'), 0.09, (0.55 + 0.06 * G.cosd(a), 1.1 + 0.04 * (k % 2), -0.76 + 0.05 * G.sind(a)),
                     M.leaf, segments=10, rings=6)
        F.sphere('leaf_top', 0.1, (0.55, 1.2, -0.76), M.leaf, segments=10, rings=6)
        F.box('book_a', 0.3, 0.06, 0.2, (0.1, 0.87, -0.76), M.body)
        F.box('book_b', 0.28, 0.05, 0.19, (0.1, 0.925, -0.76), M.sunny, rot=(0, 0, 8))
        for i, m in enumerate((M.turq, M.coral, M.sunny, M.plum)):
            F.box(_name('magazine'), 0.24, 0.03, 0.18, (0.02 * i, 0.23 + 0.035 * i, -0.75), m, rot=(0, 0, 6 * i))
        # the pouf with a tray of drinks in the free front-left corner
        G.revolve('pouf', [(0, 0), (0.2, 0), (0.23, 0.06), (0.235, 0.18), (0.23, 0.3), (0.2, 0.36), (0, 0.36)],
                  at=F.at(-0.52, 0, 0.7), material=M.pouf, segments=22)
        F.cyl('tray', 0.15, 0.02, (-0.52, 0.37, 0.7), M.oak_plain, verts=22)
        F.cyl('cup_a', 0.035, 0.09, (-0.46, 0.425, 0.66), M.porcelain, verts=12)
        F.cyl('cup_b', 0.035, 0.09, (-0.58, 0.425, 0.74), M.magenta, verts=12)
    G.join(still, 'body', origin=(0, 0, 0))
    # The seat cushions (top 0.45, where residents sit), origin on their underside: `sink_in` presses them.
    with G.new_parts() as seats:
        F.box('seat_l', 0.84, 0.13, 0.66, (-0.5, 0.385, 0.06), M.upholstery, bevel=0.05, bevel_segments=3)
        F.box('seat_r', 0.64, 0.13, 0.66, (0.32, 0.385, 0.06), M.upholstery, bevel=0.05, bevel_segments=3)
        F.box('seat_chaise', 0.5, 0.13, 0.54, (0.66, 0.385, 0.65), M.upholstery, bevel=0.05, bevel_segments=3)
    seats = G.hinged(G.join(seats, 'seats'), F.at(0, 0.32, 0.06))
    objs = G.settle(centre=False)
    G.squash(seats, 'sink_in', (1, 1, 0.8), seconds=0.6)
    return objs


# --- the karaoke stage (2x1): a low stage, a lyrics screen, two speakers and two mic stands -----------------
MICS = (-0.36, 0.36)
MIC_Z = 0.12
MIC_RISE = 0.2


def karaokeStage():
    F = G.GameFrame()
    zc, d = (WALL + 0.45) / 2, 0.45 - WALL
    with G.new_parts() as still:
        F.box('stage', 1.9, 0.13, d, (0, 0.065, zc), M.body, bevel=0.02)
        F.box('deck', 1.86, 0.016, d - 0.04, (0, 0.138, zc), M.wood, bevel=0.004)
        F.box('light_strip', 1.86, 0.025, 0.012, (0, 0.07, 0.448), M.magenta)
        # the lyrics screen on its stand at the back
        F.box('screen_case', 0.96, 0.56, 0.05, (0, 1.35, -0.34), M.speaker, bevel=0.015)
        F.box('screen', 0.9, 0.5, 0.004, (0, 1.35, -0.314), M.screen)
        for i, (w, y) in enumerate(((0.6, 1.47), (0.7, 1.38), (0.5, 1.29), (0.62, 1.2))):
            F.box(_name('lyric'), w, 0.032, 0.003, (0, y, -0.311), M.sunny if i == 1 else M.lyric)
        F.box('screen_post', 0.05, 1.0, 0.05, (0, 0.6, -0.39), M.steel, bevel=0.008)
        F.box('screen_foot', 0.5, 0.03, 0.16, (0, 0.16, -0.36), M.steel, bevel=0.008)
        for s in (-1, 1):
            _speaker(F, s * 0.74, 0.47, -0.24, 0.34, 0.64, 0.3, (0.38, 0.62))
            # floor spots at the stage's front corners, angled up at the singers
            F.cone(_name('spot'), 0.05, 0.065, 0.12, (s * 0.83, 0.22, 0.33), M.dark, rot=(-35, 0, 0), verts=14)
            F.cyl(_name('spot_lens'), 0.058, 0.008, (s * 0.83, 0.272, 0.364), M.sunny, rot=(-35, 0, 0), verts=14)
        for x in MICS:                                  # the mic stands' bases and lower tubes
            G.revolve(_name('mic_base'), [(0, 0), (0.11, 0), (0.1, 0.015), (0.02, 0.03), (0, 0.03)],
                      at=F.at(x, 0.146, MIC_Z), material=M.steel, segments=18)
            F.cyl(_name('mic_tube'), 0.014, 0.9, (x, 0.146 + 0.45, MIC_Z), M.steel, verts=10)
            F.cyl(_name('mic_clutch'), 0.02, 0.04, (x, 1.04, MIC_Z), M.dark, verts=10)
    G.join(still, 'body', origin=(0, 0, 0))
    # The upper tubes, booms and mics of both stands, lowered at rest: `mic_up` raises them 20 cm.
    with G.new_parts() as mics:
        for i, x in enumerate(MICS):
            top = 1.33 - MIC_RISE
            F.cyl(_name('mic_upper'), 0.01, 0.5, (x, top - 0.25 + 0.04, MIC_Z), M.chrome, verts=10)
            G.rod(_name('mic_boom'), F.at(x, top, MIC_Z), F.at(x, top + 0.07, MIC_Z + 0.08), 0.01, material=M.chrome,
                  verts=8)
            cap = G.rod(_name('mic_body'), F.at(x, top + 0.06, MIC_Z + 0.07), F.at(x, top + 0.13, MIC_Z + 0.16), 0.022,
                        material=M.magenta if i == 0 else M.turq, verts=12)
            F.sphere(_name('mic_head'), 0.038, (x, top + 0.15, MIC_Z + 0.19), M.steel, segments=12, rings=8)
    mics = G.hinged(G.join(mics, 'mics'), F.at(0, 1.04, MIC_Z))
    objs = G.settle(centre=False)
    G.slide(mics, 'mic_up', (0, 0, MIC_RISE), seconds=0.8)
    return objs


# --- the video-call corner (1x1): the caller sits in the middle facing the desk, a ring light beside the laptop
RING = (0.27, 1.21, 0.34)   # the ring light's middle and pivot (x, height, depth)
RING_R = 0.11
RING_TILT = 60              # tipped back at rest, its face to the ceiling


def videoCallNook():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('desk_top', 0.84, 0.03, 0.27, (0, 0.74, 0.31), M.wood, bevel=0.008)
        F.box('desk_apron', 0.78, 0.06, 0.22, (0, 0.695, 0.31), M.body, bevel=0.006)
        for x in (-0.39, 0.39):
            for z in (0.21, 0.41):
                F.cone(_name('desk_leg'), 0.012, 0.018, 0.67, (x, 0.335, z), M.body, verts=10)
        # the laptop, open towards the caller (-depth), a friend on its screen
        F.box('laptop_base', 0.34, 0.015, 0.23, (0, 0.763, 0.29), M.laptop, bevel=0.005)
        F.box('laptop_lid', 0.34, 0.22, 0.012, (0, 0.88, 0.41), M.laptop, rot=(-12, 0, 0), bevel=0.005)
        F.box('laptop_screen', 0.31, 0.19, 0.003, (0, 0.88, 0.402), M.screen, rot=(-12, 0, 0))
        F.sphere('friend', 0.045, (0, 0.875, 0.398), M.face, scale=(1, 0.08, 1.1), rot=(-12, 0, 0), segments=12, rings=8)
        # the ring light's stand: a foot, a pole and the yoke the ring turns in
        x, y, z = RING
        F.cyl('ring_foot', 0.06, 0.015, (x, 0.763, z), M.dark, verts=16)
        F.cyl('ring_pole', 0.01, y - RING_R - 0.77, (x, (0.77 + y - RING_R) / 2, z), M.dark, verts=8)
        yoke = RING_R + 0.03
        G.rod('yoke_base', F.at(x - yoke, y - RING_R - 0.02, z), F.at(x + yoke, y - RING_R - 0.02, z), 0.008,
              material=M.dark, verts=8)
        for s in (-1, 1):
            G.rod(_name('yoke_arm'), F.at(x + s * yoke, y - RING_R - 0.02, z), F.at(x + s * yoke, y, z), 0.008,
                  material=M.dark, verts=8)
        # a mug, a notebook, a little plant
        G.revolve('mug', [(0, 0), (0.034, 0), (0.037, 0.09), (0.032, 0.09), (0.03, 0.01), (0, 0.01)],
                  at=F.at(-0.28, 0.755, 0.24), material=M.coral, segments=14)
        F.box('notebook', 0.14, 0.012, 0.2, (-0.3, 0.761, 0.33), M.sunny, rot=(0, 0, -10))
        G.revolve('plant_pot', [(0, 0), (0.04, 0), (0.055, 0.1), (0.05, 0.1), (0, 0.09)], at=F.at(-0.33, 0.755, 0.4),
                  material=M.pot, segments=14)
        for k in range(4):
            a = 90 * k + 45
            F.sphere(_name('leaf'), 0.05, (-0.33 + 0.03 * G.cosd(a), 0.88, 0.4 + 0.03 * G.sind(a)), M.leaf,
                     segments=8, rings=6)
        # the chair: the seat (top 0.45) centred on the caller, a back, four oak legs
        F.box('chair_seat', 0.44, 0.08, 0.42, (0, 0.41, -0.02), M.cushion, bevel=0.03, bevel_segments=3)
        F.box('chair_back', 0.44, 0.38, 0.07, (0, 0.7, -0.26), M.cushion, rot=(-8, 0, 0), bevel=0.03, bevel_segments=3)
        F.box('chair_shell', 0.42, 0.04, 0.4, (0, 0.355, -0.02), M.wood, bevel=0.008)
        for lx in (-0.18, 0.18):
            for lz, tilt in ((0.15, -6), (-0.19, 6)):
                F.cone(_name('chair_leg'), 0.016, 0.012, 0.34, (lx, 0.17, lz), M.wood, rot=(tilt, 0, 0), verts=8)
            G.rod(_name('back_post'), F.at(lx, 0.36, -0.22), F.at(lx, 0.6, -0.26), 0.012, material=M.wood, verts=8)
    G.join(still, 'body', origin=(0, 0, 0))
    # The ring and its phone clip, built facing the caller, pivoting on the yoke: tipped back to rest, then
    # `ring_light` swings it up to face them.
    with G.new_parts() as ring:
        G.torus('ring', RING_R, 0.018, at=F.at(x, y, z), material=M.ring, rot=(90, 0, 0), major_segments=28,
                minor_segments=8)
        G.torus('ring_back', RING_R, 0.02, at=F.at(x, y, z + 0.008), material=M.dark, rot=(90, 0, 0),
                major_segments=28, minor_segments=6)
        G.rod('phone_arm', F.at(x, y - RING_R, z), F.at(x, y, z), 0.005, material=M.dark, verts=6)
        F.box('phone', 0.06, 0.11, 0.008, (x, y, z - 0.006), M.dark, bevel=0.003)
        for s in (-1, 1):
            G.rod(_name('pivot'), F.at(x + s * RING_R, y, z), F.at(x + s * yoke, y, z), 0.008, material=M.dark, verts=8)
    ring = G.hinged(G.join(ring, 'ring'), F.at(x, y, z))
    G.turn(ring, RING_TILT, 'X')
    objs = G.settle(centre=False)
    G.swing(ring, 'ring_light', -RING_TILT, seconds=0.7, axis='X')
    return objs


# --- the light-up dance floor (2x2): sixteen glowing tiles, a mirror ball on a gantry, two speaker towers ---
TILE, PITCH, TILE_Y = 0.43, 0.45, 0.06
LIFT = 0.025


def danceFloor():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('base', 1.88, 0.06, 1.88, (0, 0.03, 0), M.base, bevel=0.01)
        F.box('grid', 1.82, 0.004, 1.82, (0, 0.062, 0), M.dark)
        for s in (-1, 1):
            F.box(_name('post'), 0.06, 2.24, 0.06, (s * 0.9, 1.18, 0.0), M.steel, bevel=0.01)
            F.box(_name('post_foot'), 0.12, 0.03, 0.3, (s * 0.9, 0.075, 0.0), M.steel, bevel=0.008)
            _speaker(F, s * 0.78, 0.5, -0.8, 0.24, 0.88, 0.24, (0.62, 0.84))
            F.cone(_name('spot'), 0.05, 0.07, 0.14, (s * 0.7, 2.2, 0.0), M.dark, rot=(0, s * 40, 0), verts=14)
            F.cyl(_name('spot_lens'), 0.062, 0.008, (s * 0.655, 2.15, 0.0), M.magenta if s < 0 else M.turq,
                  rot=(0, s * 40, 0), verts=14)
        F.box('beam', 1.86, 0.06, 0.06, (0, 2.3, 0.0), M.steel, bevel=0.01)
        F.cyl('ball_cord', 0.005, 0.25, (0, 2.15, 0.0), M.chrome, verts=6)
        F.sphere('mirror_ball', 0.17, (0, 1.88, 0.0), M.mirror, segments=16, rings=10, smooth=False)
    G.join(still, 'body', origin=(0, 0, 0))
    # The tiles in two checkerboard parts, origins on the floor: `tiles` lifts one set, then the other.
    groups = []
    for parity in (0, 1):
        with G.new_parts() as tiles:
            for i in range(4):
                for j in range(4):
                    if (i + j) % 2 != parity:
                        continue
                    m = _m(TILES[(i * 3 + j * 2) % len(TILES)])
                    x, z = -1.5 * PITCH + PITCH * i, -1.5 * PITCH + PITCH * j
                    F.box(_name('tile'), TILE, 0.014, TILE, (x, TILE_Y + 0.007, z), m, bevel=0.004)
        groups.append(G.hinged(G.join(tiles, f'tiles_{"ab"[parity]}'), F.at(0, TILE_Y, 0)))
    objs = G.settle(centre=False)
    end = motion.last_frame(1.2)
    half = (end + 1) // 2
    for k, part in enumerate(groups):
        z0 = part.location[2]
        a, b = (1, half) if k == 0 else (half, end)
        keys = [(1, z0), (a, z0), ((a + b) // 2, z0 + LIFT), (b, z0), (end, z0)]
        keys = [kv for n, kv in enumerate(keys) if n == 0 or kv[0] != keys[n - 1][0]]   # no two keys on one frame
        motion._clip(part, 'tiles', 'location', 2, keys)
    motion._peak(groups[0], 'tiles', (1 + half) // 2)
    return objs


# --- the party photo wall (1x1): a cork board on legs, snapshots, string lights, balloons, a gift ---------
BOARD_Z = WALL + 0.07
NEW_PHOTO = (0.25, 0.88, 4)   # (x, height, tilt) of the photo that grows


def _photo(F, x, y, z, tilt, material, pin):
    F.box(_name('print'), 0.17, 0.2, 0.006, (x, y, z), M.paper, rot=(0, tilt, 0))
    F.box(_name('picture'), 0.14, 0.13, 0.004, (x - 0.02 * math.sin(math.radians(tilt)), y + 0.02, z + 0.004),
          material, rot=(0, tilt, 0))
    F.sphere(_name('pin'), 0.013, (x - 0.09 * math.sin(math.radians(tilt)), y + 0.09, z + 0.008), pin,
             scale=(1, 1, 0.6), segments=8, rings=6)


def photoWall():
    F = G.GameFrame()
    zb = BOARD_Z
    with G.new_parts() as still:
        F.box('cork', 0.82, 1.0, 0.03, (0, 1.2, zb), M.cork)
        for y in (0.685, 1.715):
            F.box(_name('rail'), 0.88, 0.05, 0.06, (0, y, zb), M.wood, bevel=0.01)
        for x in (-0.42, 0.42):
            F.box(_name('leg'), 0.05, 1.74, 0.05, (x, 0.87, zb), M.wood, bevel=0.01)
            F.box(_name('foot'), 0.06, 0.04, 0.34, (x, 0.02, WALL + 0.17), M.wood, bevel=0.01)
        spots = ((-0.27, 1.45, 6), (0.0, 1.5, -4), (0.26, 1.44, 8), (-0.24, 1.15, -7), (0.03, 1.18, 3),
                 (0.28, 1.12, -5), (-0.25, 0.86, 4), (0.0, 0.88, -8))
        for i, (x, y, r) in enumerate(spots):
            _photo(F, x, y, zb + 0.018, r, _m(PHOTOS[i]), (M.pin_a, M.pin_b, M.sunny)[i % 3])
        # string lights draped across the top
        pts = [(-0.42 + 0.84 * t / 8, 1.66 - 0.09 * math.sin(math.pi * t / 8)) for t in range(9)]
        for (x0, y0), (x1, y1) in zip(pts, pts[1:]):
            G.rod(_name('wire'), F.at(x0, y0, zb + 0.04), F.at(x1, y1, zb + 0.04), 0.004, material=M.string, verts=6)
        for x, y in pts[1:-1]:
            F.sphere(_name('bulb'), 0.02, (x, y - 0.03, zb + 0.05), M.bulb, scale=(1, 1, 1.3), segments=10, rings=6)
        # three balloons tied to the top rail
        for x, y, z, m in ((0.3, 1.92, -0.25, M.coral), (0.32, 2.04, -0.18, M.turq), (0.2, 2.0, -0.15, M.sunny)):
            F.sphere(_name('balloon'), 0.1, (x, y, z), m, scale=(1, 1, 1.2), segments=14, rings=10)
            F.cone(_name('knot'), 0.012, 0.0, 0.02, (x, y - 0.13, z), m, rot=(180, 0, 0), verts=8)
            G.rod(_name('balloon_string'), F.at(x, y - 0.14, z), F.at(0.36, 1.7, zb + 0.03), 0.003, material=M.paper,
                  verts=5)
        # a gift on the floor in front
        F.box('gift', 0.3, 0.2, 0.22, (0.15, 0.1, -0.05), M.sunny, bevel=0.006)
        F.box('ribbon_a', 0.04, 0.204, 0.224, (0.15, 0.101, -0.05), M.magenta)
        F.box('ribbon_b', 0.304, 0.204, 0.04, (0.15, 0.101, -0.05), M.magenta)
        for s in (-1, 1):
            F.sphere(_name('bow'), 0.04, (0.15 + s * 0.035, 0.215, -0.05), M.magenta, scale=(1.2, 0.6, 0.7),
                     rot=(0, 0, s * 30), segments=10, rings=6)
    G.join(still, 'body', origin=(0, 0, 0))
    # A new snapshot in the bottom row, origin on its middle and built at full size, then shrunk to nothing:
    # `photo` grows it onto the board.
    px, py, pr = NEW_PHOTO
    with G.new_parts() as photo:
        _photo(F, px, py, zb + 0.018, pr, M.magenta, M.pin_b)
    photo = G.hinged(G.join(photo, 'photo'), F.at(px, py, zb + 0.018))
    photo.scale[0] = photo.scale[2] = 0.001
    objs = G.settle(centre=False)
    G.grow(photo, 'photo', seconds=0.8, axis='X')
    G.grow(photo, 'photo', seconds=0.8, axis='Z')
    return objs


FAMILY = G.Family(
    'pack_outgoing', M,
    [G.Model('partyBar', partyBar, size=(1.84, 2.0, 0.875), key='model.outgoing.partyBar',
             centre=(0, 0.008), motions={'uses': {'mixDrinks': 'tap', 'soda': 'tap'}, 'hold': 2.0},
             paint=[('body', 'Body'), ('vinyl', 'Upholstery'), ('wood', 'Back bar')]),
     G.Model('conversationSectional', conversationSectional, size=(1.9, 1.42, 1.86),
             key='model.outgoing.conversationSectional', centre=(0, 0.0), motions={'uses': {'hangOut': 'sink_in'}},
             paint=[('frame', 'Body'), ('upholstery', 'Upholstery')]),
     G.Model('karaokeStage', karaokeStage, size=(1.9, 1.63, 0.894), key='model.outgoing.karaokeStage',
             centre=(0, 0.007), motions={'uses': {'practise': 'mic_up', 'perform': 'mic_up'}},
             paint=[('body', 'Body'), ('speaker', 'Speakers')]),
     G.Model('videoCallNook', videoCallNook, size=(0.84, 1.283, 0.788), key='model.outgoing.videoCallNook',
             centre=(0, 0.077), motions={'uses': {'call': 'ring_light'}},
             paint=[('body', 'Body'), ('cushion', 'Upholstery'), ('wood', 'Wood')]),
     G.Model('danceFloor', danceFloor, size=(1.92, 2.33, 1.88), key='model.outgoing.danceFloor',
             centre=(0, 0.0), motions={'loops': {'dance': 'tiles'}},
             paint=[('base', 'Body'), ('speaker', 'Speakers')]),
     G.Model('photoWall', photoWall, size=(0.9, 2.16, 0.492), key='model.outgoing.photoWall',
             centre=(0, -0.184), motions={'uses': {'reminisce': 'photo'}},
             paint=[('wood', 'Body'), ('cork', 'Board')])],
    comment='Blender-built outgoing pack models (tools/art/blender/families/pack_outgoing.py). Replaces the '
            'primitive-part placeholders of packs/outgoing by key; the placeholder is kept as a fallback if a file '
            'fails to load, and the materials map onto the game\'s finishes so a model costs one draw call per '
            'finish group. Each moves while it is used: the bar\'s tap lever pulls, the sectional\'s cushions sink, '
            'the mics rise, the ring light swings up, the dance floor\'s tiles pulse and a new photo grows.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

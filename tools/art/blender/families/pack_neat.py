# The neat pack (P13): the steam mop dock, the stacked laundry centre, the rainfall steam shower, the
# label-and-sort organizer desk, the robot vacuum and its dock and the tidy entry console.
# Notes: pack_neat.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`), one moving part each: the mop leaves its dock for the hands (`mop_off`), the washer's door swings open
# (`door_open`), the rain falls (`water_on`), a bin slides out of the desk's hutch (`drawer_open`), the robot
# rolls out of its dock and turns (`vacuum_out`) and the console's key tray drawer slides out (`tray`).
#
# Run:  python3 -B tools/art/blender/build.py pack_neat [--only steamMopDock] [--preview]
import os
import random
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    body=('#F4F3EF', 'satin'),       # crisp white: docks, cabinets, cubbies, frames; the main part
    appliance=('#F7F7F5', 'gloss'),  # the washer and dryer, the robot's dock
    trim=('#D5D7D8', 'satin'),
    birch=('#E3CFA8', 'wood'),       # textured birch: the desk, the console
    birch_plain=('#E3CFA8', 'satin'),
    oak=('#CDB08A', 'satin'),
    teak=('#B07A4A', 'satin'),       # the shower bench, plain
    chrome=('#D0D3D6', 'chrome'),
    steel=('#9EA3A8', 'metal'),
    brass=('#C9A24A', 'metal'),
    dark=('#3A4250', 'satin'),
    screen=('#2E3A46', 'gloss'),
    screen_glow=('#7FC8E8', 'gloss'),
    porthole=('#C9E0E8', 'gloss'),   # porthole and bin windows, plain (no glass draw)
    drum=('#8C949B', 'satin'),
    teal=('#4E7A7F', 'gloss'),
    led=('#6CC27A', 'gloss'),
    tile=('#F1F0EC', 'satin'),
    joint=('#D6D4CE', 'satin'),
    glass=('#DCE8EA', 'glass'),
    water=('#BFE3F2', 'gloss'),
    pad=('#F7F7F7', 'satin'),
    rug=('#D5D7D8', 'fabric'),
    rug_inner=('#E6E5E0', 'fabric'),
    paper=('#F6F3EA', 'matte'),
    ledger=('#3E6E73', 'satin'),
    coral=('#E8846B', 'gloss'),
    wicker=('#D8C29C', 'satin'),
    wicker_b=('#CDB48A', 'satin'),
    green=('#6FA35A', 'satin'),
    stem=('#5E8F4E', 'satin'),
    mirror=('#DCE6EA', 'chrome'),
    shoe_a=('#B33A32', 'satin'),
    shoe_b=('#3E6E73', 'satin'),
    shoe_c=('#E9D58A', 'satin'),
    # soft pastels: towels and shirts (fabric), bins and bottles (satin, gloss)
    towel_a=('#9FD3C7', 'fabric'), towel_b=('#F2C6A0', 'fabric'), towel_c=('#B8C9E8', 'fabric'),
    towel_d=('#E9D58A', 'fabric'), towel_e=('#D9B8D6', 'fabric'), towel_f=('#A8D5A2', 'fabric'),
    towel_w=('#F7F7F7', 'fabric'),
    pastel_a=('#9FD3C7', 'satin'), pastel_b=('#F2C6A0', 'satin'), pastel_c=('#B8C9E8', 'satin'),
    pastel_d=('#E9D58A', 'satin'), pastel_e=('#D9B8D6', 'satin'), pastel_f=('#A8D5A2', 'satin'),
    bottle_a=('#9FD3C7', 'gloss'), bottle_b=('#B8C9E8', 'gloss'), bottle_c=('#F2C6A0', 'gloss'),
)

TOWELS = ('towel_a', 'towel_b', 'towel_c', 'towel_d', 'towel_e', 'towel_f')   # by name: made anew after each reset
PASTELS = ('pastel_a', 'pastel_b', 'pastel_c', 'pastel_d', 'pastel_e', 'pastel_f')
WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


def _m(name):
    return getattr(M, name)


def _towels(F, x, z, y, n, w=0.26, d=0.2, names=TOWELS):
    """`n` folded towels stacked on `y`; returns the stack's top."""
    for i in range(n):
        F.box(_name('towel'), w, 0.045, d, (x, y + 0.0225 + i * 0.046, z), _m(names[i % len(names)]), bevel=0.016,
              bevel_segments=2)
    return y + n * 0.046


def _bottle(F, x, z, y, material, h=0.22, d=0.08):
    r = d / 2
    G.revolve(_name('bottle'), [(0, 0), (r, 0), (r, h * 0.72), (r * 0.44, h * 0.88), (r * 0.44, h), (0, h)],
              at=F.at(x, y, z), material=material, segments=12)
    F.cyl(_name('bottle_cap'), r * 0.46, 0.03, (x, y + h + 0.015, z), M.body, verts=10)


# --- the steam mop dock (1x1): a slim white tower against the wall, the mop docked before it ----------------
MOP = (0.0, 0.04, -0.07)    # the mop head's foot (x, height, depth): it pivots there


def steamMopDock():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('tower', 0.42, 1.25, 0.2, (0, 0.625, WALL + 0.1), M.body, bevel=0.03, bevel_segments=2)
        F.box('panel', 0.34, 0.26, 0.012, (0, 1.0, WALL + 0.205), M.porthole, bevel=0.008)
        for x, w, y in ((-0.08, 0.1, 1.06), (0.04, 0.16, 1.02), (-0.02, 0.2, 0.98)):
            F.box(_name('panel_line'), w, 0.012, 0.004, (x, y, WALL + 0.212), M.teal)
        F.sphere('led', 0.012, (0.12, 0.92, WALL + 0.212), M.led, segments=8, rings=6)
        F.box('base_plate', 0.44, 0.04, 0.36, (0, 0.02, WALL + 0.18), M.trim, bevel=0.01)
        F.box('clip', 0.08, 0.04, 0.06, (MOP[0], 1.1, WALL + 0.23), M.trim, bevel=0.01)
        # the refill shelf with three bottles
        F.box('shelf', 0.38, 0.02, 0.1, (0, 0.7, WALL + 0.25), M.birch_plain, bevel=0.005)
        for x, m in ((-0.13, M.bottle_a), (-0.07, M.bottle_b), (0.13, M.bottle_a)):
            _bottle(F, x, WALL + 0.25, 0.71, m, h=0.14, d=0.05)
        # a bucket and spare pads on the floor
        G.revolve('bucket', [(0, 0), (0.11, 0), (0.14, 0.26), (0.13, 0.26), (0.1, 0.015), (0, 0.015)],
                  at=F.at(-0.26, 0, 0.14), material=M.pastel_a, segments=18)
        G.torus('bucket_handle', 0.13, 0.008, at=F.at(-0.26, 0.27, 0.14), material=M.trim, rot=(70, 0, 0),
                major_segments=16, minor_segments=4)
        F.box('spare_pad', 0.22, 0.02, 0.14, (0.27, 0.01, 0.16), M.towel_w, bevel=0.008)
        F.box('spare_pad_b', 0.22, 0.02, 0.14, (0.27, 0.03, 0.16), M.towel_c, bevel=0.008, rot=(0, 0, 8))
    G.join(still, 'body', origin=(0, 0, 0))
    # The mop: pole, grip, water tank, head and pad, its origin at its head's foot: hidden while it's used
    # (`mop_off`), the game drawing it in the resident's hands (content `held`), pushed over the floor.
    x, y, z = MOP
    with G.new_parts() as mop:
        F.box('mop_head', 0.34, 0.05, 0.13, (x, y + 0.005, z), M.trim, bevel=0.02, bevel_segments=2)
        F.box('mop_pad', 0.32, 0.02, 0.12, (x, 0.012, z), M.pad, bevel=0.006)
        F.cyl('mop_neck', 0.02, 0.06, (x, y + 0.06, z), M.dark, verts=10)
        G.rod('mop_pole', F.at(x, y + 0.08, z), F.at(x, 1.12, WALL + 0.26), 0.014, material=M.trim, verts=10)
        F.box('mop_grip', 0.12, 0.045, 0.04, (x, 1.15, WALL + 0.26), M.teal, bevel=0.018)
        F.box('mop_tank', 0.1, 0.28, 0.08, (x, 0.32, z - 0.035), M.teal, rot=(-6, 0, 0), bevel=0.03, bevel_segments=2)
    mop = G.hinged(G.join(mop, 'mop'), F.at(x, 0.0, z))
    objs = G.settle(centre=False)
    G.squash(mop, 'mop_off', 0.001, seconds=0.2)
    return objs


# --- the stacked laundry centre (2x1): washer and dryer at the left, a folding counter with cubbies, a rail ---
WX, WW, WH, WD = -0.56, 0.68, 1.72, 0.58     # the stack's middle, width, height and depth
CX = 0.45                                     # the counter's middle


def _appliance(F, y0, hh, zf, door):
    """One of the stack: cabinet, control strip and dial; its porthole door unless `door` is False."""
    F.box(_name('cabinet'), WW, hh - 0.006, WD - 0.02, (WX, y0 + hh / 2, WALL + (WD - 0.02) / 2), M.appliance,
          bevel=0.02, bevel_segments=2)
    F.box(_name('strip'), WW - 0.03, 0.1, 0.012, (WX, y0 + hh - 0.075, zf - 0.016 + 0.012), M.trim, bevel=0.005)
    F.box(_name('display'), 0.16, 0.05, 0.004, (WX + 0.12, y0 + hh - 0.075, zf + 0.004), M.screen, bevel=0.003)
    F.box(_name('display_glow'), 0.07, 0.02, 0.002, (WX + 0.1, y0 + hh - 0.075, zf + 0.007), M.screen_glow)
    F.cyl(_name('dial'), 0.035, 0.025, (WX - 0.2, y0 + hh - 0.075, zf + 0.005), M.chrome, rot=(90, 0, 0), verts=16)
    cy = y0 + (hh - 0.13) / 2 + 0.03
    F.cyl(_name('drum'), 0.18, 0.01, (WX, cy, zf - 0.01), M.drum, rot=(90, 0, 0), verts=24)
    if not door:
        G.torus(_name('ring'), 0.21, 0.025, at=F.at(WX, cy, zf + 0.005), material=M.chrome, rot=(90, 0, 0),
                major_segments=28, minor_segments=8)
        F.cyl(_name('porthole'), 0.2, 0.01, (WX, cy, zf + 0.002), M.porthole, rot=(90, 0, 0), verts=28)
        F.box(_name('door_handle'), 0.04, 0.1, 0.03, (WX + 0.25, cy, zf + 0.02), M.chrome, bevel=0.012)
    return cy


def laundryCenter():
    F = G.GameFrame()
    zf = WALL + WD                                    # the stack's front
    hh = WH / 2
    with G.new_parts() as still:
        cy = _appliance(F, 0.0, hh, zf, True)         # the washer, its door the moving part
        F.box('kick', WW - 0.03, 0.05, 0.012, (WX, 0.03, zf + 0.002), M.trim, bevel=0.005)
        _appliance(F, hh, hh, zf, False)              # the dryer
        # a slim filler between the stack and the counter, with a towel bar
        F.box('filler', 0.2, 0.9, 0.56, (-0.12, 0.45, WALL + 0.28), M.body, bevel=0.006)
        G.rod('filler_bar', F.at(-0.19, 0.7, WALL + 0.57), F.at(-0.05, 0.7, WALL + 0.57), 0.008, material=M.chrome,
              verts=8)
        F.box('filler_towel', 0.13, 0.3, 0.02, (-0.12, 0.6, WALL + 0.58), M.towel_a, bevel=0.008)
        # the counter: open cubbies under a top, towels above and baskets below
        F.box('counter_top', 0.94, 0.04, 0.58, (CX, 0.92, WALL + 0.29), M.birch_plain, bevel=0.01)
        for y in (0.02, 0.42):
            F.box(_name('cubby_shelf'), 0.9, 0.04, 0.56, (CX, y, WALL + 0.28), M.body, bevel=0.006)
        for x in (CX - 0.44, CX, CX + 0.44):
            F.box(_name('cubby_side'), 0.03, 0.88, 0.56, (x, 0.45, WALL + 0.28), M.body, bevel=0.006)
        F.box('cubby_back', 0.9, 0.88, 0.02, (CX, 0.45, WALL + 0.01), M.tile)
        _towels(F, CX - 0.22, -0.14, 0.44, 4)
        _towels(F, CX + 0.22, -0.14, 0.44, 3, names=('towel_w', 'towel_c', 'towel_w'))
        for x, m in ((CX - 0.22, M.wicker), (CX + 0.22, M.wicker_b)):
            F.box(_name('basket'), 0.34, 0.3, 0.42, (x, 0.19, -0.16), m, bevel=0.04, bevel_segments=2)
            F.box(_name('basket_rim'), 0.3, 0.02, 0.38, (x, 0.34, -0.16), M.dark)
        # on the counter: a folded stack, detergents, a mini ironing board and iron
        _towels(F, CX - 0.18, -0.2, 0.94, 3, w=0.3, d=0.22, names=('towel_w', 'towel_a', 'towel_w'))
        _bottle(F, CX + 0.32, -0.3, 0.94, M.bottle_a, h=0.24)
        _bottle(F, CX + 0.2, -0.34, 0.94, M.bottle_b, h=0.2)
        F.box('ironing_board', 0.42, 0.02, 0.2, (CX + 0.12, 0.95, 0.03), M.towel_c, bevel=0.02)
        G.revolve('iron', [(0, 0), (0.06, 0), (0.05, 0.03), (0.0, 0.05)], at=F.at(CX + 0.12, 0.96, 0.03),
                  material=M.appliance, segments=3, rot=(0, 0, 90), scale=(1.6, 1, 1))
        # the rail over the counter, shirts on hangers
        G.rod('rail', F.at(CX - 0.43, 1.72, -0.29), F.at(CX + 0.43, 1.72, -0.29), 0.01, material=M.chrome, verts=10)
        for x in (CX - 0.42, CX + 0.42):
            G.rod(_name('rail_post'), F.at(x, 0.94, -0.31), F.at(x, 1.72, -0.31), 0.01, material=M.chrome, verts=8)
        for i, m in enumerate(('towel_c', 'towel_w', 'towel_b', 'towel_f')):
            x = CX - 0.27 + i * 0.18
            G.torus(_name('hook'), 0.015, 0.003, at=F.at(x, 1.72, -0.29), material=M.chrome, rot=(0, 90, 0),
                    major_segments=8, minor_segments=4)
            F.box(_name('hanger'), 0.04, 0.03, 0.26, (x, 1.68, -0.29), M.oak, bevel=0.008)
            F.box(_name('shirt'), 0.04, 0.42, 0.3, (x, 1.46, -0.29), _m(m), bevel=0.015)
    G.join(still, 'body', origin=(0, 0, 0))
    # The washer's porthole door, hinged on its left: `door_open` swings it open.
    with G.new_parts() as door:
        G.torus('door_ring', 0.21, 0.025, at=F.at(WX, cy, zf + 0.005), material=M.chrome, rot=(90, 0, 0),
                major_segments=28, minor_segments=8)
        F.cyl('door_window', 0.2, 0.01, (WX, cy, zf + 0.002), M.porthole, rot=(90, 0, 0), verts=28)
        F.box('door_handle', 0.04, 0.1, 0.03, (WX + 0.25, cy, zf + 0.02), M.chrome, bevel=0.012)
    door = G.hinged(G.join(door, 'door'), F.at(WX - 0.235, cy, zf + 0.005))
    objs = G.settle(centre=False)
    G.swing(door, 'door_open', 100, seconds=0.8)
    return objs


# --- the rainfall steam shower (1x1): a glass cabin, tiled back and side, a ceiling rain head, a teak bench ---
S, T, HC = 0.45, 0.07, 2.18      # half the cabin's width, the tray's top, the cabin's height
HEAD_Y = 2.075                   # the rain head's face


def rainShowerTower():
    F = G.GameFrame()
    w = 2 * S
    with G.new_parts() as still:
        F.box('tray', w, T - 0.012, w, (0, (T - 0.012) / 2, 0), M.body, bevel=0.02)
        F.box('tray_floor', w - 0.06, 0.012, w - 0.06, (0, T - 0.018, 0), M.body, bevel=0.01)
        F.cyl('drain', 0.04, 0.006, (0.1, T - 0.012, 0.1), M.chrome, verts=12)
        F.box('back_wall', w, HC - T, 0.03, (0, T + (HC - T) / 2, -S + 0.015), M.tile)
        F.box('side_wall', 0.03, HC - T, w - 0.03, (-S + 0.015, T + (HC - T) / 2, 0.015), M.tile)
        for k in range(1, 7):                                       # tile joints
            y = T + k * (HC - T) / 7
            F.box(_name('joint'), w - 0.04, 0.008, 0.004, (0, y, -S + 0.031), M.joint)
            F.box(_name('joint'), 0.004, 0.008, w - 0.07, (-S + 0.031, y, 0.015), M.joint)
        for k in (1, 2):
            F.box(_name('joint'), 0.008, HC - T, 0.004, (-S + k * w / 3, T + (HC - T) / 2, -S + 0.031), M.joint)
            F.box(_name('joint'), 0.004, HC - T, 0.008, (-S + 0.031, T + (HC - T) / 2, -S + k * w / 3 + 0.03), M.joint)
        # the glass: the right side and the front, a fixed pane and the door
        F.box('glass_side', 0.012, HC - T - 0.06, w - 0.06, (S - 0.012, T + (HC - T - 0.06) / 2, 0.0), M.glass)
        F.box('glass_front', w * 0.55, HC - T - 0.08, 0.012, (-S + w * 0.3, T + (HC - T - 0.08) / 2 + 0.01, S - 0.012),
              M.glass)
        F.box('glass_door', w * 0.5, HC - T - 0.1, 0.012, (S - w * 0.27, T + (HC - T - 0.1) / 2 + 0.02, S - 0.03), M.glass)
        for px, pz in ((S - 0.015, S - 0.015), (S - 0.015, -S + 0.035)):
            F.box(_name('frame_post'), 0.03, HC - T, 0.03, (px, T + (HC - T) / 2, pz), M.chrome)
        F.box('frame_top', w - 0.03, 0.03, 0.03, (0.015, HC - 0.015, S - 0.015), M.chrome)
        F.box('frame_top_side', 0.03, 0.03, w - 0.03, (S - 0.015, HC - 0.015, 0.0), M.chrome)
        F.box('frame_sill', w - 0.03, 0.02, 0.03, (0.015, T + 0.01, S - 0.015), M.chrome)
        F.box('door_pull', 0.022, 0.4, 0.022, (S - w * 0.5 + 0.03, 1.05, S - 0.04), M.chrome, bevel=0.009)
        # the rain head on its arm from the back wall
        F.box('rain_head', 0.44, 0.02, 0.44, (0, HEAD_Y + 0.015, 0.0), M.chrome, bevel=0.008)
        F.box('rain_face', 0.4, 0.004, 0.4, (0, HEAD_Y + 0.003, 0.0), M.steel)
        F.cyl('rain_stem', 0.015, 0.06, (0, HEAD_Y + 0.055, 0.0), M.chrome, verts=10)
        F.box('rain_arm', 0.03, 0.03, 0.43, (0, HEAD_Y + 0.085, -0.215), M.chrome)
        # the steam column with its jets, the control panel
        F.box('steam_column', 0.16, 1.2, 0.04, (-0.24, 1.15, -S + 0.05), M.appliance, bevel=0.01)
        for k in range(5):
            F.cyl(_name('jet'), 0.02, 0.01, (-0.24, 0.75 + k * 0.18, -S + 0.072), M.chrome, rot=(90, 0, 0), verts=10)
        F.box('controls', 0.2, 0.12, 0.012, (0.22, 1.25, -S + 0.036), M.screen, bevel=0.008)
        F.box('controls_line', 0.12, 0.04, 0.004, (0.22, 1.27, -S + 0.043), M.teal)
        # the teak bench along the back, towels and bottles on it
        F.box('bench', 0.72, 0.04, 0.28, (0.0, 0.45, -0.24), M.teak, bevel=0.01)
        for x in (-0.3, 0.3):
            F.box(_name('bench_leg'), 0.04, 0.38, 0.24, (x, 0.26, -0.24), M.teak, bevel=0.008)
        _towels(F, 0.2, -0.26, 0.47, 2, w=0.24, d=0.2, names=('pad', 'pastel_a'))   # satin: no fabric draw
        _bottle(F, -0.22, -0.3, 0.47, M.bottle_c, h=0.16, d=0.06)
        _bottle(F, -0.12, -0.3, 0.47, M.bottle_b, h=0.18, d=0.06)
    G.join(still, 'body', origin=(0, 0, 0))
    # The rain: streaks from the head's face to the tray, origin at the head: `water_on` grows them down.
    with G.new_parts() as rain:
        rnd = random.Random(3)
        for i in range(5):
            for j in range(5):
                x, z = -0.16 + i * 0.08 + rnd.uniform(-0.01, 0.01), -0.16 + j * 0.08 + rnd.uniform(-0.01, 0.01)
                G.rod(_name('stream'), F.at(x, HEAD_Y, z), F.at(x, T + rnd.uniform(0.0, 0.03), z), 0.004,
                      material=M.water, verts=4)
    rain = G.join(rain, 'water', origin=F.at(0, HEAD_Y, 0))
    rain.scale[2] = 0.001
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.grow(rain, 'water_on', seconds=0.6)
    return objs


# --- the label-and-sort organizer desk (2x1): seated in the middle, a hutch of labelled bins before them ------
HZ, HD = 0.37, 0.16       # the hutch's depth and its own depth
BIN = (0.0, 0.87)         # the bin `drawer_open` pulls out (x, middle height): the lower row's middle


def _bin(F, x, y, material):
    F.box(_name('bin'), 0.27, 0.19, HD - 0.02, (x, y, HZ + 0.005), material, bevel=0.02, bevel_segments=2)
    F.box(_name('bin_label'), 0.1, 0.04, 0.004, (x, y + 0.035, HZ - HD / 2 + 0.003), M.paper, bevel=0.002)
    F.box(_name('bin_grip'), 0.08, 0.02, 0.004, (x, y - 0.04, HZ - HD / 2 + 0.003), M.dark)


def organizerDesk():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('desk_top', 1.6, 0.035, 0.34, (0, 0.745, 0.28), M.birch, bevel=0.008)
        for x in (-0.78, 0.78):
            F.box(_name('desk_side'), 0.04, 0.73, 0.32, (x, 0.365, 0.28), M.body, bevel=0.008)
        F.box('modesty', 1.52, 0.6, 0.015, (0, 0.42, 0.44), M.birch, bevel=0.004)
        # the hutch: two rows of five cubbies, open towards the sitter
        F.box('hutch_shelf', 1.6, 0.02, HD, (0, 0.99, HZ), M.body, bevel=0.005)
        F.box('hutch_top', 1.6, 0.02, HD, (0, 1.22, HZ), M.body, bevel=0.005)
        F.box('hutch_back', 1.6, 0.5, 0.012, (0, 1.0, HZ + HD / 2 - 0.004), M.birch)
        for i in range(6):
            F.box(_name('divider'), 0.02, 0.48, HD, (-0.79 + i * 0.316, 1.0, HZ), M.body)
        for r, y in enumerate((0.87, 1.105)):
            for i in range(5):
                x = -0.632 + i * 0.316
                if (x, y) != BIN:
                    _bin(F, x, y, _m(PASTELS[(i + r * 2) % len(PASTELS)]))
        # on the desk: the ledger, the label maker, a calculator, a pen cup
        F.box('ledger', 0.36, 0.02, 0.24, (0.32, 0.773, 0.2), M.ledger, bevel=0.006)
        F.box('ledger_pages', 0.33, 0.012, 0.21, (0.32, 0.789, 0.2), M.paper)
        F.box('ledger_spine', 0.008, 0.003, 0.2, (0.32, 0.796, 0.2), M.ledger)
        F.box('label_maker', 0.1, 0.05, 0.16, (-0.4, 0.79, 0.2), M.coral, bevel=0.02)
        F.box('label_screen', 0.06, 0.004, 0.06, (-0.4, 0.816, 0.17), M.screen)
        F.box('calculator', 0.12, 0.015, 0.16, (-0.62, 0.77, 0.2), M.trim, bevel=0.006)
        F.cyl('pen_cup', 0.04, 0.11, (0.62, 0.818, 0.2), M.pastel_f, verts=14)
        for k, m in enumerate(('pastel_d', 'ledger', 'coral')):
            F.cyl(_name('pen'), 0.006, 0.16, (0.605 + 0.016 * k, 0.86, 0.19 + 0.01 * k), _m(m), verts=6)
        # the chair (seat top 0.45 at depth -0.12)
        F.box('chair_seat', 0.46, 0.06, 0.44, (0, 0.42, -0.12), M.pastel_c, bevel=0.025, bevel_segments=2)
        F.box('chair_back', 0.44, 0.34, 0.06, (0, 0.72, -0.33), M.pastel_c, rot=(6, 0, 0), bevel=0.025,
              bevel_segments=2)
        for dx in (-0.2, 0.2):
            for dz in (-0.3, 0.06):
                F.cone(_name('chair_leg'), 0.015, 0.012, 0.39, (dx, 0.195, dz), M.birch, verts=8)
            F.box(_name('chair_post'), 0.03, 0.3, 0.03, (dx, 0.55, -0.32), M.birch, bevel=0.008)
    G.join(still, 'body', origin=(0, 0, 0))
    # The bin in the lower row's middle: `drawer_open` slides it out towards the sitter.
    with G.new_parts() as drawer:
        _bin(F, BIN[0], BIN[1], M.pastel_b)
        for k in range(3):                                         # labelled cards inside
            F.box(_name('card'), 0.2, 0.1, 0.004, (BIN[0], BIN[1] + 0.06, HZ - 0.03 + 0.03 * k), M.paper)
    drawer = G.join(drawer, 'bin')
    objs = G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, 0.13, 0), seconds=0.6)
    return objs


# --- the robot vacuum and dock (1x1): a self-emptying dock against the wall, the robot docked on a round rug ---
ROBOT = (0.0, -0.02)    # the docked robot's middle (x, depth)


def robotVacuumDock():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.cyl('rug', 0.43, 0.012, (0, 0.006, 0.0), M.rug, verts=36)
        F.cyl('rug_inner', 0.35, 0.014, (0, 0.007, 0.0), M.rug_inner, verts=36)
        F.box('dock', 0.38, 0.46, 0.24, (0, 0.23, WALL + 0.12), M.appliance, bevel=0.05, bevel_segments=3)
        F.box('bin_window', 0.3, 0.2, 0.012, (0, 0.32, WALL + 0.245), M.porthole, bevel=0.02)
        F.box('bin', 0.2, 0.12, 0.01, (0, 0.32, WALL + 0.247), M.steel, bevel=0.015)
        F.box('ramp', 0.36, 0.02, 0.14, (0, 0.024, WALL + 0.29), M.trim, bevel=0.008)
        F.sphere('led', 0.012, (0.13, 0.42, WALL + 0.24), M.led, segments=8, rings=6)
        # a little plant beside the dock
        G.revolve('plant_pot', [(0, 0), (0.07, 0), (0.09, 0.2), (0, 0.2)], at=F.at(0.31, 0, WALL + 0.12),
                  material=M.body, segments=16)
        for k in range(4):
            G.boulder(_name('leaves'), 0.09 - 0.012 * k, at=F.at(0.31 + 0.02 * (k % 2), 0.2 + 0.07 * k, WALL + 0.12),
                      material=M.green, seed=k * 2.1, noise=0.5, subdivisions=2)
    G.join(still, 'body', origin=(0, 0, 0))
    # The robot: a disc with a teal bumper ring, the lidar turret and a side brush: `vacuum_out` rolls it 60 cm
    # out of its dock, turning round as it goes; played back, it rolls home.
    x, z = ROBOT
    y = 0.014
    with G.new_parts() as robot:
        G.revolve('robot_body', [(0, 0), (0.16, 0), (0.17, 0.025), (0.17, 0.07), (0.16, 0.085), (0, 0.085)],
                  at=F.at(x, y, z), material=M.appliance, segments=28)
        G.torus('robot_bumper', 0.17, 0.01, at=F.at(x, y + 0.045, z), material=M.teal, major_segments=28,
                minor_segments=6)
        F.cyl('lidar', 0.045, 0.035, (x, y + 0.1, z - 0.06), M.dark, verts=16)
        F.box('robot_front', 0.22, 0.04, 0.03, (x, y + 0.04, z + 0.155), M.dark, bevel=0.012)
        F.cyl('side_brush', 0.04, 0.004, (x + 0.12, y + 0.004, z + 0.12), M.steel, verts=8)
    robot = G.hinged(G.join(robot, 'robot'), F.at(x, y, z))
    objs = G.settle(centre=False)
    G.slide(robot, 'vacuum_out', (0, -0.6, 0), seconds=1.4)
    G.swing(robot, 'vacuum_out', 180, seconds=1.4)
    return objs


# --- the tidy entry console (1x1): a slim birch console, a shoe shelf, a key tray drawer, a leaning mirror -----
DRAWER = (-0.2, 0.74)    # the key tray drawer's middle (x, height)
FZ = WALL + 0.3          # the console's front


def entryConsole():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('top', 0.86, 0.035, 0.3, (0, 0.8, WALL + 0.15), M.birch, bevel=0.008)
        F.box('shoe_shelf', 0.82, 0.025, 0.26, (0, 0.22, WALL + 0.15), M.birch, bevel=0.006)
        for x in (-0.41, 0.41):
            for z in (WALL + 0.02, FZ - 0.02):
                F.box(_name('leg'), 0.03, 0.79, 0.03, (x, 0.395, z), M.body, bevel=0.008)
        # the apron round the drawer's opening, and its housing
        dx, dy = DRAWER
        F.box('apron_r', 0.82 - (dx + 0.17 + 0.41), 0.08, 0.02, ((dx + 0.17 + 0.41) / 2, 0.74, FZ - 0.02), M.body)
        F.box('apron_l', 0.07, 0.08, 0.02, (-0.375, 0.74, FZ - 0.02), M.body)
        F.box('housing', 0.36, 0.01, 0.26, (dx, 0.695, WALL + 0.15), M.body)
        F.box('apron_back', 0.82, 0.08, 0.02, (0, 0.74, WALL + 0.02), M.body)
        # shoes, neatly paired
        for i, m in enumerate(('shoe_a', 'shoe_b', 'shoe_c')):
            x = -0.27 + i * 0.27
            for ddx in (-0.045, 0.045):
                F.box(_name('shoe'), 0.075, 0.07, 0.22, (x + ddx, 0.267, -0.27), _m(m), bevel=0.03, bevel_segments=2)
                F.box(_name('sole'), 0.075, 0.015, 0.22, (x + ddx, 0.24, -0.27), M.body, bevel=0.006)
        # on top: a bowl, a vase of tulips
        G.revolve('bowl', [(0, 0), (0.05, 0), (0.08, 0.06), (0.072, 0.06), (0, 0.012)], at=F.at(0.02, 0.8175, -0.27),
                  material=M.appliance, segments=16)
        G.revolve('vase', [(0, 0), (0.05, 0), (0.06, 0.09), (0.03, 0.19), (0.034, 0.22), (0, 0.22)],
                  at=F.at(0.28, 0.8175, -0.3), material=M.bottle_b, segments=14)
        for k in range(5):
            a = k * 72
            x, z = 0.28 + 0.035 * G.cosd(a), -0.3 + 0.035 * G.sind(a)
            G.rod(_name('tulip_stem'), F.at(0.28, 0.98, -0.3), F.at(x, 1.2, z), 0.005, material=M.stem, verts=5)
            F.sphere(_name('tulip'), 0.5, (x, 1.23, z), _m(('coral', 'pastel_b', 'pad', 'coral', 'pastel_e')[k]),
                     scale=(0.045, 0.045, 0.06), segments=8, rings=6)
        # the mirror leaning on the wall above, coat hooks with a tote and a scarf
        F.box('mirror_frame', 0.5, 0.9, 0.03, (0.0, 1.32, -0.41), M.body, rot=(3, 0, 0), bevel=0.02)
        F.box('mirror', 0.44, 0.84, 0.006, (0.0, 1.32, -0.393), M.mirror, rot=(3, 0, 0), bevel=0.003)
        F.box('hook_board', 0.03, 0.4, 0.03, (-0.36, 1.42, WALL + 0.015), M.oak, bevel=0.008)
        for y in (1.5, 1.32):
            F.cyl(_name('hook'), 0.01, 0.06, (-0.36, y, WALL + 0.06), M.brass, rot=(90, 0, 0), verts=8)
        F.box('tote', 0.2, 0.24, 0.06, (-0.34, 1.2, WALL + 0.07), M.wicker, bevel=0.02)
        F.box('scarf', 0.06, 0.4, 0.02, (-0.33, 1.25, WALL + 0.08), M.coral, bevel=0.008)
    G.join(still, 'body', origin=(0, 0, 0))
    # The key tray drawer: front, knob, the tray and keys inside: `tray` slides it out.
    with G.new_parts() as drawer:
        F.box('drawer_front', 0.32, 0.07, 0.018, (dx, dy, FZ - 0.009), M.body, bevel=0.005)
        F.sphere('drawer_knob', 0.012, (dx, dy, FZ + 0.006), M.brass, segments=10, rings=6)
        F.box('drawer_box', 0.3, 0.05, 0.24, (dx, dy - 0.005, FZ - 0.14), M.trim)
        F.box('key_tray', 0.24, 0.012, 0.16, (dx, dy + 0.026, FZ - 0.12), M.oak, bevel=0.004)
        G.torus('key_ring', 0.025, 0.004, at=F.at(dx - 0.03, dy + 0.034, FZ - 0.11), material=M.brass,
                major_segments=10, minor_segments=4)
        F.box('key', 0.05, 0.004, 0.012, (dx + 0.01, dy + 0.034, FZ - 0.1), M.brass)
    drawer = G.join(drawer, 'drawer')
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.slide(drawer, 'tray', (0, -0.2, 0), seconds=0.6)
    return objs


FAMILY = G.Family(
    'pack_neat', M,
    [G.Model('steamMopDock', steamMopDock, size=(0.788, 1.25, 0.708), key='model.neat.steamMopDock',
             centre=(-0.006, -0.076),
             motions={'uses': {'mop': 'mop_off'}},
             paint=[('body', 'Body'), ('teal', 'Mop')]),
     G.Model('laundryCenter', laundryCenter, size=(1.82, 1.738, 0.625), key='model.neat.laundryCenter',
             centre=(0.01, -0.127),
             motions={'uses': {'laundry': 'door_open', 'fold': 'door_open'}},
             paint=[('body', 'Body'), ('appliance', 'Washer & dryer')]),
     G.Model('rainShowerTower', rainShowerTower, size=(0.9, 2.18, 0.9), key='model.neat.rainShowerTower',
             centre=(0, 0.02), motions={'uses': {'steamShower': 'water_on'}},
             paint=[('tile', 'Body'), ('teak', 'Bench')]),
     G.Model('organizerDesk', organizerDesk, size=(1.6, 1.25, 0.827), key='model.neat.organizerDesk',
             centre=(0, 0.039),
             motions={'uses': {'sort': 'drawer_open', 'ledger': 'drawer_open'}},
             paint=[('body', 'Body'), ('birch', 'Wood'), ('pastel_c', 'Chair')]),
     G.Model('robotVacuumDock', robotVacuumDock, size=(0.86, 0.519, 0.86), key='model.neat.robotVacuumDock',
             motions={'uses': {'program': 'vacuum_out'}, 'hold': 2},
             paint=[('appliance', 'Body'), ('teal', 'Trim'), ('rug', 'Rug')]),
     G.Model('entryConsole', entryConsole, size=(0.87, 1.769, 0.336), key='model.neat.entryConsole',
             centre=(-0.005, -0.262),
             motions={'uses': {'tidy': 'tray'}},
             paint=[('body', 'Body'), ('birch', 'Wood')])],
    comment='Blender-built neat pack models (tools/art/blender/families/pack_neat.py). Listed in the base '
            'manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw call '
            'per finish group. Each moves one part while it is used: the mop leaves its dock for the '
            'resident\'s hands, the washer\'s door opens, the rain falls, a bin slides out, the robot rolls '
            'out and the key tray drawer opens.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

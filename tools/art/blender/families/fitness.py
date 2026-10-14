# The home gym: the treadmill (`model.treadmill`), the weight bench (`model.weightBench`), the exercise bike
# (`model.exerciseBike`), the yoga mat (`model.yogaMat`) and the dumbbell rack (`model.dumbbells`), over their
# primitive-part placeholders (`objects_extra.py`, `furniture.py`, `catalog.py`), which stay the fallback.
# Notes: fitness.md.
#
# Each is built where its placeholder stands (`F`, game axes). The treadmill's belt slats run back under the
# runner while it is used (`belt_run`, a loop: one slat on, home unseen under the hood and the rear cap). The
# bike's cranks and pedals turn while it is cycled (`pedals_turn`, a loop). The yoga mat lies rolled up and
# unrolls flat for yoga (`unroll`), rolling up again at the end. The lift animation draws no weight in the
# hands, so the bench's bar and the rack's dumbbells stay where they are: those two are still.
#
# Run:  python3 -B tools/art/blender/build.py fitness [--only treadmill] [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    body=('#D8D9D7', 'satin'),       # a machine's frame and housings; the main part the player colours
    frame=('#5C6168', 'metal'),      # the bench's and the rack's steel
    pads=('#3E6E73', 'satin'),       # vinyl pads, foam grips
    steel=('#9EA3A8', 'metal'),      # the treadmill's uprights and handlebars
    dark=('#3A3D41', 'satin'),       # feet, caps, the flywheel
    belt=('#26282B', 'matte'),       # the treadmill's belt
    slat=('#4A4E54', 'matte'),       # its slats
    screen=('#0E2230', 'gloss'),     # a console's screen
    accent=('#3C8DBF', 'satin'),     # a console's light strip, the hood's band
    red=('#B33A32', 'satin'),        # buttons, J-hooks, the plates' inner rings
    chrome=('#D0D3D6', 'chrome'),    # bars, handles
    plate=('#2F3236', 'satin'),      # weight plates, dumbbell heads
    bottle=('#4A90C2', 'gloss'),     # water bottles
    mat=('#7FA38A', 'matte'),        # the yoga mat
    block=('#E8E2D6', 'matte'),      # the yoga block
)


def _on(c, tilt, d):
    """A game point `d` (x, y, z) from `c` on a face tilted `tilt` degrees about the game's x axis."""
    a = math.radians(tilt)
    return (c[0] + d[0], c[1] + d[1] * math.cos(a) - d[2] * math.sin(a), c[2] + d[1] * math.sin(a) + d[2] * math.cos(a))


def _dumbbell(F, name, x, y, z, w, along='x'):
    """A hex dumbbell: a chrome handle and two hexagonal heads `w` across, lying along game x (or z)."""
    rot = (0, 90, 0) if along == 'x' else (90, 0, 0)
    off = (lambda k: (x + k, y, z)) if along == 'x' else (lambda k: (x, y, z + k))
    F.cyl(f'{name}_grip', 0.014, 0.16, (x, y, z), M.chrome, rot=rot, verts=10)
    for s in (-1, 1):
        F.cyl(f'{name}_head{s}', w / 2, 0.045, off(s * 0.068), M.plate, rot=rot, verts=6, bevel=0.004)


# --- the treadmill: belt along the depth, the console at the back; the runner faces the back -------------
SLAT = 0.1       # the slats' pitch: the belt's loop moves them one on
HOOD = -0.63     # the motor hood's front edge (the belt comes out from under it)
CAP = 0.80       # the rear roller cap's front edge (the belt goes under it)
CON, TILT = (0, 1.3, -0.58), -35   # the console's middle, tilted back to face the runner


def treadmill():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('deck', 0.66, 0.1, 1.56, (0, 0.09, 0.12), M.body, bevel=0.02)
        F.box('belt', 0.56, 0.012, 1.5, (0, 0.146, 0.1), M.belt, bevel=0.003)
        for s in (-1, 1):
            F.box(f'rail{s}', 0.07, 0.05, 1.5, (s * 0.33, 0.165, 0.1), M.body, bevel=0.015)
            F.box(f'tread{s}', 0.05, 0.006, 1.4, (s * 0.33, 0.192, 0.1), M.pads, bevel=0.002)
        F.box('cap', 0.66, 0.055, 0.12, (0, 0.15, 0.86), M.body, bevel=0.02)
        F.box('hood', 0.7, 0.2, 0.3, (0, 0.15, -0.78), M.body, bevel=0.05)
        F.box('band', 0.705, 0.025, 0.24, (0, 0.205, -0.78), M.accent, bevel=0.008)
        for x in (-0.28, 0.28):
            for z in (-0.86, 0.84):
                F.box(f'foot{x}{z}', 0.06, 0.04, 0.08, (x, 0.02, z), M.dark, bevel=0.01)
        for s in (-1, 1):
            G.rod(f'upright{s}', F.at(s * 0.3, 0.2, -0.8), F.at(s * 0.3, 1.22, -0.62), 0.032, material=M.steel, verts=12)
            G.rod(f'handlebar{s}', F.at(s * 0.33, 1.16, -0.6), F.at(s * 0.33, 1.06, -0.28), 0.018, material=M.steel, verts=10)
            G.rod(f'grip{s}', F.at(s * 0.33, 1.09, -0.42), F.at(s * 0.33, 1.06, -0.3), 0.024, material=M.pads, verts=10)
        F.box('console', 0.72, 0.3, 0.1, CON, M.body, rot=(TILT, 0, 0), bevel=0.03)
        F.box('screen', 0.34, 0.16, 0.006, _on(CON, TILT, (0, 0.03, 0.051)), M.screen, rot=(TILT, 0, 0), bevel=0.003)
        F.box('strip', 0.6, 0.02, 0.006, _on(CON, TILT, (0, -0.075, 0.051)), M.accent, rot=(TILT, 0, 0))
        for k, (x, m) in enumerate(((-0.25, M.red), (-0.19, M.block), (0.19, M.block), (0.25, M.accent))):
            F.box(f'button{k}', 0.04, 0.03, 0.01, _on(CON, TILT, (x, -0.11, 0.052)), m, rot=(TILT, 0, 0), bevel=0.004)
        F.cyl('holder', 0.04, 0.03, (0.39, 0.9, -0.68), M.steel, verts=14)
        F.cyl('bottle', 0.035, 0.18, (0.39, 1.0, -0.68), M.bottle, verts=14, bevel=0.01)
        F.cyl('bottle_cap', 0.022, 0.03, (0.39, 1.1, -0.68), M.dark, verts=12)
    G.join(still, 'body', origin=(0, 0, 0))
    # The slats: from just under the hood to just short of the cap, so one slat on they are still hidden.
    with G.new_parts() as parts:
        z = HOOD - 0.05
        k = 0
        while z + SLAT <= CAP + 0.04:
            F.box(f'slat{k}', 0.54, 0.004, 0.03, (0, 0.154, z), M.slat)
            z += SLAT
            k += 1
    slats = G.join(parts, 'slats')
    G.settle(centre=False)
    G.scroll(slats, 'belt_run', (0, -SLAT, 0), seconds=0.5)   # back towards the rear (game +z): 0.2 m/s
    return G.meshes()


# --- the weight bench: a padded bench, a rack with the bar at the head end, dumbbells by the feet ----------
BAR = (0, 1.02, -0.72)   # the bar's middle on the J-hooks


def weightBench():
    F = G.GameFrame(shift=0.02)   # 2 cm forward of its placeholder: the rack's back on the wall's face
    with G.new_parts() as parts:
        F.box('pad', 0.3, 0.09, 1.22, (0, 0.455, -0.12), M.pads, bevel=0.03)
        F.box('board', 0.27, 0.02, 1.18, (0, 0.4, -0.12), M.dark, bevel=0.005)
        F.box('beam', 0.12, 0.06, 1.2, (0, 0.36, -0.12), M.frame, bevel=0.012)
        for z in (0.4, -0.62):
            F.box(f'post{z}', 0.08, 0.34, 0.08, (0, 0.18, z), M.frame, bevel=0.012)
            F.box(f'feet{z}', 0.5, 0.05, 0.08, (0, 0.025, z), M.frame, bevel=0.012)
            for s in (-1, 1):
                F.box(f'cap{z}{s}', 0.04, 0.052, 0.085, (s * 0.25, 0.026, z), M.dark, bevel=0.01)
        F.box('arm', 0.06, 0.32, 0.06, (0, 0.4, 0.62), M.frame, bevel=0.01)
        G.rod('roller_bar', F.at(-0.28, 0.32, 0.64), F.at(0.28, 0.32, 0.64), 0.02, material=M.chrome, verts=10)
        for s in (-1, 1):
            F.cyl(f'roller{s}', 0.06, 0.16, (s * 0.2, 0.32, 0.64), M.pads, rot=(0, 90, 0), verts=16, bevel=0.015)
        for s in (-1, 1):   # the rack's uprights, feet and J-hooks
            F.box(f'upright{s}', 0.06, 1.2, 0.06, (s * 0.36, 0.6, -0.8), M.frame, bevel=0.01)
            F.box(f'base{s}', 0.06, 0.05, 0.4, (s * 0.36, 0.025, -0.75), M.frame, bevel=0.01)
            G.rod(f'brace{s}', F.at(s * 0.36, 0.05, -0.58), F.at(s * 0.36, 0.45, -0.77), 0.015, material=M.frame, verts=8)
            F.box(f'hook{s}', 0.06, 0.05, 0.1, (s * 0.36, 0.98, -0.75), M.red, bevel=0.01)
            F.box(f'hook_lip{s}', 0.06, 0.06, 0.02, (s * 0.36, 1.02, -0.705), M.red, bevel=0.006)
        F.box('rack_brace', 0.66, 0.05, 0.05, (0, 0.26, -0.8), M.frame, bevel=0.01)
        for x in (-0.3, 0.3):
            _dumbbell(F, f'dumbbell{x}', x, 0.05, 0.85, 0.1)
    G.join(parts, 'body', origin=(0, 0, 0))
    # The bar on the J-hooks, its origin at its middle: hidden while lifted (`bar_off`), the game drawing it
    # in the lifter's hands (content `held`).
    with G.new_parts() as parts:
        G.rod('bar', F.at(-0.45, BAR[1], BAR[2]), F.at(0.45, BAR[1], BAR[2]), 0.014, material=M.chrome, verts=12)
        for s in (-1, 1):
            F.cyl(f'plate{s}', 0.16, 0.03, (s * 0.41, BAR[1], BAR[2]), M.plate, rot=(0, 90, 0), verts=28, bevel=0.006)
            F.cyl(f'ring{s}', 0.11, 0.032, (s * 0.41, BAR[1], BAR[2]), M.red, rot=(0, 90, 0), verts=24)
            F.cyl(f'collar{s}', 0.025, 0.025, (s * 0.38, BAR[1], BAR[2]), M.chrome, rot=(0, 90, 0), verts=12)
    bar = G.hinged(G.join(parts, 'bar'), F.at(*BAR))
    G.settle(centre=False)
    G.squash(bar, 'bar_off', 0.001, seconds=0.2)
    return G.meshes()


# --- the exercise bike: an upright bike, the flywheel at the front, the cranks under the saddle -------------
CRANK = (0, 0.3, 0.04)   # the crank axle
ARM = 0.16               # the cranks' length


def exerciseBike():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('beam', 0.12, 0.06, 0.86, (0, 0.05, 0.0), M.body, bevel=0.015)
        for z in (-0.38, 0.38):
            F.box(f'stabiliser{z}', 0.5, 0.05, 0.08, (0, 0.03, z), M.body, bevel=0.02)
            for s in (-1, 1):
                F.box(f'cap{z}{s}', 0.05, 0.035, 0.085, (s * 0.24, 0.018, z), M.dark, bevel=0.01)
        F.cyl('flywheel', 0.2, 0.05, (0, 0.3, 0.27), M.dark, rot=(0, 90, 0), verts=32, bevel=0.01)
        for s in (-1, 1):
            F.cyl(f'flywheel_cover{s}', 0.13, 0.012, (s * 0.032, 0.3, 0.27), M.body, rot=(0, 90, 0), verts=28)
        G.rod('seat_post', F.at(0, 0.06, -0.14), F.at(0, 0.7, -0.1), 0.03, material=M.body, verts=12)
        G.rod('handle_post', F.at(0, 0.06, 0.22), F.at(0, 0.95, 0.32), 0.03, material=M.body, verts=12)
        G.rod('down_tube', F.at(0, 0.62, -0.11), F.at(0, CRANK[1], CRANK[2]), 0.024, material=M.body, verts=10)
        F.cyl('bracket', 0.045, 0.1, CRANK, M.body, rot=(0, 90, 0), verts=16)
        F.box('saddle', 0.2, 0.06, 0.14, (0, 0.725, -0.15), M.pads, bevel=0.025)
        F.box('saddle_nose', 0.08, 0.05, 0.16, (0, 0.73, -0.02), M.pads, bevel=0.02)
        G.rod('bar', F.at(-0.22, 0.98, 0.32), F.at(0.22, 0.98, 0.32), 0.017, material=M.steel, verts=10)
        for s in (-1, 1):
            G.rod(f'horn{s}', F.at(s * 0.22, 0.98, 0.32), F.at(s * 0.22, 1.0, 0.16), 0.017, material=M.steel, verts=10)
            G.rod(f'grip{s}', F.at(s * 0.22, 0.995, 0.24), F.at(s * 0.22, 1.003, 0.15), 0.022, material=M.pads, verts=10)
        F.box('console', 0.2, 0.12, 0.04, (0, 1.05, 0.36), M.dark, rot=(-30, 0, 0), bevel=0.01)
        F.box('screen', 0.16, 0.08, 0.004, _on((0, 1.05, 0.36), -30, (0, 0, 0.021)), M.accent, rot=(-30, 0, 0))
    G.join(still, 'body', origin=(0, 0, 0))
    # The cranks and pedals, joined round the axle so they turn about it.
    with G.new_parts() as parts:
        for s, a in ((1, 0), (-1, 180)):
            end = (s * 0.075, CRANK[1] - ARM * G.cosd(a), CRANK[2] + ARM * G.sind(a))
            G.rod(f'crank{s}', F.at(s * 0.075, *CRANK[1:]), F.at(*end), 0.012, material=M.steel, verts=8)
            F.cyl(f'pedal{s}', 0.022, 0.1, (s * 0.13, end[1], end[2]), M.dark, rot=(0, 90, 0), verts=10)
        F.cyl('axle', 0.018, 0.16, CRANK, M.steel, rot=(0, 90, 0), verts=10)
    pedals = G.hinged(G.join(parts, 'pedals'), F.at(*CRANK))
    G.settle()
    G.spin(pedals, 'pedals_turn', turns=1, seconds=1.2, axis='X')
    return G.meshes()


# --- the yoga mat: rolled up at the back with a block and a bottle; it unrolls to the front -----------------
MAT_W, MAT_L, MAT_T = 0.62, 0.9, 0.016   # the mat, flat (the house's floor is drawn 1 cm up)
ROLL_R = 0.065                           # the roll's radius
BACK = -0.45                             # the mat's back edge (game depth)


def yogaMat():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('block', 0.15, 0.12, 0.24, (0.41, 0.06, -0.3), M.block, bevel=0.025)
        F.cyl('bottle', 0.035, 0.2, (-0.4, 0.1, -0.32), M.bottle, verts=14, bevel=0.01)
        F.cyl('bottle_cap', 0.022, 0.035, (-0.4, 0.215, -0.32), M.dark, verts=12)
    G.join(still, 'body', origin=(0, 0, 0))
    # The roll lies at the back, its origin on the floor under its axis: it shrinks onto the floor as it goes.
    roll_z = BACK + ROLL_R
    with G.new_parts() as parts:
        F.cyl('roll', ROLL_R, MAT_W, (0, ROLL_R, roll_z), M.mat, rot=(0, 90, 0), verts=24, bevel=0.004)
        F.cyl('roll_core', ROLL_R * 0.45, MAT_W + 0.002, (0, ROLL_R, roll_z), M.dark, rot=(0, 90, 0), verts=16)
    roll = G.hinged(G.join(parts, 'roll'), F.at(0, 0, roll_z))
    # The flat mat, its origin on its back edge, built full and squeezed to nothing for rest.
    with G.new_parts() as parts:
        F.box('mat', MAT_W, MAT_T, MAT_L, (0, MAT_T / 2, BACK + MAT_L / 2), M.mat, bevel=0.002)
    mat = G.hinged(G.join(parts, 'mat'), F.at(0, 0, BACK))
    mat.scale[1] = 0.001
    G.settle(centre=False)
    left = 0.12   # what is left of the roll at the front end: a curled edge
    run = MAT_L - ROLL_R - left * ROLL_R
    G.grow(mat, 'unroll', seconds=1.0, axis='Y')
    G.slide(roll, 'unroll', (0, -run, 0), seconds=1.0)
    G.squash(roll, 'unroll', (1, left, left), seconds=1.0)
    return G.meshes()


# --- the dumbbell rack: two tilted shelves of hex dumbbell pairs on A-frames, a rubber mat in front ---------
SHELVES = ((0.32, -0.08), (0.56, -0.17))   # the shelves' (height, depth): the lower, then the upper
LIFTED = (1, 0)                            # the pair lifted: the upper shelf's left pair


def _bell(k, i):
    """The width across the heads of the rack's dumbbell `i` on shelf `k`: pairs, the lower shelf's heavier."""
    return 0.1 + (i // 2) * 0.012 + (1 - k) * 0.024


def dumbbells():
    F = G.GameFrame()
    with G.new_parts() as parts:
        for s in (-1, 1):
            G.rod(f'leg_back{s}', F.at(s * 0.36, 0.0, -0.3), F.at(s * 0.36, 0.7, -0.22), 0.02, material=M.frame, verts=8)
            G.rod(f'leg_front{s}', F.at(s * 0.36, 0.0, 0.08), F.at(s * 0.36, 0.7, -0.2), 0.02, material=M.frame, verts=8)
            F.box(f'base{s}', 0.05, 0.03, 0.44, (s * 0.36, 0.015, -0.11), M.frame, bevel=0.008)
            F.box(f'top{s}', 0.05, 0.04, 0.08, (s * 0.36, 0.69, -0.21), M.dark, bevel=0.01)
        for k, (y, z) in enumerate(SHELVES):
            F.box(f'shelf{k}', 0.76, 0.025, 0.15, (0, y, z), M.frame, rot=(18, 0, 0), bevel=0.006)
            F.box(f'lip{k}', 0.76, 0.04, 0.012, (0, y + 0.015, z + 0.07), M.frame, bevel=0.004)
            for i in range(4):
                if (k, i // 2) != LIFTED:
                    w = _bell(k, i)
                    _dumbbell(F, f'dumbbell{k}{i}', -0.27 + i * 0.18, y + 0.012 + w / 2, z, w)
        F.box('floor_mat', 0.6, 0.012, 0.36, (0, 0.006, 0.2), M.dark, bevel=0.004)
    G.join(parts, 'body', origin=(0, 0, 0))
    # The lifted pair, each its own part with its origin at its middle: hidden while lifted (`pair_off`), the
    # game drawing one in each hand (content `held`).
    k, p = LIFTED
    y, z = SHELVES[k]
    pair = []
    for i, name in ((2 * p, 'dumbbell_l'), (2 * p + 1, 'dumbbell_r')):
        w = _bell(k, i)
        at = (-0.27 + i * 0.18, y + 0.012 + w / 2, z)
        with G.new_parts() as one:
            _dumbbell(F, name, *at, w)
        pair.append(G.hinged(G.join(one, name), F.at(*at)))
    G.settle(centre=False)
    for d in pair:
        G.squash(d, 'pair_off', 0.001, seconds=0.2)
    return G.meshes()


FRAME_PADS = [('body', 'Frame'), ('pads', 'Pads')]

FAMILY = G.Family(
    'fitness', M,
    [G.Model('treadmill', treadmill, size=(0.794, 1.439, 1.85), key='model.treadmill', centre=(0.032, -0.005),
             motions={'loops': {'run': 'belt_run'}}, paint=FRAME_PADS),
     G.Model('weightBench', weightBench, size=(0.9, 1.2, 1.849), key='model.weightBench', centre=(0, -0.006),
             motions={'uses': {'lift': 'bar_off'}}, paint=[('frame', 'Frame'), ('pads', 'Pads')]),
     G.Model('exerciseBike', exerciseBike, size=(0.53, 1.107, 0.9), key='model.exerciseBike',
             motions={'loops': {'cycle': 'pedals_turn'}}, paint=FRAME_PADS),
     G.Model('yogaMat', yogaMat, size=(0.919, 0.233, 0.27), key='model.yogaMat', centre=(0.025, -0.315),
             motions={'uses': {'yoga': 'unroll'}}, paint=[('mat', 'Mat')]),
     G.Model('dumbbells', dumbbells, size=(0.77, 0.717, 0.71), key='model.dumbbells', centre=(0, 0.025),
             motions={'uses': {'lift': 'pair_off'}}, paint=[('frame', 'Rack'), ('plate', 'Weights')])],
    comment='Blender-built home gym (tools/art/blender/families/fitness.py). Replaces the primitive-part '
            'placeholders by key; listed in the base manifest\'s packs. The placeholder is kept as a fallback if a '
            'file fails to load, and the materials map onto the game\'s finishes so a model costs one draw call per '
            'finish group. The treadmill runs its belt (`belt_run`) and the bike turns its pedals (`pedals_turn`) '
            'while used; the yoga mat unrolls (`unroll`) for yoga.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

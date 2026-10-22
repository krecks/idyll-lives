# The home gym: the treadmill (`model.treadmill`), the weight bench (`model.weightBench`), the exercise bike
# (`model.exerciseBike`), the yoga mat (`model.yogaMat`) and the dumbbell rack (`model.dumbbells`).
# Notes: fitness.md.
#
# Each is built where the game places it (`F`, game axes). The treadmill's belt slats run back under the
# runner while it is used (`belt_run`, a loop: one slat on, home unseen under the hood and the rear cap). The
# bike's cranks and pedals turn while it is cycled (`pedals_turn`, a loop). The yoga mat lies rolled up and
# unrolls flat for yoga (`unroll`), rolling up again at the end. The lift draws the weight in the
# hands (content `held`), so the bench's bar (`bar_off`) and a pair of the rack's dumbbells (`pair_off`) leave them.
#
# Run:  cargo art fitness [--only treadmill] [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402
from mathutils import Matrix  # noqa: E402

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
    mat_edge=('#5F836B', 'matte'),   # its rolled layers' edges and its alignment lines
    block=('#E8E2D6', 'matte'),      # the yoga block
)


def _rr(w, h, r, corner=2):
    """A rounded rectangle outline (`G.rounded_rect`), for profiles and slabs."""
    return G.rounded_rect(w, h, r, corner=corner)


def _on(c, tilt, d):
    """A game point `d` (x, y, z) from `c` on a face tilted `tilt` degrees about the game's x axis."""
    a = math.radians(tilt)
    return (c[0] + d[0], c[1] + d[1] * math.cos(a) - d[2] * math.sin(a), c[2] + d[1] * math.sin(a) + d[2] * math.cos(a))


def _plate(F, name, x, y, z, r, t, material, ring=None):
    """A weight plate on a bar along game x at (x, y, z): a turned disc `r` round and `t` thick, a raised rim, a
    dished face and a hub, both faces alike; `ring`: a coloured band inside the rim."""
    prof = [(0.0, -t / 2 + 0.006), (0.03, -t / 2 + 0.006), (0.032, -t / 2), (r - 0.012, -t / 2), (r, -t / 2 + 0.006),
            (r, t / 2 - 0.006), (r - 0.012, t / 2), (0.032, t / 2), (0.03, t / 2 - 0.006), (0.0, t / 2 - 0.006)]
    G.revolve(name, prof, at=F.at(x, y, z), material=material, segments=24, rot=(0, 90, 0))
    if ring:
        for s in (-1, 1):
            G.revolve(f'{name}_ring{s}', [(r * 0.62, 0.0), (r * 0.78, 0.0), (r * 0.78, 0.0015), (r * 0.62, 0.0015)],
                      at=F.at(x + s * (t / 2 - 0.0005), y, z), material=ring, segments=24, rot=(0, -90 * s, 0))


def _dumbbell(F, name, x, y, z, w, along='x'):
    """A hex dumbbell: a knurled chrome handle and two hexagonal heads `w` across their flats, chamfered, with
    chrome end caps, lying along game x (or z)."""
    rot = (0, 90, 0) if along == 'x' else (90, 0, 0)

    def off(k):
        return (x + k, y, z) if along == 'x' else (x, y, z + k)
    F.cyl(f'{name}_grip', 0.014, 0.1, (x, y, z), M.chrome, rot=rot, verts=8)
    F.cyl(f'{name}_knurl', 0.0145, 0.07, (x, y, z), M.frame, rot=rot, verts=8)
    r = w / 2 / math.cos(math.pi / 6)        # the hexagon's corner radius
    for s in (-1, 1):
        head = [(0.0, -0.0225), (r * 0.85, -0.0225), (r, -0.016), (r, 0.016), (r * 0.85, 0.0225), (0.0, 0.0225)]
        obj = G.revolve(f'{name}_head{s}', head, at=F.at(*off(s * 0.068)), material=M.plate, segments=6, rot=rot,
                        smooth=False)
        if along == 'x':       # a flat underneath, not a corner
            obj.data.transform(Matrix.Rotation(math.radians(30), 4, 'Z'))
        F.cyl(f'{name}_cap{s}', 0.016, 0.004, off(s * 0.0915), M.chrome, rot=rot, verts=8)


# --- the treadmill: belt along the depth, the console at the back; the runner faces the back -------------


SLAT = 0.3       # the slats' pitch: the belt's loop moves them one on
BELT_SECONDS = 0.13   # one slat on in 0.13 s (the clip, 4 frames at 30 a second): 2.25 m/s, a run
HOOD = -0.63     # the motor hood's front edge (the belt comes out from under it)
CAP = 0.80       # the rear roller cap's front edge (the belt goes under it)
CON, TILT = (0, 1.3, -0.58), -35   # the console's middle, tilted back to face the runner


def treadmill():
    F = G.GameFrame()
    with G.new_parts() as still:
        # the deck: a rounded base, the belt between two side rails capped by ribbed foot treads
        F.soft('deck', 0.66, 0.1, 1.56, (0, 0.09, 0.12), M.body, round=0.025, round_segments=2)
        F.box('belt', 0.56, 0.012, 1.5, (0, 0.146, 0.1), M.belt, bevel=0.003)
        for s in (-1, 1):
            F.sweep(f'rail{s}', [(s * 0.33, 0.165, -0.65), (s * 0.33, 0.165, 0.85)], None, M.body,
                    profile=[(-0.025, -0.035), (0.02, -0.035), (0.025, -0.03), (0.025, 0.03), (0.02, 0.035),
                             (-0.025, 0.035)], up=(0, 0, 1))
            F.box(f'tread{s}', 0.05, 0.004, 1.4, (s * 0.33, 0.1915, 0.1), M.pads, bevel=0.0015, bevel_segments=1)
            for k in range(8):
                F.box(f'tread_rib{s}{k}', 0.046, 0.002, 0.012, (s * 0.33, 0.1945, -0.5 + 0.18 * k), M.dark)
        # the rear roller cap, rounded, its roller's ends and adjusting bolts
        F.soft('cap', 0.66, 0.055, 0.12, (0, 0.15, 0.86), M.body, round=0.02, round_segments=2)
        for s in (-1, 1):
            F.cyl(f'roller_end{s}', 0.028, 0.01, (s * 0.335, 0.13, 0.86), M.dark, rot=(0, 90, 0), verts=12)
            F.cyl(f'roller_bolt{s}', 0.006, 0.012, (s * 0.335, 0.13, 0.86), M.steel, rot=(0, 90, 0), verts=6)
        # the motor hood: lofted, rounding over to the front, a band of colour round it and vents on its top
        hx, hz = F.at(0, 0, -0.78)[:2]
        rings = [(0.35, 0.15, 0.05), (0.35, 0.15, 0.2), (0.345, 0.145, 0.235), (0.33, 0.13, 0.25)]
        G.loft('hood', [G.ring(rx, rz, y, 24, square=6, at=(hx, hz)) for rx, rz, y in rings], material=M.body)
        G.loft('band', [G.ring(0.352, 0.152, y, 24, square=6, at=(hx, hz)) for y in (0.19, 0.215)], material=M.accent)
        for k in range(5):
            F.box(f'vent{k}', 0.3, 0.003, 0.012, (0, 0.25, -0.86 + 0.03 * k), M.dark)
        for x in (-0.28, 0.28):
            for z in (-0.86, 0.84):
                F.cyl(f'foot{x}{z}', 0.03, 0.04, (x, 0.02, z), M.dark, verts=10, bevel=0.006, bevel_segments=1)
        # transport wheels at the front of the hood
        for s in (-1, 1):
            F.cyl(f'wheel{s}', 0.035, 0.025, (s * 0.31, 0.037, -0.89), M.dark, rot=(0, 90, 0), verts=12, bevel=0.005,
                  bevel_segments=1)
        # the uprights: oval tubes from the hood to the console, the handlebars bent in one piece each, foam grips
        # with chrome pulse plates
        for s in (-1, 1):
            F.sweep(f'upright{s}', [(s * 0.3, 0.2, -0.8), (s * 0.3, 1.22, -0.62)], None, M.steel,
                    profile=_rr(0.07, 0.045, 0.02, 2), up=(0, 1, 0))
            bar = G.kit.fillet([(s * 0.3, 1.18, -0.63), (s * 0.33, 1.16, -0.6), (s * 0.33, 1.08, -0.38),
                                (s * 0.33, 1.06, -0.28)], 0.04)
            G.kit.tube(F, f'handlebar{s}', bar, 0.018, M.steel, verts=10, round=1)
            G.kit.tube(F, f'grip{s}', [(s * 0.33, 1.083, -0.39), (s * 0.33, 1.062, -0.29)], 0.024, M.pads, verts=10,
                       round=1)
            F.sweep(f'pulse{s}', [(s * 0.33, 1.109, -0.46), (s * 0.33, 1.087, -0.4)], 0.023, M.chrome, verts=10)
            F.sphere(f'bar_cap{s}', 0.024, (s * 0.33, 1.06, -0.285), M.pads, segments=10, rings=5)
        # the console: rounded, a screen in a bezel, a light strip, buttons, speakers and the safety key
        F.soft('console', 0.72, 0.3, 0.1, CON, M.body, rot=(TILT, 0, 0), round=0.035, round_segments=2)
        F.box('bezel', 0.38, 0.2, 0.006, _on(CON, TILT, (0, 0.03, 0.05)), M.dark, rot=(TILT, 0, 0), bevel=0.004,
              bevel_segments=1)
        F.box('screen', 0.34, 0.16, 0.006, _on(CON, TILT, (0, 0.03, 0.052)), M.screen, rot=(TILT, 0, 0), bevel=0.003)
        F.box('strip', 0.6, 0.02, 0.006, _on(CON, TILT, (0, -0.075, 0.051)), M.accent, rot=(TILT, 0, 0))
        for k, (x, m) in enumerate(((-0.25, M.red), (-0.19, M.block), (0.19, M.block), (0.25, M.accent))):
            F.box(f'button{k}', 0.04, 0.03, 0.01, _on(CON, TILT, (x, -0.11, 0.052)), m, rot=(TILT, 0, 0), bevel=0.004)
        for s in (-1, 1):
            for k in range(4):
                F.box(f'speaker{s}{k}', 0.07, 0.006, 0.004, _on(CON, TILT, (s * 0.28, 0.07 - 0.02 * k, 0.051)), M.dark,
                      rot=(TILT, 0, 0))
        F.cyl('safety_key', 0.016, 0.012, _on(CON, TILT, (0.12, -0.11, 0.056)), M.red, rot=(90 + TILT, 0, 0), verts=10)
        G.kit.tube(F, 'lanyard', [_on(CON, TILT, (0.12, -0.12, 0.06)), _on(CON, TILT, (0.13, -0.2, 0.08)),
                                  _on(CON, TILT, (0.12, -0.27, 0.07))], 0.003, M.red, verts=4, round=3)
        # a bottle in its holder
        G.revolve('holder', [(0, 0), (0.04, 0), (0.042, 0.03), (0.038, 0.03), (0.036, 0.006), (0, 0.006)],
                  at=F.at(0.39, 0.885, -0.68), material=M.steel, segments=14)
        G.revolve('bottle', [(0, 0), (0.03, 0), (0.035, 0.01), (0.035, 0.13), (0.03, 0.155), (0.022, 0.17), (0.022, 0.18),
                             (0, 0.18)], at=F.at(0.39, 0.89, -0.68), material=M.bottle, segments=14)
        F.cyl('bottle_cap', 0.022, 0.03, (0.39, 1.085, -0.68), M.dark, verts=12, bevel=0.004, bevel_segments=1)
    G.join(still, 'body', origin=(0, 0, 0))
    # The slats: the first under the hood, the last one slat short of the cap's far edge, so one slat on (the loop)
    # the first comes out from under the hood where the second was and the last goes under the cap.
    with G.new_parts() as parts:
        last = CAP + 0.105 - SLAT
        k = 0
        while last - k * SLAT > HOOD - SLAT:
            F.box(f'slat{k}', 0.54, 0.004, 0.03, (0, 0.154, last - k * SLAT), M.slat)
            k += 1
    slats = G.join(parts, 'slats')
    G.settle(centre=False)
    # Back towards the rear (game +z) at a running pace, 2.25 m/s (a slat each 0.13 s, `BELT_SECONDS`): a
    # long pitch keeps each frame's move under half of it, so the slats are seen going back, not jittering, at
    # 60 frames a second up to the fastest game speed (2.5x: 9 cm a frame of the 30).
    G.scroll(slats, 'belt_run', (0, -SLAT, 0), seconds=BELT_SECONDS)
    return G.meshes()


# --- the weight bench: a padded bench, a rack with the bar at the head end, dumbbells by the feet ----------
BAR = (0, 1.02, -0.72)   # the bar's middle on the J-hooks
PAD_TOP = 0.44           # the pad's top (characters/furniture.json `beds.weightBench`)
# `bar_off` waits while the lifter sits down on the bench (characters.ts: the `sitDown` clip but its last
# 0.35 s, 0.98 s), so the bar leaves the hooks as the game starts drawing it in the hands, lying back.
BAR_DELAY = 0.8


def weightBench():
    F = G.GameFrame(shift=0.02)   # 2 cm forward of its old part: the rack's back on the wall's face
    tube = _rr(0.06, 0.06, 0.008, 1)
    with G.new_parts() as parts:
        # A flat bench, its pad at 0.44 m (a bench-press bench is 0.43–0.45, so the lifter's feet reach the floor
        # either side; review M-fitness-01: 0.50, with leg rollers at its foot where the shins would come down).
        top = PAD_TOP
        F.soft('pad', 0.3, 0.085, 1.22, (0, top - 0.0425 - 0.004, -0.12), M.pads, round=0.025, puff=0.004, flat=2,
               round_segments=2, piping=0.004)
        F.box('board', 0.27, 0.02, 1.18, (0, top - 0.1, -0.12), M.dark, bevel=0.005, bevel_segments=1)
        F.sweep('beam', [(0, top - 0.14, -0.72), (0, top - 0.14, 0.48)], None, M.frame, profile=_rr(0.06, 0.12, 0.01, 1))
        for z in (0.4, -0.62):
            F.sweep(f'post{z}', [(0, top - 0.11, z), (0, 0.05, z)], None, M.frame, profile=_rr(0.08, 0.08, 0.01, 1),
                    up=(1, 0, 0))
            F.sweep(f'feet{z}', [(-0.23, 0.025, z), (0.23, 0.025, z)], None, M.frame, profile=_rr(0.05, 0.08, 0.01, 1))
            for s in (-1, 1):
                F.soft(f'cap{z}{s}', 0.04, 0.052, 0.085, (s * 0.25, 0.026, z), M.dark, round=0.012, round_segments=1)
            F.box(f'gusset{z}', 0.01, 0.06, 0.08, (0, 0.08, z), M.frame)
        for s in (-1, 1):
            F.cyl(f'pad_bolt{s}', 0.008, 0.006, (0, top - 0.09, -0.12 + s * 0.4), M.steel, verts=6)
        # the rack: uprights with their peg holes, feet, braces, J-hooks lined in red, a cross brace
        for s in (-1, 1):
            F.sweep(f'upright{s}', [(s * 0.36, 0.05, -0.8), (s * 0.36, 1.2, -0.8)], None, M.frame, profile=tube,
                    up=(1, 0, 0))
            F.cyl(f'upright_cap{s}', 0.026, 0.008, (s * 0.36, 1.2, -0.8), M.dark, verts=8, bevel=0.002, bevel_segments=1)
            F.sweep(f'base{s}', [(s * 0.36, 0.025, -0.95), (s * 0.36, 0.025, -0.55)], None, M.frame,
                    profile=_rr(0.05, 0.06, 0.008, 1))
            for z in (-0.94, -0.56):
                F.soft(f'base_cap{s}{z}', 0.064, 0.054, 0.02, (s * 0.36, 0.027, z), M.dark, round=0.008, round_segments=1)
            G.kit.tube(F, f'brace{s}', [(s * 0.36, 0.05, -0.58), (s * 0.36, 0.45, -0.77)], 0.015, M.frame, verts=8, round=1)
            for k in range(8):
                F.box(f'peg_hole{s}{k}', 0.016, 0.016, 0.002, (s * 0.36, 0.7 + 0.06 * k, -0.769), M.dark)
            # the J-hook: a sleeve round the upright, pinned through, and its hook in red plastic
            F.box(f'hook_sleeve{s}', 0.07, 0.1, 0.07, (s * 0.36, 0.98, -0.8), M.frame, bevel=0.004, bevel_segments=1)
            F.cyl(f'hook_pin{s}', 0.006, 0.08, (s * 0.36, 0.965, -0.8), M.chrome, rot=(0, 90, 0), verts=6)
            F.extrude(f'hook{s}', [(0.03, 0.04), (0.1, 0.04), (0.12, 0.06), (0.12, 0.1), (0.1, 0.1), (0.1, 0.075),
                                   (0.05, 0.075), (0.03, 0.09)], 0.05, (s * 0.36, 0.93, -0.8), M.red, plane='side',
                      bevel=0.004, bevel_segments=1)
        F.sweep('rack_brace', [(-0.33, 0.26, -0.8), (0.33, 0.26, -0.8)], None, M.frame, profile=_rr(0.05, 0.05, 0.008, 1))
        # two hex dumbbells on the floor by the feet
        for x in (-0.3, 0.3):
            _dumbbell(F, f'dumbbell{x}', x, 0.05, 0.842, 0.1)
    G.join(parts, 'body', origin=(0, 0, 0))
    # The bar on the J-hooks, its origin at its middle: hidden while lifted (`bar_off`), the game drawing it
    # in the lifter's hands (content `held`) once they lie down (`BAR_DELAY`).
    with G.new_parts() as parts:
        G.kit.tube(F, 'bar', [(-0.3, BAR[1], BAR[2]), (0.3, BAR[1], BAR[2])], 0.0135, M.chrome, verts=12, round=1)
        for s in (-1, 1):
            F.cyl(f'knurl{s}', 0.0138, 0.12, (s * 0.16, BAR[1], BAR[2]), M.frame, rot=(0, 90, 0), verts=12)
            F.cyl(f'sleeve{s}', 0.017, 0.16, (s * 0.37, BAR[1], BAR[2]), M.chrome, rot=(0, 90, 0), verts=12)
            F.cyl(f'shoulder{s}', 0.025, 0.012, (s * 0.296, BAR[1], BAR[2]), M.chrome, rot=(0, 90, 0), verts=12)
            _plate(F, f'plate{s}', s * 0.41, BAR[1], BAR[2], 0.16, 0.03, M.plate, ring=M.red)
            G.torus(f'collar{s}', 0.022, 0.006, at=F.at(s * 0.38, BAR[1], BAR[2]), material=M.chrome, rot=(0, 90, 0),
                    major_segments=12, minor_segments=4)
            G.kit.tube(F, f'collar_arm{s}', [(s * 0.38, BAR[1] + 0.022, BAR[2]), (s * 0.375, BAR[1] + 0.06, BAR[2] + 0.02)],
                       0.003, M.chrome, verts=4, round=1)
    bar = G.hinged(G.join(parts, 'bar'), F.at(*BAR))
    G.settle(centre=False)
    G.squash(bar, 'bar_off', 0.001, seconds=0.2)
    return G.meshes()


# --- the exercise bike: an upright bike, the flywheel at the front, the cranks under the saddle -------------
CRANK = (0, 0.3, 0.04)   # the crank axle
ARM = 0.16               # the cranks' length


def exerciseBike():
    F = G.GameFrame()
    frame = _rr(0.06, 0.05, 0.016, 2)
    with G.new_parts() as still:
        # the base: a beam on two stabilisers with rubber end caps and levelling feet, transport wheels at the front
        F.sweep('beam', [(0, 0.05, -0.43), (0, 0.05, 0.43)], None, M.body, profile=_rr(0.06, 0.12, 0.014, 2))
        for z in (-0.38, 0.38):
            F.sweep(f'stabiliser{z}', [(-0.22, 0.03, z), (0.22, 0.03, z)], None, M.body, profile=_rr(0.05, 0.08, 0.016, 2))
            for s in (-1, 1):
                F.soft(f'cap{z}{s}', 0.05, 0.035, 0.085, (s * 0.24, 0.018, z), M.dark, round=0.012, round_segments=1)
        # the frame: one swept piece from the beam up through the bottom bracket to the seat post, the handlebar
        # post curving up from the front, the flywheel's shroud between
        G.kit.tube(F, 'down_tube', [(0, 0.62, -0.11), CRANK], 0.024, M.body, verts=10, round=1)
        F.sweep('seat_tube', [(0, 0.06, -0.14), (0, 0.6, -0.105)], None, M.body, profile=frame, up=(1, 0, 0))
        G.kit.tube(F, 'seat_post', [(0, 0.58, -0.106), (0, 0.7, -0.1)], 0.02, M.steel, verts=10, round=1)
        F.cyl('seat_clamp', 0.034, 0.03, (0, 0.6, -0.105), M.dark, rot=(-3, 0, 0), verts=10, bevel=0.004, bevel_segments=1)
        G.kit.tube(F, 'seat_knob_stem', [(0, 0.585, -0.08), (0, 0.585, -0.04)], 0.006, M.steel, verts=6, round=1)
        G.revolve('seat_knob', [(0, 0), (0.016, 0), (0.02, 0.008), (0.018, 0.02), (0, 0.022)], at=F.at(0, 0.585, -0.04),
                  material=M.dark, segments=10, rot=(90, 0, 0))
        F.sweep('handle_post', [(0, 0.06, 0.22), (0, 0.5, 0.27), (0, 0.95, 0.32)], None, M.body, profile=frame,
                up=(1, 0, 0), resolution=2)
        F.cyl('bracket', 0.045, 0.1, CRANK, M.body, rot=(0, 90, 0), verts=16, bevel=0.008, bevel_segments=1)
        # the flywheel in its shroud: a dark disc with chrome spokes, covers either side
        F.cyl('flywheel', 0.2, 0.05, (0, 0.3, 0.27), M.dark, rot=(0, 90, 0), verts=32, bevel=0.01)
        for s in (-1, 1):
            G.revolve(f'flywheel_cover{s}', [(0, 0.012), (0.1, 0.01), (0.13, 0.004), (0.135, 0.0)],
                      at=F.at(s * 0.026, 0.3, 0.27), material=M.body, segments=28, rot=(0, -90 * s, 0))
            F.cyl(f'flywheel_hub{s}', 0.02, 0.012, (s * 0.04, 0.3, 0.27), M.steel, rot=(0, 90, 0), verts=10)
        G.kit.tube(F, 'resistance_cable', [(0, 0.95, 0.31), (0.04, 0.8, 0.35), (0.03, 0.55, 0.36), (0, 0.5, 0.33)],
                   0.004, M.dark, verts=4, round=3)
        # the saddle: a lofted seat, wide at the back, narrowing to its nose, a rail under it
        sections = [[F.at(u * w / 2, y, v) for u, v in _saddle_ring(back, nose)]
                    for y, w, back, nose in ((0.7, 0.1, -0.2, 0.03), (0.72, 0.19, -0.215, 0.05), (0.745, 0.2, -0.22, 0.06),
                                             (0.758, 0.17, -0.205, 0.05))]
        G.loft('saddle', sections, material=M.pads)
        G.kit.tube(F, 'saddle_rail', [(0, 0.705, -0.2), (0, 0.705, -0.04)], 0.008, M.steel, verts=6, round=1)
        # the handlebars bent in one piece, foam grips, a console with its screen and buttons
        bar = [(-0.22, 1.0, 0.16), (-0.22, 0.98, 0.3), (0.22, 0.98, 0.3), (0.22, 1.0, 0.16)]
        G.kit.tube(F, 'bar', G.kit.fillet(bar, 0.05), 0.017, M.steel, verts=10, round=1)
        for s in (-1, 1):
            G.kit.tube(F, f'grip{s}', [(s * 0.22, 0.992, 0.24), (s * 0.22, 1.0, 0.15)], 0.022, M.pads, verts=10, round=1)
            F.sphere(f'grip_end{s}', 0.022, (s * 0.22, 1.0, 0.15), M.pads, segments=10, rings=5)
        F.soft('console', 0.2, 0.12, 0.04, (0, 1.05, 0.36), M.dark, rot=(-30, 0, 0), round=0.012, round_segments=1)
        F.box('screen', 0.16, 0.08, 0.004, _on((0, 1.05, 0.36), -30, (0, 0.01, 0.021)), M.accent, rot=(-30, 0, 0))
        for k in range(3):
            F.box(f'button{k}', 0.025, 0.01, 0.006, _on((0, 1.05, 0.36), -30, (-0.04 + 0.04 * k, -0.045, 0.021)),
                  [M.red, M.block, M.block][k], rot=(-30, 0, 0), bevel=0.002, bevel_segments=1)
    G.join(still, 'body', origin=(0, 0, 0))
    # The cranks and pedals, joined round the axle so they turn about it.
    with G.new_parts() as parts:
        for s, a in ((1, 0), (-1, 180)):
            end = (s * 0.075, CRANK[1] - ARM * G.cosd(a), CRANK[2] + ARM * G.sind(a))
            F.sweep(f'crank{s}', [(s * 0.075, *CRANK[1:]), end], None, M.steel, profile=_rr(0.014, 0.034, 0.006, 1),
                    up=(1, 0, 0), scale=[1.0, 0.8])
            F.cyl(f'pedal{s}', 0.022, 0.1, (s * 0.13, end[1], end[2]), M.dark, rot=(0, 90, 0), verts=10)
            for e in (-1, 1):
                F.cyl(f'pedal_end{s}{e}', 0.024, 0.008, (s * 0.13 + e * 0.046, end[1], end[2]), M.steel, rot=(0, 90, 0),
                      verts=10)
        F.cyl('axle', 0.018, 0.16, CRANK, M.steel, rot=(0, 90, 0), verts=10)
        F.cyl('chainring', 0.06, 0.008, (0.06, *CRANK[1:]), M.steel, rot=(0, 90, 0), verts=16)
    pedals = G.hinged(G.join(parts, 'pedals'), F.at(*CRANK))
    G.settle()      # on its Model centre: the back on the wall face
    G.spin(pedals, 'pedals_turn', turns=1, seconds=1.2, axis='X')
    return G.meshes()


def _saddle_ring(back, nose, n=16):
    """A saddle's outline seen from above (game x as a fraction of half the width, depth): round at the back,
    narrowing to a nose, `n` points."""
    out = []
    for i in range(n):
        a = math.tau * i / n
        u, v = math.cos(a), math.sin(a)
        d = back + (nose - back) * (v + 1) / 2
        out.append((u * (1 - 0.65 * max(0.0, v) ** 1.5), d))
    return out


# --- the yoga mat: rolled up at the back with a block and a bottle; it unrolls to the front -----------------
MAT_W, MAT_L, MAT_T = 0.62, 1.8, 0.016   # the mat, flat (the house's floor is drawn 1 cm up): a full 1.8 m mat
                                         # on a 1 x 2 footprint (review M-fitness-02: it unrolled to 0.9)
ROLL_R = 0.065                           # the roll's radius
BACK = -0.93                             # the mat's back edge (game depth): on the wall's face


def _spiral(r0, r1, turns, width, n=40):
    """A rolled mat's spiral seen end on (game depth, height about its axis): a band `width` wide winding out from
    `r0` to `r1` in `turns` turns, as one closed outline."""
    outer, inner = [], []
    for i in range(n + 1):
        t = i / n
        a = math.tau * turns * t
        r = r0 + (r1 - r0) * t
        outer.append(((r + width / 2) * math.cos(a), (r + width / 2) * math.sin(a)))
        inner.append(((r - width / 2) * math.cos(a), (r - width / 2) * math.sin(a)))
    return outer + inner[::-1]


def yogaMat():
    F = G.GameFrame()
    with G.new_parts() as still:
        # a foam block, its edges soft, and a bottle with a loop on its cap
        F.soft('block', 0.15, 0.12, 0.24, (0.41, 0.06, BACK + 0.15), M.block, round=0.02, round_segments=2)
        G.revolve('bottle', [(0, 0), (0.03, 0), (0.035, 0.008), (0.035, 0.16), (0.03, 0.18), (0.02, 0.19), (0.02, 0.2),
                             (0, 0.2)], at=F.at(-0.4, 0.0, BACK + 0.13), material=M.bottle, segments=14)
        F.cyl('bottle_label', 0.0358, 0.06, (-0.4, 0.09, BACK + 0.13), M.block, verts=14)
        F.cyl('bottle_cap', 0.022, 0.035, (-0.4, 0.2125, BACK + 0.13), M.dark, verts=12, bevel=0.004, bevel_segments=1)
        G.torus('bottle_loop', 0.012, 0.0025, at=F.at(-0.4, 0.2305, BACK + 0.13 - 0.018), material=M.dark,
                major_segments=10, minor_segments=4)
    G.join(still, 'body', origin=(0, 0, 0))
    # The roll lies at the back, its origin on the floor under its axis: it shrinks onto the floor as it goes. Its
    # ends show the mat's spiral; two straps round it.
    roll_z = BACK + ROLL_R
    with G.new_parts() as parts:
        F.cyl('roll', ROLL_R, MAT_W, (0, ROLL_R, roll_z), M.mat, rot=(0, 90, 0), verts=24, bevel=0.004)
        F.cyl('roll_core', ROLL_R * 0.2, MAT_W + 0.002, (0, ROLL_R, roll_z), M.dark, rot=(0, 90, 0), verts=10)
        for s in (-1, 1):
            F.extrude(f'roll_spiral{s}', _spiral(ROLL_R * 0.2, ROLL_R - 0.006, 3.0, 0.003), 0.002,
                      (s * (MAT_W / 2 + 0.0005), ROLL_R, roll_z), M.mat_edge, plane='side')
            F.cyl(f'strap{s}', ROLL_R + 0.0008, 0.025, (s * 0.2, ROLL_R, roll_z), M.dark, rot=(0, 90, 0), verts=24)
    roll = G.hinged(G.join(parts, 'roll'), F.at(0, 0, roll_z))
    # The flat mat, its origin under the roll's axis, built full and squeezed to nothing for rest: inside the roll,
    # out of sight (its back edge showed as a strip). Its corners rounded, alignment lines along it.
    with G.new_parts() as parts:
        F.extrude('mat', _rr(MAT_W, MAT_L, 0.02, 2), MAT_T, (0, MAT_T / 2, BACK + MAT_L / 2), M.mat, plane='top')
        for x in (-0.2, 0.2):
            F.box(f'mat_line{x}', 0.006, 0.001, MAT_L - 0.2, (x, MAT_T + 0.0004, BACK + MAT_L / 2), M.mat_edge)
        F.box('mat_line_mid', MAT_W - 0.2, 0.001, 0.006, (0, MAT_T + 0.0004, BACK + MAT_L / 2), M.mat_edge)
    mat = G.hinged(G.join(parts, 'mat'), F.at(0, 0, roll_z))
    mat.scale[1] = 0.001
    G.settle()      # on its Model centre: the back on the wall face
    left = 0.12   # what is left of the roll at the front end: a curled edge
    run = MAT_L - ROLL_R - left * ROLL_R
    G.grow(mat, 'unroll', seconds=1.0, axis='Y')
    G.slide(roll, 'unroll', (0, -run, 0), seconds=1.0)
    G.squash(roll, 'unroll', (1, left, left), seconds=1.0)
    return G.meshes()


# --- the dumbbell rack: two tilted shelves of hex dumbbell pairs on A-frames, a rubber mat in front ---------
# The rack stands back on the wall's face and its rubber mat fills the front of the tile, where the lifter steps
# in to stand (characters.ts `STAND_BACK`; review I-fitness-02: they stood on the tile in front, the mat empty
# between them and the rack).
RACK_SHIFT = -0.1
SHELVES = ((0.32, -0.08), (0.56, -0.17))   # the shelves' (height, depth): the lower, then the upper
LIFTED = (1, 0)                            # the pair lifted: the upper shelf's left pair


def _bell(k, i):
    """The width across the heads of the rack's dumbbell `i` on shelf `k`: pairs, the lower shelf's heavier."""
    return 0.1 + (i // 2) * 0.012 + (1 - k) * 0.024


def dumbbells():
    F = G.GameFrame(shift=RACK_SHIFT)   # the rack back against the wall's face, the mat in front of it
    tilt = math.radians(18)
    tray = [(-0.075, -0.0125), (0.075, -0.0125), (0.075, 0.03), (0.063, 0.03), (0.063, 0.0125), (-0.075, 0.0125)]
    with G.new_parts() as parts:
        # the A-frames: each one square tube bent over the top, on a foot rail with rubber caps, a cap on the apex
        for s in (-1, 1):
            x = s * 0.36
            frame = G.kit.fillet([(x, 0.012, 0.08), (x, 0.7, -0.21), (x, 0.012, -0.3)], 0.05)
            F.sweep(f'frame{s}', frame, None, M.frame, profile=_rr(0.04, 0.04, 0.008, 1), up=(1, 0, 0))
            F.sweep(f'base{s}', [(x, 0.015, -0.33), (x, 0.015, 0.11)], None, M.frame, profile=_rr(0.03, 0.05, 0.008, 1))
            for z in (-0.32, 0.1):
                F.soft(f'base_cap{s}{z}', 0.054, 0.034, 0.02, (x, 0.017, z), M.dark, round=0.008, round_segments=1)
            F.soft(f'top{s}', 0.05, 0.04, 0.08, (x, 0.69, -0.21), M.dark, round=0.012, round_segments=1)
        # the shelves: trays tilted to the front with a lip, rubber-lined, end caps; a cross brace low down
        for k, (y, z) in enumerate(SHELVES):
            prof = [(z + d * math.cos(tilt) + h * math.sin(tilt), y - d * math.sin(tilt) + h * math.cos(tilt))
                    for d, h in tray]
            F.extrude(f'shelf{k}', prof, 0.76, (0, 0, 0), M.frame, plane='side', bevel=0.003, bevel_segments=1)
            d, h = -0.006, 0.0135
            F.box(f'liner{k}', 0.74, 0.002, 0.126, (0, y - d * math.sin(tilt) + h * math.cos(tilt),
                                                   z + d * math.cos(tilt) + h * math.sin(tilt)), M.dark, rot=(18, 0, 0))
            for s in (-1, 1):
                F.box(f'shelf_end{s}{k}', 0.012, 0.05, 0.15, (s * 0.384, y + 0.004, z), M.dark, rot=(18, 0, 0),
                      bevel=0.003, bevel_segments=1)
            for i in range(4):
                if (k, i // 2) != LIFTED:
                    w = _bell(k, i)
                    _dumbbell(F, f'dumbbell{k}{i}', -0.27 + i * 0.18, y + 0.012 + w / 2, z, w)
        F.sweep('brace', [(-0.34, 0.12, -0.25), (0.34, 0.12, -0.25)], None, M.frame, profile=_rr(0.03, 0.03, 0.006, 1))
        # the rubber floor mat in front, in tiles
        mat = G.GameFrame()
        mat.box('floor_mat', 0.7, 0.014, 0.48, (0, 0.007, 0.26), M.dark, bevel=0.004)
        mat.box('mat_seam_x', 0.69, 0.001, 0.003, (0, 0.0144, 0.26), M.plate)
        mat.box('mat_seam_z', 0.003, 0.001, 0.47, (0, 0.0144, 0.26), M.plate)
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
             motions={'uses': {'lift': {'clip': 'bar_off', 'delay': BAR_DELAY}}}, paint=[('frame', 'Frame'), ('pads', 'Pads')]),
     G.Model('exerciseBike', exerciseBike, size=(0.53, 1.107, 0.9), centre=(0, 0.02), key='model.exerciseBike',
             motions={'loops': {'cycle': 'pedals_turn'}}, paint=FRAME_PADS),
     G.Model('yogaMat', yogaMat, size=(0.919, 0.233, 0.27), key='model.yogaMat', centre=(0.025, BACK + 0.135),
             motions={'uses': {'yoga': 'unroll'}}, paint=[('mat', 'Mat')]),
     G.Model('dumbbells', dumbbells, size=(0.77, 0.717, 0.93), key='model.dumbbells', centre=(0, 0.035),
             motions={'uses': {'lift': 'pair_off'}}, paint=[('frame', 'Rack'), ('plate', 'Weights')])],
    comment='Blender-built home gym (tools/art/blender/families/fitness.py). Listed in the base '
            'manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw call '
            'per finish group. The treadmill runs its belt (`belt_run`) and the bike turns its pedals '
            '(`pedals_turn`) while used; the yoga mat unrolls (`unroll`) for yoga.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

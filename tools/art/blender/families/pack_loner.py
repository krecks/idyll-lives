# The loner pack (P11): the hush-shell reading pod, the shortwave radio desk, the jigsaw puzzle table, the
# bonsai potting bench, the cedar sauna for one and the no-knock delivery hatch, over their primitive-part
# placeholders (`tools/art/pack_loner.py`, `packs/loner/manifest.json`), which stay the fallback.
# Notes: pack_loner.md.
#
# Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`): the pod's hood swings half closed (`shell`), the radio's tuning dial turns (`dial`) and its
# antenna extends (`antenna`), a patch of the puzzle grows (`pieces`), a twig of the bonsai shrinks away
# (`trim`), the sauna's glass door opens and shuts (`door_open`) and the delivery hatch opens (`hatch`).
#
# Run:  python3 -B tools/art/blender/build.py pack_loner [--only quietPod] [--preview]
import os
import random
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    body=('#8FA58A', 'satin'),       # sage paint: the radio's case, the puzzle table's frame, the bonsai bench's frame
    shell=('#8FA58A', 'fabric'),     # the reading pod's upholstered shell and hood
    slate=('#7E93A3', 'satin'),      # the delivery hatch's cabinet
    oak=('#C49A6C', 'satin'),        # plain oak: the pod's foot, the radio desk and chair
    wood=('#C49A6C', 'wood'),        # textured oak: the puzzle table's top and chair, the hatch's ledge and cap
    teak=('#B07A4E', 'satin'),       # the bonsai bench's top and shelf
    cedar=('#C98B58', 'wood'),       # the sauna
    cedar_dark=('#A86E44', 'satin'), # the sauna's door handle, the bucket's ladle
    cushion=('#E6DCC8', 'satin'),    # oat cushions and linings
    headrest=('#D8CCB4', 'satin'),
    seatpad=('#7E93A3', 'satin'),    # the radio chair's slate pad
    chrome=('#D0D3D6', 'chrome'),
    steel=('#C9CDD1', 'metal'),      # the hatch door, the magnifier ring
    iron=('#7E8389', 'metal'),       # the sauna heater
    copper=('#B87333', 'metal'),
    dark=('#3A3D41', 'satin'),       # knobs, headphones, the lamp foot, the hatch's recess
    rig_face=('#C9C3B0', 'satin'),   # the receiver's front plate
    dial=('#F2B84B', 'gloss'),       # the receiver's dial window
    needle=('#C8402F', 'gloss'),
    cream=('#F4EFE2', 'satin'),      # paper, logbook, meter, plaque, towel
    mustard=('#D6A64A', 'satin'),
    terra=('#C0714C', 'satin'),      # terracotta pots, the mug
    glaze=('#3E6F9E', 'gloss'),      # the bonsai's pot, the puzzle box
    soil=('#5C4433', 'matte'),
    trunk=('#6E5340', 'satin'),      # the bonsai's trunk, plain (its leaves take the textured draw)
    leaf=('#4F7F3E', 'foliage'),
    sack=('#C9B48C', 'matte'),
    porcelain=('#F2F2F0', 'gloss'),
    lens=('#DCE6EA', 'gloss'),       # the magnifier's lens, plain (no glass draw)
    stones=('#8C8A85', 'matte'),
    glass=('#DCE6E8', 'glass'),      # the sauna's door
    lamp=('#FFE2B0', 'gloss'),
    kraft=('#C9A27A', 'matte'),      # the delivery bag
    screen=('#1F3A40', 'gloss'),
    screen_a=('#7FD1C4', 'gloss'),
    mat=('#8E7F6A', 'matte'),
    # the puzzle picture's bands: sky, sea, hills, fields
    sky_a=('#9CC8E6', 'satin'), sky_b=('#7FB6DE', 'satin'),
    sea_a=('#3E7FAE', 'satin'), sea_b=('#4C8FB8', 'satin'),
    hill_a=('#6E9E58', 'satin'), hill_b=('#7FAF63', 'satin'),
    field_a=('#D9B45A', 'satin'), field_b=('#E3C46E', 'satin'),
)

BANDS = (('sky_a', 'sky_b'), ('sea_a', 'sea_b'), ('hill_a', 'hill_b'), ('field_a', 'field_b'))   # by name
WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
WALL2 = -0.93  # in a 2-deep one
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


def _m(name):
    return getattr(M, name)


def _chair(F, z, wood, pad):
    """A spindle chair facing the front, its seat (top 0.45) centred on (0, z): the resident's seat."""
    F.box(_name('chair_seat'), 0.42, 0.04, 0.4, (0, 0.41, z), wood, bevel=0.012)
    F.box(_name('chair_pad'), 0.38, 0.03, 0.36, (0, 0.445, z + 0.01), pad, bevel=0.012, bevel_segments=3)
    for x in (-0.18, 0.18):
        F.cone(_name('chair_leg'), 0.018, 0.014, 0.39, (x, 0.195, z + 0.16), wood, verts=10)
        G.rod(_name('chair_post'), F.at(x, 0.0, z - 0.17), F.at(x, 0.92, z - 0.19), 0.018, material=wood, verts=10)
    F.box(_name('chair_crest'), 0.42, 0.1, 0.035, (0, 0.87, z - 0.19), wood, bevel=0.015)
    for x in (-0.09, 0.0, 0.09):
        G.rod(_name('spindle'), F.at(x, 0.43, z - 0.18), F.at(x, 0.82, z - 0.19), 0.009, material=wood, verts=8)
    for x in (-0.18, 0.18):                      # side stretchers
        G.rod(_name('stretcher'), F.at(x, 0.14, z + 0.16), F.at(x, 0.14, z - 0.17), 0.009, material=wood, verts=8)


def _desk(F, z0, z1, w, h, top, legs):
    """A plain desk from depth z0 to z1, `h` high: top, back apron and tapered legs."""
    zc, d = (z0 + z1) / 2, z1 - z0
    F.box(_name('desk_top'), w, 0.03, d, (0, h - 0.015, zc), top, bevel=0.008)
    F.box(_name('desk_apron'), w - 0.08, 0.08, 0.02, (0, h - 0.07, z1 - 0.03), legs, bevel=0.005)
    for x in (-w / 2 + 0.035, w / 2 - 0.035):
        F.box(_name('desk_apron_side'), 0.02, 0.08, d - 0.08, (x, h - 0.07, zc), legs, bevel=0.005)
        for z in (z0 + 0.035, z1 - 0.035):
            F.cone(_name('desk_leg'), 0.02, 0.014, h - 0.03, (x, (h - 0.03) / 2, z), legs, verts=10)


def _mug(F, x, y, z, material):
    G.revolve(_name('mug'), [(0, 0), (0.036, 0), (0.04, 0.09), (0.034, 0.09), (0.03, 0.012), (0, 0.012)],
              at=F.at(x, y, z), material=material, segments=16)
    G.torus(_name('mug_handle'), 0.022, 0.007, at=F.at(x + 0.043, y + 0.045, z), material=material, rot=(90, 0, 0),
            major_segments=12, minor_segments=6)


# --- the hush-shell reading pod (1x1): an upholstered shell chair on a swivel foot, a pram hood over it -------
HOOD = (1.1, -0.02)   # the hood's pivot (height, depth): it turns about a line across the pod's wings


def quietPod():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.cyl('foot', 0.25, 0.04, (0, 0.02, -0.04), M.oak, verts=28, bevel=0.012)
        F.cyl('stem', 0.05, 0.16, (0, 0.12, -0.04), M.chrome, verts=16)
        F.box('bucket', 0.82, 0.24, 0.74, (0, 0.32, -0.06), M.shell, bevel=0.1, bevel_segments=3)
        F.box('seat', 0.6, 0.1, 0.56, (0, 0.4, 0.02), M.cushion, bevel=0.045, bevel_segments=3)   # top 0.45
        F.box('back', 0.82, 1.04, 0.14, (0, 0.88, WALL + 0.07), M.shell, bevel=0.06, bevel_segments=3)   # up into the hood
        F.box('back_cushion', 0.58, 0.56, 0.1, (0, 0.76, -0.29), M.cushion, rot=(-6, 0, 0), bevel=0.04,
              bevel_segments=3)
        F.box('head_cushion', 0.4, 0.16, 0.1, (0, 1.08, -0.31), M.headrest, bevel=0.04, bevel_segments=3)
        for s in (-1, 1):
            # the wings, from the seat up to the hood, their front edges rounded
            F.box(_name('wing'), 0.11, 0.86, 0.6, (s * 0.355, 0.78, -0.13), M.shell, bevel=0.05, bevel_segments=3)
            F.cyl(_name('wing_edge'), 0.055, 0.86, (s * 0.355, 0.78, 0.17), M.shell, verts=14)
            F.box(_name('wing_lining'), 0.02, 0.6, 0.48, (s * 0.292, 0.8, -0.12), M.cushion, bevel=0.008)
            F.cyl(_name('hood_boss'), 0.05, 0.03, (s * 0.4, HOOD[0], HOOD[1]), M.oak, rot=(0, 90, 0), verts=16)
        # a book and a folded blanket on the seat, a reading lamp on the right wing
        F.box('book', 0.2, 0.03, 0.15, (0.18, 0.465, 0.16), M.mustard, rot=(0, 0, 15), bevel=0.004)
        F.box('blanket', 0.36, 0.03, 0.26, (-0.1, 0.465, 0.13), M.headrest, rot=(0, 0, -8), bevel=0.01)
        F.box('lamp_arm', 0.02, 0.02, 0.12, (0.29, 1.05, -0.05), M.chrome)
        F.sphere('lamp', 0.04, (0.27, 1.03, 0.02), M.lamp, segments=12, rings=8)
    G.join(still, 'body', origin=(0, 0, 0))
    # The pram hood: an arc of padded ribs over the back, about a line across the wings. `shell` swings it
    # forward over the reader's head, half closing the pod's mouth.
    with G.new_parts() as hood:
        r, n = 0.4, 6
        for k in range(n):
            a = -90 + (k + 0.5) * 90 / n                    # from behind (-90) to straight up (0)
            y = HOOD[0] + r * G.cosd(a)
            z = HOOD[1] + r * G.sind(a)
            F.box(_name('rib'), 0.84, 0.075, r * 3.1416 / 2 / n + 0.03, (0, y, z), M.shell, rot=(a, 0, 0),
                  bevel=0.03, bevel_segments=2)
        F.box('hood_lip', 0.84, 0.05, 0.05, (0, HOOD[0] + r, HOOD[1] + 0.01), M.shell, bevel=0.022, bevel_segments=2)
    hood = G.hinged(G.join(hood, 'hood'), F.at(0, *HOOD))
    objs = G.settle(centre=False)
    G.swing(hood, 'shell', 40, seconds=0.9, axis='X')
    return objs


# --- the shortwave radio desk (1x1): seated in the middle, the rig on a shallow desk before them --------------
DIAL = (0.15, 0.87, 0.243)    # the tuning knob, on the receiver's face towards the operator
ANTENNA = (0.2, 0.42)         # the telescopic antenna's (x, depth), at the receiver's back corner


def shortwaveRadio():
    F = G.GameFrame()
    rnd = random.Random(4)
    with G.new_parts() as still:
        _desk(F, 0.16, 0.45, 0.84, 0.74, M.oak, M.oak)
        _chair(F, -0.02, M.oak, M.seatpad)
        # the receiver: its face (dial window, meter, knobs, speaker) towards the operator, vents to the room
        F.box('receiver', 0.5, 0.24, 0.2, (0.02, 0.86, 0.35), M.body, bevel=0.02, bevel_segments=2)
        F.box('face', 0.46, 0.2, 0.01, (0.02, 0.86, 0.248), M.rig_face, bevel=0.004)
        F.box('dial_window', 0.22, 0.07, 0.006, (-0.07, 0.92, 0.243), M.dial, bevel=0.003)
        F.box('dial_needle', 0.004, 0.06, 0.004, (-0.03, 0.92, 0.239), M.needle)
        F.box('meter', 0.07, 0.05, 0.006, (0.15, 0.95, 0.243), M.cream, bevel=0.003)
        for x in (-0.15, -0.09, -0.03, 0.03):
            F.cyl(_name('knob'), 0.014, 0.02, (x, 0.8, 0.236), M.dark, rot=(90, 0, 0), verts=12)
        for k in range(5):
            F.box(_name('grille'), 0.06, 0.004, 0.004, (-0.18, 0.88 - 0.012 * k, 0.242), M.dark)
        for k in range(6):                                        # vents on its back, which faces the room
            F.box(_name('vent'), 0.3, 0.008, 0.004, (0.0, 0.8 + 0.024 * k, 0.452), M.dark)
        F.box('foot_l', 0.04, 0.012, 0.18, (-0.19, 0.746, 0.35), M.dark)
        F.box('foot_r', 0.04, 0.012, 0.18, (0.23, 0.746, 0.35), M.dark)
        for x in (-0.12, 0.16):                                   # the carry handle on top
            G.rod(_name('handle_post'), F.at(x, 0.98, 0.35), F.at(x, 1.02, 0.35), 0.008, material=M.chrome, verts=8)
        G.rod('handle', F.at(-0.12, 1.02, 0.35), F.at(0.16, 1.02, 0.35), 0.01, material=M.chrome, verts=10)
        # the antenna's swivel and outer tube
        F.sphere('antenna_swivel', 0.018, (ANTENNA[0], 0.985, ANTENNA[1]), M.chrome, segments=10, rings=6)
        G.rod('antenna_tube', F.at(ANTENNA[0], 0.99, ANTENNA[1]), F.at(ANTENNA[0], 1.24, ANTENNA[1]), 0.008,
              material=M.chrome, verts=8)
        # the desk microphone, headphones, the logbook and pencil, a mug
        F.cyl('mic_base', 0.05, 0.02, (-0.3, 0.75, 0.28), M.dark, verts=16)
        G.rod('mic_stem', F.at(-0.3, 0.76, 0.28), F.at(-0.28, 0.9, 0.27), 0.008, material=M.chrome, verts=8)
        F.cyl('mic_capsule', 0.025, 0.09, (-0.275, 0.95, 0.265), M.steel, verts=12, bevel=0.01)
        G.torus('headphone_band', 0.09, 0.01, at=F.at(0.3, 0.84, 0.25), material=M.dark, rot=(0, 90, 0),
                major_segments=16, minor_segments=6)
        for z in (0.17, 0.33):
            F.cyl(_name('earcup'), 0.035, 0.04, (0.3, 0.77, z), M.seatpad, verts=14)
        F.box('logbook', 0.16, 0.012, 0.2, (-0.15, 0.746, 0.21), M.cream, rot=(0, 0, 10), bevel=0.003)
        F.box('pencil', 0.008, 0.008, 0.14, (-0.1, 0.756, 0.21), M.mustard, rot=(0, 0, 30))
        _mug(F, 0.33, 0.74, 0.4, M.terra)
        del rnd
    G.join(still, 'body', origin=(0, 0, 0))
    # The tuning knob: a big ridged knob with a pointer, turned by `dial`.
    with G.new_parts() as dial:
        F.cyl('dial_knob', 0.042, 0.03, (DIAL[0], DIAL[1], DIAL[2] - 0.012), M.dark, rot=(90, 0, 0), verts=20,
              bevel=0.006)
        F.box('dial_pointer', 0.008, 0.03, 0.006, (DIAL[0], DIAL[1] + 0.02, DIAL[2] - 0.029), M.cream)
    dial = G.hinged(G.join(dial, 'dial'), F.at(*DIAL))
    # The antenna's inner section, down in the tube (and the case) at rest: `antenna` runs it up 45 cm.
    with G.new_parts() as ant:
        G.rod('antenna_mid', F.at(ANTENNA[0], 0.77, ANTENNA[1]), F.at(ANTENNA[0], 1.31, ANTENNA[1]), 0.0055,
              material=M.chrome, verts=8)
        F.sphere('antenna_ball', 0.01, (ANTENNA[0], 1.315, ANTENNA[1]), M.chrome, segments=8, rings=6)
    ant = G.hinged(G.join(ant, 'antenna'), F.at(ANTENNA[0], 1.0, ANTENNA[1]))
    objs = G.settle(centre=False)
    G.swing(dial, 'dial', 150, seconds=1.0, axis='Y')
    G.slide(ant, 'antenna', (0, 0, 0.45), seconds=1.0)
    return objs


# --- the jigsaw puzzle table (1x1): seated in the middle, a puzzle half done on the table before them ---------
COLS, ROWS, PW, PD = 7, 4, 0.087, 0.067      # the puzzle's grid: piece pitch across and deep
PX0, PZ0 = -0.29, 0.38                        # the first piece's middle (back-left as the puzzler sees it)
PATCH = {(4, 1), (5, 1), (6, 1), (4, 2), (5, 2), (6, 2), (5, 3)}   # the pieces `pieces` lays
PY = 0.729                                    # the pieces' middle height


def _piece(F, i, j):
    F.box(_name('piece'), PW - 0.004, 0.008, PD - 0.004, (PX0 + i * PW, PY, PZ0 - j * PD),
          _m(BANDS[j][(i + j) % 2]))


def puzzleTable():
    F = G.GameFrame()
    with G.new_parts() as still:
        _desk(F, 0.1, 0.45, 0.86, 0.72, M.wood, M.body)
        _chair(F, -0.06, M.wood, M.body)
        F.box('puzzle_mat', 0.68, 0.004, 0.32, (0.0, 0.722, 0.28), M.dark)
        for i in range(COLS):
            for j in range(ROWS):
                if (i, j) not in PATCH and (i, j) not in {(0, 2), (2, 3)}:
                    _piece(F, i, j)
        for x, z, r, m in [(0.38, 0.19, 20, 'sky_a'), (0.34, 0.14, -35, 'hill_b'), (0.4, 0.12, 60, 'field_a'),
                           (-0.37, 0.16, 10, 'sea_a'), (-0.33, 0.13, -50, 'field_b')]:
            F.box(_name('loose'), 0.06, 0.008, 0.05, (x, 0.726, z), _m(m), rot=(0, 0, r))
        # the box on the floor, its lid showing the picture
        F.box('puzzle_box', 0.3, 0.04, 0.22, (0.05, 0.02, 0.32), M.glaze, bevel=0.004)
        F.box('box_picture', 0.26, 0.004, 0.18, (0.05, 0.041, 0.32), M.field_b)
        F.box('box_sky', 0.26, 0.005, 0.07, (0.05, 0.041, 0.375), M.sky_a)
        # a magnifier lamp clamped at the back-left corner, a cup of tea at the right
        F.cyl('lamp_foot', 0.05, 0.015, (-0.36, 0.728, 0.4), M.dark, verts=16)
        G.rod('lamp_arm_lo', F.at(-0.36, 0.73, 0.4), F.at(-0.32, 1.0, 0.38), 0.009, material=M.chrome, verts=8)
        G.rod('lamp_arm_hi', F.at(-0.32, 1.0, 0.38), F.at(-0.2, 0.97, 0.31), 0.008, material=M.chrome, verts=8)
        G.torus('lamp_ring', 0.07, 0.014, at=F.at(-0.17, 0.95, 0.29), material=M.steel, rot=(-30, 0, 0),
                major_segments=20, minor_segments=6)
        F.cyl('lamp_lens', 0.062, 0.006, (-0.17, 0.95, 0.29), M.lens, rot=(-30, 0, 0), verts=20)
        _mug(F, 0.36, 0.72, 0.38, M.porcelain)
    G.join(still, 'body', origin=(0, 0, 0))
    # The patch of pieces the puzzler lays, built in place and flat across, origin on its left edge:
    # `pieces` grows it across the gap.
    with G.new_parts() as patch:
        for i, j in sorted(PATCH):
            _piece(F, i, j)
    x0 = PX0 + 4 * PW - PW / 2
    patch = G.join(patch, 'pieces', origin=F.at(x0, PY, PZ0 - 2 * PD))
    patch.scale[0] = 0.001
    objs = G.settle(centre=False)
    G.grow(patch, 'pieces', seconds=0.9, axis='X')
    return objs


# --- the bonsai potting bench (1x1): a teak bench against the wall, a bonsai in a glazed pot on it -------------
TWIG = (0.19, 1.16, -0.2)   # where the leggy twig leaves the top-right pad


def _pad(F, x, y, z, w, d, seed):
    """A cloud of foliage: a few flattened spheres around (x, y, z)."""
    rnd = random.Random(seed)
    F.sphere(_name('pad'), 0.5, (x, y, z), M.leaf, scale=(w, d, 0.07), segments=12, rings=6)
    for _ in range(3):
        F.sphere(_name('pad'), 0.5, (x + rnd.uniform(-w, w) * 0.3, y + 0.02, z + rnd.uniform(-d, d) * 0.25), M.leaf,
                 scale=(w * 0.55, d * 0.55, 0.06), segments=10, rings=6)


def bonsaiBench():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('bench_top', 0.86, 0.04, 0.44, (0, 0.8, WALL + 0.22), M.teak, bevel=0.01)
        F.box('bench_shelf', 0.8, 0.03, 0.38, (0, 0.26, WALL + 0.22), M.teak, bevel=0.008)
        F.box('bench_back', 0.86, 0.26, 0.03, (0, 0.95, WALL + 0.015), M.body, bevel=0.008)
        F.box('bench_back_rail', 0.86, 0.03, 0.04, (0, 1.095, WALL + 0.02), M.teak, bevel=0.008)
        for x in (-0.405, 0.405):
            for z in (WALL + 0.025, WALL + 0.415):
                F.box(_name('leg'), 0.045, 0.78, 0.045, (x, 0.39, z), M.body, bevel=0.008)
            F.box(_name('side_rail'), 0.03, 0.05, 0.36, (x, 0.7, WALL + 0.22), M.body, bevel=0.006)
        F.box('front_rail', 0.78, 0.05, 0.03, (0, 0.7, WALL + 0.415), M.body, bevel=0.006)
        # the bonsai: a glazed pot, soil, a twisting trunk with two branches, pads of leaves
        F.box('pot', 0.38, 0.08, 0.26, (0, 0.86, -0.18), M.glaze, bevel=0.02, bevel_segments=2)
        for x in (-0.14, 0.14):
            F.box(_name('pot_foot'), 0.05, 0.012, 0.2, (x, 0.826, -0.18), M.glaze)
        F.box('soil', 0.34, 0.01, 0.22, (0, 0.9, -0.18), M.soil)
        F.sphere('moss', 0.5, (0.1, 0.905, -0.15), M.leaf, scale=(0.12, 0.1, 0.03), segments=10, rings=5)
        trunk = [(-0.02, 0.9, -0.18), (0.05, 0.99, -0.19), (-0.03, 1.08, -0.17), (0.02, 1.17, -0.18), (0.09, 1.22, -0.2)]
        for a, b, r in zip(trunk, trunk[1:], (0.03, 0.025, 0.022, 0.017)):
            G.rod(_name('trunk'), F.at(*a), F.at(*b), r, material=M.trunk, verts=10)
            F.sphere(_name('knuckle'), r, b, M.trunk, segments=10, rings=6)
        F.cone('root_flare', 0.05, 0.03, 0.04, (-0.02, 0.92, -0.18), M.trunk, verts=10)
        G.rod('branch_l', F.at(0.05, 0.99, -0.19), F.at(-0.15, 1.05, -0.15), 0.013, material=M.trunk, verts=8)
        G.rod('branch_r', F.at(-0.03, 1.08, -0.17), F.at(0.17, 1.12, -0.2), 0.011, material=M.trunk, verts=8)
        _pad(F, -0.17, 1.08, -0.15, 0.2, 0.16, 1)
        _pad(F, 0.18, 1.14, -0.2, 0.18, 0.15, 2)
        _pad(F, 0.08, 1.26, -0.2, 0.22, 0.18, 3)
        _pad(F, -0.04, 1.19, -0.16, 0.14, 0.12, 4)
        # tools: copper wire, a mister, shears
        G.torus('wire', 0.04, 0.012, at=F.at(0.3, 0.835, -0.05), material=M.copper, major_segments=16, minor_segments=6)
        G.revolve('mister', [(0, 0), (0.04, 0), (0.04, 0.12), (0.016, 0.15), (0.016, 0.19), (0, 0.19)],
                  at=F.at(-0.32, 0.82, -0.06), material=M.porcelain, segments=14)
        F.box('mister_nozzle', 0.012, 0.012, 0.03, (-0.32, 1.0, -0.04), M.dark)
        F.box('shear_a', 0.012, 0.008, 0.12, (0.26, 0.824, -0.33), M.dark, rot=(0, 0, 25))
        F.box('shear_b', 0.012, 0.008, 0.12, (0.29, 0.824, -0.33), M.dark, rot=(0, 0, -10))
        G.torus('shear_loop', 0.018, 0.005, at=F.at(0.265, 0.826, -0.25), material=M.terra, major_segments=10,
                minor_segments=4)
        # spare terracotta pots on the shelf, a sack of soil on the floor
        for x, h in ((-0.25, 0.14), (0.0, 0.12), (0.24, 0.16)):
            r = h * 0.55
            G.revolve(_name('spare_pot'), [(0, 0), (r * 0.72, 0), (r, h), (r * 0.88, h), (r * 0.62, 0.012), (0, 0.012)],
                      at=F.at(x, 0.275, -0.2), material=M.terra, segments=16)
        F.box('sack', 0.24, 0.3, 0.14, (0.26, 0.15, 0.25), M.sack, bevel=0.035, bevel_segments=2)
        F.box('sack_fold', 0.24, 0.04, 0.12, (0.26, 0.31, 0.25), M.sack, rot=(10, 0, 0), bevel=0.012)
    G.join(still, 'body', origin=(0, 0, 0))
    # The leggy twig the gardener trims: a shoot and a tuft, origin where it leaves the pad: `trim` shrinks
    # it away.
    with G.new_parts() as twig:
        G.rod('twig', F.at(*TWIG), F.at(0.31, 1.25, -0.17), 0.006, material=M.leaf, verts=6)
        F.sphere('twig_tuft', 0.5, (0.32, 1.26, -0.17), M.leaf, scale=(0.08, 0.07, 0.045), segments=10, rings=6)
        F.sphere('twig_tuft2', 0.5, (0.26, 1.215, -0.18), M.leaf, scale=(0.05, 0.045, 0.03), segments=8, rings=5)
    twig = G.hinged(G.join(twig, 'twig'), F.at(*TWIG))
    objs = G.settle(centre=False)
    G.squash(twig, 'trim', 0.001, seconds=0.7)
    return objs


# --- the cedar sauna for one (1x2): seated in the middle on the bench, a glass door at the front ----------------
FRONT = 0.42    # the door's depth


def soloSauna():
    F = G.GameFrame()
    zb, zf = WALL2, FRONT + 0.01       # the cabin's back and front
    zc, d = (zb + zf) / 2, zf - zb
    with G.new_parts() as still:
        F.box('floor', 0.9, 0.06, d, (0, 0.03, zc), M.cedar, bevel=0.01)
        F.box('back_wall', 0.86, 1.96, 0.05, (0, 1.04, zb + 0.025), M.cedar, bevel=0.01)
        F.box('roof', 0.94, 0.07, d + 0.06, (0, 2.05, zc + 0.02), M.cedar, bevel=0.015)
        F.box('roof_fascia', 0.94, 0.05, 0.02, (0, 2.0, zf + 0.04), M.cedar_dark)
        for s in (-1, 1):
            x = s * 0.425
            F.box(_name('side_wall'), 0.04, 1.96, d - 0.04, (x, 1.04, zc - 0.01), M.cedar, bevel=0.008)
            for y in (0.36, 0.72, 1.08, 1.44, 1.8):                 # board seams on the outside
                F.box(_name('seam'), 0.004, 0.012, d - 0.1, (x + s * 0.021, y, zc), M.cedar_dark)
            F.box(_name('post'), 0.05, 1.96, 0.06, (x, 1.04, zf - 0.02), M.cedar, bevel=0.01)
            F.box(_name('corner'), 0.05, 1.96, 0.05, (x, 1.04, zb + 0.025), M.cedar, bevel=0.01)
        F.box('lintel', 0.82, 0.08, 0.06, (0, 1.97, zf - 0.02), M.cedar, bevel=0.01)
        F.box('sill', 0.82, 0.06, 0.06, (0, 0.06, zf - 0.02), M.cedar, bevel=0.008)
        # the bench (top 0.45) and its skirt, the upper tier and backrest slats
        for z in (-0.16, -0.06, 0.04, 0.14):
            F.box(_name('slat'), 0.78, 0.035, 0.085, (0, 0.432, z), M.cedar, bevel=0.008)
        F.box('bench_skirt', 0.78, 0.38, 0.03, (0, 0.22, 0.17), M.cedar, bevel=0.006)
        for z in (-0.54, -0.44, -0.34):
            F.box(_name('tier_slat'), 0.78, 0.035, 0.085, (0, 0.82, z), M.cedar, bevel=0.008)
        F.box('tier_skirt', 0.78, 0.4, 0.03, (0, 0.62, -0.31), M.cedar, bevel=0.006)
        for y in (0.98, 1.12):
            F.box(_name('back_slat'), 0.74, 0.06, 0.025, (0, y, -0.86), M.cedar, bevel=0.008)
        # the heater with its stones, a bucket and ladle, the lamp and the thermometer
        F.box('heater', 0.26, 0.42, 0.22, (0.22, 1.07, -0.76), M.iron, bevel=0.015)
        F.box('heater_top', 0.28, 0.03, 0.24, (0.22, 1.295, -0.76), M.dark, bevel=0.008)
        for k, (dx, dz) in enumerate(((-0.07, -0.06), (0.06, -0.05), (0.0, 0.05), (-0.08, 0.06), (0.08, 0.06), (0.0, -0.01))):
            G.boulder(_name('stone'), 0.045, at=F.at(0.22 + dx, 1.305, -0.76 + dz), material=M.stones, seed=k * 1.7,
                      subdivisions=1, scale=(1, 1, 0.75))
        G.revolve('bucket', [(0, 0), (0.08, 0), (0.1, 0.16), (0.09, 0.16), (0.07, 0.015), (0, 0.015)],
                  at=F.at(-0.22, 0.837, -0.44), material=M.cedar_dark, segments=16)
        G.torus('bucket_band', 0.095, 0.006, at=F.at(-0.22, 0.92, -0.44), material=M.copper, major_segments=16,
                minor_segments=4)
        G.rod('ladle', F.at(-0.22, 0.86, -0.44), F.at(-0.16, 1.08, -0.4), 0.01, material=M.cedar_dark, verts=8)
        F.box('lamp', 0.3, 0.1, 0.06, (0, 1.8, zb + 0.08), M.lamp, bevel=0.02)
        F.box('lamp_shade', 0.34, 0.03, 0.08, (0, 1.86, zb + 0.08), M.cedar, bevel=0.006)
        F.cyl('thermometer', 0.06, 0.02, (-0.3, 1.5, zb + 0.06), M.cream, rot=(90, 0, 0), verts=18)
        F.box('towel', 0.5, 0.02, 0.3, (0, 0.46, -0.05), M.cream, bevel=0.006)
    G.join(still, 'body', origin=(0, 0, 0))
    # The glass door, hinged on its left post, its cedar pull outside: `door_open` swings it out.
    hx = -0.39
    with G.new_parts() as door:
        F.box('door_glass', 0.78, 1.84, 0.016, (0, 1.02, FRONT), M.glass)
        for y in (0.8, 1.3):
            G.rod(_name('pull_post'), F.at(0.3, y, FRONT + 0.008), F.at(0.3, y, FRONT + 0.05), 0.008,
                  material=M.cedar_dark, verts=8)
        F.box('pull', 0.035, 0.56, 0.03, (0.3, 1.05, FRONT + 0.06), M.cedar_dark, bevel=0.012)
    door = G.hinged(G.join(door, 'door'), F.at(hx, 1.0, FRONT))
    objs = G.settle(centre=False)
    G.swing(door, 'door_open', 60, seconds=0.8)
    return objs


# --- the no-knock delivery hatch (1x1): a wall cabinet with a hatch over a ledge, a parcel drawer below ---------
HZ = -0.19     # the cabinet's front face
HATCH = (0.52, 0.42, 1.18)   # the hatch's width, height and middle height


def deliveryHatch():
    F = G.GameFrame()
    hw, hh, hy = HATCH
    zc, d = (WALL + HZ) / 2, HZ - WALL
    lo, hi = hy - hh / 2, hy + hh / 2
    with G.new_parts() as still:
        # the cabinet, around the hatch's recess
        F.box('cabinet_low', 0.82, lo, d, (0, lo / 2, zc), M.slate, bevel=0.015)
        F.box('cabinet_high', 0.82, 1.9 - hi, d, (0, (1.9 + hi) / 2, zc), M.slate, bevel=0.015)
        for s in (-1, 1):
            F.box(_name('cabinet_side'), (0.82 - hw) / 2, hh + 0.02, d, (s * (0.41 + hw / 2) / 2, hy, zc), M.slate)
        F.box('recess', hw, hh, 0.02, (0, hy, WALL + 0.07), M.dark)
        F.box('recess_floor', hw, 0.01, d - 0.08, (0, lo + 0.005, zc + 0.04), M.steel)
        F.box('parcel', 0.2, 0.12, 0.14, (0.08, lo + 0.07, zc + 0.03), M.kraft, bevel=0.006)
        F.box('parcel_tape', 0.2, 0.004, 0.03, (0.08, lo + 0.132, zc + 0.03), M.mustard)
        F.box('cap', 0.84, 0.05, d + 0.03, (0, 1.925, zc + 0.015), M.wood, bevel=0.01)
        # the ledge on two brackets, a bag and a drink waiting on it
        F.box('ledge', 0.66, 0.035, 0.26, (0, 0.94, HZ + 0.13), M.wood, bevel=0.01)
        for x in (-0.28, 0.28):
            F.box(_name('bracket'), 0.04, 0.12, 0.2, (x, 0.865, HZ + 0.1), M.wood, bevel=0.01)
        F.box('bag', 0.2, 0.26, 0.14, (-0.14, 1.09, -0.06), M.kraft, bevel=0.008)
        F.box('bag_fold', 0.2, 0.04, 0.12, (-0.14, 1.235, -0.06), M.kraft, rot=(0, 4, 0), bevel=0.008)
        F.box('sticker', 0.08, 0.06, 0.004, (-0.14, 1.1, 0.011), M.terra, bevel=0.002)
        G.revolve('cup', [(0, 0), (0.032, 0), (0.045, 0.15), (0.047, 0.155), (0.047, 0.16), (0, 0.16)],
                  at=F.at(0.14, 0.958, -0.04), material=M.porcelain, segments=16)
        G.rod('straw', F.at(0.14, 1.1, -0.04), F.at(0.16, 1.2, -0.05), 0.005, material=M.needle, verts=6)
        # the order screen and the "no knock" plaque
        F.box('screen', 0.22, 0.15, 0.02, (0.24, 1.55, HZ + 0.01), M.screen, bevel=0.01)
        F.box('screen_line_a', 0.16, 0.02, 0.004, (0.24, 1.58, HZ + 0.021), M.screen_a)
        F.box('screen_line_b', 0.1, 0.02, 0.004, (0.21, 1.53, HZ + 0.021), M.dial)
        F.box('plaque', 0.26, 0.09, 0.012, (-0.18, 1.55, HZ + 0.006), M.cream, bevel=0.006)
        F.box('plaque_bar', 0.2, 0.012, 0.004, (-0.18, 1.55, HZ + 0.013), M.needle)
        # the parcel drawer and the mat
        F.box('drawer', 0.62, 0.3, 0.02, (0, 0.4, HZ + 0.01), M.steel, bevel=0.008)
        F.box('drawer_pull', 0.2, 0.02, 0.025, (0, 0.48, HZ + 0.03), M.chrome, bevel=0.008)
        F.box('drawer_slot', 0.3, 0.012, 0.004, (0, 0.33, HZ + 0.021), M.dark)
        F.box('mat', 0.62, 0.012, 0.42, (0, 0.006, 0.2), M.mat, bevel=0.004)
    G.join(still, 'body', origin=(0, 0, 0))
    # The hatch door, hinged on its right edge: `hatch` swings it open.
    with G.new_parts() as door:
        F.box('hatch_door', hw, hh, 0.03, (0, hy, HZ + 0.015), M.steel, bevel=0.012)
        F.box('hatch_window', 0.3, 0.16, 0.004, (0, hy + 0.06, HZ + 0.031), M.screen, bevel=0.004)
        F.box('hatch_handle', 0.3, 0.025, 0.03, (0, hy - 0.12, HZ + 0.045), M.chrome, bevel=0.01)
    door = G.hinged(G.join(door, 'hatch'), F.at(hw / 2, hy, HZ + 0.03))
    objs = G.settle(centre=False)
    G.swing(door, 'hatch', -105, seconds=0.8)
    return objs


FAMILY = G.Family(
    'pack_loner', M,
    [G.Model('quietPod', quietPod, size=(0.84, 1.539, 0.769), key='model.loner.quietPod',
             centre=(0, -0.074),
             motions={'uses': {'retreat': 'shell', 'read': 'shell'}},
             paint=[('shell', 'Body'), ('oak', 'Wood'), ('cushion', 'Cushions')]),
     G.Model('shortwaveRadio', shortwaveRadio, size=(0.84, 1.325, 0.682), key='model.loner.shortwaveRadio',
             centre=(0, 0.113),
             motions={'uses': {'tune': 'dial', 'airwaves': ['antenna', 'dial']}},
             paint=[('body', 'Body'), ('oak', 'Wood'), ('seatpad', 'Seat')]),
     G.Model('puzzleTable', puzzleTable, size=(0.867, 1.008, 0.718), key='model.loner.puzzleTable',
             centre=(0.003, 0.091),
             motions={'uses': {'piece': 'pieces', 'newPuzzle': 'pieces'}},
             paint=[('body', 'Body'), ('wood', 'Wood')]),
     G.Model('bonsaiBench', bonsaiBench, size=(0.86, 1.31, 0.75), key='model.loner.bonsaiBench',
             centre=(0, -0.055),
             motions={'uses': {'shape': 'trim'}},
             paint=[('body', 'Body'), ('teak', 'Wood'), ('glaze', 'Pot')]),
     G.Model('soloSauna', soloSauna, size=(0.94, 2.085, 1.435), key='model.loner.soloSauna',
             centre=(0, -0.222),
             motions={'uses': {'sweat': {'clip': 'door_open', 'hold': 1.5}}},
             paint=[('cedar', 'Wood'), ('cedar_dark', 'Trim')]),
     G.Model('deliveryHatch', deliveryHatch, size=(0.84, 1.95, 0.84), key='model.loner.deliveryHatch',
             centre=(0, -0.01),
             motions={'uses': {'order': 'hatch', 'treat': 'hatch'}, 'hold': 2},
             paint=[('slate', 'Body'), ('wood', 'Wood')])],
    comment='Blender-built loner pack models (tools/art/blender/families/pack_loner.py). Replaces the '
            'primitive-part placeholders of packs/loner by key; the placeholder is kept as a fallback if a file '
            'fails to load, and the materials map onto the game\'s finishes so a model costs one draw call per '
            'finish group. Each moves a part while it is used: the pod\'s hood, the radio\'s dial and antenna, '
            'a patch of the puzzle, a twig of the bonsai, the sauna\'s door and the delivery hatch.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

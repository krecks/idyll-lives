# The loner pack (P11): the hush-shell reading pod, the shortwave radio desk, the jigsaw puzzle table, the
# bonsai potting bench, the cedar sauna for one and the no-knock delivery hatch.
# Notes: pack_loner.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`): the pod's hood swings half closed (`shell`), the radio's tuning dial turns (`dial`) and its
# antenna extends (`antenna`), a patch of the puzzle grows (`pieces`), a twig of the bonsai shrinks away
# (`trim`), the sauna's glass door opens and shuts (`door_open`) and the delivery hatch opens (`hatch`).
#
# Built with the forms and the kit (2026-10-22): an upholstered shell of soft panels under a hood of padded
# channels, spindle chairs swept in one piece, jigsaw pieces that nest, a bonsai's tapering trunk under jittered
# foliage pads, a cedar cabin of boards, the hatch's moulded frame.
#
# Run:  cargo art pack_loner [--only quietPod] [--preview]
import math
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


def _turned(F, name, x, y, z, profile, material, segments=16, rot=(0, 0, 0)):
    """A turned piece standing at game (x, y, z): `profile` [(radius, height up), ...] from its foot, revolved."""
    return G.revolve(name, profile, at=F.at(x, y, z), material=material, segments=segments, rot=rot)


def _lathe(F, name, x, y, z, rings, material, verts=16, square=2.0, sy=1.0):
    """A body lofted through rings [(radius, height up), ...] standing at game (x, y, z); `square` > 2 squares them."""
    bx, by, _ = F.at(x, 0, z)
    return G.loft(name, [G.ring(r, r * sy, y + h, verts, square=square, at=(bx, by)) for r, h in rings], material=material)


def _chair(F, z, wood, pad, verts=6):
    """A spindle chair facing the front, its seat (top 0.45) centred on (0, z): the resident's seat. A shaped seat with
    its pad, back posts swept up from the floor and raked back, tapered front legs, a bowed crest rail over three
    spindles, side stretchers."""
    F.extrude(_name('chair_seat'), G.smooth_outline([(-0.21, -0.2), (0.21, -0.2), (0.21, 0.17), (0.0, 0.2), (-0.21, 0.17)], 1),
              0.04, (0, 0.41, z), wood, plane='top', bevel=0.008, bevel_segments=1)
    F.soft(_name('chair_pad'), 0.36, 0.024, 0.34, (0, 0.442, z + 0.01), pad, round=0.012, puff=0.006, round_segments=1, flat=2)
    for x in (-0.18, 0.18):
        F.sweep(_name('chair_leg'), [(x, 0.39, z + 0.15), (x * 1.04, 0.0, z + 0.17)], 0.018, wood, verts=verts, scale=[1.0, 0.75])
        F.sweep(_name('chair_post'), [(x, 0.0, z - 0.165), (x, 0.42, z - 0.172), (x, 0.92, z - 0.193)], 0.018, wood, verts=verts,
                resolution=2, scale=[0.8, 1.0, 0.85])
        G.kit.tube(F, _name('stretcher'), [(x, 0.14, z + 0.15), (x, 0.14, z - 0.17)], 0.009, wood, verts=verts, round=1)
    crest = [(x, 0.87 + 0.012 * (1 - (x / 0.2) ** 2), z - 0.186 - 0.006 * (1 - (x / 0.2) ** 2)) for x in (-0.2, -0.1, 0.0, 0.1, 0.2)]
    F.sweep(_name('chair_crest'), crest, None, wood, profile=G.rounded_rect(0.09, 0.03, 0.012, corner=1), up=(0, 0, 1),
            resolution=2)
    for x in (-0.09, 0.0, 0.09):
        G.kit.tube(F, _name('spindle'), [(x, 0.43, z - 0.175), (x, 0.83, z - 0.186 - 0.006 * (1 - (x / 0.2) ** 2))], 0.009, wood,
                   verts=verts, round=1)


def _desk(F, z0, z1, w, h, top, legs, verts=8):
    """A plain desk from depth z0 to z1, `h` high: its top, an apron at the back and the ends, tapered splayed legs."""
    zc, d = (z0 + z1) / 2, z1 - z0
    F.box(_name('desk_top'), w, 0.03, d, (0, h - 0.015, zc), top, bevel=0.008, bevel_segments=2)
    F.box(_name('desk_apron'), w - 0.08, 0.07, 0.02, (0, h - 0.065, z1 - 0.03), legs, bevel=0.004, bevel_segments=1)
    for x in (-w / 2 + 0.035, w / 2 - 0.035):
        F.box(_name('desk_apron_side'), 0.02, 0.07, d - 0.08, (x, h - 0.065, zc), legs, bevel=0.004, bevel_segments=1)
        for z in (z0 + 0.035, z1 - 0.035):
            F.sweep(_name('desk_leg'), [(x, h - 0.03, z), (x + math.copysign(0.012, x), 0.0, z + math.copysign(0.01, z - zc))],
                    0.02, legs, verts=verts, scale=[1.0, 0.65])


def _mug(F, x, y, z, material, turn=0.0):
    """A mug standing on `y`: hollow, its handle out at `turn` degrees (0: +x)."""
    r, h = 0.038, 0.09
    _turned(F, _name('mug'), x, y, z, [(0, 0), (0.034, 0), (r, 0.006), (r, h), (0.032, h), (0.03, 0.012), (0, 0.012)], material, 14)
    c, s = G.cosd(turn), G.sind(turn)
    F.sweep(_name('mug_handle'), [(x + c * d, y + t * h, z + s * d) for d, t in ((r - 0.003, 0.8), (r + 0.016, 0.82),
                                                                                 (r + 0.024, 0.55), (r + 0.015, 0.27),
                                                                                 (r - 0.003, 0.25))], 0.0045, material,
            verts=5, resolution=2)


def _book(F, name, x, y, z, w, h, d, cover, turn=0.0):
    """A closed book lying flat on `y`: its cover and the block of pages, a hair inside it but for the spine (-x)."""
    F.box(f'{name}_cover', w, h, d, (x, y + h / 2, z), cover, rot=(0, 0, turn), bevel=0.002, bevel_segments=1)
    c, s = G.cosd(turn), G.sind(turn)
    F.box(f'{name}_pages', w - 0.008, h - 0.006, d + 0.002, (x + 0.004 * c, y + h / 2, z + 0.004 * s), M.cream, rot=(0, 0, turn))


# --- the hush-shell reading pod (1x1): an upholstered shell chair on a swivel foot, a pram hood over it -------
HOOD = (1.1, -0.02)   # the hood's pivot (height, depth): it turns about a line across the pod's wings


def quietPod():
    F = G.GameFrame()
    with G.new_parts() as still:
        _turned(F, 'foot', 0, 0, -0.04, [(0, 0), (0.24, 0), (0.25, 0.012), (0.24, 0.035), (0.2, 0.04), (0, 0.04)], M.oak, 28)
        _turned(F, 'stem', 0, 0.04, -0.04, [(0, 0), (0.06, 0), (0.045, 0.03), (0.04, 0.13), (0.06, 0.16), (0, 0.16)], M.chrome, 18)
        # the upholstered shell: a bucket under the seat, a tall back up into the hood, wings with rolled front edges,
        # all piped where the panels meet
        F.soft('bucket', 0.82, 0.24, 0.74, (0, 0.32, -0.06), M.shell, round=0.1, piping=0.006, flat=1)
        F.soft('back', 0.82, 1.04, 0.14, (0, 0.88, WALL + 0.07), M.shell, round=0.06, flat=1)
        for s in (-1, 1):
            F.soft(_name('wing'), 0.11, 0.86, 0.6, (s * 0.355, 0.78, -0.13), M.shell, round=0.05, puff=0.01, flat=2)
            F.sweep(_name('wing_edge'), [(s * 0.355, 0.35, 0.17), (s * 0.355, 0.78, 0.175), (s * 0.355, 1.21, 0.17)], 0.055,
                    M.shell, verts=14, resolution=2)
            F.sweep(_name('wing_piping'), [(s * 0.3, 0.36, 0.17), (s * 0.3, 1.2, 0.17)], 0.008, M.shell, verts=5)
            F.soft(_name('wing_lining'), 0.02, 0.6, 0.48, (s * 0.292, 0.8, -0.12), M.cushion, round=0.008, round_segments=1,
                   flat=1)
            F.cyl(_name('hood_boss'), 0.05, 0.03, (s * 0.4, HOOD[0], HOOD[1]), M.oak, rot=(0, 90, 0), verts=16, bevel=0.006,
                  bevel_segments=1)
        # the cushions: the seat (top 0.45), a buttoned back cushion, a head roll
        puff = 0.012
        F.soft('seat', 0.6, 0.1 - puff, 0.56, (0, 0.4 - puff / 2, 0.02), M.cushion, round=0.045, puff=puff, piping=0.005, flat=3)
        F.soft('back_cushion', 0.58, 0.56, 0.1, (0, 0.76, -0.29), M.cushion, rot=(-6, 0, 0), round=0.04, puff=0.025, flat=3)
        for u, v in ((-0.14, 0.12), (0.14, 0.12), (0.0, 0.0), (-0.14, -0.12), (0.14, -0.12)):
            out = 0.05 + 0.025 * (1 - (u / 0.29) ** 2) * (1 - (v / 0.28) ** 2) - 0.008
            F.sphere(_name('button'), 0.011, (u, 0.76 + v * G.cosd(6), -0.29 - v * G.sind(6) + out), M.headrest, scale=(1, 0.5, 1),
                     segments=6, rings=4)
        F.sweep('head_cushion', [(-0.2, 1.08, -0.31), (0.2, 1.08, -0.31)], 0.055, M.headrest, verts=12, scale=[0.9, 0.9])
        for x in (-0.2, 0.2):
            F.sphere(_name('head_cushion_end'), 0.05, (x, 1.08, -0.31), M.headrest, scale=(0.4, 1, 1), segments=12, rings=6)
        # a book and a folded blanket on the seat, a gooseneck reading lamp on the right wing
        _book(F, 'book', 0.18, 0.45, 0.16, 0.2, 0.03, 0.15, M.mustard, turn=15)
        F.soft('blanket', 0.36, 0.03, 0.26, (-0.1, 0.465, 0.13), M.headrest, rot=(0, 0, -8), round=0.012, puff=0.004, flat=2)
        F.sweep('blanket_fold', [(-0.27, 0.468, 0.255), (0.07, 0.468, 0.21)], 0.016, M.headrest, verts=8)
        G.kit.tube(F, 'lamp_arm', [(0.31, 1.0, -0.12), (0.31, 1.07, -0.08), (0.29, 1.08, -0.02), (0.27, 1.05, 0.01)], 0.008,
                   M.chrome, verts=6, round=2)
        F.cyl('lamp_clamp', 0.016, 0.03, (0.31, 1.0, -0.12), M.chrome, verts=10)
        _turned(F, 'lamp', 0.27, 0.99, 0.02, [(0.04, 0), (0.038, 0.015), (0.028, 0.045), (0.012, 0.06), (0, 0.062)], M.lamp, 14)
    G.join(still, 'body', origin=(0, 0, 0))
    # The pram hood: an arc of padded channels over the back, about a line across the wings, a bound lip at its
    # front. `shell` swings it forward over the reader's head, half closing the pod's mouth.
    with G.new_parts() as hood:
        r, n = 0.4, 6
        for k in range(n):
            a = -90 + (k + 0.5) * 90 / n                    # from behind (-90) to straight up (0)
            y = HOOD[0] + r * G.cosd(a)
            z = HOOD[1] + r * G.sind(a)
            F.soft(_name('rib'), 0.84, 0.075, r * 3.1416 / 2 / n + 0.03, (0, y, z), M.shell, rot=(a, 0, 0), round=0.03,
                   puff=0.012, round_segments=2, flat=2)
        F.sweep('hood_lip', [(-0.42, HOOD[0] + r, HOOD[1] + 0.01), (0.42, HOOD[0] + r, HOOD[1] + 0.01)], 0.025, M.shell, verts=10)
    hood = G.hinged(G.join(hood, 'hood'), F.at(0, *HOOD))
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.swing(hood, 'shell', 40, seconds=0.9, axis='X')
    return objs


# --- the shortwave radio desk (1x1): seated in the middle, the rig on a shallow desk before them --------------
DIAL = (0.15, 0.87, 0.243)    # the tuning knob, on the receiver's face towards the operator
ANTENNA = (0.2, 0.42)         # the telescopic antenna's (x, depth), at the receiver's back corner


def shortwaveRadio():
    F = G.GameFrame()
    with G.new_parts() as still:
        _desk(F, 0.16, 0.45, 0.84, 0.74, M.oak, M.oak, verts=6)
        _chair(F, -0.02, M.oak, M.seatpad)
        # the receiver: its face (a framed dial window with its scale, a meter, knobs, a speaker grille) towards the
        # operator, vents to the room, a carry handle on top
        F.box('receiver', 0.5, 0.24, 0.2, (0.02, 0.86, 0.35), M.body, bevel=0.02, bevel_segments=2)
        F.box('face', 0.46, 0.2, 0.01, (0.02, 0.86, 0.248), M.rig_face, bevel=0.004, bevel_segments=1)
        F.box('dial_window', 0.22, 0.07, 0.006, (-0.07, 0.92, 0.243), M.dial, bevel=0.003, bevel_segments=1)
        G.kit.moulding(F, 'dial_bezel', [(0.04, 0.88), (0.04, 0.96), (-0.18, 0.96), (-0.18, 0.88)],
                       [(-0.008, 0.0), (-0.008, 0.004), (0.0, 0.006), (0.004, 0.0)], M.chrome, at=0.243, plane='front')
        for k in range(9):
            F.box(_name('scale_mark'), 0.002, 0.014 if k % 2 else 0.022, 0.002, (-0.16 + 0.02 * k, 0.935, 0.2395), M.dark)
        F.box('dial_needle', 0.004, 0.06, 0.004, (-0.03, 0.92, 0.239), M.needle)
        F.box('meter', 0.07, 0.05, 0.006, (0.15, 0.95, 0.243), M.cream, bevel=0.003, bevel_segments=1)
        F.box('meter_needle', 0.002, 0.035, 0.002, (0.155, 0.945, 0.2395), M.needle, rot=(0, -25, 0))
        for x in (-0.15, -0.09, -0.03, 0.03):
            _turned(F, _name('knob'), x, 0.8, 0.243, [(0, 0), (0.016, 0), (0.014, 0.014), (0.01, 0.02), (0, 0.021)], M.dark, 10,
                    rot=(-90, 0, 0))
        F.extrude('grille', G.rounded_rect(0.06, 0.07, 0.008, corner=2), 0.006, (-0.2, 0.86, 0.243), M.dark,
                  holes=[[(u, v + 0.022 * k - 0.022) for u, v in G.rounded_rect(0.044, 0.008, 0.004, corner=1)] for k in range(3)])
        for k in range(6):                                        # vents on its back, which faces the room
            F.box(_name('vent'), 0.3, 0.008, 0.004, (0.0, 0.8 + 0.024 * k, 0.452), M.dark)
        F.box('foot_l', 0.04, 0.012, 0.18, (-0.19, 0.746, 0.35), M.dark)
        F.box('foot_r', 0.04, 0.012, 0.18, (0.23, 0.746, 0.35), M.dark)
        G.kit.tube(F, 'handle', G.kit.fillet([(-0.12, 0.975, 0.35), (-0.12, 1.02, 0.35), (0.16, 1.02, 0.35), (0.16, 0.975, 0.35)], 0.02),
                   0.009, M.chrome, verts=6, round=1)
        # the antenna's swivel and outer tube
        F.sphere('antenna_swivel', 0.018, (ANTENNA[0], 0.985, ANTENNA[1]), M.chrome, segments=8, rings=5)
        G.kit.tube(F, 'antenna_tube', [(ANTENNA[0], 0.99, ANTENNA[1]), (ANTENNA[0], 1.24, ANTENNA[1])], 0.008, M.chrome, verts=6,
                   round=1)
        # the desk microphone, headphones with their cord, the logbook and pencil, a mug
        _turned(F, 'mic_base', -0.3, 0.74, 0.28, [(0, 0), (0.05, 0), (0.048, 0.012), (0.02, 0.02), (0, 0.02)], M.dark, 14)
        G.kit.tube(F, 'mic_stem', [(-0.3, 0.76, 0.28), (-0.29, 0.86, 0.275), (-0.28, 0.9, 0.27)], 0.007, M.chrome, verts=6, round=2)
        _lathe(F, 'mic_capsule', -0.275, 0.9, 0.265, [(0.012, 0.0), (0.024, 0.012), (0.026, 0.05), (0.022, 0.085), (0.008, 0.095)],
               M.steel, verts=12)
        G.torus('mic_band', 0.026, 0.004, at=F.at(-0.275, 0.94, 0.265), material=M.dark, major_segments=12, minor_segments=3)
        band = [(0.3, 0.77 + 0.075 * math.sin(math.pi * t), 0.25 + 0.085 * math.cos(math.pi * t)) for t in (0, 0.2, 0.4, 0.6, 0.8, 1)]
        F.sweep('headphone_band', band, None, M.dark, profile=G.rounded_rect(0.014, 0.022, 0.006, corner=1), up=(1, 0, 0),
                resolution=2)
        for zz in (0.17, 0.33):
            _turned(F, _name('earcup'), 0.3, 0.765, zz, [(0, 0), (0.036, 0), (0.038, 0.018), (0.03, 0.036), (0, 0.036)], M.seatpad, 12,
                    rot=(90 if zz > 0.25 else -90, 0, 0))
        G.kit.tube(F, 'headphone_cord', [(0.3, 0.77, 0.17), (0.25, 0.745, 0.19), (0.2, 0.745, 0.24), (0.14, 0.75, 0.3)], 0.003,
                   M.dark, verts=4, round=2)
        _book(F, 'logbook', -0.15, 0.74, 0.21, 0.16, 0.012, 0.2, M.cream, turn=10)
        F.sweep('pencil', [(-0.13, 0.756, 0.15), (-0.07, 0.756, 0.27)], 0.004, M.mustard, verts=6, scale=[0.3, 1.0])
        _mug(F, 0.33, 0.74, 0.4, M.terra, turn=200)
    G.join(still, 'body', origin=(0, 0, 0))
    # The tuning knob: a big ridged knob with a skirt and a pointer, turned by `dial`.
    with G.new_parts() as dial:
        _turned(F, 'dial_knob', DIAL[0], DIAL[1], DIAL[2], [(0, 0), (0.045, 0), (0.045, 0.006), (0.036, 0.01), (0.036, 0.026),
                                                            (0.032, 0.03), (0, 0.03)], M.dark, 20, rot=(-90, 0, 0))
        F.box('dial_pointer', 0.008, 0.03, 0.006, (DIAL[0], DIAL[1] + 0.02, DIAL[2] - 0.029), M.cream)
    dial = G.hinged(G.join(dial, 'dial'), F.at(*DIAL))
    # The antenna's inner section, down in the tube (and the case) at rest: `antenna` runs it up 45 cm.
    with G.new_parts() as ant:
        G.kit.tube(F, 'antenna_mid', [(ANTENNA[0], 0.77, ANTENNA[1]), (ANTENNA[0], 1.31, ANTENNA[1])], 0.0055, M.chrome, verts=6,
                   round=1)
        F.sphere('antenna_ball', 0.01, (ANTENNA[0], 1.315, ANTENNA[1]), M.chrome, segments=8, rings=5)
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


def _jigsaw(w, d, r=0.011):
    """A jigsaw piece's outline `w` x `d` (centred, game x and depth): a knob out of its right edge, the same shape
    taken out of its left, so a row of pieces nests."""
    hw, hd = w / 2, d / 2
    a0 = math.degrees(math.acos(-0.7))
    knob = [(hw + 0.7 * r + r * G.cosd(a), r * G.sind(a)) for a in (-a0 + 2 * a0 * k / 8 for k in range(9))]
    sock = [(-hw + 0.7 * r + r * G.cosd(a), r * G.sind(a)) for a in (a0 - 2 * a0 * k / 8 for k in range(9))]
    return [(-hw, -hd), (hw, -hd)] + knob + [(hw, hd), (-hw, hd)] + sock


def _piece(F, i, j, y=PY, name='piece', turn=None, at=None, colour=None, size=(PW - 0.002, PD - 0.002, 0.011)):
    m = colour or _m(BANDS[j][(i + j) % 2])
    x, z = at or (PX0 + i * PW, PZ0 - j * PD)
    o = F.extrude(_name(name), _jigsaw(*size), 0.008, (x, y + 0.0004 * ((i + j) % 2), z), m, plane='top')
    if turn is not None:
        o.rotation_euler[2] = math.radians(turn)
    return o


def puzzleTable():
    F = G.GameFrame()
    with G.new_parts() as still:
        _desk(F, 0.1, 0.45, 0.86, 0.72, M.wood, M.body)
        _chair(F, -0.06, M.wood, M.body)
        F.box('puzzle_mat', 0.68, 0.004, 0.32, (0.0, 0.722, 0.28), M.dark, bevel=0.0015, bevel_segments=1)
        for i in range(COLS):
            for j in range(ROWS):
                if (i, j) not in PATCH and (i, j) not in {(0, 2), (2, 3)}:
                    _piece(F, i, j)
        for k, (x, z, r, m) in enumerate([(0.38, 0.19, 20, 'sky_a'), (0.34, 0.14, -35, 'hill_b'), (0.4, 0.12, 60, 'field_a'),
                                          (-0.37, 0.16, 10, 'sea_a'), (-0.33, 0.13, -50, 'field_b')]):
            _piece(F, k, 0, y=0.726, name='loose', turn=r, at=(x, z), colour=_m(m), size=(0.056, 0.046, 0.008))
        # the box on the floor, its lid off beside it showing the picture
        F.box('puzzle_box', 0.3, 0.04, 0.22, (0.05, 0.02, 0.32), M.glaze, bevel=0.003, bevel_segments=1)
        F.box('box_inside', 0.284, 0.004, 0.204, (0.05, 0.039, 0.32), M.dark)
        F.box('box_picture', 0.26, 0.004, 0.18, (0.05, 0.0425, 0.32), M.field_b)
        F.box('box_sky', 0.26, 0.005, 0.07, (0.05, 0.0425, 0.375), M.sky_a)
        F.box('box_sea', 0.26, 0.005, 0.04, (0.05, 0.0425, 0.315), M.sea_a)
        F.cyl('box_sun', 0.02, 0.006, (0.12, 0.043, 0.37), M.dial, verts=10)
        # a magnifier lamp clamped at the back-left corner: its clamp, two jointed arms, the ring round its lens
        F.box('lamp_clamp', 0.05, 0.05, 0.04, (-0.36, 0.71, 0.43), M.dark, bevel=0.006, bevel_segments=1)
        F.cyl('lamp_foot', 0.02, 0.03, (-0.36, 0.735, 0.4), M.dark, verts=10)
        G.kit.tube(F, 'lamp_arm_lo', [(-0.36, 0.745, 0.4), (-0.32, 1.0, 0.38)], 0.008, M.chrome, verts=6, round=1)
        G.kit.tube(F, 'lamp_arm_hi', [(-0.32, 1.0, 0.38), (-0.2, 0.97, 0.31)], 0.007, M.chrome, verts=6, round=1)
        F.cyl('lamp_joint', 0.013, 0.026, (-0.32, 1.0, 0.38), M.dark, rot=(0, 90, 0), verts=8)
        G.torus('lamp_ring', 0.07, 0.014, at=F.at(-0.17, 0.95, 0.29), material=M.steel, rot=(-30, 0, 0),
                major_segments=24, minor_segments=6)
        F.cyl('lamp_lens', 0.062, 0.006, (-0.17, 0.95, 0.29), M.lens, rot=(-30, 0, 0), verts=24)
        _mug(F, 0.36, 0.72, 0.38, M.porcelain, turn=160)
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
    """A cloud of foliage: a few jittered, flattened clumps around (x, y, z), the top one the fullest."""
    rnd = random.Random(seed)
    o = G.boulder(_name('pad'), 0.5, at=F.at(x, y - 0.03, z), material=M.leaf, scale=(w, d, 0.07), seed=seed * 1.3, noise=0.25,
                  subdivisions=2)
    G.jitter(o, 0.006, size=0.04, seed=seed)
    for _ in range(3):
        o = G.boulder(_name('pad'), 0.5, at=F.at(x + rnd.uniform(-w, w) * 0.3, y - 0.01, z + rnd.uniform(-d, d) * 0.25), material=M.leaf,
                      scale=(w * 0.55, d * 0.55, 0.06), seed=seed * 2.1 + _, noise=0.3, subdivisions=1)
        G.jitter(o, 0.005, size=0.03, seed=seed + _)


def bonsaiBench():
    F = G.GameFrame()
    with G.new_parts() as still:
        # the bench: a slatted teak top and shelf on a sage frame, a back board with a rail
        for k in range(5):
            F.box(_name('bench_slat'), 0.86, 0.04, 0.082, (0, 0.8, WALL + 0.045 + 0.0875 * k), M.teak, bevel=0.006, bevel_segments=1)
        for k in range(4):
            F.box(_name('shelf_slat'), 0.8, 0.03, 0.088, (0, 0.26, WALL + 0.055 + 0.0967 * k), M.teak, bevel=0.005, bevel_segments=1)
        F.box('bench_back', 0.86, 0.26, 0.03, (0, 0.95, WALL + 0.015), M.body, bevel=0.008, bevel_segments=1)
        F.sweep('bench_back_rail', [(-0.43, 1.095, WALL + 0.02), (0.43, 1.095, WALL + 0.02)], None, M.teak,
                profile=G.rounded_rect(0.03, 0.04, 0.012, corner=2), up=(0, 0, 1))
        for x in (-0.405, 0.405):
            for z in (WALL + 0.025, WALL + 0.415):
                F.box(_name('leg'), 0.045, 0.78, 0.045, (x, 0.39, z), M.body, bevel=0.008, bevel_segments=1)
            F.box(_name('side_rail'), 0.03, 0.05, 0.36, (x, 0.7, WALL + 0.22), M.body, bevel=0.006, bevel_segments=1)
            F.box(_name('shelf_rail'), 0.03, 0.04, 0.36, (x, 0.23, WALL + 0.22), M.body, bevel=0.006, bevel_segments=1)
        F.box('front_rail', 0.78, 0.05, 0.03, (0, 0.7, WALL + 0.415), M.body, bevel=0.006, bevel_segments=1)
        # the bonsai: a glazed tray pot on feet, soil and moss, roots gripping it, a twisting tapered trunk with its
        # branches, pads of leaves
        _lathe(F, 'pot', 0, 0.82, -0.18, [(0.17, 0.0), (0.19, 0.012), (0.19, 0.07), (0.2, 0.08), (0.18, 0.08), (0.175, 0.07)],
               M.glaze, verts=24, square=7, sy=0.68)
        for x in (-0.13, 0.13):
            for z in (-0.25, -0.11):
                F.box(_name('pot_foot'), 0.03, 0.012, 0.03, (x, 0.826, z), M.glaze, bevel=0.004, bevel_segments=1)
        F.box('soil', 0.34, 0.01, 0.22, (0, 0.89, -0.18), M.soil)
        G.boulder('moss', 0.5, at=F.at(0.1, 0.892, -0.15), material=M.leaf, scale=(0.12, 0.1, 0.025), seed=4.0, noise=0.3,
                  subdivisions=1)
        for k, (dx, dz) in enumerate(((-0.07, 0.02), (0.05, -0.03), (-0.02, -0.05), (0.04, 0.04))):
            F.sweep(_name('root'), [(-0.02, 0.93, -0.18), (-0.02 + dx * 0.6, 0.905, -0.18 + dz * 0.6), (-0.02 + dx, 0.895, -0.18 + dz)],
                    0.012, M.trunk, verts=6, scale=[1.0, 0.6], resolution=2)
        trunk = [(-0.02, 0.9, -0.18), (0.05, 0.99, -0.19), (-0.03, 1.08, -0.17), (0.02, 1.17, -0.18), (0.09, 1.22, -0.2)]
        t = F.sweep('trunk', trunk, 0.034, M.trunk, verts=10, resolution=3, scale=[1.0, 0.75, 0.62, 0.5, 0.3])
        G.jitter(t, 0.003, size=0.03, seed=2.0)
        F.sweep('branch_l', [(0.05, 0.99, -0.19), (-0.05, 1.04, -0.16), (-0.15, 1.05, -0.15)], 0.013, M.trunk, verts=6,
                resolution=2, scale=[1.0, 0.4])
        F.sweep('branch_r', [(-0.03, 1.08, -0.17), (0.07, 1.11, -0.19), (0.17, 1.12, -0.2)], 0.011, M.trunk, verts=6,
                resolution=2, scale=[1.0, 0.4])
        _pad(F, -0.17, 1.08, -0.15, 0.2, 0.16, 1)
        _pad(F, 0.18, 1.14, -0.2, 0.18, 0.15, 2)
        _pad(F, 0.08, 1.26, -0.2, 0.22, 0.18, 3)
        _pad(F, -0.04, 1.19, -0.16, 0.14, 0.12, 4)
        # tools: a coil of copper wire, a mister, a pair of shears
        coil = [(0.3 + 0.035 * math.cos(math.tau * k / 12), 0.825 + 0.002 * k, -0.05 + 0.035 * math.sin(math.tau * k / 12))
                for k in range(37)]
        F.sweep('wire', coil, 0.004, M.copper, verts=4)
        _lathe(F, 'mister', -0.32, 0.82, -0.06, [(0.036, 0.0), (0.04, 0.008), (0.04, 0.11), (0.03, 0.125), (0.016, 0.135),
                                                 (0.016, 0.165), (0.0, 0.165)], M.porcelain, verts=14)
        F.box('mister_head', 0.04, 0.03, 0.05, (-0.32, 1.0, -0.055), M.dark, bevel=0.006, bevel_segments=1)
        F.box('mister_nozzle', 0.012, 0.012, 0.03, (-0.32, 1.005, -0.025), M.dark)
        F.sweep('mister_trigger', [(-0.32, 0.995, -0.035), (-0.32, 0.96, -0.03), (-0.32, 0.94, -0.04)], None, M.dark,
                profile=G.rounded_rect(0.008, 0.016, 0.003, corner=1), up=(1, 0, 0), resolution=2)
        for k, a in enumerate((25, -10)):
            blade = [(0.0, -0.06), (0.006, -0.02), (0.005, 0.05), (0.0, 0.06), (-0.004, 0.05), (-0.004, -0.02)]
            F.extrude(_name('shear_blade'), blade, 0.004, (0.27 + 0.02 * k, 0.822 + 0.004 * k, -0.34), M.steel, plane='top',
                      rot=(0, 0, a))
            G.torus(_name('shear_loop'), 0.016, 0.005, at=F.at(0.25 + 0.04 * k, 0.826, -0.27), material=M.terra, major_segments=10,
                    minor_segments=4)
        # spare terracotta pots on the shelf, a sack of soil on the floor with its top rolled
        for x, h in ((-0.25, 0.14), (0.0, 0.12), (0.24, 0.16)):
            r = h * 0.55
            _turned(F, _name('spare_pot'), x, 0.275, -0.2, [(0, 0), (r * 0.72, 0), (r * 0.9, h * 0.8), (r, h * 0.8), (r, h),
                                                            (r * 0.88, h), (r * 0.62, 0.012), (0, 0.012)], M.terra, 16)
        F.soft('sack', 0.24, 0.27, 0.12, (0.26, 0.135, 0.25), M.sack, round=0.04, puff=0.01, flat=2)
        G.torus('sack_roll', 0.11, 0.022, at=F.at(0.26, 0.285, 0.25), material=M.sack, major_segments=16,
                minor_segments=5).scale = (1.05, 0.5, 1.0)
        F.sphere('sack_soil', 0.09, (0.26, 0.28, 0.25), M.soil, scale=(1.0, 0.55, 0.2), segments=10, rings=5)
    G.join(still, 'body', origin=(0, 0, 0))
    # The leggy twig the gardener trims: a shoot and a tuft, origin where it leaves the pad: `trim` shrinks
    # it away.
    with G.new_parts() as twig:
        F.sweep('twig', [TWIG, (0.25, 1.21, -0.185), (0.31, 1.25, -0.17)], 0.006, M.leaf, verts=6, resolution=2, scale=[1.0, 0.6])
        for k, (x, y, z, s) in enumerate(((0.32, 1.26, -0.17, 0.08), (0.26, 1.215, -0.18, 0.05))):
            o = G.boulder(_name('twig_tuft'), 0.5, at=F.at(x, y - s * 0.4, z), material=M.leaf, scale=(s, s * 0.85, s * 0.6),
                          seed=7.0 + k, noise=0.3, subdivisions=1)
            G.jitter(o, 0.004, size=0.03, seed=k)
    twig = G.hinged(G.join(twig, 'twig'), F.at(*TWIG))
    objs = G.settle(centre=False)
    G.squash(twig, 'trim', 0.001, seconds=0.7)
    G.rock(twig, 'mist', 10, seconds=1.2, axis='Y')   # nodding under the mister while it is misted and fed
    return objs


# --- the cedar sauna for one (1x2): seated in the middle on the bench, a glass door at the front ----------------
FRONT = 0.7     # the door's depth: the seated bather's feet inside it (2026-10-21: 0.42, the shoes at and past the glass: I-packs-17)


def soloSauna():
    F = G.GameFrame()
    zb, zf = WALL2, FRONT + 0.01       # the cabin's back and front
    zc, d = (zb + zf) / 2, zf - zb
    with G.new_parts() as still:
        F.box('floor', 0.9, 0.06, d, (0, 0.03, zc), M.cedar, bevel=0.01, bevel_segments=1)
        F.box('back_wall', 0.86, 1.96, 0.05, (0, 1.04, zb + 0.025), M.cedar, bevel=0.01, bevel_segments=1)
        # the roof: its deck, a fascia round the front and sides, a drip edge
        F.box('roof', 0.94, 0.05, d + 0.05, (0, 2.06, zc + 0.025), M.cedar, bevel=0.012, bevel_segments=1)   # its back on the wall face
        F.box('roof_fascia', 0.94, 0.05, 0.02, (0, 2.0, zf + 0.04), M.cedar_dark, bevel=0.004, bevel_segments=1)
        for s in (-1, 1):
            F.box(_name('roof_fascia_side'), 0.02, 0.05, d + 0.04, (s * 0.46, 2.0, zc + 0.02), M.cedar_dark, bevel=0.004,
                  bevel_segments=1)
        for s in (-1, 1):
            x = s * 0.425
            # the side wall in horizontal boards, each a hair proud or shy of the next, between corner posts
            for k in range(14):
                y = 0.08 + 0.137 * k + 0.0685
                F.box(_name('board'), 0.04 + 0.002 * (k % 2), 0.133, d - 0.1, (x, y, zc - 0.01), M.cedar, bevel=0.004,
                      bevel_segments=1)
            F.box(_name('post'), 0.05, 1.96, 0.06, (x, 1.04, zf - 0.02), M.cedar, bevel=0.01, bevel_segments=1)
            F.box(_name('corner'), 0.05, 1.96, 0.05, (x, 1.04, zb + 0.025), M.cedar, bevel=0.01, bevel_segments=1)
            F.box(_name('base_board'), 0.05, 0.08, d - 0.08, (x, 0.1, zc - 0.01), M.cedar_dark, bevel=0.004, bevel_segments=1)
        F.box('lintel', 0.82, 0.08, 0.06, (0, 1.97, zf - 0.02), M.cedar, bevel=0.01, bevel_segments=1)
        F.box('sill', 0.82, 0.06, 0.06, (0, 0.06, zf - 0.02), M.cedar, bevel=0.008, bevel_segments=1)
        # the bench (top 0.45) of rounded slats on its skirt of upright boards, the upper tier, backrest slats
        for z in (-0.16, -0.06, 0.04, 0.14):
            F.box(_name('slat'), 0.78, 0.035, 0.085, (0, 0.432, z), M.cedar, bevel=0.01, bevel_segments=2)
        for k in range(6):
            F.box(_name('skirt_board'), 0.125, 0.38, 0.03, (-0.325 + 0.13 * k, 0.22, 0.17), M.cedar, bevel=0.004, bevel_segments=1)
        for z in (-0.54, -0.44, -0.34):
            F.box(_name('tier_slat'), 0.78, 0.035, 0.085, (0, 0.82, z), M.cedar, bevel=0.01, bevel_segments=2)
        for k in range(6):
            F.box(_name('tier_board'), 0.125, 0.4, 0.03, (-0.325 + 0.13 * k, 0.62, -0.31), M.cedar, bevel=0.004, bevel_segments=1)
        for y in (0.98, 1.12):
            F.box(_name('back_slat'), 0.74, 0.06, 0.025, (0, y, -0.86), M.cedar, bevel=0.01, bevel_segments=2)
        # the heater with its stones behind a cedar guard, a bucket and ladle, the lamp and the thermometer
        F.box('heater', 0.26, 0.42, 0.22, (0.22, 1.07, -0.76), M.iron, bevel=0.015, bevel_segments=1)
        F.box('heater_top', 0.28, 0.03, 0.24, (0.22, 1.295, -0.76), M.dark, bevel=0.008, bevel_segments=1)
        for k in range(4):
            F.box(_name('heater_vent'), 0.18, 0.012, 0.004, (0.22, 0.95 + 0.04 * k, -0.648), M.dark)
        for k, (dx, dz) in enumerate(((-0.07, -0.06), (0.06, -0.05), (0.0, 0.05), (-0.08, 0.06), (0.08, 0.06), (0.0, -0.01),
                                      (-0.03, 0.0), (0.04, 0.01))):
            G.boulder(_name('stone'), 0.045, at=F.at(0.22 + dx, 1.305, -0.76 + dz), material=M.stones, seed=k * 1.7,
                      subdivisions=1, scale=(1, 1, 0.75))
        for k in range(3):
            F.box(_name('guard'), 0.025, 0.36, 0.025, (0.07 + 0.11 * k, 1.07, -0.625), M.cedar, bevel=0.005, bevel_segments=1)
        _turned(F, 'bucket', -0.22, 0.837, -0.44, [(0, 0), (0.08, 0), (0.1, 0.16), (0.092, 0.16), (0.073, 0.015), (0, 0.015)],
                M.cedar_dark, 16)
        for y in (0.87, 0.95):
            G.torus(_name('bucket_band'), 0.087 + (y - 0.837) * 0.12, 0.005, at=F.at(-0.22, y, -0.44), material=M.copper,
                    major_segments=16, minor_segments=3)
        G.kit.tube(F, 'ladle', [(-0.22, 0.86, -0.44), (-0.19, 0.97, -0.42), (-0.16, 1.08, -0.4)], 0.008, M.cedar_dark, verts=6,
                   round=1)
        _turned(F, 'ladle_cup', -0.225, 0.86, -0.445, [(0, 0), (0.03, 0.01), (0.034, 0.03), (0.026, 0.03), (0, 0.016)],
                M.cedar_dark, 12)
        F.box('lamp', 0.3, 0.1, 0.06, (0, 1.8, zb + 0.08), M.lamp, bevel=0.02, bevel_segments=1)
        for k in range(5):
            F.box(_name('lamp_louvre'), 0.34, 0.012, 0.02, (0, 1.75 + 0.025 * k, zb + 0.115), M.cedar, rot=(25, 0, 0))
        F.box('lamp_shade', 0.34, 0.03, 0.08, (0, 1.86, zb + 0.08), M.cedar, bevel=0.006, bevel_segments=1)
        F.cyl('thermometer', 0.06, 0.02, (-0.3, 1.5, zb + 0.06), M.cedar_dark, rot=(90, 0, 0), verts=18, bevel=0.006,
              bevel_segments=1)
        F.cyl('thermometer_face', 0.05, 0.004, (-0.3, 1.5, zb + 0.072), M.cream, rot=(90, 0, 0), verts=18)
        F.box('thermometer_needle', 0.004, 0.04, 0.002, (-0.292, 1.512, zb + 0.075), M.needle, rot=(0, -30, 0))
        # a towel over the bench's front edge
        pts = [(0.0, 0.45, -0.12), (0.0, 0.456, 0.08), (0.0, 0.45, 0.16), (0.0, 0.425, 0.192), (0.0, 0.3, 0.2)]
        towel = F.sweep('towel', pts, None, M.cream, profile=G.rounded_rect(0.5, 0.012, 0.005, corner=1), up=(1, 0, 0), resolution=2)
        G.jitter(towel, 0.003, size=0.1, seed=1.0, up=0.5)
    G.join(still, 'body', origin=(0, 0, 0))
    # The glass door, hinged on its left post, its cedar pull outside on two stand-offs: `door_open` swings it out.
    hx = -0.39
    with G.new_parts() as door:
        F.box('door_glass', 0.78, 1.84, 0.016, (0, 1.02, FRONT), M.glass)
        G.kit.tube(F, 'pull', G.kit.fillet([(0.3, 0.8, FRONT + 0.008), (0.3, 0.8, FRONT + 0.06), (0.3, 1.3, FRONT + 0.06),
                                            (0.3, 1.3, FRONT + 0.008)], 0.03), 0.014, M.cedar_dark, verts=8, round=1)
        for y in (0.8, 1.3):
            F.cyl(_name('pull_rose'), 0.02, 0.006, (0.3, y, FRONT + 0.011), M.cedar_dark, rot=(90, 0, 0), verts=10)
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
        # the cabinet, round the hatch's recess, a moulded frame round the opening
        F.box('cabinet_low', 0.82, lo, d, (0, lo / 2, zc), M.slate, bevel=0.015, bevel_segments=1)
        F.box('cabinet_high', 0.82, 1.9 - hi, d, (0, (1.9 + hi) / 2, zc), M.slate, bevel=0.015, bevel_segments=1)
        for s in (-1, 1):
            F.box(_name('cabinet_side'), (0.82 - hw) / 2, hh + 0.02, d, (s * (0.41 + hw / 2) / 2, hy, zc), M.slate)
        G.kit.moulding(F, 'hatch_frame', [(hw / 2 + 0.005, lo - 0.005), (hw / 2 + 0.005, hi + 0.005), (-hw / 2 - 0.005, hi + 0.005),
                                          (-hw / 2 - 0.005, lo - 0.005)],
                       [(-0.03, 0.0), (-0.03, 0.006), (-0.012, 0.012), (0.0, 0.012), (0.0, 0.0)], M.slate, at=HZ)
        F.box('recess', hw, hh, 0.02, (0, hy, WALL + 0.07), M.dark)
        F.box('recess_floor', hw, 0.01, d - 0.08, (0, lo + 0.005, zc + 0.04), M.steel)
        F.box('parcel', 0.2, 0.12, 0.14, (0.08, lo + 0.07, zc + 0.03), M.kraft, bevel=0.006, bevel_segments=1)
        F.box('parcel_tape', 0.2, 0.004, 0.03, (0.08, lo + 0.132, zc + 0.03), M.mustard)
        F.box('parcel_label', 0.06, 0.003, 0.05, (0.13, lo + 0.1315, zc + 0.06), M.cream)
        F.box('cap', 0.84, 0.05, d + 0.03, (0, 1.925, zc + 0.015), M.wood, bevel=0.01, bevel_segments=1)
        F.sweep('cap_nose', [(-0.42, 1.905, HZ + 0.03), (0.42, 1.905, HZ + 0.03)], 0.012, M.wood, verts=8)
        # the ledge on two scrolled brackets, a bag and a drink waiting on it
        F.box('ledge', 0.66, 0.035, 0.26, (0, 0.94, HZ + 0.13), M.wood, bevel=0.01, bevel_segments=2)
        for x in (-0.28, 0.28):
            F.extrude(_name('bracket'), G.smooth_outline([(0.0, 0.0), (0.0, 0.12), (0.2, 0.12), (0.18, 0.1), (0.08, 0.07),
                                                          (0.03, 0.02)], 1), 0.04, (x, 0.802, HZ), M.wood, plane='side',
                      bevel=0.006, bevel_segments=1)
        F.soft('bag', 0.2, 0.26, 0.14, (-0.14, 1.09, -0.06), M.kraft, round=0.012, puff=0.008, flat=2)
        F.box('bag_fold', 0.204, 0.04, 0.12, (-0.14, 1.235, -0.06), M.kraft, rot=(0, 4, 0), bevel=0.008, bevel_segments=1)
        F.box('sticker', 0.08, 0.06, 0.004, (-0.14, 1.1, 0.012), M.terra, bevel=0.002, bevel_segments=1)
        _turned(F, 'cup', 0.14, 0.9575, -0.04, [(0, 0), (0.032, 0), (0.045, 0.15), (0.047, 0.155), (0.047, 0.16), (0, 0.16)],
                M.porcelain, 16)
        _turned(F, 'cup_lid', 0.14, 1.1175, -0.04, [(0, 0), (0.049, 0), (0.049, 0.008), (0.03, 0.016), (0, 0.016)], M.porcelain, 16)
        F.box('cup_sleeve', 0.002, 0.05, 0.002, (0.14, 1.03, -0.04), M.porcelain)
        _turned(F, 'cup_band', 0.14, 1.0, -0.04, [(0.039, 0), (0.042, 0), (0.044, 0.05), (0.041, 0.05)], M.kraft, 16)
        G.kit.tube(F, 'straw', [(0.14, 1.1, -0.04), (0.16, 1.2, -0.05)], 0.005, M.needle, verts=6, round=1)
        # the order screen in its bezel and the "no knock" plaque
        F.box('screen_bezel', 0.24, 0.17, 0.02, (0.24, 1.55, HZ + 0.01), M.dark, bevel=0.01, bevel_segments=2)
        F.box('screen', 0.2, 0.13, 0.004, (0.24, 1.55, HZ + 0.021), M.screen, bevel=0.003, bevel_segments=1)
        F.box('screen_line_a', 0.16, 0.02, 0.004, (0.24, 1.58, HZ + 0.023), M.screen_a)
        F.box('screen_line_b', 0.1, 0.02, 0.004, (0.21, 1.53, HZ + 0.023), M.dial)
        F.box('plaque', 0.26, 0.09, 0.012, (-0.18, 1.55, HZ + 0.006), M.cream, bevel=0.006, bevel_segments=1)
        F.box('plaque_bar', 0.2, 0.012, 0.004, (-0.18, 1.55, HZ + 0.013), M.needle)
        for x in (-0.29, -0.07):
            F.cyl(_name('plaque_screw'), 0.006, 0.004, (x, 1.55, HZ + 0.013), M.steel, rot=(90, 0, 0), verts=8)
        # the parcel drawer with its pull and letter slot, the mat
        F.box('drawer', 0.62, 0.3, 0.02, (0, 0.4, HZ + 0.01), M.steel, bevel=0.008, bevel_segments=2)
        G.kit.pull(F, 'drawer_pull', 0, 0.48, HZ + 0.02, 0.2, M.chrome, r=0.007, out=0.025, verts=8)
        F.box('drawer_slot', 0.3, 0.012, 0.004, (0, 0.33, HZ + 0.021), M.dark)
        F.box('drawer_flap', 0.32, 0.03, 0.006, (0, 0.352, HZ + 0.022), M.steel, bevel=0.002, bevel_segments=1)
        F.soft('mat', 0.62, 0.014, 0.42, (0, 0.007, 0.2), M.mat, round=0.006, round_segments=1, flat=1)
        F.sweep('mat_border', [(u, 0.014, v + 0.2) for u, v in G.rounded_rect(0.54, 0.34, 0.02, corner=2)], None, M.dark,
                profile=[(-0.0015, -0.008), (0.0015, -0.008), (0.0015, 0.008), (-0.0015, 0.008)], up=(0, 0, 1), closed=True)
    G.join(still, 'body', origin=(0, 0, 0))
    # The hatch door, hinged on its right edge: a steel door with its window framed, a bar pull; `hatch` swings it open.
    with G.new_parts() as door:
        F.box('hatch_door', hw, hh, 0.03, (0, hy, HZ + 0.015), M.steel, bevel=0.012, bevel_segments=2)
        F.box('hatch_window', 0.3, 0.16, 0.004, (0, hy + 0.06, HZ + 0.031), M.screen, bevel=0.004, bevel_segments=1)
        G.kit.moulding(F, 'window_frame', [(0.155, hy - 0.025), (0.155, hy + 0.145), (-0.155, hy + 0.145), (-0.155, hy - 0.025)],
                       [(-0.01, 0.0), (-0.01, 0.004), (0.0, 0.006), (0.004, 0.0)], M.steel, at=HZ + 0.03)
        G.kit.pull(F, 'hatch_handle', 0, hy - 0.12, HZ + 0.03, 0.3, M.chrome, r=0.008, out=0.03, verts=8)
    door = G.hinged(G.join(door, 'hatch'), F.at(hw / 2, hy, HZ + 0.03))
    objs = G.settle(centre=False)
    G.swing(door, 'hatch', -105, seconds=0.8)
    return objs


FAMILY = G.Family(
    'pack_loner', M,
    [G.Model('quietPod', quietPod, size=(0.84, 1.539, 0.769), key='model.loner.quietPod',
             centre=(0, -0.0455),
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
             motions={'uses': {'shape': 'trim'}, 'loops': {'tend': 'mist'}},
             paint=[('body', 'Body'), ('teak', 'Wood'), ('glaze', 'Pot')]),
     G.Model('soloSauna', soloSauna, size=(0.94, 2.085, 1.705), key='model.loner.soloSauna',
             centre=(0, -0.0775),
             # the door lets the bather in, shuts behind them and opens again to let them out
             motions={'uses': {'sweat': {'clip': 'door_open', 'hold': 1.5, 'out': 1.5}}},
             paint=[('cedar', 'Wood'), ('cedar_dark', 'Trim')]),
     G.Model('deliveryHatch', deliveryHatch, size=(0.84, 1.95, 0.84), key='model.loner.deliveryHatch',
             centre=(0, -0.01),
             motions={'uses': {'order': 'hatch', 'treat': 'hatch'}, 'hold': 2},
             paint=[('slate', 'Body'), ('wood', 'Wood')])],
    comment='Blender-built loner pack models (tools/art/blender/families/pack_loner.py). Listed in the '
            'base manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw '
            'call per finish group. Each moves a part while it is used: the pod\'s hood, the radio\'s dial '
            'and antenna, a patch of the puzzle, a twig of the bonsai, the sauna\'s door and the delivery '
            'hatch.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

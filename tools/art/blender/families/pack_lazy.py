# The lazy pack (P10): the snoozecloud adjustable bed, the daydream nap pod, the heated quilt table, the doorstep
# delivery locker, the effortless vibration plate and the smart-home command hub.
# Notes: pack_lazy.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`): the bed's head section rises (`head_up`), the pod's hood swings half over (`hood`), the quilt's edge
# lifts (`quilt`), a locker door opens (`door_open`), the plate's deck shakes (`shake`, a loop) and the tablet
# tilts up on its dock (`screen_up`).
#
# Run:  python3 -B tools/art/blender/build.py pack_lazy [--only napPod] [--preview]
import math
import os
import sys

import bpy

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402
from gamekit.shapes import _recalc_normals  # noqa: E402

M = G.Palette(
    body=('#E9E4F4', 'satin'),        # the bed's puffy base and cloud headboard; the main part
    upholstery=('#F4F2FA', 'fabric'),  # the bed's mattress
    pillow=('#FFFFFF', 'fabric'),
    throw=('#CFC6EA', 'fabric'),
    blush=('#F2C6CF', 'fabric'),
    lilac=('#CFC6EA', 'fabric'),
    cloud=('#F7F7FA', 'satin'),
    birch=('#E3CFA8', 'satin'),
    shell=('#F4F4F2', 'gloss'),       # the nap pod's egg
    seat=('#7E95B8', 'fabric'),       # the nap pod's cushions and hood
    hood=('#8FA6C6', 'fabric'),
    shag=('#E6DCCB', 'fabric'),
    chrome=('#D0D3D6', 'chrome'),
    oak=('#CDB08A', 'satin'),         # the quilt table's top, plain (the quilt takes the fabric draw)
    quilt=('#C96A5B', 'fabric'),      # the kotatsu quilt
    quilt_band=('#E8B07A', 'fabric'),
    pouf=('#E8D7B8', 'fabric'),
    pouf_top=('#F3E7CF', 'fabric'),
    sage=('#B9C7A8', 'fabric'),
    heater=('#5A2A24', 'satin'),      # under the quilt
    porcelain=('#F2F2F0', 'gloss'),
    mandarin=('#F28C28', 'gloss'),
    biscuit=('#E8C27A', 'satin'),
    locker=('#EEF2F0', 'satin'),      # the delivery locker's cabinet
    door=('#CFE7DC', 'satin'),        # its doors
    accent=('#F28C28', 'satin'),      # its roof band; the plate's grips
    steel=('#8A9096', 'metal'),
    bezel=('#2B3440', 'satin'),
    screen=('#2E7DB8', 'gloss'),
    screen_light=('#3C8DBF', 'gloss'),
    white=('#F2F2F0', 'gloss'),
    led_green=('#7FB800', 'gloss'),
    cubby=('#3A3F44', 'satin'),
    kraft=('#C8A27A', 'matte'),
    tape=('#E9DFC8', 'matte'),
    bag=('#F2EEE6', 'matte'),
    bag_logo=('#D7263D', 'matte'),
    plate=('#F2F2F0', 'gloss'),       # the vibration plate's base and column
    lime=('#9BD45A', 'satin'),
    pad=('#3D4A57', 'matte'),
    pad_inner=('#4C5B6A', 'matte'),
    readout=('#6CC3F0', 'gloss'),
    bottle=('#7FC4E8', 'gloss'),
    wood=('#B07A4E', 'wood'),         # the smart hub's side table
    speaker=('#BFC3C7', 'satin'),
    ring=('#4FD1C5', 'gloss'),
    app_yellow=('#F2C14E', 'gloss'),
)

WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


def _sphere_band(name, F, centre, r_in, r_out, sides, pitch, material, n_side=14, n_pitch=10):
    """A closed shell of a sphere about `centre` (game): points at side angle `sides` (from the middle towards
    ±x) and pitch `pitch` (from the top, negative towards the back), between radii `r_in` and `r_out`. A
    pram-hood shape; turning it about the centre's x axis keeps it on its sphere."""
    a0, a1 = (math.radians(a) for a in sides)
    p0, p1 = (math.radians(p) for p in pitch)
    cx, cy, cz = centre
    verts, faces = [], []

    def grid(r):
        base = len(verts)
        for i in range(n_side + 1):
            a = a0 + (a1 - a0) * i / n_side
            for j in range(n_pitch + 1):
                p = p0 + (p1 - p0) * j / n_pitch
                verts.append(F.at(cx + r * math.sin(a), cy + r * math.cos(a) * math.cos(p), cz + r * math.cos(a) * math.sin(p)))
        return base

    def idx(base, i, j):
        return base + i * (n_pitch + 1) + j

    o, n = grid(r_out), grid(r_in)
    for i in range(n_side):
        for j in range(n_pitch):
            faces.append((idx(o, i, j), idx(o, i + 1, j), idx(o, i + 1, j + 1), idx(o, i, j + 1)))
            faces.append((idx(n, i, j + 1), idx(n, i + 1, j + 1), idx(n, i + 1, j), idx(n, i, j)))
    for i in range(n_side):                       # the two pitch edges
        for j in (0, n_pitch):
            faces.append((idx(o, i, j), idx(n, i, j), idx(n, i + 1, j), idx(o, i + 1, j)))
    for j in range(n_pitch):                      # the two side edges
        for i in (0, n_side):
            faces.append((idx(o, i, j), idx(o, i, j + 1), idx(n, i, j + 1), idx(n, i, j)))
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    _recalc_normals(obj)
    for poly in obj.data.polygons:
        poly.use_smooth = True
    return obj


# --- the snoozecloud adjustable bed (2x2): two sleepers side by side, mattress top 0.49, head at the back ----
HINGE_Z = -0.42     # the head section's front edge (depth), where it rises from
MAT_TOP = 0.49


def snoozecloudBed():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('base', 1.86, 0.26, 1.86, (0, 0.15, 0), M.body, bevel=0.1, bevel_segments=3)
        for sx in (-1, 1):
            for sz in (-1, 1):
                F.box(_name('foot'), 0.12, 0.06, 0.12, (sx * 0.8, 0.03, sz * 0.8), M.birch, bevel=0.02)
        # the cloud headboard: puffs of two whites
        for i, (x, y, s) in enumerate(((-0.66, 0.62, 0.48), (-0.34, 0.78, 0.58), (0.0, 0.84, 0.62), (0.34, 0.78, 0.58),
                                       (0.66, 0.62, 0.48), (-0.52, 1.0, 0.44), (0.5, 1.0, 0.46), (0.0, 1.12, 0.42))):
            F.sphere(_name('cloud'), s / 2, (x, y, -0.78), M.cloud if i % 3 else M.body, scale=(1, 0.24 / s, 0.82),
                     segments=18, rings=10)
        # the foot of the mattress, a throw folded over its end, a remote on the side
        d = 0.93 - HINGE_Z
        F.box('mattress_foot', 1.74, 0.2, d - 0.01, (0, MAT_TOP - 0.1, HINGE_Z + d / 2 + 0.005), M.upholstery,
              bevel=0.06, bevel_segments=3)
        F.box('throw', 1.78, 0.04, 0.42, (0, MAT_TOP + 0.015, 0.66), M.throw, bevel=0.018, bevel_segments=2)
        F.box('throw_drop', 1.78, 0.2, 0.03, (0, MAT_TOP - 0.08, 0.875), M.throw, bevel=0.012)
        F.box('remote', 0.06, 0.02, 0.14, (0.78, MAT_TOP + 0.01, 0.3), M.porcelain, bevel=0.008)
        F.box('remote_buttons', 0.03, 0.004, 0.06, (0.78, MAT_TOP + 0.021, 0.28), M.screen, bevel=0.001)
    G.join(still, 'body', origin=(0, 0, 0))
    # The head section of the mattress and its pillows, hinged on its front edge: `head_up` raises it 30°.
    with G.new_parts() as head:
        hd = HINGE_Z + 0.77
        F.box('mattress_head', 1.74, 0.2, hd, (0, MAT_TOP - 0.1, HINGE_Z - hd / 2), M.upholstery, bevel=0.06,
              bevel_segments=3)
        for s in (-1, 1):
            F.box(_name('pillow'), 0.68, 0.14, 0.36, (s * 0.42, MAT_TOP + 0.06, -0.6), M.pillow, bevel=0.06,
                  bevel_segments=3)
            F.box(_name('cushion'), 0.22, 0.26, 0.1, (s * 0.77, MAT_TOP + 0.15, -0.68), M.blush if s < 0 else M.lilac,
                  rot=(18, 0, 0), bevel=0.045, bevel_segments=3)
    head = G.hinged(G.join(head, 'head_section'), F.at(0, MAT_TOP - 0.2, HINGE_Z))
    objs = G.settle(centre=False)
    G.swing(head, 'head_up', 30, seconds=1.2, axis='X')
    return objs


# --- the daydream nap pod (1x1): an egg chair open to the front, seat top 0.45, a hood that swings over -----
POD = (0.0, 0.78, 0.04)    # the egg's centre (x, height, depth)
POD_R, POD_H = 0.44, 0.56  # its radius across and its half height (a sitter's head reaches about 1.3)
HOOD = (0.0, 0.95, 0.04)   # the hood's centre: a sphere about the egg's upper half
HOOD_R = 0.48


def napPod():
    F = G.GameFrame()
    cx, cy, cz = POD
    with G.new_parts() as still:
        # the egg: an ellipsoid open at the front, revolved about the depth axis and stretched upwards
        prof = []
        for k in range(15):
            t = math.radians(100 * k / 14)
            prof.append((POD_R * math.sin(t), -POD_R * math.cos(t)))
        ri = POD_R - 0.035
        for k in range(14, -1, -1):
            t = math.radians(100 * k / 14)
            prof.append((ri * math.sin(t), -ri * math.cos(t)))
        G.revolve('egg', prof, at=F.at(cx, cy, cz), material=M.shell, segments=32, rot=(90, 0, 0),
                  scale=(1, POD_H / POD_R, 1))
        F.cone('stem', 0.07, 0.045, 0.24, (0, 0.16, cz), M.chrome, verts=16)
        F.cyl('base_disc', 0.28, 0.04, (0, 0.02, cz), M.chrome, verts=32, bevel=0.01)
        # the cushions: a round seat (top 0.45), a back cushion and a little pillow
        F.cyl('seat', 0.3, 0.1, (0, 0.4, 0.05), M.seat, verts=28, bevel=0.04, bevel_segments=3)
        F.sphere('back_cushion', 0.3, (0, 0.78, -0.24), M.seat, scale=(1, 0.3, 1.2), segments=20, rings=12)
        F.box('pillow', 0.26, 0.2, 0.12, (0.17, 0.56, -0.06), M.blush, rot=(-20, 0, 20), bevel=0.05, bevel_segments=3)
        # a shaggy rug in front
        F.cyl('rug', 0.27, 0.016, (0, 0.008, 0.3), M.shag, verts=24, scale=(1, 0.55, 1), bevel=0.006)
    G.join(still, 'body', origin=(0, 0, 0))
    # The hood: a band of a sphere about the egg's upper half, folded back over the egg's top and turning about
    # that sphere's centre; `hood` swings it 60° forward, half over the opening.
    hood = _sphere_band('hood', F, HOOD, HOOD_R, HOOD_R + 0.02, (-62, 62), (-108, -20), M.hood)
    G.join([hood], 'hood')
    hood = G.hinged(hood, F.at(*HOOD))
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.swing(hood, 'hood', 60, seconds=1.0, axis='X')
    return objs


# --- the heated quilt table (2x1): a kotatsu in front, two poufs (tops 0.45) at x = ±0.5 behind it ---------
TOP_Y = 0.6        # the underside of the table top
FLAP_Z = 0.035     # the quilt's back panel, facing the sitters


def quiltTable():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('top', 1.74, 0.035, 0.42, (0, TOP_Y + 0.0175, 0.23), M.oak, bevel=0.012)
        F.box('quilt', 1.7, TOP_Y - 0.01, 0.4, (0, (TOP_Y - 0.01) / 2 + 0.005, 0.25), M.quilt, bevel=0.07,
              bevel_segments=3)
        F.box('quilt_band', 1.6, 0.03, 0.402, (0, TOP_Y - 0.03, 0.25), M.quilt_band)
        F.box('heater_glow', 1.44, 0.24, 0.006, (0, 0.44, 0.047), M.heater)
        F.box('quilt_skirt', 1.62, 0.3, 0.03, (0, 0.17, FLAP_Z), M.quilt, bevel=0.012)
        # a bowl of mandarins, two mugs, a plate of biscuits
        G.revolve('bowl', [(0, 0), (0.06, 0), (0.13, 0.06), (0.125, 0.065), (0.055, 0.008), (0, 0.008)],
                  at=F.at(0, TOP_Y + 0.035, 0.25), material=M.porcelain, segments=20)
        for dx, dz, dy in ((-0.05, 0.22, 0), (0.05, 0.27, 0), (0.0, 0.31, 0), (-0.02, 0.2, 0.04), (0.04, 0.2, 0.0)):
            F.sphere(_name('mandarin'), 0.035, (dx, TOP_Y + 0.07 + dy, dz), M.mandarin, scale=(1, 1, 0.85), segments=12,
                     rings=8)
        for s in (-1, 1):
            x = s * 0.5
            G.revolve(_name('mug'), [(0, 0), (0.04, 0), (0.042, 0.09), (0.037, 0.09), (0.035, 0.01), (0, 0.01)],
                      at=F.at(x + s * 0.12, TOP_Y + 0.035, 0.16), material=M.porcelain, segments=14)
            # the pouf, its seat cushion (top 0.45) and a back cushion against the wall
            F.box(_name('pouf'), 0.5, 0.38, 0.46, (x, 0.2, -0.12), M.pouf, bevel=0.12, bevel_segments=3)
            F.box(_name('pouf_top'), 0.46, 0.06, 0.44, (x, 0.42, -0.12), M.pouf_top, bevel=0.025, bevel_segments=2)
            F.box(_name('back_cushion'), 0.44, 0.3, 0.1, (x, 0.6, -0.37), M.sage if s < 0 else M.lilac, rot=(12, 0, 0),
                  bevel=0.045, bevel_segments=3)
        F.box('snack_plate', 0.2, 0.012, 0.14, (0.6, TOP_Y + 0.041, 0.32), M.porcelain, bevel=0.004)
        for dx, dz in ((-0.02, -0.01), (0.03, 0.02)):
            F.cyl(_name('biscuit'), 0.025, 0.014, (0.6 + dx, TOP_Y + 0.054, 0.32 + dz), M.biscuit, verts=12)
    G.join(still, 'body', origin=(0, 0, 0))
    # The quilt's back edge, hung from the table top over the skirt: `quilt` lifts it over the sitters' laps.
    with G.new_parts() as flap:
        F.box('quilt_flap', 1.62, 0.29, 0.03, (0, TOP_Y - 0.145, FLAP_Z + 0.004), M.quilt, bevel=0.012)
        F.box('flap_band', 1.62, 0.03, 0.032, (0, TOP_Y - 0.03, FLAP_Z), M.quilt_band)
    flap = G.hinged(G.join(flap, 'quilt_flap'), F.at(0, TOP_Y, FLAP_Z))
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.swing(flap, 'quilt', 70, seconds=0.8, axis='X')
    return objs


# --- the doorstep delivery locker (1x1): a bank of parcel lockers with a touch screen, an order waiting -----
COLS = (-0.28, 0.0, 0.28)
ROWS = ((0.34, 0.5), (0.88, 0.5), (1.42, 0.5))   # (middle, height)
OPEN = (0, 1)                                    # the door that opens: left column, middle row
FRONT = 0.0


def deliveryLocker():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('cabinet', 0.86, 1.86, -WALL, (0, 0.93, WALL / 2), M.locker, bevel=0.02)
        F.box('plinth', 0.82, 0.06, 0.02, (0, 0.03, FRONT + 0.008), M.bezel)
        for c, x in enumerate(COLS):
            for r, (y, h) in enumerate(ROWS):
                if (c, r) == (1, 1):
                    continue
                if (c, r) == OPEN:
                    F.box('cubby', 0.24, h - 0.05, 0.004, (x, y, FRONT + 0.002), M.cubby)
                    F.box('cubby_bag', 0.12, 0.16, 0.004, (x - 0.02, y - 0.1, FRONT + 0.005), M.bag)
                    F.box('cubby_bag_logo', 0.06, 0.04, 0.002, (x - 0.02, y - 0.09, FRONT + 0.008), M.bag_logo)
                    continue
                F.box(_name('door'), 0.26, h - 0.03, 0.02, (x, y, FRONT + 0.01), M.door,
                      bevel=0.006)
                F.box(_name('handle'), 0.012, 0.08, 0.016, (x + 0.1, y, FRONT + 0.026), M.steel, bevel=0.003)
                F.sphere(_name('led'), 0.008, (x - 0.09, y + h / 2 - 0.07, FRONT + 0.021),
                         M.led_green if (r + c) % 3 else M.accent, segments=8, rings=4)
        F.box('bezel', 0.26, 0.47, 0.03, (0, 0.88, FRONT + 0.008), M.bezel, bevel=0.01)
        F.box('touch_screen', 0.22, 0.3, 0.004, (0, 0.94, FRONT + 0.025), M.screen, bevel=0.004)
        F.box('screen_banner', 0.18, 0.05, 0.003, (0, 1.0, FRONT + 0.028), M.white)
        F.box('screen_button', 0.08, 0.08, 0.003, (0, 0.87, FRONT + 0.028), M.accent, bevel=0.008)
        F.box('keypad', 0.12, 0.06, 0.006, (0, 0.72, FRONT + 0.024), M.steel)
        F.box('roof', 0.86, 0.1, -WALL + 0.02, (0, 1.91, WALL / 2 + 0.01), M.accent, bevel=0.02)
        F.box('logo', 0.4, 0.06, 0.004, (0, 1.91, FRONT + 0.032), M.white, bevel=0.002)
        # the order waiting on the doorstep: a parcel and a takeout bag
        F.box('parcel', 0.32, 0.22, 0.24, (0.2, 0.11, 0.24), M.kraft, rot=(0, 0, -8), bevel=0.006)
        F.box('parcel_tape', 0.33, 0.02, 0.05, (0.2, 0.221, 0.24), M.tape, rot=(0, 0, -8))
        F.box('takeout_bag', 0.22, 0.26, 0.14, (-0.22, 0.13, 0.22), M.bag, rot=(0, 0, 6), bevel=0.015)
        F.box('takeout_logo', 0.12, 0.08, 0.004, (-0.215, 0.16, 0.293), M.bag_logo, rot=(0, 0, 6))
        for dx in (-0.05, 0.05):
            G.rod(_name('bag_handle'), F.at(-0.22 + dx, 0.26, 0.2), F.at(-0.22 + dx, 0.31, 0.22), 0.006,
                  material=M.tape, verts=5)
    G.join(still, 'body', origin=(0, 0, 0))
    # One locker's door, hinged on its left edge: `door_open` swings it open for a takeout.
    x, (y, h) = COLS[OPEN[0]], ROWS[OPEN[1]]
    with G.new_parts() as door:
        F.box('locker_door', 0.26, h - 0.03, 0.02, (x, y, FRONT + 0.01), M.door, bevel=0.006)
        F.box('locker_handle', 0.012, 0.08, 0.016, (x + 0.1, y, FRONT + 0.026), M.steel, bevel=0.003)
        F.sphere('locker_led', 0.008, (x - 0.09, y + h / 2 - 0.07, FRONT + 0.021), M.accent, segments=8, rings=4)
    door = G.hinged(G.join(door, 'locker_door'), F.at(x - 0.13, y, FRONT + 0.01))
    objs = G.settle(centre=False)
    G.swing(door, 'door_open', 105, seconds=0.7)
    return objs


# --- the effortless vibration plate (1x1): a deck at 0.125 the resident stands on, a column with a console --
DECK = (0.0, 0.105, 0.12)   # the deck's middle, at its underside


def vibePlate():
    F = G.GameFrame()
    dx, dy, dz = DECK
    with G.new_parts() as still:
        F.box('base', 0.62, dy, 0.6, (0, dy / 2, dz - 0.02), M.plate, bevel=0.04, bevel_segments=3)
        F.box('band', 0.63, 0.02, 0.61, (0, 0.025, dz - 0.02), M.lime, bevel=0.008)
        G.rod('column', F.at(0, 0.09, -0.22), F.at(0, 1.0, -0.3), 0.04, material=M.plate, verts=16)
        F.box('handlebar', 0.5, 0.05, 0.08, (0, 1.02, -0.29), M.plate, bevel=0.022)
        for s in (-1, 1):
            F.box(_name('grip'), 0.09, 0.065, 0.09, (s * 0.24, 1.02, -0.27), M.lime, bevel=0.028)
        F.box('console', 0.22, 0.14, 0.04, (0, 1.1, -0.29), M.bezel, rot=(30, 0, 0), bevel=0.015)
        F.box('console_screen', 0.18, 0.09, 0.004, (0, 1.105, -0.268), M.readout, rot=(30, 0, 0))
        G.revolve('bottle', [(0, 0), (0.035, 0), (0.035, 0.17), (0.02, 0.2), (0, 0.2)], at=F.at(0.32, 0, -0.32),
                  material=M.bottle, segments=14)
        F.cyl('bottle_cap', 0.018, 0.02, (0.32, 0.21, -0.32), M.white, verts=10)
    G.join(still, 'body', origin=(0, 0, 0))
    # The deck the resident stands on: `shake` rocks it to and fro, quickly, while they shake.
    with G.new_parts() as deck:
        F.box('deck', 0.54, 0.012, 0.48, (dx, dy + 0.006, dz), M.pad, bevel=0.01)
        F.box('deck_pad', 0.5, 0.008, 0.44, (dx, dy + 0.016, dz), M.pad_inner, bevel=0.01)
        for s in (-1, 1):
            F.box(_name('foot_mark'), 0.12, 0.003, 0.22, (dx + s * 0.13, dy + 0.0205, dz), M.lime, bevel=0.001)
    deck = G.hinged(G.join(deck, 'deck'), F.at(dx, dy, dz))
    objs = G.settle(centre=False)
    G.rock(deck, 'shake', 3.0, seconds=0.25, axis='Y')
    return objs


# --- the smart-home command hub (1x1): a side table with a smart speaker, a tablet on its dock, a robot ------
TABLE = (0.6, 0.6, 0.44)   # width, top height, depth


def smartHub():
    F = G.GameFrame()
    w, h, d = TABLE
    z = WALL + d / 2
    with G.new_parts() as still:
        F.box('table_top', w, 0.03, d, (0, h - 0.015, z), M.wood, bevel=0.008)
        F.box('apron', w - 0.06, 0.06, d - 0.06, (0, h - 0.06, z), M.wood, bevel=0.006)
        F.box('shelf', w - 0.05, 0.02, d - 0.05, (0, 0.14, z), M.wood, bevel=0.005)
        for lx in (-w / 2 + 0.03, w / 2 - 0.03):
            for lz in (z - d / 2 + 0.03, z + d / 2 - 0.03):
                F.box(_name('leg'), 0.04, h - 0.03, 0.04, (lx, (h - 0.03) / 2, lz), M.wood, bevel=0.008)
        # the smart speaker with its ring
        F.cyl('speaker', 0.065, 0.2, (-0.14, h + 0.1, z - 0.02), M.speaker, verts=22, bevel=0.01)
        G.torus('speaker_ring', 0.058, 0.006, at=F.at(-0.14, h + 0.2, z - 0.02), material=M.ring, major_segments=22,
                minor_segments=4)
        F.cyl('speaker_top', 0.052, 0.006, (-0.14, h + 0.201, z - 0.02), M.bezel, verts=22)
        # the tablet's dock, a few books on the shelf, a robot vacuum in front
        F.box('dock', 0.2, 0.025, 0.1, (0.12, h + 0.0125, z + 0.02), M.white, bevel=0.008)
        F.box('book_a', 0.22, 0.04, 0.16, (-0.1, 0.17, z), M.screen_light, bevel=0.004)
        F.box('book_b', 0.2, 0.035, 0.15, (-0.1, 0.207, z), M.app_yellow, rot=(0, 0, 6), bevel=0.004)
        F.cyl('robot', 0.15, 0.06, (0, 0.035, 0.26), M.white, verts=28, bevel=0.015)
        F.cyl('robot_band', 0.152, 0.012, (0, 0.03, 0.26), M.bezel, verts=28)
        F.cyl('robot_button', 0.04, 0.008, (0, 0.068, 0.26), M.ring, verts=14)
    G.join(still, 'body', origin=(0, 0, 0))
    # The tablet, built standing up on its hinge in the dock, then laid back to rest: `screen_up` tilts it up.
    hy, hz = h + 0.02, z + 0.02
    with G.new_parts() as tab:
        F.box('tablet', 0.26, 0.18, 0.012, (0.12, hy + 0.09, hz), M.bezel, bevel=0.006)
        F.box('tablet_screen', 0.23, 0.15, 0.003, (0.12, hy + 0.09, hz + 0.007), M.screen_light)
        F.box('app_a', 0.07, 0.05, 0.002, (0.06, hy + 0.12, hz + 0.009), M.app_yellow)
        F.box('app_b', 0.07, 0.05, 0.002, (0.16, hy + 0.12, hz + 0.009), M.led_green)
    tab = G.hinged(G.join(tab, 'tablet'), F.at(0.12, hy, hz))
    G.turn(tab, -72, 'X')
    objs = G.settle(centre=False)
    G.swing(tab, 'screen_up', 57, seconds=0.8, axis='X')
    return objs


FAMILY = G.Family(
    'pack_lazy', M,
    [G.Model('snoozecloudBed', snoozecloudBed, size=(1.86, 1.292, 1.86), key='model.lazy.snoozecloudBed',
             motions={'uses': {'sleepIn': 'head_up'}, 'hold': 2},
             paint=[('body', 'Body'), ('upholstery', 'Upholstery'), ('throw', 'Throw')]),
     G.Model('napPod', napPod, size=(0.883, 1.42, 0.908), key='model.lazy.napPod', centre=(0, 0.024),
             motions={'uses': {'powerNap': 'hood', 'curlUp': 'hood'}},
             paint=[('shell', 'Body'), ('seat', 'Upholstery'), ('hood', 'Hood')]),
     G.Model('quiltTable', quiltTable, size=(1.74, 0.743, 0.892), key='model.lazy.quiltTable', centre=(0, 0.016),
             motions={'uses': {'tuckIn': 'quilt', 'doze': 'quilt'}},
             paint=[('oak', 'Body'), ('quilt', 'Upholstery'), ('pouf', 'Poufs')]),
     G.Model('deliveryLocker', deliveryLocker, size=(0.86, 1.96, 0.81), key='model.lazy.deliveryLocker', centre=(0, -0.025),
             motions={'uses': {'takeout': 'door_open', 'noodles': 'door_open'}, 'hold': 2},
             paint=[('locker', 'Body'), ('door', 'Doors'), ('accent', 'Trim')]),
     G.Model('vibePlate', vibePlate, size=(0.67, 1.165, 0.759), key='model.lazy.vibePlate', centre=(0.02, 0.025),
             motions={'loops': {'shake': 'shake'}},
             paint=[('plate', 'Body'), ('lime', 'Trim'), ('pad', 'Deck')]),
     G.Model('smartHub', smartHub, size=(0.6, 0.806, 0.842), key='model.lazy.smartHub', centre=(0, -0.009),
             motions={'uses': {'program': 'screen_up', 'story': 'screen_up'}},
             paint=[('wood', 'Body'), ('speaker', 'Speaker')])],
    comment='Blender-built lazy pack models (tools/art/blender/families/pack_lazy.py). Listed in the base '
            'manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw call '
            'per finish group. Each moves while it is used: the bed\'s head rises, the pod\'s hood swings '
            'over, the quilt lifts, a locker door opens, the plate shakes and the tablet tilts up.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

# The hot-headed pack (P8): the heavy punching bag, the plate-smash bin, the thunder drum kit, the cool-down
# meditation cushion, the grudge-to-gratitude writing desk and the glacier cold-plunge tub. Notes: pack_hotHeaded.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`): the bag swings on its chains while it is pounded (`swing`, a loop), the crate's lid flips up
# (`lid_open`), the crash and ride cymbals rock while the kit is played (`cymbals`, a loop), the zafu sinks
# (`sink_in`), the desk's drawer slides out (`drawer_open`) and the plunge tub's slatted cover rolls back into
# its housing (`cover_off`).
#
# Run:  cargo art pack_hotHeaded [--only heavyBag] [--preview]
import math
import os
import random
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    body=('#C49A6C', 'wood'),        # the grudge desk's oak top and its chair; the desk's main part
    bag=('#C8402F', 'satin'),        # the heavy bag's leather
    crate=('#A87547', 'wood'),       # the smash crate and its lid
    shell=('#C8402F', 'gloss'),      # the drum shells
    zafu=('#5F7FA0', 'fabric'),      # the meditation cushion
    cedar=('#C98B58', 'wood'),       # the plunge tub's cladding
    fire=('#C8402F', 'satin'),       # red details: weight plate, gloves, lamp shade
    ember=('#E07A2E', 'satin'),
    flame=('#F2B84B', 'gloss'),      # logos, hazard tape
    steel=('#AEB4BA', 'metal'),
    steel_dark=('#7E858C', 'metal'),
    chrome=('#D0D3D6', 'chrome'),
    cymbal=('#C9A24A', 'metal'),
    brass=('#C9A54A', 'metal'),
    dark=('#2F3236', 'satin'),       # straps, castors, the drum throne, pen
    rubber=('#3E4A55', 'satin'),     # floor mats, the drum rug
    rug=('#6E3A2E', 'satin'),
    head=('#F4F1EA', 'satin'),       # drum heads
    porcelain=('#F4F1EA', 'gloss'),  # plates and shards
    plate_blue=('#9CC3DB', 'gloss'),
    plate_gold=('#F2D7A0', 'gloss'),
    plate_green=('#C8E0C0', 'gloss'),
    crate_dark=('#7C5432', 'wood'),  # the crate's posts and floor
    straw=('#D2B872', 'matte'),
    oak=('#B88A5C', 'satin'),        # plain wood: stool, bowl stand, drum sticks, mallet
    calm=('#5F7FA0', 'satin'),       # the chair's cushion
    calm_light=('#A9C2D6', 'fabric'),  # the meditation mat
    calm_deep=('#4E6E8E', 'fabric'),   # the zafu's bands and the mat's edge
    oat=('#E6DCC8', 'fabric'),       # the zabuton
    candle=('#FBF6EC', 'satin'),
    candle_flame=('#FFD27A', 'gloss'),
    leaf=('#5E8F4E', 'satin'),
    pot=('#F4F1EA', 'satin'),
    paper=('#F4EFE2', 'matte'),
    journal=('#7A3A2A', 'satin'),
    drawer=('#E6E3DC', 'satin'),     # the desk's drawer case and the chiller
    cactus=('#6E9E58', 'satin'),
    shell_white=('#F2F2F0', 'satin'),  # the plunge tub's liner and rims
    water=('#BFE6F0', 'gloss'),
    ice=('#F4FAFC', 'gloss'),
    cover=('#5F7FA0', 'satin'),      # the plunge cover's slats
    screen=('#1F3A40', 'gloss'),
    readout=('#7FD1F0', 'gloss'),
    hose=('#5F6368', 'satin'),
    towel=('#F4EFE2', 'satin'),
    amp=('#E4E1DA', 'satin'),
    speaker=('#3A3D41', 'satin'),
)

WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


# --- the heavy punching bag (1x1): a free-standing frame with a boom, the bag on chains --------------------
HOOK = (0.0, 1.9, 0.1)   # the swivel the bag hangs from (x, height, depth)


def heavyBag():
    F = G.GameFrame()
    hx, hy, hz = HOOK
    with G.new_parts() as still:
        F.box('mat', 0.86, 0.012, 0.86, (0, 0.006, 0), M.rubber, bevel=0.004)
        F.box('foot_long', 0.08, 0.06, 0.78, (0, 0.042, -0.04), M.steel_dark, bevel=0.01)
        F.box('foot_cross', 0.7, 0.06, 0.08, (0, 0.042, -0.37), M.steel_dark, bevel=0.01)
        F.box('upright', 0.08, 1.93, 0.08, (0, 1.035, -0.37), M.steel, bevel=0.012)
        G.rod('brace', F.at(0, 0.5, -0.34), F.at(0, 0.07, 0.1), 0.022, material=M.steel, verts=10)
        F.box('boom', 0.08, 0.08, 0.53, (0, 1.96, -0.145), M.steel, bevel=0.012)
        F.box('boom_cap', 0.09, 0.012, 0.09, (0, 2.004, -0.37), M.steel_dark, bevel=0.003)
        F.cyl('swivel_eye', 0.025, 0.05, (hx, 1.93, hz), M.chrome, verts=12)
        # weight plates on pegs at the foot, a hook with gloves on the upright
        for s, m in ((-1, M.fire), (1, M.dark)):
            F.cyl(_name('peg'), 0.015, 0.16, (s * 0.1, 0.17, -0.29), M.chrome, rot=(0, 90, 0), verts=10)
            F.cyl(_name('plate'), 0.13, 0.045, (s * 0.2, 0.17, -0.29), m, rot=(0, 90, 0), verts=24, bevel=0.006)
            F.cyl(_name('plate_hub'), 0.03, 0.05, (s * 0.2, 0.17, -0.29), M.chrome, rot=(0, 90, 0), verts=12)
        F.box('glove_hook', 0.2, 0.02, 0.03, (-0.13, 1.32, -0.35), M.steel, bevel=0.006)
        for x in (-0.24, -0.12):
            F.sphere(_name('glove'), 0.07, (x, 1.18, -0.3), M.fire, scale=(0.9, 0.85, 1.1), segments=14, rings=10)
            F.sphere(_name('glove_thumb'), 0.03, (x + 0.05, 1.2, -0.25), M.fire, segments=10, rings=6)
            F.cyl(_name('cuff'), 0.045, 0.08, (x, 1.08, -0.3), M.head, verts=12)
            G.rod(_name('lace'), F.at(x, 1.12, -0.3), F.at(-0.13, 1.31, -0.34), 0.004, material=M.dark, verts=5)
    G.join(still, 'body', origin=(0, 0, 0))
    # The bag, its straps and logo, the chains and swivel: one part hung from the swivel, `swing` rocks it
    # back and forth while it is pounded.
    with G.new_parts() as bag:
        G.revolve('bag', [(0, 0), (0.11, 0.004), (0.17, 0.04), (0.19, 0.11), (0.19, 0.89), (0.17, 0.96),
                          (0.11, 0.996), (0, 1.0)], at=F.at(hx, 0.66, hz), material=M.bag, segments=24)
        for y in (0.86, 1.5):
            F.cyl(_name('strap'), 0.194, 0.05, (hx, y, hz), M.dark, verts=24)
        F.box('logo', 0.1, 0.22, 0.01, (hx, 1.18, hz + 0.19), M.flame, bevel=0.004)
        F.cyl('swivel', 0.02, 0.06, (hx, 1.87, hz), M.chrome, verts=10)
        for a in (0, 120, 240):
            G.rod(_name('chain'), F.at(hx, 1.84, hz),
                  F.at(hx + 0.11 * G.cosd(a), 1.665, hz + 0.11 * G.sind(a)), 0.006, material=M.chrome, verts=6)
    bag = G.hinged(G.join(bag, 'bag'), F.at(hx, hy, hz))
    objs = G.settle(centre=False)
    G.rock(bag, 'swing', 8, seconds=1.4, axis='X')
    return objs


# --- the plate-smash bin (1x1): a slatted crate with a lid, a stool stacked with old plates ----------------
CRATE = (-0.08, 0.66, 0.6, WALL, 0.56)   # x, width, height, back depth, depth


def smashBin():
    F = G.GameFrame()
    rnd = random.Random(4)
    x, w, h, z0, d = CRATE
    z1, zc = z0 + d, z0 + d / 2
    with G.new_parts() as still:
        F.box('mat', 0.84, 0.012, 0.84, (0, 0.006, -0.01), M.rubber, bevel=0.004)
        F.box('floor', w - 0.04, 0.03, d - 0.04, (x, 0.04, zc), M.crate_dark)
        G.boulder('straw', 0.3, at=F.at(x, 0.05, zc), material=M.straw, scale=(1.0, 0.86, 0.5), seed=2.0,
                  noise=0.15, subdivisions=2)
        for px in (x - w / 2 + 0.03, x + w / 2 - 0.03):
            for pz in (z0 + 0.03, z1 - 0.03):
                F.box(_name('post'), 0.06, h, 0.06, (px, h / 2, pz), M.crate_dark, bevel=0.006)
        sh = (h - 0.02) / 3
        for k in range(3):
            y = 0.01 + sh * (k + 0.5)
            for zz in (z1 - 0.011, z0 + 0.011):
                F.box(_name('slat'), w - 0.02, sh - 0.025, 0.022, (x, y, zz), M.crate)
            for xx in (x - w / 2 + 0.011, x + w / 2 - 0.011):
                F.box(_name('slat'), 0.022, sh - 0.025, d - 0.02, (xx, y, zc), M.crate)
        # hazard tape across the front
        F.box('tape', 0.6, 0.05, 0.004, (x, 0.36, z1 + 0.002), M.dark)
        for i in range(5):
            F.box(_name('tape_stripe'), 0.06, 0.05, 0.005, (x - 0.24 + i * 0.12, 0.36, z1 + 0.003), M.flame,
                  rot=(0, 0, 30))
        # broken plates on the straw and the mat
        for dx, dz, r in ((-0.12, -0.05, 20), (0.1, 0.06, -35), (0.02, -0.12, 70)):
            F.box(_name('shard'), 0.1, 0.01, 0.06, (x + dx, 0.2, zc + dz), M.porcelain, rot=(6, 0, r))
        for sx, sz, r in ((-0.3, 0.32, 20), (-0.15, 0.36, -40), (0.06, 0.4, 70), (-0.38, 0.24, 10)):
            F.box(_name('shard'), 0.08, 0.01, 0.05, (sx, 0.017, sz), M.porcelain, rot=(0, 0, r))
        # the stool and its stack of thrift-store plates
        sx, sz = 0.28, 0.25
        F.cyl('stool_top', 0.15, 0.03, (sx, 0.445, sz), M.oak, verts=20, bevel=0.006)
        for a in (90, 210, 330):
            G.rod(_name('stool_leg'), F.at(sx + 0.09 * G.cosd(a), 0.44, sz + 0.09 * G.sind(a)),
                  F.at(sx + 0.13 * G.cosd(a), 0.0, sz + 0.13 * G.sind(a)), 0.016, material=M.oak, verts=8)
        y = 0.46
        for i, m in enumerate((M.porcelain, M.plate_blue, M.porcelain, M.plate_gold, M.porcelain, M.plate_green)):
            G.revolve(_name('plate'), [(0, 0), (0.07, 0), (0.12, 0.014), (0.115, 0.018), (0.066, 0.006), (0, 0.006)],
                      at=F.at(sx + rnd.uniform(-0.008, 0.008), y, sz + rnd.uniform(-0.008, 0.008)), material=m,
                      segments=20)
            y += 0.02
        # safety goggles hung on the crate's front corner post
        gx, gz = x + w / 2 - 0.03, z1 + 0.035
        F.box('goggle_strap', 0.012, 0.03, 0.1, (gx + 0.03, 0.56, gz - 0.04), M.dark)
        F.box('goggles', 0.16, 0.055, 0.04, (gx + 0.03, 0.5, gz), M.flame, bevel=0.018, bevel_segments=3)
    G.join(still, 'body', origin=(0, 0, 0))
    # The lid: three boards on two battens, hinged on the crate's back top edge: `lid_open` flips it up.
    lz0 = z0 + 0.03
    with G.new_parts() as lid:
        for k in range(3):
            bd = (z1 - lz0) / 3
            F.box(_name('lid_board'), w + 0.02, 0.022, bd - 0.006, (x, h + 0.011, lz0 + bd * (k + 0.5)), M.crate,
                  bevel=0.004)
        for xx in (x - w / 2 + 0.08, x + w / 2 - 0.08):
            F.box(_name('lid_batten'), 0.05, 0.02, z1 - lz0 - 0.04, (xx, h + 0.032, (lz0 + z1) / 2), M.crate_dark,
                  bevel=0.004)
        F.box('lid_grip', 0.16, 0.03, 0.035, (x, h + 0.015, z1 + 0.015), M.crate_dark, bevel=0.01)
    lid = G.hinged(G.join(lid, 'lid'), F.at(x, h, lz0))
    objs = G.settle(centre=False)
    G.swing(lid, 'lid_open', -80, seconds=0.6, axis='X')
    return objs


# --- the thunder drum kit (2x1): the drummer on the throne at the middle, the kit before them --------------
def _drum(F, x, y, z, dia, depth, axis='y', tilt=0.0):
    """A drum shell with chrome hoops and white heads: axis 'y' a snare or tom (tilted `tilt` about x), 'z'
    the bass drum lying towards the front."""
    r = dia / 2
    rot = (90, 0, 0) if axis == 'z' else (tilt, 0, 0)
    F.cyl(_name('shell'), r, depth, (x, y, z), M.shell, rot=rot, verts=24)
    F.cyl(_name('head'), r * 0.97, depth + 0.006, (x, y, z), M.head, rot=rot, verts=24)
    for s in (-1, 1):
        if axis == 'z':
            at = (x, y, z + s * depth / 2)
        else:
            at = (x, y + s * depth / 2 * G.cosd(tilt), z - s * depth / 2 * G.sind(tilt))
        G.torus(_name('hoop'), r + 0.008, 0.009, at=F.at(*at), material=M.chrome, rot=rot, major_segments=24,
                minor_segments=4)


def _cymbal_stand(F, x, z, h):
    F.cone(_name('stand_foot'), 0.16, 0.02, 0.05, (x, 0.025, z), M.chrome, verts=12)
    F.cyl(_name('stand'), 0.011, h - 0.02, (x, (h - 0.02) / 2, z), M.chrome, verts=8)


def _cymbal(F, name, x, y, z, d):
    """A cymbal centred on (x, y, z): a shallow cone with its bell and a felt nut."""
    G.revolve(name, [(0, 0.03), (0.04, 0.028), (0.055, 0.012), (d / 2, 0.0), (d / 2, -0.004), (0.055, 0.006),
                     (0.04, 0.022), (0, 0.024)], at=F.at(x, y, z), material=M.cymbal, segments=24)
    F.cyl(_name('felt'), 0.014, 0.025, (x, y + 0.04, z), M.dark, verts=8)


CRASH = (-0.5, 1.22, 0.22, 0.4)   # x, height, depth, diameter
RIDE = (0.66, 1.12, 0.18, 0.44)


def thunderDrums():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('rug', 1.88, 0.012, 0.9, (0, 0.006, 0), M.rug, bevel=0.004)
        _drum(F, 0.0, 0.29, 0.27, 0.52, 0.34, axis='z')                   # bass drum
        F.cyl('bass_logo', 0.1, 0.006, (0.0, 0.29, 0.443), M.flame, rot=(90, 0, 0), verts=20)
        for s in (-1, 1):                                                 # its spurs
            G.rod(_name('spur'), F.at(s * 0.22, 0.12, 0.35), F.at(s * 0.3, 0.0, 0.4), 0.008, material=M.chrome, verts=6)
        F.box('pedal', 0.08, 0.02, 0.2, (0.0, 0.02, 0.0), M.dark, rot=(-8, 0, 0))
        _drum(F, -0.15, 0.68, 0.24, 0.26, 0.18, tilt=-18)                 # rack toms
        _drum(F, 0.15, 0.68, 0.24, 0.28, 0.2, tilt=-18)
        G.rod('tom_mount', F.at(0.0, 0.56, 0.27), F.at(0.0, 0.66, 0.25), 0.014, material=M.chrome, verts=8)
        _drum(F, 0.42, 0.43, 0.12, 0.38, 0.34)                            # floor tom on legs
        for a in (30, 150, 270):
            G.rod(_name('tom_leg'), F.at(0.42 + 0.2 * G.cosd(a), 0.4, 0.12 + 0.2 * G.sind(a)),
                  F.at(0.42 + 0.24 * G.cosd(a), 0.0, 0.12 + 0.24 * G.sind(a)), 0.008, material=M.chrome, verts=6)
        _drum(F, -0.32, 0.6, 0.14, 0.34, 0.14, tilt=-6)                   # snare on its stand
        F.cyl('snare_stand', 0.01, 0.52, (-0.32, 0.27, 0.14), M.chrome, verts=8)
        F.cone('snare_foot', 0.14, 0.02, 0.04, (-0.32, 0.02, 0.14), M.chrome, verts=12)
        # the hi-hat, a pair of cymbals on its stand
        _cymbal_stand(F, -0.66, 0.06, 0.88)
        _cymbal(F, 'hihat_low', -0.66, 0.84, 0.06, 0.32)
        _cymbal(F, 'hihat_top', -0.66, 0.875, 0.06, 0.32)
        _cymbal_stand(F, CRASH[0], CRASH[2], CRASH[1])
        _cymbal_stand(F, RIDE[0], RIDE[2], RIDE[1])
        # the throne, top 0.47, and a pair of sticks on it
        F.cyl('throne_seat', 0.18, 0.07, (0, 0.435, 0.0), M.dark, verts=24, bevel=0.025, bevel_segments=3)
        F.cyl('throne_post', 0.022, 0.38, (0, 0.2, 0.0), M.chrome, verts=10)
        for a in (90, 210, 330):
            G.rod(_name('throne_leg'), F.at(0, 0.14, 0), F.at(0.22 * G.cosd(a), 0.01, 0.22 * G.sind(a)), 0.011,
                  material=M.chrome, verts=6)
        for dz, r in ((0.06, 20), (0.03, 30)):
            F.cyl(_name('stick'), 0.007, 0.4, (0.22, 0.48, dz), M.oak, rot=(90, 0, r), verts=6)
        # the monitor amp at the back
        F.box('amp', 0.34, 0.6, 0.3, (0.78, 0.3, -0.28), M.amp, bevel=0.025)
        F.cyl('amp_cone', 0.11, 0.012, (0.78, 0.36, -0.125), M.speaker, rot=(90, 0, 0), verts=20)
        F.cyl('amp_dust_cap', 0.045, 0.016, (0.78, 0.36, -0.122), M.fire, rot=(90, 0, 0), verts=14)
        F.box('amp_panel', 0.28, 0.06, 0.006, (0.78, 0.53, -0.128), M.speaker)
    G.join(still, 'body', origin=(0, 0, 0))
    # The crash and the ride, each tilted on its stand's tip: `cymbals` rocks both while the kit is played.
    moving = []
    for name, (x, y, z, dia), tilt in (('crash', CRASH, 14), ('ride', RIDE, 10)):
        with G.new_parts() as c:
            _cymbal(F, name, x, 0.0, 0.0, dia)
        part = G.join(c, name, origin=F.at(x, 0.0, 0.0))
        part.location = F.at(x, y, z)
        part.rotation_euler = (math.radians(tilt), 0, 0)   # tipped towards the room
        moving.append(part)
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.rock(moving[0], 'cymbals', 9, seconds=0.8, axis='X')
    G.rock(moving[1], 'cymbals', -6, seconds=0.8, axis='X')
    return objs


# --- the cool-down meditation cushion (1x1): a round mat, a zabuton and a tall zafu, a singing bowl ---------
def coolDownCushion():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.cyl('mat', 0.45, 0.012, (0, 0.006, 0), M.calm_light, verts=40)
        G.torus('mat_edge', 0.44, 0.009, at=F.at(0, 0.012, 0), material=M.calm_deep, major_segments=40,
                minor_segments=4)
        F.box('zabuton', 0.62, 0.07, 0.58, (0, 0.047, -0.02), M.oat, bevel=0.03, bevel_segments=3)
        # the singing bowl on its stand, the mallet beside it
        F.cyl('bowl_stand', 0.11, 0.03, (0.32, 0.027, 0.3), M.oak, verts=16, bevel=0.006)
        G.revolve('singing_bowl', [(0, 0), (0.05, 0), (0.09, 0.03), (0.1, 0.08), (0.092, 0.08), (0.082, 0.035),
                                   (0.045, 0.012), (0, 0.012)], at=F.at(0.32, 0.042, 0.3), material=M.brass,
                  segments=24)
        G.rod('mallet', F.at(0.2, 0.025, 0.3), F.at(0.06, 0.04, 0.39), 0.012, material=M.oak, verts=8)
        F.cyl('mallet_felt', 0.018, 0.04, (0.06, 0.04, 0.39), M.calm, rot=(90, 0, 58), verts=10)
        # a candle in its dish, a pot plant and an incense stick
        F.cyl('candle_dish', 0.065, 0.015, (-0.32, 0.0075, 0.3), M.brass, verts=16)
        F.cyl('candle', 0.04, 0.07, (-0.32, 0.05, 0.3), M.candle, verts=14)
        F.sphere('candle_flame', 0.012, (-0.32, 0.1, 0.3), M.candle_flame, scale=(1, 1, 2.2), segments=8, rings=6)
        F.cone('plant_pot', 0.05, 0.06, 0.1, (-0.3, 0.05, -0.32), M.pot, verts=14)
        for k, (dx, dz, r) in enumerate(((0, 0, 0.07), (0.05, 0.03, 0.05), (-0.04, 0.04, 0.05), (0.02, -0.05, 0.05))):
            F.sphere(_name('leaf_clump'), r, (-0.3 + dx, 0.15 + 0.03 * (k == 0), -0.32 + dz), M.leaf,
                     scale=(1, 1, 0.8), segments=10, rings=6)
        F.cyl('incense_holder', 0.03, 0.012, (0.3, 0.006, -0.3), M.brass, verts=10)
        G.rod('incense', F.at(0.3, 0.01, -0.3), F.at(0.33, 0.22, -0.33), 0.003, material=M.ember, verts=4)
    G.join(still, 'body', origin=(0, 0, 0))
    # The zafu, its band and pleats, origin on its underside: `sink_in` presses it to the seat height (0.40).
    with G.new_parts() as zafu:
        G.revolve('zafu', [(0, 0), (0.22, 0), (0.248, 0.02), (0.258, 0.08), (0.258, 0.26), (0.248, 0.33),
                           (0.22, 0.36), (0.12, 0.37), (0, 0.37)], at=F.at(0, 0.08, 0), material=M.zafu, segments=28)
        G.torus('zafu_band', 0.259, 0.014, at=F.at(0, 0.25, 0), material=M.calm_deep, major_segments=28,
                minor_segments=4)
        for a in range(0, 360, 45):
            F.box(_name('pleat'), 0.012, 0.24, 0.02, (0.255 * G.cosd(a), 0.25, 0.255 * G.sind(a)), M.calm_deep,
                  rot=(0, 0, -a))
        F.cyl('zafu_button', 0.025, 0.012, (0, 0.448, 0), M.calm_deep, verts=10)
    zafu = G.hinged(G.join(zafu, 'zafu'), F.at(0, 0.08, 0))
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.squash(zafu, 'sink_in', (1.03, 1.03, 0.86), seconds=0.6)
    return objs


# --- the grudge-to-gratitude writing desk (1x1): the writer sits in the middle, the desk before them -------
def grudgeJournal():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('top', 0.84, 0.03, 0.3, (0, 0.745, 0.3), M.body, bevel=0.008)
        for x in (-0.39, 0.39):
            for z in (0.18, 0.42):
                F.box(_name('leg'), 0.035, 0.73, 0.035, (x, 0.365, z), M.steel_dark, bevel=0.006)
            F.box(_name('side_rail'), 0.025, 0.03, 0.22, (x, 0.12, 0.3), M.steel_dark, bevel=0.004)
        F.box('back_rail', 0.76, 0.05, 0.02, (0, 0.69, 0.43), M.steel_dark, bevel=0.004)
        # the drawer's case, hung under the top, open towards the writer
        F.box('case_top', 0.36, 0.012, 0.28, (0.22, 0.724, 0.3), M.drawer)
        F.box('case_floor', 0.36, 0.012, 0.28, (0.22, 0.576, 0.3), M.drawer)
        for x in (0.045, 0.395):
            F.box(_name('case_side'), 0.012, 0.16, 0.28, (x, 0.65, 0.3), M.drawer)
        F.box('case_back', 0.36, 0.16, 0.012, (0.22, 0.65, 0.434), M.drawer)
        # the open journal, the pen, crumpled pages, a cactus and a red desk lamp
        F.box('journal_cover', 0.36, 0.008, 0.24, (-0.02, 0.764, 0.28), M.journal, bevel=0.003)
        for s in (-1, 1):
            F.box(_name('page'), 0.165, 0.012, 0.215, (-0.02 + s * 0.087, 0.774, 0.28), M.paper, rot=(0, 0, s * -2))
        F.cyl('pen', 0.006, 0.15, (0.17, 0.775, 0.24), M.dark, rot=(90, 0, 35), verts=6)
        for x, y, z, r in ((-0.3, 0.795, 0.22, 0.035), (-0.25, 0.79, 0.36, 0.03), (0.3, 0.035, 0.3, 0.035),
                           (-0.36, 0.02, 0.08, 0.03)):
            G.boulder(_name('crumple'), r, at=F.at(x, y - r, z), material=M.paper, seed=x * 10, noise=0.35,
                      subdivisions=1)
        G.revolve('bin', [(0, 0), (0.085, 0), (0.1, 0.26), (0.094, 0.26), (0.08, 0.008), (0, 0.008)],
                  at=F.at(-0.3, 0, 0.3), material=M.steel, segments=18)
        F.cone('cactus_pot', 0.045, 0.05, 0.08, (0.3, 0.8, 0.38), M.ember, verts=12)
        F.sphere('cactus', 0.035, (0.3, 0.89, 0.38), M.cactus, scale=(1, 1, 1.9), segments=10, rings=8)
        F.cyl('lamp_foot', 0.055, 0.015, (0.33, 0.768, 0.18), M.dark, verts=14)
        G.rod('lamp_arm', F.at(0.33, 0.77, 0.18), F.at(0.3, 1.0, 0.24), 0.008, material=M.dark, verts=6)
        G.rod('lamp_arm2', F.at(0.3, 1.0, 0.24), F.at(0.22, 0.98, 0.2), 0.008, material=M.dark, verts=6)
        G.revolve('lamp_shade', [(0.06, 0), (0.05, 0.02), (0.025, 0.08), (0, 0.085)], at=F.at(0.2, 0.92, 0.19),
                  material=M.fire, segments=16, rot=(25, 0, -20))
        # the chair, the writer's seat at 0.45
        F.box('chair_seat', 0.42, 0.04, 0.4, (0, 0.405, 0.0), M.body, bevel=0.012)
        F.box('chair_pad', 0.38, 0.035, 0.36, (0, 0.44, 0.01), M.calm, bevel=0.015, bevel_segments=3)
        F.box('chair_back', 0.4, 0.28, 0.05, (0, 0.74, -0.19), M.calm, rot=(-6, 0, 0), bevel=0.02, bevel_segments=2)
        for dx in (-0.18, 0.18):
            F.box(_name('front_leg'), 0.035, 0.39, 0.035, (dx, 0.195, 0.16), M.body, bevel=0.006)
            F.box(_name('back_leg'), 0.035, 0.9, 0.035, (dx, 0.45, -0.18), M.body, bevel=0.006)
    G.join(still, 'body', origin=(0, 0, 0))
    # The drawer: its front, tray and the letters in it; `drawer_open` slides it out towards the writer.
    with G.new_parts() as drawer:
        F.box('drawer_front', 0.34, 0.13, 0.016, (0.22, 0.65, 0.162), M.drawer, bevel=0.004)
        F.box('drawer_pull', 0.14, 0.014, 0.014, (0.22, 0.67, 0.148), M.steel, bevel=0.004)
        F.box('drawer_tray', 0.31, 0.01, 0.25, (0.22, 0.592, 0.29), M.drawer)
        for x in (0.07, 0.37):
            F.box(_name('drawer_side'), 0.008, 0.08, 0.25, (x, 0.63, 0.29), M.drawer)
        F.box('drawer_back', 0.31, 0.08, 0.008, (0.22, 0.63, 0.41), M.drawer)
        for k, (dx, dz, r) in enumerate(((-0.05, 0.25, 5), (0.04, 0.3, -8), (0.0, 0.34, 3))):
            F.box(_name('letter'), 0.16, 0.012, 0.11, (0.22 + dx, 0.603 + 0.012 * k, dz), M.paper, rot=(0, 0, r))
    drawer = G.join(drawer, 'drawer')
    objs = G.settle(centre=False)
    G.slide(drawer, 'drawer_open', (0, 0.17, 0), seconds=0.6)   # game -z: out towards the writer
    return objs


# --- the glacier cold-plunge tub (1x2): the bather sits in it at the middle; the chiller at the back --------
TUB = (-0.55, 0.45, 0.62)   # back depth, front depth, rim height


def coldPlunge():
    F = G.GameFrame()
    zb, zf, h = TUB
    zc, d = (zb + zf) / 2, zf - zb
    with G.new_parts() as still:
        F.box('plinth', 0.86, 0.06, d + 0.02, (0, 0.03, zc), M.hose, bevel=0.01)
        # the liner: walls round the water, so the bather sits down into it
        F.box('liner_floor', 0.72, 0.04, d - 0.12, (0, 0.08, zc), M.shell_white)
        for x in (-0.395, 0.395):
            F.box(_name('wall'), 0.05, h - 0.06, d - 0.02, (x, 0.06 + (h - 0.06) / 2, zc), M.shell_white)
        for z in (zb + 0.025, zf - 0.025):
            F.box(_name('wall'), 0.74, h - 0.06, 0.05, (0, 0.06 + (h - 0.06) / 2, z), M.shell_white)
        # cedar cladding, eight boards round the outside
        for i in range(8):
            y = 0.1 + i * 0.065
            F.box(_name('board_side'), 0.022, 0.058, d + 0.02, (-0.43 + 0.011, y, zc), M.cedar, bevel=0.004)
            F.box(_name('board_side'), 0.022, 0.058, d + 0.02, (0.43 - 0.011, y, zc), M.cedar, bevel=0.004)
            F.box(_name('board_front'), 0.86, 0.058, 0.022, (0, y, zf + 0.001), M.cedar, bevel=0.004)
            F.box(_name('board_back'), 0.86, 0.058, 0.022, (0, y, zb - 0.001), M.cedar, bevel=0.004)
        # the rims
        for x in (-0.39, 0.39):
            F.box(_name('rim'), 0.08, 0.03, d + 0.03, (x, h + 0.005, zc), M.shell_white, bevel=0.01)
        F.box('rim_front', 0.86, 0.03, 0.06, (0, h + 0.005, zf - 0.015), M.shell_white, bevel=0.01)
        # the icy water and floating ice
        F.box('water', 0.72, 0.012, d - 0.1, (0, h - 0.03, zc), M.water)
        for x, z, r in ((-0.2, 0.15, 10), (0.15, -0.3, 40), (0.22, 0.25, -20), (-0.1, -0.2, 70), (0.05, 0.05, 15),
                        (-0.25, -0.38, 30)):
            F.box(_name('ice'), 0.06, 0.045, 0.06, (x, h - 0.022, z), M.ice, rot=(8, 5, r), bevel=0.01)
        # the cover's roller housing on the back rim
        F.box('housing', 0.86, 0.1, 0.12, (0, h + 0.05, zb + 0.05), M.cedar, bevel=0.012)
        F.box('housing_lip', 0.8, 0.02, 0.01, (0, h + 0.03, zb + 0.112), M.dark)
        # the chiller behind: grille, display, hoses into the tub; a towel and a bottle on it
        F.box('chiller', 0.7, 0.6, 0.29, (0, 0.3, -0.78), M.drawer, bevel=0.025)
        F.box('grille', 0.6, 0.36, 0.008, (0, 0.28, -0.634), M.steel, bevel=0.004)
        for k in range(6):
            F.box(_name('grille_slot'), 0.56, 0.012, 0.004, (0, 0.14 + k * 0.055, -0.629), M.dark)
        F.box('display', 0.16, 0.07, 0.008, (0.2, 0.52, -0.632), M.screen, bevel=0.006)
        F.box('readout', 0.1, 0.025, 0.004, (0.2, 0.52, -0.627), M.readout)
        for x in (-0.22, 0.0):
            G.rod(_name('hose'), F.at(x, 0.5, -0.64), F.at(x, 0.5, zb - 0.01), 0.025, material=M.hose, verts=10)
        F.box('towel', 0.36, 0.04, 0.2, (0.17, 0.62, -0.79), M.towel, bevel=0.015)
        G.revolve('bottle', [(0, 0), (0.045, 0), (0.048, 0.006), (0.048, 0.17), (0.026, 0.21), (0.022, 0.24),
                             (0, 0.24)], at=F.at(-0.22, 0.6, -0.79), material=M.calm, segments=14)
        F.cyl('bottle_cap', 0.024, 0.02, (-0.22, 0.85, -0.79), M.dark, verts=10)
    G.join(still, 'body', origin=(0, 0, 0))
    # The slatted cover, rolled out over the water from its housing, origin at the housing: `cover_off`
    # rolls it back in.
    cz0 = zb + 0.11
    with G.new_parts() as cover:
        n = 14
        sl = (zf - 0.03 - cz0) / n
        for k in range(n):
            F.box(_name('slat'), 0.8, 0.024, sl - 0.004, (0, h + 0.03, cz0 + sl * (k + 0.5)), M.cover, bevel=0.006)
        F.box('cover_pull', 0.24, 0.025, 0.03, (0, h + 0.05, zf - 0.04), M.dark, bevel=0.01)
    cover = G.join(cover, 'cover', origin=F.at(0, h + 0.03, cz0))
    objs = G.settle(centre=False)
    G.squash(cover, 'cover_off', (1, 0.04, 1), seconds=0.9)
    return objs


FAMILY = G.Family(
    'pack_hotHeaded', M,
    [G.Model('heavyBag', heavyBag, size=(0.86, 2.01, 0.86), key='model.hotHeaded.heavyBag', centre=(0, 0.0),
             motions={'loops': {'pound': 'swing'}},
             paint=[('bag', 'Body')]),
     G.Model('smashBin', smashBin, size=(0.854, 0.643, 0.876), key='model.hotHeaded.smashBin', centre=(0.003, 0.008),
             motions={'uses': {'smash': 'lid_open'}, 'hold': 1},
             paint=[('crate', 'Body')]),
     G.Model('thunderDrums', thunderDrums, size=(1.89, 1.279, 0.9), key='model.hotHeaded.thunderDrums', centre=(0.005, 0.02),
             motions={'loops': {'drum': 'cymbals'}},
             paint=[('shell', 'Body')]),
     G.Model('coolDownCushion', coolDownCushion, size=(0.9, 0.454, 0.9), centre=(0, 0.02), key='model.hotHeaded.coolDownCushion',
             motions={'uses': {'countToTen': 'sink_in', 'breathe': 'sink_in'}},
             paint=[('zafu', 'Body')]),
     G.Model('grudgeJournal', grudgeJournal, size=(0.84, 1.018, 0.677), key='model.hotHeaded.grudgeJournal', centre=(0, 0.111),
             motions={'uses': {'writeItOut': 'drawer_open'}},
             paint=[('body', 'Body')]),
     G.Model('coldPlunge', coldPlunge, size=(0.86, 0.86, 1.39), key='model.hotHeaded.coldPlunge', centre=(0, -0.23),
             motions={'uses': {'plunge': 'cover_off'}},
             paint=[('cedar', 'Body')])],
    comment='Blender-built hot-headed pack models (tools/art/blender/families/pack_hotHeaded.py). Listed '
            'in the base manifest\'s packs. The materials map onto the game\'s finishes so a model costs '
            'one draw call per finish group. Each has its moving part: the bag swings while pounded, the '
            'crate\'s lid flips up, the cymbals rock while the kit is played, the zafu sinks, the desk '
            'drawer slides out and the plunge cover rolls back.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

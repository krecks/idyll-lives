# The kind pack (P9): the neighbourly baking counter, the community giving jar, the gift-knitting rocker, the
# welcome tea table, the songbird feeding station and the hearthside quilted settee.
# Notes: pack_kind.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`): the range's oven door drops (`oven_open`), a coin slides into the jar's slot (`coin`), the rocker
# rocks with its knitter on it (`rock`, a loop), the teapot pours (`pour`), the feeder's roof lifts for a refill
# (`lid_open`) and songbirds fly out to the tray and the bath (`bird`), and the settee's cushions sink (`sink_in`).
#
# Run:  cargo art pack_kind [--only givingJar] [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    body=('#6F8F6A', 'fabric'),      # the settee's sage velvet; its main part
    enamel=('#F1E6C8', 'gloss'),     # the range's buttercream enamel; the counter's main part
    cabinet=('#A8BFA0', 'satin'),    # the counter's sage cupboards
    cabinet_panel=('#B9CDB2', 'satin'),
    butcher=('#C79A62', 'satin'),    # the counter's top and the wall shelf, plain (the towel takes the textured draw)
    towel=('#E3A0A0', 'fabric'),     # the gingham tea towel on the range's rail
    towel_check=('#FBF6EC', 'fabric'),
    hob=('#4A4E54', 'satin'),
    grate=('#2B2C2E', 'satin'),
    oven_dark=('#2A2622', 'satin'),  # inside the oven
    oven_glass=('#4A3A30', 'gloss'),
    pedestal=('#C79A62', 'satin'),   # the giving jar's turned pedestal, plain (the jar takes the glass draw)
    glass=('#EEF4F4', 'glass'),
    cork=('#B5835A', 'satin'),
    ribbon=('#E3A0A0', 'satin'),
    rocker=('#8A5A36', 'wood'),      # the rocking chair
    cushion=('#A8BFA0', 'fabric'),   # the rocker's seat cushion, the poufs' tops
    throw=('#F1E6C8', 'fabric'),
    wicker=('#C9A16A', 'satin'),
    table=('#C79A62', 'satin'),      # the tea table, plain (the poufs and rug take the fabric draw)
    pouf=('#A8BFA0', 'fabric'),      # the left pouf
    pouf_rose=('#E3A0A0', 'fabric'),  # the right pouf
    rug=('#E8D6C0', 'fabric'),
    rug_edge=('#C0714C', 'fabric'),
    feeder=('#9CC3DB', 'satin'),     # the feeder house; the station's main part
    roof=('#C0714C', 'satin'),
    post=('#F2EFE8', 'satin'),
    tray=('#C79A62', 'satin'),
    stone=('#D8D3C8', 'matte'),      # the bird bath
    sack=('#C9B48C', 'satin'),
    quilt=('#F6EEDC', 'fabric'),     # the settee's quilt
    honey=('#C79A62', 'satin'),      # turned legs, the trolley
    porcelain=('#FBF6EC', 'gloss'),
    rose=('#E3A0A0', 'gloss'),       # tea set bands, the heart tag
    heart=('#D9483B', 'gloss'),
    brass=('#C9A54A', 'metal'),
    chrome=('#D0D3D6', 'chrome'),
    gold=('#D9B45A', 'metal'),       # coins
    note=('#7FA35B', 'matte'),
    chalk=('#3E5A48', 'matte'),
    chalk_line=('#F4F1EA', 'matte'),
    terracotta=('#C0714C', 'satin'),
    pot=('#F4F1EA', 'satin'),
    leaf=('#5E8F4E', 'satin'),
    leaf_light=('#7FA35B', 'satin'),
    mustard=('#D6A64A', 'satin'),
    rose_flower=('#E3A0A0', 'satin'),
    lilac=('#B48CD6', 'satin'),
    sky=('#9CC3DB', 'satin'),
    crust=('#E0B060', 'matte'),
    crust_dark=('#C98B3A', 'matte'),
    icing=('#F4C6D0', 'matte'),
    flour=('#F4EFE2', 'matte'),
    jar_red=('#C77D4F', 'gloss'),
    jar_gold=('#E8C46E', 'gloss'),
    jar_white=('#F4EFE2', 'gloss'),
    sage_pot=('#A8BFA0', 'gloss'),
    sage_dark=('#7E9A76', 'gloss'),
    candle=('#FBF6EC', 'satin'),
    flame=('#FFD27A', 'gloss'),
    water=('#BFE3EE', 'gloss'),
    seed=('#C9A16A', 'matte'),
    tin=('#B9BEC3', 'metal'),
    needle=('#C9CCD0', 'metal'),
    bird_brown=('#8A6A4E', 'satin'),
    bird_orange=('#E07A2E', 'satin'),
    bird_blue=('#4F7FB8', 'satin'),
    bird_cream=('#F4EFE2', 'satin'),
    bird_gold=('#C8A040', 'satin'),
    bird_yellow=('#F2D27A', 'satin'),
    bird_grey=('#9AA0A6', 'satin'),
    bird_pink=('#E8B4B8', 'satin'),
    dark=('#3A3D41', 'satin'),
    lantern=('#FFD9A0', 'gloss'),
    patch_a=('#E3A0A0', 'fabric'),
    patch_b=('#D6A64A', 'fabric'),
    patch_c=('#9CC3DB', 'fabric'),
    patch_d=('#A8BFA0', 'fabric'),
    pillow_a=('#E3A0A0', 'fabric'),
    pillow_b=('#F1E6C8', 'fabric'),
)

WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


def _teapot(F, x, y, z, s=1.0):
    """A teapot standing on `y`, spout to the right (+x game), `s` its scale."""
    G.revolve(_name('pot'), [(0, 0), (0.05 * s, 0), (0.075 * s, 0.03 * s), (0.08 * s, 0.06 * s), (0.068 * s, 0.1 * s),
                             (0.04 * s, 0.115 * s), (0, 0.115 * s)], at=F.at(x, y, z), material=M.porcelain, segments=20)
    G.torus(_name('pot_band'), 0.079 * s, 0.006 * s, at=F.at(x, y + 0.055 * s, z), material=M.rose, major_segments=20,
            minor_segments=4)
    F.sphere(_name('pot_knob'), 0.014 * s, (x, y + 0.125 * s, z), M.rose, segments=8, rings=6)
    G.rod(_name('spout'), F.at(x + 0.06 * s, y + 0.04 * s, z), F.at(x + 0.13 * s, y + 0.11 * s, z), 0.014 * s,
          material=M.porcelain, verts=8)
    G.torus(_name('pot_handle'), 0.035 * s, 0.008 * s, at=F.at(x - 0.085 * s, y + 0.065 * s, z), material=M.porcelain,
            rot=(90, 0, 0), major_segments=12, minor_segments=4)


def _cup(F, x, y, z):
    F.cyl(_name('saucer'), 0.05, 0.008, (x, y + 0.004, z), M.porcelain, verts=16)
    G.revolve(_name('cup'), [(0, 0), (0.022, 0), (0.038, 0.045), (0.034, 0.045), (0.02, 0.006), (0, 0.006)],
              at=F.at(x, y + 0.008, z), material=M.porcelain, segments=14)
    G.torus(_name('cup_band'), 0.036, 0.004, at=F.at(x, y + 0.045, z), material=M.rose, major_segments=14,
            minor_segments=4)


def _bush(F, x, y, z, r, flower):
    """A little flowering shrub: three satin leaf clumps and two flowers (plain, no foliage draw)."""
    for dx, dz, k in ((0, 0, 1.0), (0.05, 0.03, 0.75), (-0.05, -0.02, 0.8)):
        F.sphere(_name('clump'), r * k, (x + dx, y + r * k * 0.8, z + dz), M.leaf, scale=(1, 1, 0.85), segments=10,
                 rings=6)
    for dx, dz in ((0.03, 0.04), (-0.04, -0.03)):
        F.sphere(_name('flower'), 0.025, (x + dx, y + r * 1.45, z + dz), flower, segments=8, rings=5)


# --- the neighbourly baking counter (2x1): a cream retro range beside a sage counter laid with bakes --------
RANGE = (0.6, 0.62, 0.92, 0.6)   # x, width, hob height, depth


def neighbourlyOven():
    F = G.GameFrame()
    rx, rw, rh, rd = RANGE
    zf = WALL + rd                    # the range's front
    cx0, cx1 = -0.91, rx - rw / 2 - 0.01
    cw, cxm, cd = cx1 - cx0, (cx0 + cx1) / 2, 0.56
    with G.new_parts() as still:
        # the counter: plinth, carcass, three sage doors with brass knobs, a butcher-block top
        F.box('plinth', cw - 0.02, 0.08, cd - 0.05, (cxm, 0.04, WALL + (cd - 0.05) / 2), M.grate)
        F.box('carcass', cw, 0.78, cd, (cxm, 0.47, WALL + cd / 2), M.cabinet, bevel=0.008)
        for i in range(3):
            x = cx0 + cw / 6 * (2 * i + 1)
            F.box(_name('door'), cw / 3 - 0.03, 0.66, 0.016, (x, 0.45, WALL + cd + 0.008), M.cabinet, bevel=0.008)
            F.box(_name('door_panel'), cw / 3 - 0.1, 0.56, 0.008, (x, 0.45, WALL + cd + 0.018), M.cabinet_panel,
                  bevel=0.004)
            F.sphere(_name('knob'), 0.018, (x + (0.13 if i % 2 else -0.13), 0.68, WALL + cd + 0.03), M.brass,
                     segments=10, rings=6)
        F.box('worktop', cw + 0.01, 0.045, cd + 0.03, (cxm, 0.8825, WALL + (cd + 0.03) / 2), M.butcher, bevel=0.008)
        top = 0.905
        # the bakes: a lattice pie, a cooling rack of biscuits, a rolling pin, a pink cake on its stand
        G.revolve('pie_dish', [(0, 0), (0.12, 0), (0.16, 0.05), (0.15, 0.05), (0.11, 0.008), (0, 0.008)],
                  at=F.at(cxm - 0.28, top, -0.06), material=M.terracotta, segments=22)
        G.revolve('pie_crust', [(0, 0.055), (0.1, 0.05), (0.15, 0.04), (0.15, 0.03), (0, 0.03)],
                  at=F.at(cxm - 0.28, top, -0.06), material=M.crust, segments=22)
        for i in range(-2, 3):
            F.box(_name('lattice'), 0.02, 0.01, 0.26, (cxm - 0.28 + i * 0.05, top + 0.058, -0.06), M.crust_dark)
            F.box(_name('lattice'), 0.26, 0.01, 0.02, (cxm - 0.28, top + 0.062, -0.06 + i * 0.05), M.crust_dark)
        F.box('rack', 0.3, 0.012, 0.22, (cxm + 0.15, top + 0.015, -0.1), M.chrome)
        for x, z in ((0.07, -0.15), (0.15, -0.06), (0.23, -0.14), (0.1, -0.03), (0.22, -0.04), (0.05, -0.09)):
            F.cyl(_name('biscuit'), 0.03, 0.012, (cxm + x, top + 0.027, z), M.crust_dark, verts=12)
        F.cyl('rolling_pin', 0.025, 0.3, (cxm + 0.42, top + 0.025, 0.02), M.honey, rot=(0, 90, 0), verts=12)
        for s in (-1, 1):
            F.cyl(_name('pin_handle'), 0.012, 0.06, (cxm + 0.42 + s * 0.18, top + 0.025, 0.02), M.honey, rot=(0, 90, 0),
                  verts=8)
        F.box('flour_dust', 0.2, 0.002, 0.14, (cxm + 0.42, top + 0.001, -0.05), M.flour)
        G.revolve('cake_stand', [(0, 0), (0.05, 0), (0.02, 0.02), (0.02, 0.07), (0.12, 0.075), (0.12, 0.085), (0, 0.085)],
                  at=F.at(cxm - 0.42, top, -0.3), material=M.porcelain, segments=18)
        G.revolve('cake', [(0, 0), (0.09, 0), (0.09, 0.08), (0.075, 0.095), (0, 0.095)],
                  at=F.at(cxm - 0.42, top + 0.085, -0.3), material=M.icing, segments=18)
        # a beadboard back rising from the worktop's back (a dresser's), carrying the shelf with three jars: it stands
        # free or on the wall (2026-10-21: the shelf hung on the air over a free-standing counter)
        F.box('backboard', cw, 1.62 - top, 0.018, (cxm, (top + 1.62) / 2, WALL + 0.009), M.cabinet, bevel=0.004)
        for i in range(1, 8):
            F.box(_name('bead'), 0.006, 1.62 - top - 0.04, 0.004, (cx0 + i * cw / 8, (top + 1.62) / 2, WALL + 0.02),
                  M.cabinet_panel)
        F.box('backboard_cap', cw + 0.02, 0.025, 0.04, (cxm, 1.6325, WALL + 0.02), M.butcher, bevel=0.006)
        F.box('shelf', cw, 0.03, 0.2, (cxm, 1.5, WALL + 0.1), M.butcher, bevel=0.006)
        for x in (cx0 + 0.06, cx1 - 0.06):
            F.box(_name('bracket'), 0.03, 0.12, 0.16, (x, 1.44, WALL + 0.08), M.brass, bevel=0.005)
        for dx, h, m in ((-0.4, 0.16, M.jar_white), (-0.15, 0.2, M.jar_red), (0.1, 0.14, M.jar_gold)):
            G.revolve(_name('jar'), [(0, 0), (0.05, 0), (0.055, 0.01), (0.055, h - 0.03), (0.045, h - 0.02), (0, h - 0.02)],
                      at=F.at(cxm + dx, 1.515, WALL + 0.1), material=m, segments=16)
            F.cyl(_name('jar_lid'), 0.05, 0.025, (cxm + dx, 1.515 + h - 0.01, WALL + 0.1), M.cork, verts=16)
        # the range: enamel body, hob with four burners, backguard, knob panel, a chrome rail with a tea towel
        F.box('range', rw, rh - 0.04, rd - 0.03, (rx, (rh - 0.04) / 2, WALL + (rd - 0.03) / 2), M.enamel, bevel=0.02)
        F.box('kick', rw - 0.06, 0.06, 0.02, (rx, 0.03, zf - 0.035), M.hob)
        F.box('hob', rw + 0.004, 0.035, rd - 0.01, (rx, rh - 0.0175, WALL + (rd - 0.01) / 2), M.hob, bevel=0.006)
        F.box('backguard', rw, 0.16, 0.04, (rx, rh + 0.08, WALL + 0.02), M.enamel, bevel=0.012)
        for dx in (-0.15, 0.15):
            for dz in (-0.12, 0.12):
                bx, bz = rx + dx, WALL + rd / 2 + dz
                F.cyl(_name('burner'), 0.065, 0.012, (bx, rh + 0.006, bz), M.grate, verts=16)
                F.cyl(_name('burner_cap'), 0.035, 0.016, (bx, rh + 0.012, bz), M.chrome, verts=12)
                F.box(_name('grate'), 0.2, 0.012, 0.016, (bx, rh + 0.02, bz), M.grate)
                F.box(_name('grate'), 0.016, 0.012, 0.2, (bx, rh + 0.02, bz), M.grate)
        F.box('knob_panel', rw - 0.03, 0.1, 0.015, (rx, rh - 0.1, zf - 0.03 + 0.0075), M.enamel)
        for i in range(5):
            F.cyl(_name('range_knob'), 0.02, 0.025, (rx - 0.22 + i * 0.11, rh - 0.1, zf - 0.01), M.chrome, rot=(90, 0, 0),
                  verts=12)
        F.box('oven_mouth', rw - 0.1, 0.42, 0.006, (rx, 0.37, zf - 0.03 + 0.003), M.oven_dark)
        F.box('oven_rack', rw - 0.14, 0.01, 0.004, (rx, 0.3, zf - 0.03 + 0.007), M.chrome)
        G.rod('rail', F.at(rx - rw / 2 + 0.06, 0.7, zf + 0.01), F.at(rx + rw / 2 - 0.06, 0.7, zf + 0.01), 0.012,
              material=M.chrome, verts=10)
        for dx in (-(rw / 2 - 0.06), rw / 2 - 0.06):
            F.box(_name('rail_post'), 0.02, 0.02, 0.04, (rx + dx, 0.7, zf - 0.01), M.chrome)
        F.box('towel', 0.16, 0.24, 0.012, (rx + 0.1, 0.6, zf + 0.025), M.towel, bevel=0.004)
        for k in range(3):
            F.box(_name('towel_check'), 0.16, 0.025, 0.004, (rx + 0.1, 0.52 + k * 0.06, zf + 0.032), M.towel_check)
        G.revolve('stock_pot', [(0, 0), (0.11, 0), (0.11, 0.13), (0.104, 0.14), (0, 0.14)],
                  at=F.at(rx + 0.15, rh + 0.026, WALL + rd / 2 - 0.12), material=M.sage_pot, segments=20)
        F.cyl('stock_lid', 0.115, 0.02, (rx + 0.15, rh + 0.176, WALL + rd / 2 - 0.12), M.sage_dark, verts=20)
    G.join(still, 'body', origin=(0, 0, 0))
    # The oven door, hinged on its foot: `oven_open` drops it forward while a pie or treats bake.
    with G.new_parts() as door:
        F.box('oven_door', rw - 0.04, 0.5, 0.02, (rx, 0.38, zf - 0.02), M.enamel, bevel=0.01)
        F.box('oven_window', rw - 0.2, 0.24, 0.006, (rx, 0.36, zf - 0.008), M.oven_glass, bevel=0.012)
        F.box('door_handle', rw - 0.16, 0.022, 0.022, (rx, 0.58, zf + 0.012), M.chrome, bevel=0.008)
    door = G.hinged(G.join(door, 'oven_door'), F.at(rx, 0.13, zf - 0.02))
    objs = G.settle(centre=False)
    G.swing(door, 'oven_open', 85, seconds=0.8, axis='X')
    return objs


# --- the community giving jar (1x1): a glass jar of coins on a turned pedestal ----------------------------
JAR = (0.0, -0.05)   # x, depth of the jar's axis
LID_TOP = 1.28


def givingJar():
    F = G.GameFrame()
    jx, jz = JAR
    with G.new_parts() as still:
        G.revolve('pedestal', [(0, 0), (0.22, 0), (0.22, 0.04), (0.13, 0.09), (0.07, 0.17), (0.053, 0.5), (0.07, 0.75),
                               (0.17, 0.8), (0.185, 0.83), (0.18, 0.86), (0, 0.86)], at=F.at(jx, 0, jz),
                  material=M.pedestal, segments=24)
        # coins and notes heaped in the jar
        for i in range(14):
            a = i * 2.4
            rr = 0.03 + 0.07 * ((i * 7) % 5) / 5
            F.cyl(_name('coin'), 0.022, 0.006, (jx + rr * math.cos(a), 0.872 + 0.012 * (i // 4), jz + rr * math.sin(a)),
                  M.gold, rot=((i * 23) % 30, (i * 17) % 25, 0), verts=12)
        F.box('notes', 0.12, 0.02, 0.07, (jx - 0.02, 0.9, jz + 0.02), M.note, rot=(0, 8, 25))
        # the cork lid and its slot, a ribbon round the neck, a heart tag
        F.cyl('cork', 0.105, 0.05, (jx, LID_TOP - 0.025, jz), M.cork, verts=20, bevel=0.006)
        F.box('slot', 0.07, 0.004, 0.012, (jx, LID_TOP + 0.001, jz), M.dark)
        G.torus('ribbon', 0.112, 0.012, at=F.at(jx, 1.2, jz), material=M.ribbon, major_segments=24, minor_segments=4)
        G.rod('tag_string', F.at(jx, 1.19, jz + 0.11), F.at(jx, 1.12, jz + 0.15), 0.003, material=M.ribbon, verts=4)
        for s in (-1, 1):
            F.sphere(_name('heart_lobe'), 0.033, (jx + s * 0.024, 1.08, jz + 0.155), M.heart, scale=(1, 0.45, 1),
                     segments=12, rings=8)
        F.box('heart_point', 0.065, 0.065, 0.028, (jx, 1.055, jz + 0.155), M.heart, rot=(0, 45, 0), bevel=0.01)
        # a chalk sign on the pedestal and a flowering pot plant at its foot
        F.box('sign_frame', 0.24, 0.18, 0.012, (jx, 0.62, jz + 0.075), M.pedestal, bevel=0.006)
        F.box('sign', 0.21, 0.15, 0.006, (jx, 0.62, jz + 0.083), M.chalk)
        for y, w in ((0.65, 0.14), (0.6, 0.1)):
            F.box(_name('chalk_line'), w, 0.014, 0.003, (jx, y, jz + 0.087), M.chalk_line)
        G.revolve('plant_pot', [(0, 0), (0.06, 0), (0.08, 0.16), (0, 0.16)], at=F.at(0.3, 0, 0.28),
                  material=M.terracotta, segments=16)
        _bush(F, 0.3, 0.15, 0.28, 0.1, M.mustard)
        # the glass jar over the coins (the glass draw)
        G.revolve('jar', [(0, 0), (0.138, 0), (0.15, 0.076), (0.15, 0.285), (0.108, 0.334), (0.104, 0.36), (0, 0.36)],
                  at=F.at(jx, 0.86, jz), material=M.glass, segments=28)
    G.join(still, 'body', origin=(0, 0, 0))
    # A coin standing in the slot: `coin` drops it through into the jar.
    with G.new_parts() as coin:
        F.cyl('donation', 0.024, 0.005, (jx, LID_TOP + 0.016, jz), M.gold, rot=(90, 0, 0), verts=16)
    coin = G.join(coin, 'coin')
    objs = G.settle(centre=False)
    G.slide(coin, 'coin', (0, 0, -0.1), seconds=0.6)
    return objs


# --- the gift-knitting rocker (1x1): a spindle-back rocking chair, its knitter rocking on it ----------------
ROCK_R = 1.1      # the rockers' radius
CZ = -0.02        # the chair's middle (depth)


def knittingRocker():
    F = G.GameFrame()
    with G.new_parts() as still:
        # the yarn basket beside it, balls of wool, needles in the work
        G.revolve('basket', [(0, 0), (0.08, 0), (0.1, 0.18), (0.09, 0.18), (0.072, 0.016), (0, 0.016)],
                  at=F.at(0.35, 0, 0.34), material=M.wicker, segments=18)
        for k in range(3):
            G.torus(_name('weave'), 0.085 + 0.006 * k, 0.005, at=F.at(0.35, 0.04 + 0.05 * k, 0.34), material=M.honey,
                    major_segments=18, minor_segments=4)
        for x, y, z, r, m in ((0.32, 0.18, 0.32, 0.045, M.rose_flower), (0.39, 0.17, 0.36, 0.04, M.sky),
                              (0.35, 0.2, 0.39, 0.04, M.mustard)):
            F.sphere(_name('yarn'), r, (x, y, z), m, segments=12, rings=8)
        G.rod('needle_a', F.at(0.31, 0.15, 0.37), F.at(0.41, 0.32, 0.29), 0.005, material=M.needle, verts=5)
        G.rod('needle_b', F.at(0.33, 0.15, 0.39), F.at(0.38, 0.33, 0.31), 0.005, material=M.needle, verts=5)
        F.box('knitting', 0.12, 0.012, 0.09, (0.36, 0.2, 0.33), M.rose_flower, rot=(10, 0, 20))
    G.join(still, 'body', origin=(0, 0, 0))
    # The chair, its cushion and the throw over its back: one part turning about its rockers' centre, so they
    # roll on the floor; `rock` rocks it with its knitter (`ride`).
    w = 0.6
    with G.new_parts() as chair:
        F.box('seat_board', w - 0.1, 0.035, 0.44, (0, 0.42, CZ), M.rocker, bevel=0.01)
        for s in (-1, 1):
            x = s * (w / 2 - 0.07)
            n = 10
            for i in range(n):                           # the rocker: flat boxes along its arc
                a = -21 + (i + 0.5) * 42 / n
                F.box(_name('rocker'), 0.03, 0.04, ROCK_R * math.radians(42 / n) + 0.012,
                      (x, ROCK_R + 0.045 - ROCK_R * G.cosd(a), CZ + ROCK_R * G.sind(a)), M.rocker, rot=(-a, 0, 0))
            G.rod(_name('front_leg'), F.at(x, 0.04, CZ + 0.2), F.at(x, 0.42, CZ + 0.18), 0.018, material=M.rocker, verts=8)
            G.rod(_name('back_leg'), F.at(x, 0.035, CZ - 0.2), F.at(x, 0.42, CZ - 0.19), 0.018, material=M.rocker, verts=8)
            G.rod(_name('stile'), F.at(x, 0.42, CZ - 0.19), F.at(x * 1.02, 1.0, CZ - 0.32), 0.02, material=M.rocker,
                  verts=8)
            F.sphere(_name('finial'), 0.026, (x * 1.02, 1.02, CZ - 0.325), M.rocker, segments=10, rings=6)
            F.box(_name('arm'), 0.055, 0.03, 0.44, (x * 1.06, 0.66, CZ - 0.02), M.rocker, bevel=0.01)
            G.rod(_name('arm_post'), F.at(x * 1.06, 0.42, CZ + 0.17), F.at(x * 1.06, 0.66, CZ + 0.17), 0.015,
                  material=M.rocker, verts=8)
            G.rod(_name('side_stretcher'), F.at(x, 0.15, CZ - 0.19), F.at(x, 0.15, CZ + 0.19), 0.011, material=M.rocker,
                  verts=6)
        for i in range(6):
            xx = -0.15 + i * 0.06
            G.rod(_name('spindle'), F.at(xx, 0.44, CZ - 0.18), F.at(xx, 0.94, CZ - 0.3), 0.01, material=M.rocker,
                  verts=6)
        F.box('crest', w - 0.08, 0.08, 0.04, (0, 0.97, CZ - 0.31), M.rocker, rot=(12, 0, 0), bevel=0.012)
        G.rod('stretcher', F.at(-w / 2 + 0.07, 0.15, CZ), F.at(w / 2 - 0.07, 0.15, CZ), 0.011, material=M.rocker,
              verts=6)
        F.box('seat_cushion', 0.4, 0.05, 0.38, (0, 0.465, CZ), M.cushion, bevel=0.02, bevel_segments=3)
        F.box('throw', 0.36, 0.42, 0.03, (0.02, 0.8, -0.245), M.throw, rot=(-14, 4, 0), bevel=0.01)
        F.box('throw_fold', 0.36, 0.03, 0.08, (0.02, 1.0, -0.3), M.throw, rot=(-14, 4, 0), bevel=0.012)
    chair = G.hinged(G.join(chair, 'chair'), F.at(0, ROCK_R + 0.025, CZ))
    objs = G.settle(centre=False)
    G.rock(chair, 'rock', 5, seconds=1.6, axis='X')
    return objs


# --- the welcome tea table (2x1): two poufs (tops 0.45 at x = ±0.75) facing across a low table with the tea set --
POUF_X = 0.75                                   # content `places` [±0.75, 0], facing ±90


def welcomeTeaTable():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.cyl('rug', 0.97, 0.012, (0, 0.006, 0), M.rug, verts=48, scale=(1, 0.44, 1))
        edge = G.torus('rug_edge', 0.94, 0.012, at=F.at(0, 0.013, 0), material=M.rug_edge, major_segments=48,
                       minor_segments=4)
        edge.scale = (1, 0.44, 1)
        for x, m in ((-POUF_X, M.pouf), (POUF_X, M.pouf_rose)):
            G.revolve(_name('pouf'), [(0, 0), (0.18, 0), (0.2, 0.03), (0.2, 0.33), (0.18, 0.37), (0, 0.37)],
                      at=F.at(x, 0.0, 0.0), material=m, segments=24)
            G.revolve(_name('pouf_top'), [(0, 0.37), (0.19, 0.37), (0.205, 0.39), (0.195, 0.43), (0.15, 0.45), (0, 0.45)],
                      at=F.at(x, 0.0, 0.0), material=M.cushion, segments=24)
            G.torus(_name('piping'), 0.202, 0.01, at=F.at(x, 0.37, 0.0), material=M.rug, major_segments=24,
                    minor_segments=4)
        # the low table between them (top 0.36, 0.4 across: their knees stop short of its ends, their feet before
        # them), a shelf of books, a tea light
        F.box('table_top', 0.4, 0.04, 0.56, (0, 0.34, -0.02), M.table, bevel=0.012)
        for x in (-0.16, 0.16):
            for z in (-0.25, 0.21):
                F.box(_name('table_leg'), 0.04, 0.32, 0.04, (x, 0.16, z), M.table, bevel=0.006)
        F.box('table_shelf', 0.32, 0.02, 0.48, (0, 0.1, -0.02), M.table, bevel=0.005)
        F.box('book_a', 0.16, 0.035, 0.12, (-0.04, 0.128, -0.1), M.mustard, bevel=0.004)
        F.box('book_b', 0.15, 0.03, 0.11, (-0.04, 0.16, -0.1), M.sky, bevel=0.004, rot=(0, 0, 8))
        F.cyl('tea_light', 0.035, 0.04, (0.1, 0.38, -0.22), M.candle, verts=12)
        F.sphere('tea_light_flame', 0.01, (0.1, 0.415, -0.22), M.flame, scale=(1, 1, 2), segments=8, rings=5)
        _cup(F, -0.1, 0.36, 0.14)      # a cup before each of them
        _cup(F, 0.1, 0.36, 0.1)
        G.revolve('milk_jug', [(0, 0), (0.025, 0), (0.03, 0.045), (0.022, 0.065), (0.026, 0.07), (0, 0.062)],
                  at=F.at(0.11, 0.36, -0.1), material=M.porcelain, segments=14)
        # potted ferns at the back, between the poufs and the table
        for x in (-0.44, 0.44):
            G.revolve(_name('fern_pot'), [(0, 0), (0.07, 0), (0.09, 0.2), (0, 0.2)], at=F.at(x, 0, -0.33),
                      material=M.pot, segments=16)
            for k in range(7):
                a = k * 360 / 7
                G.rod(_name('frond'), F.at(x, 0.19, -0.33),
                      F.at(x + 0.08 * G.cosd(a), 0.34 + 0.04 * (k % 2), -0.33 + 0.09 * G.sind(a)), 0.022,
                      material=M.leaf if k % 2 else M.leaf_light, verts=4)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as pot:
        _teapot(F, -0.07, 0.36, -0.06)
    # `pour` tips it towards its spout about the foot's edge under the spout, over the cups.
    pot = G.hinged(G.join(pot, 'teapot'), F.at(-0.02, 0.36, -0.06))
    objs = G.settle(centre=False)
    G.swing(pot, 'pour', -30, seconds=0.8, axis='Y')
    return objs


# --- the songbird feeding station (1x1): a feeder house on a post, a bird bath, songbirds -------------------
POST = (-0.2, -0.22)    # x, depth
HOUSE_Y = 1.48          # the feeder house's middle


def _bird(F, x, y, z, body, breast, yaw=0.0):
    """A songbird perched on `y`, facing `yaw` degrees from the front (+z) towards +x."""
    fx, fz = G.sind(yaw), G.cosd(yaw)
    F.sphere(_name('bird_body'), 0.05, (x, y + 0.035, z), body, scale=(0.7, 1.0, 0.65), rot=(0, 0, -yaw), segments=10,
             rings=6)
    F.sphere(_name('bird_breast'), 0.024, (x + fx * 0.022, y + 0.03, z + fz * 0.022), breast, segments=8, rings=6)
    F.sphere(_name('bird_head'), 0.025, (x + fx * 0.045, y + 0.085, z + fz * 0.045), body, segments=8, rings=6)
    G.rod(_name('beak'), F.at(x + fx * 0.065, y + 0.085, z + fz * 0.065), F.at(x + fx * 0.09, y + 0.083, z + fz * 0.09),
          0.007, material=M.mustard, verts=5)
    G.rod(_name('tail'), F.at(x - fx * 0.04, y + 0.04, z - fz * 0.04), F.at(x - fx * 0.085, y + 0.06, z - fz * 0.085),
          0.012, material=body, verts=5)


def songbirdFeeder():
    F = G.GameFrame()
    px, pz = POST
    with G.new_parts() as still:
        F.box('post_foot', 0.3, 0.04, 0.3, (px, 0.02, pz), M.stone, bevel=0.01)
        F.box('post', 0.07, 1.34, 0.07, (px, 0.71, pz), M.post, bevel=0.01)
        F.box('seed_tray', 0.38, 0.025, 0.32, (px, 1.37, pz), M.tray, bevel=0.006)
        for dz in (-0.155, 0.155):
            F.box(_name('tray_lip'), 0.38, 0.03, 0.012, (px, 1.39, pz + dz), M.tray)
        for x, z in ((-0.31, -0.12), (-0.12, -0.3), (-0.27, -0.31), (-0.13, -0.13), (-0.3, -0.33)):
            F.sphere(_name('seeds'), 0.025, (x, 1.387, z), M.seed, scale=(1, 1, 0.35), segments=8, rings=4)
        F.box('house', 0.22, 0.2, 0.18, (px, HOUSE_Y, pz), M.feeder, bevel=0.01)
        F.box('house_hopper', 0.16, 0.06, 0.004, (px, 1.41, pz + 0.091), M.water)
        F.cyl('house_hole', 0.035, 0.006, (px, 1.5, pz + 0.091), M.dark, rot=(90, 0, 0), verts=14)
        G.rod('perch', F.at(px, 1.43, pz + 0.09), F.at(px, 1.43, pz + 0.16), 0.008, material=M.tray, verts=6)
        _bird(F, px, 1.43, pz + 0.155, M.bird_brown, M.bird_orange, 0)
        # the bird bath
        G.revolve('bath_stand', [(0, 0), (0.12, 0), (0.12, 0.02), (0.06, 0.05), (0.045, 0.31), (0.055, 0.55), (0.1, 0.62),
                                 (0, 0.62)], at=F.at(0.2, 0, 0.16), material=M.stone, segments=20)
        G.revolve('bath_bowl', [(0, 0.6), (0.15, 0.6), (0.25, 0.68), (0.25, 0.71), (0.22, 0.71), (0.2, 0.69), (0, 0.67)],
                  at=F.at(0.2, 0, 0.16), material=M.stone, segments=28)
        F.cyl('bath_water', 0.205, 0.008, (0.2, 0.69, 0.16), M.water, verts=28)
        # flowering shrubs, a seed sack and a scoop tin
        for x, z, m in ((-0.34, 0.02, M.rose_flower), (-0.05, -0.36, M.mustard), (0.34, -0.3, M.lilac),
                        (-0.34, 0.33, M.rose_flower)):
            _bush(F, x, 0.0, z, 0.08, m)
        F.box('sack', 0.16, 0.22, 0.1, (0.08, 0.11, WALL + 0.06), M.sack, bevel=0.03, bevel_segments=2)
        F.box('sack_fold', 0.17, 0.03, 0.105, (0.08, 0.225, WALL + 0.06), M.sack, bevel=0.012)
        G.revolve('scoop_tin', [(0, 0), (0.05, 0), (0.06, 0.1), (0.056, 0.1), (0.046, 0.006), (0, 0.006)],
                  at=F.at(-0.02, 0, 0.38), material=M.tin, segments=16)
    G.join(still, 'body', origin=(0, 0, 0))
    # The roof, its ridge left to right, hinged on its back eave: `lid_open` lifts it for a refill.
    eave_y, eave_z = HOUSE_Y + 0.1, pz - 0.115
    with G.new_parts() as roof:
        for s in (-1, 1):
            F.box(_name('roof_slab'), 0.28, 0.02, 0.15, (px, HOUSE_Y + 0.15, pz + s * 0.06), M.roof, rot=(s * 36, 0, 0),
                  bevel=0.005)
        F.cyl('ridge', 0.014, 0.28, (px, HOUSE_Y + 0.2, pz), M.roof, rot=(0, 90, 0), verts=8)
    roof = G.hinged(G.join(roof, 'roof'), F.at(px, eave_y, eave_z))
    # The songbirds that come while residents watch: built where they perch, joined with their origin in the
    # feeder house and shrunk into it; `bird` grows them out to the tray and the bath.
    with G.new_parts() as birds:
        _bird(F, -0.34, 1.385, -0.17, M.bird_blue, M.bird_cream, -60)
        _bird(F, 0.36, 0.71, 0.2, M.bird_gold, M.bird_yellow, 70)
        _bird(F, 0.02, 0.71, 0.1, M.bird_grey, M.bird_pink, -80)
    birds = G.join(birds, 'birds', origin=F.at(px, HOUSE_Y, pz))
    birds.scale = (0.001, 0.001, 0.001)
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.swing(roof, 'lid_open', -70, seconds=0.7, axis='X')
    for axis in 'XYZ':
        G.grow(birds, 'bird', seconds=1.0, axis=axis)
    return objs


# --- the hearthside quilted settee (2x1): two seats at x = ±0.5 (tops 0.45), a patchwork quilt -------------
def hearthSettee():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('base', 1.66, 0.28, 0.78, (0, 0.22, -0.03), M.body, bevel=0.05, bevel_segments=3)
        F.box('back', 1.66, 0.5, 0.2, (0, 0.62, -0.33), M.body, bevel=0.07, bevel_segments=3)
        for x in (-0.4, 0.4):
            F.box(_name('back_cushion'), 0.72, 0.36, 0.14, (x, 0.7, -0.2), M.body, rot=(-10, 0, 0), bevel=0.05,
                  bevel_segments=3)
        for s in (-1, 1):
            F.box(_name('arm'), 0.16, 0.36, 0.78, (s * 0.86, 0.34, -0.03), M.body, bevel=0.05, bevel_segments=2)
            F.cyl(_name('arm_roll'), 0.11, 0.8, (s * 0.84, 0.56, -0.03), M.body, rot=(90, 0, 0), verts=20)
            F.cyl(_name('arm_face'), 0.09, 0.012, (s * 0.84, 0.56, 0.372), M.body, rot=(90, 0, 0), verts=20)
            for z in (-0.36, 0.3):
                G.revolve(_name('leg'), [(0, 0), (0.021, 0), (0.035, 0.05), (0.028, 0.1), (0, 0.1)],
                          at=F.at(s * 0.82, 0, z), material=M.honey, segments=12)
        F.box('pillow_l', 0.34, 0.3, 0.12, (-0.62, 0.66, -0.12), M.pillow_a, rot=(-12, 0, 18), bevel=0.05,
              bevel_segments=3)
        F.box('pillow_r', 0.3, 0.28, 0.12, (0.66, 0.66, -0.12), M.pillow_b, rot=(-12, 0, -18), bevel=0.05,
              bevel_segments=3)
        # a brass lantern on the left arm
        F.box('lantern_base', 0.14, 0.02, 0.14, (-0.86, 0.68, 0.12), M.brass, bevel=0.005)
        F.box('lantern_light', 0.11, 0.15, 0.11, (-0.86, 0.765, 0.12), M.lantern, bevel=0.012)
        for dx in (-0.055, 0.055):
            for dz in (-0.055, 0.055):
                F.box(_name('lantern_post'), 0.012, 0.16, 0.012, (-0.86 + dx, 0.765, 0.12 + dz), M.brass)
        F.cone('lantern_cap', 0.085, 0.03, 0.06, (-0.86, 0.87, 0.12), M.brass, verts=4, rot=(0, 0, 45))
        G.torus('lantern_ring', 0.025, 0.005, at=F.at(-0.86, 0.915, 0.12), material=M.brass, rot=(90, 0, 0),
                major_segments=12, minor_segments=4)
    G.join(still, 'body', origin=(0, 0, 0))
    # The seat cushions and the quilt over the right one, origin on their underside: `sink_in` presses them.
    with G.new_parts() as seats:
        for x in (-0.41, 0.41):
            F.box(_name('seat_cushion'), 0.8, 0.12, 0.62, (x, 0.39, 0.06), M.body, bevel=0.045,
                  bevel_segments=3)
        F.box('quilt_top', 0.76, 0.02, 0.58, (0.42, 0.46, 0.08), M.quilt, bevel=0.006)
        F.box('quilt_drop', 0.76, 0.26, 0.02, (0.42, 0.34, 0.385), M.quilt, bevel=0.006)
        patches = (M.patch_a, M.quilt, M.patch_b, M.patch_c, M.patch_d, M.pillow_b)
        for i in range(4):
            for j in range(3):
                F.box(_name('patch'), 0.17, 0.004, 0.17, (0.15 + i * 0.18, 0.471, -0.1 + j * 0.18),
                      patches[(i + 2 * j) % 6])
            F.box(_name('patch_drop'), 0.17, 0.11, 0.004, (0.15 + i * 0.18, 0.4, 0.396), patches[(i + 3) % 6])
    seats = G.hinged(G.join(seats, 'seats'), F.at(0, 0.33, 0.06))
    objs = G.settle(centre=False)
    G.squash(seats, 'sink_in', (1, 1, 0.84), seconds=0.6)
    return objs


FAMILY = G.Family(
    'pack_kind', M,
    [G.Model('neighbourlyOven', neighbourlyOven, size=(1.832, 1.718, 0.634), key='model.kind.neighbourlyOven',
             # the door drops while the dish goes in, then shuts for the bake (`hold`). Keyed by the bake step's
             # tag, not the uses: the recipes' mixing is done at its worktop too, where the oven stays shut.
             centre=(-0.004, -0.113), motions={'uses': {'bake': 'oven_open'}, 'hold': 3.0},
             paint=[('enamel', 'Body'), ('towel', 'Fabric'), ('cabinet', 'Cupboards')]),
     G.Model('givingJar', givingJar, size=(0.641, 1.32, 0.655), key='model.kind.givingJar', centre=(0.101, 0.057),
             motions={'uses': {'donate': 'coin'}},
             paint=[('pedestal', 'Body'), ('ribbon', 'Fabric')]),
     G.Model('knittingRocker', knittingRocker, size=(0.723, 1.046, 0.867), key='model.kind.knittingRocker',
             centre=(0.09, 0.007), motions={'loops': {'knitGift': 'rock', 'rock': 'rock'}, 'ride': 'rock'},
             paint=[('rocker', 'Body'), ('cushion', 'Fabric')]),
     G.Model('welcomeTeaTable', welcomeTeaTable, size=(1.94, 0.499, 0.864), key='model.kind.welcomeTeaTable',
             centre=(0, -0.005),
             motions={'uses': {'shareTea': 'pour'}, 'hold': 1.5},
             paint=[('table', 'Body'), ('pouf', 'Fabric'), ('pouf_rose', 'Second pouf'), ('rug', 'Rug')]),
     G.Model('songbirdFeeder', songbirdFeeder, size=(0.901, 1.698, 0.884), key='model.kind.songbirdFeeder',
             centre=(0, 0.012), motions={'uses': {'refill': 'lid_open', 'watchBirds': 'bird'}},
             paint=[('feeder', 'Body'), ('sack', 'Fabric'), ('roof', 'Roof')]),
     G.Model('hearthSettee', hearthSettee, size=(1.9, 0.945, 0.828), key='model.kind.hearthSettee',
             centre=(0, -0.016), motions={'uses': {'curlUp': 'sink_in', 'quiltNap': 'sink_in'}},
             paint=[('body', 'Body'), ('quilt', 'Fabric')])],
    comment='Blender-built kind pack models (tools/art/blender/families/pack_kind.py). Listed in the base '
            'manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw call '
            'per finish group. Each moves while it is used: the oven door drops, a coin drops into the '
            'jar, the rocker rocks, the teapot pours, the feeder\'s roof lifts and songbirds fly out, and '
            'the settee\'s cushions sink.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

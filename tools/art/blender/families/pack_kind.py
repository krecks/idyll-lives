# The kind pack (P9): the neighbourly baking counter, the community giving jar, the gift-knitting rocker, the
# welcome tea table, the songbird feeding station and the hearthside quilted settee.
# Notes: pack_kind.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`): the range's oven door drops (`oven_open`), a coin slides into the jar's slot (`coin`), the rocker
# rocks with its knitter on it (`rock`, a loop), the teapot pours (`pour`), the feeder's roof lifts for a refill
# (`lid_open`) and songbirds fly out to the tray and the bath (`bird`), and the settee's cushions sink (`sink_in`).
#
# Built with the forms and the kit (2026-10-22): shaker cupboard doors and turned knobs, the range's grates and
# rail, a lofted teapot with a swept spout and handle, cable-knit poufs, a rocker on curved runners, the kit's
# songbirds, rolled arms and soft cushions, a strip quilt draped over a seat.
#
# Run:  cargo art pack_kind [--only givingJar] [--preview]
import math
import os
import random
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
    soil=('#5C4433', 'matte'),       # the potted plants' soil
)

WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


def _turned(F, name, x, y, z, profile, material, segments=16, rot=(0, 0, 0)):
    """A turned piece standing at game (x, y, z): `profile` [(radius, height up), ...] from its foot, revolved."""
    return G.revolve(name, profile, at=F.at(x, y, z), material=material, segments=segments, rot=rot)


def _lathe(F, name, x, y, z, rings, material, verts=16, square=2.0):
    """A body lofted through round rings [(radius, height up), ...] standing at game (x, y, z)."""
    bx, by, _ = F.at(x, 0, z)
    return G.loft(name, [G.ring(r, r, y + h, verts, square=square, at=(bx, by)) for r, h in rings], material=material)


def _fluted(r, y, verts, at, flutes, depth, sy=1.0):
    """A ring of Blender points of radius `r` at height `y`, in and out `depth` (a fraction) `flutes` times round."""
    return [(at[0] + r * (1 + depth * math.cos(flutes * math.tau * k / verts)) * math.cos(math.tau * k / verts),
             at[1] + sy * r * (1 + depth * math.cos(flutes * math.tau * k / verts)) * math.sin(math.tau * k / verts), y)
            for k in range(verts)]


def _teapot(F, x, y, z, s=1.0):
    """A teapot standing on `y`, spout to the right (+x game), `s` its scale: a round body with a band, a lid with a
    knob, a curving spout and a looped handle."""
    _lathe(F, _name('pot'), x, y, z, [(0.045 * s, 0), (0.06 * s, 0.008 * s), (0.077 * s, 0.035 * s), (0.08 * s, 0.06 * s),
                                      (0.07 * s, 0.092 * s), (0.05 * s, 0.108 * s), (0.04 * s, 0.11 * s)], M.porcelain, verts=18)
    _turned(F, _name('pot_lid'), x, y + 0.108 * s, z, [(0, 0), (0.042 * s, 0), (0.042 * s, 0.004 * s), (0.03 * s, 0.012 * s),
                                                       (0.008 * s, 0.014 * s), (0, 0.014 * s)], M.porcelain, 18)
    F.sphere(_name('pot_knob'), 0.012 * s, (x, y + 0.13 * s, z), M.rose, scale=(1, 1, 0.85), segments=8, rings=5)
    G.torus(_name('pot_band'), 0.079 * s, 0.005 * s, at=F.at(x, y + 0.055 * s, z), material=M.rose, major_segments=18,
            minor_segments=3)
    F.sweep(_name('spout'), [(x + 0.06 * s, y + 0.035 * s, z), (x + 0.1 * s, y + 0.06 * s, z), (x + 0.12 * s, y + 0.1 * s, z),
                             (x + 0.135 * s, y + 0.115 * s, z)], 0.016 * s, M.porcelain, verts=8, resolution=2,
            scale=[1.0, 0.7, 0.5, 0.45])
    F.sweep(_name('pot_handle'), [(x - 0.065 * s, y + 0.095 * s, z), (x - 0.11 * s, y + 0.1 * s, z), (x - 0.125 * s, y + 0.065 * s, z),
                                  (x - 0.1 * s, y + 0.03 * s, z), (x - 0.072 * s, y + 0.025 * s, z)], 0.008 * s, M.porcelain,
            verts=6, resolution=2)


def _cup(F, x, y, z, turn=0.0):
    """A cup on its saucer, a rose band round its rim, its handle out at `turn` degrees (0: +x)."""
    _turned(F, _name('saucer'), x, y, z, [(0, 0), (0.03, 0), (0.05, 0.006), (0.052, 0.009), (0.034, 0.005), (0, 0.005)], M.porcelain, 16)
    _turned(F, _name('cup'), x, y + 0.006, z, [(0, 0), (0.022, 0), (0.038, 0.045), (0.034, 0.045), (0.02, 0.006), (0, 0.006)],
            M.porcelain, 14)
    G.torus(_name('cup_band'), 0.0365, 0.003, at=F.at(x, y + 0.049, z), material=M.rose, major_segments=14, minor_segments=3)
    c, s = G.cosd(turn), G.sind(turn)
    F.sweep(_name('cup_handle'), [(x + c * d, y + 0.006 + h, z + s * d) for d, h in ((0.034, 0.036), (0.05, 0.034),
                                                                                     (0.05, 0.018), (0.03, 0.014))], 0.0035,
            M.porcelain, verts=5, resolution=2)


def _bush(F, x, y, z, r, flower, seed=0):
    """A little flowering shrub: lumpy satin leaf clumps (no foliage draw) dotted with flowers."""
    rnd = random.Random(seed)
    for k, (dx, dz, f) in enumerate(((0, 0, 1.0), (0.05, 0.03, 0.75), (-0.05, -0.02, 0.8))):
        o = G.boulder(_name('clump'), r * f * 0.9, at=F.at(x + dx, y, z + dz), material=M.leaf if k else M.leaf_light,
                      scale=(1, 1, 0.85), seed=seed + k * 1.7, noise=0.3, subdivisions=2)
        G.jitter(o, r * 0.08, size=r * 0.5, seed=seed + k)
    for k in range(4):
        a = rnd.uniform(0, 360)
        F.sphere(_name('flower'), 0.018, (x + 0.06 * G.cosd(a), y + r * (1.25 + 0.2 * (k % 2)), z + 0.05 * G.sind(a)), flower,
                 scale=(1, 1, 0.6), segments=7, rings=4)


# --- the neighbourly baking counter (2x1): a cream retro range beside a sage counter laid with bakes --------
RANGE = (0.6, 0.62, 0.92, 0.6)   # x, width, hob height, depth


def neighbourlyOven():
    F = G.GameFrame()
    rx, rw, rh, rd = RANGE
    zf = WALL + rd                    # the range's front
    cx0, cx1 = -0.91, rx - rw / 2 - 0.01
    cw, cxm, cd = cx1 - cx0, (cx0 + cx1) / 2, 0.56
    with G.new_parts() as still:
        # the counter: a recessed plinth, the carcass, three shaker doors with brass knobs, a butcher-block top
        F.box('plinth', cw - 0.02, 0.08, cd - 0.05, (cxm, 0.04, WALL + (cd - 0.05) / 2), M.grate)
        F.box('carcass', cw, 0.78, cd, (cxm, 0.47, WALL + cd / 2), M.cabinet, bevel=0.008, bevel_segments=1)
        for i in range(3):
            x = cx0 + cw / 6 * (2 * i + 1)
            G.kit.shaker(F, _name('door'), cw / 3 - 0.03, 0.66, (x, 0.45, WALL + cd + 0.008), 0.016, M.cabinet,
                         rail=0.055, panel=M.cabinet_panel, raised=True)
            _turned(F, _name('knob'), x + (0.13 if i % 2 else -0.13), 0.68, WALL + cd + 0.016, [(0, 0), (0.01, 0), (0.006, 0.01),
                                                                                                (0.015, 0.02), (0.01, 0.028), (0, 0.029)],
                    M.brass, 10, rot=(-90, 0, 0))
        F.box('worktop', cw + 0.01, 0.045, cd + 0.03, (cxm, 0.8825, WALL + (cd + 0.03) / 2), M.butcher, bevel=0.008,
              bevel_segments=2)
        for k in range(1, 9):                              # the butcher block's strips
            F.box(_name('strip'), 0.003, 0.0015, cd + 0.022, (cx0 + k * cw / 9, 0.9052, WALL + (cd + 0.03) / 2), M.crust_dark)
        top = 0.905
        # the bakes: a lattice pie, a wire rack of biscuits, a rolling pin in flour, a pink cake on its stand
        px, pz = cxm - 0.28, -0.06
        _turned(F, 'pie_dish', px, top, pz, [(0, 0), (0.12, 0), (0.16, 0.05), (0.165, 0.052), (0.15, 0.052), (0.11, 0.008),
                                             (0, 0.008)], M.terracotta, 20)
        _turned(F, 'pie_crust', px, top, pz, [(0, 0.052), (0.1, 0.048), (0.15, 0.04), (0.152, 0.03), (0, 0.03)], M.crust, 20)
        G.torus('pie_rim', 0.148, 0.01, at=F.at(px, top + 0.052, pz), material=M.crust_dark, major_segments=24, minor_segments=3)
        for i in range(-2, 3):
            L = 2 * math.sqrt(max(0.0, 0.145 ** 2 - (i * 0.05) ** 2))
            for k, along in enumerate(('x', 'z')):
                pts = [((px + (t - 0.5) * L, top + 0.058 + 0.004 * math.cos(math.pi * (2 * t + i + k)), pz + i * 0.05)
                        if along == 'x' else (px + i * 0.05, top + 0.058 + 0.004 * math.cos(math.pi * (2 * t + i + k + 1)),
                                              pz + (t - 0.5) * L)) for t in (0.0, 0.5, 1.0)]
                F.sweep(_name('lattice'), pts, None, M.crust_dark, profile=[(-0.004, -0.01), (0.004, -0.01), (0.004, 0.01), (-0.004, 0.01)],
                        up=(0, 0, 1))
        rx0, rz0 = cxm + 0.15, -0.1
        for k in range(5):
            G.kit.tube(F, _name('rack_bar'), [(rx0 - 0.15, top + 0.016, rz0 - 0.1 + 0.05 * k), (rx0 + 0.15, top + 0.016, rz0 - 0.1 + 0.05 * k)],
                       0.0025, M.chrome, verts=4, round=1)
        for x in (rx0 - 0.15, rx0 + 0.15):
            G.kit.tube(F, _name('rack_end'), G.kit.fillet([(x, top, rz0 - 0.11), (x, top + 0.016, rz0 - 0.11),
                                                           (x, top + 0.016, rz0 + 0.11), (x, top, rz0 + 0.11)], 0.01), 0.003,
                       M.chrome, verts=4, round=1)
        for x, z in ((0.07, -0.15), (0.15, -0.06), (0.23, -0.14), (0.1, -0.03), (0.22, -0.04), (0.05, -0.09)):
            _turned(F, _name('biscuit'), cxm + x, top + 0.019, z, [(0, 0), (0.03, 0.004), (0.026, 0.012), (0, 0.012)],
                    M.crust_dark, 10)
        _turned(F, 'rolling_pin', cxm + 0.27, top + 0.025, 0.02, [(0, 0), (0.008, 0), (0.008, 0.05), (0.014, 0.056), (0.025, 0.065),
                                                                  (0.025, 0.235), (0.014, 0.244), (0.008, 0.25), (0.008, 0.3),
                                                                  (0, 0.3)], M.honey, 14, rot=(0, -90, 0))
        F.extrude('flour_dust', G.smooth_outline([(-0.1, -0.06), (0.04, -0.075), (0.11, -0.02), (0.08, 0.06), (-0.03, 0.07),
                                                  (-0.11, 0.02)], 2), 0.002, (cxm + 0.42, top + 0.001, -0.05), M.flour, plane='top')
        _turned(F, 'cake_stand', cxm - 0.42, top, -0.3, [(0, 0), (0.06, 0), (0.05, 0.01), (0.02, 0.025), (0.016, 0.06), (0.02, 0.07),
                                                         (0.12, 0.074), (0.122, 0.082), (0.11, 0.085), (0, 0.085)], M.porcelain, 16)
        _turned(F, 'cake', cxm - 0.42, top + 0.085, -0.3, [(0, 0), (0.09, 0), (0.09, 0.08), (0.084, 0.092), (0.07, 0.095), (0, 0.095)],
                M.icing, 20)
        for k in range(8):                                  # icing run down the cake's side
            a = k * 45 + 10
            F.sweep(_name('drip'), [(cxm - 0.42 + 0.088 * G.cosd(a), top + 0.177, -0.3 + 0.088 * G.sind(a)),
                                    (cxm - 0.42 + 0.092 * G.cosd(a), top + 0.15 - 0.012 * (k % 3), -0.3 + 0.092 * G.sind(a))],
                    0.007, M.flour, verts=5, scale=[1.0, 1.25])
        F.sphere('cherry', 0.014, (cxm - 0.42, top + 0.19, -0.3), M.heart, segments=8, rings=5)
        # a beadboard back rising from the worktop's back (a dresser's), carrying the shelf with three jars: it stands
        # free or on the wall (2026-10-21: the shelf hung on the air over a free-standing counter)
        F.box('backboard', cw, 1.62 - top, 0.018, (cxm, (top + 1.62) / 2, WALL + 0.009), M.cabinet, bevel=0.004,
              bevel_segments=1)
        for i in range(1, 8):
            F.box(_name('bead'), 0.006, 1.62 - top - 0.04, 0.004, (cx0 + i * cw / 8, (top + 1.62) / 2, WALL + 0.02),
                  M.cabinet_panel)
        F.sweep('backboard_cap', [(cx0 - 0.01, 1.6325, WALL + 0.02), (cx1 + 0.01, 1.6325, WALL + 0.02)], None, M.butcher,
                profile=G.rounded_rect(0.025, 0.04, 0.008, corner=1), up=(0, 0, 1))
        F.box('shelf', cw, 0.03, 0.2, (cxm, 1.5, WALL + 0.1), M.butcher, bevel=0.006, bevel_segments=1)
        for x in (cx0 + 0.06, cx1 - 0.06):
            F.extrude(_name('bracket'), [(0.0, 0.0), (0.0, 0.12), (0.16, 0.12), (0.15, 0.1), (0.08, 0.08), (0.03, 0.03)], 0.025,
                      (x, 1.365, WALL), M.butcher, plane='side', bevel=0.003, bevel_segments=1)
        for dx, h, m in ((-0.4, 0.16, M.jar_white), (-0.15, 0.2, M.jar_red), (0.1, 0.14, M.jar_gold)):
            _lathe(F, _name('jar'), cxm + dx, 1.515, WALL + 0.1, [(0.045, 0), (0.05, 0.006), (0.055, 0.02), (0.055, h - 0.035),
                                                                  (0.046, h - 0.022), (0.042, h - 0.02), (0.0, h - 0.02)], m, verts=12)
            _turned(F, _name('jar_lid'), cxm + dx, 1.515 + h - 0.0225, WALL + 0.1, [(0, 0), (0.05, 0), (0.051, 0.018), (0.046, 0.025),
                                                                                    (0, 0.025)], M.cork, 12)
            F.box(_name('jar_label'), 0.05, 0.04, 0.004, (cxm + dx, 1.515 + h * 0.45, WALL + 0.155), M.towel_check)
        # the range: an enamel body, a hob with four burners under cast-iron grates, a backguard with a clock, a knob
        # panel, a chrome rail with a gingham tea towel hung over it, a stock pot
        F.box('range', rw, rh - 0.04, rd - 0.03, (rx, (rh - 0.04) / 2, WALL + (rd - 0.03) / 2), M.enamel, bevel=0.02,
              bevel_segments=2)
        F.box('kick', rw - 0.06, 0.06, 0.02, (rx, 0.03, zf - 0.035), M.hob)
        F.box('hob', rw + 0.004, 0.035, rd - 0.01, (rx, rh - 0.0175, WALL + (rd - 0.01) / 2), M.hob, bevel=0.006, bevel_segments=1)
        F.box('backguard', rw, 0.16, 0.04, (rx, rh + 0.08, WALL + 0.02), M.enamel, bevel=0.012, bevel_segments=2)
        F.cyl('clock', 0.045, 0.008, (rx, rh + 0.09, WALL + 0.043), M.porcelain, rot=(90, 0, 0), verts=18)
        G.torus('clock_rim', 0.046, 0.004, at=F.at(rx, rh + 0.09, WALL + 0.045), material=M.chrome, rot=(90, 0, 0),
                major_segments=18, minor_segments=3)
        for a, L in ((60, 0.025), (-60, 0.035)):
            F.box(_name('clock_hand'), 0.004, L, 0.002, (rx + L / 2 * G.sind(a), rh + 0.09 + L / 2 * G.cosd(a), WALL + 0.048),
                  M.grate, rot=(0, -a, 0))
        for dx in (-0.15, 0.15):
            for dz in (-0.12, 0.12):
                bx, bz = rx + dx, WALL + rd / 2 + dz
                _turned(F, _name('burner'), bx, rh, bz, [(0, 0), (0.065, 0), (0.05, 0.01), (0, 0.016)], M.grate, 12)
                F.cyl(_name('burner_cap'), 0.03, 0.006, (bx, rh + 0.019, bz), M.chrome, verts=12)
                for a in (0, 90):
                    c, s = G.cosd(a), G.sind(a)
                    G.kit.tube(F, _name('grate'), G.kit.fillet([(bx - 0.1 * c, rh + 0.004, bz - 0.1 * s),
                                                                (bx - 0.1 * c, rh + 0.026, bz - 0.1 * s),
                                                                (bx + 0.1 * c, rh + 0.026, bz + 0.1 * s),
                                                                (bx + 0.1 * c, rh + 0.004, bz + 0.1 * s)], 0.01), 0.0055,
                               M.grate, verts=4, round=1)
        F.box('knob_panel', rw - 0.03, 0.1, 0.015, (rx, rh - 0.1, zf - 0.03 + 0.0075), M.enamel, bevel=0.004, bevel_segments=1)
        for i in range(5):
            _turned(F, _name('range_knob'), rx - 0.22 + i * 0.11, rh - 0.1, zf - 0.015, [(0, 0), (0.02, 0), (0.017, 0.012),
                                                                                         (0.014, 0.026), (0, 0.027)], M.chrome, 10,
                    rot=(-90, 0, 0))
        F.box('oven_mouth', rw - 0.1, 0.42, 0.006, (rx, 0.37, zf - 0.03 + 0.003), M.oven_dark)
        F.box('oven_rack', rw - 0.14, 0.01, 0.004, (rx, 0.3, zf - 0.03 + 0.007), M.chrome)
        G.kit.tube(F, 'rail', G.kit.fillet([(rx - rw / 2 + 0.06, 0.7, zf - 0.016), (rx - rw / 2 + 0.06, 0.7, zf + 0.012),
                                            (rx + rw / 2 - 0.06, 0.7, zf + 0.012), (rx + rw / 2 - 0.06, 0.7, zf - 0.016)], 0.012),
                   0.009, M.chrome, verts=8, round=1)
        tx = rx + 0.1
        pts = [(tx, 0.45, zf + 0.026), (tx, 0.66, zf + 0.026), (tx, 0.704, zf + 0.025), (tx, 0.716, zf + 0.012),
               (tx, 0.704, zf - 0.001), (tx, 0.6, zf - 0.002)]
        towel = F.sweep('towel', pts, None, M.towel, profile=G.rounded_rect(0.16, 0.006, 0.003, corner=1), up=(1, 0, 0),
                        resolution=2)
        G.jitter(towel, 0.002, size=0.06, seed=1.0, up=0.3)
        for k in range(4):
            F.box(_name('towel_check'), 0.16, 0.022, 0.003, (tx, 0.49 + k * 0.05, zf + 0.0305), M.towel_check)
        for k in range(3):
            F.box(_name('towel_check_v'), 0.026, 0.21, 0.0028, (tx - 0.053 + 0.053 * k, 0.555, zf + 0.0302), M.towel_check)
        sx, sz = rx + 0.15, WALL + rd / 2 - 0.12
        _turned(F, 'stock_pot', sx, rh + 0.026, sz, [(0, 0), (0.105, 0), (0.11, 0.008), (0.11, 0.13), (0.114, 0.138), (0.104, 0.14),
                                                     (0.1, 0.135), (0, 0.135)], M.sage_pot, 18)
        _turned(F, 'stock_lid', sx, rh + 0.163, sz, [(0, 0), (0.112, 0), (0.1, 0.012), (0.03, 0.02), (0, 0.02)], M.sage_dark, 18)
        F.sphere('stock_knob', 0.014, (sx, rh + 0.186, sz), M.sage_dark, segments=8, rings=5)
        for s in (-1, 1):
            G.kit.tube(F, _name('stock_handle'), G.kit.fillet([(sx + s * 0.108, rh + 0.13, sz), (sx + s * 0.135, rh + 0.13, sz),
                                                               (sx + s * 0.135, rh + 0.11, sz), (sx + s * 0.108, rh + 0.11, sz)], 0.006),
                       0.005, M.sage_dark, verts=4, round=1)
    G.join(still, 'body', origin=(0, 0, 0))
    # The oven door, hinged on its foot: `oven_open` drops it forward while a pie or treats bake. A window in a raised
    # frame, a chrome bar handle on stand-offs.
    with G.new_parts() as door:
        F.box('oven_door', rw - 0.04, 0.5, 0.02, (rx, 0.38, zf - 0.02), M.enamel, bevel=0.01, bevel_segments=2)
        G.kit.moulding(F, 'oven_frame', [(rx + 0.2, 0.25), (rx + 0.2, 0.47), (rx - 0.2, 0.47), (rx - 0.2, 0.25)],
                       [(-0.012, 0.0), (-0.012, 0.004), (0.0, 0.008), (0.012, 0.008), (0.012, 0.0)], M.enamel, at=zf - 0.01)
        F.box('oven_window', rw - 0.22, 0.2, 0.006, (rx, 0.36, zf - 0.009), M.oven_glass)
        G.kit.pull(F, 'door_handle', rx, 0.58, zf - 0.01, rw - 0.16, M.chrome, r=0.009, out=0.032, verts=8)
    door = G.hinged(G.join(door, 'oven_door'), F.at(rx, 0.13, zf - 0.02))
    objs = G.settle(centre=False)
    G.swing(door, 'oven_open', 85, seconds=0.8, axis='X')
    return objs


# --- the community giving jar (1x1): a glass jar of coins on a turned pedestal ----------------------------
JAR = (0.0, -0.05)   # x, depth of the jar's axis
LID_TOP = 1.28


def _heart(s):
    """A heart outline `s` across, its point down."""
    pts = []
    for k in range(16):
        t = math.tau * k / 16
        pts.append((s * 0.5 * math.sin(t) ** 3, s * 0.4 * (13 * math.cos(t) - 5 * math.cos(2 * t) - 2 * math.cos(3 * t)
                                                           - math.cos(4 * t)) / 16))
    return pts


def givingJar():
    F = G.GameFrame()
    jx, jz = JAR
    rnd = random.Random(5)
    with G.new_parts() as still:
        _turned(F, 'pedestal', jx, 0, jz, [(0, 0), (0.22, 0), (0.22, 0.03), (0.205, 0.045), (0.15, 0.06), (0.13, 0.09), (0.09, 0.12),
                                           (0.07, 0.17), (0.075, 0.2), (0.058, 0.24), (0.053, 0.5), (0.06, 0.66), (0.075, 0.72),
                                           (0.1, 0.76), (0.17, 0.8), (0.185, 0.815), (0.185, 0.835), (0.18, 0.86), (0, 0.86)],
                M.pedestal, 28)
        G.torus('pedestal_bead', 0.062, 0.009, at=F.at(jx, 0.28, jz), material=M.pedestal, major_segments=20, minor_segments=4)
        # coins heaped in the jar, notes among them
        for i in range(22):
            a = i * 2.4
            rr = 0.02 + 0.1 * ((i * 7) % 6) / 6
            F.cyl(_name('coin'), 0.022, 0.006, (jx + rr * math.cos(a), 0.872 + 0.01 * (i // 5) + 0.004 * (rr < 0.06),
                                                jz + rr * math.sin(a)), M.gold, rot=((i * 23) % 30, (i * 17) % 25, 0), verts=12)
        for k, (dx, dz, a) in enumerate(((-0.02, 0.02, 25), (0.05, -0.04, -40))):
            F.sweep(_name('note'), [(jx + dx - 0.05, 0.91, jz + dz), (jx + dx, 0.915 + 0.01 * k, jz + dz + 0.01), (jx + dx + 0.05, 0.93, jz + dz)],
                    None, M.note, profile=G.rounded_rect(0.002, 0.06, 0.001, corner=1), up=(0, 0, 1), resolution=2)
        # the cork lid and its slot, a ribbon round the neck tied in a bow, a heart tag on a string
        _turned(F, 'cork', jx, LID_TOP - 0.05, jz, [(0, 0), (0.1, 0), (0.106, 0.04), (0.103, 0.05), (0, 0.05)], M.cork, 22)
        F.box('slot', 0.07, 0.004, 0.012, (jx, LID_TOP + 0.001, jz), M.dark)
        G.torus('ribbon', 0.112, 0.01, at=F.at(jx, 1.2, jz), material=M.ribbon, major_segments=24, minor_segments=4)
        bz = jz + 0.115
        for s in (-1, 1):
            G.kit.tube(F, _name('bow_loop'), [(jx, 1.2, bz), (jx + s * 0.03, 1.225, bz + 0.012), (jx + s * 0.055, 1.21, bz + 0.008),
                                              (jx + s * 0.04, 1.192, bz + 0.006), (jx, 1.2, bz)], 0.007, M.ribbon, verts=5, round=2)
            F.sweep(_name('bow_tail'), [(jx, 1.195, bz + 0.004), (jx + s * 0.02, 1.16, bz + 0.012), (jx + s * 0.03, 1.13, bz + 0.016)],
                    None, M.ribbon, profile=G.rounded_rect(0.003, 0.016, 0.001, corner=1), up=(0, 1, 0), resolution=2)
        F.sphere('bow_knot', 0.012, (jx, 1.2, bz + 0.004), M.ribbon, segments=8, rings=5)
        G.kit.tube(F, 'tag_string', [(jx, 1.19, bz + 0.004), (jx + 0.004, 1.14, bz + 0.03), (jx, 1.11, bz + 0.04)], 0.0025, M.ribbon,
                   verts=4, round=2)
        F.extrude('heart_tag', _heart(0.075), 0.01, (jx, 1.075, jz + 0.155), M.heart, bevel=0.003, bevel_segments=1)
        # a chalk sign framed on the pedestal, a chalked heart and lines; a flowering pot plant at its foot
        F.extrude('sign_frame', G.rounded_rect(0.24, 0.18, 0.012, corner=2), 0.014, (jx, 0.62, jz + 0.074), M.pedestal,
                  holes=[G.rounded_rect(0.205, 0.145, 0.006, corner=1)], bevel=0.003, bevel_segments=1)
        F.box('sign', 0.21, 0.15, 0.006, (jx, 0.62, jz + 0.07), M.chalk)
        for y, w in ((0.6, 0.14), (0.575, 0.1)):
            F.box(_name('chalk_line'), w, 0.008, 0.003, (jx, y, jz + 0.0745), M.chalk_line)
        hl = [(u, v + 0.66) for u, v in _heart(0.04)]
        F.sweep('chalk_heart', [(jx + u, v, jz + 0.075) for u, v in hl], 0.0022, M.chalk_line, verts=4, closed=True)
        _turned(F, 'plant_pot', 0.31, 0, 0.291, [(0, 0), (0.058, 0), (0.075, 0.13), (0.085, 0.135), (0.085, 0.16), (0.075, 0.16),
                                               (0.07, 0.148), (0, 0.148)], M.terracotta, 18)
        _bush(F, 0.31, 0.15, 0.291, 0.1, M.mustard, seed=3)
        # the glass jar over the coins (the glass draw): a thick rim and a lip under the cork
        _turned(F, 'jar', jx, 0.86, jz, [(0, 0), (0.13, 0), (0.145, 0.012), (0.15, 0.076), (0.15, 0.285), (0.12, 0.325),
                                         (0.108, 0.334), (0.112, 0.34), (0.112, 0.352), (0.104, 0.36), (0, 0.36)], M.glass, 28)
        del rnd
    G.join(still, 'body', origin=(0, 0, 0))
    # A coin standing in the slot: `coin` drops it through into the jar.
    with G.new_parts() as coin:
        F.cyl('donation', 0.024, 0.005, (jx, LID_TOP + 0.016, jz), M.gold, rot=(90, 0, 0), verts=16)
        F.cyl('donation_face', 0.018, 0.0055, (jx, LID_TOP + 0.016, jz), M.mustard, rot=(90, 0, 0), verts=12)
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
        # the yarn basket beside it: a woven body with a rim and a handle, balls of wool, needles in the work
        bx, by, _ = F.at(0.35, 0, 0.334)
        G.loft('basket', [G.ring(0.075, 0.075, 0.0, 24, at=(bx, by))] +
               [_fluted(0.08 + 0.02 * t, 0.012 + 0.168 * t, 24, (bx, by), 12, 0.04) for t in (0.0, 0.33, 0.66, 1.0)] +
               [G.ring(0.09, 0.09, 0.18, 24, at=(bx, by)), G.ring(0.085, 0.085, 0.03, 24, at=(bx, by))], material=M.wicker)
        G.torus('basket_rim', 0.098, 0.008, at=F.at(0.35, 0.18, 0.334), material=M.honey, major_segments=24, minor_segments=4)
        G.kit.tube(F, 'basket_handle', [(0.26, 0.18, 0.334), (0.27, 0.27, 0.334), (0.35, 0.31, 0.334), (0.43, 0.27, 0.334), (0.44, 0.18, 0.334)],
                   0.007, M.honey, verts=6, round=2)
        for k, (x, y, z, r, m) in enumerate(((0.32, 0.18, 0.32, 0.045, M.rose_flower), (0.39, 0.17, 0.36, 0.04, M.sky),
                                             (0.35, 0.2, 0.39, 0.04, M.mustard))):
            F.sphere(_name('yarn'), r, (x, y, z), m, segments=12, rings=8)
            for j in range(2):
                G.torus(_name('yarn_wrap'), r * 0.98, 0.0035, at=F.at(x, y, z), material=m, rot=(60 * j + 20 * k, 30 + 50 * j, 0),
                        major_segments=14, minor_segments=3)
        G.kit.tube(F, 'yarn_strand', [(0.32, 0.2, 0.36), (0.3, 0.17, 0.42), (0.25, 0.04, 0.4), (0.18, 0.003, 0.3)], 0.003, M.rose_flower,
                   verts=4, round=2)
        for k, (a, b) in enumerate((((0.31, 0.15, 0.37), (0.41, 0.32, 0.29)), ((0.33, 0.15, 0.39), (0.38, 0.33, 0.31)))):
            F.sweep(_name('needle'), [a, b], 0.004, M.needle, verts=5, scale=[0.5, 1.0])
            F.sphere(_name('needle_end'), 0.007, b, M.mustard, segments=6, rings=4)
        F.box('knitting', 0.12, 0.012, 0.09, (0.36, 0.2, 0.33), M.rose_flower, rot=(10, 0, 20), bevel=0.004, bevel_segments=1)
    G.join(still, 'body', origin=(0, 0, 0))
    # The chair, its cushion and the throw over its back: one part turning about its rockers' centre, so they
    # roll on the floor; `rock` rocks it with its knitter (`ride`).
    w = 0.6
    py = ROCK_R + 0.025                 # the pivot's height: the runners' undersides are ROCK_R + 0.025 from it
    with G.new_parts() as chair:
        F.extrude('seat_board', G.smooth_outline([(-0.25, -0.2), (0.25, -0.2), (0.26, 0.17), (0.0, 0.215), (-0.26, 0.17)], 2),
                  0.035, (0, 0.42, CZ), M.rocker, plane='top', bevel=0.008, bevel_segments=1)
        for s in (-1, 1):
            x = s * (w / 2 - 0.07)
            # the runner: one curved piece, its underside on the rockers' circle, its ends rising
            pts = [(x, py - (ROCK_R + 0.005) * G.cosd(a), CZ + (ROCK_R + 0.005) * G.sind(a)) for a in range(-21, 22, 3)]
            F.sweep(_name('rocker'), pts, None, M.rocker, profile=G.rounded_rect(0.04, 0.03, 0.01, corner=1), up=(0, 0, 1),
                    resolution=1)
            F.sweep(_name('front_leg'), [(x, 0.04, CZ + 0.2), (x, 0.25, CZ + 0.19), (x, 0.43, CZ + 0.18)], 0.018, M.rocker, verts=8,
                    resolution=2, scale=[0.85, 1.0, 0.9])
            F.sweep(_name('stile'), [(x, 0.035, CZ - 0.2), (x, 0.42, CZ - 0.19), (x * 1.01, 0.75, CZ - 0.25), (x * 1.02, 1.0, CZ - 0.32)],
                    0.02, M.rocker, verts=8, resolution=2, scale=[0.85, 1.0, 0.95, 0.85])
            _turned(F, _name('finial'), x * 1.02, 0.998, CZ - 0.322, [(0, 0), (0.018, 0), (0.024, 0.012), (0.018, 0.024), (0.008, 0.03),
                                                                      (0.012, 0.038), (0, 0.048)], M.rocker, 10)
            # the arm: rising from its post and rolling over at the front into a hand-rest
            F.sweep(_name('arm'), [(x * 1.06, 0.655, CZ - 0.24), (x * 1.08, 0.665, CZ - 0.02), (x * 1.08, 0.66, CZ + 0.17),
                                   (x * 1.07, 0.645, CZ + 0.21)], None, M.rocker, profile=G.rounded_rect(0.03, 0.055, 0.012, corner=1),
                    up=(0, 0, 1), resolution=2, scale=[0.8, 1.0, 1.1, 1.0])
            F.sweep(_name('arm_post'), [(x * 1.06, 0.42, CZ + 0.17), (x * 1.07, 0.55, CZ + 0.175), (x * 1.08, 0.65, CZ + 0.17)], 0.015,
                    M.rocker, verts=8, resolution=2, scale=[1.0, 0.8, 1.0])
            G.kit.tube(F, _name('side_stretcher'), [(x, 0.15, CZ - 0.19), (x, 0.15, CZ + 0.19)], 0.011, M.rocker, verts=6, round=1)
        for i in range(6):
            xx = -0.15 + i * 0.06
            F.sweep(_name('spindle'), [(xx, 0.44, CZ - 0.185), (xx, 0.68, CZ - 0.235), (xx * 1.05, 0.94, CZ - 0.3)], 0.01, M.rocker,
                    verts=6, resolution=2, scale=[0.8, 1.0, 0.8])
        crest = [(x, 0.965 + 0.03 * (1 - (x / 0.26) ** 2), CZ - 0.31 - 0.015 * (1 - (x / 0.26) ** 2)) for x in
                 (-0.26, -0.17, -0.085, 0.0, 0.085, 0.17, 0.26)]
        F.sweep('crest', crest, None, M.rocker, profile=G.rounded_rect(0.08, 0.035, 0.014, corner=2), up=(0, 0, 1))
        G.kit.tube(F, 'stretcher', [(-w / 2 + 0.07, 0.15, CZ), (w / 2 - 0.07, 0.15, CZ)], 0.011, M.rocker, verts=6, round=1)
        F.soft('seat_cushion', 0.42, 0.04, 0.38, (0, 0.46, CZ + 0.01), M.cushion, round=0.02, puff=0.012, piping=0.004, flat=2)
        for s in (-1, 1):
            G.kit.tube(F, _name('cushion_tie'), [(s * 0.17, 0.46, CZ - 0.17), (s * 0.2, 0.45, CZ - 0.19), (s * 0.21, 0.4, CZ - 0.18)],
                       0.004, M.cushion, verts=4, round=2)
        # the throw folded over the crest, hanging down the back
        pts = [(0.02, 0.62, -0.25), (0.02, 0.85, -0.305), (0.02, 1.0, -0.335), (0.02, 1.045, -0.36), (0.02, 1.0, -0.388),
               (0.02, 0.8, -0.4)]
        throw = F.sweep('throw', pts, None, M.throw, profile=G.rounded_rect(0.36, 0.014, 0.006, corner=1), up=(1, 0, 0),
                        resolution=2)
        G.jitter(throw, 0.004, size=0.1, seed=3.0, up=0.5)
    chair = G.hinged(G.join(chair, 'chair'), F.at(0, py, CZ))
    objs = G.settle(centre=False)
    G.rock(chair, 'rock', 5, seconds=1.6, axis='X')
    return objs


# --- the welcome tea table (2x1): two poufs (tops 0.45 at x = ±0.75) facing across a low table with the tea set --
POUF_X = 0.75                                   # content `places` [±0.75, 0], facing ±90


def welcomeTeaTable():
    F = G.GameFrame()
    with G.new_parts() as still:
        # a braided oval rug: rings of plait in two colours to a border
        F.cyl('rug', 0.95, 0.012, (0, 0.006, 0), M.rug, verts=48, scale=(1, 0.44, 1))
        for k, (r, rz, m) in enumerate(((0.95, 0.4086, M.rug_edge), (0.62, 0.26, M.rug_edge))):
            ring = [(r * math.cos(math.tau * i / 40), rz * math.sin(math.tau * i / 40)) for i in range(40)]
            hw, ht = (0.02, 0.006) if k == 0 else (0.015, 0.004)
            F.sweep(_name('rug_braid'), [(u, 0.012, v) for u, v in ring], None, m,
                    profile=[(-ht, -hw), (ht, -hw), (ht, hw), (-ht, hw)], up=(0, 0, 1), closed=True)
        for x, m in ((-POUF_X, M.pouf), (POUF_X, M.pouf_rose)):
            bx, by, _ = F.at(x, 0, 0)
            # a cable-knit pouf: its side ribbed, its top a cushion with piping and a button
            G.loft(_name('pouf'), [G.ring(0.18, 0.18, 0.0, 40, at=(bx, by)), _fluted(0.198, 0.02, 40, (bx, by), 10, 0.035),
                                   _fluted(0.205, 0.12, 40, (bx, by), 10, 0.045), _fluted(0.205, 0.25, 40, (bx, by), 10, 0.045),
                                   _fluted(0.198, 0.35, 40, (bx, by), 10, 0.035), G.ring(0.185, 0.185, 0.37, 40, at=(bx, by))],
                   material=m)
            _turned(F, _name('pouf_top'), x, 0.0, 0.0, [(0, 0.37), (0.19, 0.37), (0.205, 0.39), (0.197, 0.425), (0.16, 0.444),
                                                        (0.08, 0.45), (0, 0.448)], M.cushion, 28)
            G.torus(_name('piping'), 0.202, 0.009, at=F.at(x, 0.372, 0.0), material=M.rug, major_segments=28, minor_segments=4)
            F.sphere(_name('pouf_button'), 0.014, (x, 0.448, 0.0), M.rug, scale=(1, 1, 0.45), segments=8, rings=4)
        # the low table between them (top 0.36, 0.4 across: their knees stop short of its ends, their feet before
        # them), a shelf of books, a tea light
        F.box('table_top', 0.4, 0.04, 0.56, (0, 0.34, -0.02), M.table, bevel=0.012, bevel_segments=2)
        for x in (-0.16, 0.16):
            for z in (-0.25, 0.21):
                F.sweep(_name('table_leg'), [(x, 0.32, z), (x + math.copysign(0.008, x), 0.0, z + math.copysign(0.008, z + 0.02))], None,
                        M.table, profile=G.rounded_rect(0.04, 0.04, 0.01, corner=1), up=(0, 0, 1), scale=[1.0, 0.7])
        F.box('table_shelf', 0.32, 0.02, 0.48, (0, 0.1, -0.02), M.table, bevel=0.005, bevel_segments=1)
        for k, (y, h, m, r) in enumerate(((0.11, 0.035, M.mustard, 0), (0.145, 0.03, M.sky, 8))):
            F.box(_name('book'), 0.16 - 0.01 * k, h, 0.12 - 0.01 * k, (-0.04, y + h / 2, -0.1), m, bevel=0.003, bevel_segments=1,
                  rot=(0, 0, r))
            F.box(_name('book_pages'), 0.15 - 0.01 * k, h - 0.008, 0.124 - 0.01 * k, (-0.035, y + h / 2, -0.1), M.candle, rot=(0, 0, r))
        _turned(F, 'tea_light_cup', 0.1, 0.36, -0.22, [(0, 0), (0.03, 0), (0.038, 0.04), (0.034, 0.04), (0.026, 0.006), (0, 0.006)],
                M.rose, 14)
        F.cyl('tea_light', 0.024, 0.016, (0.1, 0.372, -0.22), M.candle, verts=12)
        F.sphere('tea_light_flame', 0.008, (0.1, 0.396, -0.22), M.flame, scale=(1, 1, 2), segments=7, rings=5)
        _cup(F, -0.1, 0.36, 0.14, turn=200)      # a cup before each of them
        _cup(F, 0.1, 0.36, 0.1, turn=-20)
        _turned(F, 'milk_jug', 0.11, 0.36, -0.1, [(0, 0), (0.024, 0), (0.03, 0.035), (0.026, 0.055), (0.022, 0.062), (0.026, 0.07),
                                                  (0.022, 0.07), (0.018, 0.062), (0, 0.06)], M.porcelain, 14)
        F.sweep('milk_handle', [(0.088, 0.415, -0.1), (0.075, 0.41, -0.1), (0.075, 0.385, -0.1), (0.086, 0.378, -0.1)], 0.003,
                M.porcelain, verts=4, resolution=2)
        F.sphere('milk_spout', 0.008, (0.135, 0.428, -0.1), M.porcelain, scale=(1.4, 0.6, 0.6), segments=6, rings=4)
        # potted ferns at the back, between the poufs and the table: arching fronds in two greens
        for x in (-0.44, 0.44):
            _turned(F, _name('fern_pot'), x, 0, -0.33, [(0, 0), (0.065, 0), (0.08, 0.17), (0.09, 0.175), (0.09, 0.2), (0.082, 0.2),
                                                        (0.078, 0.19), (0, 0.19)], M.pot, 16)
            for k in range(6):
                a = k * 60 - 30
                c, s = G.cosd(a), G.sind(a)
                L = 0.13 + 0.03 * (k % 2)
                s = s * (0.8 if s < 0 else 1.0)              # (less towards the wall)
                F.sweep(_name('frond'), [(x, 0.185, -0.33), (x + 0.3 * L * c, 0.3 + 0.03 * (k % 2), -0.33 + 0.33 * L * s),
                                         (x + 0.8 * L * c, 0.33, -0.33 + 0.75 * L * s), (x + L * c, 0.26, -0.33 + 0.85 * L * s)], None,
                        M.leaf if k % 2 else M.leaf_light, profile=G.rounded_rect(0.003, 0.05, 0.001, corner=1), up=(0, 0, 1),
                        resolution=2, scale=[0.3, 1.0, 0.8, 0.1])
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


def songbirdFeeder():
    F = G.GameFrame()
    px, pz = POST
    with G.new_parts() as still:
        F.box('post_foot', 0.3, 0.04, 0.3, (px, 0.02, pz), M.stone, bevel=0.01, bevel_segments=1)
        F.box('post_plinth', 0.12, 0.06, 0.12, (px, 0.07, pz), M.post, bevel=0.01, bevel_segments=1)
        F.box('post', 0.07, 1.31, 0.07, (px, 0.71, pz), M.post, bevel=0.008, bevel_segments=1)
        for s in (-1, 1):                                    # brackets under the tray, out from the post's four faces
            arm = [(0.0, 0.0), (0.0, 0.06), (s * 0.12, 0.06), (s * 0.11, 0.045), (s * 0.03, 0.01)]
            F.extrude(_name('tray_bracket'), arm, 0.016, (px + s * 0.035, 1.298, pz), M.post, plane='front')
            F.extrude(_name('tray_bracket'), arm, 0.016, (px, 1.298, pz + s * 0.035), M.post, plane='side')
        F.box('seed_tray', 0.38, 0.02, 0.32, (px, 1.368, pz), M.tray, bevel=0.004, bevel_segments=1)
        loop = [(u + px, v + pz) for u, v in G.rounded_rect(0.37, 0.31, 0.012, corner=2)]
        F.sweep('tray_lip', [(u, 1.385, v) for u, v in loop], None, M.tray, profile=G.rounded_rect(0.024, 0.012, 0.004, corner=1),
                up=(0, 0, 1), closed=True)
        for x, z in ((-0.31, -0.12), (-0.12, -0.3), (-0.27, -0.31), (-0.13, -0.13), (-0.3, -0.33)):
            F.sphere(_name('seeds'), 0.025, (x, 1.383, z), M.seed, scale=(1, 1, 0.35), segments=8, rings=4)
        # the feeder house: walls with a clear hopper of seed, a round door with a perch, corner trims
        F.box('house', 0.22, 0.2, 0.18, (px, HOUSE_Y, pz), M.feeder, bevel=0.006, bevel_segments=1)
        for sx in (-1, 1):
            for sz in (-1, 1):
                F.box(_name('house_trim'), 0.018, 0.2, 0.018, (px + sx * 0.105, HOUSE_Y, pz + sz * 0.085), M.post, bevel=0.003,
                      bevel_segments=1)
        F.box('house_hopper', 0.16, 0.07, 0.004, (px, 1.42, pz + 0.091), M.water, bevel=0.002, bevel_segments=1)
        F.box('hopper_seed', 0.15, 0.045, 0.003, (px, 1.41, pz + 0.0905), M.seed)
        F.cyl('house_hole', 0.033, 0.006, (px, 1.51, pz + 0.091), M.dark, rot=(90, 0, 0), verts=16)
        G.torus('house_hole_ring', 0.035, 0.005, at=F.at(px, 1.51, pz + 0.093), material=M.post, rot=(90, 0, 0), major_segments=16,
                minor_segments=3)
        G.kit.tube(F, 'perch', [(px, 1.465, pz + 0.09), (px, 1.465, pz + 0.16)], 0.006, M.tray, verts=6, round=1)
        G.kit.songbird(F, 'robin', px, 1.471, pz + 0.15, M.bird_brown, M.bird_orange, M.mustard, M.dark, turn=0, size=0.9)
        # the bird bath: a fluted stem, a dished bowl with a rolled rim, its water
        _turned(F, 'bath_stand', 0.2, 0, 0.16, [(0, 0), (0.12, 0), (0.12, 0.02), (0.1, 0.03), (0.06, 0.06), (0.045, 0.1),
                                                (0.042, 0.31), (0.05, 0.5), (0.06, 0.55), (0.1, 0.6), (0, 0.6)], M.stone, 20)
        _turned(F, 'bath_bowl', 0.2, 0, 0.16, [(0, 0.6), (0.15, 0.6), (0.24, 0.66), (0.252, 0.69), (0.245, 0.708), (0.228, 0.71),
                                               (0.215, 0.695), (0.18, 0.67), (0, 0.665)], M.stone, 32)
        F.cyl('bath_water', 0.212, 0.008, (0.2, 0.688, 0.16), M.water, verts=28)
        # flowering shrubs, a seed sack with its top rolled down, a scoop tin with its scoop
        for k, (x, z, m) in enumerate(((-0.34, 0.02, M.rose_flower), (-0.05, -0.36, M.mustard), (0.34, -0.3, M.lilac),
                                       (-0.34, 0.33, M.rose_flower))):
            _bush(F, x, 0.0, z, 0.08, m, seed=k * 3)
        F.soft('sack', 0.16, 0.2, 0.1, (0.08, 0.1, WALL + 0.06), M.sack, round=0.03, puff=0.012, flat=2)
        G.torus('sack_roll', 0.075, 0.016, at=F.at(0.08, 0.205, WALL + 0.06), material=M.sack, rot=(0, 0, 0), major_segments=16,
                minor_segments=5).scale = (1.05, 0.62, 1.0)
        F.sphere('sack_seed', 0.06, (0.08, 0.2, WALL + 0.06), M.seed, scale=(1.0, 0.55, 0.25), segments=10, rings=5)
        _turned(F, 'scoop_tin', -0.02, 0, 0.38, [(0, 0), (0.05, 0), (0.06, 0.1), (0.064, 0.102), (0.06, 0.106), (0.056, 0.1),
                                                 (0.046, 0.006), (0, 0.006)], M.tin, 16)
        F.sphere('tin_seed', 0.05, (-0.02, 0.09, 0.38), M.seed, scale=(1.0, 1.0, 0.25), segments=10, rings=5)
        G.kit.tube(F, 'scoop', [(-0.02, 0.085, 0.38), (0.0, 0.11, 0.37), (0.02, 0.15, 0.36)], 0.006, M.tin, verts=6, round=1)
    G.join(still, 'body', origin=(0, 0, 0))
    # The roof, its ridge left to right, hinged on its back eave: `lid_open` lifts it for a refill. Shingled slopes,
    # a ridge cap.
    eave_y, eave_z = HOUSE_Y + 0.1, pz - 0.115
    with G.new_parts() as roof:
        for s in (-1, 1):
            F.box(_name('roof_slab'), 0.28, 0.02, 0.15, (px, HOUSE_Y + 0.15, pz + s * 0.06), M.roof, rot=(s * 36, 0, 0),
                  bevel=0.004, bevel_segments=1)
            for k in range(3):                             # rows of shingles, each a strip with its lower edge proud
                t = 0.02 + 0.045 * k
                yy = HOUSE_Y + 0.2 - (0.13 - t) * G.sind(36) - 0.006
                zz = pz + s * ((0.13 - t) * G.cosd(36) - 0.01)
                F.box(_name('shingle'), 0.284, 0.012, 0.05, (px, yy + 0.012, zz), M.roof, rot=(s * 36, 0, 0), bevel=0.003,
                      bevel_segments=1)
        F.cyl('ridge', 0.016, 0.29, (px, HOUSE_Y + 0.2, pz), M.roof, rot=(0, 90, 0), verts=10)
    roof = G.hinged(G.join(roof, 'roof'), F.at(px, eave_y, eave_z))
    # The songbirds that come while residents watch: built where they perch, joined with their origin in the
    # feeder house and shrunk into it; `bird` grows them out to the tray and the bath.
    with G.new_parts() as birds:
        G.kit.songbird(F, 'tit', -0.34, 1.383, -0.17, M.bird_blue, M.bird_cream, M.mustard, M.dark, turn=-60, size=0.85, head=20)
        G.kit.songbird(F, 'finch', 0.36, 0.694, 0.2, M.bird_gold, M.bird_yellow, M.mustard, M.dark, turn=70, size=0.9)
        G.kit.songbird(F, 'sparrow', 0.02, 0.694, 0.1, M.bird_grey, M.bird_pink, M.mustard, M.dark, turn=-80, size=0.9, head=35)
    birds = G.join(birds, 'birds', origin=F.at(px, HOUSE_Y, pz))
    birds.scale = (0.001, 0.001, 0.001)
    objs = G.settle()      # on its Model centre: the back on the wall face
    G.swing(roof, 'lid_open', -70, seconds=0.7, axis='X')
    for axis in 'XYZ':
        G.grow(birds, 'bird', seconds=1.0, axis=axis)
    return objs


# --- the hearthside quilted settee (2x1): two seats at x = ±0.5 (tops 0.45), a patchwork quilt -------------
def _rolled_arm(F, name, ax, s, z0, z1, y0, top, r, mat, lean=0.02):
    """A rolled (English) arm in one piece: a panel rising from `y0` into a roll of radius `r` whose top is `top`,
    leaning out by `lean`, swept from depth z0 to z1 at x `ax` (side `s`); a welt round its front (as living.py's)."""
    cy, inner = top - r, -r * 1.1
    roll = [(cy + r * math.sin(math.radians(a)), s * (lean + r * math.cos(math.radians(a)))) for a in range(-40, 231, 18)]
    section = [(y0, s * inner), (y0, s * (lean + r * 0.92))] + roll + [(cy - r * 1.4, s * inner)]   # (height, x out)
    F.sweep(name, [(ax, 0, z0), (ax, 0, z1)], None, mat, profile=section, angle=50)
    pts = [(ax + x * 0.9, cy + (y - cy) * 0.9, z1 - 0.002) for y, x in section[2:-1]]
    pts = [(ax + s * (lean + r * 0.92) * 0.97, y0 + 0.03, z1 - 0.002)] + pts
    F.sweep(f'{name}_welt', pts, 0.006, mat, verts=5, resolution=2)


def hearthSettee():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.soft('base', 1.66, 0.28, 0.78, (0, 0.22, -0.03), M.body, round=0.05, piping=0.006, flat=2)
        F.soft('back', 1.66, 0.5, 0.2, (0, 0.62, -0.33), M.body, round=0.07, flat=2)
        for x in (-0.4, 0.4):
            F.soft(_name('back_cushion'), 0.74, 0.36, 0.14, (x, 0.7, -0.2), M.body, rot=(-10, 0, 0), round=0.05, puff=0.03,
                   piping=0.005, flat=3)
        for s in (-1, 1):
            _rolled_arm(F, _name('arm'), s * 0.82, s, -0.42, 0.37, 0.08, 0.67, 0.11, M.body)
            for z in (-0.36, 0.3):
                _turned(F, _name('leg'), s * 0.82, 0, z, [(0, 0), (0.018, 0), (0.022, 0.012), (0.03, 0.04), (0.035, 0.07),
                                                          (0.028, 0.09), (0.032, 0.1), (0, 0.1)], M.honey, 12)
        F.soft('pillow_l', 0.34, 0.3, 0.12, (-0.62, 0.66, -0.12), M.pillow_a, rot=(-12, 0, 18), round=0.05, puff=0.035, flat=3)
        F.soft('pillow_r', 0.3, 0.28, 0.12, (0.66, 0.66, -0.12), M.pillow_b, rot=(-12, 0, -18), round=0.05, puff=0.035, flat=3)
        for k, (x, y, z) in enumerate(((-0.62, 0.66, -0.06), (0.66, 0.66, -0.06))):
            F.sphere(_name('pillow_button'), 0.012, (x + 0.01 * (1 - 2 * k), y, z - 0.004), M.brass, scale=(1, 0.5, 1), segments=6,
                     rings=4)
        # a brass lantern on the left arm: a base, four posts round its light, a peaked cap, a ring
        lx, ly, lz = -0.84, 0.68, 0.12
        _turned(F, 'lantern_base', lx, ly, lz, [(0, 0), (0.07, 0), (0.07, 0.012), (0.06, 0.02), (0, 0.02)], M.brass, 4,
                rot=(0, 0, 45))
        F.box('lantern_light', 0.1, 0.15, 0.1, (lx, ly + 0.095, lz), M.lantern, bevel=0.01, bevel_segments=1)
        for dx in (-0.052, 0.052):
            for dz in (-0.052, 0.052):
                F.box(_name('lantern_post'), 0.012, 0.16, 0.012, (lx + dx, ly + 0.095, lz + dz), M.brass)
        for y in (ly + 0.03, ly + 0.165):
            F.box(_name('lantern_band'), 0.116, 0.01, 0.116, (lx, y, lz), M.brass)
        F.cone('lantern_cap', 0.085, 0.02, 0.06, (lx, ly + 0.19, lz), M.brass, verts=4, rot=(0, 0, 45))
        G.torus('lantern_ring', 0.025, 0.005, at=F.at(lx, ly + 0.235, lz), material=M.brass, rot=(90, 0, 0),
                major_segments=12, minor_segments=4)
    G.join(still, 'body', origin=(0, 0, 0))
    # The seat cushions and the quilt over the right one, origin on their underside: `sink_in` presses them. The quilt
    # is a strip quilt, its strips running over the cushion and down its front, stitched across.
    with G.new_parts() as seats:
        puff = 0.016
        for x in (-0.41, 0.41):
            F.soft(_name('seat_cushion'), 0.8, 0.12 - puff, 0.62, (x, 0.39 - puff / 2, 0.06), M.body, round=0.04, puff=puff,
                   piping=0.006, flat=3)
        z0, z1 = -0.21, 0.37
        strips = (M.patch_a, M.quilt, M.patch_b, M.patch_c, M.patch_d)
        sw = 0.76 / len(strips)
        for k, m in enumerate(strips):
            qx = 0.42 - 0.38 + sw * (k + 0.5)
            pts = [(qx, 0.454, z0), (qx, 0.46, (z0 + z1) / 2), (qx, 0.449, z1 - 0.03), (qx, 0.433, z1 - 0.004),
                   (qx, 0.405, z1 + 0.012), (qx, 0.21, z1 + 0.016)]
            strip = F.sweep(_name('quilt_strip'), pts, None, m, profile=G.rounded_rect(sw + 0.002, 0.014, 0.005, corner=1),
                            up=(1, 0, 0), resolution=2)
            G.jitter(strip, 0.003, size=0.12, seed=4.0, up=0.5)
        for k in range(4):
            t = (k + 0.5) / 4
            F.box(_name('quilt_stitch'), 0.762, 0.003, 0.006, (0.42, 0.464 + 0.002 * math.sin(math.pi * t), z0 + (z1 - 0.06 - z0) * t), M.quilt)
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

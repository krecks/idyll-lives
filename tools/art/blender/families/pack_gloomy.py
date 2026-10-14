# The gloomy pack (P7): the midnight record cabinet, the nocturne cello, the candlelit writing desk, the rainfall
# lamp, the brooding window seat and the moonstone soaking tub, over their primitive-part placeholders
# (`tools/art/pack_gloomy.py`, `packs/gloomy/manifest.json`), which stay the fallback. Notes: pack_gloomy.md.
#
# Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`). The cabinet's lid lifts (`lid_open`) and its platter turns while the record plays (`platter_spin`, a
# loop); the cello comes off its stand into the player's arms (`off_stand`); a candle is lit on the desk
# (`candle`); the rain runs down the lamp's lines (`rain`) over a pool that ripples all the time (`ripple`,
# ambient); the window seat's cushion sinks (`sink_in`); the tub fills (`fill`).
#
# Run:  python3 -B tools/art/blender/build.py pack_gloomy [--only cello] [--preview]
import os
import random
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    walnut=('#7A5234', 'wood'),      # the record cabinet; its main part
    slat=('#8E6443', 'wood'),
    rosewood=('#8A4A35', 'wood'),    # the cello's stool and stand, the writing desk
    ink=('#3A3550', 'satin'),        # legs, the cabinet's inside, trim
    plum=('#6B4A6E', 'satin'),       # cushions and upholstery
    mauve=('#9C7FA0', 'satin'),
    slate=('#5E6F8C', 'metal'),      # the rain lamp's base and canopy; its main part
    smoke=('#A7A9B4', 'satin'),
    chrome=('#D0D3D6', 'chrome'),
    brass=('#B8925A', 'metal'),
    platter=('#C9CCD0', 'metal'),
    vinyl=('#26222E', 'gloss'),
    label=('#C25B7A', 'gloss'),
    page=('#EFE6D3', 'matte'),
    sleeve_a=('#5E6F8C', 'matte'),
    sleeve_b=('#9C7FA0', 'matte'),
    sleeve_c=('#C9B37E', 'matte'),
    sleeve_d=('#4A3F5C', 'matte'),
    wax=('#F3EAD8', 'satin'),
    wick=('#2E2420', 'satin'),
    candle_flame=('#FFC45C', 'gloss'),
    flame=('#FFB347', 'gloss', {'glow': '#FFC870'}),   # the desk's candle flame, which glows at night
    vase=('#B7A9C6', 'gloss'),
    twig=('#6E5A3E', 'matte'),
    dried=('#C9A0B8', 'matte'),
    varnish=('#9A4A26', 'gloss'),    # the cello
    ebony=('#2E2420', 'gloss'),
    bridge=('#E6D3B0', 'matte'),
    string=('#E3E3E3', 'chrome'),
    rose=('#8A2E46', 'satin'),
    stem=('#5C6B3A', 'matte'),
    inkwell=('#2E3348', 'gloss'),
    pool=('#BFE3F2', 'gloss'),
    rain=('#A9D8EE', 'gloss'),
    ripple=('#E6F6FC', 'gloss'),
    pebble=('#B8B4AE', 'satin'),
    pebble_dark=('#9EA0A6', 'satin'),
    umbrella=('#3A3550', 'satin'),
    figure=('#5E6F8C', 'satin'),
    bench=('#7E8BA6', 'satin'),      # the window seat; its main part
    bench_front=('#8592AD', 'satin'),
    frame=('#D8D6DE', 'satin'),
    pane=('#4F6684', 'gloss'),       # the rainy window: opaque gloss, not see-through glass (the wall is behind it)
    drop=('#E3F0F8', 'gloss'),
    cushion=('#9C7FA0', 'fabric'),   # the window seat's cushion and throw (their own part)
    throw=('#A7A9B4', 'fabric'),
    pillow_a=('#5E6F8C', 'satin'),
    pillow_b=('#C9B37E', 'satin'),
    mug=('#E8E4EC', 'gloss'),
    stone=('#BDB7AD', 'satin'),      # the soaking tub; its main part
    moon=('#E4E1EA', 'gloss'),
    ledge=('#5A4A4E', 'wood'),       # the tub's candle ledge
    ledge_top=('#CFC9C0', 'satin'),
    water=('#8FC1D4', 'gloss'),
    foam=('#F2F6F8', 'gloss'),
    towel=('#EDE8F2', 'satin'),
    leaf=('#6E8F72', 'satin'),
)

WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
WALL2 = -0.93  # in a 2-deep one
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


def _candle(F, x, y, z, h=0.12, r=0.025, lit=True):
    """A pillar candle standing on `y`, its wick, and (lit) a small still flame."""
    F.cyl(_name('candle'), r, h, (x, y + h / 2, z), M.wax, verts=12)
    F.cyl(_name('wick'), 0.002, 0.012, (x, y + h + 0.006, z), M.wick, verts=4)
    if lit:
        F.sphere(_name('candle_flame'), 0.009, (x, y + h + 0.025, z), M.candle_flame, scale=(1, 1, 2.0), segments=8, rings=5)


def _vase(F, x, y, z, s=1.0):
    G.revolve(_name('vase'), [(0, 0), (0.04 * s, 0), (0.07 * s, 0.08 * s), (0.06 * s, 0.15 * s), (0.028 * s, 0.185 * s),
                              (0.034 * s, 0.2 * s), (0.026 * s, 0.2 * s), (0, 0.16 * s)],
              at=F.at(x, y, z), material=M.vase, segments=14)
    return y + 0.2 * s


# --- the midnight record cabinet (1x1): a walnut console, a turntable under a hinged lid -------------------
TOP = 0.62


def recordCabinet():
    F = G.GameFrame()
    rnd = random.Random(8)
    zf = -0.01            # the console's front
    zc = (zf + WALL) / 2
    d = zf - WALL
    with G.new_parts() as still:
        for x in (-0.4, 0.4):
            for z in (WALL + 0.05, zf - 0.05):
                F.cone(_name('leg'), 0.02, 0.012, 0.14, (x, 0.07, z), M.ink, verts=10)
        F.box('bottom', 0.9, 0.03, d, (0, 0.155, zc), M.walnut, bevel=0.006)
        F.box('deck', 0.9, 0.03, d, (0, 0.545, zc), M.walnut)                        # under the turntable and the top
        for x in (-0.435, 0.435):
            F.box(_name('side'), 0.03, TOP - 0.14, d, (x, (TOP + 0.14) / 2, zc), M.walnut, bevel=0.006)
        F.box('back', 0.84, 0.42, 0.01, (0, 0.35, WALL + 0.005), M.ink)
        F.box('divider', 0.02, 0.37, d - 0.02, (0, 0.355, zc - 0.01), M.walnut)
        # the left (game -x) bay: LP sleeves standing on end
        x = -0.415
        while x < -0.03:
            t = rnd.uniform(0.008, 0.016)
            m = getattr(M, rnd.choice(('sleeve_a', 'sleeve_b', 'sleeve_c', 'sleeve_d')))
            h = 0.31 if rnd.random() < 0.8 else 0.26
            F.box(_name('lp'), t, h, 0.31, (x + t / 2, 0.17 + h / 2, WALL + 0.025 + 0.155), m, rot=(0, rnd.uniform(-4, 4), 0))
            x += t + 0.003
        # the right bay: slatted doors
        F.box('door_back', 0.41, 0.36, 0.012, (0.215, 0.355, zf - 0.006), M.ink)
        for k in range(9):
            F.box(_name('door_slat'), 0.034, 0.36, 0.016, (0.035 + k * 0.045, 0.355, zf + 0.004), M.slat)
        F.box('door_pull', 0.01, 0.08, 0.012, (0.04, 0.4, zf + 0.018), M.brass, bevel=0.003)
        # the top: the turntable's well on the left, a board on the right with sleeves and a candle
        F.box('top_right', 0.45, 0.06, d, (0.225, TOP - 0.03, zc), M.walnut, bevel=0.008)
        F.box('well_front', 0.45, 0.06, 0.02, (-0.225, TOP - 0.03, zf - 0.01), M.walnut)
        F.box('well_back', 0.45, 0.06, 0.02, (-0.225, TOP - 0.03, WALL + 0.01), M.walnut)
        F.box('well_side', 0.02, 0.06, d, (-0.44, TOP - 0.03, zc), M.walnut)
        F.box('well_floor', 0.43, 0.01, d - 0.04, (-0.225, 0.565, zc), M.ink)
        F.box('plinth', 0.38, 0.03, 0.32, (-0.225, 0.585, zc), M.mauve, bevel=0.006)
        G.rod('tonearm', F.at(-0.08, 0.612, WALL + 0.06), F.at(-0.16, 0.612, zc + 0.04), 0.005, material=M.chrome, verts=6)
        F.cyl('tonearm_base', 0.018, 0.02, (-0.08, 0.605, WALL + 0.06), M.chrome, verts=10)
        for k, (m, tilt) in enumerate(((M.sleeve_a, -14), (M.sleeve_b, -9), (M.sleeve_c, -4))):
            F.box(_name('sleeve'), 0.31, 0.31, 0.01, (0.24, TOP + 0.15, WALL + 0.04 + k * 0.025), m, rot=(tilt, 0, 0))
        _candle(F, 0.34, TOP, -0.12, h=0.1, r=0.03)
        # a floor vase of dried branches at the front right
        top = _vase(F, 0.33, 0, 0.3, 1.2)
        for a, (dx, dz, h) in enumerate(((-0.04, -0.02, 0.4), (0.06, 0.02, 0.33), (0.0, 0.05, 0.36))):
            G.rod(_name('twig'), F.at(0.33, top - 0.05, 0.3), F.at(0.33 + dx, top + h, 0.3 + dz), 0.005, material=M.twig,
                  verts=5)
            F.sphere(_name('dried'), 0.03, (0.33 + dx, top + h, 0.3 + dz), M.dried, segments=8, rings=5)
    G.join(still, 'body', origin=(0, 0, 0))
    # The platter and its record, origin on the spindle: `platter_spin` turns it (33 rpm) while the record plays.
    with G.new_parts() as plat:
        F.cyl('platter', 0.13, 0.012, (-0.225, 0.606, zc), M.platter, verts=32)
        F.cyl('record', 0.125, 0.004, (-0.225, 0.614, zc), M.vinyl, verts=32)
        F.cyl('record_label', 0.04, 0.002, (-0.225, 0.617, zc), M.label, verts=20)
        F.box('label_mark', 0.012, 0.0015, 0.03, (-0.225, 0.6185, zc + 0.022), M.page)
        F.cyl('spindle', 0.004, 0.012, (-0.225, 0.62, zc), M.chrome, verts=6)
    plat = G.hinged(G.join(plat, 'platter'), F.at(-0.225, 0.606, zc))
    # The lid over the well, hinged along its back: `lid_open` lifts it.
    with G.new_parts() as lid:
        F.box('lid', 0.45, 0.025, d, (-0.225, TOP + 0.0125, zc), M.walnut, bevel=0.006)
        F.box('lid_pull', 0.08, 0.012, 0.012, (-0.225, TOP + 0.012, zf + 0.006), M.slat, bevel=0.004)   # wood: one draw
    lid = G.hinged(G.join(lid, 'lid'), F.at(-0.225, TOP + 0.0125, WALL))
    objs = G.settle(centre=False)
    G.swing(lid, 'lid_open', -80, seconds=0.9, axis='X')
    G.spin(plat, 'platter_spin', turns=-1, seconds=1.8)
    return objs


# --- the nocturne cello (1x1): a stool, the cello on its stand, a music stand --------------------------------
REST, PLAY = (0.22, 0.02, 0.36), (0.0, 0.02, 0.3)   # the end pin's foot on the stand, and played


def cello():
    F = G.GameFrame()
    with G.new_parts() as still:
        # the stool: the player's seat at 0.45, hips on the footprint's middle
        F.cyl('stool_base', 0.17, 0.03, (0, 0.015, -0.08), M.rosewood, verts=20, bevel=0.006)
        F.cyl('stool_column', 0.025, 0.36, (0, 0.21, -0.08), M.rosewood, verts=10)
        F.cyl('stool_top', 0.2, 0.04, (0, 0.4, -0.08), M.rosewood, verts=24, bevel=0.008)
        F.cyl('stool_cushion', 0.19, 0.03, (0, 0.435, -0.08), M.plum, verts=24, bevel=0.012)
        # the cello's stand: a three-footed base with a cup for the end pin, a post behind, a yoke for the neck
        x, _, z = REST
        for k in range(3):
            a = 90 + 120 * k
            F.box(_name('stand_foot'), 0.13, 0.02, 0.025, (x + 0.06 * G.cosd(a), 0.01, z - 0.08 + 0.06 * G.sind(a)),
                  M.rosewood, rot=(0, 0, a), bevel=0.004)
        F.cyl('stand_cup', 0.03, 0.03, (x, 0.015, z), M.rosewood, verts=12)
        G.rod('stand_heel', F.at(x, 0.015, z), F.at(x, 0.015, z - 0.08), 0.012, material=M.rosewood, verts=8)
        G.rod('stand_post', F.at(x, 0.01, z - 0.12), F.at(x, 1.0, z - 0.12), 0.012, material=M.rosewood, verts=8)
        G.rod('stand_arm', F.at(x, 0.98, z - 0.12), F.at(x, 0.98, z - 0.02), 0.01, material=M.rosewood, verts=8)
        for s in (-1, 1):
            F.cyl(_name('stand_fork'), 0.007, 0.05, (x + s * 0.03, 1.0, z - 0.02), M.rosewood, verts=6)
        G.rod('stand_cradle', F.at(x, 0.3, z - 0.12), F.at(x, 0.3, z - 0.07), 0.01, material=M.rosewood, verts=8)
        # the bow hung on the stand's post; an open book of music and a candle on the floor at the left
        G.rod('bow', F.at(x + 0.03, 0.25, z - 0.135), F.at(x + 0.03, 0.95, z - 0.135), 0.005, material=M.wick, verts=6)
        G.rod('bow_hair', F.at(x + 0.045, 0.27, z - 0.135), F.at(x + 0.045, 0.93, z - 0.135), 0.003, material=M.page,
              verts=4)
        for s in (-1, 1):
            F.box(_name('music_page'), 0.135, 0.008, 0.2, (-0.19 + s * 0.068, 0.006, 0.24), M.page, rot=(0, s * 3, 0))
        F.box('music_cover', 0.28, 0.004, 0.21, (-0.19, 0.002, 0.24), M.plum)
        for k in range(5):
            F.box(_name('stave'), 0.1, 0.002, 0.004, (-0.12, 0.0105, 0.18 + 0.025 * k), M.ink)
        _candle(F, -0.27, 0.0, -0.2, h=0.16, r=0.035)
    G.join(still, 'body', origin=(0, 0, 0))
    # The cello, built upright on its stand with its origin on the end pin's foot: `off_stand` moves it across
    # to between the player's knees and leans it back 14 degrees.
    x, y, z = REST
    with G.new_parts() as body:
        G.rod('end_pin', F.at(x, y, z), F.at(x, y + 0.2, z), 0.007, material=M.chrome, verts=6)
        F.sphere('lower_bout', 0.5, (x, y + 0.42, z), M.varnish, scale=(0.46, 0.17, 0.4), segments=24, rings=14)
        F.sphere('upper_bout', 0.5, (x, y + 0.72, z), M.varnish, scale=(0.36, 0.16, 0.32), segments=24, rings=14)
        F.box('waist', 0.28, 0.2, 0.15, (x, y + 0.57, z), M.varnish, bevel=0.05, bevel_segments=3)
        F.box('tailpiece', 0.07, 0.18, 0.012, (x, y + 0.42, z + 0.088), M.ebony, bevel=0.005)
        F.box('bridge', 0.09, 0.05, 0.008, (x, y + 0.53, z + 0.09), M.bridge, bevel=0.002)
        F.box('fingerboard', 0.05, 0.56, 0.02, (x, y + 0.86, z + 0.088), M.ebony, bevel=0.006)
        F.box('neck', 0.04, 0.3, 0.04, (x, y + 1.1, z + 0.06), M.varnish, bevel=0.012)
        F.box('pegbox', 0.035, 0.12, 0.05, (x, y + 1.29, z + 0.05), M.varnish, bevel=0.01)
        F.sphere('scroll', 0.035, (x, y + 1.37, z + 0.05), M.varnish, scale=(0.9, 1.2, 1), segments=12, rings=8)
        for s in (-1, 1):
            F.box(_name('f_hole'), 0.01, 0.1, 0.004, (x + s * 0.07, y + 0.56, z + 0.08), M.ebony, rot=(0, s * 8, 0))
            for k in (0, 1):
                F.cyl(_name('peg'), 0.006, 0.05, (x + s * 0.03, y + 1.26 + 0.05 * k, z + 0.05), M.ebony, rot=(0, 90, 0),
                      verts=6)
        for dx in (-0.015, -0.005, 0.005, 0.015):
            F.box(_name('string'), 0.002, 0.82, 0.002, (x + dx, y + 0.86, z + 0.1), M.string)
    body = G.join(body, 'cello', origin=F.at(*REST))
    objs = G.settle(centre=False)
    G.slide(body, 'off_stand', (-(PLAY[0] - REST[0]), -(PLAY[2] - REST[2]), 0), seconds=0.9)   # Blender axes
    G.swing(body, 'off_stand', -14, seconds=0.9, axis='X')
    return objs


# --- the candlelit writing desk (2x1): the writer's chair in the middle, the desk before them ---------------
WICK = (-0.25, 0.78 + 0.14, 0.38)   # the lit candle's wick top


def journalDesk():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('top', 1.5, 0.04, 0.26, (0, 0.76, 0.32), M.rosewood, bevel=0.01)
        # an apron only along the far edge and the ends, so the writer's hands have room under the top
        F.box('apron', 1.42, 0.13, 0.02, (0, 0.675, 0.43), M.walnut, bevel=0.004)
        for x in (-0.7, 0.7):
            F.box(_name('apron_end'), 0.02, 0.1, 0.2, (x, 0.69, 0.32), M.walnut)
        F.box('gallery', 1.5, 0.14, 0.04, (0, 0.85, 0.43), M.rosewood, bevel=0.01)
        F.box('shelf', 1.44, 0.02, 0.12, (0, 0.93, 0.4), M.walnut, bevel=0.004)
        for x in (-0.7, 0.7):
            for z in (0.22, 0.42):
                F.cone(_name('leg'), 0.025, 0.016, 0.72, (x, 0.36, z), M.walnut, verts=10)
        # on the desk: the open journal, an inkwell and quill, books, a wilted rose, candles on a dish
        F.box('journal_left', 0.2, 0.012, 0.24, (-0.1, 0.786, 0.31), M.page, rot=(0, 0, 3), bevel=0.003)
        F.box('journal_right', 0.2, 0.012, 0.24, (0.1, 0.786, 0.31), M.page, rot=(0, 0, -3), bevel=0.003)
        F.box('journal_cover', 0.42, 0.006, 0.25, (0, 0.781, 0.31), M.plum)
        for k in range(4):
            F.box(_name('line'), 0.15, 0.002, 0.004, (-0.1, 0.793, 0.25 + 0.03 * k), M.ink)
        F.cyl('inkwell', 0.03, 0.05, (0.3, 0.805, 0.32), M.inkwell, verts=12)
        G.rod('quill', F.at(0.3, 0.82, 0.32), F.at(0.36, 1.02, 0.36), 0.004, material=M.page, verts=6)
        F.sphere('quill_vane', 0.02, (0.345, 0.96, 0.35), M.page, scale=(1, 0.3, 4), rot=(10, -16, 0), segments=8, rings=6)
        y = 0.78
        for w, h, m, r in ((0.22, 0.05, M.sleeve_a, 0), (0.2, 0.04, M.sleeve_b, 10), (0.21, 0.045, M.sleeve_d, -6)):
            F.box(_name('book'), w, h, 0.15, (-0.5, y + h / 2, 0.36), m, rot=(0, 0, r), bevel=0.005)
            y += h
        top = _vase(F, 0.6, 0.78, 0.38, 0.7)
        G.rod('rose_stem', F.at(0.6, top - 0.04, 0.38), F.at(0.66, top + 0.1, 0.35), 0.004, material=M.stem, verts=5)
        F.sphere('rose', 0.03, (0.67, top + 0.08, 0.35), M.rose, scale=(1, 1, 0.8), segments=10, rings=6)
        F.cyl('dish', 0.07, 0.01, (-0.215, 0.785, 0.39), M.brass, verts=16)
        _candle(F, WICK[0], 0.79, WICK[2], h=WICK[1] - 0.79, r=0.022, lit=False)
        _candle(F, -0.18, 0.79, 0.4, h=0.09, r=0.02, lit=False)
        _candle(F, 0.5, 0.94, 0.4, h=0.12, r=0.022, lit=False)
        # the chair: a high plum-upholstered back, its seat at 0.45 under the writer
        F.box('seat', 0.48, 0.05, 0.44, (0, 0.4, -0.06), M.walnut, bevel=0.012)
        F.box('seat_pad', 0.44, 0.025, 0.4, (0, 0.4375, -0.05), M.plum, bevel=0.01)
        F.box('back_pad', 0.42, 0.62, 0.06, (0, 0.82, -0.28), M.plum, rot=(-6, 0, 0), bevel=0.025, bevel_segments=2)
        F.box('crest', 0.48, 0.06, 0.06, (0, 1.16, -0.31), M.walnut, rot=(-6, 0, 0), bevel=0.02)
        for x in (-0.21, 0.21):
            F.cone(_name('front_leg'), 0.02, 0.015, 0.4, (x, 0.2, 0.13), M.walnut, verts=10)
            F.box(_name('back_leg'), 0.045, 1.2, 0.045, (x, 0.6, -0.29), M.walnut, rot=(-6, 0, 0), bevel=0.01)
    G.join(still, 'body', origin=(0, 0, 0))
    # The flame on the tall candle, built full size on its wick and scaled to nothing there: `candle` lights it.
    wx, wy, wz = WICK
    with G.new_parts() as fl:
        F.sphere('flame_outer', 0.014, (wx, wy + 0.026, wz), M.flame, scale=(1, 1, 2.2), segments=10, rings=6)
        F.sphere('flame_core', 0.008, (wx, wy + 0.018, wz), M.candle_flame, scale=(1, 1, 1.8), segments=8, rings=5)
    flame = G.join(fl, 'flame', origin=F.at(wx, wy, wz))
    flame.scale = (0.001, 0.001, 0.001)
    objs = G.settle(centre=False)
    for axis in 'XYZ':
        G.grow(flame, 'candle', seconds=0.6, axis=axis)
    return objs


# --- the rainfall lamp (1x1): rain running down lines over a pool, a little figure under an umbrella ---------
LZ = -0.05           # the lamp's middle (depth)
POOL, CANOPY = 0.165, 1.36


def rainLamp():
    F = G.GameFrame()
    with G.new_parts() as still:
        G.revolve('base', [(0, 0), (0.21, 0), (0.21, 0.1), (0.19, 0.14), (0.17, 0.14), (0.17, 0.15), (0, 0.15)],
                  at=F.at(0, 0, LZ), material=M.slate, segments=36)
        F.cyl('pool', 0.17, 0.015, (0, POOL - 0.0075, LZ), M.pool, verts=32)
        for k in range(3):
            a = 90 + 120 * k
            G.rod(_name('post'), F.at(0.195 * G.cosd(a), 0.14, LZ + 0.195 * G.sind(a)),
                  F.at(0.195 * G.cosd(a), CANOPY + 0.01, LZ + 0.195 * G.sind(a)), 0.007, material=M.slate, verts=8)
        G.revolve('canopy', [(0, 0), (0.22, 0), (0.22, 0.02), (0.18, 0.06), (0.06, 0.09), (0.02, 0.1), (0, 0.1)],
                  at=F.at(0, CANOPY, LZ), material=M.slate, segments=36)
        F.sphere('finial', 0.025, (0, CANOPY + 0.11, LZ), M.brass, segments=10, rings=6)
        F.cyl('canopy_under', 0.2, 0.004, (0, CANOPY - 0.002, LZ), M.smoke, verts=32)
        # the rock in the pool and the little figure under an umbrella
        F.sphere('rock', 0.06, (0, POOL + 0.01, LZ), M.pebble_dark, scale=(1.3, 1.1, 0.6), segments=12, rings=7)
        F.cone('figure', 0.03, 0.014, 0.12, (0.0, POOL + 0.09, LZ), M.figure, verts=10)
        F.sphere('figure_head', 0.018, (0.0, POOL + 0.165, LZ), M.page, segments=10, rings=6)
        G.rod('umbrella_handle', F.at(0.02, POOL + 0.1, LZ + 0.01), F.at(0.02, POOL + 0.25, LZ + 0.01), 0.003,
              material=M.wick, verts=5)
        F.cone('umbrella', 0.09, 0.0, 0.045, (0.02, POOL + 0.24, LZ + 0.01), M.umbrella, verts=8)
        # pebbles round the base
        for i, (x, z, s) in enumerate(((0.3, 0.2, 1.0), (0.36, 0.08, 0.8), (-0.3, 0.22, 1.1), (0.22, 0.3, 0.7),
                                       (-0.36, 0.08, 0.9), (-0.22, 0.32, 0.6))):
            F.sphere(_name('pebble'), 0.045 * s, (x, 0.02 * s, z), (M.pebble, M.pebble_dark)[i % 2],
                     scale=(1.2, 1, 0.5), rot=(0, 0, 30 * i), segments=10, rings=6)
    G.join(still, 'body', origin=(0, 0, 0))
    # The rain: lines from the canopy to the pool, built full length and squashed to nothing under the canopy:
    # `rain` runs them down while a resident watches.
    with G.new_parts() as rain:
        for i in range(18):
            a = i * 137.5
            r = 0.1 + (i * 7 % 6) * 0.015
            x, z = r * G.cosd(a), LZ + r * G.sind(a)
            F.cyl(_name('strand'), 0.0035, CANOPY - POOL, (x, (CANOPY + POOL) / 2, z), M.rain, verts=5)
            for k in range(3):
                F.sphere(_name('drop'), 0.007, (x, POOL + 0.15 + ((i * 3 + k * 5) % 11) * 0.09, z), M.rain, scale=(1, 1, 1.6),
                         segments=6, rings=4)
    rain = G.join(rain, 'rain', origin=F.at(0, CANOPY - 0.004, LZ))
    rain.scale[2] = 0.001
    # The ripples on the pool: `ripple` swells them out and back, all the time (ambient).
    with G.new_parts() as rip:
        for i in range(4):
            G.torus(_name('ripple'), 0.06 + i * 0.03, 0.0025, at=F.at(0, POOL + 0.002, LZ), material=M.ripple,
                    major_segments=28, minor_segments=4)
    rip = G.join(rip, 'ripples', origin=F.at(0, POOL, LZ))
    objs = G.settle(centre=False)
    G.grow(rain, 'rain', seconds=0.6)
    G.squash(rip, 'ripple', (1.12, 1.12, 1), seconds=2.0, back=True)
    return objs


# --- the brooding window seat (2x1): a cushioned bench under a rain-streaked window ------------------------
FRAME_Z = WALL + 0.08   # the window wall's front


def windowSeat():
    F = G.GameFrame()
    rnd = random.Random(12)
    with G.new_parts() as still:
        F.box('bench', 1.84, 0.37, 0.58, (0, 0.2, FRAME_Z + 0.29), M.bench, bevel=0.012)
        F.box('kick', 1.8, 0.06, 0.54, (0, 0.03, FRAME_Z + 0.25), M.ink)
        for x in (-0.45, 0.45):
            F.box(_name('drawer'), 0.86, 0.24, 0.014, (x, 0.22, FRAME_Z + 0.587), M.bench_front, bevel=0.005)
            F.box(_name('pull'), 0.14, 0.018, 0.018, (x, 0.28, FRAME_Z + 0.6), M.brass, bevel=0.005)
        # the window wall: a panel, the rainy pane, its casing, mullion and transom, a sill
        F.box('wall_panel', 1.84, 1.5, 0.08, (0, 1.22, WALL + 0.04), M.frame, bevel=0.01)
        F.box('pane', 1.5, 1.04, 0.01, (0, 1.24, FRAME_Z + 0.005), M.pane)
        for y in (0.71, 1.77):
            F.box(_name('casing'), 1.62, 0.06, 0.04, (0, y, FRAME_Z + 0.02), M.frame, bevel=0.008)
        for x in (-0.78, 0.78):
            F.box(_name('casing'), 0.06, 1.12, 0.04, (x, 1.24, FRAME_Z + 0.02), M.frame, bevel=0.008)
        F.box('mullion', 0.04, 1.04, 0.03, (0, 1.24, FRAME_Z + 0.015), M.frame, bevel=0.006)
        F.box('transom', 1.5, 0.04, 0.03, (0, 1.42, FRAME_Z + 0.015), M.frame, bevel=0.006)
        F.box('sill', 1.66, 0.04, 0.12, (0, 0.7, FRAME_Z + 0.06), M.frame, bevel=0.008)
        for i in range(26):                                     # raindrops and runs on the glass
            x, y = rnd.uniform(-0.72, 0.72), rnd.uniform(0.76, 1.72)
            if abs(x) < 0.03:
                continue
            if i % 3 == 0:
                L = rnd.uniform(0.06, 0.16)
                F.box(_name('run'), 0.006, L, 0.004, (x, y - L / 2, FRAME_Z + 0.011), M.drop)
            F.sphere(_name('drop'), 0.012, (x, y, FRAME_Z + 0.011), M.drop, scale=(0.8, 0.3, 1.1), segments=6, rings=4)
        # on the sill: two candles and a mug
        _candle(F, -0.7, 0.72, FRAME_Z + 0.06, h=0.12, r=0.025)
        _candle(F, -0.62, 0.72, FRAME_Z + 0.05, h=0.08, r=0.022)
        G.revolve('mug', [(0, 0), (0.035, 0), (0.04, 0.09), (0.034, 0.09), (0.03, 0.012), (0, 0.012)],
                  at=F.at(0.72, 0.72, FRAME_Z + 0.06), material=M.mug, segments=14)
        # pillows against the window wall
        F.box('pillow_l', 0.48, 0.34, 0.13, (-0.62, 0.62, FRAME_Z + 0.1), M.pillow_a, rot=(-10, 8, 0), bevel=0.055,
              bevel_segments=3)
        F.box('pillow_r', 0.42, 0.3, 0.12, (0.66, 0.6, FRAME_Z + 0.1), M.pillow_b, rot=(-10, -6, 0), bevel=0.05,
              bevel_segments=3)
    G.join(still, 'body', origin=(0, 0, 0))
    # The seat cushion with its throw and a book left on it, joined with their origin on the cushion's underside:
    # `sink_in` presses them 2 cm while a resident broods or dozes on it.
    zc = FRAME_Z + 0.3
    with G.new_parts() as seat:
        F.box('seat_cushion', 1.8, 0.08, 0.56, (0, 0.425, zc), M.cushion, bevel=0.03, bevel_segments=3)
        F.box('throw_top', 0.6, 0.015, 0.5, (0.3, 0.472, zc + 0.02), M.throw, rot=(0, 0, 12), bevel=0.006)
        F.box('throw_drop', 0.5, 0.22, 0.015, (0.32, 0.37, zc + 0.29), M.throw, bevel=0.006)
        F.box('book', 0.16, 0.03, 0.22, (-0.25, 0.48, zc + 0.08), M.plum, rot=(0, 0, -20), bevel=0.005)
    seat = G.hinged(G.join(seat, 'seat'), F.at(0, 0.385, zc))
    objs = G.settle(centre=False)
    G.squash(seat, 'sink_in', (1, 1, 0.78), seconds=0.6)
    return objs


# --- the moonstone soaking tub (2x2): a deep round stone tub, a candle ledge behind -------------------------
TZ = 0.05            # the tub's middle (depth)


def soakingTub():
    F = G.GameFrame()
    with G.new_parts() as still:
        # the tub: up the outside, over the rim and down the inside to its floor (the placeholder's TUB profile)
        G.revolve('tub', [(0, 0), (0.66, 0), (0.735, 0.132), (0.75, 0.6), (0.735, 0.645), (0.7, 0.66), (0.655, 0.655),
                          (0.635, 0.62), (0.57, 0.13), (0.5, 0.1), (0, 0.095)],
                  at=F.at(0, 0, TZ), material=M.stone, segments=56)
        G.torus('moon_band', 0.742, 0.012, at=F.at(0, 0.52, TZ), material=M.moon, major_segments=56, minor_segments=4)
        # the ledge behind: dark wood under a stone top, candles, towels, a vase with a fern
        F.box('ledge', 1.84, 0.49, 0.3, (0, 0.245, WALL2 + 0.15), M.ledge, bevel=0.01)
        F.box('ledge_top', 1.86, 0.04, 0.32, (0, 0.51, WALL2 + 0.16), M.ledge_top, bevel=0.008)
        for i, x in enumerate((-0.78, -0.66, -0.56, 0.6, 0.72)):
            _candle(F, x, 0.53, WALL2 + 0.17 + (i % 2) * 0.04, h=0.08 + (i % 3) * 0.05, r=0.03)
        F.box('towel_a', 0.36, 0.08, 0.24, (0.2, 0.57, WALL2 + 0.16), M.towel, bevel=0.03, bevel_segments=2)
        F.box('towel_b', 0.34, 0.07, 0.23, (0.2, 0.645, WALL2 + 0.16), M.mauve, bevel=0.028, bevel_segments=2)
        top = _vase(F, -0.25, 0.53, WALL2 + 0.14, 1.3)
        for k in range(6):
            a = k * 60
            F.sphere(_name('fern'), 0.07, (-0.25 + 0.06 * G.cosd(a), top + 0.05, WALL2 + 0.14 + 0.05 * G.sind(a)), M.leaf,
                     scale=(1.6, 0.6, 0.35), rot=(0, 25, -a), segments=8, rings=5)
        for k in range(4):
            a = k * 90 + 20
            G.rod(_name('frond'), F.at(-0.25, top - 0.02, WALL2 + 0.14),
                  F.at(-0.25 + 0.07 * G.cosd(a), top + 0.13, WALL2 + 0.14 + 0.05 * G.sind(a)), 0.006, material=M.leaf, verts=5)
        # the bath mat in front
        F.box('mat', 0.8, 0.02, 0.4, (0.1, 0.01, 0.72), M.mauve, bevel=0.008)
    G.join(still, 'body', origin=(0, 0, 0))
    # The water: the tub's inside up to 0.52, a little foam on it, its origin under the tub's floor and flat
    # there at rest; `fill` raises it while a resident soaks.
    with G.new_parts() as water:
        G.revolve('water', [(0, 0), (0.565, 0), (0.615, 0.425), (0, 0.425)], at=F.at(0, 0.095, TZ), material=M.water,
                  segments=48)
        for k, (x, z, r) in enumerate(((0.25, 0.25, 0.1), (0.32, 0.12, 0.07), (-0.3, 0.05, 0.09), (-0.2, 0.3, 0.06))):
            F.sphere(_name('foam'), r, (x, 0.52, TZ + z), M.foam, scale=(1.3, 1, 0.18), segments=12, rings=6)
    water = G.join(water, 'water', origin=F.at(0, 0.09, TZ))
    water.scale[2] = 0.001
    objs = G.settle(centre=False)
    G.grow(water, 'fill', seconds=1.5)
    return objs


FAMILY = G.Family(
    'pack_gloomy', M,
    [G.Model('recordCabinet', recordCabinet, size=(0.9, 0.925, 0.814), key='model.gloomy.recordCabinet', centre=(0, -0.025),
             motions={'uses': {'listen': 'lid_open', 'newAlbum': 'lid_open'},
                      'loops': {'listen': 'platter_spin', 'newAlbum': 'platter_spin'}},
             paint=[('walnut', 'Body'), ('ink', 'Trim')]),
     G.Model('cello', cello, size=(0.78, 1.427, 0.741), key='model.gloomy.cello', centre=(0.06, 0.091),
             motions={'uses': {'practise': 'off_stand', 'lament': 'off_stand'}},
             paint=[('rosewood', 'Body'), ('plum', 'Trim'), ('varnish', 'Cello')]),
     G.Model('journalDesk', journalDesk, size=(1.5, 1.198, 0.834), key='model.gloomy.journalDesk', centre=(0, 0.043),
             motions={'uses': {'journal': 'candle', 'poetry': 'candle'}},
             paint=[('rosewood', 'Body'), ('plum', 'Trim')]),
     G.Model('rainLamp', rainLamp, size=(0.804, 1.498, 0.618), key='model.gloomy.rainLamp', centre=(0, 0.039),
             motions={'uses': {'watchRain': 'rain'}, 'idle': 'ripple'},
             paint=[('slate', 'Body'), ('umbrella', 'Trim')]),
     G.Model('windowSeat', windowSeat, size=(1.84, 1.97, 0.706), key='model.gloomy.windowSeat', centre=(0, -0.077),
             motions={'uses': {'brood': 'sink_in', 'doze': 'sink_in'}},
             paint=[('bench', 'Body'), ('cushion', 'Cushion'), ('frame', 'Trim')]),
     G.Model('soakingTub', soakingTub, size=(1.86, 0.922, 1.85), key='model.gloomy.soakingTub', centre=(0, -0.005),
             motions={'uses': {'soak': 'fill'}},
             paint=[('stone', 'Body'), ('ledge', 'Trim')])],
    comment='Blender-built gloomy pack models (tools/art/blender/families/pack_gloomy.py). Replaces the '
            'primitive-part placeholders of packs/gloomy by key; the placeholder is kept as a fallback if a file '
            'fails to load, and the materials map onto the game\'s finishes so a model costs one draw call per '
            'finish group. The cabinet\'s lid lifts and its record turns, the cello comes off its stand, a candle '
            'is lit, the rain runs down the lamp, the window seat\'s cushion sinks and the tub fills.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

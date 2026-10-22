# The gloomy pack (P7): the midnight record cabinet, the nocturne cello, the candlelit writing desk, the rainfall
# lamp, the brooding window seat and the moonstone soaking tub. Notes: pack_gloomy.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`). The cabinet's lid lifts (`lid_open`) and its platter turns while the record plays (`platter_spin`, a
# loop); the cello comes off its stand into the player's arms (`off_stand`); a candle is lit on the desk
# (`candle`); the rain runs down the lamp's lines (`rain`) over a pool that ripples all the time (`ripple`,
# ambient); the window seat's cushion sinks (`sink_in`); the tub fills (`fill`).
#
# Built with the forms and the kit (2026-10-22): swept legs, rails and tubes, lofted bodies (the cello, the vase, the
# figure's coat), outlines with holes (the cabinet's slatted door, the desk's gothic gallery, the window's sash, the
# cello's bridge), soft cushions and pillows, candles with melted tops and drips.
#
# Run:  cargo art pack_gloomy [--only cello] [--preview]
import math
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
    lamp_light=('#DDE6F2', 'satin', {'glow': '#CFE0FF'}),   # the rain lamp's light under its canopy: it glows at night
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
    pool=('#BFE3F2', 'gloss', {'glow': '#9CCBFF'}),   # the lit pool and the rain over it glow with the lamp
    rain=('#A9D8EE', 'gloss', {'glow': '#BFE4FF'}),
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
    moss=('#5F7A4E', 'satin'),       # the moss on the rain lamp's rock
)

WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
WALL2 = -0.93  # in a 2-deep one
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


def _turned(F, name, x, y, z, profile, material, segments=16):
    """A turned piece standing at game (x, y, z): `profile` [(radius, height up), ...] from its foot, revolved."""
    return G.revolve(name, profile, at=F.at(x, y, z), material=material, segments=segments)


def _lathe(F, name, x, y, z, rings, material, verts=16, square=2.0):
    """A body lofted through round rings [(radius, height up), ...] standing at game (x, y, z): a vase, a bottle."""
    bx, by, _ = F.at(x, 0, z)
    return G.loft(name, [G.ring(r, r, y + h, verts, square=square, at=(bx, by)) for r, h in rings], material=material)


def _turn(dx, dz, deg):
    """A game (x, depth) offset turned `deg` degrees as a part's Blender z turn turns it."""
    c, s = G.cosd(deg), G.sind(deg)
    return dx * c - dz * s, dx * s + dz * c


def _candle(F, x, y, z, h=0.12, r=0.025, lit=True, holder=False, drips=2, seed=0):
    """A pillar candle standing on `y`: its top melted into a hollow round the wick, wax run down its side in
    `drips`, a still flame (`lit`); `holder`: on a brass dish (its rim 3 cm out), which lifts it 6 mm."""
    if holder:
        R = r + 0.028
        _turned(F, _name('candle_dish'), x, y, z, [(0, 0), (R - 0.008, 0), (R, 0.006), (R - 0.003, 0.011), (r + 0.004, 0.006),
                                                  (0, 0.006)], M.brass, 12)
        y += 0.006
    _turned(F, _name('candle'), x, y, z, [(0, 0), (r, 0), (r, h - 0.006), (r * 0.94, h), (r * 0.55, h - 0.007), (0, h - 0.009)],
            M.wax, 12)
    rnd = random.Random(seed or int(abs(x) * 1000 + abs(z) * 300 + h * 50))
    for _ in range(drips):
        a = rnd.uniform(0, 360)
        c, s = G.cosd(a), G.sind(a)
        L = rnd.uniform(0.25, 0.55) * h
        F.sweep(_name('drip'), [(x + (r - 0.004) * c, y + h - 0.003, z + (r - 0.004) * s),
                                (x + (r + 0.001) * c, y + h - 0.008, z + (r + 0.001) * s),
                                (x + (r + 0.0015) * c, y + h - L, z + (r + 0.0015) * s)], 0.0035, M.wax, verts=6,
                scale=[0.8, 1.0, 1.35])
    F.sweep(_name('wick'), [(x, y + h - 0.009, z), (x + 0.001, y + h + 0.002, z), (x + 0.003, y + h + 0.008, z)], 0.0016,
            M.wick, verts=4)
    if lit:
        F.sphere(_name('candle_flame'), 0.008, (x + 0.003, y + h + 0.026, z), M.candle_flame, scale=(1, 1, 2.2),
                 segments=7, rings=5)
    return y + h


def _vase(F, x, y, z, s=1.0):
    """A ceramic vase: a full belly, a narrow neck, a flared lip, hollow at the mouth. Returns its top."""
    _lathe(F, _name('vase'), x, y, z, [(0.042 * s, 0), (0.06 * s, 0.025 * s), (0.07 * s, 0.075 * s), (0.064 * s, 0.12 * s),
                                       (0.042 * s, 0.158 * s), (0.026 * s, 0.18 * s), (0.028 * s, 0.192 * s),
                                       (0.034 * s, 0.2 * s), (0.026 * s, 0.198 * s), (0.02 * s, 0.17 * s)], M.vase, verts=12)
    return y + 0.2 * s


def _open_book(F, name, x, y, z, w, d, cover, page, lines=None, turn=0.0):
    """A book lying open at game (x, y, z), its spine along the depth: a cover `2w` x `d`, each half of pages rising in
    a curve from the spine and falling to its edge; `lines` (a material): five ruled lines or staves on each page."""
    F.box(f'{name}_cover', 2 * w + 0.012, 0.004, d + 0.01, (x, y + 0.002, z), cover, rot=(0, 0, turn))

    def lift(t):
        return 0.004 + 0.003 + 0.012 * (1 - (1 - t) ** 3) * (1 - 0.35 * t)
    n = 8
    for s in (-1, 1):
        top = [(s * w * k / n, lift(k / n)) for k in range(n + 1)]
        outline = [(0.0, 0.004)] + [(s * w, 0.004)] + top[::-1]
        if s < 0:
            outline = outline[::-1]
        o = F.extrude(f'{name}_pages{s}', outline, d, (x, y, z), page, smooth=False)
        o.rotation_euler[2] = math.radians(turn)
        if lines is not None:
            for k in range(5):
                lz = -d * 0.32 + d * 0.16 * k
                pts = []
                for i in range(3):
                    t = 0.12 + 0.76 * i / 2
                    dx, dz = _turn(s * w * t, lz, turn)
                    pts.append((x + dx, y + lift(t) + 0.0007, z + dz))
                F.sweep(f'{name}_line{s}{k}', pts, None, lines, profile=G.rounded_rect(0.0012, 0.0025, 0.0004, corner=1),
                        up=(0, 0, 1))


def _book(F, name, x, y, z, w, h, d, cover, turn=0.0, page=None):
    """A closed book lying flat on `y`, its spine at its -x side: boards, spine and the block of pages between."""
    page = page or M.page
    for k, (dx, dy, bw, bh, bd, m) in enumerate(((0.0, 0.0015, w, 0.003, d, cover), (0.0, h - 0.0015, w, 0.003, d, cover),
                                                  (-w / 2 + 0.002, h / 2, 0.004, h, d, cover),
                                                  (0.002, h / 2, w - 0.006, h - 0.005, d - 0.006, page))):
        ox, oz = _turn(dx, 0.0, turn)
        F.box(f'{name}{k}', bw, bh, bd, (x + ox, y + dy, z + oz), m, rot=(0, 0, turn))


def _leg(F, name, x, z, top, material, r=0.02, splay=(0.0, 0.0), foot=None, verts=10):
    """A tapered leg from `top` down to the floor, splayed by (dx, dz) at its foot; `foot`: a metal ferrule."""
    sx, sz = splay
    F.sweep(name, [(x, top, z), (x + sx * 0.45, top * 0.5, z + sz * 0.45), (x + sx, 0.0, z + sz)], r, material, verts=verts,
            scale=[1.0, 0.82, 0.58])
    if foot is not None:
        F.cyl(f'{name}_ferrule', r * 0.63, 0.03, (x + sx, 0.015, z + sz), foot, verts=6)


# --- the midnight record cabinet (1x1): a walnut console, a turntable under a hinged lid -------------------
TOP = 0.62


def recordCabinet():
    F = G.GameFrame()
    rnd = random.Random(8)
    zf = -0.01            # the console's front
    zc = (zf + WALL) / 2
    d = zf - WALL
    with G.new_parts() as still:
        # splayed, tapered mid-century legs with brass ferrules
        for x in (-0.4, 0.4):
            for z in (WALL + 0.05, zf - 0.05):
                _leg(F, _name('leg'), x, z, 0.145, M.ink, r=0.019, splay=(math.copysign(0.018, x), math.copysign(0.014, z - zc)),
                     foot=M.brass, verts=8)
        F.box('bottom', 0.9, 0.03, d, (0, 0.155, zc), M.walnut, bevel=0.006, bevel_segments=1)
        # a shallow arched apron under the front
        edge = [(-0.36 + 0.72 * i / 10, 0.0) for i in range(11)]
        edge = [(u, 0.136 - 0.024 * math.sqrt(max(0.0, 1 - (u / 0.36) ** 2))) for u, _ in edge]
        F.extrude('apron', [(0.36, 0.145), (-0.36, 0.145)] + edge, 0.018, (0, 0, zf - 0.014), M.walnut, bevel=0.003,
                  bevel_segments=1)
        F.box('deck', 0.9, 0.03, d, (0, 0.545, zc), M.walnut)                        # under the turntable and the top
        for x in (-0.435, 0.435):
            F.box(_name('side'), 0.03, TOP - 0.14, d, (x, (TOP + 0.14) / 2, zc), M.walnut, bevel=0.008, bevel_segments=1)
        # the back: dark inside the bays, walnut outside (it shows when the cabinet stands free)
        F.box('back', 0.84, 0.42, 0.004, (0, 0.35, WALL + 0.008), M.ink)
        F.box('back_outside', 0.84, 0.42, 0.006, (0, 0.35, WALL + 0.003), M.walnut)
        F.box('divider', 0.02, 0.37, d - 0.02, (0, 0.355, zc - 0.01), M.walnut, bevel=0.004, bevel_segments=1)
        # the left (game -x) bay: LP sleeves standing on end, one record pulled half out
        x = -0.415
        while x < -0.03:
            t = rnd.uniform(0.008, 0.016)
            m = getattr(M, rnd.choice(('sleeve_a', 'sleeve_b', 'sleeve_c', 'sleeve_d')))
            h = 0.31 if rnd.random() < 0.8 else 0.26
            F.box(_name('lp'), t, h, 0.31, (x + t / 2, 0.17 + h / 2, WALL + 0.025 + 0.155), m, rot=(0, rnd.uniform(-4, 4), 0))
            x += t + 0.003
        F.cyl('lp_out', 0.15, 0.003, (-0.2, 0.33, -0.1), M.vinyl, rot=(0, 90, 0), verts=24, smooth=False)
        F.cyl('lp_out_label', 0.05, 0.0035, (-0.2, 0.33, -0.1), M.label, rot=(0, 90, 0), verts=12, smooth=False)
        # the right bay: a slatted door in a walnut frame, a brass bar pull
        F.box('door_back', 0.41, 0.36, 0.012, (0.215, 0.355, zf - 0.012), M.ink)
        F.extrude('door_frame', G.rounded_rect(0.41, 0.36, 0.006, corner=2), 0.02, (0.215, 0.355, zf + 0.004), M.walnut,
                  holes=[G.rounded_rect(0.37, 0.32, 0.003, corner=1)], bevel=0.003, bevel_segments=1)
        pitch = 0.37 / 8
        for k in range(8):
            F.box(_name('door_slat'), 0.03, 0.32, 0.014, (0.03 + pitch * (k + 0.5), 0.355, zf + 0.002), M.slat)
        G.kit.pull(F, 'door_pull', 0.055, 0.4, zf + 0.014, 0.09, M.brass, r=0.005, out=0.022, upright=True, verts=6)
        # the top: the turntable's well on the left, a board on the right with sleeves and a candle
        F.box('top_right', 0.45, 0.06, d, (0.225, TOP - 0.03, zc), M.walnut, bevel=0.01, bevel_segments=1)
        F.box('well_front', 0.45, 0.06, 0.02, (-0.225, TOP - 0.03, zf - 0.01), M.walnut, bevel=0.004, bevel_segments=1)
        F.box('well_back', 0.45, 0.06, 0.02, (-0.225, TOP - 0.03, WALL + 0.01), M.walnut, bevel=0.004, bevel_segments=1)
        F.box('well_side', 0.02, 0.06, d, (-0.44, TOP - 0.03, zc), M.walnut, bevel=0.004, bevel_segments=1)
        F.box('well_floor', 0.43, 0.01, d - 0.04, (-0.225, 0.565, zc), M.ink)
        # the turntable: its plinth, speed buttons, the tonearm on its pivot with a counterweight and a headshell
        F.box('plinth', 0.38, 0.03, 0.32, (-0.225, 0.585, zc), M.mauve, bevel=0.008, bevel_segments=1)
        for k in range(2):
            F.cyl(_name('speed'), 0.008, 0.006, (-0.39 + 0.022 * k, 0.601, zf - 0.075), M.chrome, verts=8)
        _turned(F, 'tonearm_base', -0.08, 0.6, -0.36, [(0, 0), (0.022, 0), (0.022, 0.008), (0.013, 0.012), (0.009, 0.026),
                                                      (0, 0.027)], M.chrome, 10)
        G.kit.tube(F, 'tonearm', [(-0.075, 0.624, -0.39), (-0.08, 0.624, -0.36), (-0.1, 0.624, -0.28), (-0.13, 0.623, -0.205),
                                  (-0.155, 0.621, -0.172)], 0.0035, M.chrome, verts=6, round=2)
        F.cyl('counterweight', 0.012, 0.028, (-0.075, 0.624, -0.39), M.chrome, rot=(90, 0, 0), verts=12)
        F.box('headshell', 0.016, 0.005, 0.032, (-0.162, 0.6205, -0.16), M.chrome, rot=(0, 0, 28), bevel=0.0015,
              bevel_segments=1)
        F.box('cartridge', 0.01, 0.006, 0.016, (-0.164, 0.6175, -0.155), M.ink, rot=(0, 0, 28))
        F.cyl('arm_rest', 0.004, 0.024, (-0.05, 0.612, -0.24), M.chrome, verts=6)
        for k, (m, tilt) in enumerate(((M.sleeve_a, -14), (M.sleeve_b, -9), (M.sleeve_c, -4))):
            F.box(_name('sleeve'), 0.31, 0.31, 0.01, (0.24, TOP + 0.15, WALL + 0.04 + k * 0.025), m, rot=(tilt, 0, 0))
        _candle(F, 0.34, TOP, -0.12, h=0.1, r=0.03, holder=True, seed=3)
        # a floor vase of dried branches at the front right, each branching, with honesty-pod seed heads
        top = _vase(F, 0.33, 0, 0.3, 1.2)
        for a, (dx, dz, h) in enumerate(((-0.04, -0.02, 0.4), (0.06, 0.02, 0.33), (0.0, 0.05, 0.36))):
            p0, p1 = (0.33, top - 0.05, 0.3), (0.33 + dx, top + h, 0.3 + dz)
            mid = (0.33 + dx * 0.45, top + h * 0.45, 0.3 + dz * 0.45 + 0.01)
            F.sweep(_name('twig'), [p0, mid, p1], 0.005, M.twig, verts=5, resolution=2, scale=[1.0, 0.8, 0.45])
            for k, (bx, by, bz) in enumerate(((0.05, 0.08, 0.0),)):
                at = (0.33 + dx * (0.55 + 0.15 * k), top + h * (0.55 + 0.15 * k), 0.3 + dz * (0.55 + 0.15 * k) + 0.005)
                end = (at[0] + bx * (1 - 2 * (a % 2)), at[1] + by, at[2] + bz)
                F.sweep(_name('sprig'), [at, end], 0.0028, M.twig, verts=4, scale=[1.0, 0.5])
                F.cyl(_name('pod'), 0.016, 0.003, end, M.dried, rot=(90, 0, 30 * a + 50 * k), verts=7, smooth=False)
            for k in range(2):
                F.cyl(_name('pod'), 0.018 - 0.002 * k, 0.003, (p1[0] + 0.012 * (2 * k - 1), p1[1] - 0.01 * k, p1[2]), M.dried,
                      rot=(90, 0, 40 * k + 20 * a), verts=7, smooth=False)
    G.join(still, 'body', origin=(0, 0, 0))
    # The platter and its record, origin on the spindle: `platter_spin` turns it (33 rpm) while the record plays.
    with G.new_parts() as plat:
        _turned(F, 'platter', -0.225, 0.6, zc, [(0, 0), (0.131, 0), (0.131, 0.012), (0, 0.012)], M.platter, 28)
        _turned(F, 'record', -0.225, 0.6, zc, [(0, 0.012), (0.125, 0.012), (0.1255, 0.015), (0.123, 0.016), (0, 0.016)],
                M.vinyl, 28)
        F.cyl('record_label', 0.04, 0.002, (-0.225, 0.617, zc), M.label, verts=16)
        F.box('label_mark', 0.012, 0.0015, 0.03, (-0.225, 0.6185, zc + 0.022), M.page)
        F.cyl('spindle', 0.004, 0.012, (-0.225, 0.62, zc), M.chrome, verts=6)
    plat = G.hinged(G.join(plat, 'platter'), F.at(-0.225, 0.606, zc))
    # The lid over the well, hinged along its back: a walnut frame round a sunk panel, a finger lip at the front;
    # `lid_open` lifts it.
    with G.new_parts() as lid:
        F.extrude('lid', G.rounded_rect(0.45, d, 0.008, corner=2), 0.025, (-0.225, TOP + 0.0125, zc), M.walnut, plane='top',
                  holes=[G.rounded_rect(0.37, d - 0.08, 0.004, corner=1)], bevel=0.004, bevel_segments=1)
        F.box('lid_panel', 0.372, 0.017, d - 0.078, (-0.225, TOP + 0.0105, zc), M.slat, bevel=0.002, bevel_segments=1)
        F.box('lid_pull', 0.09, 0.012, 0.012, (-0.225, TOP + 0.006, zf + 0.006), M.slat, bevel=0.005, bevel_segments=1)
    lid = G.hinged(G.join(lid, 'lid'), F.at(-0.225, TOP + 0.0125, WALL))
    objs = G.settle(centre=False)
    G.swing(lid, 'lid_open', -80, seconds=0.9, axis='X')
    G.spin(plat, 'platter_spin', turns=-1, seconds=1.8)
    return objs


# --- the nocturne cello (1x1): a stool, the cello on its stand, a music stand --------------------------------
REST, PLAY = (0.22, 0.02, 0.36), (0.0, 0.02, 0.3)   # the end pin's foot on the stand, and played


def _figure8(lower, upper, n=10, passes=1):
    """An instrument's body outline (game x, height), its lower and upper bouts the ellipses `lower` and `upper`
    (middle height, half-width, half-height) meeting at a rounded waist; from the foot round (as hobbies.py's)."""
    (c1, a1, b1), (c2, a2, b2) = lower, upper

    def half(c, a, b, y):
        t = (y - c) / b
        return a * math.sqrt(max(0.0, 1 - t * t))
    lo, hi = c1, c2
    for _ in range(40):
        m = (lo + hi) / 2
        lo, hi = (m, hi) if half(c1, a1, b1, m) > half(c2, a2, b2, m) else (lo, m)
    yw = (lo + hi) / 2
    t1 = math.degrees(math.asin(max(-1.0, min(1.0, (yw - c1) / b1))))
    t2 = math.degrees(math.asin(max(-1.0, min(1.0, (yw - c2) / b2))))
    right = [(a1 * G.cosd(t), c1 + b1 * G.sind(t)) for t in (-90 + (t1 + 90) * i / n for i in range(n + 1))]
    right += [(a2 * G.cosd(t), c2 + b2 * G.sind(t)) for t in (t2 + (90 - t2) * i / n for i in range(n + 1))]
    left = [(-x, y) for x, y in reversed(right[1:-1])]
    return G.smooth_outline(right + left, passes)


def _cello(F):
    """The cello, upright with its end pin's foot at REST, its face to the front: a body lofted from its outline, the
    plates arched over the ribs, f-holes, a tailpiece and a carved bridge, the neck, fingerboard, pegbox and scroll,
    four strings."""
    x, y0, z = REST
    out = _figure8((0.43, 0.233, 0.22), (0.75, 0.177, 0.19), n=9)
    cy = 0.58
    secs = [(0.35, -0.071), (0.72, -0.066), (0.93, -0.056), (1.0, -0.045), (1.0, 0.04), (0.93, 0.052), (0.72, 0.061),
            (0.35, 0.066)]
    G.loft('cello_body', [[F.at(x + s * u, cy + s * (v - cy), z + dz) for u, v in out] for s, dz in secs],
           material=M.varnish)
    # the f-holes, between the waist's bouts
    for s in (-1, 1):
        F.sweep(_name('f_hole'), [(x + s * 0.074, 0.49, z + 0.063), (x + s * 0.062, 0.535, z + 0.064),
                                  (x + s * 0.07, 0.585, z + 0.064), (x + s * 0.059, 0.63, z + 0.063)], 0.0035, M.ebony,
                verts=6, resolution=2, scale=[1.7, 0.65, 0.65, 1.6])
    # the end pin out of its collar
    F.sweep('end_pin', [(x, y0, z), (x, 0.235, z)], 0.006, M.chrome, verts=8, scale=[0.35, 1.0])
    F.cyl('end_collar', 0.016, 0.02, (x, 0.215, z), M.ebony, verts=12, bevel=0.003, bevel_segments=1)
    # the tailpiece with its fine tuners, the carved bridge (feet, kidneys, a heart)
    tail = [(0.016, 0.0), (0.034, 0.165), (0.03, 0.195), (0.0, 0.203), (-0.03, 0.195), (-0.034, 0.165), (-0.016, 0.0)]
    F.extrude('tailpiece', G.smooth_outline(tail, 1), 0.008, (x, 0.27, z + 0.069), M.ebony, bevel=0.002, bevel_segments=1)
    for k in range(4):
        F.cyl(_name('tuner'), 0.0035, 0.008, (x - 0.018 + 0.012 * k, 0.455, z + 0.076), M.chrome, rot=(90, 0, 0), verts=6)
    arch = [(0.045, 0.0), (0.03, 0.0), (0.024, 0.008), (0.0, 0.011), (-0.024, 0.008), (-0.03, 0.0), (-0.045, 0.0),
            (-0.043, 0.012), (-0.036, 0.019), (-0.042, 0.027), (-0.03, 0.033), (0.0, 0.0355), (0.03, 0.033), (0.042, 0.027),
            (0.036, 0.019), (0.043, 0.012)]
    heart = [(0.004 * math.cos(math.tau * k / 8), 0.023 + 0.004 * math.sin(math.tau * k / 8)) for k in range(8)]
    F.extrude('bridge', arch, 0.005, (x, 0.53, z + 0.066), M.bridge, plane='top', holes=[heart])
    # the neck, its fingerboard, the nut, the pegbox with its pegs, the scroll
    prof = [(0.0225 * math.cos(math.pi * i / 8), 0.03 * math.sin(math.pi * i / 8)) for i in range(9)]
    F.sweep('neck', [(x, 0.9, z + 0.058), (x, 1.21, z + 0.058)], None, M.varnish, profile=prof, up=(1, 0, 0),
            scale=[1.08, 0.85], angle=70)
    F.sweep('fingerboard', [(x, 0.62, z + 0.083), (x, 1.205, z + 0.065)], None, M.ebony,
            profile=G.rounded_rect(0.052, 0.012, 0.004, corner=1), up=(1, 0, 0), scale=[1.0, 0.64])
    F.box('nut', 0.034, 0.006, 0.012, (x, 1.205, z + 0.068), M.bridge)
    for s in (-1, 1):
        F.box(_name('pegbox_cheek'), 0.008, 0.13, 0.034, (x + s * 0.016, 1.27, z + 0.045), M.varnish, bevel=0.003,
              bevel_segments=1)
    F.box('pegbox_back', 0.04, 0.13, 0.01, (x, 1.27, z + 0.03), M.varnish, bevel=0.003, bevel_segments=1)
    F.box('pegbox_well', 0.024, 0.12, 0.02, (x, 1.27, z + 0.045), M.ebony)
    G.kit.scroll(F, 'scroll', (x, 1.335, z + 0.045), (1.385, z + 0.04), M.varnish, turns=-1.2, end=0.22, t=0.042, w=0.02,
                 steps=14)
    for s, yk in ((-1, 1.245), (1, 1.265), (-1, 1.29), (1, 1.31)):
        F.cyl(_name('peg'), 0.005, 0.07, (x + s * 0.032, yk, z + 0.045), M.ebony, rot=(0, 90, 0), verts=8)
        F.sphere(_name('peg_head'), 0.017, (x + s * 0.07, yk, z + 0.045), M.ebony, scale=(0.35, 0.75, 1.0), segments=10,
                 rings=6)
    for k in range(4):
        dx = -0.015 + 0.01 * k
        F.sweep(_name('string'), [(x + dx * 0.9, 0.465, z + 0.078), (x + dx, 0.532, z + 0.101), (x + dx * 0.75, 1.205, z + 0.075)],
                0.0012, M.string, verts=4)


def cello():
    F = G.GameFrame()
    with G.new_parts() as still:
        # the stool: a padded round seat (top 0.45, hips on the footprint's middle) on a turned column and three
        # splayed legs with a foot ring
        sz = -0.08
        _turned(F, 'stool_top', 0, 0.38, sz, [(0, 0), (0.19, 0), (0.2, 0.012), (0.2, 0.03), (0.19, 0.04), (0, 0.04)],
                M.rosewood, 28)
        _lathe(F, 'stool_cushion', 0, 0.42, sz, [(0.186, 0.0), (0.194, 0.008), (0.19, 0.02), (0.168, 0.028), (0.1, 0.0305),
                                                 (0.0, 0.031)], M.plum, verts=28)
        G.torus('stool_piping', 0.192, 0.0035, at=F.at(0, 0.428, sz), material=M.mauve, major_segments=28, minor_segments=4)
        F.sphere('stool_button', 0.012, (0, 0.449, sz), M.mauve, scale=(1, 1, 0.45), segments=8, rings=4)
        _turned(F, 'stool_column', 0, 0.12, sz, [(0, 0), (0.035, 0), (0.035, 0.02), (0.022, 0.05), (0.028, 0.14),
                                                 (0.02, 0.22), (0.03, 0.24), (0.04, 0.26), (0, 0.26)], M.rosewood, 14)
        for k in range(3):
            a = 90 + 120 * k
            c, s = G.cosd(a), G.sind(a)
            F.sweep(_name('stool_leg'), [(0.03 * c, 0.2, sz + 0.03 * s), (0.11 * c, 0.09, sz + 0.11 * s),
                                         (0.16 * c, 0.0, sz + 0.16 * s)], 0.014, M.rosewood, verts=8, resolution=2,
                    scale=[1.0, 0.9, 0.75])
        G.torus('stool_ring', 0.105, 0.007, at=F.at(0, 0.1, sz), material=M.rosewood, major_segments=20, minor_segments=5)
        # the cello's stand: three curved feet round a hub, a cup for the end pin, a post behind with a cradle for the
        # back and a yoke round the neck
        x, _, z = REST
        hz = z - 0.08
        _turned(F, 'stand_hub', x, 0, hz, [(0, 0), (0.025, 0), (0.025, 0.04), (0.018, 0.06), (0, 0.06)], M.rosewood, 12)
        for k in range(3):
            a = 90 + 120 * k
            c, s = G.cosd(a), G.sind(a)
            F.sweep(_name('stand_foot'), [(x + 0.015 * c, 0.035, hz + 0.015 * s), (x + 0.07 * c, 0.02, hz + 0.07 * s),
                                          (x + 0.125 * c, 0.008, hz + 0.125 * s)], None, M.rosewood,
                    profile=G.rounded_rect(0.018, 0.024, 0.006, corner=1), up=(0, 0, 1), resolution=2, scale=[1.0, 0.9, 0.8])
            F.sphere(_name('stand_pad'), 0.012, (x + 0.125 * c, 0.008, hz + 0.125 * s), M.rosewood, scale=(1, 1, 0.65),
                     segments=8, rings=4)
        _turned(F, 'stand_cup', x, 0, z, [(0, 0), (0.03, 0), (0.033, 0.016), (0.03, 0.03), (0.022, 0.027), (0, 0.022)],
                M.rosewood, 14)
        G.kit.tube(F, 'stand_heel', [(x, 0.015, z - 0.02), (x, 0.03, hz)], 0.011, M.rosewood, verts=8, round=1)
        F.sweep('stand_post', [(x, 0.05, z - 0.12), (x, 1.0, z - 0.12)], 0.012, M.rosewood, verts=10, scale=[1.15, 0.8])
        F.cyl('stand_collar', 0.016, 0.03, (x, 0.62, z - 0.12), M.rosewood, verts=12, bevel=0.004, bevel_segments=1)
        G.kit.tube(F, 'stand_arm', [(x, 0.985, z - 0.12), (x, 0.985, z - 0.005)], 0.008, M.rosewood, verts=8, round=1)
        G.kit.tube(F, 'stand_yoke', G.kit.fillet([(x - 0.03, 0.985, z + 0.045), (x - 0.03, 0.985, z - 0.005),
                                                  (x + 0.03, 0.985, z - 0.005), (x + 0.03, 0.985, z + 0.045)], 0.012), 0.006,
                   M.rosewood, verts=6, round=1)
        G.kit.tube(F, 'stand_cradle', [(x, 0.3, z - 0.12), (x, 0.3, z - 0.075)], 0.009, M.rosewood, verts=8, round=1)
        F.box('stand_cradle_bar', 0.13, 0.022, 0.016, (x, 0.3, z - 0.07), M.rosewood, bevel=0.006, bevel_segments=1)
        # the bow hung on the stand's post: a cambered stick, its frog and tip, the hair
        bx, bz = x + 0.035, z - 0.135
        G.kit.tube(F, 'bow_hook', [(x, 0.95, z - 0.12), (bx, 0.95, bz + 0.005), (bx, 0.965, bz + 0.005)], 0.003, M.chrome,
                   verts=5, round=1)
        F.sweep('bow', [(bx, 0.25, bz), (bx, 0.6, bz - 0.008), (bx, 0.95, bz)], 0.0045, M.wick, verts=6, resolution=3,
                scale=[1.0, 0.85, 0.6])
        F.box('bow_frog', 0.012, 0.05, 0.026, (bx, 0.29, bz + 0.012), M.ebony, bevel=0.003, bevel_segments=1)
        F.box('bow_tip', 0.008, 0.025, 0.018, (bx, 0.945, bz + 0.008), M.page, bevel=0.002, bevel_segments=1)
        F.box('bow_hair', 0.009, 0.64, 0.0015, (bx, 0.62, bz + 0.022), M.page)
        # an open book of music on the floor, and a candle on a dish at the left
        _open_book(F, 'music', -0.188, 0.0, 0.24, 0.135, 0.2, M.plum, M.page, lines=M.ink)
        _candle(F, -0.267, 0.0, -0.2, h=0.16, r=0.035, holder=True, drips=3, seed=5)
    G.join(still, 'body', origin=(0, 0, 0))
    # The cello, built upright on its stand with its origin on the end pin's foot: `off_stand` moves it across
    # to between the player's knees and leans it back 14 degrees.
    with G.new_parts() as body:
        _cello(F)
    body = G.join(body, 'cello', origin=F.at(*REST))
    objs = G.settle(centre=False)
    G.slide(body, 'off_stand', (-(PLAY[0] - REST[0]), -(PLAY[2] - REST[2]), 0), seconds=0.9)   # Blender axes
    G.swing(body, 'off_stand', -14, seconds=0.9, axis='X')
    return objs


# --- the candlelit writing desk (2x1): the writer's chair in the middle, the desk before them ---------------
WICK = (-0.25, 0.75 + 0.14, 0.38)   # the lit candle's wick top


def _arch_hole(cx, w, y0, y1):
    """A pointed (gothic) arch opening `w` wide from height y0 to its apex at y1, centred on cx."""
    r = w * 0.8                        # each side's arc, centred across the opening
    spring = y1 - math.sqrt(r * r - (r - w / 2) ** 2)
    pts = [(cx - w / 2, y0), (cx + w / 2, y0)]
    a0 = math.acos((r - w / 2) / r)    # where each arc meets the middle
    for k in range(5):                 # up the right side from the spring to the apex
        a = a0 * k / 5
        pts.append((cx + w / 2 - r + r * math.cos(a), spring + r * math.sin(a)))
    pts.append((cx, y1))
    for k in range(4, -1, -1):
        a = a0 * k / 5
        pts.append((cx - w / 2 + r - r * math.cos(a), spring + r * math.sin(a)))
    return pts


def journalDesk():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('top', 1.5, 0.04, 0.26, (0, 0.73, 0.32), M.rosewood, bevel=0.012, bevel_segments=2)
        # cabriole legs, and an apron only along the far edge and the ends (so the writer's hands have room under the
        # top), its lower edge a row of scallops
        for x in (-0.68, 0.68):
            for z in (0.22, 0.42):
                G.kit.cabriole(F, _name('leg'), x, z, 0.712, M.walnut, math.copysign(0.025, x), math.copysign(0.014, z - 0.32),
                               r=0.021, verts=8)
        edge = [(-0.66 + 1.32 * i / 48, 0.0) for i in range(49)]
        edge = [(u, 0.625 + 0.016 * abs(math.sin(math.pi * u / 0.165))) for u, _ in edge]
        F.extrude('apron', [(0.66, 0.712), (-0.66, 0.712)] + edge, 0.02, (0, 0, 0.43), M.walnut, bevel=0.003,
                  bevel_segments=1)
        for x in (-0.69, 0.69):
            side = [(0.4 - 0.18 * i / 10, 0.64 + 0.012 * abs(math.sin(math.pi * i / 5))) for i in range(11)]
            F.extrude(_name('apron_end'), [(0.22, 0.712), (0.4, 0.712)] + side, 0.02, (x, 0, 0), M.walnut, plane='side',
                      bevel=0.003, bevel_segments=1)
        # the gallery along the far edge: a board pierced by a row of pointed arches, a moulded shelf over it on
        # scrolled brackets
        holes = [_arch_hole(-0.6 + 0.15 * k, 0.08, -0.045, 0.045) for k in range(9)]
        F.extrude('gallery', G.rounded_rect(1.5, 0.14, 0.01, corner=2), 0.03, (0, 0.82, 0.435), M.rosewood, holes=holes)
        F.box('shelf', 1.44, 0.02, 0.12, (0, 0.90, 0.4), M.walnut, bevel=0.006, bevel_segments=2)
        F.sweep('shelf_nose', [(-0.72, 0.902, 0.341), (0.72, 0.902, 0.341)], 0.011, M.walnut, verts=8)
        for x in (-0.66, 0.66):
            G.kit.scroll(F, _name('bracket'), (x, 0.888, 0.35), (0.86, 0.38), M.walnut, turns=-0.9, end=0.3, t=0.018,
                         w=0.02, steps=7)
        # on the desk: the open journal, an inkwell and quill, books, a wilted rose, candles on a dish
        _open_book(F, 'journal', 0, 0.75, 0.3, 0.2, 0.24, M.plum, M.page, lines=M.ink)
        _lathe(F, 'inkwell', 0.3, 0.75, 0.32, [(0.03, 0), (0.033, 0.004), (0.033, 0.034), (0.024, 0.044), (0.013, 0.05),
                                               (0.013, 0.058), (0.016, 0.06), (0.011, 0.06), (0.009, 0.052)],
               M.inkwell, verts=16, square=4)
        G.torus('inkwell_collar', 0.0145, 0.003, at=F.at(0.3, 0.802, 0.32), material=M.brass, major_segments=14,
                minor_segments=4)
        F.sweep('quill', [(0.3, 0.765, 0.32), (0.33, 0.88, 0.342), (0.36, 0.99, 0.36)], 0.0022, M.page, verts=5)
        F.sweep('quill_vane', [(0.307, 0.79, 0.325), (0.322, 0.85, 0.338), (0.338, 0.91, 0.349), (0.355, 0.975, 0.358)],
                None, M.page, profile=G.rounded_rect(0.0015, 0.034, 0.0007, corner=1), up=(0, 1, 0),
                scale=[0.15, 1.0, 0.95, 0.3], resolution=2)
        y = 0.75
        for k, (w, h, m, r) in enumerate(((0.22, 0.05, M.sleeve_a, 0), (0.2, 0.04, M.sleeve_b, 10), (0.21, 0.045, M.sleeve_d, -6))):
            _book(F, _name('book'), -0.5, y, 0.36, w, h, 0.15, m, turn=r)
            y += h
        top = _vase(F, 0.6, 0.75, 0.38, 0.7)
        F.sweep('rose_stem', [(0.6, top - 0.04, 0.38), (0.61, top + 0.06, 0.375), (0.64, top + 0.1, 0.365),
                              (0.665, top + 0.075, 0.352)], 0.0035, M.stem, verts=5, resolution=3)
        for k, (dx, dy, dz, s) in enumerate(((0.0, 0.0, 0.0, 1.0), (0.012, 0.004, 0.006, 0.8), (-0.01, 0.006, -0.004, 0.75),
                                             (0.004, -0.012, 0.002, 0.7))):
            F.sphere(_name('rose_petal'), 0.02 * s, (0.672 + dx, top + 0.062 + dy, 0.35 + dz), M.rose, scale=(1, 0.8, 0.75),
                     rot=(30 * k, 20, 15 * k), segments=8, rings=5)
        F.sphere('rose_leaf', 0.016, (0.62, top + 0.04, 0.378), M.stem, scale=(1.6, 0.2, 0.7), rot=(0, 0, 25), segments=6,
                 rings=4)
        for k, (px, pz, a) in enumerate(((0.69, 0.3, 20), (0.55, 0.28, -40), (0.71, 0.36, 70))):
            F.sphere(_name('fallen_petal'), 0.014, (px, 0.752, pz), M.rose, scale=(1.0, 0.75, 0.12), rot=(0, 0, a), segments=8,
                     rings=4)
        _turned(F, 'dish', -0.215, 0.75, 0.39, [(0, 0), (0.06, 0), (0.07, 0.006), (0.068, 0.012), (0.062, 0.009),
                                                (0, 0.008)], M.brass, 22)
        _candle(F, WICK[0], 0.758, WICK[2], h=WICK[1] - 0.758 - 0.008, r=0.022, lit=False, drips=3, seed=11)
        _candle(F, -0.18, 0.758, 0.4, h=0.09, r=0.02, lit=False, seed=12)
        _candle(F, 0.5, 0.91, 0.4, h=0.12, r=0.022, lit=False, seed=13)
        # the chair: a buttoned plum back in a show-wood frame under a pointed crest, sabre back legs, turned front
        # legs, an H stretcher; its seat at 0.45 under the writer
        F.box('seat', 0.48, 0.04, 0.44, (0, 0.395, -0.06), M.walnut, bevel=0.01, bevel_segments=2)
        F.soft('seat_pad', 0.44, 0.025, 0.4, (0, 0.4325, -0.05), M.plum, round=0.012, puff=0.006, flat=2)
        lean = 6.0
        F.soft('back_pad', 0.36, 0.6, 0.05, (0, 0.8, -0.288), M.plum, rot=(-lean, 0, 0), round=0.02, puff=0.016, flat=3)
        for i in range(3):
            for j in range(3):
                u, v = (i - 1) * 0.1, (j - 1) * 0.17 + 0.02
                out = 0.025 + 0.016 * (1 - (u / 0.18) ** 2) * (1 - (v / 0.3) ** 2) - 0.004
                F.sphere(_name('button'), 0.008, (u, 0.8 + v * G.cosd(lean), -0.288 - v * G.sind(lean) + out * G.cosd(lean)),
                         M.ink, scale=(1, 0.5, 1), segments=6, rings=3)
        for x in (-0.21, 0.21):
            F.sweep(_name('back_leg'), [(x, 0.0, -0.228), (x, 0.42, -0.27), (x, 1.165, -0.355)], None, M.walnut,
                    profile=G.rounded_rect(0.04, 0.042, 0.01, corner=1), up=(1, 0, 0), resolution=2, scale=[0.8, 1.0, 0.85])
            F.sphere(_name('finial'), 0.017, (x, 1.18, -0.356), M.walnut, segments=10, rings=6)
            F.sweep(_name('front_leg'), [(x, 0.39, 0.13), (x, 0.36, 0.13), (x, 0.0, 0.14)], 0.02, M.walnut, verts=10,
                    scale=[1.0, 0.85, 0.62])
            G.kit.tube(F, _name('side_stretcher'), [(x, 0.13, 0.13), (x, 0.13, -0.246)], 0.008, M.walnut, verts=6, round=1)
        G.kit.tube(F, 'cross_stretcher', [(-0.21, 0.13, -0.06), (0.21, 0.13, -0.06)], 0.008, M.walnut, verts=6, round=1)
        crest = [(x, 1.11 + 0.065 * (1 - abs(x) / 0.21) ** 1.4, -0.35 - 0.003 * (1 - abs(x) / 0.21)) for x in
                 (-0.21, -0.14, -0.07, 0.0, 0.07, 0.14, 0.21)]
        F.sweep('crest', crest, None, M.walnut, profile=G.rounded_rect(0.034, 0.03, 0.01, corner=1), up=(0, 0, 1))
        F.sweep('back_rail', [(-0.21, 0.47, -0.272), (0.21, 0.47, -0.272)], None, M.walnut,
                profile=G.rounded_rect(0.03, 0.03, 0.01, corner=1), up=(0, 0, 1))
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


def _scalloped(r, y, verts, at, dip=0.0, lobes=8, drop=0.0):
    """A ring of `verts` Blender points of radius `r` at height `y`, pulled in by `dip` (a fraction) and down by `drop`
    between `lobes` points: a scalloped eave, an umbrella's edge between its ribs."""
    out = []
    for k in range(verts):
        a = math.tau * k / verts
        f = 0.5 - 0.5 * math.cos(lobes * a)
        rr = r * (1 - dip * f)
        out.append((at[0] + rr * math.cos(a), at[1] + rr * math.sin(a), y - drop * f))
    return out


def rainLamp():
    F = G.GameFrame()
    bx, by, _ = F.at(0, 0, LZ)
    with G.new_parts() as still:
        # the base: a moulded slate bowl round the pool, its lip stepped in
        _turned(F, 'base', 0, 0, LZ, [(0, 0), (0.205, 0), (0.21, 0.008), (0.205, 0.016), (0.196, 0.022), (0.196, 0.085),
                                      (0.206, 0.092), (0.21, 0.1), (0.205, 0.12), (0.19, 0.14), (0.172, 0.142), (0.17, 0.15),
                                      (0, 0.15)], M.slate, 40)
        F.cyl('pool', 0.17, 0.015, (0, POOL - 0.0075, LZ), M.pool, verts=32)
        for k in range(3):
            a = 90 + 120 * k
            px, pz = 0.195 * G.cosd(a), LZ + 0.195 * G.sind(a)
            F.sweep(_name('post'), [(px, 0.13, pz), (px, CANOPY + 0.01, pz)], 0.0075, M.slate, verts=8, scale=[1.25, 0.9])
            F.cyl(_name('post_foot'), 0.014, 0.02, (px, 0.14, pz), M.slate, verts=10, bevel=0.004, bevel_segments=1)
            for y in (0.55, 0.95):
                F.cyl(_name('post_collar'), 0.011, 0.014, (px, y, pz), M.brass, verts=8, bevel=0.003, bevel_segments=1)
        # the canopy: a pagoda roof with a scalloped eave, its light underneath (content `light`), a brass finial
        rings = [_scalloped(0.218, CANOPY, 48, (bx, by), dip=0.05, drop=0.012),
                 _scalloped(0.22, CANOPY + 0.016, 48, (bx, by), dip=0.05, drop=0.01),
                 _scalloped(0.19, CANOPY + 0.045, 48, (bx, by), dip=0.02),
                 G.ring(0.11, 0.11, CANOPY + 0.075, 48, at=(bx, by)), G.ring(0.05, 0.05, CANOPY + 0.092, 48, at=(bx, by)),
                 G.ring(0.02, 0.02, CANOPY + 0.1, 48, at=(bx, by))]
        G.loft('canopy', rings, material=M.slate)
        _turned(F, 'finial', 0, CANOPY + 0.1, LZ, [(0, 0), (0.016, 0), (0.02, 0.008), (0.012, 0.015), (0.007, 0.02),
                                                  (0.016, 0.028), (0.006, 0.036), (0, 0.038)], M.brass, 14)
        F.cyl('canopy_under', 0.2, 0.004, (0, CANOPY - 0.002, LZ), M.lamp_light, verts=32)   # the light (content `light`)
        # the rock in the pool, a cushion of moss on it, the little figure in a coat under an umbrella
        G.boulder('rock', 0.055, at=F.at(0, POOL - 0.012, LZ), material=M.pebble_dark, scale=(1.35, 1.15, 0.58), seed=3.0,
                  noise=0.18, subdivisions=2)
        G.boulder('moss', 0.03, at=F.at(-0.04, POOL + 0.03, LZ + 0.02), material=M.moss, scale=(1.4, 1.1, 0.45), seed=5.0,
                  noise=0.25, subdivisions=1)
        fx, fz = 0.0, LZ
        _lathe(F, 'figure', fx, POOL + 0.03, fz, [(0.03, 0.0), (0.031, 0.01), (0.024, 0.05), (0.019, 0.085), (0.017, 0.1),
                                                 (0.012, 0.108), (0.008, 0.112)], M.figure, verts=12)
        F.sphere('figure_head', 0.018, (fx, POOL + 0.162, fz), M.page, segments=10, rings=6)
        F.cyl('figure_hat', 0.022, 0.006, (fx, POOL + 0.176, fz), M.umbrella, verts=12)
        F.cyl('figure_crown', 0.013, 0.014, (fx, POOL + 0.185, fz), M.umbrella, verts=10)
        ux, uz = 0.02, LZ + 0.01
        G.kit.tube(F, 'umbrella_handle', G.kit.fillet([(ux, POOL + 0.26, uz), (ux, POOL + 0.1, uz), (ux + 0.014, POOL + 0.09, uz),
                                                       (ux + 0.024, POOL + 0.1, uz)], 0.007), 0.003, M.wick, verts=5, round=1)
        ubx, uby, _ = F.at(ux, 0, uz)
        uy = POOL + 0.218
        G.loft('umbrella', [_scalloped(0.09, uy, 32, (ubx, uby), dip=0.06, drop=0.006),
                            _scalloped(0.075, uy + 0.018, 32, (ubx, uby), dip=0.03),
                            G.ring(0.04, 0.04, uy + 0.036, 32, at=(ubx, uby)), G.ring(0.006, 0.006, uy + 0.045, 32, at=(ubx, uby))],
               material=M.umbrella)
        F.cyl('umbrella_tip', 0.003, 0.016, (ux, uy + 0.05, uz), M.wick, verts=5)
        # pebbles round the base
        for i, (x, z, s) in enumerate(((0.3, 0.2, 1.0), (0.362, 0.08, 0.8), (-0.3, 0.22, 1.1), (0.22, 0.3, 0.7),
                                       (-0.358, 0.08, 0.9), (-0.22, 0.324, 0.6))):
            G.boulder(_name('pebble'), 0.04 * s, at=F.at(x, 0, z), material=(M.pebble, M.pebble_dark)[i % 2],
                      scale=(1.3, 0.95, 0.55), rot=(0, 0, 0 if i in (1, 4) else 30 * i), seed=1.3 * i, noise=0.14,
                      subdivisions=2)
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
                    major_segments=32, minor_segments=4)
    rip = G.join(rip, 'ripples', origin=F.at(0, POOL, LZ))
    objs = G.settle(centre=False)
    G.grow(rain, 'rain', seconds=0.6)
    G.squash(rip, 'ripple', (1.12, 1.12, 1), seconds=2.0, back=True)
    return objs


# --- the brooding window seat (2x1): a cushioned bench under a rain-streaked window ------------------------
FRAME_Z = WALL + 0.08   # the window wall's front


def _throw(F, name, x, w, top, z0, z1, mat, hang=0.12, seed=0.0):
    """A throw lying on a cushion's top (`top`) from depth z0 to its front edge z1, rolling over it and hanging `hang`
    down its front, with a soft wobble."""
    t = 0.012
    y = top + t / 2
    pts = [(x, y, z0), (x, y + 0.006, (z0 + z1) / 2), (x, y + 0.002, z1 - 0.035), (x, y - 0.012, z1 - 0.004),
           (x, y - 0.04, z1 + t / 2 - 0.004), (x, y - hang, z1 + t / 2 - 0.006)]
    obj = F.sweep(name, pts, None, mat, profile=G.rounded_rect(w, t, t / 2, corner=1), up=(1, 0, 0), resolution=3)
    G.jitter(obj, 0.004, size=0.12, seed=seed, up=0.6)
    return obj


def _mug(F, name, x, y, z, r, h, material, turn=0.0):
    """A mug standing on `y`: hollow, its handle out at `turn` degrees (0: +x, 90: to the front)."""
    _turned(F, name, x, y, z, [(0.0, 0.0), (0.88 * r, 0.0), (r, 0.06 * h), (r, h), (0.86 * r, h), (0.86 * r, 0.15 * h),
                               (0.0, 0.15 * h)], material, 16)
    c, s = G.cosd(turn), G.sind(turn)
    F.sweep(f'{name}_handle', [(x + c * d, y + t * h, z + s * d) for d, t in ((r - 0.003, 0.8), (r + 0.016, 0.82),
                                                                              (r + 0.024, 0.55), (r + 0.015, 0.27),
                                                                              (r - 0.003, 0.25))], 0.0045, material,
            verts=6, resolution=2)


def windowSeat():
    F = G.GameFrame()
    rnd = random.Random(12)
    with G.new_parts() as still:
        # the bench: a box on a recessed dark plinth, two panelled drawers with brass bar pulls, a rounded nosing
        F.box('bench', 1.84, 0.325, 0.58, (0, 0.2225, FRAME_Z + 0.29), M.bench, bevel=0.012, bevel_segments=2)
        F.box('kick', 1.8, 0.06, 0.54, (0, 0.03, FRAME_Z + 0.25), M.ink)
        F.sweep('bench_nose', [(-0.92, 0.372, FRAME_Z + 0.583), (0.92, 0.372, FRAME_Z + 0.583)], 0.012, M.frame, verts=10)
        for x in (-0.45, 0.45):
            G.kit.shaker(F, _name('drawer'), 0.86, 0.24, (x, 0.215, FRAME_Z + 0.587), 0.014, M.bench_front, rail=0.05)
            G.kit.pull(F, _name('pull'), x, 0.28, FRAME_Z + 0.594, 0.14, M.brass, r=0.006, out=0.03)
        # the window wall: a panel, the rainy pane behind a sash of four lights, a moulded casing, a nosed sill
        F.box('wall_panel', 1.84, 1.5, 0.08, (0, 1.22, WALL + 0.04), M.frame, bevel=0.01)
        F.box('pane', 1.5, 1.04, 0.01, (0, 1.24, FRAME_Z + 0.005), M.pane)
        lights = [G.rounded_rect(0.69, 0.58, 0.004, corner=1), G.rounded_rect(0.69, 0.3, 0.004, corner=1)]
        holes = [[(u + s * 0.365, v + dy) for u, v in lights[k]] for s in (-1, 1) for k, dy in ((0, -0.15), (1, 0.33))]
        F.extrude('sash', G.rounded_rect(1.5, 1.04, 0.004, corner=1), 0.03, (0, 1.24, FRAME_Z + 0.018), M.frame,
                  holes=holes, bevel=0.003, bevel_segments=1)
        G.kit.moulding(F, 'casing', [(0.81, 0.69), (0.81, 1.79), (-0.81, 1.79), (-0.81, 0.69)],
                       [(-0.045, 0.0), (-0.045, 0.026), (-0.02, 0.04), (0.012, 0.04), (0.012, 0.0)], M.frame, at=FRAME_Z)
        F.sweep('sill', [(-0.84, 0.7, FRAME_Z + 0.06), (0.84, 0.7, FRAME_Z + 0.06)], None, M.frame,
                profile=G.rounded_rect(0.04, 0.12, 0.014, corner=2), up=(0, 0, 1))
        F.box('sill_apron', 1.5, 0.05, 0.016, (0, 0.655, FRAME_Z + 0.008), M.frame, bevel=0.004, bevel_segments=1)
        for i in range(26):                                     # raindrops and runs on the glass
            x, y = rnd.uniform(-0.72, 0.72), rnd.uniform(0.76, 1.72)
            if abs(x) < 0.03:
                continue
            if i % 3 == 0:
                L = rnd.uniform(0.06, 0.16)
                F.sweep(_name('run'), [(x, y, FRAME_Z + 0.011), (x + 0.004, y - L * 0.5, FRAME_Z + 0.011),
                                       (x + 0.001, y - L, FRAME_Z + 0.011)], 0.003, M.drop, verts=4, resolution=2,
                        scale=[1.0, 0.7, 1.2])
            F.sphere(_name('drop'), 0.012, (x, y, FRAME_Z + 0.011), M.drop, scale=(0.8, 0.3, 1.1), segments=6, rings=4)
        # on the sill: two candles on dishes and a mug
        _candle(F, -0.7, 0.72, FRAME_Z + 0.065, h=0.12, r=0.025, holder=True, seed=21)
        _candle(F, -0.6, 0.72, FRAME_Z + 0.06, h=0.08, r=0.022, holder=True, seed=22)
        _mug(F, 'mug', 0.7, 0.72, FRAME_Z + 0.065, 0.038, 0.09, M.mug, turn=200)
        # pillows against the window wall
        F.soft('pillow_l', 0.48, 0.34, 0.12, (-0.62, 0.62, FRAME_Z + 0.1), M.pillow_a, rot=(-10, 0, 8), round=0.05,
               puff=0.04, flat=3)
        F.soft('pillow_r', 0.42, 0.3, 0.11, (0.66, 0.6, FRAME_Z + 0.1), M.pillow_b, rot=(-10, 0, -6), round=0.045,
               puff=0.035, flat=3)
    G.join(still, 'body', origin=(0, 0, 0))
    # The seat cushion with its throw and a book left on it, joined with their origin on the cushion's underside:
    # `sink_in` presses them 2 cm while a resident broods or dozes on it.
    zc = FRAME_Z + 0.3
    with G.new_parts() as seat:
        puff = 0.012
        F.soft('seat_cushion', 1.8, 0.08 - puff, 0.56, (0, 0.425 - puff / 2, zc), M.cushion, round=0.03, puff=puff,
               piping=0.006, flat=3)
        _throw(F, 'throw', 0.32, 0.56, 0.465, zc - 0.2, zc + 0.282, M.throw, hang=0.14, seed=2.0)
        _book(F, 'book', -0.25, 0.465, zc + 0.08, 0.16, 0.03, 0.22, M.plum, turn=-20)
    seat = G.hinged(G.join(seat, 'seat'), F.at(0, 0.385, zc))
    objs = G.settle(centre=False)
    G.squash(seat, 'sink_in', (1, 1, 0.78), seconds=0.6)
    return objs


# --- the moonstone soaking tub (2x2): a deep round stone tub, a candle ledge behind -------------------------
TZ = 0.05            # the tub's middle (depth)


def soakingTub():
    F = G.GameFrame()
    with G.new_parts() as still:
        # the tub: up the outside from a rounded foot, over a rolled rim and down the inside to its floor
        _turned(F, 'tub', 0, 0, TZ, [(0, 0), (0.64, 0), (0.675, 0.018), (0.735, 0.132), (0.75, 0.6), (0.742, 0.638),
                                     (0.722, 0.657), (0.7, 0.662), (0.674, 0.658), (0.652, 0.645), (0.636, 0.62), (0.57, 0.13),
                                     (0.53, 0.102), (0, 0.095)], M.stone, 56)
        G.torus('moon_band', 0.749, 0.012, at=F.at(0, 0.52, TZ), material=M.moon, major_segments=56, minor_segments=4)
        G.torus('moon_band_low', 0.705, 0.008, at=F.at(0, 0.06, TZ), material=M.moon, major_segments=48, minor_segments=3)
        # the ledge behind: dark wood with panelled ends, a bullnosed stone top; candles, towels, a vase with a fern
        F.box('ledge', 1.84, 0.49, 0.3, (0, 0.245, WALL2 + 0.15), M.ledge, bevel=0.01)
        for x in (-0.69, 0.69):
            G.kit.shaker(F, _name('ledge_panel'), 0.38, 0.36, (x, 0.26, WALL2 + 0.307), 0.016, M.ledge, rail=0.05,
                         raised=True)
        F.sweep('ledge_top', [(-0.93, 0.51, WALL2 + 0.16), (0.93, 0.51, WALL2 + 0.16)], None, M.ledge_top,
                profile=G.rounded_rect(0.04, 0.32, 0.014, corner=2), up=(0, 0, 1))
        for i, x in enumerate((-0.78, -0.66, -0.56, 0.6, 0.72)):
            _candle(F, x, 0.53, WALL2 + 0.17 + (i % 2) * 0.04, h=0.08 + (i % 3) * 0.05, r=0.03, drips=2 + i % 2, seed=30 + i)
        F.soft('towel_a', 0.36, 0.07, 0.24, (0.2, 0.567, WALL2 + 0.16), M.towel, round=0.03, puff=0.008, flat=2)
        F.soft('towel_b', 0.34, 0.06, 0.23, (0.2, 0.636, WALL2 + 0.16), M.mauve, round=0.026, puff=0.008, flat=2)
        for k, (y, w, d) in enumerate(((0.567, 0.36, 0.24), (0.636, 0.34, 0.23))):
            F.box(_name('towel_band'), w + 0.004, 0.012, 0.03, (0.2, y, WALL2 + 0.16 + d / 2 - 0.035), (M.mauve, M.towel)[k])
        top = _vase(F, -0.25, 0.53, WALL2 + 0.14, 1.3)
        for k in range(7):
            a = k * 360 / 7 + 10
            c, s = G.cosd(a), G.sind(a)
            L = 0.17 + 0.03 * (k % 3)
            base = (-0.25, top - 0.02, WALL2 + 0.14)
            sd = s * (0.7 if s > 0 else 0.4)                    # (towards the wall, less: the ledge's back)
            pts = [base, (-0.25 + 0.04 * c, top + 0.09, WALL2 + 0.14 + 0.03 * sd),
                   (-0.25 + L * 0.6 * c, top + 0.128, WALL2 + 0.14 + L * 0.6 * sd),
                   (-0.25 + L * c, top + 0.04, WALL2 + 0.14 + L * sd)]
            F.sweep(_name('frond'), pts, None, M.leaf, profile=G.rounded_rect(0.003, 0.05, 0.001, corner=1), up=(0, 0, 1),
                    resolution=2, scale=[0.15, 0.8, 1.0, 0.1])
        # the bath mat in front, a fringe at its ends
        F.soft('mat', 0.8, 0.02, 0.4, (0.1, 0.01, 0.72), M.mauve, round=0.008, round_segments=1, flat=1)
        for s in (-1, 1):
            for k in range(9):
                F.box(_name('tassel'), 0.02, 0.012, 0.012, (0.1 + s * 0.41, 0.006, 0.56 + 0.04 * k), M.mauve)
    G.join(still, 'body', origin=(0, 0, 0))
    # The water: the tub's inside up to 0.52, a little foam on it, its origin under the tub's floor and flat
    # there at rest; `fill` raises it while a resident soaks.
    with G.new_parts() as water:
        _turned(F, 'water', 0, 0.095, TZ, [(0, 0), (0.565, 0), (0.615, 0.425), (0, 0.425)], M.water, 48)
        for k, (x, z, r) in enumerate(((0.25, 0.25, 0.1), (0.32, 0.12, 0.07), (-0.3, 0.05, 0.09), (-0.2, 0.3, 0.06))):
            f = G.boulder(_name('foam'), r, at=F.at(x, 0.515, TZ + z), material=M.foam, scale=(1.3, 1.0, 0.16), seed=k * 2.3,
                          noise=0.3, subdivisions=2)
            del f
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
    comment='Blender-built gloomy pack models (tools/art/blender/families/pack_gloomy.py). Listed in the '
            'base manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw '
            'call per finish group. The cabinet\'s lid lifts and its record turns, the cello comes off its '
            'stand, a candle is lit, the rain runs down the lamp, the window seat\'s cushion sinks and the '
            'tub fills.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

# The outgoing pack (P14): the home party bar, the conversation sectional, the karaoke stage, the video-call
# corner, the light-up dance floor and the party photo wall.
# Notes: pack_outgoing.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`), with its moving parts: the bar's beer-tap lever pulls (`tap`), the sectional's seat cushions sink
# (`sink_in`), the karaoke mics rise on their stands (`mic_up`), the ring light swings up to face the caller
# (`ring_light`), the dance floor's tiles pulse up and down in a checkerboard (`tiles`, a loop) and a new
# photo grows on the board (`photo`).
#
# Upholstery is stuffed and piped (`G.soft_box`); the bar's fluted front one slab extruded from a scalloped outline,
# its cabinet doors `G.kit.shaker`, its rail and the stands' legs bent tube (`G.kit.tube`); bottles, glasses, mics,
# speaker cones and lamps turned (`G.revolve`) or lofted; the gantry's posts and beam triangular trusses swept; the
# board's frame `G.kit.moulding`; the balloons lofted, their strings and the bulbs' wire swept.
#
# Run:  cargo art pack_outgoing [--only partyBar] [--preview] [--lit]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gamekit as G  # noqa: E402
from plants import leaf  # noqa: E402  (leaf blades)

M = G.Palette(
    body=('#E8735A', 'satin'),       # coral: the bar's counter, the desk frame, the stage, the floor's frame
    upholstery=('#3FA9A5', 'fabric'),  # the sectional's cushions (fabric); see the per-model notes
    vinyl=('#E8735A', 'satin'),      # bar stool tops
    cushion=('#3FA9A5', 'satin'),    # the desk chair's seat and back
    frame=('#3FA9A5', 'fabric'),     # the sectional's frame, back and arms
    wood=('#8A5E3C', 'wood'),        # the back bar, the stage deck, the desk top, the photo board's frame
    oak=('#CDB08A', 'wood'),         # the bar's counter top and shelves
    oak_plain=('#CDB08A', 'satin'),  # the sectional's console (plain: the sofa takes the textured draw)
    brass=('#B8925A', 'metal'),
    chrome=('#D0D3D6', 'chrome'),
    steel=('#C9CCD0', 'metal'),      # the dance floor's gantry, mic bases
    dark=('#3A3D41', 'satin'),       # speaker cones, the ring light's stand
    speaker=('#E4E1DA', 'satin'),    # speaker cabinets
    screen=('#26305A', 'gloss', {'glow': '#5A78D8', 'screen': True}),   # the karaoke and laptop screens: lit while used
    lyric=('#F2F2F0', 'gloss', {'glow': '#FFFFFF', 'screen': True}),
    lyric_hi=('#F2C14E', 'gloss', {'glow': '#FFD45A', 'screen': True}),   # the line being sung
    sunny=('#F2C14E', 'satin'),
    coral=('#E8735A', 'gloss'),
    magenta=('#D64B8A', 'gloss'),
    turq=('#3FA9A5', 'gloss'),
    plum=('#6B4A7E', 'satin'),
    pouf=('#F2C14E', 'fabric'),
    cream=('#F4EDE4', 'satin'),
    pot=('#E9E6E0', 'satin'),
    leaf=('#5E8F4E', 'satin'),       # plants plain, so no foliage draw
    lime=('#9CCB4A', 'satin'),
    porcelain=('#F5F3EE', 'gloss'),
    glassy=('#DDEBEC', 'gloss'),     # glasses, plain gloss (no see-through draw)
    beer=('#E8B04A', 'gloss'),
    laptop=('#D9DADB', 'metal'),
    face=('#E8B892', 'gloss'),
    ring=('#FFF6E2', 'gloss', {'glow': '#FFF3D8', 'screen': True}),   # the ring light: on while the call lasts
    cork=('#C9A27A', 'matte'),
    paper=('#FAFAF7', 'satin'),
    bulb=('#FFE9A8', 'gloss', {'glow': '#FFDC8A'}),   # the photo wall's string of bulbs: lit at night
    string=('#3A3D41', 'satin'),
    base=('#E4E1DA', 'satin'),       # the dance floor's base
    mirror=('#E8EEF2', 'chrome'),
    tile_a=('#D64B8A', 'gloss', {'glow': '#FF5AA8', 'screen': True}),   # the floor's tiles light up while it's danced on
    tile_b=('#3FA9A5', 'gloss', {'glow': '#4FE0D8', 'screen': True}),
    tile_c=('#F2C14E', 'gloss', {'glow': '#FFD860', 'screen': True}),
    tile_d=('#8E6BD6', 'gloss', {'glow': '#B08CFF', 'screen': True}),
    tile_e=('#58C7E8', 'gloss', {'glow': '#70DCFF', 'screen': True}),
    tile_f=('#E8735A', 'gloss', {'glow': '#FF8A6A', 'screen': True}),
    bottle_a=('#2E7D5B', 'gloss'),
    bottle_b=('#B5372E', 'gloss'),
    bottle_c=('#E8B04A', 'gloss'),
    bottle_d=('#6B4A7E', 'gloss'),
    bottle_e=('#2E5B8E', 'gloss'),
    bottle_f=('#E07A2E', 'gloss'),
    wine_a=('#2E4A30', 'gloss'),
    wine_b=('#5A1F2A', 'gloss'),
    label=('#EFE5D3', 'satin'),
    photo_a=('#7FB069', 'gloss'),
    photo_b=('#8E6BD6', 'gloss'),
    photo_c=('#58C7E8', 'gloss'),
    pin_a=('#D9483B', 'gloss'),
    pin_b=('#3E8EDE', 'gloss'),
)

# By name: palette materials are made anew after each reset.
TILES = ('tile_a', 'tile_b', 'tile_c', 'tile_d', 'tile_e', 'tile_f')
BOTTLES = ('bottle_a', 'bottle_b', 'bottle_c', 'bottle_d', 'bottle_e', 'bottle_f', 'magenta', 'turq')
PHOTOS = ('coral', 'turq', 'sunny', 'magenta', 'photo_b', 'photo_a', 'photo_c', 'beer')
WALL = -0.43   # the wall's face in a 1-deep footprint, game depth
WALL2 = -0.93  # in a 2-deep one


def _m(name):
    return getattr(M, name)


def _sq(w, h=None):
    """A square bar's profile, `w` across (`h` the other way)."""
    h = w if h is None else h
    return [(-w / 2, -h / 2), (w / 2, -h / 2), (w / 2, h / 2), (-w / 2, h / 2)]


def _bottle(F, name, x, y, z, h, material, square=False):
    """A spirit bottle standing on `y`: a body (round, or square as a whisky's), its label, a shoulder, neck and cap."""
    bx, by, bz = F.at(x, y, z)
    sq = 5 if square else 2
    G.loft(name, [G.ring(0.036 * k, 0.036 * k, bz + h * t, 8, square=s, at=(bx, by)) for t, k, s in
                  ((0.0, 0.94, sq), (0.02, 1.0, sq), (0.6, 1.0, sq), (0.72, 0.7, 2), (0.8, 0.34, 2), (1.0, 0.34, 2))],
           material=material)
    G.loft(f'{name}_label', [G.ring(0.0375, 0.0375, bz + h * t, 8, square=sq, at=(bx, by)) for t in (0.22, 0.44)],
           material=M.label)
    F.cyl(f'{name}_cap', 0.014, 0.025, (x, y + h + 0.01, z), M.brass, verts=8)


def _glass(F, name, x, y, z, h=0.14, r=0.035, drink=None):
    """A stemmed cocktail glass standing on `y`, a drink in its bowl."""
    G.revolve(name, [(0, 0), (r * 0.8, 0), (r * 0.8, 0.006), (0.006, 0.012), (0.005, h * 0.5), (r, h), (r * 0.85, h),
                     (0, h * 0.55)], at=F.at(x, y, z), material=M.glassy, segments=8)
    if drink:
        F.cone(f'{name}_drink', 0.006, r * 0.8, h * 0.36, (x, y + h * 0.75, z), drink, verts=8)


def _stool(F, name, x, z, h=0.74):
    """A bar stool: a stuffed round seat on a brass ring, a column, a foot ring and a domed base."""
    bx, by, bz = F.at(x, h - 0.07, z)
    G.loft(f'{name}_top', [G.ring(0.165 * k, 0.165 * k, bz + hh, 20, at=(bx, by)) for hh, k in
                           ((0.0, 0.9), (0.006, 1.0), (0.05, 1.0), (0.066, 0.85), (0.07, 0.0))], material=M.vinyl)
    F.cyl(f'{name}_ring', 0.15, 0.025, (x, h - 0.08, z), M.brass, verts=20)
    F.sweep(f'{name}_post', [(x, 0.02, z), (x, h - 0.09, z)], 0.022, M.brass, verts=10)
    G.torus(f'{name}_foot', 0.13, 0.009, at=F.at(x, 0.28, z), material=M.brass, major_segments=18, minor_segments=5)
    for a in (0, 120, 240):                                 # the foot ring's struts
        F.sweep(f'{name}_strut{a}', [(x, 0.28, z), (x + 0.13 * G.cosd(a), 0.28, z + 0.13 * G.sind(a))], 0.006, M.brass, verts=4)
    G.revolve(f'{name}_base', [(0, 0), (0.15, 0), (0.15, 0.008), (0.1, 0.022), (0.03, 0.035), (0, 0.035)], at=F.at(x, 0, z),
              material=M.brass, segments=20)


def _speaker(F, name, x, y, z, w, h, d, cone_ys, facing=1):
    """A speaker cabinet centred on (x, y, z), its edges rounded, its turned cones (a surround, the cone, a dust cap)
    on the face towards `facing` (+1: the front)."""
    F.soft(name, w, h, d, (x, y, z), M.speaker, round=0.02, round_segments=1)
    zf = z + facing * d / 2
    for k, cy in enumerate(cone_ys):
        r = w * (0.36 if k == 0 else 0.2)
        G.revolve(f'{name}_cone{k}', [(0, -0.002), (r * 1.08, -0.002), (r * 1.08, 0.004), (r, 0.01), (r * 0.82, 0.004),
                                      (r * 0.32, -0.012), (r * 0.3, -0.006), (r * 0.2, 0.0), (0, 0.004)],
                  at=F.at(x, cy, zf), material=M.dark if k else M.coral, segments=16, rot=(90 * facing, 0, 0))


# --- the home party bar (2x1): a back bar of bottles, the front counter with a beer tap, two stools ---------
TAP = (0.24, 1.29, 0.04)   # the tap lever's pivot (x, height, depth)


def _back_bar(F):
    """The back bar against the wall: a cabinet of four raised-panel doors with brass knobs, two shelves of bottles
    and glasses on brass uprights, each shelf with a gallery rail and a strip light, a plum panel behind."""
    F.box('back_cabinet', 1.82, 0.84, 0.25, (0, 0.44, WALL + 0.125), M.wood, bevel=0.008, bevel_segments=1)
    F.box('back_plinth', 1.78, 0.03, 0.23, (0, 0.015, WALL + 0.12), M.dark)
    for x in (-0.68, -0.23, 0.23, 0.68):
        G.kit.shaker(F, f'cabinet_door{x}', 0.42, 0.7, (x, 0.44, WALL + 0.256), 0.014, M.oak, rail=0.06, raised=True)
        G.kit.knob(F, f'cabinet_knob{x}', x + (0.17 if x < 0 else -0.17), 0.55, WALL + 0.263, 0.012, M.brass, segments=8)
    F.box('back_top', 1.84, 0.03, 0.27, (0, 0.875, WALL + 0.135), M.wood, bevel=0.008, bevel_segments=2)
    for y in (1.28, 1.62):
        F.box(f'shelf{y}', 1.8, 0.03, 0.18, (0, y, WALL + 0.09), M.wood, bevel=0.005, bevel_segments=1)
        F.box(f'led{y}', 1.78, 0.012, 0.01, (0, y - 0.021, WALL + 0.17), M.sunny)
        G.kit.tube(F, f'gallery{y}', [(-0.88, y + 0.015, WALL + 0.17), (-0.88, y + 0.06, WALL + 0.17),
                                      (0.88, y + 0.06, WALL + 0.17), (0.88, y + 0.015, WALL + 0.17)], 0.005, M.brass, verts=6,
                   round=1)
    for x in (-0.89, 0.89):
        for z in (WALL + 0.02, WALL + 0.16):
            F.sweep(f'upright{x}{z}', [(x, 0.89, z), (x, 1.82, z)], 0.012, M.brass, verts=8)
    F.box('mirror_back', 1.76, 0.62, 0.008, (0, 1.36, WALL + 0.004), M.plum)
    for i, x in enumerate((-0.76, -0.64, -0.52, -0.4, 0.34, 0.46, 0.6, 0.74)):
        _bottle(F, f'bottle{i}', x, 1.635, WALL + 0.09, 0.26 + 0.04 * (i % 3), _m(BOTTLES[i]), square=i % 3 == 1)
    for x in (-0.68, -0.46, 0.42, 0.68):
        _glass(F, f'shelf_glass{x}', x, 1.295, WALL + 0.09, h=0.13)
    for i, x in enumerate((-0.1, 0.0, 0.1, 0.2)):              # wine on the cabinet's top
        bx, by, bz = F.at(x, 0.89, WALL + 0.13)
        h = 0.3 - 0.02 * (i % 2)
        G.loft(f'wine{i}', [G.ring(0.036 * k, 0.036 * k, bz + h * t, 8, at=(bx, by)) for t, k in
                            ((0.0, 0.94), (0.02, 1.0), (0.62, 1.0), (0.75, 0.42), (0.8, 0.34), (1.0, 0.34))],
               material=M.wine_a if i % 2 else M.wine_b)
        F.cyl(f'wine_foil{i}', 0.013, 0.05, (x, 0.89 + h - 0.02, WALL + 0.13), M.coral if i % 2 else M.brass, verts=8)


def _counter(F):
    """The front counter: a coral front fluted in one slab (extruded from a scalloped outline), an oak top with a
    rounded nose, a brass foot rail bent back into the counter at each end."""
    n, w, zf = 24, 1.5, 0.12
    pts = [(-w / 2, -0.12)]
    for k in range(n + 1):                                  # the scalloped face: a flute every 1/n of the width
        x = -w / 2 + k * w / n
        pts.append((x, zf))
        if k < n:
            pts.append((x + w / n / 2, zf + 0.012))
    pts.append((w / 2, -0.12))
    F.extrude('counter', pts[::-1], 0.9, (0, 0.48, 0), M.body, plane='top')
    F.box('counter_plinth', 1.48, 0.03, 0.22, (0, 0.015, 0.0), M.dark)
    F.box('counter_top', 1.62, 0.05, 0.38, (0, 0.955, 0.06), M.oak, bevel=0.018, bevel_segments=3)
    F.box('counter_cap', 1.5, 0.012, 0.24, (0, 0.936, 0.0), M.body)
    G.kit.tube(F, 'foot_rail', G.kit.fillet([(-0.72, 0.2, 0.135), (-0.72, 0.2, 0.17), (0.72, 0.2, 0.17), (0.72, 0.2, 0.135)],
                                            0.03), 0.015, M.brass, verts=10, round=1)
    for x in (-0.6, 0.6):
        F.sweep(f'rail_bracket{x}', [(x, 0.08, 0.13), (x, 0.18, 0.165), (x, 0.2, 0.17)], 0.008, M.brass, verts=6,
                resolution=2)


def _bar_top(F):
    """On the counter: the beer tower over its drip tray, a pint, two cocktails, a shaker and a bowl of limes."""
    ty = 1.01
    F.soft('drip_tray', 0.16, 0.02, 0.1, (TAP[0], ty + 0.01, 0.13), M.chrome, round=0.006, round_segments=1)
    for k in range(5):
        F.box(f'drip_slot{k}', 0.13, 0.002, 0.006, (TAP[0], ty + 0.021, 0.095 + k * 0.018), M.dark)
    G.kit.tube(F, 'tap_tower', G.kit.fillet([(TAP[0], ty, 0.0), (TAP[0], TAP[1] - 0.01, 0.0), (TAP[0], TAP[1] - 0.01, TAP[2] + 0.02)],
                                            0.03), 0.024, M.chrome, verts=12, round=1)
    G.revolve('tap_flange', [(0, 0), (0.045, 0), (0.045, 0.006), (0.028, 0.02), (0, 0.02)], at=F.at(TAP[0], ty, 0.0),
              material=M.chrome, segments=14)
    F.cone('tap_spout', 0.012, 0.007, 0.08, (TAP[0], TAP[1] - 0.06, TAP[2] + 0.03), M.chrome, verts=10)
    G.revolve('pint', [(0, 0), (0.03, 0), (0.036, 0.13), (0.038, 0.14), (0.034, 0.14), (0.032, 0.02), (0, 0.02)],
              at=F.at(TAP[0], ty + 0.02, 0.13), material=M.glassy, segments=12)
    F.cone('pint_beer', 0.027, 0.032, 0.1, (TAP[0], ty + 0.075, 0.13), M.beer, verts=12)
    G.revolve('pint_head', [(0, 0), (0.033, 0), (0.034, 0.012), (0.02, 0.022), (0, 0.024)], at=F.at(TAP[0], ty + 0.124, 0.13),
              material=M.cream, segments=12)
    _glass(F, 'cocktail_a', -0.42, ty, 0.1, drink=M.magenta)
    _glass(F, 'cocktail_b', 0.02, ty, 0.15, drink=M.sunny)
    G.revolve('shaker', [(0, 0), (0.032, 0), (0.04, 0.12), (0.042, 0.13), (0.03, 0.15), (0.022, 0.19), (0.016, 0.2), (0, 0.205)],
              at=F.at(0.5, ty, 0.04), material=M.chrome, segments=12)
    G.revolve('bowl', [(0, 0), (0.05, 0), (0.09, 0.06), (0.084, 0.06), (0.045, 0.008), (0, 0.008)],
              at=F.at(-0.15, ty, 0.04), material=M.porcelain, segments=14)
    for dx, dz, m in ((-0.03, 0.0, M.lime), (0.03, 0.02, M.lime), (0.0, -0.03, M.sunny)):
        F.sphere(f'lime{dx}{dz}', 0.03, (-0.15 + dx, ty + 0.06, 0.04 + dz), m, scale=(1, 1, 0.85), segments=8, rings=5)


def partyBar():
    F = G.GameFrame()
    with G.new_parts() as still:
        _back_bar(F)
        _counter(F)
        _bar_top(F)
        _stool(F, 'stool_l', -0.5, 0.28)
        _stool(F, 'stool_r', 0.5, 0.28)
    G.join(still, 'body', origin=(0, 0, 0))
    # The tap's lever: a tall turned handle on the faucet, its origin on the pivot: `tap` pulls it towards the front.
    with G.new_parts() as lever:
        G.revolve('lever', [(0, 0), (0.016, 0), (0.016, 0.03), (0.012, 0.035), (0.014, 0.05), (0.022, 0.16), (0.024, 0.18),
                            (0.016, 0.2), (0, 0.205)], at=F.at(*TAP), material=M.body, segments=12)
    lever = G.hinged(G.join(lever, 'lever'), F.at(*TAP))
    objs = G.settle(centre=False)
    G.swing(lever, 'tap', 40, seconds=0.5, axis='X')
    return objs


# --- the conversation sectional (2x2): an L-sofa, a console behind it, a pouf in the free corner ----------
def _panel(name, w, h, d, at, mat, round=0.05, piping=None):
    """An upholstered panel (a base, an arm, a back): firm, its edges rounded, a welt round its thin side's seams."""
    return G.GameFrame().soft(name, w, h, d, at, mat, round=round, puff=None, piping=piping, flat=2)


def conversationSectional():
    F = G.GameFrame()
    with G.new_parts() as still:
        _panel('base', 1.9, 0.28, 0.86, (0, 0.18, -0.02), M.frame, round=0.04)
        _panel('chaise_base', 0.56, 0.28, 0.54, (0.66, 0.18, 0.66), M.frame, round=0.04)
        _panel('back', 1.9, 0.4, 0.2, (0, 0.6, -0.36), M.frame, round=0.07)
        _panel('arm_l', 0.18, 0.3, 1.38, (-0.86, 0.47, 0.06), M.frame, round=0.07, piping=0.006)
        _panel('arm_r', 0.16, 0.24, 1.36, (0.86, 0.44, 0.25), M.frame, round=0.06, piping=0.006)
        for x in (-0.42, 0.3):
            F.soft(f'back_cushion{x}', 0.68, 0.36, 0.16, (x, 0.68, -0.2), M.upholstery, rot=(-12, 0, 0), round=0.05,
                   puff=0.03, piping=0.006, flat=2)
        F.soft('pillow_l', 0.34, 0.3, 0.1, (-0.6, 0.66, -0.08), M.sunny, rot=(-15, 0, -20), round=0.045, puff=0.035, flat=2, round_segments=1)
        F.soft('pillow_r', 0.32, 0.3, 0.1, (0.6, 0.66, -0.06), M.body, rot=(-15, 0, 18), round=0.045, puff=0.035, flat=2, round_segments=1)
        F.soft('pillow_c', 0.28, 0.26, 0.1, (0.74, 0.6, 0.32), M.cream, rot=(-10, 0, 80), round=0.045, puff=0.03, flat=2, round_segments=1)
        for x, z in ((-0.86, -0.4), (-0.86, 0.36), (0.86, -0.4), (0.86, 0.88), (0.42, 0.88)):
            F.sweep(f'foot{x}{z}', [(x, 0.045, z), (x, 0.0, z)], 0.025, M.brass, verts=10, scale=[1.0, 0.75])
        # the console behind it: oak top and shelf on brass legs, a lamp, a plant, books and magazines
        F.box('console_top', 1.8, 0.04, 0.32, (0, 0.82, -0.75), M.oak_plain, bevel=0.01, bevel_segments=2)
        F.box('console_shelf', 1.76, 0.03, 0.28, (0, 0.2, -0.75), M.oak_plain, bevel=0.006, bevel_segments=1)
        for x in (-0.86, 0.86):
            for z in (-0.88, -0.62):
                F.sweep(f'console_leg{x}{z}', [(x, 0.0, z), (x, 0.8, z)], 0.014, M.brass, verts=8, scale=[0.7, 1.0])
            F.sweep(f'console_stretcher{x}', [(x, 0.1, -0.88), (x, 0.1, -0.62)], 0.008, M.brass, verts=6)
        G.revolve('lamp_base', [(0, 0), (0.07, 0), (0.075, 0.01), (0.11, 0.1), (0.1, 0.16), (0.07, 0.22), (0.025, 0.27),
                                (0.022, 0.3), (0, 0.3)], at=F.at(-0.6, 0.84, -0.76), material=M.porcelain, segments=16)
        G.revolve('lamp_shade', [(0.17, 0), (0.135, 0.24), (0.13, 0.24), (0.165, 0.005)], at=F.at(-0.6, 1.18, -0.76),
                  material=M.cream, segments=24)
        G.revolve('plant_pot', [(0, 0), (0.07, 0), (0.09, 0.17), (0.094, 0.18), (0.082, 0.18), (0.078, 0.16), (0, 0.16)],
                  at=F.at(0.55, 0.84, -0.76), material=M.pot, segments=14)
        for k in range(10):
            leaf(f'plant_leaf{k}', F.at(0.55, 1.0, -0.76), k * 36 + 10, 35 + 25 * (k % 2), 0.17 - 0.04 * (k % 2), 0.09,
                 M.leaf if k % 3 else M.lime, shape='heart', droop=0.4, rows=3, ring=3)
        F.soft('book_a', 0.3, 0.06, 0.2, (0.1, 0.87, -0.76), M.body, round=0.006, round_segments=1)
        F.soft('book_b', 0.28, 0.05, 0.19, (0.1, 0.925, -0.76), M.sunny, rot=(0, 0, 8), round=0.006, round_segments=1)
        for i, m in enumerate((M.turq, M.coral, M.sunny, M.plum)):
            F.box(f'magazine{i}', 0.24, 0.012, 0.18, (0.02 * i, 0.222 + 0.013 * i, -0.75), m, rot=(0, 0, 6 * i))
        # the pouf with a tray of drinks in the free front-left corner, its top buttoned
        G.revolve('pouf', [(0, 0), (0.2, 0), (0.23, 0.06), (0.235, 0.18), (0.23, 0.3), (0.2, 0.35), (0.06, 0.36), (0, 0.355)],
                  at=F.at(-0.52, 0, 0.7), material=M.pouf, segments=22)
        G.torus('pouf_welt', 0.228, 0.007, at=F.at(-0.52, 0.18, 0.7), material=M.pouf, major_segments=22, minor_segments=4)
        G.revolve('tray', [(0, 0), (0.15, 0), (0.15, 0.025), (0.142, 0.025), (0.14, 0.008), (0, 0.008)],
                  at=F.at(-0.52, 0.355, 0.7), material=M.oak_plain, segments=18)
        for x, z, m in ((-0.46, 0.66, M.porcelain), (-0.58, 0.74, M.magenta)):
            G.revolve(f'cup{x}', [(0, 0), (0.03, 0), (0.036, 0.09), (0.032, 0.09), (0.028, 0.01), (0, 0.01)],
                      at=F.at(x, 0.363, z), material=m, segments=10)
    G.join(still, 'body', origin=(0, 0, 0))
    # The seat cushions (top 0.45, where residents sit), origin on their underside: `sink_in` presses them.
    with G.new_parts() as seats:
        for name, w, d, x, z in (('seat_l', 0.84, 0.66, -0.5, 0.06), ('seat_r', 0.64, 0.66, 0.32, 0.06),
                                 ('seat_chaise', 0.5, 0.54, 0.66, 0.65)):
            F.soft(name, w, 0.11, d, (x, 0.385 - 0.008, z), M.upholstery, round=0.045, puff=0.016, piping=0.006, flat=2)
    seats = G.hinged(G.join(seats, 'seats'), F.at(0, 0.32, 0.06))
    objs = G.settle(centre=False)
    G.squash(seats, 'sink_in', (1, 1, 0.8), seconds=0.6)
    return objs


# --- the karaoke stage (2x1): a low stage, a lyrics screen, two speakers and two mic stands -----------------
MICS = (-0.36, 0.36)
MIC_Z = 0.12
MIC_RISE = 0.2


def _tripod(F, name, x, y, z, reach, top, material):
    """A stand's three splayed legs, bent tube from a collar at `top` down to rubber feet on `y`."""
    for a in (90, 210, 330):
        fx, fz = x + reach * G.cosd(a), z + reach * G.sind(a)
        F.sweep(f'{name}{a}', [(x, top, z), (x + reach * 0.5 * G.cosd(a), y + 0.06, z + reach * 0.5 * G.sind(a)), (fx, y + 0.012, fz)],
                0.007, material, verts=5, resolution=2)
        F.sphere(f'{name}_foot{a}', 0.012, (fx, y + 0.008, fz), M.dark, scale=(1, 1, 0.7), segments=6, rings=3)
    F.cyl(f'{name}_collar', 0.016, 0.03, (x, top, z), M.dark, verts=10)


def karaokeStage():
    F = G.GameFrame()
    zc, d = (WALL + 0.45) / 2, 0.45 - WALL
    with G.new_parts() as still:
        F.box('stage', 1.9, 0.12, d, (0, 0.06, zc), M.body, bevel=0.012, bevel_segments=2)
        for k in range(8):                                          # the deck's boards
            F.box(f'deck{k}', 1.86, 0.016, (d - 0.04) / 8 - 0.004, (0, 0.128, zc - (d - 0.04) / 2 + (k + 0.5) * (d - 0.04) / 8),
                  M.wood, bevel=0.003, bevel_segments=1)
        F.box('light_strip', 1.86, 0.022, 0.012, (0, 0.07, 0.448), M.magenta, bevel=0.004, bevel_segments=1)
        # the lyrics screen on its stand at the back: a slim bezel, the lyrics, a column on a foot
        F.soft('screen_case', 0.96, 0.56, 0.05, (0, 1.35, -0.34), M.speaker, round=0.015, round_segments=1)
        F.box('screen', 0.9, 0.5, 0.004, (0, 1.35, -0.314), M.screen)
        for i, (w, y) in enumerate(((0.6, 1.47), (0.7, 1.38), (0.5, 1.29), (0.62, 1.2))):
            F.box(f'lyric{i}', w, 0.032, 0.003, (0, y, -0.311), M.lyric_hi if i == 1 else M.lyric)
        F.box('screen_post', 0.05, 1.0, 0.05, (0, 0.6, -0.39), M.steel, bevel=0.01, bevel_segments=1)
        F.soft('screen_foot', 0.5, 0.03, 0.16, (0, 0.151, -0.348), M.steel, round=0.012, round_segments=1)
        for s in (-1, 1):
            _speaker(F, f'speaker{s}', s * 0.74, 0.47, -0.24, 0.34, 0.64, 0.3, (0.38, 0.62))
            # floor spots at the stage's front corners, angled up at the singers, in their yokes
            G.revolve(f'spot{s}', [(0, -0.07), (0.05, -0.07), (0.06, 0.03), (0.066, 0.05), (0.058, 0.05), (0, 0.04)],
                      at=F.at(s * 0.83, 0.23, 0.33), material=M.dark, segments=14, rot=(-55, 0, 0))
            F.cyl(f'spot_lens{s}', 0.056, 0.006, (s * 0.83, 0.262, 0.374), M.sunny, rot=(-55, 0, 0), verts=14)
            F.sweep(f'spot_yoke{s}', [(s * 0.83 - 0.07, 0.23, 0.33), (s * 0.83 - 0.07, 0.15, 0.33), (s * 0.83 + 0.07, 0.15, 0.33),
                                      (s * 0.83 + 0.07, 0.23, 0.33)], None, M.dark, profile=_sq(0.008, 0.025), up=(0, 1, 0))
        for x in MICS:                                  # the mic stands' tripods and lower tubes
            _tripod(F, f'mic_leg{x}', x, 0.136, MIC_Z, 0.16, 0.34, M.steel)
            F.sweep(f'mic_tube{x}', [(x, 0.32, MIC_Z), (x, 1.03, MIC_Z)], 0.013, M.steel, verts=10)
            F.cyl(f'mic_clutch{x}', 0.02, 0.04, (x, 1.04, MIC_Z), M.dark, verts=10)
    G.join(still, 'body', origin=(0, 0, 0))
    # The upper tubes, booms and mics of both stands, lowered at rest: `mic_up` raises them 20 cm.
    with G.new_parts() as mics:
        for i, x in enumerate(MICS):
            top = 1.33 - MIC_RISE
            F.sweep(f'mic_upper{x}', [(x, top - 0.46, MIC_Z), (x, top, MIC_Z)], 0.009, M.chrome, verts=8)
            F.sweep(f'mic_boom{x}', [(x, top, MIC_Z), (x, top + 0.05, MIC_Z + 0.04), (x, top + 0.07, MIC_Z + 0.08)], 0.008,
                    M.chrome, verts=6, resolution=2)
            G.revolve(f'mic_body{x}', [(0, 0), (0.012, 0), (0.018, 0.08), (0.022, 0.09), (0, 0.092)],
                      at=F.at(x, top + 0.06, MIC_Z + 0.07), material=M.magenta if i == 0 else M.turq, segments=10, rot=(45, 0, 0))
            F.sphere(f'mic_head{x}', 0.034, (x, top + 0.135, MIC_Z + 0.145), M.steel, segments=10, rings=6)
    mics = G.hinged(G.join(mics, 'mics'), F.at(0, 1.04, MIC_Z))
    objs = G.settle(centre=False)
    G.slide(mics, 'mic_up', (0, 0, MIC_RISE), seconds=0.8)
    return objs


# --- the video-call corner (1x1): the caller sits in the middle facing the desk, a ring light beside the laptop
RING = (0.27, 1.21, 0.34)   # the ring light's middle and pivot (x, height, depth)
RING_R = 0.11
RING_TILT = 60              # tipped back at rest, its face to the ceiling


def videoCallNook():
    F = G.GameFrame()
    x, y, z = RING
    with G.new_parts() as still:
        F.box('desk_top', 0.84, 0.03, 0.27, (0, 0.74, 0.31), M.wood, bevel=0.01, bevel_segments=2)
        F.box('desk_apron', 0.78, 0.05, 0.22, (0, 0.7, 0.31), M.body, bevel=0.004, bevel_segments=1)
        for sx in (-0.37, 0.37):                                   # hairpin legs: a bent rod each pair of feet
            F.sweep(f'hairpin{sx}', G.kit.fillet([(sx, 0.675, 0.22), (sx, 0.007, 0.25), (sx, 0.007, 0.37), (sx, 0.675, 0.4)], 0.03),
                    0.007, M.body, verts=6)
        # the laptop, open towards the caller (-depth), a friend on its screen
        F.soft('laptop_base', 0.34, 0.015, 0.23, (0, 0.763, 0.29), M.laptop, round=0.006, round_segments=1)
        F.box('laptop_keys', 0.29, 0.002, 0.1, (0, 0.771, 0.3), M.dark)
        F.box('laptop_pad', 0.1, 0.002, 0.06, (0, 0.771, 0.22), M.porcelain)
        F.soft('laptop_lid', 0.34, 0.22, 0.012, (0, 0.88, 0.41), M.laptop, rot=(-12, 0, 0), round=0.006, round_segments=1)
        F.box('laptop_screen', 0.31, 0.19, 0.003, (0, 0.88, 0.402), M.screen, rot=(-12, 0, 0))
        F.sphere('friend', 0.045, (0, 0.875, 0.398), M.face, scale=(1, 0.08, 1.1), rot=(-12, 0, 0), segments=10, rings=6)
        # the ring light's stand: a weighted foot, a pole and the yoke the ring turns in
        G.revolve('ring_foot', [(0, 0), (0.06, 0), (0.06, 0.008), (0.04, 0.016), (0.012, 0.02), (0, 0.02)],
                  at=F.at(x, 0.755, z), material=M.dark, segments=14)
        F.sweep('ring_pole', [(x, 0.77, z), (x, y - RING_R - 0.02, z)], 0.009, M.dark, verts=8)
        yoke = RING_R + 0.03
        G.kit.tube(F, 'yoke', G.kit.fillet([(x - yoke, y, z), (x - yoke, y - RING_R - 0.02, z), (x + yoke, y - RING_R - 0.02, z),
                                            (x + yoke, y, z)], 0.03), 0.008, M.dark, verts=6, round=1)
        # a mug, a notebook with a pen, a little plant
        G.revolve('mug', [(0, 0), (0.034, 0), (0.037, 0.09), (0.032, 0.09), (0.03, 0.01), (0, 0.01)],
                  at=F.at(-0.28, 0.755, 0.24), material=M.coral, segments=14)
        F.sweep('mug_handle', [(-0.244, 0.82, 0.24), (-0.222, 0.81, 0.24), (-0.222, 0.785, 0.24), (-0.244, 0.775, 0.24)], 0.006,
                M.coral, verts=5, resolution=2)
        F.soft('notebook', 0.14, 0.012, 0.2, (-0.3, 0.761, 0.33), M.sunny, rot=(0, 0, -10), round=0.004, round_segments=1)
        F.sweep('pen', [(-0.25, 0.769, 0.28), (-0.24, 0.769, 0.42)], 0.004, M.dark, verts=5)
        G.revolve('plant_pot', [(0, 0), (0.04, 0), (0.055, 0.1), (0.05, 0.1), (0, 0.09)], at=F.at(-0.33, 0.755, 0.4),
                  material=M.pot, segments=12)
        for k in range(7):
            leaf(f'plant_leaf{k}', F.at(-0.33, 0.845, 0.4), k * 51, 45 + 20 * (k % 2), 0.08, 0.05, M.leaf, droop=0.35, rows=3,
                 ring=3)
        # the chair: a moulded shell (seat top 0.45, centred on the caller) on four oak legs, a padded seat
        F.soft('chair_seat', 0.44, 0.06, 0.42, (0, 0.415, -0.02), M.cushion, round=0.025, puff=0.008, flat=2)
        bx, by, bz = F.at(0, 0.36, -0.02)
        G.loft('chair_shell', [[(bx + px, by + py, bz + h) for px, py in pts] for h, pts in (
            (0.0, [(0.21, -0.2), (0.21, 0.2), (-0.21, 0.2), (-0.21, -0.2)]),
            (0.025, [(0.22, -0.21), (0.22, 0.21), (-0.22, 0.21), (-0.22, -0.21)]))], material=M.wood, smooth=False)
        F.soft('chair_back', 0.44, 0.36, 0.06, (0, 0.7, -0.26), M.cushion, rot=(-8, 0, 0), round=0.028, puff=0.008, flat=2)
        for lx in (-0.18, 0.18):
            for lz, dz in ((0.15, 0.03), (-0.19, -0.03)):
                F.sweep(f'chair_leg{lx}{lz}', [(lx, 0.36, lz), (lx * 1.1, 0.0, lz + dz)], 0.016, M.wood, verts=8, scale=[1.0, 0.7])
            F.sweep(f'back_post{lx}', [(lx, 0.36, -0.22), (lx, 0.6, -0.255)], 0.012, M.wood, verts=6)
    G.join(still, 'body', origin=(0, 0, 0))
    # The ring and its phone clip, built facing the caller, pivoting on the yoke: tipped back to rest, then
    # `ring_light` swings it up to face them.
    with G.new_parts() as ring:
        G.torus('ring', RING_R, 0.018, at=F.at(x, y, z), material=M.ring, rot=(90, 0, 0), major_segments=28,
                minor_segments=8)
        G.torus('ring_back', RING_R, 0.02, at=F.at(x, y, z + 0.008), material=M.dark, rot=(90, 0, 0),
                major_segments=28, minor_segments=6)
        F.sweep('phone_arm', [(x, y - RING_R, z), (x, y, z)], 0.005, M.dark, verts=6)
        F.soft('phone', 0.06, 0.11, 0.008, (x, y, z - 0.006), M.dark, round=0.004, round_segments=1)
        F.box('phone_screen', 0.052, 0.1, 0.002, (x, y, z - 0.011), M.screen)
        for s in (-1, 1):
            F.sweep(f'pivot{s}', [(x + s * RING_R, y, z), (x + s * yoke, y, z)], 0.008, M.dark, verts=8)
            F.cyl(f'pivot_knob{s}', 0.014, 0.012, (x + s * (yoke - 0.014), y, z), M.dark, rot=(0, 90, 0), verts=10)
    ring = G.hinged(G.join(ring, 'ring'), F.at(x, y, z))
    G.turn(ring, RING_TILT, 'X')
    objs = G.settle(centre=False)
    G.swing(ring, 'ring_light', -RING_TILT, seconds=0.7, axis='X')
    return objs


# --- the light-up dance floor (2x2): sixteen glowing tiles, a mirror ball on a truss gantry, two speaker towers ---
TILE, PITCH, TILE_Y = 0.43, 0.45, 0.06
LIFT = 0.025


def _truss(F, name, a, b, w, material, bays=8):
    """A triangular box truss from game point `a` to `b`, `w` across: three chords and zigzag lacing on each face."""
    a, b = [list(p) for p in (a, b)]
    d = [b[i] - a[i] for i in range(3)]
    length = math.sqrt(sum(c * c for c in d))
    u = [c / length for c in d]
    # two directions square to the run: one level (across), one the cross of the two
    side = [-u[2], 0.0, u[0]] if abs(u[1]) < 0.9 else [1.0, 0.0, 0.0]
    n = math.sqrt(sum(c * c for c in side))
    side = [c / n for c in side]
    up = [u[1] * side[2] - u[2] * side[1], u[2] * side[0] - u[0] * side[2], u[0] * side[1] - u[1] * side[0]]
    r = w / math.sqrt(3)
    corners = [[r * math.cos(math.radians(90 + 120 * k)) * s + r * math.sin(math.radians(90 + 120 * k)) * t
                for s, t in zip(side, up)] for k in range(3)]
    for k, c in enumerate(corners):
        F.sweep(f'{name}_chord{k}', [[a[i] + c[i] for i in range(3)], [b[i] + c[i] for i in range(3)]], 0.012, material, verts=6)
    for k in range(3):
        c0, c1 = corners[k], corners[(k + 1) % 3]
        pts = []
        for j in range(bays + 1):
            t = j / bays
            c = c0 if j % 2 == 0 else c1
            pts.append([a[i] + d[i] * t + c[i] for i in range(3)])
        F.sweep(f'{name}_lace{k}', pts, 0.005, material, verts=4)


def danceFloor():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('base', 1.88, 0.06, 1.88, (0, 0.03, 0), M.base, bevel=0.01, bevel_segments=1)
        G.kit.moulding(F, 'edge', [(-0.94, -0.94), (0.94, -0.94), (0.94, 0.94), (-0.94, 0.94)],
                       [(0, 0), (0.03, 0), (0.03, 0.05), (0.01, 0.068), (0, 0.068)], M.steel, at=0.0, plane='top')
        F.box('grid', 1.82, 0.004, 1.82, (0, 0.062, 0), M.dark)
        for s in (-1, 1):
            _truss(F, f'post{s}', (s * 0.9, 0.06, 0.0), (s * 0.9, 2.29, 0.0), 0.09, M.steel, bays=12)
            F.soft(f'post_foot{s}', 0.14, 0.03, 0.3, (s * 0.9, 0.075, 0.0), M.steel, round=0.01, round_segments=1)
            _speaker(F, f'speaker{s}', s * 0.78, 0.5, -0.8, 0.24, 0.88, 0.24, (0.62, 0.84))
            # a moving-head light on the beam, angled down onto the floor
            G.revolve(f'spot{s}', [(0, -0.07), (0.05, -0.07), (0.06, 0.04), (0.068, 0.06), (0.06, 0.06), (0, 0.05)],
                      at=F.at(s * 0.68, 2.18, 0.0), material=M.dark, segments=14, rot=(0, s * 140, 0))
            F.cyl(f'spot_lens{s}', 0.058, 0.006, (s * 0.642, 2.135, 0.0), M.magenta if s < 0 else M.turq, rot=(0, s * 40, 0),
                  verts=14)
            F.sweep(f'spot_hanger{s}', [(s * 0.68, 2.27, 0.0), (s * 0.68, 2.22, 0.0)], 0.01, M.dark, verts=6)
        _truss(F, 'beam', (-0.95, 2.29, 0.0), (0.95, 2.29, 0.0), 0.09, M.steel, bays=12)
        F.sweep('ball_cord', [(0, 2.25, 0.0), (0, 2.05, 0.0)], 0.004, M.chrome, verts=5)
        G.revolve('ball_motor', [(0, 0), (0.02, 0), (0.02, 0.03), (0, 0.03)], at=F.at(0, 2.03, 0.0), material=M.dark, segments=8)
        F.sphere('mirror_ball', 0.17, (0, 1.86, 0.0), M.mirror, segments=16, rings=10, smooth=False)
    G.join(still, 'body', origin=(0, 0, 0))
    # The tiles in two checkerboard parts, origins on the floor: `tiles` lifts one set, then the other.
    groups = []
    for parity in (0, 1):
        with G.new_parts() as tiles:
            for i in range(4):
                for j in range(4):
                    if (i + j) % 2 != parity:
                        continue
                    m = _m(TILES[(i * 3 + j * 2) % len(TILES)])
                    x, z = -1.5 * PITCH + PITCH * i, -1.5 * PITCH + PITCH * j
                    F.box(f'tile{i}{j}', TILE, 0.014, TILE, (x, TILE_Y + 0.007, z), m, bevel=0.005, bevel_segments=1)
        groups.append(G.hinged(G.join(tiles, f'tiles_{"ab"[parity]}'), F.at(0, TILE_Y, 0)))
    objs = G.settle(centre=False)
    G.turns(groups, 'tiles', (0, 0, LIFT), seconds=1.2)
    return objs


# --- the party photo wall (1x1): a cork board on legs, snapshots, string lights, balloons, a gift ---------
BOARD_Z = WALL + 0.07
NEW_PHOTO = (0.25, 0.88, 4)   # (x, height, tilt) of the photo that grows


def _photo(F, name, x, y, z, tilt, material, pin):
    """A snapshot pinned to the board: a white print, its picture, a round-headed pin."""
    F.box(name, 0.17, 0.2, 0.004, (x, y, z), M.paper, rot=(0, tilt, 0), bevel=0.0015, bevel_segments=1)
    F.box(f'{name}_picture', 0.14, 0.13, 0.002, (x - 0.02 * math.sin(math.radians(tilt)), y + 0.02, z + 0.003), material,
          rot=(0, tilt, 0))
    px, py = x - 0.09 * math.sin(math.radians(tilt)), y + 0.09
    G.revolve(f'{name}_pin', [(0, 0), (0.003, 0), (0.003, 0.006), (0.012, 0.008), (0.012, 0.014), (0.006, 0.02), (0, 0.021)],
              at=F.at(px, py, z), material=pin, segments=8, rot=(90, 0, 0))


def _balloon(F, name, x, y, z, r, material, tie):
    """A balloon: lofted, rounder at the top and narrowing to its knot; its string swept in a loose curl to `tie`."""
    bx, by, bz = F.at(x, y, z)
    rows = [(-1.15, 0.0), (-1.1, 0.12), (-0.95, 0.42), (-0.6, 0.78), (0.0, 1.0), (0.55, 0.92), (0.9, 0.6), (1.0, 0.0)]
    G.loft(name, [G.ring(r * k, r * k, bz + r * h, 12, at=(bx, by)) for h, k in rows], material=material)
    F.cone(f'{name}_knot', 0.012, 0.004, 0.02, (x, y - r * 1.2, z), material, verts=6)
    tx, ty, tz = tie
    pts = []
    for k in range(7):
        t = k / 6
        pts.append((x + (tx - x) * t + 0.02 * math.sin(t * 9), y - r * 1.22 + (ty - y + r * 1.22) * t, z + (tz - z) * t +
                    0.02 * math.cos(t * 9)))
    F.sweep(f'{name}_string', pts, 0.0018, M.paper, verts=3, resolution=2)


def photoWall():
    F = G.GameFrame()
    zb = BOARD_Z
    with G.new_parts() as still:
        F.box('cork', 0.82, 1.0, 0.03, (0, 1.2, zb), M.cork)
        G.kit.moulding(F, 'board_frame', [(-0.44, 0.685), (0.44, 0.685), (0.44, 1.715), (-0.44, 1.715)],
                       [(0, -0.03), (0.05, -0.03), (0.05, 0.02), (0.035, 0.034), (0.008, 0.034), (0, 0.02)], M.wood, at=zb)
        for x in (-0.42, 0.42):
            F.box(f'leg{x}', 0.05, 1.74, 0.05, (x, 0.87, zb), M.wood, bevel=0.008, bevel_segments=1)
            F.soft(f'foot{x}', 0.06, 0.04, 0.34, (x, 0.02, WALL + 0.17), M.wood, round=0.012, round_segments=1)
        spots = ((-0.27, 1.45, 6), (0.0, 1.5, -4), (0.26, 1.44, 8), (-0.24, 1.15, -7), (0.03, 1.18, 3),
                 (0.28, 1.12, -5), (-0.25, 0.86, 4), (0.0, 0.88, -8))
        for i, (x, y, r) in enumerate(spots):
            _photo(F, f'photo{i}', x, y, zb + 0.017, r, _m(PHOTOS[i]), (M.pin_a, M.pin_b, M.sunny)[i % 3])
        # string lights draped across the top: one wire in a sag, a socket and a bulb at each dip
        pts = [(-0.42 + 0.84 * t / 16, 1.66 - 0.09 * math.sin(math.pi * t / 16) - 0.012 * (t % 2)) for t in range(17)]
        F.sweep('wire', [(x, yy, zb + 0.04) for x, yy in pts], 0.0035, M.string, verts=4)
        for x, yy in pts[2:-1:2]:
            F.cyl(f'socket{x:.2f}', 0.008, 0.018, (x, yy - 0.012, zb + 0.045), M.string, verts=6)
            F.sphere(f'bulb{x:.2f}', 0.018, (x, yy - 0.036, zb + 0.048), M.bulb, scale=(1, 1, 1.3), segments=8, rings=5)
        # three balloons tied to the top rail
        for x, y, z, m in ((0.3, 1.92, -0.25, M.coral), (0.32, 2.04, -0.18, M.turq), (0.2, 2.0, -0.15, M.sunny)):
            _balloon(F, f'balloon{x}{y}', x, y, z, 0.1, m, (0.36, 1.715, zb + 0.03))
        # a gift on the floor in front: a box, a ribbon both ways, a bow of two loops and tails
        F.box('gift', 0.3, 0.2, 0.22, (0.15, 0.1, -0.05), M.sunny, bevel=0.004, bevel_segments=1)
        F.box('ribbon_a', 0.04, 0.204, 0.224, (0.15, 0.101, -0.05), M.magenta)
        F.box('ribbon_b', 0.304, 0.204, 0.04, (0.15, 0.101, -0.05), M.magenta)
        for s in (-1, 1):
            F.sweep(f'bow{s}', [(0.15, 0.2, -0.05), (0.15 + s * 0.04, 0.24, -0.06), (0.15 + s * 0.075, 0.225, -0.05),
                                (0.15 + s * 0.05, 0.205, -0.04), (0.15, 0.2, -0.05)], None, M.magenta,
                    profile=_sq(0.004, 0.035), up=(0, 0, 1), resolution=2)
            F.sweep(f'bow_tail{s}', [(0.15, 0.2, -0.05), (0.15 + s * 0.03, 0.202, -0.02), (0.15 + s * 0.05, 0.201, 0.0)], None,
                    M.magenta, profile=_sq(0.003, 0.03), up=(0, 1, 0))
    G.join(still, 'body', origin=(0, 0, 0))
    # A new snapshot in the bottom row, origin on its middle and built at full size, then shrunk to nothing:
    # `photo` grows it onto the board.
    px, py, pr = NEW_PHOTO
    with G.new_parts() as photo:
        _photo(F, 'new_photo', px, py, zb + 0.017, pr, M.magenta, M.pin_b)
    photo = G.hinged(G.join(photo, 'photo'), F.at(px, py, zb + 0.017))
    photo.scale[0] = photo.scale[2] = 0.001
    objs = G.settle(centre=False)
    G.grow(photo, 'photo', seconds=0.8, axis='X')
    G.grow(photo, 'photo', seconds=0.8, axis='Z')
    return objs


FAMILY = G.Family(
    'pack_outgoing', M,
    [G.Model('partyBar', partyBar, size=(1.84, 2.0, 0.875), key='model.outgoing.partyBar',
             centre=(0, 0.008), motions={'uses': {'mixDrinks': 'tap', 'soda': 'tap'}, 'hold': 2.0},
             paint=[('body', 'Body'), ('vinyl', 'Upholstery'), ('wood', 'Back bar')]),
     G.Model('conversationSectional', conversationSectional, size=(1.9, 1.42, 1.86),
             key='model.outgoing.conversationSectional', centre=(0, 0.0), motions={'uses': {'hangOut': 'sink_in'}},
             paint=[('frame', 'Body'), ('upholstery', 'Upholstery')]),
     G.Model('karaokeStage', karaokeStage, size=(1.9, 1.63, 0.894), key='model.outgoing.karaokeStage',
             # (centred on 0.007 before 2026-10-22: its screen's foot stood 1 cm into the wall)
             centre=(0, 0.012), motions={'uses': {'practise': 'mic_up', 'perform': 'mic_up'}},
             paint=[('body', 'Body'), ('speaker', 'Speakers')]),
     G.Model('videoCallNook', videoCallNook, size=(0.84, 1.283, 0.788), key='model.outgoing.videoCallNook',
             centre=(0, 0.077), motions={'uses': {'call': 'ring_light'}},
             paint=[('body', 'Body'), ('cushion', 'Upholstery'), ('wood', 'Wood')]),
     G.Model('danceFloor', danceFloor, size=(1.92, 2.33, 1.88), key='model.outgoing.danceFloor',
             centre=(0, 0.0), motions={'loops': {'dance': 'tiles'}},
             paint=[('base', 'Body'), ('speaker', 'Speakers')]),
     G.Model('photoWall', photoWall, size=(0.9, 2.16, 0.492), key='model.outgoing.photoWall',
             centre=(0, -0.184), motions={'uses': {'reminisce': 'photo'}},
             paint=[('wood', 'Body'), ('cork', 'Board')])],
    comment='Blender-built outgoing pack models (tools/art/blender/families/pack_outgoing.py). Listed in '
            'the base manifest\'s packs. The materials map onto the game\'s finishes so a model costs one '
            'draw call per finish group. Each moves while it is used: the bar\'s tap lever pulls, the '
            'sectional\'s cushions sink, the mics rise, the ring light swings up, the dance floor\'s tiles '
            'pulse and a new photo grows.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

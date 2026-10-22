# The example pack (P5): the chef's stove and the whirlpool tub of `content/packs/example.json`. Notes: pack_example.md.
#
# The pack people copy to make their own, so each model is built plainly, a helper per kind of part, from the
# toolkit's forms (gamekit/forms.py, kit.py): a frame-and-panel oven door (`G.kit.shaker`) with a bar pull
# (`G.kit.pull`), turned knobs and burners (`G.revolve`), a canopy hood lofted from its band up into the chimney
# (`G.loft` through `G.ring`s), the cast-iron grates' frames mitred (`G.kit.moulding`), a rail bent in one piece
# (`G.kit.tube` through `G.kit.fillet`ed corners), the tub's acrylic shell lofted in one piece, its cedar cladding
# extruded from a grooved outline (`G.extrude`), headrests and towels stuffed (`G.soft_box`).
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`). The stove's left oven door drops (`oven_open`) while a resident bakes, and a knob turns as a ring of
# gas flame grows on the front burner (`burner_on`) while they cook on the hob; the tub's water rises (`fill`) and
# bubbles well up through it while it lasts (`bubbles`, a loop).
#
# Run:  cargo art pack_example [--only chefStove] [--preview] [--lit]
import math
import os
import random
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gamekit as G  # noqa: E402
from plants import leaf  # noqa: E402  (a closed, bent leaf blade: the tub's plant)

M = G.Palette(
    body=('#C3C7CB', 'metal'),       # the stove's stainless steel; its main part
    dark=('#2F3236', 'satin'),       # the hob top, kick plate, control strip and the oven's enamel inside
    grate=('#1E1F22', 'satin'),      # cast-iron grates and burner heads
    cap=('#4A4D52', 'metal'),        # burner caps
    window=('#1D2228', 'gloss'),     # the oven windows (opaque, glossy black)
    chrome=('#D0D3D6', 'chrome'),    # handles, knobs' skirts, the utensil rail, the tub's jets
    knob=('#2B2C2E', 'metal'),       # black metal knobs
    tile=('#ECEAE5', 'gloss'),       # the splashback's metro tiles
    grout=('#B9B4AA', 'matte'),
    pot=('#B9BDC1', 'metal'),        # the stock pot
    copper=('#B87333', 'metal'),     # the hanging pan and ladle
    pan=('#2F3236', 'satin'),
    egg=('#E9C46A', 'satin'),
    crust=('#B8763C', 'satin'),      # the loaf baking in the oven
    flame=('#3E7CC2', 'gloss'),      # the gas flame (gloss, never emissive)
    shell=('#F4F4F2', 'gloss'),      # the tub's acrylic shell; its main part
    wood=('#A8754A', 'wood'),        # the tub's slatted cedar surround
    wood_dark=('#7A5234', 'wood'),   # the surround's posts and foot rail
    plinth=('#3A3D41', 'satin'),
    water=('#6FC3CF', 'gloss'),
    foam=('#F2FAFA', 'satin'),       # the bubbles
    pillow=('#E9E6DE', 'satin'),     # headrests
    towel=('#FFFFFF', 'satin'),
    towel2=('#9DC6CF', 'satin'),
    candle=('#F3EAD8', 'satin'),
    leaf=('#5E8F4E', 'satin'),
    terracotta=('#B5653F', 'satin'),
)

WALL = -0.43   # the wall's face in a 1-deep footprint, game depth


# --- the chef's stove (2x1): a six-burner range with a double oven, under a chimney hood -------------------
HOB = 0.92     # the hob's top
FRONT = 0.15   # the carcass's front face
BURNERS = [(x, z) for z in (-0.27, 0.0) for x in (-0.5, 0.0, 0.5)]
FLAME = (0.0, 0.0)   # the front middle burner, lit by `burner_on`
KNOBS = (-0.65, -0.45, 0.25, 0.45, 0.65)   # the still knobs' x; the one at -0.25 turns


def _burner(F, name, x, z):
    """A gas burner on the hob: a flared head with its flame ports, a black enamel cap on it."""
    G.revolve(name, [(0, 0), (0.064, 0), (0.064, 0.005), (0.052, 0.012), (0.047, 0.02), (0, 0.02)],
              at=F.at(x, HOB, z), material=M.grate, segments=14)
    G.revolve(f'{name}_cap', [(0, 0.019), (0.04, 0.019), (0.04, 0.024), (0.03, 0.031), (0, 0.033)],
              at=F.at(x, HOB, z), material=M.cap, segments=12)


def _grate(F, name, x):
    """A cast-iron grate over a pair of burners (the back and front one at `x`): a mitred frame standing on
    four feet, a bar across the middle and fingers reaching in towards each burner."""
    y, z0, z1, hw = HOB + 0.026, -0.4, 0.13, 0.24
    bar = [(0, 0), (0.016, 0), (0.016, 0.016), (0, 0.016)]                      # (inset, up): a square bar
    G.kit.moulding(F, name, [(x - hw, z0), (x + hw, z0), (x + hw, z1), (x - hw, z1)], bar, M.grate, at=y,
                   plane='top')
    prof = [(-0.007, -0.008), (0.007, -0.008), (0.007, 0.008), (-0.007, 0.008)]   # a square bar, as the frame's
    F.sweep(f'{name}_mid', [(x - hw + 0.008, y + 0.008, -0.135), (x + hw - 0.008, y + 0.008, -0.135)], None,
            M.grate, profile=prof, up=(0, 0, 1))
    for bz in (-0.27, 0.0):                                  # four fingers round each burner, dipping at their tips
        for dx, dz in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            reach = hw - 0.008 if dx else 0.12
            a = (x + dx * reach, y + 0.008, bz + dz * reach)
            b = (x + dx * 0.075, y + 0.006, bz + dz * 0.075)
            F.sweep(f'{name}_finger{bz}{dx}{dz}', [a, b], None, M.grate, profile=prof, up=(0, 0, 1), scale=[1.0, 0.8])
    for fx in (x - hw + 0.008, x + hw - 0.008):
        for fz in (z0 + 0.008, z1 - 0.008):
            F.box(f'{name}_foot{fx}{fz}', 0.014, 0.026, 0.014, (fx, HOB + 0.013, fz), M.grate)


def _knob(F, name, x, y):
    """A range knob on the control strip: a chrome skirt, a black grip with a domed face, a chrome pointer."""
    G.revolve(f'{name}_skirt', [(0, 0), (0.031, 0), (0.031, 0.003), (0.027, 0.008), (0, 0.008)],
              at=F.at(x, y, FRONT + 0.012), material=M.chrome, segments=12, rot=(90, 0, 0))
    G.revolve(name, [(0, 0.006), (0.022, 0.006), (0.024, 0.012), (0.024, 0.03), (0.02, 0.036), (0, 0.038)],
              at=F.at(x, y, FRONT + 0.012), material=M.knob, segments=12, rot=(90, 0, 0))
    F.box(f'{name}_mark', 0.005, 0.018, 0.004, (x, y + 0.01, FRONT + 0.05), M.chrome)


def _oven_door(F, name, x, w=0.76):
    """An oven door on its bottom hinge at the oven mouth: a steel frame round a black window (`G.kit.shaker`), a
    bar pull across its top on two stand-offs (`G.kit.pull`)."""
    G.kit.shaker(F, name, w, 0.54, (x, 0.41, FRONT + 0.0125), 0.025, M.body, rail=0.085, panel=M.window, recess=0.007)
    G.kit.pull(F, f'{name}_pull', x, 0.63, FRONT + 0.025, w - 0.16, M.chrome, r=0.012, out=0.045)


def _oven(F, x):
    """The left oven's inside, seen with its door down: dark enamel walls, two wire racks, a loaf on a tray."""
    w, h, d = 0.62, 0.42, 0.34
    yc, back = 0.42, FRONT - 0.34
    F.box('oven_back', w, h, 0.01, (x, yc, back), M.dark)
    for s in (-1, 1):
        F.box(f'oven_side{s}', 0.01, h, d, (x + s * w / 2, yc, back + d / 2), M.dark)
        F.box(f'oven_lid{s}', w, 0.01, d, (x, yc + s * h / 2, back + d / 2), M.dark)
    for k, y in enumerate((0.3, 0.46)):                      # wire racks: a bent rim and rods front to back
        G.kit.tube(F, f'rack{k}', G.kit.fillet([(x - 0.29, y, back + 0.02), (x - 0.29, y, FRONT - 0.03),
                                                (x + 0.29, y, FRONT - 0.03), (x + 0.29, y, back + 0.02)], 0.02),
                   0.004, M.chrome, verts=4, round=1)
        for i in range(7):
            rx = x - 0.24 + i * 0.08
            F.sweep(f'rack{k}_rod{i}', [(rx, y, back + 0.02), (rx, y, FRONT - 0.03)], 0.003, M.chrome, verts=4)
    F.box('tray', 0.4, 0.015, 0.26, (x, 0.315, -0.02), M.dark, bevel=0.004, bevel_segments=1)
    F.soft('loaf', 0.26, 0.09, 0.14, (x, 0.365, -0.02), M.crust, round=0.04, puff=(0, 0.015, 0), flat=2, round_segments=1)


def _tiles(F):
    """The splashback: metro tiles in running bond over a grout board."""
    F.box('grout', 1.6, 0.62, 0.01, (0, HOB + 0.07 + 0.31, WALL + 0.005), M.grout)
    tw, th = 0.15, 0.075
    for r in range(8):
        y = HOB + 0.07 + 0.004 + th / 2 + r * (th + 0.003)
        x = -0.8 - (tw + 0.003) / 2 * (r % 2)
        while x < 0.8:
            a, b = max(x, -0.797), min(x + tw, 0.797)
            if b - a > 0.02:
                F.box(f'tile{r}_{x:.3f}', b - a, th, 0.008, ((a + b) / 2, y, WALL + 0.014), M.tile)
            x += tw + 0.003


def _hood(F):
    """The hood in one piece, lofted through rounded rectangles: its band, the canopy rising to the chimney and the
    chimney's two telescoped sections up to the ceiling; under it a baffle filter and two lamps."""
    bx, by, _ = F.at(0, 0, 0)

    def ring(w, d, zc, y, sq=12):
        cx, cy, _ = F.at(0, 0, zc)
        return G.ring(w / 2, d / 2, y, 24, square=sq, at=(cx, cy))
    G.loft('hood', [ring(1.18, 0.48, WALL + 0.24, 1.556), ring(1.2, 0.5, WALL + 0.25, 1.564),
                    ring(1.2, 0.5, WALL + 0.25, 1.68), ring(1.16, 0.48, WALL + 0.24, 1.7),
                    ring(0.38, 0.29, WALL + 0.145, 2.04), ring(0.36, 0.28, WALL + 0.14, 2.06),
                    ring(0.36, 0.28, WALL + 0.14, 2.26), ring(0.37, 0.285, WALL + 0.1425, 2.26),
                    ring(0.37, 0.285, WALL + 0.1425, 2.55)], material=M.body, smooth=False)
    F.box('hood_filter', 1.12, 0.01, 0.44, (0, 1.552, WALL + 0.24), M.dark)
    for i in range(9):                                      # the baffles' folds
        F.box(f'baffle{i}', 0.008, 0.008, 0.42, (-0.48 + i * 0.12, 1.547, WALL + 0.24), M.chrome)
    for x in (-0.38, 0.38):
        F.cyl(f'hood_lamp{x}', 0.03, 0.006, (x, 1.548, -0.05), M.egg, verts=12)


def _rail(F):
    """The utensil rail, bent in one piece out from the wall and back, with a ladle, a slotted spatula, a whisk and a
    copper pan on S-hooks."""
    z, y = WALL + 0.06, 1.36
    G.kit.tube(F, 'rail', G.kit.fillet([(-0.62, y, WALL + 0.002), (-0.62, y, z), (0.62, y, z), (0.62, y, WALL + 0.002)],
                                       0.03), 0.008, M.chrome, verts=8, round=1)
    for x in (-0.62, 0.62):
        F.cyl(f'rail_rose{x}', 0.018, 0.008, (x, y, WALL + 0.004), M.chrome, rot=(90, 0, 0), verts=12)
    for x in (-0.42, -0.28, -0.14, 0.32):                   # S-hooks: over the rail, down and up into a hook
        F.sweep(f'hook{x}', [(x, y + 0.004, z - 0.012), (x, y + 0.016, z - 0.004), (x, y + 0.014, z + 0.01),
                             (x, y - 0.01, z + 0.012), (x, y - 0.04, z + 0.004), (x, y - 0.05, z - 0.006),
                             (x, y - 0.042, z - 0.014)], 0.0025, M.chrome, verts=4)
    hz = z - 0.004
    # the ladle: a handle with a hooked end and a turned bowl
    F.sweep('ladle_handle', [(-0.42, y - 0.03, hz + 0.004), (-0.42, y - 0.06, hz + 0.004), (-0.42, 1.12, hz + 0.008),
                             (-0.42, 1.08, hz + 0.022)], None, M.copper, profile=G.rounded_rect(0.014, 0.004, 0.002, corner=1),
            up=(0, 1, 0), resolution=2)
    G.revolve('ladle_bowl', [(0, 0), (0.035, 0.008), (0.045, 0.03), (0.046, 0.04), (0.04, 0.04), (0.03, 0.016), (0, 0.008)],
              at=F.at(-0.42, 1.04, hz + 0.065), material=M.copper, segments=14, rot=(-80, 0, 0))
    # the spatula: a handle and a blade with three slots
    F.sweep('spatula_handle', [(-0.28, y - 0.035, hz), (-0.28, 1.16, hz + 0.004)], None, M.body,
            profile=G.rounded_rect(0.016, 0.008, 0.003, corner=1), up=(0, 1, 0))
    F.extrude('spatula_blade', G.rounded_rect(0.07, 0.1, 0.012, corner=2), 0.003, (-0.28, 1.105, hz + 0.004), M.body,
              holes=[[(u + dx, v - 0.01) for u, v in G.rounded_rect(0.008, 0.055, 0.004, corner=1)]
                     for dx in (-0.018, 0.0, 0.018)])
    # the whisk: a handle and four wire loops, each in its own plane round the handle's axis
    F.sweep('whisk_handle', [(-0.14, y - 0.035, hz), (-0.14, 1.23, hz)], 0.009, M.body, verts=10)
    for k in range(4):
        a = math.radians(45 * k)
        pts = []
        for i in range(12):
            s = math.tau * i / 12
            u = 0.03 * math.sin(s) * (0.35 + 0.65 * (1 - math.cos(s)) / 2)
            pts.append((-0.14 + u * math.cos(a), 1.23 - 0.13 * (1 - math.cos(s)) / 2, hz + u * math.sin(a)))
        F.sweep(f'whisk_wire{k}', pts, 0.0018, M.body, verts=4, closed=True)
    # the copper pan hanging by its handle, its bottom to the room
    G.revolve('pan_hang', [(0, 0), (0.1, 0), (0.112, 0.004), (0.118, 0.04), (0.112, 0.04), (0.104, 0.008), (0, 0.006)],
              at=F.at(0.32, 1.13, WALL + 0.046), material=M.copper, segments=20, rot=(-90, 0, 0))
    F.sweep('pan_hang_handle', [(0.32, 1.24, WALL + 0.035), (0.32, y - 0.035, WALL + 0.05)], None, M.copper,
            profile=G.rounded_rect(0.02, 0.008, 0.003, corner=1), up=(0, 1, 0), scale=[1.0, 0.8])


def _cookware(F):
    """On the hob: a stock pot with its lid and loop handles, a frying pan with an omelette; salt and pepper mills."""
    y = HOB + 0.043                                         # the grates' top
    G.revolve('stock_pot', [(0, 0), (0.12, 0), (0.128, 0.008), (0.13, 0.19), (0.134, 0.196), (0.13, 0.2), (0.122, 0.2),
                            (0.122, 0.012), (0, 0.012)], at=F.at(-0.5, y, 0.0), material=M.pot, segments=22)
    G.revolve('pot_lid', [(0, 0), (0.132, 0), (0.13, 0.006), (0.1, 0.018), (0.04, 0.026), (0, 0.027)],
              at=F.at(-0.5, y + 0.2, 0.0), material=M.pot, segments=22)
    G.revolve('pot_lid_knob', [(0, 0), (0.012, 0), (0.009, 0.012), (0.018, 0.02), (0.016, 0.03), (0, 0.032)],
              at=F.at(-0.5, y + 0.226, 0.0), material=M.knob, segments=10)
    for s in (-1, 1):
        x = -0.5 + s * 0.128
        F.sweep(f'pot_handle{s}', [(x, y + 0.17, -0.035), (x + s * 0.03, y + 0.172, -0.03), (x + s * 0.036, y + 0.175, 0.0),
                                   (x + s * 0.03, y + 0.172, 0.03), (x, y + 0.17, 0.035)], 0.006, M.pot, verts=6,
                resolution=2)
    G.revolve('frying_pan', [(0, 0), (0.13, 0), (0.15, 0.04), (0.142, 0.04), (0.124, 0.008), (0, 0.008)],
              at=F.at(0.5, y, -0.27), material=M.pan, segments=22)
    F.sweep('pan_handle', [(0.6, y + 0.032, -0.18), (0.68, y + 0.05, -0.11), (0.76, y + 0.06, -0.04)], None, M.pan,
            profile=G.rounded_rect(0.022, 0.012, 0.005, corner=1), up=(0, 1, 0), scale=[0.9, 1.0, 1.15])
    F.soft('omelette', 0.17, 0.016, 0.15, (0.5, y + 0.016, -0.27), M.egg, round=0.008, puff=0.006, flat=2,
           round_segments=1)
    for x, z, m in ((0.72, -0.38, M.tile), (0.64, -0.39, M.knob)):   # mills: a waisted body, a knob on top
        G.revolve(f'mill{x}', [(0, 0), (0.024, 0), (0.026, 0.02), (0.018, 0.06), (0.024, 0.1), (0.022, 0.115),
                               (0.008, 0.125), (0.006, 0.135), (0, 0.137)], at=F.at(x, HOB, z), material=m, segments=10)


def chefStove():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('kick', 1.54, 0.08, 0.52, (0, 0.04, -0.15), M.dark)
        carcass = F.box('carcass', 1.6, 0.82, FRONT - WALL, (0, 0.49, (FRONT + WALL) / 2), M.body, bevel=0.01)
        # the left oven's mouth is cut into the carcass, so its door opens onto an inside
        G.cut(carcass, F.box('oven_cut', 0.62, 0.42, 0.36, (-0.4, 0.42, FRONT - 0.17), M.body))
        _oven(F, -0.4)
        F.box('hob', 1.62, 0.03, FRONT - WALL + 0.02, (0, HOB - 0.015, (FRONT + WALL) / 2 + 0.01), M.dark, bevel=0.006)
        F.box('upstand', 1.62, 0.07, 0.04, (0, HOB + 0.035, WALL + 0.02), M.body, bevel=0.006)
        F.box('strip', 1.56, 0.1, 0.01, (0, 0.8, FRONT + 0.005), M.dark, bevel=0.003, bevel_segments=1)
        for x in KNOBS:
            _knob(F, f'knob{x}', x, 0.8)
        F.box('logo', 0.1, 0.02, 0.006, (-0.06, 0.8, FRONT + 0.013), M.chrome, bevel=0.002, bevel_segments=1)
        F.box('lamp', 0.03, 0.03, 0.006, (0.08, 0.8, FRONT + 0.013), M.egg)
        F.box('oven_split', 0.02, 0.56, 0.02, (0, 0.41, FRONT + 0.005), M.dark)
        _oven_door(F, 'right_door', 0.4)                     # the right oven, still
        for x, z in BURNERS:
            _burner(F, f'burner{x}{z}', x, z)
        for x in (-0.5, 0.0, 0.5):
            _grate(F, f'grate{x}', x)
        _tiles(F)
        _rail(F)
        _hood(F)
        _cookware(F)
    G.join(still, 'body', origin=(0, 0, 0))
    # The left oven door, on its bottom hinge: `oven_open` drops it 85 degrees to the front.
    with G.new_parts() as door:
        _oven_door(F, 'oven_door', -0.4)
    door = G.hinged(G.join(door, 'oven_door'), F.at(-0.4, 0.14, FRONT + 0.0125))
    # The front middle burner's knob, turned by `burner_on` as its flame grows.
    with G.new_parts() as k:
        _knob(F, 'burner_knob', -0.25, 0.8)
    knob = G.hinged(G.join(k, 'burner_knob'), F.at(-0.25, 0.8, FRONT + 0.03))
    # The ring of gas flame round the front middle burner's cap, built full size and scaled to nothing.
    fx, fz = FLAME
    with G.new_parts() as fl:
        G.torus('flame_ring', 0.046, 0.008, at=F.at(fx, HOB + 0.026, fz), material=M.flame, major_segments=18,
                minor_segments=5)
        for k in range(12):
            a = 30 * k
            F.cone(f'flame_tip{k}', 0.01, 0.0, 0.035, (fx + 0.05 * G.cosd(a), HOB + 0.043, fz + 0.05 * G.sind(a)), M.flame,
                   verts=6)
    flame = G.join(fl, 'flame', origin=F.at(fx, HOB + 0.026, fz))
    flame.scale = (0.001, 0.001, 0.001)
    objs = G.settle(centre=False)
    G.swing(door, 'oven_open', 85, seconds=1.2, axis='X')
    G.swing(knob, 'burner_on', 90, seconds=0.5, axis='Y')
    for axis in 'XYZ':
        G.grow(flame, 'burner_on', seconds=0.5, axis=axis)
    return objs


# --- the whirlpool tub (2x2): an acrylic tub in a slatted cedar surround, two bathers on its bench ---------
RIM = 0.72      # the rim's top
FLOOR = 0.2     # the footwell's floor top
WATER = 0.65    # the water's top when full
IN = 0.7        # the basin's half width inside
OUT = 0.9       # the cladding's face


def _shell(F):
    """The acrylic shell in one piece (`G.loft` through rounded squares, as the kitchen sinks' bowls): under the
    rim's edge, over its broad flat top, rolling over into the basin and down its walls to the footwell's floor."""
    def ring(r, y, sq):
        return G.ring(r, r, y, 32, square=sq)
    G.loft('shell', [ring(0.9, RIM - 0.075, 12), ring(0.93, RIM - 0.06, 12), ring(0.93, RIM - 0.02, 12),
                     ring(0.918, RIM - 0.002, 12), ring(0.9, RIM, 12), ring(IN + 0.03, RIM, 7), ring(IN + 0.004, RIM - 0.01, 7),
                     ring(IN, RIM - 0.04, 7), ring(IN - 0.004, FLOOR + 0.06, 7), ring(IN - 0.03, FLOOR + 0.006, 7),
                     ring(IN - 0.08, FLOOR, 7), ring(0.0, FLOOR, 7)], material=M.shell)
    # the moulded bench along the back and its two back rests, leaning up the back wall
    F.soft('bench', 2 * IN - 0.06, 0.26, 0.74, (0, FLOOR + 0.12, -IN + 0.37), M.shell, round=0.04)
    for x in (-0.5, 0.5):
        F.soft(f'backrest{x}', 0.5, 0.3, 0.07, (x, 0.58, -IN + 0.045), M.shell, rot=(12, 0, 0), round=0.03, puff=0.012,
               flat=2, round_segments=1)


def _jet(F, name, x, y, z, rot):
    """A jet: a chrome ring round a dark nozzle, set in the shell."""
    F.cyl(name, 0.024, 0.01, (x, y, z), M.chrome, rot=rot, verts=12, bevel=0.003, bevel_segments=1)
    F.cyl(f'{name}_nozzle', 0.011, 0.012, (x, y, z), M.plinth, rot=rot, verts=8)


def _cladding(F):
    """The cedar surround: on each side tongue-and-groove boards, one slab extruded upright from an outline grooved
    between the boards (`G.extrude`); posts at the corners, a foot rail round the bottom, a plinth under it."""
    n, span, t = 11, 1.74, 0.02
    pts = [(-span / 2, OUT - t)]
    for k in range(n):                                      # along the face: each board, a V-groove after it
        x0 = -span / 2 + k * span / n
        pts += [(x0 + 0.006, OUT), (x0 + span / n - 0.006, OUT), (x0 + span / n, OUT - 0.01)]
    pts[-1] = (span / 2, OUT - t)
    face = pts
    for i, rot in enumerate((0, 90, 180, 270)):
        c, s = math.cos(math.radians(rot)), math.sin(math.radians(rot))
        F.extrude(f'cladding{i}', [(x * c - z * s, x * s + z * c) for x, z in reversed(face)], 0.6, (0, 0.35, 0), M.wood,
                  plane='top')
    for sx in (-1, 1):
        for sz in (-1, 1):
            F.box(f'post{sx}{sz}', 0.08, 0.6, 0.08, (sx * (OUT - 0.03), 0.35, sz * (OUT - 0.03)), M.wood_dark, bevel=0.006,
                  bevel_segments=1)
    G.kit.moulding(F, 'foot_rail', [(-0.92, -0.92), (0.92, -0.92), (0.92, 0.92), (-0.92, 0.92)],
                   [(0, 0), (0.03, 0), (0.03, 0.04), (0.012, 0.055), (0, 0.055)], M.wood_dark, at=0.05, plane='top')
    F.box('plinth', 1.78, 0.05, 1.78, (0, 0.025, 0), M.plinth)


def _tub_things(F):
    """Headrests on the back rim, the control panel, towels, candles and a plant on the rim's corners."""
    for x in (-0.5, 0.5):
        F.soft(f'headrest{x}', 0.36, 0.08, 0.17, (x, RIM + 0.035, -0.79), M.pillow, rot=(-14, 0, 0), round=0.035,
               puff=0.012, flat=2)
    F.box('panel', 0.12, 0.012, 0.16, (0.815, RIM + 0.004, -0.25), M.plinth, bevel=0.004, bevel_segments=1)
    for k in range(3):
        F.cyl(f'button{k}', 0.013, 0.008, (0.815, RIM + 0.012, -0.3 + 0.05 * k), M.chrome, verts=10)
    F.soft('towel_a', 0.4, 0.07, 0.19, (-0.6, RIM + 0.035, 0.815), M.towel, round=0.03, puff=0.008, flat=2)
    F.soft('towel_b', 0.38, 0.06, 0.18, (-0.6, RIM + 0.1, 0.815), M.towel2, round=0.026, puff=0.008, flat=2)
    for x, z, h in ((0.78, 0.8, 0.12), (0.64, 0.84, 0.08), (0.8, 0.66, 0.06)):   # pillar candles, their tops dished
        G.revolve(f'candle{x}', [(0, 0), (0.035, 0), (0.035, h), (0.03, h - 0.004), (0, h - 0.008)],
                  at=F.at(x, RIM, z), material=M.candle, segments=14)
        F.cyl(f'wick{x}', 0.002, 0.014, (x, RIM + h, z), M.plinth, verts=4)
    px, pz = -0.81, -0.81                                    # a little potted plant: leaves arching from a clay pot
    G.revolve('plant_pot', [(0, 0), (0.055, 0), (0.07, 0.1), (0.078, 0.104), (0.078, 0.115), (0.066, 0.115),
                            (0.062, 0.1), (0, 0.1)], at=F.at(px, RIM, pz), material=M.terracotta, segments=14)
    base = F.at(px, RIM + 0.1, pz)
    for k in range(7):
        leaf(f'leaf{k}', base, k * 360 / 7 + 10, 55 + 12 * (k % 2), 0.15, 0.05, M.leaf, shape='lance', droop=0.3,
             rows=4, ring=4)


def whirlpoolTub():
    F = G.GameFrame()
    with G.new_parts() as still:
        _shell(F)
        for x in (-0.5, 0.5):                                # jets: in the back rests, the walls and the footwell
            for dx in (-0.12, 0.12):
                _jet(F, f'jet_back{x}{dx}', x + dx, 0.56, -IN + 0.082, (78, 0, 0))
            _jet(F, f'jet_front{x}', x, 0.34, IN - 0.006, (90, 0, 0))
        for s in (-1, 1):
            for z in (-0.35, 0.25):
                _jet(F, f'jet_side{s}{z}', s * (IN - 0.006), 0.52, z, (0, 90, 0))
        _cladding(F)
        _tub_things(F)
    G.join(still, 'body', origin=(0, 0, 0))
    # The water: the basin's inside up to WATER, its origin just under the footwell's floor; scaled flat
    # there at rest, `fill` raises it while residents soak.
    with G.new_parts() as water:
        G.loft('water', [G.ring(IN - 0.004, IN - 0.004, FLOOR - 0.01, 32, square=7),
                         G.ring(IN - 0.004, IN - 0.004, WATER, 32, square=7)], material=M.water)
    water = G.join(water, 'water', origin=F.at(0, FLOOR - 0.01, 0))
    water.scale[2] = 0.001
    # Bubbles welling up at the surface over the jets: domes just above and below it, joined with their
    # origin under the floor and scaled flat there; `bubbles` swells them up through the water and back
    # down, over and over while the soak lasts. Its rise takes as long as `fill`, from the same origin, so on
    # the first round the bubbles ride up on the rising water.
    with G.new_parts() as foam:
        rnd = random.Random(4)
        for cx, cz, n in ((-0.5, 0.3, 7), (0.5, 0.3, 7), (-0.5, -0.1, 5), (0.5, -0.1, 5), (0.0, 0.5, 5), (-0.62, 0.55, 3),
                          (0.62, 0.55, 3)):
            for i in range(n):
                r = rnd.uniform(0.018, 0.042)
                F.sphere(f'bubble{cx}{cz}{i}', r, (cx + rnd.uniform(-0.12, 0.12), WATER + rnd.uniform(-0.012, 0.004),
                                                   cz + rnd.uniform(-0.1, 0.1)), M.foam, scale=(1, 1, 0.7), segments=7,
                         rings=4)
    foam = G.join(foam, 'bubbles', origin=F.at(0, FLOOR - 0.01, 0))
    foam.scale[2] = 0.001
    objs = G.settle(centre=False)
    G.grow(water, 'fill', seconds=0.8)
    G.squash(foam, 'bubbles', (1, 1, 1000), seconds=1.6, back=True)
    return objs


FAMILY = G.Family(
    'pack_example', M,
    [G.Model('chefStove', chefStove, size=(1.62, 2.55, 0.662), key='model.chefStove', centre=(0, -0.099),
             # a meal on the hob lights the burner; a bake (practise, or any `bake` step) drops the oven door
             # for 3 s; by the cooking step's tag too (`stir`, `bake`), for a meal started at the fridge
             motions={'uses': {'cook': 'burner_on', 'stir': 'burner_on',
                               'practise': {'clip': 'oven_open', 'hold': 3.0}, 'bake': {'clip': 'oven_open', 'hold': 3.0}}},
             paint=[('body', 'Body'), ('chrome', 'Handles & knobs'), ('tile', 'Tiles')]),
     G.Model('whirlpoolTub', whirlpoolTub, size=(1.86, 0.929, 1.86), key='model.whirlpoolTub',
             motions={'uses': {'soak': 'fill'}, 'loops': {'soak': 'bubbles'}},
             paint=[('shell', 'Tub'), ('wood', 'Surround'), ('pillow', 'Headrests')])],
    comment='Blender-built example pack models (tools/art/blender/families/pack_example.py). Listed in '
            'the base manifest\'s packs. The materials map onto the game\'s finishes so a model costs one '
            'draw call per finish group. The stove\'s oven door drops and its front burner lights; the tub '
            'fills and bubbles while residents soak.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

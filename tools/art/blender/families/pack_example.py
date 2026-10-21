# The example pack (P5): the chef's stove and the whirlpool tub of `content/packs/example.json`. Notes: pack_example.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`). The stove's left oven door drops (`oven_open`) while a resident bakes, and a knob turns as a ring of
# gas flame grows on the front burner (`burner_on`) while they cook on the hob; the tub's water rises (`fill`) and bubbles well up through it while it lasts
# (`bubbles`, a loop).
#
# Run:  cargo art pack_example [--only chefStove] [--preview]
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import bpy  # noqa: E402
import gamekit as G  # noqa: E402
from gamekit.shapes import _recalc_normals  # noqa: E402

M = G.Palette(
    body=('#C3C7CB', 'metal'),       # the stove's stainless steel; its main part
    dark=('#2F3236', 'satin'),       # the hob top, kick plate and control strip
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
    flame=('#3E7CC2', 'gloss'),      # the gas flame (gloss, never emissive)
    shell=('#F4F4F2', 'gloss'),      # the tub's acrylic shell; its main part
    wood=('#A8754A', 'wood'),        # the tub's slatted cedar surround
    wood_dark=('#7A5234', 'wood'),   # the surround's backing behind the slats
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
_n = [0]


def _name(stem):
    _n[0] += 1
    return f'{stem}{_n[0]}'


def _frustum(F, name, bottom, top, material):
    """A closed four-sided frustum in game axes: `bottom` and `top` are (width, depth, height, depth centre)."""
    verts = []
    for w, d, y, zc in (bottom, top):
        for sx, sz in ((-1, -1), (1, -1), (1, 1), (-1, 1)):
            verts.append(F.at(sx * w / 2, y, zc + sz * d / 2))
    faces = [(0, 1, 2, 3), (4, 7, 6, 5)] + [(k, (k + 1) % 4, (k + 1) % 4 + 4, k + 4) for k in range(4)]
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    _recalc_normals(obj)
    return obj


# --- the chef's stove (2x1): a six-burner range with a double oven, under a chimney hood -------------------
HOB = 0.92     # the hob's top
BURNERS = [(x, z) for z in (-0.27, 0.0) for x in (-0.5, 0.0, 0.5)]
FLAME = (0.0, 0.0)   # the front middle burner, lit by `burner_on`


def _burner(F, x, z):
    F.cyl(_name('burner'), 0.06, 0.02, (x, HOB + 0.01, z), M.grate, verts=16)
    F.cyl(_name('burner_cap'), 0.038, 0.012, (x, HOB + 0.026, z), M.cap, verts=14)


def _grate(F, x0, x1, z0, z1):
    """A cast-iron grate over two burners: a frame, a cross bar and fingers towards each burner."""
    y = HOB + 0.035
    w, d = x1 - x0, z1 - z0
    xc, zc = (x0 + x1) / 2, (z0 + z1) / 2
    for z in (z0, z1):
        F.box(_name('grate_bar'), w, 0.016, 0.016, (xc, y, z), M.grate)
    for x in (x0, x1):
        F.box(_name('grate_bar'), 0.016, 0.016, d, (x, y, zc), M.grate)
    F.box(_name('grate_mid'), w, 0.016, 0.016, (xc, y, zc), M.grate)
    for s in (-1, 1):                                    # fingers in from the sides towards each burner
        for z in (z0 + d / 4, z1 - d / 4):
            F.box(_name('grate_finger'), 0.12, 0.016, 0.016, (xc + s * (w / 2 - 0.06), y, z), M.grate)
    # feet of the grate on the hob
    for x in (x0, x1):
        for z in (z0, z1):
            F.box(_name('grate_foot'), 0.02, 0.02, 0.02, (x, HOB + 0.012, z), M.grate)


def _knob(F, x, y, z):
    F.cyl(_name('knob_skirt'), 0.03, 0.008, (x, y, z), M.chrome, rot=(90, 0, 0), verts=16)
    F.cyl(_name('knob'), 0.024, 0.03, (x, y, z + 0.018), M.knob, rot=(90, 0, 0), verts=12)
    F.box(_name('knob_mark'), 0.006, 0.02, 0.006, (x, y + 0.012, z + 0.034), M.chrome)


def _oven_door(F, x, w, front):
    """An oven door on its bottom hinge at the oven mouth: a steel front, a window and a bar handle."""
    F.box(_name('door'), w, 0.54, 0.025, (x, 0.41, front + 0.0125), M.body, bevel=0.008)
    F.box(_name('door_window'), w - 0.18, 0.26, 0.008, (x, 0.43, front + 0.028), M.window, bevel=0.006)
    G.rod(_name('door_handle'), F.at(x - w / 2 + 0.07, 0.63, front + 0.07), F.at(x + w / 2 - 0.07, 0.63, front + 0.07),
          0.012, material=M.chrome, verts=12)
    for s in (-1, 1):
        G.rod(_name('door_post'), F.at(x + s * (w / 2 - 0.08), 0.63, front + 0.025),
              F.at(x + s * (w / 2 - 0.08), 0.63, front + 0.07), 0.008, material=M.chrome, verts=8)


def chefStove():
    F = G.GameFrame()
    front = 0.15     # the carcass's front face
    with G.new_parts() as still:
        F.box('kick', 1.54, 0.08, 0.52, (0, 0.04, -0.15), M.dark)
        F.box('carcass', 1.6, 0.82, front - WALL, (0, 0.49, (front + WALL) / 2), M.body, bevel=0.01)
        F.box('hob', 1.62, 0.03, front - WALL + 0.02, (0, HOB - 0.015, (front + WALL) / 2 + 0.01), M.dark, bevel=0.006)
        F.box('upstand', 1.62, 0.07, 0.04, (0, HOB + 0.035, WALL + 0.02), M.body, bevel=0.006)
        F.box('strip', 1.56, 0.1, 0.01, (0, 0.8, front + 0.005), M.dark)
        for x in (-0.65, -0.45, 0.25, 0.45, 0.65):
            _knob(F, x, 0.8, front + 0.012)
        F.box('logo', 0.1, 0.02, 0.006, (-0.06, 0.8, front + 0.013), M.chrome)
        F.box('lamp', 0.03, 0.03, 0.006, (0.08, 0.8, front + 0.013), M.egg)
        F.box('oven_split', 0.02, 0.56, 0.02, (0, 0.41, front + 0.005), M.dark)
        _oven_door(F, 0.4, 0.76, front)                                      # the right oven, still
        F.box('oven_mouth', 0.66, 0.44, 0.004, (-0.4, 0.42, front + 0.002), M.window)   # behind the left door
        for y in (0.3, 0.46):
            F.box(_name('oven_rack'), 0.6, 0.008, 0.006, (-0.4, y, front + 0.005), M.chrome)
        for x, z in BURNERS:
            _burner(F, x, z)
        for x in (-0.5, 0.0, 0.5):
            _grate(F, x - 0.24, x + 0.24, -0.4, 0.13)
        # the splashback, metro tiles in running bond, and the hood over it on its chimney
        F.box('grout', 1.6, 0.62, 0.01, (0, HOB + 0.07 + 0.31, WALL + 0.005), M.grout)
        tw, th = 0.15, 0.075
        for r in range(8):
            y = HOB + 0.07 + 0.004 + th / 2 + r * (th + 0.003)
            off = (tw + 0.003) / 2 * (r % 2)
            x = -0.8 - off
            while x < 0.8:
                a, b = max(x, -0.797), min(x + tw, 0.797)
                if b - a > 0.02:
                    F.box(_name('tile'), b - a, th, 0.008, ((a + b) / 2, y, WALL + 0.014), M.tile)
                x += tw + 0.003
        G.rod('rail', F.at(-0.6, 1.36, WALL + 0.06), F.at(0.6, 1.36, WALL + 0.06), 0.008, material=M.chrome, verts=10)
        for x in (-0.6, 0.6):
            F.cyl(_name('rail_post'), 0.008, 0.05, (x, 1.36, WALL + 0.035), M.chrome, rot=(90, 0, 0), verts=8)
        # on the rail: a ladle, a spatula, a whisk and a copper pan
        for x, m in ((-0.42, M.copper), (-0.28, M.body), (-0.14, M.body)):
            G.torus(_name('hook'), 0.012, 0.003, at=F.at(x, 1.36, WALL + 0.06), material=M.chrome, rot=(0, 90, 0),
                    major_segments=10, minor_segments=4)
        F.cyl('ladle_handle', 0.006, 0.26, (-0.42, 1.22, WALL + 0.065), M.copper, verts=8)
        G.revolve('ladle_bowl', [(0, 0), (0.035, 0.012), (0.045, 0.04), (0.04, 0.04), (0.03, 0.016), (0, 0.006)],
                  at=F.at(-0.42, 1.05, WALL + 0.085), material=M.copper, segments=14, rot=(-80, 0, 0))
        F.cyl('spatula_handle', 0.006, 0.2, (-0.28, 1.25, WALL + 0.065), M.body, verts=8)
        F.box('spatula_blade', 0.07, 0.09, 0.006, (-0.28, 1.105, WALL + 0.065), M.body, bevel=0.002)
        F.cyl('whisk_handle', 0.009, 0.12, (-0.14, 1.29, WALL + 0.065), M.body, verts=8)
        for k in range(4):
            G.torus(_name('whisk_wire'), 0.032, 0.0025, at=F.at(-0.14, 1.17, WALL + 0.065), material=M.body,
                    rot=(0, 90, 45 * k), major_segments=12, minor_segments=4)
        F.cyl('pan_hang', 0.11, 0.012, (0.32, 1.17, WALL + 0.03), M.copper, rot=(90, 0, 0), verts=22)
        F.box('pan_hang_handle', 0.02, 0.17, 0.01, (0.32, 1.33, WALL + 0.04), M.copper)
        # the hood: a steel canopy band, a sloped top and the chimney to the ceiling
        F.box('hood_band', 1.2, 0.12, 0.5, (0, 1.62, WALL + 0.25), M.body, bevel=0.008)
        F.box('hood_filter', 1.12, 0.01, 0.44, (0, 1.556, WALL + 0.24), M.dark)
        for x in (-0.35, 0.35):
            F.cyl(_name('hood_lamp'), 0.03, 0.006, (x, 1.553, -0.05), M.egg, verts=12)
        _frustum(F, 'hood_top', (1.2, 0.5, 1.68, WALL + 0.25), (0.36, 0.28, 2.05, WALL + 0.14), M.body)
        F.box('chimney', 0.36, 0.5, 0.28, (0, 2.3, WALL + 0.14), M.body)
        # on the hob: a stock pot (left front) and a frying pan with an omelette (right back)
        G.revolve('stock_pot', [(0, 0), (0.12, 0), (0.13, 0.01), (0.13, 0.2), (0.122, 0.2), (0.122, 0.012), (0, 0.012)],
                  at=F.at(-0.5, HOB + 0.043, 0.0), material=M.pot, segments=22)
        F.cyl('pot_lid', 0.128, 0.012, (-0.5, HOB + 0.25, 0.0), M.pot, verts=22)
        F.sphere('pot_lid_knob', 0.016, (-0.5, HOB + 0.265, 0.0), M.knob, segments=10, rings=6)
        for s in (-1, 1):
            G.torus(_name('pot_handle'), 0.028, 0.006, at=F.at(-0.5 + s * 0.135, HOB + 0.21, 0.0), material=M.pot,
                    rot=(90, 0, 90), major_segments=10, minor_segments=4)
        G.revolve('frying_pan', [(0, 0), (0.13, 0), (0.15, 0.04), (0.142, 0.04), (0.124, 0.008), (0, 0.008)],
                  at=F.at(0.5, HOB + 0.043, -0.27), material=M.pan, segments=22)
        G.rod('pan_handle', F.at(0.6, HOB + 0.075, -0.18), F.at(0.75, HOB + 0.095, -0.05), 0.012, material=M.pan, verts=8)
        F.sphere('omelette', 0.1, (0.5, HOB + 0.053, -0.27), M.egg, scale=(1, 1, 0.12), segments=16, rings=6)
        F.box('salt', 0.05, 0.08, 0.05, (0.72, HOB + 0.04, -0.38), M.tile, bevel=0.01)
        F.cyl('pepper', 0.022, 0.13, (0.64, HOB + 0.065, -0.39), M.knob, verts=10)
    G.join(still, 'body', origin=(0, 0, 0))
    # The left oven door, on its bottom hinge: `oven_open` drops it 85 degrees to the front.
    with G.new_parts() as door:
        _oven_door(F, -0.4, 0.76, front)
    door = G.hinged(G.join(door, 'oven_door'), F.at(-0.4, 0.14, front + 0.0125))
    # The front middle burner's knob, turned by `burner_on` as its flame grows.
    with G.new_parts() as k:
        _knob(F, -0.25, 0.8, front + 0.012)
    knob = G.hinged(G.join(k, 'burner_knob'), F.at(-0.25, 0.8, front + 0.03))
    # The ring of gas flame around the front middle burner's cap, built full size and scaled to nothing.
    fx, fz = FLAME
    with G.new_parts() as fl:
        G.torus('flame_ring', 0.046, 0.009, at=F.at(fx, HOB + 0.028, fz), material=M.flame, major_segments=18,
                minor_segments=6)
        for k in range(12):
            a = 30 * k
            F.cone(_name('flame_tip'), 0.01, 0.0, 0.035, (fx + 0.05 * G.cosd(a), HOB + 0.045, fz + 0.05 * G.sind(a)),
                   M.flame, verts=6)
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


def whirlpoolTub():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('plinth', 1.78, 0.05, 1.78, (0, 0.025, 0), M.plinth)
        # the slatted surround: a dark backing on each side with slats over it, posts at the corners
        out = 0.9
        for s in (-1, 1):
            F.box(_name('backing'), 1.76, 0.6, 0.02, (0, 0.33, s * (out - 0.02)), M.wood_dark)
            F.box(_name('backing'), 0.02, 0.6, 1.76, (s * (out - 0.02), 0.33, 0), M.wood_dark)
            n = 11
            for k in range(n):
                c = -0.8 + (k + 0.5) * 1.6 / n
                F.box(_name('slat'), 1.6 / n - 0.012, 0.56, 0.018, (c, 0.33, s * (out - 0.004)), M.wood)
                F.box(_name('slat'), 0.018, 0.56, 1.6 / n - 0.012, (s * (out - 0.004), 0.33, c), M.wood)
            for t in (-1, 1):
                F.box(_name('post'), 0.08, 0.62, 0.08, (s * (out - 0.03), 0.34, t * (out - 0.03)), M.wood, bevel=0.006)
        for s in (-1, 1):
            F.box(_name('foot_rail'), 1.82, 0.05, 0.03, (0, 0.075, s * (out + 0.005)), M.wood, bevel=0.004)
            F.box(_name('foot_rail'), 0.03, 0.05, 1.82, (s * (out + 0.005), 0.075, 0), M.wood, bevel=0.004)
        # the acrylic rim, rounded, over the surround's top and the basin's walls
        rd = 0.93 - IN
        for s in (-1, 1):
            F.box(_name('rim'), 1.86, 0.08, rd, (0, RIM - 0.04, s * (IN + rd / 2)), M.shell, bevel=0.025, bevel_segments=3)
            F.box(_name('rim'), rd, 0.08, 1.86 - 2 * rd + 0.04, (s * (IN + rd / 2), RIM - 0.04, 0), M.shell, bevel=0.025,
                  bevel_segments=3)
        # the basin: its walls, the footwell's floor and the moulded bench along the back
        for s in (-1, 1):
            F.box(_name('wall'), 2 * IN + 0.04, RIM - FLOOR, 0.02, (0, (RIM + FLOOR) / 2 - 0.01, s * (IN + 0.01)), M.shell)
            F.box(_name('wall'), 0.02, RIM - FLOOR, 2 * IN + 0.04, (s * (IN + 0.01), (RIM + FLOOR) / 2 - 0.01, 0), M.shell)
        F.box('floor', 2 * IN, 0.04, 2 * IN, (0, FLOOR - 0.02, 0), M.shell)
        F.box('bench', 2 * IN, 0.25, 0.78, (0, FLOOR + 0.12, -IN + 0.39), M.shell, bevel=0.03, bevel_segments=3)
        for x in (-0.5, 0.5):                                # two seat hollows, a back rest up the wall
            F.box(_name('backrest'), 0.5, 0.26, 0.06, (x, 0.58, -IN + 0.03), M.shell, rot=(12, 0, 0), bevel=0.025,
                  bevel_segments=3)
        # jets: in the back rests, the side walls and the footwell
        for x in (-0.5, 0.5):
            for dx in (-0.12, 0.12):
                F.cyl(_name('jet'), 0.022, 0.012, (x + dx, 0.56, -IN + 0.075), M.chrome, rot=(78, 0, 0), verts=12)
            F.cyl(_name('jet'), 0.026, 0.012, (x, 0.34, IN - 0.004), M.chrome, rot=(90, 0, 0), verts=12)
        for s in (-1, 1):
            for z in (-0.35, 0.25):
                F.cyl(_name('jet'), 0.022, 0.012, (s * (IN - 0.004), 0.52, z), M.chrome, rot=(0, 90, 0), verts=12)
        # headrests on the back rim, the control panel on the right
        for x in (-0.5, 0.5):
            F.box(_name('headrest'), 0.36, 0.08, 0.17, (x, RIM + 0.03, -0.79), M.pillow, rot=(-14, 0, 0), bevel=0.035,
                  bevel_segments=3)
        F.box('panel', 0.12, 0.012, 0.16, (0.815, RIM + 0.004, -0.25), M.plinth, bevel=0.004)
        for k in range(3):
            F.cyl(_name('button'), 0.013, 0.008, (0.815, RIM + 0.012, -0.3 + 0.05 * k), M.chrome, verts=10)
        # towels at the front left, candles at the front right, a little plant at the back left
        F.box('towel_a', 0.4, 0.07, 0.19, (-0.6, RIM + 0.035, 0.815), M.towel, bevel=0.025, bevel_segments=2)
        F.box('towel_b', 0.38, 0.06, 0.18, (-0.6, RIM + 0.1, 0.815), M.towel2, bevel=0.022, bevel_segments=2)
        for x, z, h in ((0.78, 0.8, 0.12), (0.64, 0.84, 0.08), (0.8, 0.66, 0.06)):
            F.cyl(_name('candle'), 0.035, h, (x, RIM + h / 2, z), M.candle, verts=14)
            F.cyl(_name('wick'), 0.003, 0.015, (x, RIM + h + 0.007, z), M.plinth, verts=6)
        G.revolve('plant_pot', [(0, 0), (0.06, 0), (0.08, 0.11), (0.074, 0.11), (0, 0.1)], at=F.at(-0.81, RIM, -0.81),
                  material=M.terracotta, segments=14)
        for k in range(7):
            a = k * 360 / 7
            F.sphere(_name('leaf'), 0.05, (-0.81 + 0.04 * G.cosd(a), RIM + 0.15 + 0.02 * (k % 2), -0.81 + 0.04 * G.sind(a)),
                     M.leaf, scale=(1.4, 0.55, 0.4), rot=(0, 30, -a), segments=8, rings=5)
    G.join(still, 'body', origin=(0, 0, 0))
    # The water: the basin's inside up to WATER, its origin just under the footwell's floor; scaled flat
    # there at rest, `fill` raises it while residents soak.
    with G.new_parts() as water:
        F.box('water', 2 * IN - 0.008, WATER - FLOOR + 0.01, 2 * IN - 0.008, (0, (WATER + FLOOR - 0.01) / 2, 0), M.water)
    water = G.join(water, 'water', origin=F.at(0, FLOOR - 0.01, 0))
    water.scale[2] = 0.001
    # Bubbles welling up at the surface over the jets: domes just above and below it, joined with their
    # origin under the floor and scaled flat there; `bubbles` swells them up through the water and back
    # down, over and over while the soak lasts. Its rise takes as long as `fill`, from the same origin, so on
    # the first round the bubbles ride up on the rising water.
    with G.new_parts() as foam:
        import random
        rnd = random.Random(4)
        for cx, cz, n in ((-0.5, 0.3, 7), (0.5, 0.3, 7), (-0.5, -0.1, 5), (0.5, -0.1, 5), (0.0, 0.5, 5), (-0.62, 0.55, 3),
                          (0.62, 0.55, 3)):
            for _ in range(n):
                r = rnd.uniform(0.018, 0.042)
                F.sphere(_name('bubble'), r, (cx + rnd.uniform(-0.12, 0.12), WATER + rnd.uniform(-0.012, 0.004),
                                              cz + rnd.uniform(-0.1, 0.1)), M.foam, scale=(1, 1, 0.7), segments=8, rings=5)
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

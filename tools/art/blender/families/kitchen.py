# The kitchen: the top-freezer fridge (`model.fridge`), over the primitive-part placeholder of
# `furniture_prims.py` (`fridge_classic`), which stays the fallback. Notes: kitchen.md.
#
# A cabinet and two doors, each door its own node turning on its hinge, with a clip per door: the game
# opens the fridge door while a resident takes food or a drink out (`motions`). At rest both are closed.
# Built to the placeholder's size, in its numbers: the parts are placed in game axes (`F`).
#
# Run:  python3 -B tools/art/blender/build.py kitchen [--preview]
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    body=('#F3F3F1', 'gloss'),       # white cabinet and doors; the main part the player colours (`paint`)
    trim=('#B9BCBF', 'satin'),       # plinth and badge
    seal=('#3A3D41', 'satin'),       # the gap between the doors
    chrome=('#D0D3D6', 'chrome'),    # bar handles
    liner=('#DCE0E3', 'matte'),      # inside walls
    shelf=('#E9ECEE', 'gloss'),      # shelves, divider, tub
    carton=('#EAF0F4', 'satin'),     # milk and egg cartons
    bottle=('#4E8A54', 'gloss'),     # juice bottle
    produce=('#C0392B', 'satin'),    # apples
    produce2=('#E08A2E', 'satin'),   # orange
    egg=('#F2E6C9', 'matte'),        # eggs
    frozen=('#8FB6D6', 'satin'),     # frozen pizza box
    icecream=('#E8B4C8', 'satin'),   # ice cream tub
    peas=('#6FA96F', 'satin'),       # frozen peas
    porcelain=('#F6F6F4', 'gloss'),  # sink basins and the farmhouse apron
    stone=('#DCDDDA', 'gloss'),      # sink worktops
    wood=('#B98A5A', 'wood'),        # the farmhouse worktop
    board=('#B98A5A', 'satin'),      # the mini fridge's chopping board (plain: no draw call of its own)
    metal=('#8E9196', 'metal'),      # basin interiors
    dark=('#2F3236', 'satin'),       # drains
    plant=('#5C8A4A', 'satin'),      # a potted herb on the counter
    water=('#9EC9DD', 'gloss'),      # the stream from the tap
    # the rest of the fridge family (K1)
    steel=('#C3C7CB', 'metal'),      # a stainless French-door fridge
    panel=('#EDEBE6', 'satin'),      # built-in cabinet fronts and the wine fridge's body
    mint=('#BFE0D2', 'gloss'),       # a retro fridge's rounded body
    mint2=('#A9CFC0', 'gloss'),      # its trim (the plinth and the door seam)
    doorframe=('#2B2C2E', 'satin'),  # the wine fridge's dark door frame
    smoked=('#3A2A2E', 'glass'),     # its smoked glass
    wine=('#5A1E2A', 'gloss'),       # red wine bottles
    wine2=('#E3D9A0', 'gloss'),      # white wine bottles
)

# The placeholder's back sits 9.4 cm behind the middle of its tile (`furniture_prims.BACK = -0.455`);
# the model is centred on its tile, so every game depth is shifted back by that much.
F = G.GameFrame(shift=0.094)

# Both doors open at one speed; the short freezer door swings wider than the tall fridge door.
DOOR_SECONDS = 1.75
FREEZER_ANGLE = -118
FRIDGE_ANGLE = -102
# Seconds (the residents' animation time) a door stays opening or open once a use starts: it opens in
# 1.75 s, stays open while they reach in, then closes.
HOLD = 3.5
# The retro fridge: its lever turns out first, then the door swings (`delay`) while the lever springs back.
LEVER_SECONDS = 0.3
RETRO_OPEN = [{'clip': 'lever', 'hold': 0.4}, {'clip': 'fridge_open', 'delay': LEVER_SECONDS}]


def _bar(name, x, y0, y1, z, t=0.022, stand=0.03, frame=F):
    """A vertical bar handle on two stand-offs, its back on the door face at game depth `z`."""
    h = y1 - y0
    frame.box(f'{name}_bar', t, h, t, (x, (y0 + y1) / 2, z + stand + t / 2), M.chrome, bevel=t * 0.45)
    for i, y in enumerate((y0 + 0.03, y1 - 0.03)):
        frame.box(f'{name}_stand{i}', t * 0.8, t * 0.8, stand, (x, y, z + stand / 2), M.chrome, bevel=0.004)


def _shell():
    """The cabinet as five panels, so the doors open onto a real interior instead of a solid block."""
    F.box('side_left', 0.03, 1.74, 0.622, (-0.355, 0.93, -0.144), M.body, bevel=0.006)
    F.box('side_right', 0.03, 1.74, 0.622, (0.355, 0.93, -0.144), M.body, bevel=0.006)
    F.box('top', 0.74, 0.03, 0.622, (0, 1.785, -0.144), M.body, bevel=0.006)
    F.box('bottom', 0.74, 0.03, 0.622, (0, 0.075, -0.144), M.body, bevel=0.006)
    F.box('back', 0.74, 1.74, 0.03, (0, 0.93, -0.44), M.body, bevel=0.006)


def _interior():
    """The liner, shelves and food, seen once the doors are open."""
    F.box('liner_back', 0.678, 1.678, 0.006, (0, 0.93, -0.421), M.liner, bevel=0.002)
    F.box('liner_left', 0.006, 1.678, 0.578, (-0.337, 0.93, -0.134), M.liner, bevel=0.002)
    F.box('liner_right', 0.006, 1.678, 0.578, (0.337, 0.93, -0.134), M.liner, bevel=0.002)
    F.box('liner_top', 0.678, 0.006, 0.578, (0, 1.767, -0.134), M.liner, bevel=0.002)
    F.box('liner_bottom', 0.678, 0.006, 0.578, (0, 0.093, -0.134), M.liner, bevel=0.002)
    # the divider between the freezer (top) and the fridge (bottom), the freezer's floor
    F.box('divider', 0.672, 0.02, 0.55, (0, 1.234, -0.13), M.shelf, bevel=0.002)
    # three shelves in the fridge, one in the freezer
    for i, y in enumerate((0.37, 0.66, 0.95)):
        F.box(f'shelf{i}', 0.672, 0.008, 0.55, (0, y, -0.13), M.shelf, bevel=0.002)
    F.box('shelf_freezer', 0.672, 0.008, 0.55, (0, 1.52, -0.13), M.shelf, bevel=0.002)
    _fridge_food()
    _freezer_food()


def _fridge_food():
    """Normal food, in the bottom (fridge) compartment: dairy, produce and eggs."""
    top, mid, low = 0.954, 0.664, 0.374  # the fridge shelf surfaces
    F.box('milk', 0.085, 0.20, 0.085, (-0.20, top + 0.10, -0.14), M.carton, bevel=0.006)
    F.cone('milk_gable', 0.062, 0.0, 0.055, (-0.20, top + 0.2275, -0.14), M.carton, verts=4, rot=(0, 0, 45))
    F.cyl('juice', 0.032, 0.20, (0.02, top + 0.10, -0.14), M.bottle, verts=16, bevel=0.004)
    F.cyl('juice_cap', 0.014, 0.025, (0.02, top + 0.2125, -0.14), M.carton, verts=12)
    for i, x in enumerate((-0.20, -0.11, -0.02)):
        F.sphere(f'apple{i}', 0.038, (x, mid + 0.038, -0.15), M.produce, segments=10, rings=6)
    F.sphere('orange', 0.042, (0.13, mid + 0.042, -0.15), M.produce2, segments=10, rings=6)
    F.box('egg_carton', 0.16, 0.03, 0.11, (-0.16, low + 0.015, -0.14), M.carton, bevel=0.006)
    for i, x in enumerate((-0.205, -0.155, -0.105)):
        F.sphere(f'egg{i}', 0.017, (x, low + 0.03 + 0.013, -0.16), M.egg, segments=8, rings=5)
    F.box('tub', 0.13, 0.06, 0.13, (0.14, low + 0.03, -0.14), M.shelf, bevel=0.01)


def _freezer_food():
    """Frozen food, in the top (freezer) compartment: a pizza, ice cream and peas."""
    floor, shelf = 1.244, 1.524  # the divider's top and the freezer shelf
    F.box('pizza', 0.26, 0.035, 0.26, (-0.13, floor + 0.0175, -0.14), M.frozen, bevel=0.006)
    F.box('ice_cream', 0.12, 0.10, 0.12, (0.14, shelf + 0.05, -0.14), M.icecream, bevel=0.01)
    F.box('peas', 0.11, 0.15, 0.06, (-0.14, shelf + 0.075, -0.16), M.peas, bevel=0.008)


def _door(name, y, h, handle_y0, handle_y1, badge=False):
    """A door, rounded and proud of the opening, its handle on the left and its hinge on the right."""
    with G.new_parts() as parts:
        F.box(name, 0.728, h, 0.04, (0, y, 0.195), M.body, bevel=0.010)
        _bar(f'handle_{name}', -0.3, handle_y0, handle_y1, 0.215)
        if badge:
            F.box('badge', 0.1, 0.018, 0.004, (0.25, 1.74, 0.217), M.trim, bevel=0.002)
    return G.hinged(G.join(parts, name), F.at(0.364, y, 0.195))


def fridge():
    # the cabinet on a recessed plinth, opening onto a liner, shelves and food: the part that stays still
    with G.new_parts() as cabinet:
        F.box('plinth', 0.68, 0.06, 0.57, (0, 0.03, -0.17), M.trim, bevel=0.006)
        _shell()
        _interior()
        F.box('gap', 0.7, 0.014, 0.03, (0, 1.224, 0.187), M.seal, bevel=0.004)
    G.join(cabinet, 'cabinet', origin=(0, 0, 0))
    # the freezer door over the fridge door, a clip each
    freezer = _door('door_freezer', 1.51, 0.556, 1.274, 1.524, badge=True)
    door = _door('door_fridge', 0.643, 1.146, 0.804, 1.164)
    G.settle()   # where its placeholder stands (`centre`), its back on the wall
    G.swing(freezer, 'freezer_open', FREEZER_ANGLE, DOOR_SECONDS)
    G.swing(door, 'fridge_open', FRIDGE_ANGLE, DOOR_SECONDS)


# --- the sinks ----------------------------------------------------------------------------------------
# Shared by the two kitchen sinks (the same helpers as families/bathroom.py; a family script stays
# self-contained). The tap and its stream are built in the sink's own GameFrame.
def _tube(F, name, pts, r, material, verts=8):
    """A tube through game points: a rod between each pair, a sphere at each joint."""
    bpts = [F.at(*p) for p in pts]
    for i in range(len(bpts) - 1):
        G.rod(f'{name}_{i}', bpts[i], bpts[i + 1], r, material=material, verts=verts)
    for i, p in enumerate(pts[1:-1], 1):
        F.sphere(f'{name}_j{i}', r, p, material, segments=8, rings=5)


def _gooseneck(F, name, x, base_y, z, spout_z, top, material, r=0.022, tip_y=None):
    """A tap: a riser from (x, base_y, z) up to `top`, arcing forward to a downward spout. Returns the
    spout tip's game point (where the water starts)."""
    tip_y = tip_y if tip_y is not None else base_y + 0.16
    pts = [(x, base_y, z),
           (x, top - 0.05, z),
           (x, top, z + (spout_z - z) * 0.35),
           (x, top, z + (spout_z - z) * 0.8),
           (x, top - 0.03, spout_z),
           (x, tip_y + 0.04, spout_z),
           (x, tip_y, spout_z)]
    _tube(F, name, pts, r, material)
    return (x, tip_y, spout_z)


def _rect_basin(F, name, x, z, w, d, floor_y, rim_y, material, wall=0.02):
    """A rectangular basin shell: a floor and four thin walls, so it reads as a recessed sink."""
    F.box(f'{name}_floor', w, wall, d, (x, floor_y + wall / 2, z), material, bevel=0.008)
    for s in (-1, 1):
        F.box(f'{name}_side{s}', wall, rim_y - floor_y, d, (x + s * (w - wall) / 2, (floor_y + rim_y) / 2, z),
              material, bevel=0.008)
        F.box(f'{name}_end{s}', w - 2 * wall, rim_y - floor_y, wall,
              (x, (floor_y + rim_y) / 2, z + s * (d - wall) / 2), material, bevel=0.008)


def _water(F, spout, floor_y):
    """The stream: a thin cylinder from the spout tip down to the basin, joined with its origin at the
    tip and scaled to nothing along its length, so it is hidden until `grow` runs the clip."""
    x, top_y, z = spout
    length = max(0.06, top_y - floor_y)
    with G.new_parts() as parts:
        F.cyl('water', 0.012, length, (x, (top_y + floor_y) / 2, z), M.water, verts=10)
    obj = G.join(parts, 'water', origin=F.at(x, top_y, z))
    obj.scale[2] = 0.001
    return obj


# --- farmhouse sink: a white apron basin in a painted cabinet under a wood worktop ---------------------
def farmhouseSink():
    Fk = G.GameFrame(shift=0.145)
    with G.new_parts() as still:
        Fk.box('plinth', 0.96, 0.08, 0.56, (0, 0.04, -0.14), M.trim, bevel=0.006)
        Fk.box('cabinet', 0.98, 0.66, 0.58, (0, 0.39, -0.13), M.body, bevel=0.012)
        for s in (-1, 1):
            Fk.box(f'door{s}', 0.4, 0.62, 0.02, (s * 0.22, 0.42, 0.165), M.body, bevel=0.008)
            Fk.box(f'door_panel{s}', 0.3, 0.52, 0.012, (s * 0.22, 0.42, 0.177), M.body, bevel=0.004)
            Fk.box(f'pull{s}', 0.012, 0.12, 0.02, (s * 0.06, 0.42, 0.18), M.chrome, bevel=0.004)
        # a wood worktop framing the apron basin, and the painted cabinet shoulders up to it
        Fk.box('top_back', 1.0, 0.04, 0.15, (0, 0.94, -0.405), M.wood, bevel=0.008)
        Fk.box('top_left', 0.17, 0.04, 0.51, (-0.415, 0.94, -0.075), M.wood, bevel=0.008)
        Fk.box('top_right', 0.17, 0.04, 0.51, (0.415, 0.94, -0.075), M.wood, bevel=0.008)
        Fk.box('shoulder_back', 1.0, 0.2, 0.15, (0, 0.82, -0.405), M.body, bevel=0.008)
        Fk.box('shoulder_left', 0.17, 0.2, 0.51, (-0.415, 0.82, -0.075), M.body, bevel=0.008)
        Fk.box('shoulder_right', 0.17, 0.2, 0.51, (0.415, 0.82, -0.075), M.body, bevel=0.008)
        # the white apron basin: a shell proud of the cabinet front
        _rect_basin(Fk, 'apron', 0, -0.075, 0.66, 0.51, 0.72, 0.96, M.porcelain, wall=0.03)
        Fk.box('basin_floor', 0.6, 0.01, 0.45, (0, 0.785, -0.075), M.metal, bevel=0.004)
        Fk.cyl('drain', 0.035, 0.012, (0, 0.79, -0.075), M.dark, verts=12)
        # tap, a side handle, a soap dish and a potted herb
        Fk.cyl('tap_base', 0.05, 0.03, (0, 0.955, -0.38), M.chrome, verts=14)
        Fk.cyl('tap_handle', 0.03, 0.05, (0.12, 0.975, -0.38), M.chrome, verts=12)
        spout = _gooseneck(Fk, 'tap', 0, 0.94, -0.38, -0.2, 1.235, M.chrome, r=0.028, tip_y=1.17)
        Fk.box('soap_dish', 0.2, 0.05, 0.12, (0.36, 0.985, -0.36), M.porcelain, bevel=0.02)
        Fk.cyl('pot', 0.06, 0.14, (-0.36, 1.03, -0.38), M.plant, verts=14)
        Fk.sphere('herb', 0.07, (-0.36, 1.14, -0.38), M.plant, segments=10, rings=6, scale=(1, 1.2, 1))
    G.join(still, 'body', origin=(0, 0, 0))
    water = _water(Fk, spout, 0.79)
    G.settle()
    G.grow(water, 'water_on', seconds=0.4)
    return G.meshes()


# --- double counter sink: two basins in a two-metre counter run ----------------------------------------
# catalog2's 0.79-deep counter numbers drawn 0.66 deep against the wall (kitchen_counters.py COUNTER66)
COUNTER66 = ((-0.48, -0.43), (-0.46, -0.41), (0.24, 0.19), (0.31, 0.23))


def counterSinkDouble():
    Fk = G.GameFrame(shift=0.085, depths=COUNTER66)
    with G.new_parts() as still:
        Fk.box('plinth', 1.996, 0.1, 0.64, (0, 0.05, -0.14), M.trim, bevel=0.006)
        Fk.box('carcass', 1.996, 0.78, 0.7, (0, 0.49, -0.11), M.body, bevel=0.012)
        for i, x in enumerate((-0.74, -0.26, 0.26, 0.74)):
            Fk.box(f'door{i}', 0.468, 0.74, 0.02, (x, 0.49, 0.25), M.body, bevel=0.008)
            Fk.box(f'door_panel{i}', 0.368, 0.64, 0.012, (x, 0.49, 0.262), M.body, bevel=0.004)
            _bar(f'handle{i}', x + (0.19 if i in (0, 2) else -0.19), 0.3, 0.5, 0.26, frame=Fk, stand=0.014)
        # a worktop framing the two basins
        Fk.box('top_back', 2.0, 0.07, 0.18, (0, 0.915, -0.39), M.stone, bevel=0.008)
        Fk.box('top_front', 2.0, 0.07, 0.13, (0, 0.915, 0.245), M.stone, bevel=0.008)
        Fk.box('top_left', 0.25, 0.07, 0.48, (-0.875, 0.915, -0.06), M.stone, bevel=0.008)
        Fk.box('top_right', 0.25, 0.07, 0.48, (0.875, 0.915, -0.06), M.stone, bevel=0.008)
        Fk.box('top_mid', 0.18, 0.07, 0.48, (0, 0.915, -0.06), M.stone, bevel=0.008)
        for s in (-1, 1):
            x = s * 0.42
            _rect_basin(Fk, f'basin{s}', x, -0.06, 0.66, 0.48, 0.88, 0.95, M.metal)
            Fk.cyl(f'drain{s}', 0.035, 0.012, (x, 0.89, -0.06), M.dark, verts=12)
            Fk.cyl(f'tap_base{s}', 0.05, 0.03, (x, 0.955, -0.38), M.chrome, verts=14)
            _gooseneck(Fk, f'tap{s}', x, 0.95, -0.38, -0.18, 1.235, M.chrome, r=0.03, tip_y=1.17)
        Fk.box('soap_dish', 0.2, 0.04, 0.14, (0, 0.97, -0.34), M.porcelain, bevel=0.02)
        Fk.cyl('pot', 0.06, 0.14, (0.05, 1.03, -0.36), M.plant, verts=14)
        Fk.sphere('herb', 0.07, (0.05, 1.14, -0.36), M.plant, segments=10, rings=6, scale=(1, 1.2, 1))
    G.join(still, 'body', origin=(0, 0, 0))
    waters = [_water(Fk, (x, 1.17, -0.18), 0.89) for x in (-0.42, 0.42)]
    G.settle()
    for water in waters:
        G.grow(water, 'water_on', seconds=0.4)
    return G.meshes()


# --- K1: the rest of the fridge family ---------------------------------------------------------------
# Five more fridges, built to their placeholders' sizes with the classic fridge's mechanics: a door
# hinged on one edge (`G.hinged`) with `G.swing`, and a `hold` so it shuts while the resident is busy.
# The French-door freezer and the wine fridge's top rack slide (`G.slide`).


def _small_bar(F, name, x, y0, y1, z, t=0.02, stand=0.014):
    """A short vertical bar handle, its back on the door face at game depth `z` (a slimmer `_bar`)."""
    F.box(f'{name}_bar', t, y1 - y0, t, (x, (y0 + y1) / 2, z + stand + t / 2), M.chrome, bevel=t * 0.45)
    for i, y in enumerate((y0 + 0.03, y1 - 0.03)):
        F.box(f'{name}_stand{i}', t * 0.8, t * 0.8, stand, (x, y, z + stand / 2), M.chrome, bevel=0.004)


def _fridgeMini():
    """A small fridge: a white cabinet with one door over its whole front, a board and a jar on top."""
    Fm = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        Fm.box('plinth', 0.56, 0.04, 0.52, (0, 0.02, -0.025), M.trim, bevel=0.006)
        Fm.box('side_left', 0.03, 0.80, 0.54, (-0.285, 0.44, -0.015), M.body, bevel=0.006)
        Fm.box('side_right', 0.03, 0.80, 0.54, (0.285, 0.44, -0.015), M.body, bevel=0.006)
        Fm.box('top', 0.60, 0.03, 0.54, (0, 0.855, -0.015), M.body, bevel=0.006)
        Fm.box('bottom', 0.60, 0.03, 0.54, (0, 0.055, -0.015), M.body, bevel=0.006)
        Fm.box('back', 0.60, 0.80, 0.03, (0, 0.44, -0.27), M.body, bevel=0.006)
        # the liner, a divider (the freezer floor) and a shelf
        Fm.box('liner', 0.54, 0.74, 0.01, (0, 0.44, -0.25), M.liner, bevel=0.002)
        Fm.box('divider', 0.54, 0.02, 0.50, (0, 0.66, -0.015), M.shelf, bevel=0.002)
        Fm.box('shelf', 0.54, 0.015, 0.50, (0, 0.40, -0.015), M.shelf, bevel=0.002)
        # food
        Fm.box('milk', 0.07, 0.15, 0.07, (-0.15, 0.50, -0.04), M.carton, bevel=0.006)
        Fm.cone('milk_top', 0.05, 0.0, 0.045, (-0.15, 0.5975, -0.04), M.carton, verts=4, rot=(0, 0, 45))
        for i, x in enumerate((0.03, 0.11)):
            Fm.sphere(f'apple{i}', 0.035, (x, 0.4175, -0.05), M.produce, segments=10, rings=6)
        # a chopping board, a jar and an apple on top
        Fm.box('board', 0.50, 0.02, 0.40, (0, 0.88, -0.02), M.board, bevel=0.006)
        Fm.cone('jar', 0.05, 0.04, 0.12, (-0.14, 0.94, -0.04), M.porcelain, verts=14)
        Fm.sphere('apple_top', 0.045, (0.12, 0.92, 0.0), M.produce, segments=10, rings=6)
    G.join(still, 'body', origin=(0, 0, 0))
    # the door over the whole front, hinged on its left edge
    with G.new_parts() as parts:
        Fm.box('door_low', 0.58, 0.62, 0.03, (0, 0.40, 0.27), M.body, bevel=0.010)
        Fm.box('door_high', 0.58, 0.16, 0.03, (0, 0.78, 0.27), M.body, bevel=0.010)
        Fm.box('seal', 0.50, 0.018, 0.026, (0, 0.705, 0.285), M.trim, bevel=0.004)
        _small_bar(Fm, 'handle', 0.24, 0.36, 0.58, 0.285)
    door = G.hinged(G.join(parts, 'door'), Fm.at(-0.29, 0.40, 0.27))
    G.settle()
    G.swing(door, 'fridge_open', 100, seconds=1.6)


def _fridgeBuiltIn():
    """A tall fridge-freezer behind cabinet fronts, a fridge door over a freezer door, long bar handles."""
    Fb = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        Fb.box('plinth', 0.78, 0.10, 0.56, (0, 0.05, -0.04), M.dark, bevel=0.004)
        Fb.box('side_left', 0.03, 1.88, 0.58, (-0.395, 1.06, -0.03), M.panel, bevel=0.006)
        Fb.box('side_right', 0.03, 1.88, 0.58, (0.395, 1.06, -0.03), M.panel, bevel=0.006)
        Fb.box('top', 0.82, 0.03, 0.58, (0, 1.995, -0.03), M.panel, bevel=0.006)
        Fb.box('bottom', 0.82, 0.03, 0.58, (0, 0.115, -0.03), M.panel, bevel=0.006)
        Fb.box('back', 0.82, 1.88, 0.03, (0, 1.06, -0.30), M.panel, bevel=0.006)
        Fb.box('cornice', 0.84, 0.05, 0.62, (0, 2.00, -0.005), M.panel, bevel=0.008)
        Fb.box('vent', 0.60, 0.05, 0.01, (0, 0.07, 0.24), M.dark, bevel=0.004)
        # the liner, the divider between freezer (below) and fridge (above), and the shelves
        Fb.box('liner_back', 0.76, 1.72, 0.01, (0, 1.06, -0.28), M.liner, bevel=0.002)
        Fb.box('liner_left', 0.006, 1.72, 0.54, (-0.377, 1.06, -0.02), M.liner, bevel=0.002)
        Fb.box('liner_right', 0.006, 1.72, 0.54, (0.377, 1.06, -0.02), M.liner, bevel=0.002)
        Fb.box('divider', 0.76, 0.02, 0.52, (0, 0.80, -0.02), M.shelf, bevel=0.002)
        for i, y in enumerate((1.08, 1.38, 1.68)):
            Fb.box(f'shelf{i}', 0.76, 0.008, 0.52, (0, y, -0.02), M.shelf, bevel=0.002)
        # food, fresh above and frozen below
        Fb.box('milk', 0.08, 0.19, 0.08, (-0.22, 1.20, -0.03), M.carton, bevel=0.006)
        Fb.cyl('juice', 0.032, 0.19, (0.0, 1.19, -0.03), M.bottle, verts=16, bevel=0.004)
        for i, x in enumerate((-0.20, -0.11, -0.02)):
            Fb.sphere(f'apple{i}', 0.036, (x, 1.418, -0.05), M.produce, segments=10, rings=6)
        Fb.box('egg_carton', 0.16, 0.03, 0.11, (-0.16, 1.40, -0.03), M.carton, bevel=0.006)
        Fb.box('pizza', 0.24, 0.03, 0.24, (-0.12, 0.815, -0.03), M.frozen, bevel=0.006)
        Fb.box('ice_cream', 0.12, 0.10, 0.12, (0.16, 0.86, -0.03), M.icecream, bevel=0.01)
    G.join(still, 'body', origin=(0, 0, 0))
    # the tall fridge door, hinged on its left edge, with its water dispenser and handle
    with G.new_parts() as parts:
        Fb.box('fridge_door', 0.78, 1.14, 0.025, (0, 1.39, 0.30), M.panel, bevel=0.010)
        Fb.box('water', 0.14, 0.20, 0.012, (-0.24, 1.42, 0.315), M.dark, bevel=0.012)
        Fb.box('spout', 0.08, 0.025, 0.005, (-0.24, 1.48, 0.322), M.water, bevel=0.002)
        _small_bar(Fb, 'handle_fridge', 0.33, 1.08, 1.70, 0.3125, t=0.02, stand=0.010)
    fridge_door = G.hinged(G.join(parts, 'fridge_door'), Fb.at(-0.39, 1.39, 0.30))
    # the freezer door below, same hinge, a clip of its own (unused until a use takes frozen food)
    with G.new_parts() as parts:
        Fb.box('freezer_door', 0.78, 0.62, 0.025, (0, 0.50, 0.30), M.panel, bevel=0.010)
        Fb.box('handle_freezer', 0.50, 0.022, 0.032, (0, 0.72, 0.315), M.chrome, bevel=0.008)
    freezer = G.hinged(G.join(parts, 'freezer_door'), Fb.at(-0.39, 0.50, 0.30))
    G.settle()
    G.swing(fridge_door, 'fridge_open', 100, seconds=DOOR_SECONDS)
    G.swing(freezer, 'freezer_open', 100, seconds=DOOR_SECONDS)


def _fridgeFrenchDoor():
    """A stainless French-door fridge: two upper doors, a full-width freezer drawer below."""
    Ff = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        Ff.box('plinth', 0.86, 0.04, 0.66, (0, 0.02, -0.05), M.trim, bevel=0.006)
        Ff.box('side_left', 0.03, 1.76, 0.66, (-0.435, 0.92, -0.05), M.steel, bevel=0.008)
        Ff.box('side_right', 0.03, 1.76, 0.66, (0.435, 0.92, -0.05), M.steel, bevel=0.008)
        Ff.box('top', 0.90, 0.03, 0.66, (0, 1.805, -0.05), M.steel, bevel=0.008)
        Ff.box('bottom', 0.90, 0.03, 0.66, (0, 0.035, -0.05), M.steel, bevel=0.008)
        Ff.box('back', 0.90, 1.76, 0.03, (0, 0.92, -0.365), M.steel, bevel=0.008)
        # the liner, the divider above the freezer and the fridge shelves
        Ff.box('liner_back', 0.84, 1.70, 0.01, (0, 0.92, -0.345), M.liner, bevel=0.002)
        Ff.box('divider', 0.84, 0.02, 0.64, (0, 0.68, -0.05), M.shelf, bevel=0.002)
        for i, y in enumerate((0.96, 1.24, 1.52)):
            Ff.box(f'shelf{i}', 0.84, 0.008, 0.64, (0, y, -0.05), M.shelf, bevel=0.002)
        # food
        Ff.box('milk', 0.08, 0.19, 0.08, (-0.24, 1.06, -0.06), M.carton, bevel=0.006)
        Ff.cyl('juice', 0.033, 0.20, (0.02, 1.06, -0.06), M.bottle, verts=16, bevel=0.004)
        for i, x in enumerate((-0.24, -0.15, -0.06)):
            Ff.sphere(f'apple{i}', 0.036, (x, 1.278, -0.08), M.produce, segments=10, rings=6)
        Ff.box('egg_carton', 0.16, 0.03, 0.11, (-0.20, 1.26, -0.06), M.carton, bevel=0.006)
        Ff.box('pizza', 0.26, 0.03, 0.26, (-0.10, 0.71, -0.06), M.frozen, bevel=0.006)
    G.join(still, 'body', origin=(0, 0, 0))
    # the left upper door, hinged on its left edge, and the right, hinged on its right
    with G.new_parts() as parts:
        Ff.box('door_left', 0.436, 1.10, 0.03, (-0.222, 1.25, 0.345), M.steel, bevel=0.014)
        Ff.box('handle_left', 0.022, 0.70, 0.03, (-0.03, 1.25, 0.36), M.chrome, bevel=0.008)
    door_left = G.hinged(G.join(parts, 'door_left'), Ff.at(-0.44, 1.25, 0.345))
    with G.new_parts() as parts:
        Ff.box('door_right', 0.436, 1.10, 0.03, (0.222, 1.25, 0.345), M.steel, bevel=0.014)
        Ff.box('handle_right', 0.022, 0.70, 0.03, (0.03, 1.25, 0.36), M.chrome, bevel=0.008)
    door_right = G.hinged(G.join(parts, 'door_right'), Ff.at(0.44, 1.25, 0.345))
    # the freezer drawer, sliding out to the front
    with G.new_parts() as parts:
        Ff.box('drawer', 0.88, 0.58, 0.03, (0, 0.38, 0.345), M.steel, bevel=0.014)
        Ff.box('drawer_tub', 0.82, 0.50, 0.40, (0, 0.38, 0.12), M.liner, bevel=0.006)
        Ff.box('handle_drawer', 0.60, 0.014, 0.022, (0, 0.60, 0.36), M.chrome, bevel=0.005)
    drawer = G.join(parts, 'drawer', origin=Ff.at(0, 0.38, 0.345))
    G.settle()
    G.swing(door_left, 'fridge_open', 100, seconds=DOOR_SECONDS)
    G.swing(door_right, 'fridge_open', -100, seconds=DOOR_SECONDS)
    G.swing(door_right, 'door_right', -100, seconds=DOOR_SECONDS)
    G.slide(drawer, 'freezer_open', (0, -0.35, 0), seconds=DOOR_SECONDS)


def _fridgeRetro():
    """A rounded mint fridge: a single door with a chrome lever handle and a top trim."""
    Fr = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        Fr.box('plinth', 0.66, 0.08, 0.60, (0, 0.04, -0.05), M.mint2, bevel=0.010)
        Fr.box('side_left', 0.04, 1.52, 0.62, (-0.34, 0.84, -0.05), M.mint, bevel=0.02)
        Fr.box('side_right', 0.04, 1.52, 0.62, (0.34, 0.84, -0.05), M.mint, bevel=0.02)
        Fr.box('top', 0.72, 0.04, 0.62, (0, 1.62, -0.05), M.mint, bevel=0.02)
        Fr.box('bottom', 0.72, 0.04, 0.62, (0, 0.10, -0.05), M.mint, bevel=0.02)
        Fr.box('back', 0.72, 1.52, 0.04, (0, 0.84, -0.34), M.mint, bevel=0.02)
        # the liner, a divider and two shelves
        Fr.box('liner', 0.64, 1.46, 0.01, (0, 0.84, -0.32), M.liner, bevel=0.002)
        Fr.box('divider', 0.64, 0.02, 0.56, (0, 1.16, -0.05), M.shelf, bevel=0.002)
        for i, y in enumerate((0.45, 0.75)):
            Fr.box(f'shelf{i}', 0.64, 0.008, 0.56, (0, y, -0.05), M.shelf, bevel=0.002)
        Fr.box('milk', 0.08, 0.19, 0.08, (-0.18, 0.56, -0.06), M.carton, bevel=0.006)
        for i, x in enumerate((-0.18, -0.09, 0.0)):
            Fr.sphere(f'apple{i}', 0.036, (x, 0.786, -0.08), M.produce, segments=10, rings=6)
    G.join(still, 'body', origin=(0, 0, 0))
    # the rounded door, hinged on its left edge, with a top handle
    with G.new_parts() as parts:
        Fr.box('door', 0.68, 1.50, 0.04, (0, 0.84, 0.30), M.mint, bevel=0.06)
        Fr.box('seam', 0.68, 0.012, 0.02, (0, 1.18, 0.325), M.mint2, bevel=0.004)
        Fr.box('top_handle', 0.18, 0.05, 0.012, (0, 1.5, 0.325), M.chrome, bevel=0.01)
    door = G.hinged(G.join(parts, 'door'), Fr.at(-0.36, 0.84, 0.30))
    # the chrome lever on the door, pivoting on its top: pulled out first, it unlatches the door
    with G.new_parts() as parts:
        Fr.box('handle', 0.06, 0.24, 0.05, (0.27, 1.0, 0.34), M.chrome, bevel=0.02)
    lever = G.hinged(G.join(parts, 'lever'), Fr.at(0.27, 1.12, 0.33))
    G.settle()
    G.attach(lever, door)
    G.swing(lever, 'lever', -30, seconds=LEVER_SECONDS, axis='X')   # its foot out to the front
    G.swing(door, 'fridge_open', 100, seconds=DOOR_SECONDS)


def _wineFridge():
    """A dark glass door over a bottle grid, a stack of drawer racks beside it; the top rack slides."""
    Fw = G.GameFrame(shift=0.0, depths=tuple((a + 0.085, b) for a, b in COUNTER66))   # its numbers centred
    with G.new_parts() as still:
        Fw.box('plinth', 0.996, 0.10, 0.64, (0, 0.05, -0.075), M.dark, bevel=0.004)
        Fw.box('side_left', 0.03, 0.80, 0.70, (-0.483, 0.50, -0.045), M.panel, bevel=0.006)
        Fw.box('side_right', 0.03, 0.80, 0.70, (0.483, 0.50, -0.045), M.panel, bevel=0.006)
        Fw.box('top', 1.00, 0.05, 0.79, (0, 0.925, 0.0), M.stone, bevel=0.008)
        Fw.box('bottom', 0.996, 0.03, 0.70, (0, 0.115, -0.045), M.panel, bevel=0.006)
        Fw.box('back', 0.996, 0.80, 0.03, (0, 0.50, -0.38), M.panel, bevel=0.006)
        Fw.box('liner', 0.90, 0.74, 0.01, (0, 0.50, -0.36), M.liner, bevel=0.002)
        # the bottle grid on the left: five shelf rows of three bottles
        for r, y in enumerate((0.22, 0.35, 0.48, 0.61, 0.74)):
            Fw.box(f'rack_left{r}', 0.46, 0.01, 0.52, (-0.22, y, -0.06), M.shelf, bevel=0.0)
            for c, x in enumerate((-0.40, -0.25, -0.10)):
                Fw.cyl(f'bottle{r}_{c}', 0.035, 0.30, (x, y + 0.04, -0.06), M.wine if (r + c) % 2 == 0 else M.wine2,
                       rot=(90, 0, 0), verts=8, bevel=0.0)
        # two static drawer racks on the right; the top one is the moving part
        for r, y in enumerate((0.30, 0.55)):
            Fw.box(f'rack_right{r}', 0.37, 0.2347, 0.02, (0.30, y, 0.25), M.panel, bevel=0.008)
            Fw.box(f'rack_handle{r}', 0.16, 0.014, 0.022, (0.30, y + 0.0617, 0.27), M.chrome, bevel=0.005)
    G.join(still, 'body', origin=(0, 0, 0))
    # the smoked-glass door, hinged on its left edge
    with G.new_parts() as parts:
        Fw.box('doorframe', 0.58, 0.76, 0.03, (-0.20, 0.50, 0.325), M.doorframe, bevel=0.010)
        Fw.box('glass', 0.48, 0.64, 0.012, (-0.20, 0.50, 0.345), M.smoked, bevel=0.004)
        Fw.box('door_handle', 0.014, 0.40, 0.022, (0.06, 0.50, 0.35), M.chrome, bevel=0.005)
    door = G.hinged(G.join(parts, 'door'), Fw.at(-0.49, 0.50, 0.325))
    # the top drawer rack, sliding out 15 cm
    with G.new_parts() as parts:
        Fw.box('top_rack', 0.37, 0.2347, 0.02, (0.30, 0.74, 0.25), M.panel, bevel=0.008)
        Fw.box('top_rack_handle', 0.16, 0.014, 0.022, (0.30, 0.8017, 0.27), M.chrome, bevel=0.005)
        Fw.box('top_rack_tray', 0.35, 0.20, 0.30, (0.30, 0.74, 0.09), M.liner, bevel=0.002)
    rack = G.join(parts, 'top_rack', origin=Fw.at(0.30, 0.74, 0.25))
    G.settle()
    G.swing(door, 'door_open', 100, seconds=1.5)
    G.slide(rack, 'door_open', (0, -0.15, 0), seconds=1.5)


# A floor thing against a wall is centred where its placeholder stands (`centre`): its back on the wall,
# its top under its content `surface` (catalog2.py TOPS).
FAMILY = G.Family(
    'kitchen', M,
    [G.Model('fridge', fridge, size=(0.74, 1.8, 0.722), key='model.fridge', centre=(0, -0.069),
             # Taking food or a drink out opens the fridge door (snacks, a meal's ingredients (cook), a cold
             # drink, putting dishes away); nothing opens the freezer yet: no use takes frozen food.
             motions={'uses': {'fetch': 'fridge_open', 'cook': 'fridge_open', 'drink': 'fridge_open'}, 'hold': HOLD},
             # The player colours the cabinet and doors, the handles, the plinth and the inside; the
             # food keeps its own colours.
             paint=[('body', 'Body'), ('chrome', 'Handles'), ('trim', 'Plinth & badge'), ('liner', 'Inside')]),
     G.Model('fridgeMini', _fridgeMini, size=(0.60, 1.00, 0.60), key='model.fridgeMini', centre=(0, -0.128),
             motions={'uses': {'fetch': 'fridge_open', 'cook': 'fridge_open'}, 'hold': 3.0},
             paint=[('body', 'Body'), ('chrome', 'Handles'), ('trim', 'Plinth')]),
     G.Model('fridgeBuiltIn', _fridgeBuiltIn, size=(0.84, 2.025, 0.656), key='model.fridgeBuiltIn', centre=(0, -0.099),
             # the tall door serves every use; the freezer door's clip waits for a use that takes frozen food
             motions={'uses': {'fetch': 'fridge_open', 'cook': 'fridge_open', 'drink': 'fridge_open'}, 'hold': HOLD},
             paint=[('panel', 'Panels'), ('chrome', 'Handles')]),
     G.Model('fridgeFrenchDoor', _fridgeFrenchDoor, size=(0.90, 1.82, 0.76), key='model.fridgeFrenchDoor', centre=(0, -0.052),
             # both doors for a snack or a meal; the right door alone for a cold drink
             motions={'uses': {'fetch': 'fridge_open', 'cook': 'fridge_open', 'drink': 'door_right'}, 'hold': HOLD},
             paint=[('steel', 'Body'), ('chrome', 'Handles'), ('trim', 'Plinth')]),
     G.Model('fridgeRetro', _fridgeRetro, size=(0.72, 1.64, 0.715), key='model.fridgeRetro', centre=(0, -0.067),
             motions={'uses': {use: RETRO_OPEN for use in ('fetch', 'cook', 'drink')}, 'hold': HOLD},
             paint=[('mint', 'Body'), ('chrome', 'Handle'), ('mint2', 'Trim')]),
     G.Model('wineFridge', _wineFridge, size=(1.00, 0.95, 0.66), key='model.wineFridge', centre=(0, -0.10),
             motions={'uses': {'drink': 'door_open'}, 'hold': 2.5},
             paint=[('panel', 'Body'), ('doorframe', 'Door frame')]),
     G.Model('farmhouseSink', farmhouseSink, size=(1.0, 1.265, 0.67), key='model.farmhouseSink', centre=(0, -0.095),
             motions={'uses': {'wash': 'water_on'}},
             paint=[('body', 'Cabinet'), ('wood', 'Worktop'), ('porcelain', 'Basin'), ('chrome', 'Tap & handles'),
                    ('trim', 'Plinth')]),
     G.Model('counterSinkDouble', counterSinkDouble, size=(2.0, 1.265, 0.66), key='model.counterSinkDouble', centre=(0, -0.10),
             motions={'uses': {'wash': 'water_on'}},
             paint=[('body', 'Cabinets'), ('stone', 'Worktop'), ('metal', 'Basins'), ('chrome', 'Tap & handles'),
                    ('trim', 'Plinth')])],
    comment='Blender-built kitchen models (tools/art/blender/families/kitchen.py). Replaces the '
            'primitive-part placeholders by key; listed in the base manifest\'s packs. The placeholder is '
            'kept as a fallback if a file fails to load, and the materials map onto the game\'s finishes so '
            'a model costs one draw call per finish group. Each sink has a `water_on` clip (the stream from '
            'the tap) played while a resident washes up. The fridges (K1) each hinge a door built closed and '
            'swing it open while a resident takes food or a drink (`motions`); the French-door freezer and the '
            'wine fridge\'s top rack slide, and the built-in freezer door and French freezer drawer carry clips '
            'no use plays yet.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

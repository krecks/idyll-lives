# The kitchen's base counters (K4): the plain counter and its corner, the counter end with open
# shelves, the outer corner counter, the drawer counter, the open-shelf counter, the butcher block and
# the marble counter. Each replaces the primitive-part placeholder of `catalog.py` / `catalog2.py`,
# which stays the fallback if the file fails to load. Notes: kitchen_counters.md.
#
# A base counter is a plinth, a carcass and a worktop; the front carries a drawer over two doors (the
# plain, corner and marble counters), four drawers, one door, or nothing (the open shelves). The four
# with a `sandwich` use slide their top drawer out 25 cm. The parts are placed in game axes (`F`): the
# placeholder's depths are shifted back by the placeholder's depth centre, so the model is centred on
# its tile (see GameFrame). There is one worktop height, 0.95, the content's `surface` (catalog2.py CT),
# for the catalogue counters (`_unit66`) and the catalog2 ones (`_unit79`) alike. There is one depth, the
# catalogue's 0.66: the catalog2 counters keep their placeholder's old 0.79-deep numbers and are drawn
# 0.66 deep through `COUNTER66` (as catalog2.py `shallow`), so the fronts of a mixed run line up. Every
# back stands on the wall face (-0.43).
#
# Run:  python3 -B tools/art/blender/build.py kitchen_counters [--preview]
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    cab=('#EDEBE6', 'satin'),        # a cabinet carcass and its fronts; the main part the player colours
    worktop=('#E8E6E1', 'gloss'),    # a laminate worktop
    dark=('#2F3236', 'satin'),       # plinths
    chrome=('#D0D3D6', 'chrome'),    # handles and pulls
    oak=('#CDB08A', 'wood'),         # a wood end unit, open shelves and a utensil handle
    navy=('#3E5470', 'satin'),       # a painted cabinet (the marble counter)
    marble=('#F4F3F0', 'gloss'),     # a marble worktop
    vein=('#C9CBCC', 'gloss'),       # a vein through the marble
    brass=('#B8925A', 'metal'),      # brass handles
    block=('#B98A5A', 'wood'),       # a butcher-block top
    block2=('#C99A68', 'wood'),      # the top layer of the block
    leg=('#9C6B43', 'wood'),         # the block's legs and shelf
    wood2=('#8A5A38', 'satin'),      # the knife drawer's front (plain, so it draws with the body's group)
    spoon=('#CDB08A', 'satin'),      # the corner counter's wooden spoons (plain: no draw call of their own)
    basket=('#C9A979', 'satin'),     # a basket
    metal=('#2B2C2E', 'metal'),      # a dark metal handle
    porcelain=('#F5F3EE', 'gloss'),  # plates, a bowl, a jar
    cream=('#EFE5D3', 'gloss'),      # a cream jar
    rust=('#B5653F', 'gloss'),       # a terracotta pot
    teal=('#4E7A7F', 'gloss'),       # a teal pot
    sage=('#B9C9C4', 'gloss'),       # a sage pot
    yellow=('#F2C14E', 'satin'),     # a lemon
    soil=('#3B2E24', 'matte'),       # the dark back of an open shelf
    oakd=('#8A5A38', 'wood'),        # a darker wood: the brackets under a wall shelf
    steel=('#B9BDC1', 'metal'),      # a cooker hood, a bin: brushed stainless
    trim=('#3A3D41', 'satin'),       # dark trim, keys and a hood's filter
    bulb=('#FFF1CF', 'gloss'),       # a strip light or a hood lamp
    copper=('#B87333', 'metal'),     # copper pans on the pot rail
    glass=('#DCE6EA', 'glass'),      # glazed doors, jars and glasses
    back=('#C9D6DA', 'satin'),       # a cabinet's back, seen through its glass
    leaf=('#5E8F4E', 'satin'),       # a pot plant's leaves (plain: foliage is a draw call of its own)
    bin1=('#3E7CC2', 'satin'),       # recycling bins: blue, green and yellow
    bin2=('#4E9A5A', 'satin'),
    bin3=('#E3B23C', 'satin'),
    lid=('#2F3236', 'satin'),        # a bin's lid
    walnut=('#7A5234', 'satin'),     # a bar's body and slats
    island=('#9DAF8C', 'satin'),     # the kitchen island's sage stool seats
)

SECONDS = 1.4   # a drawer slides in this long
HOLD = 1.5      # seconds it stays open once a use starts
DOOR_SECONDS = 1.6   # a cupboard door swings in this long
DOOR_HOLD = 2.5      # seconds a pantry's doors stay open once a use starts
WALL_DOOR_SECONDS = 0.8   # a wall cabinet's door swings open in this long, while clean things are put away
# catalog2's 0.79-deep counter numbers (worktop -0.48..0.31, carcass -0.46..0.24) as drawn: 0.66 deep, the
# worktop -0.43..0.23 and the carcass -0.41..0.19 (catalog2.py COUNTER66; gamekit GameFrame `depths`)
COUNTER66 = ((-0.48, -0.43), (-0.46, -0.41), (0.24, 0.19), (0.31, 0.23))


def _handle_v(F, name, x, y, z, material=None):
    """A vertical bar handle on a door."""
    F.box(name, 0.014, 0.14, 0.022, (x, y, z), material or M.chrome, bevel=0.005)


def _handle_h(F, name, x, y, z, w=0.18, material=None):
    """A horizontal bar handle on a drawer."""
    F.box(name, w, 0.014, 0.022, (x, y, z), material or M.chrome, bevel=0.005)


def _unit66(F, w=1.0, body=None, top=None):
    """A base unit as the catalogue's `k_base`: worktop to 0.95, front face at depth 0.14."""
    body, top = body or M.cab, top or M.worktop
    F.box('carcass', w - 0.004, 0.82, 0.6, (0, 0.51, -0.16), body, bevel=0.01)
    F.box('plinth', w - 0.04, 0.10, 0.56, (0, 0.05, -0.18), M.dark, bevel=0.004)
    F.box('worktop', w, 0.04, 0.66, (0, 0.93, -0.15), top, bevel=0.006)


def _unit79(F, w=1.0, x=0.0, body=None, top=None):
    """A base unit as catalog2's `counter_body`: worktop to 0.95, front face at depth 0.24."""
    body, top = body or M.cab, top or M.worktop
    F.box('carcass', w - 0.004, 0.8, 0.7, (x, 0.5, -0.11), body, bevel=0.01)
    F.box('plinth', w - 0.004, 0.10, 0.64, (x, 0.05, -0.14), M.dark, bevel=0.004)
    F.box('worktop', w, 0.05, 0.79, (x, 0.925, -0.085), top, bevel=0.008)


def _doors(F, w, front, y0, y1, body=None, hm=None, hx=0.06, hy=None):
    """Two door fronts on the front face `front`, their handles near the inner top corners."""
    body, hm = body or M.cab, hm or M.chrome
    for s in (-1, 1):
        F.box(f'door{s}', w / 2 - 0.03, y1 - y0, 0.02, (s * w / 4, (y0 + y1) / 2, front), body, bevel=0.008)
        _handle_v(F, f'door_pull{s}', s * hx, hy if hy is not None else y1 - 0.10, front + 0.018, hm)


def _drawer(F, name, w, y, z, material, hm=None, handle_w=0.18):
    """A drawer front with a bar handle and the box behind it (hidden inside the carcass at rest),
    as its own object, for `drawer_open`."""
    with G.new_parts() as parts:
        F.box(f'{name}_front', w, 0.17, 0.02, (0, y, z), material, bevel=0.008)
        _handle_h(F, f'{name}_pull', 0, y, z + 0.018, handle_w, hm)
        F.box(f'{name}_bottom', w - 0.06, 0.02, 0.5, (0, y - 0.07, z - 0.26), material, bevel=0.003)
        for s in (-1, 1):
            F.box(f'{name}_side{s}', 0.02, 0.14, 0.5, (s * (w / 2 - 0.04), y - 0.01, z - 0.26), material, bevel=0.003)
        F.box(f'{name}_back', w - 0.06, 0.14, 0.02, (0, y - 0.01, z - 0.5), material, bevel=0.003)
    return G.join(parts, name, origin=F.at(0, y, z))


# --- the plain counter: 1x1, a drawer over two doors, the drawer sliding out for a sandwich -----------
def counter():
    F = G.GameFrame(shift=0.15)
    with G.new_parts() as still:
        _unit66(F, w=1.0)
        _doors(F, w=1.0, front=0.15, y0=0.12, y1=0.72, hy=0.62)
    G.join(still, 'body', origin=(0, 0, 0))
    drawer = _drawer(F, 'drawer', w=0.974, y=0.815, z=0.15, material=M.cab)
    G.settle()
    G.slide(drawer, 'drawer_open', (0, -0.25, 0), seconds=SECONDS)


# --- the corner counter: 1x1, a deep L unit with a drawer, two doors and a utensil pot ----------------
def counterCorner():
    F = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        F.box('carcass', 1.0, 0.82, 0.89, (0, 0.51, 0.015), M.cab, bevel=0.012)       # its back on the wall face
        F.box('plinth', 0.96, 0.10, 0.85, (0, 0.05, -0.005), M.dark, bevel=0.006)
        F.box('worktop', 1.0, 0.04, 0.93, (0, 0.93, 0.035), M.worktop, bevel=0.006)
        _doors(F, w=1.0, front=0.47, y0=0.12, y1=0.72, hy=0.62)
        # a utensil pot and two wooden spoons on the worktop
        F.cone('pot', 0.07, 0.0595, 0.18, (-0.3, 1.04, -0.3), M.worktop, verts=16)
        F.cyl('spoon1', 0.010, 0.20, (-0.32, 1.16, -0.30), M.spoon, verts=8)
        F.cyl('spoon2', 0.010, 0.20, (-0.28, 1.15, -0.31), M.spoon, verts=8)
    G.join(still, 'body', origin=(0, 0, 0))
    drawer = _drawer(F, 'drawer', w=0.974, y=0.815, z=0.47, material=M.cab)
    G.settle()
    G.slide(drawer, 'drawer_open', (0, -0.25, 0), seconds=SECONDS)


# --- the counter end: 1x1, a unit with one door and an open end unit of two shelves -------------------
def counterEnd():
    F = G.GameFrame(shift=0.085, depths=COUNTER66)
    with G.new_parts() as still:
        _unit79(F, w=0.84, x=-0.08)
        F.box('door', 0.808, 0.74, 0.02, (-0.08, 0.49, 0.25), M.cab, bevel=0.008)
        _handle_v(F, 'door_pull', -0.44, 0.74, 0.27)
        # the open end unit: a back, two sides, a bottom, a top and two shelves
        F.box('end_back', 0.16, 0.8, 0.03, (0.42, 0.5, -0.43), M.oak, bevel=0.004)
        for s in (-1, 1):
            F.box(f'end_side{s}', 0.012, 0.8, 0.7, (0.42 + s * 0.074, 0.5, -0.11), M.oak, bevel=0.004)
        F.box('end_top', 0.16, 0.05, 0.79, (0.42, 0.925, -0.085), M.worktop, bevel=0.008)
        F.box('end_bottom', 0.16, 0.05, 0.7, (0.42, 0.125, -0.11), M.oak, bevel=0.004)
        for i, y in enumerate((0.38, 0.66)):
            F.box(f'end_shelf{i}', 0.136, 0.025, 0.66, (0.42, y, -0.11), M.oak, bevel=0.004)
        # a pot, a jar and a tin on the shelves
        F.cone('pot', 0.055, 0.048, 0.11, (0.42, 0.44, 0.05), M.porcelain, verts=12)
        F.cyl('jar', 0.05, 0.10, (0.42, 0.72, -0.15), M.cream, verts=12)
        F.cyl('tin', 0.055, 0.08, (0.42, 0.18, -0.05), M.rust, verts=12)
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle()


# --- the outer corner counter: 1x1, a short unit with a side panel, a plate and a lemon ---------------
def counterCornerOuter():
    F = G.GameFrame(shift=0.085, depths=COUNTER66)
    with G.new_parts() as still:
        F.box('carcass', 0.74, 0.8, 0.7, (-0.13, 0.5, -0.11), M.cab, bevel=0.01)
        F.box('plinth', 0.70, 0.10, 0.64, (-0.15, 0.05, -0.14), M.dark, bevel=0.004)
        F.box('worktop', 0.81, 0.05, 0.79, (-0.095, 0.925, -0.085), M.worktop, bevel=0.008)
        F.box('door', 0.708, 0.74, 0.02, (-0.13, 0.49, 0.25), M.cab, bevel=0.008)
        _handle_v(F, 'door_pull', -0.44, 0.74, 0.27)
        # the end panel that closes the run, with a pull on its front edge
        F.box('panel', 0.02, 0.74, 0.66, (0.25, 0.49, -0.11), M.cab, bevel=0.006)
        _handle_v(F, 'panel_pull', 0.27, 0.74, 0.12)
        # a plate with a lemon on the worktop
        F.cyl('plate', 0.12, 0.02, (0.12, 0.96, 0.08), M.porcelain, verts=20)
        F.sphere('lemon', 0.04, (0.1, 1.0, 0.07), M.yellow, scale=(1, 1, 0.875), segments=10, rings=8)
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle()


# --- the drawer counter: 1x1, four deep drawers under the worktop (static) ----------------------------
def counterDrawers():
    F = G.GameFrame(shift=0.085, depths=COUNTER66)
    with G.new_parts() as still:
        _unit79(F, w=1.0)
        for i, y in enumerate((0.2125, 0.3975, 0.5825)):
            F.box(f'drawer{i}', 0.97, 0.173, 0.02, (0, y, 0.25), M.cab, bevel=0.008)
            _handle_h(F, f'drawer_pull{i}', 0, y + 0.046, 0.27, 0.30)
    G.join(still, 'body', origin=(0, 0, 0))
    # the top drawer with its box behind, sliding out while a resident puts clean dishes away
    y = 0.7675
    with G.new_parts() as parts:
        F.box('drawer_front', 0.97, 0.173, 0.02, (0, y, 0.25), M.cab, bevel=0.008)
        _handle_h(F, 'drawer_pull', 0, y + 0.046, 0.27, 0.30)
        F.box('drawer_bottom', 0.91, 0.02, 0.5, (0, y - 0.07, -0.01), M.cab, bevel=0.003)
        for s in (-1, 1):
            F.box(f'drawer_side{s}', 0.02, 0.14, 0.5, (s * 0.445, y - 0.01, -0.01), M.cab, bevel=0.003)
        F.box('drawer_back', 0.91, 0.14, 0.02, (0, y - 0.01, -0.25), M.cab, bevel=0.003)
    drawer = G.join(parts, 'drawer', origin=F.at(0, y, 0.25))
    G.settle()
    G.slide(drawer, 'drawer_open', (0, -0.25, 0), seconds=SECONDS)


# --- the open-shelf counter: 1x1, an open carcass with two shelves of plates and pots (static) --------
def counterOpenShelf():
    F = G.GameFrame(shift=0.085, depths=COUNTER66)
    with G.new_parts() as still:
        F.box('plinth', 0.996, 0.10, 0.64, (0, 0.05, -0.14), M.dark, bevel=0.004)
        F.box('worktop', 1.0, 0.05, 0.79, (0, 0.925, -0.085), M.worktop, bevel=0.008)
        # an open carcass: a back, two sides, a top and a bottom, with two shelves between
        # (dark inside, the cabinet's colour outside, for one standing free)
        F.box('back', 0.996, 0.8, 0.02, (0, 0.5, -0.42), M.soil, bevel=0.004)
        F.box('back_skin', 0.996, 0.8, 0.02, (0, 0.5, -0.44), M.cab, bevel=0.004)
        for s in (-1, 1):
            F.box(f'side{s}', 0.03, 0.8, 0.66, (s * 0.483, 0.5, -0.11), M.cab, bevel=0.004)
        F.box('top', 0.996, 0.03, 0.7, (0, 0.885, -0.11), M.cab, bevel=0.004)
        F.box('bottom', 0.996, 0.05, 0.7, (0, 0.125, -0.11), M.cab, bevel=0.004)
        for i, y in enumerate((0.16, 0.5)):
            F.box(f'shelf{i}', 0.93, 0.025, 0.66, (0, y, -0.11), M.oak, bevel=0.004)
        # plates and pots on the shelves
        for i, (x, y) in enumerate(((-0.32, 0.2), (0.14, 0.2))):
            F.cyl(f'plate{i}', 0.08, 0.025, (x, y, -0.1), M.porcelain, verts=16)
        for i, (x, y) in enumerate(((-0.10, 0.23), (0.34, 0.23))):
            F.cyl(f'bowl{i}', 0.065, 0.085, (x, y, -0.1), M.cream, verts=16)
        F.cone('pot0', 0.07, 0.063, 0.15, (-0.3, 0.60, -0.1), M.sage, verts=16)
        F.cyl('pot1', 0.06, 0.20, (-0.1, 0.62, -0.1), M.teal, verts=16)
        for i, y in enumerate((0.54, 0.59)):
            F.cyl(f'stack{i}', 0.1, 0.05, (0.25, y, -0.1), M.porcelain, verts=16)
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle()


# --- the butcher block: 1x1, a block on legs with a shelf, a basket and a knife drawer ----------------
def butcherBlock():
    F = G.GameFrame(shift=-0.0055)
    with G.new_parts() as still:
        F.box('top', 0.9, 0.10, 0.8, (0, 0.88, 0), M.block, bevel=0.012)
        F.box('top_layer', 0.84, 0.035, 0.74, (0, 0.9325, 0), M.block2, bevel=0.004)
        for sx in (-1, 1):
            for sz in (-1, 1):
                F.box(f'leg{sx}{sz}', 0.07, 0.83, 0.07, (sx * 0.41, 0.415, sz * 0.36), M.leg, bevel=0.01)
        for i, z in enumerate((-0.24, -0.08, 0.08, 0.24)):
            F.box(f'slat{i}', 0.8, 0.025, 0.12, (0, 0.2, z), M.leg, bevel=0.004)
        # a basket and a box on the shelf
        F.box('basket', 0.4, 0.2, 0.3, (-0.16, 0.32, 0), M.basket, bevel=0.03)
        F.box('box', 0.2, 0.16, 0.2, (0.24, 0.29, 0.06), M.porcelain, bevel=0.03)
    G.join(still, 'body', origin=(0, 0, 0))
    # the knife drawer under the block, sliding out for a sandwich
    drawer = _drawer(F, 'drawer', 0.7, 0.72, 0.39, M.wood2, M.metal, handle_w=0.6)   # with its box behind
    G.settle()
    G.slide(drawer, 'drawer_open', (0, -0.25, 0), seconds=SECONDS)


# --- the marble counter: 1x1, a navy unit with a marble top and brass handles, a drawer for sandwich --
def counterMarble():
    F = G.GameFrame(shift=0.085, depths=COUNTER66)
    with G.new_parts() as still:
        _unit79(F, w=1.0, body=M.navy, top=M.marble)
        _doors(F, w=1.0, front=0.25, y0=0.12, y1=0.72, body=M.navy, hm=M.brass, hx=0.05, hy=0.62)
        # two veins through the marble
        F.box('vein0', 0.6, 0.004, 0.03, (-0.1, 0.951, -0.05), M.vein, rot=(0, 0, 28), bevel=0.001)
        F.box('vein1', 0.35, 0.004, 0.02, (0.22, 0.951, 0.08), M.vein, rot=(0, 0, -15), bevel=0.001)
    G.join(still, 'body', origin=(0, 0, 0))
    drawer = _drawer(F, 'drawer', w=0.97, y=0.815, z=0.25, material=M.navy, hm=M.brass)
    G.settle()
    G.slide(drawer, 'drawer_open', (0, -0.25, 0), seconds=SECONDS)


# === K5: tall and wall storage =========================================================================
# Tall larders (`pantry`, `pantryTall`), wall cabinets (`wallCabinet`, `wallCabinetGlass`,
# `wallCabinetCorner`), a cooker hood (`rangeHood`), a spice rack, a pot rail, a wall shelf
# (`kitchenShelfWall`), a pedal bin (`trashCan`) and recycling bins. The two larders' doors swing open
# (`doors_open`) for a snack, the wall cabinets' while clean things are put away in them, the pedal bin's lid
# while a plate is scraped and the recycling bins' lids while the recycling is sorted; the hood, rack, rail
# and shelf are still. A wall thing sits on the wall face (z = WALL)
# at its real height: `floor=False`, centred in x and depth.

WALL = -0.43   # the back face of a wall thing, in game depth (catalog.py WALL, docs/design/objects.md)


def _case(F, w, h, d, x, y, zback, material, back=None, t=0.02, p='case'):
    """An open case: a back at depth `zback`, two sides, a top and a bottom, open at the front."""
    F.box(f'{p}_back', w, h, t, (x, y, zback + t / 2), back or material, bevel=0.004)
    for s in (-1, 1):
        F.box(f'{p}_side{s}', t, h, d, (x + s * (w / 2 - t / 2), y, zback + d / 2), material, bevel=0.004)
    F.box(f'{p}_top', w - 2 * t, t, d, (x, y + h / 2 - t / 2, zback + d / 2), material, bevel=0.004)
    F.box(f'{p}_bottom', w - 2 * t, t, d, (x, y - h / 2 + t / 2, zback + d / 2), material, bevel=0.004)


def _shelf(F, name, w, y, zback, d, material=None):
    """A shelf across a case, from the back face `zback`."""
    F.box(name, w, 0.02, d, (0, y, zback + d / 2), material or M.worktop, bevel=0.003)


def _jar(F, name, x, y, z, colour, h=0.13, r=0.045, lid=True, lidmat=None, verts=10):
    """A glass jar with its contents, standing on `y` at depth `z`."""
    F.cyl(f'{name}_glass', r, h, (x, y + h / 2, z), M.glass, verts=verts)
    F.cyl(f'{name}_in', r * 0.8, h * 0.6, (x, y + h * 0.44, z), colour, verts=verts)
    if lid:
        F.cyl(f'{name}_lid', r * 1.1, 0.016, (x, y + h + 0.008, z), lidmat or M.chrome, verts=verts)


def _basket(F, name, w, h, d, x, y, z, material=None):
    """A woven basket with a rim, sitting on `y`."""
    material = material or M.basket
    F.box(name, w, h, d, (x, y + h / 2, z), material, bevel=0.03)
    F.box(f'{name}_rim', w + 0.02, 0.02, d + 0.02, (x, y + h, z), material, bevel=0.006)


def _tin(F, name, x, y, z, colour, w=0.16, h=0.16, d=0.14):
    """A tin or box of food on a shelf."""
    F.box(name, w, h, d, (x, y + h / 2, z), colour, bevel=0.02)


# --- the larder: 1x1, a tall cupboard with two doors that swing open for a snack -----------------------
def pantry():
    F = G.GameFrame(shift=0.141)
    with G.new_parts() as still:
        _case(F, 0.92, 1.92, 0.6, 0, 1.06, -0.46, M.cab)
        F.box('plinth', 0.88, 0.1, 0.56, (0, 0.05, -0.18), M.dark, bevel=0.004)
        F.box('cornice', 0.94, 0.04, 0.62, (0, 2.04, -0.15), M.cab, bevel=0.008)
        for i, y in enumerate((0.5, 0.88, 1.26, 1.64)):
            _shelf(F, f'shelf{i}', 0.86, y, -0.44, 0.56)
        for i, (x, y, c) in enumerate(((-0.24, 0.52, M.rust), (0.0, 0.52, M.cream), (0.24, 0.52, M.teal),
                                       (-0.2, 0.9, M.porcelain), (0.16, 0.9, M.yellow),
                                       (-0.22, 1.28, M.sage), (0.1, 1.28, M.porcelain))):
            _tin(F, f'tin{i}', x, y, -0.15, c)
        _basket(F, 'basket', 0.36, 0.2, 0.3, -0.2, 2.06, -0.2)
    G.join(still, 'body', origin=(0, 0, 0))
    doors = []
    for s in (-1, 1):
        with G.new_parts() as parts:
            F.box(f'door{s}_front', 0.456, 1.88, 0.02, (s * 0.228, 1.06, 0.14), M.cab, bevel=0.008)
            _handle_v(F, f'door{s}_pull', s * 0.06, 1.05, 0.158)
        doors.append(G.join(parts, f'door{s}', origin=F.at(s * 0.456, 1.06, 0.14)))
    G.settle()
    G.swing(doors[0], 'doors_open', 100, seconds=DOOR_SECONDS)
    G.swing(doors[1], 'doors_open', -100, seconds=DOOR_SECONDS)


# --- the tall larder: 1x1, two doors over two drawers, a basket on top ---------------------------------
def pantryTall():
    F = G.GameFrame(shift=0.104)
    with G.new_parts() as still:
        _case(F, 0.92, 2.06, 0.66, 0, 1.08, -0.45, M.cab)
        F.box('plinth', 0.9, 0.1, 0.6, (0, 0.05, -0.14), M.dark, bevel=0.004)
        F.box('cornice', 0.96, 0.05, 0.7, (0, 2.135, -0.11), M.cab, bevel=0.01)
        for i, y in enumerate((0.24, 0.48)):
            F.box(f'drawer{i}', 0.9, 0.22, 0.02, (0, y, 0.21), M.cab, bevel=0.008)
            _handle_h(F, f'drawer{i}_pull', 0, y + 0.06, 0.228, 0.3, M.brass)
        for i, y in enumerate((0.74, 1.12, 1.5, 1.88)):
            _shelf(F, f'shelf{i}', 0.86, y, -0.43, 0.62)
        for i, (x, y, c) in enumerate(((-0.24, 0.76, M.porcelain), (0.0, 0.76, M.rust), (0.24, 0.76, M.sage),
                                       (-0.2, 1.14, M.cream), (0.16, 1.14, M.teal),
                                       (-0.22, 1.52, M.yellow), (0.1, 1.52, M.porcelain))):
            _tin(F, f'tin{i}', x, y, -0.14, c)
        _basket(F, 'basket', 0.3, 0.22, 0.26, -0.22, 2.16, -0.16)
    G.join(still, 'body', origin=(0, 0, 0))
    doors = []
    for s in (-1, 1):
        with G.new_parts() as parts:
            F.box(f'door{s}_front', 0.45, 1.44, 0.02, (s * 0.228, 1.34, 0.21), M.cab, bevel=0.008)
            _handle_v(F, f'door{s}_pull', s * 0.06, 0.72, 0.228, M.brass)
        doors.append(G.join(parts, f'door{s}', origin=F.at(s * 0.453, 1.34, 0.21)))
    G.settle()
    G.swing(doors[0], 'doors_open', 100, seconds=DOOR_SECONDS)
    G.swing(doors[1], 'doors_open', -100, seconds=DOOR_SECONDS)


# --- the wall cabinet: 1x1, two doors with pulls under it and a strip light; plates, bowls and mugs
# inside, the doors swinging open while a resident puts the clean things away (`doors_open`)
def wallCabinet():
    F = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        _case(F, 0.98, 0.72, 0.34, 0, 1.81, WALL, M.cab)
        _shelf(F, 'shelf', 0.94, 1.81, WALL + 0.02, 0.3, M.cab)
        for i in range(6):
            F.cyl(f'plate{i}', 0.1, 0.012, (-0.24, 1.476 + i * 0.015, WALL + 0.18), M.porcelain, verts=16)
        for i in range(3):
            F.cone(f'bowl{i}', 0.055, 0.075, 0.05, (0.2, 1.497 + i * 0.022, WALL + 0.18), M.porcelain, verts=14)
        for i, x in enumerate((-0.3, -0.16, 0.12, 0.26)):
            F.cyl(f'mug{i}', 0.038, 0.09, (x, 1.865, WALL + 0.17), M.porcelain, verts=12)
        F.box('light', 0.9, 0.012, 0.03, (0, 1.444, WALL + 0.07), M.bulb, bevel=0.004)
    G.join(still, 'body', origin=(0, 0, 0))
    doors = []
    for s in (-1, 1):
        with G.new_parts() as parts:
            F.box(f'door{s}', 0.484, 0.7, 0.02, (s * 0.245, 1.81, WALL + 0.35), M.cab, bevel=0.008)
            _handle_v(F, f'door{s}_pull', s * 0.06, 1.56, WALL + 0.368)   # on the doors, near their bottom
        doors.append(G.join(parts, f'door{s}', origin=F.at(s * 0.487, 1.81, WALL + 0.34)))
    G.settle(centre=False, floor=False)
    G.swing(doors[0], 'doors_open', 100, seconds=WALL_DOOR_SECONDS)
    G.swing(doors[1], 'doors_open', -100, seconds=WALL_DOOR_SECONDS)


# --- the glazed wall cabinet: 1x1, glass doors showing glasses and plates; the right one opens while
# clean things are put away (`door_open`) ---------------------------------------------------------------
def wallCabinetGlass():
    F = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        _case(F, 0.96, 0.72, 0.36, 0, 1.81, WALL, M.cab, back=M.back)
        for i, y in enumerate((1.47, 1.84)):
            _shelf(F, f'shelf{i}', 0.9, y, WALL, 0.32)
        for i, x in enumerate((-0.36, -0.2, -0.04)):
            F.cyl(f'glass{i}', 0.035, 0.12, (x, 1.54, WALL + 0.17), M.glass, verts=10)
        for i in range(5):
            F.cyl(f'plate{i}', 0.1, 0.012, (0.24, 1.852 + i * 0.016, WALL + 0.17), M.porcelain, verts=16)
    G.join(still, 'body', origin=(0, 0, 0))
    doors = []
    for s in (-1, 1):
        cx = s * 0.24
        with G.new_parts() as parts:
            F.box(f'door{s}_l', 0.045, 0.7, 0.02, (cx - 0.2075, 1.81, WALL + 0.37), M.cab, bevel=0.004)
            F.box(f'door{s}_r', 0.045, 0.7, 0.02, (cx + 0.2075, 1.81, WALL + 0.37), M.cab, bevel=0.004)
            F.box(f'door{s}_b', 0.46, 0.045, 0.02, (cx, 1.4825, WALL + 0.37), M.cab, bevel=0.004)
            F.box(f'door{s}_t', 0.46, 0.045, 0.02, (cx, 2.1375, WALL + 0.37), M.cab, bevel=0.004)
            F.box(f'door{s}_glass', 0.37, 0.61, 0.012, (cx, 1.81, WALL + 0.365), M.glass, bevel=0.002)
            _handle_v(F, f'door{s}_pull', s * 0.04, 1.6, WALL + 0.395)
        doors.append(G.join(parts, f'door{s}', origin=F.at(s * 0.48, 1.81, WALL + 0.36)))
    G.settle(centre=False, floor=False)
    # only the right door, in front of the plates, swings open: each glazed door is two draws (frame, glass)
    G.swing(doors[1], 'door_open', -100, seconds=WALL_DOOR_SECONDS)


# --- the corner wall cabinet: 1x1, a door on one side and open shelves on the other -------------------
def wallCabinetCorner():
    F = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        _case(F, 0.6, 0.72, 0.36, -0.18, 1.81, WALL, M.cab)
        F.box('shelf', 0.56, 0.02, 0.32, (-0.18, 1.81, WALL + 0.18), M.cab, bevel=0.003)
        for i in range(5):
            F.cyl(f'plate{i}', 0.1, 0.012, (-0.3, 1.476 + i * 0.015, WALL + 0.19), M.porcelain, verts=16)
        for i, x in enumerate((-0.36, -0.22, -0.08)):
            F.cyl(f'mug{i}', 0.038, 0.09, (x, 1.865, WALL + 0.18), M.porcelain, verts=12)
        for i, y in enumerate((1.46, 1.81, 2.15)):
            F.box(f'end_shelf{i}', 0.36, 0.025, 0.34, (0.3, y, WALL + 0.17), M.oak, bevel=0.006)
        F.box('end_panel', 0.03, 0.72, 0.36, (0.46, 1.81, WALL + 0.18), M.oak, bevel=0.006)
        F.cone('pot', 0.05, 0.04, 0.14, (0.24, 1.53, WALL + 0.16), M.sage, verts=14)
        F.cyl('bowl', 0.07, 0.05, (0.3, 1.86, WALL + 0.14), M.porcelain, verts=14)
    G.join(still, 'body', origin=(0, 0, 0))
    with G.new_parts() as parts:
        F.box('door', 0.58, 0.7, 0.02, (-0.18, 1.81, WALL + 0.37), M.cab, bevel=0.008)
        _handle_v(F, 'door_pull', 0.06, 1.6, WALL + 0.395)
    door = G.join(parts, 'door', origin=F.at(-0.47, 1.81, WALL + 0.36))
    G.settle(centre=False, floor=False)
    G.swing(door, 'door_open', 100, seconds=WALL_DOOR_SECONDS)


# --- the cooker hood: 1x1, a canopy and a chimney with a lamp and a filter ------------------------------
def rangeHood():
    F = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        F.box('flue', 0.3, 0.58, 0.3, (0, 1.96, -0.28), M.steel, bevel=0.008)
        F.box('canopy', 0.9, 0.12, 0.5, (0, 1.61, -0.18), M.steel, bevel=0.01)
        F.box('trim', 0.9, 0.025, 0.02, (0, 1.58, 0.075), M.trim, bevel=0.004)
        F.box('filter', 0.84, 0.012, 0.44, (0, 1.548, -0.18), M.dark, bevel=0.002)
        F.box('light', 0.5, 0.008, 0.04, (0, 1.542, -0.06), M.bulb, bevel=0.002)
        F.box('keys', 0.16, 0.03, 0.01, (0.28, 1.63, 0.072), M.dark, bevel=0.002)
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle(centre=False, floor=False)


# --- the spice rack: 1x1 wall, three rails of little jars ---------------------------------------------
def spiceRack():
    F = G.GameFrame(shift=0.0)
    colours = (M.rust, M.teal, M.sage, M.cream, M.yellow, M.porcelain)
    with G.new_parts() as still:
        F.box('board', 0.52, 0.44, 0.02, (0, 1.52, WALL + 0.01), M.oak, bevel=0.004)
        for i, y in enumerate((1.33, 1.47, 1.61)):
            F.box(f'shelf{i}', 0.5, 0.015, 0.09, (0, y, WALL + 0.055), M.oak, bevel=0.003)
            F.box(f'rail{i}', 0.5, 0.012, 0.012, (0, y + 0.06, WALL + 0.1), M.brass, bevel=0.003)
            for j in range(6):
                _jar(F, f'jar{i}{j}', -0.2 + j * 0.08, y + 0.0075, WALL + 0.055,
                     colours[(i + j) % 6], h=0.075, r=0.024, lidmat=M.dark, verts=8)
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle(centre=False, floor=False)


# --- the pot rail: 1x1 wall, a rail of hooks with pans, a pot and ladles -------------------------------
def potRail():
    F = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        F.cyl('rail', 0.02, 0.9, (0, 1.6, WALL + 0.08), M.metal, rot=(0, 90, 0), verts=12)
        for s in (-1, 1):
            F.box(f'bracket{s}', 0.03, 0.03, 0.08, (s * 0.44, 1.6, WALL + 0.04), M.metal, bevel=0.006)   # from the wall face
        for x, kind in ((-0.32, 'pan'), (-0.12, 'ladle'), (0.06, 'pot'), (0.28, 'pan'), (0.4, 'ladle')):
            G.torus(f'hook{x}', 0.035, 0.007, at=F.at(x, 1.585, WALL + 0.08), material=M.metal,
                    rot=(0, 90, 0), major_segments=8, minor_segments=6)
            if kind == 'pan':
                F.cyl(f'handle{x}', 0.012, 0.22, (x, 1.43, WALL + 0.08), M.metal, verts=8)
                F.cyl(f'pan{x}', 0.13, 0.04, (x, 1.21, WALL + 0.06), M.copper, rot=(90, 0, 0), verts=18)
            elif kind == 'pot':
                F.cyl(f'handle{x}', 0.012, 0.1, (x, 1.5, WALL + 0.08), M.metal, verts=8)
                F.cyl(f'pot{x}', 0.1, 0.16, (x, 1.37, WALL + 0.12), M.copper, verts=16)
            else:
                F.cyl(f'handle{x}', 0.012, 0.3, (x, 1.42, WALL + 0.08), M.steel, verts=8)
                F.sphere(f'bowl{x}', 0.05, (x, 1.27, WALL + 0.09), M.steel, scale=(1, 1, 0.7), segments=8, rings=6)
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle(centre=False, floor=False)


# --- the wall shelf: 1x1 wall, two shelves of jars, plates, pots and a plant ---------------------------
def kitchenShelfWall():
    F = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        for i, y in enumerate((1.4, 1.75)):
            F.box(f'shelf{i}', 0.9, 0.03, 0.24, (0, y, WALL + 0.12), M.oak, bevel=0.004)
        for s in (-1, 1):
            F.box(f'bracket{s}', 0.03, 0.12, 0.2, (s * 0.36, 1.35, WALL + 0.1), M.oakd, bevel=0.004)
        _jar(F, 'jar0', -0.32, 1.415, WALL + 0.12, M.yellow, h=0.13, r=0.045)
        _jar(F, 'jar1', -0.2, 1.415, WALL + 0.12, M.porcelain, h=0.16, r=0.045)
        _jar(F, 'jar2', -0.08, 1.415, WALL + 0.12, M.rust, h=0.13, r=0.045)
        for i in range(4):
            F.cyl(f'plate{i}', 0.11, 0.012, (0.22, 1.425 + i * 0.014, WALL + 0.12), M.porcelain, verts=16)
        F.cyl('pot0', 0.05, 0.1, (-0.25, 1.815, WALL + 0.12), M.teal, verts=14)
        F.cyl('pot1', 0.05, 0.1, (-0.12, 1.815, WALL + 0.12), M.cream, verts=14)
        F.cone('plant_pot', 0.06, 0.05, 0.1, (0.25, 1.815, WALL + 0.12), M.porcelain, verts=14)
        F.cyl('soil', 0.052, 0.012, (0.25, 1.868, WALL + 0.12), M.soil, verts=12)
        F.sphere('leaf0', 0.09, (0.25, 1.97, WALL + 0.12), M.leaf, scale=(1, 1, 0.85), segments=10, rings=7)
        F.sphere('leaf1', 0.055, (0.19, 1.95, WALL + 0.14), M.leaf, segments=8, rings=6)
        F.sphere('leaf2', 0.05, (0.31, 1.94, WALL + 0.1), M.leaf, segments=8, rings=6)
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle(centre=False, floor=False)


# --- the pedal bin: 1x1, a stainless drum with a domed lid and a pedal (static) -----------------------
def trashCan():
    F = G.GameFrame(shift=0.172)
    with G.new_parts() as still:
        F.cyl('drum', 0.15, 0.6, (0, 0.32, -0.2), M.steel, verts=20)
        F.cyl('base', 0.15, 0.02, (0, 0.01, -0.2), M.dark, verts=20)
        F.box('pedal', 0.1, 0.03, 0.08, (0, 0.03, -0.03), M.dark, bevel=0.01)
    G.join(still, 'body', origin=(0, 0, 0))
    # the lid, hinged at its back edge, lifting while a resident scrapes a plate into the bin
    with G.new_parts() as parts:
        F.sphere('lid', 0.155, (0, 0.62, -0.2), M.lid, scale=(1, 1, 0.42), segments=20, rings=8)
    lid = G.hinged(G.join(parts, 'lid'), F.at(0, 0.62, -0.355))
    G.settle()
    G.swing(lid, 'lid_open', -75, seconds=0.5, axis='X')


# --- the recycling bins: 1x1, three coloured bins with lids, each lid hinged at its back; they lift one
# after another while a resident sorts the recycling (`lid0_open` .. `lid2_open`)
BINS = ((-0.31, 'bin1'), (0.0, 'bin2'), (0.31, 'bin3'))


def recyclingBins():
    F = G.GameFrame(shift=0.212)
    with G.new_parts() as still:
        for i, (x, c) in enumerate(BINS):
            F.box(f'body{i}', 0.29, 0.62, 0.42, (x, 0.31, -0.22), getattr(M, c), bevel=0.03)
            F.box(f'handle{i}', 0.14, 0.03, 0.03, (x, 0.6, 0.0), M.lid, bevel=0.01)
    G.join(still, 'body', origin=(0, 0, 0))
    lids = []
    for i, (x, c) in enumerate(BINS):
        with G.new_parts() as parts:
            F.box(f'lid{i}', 0.3, 0.04, 0.44, (x, 0.64, -0.22), getattr(M, c), bevel=0.015)
        lids.append(G.hinged(G.join(parts, f'lid{i}'), F.at(x, 0.62, -0.44)))
    G.settle()
    for i, lid in enumerate(lids):
        G.swing(lid, f'lid{i}_open', -80, seconds=0.5, axis='X')


# --- K6: islands and bars ---------------------------------------------------------------------------
# A freestanding island: cupboards at the front (+Z), where residents stand to use it, as the kitchen island;
# the worktop overhangs the back (-Z) 30 cm for stools, its edge on the wall face.
def _island(F, w, body=None, top=None, handles=None):
    """A freestanding island unit as catalog2's `island_unit`: worktop to 0.95, cupboards at the front."""
    body, top, handles = body or M.cab, top or M.worktop, handles or M.chrome
    F.box('carcass', w - 0.16, 0.82, 0.58, (0, 0.51, 0.16), body, bevel=0.012)
    F.box('plinth', w - 0.2, 0.10, 0.52, (0, 0.05, 0.16), M.dark, bevel=0.004)
    F.box('worktop', w - 0.04, 0.05, 0.90, (0, 0.925, 0.02), top, bevel=0.012)
    n = max(1, round((w - 0.16) / 0.46))
    dw = (w - 0.18) / n
    for i in range(n):
        x = -(w - 0.18) / 2 + dw * (i + 0.5)
        F.box(f'door{i}', dw - 0.02, 0.72, 0.02, (x, 0.5, 0.455), body, bevel=0.008)
        _handle_h(F, f'door{i}_pull', x, 0.78, 0.47, 0.14, handles)
    for s in (-1, 1):
        F.box(f'end{s}', 0.04, 0.92, 0.6, (s * (w - 0.16) / 2, 0.46, 0.15), body, bevel=0.008)

# --- the kitchen island: 2x1, two stools at the front, a drawer for a sandwich -----------------------
def kitchenIsland():
    F = G.GameFrame(shift=-0.0495)
    with G.new_parts() as still:
        # the cupboards face the front (+Z), where residents stand to use the island; the worktop
        # overhangs the back (-Z), where the stools are set (housegen sets them at a used table's back)
        F.box('carcass', 1.84, 0.86, 0.56, (0, 0.47, 0.16), M.cab, bevel=0.014)
        F.box('plinth', 1.82, 0.08, 0.54, (0, 0.04, 0.16), M.dark, bevel=0.004)
        F.box('worktop', 1.94, 0.05, 0.82, (0, 0.925, 0.04), M.worktop, bevel=0.01)
        for s in (-1, 1):                                       # two doors under the drawer
            F.box(f'door{s}', 0.86, 0.6, 0.02, (s * 0.47, 0.35, 0.44), M.cab, bevel=0.008)
            _handle_h(F, f'door{s}_pull', s * 0.47, 0.6, 0.458, 0.14)
        # no stools of its own, and a bare top: stools and the things on its top (a fruit bowl, mugs)
        # are things of their own (housegen sets them), so nothing is doubled or stands in a stool
    G.join(still, 'body', origin=(0, 0, 0))
    # the drawer over the doors, at the front where residents stand
    drawer = _drawer(F, 'drawer', 0.9, 0.77, 0.44, M.cab, handle_w=0.3)
    G.settle()
    # sandwich slides the drawer out to the front
    G.slide(drawer, 'drawer_open', (0, -0.25, 0), seconds=SECONDS)

# --- the small island: 1x1, cupboards at the front, a bare top (static) -----------------------------
def islandSmall():
    F = G.GameFrame()
    with G.new_parts() as still:
        _island(F, 1.0, handles=M.brass)       # a bare top: housegen sets things on it
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle()

# --- the long island: 3x1, six cupboards at the front, a bare top (static) ---------------------------
def islandLong():
    F = G.GameFrame()
    with G.new_parts() as still:
        _island(F, 3.0, handles=M.metal)       # a bare top: housegen sets things on it
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle()

# --- a bar: a slatted body, a top to 1.06 overhanging the front (+Z) and a foot rail (static) ---------
def _bar(F, bw, bx, end=False, body=None, top=None, rail=None):
    """A bar as catalog2's `bar`: a slatted body, a top at 1.06 overhanging the front (+Z), a foot rail."""
    body, top, rail = body or M.walnut, top or M.oak, rail or M.brass
    F.box('body', bw, 0.98, 0.5, (bx, 0.49, -0.12), body, bevel=0.01)
    F.box('top', 1.0, 0.06, 0.88, (0, 1.03, -0.04), top, bevel=0.012)
    F.box('ledge', bw, 0.04, 0.26, (bx, 0.92, -0.36), top, bevel=0.006)
    n = int(bw / 0.1)
    for i in range(n):
        F.box(f'slat{i}', bw / n - 0.012, 0.92, 0.02, (-0.5 + bw / n * (i + 0.5), 0.47, 0.135), body, bevel=0.004)
    F.cyl('rail', 0.02, bw, (bx, 0.2, 0.26), rail, rot=(0, 90, 0), verts=10)
    for x in (-0.38, -0.5 + bw - 0.12):
        F.box(f'bracket{x}', 0.03, 0.06, 0.12, (x, 0.2, 0.2), rail, bevel=0.006)
    if end:
        F.box('waterfall', 0.06, 1.0, 0.88, (0.47, 0.5, -0.04), top, bevel=0.008)

def barSegment():
    F = G.GameFrame(shift=0.40)
    with G.new_parts() as still:
        _bar(F, 1.0, 0.0)
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle()

def barEnd():
    F = G.GameFrame(shift=0.40)
    with G.new_parts() as still:
        _bar(F, 0.92, -0.04, end=True)
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle()


# A floor thing against a wall is centred where its placeholder stands (`centre`): its back on the wall,
# its top under its content `surface` (catalog2.py TOPS).
FAMILY = G.Family(
    'kitchen_counters', M,
    [G.Model('counter', counter, size=(1.0, 0.95, 0.66), key='model.counter', centre=(0, -0.10),
             # sandwich slides the top drawer out
             motions={'uses': {'sandwich': 'drawer_open'}, 'hold': HOLD},
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop'), ('chrome', 'Handles')]),
     G.Model('counterCorner', counterCorner, size=(1.0, 1.26, 0.93), key='model.counterCorner', centre=(0, 0.035),
             # sandwich, as the counter
             motions={'uses': {'sandwich': 'drawer_open'}, 'hold': HOLD},
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop'), ('chrome', 'Handles')]),
     G.Model('counterEnd', counterEnd, size=(1.0, 0.95, 0.66), key='model.counterEnd', centre=(0, -0.10),
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop'), ('chrome', 'Handles')]),
     G.Model('counterCornerOuter', counterCornerOuter, size=(0.81, 1.035, 0.66), key='model.counterCornerOuter', centre=(-0.095, -0.10),
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop'), ('chrome', 'Handles')]),
     G.Model('counterDrawers', counterDrawers, size=(1.0, 0.95, 0.66), key='model.counterDrawers', centre=(0, -0.10),
             # putting clean dishes away here (cooking putAwayAt prep, anim fetch) slides the top drawer out
             motions={'uses': {'fetch': 'drawer_open'}, 'hold': HOLD},
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop'), ('chrome', 'Handles')]),
     G.Model('counterOpenShelf', counterOpenShelf, size=(1.0, 0.95, 0.66), key='model.counterOpenShelf', centre=(0, -0.10),
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop')]),
     G.Model('butcherBlock', butcherBlock, size=(0.9, 0.95, 0.813), key='model.butcherBlock',
             # sandwich slides the knife drawer out
             motions={'uses': {'sandwich': 'drawer_open'}, 'hold': HOLD},
             paint=[('leg', 'Base'), ('block', 'Block'), ('block2', 'Top')]),
     G.Model('counterMarble', counterMarble, size=(1.0, 0.953, 0.66), key='model.counterMarble', centre=(0, -0.10),
             # sandwich slides the top drawer out
             motions={'uses': {'sandwich': 'drawer_open'}, 'hold': HOLD},
             paint=[('navy', 'Cabinet'), ('marble', 'Marble top'), ('brass', 'Handles')]),
     # K5: tall and wall storage
     G.Model('pantry', pantry, size=(0.94, 2.27, 0.63), key='model.pantry', centre=(0, -0.115),
             # snack fetches from the larder: both doors swing open
             motions={'uses': {'snack': 'doors_open'}, 'hold': DOOR_HOLD},
             paint=[('cab', 'Cabinet'), ('chrome', 'Handles')]),
     G.Model('pantryTall', pantryTall, size=(0.96, 2.38, 0.71), key='model.pantryTall', centre=(0, -0.08),
             # snack, as the larder
             motions={'uses': {'snack': 'doors_open'}, 'hold': DOOR_HOLD},
             paint=[('cab', 'Cabinet'), ('brass', 'Handles')]),
     # putting clean dishes away (a `cupboard` station, cooking putAwayAt, anim fetch) opens the doors
     G.Model('wallCabinet', wallCabinet, size=(0.98, 0.732, 0.379), key='model.wallCabinet',
             centre=(0, -0.2405), floor=False, motions={'uses': {'fetch': 'doors_open'}, 'hold': HOLD},
             paint=[('cab', 'Cabinet'), ('chrome', 'Handles')]),
     G.Model('wallCabinetGlass', wallCabinetGlass, size=(0.96, 0.72, 0.406), key='model.wallCabinetGlass',
             centre=(0, -0.227), floor=False, motions={'uses': {'fetch': 'door_open'}, 'hold': HOLD},
             paint=[('cab', 'Cabinet'), ('chrome', 'Handles')]),
     G.Model('wallCabinetCorner', wallCabinetCorner, size=(0.96, 0.722, 0.406), key='model.wallCabinetCorner',
             centre=(0, -0.227), floor=False, motions={'uses': {'fetch': 'door_open'}, 'hold': HOLD},
             paint=[('cab', 'Cabinet'), ('chrome', 'Handles')]),
     G.Model('rangeHood', rangeHood, size=(0.9, 0.712, 0.515), key='model.rangeHood',
             centre=(0, -0.1725), floor=False,
             paint=[('steel', 'Hood'), ('trim', 'Trim')]),
     G.Model('spiceRack', spiceRack, size=(0.52, 0.44, 0.106), key='model.spiceRack',
             centre=(0, -0.377), floor=False,
             paint=[('oak', 'Rack')]),
     G.Model('potRail', potRail, size=(0.91, 0.545, 0.22), key='model.potRail',
             centre=(0, -0.32), floor=False,
             paint=[('metal', 'Rail'), ('copper', 'Pots')]),
     G.Model('kitchenShelfWall', kitchenShelfWall, size=(0.9, 0.76, 0.24), key='model.kitchenShelfWall',
             centre=(0, -0.31), floor=False,
             paint=[('oak', 'Shelves'), ('oakd', 'Brackets')]),
     G.Model('trashCan', trashCan, size=(0.31, 0.685, 0.36), key='model.trashCan', centre=(0, -0.247),
             # scraping a plate into the bin (the cooking `bin` station, anim clean) lifts the lid
             motions={'uses': {'clean': 'lid_open'}, 'hold': 1.0},
             paint=[('steel', 'Bin'), ('lid', 'Lid')]),
     G.Model('recyclingBins', recyclingBins, size=(0.92, 0.66, 0.455), key='model.recyclingBins', centre=(0, -0.212),
             # sorting the recycling lifts the three lids one after another
             motions={'uses': {'sortRecycling': [{'clip': f'lid{i}_open', 'delay': 0.6 * i} for i in range(3)]},
                      'hold': 0.9},
             paint=[('bin3', 'Left bin'), ('bin2', 'Middle bin'), ('bin1', 'Right bin'), ('lid', 'Handles')]),   # as seen from the front
     # K6: islands and bars
     G.Model('kitchenIsland', kitchenIsland, size=(1.94, 0.95, 0.839), key='model.kitchenIsland', centre=(0, 0.0495),
             # sandwich slides the drawer out (breakfast is eaten standing, or on a stool at the island)
             motions={'uses': {'sandwich': 'drawer_open'}, 'hold': HOLD},
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop')]),
     G.Model('islandSmall', islandSmall, size=(0.96, 0.95, 0.911), key='model.islandSmall', centre=(0, 0.0255),
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop')]),
     G.Model('islandLong', islandLong, size=(2.96, 0.95, 0.911), key='model.islandLong', centre=(0, 0.0255),
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop')]),
     G.Model('barSegment', barSegment, size=(1.0, 1.06, 0.89), key='model.barSegment', centre=(0, 0.015),
             paint=[('walnut', 'Cabinet'), ('oak', 'Worktop')]),
     G.Model('barEnd', barEnd, size=(1.0, 1.06, 0.89), key='model.barEnd', centre=(0, 0.015),
             paint=[('walnut', 'Cabinet'), ('oak', 'Worktop')])],
    comment='Blender-built kitchen base counters (tools/art/blender/families/kitchen_counters.py). '
            'Replaces the primitive-part placeholders by key; listed in the base manifest\'s packs. The '
            'placeholder is kept as a fallback if a file fails to load. Each counter is a plinth, a '
            'carcass and a worktop (to 0.95): the plain, corner and marble counters carry a '
            'drawer over two doors and slide it out 25 cm (`drawer_open`) while a resident makes a '
            'sandwich, as does the butcher block\'s knife drawer. The tall and wall storage (K5) adds a '
            'larder whose doors swing open for a snack; the islands and bars (K6) add freestanding '
            'islands with their cupboards at the front and a worktop overhanging the back for stools, and slatted bars with a top at '
            '1.06, the kitchen island sliding a drawer out between its stools for a sandwich.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

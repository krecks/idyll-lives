# The kitchen's base counters (K4): the plain counter and its corner, the counter end with open
# shelves, the outer corner counter, the drawer counter, the open-shelf counter, the butcher block and
# the marble counter. Each replaces the primitive-part placeholder of `catalog.py` / `catalog2.py`,
# which stays the fallback if the file fails to load. Notes: kitchen_counters.md.
#
# A base counter is a plinth, a carcass and a worktop; the front carries a drawer over two doors (the
# plain, corner and marble counters), four drawers, one door, or nothing (the open shelves). The four
# with a `sandwich` use slide their top drawer out 25 cm. The parts are placed in game axes (`F`): the
# placeholder's depths are shifted back by the placeholder's depth centre, so the model is centred on
# its tile (see GameFrame). Two worktop heights are in play: 0.96 (the catalogue counters, `_unit66`)
# and 0.95 (the catalog2 ones, `_unit79`), matching the content's `surface`.
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
    basket=('#C9A979', 'satin'),     # a basket
    metal=('#2B2C2E', 'metal'),      # a dark metal handle
    porcelain=('#F5F3EE', 'gloss'),  # plates, a bowl, a jar
    cream=('#EFE5D3', 'gloss'),      # a cream jar
    rust=('#B5653F', 'gloss'),       # a terracotta pot
    teal=('#4E7A7F', 'gloss'),       # a teal pot
    sage=('#B9C9C4', 'gloss'),       # a sage pot
    yellow=('#F2C14E', 'satin'),     # a lemon
    soil=('#3B2E24', 'matte'),       # the dark back of an open shelf
)

SECONDS = 1.4   # a drawer slides in this long
HOLD = 1.5      # seconds it stays open once a use starts


def _handle_v(F, name, x, y, z, material=None):
    """A vertical bar handle on a door."""
    F.box(name, 0.014, 0.14, 0.022, (x, y, z), material or M.chrome, bevel=0.005)


def _handle_h(F, name, x, y, z, w=0.18, material=None):
    """A horizontal bar handle on a drawer."""
    F.box(name, w, 0.014, 0.022, (x, y, z), material or M.chrome, bevel=0.005)


def _unit66(F, w=1.0, body=None, top=None):
    """A base unit as the catalogue's `k_base`: worktop to 0.96, front face at depth 0.14."""
    body, top = body or M.cab, top or M.worktop
    F.box('carcass', w - 0.004, 0.82, 0.6, (0, 0.51, -0.16), body, bevel=0.01)
    F.box('plinth', w - 0.04, 0.10, 0.56, (0, 0.05, -0.18), M.dark, bevel=0.004)
    F.box('worktop', w, 0.04, 0.66, (0, 0.94, -0.15), top, bevel=0.006)


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
        F.box('carcass', 1.0, 0.82, 0.96, (0, 0.51, -0.02), M.cab, bevel=0.012)
        F.box('plinth', 0.96, 0.10, 0.92, (0, 0.05, -0.04), M.dark, bevel=0.006)
        F.box('worktop', 1.0, 0.04, 1.0, (0, 0.94, 0.0), M.worktop, bevel=0.006)
        _doors(F, w=1.0, front=0.47, y0=0.12, y1=0.72, hy=0.62)
        # a utensil pot and two wooden spoons on the worktop
        F.cone('pot', 0.07, 0.0595, 0.18, (-0.3, 1.05, -0.3), M.worktop, verts=16)
        F.cyl('spoon1', 0.010, 0.20, (-0.32, 1.17, -0.30), M.oak, verts=8)
        F.cyl('spoon2', 0.010, 0.20, (-0.28, 1.16, -0.31), M.oak, verts=8)
    G.join(still, 'body', origin=(0, 0, 0))
    drawer = _drawer(F, 'drawer', w=0.974, y=0.815, z=0.47, material=M.cab)
    G.settle()
    G.slide(drawer, 'drawer_open', (0, -0.25, 0), seconds=SECONDS)


# --- the counter end: 1x1, a unit with one door and an open end unit of two shelves -------------------
def counterEnd():
    F = G.GameFrame(shift=0.085)
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
    F = G.GameFrame(shift=0.085)
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
    F = G.GameFrame(shift=0.085)
    with G.new_parts() as still:
        _unit79(F, w=1.0)
        for i, y in enumerate((0.2125, 0.3975, 0.5825, 0.7675)):
            F.box(f'drawer{i}', 0.97, 0.173, 0.02, (0, y, 0.25), M.cab, bevel=0.008)
            _handle_h(F, f'drawer_pull{i}', 0, y + 0.046, 0.27, 0.30)
    G.join(still, 'body', origin=(0, 0, 0))
    G.settle()


# --- the open-shelf counter: 1x1, an open carcass with two shelves of plates and pots (static) --------
def counterOpenShelf():
    F = G.GameFrame(shift=0.085)
    with G.new_parts() as still:
        F.box('plinth', 0.996, 0.10, 0.64, (0, 0.05, -0.14), M.dark, bevel=0.004)
        F.box('worktop', 1.0, 0.05, 0.79, (0, 0.925, -0.085), M.worktop, bevel=0.008)
        # an open carcass: a back, two sides, a top and a bottom, with two shelves between
        F.box('back', 0.996, 0.8, 0.04, (0, 0.5, -0.43), M.soil, bevel=0.004)
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
        F.box('top_layer', 0.84, 0.03, 0.74, (0, 0.93, 0), M.block2, bevel=0.004)
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
    with G.new_parts() as parts:
        F.box('drawer_front', 0.7, 0.18, 0.02, (0, 0.72, 0.39), M.wood2, bevel=0.008)
        _handle_h(F, 'drawer_pull', 0, 0.72, 0.402, 0.6, M.metal)
    drawer = G.join(parts, 'drawer', origin=F.at(0, 0.72, 0.39))
    G.settle()
    G.slide(drawer, 'drawer_open', (0, -0.25, 0), seconds=SECONDS)


# --- the marble counter: 1x1, a navy unit with a marble top and brass handles, a drawer for sandwich --
def counterMarble():
    F = G.GameFrame(shift=0.085)
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


FAMILY = G.Family(
    'kitchen_counters', M,
    [G.Model('counter', counter, size=(1.0, 0.96, 0.66), key='model.counter',
             # sandwich slides the top drawer out
             motions={'uses': {'sandwich': 'drawer_open'}, 'hold': HOLD},
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop'), ('chrome', 'Handles')]),
     G.Model('counterCorner', counterCorner, size=(1.0, 1.27, 1.0), key='model.counterCorner',
             # sandwich, as the counter
             motions={'uses': {'sandwich': 'drawer_open'}, 'hold': HOLD},
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop'), ('chrome', 'Handles')]),
     G.Model('counterEnd', counterEnd, size=(1.0, 0.95, 0.79), key='model.counterEnd',
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop'), ('chrome', 'Handles')]),
     G.Model('counterCornerOuter', counterCornerOuter, size=(0.81, 1.035, 0.79), key='model.counterCornerOuter',
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop'), ('chrome', 'Handles')]),
     G.Model('counterDrawers', counterDrawers, size=(1.0, 0.95, 0.79), key='model.counterDrawers',
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop'), ('chrome', 'Handles')]),
     G.Model('counterOpenShelf', counterOpenShelf, size=(1.0, 0.95, 0.79), key='model.counterOpenShelf',
             paint=[('cab', 'Cabinet'), ('worktop', 'Worktop')]),
     G.Model('butcherBlock', butcherBlock, size=(0.9, 0.945, 0.813), key='model.butcherBlock',
             # sandwich slides the knife drawer out
             motions={'uses': {'sandwich': 'drawer_open'}, 'hold': HOLD},
             paint=[('leg', 'Base'), ('block', 'Block top')]),
     G.Model('counterMarble', counterMarble, size=(1.0, 0.953, 0.79), key='model.counterMarble',
             # sandwich slides the top drawer out
             motions={'uses': {'sandwich': 'drawer_open'}, 'hold': HOLD},
             paint=[('navy', 'Cabinet'), ('marble', 'Marble top'), ('brass', 'Handles')])],
    comment='Blender-built kitchen base counters (tools/art/blender/families/kitchen_counters.py). '
            'Replaces the primitive-part placeholders by key; listed in the base manifest\'s packs. The '
            'placeholder is kept as a fallback if a file fails to load. Each counter is a plinth, a '
            'carcass and a worktop (to 0.96 or 0.95): the plain, corner and marble counters carry a '
            'drawer over two doors and slide it out 25 cm (`drawer_open`) while a resident makes a '
            'sandwich, as does the butcher block\'s knife drawer.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

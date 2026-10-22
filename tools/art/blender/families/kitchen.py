# The kitchen: the top-freezer fridge (`model.fridge`). Notes: kitchen.md.
#
# A cabinet and two doors, each door its own node turning on its hinge, with a clip per door: the game
# opens the fridge door while a resident takes food or a drink out (`motions`). At rest both are closed.
# Built to the old part's size, in its numbers: the parts are placed in game axes (`F`).
#
# Run:  cargo art kitchen [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import bmesh  # noqa: E402
import bpy  # noqa: E402

import gamekit as G  # noqa: E402

M = G.Palette(
    body=('#F3F3F1', 'gloss'),       # white cabinet and doors; the main part the player colours (`paint`)
    trim=('#B9BCBF', 'satin'),       # plinth and badge
    seal=('#3A3D41', 'satin'),       # the gap between the doors
    gasket=('#C9CDD0', 'satin'),     # the doors' rubber seals, pale grey as on a white fridge
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
    pot=('#B5653F', 'satin'),        # a terracotta herb pot
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
    pink=('#F4B9CF', 'gloss'),       # a retro mini fridge's pastel body
    pink2=('#E9A2BD', 'gloss'),      # its trim (the base band and the door seam)
    gold=('#D2AE4E', 'metal'),       # its brass handle, feet and letters
    vent=('#3A3D41', 'satin'),       # its side vents
    uni_white=('#FFFFFF', 'satin'),  # the unicorn sticker: its die-cut border, a streak and the horn
    uni_face=('#FCEEF3', 'satin'),   # its head and ear
    uni_pink=('#F2A6C6', 'satin'),   # its mane and inner ear
    uni_rose=('#E07FAA', 'satin'),   # a deep streak, the nostril and the eyelid
    uni_blush=('#F6C6D9', 'satin'),  # the muzzle
    uni_blue=('#B9D3EE', 'satin'),   # a streak of the mane
    uni_lilac=('#D4C1EC', 'satin'),  # a streak and the horn's spiral
    uni_dark=('#3A2B33', 'matte'),   # its eye
    doorframe=('#2B2C2E', 'satin'),  # the wine fridge's dark door frame
    smoked=('#3A2A2E', 'glass'),     # its smoked glass
    wine=('#5A1E2A', 'gloss'),       # red wine bottles
    wine2=('#E3D9A0', 'gloss'),      # white wine bottles
)

# The old part's back sits 9.4 cm behind the middle of its tile (the old `BACK = -0.455`);
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


def _shell():
    """The cabinet as five panels, so the doors open onto a real interior instead of a solid block. The sides are
    rounded on their vertical edges, as a pressed-steel body is; the top's edges are rounded to meet them."""
    side = G.rounded_rect(0.03, 0.622, 0.012, corner=2)
    F.extrude('side_left', side, 1.74, (-0.355, 0.93, -0.144), M.body, plane='top', bevel=0.004, bevel_segments=1)
    F.extrude('side_right', side, 1.74, (0.355, 0.93, -0.144), M.body, plane='top', bevel=0.004, bevel_segments=1)
    F.box('top', 0.74, 0.03, 0.622, (0, 1.785, -0.144), M.body, bevel=0.012, bevel_segments=2)
    F.box('bottom', 0.74, 0.03, 0.622, (0, 0.075, -0.144), M.body, bevel=0.006)
    F.box('back', 0.74, 1.74, 0.03, (0, 0.93, -0.44), M.body, bevel=0.006)
    # the front of the plinth: a vented kick plate over a dark recess, and the hinges' covers on the right
    F.box('kick_recess', 0.66, 0.05, 0.01, (0, 0.035, 0.108), M.dark)
    slot = G.rounded_rect(0.07, 0.012, 0.006, corner=1)
    slots = [[(u + (i - 2.5) * 0.1, v) for u, v in slot] for i in range(6)]
    F.extrude('kick_plate', G.rounded_rect(0.68, 0.05, 0.008, corner=1), 0.008, (0, 0.035, 0.117), M.seal, holes=slots)
    for y in (1.224, 0.08):
        F.box(f'hinge{y}', 0.024, 0.026, 0.05, (0.356, y, 0.19), M.trim, bevel=0.006, bevel_segments=1)


def _interior():
    """The liner, shelves with trimmed fronts, two crisper drawers and food, seen once the doors are open. The
    shelves stop short of the door's bins."""
    F.box('liner_back', 0.678, 1.678, 0.006, (0, 0.93, -0.421), M.liner, bevel=0.002, bevel_segments=1)
    F.box('liner_left', 0.006, 1.678, 0.578, (-0.337, 0.93, -0.134), M.liner, bevel=0.002, bevel_segments=1)
    F.box('liner_right', 0.006, 1.678, 0.578, (0.337, 0.93, -0.134), M.liner, bevel=0.002, bevel_segments=1)
    F.box('liner_top', 0.678, 0.006, 0.578, (0, 1.767, -0.134), M.liner, bevel=0.002, bevel_segments=1)
    F.box('liner_bottom', 0.678, 0.006, 0.578, (0, 0.093, -0.134), M.liner, bevel=0.002, bevel_segments=1)
    # the divider between the freezer (top) and the fridge (bottom), the freezer's floor
    F.box('divider', 0.672, 0.02, 0.55, (0, 1.234, -0.13), M.shelf, bevel=0.002, bevel_segments=1)
    # three shelves in the fridge, one in the freezer, each with a trim along its front
    for i, y in enumerate((0.37, 0.66, 0.95, 1.52)):
        F.box(f'shelf{i}', 0.672, 0.008, 0.49, (0, y, -0.16), M.shelf)
        F.box(f'shelf_trim{i}', 0.672, 0.022, 0.01, (0, y - 0.004, 0.083), M.trim, bevel=0.003, bevel_segments=1)
    # two crisper drawers under the lowest shelf, a grip cut along each front's top
    for s in (-1, 1):
        F.box(f'crisper{s}', 0.322, 0.24, 0.46, (s * 0.166, 0.226, -0.16), M.shelf)
        F.extrude(f'crisper_front{s}', G.rounded_rect(0.322, 0.25, 0.012, corner=1), 0.012, (s * 0.166, 0.226, 0.076),
                  M.carton, holes=[[(u, v + 0.09) for u, v in G.rounded_rect(0.14, 0.022, 0.011, corner=2)]])
    # the lamp's housing and the thermostat's dial under the divider
    F.box('lamp', 0.16, 0.035, 0.09, (0.17, 1.205, -0.2), M.liner, bevel=0.008, bevel_segments=1)
    F.cyl('dial', 0.022, 0.014, (0.17, 1.18, -0.2), M.trim, verts=12)
    _fridge_food()
    _freezer_food()


def _fridge_food():
    """Normal food, in the bottom (fridge) compartment: dairy, produce and eggs."""
    top, mid, low = 0.954, 0.664, 0.374  # the fridge shelf surfaces
    F.box('milk', 0.085, 0.20, 0.085, (-0.20, top + 0.10, -0.14), M.carton, bevel=0.006)
    F.cone('milk_gable', 0.062, 0.0, 0.055, (-0.20, top + 0.2275, -0.14), M.carton, verts=4, rot=(0, 0, 45))
    _bottle('juice', (0.02, top, -0.14), 0.032, 0.22, M.bottle, M.carton)
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


def _bottle(name, foot, r, h, material, cap, verts=10, frame=F):
    """A bottle standing at game point `foot`: a round body, a shoulder, a neck and its cap."""
    bx, by, bz = frame.at(*foot)
    rings = [(0.0, 0.9), (0.025, 1.0), (0.62, 1.0), (0.78, 0.5), (0.86, 0.36), (0.94, 0.36)]
    G.loft(name, [G.ring(r * k, r * k, bz + h * t, verts, at=(bx, by)) for t, k in rings], material=material)
    frame.cyl(f'{name}_cap', r * 0.4, h * 0.06, (foot[0], foot[1] + h * 0.97, foot[2]), cap, verts=verts)


def _door_bin(name, y, x0, x1, z0, z1, frame=F):
    """A bin on a door's inside: a tray with a lip along its inner edge, from game x0 to x1, z0 (inner) to z1."""
    w, d = x1 - x0, z1 - z0
    frame.box(f'{name}_tray', w, 0.01, d, ((x0 + x1) / 2, y + 0.005, (z0 + z1) / 2), M.shelf)
    frame.box(f'{name}_lip', w, 0.05, 0.008, ((x0 + x1) / 2, y + 0.025, z0 + 0.004), M.shelf, bevel=0.003,
              bevel_segments=1)
    for s in (x0, x1):
        frame.box(f'{name}_end{s}', 0.008, 0.05, d, (s, y + 0.025, (z0 + z1) / 2), M.shelf)


def _handle(name, x, y0, y1, face):
    """A bar handle in one piece: out from the door face at `face` on rounded stand-offs, up, and back in."""
    out = face + 0.041
    F.sweep(name, [(x, y0, face - 0.004), (x, y0, out - 0.012), (x, y0 + 0.012, out), (x, y1 - 0.012, out),
                   (x, y1, out - 0.012), (x, y1, face - 0.004)],
            None, M.chrome, profile=G.rounded_rect(0.016, 0.022, 0.007, corner=1), up=(1, 0, 0), resolution=2,
            angle=60)


def _door(name, y, h, handle_y0, handle_y1, badge=False, bins=()):
    """A door, rounded and proud of the opening, its handle on the right and its hinge on the left as seen from
    the front: a dark seal round its back, an inner liner and the bins on its inside (`bins`: their heights)."""
    with G.new_parts() as parts:
        F.soft(name, 0.728, h, 0.04, (0, y, 0.195), M.body, round=0.014)
        _handle(f'handle_{name}', -0.3, handle_y0, handle_y1, 0.215)
        if badge:
            F.soft('badge', 0.1, 0.018, 0.006, (0.25, 1.74, 0.216), M.trim, round=0.003, round_segments=1)
        seal = G.rounded_rect(0.7, h - 0.024, 0.03, corner=2)
        F.sweep(f'{name}_seal', [(u, y + v, 0.172) for u, v in seal], 0.009, M.gasket, verts=5, closed=True)
        F.box(f'{name}_liner', 0.64, h - 0.08, 0.008, (0, y, 0.172), M.liner)
        for i, by in enumerate(bins):
            _door_bin(f'{name}_bin{i}', by, -0.3, 0.3, 0.1, 0.168)
    return G.hinged(G.join(parts, name), F.at(0.364, y, 0.195))


def fridge():
    # the cabinet on a recessed plinth, opening onto a liner, shelves, drawers and food: the part that stays still
    with G.new_parts() as cabinet:
        F.box('plinth', 0.68, 0.06, 0.57, (0, 0.03, -0.17), M.trim, bevel=0.006)
        _shell()
        _interior()
        F.box('gap', 0.7, 0.014, 0.03, (0, 1.224, 0.187), M.seal, bevel=0.004)
    G.join(cabinet, 'cabinet', origin=(0, 0, 0))
    # the freezer door over the fridge door, a clip each; the fridge door's bins hold bottles, jars and butter
    freezer = _door('door_freezer', 1.51, 0.556, 1.274, 1.524, badge=True, bins=(1.33,))
    with G.new_parts() as stock:
        F.box('ice_tray', 0.24, 0.035, 0.05, (-0.12, 1.358, 0.134), M.frozen, bevel=0.006, bevel_segments=1)
    door = _door('door_fridge', 0.643, 1.146, 0.804, 1.164, bins=(0.2, 0.55, 0.9))
    with G.new_parts() as stock2:
        for i, (x, mat) in enumerate(((-0.22, M.bottle), (-0.13, M.wine2), (-0.04, M.produce2))):
            _bottle(f'door_bottle{i}', (x, 0.21, 0.134), 0.03, 0.27 - 0.03 * i, mat, M.carton)
        for i, x in enumerate((0.1, 0.19)):
            F.cyl(f'jar{i}', 0.034, 0.09, (x, 0.605, 0.134), M.produce if i else M.egg, verts=10, bevel=0.004, bevel_segments=1)
            F.cyl(f'jar_lid{i}', 0.036, 0.016, (x, 0.658, 0.134), M.trim, verts=10)
        F.box('butter', 0.12, 0.05, 0.05, (-0.15, 0.935, 0.134), M.egg, bevel=0.008, bevel_segments=1)
    # the door's stock moves with it
    freezer = G.join([freezer, *stock], 'door_freezer')
    door = G.join([door, *stock2], 'door_fridge')
    G.settle()   # where the game places it (`centre`), its back on the wall
    G.swing(freezer, 'freezer_open', FREEZER_ANGLE, DOOR_SECONDS)
    G.swing(door, 'fridge_open', FRIDGE_ANGLE, DOOR_SECONDS)


# --- the sinks ----------------------------------------------------------------------------------------
# Shared by the two kitchen sinks (a family script stays self-contained). The tap and its stream are built in the
# sink's own GameFrame.
def _gooseneck(F, name, x, base_y, z, spout_z, top, material, r=0.022, tip_y=None):
    """A tap in one piece: a riser from (x, base_y, z) up to `top`, arcing forward to a downward spout with a spray
    head, on a round escutcheon, a lever on a short body beside it. Returns the spout tip's game point (where the
    water starts)."""
    tip_y = tip_y if tip_y is not None else base_y + 0.16
    pts = [(x, base_y, z),
           (x, top - 0.05, z),
           (x, top - 0.004, z + (spout_z - z) * 0.35),
           (x, top - 0.004, z + (spout_z - z) * 0.8),
           (x, top - 0.03, spout_z),
           (x, tip_y + 0.04, spout_z),
           (x, tip_y, spout_z)]
    G.kit.tube(F, name, pts, r, material, verts=10, round=3)
    F.cyl(f'{name}_head', r * 1.2, 0.045, (x, tip_y + 0.0225, spout_z), material, verts=10, bevel=0.004,
          bevel_segments=1)
    bx, by, bz = F.at(x, base_y, z)
    G.revolve(f'{name}_base', [(0.0, 0.0), (r * 1.9, 0.0), (r * 1.9, 0.006), (r * 1.5, 0.016), (r * 1.1, 0.024),
                                (0.0, 0.024)], at=(bx, by, bz), material=material, segments=14)
    lx = x + (0.075 if x <= 0 else -0.075)
    F.cyl(f'{name}_valve', 0.018, 0.045, (lx, base_y + 0.0225, z), material, verts=10, bevel=0.004, bevel_segments=1)
    F.sweep(f'{name}_lever', [(lx, base_y + 0.04, z), (lx, base_y + 0.058, z + 0.02), (lx, base_y + 0.066, z + 0.075)],
            None, material, profile=G.rounded_rect(0.014, 0.008, 0.003, corner=1), up=(1, 0, 0), resolution=2,
            scale=[1.3, 1.0, 1.15])
    return (x, tip_y, spout_z)


def _bowl(Fx, name, x, z, w, d, rim_y, floor_y, material, lip=0.02, apron=None, verts=32, square=6):
    """A sink bowl in one piece, open at the top: a flat rim `lip` wide round a w × d opening at height `rim_y`, the
    walls running down with rounded corners to a floor at `floor_y` that dips to the drain. `apron` (a height): the
    outside comes down square to there, as a fireclay block's (an apron-front sink); else it tucks under the rim (a
    bowl set into a worktop)."""
    bx, by, _ = Fx.at(x, 0, z)
    hw, hd = w / 2, d / 2

    def ring(a, b, y, sq=square):
        return G.ring(a, b, y, verts, square=sq, at=(bx, by))
    if apron is not None:
        outside = [ring(hw + lip - 0.006, hd + lip - 0.006, apron, 12), ring(hw + lip, hd + lip, apron + 0.006, 12),
                   ring(hw + lip, hd + lip, rim_y - 0.006, 12)]
    else:
        outside = [ring(hw - 0.03, hd - 0.03, floor_y - 0.012), ring(hw, hd, floor_y + 0.03), ring(hw, hd, rim_y - 0.008),
                   ring(hw + lip, hd + lip, rim_y - 0.005)]
    top = [ring(hw + lip - 0.004, hd + lip - 0.004, rim_y, 12 if apron is not None else square),
           ring(hw + 0.003, hd + 0.003, rim_y)]
    inside = [ring(hw, hd, rim_y - 0.006), ring(hw - 0.006, hd - 0.006, floor_y + 0.03),
              ring(hw - 0.025, hd - 0.025, floor_y + 0.004), ring(hw - 0.06, hd - 0.06, floor_y),
              ring(0.0, 0.0, floor_y - 0.004)]
    return G.loft(name, outside + top + inside, material=material)


def _drain(Fx, name, x, y, z, r=0.035):
    """A drain in a bowl's floor: a dark hole in a chrome ring, a strainer cross over it."""
    Fx.cyl(name, r, 0.006, (x, y, z), M.dark, verts=12)
    G.torus(f'{name}_ring', r, 0.005, at=Fx.at(x, y + 0.003, z), material=M.chrome, major_segments=12,
            minor_segments=4)
    for i in range(2):
        Fx.box(f'{name}_bar{i}', 2 * r if i else 0.006, 0.004, 0.006 if i else 2 * r, (x, y + 0.002, z), M.chrome)


def _herb(Fx, name, x, y, z, r=0.06):
    """A potted herb on game height `y`: a tapered terracotta pot with a rim, a mound of leaves on it."""
    bx, by, bz = Fx.at(x, y, z)
    G.loft(f'{name}_pot', [G.ring(r * k, r * k, bz + h, 14, at=(bx, by)) for h, k in
                           ((0.0, 0.78), (0.1, 0.95), (0.1, 1.06), (0.13, 1.06), (0.13, 0.98))], material=M.pot)
    Fx.cyl(f'{name}_soil', r * 0.94, 0.01, (x, y + 0.122, z), M.board, verts=12)
    for i in range(7):
        a = i * 2.4
        d = 0.0 if i == 0 else r * (0.45 + 0.08 * (i % 3))
        h = 0.06 if i == 0 else 0.03 + 0.012 * (i % 3)
        Fx.sphere(f'{name}_leaves{i}', r * (0.62 if i == 0 else 0.48),
                  (x + d * math.cos(a), y + 0.13 + h, z + d * math.sin(a)), M.plant if i % 2 else M.bottle,
                  segments=8, rings=5, scale=(1, 1, 0.8))


def _soap_dish(Fx, name, x, y, z, w=0.16, d=0.1):
    """A soap dish on game height `y` with a bar of soap in it."""
    bx, by, bz = Fx.at(x, y, z)
    G.loft(name, [G.ring(w / 2 * k, d / 2 * k, bz + h, 16, square=3, at=(bx, by)) for h, k in
                  ((0.0, 0.8), (0.006, 0.9), (0.02, 1.0), (0.022, 0.96), (0.012, 0.85))], material=M.porcelain)
    Fx.soft(f'{name}_soap', w * 0.55, 0.025, d * 0.55, (x, y + 0.026, z), M.egg, round=0.01, round_segments=1)


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
    """A fireclay apron-front basin standing proud of a painted cabinet with a face frame and two shaker doors,
    under a wooden worktop cut round it; a gooseneck tap with a lever, a soap dish and a potted herb."""
    Fk = G.GameFrame(shift=0.145)
    with G.new_parts() as still:
        Fk.box('plinth', 0.96, 0.08, 0.54, (0, 0.04, -0.15), M.trim, bevel=0.006)
        Fk.box('cabinet', 0.98, 0.64, 0.56, (0, 0.40, -0.14), M.body, bevel=0.006)
        Fk.extrude('face_frame', G.rounded_rect(0.98, 0.64, 0.004, corner=1), 0.02, (0, 0.40, 0.15), M.body,
                   holes=[[(u, v + 0.01) for u, v in G.rounded_rect(0.88, 0.56, 0.002, corner=1)]], bevel=0.003,
                   bevel_segments=1)
        for s in (-1, 1):
            _shaker(Fk, f'door{s}', 0.436, 0.552, (s * 0.2205, 0.41, 0.15), M.body, t=0.02, rail=0.065)
            _pull(Fk, f'pull{s}', (s * 0.05, 0.49), (s * 0.05, 0.63), 0.16, out=0.026, w=0.012, t=0.012, r=0.005)
        # the painted shoulders round the basin up to the worktop, and the worktop cut round the basin
        Fk.box('shoulder_back', 1.0, 0.2, 0.15, (0, 0.82, -0.405), M.body, bevel=0.006)
        for s in (-1, 1):
            Fk.box(f'shoulder{s}', 0.17, 0.2, 0.51, (s * 0.415, 0.82, -0.075), M.body, bevel=0.006)
        top = [(-0.5, -0.48), (0.5, -0.48), (0.5, 0.18), (0.33, 0.18), (0.33, -0.33), (-0.33, -0.33), (-0.33, 0.18),
               (-0.5, 0.18)]
        Fk.extrude('worktop', top, 0.04, (0, 0.93, 0), M.wood, plane='top', bevel=0.006, bevel_segments=2)
        # the apron basin: one fireclay piece, square outside, a rounded bowl inside, a drain in its floor
        _bowl(Fk, 'apron', 0, -0.075, 0.6, 0.45, 0.95, 0.785, M.porcelain, lip=0.03, apron=0.72)
        _drain(Fk, 'drain', 0, 0.783, -0.075)
        # the tap, a soap dish and a potted herb on the worktop
        spout = _gooseneck(Fk, 'tap', 0, 0.95, -0.38, -0.2, 1.228, M.chrome, r=0.025, tip_y=1.16)
        _soap_dish(Fk, 'soap_dish', 0.36, 0.95, -0.36, 0.18, 0.11)
        _herb(Fk, 'herb', -0.36, 0.95, -0.38)
    G.join(still, 'body', origin=(0, 0, 0))
    water = _water(Fk, spout, 0.785)
    G.settle()
    G.grow(water, 'water_on', seconds=0.4)
    return G.meshes()


# --- double counter sink: two basins in a two-metre counter run ----------------------------------------
def counterSinkDouble():
    """Two steel bowls set into a stone worktop over a two-metre run of four shaker doors, a gooseneck tap with a
    lever behind each bowl, a soap dish and a potted herb between them. Built in the counters' drawn 0.66 m depth
    (kitchen_counters.py COUNTER66's drawn numbers: the worktop -0.43..0.23)."""
    Fk = G.GameFrame(shift=0.085)
    with G.new_parts() as still:
        Fk.box('plinth', 1.996, 0.1, 0.55, (0, 0.05, -0.136), M.trim, bevel=0.006)
        Fk.box('carcass', 1.996, 0.66, 0.60, (0, 0.43, -0.11), M.body, bevel=0.006)
        # the rails round the bowls under the worktop (the carcass stops below the bowls)
        Fk.box('rail_front', 1.996, 0.12, 0.05, (0, 0.82, 0.165), M.body)
        Fk.box('rail_back', 1.996, 0.12, 0.1, (0, 0.82, -0.36), M.body)
        for s in (-1, 1):
            Fk.box(f'rail_end{s}', 0.02, 0.12, 0.6, (s * 0.988, 0.82, -0.11), M.body)
        for i, x in enumerate((-0.74, -0.26, 0.26, 0.74)):
            _shaker(Fk, f'door{i}', 0.468, 0.74, (x, 0.49, 0.2), M.body, t=0.02, rail=0.065)
            hx = x + (0.19 if i in (0, 2) else -0.19)
            _pull(Fk, f'handle{i}', (hx, 0.62), (hx, 0.8), 0.21, out=0.018, w=0.012, t=0.01, r=0.004)
        # the worktop with the two bowls' openings, and the bowls set into it
        holes = [[(u + s * 0.42, v + 0.033) for u, v in G.rounded_rect(0.625, 0.385, 0.04, corner=3)] for s in (-1, 1)]
        Fk.extrude('worktop', G.rounded_rect(2.0, 0.66, 0.004, corner=1), 0.07, (0, 0.915, -0.10), M.stone, plane='top',
                   holes=holes, bevel=0.006, bevel_segments=2)
        for s in (-1, 1):
            x = s * 0.42
            _bowl(Fk, f'basin{s}', x, -0.067, 0.6, 0.36, 0.955, 0.79, M.metal, lip=0.025, verts=24)
            _drain(Fk, f'drain{s}', x, 0.788, -0.067)
            _gooseneck(Fk, f'tap{s}', x, 0.95, -0.341, -0.17, 1.235, M.chrome, r=0.028, tip_y=1.17)
        _soap_dish(Fk, 'soap_dish', -0.2, 0.95, -0.37, 0.16, 0.1)
        _herb(Fk, 'herb', 0.0, 0.95, -0.36)
    G.join(still, 'body', origin=(0, 0, 0))
    # Each bowl's tap runs for the one washing at it (`water_on_<slot>`): slot 0 is the bowl at -x, 1 at +x, as
    # sim-core spreads the two slots along the width (objectMotion.ts `slotOf`).
    waters = [_water(Fk, (x, 1.17, -0.17), 0.79) for x in (-0.42, 0.42)]
    G.settle()
    for slot, water in enumerate(waters):
        G.grow(water, f'water_on_{slot}', seconds=0.4)
    return G.meshes()


# --- K1: the rest of the fridge family ---------------------------------------------------------------
# Five more fridges, built to the old parts' sizes with the classic fridge's mechanics: a door
# hinged on one edge (`G.hinged`) with `G.swing`, and a `hold` so it shuts while the resident is busy.
# The French-door freezer and the wine fridge's top rack slide (`G.slide`).


# --- parts the K1 fridges and the sinks share (each in the model's own GameFrame) ---------------------------

def _pull(Fx, name, a, b, face, material=None, out=0.034, w=0.016, t=0.02, r=0.007, verts=8):
    """A bar handle in one piece from face point `a` (x, height) to `b`: out from the face at game depth `face` on
    rounded stand-offs, along, and back in, its front `out` proud; a rounded `w` × `t` section (`r` None: a round bar of radius w / 2)."""
    (x0, y0), (x1, y1) = a, b
    n = math.hypot(x1 - x0, y1 - y0)
    ux, uy = (x1 - x0) / n, (y1 - y0) / n
    e, o = min(0.012, n / 4), face + out - (w if r is None else t) / 2
    pts = [(x0, y0, face - 0.004), (x0, y0, o - e), (x0 + ux * e, y0 + uy * e, o), (x1 - ux * e, y1 - uy * e, o),
           (x1, y1, o - e), (x1, y1, face - 0.004)]
    kw = dict(radius=w / 2, verts=verts) if r is None else dict(profile=G.rounded_rect(w, t, r, corner=1))
    return Fx.sweep(name, pts, kw.pop('radius', None), material or M.chrome, up=(uy, 0, ux), resolution=2, angle=60, **kw)


def _shaker(Fx, name, w, h, at, material, t=0.02, rail=0.06, r=0.004):
    """A cabinet door or drawer front: a frame (stiles and rails, one slab with a hole) round a panel set 6 mm back.
    `at` is the middle of its front face's slab (game x, height, depth)."""
    x, y, z = at
    hole = G.rounded_rect(w - 2 * rail, h - 2 * rail, 0.003, corner=1)
    Fx.extrude(f'{name}_frame', G.rounded_rect(w, h, r, corner=1), t, at, material, holes=[hole], bevel=0.003,
               bevel_segments=1)
    # the panel a hair inside the hole, lightly puffed: its grid gives the baked shading points away from the frame's
    # edge, so only its border darkens (a flat box's four corners, hidden in the frame, darkened all of it)
    Fx.soft(f'{name}_panel', w - 2 * rail - 0.002, h - 2 * rail - 0.002, t * 0.6, (x, y, z + t * 0.2 - 0.006), material,
            puff=0.002, round=0.003, round_segments=1, flat=2)


def _gasket(Fx, name, w, h, y, z, r=0.008, corner=0.03, x=0.0):
    """A door's rubber seal: a cord round the back of a w × h door whose middle is at (x, y), at depth `z`."""
    loop = G.rounded_rect(w, h, corner, corner=2)
    return Fx.sweep(name, [(x + u, y + v, z) for u, v in loop], r, M.gasket, verts=5, closed=True)


def _kick(Fx, name, w, h, y, z, material=None, slots=6):
    """A kick plate with a row of vent slots, over a dark recess, at height `y` (its middle) and depth `z`."""
    Fx.box(f'{name}_recess', w - 0.02, h, 0.01, (0, y, z - 0.009), M.dark)
    slot = G.rounded_rect(min(0.07, w / slots - 0.03), min(0.012, h * 0.3), 0.005, corner=1)
    step = (w - 0.06) / slots
    holes = [[(u + (i - (slots - 1) / 2) * step, v) for u, v in slot] for i in range(slots)]
    Fx.extrude(name, G.rounded_rect(w, h, 0.006, corner=1), 0.008, (0, y, z), material or M.seal, holes=holes)


def _carton(Fx, name, x, y, z, w, h, material, turn=0):
    """A gable-top carton standing on game height `y`: its side outline (a house with a fin) extruded across."""
    d, g = w, w * 0.42
    outline = [(-d / 2, 0), (d / 2, 0), (d / 2, h), (0.003, h + g), (0.003, h + g + 0.014), (-0.003, h + g + 0.014),
               (-0.003, h + g), (-d / 2, h)]
    return Fx.extrude(name, outline, w, (x, y, z), material, plane='side', rot=(0, 0, turn), bevel=0.002,
                      bevel_segments=1)


def _apple(Fx, name, x, y, z, r=0.035, material=None):
    """An apple sitting on game height `y`: a squat ball with a dimple's stalk."""
    Fx.sphere(name, r, (x, y + r * 0.92, z), material or M.produce, segments=10, rings=7, scale=(1, 1, 0.92))
    Fx.cyl(f'{name}_stalk', 0.0028, 0.016, (x + 0.002, y + r * 1.78 + 0.006, z), M.board, verts=5, smooth=False)


def _jar(Fx, name, x, y, z, r, h, material, lid, verts=12):
    """A jar on game height `y`: a rounded body, a shoulder and a screw lid."""
    bx, by, bz = Fx.at(x, y, z)
    rings = [(0.0, 0.86), (0.04, 1.0), (0.8, 1.0), (0.9, 0.84), (0.92, 0.8)]
    G.loft(name, [G.ring(r * k, r * k, bz + h * t, verts, at=(bx, by)) for t, k in rings], material=material)
    Fx.cyl(f'{name}_lid', r * 0.84, h * 0.12, (x, y + h * 0.97, z), lid, verts=verts, bevel=0.002, bevel_segments=1)


def _fridgeMini():
    """A small under-counter fridge: a white cabinet on a vented plinth, one door over its whole front (its seal,
    two bins and their bottles inside), an ice box with a flap at the top, a shelf and a crisper below, and a
    chopping board, a jar and an apple on top."""
    Fm = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        Fm.box('plinth', 0.56, 0.04, 0.50, (0, 0.02, -0.035), M.trim, bevel=0.006)
        _kick(Fm, 'kick', 0.52, 0.03, 0.02, 0.215, slots=5)
        # the cabinet: sides rounded on their front and back edges, a top rounded all round, a back and a floor
        side = G.rounded_rect(0.03, 0.54, 0.012, corner=2)
        for s in (-1, 1):
            Fm.extrude(f'side{s}', side, 0.80, (s * 0.285, 0.44, -0.015), M.body, plane='top', bevel=0.003,
                       bevel_segments=1)
        Fm.box('top', 0.60, 0.03, 0.54, (0, 0.855, -0.015), M.body, bevel=0.012, bevel_segments=2)
        Fm.box('bottom', 0.57, 0.03, 0.54, (0, 0.055, -0.015), M.body, bevel=0.004)
        Fm.box('back', 0.57, 0.80, 0.03, (0, 0.44, -0.27), M.body, bevel=0.004)
        # the liner, the ice box across the top (a flap with a grip), a shelf with a trim, a crisper on the floor
        Fm.box('liner_back', 0.54, 0.77, 0.006, (0, 0.455, -0.252), M.liner)
        for s in (-1, 1):
            Fm.box(f'liner{s}', 0.006, 0.77, 0.5, (s * 0.267, 0.455, -0.005), M.liner)
        Fm.box('icebox', 0.52, 0.13, 0.42, (0, 0.765, -0.04), M.liner, bevel=0.01, bevel_segments=1)
        Fm.soft('icebox_flap', 0.50, 0.12, 0.014, (0, 0.765, 0.176), M.shelf, round=0.006, round_segments=1)
        Fm.box('icebox_grip', 0.14, 0.014, 0.012, (0, 0.715, 0.188), M.trim, bevel=0.005, bevel_segments=1)
        Fm.box('shelf', 0.52, 0.008, 0.40, (0, 0.40, -0.05), M.shelf)
        Fm.box('shelf_trim', 0.52, 0.02, 0.01, (0, 0.397, 0.15), M.trim, bevel=0.003, bevel_segments=1)
        Fm.box('crisper', 0.50, 0.14, 0.40, (0, 0.145, -0.05), M.shelf, bevel=0.006, bevel_segments=1)
        Fm.extrude('crisper_front', G.rounded_rect(0.50, 0.15, 0.012, corner=1), 0.012, (0, 0.145, 0.156), M.carton,
                   holes=[[(u, v + 0.045) for u, v in G.rounded_rect(0.13, 0.022, 0.011, corner=2)]])
        # food: milk, a bottle and apples on the shelf
        _carton(Fm, 'milk', -0.16, 0.404, -0.08, 0.07, 0.14, M.carton)
        _bottle('juice', (-0.05, 0.404, -0.1), 0.028, 0.2, M.bottle, M.carton, frame=Fm)
        for i, (x, z) in enumerate(((0.07, -0.02), (0.15, -0.05), (0.11, -0.12))):
            _apple(Fm, f'apple{i}', x, 0.404, z, 0.034)
        # a chopping board (a hanging hole at one end), a jar and an apple on top
        Fm.extrude('board', G.rounded_rect(0.50, 0.40, 0.03, corner=3), 0.02, (0, 0.88, -0.02), M.board, plane='top',
                   holes=[[(u + 0.2, v) for u, v in G.circle(0.012, 8)]], bevel=0.004, bevel_segments=1)
        _jar(Fm, 'jar', -0.14, 0.89, -0.04, 0.05, 0.107, M.porcelain, M.trim)
        _apple(Fm, 'apple_top', 0.12, 0.89, 0.0, 0.042)
    G.join(still, 'body', origin=(0, 0, 0))
    # the door over the whole front, hinged on its left edge: its seal, a liner and two bins with bottles and a can
    with G.new_parts() as parts:
        Fm.soft('door', 0.58, 0.77, 0.03, (0, 0.475, 0.27), M.body, round=0.012)
        Fm.soft('badge', 0.08, 0.016, 0.005, (-0.18, 0.80, 0.286), M.trim, round=0.003, round_segments=1)
        _pull(Fm, 'handle', (0.24, 0.36), (0.24, 0.62), 0.285)
        _gasket(Fm, 'door_seal', 0.55, 0.74, 0.475, 0.252)
        Fm.box('door_liner', 0.50, 0.68, 0.008, (0, 0.475, 0.25), M.liner)
        for i, y in enumerate((0.16, 0.50)):
            _door_bin(f'bin{i}', y, -0.23, 0.23, 0.17, 0.246, frame=Fm)
        for i, (x, mat) in enumerate(((-0.18, M.bottle), (-0.11, M.wine2))):
            _bottle(f'door_bottle{i}', (x, 0.17, 0.205), 0.027, 0.24 - 0.03 * i, mat, M.carton, frame=Fm)
        Fm.cyl('can', 0.026, 0.1, (0.15, 0.56, 0.205), M.produce, verts=10, bevel=0.003, bevel_segments=1)
        _jar(Fm, 'door_jar', 0.07, 0.51, 0.205, 0.03, 0.08, M.egg, M.trim, verts=10)
    door = G.hinged(G.join(parts, 'door'), Fm.at(-0.29, 0.40, 0.27))
    G.settle()
    G.swing(door, 'fridge_open', 100, seconds=1.6)


def _fridgeBuiltIn():
    """A tall fridge-freezer behind shaker cabinet fronts under a moulded cornice: a fridge door (a water dispenser
    set into it) over a freezer door, long bar handles. Inside, glass shelves, crispers and food over two freezer
    baskets; the doors carry their seals, bins and bottles."""
    Fb = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        Fb.box('plinth', 0.78, 0.10, 0.54, (0, 0.05, -0.05), M.dark, bevel=0.004)
        _kick(Fb, 'kick', 0.66, 0.05, 0.065, 0.236, slots=7)
        for s in (-1, 1):
            Fb.box(f'side{s}', 0.03, 1.88, 0.58, (s * 0.395, 1.06, -0.03), M.panel, bevel=0.004)
        Fb.box('top', 0.76, 0.03, 0.58, (0, 1.985, -0.03), M.panel)
        Fb.box('bottom', 0.76, 0.03, 0.58, (0, 0.115, -0.03), M.panel)
        Fb.box('back', 0.76, 1.88, 0.03, (0, 1.06, -0.30), M.panel)
        # the cornice: a slab with a cove moulding swept round its front and sides
        Fb.box('cornice', 0.80, 0.05, 0.60, (0, 2.0, -0.015), M.panel)
        # (up, out) from the slab's edge: a cove rising from the panel face to a square lip
        cove = [(0.0, -0.01), (0.0, 0.004)] + [(0.004 + 0.036 * math.sin(math.radians(a)),
                                                0.004 + 0.016 * (1 - math.cos(math.radians(a)))) for a in range(0, 91, 18)]
        cove += [(0.05, 0.02), (0.05, -0.01)]
        Fb.sweep('cornice_moulding', [(0.4, 1.975, -0.315), (0.4, 1.975, 0.285), (-0.4, 1.975, 0.285),
                                      (-0.4, 1.975, -0.315)], None, M.panel, profile=cove, up=(0, 0, 1), angle=50)
        # the liner, glass shelves with trims, two crispers, the divider and the freezer's two baskets
        Fb.box('liner_back', 0.76, 1.72, 0.01, (0, 1.06, -0.28), M.liner)
        for s in (-1, 1):
            Fb.box(f'liner{s}', 0.006, 1.72, 0.54, (s * 0.377, 1.06, -0.02), M.liner)
        Fb.box('liner_top', 0.76, 0.006, 0.54, (0, 1.917, -0.02), M.liner)
        Fb.box('divider', 0.76, 0.02, 0.44, (0, 0.80, -0.06), M.shelf)
        Fb.box('divider_trim', 0.76, 0.05, 0.02, (0, 0.80, 0.17), M.liner, bevel=0.006, bevel_segments=1)
        for i, y in enumerate((1.08, 1.38, 1.68)):
            Fb.box(f'shelf{i}', 0.74, 0.008, 0.42, (0, y, -0.06), M.shelf)
            Fb.box(f'shelf_trim{i}', 0.74, 0.022, 0.012, (0, y - 0.004, 0.15), M.trim, bevel=0.004, bevel_segments=1)
        for s in (-1, 1):
            Fb.box(f'crisper{s}', 0.36, 0.22, 0.40, (s * 0.185, 0.94, -0.06), M.shelf, bevel=0.006, bevel_segments=1)
            Fb.extrude(f'crisper_front{s}', G.rounded_rect(0.36, 0.23, 0.012, corner=1), 0.012, (s * 0.185, 0.94, 0.146),
                       M.carton, holes=[[(u, v + 0.08) for u, v in G.rounded_rect(0.14, 0.022, 0.011, corner=2)]])
        for i, y in enumerate((0.16, 0.48)):
            Fb.box(f'basket{i}', 0.70, 0.02, 0.42, (0, y, -0.06), M.liner)
            Fb.extrude(f'basket_front{i}', G.rounded_rect(0.70, 0.2, 0.012, corner=1), 0.014, (0, y + 0.09, 0.152),
                       M.shelf, holes=[[(u, v + 0.06) for u, v in G.rounded_rect(0.16, 0.026, 0.013, corner=2)]])
            for s in (-1, 1):
                Fb.box(f'basket_side{i}{s}', 0.012, 0.18, 0.42, (s * 0.344, y + 0.09, -0.06), M.liner)
        # food, fresh above and frozen below
        _carton(Fb, 'milk', -0.24, 1.084, -0.06, 0.08, 0.17, M.carton)
        _bottle('juice', (-0.1, 1.084, -0.08), 0.032, 0.24, M.bottle, M.carton, frame=Fb)
        _jar(Fb, 'jam', 0.08, 1.084, -0.04, 0.04, 0.1, M.produce, M.trim)
        for i, (x, z) in enumerate(((-0.24, -0.02), (-0.15, -0.06), (-0.2, -0.13))):
            _apple(Fb, f'apple{i}', x, 1.384, z, 0.036)
        _apple(Fb, 'orange', -0.06, 1.384, -0.03, 0.04, M.produce2)
        Fb.box('egg_carton', 0.22, 0.03, 0.11, (0.16, 1.40, -0.05), M.carton, bevel=0.006)
        for i in range(4):
            Fb.sphere(f'egg{i}', 0.018, (0.085 + 0.05 * i, 1.432, -0.05), M.egg, segments=8, rings=5, scale=(1, 1, 1.25))
        Fb.box('tub', 0.14, 0.07, 0.14, (-0.16, 1.719, -0.06), M.shelf, bevel=0.012, bevel_segments=1)
        Fb.box('pizza', 0.26, 0.03, 0.26, (-0.15, 0.505, -0.06), M.frozen, bevel=0.006)
        Fb.box('peas', 0.12, 0.16, 0.06, (0.14, 0.57, -0.1), M.peas, bevel=0.012, bevel_segments=1)
        Fb.box('ice_cream', 0.12, 0.10, 0.12, (0.16, 0.22, -0.06), M.icecream, bevel=0.01)
    G.join(still, 'body', origin=(0, 0, 0))
    # the tall fridge door, hinged on its left edge: a shaker front with a water dispenser set into it and a bar
    # handle; its seal, liner and bins with bottles on its inside
    with G.new_parts() as parts:
        _shaker(Fb, 'fridge_door', 0.78, 1.14, (0, 1.39, 0.30), M.panel, t=0.025, rail=0.07)
        Fb.extrude('dispenser', G.rounded_rect(0.16, 0.24, 0.02, corner=2), 0.014, (-0.24, 1.42, 0.31), M.trim,
                   holes=[G.rounded_rect(0.13, 0.21, 0.012, corner=2)], bevel=0.003, bevel_segments=1)
        Fb.box('dispenser_back', 0.14, 0.22, 0.01, (-0.24, 1.42, 0.305), M.dark)
        Fb.box('dispenser_screen', 0.08, 0.025, 0.004, (-0.24, 1.505, 0.311), M.water)
        Fb.box('dispenser_paddle', 0.06, 0.07, 0.008, (-0.24, 1.43, 0.311), M.seal, bevel=0.003, bevel_segments=1)
        Fb.cyl('dispenser_spout', 0.008, 0.02, (-0.24, 1.475, 0.302), M.chrome, verts=8)
        Fb.extrude('drip_tray', G.rounded_rect(0.12, 0.03, 0.008, corner=1), 0.012, (-0.24, 1.33, 0.312), M.seal,
                   plane='front')
        _pull(Fb, 'handle_fridge', (0.33, 1.08), (0.33, 1.70), 0.3125, out=0.03, w=0.018, r=None)
        _gasket(Fb, 'fridge_seal', 0.75, 1.11, 1.39, 0.282, r=0.008)
        Fb.box('fridge_liner', 0.70, 1.04, 0.008, (0, 1.39, 0.284), M.liner)
        for i, y in enumerate((0.88, 1.18, 1.5)):
            _door_bin(f'fridge_bin{i}', y, -0.33, 0.33, 0.18, 0.28, frame=Fb)
        for i, (x, mat) in enumerate(((-0.28, M.bottle), (-0.2, M.wine2), (-0.12, M.produce2))):
            _bottle(f'door_bottle{i}', (x, 0.89, 0.23), 0.03, 0.27 - 0.025 * i, mat, M.carton, frame=Fb)
        _carton(Fb, 'door_milk', 0.2, 0.89, 0.23, 0.07, 0.17, M.carton)
        for i, x in enumerate((-0.2, -0.11)):
            _jar(Fb, f'door_jar{i}', x, 1.19, 0.23, 0.033, 0.09, M.egg if i else M.produce, M.trim, verts=10)
        Fb.box('butter', 0.12, 0.05, 0.06, (0.18, 1.215, 0.23), M.egg, bevel=0.008, bevel_segments=1)
    fridge_door = G.hinged(G.join(parts, 'fridge_door'), Fb.at(-0.39, 1.39, 0.30))
    # the freezer door below, same hinge, a clip of its own (unused until a use takes frozen food)
    with G.new_parts() as parts:
        _shaker(Fb, 'freezer_door', 0.78, 0.62, (0, 0.50, 0.30), M.panel, t=0.025, rail=0.07)
        _pull(Fb, 'handle_freezer', (-0.25, 0.72), (0.25, 0.72), 0.3125, out=0.03, w=0.018, r=None)
        _gasket(Fb, 'freezer_seal', 0.75, 0.59, 0.50, 0.282, r=0.008)
        Fb.box('freezer_liner', 0.70, 0.52, 0.008, (0, 0.50, 0.284), M.liner)
    freezer = G.hinged(G.join(parts, 'freezer_door'), Fb.at(-0.39, 0.50, 0.30))
    G.settle()
    G.swing(fridge_door, 'fridge_open', 100, seconds=DOOR_SECONDS)
    G.swing(freezer, 'freezer_open', 100, seconds=DOOR_SECONDS)


def _fridgeFrenchDoor():
    """A stainless French-door fridge: two upper doors (an ice and water dispenser set into the left one) with long
    bar handles, a full-width freezer drawer below with frozen food in its bin. Inside, glass shelves, crispers and
    food; the doors carry their seals, bins and bottles."""
    Ff = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        Ff.box('plinth', 0.86, 0.04, 0.62, (0, 0.02, -0.07), M.trim, bevel=0.006)
        _kick(Ff, 'kick', 0.8, 0.034, 0.024, 0.24, M.dark, slots=8)
        # the cabinet's sides, top and bottom come forward to the doors' backs, so a shut door sits flush on them;
        # the sides rounded on their front and back edges, as pressed steel is
        side = G.rounded_rect(0.03, 0.71, 0.012, corner=2)
        for s in (-1, 1):
            Ff.extrude(f'side{s}', side, 1.76, (s * 0.435, 0.92, -0.025), M.steel, plane='top', bevel=0.003,
                       bevel_segments=1)
        Ff.box('top', 0.87, 0.03, 0.71, (0, 1.805, -0.025), M.steel, bevel=0.01, bevel_segments=2)
        Ff.box('bottom', 0.87, 0.03, 0.71, (0, 0.035, -0.025), M.steel)
        Ff.box('back', 0.87, 1.76, 0.03, (0, 0.92, -0.365), M.steel)
        # the liner, the divider above the freezer, glass shelves with trims, two crispers
        Ff.box('liner_back', 0.84, 1.70, 0.01, (0, 0.92, -0.345), M.liner)
        for s in (-1, 1):
            Ff.box(f'liner{s}', 0.006, 1.70, 0.66, (s * 0.417, 0.92, -0.02), M.liner)
        Ff.box('liner_top', 0.84, 0.006, 0.66, (0, 1.77, -0.02), M.liner)
        Ff.box('divider', 0.84, 0.02, 0.64, (0, 0.68, -0.03), M.shelf)
        Ff.box('divider_trim', 0.84, 0.05, 0.02, (0, 0.68, 0.30), M.steel, bevel=0.006, bevel_segments=1)
        for i, y in enumerate((0.96, 1.24, 1.52)):
            Ff.box(f'shelf{i}', 0.82, 0.008, 0.54, (0, y, -0.07), M.shelf)
            Ff.box(f'shelf_trim{i}', 0.82, 0.022, 0.012, (0, y - 0.004, 0.2), M.trim, bevel=0.004, bevel_segments=1)
        for s in (-1, 1):
            Ff.box(f'crisper{s}', 0.4, 0.2, 0.5, (s * 0.205, 0.80, -0.07), M.shelf, bevel=0.006, bevel_segments=1)
            Ff.extrude(f'crisper_front{s}', G.rounded_rect(0.4, 0.21, 0.012, corner=1), 0.012, (s * 0.205, 0.80, 0.186),
                       M.carton, holes=[[(u, v + 0.07) for u, v in G.rounded_rect(0.15, 0.022, 0.011, corner=2)]])
        # food
        _carton(Ff, 'milk', -0.28, 0.964, -0.1, 0.08, 0.17, M.carton)
        _bottle('juice', (-0.15, 0.964, -0.12), 0.033, 0.25, M.bottle, M.carton, frame=Ff)
        _jar(Ff, 'jam', 0.02, 0.964, -0.06, 0.04, 0.1, M.produce, M.trim)
        Ff.box('tub', 0.15, 0.07, 0.15, (0.22, 0.999, -0.08), M.shelf, bevel=0.012, bevel_segments=1)
        for i, (x, z) in enumerate(((-0.28, -0.02), (-0.19, -0.06), (-0.24, -0.13))):
            _apple(Ff, f'apple{i}', x, 1.244, z, 0.036)
        _apple(Ff, 'orange', -0.1, 1.244, -0.02, 0.04, M.produce2)
        Ff.box('egg_carton', 0.22, 0.03, 0.11, (0.18, 1.26, -0.06), M.carton, bevel=0.006)
        for i in range(4):
            Ff.sphere(f'egg{i}', 0.018, (0.105 + 0.05 * i, 1.292, -0.06), M.egg, segments=8, rings=5, scale=(1, 1, 1.25))
        Ff.box('cake', 0.2, 0.09, 0.2, (0.05, 1.569, -0.1), M.icecream, bevel=0.02, bevel_segments=2)
    G.join(still, 'body', origin=(0, 0, 0))

    def door(name, s):
        """An upper door, `s` -1 on the left: a rounded steel slab, its handle by the middle, its seal, liner and bins."""
        x = s * 0.222
        Ff.soft(name, 0.436, 1.10, 0.026, (x, 1.25, 0.343), M.steel, round=0.012)
        _pull(Ff, f'handle_{name}', (s * 0.04, 0.92), (s * 0.04, 1.62), 0.356, out=0.019, w=0.016, t=0.012, r=0.005)
        _gasket(Ff, f'{name}_seal', 0.41, 1.07, 1.25, 0.326, r=0.007, x=x)
        Ff.box(f'{name}_liner', 0.36, 1.0, 0.008, (x, 1.25, 0.328), M.liner)
        for i, y in enumerate((0.78, 1.08, 1.38)):
            _door_bin(f'{name}_bin{i}', y, x - 0.17, x + 0.17, 0.235, 0.325, frame=Ff)
    # the left upper door (game -x: the viewer's right), hinged on its left edge, and the right, hinged on its right
    with G.new_parts() as parts:
        door('door_left', -1)
        for i, x in enumerate((-0.12, -0.2, -0.28)):
            _jar(Ff, f'door_jar{i}', x, 1.09, 0.28, 0.032, 0.09, (M.egg, M.produce, M.bottle)[i], M.trim, verts=10)
        Ff.box('butter', 0.12, 0.05, 0.06, (-0.2, 1.405, 0.28), M.egg, bevel=0.008, bevel_segments=1)
    door_left = G.hinged(G.join(parts, 'door_left'), Ff.at(-0.44, 1.25, 0.345))
    with G.new_parts() as parts:
        door('door_right', 1)
        # the dispenser on the door the viewer sees on the left: a dark recess in a trim, a paddle and a spout, and
        # the display over it
        Ff.extrude('dispenser', G.rounded_rect(0.18, 0.28, 0.025, corner=2), 0.01, (0.24, 1.3, 0.358), M.trim,
                   holes=[G.rounded_rect(0.15, 0.25, 0.016, corner=2)], bevel=0.002, bevel_segments=1)
        Ff.box('dispenser_back', 0.16, 0.26, 0.008, (0.24, 1.3, 0.354), M.dark)
        Ff.box('dispenser_paddle', 0.07, 0.08, 0.008, (0.24, 1.32, 0.36), M.seal, bevel=0.003, bevel_segments=1)
        Ff.cyl('dispenser_spout', 0.009, 0.02, (0.24, 1.38, 0.352), M.chrome, verts=8)
        Ff.box('dispenser_tray', 0.12, 0.012, 0.03, (0.24, 1.19, 0.358), M.seal, bevel=0.004, bevel_segments=1)
        Ff.box('display', 0.12, 0.05, 0.004, (0.24, 1.49, 0.357), M.dark, bevel=0.002, bevel_segments=1)
        Ff.box('display_glow', 0.07, 0.012, 0.003, (0.24, 1.495, 0.359), M.water)
        # the flip strip the other door's edge closes on
        Ff.box('mullion', 0.04, 1.02, 0.022, (0.025, 1.25, 0.318), M.liner, bevel=0.006, bevel_segments=1)
        for i, (x, mat) in enumerate(((0.36, M.bottle), (0.29, M.wine2), (0.22, M.produce2))):
            _bottle(f'door_bottle{i}', (x, 0.79, 0.28), 0.028, 0.26 - 0.025 * i, mat, M.carton, frame=Ff)
        _carton(Ff, 'door_milk', 0.12, 0.79, 0.28, 0.07, 0.17, M.carton)
    door_right = G.hinged(G.join(parts, 'door_right'), Ff.at(0.44, 1.25, 0.345))
    # the freezer drawer, sliding out to the front: a steel front with a long pull, its bin of frozen food
    with G.new_parts() as parts:
        Ff.soft('drawer', 0.88, 0.58, 0.026, (0, 0.38, 0.343), M.steel, round=0.012)
        _pull(Ff, 'handle_drawer', (-0.3, 0.6), (0.3, 0.6), 0.356, out=0.019, w=0.016, t=0.012, r=0.005)
        Ff.box('drawer_floor', 0.80, 0.012, 0.42, (0, 0.13, 0.11), M.liner)
        for s in (-1, 1):
            Ff.box(f'drawer_side{s}', 0.012, 0.44, 0.42, (s * 0.394, 0.35, 0.11), M.liner)
        Ff.box('drawer_back', 0.80, 0.44, 0.012, (0, 0.35, -0.094), M.liner)
        Ff.box('drawer_inner', 0.80, 0.44, 0.012, (0, 0.35, 0.314), M.liner)
        Ff.box('pizza', 0.30, 0.035, 0.30, (-0.2, 0.154, 0.1), M.frozen, bevel=0.006)
        Ff.box('pizza2', 0.30, 0.035, 0.30, (-0.19, 0.189, 0.11), M.frozen, bevel=0.006)
        Ff.box('ice_cream', 0.14, 0.11, 0.14, (0.13, 0.191, 0.15), M.icecream, bevel=0.012, bevel_segments=1)
        Ff.box('peas', 0.13, 0.06, 0.2, (0.15, 0.166, -0.03), M.peas, bevel=0.015, bevel_segments=1)
    drawer = G.join(parts, 'drawer', origin=Ff.at(0, 0.38, 0.345))
    G.settle()
    G.swing(door_left, 'fridge_open', 100, seconds=DOOR_SECONDS)
    G.swing(door_right, 'fridge_open', -100, seconds=DOOR_SECONDS)
    G.swing(door_right, 'door_right', -100, seconds=DOOR_SECONDS)
    G.slide(drawer, 'freezer_open', (0, -0.35, 0), seconds=DOOR_SECONDS)


def _fridgeRetro():
    """A 1950s mint fridge: one rounded body, its top rolling over, on a plinth with a chrome kick grille; a single
    rounded door on chrome hinges with a trim line, a chrome nameplate and a chrome lever handle. Inside, an ice box
    with a flap, shelves with chrome rails, a crisper and food; on the door's inside, railed shelves with bottles."""
    Fr = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        Fr.box('plinth', 0.66, 0.08, 0.60, (0, 0.04, -0.05), M.mint2, bevel=0.010)
        _kick(Fr, 'kick', 0.56, 0.05, 0.042, 0.255, M.chrome, slots=7)
        # the body: rounded-square rings, full from the plinth up to where the top rolls over towards the back
        bx, by, _ = Fr.at(0, 0, -0.04)
        rings = [(0.08, 0.35, 0.31), (0.1, 0.36, 0.32), (1.54, 0.36, 0.32), (1.585, 0.354, 0.314), (1.615, 0.34, 0.3),
                 (1.633, 0.315, 0.275), (1.64, 0.27, 0.23)]
        shell = G.loft('shell', [G.ring(rx, rz, y, 32, square=7, at=(bx, by)) for y, rx, rz in rings], material=M.mint)
        # the inside cut out through the front, lined (the cut's faces take the liner)
        cavity = Fr.box('cavity', 0.62, 1.42, 0.62, (0, 0.85, -0.01), M.liner)
        b = shell.modifiers.new('cut', 'BOOLEAN')
        b.operation, b.object, b.solver, b.material_mode = 'DIFFERENCE', cavity, 'EXACT', 'TRANSFER'
        _soft(shell)
        bpy.data.objects.remove(cavity)
        # chrome hinges on the door's hinge side
        for y in (0.2, 1.47):
            Fr.cyl(f'hinge{y}', 0.011, 0.07, (-0.348, y, 0.30), M.chrome, verts=10)
        # the ice box across the top with its flap and latch, its drip tray, two shelves with chrome rails, a crisper
        Fr.box('icebox', 0.5, 0.2, 0.4, (0, 1.42, -0.11), M.liner, bevel=0.01, bevel_segments=1)
        Fr.soft('icebox_flap', 0.48, 0.18, 0.014, (0, 1.42, 0.097), M.shelf, round=0.006, round_segments=1)
        Fr.box('icebox_latch', 0.06, 0.02, 0.016, (0, 1.35, 0.108), M.chrome, bevel=0.006, bevel_segments=1)
        Fr.box('drip_tray', 0.56, 0.02, 0.4, (0, 1.29, -0.11), M.shelf, bevel=0.004, bevel_segments=1)
        for i, y in enumerate((0.45, 0.80)):
            Fr.box(f'shelf{i}', 0.6, 0.008, 0.4, (0, y, -0.11), M.shelf)
            Fr.sweep(f'shelf_rail{i}', [(-0.3, y + 0.004, 0.09), (0.3, y + 0.004, 0.09)], 0.006, M.chrome, verts=6)
        Fr.box('crisper', 0.56, 0.2, 0.4, (0, 0.25, -0.11), M.shelf, bevel=0.008, bevel_segments=1)
        Fr.extrude('crisper_front', G.rounded_rect(0.56, 0.21, 0.014, corner=1), 0.012, (0, 0.25, 0.096), M.carton,
                   holes=[[(u, v + 0.07) for u, v in G.rounded_rect(0.16, 0.024, 0.012, corner=2)]])
        # glass milk bottles, a pie and apples, butter
        for i, x in enumerate((-0.2, -0.12)):
            _bottle(f'milk{i}', (x, 0.804, -0.16), 0.034, 0.22, M.carton, M.trim, frame=Fr)
        Fr.cyl('pie_dish', 0.11, 0.03, (0.12, 0.47, -0.12), M.porcelain, verts=16, bevel=0.006, bevel_segments=1)
        Fr.cyl('pie', 0.1, 0.025, (0.12, 0.49, -0.12), M.board, verts=16, bevel=0.01, bevel_segments=2)
        for i, (x, z) in enumerate(((-0.2, -0.05), (-0.12, -0.08), (0.02, -0.06))):
            _apple(Fr, f'apple{i}', x, 0.804, z + 0.02 * i, 0.036)
        Fr.box('butter_dish', 0.14, 0.03, 0.09, (0.14, 0.82, -0.08), M.porcelain, bevel=0.01, bevel_segments=1)
        Fr.box('butter', 0.1, 0.04, 0.06, (0.14, 0.85, -0.08), M.egg, bevel=0.006, bevel_segments=1)
    G.join(still, 'body', origin=(0, 0, 0))
    # the rounded door, hinged on its left edge: a trim line, a chrome nameplate, the lever's escutcheon; on its
    # inside, a seal, a liner and two shelves with chrome guard rails holding bottles and eggs
    with G.new_parts() as parts:
        door = Fr.extrude('door', G.rounded_rect(0.68, 1.50, 0.075, corner=4), 0.04, (0, 0.84, 0.30), M.mint,
                          bevel=0.014, bevel_segments=3)
        _soft(door)
        Fr.sweep('seam', [(-0.33, 1.18, 0.318), (0.33, 1.18, 0.318)], 0.006, M.mint2, verts=6)
        Fr.extrude('nameplate', G.rounded_rect(0.2, 0.05, 0.025, corner=4), 0.006, (0, 1.46, 0.321), M.chrome,
                   bevel=0.002, bevel_segments=1)
        Fr.extrude('nameplate_inlay', G.rounded_rect(0.17, 0.028, 0.014, corner=3), 0.004, (0, 1.46, 0.324), M.mint2)
        Fr.soft('escutcheon', 0.07, 0.09, 0.01, (0.27, 1.11, 0.322), M.chrome, round=0.005, round_segments=2)
        _gasket(Fr, 'door_seal', 0.64, 1.46, 0.84, 0.276, r=0.008, corner=0.06)
        Fr.box('door_liner', 0.58, 1.36, 0.008, (0, 0.84, 0.278), M.liner)
        for i, y in enumerate((0.38, 0.86)):
            Fr.box(f'door_shelf{i}', 0.52, 0.012, 0.08, (0, y, 0.234), M.shelf, bevel=0.004, bevel_segments=1)
            Fr.sweep(f'door_rail{i}', [(-0.26, y, 0.27), (-0.26, y + 0.07, 0.205), (0.26, y + 0.07, 0.205),
                                       (0.26, y, 0.27)], 0.005, M.chrome, verts=6, resolution=2)
        for i, (x, mat) in enumerate(((-0.2, M.bottle), (-0.12, M.wine2), (-0.04, M.produce2))):
            _bottle(f'door_bottle{i}', (x, 0.386, 0.234), 0.03, 0.26 - 0.03 * i, mat, M.carton, frame=Fr)
        for i in range(5):
            Fr.sphere(f'egg{i}', 0.02, (-0.16 + 0.07 * i, 0.886, 0.234), M.egg, segments=8, rings=5, scale=(1, 1, 1.25))
    door = G.hinged(G.join(parts, 'door'), Fr.at(-0.36, 0.84, 0.30))
    # the chrome lever on the door, pivoting on its top: pulled out first, it unlatches the door
    with G.new_parts() as parts:
        Fr.sweep('handle', [(0.27, 1.14, 0.328), (0.27, 1.07, 0.348), (0.27, 0.95, 0.353), (0.27, 0.89, 0.348)], None,
                 M.chrome, profile=G.rounded_rect(0.048, 0.02, 0.009, corner=2), up=(1, 0, 0), resolution=3,
                 scale=[1.0, 0.85, 0.9, 1.05], angle=60)
    lever = G.hinged(G.join(parts, 'lever'), Fr.at(0.27, 1.12, 0.33))
    G.settle()
    G.attach(lever, door)
    G.swing(lever, 'lever', -30, seconds=LEVER_SECONDS, axis='X')   # its foot out to the front
    G.swing(door, 'fridge_open', 100, seconds=DOOR_SECONDS)


# The retro mini fridge's die-cut unicorn sticker, drawn in the reference photo's pixels (1024² image,
# the sticker's middle at (652, 515)), each layer a thin flat plate a little prouder than the last.
STICKER_PX = (652, 515)
STICKER_SCALE = (0.0012, 0.001)   # metres per pixel across (the photo sees the door at an angle) and up


def _chaikin(pts, closed=True, passes=1):
    """Rounds a polygon (or an open line) by cutting its corners."""
    for _ in range(passes):
        out = []
        n = len(pts)
        for i in range(n if closed else n - 1):
            (ax, ay), (bx, by) = pts[i], pts[(i + 1) % n]
            out += [(0.75 * ax + 0.25 * bx, 0.75 * ay + 0.25 * by), (0.25 * ax + 0.75 * bx, 0.25 * ay + 0.75 * by)]
        pts = out if closed else [pts[0]] + out + [pts[-1]]
    return pts


def _ribbon(line, width):
    """A tapered strand round a centre line (pixels): its outline, for `_plate`."""
    line = _chaikin(line, closed=False, passes=1)
    left, right = [], []
    n = len(line)
    for i, (x, y) in enumerate(line):
        (ax, ay), (bx, by) = line[max(i - 1, 0)], line[min(i + 1, n - 1)]
        dx, dy = bx - ax, by - ay
        k = math.hypot(dx, dy) or 1.0
        w = width / 2 * (0.25 + 0.75 * math.sin(math.pi * i / (n - 1)))
        left.append((x - dy / k * w, y + dx / k * w))
        right.append((x + dy / k * w, y - dx / k * w))
    return left + right[::-1]


def _oval(cx, cy, rx, ry, n=12):
    return [(cx + rx * math.cos(2 * math.pi * k / n), cy + ry * math.sin(2 * math.pi * k / n)) for k in range(n)]


def _plate(F, name, px, at, layer, material, round_passes=1):
    """A flat plate of the sticker, its outline in photo pixels, `layer` × 1.2 mm proud of the door face
    `at` (game x, height, depth of the sticker's middle). Front and edges only: its back is on the door."""
    if round_passes:
        px = _chaikin(px, passes=round_passes)
    (ox, oy), (su, sv) = STICKER_PX, STICKER_SCALE
    x0, y0, z0 = at
    z = z0 + 0.0008 + 0.0012 * layer   # the border's back just into the door, each layer 1.2 mm on
    pts = [(x0 - (u - ox) * su, y0 - (v - oy) * sv) for u, v in px]   # the viewer's right is game -x
    n = len(pts)
    verts = [F.at(x, y, z - 0.0016) for x, y in pts] + [F.at(x, y, z) for x, y in pts]
    bm = bmesh.new()
    vs = [bm.verts.new(v) for v in verts]
    front = bm.faces.new(vs[n:])
    for i in range(n):
        j = (i + 1) % n
        bm.faces.new((vs[i], vs[j], vs[n + j], vs[n + i]))
    bm.normal_update()
    if front.normal.y > 0:   # face the front (-y), the edges outward with it
        for f in bm.faces:
            f.normal_flip()
    mesh = bpy.data.meshes.new(name)
    bm.to_mesh(mesh)
    bm.free()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    return obj


def _unicorn(F, at):
    """The unicorn sticker on the mini fridge's door, as in the photo: a white die-cut border round a
    unicorn's head in profile, facing the viewer's right, its pink mane streaked blue and lilac."""
    _plate(F, 'uni_border', [
        (748, 360), (738, 400), (730, 426), (744, 450), (754, 480), (762, 508), (770, 530), (766, 554),
        (748, 568), (726, 570), (710, 566), (704, 582), (708, 610), (716, 648), (692, 666), (650, 674),
        (610, 674), (582, 662), (558, 632), (544, 595), (540, 555), (546, 514), (558, 476), (576, 446),
        (598, 420), (622, 400), (646, 386), (662, 396), (678, 392), (692, 410), (706, 418), (726, 384)],
        at, 0, M.uni_white)
    # the mane behind the head, and the head and neck
    _plate(F, 'uni_mane', [
        (652, 402), (668, 412), (680, 428), (668, 452), (656, 490), (648, 535), (646, 580), (652, 625),
        (664, 656), (640, 664), (612, 664), (588, 652), (566, 624), (553, 590), (550, 555), (555, 518),
        (566, 484), (584, 455), (604, 431), (628, 413)], at, 1, M.uni_pink)
    _plate(F, 'uni_head', [
        (700, 428), (726, 436), (740, 458), (749, 486), (756, 512), (761, 532), (756, 548), (742, 558),
        (724, 559), (710, 553), (697, 561), (691, 583), (696, 613), (704, 648), (680, 657), (664, 656),
        (652, 625), (646, 580), (648, 535), (656, 490), (668, 452), (682, 432)], at, 1, M.uni_face)
    # streaks through the mane
    for i, (line, w, mat) in enumerate((
            ([(640, 420), (610, 452), (590, 492), (582, 540), (588, 590), (606, 634)], 9, M.uni_blue),
            ([(660, 432), (636, 472), (621, 520), (619, 570), (628, 618), (642, 652)], 8, M.uni_lilac),
            ([(624, 420), (598, 456), (576, 506), (571, 560), (582, 612)], 5, M.uni_white),
            ([(667, 458), (650, 502), (640, 552), (641, 604), (651, 642)], 6, M.uni_rose))):
        _plate(F, f'uni_streak{i}', _ribbon(line, w), at, 2, mat, round_passes=0)
    # the ear, the horn with its spiral, the muzzle, nostril and eye
    _plate(F, 'uni_ear', [(650, 392), (664, 402), (678, 398), (686, 424), (670, 434), (657, 418)], at, 2, M.uni_face)
    _plate(F, 'uni_ear_in', [(658, 402), (667, 410), (674, 425), (664, 422)], at, 3, M.uni_pink)
    base_l, base_r, tip = (708, 424), (731, 431), (746, 366)
    _plate(F, 'uni_horn', [base_l, tip, base_r], at, 2, M.uni_white, round_passes=0)
    lerp = lambda a, b, t: (a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t)  # noqa: E731
    for i, t in enumerate((0.12, 0.36, 0.60)):
        _plate(F, f'uni_horn_stripe{i}', [lerp(base_l, tip, t), lerp(base_l, tip, t + 0.07),
                                          lerp(base_r, tip, t + 0.15), lerp(base_r, tip, t + 0.08)],
               at, 3, M.uni_lilac, round_passes=0)
    _plate(F, 'uni_muzzle', _oval(744, 540, 15, 13), at, 2, M.uni_blush, round_passes=0)
    _plate(F, 'uni_nostril', _oval(752, 533, 3.5, 2.5, 8), at, 3, M.uni_rose, round_passes=0)
    _plate(F, 'uni_eye', [(695, 481), (703, 474), (714, 475), (719, 483), (709, 489), (699, 487)], at, 2,
           M.uni_dark, round_passes=0)
    _plate(F, 'uni_eye_lid', _ribbon([(692, 476), (704, 469), (717, 472)], 3), at, 3, M.uni_rose, round_passes=0)


def _soft(obj, bevel_first=True):
    """Glossy round shading for a bevelled shell: smooth over the bevels, flat on the broad faces."""
    bpy.ops.object.select_all(action='DESELECT')
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    for m in list(obj.modifiers):
        bpy.ops.object.modifier_apply(modifier=m.name)
    bpy.ops.object.shade_smooth_by_angle(angle=math.radians(40))
    wn = obj.modifiers.new('weighted', 'WEIGHTED_NORMAL')
    wn.keep_sharp = True
    return obj


def _letters(F, text, at, size, material):
    """Raised metal letters on a face at game depth `at[2]`, centred on (x, height), facing the front."""
    x, y, z = at
    bpy.ops.object.text_add(location=F.at(x, y, z + 0.0015), rotation=(math.radians(90), 0, 0))
    obj = bpy.context.object
    obj.data.body = text
    obj.data.size = size
    obj.data.extrude = 0.0015
    obj.data.space_character = 1.25
    obj.data.align_x = 'CENTER'
    obj.data.align_y = 'CENTER'
    obj.data.resolution_u = 3
    bpy.ops.object.select_all(action='DESELECT')
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj
    bpy.ops.object.convert(target='MESH')
    obj = bpy.context.object
    obj.name = obj.data.name = 'letters'
    bm = bmesh.new()   # the converted text's faces point every way: make them outward
    bm.from_mesh(obj.data)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(obj.data)
    bm.free()
    obj.data.materials.clear()
    obj.data.materials.append(material)
    return obj


def _fridgeMiniRetro():
    """A pastel-pink retro mini fridge, as in the reference photo: a soft rounded cabinet on tapered brass
    feet, vents at the back of its sides, and one door over the whole front with a brass D handle, raised
    gold letters ("MARIE") and a die-cut unicorn sticker."""
    Fp = G.GameFrame(shift=0.0)
    front = 0.115   # the door's face; the handle reaches the old part's front (0.15)
    with G.new_parts() as still:
        # the cabinet: one rounded shell, its inside cut out through the front and lined
        cab = Fp.box('cabinet', 0.57, 0.81, 0.50, (0, 0.455, -0.18), M.pink, bevel=0.05, bevel_segments=4)
        cut = Fp.box('cavity', 0.47, 0.66, 0.51, (0, 0.46, -0.135), M.liner, bevel=0.015)
        bpy.ops.object.select_all(action='DESELECT')
        cut.select_set(True)
        bpy.context.view_layer.objects.active = cut
        bpy.ops.object.modifier_apply(modifier='bevel')
        b = cab.modifiers.new('cut', 'BOOLEAN')
        b.operation, b.object, b.solver, b.material_mode = 'DIFFERENCE', cut, 'EXACT', 'TRANSFER'
        _soft(cab)
        bpy.data.objects.remove(cut)
        # a shelf and the freezer floor; food, seen once the door opens
        Fp.box('divider', 0.47, 0.014, 0.44, (0, 0.62, -0.16), M.shelf)
        Fp.box('shelf', 0.47, 0.008, 0.44, (0, 0.36, -0.16), M.shelf)
        Fp.box('shelf_trim', 0.47, 0.016, 0.008, (0, 0.358, 0.058), M.pink2, bevel=0.003, bevel_segments=1)
        _carton(Fp, 'milk', -0.12, 0.364, -0.20, 0.07, 0.13, M.carton)
        for i, x in enumerate((0.05, 0.13)):
            _apple(Fp, f'apple{i}', x, 0.364, -0.17 + 0.03 * i, 0.032)
        _bottle('juice', (0.08, 0.135, -0.2), 0.026, 0.15, M.bottle, M.carton, frame=Fp)
        Fp.box('icecream', 0.12, 0.07, 0.10, (-0.05, 0.662, -0.18), M.icecream, bevel=0.01)
        # tapered brass feet, wide at the top
        for x in (-0.215, 0.215):
            for z in (-0.37, 0.02):
                Fp.cone('foot', 0.019, 0.027, 0.05, (x, 0.025, z), M.gold, verts=12)
        for side in (-1, 1):
            x = side * 0.285
            # the compressor panel's seam low on each side
            Fp.box('side_seam', 0.003, 0.006, 0.39, (x, 0.165, -0.18), M.pink2, bevel=0.0)
            # two groups of seven vent slots at the back of each side
            for y0 in (0.20, 0.61):
                for i in range(7):
                    Fp.box('vent', 0.004, 0.006, 0.07, (x, y0 + i * 0.014, -0.33), M.vent, bevel=0.0)
    G.join(still, 'body', origin=(0, 0, 0))
    # the door over the whole front, hinged on the viewer's right (its handle on the left)
    with G.new_parts() as parts:
        door = Fp.box('door', 0.555, 0.785, 0.045, (0, 0.4525, front - 0.0225), M.pink, bevel=0.035,
                      bevel_segments=3)
        _soft(door)
        Fp.box('door_liner', 0.45, 0.64, 0.008, (0, 0.46, 0.066), M.liner)
        _gasket(Fp, 'door_seal', 0.5, 0.72, 0.4525, 0.071, r=0.006, corner=0.04)
        _door_bin('door_bin', 0.27, -0.2, 0.2, 0.022, 0.064, frame=Fp)
        Fp.cyl('can', 0.03, 0.11, (0.12, 0.335, 0.043), M.produce, verts=10)
        _jar(Fp, 'door_jar', -0.1, 0.28, 0.043, 0.026, 0.08, M.egg, M.trim, verts=10)
        # the brass D handle in one piece: a bar on two curved arms into the door, a round pad under each
        hx, y0, y1, out = 0.17, 0.35, 0.60, front + 0.025
        pts = [(hx, y0 - 0.02, front), (hx, y0, out - 0.006), (hx, y0 + 0.025, out),
               (hx, y1 - 0.025, out), (hx, y1, out - 0.006), (hx, y1 + 0.02, front)]
        G.kit.tube(Fp, 'handle', pts, 0.0095, M.gold, verts=8, round=3)
        for i, y in enumerate((y0 - 0.02, y1 + 0.02)):
            Fp.cyl(f'handle_pad{i}', 0.016, 0.006, (hx, y, front + 0.002), M.gold, verts=12, rot=(90, 0, 0))
        _letters(Fp, 'MARIE', (-0.04, 0.705, front), 0.064, M.gold)
        _unicorn(Fp, (-0.06, 0.44, front))
    door = G.hinged(G.join(parts, 'door'), Fp.at(-0.2775, 0.4525, front - 0.0225))
    G.settle()
    G.swing(door, 'fridge_open', 100, seconds=1.6)


def _wineFridge():
    """A wine cooler under a stone worktop: a smoked-glass door in a dark frame over five shelves of bottles lying
    necks out on wood-fronted racks, a stack of three drawers beside it, a vented kick plate; once the door is open,
    the top shelf of bottles slides out to the front. Drawn in the counters' 0.66 m depth (its worktop -0.43..0.23)."""
    Fw = G.GameFrame(shift=0.0)
    with G.new_parts() as still:
        Fw.box('plinth', 0.996, 0.10, 0.55, (0, 0.05, -0.155), M.dark, bevel=0.004)
        _kick(Fw, 'kick', 0.9, 0.06, 0.06, 0.125, M.seal, slots=9)
        for s in (-1, 1):
            Fw.box(f'side{s}', 0.03, 0.80, 0.60, (s * 0.483, 0.50, -0.13), M.panel, bevel=0.004)
        Fw.box('bottom', 0.94, 0.03, 0.60, (0, 0.115, -0.13), M.panel)
        Fw.box('back', 0.94, 0.80, 0.03, (0, 0.50, -0.415), M.panel)
        Fw.box('partition', 0.02, 0.76, 0.58, (0.07, 0.50, -0.12), M.panel)
        Fw.box('ceiling', 0.54, 0.02, 0.58, (-0.2, 0.89, -0.12), M.panel)   # closes the gap over the door
        Fw.box('top', 1.00, 0.05, 0.66, (0, 0.925, -0.10), M.stone, bevel=0.012, bevel_segments=2)
        # the cellar: a pale liner, four racks of bottles (the top one slides out: below)
        Fw.box('liner', 0.52, 0.74, 0.008, (-0.2, 0.50, -0.395), M.liner)
        Fw.box('liner_side', 0.008, 0.74, 0.56, (-0.463, 0.50, -0.12), M.liner)
        for r, y in enumerate(WINE_ROWS[:-1]):
            _wine_row(Fw, f'rack_left{r}', y, r)
        # three drawers beside it, flush with the door, each with a long pull
        for r in range(3):
            y = 0.2445 + 0.255 * r
            Fw.box(f'rack_right{r}', 0.404, 0.249, 0.02, (0.276, y, 0.19), M.panel, bevel=0.005, bevel_segments=1)
            _pull(Fw, f'rack_handle{r}', (0.18, y + 0.07), (0.372, y + 0.07), 0.2, out=0.026, w=0.014, r=None, verts=6)
    G.join(still, 'body', origin=(0, 0, 0))
    # the door, hinged on its left edge: a dark frame round a pane of smoked glass the bottles show through, a small
    # display on its top rail, a bar handle on its free edge and a seal round its back
    with G.new_parts() as parts:
        Fw.extrude('frame', G.rounded_rect(0.545, 0.76, 0.012, corner=2), 0.02, (-0.2075, 0.50, 0.19), M.doorframe,
                   holes=[G.rounded_rect(0.455, 0.64, 0.008, corner=2)], bevel=0.003, bevel_segments=1)
        Fw.box('glass', 0.463, 0.648, 0.006, (-0.2075, 0.50, 0.188), M.smoked)
        Fw.box('display', 0.08, 0.018, 0.004, (-0.2, 0.85, 0.201), M.water)
        _pull(Fw, 'door_handle', (0.035, 0.30), (0.035, 0.70), 0.2, out=0.03, w=0.016, r=None, verts=6)
        _gasket(Fw, 'door_seal', 0.52, 0.73, 0.50, 0.178, r=0.005, corner=0.02, x=-0.2075)
    door = G.hinged(G.join(parts, 'door'), Fw.at(-0.49, 0.50, 0.19))
    # the top shelf of bottles, sliding out 15 cm once the door is open
    with G.new_parts() as parts:
        _wine_row(Fw, 'wine_rack', WINE_ROWS[-1], len(WINE_ROWS) - 1)
    rack = G.join(parts, 'wine_rack', origin=Fw.at(-0.22, WINE_ROWS[-1], -0.14))
    G.settle()
    G.swing(door, 'door_open', 100, seconds=1.5)
    G.slide(rack, 'rack_out', (0, -0.15, 0), seconds=0.8)


WINE_ROWS = (0.22, 0.35, 0.48, 0.61, 0.74)    # the wine fridge's shelves


def _wine_bottle(Fw, name, x, y, z, material, foil):
    """A wine bottle lying on its side at game (x, y: its lowest point, z: its base), its neck to the front: a body,
    a shoulder, a neck and a foil capsule."""
    r, h = 0.036, 0.30
    rings = [(0.0, 0.94), (0.6, 1.0), (0.72, 0.78), (0.8, 0.4), (0.86, 0.36), (0.861, 0.4), (1.0, 0.4)]
    G.loft(name, [G.ring(r * k, r * k, h * t, 8) for t, k in rings[:5]], material=material, at=Fw.at(x, y + r, z),
           rot=(90, 0, 0))
    G.loft(f'{name}_foil', [G.ring(r * k, r * k, h * t, 8) for t, k in rings[5:]], material=foil,
           at=Fw.at(x, y + r, z), rot=(90, 0, 0))


def _wine_row(Fw, name, y, r):
    """A shelf of the wine fridge, its front a wooden lip, with three bottles lying on it, necks to the front."""
    Fw.box(name, 0.47, 0.01, 0.50, (-0.215, y, -0.14), M.shelf)
    Fw.box(f'{name}_lip', 0.45, 0.03, 0.016, (-0.215, y + 0.008, 0.09), M.board, bevel=0.004, bevel_segments=1)
    for c, x in enumerate((-0.38, -0.215, -0.05)):
        red = (r + c) % 2 == 0
        _wine_bottle(Fw, f'{name}_bottle{c}', x, y + 0.005, -0.27, M.wine if red else M.wine2,
                     M.produce if red else M.gold)


# The fridges with a freezer: food, a meal's ingredients and a drink from the fridge door, ice cream from the freezer.
FRIDGE_USES = {'frozenTreat': 'freezer_open', 'fetch': 'fridge_open', 'cook': 'fridge_open', 'drink': 'fridge_open'}

# A floor thing against a wall is centred where the game places it (`centre`): its back on the wall,
# its top under its content `surface`.
FAMILY = G.Family(
    'kitchen', M,
    [G.Model('fridge', fridge, size=(0.74, 1.8, 0.722), key='model.fridge', centre=(0, -0.069),
             # Taking food or a drink out opens the fridge door (snacks, a meal's ingredients (cook), a cold
             # drink, putting dishes away); ice cream (`frozenTreat`) opens the freezer door.
             motions={'uses': FRIDGE_USES, 'hold': HOLD},
             # The player colours the cabinet and doors, the handles, the plinth and the inside; the
             # food keeps its own colours.
             paint=[('body', 'Body'), ('chrome', 'Handles'), ('trim', 'Plinth & badge'), ('liner', 'Inside')]),
     G.Model('fridgeMini', _fridgeMini, size=(0.60, 1.00, 0.60), key='model.fridgeMini', centre=(0, -0.128),
             motions={'uses': {'fetch': 'fridge_open', 'cook': 'fridge_open'}, 'hold': 3.0},
             paint=[('body', 'Body'), ('chrome', 'Handles'), ('trim', 'Plinth')]),
     G.Model('fridgeBuiltIn', _fridgeBuiltIn, size=(0.84, 2.025, 0.656), key='model.fridgeBuiltIn', centre=(0, -0.099),
             # the tall door serves every use but ice cream, which opens the freezer door
             motions={'uses': FRIDGE_USES, 'hold': HOLD},
             paint=[('panel', 'Panels'), ('chrome', 'Handles')]),
     G.Model('fridgeFrenchDoor', _fridgeFrenchDoor, size=(0.90, 1.82, 0.76), key='model.fridgeFrenchDoor', centre=(0, -0.052),
             # both doors for a snack or a meal; the right door alone for a cold drink
             motions={'uses': {**FRIDGE_USES, 'drink': 'door_right'}, 'hold': HOLD},
             paint=[('steel', 'Body'), ('chrome', 'Handles'), ('trim', 'Plinth')]),
     G.Model('fridgeRetro', _fridgeRetro, size=(0.72, 1.64, 0.715), key='model.fridgeRetro', centre=(0, -0.067),
             motions={'uses': {use: RETRO_OPEN for use in ('fetch', 'cook', 'drink')}, 'hold': HOLD},
             paint=[('mint', 'Body'), ('chrome', 'Handle'), ('mint2', 'Trim')]),
     G.Model('fridgeMiniRetro', _fridgeMiniRetro, size=(0.58, 0.86, 0.58), key='model.fridgeMiniRetro',
             centre=(0, -0.14),
             motions={'uses': {use: 'fridge_open' for use in ('fetch', 'cook', 'drink')}, 'hold': 3.0},
             paint=[('pink', 'Body'), ('gold', 'Handle & feet'), ('pink2', 'Trim')]),
     G.Model('wineFridge', _wineFridge, size=(1.00, 0.95, 0.66), key='model.wineFridge', centre=(0, -0.10),
             # pouring a glass opens the door, then slides the top shelf of bottles out (back in before it shuts)
             motions={'uses': {'drink': ['door_open', {'clip': 'rack_out', 'delay': 0.9, 'hold': 0.9}]}, 'hold': 2.5},
             paint=[('panel', 'Body'), ('doorframe', 'Door frame')]),
     G.Model('farmhouseSink', farmhouseSink, size=(1.0, 1.253, 0.67), key='model.farmhouseSink', centre=(0, -0.095),
             motions={'uses': {'wash': 'water_on'}},
             paint=[('body', 'Cabinet'), ('wood', 'Worktop'), ('porcelain', 'Basin'), ('chrome', 'Tap & handles'),
                    ('trim', 'Plinth')]),
     G.Model('counterSinkDouble', counterSinkDouble, size=(2.0, 1.265, 0.66), key='model.counterSinkDouble', centre=(0, -0.10),
             motions={'uses': {'wash': [{'clip': f'water_on_{slot}', 'slot': slot} for slot in (0, 1)]}},
             paint=[('body', 'Cabinets'), ('stone', 'Worktop'), ('metal', 'Basins'), ('chrome', 'Tap & handles'),
                    ('trim', 'Plinth')])],
    comment='Blender-built kitchen models (tools/art/blender/families/kitchen.py). Listed in the base '
            'manifest\'s packs. The materials map onto the game\'s finishes so a model costs one draw call '
            'per finish group. Each sink has a `water_on` clip (the stream from the tap) played while a '
            'resident washes up. The fridges (K1) each hinge a door built closed and swing it open while '
            'a resident takes food or a drink (`motions`); the French-door freezer slides and the wine '
            'fridge\'s top shelf of bottles slides out once its glass door is open; every freezer door and '
            'the French freezer drawer open for \'Get some ice cream\'.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

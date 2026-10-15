# The town's food stalls and vending machines (CM3): the ice cream cart (`model.iceCreamCart`), the hot dog
# stand (`model.hotDogStand`), the snack machine (`model.vendingSnack`), the drinks machine
# (`model.vendingDrink`) and the market stall (`model.marketStall`), over their primitive-part placeholders
# (`tools/art/community.py`, "Carts and machines"), which stay the fallback. Notes: stalls.md.
#
# Each is built where its placeholder stands (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`): the carts and the stall across their 2 x 1 tiles, the machines against the back of their tile.
# The moving parts: the cart's freezer lid slides across (`lid_open`, held 2 s), the stand's hot-plate lid
# lifts (`lid_open`), a snack slides down to the machine's tray and a can drops to it (`drop`). The
# machines' lit signs (and the drinks machine's front) glow at night. They are placed on town lots and seen
# from far, so each has a far version (`far=0.25`).
#
# Run:  python3 -B tools/art/blender/build.py stalls [--only iceCreamCart] [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    # the ice cream cart
    cart=('#F4EFE6', 'gloss'),          # its cream body; the main part the player colours
    stripe=('#E9A6B4', 'gloss'),        # the pink bands round it
    parasol=('#E9A6B4', 'fabric'),      # the parasol's coloured gores and valance
    parasol_white=('#F6F3EE', 'fabric'),  # its white gores
    cream_pink=('#F3B6C4', 'satin'),    # the ice creams in their tubs
    cream_vanilla=('#F3E6C4', 'satin'),
    cream_choc=('#7A4E33', 'satin'),
    cream_mint=('#BFE3C8', 'satin'),
    wafer=('#D9A35F', 'matte'),         # the cone on the sign
    # the hot dog stand
    stand=('#B8322B', 'gloss'),         # its red body
    awning=('#B8322B', 'fabric'),       # the awning's red stripes
    awning_white=('#F2F0EA', 'fabric'),  # and its white ones
    sign=('#F2F0EA', 'gloss'),          # the sign on its front
    bun=('#D9A35F', 'satin'),
    sausage=('#9E3B2A', 'gloss'),
    mustard=('#D6A64A', 'gloss'),
    ketchup=('#C0392B', 'gloss'),
    grill=('#2A2F36', 'satin'),         # the hot plate
    # the machines
    snack_red=('#B8322B', 'gloss'),     # the snack machine's cabinet
    drink_blue=('#2E6FA8', 'gloss'),    # the drinks machine's cabinet
    lit_sign=('#F2E6C2', 'gloss', {'glow': '#FFE2A6'}),   # the header sign over the window, lit at night
    lit_panel=('#EAF4F8', 'gloss', {'glow': '#CFE8FF'}),  # the drinks machine's lit front
    band=('#2E5C8A', 'gloss'),          # the bands on the lit front
    glass=('#CFE3EA', 'glass'),         # the snack machine's window (see-through: its own draw call)
    inside=('#2A2F36', 'satin'),        # the dark inside, the keypad panel, the tray
    tray=('#1E2226', 'satin'),          # the pick-up tray's dark well
    button=('#C3C7CB', 'satin'),        # keypad buttons, the card reader
    display=('#7FD1A8', 'gloss'),       # the price display
    label=('#F2F0EA', 'satin'),         # the shelf labels
    snack_a=('#F2C14E', 'gloss'),       # the snacks' wrappers and the cans
    snack_b=('#4E7A7F', 'gloss'),
    snack_c=('#D9483B', 'gloss'),
    snack_d=('#8C5A3C', 'gloss'),
    snack_e=('#9DAF8C', 'gloss'),
    snack_f=('#E9A6B4', 'gloss'),
    snack_g=('#3E5470', 'gloss'),
    can_green=('#5EC28B', 'gloss'),
    # the market stall
    timber=('#7A5234', 'wood'),         # its frame: posts, beams, apron; the main part
    board=('#A8754A', 'wood'),          # the table top
    crate=('#CDB08A', 'wood'),          # the fruit crates
    awning_green=('#4E8A5A', 'fabric'),  # the awning's green stripes
    tomato=('#D9483B', 'gloss'),
    orange=('#E8902E', 'gloss'),
    apple=('#8DB84A', 'gloss'),
    lemon=('#F2C14E', 'gloss'),
    slate=('#2A2C30', 'matte'),         # the chalk board
    chalk=('#E8E6E0', 'matte'),
    # shared
    steel=('#C3C7CB', 'metal'),         # tops, frames, push bars, poles
    tyre=('#26282A', 'matte'),
)

SNACKS = ['snack_a', 'snack_b', 'snack_c', 'snack_d', 'snack_e', 'snack_f', 'snack_g']


def _wheel(F, name, x, z, r):
    """A wheel at game (x, z), its axle along the depth: a tyre and a steel hub."""
    F.cyl(f'{name}_tyre', r, 0.06, (x, r, z), M.tyre, rot=(90, 0, 0), verts=18)
    F.cyl(f'{name}_hub', r * 0.45, 0.07, (x, r, z), M.steel, rot=(90, 0, 0), verts=12)


def _striped(obj, gores, first, second):
    """Gives a cone's side faces alternate materials round it (a parasol's gores); its cap the second."""
    obj.data.materials.append(second)
    for p in obj.data.polygons:
        if abs(p.normal.z) > 0.9 and p.normal.z < 0:
            p.material_index = 1
            continue
        a = math.atan2(p.center.y, p.center.x) % G.TAU
        p.material_index = int(a / (G.TAU / gores)) % 2
    return obj


# --- the ice cream cart: a cream freezer on wheels, a parasol over it ------------------------------------
CART_X = -0.15          # the body's middle across
TILT = 11.0             # the parasol leans forward this much, from its socket at the back of the top


def _pole_at(s):
    """A point `s` metres up the parasol's pole (game axes), from its socket at the back of the top."""
    return (CART_X, 0.97 + s * G.cosd(TILT), -0.2 + s * G.sind(TILT))


def iceCreamCart():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('cart', 1.3, 0.62, 0.62, (CART_X, 0.62, 0.0), M.cart, bevel=0.03)
        for y in (0.5, 0.74):
            F.box(f'band{y}', 1.31, 0.1, 0.63, (CART_X, y, 0.0), M.stripe, bevel=0.01)
        F.box('chassis', 1.16, 0.04, 0.46, (CART_X, 0.3, 0.0), M.steel)
        for x in (-0.62, 0.38):
            for z in (-0.27, 0.27):
                _wheel(F, f'wheel{x}{z}', x, z, 0.17)
        # the push bar
        for z in (-0.22, 0.22):
            G.rod(f'bar{z}', F.at(0.5, 0.86, z), F.at(0.82, 1.0, z), 0.015, material=M.steel, verts=10)
        G.rod('handle', F.at(0.82, 1.0, -0.23), F.at(0.82, 1.0, 0.23), 0.0175, material=M.steel, verts=10)
        G.rod('grip', F.at(0.82, 1.0, -0.12), F.at(0.82, 1.0, 0.12), 0.022, material=M.tyre, verts=10)
        # the steel deck round the freezer well, the lids' rails, the fixed lid on the right
        F.box('deck_back', 1.3, 0.04, 0.18, (CART_X, 0.95, -0.22), M.steel, bevel=0.006)
        F.box('deck_front', 1.3, 0.04, 0.02, (CART_X, 0.95, 0.30), M.steel, bevel=0.006)
        for x in (-0.77, 0.47):
            F.box(f'deck_end{x}', 0.06, 0.04, 0.42, (x, 0.95, 0.08), M.steel, bevel=0.006)
        for z in (-0.115, 0.275):
            F.box(f'rail{z}', 1.18, 0.06, 0.03, (CART_X + 0.005, 0.96, z), M.steel, bevel=0.004)
        F.box('lid_fixed', 0.59, 0.02, 0.36, (0.145, 0.98, 0.08), M.steel, bevel=0.004)
        # the tubs in the well (seen with the lid open), sunk into the top
        creams = [M.cream_pink, M.cream_vanilla, M.cream_choc, M.cream_mint, M.cream_pink, M.cream_vanilla]
        for k, x in enumerate((-0.64, -0.445, -0.25, -0.05, 0.145, 0.34)):
            F.cyl(f'tub{k}', 0.085, 0.07, (x, 0.915, 0.08), M.steel, verts=16)
            G.sphere(f'cream{k}', 0.072, at=F.at(x, 0.945, 0.08), material=creams[k], scale=(1, 1, 0.3),
                     segments=12, rings=6)
        # the sign: a cone with a scoop on the back of the top
        F.cone('wafer', 0.006, 0.065, 0.2, (0.3, 1.07, -0.225), M.wafer, verts=10)
        F.sphere('scoop', 0.08, (0.3, 1.2, -0.225), M.cream_pink, segments=12, rings=8)
        # the parasol, leaning forward from a socket at the back of the top
        F.cyl('socket', 0.035, 0.08, (CART_X, 1.0, -0.2), M.steel, verts=12)
        G.rod('pole', F.at(*_pole_at(0.0)), F.at(*_pole_at(1.38)), 0.018, material=M.steel, verts=10)
        centre = F.at(*_pole_at(1.2))
        canopy = G.cone('canopy', 0.47, 0.0, 0.28, at=centre, material=M.parasol, scale=(1.5, 1, 1),
                        rot=(TILT, 0, 0), verts=16, smooth=False)
        _striped(canopy, 16, M.parasol, M.parasol_white)
        G.torus('valance', 0.47, 0.022, at=F.at(*_pole_at(1.06)), material=M.parasol, rot=(TILT, 0, 0),
                major_segments=32, minor_segments=6).scale = (1.5, 1, 1)
        G.sphere('finial', 0.03, at=F.at(*_pole_at(1.38)), material=M.steel, segments=10, rings=6)
    G.join(still, 'body', origin=(0, 0, 0))
    # the sliding lid over the left half of the well, on the rails; its grip at its left end
    with G.new_parts() as parts:
        F.box('lid', 0.59, 0.02, 0.42, (-0.445, 1.0, 0.08), M.steel, bevel=0.004)
        F.box('lid_grip', 0.03, 0.025, 0.26, (-0.70, 1.02, 0.08), M.tyre, bevel=0.006)
    lid = G.join(parts, 'lid')
    G.settle(centre=False)
    G.slide(lid, 'lid_open', (-0.58, 0, 0), seconds=1.0)   # to the right (game +x), over the fixed lid
    return G.meshes()


# --- the hot dog stand: a red cart with a hot plate, under a striped awning on four posts -----------------
def hotDogStand():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('stand', 1.5, 0.66, 0.66, (-0.1, 0.62, -0.05), M.stand, bevel=0.03)
        F.box('top', 1.56, 0.05, 0.72, (-0.1, 0.975, -0.05), M.steel, bevel=0.008)
        F.box('chassis', 1.4, 0.04, 0.5, (-0.1, 0.27, -0.05), M.steel)
        for x in (-0.68, 0.5):
            for z in (0.24, -0.34):
                _wheel(F, f'wheel{x}{z}', x, z, 0.2)
        # the sign on the front: a hot dog in its bun
        F.box('sign', 0.62, 0.22, 0.02, (-0.1, 0.72, 0.29), M.sign, bevel=0.008)
        F.box('sign_bun', 0.44, 0.08, 0.02, (-0.1, 0.71, 0.302), M.bun, bevel=0.025)
        F.box('sign_sausage', 0.48, 0.045, 0.02, (-0.1, 0.735, 0.31), M.sausage, bevel=0.018)
        F.box('sign_mustard', 0.3, 0.012, 0.01, (-0.1, 0.758, 0.32), M.mustard, bevel=0.004)
        # the hot plate: a steel tray, its dark plate, sausages and buns under the lid
        F.box('plate_tray', 0.6, 0.04, 0.4, (-0.35, 1.02, -0.1), M.steel, bevel=0.006)
        F.box('plate', 0.54, 0.008, 0.34, (-0.35, 1.042, -0.1), M.grill)
        for k in range(5):
            x = -0.55 + 0.08 * k
            F.cyl(f'sausage{k}', 0.018, 0.22, (x, 1.064, -0.12), M.sausage, rot=(90, 0, 0), verts=10)
        for k, x in enumerate((-0.6, -0.13)):
            F.box(f'bun{k}', 0.08, 0.05, 0.2, (x, 1.071, -0.12), M.bun, bevel=0.02)
        # sauces and a napkin box at the right
        for x, m in ((0.3, M.mustard), (0.42, M.ketchup)):
            F.cyl(f'bottle{x}', 0.04, 0.17, (x, 1.085, 0.1), m, verts=12)
            F.cone(f'nozzle{x}', 0.02, 0.006, 0.05, (x, 1.195, 0.1), m, verts=8)
        F.box('napkins', 0.14, 0.08, 0.1, (0.36, 1.04, -0.15), M.sign, bevel=0.006)
        # the posts and the awning: six stripes leaning down to the front, a valance along it
        for x in (-0.82, 0.62):
            for z, top in ((-0.4, 2.35), (0.38, 2.23)):
                G.rod(f'post{x}{z}', F.at(x, 1.0, z), F.at(x, top, z), 0.02, material=M.steel, verts=10)
        for i in range(6):
            x = -0.85 + 0.25 * i + 0.125
            m = M.awning if i % 2 else M.awning_white
            F.box(f'stripe{i}', 0.25, 0.04, 0.98, (x, 2.3, 0.0), m, rot=(8, 0, 0), bevel=0.004)
            F.box(f'flap{i}', 0.25, 0.1, 0.012, (x, 2.2, 0.488), m, bevel=0.004)
    G.join(still, 'body', origin=(0, 0, 0))
    # the hot plate's lid, hinged on its back edge
    with G.new_parts() as parts:
        F.box('lid', 0.6, 0.1, 0.4, (-0.35, 1.09, -0.1), M.steel, bevel=0.03)
        F.box('lid_handle', 0.22, 0.025, 0.03, (-0.35, 1.1, 0.115), M.steel, bevel=0.008)
    lid = G.hinged(G.join(parts, 'lid'), F.at(-0.35, 1.04, -0.3))
    G.settle(centre=False)
    G.swing(lid, 'lid_open', -75, seconds=0.9, axis='X')
    return G.meshes()


# --- the vending machines: a cabinet against the back of the tile, a window or lit front, a tray below ----
BACK = -0.43       # the wall's face, game depth
FRONT = 0.29       # the cabinet's front face
WIN_X = (-0.41, 0.16)   # the window's sides
BAY_X = (-0.36, 0.06)   # the pick-up tray's opening
BAY_Y = (0.185, 0.34)   # its floor and top
WIN_Y = (0.44, 1.72)    # the window's bottom and top


def _cabinet(F, body):
    """The machine's still shell: back block, sides, header with its lit sign, the keypad column, the
    kick panel round the tray's opening, and the dark lining inside."""
    w = WIN_X[1] - WIN_X[0]
    cx = (WIN_X[0] + WIN_X[1]) / 2
    F.box('plinth', 0.86, 0.04, 0.68, (0, 0.02, -0.08), M.tray)
    F.box('back', 0.9, 1.84, 0.46, (0, 0.96, -0.2), body, bevel=0.02)
    for s in (-1, 1):
        F.box(f'side{s}', 0.04, 1.84, 0.27, (s * 0.43, 0.96, 0.155), body, bevel=0.012)
    F.box('header', 0.82, 0.16, 0.26, (0, 1.80, 0.16), body, bevel=0.01)
    F.box('header_sign', 0.72, 0.1, 0.01, (0, 1.80, FRONT + 0.005), M.lit_sign, bevel=0.003)
    # the keypad column on the right
    F.box('column', 0.25, 1.68, 0.26, (0.285, 0.88, 0.16), body, bevel=0.008)
    F.box('keypad', 0.18, 0.5, 0.02, (0.285, 1.2, FRONT + 0.01), M.inside, bevel=0.008)
    F.box('screen', 0.12, 0.07, 0.01, (0.285, 1.37, FRONT + 0.022), M.display)
    for r in range(4):
        for c in range(3):
            F.box(f'key{r}{c}', 0.035, 0.03, 0.01, (0.245 + 0.04 * c, 1.27 - 0.045 * r, FRONT + 0.022), M.button,
                  bevel=0.003)
    F.box('reader', 0.1, 0.1, 0.012, (0.285, 1.03, FRONT + 0.022), M.button, bevel=0.004)
    F.box('coin_return', 0.08, 0.06, 0.01, (0.285, 0.6, FRONT + 0.004), M.inside, bevel=0.004)
    # the kick panel round the tray's opening
    F.box('kick', w, BAY_Y[0] - 0.04, 0.26, (cx, (0.04 + BAY_Y[0]) / 2, 0.16), body)
    F.box('lip', w, WIN_Y[0] - BAY_Y[1], 0.025, (cx, (BAY_Y[1] + WIN_Y[0]) / 2, FRONT - 0.0125), body)
    for x0, x1 in ((WIN_X[0], BAY_X[0]), (BAY_X[1], WIN_X[1])):
        F.box(f'cheek{x0}', x1 - x0, BAY_Y[1] - BAY_Y[0], 0.26, ((x0 + x1) / 2, sum(BAY_Y) / 2, 0.16), body)
    # the dark inside: the back, the sides and the top behind the window, and the tray's well
    F.box('lining_back', w, WIN_Y[1] - BAY_Y[0], 0.01, (cx, (BAY_Y[0] + WIN_Y[1]) / 2, 0.035), M.inside)
    for x in (WIN_X[0] + 0.003, WIN_X[1] - 0.003):
        F.box(f'lining{x}', 0.006, WIN_Y[1] - BAY_Y[0], 0.25, (x, (BAY_Y[0] + WIN_Y[1]) / 2, 0.16), M.inside)
    F.box('lining_top', w, 0.006, 0.25, (cx, WIN_Y[1] - 0.003, 0.16), M.inside)
    F.box('tray_floor', w, 0.006, 0.25, (cx, BAY_Y[0] + 0.003, 0.16), M.tray)


def vendingSnack():
    F = G.GameFrame()
    cx = (WIN_X[0] + WIN_X[1]) / 2
    drop = None
    with G.new_parts() as still:
        _cabinet(F, M.snack_red)
        # five shelves of snacks, two deep, a label strip along each
        for row in range(5):
            yb = 0.47 + row * 0.25
            F.box(f'shelf{row}', WIN_X[1] - WIN_X[0], 0.015, 0.19, (cx, yb, 0.125), M.steel)
            F.box(f'labels{row}', WIN_X[1] - WIN_X[0], 0.025, 0.004, (cx, yb - 0.004, 0.222), M.label)
            for col in range(5):
                x = -0.355 + col * 0.115
                m = getattr(M, SNACKS[(row * 5 + col * 3) % len(SNACKS)])
                h = 0.14 - 0.02 * ((row + col) % 2)
                F.box(f'back{row}{col}', 0.09, h, 0.03, (x, yb + 0.0075 + h / 2, 0.12), m)
                if (row, col) == (1, 1):
                    drop = (x, yb + 0.0075 + h / 2, h, m)
                    continue
                F.box(f'snack{row}{col}', 0.09, h, 0.03, (x, yb + 0.0075 + h / 2, 0.195), m)
        F.box('window', WIN_X[1] - WIN_X[0], WIN_Y[1] - WIN_Y[0], 0.01, (cx, sum(WIN_Y) / 2, FRONT - 0.015), M.glass)
    G.join(still, 'body', origin=(0, 0, 0))
    # the snack that drops: the front one of the second shelf's second column
    x, y, h, m = drop
    with G.new_parts() as parts:
        F.box('snack', 0.09, h, 0.03, (x, y, 0.195), m)
    snack = G.join(parts, 'snack', origin=F.at(x, y, 0.195))
    G.settle(centre=False)
    fall = y - (BAY_Y[0] + 0.006 + h / 2)
    G.slide(snack, 'drop', (0, -0.05, 0), seconds=0.15)       # pushed off the shelf to the front
    G.slide(snack, 'drop', (0, 0, -fall), seconds=0.9)        # and down past the shelves into the tray
    G.swing(snack, 'drop', -15, seconds=0.9, axis='X')        # tipping back as it lands
    return G.meshes()


def vendingDrink():
    F = G.GameFrame()
    cx = (WIN_X[0] + WIN_X[1]) / 2
    with G.new_parts() as still:
        _cabinet(F, M.drink_blue)
        # the lit front, its bands, three big cans standing proud of it with a button under each
        F.box('panel', WIN_X[1] - WIN_X[0], WIN_Y[1] - WIN_Y[0], 0.012, (cx, sum(WIN_Y) / 2, FRONT - 0.006), M.lit_panel)
        F.box('band_top', WIN_X[1] - WIN_X[0], 0.12, 0.012, (cx, 1.6, FRONT + 0.004), M.band)
        F.box('band_low', WIN_X[1] - WIN_X[0], 0.07, 0.012, (cx, 0.7, FRONT + 0.004), M.band)
        for i, m in enumerate((M.snack_c, M.snack_a, M.can_green)):
            x = -0.315 + i * 0.19
            F.cyl(f'can{i}', 0.055, 0.24, (x, 1.2, FRONT + 0.01), m, verts=16)
            F.cyl(f'can_top{i}', 0.047, 0.01, (x, 1.325, FRONT + 0.01), M.steel, verts=16)
            F.box(f'button{i}', 0.1, 0.05, 0.016, (x, 0.95, FRONT + 0.006), M.button, bevel=0.006)
            F.box(f'price{i}', 0.08, 0.025, 0.006, (x, 1.03, FRONT + 0.003), M.display)
    G.join(still, 'body', origin=(0, 0, 0))
    # the can that drops: lying across, hidden inside above the tray until it falls into it
    x, y, z = -0.15, 0.4, 0.2
    with G.new_parts() as parts:
        G.cylinder('can', 0.033, 0.12, at=F.at(x, y, z), material=M.snack_c, rot=(0, 90, 0), verts=14)
        for s in (-1, 1):
            G.cylinder(f'can_end{s}', 0.028, 0.004, at=F.at(x + s * 0.061, y, z), material=M.steel,
                       rot=(0, 90, 0), verts=14)
    can = G.join(parts, 'can', origin=F.at(x, y, z))
    G.settle(centre=False)
    G.slide(can, 'drop', (0, 0, -(y - (BAY_Y[0] + 0.006 + 0.033))), seconds=0.7)   # down into the tray
    G.slide(can, 'drop', (0, -0.04, 0), seconds=0.9)                                 # rolling to its front
    return G.meshes()


# --- the market stall: a table of fruit crates under a striped awning on four posts ------------------------
def _crate(F, name, x, fruit):
    """An open crate on the table, tilted to the front, heaped with fruit; joined and turned whole."""
    with G.new_parts() as parts:
        y0, z0 = 0.84, 0.05
        F.box(f'{name}_floor', 0.36, 0.012, 0.5, (x, y0, z0), M.crate)
        for s in (-1, 1):
            F.box(f'{name}_side{s}', 0.012, 0.13, 0.5, (x + s * 0.174, y0 + 0.065, z0), M.crate)
            F.box(f'{name}_end{s}', 0.36, 0.1 if s > 0 else 0.13, 0.012, (x, y0 + (0.05 if s > 0 else 0.065), z0 + s * 0.244),
                  M.crate)
        for i in range(3):
            for j in range(3):
                r = 0.05 if fruit is not M.lemon else 0.045
                fx = x - 0.1 + i * 0.1 + 0.015 * (j % 2)
                fz = z0 - 0.15 + j * 0.15
                G.sphere(f'{name}_fruit{i}{j}', r, at=F.at(fx, y0 + 0.085 + 0.012 * ((i + j) % 2), fz), material=fruit,
                         scale=(1, 1, 0.9) if fruit is not M.lemon else (1.25, 1, 0.9), segments=8, rings=5)
    crate = G.join(parts, name, origin=F.at(x, y0, z0))
    crate.rotation_euler[0] = math.radians(10)   # its front down, so the fruit shows
    return crate


def marketStall():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.box('table', 1.84, 0.05, 0.8, (0, 0.8, 0.02), M.board, bevel=0.006)
        F.box('apron_front', 1.7, 0.1, 0.03, (0, 0.725, 0.4), M.timber)
        F.box('apron_back', 1.7, 0.1, 0.03, (0, 0.725, -0.36), M.timber)
        F.box('shelf', 1.7, 0.03, 0.6, (0, 0.2, 0.02), M.timber)
        for x in (-0.88, 0.88):
            for z, h in ((-0.4, 2.1), (0.4, 1.9)):
                F.box(f'post{x}{z}', 0.06, h, 0.06, (x, h / 2, z), M.timber, bevel=0.006)
        F.box('beam_front', 1.82, 0.06, 0.05, (0, 1.87, 0.4), M.timber, bevel=0.006)
        F.box('beam_back', 1.82, 0.06, 0.05, (0, 2.07, -0.4), M.timber, bevel=0.006)
        for i in range(7):
            x = -0.92 + 0.263 * i + 0.1315
            m = M.awning_green if i % 2 == 0 else M.awning_white
            F.box(f'stripe{i}', 0.263, 0.03, 1.0, (x, 2.03, 0.02), m, rot=(12, 0, 0), bevel=0.004)
            F.box(f'flap{i}', 0.263, 0.1, 0.012, (x, 1.88, 0.5), m, bevel=0.004)
        # a chalk board hung from the front beam
        for dx in (-0.14, 0.14):
            G.rod(f'string{dx}', F.at(0.5 + dx, 1.84, 0.4), F.at(0.5 + dx, 1.72, 0.4), 0.004, material=M.chalk, verts=6)
        F.box('board', 0.36, 0.2, 0.02, (0.5, 1.62, 0.4), M.slate, bevel=0.006)
        for k, (w, dy) in enumerate(((0.22, 0.04), (0.16, 0.0), (0.2, -0.045))):
            F.box(f'chalk{k}', w, 0.012, 0.004, (0.48 - (0.22 - w) / 2, 1.62 + dy, 0.412), M.chalk)
        # spare crates under the table
        for k, x in enumerate((-0.6, -0.32)):
            F.box(f'spare{k}', 0.24, 0.12, 0.42, (x, 0.275, -0.12), M.crate, bevel=0.006)
        for k, (x, m) in enumerate(((-0.6, M.tomato), (-0.2, M.orange), (0.2, M.apple), (0.6, M.lemon))):
            _crate(F, f'crate{k}', x, m)
    G.join_all('marketStall')
    G.settle(centre=False)
    return G.meshes()


FAR = 0.25

FAMILY = G.Family(
    'stalls', M,
    [G.Model('iceCreamCart', iceCreamCart, size=(1.729, 2.355, 0.966), key='model.iceCreamCart', centre=(-0.0235, 0.0023),
             motions={'uses': {'iceCream': 'lid_open'}, 'hold': 2}, far=FAR,
             paint=[('cart', 'Body'), ('parasol', 'Parasol'), ('stripe', 'Bands')]),
     G.Model('hotDogStand', hotDogStand, size=(1.577, 2.387, 0.981), key='model.hotDogStand', centre=(-0.0915, 0.0033),
             motions={'uses': {'hotDog': 'lid_open'}}, far=FAR,
             paint=[('stand', 'Body'), ('awning', 'Awning')]),
     G.Model('vendingSnack', vendingSnack, size=(0.9, 1.88, 0.748), key='model.vendingSnack', centre=(0, -0.056),
             motions={'uses': {'snack': 'drop'}}, far=FAR,
             paint=[('snack_red', 'Body')]),
     G.Model('vendingDrink', vendingDrink, size=(0.9, 1.88, 0.785), key='model.vendingDrink', centre=(0, -0.0375),
             motions={'uses': {'drink': 'drop'}}, far=FAR,
             paint=[('drink_blue', 'Body')]),
     G.Model('marketStall', marketStall, size=(1.841, 2.148, 0.983), key='model.marketStall', centre=(0.0005, 0.02),
             far=FAR, paint=[('timber', 'Body'), ('awning_green', 'Awning')])],
    comment='Blender-built food stalls and vending machines (tools/art/blender/families/stalls.py). Replaces the '
            'primitive-part placeholders of tools/art/community.py by key; listed in the base manifest\'s packs. The '
            'placeholder is kept as a fallback if a file fails to load, and the materials map onto the game\'s finishes '
            'so a model costs one draw call per finish group. The ice cream cart slides its freezer lid open and the '
            'hot dog stand lifts its hot-plate lid (`lid_open`); a snack and a can drop to the machines\' trays '
            '(`drop`). Each has a far version for the town overview.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

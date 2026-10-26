# The town's food stalls and vending machines (CM3): the ice cream cart (`model.iceCreamCart`), the hot dog
# stand (`model.hotDogStand`), the snack machine (`model.vendingSnack`), the drinks machine
# (`model.vendingDrink`) and the market stall (`model.marketStall`). Notes: stalls.md.
#
# Each is built where the game places it (`F`, game axes; `G.settle(centre=False)` and `centre=` on its
# `Model`): the carts and the stall across their 2 x 1 tiles, the machines against the back of their tile.
# The moving parts: the cart's freezer lid slides across (`lid_open`), the stand's hot-plate lid lifts
# (`lid_open`), each a second into the sale as the vendor behind reaches in, for 1.4 s; a snack slides
# down to the machine's tray and a can drops to it (`drop`). The
# machines' lit signs (and the drinks machine's front) glow at night. They are placed on town lots and seen
# from far, so each has a far version (`_far`: a quarter of the triangles).
#
# Run:  cargo art stalls [--only iceCreamCart] [--preview]
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


def _wheel(F, name, x, z, r, spokes=6):
    """A wheel at game (x, z), its axle along the depth: a rounded rubber tyre, a steel rim, spokes and a domed hub."""
    w = 0.06
    tyre = [(r * 0.72, -w / 2), (r * 0.93, -w / 2), (r, -w * 0.22), (r, w * 0.22), (r * 0.93, w / 2), (r * 0.72, w / 2)]
    G.revolve(f'{name}_tyre', tyre + [tyre[0]], at=F.at(x, r, z), material=M.tyre, segments=14, rot=(90, 0, 0))
    rim = [(r * 0.5, -w * 0.3), (r * 0.74, -w * 0.38), (r * 0.74, w * 0.38), (r * 0.5, w * 0.3)]
    G.revolve(f'{name}_rim', rim + [rim[0]], at=F.at(x, r, z), material=M.steel, segments=14, rot=(90, 0, 0))
    G.revolve(f'{name}_hub', [(0, -w * 0.62), (r * 0.24, -w * 0.62), (r * 0.24, w * 0.5), (r * 0.12, w * 0.7), (0, w * 0.7)],
              at=F.at(x, r, z), material=M.steel, segments=8, rot=(90, 0, 0))
    for k in range(spokes):
        a = math.tau * k / spokes
        F.box(f'{name}_spoke{k}', 0.012, r * 0.52, 0.01, (x + r * 0.46 * math.sin(a), r + r * 0.46 * math.cos(a), z), M.steel,
              rot=(0, 0, math.degrees(a)))


def _striped(obj, gores, first, second):
    """Gives a canopy's faces alternate materials round it by the angle of their middles (a parasol's gores); its
    underside the second."""
    obj.data.materials.append(second)
    for p in obj.data.polygons:
        if abs(p.normal.z) > 0.9 and p.normal.z < 0:
            p.material_index = 1
            continue
        a = math.atan2(p.center.y, p.center.x) % G.TAU
        p.material_index = int(a / (G.TAU / gores)) % 2
    return obj


def _valance(F, name, x0, x1, y, z, drop, materials, scallops):
    """A scalloped valance along the front of an awning, from game x0 to x1 hanging from height y at depth z: one
    flat strip per stripe, each `scallops` round tongues across, in the stripes' materials in turn."""
    n = len(materials)
    w = (x1 - x0) / n
    for i in range(n):
        cx = x0 + w * (i + 0.5)
        pts = [(w / 2, 0.0), (-w / 2, 0.0)]
        for k in range(scallops):
            u0 = -w / 2 + w * k / scallops
            for j in range(5):
                f = j / 4
                pts.append((u0 + w / scallops * f, -drop + 0.022 * (1 - math.sin(math.pi * f))))
        pts = G.forms._dedupe(pts)
        F.extrude(f'{name}{i}', pts, 0.008, (cx, y, z), materials[i])


# --- the ice cream cart: a cream freezer on wheels, a parasol over it ------------------------------------
CART_X = -0.15          # the body's middle across
TILT = 11.0             # the parasol leans forward this much, from its socket at the back of the top


def _pole_at(s):
    """A point `s` metres up the parasol's pole (game axes), from its socket at the back of the top."""
    return (CART_X, 0.97 + s * G.cosd(TILT), -0.2 + s * G.sind(TILT))


def _parasol(name, r, h, gores, a, b):
    """A parasol's canopy at the origin (Blender, its rim at z 0, its peak at h): gores sagging a little between
    the ribs, in materials a and b by turns, a scalloped valance hanging from the rim, ribs under it, a finial.
    Built upright; the caller tilts and places it."""
    n = gores * 2
    rings = []
    for t, k in ((0.0, 1.0), (0.35, 0.68), (0.7, 0.32), (0.95, 0.06)):
        ring = []
        for i in range(n):
            ang = math.tau * i / n
            sag = 0.0 if i % 2 == 0 else 0.03 * (1 - t)
            ring.append(((r * k - sag) * math.cos(ang), (r * k - sag) * math.sin(ang), h * t * (1 + 0.04 * (1 - t))))
        rings.append(ring)
    canopy = _striped(G.loft(f'{name}_canopy', rings, material=a, smooth=False), gores, a, b)
    # the valance: a band of scallops round the rim, a tongue a gore
    out, inn = [], []
    for i in range(gores * 3):
        ang = math.tau * i / (gores * 3)
        f = (i % 3) / 3
        d = 0.06 - 0.02 * (1 - math.sin(math.pi * f))
        out.append((r * math.cos(ang), r * math.sin(ang), -d))
        inn.append(((r - 0.006) * math.cos(ang), (r - 0.006) * math.sin(ang), -d))
    top_o = [(r * math.cos(math.tau * i / (gores * 3)), r * math.sin(math.tau * i / (gores * 3)), 0.004) for i in range(gores * 3)]
    top_i = [((r - 0.006) * math.cos(math.tau * i / (gores * 3)), (r - 0.006) * math.sin(math.tau * i / (gores * 3)), 0.004)
             for i in range(gores * 3)]
    val = forms_band(f'{name}_valance', top_o, out, inn, top_i, a)
    _striped(val, gores, a, b)
    made = [canopy, val]
    for i in range(0, n, 2):                       # the ribs under the canopy
        ang = math.tau * i / n
        made.append(G.sweep(f'{name}_rib{i}', [(0.02 * math.cos(ang), 0.02 * math.sin(ang), h * 0.82),
                                               ((r - 0.01) * math.cos(ang), (r - 0.01) * math.sin(ang), -0.004)], 0.004,
                            material=M.steel, verts=4))
    made.append(G.revolve(f'{name}_finial', [(0, h * 0.95), (0.022, h * 0.95), (0.026, h * 0.95 + 0.03),
                                              (0.016, h * 0.95 + 0.05), (0.02, h * 0.95 + 0.07), (0.012, h * 0.95 + 0.083),
                                              (0, h * 0.95 + 0.086)], material=M.steel, segments=8))
    return made


def forms_band(name, a, b, c, d, material):
    """A closed thin band through four loops of equal count (outer top, outer bottom, inner bottom, inner top), each
    joined to the next and the last back to the first: a valance."""
    import bpy
    m = len(a)
    vs = [*a, *b, *c, *d]
    faces = []
    for r0 in range(4):
        r1 = (r0 + 1) % 4
        for k in range(m):
            k2 = (k + 1) % m
            faces.append((r0 * m + k, r0 * m + k2, r1 * m + k2, r1 * m + k))
    return G.forms._object(name, vs, faces, material, False, 40)


def iceCreamCart():
    F = G.GameFrame()
    with G.new_parts() as still:
        # the body: a rounded cream box, two pink bands round it, a pink cone in a circle on its front
        F.soft('cart', 1.3, 0.62, 0.62, (CART_X, 0.62, 0.0), M.cart, round=0.04, round_segments=2, flat=1)
        for y in (0.5, 0.74):
            F.soft(f'band{y}', 1.312, 0.1, 0.632, (CART_X, y, 0.0), M.stripe, round=0.04, round_segments=2, flat=1)
        F.cyl('badge', 0.1, 0.008, (CART_X, 0.62, 0.312), M.stripe, verts=20, rot=(90, 0, 0))
        F.extrude('badge_cone', [(-0.035, 0.0), (0.035, 0.0), (0.0, -0.09)], 0.006, (CART_X, 0.64, 0.318), M.wafer)
        F.cyl('badge_scoop', 0.04, 0.006, (CART_X, 0.655, 0.318), M.cart, verts=12, rot=(90, 0, 0))
        # the chassis: a steel frame, mudguards over the wheels
        F.box('chassis', 1.16, 0.04, 0.46, (CART_X, 0.3, 0.0), M.steel, bevel=0.006, bevel_segments=1)
        for x in (-0.62, 0.38):
            for z in (-0.27, 0.27):
                _wheel(F, f'wheel{x}{z}', x, z, 0.17)
            for z in (-0.3, 0.3):
                F.sweep(f'axle{x}{z}', [(x, 0.17, 0.0), (x, 0.17, z)], 0.012, M.steel, verts=6)
        # the push bar: one bent tube from the body out to the handle and back, a rubber grip on it
        bar = [(0.5, 0.86, -0.22), (0.82, 1.0, -0.22), (0.82, 1.0, 0.22), (0.5, 0.86, 0.22)]
        F.sweep('bar', G.kit.fillet(bar, 0.05), 0.016, M.steel, verts=8)
        F.sweep('grip', [(0.82, 1.0, -0.13), (0.82, 1.0, 0.13)], 0.022, M.tyre, verts=8)
        for z in (-0.22, 0.22):
            F.cyl(f'bar_mount{z}', 0.03, 0.02, (0.495, 0.86, z), M.steel, verts=8, rot=(0, 0, 90))
        # the steel deck round the freezer well, the lids' rails, the fixed lid on the right with a hinge strip
        F.box('deck_back', 1.3, 0.04, 0.18, (CART_X, 0.95, -0.22), M.steel, bevel=0.006)
        F.box('deck_front', 1.3, 0.04, 0.02, (CART_X, 0.95, 0.30), M.steel, bevel=0.006)
        for x in (-0.77, 0.47):
            F.box(f'deck_end{x}', 0.06, 0.04, 0.42, (x, 0.95, 0.08), M.steel, bevel=0.006, bevel_segments=1)
        for z in (-0.115, 0.275):
            F.box(f'rail{z}', 1.18, 0.06, 0.03, (CART_X + 0.005, 0.96, z), M.steel, bevel=0.004, bevel_segments=1)
        F.box('lid_fixed', 0.59, 0.02, 0.36, (0.145, 0.98, 0.08), M.steel, bevel=0.004)
        for k in range(5):                             # its pressed ribs
            F.box(f'lid_rib{k}', 0.5, 0.006, 0.012, (0.145, 0.991, 0.0 + 0.04 * k), M.steel)
        # the tubs in the well (seen with the lid open), sunk into the top, each heaped with scooped ice cream
        creams = [M.cream_pink, M.cream_vanilla, M.cream_choc, M.cream_mint, M.cream_pink, M.cream_vanilla]
        for k, x in enumerate((-0.64, -0.445, -0.25, -0.05, 0.145, 0.34)):
            G.revolve(f'tub{k}', [(0, 0.88), (0.08, 0.88), (0.088, 0.952), (0.09, 0.958), (0.077, 0.958), (0, 0.93)],
                      at=F.at(x, 0, 0.08), material=M.steel, segments=10)
            cream = G.sphere(f'cream{k}', 0.07, at=F.at(x, 0.94, 0.08), material=creams[k], scale=(1, 1, 0.32),
                             segments=10, rings=5)
            G.jitter(cream, 0.008, 0.04, seed=k * 1.3)
        # a scoop resting on the back of the deck
        F.sweep('scoop_handle', [(-0.5, 0.975, -0.24), (-0.36, 0.975, -0.22)], 0.009, M.tyre, verts=6)
        F.sphere('scoop_bowl', 0.022, (-0.335, 0.98, -0.22), M.steel, segments=8, rings=4, scale=(1, 1, 0.6))
        # the sign: a waffle cone with a scoop on top, a drip round its rim, on the back of the top
        G.revolve('wafer', [(0, 0.97), (0.008, 0.97), (0.065, 1.16), (0.07, 1.17), (0.0, 1.17)], at=F.at(0.3, 0, -0.225),
                  material=M.wafer, segments=10)
        for k in range(3):
            F.box(f'waffle{k}', 0.004, 0.16, 0.004, (0.3 + 0.03 * (k - 1), 1.08, -0.225 + 0.05), M.cream_choc,
                  rot=(-18, 0, (k - 1) * 20))
        scoop = F.sphere('scoop', 0.08, (0.3, 1.2, -0.225), M.cream_pink, segments=12, rings=7)
        G.jitter(scoop, 0.006, 0.05, seed=2.0)
        F.sweep('drip', [(0.3 + 0.075 * math.cos(math.tau * i / 12), 1.18, -0.225 + 0.075 * math.sin(math.tau * i / 12))
                         for i in range(12)], 0.014, M.cream_pink, verts=5, closed=True)
        # the parasol, leaning forward from a socket at the back of the top
        F.cyl('socket', 0.035, 0.08, (CART_X, 1.0, -0.2), M.steel, verts=10)
        F.sweep('pole', [_pole_at(0.0), _pole_at(1.38)], 0.018, M.steel, verts=8)
        F.cyl('runner', 0.026, 0.06, _pole_at(0.9), M.steel, verts=8, rot=(TILT, 0, 0))
    with G.new_parts() as top:
        _parasol('parasol', 0.492, 0.28, 16, M.parasol, M.parasol_white)
    canopy = G.join(top, 'canopy', origin=(0, 0, 0))
    canopy.scale = (1.5, 0.985, 1)                 # (as wide and deep as the old parasol)
    canopy.rotation_euler = (math.radians(TILT), 0, 0)
    px, py, pz = _pole_at(1.06)
    canopy.location = F.at(px, py, pz + 0.004)
    G.join([*still, canopy], 'body', origin=(0, 0, 0))
    # the sliding lid over the left half of the well, on the rails; a recessed grip at its left end
    with G.new_parts() as parts:
        F.box('lid', 0.59, 0.02, 0.42, (-0.445, 1.0, 0.08), M.steel, bevel=0.004)
        for k in range(5):
            F.box(f'lid_rib{k}', 0.46, 0.006, 0.012, (-0.42, 1.011, 0.0 + 0.04 * k), M.steel)
        F.sweep('lid_grip', G.kit.fillet([(-0.70, 1.008, -0.02), (-0.70, 1.035, -0.02), (-0.70, 1.035, 0.18),
                                          (-0.70, 1.008, 0.18)], 0.012), 0.009, M.tyre, verts=6)
    lid = G.join(parts, 'lid')
    G.settle(centre=False)
    G.slide(lid, 'lid_open', (-0.58, 0, 0), seconds=1.0)   # to the right (game +x), over the fixed lid
    return G.meshes()


# --- the hot dog stand: a red cart with a hot plate, under a striped awning on four posts -----------------
def _sausage(F, name, x, y, z, length, r, material, along='z', bend=0.006):
    """A sausage lying along the depth (or across, `along` 'x'): a tube bowed a little, rounded off at its ends."""
    pts = []
    for i in range(7):
        f = i / 6 - 0.5
        off = bend * (1 - (2 * f) ** 2)
        pts.append((x + off, y, z + f * length) if along == 'z' else (x + f * length, y + off, z))
    return F.sweep(name, pts, r, material, verts=8, resolution=1,
                   scale=lambda t: max(0.35, min(1.0, math.sin(math.pi * t) * 2.2)))


def _bun(F, name, x, y, z, length, material, along='z'):
    """A split bun lying along the depth or across: two soft halves, a little apart."""
    for s in (-1, 1):
        if along == 'z':
            F.soft(f'{name}{s}', 0.034, 0.04, length, (x + s * 0.02, y, z), material, round=0.017, round_segments=1, flat=1)
        else:
            F.soft(f'{name}{s}', length, 0.04, 0.034, (x, y, z + s * 0.02), material, round=0.017, round_segments=1, flat=1)


def _squeezy(F, name, x, y, z, material, cap):
    """A squeeze bottle standing at game (x, y, z): a soft body, a shoulder, a cap and a long nozzle."""
    bx, by, bz = F.at(x, y, z)
    rows = [(0.0, 0.034), (0.01, 0.04), (0.12, 0.04), (0.15, 0.03), (0.165, 0.016)]
    G.loft(name, [G.ring(r, r, bz + h, 10, at=(bx, by)) for h, r in rows], material=material)
    F.cyl(f'{name}_cap', 0.018, 0.016, (x, y + 0.172, z), cap, verts=8)
    F.cone(f'{name}_nozzle', 0.01, 0.003, 0.045, (x, y + 0.2, z), cap, verts=6)


def hotDogStand():
    F = G.GameFrame()
    with G.new_parts() as still:
        F.soft('stand', 1.5, 0.66, 0.66, (-0.1, 0.62, -0.05), M.stand, round=0.035, round_segments=2, flat=1)
        # the steel top with a rolled front edge, a lower shelf rail along the front
        F.soft('top', 1.56, 0.05, 0.72, (-0.1, 0.975, -0.05), M.steel, round=0.02, round_segments=2, flat=1)
        F.box('chassis', 1.4, 0.04, 0.5, (-0.1, 0.27, -0.05), M.steel, bevel=0.006, bevel_segments=1)
        F.box('trim', 1.52, 0.03, 0.68, (-0.1, 0.3, -0.05), M.steel, bevel=0.008, bevel_segments=1)
        for x in (-0.68, 0.5):
            for z in (0.24, -0.34):
                _wheel(F, f'wheel{x}{z}', x, z, 0.2)
        # the sign on the front: a framed plate, a hot dog in its bun with a mustard zigzag
        F.extrude('sign_frame', G.rounded_rect(0.66, 0.26, 0.02, corner=2), 0.016, (-0.1, 0.72, 0.284), M.steel,
                  holes=[G.rounded_rect(0.62, 0.22, 0.012, corner=2)])
        F.box('sign', 0.62, 0.22, 0.02, (-0.1, 0.72, 0.29), M.sign, bevel=0.004, bevel_segments=1)
        F.soft('sign_bun', 0.44, 0.07, 0.02, (-0.1, 0.705, 0.302), M.bun, round=0.01, round_segments=2, flat=1)
        _sausage(F, 'sign_sausage', -0.1, 0.735, 0.31, 0.5, 0.024, M.sausage, along='x', bend=0.0)
        F.sweep('sign_mustard', [(-0.1 - 0.18 + 0.03 * i, 0.758 + 0.006 * (-1) ** i, 0.334) for i in range(13)], 0.004,
                M.mustard, verts=4)
        # the hot plate: a steel tray, a dark ribbed plate, sausages and buns under the lid
        F.box('plate_tray', 0.6, 0.04, 0.4, (-0.35, 1.02, -0.1), M.steel, bevel=0.006, bevel_segments=1)
        F.box('plate', 0.54, 0.008, 0.34, (-0.35, 1.042, -0.1), M.grill)
        for k in range(8):
            F.box(f'rib{k}', 0.54, 0.004, 0.006, (-0.35, 1.047, -0.25 + 0.042 * k), M.grill)
        for k in range(5):
            _sausage(F, f'sausage{k}', -0.55 + 0.08 * k, 1.066, -0.12, 0.22, 0.018, M.sausage)
        for k, x in enumerate((-0.6, -0.13)):
            _bun(F, f'bun{k}', x, 1.07, -0.12, 0.2, M.bun)
        # sauces, a napkin box with a napkin pulled up, a stack of paper trays at the right
        for x, m in ((0.3, M.mustard), (0.42, M.ketchup)):
            _squeezy(F, f'bottle{x}', x, 1.0, 0.1, m, m)
        F.box('napkins', 0.14, 0.08, 0.1, (0.36, 1.04, -0.15), M.steel, bevel=0.006, bevel_segments=1)
        F.extrude('napkin', [(-0.04, 0.0), (0.04, 0.0), (0.03, 0.05), (0.0, 0.06), (-0.035, 0.045)], 0.004,
                  (0.36, 1.075, -0.15), M.sign)
        for k in range(3):
            F.box(f'tray{k}', 0.16, 0.012, 0.08, (0.12, 1.006 + 0.012 * k, -0.2), M.sign, bevel=0.003, bevel_segments=1)
        # the posts and the awning: six stripes leaning down to the front, a scalloped valance along it
        for x in (-0.82, 0.62):
            for z, top in ((-0.4, 2.35), (0.38, 2.23)):
                F.sweep(f'post{x}{z}', [(x, 1.0, z), (x, top, z)], 0.02, M.steel, verts=8)
                F.cyl(f'post_foot{x}{z}', 0.03, 0.02, (x, 1.01, z), M.steel, verts=8)
        mats = []
        for i in range(6):
            x = -0.85 + 0.25 * i + 0.125
            m = M.awning if i % 2 else M.awning_white
            mats.append(m)
            F.box(f'stripe{i}', 0.25, 0.04, 0.98, (x, 2.3, 0.0), m, rot=(8, 0, 0), bevel=0.004, bevel_segments=1)
        _valance(F, 'flap', -0.85, 0.65, 2.25, 0.488, 0.1, mats, 2)
    G.join(still, 'body', origin=(0, 0, 0))
    # the hot plate's lid, hinged on its back edge: a rounded steel hood, a bar handle along its front
    with G.new_parts() as parts:
        F.soft('lid', 0.6, 0.1, 0.4, (-0.35, 1.09, -0.1), M.steel, round=0.035, round_segments=2, flat=1)
        G.kit.pull(F, 'lid_handle', -0.35, 1.1, 0.1, 0.22, M.steel, r=0.008, out=0.03)
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
    kick panel round the tray's opening with its push flap, and the dark lining inside."""
    w = WIN_X[1] - WIN_X[0]
    cx = (WIN_X[0] + WIN_X[1]) / 2
    F.box('plinth', 0.86, 0.04, 0.68, (0, 0.02, -0.08), M.tray)
    for sx in (-1, 1):                                # levelling feet
        for z in (-0.38, 0.22):
            F.cyl(f'foot{sx}{z}', 0.022, 0.03, (sx * 0.38, 0.015, z), M.tray, verts=8)
    F.box('back', 0.9, 1.84, 0.46, (0, 0.96, -0.2), body, bevel=0.02, bevel_segments=2)
    for s in (-1, 1):
        F.box(f'side{s}', 0.04, 1.84, 0.27, (s * 0.43, 0.96, 0.155), body, bevel=0.012, bevel_segments=2)
    F.box('header', 0.82, 0.16, 0.26, (0, 1.80, 0.16), body, bevel=0.01, bevel_segments=1)
    F.extrude('header_sign', G.rounded_rect(0.72, 0.1, 0.02, corner=2), 0.01, (0, 1.80, FRONT + 0.005), M.lit_sign)
    for k in range(6):                                # its six letters
        F.box(f'letter{k}', 0.05 if k else 0.06, 0.05 if k else 0.06, 0.004, (-0.19 + 0.076 * k, 1.80, FRONT + 0.011), body)
    # the keypad column on the right: a dark panel with a display, twelve keys, a card reader, a coin slot and
    # a coin return
    F.box('column', 0.25, 1.68, 0.26, (0.285, 0.88, 0.16), body, bevel=0.008, bevel_segments=1)
    F.extrude('keypad', G.rounded_rect(0.18, 0.5, 0.015, corner=2), 0.02, (0.285, 1.2, FRONT + 0.01), M.inside)
    F.box('screen', 0.12, 0.07, 0.01, (0.285, 1.37, FRONT + 0.022), M.display)
    for r in range(4):
        for c in range(3):
            F.box(f'key{r}{c}', 0.035, 0.03, 0.01, (0.245 + 0.04 * c, 1.27 - 0.045 * r, FRONT + 0.022), M.button,
                  bevel=0.003, bevel_segments=1)
    F.box('reader', 0.1, 0.1, 0.012, (0.285, 1.03, FRONT + 0.022), M.button, bevel=0.004, bevel_segments=1)
    F.box('reader_slot', 0.07, 0.006, 0.004, (0.285, 1.05, FRONT + 0.029), M.tray)
    F.box('coin_slot', 0.012, 0.04, 0.004, (0.285, 1.105, FRONT + 0.022), M.tray)
    F.extrude('coin_return', G.rounded_rect(0.08, 0.06, 0.01, corner=1), 0.01, (0.285, 0.6, FRONT + 0.004), M.inside)
    F.box('coin_cup', 0.05, 0.02, 0.006, (0.285, 0.59, FRONT + 0.01), M.tray)
    # the kick panel round the tray's opening, a push flap over the opening
    F.box('kick', w, BAY_Y[0] - 0.04, 0.26, (cx, (0.04 + BAY_Y[0]) / 2, 0.16), body)
    F.box('lip', w, WIN_Y[0] - BAY_Y[1], 0.025, (cx, (BAY_Y[1] + WIN_Y[0]) / 2, FRONT - 0.0125), body)
    for x0, x1 in ((WIN_X[0], BAY_X[0]), (BAY_X[1], WIN_X[1])):
        F.box(f'cheek{x0}', x1 - x0, BAY_Y[1] - BAY_Y[0], 0.26, ((x0 + x1) / 2, sum(BAY_Y) / 2, 0.16), body)
    F.box('flap_word', 0.08, 0.014, 0.004, (cx, BAY_Y[1] + 0.05, FRONT + 0.001), M.label)   # "PUSH"
    # the dark inside: the back, the sides and the top behind the window, and the tray's well
    F.box('lining_back', w, WIN_Y[1] - BAY_Y[0], 0.01, (cx, (BAY_Y[0] + WIN_Y[1]) / 2, 0.035), M.inside)
    for x in (WIN_X[0] + 0.003, WIN_X[1] - 0.003):
        F.box(f'lining{x}', 0.006, WIN_Y[1] - BAY_Y[0], 0.25, (x, (BAY_Y[0] + WIN_Y[1]) / 2, 0.16), M.inside)
    F.box('lining_top', w, 0.006, 0.25, (cx, WIN_Y[1] - 0.003, 0.16), M.inside)
    F.box('tray_floor', w, 0.006, 0.25, (cx, BAY_Y[0] + 0.003, 0.16), M.tray)
    # the window's frame: a rounded bezel round the glass
    F.extrude('bezel', G.rounded_rect(w + 0.03, WIN_Y[1] - WIN_Y[0] + 0.03, 0.02, corner=2), 0.012,
              (cx, sum(WIN_Y) / 2, FRONT + 0.002), body, holes=[G.rounded_rect(w - 0.01, WIN_Y[1] - WIN_Y[0] - 0.01, 0.012, corner=2)])


def _coil(F, name, x, y, z0, z1, r=0.03, turns=2.0):
    """A vending machine's spiral, from depth z0 to z1 at (x, y): the wire a snack sits in."""
    n = int(turns * 5)
    pts = [(x + r * math.sin(math.tau * turns * i / n), y + r * math.cos(math.tau * turns * i / n), z0 + (z1 - z0) * i / n)
           for i in range(n + 1)]
    F.sweep(name, pts, 0.0035, M.steel, verts=3)


def _bag(F, name, x, y, z, w, h, d, material):
    """A crisp packet standing up: a pillow puffed out front and back, a crimped seal along its top."""
    F.soft(name, w, h, d, (x, y, z), material, round=0.01, puff=(0, 0, d * 0.35), round_segments=1, flat=2)
    F.box(f'{name}_seal', w * 0.98, 0.012, d * 0.3, (x, y + h / 2 - 0.004, z), material)


def vendingSnack():
    F = G.GameFrame()
    cx = (WIN_X[0] + WIN_X[1]) / 2
    drop = None
    with G.new_parts() as still:
        _cabinet(F, M.snack_red)
        # five shelves of snacks, two deep, each in its spiral, a label strip with numbers along each shelf
        for row in range(5):
            yb = 0.47 + row * 0.25
            F.box(f'shelf{row}', WIN_X[1] - WIN_X[0], 0.015, 0.19, (cx, yb, 0.125), M.steel)
            F.box(f'labels{row}', WIN_X[1] - WIN_X[0], 0.025, 0.004, (cx, yb - 0.004, 0.222), M.label)
            for col in range(5):
                x = -0.355 + col * 0.115
                F.box(f'number{row}{col}', 0.03, 0.012, 0.003, (x, yb - 0.004, 0.2245), M.inside)
                _coil(F, f'coil{row}{col}', x, yb + 0.04, 0.05, 0.215)
                m = getattr(M, SNACKS[(row * 5 + col * 3) % len(SNACKS)])
                h = 0.14 - 0.02 * ((row + col) % 2)
                F.box(f'back{row}{col}', 0.09, h, 0.03, (x, yb + 0.0075 + h / 2, 0.12), m)
                if (row, col) == (1, 1):
                    drop = (x, yb + 0.0075 + h / 2, h, m)
                    continue
                if (row + col) % 2:
                    _bag(F, f'snack{row}{col}', x, yb + 0.0075 + h / 2, 0.195, 0.09, h, 0.03, m)
                else:
                    F.box(f'snack{row}{col}', 0.09, h, 0.03, (x, yb + 0.0075 + h / 2, 0.195), m)
                    F.box(f'snack_label{row}{col}', 0.06, h * 0.35, 0.002, (x, yb + 0.0075 + h * 0.55, 0.211), M.label)
        F.box('window', WIN_X[1] - WIN_X[0], WIN_Y[1] - WIN_Y[0], 0.01, (cx, sum(WIN_Y) / 2, FRONT - 0.015), M.glass)
    G.join(still, 'body', origin=(0, 0, 0))
    # the snack that drops: the front one of the second shelf's second column (a crisp packet)
    x, y, h, m = drop
    with G.new_parts() as parts:
        _bag(F, 'snack', x, y, 0.195, 0.09, h, 0.03, m)
    snack = G.join(parts, 'snack', origin=F.at(x, y, 0.195))
    G.settle(centre=False)
    fall = y - (BAY_Y[0] + 0.006 + h / 2)
    G.slide(snack, 'drop', (0, -0.05, 0), seconds=0.15)       # pushed off the shelf to the front
    G.slide(snack, 'drop', (0, 0, -fall), seconds=0.9)        # and down past the shelves into the tray
    G.swing(snack, 'drop', -15, seconds=0.9, axis='X')        # tipping back as it lands
    return G.meshes()


def _can(name, at, r, h, material, top, verts=14, rot=(0, 0, 0)):
    """A drinks can about Blender point `at` (its middle), upright unless turned: a body necked in at the top and the
    bottom, its lid's rim and a ring pull."""
    prof = [(0, -h / 2), (r * 0.8, -h / 2), (r, -h / 2 + r * 0.25), (r, h / 2 - r * 0.45), (r * 0.82, h / 2 - r * 0.05),
            (r * 0.84, h / 2), (0, h / 2)]
    body = G.revolve(name, prof, material=material, segments=verts)
    lid = G.revolve(f'{name}_lid', [(0, h / 2 - 0.004), (r * 0.78, h / 2 - 0.004), (r * 0.8, h / 2 + 0.002),
                                    (r * 0.7, h / 2 + 0.002), (r * 0.68, h / 2 - 0.001), (0, h / 2 - 0.001)],
                    material=top, segments=verts)
    for o in (body, lid):
        o.location = at
        o.rotation_euler = tuple(math.radians(a) for a in rot)
    return body, lid


def vendingDrink():
    F = G.GameFrame()
    cx = (WIN_X[0] + WIN_X[1]) / 2
    with G.new_parts() as still:
        _cabinet(F, M.drink_blue)
        # the lit front, its bands, three big cans standing proud of it with a lit button and a price under each
        F.box('panel', WIN_X[1] - WIN_X[0], WIN_Y[1] - WIN_Y[0], 0.012, (cx, sum(WIN_Y) / 2, FRONT - 0.006), M.lit_panel)
        F.box('band_top', WIN_X[1] - WIN_X[0], 0.12, 0.012, (cx, 1.6, FRONT + 0.004), M.band)
        F.box('band_low', WIN_X[1] - WIN_X[0], 0.07, 0.012, (cx, 0.7, FRONT + 0.004), M.band)
        # bubbles rising on the panel behind the cans
        for k, (x, y, r) in enumerate(((-0.36, 1.45, 0.02), (-0.2, 1.48, 0.014), (-0.02, 1.44, 0.018), (0.1, 1.47, 0.012),
                                       (-0.29, 1.04, 0.012), (-0.08, 1.06, 0.016), (0.08, 1.02, 0.012))):
            F.cyl(f'bubble{k}', r, 0.004, (x, y, FRONT + 0.001), M.band, verts=10, rot=(90, 0, 0))
        for i, m in enumerate((M.snack_c, M.snack_a, M.can_green)):
            x = -0.315 + i * 0.19
            _can(f'can{i}', F.at(x, 1.2, FRONT + 0.01), 0.055, 0.25, m, M.steel, verts=16)
            F.cyl(f'can_label{i}', 0.0565, 0.07, (x, 1.19, FRONT + 0.01), M.label, verts=16)
            F.extrude(f'button{i}', G.rounded_rect(0.1, 0.05, 0.012, corner=2), 0.016, (x, 0.95, FRONT + 0.006), M.button)
            F.box(f'button_light{i}', 0.06, 0.01, 0.004, (x, 0.95, FRONT + 0.015), M.display)
            F.box(f'price{i}', 0.08, 0.025, 0.006, (x, 1.03, FRONT + 0.003), M.display)
    G.join(still, 'body', origin=(0, 0, 0))
    # the can that drops: lying across, hidden inside above the tray until it falls into it
    x, y, z = -0.15, 0.4, 0.2
    with G.new_parts() as parts:
        _can('can', F.at(x, y, z), 0.033, 0.12, M.snack_c, M.steel, verts=12, rot=(0, 90, 0))
    can = G.join(parts, 'can', origin=F.at(x, y, z))
    G.settle(centre=False)
    G.slide(can, 'drop', (0, 0, -(y - (BAY_Y[0] + 0.006 + 0.033))), seconds=0.7)   # down into the tray
    G.slide(can, 'drop', (0, -0.04, 0), seconds=0.9)                                 # rolling to its front
    return G.meshes()


# --- the market stall: a table of fruit crates under a striped awning on four posts ------------------------
def _crate(F, name, x, fruit, rng):
    """An open slatted crate on the table, tilted to the front, heaped with fruit; joined and turned whole."""
    with G.new_parts() as parts:
        y0, z0 = 0.84, 0.05
        F.box(f'{name}_floor', 0.36, 0.012, 0.5, (x, y0, z0), M.crate)
        for s in (-1, 1):
            for k, (yy, hh) in enumerate(((0.035, 0.05), (0.1, 0.05))):            # two slats a side, a gap between
                F.box(f'{name}_side{s}{k}', 0.012, hh, 0.5, (x + s * 0.174, y0 + yy, z0), M.crate)
            F.box(f'{name}_end{s}', 0.36, 0.1 if s > 0 else 0.13, 0.012, (x, y0 + (0.05 if s > 0 else 0.065), z0 + s * 0.244),
                  M.crate)
            for e in (-1, 1):                                                     # corner posts
                F.box(f'{name}_post{s}{e}', 0.022, 0.13 if e < 0 else 0.1, 0.022,
                      (x + s * 0.163, y0 + (0.065 if e < 0 else 0.05), z0 + e * 0.233), M.crate)
        # the fruit: a heap, a low row and a high row between, each a little turned and jostled
        lemon = fruit is M.lemon
        r = 0.045 if lemon else 0.05
        for i in range(3):
            for j in range(3):
                fx = x - 0.1 + i * 0.1 + 0.015 * (j % 2) + rng.uniform(-0.008, 0.008)
                fz = z0 - 0.15 + j * 0.15 + rng.uniform(-0.01, 0.01)
                fy = y0 + 0.085 + 0.012 * ((i + j) % 2)
                G.sphere(f'{name}_fruit{i}{j}', r, at=F.at(fx, fy, fz), material=fruit,
                         scale=(1.25, 1, 0.9) if lemon else (1, 1, 0.9), segments=6, rings=4,
                         rot=(0, 0, rng.uniform(0, 180)))
        for i in range(2):                                                        # a second layer on top of the first
            for j in range(2):
                fx, fz = x - 0.05 + i * 0.1, z0 - 0.075 + j * 0.15
                G.sphere(f'{name}_top{i}{j}', r, at=F.at(fx, y0 + 0.155, fz), material=fruit,
                         scale=(1.25, 1, 0.9) if lemon else (1, 1, 0.9), segments=6, rings=4, rot=(0, 0, rng.uniform(0, 180)))
        # a price card on a stick
        F.box(f'{name}_stick', 0.006, 0.12, 0.006, (x + 0.12, y0 + 0.17, z0 + 0.2), M.crate)
        F.box(f'{name}_card', 0.08, 0.05, 0.004, (x + 0.12, y0 + 0.24, z0 + 0.2), M.chalk)
        F.box(f'{name}_price', 0.05, 0.012, 0.002, (x + 0.12, y0 + 0.24, z0 + 0.203), M.slate)
    crate = G.join(parts, name, origin=F.at(x, y0, z0))
    crate.rotation_euler[0] = math.radians(10)   # its front down, so the fruit shows
    return crate


def marketStall():
    F = G.GameFrame()
    import random
    rng = random.Random(5)
    with G.new_parts() as still:
        # the table: a top of four planks, an apron, a slatted lower shelf
        for k in range(4):
            F.box(f'plank{k}', 1.84, 0.05, 0.196, (0, 0.8, 0.02 - 0.3 + 0.2 * k), M.board, bevel=0.005, bevel_segments=1)
        F.box('apron_front', 1.7, 0.1, 0.03, (0, 0.725, 0.4), M.timber)
        F.box('apron_back', 1.7, 0.1, 0.03, (0, 0.725, -0.36), M.timber)
        for k in range(4):
            F.box(f'shelf{k}', 1.7, 0.03, 0.13, (0, 0.2, 0.02 - 0.225 + 0.15 * k), M.timber)
        for z in (-0.38, 0.38):
            F.box(f'shelf_rail{z}', 1.76, 0.04, 0.04, (0, 0.17, z), M.timber)
        for x in (-0.88, 0.88):
            for z, h in ((-0.4, 2.1), (0.4, 1.9)):
                F.box(f'post{x}{z}', 0.06, h, 0.06, (x, h / 2, z), M.timber, bevel=0.006, bevel_segments=1)
            # knee braces under the beams
            for z, y in ((0.4, 1.84), (-0.4, 2.04)):
                F.box(f'brace{x}{z}', 0.04, 0.2, 0.04, (x - 0.08 * (1 if x > 0 else -1), y - 0.07, z), M.timber,
                      rot=(0, 45 * (1 if x > 0 else -1), 0))
        F.box('beam_front', 1.82, 0.06, 0.05, (0, 1.87, 0.4), M.timber, bevel=0.006, bevel_segments=1)
        F.box('beam_back', 1.82, 0.06, 0.05, (0, 2.07, -0.4), M.timber, bevel=0.006, bevel_segments=1)
        mats = []
        for i in range(7):
            x = -0.92 + 0.263 * i + 0.1315
            m = M.awning_green if i % 2 == 0 else M.awning_white
            mats.append(m)
            F.box(f'stripe{i}', 0.263, 0.02, 1.0, (x, 2.03, 0.02), m, rot=(12, 0, 0))
        _valance(F, 'flap', -0.9205, 0.9205, 1.93, 0.5, 0.1, mats, 1)
        # a chalk board hung from the front beam: a framed slate, chalked lines and a drawn apple
        for dx in (-0.14, 0.14):
            F.sweep(f'string{dx}', [(0.5 + dx, 1.84, 0.4), (0.5 + dx * 0.9, 1.72, 0.4)], 0.004, M.chalk, verts=4)
        F.box('board', 0.34, 0.18, 0.016, (0.5, 1.62, 0.4), M.slate)
        F.extrude('board_frame', G.rounded_rect(0.38, 0.22, 0.006, corner=1), 0.024, (0.5, 1.62, 0.4), M.timber,
                  holes=[G.rounded_rect(0.34, 0.18, 0.004, corner=1)])
        for k, (w, dy) in enumerate(((0.18, 0.04), (0.12, 0.0), (0.16, -0.045))):
            F.box(f'chalk{k}', w, 0.012, 0.004, (0.46 - (0.18 - w) / 2, 1.62 + dy, 0.41), M.chalk)
        F.cyl('chalk_apple', 0.022, 0.004, (0.62, 1.625, 0.41), M.tomato, verts=8, rot=(90, 0, 0))
        # a set of scales on the table's right end, paper bags beside them
        G.revolve('scale_base', [(0, 0), (0.06, 0), (0.06, 0.03), (0.035, 0.05), (0.018, 0.18), (0, 0.18)],
                  at=F.at(0.8, 0.825, -0.31), material=M.steel, segments=10)
        G.revolve('scale_pan', [(0, 0.18), (0.08, 0.18), (0.1, 0.21), (0.092, 0.212), (0.075, 0.192), (0, 0.192)],
                  at=F.at(0.8, 0.825, -0.31), material=M.steel, segments=12)
        F.cyl('scale_dial', 0.04, 0.01, (0.8, 0.94, -0.268), M.chalk, verts=10, rot=(90, 0, 0))
        for k in range(3):
            F.box(f'bag{k}', 0.12, 0.012, 0.16, (0.84, 0.832 + 0.012 * k, -0.33), M.crate, rot=(0, 5 * k, 0))
        # spare crates under the table, slatted
        for k, x in enumerate((-0.6, -0.32)):
            for j, y in enumerate((0.235, 0.315)):
                F.box(f'spare{k}{j}', 0.24, 0.05, 0.42, (x, y, -0.12), M.crate)
        for k, (x, m) in enumerate(((-0.6, M.tomato), (-0.2, M.orange), (0.2, M.apple), (0.6, M.lemon))):
            _crate(F, f'crate{k}', x, m, rng)
    G.join_all('marketStall')
    G.settle(centre=False)
    return G.meshes()


FAR = 0.25
FAR_FEET = 0.01      # (m) a far version keeps every vertex this close to the floor, as gamekit's far= does


def _far(build, ratio=FAR):
    """A far version as gamekit's `far=ratio` (the model at rest, still, joined, decimated to `ratio`, its feet
    kept), but with the vertices that sit on one another merged first: the cans' turned ends meet on their axis in
    many vertices, and decimating them unmerged tore the drinks machine's cabinet into spikes 26 cm below the floor
    (and the café table's legs away);
    and its outermost faces kept, so its box stays the model's."""
    import bmesh
    import bpy

    def far():
        build()
        bpy.context.scene.frame_set(1)
        for o in G.meshes():
            o.animation_data_clear()
        obj = G.join(G.meshes(), f'{build.__name__}_far', origin=(0, 0, 0))
        bm = bmesh.new()
        bm.from_mesh(obj.data)
        bmesh.ops.remove_doubles(bm, verts=bm.verts, dist=1e-5)
        bm.to_mesh(obj.data)
        bm.free()
        # its feet and its outermost faces are kept whole, so it stands and measures as the model does
        lo = [min(v.co[i] for v in obj.data.vertices) for i in range(3)]
        hi = [max(v.co[i] for v in obj.data.vertices) for i in range(3)]

        def free(v):
            return v.co.z > FAR_FEET and all(lo[i] + 0.003 < v.co[i] < hi[i] - 0.003 for i in range(3))
        keep = obj.vertex_groups.new(name='decimate')
        keep.add([v.index for v in obj.data.vertices if free(v)], 1.0, 'REPLACE')
        decimate = obj.modifiers.new('far', 'DECIMATE')
        decimate.ratio = ratio
        decimate.vertex_group = 'decimate'
        return [obj]
    far.__name__ = f'{build.__name__}_far'
    return far

FAMILY = G.Family(
    'stalls', M,
    [G.Model('iceCreamCart', iceCreamCart, size=(1.729, 2.355, 0.966), key='model.iceCreamCart', centre=(-0.0235, 0.0023),
             # the seller slides the freezer's lid open to scoop (the game's `sell` clip: 1.5-2.1 s in)
             motions={'uses': {'iceCream': {'clip': 'lid_open', 'delay': 1.0, 'hold': 1.4}}}, far=_far(iceCreamCart),
             paint=[('cart', 'Body'), ('parasol', 'Parasol'), ('stripe', 'Bands')]),
     G.Model('hotDogStand', hotDogStand, size=(1.577, 2.387, 0.981), key='model.hotDogStand', centre=(-0.0915, 0.0033),
             # the vendor lifts the hot plate's lid for a sausage
             motions={'uses': {'hotDog': {'clip': 'lid_open', 'delay': 1.0, 'hold': 1.4}}}, far=_far(hotDogStand),
             paint=[('stand', 'Body'), ('awning', 'Awning')]),
     G.Model('vendingSnack', vendingSnack, size=(0.9, 1.88, 0.748), key='model.vendingSnack', centre=(0, -0.056),
             motions={'uses': {'snack': 'drop'}}, far=_far(vendingSnack),
             paint=[('snack_red', 'Body')]),
     G.Model('vendingDrink', vendingDrink, size=(0.9, 1.88, 0.785), key='model.vendingDrink', centre=(0, -0.0375),
             motions={'uses': {'drink': 'drop'}}, far=_far(vendingDrink),
             paint=[('drink_blue', 'Body')]),
     G.Model('marketStall', marketStall, size=(1.841, 2.148, 0.983), key='model.marketStall', centre=(0.0005, 0.02),
             far=_far(marketStall), paint=[('timber', 'Body'), ('awning_green', 'Awning')])],
    comment='Blender-built food stalls and vending machines (tools/art/blender/families/stalls.py). '
            'Listed in the base manifest\'s packs. The materials map onto the game\'s finishes so a model '
            'costs one draw call per finish group. The ice cream cart slides its freezer lid open and the '
            'hot dog stand lifts its hot-plate lid (`lid_open`); a snack and a can drop to the machines\' '
            'trays (`drop`). Each has a far version for the town overview.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

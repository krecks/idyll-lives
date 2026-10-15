# The kitchen's dining things, over their primitive-part placeholders (`catalog.py`, `catalog2.py`, `furniture.py`),
# which stay the fallback if a file fails to load. Notes: dining.md.
#
# K7, dining tables: the dining table with its two chairs (`model.diningTable`, base manifest), the bistro table
# and chair (`model.bistroSet`), the glass and farmhouse dining sets (`model.diningGlass`, `model.diningOak`), and
# the chair-less tables of catalog2: the square table for two, the round bistro table, the tables for four, six and
# eight, the round pedestal table, the square table for eight, the farmhouse trestle table and the high café table.
# K8, chairs and stools: the bar, counter and high bar stools, whose seat swivels 20 degrees and back as a resident
# sits (`swivel`, carrying them: `ride`), the dining chair, the wooden, upholstered, moulded, Windsor, bentwood,
# folding and carver chairs and the dining bench.
# K9, dining storage: the walnut sideboard, the painted buffet, the china cabinet and the bar cart, whose cocktail
# shaker lifts 10 cm and tips 30 degrees while a resident mixes a drink (`shake`).
#
# Each is built in place where its placeholder stands (`F`, game axes): table tops at the content's `surface`
# heights (catalog2.py `TT` 0.75, `BT` 1.06; 0.75-0.765 for the catalog sets), seats where the game seats residents
# (`characters.ts` SEATS, content `seat`: the seat's top at that height, its middle 5 cm forward for catalog2's
# chairs). The sets' chairs are part of the table, as the placeholders draw them, since residents sit on them. The
# storage stands against the wall where its placeholder stands (`centre=`, the placeholder's middle: its back about
# 5 cm behind the tile's wall face, where the content's `surface` is). The sideboard's, buffet's and china cabinet's
# doors are parts of their own, hinged but still: no use opens them yet (the plan's "+use").
#
# Run:  python3 -B tools/art/blender/build.py dining [--only barStool] [--preview]
import math
import os
import sys

import bpy
from mathutils import Matrix, Vector

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402

M = G.Palette(
    # table tops and legs (each its own material, so the player colours them apart)
    top_oak=('#CDB08A', 'wood'),          # white oak tops; the dining sets' chair seats
    legs_oak=('#CDB08A', 'wood'),
    legs_oakd=('#B08E66', 'wood'),        # darker oak legs and frames
    top_walnut=('#7A5234', 'wood'),
    legs_walnut=('#7A5234', 'wood'),
    top_banquet=('#8A5A36', 'wood'),
    legs_banquet=('#6B4A33', 'wood'),
    top_birch=('#E3CFA8', 'wood'),
    legs_birch=('#E3CFA8', 'wood'),
    top_farm=('#B48A5E', 'wood'),         # the farmhouse table's planks
    legs_farm=('#8E6A48', 'wood'),
    groove=('#7A5A3A', 'wood'),           # the joints between its planks
    top_white=('#F1F0EC', 'gloss'),       # the round bistro table
    top_white_satin=('#F1F0EC', 'satin'),  # the square table for eight
    top_black=('#2B2C2E', 'satin'),       # the high café table
    legs_black=('#2B2C2E', 'metal'),      # black metal frames
    legs_chrome=('#D0D3D6', 'chrome'),
    glass=('#DCE6E8', 'glass'),           # the glass table's top
    # the sets' chairs and the table settings
    seat_sage=('#9DAF8C', 'fabric'),      # seat pads
    chair_dark=('#3A3C40', 'metal'),      # the glass set's chair frames
    seat_grey=('#C9CBCC', 'satin'),       # and their seats
    runner=('#E8DCC0', 'fabric'),         # a table runner
    porcelain=('#F5F3EE', 'gloss'),       # plates, a cup
    food=('#E7D9BF', 'gloss'),
    tumbler=('#DCE6EA', 'gloss'),         # drinking glasses (gloss, as the placeholders')
    cutlery=('#B9BDC1', 'satin'),
    vase=('#D8D2C4', 'gloss'),
    stem=('#7E9A62', 'satin'),
    bloom_red=('#D9483B', 'satin'),
    bloom_yellow=('#F2C14E', 'satin'),
    bloom_white=('#F0EDE6', 'satin'),
    # chairs and stools
    frame_cream=('#E4E0D6', 'satin'),     # the painted dining chair
    frame_honey=('#A8754A', 'wood'),      # the wooden chair
    seat_honey=('#A8754A', 'wood'),
    upholstery=('#B7A48A', 'fabric'),     # the upholstered chair
    shell=('#E8E4DC', 'gloss'),           # the moulded chair
    frame_windsor=('#6B4A33', 'wood'),
    seat_windsor=('#6B4A33', 'wood'),
    frame_bent=('#3B2A20', 'wood'),       # the bentwood chair
    seat_bent=('#8E2A2A', 'fabric'),
    frame_steel=('#C3C7CB', 'metal'),     # the folding chair
    seat_teal=('#4E7A7F', 'satin'),
    cap=('#2B2C2E', 'satin'),             # rubber feet
    frame_oak=('#CDB08A', 'wood'),        # the carver chair, the bar stool
    seat_oak=('#CDB08A', 'wood'),         # the bench's seat
    leather=('#C08A52', 'satin'),         # the high bar stool's seat
    # storage
    body_walnut=('#7A5234', 'wood'),      # the sideboard
    top_sideboard=('#7A5234', 'wood'),
    flute=('#5E3F28', 'wood'),            # the fluting on its doors
    brass=('#B8925A', 'metal'),
    cream=('#EFE5D3', 'gloss'),           # a vase
    body_sage=('#9DAF8C', 'satin'),       # the buffet
    body_china=('#F1EDE4', 'satin'),      # the china cabinet
    top_china=('#F1EDE4', 'satin'),
    back_blue=('#9DB7C9', 'satin'),       # its back, behind the glass
    plinth=('#B9A890', 'satin'),
    plate_blue=('#9DC6CF', 'gloss'),
    # the bar cart
    frame_brass=('#B8925A', 'metal'),
    tray=('#DCE6EA', 'glass'),
    castor=('#2B2C2E', 'satin'),
    bottle_green=('#2F5A3A', 'glass'),
    bottle_red=('#7A2E2E', 'glass'),
    bottle_amber=('#C9A64A', 'glass'),
    shaker=('#D0D3D6', 'chrome'),
    basket=('#C9A979', 'satin'),
    lemon=('#F2C14E', 'satin'),
)

F = G.GameFrame()
TT = 0.75     # a dining table's top (catalog2.py TT, content surface)
BT = 1.06     # a bar table's top (catalog2.py BT)
SZ = 0.05     # catalog2's chairs: the seat's middle sits 5 cm forward, towards the table


def _v(p):
    """A game position or direction as a Blender vector."""
    return Vector(F.at(*p))


def _board(name, a, b, w, t, material, face=(0, 1, 0), bevel=0.006):
    """A board from game point `a` to `b`, `w` wide and `t` thick, its broad face turned towards game direction
    `face`: slats, rails and legs at any angle."""
    a, b = _v(a), _v(b)
    z = (b - a).normalized()
    x = _v(face).cross(z).normalized()
    y = z.cross(x)
    obj = G.cube(name, (w, t, (b - a).length), material=material, bevel=bevel)
    rot = Matrix((x, y, z)).transposed()
    obj.matrix_world = Matrix.Translation((a + b) / 2) @ rot.to_4x4()
    return obj


def _rod(name, a, b, r, material, verts=10):
    """A round bar from game point `a` to `b`."""
    return G.rod(name, F.at(*a), F.at(*b), r, material=material, verts=verts)


def _taper(name, x, z, h, r_top, r_foot, material, y0=0.0, square=False, verts=12):
    """An upright leg from `y0` up `h`, `r_top` under the top narrowing to `r_foot` at the floor; `square`: a
    four-sided taper (its radius is to the corners)."""
    if square:
        return F.cone(name, r_foot * 1.414, r_top * 1.414, h, (x, y0 + h / 2, z), material, verts=4, rot=(0, 0, 45),
                      smooth=False)
    return F.cone(name, r_foot, r_top, h, (x, y0 + h / 2, z), material, verts=verts)


def _ring(name, r, minor, y, x, z, material, segments=24):
    """A flat ring (a foot ring, a rim) in the floor's plane."""
    return G.torus(name, r, minor, at=F.at(x, y, z), material=material, major_segments=segments, minor_segments=6)


def _hoop(name, rx, ry, minor, at, material, segments=24, lean=0.0):
    """An upright ring in the front's plane, `rx` across and `ry` high (a chair's back loop), leant back `lean`
    degrees."""
    obj = G.torus(name, rx, minor, at=F.at(*at), material=material, rot=(90 - lean, 0, 0),
                  major_segments=segments, minor_segments=6)
    obj.scale = (1, ry / rx, 1)
    return obj


def _tube(name, pts, r, material, verts=8):
    """A smooth round tube of radius `r` along game points `pts` (a bent rail), its ends capped."""
    path = [_v(p) for p in pts]
    ref = Vector((0, 1, 0))                     # Blender depth: across the bent rail's plane
    vs, faces = [], []
    for i, p in enumerate(path):
        t = (path[min(i + 1, len(path) - 1)] - path[max(i - 1, 0)]).normalized()
        u = t.cross(ref).normalized()
        w = t.cross(u)
        for k in range(verts):
            a = 2 * math.pi * k / verts
            vs.append(p + r * (math.cos(a) * u + math.sin(a) * w))
    for i in range(len(path) - 1):
        for k in range(verts):
            a, b = i * verts + k, i * verts + (k + 1) % verts
            faces.append((a, b, b + verts, a + verts))
    last = (len(path) - 1) * verts
    faces += [tuple(range(verts - 1, -1, -1)), tuple(range(last, last + verts))]
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata([tuple(v) for v in vs], [], faces)
    mesh.validate()
    for poly in mesh.polygons[:-2]:
        poly.use_smooth = True
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    obj.data.update()
    return obj


def _bow(prefix, rx, y0, ry, z0, back, r, material, n=24):
    """A bow: a half ellipse `rx` across, from height `y0` up `ry`, leaning back `back` metres at its top from
    depth `z0`, bent from one tube; returns its height at x."""
    pts = []
    for i in range(n + 1):
        t = math.pi * i / n
        y = y0 + ry * math.sin(t)
        pts.append((rx * math.cos(t), y, z0 - back * (y - y0) / ry))
    _tube(prefix, pts, r, material)
    return lambda x: (y0 + ry * math.sqrt(max(0.0, 1 - (x / rx) ** 2)))


def _finish(name):
    G.join_all(name)
    G.settle(centre=False)
    return G.meshes()


# === K7: dining tables ===============================================================================================
# --- table settings, vases, a runner ---------------------------------------------------------------------------------
def _place(prefix, x, z, y, s=1):
    """A place setting at (x, z) on a top at height `y`: a plate with food, a glass to the side (`s`), cutlery."""
    F.cyl(f'{prefix}plate', 0.12, 0.012, (x, y + 0.006, z), M.porcelain, verts=20, bevel=0.003)
    F.cyl(f'{prefix}food', 0.075, 0.018, (x, y + 0.02, z), M.food, verts=16, bevel=0.006)
    F.cone(f'{prefix}glass', 0.026, 0.034, 0.1, (x + s * 0.17, y + 0.05, z + 0.1), M.tumbler, verts=12)
    for dx in (-0.15, 0.15):
        F.box(f'{prefix}cutlery{dx}', 0.016, 0.004, 0.17, (x + dx, y + 0.002, z), M.cutlery, bevel=0.001)


def _flowers(prefix, x, z, y, h=0.18):
    """A vase on a top at height `y`, with a few stems and blooms."""
    G.revolve(f'{prefix}vase', [(0, 0), (0.035, 0), (0.05, 0.05), (0.045, 0.11), (0.028, h - 0.02), (0.032, h),
                                (0.026, h), (0, h - 0.01)], at=F.at(x, y, z), material=M.vase, segments=14)
    blooms = ((-0.03, 0.1, 0.0, M.bloom_red), (0.035, 0.11, -0.02, M.bloom_yellow), (0.0, 0.09, 0.04, M.bloom_white),
              (0.02, 0.06, 0.03, M.bloom_red), (-0.04, 0.07, -0.03, M.bloom_white))
    for k, (dx, dh, dz, m) in enumerate(blooms):
        _rod(f'{prefix}stem{k}', (x, y + h - 0.04, z), (x + dx, y + h + dh, z + dz), 0.004, M.stem, verts=5)
        F.sphere(f'{prefix}bloom{k}', 0.028, (x + dx, y + h + dh, z + dz), m, scale=(1, 1, 0.8), segments=8, rings=6)


def _ladder_chair(prefix, x, z, frame, seat, pad, seat_h=0.46, back_h=0.92, facing=1, metal=False):
    """A dining set's chair, its seat centred on (x, z) and its sitter facing +z (`facing` -1: -z): a seat board
    and pad, tapered front legs, back legs rising and leaning into a top rail and two ladder rails."""
    def at(dx, y, dz):
        return (x + dx * facing, y, z + dz * facing)
    F.box(f'{prefix}seat', 0.42, 0.04, 0.4, at(0, seat_h - 0.04, 0), seat, bevel=0.012)
    F.box(f'{prefix}pad', 0.38, 0.03, 0.36, at(0, seat_h - 0.005, 0.01), pad, bevel=0.012, bevel_segments=3)
    for s in (-1, 1):
        if metal:
            _rod(f'{prefix}front{s}', at(s * 0.18, 0, 0.17), at(s * 0.18, seat_h - 0.06, 0.16), 0.013, frame, verts=8)
            _rod(f'{prefix}back_leg{s}', at(s * 0.18, 0, -0.15), at(s * 0.18, back_h, -0.2), 0.013, frame, verts=8)
        else:
            _taper(f'{prefix}front{s}', x + s * 0.18 * facing, z + 0.17 * facing, seat_h - 0.06, 0.02, 0.014, frame,
                   square=True)
            _board(f'{prefix}back_leg{s}', at(s * 0.18, 0, -0.15), at(s * 0.18, back_h, -0.2), 0.035, 0.035, frame,
                   face=(0, 0, facing))
        _board(f'{prefix}side{s}', at(s * 0.18, 0.16, -0.15), at(s * 0.18, 0.16, 0.17), 0.02, 0.02, frame)
    for k, (y, h) in enumerate(((back_h - 0.07, 0.1), (back_h - 0.25, 0.05), (back_h - 0.38, 0.05))):
        dz = -0.15 - 0.05 * y / back_h                                  # on the back legs, leaning with them
        _board(f'{prefix}rail{k}', at(-0.18, y, dz), at(0.18, y, dz), h, 0.022, seat if k == 0 else frame,
               face=(0, 0.1, facing), bevel=0.008)
    _board(f'{prefix}stretcher', at(-0.18, 0.16, 0.17), at(0.18, 0.16, 0.17), 0.02, 0.02, frame)


def _set_chair(prefix, x, z, frame, seat, pad, metal=False):
    _ladder_chair(prefix, x, z, frame, seat, pad, metal=metal)


# --- the dining table (base manifest): an oak top to 0.765 on tapered legs, two chairs behind it --------------------
def diningTable():
    y = 0.765
    F.box('top', 1.62, 0.04, 0.38, (0, y - 0.02, 0.26), M.top_oak, bevel=0.01)
    F.box('apron', 1.5, 0.07, 0.3, (0, y - 0.075, 0.27), M.legs_oakd, bevel=0.006)
    for x in (-0.76, 0.76):
        for z in (0.1, 0.42):
            _taper(f'leg{x}{z}', x, z, y - 0.04, 0.025, 0.017, M.legs_oakd, verts=12)
    F.box('runner', 0.22, 0.004, 0.38, (0, y + 0.002, 0.26), M.runner)
    _flowers('', 0, 0.33, y + 0.004, h=0.13)
    for s in (-1, 1):
        _place(f'p{s}', s * 0.5, 0.24, y, s)
        _set_chair(f'c{s}', s * 0.5, -0.2, M.legs_oakd, M.top_oak, M.seat_sage)
    return _finish('diningTable')


# --- the bistro set: a round oak café table and a black metal café chair behind it ----------------------------------
def _cafe_chair(prefix, x, z, frame, pad, seat_h=0.46):
    """A café chair facing +z, its seat centred on (x, z): a round padded seat on a metal ring, four legs with a
    ring stretcher, and a looped back."""
    _ring(f'{prefix}seat_ring', 0.2, 0.012, seat_h - 0.045, x, z, frame)
    F.cyl(f'{prefix}seat', 0.205, 0.045, (x, seat_h - 0.0225, z), pad, verts=24, bevel=0.015, bevel_segments=3)
    for k, (sx, sz) in enumerate(((-1, 1), (1, 1), (-1, -1), (1, -1))):
        top = (x + sx * 0.13, seat_h - 0.045, z + sz * 0.13)
        foot = (x + sx * 0.17, 0.0, z + sz * 0.17 + (0.03 if sz < 0 else 0))
        _rod(f'{prefix}leg{k}', foot, top, 0.011, frame, verts=8)
    _ring(f'{prefix}stretcher', 0.16, 0.008, 0.2, x, z + 0.01, frame, segments=20)
    for s in (-1, 1):
        _rod(f'{prefix}post{s}', (x + s * 0.13, seat_h - 0.045, z - 0.13), (x + s * 0.158, 0.75, z - 0.21), 0.011,
             frame, verts=8)
    _hoop(f'{prefix}back', 0.16, 0.15, 0.012, (x, 0.75, z - 0.21), frame, lean=8)
    _hoop(f'{prefix}back_in', 0.1, 0.09, 0.008, (x, 0.75, z - 0.212), frame, segments=18, lean=8)


def bistroSet():
    y = 0.75
    z = 0.17
    F.cyl('top', 0.31, 0.035, (0, y - 0.0175, z), M.top_oak, verts=40, bevel=0.008)
    F.cyl('hub', 0.08, 0.03, (0, y - 0.05, z), M.legs_black, verts=16)
    F.cyl('column', 0.025, y - 0.08, (0, (y - 0.08) / 2 + 0.01, z), M.legs_black, verts=14)
    G.revolve('foot', [(0, 0), (0.21, 0), (0.21, 0.012), (0.12, 0.03), (0.04, 0.05), (0, 0.05)],
              at=F.at(0, 0, z), material=M.legs_black, segments=28)
    # a cup on its saucer and a little vase
    F.cyl('saucer', 0.07, 0.01, (0.12, y + 0.005, 0.24), M.porcelain, verts=18, bevel=0.003)
    F.cone('cup', 0.035, 0.048, 0.07, (0.12, y + 0.045, 0.24), M.porcelain, verts=16)
    G.torus('cup_handle', 0.02, 0.005, at=F.at(0.175, y + 0.05, 0.24), material=M.porcelain, rot=(90, 0, 0),
            major_segments=10, minor_segments=5)
    _flowers('', -0.15, 0.1, y, h=0.12)
    _cafe_chair('c', 0, -0.19, M.legs_black, M.seat_sage)        # its back on the wall face, as the placeholder's
    return _finish('bistroSet')


# --- the glass dining set: a glass top on chrome sled legs, two dark chairs behind it -------------------------------
def diningGlass():
    y = 0.76
    z = 0.16
    F.box('top', 1.7, 0.02, 0.62, (0, y - 0.01, z), M.glass, bevel=0.008)
    for x in (-0.7, 0.7):
        for dz in (-0.22, 0.22):
            _rod(f'post{x}{dz}', (x, 0.02, z + dz), (x, y - 0.03, z + dz), 0.018, M.legs_chrome, verts=12)
            F.sphere(f'corner{x}{dz}', 0.018, (x, y - 0.03, z + dz), M.legs_chrome, segments=10, rings=6)
            F.sphere(f'heel{x}{dz}', 0.018, (x, 0.02, z + dz), M.legs_chrome, segments=10, rings=6)
        _rod(f'foot{x}', (x, 0.02, z - 0.22), (x, 0.02, z + 0.22), 0.018, M.legs_chrome, verts=12)
        _rod(f'cross{x}', (x, y - 0.03, z - 0.22), (x, y - 0.03, z + 0.22), 0.018, M.legs_chrome, verts=12)
        for dz in (-0.18, 0.18):
            F.cyl(f'pad{x}{dz}', 0.02, 0.008, (x, y - 0.016, z + dz), M.legs_chrome, verts=10)
    _rod('rail', (-0.7, y - 0.05, z), (0.7, y - 0.05, z), 0.012, M.legs_chrome, verts=10)
    _flowers('', 0, 0.2, y, h=0.22)
    for s in (-1, 1):
        x = s * 0.5
        _place(f'p{s}', x, 0.06, y, s)
        F.cyl(f'far_plate{s}', 0.12, 0.012, (x, y + 0.006, 0.36), M.porcelain, verts=20, bevel=0.003)
        _set_chair(f'c{s}', x, -0.22, M.chair_dark, M.seat_grey, M.seat_grey, metal=True)
    return _finish('diningGlass')


# --- the farmhouse dining set: a thick oak top on square legs, a runner, two chairs behind it -----------------------
def diningOak():
    y = 0.765
    z = 0.15
    F.box('top', 1.76, 0.05, 0.66, (0, y - 0.025, z), M.top_oak, bevel=0.012)
    for s in (-1, 1):
        F.box(f'cleat{s}', 0.06, 0.05, 0.66, (s * 0.85, y - 0.025, z), M.top_oak, bevel=0.01)   # breadboard ends
        F.box(f'apron{s}', 1.5, 0.08, 0.025, (0, y - 0.09, z + s * 0.26), M.legs_oakd, bevel=0.005)
        F.box(f'apron_end{s}', 0.025, 0.08, 0.5, (s * 0.76, y - 0.09, z), M.legs_oakd, bevel=0.005)
    for x in (-0.79, 0.79):
        for dz in (-0.27, 0.27):
            _taper(f'leg{x}{dz}', x, z + dz, y - 0.05, 0.04, 0.03, M.legs_oakd, square=True)
    F.box('runner', 0.3, 0.004, 0.66, (0, y + 0.002, z), M.runner)
    _flowers('', 0, 0.2, y + 0.004, h=0.2)
    for s in (-1, 1):
        x = s * 0.5
        _place(f'p{s}', x, 0.04, y, s)
        F.cyl(f'far_plate{s}', 0.12, 0.012, (x, y + 0.006, 0.34), M.porcelain, verts=20, bevel=0.003)
        _set_chair(f'c{s}', x, -0.22, M.legs_oakd, M.top_oak, M.seat_sage)
    return _finish('diningOak')


# --- the catalog2 tables: a top at 0.75 on four (or six) legs, aprons under it -------------------------------------
def _rect_table(w, d, top, legs, t=0.045, leg=0.06, inset=0.06, round_legs=False, taper=0.7, apron=True, h=TT):
    F.box('top', w, t, d, (0, h - t / 2, 0), top, bevel=0.012)
    lx, lz = w / 2 - inset - leg / 2, d / 2 - inset - leg / 2
    if apron:
        for s in (-1, 1):
            F.box(f'apron{s}', 2 * lx, 0.08, 0.022, (0, h - t - 0.04, s * (lz - leg / 2 + 0.011)), legs, bevel=0.004)
            F.box(f'apron_end{s}', 0.022, 0.08, 2 * lz, (s * (lx - leg / 2 + 0.011), h - t - 0.04, 0), legs, bevel=0.004)
    xs = (-lx, lx) + ((0.0,) if w > 2.6 else ())
    for x in xs:
        for z in (-lz, lz):
            _taper(f'leg{x}{z}', x, z, h - t, leg / 2, leg / 2 * taper, legs, square=not round_legs)
    return lx, lz


def tableSquare2():
    """Square table for two (catalog2): an oak top on round tapered darker legs, aprons."""
    _rect_table(0.82, 0.82, M.top_oak, M.legs_oakd, round_legs=True, taper=0.75)
    for x in (-0.32, 0.32):
        for z in (-0.32, 0.32):
            _ring(f'collar{x}{z}', 0.03, 0.006, 0.52, x, z, M.legs_oakd, segments=14)
    return _finish('tableSquare2')


def tableDining4():
    """Dining table for four (catalog2): an oak top on square tapered oak legs, aprons."""
    _rect_table(1.84, 0.88, M.top_oak, M.legs_oak, leg=0.065, taper=0.65)
    return _finish('tableDining4')


def tableDining6():
    """Dining table for six (catalog2): a walnut top on six round tapered legs, aprons."""
    lx, lz = _rect_table(2.84, 0.88, M.top_walnut, M.legs_walnut, leg=0.07, round_legs=True, taper=0.7)
    for x in (-lx, 0.0, lx):
        for z in (-lz, lz):
            _ring(f'collar{x}{z}', 0.032, 0.006, 0.5, x, z, M.legs_walnut, segments=14)
    return _finish('tableDining6')


def tableBanquet8():
    """Banquet table (catalog2): a long planked top on six chunky legs with feet, aprons and a stretcher."""
    lx, lz = _rect_table(3.84, 0.88, M.top_banquet, M.legs_banquet, leg=0.08, taper=0.8)
    for z in (-0.22, 0.0, 0.22):
        F.box(f'joint{z}', 3.84, 0.003, 0.005, (0, TT - 0.001, z), M.legs_banquet)
    for x in (-lx, 0.0, lx):
        for z in (-lz, lz):
            F.box(f'foot{x}{z}', 0.075, 0.03, 0.075, (x, 0.015, z), M.legs_banquet, bevel=0.006)
    F.box('stretcher', 2 * lx, 0.05, 0.04, (0, 0.18, 0), M.legs_banquet, bevel=0.006)
    for x in (-lx, 0.0, lx):
        F.box(f'cross{x}', 0.04, 0.05, 2 * lz, (x, 0.18, 0), M.legs_banquet, bevel=0.006)
    return _finish('tableBanquet8')


def tableSquare8():
    """Square table for eight (catalog2): a white top on black metal legs braced by a square frame."""
    _rect_table(1.86, 1.86, M.top_white_satin, M.legs_black, leg=0.06, inset=0.1, apron=False, taper=1.0)
    for s in (-1, 1):
        F.box(f'brace{s}', 1.54, 0.03, 0.03, (0, 0.66, s * 0.8), M.legs_black, bevel=0.006)
        F.box(f'brace_end{s}', 0.03, 0.03, 1.54, (s * 0.8, 0.66, 0), M.legs_black, bevel=0.006)
    for x in (-0.8, 0.8):
        for z in (-0.8, 0.8):
            F.box(f'foot{x}{z}', 0.07, 0.012, 0.07, (x, 0.006, z), M.legs_black, bevel=0.003)
    return _finish('tableSquare8')


def tableFarmhouse6():
    """Farmhouse table (catalog2): a thick planked top on two trestles joined by a stretcher."""
    F.box('top', 2.86, 0.06, 0.88, (0, TT - 0.03, 0), M.top_farm, bevel=0.01)
    for z in (-0.44 / 3, 0.44 / 3):
        F.box(f'joint{z}', 2.86, 0.003, 0.006, (0, TT - 0.001, z), M.groove)
    for x in (-1.15, 1.15):
        F.box(f'post{x}', 0.1, 0.6, 0.1, (x, 0.37, 0), M.legs_farm, bevel=0.012)
        F.box(f'foot{x}', 0.1, 0.07, 0.8, (x, 0.035, 0), M.legs_farm, bevel=0.012)
        F.box(f'cleat{x}', 0.1, 0.06, 0.78, (x, TT - 0.09, 0), M.legs_farm, bevel=0.012)
        for s in (-1, 1):
            _board(f'brace{x}{s}', (x, 0.07, s * 0.3), (x, 0.32, s * 0.05), 0.05, 0.06, M.legs_farm, face=(1, 0, 0))
            F.box(f'pad{x}{s}', 0.11, 0.015, 0.1, (x, 0.0075, s * 0.33), M.legs_farm, bevel=0.004)
    F.box('stretcher', 2.3, 0.08, 0.06, (0, 0.32, 0), M.legs_farm, bevel=0.01)
    for x in (-1.22, 1.22):
        F.box(f'wedge{x}', 0.04, 0.1, 0.03, (x, 0.32, 0), M.groove, bevel=0.006)
    return _finish('tableFarmhouse6')


def _pedestal(h, top_r, top, base, hub_r, col_r, foot_r, foot_h=0.05, t=0.04, verts=40):
    F.cyl('top', top_r, t, (0, h - t / 2, 0), top, verts=verts, bevel=0.008)
    F.cyl('hub', hub_r, 0.03, (0, h - t - 0.015, 0), base, verts=18)
    F.cyl('column', col_r, h - t - 0.03 - foot_h + 0.01, (0, (h - t - 0.03 + foot_h - 0.01) / 2, 0), base, verts=14)
    G.revolve('foot', [(0, 0), (foot_r, 0), (foot_r, 0.012), (foot_r * 0.55, foot_h * 0.6), (col_r * 1.6, foot_h),
                       (0, foot_h)], at=F.at(0, 0, 0), material=base, segments=28)


def tableBistroRound():
    """Round bistro table (catalog2): a white gloss top on a black column and cast foot."""
    _pedestal(TT, 0.36, M.top_white, M.legs_black, 0.1, 0.03, 0.25)
    G.torus('edge', 0.352, 0.006, at=F.at(0, TT - 0.024, 0), material=M.legs_black, major_segments=40,
            minor_segments=4)
    return _finish('tableBistroRound')


def tableBarHigh():
    """High café table (catalog2): a dark top at bar height on a column, a foot ring for the stools."""
    _pedestal(BT, 0.35, M.top_black, M.legs_black, 0.1, 0.035, 0.25)
    _ring('foot_ring', 0.185, 0.013, 0.3, 0, 0, M.legs_black, segments=28)
    for k in range(4):
        a = 45 + 90 * k
        c, s = G.cosd(a), G.sind(a)
        _rod(f'spoke{k}', (c * 0.03, 0.3, s * 0.03), (c * 0.18, 0.3, s * 0.18), 0.008, M.legs_black, verts=8)
    return _finish('tableBarHigh')


def tableRound4():
    """Round pedestal table (catalog2, 2x2): a birch top on a turned column and four splayed feet."""
    F.cyl('top', 0.76, 0.04, (0, TT - 0.02, 0), M.top_birch, verts=56, bevel=0.01)
    F.cyl('apron', 0.3, 0.06, (0, TT - 0.07, 0), M.legs_birch, verts=28)
    G.revolve('column', [(0, 0.08), (0.13, 0.08), (0.13, 0.12), (0.1, 0.16), (0.075, 0.3), (0.09, 0.42),
                         (0.08, 0.5), (0.065, 0.6), (0.09, 0.64), (0.09, 0.67), (0, 0.67)],
              at=F.at(0, 0, 0), material=M.legs_birch, segments=24)
    for k in range(4):
        a = 45 + 90 * k
        c, s = G.cosd(a), G.sind(a)
        _board(f'foot{k}', (c * 0.08, 0.075, s * 0.08), (c * 0.36, 0.045, s * 0.36), 0.07, 0.07, M.legs_birch,
               face=(-s, 0, c), bevel=0.012)
        F.cyl(f'toe{k}', 0.03, 0.02, (c * 0.36, 0.01, s * 0.36), M.legs_birch, verts=12)
    return _finish('tableRound4')


# === K8: chairs and stools ===========================================================================================
def _stool(seat_h, z, frame, seat, legs_r=0.22, back=False, cushion_h=0.04, verts=24, cap=None):
    """A stool on four splayed legs and a foot ring, its seat (and back) a part of its own turning on a hub.
    `cap`: the feet and the swivel plate (the frame's own material on a wood stool, so it draws in one group)."""
    cap = cap or frame
    with G.new_parts() as base:
        for k in range(4):
            a = 45 + 90 * k
            c, s = G.cosd(a), G.sind(a)
            _rod(f'leg{k}', (c * legs_r, 0, z + s * legs_r), (c * 0.12, seat_h - 0.06, z + s * 0.12), 0.017, frame,
                 verts=10)
            F.cyl(f'shoe{k}', 0.02, 0.012, (c * legs_r, 0.006, z + s * legs_r), cap, verts=10)
        ring_y = seat_h * 0.36
        k_r = legs_r - (legs_r - 0.12) * ring_y / (seat_h - 0.06)
        _ring('foot_ring', k_r, 0.012, ring_y, 0, z, frame, segments=28)
        F.cyl('hub', 0.15, 0.02, (0, seat_h - 0.065, z), frame, verts=20, bevel=0.005)
    G.join(base, 'base', origin=(0, 0, 0))
    with G.new_parts() as top:
        F.cyl('swivel', 0.07, 0.012, (0, seat_h - 0.05, z), cap, verts=16)
        F.cyl('seat_board', 0.195, 0.035, (0, seat_h - cushion_h - 0.0175 + 0.01, z), frame, verts=verts, bevel=0.008)
        F.cyl('cushion', 0.185, cushion_h, (0, seat_h - cushion_h / 2, z), seat, verts=verts, bevel=0.016,
              bevel_segments=3)
        if back:
            for s in (-1, 1):
                _rod(f'back_post{s}', (s * 0.12, seat_h - 0.04, z - 0.14), (s * 0.14, seat_h + 0.2, z - 0.175), 0.013,
                     frame, verts=8)
            _board('back', (-0.18, seat_h + 0.24, z - 0.18), (0.18, seat_h + 0.24, z - 0.18), 0.14, 0.03, seat,
                   face=(0, 0.14, 1), bevel=0.014)
    seat_part = G.hinged(G.join(top, 'seat'), F.at(0, seat_h - 0.056, z))
    G.settle(centre=False)
    G.swing(seat_part, 'swivel', 20, seconds=0.5)
    return G.meshes()


def barStool():
    """Bar stool (catalog): an oak stool with a sage seat, top at 0.74 (sitters at 0.72), centred."""
    return _stool(0.74, 0.0, M.frame_oak, M.seat_sage, legs_r=0.21, cushion_h=0.05)


def stoolCounter():
    """Counter stool (catalog2): an oak stool, seat top at 0.68, 5 cm forward."""
    return _stool(0.685, SZ, M.frame_oak, M.seat_sage, legs_r=0.21)


def stoolBarHigh():
    """High bar stool (catalog2): black metal with a cognac seat and a low back, seat top at 0.805."""
    return _stool(0.805, SZ, M.legs_black, M.leather, legs_r=0.21, back=True, cushion_h=0.05, cap=M.cap)


def _legs(prefix, frame, w, d, seat_y, back_h, z=SZ, lean=4, square=True, r=0.018, back_t=0.035):
    """Four legs: tapered front legs, the back legs rising into the back's posts, leaning back `lean` degrees."""
    x = w / 2 - 0.03
    z0, z1 = z - d / 2 + 0.03, z + d / 2 - 0.03
    for s in (-1, 1):
        _taper(f'{prefix}front{s}', s * x, z1, seat_y, r, r * 0.7, frame, square=square)
        top = (s * x, back_h, z0 - back_h * math.tan(math.radians(lean)) + 0.02)
        _board(f'{prefix}back{s}', (s * x, 0, z0 + 0.02), top, back_t, back_t, frame, face=(0, 0, 1))
    return x, z0, z1


def diningChair():
    """Dining chair (catalog): a painted ladder-back chair on turned legs with a tie-on pad, seat top 0.46."""
    z = 0.02
    F.box('seat', 0.44, 0.04, 0.42, (0, 0.44, z), M.frame_cream, bevel=0.012)
    F.box('pad', 0.38, 0.025, 0.36, (0, 0.4725, z + 0.01), M.seat_sage, bevel=0.01, bevel_segments=3)
    for x in (-0.19, 0.19):
        F.cone(f'front{x}', 0.014, 0.019, 0.42, (x, 0.21, 0.19), M.frame_cream, verts=12)
        _ring(f'turn{x}', 0.02, 0.005, 0.3, x, 0.19, M.frame_cream, segments=12)
        _rod(f'back{x}', (x, 0, -0.17), (x * 0.95, 0.96, -0.21), 0.017, M.frame_cream, verts=12)
        F.sphere(f'finial{x}', 0.022, (x * 0.95, 0.975, -0.211), M.frame_cream, scale=(1, 1, 1.2), segments=12, rings=8)
        F.box(f'side{x}', 0.02, 0.02, 0.34, (x, 0.2, 0.01), M.frame_cream, bevel=0.006)
        for dz in (0.05, -0.08):                                             # the pad's ties
            F.box(f'tie{x}{dz}', 0.012, 0.05, 0.008, (x * 1.04 + (0.005 if x > 0 else -0.005), 0.43, z + dz),
                  M.seat_sage)
    for k, y in enumerate((0.6, 0.73, 0.86)):
        zz = -0.17 - (y * 0.04 / 0.96) - 0.0
        _board(f'slat{k}', (-0.18, y, zz), (0.18, y, zz), 0.06, 0.02, M.frame_cream, face=(0, 0.04, 1), bevel=0.008)
    for zz in (-0.16, 0.19):
        F.box(f'stretcher{zz}', 0.36, 0.02, 0.02, (0, 0.16, zz), M.frame_cream, bevel=0.006)
    return _finish('diningChair')


def chairWood():
    """Wooden chair (catalog2): honey wood, a spindle back between two rails, seat top 0.47."""
    x, z0, z1 = _legs('', M.frame_honey, 0.44, 0.42, 0.43, 0.9)
    F.box('seat', 0.44, 0.04, 0.42, (0, 0.45, SZ), M.seat_honey, bevel=0.014)
    for k, (y, h) in enumerate(((0.83, 0.1), (0.6, 0.05))):
        zz = z0 + 0.02 - y * math.tan(math.radians(4))
        _board(f'rail{k}', (-x, y, zz), (x, y, zz), h, 0.028, M.frame_honey, face=(0, 0.07, 1), bevel=0.01)
    for k, sx in enumerate((-0.1, -0.05, 0.0, 0.05, 0.1)):
        za, zb = z0 + 0.02 - 0.62 * 0.07, z0 + 0.02 - 0.79 * 0.07
        _rod(f'spindle{k}', (sx, 0.62, za), (sx, 0.79, zb), 0.009, M.frame_honey, verts=8)
    for s in (-1, 1):
        F.box(f'stretcher{s}', 0.36, 0.022, 0.022, (0, 0.18, SZ + s * 0.18), M.frame_honey, bevel=0.005)
        F.box(f'side{s}', 0.022, 0.022, 0.36, (s * x, 0.24, SZ), M.frame_honey, bevel=0.005)
    return _finish('chairWood')


def chairUpholstered():
    """Upholstered chair (catalog2): a padded seat and back in oatmeal fabric on walnut legs, seat at 0.48."""
    F.box('seat', 0.46, 0.09, 0.44, (0, 0.435, SZ + 0.01), M.upholstery, bevel=0.03, bevel_segments=3)
    F.box('back', 0.44, 0.5, 0.07, (0, 0.74, SZ - 0.19), M.upholstery, rot=(-7, 0, 0), bevel=0.03, bevel_segments=3)
    F.box('back_seam', 0.4, 0.012, 0.074, (0, 0.73, SZ - 0.19), M.upholstery, rot=(-7, 0, 0), bevel=0.005)
    for x in (-0.19, 0.19):
        _taper(f'front{x}', x, SZ + 0.17, 0.39, 0.02, 0.012, M.legs_walnut, verts=12)
        _rod(f'back_leg{x}', (x, 0, SZ - 0.25), (x, 0.4, SZ - 0.17), 0.016, M.legs_walnut, verts=10)
        F.sphere(f'foot{x}', 0.013, (x, 0.012, SZ - 0.25), M.legs_walnut, segments=8, rings=6)
    return _finish('chairUpholstered')


def _arc_back(prefix, n, step, radius, cz, y, h, lean, material, w=None, t=0.035):
    """A curved back from `n` panels round an arc of `radius` about (0, cz), `step` degrees apart."""
    w = w or 2 * radius * G.sind(step / 2) + 0.012
    for k in range(n):
        a = (k - (n - 1) / 2) * step
        F.box(f'{prefix}{k}', w, h, t, (radius * G.sind(a), y, cz - radius * G.cosd(a)), material,
              rot=(-lean, 0, a), bevel=0.004)


def chairModern():
    """Moulded chair (catalog2): a gloss shell, its back curved round the seat, on a black sled frame."""
    F.box('seat', 0.44, 0.04, 0.42, (0, 0.45, SZ + 0.02), M.shell, bevel=0.018, bevel_segments=3)
    _arc_back('back', 9, 10, 0.25, SZ + 0.03, 0.62, 0.34, 10, M.shell)
    for s in (-1, 1):
        x = s * 0.19
        _rod(f'front{s}', (x, 0.012, SZ + 0.17), (x, 0.43, SZ + 0.16), 0.011, M.legs_black, verts=8)
        _rod(f'rear{s}', (x, 0.012, SZ - 0.17), (x, 0.43, SZ - 0.13), 0.011, M.legs_black, verts=8)
        _rod(f'runner{s}', (x, 0.012, SZ - 0.17), (x, 0.012, SZ + 0.17), 0.011, M.legs_black, verts=8)
        _rod(f'under{s}', (x, 0.43, SZ - 0.13), (x, 0.43, SZ + 0.16), 0.01, M.legs_black, verts=8)
        for zz in (SZ - 0.17, SZ + 0.17):
            F.sphere(f'bend{s}{zz}', 0.011, (x, 0.012, zz), M.legs_black, segments=8, rings=6)
    _rod('cross', (-0.19, 0.43, SZ + 0.0), (0.19, 0.43, SZ + 0.0), 0.009, M.legs_black, verts=8)
    return _finish('chairModern')


def chairWindsor():
    """Windsor chair (catalog2): a saddle seat on splayed turned legs and an H-stretcher, a bow back on spindles."""
    G.revolve('seat', [(0, 0), (0.48, 0), (0.5, 0.03), (0.49, 0.045), (0.3, 0.05), (0, 0.042)],
              at=F.at(0, 0.42, SZ), material=M.seat_windsor, segments=28, scale=(0.46, 0.44, 1))
    legs = []
    for sx in (-1, 1):
        for sz in (-1, 1):
            foot = (sx * 0.22, 0, SZ + sz * 0.21)
            _rod(f'leg{sx}{sz}', foot, (sx * 0.15, 0.425, SZ + sz * 0.13), 0.018, M.frame_windsor, verts=10)
            legs.append(foot)
    for sx in (-1, 1):
        y = 0.16
        _rod(f'side{sx}', (sx * 0.195, y, SZ + 0.18), (sx * 0.195, y, SZ - 0.18), 0.011, M.frame_windsor, verts=8)
    _rod('cross', (-0.195, 0.16, SZ), (0.195, 0.16, SZ), 0.011, M.frame_windsor, verts=8)
    # the bow: a half ellipse leaning back, 0.21 across and up to 0.97, on spindles
    height = _bow('bow', 0.21, 0.455, 0.5, SZ - 0.15, 0.08, 0.016, M.frame_windsor)
    for k, sx in enumerate((-0.12, -0.06, 0.0, 0.06, 0.12)):
        y = height(sx)
        _rod(f'spindle{k}', (sx * 0.9, 0.46, SZ - 0.13), (sx, y, SZ - 0.15 - 0.08 * (y - 0.455) / 0.5), 0.008,
             M.frame_windsor, verts=6)
    return _finish('chairWindsor')


def chairBentwood():
    """Bentwood chair (catalog2): dark steam-bent beech, a round seat with a red cane insert, a looped back."""
    F.cyl('seat', 0.21, 0.035, (0, 0.4575, SZ), M.frame_bent, verts=28, bevel=0.01)
    F.cyl('cane', 0.18, 0.012, (0, 0.474, SZ), M.seat_bent, verts=24)
    G.torus('seat_rim', 0.205, 0.012, at=F.at(0, 0.44, SZ), material=M.frame_bent, major_segments=28,
            minor_segments=6)
    _ring('leg_ring', 0.165, 0.012, 0.2, 0, SZ, M.frame_bent, segments=24)
    for sx in (-1, 1):
        _rod(f'front{sx}', (sx * 0.18, 0, SZ + 0.2), (sx * 0.14, 0.44, SZ + 0.13), 0.016, M.frame_bent, verts=10)
        _rod(f'back_leg{sx}', (sx * 0.17, 0, SZ - 0.2), (sx * 0.155, 0.44, SZ - 0.17), 0.015, M.frame_bent, verts=10)
    # the back legs bend on up into the back's loop, and a smaller loop inside it
    _bow('back', 0.155, 0.44, 0.51, SZ - 0.17, 0.05, 0.015, M.frame_bent)
    _hoop('back_in', 0.085, 0.11, 0.011, (0, 0.71, SZ - 0.195), M.frame_bent, segments=20, lean=6)
    for s in (-1, 1):
        _rod(f'tie{s}', (s * 0.085, 0.71, SZ - 0.195), (s * 0.154, 0.7, SZ - 0.196), 0.008, M.frame_bent, verts=6)
    return _finish('chairBentwood')


def chairFolding():
    """Folding chair (catalog2): a steel X-frame with a teal seat and back panel, rubber feet."""
    for s in (-1, 1):
        x = s * 0.215
        _rod(f'front{s}', (x, 0.012, SZ + 0.22), (x, 0.88, SZ - 0.2), 0.011, M.frame_steel, verts=8)
        _rod(f'rear{s}', (x * 0.93, 0.012, SZ - 0.2), (x * 0.93, 0.44, SZ + 0.13), 0.011, M.frame_steel, verts=8)
        F.sphere(f'pivot{s}', 0.014, (x, 0.29, SZ - 0.02), M.frame_steel, scale=(0.8, 1, 1), segments=10, rings=6)
        for zz, xx in ((SZ + 0.22, x), (SZ - 0.2, x * 0.93)):
            F.box(f'foot{s}{zz}', 0.03, 0.02, 0.06, (xx, 0.01, zz), M.cap, bevel=0.006)
        F.sphere(f'cap{s}', 0.012, (x, 0.88, SZ - 0.2), M.frame_steel, segments=8, rings=6)
    F.box('seat', 0.42, 0.03, 0.38, (0, 0.45, SZ + 0.02), M.seat_teal, bevel=0.012)
    _rod('seat_bar', (-0.215, 0.43, SZ + 0.18), (0.215, 0.43, SZ + 0.18), 0.009, M.frame_steel, verts=8)
    F.box('back', 0.41, 0.2, 0.022, (0, 0.77, SZ - 0.17), M.seat_teal, rot=(-25, 0, 0), bevel=0.01)
    _rod('stretcher', (-0.2, 0.1, SZ + 0.15), (0.2, 0.1, SZ + 0.15), 0.009, M.frame_steel, verts=8)
    return _finish('chairFolding')


def chairArm():
    """Carver chair (catalog2): an oak armchair, padded sage seat and back, seat at 0.5."""
    x, z0, z1 = _legs('', M.frame_oak, 0.54, 0.42, 0.43, 0.92, back_t=0.04, r=0.021)
    F.box('seat', 0.54, 0.05, 0.44, (0, 0.44, SZ), M.frame_oak, bevel=0.012)
    F.box('cushion', 0.46, 0.06, 0.4, (0, 0.49, SZ + 0.01), M.seat_sage, bevel=0.022, bevel_segments=3)
    F.box('back_pad', 0.42, 0.3, 0.05, (0, 0.72, SZ - 0.17), M.seat_sage, rot=(-6, 0, 0), bevel=0.02,
          bevel_segments=3)
    zz = z0 + 0.02 - 0.88 * math.tan(math.radians(4))
    _board('top_rail', (-x - 0.01, 0.88, zz), (x + 0.01, 0.88, zz), 0.07, 0.035, M.frame_oak, face=(0, 0.07, 1),
           bevel=0.012)
    for s in (-1, 1):
        F.box(f'arm{s}', 0.06, 0.03, 0.46, (s * 0.255, 0.66, SZ - 0.02), M.frame_oak, bevel=0.01)
        F.box(f'arm_post{s}', 0.03, 0.2, 0.03, (s * 0.255, 0.55, SZ + 0.18), M.frame_oak, bevel=0.006)
        F.box(f'stretcher{s}', 0.025, 0.025, 0.36, (s * x, 0.18, SZ), M.frame_oak, bevel=0.005)
    F.box('stretcher_front', 0.46, 0.025, 0.025, (0, 0.2, SZ + 0.18), M.frame_oak, bevel=0.005)
    return _finish('chairArm')


def benchDining():
    """Dining bench (catalog2, 2x1, two sitters): an oak plank on splayed darker legs, a rail under it."""
    F.box('seat', 1.6, 0.05, 0.36, (0, 0.44, SZ), M.seat_oak, bevel=0.012)
    F.box('rail', 1.4, 0.06, 0.03, (0, 0.385, SZ), M.legs_oakd, bevel=0.008)
    for x in (-0.68, 0.68):
        s = 1 if x > 0 else -1
        F.box(f'cleat{x}', 0.07, 0.035, 0.3, (x, 0.3975, SZ), M.legs_oakd, bevel=0.008)
        for dz in (-0.12, 0.12):
            _board(f'leg{x}{dz}', (x + s * 0.05, 0, SZ + dz * 1.25), (x, 0.415, SZ + dz), 0.045, 0.045, M.legs_oakd,
                   face=(0, 0, 1), bevel=0.008)
        _board(f'stretcher{x}', (x + s * 0.025, 0.15, SZ - 0.135), (x + s * 0.025, 0.15, SZ + 0.135), 0.03, 0.025,
               M.legs_oakd, bevel=0.005)
    return _finish('benchDining')


# === K9: dining storage ===============================================================================================
WALL = -0.43   # where the storage is built from (game depth); `settle()` then moves it to its placeholder's centre
LEG_H = 0.16   # a sideboard's legs


def _cabinet(w, h, d, body, top, legs, top_t=0.03, top_over=0.01):
    """A cabinet on four round tapered legs, its back on the wall face, a top over it; returns the front's depth."""
    z = WALL + d / 2
    F.box('carcass', w, h - LEG_H - top_t, d, (0, LEG_H + (h - LEG_H - top_t) / 2, z), body, bevel=0.01)
    F.box('top', w + 2 * top_over, top_t, d + top_over, (0, h - top_t / 2, z + top_over / 2), top, bevel=0.008)
    for x in (-w / 2 + 0.08, w / 2 - 0.08):
        for zz in (WALL + 0.06, WALL + d - 0.06):
            _taper(f'leg{x}{zz}', x, zz, LEG_H, 0.022, 0.014, legs, verts=12)
    return WALL + d


def _door(name, x0, x1, y0, y1, zf, material, hinge_left, build=None, frame=0.0):
    """A door from x0 to x1 (game), `y0` to `y1`, its back on the front face `zf`: a part of its own, hinged on
    its outer edge, still (no use opens it yet). `frame`: only a frame that wide (a glazed door)."""
    with G.new_parts() as parts:
        xm, ym, w, h = (x0 + x1) / 2, (y0 + y1) / 2, x1 - x0, y1 - y0
        if frame:
            for s in (-1, 1):
                F.box(f'{name}_stile{s}', frame, h, 0.02, (xm + s * (w - frame) / 2, ym, zf + 0.01), material,
                      bevel=0.005)
                F.box(f'{name}_rail{s}', w - 2 * frame + 0.004, frame, 0.02, (xm, ym + s * (h - frame) / 2, zf + 0.01),
                      material, bevel=0.005)
        else:
            F.box(name, w, h, 0.02, (xm, ym, zf + 0.01), material, bevel=0.006)
        if build:
            build(x0, x1, y0, y1, zf + 0.02)
    door = G.join(parts, name)
    return G.hinged(door, F.at(x0 if hinge_left else x1, (y0 + y1) / 2, zf + 0.01))


def sideboard():
    """Walnut sideboard (catalog2, 2x1): four fluted doors with brass knobs on tapered legs, a vase on top."""
    w, h, d = 1.8, 0.8, 0.62
    with G.new_parts() as still:
        zf = _cabinet(w, h, d, M.body_walnut, M.top_sideboard, M.body_walnut)
        F.cyl('vase_foot', 0.05, 0.02, (-0.6, h + 0.01, -0.3), M.cream, verts=16)
        G.revolve('vase', [(0, 0), (0.06, 0), (0.07, 0.08), (0.05, 0.2), (0.035, 0.27), (0.04, 0.3), (0.033, 0.3),
                           (0, 0.28)], at=F.at(-0.6, h, -0.3), material=M.cream, segments=18)
    G.join(still, 'body', origin=(0, 0, 0))
    n, y0, y1 = 4, LEG_H + 0.02, h - 0.05
    dw = (w - 0.04) / n

    def flutes(x0, x1, y0, y1, zf):
        for k in range(-3, 4):
            F.box(f'flute{x0}{k}', 0.012, y1 - y0 - 0.06, 0.008, ((x0 + x1) / 2 + k * dw / 8, (y0 + y1) / 2, zf + 0.002),
                  M.flute, bevel=0.002)

    for i in range(n):
        x0 = -w / 2 + 0.02 + dw * i + 0.006
        x1 = x0 + dw - 0.012
        left = i % 2 == 0

        def knob(a, b, c, e, zf, left=left):
            F.sphere(f'knob{a}', 0.016, ((b - 0.05) if left else (a + 0.05), (c + e) / 2, zf + 0.012), M.brass,
                     segments=10, rings=6)
            flutes(a, b, c, e, zf)
        _door(f'door{i}', x0, x1, y0, y1, zf, M.body_walnut, left, knob)
    G.settle()                       # where its placeholder stands, against the wall (its Model's centre)
    return G.meshes()


def buffet():
    """Painted buffet (catalog2, 2x1): a sage cabinet, three drawers over five doors, an oak top and legs."""
    w, h, d = 1.84, 0.9, 0.62
    ym = h - 0.035 - 0.15
    with G.new_parts() as still:
        zf = _cabinet(w, h, d, M.body_sage, M.top_oak, M.legs_oak, top_t=0.035, top_over=0.02)
        for x in (-0.6, 0.0, 0.6):
            F.box(f'drawer{x}', 0.56, 0.13, 0.02, (x, ym + 0.075, zf + 0.01), M.body_sage, bevel=0.006)
            F.box(f'pull{x}', 0.14, 0.014, 0.018, (x, ym + 0.075, zf + 0.028), M.brass, bevel=0.005)
        for x in (-0.3, 0.3):
            F.box(f'stile{x}', 0.03, ym - LEG_H - 0.02, 0.01, (x, (ym + LEG_H) / 2, zf + 0.005), M.body_sage)
    G.join(still, 'body', origin=(0, 0, 0))

    def panel(x0, x1, y0, y1, zf, left):
        F.box(f'panel{x0}', x1 - x0 - 0.07, y1 - y0 - 0.08, 0.008, ((x0 + x1) / 2, (y0 + y1) / 2, zf + 0.002),
              M.body_sage, bevel=0.003)
        F.sphere(f'knob{x0}', 0.016, ((x1 - 0.04) if left else (x0 + 0.04), y1 - 0.09, zf + 0.012), M.brass,
                 segments=10, rings=6)
    y0, y1 = LEG_H + 0.02, ym - 0.01
    doors = [(-0.89, -0.605, True), (-0.595, -0.31, False), (-0.285, 0.285, True), (0.31, 0.595, True),
             (0.605, 0.89, False)]
    for i, (x0, x1, left) in enumerate(doors):
        _door(f'door{i}', x0, x1, y0, y1, zf, M.body_sage, left,
              lambda a, b, c, e, z, left=left: panel(a, b, c, e, z, left))
    G.settle()                       # where its placeholder stands, against the wall (its Model's centre)
    return G.meshes()


def chinaCabinet():
    """China cabinet (catalog2): a cream two-door base and a glazed hutch with plates on show, a crown on top."""
    z_base, z_hutch = WALL + 0.24, WALL + 0.18
    with G.new_parts() as still:
        F.box('plinth', 0.94, 0.06, 0.46, (0, 0.03, z_base - 0.01), M.plinth, bevel=0.006)
        F.box('base', 0.94, 0.8, 0.48, (0, 0.46, z_base), M.body_china, bevel=0.012)
        F.box('worktop', 0.98, 0.04, 0.51, (0, 0.88, z_base + 0.015), M.top_china, bevel=0.008)
        for s in (-1, 1):                                                  # two base doors with brass knobs
            F.box(f'door{s}', 0.45, 0.7, 0.02, (s * 0.235, 0.46, z_base + 0.25), M.body_china, bevel=0.008)
            F.box(f'door_panel{s}', 0.37, 0.6, 0.008, (s * 0.235, 0.46, z_base + 0.262), M.body_china, bevel=0.003)
            F.sphere(f'knob{s}', 0.016, (s * 0.035, 0.6, z_base + 0.272), M.brass, segments=10, rings=6)
        # the hutch: sides, top, back, shelves of plates
        hy0, hy1 = 0.9, 2.04
        F.box('hutch_back', 0.88, hy1 - hy0, 0.02, (0, (hy0 + hy1) / 2, WALL + 0.01), M.back_blue)
        for s in (-1, 1):
            F.box(f'hutch_side{s}', 0.03, hy1 - hy0, 0.36, (s * 0.445, (hy0 + hy1) / 2, z_hutch), M.body_china,
                  bevel=0.006)
        F.box('crown', 0.98, 0.06, 0.42, (0, 2.07, z_hutch + 0.01), M.top_china, bevel=0.01)
        F.box('cornice', 0.94, 0.03, 0.39, (0, 2.025, z_hutch), M.top_china, bevel=0.006)
        for k, y in enumerate((1.04, 1.39, 1.73)):
            F.box(f'shelf{k}', 0.86, 0.02, 0.32, (0, y, z_hutch - 0.01), M.body_china, bevel=0.004)
            for j, x in enumerate((-0.3, -0.1, 0.1, 0.3)):
                m = M.porcelain if (j + k) % 2 == 0 else M.plate_blue
                F.cyl(f'plate{k}{j}', 0.085, 0.012, (x, y + 0.095, WALL + 0.07), m, verts=18, rot=(-78, 0, 0),
                      bevel=0.003)
            F.box(f'rail{k}', 0.86, 0.012, 0.01, (0, y + 0.04, WALL + 0.11), M.body_china)
    G.join(still, 'body', origin=(0, 0, 0))
    zf = z_hutch + 0.18

    def glazing(x0, x1, y0, y1, zf):
        F.box(f'pane{x0}', x1 - x0 - 0.06, y1 - y0 - 0.06, 0.006, ((x0 + x1) / 2, (y0 + y1) / 2, zf - 0.01), M.glass)
        F.box(f'bar{x0}', x1 - x0 - 0.06, 0.02, 0.012, ((x0 + x1) / 2, (y0 + y1) / 2 + 0.17, zf - 0.01),
              M.body_china, bevel=0.003)
    for s, (x0, x1) in ((-1, (-0.43, -0.005)), (1, (0.005, 0.43))):
        def glazed(a, b, c, e, z, s=s):
            glazing(a, b, c, e, z)
            F.sphere(f'knob_g{s}', 0.014, (s * 0.04, 1.45, z + 0.01), M.brass, segments=10, rings=6)
        _door(f'glass_door{s}', x0, x1, 0.93, 2.0, zf, M.body_china, s < 0, glazed, frame=0.045)
    G.settle()                       # where its placeholder stands, against the wall (its Model's centre)
    return G.meshes()


def barCart():
    """Bar cart (catalog2): two glass tiers in brass frames on castors, bottles and glasses; the shaker lifts
    and tips while a resident mixes a drink."""
    with G.new_parts() as still:
        for k, y in enumerate((0.3, 0.76)):
            F.box(f'tray{k}', 0.7, 0.012, 0.4, (0, y, 0), M.tray, bevel=0.004)
            # the frame: a rounded rectangle of brass tube round the glass
            for s in (-1, 1):
                _rod(f'rim_long{k}{s}', (-0.33, y + 0.012, s * 0.2), (0.33, y + 0.012, s * 0.2), 0.011, M.frame_brass)
                _rod(f'rim_end{k}{s}', (s * 0.35, y + 0.012, -0.18), (s * 0.35, y + 0.012, 0.18), 0.011, M.frame_brass)
                for e in (-1, 1):
                    G.torus(f'rim_corner{k}{s}{e}', 0.02, 0.011, at=F.at(s * 0.33, y + 0.012, e * 0.18),
                            material=M.frame_brass, major_segments=12, minor_segments=6)
        for x in (-0.35, 0.35):
            for z in (-0.2, 0.2):
                _rod(f'post{x}{z}', (x, 0.06, z), (x, 0.8, z), 0.011, M.frame_brass)
                F.sphere(f'ball{x}{z}', 0.016, (x, 0.8, z), M.frame_brass, segments=10, rings=6)
                F.cyl(f'fork{x}{z}', 0.018, 0.02, (x, 0.055, z), M.frame_brass, verts=10)
                F.cyl(f'castor{x}{z}', 0.025, 0.02, (x, 0.025, z), M.castor, rot=(0, 90, 0), verts=14)
        for s in (-1, 1):
            _rod(f'handle_arm{s}', (-0.35, 0.8, s * 0.2), (-0.39, 0.84, s * 0.18), 0.009, M.frame_brass, verts=8)
        _rod('handle', (-0.39, 0.84, -0.2), (-0.39, 0.84, 0.2), 0.012, M.frame_brass)
        # bottles on the top tier
        bottle = [(0, 0), (0.035, 0), (0.035, 0.17), (0.014, 0.22), (0.011, 0.29), (0.013, 0.3), (0, 0.3)]
        for i, (x, m) in enumerate(((-0.24, M.bottle_green), (-0.15, M.bottle_red), (-0.06, M.bottle_amber))):
            G.revolve(f'bottle{i}', bottle, at=F.at(x, 0.766, -0.08), material=m, segments=12)
            F.cyl(f'cork{i}', 0.012, 0.03, (x, 0.766 + 0.31, -0.08), M.basket, verts=8)
        # two wine glasses
        for i, (x, z) in enumerate(((0.2, 0.1), (0.28, 0.0))):
            G.revolve(f'wine_glass{i}', [(0, 0), (0.03, 0), (0.03, 0.004), (0.005, 0.01), (0.004, 0.08),
                                         (0.035, 0.11), (0.04, 0.16), (0.036, 0.16), (0.03, 0.12), (0, 0.1)],
                      at=F.at(x, 0.766, z), material=M.glass, segments=12)
        # a lemon dish on the top tier; an ice bucket and a basket on the lower one
        F.cyl('dish', 0.06, 0.015, (0.1, 0.774, -0.1), M.porcelain, verts=14, bevel=0.004)
        for i, (dx, dz) in enumerate(((0, 0), (0.035, 0.025))):
            F.sphere(f'lemon{i}', 0.025, (0.1 + dx, 0.8, -0.1 + dz), M.lemon, scale=(1, 1, 0.85), segments=10,
                     rings=6)
        G.revolve('ice_bucket', [(0, 0), (0.065, 0), (0.08, 0.15), (0.075, 0.15), (0.06, 0.012), (0, 0.012)],
                  at=F.at(0.0, 0.306, 0.0), material=M.shaker, segments=18)
        F.box('basket', 0.18, 0.1, 0.16, (0.2, 0.356, 0.05), M.basket, bevel=0.02)
    G.join(still, 'cart', origin=(0, 0, 0))
    # the cocktail shaker, standing at the front of the top tier, in the middle, before the resident who
    # mixes from the front: its own part, pivoting at its foot, tipping towards them
    sx, sz = 0.0, 0.1
    with G.new_parts() as shake:
        G.revolve('shaker', [(0, 0), (0.035, 0), (0.04, 0.12), (0.036, 0.13), (0.03, 0.17), (0.016, 0.2),
                             (0.012, 0.215), (0, 0.22)], at=F.at(sx, 0.766, sz), material=M.shaker, segments=16)
    shaker = G.hinged(G.join(shake, 'shaker'), F.at(sx, 0.766, sz))
    G.settle(centre=False)
    G.slide(shaker, 'shake', (0, 0, 0.1), seconds=0.6)
    G.swing(shaker, 'shake', 30, seconds=0.6, axis='X')
    return G.meshes()


DINING_TOP = [('top_oak', 'Top'), ('legs_oakd', 'Legs')]
SEAT = [('frame_oak', 'Frame'), ('seat_sage', 'Seat')]
STOOL = {'uses': {'sit': 'swivel', 'drink': 'swivel'}, 'hold': 0.6, 'ride': 'swivel'}

FAMILY = G.Family(
    'dining', M,
    [  # K7: dining tables
     G.Model('diningTable', diningTable, size=(1.62, 1.032, 0.867), centre=(0, 0.016), key='model.diningTable',
             paint=DINING_TOP),
     G.Model('bistroSet', bistroSet, size=(0.62, 1.004, 0.912), key='model.bistroSet', centre=(0, 0.024),
             paint=[('top_oak', 'Top'), ('legs_black', 'Frame'), ('seat_sage', 'Seats')]),
     G.Model('diningGlass', diningGlass, size=(1.7, 1.113, 0.913), centre=(0, 0.024), key='model.diningGlass',
             paint=[('glass', 'Top'), ('legs_chrome', 'Legs')]),
     G.Model('diningOak', diningOak, size=(1.76, 1.102, 0.917), centre=(0, 0.021), key='model.diningOak',
             paint=DINING_TOP),
     G.Model('tableSquare2', tableSquare2, size=(0.82, 0.75, 0.82), key='model.tableSquare2', paint=DINING_TOP),
     G.Model('tableBistroRound', tableBistroRound, size=(0.72, 0.75, 0.72), key='model.tableBistroRound',
             paint=[('top_white', 'Top'), ('legs_black', 'Legs')]),
     G.Model('tableDining4', tableDining4, size=(1.84, 0.75, 0.88), key='model.tableDining4',
             paint=[('top_oak', 'Top'), ('legs_oak', 'Legs')]),
     G.Model('tableDining6', tableDining6, size=(2.84, 0.75, 0.88), key='model.tableDining6',
             paint=[('top_walnut', 'Top'), ('legs_walnut', 'Legs')]),
     G.Model('tableBanquet8', tableBanquet8, size=(3.84, 0.751, 0.88), key='model.tableBanquet8',
             paint=[('top_banquet', 'Top'), ('legs_banquet', 'Legs')]),
     G.Model('tableRound4', tableRound4, size=(1.52, 0.75, 1.52), key='model.tableRound4',
             paint=[('top_birch', 'Top'), ('legs_birch', 'Legs')]),
     G.Model('tableSquare8', tableSquare8, size=(1.86, 0.75, 1.86), key='model.tableSquare8',
             paint=[('top_white_satin', 'Top'), ('legs_black', 'Legs')]),
     G.Model('tableFarmhouse6', tableFarmhouse6, size=(2.86, 0.751, 0.88), key='model.tableFarmhouse6',
             paint=[('top_farm', 'Top'), ('legs_farm', 'Legs')]),
     G.Model('tableBarHigh', tableBarHigh, size=(0.7, 1.06, 0.7), key='model.tableBarHigh',
             paint=[('top_black', 'Top'), ('legs_black', 'Legs')]),
     # K8: chairs and stools
     G.Model('barStool', barStool, size=(0.39, 0.742, 0.39), key='model.barStool', motions=STOOL, paint=SEAT),
     G.Model('stoolCounter', stoolCounter, size=(0.39, 0.687, 0.39), centre=(0, 0.05), key='model.stoolCounter',
             motions=STOOL, paint=SEAT),
     G.Model('stoolBarHigh', stoolBarHigh, size=(0.39, 1.117, 0.398), centre=(0, 0.046), key='model.stoolBarHigh',
             motions=STOOL, paint=[('legs_black', 'Frame'), ('leather', 'Seat')]),
     G.Model('diningChair', diningChair, size=(0.44, 1.002, 0.463), centre=(0, -0.002), key='model.diningChair',
             paint=[('frame_cream', 'Frame'), ('seat_sage', 'Seat')]),
     G.Model('chairWood', chairWood, size=(0.44, 0.902, 0.45), centre=(0, 0.035), key='model.chairWood',
             paint=[('frame_honey', 'Frame'), ('seat_honey', 'Seat')]),
     G.Model('chairUpholstered', chairUpholstered, size=(0.46, 0.992, 0.496), centre=(0, 0.032),
             key='model.chairUpholstered', paint=[('legs_walnut', 'Frame'), ('upholstery', 'Upholstery')]),
     G.Model('chairModern', chairModern, size=(0.44, 0.789, 0.496), centre=(0, 0.032), key='model.chairModern',
             paint=[('legs_black', 'Frame'), ('shell', 'Seat')]),
     G.Model('chairWindsor', chairWindsor, size=(0.475, 0.975, 0.474), centre=(0, 0.041), key='model.chairWindsor',
             paint=[('frame_windsor', 'Frame'), ('seat_windsor', 'Seat')]),
     G.Model('chairBentwood', chairBentwood, size=(0.434, 0.968, 0.452), centre=(0, 0.041), key='model.chairBentwood',
             paint=[('frame_bent', 'Frame'), ('seat_bent', 'Seat')]),
     G.Model('chairFolding', chairFolding, size=(0.46, 0.892, 0.48), centre=(0, 0.06), key='model.chairFolding',
             paint=[('frame_steel', 'Frame'), ('seat_teal', 'Seat')]),
     G.Model('chairArm', chairArm, size=(0.57, 0.922, 0.47), centre=(0, 0.035), key='model.chairArm', paint=SEAT),
     G.Model('benchDining', benchDining, size=(1.6, 0.468, 0.36), centre=(0, 0.05), key='model.benchDining',
             paint=[('legs_oakd', 'Frame'), ('seat_oak', 'Seat')]),
     # K9: dining storage
     G.Model('sideboard', sideboard, size=(1.82, 1.1, 0.668), centre=(0, -0.096), key='model.sideboard',
             paint=[('body_walnut', 'Body'), ('top_sideboard', 'Top'), ('brass', 'Handles')]),
     G.Model('buffet', buffet, size=(1.88, 0.9, 0.668), centre=(0, -0.096), key='model.buffet',
             paint=[('body_sage', 'Body'), ('top_oak', 'Top'), ('brass', 'Handles')]),
     G.Model('chinaCabinet', chinaCabinet, size=(0.98, 2.1, 0.548), centre=(0, -0.156), key='model.chinaCabinet',
             paint=[('body_china', 'Body'), ('top_china', 'Top'), ('brass', 'Handles')]),
     G.Model('barCart', barCart, size=(0.769, 1.09, 0.45), centre=(-0.017, 0), key='model.barCart',
             motions={'uses': {'mix': 'shake'}, 'hold': 1.5},
             paint=[('frame_brass', 'Frame'), ('tray', 'Trays')])],
    comment='Blender-built dining tables, chairs, stools and storage (tools/art/blender/families/dining.py). Replaces '
            'the primitive-part placeholders by key; listed in the base manifest\'s packs. The placeholder is kept as a '
            'fallback if a file fails to load, and the materials map onto the game\'s finishes so a model costs one '
            'draw call per finish group. Table tops stand at the content\'s surface heights and seats where the game '
            'seats residents; the stools\' seats swivel 20 degrees as a resident sits (`swivel`, carrying them), and '
            'the bar cart\'s cocktail shaker lifts and tips while a drink is mixed (`shake`).',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

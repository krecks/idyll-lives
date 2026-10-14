# The kids' things (plan/04-kids.md), over their primitive-part placeholders (`catalog.py`, `catalog2.py`,
# `furniture.py`), which stay the fallback. Notes: kids.md.
#
# KD1 baby and kids' beds: the crib with its changing table (`model.crib`), the kids' bed (`model.kidsBed`),
#   the race car bed (`model.kidsBedCar`) and the high chair (`model.highChair`).
# KD2 toys: the toy box and toy chest, the doll house, the rocking horse, the play tent, the ball pit, the
#   train table, the play mat, the giant teddy bear and the kids' bean bag.
# KD3 kids' study and art: the kids' desk, table and chair, the picture-book shelf, the chalkboard easel and
#   the kids' easel.
#
# Each is built where its placeholder stands (`F`, game axes: x, height, depth; front +depth), then stood on
# the floor (`G.settle(centre=False)`, `centre=` on its `Model`), so seats, mattresses and wall backs stay
# where the game puts residents. Seat and mattress heights are the renderer's (characters.ts `SEATS`,
# `BEDS`; catalog2.py `TOPS`, `SEATS`).
#
# Run:  python3 -B tools/art/blender/build.py kids [--only crib] [--preview]
import math
import os
import sys

import bpy

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402
from gamekit import shapes  # noqa: E402

M = G.Palette(
    # toys' colours, shared
    toy_red=('#D9483B', 'satin'),
    toy_blue=('#3E8EDE', 'satin'),
    toy_yellow=('#F2C14E', 'satin'),
    toy_green=('#5BAE6A', 'satin'),
    toy_pink=('#E07BC3', 'satin'),
    ball_red=('#D9483B', 'gloss'),
    ball_blue=('#3E8EDE', 'gloss'),
    ball_yellow=('#F2C14E', 'gloss'),
    ball_green=('#5BAE6A', 'gloss'),
    ball_pink=('#E07BC3', 'gloss'),
    dark=('#2B2C2E', 'satin'),        # tyres, eyes' backs, a spoiler
    eye=('#2A2624', 'gloss'),
    paper=('#F7F6F1', 'matte'),
    # the crib and its changing table
    crib=('#F2F0EA', 'satin'),          # the frame (Frame)
    crib_bedding=('#CFE0EA', 'matte'),  # the mattress cover and the changing pad (Bedding)
    crib_blanket=('#FBFBF8', 'matte'),  # the folded blanket at the foot
    crib_wood=('#CDB08A', 'satin'),     # the changing table's top and drawer fronts
    mobile=('#F2C14E', 'satin'),        # the mobile's arms and stars (Mobile)
    mobile_pink=('#E4B8AE', 'satin'),
    mobile_blue=('#9DC6CF', 'satin'),
    # the kids' bed
    kids_frame=('#9DC6CF', 'satin'),
    kids_bedding=('#F2C14E', 'fabric'),
    mattress=('#F7F7F5', 'fabric'),
    sheet=('#FFFFFF', 'fabric'),        # the sheet turned down over a duvet's top edge, pillows
    teddy_small=('#C08A52', 'fabric'),
    pennant_a=('#D6A64A', 'satin'),
    pennant_b=('#4E7A7F', 'satin'),
    pennant_c=('#B5653F', 'satin'),
    # the race car bed
    car=('#D9483B', 'gloss'),
    car_bedding=('#3E8EDE', 'fabric'),
    car_stripe=('#F7F6F1', 'gloss'),
    hub=('#D0D3D6', 'satin'),
    headlight=('#F2C14E', 'gloss'),
    # the high chair
    chair_white=('#F2F2F0', 'satin'),
    chair_legs=('#E3CFA8', 'satin'),
    chair_cushion=('#9DC6CF', 'fabric'),
    # the toy box and the toy chest
    toybox=('#F2C14E', 'satin'),
    toybox_lid=('#E9A23B', 'satin'),
    toybox_inside=('#F4E2B0', 'satin'),
    chest=('#E9C46A', 'satin'),
    chest_trim=('#D9483B', 'satin'),
    chest_inside=('#C9A66A', 'satin'),
    plush=('#C08A52', 'satin'),
    # the doll house
    house=('#F4EBD8', 'satin'),          # its walls (Walls)
    roof=('#D9483B', 'satin'),           # (Roof)
    house_trim=('#FFFFFF', 'satin'),     # window frames, the gable's edge
    house_window=('#BFE0E6', 'satin'),
    house_door=('#6E4A33', 'satin'),
    house_floor=('#A8754A', 'satin'),
    wallpaper=('#F2D9D5', 'satin'),
    wallpaper_up=('#DCEBD9', 'satin'),
    # the rocking horse
    horse=('#F2F0EA', 'satin'),          # (Body)
    saddle=('#D9483B', 'fabric'),        # the saddle and the bridle (Trim)
    mane=('#C08A52', 'fabric'),
    rocker=('#C9A27A', 'wood'),
    spot=('#C9C4BA', 'satin'),
    handle=('#2A2C30', 'metal'),
    # the play tent
    canvas=('#F2EBDD', 'fabric'),        # (Canvas)
    poles=('#C9A27A', 'satin'),          # (Poles)
    flap=('#E4B8AE', 'fabric'),
    tent_door=('#3B302A', 'matte'),
    tent_mat=('#9DC6CF', 'fabric'),
    # the ball pit
    pit=('#9DC6CF', 'satin'),
    pit_rim=('#F4F1EA', 'satin'),
    pit_fill=('#7FA9C9', 'satin'),
    # the train table
    train_table=('#F4F1EA', 'satin'),    # (Table)
    grass=('#6FA36A', 'satin'),
    track=('#C9A27A', 'wood'),
    track_groove=('#8E6E52', 'satin'),
    train=('#D9483B', 'gloss'),          # the engine (Train)
    water=('#9DC6CF', 'gloss'),
    tree=('#3F7A4A', 'satin'),
    trunk=('#8E6E52', 'satin'),
    # the play mat
    mat=('#9DC6CF', 'fabric'),
    mat_yellow=('#F2C14E', 'fabric'),
    mat_pink=('#E4B8AE', 'fabric'),
    mat_green=('#B5D99C', 'fabric'),
    mat_white=('#F2F2F0', 'fabric'),
    # the teddy bear
    fur=('#C08A52', 'fabric'),
    fur_light=('#E9D5B5', 'fabric'),
    bow=('#D9483B', 'satin'),
    # the bean bag
    bag=('#5BB3C9', 'fabric'),
    bag_patch=('#F2C14E', 'fabric'),
    # the kids' desk, table and chair
    kdesk=('#F2F2F0', 'satin'),          # legs, apron and drawer (Frame)
    kdesk_top=('#E3CFA8', 'wood'),       # (Top)
    kdesk_chair=('#9DC6CF', 'satin'),    # (Chair)
    ktable=('#3E8EDE', 'gloss'),         # legs and apron (Frame)
    ktable_top=('#F4F1EA', 'satin'),     # (Top)
    kchair=('#3E8EDE', 'gloss'),         # (Frame)
    kchair_seat=('#F4F1EA', 'satin'),    # (Top)
    # the picture-book shelf
    kshelf=('#F4F1EA', 'satin'),         # (Frame)
    kshelf_top=('#9DC6CF', 'satin'),     # the top board (Top)
    book_teal=('#4E7A7F', 'matte'),
    book_rust=('#B5653F', 'matte'),
    book_blue=('#3E8EDE', 'matte'),
    book_ochre=('#D6A64A', 'matte'),
    book_sage=('#9DAF8C', 'matte'),
    book_rose=('#E4B8AE', 'matte'),
    # the chalkboard easel and the kids' easel
    chalk_frame=('#B5784A', 'wood'),     # (Frame)
    slate=('#2E3532', 'matte'),          # (Board)
    chalk_white=('#F2F2EE', 'matte'),
    chalk_yellow=('#F2D16B', 'matte'),
    chalk_pink=('#E9A0A8', 'matte'),
    chalk_blue=('#8EC5E8', 'matte'),
    chalk_green=('#9DD18B', 'matte'),
    easel=('#E3CFA8', 'wood'),           # (Easel)
    paint_red=('#D9483B', 'matte'),
    paint_blue=('#3E8EDE', 'matte'),
    paint_green=('#5BAE6A', 'matte'),
    paint_yellow=('#F2C14E', 'matte'),
)

F = G.GameFrame()


# --- helpers ---------------------------------------------------------------------------------------------
def _add(*vs):
    return tuple(sum(c) for c in zip(*vs))


def _mul(v, k):
    return tuple(c * k for c in v)


def _mesh(name, verts, faces, material):
    """A flat-shaded mesh from game points and faces, its normals made to face out."""
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata([F.at(*v) for v in verts], [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    shapes._recalc_normals(obj)
    return obj


def _plane(origin, s, u):
    """A frame on a flat face: its point `origin`, the across `s` and up `u` directions (game, unit), and its
    outward normal."""
    n = (s[1] * u[2] - s[2] * u[1], s[2] * u[0] - s[0] * u[2], s[0] * u[1] - s[1] * u[0])
    return origin, s, u, n


def _board(centre, tilt):
    """The frame on the front face of a board leaning back by `tilt` degrees, centred on `centre`."""
    t = math.radians(tilt)
    return _plane(centre, (1, 0, 0), (0, math.cos(t), -math.sin(t)))


def _slab(name, pts, frame, material, out=0.0, thick=0.004):
    """A flat piece: the convex outline `pts` [(across, up)] on `frame`, `out` in front of it, `thick` thick."""
    o, s, u, n = frame
    at = lambda a, b, c: _add(o, _mul(s, a), _mul(u, b), _mul(n, c))  # noqa: E731
    k = len(pts)
    verts = [at(a, b, out) for a, b in pts] + [at(a, b, out + thick) for a, b in pts]
    faces = [tuple(range(k)), tuple(range(2 * k - 1, k - 1, -1))]
    faces += [(i, (i + 1) % k, k + (i + 1) % k, k + i) for i in range(k)]
    return _mesh(name, verts, faces, material)


def _tube(name, pts, r, material, sides=10):
    """A round tube along the game points `pts`, which lie in a plane across x (the rockers' bow)."""
    verts = []
    for i, p in enumerate(pts):
        a, b = pts[max(i - 1, 0)], pts[min(i + 1, len(pts) - 1)]
        t = _add(b, _mul(a, -1))
        t = _mul(t, 1 / math.sqrt(sum(c * c for c in t)))
        n = (0.0, -t[2], t[1])   # across x: (1, 0, 0) x t
        for k in range(sides):
            f = 2 * math.pi * k / sides
            verts.append(_add(p, _mul((1, 0, 0), r * math.cos(f)), _mul(n, r * math.sin(f))))
    faces = [(i * sides + k, i * sides + (k + 1) % sides, (i + 1) * sides + (k + 1) % sides, (i + 1) * sides + k)
             for i in range(len(pts) - 1) for k in range(sides)]
    obj = _mesh(name, verts, faces, material)
    for poly in obj.data.polygons:
        poly.use_smooth = True
    return obj


def _arc(r, a0, a1, n, cx=0.0, cy=0.0):
    """`n` + 1 points on a circle of radius `r` round (cx, cy), from `a0` to `a1` degrees."""
    return [(cx + r * math.cos(math.radians(a0 + (a1 - a0) * i / n)),
             cy + r * math.sin(math.radians(a0 + (a1 - a0) * i / n))) for i in range(n + 1)]


def _place(forward=0.0):
    """Stands the model on the floor where it was built (over its placeholder) and prints its centre;
    `forward` (game depth, m) moves it towards the front first: its back onto the wall face."""
    for o in G.meshes():
        o.location.y -= forward
    lo, hi = G.measure()
    print(f'[kids] centre (game x, depth) {-(lo[0] + hi[0]) / 2:.4f}, {-(lo[1] + hi[1]) / 2:.4f}')
    G.settle(centre=False)


def _still(name, parts=None):
    """Joins what's in the scene (or `parts`) as the still body, its origin on the floor."""
    return G.join(parts if parts is not None else G.meshes(), name, origin=(0, 0, 0))


# --- beds: a duvet whose top third turns down (as bedroom.py) ----------------------------------------------
DUVET = 0.05      # the duvet's top above the mattress top
FLAP = 0.045      # the flap's thickness


def _pillow(name, w, h, d, x, top, z, tilt=8):
    F.box(name, w, h, d, (x, top + h / 2 - 0.01, z), M.sheet, rot=(tilt, 0, 0), bevel=min(h, d) * 0.42,
          bevel_segments=3)


def _duvet(top, half, z_fold, z_foot, drape, material):
    """The duvet's lower two thirds, from the fold to the foot, draped down the sides and the foot end."""
    over = 0.02
    y0, y1 = top - drape, top + DUVET
    F.box('duvet', 2 * (half + over), y1 - y0, z_foot + over - z_fold, (0, (y0 + y1) / 2, (z_fold + z_foot + over) / 2),
          material, bevel=0.022, bevel_segments=3)


def _flap(top, half, z_top, z_fold, material):
    """The duvet's top third with the sheet turned down over its head edge, hinged across its fold."""
    with G.new_parts() as parts:
        y = top + DUVET - FLAP / 2
        F.box('flap', 2 * (half + 0.012), FLAP, z_fold - z_top, (0, y, (z_top + z_fold) / 2), material,
              bevel=0.02, bevel_segments=3)
        F.box('flap_sheet', 2 * (half + 0.016), FLAP + 0.008, 0.12, (0, y, z_top + 0.05), M.sheet,
              bevel=0.02, bevel_segments=3)
    return G.hinged(G.join(parts, 'duvet_flap'), F.at(0, top + DUVET + 0.003, z_fold))


def _turn_down(top, half, z_top, z_foot, drape, material):
    """Builds the duvet (its still two thirds and the flap), joins the bed, settles, keys `turn_down`."""
    z_fold = z_top + (z_foot - z_top) / 3
    _duvet(top, half, z_fold, z_foot, drape, material)
    _still('bed')
    flap = _flap(top, half, z_top, z_fold, material)
    _place()
    G.swing(flap, 'turn_down', 180, seconds=1.0, axis='X')
    return G.meshes()


# --- KD1: the crib and its changing table ------------------------------------------------------------------
CRIB_X = -0.5     # the crib's middle: the baby lies here (slot 0 of the 2 x 1)
CRIB_TOP = 0.56   # the mattress top (characters.ts BEDS.crib)


def _spindles(name, a, b, along, material, n, y=0.69, h=0.42):
    """`n` spindles between `a` and `b` along x (`along` 'x') or depth ('z'), the other coordinate fixed."""
    for i in range(n):
        t = a + (b - a) * (i + 0.5) / n
        x, z = (t, along[1]) if along[0] == 'x' else (along[1], t)
        F.box(f'{name}{i}', 0.022, h, 0.022, (x, y, z), material, bevel=0.005)


def _crib_rails(name, z):
    """A long side's top and bottom rails and spindles, between the posts."""
    F.box(f'{name}_top', 0.79, 0.04, 0.04, (CRIB_X, 0.9, z), M.crib, bevel=0.01)
    F.box(f'{name}_bottom', 0.79, 0.04, 0.04, (CRIB_X, 0.47, z), M.crib, bevel=0.01)
    _spindles(f'{name}_spindle', CRIB_X - 0.385, CRIB_X + 0.385, ('x', z), M.crib, 7)


def _changing_table():
    x, z = 0.5, -0.2
    for sx in (-1, 1):
        for sz in (-1, 1):
            F.box(f'foot{sx}{sz}', 0.05, 0.04, 0.05, (x + sx * 0.35, 0.02, z + sz * 0.21), M.crib, bevel=0.008)
    F.box('chest', 0.78, 0.74, 0.5, (x, 0.41, z), M.crib, bevel=0.012)
    for i, y in enumerate((0.16, 0.4, 0.64)):
        F.box(f'drawer{i}', 0.72, 0.21, 0.015, (x, y, z + 0.2575), M.crib_wood, bevel=0.006)
        G.sphere(f'knob{i}', 0.018, at=F.at(x, y, z + 0.272), material=M.crib, scale=(1, 0.7, 1), segments=12, rings=8)
    F.box('chest_top', 0.82, 0.025, 0.52, (x, 0.7925, z), M.crib_wood, bevel=0.006)
    F.box('pad', 0.7, 0.06, 0.42, (x, 0.835, z + 0.01), M.crib_bedding, bevel=0.025, bevel_segments=3)
    F.box('guard_back', 0.8, 0.08, 0.025, (x, 0.845, z - 0.245), M.crib, bevel=0.008)
    for s in (-1, 1):
        F.box(f'guard{s}', 0.025, 0.08, 0.5, (x + s * 0.39, 0.845, z), M.crib, bevel=0.008)


def _mobile():
    """The mobile: a hub on the arm's end, four arms and a toy on a string under each, turning about the hub."""
    hx, hy, hz = CRIB_X, 0.975, -0.24   # over the head end, clear of arms reaching in over the front rail
    with G.new_parts() as parts:
        F.cyl('hub', 0.03, 0.025, (hx, hy, hz), M.mobile, verts=16)
        toys = (M.mobile, M.mobile_pink, M.mobile_blue, M.mobile_pink)
        for k, m in enumerate(toys):
            a = math.radians(90 * k + 45)
            x, z = hx + 0.12 * math.cos(a), hz + 0.12 * math.sin(a)
            G.rod(f'arm{k}', F.at(hx, hy, hz), F.at(x, hy - 0.01, z), 0.005, material=M.mobile, verts=6)
            G.rod(f'string{k}', F.at(x, hy - 0.01, z), F.at(x, 0.875, z), 0.0015, material=M.crib, verts=4)
            if k % 2 == 0:  # a star
                _slab(f'star{k}', [(0.04 * math.cos(math.radians(90 + 72 * i)), 0.04 * math.sin(math.radians(90 + 72 * i)))
                                   for i in range(5)], _plane((x, 0.84, z), (1, 0, 0), (0, 1, 0)), m, out=-0.008,
                      thick=0.016)
            else:           # a ball
                G.sphere(f'ball{k}', 0.032, at=F.at(x, 0.845, z), material=m, segments=12, rings=8)
    return G.hinged(G.join(parts, 'mobile'), F.at(hx, hy, hz))


def crib():
    for x in (CRIB_X - 0.42, CRIB_X + 0.42):
        for z in (-0.44, 0.44):
            F.box(f'post{x}{z}', 0.05, 0.92, 0.05, (x, 0.46, z), M.crib, bevel=0.012)
            G.sphere(f'cap{x}{z}', 0.03, at=F.at(x, 0.93, z), material=M.crib, segments=12, rings=8)
    F.box('platform', 0.84, 0.04, 0.84, (CRIB_X, 0.44, 0), M.crib, bevel=0.008)
    F.box('mattress', 0.8, 0.1, 0.8, (CRIB_X, CRIB_TOP - 0.05, 0), M.crib_bedding, bevel=0.03, bevel_segments=3)
    F.box('blanket', 0.56, 0.035, 0.2, (CRIB_X, CRIB_TOP + 0.015, 0.25), M.crib_blanket, bevel=0.014, bevel_segments=3)
    F.box('blanket_stripe', 0.565, 0.036, 0.03, (CRIB_X, CRIB_TOP + 0.015, 0.2), M.mobile_blue, bevel=0.01)
    _crib_rails('back', -0.44)
    for x in (CRIB_X - 0.42, CRIB_X + 0.42):
        F.box(f'end_top{x}', 0.04, 0.04, 0.83, (x, 0.9, 0), M.crib, bevel=0.01)
        F.box(f'end_bottom{x}', 0.04, 0.04, 0.83, (x, 0.47, 0), M.crib, bevel=0.01)
        _spindles(f'end{x}', -0.39, 0.39, ('z', x), M.crib, 6)
    # the mobile's arm, clamped to the back rail
    G.rod('arm_up', F.at(CRIB_X, 0.9, -0.44), F.at(CRIB_X, 1.0, -0.44), 0.008, material=M.mobile, verts=8)
    G.rod('arm_over', F.at(CRIB_X, 1.0, -0.44), F.at(CRIB_X, 1.0, -0.24), 0.008, material=M.mobile, verts=8)
    G.rod('arm_drop', F.at(CRIB_X, 1.0, -0.24), F.at(CRIB_X, 0.985, -0.24), 0.006, material=M.mobile, verts=8)
    _changing_table()
    _still('body')

    with G.new_parts() as parts:
        _crib_rails('front', 0.44)
    side = G.join(parts, 'front_rail')
    mobile = _mobile()
    _place()
    G.slide(side, 'side_down', (0, 0, -0.25), seconds=0.9)
    G.spin(mobile, 'mobile', turns=1, seconds=6.0, axis='Z')
    return G.meshes()


# --- KD1: the kids' bed --------------------------------------------------------------------------------------
KIDS_TOP = 0.44   # the mattress top (characters.ts BEDS.kidsBed)


def kidsBed():
    for sx in (-1, 1):
        for sz in (-1, 1):
            F.box(f'foot{sx}{sz}', 0.06, 0.07, 0.06, (sx * 0.42, 0.035, sz * 0.88), M.kids_frame, bevel=0.01)
    F.box('base', 0.96, 0.24, 1.9, (0, 0.18, 0), M.kids_frame, bevel=0.03)
    F.box('mattress', 0.88, 0.16, 1.82, (0, KIDS_TOP - 0.08, 0.02), M.mattress, bevel=0.04, bevel_segments=3)
    # a headboard with an arched top, and bunting across it
    F.box('headboard', 0.96, 0.5, 0.06, (0, 0.32, -0.95), M.kids_frame, bevel=0.012)
    F.cyl('arch', 0.48, 0.06, (0, 0.5, -0.95), M.kids_frame, rot=(90, 0, 0), scale=(1, 0.55, 1), verts=32)
    G.rod('bunting', F.at(-0.4, 0.66, -0.917), F.at(0.4, 0.66, -0.917), 0.003, material=M.kids_bedding, verts=4)
    flags = (M.pennant_a, M.pennant_b, M.pennant_c, M.pennant_a, M.pennant_b)
    for k, m in enumerate(flags):
        x = -0.3 + 0.15 * k
        _slab(f'pennant{k}', [(-0.045, 0.0), (0.045, 0.0), (0.0, -0.08)],
              _plane((x, 0.665, -0.92), (1, 0, 0), (0, 1, 0)), m, thick=0.004)
    # a guard rail along the right side
    for z in (-0.6, -0.2, 0.2):
        F.box(f'rail_post{z}', 0.025, 0.38, 0.025, (0.475, 0.49, z), M.kids_frame, bevel=0.006)
    F.box('rail_top', 0.03, 0.03, 0.88, (0.475, 0.675, -0.2), M.kids_frame, bevel=0.01)
    _pillow('pillow', 0.58, 0.12, 0.28, 0, KIDS_TOP, -0.73)
    # a little teddy in the corner by the pillow
    G.sphere('teddy_body', 0.06, at=F.at(-0.36, KIDS_TOP + 0.06, -0.8), material=M.teddy_small, scale=(1, 0.9, 1.1),
             segments=12, rings=8)
    G.sphere('teddy_head', 0.045, at=F.at(-0.36, KIDS_TOP + 0.16, -0.81), material=M.teddy_small, segments=12, rings=8)
    for s in (-1, 1):
        G.sphere(f'teddy_ear{s}', 0.016, at=F.at(-0.36 + s * 0.035, KIDS_TOP + 0.2, -0.81), material=M.teddy_small,
                 segments=8, rings=6)
    return _turn_down(KIDS_TOP, 0.435, -0.5, 0.93, 0.12, M.kids_bedding)


# --- KD1: the race car bed -----------------------------------------------------------------------------------
CAR_TOP = 0.5     # the mattress top (the renderer's default bed height)


def kidsBedCar():
    F.box('chassis', 0.94, 0.3, 1.94, (0, 0.22, 0), M.car, bevel=0.1, bevel_segments=4)
    F.box('mattress', 0.84, 0.16, 1.44, (0, CAR_TOP - 0.08, -0.06), M.mattress, bevel=0.04, bevel_segments=3)
    # the bonnet at the foot: stripes, headlights, a grille
    F.box('bonnet', 0.94, 0.42, 0.32, (0, 0.38, 0.81), M.car, bevel=0.06, bevel_segments=4)
    for s in (-1, 1):
        F.box(f'stripe{s}', 0.08, 0.006, 0.18, (s * 0.09, 0.592, 0.81), M.car_stripe, bevel=0.002)
        G.sphere(f'headlight{s}', 0.06, at=F.at(s * 0.3, 0.45, 0.965), material=M.headlight, scale=(1, 0.35, 0.8),
                 segments=14, rings=8)
        # a number roundel on each side
        F.cyl(f'roundel{s}', 0.07, 0.006, (s * 0.471, 0.22, 0.0), M.car_stripe, rot=(0, 90, 0), verts=20)
    F.box('grille', 0.36, 0.08, 0.01, (0, 0.3, 0.972), M.dark, bevel=0.004)
    # the tail and its spoiler
    F.box('tail', 0.94, 0.26, 0.18, (0, 0.47, -0.88), M.car, bevel=0.07, bevel_segments=3)
    for s in (-1, 1):
        F.box(f'strut{s}', 0.04, 0.12, 0.04, (s * 0.3, 0.66, -0.9), M.dark, bevel=0.008)
        F.box(f'endplate{s}', 0.02, 0.09, 0.16, (s * 0.4, 0.73, -0.92), M.dark, bevel=0.006)
    F.box('wing', 0.8, 0.03, 0.14, (0, 0.735, -0.92), M.dark, bevel=0.01)
    # the wheels
    for sx in (-1, 1):
        for sz in (-1, 1):
            F.cyl(f'tyre{sx}{sz}', 0.18, 0.08, (sx * 0.45, 0.18, sz * 0.6), M.dark, rot=(0, 90, 0), verts=24)
            F.cyl(f'hub{sx}{sz}', 0.09, 0.085, (sx * 0.45, 0.18, sz * 0.6), M.hub, rot=(0, 90, 0), verts=16)
    _pillow('pillow', 0.5, 0.12, 0.26, 0, CAR_TOP, -0.64, tilt=12)
    return _turn_down(CAR_TOP, 0.42, -0.42, 0.66, 0.1, M.car_bedding)


# --- KD1: the high chair -------------------------------------------------------------------------------------
def highChair():
    for sx in (-1, 1):
        for zf, zt in ((-0.16, -0.07), (0.28, 0.17)):
            G.rod(f'leg{sx}{zf}', F.at(sx * 0.205, 0.0, zf), F.at(sx * 0.14, 0.57, zt), 0.0175, material=M.chair_legs,
                  verts=10)
    F.box('footrest', 0.36, 0.025, 0.1, (0, 0.32, 0.23), M.chair_white, bevel=0.008)
    F.box('seat', 0.38, 0.05, 0.36, (0, 0.58, 0.05), M.chair_white, bevel=0.015)
    F.box('cushion', 0.32, 0.04, 0.3, (0, 0.62, 0.06), M.chair_cushion, bevel=0.015, bevel_segments=3)
    F.box('back', 0.36, 0.32, 0.04, (0, 0.78, -0.12), M.chair_white, rot=(-6, 0, 0), bevel=0.03, bevel_segments=3)
    F.box('back_cushion', 0.28, 0.22, 0.02, (0, 0.77, -0.095), M.chair_cushion, rot=(-6, 0, 0), bevel=0.008)
    for s in (-1, 1):
        F.box(f'arm{s}', 0.04, 0.16, 0.32, (s * 0.17, 0.7, 0.05), M.chair_white, bevel=0.015)
    _still('body')
    # the tray, its own part so it can slide out once the chair has a use
    with G.new_parts() as parts:
        F.box('tray', 0.44, 0.025, 0.18, (0, 0.792, 0.2), M.chair_white, bevel=0.01)
        F.box('tray_lip', 0.4, 0.02, 0.012, (0, 0.81, 0.283), M.chair_white, bevel=0.005)
        F.box('tray_mat', 0.34, 0.004, 0.12, (0, 0.806, 0.2), M.chair_cushion, bevel=0.002)
    G.join(parts, 'tray')
    _place()
    return G.meshes()


# --- KD2: the toy box and the toy chest ----------------------------------------------------------------------
def _box_shell(w, h, d, z, y0, body, inside, name='box'):
    """An open box of walls on a floor, `y0` to `h`, with a hidden false bottom near the top for the toys."""
    yc = (y0 + h) / 2
    F.box(f'{name}_floor', w, 0.03, d, (0, y0 + 0.015, z), body, bevel=0.006)
    F.box(f'{name}_front', w, h - y0, 0.025, (0, yc, z + d / 2 - 0.0125), body, bevel=0.008)
    F.box(f'{name}_back', w, h - y0, 0.025, (0, yc, z - d / 2 + 0.0125), body, bevel=0.008)
    for s in (-1, 1):
        F.box(f'{name}_side{s}', 0.025, h - y0, d - 0.05, (s * (w / 2 - 0.0125), yc, z), body, bevel=0.008)
    F.box(f'{name}_fill', w - 0.05, 0.01, d - 0.05, (0, h - 0.15, z), inside)


def _toys_inside(top, z):
    """Toys heaped inside a box, seen with the lid open."""
    G.sphere('in_ball', 0.075, at=F.at(0.16, top - 0.07, z + 0.05), material=M.ball_blue, segments=14, rings=8)
    F.box('in_block', 0.09, 0.09, 0.09, (-0.2, top - 0.1, z + 0.02), M.toy_red, rot=(0, 0, 20), bevel=0.012)
    F.box('in_block2', 0.08, 0.08, 0.08, (-0.08, top - 0.1, z - 0.08), M.toy_green, rot=(10, 0, -15), bevel=0.012)
    G.sphere('in_plush', 0.07, at=F.at(0.0, top - 0.08, z + 0.06), material=M.plush, scale=(1, 0.9, 1),
             segments=12, rings=8)
    for s in (-1, 1):
        G.sphere(f'in_plush_ear{s}', 0.022, at=F.at(s * 0.045, top - 0.02, z + 0.06), material=M.plush,
                 segments=8, rings=6)


def toyBox():
    z = -0.18
    _box_shell(0.8, 0.46, 0.5, z, 0.0, M.toybox, M.toybox_inside)
    for k, (x, m) in enumerate(((-0.25, M.toy_red), (0.0, M.toy_green), (0.25, M.toy_blue))):
        F.box(f'tile{k}', 0.16, 0.14, 0.012, (x, 0.26, 0.076), m, bevel=0.008)
        F.box(f'tile_in{k}', 0.08, 0.07, 0.004, (x, 0.26, 0.084), M.toybox_inside, bevel=0.003)
    _toys_inside(0.46, z)
    # blocks left out on the floor in front
    F.box('floor_block0', 0.1, 0.1, 0.1, (0.32, 0.05, 0.25), M.toy_green, bevel=0.015)
    F.box('floor_block1', 0.1, 0.1, 0.1, (0.2, 0.05, 0.3), M.toy_red, rot=(0, 0, 30), bevel=0.015)
    F.box('floor_block2', 0.1, 0.1, 0.1, (0.27, 0.15, 0.27), M.toy_blue, rot=(0, 0, 12), bevel=0.015)
    _still('body')
    with G.new_parts() as parts:
        F.box('lid', 0.82, 0.05, 0.52, (0, 0.485, z), M.toybox_lid, bevel=0.015, bevel_segments=3)
        F.box('lid_grip', 0.16, 0.025, 0.03, (0, 0.475, z + 0.27), M.toybox_lid, bevel=0.01)
    lid = G.hinged(G.join(parts, 'lid'), F.at(0, 0.46, z - 0.26))
    _place()
    G.swing(lid, 'lid_open', -100, seconds=1.0, axis='X')
    return G.meshes()


def toyChest():
    z = -0.18
    for sx in (-1, 1):
        for sz in (-1, 1):
            G.sphere(f'foot{sx}{sz}', 0.03, at=F.at(sx * 0.36, 0.03, z + sz * 0.19), material=M.chest_trim,
                     segments=12, rings=8)
    _box_shell(0.8, 0.44, 0.46, z, 0.04, M.chest, M.chest_inside)
    F.box('band', 0.8, 0.06, 0.02, (0, 0.32, 0.055), M.chest_trim, bevel=0.006)
    for s in (-1, 1):
        F.box(f'strap{s}', 0.05, 0.4, 0.02, (s * 0.3, 0.24, 0.056), M.chest_trim, bevel=0.006)
        F.box(f'band_side{s}', 0.02, 0.06, 0.46, (s * 0.405, 0.32, z), M.chest_trim, bevel=0.006)
        G.torus(f'handle{s}', 0.04, 0.009, at=F.at(s * 0.415, 0.33, z), material=M.toy_blue, rot=(90, 0, 90),
                major_segments=16, minor_segments=6)
    _toys_inside(0.44, z)
    _still('body')
    with G.new_parts() as parts:
        F.box('lid', 0.84, 0.09, 0.5, (0, 0.485, z), M.chest, bevel=0.035, bevel_segments=4)
        F.box('lid_band', 0.84, 0.03, 0.02, (0, 0.47, z + 0.245), M.chest_trim, bevel=0.006)
        for s in (-1, 1):
            F.box(f'lid_strap{s}', 0.05, 0.096, 0.5, (s * 0.3, 0.486, z), M.chest_trim, bevel=0.03, bevel_segments=3)
        F.box('latch', 0.06, 0.07, 0.012, (0, 0.445, z + 0.257), M.toy_yellow, bevel=0.006)
    lid = G.hinged(G.join(parts, 'lid'), F.at(0, 0.44, z - 0.25))
    _place()
    G.swing(lid, 'lid_open', -100, seconds=1.0, axis='X')
    return G.meshes()


# --- KD2: the doll house -------------------------------------------------------------------------------------
ROOF_RISE = 0.18


def dollHouse():
    z = -0.16
    F.box('base', 0.8, 0.04, 0.42, (0, 0.02, -0.15), M.house_floor, bevel=0.008)
    F.box('back', 0.76, 0.66, 0.02, (0, 0.37, -0.34), M.house, bevel=0.004)
    F.box('paper_down', 0.72, 0.31, 0.004, (0, 0.205, -0.328), M.wallpaper)
    F.box('paper_up', 0.72, 0.30, 0.004, (0, 0.53, -0.328), M.wallpaper_up)
    for s in (-1, 1):
        F.box(f'side{s}', 0.02, 0.66, 0.38, (s * 0.37, 0.37, z), M.house, bevel=0.004)
    F.box('floor_down', 0.72, 0.006, 0.36, (0, 0.043, z), M.house_floor)
    F.box('floor_up', 0.72, 0.02, 0.37, (0, 0.37, z), M.house_floor, bevel=0.003)
    F.box('ceiling', 0.76, 0.02, 0.38, (0, 0.69, z), M.house, bevel=0.003)
    # the gables and the roof
    gable = [(-0.38, 0.0), (0.38, 0.0), (0.0, ROOF_RISE)]
    for k, zg in enumerate((0.03, -0.35)):
        _slab(f'gable{k}', gable, _plane((0, 0.7, zg), (1, 0, 0), (0, 1, 0)), M.house, thick=0.02)
    F.cyl('gable_window', 0.045, 0.012, (0, 0.77, 0.05), M.house_trim, rot=(90, 0, 0), verts=20)
    F.cyl('gable_glass', 0.033, 0.014, (0, 0.77, 0.05), M.house_window, rot=(90, 0, 0), verts=20)
    a = math.degrees(math.atan2(ROOF_RISE, 0.38))
    run = 0.23 * math.cos(math.radians(a))
    for s in (-1, 1):
        y = 0.7 + ROOF_RISE - run * math.tan(math.radians(a)) + 0.015 / math.cos(math.radians(a))
        F.box(f'roof{s}', 0.46, 0.03, 0.44, (s * run, y, z), M.roof, rot=(0, -s * a, 0), bevel=0.006)
    # the rooms' furniture: a sofa, a table and a rug downstairs, a bed, a lamp and a picture up
    F.box('rug', 0.26, 0.004, 0.16, (-0.05, 0.048, -0.12), M.wallpaper_up)
    F.box('sofa', 0.2, 0.06, 0.08, (-0.17, 0.08, -0.28), M.toy_blue, bevel=0.012)
    F.box('sofa_back', 0.2, 0.06, 0.03, (-0.17, 0.13, -0.31), M.toy_blue, bevel=0.01)
    F.box('table', 0.12, 0.012, 0.08, (0.18, 0.12, -0.2), M.house_door, bevel=0.003)
    F.cyl('table_leg', 0.008, 0.07, (0.18, 0.08, -0.2), M.house_door, verts=8)
    F.box('bed', 0.2, 0.05, 0.12, (-0.18, 0.405, -0.26), M.toy_pink, bevel=0.01)
    F.box('bed_pillow', 0.06, 0.03, 0.08, (-0.25, 0.44, -0.26), M.house_trim, bevel=0.01)
    F.box('cupboard', 0.1, 0.14, 0.06, (0.2, 0.45, -0.3), M.toy_yellow, bevel=0.006)
    F.box('picture', 0.1, 0.08, 0.006, (0.0, 0.56, -0.324), M.toy_green, bevel=0.002)
    _still('body')
    # the front wall, hinged on its left edge
    with G.new_parts() as parts:
        F.box('front', 0.76, 0.66, 0.02, (0, 0.37, 0.04), M.house, bevel=0.004)
        F.box('door', 0.12, 0.2, 0.008, (-0.2, 0.14, 0.054), M.house_door, bevel=0.003)
        F.box('door_frame', 0.14, 0.215, 0.004, (-0.2, 0.145, 0.051), M.house_trim, bevel=0.002)
        G.sphere('door_knob', 0.007, at=F.at(-0.165, 0.14, 0.06), material=M.toy_yellow, segments=8, rings=6)
        for k, (x, y) in enumerate(((0.18, 0.2), (-0.18, 0.54), (0.18, 0.54))):
            F.box(f'window{k}', 0.13, 0.13, 0.006, (x, y, 0.052), M.house_trim, bevel=0.003)
            F.box(f'glass{k}', 0.105, 0.105, 0.006, (x, y, 0.054), M.house_window)
            F.box(f'bar_v{k}', 0.008, 0.105, 0.004, (x, y, 0.0575), M.house_trim)
            F.box(f'bar_h{k}', 0.105, 0.008, 0.004, (x, y, 0.0575), M.house_trim)
        F.box('flower_box', 0.14, 0.03, 0.025, (0.18, 0.125, 0.06), M.house_door, bevel=0.004)
        for k in range(4):
            G.sphere(f'flower{k}', 0.014, at=F.at(0.135 + 0.03 * k, 0.148, 0.062), material=(M.toy_pink, M.toy_yellow)[k % 2],
                     segments=8, rings=6)
        F.box('step', 0.16, 0.02, 0.012, (-0.2, 0.035, 0.057), M.house_trim, bevel=0.003)
    front = G.hinged(G.join(parts, 'front_wall'), F.at(-0.38, 0.37, 0.04))
    _place(forward=-0.05)
    G.swing(front, 'front_open', 120, seconds=1.0)
    return G.meshes()


# --- KD2: the rocking horse ----------------------------------------------------------------------------------
ROCK_R = 1.0      # the rockers' radius: the horse rocks about their centre, 1 m up


def _rocker_y(z):
    return ROCK_R - math.sqrt(ROCK_R * ROCK_R - z * z) + 0.022


def rockingHorse():
    end = math.degrees(math.asin(0.37 / ROCK_R))
    pts = [math.sin(math.radians(-end + 2 * end * i / 16)) * ROCK_R for i in range(17)]
    for x in (-0.12, 0.12):
        _tube(f'rocker{x}', [(x, _rocker_y(z), z) for z in pts], 0.022, M.rocker)
        for z in (pts[0], pts[-1]):
            G.sphere(f'rocker_end{x}{z}', 0.026, at=F.at(x, _rocker_y(z) + 0.01, z), material=M.rocker, segments=10, rings=6)
    for z in (-0.3, 0.3):
        G.rod(f'stretcher{z}', F.at(-0.12, _rocker_y(z), z), F.at(0.12, _rocker_y(z), z), 0.016, material=M.rocker, verts=8)
    for sx in (-1, 1):
        for sz in (-1, 1):
            G.rod(f'leg{sx}{sz}', F.at(sx * 0.06, 0.42, sz * 0.16), F.at(sx * 0.12, _rocker_y(sz * 0.22) + 0.015, sz * 0.22),
                  0.022, material=M.horse, verts=10)
    G.sphere('body', 0.1, at=F.at(0, 0.45, 0.0), material=M.horse, scale=(1, 2.6, 1.05), segments=20, rings=12)
    for k, (x, y, z) in enumerate(((0.09, 0.47, -0.1), (-0.09, 0.44, 0.06), (0.085, 0.42, 0.12), (-0.08, 0.48, -0.14))):
        G.sphere(f'spot{k}', 0.03, at=F.at(x, y, z), material=M.spot, scale=(0.35, 1, 0.8), segments=10, rings=6)
    G.rod('neck', F.at(0, 0.48, 0.16), F.at(0, 0.7, 0.3), 0.055, material=M.horse, verts=14)
    G.sphere('head', 0.06, at=F.at(0, 0.73, 0.37), material=M.horse, scale=(0.9, 2.0, 1.0), rot=(25, 0, 0),
             segments=16, rings=10)
    G.torus('bridle', 0.05, 0.007, at=F.at(0, 0.7, 0.43), material=M.saddle, rot=(-65, 0, 0),
            major_segments=16, minor_segments=5)
    for s in (-1, 1):
        G.sphere(f'eye{s}', 0.012, at=F.at(s * 0.05, 0.765, 0.37), material=M.eye, segments=8, rings=6)
        F.cone(f'ear{s}', 0.018, 0.003, 0.05, (s * 0.03, 0.8, 0.32), M.horse, verts=8)
    G.rod('mane', F.at(0, 0.52, 0.12), F.at(0, 0.79, 0.29), 0.026, material=M.mane, verts=10)
    G.rod('forelock', F.at(0, 0.79, 0.29), F.at(0, 0.8, 0.34), 0.02, material=M.mane, verts=8)
    G.cone('tail', 0.028, 0.012, 0.22, at=F.at(0, 0.376, -0.305), material=M.mane, rot=(-150, 0, 0), verts=10)
    F.box('saddle', 0.2, 0.025, 0.2, (0, 0.548, -0.02), M.saddle, bevel=0.01, bevel_segments=3)
    for s in (-1, 1):
        F.box(f'saddle_flap{s}', 0.012, 0.09, 0.14, (s * 0.1, 0.5, -0.02), M.saddle, bevel=0.004)
    G.rod('handle', F.at(-0.11, 0.68, 0.28), F.at(0.11, 0.68, 0.28), 0.009, material=M.handle, verts=8)
    for s in (-1, 1):
        G.sphere(f'grip{s}', 0.018, at=F.at(s * 0.115, 0.68, 0.28), material=M.rocker, segments=10, rings=6)
    # the whole horse rocks, its origin on the rockers' centre; the rider rides it
    horse = G.join(G.meshes(), 'horse', origin=F.at(0, ROCK_R, 0))
    _place()
    G.rock(horse, 'rock', 12, seconds=1.2, axis='X')
    return G.meshes()


# --- KD2: the play tent --------------------------------------------------------------------------------------
TENT_R, TENT_H = 0.46, 1.3


def _tent_face(k):
    """The frame on face `k` of the six (0 the front), from the middle of its base edge up to the apex."""
    th = math.radians(90 + 60 * k)
    a = TENT_R * math.cos(math.radians(30))
    base = (a * math.cos(th), 0.0, a * math.sin(th))
    up = (-base[0], TENT_H, -base[2])
    length = math.sqrt(sum(c * c for c in up))
    return _plane(base, (math.sin(th), 0, -math.cos(th)), _mul(up, 1 / length)), length


DOOR = [(-0.16, 0.0), (0.16, 0.0), (0.13, 0.35), (0.05, 0.56), (-0.05, 0.56), (-0.13, 0.35)]


def playTent():
    corners = [(TENT_R * math.cos(math.radians(60 * i)), 0.0, TENT_R * math.sin(math.radians(60 * i))) for i in range(6)]
    apex = (0.0, TENT_H, 0.0)
    _mesh('canvas', corners + [apex], [(i, (i + 1) % 6, 6) for i in range(6)] + [tuple(range(5, -1, -1))], M.canvas)
    for i, c in enumerate(corners):
        d = _add(apex, _mul(c, -1))
        length = math.sqrt(sum(v * v for v in d))
        foot = _add(c, _mul(d, -0.012 / length))
        top = _add(apex, _mul(d, 0.28 / length))
        G.rod(f'pole{i}', F.at(*_add(foot, (0, 0.01, 0))), F.at(*top), 0.0125, material=M.poles, verts=8)
    F.cyl('floor_mat', 0.36, 0.03, (0, 0.015, 0), M.tent_mat, verts=24)
    front, _ = _tent_face(0)
    _slab('door', DOOR, front, M.tent_door, out=0.002, thick=0.003)
    F.cyl('roll', 0.022, 0.2, (0, 0.0, 0), M.flap, verts=10, rot=(0, 90, 0)).location = F.at(
        *_add(front[0], _mul(front[2], 0.575), _mul(front[3], 0.02)))
    # bunting round the front three faces
    flags = (M.toy_red, M.toy_yellow, M.toy_blue, M.toy_green, M.toy_red)
    for k in (-1, 0, 1):
        frame, _ = _tent_face(k)
        for j in (range(-1, 2) if k == 0 else (0,)):
            m = flags[(k + 1) * 2 + j % 2]
            _slab(f'flag{k}{j}', [(-0.035, 0.0), (0.035, 0.0), (0.0, -0.06)],
                  _plane(_add(frame[0], _mul(frame[2], 0.7), _mul(frame[1], 0.075 * j)), frame[1], frame[2]),
                  m, out=0.003, thick=0.003)
    _still('body')
    # the door flap: rolled up for play (squashed to its top edge along the canvas)
    flap = _slab('flap', [(x * 1.12, y * 1.02) for x, y in DOOR], front, M.flap, out=0.006, thick=0.006)
    top = _add(front[0], _mul(front[2], 0.575), _mul(front[3], 0.009))
    flap = G.hinged(G.join([flap], 'flap'), F.at(*top))
    _place()
    G.squash(flap, 'flap_open', (1, 0.1, 0.1), seconds=0.8)
    return G.meshes()


# --- KD2: the ball pit ---------------------------------------------------------------------------------------
def _balls():
    """The balls' five colours (the palette's materials are made afresh for each model)."""
    return (M.ball_red, M.ball_yellow, M.ball_blue, M.ball_green, M.ball_pink)


def _ball_grid(r_max, step, y, offset=0.0):
    pts = []
    rows = int(r_max / (step * 0.866)) + 1
    for j in range(-rows, rows + 1):
        for i in range(-rows - 1, rows + 2):
            x = (i + 0.5 * (j % 2) + offset) * step
            z = j * step * 0.866
            if x * x + z * z <= r_max * r_max:
                pts.append((x, y, z))
    return pts


def ballPit():
    balls = _balls()
    G.revolve('pit', [(0, 0), (0.45, 0), (0.46, 0.015), (0.46, 0.34), (0.42, 0.34), (0.42, 0.32), (0, 0.32)],
              at=F.at(0, 0, 0), material=M.pit, segments=32)
    G.torus('rim', 0.44, 0.035, at=F.at(0, 0.36, 0), material=M.pit_rim, major_segments=32, minor_segments=8)
    for k in range(8):
        a = math.radians(45 * k + 22.5)
        F.cyl(f'dot{k}', 0.045, 0.004, (0.462 * math.cos(a), 0.17, 0.462 * math.sin(a)), balls[k % 5],
              rot=(90, 0, 45 * k + 22.5 - 90), verts=14)
    F.cyl('fill', 0.42, 0.01, (0, 0.335, 0), M.pit_fill, verts=24)
    lower = _ball_grid(0.37, 0.085, 0.37)
    for k, (x, y, z) in enumerate(lower):
        # jiggled off the grid, so the heap doesn't look laid out
        jx, jy, jz = (math.sin(k * 12.9898) * 0.016, math.sin(k * 78.233) * 0.012, math.sin(k * 37.719) * 0.016)
        G.sphere(f'ball{k}', 0.04, at=F.at(x + jx, y + jy, z + jz), material=balls[(k * 7 + k // 3) % 5],
                 segments=6, rings=4)
    _still('body')
    with G.new_parts() as parts:
        for k, (x, z) in enumerate(((-0.17, -0.1), (0.04, 0.2), (0.2, -0.12), (-0.06, -0.24), (0.24, 0.12), (-0.22, 0.14))):
            G.sphere(f'top{k}', 0.04, at=F.at(x, 0.43, z), material=balls[k % 5], segments=6, rings=4)
    balls = G.join(parts, 'balls', origin=(0, 0, 0))
    _place()
    G.squash(balls, 'bounce', (1, 1, 1.18), seconds=0.7, back=True)
    return G.meshes()


# --- KD2: the train table ------------------------------------------------------------------------------------
RING_X, RING_Z, RING_R = 0.42, 0.02, 0.3
TABLE_TOP = 0.44


def _wagon(name, angle, parts_fn):
    """A piece of the train on the ring at `angle` (degrees round the ring, game), facing along it."""
    a = math.radians(angle)
    x, z = RING_X + RING_R * math.cos(a), RING_Z + RING_R * math.sin(a)
    for obj in parts_fn(name, x, z):
        obj.rotation_euler[2] = a    # along the ring (a game angle round the ring is the same Blender turn, + 180)


def _engine(name, x, z):
    y = TABLE_TOP + 0.012
    return [F.box(f'{name}_body', 0.05, 0.05, 0.11, (x, y + 0.035, z), M.train, bevel=0.01),
            F.box(f'{name}_cab', 0.055, 0.05, 0.045, (x, y + 0.08, z), M.train, bevel=0.008)]


def trainTable():
    for sx in (-1, 1):
        for sz in (-1, 1):
            F.box(f'leg{sx}{sz}', 0.06, 0.36, 0.06, (sx * 0.82, 0.18, sz * 0.34), M.train_table, bevel=0.01)
    F.box('frame', 1.8, 0.06, 0.84, (0, 0.38, 0), M.train_table, bevel=0.012)
    F.box('grass', 1.76, 0.03, 0.8, (0, 0.425, 0), M.grass, bevel=0.006)
    for s in (-1, 1):
        F.box(f'lip_x{s}', 1.8, 0.03, 0.02, (0, 0.425, s * 0.41), M.train_table, bevel=0.006)
        F.box(f'lip_z{s}', 0.02, 0.03, 0.8, (s * 0.89, 0.425, 0), M.train_table, bevel=0.006)
    # the ring of wooden track, with two grooves
    G.revolve('track', [(RING_R - 0.04, 0), (RING_R + 0.04, 0), (RING_R + 0.04, 0.012), (RING_R - 0.04, 0.012),
                        (RING_R - 0.04, 0)], at=F.at(RING_X, TABLE_TOP - 0.002, RING_Z), material=M.track, segments=40,
              smooth=False)
    for r in (RING_R - 0.016, RING_R + 0.016):
        G.torus(f'groove{r}', r, 0.003, at=F.at(RING_X, TABLE_TOP + 0.0105, RING_Z), material=M.track_groove,
                major_segments=40, minor_segments=4)
    # the town: a station, a pond, a road and trees
    F.box('station', 0.2, 0.1, 0.12, (-0.45, TABLE_TOP + 0.05, -0.22), M.toy_yellow, bevel=0.006)
    F.box('station_roof', 0.24, 0.02, 0.16, (-0.45, TABLE_TOP + 0.11, -0.22), M.toy_red, bevel=0.006)
    F.box('station_door', 0.05, 0.07, 0.004, (-0.45, TABLE_TOP + 0.035, -0.158), M.house_door)
    F.cyl('pond', 0.15, 0.006, (-0.18, TABLE_TOP + 0.003, 0.15), M.water, scale=(1.3, 1, 1), verts=24)
    F.box('road', 0.5, 0.004, 0.08, (-0.5, TABLE_TOP + 0.002, -0.08), M.track_groove)
    for k, (x, z, h) in enumerate(((-0.78, -0.28, 0.2), (-0.66, -0.31, 0.16), (-0.8, 0.26, 0.18), (0.82, -0.3, 0.16))):
        F.cyl(f'trunk{k}', 0.012, 0.04, (x, TABLE_TOP + 0.02, z), M.trunk, verts=8)
        F.cone(f'tree{k}', 0.05, 0.0, h, (x, TABLE_TOP + 0.04 + h / 2, z), M.tree, verts=10)
    _still('body')
    # the train: an engine and two wagons, turning round the ring's middle
    with G.new_parts() as parts:
        y = TABLE_TOP + 0.012

        def engine(name, x, z):
            objs = _engine(name, x, z)
            objs.append(F.cyl(f'{name}_chimney', 0.012, 0.035, (x, y + 0.075, z), M.dark, verts=10))
            return objs

        def wagon(colour, cargo):
            def make(name, x, z):
                objs = [F.box(f'{name}_tub', 0.05, 0.035, 0.09, (x, y + 0.03, z), colour, bevel=0.006)]
                objs.append(cargo(name, x, z))
                return objs
            return make

        ball = lambda n, x, z: G.sphere(f'{n}_ball', 0.022, at=F.at(x, y + 0.06, z), material=M.ball_yellow,  # noqa: E731
                                        segments=10, rings=6)
        log = lambda n, x, z: F.box(f'{n}_log', 0.03, 0.025, 0.08, (x, y + 0.055, z), M.trunk, bevel=0.008)  # noqa: E731
        _wagon('engine', 90, engine)
        _wagon('wagon0', 90 + 28, wagon(M.toy_blue, ball))
        _wagon('wagon1', 90 + 52, wagon(M.toy_green, log))
        for k, angle in enumerate((90, 90 + 28, 90 + 52)):
            a = math.radians(angle)
            for s in (-1, 1):
                F.cyl(f'wheels{k}{s}', 0.014, 0.008, (RING_X + (RING_R + s * 0.016) * math.cos(a), y + 0.012,
                                                      RING_Z + (RING_R + s * 0.016) * math.sin(a)), M.dark,
                      rot=(0, 90, math.degrees(a)), verts=10)
    train = G.join(parts, 'train', origin=F.at(RING_X, TABLE_TOP, RING_Z))
    _place()
    G.spin(train, 'train', turns=-1, seconds=6.0, axis='Z')
    return G.meshes()


# --- KD2: the play mat ---------------------------------------------------------------------------------------
def playMat():
    F.box('mat', 0.92, 0.02, 0.92, (0, 0.01, 0), M.mat, bevel=0.008, bevel_segments=3)
    for (x, z), m in zip(((-0.22, -0.22), (0.22, -0.22), (-0.22, 0.22), (0.22, 0.22)),
                         (M.mat_yellow, M.mat_pink, M.mat_green, M.mat_white)):
        F.box(f'tile{x}{z}', 0.4, 0.006, 0.4, (x, 0.021, z), m, bevel=0.004)
    y = 0.024
    blocks = (((-0.3, -0.15), M.toy_red, 10), ((-0.18, -0.24), M.toy_blue, 35), ((0.26, 0.14), M.toy_green, 50),
              ((-0.28, 0.24), M.toy_yellow, 70))
    for k, ((x, z), m, r) in enumerate(blocks):
        F.box(f'block{k}', 0.09, 0.09, 0.09, (x, y + 0.045, z), m, rot=(0, 0, r), bevel=0.012)
    F.box('block_top', 0.09, 0.09, 0.09, (-0.29, y + 0.135, -0.16), M.toy_green, rot=(0, 0, 25), bevel=0.012)
    G.sphere('ball', 0.06, at=F.at(0.3, y + 0.06, -0.25), material=M.ball_red, segments=14, rings=8)
    _still('body')
    # the tower the resident builds: three blocks, grown up from the mat
    with G.new_parts() as parts:
        for k, (m, r) in enumerate(((M.toy_blue, 0), (M.toy_yellow, 18), (M.toy_red, -12))):
            F.box(f'tower{k}', 0.09, 0.09, 0.09, (0.04, y + 0.045 + 0.09 * k, 0.12), m, rot=(0, 0, r), bevel=0.012)
    tower = G.join(parts, 'tower', origin=F.at(0.04, 0.012, 0.12))   # squeezed into the mat until it grows
    tower.scale[2] = 0.001
    _place()
    G.grow(tower, 'blocks', seconds=1.5)
    return G.meshes()


# --- KD2: the giant teddy bear and the kids' bean bag ---------------------------------------------------------
def teddyBear():
    G.sphere('body', 0.25, at=F.at(0, 0.28, -0.02), material=M.fur, scale=(1, 0.84, 1), segments=20, rings=12)
    G.sphere('belly', 0.15, at=F.at(0, 0.3, 0.165), material=M.fur_light, scale=(0.95, 0.45, 0.85), segments=16, rings=10)
    G.sphere('head', 0.2, at=F.at(0, 0.68, 0.0), material=M.fur, scale=(1, 0.9, 0.95), segments=20, rings=12)
    G.sphere('muzzle', 0.08, at=F.at(0, 0.64, 0.16), material=M.fur_light, scale=(1, 0.75, 0.75), segments=14, rings=8)
    G.sphere('nose', 0.028, at=F.at(0, 0.67, 0.22), material=M.eye, scale=(1.2, 0.8, 0.8), segments=10, rings=6)
    for s in (-1, 1):
        G.sphere(f'eye{s}', 0.02, at=F.at(s * 0.075, 0.73, 0.165), material=M.eye, segments=10, rings=6)
        G.sphere(f'ear{s}', 0.07, at=F.at(s * 0.15, 0.85, -0.02), material=M.fur, scale=(1, 0.55, 1), segments=12, rings=8)
        G.sphere(f'ear_in{s}', 0.045, at=F.at(s * 0.15, 0.85, 0.0), material=M.fur_light, scale=(1, 0.3, 1), segments=10, rings=6)
        G.sphere(f'arm{s}', 0.07, at=F.at(s * 0.25, 0.36, 0.06), material=M.fur, scale=(1, 1, 1.85), rot=(-20, 0, s * 15),
                 segments=12, rings=8)
        G.sphere(f'leg{s}', 0.09, at=F.at(s * 0.14, 0.08, 0.18), material=M.fur, scale=(1, 1.7, 0.8), segments=14, rings=8)
        G.sphere(f'pad{s}', 0.06, at=F.at(s * 0.14, 0.09, 0.33), material=M.fur_light, scale=(1, 0.25, 1.1), segments=12, rings=6)
        G.cone(f'bow{s}', 0.045, 0.012, 0.09, at=F.at(s * 0.055, 0.5, 0.17), material=M.bow, rot=(0, s * 90, 0), verts=10)
    G.sphere('knot', 0.022, at=F.at(0, 0.5, 0.175), material=M.bow, segments=10, rings=6)
    bear = G.join(G.meshes(), 'bear', origin=(0, 0, 0))
    _place()
    G.squash(bear, 'squish', (1.06, 1.06, 0.9), seconds=0.5)
    return G.meshes()


def beanBagKids():
    G.sphere('bag', 0.31, at=F.at(0, 0.18, 0.02), material=M.bag, scale=(1, 1, 0.58), segments=24, rings=14)
    G.sphere('back', 0.26, at=F.at(0, 0.32, -0.15), material=M.bag, scale=(1, 0.58, 0.81), rot=(-12, 0, 0),
             segments=24, rings=14)
    _slab('tag', _arc(0.045, 0, 330, 11), _board((0, 0.24, 0.315), 30), M.bag_patch, out=-0.002, thick=0.006)
    bag = G.join(G.meshes(), 'bag', origin=(0, 0, 0))
    _place()
    G.squash(bag, 'squash', (1.04, 1.04, 0.85), seconds=0.5)
    return G.meshes()


# --- KD3: the kids' desk (and its chair), table and chair -----------------------------------------------------
def kidsDesk():
    for x in (-0.34, 0.34):
        for z in (0.02, 0.38):
            F.cyl(f'leg{x}{z}', 0.02, 0.545, (x, 0.2725, z), M.kdesk, verts=12)
    F.box('top', 0.76, 0.03, 0.44, (0, 0.56, 0.2), M.kdesk_top, bevel=0.01)
    F.box('apron', 0.64, 0.06, 0.38, (0, 0.515, 0.2), M.kdesk, bevel=0.006)
    F.box('paper', 0.3, 0.004, 0.22, (-0.12, 0.577, 0.2), M.paper)
    F.box('drawing', 0.2, 0.002, 0.14, (-0.12, 0.5795, 0.2), M.chalk_blue)
    F.cyl('cup', 0.035, 0.08, (0.24, 0.615, 0.3), M.toy_yellow, verts=14)
    for k, (dx, m) in enumerate(((-0.012, M.toy_red), (0.01, M.toy_blue), (0.0, M.toy_green))):
        G.rod(f'crayon{k}', F.at(0.24 + dx, 0.6, 0.3 + dx), F.at(0.24 + dx * 2.5, 0.69, 0.3 + 0.012 * (k - 1)), 0.005,
              material=m, verts=6)
    # the chair, joined to the desk: its seat where the game sits a child (SEATS.kidsDesk)
    F.box('seat', 0.42, 0.04, 0.4, (0, 0.32, -0.2), M.kdesk_chair, bevel=0.012)
    F.box('seat_top', 0.38, 0.03, 0.36, (0, 0.35, -0.19), M.kdesk_chair, bevel=0.012)
    for s in (-1, 1):
        F.box(f'front_leg{s}', 0.035, 0.31, 0.035, (s * 0.18, 0.155, -0.03), M.kdesk_chair, bevel=0.008)
        F.box(f'back_post{s}', 0.035, 0.68, 0.035, (s * 0.18, 0.34, -0.38), M.kdesk_chair, bevel=0.008)
    F.box('backrest', 0.4, 0.1, 0.035, (0, 0.61, -0.38), M.kdesk_chair, bevel=0.015)
    F.box('back_rail', 0.4, 0.04, 0.03, (0, 0.44, -0.38), M.kdesk_chair, bevel=0.01)
    _still('body')
    # the pencil drawer under the top, pulled out towards the child
    with G.new_parts() as parts:
        F.box('drawer_front', 0.4, 0.05, 0.015, (0, 0.512, 0.0025), M.kdesk, bevel=0.005)
        F.box('drawer_tray', 0.38, 0.04, 0.3, (0, 0.512, 0.15), M.kdesk)
        G.sphere('drawer_knob', 0.012, at=F.at(0, 0.512, -0.01), material=M.toy_red, segments=10, rings=6)
        for k, m in enumerate((M.toy_red, M.toy_blue, M.toy_green, M.toy_yellow)):
            G.rod(f'pencil{k}', F.at(-0.12 + 0.07 * k, 0.536, 0.03), F.at(-0.1 + 0.07 * k, 0.536, 0.2), 0.005,
                  material=m, verts=6)
    drawer = G.join(parts, 'drawer')
    _place()
    G.slide(drawer, 'drawer_open', (0, 0.15, 0), seconds=0.6)
    return G.meshes()


def kidsTable():
    F.box('top', 0.64, 0.035, 0.64, (0, 0.4825, 0), M.ktable_top, bevel=0.03, bevel_segments=3)
    F.box('apron', 0.54, 0.05, 0.54, (0, 0.44, 0), M.ktable, bevel=0.008)
    for sx in (-1, 1):
        for sz in (-1, 1):
            F.cyl(f'leg{sx}{sz}', 0.025, 0.465, (sx * 0.26, 0.2325, sz * 0.26), M.ktable, verts=14)
            G.sphere(f'foot{sx}{sz}', 0.028, at=F.at(sx * 0.26, 0.018, sz * 0.26), material=M.ktable, scale=(1, 1, 0.65),
                     segments=12, rings=6)
    G.join_all('kidsTable')
    _place()
    return G.meshes()


def kidsChair():
    F.box('seat', 0.32, 0.03, 0.3, (0, 0.29, 0.05), M.kchair_seat, bevel=0.012)
    F.box('back', 0.32, 0.18, 0.03, (0, 0.48, -0.1), M.kchair, rot=(-6, 0, 0), bevel=0.03, bevel_segments=3)
    F.cyl('back_hole', 0.035, 0.032, (0, 0.5, -0.1), M.kchair_seat, rot=(84, 0, 0), verts=16)
    for sx in (-1, 1):
        for z in (-0.07, 0.17):
            F.cyl(f'leg{sx}{z}', 0.0175, 0.28, (sx * 0.13, 0.14, z), M.kchair, verts=10)
        F.cyl(f'post{sx}', 0.015, 0.2, (sx * 0.13, 0.4, -0.09), M.kchair, verts=8)
        G.rod(f'rung{sx}', F.at(sx * 0.13, 0.1, -0.07), F.at(sx * 0.13, 0.1, 0.17), 0.01, material=M.kchair, verts=8)
    G.join_all('kidsChair')
    _place()
    return G.meshes()


# --- KD3: the picture-book shelf -----------------------------------------------------------------------------
SHELVES = ((0.12, -0.1), (0.40, -0.17), (0.68, -0.24))    # (height, front lip's depth)
BOOK_TILT = 14


def _book(name, x, y, z, cover, picture):
    """A picture book stood face out, leaning back; its middle at (x, y, z)."""
    frame = _board((x, y, z), BOOK_TILT)
    pts = [(-0.09, -0.12), (0.09, -0.12), (0.09, 0.12), (-0.09, 0.12)]
    yield _slab(name, pts, frame, cover, out=-0.01, thick=0.02)
    yield _slab(f'{name}_pic', [(-0.05, -0.02), (0.05, -0.02), (0.05, 0.08), (-0.05, 0.08)], frame, picture,
                out=0.01, thick=0.002)


def kidsBookshelf():
    side = [(0.46, 0.0), (0.06, 0.0), (0.06, 0.2), (0.235, 0.9), (0.46, 0.9)]   # (back from the front, height)
    for s in (-1, 1):
        _slab(f'side{s}', [(-d, y) for d, y in side], _plane((s * 0.36, 0, 0), (0, 0, 1), (0, 1, 0)), M.kshelf,
              out=-0.0125, thick=0.025)
    F.box('back', 0.72, 0.9, 0.02, (0, 0.45, -0.45), M.kshelf, bevel=0.004)
    F.box('top', 0.77, 0.025, 0.26, (0, 0.9125, -0.33), M.kshelf_top, bevel=0.008)
    F.box('plinth', 0.7, 0.06, 0.02, (0, 0.03, -0.07), M.kshelf_top, bevel=0.004)
    lips = (M.toy_red, M.toy_blue, M.toy_yellow)
    covers = ((M.book_teal, M.toy_yellow), (M.book_rust, M.toy_blue), (M.book_blue, M.toy_red),
              (M.book_ochre, M.toy_green), (M.book_sage, M.toy_pink), (M.book_rose, M.toy_blue),
              (M.book_rust, M.toy_green), (M.book_blue, M.toy_yellow), (M.book_teal, M.toy_red))
    out = None
    for i, (y, z) in enumerate(SHELVES):
        F.box(f'shelf{i}', 0.7, 0.02, z + 0.44, (0, y, (z - 0.44) / 2), M.kshelf, bevel=0.003)
        F.box(f'lip{i}', 0.7, 0.08, 0.015, (0, y + 0.04, z), lips[i], bevel=0.004)
        for j, x in enumerate((-0.22, 0.0, 0.22)):
            cover, picture = covers[3 * i + j]
            bz = z - 0.04
            by = y + 0.01 + 0.12 * math.cos(math.radians(BOOK_TILT))
            bz -= 0.12 * math.sin(math.radians(BOOK_TILT))
            if (i, j) == (1, 1):
                with G.new_parts() as parts:
                    list(_book(f'book{i}{j}', x, by, bz, cover, picture))
                out = (parts, (x, y + 0.01, z - 0.03))
            else:
                list(_book(f'book{i}{j}', x, by, bz, cover, picture))
    still = [o for o in G.meshes() if o not in out[0]]
    _still('body', still)
    book = G.hinged(G.join(out[0], 'book'), F.at(*out[1]))
    _place(forward=0.03)
    G.swing(book, 'book_out', 20, seconds=0.6, axis='X')
    return G.meshes()


# --- KD3: the chalkboard easel and the kids' easel ------------------------------------------------------------
EASEL_TILT = 14


def _drawing(name, pieces, frame, left):
    """A picture of flat `pieces` [(outline, material)] on `frame`, joined with its origin on its left edge
    and squeezed to it, so `grow` draws it from left to right."""
    with G.new_parts() as parts:
        for k, (pts, m) in enumerate(pieces):
            _slab(f'{name}{k}', pts, frame, m, out=0.0008 * k, thick=0.002)   # each over the last
    o, s, u, n = frame
    obj = G.join(parts, name, origin=F.at(*_add(o, _mul(s, left))))
    obj.scale[0] = 0.001
    return obj


def _rect(x0, y0, x1, y1):
    return [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]


def kidsChalkboard():
    t = math.radians(EASEL_TILT)
    for k, (z, tilt) in enumerate(((0.11, -EASEL_TILT), (-0.11, EASEL_TILT))):
        F.box(f'frame{k}', 0.62, 0.96, 0.03, (0, 0.5, z), M.chalk_frame, rot=(tilt, 0, 0), bevel=0.01)
        F.box(f'slate{k}', 0.52, 0.8, 0.036, (0, 0.5, z), M.slate, rot=(tilt, 0, 0), bevel=0.004)
    F.cyl('top_bar', 0.015, 0.66, (0, 0.975, 0.0), M.chalk_frame, rot=(0, 90, 0), verts=10)
    # the chalk tray across the front, three sticks of chalk
    F.box('tray', 0.54, 0.02, 0.07, (0, 0.102, 0.25), M.chalk_frame, bevel=0.006)
    F.box('tray_lip', 0.54, 0.03, 0.01, (0, 0.12, 0.285), M.chalk_frame, bevel=0.004)
    for k, (x, m) in enumerate(((-0.12, M.chalk_white), (0.0, M.chalk_yellow), (0.1, M.chalk_pink))):
        F.cyl(f'chalk{k}', 0.007, 0.08, (x, 0.12, 0.258), m, rot=(0, 90, 0), verts=6)
    # the board's face: a sun drawn already
    face = _board((0, 0.5 + 0.0185 * math.sin(t), 0.11 + 0.0185 * math.cos(t)), EASEL_TILT)
    _slab('sun', [(-0.13 + 0.055 * math.cos(math.radians(a)), 0.25 + 0.055 * math.sin(math.radians(a)))
                  for a in range(0, 360, 30)], face, M.chalk_yellow, thick=0.002)
    for k in range(6):
        a = math.radians(60 * k + 15)
        c, s = math.cos(a), math.sin(a)
        _slab(f'ray{k}', [(-0.13 + 0.07 * c - 0.004 * s, 0.25 + 0.07 * s + 0.004 * c),
                          (-0.13 + 0.07 * c + 0.004 * s, 0.25 + 0.07 * s - 0.004 * c),
                          (-0.13 + 0.1 * c + 0.004 * s, 0.25 + 0.1 * s - 0.004 * c),
                          (-0.13 + 0.1 * c - 0.004 * s, 0.25 + 0.1 * s + 0.004 * c)], face, M.chalk_yellow, thick=0.002)
    _still('body')
    # the drawing the resident makes: a house in the grass
    house = [(_rect(0.0, -0.13, 0.16, -0.01), M.chalk_pink),
             ([(-0.02, -0.01), (0.18, -0.01), (0.08, 0.09)], M.chalk_white),
             (_rect(0.06, -0.13, 0.1, -0.06), M.chalk_blue),
             (_rect(0.02, -0.04, 0.05, -0.02), M.chalk_blue),
             (_rect(-0.2, -0.15, 0.22, -0.135), M.chalk_green)]
    drawing = _drawing('drawing', house, face, -0.2)
    _place()
    G.grow(drawing, 'drawing', seconds=2.5, axis='X')
    return G.meshes()


def _leg(name, x, z0, z1):
    G.rod(name, F.at(x, 0.0, z0), F.at(x, 1.2, z1), 0.0175, material=M.easel, verts=8)


def kidsEasel():
    t = math.radians(EASEL_TILT)
    for s in (-1, 1):
        _leg(f'leg_front{s}', s * 0.3, 0.3, 0.0)
        _leg(f'leg_back{s}', s * 0.3, -0.3, 0.0)
        F.box(f'paper{s}', 0.56, 0.62, 0.012, (0, 0.82, s * 0.11), M.paper, rot=(-s * EASEL_TILT, 0, 0))
        F.box(f'tray{s}', 0.6, 0.03, 0.1, (0, 0.48, s * 0.21), M.easel, bevel=0.006)
        F.box(f'tray_lip{s}', 0.6, 0.035, 0.012, (0, 0.5, s * 0.255), M.easel, bevel=0.004)
        F.box(f'clip{s}', 0.12, 0.04, 0.02, (0, 1.12, s * 0.035), M.easel, rot=(-s * EASEL_TILT, 0, 0), bevel=0.006)
    G.rod('top_bar', F.at(-0.33, 1.17, 0.0), F.at(0.33, 1.17, 0.0), 0.015, material=M.easel, verts=8)
    for k, m in enumerate((M.ball_red, M.ball_yellow, M.ball_blue, M.ball_green)):
        F.cyl(f'pot{k}', 0.025, 0.05, (-0.18 + 0.12 * k, 0.52, 0.22), m, verts=12)
        F.cyl(f'pot_lid{k}', 0.02, 0.004, (-0.18 + 0.12 * k, 0.547, 0.22), M.paper, verts=12)
    G.rod('brush', F.at(0.1, 0.505, 0.19), F.at(0.25, 0.51, 0.24), 0.005, material=M.easel, verts=6)
    _still('body')
    # the painting: a sky, a sun, a flower and some grass, painted on from the left
    face = _board((0, 0.82 + 0.0065 * math.sin(t), 0.11 + 0.0065 * math.cos(t)), EASEL_TILT)
    circle = lambda cx, cy, r, n=12: [(cx + r * math.cos(2 * math.pi * i / n), cy + r * math.sin(2 * math.pi * i / n))  # noqa: E731
                                      for i in range(n)]
    pieces = [(_rect(-0.24, 0.12, 0.24, 0.25), M.paint_blue),
              (circle(0.15, 0.18, 0.05), M.paint_yellow),
              (_rect(-0.24, -0.27, 0.24, -0.18), M.paint_green),
              (_rect(-0.07, -0.2, -0.05, 0.02), M.paint_green),
              (circle(-0.06, 0.06, 0.035), M.paint_yellow)]
    for k in range(5):
        a = 2 * math.pi * k / 5
        pieces.append((circle(-0.06 + 0.055 * math.cos(a), 0.06 + 0.055 * math.sin(a), 0.03), M.paint_red))
    painting = _drawing('painting', pieces, face, -0.25)
    _place()
    G.grow(painting, 'painting', seconds=2.5, axis='X')
    return G.meshes()


# --- the family --------------------------------------------------------------------------------------------
TURN_DOWN = {'uses': {'sleep': 'turn_down', 'nap': 'turn_down'}}
LID = {'uses': {'play': 'lid_open'}, 'hold': 2}

FAMILY = G.Family(
    'kids', M,
    [
        # KD1
        G.Model('crib', crib, size=(1.86, 1.008, 0.94), key='model.crib', centre=(-0.02, 0.0),
                motions={'uses': {'feed': 'side_down', 'change': 'side_down', 'cuddle': 'side_down'},
                         'loops': {'lie': 'mobile'}},
                paint=[('crib', 'Frame'), ('crib_bedding', 'Bedding'), ('mobile', 'Mobile')]),
        G.Model('kidsBed', kidsBed, size=(0.97, 0.764, 1.93), key='model.kidsBed', centre=(0.005, -0.015),
                motions=TURN_DOWN, paint=[('kids_frame', 'Frame'), ('kids_bedding', 'Bedding')]),
        G.Model('kidsBedCar', kidsBedCar, size=(0.985, 0.775, 1.986), key='model.kidsBedCar', centre=(0.0, -0.007),
                motions=TURN_DOWN, paint=[('car', 'Car body'), ('car_bedding', 'Bedding')]),
        G.Model('highChair', highChair, size=(0.444, 0.943, 0.474), key='model.highChair', centre=(0.0, 0.0599),
                paint=[('chair_white', 'Frame'), ('chair_cushion', 'Bedding')]),
        # KD2
        G.Model('toyBox', toyBox, size=(0.82, 0.51, 0.802), key='model.toyBox', centre=(0.0, -0.0389),
                motions=LID, paint=[('toybox', 'Body'), ('toybox_lid', 'Trim')]),
        G.Model('toyChest', toyChest, size=(0.846, 0.534, 0.513), key='model.toyChest', centre=(0.0, -0.1735),
                motions=LID, paint=[('chest', 'Body'), ('chest_trim', 'Trim')]),
        G.Model('dollHouse', dollHouse, size=(0.84, 0.908, 0.456), key='model.dollHouse', centre=(0.0, -0.202),
                motions={'uses': {'dolls': 'front_open'}}, paint=[('house', 'Walls'), ('roof', 'Roof')]),
        G.Model('rockingHorse', rockingHorse, size=(0.289, 0.825, 0.877), key='model.rockingHorse', centre=(0.0, 0.0426),
                motions={'loops': {'ride': 'rock'}, 'ride': 'rock'}, paint=[('horse', 'Body'), ('saddle', 'Trim')]),
        G.Model('playTent', playTent, size=(0.952, 1.574, 0.827), key='model.playTent', centre=(0.0, 0.0),
                motions={'uses': {'play': 'flap_open'}}, paint=[('canvas', 'Canvas'), ('poles', 'Poles')]),
        G.Model('ballPit', ballPit, size=(0.95, 0.47, 0.95), key='model.ballPit', centre=(0.0, 0.0),
                motions={'loops': {'jump': 'bounce'}}, paint=[('pit', 'Pit'), ('pit_rim', 'Rim')]),
        G.Model('trainTable', trainTable, size=(1.8, 0.68, 0.84), key='model.trainTable', centre=(0.0, 0.0),
                motions={'loops': {'trains': 'train'}}, paint=[('train_table', 'Table'), ('train', 'Train')]),
        G.Model('playMat', playMat, size=(0.92, 0.204, 0.92), key='model.playMat', centre=(0.0, 0.0),
                motions={'uses': {'blocks': 'blocks'}}, paint=[('mat', 'Mat')]),
        G.Model('teddyBear', teddyBear, size=(0.635, 0.912, 0.575), key='model.teddyBear', centre=(0.0, 0.0575),
                motions={'uses': {'hug': 'squish'}}, paint=[('fur', 'Fur'), ('bow', 'Bow')]),
        G.Model('beanBagKids', beanBagKids, size=(0.62, 0.528, 0.645), key='model.beanBagKids', centre=(0.0, 0.0187),
                motions={'uses': {'flop': 'squash'}}, paint=[('bag', 'Bag')]),
        # KD3
        G.Model('kidsDesk', kidsDesk, size=(0.76, 0.691, 0.82), key='model.kidsDesk', centre=(0.0, 0.01),
                motions={'uses': {'homework': 'drawer_open', 'drawing': 'drawer_open'}, 'hold': 2},
                paint=[('kdesk', 'Frame'), ('kdesk_top', 'Top'), ('kdesk_chair', 'Chair')]),
        G.Model('kidsTable', kidsTable, size=(0.64, 0.5, 0.64), key='model.kidsTable', centre=(0.0, 0.0),
                paint=[('ktable', 'Frame'), ('ktable_top', 'Top')]),
        G.Model('kidsChair', kidsChair, size=(0.32, 0.57, 0.323), key='model.kidsChair', centre=(0.0, 0.0386),
                paint=[('kchair', 'Frame'), ('kchair_seat', 'Top')]),
        G.Model('kidsBookshelf', kidsBookshelf, size=(0.77, 0.925, 0.4), key='model.kidsBookshelf', centre=(0.0, -0.23),
                motions={'uses': {'read': 'book_out'}}, paint=[('kshelf', 'Frame'), ('kshelf_top', 'Top')]),
        G.Model('kidsChalkboard', kidsChalkboard, size=(0.66, 0.956, 0.528), key='model.kidsChalkboard',
                centre=(0.0, 0.0259), motions={'uses': {'draw': 'drawing'}},
                paint=[('chalk_frame', 'Frame'), ('slate', 'Board')]),
        G.Model('kidsEasel', kidsEasel, size=(0.66, 1.208, 0.634), key='model.kidsEasel', centre=(0.0, 0.0),
                motions={'uses': {'paint': 'painting'}}, paint=[('easel', 'Easel')]),
    ],
    comment='Blender-built kids\' things (tools/art/blender/families/kids.py): the crib, the beds and the high '
            'chair, the toys, and the kids\' study and art things. Replaces the primitive-part placeholders by '
            'key; listed in the base manifest\'s packs. The placeholder is kept as a fallback if a file fails to '
            'load, and the materials map onto the game\'s finishes so a model costs one draw call per finish group.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

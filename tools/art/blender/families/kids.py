# The kids' things (docs/objects/kids.md). Notes: kids.md.
#
# KD1 baby and kids' beds: the crib with its changing table (`model.crib`), the kids' bed (`model.kidsBed`),
#   the race car bed (`model.kidsBedCar`) and the high chair (`model.highChair`).
# KD2 toys: the toy box and toy chest, the doll house, the rocking horse, the play tent, the ball pit, the
#   train table, the play mat, the giant teddy bear and the kids' bean bag.
# KD3 kids' study and art: the kids' desk, table and chair, the picture-book shelf, the chalkboard easel and
#   the kids' easel.
#
# Each is built where the game places it (`F`, game axes: x, height, depth; front +depth), then stood on
# the floor (`G.settle(centre=False)`, `centre=` on its `Model`), so seats, mattresses and wall backs stay
# where the game puts residents. Seat and mattress heights are the renderer's (characters.ts `SEATS`,
# `BEDS`; the content's `surface` and `seat`).
#
# Built with the forms (2026-10-22): mattresses, pillows, duvets and cushions are soft boxes, frames and rails
# one swept piece, shaped boards (the car's sides, arched headboards and ends, the doll house's windows, the
# toy box's lid) extruded outlines with their cut-outs, turned parts revolved, round bodies (the horse, the
# teddy, the bean bag, the tent's canvas) lofted.
#
# Run:  cargo art kids [--only crib] [--preview]
import math
import os
import sys

import bpy
from mathutils import Vector
from mathutils import noise as vnoise

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import gamekit as G  # noqa: E402
from gamekit import shapes  # noqa: E402
from gamekit.motion import _clip, last_frame  # noqa: E402

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
    kids_drawer=('#F4F1EA', 'satin'),   # the drawers under the bed
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
    tray_mat=('#9DC6CF', 'satin'),     # the high chair's wipe-clean mat: plain, so its sliding tray is one draw
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


def _arc(r, a0, a1, n, cx=0.0, cy=0.0):
    """`n` + 1 points on a circle of radius `r` round (cx, cy), from `a0` to `a1` degrees."""
    return [(cx + r * math.cos(math.radians(a0 + (a1 - a0) * i / n)),
             cy + r * math.sin(math.radians(a0 + (a1 - a0) * i / n))) for i in range(n + 1)]


def _place(forward=0.0):
    """Stands the model on the floor where it was built (over where it stood) and prints its centre;
    `forward` (game depth, m) moves it towards the front first: its back onto the wall face."""
    for o in G.meshes():
        o.location.y -= forward
    lo, hi = G.measure()
    print(f'[kids] centre (game x, depth) {-(lo[0] + hi[0]) / 2:.4f}, {-(lo[1] + hi[1]) / 2:.4f}')
    G.settle(centre=False)


def _still(name, parts=None):
    """Joins what's in the scene (or `parts`) as the still body, its origin on the floor."""
    return G.join(parts if parts is not None else G.meshes(), name, origin=(0, 0, 0))


def _star(cx, cy, r, inner=0.45, n=5, turn=90.0):
    """A star's outline round (cx, cy), its points `r` out and its notches `inner` of that, a point up."""
    return [(cx + r * (1 if i % 2 == 0 else inner) * math.cos(math.radians(turn + 180 * i / n)),
             cy + r * (1 if i % 2 == 0 else inner) * math.sin(math.radians(turn + 180 * i / n))) for i in range(2 * n)]


def _heart(cx, cy, r, n=20):
    """A heart's outline round (cx, cy), about `r` across from its middle."""
    pts = []
    for i in range(n):
        t = math.tau * i / n
        x = 16 * math.sin(t) ** 3
        y = 13 * math.cos(t) - 5 * math.cos(2 * t) - 2 * math.cos(3 * t) - math.cos(4 * t)
        pts.append((cx + r * x / 16, cy + r * (y + 2) / 16))
    return list(reversed(pts))


def _moon(cx, cy, r, n=10):
    """A crescent moon's outline round (cx, cy): an outer arc and a narrower inner one, its horns to the right."""
    outer = _arc(r, 70, 290, n, cx, cy)
    inner = [(cx + r * 0.42 + r * 0.82 * math.cos(math.radians(a)), cy + r * 0.82 * math.sin(math.radians(a)))
             for a in (255 - 150 * i / n for i in range(n + 1))]
    return outer + inner[1:-1]


def _turned(name, x, z, profile, material, segments=10, y=0.0):
    """A turned part: `profile` [(radius, height over y)] revolved about the upright through game (x, z)."""
    return G.revolve(name, profile, at=F.at(x, y, z), material=material, segments=segments)


def _rumple(obj, amount, above, seed, size=0.2):
    """Rumples a cover's top: its vertices above height `above` moved up or down by smooth noise, up to `amount`,
    never sideways, so its bounds across stay put (as bedroom.py's)."""
    bpy.context.view_layer.update()
    mw = obj.matrix_world
    for v in obj.data.vertices:
        p = mw @ v.co
        if p.z > above:
            v.co.z += vnoise.noise(Vector((p.x / size + seed, p.y / size - seed, seed))) * amount
    obj.data.update()
    return obj


def _mattress(name, w, h, d, top, z, x=0.0, material=None, puff=0.006, piping=0.006, round=0.03, flat=3):
    """A mattress `w` x `d`, `h` deep, the middle of its top at `top` (where the game lays the sleeper): rounded,
    a hair puffed, a piped seam round its top and bottom."""
    return F.soft(name, w, h - 2 * puff, d, (x, top - h / 2, z), material or M.mattress, round=round, puff=puff,
                  flat=flat, piping=piping)


def _soft_pillow(name, w, h, d, x, top, z, material, tilt=8, seed=0.0):
    """A pillow on `top`, `h` thick in the middle: plump, thinning to its seams, a little crumpled."""
    p = F.soft(name, w, h * 0.42, d, (x, top + h / 2 - 0.01, z), material, rot=(tilt, 0, 0),
               round=min(h * 0.2, 0.03), puff=h * 0.29, flat=2)
    return G.jitter(p, 0.004, size=0.12, seed=seed + x * 7, up=0.4)


# --- beds: a duvet whose top third turns down (as bedroom.py) ----------------------------------------------
DUVET = 0.05      # the duvet's top above the mattress top
FLAP = 0.045      # the flap's thickness


def _duvet(top, half, z_fold, z_foot, drape, material, seed=0.0):
    """The duvet's lower two thirds, from the fold to the foot, draped down the sides and the foot end, puffed on
    top (its middle at `DUVET`) and a little rumpled."""
    over, puff = 0.02, 0.01
    y0, y1 = top - drape, top + DUVET
    d = F.soft('duvet', 2 * (half + over), y1 - y0 - 2 * puff, z_foot + over - z_fold,
               (0, (y0 + y1) / 2, (z_fold + z_foot + over) / 2), material, round=0.03, puff=puff, flat=4)
    return _rumple(d, 0.004, top, seed)


def _flap(top, half, z_top, z_fold, material):
    """The duvet's top third with the sheet turned down over its head edge, hinged across its fold."""
    with G.new_parts() as parts:
        y = top + DUVET - FLAP / 2
        F.soft('flap', 2 * (half + 0.012), FLAP - 0.01, z_fold - z_top, (0, y, (z_top + z_fold) / 2), material,
               round=0.018, puff=0.005, flat=3)
        F.soft('flap_sheet', 2 * (half + 0.016), FLAP + 0.008, 0.12, (0, y, z_top + 0.05), M.sheet, round=0.02)
    return G.hinged(G.join(parts, 'duvet_flap'), F.at(0, top + DUVET + 0.003, z_fold))


def _turn_down(top, half, z_top, z_foot, drape, material):
    """Builds the duvet (its still two thirds and the flap), joins the bed, settles, keys `turn_down`."""
    z_fold = z_top + (z_foot - z_top) / 3
    _duvet(top, half, z_fold, z_foot, drape, material)
    _still('bed')
    flap = _flap(top, half, z_top, z_fold, material)
    _place()
    # As it turns it flattens and narrows to stay under the game's quilt (0.46 m from the bed's middle,
    # never lower than 8.5 cm over the mattress), as the bedroom family's beds do (`bedroom.py` `_turn_down`).
    G.swing(flap, 'turn_down', 180, seconds=1.0, axis='X')
    G.squash(flap, 'turn_down', (min(1.0, (0.46 - 0.08) / (half + 0.016)), 1, 0.4), seconds=1.0)
    # half way over it's gathered to a fifth of its length, so it rolls over as cloth, not a board standing up
    G.squash(flap, 'turn_down', (1, 0.2, 1), seconds=1.0, back=True)
    return G.meshes()


def _teddy(x, y, z, material, eye, r=0.06):
    """A little teddy sitting at (x, y, z) (its seat), facing the front: a pear of a body, a head with a snout,
    round ears, stubby arms and legs, bead eyes and nose."""
    F.sphere('teddy_body', r, (x, y + r * 0.95, z), material, scale=(0.95, 0.85, 1.1), segments=12, rings=8)
    F.sphere('teddy_head', r * 0.78, (x, y + r * 2.6, z + r * 0.05), material, segments=12, rings=8)
    F.sphere('teddy_snout', r * 0.32, (x, y + r * 2.45, z + r * 0.75), material, scale=(1.1, 1, 0.8), segments=8, rings=6)
    F.sphere('teddy_nose', r * 0.12, (x, y + r * 2.55, z + r * 1.0), eye, segments=6, rings=4)
    for s in (-1, 1):
        F.sphere(f'teddy_ear{s}', r * 0.28, (x + s * r * 0.58, y + r * 3.2, z), material, scale=(1, 1, 0.6),
                 segments=8, rings=6)
        F.sphere(f'teddy_eye{s}', r * 0.09, (x + s * r * 0.3, y + r * 2.8, z + r * 0.68), eye, segments=6, rings=4)
        F.sweep(f'teddy_arm{s}', [(x + s * r * 0.7, y + r * 1.5, z), (x + s * r * 0.95, y + r * 0.9, z + r * 0.5)],
                r * 0.26, material, verts=8, scale=[1.0, 0.9])
        F.sweep(f'teddy_leg{s}', [(x + s * r * 0.45, y + r * 0.35, z + r * 0.2), (x + s * r * 0.55, y + r * 0.3, z + r * 1.2)],
                r * 0.32, material, verts=8, scale=[1.0, 1.05])


# --- KD1: the crib and its changing table ------------------------------------------------------------------
# An oblong cot, a 1.18 x 0.58 m mattress (a 60 x 120 cot's; review M-kids-01: it was 0.80 m square) running
# across the left of the 2 x 1 tiles, its back on the wall's face, and a changing chest at the right. The baby lies
# along it (characters/furniture.json `beds.crib` `across`) at slot 0 (game x -0.5), the carers reach in over the
# front rail (characters.ts `CUDDLE.rail`).
CRIB_X = -0.5     # where the baby lies (slot 0 of the 2 x 1)
CRIB_TOP = 0.56   # the mattress top (characters/furniture.json beds.crib)
COT_X = (-0.955, 0.275)    # the cot's end posts (game x)
COT_Z = (-0.405, 0.225)    # its back and front posts (game depth): the back on the wall's face
COT_MID = (sum(COT_X) / 2, sum(COT_Z) / 2)
COT_LEN = COT_X[1] - COT_X[0] - 0.05     # between the posts
COT_DEP = COT_Z[1] - COT_Z[0] - 0.05


def _spindles(name, a, b, along, material, n, y0=0.47, y1=0.9):
    """`n` round spindles between `a` and `b` along x (`along` 'x') or depth ('z'), the other coordinate fixed,
    from rail to rail."""
    for i in range(n):
        t = a + (b - a) * (i + 0.5) / n
        x, z = (t, along[1]) if along[0] == 'x' else (along[1], t)
        F.sweep(f'{name}{i}', [(x, y0, z), (x, y1, z)], 0.011, material, verts=8)


def _crib_rails(name, z):
    """A long side's top and bottom rails and spindles, between the posts: the top rail rounded over to hold."""
    F.sweep(f'{name}_top', [(COT_X[0] + 0.02, 0.9, z), (COT_X[1] - 0.02, 0.9, z)], None, M.crib,
            profile=G.rounded_rect(0.042, 0.04, 0.016, corner=2), up=(0, 0, 1))
    F.box(f'{name}_bottom', COT_LEN + 0.01, 0.04, 0.035, (COT_MID[0], 0.47, z), M.crib, bevel=0.008, bevel_segments=1)
    _spindles(f'{name}_spindle', COT_X[0] + 0.03, COT_X[1] - 0.03, ('x', z), M.crib, 11)


def _finial(name, x, z, y, material):
    """A turned finial on a post's top at height `y`: a collar, a waist and a ball."""
    return _turned(name, x, z, [(0.0, 0.0), (0.025, 0.0), (0.025, 0.006), (0.012, 0.016), (0.02, 0.026),
                                (0.021, 0.036), (0.013, 0.047), (0.0, 0.05)], material, segments=8, y=y)


def _crib_end(name, x, hole):
    """A solid end board between the posts at `x`, its top arched, a shape cut through it."""
    z0, z1 = COT_Z[0] + 0.025, COT_Z[1] - 0.025
    n = 10
    top = [(z1 - (z1 - z0) * i / n, 0.885 + 0.04 * math.sin(math.pi * i / n)) for i in range(n + 1)]
    F.extrude(name, [(z0, 0.43), (z1, 0.43)] + top, 0.022, (x, 0, 0), M.crib, plane='side', holes=[hole],
              bevel=0.006, bevel_segments=1)


def _changing_table():
    x, z, w = 0.635, -0.18, 0.62
    F.box('plinth', w - 0.06, 0.05, 0.44, (x, 0.025, z - 0.01), M.crib, bevel=0.006, bevel_segments=1)
    F.box('chest', w - 0.02, 0.73, 0.5, (x, 0.415, z), M.crib, bevel=0.01, bevel_segments=2)
    for i, y in enumerate((0.165, 0.4, 0.635)):
        G.kit.shaker(F, f'drawer{i}', w - 0.08, 0.215, (x, y, z + 0.259), 0.018, M.crib_wood, rail=0.035, recess=0.004)
        G.kit.knob(F, f'knob{i}', x, y, z + 0.268, 0.014, M.crib, segments=8)
    F.box('chest_top', w + 0.02, 0.025, 0.52, (x, 0.7925, z), M.crib_wood, bevel=0.008, bevel_segments=2)
    # a contoured changing pad: a soft middle between two raised bolsters, a guard rail round its back and sides
    F.soft('pad', w - 0.1, 0.035, 0.42, (x, 0.825, z + 0.01), M.crib_bedding, round=0.014, puff=0.006, flat=2)
    for s in (-1, 1):
        F.soft(f'pad_side{s}', 0.075, 0.06, 0.42, (x + s * (w / 2 - 0.085), 0.838, z + 0.01), M.crib_bedding,
               round=0.026, flat=1)
    gx0, gx1, gz0, gz1 = x - w / 2 + 0.0125, x + w / 2 - 0.0125, z - 0.245, z + 0.25
    pts = G.kit.fillet([(gx0, 0.845, gz1), (gx0, 0.845, gz0), (gx1, 0.845, gz0), (gx1, 0.845, gz1)], 0.04)
    F.sweep('guard', pts, None, M.crib, profile=G.rounded_rect(0.08, 0.025, 0.011, corner=2), up=(0, 0, 1))


def _mobile():
    """The mobile: a turned hub on the arm's end, four arms curving down, and a toy on a string under each (a star,
    a heart, a moon and a ball), each facing out, turning about the hub."""
    hx, hy, hz = CRIB_X, 0.975, -0.17   # over the baby, clear of arms reaching in over the front rail
    with G.new_parts() as parts:
        _turned('hub', hx, hz, [(0.0, 0.0), (0.03, 0.0), (0.032, 0.007), (0.026, 0.018), (0.012, 0.026), (0.0, 0.028)],
                M.mobile, segments=12, y=hy - 0.0125)
        toys = (('star', M.mobile), ('heart', M.mobile_pink), ('moon', M.mobile_blue), ('ball', M.mobile_pink))
        for k, (kind, m) in enumerate(toys):
            a = math.radians(90 * k + 45)
            c, s = math.cos(a), math.sin(a)
            x, z = hx + 0.12 * c, hz + 0.12 * s
            F.sweep(f'arm{k}', [(hx, hy, hz), (hx + 0.06 * c, hy + 0.004, hz + 0.06 * s), (x, hy - 0.012, z)], 0.0045,
                    M.mobile, verts=6, resolution=3)
            F.sweep(f'string{k}', [(x, hy - 0.012, z), (x, 0.872, z)], 0.0015, M.crib, verts=4)
            turn = (0, 0, math.degrees(a) - 90)   # its face out from the hub
            if kind == 'ball':
                F.sphere(f'ball{k}', 0.03, (x, 0.845, z), m, segments=12, rings=8)
                continue
            outline = {'star': _star(0, 0, 0.042), 'heart': _heart(0, 0, 0.034), 'moon': _moon(0, 0, 0.036)}[kind]
            F.extrude(f'{kind}{k}', outline, 0.014, (x, 0.84, z), m, rot=turn, bevel=0.004, bevel_segments=1)
    return G.hinged(G.join(parts, 'mobile'), F.at(hx, hy, hz))


def crib():
    cx, cz = COT_MID
    for x in COT_X:
        for z in COT_Z:
            F.box(f'post{x}{z}', 0.05, 0.92, 0.05, (x, 0.46, z), M.crib, bevel=0.01, bevel_segments=2)
            _finial(f'cap{x}{z}', x, z, 0.92, M.crib)
    F.box('platform', COT_LEN + 0.04, 0.04, COT_DEP + 0.04, (cx, 0.44, cz), M.crib, bevel=0.008, bevel_segments=1)
    _mattress('mattress', COT_LEN, 0.1, COT_DEP, CRIB_TOP, cz, x=cx, material=M.crib_bedding, round=0.025, flat=2)
    # the blanket folded at the foot end (the baby's head to the left, characters.ts `BEDS` `across`), a band near
    # its fold
    b = F.soft('blanket', 0.2, 0.028, COT_DEP - 0.08, (COT_X[1] - 0.17, CRIB_TOP + 0.017, cz), M.crib_blanket,
               round=0.012, puff=0.005, flat=2)
    G.jitter(b, 0.002, size=0.1, seed=3.0, up=0.5)
    F.box('blanket_stripe', 0.03, 0.036, COT_DEP - 0.076, (COT_X[1] - 0.22, CRIB_TOP + 0.017, cz), M.mobile_blue,
          bevel=0.012, bevel_segments=2)
    _crib_rails('back', COT_Z[0])
    # the ends: solid boards between the posts, arched, a heart cut through the head's and a star the foot's
    _crib_end('end_head', COT_X[0], [(COT_Z[0] + 0.315 + u, 0.71 + v) for u, v in _heart(0, 0, 0.07)])
    _crib_end('end_foot', COT_X[1], [(COT_Z[0] + 0.315 + u, 0.7 + v) for u, v in _star(0, 0, 0.08)])
    # the mobile's arm, bent from one tube and clamped to the back rail
    F.box('clamp', 0.035, 0.07, 0.06, (CRIB_X, 0.9, COT_Z[0] + 0.005), M.mobile, bevel=0.008, bevel_segments=1)
    pts = G.kit.fillet([(CRIB_X, 0.88, -0.37), (CRIB_X, 1.0, -0.37), (CRIB_X, 1.0, -0.17), (CRIB_X, 0.985, -0.17)], 0.05)
    F.sweep('arm', pts, 0.008, M.mobile, verts=8)
    _changing_table()
    _still('body')

    with G.new_parts() as parts:
        _crib_rails('front', COT_Z[1])
    side = G.join(parts, 'front_rail')
    mobile = _mobile()
    _place()   # built in place: the back posts on the wall's face
    G.slide(side, 'side_down', (0, 0, -0.25), seconds=0.9)
    G.spin(mobile, 'mobile', turns=1, seconds=6.0, axis='Z')
    return G.meshes()


# --- KD1: the kids' bed --------------------------------------------------------------------------------------
KIDS_TOP = 0.44   # the mattress top (characters.ts BEDS.kidsBed)


def _bun(name, x, z, h, r, material, segments=10):
    """A turned bun foot `h` tall, `r` round."""
    return _turned(name, x, z, [(0.0, 0.0), (r * 0.8, 0.0), (r, h * 0.2), (r * 1.02, h * 0.45), (r * 0.82, h * 0.7),
                                (r * 0.9, h * 0.85), (r * 0.9, h), (0.0, h)], material, segments=segments)


def kidsBed():
    for sx in (-1, 1):
        for sz in (-1, 1):
            _bun(f'foot{sx}{sz}', sx * 0.4, sz * 0.86, 0.07, 0.032, M.kids_frame)
    # the base: a box with a rounded rim round its top, two drawers under the bed on its near side
    F.box('base', 0.94, 0.24, 1.88, (0, 0.18, 0.01), M.kids_frame, bevel=0.02, bevel_segments=2)
    rim = G.rounded_rect(0.93, 1.85, 0.06, corner=3)
    F.sweep('rim', [(u, 0.3, 0.01 + v) for u, v in rim], None, M.kids_frame, profile=G.rounded_rect(0.034, 0.03, 0.012, corner=1),
            up=(0, 0, 1), closed=True, angle=60)
    for k, zc in enumerate((-0.42, 0.44)):
        F.box(f'drawer{k}', 0.012, 0.17, 0.8, (-0.474, 0.175, zc), M.kids_drawer, bevel=0.006, bevel_segments=1)
        F.extrude(f'drawer_grip{k}', G.rounded_rect(0.13, 0.035, 0.017, corner=3), 0.004, (-0.4785, 0.22, zc),
                  M.kids_frame, plane='side')
    _mattress('mattress', 0.86, 0.16, 1.8, KIDS_TOP, 0.02)
    # a headboard with an arched top, a moon and two stars cut through it, and bunting draped across it; its back
    # on the wall's face (depth -0.93)
    arch = [(0.48 * math.cos(math.pi * i / 16), 0.5 + 0.264 * math.sin(math.pi * i / 16)) for i in range(17)]
    holes = [_moon(0.0, 0.675, 0.048), _star(-0.21, 0.655, 0.036), _star(0.21, 0.655, 0.036)]
    F.extrude('headboard', [(-0.48, 0.07), (0.48, 0.07)] + arch, 0.04, (0, 0, -0.91), M.kids_frame, holes=holes,
              bevel=0.008, bevel_segments=2)
    sag = [(-0.43 + 0.86 * i / 10, 0.6 - 0.035 * math.sin(math.pi * i / 10)) for i in range(11)]
    F.sweep('bunting', [(x, y, -0.886) for x, y in sag], 0.003, M.kids_bedding, verts=5, resolution=2)
    flags = (M.pennant_a, M.pennant_b, M.pennant_c, M.pennant_a, M.pennant_b)
    for k, m in enumerate(flags):
        x = -0.3 + 0.15 * k
        y = 0.6 - 0.035 * math.sin(math.pi * (x + 0.43) / 0.86)
        F.extrude(f'pennant{k}', [(-0.042, 0.0), (0.042, 0.0), (0.0, -0.075)], 0.003, (x, y + 0.002, -0.886), m,
                  rot=(0, 0, (k - 2) * 2.5))
    # a guard rail along the right side, bent from one tube, two spindles in it
    pts = G.kit.fillet([(0.475, 0.3, -0.62), (0.475, 0.675, -0.62), (0.475, 0.675, 0.22), (0.475, 0.3, 0.22)], 0.06)
    F.sweep('rail', pts, 0.015, M.kids_frame, verts=10)
    for z in (-0.34, -0.06):
        F.sweep(f'rail_spindle{z}', [(0.475, 0.3, z), (0.475, 0.665, z)], 0.01, M.kids_frame, verts=8)
    _soft_pillow('pillow', 0.58, 0.12, 0.3, 0, KIDS_TOP, -0.72, M.sheet, seed=1.0)
    _teddy(-0.33, KIDS_TOP - 0.004, -0.79, M.teddy_small, M.eye)
    return _turn_down(KIDS_TOP, 0.435, -0.5, 0.93, 0.12, M.kids_bedding)


# --- KD1: the race car bed -----------------------------------------------------------------------------------
CAR_TOP = 0.5     # the mattress top (the renderer's default bed height)
CAR_SILL = 0.45   # the cockpit's sides, either side of the mattress


def _car_arc(cz, cy, r, a0, a1, n):
    return [(cz + r * math.cos(math.radians(a0 + (a1 - a0) * i / n)), cy + r * math.sin(math.radians(a0 + (a1 - a0) * i / n)))
            for i in range(n + 1)]


def _car_side():
    """The car's side seen from the right (game depth, height): a low nose rounding up to the bonnet, the sill along
    the cockpit, the tail rising behind, and an arch over each wheel."""
    out = [(-0.93, 0.6), (-0.93, 0.09)] + _car_arc(-0.91, 0.09, 0.02, 180, 270, 2)[1:]
    for wz in (-0.6, 0.6):
        out += _car_arc(wz, 0.18, 0.215, 211.6, -31.6, 12)
    out += _car_arc(0.9, 0.14, 0.07, 270, 360, 4) + _car_arc(0.89, 0.46, 0.08, 0, 90, 4)
    out += [(0.66, 0.542), (0.63, 0.5), (0.6, CAR_SILL + 0.02)]
    out += [(-0.74, CAR_SILL + 0.02), (-0.79, CAR_SILL + 0.06)] + _car_arc(-0.86, 0.56, 0.05, 30, 90, 3)
    return out


def _wheel(name, x, z, side):
    """A wheel at (x, z): a rounded tyre and a dished hub with a cap, its axle across."""
    tyre = [(0.105, -0.04), (0.16, -0.04), (0.174, -0.035), (0.18, -0.02), (0.18, 0.02), (0.174, 0.035), (0.16, 0.04),
            (0.105, 0.04)]
    hub = [(0.0, 0.0425), (0.028, 0.0425), (0.034, 0.036), (0.08, 0.032), (0.098, 0.036), (0.106, 0.03),
           (0.106, -0.03), (0.0, -0.03)]
    for part, prof, m in (('tyre', tyre, M.dark), ('hub', hub, M.hub)):
        G.revolve(f'{part}{name}', prof, at=F.at(x, 0.18, z), material=m, segments=16, rot=(0, -90 * side, 0))


def kidsBedCar():
    # the body: the two sides (arched over the wheels), the bonnet and tail between them, the deck under the
    # mattress; the tail end's back on the wall's face (depth -0.93)
    side = _car_side()
    for s in (-1, 1):
        F.extrude(f'side{s}', side, 0.06, (s * 0.44, 0, 0), M.car, plane='side', bevel=0.02, bevel_segments=1)
    nose = [(0.6, 0.12), (0.92, 0.12), (0.97, 0.17)] + _car_arc(0.89, 0.46, 0.08, 0, 90, 4) + [(0.66, 0.542), (0.6, 0.5)]
    F.extrude('bonnet', nose, 0.84, (0, 0, 0), M.car, plane='side', bevel=0.03, bevel_segments=2)
    F.extrude('tail', [(-0.93, 0.32), (-0.79, 0.32), (-0.79, CAR_SILL + 0.06)] + _car_arc(-0.86, 0.56, 0.05, 30, 90, 3)
              + [(-0.93, 0.61)], 0.84, (0, 0, 0), M.car, plane='side', bevel=0.02, bevel_segments=2)
    F.box('deck', 0.84, 0.26, 1.42, (0, 0.21, -0.08), M.car, bevel=0.01, bevel_segments=1)
    _mattress('mattress', 0.8, 0.16, 1.4, CAR_TOP, -0.08)
    # the bonnet: two stripes over it, a grille with bars, headlights in their rims
    for s in (-1, 1):
        F.extrude(f'stripe{s}', G.rounded_rect(0.08, 0.22, 0.01, corner=1), 0.004, (s * 0.09, 0.543, 0.77), M.car_stripe,
                  plane='top')
        G.sphere(f'headlight{s}', 0.06, at=F.at(s * 0.3, 0.45, 0.965), material=M.headlight, scale=(1, 0.35, 0.8),
                 segments=12, rings=6)
        F.sweep(f'headlight_rim{s}', [(s * 0.3 + 0.064 * math.cos(math.tau * i / 12), 0.45 + 0.05 * math.sin(math.tau * i / 12),
                                       0.97) for i in range(12)], 0.007, M.hub, verts=5, closed=True)
        # a number roundel on each side, a 7 on it
        F.cyl(f'roundel{s}', 0.07, 0.006, (s * 0.473, 0.3, 0.0), M.car_stripe, rot=(0, 90, 0), verts=20)
        seven = [(-0.03, 0.035), (0.032, 0.035), (0.032, 0.022), (-0.004, -0.04), (-0.02, -0.04), (0.014, 0.022),
                 (-0.03, 0.022)]
        F.extrude(f'number{s}', [(u * s, v) for u, v in seven], 0.004, (s * 0.477, 0.3, 0.0), M.dark, plane='side')
        # a racing stripe down each side, under the sill
        F.extrude(f'side_stripe{s}', G.rounded_rect(1.0, 0.03, 0.012, corner=2), 0.003, (s * 0.4715, 0.4, -0.02),
                  M.car_stripe, plane='side')
    F.extrude('grille', G.rounded_rect(0.38, 0.11, 0.03, corner=3), 0.012, (0, 0.3, 0.97), M.dark, plane='front')
    for k in range(3):
        F.box(f'grille_bar{k}', 0.34, 0.01, 0.008, (0, 0.27 + 0.03 * k, 0.978), M.hub, bevel=0.003, bevel_segments=1)
    # the tail's spoiler: a wing on two struts between end plates
    for s in (-1, 1):
        F.extrude(f'strut{s}', [(-0.885, 0.6), (-0.865, 0.6), (-0.885, 0.72), (-0.905, 0.72)], 0.03, (s * 0.3, 0, 0), M.dark,
                  plane='side', bevel=0.004, bevel_segments=1)
        F.extrude(f'endplate{s}', G.rounded_rect(0.09, 0.09, 0.02, corner=2), 0.018, (s * 0.4, 0.73, -0.885), M.dark,
                  plane='side', bevel=0.004, bevel_segments=1)
    # an aerofoil, thick at its leading edge (u up, v back from it)
    wing = [(0.0, 0.0), (0.012, 0.004), (0.018, 0.02), (0.016, 0.045), (0.008, 0.077), (0.002, 0.077), (-0.004, 0.05),
            (-0.006, 0.02), (-0.004, 0.004)]
    F.sweep('wing', [(-0.39, 0.725, -0.852), (0.39, 0.725, -0.852)], None, M.dark, profile=wing, up=(0, 0, 1), angle=50)
    for sx in (-1, 1):
        for sz in (-1, 1):
            _wheel(f'{sx}{sz}', sx * 0.45, sz * 0.6, sx)
    _soft_pillow('pillow', 0.5, 0.12, 0.26, 0, CAR_TOP, -0.64, M.sheet, tilt=12, seed=2.0)
    return _turn_down(CAR_TOP, 0.4, -0.42, 0.66, 0.1, M.car_bedding)


# --- KD1: the high chair -------------------------------------------------------------------------------------
def _on_leg(sx, zf, zt, y, top=0.575, foot=0.008):
    """The point at height `y` on a high chair's leg from (sx * 0.25, foot, zf) up to (sx * 0.14, top, zt)."""
    t = (y - foot) / (top - foot)
    return (sx * (0.25 + (0.14 - 0.25) * t), y, zf + (zt - zf) * t)


def highChair():
    # the legs splayed wide at the floor, 0.54 m each way, so it can't tip (review M-kids-04: 0.44 x 0.47); each
    # tapers to a rubber foot, a stretcher between each side's pair, a footrest across the front pair
    legs = ((-0.2, -0.07), (0.32, 0.17))
    for sx in (-1, 1):
        for zf, zt in legs:
            F.sweep(f'leg{sx}{zf}', [(sx * 0.14, 0.575, zt), (sx * 0.2495, 0.012, zf)], 0.0175, M.chair_legs, verts=10,
                    scale=[1.15, 1.0])
            F.cyl(f'foot{sx}{zf}', 0.016, 0.014, (sx * 0.25, 0.007, zf), M.dark, verts=8)
        a, b = _on_leg(sx, *legs[0], 0.2), _on_leg(sx, *legs[1], 0.2)
        F.sweep(f'stretcher{sx}', [a, b], 0.011, M.chair_legs, verts=8)
    fa, fb = _on_leg(-1, *legs[1], 0.32), _on_leg(1, *legs[1], 0.32)
    F.sweep('footrest_bar', [fa, fb], 0.012, M.chair_legs, verts=8)
    F.extrude('footrest', G.rounded_rect(0.34, 0.09, 0.03, corner=3), 0.02, (0, 0.34, 0.225), M.chair_white, plane='top',
              bevel=0.006, bevel_segments=1)
    # the seat: a moulded shell, a cushion in it; a curved back with its own cushion, rolled arms
    F.soft('seat', 0.38, 0.05, 0.36, (0, 0.58, 0.05), M.chair_white, round=0.02, flat=1)
    F.soft('cushion', 0.31, 0.03, 0.3, (0, 0.618, 0.06), M.chair_cushion, round=0.012, puff=0.007, piping=0.004, flat=2,
           round_segments=1)
    F.extrude('back', G.rounded_rect(0.36, 0.32, 0.07, corner=3), 0.03, (0, 0.78, -0.12), M.chair_white, rot=(-6, 0, 0),
              bevel=0.01, bevel_segments=2)
    F.soft('back_cushion', 0.28, 0.23, 0.022, (0, 0.77, -0.097), M.chair_cushion, rot=(-6, 0, 0), round=0.01, puff=0.006,
           piping=0.004, flat=2, round_segments=1)
    for s in (-1, 1):
        pts = G.kit.fillet([(s * 0.17, 0.6, -0.1), (s * 0.17, 0.75, -0.1), (s * 0.17, 0.75, 0.16), (s * 0.17, 0.6, 0.2)], 0.05)
        F.sweep(f'arm{s}', pts, None, M.chair_white, profile=G.rounded_rect(0.034, 0.04, 0.014, corner=1), up=(1, 0, 0))
    # a harness: two straps down from the back to a buckle on the cushion
    for s in (-1, 1):
        F.sweep(f'strap{s}', [(s * 0.07, 0.85, -0.098), (s * 0.06, 0.7, -0.09), (s * 0.03, 0.645, 0.0), (0, 0.64, 0.07)],
                None, M.chair_cushion, profile=G.rounded_rect(0.004, 0.03, 0.002, corner=1), up=(0, 1, 0), resolution=2)
    F.box('buckle', 0.04, 0.012, 0.04, (0, 0.645, 0.08), M.dark, bevel=0.005, bevel_segments=1)
    _still('body')
    # the tray, its own part, sliding out to let the baby in: a rim round it, a cup well, a bowl
    with G.new_parts() as parts:
        F.extrude('tray', G.rounded_rect(0.44, 0.17, 0.05, corner=4), 0.02, (0, 0.792, 0.2), M.chair_white, plane='top',
                  bevel=0.006, bevel_segments=1)
        rim = G.rounded_rect(0.424, 0.154, 0.042, corner=3)
        F.sweep('tray_rim', [(u, 0.804, 0.2 + v) for u, v in rim], 0.006, M.chair_white, verts=6, closed=True)
        F.extrude('tray_mat', G.rounded_rect(0.24, 0.11, 0.03, corner=3), 0.003, (-0.05, 0.8035, 0.2), M.tray_mat,
                  plane='top')
        _turned('bowl', 0.13, 0.2, [(0.0, 0.0), (0.022, 0.0), (0.034, 0.012), (0.04, 0.03), (0.036, 0.031), (0.03, 0.016),
                                    (0.0, 0.012)], M.toy_green, segments=12, y=0.802)
        F.sweep('spoon', [(0.06, 0.806, 0.17), (0.1, 0.81, 0.23), (0.15, 0.83, 0.235)], 0.0035, M.toy_yellow, verts=5)
    tray = G.join(parts, 'tray')
    _place()
    # out along the arms as the baby is put in (and taken out), then back in front of them
    G.slide(tray, 'tray_out', (0, -0.16, 0), seconds=0.5)
    return G.meshes()


# --- KD2: the toy box and the toy chest ----------------------------------------------------------------------
def _walls(name, w, d, y0, y1, z, t, material, r=0.03, x=0.0):
    """An open box's four walls in one piece, `t` thick, w x d outside, their corners rounded, from y0 up to y1."""
    out = G.rounded_rect(w, d, r, corner=3)
    hole = G.rounded_rect(w - 2 * t, d - 2 * t, max(0.004, r - t), corner=3)
    return F.extrude(name, out, y1 - y0, (x, (y0 + y1) / 2, z), material, plane='top', holes=[hole])


def _block(name, x, y, z, s, material, face, turn=0.0, tip=0.0):
    """A toy block `s` across standing on its bottom at height `y`: rounded, a raised square on its faces."""
    cube = F.box(name, s, s, s, (x, y + s / 2, z), material, rot=(tip, 0, turn), bevel=s * 0.12, bevel_segments=2)
    face_obj = F.box(f'{name}_face', s * 0.62, s * 0.62, s * 1.04, (x, y + s / 2, z), face, rot=(tip, 0, turn),
                     bevel=s * 0.05, bevel_segments=1)
    return cube, face_obj


def _toys_inside(top, z):
    """Toys heaped inside a box, seen with the lid open: a ball, two blocks, a teddy and a toy car's wheels."""
    G.sphere('in_ball', 0.075, at=F.at(0.16, top - 0.07, z + 0.05), material=M.ball_blue, segments=14, rings=8)
    F.sphere('in_ball_band', 0.0755, (0.16, top - 0.07, z + 0.05), M.ball_yellow, scale=(1, 0.3, 1), segments=14, rings=4)
    _block('in_block', -0.2, top - 0.145, z + 0.02, 0.09, M.toy_red, M.toy_yellow, turn=20)
    _block('in_block2', -0.08, top - 0.14, z - 0.08, 0.08, M.toy_green, M.paper, turn=-15, tip=10)
    _teddy(0.0, top - 0.17, z + 0.07, M.plush, M.eye, r=0.05)


def toyBox():
    z = -0.18
    F.box('box_floor', 0.78, 0.03, 0.48, (0, 0.03, z), M.toybox_inside)
    _walls('box_walls', 0.8, 0.5, 0.0, 0.445, z, 0.025, M.toybox)
    # hand holds cut through the ends
    walls = bpy.data.objects['box_walls']
    for s in (-1, 1):
        G.cut(walls, F.extrude(f'grip{s}', G.rounded_rect(0.13, 0.035, 0.0175, corner=3), 0.1, (s * 0.4, 0.37, z),
                               M.toybox, plane='side'))
    # a rounded rim along the walls' top edge, under the lid
    rim = G.rounded_rect(0.775, 0.475, 0.018, corner=3)
    F.sweep('box_rim', [(u, 0.445, z + v) for u, v in rim], None, M.toybox, profile=G.rounded_rect(0.016, 0.03, 0.007, corner=1),
            up=(0, 0, 1), closed=True)
    F.box('fill', 0.75, 0.01, 0.45, (0, 0.31, z), M.toybox_inside)
    # the front: a star, a circle and a triangle set into it, each in its frame
    shapes_ = (_star(0, 0, 0.06), [(0.055 * math.cos(math.tau * i / 16), 0.055 * math.sin(math.tau * i / 16)) for i in range(16)],
               [(0.0, 0.06), (-0.06, -0.045), (0.06, -0.045)])
    for k, (x, m, outline) in enumerate(((-0.25, M.toy_red, shapes_[0]), (0.0, M.toy_green, shapes_[1]),
                                         (0.25, M.toy_blue, shapes_[2]))):
        F.extrude(f'tile{k}', G.rounded_rect(0.16, 0.15, 0.025, corner=2), 0.008, (x, 0.24, 0.072), M.toybox_inside,
                  bevel=0.003, bevel_segments=1)
        F.extrude(f'shape{k}', outline, 0.012, (x, 0.24, 0.074), m, bevel=0.003, bevel_segments=1)
    _toys_inside(0.46, z)
    # blocks left out on the floor in front
    _block('floor_block0', 0.32, 0.0, 0.25, 0.1, M.toy_green, M.toy_yellow)
    _block('floor_block1', 0.2, 0.0, 0.3, 0.1, M.toy_red, M.paper, turn=30)
    _block('floor_block2', 0.27, 0.1, 0.27, 0.1, M.toy_blue, M.toy_yellow, turn=12)
    _still('body')
    with G.new_parts() as parts:
        # the lid: a rounded board, a finger notch in its front edge, a piano hinge along its back
        notch = [(0.05 - 0.1 * i / 8, 0.25 - 0.022 * math.sin(math.pi * i / 8)) for i in range(9)]
        outline = [(0.41, -0.258), (0.41, 0.25)] + notch + [(-0.41, 0.25), (-0.41, -0.258)]
        F.extrude('lid', outline, 0.042, (0, 0.486, z), M.toybox_lid, plane='top', bevel=0.012, bevel_segments=2)
        F.extrude('lid_panel', G.rounded_rect(0.66, 0.36, 0.03, corner=3), 0.006, (0, 0.508, z - 0.01), M.toybox_lid,
                  plane='top', bevel=0.003, bevel_segments=1)
        F.cyl('hinge', 0.006, 0.7, (0, 0.462, z - 0.252), M.handle, rot=(0, 90, 0), verts=8)
    lid = G.hinged(G.join(parts, 'lid'), F.at(0, 0.46, z - 0.26))
    _place()
    G.swing(lid, 'lid_open', -100, seconds=1.0, axis='X')
    return G.meshes()


def _arch(z0, z1, y0, rise, n=10):
    """A domed lid's cross-section seen from the side (game depth, height): flat from z0 to z1 at y0, arched `rise`
    over it."""
    pts = [(z0, y0), (z1, y0)]
    for i in range(n + 1):
        a = math.pi * i / n
        pts.append(((z0 + z1) / 2 + (z1 - z0) / 2 * math.cos(a), y0 + 0.012 + (rise - 0.012) * math.sin(a)))
    return pts


def toyChest():
    z = -0.18
    for sx in (-1, 1):
        for sz in (-1, 1):
            _bun(f'foot{sx}{sz}', sx * 0.36, z + sz * 0.19, 0.045, 0.03, M.chest_trim, segments=8)
    F.box('box_floor', 0.78, 0.03, 0.44, (0, 0.055, z), M.chest_inside)
    _walls('box_walls', 0.8, 0.46, 0.04, 0.44, z, 0.025, M.chest, r=0.012)
    F.box('fill', 0.75, 0.01, 0.41, (0, 0.29, z), M.chest_inside)
    # planks: grooves across the front and the ends
    for y in (0.17, 0.3):
        F.box(f'groove_front{y}', 0.79, 0.006, 0.004, (0, y, z + 0.23), M.chest_inside)
        for s in (-1, 1):
            F.box(f'groove_end{y}{s}', 0.004, 0.006, 0.45, (s * 0.4, y, z), M.chest_inside)
    # the bands and straps, iron corners, a bail handle on each end
    F.box('band', 0.81, 0.05, 0.012, (0, 0.33, z + 0.235), M.chest_trim, bevel=0.004, bevel_segments=1)
    for s in (-1, 1):
        F.box(f'strap{s}', 0.05, 0.4, 0.012, (s * 0.3, 0.24, z + 0.235), M.chest_trim, bevel=0.004, bevel_segments=1)
        F.box(f'band_side{s}', 0.012, 0.05, 0.47, (s * 0.405, 0.33, z), M.chest_trim, bevel=0.004, bevel_segments=1)
        for k, (y0, dy) in enumerate(((0.04, 0.075), (0.44, -0.075))):   # iron corner plates on the front
            F.extrude(f'corner{s}{k}', [(s * 0.4, y0), (s * 0.4 - s * 0.065, y0), (s * 0.4, y0 + dy)], 0.004, (0, 0, z + 0.232),
                      M.handle)
        F.box(f'handle_plate{s}', 0.006, 0.05, 0.12, (s * 0.403, 0.4, z), M.handle, bevel=0.003, bevel_segments=1)
        pts = G.kit.fillet([(s * 0.41, 0.4, z - 0.045), (s * 0.416, 0.355, z - 0.045), (s * 0.416, 0.355, z + 0.045),
                            (s * 0.41, 0.4, z + 0.045)], 0.012)
        F.sweep(f'handle{s}', pts, 0.006, M.handle, verts=6)
    _toys_inside(0.44, z)
    _still('body')
    with G.new_parts() as parts:
        # the lid, domed from back to front, bound with straps over the dome, a hasp and lock plate at its front
        F.extrude('lid', _arch(z - 0.25, z + 0.25, 0.44, 0.085), 0.84, (0, 0, 0), M.chest, plane='side', bevel=0.01,
                  bevel_segments=1)
        for k, x in enumerate((-0.3, 0.3, -0.416, 0.416)):
            w = 0.05 if k < 2 else 0.012
            band = [(z + 0.246 * math.cos(math.pi * i / 10), 0.452 + 0.077 * math.sin(math.pi * i / 10)) for i in range(11)]
            F.sweep(f'lid_strap{k}', [(x, y, d) for d, y in band], None, M.chest_trim,
                    profile=G.rounded_rect(w, 0.008, 0.002, corner=1), up=(1, 0, 0))
        F.box('lid_band', 0.84, 0.03, 0.012, (0, 0.455, z + 0.249), M.chest_trim, bevel=0.004, bevel_segments=1)
        F.extrude('latch', G.rounded_rect(0.06, 0.075, 0.012, corner=2), 0.008, (0, 0.44, z + 0.257), M.handle,
                  bevel=0.002, bevel_segments=1)
        F.extrude('keyhole', [(0.0, 0.012), (0.007, 0.006), (0.004, -0.018), (-0.004, -0.018), (-0.007, 0.006)], 0.003,
                  (0, 0.43, z + 0.2615), M.eye)
    lid = G.hinged(G.join(parts, 'lid'), F.at(0, 0.44, z - 0.25))
    _place()
    G.swing(lid, 'lid_open', -100, seconds=1.0, axis='X')
    return G.meshes()


# --- KD2: the doll house -------------------------------------------------------------------------------------
ROOF_RISE = 0.18


def _window(name, x, y, z, w, h, frame, glass, shutters=None):
    """A window on a front at depth `z`: a moulded frame round its glass, glazing bars, a sill, and shutters."""
    F.extrude(f'{name}_frame', _rect(-w / 2, -h / 2, w / 2, h / 2), 0.008, (x, y, z), frame,
              holes=[_rect(-w / 2 + 0.012, -h / 2 + 0.012, w / 2 - 0.012, h / 2 - 0.012)], bevel=0.002, bevel_segments=1)
    F.box(f'{name}_glass', w - 0.022, h - 0.022, 0.004, (x, y, z - 0.002), glass)
    F.box(f'{name}_bar_v', 0.007, h - 0.022, 0.004, (x, y, z + 0.001), frame)
    F.box(f'{name}_bar_h', w - 0.022, 0.007, 0.004, (x, y, z + 0.001), frame)
    F.box(f'{name}_sill', w + 0.02, 0.01, 0.016, (x, y - h / 2 - 0.004, z + 0.004), frame, bevel=0.003, bevel_segments=1)
    if shutters:
        for s in (-1, 1):
            F.box(f'{name}_shutter{s}', w * 0.42, h, 0.005, (x + s * (w / 2 + w * 0.21 + 0.004), y, z - 0.001), shutters,
                  bevel=0.0015, bevel_segments=1)
            for k in range(3):
                F.box(f'{name}_louvre{s}{k}', w * 0.32, 0.004, 0.003, (x + s * (w / 2 + w * 0.21 + 0.004), y - h / 3 + h / 3 * k,
                                                                        z + 0.002), shutters)


def dollHouse():
    z = -0.16
    F.box('base', 0.8, 0.04, 0.42, (0, 0.02, -0.15), M.house_floor, bevel=0.008, bevel_segments=2)
    F.box('back', 0.76, 0.66, 0.02, (0, 0.37, -0.34), M.house, bevel=0.004, bevel_segments=1)
    F.box('paper_down', 0.72, 0.31, 0.004, (0, 0.205, -0.328), M.wallpaper)
    F.box('paper_up', 0.72, 0.30, 0.004, (0, 0.53, -0.328), M.wallpaper_up)
    for s in (-1, 1):
        F.box(f'side{s}', 0.02, 0.66, 0.38, (s * 0.37, 0.37, z), M.house, bevel=0.004, bevel_segments=1)
        # corner boards up the front edges of the sides
        F.box(f'corner_board{s}', 0.024, 0.66, 0.016, (s * 0.37, 0.37, 0.03), M.house_trim, bevel=0.003, bevel_segments=1)
    F.box('floor_down', 0.72, 0.006, 0.36, (0, 0.043, z), M.house_floor)
    F.box('floor_up', 0.72, 0.02, 0.37, (0, 0.37, z), M.house_floor, bevel=0.003, bevel_segments=1)
    F.box('ceiling', 0.76, 0.02, 0.38, (0, 0.69, z), M.house, bevel=0.003, bevel_segments=1)
    # the gables, a round window in the front one, and the roof: two slabs with courses of shingles, a ridge, a
    # chimney
    gable = [(-0.38, 0.0), (0.38, 0.0), (0.0, ROOF_RISE)]
    for k, zg in enumerate((0.03, -0.35)):
        _slab(f'gable{k}', gable, _plane((0, 0.7, zg), (1, 0, 0), (0, 1, 0)), M.house, thick=0.02)
    G.torus('gable_window', 0.042, 0.008, at=F.at(0, 0.77, 0.052), material=M.house_trim, rot=(90, 0, 0),
            major_segments=16, minor_segments=4)
    F.cyl('gable_glass', 0.036, 0.008, (0, 0.77, 0.05), M.house_window, rot=(90, 0, 0), verts=16)
    a = math.degrees(math.atan2(ROOF_RISE, 0.38))
    ca, sa = math.cos(math.radians(a)), math.sin(math.radians(a))
    run = 0.23 * ca
    for s in (-1, 1):
        y = 0.7 + ROOF_RISE - run * math.tan(math.radians(a)) + 0.015 / ca
        F.box(f'roof{s}', 0.46, 0.03, 0.44, (s * run, y, z), M.roof, rot=(0, -s * a, 0), bevel=0.006, bevel_segments=1)
        # four courses of shingles, each a strip lying on the roof, its lower edge proud of the one under it
        for k in range(4):
            d = 0.075 + 0.1 * k                # down the slope from the ridge, the strip's middle
            F.box(f'shingles{s}{k}', 0.11, 0.008, 0.43, (s * (d * ca + 0.034 * sa), 0.7 + ROOF_RISE - d * sa + 0.034 * ca, z),
                  M.roof, rot=(0, -s * (a + 7), 0), bevel=0.003, bevel_segments=1)
    F.cyl('ridge', 0.012, 0.44, (0, 0.894, z), M.house_trim, rot=(90, 0, 0), verts=8)
    F.box('chimney', 0.07, 0.12, 0.08, (0.2, 0.82, -0.25), M.roof, bevel=0.004, bevel_segments=1)
    F.box('chimney_cap', 0.085, 0.015, 0.095, (0.2, 0.8875, -0.25), M.house_trim, bevel=0.003, bevel_segments=1)
    # the rooms: downstairs a sofa, a table with a lamp, a rug and the stairs; upstairs a bed, a cupboard and a picture
    F.box('rug', 0.26, 0.004, 0.16, (-0.05, 0.048, -0.12), M.wallpaper_up, bevel=0.0015, bevel_segments=1)
    F.soft('sofa', 0.2, 0.05, 0.08, (-0.17, 0.073, -0.28), M.toy_blue, round=0.012, round_segments=1, flat=1)
    F.soft('sofa_back', 0.2, 0.07, 0.03, (-0.17, 0.125, -0.31), M.toy_blue, round=0.012, round_segments=1, flat=1)
    for s in (-1, 1):
        F.box(f'sofa_arm{s}', 0.025, 0.065, 0.09, (-0.17 + s * 0.1, 0.09, -0.285), M.toy_blue, bevel=0.008, bevel_segments=1)
    F.cyl('table', 0.045, 0.01, (0.06, 0.1, -0.2), M.house_door, verts=14)
    F.cyl('table_leg', 0.006, 0.055, (0.06, 0.0735, -0.2), M.house_door, verts=6)
    F.cone('lamp', 0.018, 0.01, 0.022, (0.06, 0.122, -0.2), M.wallpaper, verts=10)
    for k in range(6):   # the stairs up the right wall
        F.box(f'stair{k}', 0.1, 0.05 * (k + 1), 0.045, (0.3, 0.025 * (k + 1) + 0.046, -0.06 - 0.045 * k), M.house_floor)
    F.soft('bed', 0.2, 0.045, 0.12, (-0.18, 0.405, -0.26), M.toy_pink, round=0.012, round_segments=1, flat=1)
    F.box('bed_head', 0.012, 0.07, 0.13, (-0.285, 0.415, -0.26), M.house_door, bevel=0.003, bevel_segments=1)
    F.box('bed_pillow', 0.05, 0.02, 0.08, (-0.25, 0.437, -0.26), M.house_trim, bevel=0.008, bevel_segments=1)
    F.box('cupboard', 0.1, 0.14, 0.06, (0.2, 0.45, -0.3), M.toy_yellow, bevel=0.006, bevel_segments=1)
    F.box('cupboard_split', 0.003, 0.12, 0.062, (0.2, 0.45, -0.3), M.house_door)
    F.box('picture_frame', 0.11, 0.09, 0.005, (0.0, 0.56, -0.324), M.house_door, bevel=0.002, bevel_segments=1)
    F.box('picture', 0.09, 0.07, 0.006, (0.0, 0.56, -0.324), M.toy_green)
    _still('body')
    # the front wall, hinged on its left edge: a panelled door under a little porch roof, windows with shutters,
    # a flower box
    with G.new_parts() as parts:
        F.box('front', 0.76, 0.66, 0.02, (0, 0.37, 0.04), M.house, bevel=0.004, bevel_segments=1)
        F.box('skirt', 0.76, 0.04, 0.008, (0, 0.06, 0.052), M.house_trim, bevel=0.002, bevel_segments=1)
        F.box('band', 0.76, 0.014, 0.008, (0, 0.37, 0.052), M.house_trim, bevel=0.002, bevel_segments=1)
        G.kit.shaker(F, 'door', 0.11, 0.19, (-0.2, 0.135, 0.054), 0.008, M.house_door, rail=0.02, recess=0.002)
        F.extrude('door_frame', [(0.072, 0.0), (0.072, 0.21), (-0.072, 0.21), (-0.072, 0.0), (-0.058, 0.0), (-0.058, 0.196),
                                 (0.058, 0.196), (0.058, 0.0)], 0.006, (-0.2, 0.04, 0.054), M.house_trim)
        G.sphere('door_knob', 0.007, at=F.at(-0.165, 0.135, 0.062), material=M.toy_yellow, segments=6, rings=4)
        F.extrude('porch', [(-0.09, 0.0), (0.09, 0.0), (0.0, 0.045)], 0.02, (-0.2, 0.255, 0.064), M.roof, bevel=0.003,
                  bevel_segments=1)
        for k, (x, y) in enumerate(((0.18, 0.2), (-0.18, 0.54), (0.18, 0.54))):
            _window(f'window{k}', x, y, 0.054, 0.12, 0.13, M.house_trim, M.house_window, shutters=M.wallpaper_up)
        F.box('flower_box', 0.14, 0.03, 0.022, (0.18, 0.11, 0.061), M.house_door, bevel=0.004, bevel_segments=1)
        for k in range(5):
            F.sphere(f'leaf{k}', 0.012, (0.13 + 0.025 * k, 0.128, 0.063), M.grass, segments=5, rings=3)
            F.sphere(f'flower{k}', 0.009, (0.13 + 0.025 * k, 0.136, 0.066), (M.toy_pink, M.toy_yellow)[k % 2],
                     segments=5, rings=3)
        F.box('step', 0.16, 0.02, 0.012, (-0.2, 0.035, 0.057), M.house_trim, bevel=0.003, bevel_segments=1)
    front = G.hinged(G.join(parts, 'front_wall'), F.at(-0.38, 0.37, 0.04))
    _place(forward=-0.05)
    G.swing(front, 'front_open', 120, seconds=1.0)
    return G.meshes()


# --- KD2: the rocking horse ----------------------------------------------------------------------------------
ROCK_R = 1.0      # the rockers' radius: the horse rocks about their centre, 1 m up


def _rocker_y(z):
    return ROCK_R - math.sqrt(ROCK_R * ROCK_R - z * z) + 0.022


def _hoop(zc, yc, rx, ry, n=12, square=2.0):
    """A loop round the depth axis at game depth `zc`: an ellipse `rx` across and `ry` up round height `yc`, as
    Blender points, for a body lofted front to back."""
    return [F.at(p[0], yc + p[1], zc) for p in G.ring(rx, ry, 0, n, square=square)]


def rockingHorse():
    # the rockers: two bows curling up at their ends, a stretcher between them at each end
    zs = [0.35 * (-1 + 2 * i / 14) for i in range(15)]
    for x in (-0.125, 0.125):
        e = _rocker_y(0.35)
        pts = [(x, e + 0.04, -0.371), (x, e + 0.016, -0.364)] + [(x, _rocker_y(z), z) for z in zs] \
            + [(x, e + 0.016, 0.364), (x, e + 0.04, 0.371)]
        F.sweep(f'rocker{x}', pts, None, M.rocker, profile=G.rounded_rect(0.045, 0.039, 0.01, corner=1), up=(0, 0, 1))
    for z in (-0.3, 0.3):
        F.sweep(f'stretcher{z}', [(-0.125, _rocker_y(z) + 0.01, z), (0.125, _rocker_y(z) + 0.01, z)], 0.016, M.rocker, verts=8)
    # the legs, tapering to hooves on the rockers
    for sx in (-1, 1):
        for sz in (-1, 1):
            foot = (sx * 0.125, _rocker_y(sz * 0.22) + 0.045, sz * 0.22)
            F.sweep(f'leg{sx}{sz}', [(sx * 0.055, 0.44, sz * 0.14), (sx * 0.085, 0.27, sz * 0.19), foot], 0.03, M.horse,
                    verts=10, resolution=2, scale=[1.0, 0.72, 0.68])
            F.cyl(f'hoof{sx}{sz}', 0.019, 0.032, (foot[0], foot[1] - 0.012, foot[2]), M.dark, verts=8)
    # the body: a barrel lofted from rump to chest
    rings = [(-0.275, 0.475, 0.004, 0.004), (-0.268, 0.474, 0.05, 0.06), (-0.24, 0.47, 0.082, 0.092),
             (-0.17, 0.46, 0.094, 0.1), (-0.05, 0.452, 0.09, 0.098), (0.08, 0.458, 0.09, 0.1), (0.17, 0.475, 0.08, 0.094),
             (0.225, 0.495, 0.055, 0.066), (0.242, 0.505, 0.004, 0.004)]
    G.loft('body', [_hoop(*r, n=14) for r in rings], material=M.horse)
    for k, (x, y, z) in enumerate(((0.088, 0.48, -0.12), (-0.089, 0.44, 0.06), (0.085, 0.42, 0.11), (-0.083, 0.49, -0.16),
                                   (0.075, 0.38, -0.02), (-0.07, 0.39, -0.08))):
        F.sphere(f'spot{k}', 0.028, (x, y, z), M.spot, scale=(0.3, 1, 0.8), segments=8, rings=4)
    # the neck and the head, swept and tapering, a muzzle on its end
    F.sweep('neck', [(0, 0.47, 0.16), (0, 0.6, 0.235), (0, 0.72, 0.29)], None, M.horse,
            profile=[(0.062 * math.cos(math.tau * i / 12), 0.048 * math.sin(math.tau * i / 12)) for i in range(12)],
            up=(0, 0, 1), resolution=3, scale=[1.0, 0.85, 0.78])
    F.sweep('head', [(0, 0.76, 0.28), (0, 0.735, 0.36), (0, 0.69, 0.43)], None, M.horse,
            profile=[(0.05 * math.cos(math.tau * i / 12), 0.04 * math.sin(math.tau * i / 12)) for i in range(12)],
            up=(0, 0, 1), resolution=3, scale=[1.0, 0.85, 0.72])
    F.sphere('muzzle', 0.034, (0, 0.686, 0.443), M.horse, scale=(1.0, 1.05, 1.0), segments=12, rings=8)
    F.sphere('poll', 0.045, (0, 0.765, 0.29), M.horse, segments=12, rings=8)
    for s in (-1, 1):
        F.sphere(f'eye{s}', 0.011, (s * 0.034, 0.755, 0.35), M.eye, segments=6, rings=4)
        F.sphere(f'nostril{s}', 0.006, (s * 0.016, 0.69, 0.474), M.eye, segments=6, rings=4)
        F.cone(f'ear{s}', 0.018, 0.003, 0.05, (s * 0.028, 0.8, 0.3), M.horse, verts=8, rot=(-12, s * 10, 0))
    # the bridle round the muzzle, reins back to the saddle
    F.sweep('bridle', [(0.04 * math.cos(math.tau * i / 12), 0.69 + 0.038 * math.sin(math.tau * i / 12), 0.42)
                       for i in range(12)], 0.005, M.saddle, verts=5, closed=True)
    for s in (-1, 1):
        F.sweep(f'rein{s}', [(s * 0.04, 0.69, 0.42), (s * 0.07, 0.6, 0.3), (s * 0.08, 0.565, 0.13)], 0.004, M.saddle,
                verts=4, resolution=3)
    # the mane down the crest and a forelock, the tail
    mane = F.sweep('mane', [(0, 0.52, 0.1), (0, 0.635, 0.18), (0, 0.755, 0.245), (0, 0.8, 0.295), (0, 0.79, 0.34)], None,
                   M.mane, profile=G.rounded_rect(0.05, 0.026, 0.01, corner=1), up=(0, 0, 1), resolution=2,
                   scale=[0.7, 1.0, 1.0, 0.8, 0.5])
    G.jitter(mane, 0.006, size=0.04, seed=4.0)
    tail = F.sweep('tail', [(0, 0.5, -0.265), (0, 0.49, -0.31), (0, 0.42, -0.345), (0, 0.33, -0.35)], 0.024, M.mane,
                   verts=8, resolution=2, scale=[0.8, 1.15, 1.0, 0.4])
    G.jitter(tail, 0.005, size=0.04, seed=5.0)
    # the saddle on a cloth, its flaps, stirrups on leathers; the seat where the game seats the rider (0.56)
    F.soft('saddle', 0.2, 0.024, 0.21, (0, 0.545, -0.02), M.saddle, round=0.01, puff=0.004, flat=2, round_segments=1)
    F.soft('pommel', 0.08, 0.03, 0.04, (0, 0.56, 0.075), M.saddle, round=0.012, round_segments=1, flat=1)
    F.soft('cantle', 0.12, 0.03, 0.035, (0, 0.562, -0.115), M.saddle, round=0.012, round_segments=1, flat=1)
    for s in (-1, 1):
        F.box(f'saddle_flap{s}', 0.012, 0.1, 0.15, (s * 0.098, 0.49, -0.02), M.saddle, rot=(0, s * 8, 0), bevel=0.005,
              bevel_segments=1)
        F.sweep(f'leather{s}', [(s * 0.104, 0.52, 0.0), (s * 0.107, 0.36, 0.0)], None, M.saddle,
                profile=G.rounded_rect(0.003, 0.016, 0.001, corner=1), up=(0, 0, 1))
        G.torus(f'stirrup{s}', 0.018, 0.004, at=F.at(s * 0.107, 0.345, 0.0), material=M.handle, rot=(0, 90, 0),
                major_segments=10, minor_segments=4)
    # the handle through the head, a turned wooden grip on each end
    F.sweep('handle', [(-0.085, 0.68, 0.28), (0.085, 0.68, 0.28)], 0.008, M.handle, verts=8)
    for s in (-1, 1):
        G.revolve(f'grip{s}', [(0.0, -0.028), (0.011, -0.028), (0.016, -0.016), (0.017, 0.0), (0.015, 0.018),
                               (0.009, 0.028), (0.0, 0.028)], at=F.at(s * 0.1, 0.68, 0.28), material=M.rocker,
                  segments=8, rot=(0, 90, 0))
    # the whole horse rocks, its origin on the rockers' centre; the rider rides it
    horse = G.join(G.meshes(), 'horse', origin=F.at(0, ROCK_R, 0))
    _place()
    G.rock(horse, 'rock', 12, seconds=1.2, axis='X')
    return G.meshes()


# --- KD2: the play tent --------------------------------------------------------------------------------------
TENT_R, TENT_H = 0.46, 1.3
CANVAS_TOP = 0.86   # how far up the canvas runs (of the height); above it the poles cross bare, lashed together
FRONT_SIDE = 1      # the side between corners 1 and 2 faces the front (+depth)


def _tent_face(k):
    """The frame on face `k` of the six (0 the front), from the middle of its base edge up to the apex."""
    th = math.radians(90 + 60 * k)
    a = TENT_R * math.cos(math.radians(30))
    base = (a * math.cos(th), 0.0, a * math.sin(th))
    up = (-base[0], TENT_H, -base[2])
    length = math.sqrt(sum(c * c for c in up))
    return _plane(base, (math.sin(th), 0, -math.cos(th)), _mul(up, 1 / length)), length


DOOR = [(-0.16, 0.0), (0.16, 0.0), (0.13, 0.35), (0.05, 0.56), (-0.05, 0.56), (-0.13, 0.35)]


def _canvas_point(i, u, t, out=0.0):
    """A point on the canvas: on side `i` (corner i to i + 1), `u` along it, `t` of the way up; the cloth between
    the poles sags in a little (not the front, taut round its door), `out` metres off it."""
    a0, a1 = math.radians(60 * i), math.radians(60 * (i + 1))
    x = TENT_R * ((1 - u) * math.cos(a0) + u * math.cos(a1))
    z = TENT_R * ((1 - u) * math.sin(a0) + u * math.sin(a1))
    sag = 0.0 if i == FRONT_SIDE else 0.035 * math.sin(math.pi * u) * (0.4 + 0.6 * math.sin(math.pi * min(t / 0.7, 1.0)))
    k = (1 - t) * (1 - sag) + out / TENT_R
    return (x * k, TENT_H * t, z * k)


def _canvas_ring(t, out=0.0, n=4):
    return [F.at(*_canvas_point(i, j / n, t, out)) for i in range(6) for j in range(n)]


def playTent():
    G.loft('canvas', [_canvas_ring(t) for t in (0.0, 0.12, 0.26, 0.4, 0.55, 0.7, CANVAS_TOP)], material=M.canvas)
    # a hem round its foot
    G.loft('hem', [_canvas_ring(t, 0.004) for t in (0.0, 0.045)], material=M.flap, caps=False, smooth=False)
    apex = (0.0, TENT_H, 0.0)
    corners = [(TENT_R * math.cos(math.radians(60 * i)), 0.0, TENT_R * math.sin(math.radians(60 * i))) for i in range(6)]
    for i, c in enumerate(corners):
        d = _add(apex, _mul(c, -1))
        length = math.sqrt(sum(v * v for v in d))
        foot = _add(c, _mul(d, -0.012 / length))
        top = _add(apex, _mul(d, 0.28 / length))
        F.sweep(f'pole{i}', [_add(foot, (0, 0.01, 0)), top], 0.0125, M.poles, verts=8, scale=[1.0, 0.8])
    # the rope lashing the poles where they cross
    for k, y in enumerate((1.135, 1.16, 1.185)):
        r = TENT_R * (1 - y / TENT_H) + 0.014
        G.torus(f'lashing{k}', r, 0.005, at=F.at(0, y, 0), material=M.canvas, major_segments=12, minor_segments=4)
    # a round padded mat on the floor, two cushions on it
    _turned('floor_mat', 0, 0, [(0.0, 0.0), (0.345, 0.0), (0.362, 0.008), (0.365, 0.02), (0.35, 0.03), (0.0, 0.031)],
            M.tent_mat, segments=24)
    F.soft('cushion0', 0.3, 0.12, 0.09, (-0.11, 0.09, -0.2), M.flap, rot=(-12, 0, 18), round=0.04, puff=0.025, flat=2)
    F.soft('cushion1', 0.26, 0.11, 0.08, (0.14, 0.085, -0.18), M.tent_mat, rot=(-10, 0, -22), round=0.035, puff=0.022,
           flat=2)
    front, _ = _tent_face(0)
    _slab('door', DOOR, front, M.tent_door, out=0.002, thick=0.003)
    roll_at = _add(front[0], _mul(front[2], 0.575), _mul(front[3], 0.02))
    F.cyl('roll', 0.022, 0.2, (0, 0.0, 0), M.flap, verts=10, rot=(0, 90, 0)).location = F.at(*roll_at)
    # the ties the flap is rolled up with, hanging from the roll
    for s in (-1, 1):
        p = _add(roll_at, _mul(front[1], s * 0.06))
        F.sweep(f'tie{s}', [_add(p, _mul(front[3], 0.022)), _add(p, _mul(front[2], -0.06), _mul(front[3], 0.026)),
                            _add(p, _mul(front[2], -0.12), _mul(front[1], s * 0.01), _mul(front[3], 0.024))], None, M.canvas,
                profile=G.rounded_rect(0.003, 0.016, 0.001, corner=1), up=(0, 0, 1), resolution=2)
    # bunting on a string round the front three faces
    flags = (M.toy_red, M.toy_yellow, M.toy_blue, M.toy_green, M.toy_red)
    tb = 0.7 / _tent_face(0)[1]
    F.sweep('bunting', [_canvas_point(i, j / 4, tb, 0.007) for i in (0, 1, 2) for j in range(4)] + [_canvas_point(3, 0, tb, 0.007)],
            0.002, M.canvas, verts=4)
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


PADS = 12   # the foam wall's padded channels round the pit


def _pad_ring(r, y, puff, n=48):
    """A loop round the pit at height `y`: radius `r`, padded out by `puff` in each channel, creased between."""
    return [F.at((r + puff * abs(math.sin(PADS / 2 * math.tau * i / n))) * math.cos(math.tau * i / n), y,
                 (r + puff * abs(math.sin(PADS / 2 * math.tau * i / n))) * math.sin(math.tau * i / n)) for i in range(n)]


def ballPit():
    balls = _balls()
    # the pit: a foam wall in padded channels on a floor, a puffed roll round the top, the balls' level inside
    G.loft('pit', [_pad_ring(0.445, 0.0, 0.0), _pad_ring(0.452, 0.012, 0.008), _pad_ring(0.452, 0.33, 0.008),
                   _pad_ring(0.42, 0.345, 0.0)], material=M.pit)
    n = 48
    F.sweep('rim', [(0.43 * math.cos(math.tau * i / n), 0.36, 0.43 * math.sin(math.tau * i / n)) for i in range(n)], 0.035,
            M.pit_rim, verts=8, closed=True, scale=lambda w: 0.86 + 0.14 * abs(math.sin(PADS * math.pi * w)))
    for k in range(6):
        a = math.radians(60 * k + 15)
        F.cyl(f'dot{k}', 0.045, 0.004, (0.461 * math.cos(a), 0.17, 0.461 * math.sin(a)), balls[k % 5],
              rot=(90, 0, 60 * k + 15 - 90), verts=10)
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
    _place(forward=0.035)   # its back on the wall's face, its front on the tile's edge
    G.squash(balls, 'bounce', (1, 1, 1.18), seconds=0.7, back=True)
    return G.meshes()


# --- KD2: the train table ------------------------------------------------------------------------------------
RING_X, RING_Z, RING_R = 0.26, 0.02, 0.28
TRAIN_W = 1.24    # the table's length: a train table is 0.9-1.3 m (review M-kids-02: it was 1.8)
TABLE_TOP = 0.44


def _train_piece(name, a, build):
    """A piece of the train made by `build(name, y)` along its own depth axis at the middle (x across, its front to
    -depth, standing on the track's top `y`), then turned onto the ring at `a` degrees round it."""
    with G.new_parts() as parts:
        build(name, TABLE_TOP + 0.012)
    obj = G.join(parts, name, origin=F.at(0, TABLE_TOP, 0))
    ar = math.radians(a)
    obj.location = F.at(RING_X + RING_R * math.cos(ar), TABLE_TOP, RING_Z + RING_R * math.sin(ar))
    obj.rotation_euler[2] = ar
    return obj


def _train_wheels(name, y, zs):
    for k, z in enumerate(zs):
        for s in (-1, 1):
            F.cyl(f'{name}_wheel{k}{s}', 0.013, 0.007, (s * 0.027, y + 0.013, z), M.dark, rot=(0, 90, 0), verts=10)


def _engine(name, y):
    """A wooden toy engine: a boiler with a chimney and a dome, a cab with a roof, three wheels a side."""
    F.box(f'{name}_chassis', 0.05, 0.012, 0.12, (0, y + 0.016, 0), M.train, bevel=0.003, bevel_segments=1)
    F.cyl(f'{name}_boiler', 0.022, 0.075, (0, y + 0.042, -0.02), M.train, rot=(90, 0, 0), verts=12)
    F.cyl(f'{name}_smokebox', 0.0235, 0.01, (0, y + 0.042, -0.058), M.dark, rot=(90, 0, 0), verts=12)
    F.cone(f'{name}_chimney', 0.009, 0.013, 0.03, (0, y + 0.074, -0.042), M.dark, verts=10)
    F.sphere(f'{name}_dome', 0.011, (0, y + 0.064, -0.012), M.toy_yellow, segments=8, rings=5)
    F.box(f'{name}_cab', 0.054, 0.052, 0.042, (0, y + 0.048, 0.037), M.train, bevel=0.004, bevel_segments=1)
    F.box(f'{name}_roof', 0.064, 0.008, 0.054, (0, y + 0.078, 0.037), M.dark, bevel=0.003, bevel_segments=1)
    for s in (-1, 1):
        F.box(f'{name}_window{s}', 0.004, 0.018, 0.022, (s * 0.027, y + 0.058, 0.037), M.water)
    F.box(f'{name}_buffer', 0.044, 0.01, 0.008, (0, y + 0.018, -0.063), M.dark, bevel=0.002, bevel_segments=1)
    _train_wheels(name, y, (-0.038, 0.0, 0.038))


def _wagon(colour, cargo):
    def build(name, y):
        F.box(f'{name}_chassis', 0.05, 0.012, 0.09, (0, y + 0.016, 0), M.toy_yellow, bevel=0.003, bevel_segments=1)
        _walls(f'{name}_tub', 0.054, 0.08, y + 0.022, y + 0.052, 0, 0.006, colour, r=0.008)
        F.box(f'{name}_tub_floor', 0.046, 0.006, 0.072, (0, y + 0.025, 0), colour)
        cargo(name, y)
        _train_wheels(name, y, (-0.028, 0.028))
    return build


def _tree(name, x, z, h, y=TABLE_TOP):
    """A toy tree: a trunk and two cones of leaves, one over the other."""
    F.cyl(f'{name}_trunk', 0.012, 0.04, (x, y + 0.02, z), M.trunk, verts=8)
    F.cone(f'{name}_low', 0.055, 0.012, h * 0.55, (x, y + 0.035 + h * 0.275, z), M.tree, verts=10)
    F.cone(f'{name}_high', 0.042, 0.0, h * 0.5, (x, y + 0.035 + h * 0.75, z), M.tree, verts=10)


def _archway(ro, ho, ri, hi, n=8):
    """An arch's outline (x, height) standing on the ground: `ro` out and `ho` up to where its top rounds over, an
    opening `ri` and `hi` likewise through it."""
    return ([(ro, 0.0), (ro, ho)] + _arc(ro, 0, 180, n, 0.0, ho)[1:] + [(-ri, 0.0)] + _arc(ri, 180, 0, n, 0.0, hi)
            + [(ri, 0.0)])


def trainTable():
    # the table: round tapered legs, an apron, the board with a raised lip round it, a shelf below with two bins
    for sx in (-1, 1):
        for sz in (-1, 1):
            F.sweep(f'leg{sx}{sz}', [(sx * (TRAIN_W / 2 - 0.08), 0.36, sz * 0.34), (sx * (TRAIN_W / 2 - 0.08), 0.0, sz * 0.34)],
                    0.03, M.train_table, verts=12, scale=[1.0, 0.78])
    F.box('frame', TRAIN_W - 0.02, 0.06, 0.82, (0, 0.38, 0), M.train_table, bevel=0.012, bevel_segments=2)
    F.box('grass', TRAIN_W - 0.04, 0.03, 0.8, (0, 0.425, 0), M.grass, bevel=0.006, bevel_segments=1)
    lip = G.rounded_rect(TRAIN_W - 0.02, 0.82, 0.03, corner=3)
    F.sweep('lip', [(u, 0.432, v) for u, v in lip], None, M.train_table, profile=G.rounded_rect(0.036, 0.02, 0.008, corner=1),
            up=(0, 0, 1), closed=True)
    F.box('shelf', TRAIN_W - 0.16, 0.02, 0.62, (0, 0.1, 0), M.train_table, bevel=0.005, bevel_segments=1)
    for k, (x, m) in enumerate(((-0.25, M.toy_red), (0.12, M.toy_blue))):
        _walls(f'bin{k}', 0.3, 0.26, 0.11, 0.22, 0.02, 0.008, m, r=0.02, x=x)
        F.box(f'bin_floor{k}', 0.29, 0.006, 0.25, (x, 0.113, 0.02), m)
    F.box('bin_block0', 0.05, 0.05, 0.05, (-0.3, 0.185, 0.05), M.toy_yellow, rot=(0, 0, 20), bevel=0.006, bevel_segments=1)
    F.sphere('bin_ball', 0.035, (0.08, 0.19, 0.0), M.ball_yellow, segments=10, rings=6)
    # the ring of wooden track, with two grooves
    G.revolve('track', [(RING_R - 0.04, 0), (RING_R + 0.04, 0), (RING_R + 0.04, 0.012), (RING_R - 0.04, 0.012),
                        (RING_R - 0.04, 0)], at=F.at(RING_X, TABLE_TOP - 0.002, RING_Z), material=M.track, segments=40,
              smooth=False)
    for r in (RING_R - 0.016, RING_R + 0.016):
        G.torus(f'groove{r}', r, 0.003, at=F.at(RING_X, TABLE_TOP + 0.0105, RING_Z), material=M.track_groove,
                major_segments=40, minor_segments=4)
    # a tunnel the track runs through on the right, grass over its top, a stone portal at each end
    tx, tz = RING_X + RING_R, RING_Z
    F.extrude('tunnel', _archway(0.072, 0.09, 0.045, 0.075), 0.12, (tx, TABLE_TOP, tz), M.grass, bevel=0.008,
              bevel_segments=1)
    for s in (-1, 1):
        F.extrude(f'portal{s}', _archway(0.06, 0.08, 0.045, 0.075, n=6), 0.01, (tx, TABLE_TOP, tz + s * 0.062), M.spot,
                  bevel=0.002, bevel_segments=1)
    # the station on the left of the track: a platform, a building with a pitched roof, a signal
    F.box('platform', 0.05, 0.02, 0.26, (-0.09, TABLE_TOP + 0.01, RING_Z), M.spot, bevel=0.004, bevel_segments=1)
    F.box('station', 0.1, 0.09, 0.2, (-0.16, TABLE_TOP + 0.045, RING_Z), M.toy_yellow, bevel=0.005, bevel_segments=1)
    F.extrude('station_roof', [(-0.065, 0.0), (0.065, 0.0), (0.0, 0.05)], 0.22, (-0.16, TABLE_TOP + 0.09, RING_Z), M.toy_red,
              bevel=0.004, bevel_segments=1)
    F.box('station_door', 0.004, 0.06, 0.04, (-0.108, TABLE_TOP + 0.03, RING_Z), M.house_door)
    for s in (-1, 1):
        F.box(f'station_window{s}', 0.004, 0.03, 0.035, (-0.108, TABLE_TOP + 0.055, RING_Z + s * 0.06), M.water)
    F.cyl('signal_post', 0.005, 0.14, (-0.09, TABLE_TOP + 0.07, RING_Z + 0.17), M.dark, verts=6)
    F.box('signal', 0.016, 0.04, 0.014, (-0.09, TABLE_TOP + 0.13, RING_Z + 0.17), M.dark, bevel=0.003, bevel_segments=1)
    F.sphere('signal_light', 0.006, (-0.09, TABLE_TOP + 0.14, RING_Z + 0.178), M.toy_red, segments=6, rings=4)
    # the town: a road with its line, a house, a pond with stones round it, trees
    F.box('road', 0.08, 0.004, 0.8, (-0.3, TABLE_TOP + 0.002, 0), M.track_groove)
    for k in range(5):
        F.box(f'road_line{k}', 0.008, 0.002, 0.07, (-0.3, TABLE_TOP + 0.005, -0.32 + 0.16 * k), M.paper)
    F.box('house', 0.11, 0.08, 0.1, (-0.5, TABLE_TOP + 0.04, -0.25), M.house, bevel=0.004, bevel_segments=1)
    F.extrude('house_roof', [(-0.065, 0.0), (0.065, 0.0), (0.0, 0.05)], 0.12, (-0.5, TABLE_TOP + 0.08, -0.25), M.roof,
              bevel=0.003, bevel_segments=1)
    F.box('house_door', 0.03, 0.045, 0.004, (-0.5, TABLE_TOP + 0.0225, -0.199), M.house_door)
    F.cyl('pond', 0.09, 0.006, (-0.49, TABLE_TOP + 0.003, 0.2), M.water, verts=20)
    for k in range(7):
        a = math.tau * k / 7
        F.sphere(f'stone{k}', 0.014, (-0.49 + 0.103 * math.cos(a), TABLE_TOP + 0.004, 0.2 + 0.103 * math.sin(a)), M.spot,
                 scale=(1.2, 1, 0.6), segments=6, rings=4)
    for k, (x, z, h) in enumerate(((-0.55, -0.02, 0.2), (-0.17, -0.3, 0.16), (-0.17, 0.3, 0.18), (0.55, -0.33, 0.16),
                                   (0.56, 0.33, 0.14))):
        _tree(f'tree{k}', x, z, h)
    _still('body')
    # the train: an engine and two wagons, turning round the ring's middle
    with G.new_parts() as parts:
        _train_piece('engine', 90, _engine)
        _train_piece('wagon0', 90 + 28, _wagon(M.toy_blue, lambda n, y: F.sphere(f'{n}_ball', 0.022, (0, y + 0.065, 0),
                                                                                    M.ball_yellow, segments=10, rings=6)))
        _train_piece('wagon1', 90 + 52, _wagon(M.toy_green, lambda n, y: F.cyl(f'{n}_log', 0.014, 0.08, (0, y + 0.06, 0),
                                                                                  M.trunk, rot=(90, 0, 0), verts=8)))
    train = G.join(parts, 'train', origin=F.at(RING_X, TABLE_TOP, RING_Z))
    _place()
    G.spin(train, 'train', turns=-1, seconds=6.0, axis='Z')
    return G.meshes()


# --- KD2: the play mat ---------------------------------------------------------------------------------------
MAT_TILE = 0.44   # each foam tile's side; four interlock in the middle of the mat


def _jigsaw(p0, p1, side):
    """An edge from p0 to p1 (game x, depth) with a jigsaw knob in its middle bulging to `side` (+1: left of the way
    along it, -1: right), as foam tiles lock."""
    (x0, z0), (x1, z1) = p0, p1
    L = math.hypot(x1 - x0, z1 - z0)
    ux, uz = (x1 - x0) / L, (z1 - z0) / L
    nx, nz = -uz * side, ux * side

    def at(t, o):
        return (x0 + ux * t + nx * o, z0 + uz * t + nz * o)
    m, neck, r, out = L / 2, 0.02, 0.032, 0.04
    pts = [at(0, 0), at(m - neck, 0)]
    for i in range(9):   # round the knob, from its near side over its far end to the other side
        a = math.radians(220 - 260 * i / 8)
        pts.append(at(m + r * math.cos(a), out + r * math.sin(a)))
    return pts + [at(m + neck, 0), at(L, 0)]


def playMat():
    # four foam tiles locking in the middle, each with a shape pressed out of it, on a backing in the mat's colour
    F.extrude('mat', G.rounded_rect(0.92, 0.92, 0.02, corner=2), 0.012, (0, 0.006, 0), M.mat, plane='top', bevel=0.003,
              bevel_segments=1)
    T = MAT_TILE
    ez = {sx: _jigsaw((0, 0), (sx * T, 0), sx) for sx in (-1, 1)}     # the edges along depth 0, out from the middle
    ex = {sz: _jigsaw((0, 0), (0, sz * T), -sz) for sz in (-1, 1)}    # along x 0
    tiles = {(-1, -1): M.mat_yellow, (1, -1): M.mat_pink, (-1, 1): M.mat_green, (1, 1): M.mat_white}
    marks = {(-1, -1): _star(0, 0, 0.09), (1, -1): _heart(0, 0, 0.075), (-1, 1): _moon(0, 0, 0.08),
             (1, 1): [(0.08 * math.cos(math.tau * i / 16), 0.08 * math.sin(math.tau * i / 16)) for i in range(16)]}
    mark_colours = {(-1, -1): M.mat_pink, (1, -1): M.mat_yellow, (-1, 1): M.mat_white, (1, 1): M.mat_green}
    for (sx, sz), m in tiles.items():
        outline = ez[sx] + [(sx * T, sz * T)] + list(reversed(ex[sz]))[:-1]
        F.extrude(f'tile{sx}{sz}', outline, 0.01, (0, 0.017, 0), m, plane='top')
        cx, cz = sx * T / 2, sz * T / 2
        F.extrude(f'mark{sx}{sz}', [(cx + u, cz - v) for u, v in marks[(sx, sz)]], 0.004, (0, 0.0225, 0),
                  mark_colours[(sx, sz)], plane='top')
    y = 0.022
    blocks = (((-0.3, -0.15), M.toy_red, M.toy_yellow, 10), ((-0.18, -0.24), M.toy_blue, M.paper, 35),
              ((0.26, 0.14), M.toy_green, M.toy_yellow, 50), ((-0.28, 0.24), M.toy_yellow, M.toy_red, 70))
    for k, ((x, z), m, face, r) in enumerate(blocks):
        _block(f'block{k}', x, y, z, 0.09, m, face, turn=r)
    _block('block_top', -0.29, y + 0.09, -0.16, 0.09, M.toy_green, M.paper, turn=25)
    G.sphere('ball', 0.06, at=F.at(0.3, y + 0.06, -0.25), material=M.ball_red, segments=14, rings=8)
    F.sphere('ball_band', 0.0605, (0.3, y + 0.06, -0.25), M.ball_yellow, scale=(1, 0.3, 1), rot=(0, 30, 0), segments=14,
             rings=4)
    _still('body')
    # the tower the resident builds: three blocks, grown up from the mat
    with G.new_parts() as parts:
        for k, (m, face, r) in enumerate(((M.toy_blue, M.toy_yellow, 0), (M.toy_yellow, M.toy_red, 18),
                                          (M.toy_red, M.paper, -12))):
            _block(f'tower{k}', 0.04, y + 0.09 * k, 0.12, 0.09, m, face, turn=r)
    tower = G.join(parts, 'tower', origin=F.at(0.04, 0.012, 0.12))   # squeezed into the mat until it grows
    tower.scale[2] = 0.001
    _place(forward=0.03)    # its back on the wall's face
    G.grow(tower, 'blocks', seconds=1.5)
    return G.meshes()


# --- KD2: the giant teddy bear and the kids' bean bag ---------------------------------------------------------
def _upright(name, rings, material, z=0.0, n=16):
    """A body lofted up through `rings` [(height, half width, half depth)], round the upright at game depth `z`."""
    return G.loft(name, [G.ring(rx, rz, y, n, at=(0.0, -z)) for y, rx, rz in rings], material=material)


def teddyBear():
    # the body: a pear sitting on the floor; the head: round, fuller at the cheeks
    _upright('body', [(0.02, 0.004, 0.004), (0.03, 0.14, 0.12), (0.08, 0.215, 0.18), (0.2, 0.248, 0.21), (0.32, 0.24, 0.2),
                      (0.42, 0.2, 0.17), (0.5, 0.14, 0.12), (0.54, 0.06, 0.05), (0.55, 0.004, 0.004)], M.fur, z=-0.02, n=16)
    F.sphere('belly', 0.15, (0, 0.27, 0.15), M.fur_light, scale=(0.95, 0.4, 0.95), segments=12, rings=7)
    _upright('head', [(0.5, 0.004, 0.004), (0.51, 0.09, 0.08), (0.55, 0.16, 0.14), (0.62, 0.2, 0.18), (0.69, 0.205, 0.185),
                      (0.76, 0.185, 0.165), (0.82, 0.14, 0.125), (0.86, 0.07, 0.06), (0.87, 0.004, 0.004)], M.fur, z=0.0, n=12)
    F.sphere('muzzle', 0.08, (0, 0.635, 0.155), M.fur_light, scale=(1, 0.75, 0.72), segments=12, rings=8)
    F.sphere('nose', 0.026, (0, 0.668, 0.21), M.eye, scale=(1.3, 0.8, 0.85), segments=10, rings=6)
    for s in (-1, 1):
        # the mouth stitched under the nose
        F.sweep(f'mouth{s}', [(0, 0.648, 0.214), (0, 0.628, 0.212), (s * 0.016, 0.62, 0.207), (s * 0.03, 0.628, 0.198)], 0.0025,
                M.eye, verts=4, resolution=2)
        F.sphere(f'eye{s}', 0.02, (s * 0.075, 0.73, 0.163), M.eye, segments=8, rings=5)
        F.sphere(f'ear{s}', 0.07, (s * 0.15, 0.85, -0.02), M.fur, scale=(1, 0.55, 1), segments=10, rings=6)
        F.sphere(f'ear_in{s}', 0.045, (s * 0.15, 0.85, 0.0), M.fur_light, scale=(1, 0.3, 1), segments=8, rings=5)
        # the arms reaching forward, a paw pad on each
        F.sweep(f'arm{s}', [(s * 0.17, 0.46, 0.02), (s * 0.235, 0.36, 0.06), (s * 0.255, 0.28, 0.12)], 0.068, M.fur, verts=10,
                resolution=2, scale=[0.95, 0.92, 0.85])
        F.sphere(f'paw{s}', 0.058, (s * 0.26, 0.27, 0.13), M.fur, segments=10, rings=7)
        F.sphere(f'paw_pad{s}', 0.036, (s * 0.245, 0.25, 0.17), M.fur_light, scale=(0.9, 0.4, 0.9), rot=(-30, 0, s * 20),
                 segments=8, rings=5)
        # the legs out in front, a pad on each foot
        F.sweep(f'leg{s}', [(s * 0.13, 0.1, 0.0), (s * 0.14, 0.088, 0.18), (s * 0.145, 0.086, 0.3)], 0.085, M.fur, verts=10,
                resolution=2, scale=[1.0, 0.95, 0.9])
        F.sphere(f'foot{s}', 0.077, (s * 0.145, 0.086, 0.3), M.fur, scale=(1, 0.45, 1), segments=10, rings=7)
        F.sphere(f'pad{s}', 0.06, (s * 0.145, 0.09, 0.33), M.fur_light, scale=(1, 0.25, 1.1), segments=10, rings=5)
        # the bow at the neck: two loops of ribbon and their tails
        loop = [(s * (0.012 + 0.07 * (1 - math.cos(a)) / 2), 0.505 + 0.032 * math.sin(a), 0.19 - 0.012 * (1 - math.cos(a)) / 2)
                for a in (math.tau * i / 10 for i in range(10))]
        F.sweep(f'bow{s}', loop, None, M.bow, profile=G.rounded_rect(0.006, 0.026, 0.002, corner=1), up=(0, 0, 1),
                closed=True)
        F.sweep(f'bow_tail{s}', [(s * 0.008, 0.5, 0.19), (s * 0.03, 0.46, 0.195), (s * 0.045, 0.42, 0.185)], None, M.bow,
                profile=G.rounded_rect(0.005, 0.022, 0.002, corner=1), up=(0, 0, 1), resolution=2)
    F.sphere('knot', 0.02, (0, 0.505, 0.193), M.bow, scale=(1, 0.8, 1.1), segments=10, rings=6)
    bear = G.join(G.meshes(), 'bear', origin=(0, 0, 0))
    _place()
    G.squash(bear, 'squish', (1.06, 1.06, 0.9), seconds=0.5)
    return G.meshes()


def beanBagKids():
    # a sack lofted up from the floor, leaning back into its back rest, a seat pressed into its front where the game
    # sits a child (0.24, furniture.json `seats`), creased all over; a handle on its back, a label on its front
    rings = [(0.0, 0.004, 0.004, 0.03), (0.004, 0.25, 0.26, 0.03), (0.05, 0.305, 0.315, 0.025), (0.13, 0.31, 0.315, 0.02),
             (0.2, 0.3, 0.3, 0.012), (0.26, 0.28, 0.27, -0.005), (0.32, 0.255, 0.22, -0.05), (0.38, 0.235, 0.18, -0.09),
             (0.44, 0.205, 0.145, -0.125), (0.49, 0.155, 0.11, -0.15), (0.515, 0.09, 0.07, -0.16), (0.528, 0.004, 0.004, -0.165)]
    bag = G.loft('bag', [G.ring(rx, rz, y, 24, at=(0.0, -z)) for y, rx, rz, z in rings], material=M.bag)
    for v in bag.data.vertices:
        gx, gz, gy = -v.co.x, -v.co.y, v.co.z
        d = math.hypot(gx / 0.24, (gz - 0.07) / 0.2)
        if gy > 0.12 and d < 1:
            v.co.z -= 0.17 * (1 - d * d) ** 2 * min(1.0, (gy - 0.12) / 0.12)
    bag.data.update()
    _rumple(bag, 0.014, 0.03, 7.0, size=0.08)   # up and down only: its bounds (the size and centre checks) stay put
    _slab('tag', _arc(0.045, 0, 330, 11), _board((0, 0.2, 0.315), 20), M.bag_patch, out=-0.004, thick=0.006)
    F.sweep('handle', [(-0.05, 0.47, -0.27), (-0.03, 0.495, -0.29), (0.03, 0.495, -0.29), (0.05, 0.47, -0.27)], None,
            M.bag_patch, profile=G.rounded_rect(0.006, 0.03, 0.002, corner=1), up=(0, 0, 1), resolution=2)
    bag = G.join(G.meshes(), 'bag', origin=(0, 0, 0))
    _place()
    G.squash(bag, 'squash', (1.04, 1.04, 0.85), seconds=0.5)
    return G.meshes()


# --- KD3: the kids' desk (and its chair), table and chair -----------------------------------------------------
def _back_knob(name, x, y, face, r, material, segments=8):
    """A turned knob on a front at game depth `face` that faces the back (-depth): kit.knob turned round."""
    prof = [(0, 0), (r * 0.75, 0), (r * 0.7, r * 0.15), (r * 0.36, r * 0.35), (r * 0.42, r * 0.75), (r * 0.95, r * 1.05),
            (r, r * 1.35), (r * 0.8, r * 1.7), (r * 0.4, r * 1.88), (0, r * 1.92)]
    return G.revolve(name, prof, at=F.at(x, y, face), material=material, segments=segments, rot=(-90, 0, 0))


def _crayon(name, a, b, material, r=0.005):
    """A crayon from `a` to its sharpened tip at `b` (game points), a paper wrapper round its middle."""
    k = 0.82
    mid = tuple(p + (q - p) * k for p, q in zip(a, b))
    F.sweep(name, [a, mid, b], r, material, verts=6, scale=[1.0, 1.0, 0.25])


def kidsDesk():
    # the desk: round legs tapering to the floor, side stretchers and a foot bar, a rounded wooden top with an
    # upstand along its far edge
    for x in (-0.34, 0.34):
        for z in (0.02, 0.38):
            F.sweep(f'leg{x}{z}', [(x, 0.545, z), (x, 0.0, z)], 0.021, M.kdesk, verts=10, scale=[1.0, 0.8])
        F.sweep(f'stretcher{x}', [(x, 0.12, 0.02), (x, 0.12, 0.38)], 0.009, M.kdesk, verts=8)
    F.sweep('foot_bar', [(-0.34, 0.1, 0.02), (0.34, 0.1, 0.02)], 0.011, M.kdesk, verts=8)
    F.extrude('top', G.rounded_rect(0.76, 0.44, 0.03, corner=3), 0.03, (0, 0.56, 0.2), M.kdesk_top, plane='top', bevel=0.006,
              bevel_segments=2)
    F.box('apron', 0.64, 0.06, 0.38, (0, 0.515, 0.2), M.kdesk, bevel=0.006, bevel_segments=1)
    F.box('upstand', 0.72, 0.045, 0.016, (0, 0.5975, 0.404), M.kdesk, bevel=0.006, bevel_segments=2)
    # on it: a sheet of paper with a drawing, a pot of crayons
    F.box('paper', 0.3, 0.004, 0.22, (-0.12, 0.577, 0.2), M.paper, rot=(0, 0, 6))
    F.box('drawing', 0.2, 0.002, 0.14, (-0.12, 0.5795, 0.2), M.chalk_blue, rot=(0, 0, 6))
    F.extrude('drawing_sun', [(0.022 * math.cos(math.tau * i / 10), 0.022 * math.sin(math.tau * i / 10)) for i in range(10)],
              0.002, (-0.06, 0.581, 0.15), M.paint_yellow, plane='top')
    F.extrude('drawing_hill', [(-0.1, 0.0), (0.1, 0.0)] + [(0.1 - 0.2 * i / 6, -0.035 * math.sin(math.pi * i / 6)) for i in range(7)],
              0.002, (-0.12, 0.581, 0.265), M.paint_green, plane='top', rot=(0, 0, 6))
    _turned('cup', 0.24, 0.3, [(0.0, 0.0), (0.03, 0.0), (0.036, 0.075), (0.033, 0.077), (0.028, 0.012), (0.0, 0.012)],
            M.toy_yellow, segments=14, y=0.575)
    for k, (dx, dz, m) in enumerate(((-0.012, -0.006, M.toy_red), (0.01, 0.004, M.toy_blue), (0.0, 0.012, M.toy_green))):
        _crayon(f'crayon{k}', (0.24 + dx, 0.59, 0.3 + dz), (0.24 + dx * 2.5, 0.69, 0.3 + dz * 2), m)
    # the chair, joined to the desk: its seat where the game sits a child (SEATS.kidsDesk), a curved back on two posts
    F.extrude('seat', G.rounded_rect(0.4, 0.38, 0.05, corner=3), 0.03, (0, 0.35, -0.19), M.kdesk_chair, plane='top',
              bevel=0.008, bevel_segments=2)
    F.box('seat_rail', 0.34, 0.04, 0.32, (0, 0.315, -0.2), M.kdesk_chair, bevel=0.006, bevel_segments=1)
    for s in (-1, 1):
        F.sweep(f'front_leg{s}', [(s * 0.17, 0.32, -0.04), (s * 0.18, 0.0, -0.025)], 0.0175, M.kdesk_chair, verts=8,
                scale=[1.0, 0.85])
        F.sweep(f'back_post{s}', [(s * 0.18, 0.0, -0.38), (s * 0.18, 0.66, -0.38)], 0.0175, M.kdesk_chair, verts=8)
        F.sphere(f'post_cap{s}', 0.019, (s * 0.18, 0.662, -0.38), M.kdesk_chair, segments=8, rings=5)
        F.sweep(f'rung{s}', [(s * 0.178, 0.12, -0.03), (s * 0.18, 0.12, -0.38)], 0.009, M.kdesk_chair, verts=6)
    for name, y, h in (('backrest', 0.61, 0.1), ('back_rail', 0.44, 0.04)):
        F.sweep(name, [(-0.19, y, -0.374), (0.0, y, -0.385), (0.19, y, -0.374)], None, M.kdesk_chair,
                profile=G.rounded_rect(h, 0.02, 0.008, corner=2), up=(0, 0, 1), resolution=3)
    _still('body')
    # the pencil drawer under the top, pulled out towards the child
    with G.new_parts() as parts:
        F.box('drawer_front', 0.4, 0.05, 0.015, (0, 0.512, 0.0025), M.kdesk, bevel=0.005, bevel_segments=2)
        F.box('drawer_tray', 0.38, 0.04, 0.3, (0, 0.512, 0.15), M.kdesk)
        _back_knob('drawer_knob', 0, 0.512, -0.005, 0.01, M.handle)
        for k, m in enumerate((M.toy_red, M.toy_blue, M.toy_green, M.toy_yellow)):
            _crayon(f'pencil{k}', (-0.12 + 0.07 * k, 0.536, 0.03), (-0.1 + 0.07 * k, 0.536, 0.2), m)
        F.box('eraser', 0.04, 0.012, 0.025, (0.15, 0.538, 0.12), M.chalk_pink, bevel=0.003, bevel_segments=1)
    drawer = G.join(parts, 'drawer')
    _place()
    G.slide(drawer, 'drawer_open', (0, 0.15, 0), seconds=0.6)
    return G.meshes()


def kidsTable():
    # a rounded top with a band in the frame's colour round its edge, a rounded apron, turned legs on bun feet
    F.extrude('top', G.rounded_rect(0.632, 0.632, 0.08, corner=4), 0.03, (0, 0.485, 0), M.ktable_top, plane='top',
              bevel=0.008, bevel_segments=2)
    band = G.rounded_rect(0.632, 0.632, 0.08, corner=4)
    F.sweep('band', [(u, 0.481, v) for u, v in band], None, M.ktable, profile=G.rounded_rect(0.016, 0.008, 0.003, corner=1),
            up=(0, 0, 1), closed=True)
    _walls('apron', 0.52, 0.52, 0.415, 0.47, 0, 0.02, M.ktable, r=0.05)
    prof = [(0.0, 0.0), (0.02, 0.0), (0.03, 0.012), (0.03, 0.026), (0.021, 0.042), (0.018, 0.1), (0.016, 0.26),
            (0.019, 0.34), (0.026, 0.37), (0.022, 0.39), (0.025, 0.47), (0.0, 0.47)]
    for sx in (-1, 1):
        for sz in (-1, 1):
            _turned(f'leg{sx}{sz}', sx * 0.235, sz * 0.235, prof, M.ktable, segments=10)
    G.join_all('kidsTable')
    _place()
    return G.meshes()


def kidsChair():
    # a rounded seat, legs splayed a little, the back legs running up into posts behind a back with a heart cut
    # through it, rungs along the sides and across the front
    F.extrude('seat', G.rounded_rect(0.32, 0.3, 0.05, corner=3), 0.03, (0, 0.29, 0.05), M.kchair_seat, plane='top',
              bevel=0.008, bevel_segments=2)
    for sx in (-1, 1):
        F.sweep(f'leg{sx}', [(sx * 0.13, 0.28, 0.165), (sx * 0.135, 0.0, 0.172)], 0.0175, M.kchair, verts=8, scale=[1.0, 0.85])
        F.sweep(f'post{sx}', G.kit.fillet([(sx * 0.135, 0.0, -0.075), (sx * 0.13, 0.28, -0.07), (sx * 0.13, 0.5, -0.1)], 0.06),
                0.016, M.kchair, verts=8)
        F.sweep(f'rung{sx}', [(sx * 0.133, 0.1, -0.072), (sx * 0.133, 0.1, 0.168)], 0.009, M.kchair, verts=6)
    F.sweep('rung_front', [(-0.133, 0.14, 0.168), (0.133, 0.14, 0.168)], 0.008, M.kchair, verts=6)
    F.extrude('back', G.rounded_rect(0.32, 0.18, 0.06, corner=3), 0.028, (0, 0.48, -0.1), M.kchair, rot=(-6, 0, 0),
              holes=[_heart(0, 0.005, 0.04)], bevel=0.008, bevel_segments=2)
    G.join_all('kidsChair')
    _place()
    return G.meshes()


# --- KD3: the picture-book shelf -----------------------------------------------------------------------------
SHELVES = ((0.12, -0.1), (0.40, -0.17), (0.68, -0.24))    # (height, front lip's depth)
BOOK_TILT = 14
BOOK_H = 0.115    # half a book's height: the top row's tops clear the top board


def _book(name, x, y, z, cover, picture):
    """A picture book stood face out, leaning back; its middle at (x, y, z): two boards round a block of pages that
    shows white along its top, a picture and a title band on its front."""
    frame = _board((x, y, z), BOOK_TILT)
    yield _slab(name, _rect(-0.09, -BOOK_H, 0.09, BOOK_H), frame, cover, out=0.006, thick=0.004)
    yield _slab(f'{name}_back', _rect(-0.09, -BOOK_H, 0.09, BOOK_H), frame, cover, out=-0.01, thick=0.004)
    yield _slab(f'{name}_spine', _rect(-0.093, -BOOK_H, -0.087, BOOK_H), frame, cover, out=-0.01, thick=0.02)
    yield _slab(f'{name}_pages', _rect(-0.087, 0.003 - BOOK_H, 0.086, BOOK_H - 0.003), frame, M.paper, out=-0.006, thick=0.012)
    yield _slab(f'{name}_pic', _rect(-0.055, -0.03, 0.055, 0.075), frame, picture, out=0.01, thick=0.002)
    yield _slab(f'{name}_title', _rect(-0.06, -0.085, 0.06, -0.06), frame, M.paper, out=0.01, thick=0.002)


def kidsBookshelf():
    # the sides: a curve from the foot up to the top, set back as it rises, a star cut through each
    side = G.smooth_outline([(-0.46, 0.0), (-0.06, 0.0), (-0.06, 0.2), (-0.15, 0.62), (-0.235, 0.9), (-0.46, 0.9)], passes=2)
    side = [(-0.46, 0.0), (-0.06, 0.0)] + [p for p in side if p[1] > 0.02 and p[0] > -0.44] + [(-0.46, 0.9)]
    for s in (-1, 1):
        F.extrude(f'side{s}', side, 0.025, (s * 0.3725, 0, 0), M.kshelf, plane='side', holes=[_star(-0.34, 0.78, 0.045)],
                  bevel=0.006, bevel_segments=1)
    F.box('back', 0.72, 0.9, 0.02, (0, 0.45, -0.45), M.kshelf, bevel=0.004, bevel_segments=1)
    F.extrude('top', G.rounded_rect(0.77, 0.26, 0.02, corner=2), 0.025, (0, 0.9125, -0.33), M.kshelf_top, plane='top',
              bevel=0.008, bevel_segments=2)
    F.box('plinth', 0.7, 0.06, 0.02, (0, 0.03, -0.07), M.kshelf_top, bevel=0.004, bevel_segments=1)
    lips = (M.toy_red, M.toy_blue, M.toy_yellow)
    covers = ((M.book_teal, M.toy_yellow), (M.book_rust, M.toy_blue), (M.book_blue, M.toy_red),
              (M.book_ochre, M.toy_green), (M.book_sage, M.toy_pink), (M.book_rose, M.toy_blue),
              (M.book_rust, M.toy_green), (M.book_blue, M.toy_yellow), (M.book_teal, M.toy_red))
    out = None
    for i, (y, z) in enumerate(SHELVES):
        F.box(f'shelf{i}', 0.72, 0.02, z + 0.44, (0, y, (z - 0.44) / 2), M.kshelf, bevel=0.003, bevel_segments=1)
        F.extrude(f'lip{i}', G.rounded_rect(0.72, 0.08, 0.025, corner=3), 0.015, (0, y + 0.04, z), lips[i], bevel=0.004,
                  bevel_segments=1)
        for j, x in enumerate((-0.22, 0.0, 0.22)):
            cover, picture = covers[3 * i + j]
            bz = z - 0.04
            by = y + 0.01 + BOOK_H * math.cos(math.radians(BOOK_TILT))
            bz -= BOOK_H * math.sin(math.radians(BOOK_TILT))
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
    and shrunk to a speck there, so `_draw_on` draws it from left to right."""
    with G.new_parts() as parts:
        for k, (pts, m) in enumerate(pieces):
            _slab(f'{name}{k}', pts, frame, m, out=0.0008 * k, thick=0.002)   # each over the last
    o, s, u, n = frame
    obj = G.join(parts, name, origin=F.at(*_add(o, _mul(s, left))))
    obj.scale = (0.001, 0.001, 0.001)
    return obj


def _draw_on(part, clip, seconds):
    """The drawing's clip: it grows along x from its left edge, its height and depth back to full in the
    first frames. Squeezed along x only, at rest it showed as a hairline of colours down the board."""
    G.grow(part, clip, seconds=seconds, axis='X')
    for axis in (1, 2):
        _clip(part, clip, 'scale', axis, [(1, 0.001), (3, 1.0), (last_frame(seconds), 1.0)])


def _rect(x0, y0, x1, y1):
    return [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]


def _tilted(y, z0, tilt, d=0.0):
    """Where a board leaning back by `tilt` degrees through (height 0.5, depth z0) is at height `y`, `d` in front of
    its middle plane."""
    t = math.radians(tilt)
    return z0 - (y - 0.5) * math.tan(t) + d / math.cos(t)


def kidsChalkboard():
    t = math.radians(EASEL_TILT)
    # two frames leaning together, a slate in each; their stiles run down to the floor as feet
    for k, (z, tilt) in enumerate(((0.11, -EASEL_TILT), (-0.11, EASEL_TILT))):
        F.extrude(f'frame{k}', G.rounded_rect(0.62, 0.96, 0.03, corner=2), 0.03, (0, 0.5, z), M.chalk_frame, rot=(tilt, 0, 0),
                  holes=[G.rounded_rect(0.52, 0.8, 0.006, corner=1)], bevel=0.006, bevel_segments=2)
        F.box(f'slate{k}', 0.522, 0.802, 0.036, (0, 0.5, z), M.slate, rot=(tilt, 0, 0))
    # the hinge bar across the top, a turned knob on each end
    F.cyl('top_bar', 0.015, 0.6, (0, 0.975, 0.0), M.chalk_frame, rot=(0, 90, 0), verts=10)
    for s in (-1, 1):
        G.revolve(f'bar_end{s}', [(0.0, 0.0), (0.017, 0.0), (0.019, 0.01), (0.014, 0.02), (0.0, 0.03)],
                  at=F.at(s * 0.3, 0.975, 0.0), material=M.chalk_frame, segments=10, rot=(0, -90 * s, 0))
        # a cord between the frames, so they can't spread
        zi = _tilted(0.45, 0.11, EASEL_TILT, -0.015)
        F.sweep(f'cord{s}', [(s * 0.285, 0.45, zi), (s * 0.285, 0.415, 0.0), (s * 0.285, 0.45, -zi)], 0.003, M.slate, verts=4,
                resolution=3)
    # the chalk tray across the front: chalk, a duster
    F.box('tray', 0.54, 0.02, 0.07, (0, 0.102, 0.25), M.chalk_frame, bevel=0.006, bevel_segments=2)
    F.box('tray_lip', 0.54, 0.03, 0.01, (0, 0.12, 0.285), M.chalk_frame, bevel=0.004, bevel_segments=1)
    for s in (-1, 1):
        F.box(f'tray_end{s}', 0.01, 0.025, 0.07, (s * 0.265, 0.12, 0.25), M.chalk_frame, bevel=0.003, bevel_segments=1)
    for k, (x, m) in enumerate(((-0.12, M.chalk_white), (0.0, M.chalk_yellow), (0.1, M.chalk_pink))):
        F.cyl(f'chalk{k}', 0.007, 0.08, (x, 0.12, 0.258), m, rot=(0, 90, 10 * (k - 1)), verts=6)
    F.box('duster', 0.08, 0.022, 0.035, (0.19, 0.123, 0.252), M.chalk_frame, bevel=0.005, bevel_segments=1)
    F.box('duster_felt', 0.078, 0.008, 0.033, (0.19, 0.11, 0.252), M.slate, bevel=0.002, bevel_segments=1)
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
    _draw_on(drawing, 'drawing', 2.5)
    return G.meshes()


def _leg(name, x, z0, z1):
    """An easel's leg from the floor at depth z0 up to the hinge at z1, 1.2 m up: a rounded batten."""
    F.sweep(name, [(x, 0.0, z0), (x, 1.2, z1)], None, M.easel, profile=G.rounded_rect(0.03, 0.036, 0.01, corner=2),
            up=(1, 0, 0))


def kidsEasel():
    t = math.radians(EASEL_TILT)
    for s in (-1, 1):
        _leg(f'leg_front{s}', s * 0.3, 0.3, 0.0)
        _leg(f'leg_back{s}', s * 0.3, -0.3, 0.0)
        F.box(f'board{s}', 0.58, 0.64, 0.01, (0, 0.82, s * 0.104), M.easel, rot=(-s * EASEL_TILT, 0, 0), bevel=0.003,
              bevel_segments=1)
        F.box(f'paper{s}', 0.56, 0.62, 0.012, (0, 0.82, s * 0.11), M.paper, rot=(-s * EASEL_TILT, 0, 0))
        # the tray: a shelf with a lip and ends
        F.box(f'tray{s}', 0.6, 0.03, 0.1, (0, 0.48, s * 0.21), M.easel, bevel=0.006, bevel_segments=2)
        F.box(f'tray_lip{s}', 0.6, 0.035, 0.012, (0, 0.5, s * 0.255), M.easel, bevel=0.004, bevel_segments=1)
        # a spring clip holding the paper, and a stay folding between the legs
        F.box(f'clip{s}', 0.12, 0.04, 0.02, (0, 1.12, s * 0.035), M.easel, rot=(-s * EASEL_TILT, 0, 0), bevel=0.006,
              bevel_segments=1)
        F.cyl(f'clip_spring{s}', 0.008, 0.1, (0, 1.142, s * 0.03), M.handle, rot=(0, 90, 0), verts=8)
        F.sweep(f'stay{s}', [(s * 0.3, 0.4, 0.2), (s * 0.3, 0.37, 0.0), (s * 0.3, 0.4, -0.2)], 0.005, M.handle, verts=5,
                resolution=2)
    F.sweep('top_bar', [(-0.315, 1.17, 0.0), (0.315, 1.17, 0.0)], 0.015, M.easel, verts=8)
    for s in (-1, 1):
        G.revolve(f'bar_end{s}', [(0.0, 0.0), (0.017, 0.0), (0.019, 0.008), (0.0, 0.016)], at=F.at(s * 0.315, 1.17, 0.0),
                  material=M.easel, segments=8, rot=(0, -90 * s, 0))
    # open paint pots on the front tray, each with its paint, a jar of brushes
    for k, m in enumerate((M.ball_red, M.ball_yellow, M.ball_blue, M.ball_green)):
        x = -0.2 + 0.1 * k
        _turned(f'pot{k}', x, 0.21, [(0.0, 0.0), (0.024, 0.0), (0.026, 0.045), (0.022, 0.045), (0.02, 0.006), (0.0, 0.006)],
                M.paper, segments=12, y=0.495)
        F.cyl(f'paint{k}', 0.021, 0.004, (x, 0.53, 0.21), m, verts=12)
    _turned('jar', 0.22, 0.21, [(0.0, 0.0), (0.028, 0.0), (0.03, 0.07), (0.026, 0.07), (0.024, 0.006), (0.0, 0.006)],
            M.water, segments=12, y=0.495)
    for k, (dx, dz, m) in enumerate(((-0.01, 0.0, M.paint_red), (0.008, -0.006, M.paint_blue), (0.0, 0.008, M.paint_yellow))):
        a, b = (0.22 + dx * 0.5, 0.505, 0.21 + dz * 0.5), (0.22 + dx * 3, 0.66, 0.21 + dz * 3)
        F.sweep(f'brush{k}', [a, b], 0.0035, M.easel, verts=5)
        F.cone(f'bristles{k}', 0.004, 0.0015, 0.022, (b[0], b[1] + 0.009, b[2]), m, verts=6)
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
    _draw_on(painting, 'painting', 2.5)
    return G.meshes()


# --- the family --------------------------------------------------------------------------------------------
TURN_DOWN = {'uses': {'sleep': 'turn_down', 'nap': 'turn_down'}}
LID = {'uses': {'play': 'lid_open'}}   # open while the child plays (the toys are out), shut when they're done

FAMILY = G.Family(
    'kids', M,
    [
        # KD1
        G.Model('crib', crib, size=(1.935, 1.008, 0.69), key='model.crib', centre=(-0.0125, -0.095),
                motions={'uses': {'feed': 'side_down', 'change': 'side_down', 'cuddle': 'side_down'},
                         # the mobile turns for an awake baby, while someone plays with them; not all night
                         'loops': {'play': 'mobile'}},
                paint=[('crib', 'Frame'), ('crib_bedding', 'Bedding'), ('mobile', 'Mobile')]),
        G.Model('kidsBed', kidsBed, size=(0.97, 0.764, 1.88), key='model.kidsBed', centre=(0.005, 0.01),
                motions=TURN_DOWN, paint=[('kids_frame', 'Frame'), ('kids_bedding', 'Bedding')]),
        G.Model('kidsBedCar', kidsBedCar, size=(0.985, 0.775, 1.916), key='model.kidsBedCar', centre=(0.0, 0.028),
                motions=TURN_DOWN, paint=[('car', 'Car body'), ('car_bedding', 'Bedding')]),
        G.Model('highChair', highChair, size=(0.533, 0.945, 0.554), key='model.highChair', centre=(0.0, 0.06),
                motions={'uses': {'sit': {'clip': 'tray_out', 'hold': 1.0, 'out': 1.2}}},
                paint=[('chair_white', 'Frame'), ('chair_cushion', 'Bedding')]),
        # KD2
        G.Model('toyBox', toyBox, size=(0.82, 0.51, 0.802), key='model.toyBox', centre=(0.0, -0.0389),
                motions=LID, paint=[('toybox', 'Body'), ('toybox_lid', 'Lid')]),
        G.Model('toyChest', toyChest, size=(0.846, 0.534, 0.513), key='model.toyChest', centre=(0.0, -0.1735),
                motions=LID, paint=[('chest', 'Body'), ('chest_trim', 'Trim')]),
        G.Model('dollHouse', dollHouse, size=(0.84, 0.908, 0.456), key='model.dollHouse', centre=(0.0, -0.202),
                motions={'uses': {'dolls': 'front_open'}}, paint=[('house', 'Walls'), ('roof', 'Roof')]),
        G.Model('rockingHorse', rockingHorse, size=(0.289, 0.825, 0.877), key='model.rockingHorse', centre=(0.0, 0.0426),
                motions={'loops': {'ride': 'rock'}, 'ride': 'rock'}, paint=[('horse', 'Body'), ('saddle', 'Trim')]),
        G.Model('playTent', playTent, size=(0.952, 1.574, 0.827), key='model.playTent', centre=(0.0, 0.0),
                motions={'uses': {'play': 'flap_open'}}, paint=[('canvas', 'Canvas'), ('poles', 'Poles')]),
        G.Model('ballPit', ballPit, size=(0.93, 0.47, 0.93), key='model.ballPit', centre=(0.0, 0.035),
                motions={'loops': {'jump': 'bounce'}}, paint=[('pit', 'Pit'), ('pit_rim', 'Rim')]),
        G.Model('trainTable', trainTable, size=(1.24, 0.68, 0.84), key='model.trainTable', centre=(0.0, 0.0),
                motions={'loops': {'trains': 'train'}}, paint=[('train_table', 'Table'), ('train', 'Train')]),
        G.Model('playMat', playMat, size=(0.92, 0.204, 0.92), key='model.playMat', centre=(0.0, 0.03),
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
                paint=[('kchair', 'Frame'), ('kchair_seat', 'Seat')]),
        G.Model('kidsBookshelf', kidsBookshelf, size=(0.77, 0.925, 0.4), key='model.kidsBookshelf', centre=(0.0, -0.23),
                motions={'uses': {'read': 'book_out'}}, paint=[('kshelf', 'Frame'), ('kshelf_top', 'Top')]),
        G.Model('kidsChalkboard', kidsChalkboard, size=(0.66, 0.956, 0.528), key='model.kidsChalkboard',
                centre=(0.0, 0.0259), motions={'uses': {'draw': 'drawing'}},
                paint=[('chalk_frame', 'Frame'), ('slate', 'Board')]),
        G.Model('kidsEasel', kidsEasel, size=(0.66, 1.208, 0.634), key='model.kidsEasel', centre=(0.0, 0.0),
                motions={'uses': {'paint': 'painting'}}, paint=[('easel', 'Easel')]),
    ],
    comment='Blender-built kids\' things (tools/art/blender/families/kids.py): the crib, the beds and the '
            'high chair, the toys, and the kids\' study and art things. Listed in the base manifest\'s '
            'packs. The materials map onto the game\'s finishes so a model costs one draw call per finish '
            'group.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

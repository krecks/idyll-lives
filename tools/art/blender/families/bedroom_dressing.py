# The bedroom's dressing things (BR4): the cheval mirror (`model.mirror`), the dressing table
# (`model.dressingTable`), the wall mirror (`model.mirrorWall`) and the standing mirror on a brass stand
# (`model.mirrorFloor`), over their primitive-part placeholders (`furniture.py` / `objects_extra.py`,
# `catalog.py`, `catalog2.py`), which stay the fallback. Notes: bedroom_dressing.md.
#
# Each mirror's glass is one flat piece of a mirror material (E6), so the game reflects the room in it
# live: upright, never tilted (the game lays the reflection on the glass's box). The dressing table is
# one still body plus its right-hand drawer, which slides out to the seated resident while they get
# ready. The wall mirror is on the wall layer: built at its real height, its back on the wall face.
# Built to the placeholders' sizes, in their numbers: the parts are placed in game axes (`F`).
#
# Run:  python3 -B tools/art/blender/build.py bedroom_dressing [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import bpy  # noqa: E402
import gamekit as G  # noqa: E402
from gamekit.shapes import _recalc_normals  # noqa: E402
from mathutils import Matrix, Vector  # noqa: E402

M = G.Palette(
    # the cheval mirror
    frame=('#A8754A', 'wood'),          # posts, feet, swing frame and crest (the colour part)
    backing=('#7A5234', 'wood'),        # the board behind the glass
    brass=('#B8925A', 'metal'),         # pivot knobs and axles
    mirror=('#DCE6EA', 'gloss', 'front'),       # glass that reflects to the front (+z)
    # the dressing table
    table=('#CDB08A', 'wood'),          # top, drawer cases, drawers (the main colour part)
    table_frame=('#B8925A', 'metal'),   # legs, the mirror's ring and stand
    pouf=('#E4B8AE', 'satin'),          # the pouf (plain, not fabric: a draw call saved for the drawer)
    inside=('#3A332C', 'satin'),        # the dark behind an open drawer
    perfume=('#E8B4C0', 'gloss'),
    lotion=('#E8E6E1', 'gloss'),
    gilt=('#C9A54A', 'metal'),          # bottle caps
    case=('#2A2C30', 'satin'),          # the jewellery box
    # the wall mirror
    wall_frame=('#2B2C2E', 'metal'),    # frame and ledge (the colour part)
    cup=('#9DC6CF', 'gloss'),
    brush=('#F2F1EE', 'gloss'),
    # the standing mirror
    stand=('#C9A06A', 'metal'),         # the brass frame and stand (the colour part)
    stand_back=('#3A332C', 'satin'),
)

F = G.GameFrame()


# --- shapes ---------------------------------------------------------------------------------------------

def _rounded(w, h, r, seg=3):
    """A rounded rectangle `w` x `h` (corner radius `r`) as [(u, v, nu, nv)]: points round it, anticlockwise,
    with their outward normals."""
    pts = []
    for k, (sx, sy) in enumerate(((1, 1), (-1, 1), (-1, -1), (1, -1))):
        cu, cv = sx * (w / 2 - r), sy * (h / 2 - r)
        for i in range(seg + 1):
            a = math.radians(90 * k + 90 * i / seg)
            pts.append((cu + r * math.cos(a), cv + r * math.sin(a), math.cos(a), math.sin(a)))
    return pts


def _ellipse(a, b, n=48):
    """An ellipse of half-axes `a` (across) and `b` (up) as [(u, v, nu, nv)]."""
    pts = []
    for k in range(n):
        t = 2 * math.pi * k / n
        nu, nv = b * math.cos(t), a * math.sin(t)
        length = math.hypot(nu, nv)
        pts.append((a * math.cos(t), b * math.sin(t), nu / length, nv / length))
    return pts


def _loop(name, outline, width, depth, centre, material, r=0.006):
    """A frame swept round `outline` (in the mirror's plane, about game `centre`), its cross-section a
    rounded rectangle `width` across (along the outline's normals) and `depth` deep (along game z)."""
    section = [(u, v) for u, v, _, _ in _rounded(width, depth, r, seg=2)]
    cx, cy, cz = centre
    verts = [F.at(cx + u + nu * o, cy + v + nv * o, cz + d) for u, v, nu, nv in outline for o, d in section]
    n, m = len(outline), len(section)
    faces = [(i * m + j, ((i + 1) % n) * m + j, ((i + 1) % n) * m + (j + 1) % m, i * m + (j + 1) % m)
             for i in range(n) for j in range(m)]
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.validate()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.collection.objects.link(obj)
    obj.data.materials.append(material)
    _recalc_normals(obj)
    for p in obj.data.polygons:
        p.use_smooth = True
    return obj


def _oval(name, w, h, t, at, material, smooth=False):
    """A flat oval `w` x `h`, `t` thick along game z, upright at game `at` (mirror glass, a back plate)."""
    return G.cylinder(name, w / 2, t, at=F.at(*at), material=material, scale=(1, h / w, 1), rot=(90, 0, 0),
                      verts=48, smooth=smooth)


def _turned(name, x, z, profile, material, segments=16):
    """A turned post at game (x, z): `profile` is [(radius, height)], bottom first."""
    return G.revolve(name, profile, at=F.at(x, 0, z), material=material, segments=segments)


# --- the cheval mirror: 1x1, a tall swing frame between two turned posts on feet -----------------------
# The placeholder's frame leans back 6°; this one hangs upright, so the reflection lies on the glass.

CHEVAL_Y = 0.95                         # the frame's middle and its pivots
CHEVAL_W, CHEVAL_H = 0.57, 1.57         # the frame's moulding, along its middle
CHEVAL_RAIL = 0.05                      # the moulding's width
POST_X = 0.35


def mirror():
    rail = CHEVAL_RAIL
    # the swing frame: moulding, the glass (front, +z) set into it, the board behind
    _loop('frame', _rounded(CHEVAL_W, CHEVAL_H, 0.03), rail, 0.045, (0, CHEVAL_Y, 0), M.frame, r=0.008)
    F.box('glass', CHEVAL_W - rail + 0.01, CHEVAL_H - rail + 0.01, 0.004, (0, CHEVAL_Y, 0.009), M.mirror)
    F.box('backing', CHEVAL_W - 0.02, CHEVAL_H - 0.02, 0.01, (0, CHEVAL_Y, -0.005), M.backing)
    top = CHEVAL_Y + CHEVAL_H / 2 + rail / 2
    F.box('crest', 0.66, 0.04, 0.055, (0, top + 0.02, 0), M.frame, bevel=0.012)
    _turned('crest_finial', 0, 0, [(0, top + 0.04), (0.014, top + 0.04), (0.01, top + 0.05), (0.024, top + 0.075),
                                   (0.012, top + 0.093), (0, top + 0.096)], M.frame)
    # two turned posts on their feet, a stretcher low between them
    for s in (-1, 1):
        x = s * POST_X
        F.box(f'foot{s}', 0.055, 0.04, 0.56, (x, 0.032, 0), M.frame, bevel=0.012)
        for z in (-0.25, 0.25):
            F.box(f'pad{s}{z}', 0.05, 0.012, 0.06, (x, 0.006, z), M.frame, bevel=0.003)
        _turned(f'post{s}', x, 0, [(0, 0.05), (0.026, 0.05), (0.026, 0.1), (0.018, 0.13), (0.015, 0.55),
                                   (0.02, 0.86), (0.026, 0.9), (0.026, 0.99), (0.012, 1.0), (0, 1.0)], M.frame)
        F.sphere(f'finial{s}', 0.026, (x, 1.022, 0), M.frame, segments=14, rings=8)
        # the pivot: an axle from the frame into the post, a brass knob outside
        G.cylinder(f'axle{s}', 0.008, 0.07, at=F.at(s * 0.335, CHEVAL_Y, 0), material=M.brass, rot=(0, 90, 0),
                   verts=10)
        G.cylinder(f'knob{s}', 0.024, 0.024, at=F.at(s * 0.388, CHEVAL_Y, 0), material=M.brass, rot=(0, 90, 0),
                   verts=20, bevel=0.005)
    G.rod('stretcher', F.at(-POST_X, 0.09, 0), F.at(POST_X, 0.09, 0), 0.013, material=M.frame, verts=12)
    G.join_all('mirror')
    return G.settle()


# --- the dressing table: 1x1, a kneehole table on brass legs, an oval mirror, a pouf -------------------
# Its mirror against the wall, the pouf at the front (depth +0.22, the seat 0.45 high: characters.ts SEATS,
# `back`: the sitter faces the mirror). The numbers below are as it was first built, facing the wall (the
# pouf at -0.22, the mirror at its far edge facing back at it); `dressingTable` turns it round at the end.

TOP_Y = 0.76            # the table top's surface
CASE_Y0 = 0.585         # the drawer cases' bottom
CASE_Z0, CASE_Z1 = 0.04, 0.41   # the cases' face to the seat, and their back
OVAL_Y, OVAL_Z = 1.165, 0.395   # the mirror's middle
DRAWER_X = 0.26         # the drawer that opens: on the seated resident's right (game +x: the game is left-handed)
DRAWER_OUT = 0.18
DRAWER_SECONDS = 0.8
DRAWER_HOLD = 3.0
TURN_BACK = 0.01        # turned round, moved back (Blender +y) to put the mirror's back on the wall face


def _drawer_front(name, x):
    """A drawer front on a case's face to the seat, with a wooden knob."""
    y = (CASE_Y0 + TOP_Y - 0.035) / 2
    F.box(f'{name}_front', 0.28, 0.118, 0.016, (x, y, CASE_Z0 - 0.009), M.table, bevel=0.005)
    F.sphere(f'{name}_knob', 0.013, (x, y, CASE_Z0 - 0.024), M.table, segments=12, rings=7)
    return y


def dressingTable():
    with G.new_parts() as body:
        F.box('top', 0.86, 0.035, 0.40, (0, TOP_Y - 0.0175, 0.22), M.table, bevel=0.008)
        for s in (-1, 1):
            F.box(f'case{s}', 0.30, TOP_Y - 0.035 - CASE_Y0, CASE_Z1 - CASE_Z0,
                  (s * 0.26, (CASE_Y0 + TOP_Y - 0.035) / 2, (CASE_Z0 + CASE_Z1) / 2), M.table, bevel=0.004)
        F.box('apron', 0.24, 0.05, 0.02, (0, TOP_Y - 0.06, 0.40), M.table, bevel=0.003)
        F.box('rail', 0.24, 0.03, 0.02, (0, TOP_Y - 0.05, 0.05), M.table, bevel=0.003)
        # the left drawer stays shut; behind the right one, the dark of the case
        _drawer_front('drawer_l', -DRAWER_X)
        F.box('cavity', 0.25, 0.1, 0.0015, (DRAWER_X, (CASE_Y0 + TOP_Y - 0.035) / 2, CASE_Z0 - 0.0012), M.inside)
        # four tapered brass legs under the cases, with feet
        for x in (-0.385, 0.385):
            for z in (0.07, 0.38):
                F.cone(f'leg{x}{z}', 0.011, 0.018, CASE_Y0 - 0.012, (x, (CASE_Y0 + 0.012) / 2, z), M.table_frame,
                       verts=12)
                F.cyl(f'shoe{x}{z}', 0.012, 0.012, (x, 0.006, z), M.table_frame, verts=12)
        # the oval mirror on a brass stand at the far edge, facing the seat
        _loop('ring', _ellipse(0.29, 0.35), 0.035, 0.03, (0, OVAL_Y, OVAL_Z), M.table_frame)
        _oval('glass', 0.545, 0.665, 0.003, (0, OVAL_Y, OVAL_Z - 0.0055), M.mirror)
        _oval('back', 0.56, 0.68, 0.008, (0, OVAL_Y, OVAL_Z + 0.005), M.table)
        F.box('stand_base', 0.70, 0.014, 0.05, (0, TOP_Y + 0.007, OVAL_Z), M.table_frame, bevel=0.004)
        for s in (-1, 1):
            G.rod(f'upright{s}', F.at(s * 0.33, TOP_Y, OVAL_Z), F.at(s * 0.33, OVAL_Y + 0.03, OVAL_Z), 0.009,
                  material=M.table_frame, verts=10)
            F.sphere(f'upright_top{s}', 0.014, (s * 0.33, OVAL_Y + 0.035, OVAL_Z), M.table_frame, segments=12, rings=7)
            G.rod(f'pivot{s}', F.at(s * 0.30, OVAL_Y, OVAL_Z), F.at(s * 0.35, OVAL_Y, OVAL_Z), 0.007,
                  material=M.table_frame, verts=10)
            F.sphere(f'pivot_knob{s}', 0.015, (s * 0.35, OVAL_Y, OVAL_Z), M.table_frame, segments=12, rings=7)
        # on the top: two perfume bottles and a jewellery box
        G.revolve('perfume', [(0, 0), (0.026, 0), (0.028, 0.012), (0.028, 0.055), (0.012, 0.07), (0.008, 0.075),
                              (0, 0.075)], at=F.at(0.30, TOP_Y, 0.30), material=M.perfume, segments=16)
        F.cyl('perfume_cap', 0.011, 0.025, (0.30, TOP_Y + 0.087, 0.30), M.gilt, verts=12)
        G.revolve('lotion', [(0, 0), (0.019, 0), (0.02, 0.09), (0.008, 0.1), (0, 0.1)],
                  at=F.at(0.355, TOP_Y, 0.34), material=M.lotion, segments=14)
        F.cyl('lotion_cap', 0.009, 0.022, (0.355, TOP_Y + 0.111, 0.34), M.gilt, verts=12)
        F.box('jewellery', 0.16, 0.036, 0.10, (-0.28, TOP_Y + 0.018, 0.18), M.case, bevel=0.006)
        F.box('jewellery_lid', 0.164, 0.012, 0.104, (-0.28, TOP_Y + 0.042, 0.18), M.case, bevel=0.004)
        F.box('jewellery_clasp', 0.02, 0.014, 0.004, (-0.28, TOP_Y + 0.032, 0.129), M.gilt, bevel=0.001)
        # the pouf: a round cushion on four brass feet, a button in its top
        G.revolve('pouf', [(0, 0.045), (0.19, 0.045), (0.205, 0.06), (0.208, 0.36), (0.214, 0.39), (0.212, 0.415),
                           (0.2, 0.437), (0.17, 0.449), (0.08, 0.45), (0, 0.447)],
                  at=F.at(0, 0, -0.22), material=M.pouf, segments=28)
        F.sphere('pouf_button', 0.011, (0, 0.446, -0.22), M.pouf, scale=(1, 1, 0.5), segments=10, rings=6)
        for k in range(4):
            a = math.radians(45 + 90 * k)
            F.cone(f'pouf_foot{k}', 0.01, 0.016, 0.045, (0.14 * math.cos(a), 0.0225, -0.22 + 0.14 * math.sin(a)),
                   M.table_frame, verts=10)
    body = G.join(body, 'body', origin=(0, 0, 0))

    # the right drawer: its front and the tray behind it, hidden in the case at rest
    with G.new_parts() as parts:
        y = _drawer_front('drawer', DRAWER_X)
        tray_z0, tray_z1 = CASE_Z0, CASE_Z0 + 0.3
        tz = (tray_z0 + tray_z1) / 2
        F.box('drawer_floor', 0.24, 0.01, 0.3, (DRAWER_X, y - 0.04, tz), M.table, bevel=0.002)
        for s in (-1, 1):
            F.box(f'drawer_side{s}', 0.01, 0.08, 0.3, (DRAWER_X + s * 0.115, y - 0.005, tz), M.table, bevel=0.002)
        F.box('drawer_back', 0.24, 0.08, 0.01, (DRAWER_X, y - 0.005, tray_z1 - 0.005), M.table, bevel=0.002)
    drawer = G.join(parts, 'drawer', origin=F.at(DRAWER_X, y, CASE_Z0))

    # Built facing the wall (as the placeholder was), then turned round about the footprint's middle: the
    # mirror's back on the wall face, the pouf at the front, where the resident sits facing the mirror.
    half = Matrix.Rotation(math.pi, 4, 'Z')
    for o in (body, drawer):
        o.data.transform(half)
        o.data.update()
        o.location = half @ o.location + Vector((0, TURN_BACK, 0))
    objs = G.settle(centre=False)
    # out to the seat: the game's +z, Blender's -y
    G.slide(drawer, 'drawer_open', (0, -DRAWER_OUT, 0), seconds=DRAWER_SECONDS)
    return objs


# --- the wall mirror: on the wall layer, a black metal frame and a small ledge --------------------------
# Its back on the wall face (depth -0.43), at the placeholder's height: the frame from 1.0 to 1.9 m.

WALL = -0.43
WM_Y, WM_W, WM_H, WM_RIM, WM_T = 1.45, 0.62, 0.9, 0.03, 0.035


def mirrorWall():
    zf = WALL + WM_T / 2
    _loop('frame', _rounded(WM_W - WM_RIM, WM_H - WM_RIM, 0.012), WM_RIM, WM_T, (0, WM_Y, zf), M.wall_frame,
          r=0.004)
    F.box('glass', WM_W - 2 * WM_RIM + 0.006, WM_H - 2 * WM_RIM + 0.006, 0.004, (0, WM_Y, WALL + WM_T - 0.008),
          M.mirror)
    F.box('backing', WM_W - 0.04, WM_H - 0.04, 0.012, (0, WM_Y, WALL + 0.008), M.wall_frame)
    # the ledge under it: a shelf with a lip on two brackets, a cup and a toothbrush
    shelf_y = 0.972
    F.box('shelf', 0.52, 0.012, 0.085, (0, shelf_y, WALL + 0.0425), M.wall_frame, bevel=0.003)
    F.box('lip', 0.52, 0.024, 0.006, (0, shelf_y + 0.006, WALL + 0.087), M.wall_frame, bevel=0.002)
    for s in (-1, 1):
        F.box(f'bracket{s}', 0.008, 0.03, 0.05, (s * 0.2, shelf_y - 0.021, WALL + 0.025), M.wall_frame, bevel=0.002)
    top = shelf_y + 0.006
    G.revolve('cup', [(0, 0), (0.021, 0), (0.024, 0.085), (0.021, 0.085), (0.018, 0.012), (0, 0.012)],
              at=F.at(-0.16, top, -0.368), material=M.cup, segments=16)
    G.rod('toothbrush', F.at(-0.165, top + 0.015, -0.368), F.at(-0.148, top + 0.125, -0.372), 0.0045,
          material=M.brush, verts=8)
    F.box('toothbrush_head', 0.012, 0.03, 0.01, (-0.147, top + 0.12, -0.366), M.brush, bevel=0.003)
    G.join_all('mirrorWall')
    return G.settle(centre=False, floor=False)


# --- the standing mirror: 1x1, a full-length mirror upright on a brass stand ----------------------------
# Where its placeholder stands: the frame a little behind the middle, the stand's strut behind it. The
# placeholder leans back 8°; this one stands upright on two sled feet, held by a strut from behind.

SM_Z = -0.05            # the frame's middle, in depth
SM_W, SM_H, SM_RIM, SM_T = 0.56, 1.66, 0.028, 0.035
SM_Y0 = 0.12            # the frame's bottom
FOOT_X, FOOT_Z0, FOOT_Z1 = 0.22, -0.38, 0.07


def mirrorFloor():
    y = SM_Y0 + SM_H / 2
    _loop('frame', _rounded(SM_W - SM_RIM, SM_H - SM_RIM, 0.03), SM_RIM, SM_T, (0, y, SM_Z), M.stand, r=0.005)
    F.box('glass', SM_W - 2 * SM_RIM + 0.006, SM_H - 2 * SM_RIM + 0.006, 0.004, (0, y, SM_Z + SM_T / 2 - 0.0105),
          M.mirror)
    F.box('backing', SM_W - 0.03, SM_H - 0.03, 0.012, (0, y, SM_Z - 0.008), M.stand_back)
    # the stand: two sled feet, a clamp up from each to the frame, a rear bar and a strut to the frame's back
    for s in (-1, 1):
        x = s * FOOT_X
        F.box(f'foot{s}', 0.024, 0.02, FOOT_Z1 - FOOT_Z0, (x, 0.01, (FOOT_Z0 + FOOT_Z1) / 2), M.stand, bevel=0.006)
        F.box(f'clamp{s}', 0.026, SM_Y0 + 0.04 - 0.02, SM_T + 0.012, (x, (0.02 + SM_Y0 + 0.04) / 2, SM_Z), M.stand,
              bevel=0.005)
    G.rod('rear_bar', F.at(-FOOT_X, 0.012, FOOT_Z0 + 0.02), F.at(FOOT_X, 0.012, FOOT_Z0 + 0.02), 0.012,
          material=M.stand, verts=12)
    back = SM_Z - SM_T / 2
    G.rod('strut', F.at(0, 0.02, FOOT_Z0 + 0.02), F.at(0, 1.40, back - 0.006), 0.011, material=M.stand, verts=12)
    F.box('strut_plate', 0.07, 0.06, 0.008, (0, 1.40, back - 0.004), M.stand, bevel=0.002)
    G.join_all('mirrorFloor')
    return G.settle(centre=False)


FAMILY = G.Family(
    'bedroom_dressing', M,
    [G.Model('mirror', mirror, size=(0.80, 1.856, 0.56), key='model.mirror', paint=[('frame', 'Frame')]),
     G.Model('dressingTable', dressingTable, size=(0.86, 1.533, 0.858), key='model.dressingTable',
             centre=(0, -0.003),
             # getReady (the `wash` tag) slides the right drawer out to the seat, and back after a while
             motions={'uses': {'getReady': 'drawer_open'}, 'hold': DRAWER_HOLD},
             paint=[('table', 'Table'), ('table_frame', 'Frame')]),
     G.Model('mirrorWall', mirrorWall, size=(0.62, 0.97, 0.09), key='model.mirrorWall', centre=(0, -0.385),
             floor=False, paint=[('wall_frame', 'Frame')]),
     G.Model('mirrorFloor', mirrorFloor, size=(0.56, 1.78, 0.45), key='model.mirrorFloor', centre=(0, -0.155),
             paint=[('stand', 'Frame')])],
    comment='Blender-built bedroom dressing models (tools/art/blender/families/bedroom_dressing.py): the cheval '
            'mirror, the dressing table, the wall mirror and the standing mirror. Replaces the primitive-part '
            'placeholders by key; listed in the base manifest\'s packs. The placeholder is kept as a fallback if a '
            'file fails to load. Each mirror\'s glass maps to a `mirror` material, so the game reflects the room in '
            'it live. The dressing table slides its drawer out (`drawer_open`) while a resident gets ready.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)

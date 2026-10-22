# The bedroom's dressing things (BR4): the cheval mirror (`model.mirror`), the dressing table
# (`model.dressingTable`), the wall mirror (`model.mirrorWall`) and the standing mirror on a brass stand
# (`model.mirrorFloor`). Notes: bedroom_dressing.md.
#
# Each mirror's glass is one flat piece of a mirror material (E6), so the game reflects the room in it
# live: upright, never tilted (the game lays the reflection on the glass's box). The dressing table is
# one still body plus its right-hand drawer, which slides out to the seated resident while they get
# ready. The wall mirror is on the wall layer: built at its real height, its back on the wall face.
# Built to the old parts' sizes, in their numbers: the parts are placed in game axes (`F`).
#
# Run:  cargo art bedroom_dressing [--preview]
import math
import os
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
import bmesh  # noqa: E402
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
    table_frame=('#B8925A', 'metal'),   # legs, the mirror's ring and stand, the drawer knobs
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
    bristle=('#DDEFF2', 'matte'),       # the toothbrush's bristles
    paste=('#E9E6EF', 'gloss'),         # a tube of toothpaste
    paste_cap=('#C65D52', 'gloss'),
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


def _moulding(w, d):
    """A frame's cross-section [(o, d)]: `w` wide along the outline's normal (o: - in, + out) and `d` deep (+ to the
    front): a flat back, a rounded outer edge, a cove and a bead, and a lip falling to the glass, standing well
    in front of it (the glass is set in behind)."""
    hw, hd = w / 2, d / 2
    return [(-hw, -hd), (hw, -hd), (hw, hd * 0.2), (hw * 0.85, hd * 0.75), (hw * 0.5, hd), (hw * 0.15, hd * 0.92),
            (-hw * 0.05, hd * 0.8), (-hw * 0.3, hd * 0.98), (-hw * 0.6, hd * 0.92), (-hw * 0.85, hd * 0.82),
            (-hw, hd * 0.75)]


def _loop(name, outline, section, centre, material, facing=1):
    """A frame swept round `outline` (in the mirror's plane, about game `centre`), its cross-section `section`
    [(o, d)]: o along the outline's normals, d along game z (`facing` -1: its front to -z)."""
    cx, cy, cz = centre
    verts = [F.at(cx + u + nu * o, cy + v + nv * o, cz + facing * d) for u, v, nu, nv in outline for o, d in section]
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
    obj.data.shade_smooth()
    obj.data.set_sharp_from_angle(angle=math.radians(50))
    return obj


def _pane(name, w, h, t, at, material, cuts=5, inset=0.012):
    """Mirror glass: a flat box `w` x `h`, `t` thick at game `at`, its faces gridded, the grid's outer lines `inset`
    in from its edges, so the shading baked at its vertices (shade.py) darkens only that rim, set into the frame,
    not the whole pane (with only its corners, buried in the frame, it baked grey all over)."""
    obj = F.box(name, w, h, t, at, material)
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bmesh.ops.subdivide_edges(bm, edges=[e for e in bm.edges if e.calc_length() > t * 2], cuts=cuts, use_grid_fill=True)

    def remap(c, half):
        # the first grid line in from each edge moves out to `inset`, the lines between spread evenly
        k = abs(c) / half
        step = 1 / (cuts + 1)
        a = 1 - inset / half                    # where the first line in from the edge goes, as a fraction of `half`
        if k <= 1 - 2 * step + 1e-9:
            k = k * a / (1 - 2 * step)
        else:
            k = a + (k - (1 - 2 * step)) / (2 * step) * (1 - a)
        return math.copysign(k * half, c)
    for v in bm.verts:
        v.co.x = remap(v.co.x, w / 2)
        v.co.z = remap(v.co.z, h / 2)
    bm.to_mesh(obj.data)
    bm.free()
    return obj


def _oval(name, w, h, t, at, material, rings=(0.96, 0.62, 0.3)):
    """A flat oval `w` x `h`, `t` thick along game z, upright at game `at` (mirror glass, a back plate), its faces
    in rings, so the shading baked at its vertices darkens it only at its rim."""
    r = w / 2
    prof = [(0, -t / 2), (r, -t / 2), (r, t / 2)] + [(r * k, t / 2) for k in rings] + [(0, t / 2)]
    return G.revolve(name, prof, at=F.at(*at), material=material, segments=48, scale=(1, h / w, 1), rot=(90, 0, 0))


def _turned(name, x, z, profile, material, segments=16):
    """A turned post at game (x, z): `profile` is [(radius, height)], bottom first."""
    return G.revolve(name, profile, at=F.at(x, 0, z), material=material, segments=segments)


def _side_knob(name, x, y, z, r, s, material):
    """A turned knob on the side facing game x's sign `s`, its rose at game (x, y, z)."""
    k = G.kit.knob(F, name, x, y, z, r, material)
    k.rotation_euler = (0, math.radians(-90 * s), 0)
    return k


# --- the cheval mirror: 1x1, a tall swing frame between two turned posts on feet -----------------------
# The old part's frame leaned back 6°; this one hangs upright, so the reflection lies on the glass.

CHEVAL_Y = 0.95                         # the frame's middle and its pivots
CHEVAL_W, CHEVAL_H = 0.57, 1.57         # the frame's moulding, along its middle
CHEVAL_RAIL = 0.05                      # the moulding's width
POST_X = 0.35


def mirror():
    rail = CHEVAL_RAIL
    # the swing frame: a moulding with a cove and a bead, the glass (front, +z) set in behind its lip, the board
    # behind
    _loop('frame', _rounded(CHEVAL_W, CHEVAL_H, 0.03, seg=4), _moulding(rail, 0.045), (0, CHEVAL_Y, 0), M.frame)
    _pane('glass', CHEVAL_W - rail + 0.01, CHEVAL_H - rail + 0.01, 0.004, (0, CHEVAL_Y, 0.009), M.mirror)
    F.box('backing', CHEVAL_W - 0.02, CHEVAL_H - 0.02, 0.01, (0, CHEVAL_Y, -0.005), M.backing)
    top = CHEVAL_Y + CHEVAL_H / 2 + rail / 2
    # a crest over it, rising to the finial in a shallow arch with a scroll at each end
    crest = [(0.33, top + 0.0), (0.33, top + 0.03), (0.3, top + 0.04)] + \
        [(0.27 - 0.54 * i / 12, top + 0.04 + 0.012 * math.sin(math.pi * i / 12)) for i in range(13)] + \
        [(-0.3, top + 0.04), (-0.33, top + 0.03), (-0.33, top + 0.0)]
    F.extrude('crest', crest[::-1], 0.05, (0, 0, 0), M.frame, bevel=0.008, bevel_segments=2)
    _turned('crest_finial', 0, 0, [(0, top + 0.05), (0.016, top + 0.05), (0.01, top + 0.058), (0.024, top + 0.075),
                                   (0.012, top + 0.093), (0, top + 0.096)], M.frame)
    # two turned posts on their feet, a turned stretcher low between them
    for s in (-1, 1):
        x = s * POST_X
        foot = [(0.28, 0.0), (0.28, 0.02), (0.24, 0.045), (0.1, 0.052), (-0.1, 0.052), (-0.24, 0.045), (-0.28, 0.02),
                (-0.28, 0.0), (-0.2, 0.0), (-0.18, 0.012)] + \
            [(-0.16 + 0.32 * i / 8, 0.012 + 0.008 * math.sin(math.pi * i / 8)) for i in range(9)] + \
            [(0.18, 0.012), (0.2, 0.0)]
        F.extrude(f'foot{s}', foot[::-1], 0.052, (x, 0, 0), M.frame, plane='side', bevel=0.008, bevel_segments=2)
        _turned(f'post{s}', x, 0, [(0, 0.05), (0.026, 0.05), (0.028, 0.07), (0.026, 0.1), (0.018, 0.125),
                                   (0.022, 0.15), (0.015, 0.18), (0.015, 0.5), (0.019, 0.55), (0.016, 0.6),
                                   (0.02, 0.86), (0.027, 0.89), (0.027, 0.92), (0.022, 0.94), (0.027, 0.97),
                                   (0.027, 0.99), (0.012, 1.0), (0, 1.0)], M.frame)
        _turned(f'finial{s}', x, 0, [(0, 0.996), (0.013, 0.996), (0.009, 1.004), (0.024, 1.02), (0.026, 1.03),
                                     (0.02, 1.042), (0.008, 1.048), (0, 1.049)], M.frame, segments=14)
        # the pivot: an axle from the frame into the post, a turned brass knob outside it
        G.cylinder(f'axle{s}', 0.008, 0.07, at=F.at(s * 0.335, CHEVAL_Y, 0), material=M.brass, rot=(0, 90, 0),
                   verts=10)
        _side_knob(f'knob{s}', s * 0.374, CHEVAL_Y, 0, 0.014, s, M.brass)
    F.sweep('stretcher', [(-POST_X, 0.09, 0), (-0.2, 0.09, 0), (-0.06, 0.09, 0), (0, 0.09, 0), (0.06, 0.09, 0),
                          (0.2, 0.09, 0), (POST_X, 0.09, 0)], 0.013, M.frame, verts=12,
            scale=[1, 0.8, 1.2, 0.85, 1.2, 0.8, 1])
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
    """A drawer front on a case's face to the seat, its edges rounded, with a turned brass knob."""
    y = (CASE_Y0 + TOP_Y - 0.035) / 2
    F.soft(f'{name}_front', 0.28, 0.118, 0.016, (x, y, CASE_Z0 - 0.009), M.table, round=0.005, round_segments=1)
    k = G.kit.knob(F, f'{name}_knob', x, y, CASE_Z0 - 0.017, 0.011, M.table_frame)
    k.rotation_euler = (math.radians(-90), 0, 0)   # facing the seat (-z, as built)
    return y


def _bottle(name, x, z, y0, h, r, material, cap, square=4.0):
    """A perfume bottle at game (x, z) on height y0: a squarish faceted body, shoulders, a neck and a gilt cap."""
    bx, by = F.at(x, 0, z)[:2]
    secs = [G.ring(r * k, r * k * 0.7, y0 + h * t, 16, square=square, at=(bx, by))
            for t, k in ((0.0, 0.92), (0.03, 1.0), (0.66, 1.0), (0.76, 0.75), (0.8, 0.36), (0.88, 0.34))]
    G.loft(name, secs, material)
    G.revolve(f'{name}_cap', [(0, 0), (r * 0.42, 0), (r * 0.46, h * 0.05), (r * 0.3, h * 0.35), (r * 0.2, h * 0.4),
                              (0, h * 0.41)], at=F.at(x, y0 + h * 0.86, z), material=cap, segments=12)


def dressingTable():
    with G.new_parts() as body:
        F.soft('top', 0.86, 0.035, 0.40, (0, TOP_Y - 0.0175, 0.22), M.table, round=0.012, round_segments=2)
        for s in (-1, 1):
            F.box(f'case{s}', 0.30, TOP_Y - 0.035 - CASE_Y0, CASE_Z1 - CASE_Z0,
                  (s * 0.26, (CASE_Y0 + TOP_Y - 0.035) / 2, (CASE_Z0 + CASE_Z1) / 2), M.table, bevel=0.004,
                  bevel_segments=1)
        F.box('apron', 0.24, 0.05, 0.02, (0, TOP_Y - 0.06, 0.40), M.table, bevel=0.003, bevel_segments=1)
        # the kneehole's rail to the seat, its lower edge arched
        edge = [(0.12 - 0.24 * i / 10, TOP_Y - 0.065 + 0.012 * math.sin(math.pi * i / 10)) for i in range(11)]
        F.extrude('rail', [(0.12, TOP_Y - 0.035), (-0.12, TOP_Y - 0.035)] + edge[::-1], 0.02, (0, 0, 0.05), M.table,
                  bevel=0.003, bevel_segments=1)
        # the left drawer stays shut; behind the right one, the dark of the case
        _drawer_front('drawer_l', -DRAWER_X)
        F.box('cavity', 0.25, 0.1, 0.0015, (DRAWER_X, (CASE_Y0 + TOP_Y - 0.035) / 2, CASE_Z0 - 0.0012), M.inside)
        # four tapered brass legs under the cases, each on a turned shoe
        for x in (-0.385, 0.385):
            for z in (0.07, 0.38):
                F.cone(f'leg{x}{z}', 0.011, 0.018, CASE_Y0 - 0.03, (x, (CASE_Y0 + 0.03) / 2, z), M.table_frame,
                       verts=12)
                G.revolve(f'shoe{x}{z}', [(0, 0), (0.015, 0), (0.016, 0.006), (0.011, 0.012), (0.012, 0.024),
                                          (0.0105, 0.031), (0, 0.031)], at=F.at(x, 0, z), material=M.table_frame,
                          segments=12)
        # the oval mirror on a brass stand at the far edge, facing the seat
        _loop('ring', _ellipse(0.29, 0.35), _moulding(0.035, 0.03), (0, OVAL_Y, OVAL_Z), M.table_frame, facing=-1)
        _oval('glass', 0.545, 0.665, 0.003, (0, OVAL_Y, OVAL_Z - 0.0055), M.mirror)
        _oval('back', 0.56, 0.68, 0.008, (0, OVAL_Y, OVAL_Z + 0.005), M.table, rings=())
        F.soft('stand_base', 0.70, 0.014, 0.05, (0, TOP_Y + 0.007, OVAL_Z), M.table_frame, round=0.005,
               round_segments=1)
        for s in (-1, 1):
            G.kit.tube(F, f'upright{s}', [(s * 0.33, TOP_Y, OVAL_Z), (s * 0.33, OVAL_Y + 0.03, OVAL_Z)], 0.009,
                       M.table_frame, verts=10, round=1)
            _turned(f'upright_top{s}', s * 0.33, OVAL_Z, [(0, OVAL_Y + 0.025), (0.011, OVAL_Y + 0.025),
                                                          (0.008, OVAL_Y + 0.032), (0.014, OVAL_Y + 0.042),
                                                          (0.012, OVAL_Y + 0.054), (0.004, OVAL_Y + 0.06),
                                                          (0, OVAL_Y + 0.061)], M.table_frame, segments=12)
            G.rod(f'pivot{s}', F.at(s * 0.30, OVAL_Y, OVAL_Z), F.at(s * 0.34, OVAL_Y, OVAL_Z), 0.007,
                  material=M.table_frame, verts=10)
            _side_knob(f'pivot_knob{s}', s * 0.339, OVAL_Y, OVAL_Z, 0.011, s, M.table_frame)
        # on the top: two perfume bottles and a jewellery box
        _bottle('perfume', 0.30, 0.30, TOP_Y, 0.086, 0.028, M.perfume, M.gilt)
        _bottle('lotion', 0.355, 0.34, TOP_Y, 0.11, 0.02, M.lotion, M.gilt, square=2.2)
        F.soft('jewellery', 0.16, 0.036, 0.10, (-0.28, TOP_Y + 0.018, 0.18), M.case, round=0.006, round_segments=1)
        F.soft('jewellery_lid', 0.164, 0.014, 0.104, (-0.28, TOP_Y + 0.042, 0.18), M.case, round=0.005,
               round_segments=1)
        F.box('jewellery_clasp', 0.02, 0.014, 0.004, (-0.28, TOP_Y + 0.032, 0.129), M.gilt, bevel=0.001)
        # the pouf: a round cushion, piped round its top and foot, buttoned in the middle, on four brass feet
        G.revolve('pouf', [(0, 0.045), (0.19, 0.045), (0.205, 0.06), (0.208, 0.36), (0.214, 0.39), (0.212, 0.415),
                           (0.2, 0.437), (0.17, 0.449), (0.08, 0.45), (0, 0.447)],
                  at=F.at(0, 0, -0.22), material=M.pouf, segments=28)
        for k, (r, y) in enumerate(((0.21, 0.405), (0.2, 0.055))):
            F.sweep(f'pouf_piping{k}', [(r * math.cos(math.tau * i / 36), y, -0.22 + r * math.sin(math.tau * i / 36))
                                        for i in range(36)], 0.006, M.pouf, verts=5, closed=True)
        F.sphere('pouf_button', 0.012, (0, 0.444, -0.22), M.pouf, scale=(1, 1, 0.5), segments=10, rings=6)
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

    # Built facing the wall (as the old part was), then turned round about the footprint's middle: the
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
# Its back on the wall face (depth -0.43), at the old part's height: the frame from 1.0 to 1.9 m.

WALL = -0.43
WM_Y, WM_W, WM_H, WM_RIM, WM_T = 1.45, 0.62, 0.9, 0.03, 0.035


def mirrorWall():
    zf = WALL + WM_T / 2
    _loop('frame', _rounded(WM_W - WM_RIM, WM_H - WM_RIM, 0.012), _moulding(WM_RIM, WM_T), (0, WM_Y, zf),
          M.wall_frame)
    _pane('glass', WM_W - 2 * WM_RIM + 0.006, WM_H - 2 * WM_RIM + 0.006, 0.004, (0, WM_Y, WALL + WM_T - 0.008),
          M.mirror)
    F.box('backing', WM_W - 0.04, WM_H - 0.04, 0.012, (0, WM_Y, WALL + 0.008), M.wall_frame)
    # the ledge under it: a tray with a rolled lip, on two bent brackets; a cup, a toothbrush and the toothpaste
    shelf_y = 0.972
    tray = [(0.0, -0.0425), (0.0, 0.045), (0.006, 0.0472), (0.022, 0.0468), (0.026, 0.0428), (0.024, 0.0388),
            (0.012, 0.0398), (0.012, -0.0425)]
    F.sweep('shelf', [(-0.26, shelf_y - 0.006, WALL + 0.0425), (0.26, shelf_y - 0.006, WALL + 0.0425)], None,
            M.wall_frame, profile=[(u, -v) for u, v in tray], up=(0, 0, 1), angle=35)
    for s in (-1, 1):
        F.sweep(f'bracket{s}', G.kit.fillet([(s * 0.2, shelf_y - 0.036, WALL + 0.002), (s * 0.2, shelf_y - 0.012, WALL + 0.002),
                                             (s * 0.2, shelf_y - 0.012, WALL + 0.06)], 0.016, 3),
                None, M.wall_frame, profile=G.rounded_rect(0.004, 0.012, 0.0015, corner=1), up=(0, 1, 0))
    top = shelf_y + 0.006
    G.revolve('cup', [(0, 0), (0.019, 0), (0.021, 0.004), (0.024, 0.082), (0.0245, 0.085), (0.0215, 0.085),
                      (0.018, 0.012), (0, 0.012)], at=F.at(-0.16, top, -0.368), material=M.cup, segments=16)
    F.sweep('toothbrush', [(-0.165, top + 0.015, -0.368), (-0.16, top + 0.07, -0.369), (-0.152, top + 0.11, -0.371),
                           (-0.149, top + 0.135, -0.372)], 0.0045, M.brush, verts=6, resolution=2,
            scale=[1, 0.8, 0.75, 0.9])
    F.box('toothbrush_head', 0.01, 0.028, 0.008, (-0.148, top + 0.122, -0.3665), M.brush, bevel=0.003,
          bevel_segments=1)
    F.box('toothbrush_bristles', 0.008, 0.024, 0.01, (-0.148, top + 0.122, -0.3585), M.bristle, bevel=0.002,
          bevel_segments=1)
    bx, by = F.at(0.12, 0, -0.385)[:2]
    G.loft('paste', [G.ring(rx, ry, top + yy, 12, at=(bx + 0.0, by)) for yy, rx, ry in
                     ((0.004, 0.002, 0.016), (0.01, 0.01, 0.016), (0.11, 0.016, 0.015), (0.125, 0.012, 0.012))],
           M.paste, rot=(0, 0, 0))
    G.revolve('paste_cap', [(0, 0), (0.011, 0), (0.011, 0.018), (0, 0.018)], at=F.at(0.12, top + 0.125, -0.385),
              material=M.paste_cap, segments=10)
    G.join_all('mirrorWall')
    return G.settle(centre=False, floor=False)


# --- the standing mirror: 1x1, a full-length mirror upright on a brass stand ----------------------------
# Where the game places it: the frame a little behind the middle, the stand's strut behind it. The
# old part leaned back 8°; this one stands upright on two sled feet, held by a strut from behind.

SM_Z = -0.05            # the frame's middle, in depth
SM_W, SM_H, SM_RIM, SM_T = 0.56, 1.66, 0.028, 0.035
SM_Y0 = 0.12            # the frame's bottom
FOOT_X, FOOT_Z0, FOOT_Z1 = 0.22, -0.38, 0.07


def mirrorFloor():
    y = SM_Y0 + SM_H / 2
    _loop('frame', _rounded(SM_W - SM_RIM, SM_H - SM_RIM, 0.03, seg=4), _moulding(SM_RIM, SM_T), (0, y, SM_Z),
          M.stand)
    _pane('glass', SM_W - 2 * SM_RIM + 0.006, SM_H - 2 * SM_RIM + 0.006, 0.004, (0, y, SM_Z + SM_T / 2 - 0.0105),
          M.mirror)
    F.box('backing', SM_W - 0.03, SM_H - 0.03, 0.012, (0, y, SM_Z - 0.008), M.stand_back)
    # the stand: two sled feet turned up at the toe, on glides; a clamp up from each to the frame, tightened by a
    # knob; a rear bar, and a strut folding at a hinge up to a plate on the frame's back
    for s in (-1, 1):
        x = s * FOOT_X
        F.sweep(f'foot{s}', G.kit.fillet([(x, 0.012, FOOT_Z0), (x, 0.012, FOOT_Z1 - 0.03), (x, 0.018, FOOT_Z1)],
                                         0.03, 3), None, M.stand,
                profile=G.rounded_rect(0.016, 0.024, 0.006, corner=2), up=(0, 0, 1))
        for z in (FOOT_Z0 + 0.03, FOOT_Z1 - 0.05):
            F.cyl(f'glide{s}{z}', 0.009, 0.006, (x, 0.003, z), M.stand_back, verts=10)
        F.soft(f'clamp{s}', 0.026, SM_Y0 + 0.04 - 0.02, SM_T + 0.012, (x, (0.02 + SM_Y0 + 0.04) / 2, SM_Z), M.stand,
               round=0.006, round_segments=1)
        k = G.kit.knob(F, f'clamp_knob{s}', x, SM_Y0 + 0.01, SM_Z - SM_T / 2 - 0.006, 0.01, M.stand)
        k.rotation_euler = (math.radians(-90), 0, 0)   # out to the back
    F.sweep('rear_bar', [(-FOOT_X, 0.012, FOOT_Z0 + 0.02), (FOOT_X, 0.012, FOOT_Z0 + 0.02)], 0.012, M.stand, verts=12)
    back = SM_Z - SM_T / 2
    mid = (0, 0.71, (FOOT_Z0 + 0.02 + back - 0.006) / 2)
    G.kit.tube(F, 'strut_low', [(0, 0.02, FOOT_Z0 + 0.02), mid], 0.011, M.stand, verts=12, round=1)
    G.kit.tube(F, 'strut_high', [mid, (0, 1.40, back - 0.006)], 0.01, M.stand, verts=12, round=1)
    G.cylinder('strut_hinge', 0.014, 0.034, at=F.at(*mid), material=M.stand, rot=(0, 90, 0), verts=12)
    F.soft('strut_plate', 0.07, 0.06, 0.008, (0, 1.40, back - 0.004), M.stand, round=0.003, round_segments=1)
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
    comment='Blender-built bedroom dressing models (tools/art/blender/families/bedroom_dressing.py): the '
            'cheval mirror, the dressing table, the wall mirror and the standing mirror. Listed in the '
            'base manifest\'s packs. Each mirror\'s glass maps to a `mirror` material, so the game reflects '
            'the room in it live. The dressing table slides its drawer out (`drawer_open`) while a '
            'resident gets ready.',
)

if __name__ == '__main__':
    FAMILY.run(sys.argv)
